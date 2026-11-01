import unittest

from server import probe_codec as codec


class ProbeCodecTests(unittest.TestCase):
    def test_fragmented_and_coalesced_stream_frames(self):
        first = codec.encode(bytes(37), 118, bytes(range(22)), 40)
        second = codec.encode(bytes(37), 99, b'abc', 43)
        buffer = bytearray(first[:20])
        self.assertIsNone(codec.pop_frame(buffer))
        buffer.extend(first[20:] + second)
        self.assertEqual(codec.pop_frame(buffer), first)
        self.assertEqual(codec.pop_frame(buffer), second)
        self.assertEqual(buffer, b'')

    def test_bad_checksum_is_rejected_before_payload_dispatch(self):
        frame = codec.encode(bytes(37), 118, bytes(range(22)), 40)
        damaged = frame[:-1] + bytes([frame[-1] ^ 1])
        with self.assertRaises(ValueError):
            codec.frame_body(damaged, 40)

    def test_oversized_stream_does_not_wait_for_body(self):
        header = bytearray(38)
        header[13:17] = (65536).to_bytes(4, 'little')
        with self.assertRaises(ValueError):
            codec.pop_frame(header)

    def test_fields_cannot_silently_truncate_identity_or_coordinates(self):
        for values, widths in [([256], [1]), ([-1], [4]), ([True], [1]), ([1, 2], [4])]:
            with self.subTest(values=values), self.assertRaises(ValueError):
                codec.pack_fields(values, widths)
