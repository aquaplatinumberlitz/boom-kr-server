"""Bounded M8 language pair through real launchers in isolated Wine hosts.

All inputs, commands and receipts stay local. Ordinary input is supplied as
numbered input-N.json files after observing captures; process presence and
controller completion do not establish room/chat/gameplay acceptance.
"""
import argparse
import fcntl
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import secrets
import shutil
import signal
import sqlite3
import subprocess
import sys
import time

import native_run_cleanup as cleanup

ROOT = Path(__file__).resolve().parents[1]
LOCAL = (ROOT / 'local').resolve()


def write(path, value):
    cleanup.write_json(path, value)


def checked(row):
    path = Path(row['path']).resolve(strict=True)
    if not path.is_relative_to(LOCAL) or cleanup.checksum(path) != row['sha256']:
        raise ValueError('Private input identity or path mismatch')
    return path


def inside(control, run):
    if os.getpid() != 1 or os.getuid() != 0 or Path('/proc/self/uid_map').read_text().split()[2] != '1':
        raise RuntimeError('Private PID/user namespace required')
    subprocess.run(['mount', '--make-rprivate', '/'], check=True)
    for path in ['/tmp', '/run']:
        subprocess.run(['mount', '-t', 'tmpfs', '-o', 'size=256m', 'tmpfs', path], check=True)
    subprocess.run(['ip', 'link', 'set', 'lo', 'up'], check=True)
    topology_path = checked(control['topology'])
    spec = importlib.util.spec_from_file_location('m8_owned_topology', topology_path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    topology = module.Topology(run, run / 'scratch/prefix-A')
    hosts = run / 'hosts'
    hosts.write_text('127.0.0.1 localhost lslist.crazyarcade.com dream-login-ca.nexon.com happy-login-ca.nexon.com\n')
    subprocess.run(['mount', '--bind', str(hosts), '/etc/hosts'], check=True)
    nss = run / 'nsswitch.conf'
    nss.write_text('\n'.join(line for line in Path('/etc/nsswitch.conf').read_text().splitlines()
                              if not line.lstrip().startswith('hosts:')) + '\nhosts: files\n')
    subprocess.run(['mount', '--bind', str(nss), '/etc/nsswitch.conf'], check=True)
    binaries = {name: checked(row) for name, row in control['binaries'].items()}
    record = dict(schema='kre-m8-native-wine-pair-v1', started_epoch=time.time(), inputs=[], launchers={},
                  limits='Isolated virtual LAN, real TLS launchers and companion, declared native copies. No Internet/clean-install/gameplay acceptance inferred.')
    record['wine_version'] = subprocess.check_output(['wine', '--version'], text=True, timeout=10).strip()
    record['control_sha256'] = cleanup.checksum(run / 'control.json')
    children, streams, environments = [], [], {}
    failure = None
    dbdir = run / 'account-db'; dbdir.mkdir(mode=0o700)
    subprocess.run(['mount', '-t', 'tmpfs', '-o', 'size=32m,mode=0700', 'tmpfs', str(dbdir)], check=True)
    database = dbdir / 'accounts.sqlite'
    record['database_storage'] = 'Private32MiB tmpfs, same diagnostic scope as G1 Wine harness; exported closed after stop. No durability claim.'

    def spawn(argv, label, env=None, stdin=None):
        log = (run / (label + '.log')).open('xb')
        streams.append(log)
        process = subprocess.Popen(argv, env=env, stdin=stdin, stdout=log, stderr=subprocess.STDOUT,
                                   start_new_session=True)
        children.append(process)
        return process

    def stop_signal(signum, frame):
        raise RuntimeError('Owned runtime stop signal ' + str(signum))

    for sig in [signal.SIGTERM, signal.SIGINT, signal.SIGHUP]:
        signal.signal(sig, stop_signal)

    def capture(label):
        for role, env in environments.items():
            subprocess.run(['import', '-window', 'root', str(run / (label + '-' + role + '.png'))],
                           env=env, check=True, capture_output=True, timeout=8)

    def admin(command):
        request = dict(actor='m8-wine-pair', reason='Isolated native language control', command=command)
        result = subprocess.run([str(binaries['kr-admin']), str(database)],
                                input=json.dumps(request), text=True, capture_output=True, timeout=15)
        record.setdefault('admin_operations', []).append(dict(operation=command['op'], exit=result.returncode,
                                                              stderr=result.stderr, stdout=result.stdout))
        write(run / 'admin-operations.json', record['admin_operations'])
        result.check_returncode()
        return json.loads(result.stdout)

    try:
        identity = run / 'identity'
        subprocess.run([str(binaries['kr-launch-identity']), str(identity)], check=True, capture_output=True, timeout=15)
        certificate = identity / 'certificate.der'
        admin(dict(op='init'))
        players = {}
        passwords = {role: secrets.token_hex(16) for role in ['A', 'B']}
        for role, language in [('A', 'vi'), ('B', 'ko')]:
            players[role] = admin(dict(op='create', login='M8Wine' + role,
                                      name=control['names'][language], password=passwords[role]))['player']
        config = json.loads(checked(control['server_config']).read_text())
        admission = config['diagnostic_two_lobby']['account_admission']
        Path('/tmp/krlaunch').mkdir(mode=0o700)
        public = dict(bind='10.77.0.1:45000', certificate=dict(path=str(certificate), bytes=certificate.stat().st_size,
                      sha256=cleanup.checksum(certificate)), private_key=str(identity / 'private.pk8'),
                      max_connections=8, timeout_ms=2000, session_ttl_seconds=600, session_idle_seconds=300)
        admission.update(database=str(database), launch_socket='/tmp/krlaunch/launch.sock',
                         profile_mode='database_zero', patched_949x_names=True, public_launcher=public)
        config['authenticated_relay'].update(bind='10.77.0.1:41990', advertise='10.77.0.1:41990')
        path = run / 'service-config.json'; write(path, config); path.chmod(0o600)
        service = spawn([str(binaries['kr-service']), '--config', str(path), '--out', str(run / 'service')], 'service')
        deadline = time.monotonic() + 25
        while True:
            state_path = run / 'service/service-state.json'
            if state_path.exists() and json.loads(state_path.read_text()).get('ready'): break
            if service.poll() is not None or time.monotonic() >= deadline: raise RuntimeError('Service readiness failed')
            time.sleep(.1)
        record['players'] = {role: value['player_id'] for role, value in players.items()}
        wrapper = run / 'wine-launch.py'
        wrapper.write_text("import os,sys\nfrom pathlib import Path\n"
            "out=Path(__file__).resolve().parent\nrole=Path.cwd().name\n"
            "assert role in ['game-A','game-B']\n"
            "fd=os.open(out/('native-'+role+'.log'),os.O_WRONLY|os.O_CREAT|os.O_EXCL,0o600)\n"
            "os.dup2(fd,1);os.dup2(fd,2);os.close(fd)\n"
            "os.execvpe('wine',['wine',*sys.argv[1:]],dict(os.environ,WINEDEBUG='-all,+seh'))\n")
        for role in ['A', 'B']:
            env = os.environ.copy()
            for key in ['DISPLAY', 'WAYLAND_DISPLAY', 'XAUTHORITY', 'DBUS_SESSION_BUS_ADDRESS',
                        'DBUS_STARTER_ADDRESS', 'SESSION_MANAGER', 'WINEDLLOVERRIDES', 'WINELOADER', 'WINESERVER']:
                env.pop(key, None)
            env.update(WINEPREFIX=str(run / ('scratch/prefix-' + role)), WINEARCH='win64',
                       WINEDEBUG='-all', XDG_RUNTIME_DIR='/tmp/runtime-' + role,
                       LIBGL_ALWAYS_SOFTWARE='1', MESA_LOADER_DRIVER_OVERRIDE='llvmpipe',
                       GALLIUM_DRIVER='llvmpipe', TMPDIR='/tmp', LANG='C.UTF-8')
            if 'alsa_config' in control: env['ALSA_CONFIG_PATH'] = str(checked(control['alsa_config']))
            Path(env['XDG_RUNTIME_DIR']).mkdir(mode=0o700)
            read_fd, write_fd = os.pipe()
            log = (run / ('xvfb-' + role + '.log')).open('xb'); streams.append(log)
            xvfb = subprocess.Popen(['Xvfb', '-displayfd', str(write_fd), '-screen', '0', '1024x768x24', '-nolisten', 'tcp'],
                                    stdout=log, stderr=subprocess.STDOUT, pass_fds=(write_fd,), start_new_session=True)
            children.append(xvfb); os.close(write_fd)
            import select
            if not select.select([read_fd], [], [], 10)[0]:
                raise TimeoutError('Private display readiness')
            display = os.read(read_fd, 32).decode().strip(); os.close(read_fd)
            if not display.isdigit(): raise ValueError('Invalid private display')
            env['DISPLAY'] = ':' + display
            environments[role] = env
            boot = spawn(topology.command(role, ['wineboot', '-u']), 'wineboot-' + role, env)
            if boot.wait(timeout=90): raise RuntimeError('Owned Wine prefix startup failed')
            subprocess.run(topology.command(role, ['wine', 'reg', 'add', r'HKCU\Software\Wine\Drivers',
                           '/v', 'Audio', '/t', 'REG_SZ', '/d', '', '/f']), env=env, check=True,
                           capture_output=True, timeout=15)
        for role, address in [('A', '10.77.0.2'), ('B', '10.77.0.3')]:
            game = run / ('scratch/game-' + role)
            path = run / ('launcher-' + role + '.json')
            write(path, dict(server=public['bind'], certificate=str(certificate), reported_login_ipv4=address,
                  native_ip='127.0.0.1', companion=str(binaries['kr-companion']), native_program=sys.executable,
                  native_arguments=[str(wrapper), str(game / 'CA.exe'), 'multiserveridx:0'], native_directory=str(game),
                  port_budget=100, cancel_file=str(run / ('cancel-' + role)), udp_backend='portable'))
            process = spawn(topology.command(role, [str(binaries['kr-launch']), str(path)]), 'launcher-' + role,
                            environments[role], subprocess.PIPE)
            process.stdin.write(json.dumps(dict(login='M8Wine' + role, password=passwords.pop(role))).encode())
            process.stdin.close()
            record['launchers'][role] = dict(pid=process.pid)
        write(run / 'runtime-status.json', record)
        index = 0; next_capture = time.monotonic(); deadline = time.monotonic() + control.get('seconds', 180)
        while time.monotonic() < deadline and not (run / 'stop.request').exists():
            if service.poll() is not None: raise RuntimeError('Owned service stopped during observation')
            for role, state in record['launchers'].items():
                process = next(p for p in children if p.pid == state['pid'])
                if process.poll() is not None and 'exit' not in state:
                    state['exit'] = process.returncode
                    write(run / 'runtime-status.json', record)
            if all('exit' in state for state in record['launchers'].values()):
                raise RuntimeError('Both launchers ended before observation completed')
            if time.monotonic() >= next_capture:
                capture('observe-' + str(index)); index += 1; next_capture = time.monotonic() + 5
            request_path = run / ('input-' + str(len(record['inputs'])) + '.json')
            if request_path.exists():
                request = json.loads(request_path.read_text())
                role = request['role']; env = environments[role]
                if role not in ['A', 'B']: raise ValueError('Unknown input role')
                if request['op'] == 'capture': pass
                else:
                    ids = subprocess.check_output(['xdotool', 'search', '--onlyvisible', '--name', '^Crazy Arcade$'],
                                                  env=env, text=True, timeout=5).split()
                    if len(ids) != 1: raise ValueError('Native window not uniquely observed')
                    subprocess.run(['xdotool', 'windowfocus', '--sync', ids[0]], env=env, check=True, timeout=5)
                    if request['op'] == 'click':
                        x, y = request['xy']
                        if type(x) is not int or type(y) is not int or not (0 <= x < 800 and 0 <= y < 600):
                            raise ValueError('Native input coordinate bound')
                        argv = ['xdotool', 'mousemove', '--window', ids[0], str(x), str(y), 'click']
                        if request.get('double'): argv.extend(['--repeat', '2', '--delay', '150'])
                        argv.append('1')
                    elif request['op'] == 'text':
                        value = request['text']
                        if not 1 <= len(value) <= 96 or any(not (c.isascii() and (c.isalnum() or c == ' ')) for c in value):
                            raise ValueError('Declared ASCII chat control bound')
                        argv = ['xdotool', 'type', '--clearmodifiers', '--', value]
                    elif request['op'] == 'key' and request['key'] in ['Return', 'Escape', 'Tab']:
                        argv = ['xdotool', 'key', '--clearmodifiers', request['key']]
                    else: raise ValueError('Unknown ordinary input')
                    subprocess.run(argv, env=env, check=True, capture_output=True, timeout=8)
                record['inputs'].append(dict(number=len(record['inputs']), request=request, observed_epoch=time.time()))
                write(run / 'runtime-status.json', record)
                capture('input-' + str(len(record['inputs']) - 1))
            time.sleep(.1)
        capture('final')
    except BaseException as error:
        failure = error; record['error'] = type(error).__name__ + ': ' + str(error)
        write(run / 'runtime-failure.json', dict(error=record['error'], observed_epoch=time.time()))
        try: capture('failure')
        except Exception: pass
    finally:
        for sig in [signal.SIGTERM, signal.SIGINT, signal.SIGHUP]: signal.signal(sig, signal.SIG_IGN)
        for role in environments: (run / ('cancel-' + role)).write_text('bounded_owned_run_end\n')
        end = time.monotonic() + 12
        for role in record['launchers']:
            pid = record['launchers'][role]['pid']; child = next(p for p in children if p.pid == pid)
            try: record['launchers'][role]['exit'] = child.wait(timeout=max(.1, end - time.monotonic()))
            except subprocess.TimeoutExpired: record.setdefault('cleanup_errors', []).append('Launcher deadline ' + role)
        for role, env in environments.items():
            log = run / ('scratch/game-' + role + '/CALog.log')
            if log.is_file(): shutil.copy2(log, run / ('CALog-' + role + '.log'))
            try: subprocess.run(topology.command(role, ['wineserver', '-k']), env=env, capture_output=True, timeout=8)
            except Exception as error: record.setdefault('cleanup_errors', []).append(type(error).__name__)
        for child in reversed(children):
            if child.poll() is None:
                try:
                    os.killpg(child.pid, signal.SIGTERM)
                    try: child.wait(timeout=5)
                    except subprocess.TimeoutExpired: os.killpg(child.pid, signal.SIGKILL); child.wait(timeout=3)
                except ProcessLookupError: child.wait(timeout=3)
        for stream in streams: stream.close()
        if database.exists():
            db = None
            try:
                db = sqlite3.connect('file:' + str(database) + '?mode=ro', uri=True)
                record['database'] = dict(integrity=db.execute('PRAGMA integrity_check').fetchone()[0],
                    unrevoked_sessions=db.execute('SELECT count(*) FROM sessions WHERE revoked_at IS NULL').fetchone()[0],
                    logout_count=db.execute("SELECT count(*) FROM admin_audit WHERE operation='logout'").fetchone()[0])
            except sqlite3.Error as error:
                record['database_inspection_error'] = str(error)
            finally:
                if db is not None: db.close()
        export = run / 'account-db-export'; shutil.copytree(dbdir, export)
        subprocess.run(['umount', str(dbdir)], check=True, capture_output=True, timeout=10)
        shutil.copytree(export, dbdir, dirs_exist_ok=True)
        topology.stop()
        from g1_native_runtime import cleanup_owned_namespace
        record['owned_namespace_cleanup'] = cleanup_owned_namespace()
        record['ended_epoch'] = time.time(); write(run / 'runtime-result.json', record)
    return 1 if failure else 0


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--control', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--inside', action='store_true', help=argparse.SUPPRESS)
    args = parser.parse_args()
    control_path = args.control.resolve(strict=True); run = args.out.resolve()
    if not control_path.is_relative_to(LOCAL) or not run.is_relative_to(LOCAL / 'research/codex') or not run.name.startswith('m8-'):
        parser.error('Inputs/output must stay under the private M8 local tree')
    control = json.loads(control_path.read_text())
    if set(control['native']) != {'vi', 'ko'} or not 30 <= control.get('seconds', 180) <= 600:
        parser.error('Two declared languages and bounded runtime required')
    if args.inside: raise SystemExit(inside(control, run))
    cleanup.require_storage()
    with (LOCAL / 'research/codex/.kr-wine-runtime.lock').open('a+') as lease:
        fcntl.flock(lease, fcntl.LOCK_EX | fcntl.LOCK_NB)
        run.mkdir(mode=0o700, exist_ok=False)
        control_path = run / 'control.json'; write(control_path, control)
        shutil.copy2(__file__, run / 'harness-source.py')
        scratch = run / 'scratch'; scratch.mkdir(mode=0o700)
        before = cleanup.storage(); result = None; namespace_started = False
        try:
            install = Path(control['installation']).resolve(strict=True)
            prefix = Path(control['prefix_template']).resolve(strict=True)
            if not install.is_relative_to(LOCAL) or not prefix.is_relative_to(LOCAL): raise ValueError('Private templates required')
            def fixtures(value):
                if isinstance(value, dict):
                    if {'path', 'bytes', 'sha256'} <= value.keys(): checked(value)
                    for child in value.values(): fixtures(child)
                elif isinstance(value, list):
                    for child in value: fixtures(child)
            fixtures(json.loads(checked(control['server_config']).read_text()))
            subprocess.run([str(checked(control['binaries']['kr-player-install'])), 'verify',
                            str(Path(control['bundle']).resolve(strict=True)), control['manifest_sha256'], str(install)],
                           check=True, capture_output=True, timeout=90)
            for role, language in [('A', 'vi'), ('B', 'ko')]:
                for source, target in [(install / language, scratch / ('game-' + role)),
                                       (prefix, scratch / ('prefix-' + role))]:
                    subprocess.run(['cp', '-a', '--reflink=always', str(source), str(target)], check=True, timeout=90)
                shutil.copy2(checked(control['native'][language]), scratch / ('game-' + role + '/CA.exe'))
            command = ['unshare', '--user', '--map-root-user', '--mount', '--net', '--pid', '--fork', '--mount-proc',
                       '--kill-child=KILL', sys.executable, str(Path(__file__).resolve()), '--inside',
                       '--control', str(control_path), '--out', str(run)]
            with (run / 'namespace.log').open('xb') as log:
                namespace_started = True
                result = subprocess.run(command, stdout=log, stderr=subprocess.STDOUT, timeout=control.get('seconds', 180) + 300)
        finally:
            # The owned PID namespace has ended, including all descendant Wine
            # processes. Fresh scratch was never used as an input to another
            # run or checkpoint. Preserve inventory before deleting any byte.
            protected, directories = cleanup.reference_guards(ROOT)
            rows = []; blocked = []
            if namespace_started and result is None: blocked.append('Namespace stop not confirmed; preserve scratch for recovery')
            for path in sorted(scratch.rglob('*')):
                relative = 'local/' + path.relative_to(LOCAL).as_posix()
                if relative in protected or cleanup.retention.covered(relative, directories): blocked.append(relative)
                if path.is_symlink(): rows.append(dict(relative_path=path.relative_to(scratch).as_posix(), kind='symlink', target=os.readlink(path)))
                elif path.is_file(): rows.append(dict(relative_path=path.relative_to(scratch).as_posix(), kind='file', bytes=path.stat().st_size, sha256=cleanup.checksum(path)))
            manifest = run / 'scratch-manifest.json'; write(manifest, dict(files=rows, disposable=True))
            receipt = dict(authorization='docs/evidence/owner-directives/run-cleanup.json', before=before,
                           namespace_exit=result.returncode if result else None, blocked=blocked,
                           manifest_sha256=cleanup.checksum(manifest), deleted_apparent_bytes=0)
            if not blocked:
                shutil.rmtree(scratch)
                receipt['deleted_apparent_bytes'] = sum(r.get('bytes', 0) for r in rows)
                receipt['scratch_removed'] = True
            receipt['after'] = cleanup.storage(); write(run / 'scratch-cleanup.json', receipt)
        raise SystemExit(result.returncode if result else 1)


if __name__ == '__main__': main()
