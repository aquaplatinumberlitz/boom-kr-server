import hashlib
import struct
import unittest
from unittest.mock import patch

from export_map_profile import classify, classify_rules, decode_map, load_tile_table, verify_seed


def string(value):
    value = value.encode("ascii") + b"\0"
    return struct.pack("<I", len(value)) + value


def sample():
    header = [1001, 2, 4, 2, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    return (struct.pack("<20i", *header) + struct.pack("<2i", 1, 0)
            + struct.pack("<i", 0) + struct.pack("<6i", 10, -1, -1, 11, 1932, -1)
            + struct.pack("<i", 1) + string("Bomb") + struct.pack("<i", 7)
            + struct.pack("<2i", 100, 1) + string("Shoe") + bytes([1, 0, 30])
            + struct.pack("<3i", 0, 0, 0))


class MapProfileTests(unittest.TestCase):
    def test_full_data(self):
        result = decode_map(sample())
        self.assertEqual(result["spawns"], [[1, 0]])
        self.assertEqual(result["cell_fields"], ["tile", "object", "overlay"])
        self.assertEqual(result["cells"], [[10, -1, -1], [11, 1932, -1]])
        self.assertEqual(result["drops"], [["Bomb", 7]])
        self.assertEqual(result["placements"], [dict(name="Shoe", x=1, y=0, probability=30)])
        self.assertFalse(result["scripted"])

    def test_unknown_values_are_not_guessed(self):
        baseline = [[10, -1], [11, 1932]]
        kinds, count, unknown = classify([[10, -1], [11, 1932], [12, 1932], [10, 999]], baseline)
        self.assertEqual(kinds, [0, 2, None, None])
        self.assertEqual(count, 2)
        self.assertEqual(unknown, {"tile": {12: 1}, "object": {999: 1}})

    def test_seed_compares_each_cell_and_rejects_mismatch(self):
        baseline = {"width": 2, "height": 1, "cells": [[10, -1], [11, 1932]]}
        kinds = [0, 2]
        seed = hashlib.sha256(bytes(kinds)).hexdigest()
        fixture = {"bomb_terrain": dict(width=2, height=1, kinds=kinds,
                                        seed_sha256=seed, map_sha256="map")}
        with patch("export_map_profile.SEED_SHA", seed), patch("export_map_profile.MAP_SHA", "map"):
            self.assertEqual(verify_seed(baseline, fixture), 2)
            baseline["cells"][1][1] = -1
            with self.assertRaisesRegex(ValueError, r"\(1, 0, 0, 2\)"):
                verify_seed(baseline, fixture)
            fixture["bomb_terrain"]["kinds"] = [2, 0]
            with self.assertRaisesRegex(ValueError, "hash"):
                verify_seed(baseline, fixture)

    def test_table_uses_object_order_and_joins_names(self):
        def record(name, offset):
            encoded = name.encode() + b"\0"
            return struct.pack("<I", len(encoded)) + encoded + struct.pack("<I", offset)

        table = struct.pack("<I", 2) + record("theme/box", 100) + record("theme/floor", 120)
        box = struct.pack("<18i", 2, *([0] * 17)).hex()
        floor = struct.pack("<18i", 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, -1, *([0] * 7)).hex()
        xml = (f'<TileInfo><Tile Name="theme/floor" Data="{floor}"/>'
               f'<Tile Name="theme/box" Data="{box}"/></TileInfo>').encode()

        class FakePack:
            def read(self, name):
                return {"ObjectTable/ObjectTable": table,
                        "TileInfo/TileInfoList": struct.pack("<I", len(xml)) + xml}[name]

        names, tiles = load_tile_table(FakePack())
        self.assertEqual(names, ["theme/box", "theme/floor"])
        self.assertEqual(classify_rules([[1, -1], [1, 0]], (names, tiles))[0], [0, 2])

    def test_rules_leave_special_and_invalid_cells_unknown(self):
        ground = (0, 0, 0, 0, 0, 0, 1, 1, 0, 0, -1) + (0,) * 7
        static = (1, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0) + (0,) * 7
        special = (2, 0, 0, 0, 0, 0, 5, 1, 0, 0, 0) + (0,) * 7
        names = ["floor", "house", "strong", "PinGround", "herb"]
        tiles = dict(zip(names, [ground, static, special, ground, (3,) + (0,) * 17]))
        kinds, count, _ = classify_rules([[0, -1], [0, 1], [0, 2], [3, -1], [0, 3],
                                          [0, 4], [-1, -1], [0, -1, 1], [99, -1]],
                                         (names, tiles))
        self.assertEqual(kinds, [0, 1, None, None, None, None, None, None, None])
        self.assertEqual(count, 7)

    def test_partial_map_rejected(self):
        with self.assertRaises(ValueError):
            decode_map(sample()[:-1])


if __name__ == "__main__":
    unittest.main()
