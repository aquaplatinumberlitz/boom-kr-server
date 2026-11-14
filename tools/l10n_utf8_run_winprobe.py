"""Run only the standalone i686 probe in disposable namespaces; no client."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import socket
import subprocess
import sys


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--probe', required=True, type=Path)
    parser.add_argument('--out', required=True, type=Path)
    parser.add_argument('--sha256', required=True)
    parser.add_argument('--inside', action='store_true', help=argparse.SUPPRESS)
    args = parser.parse_args()
    probe = args.probe.resolve(strict=True)
    out = args.out.resolve()
    root = (Path(__file__).resolve().parents[1] / 'local/research/l10n-utf8-codex').resolve()
    if not probe.is_relative_to(root) or not out.is_relative_to(root) or probe == out:
        parser.error('probe and output must be distinct local research paths')
    if len(args.sha256) != 64 or sha(probe) != args.sha256:
        parser.error('probe SHA-256 mismatch')
    if not args.inside:
        if out.exists(): parser.error('output must not exist')
        out.mkdir(mode=0o700)
        command = ['unshare', '--user', '--map-root-user', '--net', '--mount', '--pid',
                   '--fork', '--mount-proc', '--kill-child=KILL', sys.executable,
                   str(Path(__file__).resolve()), '--inside', '--probe', str(probe), '--out', str(out),
                   '--sha256', args.sha256]
        result = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                                text=True, timeout=180)
        (out / 'outer.log').write_text(result.stdout)
        run = json.loads((out / 'run.json').read_text()) if (out / 'run.json').exists() else {}
        namespace = run.get('network_namespace')
        alive = []
        for proc in Path('/proc').iterdir():
            if not proc.name.isdigit(): continue
            try:
                if namespace and os.readlink(proc / 'ns/net') == namespace: alive.append(int(proc.name))
            except (FileNotFoundError, PermissionError, ProcessLookupError): pass
        files = [out / 'outer.log'] + ([out / 'run.json'] if run else [])
        seal = {'returncode': result.returncode, 'network_namespace': namespace,
                'remaining_pids': alive, 'probe_sha256_after': sha(probe),
                'raw': [{'path': str(p), 'sha256': sha(p), 'bytes': p.stat().st_size} for p in files]}
        (out / 'raw-seal.json').write_text(json.dumps(seal, indent=2) + '\n')
        return 0 if result.returncode == 0 and not alive else 1
    mapping = Path('/proc/self/uid_map').read_text().split()
    if os.getpid() != 1 or os.getuid() != 0 or len(mapping) != 3 or mapping[2] != '1':
        raise RuntimeError('private user and PID namespaces required')
    subprocess.run(['mount', '--make-rprivate', '/'], check=True)
    subprocess.run(['mount', '-t', 'tmpfs', '-o', 'size=128m', 'tmpfs', '/tmp'], check=True)
    subprocess.run(['mount', '-t', 'tmpfs', '-o', 'size=16m', 'tmpfs', '/run'], check=True)
    subprocess.run(['ip', 'link', 'set', 'lo', 'up'], check=True)
    interfaces = socket.if_nameindex()
    routes = Path('/proc/net/route').read_text()
    ipv6_routes = subprocess.check_output(['ip', '-6', 'route', 'show', 'default'], text=True)
    if [name for _, name in interfaces] != ['lo'] or routes.splitlines()[1:] or ipv6_routes.strip():
        raise RuntimeError('network isolation failed')
    env = os.environ.copy()
    for key in ('DISPLAY', 'WAYLAND_DISPLAY', 'XAUTHORITY', 'DBUS_SESSION_BUS_ADDRESS',
                'WINEDLLOVERRIDES', 'WINELOADER', 'WINESERVER'):
        env.pop(key, None)
    env.update(WINEPREFIX=str(out / 'prefix'), WINEARCH='win64', WINEDEBUG='-all',
               TMPDIR='/tmp', LANG='C.UTF-8')
    before = sha(probe)
    result = subprocess.run(['wine', str(probe)], env=env, capture_output=True, text=True, timeout=120)
    subprocess.run(['wineserver', '-k'], env=env, capture_output=True, timeout=10, check=False)
    run = {'probe_sha256_before': before, 'returncode': result.returncode,
           'stdout': result.stdout, 'stderr': result.stderr, 'interfaces': interfaces,
           'ipv4_routes': routes, 'ipv6_default_routes': ipv6_routes,
           'network_namespace': os.readlink('/proc/self/ns/net'),
           'wine_version': subprocess.check_output(['wine', '--version'], env=env, text=True).strip()}
    (out / 'run.json').write_text(json.dumps(run, indent=2) + '\n')
    return result.returncode


if __name__ == '__main__':
    sys.exit(main())
