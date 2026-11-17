"""Isolated TLS→SQLite→launch→real companion→logout smoke.

The child representing the native game is /bin/true: this explicitly does not
establish KR login, game wire, gameplay, Windows or Internet acceptance.
"""
import argparse
import hashlib
import ipaddress
import json
import os
from pathlib import Path
import secrets
import signal
import sqlite3
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--base', type=Path, required=True)
    parser.add_argument('--binary-dir', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--inside', action='store_true', help=argparse.SUPPRESS)
    args = parser.parse_args()
    local = (ROOT / 'local').resolve()
    out = args.out.resolve()
    if out == local or not out.is_relative_to(local):
        parser.error('Raw phải ở local/')
    if not args.inside:
        out.mkdir(mode=0o700, parents=False, exist_ok=False)
        command = ['unshare', '--user', '--map-root-user', '--mount', '--net', '--pid', '--fork',
                   '--mount-proc', '--kill-child=KILL', sys.executable, str(Path(__file__).resolve()),
                   '--inside', '--base', str(args.base.resolve()), '--binary-dir', str(args.binary_dir.resolve()),
                   '--out', str(out)]
        with (out / 'namespace.log').open('x') as stream:
            result = subprocess.run(command, stdout=stream, stderr=subprocess.STDOUT, timeout=100)
        (out / 'exit.json').write_text(json.dumps(dict(returncode=result.returncode)) + '\n')
        raise SystemExit(result.returncode)
    subprocess.run(['mount', '--make-rprivate', '/'], check=True)
    subprocess.run(['mount', '-t', 'tmpfs', 'tmpfs', '/tmp'], check=True)
    Path('/tmp/krlaunch').mkdir(mode=0o700)
    subprocess.run(['ip', 'link', 'set', 'lo', 'up'], check=True)
    binaries = args.binary_dir.resolve()
    config = json.loads(args.base.read_text())
    addresses = {config['bind']} | set(config['allowed_peers'])
    addresses |= {actor['peer'] for actor in config['diagnostic_two_lobby']['actors']}
    for value in sorted(addresses):
        address = ipaddress.IPv4Address(value)
        if not address.is_loopback and not address.is_unspecified:
            subprocess.run(['ip', 'addr', 'add', str(address) + '/32', 'dev', 'lo'], check=True)
    database = out / 'accounts.sqlite'

    def admin(command):
        request = dict(actor='launcher-smoke', reason='Isolated first-party launcher controls', command=command)
        result = subprocess.run([str(binaries / 'kr-admin'), str(database)], input=json.dumps(request),
                                text=True, capture_output=True, check=True, timeout=10)
        value = json.loads(result.stdout)
        assert 'error' not in value, 'Administrative preparation failed'
        return value

    admin(dict(op='init'))
    password = secrets.token_urlsafe(24)
    created = admin(dict(op='create', login='SmokePlayer', name='SmokePlayer', password=password))
    expected_player = created['player']['player_id']
    identity = out / 'identity'
    subprocess.run([str(binaries / 'kr-launch-identity'), str(identity)], check=True, capture_output=True)
    certificate = identity / 'certificate.der'
    public = dict(bind='127.0.0.1:45000', certificate=dict(path=str(certificate),
                  bytes=certificate.stat().st_size, sha256=hashlib.sha256(certificate.read_bytes()).hexdigest()),
                  private_key=str(identity / 'private.pk8'), max_connections=4, timeout_ms=2000,
                  session_ttl_seconds=120, session_idle_seconds=60)
    admission = config['diagnostic_two_lobby']['account_admission']
    admission.update(database=str(database), launch_socket='/tmp/krlaunch/launch.sock', public_launcher=public)
    if '127.0.0.1' not in config['allowed_peers']:
        config['allowed_peers'].append('127.0.0.1')
    config['authenticated_relay'].update(bind='127.0.0.1:46000', advertise='127.0.0.1:46000')
    serving = out / 'service-config.json'
    serving.write_text(json.dumps(config, indent=2) + '\n')
    serving.chmod(0o600)
    launcher = out / 'player-config.json'
    launcher.write_text(json.dumps(dict(server=public['bind'], certificate='identity/certificate.der',
                        native_ip='127.0.0.1', reported_login_ipv4='192.0.2.173',
                        companion=os.path.relpath(binaries / 'kr-companion', out),
                        native_program=os.path.relpath('/bin/true', out), native_arguments=[],
                        native_directory='/tmp', port_budget=100)) + '\n')
    launcher.chmod(0o600)
    with (out / 'service.log').open('x') as log:
        service = subprocess.Popen([str(binaries / 'kr-service'), '--config', str(serving), '--out', str(out / 'service')],
                                   stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
        try:
            state = out / 'service/service-state.json'
            deadline = time.monotonic() + 20
            while not state.is_file() or not json.loads(state.read_text()).get('ready'):
                if service.poll() is not None or time.monotonic() > deadline:
                    raise RuntimeError('Service readiness failed; raw retained')
                time.sleep(.05)
            launches = []
            for index in range(2):
                result = subprocess.run([str(binaries / 'kr-launch'), str(launcher)],
                                        input=json.dumps(dict(login='SmokePlayer', password=password)),
                                        text=True, capture_output=True, timeout=20)
                (out / f'launcher-{index}.stdout').write_text(result.stdout)
                (out / f'launcher-{index}.stderr').write_text(result.stderr)
                assert result.returncode == 0, ('launcher exit', result.returncode)
                event = json.loads(result.stdout)
                assert event == dict(event='client_started', player_id=expected_player)
                with sqlite3.connect(database) as db:
                    active = db.execute('SELECT count(*) FROM sessions WHERE revoked_at IS NULL').fetchone()[0]
                    count = db.execute("SELECT count(*) FROM admin_audit WHERE operation='logout'").fetchone()[0]
                    assert active == 0 and count == index + 1, (active, count)
                launches.append(dict(player_id=expected_player, exit=0, active_sessions_after_exit=active, audited_logouts=count))
            # A real child failure must retain a useful numeric status while
            # releasing its companion and exact backend session.
            failed_config = out / 'player-failure-config.json'
            value = json.loads(launcher.read_text())
            value['native_program'] = os.path.relpath('/bin/false', out)
            failed_config.write_text(json.dumps(value) + '\n')
            failed = subprocess.run([str(binaries / 'kr-launch'), str(failed_config)],
                                    input=json.dumps(dict(login='SmokePlayer', password=password)),
                                    text=True, capture_output=True, timeout=20)
            (out / 'launcher-failure.stdout').write_text(failed.stdout)
            (out / 'launcher-failure.stderr').write_text(failed.stderr)
            assert failed.returncode == 1
            assert [json.loads(line) for line in failed.stdout.splitlines()] == [
                dict(event='client_started', player_id=expected_player),
                dict(event='client_failed', exit_code=1)]
            with sqlite3.connect(database) as db:
                assert db.execute('SELECT count(*) FROM sessions WHERE revoked_at IS NULL').fetchone()[0] == 0
                assert db.execute("SELECT count(*) FROM admin_audit WHERE operation='logout'").fetchone()[0] == 3
            # Exercise launcher termination after both owned children exist.
            # This child is a harness stub, never a substitute for native KR.
            stub = out / 'native-stub.py'
            pid_file = out / 'native-stub.pid'
            stub.write_text('from pathlib import Path\nimport os,time\n'
                            f'Path({str(pid_file)!r}).write_text(str(os.getpid()))\n'
                            'time.sleep(60)\n')
            value = json.loads(launcher.read_text())
            value.update(native_program=sys.executable, native_arguments=[str(stub)])
            cancellation_config = out / 'player-cancellation-config.json'
            cancellation_config.write_text(json.dumps(value) + '\n')
            with (out / 'launcher-cancellation.stdout').open('x') as stdout, \
                 (out / 'launcher-cancellation.stderr').open('x') as stderr:
                cancelled = subprocess.Popen([str(binaries / 'kr-launch'), str(cancellation_config)],
                                             stdin=subprocess.PIPE, stdout=stdout, stderr=stderr, text=True)
                try:
                    cancelled.stdin.write(json.dumps(dict(login='SmokePlayer', password=password)))
                    cancelled.stdin.close()
                    deadline = time.monotonic() + 15
                    while not pid_file.exists():
                        if cancelled.poll() is not None or time.monotonic() > deadline:
                            raise RuntimeError('Cancellation control did not start child')
                        time.sleep(.05)
                    native_pid = int(pid_file.read_text())
                    cancelled.send_signal(signal.SIGTERM)
                    assert cancelled.wait(timeout=15) == 1
                    assert not Path(f'/proc/{native_pid}').exists(), 'Owned native stub still exists'
                    for process in Path('/proc').iterdir():
                        if not process.name.isdigit():
                            continue
                        try:
                            executable = os.readlink(process / 'exe')
                        except OSError:
                            continue
                        assert executable != str(binaries / 'kr-companion'), 'Owned companion still exists'
                finally:
                    if cancelled.poll() is None:
                        cancelled.kill()
                        cancelled.wait(timeout=5)
            with sqlite3.connect(database) as db:
                assert db.execute('SELECT count(*) FROM sessions WHERE revoked_at IS NULL').fetchone()[0] == 0
                assert db.execute("SELECT count(*) FROM admin_audit WHERE operation='logout'").fetchone()[0] == 4
            service.send_signal(signal.SIGTERM)
            assert service.wait(timeout=15) == 0
            assert json.loads(state.read_text())['stopped']
        finally:
            if service.poll() is None:
                os.killpg(service.pid, signal.SIGKILL)
                service.wait(timeout=5)
    logs = ''.join(path.read_text() for path in out.glob('launcher-*'))
    assert password not in logs and 'capability' not in logs and 'token_1:' not in logs
    report = dict(result='tls_sqlite_companion_logout_smoke_passed', runs=launches,
                  native_failure=dict(exit=1, reported_native_exit=1, active_sessions_after_exit=0,
                                      audited_logouts=3),
                  address_roles=dict(reported_login_ipv4='192.0.2.173', native_udp_source='127.0.0.1'),
                  launcher_sigterm=dict(exit=1, owned_native_gone=True, owned_companion_gone=True,
                                        active_sessions_after_exit=0, audited_logouts=4),
                  service_graceful_exit=0, binaries=[dict(name=name, sha256=hashlib.sha256((binaries / name).read_bytes()).hexdigest())
                  for name in ['kr-service', 'kr-admin', 'kr-launch', 'kr-companion']],
                  limits='Linux isolated namespace, local loopback TLS and real QUIC companion. Native program is /bin/true. No original KR login/gameplay, Windows, Internet, load, restore or soak acceptance.')
    (out / 'verification.json').write_text(json.dumps(report, indent=2) + '\n')
    print(report['result'])


if __name__ == '__main__':
    main()
