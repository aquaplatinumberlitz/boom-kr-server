"""Loopback TCP checks for operator-bound Rust lobby policy, not native KR proof.

All requests below are synthetic policy inputs. Native acceptance uses the
separately retained Wine traces/cursors/render and packet capture.
"""
import argparse
import copy
import hashlib
import json
import os
from pathlib import Path
import signal
import socket
import subprocess
import sys
import time

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from server import probe_codec as codec
from server.room_title import _encode_bytes
from tools.test_rust_foundation import exact, free_port


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--binary', type=Path, required=True)
    p.add_argument('--config', type=Path, required=True)
    p.add_argument('--out', type=Path, required=True)
    a = p.parse_args()
    a.out.mkdir(parents=True, exist_ok=False)
    cfg = json.loads(a.config.read_text())
    ports = set()
    while len(ports) < 3:
        ports.add(free_port())
    wcp, login, channel = sorted(ports)
    cfg.update(bind='127.0.0.1', allowed_peers=['127.0.0.2', '127.0.0.3'], ports=sorted(ports))
    endpoints = cfg['diagnostic_two_lobby']
    endpoints.update(wcp_ports=sorted(ports), login_port=login, channel_port=channel)
    for i, actor in enumerate(endpoints['actors']):
        actor['peer'] = cfg['allowed_peers'][i]
    hello = Path(cfg['bootstrap']['path']).read_bytes()
    wcp_reply = Path(cfg['wcp_second']['path']).read_bytes()
    header = Path(endpoints['actors'][0]['profile']['own_header']['path']).read_bytes()[:37]
    cases, sockets = [], []
    proc, output = None, None

    def note(case):
        cases.append(dict(case=case, result='pass'))

    def launch(config, name, ready=True):
        nonlocal proc, output
        cp = a.out/(name+'-config.json')
        cp.write_text(json.dumps(config, indent=2)+'\n')
        output = (a.out/(name+'.log')).open('x')
        proc = subprocess.Popen([str(a.binary.resolve()), '--config', str(cp), '--out', str(a.out/name)],
                                stdout=output, stderr=subprocess.STDOUT, start_new_session=True)
        if ready:
            deadline = time.monotonic()+5
            while time.monotonic() < deadline:
                f = a.out/name/'service-state.json'
                if f.exists() and json.loads(f.read_text())['ready']:
                    return
                assert proc.poll() is None, 'startup failure'
                time.sleep(.01)
            raise AssertionError('startup timeout')

    def stop():
        nonlocal proc, output
        if proc is None:
            return
        if proc.poll() is None:
            os.killpg(proc.pid, signal.SIGTERM)
        try:
            code = proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            os.killpg(proc.pid, signal.SIGKILL)
            proc.wait(timeout=2)
            raise AssertionError('shutdown timeout')
        finally:
            output.close()
            proc = None
        return code

    def connect(role, port):
        s = socket.socket()
        s.settimeout(2)
        s.bind(('127.0.0.'+str(role+2), 0))
        s.connect(('127.0.0.1', port))
        sockets.append(s)
        assert exact(s, 15) == hello
        return s

    def closed(s):
        try:
            assert s.recv(1) == b''
        except ConnectionResetError:
            pass
        s.close()
        time.sleep(.02)

    def send(s, op, body, sequence=40):
        s.sendall(codec.encode(header, op, body, sequence))

    def reply(s, op, sequence):
        prefix = exact(s, 37)
        n = int.from_bytes(prefix[13:17], 'little')
        assert 0 <= n <= 65536
        frame = prefix + exact(s, n+1)
        assert int.from_bytes(frame[5:9], 'little') == op
        return codec.frame_body(frame, sequence)

    login_body = bytes([0x5e, 0x5b])+codec.pack_fields([13], [2])+bytes(38)

    def channel_body(role):
        name = [b'KRLocal', b'KRGuest'][role]
        return bytes(2)+codec.pack_fields([len(name)], [2])+_encode_bytes(name)+codec.pack_fields([role+1], [4])+bytes(28)

    error = None
    try:
        launch(cfg, 'bindings')
        # Interleave actors and ports: empty bootstrap sockets allocate no lease.
        connect(0, login).close()
        connect(1, channel).close()
        time.sleep(.03)
        for role, port in [(1, login), (0, channel)]:
            s = connect(role, port)
            frame = codec.encode(header, 0, bytes(28), 40)
            for part in [frame[:1], frame[1:19], frame[19:]]:
                s.sendall(part)
            assert exact(s, len(wcp_reply)) == wcp_reply
            s.close()
        note('WCP_request_on_login_and_channel_ports_no_global_accept_mapping')
        s = connect(0, channel)
        send(s, 6, channel_body(0))
        closed(s)
        note('channel_without_redirect_denied')
        s = connect(0, wcp)
        send(s, 1, login_body)
        closed(s)
        note('login_on_wrong_endpoint_denied_without_lease')
        s = connect(0, login)
        bad = bytearray(codec.encode(header, 1, login_body, 40))
        bad[-1] ^= 1
        s.sendall(bad)
        closed(s)
        note('invalid_CRC_does_not_reserve_identity')
        logins = {}
        for role in [1, 0]:
            s = connect(role, login)
            send(s, 1, login_body)
            auth = reply(s, 4, 43)
            assert codec.read_fields(auth[25:29], [4]) == [role+1]
            logins[role] = s
        note('interleaved_login_profiles_keep_distinct_IDs')
        s = connect(0, login)
        send(s, 1, login_body)
        closed(s)
        note('duplicate_active_login_preserves_owner')
        for role in [0, 1]:
            s = logins[role]
            send(s, 9, bytes(8), 43)
            reply(s, 15, 46)
            reply(s, 62, 49)
            send(s, 5, bytes(9), 46)
            reply(s, 7, 52)
            s.close()
        time.sleep(.04)
        note('redirect_grants_per_identity_survive_login_EOF')
        s = connect(0, channel)
        send(s, 6, channel_body(1))
        closed(s)
        note('wrong_channel_identity_does_not_consume_owner_grant')
        channels = {}
        for role in [1, 0]:
            s = connect(role, channel)
            send(s, 6, channel_body(role))
            reply(s, 17, 43)
            send(s, 67, bytes(2), 43)
            for op, seq in [(241,46), (91,49), (14,52), (291,55)]:
                reply(s, op, seq)
            channels[role] = s
        note('both_channel_IDs_profiles_sequence_reset')
        s = connect(0, channel)
        send(s, 6, channel_body(0))
        closed(s)
        note('duplicate_channel_denied_survivor_unchanged')
        send(channels[0], 6, channel_body(0), 46)
        closed(channels[0])
        note('duplicate_identity_packet_closes_only_its_connection')
        # B remains usable after A failure; no second profile burst for ancillary.
        send(channels[1], 715, b'', 46)
        channels[1].settimeout(.1)
        try:
            got = channels[1].recv(1)
        except socket.timeout:
            got = None
        assert got is None
        channels[1].close()
        time.sleep(.03)
        s = connect(0, channel)
        send(s, 6, channel_body(0))
        closed(s)
        note('bound_channel_EOF_revokes_old_redirect')
        assert stop() == 0
        note('bounded_SIGTERM_cleanup_after_negative_connections')
        for name, mutate in [
            ('duplicate-peer', lambda x: x['diagnostic_two_lobby']['actors'][1].update(peer='127.0.0.2')),
            ('mixed-mode', lambda x: x.update(diagnostic_login_error_only=True)),
            ('wrong-profile-ID', lambda x: x['diagnostic_two_lobby']['actors'][1].update(identity=1, name='KRLocal')),
        ]:
            bad = copy.deepcopy(cfg)
            mutate(bad)
            launch(bad, name, ready=False)
            assert proc.wait(timeout=3) != 0
            assert not (a.out/name/'service-state.json').exists()
            stop()
            note(name+'_prelisten_rejected')
    except Exception as e:
        error = repr(e)
        raise
    finally:
        for s in sockets:
            s.close()
        stop()
        rows = [dict(path=str(p), bytes=p.stat().st_size, sha256=hashlib.sha256(p.read_bytes()).hexdigest())
                for p in sorted(a.out.rglob('*')) if p.is_file()]
        (a.out/'verification.json').write_text(json.dumps(dict(result='pass' if error is None else 'fail',
            error=error, cases=cases, raw_files=rows, binary_sha256=hashlib.sha256(a.binary.read_bytes()).hexdigest(),
            limits='Synthetic policy/kernelTCP checks, not KR native acceptance or account security.'), indent=2)+'\n')
    print('PASS', len(cases), 'Rust diagnostic binding kernelTCP/policy cases; not native proof')


if __name__ == '__main__':
    main()
