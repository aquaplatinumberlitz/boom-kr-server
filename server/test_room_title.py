"""Policy regression only; independent native/runtime evidence is kept under local/."""
import struct
import tempfile
from pathlib import Path
from types import SimpleNamespace
import unittest

from server import probe_codec as codec
from server import room_title as titles
from server.probe_service import ProbeService


def create_body(title, password=b''):
    # Fixture builder, not a native compatibility oracle.
    return (codec.pack_fields([len(title)], [2]) + titles._encode_bytes(title)
            + codec.pack_fields([len(password)], [2]) + titles._encode_bytes(password)
            + codec.pack_fields(list(range(12)) + [0, 1, 2, 3], [4] * 12 + [1] * 4))


def listing_body(title=b'KRRoom'):
    row = struct.pack('<HI', 1, 17) + struct.pack('<I', len(title)) + title + bytes(range(22))
    stream = b'\x18' + struct.pack('<I', len(row) + 1) + b'\0' + row
    return codec.pack_fields([1, len(stream)], [2, 4]) + titles._encode_bytes(stream)


def listing_row(body):
    stream = titles._decode_bytes(body[6:])
    return stream[6:]


class TitleCodecTests(unittest.TestCase):
    def test_title_lengths_korean_and_inline_heap_boundary(self):
        for title in [b'A' * 15, b'B' * 16, b'C' * 32, '매너겜 필수'.encode('cp949')]:
            with self.subTest(size=len(title)):
                self.assertEqual(titles.create_title(create_body(title)), title)

    def test_truncated_or_extended_request_password_and_bad_text_are_rejected(self):
        original = create_body(b'Room')
        for payload in [original[:-1], original + b'X', create_body(b'Room', b'pw'),
                        create_body(b'X' * 33), create_body(b''), create_body(b'x\0y'),
                        create_body(b'\xff')]:
            with self.subTest(size=len(payload)):
                with self.assertRaises(ValueError):
                    titles.create_title(payload)

    def test_row_identity_and_opaque_tail_remain_exact(self):
        original = listing_body()
        before = listing_row(original)
        name = '매너겜 필수'.encode('cp949')
        after = listing_row(titles.replace_listing_title(original, name))
        self.assertEqual(before[:6], after[:6])
        self.assertEqual(after[10:10 + len(name)], name)
        self.assertEqual(after[10 + len(name):], before[16:])
        self.assertEqual(struct.unpack_from('<I', after, 6)[0], len(name))
        self.assertEqual(titles.replace_listing_title(codec.pack_fields([0], [2]), name),
                         codec.pack_fields([0], [2]))

    def test_unsupported_listing_fails_instead_of_truncating_opaque_data(self):
        original = listing_body()
        for body in [original[:-1], original + b'X', codec.pack_fields([2], [2]) + original[2:],
                     codec.pack_fields([0], [2]) + b'X']:
            with self.assertRaises(ValueError):
                titles.replace_listing_title(body, b'Room')


class TitleServiceTests(unittest.TestCase):
    def setUp(self):
        folder = tempfile.TemporaryDirectory()
        self.addCleanup(folder.cleanup)
        self.out = Path(folder.name)
        self.bodies = {'roomlist_A': codec.pack_fields([0], [2]), 'roomlist_B': listing_body()}
        names = ['rank_Awin', 'promote_B', 'auth_C', 'roomlist_C', 'join_C',
                 'player_C', 'slot_C', 'guest_C', 'ready_C', 'ready', 'channel_list', 'create',
                 *self.bodies]
        self.sequences = {name: 49 if name.startswith('roomlist') else 55 for name in names}
        opcodes = {'roomlist_A': 91, 'roomlist_B': 91, 'create': 86, 'ready': 241, 'channel_list': 14}
        self.profile = SimpleNamespace(
            frames=dict.fromkeys(names),
            data={'items': [], 'initial_key': 12345,
                  'room_title': {'mode': 'create81_to_row91', 'max_bytes': 32},
                  'fixtures': {name: {'sequence': seq} for name, seq in self.sequences.items()}},
            body=lambda name: self.bodies.get(name, b'opaque'),
            raw=lambda name: codec.encode(bytes(37), opcodes.get(name, 0),
                                           self.bodies.get(name, b'opaque'), self.sequences[name]))
        self.service = ProbeService(self.profile, self.out, {})
        self.addCleanup(self.service.selector.close)
        self.sent = []
        self.service.send_raw = lambda event, frame: self.sent.append((event['role'], frame))
        self.a = {'role': 'A', 'global_ordinal': 4, 'reply_ordinal': 4}
        self.b = {'role': 'B', 'global_ordinal': 8, 'reply_ordinal': 4}

    def create(self, body):
        frame = codec.encode(bytes(37), 81, body, 82)
        self.service.receive_channel(self.a, frame, 82, body)

    def test_commit_only_after_ack_and_guest_initial_listing_uses_title(self):
        title = '매너겜 필수'.encode('cp949')
        self.create(create_body(title))
        self.service.receive_channel(self.b, codec.encode(bytes(37), 67, b'', 79), 79, b'')
        buf = bytearray(self.sent[-1][1])
        frames = []
        while (frame := codec.pop_frame(buf)) is not None:
            frames.append(frame)
        self.assertEqual([int.from_bytes(f[5:9], 'little') for f in frames], [241, 91, 14])
        row = listing_row(codec.frame_body(frames[1], 49))
        self.assertEqual(row[10:10 + len(title)], title)
        self.assertEqual(self.profile.body('roomlist_B'), listing_body())
        self.assertNotIn('s2c_next_seq', self.b)  # Static setup sequencing remains unchanged.
        self.assertEqual(len(self.service.state['room_title_updates']), 1)

    def test_failed_ack_keeps_old_title_and_no_commit_marker(self):
        self.service.room_title_bytes = b'Old'
        def fail(*_):
            raise OSError('test send failure')
        self.service.send_raw = fail
        with self.assertRaises(OSError):
            self.create(create_body(b'New'))
        self.assertEqual(self.service.room_title_bytes, b'Old')
        self.assertNotIn('room_title_updates', self.service.state)
        self.assertNotIn('create_response_sent', self.a)

    def test_bad_request_has_no_ack_or_membership_mutation(self):
        self.create(create_body(b'Room')[:-1])
        self.assertEqual(self.sent, [])
        self.assertIsNone(self.service.room_title_bytes)
        self.assertNotIn('create_response_sent', self.a)
        self.assertEqual(len(self.service.state['room_title_rejections']), 1)

    def test_dynamic_listing_uses_continuing_sequence(self):
        self.service.room_title_bytes = b'Promoted Room'
        self.b['s2c_next_seq'] = 103
        self.service.send(self.b, 'roomlist_B', 'title-after-leave')
        row = listing_row(codec.frame_body(self.sent[-1][1], 103))
        self.assertEqual(row[10:23], b'Promoted Room')
        self.assertEqual(self.b['s2c_next_seq'], 106)

    def test_mode_off_preserves_legacy_body_acceptance(self):
        self.service.room_title_mode = False
        self.create(b'legacy-test-placeholder')
        self.assertTrue(self.a['create_response_sent'])
        self.assertNotIn('room_title_updates', self.service.state)

    def test_bad_fixture_rejected_at_preflight(self):
        self.bodies['roomlist_B'] = listing_body()[:-1]
        with self.assertRaisesRegex(ValueError, 'length mismatch'):
            ProbeService(self.profile, self.out, {})


if __name__ == '__main__':
    unittest.main()
