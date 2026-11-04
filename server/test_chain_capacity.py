"""Model checks for optional direct-chain reservation policy, not KR oracles."""
import copy
import hashlib
import unittest
from unittest.mock import patch

from server.chain_capacity import ClearCellChains
from server.probe_service import ProbeService
from server import test_capacity_reservations as capacity_fixture


FRAMES = {'map': b'map', 'start': b'start'}


def spec(cells):
    return {'fixtures': {k: hashlib.sha256(v).hexdigest() for k, v in FRAMES.items()},
            'clear_cells': cells, 'maximum_power': 2}


def bomb(index, row, column, power=2):
    return dict(id=index, row=row, column=column, power=power)


class ChainGeometryTests(unittest.TestCase):
    def test_first_hit_only_no_transitive_non_aligned_or_out_of_range_release(self):
        rule = ClearCellChains(spec([[3, c] for c in range(3, 7)] + [[4, 3], [5, 3]]), FRAMES)
        root = bomb(1, 3, 3)
        targets = [bomb(2, 3, 4), bomb(3, 3, 5), bomb(4, 5, 3),
                   bomb(5, 4, 4), bomb(6, 3, 6)]
        self.assertEqual([b['id'] for b in rule.direct_targets(root, targets)], [2, 4])
        self.assertEqual(rule.direct_targets(bomb(7, 3, 3, 3), targets), [])

    def test_unknown_intermediate_cell_stops_ray_and_ambiguous_overlap_retains_budget(self):
        rule = ClearCellChains(spec([[3, 3], [3, 5]]), FRAMES)
        self.assertEqual(rule.direct_targets(bomb(1, 3, 3), [bomb(2, 3, 5)]), [])
        rule = ClearCellChains(spec([[3, 3], [3, 4], [3, 5]]), FRAMES)
        self.assertEqual(rule.direct_targets(bomb(1, 3, 3),
                                            [bomb(2, 3, 4), bomb(3, 3, 4), bomb(4, 3, 5)]), [])

    def test_profile_requires_matching_fixtures_unique_bounded_cells_and_power(self):
        good = spec([[3, 3], [3, 4]])
        for field, value in [('clear_cells', [[3, 3], [3, 3]]),
                             ('clear_cells', [[True, 3]]), ('clear_cells', []),
                             ('maximum_power', True), ('maximum_power', 256),
                             ('fixtures', {'map': '0' * 64, 'start': good['fixtures']['start']})]:
            bad = copy.deepcopy(good)
            bad[field] = value
            with self.subTest(field=field, value=value), self.assertRaises(ValueError):
                ClearCellChains(bad, FRAMES)


class ChainLifetimeTests(unittest.TestCase):
    setUp = capacity_fixture.CapacityReservationTests.setUp
    profile = capacity_fixture.CapacityReservationTests.profile
    pickup = capacity_fixture.CapacityReservationTests.pickup
    bomb = capacity_fixture.CapacityReservationTests.bomb

    def install(self):
        self.service.capacity_chains = ClearCellChains(spec([[5, 3], [5, 4], [5, 5],
                                                           [4, 3], [3, 3]]), FRAMES)

    def place(self, serial, row, column, role='A'):
        # Use the real receive_channel contract, with distinct geometry.
        from server import probe_codec as codec
        values = [self.service.room.key, 0, 0 if role == 'A' else 1, serial,
                  row, column, 1, 0, 0, 0, 2, 1, 0]
        body = codec.pack_fields(values, [4, 1, 1, 4, 1, 1, 1, 1, 1, 1, 1, 4, 1])
        self.service.receive_channel(self.a if role == 'A' else self.b,
                                     codec.encode(bytes(37), 118, body, 100), 100, body)
        return self.service.state['bomb_requests'][-1]

    def expire_first(self):
        due = self.service.jobs[0][0]
        with patch('server.probe_service.time.monotonic', return_value=due + .01):
            self.service.tick()

    def test_preblast_guard_then_release_secondary_budget_without_retiring_job_or_fence(self):
        self.install()
        self.pickup()
        self.place(0, 5, 3)
        with patch('server.probe_service.time.monotonic', return_value=self.service.jobs[0][0]):
            self.place(28, 5, 5)
        self.assertEqual(self.place(56, 4, 3)['rejected'], 'bomb_capacity_reserved')
        secondary_ticket = self.service.jobs[1][1]
        self.expire_first()
        self.assertEqual([b['id'] for _, _, b in self.service.jobs], [2])
        self.assertIsNone(self.service.room.fence.rejection(secondary_ticket))
        self.assertEqual(self.service.jobs[0][2]['capacity_release']['root_id'], 1)
        self.assertNotIn('rejected', self.place(56, 4, 3))
        self.assertNotIn('rejected', self.place(84, 3, 3))
        self.assertEqual(self.place(112, 5, 4)['capacity_guard']['reserved_ids'], [3, 4])

    def test_old_secondary_timer_cannot_release_new_bomb_on_its_own_ray(self):
        self.install()
        self.pickup()
        self.place(0, 5, 3)
        with patch('server.probe_service.time.monotonic', return_value=self.service.jobs[0][0]):
            self.place(28, 5, 5)
        self.expire_first()
        # Keep model time forward after the first simulated deadline; reverting
        # to wall time would make the replacement expire before the secondary.
        with patch('server.probe_service.time.monotonic', return_value=self.service.jobs[0][0] - .2):
            self.place(56, 5, 4)
        self.expire_first()
        new = self.service.state['round_bombs'][-1]
        self.assertNotIn('capacity_release', new)
        self.assertEqual(len([b for b in self.sent if b.get('label', '').startswith('blast158')]), 4)

    def test_other_owner_budget_released_but_stale_generation_never_releases_current(self):
        self.install()
        self.place(0, 5, 3)
        with patch('server.probe_service.time.monotonic', return_value=self.service.jobs[0][0]):
            self.place(1, 5, 5, role='B')
        self.expire_first()
        self.assertEqual(self.service.state['round_bombs'][-1]['capacity_release']['root_id'], 1)
        self.service.room.finish()
        self.service.start_round()
        current = self.place(0, 5, 3)
        self.expire_first()  # Pending old secondary ticket from the previous generation.
        self.assertNotIn('capacity_release', self.service.state['round_bombs'][-1])
        self.assertEqual(current['capacity_guard']['reserved_ids'], [])

    def test_configuration_without_capacity_is_rejected_before_selector(self):
        profile = self.profile({'A': {'initial': 1, 'maximum': 2},
                                'B': {'initial': 2, 'maximum': 7}})
        profile.data.pop('bomb_capacity')
        profile.data['capacity_chain'] = spec([[3, 3]])
        profile.frames.update(FRAMES)
        with self.assertRaisesRegex(ValueError, 'requires bomb_capacity'):
            ProbeService(profile, self.out, {})


if __name__ == '__main__':
    unittest.main()
