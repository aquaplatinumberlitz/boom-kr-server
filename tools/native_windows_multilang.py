"""Prepare/run one bounded language control from an existing verified Windows VM.

All actual deployment/VM/client inputs and raw output stay under local/. No
assets are published. The previous owner must have stopped and cleaned its
rules. This does not certify clean installation or native gameplay.
"""
import argparse
import hashlib
import json
import os
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
    def __init__(self, reference_path, local, language, chat_message=None, paired=False):
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
        self.paired = paired

    def observe(self, capture, run):
        from PIL import Image, ImageChops, ImageStat
        stages = [('lobby', [394, 173]), ('create', [453, 573]),
                  ('room', [229, 679]), ('lobby', [876, 679])]
        if self.chat_message:
            stages.insert(2, ('room', None))
        if self.paired:
            stages = stages[:2] + [('room', 'start-paired')]
        if 'exit' in self.references and not self.paired:
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
            if coordinate == 'start-paired':
                request_guest_marker(run, 'pair.go')
                return
            actions = run / ('room-lifecycle-input-' + str(step) + '.json')
            inputs = ([dict(op='click', xy=[400, 629]),
                       dict(op='type', text=self.chat_message), dict(op='key', key='ret')]
                      if coordinate is None else [dict(op='click', xy=coordinate)])
            actions.write_text(json.dumps(inputs) + '\n')
            subprocess.run([sys.executable, str(ROOT / 'tools/native_windows_input.py'),
                '--run', str(run), '--actions', str(actions),
                '--out', str(run / ('room-lifecycle-input-' + str(step)))],
                check=True, capture_output=True, timeout=45)


