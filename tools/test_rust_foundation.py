"""Real loopback TCP/process negatives; independent native acceptance is separate.

Input login frame must be an existing captured KR C2S1 frame, not this encoder's
output. Python encoding below is used only for a duplicate/stage policy negative.
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
from tools.export_rust_profile import export


def free_port():
    with socket.socket() as s:
        s.bind(('127.0.0.1', 0))
        return s.getsockname()[1]


def exact(sock, n):
    data = b''
    while len(data) < n:
        part = sock.recv(n-len(data))
        if not part:
            raise AssertionError('unexpected EOF')
        data += part
    return data


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--binary', type=Path, required=True)
    p.add_argument('--python-profile', type=Path, required=True)
    p.add_argument('--login-error', type=Path, required=True)
    p.add_argument('--captured-login-frame', type=Path, required=True)
    p.add_argument('--out', type=Path, required=True)
    a = p.parse_args()
    a.out.mkdir(parents=True, exist_ok=False)
    a.binary = a.binary.resolve(strict=True)
    frame = a.captured_login_frame.read_bytes()
    assert int.from_bytes(frame[5:9], 'little') == 1
    plain = codec.frame_body(frame, 40)
    fixture = a.login_error.read_bytes()
    config = export(a.python_profile, a.login_error, ports=[free_port()], idle_seconds=2, max_connections=2)
    hello = Path(config['bootstrap']['path']).read_bytes()
    cases, sockets = [], []
    process = None
    stream = None

    def note(name):
        cases.append(dict(case=name, result='pass'))

    def launch(cfg, name, expect_ready=True):
        nonlocal process, stream
        cp = a.out/(name+'-config.json')
        with cp.open('x') as f:
            json.dump(cfg, f, indent=2)
        out = a.out/name
        stream = (a.out/(name+'-process.log')).open('x')
        process = subprocess.Popen([str(a.binary), '--config', str(cp), '--out', str(out)],
                                   stdout=stream, stderr=subprocess.STDOUT, start_new_session=True)
        if expect_ready:
            until = time.monotonic()+5
            while time.monotonic() < until:
                state = out/'service-state.json'
                if state.exists() and json.loads(state.read_text())['ready']:
                    break
                assert process.poll() is None, 'service startup failed'
                time.sleep(.02)
            else:
                raise AssertionError('ready timeout')
        return out

    def stop():
        nonlocal process, stream
        if process is not None:
            if process.poll() is None:
                os.killpg(process.pid, signal.SIGTERM)
                try:
                    process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    os.killpg(process.pid, signal.SIGKILL)
                    process.wait(timeout=2)
                    raise AssertionError('shutdown deadline exceeded')
            rc = process.returncode
            process = None
            stream.close()
            stream = None
            return rc

    port = config['ports'][0]

    def connect():
        s = socket.create_connection(('127.0.0.1', port), timeout=2)
        sockets.append(s)
        assert exact(s, 15) == hello
        return s

    def closed(s):
        try:
            assert s.recv(1) == b''
        except ConnectionResetError:
            pass
        s.close()

    error = None
    try:
        out = launch(config, 'transport')
        # Explicit CP181 diagnostic WCP mapping, not account/session routing.
        for _ in range(2):
            s = connect()
            s.close()
            time.sleep(.05)
        note('fresh_connection_hello_pins')
        s = connect()
        for part in [frame[:1], frame[1:17], frame[17:37], frame[37:]]:
            s.sendall(part)
        assert exact(s, 39) == fixture
        for opcode, length in [(60,20),(15,31)]:
            seq=43 if opcode==60 else 46
            s.sendall(codec.encode(frame[:37],opcode,bytes(length),seq))
        s.settimeout(.15)
        try:
            extra=s.recv(1)
        except socket.timeout:
            pass
        else:
            raise AssertionError('ancillary policy unexpectedly closed/replied: '+repr(extra))
        s.close()
        time.sleep(.05)
        note('captured_native_login_split_exact_result1')
        note('observed_ancillary_shapes_keep_socket_open_without_reply')
        s = connect()
        bad = frame[:-1]+bytes([frame[-1] ^ 1])
        s.sendall(bad)
        closed(s)
        note('invalid_crc_no_reply_or_dispatch')
        s = connect()
        oversized = bytearray(frame[:38])
        oversized[13:17] = (2**32-1).to_bytes(4, 'little')
        s.sendall(oversized)
        closed(s)
        note('oversized_header_rejected_without_body_allocation')
        s = connect()
        s.sendall(frame[:20])
        s.shutdown(socket.SHUT_WR)
        closed(s)
        note('partial_eof_no_reply')
        s = connect()
        duplicate = codec.encode(frame[:37], 1, plain, 43)
        s.sendall(frame+duplicate)
        assert exact(s, 39) == fixture
        closed(s)
        note('coalesced_duplicate_valid_frame_single_action_wrong_stage_closed')
        s = connect()
        s.sendall(frame[:5])
        s.settimeout(4)
        closed(s)
        note('partial_idle_timeout_releases_connection')
        time.sleep(.1)
        first, second = connect(), connect()
        third = socket.create_connection(('127.0.0.1', port), timeout=2)
        sockets.append(third)
        closed(third)
        first.close()
        second.close()
        time.sleep(.1)
        note('bounded_admission_without_extra_hello')
        s = connect()
        s.sendall(frame)
        assert exact(s, 39) == fixture
        note('fresh_connection_after_negatives_resets_sequence')
        assert stop() == 0
        closed(s)
        state = json.loads((out/'service-state.json').read_text())
        assert state['stopped'] and not state['ready']
        events = [json.loads(line) for line in (out/'rust-events.jsonl').read_text().splitlines()]
        replies = [x for x in events if x['event']=='login_error_sent']
        assert len(replies)==3
        assert any(x['event']=='closed' and x['reason']=='Checksum' for x in events)
        assert any(x['event']=='closed' and x['reason']=='FrameTooLarge' for x in events)
        assert any(x['event']=='closed' and x['reason']=='TruncatedEof' for x in events)
        assert any(x['event']=='closed' and x['reason']=='read idle timeout' for x in events)
        assert all('payload' not in x and 'token' not in x for x in events)
        note('graceful_sigterm_drains_and_closes_active_socket')
        launch(config, 'restart')
        s = connect()
        s.close()
        assert stop()==0
        note('fresh_server_rebinds_with_fresh_codec')
        bad_config = copy.deepcopy(config)
        bad_config['login_error']['sha256']='0'*64
        launch(bad_config, 'bad-pin', False)
        assert process.wait(timeout=5)!=0
        stop()
        with socket.socket() as s:
            s.setsockopt(socket.SOL_SOCKET,socket.SO_REUSEADDR,1)
            s.bind(('127.0.0.1', port))
        note('fixture_hash_failure_before_bind')
        with socket.socket() as busy:
            busy.bind(('127.0.0.1',0))
            busy.listen()
            failed = copy.deepcopy(config)
            failed['ports']=[port,busy.getsockname()[1]]
            launch(failed,'bind-conflict',False)
            assert process.wait(timeout=5)!=0
            stop()
        with socket.socket() as s:
            s.setsockopt(socket.SOL_SOCKET,socket.SO_REUSEADDR,1)
            s.bind(('127.0.0.1', port))
        note('multi_listener_startup_failure_releases_previously_bound_port')
    except Exception as exc:
        error = repr(exc)
    finally:
        for s in sockets:
            s.close()
        stop()
        report=dict(result='fail' if error else 'pass', error=error, cases=cases,
                    binary_sha256=hashlib.sha256(a.binary.read_bytes()).hexdigest(),
                    captured_input_sha256=hashlib.sha256(frame).hexdigest(),
                    limits='KernelTCP/process policy negatives, not native KR acceptance. Diagnostic first2WCP transports/loopback/result1 only; no accountauthentication/channel/gameplay/UDP/DB/security/SLO claim. Python-generated duplicate is a policy input, not protocol oracle.')
        with (a.out/'verification.json').open('x') as f:
            json.dump(report,f,indent=2)
        print(json.dumps(dict(result=report['result'],cases=len(cases),error=error)))
    if error:
        raise SystemExit(1)


if __name__ == '__main__':
    main()
