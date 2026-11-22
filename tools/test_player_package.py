"""Kiểm prefix/idx/hash của gói; dữ liệu nhỏ tự dựng không là oracle native."""
import json
from pathlib import Path
import os
import struct
import tempfile
import unittest

from player_package import KR_PORTS, ROOT, build, digest, preflight, windows_runtime
from rust_package_runtime import friends_pack


def index(offset, size):
    name = b'owned-test'
    raw = struct.pack('<IIII', 0, 0, 1, len(name)) + name + struct.pack('<IIII', offset, size, 0, 0)
    raw += bytes(-len(raw) % 4)
    words = struct.unpack('<' + 'I' * (len(raw) // 4), raw)
    return struct.pack('<' + 'I' * len(words), 0x31415926, *(a ^ b for a, b in zip(words, words[1:])))


class PackageTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.no_art = self.root / 'absent.png'
        self.base = self.root / 'base'
        self.base.mkdir()
        (self.base / 'fx.idd').write_bytes(b'KR')
        (self.base / 'fx.idx').write_bytes(index(0, 2))
        (self.base / 'support.dll').write_bytes(b'common')
        (self.base / 'nm.bik').write_bytes(b'video')
        (self.base / 'CA.exe').write_bytes(b'original-must-not-be-selected')
        (self.base / 'csysstr.ssd').write_bytes(b'KR text')
        self.recipe = dict(base=str(self.base), pack_stages=[], languages={})
        for language in ['vi', 'en', 'ko']:
            data = self.root / language
            data.mkdir()
            (data / 'CA.exe').write_bytes(('patched-949x-' + language).encode())
            (data / 'csysstr.ssd').write_bytes(language.encode())
            def pin(path):
                return dict(path=str(path), bytes=path.stat().st_size, sha256=digest(path))
            self.recipe['languages'][language] = dict(encoding='949x', data=str(data), fonts=str(data),
                video=pin(self.base / 'nm.bik'), executable=pin(data / 'CA.exe'), index_stage='base', images='pending')

    def test_friends_uses_original_prefix_and_original_index(self):
        stage = self.root / 'vi-pack'
        stage.mkdir()
        (stage / 'fx.idd').write_bytes(b'KRVI')
        (stage / 'fx.idx').write_bytes(index(2, 2))
        self.recipe['pack_stages'] = [dict(language='vi', root=str(stage))]
        self.recipe['languages']['vi']['index_stage'] = 'vi'
        output = self.root / 'output'
        build(self.recipe, output, self.no_art)
        manifest = digest(output / 'manifest.json')
        pins = {name: digest(self.base / name) for name in ('fx.idx', 'fx.idd')}
        chosen = friends_pack(output, manifest, pins)
        self.assertEqual(chosen['fx.idd'].read_bytes(), b'KR')
        self.assertEqual(chosen['fx.idx'].read_bytes(), index(0, 2))
        self.assertNotEqual(chosen['fx.idx'], output / 'payload/languages/vi/fx.idx')
        with self.assertRaisesRegex(ValueError, 'pack gốc'):
            friends_pack(output, manifest, pins | {'fx.idx': '0' * 64})

    def test_sequential_shared_pack_and_distinct_indices(self):
        for language, data, offset in [('vi', b'KRVI', 2), ('en', b'KRVIEN', 4)]:
            root = self.root / language
            (root / 'fx.idd').write_bytes(data)
            (root / 'fx.idx').write_bytes(index(offset, 2))
            self.recipe['pack_stages'].append(dict(language=language, root=str(root)))
            self.recipe['languages'][language]['index_stage'] = language
        output = self.root / 'output'
        summary = build(self.recipe, output, self.no_art)
        self.assertEqual(set(summary['guides']), {f'player/guide.{lang}.md' for lang in ('vi', 'en', 'ko')})
        for relative, pin in summary['guides'].items():
            packaged = output / relative
            self.assertEqual(packaged.read_bytes(), (ROOT / 'docs' / relative).read_bytes())
            self.assertEqual(digest(packaged), pin['sha256'])
        manifest = json.loads((output / 'manifest.json').read_text())
        pack = manifest['packs'][0]
        self.assertEqual(b''.join((output / part['path']).read_bytes() for part in pack['parts']), b'KRVIEN')
        self.assertEqual(len(pack['parts']), 3)
        for language, offset in [('vi', 2), ('en', 4), ('ko', 0)]:
            rows = manifest['languages'][language]['files']
            idx = next(row for row in rows if row['path'] == 'fx.idx')
            self.assertEqual((output / idx['payload']['path']).read_bytes(), index(offset, 2))
            exe = next(row for row in rows if row['path'] == 'CA.exe')
            self.assertIn(b'patched-949x', (output / exe['payload']['path']).read_bytes())
        self.assertNotIn('original-must-not', (output / 'manifest.json').read_text())

    def test_launcher_art_is_optional_and_pins_resized_jpeg(self):
        from PIL import Image

        no_art = self.root / 'without-art'
        bare = build(self.recipe, no_art, self.no_art)
        self.assertNotIn('launcher_art', json.loads((no_art / 'manifest.json').read_text()))

        source = self.root / 'launch.png'
        Image.new('RGB', (1450, 1085), (35, 145, 235)).save(source)
        output = self.root / 'with-art'
        summary = build(self.recipe, output, source)
        pin = json.loads((output / 'manifest.json').read_text())['launcher_art']
        staged = output / pin['path']
        self.assertEqual(pin['sha256'], digest(staged))
        self.assertEqual(pin['bytes'], staged.stat().st_size)
        self.assertEqual(summary['payload_bytes'] - bare['payload_bytes'], pin['bytes'])
        with Image.open(staged) as art:
            self.assertEqual(art.format, 'JPEG')
            self.assertEqual(art.size, (960, 718))

    def test_english_built_on_original_prefix_is_rejected(self):
        for language, data in [('vi', b'KRVI'), ('en', b'KREN')]:
            root = self.root / language
            (root / 'fx.idd').write_bytes(data)
            self.recipe['pack_stages'].append(dict(language=language, root=str(root)))
        with self.assertRaisesRegex(ValueError, 'prefix'):
            build(self.recipe, self.root / 'output', self.no_art)
        self.assertFalse((self.root / 'output/manifest.json').exists())

    def test_windows_bundle_pins_companion_and_rejects_a_non_pe_binary(self):
        def pin(path):
            return dict(path=str(path), bytes=path.stat().st_size, sha256=digest(path))
        program = self.root / 'first-party.exe'
        header = bytearray(64)
        header[:2] = b'MZ'
        struct.pack_into('<I', header, 60, 64)
        program.write_bytes(header + b'PE\0\0\x64\x86')
        cert = self.root / 'public.der'
        cert.write_bytes(b'test-only-public-certificate')
        output = self.root / 'windows'
        summary = build(self.recipe, output, self.no_art)
        recipe = dict(launcher=pin(program), companion=pin(cert), certificate=pin(cert),
                      prelogin_certificate=pin(cert), server='127.0.0.1:2000',
                      prelogin_server='127.0.0.1:2001', ports=[3838, 3938])
        with self.assertRaisesRegex(ValueError, 'PE x86-64'):
            windows_runtime(recipe, output, summary['manifest_sha256'])
        self.assertFalse((output / 'kr-player.exe').exists())
        recipe['companion'] = pin(program)
        recipe['release_url'] = 'https://drive.google.com.evil.invalid/drive/folders/example'
        with self.assertRaisesRegex(ValueError, 'HTTPS Google Drive'):
            windows_runtime(recipe, output, summary['manifest_sha256'])
        self.assertFalse((output / 'kr-player.exe').exists())
        recipe['release_url'] = 'https://drive.google.com/drive/folders/example?usp=sharing'
        recipe['server_name'] = 'Máy chủ thử'
        runtime = windows_runtime(recipe, output, summary['manifest_sha256'])
        config = json.loads((output / 'launcher.json').read_text())
        self.assertEqual(config['companion'], 'kr-prelogin.exe')
        self.assertEqual(config['udp_backend'], 'portable')
        self.assertIn('manifest.json', config['file_hashes'])
        self.assertIn('kr-prelogin.exe', config['file_hashes'])
        self.assertEqual(config['server_name'], recipe['server_name'])
        self.assertEqual(config['release_url'], recipe['release_url'])
        self.assertEqual(runtime['kr-prelogin.exe'], digest(program))
        self.assertEqual(config['manifest_sha256'], digest(output / 'manifest.json'))

    def test_wrong_executable_pin_and_unpatched_korean_are_rejected(self):
        self.recipe['languages']['ko']['encoding'] = 'cp949'
        with self.assertRaisesRegex(ValueError, '949x'):
            build(self.recipe, self.root / 'output', self.no_art)
        self.recipe['languages']['ko']['encoding'] = '949x'
        self.recipe['languages']['vi']['executable']['sha256'] = '0' * 64
        with self.assertRaisesRegex(ValueError, 'pin'):
            build(self.recipe, self.root / 'output2', self.no_art)

    def test_o1_preflight_checks_hashes_ports_and_components(self):
        output = self.root / 'preflight'
        build(self.recipe, output, self.no_art)
        for name in ('kr-player.exe', 'kr-prelogin.exe'):
            (output / name).write_bytes(name.encode())
        server = output / 'friends-server'
        (server / 'bin').mkdir(parents=True)
        (server / 'bin' / 'kr-service.exe').write_bytes(b'service')
        (server / 'kr_pack_export.py').write_bytes(b'# exporter\n')
        service_rows = []
        for path in (server / 'bin' / 'kr-service.exe', server / 'kr_pack_export.py'):
            service_rows.append(dict(path=path.relative_to(server).as_posix(),
                                     mode=os.stat(path).st_mode & 0o777,
                                     bytes=path.stat().st_size, sha256=digest(path)))
        service_manifest = dict(schema='kre-service-package-v1', target='x86_64-pc-windows-gnu',
                                files=service_rows, pack_scope='g1')
        (server / 'manifest.json').write_text(json.dumps(service_manifest) + '\n')
        python = output / 'friends-python'
        python.mkdir()
        (python / 'python.exe').write_bytes(b'python')
        launcher = dict(ports=list(KR_PORTS), companion='kr-prelogin.exe',
                        manifest_sha256=digest(output / 'manifest.json'),
                        friends=dict(package='friends-server', manifest_sha256=digest(server / 'manifest.json'),
                                      python='friends-python/python.exe'))
        launcher['file_hashes'] = {
            path.relative_to(output).as_posix(): dict(bytes=path.stat().st_size, sha256=digest(path))
            for path in output.rglob('*') if path.is_file()
        }
        (output / 'launcher.json').write_text(json.dumps(launcher) + '\n')
        result = preflight(output)
        self.assertEqual(result['ports'], len(KR_PORTS))

        tampered = output / 'friends-server' / 'kr_pack_export.py'
        tampered.write_bytes(b'changed')
        with self.assertRaisesRegex(ValueError, 'hash'):
            preflight(output)

    def test_o1_preflight_rejects_incomplete_recipe_and_nonempty_calog(self):
        output = self.root / 'preflight-fail'
        output.mkdir()
        (output / 'manifest.json').write_text(json.dumps({'schema': 'kre-player-package-v1'}))
        (output / 'launcher.json').write_text(json.dumps({'manifest_sha256': digest(output / 'manifest.json')}))
        (output / 'CALog.log').write_bytes(b'stale')
        with self.assertRaisesRegex(ValueError, 'CALog|file_hashes|cổng'):
            preflight(output)


if __name__ == '__main__':
    unittest.main()
