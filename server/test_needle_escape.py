"""Bounded opt-in report-pair policy. Native compatibility is checked separately."""
import copy
from pathlib import Path
import socket
from types import SimpleNamespace
import unittest

from server import probe_codec as codec
from server.probe_service import ProbeService


class NeedleEscapeTests(unittest.TestCase):
    # CP037 needle-forward actual A124/17/1/0, seq175/205/208/211.
    pickup = bytes.fromhex('5a5a5e5996ca53bca5f896ca5abc96ca63ac5a')
    trap = bytes.fromhex('4b5aa511a46496ca59fb96ca539096ca539b96ca5a8d96c8539596ca5395a569')
    escape = bytes.fromhex('5b5aa511a46496ca593596ca5390')
    use = bytes.fromhex('5a5aa511a46496ca593596ca53935ba5f85a96ca5395')

    def profile(self, option=True):
        aliases = ['rank_Awin', 'promote_B', 'auth_C', 'roomlist_C', 'join_C',
                   'player_C', 'slot_C', 'guest_C', 'ready_C']
        data = {'items': [dict(index=41, row=4, column=3, kind=145)], 'initial_key': 12345}
        if option:
            data['needle_escape'] = {'mode': 'needle_145_report_pair'}
        return SimpleNamespace(frames=dict.fromkeys(aliases), data=data, body=lambda alias: b'')

    def setUp(self):
        self.s = ProbeService(self.profile(), Path('/tmp'), {})
        self.addCleanup(self.s.selector.close)
        self.a = dict(role='A', reply_ordinal=4, global_ordinal=1,
                      map_response_sent=True, create_response_sent=True)
        self.b = dict(role='B', reply_ordinal=4, global_ordinal=2,
                      map_response_sent=True, create_response_sent=True)
        self.s.events = [self.a, self.b]
        self.s.channels = dict(A=self.a, B=self.b)
        self.sent = []
        self.s.send = self.send
        self.s.start_round()
        self.sent.clear()

    def send(self, event, alias, label, **kwargs):
        row = dict(role=event['role'], alias=alias, label=label, **kwargs)
        self.sent.append(row)
        return row

    def dispatch(self, body, op=127, *, event=None, seq=100):
        event = self.a if event is None else event
        self.s.receive_channel(event, codec.encode(bytes(37), op, body, seq), seq, body)
        return self.s.state['item_requests' if op == 124 else 'event_reports'][-1]

    def ready_pair(self):
        self.dispatch(self.pickup, 124)
        self.dispatch(self.trap)
        self.sent.clear()
        self.dispatch(self.escape)

    def prefix(self, body, *, tick=None, x=None, subtype=None):
        fields = codec.read_fields(body[:14], [1, 1, 2, 2, 4, 4])
        for index, value in [(0, subtype), (2, x), (4, tick)]:
            if value is not None:
                fields[index] = value
        return codec.pack_fields(fields, [1, 1, 2, 2, 4, 4]) + body[14:]

    def test_actual_pair_stages_then_forwards_exact_prefix_once_and_unblocks(self):
        self.ready_pair()
        self.assertEqual(self.sent, [])
        self.assertEqual(self.a['needle_inventory']['count'], 1)
        self.assertEqual(self.a['pickup_ineligible']['subtype'], 17)
        self.assertEqual(self.a['needle_pending']['prefix_hex'], self.escape[:6].hex())
        report = self.dispatch(self.use)
        self.assertNotIn('rejected', report)
        self.assertEqual([(q['role'], q['opcode'], q['body']) for q in self.sent],
                         [('B', 170, self.escape[:6])])
        self.assertEqual(self.a['needle_inventory']['count'], 0)
        self.assertNotIn('needle_pending', self.a)
        self.assertNotIn('pickup_ineligible', self.a)
        self.assertEqual(report['needle_consumed']['before'], 1)
        self.assertEqual(self.dispatch(self.use, seq=103)['rejected'], 'duplicate_report_body')
        self.assertEqual(len(self.sent), 1)

    def test_absent_option_retains_old_no_forward_no_inventory_behavior(self):
        self.s.needle_escape = False
        self.ready_pair()
        self.dispatch(self.use)
        self.assertEqual(self.sent, [])
        self.assertNotIn('needle_inventory', self.a)
        self.assertEqual(self.a['pickup_ineligible']['subtype'], 17)

    def test_no_credit_rejects_forward_tick_without_poisoning_cursor(self):
        self.dispatch(self.trap)
        self.sent.clear()
        cursor = copy.deepcopy(self.a['report127_cursor'])
        report = self.dispatch(self.prefix(self.escape, tick=0x7FFFFFFF))
        self.assertEqual(report['rejected'], 'needle_no_credit')
        self.assertEqual(self.a['report127_cursor'], cursor)
        self.assertNotIn('needle_pending', self.a)
        self.assertEqual(self.sent, [])
        self.assertNotIn('rejected', self.dispatch(self.prefix(self.trap, tick=2700)))

    def test_unmatched_use_bad_tail_and_pair_context_cannot_consume(self):
        self.dispatch(self.pickup, 124)
        self.dispatch(self.trap)
        self.sent.clear()
        cursor = copy.deepcopy(self.a['report127_cursor'])
        self.assertEqual(self.dispatch(self.use)['rejected'], 'needle_pair_mismatch')
        self.assertEqual(self.a['report127_cursor'], cursor)
        self.dispatch(self.escape)
        cursor = copy.deepcopy(self.a['report127_cursor'])
        for bad, reason in [(self.use[:-1], 'unsupported_needle_use_shape'),
                            (self.use[:14] + codec.pack_fields([1, 117, 0, 0], [1, 2, 1, 4]),
                             'unsupported_needle_use_shape'),
                            (self.prefix(self.use, tick=2721), 'needle_pair_mismatch'),
                            (self.prefix(self.use, x=121), 'needle_pair_mismatch')]:
            with self.subTest(reason=reason):
                self.assertEqual(self.dispatch(bad)['rejected'], reason)
                self.assertEqual(self.a['report127_cursor'], cursor)
                self.assertEqual(self.a['needle_inventory']['count'], 1)
                self.assertEqual(self.sent, [])
        self.assertNotIn('rejected', self.dispatch(self.use))

    def test_wrong_connection_cannot_claim_or_use_another_members_credit(self):
        self.ready_pair()
        outsider = dict(self.a)
        cursor = copy.deepcopy(outsider['report127_cursor'])
        self.assertEqual(self.dispatch(self.use, event=outsider)['rejected'],
                         'needle_not_current_AB_member')
        self.assertEqual(outsider['report127_cursor'], cursor)
        self.assertEqual(self.sent, [])
        other_prefix = bytearray(self.escape)
        other_prefix[1:2] = codec.pack_fields([1], [1])
        self.assertEqual(self.dispatch(bytes(other_prefix), event=self.b)['rejected'],
                         'needle_no_current_trap')
        self.assertEqual(self.a['needle_inventory']['count'], 1)

    def test_duplicate_pickup_cannot_grant_an_extra_credit(self):
        self.dispatch(self.pickup, 124)
        self.assertEqual(self.dispatch(self.pickup, 124, seq=103)['rejected'],
                         'claim_not_current_or_already_consumed')
        self.assertEqual(self.a['needle_inventory']['count'], 1)
        self.assertEqual(len(self.sent), 2)

    def test_failed_pickup_broadcast_does_not_grant_credit(self):
        def fail(*args, **kwargs):
            raise BrokenPipeError('measured dispatch model failure')
        self.s.send = fail
        with self.assertRaises(BrokenPipeError):
            self.dispatch(self.pickup, 124)
        self.assertNotIn('needle_inventory', self.a)

    def test_failed_escape_send_keeps_credit_trap_candidate_and_cursor(self):
        self.ready_pair()
        before = copy.deepcopy(self.a)
        def fail(*args, **kwargs):
            raise BrokenPipeError('measured dispatch model failure')
        self.s.send = fail
        with self.assertRaises(BrokenPipeError):
            self.dispatch(self.use)
        self.assertEqual(self.a, before)
        self.assertEqual(self.sent, [])
        # Direct model retry only: real service exception handling closes the socket.
        self.s.send = self.send
        self.dispatch(self.use, seq=103)
        self.assertEqual(self.a['needle_inventory']['count'], 0)

    def test_new_trap_cancels_pending_and_cannot_reuse_old_pair(self):
        self.ready_pair()
        new = self.prefix(self.trap, tick=2721)
        self.dispatch(new)
        self.assertNotIn('needle_pending', self.a)
        self.sent.clear()
        self.assertIn('rejected', self.dispatch(self.use))
        self.assertEqual(self.sent, [])
        self.assertEqual(self.a['needle_inventory']['count'], 1)

    def test_death_cancels_pending_and_cannot_be_hidden_by_a_later_trap(self):
        self.ready_pair()
        death = codec.pack_fields([19, 0, 120, 269, 2721, 0], [1, 1, 2, 2, 4, 4])
        death += codec.pack_fields([11, int.from_bytes(bytes.fromhex('18060000'), 'little'),
                                    0, 0, 0], [4, 4, 4, 2, 1])
        self.dispatch(death)
        self.assertNotIn('needle_pending', self.a)
        self.dispatch(self.prefix(self.trap, tick=2722))
        self.sent.clear()
        self.assertEqual(self.dispatch(self.prefix(self.escape, tick=2723))['rejected'],
                         'needle_actor_report_dead')
        self.assertEqual(self.sent, [])
        self.assertEqual(self.a['needle_inventory']['count'], 1)

    def test_start_reset_clears_credit_pending_but_keeps_connection_report_cursor(self):
        self.ready_pair()
        cursor = copy.deepcopy(self.a['report127_cursor'])
        self.s.room.finish()
        self.s.start_round()
        self.assertNotIn('needle_inventory', self.a)
        self.assertNotIn('needle_pending', self.a)
        self.assertEqual(self.a['report127_cursor'], cursor)
        self.assertIn('rejected', self.dispatch(self.use))

    def test_closed_peer_and_outsider_cannot_resolve_pending_pair(self):
        self.ready_pair()
        self.b['closed'] = {'reason': 'model'}
        cursor = copy.deepcopy(self.a['report127_cursor'])
        self.assertEqual(self.dispatch(self.use)['rejected'], 'needle_not_current_AB_member')
        self.assertEqual(self.a['report127_cursor'], cursor)
        self.assertEqual(self.a['needle_inventory']['count'], 1)
        self.assertEqual(self.sent, [])

    def test_close_and_accepted_leave_clear_credit_and_pending(self):
        self.ready_pair()
        left, right = socket.socketpair()
        self.addCleanup(right.close)
        self.s.connections[1] = dict(socket=left, event=self.a)
        self.s.close_connection(1, 'model')
        self.assertNotIn('needle_inventory', self.a)
        self.assertNotIn('needle_pending', self.a)
        self.assertFalse(self.s.room.active)
        self.b['needle_inventory'] = {'generation': self.s.room.generation, 'count': 1}
        self.b['needle_pending'] = {'generation': self.s.room.generation}
        self.s.round['refresh'] = {'monotonic_ns': 1}
        # Keep an idle survivor for the measured guest Back membership variant.
        self.a.pop('closed')
        frame = codec.encode(bytes(37), 86, b'', 103)
        self.s.leave_room(self.b, frame, 103, b'')
        self.assertNotIn('rejected', self.s.state['room_leave_requests'][-1])
        self.assertNotIn('needle_inventory', self.b)
        self.assertNotIn('needle_pending', self.b)

    def test_profile_and_roster_bounds_fail_before_round_allocation(self):
        for data in [None, {}, {'mode': 'all_items'}, {'mode': 'needle_145_report_pair', 'extra': 1}]:
            profile = self.profile()
            profile.data['needle_escape'] = data
            with self.assertRaisesRegex(ValueError, 'unsupported needle_escape'):
                ProbeService(profile, Path('/tmp'), {})
        for catalog in [[], [dict(index=1, row=1, column=1, kind=145),
                            dict(index=2, row=2, column=1, kind=145)]]:
            profile = self.profile()
            profile.data['items'] = catalog
            with self.assertRaisesRegex(ValueError, 'exactly one'):
                ProbeService(profile, Path('/tmp'), {})
        self.s.room.finish()
        self.b.pop('map_response_sent')
        before = self.s.room.generation
        with self.assertRaisesRegex(ValueError, 'live mapped A/B'):
            self.s.start_round()
        self.assertEqual(self.s.room.generation, before)


if __name__ == '__main__':
    unittest.main()
