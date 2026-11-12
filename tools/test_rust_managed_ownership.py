"""M7d real SQLite and synthetic TCP ownership/failure checks; not native proof."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import signal
import socket
import sqlite3
import subprocess
import sys
import time

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from server import probe_codec as codec
from server.room_title import _encode_bytes
from tools.rust_account_fixture import AccountFixture
from tools.test_rust_foundation import exact, free_port
from tools.test_rust_managed_duel import population
from tools.test_rust_room import Client


class TraceClient(Client):
    trace = None

    def send(self, op, body):
        self.trace.append(dict(direction='send', ns=time.monotonic_ns(), peer=self.sock.getsockname(),
                               opcode=op, sequence=self.tx, body_hex=body.hex()))
        super().send(op, body)

    def receive(self, expected=None):
        header = exact(self.sock, 37)
        size = int.from_bytes(header[13:17], 'little')
        assert size <= 65536
        frame = header + exact(self.sock, size+1)
        body = codec.frame_body(frame, self.rx)
        op = int.from_bytes(header[5:9], 'little')
        self.trace.append(dict(direction='receive', ns=time.monotonic_ns(), peer=self.sock.getsockname(),
                               opcode=op, sequence=self.rx, frame_hex=frame.hex()))
        self.rx += 3
        if expected is not None:
            assert op == expected, (op, expected)
        return op, body


def run(args, mode):
    root = args.out.resolve()/mode
    root.mkdir(mode=0o700)
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
               channel_port=channel, managed_rooms=dict(max_rooms=2, round_profile=dict(
                   battle=baseline['battle'], initial_bomb_capacity=[1, 2])))
    for actor, peer in zip(two['actors'], cfg['allowed_peers']):
        actor['peer'] = peer
    account = AccountFixture(args.admin, root, cfg)
    expiry = None
    if mode == 'expiry':
        account.command(dict(op='revoke', account_id=account.players[0]['account_id']))
        grant = account.command(dict(op='login', login='alice', password='account-basic-test-password', ttl_seconds=30))
        account.sessions[0] = grant['token']
        expiry = grant['expires_at']
    config = root/'config.json'
    config.write_text(json.dumps(cfg, indent=2)+'\n')
    files = [args.binary, args.admin, args.config, args.baseline, config, Path(__file__),
             Path('tools/rust_account_fixture.py'), Path('tools/test_rust_room.py'),
             Path('tools/test_rust_foundation.py'), Path('tools/test_rust_managed_duel.py'),
             *Path('rust/crates').rglob('*.rs'), *Path('rust/crates').rglob('*.sql'), Path('rust/Cargo.lock')]

    def fixtures(value):
        if isinstance(value, dict):
            if {'path', 'bytes', 'sha256'} <= value.keys():
                p = Path(value['path'])
                assert p.stat().st_size == value['bytes']
                assert hashlib.sha256(p.read_bytes()).hexdigest() == value['sha256']
                files.append(p)
            for item in value.values():
                fixtures(item)
        elif isinstance(value, list):
            for item in value:
                fixtures(item)
    fixtures(cfg)
    frozen = root/'frozen'
    frozen.mkdir()
    pins = []
    for index, source in enumerate(dict.fromkeys(files)):
        dest = frozen/f'{index:03d}-{source.name}'
        shutil.copyfile(source, dest)
        pins.append(dict(path=str(source), retained_path=str(dest), bytes=source.stat().st_size,
                         sha256=hashlib.sha256(source.read_bytes()).hexdigest()))
    (root/'preflight.json').write_text(json.dumps(dict(files=pins,
        limits='Synthetic TCP/real SQLite. Admin grants/revoke and DB lock are declared runtime inputs. No native proof.'), indent=2)+'\n')
    hello = Path(cfg['bootstrap']['path']).read_bytes()
    header = Path(two['actors'][0]['profile']['own_header']['path']).read_bytes()[:37]
    clients, cases, trace = [], [], []
    TraceClient.trace = trace
    proc = None
    db_lock = None
    error = None
    output = (root/'process.log').open('x')

    def note(label):
        cases.append(label)

    def connect(index):
        client = account.connect(index, login, channel, header, hello, TraceClient)
        clients.append(client)
        return client

    def closed(client):
        # Consume exact codec frames until EOF, retaining any forbidden delivery.
        seen = []
        client.sock.settimeout(6)
        while True:
            try:
                op, _ = client.receive()
                seen.append(op)
            except (ConnectionResetError, AssertionError) as exc:
                if isinstance(exc, AssertionError):
                    assert str(exc) == 'unexpected EOF', repr(exc)
                break
        return seen

    def create(client, title):
        body = (codec.pack_fields([len(title)], [2])+_encode_bytes(title)+codec.pack_fields([0], [2])
                +codec.pack_fields([0, 0, 5, 8191, 0, 0, 0, 0xffffffff, 0, 0, 0, 0], [4]*12)
                +codec.pack_fields([0]*4, [1]*4))
        client.send(81, body)
        _, body = client.receive(86)
        return codec.read_fields(body[1:3], [2])[0]

    def join(client, key):
        client.send(82, codec.pack_fields([key, key, 0, 1, 5], [2, 4, 2, 1, 2]))
        client.receive(87)

    def setup(client, identities):
        client.send(585, codec.pack_fields([0], [4]))
        seen = []
        while True:
            op, body = client.receive()
            if op == 106:
                seen.append(codec.read_fields(body[1:5], [4])[0])
            if op == 117:
                break
        assert sorted(seen) == identities, (seen, identities)

    def start(pair):
        pair[1].send(95, codec.pack_fields([6], [4])+bytes(6)+codec.pack_fields([1, 0], [1, 4]))
        for client in pair:
            client.receive(110)
        pair[0].send(104, bytes(20))
        for client in pair:
            client.receive(582)
            client.receive(121)
        for i, client in enumerate(pair):
            client.send(99, codec.pack_fields([0, 0, 32101+i, 0, 0, 0, 0, 0, 0], [4, 4, 2, 4, 2, 1, 1, 1, 4]))
        keys = []
        for client in pair:
            client.receive(197)
            client.receive(196)
            _, body = client.receive(153)
            keys.append(codec.read_fields(body[15:19], [4])[0])
        assert keys[0] == keys[1]
        return keys[0]

    def bomb(key, slot=0):
        return codec.pack_fields([key, 0, slot, 1, 3, 3, 1, 0, 0, 0, 1, 1, 0],
                                 [4, 1, 1, 4, 1, 1, 1, 1, 1, 1, 1, 4, 1])

    try:
        proc = subprocess.Popen([str(args.binary.resolve()), '--config', str(config), '--out', str(root/'service')],
                                stdout=output, stderr=subprocess.STDOUT, start_new_session=True)
        for _ in range(500):
            state = root/'service/service-state.json'
            if state.exists() and json.loads(state.read_text()).get('ready'):
                break
            assert proc.poll() is None, 'startup failed'
            time.sleep(.01)
        else:
            raise AssertionError('startup timeout')
        a, b, c, d = [connect(i) for i in range(4)]
        alpha = create(a, b'Alpha')
        for client in [b, c, d]:
            client.receive(91)
        beta = create(c, b'Beta')
        for client in [b, d]:
            client.receive(91)
        join(d, beta)
        b.receive(91)
        join(b, alpha)
        # Send both setup requests before reading their full snapshots.
        for client in clients:
            client.send(585, codec.pack_fields([0], [4]))
        for i, client in enumerate(clients):
            ids = []
            while True:
                op, body = client.receive()
                if op == 106:
                    ids.append(codec.read_fields(body[1:5], [4])[0])
                if op == 117:
                    break
            assert sorted(ids) == ([1, 2] if i < 2 else [3, 4])
        note('four_owned_accounts_two_distinct_rooms_correct_slots')
        if mode == 'ownership':
            rogue = TraceClient(0, channel, header, hello)
            clients.append(rogue)
            rogue.send(6, account.channel_requests[0])
            assert not closed(rogue)
            rogue.close()
            with socket.socket(socket.AF_UNIX) as control:
                control.settimeout(4)
                control.connect(str(account.socket))
                control.sendall(json.dumps(dict(session=account.sessions[0], player_id=account.players[0]['player_id'],
                                               peer=account.peers[0])).encode()+b'\n')
                assert not json.loads(control.recv(4096))['ok']
            for client in [a, b, c, d]:
                client.quiet()
            note('duplicate_channel_and_pending_launcher_denied_without_second_slot_or_owner_loss')
            b.send(104, bytes(20))  # Guest cannot act as host.
            assert not closed(b)
            a.receive(107)
            a.receive(110)
            c.quiet()
            d.quiet()
            note('guest_Start_denied_without_cross_room_effect_and_membership_removed')
            b.close()
            fresh = connect(1)
            fresh.send(104, bytes(20))  # Lobby outsider cannot start a room.
            assert not closed(fresh)
            a.quiet()
            c.quiet()
            d.quiet()
            note('lobby_outsider_Start_denied_without_ghost_membership')
            fresh.close()
            c.send(82, codec.pack_fields([alpha, beta, 0, 1, 5], [2, 4, 2, 1, 2]))
            assert not closed(c)
            d.receive(107)
            d.receive(110)
            a.quiet()
            note('member_wrong_room_tuple_rejected_without_moving_to_target_room')
        else:
            alpha_key = start([a, b])
            beta_key = start([c, d])
            assert alpha_key != beta_key
            if expiry is not None:
                delay = expiry-time.time()-2
                assert delay > 0, 'not enough time before actual expiry; do not backdate DB'
                time.sleep(delay)
            a.send(118, bomb(alpha_key))
            for client in [a, b]:
                client.receive(159)
            pending_ns = time.monotonic_ns()
            note('current_Alpha_bomb_confirmed_before_failure_with_pending_fuse')
            if mode == 'revoke':
                account.command(dict(op='revoke', account_id=account.players[0]['account_id']))
            elif mode == 'db-lock':
                db_lock = sqlite3.connect(account.database)
                db_lock.execute('BEGIN EXCLUSIVE')
            else:
                assert mode == 'expiry'
            assert 158 not in closed(a)
            a.close()
            if mode == 'db-lock':
                # The backend is shared: loss of authorization is service-wide.
                # Healthy-room continuation is not promised through a global DB outage.
                for client in [b, c, d]:
                    assert 158 not in closed(client)
                    client.close()
                db_lock.rollback()
                db_lock.close()
                db_lock = None
                a = connect(0)
                initial = next(x for x in reversed(trace) if x['direction']=='receive' and x['opcode']==91)
                raw = bytes.fromhex(initial['frame_hex'])
                assert population(codec.frame_body(raw, initial['sequence'])) == {}
                key = create(a, b'Recovered')
                assert key not in [alpha, beta]
                setup(a, [1])
                note('SQLite_busy_fails_closed_all_current_channels_then_no_ghost_rooms_after_recovery')
            else:
                b.sock.settimeout(15)
                # Forfeit is delayed by the accepted round lifecycle policy.
                b.receive(175)
                b.receive(180)
                b.receive(107)
                b.receive(110)
                b.sock.settimeout(6)
                time.sleep(max(0, (pending_ns+7_000_000_000-time.monotonic_ns())/1e9))
                b.quiet()
                c.send(118, bomb(beta_key))
                for client in [c, d]:
                    client.receive(159)
                for client in [c, d]:
                    client.sock.settimeout(10)
                    client.receive(158)
                    client.sock.settimeout(6)
                note(mode+'_fences_Alpha_pending_blast_and_forfeits_only_Alpha_Beta_current_blast_completes')
                grant = account.command(dict(op='login', login='alice', password='account-basic-test-password', ttl_seconds=60))
                account.sessions[0] = grant['token']
                a = connect(0)
                join(a, alpha)
                setup(a, [1, 2])
                b.receive(106)
                # Old connection6/echo must not invalidate the fresh account generation.
                rogue = TraceClient(0, channel, header, hello)
                clients.append(rogue)
                old_request = next(x for x in trace if x['direction']=='send' and x['opcode']==6)['body_hex']
                rogue.send(6, bytes.fromhex(old_request))
                assert not closed(rogue)
                rogue.close()
                new_key = start([b, a])
                assert new_key not in [alpha_key, beta_key]
                a.send(118, bomb(alpha_key))
                a.quiet()
                b.quiet()
                a.send(118, bomb(new_key))
                for client in [a, b]:
                    client.receive(159)
                note('fresh_account_channel_rejoins_vacant_slot_old_echo_and_round_key_denied_current_positive')
        for client in clients:
            client.close()
        clients.clear()
        os.killpg(proc.pid, signal.SIGTERM)
        assert proc.wait(timeout=6) == 0
        assert not account.socket.exists()
        account.close()
    except Exception as exc:
        error = repr(exc)
        raise
    finally:
        if db_lock is not None:
            db_lock.rollback()
            db_lock.close()
        for client in clients:
            client.close()
        if proc is not None and proc.poll() is None:
            os.killpg(proc.pid, signal.SIGTERM)
            try:
                proc.wait(timeout=6)
            except subprocess.TimeoutExpired:
                os.killpg(proc.pid, signal.SIGKILL)
                proc.wait(timeout=3)
        output.close()
        (root/'tcp-trace.json').write_text(json.dumps(trace, indent=2)+'\n')
        (root/'verification.json').write_text(json.dumps(dict(result='pass' if error is None else 'fail', mode=mode,
            cases=cases, error=error, returncode=None if proc is None else proc.returncode,
            cleanup=dict(pid=None if proc is None else proc.pid, pid_gone=proc is None or not Path(f'/proc/{proc.pid}').exists()),
            limits='Real SQLite/private launcher and synthetic TCP. No native client, UDP, DB-queue native fault or arbitrary profile acceptance.'), indent=2)+'\n')
        assert all(Path(row['path']).stat().st_size == row['bytes'] and hashlib.sha256(Path(row['path']).read_bytes()).hexdigest() == row['sha256'] for row in pins)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['binary', 'admin', 'config', 'baseline', 'out']:
        parser.add_argument('--'+name, type=Path, required=True)
    parser.add_argument('--case', choices=['ownership', 'revoke', 'expiry', 'db-lock'], action='append', dest='cases')
    args = parser.parse_args()
    selected = args.cases or ['ownership', 'revoke', 'expiry', 'db-lock']
    args.out.mkdir(parents=True, exist_ok=False, mode=0o700)
    for mode in selected:
        run(args, mode)
    reports = [json.loads((args.out/mode/'verification.json').read_text()) for mode in selected]
    (args.out/'verification.json').write_text(json.dumps(dict(result='pass',
        cases=[f"{report['mode']}: {case}" for report in reports for case in report['cases']],
        limits='Synthetic TCP plus real SQLite, distinct from native KR acceptance.'), indent=2)+'\n')


if __name__ == '__main__':
    main()
