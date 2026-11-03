"""All-eliminated fixture policy regressions; native rank/UI checked separately."""
from pathlib import Path
from types import SimpleNamespace
import tempfile
import unittest
from unittest.mock import patch

from server import probe_codec as codec
from server.probe_service import ProbeService


class AllDeadResultTests(unittest.TestCase):
    def setUp(self):
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        aliases = ['rank_Awin', 'promote_B', 'auth_C', 'roomlist_C', 'join_C',
                   'player_C', 'slot_C', 'guest_C', 'ready_C', 'rank_all_dead']
        profile = SimpleNamespace(frames=dict.fromkeys(aliases),
                                  data={'items': [], 'initial_key': 12345},
                                  body=lambda alias: alias.encode())
        self.service = ProbeService(profile, Path(tmp.name), {})
        self.addCleanup(self.service.selector.close)
        self.a = dict(role='A', reply_ordinal=4, map_response_sent=True)
        self.b = dict(role='B', reply_ordinal=4, map_response_sent=True)
        self.service.events.extend([self.a, self.b])
        self.service.channels.update(A=self.a, B=self.b)
        self.sent = []
        def send(event, alias, label, **kwargs):
            row = dict(role=event['role'], alias=alias, label=label,
                       monotonic_ns=0, **kwargs)
            self.sent.append(row)
            return row
        self.service.send = send
        self.service.start_round()
        self.sent.clear()

    def report(self, role, subtype, tick, seconds):
        event = self.a if role == 'A' else self.b
        slot = 0 if role == 'A' else 1
        body = codec.pack_fields([subtype, slot, 120, 159, tick, 0],
                                 [1, 1, 2, 2, 4, 4])
        # The existing policy preserves this opaque tail. This is not native
        # packet/original-authority evidence; the runtime fixture has its own RE.
        if subtype == 19:
            body += b'fixture-death-tail'
        frame = codec.encode(bytes(37), 127, body, 100 + tick * 3)
        with patch('server.probe_service.time.monotonic_ns', return_value=seconds * 10**9):
            self.service.receive_channel(event, frame, 100 + tick * 3, body)
        return self.service.state['event_reports'][-1]

    def tick(self, seconds):
        with patch('server.probe_service.time.monotonic_ns', return_value=seconds * 10**9):
            self.service.tick()

    def rank_sends(self):
        return [row for row in self.sent if row['alias'].startswith('rank')]

    def test_both_dead_waits_for_latest_death_and_ignores_a_late_survivor_report(self):
        self.report('A', 19, 10, 1)
        self.report('B', 19, 10, 3)
        self.report('B', 21, 11, 4)
        actual_reports = [row['frame_sha256'] for row in self.service.state['event_reports']]
        self.tick(10)  # First death is old enough; latest death is not.
        self.assertEqual(self.rank_sends(), [])
        self.assertTrue(self.service.room.active)
        self.tick(11)
        self.assertEqual([(row['role'], row['alias']) for row in self.rank_sends()],
                         [('A', 'rank_all_dead'), ('B', 'rank_all_dead')])
        self.assertEqual(self.service.round['rank']['death_sequences'],
                         {'A': 130, 'B': 130})
        self.assertFalse(self.service.room.active)
        self.tick(12)
        self.assertEqual(len(self.rank_sends()), 2)
        self.assertEqual([row['frame_sha256'] for row in self.service.state['event_reports']],
                         actual_reports)
        self.tick(16)
        self.assertEqual([row['alias'] for row in self.sent if row['alias'] == 'return'],
                         ['return', 'return'])
        self.tick(19)
        self.assertIn('refresh', self.service.round)
        self.service.start_round()
        self.assertNotIn('death_event', self.a)
        self.assertNotIn('death_event', self.b)
        self.assertNotIn('rank', self.service.round)
        self.assertEqual(self.service.room.key, 12346)

    def test_duplicate_death_does_not_extend_latest_accepted_death_delay(self):
        first = self.report('A', 19, 10, 1)
        last = self.report('B', 19, 10, 2)
        duplicate = self.report('B', 19, 10, 9)
        self.assertEqual(duplicate['rejected'], 'duplicate_report_body')
        self.assertIs(self.a['death_event'], first)
        self.assertIs(self.b['death_event'], last)
        self.tick(10)
        self.assertEqual([row['alias'] for row in self.rank_sends()],
                         ['rank_all_dead', 'rank_all_dead'])

    def test_one_dead_still_requires_actual_other21_and_uses_existing_outcome(self):
        self.report('A', 19, 10, 1)
        self.tick(20)
        self.assertEqual(self.rank_sends(), [])
        self.report('B', 21, 10, 21)
        self.tick(22)
        self.assertEqual([row['alias'] for row in self.rank_sends()], ['rank', 'rank'])
        self.assertNotIn('policy', self.service.round['rank'])

    def test_legacy_profile_leaves_explicit_pending_instead_of_assigning_false_winner(self):
        self.service.profile.frames.pop('rank_all_dead')
        self.report('A', 19, 10, 1)
        self.report('B', 19, 10, 2)
        self.report('B', 21, 11, 3)
        self.tick(20)
        self.tick(21)
        self.assertTrue(self.service.room.active)
        self.assertEqual(self.rank_sends(), [])
        unsupported = [row for row in self.service.state['policy_events']
                       if row['op'] == 'all_dead_result_unsupported']
        self.assertEqual(len(unsupported), 1)

    def test_old_deaths_cannot_finish_a_new_round(self):
        self.report('A', 19, 10, 1)
        self.report('B', 19, 10, 2)
        old_a, old_b = self.a['death_event'], self.b['death_event']
        self.service.room.finish()
        self.service.start_round()
        self.sent.clear()
        self.a['death_event'], self.b['death_event'] = old_a, old_b
        self.assertFalse(self.service.finish_all_dead(100 * 10**9))
        self.assertNotIn('rank', self.service.round)
        self.assertTrue(self.service.room.active)


if __name__ == '__main__':
    unittest.main()
