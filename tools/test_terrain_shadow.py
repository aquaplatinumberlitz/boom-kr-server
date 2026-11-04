"""Candidate model constraints; retained native runs supply behavioral oracles."""
import unittest

from terrain_shadow import Bomb, TerrainShadow


class TerrainShadowTests(unittest.TestCase):
    def world(self, kinds):
        return TerrainShadow(5, 3, kinds, map_sha256='a' * 64)

    def test_invalid_input_does_not_take_id_or_change_world(self):
        world = self.world([0] * 15)
        world.place(Bomb(1, 0, 1, 1, 2))
        before = (world.flags, dict(world.pending), set(world.seen_ids))
        for bomb in [Bomb(2, 0, 1, 1, 1), Bomb(2, 0, -1, 0, 1),
                     Bomb(2, 0, 1, 2, 3), Bomb(True, 0, 1, 2, 1),
                     Bomb(1, 0, 1, 2, 1), Bomb(2, 2, 1, 2, 1)]:
            with self.subTest(bomb=bomb), self.assertRaises(ValueError):
                world.place(bomb)
            self.assertEqual((world.flags, world.pending, world.seen_ids), before)

    def test_box_blocks_this_cascade_then_later_blast_can_cross(self):
        cells = [0] * 15
        cells[7] = 2
        world = self.world(cells)
        world.place(Bomb(1, 0, 1, 1, 2))
        world.place(Bomb(2, 1, 1, 3, 1))
        first = world.blast(1)
        self.assertEqual(first.processed, (1,))
        self.assertEqual(first.cleared, ((1, 2),))
        self.assertEqual(set(world.pending), {2})
        world.place(Bomb(3, 0, 1, 1, 2))
        self.assertEqual(set(world.blast(3).processed), {2, 3})

    def test_solid_stays_blocked_and_old_callback_cannot_affect_new_bomb(self):
        cells = [0] * 15
        cells[7] = 1
        world = self.world(cells)
        world.place(Bomb(1, 0, 1, 1, 2))
        world.place(Bomb(2, 1, 1, 3, 1))
        self.assertEqual(world.blast(1).processed, (1,))
        self.assertEqual(world.flags[7], 1)
        world.place(Bomb(3, 0, 1, 1, 2))
        self.assertFalse(world.blast(1).processed)
        self.assertEqual(set(world.pending), {2, 3})
        world.begin()
        self.assertEqual(world.flags, tuple(int(x != 0) for x in cells))
        self.assertFalse(world.pending)
        world.place(Bomb(1, 0, 1, 1, 2))

    def test_unknown_profile_values_are_rejected(self):
        for cell in [3, -1, True, 'box']:
            cells = [0] * 15
            cells[0] = cell
            with self.subTest(cell=cell), self.assertRaises(ValueError):
                self.world(cells)


if __name__ == '__main__':
    unittest.main()
