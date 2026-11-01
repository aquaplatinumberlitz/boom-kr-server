import unittest

from server.item_claims import ItemSpec
from server.probe_room import ProbeRoom


class ProbeRoomTests(unittest.TestCase):
    def test_finish_then_begin_invalidates_both_jobs_and_confirmations(self):
        item = ItemSpec(17, 4, 3, 2)
        room = ProbeRoom([item])
        first_key = room.begin()
        bomb = room.bomb()
        claim = room.claim(0, 0, 0, item)
        room.finish()
        self.assertFalse(room.fence.consume(bomb))
        self.assertFalse(room.claims.consume(claim))
        self.assertNotEqual(room.begin() & 255, first_key & 255)
        self.assertEqual(room.fence.rejection(bomb), 'stale_round')
        self.assertEqual(room.claims.rejection(claim), 'stale_round')
        self.assertIsNotNone(room.claim(1, 1, 0, item))
        self.assertNotEqual(room.bomb().wire_id, bomb.wire_id)

    def test_duplicate_start_keeps_same_key_and_existing_job(self):
        room = ProbeRoom([])
        key = room.begin()
        bomb = room.bomb()
        with self.assertRaises(RuntimeError):
            room.begin()
        self.assertEqual(room.key, key)
        self.assertEqual(room.keys.issued_count, 1)
        self.assertTrue(room.fence.consume(bomb))

    def test_key_exhaustion_does_not_reopen_finished_round(self):
        room = ProbeRoom([])
        for _ in range(256):
            room.begin()
            room.finish()
        with self.assertRaises(RuntimeError):
            room.begin()
        self.assertFalse(room.active)
        self.assertEqual(room.generation, 256)
        with self.assertRaises(RuntimeError):
            room.bomb()
