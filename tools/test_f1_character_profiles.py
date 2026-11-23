"""CI-safe contract checks for the complete F1-c character roster."""
import unittest

from kr_pack_config import enabled_characters
from kr_pack_export import F1_CHARACTERS
from server import probe_codec as codec


class F1CharacterProfileTests(unittest.TestCase):
    def setUp(self):
        self.keys = sorted(F1_CHARACTERS)
        self.stats = {
            str(key): {
                'capacity': 1 + key % 3,
                'maximum_capacity': 8,
                'power': 1,
                'maximum_power': 9,
                'speed': 4,
                'maximum_speed': 9,
            }
            for key in self.keys
        }
        self.stats['0'] = {
            'capacity': 1, 'maximum_capacity': 10, 'power': 1,
            'maximum_power': 7, 'speed': 5, 'maximum_speed': 7,
        }
        self.stats['1'] = {
            'capacity': 2, 'maximum_capacity': 7, 'power': 1,
            'maximum_power': 9, 'speed': 4, 'maximum_speed': 8,
        }

    def test_export_scope_is_exactly_31_keys_and_config_unlocks_them(self):
        self.assertEqual(len(F1_CHARACTERS), 31)
        self.assertEqual(enabled_characters(self.stats), self.keys)
        self.assertEqual(enabled_characters(self.stats, uses_terrain=True), self.keys)
        self.assertEqual(
            set(self.keys) - set(enabled_characters({**self.stats, '3003': {
                **self.stats['3003'], 'maximum_power': 10
            }}, uses_terrain=True)),
            {3003},
        )
        self.assertEqual({4019, 4025, 4026, 4000} & F1_CHARACTERS, set())

    def test_character92_fixture_round_trips_every_key_without_truncation(self):
        header = bytes(37)
        for key in self.keys:
            body = codec.pack_fields([0, key], [1, 2])
            frame = codec.encode(header, 92, body, 43)
            self.assertEqual(codec.frame_body(frame, 43), body)
            self.assertEqual(codec.read_fields(body, [1, 2]), [0, key])


if __name__ == '__main__':
    unittest.main()
