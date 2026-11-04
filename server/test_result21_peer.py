"""Bounded peer21 translation; native execution is verified in Wine separately."""
import copy
import unittest
from unittest.mock import patch

from server import probe_codec as codec
from server import test_result_freshness as fixture


class Result21PeerTests(unittest.TestCase):
    setUp = fixture.ResultFreshnessTests.setUp
    enable = fixture.ResultFreshnessTests.enable
    drain = fixture.ResultFreshnessTests.drain
    report = fixture.ResultFreshnessTests.report
    trap = fixture.ResultFreshnessTests.trap
    bomb = fixture.ResultFreshnessTests.bomb
    blast = fixture.ResultFreshnessTests.blast
    tick = fixture.ResultFreshnessTests.tick
    rank_sends = fixture.ResultFreshnessTests.rank_sends

    def clear_sends(self):
        return [row for row in self.sent if row.get('opcode') == 170
                and codec.read_fields(row['body'][:1], [1]) == [21]]

    def test_exact_accepted_prefix6_goes_to_other_peer_once_per_trap(self):
        self.drain()
        self.trap(10, 1)
        marker = copy.deepcopy(self.a['pickup_ineligible'])
        clear = self.report('A', 21, 11, 2)
        sent, = self.clear_sends()
        self.assertEqual(sent['role'], 'B')
        self.assertEqual(sent['body'], bytes.fromhex(clear['payload_hex'])[:6])
        self.assertEqual(len(sent['body']), 6)
        self.assertEqual(self.a['pickup_ineligible'], marker)
        self.assertEqual(self.a['result21_forward']['trap_sha256'], marker['report_sha256'])
        self.assertEqual(clear['forward170'], sent)
        self.assertNotIn('forward170', self.report('A', 21, 12, 3))
        self.assertEqual(len(self.clear_sends()), 1)
        self.trap(13, 4)
        self.report('A', 21, 14, 5)
        self.assertEqual(len(self.clear_sends()), 2)
        self.assertNotEqual(self.a['result21_forward']['trap_sha256'], marker['report_sha256'])

    def test_stale_and_duplicate_reports_cannot_create_another_transition(self):
        self.drain()
        self.trap(10, 1)
        self.report('A', 21, 11, 2)
        before = copy.deepcopy(self.a['result21_forward'])
        self.assertEqual(self.report('A', 21, 11, 3)['rejected'], 'duplicate_report_body')
        self.assertEqual(self.trap(9, 4)['rejected'], 'report_tick_not_forward')
        self.report('A', 21, 12, 5)
        self.assertEqual(self.a['result21_forward'], before)
        self.assertEqual(len(self.clear_sends()), 1)

    def test_missing_trap_and_legacy_or_nondrained_mode_do_not_forward21(self):
        self.report('A', 21, 1, 1)
        self.trap(2, 2)
        self.report('A', 21, 3, 3)
        self.enable()
        self.report('A', 21, 4, 4)
        self.assertEqual(self.clear_sends(), [])
        self.drain()
        self.service.room.finish()
        self.service.start_round()
        self.report('A', 21, 5, 5)
        self.assertEqual(self.clear_sends(), [])

    def test_start_resets_transition_but_preserves_report_cursor(self):
        self.drain()
        self.trap(10, 1)
        self.report('A', 21, 11, 2)
        cursor = copy.deepcopy(self.a['report127_cursor'])
        self.service.room.finish()
        self.service.start_round()
        self.assertNotIn('result21_forward', self.a)
        self.assertEqual(self.a['report127_cursor'], cursor)
        self.assertEqual(self.trap(10, 3)['rejected'], 'report_tick_not_forward')
        self.report('A', 21, 12, 4)
        self.assertEqual(len(self.clear_sends()), 1)
        self.trap(13, 5)
        self.report('A', 21, 14, 6)
        self.assertEqual(len(self.clear_sends()), 2)

    def test_failed_send_does_not_commit_transition_and_later_fresh21_can_retry(self):
        self.drain()
        self.trap(10, 1)
        with patch.object(self.service, 'send', side_effect=BrokenPipeError('partial17021')):
            with self.assertRaises(BrokenPipeError):
                self.report('A', 21, 11, 2)
        self.assertNotIn('result21_forward', self.a)
        self.assertNotIn('forward170', self.service.state['event_reports'][-1])
        self.report('A', 21, 12, 3)
        self.assertEqual(len(self.clear_sends()), 1)

    def test_current_death19_cannot_be_hidden_by_later17_or21(self):
        self.drain()
        self.report('A', 19, 10, 1)
        self.trap(11, 2)
        self.report('A', 21, 12, 3)
        self.assertEqual(self.clear_sends(), [])

    def test_closed_round_or_missing_live_other_peer_cannot_receive_transition(self):
        self.drain()
        self.trap(10, 1)
        self.b['closed'] = {'reason': 'fixture'}
        self.report('A', 21, 11, 2)
        self.assertNotIn('result21_forward', self.a)
        self.b.pop('closed')
        self.service.room.finish()
        self.report('A', 21, 12, 3)
        self.assertEqual(self.clear_sends(), [])
        self.assertEqual(len(self.service.state['inactive_reports']), 1)

    def test_retained_old21_cannot_clear_a_newer_trap_after_blast(self):
        self.drain()
        old = self.report('A', 21, 10, 1)
        self.trap(11, 2)
        self.blast(3)
        self.service.forward_result21(self.a, old, bytes.fromhex(old['payload_hex']))
        self.assertEqual(self.clear_sends(), [])
        self.report('A', 21, 12, 4)
        self.assertEqual(len(self.clear_sends()), 1)


if __name__ == '__main__':
    unittest.main()
