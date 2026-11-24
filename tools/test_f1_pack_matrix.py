import tempfile
import unittest
from pathlib import Path

from f1_pack_matrix import (adapter_rejection_group, failure_kind, load_events,
                            omitted_items)


def row(**updates):
    value = {
        'script_tags': [],
        'omitted_placements': [],
        'omitted_positive_drops': [],
    }
    value.update(updates)
    return value


class FailureKindTest(unittest.TestCase):
    def test_known_placements_and_drops_are_not_reported_as_unknown(self):
        shoe, punch, needle = ('Item/ItemShoe', 'Item/ItemPunch', 'Item/ItemNiddle')
        dart, trap, owl = ('Item/ItemDart', 'Item/ItemTrap', 'Item/ItemOwl')
        self.assertEqual(omitted_items([shoe, punch, needle, trap],
                                      [(shoe, 30), (punch, 1), (needle, 1), (dart, 1), (owl, 0)]),
                         ([dart, needle], [dart, needle, trap]))

    def test_script_is_contract_refusal(self):
        with tempfile.TemporaryDirectory() as root:
            self.assertEqual(failure_kind(row(script_tags=['boss']), Path(root)), 'contract_script')

    def test_placement_or_drop_is_contract_refusal(self):
        with tempfile.TemporaryDirectory() as root:
            self.assertEqual(failure_kind(row(omitted_placements=[{'x': 1}]), Path(root)),
                             'contract_placement_or_flag')
            self.assertEqual(failure_kind(row(omitted_positive_drops=['Item/ItemDart']), Path(root)),
                             'contract_placement_or_flag')

    def test_no_round_active_is_driver_error(self):
        with tempfile.TemporaryDirectory() as root:
            self.assertEqual(failure_kind(row(), Path(root)), 'driver_error')

    def test_active_without_return_is_server_error(self):
        with tempfile.TemporaryDirectory() as root:
            path = Path(root) / 'load.log'
            path.write_text('{"event":"round_active"}\n')
            self.assertEqual(failure_kind(row(), Path(root)), 'server_or_adapter_error')

    def test_outer_timeout_after_delivery_does_not_blame_server(self):
        with tempfile.TemporaryDirectory() as root:
            result = Path(root)
            (result / 'load.log').write_text(
                '{"event":"round_active"}\n{"event":"delivery","passed":true}\n{"event":'
            )
            self.assertEqual([e['event'] for e in load_events(result)], ['round_active', 'delivery'])
            self.assertEqual(failure_kind(row(), result, 124), 'harness_timeout')


class AdapterRejectionGroupTest(unittest.TestCase):
    def test_all_remaining_terrain_groups_have_stable_labels(self):
        cases = {
            'partial_payload': 'map parser không đọc hết payload',
            'service_scope': 'Map ngoài phạm vi phục vụ: NXCL_Aqua01',
            'conveyor': 'nền TileConv/ConvUR00',
            'mine_rail': 'nền Tile08_Ground/Rail00',
            'Tile07_ground_flags': 'nền Tile07_Ground/Tile00',
            'Tile42_gameplay_flags': 'vật thể/cờ Tile42_Static/Block03',
            'passable_variant_flags': 'vật thể/cờ Tile16_Passble/ArmyTent01',
            'static_variant_flags': 'vật thể/cờ Tile19_Static/Block12',
            'box_variant_flags': 'vật thể/cờ Tile53_Box/Block30',
            'transparent_static_flags': 'vật thể/cờ TileTransParent_Static/Static01',
        }
        for expected, error in cases.items():
            with self.subTest(expected=expected):
                self.assertEqual(adapter_rejection_group(error), expected)


if __name__ == '__main__':
    unittest.main()
