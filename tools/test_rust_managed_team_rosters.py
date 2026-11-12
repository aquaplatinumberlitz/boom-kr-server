"""M7b configured team rosters in managed rooms; synthetic TCP, not native proof."""
import argparse
import copy
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
    parser.add_argument('--roster-size', type=int, choices=[4,8], required=True)
    args = parser.parse_args()
    root = args.out.resolve()
    root.mkdir(parents=True, exist_ok=False, mode=0o700)
    cfg = json.loads(args.config.read_text())
    baseline = json.loads(args.baseline.read_text())['diagnostic_two_lobby']['fixed_room']
    two = cfg['diagnostic_two_lobby']
    assert len(two['actors']) == 8
    count=args.roster_size
    round_teams=[slot%2 for slot in range(count)]
    limits=[1,2,1,1]if count==4 else[1,2,1,1,2,1,1,1]
    ports = set()
    while len(ports) < 3:
        ports.add(free_port())
    login, wcp, channel = sorted(ports)
    cfg.update(bind='127.0.0.1', allowed_peers=[f'127.0.0.{n+2}' for n in range(8)], ports=sorted(ports))
    two.update(login_port=login, login_candidate_ports=[login, wcp], wcp_ports=sorted(ports),
               channel_port=channel, managed_rooms=dict(max_rooms=2, round_profile=dict(mode='teams', teams=round_teams, initial_bomb_capacity=limits)))
    for actor, peer in zip(two['actors'], cfg['allowed_peers']):
        actor['peer'] = peer
    account = AccountFixture(args.admin, root, cfg)
    two['account_admission']['worker_queue_capacity']=64
    config = root/'config.json'
    config.write_text(json.dumps(cfg, indent=2)+'\n')
    # Concrete startup negatives preserve actual account/fixture inputs and must
    # fail before creating the serving output directory or binding game ports.
    rejections=[]
    for label,value in [('queue_zero',0),('queue_above_bound',65)]:
        bad=copy.deepcopy(cfg);bad['diagnostic_two_lobby']['account_admission']['worker_queue_capacity']=value
        rejections.append((label,bad))
    for label,change in [('unknown_count',dict(teams=[0,1,0],initial_bomb_capacity=[1,2,1])),
                         ('wrong_teams',dict(teams=[0]*count)),
                         ('wrong_limits',dict(initial_bomb_capacity=[2]*count)),
                         ('duel_mode_with_team_roster',dict(mode='duel'))]:
        bad=copy.deepcopy(cfg);bad['diagnostic_two_lobby']['managed_rooms']['round_profile'].update(change)
        rejections.append((label,bad))
    rejection_files=[]
    for label,bad in rejections:
        file=root/(label+'.json');file.write_text(json.dumps(bad,indent=2)+'\n');rejection_files.append(file)
        dest=root/(label+'-must-not-serve')
        result=subprocess.run([str(args.binary.resolve()),'--config',str(file),'--out',str(dest)],capture_output=True,text=True,timeout=10)
        assert result.returncode!=0 and not dest.exists(),(label,result.returncode)
        (root/(label+'-result.json')).write_text(json.dumps(dict(returncode=result.returncode,stdout=result.stdout,stderr=result.stderr,output_created=dest.exists()),indent=2)+'\n')
    files = [*rejection_files, args.binary, args.admin, config, args.config, args.baseline, Path(__file__), Path('tools/rust_account_fixture.py'),
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
    cases = ['six_invalid_configs_rejected_before_serving']
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
        for index in range(8):
            clients.append(account.connect(index, login, channel, header, hello, Client))
            clients[-1].sock.settimeout(15)
        groups=[clients[:count]] if count==8 else[clients[:4],clients[4:]]
        def catalog(client):
            while True:
                op,body=client.receive()
                assert op in[90,91]
                if op==91:return population(body)
        def drain_catalogs(available):
            for client in available:catalog(client)
        remaining=list(clients)
        tuples=[]
        for n,group in enumerate(groups):
            key=create(group[0],b'Alpha'if n==0 else b'Beta');tuples.append(key);remaining.remove(group[0]);drain_catalogs(remaining)
            for client in group[1:]:
                client.send(82,codec.pack_fields([key,key,0,1,5],[2,4,2,1,2]));_,body=client.receive(87)
                assert codec.read_fields(body[1:2],[1])==[group.index(client)]
                remaining.remove(client);drain_catalogs(remaining)
        # All roster arrivals happen before setup to isolate membership projection.
        for client in clients:client.send(585,codec.pack_fields([0],[4]))
        for i,client in enumerate(clients):
            ids=[]
            while True:
                op,body=client.receive()
                if op==106:ids.append(codec.read_fields(body[1:5],[4])[0])
                if op==117:break
            offset=(i//count)*count
            assert sorted(ids)==list(range(offset+1,offset+count+1)),ids
        cases.append('managed_rosters_identity_slot_independent_and_current_population')
        def ready(group):
            for client in group[1:]:
                client.send(95,codec.pack_fields([6],[4])+bytes(6)+codec.pack_fields([1,0],[1,4]))
                for peer in group:peer.receive(110)
        def start(group):
            ready(group);group[0].send(104,bytes(20))
            for client in group:
                client.receive(582);_,body=client.receive(121)
                pin=baseline['packets']['start'];expected=codec.frame_body(Path(pin['fixture']['path']).read_bytes(),pin['sequence'])
                assert body==expected
        def load(client,port):client.send(99,codec.pack_fields([0,0,port,0,0,0,0,0,0],[4,4,2,4,2,1,1,1,4]))
        def go(group):
            keys=[]
            for client in group:
                client.receive(197);client.receive(196);_,body=client.receive(153)
                assert len(body)==61+18*count;keys.append(codec.read_fields(body[15:19],[4])[0])
            assert len(set(keys))==1
            return keys[0]
        for group in groups:start(group)
        for client in groups[0][:-1]:load(client,32101+clients.index(client))
        for client in groups[0]:client.quiet()
        keys=[]
        if len(groups)>1:
            for client in groups[1]:load(client,32101+clients.index(client))
            other=go(groups[1])
            for client in groups[0]:client.quiet()
        load(groups[0][-1],32101+clients.index(groups[0][-1]));keys.append(go(groups[0]))
        if len(groups)>1:keys.append(other)
        assert len(set(keys))==len(keys)
        cases.append('retained_native_Start_exact_and_final_current_roster_ACK_only_opens_own_barrier')
        def report(kind,tick,slot):
            body=codec.pack_fields([kind,slot,120,159,tick,0],[1,1,2,2,4,4])
            if kind in[17,19]:
                size=14 if kind==17 else 11
                blob=bytes([24])+(size-5).to_bytes(4,'little')+bytes(1)+bytes(size-6)
                if kind==17:blob=blob[:10]+(7).to_bytes(4,'little')
                body+=codec.pack_fields([size],[4])+_encode_bytes(blob)
            return body
        def kill(group,slot,tick):
            for kind in[17,19]:
                payload=report(kind,tick,slot);group[slot].send(127,payload);tick+=1
                expected=payload[:6]+(codec.pack_fields([7],[4])if kind==17 else payload[14:])
                for peer in group:
                    if peer!=group[slot]:assert peer.receive(170)[1]==expected
        def finish(group,winner,tick):
            for slot in range(count):
                if slot%2!=winner:kill(group,slot,tick)
            for slot in range(count):
                if slot%2==winner:group[slot].send(127,report(21,tick+2,slot))
        winners=[1 if n==0 else 0 for n in range(len(groups))]
        for group,winner in zip(groups,winners):finish(group,winner,10)
        def outcome(group,winner):
            for client in group:
                client.sock.settimeout(15);_,body=client.receive(175);client.sock.settimeout(6)
                # KR-native accepted layout check: rank16slots/each measured row131,
                # not the Rust encoder or server event as the oracle.
                blob=_decode_bytes(body[4:]);inner=blob[6:];assert len(inner)==32+130*count and len(body)==42+130*count
                assert [inner[16+131*slot+5]for slot in range(count)]==[1 if slot%2==winner else 2 for slot in range(count)]
            for client in group:
                assert codec.read_fields(client.receive(180)[1],[1,1,1])==[0,0,0]
                slots=[codec.read_fields(client.receive(110)[1][:1],[1])[0]for _ in range(count)]
                assert sorted(slots)==list(range(count))
        for group,winner in zip(groups,winners):outcome(group,winner)
        cases.append('opposite_independent_team_ranks_and_current_roster_return_reset')
        # New rounds keep IDs/slots but every room obtains fresh service epoch.
        for group in groups:start(group)
        for client in clients:load(client,32101+clients.index(client))
        fresh=[go(group)for group in groups];assert len(set(keys+fresh))==len(keys+fresh)
        cases.append('next_matches_receive_fresh_keys_and_current_member_peer_tables')
        def bomb(slot,key):return codec.pack_fields([key,0,slot,1,3,3,1,0,0,0,1,1,0],[4,1,1,4,1,1,1,1,1,1,1,4,1])
        if len(groups)>1:
            groups[0][0].send(118,bomb(0,fresh[1]));groups[1][0].send(118,bomb(0,keys[1]))
            for client in clients:client.quiet()
            cases.append('wrong_room_and_old_round_bomb_keys_have_no_effect')
        lost=groups[0][0];lost.close();clients.remove(lost)
        for peer in groups[0][1:]:peer.receive(107);peer.receive(110)
        # An unsupported survivor roster never receives a new round/result.
        for peer in groups[0][1:]:peer.quiet()
        if len(groups)>1:
            groups[1][0].send(118,bomb(0,fresh[1]))
            for peer in groups[1]:peer.receive(159)
            for peer in groups[1]:peer.sock.settimeout(10);peer.receive(158);peer.sock.settimeout(6)
            for peer in groups[0][1:]:peer.quiet()
        cases.append('current_team_member_EOF_fences_only_affected_room_other_room_bomb_still_runs')
        for client in clients:
            client.close()
        clients.clear()
        os.killpg(process.pid, signal.SIGTERM)
        assert process.wait(timeout=6) == 0
        account.close()
    except Exception as exc:
        error = repr(exc)
        if process is not None:
            samples=[]
            for task in (Path('/proc')/str(process.pid)/'task').glob('*'):
                try:samples.append(dict(tid=task.name,status=(task/'status').read_text(),wchan=(task/'wchan').read_text()))
                except OSError:pass
            (root/'failure-threads.json').write_text(json.dumps(samples,indent=2)+'\n')
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
            limits='Synthetic TCP, actual SQLite/private launcher, measured Start/rank layout vectors; no native client/UDP proof.'), indent=2)+'\n')
        assert all(Path(p['path']).stat().st_size == p['bytes'] and hashlib.sha256(Path(p['path']).read_bytes()).hexdigest() == p['sha256'] for p in pins)


if __name__ == '__main__':
    main()
