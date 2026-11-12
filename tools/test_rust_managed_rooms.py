"""M7a real SQLite/private-launcher/synthetic TCP. Not native client evidence."""
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
from server.room_title import _encode_bytes
from tools.rust_account_fixture import AccountFixture
from tools.test_rust_foundation import free_port
from tools.test_rust_room import Client


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
    sources = [args.binary, args.admin, args.config, config, Path(__file__),
               Path('tools/rust_account_fixture.py'), Path('tools/test_rust_room.py'),
               *Path('rust/crates').rglob('*.rs'), *Path('rust/crates').rglob('*.sql'), Path('rust/Cargo.lock')]
    def fixtures(obj):
        if isinstance(obj, dict):
            if {'path', 'bytes', 'sha256'} <= obj.keys():
                p = Path(obj['path'])
                assert p.stat().st_size == obj['bytes'] and hashlib.sha256(p.read_bytes()).hexdigest() == obj['sha256']
                sources.append(p)
            for value in obj.values():
                fixtures(value)
        elif isinstance(obj, list):
            for value in obj:
                fixtures(value)
    fixtures(cfg)
    frozen = root/'frozen'
    frozen.mkdir()
    pins = []
    for index, source in enumerate(dict.fromkeys(sources)):
        dest = frozen/f'{index:03d}-{source.name}'
        shutil.copyfile(source, dest)
        pins.append(dict(path=str(source), retained_path=str(dest), bytes=source.stat().st_size,
                         sha256=hashlib.sha256(source.read_bytes()).hexdigest()))
    (root/'preflight.json').write_text(json.dumps(dict(files=pins, limits='Synthetic TCP and real SQLite, not native. DB/admin calls are declared runtime inputs.'), indent=2)+'\n')
    hello = Path(cfg['bootstrap']['path']).read_bytes()
    header = Path(two['actors'][0]['profile']['own_header']['path']).read_bytes()[:37]
    clients = []
    observed = []
    cases = []
    process = None
    error = None
    log = (root/'process.log').open('x')
    def connect(index):
        client = account.connect(index, login, channel, header, hello, Client)
        clients.append(client)
        observed.append(client)
        return client
    def note(case):
        cases.append(case)
    def create(client, title):
        body = (codec.pack_fields([len(title)], [2])+_encode_bytes(title)+codec.pack_fields([0], [2])
                +codec.pack_fields([0, 0, 5, 8191, 0, 0, 0, 0xffffffff, 0, 0, 0, 0], [4]*12)
                +codec.pack_fields([0]*4, [1]*4))
        client.send(81, body)
        _, response = client.receive(86)
        session = codec.read_fields(response[1:3], [2])[0]
        assert session > 0
        return session
    def join(client, session):
        client.send(82, codec.pack_fields([session, session, 0, 1, 5], [2, 4, 2, 1, 2]))
        _, response = client.receive(87)
        assert codec.read_fields(response[2:4], [2])[0] == session
        return codec.read_fields(response[1:2], [1])[0]
    def setup(client, own, roster):
        client.send(585, codec.pack_fields([0], [4]))
        ids = []
        while True:
            op, body = client.receive()
            if op == 106:
                ids.append(codec.read_fields(body[1:5], [4])[0])
            if op == 117:
                break
        assert ids[0] == own and sorted(ids) == sorted(roster), ids
    def catalog(client):
        # watch coalesces changes; consume the current list notification.
        client.receive(91)
    def closed(client):
        client.sock.settimeout(4)
        try:
            assert client.sock.recv(1) == b''
        except ConnectionResetError:
            pass
    def leave(client, removed=0):
        client.send(86, b'')
        client.receive(241)
        for _ in range(removed):
            client.receive(90)
        catalog(client)
    try:
        process = subprocess.Popen([str(args.binary.resolve()), '--config', str(config), '--out', str(root/'service')],
                                   stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
        for _ in range(500):
            state = root/'service/service-state.json'
            if state.exists() and json.loads(state.read_text()).get('ready'):
                break
            assert process.poll() is None, 'startup failure'
            time.sleep(.01)
        else:
            raise AssertionError('readiness timeout')
        a, b, c, d = [connect(i) for i in range(4)]
        for client in clients:
            client.quiet()
        note('four authenticated accounts remain in lobby without room membership')
        first = create(a, b'Alpha')
        for client in [b, c, d]:
            catalog(client)
        second = create(c, b'Beta')
        assert first != second
        for client in [b, d]:
            catalog(client)
        assert join(b, first) == 1
        assert join(d, second) == 1
        setup(a, 1, [1, 2])
        setup(b, 2, [1, 2])
        # Initial setup already includes the current member, so no duplicate peer106.
        setup(c, 3, [3, 4])
        setup(d, 4, [3, 4])
        for client in clients:
            client.quiet()
        note('two public tuples and independent rosters; identity3 owns slot0 in second room')
        a.send(86, b'')
        _, clear = b.receive(107)
        assert codec.read_fields(clear[1:5], [4])[0] == 1
        b.receive(110)
        a.receive(241)
        catalog(a)
        assert join(a, second) == 2
        setup(a, 1, [1, 3, 4])
        c.receive(106)
        d.receive(106)
        b.quiet()
        note('host leave transfers first room chief and exact cross-room rejoin uses vacant slot2')
        # Native UI errors for this rejection belong to M7c; current design fails closed.
        b.send(82, codec.pack_fields([second, first, 0, 1, 5], [2, 4, 2, 1, 2]))
        closed(b)
        b.close()
        clients.remove(b)
        time.sleep(.1)
        # EOF of sole remaining first-room member retires it. A will receive90 on return.
        leave(a, removed=1)
        c.receive(107)
        c.receive(110)
        d.receive(107)
        d.receive(110)
        # Returning A has a previously known first tuple and must remove it.
        note('wrong selection while already member fails closed; room-local EOF cleanup')
        a.close()
        clients.remove(a)
        account.command(dict(op='revoke', account_id=account.players[2]['account_id']))
        d.receive(107)
        d.receive(110)
        closed(c)
        c.close()
        clients.remove(c)
        note('revoke removes only selected membership and transfers chief in second room')
        d.send(86, b'')
        d.receive(241)
        # D last saw both cards before joining: deletes both then empty91.
        d.receive(90)
        d.receive(90)
        _, body = d.receive(91)
        assert codec.read_fields(body, [2]) == [0]
        new = create(d, b'Gamma')
        assert new not in (first, second)
        note('empty-room retirement sends matching90 before91 and public tuple never reused')
        d.send(86, b'')
        d.receive(241)
        d.receive(91)
        for client in clients:
            client.close()
        clients.clear()
        os.killpg(process.pid, signal.SIGTERM)
        assert process.wait(timeout=6) == 0
        assert not account.socket.exists()
        account.close()
        note('positive create after rejection/revoke and owned shutdown')
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
        (root/'result.json').write_text(json.dumps(dict(cases=cases, error=error,
            returncode=None if process is None else process.returncode,
            limits='Source/TCP/design policy verification only. Native M7a remains required.'), indent=2)+'\n')
        # Keep runtime static inputs immutable through cleanup.
        assert all(Path(row['path']).stat().st_size == row['bytes'] and hashlib.sha256(Path(row['path']).read_bytes()).hexdigest() == row['sha256'] for row in pins)


if __name__ == '__main__':
    main()
