"""Bounded admission regressions; recorded KR controls are not authority oracles."""
import copy
import unittest

from server import probe_codec as codec
from server import test_report127_freshness as fixtures


class DeathEnvelopeTests(unittest.TestCase):
    setUp = fixtures.Report127FreshnessTests.setUp
    receive = fixtures.Report127FreshnessTests.receive

    # CP110 native front B seq280 and seq499; metadata/SHA in
    # docs/evidence/firstseen-admission/README.md. Inner values remain opaque.
    captured = [
        bytes.fromhex('495ba5a1a5f696ca5fe196ca539596ca539e96ca558d96ca5395a5695a'),
        bytes.fromhex('495ba5a1a5f696ca4a8096ca539596ca539e96ca558d693553955a965a'),
    ]

    def test_both_recorded_death_variants_preserve_exact_forwarded_tail(self):
        for body in self.captured:
            with self.subTest(body=body.hex()):
                self.sent.clear()
                result = self.receive(body, self.b)
                self.assertNotIn('rejected', result)
                self.assertEqual(len(self.sent), 1)
                self.assertEqual(self.sent[0]['body'], body[:6] + body[14:])
                self.assertIs(self.b['death_event'], result)

    def bad_bodies(self):
        good = self.captured[0]
        # High forward tick would poison the cursor if validation ran later.
        prefix = codec.pack_fields([19, 1, 200, 159, 0x7FFFFFFF, 0], [1, 1, 2, 2, 4, 4])
        for name, tail in [('missing', b''), ('truncated', good[14:-1]),
                           ('extra', good[14:] + b'\0'),
                           ('short_length', codec.pack_fields([10], [4]) + good[18:]),
                           ('long_length', codec.pack_fields([12], [4]) + good[18:])]:
            yield name, prefix + tail
        for name, raw in [('tag', bytes.fromhex('1706000000000000000000')),
                          ('inner_length', bytes.fromhex('1805000000000000000000')),
                          ('reader_flag', bytes.fromhex('1806000000010000000000'))]:
            values, widths = [], []
            offset = 0
            while offset < len(raw):
                width = 4 if len(raw) - offset >= 4 else 2 if len(raw) - offset >= 2 else 1
                values.append(int.from_bytes(raw[offset:offset + width], 'little'))
                widths.append(width)
                offset += width
            yield name, prefix + codec.pack_fields([11], [4]) + codec.pack_fields(values, widths)

    def test_bad_first_seen_or_forward_tick_cannot_poison_marker_cursor_or_history(self):
        for anchored in [False, True]:
            for name, body in self.bad_bodies():
                with self.subTest(anchored=anchored, case=name):
                    self.b.pop('report127_cursor', None)
                    if anchored:
                        fixtures.Report127FreshnessTests.receive(self, fixtures.Report127FreshnessTests.body(self, 39, 1000, slot=1), self.b)
                    cursor = copy.deepcopy(self.b.get('report127_cursor'))
                    history = len(self.service.state['event_reports'])
                    with self.assertRaisesRegex(ValueError, 'unsupported death'):
                        self.receive(body, self.b)
                    self.assertEqual(self.b.get('report127_cursor'), cursor)
                    self.assertNotIn('pickup_ineligible', self.b)
                    self.assertNotIn('death_event', self.b)
                    self.assertEqual(self.sent, [])
                    self.assertEqual(len(self.service.state['event_reports']), history)

    def test_bad_report_keeps_legitimate_pickup_and_later_death_admissible(self):
        _, bad = next(self.bad_bodies())
        with self.assertRaises(ValueError):
            self.receive(bad, self.b)
        pickup = codec.pack_fields([0, 1, 4, 3, 17, 2, 1234, self.service.room.key, 0],
                                   [1, 1, 1, 1, 4, 2, 4, 4, 1])
        self.service.receive_channel(self.b, codec.encode(bytes(37), 124, pickup, 103), 103, pickup)
        self.assertNotIn('rejected', self.service.state['item_requests'][-1])
        self.assertEqual(len(self.sent), 2)
        self.sent.clear()
        self.assertNotIn('rejected', self.receive(self.captured[0], self.b, seq=106))
        self.assertEqual(len(self.sent), 1)

    def test_well_formed_first_seen_claim_remains_outside_life_authority(self):
        # Structural validity alone still permits an untrusted first-seen death.
        # No bomb/location/life-state oracle is fabricated by this guard.
        self.assertNotIn('rejected', self.receive(self.captured[0], self.b))
        self.assertIsNotNone(self.b.get('death_event'))


if __name__ == '__main__':
    unittest.main()
