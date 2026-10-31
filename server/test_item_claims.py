import unittest
from server.item_claims import ItemClaims, ItemSpec


class ItemClaimsTests(unittest.TestCase):
    def setUp(self):
        self.item = ItemSpec(17, 4, 3, 0)
        self.room = ItemClaims()
        self.room.begin([self.item])

    def test_one_winner_and_one_confirmation(self):
        winner = self.room.claim(0, 0, 0, self.item)
        self.assertIsNotNone(winner)
        self.assertIsNone(self.room.claim(1, 1, 0, self.item))
        self.assertTrue(self.room.consume(winner))
        self.assertFalse(self.room.consume(winner))
        self.assertIsNone(self.room.claim(0, 0, 0, self.item))

    def test_old_confirmation_cannot_consume_reused_index(self):
        old = self.room.claim(0, 0, 0, self.item)
        self.room.begin([self.item])
        current = self.room.claim(1, 1, 0, self.item)
        self.assertEqual(self.room.rejection(old), 'stale_round')
        self.assertFalse(self.room.consume(old))
        self.assertTrue(self.room.consume(current))

    def test_invalid_requests_do_not_reserve_item(self):
        for args in [(0, 1, 0, self.item), (0, 0, 1, self.item),
                     (0, 0, 0, ItemSpec(18, 4, 3, 0)),
                     (0, 0, 0, ItemSpec(17, 5, 3, 0)),
                     (0, 0, 0, ItemSpec(17, 4, 4, 0)),
                     (0, 0, 0, ItemSpec(17, 4, 3, 1)),
                     (True, 1, 0, self.item), (0, 0, 0, ItemSpec(17., 4, 3, 0))]:
            with self.subTest(args=args):
                self.assertIsNone(self.room.claim(*args))
        self.assertIsNotNone(self.room.claim(0, 0, 0, self.item))

    def test_finish_and_owner_change_invalidate_pending(self):
        pending = self.room.claim(0, 0, 0, self.item)
        self.room.finish()
        self.assertEqual(self.room.rejection(pending), 'round_closed')
        self.assertFalse(self.room.consume(pending))
        self.assertIsNone(self.room.claim(0, 0, 0, self.item))
        other = ItemClaims()
        other.begin([self.item])
        self.assertEqual(other.rejection(pending), 'different_room_owner')
        self.assertIsNotNone(other.claim(1, 1, 0, self.item))

    def test_invalid_catalog_does_not_reset_current_reservation(self):
        pending = self.room.claim(0, 0, 0, self.item)
        for catalog in [[self.item, self.item], [ItemSpec(-1, 4, 3, 0)],
                        [ItemSpec(1, 256, 3, 0)], [ItemSpec(1, 4, 3, 65536)]]:
            with self.assertRaises(ValueError):
                self.room.begin(catalog)
        self.assertTrue(self.room.consume(pending))

    def test_indistinguishable_old_request_is_explicit_limit(self):
        # No wire epoch: this policy accepts matching fields in a new catalog.
        self.room.begin([self.item])
        self.assertIsNotNone(self.room.claim(0, 0, 0, self.item))


if __name__ == '__main__':
    unittest.main()
