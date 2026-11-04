"""CP099 temporal regression; fixture policy, not a native outcome oracle."""
import copy
import hashlib
import struct
import unittest
from unittest.mock import Mock, patch

from server import probe_codec as codec
from server import test_all_dead_result as fixture
from server.probe_service import ProbeService


class ResultFreshnessTests(unittest.TestCase):
    setUp = fixture.AllDeadResultTests.setUp
    report = fixture.AllDeadResultTests.report
    tick = fixture.AllDeadResultTests.tick
    rank_sends = fixture.AllDeadResultTests.rank_sends

    def enable(self):
        self.service.result_freshness = True
        # Give successful sends their actual controlled test clock, including
        # both recipients. The production boundary uses send-completion time.
        original = self.service.send
        def send(*args, **kwargs):
            row = original(*args, **kwargs)
            from server.probe_service import time
            row['monotonic_ns'] = time.monotonic_ns()
            return row
        self.service.send = send

    def bomb(self, seconds=9, serial=0):
        body = codec.pack_fields([self.service.room.key, 0, 0, serial, 5, 3,
                                  1, 0, 0, 0, 2, 1, 0],
                                 [4, 1, 1, 4, 1, 1, 1, 1, 1, 1, 1, 4, 1])
        with patch('server.probe_service.time.monotonic', return_value=seconds - 6):
            self.service.receive_channel(self.a, codec.encode(bytes(37), 118, body, 100), 100, body)

    def blast(self, seconds=9):
        self.bomb(seconds)
        with patch('server.probe_service.time.monotonic', return_value=seconds + .01):
            self.tick(seconds)

    def trap(self, tick, seconds):
        raw = bytes.fromhex('180900000000') + struct.pack('<II', 0, 7)
        tail = codec.pack_fields([14], [4]) + codec.pack_fields(
            [int.from_bytes(raw[i:i + w], 'little') for i, w in [(0, 4), (4, 4), (8, 4), (12, 2)]],
            [4, 4, 4, 2])
        body = codec.pack_fields([17, 0, 120, 159, tick, 0], [1, 1, 2, 2, 4, 4]) + tail
        with patch('server.probe_service.time.monotonic_ns', return_value=seconds * 10**9):
            self.service.receive_channel(self.a, codec.encode(bytes(37), 127, body, 103), 103, body)
        return self.service.state['event_reports'][-1]

    def test_cp099_old21_then_new158_cannot_finish_but_new21_can(self):
        self.enable()
        self.report('B', 19, 10, 1)
        self.report('A', 21, 10, 8)
        self.blast(9)
        self.assertTrue(self.service.room.active)
        self.assertEqual(self.rank_sends(), [])
        self.assertEqual(self.service.round['result_last_blast']['monotonic_ns'], 9 * 10**9)
        self.report('A', 21, 11, 9)  # Equal to send completion is not newer.
        self.tick(9)
        self.assertTrue(self.service.room.active)
        self.report('A', 21, 12, 10)
        self.tick(10)
        self.assertEqual([row['alias'] for row in self.rank_sends()], ['rank_Awin'] * 2)
        self.assertEqual(self.service.round['rank']['policy'], 'post_blast_untrapped')
        self.tick(15)
        self.assertIn('return', self.service.round)

    def test_absent_policy_keeps_legacy_old21_finish(self):
        self.report('B', 19, 10, 1)
        self.report('A', 21, 10, 8)
        self.blast(9)
        self.assertFalse(self.service.room.active)
        self.assertNotIn('result_last_blast', self.service.round)
        self.assertNotIn('policy', self.service.round['rank'])

    def test_trap_blocks_even_fresh21_until_start_no_rescue_assumed(self):
        self.enable()
        self.report('B', 19, 10, 1)
        self.trap(10, 8)
        self.report('A', 21, 11, 9)
        self.tick(10)
        self.assertTrue(self.service.room.active)
        self.assertEqual(self.rank_sends(), [])
        cursor = copy.deepcopy(self.a['report127_cursor'])
        self.service.room.finish()
        self.service.start_round()
        self.assertNotIn('pickup_ineligible', self.a)
        self.assertNotIn('result_last_blast', self.service.round)
        self.assertEqual(self.a['report127_cursor'], cursor)
        self.report('B', 19, 11, 12)
        self.report('A', 21, 12, 13)
        self.tick(20)
        self.assertFalse(self.service.room.active)

    def test_rejected_old_trap_and_duplicate21_do_not_change_eligibility(self):
        self.enable()
        self.report('B', 19, 10, 1)
        self.report('A', 21, 20, 8)
        self.blast(9)
        self.assertEqual(self.trap(19, 10)['rejected'], 'report_tick_not_forward')
        self.assertNotIn('pickup_ineligible', self.a)
        self.assertEqual(self.report('A', 21, 20, 11)['rejected'], 'duplicate_report_body')
        self.tick(11)
        self.assertTrue(self.service.room.active)
        self.report('A', 21, 21, 12)
        self.tick(12)
        self.assertFalse(self.service.room.active)

    def test_closed_member_and_old_round_report_cannot_win(self):
        self.enable()
        self.report('B', 19, 10, 1)
        self.report('A', 21, 10, 2)
        self.a['closed'] = {'reason': 'fixture'}
        self.tick(10)
        self.assertEqual(self.rank_sends(), [])
        self.a.pop('closed')
        self.service.room.finish()
        self.service.start_round()
        self.report('B', 19, 11, 12)
        self.tick(20)
        self.assertEqual(self.rank_sends(), [])

    def test_failed_blast_does_not_commit_freshness_marker(self):
        self.enable()
        self.bomb()
        self.service.broadcast = Mock(side_effect=BrokenPipeError('partial158'))
        with self.assertRaises(BrokenPipeError):
            with patch('server.probe_service.time.monotonic', return_value=9.01):
                self.tick(9)
        self.assertNotIn('result_last_blast', self.service.round)

    def test_failed_later_broadcast_preserves_last_successful_marker(self):
        self.enable()
        self.blast(9)
        previous = dict(self.service.round['result_last_blast'])
        # A distinct accepted serial creates a second real deadline job.
        self.bomb(10, serial=28)
        self.service.broadcast = Mock(side_effect=BrokenPipeError('partial later158'))
        with self.assertRaises(BrokenPipeError):
            with patch('server.probe_service.time.monotonic', return_value=10.01):
                self.tick(10)
        self.assertEqual(self.service.round['result_last_blast'], previous)

    def test_fenced_old_job_cannot_invalidate_new_round21(self):
        self.enable()
        self.bomb()
        self.service.room.finish()
        self.service.start_round()
        self.report('B', 19, 10, 1)
        self.report('A', 21, 10, 8)
        with patch('server.probe_service.time.monotonic', return_value=9.01):
            self.tick(9)
        self.assertNotIn('result_last_blast', self.service.round)
        self.assertFalse(self.service.room.active)

    def test_inactive127_is_only_transport_observation_even_short_body(self):
        self.enable()
        self.report('A', 21, 10, 1)
        self.service.room.finish()
        before = (copy.deepcopy(self.a), copy.deepcopy(self.service.round),
                  copy.deepcopy(self.service.state['event_reports']), list(self.sent))
        for body in [b'bad', codec.pack_fields([17, 0, 80, 159, 11, 5], [1, 1, 2, 2, 4, 4])]:
            frame = codec.encode(bytes(37), 127, body, 133)
            self.service.receive_channel(self.a, frame, 133, body)
            row = self.service.state['inactive_reports'][-1]
            self.assertEqual(row['body_sha256'], hashlib.sha256(body).hexdigest())
            self.assertEqual(row['prefix'] is None, len(body) < 14)
        self.assertEqual((self.a, self.service.round, self.service.state['event_reports'], self.sent), before)

    def test_both_dead_guard_reuses_optional_fixture_and_explicit_missing_alias(self):
        for configured in [True, False]:
            with self.subTest(configured=configured):
                self.enable()
                if not configured:
                    self.service.profile.frames.pop('rank_all_dead')
                self.report('A', 19, 10, 1)
                self.report('B', 19, 10, 2)
                self.report('A', 21, 11, 3)
                self.tick(10)
                if configured:
                    self.assertEqual([r['alias'] for r in self.rank_sends()], ['rank_all_dead'] * 2)
                    self.service.start_round()
                    # A new connection cursor is not needed: next equal-body
                    # reports would be duplicates, so use different native ticks.
                    self.a.pop('report127_cursor'); self.b.pop('report127_cursor')
                    self.sent.clear()
                else:
                    self.assertTrue(self.service.room.active)
                    self.assertEqual(self.rank_sends(), [])
                    self.assertEqual(self.service.state['policy_events'][-1]['op'], 'all_dead_result_unsupported')

    def test_config_validates_before_any_selector_or_socket(self):
        for value in [None, False, {}, {'mode': 'other'}, {'mode': 'post_blast_untrapped', 'extra': 1}]:
            with self.subTest(value=value):
                profile = copy.deepcopy(self.service.profile)
                profile.data['result_freshness'] = value
                with patch('server.probe_service.selectors.DefaultSelector') as selector:
                    with self.assertRaisesRegex(ValueError, 'result_freshness'):
                        ProbeService(profile, self.service.out, {})
                    selector.assert_not_called()
        profile = copy.deepcopy(self.service.profile)
        profile.data['result_freshness'] = {'mode': 'post_blast_untrapped'}
        service = ProbeService(profile, self.service.out, {})
        self.addCleanup(service.selector.close)
        self.assertTrue(service.result_freshness)


if __name__ == '__main__':
    unittest.main()
