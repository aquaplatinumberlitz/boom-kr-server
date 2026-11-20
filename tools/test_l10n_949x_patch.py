"""Kiểm byte của hộp đăng nhập vi; không cần exe hay trình biên dịch PE."""
import struct
import unittest

from l10n_949x_patch import apply_checked, vi_login_dialog


class LoginDialogTests(unittest.TestCase):
    def test_only_the_two_login_calls_use_wide_strings(self):
        va = 0x2265000
        blob, patches = vi_login_dialog(va)
        sites = {0x10FA74E: 1, 0x13AE104: 21}
        self.assertEqual([patch[0] for patch in patches], list(sites))
        ansi = bytes.fromhex('68748f7801 68848f7801 6a00 ff154ca66e01')
        image = bytearray(b'!' + (ansi + b'!!') * 3)
        apply_checked(image, sites.__getitem__, patches)
        for offset in sites.values():
            instruction = image[offset:offset + 18]
            self.assertEqual(instruction[10:], bytes.fromhex('6a00 ff1578a66e01'))
            caption, text = [struct.unpack_from('<I', instruction, i)[0] - va for i in (1, 6)]
            self.assertEqual(instruction[0], 0x68)
            self.assertEqual(instruction[5], 0x68)
            self.assertEqual(blob[caption:text].decode('utf-16le'), 'Lỗi nghiêm trọng\0')
            self.assertEqual(blob[text:].decode('utf-16le'), 'Đăng nhập không hợp lệ!\0')
        self.assertEqual(image[41:59], ansi)

    def test_changed_call_is_rejected(self):
        _, patches = vi_login_dialog(0x2265000)
        image = bytearray(patches[0][1])
        image[-1] ^= 1
        with self.assertRaises(SystemExit):
            apply_checked(image, lambda _: 0, patches[:1])
