"""Local loading deadline regressions; not an oracle for Nexon timeout policy."""
from unittest.mock import patch
import unittest

from server.probe_service import ProbeService
from server import test_probe_eof as eof_fixture


class LoadingTimeoutTests(unittest.TestCase):
    def setUp(self):
        eof_fixture.EofTests.setUp(self)
        self.service.room.finish()

    tick_at = eof_fixture.EofTests.tick_at

    def begin(self, seconds=100):
        self.service.loading_timeout_ns = 30 * 10**9
        for event in self.service.events:
            event['create_response_sent'] = True
        with patch('server.probe_service.time.monotonic_ns', return_value=seconds * 10**9):
            self.service.start_round()
        self.sends.clear()
        return self.service.round['loading']

    def test_deadline_closes_missing_guest_once_fences_jobs_and_returns_survivor(self):
        loading = self.begin()
        a, b = self.service.events
        a['init99'] = {'actual': True}
        ticket = self.service.room.bomb()
        self.tick_at(129)
        self.assertNotIn('closed', b)
        self.assertEqual(self.sends, [])
        self.tick_at(130)
        self.assertEqual(b['closed']['reason'], 'loading_timeout')
        self.assertNotIn('closed', a)
        self.assertEqual(loading['closed_connections'], [2])
        self.assertFalse(self.service.room.active)
        self.assertEqual(self.service.room.fence.rejection(ticket), 'round_closed')
        self.assertNotIn('death_event', b)
        self.assertEqual(self.sends, [])
        self.tick_at(142)
        self.tick_at(147)
        self.tick_at(150)
        self.assertEqual([s['opcode'] for s in self.sends], [175, 180, 107, 110])
        self.assertTrue(all(s['role'] == 'A' for s in self.sends))
        self.tick_at(500)
        self.assertEqual(len(self.sends), 4)
        self.assertEqual(sum(e['op'] == 'loading_timeout'
                             for e in self.service.state['policy_events']), 1)

    def test_dispatched_barrier_wins_at_deadline_and_remains_resolved(self):
        loading = self.begin()
        for event in self.service.events:
            event['init99'] = {'actual': True}
        self.service.tick_loading(130 * 10**9)
        self.assertEqual(loading['resolution'], 'barrier_complete')
        for event in self.service.events:
            event.pop('init99')
        self.service.tick_loading(900 * 10**9)
        self.assertTrue(self.service.room.active)
        self.assertTrue(all(not e.get('closed') for e in self.service.events))

    def test_expiry_with_both_missing_never_sends_to_closed_sockets(self):
        loading = self.begin()
        self.tick_at(130)
        self.assertEqual(loading['closed_connections'], [1, 2])
        for seconds in [142, 147, 150, 500]:
            self.tick_at(seconds)
        self.assertEqual(self.sends, [])

    def test_new_round_rearms_deadline_and_clears_previous_init_markers(self):
        old = self.begin()
        for event in self.service.events:
            event['init99'] = {'actual': True}
        self.service.tick_loading(110 * 10**9)
        self.service.room.finish()
        new = self.begin(200)
        self.assertGreater(new['generation'], old['generation'])
        self.assertEqual(new['deadline_ns'], 230 * 10**9)
        self.assertTrue(all('init99' not in e for e in self.service.events))
        self.tick_at(210)
        self.assertTrue(self.service.room.active)
        self.assertTrue(all('closed' not in e for e in self.service.events))

    def test_replaced_channel_and_closed_round_cannot_be_closed_by_old_deadline(self):
        loading = self.begin()
        old = self.service.channels['B']
        replacement = {**old, 'global_ordinal': 99}
        self.service.channels['B'] = replacement
        self.service.tick_loading(130 * 10**9)
        self.assertEqual(loading['resolution'], 'membership_changed')
        self.assertNotIn('closed', old)
        self.assertNotIn('closed', replacement)
        self.service.channels['B'] = old
        self.service.room.finish()
        self.begin(200)
        self.service.room.finish()
        self.service.tick_loading(400 * 10**9)
        self.assertTrue(all('closed' not in e for e in self.service.events))

    def test_default_off_and_invalid_configuration_rejected(self):
        self.assertIsNone(self.service.loading_timeout_ns)
        self.service.start_round()
        self.assertNotIn('loading', self.service.round)
        self.tick_at(900)
        self.assertTrue(self.service.room.active)
        profile = self.service.profile
        for bad in [None, {}, {'mode': 'close_missing_init99', 'seconds': True},
                    {'mode': 'close_missing_init99', 'seconds': 0},
                    {'mode': 'close_missing_init99', 'seconds': 301},
                    {'mode': 'invent99', 'seconds': 30},
                    {'mode': 'close_missing_init99', 'seconds': 30, 'extra': 0}]:
            profile.data['loading_timeout'] = bad
            with self.assertRaisesRegex(ValueError, 'loading_timeout'):
                ProbeService(profile, self.service.out, {})
        profile.data['loading_timeout'] = {'mode': 'close_missing_init99', 'seconds': 30}
        configured = ProbeService(profile, self.service.out, {})
        self.addCleanup(configured.selector.close)
        self.assertEqual(configured.loading_timeout_ns, 30 * 10**9)


if __name__ == '__main__':
    unittest.main()
