"""Relocatable packaging of pinned probe fixtures and the existing Python service.

No client, asset, native database or fixture encoder is included. Hash checks
detect drift; they are not signatures or evidence of native compatibility.
"""
import copy
import hashlib
import json
from pathlib import Path
import re
import subprocess


ROOT = Path(__file__).resolve().parents[1]

LAUNCHER = '''#!/usr/bin/env python3
"""Verify this local diagnostic bundle, then invoke its unchanged service."""
import hashlib
import json
import os
from pathlib import Path
import sys

root = Path(__file__).resolve().parent
manifest = json.loads((root / 'bundle.json').read_text())
if manifest.get('schema') != 'kre-probe-bundle-v1':
    raise SystemExit('unsupported bundle schema')
seen = set()
for row in manifest['files']:
    relative = Path(row['path'])
    path = (root / relative).resolve(strict=True)
    if relative.is_absolute() or not path.is_relative_to(root) or str(relative) in seen:
        raise SystemExit('invalid bundle file path')
    seen.add(str(relative))
    raw = path.read_bytes()
    if len(raw) != row['bytes'] or hashlib.sha256(raw).hexdigest() != row['sha256']:
        raise SystemExit('bundle file mismatch: ' + str(relative))
if not {'run.py', 'profile.json', 'server/__init__.py', 'server/probe_service.py'} <= seen:
    raise SystemExit('incomplete bundle manifest')
# -I does not add the script directory: insert only this verified bundle root.
sys.path.insert(0, str(root))
os.chdir(root)
from server.probe_service import FixtureProfile, main
profile = FixtureProfile(root / 'profile.json')
if sys.argv[1:] == ['--verify-only']:
    print(json.dumps({'verified_files': len(seen), 'fixtures': len(profile.frames),
                      'scope': 'file pins/profile only; not native/LAN certification'}))
    raise SystemExit(0)
if any(arg == '--profile' or arg.startswith('--profile=') for arg in sys.argv[1:]):
    raise SystemExit('bundle fixes --profile; export another bundle to change it')
print(json.dumps({'bundle_root': str(root),
                  'profile_sha256': hashlib.sha256((root / 'profile.json').read_bytes()).hexdigest(),
                  'service_module': sys.modules[main.__module__].__file__}), flush=True)
sys.argv[1:1] = ['--profile', str(root / 'profile.json')]
main()
'''


def encoded(value):
    return (json.dumps(value, ensure_ascii=False, indent=2) + '\n').encode()


def export_bundle(profile, source_root, out, *, four_bomb_profile=None):
    """Validate every input before reserving a new directory; never overwrite.

    CLI callers supply load_preset's validated result. Repeat the pin checks
    here so a file changed between preset loading and packaging is refused.
    """
    source_root = Path(source_root).resolve(strict=True)
    out = Path(out).resolve()
    if out.exists():
        raise FileExistsError('bundle output already exists: ' + str(out))
    result = copy.deepcopy(profile)
    if result.get('schema') != 'kr-probe-profile-v1':
        raise ValueError('unsupported profile')
    four = four_bomb_profile is not None
    if four and (json.dumps(four_bomb_profile, sort_keys=True) != json.dumps(dict(mode='four_current_baseline',
            capacities={'A':1,'B':2,'C':1,'D':1}, powers=dict.fromkeys('ABCD',1), fuse_seconds=6), sort_keys=True)):
        raise ValueError('only measured current four-player bomb preset is supported')
    payloads = {}
    for alias, entry in result['fixtures'].items():
        path = Path(entry['path']).resolve(strict=True)
        if not path.is_relative_to(source_root):
            raise ValueError('fixture escapes source root: ' + alias)
        raw = path.read_bytes()
        if (type(entry['bytes']) is not int or not 1 <= entry['bytes'] <= 65536
                or not re.fullmatch('[0-9a-f]{64}', str(entry['sha256']))
                or len(raw) != entry['bytes']
                or hashlib.sha256(raw).hexdigest() != entry['sha256']):
            raise ValueError('fixture size/hash mismatch: ' + alias)
        relative = 'fixtures/' + entry['sha256'] + '.bin'
        payloads[relative] = raw
        entry['path'] = relative
    # Explicitly copy only runtime Python sources, never caches/tests/assets.
    for path in sorted((ROOT / 'server').glob('*.py')):
        if not path.name.startswith('test_'):
            payloads['server/' + path.name] = path.read_bytes()
    payloads['server/__init__.py'] = b''
    payloads['profile.json'] = encoded(result)
    launcher = LAUNCHER
    entrypoint = 'server.probe_service'
    if four:
        entrypoint = 'server.four_player_service'
        payloads['four-bomb-profile.json'] = encoded(four_bomb_profile)
        launcher = launcher.replace("'server/probe_service.py'} <= seen", "'server/probe_service.py', 'server/four_player_service.py', 'server/four_battle_policy.py', 'four-bomb-profile.json'} <= seen")
        launcher = launcher.replace('from server.probe_service import FixtureProfile, main',
            'from server.probe_service import FixtureProfile\nfrom server.four_player_service import main')
        launcher = launcher.replace("if any(arg == '--profile' or arg.startswith('--profile=') for arg in sys.argv[1:]):",
            "if any(arg == flag or arg.startswith(flag + '=') for flag in ['--profile', '--bomb-profile'] for arg in sys.argv[1:]):")
        launcher = launcher.replace("sys.argv[1:1] = ['--profile', str(root / 'profile.json')]",
            "sys.argv[1:1] = ['--profile', str(root / 'profile.json'), '--bomb-profile', str(root / 'four-bomb-profile.json'), '--battle-outcome']")
    payloads['run.py'] = launcher.encode()
    context = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    manifest = dict(
        schema='kre-probe-bundle-v1', context_commit=context, entrypoint=entrypoint,
        source='design', status='design',
        limits='Pinned synthetic wire fixtures/exact Python sources only. Not client/assets/auth, '
               'a signed release, runtime/LAN proof or network isolation. Default startup remains '
               'serialized; explicit source-IP admission is opt-in, not account authentication.',
        fixture_aliases=len(result['fixtures']),
        fixture_files=len({q['path'] for q in result['fixtures'].values()}),
        files=[dict(path=path, bytes=len(raw), sha256=hashlib.sha256(raw).hexdigest())
               for path, raw in sorted(payloads.items())])
    out.parent.mkdir(parents=True, exist_ok=True)
    out.mkdir()  # Atomic refusal if another writer reserved it after validation.
    # An I/O failure leaves this new directory for diagnosis; absence of the
    # final manifest prevents the launcher from treating partial output as valid.
    for path, raw in payloads.items():
        target = out / path
        target.parent.mkdir(parents=True, exist_ok=True)
        with target.open('xb') as stream:
            stream.write(raw)
    with (out / 'bundle.json').open('xb') as stream:
        stream.write(encoded(manifest))
    return manifest
