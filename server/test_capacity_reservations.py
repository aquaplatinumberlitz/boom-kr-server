"""Declared fixture capacities reserve server jobs, not a recovered native pool."""
from pathlib import Path
from types import SimpleNamespace
import tempfile
import unittest
from unittest.mock import patch

from server import probe_codec as codec
from server.item_claims import ItemSpec
from server.probe_service import ProbeService


class CapacityReservationTests(unittest.TestCase):
    def profile(self, limits):
        aliases = ['rank_Awin', 'promote_B', 'auth_C', 'roomlist_C', 'join_C',
                   'player_C', 'slot_C', 'guest_C', 'ready_C']
        data = {'items': [dict(index=i, row=4, column=3, kind=0) for i in [16, 18]],
                'initial_key': 12345, 'bomb_capacity': limits}
        return SimpleNamespace(frames=dict.fromkeys(aliases), data=data)

    def setUp(self):
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        self.out = Path(tmp.name)
        profile = self.profile({'A': {'initial': 1, 'maximum': 2},
                                'B': {'initial': 2, 'maximum': 7}})
        self.service = ProbeService(profile, self.out, {})
        self.addCleanup(self.service.selector.close)
        self.a = dict(role='A', reply_ordinal=4, map_response_sent=True)
        self.b = dict(role='B', reply_ordinal=4, map_response_sent=True)
        self.service.events.extend([self.a, self.b])
        self.service.channels.update(A=self.a, B=self.b)
        self.sent = []
        def send(event, alias, label, **kwargs):
            row = dict(role=event['role'], alias=alias, label=label, monotonic_ns=0, **kwargs)
            self.sent.append(row)
            return row
        self.service.send = send
        self.service.start_round()
        self.sent.clear()

    def bomb(self, serial=0, *, role='A', key=None):
        key = self.service.room.key if key is None else key
        values = [key, 0, 0 if role == 'A' else 1, serial, 5, 3, 1, 0, 0, 0, 2, 1, 0]
        body = codec.pack_fields(values, [4, 1, 1, 4, 1, 1, 1, 1, 1, 1, 1, 4, 1])
        self.service.receive_channel(self.a if role == 'A' else self.b,
                                     codec.encode(bytes(37), 118, body, 100), 100, body)
        return self.service.state['bomb_requests'][-1]

    def pickup(self, index=16, *, key=None):
        key = self.service.room.key if key is None else key
        body = codec.pack_fields([0, 0, 4, 3, index, 0, 123, key, 0],
                                 [1, 1, 1, 1, 4, 2, 4, 4, 1])
        self.service.receive_channel(self.a, codec.encode(bytes(37), 124, body, 103), 103, body)
        return self.service.state['item_requests'][-1]

    def test_capacity_rejects_fresh_serial_before_id_job_and_does_not_take_serial(self):
        self.pickup()
        self.bomb(0)
        self.bomb(28)
        before = (len(self.sent), self.service.room._wire_id, list(self.service.jobs))
        rejected = self.bomb(56)
        self.assertEqual(rejected['rejected'], 'bomb_capacity_reserved')
        self.assertEqual(rejected['capacity_guard']['reserved_ids'], [1, 2])
        self.assertNotIn('id', rejected)
        self.assertNotIn(56, self.a['bomb_serials'])
        self.assertEqual((len(self.sent), self.service.room._wire_id, self.service.jobs), before)
        # Owner B retains a separate capacity budget.
        self.assertNotIn('rejected', self.bomb(1, role='B'))

    def test_only_current_catalog_consumed_bomb_increments_once_and_clamps(self):
        self.pickup(key=12344)
        self.assertEqual(self.service.round['bomb_capacity']['A'], 1)
        self.pickup(index=999)
        self.assertEqual(self.service.round['bomb_capacity']['A'], 1)
        self.pickup()
        self.assertEqual(self.service.round['bomb_capacity']['A'], 2)
        self.pickup()
        self.assertEqual(self.service.round['bomb_capacity']['A'], 2)
        self.pickup(index=18)
        self.assertEqual(self.service.round['bomb_capacity']['A'], 2)
        changes = [x for x in self.service.state['policy_events'] if x['op'] == 'bomb_capacity_pickup']
        self.assertEqual([(x['before'], x['after']) for x in changes], [(1, 2), (2, 2)])

    def test_job_retirement_releases_budget_and_no_other_release_is_assumed(self):
        self.bomb(0)
        self.assertEqual(self.bomb(28)['rejected'], 'bomb_capacity_reserved')
        due = self.service.jobs[0][0]
        with patch('server.probe_service.time.monotonic', return_value=due + 0.01):
            self.service.tick()
        self.assertEqual(self.service.jobs, [])
        self.assertNotIn('rejected', self.bomb(28))
        self.assertEqual(self.service.room._wire_id, 2)

    def test_start_resets_budget_and_fences_old_jobs_without_consuming_new_ids(self):
        self.pickup()
        self.bomb(0)
        self.bomb(28)
        self.service.room.finish()
        self.service.start_round()
        self.assertEqual(self.service.round['bomb_capacity'], {'A': 1, 'B': 2})
        self.assertEqual(self.bomb(56, key=12345)['rejected'], 'bomb_round_key_mismatch')
        self.assertNotIn('rejected', self.bomb(0))
        self.assertEqual(self.service.room._wire_id, 3)
        self.assertEqual(self.bomb(28)['rejected'], 'bomb_capacity_reserved')

    def test_duplicate_serial_and_wrong_key_keep_existing_reasons(self):
        self.bomb(0)
        self.assertEqual(self.bomb(0)['rejected'], 'duplicate_bomb_serial')
        self.assertEqual(self.bomb(28, key=12344)['rejected'], 'bomb_round_key_mismatch')
        self.assertEqual(self.service.room._wire_id, 1)

    def test_unconfigured_replacement_cannot_start_or_advance_key(self):
        self.service.room.finish()
        self.service.round['refresh'] = {'monotonic_ns': 1}
        self.a['closed'] = {'reason': 'eof'}
        c = dict(role='C', reply_ordinal=4, map_response_sent=True, guest_ready_received=True)
        self.service.events.append(c)
        self.service.host_role = 'B'
        self.service.receive_channel(self.b, codec.encode(bytes(37), 104, b'', 100), 100, b'')
        self.assertEqual(self.service.state['start_rejections'][-1]['reason'],
                         'bomb_capacity_roster_not_configured')
        self.assertEqual(self.service.room.generation, 1)
        self.assertFalse(self.service.room.active)

    def test_invalid_profile_limits_fail_before_selector_or_listener(self):
        for limits in [None, {}, {'A': {'initial': 1, 'maximum': 2}},
                       {'A': {'initial': True, 'maximum': 2}, 'B': {'initial': 2, 'maximum': 7}},
                       {'A': {'initial': 3, 'maximum': 2}, 'B': {'initial': 2, 'maximum': 7}}]:
            with self.subTest(limits=limits), self.assertRaises(ValueError):
                ProbeService(self.profile(limits), self.out, {})


if __name__ == '__main__':
    unittest.main()
