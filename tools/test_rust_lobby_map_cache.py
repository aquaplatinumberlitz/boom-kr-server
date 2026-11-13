"""Actual Rust/account/SQLite synthetic TCP cache controls, not native KR proof."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import time

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from server import probe_codec as codec
from tools.rust_account_fixture import AccountFixture
from tools.test_rust_foundation import free_port
from tools.test_rust_room import Client


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['binary', 'admin', 'config', 'out']:
        parser.add_argument('--'+name, type=Path, required=True)
    args = parser.parse_args()
    root = args.out.resolve()
    root.mkdir(parents=True, mode=0o700, exist_ok=False)
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
    frozen = root/'frozen'
    frozen.mkdir()
    inputs = [args.binary, args.admin, args.config, Path(__file__), config,
              *Path('rust/crates').rglob('*.rs'), *Path('rust/crates').rglob('*.sql')]
    def visit(value):
        if isinstance(value, dict):
            if {'path', 'bytes', 'sha256'} <= value.keys():
                p = Path(value['path'])
                assert p.stat().st_size == value['bytes'] and hashlib.sha256(p.read_bytes()).hexdigest() == value['sha256']
                inputs.append(p)
            for item in value.values():
                visit(item)
        elif isinstance(value, list):
            for item in value:
                visit(item)
    visit(cfg)
    pins = []
    for i, p in enumerate(dict.fromkeys(inputs)):
        data = p.read_bytes()
        retained = frozen/f'{i:03d}-{p.name}'
        retained.write_bytes(data)
        pins.append(dict(path=str(p.resolve()), bytes=len(data), sha256=hashlib.sha256(data).hexdigest(), retained=str(retained)))
    (root/'preflight.json').write_text(json.dumps(dict(files=pins, net_namespace=os.readlink('/proc/self/ns/net')), indent=2)+'\n')
    pin = two['fixed_room']['packets']['table']
    expected = codec.frame_body(Path(pin['fixture']['path']).read_bytes(), pin['sequence'])
    header = Path(cfg['login_error']['path']).read_bytes()[:37]
    # Existing Client expects the retained KR server-first hello, not a frame.
    hello = Path(cfg['bootstrap']['path']).read_bytes()
    clients = []
    cases = []
    process = None
    error = None
    def connect(role):
        value = account.connect(role, login, channel, header, hello, Client)
        clients.append(value)
        return value
    def fetch(client):
        client.send(528, b'')
        assert client.receive(238)[1] == expected
    def closed(client):
        client.sock.settimeout(4)
        try:
            assert client.sock.recv(1) == b''
        except ConnectionResetError:
            pass
        client.close()
        clients.remove(client)
    log = (root/'process.log').open('x')
    try:
        process = subprocess.Popen([str(args.binary.resolve()), '--config', str(config), '--out', str(root/'service')],
                                   stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
        for _ in range(500):
            ready = root/'service/service-state.json'
            if ready.exists() and json.loads(ready.read_text()).get('ready'):
                break
            assert process.poll() is None, 'startup failed'
            time.sleep(.01)
        else:
            raise AssertionError('readiness timeout')
        a, b = connect(0), connect(1)
        fetch(a)
        fetch(b)
        cases.append('two authenticated lobby owners receive exact current238 without room membership')
        a.send(528, bytes(1))
        closed(a)
        fetch(b)
        cases.append('nonempty528 closes only offending channel; other account retains current positive')
        time.sleep(.1)
        a = connect(0)
        for _ in range(16):
            fetch(a)
        a.send(528, b'')
        closed(a)
        fetch(b)
        cases.append('sixteen bounded cache requests succeed; seventeenth closes only that channel')
        account.command(dict(op='revoke', account_id=account.players[1]['account_id']))
        closed(b)
        time.sleep(.1)
        a = connect(0)
        fetch(a)
        cases.append('revoke closes owned channel; fresh nonrevoked account still fetches current table')
        for client in clients:
            client.close()
        clients.clear()
        os.killpg(process.pid, signal.SIGTERM)
        assert process.wait(timeout=6) == 0
        assert not account.socket.exists()
        account.close()
        cases.append('owned shutdown and launcher cleanup')
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
        frozen_ok = all(Path(p['path']).stat().st_size == p['bytes'] and hashlib.sha256(Path(p['path']).read_bytes()).hexdigest() == p['sha256'] for p in pins)
        (root/'result.json').write_text(json.dumps(dict(result='pass' if error is None else 'fail', cases=cases, error=error,
            returncode=None if process is None else process.returncode, source_frozen=frozen_ok,
            limits='Synthetic TCP/actual account DB/cache policy controls; native Factory07 proof is separate.'), indent=2)+'\n')
        assert frozen_ok


if __name__ == '__main__':
    main()
