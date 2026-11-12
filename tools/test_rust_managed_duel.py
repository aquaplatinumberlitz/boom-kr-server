"""M7b two managed duel lifecycles, real backend, synthetic TCP only."""
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
from tools.test_rust_room import Client


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
               channel_port=channel, managed_rooms=dict(max_rooms=2, round_profile=dict(battle=baseline['battle'], bomb_capacity={str(i): n for i,n in enumerate([1,2,1,1],1)})))
    for actor, peer in zip(two['actors'], cfg['allowed_peers']):
        actor['peer'] = peer
    account = AccountFixture(args.admin, root, cfg)
    config = root/'config.json'
    config.write_text(json.dumps(cfg, indent=2)+'\n')
    files = [args.binary, args.admin, config, args.config, args.baseline, Path(__file__), Path('tools/rust_account_fixture.py'),
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
        for index in range(4):
            clients.append(account.connect(index, login, channel, header, hello, Client))
        a, b, c, d = clients
        first = create(a, b'Alpha')
        for client in [b,c,d]: client.receive(91)
        second = create(c, b'Beta')
        for client in [b,d]: client.receive(91)
        d.send(82, codec.pack_fields([second,second,0,1,5],[2,4,2,1,2])); d.receive(87); b.receive(91)
        b.send(82, codec.pack_fields([first,first,0,1,5],[2,4,2,1,2])); b.receive(87)
        for client in clients: client.send(585, codec.pack_fields([0],[4]))
        for index,client in enumerate(clients):
            ids=[]
            while True:
                op,body=client.receive()
                if op==106: ids.append(codec.read_fields(body[1:5],[4])[0])
                if op==117: break
            assert sorted(ids)==([1,2] if index<2 else [3,4])
        cases.append('two_rooms_two_current_members_each_in_capacity4_and_nonidentity_slots')
        def ready(pair):
            pair[1].send(95,codec.pack_fields([6],[4])+bytes(6)+codec.pack_fields([1,0],[1,4]))
            for client in pair: client.receive(110)
        pin=baseline['packets']['start']
        retained_start=codec.frame_body(Path(pin['fixture']['path']).read_bytes(),pin['sequence'])
        def start(pair):
            ready(pair); pair[0].send(104,bytes(20))
            for client in pair:
                client.receive(582); _,body=client.receive(121); assert body==retained_start
        def load(client,port): client.send(99,codec.pack_fields([0,0,port,0,0,0,0,0,0],[4,4,2,4,2,1,1,1,4]))
        def go(pair):
            keys=[]
            for client in pair:
                client.receive(197); client.receive(196); _,body=client.receive(153)
                assert len(body)==97
                keys.append(codec.read_fields(body[15:19],[4])[0])
            assert keys[0]==keys[1]
            return keys[0]
        def bomb(slot,key,serial=1): return codec.pack_fields([key,0,slot,serial,3,3,1,0,0,0,1,1,0],[4,1,1,4,1,1,1,1,1,1,1,4,1])
        def report(kind,tick,slot):
            body=codec.pack_fields([kind,slot,120,159,tick,0],[1,1,2,2,4,4])
            if kind in [17,19]:
                size=14 if kind==17 else 11
                blob=bytes([24])+(size-5).to_bytes(4,'little')+bytes(1)+bytes(size-6)
                if kind==17: blob=blob[:10]+(7).to_bytes(4,'little')
                body+=codec.pack_fields([size],[4])+_encode_bytes(blob)
            return body
        def kill(pair,slot,tick):
            for kind in [17,19]:
                pair[slot].send(127,report(kind,tick,slot)); tick+=1
                pair[1-slot].receive(170)
            time.sleep(.01)
            pair[1-slot].send(127,report(21,tick,1-slot))
        start([a,b]); start([c,d]); load(a,32101); a.quiet(); b.quiet()
        load(c,32103); load(d,32104); beta=go([c,d]); a.quiet(); b.quiet()
        cases.append('Start121_exact_independent_two_row_fixture_and_room_specific_loading_barriers')
        load(b,32102); alpha=go([a,b]); assert alpha!=beta and alpha%256!=beta%256
        cases.append('room_scoped_peer153_keys_endpoints_with_independent_round_epochs')
        a.send(118,bomb(0,beta)); c.send(118,bomb(0,alpha))
        for client in clients: client.quiet()
        c.send(118,bomb(0,beta))
        for client in [c,d]: client.receive(159)
        a.quiet(); b.quiet()
        c.send(118,bomb(0,beta,2)); c.quiet(); d.quiet()
        cases.append('wrong_room_key_has_no_effect_and_C_slot0_uses_own_measured_capacity1')
        for client in [c,d]: client.sock.settimeout(10); client.receive(158); client.sock.settimeout(6)
        kill([a,b],0,10); kill([c,d],1,10)
        expected={0:'rank',1:'rank',2:'rank_Awin',3:'rank_Awin'}
        for index,client in enumerate(clients):
            pin=baseline['battle']['packets'][expected[index]]
            want=codec.frame_body(Path(pin['fixture']['path']).read_bytes(),pin['sequence'])
            client.sock.settimeout(15); _,got=client.receive(175); assert got==want; client.sock.settimeout(6)
        for client in clients: client.receive(180); client.receive(110); client.receive(110)
        cases.append('opposite_room_results_exact_retained175_and_return180_reset_only_current_roster')
        start([a,b]); start([c,d])
        load(c,32103); load(d,32104); beta2=go([c,d]); assert beta2 not in [alpha,beta]
        c.send(118,bomb(0,beta)); c.quiet(); d.quiet()
        a.quiet(); b.quiet()
        cases.append('next_match_fresh_key_old_key_rejected_and_other_room_still_loading')
        load(a,32101); load(b,32102); alpha2=go([a,b]); assert len({alpha,beta,alpha2,beta2})==4
        a.send(118,bomb(0,alpha2))
        for client in [a,b]: client.receive(159)
        a.close(); clients.remove(a)
        c.send(118,bomb(0,beta2))
        for client in [c,d]: client.receive(159)
        for client in [c,d]: client.sock.settimeout(10); client.receive(158); client.sock.settimeout(6)
        b.sock.settimeout(15); b.receive(175); b.sock.settimeout(6); b.receive(180); b.receive(107); b.receive(110)
        cases.append('EOF_fences_pending_blast_in_Alpha_while_Beta_completes_its_blast_and_forfeit_is_room_local')
        for client in clients:
            client.close()
        clients.clear()
        os.killpg(process.pid, signal.SIGTERM)
        assert process.wait(timeout=6) == 0
        account.close()
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
        (root/'verification.json').write_text(json.dumps(dict(result='pass' if error is None else 'fail',
            cases=cases, observations=observations, error=error, returncode=None if process is None else process.returncode,
            limits='Synthetic TCP with real SQLite/private launcher; exact retained native fixture comparisons. No native client/runtime or UDP compatibility proof.'), indent=2)+'\n')
        assert all(Path(p['path']).stat().st_size == p['bytes'] and hashlib.sha256(Path(p['path']).read_bytes()).hexdigest() == p['sha256'] for p in pins)


if __name__ == '__main__':
    main()
