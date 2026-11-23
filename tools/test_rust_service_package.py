"""Kiểm integrity/đường dẫn gói; fake ELF ở đây không phải chứng nhận runtime."""
import copy
import json
from pathlib import Path
import stat
import shutil
import struct
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parent))
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
        for name in ['kr-service', 'kr-admin', 'kr-companion']:
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
        for name in ['kr-launch-identity', 'kr-load']:
            (self.binaries / name).write_bytes((self.binaries / 'kr-admin').read_bytes())
        expected = build(fake_recipe(self.root / 'wire'), self.binaries, self.out, 'x86_64-unknown-linux-gnu',
                         pack_scope='g1', pack_data=PACK_DATA)
        manifest = verify(self.out, expected)
        export = json.loads((PACK_DATA / 'manifest.json').read_bytes())
        self.assertEqual((manifest['pack_data'], manifest['pack_data_identity']), ('pack-data', export['identity']))
        rows = {row['path']: row for row in manifest['files'] if row['path'].startswith('pack-data/')}
        self.assertEqual(set(rows), {'pack-data/' + path.name for path in PACK_DATA.iterdir()})
        for path in PACK_DATA.iterdir():
            self.assertEqual(rows['pack-data/' + path.name]['sha256'], digest(path))


if __name__ == '__main__':
    unittest.main()
