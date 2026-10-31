import unittest

from server.udp_round_keys import UdpRoundKeys


class UdpRoundKeysTests(unittest.TestCase):
    def test_first_two_values_match_measured_policy(self):
        keys = UdpRoundKeys(12345)
        self.assertEqual((keys.next_key(), keys.next_key()), (12345, 12346))

    def test_low_bytes_are_not_reused_including_u32_wrap(self):
        keys = UdpRoundKeys(0xFFFFFF80)
        values = [keys.next_key() for _ in range(256)]
        self.assertEqual(len(set(value & 0xFF for value in values)), 256)
        self.assertTrue(all(0 <= value <= 0xFFFFFFFF for value in values))
        self.assertIn(0, values)
        self.assertIn(0xFFFFFFFF, values)

    def test_exhaustion_does_not_wrap_or_reset_on_retry(self):
        keys = UdpRoundKeys(12345)
        for _ in range(256):
            keys.next_key()
        for _ in range(3):
            with self.assertRaises(RuntimeError):
                keys.next_key()
        self.assertEqual(keys.issued_count, 256)

    def test_rejects_values_that_cannot_be_encoded_as_a_u32(self):
        for value in (-1, 0x100000000, True, 1.0, '12345', None):
            with self.subTest(value=value), self.assertRaises(ValueError):
                UdpRoundKeys(value)
