"""Bounded emulator admission tests; native compatibility is checked in Wine."""
import copy
import hashlib
import unittest
from unittest.mock import patch

from server import test_capacity_reservations as capacity_fixture
from server import test_hidden_items as hidden_fixture
from server import test_terrain_policy as terrain_fixture


class HiddenTerrainAdmissionTests(unittest.TestCase):
    place = terrain_fixture.TerrainPolicyTests.place
    expire_first = terrain_fixture.TerrainPolicyTests.expire_first
    report = hidden_fixture.HiddenItemPolicyTests.report
    pickup = hidden_fixture.HiddenItemPolicyTests.pickup

    def profile(self, limits):
        p = terrain_fixture.TerrainPolicyTests.profile(self, limits)
        p.data['hidden_items'] = [dict(index=27, row=2, column=3, kind=0),
                                  dict(index=28, row=3, column=2, kind=1)]
        t = p.data['bomb_terrain']
        t['kinds'][3 * 15 + 2] = 2
        t['seed_sha256'] = hashlib.sha256(bytes(t['kinds'])).hexdigest()
        return p

    def setUp(self):
        capacity_fixture.CapacityReservationTests.setUp(self)
        self.s = self.service

    def test_unrelated_blast_and_pending_matching_bomb_do_not_release_catalog_cell(self):
        self.assertEqual(self.report()['rejected'], 'no_current_blast_sent')
        self.place(0, row=9)
        self.expire_first()
        self.assertEqual(self.s.state['round_bombs'][0]['terrain_blast']['cleared'], [])
        self.assertEqual(self.report()['rejected'], 'hidden_cell_not_cleared_current_round')
        self.place(28)
        self.assertEqual(self.report()['rejected'], 'hidden_cell_not_cleared_current_round')
        self.assertEqual(self.pickup()['rejected'], 'hidden_item_not_reported_revealed')
        self.assertEqual(self.s.round['revealed_hidden'], {})
        self.expire_first()
        accepted = self.report()
        self.assertEqual(accepted['revealed_indices'], [27])
        self.assertEqual(accepted['terrain_clearance']['bomb_id'], 2)
        self.assertNotIn('rejected', self.pickup())

    def test_other_peer_report_duplicates_and_two_independent_items_after_clear(self):
        self.place(0)
        self.expire_first()
        accepted = self.report(event=self.b, slot=1)
        self.assertEqual(accepted['revealed_indices'], [27])
        self.assertEqual(self.report()['already_revealed'], [27])
        self.assertEqual(self.pickup(28)['rejected'], 'hidden_item_not_reported_revealed')
        self.assertEqual(self.report(row=3, col=2)['revealed_indices'], [28])
        self.assertNotIn('rejected', self.pickup(27))
        self.assertNotIn('rejected', self.pickup(28))
        sent = len(self.sent)
        self.assertEqual(self.pickup(27)['rejected'], 'claim_not_current_or_already_consumed')
        self.assertEqual(len(self.sent), sent)

    def test_old_generation_and_old_clear_cannot_release_new_round_after_unrelated_blast(self):
        self.place(0)
        self.expire_first()
        old = copy.deepcopy(self.s.state['round_bombs'][0])
        self.report()
        self.pickup()
        self.s.room.finish()
        self.s.start_round()
        self.assertEqual(self.s.round['revealed_hidden'], {})
        self.place(0, row=9)
        self.expire_first()
        self.assertEqual(self.report()['rejected'], 'hidden_cell_not_cleared_current_round')
        # The metadata must agree with the generation, not just its outer row.
        old['round'] = self.s.room.generation
        self.s.state['round_bombs'].append(old)
        self.assertEqual(self.report()['rejected'], 'hidden_cell_not_cleared_current_round')
        self.s.state['round_bombs'].pop()
        self.place(28)
        self.expire_first()
        # 142 has no recovered round key. Identical bytes become admissible
        # after a genuine current clear; this is not full replay protection.
        self.assertEqual(self.report()['revealed_indices'], [27])
        self.assertEqual(self.pickup(key=12345)['rejected'], 'round_key_mismatch')
        self.assertNotIn('rejected', self.pickup())

    def test_partial_blast_broadcast_never_commits_clearance(self):
        self.place(0)
        original = self.s.broadcast
        def partial(alias, *args, **kwargs):
            if alias == 'blast':
                self.s.send(self.a, alias, 'partial-test', **kwargs)
                raise BrokenPipeError('second peer send failed')
            return original(alias, *args, **kwargs)
        self.s.broadcast = partial
        with self.assertRaises(BrokenPipeError):
            self.expire_first()
        self.assertNotIn('terrain_blast', self.s.state['round_bombs'][0])
        self.assertEqual(self.report()['rejected'], 'no_current_blast_sent')
        self.assertEqual(self.s.round['revealed_hidden'], {})

    def test_clearance_requires_full_roster_receipts_and_commit_before_report(self):
        self.place(0)
        self.expire_first()
        bomb = self.s.state['round_bombs'][0]
        receipts = bomb['broadcast158']
        bomb['broadcast158'] = receipts[:1]
        self.assertEqual(self.report()['rejected'], 'hidden_cell_not_cleared_current_round')
        bomb['broadcast158'] = receipts
        committed = bomb['terrain_blast']['committed_ns']
        with patch('server.probe_service.time.monotonic_ns', return_value=committed - 1):
            self.assertEqual(self.report()['rejected'], 'hidden_cell_not_cleared_current_round')
        self.assertEqual(self.report()['revealed_indices'], [27])


if __name__ == '__main__':
    unittest.main()
