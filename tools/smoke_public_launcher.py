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
            diagnostic_launcher = out / 'player-socket-diagnostic.json'
            diagnostic_value = json.loads(launcher.read_text())
            diagnostic_value['peer_diagnostics'] = True
            diagnostic_launcher.write_text(json.dumps(diagnostic_value) + '\n')
            portable_launcher=out/'player-portable-udp.json'
            portable_value=json.loads(launcher.read_text())
            portable_value['udp_backend']='portable'
            portable_launcher.write_text(json.dumps(portable_value)+'\n')
            for index in range(2):
                result = subprocess.run([str(binaries / 'kr-launch'), str(diagnostic_launcher if index == 0 else portable_launcher)],
                                        input=json.dumps(dict(login='SmokePlayer', password=password)),
                                        text=True, capture_output=True, timeout=20)
                (out / f'launcher-{index}.stdout').write_text(result.stdout)
                (out / f'launcher-{index}.stderr').write_text(result.stderr)
                assert result.returncode == 0, ('launcher exit', result.returncode)
                event = json.loads(result.stdout)
                assert event == dict(event='client_started', player_id=expected_player)
                if index == 0:
                    socket_event = next(json.loads(line) for line in result.stderr.splitlines()
                                        if json.loads(line).get('phase') == 'authenticated')
                    assert socket_event['event'] == 'companion_udp_counters'
                    assert socket_event['counters']['send_ok'] > 0
                    assert socket_event['counters']['recv_batches'] > 0
                    assert socket_event['counters']['io_errors'] == 0
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
            assert json.loads(failed.stderr.splitlines()[0]) == dict(
                event='launcher_failed', stage='native_running')
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
            # A transport ending while the native stub is still alive must
            # report that ordering and retire both children/exact session.
            transport_pid = out / 'transport-native.pid'
            transport_stub = out / 'transport-native.py'
            transport_stub.write_text('from pathlib import Path\nimport os,time\n'
                                      f'Path({str(transport_pid)!r}).write_text(str(os.getpid()))\n'
                                      'time.sleep(60)\n')
            value['native_arguments'] = [str(transport_stub)]
            transport_config = out / 'player-transport-failure.json'
            transport_config.write_text(json.dumps(value) + '\n')
            with (out / 'launcher-transport.stdout').open('x') as stdout, \
                 (out / 'launcher-transport.stderr').open('x') as stderr:
                parent = subprocess.Popen([str(binaries / 'kr-launch'), str(transport_config)],
                                          stdin=subprocess.PIPE, stdout=stdout, stderr=stderr, text=True)
                try:
                    parent.stdin.write(json.dumps(dict(login='SmokePlayer', password=password)))
                    parent.stdin.close()
                    deadline = time.monotonic() + 15
                    while not transport_pid.exists():
                        if parent.poll() is not None or time.monotonic() > deadline:
                            raise RuntimeError('Transport failure control did not start native stub')
                        time.sleep(.05)
                    # Rust starts children from spawn_blocking, so Linux lists
                    # them on that thread's children file, not the main task.
                    owned = {pid for task in Path(f'/proc/{parent.pid}/task').iterdir()
                             for pid in (task / 'children').read_text().split()}
                    companions = [int(pid) for pid in owned
                                  if os.readlink(f'/proc/{pid}/exe') == str(binaries / 'kr-companion')]
                    assert len(companions) == 1
                    os.kill(companions[0], signal.SIGTERM)
                    assert parent.wait(timeout=15) == 1
                    assert not Path('/proc/' + transport_pid.read_text()).exists()
                    assert not Path(f'/proc/{companions[0]}').exists()
                finally:
                    if parent.poll() is None:
                        parent.kill()
                        parent.wait(timeout=5)
            events = [json.loads(line) for line in (out / 'launcher-transport.stdout').read_text().splitlines()]
            assert events == [dict(event='client_started', player_id=expected_player),
                              dict(event='companion_stopped', exit_code=0, native_exit_code=None)]
            assert json.loads((out / 'launcher-transport.stderr').read_text().splitlines()[0]) == dict(
                event='launcher_failed', stage='companion_stopped')
            with sqlite3.connect(database) as db:
                assert db.execute('SELECT count(*) FROM sessions WHERE revoked_at IS NULL').fetchone()[0] == 0
                assert db.execute("SELECT count(*) FROM admin_audit WHERE operation='logout'").fetchone()[0] == 5
            # GUI cancellation uses an owned per-launch file on both platforms.
            # It must use normal launcher cleanup and logout, not kill the CLI.
            # The preceding five controls exhaust this principal's intentional
            # login window. Use a distinct declared control account, never
            # relax the production limiter or retry denied login.
            admin(dict(op='create', login='GuiControl', name='GuiControl', password=password))
            gui_pid = out / 'gui-native.pid'
            gui_stub = out / 'gui-native.py'
            gui_stub.write_text('from pathlib import Path\nimport os,time\n'
                                f'Path({str(gui_pid)!r}).write_text(str(os.getpid()))\n'
                                'time.sleep(60)\n')
            cancel_file = out / 'gui-cancel'
            value['native_arguments'] = [str(gui_stub)]
            value['cancel_file'] = str(cancel_file)
            gui_config = out / 'player-gui-cancel.json'
            gui_config.write_text(json.dumps(value) + '\n')
            with (out / 'launcher-gui.stdout').open('x') as stdout, (out / 'launcher-gui.stderr').open('x') as stderr:
                parent = subprocess.Popen([str(binaries / 'kr-launch'), str(gui_config)],
                                          stdin=subprocess.PIPE, stdout=stdout, stderr=stderr, text=True)
                try:
                    parent.stdin.write(json.dumps(dict(login='GuiControl', password=password)))
                    parent.stdin.close()
                    deadline = time.monotonic() + 15
                    while not gui_pid.exists():
                        if parent.poll() is not None or time.monotonic() > deadline:
                            raise RuntimeError('GUI cancellation control did not start native stub')
                        time.sleep(.05)
                    cancel_file.touch(exist_ok=False)
                    assert parent.wait(timeout=15) == 1
                    assert not Path('/proc/' + gui_pid.read_text()).exists()
                finally:
                    if parent.poll() is None:
                        parent.kill()
                        parent.wait(timeout=5)
            with sqlite3.connect(database) as db:
                assert db.execute('SELECT count(*) FROM sessions WHERE revoked_at IS NULL').fetchone()[0] == 0
                assert db.execute("SELECT count(*) FROM admin_audit WHERE operation='logout'").fetchone()[0] == 6
            # A crashed GUI no longer updates its private per-launch heartbeat.
            # The CLI must still own cleanup/logout, with no GUI process left
            # to write the cancellation file.
            admin(dict(op='create', login='GuiHeartbeat', name='GuiHeartbeat', password=password))
            heartbeat_pid = out / 'heartbeat-native.pid'
            heartbeat_stub = out / 'heartbeat-native.py'
            heartbeat_stub.write_text('from pathlib import Path\nimport os,time\n'
                                      f'Path({str(heartbeat_pid)!r}).write_text(str(os.getpid()))\n'
                                      'time.sleep(60)\n')
            heartbeat = out / 'gui-heartbeat'
            heartbeat.write_text('0')
            value['native_arguments'] = [str(heartbeat_stub)]
            value['cancel_file'] = str(out / 'unused-gui-cancel')
            value['owner_heartbeat'] = str(heartbeat)
            heartbeat_config = out / 'player-gui-heartbeat.json'
            heartbeat_config.write_text(json.dumps(value) + '\n')
            with (out / 'launcher-heartbeat.stdout').open('x') as stdout, (out / 'launcher-heartbeat.stderr').open('x') as stderr:
                parent = subprocess.Popen([str(binaries / 'kr-launch'), str(heartbeat_config)],
                                          stdin=subprocess.PIPE, stdout=stdout, stderr=stderr, text=True)
                try:
                    parent.stdin.write(json.dumps(dict(login='GuiHeartbeat', password=password)))
                    parent.stdin.close()
                    deadline = time.monotonic() + 15
                    while not heartbeat_pid.exists():
                        if parent.poll() is not None or time.monotonic() > deadline:
                            raise RuntimeError('Heartbeat control did not start native stub')
                        time.sleep(.05)
                    assert parent.wait(timeout=20) == 1
                    assert not Path('/proc/' + heartbeat_pid.read_text()).exists()
                finally:
                    if parent.poll() is None:
                        parent.kill()
                        parent.wait(timeout=5)
            with sqlite3.connect(database) as db:
                assert db.execute('SELECT count(*) FROM sessions WHERE revoked_at IS NULL').fetchone()[0] == 0
                assert db.execute("SELECT count(*) FROM admin_audit WHERE operation='logout'").fetchone()[0] == 7
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
                  udp_socket_control=socket_event,
                  native_failure=dict(exit=1, reported_native_exit=1, active_sessions_after_exit=0,
                                      audited_logouts=3),
                  address_roles=dict(reported_login_ipv4='192.0.2.173', native_udp_source='127.0.0.1'),
                  companion_stop=dict(exit=0, native_running_at_stop=True, native_gone_after_launcher=True,
                                      active_sessions_after_exit=0, audited_logouts=5),
                  launcher_sigterm=dict(exit=1, owned_native_gone=True, owned_companion_gone=True,
                                        active_sessions_after_exit=0, audited_logouts=4),
                  gui_cancel_file=dict(exit=1, native_gone=True, active_sessions_after_exit=0, audited_logouts=6),
                  gui_heartbeat_stopped=dict(exit=1, native_gone=True, active_sessions_after_exit=0, audited_logouts=7),
                  service_graceful_exit=0, binaries=[dict(name=name, sha256=hashlib.sha256((binaries / name).read_bytes()).hexdigest())
                  for name in ['kr-service', 'kr-admin', 'kr-launch', 'kr-companion']],
                  limits='Linux isolated namespace, local loopback TLS and real QUIC companion. Native programs are /bin/true, /bin/false and Python lifecycle stubs. No original KR login/gameplay, Windows, Internet, load, restore or soak acceptance.')
    (out / 'verification.json').write_text(json.dumps(report, indent=2) + '\n')
    print(report['result'])


if __name__ == '__main__':
    main()
