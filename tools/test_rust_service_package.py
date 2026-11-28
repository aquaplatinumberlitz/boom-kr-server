"""Kiểm integrity/đường dẫn gói; fake ELF ở đây không phải chứng nhận runtime."""
import copy
from contextlib import nullcontext, redirect_stderr, redirect_stdout
import hashlib
from io import StringIO
import json
from pathlib import Path
import stat
import shutil
import struct
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parent))
import rust_package_runtime as runtime
import rust_service_package as builder
from rust_package_runtime import digest, prepare, verify
from rust_service_package import PACK_DATA, build


class PackageTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.binaries = self.root / 'binaries'
        self.binaries.mkdir()
        header = bytearray(32)
        header[:6] = b'\x7fELF\x02\x01'
        struct.pack_into('<H', header, 18, 62)
        for name in ['kr-service', 'kr-admin', 'kr-companion', 'kr-launch-identity', 'kr-load']:
            (self.binaries / name).write_bytes(header + name.encode())
        self.fixture = self.root / 'source.bin'
        self.fixture.write_bytes(b'owned fixture bytes')
        pin = dict(path=str(self.fixture), bytes=self.fixture.stat().st_size, sha256=digest(self.fixture))
        self.config = dict(authenticated_relay=dict(bind='127.0.0.1:41000'), diagnostic_two_lobby=dict(
            account_admission=dict(profile_mode='database_zero', database='/old/state.sqlite', launch_socket='/old/launch.sock'),
            fixed_room=dict(endpoint_policy='authenticated_relay', packets=dict(first=dict(fixture=pin), second=dict(fixture=copy.deepcopy(pin))))))
        self.out = self.root / 'package'

    def package(self):
        return build(self.config, self.binaries, self.out, 'x86_64-unknown-linux-gnu')

    def test_portable_assets_deduplicate_and_do_not_copy_account_database(self):
        expected = self.package()
        data = verify(self.out, expected)
        self.assertEqual(len(list((self.out / 'data').iterdir())), 1)
        value = json.loads((self.out / 'config.template.json').read_text())
        self.assertEqual(value['diagnostic_two_lobby']['account_admission']['database'], '@STATE@/accounts.sqlite')
        for row in data['files']:
            self.assertFalse(Path(row['path']).is_absolute())
        self.assertEqual(self.config['diagnostic_two_lobby']['account_admission']['database'], '/old/state.sqlite')
        self.assertFalse(list(self.out.rglob('*.sqlite')))
        with self.assertRaises(FileExistsError):
            self.package()

    def test_wrong_source_hash_fails_before_installable_manifest(self):
        self.fixture.write_bytes(b'changed fixture')
        with self.assertRaisesRegex(ValueError, 'pin'):
            self.package()
        self.assertFalse((self.out / 'manifest.json').exists())

    def test_launcher_key_is_external_state_and_never_copied_into_bundle(self):
        secret = self.root / 'operator-key.pk8'
        secret.write_bytes(b'private-test-material-never-shipped')
        public = self.config['diagnostic_two_lobby']['account_admission']['public_launcher'] = dict(
            private_key=str(secret), certificate=dict(path=str(self.fixture),
            bytes=self.fixture.stat().st_size, sha256=digest(self.fixture)))
        native = self.config['diagnostic_two_lobby']['account_admission']['prelogin_tls'] = dict(public, certificate=dict(public['certificate']))
        public['session_ttl_seconds'], public['session_idle_seconds'] = 300, 120
        expected = self.package()
        template = json.loads((self.out / 'config.template.json').read_text())
        config, _, _ = prepare(self.out, self.root / 'operator-state')
        runtime = json.loads(config.read_text())
        for section, name in [('public_launcher', 'launcher.pk8'), ('prelogin_tls', 'prelogin.pk8')]:
            self.assertEqual(template['diagnostic_two_lobby']['account_admission'][section]['private_key'],
                             '@STATE@/' + name)
            self.assertEqual(runtime['diagnostic_two_lobby']['account_admission'][section]['private_key'],
                             str(self.root / 'operator-state' / name))
        tls = runtime['diagnostic_two_lobby']['account_admission']
        self.assertEqual((tls['prelogin_tls']['session_ttl_seconds'], tls['prelogin_tls']['session_idle_seconds']),
                         (28_800, 1_800))
        self.assertEqual((tls['public_launcher']['session_ttl_seconds'], tls['public_launcher']['session_idle_seconds']),
                         (300, 120))
        self.assertEqual(public['private_key'], str(secret))
        self.assertEqual(native['private_key'], str(secret))
        for path in self.out.rglob('*'):
            if path.is_file():
                self.assertNotIn(secret.read_bytes(), path.read_bytes())
        verify(self.out, expected)

    def test_relative_fixture_paths_follow_recipe_directory_not_process_cwd(self):
        for packet in self.config['diagnostic_two_lobby']['fixed_room']['packets'].values():
            packet['fixture']['path'] = 'source.bin'
        expected = build(self.config, self.binaries, self.out, 'x86_64-unknown-linux-gnu', self.root)
        verify(self.out, expected)
        self.assertEqual(next((self.out / 'data').iterdir()).read_bytes(), self.fixture.read_bytes())

    def test_file_and_manifest_tampering_are_rejected(self):
        expected = self.package()
        fixture = next((self.out / 'data').iterdir())
        fixture.chmod(0o600)
        fixture.write_bytes(b'tampered')
        with self.assertRaisesRegex(ValueError, 'khớp manifest'):
            verify(self.out, expected)
        manifest = self.out / 'manifest.json'
        manifest.chmod(0o600)
        manifest.write_text('{}')
        with self.assertRaisesRegex(ValueError, 'hash bàn giao'):
            verify(self.out, expected)

    def test_identical_external_symlink_and_extra_files_are_rejected(self):
        expected = self.package()
        fixture = next((self.out / 'data').iterdir())
        fixture.unlink()
        fixture.symlink_to(self.fixture)
        with self.assertRaisesRegex(ValueError, 'symlink'):
            verify(self.out, expected)
        fixture.unlink()
        fixture.write_bytes(self.fixture.read_bytes())
        fixture.chmod(0o400)
        (self.out / 'unlisted').write_text('extra')
        with self.assertRaisesRegex(ValueError, 'ngoài manifest'):
            verify(self.out, expected)

    def test_traversal_is_rejected_even_with_new_manifest_hash(self):
        self.package()
        path = self.out / 'manifest.json'
        value = json.loads(path.read_text())
        value['files'][0]['path'] = '../source.bin'
        path.chmod(0o600)
        path.write_text(json.dumps(value))
        with self.assertRaisesRegex(ValueError, 'Đường dẫn'):
            verify(self.out, digest(path))

    def test_state_is_private_separate_and_rebased_after_install(self):
        expected = self.package()
        state = self.root / 'state'
        config, out, database = prepare(self.out, state)
        value = json.loads(config.read_text())['diagnostic_two_lobby']['account_admission']
        self.assertEqual(value['database'], str(state / 'accounts.sqlite'))
        self.assertEqual(value['launch_socket'], str(state / 'launch.sock'))
        self.assertEqual(stat.S_IMODE(state.stat().st_mode), 0o700)
        self.assertEqual(stat.S_IMODE(config.stat().st_mode), 0o600)
        self.assertFalse(database.exists())
        self.assertFalse(out.exists())
        verify(self.out, expected)
        with self.assertRaisesRegex(ValueError, 'ngoài gói'):
            prepare(self.out, self.out / 'state')

    def test_target_and_legacy_transport_are_not_silently_selected(self):
        with self.assertRaisesRegex(ValueError, 'Target Linux'):
            build(self.config, self.binaries, self.out, 'x86_64-pc-windows-msvc')
        with self.assertRaisesRegex(ValueError, 'Target'):
            build(self.config, self.binaries, self.out, 'aarch64-unknown-linux-gnu')
        self.config['diagnostic_two_lobby']['fixed_room']['endpoint_policy'] = 'legacy_direct'
        with self.assertRaisesRegex(ValueError, 'relay xác thực'):
            build(self.config, self.binaries, self.root / 'legacy', 'x86_64-unknown-linux-gnu')

    def test_prepare_friends_uses_private_guest_transport(self):
        self.package()
        path = self.out / 'config.template.json'
        template = json.loads(path.read_bytes())
        two = template['diagnostic_two_lobby']
        two['actors'] = [{'identity': 1, 'peer': '192.168.0.2', 'reported_login_ipv4': '192.168.0.2'}]
        two['account_admission'].update(
            patched_949x_names=True, public_launcher={'certificate': '@INSTALL@'},
            prelogin_tls={'certificate': '@INSTALL@'}, backup={}
        )
        path.chmod(0o600)
        path.write_text(json.dumps(template))
        state = self.root / 'friends-state'
        state.mkdir(mode=0o700)
        directory = state / 'prelogin_tls-identity'
        directory.mkdir(mode=0o700)
        (directory / 'certificate.der').write_bytes(b'certificate fixture')
        runtime, _, _ = prepare(self.out, state, friends=True)
        config = json.loads(runtime.read_bytes())
        self.assertEqual(config['allowed_peers'], ['127.0.0.1'])
        self.assertEqual(config['diagnostic_two_lobby']['actors'], [{'identity': 1, 'peer': '127.0.0.1'}])
        admission = config['diagnostic_two_lobby']['account_admission']
        self.assertNotIn('public_launcher', admission)
        self.assertNotIn('backup', admission)
        tls = admission['prelogin_tls']
        self.assertTrue(tls['friends_guest_only'])
        self.assertEqual(tls['certificate']['path'], str(directory / 'certificate.der'))
        relay = config['authenticated_relay']
        self.assertEqual(relay['bind'], relay['advertise'])
        self.assertEqual(relay['bind'].split(':')[0], '127.0.0.1')
        self.assertEqual(tls['bind'].split(':')[0], '127.0.0.1')
        self.assertNotEqual(relay['bind'], tls['bind'])

    def test_o1_excludes_power_ten_only_if_a_map_uses_terrain(self):
        from kr_pack_config import configure
        from kr_pack_export import TERRAIN_SEEDS, export
        from test_kr_pack_install import MAPS, fake_game, fake_recipe

        game, source = self.root / 'game', self.root / 'wire'
        pins, seeds = fake_game(game)
        recipe = fake_recipe(source)
        recipe['diagnostic_two_lobby']['fixed_room']['map_choices'] = []
        with patch.dict(TERRAIN_SEEDS, seeds):
            cache, _ = export(game, self.root / 'cache', pins=pins, maps=MAPS)
        tables = cache / 'tables.json'
        value = json.loads(tables.read_bytes())
        stats = value['characters']
        stats['3002'] = dict(stats['1'])
        stats['3003'] = dict(stats['1'], maximum_power=10)
        tables.write_text(json.dumps(value))
        for scope, unlocked, expected in [
            ('g1', True, [0, 1, 3002]),
            ('g1', False, [0, 1]),
            ('f1', True, [0, 1, 3002, 3003]),
        ]:
            with self.subTest(scope=scope, unlocked=unlocked):
                config = copy.deepcopy(recipe)
                out = self.root / f'{scope}-{unlocked}'
                out.mkdir()
                configure(config, source, cache, out, MAPS, scope, unlock_all=unlocked)
                selected = config['diagnostic_two_lobby']['managed_rooms']['player_round']
                self.assertEqual(selected['characters'], expected)
                self.assertEqual(selected['pack_stats'], {str(key): stats[str(key)] for key in expected})

    def test_prebuilt_pack_data_needs_a_pack_scope_and_an_exact_export(self):
        with self.assertRaisesRegex(ValueError, 'g1 hoặc f1'):
            build(self.config, self.binaries, self.out, 'x86_64-unknown-linux-gnu', pack_data=PACK_DATA)
        from test_kr_pack_install import fake_recipe
        recipe = fake_recipe(self.root / 'wire')
        with self.assertRaisesRegex(ValueError, 'g1 hoặc f1'):
            build(recipe, self.binaries, self.out, 'x86_64-unknown-linux-gnu', pack_scope='f1-a',
                  pack_data=PACK_DATA)
        extra = self.root / 'pack-data'
        shutil.copytree(PACK_DATA, extra)
        (extra / 'FxMap.idd').write_bytes(b'client pack is never shipped')
        with self.assertRaisesRegex(ValueError, 'Manifest cache'):
            build(recipe, self.binaries, self.out, 'x86_64-unknown-linux-gnu', pack_scope='g1', pack_data=extra)
        self.assertFalse(self.out.exists())

    def test_repository_pack_data_ships_in_a_g1_bundle(self):
        from test_kr_pack_install import fake_recipe
        expected = build(fake_recipe(self.root / 'wire'), self.binaries, self.out, 'x86_64-unknown-linux-gnu',
                         pack_scope='g1', pack_data=PACK_DATA)
        manifest = verify(self.out, expected)
        export = json.loads((PACK_DATA / 'manifest.json').read_bytes())
        self.assertEqual((manifest['pack_data'], manifest['pack_data_identity']), ('pack-data', export['identity']))
        rows = {row['path']: row for row in manifest['files'] if row['path'].startswith('pack-data/')}
        self.assertEqual(set(rows), {'pack-data/' + path.name for path in PACK_DATA.iterdir()})
        for path in PACK_DATA.iterdir():
            self.assertEqual(rows['pack-data/' + path.name]['sha256'], digest(path))

    def test_legacy_bundle_manifests_match_pre_opt_in_bytes(self):
        # Pin the whole manifest (including every member's hash and mode) with stable source bytes.
        for relative in (
            'tools/rust_package_runtime.py', 'tools/kr_pack_export.py', 'tools/kr_pack_config.py',
            'tools/content_census.py', 'tools/export_map_profile.py', 'tools/rust_map_profile.py',
            'server/probe_codec.py', 'server/room_title.py',
        ):
            source = self.root / relative
            source.parent.mkdir(exist_ok=True)
            source.write_bytes(relative.encode())
        pack_data = self.root / 'prebuilt'
        pack_data.mkdir()
        (pack_data / 'manifest.json').write_bytes(b'{"identity":"test-export"}\n')
        (pack_data / 'tables.json').write_bytes(b'{}\n')
        self.config['diagnostic_two_lobby']['managed_rooms'] = dict(player_round={'characters': [0]}, player_choices=True)
        recipe = self.root / 'recipe.json'
        recipe.write_text(json.dumps(self.config), encoding='utf-8')
        local = self.root / 'local'
        local.mkdir()
        # Fingerprints from the builder at ef285c1.
        baselines = {
            'g1': '72d6b4704cdfec416025d3a5c516c208c7cad14e699abd2bfa4dc7a7ef1c488a',
            'f1': '2219fe741fcbd931098f3ddd055c2506636043092870a8abdb0a0b3823b7de19',
            'f1-a': 'a7dccff47646239fe4a348d18869cc73d3fbc217c116d95abc2cd0ed00747252',
        }
        with (patch.object(builder, 'ROOT', self.root),
              patch.object(builder, 'PACK_DATA', pack_data),
              patch.object(builder, '__file__', str(self.root / 'tools/rust_service_package.py')),
              patch('kr_pack_export.prebuilt', return_value=pack_data)):
            for scope, baseline in baselines.items():
                with self.subTest(scope=scope):
                    bundle = local / scope
                    options = [] if scope == 'g1' else ['--pack-scope', scope]
                    with (patch.object(sys, 'argv', self.builder_argv(recipe, bundle, *options)),
                          redirect_stdout(StringIO()) as stdout):
                        builder.main()
                    expected = json.loads(stdout.getvalue())['manifest_sha256']
                    manifest = verify(bundle, expected)
                    self.assertNotIn('f2_maps', manifest)
                    self.assertNotIn('f3_maps', manifest)
                    self.assertEqual(digest(bundle / 'manifest.json'), baseline)

    def test_builder_rejects_invalid_opt_in_selections_before_creating_bundle(self):
        target = 'x86_64-unknown-linux-gnu'
        for adapter, other in [('f2', 'f3'), ('f3', 'f2')]:
            key, flag = f'{adapter}_maps', f'--{adapter}-map'
            selected = {key: ['CAPong01']}
            scope = f'{adapter}-optin'
            cases = [
                (scope, {}, flag),
                (scope, dict(selected, **{f'{other}_maps': ['CAPong02']}), f'--{other}-map'),
                (scope, dict(selected, pack_data=PACK_DATA), 'g1 hoặc f1'),
            ]
            for malformed in ([], '', 'CAPong01', [1], [''], [' Map'], ['../Map'], ['Map-1'], ['맵01'],
                              ['A' * 65], ['CAPong01', 'CAPong01'], [f'Map{i}' for i in range(17)]):
                cases.append((scope, {key: malformed}, flag))
            for legacy in (None, 'g1', 'f1', 'f1-a'):
                cases.append((legacy, selected, flag))
            for pack_scope, options, message in cases:
                with (self.subTest(scope=pack_scope, options=options),
                      self.assertRaisesRegex(ValueError, message)):
                    build(self.config, self.binaries, self.out, target, pack_scope=pack_scope, **options)
                self.assertFalse(self.out.exists())

    def builder_argv(self, recipe, out, *options):
        return ['rust_service_package.py', '--base', str(recipe), '--binary-dir', str(self.binaries),
                '--out', str(out), '--target', 'x86_64-unknown-linux-gnu', *options]

    def test_builder_cli_rejects_conflicting_pack_options(self):
        recipe = self.root / 'recipe.json'
        recipe.write_text(json.dumps(self.config), encoding='utf-8')
        out = self.root / 'local' / 'package'
        out.parent.mkdir()
        for adapter, other in (('f2', 'f3'), ('f3', 'f2')):
            scope, flag = f'{adapter}-optin', f'--{adapter}-map'
            opt_in = ('--pack-scope', scope, flag, 'CAPong01')
            cases = [
                (('--pack-scope', scope), flag),
                (('--pack-scope', scope, flag, ''), flag),
                ((*opt_in, f'--{other}-map', 'CAPong02'), f'--{other}-map'),
                ((*opt_in, '--pack-data', str(PACK_DATA)), '--pack-data'),
                ((*opt_in, '--pack-data', str(PACK_DATA), '--no-pack-data'), '--pack-data'),
                ((*opt_in, '--legacy-fixtures'), '--legacy-fixtures'),
            ]
            cases.extend((('--pack-scope', legacy, flag, 'CAPong01'), flag) for legacy in ('g1', 'f1', 'f1-a'))
            for options, message in cases:
                with (self.subTest(options=options), patch.object(builder, 'ROOT', self.root),
                      patch.object(sys, 'argv', self.builder_argv(recipe, out, *options)),
                      redirect_stderr(StringIO()) as stderr, self.assertRaises(SystemExit) as error):
                    builder.main()
                self.assertIn(error.exception.code, (1, 2))
                self.assertIn(message, stderr.getvalue().splitlines()[-1])
                self.assertFalse(out.exists())

    def invoke_pack_runtime(self, manifest, command, state, *options):
        entry = self.out / 'kr-package.py'
        if manifest is None:
            expected = digest(self.out / 'manifest.json')
            verify_context = nullcontext()
        else:
            expected = 'a' * 64
            verify_context = patch.object(runtime, 'verify', return_value=dict(manifest, target='x86_64-unknown-linux-gnu'))
        argv = [str(entry), command, '--manifest-sha256', expected, '--state', str(state), *options]
        output = StringIO()
        with (patch.object(runtime, '__file__', str(entry)),
              verify_context,
              patch.object(sys, 'argv', argv), patch.object(sys, 'dont_write_bytecode'),
              patch.object(runtime.os, 'chdir'), patch.object(runtime.os, 'execv') as execv,
              redirect_stdout(output)):
            runtime.main()
        if command == 'run':
            execv.assert_called_once()
        return output.getvalue()

    def test_opt_in_pack_maps_keep_order_through_export_and_run(self):
        from test_kr_pack_install import fake_recipe

        game, cache = self.root / 'game', self.root / 'cache'
        game.mkdir()
        recipe = self.root / 'recipe.json'
        recipe.write_text(json.dumps(fake_recipe(self.root / 'wire')), encoding='utf-8')
        local = self.root / 'local'
        local.mkdir()
        for directory in ('tools', 'server', 'docs'):
            (self.root / directory).symlink_to(builder.ROOT / directory, target_is_directory=True)
        for adapter, names in [('f2', ['CAPong02', 'CAPong01']), ('f3', ['Tutorial01', 'MonSeal01'])]:
            scope = f'{adapter}-optin'
            other = 'f3' if adapter == 'f2' else 'f2'
            self.out = local / scope
            options = ['--pack-scope', scope, *(arg for name in names for arg in (f'--{adapter}-map', name))]
            with (patch.object(builder, 'ROOT', self.root),
                  patch.object(sys, 'argv', self.builder_argv(recipe, self.out, *options)),
                  redirect_stdout(StringIO()) as stdout):
                builder.main()
            expected = json.loads(stdout.getvalue())['manifest_sha256']
            manifest = verify(self.out, expected)
            self.assertEqual(manifest['pack_scope'], scope)
            self.assertEqual(manifest[f'{adapter}_maps'], names)
            self.assertNotIn(f'{other}_maps', manifest)
            self.assertNotIn('pack_data', manifest)
            entry = self.out / 'kr-package.py'
            command = [sys.executable, '-E', str(entry), 'export', '--manifest-sha256', expected,
                       '--state', str(self.root / f'{adapter}-standalone'), '--game-dir', str(game)]
            result = subprocess.run(command, cwd=self.root, capture_output=True, text=True)
            # The standalone exporter must pass catalog checks inside the bundle before checking the game pack.
            self.assertEqual(result.returncode, 1, result.stderr)
            self.assertIn('Thiếu pack KR 00967', result.stderr)
            for source in ('cli', 'manifest'):
                with self.subTest(scope=scope, source=source):
                    state = self.root / f'{adapter}-{source}'
                    state.mkdir(mode=0o700)
                    (state / 'accounts.sqlite').touch()
                    tls = state / 'prelogin_tls-identity'
                    tls.mkdir(mode=0o700)
                    (tls / 'certificate.der').write_bytes(b'certificate fixture')
                    manifest = None
                    options = ['--game-dir', str(game)]
                    if source == 'cli':
                        manifest = {'pack_scope': scope}
                        for name in names:
                            options.extend([f'--{adapter}-map', name])
                    with (patch('kr_pack_export.export', return_value=(cache, False)) as export,
                          patch('kr_pack_config.configure') as configure):
                        result = self.invoke_pack_runtime(manifest, 'export', state, *options)
                        self.assertEqual(json.loads(result), {'cache': str(cache), 'reused': False})
                        configure.assert_not_called()
                        self.invoke_pack_runtime(manifest, 'run', state, *options)
                    self.assertEqual(export.call_count, 2)
                    for invocation in export.call_args_list:
                        self.assertEqual(invocation.args, (game, state / 'pack-cache'))
                        self.assertEqual(invocation.kwargs, {'maps': names, adapter: True})
                    configure.assert_called_once()
                    self.assertEqual(configure.call_args.args[1:3], (self.out, cache))
                    self.assertEqual(configure.call_args.args[4:], (names, scope))

    def test_pack_scope_and_opt_in_map_validation_fail_before_export(self):
        state = self.root / 'invalid-state'
        cases = [('not-a-scope', {}, [], 'Phạm vi pack'), ([], {}, [], 'Phạm vi pack')]
        for adapter, other in [('f2', 'f3'), ('f3', 'f2')]:
            scope, key, flag = f'{adapter}-optin', f'{adapter}_maps', f'--{adapter}-map'
            cases.extend([
                (scope, {}, [], flag),
                (scope, {}, [flag, ''], flag),
                (scope, {}, [flag, 'CAPong01', f'--{other}-map', 'CAPong02'], f'--{other}-map'),
                (scope, {key: ['CAPong01'], f'{other}_maps': []}, [], f'{other}_maps'),
                (scope, {key: ['CAPong01']}, [flag, 'CAPong02'], 'Chỉ chọn'),
                (scope, {}, [flag, 'CAPong01', '--f1-map', 'Camp01'], '--f1-map'),
            ])
            for malformed in ([], None, 'CAPong01', [' '], [1]):
                cases.append((scope, {key: malformed}, [], key))
            for legacy in (None, 'g1', 'f1', 'f1-a'):
                cases.extend([(legacy, {}, [flag, 'CAPong01'], flag),
                              (legacy, {key: ['CAPong01']}, [], key)])
        with patch('kr_pack_export.export') as export:
            for scope, extra, flags, error_text in cases:
                with (self.subTest(scope=scope, manifest=extra, flags=flags),
                      redirect_stderr(StringIO()) as stderr, self.assertRaises(SystemExit) as error):
                    self.invoke_pack_runtime(dict(extra, pack_scope=scope), 'export', state,
                                             '--game-dir', str(self.root / 'game'), *flags)
                self.assertEqual(error.exception.code, 1)
                self.assertIn(error_text, stderr.getvalue())
            for command in ('verify', 'init-db'):
                with (self.subTest(command=command), redirect_stderr(StringIO()) as stderr,
                      self.assertRaises(SystemExit) as error):
                    self.invoke_pack_runtime({'pack_scope': 'f2-optin'}, command, state,
                                             '--f2-map', 'CAPong01')
                self.assertEqual(error.exception.code, 1)
                self.assertIn('--f2-map', stderr.getvalue())
            export.assert_not_called()
        self.assertFalse(state.exists())

    def test_legacy_runtime_export_and_run_match_baseline_bytes(self):
        import kr_pack_export
        from test_kr_pack_install import MAPS, fake_game, fake_recipe

        game = self.root / 'game'
        pins, seeds = fake_game(game)
        recipe = fake_recipe(self.out)
        recipe['diagnostic_two_lobby']['fixed_room']['map_choices'] = []
        (self.out / 'config.template.json').write_text(json.dumps(recipe), encoding='utf-8')
        # Fingerprints of the export cache, wire files and normalized config at 9d44fe3.
        baselines = {
            'g1': 'c01a9319f4a6182daa17e39ee1ff69a749a4253f3d6ff0da769e13a854b066b1',
            'f1': '62e2d827792b55f8dc32d50b8bdc163a830444fb00470242a10d43c27898f729',
            'f1-a': '65483129bc33a9e300fadc5cedd015699b255c246d550776f98a4e864c600da5',
        }
        with (patch.dict(kr_pack_export.PACK_SHA256, pins, clear=True),
              patch.dict(kr_pack_export.TERRAIN_SEEDS, seeds, clear=True),
              patch.multiple(kr_pack_export, MAPS=MAPS, G1=MAPS, F1=MAPS[::-1]),
              patch.object(runtime.time, 'time_ns', return_value=123)):
            for scope, baseline in baselines.items():
                with self.subTest(scope=scope):
                    state = self.root / scope
                    options = ['--game-dir', str(game)]
                    if scope == 'f1-a':
                        for name in MAPS[::-1]:
                            options.extend(['--f1-map', name])
                    manifest = {'pack_scope': scope}
                    result = self.invoke_pack_runtime(manifest, 'export', state, *options)
                    cache = Path(json.loads(result)['cache'])
                    (state / 'accounts.sqlite').touch()
                    self.invoke_pack_runtime(manifest, 'run', state, *options)
                    run = state / 'runs/123'
                    outputs = {p.relative_to(state).as_posix(): digest(p)
                               for folder in (cache, run / 'wire') for p in folder.iterdir()}
                    config = (run / 'config.json').read_bytes().replace(str(self.root).encode(), b'<temp>')
                    outputs['config.json'] = hashlib.sha256(config).hexdigest()
                    sha = hashlib.sha256(json.dumps(outputs, sort_keys=True).encode()).hexdigest()
                    self.assertEqual(sha, baseline)

    def test_f4_economy_and_web_shop_flags_are_opt_in_and_validated(self):
        self.package()
        default_cfg, _, _ = prepare(self.out, self.root / 'default-state')
        default_adm = json.loads(default_cfg.read_bytes())['diagnostic_two_lobby']['account_admission']
        for key in ('economy_profile_path', 'emit_lucci_135', 'project_own291_level', 'web_shop', 'unlock_all_catalog'):
            self.assertNotIn(key, default_adm)

        profile = self.root / 'f4-profile.json'
        profile.write_text('{"economy_policy":{"enabled":true},"shop_catalog":{"items":[]}}')
        std_cfg, _, _ = prepare(
            self.out,
            self.root / 'std-state',
            economy_profile=profile,
            emit_lucci_135=True,
            project_own291_level=True,
            web_shop_bind='127.0.0.1:8080',
        )
        std_adm = json.loads(std_cfg.read_bytes())['diagnostic_two_lobby']['account_admission']
        self.assertEqual(std_adm['economy_profile_path'], str(profile.resolve()))
        self.assertTrue(std_adm['persist_matches'])
        self.assertTrue(std_adm['emit_lucci_135'])
        self.assertTrue(std_adm['project_own291_level'])
        self.assertEqual(std_adm['web_shop']['bind'], '127.0.0.1:8080')
        self.assertFalse(std_adm['web_shop']['unlock_all'])
        self.assertEqual(std_adm['web_shop']['allowed_peers'], ['127.0.0.1'])

        friends_pkg = self.root / 'friends-pkg'
        friends_cfg_in = copy.deepcopy(self.config)
        two = friends_cfg_in['diagnostic_two_lobby']
        two['actors'] = [{'identity': 1, 'peer': '192.168.0.2', 'reported_login_ipv4': '192.168.0.2'}]
        two['account_admission'].update(
            patched_949x_names=True,
            public_launcher={'certificate': '@INSTALL@'},
            prelogin_tls={'certificate': '@INSTALL@'},
        )
        build(friends_cfg_in, self.binaries, friends_pkg, 'x86_64-unknown-linux-gnu')
        friends_state = self.root / 'friends-f4-state'
        friends_state.mkdir(mode=0o700)
        identity_dir = friends_state / 'prelogin_tls-identity'
        identity_dir.mkdir(mode=0o700)
        (identity_dir / 'certificate.der').write_bytes(b'certificate fixture')
        o1_cfg, _, _ = prepare(
            friends_pkg,
            friends_state,
            friends=True,
            economy_profile=profile,
            web_shop_bind='127.0.0.1:8081',
        )
        o1_adm = json.loads(o1_cfg.read_bytes())['diagnostic_two_lobby']['account_admission']
        self.assertTrue(o1_adm['unlock_all_catalog'])
        self.assertTrue(o1_adm['web_shop']['unlock_all'])
        self.assertEqual(o1_adm['web_shop']['allowed_peers'], ['127.0.0.1'])

        for bad_bind in ('0.0.0.0:8080', '127.0.0.1:0', 'not-a-host'):
            with self.subTest(bad_bind=bad_bind):
                with self.assertRaisesRegex(ValueError, 'web_shop'):
                    prepare(self.out, self.root / 'bad-bind', web_shop_bind=bad_bind)

        with self.assertRaisesRegex(ValueError, 'Hồ sơ kinh tế'):
            prepare(self.out, self.root / 'missing-prof', economy_profile=self.root / 'missing.json')
        symlink_profile = self.root / 'link.json'
        symlink_profile.symlink_to(profile)
        with self.assertRaisesRegex(ValueError, 'Hồ sơ kinh tế'):
            prepare(self.out, self.root / 'sym-prof', economy_profile=symlink_profile)

        fixed_pkg = self.root / 'fixed-pkg'
        build(copy.deepcopy(self.config), self.binaries, fixed_pkg, 'x86_64-unknown-linux-gnu')
        tmpl_path = fixed_pkg / 'config.template.json'
        fixed_cfg = json.loads(tmpl_path.read_bytes())
        fixed_cfg['diagnostic_two_lobby']['account_admission']['profile_mode'] = 'fixed'
        tmpl_path.chmod(0o600)
        tmpl_path.write_text(json.dumps(fixed_cfg))
        with self.assertRaisesRegex(ValueError, 'database_zero'):
            prepare(fixed_pkg, self.root / 'fixed-state', emit_lucci_135=True)
        with self.assertRaisesRegex(ValueError, 'database_zero'):
            prepare(fixed_pkg, self.root / 'fixed-state-lvl', project_own291_level=True)
        with self.assertRaisesRegex(ValueError, 'database_zero'):
            prepare(fixed_pkg, self.root / 'fixed-state-ws', web_shop_bind='127.0.0.1:8080')
        with self.assertRaisesRegex(ValueError, 'database_zero'):
            prepare(fixed_pkg, self.root / 'fixed-state-ua', unlock_all=True)

        ua_cfg, _, _ = prepare(
            self.out,
            self.root / 'ua-state',
            economy_profile=profile,
            unlock_all=True,
            web_shop_bind='127.0.0.1:8082',
        )
        ua_adm = json.loads(ua_cfg.read_bytes())['diagnostic_two_lobby']['account_admission']
        self.assertTrue(ua_adm['unlock_all_catalog'])
        self.assertTrue(ua_adm['web_shop']['unlock_all'])
        self.assertEqual(ua_adm['web_shop']['bind'], '127.0.0.1:8082')

    def test_f3_f4_f6_policy_flags_are_opt_in_and_fail_closed(self):
        profiles = {
            'f4_extended_policy': Path(__file__).resolve().parents[1] / 'server/profiles/f4-extended-disabled.json',
            'f3_pve_policy': Path(__file__).resolve().parents[1] / 'server/profiles/f3-pve-disabled.json',
            'f6_events_policy': Path(__file__).resolve().parents[1] / 'server/profiles/f6-events-disabled.json',
            'f5_social_policy': Path(__file__).resolve().parents[1] / 'server/profiles/f5-social-chat-disabled.json',
            'f7_custom_map_policy': Path(__file__).resolve().parents[1] / 'server/profiles/f7-custom-map-disabled.json',
        }
        self.package()
        default_cfg, _, _ = prepare(self.out, self.root / 'policy-default')
        default_admission = json.loads(default_cfg.read_bytes())['diagnostic_two_lobby']['account_admission']
        self.assertEqual(
            set(default_admission),
            set(self.config['diagnostic_two_lobby']['account_admission']) | {'database', 'launch_socket'},
        )
        expected = json.loads((self.out / 'config.template.json').read_bytes())
        expected_admission = expected['diagnostic_two_lobby']['account_admission']
        expected_admission.update(
            database=str(self.root / 'policy-default' / 'accounts.sqlite'),
            launch_socket=str(self.root / 'policy-default' / 'launch.sock'),
        )
        expected_bytes = json.dumps(expected, ensure_ascii=False, separators=(',', ':')) + '\n'
        self.assertEqual(default_cfg.read_bytes(), expected_bytes.encode())
        configured_cfg, _, _ = prepare(
            self.out, self.root / 'policy-configured', **profiles
        )
        configured_admission = json.loads(configured_cfg.read_bytes())['diagnostic_two_lobby']['account_admission']
        for key, path in profiles.items():
            self.assertEqual(configured_admission[key], str(path.resolve()))

        missing = self.root / 'missing-policy.json'
        directory = self.root / 'policy-directory'
        directory.mkdir()
        symlink = self.root / 'policy-link.json'
        symlink.symlink_to(profiles['f3_pve_policy'])
        invalid = self.root / 'invalid-policy.json'
        invalid.write_text('{not-json')
        non_object = self.root / 'array-policy.json'
        non_object.write_text('[]')
        parent = self.root / 'symlink-parent'
        parent.symlink_to(profiles['f3_pve_policy'].parent, target_is_directory=True)
        cases = [missing, directory, symlink, invalid, non_object, parent / 'f3-pve-disabled.json']
        for policy_key in profiles:
            for bad in cases:
                with self.subTest(policy=policy_key, path=bad):
                    with self.assertRaisesRegex(ValueError, 'policy|Hồ sơ'):
                        prepare(self.out, self.root / f'bad-{policy_key}-{len(str(bad))}', **{policy_key: bad})

    def test_policy_profiles_work_for_friends_and_s1_preparation(self):
        profiles = {
            'f4_extended_policy': Path(__file__).resolve().parents[1] / 'server/profiles/f4-extended-disabled.json',
            'f3_pve_policy': Path(__file__).resolve().parents[1] / 'server/profiles/f3-pve-disabled.json',
            'f6_events_policy': Path(__file__).resolve().parents[1] / 'server/profiles/f6-events-disabled.json',
            'f5_social_policy': Path(__file__).resolve().parents[1] / 'server/profiles/f5-social-chat-disabled.json',
            'f7_custom_map_policy': Path(__file__).resolve().parents[1] / 'server/profiles/f7-custom-map-disabled.json',
        }
        friends_config = copy.deepcopy(self.config)
        admission = friends_config['diagnostic_two_lobby']['account_admission']
        friends_config['diagnostic_two_lobby']['actors'] = [{'identity': 1, 'peer': '192.168.0.2'}]
        admission.update(
            patched_949x_names=True,
            prelogin_tls={'certificate': '@INSTALL@'},
        )
        friends_pkg = self.root / 'policy-friends-pkg'
        build(friends_config, self.binaries, friends_pkg, 'x86_64-unknown-linux-gnu')
        friends_state = self.root / 'policy-friends-state'
        friends_state.mkdir(mode=0o700)
        identity = friends_state / 'prelogin_tls-identity'
        identity.mkdir(mode=0o700, parents=True)
        (identity / 'certificate.der').write_bytes(b'certificate fixture')
        friends_cfg, _, _ = prepare(friends_pkg, friends_state, friends=True, **profiles)
        friends_admission = json.loads(friends_cfg.read_bytes())['diagnostic_two_lobby']['account_admission']
        for key, path in profiles.items():
            self.assertEqual(friends_admission[key], str(path.resolve()))

        s1_config = copy.deepcopy(self.config)
        s1_config['diagnostic_two_lobby']['actors'] = [{'identity': 1, 'peer': '192.168.0.2'}]
        s1_config['diagnostic_two_lobby']['account_admission']['prelogin_tls'] = {
            'certificate': '@INSTALL@', 'max_connections': 8,
            'session_ttl_seconds': 300, 'session_idle_seconds': 120,
        }
        s1_pkg = self.root / 'policy-s1-pkg'
        build(s1_config, self.binaries, s1_pkg, 'x86_64-unknown-linux-gnu')
        s1_state = self.root / 'policy-s1-state'
        s1_state.mkdir(mode=0o700)
        s1_identity = s1_state / 'prelogin_tls-identity'
        s1_identity.mkdir(mode=0o700, parents=True)
        (s1_identity / 'certificate.der').write_bytes(b'certificate fixture')
        s1_cfg, _, _ = prepare(s1_pkg, s1_state, s1=True, **profiles)
        s1_admission = json.loads(s1_cfg.read_bytes())['diagnostic_two_lobby']['account_admission']
        for key, path in profiles.items():
            self.assertEqual(s1_admission[key], str(path.resolve()))

    def test_policy_flags_are_rejected_on_non_runtime_subcommands(self):
        state = self.root / 'unsupported-state'
        for command in ('init-db', 'migrate-db', 'export'):
            for flag in ('--f5-social-policy', '--f7-custom-map-policy'):
                with self.subTest(command=command, flag=flag):
                    argv = [
                        'rust_package_runtime.py', command, '--manifest-sha256', 'a' * 64,
                        '--state', str(state), flag, str(self.root / 'policy.json'),
                    ]
                    with patch.object(runtime, 'verify', return_value={'target': 'x86_64-unknown-linux-gnu'}), \
                            patch.object(sys, 'argv', argv):
                        with self.assertRaises(SystemExit) as error:
                            runtime.main()
                    self.assertEqual(error.exception.code, 1)


if __name__ == '__main__':
    unittest.main()
