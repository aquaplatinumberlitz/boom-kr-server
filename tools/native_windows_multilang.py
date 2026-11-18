"""Prepare/run one bounded language control from an existing verified Windows VM.

All actual deployment/VM/client inputs and raw output stay under local/. No
assets are published. The previous owner must have stopped and cleaned its
rules. This does not certify clean installation or native gameplay.
"""
import argparse
import hashlib
import json
from pathlib import Path
import shlex
import shutil
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[1]


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['previous', 'out', 'deployment', 'manifest', 'native-control', 'windows-credentials']:
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--manifest-sha256', required=True)
    parser.add_argument('--native-control-sha256', required=True)
    parser.add_argument('--language', choices=['vi', 'en', 'ko'], required=True)
    parser.add_argument('--observe-seconds', type=int, default=90)
    parser.add_argument('--run', action='store_true')
    parser.add_argument('--socket-probe', action='store_true', help='Declared owned debugger control; not native acceptance')
    args = parser.parse_args()
    local = (ROOT / 'local').resolve(strict=True)
    paths = {name: getattr(args, name.replace('-', '_')).resolve()
             for name in ['previous', 'out', 'deployment', 'manifest', 'native-control', 'windows-credentials']}
    if any(path == local or not path.is_relative_to(local) for path in paths.values()):
        parser.error('Every input/output must be under local/')
    if not 30 <= args.observe_seconds <= 240:
        parser.error('Observation budget is 30..240 seconds')
    previous, output, stage = paths['previous'], paths['out'], paths['deployment']
    if output.exists() or stage.exists() or output == stage:
        parser.error('Output and deployment must be distinct and new')
    if digest(paths['manifest']) != args.manifest_sha256 or digest(paths['native-control']) != args.native_control_sha256:
        parser.error('Input identity mismatch')
    old = json.loads((previous / 'owner-status.json').read_text())
    if not old.get('ended_epoch') or not old.get('owned_container_stopped') or not old['remote_cleanup']['owned_cloud_rules_absent']:
        parser.error('Previous VM/rules not confirmed stopped')
    manifest = json.loads(paths['manifest'].read_text())
    if manifest['languages'][args.language]['encoding'] != '949x':
        parser.error('Language is not a patched949x input')
    files = [dict(path=row['path'], bytes=row['bytes'], sha256=row['sha256']) for row in manifest['packs']]
    files += [dict(path=row['path'], bytes=row['payload']['bytes'], sha256=row['payload']['sha256'])
              for row in manifest['common'] + manifest['languages'][args.language]['files'] if row['path'] != 'CA.exe']
    output.mkdir(mode=0o700)
    stage.mkdir(mode=0o700)
    config = json.loads((previous / 'private-owner-config.json').read_text())
    connection = json.loads(Path(config['ssh_connection']).read_text())
    plan = json.loads(Path(config['vps_plan']).read_text())
    ssh = ['ssh', '-i', config['ssh_identity'], '-p', str(connection['port']), '-o', 'StrictHostKeyChecking=yes',
           '-o', 'UserKnownHostsFile=' + config['ssh_known_hosts'], connection['user'] + '@' + connection['host']]
    old_remote = Path(plan['root'])
    # Unique remote staging only; runner creates fresh DB and only its own rules.
    suffix = str(time.time_ns())
    new_remote = old_remote.with_name('boom-m8-multilang-' + args.language + '-' + suffix)
    recipe = '''from pathlib import Path
import hashlib,ipaddress,json,os,shutil,subprocess,sys
old=Path(sys.argv[1]);new=Path(sys.argv[2]);assert not new.exists()
new.mkdir(mode=0o700);shutil.copytree(old/'package',new/'package');(new/'state').mkdir(mode=0o700)
shutil.copy2(old/'state/launcher.pk8',new/'state/launcher.pk8');(new/'state/launcher.pk8').chmod(0o600)
for name in ['guarded-runner.py','cloud-list.json']:shutil.copy2(old/name,new/name)
p=json.loads((old/'private-provision.json').read_text());p['root']=str(new)
observed=str(ipaddress.IPv4Address(os.environ['SSH_CONNECTION'].split()[0]))
previous_source=p['source'];source_changed=observed!=previous_source
config_path=new/'package/config.template.json';config=json.loads(config_path.read_text())
assert previous_source in config['allowed_peers'], 'Previous source is absent from package allowlist'
config['allowed_peers']=[observed if peer==previous_source else peer for peer in config['allowed_peers']]
for actor in config.get('diagnostic_two_lobby',{}).get('actors',[]):
 if actor['peer']==previous_source:actor['peer']=observed
assert observed in config['allowed_peers']
config_path.chmod(0o600);config_path.write_text(json.dumps(config,ensure_ascii=False,indent=2)+chr(10));config_path.chmod(0o400)
manifest_path=new/'package/manifest.json';manifest=json.loads(manifest_path.read_text())
rows=[row for row in manifest['files'] if row['path']=='config.template.json'];assert len(rows)==1
rows[0].update(bytes=config_path.stat().st_size,sha256=hashlib.sha256(config_path.read_bytes()).hexdigest())
manifest_path.chmod(0o600);manifest_path.write_text(json.dumps(manifest,ensure_ascii=False,indent=2)+chr(10));manifest_path.chmod(0o400)
p['manifest_sha256']=hashlib.sha256(manifest_path.read_bytes()).hexdigest();p['source']=observed
subprocess.run([sys.executable,str(new/'package/kr-package.py'),'verify','--manifest-sha256',p['manifest_sha256']],check=True,capture_output=True,timeout=30)
(new/'private-provision.json').write_text(json.dumps(p,indent=2)+chr(10))
print(json.dumps(dict(service_started=False,cloud_changes=False,source_refreshed=source_changed,service_allowlist_refreshed=True,package_verified=True)))
'''
    (stage / 'staging-recipe.py').write_text(recipe)
    result = subprocess.run(ssh + ['python3 - ' + shlex.quote(str(old_remote)) + ' ' + shlex.quote(str(new_remote))],
                            input=recipe, capture_output=True, text=True, timeout=60)
    (stage / 'staging.stdout').write_text(result.stdout)
    (stage / 'staging.stderr').write_text(result.stderr)
    result.check_returncode()
    plan['root'] = str(new_remote)
    # Fetch the exact staged plan, including its freshly observed /32 source.
    # Never widen the ACL or publish the private source to stdout.
    staged = subprocess.run(ssh + ['cat ' + shlex.quote(str(new_remote / 'private-provision.json'))],
                            capture_output=True, text=True, check=True, timeout=30)
    plan = json.loads(staged.stdout)
    (stage / 'private-provision.json').write_text(json.dumps(plan, indent=2) + '\n')
    config.update(out=str(output), vps_plan=str(stage / 'private-provision.json'),
                  container='kr-m8-native-' + args.language + '-' + suffix)
    (output / 'private-owner-config.json').write_text(json.dumps(config, indent=2) + '\n')
    subprocess.run(['qemu-img', 'create', '-f', 'qcow2', '-F', 'qcow2', '-b', config['vm_base'],
                    str(output / 'native-system.qcow2')], check=True, capture_output=True, timeout=30)
    with (output / 'seed.img').open('xb') as stream:
        stream.truncate(128 * 1024 * 1024)
    subprocess.run(['mkfs.vfat', '-n', 'M8SETUP', str(output / 'seed.img')], check=True, capture_output=True, timeout=30)
    shutil.copy2(previous / 'start-vm.sh', output / 'start-vm.sh')
    for name in ['kr-launch.exe', 'kr-companion.exe', 'certificate.der', 'deployment-private.json', 'credentials.json']:
        shutil.copy2(previous / name, output / name)
    shutil.copy2(paths['native-control'], output / 'CA-control.exe')
    shutil.copy2(ROOT / 'tools/native_windows_multilang.ps1', output / 'run.ps1')
    pins = {name: digest(output / name) for name in ['kr-launch.exe', 'kr-companion.exe', 'certificate.der', 'CA-control.exe']}
    (output / 'control-pins.json').write_text(json.dumps(pins) + '\n')
    control = dict(language=args.language, runtime_name='M8Native' + args.language.capitalize() + suffix[-10:],
                   observe_seconds=args.observe_seconds)
    if args.socket_probe:
        subprocess.run(['i686-w64-mingw32-gcc', '-municode', '-std=c11', '-O2', '-Wall', '-Wextra', '-Werror',
                        str(ROOT / 'tools/native_windows_socket_probe.c'), '-o', str(output / 'socket-probe.exe')],
                       check=True, capture_output=True, timeout=60)
        control['socket_probe_sha256'] = digest(output / 'socket-probe.exe')
        subprocess.run(['mcopy', '-i', str(output / 'seed.img'), str(output / 'socket-probe.exe'), '::socket-probe.exe'],
                       check=True, capture_output=True, timeout=30)
    (output / 'multilang-run.json').write_text(json.dumps(control) + '\n')
    (output / 'language-inputs.json').write_text(json.dumps(files, indent=2) + '\n')
    (output / 'input-summary.json').write_text(json.dumps(dict(files=len(files) + 1,
        exe_sha256=args.native_control_sha256, launcher_sha256=pins['kr-launch.exe'],
        manifest_sha256=args.manifest_sha256, language=args.language,
        scope='Existing verified language installation with separate loader-control runtime; no acceptance inferred')) + '\n')
    for name in ['run.ps1', 'multilang-run.json', 'language-inputs.json', 'control-pins.json', 'kr-launch.exe',
                 'kr-companion.exe', 'CA-control.exe', 'certificate.der', 'deployment-private.json', 'credentials.json']:
        subprocess.run(['mcopy', '-i', str(output / 'seed.img'), str(output / name), '::' + name],
                       check=True, capture_output=True, timeout=30)
    if not args.run:
        print('Prepared private inputs; no VM/service/rules started.')
        return
    record = dict(started_epoch=time.time(), language=args.language,
                  source_commit=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip())
    with (output / 'owner.stdout').open('xb') as stdout, (output / 'owner.stderr').open('xb') as stderr:
        owner = subprocess.Popen([sys.executable, str(ROOT / 'tools/native_windows_candidate.py'),
                                  '--config', str(output / 'private-owner-config.json')],
                                 stdout=stdout, stderr=stderr, start_new_session=True)
    (output / 'owner-process.json').write_text(json.dumps(dict(pid=owner.pid,
        start_ticks=Path('/proc/' + str(owner.pid) + '/stat').read_text().split()[21],
        source_sha256=digest(ROOT / 'tools/native_windows_candidate.py'))) + '\n')
    try:
        subprocess.run([sys.executable, str(ROOT / 'tools/native_windows_bootstrap.py'), '--run', str(output),
                        '--out', str(output / 'bootstrap-r1'), '--windows-credentials', str(paths['windows-credentials'])],
                       check=True, timeout=420)
        subprocess.run([sys.executable, str(ROOT / 'tools/native_windows_ready_service.py'), '--run', str(output),
                        '--input-summary', str(output / 'input-summary.json')], check=True, timeout=640)
        ready_deadline = time.monotonic() + 75
        while True:
            status = json.loads((output / 'owner-status.json').read_text())
            if status.get('remote_ready_epoch'):
                break
            if owner.poll() is not None or status.get('error') or time.monotonic() >= ready_deadline:
                raise RuntimeError('Remote service readiness deadline; no native acceptance')
            time.sleep(.5)
        deadline = time.monotonic() + args.observe_seconds + 60
        capture = 0
        while time.monotonic() < deadline:
            if owner.poll() is not None:
                raise RuntimeError('Owner ended during observation; inspect status')
            result = subprocess.run(['mtype', '-i', str(output / 'seed.img'), '::native-windows-install.json'],
                                    capture_output=True, text=True, timeout=5)
            try:
                guest = json.loads(result.stdout.lstrip('\ufeff')) if result.returncode == 0 else {}
            except json.JSONDecodeError:
                guest = {}  # Live FAT read can race a report write; not final evidence.
            phase = guest.get('phase')
            if capture < 12 or phase in ['finished', 'failed']:
                actions = output / ('observe-' + str(capture) + '.json')
                actions.write_text(json.dumps([dict(op='capture')]) + '\n')
                subprocess.run([sys.executable, str(ROOT / 'tools/native_windows_input.py'), '--run', str(output),
                                '--actions', str(actions), '--out', str(output / ('observe-' + str(capture)))],
                               check=True, capture_output=True, timeout=45)
                capture += 1
            if phase in ['finished', 'failed']:
                record.update(guest_phase=phase, live_launcher_exit=guest.get('launcher_exit'))
                break
            time.sleep(2)
        else:
            record['observation_deadline'] = True
    except BaseException as error:
        record['error'] = type(error).__name__
        raise
    finally:
        (output / 'stop-requested').touch()
        try:
            record['owner_exit'] = owner.wait(timeout=120)
        except subprocess.TimeoutExpired:
            record['owner_cleanup_deadline'] = True
        record['ended_epoch'] = time.time()
        (output / 'controller-result.json').write_text(json.dumps(record, indent=2) + '\n')
    print('Bounded language control ended; verify exported native and cleanup evidence.')


if __name__ == '__main__':
    main()
