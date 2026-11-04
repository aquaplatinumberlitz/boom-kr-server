"""Policy/lifecycle constraints; native evidence remains the compatibility oracle."""
import copy
import hashlib
import unittest
from unittest.mock import patch

from server import probe_codec as codec
from server import test_capacity_reservations as capacity_fixture
from server.probe_service import ProbeService


FRAMES = {'map': b'measured-map-fixture', 'start': b'measured-start-fixture'}


def terrain_spec():
    cells = [0] * 195
    cells[2 * 15 + 3] = 2
    cells[3 * 15 + 6] = 1
    return dict(width=15, height=13, kinds=cells, maximum_power=2,
                map_sha256='a' * 64, seed_sha256=hashlib.sha256(bytes(cells)).hexdigest(),
                fixtures={k: hashlib.sha256(v).hexdigest() for k, v in FRAMES.items()})


class TerrainPolicyTests(unittest.TestCase):
    setUp = capacity_fixture.CapacityReservationTests.setUp
    pickup = capacity_fixture.CapacityReservationTests.pickup

    def profile(self, limits):
        profile = capacity_fixture.CapacityReservationTests.profile(self, limits)
        profile.frames.update(FRAMES)
        profile.data['bomb_terrain'] = terrain_spec()
        return profile

    def place(self, serial, row=3, col=3, *, role='A', key=None, power=2, variant=1):
        values = [self.service.room.key if key is None else key, 0,
                  0 if role == 'A' else 1, serial, row, col, variant, 0, 0, 0, power, 1, 0]
        body = codec.pack_fields(values, [4, 1, 1, 4, 1, 1, 1, 1, 1, 1, 1, 4, 1])
        self.service.receive_channel(self.a if role == 'A' else self.b,
                                     codec.encode(bytes(37), 118, body, 100), 100, body)
        return self.service.state['bomb_requests'][-1]

    def expire_first(self):
        due = self.service.jobs[0][0]
        with patch('server.probe_service.time.monotonic', return_value=due + .01):
            self.service.tick()

    def test_variant_cell_overlap_power_reject_without_id_serial_job_or_world_mutation(self):
        self.pickup()
        self.place(0)
        before = (self.service.room._wire_id, list(self.service.jobs),
                  self.service.terrain.flags, dict(self.service.terrain.pending), len(self.sent))
        for args, reason in [({'row': 3, 'col': 3}, 'bomb_terrain_placement'),
                             ({'row': 2, 'col': 3}, 'bomb_terrain_placement'),
                             ({'row': 3, 'col': 6}, 'bomb_terrain_placement'),
                             ({'row': 255}, 'bomb_terrain_placement'),
                             ({'power': 3}, 'bomb_terrain_placement'),
                             ({'variant': 2}, 'bomb_terrain_variant')]:
            self.assertEqual(self.place(28, **args)['rejected'], reason)
            self.assertNotIn(28, self.a['bomb_serials'])
            self.assertEqual((self.service.room._wire_id, self.service.jobs,
                              self.service.terrain.flags, self.service.terrain.pending,
                              len(self.sent)), before)

    def test_capacity_current_key_duplicate_and_independent_owner_guards_remain(self):
        self.assertNotIn('rejected', self.place(0))
        self.assertEqual(self.place(28, row=5)['rejected'], 'bomb_capacity_reserved')
        self.assertNotIn(28, self.a['bomb_serials'])
        self.assertEqual(self.place(28, key=12344)['rejected'], 'bomb_round_key_mismatch')
        self.assertEqual(self.place(0)['rejected'], 'duplicate_bomb_serial')
        self.assertNotIn('rejected', self.place(1, col=5, role='B'))

    def test_cascade_commits_after_broadcast_releases_other_owner_and_retains_own_timer(self):
        self.place(0)
        with patch('server.probe_service.time.monotonic', return_value=self.service.jobs[0][0]):
            self.place(1, col=5, role='B')
        old_secondary = self.service.jobs[1][1]
        seen = []
        original = self.service.broadcast
        def broadcast(*args, **kwargs):
            if args[0] == 'blast':
                seen.append(self.service.terrain.flags[2 * 15 + 3])
                self.assertNotIn('capacity_release', self.service.jobs[0][2])
            return original(*args, **kwargs)
        self.service.broadcast = broadcast
        self.expire_first()
        self.assertEqual(seen, [1])
        self.assertEqual(self.service.terrain.flags[2 * 15 + 3], 0)
        root = self.service.state['round_bombs'][0]
        self.assertEqual(root['terrain_blast']['processed'], [1, 2])
        self.assertEqual(root['terrain_blast']['cleared'], [[2, 3]])
        self.assertEqual(self.service.jobs[0][2]['capacity_release']['root_id'], 1)
        self.assertIsNone(self.service.room.fence.rejection(old_secondary))
        self.assertFalse(self.service.terrain.pending)

    def test_old_secondary_deadline_cannot_ghost_explode_new_bomb(self):
        self.place(0)
        with patch('server.probe_service.time.monotonic', return_value=self.service.jobs[0][0]):
            self.place(1, col=5, role='B')
        self.expire_first()
        due = self.service.jobs[0][0]
        with patch('server.probe_service.time.monotonic', return_value=due - .2):
            self.place(28, row=3, col=4)
        flags = self.service.terrain.flags
        self.expire_first()
        self.assertEqual(self.service.state['round_bombs'][1]['terrain_blast']['processed'], [])
        self.assertEqual(self.service.terrain.flags, flags)
        self.assertEqual(set(self.service.terrain.pending), {3})
        self.assertNotIn('capacity_release', self.service.state['round_bombs'][2])

    def test_start_restores_seed_and_stale_jobs_do_not_clear_new_round(self):
        self.place(0)
        with patch('server.probe_service.time.monotonic', return_value=self.service.jobs[0][0]):
            self.place(1, col=5, role='B')
        self.expire_first()
        self.service.room.finish()
        self.service.start_round()
        self.assertEqual(self.service.terrain.flags[2 * 15 + 3], 1)
        with patch('server.probe_service.time.monotonic',
                   return_value=self.service.jobs[0][0] - .2):
            self.assertNotIn('rejected', self.place(0))
        self.expire_first()  # Stale secondary job, not the new root.
        self.assertEqual(self.service.terrain.flags[2 * 15 + 3], 1)
        self.assertEqual(set(self.service.terrain.pending), {3})

    def test_failed_blast_broadcast_does_not_commit_terrain_or_release(self):
        self.place(0)
        with patch('server.probe_service.time.monotonic', return_value=self.service.jobs[0][0]):
            self.place(1, col=5, role='B')
        self.service.broadcast = unittest.mock.Mock(side_effect=BrokenPipeError('partial transport'))
        with self.assertRaises(BrokenPipeError):
            self.expire_first()
        self.assertEqual(self.service.terrain.flags[2 * 15 + 3], 1)
        self.assertEqual(set(self.service.terrain.pending), {1, 2})
        self.assertNotIn('capacity_release', self.service.jobs[0][2])
        # Existing fence consumption/job retirement is not rolled back. This
        # boundary is not general send-error recovery or a whole-room transaction.
        self.assertEqual(len(self.service.jobs), 1)

    def test_failed_placement_broadcast_does_not_commit_pending_terrain_bomb(self):
        before = self.service.terrain.flags
        self.service.broadcast = unittest.mock.Mock(side_effect=BrokenPipeError('partial159'))
        with self.assertRaises(BrokenPipeError):
            self.place(0)
        self.assertEqual(self.service.terrain.flags, before)
        self.assertFalse(self.service.terrain.pending)
        self.assertFalse(self.service.jobs)
        # Existing allocator/serial bookkeeping precedes159. A transport failure
        # is surfaced, not a retry-safe whole-placement transaction.
        self.assertEqual(self.service.room._wire_id, 1)
        self.assertIn(0, self.a['bomb_serials'])

    def test_bad_profile_seed_fixture_hash_or_combination_fails_before_selector(self):
        limits = {'A': {'initial': 1, 'maximum': 2}, 'B': {'initial': 2, 'maximum': 7}}
        base = self.profile(limits)
        for mutate in [lambda p: p.data['bomb_terrain'].update(seed_sha256='0' * 64),
                       lambda p: p.data['bomb_terrain']['fixtures'].update(map='0' * 64),
                       lambda p: p.data['bomb_terrain'].update(width=14),
                       lambda p: p.data.pop('bomb_capacity'),
                       lambda p: p.data.update(capacity_chain={})]:
            profile = copy.deepcopy(base)
            mutate(profile)
            with self.assertRaises(ValueError):
                ProbeService(profile, self.out, {})


if __name__ == '__main__':
    unittest.main()
