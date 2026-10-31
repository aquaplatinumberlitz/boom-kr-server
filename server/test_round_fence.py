"""Behavioral tests for emulator job lifetime, independent of the KR codec."""

import unittest

from server.round_fence import RoundFence


class RoundFenceTests(unittest.TestCase):
    def test_previous_round_job_cannot_consume_reused_id(self):
        room = RoundFence()
        room.begin()
        old = room.register(1)
        room.begin()
        current = room.register(1)
        self.assertEqual(room.rejection(old), 'stale_round')
        self.assertFalse(room.consume(old))
        self.assertTrue(room.consume(current))

    def test_duplicate_and_same_round_reallocation(self):
        room = RoundFence()
        room.begin()
        old = room.register(7)
        self.assertTrue(room.consume(old))
        self.assertFalse(room.consume(old))
        current = room.register(7)
        self.assertFalse(room.consume(old))
        self.assertTrue(room.consume(current))

    def test_finish_cancels_jobs_before_next_begin(self):
        room = RoundFence()
        with self.assertRaises(RuntimeError):
            room.register(1)
        room.begin()
        old = room.register(1)
        room.finish()
        room.finish()
        self.assertEqual(room.rejection(old), 'round_closed')
        self.assertFalse(room.consume(old))
        with self.assertRaises(RuntimeError):
            room.register(2)

    def test_new_owner_cannot_accept_old_owner_ticket(self):
        old_owner = RoundFence()
        new_owner = RoundFence()
        old_owner.begin()
        new_owner.begin()
        old = old_owner.register(1)
        current = new_owner.register(1)
        self.assertEqual(new_owner.rejection(old), 'different_room_owner')
        self.assertFalse(new_owner.consume(old))
        self.assertTrue(new_owner.consume(current))

    def test_active_id_collision_preserves_existing_job(self):
        room = RoundFence()
        room.begin()
        ticket = room.register(65535)
        with self.assertRaises(ValueError):
            room.register(65535)
        self.assertTrue(room.consume(ticket))

    def test_invalid_wire_ids_do_not_replace_active_job(self):
        room = RoundFence()
        room.begin()
        current = room.register(1)
        for wire_id in [0, -1, 65536, True, 1.0, '1', None]:
            with self.subTest(wire_id=wire_id), self.assertRaises(ValueError):
                room.register(wire_id)
        self.assertTrue(room.consume(current))


if __name__ == '__main__':
    unittest.main()
