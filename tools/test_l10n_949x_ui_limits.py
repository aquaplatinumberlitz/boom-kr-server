"""Kiểm giới hạn byte 949x ở cả hai nguồn chuỗi UI của bộ dựng."""
import struct
import unittest

import l10n_949x as codec


def pair(key, value):
    return struct.pack('<H', 1) + codec.packed(key.encode()) + codec.packed(value.encode())


class UITextLimitTests(unittest.TestCase):
    def test_room_title_limits_follow_encoded_bytes_in_ssi(self):
        key = 'DefaultRoomTitle2'
        raw = codec.conv_pair(pair(key, 'ệ' * 11), {})
        _, at = codec.lstr(raw, 2)
        value, _ = codec.lstr(raw, at)
        self.assertEqual(len(value), 22)
        self.assertEqual(codec.decode(value), 'ệ' * 11)
        with self.assertRaisesRegex(ValueError, 'DefaultRoomTitle2.*23.*22'):
            codec.conv_pair(pair(key, 'ệ' * 11 + 'a'), {})

    def test_label_counts_display_bytes_after_unescaping_client_string_xml(self):
        fragment = '<Elem><key>ChatToAll</key><string>{}</string></Elem>'
        codec.check_client_string_ui(fragment.format('&lt;ệệệệệệ&gt;'))
        with self.assertRaisesRegex(ValueError, 'ChatToAll.*15.*14'):
            codec.check_client_string_ui(fragment.format('&lt;ệệệệệệa&gt;'))

    def test_bounded_unsupported_text_fails_without_replacement(self):
        with self.assertRaisesRegex(ValueError, 'ChatToOurTeam.*ngoài 949x'):
            codec.conv_pair(pair('ChatToOurTeam', '😀'), {})
        # Other strings retain the existing replacement policy.
        stats = {}
        codec.conv_pair(pair('Unrelated', '😀'), stats)
        self.assertEqual(stats['unmapped'], {'😀': 1})
