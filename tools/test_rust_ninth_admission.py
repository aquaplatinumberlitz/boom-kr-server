"""Real TCP synthetic ninth visitor controls, not native KR evidence."""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import shutil
import signal
import subprocess
import sys
import time
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from tools.rust_ninth_probe import seed_eight, connect_actor, join_body
from tools.test_rust_foundation import free_port
from server import probe_codec as c


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['binary', 'config', 'out']:
        parser.add_argument('--' + name, type=Path, required=True)
    args = parser.parse_args()
    root = args.out
    root.mkdir(parents=True, exist_ok=False)
    config = copy.deepcopy(json.loads(args.config.read_text()))
    two = config['diagnostic_two_lobby']
    assert len(two['actors']) == 9 and two['fixed_room']['ninth_admission_probe']
    ports = set()
    while len(ports) < 3:
        ports.add(free_port())
    login, wcp, channel = sorted(ports)
    config.update(bind='127.0.0.1', ports=sorted(ports), allowed_peers=['127.0.0.' + str(i) for i in range(2, 11)])
    two.update(login_port=login, channel_port=channel, login_candidate_ports=[login, wcp], wcp_ports=sorted(ports))
    for slot, actor in enumerate(two['actors']):
        actor['peer'] = config['allowed_peers'][slot]
    spec = root/'config.json'
    spec.write_text(json.dumps(config, indent=2) + '\n')
    sha = lambda p: hashlib.sha256(Path(p).read_bytes()).hexdigest()
    files = [args.binary, args.config, spec, Path(__file__), Path('tools/rust_ninth_probe.py'), Path('tools/test_rust_room.py'), Path('tools/test_rust_foundation.py'), Path('tools/export_rust_profile.py'), Path('server/probe_codec.py'), Path('server/room_title.py')]
    files += [p for p in Path('rust').rglob('*') if p.is_file() and 'target' not in p.parts]
    def fixtures(value):
        if isinstance(value, dict):
            if {'path', 'bytes', 'sha256'} <= value.keys():
                p = Path(value['path'])
                assert p.stat().st_size == value['bytes'] and sha(p) == value['sha256']
                files.append(p)
            for child in value.values():
                fixtures(child)
        elif isinstance(value, list):
            for child in value:
                fixtures(child)
    fixtures(config)
    frozen = root/'frozen-inputs'
    frozen.mkdir()
    pins = []
    for i, path in enumerate(sorted(set(files))):
        dst = frozen/f'{i:03d}-{path.name}'
        shutil.copyfile(path, dst)
        pins.append(dict(path=str(path), retained_path=str(dst), bytes=path.stat().st_size, sha256=sha(path)))
    (root/'preflight.json').write_text(json.dumps(dict(files=pins, scope='Synthetic nine lobby identities/eight room members; not native KR proof'), indent=2) + '\n')
    output = (root/'process.log').open('x')
    proc = subprocess.Popen([str(args.binary.resolve()), '--config', str(spec), '--out', str(root/'service')], stdout=output, stderr=subprocess.STDOUT)
    clients, probe, visitor, error, cases = [], None, None, None, []
    try:
        deadline = time.monotonic() + 5
        while time.monotonic() < deadline:
            state = root/'service/service-state.json'
            if state.exists() and json.loads(state.read_text())['ready']:
                break
            assert proc.poll() is None
            time.sleep(.01)
        else:
            raise AssertionError('service readiness timeout')
        clients = seed_eight(config)
        cases.append('eight setup-complete ready members/64roster fields')
        probe = connect_actor(config, 9)
        visitor = probe
        probe.receive(91)  # Current listing queued at channel open after room exists.
        cases.append('ninth diagnostic identity login/channel/profile, no room slot')
        for attempt in range(2):
            probe.send(82, join_body())
            assert c.read_fields(probe.receive(87)[1], [1]) == [2]
            for target in clients + [probe]:
                target.quiet()
            cases.append('full87 generic2/onebyte/continuouschannel attempt' + str(attempt+1))
        probe.close()
        probe = None
        for target in clients:
            target.quiet()
        cases.append('visitorEOF sends no roster removal to eight members')
        clients[0].send(104, bytes(20))
        for target in clients:
            target.receive(582)
            assert len(target.receive(121)[1]) == 285
        for slot, target in enumerate(clients):
            target.send(99, c.pack_fields([0, 0, 32001+slot, 0, 0, 0, 0, 0, 0], [4, 4, 2, 4, 2, 1, 1, 1, 4]))
            if slot < 7:
                for peer in clients:
                    peer.quiet()
        for target in clients:
            target.receive(197)
            target.receive(196)
            body = target.receive(153)[1]
            assert len(body) == 205 and c.read_fields(body[15:19], [4]) == [12346]
        cases.append('eight unchanged/Ready preserved/first round key12346/peer205 without ninth')
    except BaseException as exc:
        error = repr(exc)
        raise
    finally:
        if error is None:
            receivers = [dict(identity=i+1, next_sequence=client.rx) for i, client in enumerate(clients)]
            if visitor is not None:
                receivers.append(dict(identity=9, next_sequence=visitor.rx))
            (root/'wire-boundary.json').write_text(json.dumps(dict(
                receivers=receivers,
                limits='Decoded TCP frame boundaries before final member closes; visitor EOF and rejection assertions remain covered. Unread final cleanup notifications stay in raw logs. Not native KR evidence.'
            ), indent=2) + '\n')
        if probe:
            probe.close()
        for target in clients:
            target.close()
        if proc.poll() is None:
            proc.send_signal(signal.SIGTERM)
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()
            proc.wait(timeout=3)
        output.close()
        unchanged = all(Path(row['path']).stat().st_size == row['bytes'] and sha(row['path']) == row['sha256'] for row in pins)
        report = dict(result='pass' if error is None and proc.returncode == 0 and unchanged else 'fail', cases=cases, error=error, returncode=proc.returncode, source_frozen=unchanged, remaining_pid=None if proc.poll() is not None else proc.pid, binary_sha256=sha(args.binary), limits='Synthetic TCP only, diagnostic IDs, no native ninth/fullM4/auth proof.')
        report['raw_files'] = [dict(path=str(p), bytes=p.stat().st_size, sha256=sha(p)) for p in sorted(root.rglob('*')) if p.is_file()]
        (root/'verification.json').write_text(json.dumps(report, indent=2) + '\n')
        print(report['result'], len(cases), 'synthetic ninth cases')
        assert report['result'] == 'pass', error or 'cleanup/source failure'


if __name__ == '__main__':
    main()
