"""Kiểm chính sách item của adapter pack F1 và payload 582 được cấu hình."""
import struct
import unittest

from kr_pack_config import enable_floor_policies, floor582
from kr_pack_export import ITEMS, f1_floor, f1_terrain_rules
from rust_map_profile import blob


def placement(kind, x, y, probability=100):
    return dict(name=ITEMS.get(kind, kind), x=x, y=y, probability=probability)


class FloorTests(unittest.TestCase):
    def test_fixed_items_and_weighted_drops_share_only_the_measured_scope(self):
        terrain = dict(width=15, height=13, kinds=[0] * 195)
        terrain['kinds'][31] = 2
        fixed = [placement(1, 6, 2), placement(3, 5, 2), placement(4, 7, 2), placement(145, 8, 2)]
        refused = [placement(145, 9, 2), placement('Item/ItemDart', 10, 2),
                   placement(3, 11, 2, 50), placement(4, 1, 2), placement(4, 1, 1)]
        for kind in (0, 3, 4):
            profile = dict(name='ArbitraryMap', placement_header=100, spawns=[[1, 1]],
                           payload_sha256='0' * 64, placements=fixed + refused,
                           drops=[(ITEMS[145], 999), ('Item/ItemDart', 999), (ITEMS[kind], 1)])
            plan = f1_floor(profile, terrain)
            self.assertEqual(plan['fixed'], [[2, 6, 1], [2, 5, 3], [2, 7, 4], [2, 8, 145]])
            self.assertEqual(plan['hidden'], [[2, 1, kind]])
            self.assertEqual(plan['omitted_placements'], refused)
            packet = floor582(plan)
            raw, end, _ = blob(packet)
            self.assertEqual(end, len(packet))
            self.assertEqual(struct.unpack_from('<H', raw)[0], 4)
            self.assertEqual(list(struct.iter_unpack('<IBBI', raw[2:42])),
                             [(0, 2, 6, 1), (1, 2, 5, 3), (2, 2, 7, 4), (3, 2, 8, 145)])
            self.assertEqual(struct.unpack('<HIBBIH', raw[42:]), (1, 4, 2, 1, kind, 0))

    def test_encoder_rejects_unknown_and_out_of_scope_needle(self):
        for fixed, hidden in [([[2, 6, 6]], []), ([], [[2, 1, 145]]),
                              ([[2, 6, 145], [2, 7, 145]], [])]:
            with self.assertRaises(ValueError):
                floor582(dict(fixed=fixed, hidden=hidden))

    def test_floor_policies_follow_all_generated_catalog_entries(self):
        room = dict(item_policy='report_derived_basic', life_actions='rescue')
        enable_floor_policies(room, {0, 1, 2, 5})
        self.assertEqual(room, dict(item_policy='report_derived_basic', life_actions='rescue'))
        enable_floor_policies(room, {4})
        self.assertEqual(room, dict(item_policy='report_derived_shoe_punch', life_actions='rescue'))
        enable_floor_policies(room, {145})
        self.assertEqual(room, dict(item_policy='report_derived_shoe_punch', life_actions='needle_rescue'))


