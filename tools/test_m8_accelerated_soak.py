import unittest

from m8_accelerated_soak import build_driver, partition


class AcceleratedSoakTest(unittest.TestCase):
    def accounts(self, count=24):
        return [{"login": f"User{index:02}", "name": f"Name{index:02}",
                 "password": "password123"} for index in range(count)]

    def test_balances_eight_short_rooms(self):
        source = {"rooms": [self.accounts()]}
        result = build_driver(source, 8, 2, 180, 100)
        self.assertEqual(len(result["rooms"]), 8)
        self.assertTrue(all(len(room) == 3 for room in result["rooms"]))
        self.assertEqual(result["active_seconds"], 2)
        self.assertEqual(result["minimum_run_seconds"], 180)

    def test_rejects_room_capacity_mismatch(self):
        with self.assertRaises(ValueError):
            partition(self.accounts(3), 2)


if __name__ == "__main__":
    unittest.main()
