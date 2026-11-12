"""M7e explicit player choices, stat budgets, variable/sparse roster; synthetic TCP only."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import signal
import socket
import subprocess
import sys
import time

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from server import probe_codec as codec
from server.room_title import _encode_bytes, _decode_bytes
from tools.rust_account_fixture import AccountFixture
from tools.test_rust_foundation import free_port
from tools.test_rust_managed_ownership import TraceClient as Client


def population(body):
    count = codec.read_fields(body[:2], [2])[0]
    if not count:
        assert len(body) == 2
        return {}
    blob = _decode_bytes(body[6:])
    assert codec.read_fields(body[2:6], [4]) == [len(blob)]
    assert blob[0] == 24 and int.from_bytes(blob[1:5], 'little') == len(blob)-5 and blob[5] == 0
    pos = 6
    rows = {}
    for _ in range(count):
        session = int.from_bytes(blob[pos:pos+2], 'little')
        room_id = int.from_bytes(blob[pos+2:pos+6], 'little')
        n = int.from_bytes(blob[pos+6:pos+10], 'little')
        title = blob[pos+10:pos+10+n]
        tail = blob[pos+10+n:pos+32+n]
        rows[(session, room_id)] = (tail[5], tail[7], title.hex())
        pos += 32+n
    assert pos == len(blob)
    return rows


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['binary', 'admin', 'config', 'baseline', 'out']:
        parser.add_argument('--'+name, type=Path, required=True)
    parser.add_argument('--active-departure', action='store_true')
    args = parser.parse_args()
    root = args.out.resolve()
    root.mkdir(parents=True, exist_ok=False, mode=0o700)
    cfg = json.loads(args.config.read_text())
    baseline = json.loads(args.baseline.read_text())['diagnostic_two_lobby']['fixed_room']
    two = cfg['diagnostic_two_lobby']
    assert len(two['actors']) == 4
    ports = set()
    while len(ports) < 3:
        ports.add(free_port())
    login, wcp, channel = sorted(ports)
    cfg.update(bind='127.0.0.1', allowed_peers=[f'127.0.0.{n+2}' for n in range(4)], ports=sorted(ports))
    two.update(login_port=login, login_candidate_ports=[login, wcp], wcp_ports=sorted(ports),
               channel_port=channel, managed_rooms=dict(max_rooms=2, player_choices=True, player_round=dict(characters=[0,1])))
    for actor, peer in zip(two['actors'], cfg['allowed_peers']):
        actor['peer'] = peer
    account = AccountFixture(args.admin, root, cfg)
    config = root/'config.json'
    config.write_text(json.dumps(cfg, indent=2)+'\n')
    files = [args.binary, args.admin, config, args.config, args.baseline, Path(__file__), Path('tools/rust_account_fixture.py'), Path('tools/test_rust_managed_ownership.py'),
             *Path('rust/crates').rglob('*.rs'), *Path('rust/crates').rglob('*.sql'), Path('rust/Cargo.lock')]
    def fixtures(value):
        if isinstance(value, dict):
            if {'path', 'bytes', 'sha256'} <= value.keys():
                p = Path(value['path'])
                assert p.stat().st_size == value['bytes'] and hashlib.sha256(p.read_bytes()).hexdigest() == value['sha256']
                files.append(p)
            for v in value.values():
                fixtures(v)
        elif isinstance(value, list):
            for v in value:
                fixtures(v)
    fixtures(cfg)
    fixtures(baseline)
    frozen = root/'frozen'
    frozen.mkdir()
    pins = []
    for index, source in enumerate(dict.fromkeys(files)):
        dest = frozen/f'{index:03d}-{source.name}'
        shutil.copyfile(source, dest)
        pins.append(dict(path=str(source), retained_path=str(dest), bytes=source.stat().st_size,
                         sha256=hashlib.sha256(source.read_bytes()).hexdigest()))
    (root/'preflight.json').write_text(json.dumps(dict(files=pins, limits='Synthetic TCP, real SQLite; no native client. Admin/session grants are declared runtime inputs.'), indent=2)+'\n')
    hello = Path(cfg['bootstrap']['path']).read_bytes()
    header = Path(two['actors'][0]['profile']['own_header']['path']).read_bytes()[:37]
    clients = []
    trace=[]
    Client.trace=trace
    cases = []
    observations = []
    error = None
    process = None
    log = (root/'process.log').open('x')
    def create(client, title):
        body = (codec.pack_fields([len(title)], [2])+_encode_bytes(title)+codec.pack_fields([0], [2])
                +codec.pack_fields([0, 0, 5, 8191, 0, 0, 0, 0xffffffff, 0, 0, 0, 0], [4]*12)
                +codec.pack_fields([0]*4, [1]*4))
        client.send(81, body)
        _, response = client.receive(86)
        return codec.read_fields(response[1:3], [2])[0]
    def listing(client, expected, label):
        _, body = client.receive(91)
        got = population(body)
        assert {key: value[:2] for key, value in got.items()} == expected, (label, got, expected)
        observations.append(dict(label=label, body_hex=body.hex(), rows=[dict(tuple=k, population=v[:2], title_hex=v[2]) for k, v in got.items()]))
        cases.append(label)
    try:
        process = subprocess.Popen([str(args.binary.resolve()), '--config', str(config), '--out', str(root/'service')],
                                   stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
        for _ in range(500):
            p = root/'service/service-state.json'
            if p.exists() and json.loads(p.read_text()).get('ready'):
                break
            assert process.poll() is None, 'startup failed'
            time.sleep(.01)
        else:
            raise AssertionError('startup timeout')
        for index in range(3):
            clients.append(account.connect(index, login, channel, header, hello, Client))
        a,b,c=clients
        room=create(a,b'Players')
        b.receive(91);c.receive(91)
        def join(client):
            client.send(82,codec.pack_fields([room,room,0,1,5],[2,4,2,1,2]));client.receive(87)
        join(b);c.receive(91);join(c)
        for client in clients:client.send(585,codec.pack_fields([0],[4]))
        for client in clients:
            ids=[]
            while True:
                op,body=client.receive()
                if op==106:ids.append(codec.read_fields(body[1:5],[4])[0])
                if op==117:break
            assert sorted(ids)==[1,2,3]
        cases.append('three_current_members_in_capacity4_not_identity_aliases')
        def choose(client,op,value,current):
            body=codec.pack_fields([0,value],[1,2]) if op==92 else codec.pack_fields([value],[1])
            client.send(op,body)
            for recipient in current:
                for _ in current:recipient.receive(110)
        choose(a,92,1,clients);choose(c,92,1,clients);choose(c,97,3,clients)
        cases.append('actual92_97_shapes_update_current_choices_and_team3_without_fixed_slot_stats')
        def ready_start(current,guests,expected):
            for guest in guests:
                guest.send(95,codec.pack_fields([6],[4])+bytes(6)+codec.pack_fields([1,0],[1,4]))
                for client in current:client.receive(110)
            a.send(104,bytes(20))
            for client in current:
                client.receive(582);_,body=client.receive(121)
                size=codec.read_fields(body[1:5],[4])[0];stream=_decode_bytes(body[5:5+size])
                assert stream[49]==len(current) and size==54+26*len(current)
                rows=[stream[54+i*26:80+i*26]for i in range(len(current))]
                got=[(x[0],x[2],int.from_bytes(x[3:5],'little'),x[5])for x in rows]
                assert got==expected,(got,expected)
            for row,client in zip(expected,current):
                client.send(79,bytes.fromhex('5b5b' if row[2]==1 else '5b5a'));client.quiet()
            for slot,client in zip([x[0]for x in expected],current):
                client.send(99,codec.pack_fields([0,0,32101+slot,0,0,0,0,0,0],[4,4,2,4,2,1,1,1,4]))
            keys=[]
            for slot,client in zip([x[0]for x in expected],current):
                client.receive(197);client.receive(196);_,body=client.receive(153)
                keys.append(codec.read_fields(body[15:19],[4])[0])
                # Independent slot-indexed peer decoder checks sparse inactive flags.
                at=35;active=[]
                for index in range(26):
                    flag=codec.read_fields(body[at:at+1],[1])[0];at+=1
                    if flag:
                        assert flag==1;values=codec.read_fields(body[at:at+18],[4,2,4,2,4,2]);at+=18
                        assert values==([0]*6 if index==slot else [int.from_bytes(socket.inet_aton('127.0.0.'+str(index+2)),'big'),32101+index,0,0,0,0]),(index,values)
                        active.append(index)
                assert at==len(body) and active==[x[0]for x in expected]
            assert len(set(keys))==1
            return keys[0]
        first=ready_start(clients,[b,c],[(0,0,1,0),(1,1,0,0),(2,2,1,3)])
        if args.active_departure:
            b.send(127,codec.pack_fields([39,1,120,399,1,0],[1,1,2,2,4,4]))
            b.quiet()
        cases.append('three_row_Start_uses_chosen_keys_1_0_1_and_team3_current_peer_barrier')
        def bomb(key,slot,serial,column=3):
            return codec.pack_fields([key,0,slot,serial,3+3*slot,column,1,0,0,0,1,1,0],[4,1,1,4,1,1,1,1,1,1,1,4,1])
        for client,slot,capacity in [(a,0,2),(b,1,1),(c,2,2)]:
            for serial in range(1,capacity+1):
                client.send(118,bomb(first,slot,serial,3+2*(serial-1)))
                for peer in clients:peer.receive(159)
            client.send(118,bomb(first,slot,capacity+1,7))
            for peer in clients:peer.quiet()
        if args.active_departure:
            b.close();clients.remove(b)
            for peer in clients:
                _,body=peer.receive(170)
                assert len(body)==21 and codec.read_fields(body[:6],[1,1,2,2])==[19,1,120,399]
                peer.quiet()
            cases.append('active_departure_removes_owner_but_retains_round_and_pending_bomb_recipients')
        for peer in clients:
            peer.sock.settimeout(10)
            for _ in range(5):peer.receive(158)
            peer.sock.settimeout(6)
        cases.append('character_capacity2_1_2_enforced_independently_of_fixture_slots_all_five_fuses_complete')
        def report(kind,tick,slot):
            body=codec.pack_fields([kind,slot,120,159,tick,0],[1,1,2,2,4,4])
            if kind in [17,19]:
                size=14 if kind==17 else 11;blob=bytes([24])+(size-5).to_bytes(4,'little')+bytes(1)+bytes(size-6)
                if kind==17:blob=blob[:10]+(7).to_bytes(4,'little')
                body+=codec.pack_fields([size],[4])+_encode_bytes(blob)
            return body
        for kind,tick in [(17,10),(19,11)]:
            c.send(127,report(kind,tick,2))
            a.receive(170)
            if not args.active_departure:b.receive(170)
        time.sleep(.1);a.send(127,report(21,12,0))
        for client in clients:
            client.sock.settimeout(15);_,body=client.receive(175);client.sock.settimeout(6)
            size=codec.read_fields(body[:4],[4])[0];stream=_decode_bytes(body[4:]);assert len(stream)==size
            at=22;ranks={}
            for slot in range(16):
                if stream[at]:assert stream[at]==1;ranks[slot]=stream[at+5];at+=131
                else:at+=1
            assert at==len(stream) and ranks=={0:1,1:1,2:2}
        for client in clients:
            client.receive(180)
            if args.active_departure:
                _,body=client.receive(107);assert codec.read_fields(body,[1,4,1])==[1,2,0]
            for _ in clients:client.receive(110)
        cases.append('three_member_team_result_and_reset_current_rank_rows_only')
        if not args.active_departure:
            b.send(86,b'');b.receive(241);b.receive(91)
            for peer in [a,c]:peer.receive(107);peer.receive(110)
            b.close();clients.remove(b)
        choose(c,92,0,clients)
        second=ready_start(clients,[c],[(0,0,1,0),(2,2,0,3)])
        assert first!=second
        a.send(118,bomb(first,0,1))
        for peer in clients:peer.quiet()
        a.send(118,bomb(second,0,1))
        for peer in clients:peer.receive(159)
        c.send(118,bomb(second,2,1))
        for peer in clients:peer.receive(159)
        c.send(118,bomb(second,2,2,5))
        for peer in clients:peer.quiet()
        cases.append('sparse_slots0_2_next_Start_peer_current_generation_and_updated_character_budget_no_old_key')
        for client in clients:client.close()
        clients.clear();os.killpg(process.pid,signal.SIGTERM);assert process.wait(timeout=6)==0;account.close()

    except Exception as exc:
        error = repr(exc)
        raise
    finally:
        for client in clients:
            client.close()
        if process is not None and process.poll() is None:
            os.killpg(process.pid, signal.SIGTERM)
            try:
                process.wait(timeout=6)
            except subprocess.TimeoutExpired:
                os.killpg(process.pid, signal.SIGKILL)
                process.wait(timeout=3)
        log.close()
        (root/'tcp-trace.json').write_text(json.dumps(trace,indent=2)+'\n')
        (root/'verification.json').write_text(json.dumps(dict(result='pass' if error is None else 'fail',
            cases=cases, observations=observations, error=error, returncode=None if process is None else process.returncode,
            limits='Synthetic TCP with real SQLite/private launcher and decoded Start/peer/rank invariants. No native client/runtime or UDP compatibility proof.'), indent=2)+'\n')
        assert all(Path(p['path']).stat().st_size == p['bytes'] and hashlib.sha256(Path(p['path']).read_bytes()).hexdigest() == p['sha256'] for p in pins)


if __name__ == '__main__':
    main()