class TerrainAdapterTests(unittest.TestCase):
    def profile(self, tile, data, *, ground=False):
        return dict(name='SyntheticMap', width=2, height=1, rectangles=[],
                    spawns=[[1, 0]], payload_sha256='1' * 64,
                    cells=[([0, -1] if ground else [-1, 0]), [-1, -1]]), ([tile], {tile: data})

    def setUp(self):
        all_pass = 0x01010101
        self.tiles = {
            'Tile16_Static/PinPillar01': (1, 0, 0, 0, 0, 0, 1, 1, 0, 0, 16843009, 0, 0, 0, 0, 0, 0, 0),
            'Tile16_Static/PinPillar02': (1, 0, 0, 0, 0, 0, 1, 1, 0, 0, 256, 0, 0, 0, 0, 0, 0, 0),
            'Tile16_Static/PinPillar05': (1, 0, 0, 0, 0, 0, 1, 1, 0, 0, 16777216, 0, 0, 0, 0, 0, 0, 0),
            'Tile62_Static/PinPillar_left': (1, 0, 0, 0, 0, 0, 1, 1, 0, 0, 65536, 0, 0, 0, 0, 0, 0, 0),
            'Tile62_Passable/PoliceLine01': (3, 0, 0, 0, 0, 0, 0, 65537, all_pass, 16777217, 1, 1, 0, 0, 0, 0, 0, 0),
            'Tile62_Passable/PoliceLine02': (3, 0, 0, 0, 0, 0, 0, 65537, all_pass, 16777217, 1, 1, 0, 0, 0, 0, 0, 0),
            'Tile10_Passable/BasicBlock_pass00': (3, 0, 0, 0, 0, 0, all_pass, all_pass, all_pass, 0, 0, 0, 99, 0, 0, 0, 0, 0),
            'Tile10_Passable/BasicBlock_pass01': (3, 0, 0, 0, 0, 0, all_pass, all_pass, all_pass, 0, 0, 0, 99, 0, 0, 0, 0, 0),
        }

    def test_measured_tile_variants_are_accepted(self):
        for tile, data in self.tiles.items():
            profile, table = self.profile(tile, data)
            rules = f1_terrain_rules(profile, table)
            expected = 1 if tile.startswith(('Tile16_', 'Tile62_Static/')) else 0
            self.assertEqual(rules['terrain']['kinds'][0], expected)

    def test_island_flower_variants_leave_the_passable_cell_clear(self):
        # TileInfoList KR 00967, ObjectTable IDs 5162 and 5163.
        for tile, tail in [('Flower01', 120722392), ('Flower02', 120723496)]:
            with self.subTest(tile=tile):
                name = 'Tile53_Passable/' + tile
                data = (3, 0, 0, 0, 0, 0, 16843009, 16843009, 16843009,
                        0, 0, 1, 0, 0, 0, 0, 11073034, tail)
                terrain = f1_terrain_rules(*self.profile(name, data))['terrain']
                self.assertEqual(terrain['kinds'], [0, 0])
                self.assertNotIn('ray_barriers', terrain)
                changed_ray = (*data[:8], 16777472, *data[9:])
                with self.assertRaisesRegex(ValueError, 'ngoài adapter'):
                    f1_terrain_rules(*self.profile(name, changed_ray))

    def test_unmeasured_mechanics_remain_rejected(self):
        refused = {
            'TileConv/ConvUR00': (0, 0, 0, 0, 0, 0, 1, 1, 2, 2, 2, 0, 0, 0, 0, 52224, 0, 0),
            'Tile08_Ground/Rail00': (0, 0, 0, 0, 0, 0, 1, 1, 0, 0, -1, 0, 0, 1, 0, 64512, 0, 0),
            'Tile07_Ground/Tile00': (0, 0, 0, 0, 0, 0, 1, 1, 0, 0, -1, 0, 0, 0, 1, 64512, 0, 0),
            'Tile07_Ground/Tile01': (0, 0, 0, 0, 0, 0, 1, 1, 0, 0, -1, 0, 0, 0, 1, 64512, 0, 0),
            'Tile42_Ground/TileRed': (0, 0, 0, 0, 0, 0, 1, 1, 0, 0, -1, 0, 0, 0, 0, 65536, 0, 0),
            'Tile42_Static/Block03': (1, 0, 0, 0, 0, 0, 1, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0),
            'TileTransParent_Static/Static01': (1, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 1637888, 0, 0, 0, 0, 0, 0),
            'Tile19_Static/Block12': (1, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 2010089472, 0, 0, 0, 0, 0, 0),
            'Tile53_Box/Block30': (2, 0, 0, 0, 0, 0, 0, 0, 256, 0, 0, 0, 0, 0, 0, 0, 2487818, 117307672),
            'Tile16_Passble/ArmyTent01': (3, 0, 0, 0, 0, 0, 16777472, 16777472, 16777472, 0, 0, 1, 0, 2010265866, 0, -1, 0, 0),
            'Tile01_Passable/TentOrange': (3, 0, 0, 0, 0, 0, 16777472, 16777472, 16777472, 0, 0, 1, 0, 18730216, 0, 18730216, 0, 0),
            'Tile02_Passble/IceHouse00': (3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 18730216, 0, 18730216, 0, 0),
            'Tile10_Passable/Building00': (3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 64069472, 1, 1244184, 0, 0),
        }
        for tile, data in refused.items():
            with self.subTest(tile=tile), self.assertRaisesRegex(ValueError, 'ngoài adapter'):
                f1_terrain_rules(*self.profile(tile, data, ground=data[0] == 0))


if __name__ == '__main__':
    unittest.main()
