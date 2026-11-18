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

import native_run_cleanup

ROOT = Path(__file__).resolve().parents[1]


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


class RoomLifecycle:
    """Ordinary UI control gated by pinned, previously observed screen crops.

    A failed/uncertain input is never repeated. Screenshots and wire/lifecycle
    receipts, rather than this controller's state, establish client behavior.
    """
    def __init__(self, reference_path, local, language, chat_message=None):
        from PIL import Image
        if not reference_path.resolve(strict=True).is_relative_to(local):
            raise ValueError('UI references must be local')
        data = json.loads(reference_path.read_text())
        if data['language'] != language:
            raise ValueError('UI reference language mismatch')
        self.references = {}
        for state in ['lobby', 'create', 'room']:
            row = data[state]
            path = Path(row['path']).resolve(strict=True)
            if not path.is_relative_to(local) or digest(path) != row['sha256']:
                raise ValueError('UI reference identity mismatch')
            with Image.open(path) as screen:
                if screen.size != (1024, 768):
                    raise ValueError('UI reference viewport mismatch')
                regions = []
                for box in row['regions']:
                    x, y, right, bottom = box
                    if not (0 <= x < right <= 1024 and 0 <= y < bottom <= 768):
                        raise ValueError('UI crop bounds')
                    regions.append((box, screen.crop(box).convert('RGB')))
                if len(regions) < 2:
                    raise ValueError('UI state needs two independent regions')
                self.references[state] = regions
        # UI exit opens a confirmation modal on the measured vi client.
        # It remains optional for older reference sets, never blindly clicked.
        if 'exit' in data:
            row = data['exit']
            path = Path(row['path']).resolve(strict=True)
            if not path.is_relative_to(local) or digest(path) != row['sha256']:
                raise ValueError('Exit UI reference identity mismatch')
            with Image.open(path) as screen:
                if screen.size != (1024, 768) or len(row['regions']) < 2:
                    raise ValueError('Exit UI reference shape')
                regions = []
                for box in row['regions']:
                    x, y, right, bottom = box
                    if not (0 <= x < right <= 1024 and 0 <= y < bottom <= 768):
                        raise ValueError('Exit UI crop bounds')
                    regions.append((box, screen.crop(box).convert('RGB')))
                self.references['exit'] = regions
        self.step = 0
        self.observations = []
        self.chat_message = chat_message

    def observe(self, capture, run):
        from PIL import Image, ImageChops, ImageStat
        stages = [('lobby', [394, 173]), ('create', [453, 573]),
                  ('room', [229, 679]), ('lobby', [876, 679])]
        if self.chat_message:
            stages.insert(2, ('room', None))
        if 'exit' in self.references:
            stages.append(('exit', [571, 491]))
        if self.step >= len(stages):
            return
        state, coordinate = stages[self.step]
        with Image.open(capture) as screen:
            if screen.size != (1024, 768):
                raise ValueError('Observed UI viewport mismatch')
            errors = [max(ImageStat.Stat(ImageChops.difference(
                screen.crop(box).convert('RGB'), reference)).mean)
                for box, reference in self.references[state]]
        matched = all(value <= 8 for value in errors)
        self.observations.append(dict(capture=str(capture), step=self.step,
                                      expected=state, channel_mean_errors=errors, matched=matched))
        result = run / 'room-lifecycle-controller.json'
        result.write_text(json.dumps(dict(step=self.step, observations=self.observations,
            limits='Pinned screen matching gates ordinary input only; no native success or clean logout inferred'), indent=2) + '\n')
        if matched:
            step = self.step
            self.step += 1  # Never retry this input, including uncertain QMP failure.
            actions = run / ('room-lifecycle-input-' + str(step) + '.json')
            inputs = ([dict(op='click', xy=[400, 629]),
                       dict(op='type', text=self.chat_message), dict(op='key', key='ret')]
                      if coordinate is None else [dict(op='click', xy=coordinate)])
            actions.write_text(json.dumps(inputs) + '\n')
            subprocess.run([sys.executable, str(ROOT / 'tools/native_windows_input.py'),
                '--run', str(run), '--actions', str(actions),
                '--out', str(run / ('room-lifecycle-input-' + str(step)))],
                check=True, capture_output=True, timeout=45)


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
    parser.add_argument('--ui-exit-probe', action='store_true', help='Observe pinned exit button/modal/predicate sites; requires socket probe')
    parser.add_argument('--crash-dump', action='store_true', help='Owned Windows WER minidump on native failure, without debugger')
    parser.add_argument('--room-lifecycle', type=Path, help='Pinned local language screenshots for bounded create/leave/UI exit control')
    parser.add_argument('--room-chat-message', help='Declared ASCII test text typed in the measured vi waiting-room chat box')
    parser.add_argument('--service-build', type=Path, help='Private JSON pinning three already built remote binaries and patched949x test account name')
    parser.add_argument('--launcher-control', type=Path,
                        help='Private JSON pinning an explicitly declared diagnostic launcher copy')
    parser.add_argument('--resume-stage', action='store_true', help='Resume an already completed private remote staging operation, never repeat it')
    parser.add_argument('--wait-timeout-control-ms', type=int, default=0,
                        help='Explicit debugger-only zero-limit timing control; not a final client patch')
    args = parser.parse_args()
    local = (ROOT / 'local').resolve(strict=True)
    launcher_control = None
    if args.launcher_control:
        declaration = args.launcher_control.resolve(strict=True)
        if not declaration.is_relative_to(local):
            parser.error('Launcher control declaration must be local')
        launcher_control = json.loads(declaration.read_text())
        binary = Path(launcher_control['path']).resolve(strict=True)
        if (not binary.is_relative_to(local) or not isinstance(launcher_control.get('limits'), str)
                or digest(binary) != launcher_control['sha256']):
            parser.error('Launcher control identity or scope mismatch')
    if args.room_chat_message and (args.language != 'vi' or not args.room_lifecycle
            or not 1 <= len(args.room_chat_message) <= 96
            or any(not (c.isascii() and (c.isalnum() or c == ' ')) for c in args.room_chat_message)):
        parser.error('Chat control needs vi room references and 1..96 ASCII letters/digits/spaces')
    lifecycle = (RoomLifecycle(args.room_lifecycle, local, args.language, args.room_chat_message)
                 if args.room_lifecycle else None)
    service_build = None
    if args.service_build:
        if not args.service_build.resolve(strict=True).is_relative_to(local):
            parser.error('Service build declaration must be local')
        service_build = json.loads(args.service_build.read_text())
        if (not Path(service_build['build']).is_absolute()
                or set(service_build['binaries']) != {'kr-service', 'kr-admin', 'kr-companion'}
                or not isinstance(service_build['account_display_name'], str)):
            parser.error('Service build declaration shape')
        for pin in service_build['binaries'].values():
            if (type(pin['bytes']) is not int or not 0 < pin['bytes'] < 64 * 1024 * 1024
                    or len(pin['sha256']) != 64
                    or any(char not in '0123456789abcdef' for char in pin['sha256'])):
                parser.error('Service binary pin')
    paths = {name: getattr(args, name.replace('-', '_')).resolve()
             for name in ['previous', 'out', 'deployment', 'manifest', 'native-control', 'windows-credentials']}
    if any(path == local or not path.is_relative_to(local) for path in paths.values()):
        parser.error('Every input/output must be under local/')
    native_run_cleanup.require_storage()
    if not 30 <= args.observe_seconds <= 240:
        parser.error('Observation budget is 30..240 seconds')
    if args.wait_timeout_control_ms and (not args.socket_probe or not 1 <= args.wait_timeout_control_ms <= 60000):
        parser.error('Wait timing control requires socket probe and1..60000ms')
    if args.crash_dump and args.socket_probe:
        parser.error('WER control must run without a debugger')
    if args.ui_exit_probe and not args.socket_probe:
        parser.error('UI exit observer requires declared socket probe')
    previous, output, stage = paths['previous'], paths['out'], paths['deployment']
    if output == stage:
        parser.error('Output and deployment must be distinct')
    if not args.resume_stage and (output.exists() or stage.exists()):
        parser.error('Output and deployment must be distinct and new')
    if args.resume_stage and (not output.is_dir() or not stage.is_dir() or any(output.iterdir())):
        parser.error('Resume is only for completed remote staging before any local VM/owner inputs')
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
    if not args.resume_stage:
        output.mkdir(mode=0o700)
        stage.mkdir(mode=0o700)
    config = json.loads((previous / 'private-owner-config.json').read_text())
    connection = json.loads(Path(config['ssh_connection']).read_text())
    plan = json.loads(Path(config['vps_plan']).read_text())
    ssh = ['ssh', '-i', config['ssh_identity'], '-p', str(connection['port']), '-o', 'StrictHostKeyChecking=yes',
           '-o', 'UserKnownHostsFile=' + config['ssh_known_hosts'], connection['user'] + '@' + connection['host']]
    old_remote = Path(plan['root'])
    # Unique remote staging only; runner creates fresh DB and only its own rules.
    stage_record = stage / 'remote-stage.json'
    saved = json.loads(stage_record.read_text()) if args.resume_stage else None
    suffix = saved['suffix'] if saved else str(time.time_ns())
    if not suffix.isdecimal():
        parser.error('Invalid stage identity')
    new_remote = old_remote.with_name('boom-m8-multilang-' + args.language + '-' + suffix)
    recipe = '''from pathlib import Path
import hashlib,ipaddress,json,os,shutil,subprocess,sys
old=Path(sys.argv[1]);new=Path(sys.argv[2]);assert not new.exists()
new.mkdir(mode=0o700);shutil.copytree(old/'package',new/'package');(new/'state').mkdir(mode=0o700)
shutil.copy2(old/'state/launcher.pk8',new/'state/launcher.pk8');(new/'state/launcher.pk8').chmod(0o600)
for name in ['guarded-runner.py','cloud-list.json']:shutil.copy2(old/name,new/name)
runner_path=new/'guarded-runner.py';runner=runner_path.read_text()
old_signals='for sig in [signal.SIGTERM,signal.SIGINT]'
new_signals='for sig in [signal.SIGTERM,signal.SIGINT,signal.SIGHUP]'
if old_signals in runner:
 assert runner.count(old_signals)==1
 runner_path.write_text(runner.replace(old_signals,new_signals))
else:assert new_signals in runner, 'Owned runner signal cleanup declaration differs'
p=json.loads((old/'private-provision.json').read_text());p['root']=str(new)
observed=str(ipaddress.IPv4Address(os.environ['SSH_CONNECTION'].split()[0]))
previous_source=p['source'];source_changed=observed!=previous_source
config_path=new/'package/config.template.json';config=json.loads(config_path.read_text())
# SERVICE_BUILD_CONFIG
assert previous_source in config['allowed_peers'], 'Previous source is absent from package allowlist'
config['allowed_peers']=[observed if peer==previous_source else peer for peer in config['allowed_peers']]
for actor in config.get('diagnostic_two_lobby',{}).get('actors',[]):
 if actor['peer']==previous_source:actor['peer']=observed
assert observed in config['allowed_peers']
config_path.chmod(0o600);config_path.write_text(json.dumps(config,ensure_ascii=False,indent=2)+chr(10));config_path.chmod(0o400)
manifest_path=new/'package/manifest.json';manifest=json.loads(manifest_path.read_text())
# SERVICE_BUILD_BINARIES
rows=[row for row in manifest['files'] if row['path']=='config.template.json'];assert len(rows)==1
rows[0].update(bytes=config_path.stat().st_size,sha256=hashlib.sha256(config_path.read_bytes()).hexdigest())
manifest_path.chmod(0o600);manifest_path.write_text(json.dumps(manifest,ensure_ascii=False,indent=2)+chr(10));manifest_path.chmod(0o400)
p['manifest_sha256']=hashlib.sha256(manifest_path.read_bytes()).hexdigest();p['source']=observed
subprocess.run([sys.executable,str(new/'package/kr-package.py'),'verify','--manifest-sha256',p['manifest_sha256']],check=True,capture_output=True,timeout=30)
(new/'private-provision.json').write_text(json.dumps(p,indent=2)+chr(10))
print(json.dumps(dict(service_started=False,cloud_changes=False,source_refreshed=source_changed,service_allowlist_refreshed=True,package_verified=True,owned_runner_handles_hup=True)))
'''
    if service_build:
        recipe = recipe.replace('# SERVICE_BUILD_CONFIG',
            'build_spec=' + repr(service_build) + "\nconfig['diagnostic_two_lobby']['account_admission']['patched_949x_names']=True\np['account']['name']=build_spec['account_display_name']")
        recipe = recipe.replace('# SERVICE_BUILD_BINARIES', '''for name,pin in build_spec['binaries'].items():
 source=Path(build_spec['build'])/name
 assert source.is_file() and source.stat().st_size==pin['bytes'] and hashlib.sha256(source.read_bytes()).hexdigest()==pin['sha256'], 'Service build identity mismatch'
 target=new/'package/bin'/name;target.chmod(0o700);shutil.copyfile(source,target);target.chmod(0o500)
 binary_rows=[row for row in manifest['files'] if row['path']=='bin/'+name];assert len(binary_rows)==1
 binary_rows[0].update(pin)
''')
    expected_stage = dict(suffix=suffix, remote_root=str(new_remote), previous=str(previous), language=args.language,
                          manifest_sha256=args.manifest_sha256, native_control_sha256=args.native_control_sha256,
                          recipe_sha256=hashlib.sha256(recipe.encode()).hexdigest())
    if args.resume_stage:
        if any(saved.get(key) != value for key, value in expected_stage.items()):
            parser.error('Resume inputs differ from the declared stage')
        receipt = json.loads((stage / 'staging.stdout').read_text())
        if not receipt.get('package_verified') or receipt.get('service_started') or receipt.get('cloud_changes'):
            parser.error('Remote staging is not confirmed complete without service/rule mutations')
    else:
        stage_record.write_text(json.dumps(expected_stage, indent=2) + '\n')
        (stage / 'staging-recipe.py').write_text(recipe)
        result = subprocess.run(ssh + ['python3 - ' + shlex.quote(str(old_remote)) + ' ' + shlex.quote(str(new_remote))],
                                input=recipe, capture_output=True, text=True, timeout=60)
        (stage / 'staging.stdout').write_text(result.stdout)
        (stage / 'staging.stderr').write_text(result.stderr)
        result.check_returncode()
    plan['root'] = str(new_remote)
    # Fetch the exact staged plan, including its freshly observed /32 source.
    # Never widen the ACL or publish the private source to stdout.
    for attempt in range(2):
        try:
            staged = subprocess.run(ssh + ['cat ' + shlex.quote(str(new_remote / 'private-provision.json'))],
                                    capture_output=True, text=True, check=True, timeout=30)
            break
        except subprocess.TimeoutExpired:
            if attempt:
                raise
    plan = json.loads(staged.stdout)
    (stage / 'private-provision.json').write_text(json.dumps(plan, indent=2) + '\n')
    config.update(out=str(output), vps_plan=str(stage / 'private-provision.json'),
                  container='kr-m8-native-' + args.language + '-' + suffix)
    (output / 'private-owner-config.json').write_text(json.dumps(config, indent=2) + '\n')
    owner = None
    prepared_only = False
    try:
        subprocess.run(['qemu-img', 'create', '-f', 'qcow2', '-F', 'qcow2', '-b', config['vm_base'],
                        str(output / 'native-system.qcow2')], check=True, capture_output=True, timeout=30)
        with (output / 'seed.img').open('xb') as stream:
            stream.truncate(128 * 1024 * 1024)
        subprocess.run(['mkfs.vfat', '-n', 'M8SETUP', str(output / 'seed.img')], check=True, capture_output=True, timeout=30)
        shutil.copy2(previous / 'start-vm.sh', output / 'start-vm.sh')
        for name in ['kr-launch.exe', 'kr-companion.exe', 'certificate.der', 'deployment-private.json', 'credentials.json']:
            shutil.copy2(previous / name, output / name)
        if launcher_control:
            shutil.copy2(launcher_control['path'], output / 'kr-launch.exe')
            (output / 'launcher-control.json').write_text(json.dumps(launcher_control, indent=2) + '\n')
        shutil.copy2(paths['native-control'], output / 'CA-control.exe')
        shutil.copy2(ROOT / 'tools/native_windows_multilang.ps1', output / 'run.ps1')
        pins = {name: digest(output / name) for name in ['kr-launch.exe', 'kr-companion.exe', 'certificate.der', 'CA-control.exe']}
        (output / 'control-pins.json').write_text(json.dumps(pins) + '\n')
        control = dict(language=args.language, runtime_name='M8Native' + args.language.capitalize() + suffix[-10:],
                       observe_seconds=args.observe_seconds)
        if args.crash_dump:
            control['crash_dump'] = True
        if args.ui_exit_probe:
            control['ui_exit_probe'] = True
        if args.wait_timeout_control_ms:
            control['wait_timeout_control_ms'] = args.wait_timeout_control_ms
        if args.socket_probe:
            shutil.copy2(ROOT / 'tools/native_windows_socket_probe.c', output / 'socket-probe-source.c')
            subprocess.run(['i686-w64-mingw32-gcc', '-municode', '-std=c11', '-O2', '-Wall', '-Wextra', '-Werror',
                            str(output / 'socket-probe-source.c'), '-o', str(output / 'socket-probe.exe')],
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
            prepared_only = True
            print('Prepared private inputs; no VM/service/rules started.')
            return
        record = dict(started_epoch=time.time(), language=args.language,
                      source_commit=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip())
        with (output / 'owner.stdout').open('xb') as stdout, (output / 'owner.stderr').open('xb') as stderr:
            owner = subprocess.Popen([sys.executable, str(ROOT / 'tools/native_windows_candidate.py'),
                                      '--config', str(output / 'private-owner-config.json')],
                                     stdout=stdout, stderr=stderr, start_new_session=True)
        try:
            (output / 'owner-process.json').write_text(json.dumps(dict(pid=owner.pid,
                start_ticks=Path('/proc/' + str(owner.pid) + '/stat').read_text().split()[21],
                source_sha256=digest(ROOT / 'tools/native_windows_candidate.py'))) + '\n')
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
                if capture < 48 or phase in ['finished', 'failed']:
                    actions = output / ('observe-' + str(capture) + '.json')
                    actions.write_text(json.dumps([dict(op='capture')]) + '\n')
                    subprocess.run([sys.executable, str(ROOT / 'tools/native_windows_input.py'), '--run', str(output),
                                    '--actions', str(actions), '--out', str(output / ('observe-' + str(capture)))],
                                   check=True, capture_output=True, timeout=45)
                    if lifecycle and phase not in ['finished', 'failed']:
                        lifecycle.observe(output / ('observe-' + str(capture)) / 'capture-0.png', output)
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
                record['owner_exit'] = owner.wait(timeout=180)
            except subprocess.TimeoutExpired:
                record['owner_cleanup_deadline'] = True
            record['ended_epoch'] = time.time()
            if lifecycle:
                record['ordinary_ui_steps_sent'] = lifecycle.step
            (output / 'controller-result.json').write_text(json.dumps(record, indent=2) + '\n')
        print('Bounded language control ended; verify exported native and cleanup evidence.')
    finally:
        # No VM was started if preparation failed before Popen. Successful
        # prepare-only output is an explicit input for a later owned run.
        if owner is None and not prepared_only:
            native_run_cleanup.cleanup(ROOT, output, stopped=True, exports_complete=True,
                inputs=[config['vm_base'], config['player_media'],
                        *config.get('additional_readonly_backing', [])])
        elif (owner is not None and owner.poll() is not None
              and not (output / 'run-cleanup-manifest.json').exists()):
            # The owner can fail before its try/finally (for example a busy
            # runtime lease). Only absence of its unique container proves no
            # guest could have produced unexported observations.
            state = subprocess.run(['docker', 'inspect', config['container']],
                                   capture_output=True, timeout=10)
            absent = native_run_cleanup.container_absent(state.returncode, state.stderr)
            native_run_cleanup.cleanup(ROOT, output, stopped=absent, exports_complete=absent,
                inputs=[config['vm_base'], config['player_media'],
                        *config.get('additional_readonly_backing', [])])



if __name__ == '__main__':
    main()
