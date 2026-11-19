"""Kiểm prefix/idx/hash của gói; dữ liệu nhỏ tự dựng không là oracle native."""
import json
from pathlib import Path
import struct
import tempfile
import unittest

from player_package import build, digest


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

    def test_sequential_shared_pack_and_distinct_indices(self):
        for language, data, offset in [('vi', b'KRVI', 2), ('en', b'KRVIEN', 4)]:
            root = self.root / language
            (root / 'fx.idd').write_bytes(data)
            (root / 'fx.idx').write_bytes(index(offset, 2))
            self.recipe['pack_stages'].append(dict(language=language, root=str(root)))
            self.recipe['languages'][language]['index_stage'] = language
        output = self.root / 'output'
        build(self.recipe, output)
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

    def test_english_built_on_original_prefix_is_rejected(self):
        for language, data in [('vi', b'KRVI'), ('en', b'KREN')]:
            root = self.root / language
            (root / 'fx.idd').write_bytes(data)
            self.recipe['pack_stages'].append(dict(language=language, root=str(root)))
        with self.assertRaisesRegex(ValueError, 'prefix'):
            build(self.recipe, self.root / 'output')
        self.assertFalse((self.root / 'output/manifest.json').exists())

    def test_wrong_executable_pin_and_unpatched_korean_are_rejected(self):
        self.recipe['languages']['ko']['encoding'] = 'cp949'
        with self.assertRaisesRegex(ValueError, '949x'):
            build(self.recipe, self.root / 'output')
        self.recipe['languages']['ko']['encoding'] = '949x'
        self.recipe['languages']['vi']['executable']['sha256'] = '0' * 64
        with self.assertRaisesRegex(ValueError, 'pin'):
            build(self.recipe, self.root / 'output2')


if __name__ == '__main__':
    unittest.main()
