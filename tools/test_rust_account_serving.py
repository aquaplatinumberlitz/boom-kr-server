"""Synthetic TCP/private-launcher M5 checks; no native KR authentication proof."""
import argparse
import copy
import hashlib
import json
import os
from pathlib import Path
import shutil
import signal
import socket
import sqlite3
import subprocess
import tempfile
import time
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from server import probe_codec as codec
from server.room_title import _encode_bytes, _decode_bytes
from tools.test_rust_foundation import exact, free_port


def sha(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--binary', type=Path, required=True)
    p.add_argument('--admin', type=Path, required=True)
    p.add_argument('--config', type=Path, required=True)
    p.add_argument('--out', type=Path, required=True)
    a = p.parse_args()
    a.out.mkdir(parents=True, exist_ok=False, mode=0o700)
    os.chmod(a.out, 0o700)
    root = a.out.resolve()
    private = Path(tempfile.mkdtemp(prefix='kr-m5-'))
    os.chmod(private, 0o700)
    api = private / 'launch.sock'
    db = root / 'accounts.sqlite'
    cfg = json.loads(a.config.read_text())
    cfg['diagnostic_two_lobby'].pop('fixed_room', None)
    assert len(cfg['diagnostic_two_lobby']['actors']) == 2
    ports = set()
    while len(ports) < 3:
        ports.add(free_port())
    wcp, login_port, channel_port = sorted(ports)
    cfg.update(bind='127.0.0.1', allowed_peers=['127.0.0.2', '127.0.0.3'], ports=sorted(ports))
    two = cfg['diagnostic_two_lobby']
    two.update(wcp_ports=sorted(ports), login_port=login_port, login_candidate_ports=[], channel_port=channel_port,
               account_admission=dict(database=str(db), launch_socket=str(api), launch_ttl_seconds=60))
    for i, actor in enumerate(two['actors']):
        actor['peer'] = cfg['allowed_peers'][i]
    config = root / 'config.json'
    config.write_text(json.dumps(cfg, indent=2)+'\n')
    calls = []
    cases = []
    sockets = []
    proc = None
    log = None
    error = None

    def admin(command):
        request = dict(actor='M5-serving-test', reason='synthetic ownership/expiry acceptance', command=command)
        result = subprocess.run([str(a.admin.resolve()), str(db)], input=json.dumps(request),
                                capture_output=True, text=True, timeout=10)
        calls.append(dict(request=request, returncode=result.returncode, stdout=result.stdout, stderr=result.stderr))
        assert result.returncode == 0, 'admin failed (private raw retained)'
        return json.loads(result.stdout)

    admin(dict(op='init'))
    admin(dict(op='begin-instance'))
    players = [admin(dict(op='create', login=login, name=name, password='serving-test-password'))['player']
               for login, name in [('alice', 'KRLocal'), ('bob', 'KRGuest')]]
    grants = [admin(dict(op='login', login=login, password='serving-test-password', ttl_seconds=120))
              for login in ['alice', 'bob']]
    shutil.copyfile(db, root/'initial-db.sqlite')
    sources = [a.binary, a.admin, a.config, config, Path(__file__), Path('rust/Cargo.lock'),
               *sorted(Path('rust/crates').rglob('*.rs')), *sorted(Path('rust/crates').rglob('*.sql'))]
    for key in ['bootstrap', 'login_error', 'wcp_second']:
        sources.append(Path(cfg[key]['path']))
    for actor in two['actors']:
        sources += [Path(actor['profile']['own_header']['path'])]
        sources += [Path(v['fixture']['path']) for v in actor['profile']['packets'].values()]
    sources = list(dict.fromkeys(sources))
    frozen = root/'frozen'
    frozen.mkdir(mode=0o700)
    pins = []
    for i, path in enumerate(sources):
        target = frozen/f'{i:03d}-{path.name}'
        shutil.copyfile(path, target)
        pins.append(dict(path=str(path), retained_path=str(target), bytes=path.stat().st_size, sha256=sha(path)))
    (root/'preflight.json').write_text(json.dumps(dict(files=pins, dynamic_database=str(db),
        private_control=str(api), limits='Loopback synthetic TCP, no native client. DB/admin mutations are declared runtime actions.'), indent=2)+'\n')
    hello = Path(cfg['bootstrap']['path']).read_bytes()
    header = Path(two['actors'][0]['profile']['own_header']['path']).read_bytes()[:37]

    def note(case):
        cases.append(dict(case=case, result='pass'))

    def launch(session, player, peer, extra=None):
        request = dict(session=session, player_id=player, peer=peer)
        if extra:
            request.update(extra)
        with socket.socket(socket.AF_UNIX) as s:
            s.settimeout(4)
            s.connect(str(api))
            s.sendall(json.dumps(request).encode()+b'\n')
            data = b''
            while b'\n' not in data:
                chunk = s.recv(4096)
                assert chunk
                data += chunk
            return json.loads(data)

    def connect(peer, port):
        s = socket.socket()
        s.settimeout(3)
        s.bind((peer, 0))
        s.connect(('127.0.0.1', port))
        sockets.append(s)
        assert exact(s, 15) == hello
        return s

    def send(s, op, body, seq=40):
        s.sendall(codec.encode(header, op, body, seq))

    def reply(s, op, seq):
        prefix = exact(s, 37)
        size = int.from_bytes(prefix[13:17], 'little')
        assert size <= 65536 and int.from_bytes(prefix[5:9], 'little') == op
        return codec.frame_body(prefix+exact(s, size+1), seq)

    def login_body(front, peer):
        return (codec.pack_fields([4, 1, 32], [1, 1, 2])+codec.crypt(front.encode(), 0x11223344)
                +bytes(16)+codec.pack_fields([0, int.from_bytes(socket.inet_aton(peer), 'little'), 0], [4, 4, 1]))

    def login(front, peer, expected):
        s = connect(peer, login_port)
        send(s, 1, login_body(front, peer))
        body = reply(s, 4, 43)
        assert codec.read_fields(body[:1], [1]) == [expected]
        if expected:
            s.close()
            return None
        assert len(body) == 137
        identity = codec.read_fields(body[25:29], [4])[0]
        name = _decode_bytes(body[31:38])
        echo = codec.read_fields(body[93:97], [4])[0]
        send(s, 9, bytes(8), 43)
        reply(s, 15, 46)
        reply(s, 62, 49)
        send(s, 5, bytes(9), 46)
        reply(s, 7, 52)
        return dict(socket=s, identity=identity, name=name, echo=echo)

    def channel_body(player, echo):
        name = player['display_name'].encode()
        return (codec.pack_fields([1, 255, len(name)], [1, 1, 2])+_encode_bytes(name)
                +codec.pack_fields([player['wire_id'], echo], [4, 4])
                +codec.pack_fields([0]*8, [8, 1, 1, 1, 4, 4, 1, 4]))

    def closed(s):
        try:
            assert s.recv(1) == b''
        except ConnectionResetError:
            pass
        s.close()

    started = time.monotonic()
    try:
        log = (root/'service.log').open('x')
        proc = subprocess.Popen([str(a.binary.resolve()), '--config', str(config), '--out', str(root/'service')],
                                stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
        deadline = time.monotonic()+8
        while time.monotonic() < deadline:
            state = root/'service/service-state.json'
            if state.exists() and json.loads(state.read_text())['ready']:
                break
            assert proc.poll() is None, 'startup failed'
            time.sleep(.01)
        else:
            raise AssertionError('readiness deadline')
        assert (api.stat().st_mode & 0o777) == 0o600
        note('private control mode600 and ready')
        assert not launch('0'*64, players[0]['player_id'], '127.0.0.2')['ok']
        assert not launch(grants[0]['token'], players[1]['player_id'], '127.0.0.2')['ok']
        assert not launch(grants[0]['token'], players[0]['player_id'], '127.0.0.9')['ok']
        assert not launch(grants[0]['token'], players[0]['player_id'], '127.0.0.2', {'unknown': True})['ok']
        note('unknown session wrong owner peer and unknown request field denied')
        test_peers = ['127.0.0.2', '127.0.0.2']
        for peer in test_peers:
            s = connect(peer, wcp)
            send(s, 0, bytes(28))
            assert exact(s, len(Path(cfg['wcp_second']['path']).read_bytes())) == Path(cfg['wcp_second']['path']).read_bytes()
            s.close()
        note('same source WCP transports classified by request not account identity')
        front = [launch(g['token'], pl['player_id'], peer)['frontend']
                 for g, pl, peer in zip(grants, players, test_peers)]
        assert all(len(x) == 32 for x in front) and front[0] != front[1]
        assert not launch(grants[0]['token'], players[0]['player_id'], '127.0.0.2')['ok']
        note('distinct one-owner frontend32 grants; duplicate pending denied')
        s = connect(test_peers[0], login_port)
        malformed = bytearray(codec.encode(header, 1, login_body(front[0], test_peers[0]), 40))
        malformed[-1] ^= 1
        s.sendall(malformed)
        closed(s)
        note('bad CRC closes before consuming frontend; later valid login succeeds')
        auth = [login(f, peer, 0) for f, peer in zip(front, test_peers)]
        assert [(x['identity'], x['name']) for x in auth] == [(1, b'KRLocal'), (2, b'KRGuest')]
        assert auth[0]['echo'] != auth[1]['echo']
        note('backend owned names IDs and distinct field36 echoes at login')
        login(front[0], '127.0.0.2', 1)
        login('x'*32, '127.0.0.2', 1)
        note('used and unknown frontend token receive result1')
        for peer, player, echo in [('127.0.0.2', players[1], auth[0]['echo']),
                                   ('127.0.0.3', players[0], auth[0]['echo'])]:
            s = connect(peer, channel_port)
            send(s, 6, channel_body(player, echo))
            closed(s)
        note('swapped name ID and peer cannot consume current owner redirect')
        channels = []
        for peer, player, authn in zip(test_peers, players, auth):
            s = connect(peer, channel_port)
            send(s, 6, channel_body(player, authn['echo']))
            reply(s, 17, 43)
            send(s, 67, bytes(2), 43)
            for op, seq in [(241, 46), (91, 49), (14, 52), (291, 55)]:
                body = reply(s, op, seq)
                if op == 291:
                    size = codec.read_fields(body[1:5], [4])[0]
                    blob = _decode_bytes(body[5:5+size])
                    assert blob[10:17] == player['display_name'].encode()
            channels.append(s)
        note('two owned channels reach ready own291 zero profiles')
        s = connect('127.0.0.2', channel_port)
        send(s, 6, channel_body(players[0], auth[0]['echo']))
        closed(s)
        note('channel replay cannot allocate second owner')
        send(auth[0]['socket'], 87, codec.pack_fields([0], [2]), 49)
        closed(auth[0]['socket'])
        note('opaque87 not allowed on completed login transport; current channel remains owned')
        admin(dict(op='revoke', account_id=players[0]['account_id']))
        closed(channels[0])
        channels[1].settimeout(.2)
        try:
            data = channels[1].recv(1)
            raise AssertionError(f'healthy B unexpectedly changed transport ({len(data)} bytes)')
        except socket.timeout:
            pass
        note('revoke A closes its channel; B remains current')
        expired = admin(dict(op='login', login='alice', password='serving-test-password', ttl_seconds=2))
        pending = launch(expired['token'], players[0]['player_id'], '127.0.0.2')
        assert pending['ok']
        time.sleep(2.1)
        login(pending['frontend'], '127.0.0.2', 1)
        note('backend session expired after frontend delivery denied at login')
        admin(dict(op='rename', account_id=players[0]['account_id'], name='Renamed', expected_version=1))
        expired = admin(dict(op='revoke', account_id=players[0]['account_id']))
        renamed = admin(dict(op='login', login='alice', password='serving-test-password', ttl_seconds=60))
        assert not launch(renamed['token'], players[0]['player_id'], '127.0.0.2')['ok']
        note('unsupported changed profile has no fallback KR identity')
        for i in range(16):
            send(channels[1], 87, codec.pack_fields([0], [2]), 46+3*i)
        channels[1].settimeout(.2)
        try:
            channels[1].recv(1)
            raise AssertionError('opaque87 observation unexpectedly replied or closed')
        except socket.timeout:
            pass
        note('exact post-lobby opaque87 profile observed16 times without ACK or state change')
        send(channels[1], 87, codec.pack_fields([0], [2]), 94)
        channels[1].settimeout(3)
        closed(channels[1])
        note('17th opaque87 closes its channel at bounded budget')
    except Exception as caught:
        error = repr(caught)
    finally:
        for s in sockets:
            s.close()
        if proc is not None:
            if proc.poll() is None:
                os.killpg(proc.pid, signal.SIGTERM)
            try:
                code = proc.wait(timeout=8)
            except subprocess.TimeoutExpired:
                os.killpg(proc.pid, signal.SIGKILL)
                proc.wait(timeout=2)
                code = proc.returncode
                error = error or 'shutdown timeout'
        else:
            code = None
        if log:
            log.close()
        (root/'private-admin-transcript.json').write_text(json.dumps(calls, indent=2)+'\n')
        source_frozen = all(p.stat().st_size == spec['bytes'] and sha(p) == spec['sha256'] for p, spec in zip(sources, pins))
        socket_clean = not api.exists()
        if socket_clean:
            private.rmdir()
        with sqlite3.connect(db.as_uri()+'?mode=ro', uri=True) as conn:
            integrity = conn.execute('PRAGMA integrity_check').fetchone() == ('ok',)
            foreign_keys = not conn.execute('PRAGMA foreign_key_check').fetchall()
        conn.close()
        event_log = root/'service/rust-events.jsonl'
        secrets = [g['token'] for g in grants]+['serving-test-password']
        redacted = event_log.exists() and all(secret not in event_log.read_text() for secret in secrets)
        report = dict(result='pass' if error is None and code == 0 and source_frozen and socket_clean and integrity and foreign_keys and redacted else 'fail',
            error=error, duration_s=time.monotonic()-started, returncode=code, pid=proc.pid if proc else None,
            remaining_pid=proc.pid if proc and proc.poll() is None else None, source_frozen=source_frozen,
            socket_clean=socket_clean, integrity=integrity, foreign_keys=foreign_keys, redacted_logs=redacted, cases=cases,
            limits='Synthetic TCP/private company launcher API with actual Rust worker/SQLite; not a native KR run, general profile/room/game adapter, full authentication or M5 closure.')
        (root/'verification.json').write_text(json.dumps(report, indent=2)+'\n')
    assert report['result'] == 'pass', 'account serving test failed; private raw and error retained'
    print(f'PASS {len(cases)} synthetic account serving cases/source frozen/PID gone/control cleanup; not native KR proof')


if __name__ == '__main__':
    main()
