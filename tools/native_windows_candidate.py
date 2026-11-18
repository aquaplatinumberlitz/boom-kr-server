"""Own one isolated Windows candidate and its pre-staged bounded VPS runner.

Deployment configuration and all output remain local. This controller does not
infer native success from VM, service or process readiness.
"""
import argparse
import fcntl
import json
import os
from pathlib import Path
import re
import shlex
import signal
import socket
import subprocess
import time

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--config', type=Path, required=True)
    args = parser.parse_args()
    local = (ROOT / 'local').resolve()
    config_path = args.config.resolve(strict=True)
    if not config_path.is_relative_to(local):
        parser.error('Private config phải ở local/')
    config = json.loads(config_path.read_text())
    out = Path(config['out']).resolve(strict=True)
    if not out.is_relative_to(local) or (out / 'owner-status.json').exists():
        parser.error('Output phải mới, riêng và ở local/')
    for key in ['vps_plan', 'ssh_connection', 'ssh_identity', 'ssh_known_hosts', 'vm_base', 'player_media']:
        if not Path(config[key]).resolve(strict=True).is_relative_to(local):
            parser.error('Input phải ở local/')
    lease = (local / 'research/codex/.kr-wine-runtime.lock').open('a+')
    fcntl.flock(lease, fcntl.LOCK_EX | fcntl.LOCK_NB)
    connection = json.loads(Path(config['ssh_connection']).read_text())
    plan = json.loads(Path(config['vps_plan']).read_text())
    remote_root = Path(plan['root'])
    ssh = ['ssh', '-i', config['ssh_identity'], '-p', str(connection['port']),
           '-o', 'StrictHostKeyChecking=yes', '-o', 'UserKnownHostsFile=' + config['ssh_known_hosts'],
           connection['user'] + '@' + connection['host']]
    name = config['container']
    if not name.startswith('kr-m8-native-') or any(char not in 'abcdefghijklmnopqrstuvwxyz0123456789-' for char in name):
        parser.error('Invalid owned container name')
    startup_cases = config.get('startup_cases', [])
    if (not isinstance(startup_cases, list) or len(startup_cases) > 8
            or any(not isinstance(case, str) or not re.fullmatch('[a-z0-9-]{1,32}', case)
                   for case in startup_cases) or len(set(startup_cases)) != len(startup_cases)):
        parser.error('Invalid declared startup cases')
    record = dict(source_commit=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
                  started_epoch=time.time(), scope='Native Windows candidate; no success inferred', cleanup_errors=[])
    remote = None
    container = False
    stopped = False

    def stop(signum, _frame):
        nonlocal stopped
        stopped = True
        record.setdefault("stop_signals", []).append(dict(signal=signum, epoch=time.time()))

    signal.signal(signal.SIGTERM, stop)
    signal.signal(signal.SIGINT, stop)

    def save():
        (out / 'owner-status.json').write_text(json.dumps(record, indent=2) + '\n')

    def container_running():
        # Retry only a read-only status query, never Docker create/start or
        # guest input. A transient daemon timeout does not prove VM exit.
        for attempt in range(2):
            try:
                state = subprocess.run(['docker', 'inspect', '--format', '{{.State.Running}}', name],
                                       capture_output=True, text=True, timeout=5)
                return state.returncode == 0 and state.stdout.strip() == 'true'
            except subprocess.TimeoutExpired:
                record.setdefault('container_status_timeouts', []).append(dict(epoch=time.time(), attempt=attempt + 1))
                save()
        raise TimeoutError('Owned container status unavailable after two bounded observations')

    def qmp(command):
        previous = Path.cwd()
        try:
            os.chdir(out)
            with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as client:
                client.settimeout(10)
                client.connect('qmp.sock')
                stream = client.makefile('rwb', buffering=0)
                json.loads(stream.readline())
                for cmd in ['qmp_capabilities', command]:
                    stream.write((json.dumps(dict(execute=cmd)) + '\n').encode())
                    while True:
                        response = json.loads(stream.readline())
                        if 'return' in response or 'error' in response:
                            break
                return response
        finally:
            os.chdir(previous)

    try:
        existing = subprocess.run(['docker', 'inspect', name], capture_output=True, timeout=10)
        if existing.returncode == 0:
            raise RuntimeError('Owned candidate name already exists; use a fresh run')
        # Register ownership before a potentially slow Docker create/start so
        # timeout cleanup also covers a container left in Created state.
        container = True
        command = ['docker', 'create', '--pull', 'never', '--name', name,
                   '--label', 'boom.task=m8-native-windows', '--cap-add', 'NET_ADMIN', '--device', '/dev/kvm',
                   '--mount', f'type=bind,src={out},dst=/task',
                   '--mount', f'type=bind,src={config["vm_base"]},dst={config["vm_base"]},readonly',
                   '--mount', f'type=bind,src={out / "seed.img"},dst=/setup.img',
                   '--mount', f'type=bind,src={config["player_media"]},dst=/player.iso,readonly',
                   '--entrypoint', '/bin/sh', 'ghcr.io/dockur/windows:5.14', '/task/start-vm.sh']
        for backing in config.get('additional_readonly_backing', []):
            path = Path(backing).resolve(strict=True)
            if not path.is_relative_to(local) or not path.is_file():
                raise ValueError('Additional backing input boundary')
            command[-4:-4] = ['--mount', f'type=bind,src={path},dst={path},readonly']
        result = subprocess.run(command, capture_output=True, text=True, timeout=90)
        (out / 'docker-start.stdout').write_text(result.stdout)
        (out / 'docker-start.stderr').write_text(result.stderr)
        if result.returncode:
            raise RuntimeError('Owned VM container failed to start')
        result = subprocess.run(['docker', 'start', name], capture_output=True, text=True, timeout=90)
        (out / 'docker-run.stdout').write_text(result.stdout)
        (out / 'docker-run.stderr').write_text(result.stderr)
        if result.returncode:
            raise RuntimeError('Owned VM start failed')
        deadline = time.monotonic() + 35
        while time.monotonic() < deadline:
            state = subprocess.run(['docker', 'inspect', '--format', '{{.State.Running}}', name],
                                   capture_output=True, text=True, timeout=5)
            if state.stdout.strip() != 'true':
                raise RuntimeError('VM exited during preflight')
            try:
                if qmp('query-status').get('return', {}).get('status') == 'running':
                    break
            except OSError:
                pass
            time.sleep(.2)
        else:
            raise TimeoutError('VM QMP preflight deadline')
        if config.get('wait_install_ready', False):
            record.update(vm_preflight_epoch=time.time(), waiting_for_install_ready=True)
            save()
            print('VM preflight passed; service stays stopped until install-ready is confirmed.', flush=True)
            install_wait = config.get('install_wait_seconds', 600)
            if type(install_wait) is not int or not 60 <= install_wait <= 1800:
                raise ValueError('Installation wait bound')
            deadline = time.monotonic() + install_wait
            while not (out / 'start-service-requested').exists():
                if stopped or (out / 'stop-requested').exists():
                    record['stopped_before_service'] = True
                    save()
                    return
                if time.monotonic() >= deadline:
                    raise TimeoutError('Installation readiness deadline')
                if not container_running():
                    raise RuntimeError('VM stopped before installation readiness')
                time.sleep(.5)
            record['waiting_for_install_ready'] = False
            save()
        with (out / 'remote.stdout').open('xb') as stdout, (out / 'remote.stderr').open('xb') as stderr:
            remote = subprocess.Popen(ssh + ['python3 ' + shlex.quote(str(remote_root / 'guarded-runner.py'))],
                                      stdout=stdout, stderr=stderr)
        deadline = time.monotonic() + 60
        while time.monotonic() < deadline:
            result = subprocess.run(ssh + ['cat ' + shlex.quote(str(remote_root / 'status.json'))],
                                    capture_output=True, timeout=15)
            if result.returncode == 0:
                status = json.loads(result.stdout)
                if status.get('error') or status.get('ended_epoch'):
                    raise RuntimeError('Remote candidate failed before readiness')
                if status.get('ready'):
                    record.update(remote_ready_epoch=time.time(), player_id=status['player']['player_id'])
                    save()
                    break
            if remote.poll() is not None:
                raise RuntimeError('Remote candidate exited before readiness')
            time.sleep(.4)
        else:
            raise TimeoutError('Remote candidate readiness')
        record.update(container=name, vm_started_epoch=time.time(), lease_held=True)
        save()
        print('Candidate VM and bounded service started; native launch awaits explicit QMP input.', flush=True)
        deadline = time.monotonic() + 850
        while not stopped and not (out / 'stop-requested').exists() and time.monotonic() < deadline:
            if remote.poll() is not None:
                raise RuntimeError('Remote candidate ended')
            if not container_running():
                raise RuntimeError('Owned VM container ended')
            time.sleep(.5)
    except Exception as error:
        record['error'] = str(error)
        save()
        print('Candidate failed; private observations retained.', flush=True)
    finally:
        if container:
            try:
                qmp('system_powerdown')
                deadline = time.monotonic() + 35
                while time.monotonic() < deadline:
                    if qmp('query-status').get('return', {}).get('status') == 'shutdown':
                        record['guest_graceful_shutdown'] = True
                        break
                    time.sleep(.5)
                else:
                    record['guest_shutdown_deadline'] = True
                qmp('quit')
            except Exception as error:
                record['cleanup_errors'].append('VM shutdown: ' + type(error).__name__)
            try:
                result = subprocess.run(['docker', 'stop', '--timeout', '5', name], capture_output=True, timeout=45)
                record['docker_stop_exit'] = result.returncode
            except subprocess.TimeoutExpired:
                record['docker_stop_response_timeout'] = True
            try:
                inspected = subprocess.run(['docker', 'inspect', '--format', '{{.State.Running}}', name],
                                           capture_output=True, text=True, timeout=10)
                record['owned_container_stopped'] = inspected.returncode == 0 and inspected.stdout.strip() == 'false'
                if not record['owned_container_stopped']:
                    record['cleanup_errors'].append('Owned container stop not confirmed')
            except Exception as error:
                record['cleanup_errors'].append('Container stop verification: ' + type(error).__name__)
            for filename in ['native-windows-install.json', 'windows-firstboot-error.txt',
                             'native-startup-diagnostic.json', 'native-socket-probe.jsonl']:
                try:
                    result = subprocess.run(['mcopy', '-i', str(out / 'seed.img'), '::' + filename, str(out / filename)],
                                            capture_output=True, timeout=30)
                    if result.returncode == 0:
                        record.setdefault('guest_reports', []).append(filename)
                except Exception as error:
                    record['cleanup_errors'].append('Guest export: ' + type(error).__name__)
            if (out / 'native-startup-diagnostic.json').exists():
                try:
                    result = subprocess.run(['mcopy', '-s', '-i', str(out / 'seed.img'),
                                             '::dumps', str(out)], capture_output=True, timeout=60)
                    record['startup_dump_export_exit'] = result.returncode
                except Exception as error:
                    record['cleanup_errors'].append('Startup dump export: ' + type(error).__name__)
            for case in startup_cases:
                try:
                    result = subprocess.run(['mcopy', '-s', '-i', str(out / 'seed.img'),
                                             '::' + case, str(out)], capture_output=True, timeout=60)
                    record.setdefault('startup_case_exports', {})[case] = result.returncode
                except Exception as error:
                    record['cleanup_errors'].append('Startup case export: ' + type(error).__name__)
        if remote is not None:
            try:
                subprocess.run(ssh + ['touch ' + shlex.quote(str(remote_root / 'stop-requested'))],
                               capture_output=True, timeout=20)
            except Exception as error:
                record['cleanup_errors'].append('Remote stop request: ' + type(error).__name__)
            try:
                record['remote_process_exit'] = remote.wait(timeout=60)
            except subprocess.TimeoutExpired:
                record['cleanup_errors'].append('Remote bounded runner still active; inspect its status')
            for filename in ['status.json', 'service.log', 'capture.log', 'capture.pcap']:
                try:
                    result = subprocess.run(ssh + ['cat ' + shlex.quote(str(remote_root / filename))],
                                            capture_output=True, timeout=40)
                    if result.returncode == 0:
                        (out / ('remote-' + filename)).write_bytes(result.stdout)
                except Exception as error:
                    record['cleanup_errors'].append('Remote export: ' + type(error).__name__)
            path = out / 'remote-status.json'
            if path.exists():
                status = json.loads(path.read_text())
                record['remote_cleanup'] = {key: status.get(key) for key in [
                    'service_exit', 'cloud_append_status', 'cloud_cleanup_status',
                    'owned_cloud_rules_absent', 'cleanup_errors', 'error']}
        record['ended_epoch'] = time.time()
        save()
        print('Candidate owner finished; check private cleanup and native evidence.', flush=True)


if __name__ == '__main__':
    main()
