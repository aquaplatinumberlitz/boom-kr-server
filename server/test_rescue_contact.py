"""Admission/lifecycle policy tests; native/runtime compatibility is separate."""
import copy
import hashlib
from pathlib import Path
import unittest
from unittest.mock import patch

from server import probe_codec as codec
from server import probe_service as service
from server import test_needle_escape as needle_tests


class RescueContactTests(unittest.TestCase):
    # CP036 actual A196/199; captured, not generated compatibility oracles.
    trap = bytes.fromhex('4b5aa511a5f696ca59ae96ca539096ca539b96ca5a8d96c8539596ca5395a569')
    rescue = bytes.fromhex('485aa511a5f696ca591c96ca539096ca539f96ca568d96cb5395a569')
    reverse_trap = bytes.fromhex('4b5ba511a5f696ca57fe96ca539096ca539b96ca5a8d96c8539596ca5395a569')
    reverse_rescue = bytes.fromhex('485ba511a5f696ca574c96ca539096ca539f96ca568d96ca5395a569')

    def setUp(self):
        self.s = service.ProbeService(needle_tests.NeedleEscapeTests().profile(False), Path('/tmp'), {})
        self.addCleanup(self.s.selector.close)
        self.a = dict(role='A', reply_ordinal=4, global_ordinal=1, map_response_sent=True)
        self.b = dict(role='B', reply_ordinal=4, global_ordinal=2, map_response_sent=True)
        self.s.events = [self.a, self.b]
        self.s.channels = dict(A=self.a, B=self.b)
        self.sent = []
        def send(event, alias, label, **kwargs):
            row = dict(role=event['role'], alias=alias, label=label, **kwargs)
            self.sent.append(row)
            return row
        self.s.send = send
        self.s.start_round()
        self.s.rescue_contact = True
        self.sent.clear()

    def dispatch(self, body, event=None):
        event = self.a if event is None else event
        self.s.receive_channel(event, codec.encode(bytes(37), 127, body, 100), 100, body)
        return self.s.state['event_reports'][-1]

    def trapped(self):
        self.dispatch(self.trap)
        self.sent.clear()

    def test_actual18_exact_translation_clears_trap_after_delivery_and_preserves_credit(self):
        self.trapped()
        self.a['needle_inventory'] = dict(generation=1, count=1)
        self.a['needle_pending'] = dict(generation=1)
        row = self.dispatch(self.rescue)
        self.assertNotIn('rejected', row)
        self.assertEqual([(q['role'], q['opcode'], q['body']) for q in self.sent],
                         [('B', 170, self.rescue[:6] + codec.pack_fields([1], [1]))])
        self.assertNotIn('pickup_ineligible', self.a)
        self.assertNotIn('needle_pending', self.a)
        self.assertEqual(self.a['needle_inventory']['count'], 1)
        self.assertEqual(row['rescue_cleared']['aux'], 1)
        self.assertEqual(self.dispatch(self.rescue)['rejected'], 'duplicate_report_body')
        self.assertEqual(len(self.sent), 1)

    def test_default_disabled_records_without_clear_or_forward(self):
        self.s.rescue_contact = False
        self.trapped()
        self.dispatch(self.rescue)
        self.assertEqual(self.a['pickup_ineligible']['subtype'], 17)
        self.assertEqual(self.sent, [])

    def test_missing_stale_or_dead_trap_cannot_be_rescued(self):
        for trap in [None, dict(generation=0, subtype=17), dict(generation=1, subtype=19)]:
            if trap is not None:
                self.a['pickup_ineligible'] = trap
            self.assertEqual(self.dispatch(self.rescue)['rejected'], 'rescue_no_current_trap')
            self.assertNotIn('report127_cursor', self.a)
        self.assertEqual(self.sent, [])

    def test_malformed_envelope_and_u32_aux_are_not_narrowed_or_cursor_committed(self):
        self.trapped()
        before = copy.deepcopy(self.a)
        for body in [self.rescue[:-1], self.rescue + b'\0',
                     self.rescue[:14] + codec.pack_fields([11], [4]) + self.rescue[18:],
                     self.rescue[:18] + codec.pack_fields([0x518, 0x01010000, 0], [4, 4, 2]),
                     self.rescue[:18] + codec.pack_fields([0x518, 0x00020000, 0], [4, 4, 2])]:
            with self.subTest(body=body.hex()):
                self.assertIn('rejected', self.dispatch(body))
                self.assertEqual(self.a, before)
                self.assertEqual(self.sent, [])

    def test_send_failure_preserves_trap_cursor_credit_pending(self):
        self.trapped()
        self.a['needle_inventory'] = dict(generation=1, count=1)
        self.a['needle_pending'] = dict(generation=1)
        before = copy.deepcopy(self.a)
        def fail(*args, **kwargs):
            raise BrokenPipeError('direct model send failure')
        self.s.send = fail
        with self.assertRaises(BrokenPipeError):
            self.dispatch(self.rescue)
        self.assertEqual(self.a, before)

    def test_closed_peer_unmapped_or_replacement_cannot_enable_rescue(self):
        self.trapped()
        for field, value in [('closed', True), ('map_response_sent', False), ('role', 'C')]:
            prior = dict(self.b)
            self.b[field] = value
            self.assertEqual(self.dispatch(self.rescue)['rejected'], 'rescue_not_supported_AB_member')
            self.b.clear()
            self.b.update(prior)
        outsider = dict(self.a)
        self.assertEqual(self.dispatch(self.rescue, outsider)['rejected'], 'rescue_not_supported_AB_member')
        self.assertEqual(self.sent, [])

    def test_death_history_on_either_actor_blocks_even_with_later_trap(self):
        self.trapped()
        for member in [self.a, self.b]:
            member['death_event'] = dict(round=1)
            self.assertEqual(self.dispatch(self.rescue)['rejected'], 'rescue_member_report_dead')
            member.pop('death_event')
        self.assertEqual(self.sent, [])

    def test_reported_trapped_peer_is_not_eligible_rescuer(self):
        self.trapped()
        self.b['pickup_ineligible'] = dict(generation=1, subtype=17)
        self.assertEqual(self.dispatch(self.rescue)['rejected'], 'rescue_peer_report_ineligible')
        self.assertEqual(self.sent, [])

    def test_roster_sent_once_in_room_before_start_and_start_clears_trap(self):
        self.trapped()
        self.s.room.finish()
        self.s.prepare_rescue_roster()
        self.s.prepare_rescue_roster()
        self.s.start_round()
        self.assertNotIn('pickup_ineligible', self.a)
        for role in 'AB':
            rows = [r for r in self.sent if r['role'] == role]
            self.assertEqual(rows[0]['opcode'], 106)
            self.assertEqual(rows[0]['alias'], 'synthetic_opponent')
            self.assertEqual(rows[-1]['alias'], 'start')
        self.assertEqual(self.dispatch(self.rescue)['rejected'], 'rescue_no_current_trap')
        self.assertEqual([p['role'] for p in self.s.peers()], ['A', 'B'])

    def test_start_cannot_mutate_round_before_synthetic_roster_is_sent(self):
        self.s.room.finish()
        generation = self.s.room.generation
        with self.assertRaises(ValueError):
            self.s.start_round()
        self.assertEqual(self.s.room.generation, generation)
        self.assertFalse(self.s.room.active)

    def test_return_opponent_refresh_declares_slot2_ready_without_real95_or_trap_mutation(self):
        self.trapped()
        before = copy.deepcopy(self.a)
        result = self.s.refresh_rescue_opponent()
        self.assertEqual([(q['role'], q['opcode'], codec.read_fields(q['body'], [1, 4, 1]))
                          for q in result], [('A', 110, [2, 0x24, 0]), ('B', 110, [2, 0x24, 0])])
        self.assertEqual(self.a, before)
        self.assertNotIn('guest_ready_received', self.a)
        self.assertNotIn('guest_ready_received', self.b)
        self.s.rescue_contact = False
        self.sent.clear()
        self.assertEqual(self.s.refresh_rescue_opponent(), [])
        self.assertEqual(self.sent, [])

    def test_reverse_actual_aux0_clears_B_and_sends170_to_A(self):
        self.dispatch(self.reverse_trap, self.b)
        self.sent.clear()
        row = self.dispatch(self.reverse_rescue, self.b)
        self.assertNotIn('rejected', row)
        self.assertEqual(row['rescue_cleared']['aux'], 0)
        self.assertEqual(row['rescue_cleared']['peer_role'], 'A')
        self.assertEqual([(r['role'], r['body']) for r in self.sent],
                         [('A', self.reverse_rescue[:6] + codec.pack_fields([0], [1]))])
        self.assertNotIn('pickup_ineligible', self.b)
        self.assertEqual(self.dispatch(self.reverse_rescue, self.b)['rejected'], 'duplicate_report_body')

    def test_other_direction_aux_and_chief_are_not_supported(self):
        body = self.rescue[:1] + codec.pack_fields([1], [1]) + self.rescue[2:]
        self.assertEqual(self.dispatch(body, self.b)['rejected'], 'unsupported_rescue_envelope_or_aux')
        self.s.host_role = 'B'
        with self.assertRaisesRegex(ValueError, 'A chief'):
            self.s.start_round()

    def test_reverse_send_failure_preserves_B_trap_cursor_and_inventory(self):
        self.dispatch(self.reverse_trap, self.b)
        self.b['needle_inventory'] = dict(generation=1, count=1)
        before = copy.deepcopy(self.b)
        def fail(*args, **kwargs):
            raise BrokenPipeError('direct model reverse failure')
        self.s.send = fail
        with self.assertRaises(BrokenPipeError):
            self.dispatch(self.reverse_rescue, self.b)
        self.assertEqual(self.b, before)

    def test_profile_rejects_unknown_policy_conflicting_promotion_and_missing_fixed_roster(self):
        for policy in [{'mode': 'any'}, {'mode': 'friendly_AB_synthetic2_aux1'}]:
            p = needle_tests.NeedleEscapeTests().profile(False)
            p.data['rescue_contact'] = policy
            with self.assertRaises(ValueError):
                service.ProbeService(p, Path('/tmp'), {})
        p.data['host_back_rejoin'] = {'mode': 'idle_A_to_B'}
        with self.assertRaisesRegex(ValueError, 'policy'):
            service.ProbeService(p, Path('/tmp'), {})

    def test_profile_checks_all_declared_body_hashes(self):
        p = needle_tests.NeedleEscapeTests().profile(False)
        p.data['rescue_contact'] = {'mode': 'friendly_AB_synthetic2_aux1'}
        p.frames.update(dict.fromkeys(service.RESCUE_PROFILE_BODIES))
        p.body = lambda alias: alias.encode()
        hashes = {a: hashlib.sha256(a.encode()).hexdigest() for a in service.RESCUE_PROFILE_BODIES}
        # Portable policy test. Real CP036 hashes are joined separately in local evidence.
        with patch.object(service, 'RESCUE_PROFILE_BODIES', hashes):
            s = service.ProbeService(p, Path('/tmp'), {})
            s.selector.close()
            self.assertTrue(s.rescue_contact)
            p.body = lambda alias: b'changed'
            with self.assertRaisesRegex(ValueError, 'fixture'):
                service.ProbeService(p, Path('/tmp'), {})


if __name__ == '__main__':
    unittest.main()
