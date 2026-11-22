import tempfile
import unittest
from pathlib import Path

from f1_pack_matrix import failure_kind


def row(**updates):
    value = {
        'script_tags': [],
        'omitted_placements': [],
        'omitted_positive_drops': [],
    }
    value.update(updates)
    return value


class FailureKindTest(unittest.TestCase):
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


if __name__ == '__main__':
    unittest.main()
