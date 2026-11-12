"""M7b managed population from real account membership; synthetic TCP only."""
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
    for name in ['binary', 'admin', 'config', 'out']:
        parser.add_argument('--'+name, type=Path, required=True)
    args = parser.parse_args()
    root = args.out.resolve()
    root.mkdir(parents=True, exist_ok=False, mode=0o700)
    cfg = json.loads(args.config.read_text())
    two = cfg['diagnostic_two_lobby']
    assert len(two['actors']) == 4
    ports = set()
    while len(ports) < 3:
        ports.add(free_port())
    login, wcp, channel = sorted(ports)
    cfg.update(bind='127.0.0.1', allowed_peers=[f'127.0.0.{n+2}' for n in range(4)], ports=sorted(ports))
    two.update(login_port=login, login_candidate_ports=[login, wcp], wcp_ports=sorted(ports),
               channel_port=channel, managed_rooms=dict(max_rooms=2))
    for actor, peer in zip(two['actors'], cfg['allowed_peers']):
        actor['peer'] = peer
    account = AccountFixture(args.admin, root, cfg)
    config = root/'config.json'
    config.write_text(json.dumps(cfg, indent=2)+'\n')
    files = [args.binary, args.admin, config, args.config, Path(__file__), Path('tools/rust_account_fixture.py'),
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
        for client in [b, c, d]:
            listing(client, {(first, first): (1, 4)}, 'native_carrier_candidate_current1_capacity4')
        second = create(c, b'Beta')
        for client in [b, d]:
            listing(client, {(first, first): (1, 4), (second, second): (1, 4)}, 'independent_initial_populations')
        b.send(82, codec.pack_fields([first, first, 0, 1, 5], [2, 4, 2, 1, 2]))
        b.receive(87)
        listing(d, {(first, first): (2, 4), (second, second): (1, 4)}, 'join_changes_only_selected_room')
        b.send(86, b'')
        a.receive(107)
        a.receive(110)
        b.receive(241)
        listing(b, {(first, first): (1, 4), (second, second): (1, 4)}, 'leaver_returns_with_current_populations')
        listing(d, {(first, first): (1, 4), (second, second): (1, 4)}, 'leave_updates_existing_observer')
        # Reject a tuple from another room without acquiring a phantom slot.
        b.send(82, codec.pack_fields([first, second, 0, 1, 5], [2, 4, 2, 1, 2]))
        try:
            assert b.sock.recv(1) == b''
        except ConnectionResetError:
            pass
        b.close()
        clients.remove(b)
        d.quiet()
        cases.append('wrong_tuple_has_no_population_or_watch_effect')
        c.close()
        clients.remove(c)
        d.receive(90)
        listing(d, {(first, first): (1, 4)}, 'last_EOF_removes_exact_room')
        a.send(86, b'')
        a.receive(241)
        # A never saw a card before selecting its first room.
        listing(a, {}, 'last_Back_retires_room')
        d.receive(90)
        listing(d, {}, 'active_lobby_observer_removal_then_empty')
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
            limits='Synthetic candidate field/ownership projection only; native reader/UI required before confirming field semantics.'), indent=2)+'\n')
        assert all(Path(p['path']).stat().st_size == p['bytes'] and hashlib.sha256(Path(p['path']).read_bytes()).hexdigest() == p['sha256'] for p in pins)


if __name__ == '__main__':
    main()
