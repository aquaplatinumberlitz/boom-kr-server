"""Compare the C research primitive against Python's strict Unicode codec."""
import ctypes
from pathlib import Path
import random
import subprocess
import tempfile
import unittest


class CoreTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temp = tempfile.TemporaryDirectory()
        source = Path(__file__).with_name('l10n_utf8_core.c')
        lib = Path(cls.temp.name) / 'codec.so'
        subprocess.run(['cc', '-std=c11', '-Wall', '-Wextra', '-Werror', '-shared',
                        '-fPIC', str(source), '-o', str(lib)], check=True)
        cls.lib = ctypes.CDLL(str(lib))
        p8 = ctypes.POINTER(ctypes.c_uint8)
        psz = ctypes.POINTER(ctypes.c_size_t)
        cls.lib.l10n_utf8_to_utf16.argtypes = [p8, ctypes.c_size_t,
            ctypes.POINTER(ctypes.c_uint16), ctypes.c_size_t, psz]
        cls.lib.l10n_utf16_to_utf8.argtypes = [ctypes.POINTER(ctypes.c_uint16),
            ctypes.c_size_t, p8, ctypes.c_size_t, psz]
        cls.lib.l10n_utf8_prefix.argtypes = [p8, ctypes.c_size_t, ctypes.c_size_t, psz]

    @classmethod
    def tearDownClass(cls):
        cls.temp.cleanup()

    def check_valid(self, text):
        raw = text.encode('utf-8')
        data = (ctypes.c_uint8 * len(raw)).from_buffer_copy(raw)
        expected = text.encode('utf-16-le')
        units = [int.from_bytes(expected[i:i + 2], 'little') for i in range(0, len(expected), 2)]
        dst = (ctypes.c_uint16 * (len(units) + 1))(*([0xbeef] * (len(units) + 1)))
        out = ctypes.c_size_t(12345)
        if units:
            self.assertEqual(self.lib.l10n_utf8_to_utf16(data, len(raw), dst, len(units)-1, ctypes.byref(out)), -2)
            self.assertEqual(list(dst), [0xbeef] * len(dst))
            self.assertEqual(out.value, 12345)
        self.assertEqual(self.lib.l10n_utf8_to_utf16(data, len(raw), dst, len(units), ctypes.byref(out)), 0)
        self.assertEqual(list(dst)[:out.value], units)
        self.assertEqual(dst[len(units)], 0xbeef)
        result = (ctypes.c_uint8 * (len(raw) + 1))(*([0xee] * (len(raw) + 1)))
        if raw:
            out.value = 12345
            self.assertEqual(self.lib.l10n_utf16_to_utf8(dst, len(units), result, len(raw)-1, ctypes.byref(out)), -2)
            self.assertEqual(list(result), [0xee] * len(result))
            self.assertEqual(out.value, 12345)
        self.assertEqual(self.lib.l10n_utf16_to_utf8(dst, len(units), result, len(raw), ctypes.byref(out)), 0)
        self.assertEqual(bytes(result[:out.value]), raw)
        self.assertEqual(result[len(raw)], 0xee)
        for limit in range(len(raw) + 2):
            expected_prefix = max(i for i in range(len(raw)+1)
                                  if i <= limit and self.is_valid(raw[:i]))
            self.assertEqual(self.lib.l10n_utf8_prefix(data, len(raw), limit, ctypes.byref(out)), 0)
            self.assertEqual(out.value, expected_prefix)

    @staticmethod
    def is_valid(raw):
        try:
            raw.decode('utf-8')
            return True
        except UnicodeDecodeError:
            return False

    def test_multilingual_and_scalar_boundaries(self):
        for text in ('', 'A한Việt😀', '\x00가ệ', 'a\u0301', '\u007f\u0080\u07ff\u0800\uffff\U0010ffff'):
            with self.subTest(text=text): self.check_valid(text)

    def test_invalid_input_is_atomic(self):
        samples = [b'\x80', b'\xc0\xaf', b'\xed\xa0\x80', b'\xf4\x90\x80\x80', b'A\xe1\xbb']
        for raw in samples:
            data = (ctypes.c_uint8 * len(raw)).from_buffer_copy(raw)
            dst = (ctypes.c_uint16 * 10)(*([0xbeef] * 10)); out = ctypes.c_size_t(12345)
            self.assertEqual(self.lib.l10n_utf8_to_utf16(data, len(raw), dst, 10, ctypes.byref(out)), -1)
            self.assertEqual(list(dst), [0xbeef] * 10)
            self.assertEqual(out.value, 12345)
            self.assertEqual(self.lib.l10n_utf8_prefix(data, len(raw), 1, ctypes.byref(out)), -1)
            self.assertEqual(out.value, 12345)

    def test_invalid_utf16_is_atomic(self):
        for units in ((0xd800,), (0xdc00,), (0xd800, 65), (65, 0xdc00)):
            data = (ctypes.c_uint16 * len(units))(*units)
            dst = (ctypes.c_uint8 * 20)(*([0xee] * 20)); out = ctypes.c_size_t(12345)
            self.assertEqual(self.lib.l10n_utf16_to_utf8(data, len(units), dst, 20, ctypes.byref(out)), -1)
            self.assertEqual(list(dst), [0xee] * 20)
            self.assertEqual(out.value, 12345)

    def test_deterministic_random_scalars(self):
        rng = random.Random(7289)
        for _ in range(100):
            chars = []
            for _ in range(rng.randrange(0, 20)):
                value = rng.randrange(0x110000)
                if not 0xd800 <= value <= 0xdfff: chars.append(chr(value))
            self.check_valid(''.join(chars))

    def test_deterministic_byte_sequences_against_python(self):
        rng = random.Random(64551)
        for _ in range(2000):
            raw = bytes(rng.randrange(256) for _ in range(rng.randrange(0, 9)))
            data = (ctypes.c_uint8 * len(raw)).from_buffer_copy(raw)
            dst = (ctypes.c_uint16 * 16)(*([0xbeef] * 16)); out = ctypes.c_size_t(12345)
            result = self.lib.l10n_utf8_to_utf16(data, len(raw), dst, 16, ctypes.byref(out))
            if self.is_valid(raw):
                expected = raw.decode('utf-8').encode('utf-16-le')
                units = [int.from_bytes(expected[i:i+2], 'little') for i in range(0, len(expected), 2)]
                self.assertEqual(result, 0)
                self.assertEqual(list(dst)[:out.value], units)
            else:
                self.assertEqual(result, -1)
                self.assertEqual(list(dst), [0xbeef] * 16)
                self.assertEqual(out.value, 12345)


if __name__ == '__main__':
    unittest.main()
