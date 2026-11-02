"""Connection-local diagnostic admission; these tests are not native RE evidence."""
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch
import hashlib
import struct
import tempfile
import unittest

from server import probe_codec as codec
from server.probe_service import ProbeService


class Report127FreshnessTests(unittest.TestCase):
    def setUp(self):
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        aliases = ['rank_Awin', 'promote_B', 'auth_C', 'roomlist_C', 'join_C',
                   'player_C', 'slot_C', 'guest_C', 'ready_C']
        profile = SimpleNamespace(frames=dict.fromkeys(aliases, b''), data={
            'items': [dict(index=17, row=4, column=3, kind=2)], 'initial_key': 12345})
        self.service = ProbeService(profile, Path(tmp.name), {})
        self.addCleanup(self.service.selector.close)
        self.a = dict(role='A', reply_ordinal=4, map_response_sent=True)
        self.b = dict(role='B', reply_ordinal=4, map_response_sent=True)
        self.service.events = [self.a, self.b]
        self.service.channels = dict(A=self.a, B=self.b)
        self.sent = []
        def send(event, alias, label, **kwargs):
            result = dict(role=event['role'], alias=alias, label=label,
                          monotonic_ns=1, **kwargs)
            self.sent.append(result)
            return result
        self.service.send = send
        self.service.start_round()
        self.sent.clear()

    def body(self, subtype, tick, *, slot=0, x=120):
        body = codec.pack_fields([subtype, slot, x, 159, tick, 0], [1, 1, 2, 2, 4, 4])
        if subtype == 17:
            blob = bytes.fromhex('180900000000') + struct.pack('<II', 0, 7)
            body += codec.pack_fields([14], [4])
            body += codec.pack_fields([int.from_bytes(blob[i:i + 4], 'little')
                                       for i in range(0, len(blob), 4)], [4, 4, 4, 2])
        elif subtype == 19:
            # This test does not establish the death-tail contract. Admission
            # preserves the existing service's opaque-tail behavior.
            body += b'fixture-tail'
        return body

    def receive(self, body, event=None, seq=100):
        frame = codec.encode(bytes(37), 127, body, seq)
        self.service.receive_channel(self.a if event is None else event, frame, seq, body)
        return self.service.state['event_reports'][-1]

    def test_seen_old_reports_cannot_poison_new_round_before_first_report(self):
        trap, death = self.body(17, 100), self.body(19, 110)
        self.receive(trap)
        self.receive(death)
        self.service.room.finish()
        self.service.start_round()
        self.sent.clear()
        for body, reason in [(trap, 'report_tick_not_forward'), (death, 'duplicate_report_body')]:
            self.assertEqual(self.receive(body)['rejected'], reason)
        self.assertNotIn('pickup_ineligible', self.a)
        self.assertNotIn('death_event', self.a)
        self.assertEqual(self.sent, [])
        pickup = codec.pack_fields([0, 0, 4, 3, 17, 2, 120, self.service.room.key, 0],
                                   [1, 1, 1, 1, 4, 2, 4, 4, 1])
        self.service.receive_channel(self.a, codec.encode(bytes(37), 124, pickup, 130), 130, pickup)
        self.assertNotIn('rejected', self.service.state['item_requests'][-1])
        self.assertEqual(len(self.sent), 2)

    def test_duplicate_body_at_new_wire_sequence_does_not_refresh_death_or_forward(self):
        body = self.body(19, 110)
        first = self.receive(body)
        self.assertEqual(len(self.sent), 1)
        original_marker = dict(self.a['pickup_ineligible'])
        duplicate = self.receive(body, seq=103)
        self.assertNotEqual(first['frame_sha256'], duplicate['frame_sha256'])
        self.assertEqual(duplicate['body_sha256'], hashlib.sha256(body).hexdigest())
        self.assertEqual(duplicate['rejected'], 'duplicate_report_body')
        self.assertIs(self.a['death_event'], first)
        self.assertEqual(self.a['pickup_ineligible'], original_marker)
        self.assertEqual(len(self.sent), 1)

    def test_equal_tick_distinct_reports_and_independent_peer_clocks_are_allowed(self):
        for body in [self.body(39, 1000, x=140), self.body(39, 1000, x=160),
                     self.body(17, 1000)]:
            self.assertNotIn('rejected', self.receive(body))
        self.assertNotIn('rejected', self.receive(self.body(21, 1, slot=1), self.b))
        self.assertEqual(self.b['report127_cursor']['tick'], 1)

    def test_unsigned_wrap_and_ambiguous_half_range_keep_last_good_cursor(self):
        self.receive(self.body(39, 0xFFFFFFFE))
        self.assertNotIn('rejected', self.receive(self.body(39, 1)))
        for tick in [0xFFFFFFFF, 0x80000001]:
            self.assertEqual(self.receive(self.body(39, tick))['rejected'], 'report_tick_not_forward')
            self.assertEqual(self.a['report127_cursor']['tick'], 1)
        self.assertNotIn('rejected', self.receive(self.body(39, 2)))

    def test_invalid_slot_or_trap_does_not_advance_cursor(self):
        self.receive(self.body(39, 100))
        for bad in [self.body(17, 10000)[:14], self.body(19, 10000, slot=1)]:
            with self.assertRaises(ValueError):
                self.receive(bad)
            self.assertEqual(self.a['report127_cursor']['tick'], 100)
        self.assertNotIn('rejected', self.receive(self.body(17, 101)))

    def test_rejected_peer21_cannot_finish_round(self):
        self.receive(self.body(39, 100, slot=1), self.b)
        death = self.receive(self.body(19, 110))
        death['monotonic_ns'] = 0
        self.assertEqual(self.receive(self.body(21, 90, slot=1), self.b)['rejected'],
                         'report_tick_not_forward')
        with patch('server.probe_service.time.monotonic_ns', return_value=10_000_000_000):
            self.service.tick()
        self.assertTrue(self.service.room.active)
        self.assertNotIn('rank', self.service.round)
        self.receive(self.body(21, 101, slot=1), self.b)
        with patch('server.probe_service.time.monotonic_ns', return_value=10_000_000_000):
            self.service.tick()
        self.assertFalse(self.service.room.active)
        self.assertIn('rank', self.service.round)

    def test_first_seen_or_forged_forward_tick_remains_outside_policy(self):
        # Explicit limits: no round key/authentication in prefix14 and no clock
        # anchor for a fresh connection. This guard cannot infer generation.
        self.assertNotIn('rejected', self.receive(self.body(39, 1000)))
        fresh = dict(role='A')
        self.assertNotIn('rejected', self.receive(self.body(19, 1), fresh))
        self.assertNotIn('rejected', self.receive(self.body(19, 1001)))


if __name__ == '__main__':
    unittest.main()