def request_guest_marker(run, marker):
    """Guest owns FAT writes; signal via an observed ordinary Windows Run dialog."""
    from PIL import Image
    import re
    if marker not in ('pair.go', 'pair.end'):
        raise ValueError('Unknown bounded pair marker')
    prefix = 'guest-marker-' + marker.replace('.', '-')
    record = dict(marker=marker, started_epoch=time.time(), inputs=[])

    def actions(label, values):
        path = run / (prefix + '-' + label + '.json')
        path.write_text(json.dumps(values) + '\n')
        output = run / (prefix + '-' + label)
        result = subprocess.run([sys.executable, str(ROOT / 'tools/native_windows_input.py'),
            '--run', str(run), '--actions', str(path), '--out', str(output)],
            capture_output=True, text=True, timeout=45)
        (run / (prefix + '-' + label + '.stderr')).write_text(result.stderr)
        record['inputs'].append(dict(step=label, exit=result.returncode))
        result.check_returncode()
        return output / ('capture-' + str(len(values) - 1) + '.png')

    def ocr(image, box, label):
        with Image.open(image) as screen:
            if screen.size != (1024, 768):
                raise ValueError('Marker viewport mismatch')
            region = screen.crop(box)
            region = region.resize((region.width * 4, region.height * 4))
            target = run / (prefix + '-' + label + '-ocr.png')
            region.save(target)
        result = subprocess.run(['tesseract', str(target), 'stdout', '--psm', '6' if label == 'dialog' else '7'],
                                capture_output=True, text=True, check=True, timeout=15)
        (run / (prefix + '-' + label + '-ocr.txt')).write_text(result.stdout)
        return result.stdout.lower()

    try:
        screen = actions('open', [dict(op='key', key='meta_l-r', wait_seconds=1), dict(op='capture')])
        text = ocr(screen, (16, 511, 415, 719), 'dialog')
        if not all(value in text for value in ('open', 'windows will open', 'type the name')):
            raise ValueError('Marker Run dialog not observed')
        screen = actions('clear', [dict(op='click', xy=[229, 622]),
            dict(op='key', key='home', hold_ms=200), dict(op='key', key='shift-end', hold_ms=200),
            dict(op='key', key='backspace', hold_ms=200), dict(op='click', xy=[200, 526]), dict(op='capture')])
        with Image.open(screen) as image:
            if any(pixel < 128 for pixel in image.crop((90, 616, 380, 629)).convert('L').tobytes()):
                raise ValueError('Marker Run input not observed empty')
        command = 'cmd /c echo go>d:\\' + marker
        screen = actions('command', [dict(op='type', text=command), dict(op='key', key='tab'), dict(op='capture')])
        text = ocr(screen, (80, 612, 388, 635), 'command')
        if re.sub('[^a-z0-9]', '', text) != re.sub('[^a-z0-9]', '', command):
            raise ValueError('Marker command not observed; no execution')
        final_input = ([dict(op='key', key='ret')] if marker == 'pair.end' else
                       [dict(op='key', key='ret', wait_seconds=2), dict(op='capture')])
        actions('execute', final_input)
        record['ordinary_run_command_sent'] = True
    finally:
        record['ended_epoch'] = time.time()
        (run / (prefix + '-result.json')).write_text(json.dumps(record, indent=2) + '\n')


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
    parser.add_argument('--paired-control', type=Path,
                        help='Private ko input/account declaration; start second client after vi room is observed')
    parser.add_argument('--paired-startup-probe', action='store_true',
                        help='Observe only the secondary native mutex branch under a declared debugger control')
    parser.add_argument('--resume-stage', action='store_true', help='Resume an already completed private remote staging operation, never repeat it')
    parser.add_argument('--wait-timeout-control-ms', type=int, default=0,
                        help='Explicit debugger-only zero-limit timing control; not a final client patch')
    args = parser.parse_args()
    if args.paired_startup_probe and (not args.paired_control or args.crash_dump or args.socket_probe):
        parser.error('Paired startup observer requires paired control without WER or socket observer')
    local = (ROOT / 'local').resolve(strict=True)
    paired = None
    if args.paired_control:
        path = args.paired_control.resolve(strict=True)
        if not path.is_relative_to(local):
            parser.error('Paired declaration must stay local')
        paired = json.loads(path.read_text())
        binary = Path(paired['path']).resolve(strict=True)
        account = paired['account']
        if (args.language != 'vi' or paired.get('language') != 'ko' or not args.room_lifecycle
                or not binary.is_relative_to(local) or digest(binary) != paired['sha256']
                or set(account) != {'login', 'name', 'password'}
                or not all(isinstance(value, str) for value in account.values())
                or not 3 <= len(account['login']) <= 32 or not account['login'].isascii()
                or not account['login'].replace('_', '').replace('-', '').isalnum()
                or not 12 <= len(account['password']) <= 128
                or any(char.isspace() or ord(char) < 32 or ord(char) == 127 for char in account['name'])
                or args.socket_probe or args.room_chat_message):
            parser.error('Paired vi/ko control identity, account, lifecycle or probe bounds')
        from l10n_949x import encode
        if not 3 <= len(encode(account['name'])) <= 15:
            parser.error('Paired name is outside native byte bounds')
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
    lifecycle = (RoomLifecycle(args.room_lifecycle, local, args.language, args.room_chat_message, bool(paired))
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
    if paired:
        account = paired['account']
        anchor = "assert observed in config['allowed_peers']"
        patch = r"""pair_account=PAIR_ACCOUNT
assert pair_account['login'].casefold()!=p['account']['login'].casefold(), 'Paired account must be distinct'
p['paired_account']=pair_account
admission=config['diagnostic_two_lobby']['account_admission']
assert admission['profile_mode']=='database_zero' and admission.get('patched_949x_names'), 'Paired run needs patched database profiles'
assert admission.get('max_sessions') is None or admission['max_sessions']>=2, 'Paired session capacity'
runner=runner_path.read_text()
anchor="state['player']=json.loads(provision.stdout)['player'];save()"
assert runner.count(anchor)==1, 'Owned runner account seam changed'
extra="\n pair_request=dict(request,command=dict(request['command'],**plan['paired_account']))\n pair_provision=subprocess.run([str(package/'bin/kr-admin'),str(sroot/'accounts.sqlite')],input=json.dumps(pair_request),capture_output=True,text=True,check=True,timeout=20)\n state['paired_player']=json.loads(pair_provision.stdout)['player'];save()"
patched_runner=runner.replace(anchor,anchor+extra)
compile(patched_runner,'owned-pair-runner','exec')
runner_path.write_text(patched_runner)
""".replace('PAIR_ACCOUNT', repr(account))
        recipe = recipe.replace(anchor, anchor + '\n' + patch)
    # Revoke only still-live sessions of the accounts created by this isolated
    # runner, after its service stops. Preserve native logout counts separately.
    cleanup_patch = r'''runner=runner_path.read_text()
anchor=' if capture is not None:\n'
if 'owned_account_cleanup' not in runner:
 assert runner.count(anchor)==1, 'Owned runner cleanup seam changed'
 cleanup=""" if state.get('player') and (root/'state/accounts.sqlite').exists():
  try:
   import sqlite3
   db=root/'state/accounts.sqlite'
   connection=sqlite3.connect('file:'+str(db)+'?mode=ro',uri=True)
   players=[p for p in [state.get('player'),state.get('paired_player')] if p is not None]
   pending=[p for p in players if connection.execute('SELECT count(*) FROM sessions WHERE account_id=? AND revoked_at IS NULL',[p['account_id']]).fetchone()[0]]
   connection.close();state['owned_account_cleanup']=[]
   for player in pending:
    request={'actor':'m8-owned-run-cleanup','reason':'Revoke residual session after isolated run service stopped; not native logout proof','command':{'op':'revoke','account_id':player['account_id']}}
    result=subprocess.run([str(root/'package/bin/kr-admin'),str(db)],input=json.dumps(request),capture_output=True,text=True,timeout=20)
    state['owned_account_cleanup'].append(dict(account_id=player['account_id'],revoke_exit=result.returncode));result.check_returncode()
   save()
  except Exception as error:state.setdefault('cleanup_errors',[]).append('owned account cleanup: '+type(error).__name__);save()
"""
 patched=runner.replace(anchor,cleanup+anchor)
 compile(patched,'owned-runner-account-cleanup','exec');runner_path.write_text(patched)
'''
    recipe = recipe.replace("p=json.loads((old/'private-provision.json').read_text());p['root']=str(new)",
        cleanup_patch + "\np=json.loads((old/'private-provision.json').read_text());p['root']=str(new)")
    compile(recipe, '<private-staging-recipe>', 'exec')
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
        script = 'native_windows_pair.ps1' if paired else 'native_windows_multilang.ps1'
        shutil.copy2(ROOT / 'tools' / script, output / 'run.ps1')
        pins = {name: digest(output / name) for name in ['kr-launch.exe', 'kr-companion.exe', 'certificate.der', 'CA-control.exe']}
        (output / 'control-pins.json').write_text(json.dumps(pins) + '\n')
        control = dict(language=args.language, runtime_name='M8Native' + args.language.capitalize() + suffix[-10:],
                       observe_seconds=args.observe_seconds)
        paired_files = []
        if paired:
            (output / 'run.ps1').rename(output / 'paired-run.ps1')
            (output / 'run.ps1').write_text(r'''$ErrorActionPreference='Stop'
$seed=Split-Path -Parent $MyInvocation.MyCommand.Path
$script=Join-Path $seed 'paired-run.ps1'
$tokens=$null;$parseErrors=$null
$null=[System.Management.Automation.Language.Parser]::ParseFile($script,[ref]$tokens,[ref]$parseErrors)
if($parseErrors.Count -gt 0){
 $report=@{phase='failed';error='Paired script parsing failed';parse_errors=@($parseErrors|ForEach-Object {$_.Message})}
 [IO.File]::WriteAllText((Join-Path $seed 'native-windows-install.json'),($report|ConvertTo-Json -Depth 6))
 exit 1
}
& $script
''')
            rows = manifest['languages']['ko']['files']
            pair_inputs = [dict(path=row['path'], bytes=row['bytes'], sha256=row['sha256']) for row in manifest['packs']]
            pair_inputs += [dict(path=row['path'], bytes=row['payload']['bytes'], sha256=row['payload']['sha256'])
                            for row in manifest['common'] + rows if row['path'] != 'CA.exe']
            shutil.copy2(paired['path'], output / 'CA-paired-control.exe')
            (output / 'paired-language-inputs.json').write_text(json.dumps(pair_inputs, indent=2) + '\n')
            (output / 'paired-credentials.json').write_text(json.dumps(
                {key: paired['account'][key] for key in ('login', 'password')}) + '\n')
            control['paired'] = dict(language='ko', runtime_name='M8NativeKo' + suffix[-10:],
                                     exe_sha256=paired['sha256'])
            paired_files = ['paired-run.ps1', 'CA-paired-control.exe', 'paired-language-inputs.json', 'paired-credentials.json']
        if args.crash_dump:
            control['crash_dump'] = True
        if args.ui_exit_probe:
            control['ui_exit_probe'] = True
        if args.wait_timeout_control_ms:
            control['wait_timeout_control_ms'] = args.wait_timeout_control_ms
        if args.socket_probe or args.paired_startup_probe:
            shutil.copy2(ROOT / 'tools/native_windows_socket_probe.c', output / 'socket-probe-source.c')
            subprocess.run(['i686-w64-mingw32-gcc', '-municode', '-std=c11', '-O2', '-Wall', '-Wextra', '-Werror',
                            str(output / 'socket-probe-source.c'), '-o', str(output / 'socket-probe.exe')],
                           check=True, capture_output=True, timeout=60)
            field = 'paired_startup_probe_sha256' if args.paired_startup_probe else 'socket_probe_sha256'
            control[field] = digest(output / 'socket-probe.exe')
            subprocess.run(['mcopy', '-i', str(output / 'seed.img'), str(output / 'socket-probe.exe'), '::socket-probe.exe'],
                           check=True, capture_output=True, timeout=30)
        (output / 'multilang-run.json').write_text(json.dumps(control) + '\n')
        (output / 'language-inputs.json').write_text(json.dumps(files, indent=2) + '\n')
        (output / 'input-summary.json').write_text(json.dumps(dict(files=len(files) + 1,
            exe_sha256=args.native_control_sha256, launcher_sha256=pins['kr-launch.exe'],
            manifest_sha256=args.manifest_sha256, language=args.language,
            scope='Existing verified language installation with separate loader-control runtime; no acceptance inferred')) + '\n')
        for name in ['run.ps1', 'multilang-run.json', 'language-inputs.json', 'control-pins.json', 'kr-launch.exe',
                     'kr-companion.exe', 'CA-control.exe', 'certificate.der', 'deployment-private.json', 'credentials.json'] + paired_files:
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
                # Read-only diagnostic of guest-owned live FAT. Windows can
                # temporarily mark FAT mirrors dirty while replacing a report;
                # mtools' mount check otherwise rejects the whole image. This
                # never changes FAT bytes; final evidence is exported stopped.
                live_env = dict(os.environ, MTOOLS_SKIP_CHECK='1')
                result = subprocess.run(['mtype', '-i', str(output / 'seed.img'), '::native-windows-install.json'],
                                        capture_output=True, text=True, timeout=5, env=live_env)
                try:
                    guest = json.loads(result.stdout.lstrip('\ufeff')) if result.returncode == 0 else {}
                except json.JSONDecodeError:
                    guest = {}  # Live FAT read can race a report write; not final evidence.
                phase = guest.get('phase')
                manual_pair = paired and lifecycle and lifecycle.step >= 3
                if not manual_pair and (capture < 48 or phase in ['finished', 'failed']):
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
