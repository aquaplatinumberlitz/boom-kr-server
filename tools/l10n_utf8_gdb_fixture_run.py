"""Run one owned i686 fixture under Wine-GDB in isolated namespaces."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import socket
import subprocess
import sys
import pefile


def sha(path): return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--fixture', type=Path, required=True)
    p.add_argument('--hook', type=Path, required=True)
    p.add_argument('--out', type=Path, required=True)
    p.add_argument('--fixture-sha256', required=True)
    p.add_argument('--hook-sha256', required=True)
    p.add_argument('--mode', choices=('pass', 'replace'), required=True)
    p.add_argument('--inside', action='store_true', help=argparse.SUPPRESS)
    a = p.parse_args()
    fixture, hook, out = a.fixture.resolve(strict=True), a.hook.resolve(strict=True), a.out.resolve()
    root = (Path(__file__).resolve().parents[1] / 'local/research/l10n-utf8-codex').resolve()
    if any(not q.is_relative_to(root) for q in (fixture, hook, out)) or len({fixture, hook, out}) != 3:
        p.error('all paths must be distinct under private research directory')
    if sha(fixture) != a.fixture_sha256 or sha(hook) != a.hook_sha256:
        p.error('input hash mismatch')
    pe = pefile.PE(str(fixture))
    exports = [item for item in pe.DIRECTORY_ENTRY_EXPORT.symbols if item.name == b'l10n_test_convert']
    if len(exports) != 1 or pe.OPTIONAL_HEADER.ImageBase + exports[0].address != 0x401560 or pe.OPTIONAL_HEADER.DllCharacteristics & 0x40:
        p.error('fixture export or fixed image base mismatch')
    if not a.inside:
        if out.exists(): p.error('output must be new')
        out.mkdir(mode=0o700)
        cmd = ['unshare', '--user', '--map-root-user', '--net', '--mount', '--pid', '--fork',
               '--mount-proc', '--kill-child=KILL', sys.executable, str(Path(__file__).resolve()),
               '--inside', '--fixture', str(fixture), '--hook', str(hook), '--out', str(out),
               '--fixture-sha256', a.fixture_sha256, '--hook-sha256', a.hook_sha256, '--mode', a.mode]
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=180)
        (out / 'outer.log').write_text(result.stdout + result.stderr)
        run = json.loads((out/'run.json').read_text()) if (out/'run.json').exists() else {}
        ns = run.get('network_namespace'); alive = []
        for item in Path('/proc').iterdir():
            if not item.name.isdigit(): continue
            try:
                if ns and os.readlink(item/'ns/net') == ns: alive.append(int(item.name))
            except (FileNotFoundError, PermissionError, ProcessLookupError): pass
        files = [q for q in sorted(out.iterdir()) if q.is_file()]
        seal = {'returncode': result.returncode, 'namespace': ns, 'remaining_pids': alive,
                'fixture_sha256_after': sha(fixture), 'hook_sha256_after': sha(hook),
                'raw': [{'name': q.name, 'bytes': q.stat().st_size, 'sha256': sha(q)} for q in files]}
        (out/'raw-seal.json').write_text(json.dumps(seal, indent=2)+'\n')
        return 0 if result.returncode == 0 and not alive else 1
    uidmap = Path('/proc/self/uid_map').read_text().split()
    if os.getpid() != 1 or os.getuid() != 0 or len(uidmap) != 3 or uidmap[2] != '1':
        raise RuntimeError('private user and PID namespaces required')
    subprocess.run(['mount','--make-rprivate','/'], check=True)
    subprocess.run(['mount','-t','tmpfs','-o','size=128m','tmpfs','/tmp'], check=True)
    subprocess.run(['mount','-t','tmpfs','-o','size=16m','tmpfs','/run'], check=True)
    subprocess.run(['ip','link','set','lo','up'], check=True)
    interfaces = socket.if_nameindex(); routes = Path('/proc/net/route').read_text()
    ipv6 = subprocess.check_output(['ip','-6','route','show','default'], text=True)
    if [name for _, name in interfaces] != ['lo'] or routes.splitlines()[1:] or ipv6.strip():
        raise RuntimeError('network isolation failed')
    env = os.environ.copy()
    for key in ('DISPLAY','WAYLAND_DISPLAY','XAUTHORITY','DBUS_SESSION_BUS_ADDRESS','WINEDLLOVERRIDES','WINELOADER','WINESERVER'):
        env.pop(key, None)
    env.update(WINEPREFIX=str(out/'prefix'), WINEARCH='win64', WINEDEBUG='-all', TMPDIR='/tmp', LANG='C.UTF-8')
    # Fixture export VA was read from its own PE export table; the image has fixed base 0x400000.
    commands = ("set pagination off\nset confirm off\npython\nfrom pathlib import Path\n"
                "ctx={'__name__':'fixture_probe','L10N_GLYPH_MODE':%r,'L10N_GLYPH_ENTRY':0x401560}\n"
                "exec(compile(Path(%r).read_text(),'glyph-probe','exec'),ctx)\nend\ncontinue\nquit\n" % (a.mode, str(hook)))
    (out/'commands.gdb').write_text(commands)
    with (out/'commands.gdb').open() as stream:
        result = subprocess.run(['winedbg','--gdb',str(fixture)], stdin=stream, env=env,
                                capture_output=True, text=True, timeout=120)
    subprocess.run(['wineserver','-k'], env=env, capture_output=True, timeout=10, check=False)
    (out/'debugger.log').write_text(result.stdout + result.stderr)
    run = {'mode': a.mode, 'returncode': result.returncode, 'fixture_sha256': sha(fixture),
           'hook_sha256': sha(hook), 'network_namespace': os.readlink('/proc/self/ns/net'),
           'interfaces': interfaces, 'ipv4_routes': routes, 'ipv6_default_routes': ipv6,
           'wine_version': subprocess.check_output(['wine','--version'],env=env,text=True).strip()}
    (out/'run.json').write_text(json.dumps(run,indent=2)+'\n')
    return result.returncode


if __name__ == '__main__': sys.exit(main())
