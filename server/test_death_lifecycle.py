"""Optional emulator admission, with captured-envelope compatibility elsewhere."""
import copy
from pathlib import Path
import unittest

from server import probe_codec as codec
from server import test_report127_freshness as reports
from server import test_needle_escape as needle
from server import test_rescue_contact as rescue
from server.probe_service import ProbeService


class DeathLifecycleTests(unittest.TestCase):
    setUp = reports.Report127FreshnessTests.setUp
    body = reports.Report127FreshnessTests.body
    receive = reports.Report127FreshnessTests.receive

    def enable(self):
        self.service.death_lifecycle = True

    def assert_rejected_unchanged(self, body, reason, event=None):
        event = self.a if event is None else event
        before = copy.deepcopy(event)
        sent = len(self.sent)
        row = self.receive(body, event)
        self.assertEqual(row['rejected'], reason)
        self.assertEqual(event, before)
        self.assertEqual(len(self.sent), sent)
        return row

    def test_first_seen_high_tick_rejection_does_not_poison_later_valid_death(self):
        self.enable()
        self.assert_rejected_unchanged(self.body(19, 0x7FFFFFFF), 'death_no_current_trap')
        trap = self.receive(self.body(17, 10))
        row = self.receive(self.body(19, 20))
        self.assertNotIn('rejected', row)
        self.assertEqual(row['death_lifecycle']['trap_sha256'], trap['frame_sha256'])
        self.assertIs(self.a['death_event'], row)
        self.assertEqual(len(self.sent), 2)

    def test_default_off_keeps_first_seen_behavior(self):
        self.assertFalse(self.service.death_lifecycle)
        self.assertNotIn('rejected', self.receive(self.body(19, 10)))
        self.assertIn('death_event', self.a)

    def test_stale_or_dead_marker_cannot_substitute_for_current17(self):
        self.enable()
        for marker in [dict(generation=0, subtype=17), dict(generation=1, subtype=19)]:
            self.a['pickup_ineligible'] = marker
            self.assert_rejected_unchanged(self.body(19, 100), 'death_no_current_trap')

    def test_new_trap_cannot_hide_already_accepted_death(self):
        self.enable()
        self.receive(self.body(17, 10))
        death = self.receive(self.body(19, 20))
        self.receive(self.body(17, 30))
        self.assert_rejected_unchanged(self.body(19, 40), 'death_already_accepted')
        self.assertIs(self.a['death_event'], death)

    def test_unbound_unmapped_and_closed_members_cannot_send_death(self):
        self.enable()
        self.receive(self.body(17, 10))
        fake = copy.deepcopy(self.a)
        self.assert_rejected_unchanged(self.body(19, 20), 'death_member_not_current', fake)
        for field, value in [('map_response_sent', False), ('closed', True)]:
            prior = dict(self.a)
            self.a[field] = value
            # Dispatch directly because a closed socket cannot receive normally.
            self.assert_rejected_unchanged(self.body(19, 20), 'death_member_not_current')
            self.a.clear()
            self.a.update(prior)

    def test_round_reset_requires_fresh17_but_preserves_connection_cursor(self):
        self.enable()
        self.receive(self.body(17, 10))
        self.receive(self.body(19, 20))
        cursor = copy.deepcopy(self.a['report127_cursor'])
        self.service.room.finish()
        self.service.start_round()
        self.assertEqual(self.a['report127_cursor'], cursor)
        self.assert_rejected_unchanged(self.body(19, 30), 'death_no_current_trap')
        self.receive(self.body(17, 31))
        self.assertNotIn('rejected', self.receive(self.body(19, 32)))

    def test_malformed_envelope_still_fails_before_log_or_cursor(self):
        self.enable()
        before = copy.deepcopy(self.a)
        reports_before = len(self.service.state['event_reports'])
        with self.assertRaises(ValueError):
            self.receive(self.body(19, 0x7FFFFFFF)[:-1])
        self.assertEqual(self.a, before)
        self.assertEqual(len(self.service.state['event_reports']), reports_before)
        self.assertEqual(self.sent, [])

    def test_send_failure_keeps_existing_exception_and_abort_boundary(self):
        self.enable()
        self.receive(self.body(17, 10))
        def fail(*args, **kwargs):
            raise BrokenPipeError('recording transport')
        self.service.send = fail
        with self.assertRaises(BrokenPipeError):
            self.receive(self.body(19, 20))
        # Existing accepted17/19 path commits cursor/marker before send. This
        # option does not introduce a whole-service transaction or fake death.
        self.assertNotIn('death_event', self.a)
        self.assertEqual(self.a['pickup_ineligible']['subtype'], 19)
        self.assertEqual(self.a['report127_cursor']['tick'], 20)

    def test_supported_profile_configuration_only(self):
        profile = self.service.profile
        for value in [None, True, {'mode': 'all_modes'}, {'mode': 'current_trap', 'extra': 1}]:
            profile.data['death_lifecycle'] = value
            with self.assertRaisesRegex(ValueError, 'death_lifecycle'):
                ProbeService(profile, Path('/tmp'), {})
        profile.data['death_lifecycle'] = {'mode': 'current_trap'}
        s = ProbeService(profile, Path('/tmp'), {})
        self.addCleanup(s.selector.close)
        self.assertTrue(s.death_lifecycle)

    def test_real_needle_commit_removes_trap_until_fresh17(self):
        f = needle.NeedleEscapeTests(methodName='test_actual_pair_stages_then_forwards_exact_prefix_once_and_unblocks')
        f.setUp()
        self.addCleanup(f.doCleanups)
        f.s.death_lifecycle = True
        f.ready_pair()
        f.dispatch(f.use)
        tick = f.a['report127_cursor']['tick']
        before = copy.deepcopy(f.a)
        row = f.dispatch(self.body(19, tick + 1))
        self.assertEqual(row['rejected'], 'death_no_current_trap')
        self.assertEqual(f.a, before)
        f.dispatch(self.body(17, tick + 2))
        self.assertNotIn('rejected', f.dispatch(self.body(19, tick + 3)))

    def test_real_rescue_commit_removes_trap_until_fresh17(self):
        f = rescue.RescueContactTests(methodName='test_actual18_exact_translation_clears_trap_after_delivery_and_preserves_credit')
        f.setUp()
        self.addCleanup(f.doCleanups)
        f.s.death_lifecycle = True
        f.trapped()
        f.dispatch(f.rescue)
        tick = f.a['report127_cursor']['tick']
        before = copy.deepcopy(f.a)
        row = f.dispatch(self.body(19, tick + 1))
        self.assertEqual(row['rejected'], 'death_no_current_trap')
        self.assertEqual(f.a, before)
        f.dispatch(self.body(17, tick + 2))
        self.assertNotIn('rejected', f.dispatch(self.body(19, tick + 3)))


if __name__ == '__main__':
    unittest.main()
