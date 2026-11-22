"""Kiểm chính sách item của adapter pack F1 và payload 582 được cấu hình."""
import struct
import unittest

from kr_pack_config import enable_floor_policies, floor582
from kr_pack_export import ITEMS, f1_floor
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


if __name__ == '__main__':
    unittest.main()
