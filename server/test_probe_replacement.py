"""Regression for the measured A/B -> B/C diagnostic lifecycle, not full auth."""
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

from server import probe_codec as codec
from server.probe_service import ProbeService, ROLE_IDS, ROLE_SLOTS


class ReplacementTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        names = ['rank_Awin', 'promote_B', 'auth_C', 'roomlist_C', 'join_C',
                 'player_C', 'slot_C', 'guest_C', 'ready_C']
        profile = SimpleNamespace(frames={n: b'' for n in names}, data={'items': [], 'initial_key': 12345})
        self.service = ProbeService(profile, Path(temporary.name), {'A': 0, 'B': 0})
        self.addCleanup(self.service.selector.close)
        self.service.host_role = 'B'
        self.service.room.begin()
        self.old_bomb = self.service.room.bomb()
        self.service.room.finish()
        self.service.round = {'index': 1, 'refresh': {'monotonic_ns': 1}}
        self.a = {'role': 'A', 'reply_ordinal': 4, 'closed': {'reason': 'eof'}, 'map_response_sent': True}
        self.b = {'role': 'B', 'reply_ordinal': 4, 'map_response_sent': True, 's2c_next_seq': 106}
        self.c = {'role': 'C', 'reply_ordinal': 4, 'map_response_sent': True, 's2c_next_seq': 73}
        self.service.events = [self.a, self.b, self.c]
        self.service.channels = dict(A=self.a, B=self.b, C=self.c)
        self.sent = []
        def send(event, alias, label, **kwargs):
            seq = event['s2c_next_seq']
            event['s2c_next_seq'] += 3
            item = {'role': event['role'], 'alias': alias, 'seq': seq,
                    'monotonic_ns': 1, 'body': kwargs.get('body')}
            self.sent.append(item)
            return item
        self.service.send = send

    def dispatch(self, event, opcode, body=b''):
        frame = codec.encode(bytes(37), opcode, body, 40)
        self.service.receive_channel(event, frame, 40, body)

    def test_new_host_start_requires_current_guest_ready_and_new_roster_only(self):
        self.dispatch(self.b, 104)
        self.assertFalse(self.service.room.active)
        self.dispatch(self.c, 95)
        self.assertTrue(self.c['guest_ready_received'])
        self.assertEqual([s['role'] for s in self.sent], ['B', 'C'])
        self.assertTrue(all(s['alias'] == 'ready_C' for s in self.sent))
        self.dispatch(self.c, 104)
        self.assertFalse(self.service.room.active)
        self.dispatch(self.b, 104)
        self.assertEqual(self.service.round['host_role'], 'B')
        self.assertEqual(self.service.round['roster'],
                         [{'role': 'B', 'slot': 1, 'identity': 2}, {'role': 'C', 'slot': 0, 'identity': 3}])
        self.assertEqual(self.service.room.key, 12346)
        self.assertEqual(self.service.room.fence.rejection(self.old_bomb), 'stale_round')
        self.assertFalse(any(s['role'] == 'A' for s in self.sent))
        self.assertEqual(self.b['s2c_next_seq'], 115)
        self.assertEqual(self.c['s2c_next_seq'], 82)
        count = len(self.sent)
        self.dispatch(self.b, 104)
        self.assertEqual(len(self.sent), count)
        self.assertEqual(self.service.room.keys.issued_count, 2)

    def test_b_death_forwards_to_current_c_and_result_selects_slot1_loss(self):
        self.c['guest_ready_received'] = True
        self.dispatch(self.b, 104)
        # prefix + opaque measured serialized tail; this path preserves the tail.
        body = codec.pack_fields([19, 1, 120, 159, 1234, 0], [1, 1, 2, 2, 4, 4]) + b'preserved-tail'
        with patch('server.probe_service.time.monotonic_ns', return_value=1):
            self.service.receive_event(self.b, b'frame', 40, body)
        forwarded = self.sent[-1]
        self.assertEqual(forwarded['role'], 'C')
        self.assertEqual(forwarded['body'], body[:6] + b'preserved-tail')
        self.service.state['event_reports'].append({'role': 'A', 'round': 2, 'subtype': 21, 'monotonic_ns': 2})
        with patch('server.probe_service.time.monotonic_ns', return_value=9 * 10**9):
            self.service.tick()
        self.assertNotIn('rank', self.service.round)
        self.service.state['event_reports'].append({'role': 'C', 'round': 2, 'subtype': 21, 'monotonic_ns': 2})
        with patch('server.probe_service.time.monotonic_ns', return_value=9 * 10**9):
            self.service.tick()
        self.assertEqual([s['alias'] for s in self.sent[-2:]], ['rank_Awin', 'rank_Awin'])
        self.assertEqual([s['role'] for s in self.sent[-2:]], ['B', 'C'])

    def test_replacement_admission_requires_departure_cleanup_and_live_b(self):
        self.assertTrue(self.service.replacement_allowed())
        self.b['closed'] = {'reason': 'eof'}
        self.assertFalse(self.service.replacement_allowed())
        self.b.pop('closed')
        self.service.round.pop('refresh')
        self.assertFalse(self.service.replacement_allowed())
        self.assertEqual((ROLE_SLOTS['A'], ROLE_SLOTS['C']), (0, 0))
        self.assertNotEqual(ROLE_IDS['A'], ROLE_IDS['C'])


if __name__ == '__main__':
    unittest.main()
