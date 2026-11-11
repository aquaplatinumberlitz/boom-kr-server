"""Synthetic TCP slot/identity separation and common106 pre-bind guards.

Uses retained fixture input pins. This does not certify native KR behavior.
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
from server import probe_codec as c
from server.room_title import _decode_bytes, _encode_bytes
from tools.test_rust_foundation import free_port
from tools.test_rust_room import Client


def configure(base):
    cfg = copy.deepcopy(base)
    ports = set()
    while len(ports) < 3:
        ports.add(free_port())
    login, wcp, channel = sorted(ports)
    cfg.update(bind='127.0.0.1', ports=sorted(ports), allowed_peers=['127.0.0.2', '127.0.0.3'])
    two = cfg['diagnostic_two_lobby']
    two.update(login_port=login, login_candidate_ports=[login, wcp], channel_port=channel, wcp_ports=sorted(ports))
    for i, actor in enumerate(two['actors']):
        actor['peer'] = cfg['allowed_peers'][i]
    return cfg


def stop(proc):
    if proc.poll() is None:
        os.killpg(proc.pid, signal.SIGTERM)
        try:
            proc.wait(timeout=4)
        except subprocess.TimeoutExpired:
            os.killpg(proc.pid, signal.SIGKILL)
            proc.wait(timeout=2)
    assert proc.poll() is not None


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--binary', type=Path, required=True)
    p.add_argument('--config', type=Path, required=True)
    p.add_argument('--out', type=Path, required=True)
    a = p.parse_args()
    a.out.mkdir(parents=True, exist_ok=False)
    base = json.loads(a.config.read_text())
    binary_sha = hashlib.sha256(a.binary.read_bytes()).hexdigest()
    cases = []
    for case in ['opaque-body', 'opaque-header', 'disagreeing-names', 'wrong-identity', 'invalid-slot-policy']:
        root = a.out / case
        root.mkdir()
        cfg = configure(base)
        room = cfg['diagnostic_two_lobby']['fixed_room']
        if case == 'invalid-slot-policy':
            room['slot_allocation'] = 'identity_is_slot'
        else:
            pin = room['packets']['player_B']
            raw = Path(pin['fixture']['path']).read_bytes()
            body = bytearray(c.frame_body(raw, pin['sequence']))
            header = bytearray(raw[:37])
            if case == 'opaque-body':
                body[-1] ^= 1
            elif case == 'opaque-header':
                header[30] ^= 1
            elif case == 'wrong-identity':
                body[1:5] = c.pack_fields([999], [4])
            else:
                size = c.read_fields(body[16:20], [4])[0]
                body[20 + size + 5] ^= 1
            raw = c.encode(header, 106, bytes(body), pin['sequence'])
            fixture = root / 'valid-pin-changed-field.bin'
            fixture.write_bytes(raw)
            pin['fixture'] = dict(path=str(fixture.resolve()), bytes=len(raw), sha256=hashlib.sha256(raw).hexdigest())
        cp = root / 'config.json'
        cp.write_text(json.dumps(cfg, indent=2) + '\n')
        with (root / 'process.log').open('x') as output:
            proc = subprocess.Popen([str(a.binary.resolve()), '--config', str(cp), '--out', str(root / 'service')], stdout=output, stderr=subprocess.STDOUT, start_new_session=True)
            try:
                rc = proc.wait(timeout=4)
            finally:
                stop(proc)
        assert rc != 0 and not (root / 'service/service-state.json').exists(), case
        error = (root / 'process.log').read_text()
        assert ('room player common-template mismatch' in error if case != 'invalid-slot-policy' else 'unknown variant' in error), (case, error)
        for port in cfg['ports']:
            with socket.socket() as sock:
                sock.bind(('127.0.0.1', port))
        cases.append(dict(case=case, result='pass', returncode=rc))

    root = a.out / 'first-vacant'
    root.mkdir()
    cfg = configure(base)
    two = cfg['diagnostic_two_lobby']
    two['fixed_room']['slot_allocation'] = 'first_vacant'
    cp = root / 'config.json'
    cp.write_text(json.dumps(cfg, indent=2) + '\n')
    hello = Path(cfg['bootstrap']['path']).read_bytes()
    header = Path(two['actors'][0]['profile']['own_header']['path']).read_bytes()[:37]
    clients = []
    with (root / 'process.log').open('x') as output:
        proc = subprocess.Popen([str(a.binary.resolve()), '--config', str(cp), '--out', str(root / 'service')], stdout=output, stderr=subprocess.STDOUT, start_new_session=True)
        try:
            deadline = time.monotonic() + 5
            while time.monotonic() < deadline:
                state = root / 'service/service-state.json'
                if state.exists() and json.loads(state.read_text())['ready']:
                    break
                assert proc.poll() is None
                time.sleep(.01)
            else:
                raise AssertionError('startup timeout')
            for role in [0, 1]:
                login = Client(role, two['login_port'], header, hello)
                try:
                    login.send(1, bytes([0x5e, 0x5b]) + c.pack_fields([13], [2]) + bytes(38))
                    login.receive(4)
                    login.send(9, bytes(8))
                    login.receive(15)
                    login.receive(62)
                    login.send(5, bytes(9))
                    login.receive(7)
                finally:
                    login.close()
                client = Client(role, two['channel_port'], header, hello)
                clients.append(client)
                name = two['actors'][role]['name'].encode('ascii')
                client.send(6, bytes(2) + c.pack_fields([len(name)], [2]) + _encode_bytes(name) + c.pack_fields([role + 1], [4]) + bytes(28))
                client.receive(17)
                client.send(67, bytes(2))
                for opcode in [241, 91, 14, 291]:
                    client.receive(opcode)
            A, B = clients
            title = two['fixed_room']['title'].encode('ascii')
            create = c.pack_fields([len(title)], [2]) + _encode_bytes(title) + c.pack_fields([0], [2]) + c.pack_fields([0, 0, 5, 8191, 0, 0, 0, 0xffffffff, 0, 0, 0, 0], [4] * 12) + c.pack_fields([0] * 4, [1] * 4)
            B.send(81, create)
            B.receive(86)
            A.receive(91)
            join = c.pack_fields([1, 1, 0, 1, 5], [2, 4, 2, 1, 2])
            A.send(82, join)
            assert c.read_fields(A.receive(87)[1][1:2], [1]) == [1]
            for client in clients:
                client.send(585, c.pack_fields([0], [4]))
            def roster(client):
                seen = {}
                while True:
                    op, body = client.receive()
                    if op == 106:
                        slot, identity = c.read_fields(body[:5], [1, 4])
                        state = c.read_fields(body[12:16], [4])[0] & 0xffff
                        size = c.read_fields(body[16:20], [4])[0]
                        profile = _decode_bytes(body[20:20 + size])
                        n = int.from_bytes(profile[6:10], 'little')
                        seen[identity] = (slot, state, profile[10:10 + n])
                    if op == 117:
                        return seen
            expected = {1: (1, 32, b'KRLocal'), 2: (0, 2, b'KRGuest')}
            assert roster(A) == expected
            assert roster(B) == expected
            cases.append(dict(case='B_creator_host_slot0_A_identity1_slot1_correct_names_both_views', result='pass'))
            B.send(86, b'')
            assert c.read_fields(A.receive(107)[1], [1, 4, 1]) == [0, 2, 0]
            assert c.read_fields(A.receive(110)[1], [1, 4, 1]) == [1, 34, 0]
            B.receive(241)
            B.receive(91)
            B.send(82, join)
            assert c.read_fields(B.receive(87)[1][1:2], [1]) == [0]
            B.send(585, c.pack_fields([0], [4]))
            assert roster(B) == {1: (1, 34, b'KRLocal'), 2: (0, 0, b'KRGuest')}
            peer = A.receive(106)[1]
            assert c.read_fields(peer[:5], [1, 4]) == [0, 2]
            assert c.read_fields(peer[12:16], [4])[0] & 0xffff == 0
            cases.append(dict(case='host_promotion_preserves_slot_rejoin_reuses_vacancy_without_restoring_old_host', result='pass'))
        finally:
            (root / 'wire-boundary.json').write_text(json.dumps(dict(receivers=[dict(identity=i + 1, next_sequence=client.rx) for i, client in enumerate(clients)]), indent=2) + '\n')
            for client in clients:
                client.close()
            stop(proc)
    assert hashlib.sha256(a.binary.read_bytes()).hexdigest() == binary_sha
    report = dict(result='pass', cases=cases, binary_sha256=binary_sha, binary_unchanged=True, remaining_pid=None, limits='Synthetic TCP and valid-pin config negatives only; not native KR proof, general account projection, shared-NAT transport or final R1 acceptance.')
    (a.out / 'verification.json').write_text(json.dumps(report, indent=2) + '\n')
    print('PASS', len(cases), 'slot/template cases; synthetic only')


if __name__ == '__main__':
    main()
