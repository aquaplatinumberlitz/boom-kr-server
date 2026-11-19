"""Smoke gói trong filesystem/network namespace không nhìn thấy repo/local.

Đây là kiểm isolation trên cùng kernel/ABI, không phải máy Linux sạch hoặc
nghiệm thu Internet/native/load. Chỉ dùng package/state riêng trong local/.
"""
import argparse
import hashlib
import ipaddress
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bundle', type=Path, required=True)
    parser.add_argument('--manifest-sha256', required=True)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--inside', action='store_true', help=argparse.SUPPRESS)
    args = parser.parse_args()
    local = (ROOT / 'local').resolve()
    bundle = args.bundle.resolve(strict=True)
    out = args.out.resolve()
    if any(path == local or not path.is_relative_to(local) for path in [bundle, out]):
        parser.error('Package và raw chỉ ở local/')
    if bundle.is_relative_to(out) or out.is_relative_to(bundle):
        parser.error('Package và output phải riêng')
    if not args.inside:
        out.mkdir(mode=0o700, parents=False, exist_ok=False)
        command = ['unshare', '--user', '--map-root-user', '--net', '--mount', '--pid', '--fork',
                   '--mount-proc', '--kill-child=KILL', sys.executable, str(Path(__file__).resolve()),
                   '--inside', '--bundle', str(bundle), '--manifest-sha256', args.manifest_sha256,
                   '--out', str(out)]
        with (out / 'namespace.log').open('x') as stream:
            result = subprocess.run(command, stdout=stream, stderr=subprocess.STDOUT, timeout=90)
        (out / 'exit.json').write_text(json.dumps(dict(returncode=result.returncode)) + '\n')
        if result.returncode:
            raise SystemExit(result.returncode)
        print('Package namespace smoke passed; clean-machine/Internet acceptance remains separate.')
        return
    assert os.getpid() == 1 and os.getuid() == 0
    assert Path('/proc/self/uid_map').read_text().split()[::2] == ['0', '1']
    subprocess.run(['mount', '--make-rprivate', '/'], check=True)
    subprocess.run(['ip', 'link', 'set', 'lo', 'up'], check=True)
    config = json.loads((bundle / 'config.template.json').read_text())
    addresses = {config['bind'], config['authenticated_relay']['bind'].rsplit(':', 1)[0]}
    for value in sorted(addresses):
        address = ipaddress.IPv4Address(value)
        if not address.is_unspecified and not address.is_loopback:
            subprocess.run(['ip', 'addr', 'add', str(address) + '/32', 'dev', 'lo'], check=True)
    state = out / 'state'
    state.mkdir(mode=0o700)
    common = ['bwrap', '--die-with-parent', '--unshare-pid', '--ro-bind', '/usr', '/usr',
              '--symlink', 'usr/lib', '/lib', '--symlink', 'usr/lib', '/lib64',
              '--symlink', 'usr/bin', '/bin', '--proc', '/proc', '--dev', '/dev',
              '--tmpfs', '/tmp', '--tmpfs', '/run', '--dir', '/etc',
              '--ro-bind', str(bundle), '/opt/kr', '--bind', str(state), '/var/lib/kr',
              '--chdir', '/opt/kr']
    probe = 'import pathlib; assert not pathlib.Path("/home").exists(); assert not pathlib.Path("/root").exists(); print("repository_and_local_hidden")'
    checked = subprocess.run(common + ['python3', '-c', probe], capture_output=True, text=True)
    (out / 'isolation-probe.log').write_text(checked.stdout + checked.stderr)
    checked.check_returncode()
    runner = ['python3', '/opt/kr/kr-package.py']
    options = ['--manifest-sha256', args.manifest_sha256, '--state', '/var/lib/kr']
    with (out / 'init-db.log').open('x') as stream:
        subprocess.run(common + runner + ['init-db', *options], stdout=stream, stderr=subprocess.STDOUT, check=True)
    database = state / 'accounts.sqlite'
    initial_database_hash = hashlib.sha256(database.read_bytes()).hexdigest()
    with (out / 'duplicate-init.log').open('x') as stream:
        duplicate_init = subprocess.run(common + runner + ['init-db', *options],
                                        stdout=stream, stderr=subprocess.STDOUT, timeout=12)
    assert duplicate_init.returncode != 0, 'init-db must refuse an existing database'
    assert hashlib.sha256(database.read_bytes()).hexdigest() == initial_database_hash
    started = time.monotonic()
    with (out / 'service.log').open('x') as stream:
        child = subprocess.Popen(common + runner + ['run', *options], stdout=stream,
                                 stderr=subprocess.STDOUT, start_new_session=True)
        try:
            deadline = time.monotonic() + 25
            ready = []
            while not ready:
                for path in state.glob('runs/*/service/service-state.json'):
                    try:
                        value = json.loads(path.read_text())
                    except json.JSONDecodeError:
                        continue
                    if value.get('ready') and not value.get('stopped'):
                        ready.append(path)
                if ready:
                    break
                if child.poll() is not None:
                    raise RuntimeError('Packaged service exited before ready')
                if time.monotonic() > deadline:
                    raise TimeoutError('Packaged service readiness deadline')
                time.sleep(.05)
            service_pids = []
            for proc in Path('/proc').iterdir():
                if not proc.name.isdigit():
                    continue
                try:
                    if os.readlink(proc / 'exe') == '/opt/kr/bin/kr-service':
                        service_pids.append(int(proc.name))
                except OSError:
                    pass
            assert len(service_pids) == 1, service_pids
            with (out / 'duplicate-run.log').open('x') as duplicate_log:
                duplicate_run = subprocess.run(common + runner + ['run', *options],
                                               stdout=duplicate_log, stderr=subprocess.STDOUT,
                                               timeout=12)
            assert duplicate_run.returncode != 0, 'second runner must refuse the locked state'
            assert child.poll() is None, 'duplicate runner must not stop the original service'
            assert all(json.loads(path.read_text()).get('ready') and
                       not json.loads(path.read_text()).get('stopped') for path in ready)
            os.kill(service_pids[0], signal.SIGTERM)
            code = child.wait(timeout=12)
            assert code == 0, ('packaged graceful stop', code)
            assert all(json.loads(path.read_text()).get('stopped') for path in ready)
        finally:
            if child.poll() is None:
                os.killpg(child.pid, signal.SIGKILL)
                child.wait(timeout=5)
    report = dict(result='verified_packaged_service_with_repository_hidden',
                  isolation_probe=checked.stdout.strip(), readiness=True, graceful_stop_exit=code,
                  existing_database_refused=True, duplicate_runner_refused=True,
                  duration_seconds=time.monotonic() - started, manifest_sha256=args.manifest_sha256,
                  limits='Same kernel and read-only host /usr libraries. Package/state at new paths, no repo/local mounted inside service. Not a fresh Linux machine, Internet, client gameplay, load, restore or soak acceptance.')
    (out / 'verification.json').write_text(json.dumps(report, indent=2) + '\n')
    print(report['result'])


if __name__ == '__main__':
    main()
