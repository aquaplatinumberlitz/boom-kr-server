"""Independent examples for research helpers; these do not test the game."""
import unittest

import l10n_utf8_codec as codec


class Utf8ResearchTests(unittest.TestCase):
    def test_known_multilingual_boundaries_and_limits(self):
        data = 'A한Việt😀'.encode('utf-8')
        self.assertEqual(codec.boundaries(data), (0, 1, 4, 5, 6, 9, 10, 14))
        self.assertEqual(codec.next_boundary(data, 6), 9)
        self.assertEqual(codec.previous_boundary(data, 9), 6)
        self.assertEqual(codec.next_boundary(data, 14), 14)
        self.assertEqual(codec.previous_boundary(data, 0), 0)
        self.assertEqual(codec.truncate(data, 8), 'A한Vi'.encode())
        self.assertEqual(codec.truncate(data, 13), 'A한Việt'.encode())
        self.assertEqual(codec.truncate(data, 14), data)
        with self.assertRaises(ValueError):
            codec.next_boundary(data, 2)
        with self.assertRaises(ValueError):
            codec.truncate(data, -1)

    def test_utf16_pairs_and_explicit_nul(self):
        data = '가\x00ệ😀'.encode('utf-8')
        self.assertEqual(codec.to_utf16_units(data), (0xac00, 0, 0x1ec7, 0xd83d, 0xde00))
        self.assertEqual(codec.from_utf16_units((0xac00, 0, 0x1ec7, 0xd83d, 0xde00)), data)
        with self.assertRaises(UnicodeDecodeError):
            codec.from_utf16_units((0xd83d,))
        with self.assertRaises(ValueError):
            codec.from_utf16_units((0x10000,))

    def test_combining_mark_is_a_separate_scalar(self):
        data = 'a\u0301'.encode('utf-8')
        self.assertEqual(codec.boundaries(data), (0, 1, 3))
        self.assertEqual(codec.truncate(data, 1), b'a')

    def test_reject_invalid_utf8_before_cutting(self):
        for data in (b'\x80', b'\xc0\xaf', b'\xed\xa0\x80', b'\xf4\x90\x80\x80', b'\xe1\xbb'):
            with self.subTest(data=data), self.assertRaises(UnicodeDecodeError):
                codec.truncate(data, 0)

    def test_width_uses_external_scalar_metrics(self):
        advances = {ord('A'): 5, ord('한'): 11, ord('ệ'): 7, 0x1f600: 13}
        self.assertEqual(codec.measured_width('A한ệ😀'.encode(), advances.__getitem__), 36)
        with self.assertRaises(ValueError):
            codec.measured_width(b'A', lambda _codepoint: -1)


if __name__ == '__main__':
    unittest.main()
