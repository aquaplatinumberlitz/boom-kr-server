"""Guest membership policy regressions, not an oracle for the original KR server."""
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

from server import probe_codec as codec
from server.probe_service import ProbeService


class GuestRejoinTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        names = ['rank_Awin', 'promote_B', 'auth_C', 'roomlist_C', 'join_C',
                 'player_C', 'slot_C', 'guest_C', 'ready_C']
        profile = SimpleNamespace(frames=dict.fromkeys(names),
                                  data={'items': [], 'initial_key': 12345},
                                  raw=lambda _: b'', body=lambda _: b'fixture')
        self.service = ProbeService(profile, Path(temporary.name), {})
        self.addCleanup(self.service.selector.close)
        self.a = {'role': 'A', 'global_ordinal': 4, 'reply_ordinal': 4,
                  'create_response_sent': {'monotonic_ns': 1},
                  'map_response_sent': {'monotonic_ns': 1}, 's2c_next_seq': 73}
        self.b = {'role': 'B', 'global_ordinal': 8, 'reply_ordinal': 4,
                  'create_response_sent': {'monotonic_ns': 1},
                  'player_response_sent': {'monotonic_ns': 1},
                  'slot_response_sent': {'monotonic_ns': 1},
                  'guest_response_sent': {'monotonic_ns': 1},
                  'map_response_sent': {'monotonic_ns': 1}, 's2c_next_seq': 73}
        self.service.events.extend([self.a, self.b])
        self.service.channels.update(A=self.a, B=self.b)
        self.sent = []
        def send(event, alias, label, **kwargs):
            row = {'role': event['role'], 'alias': alias, 'seq': event['s2c_next_seq'],
                   'monotonic_ns': 10**10, 'body': kwargs.get('body')}
            event['s2c_next_seq'] += 3
            self.sent.append(row)
            return row
        self.service.send = send
        self.service.send_raw = lambda *_: self.fail('rejoin must encode current sequence')

    def dispatch(self, event, opcode, sequence=106):
        with patch('server.probe_service.time.monotonic_ns', return_value=10**10):
            self.service.receive_channel(event, codec.encode(bytes(37), opcode, b'', sequence), sequence, b'')

    def tick(self, seconds):
        with patch('server.probe_service.time.monotonic_ns', return_value=seconds * 10**9):
            self.service.tick()

    def test_guest_leave_clears_only_membership_not_transport_and_removes_peer_slot(self):
        self.dispatch(self.b, 86)
        self.assertNotIn('create_response_sent', self.b)
        self.assertNotIn('map_response_sent', self.b)
        self.assertNotIn('closed', self.b)
        self.assertEqual(self.service.host_role, 'A')
        self.assertEqual([x['role'] for x in self.service.peers()], ['A'])
        self.assertEqual([(x['role'], x['alias']) for x in self.sent], [('A', 'start'), ('A', 'slot_A')])
        self.assertEqual(codec.read_fields(self.sent[0]['body'], [1, 4, 1]), [1, 2, 0])
        self.assertEqual(self.b['last_room_leave']['previous_membership']['create_response_sent'], {'monotonic_ns': 1})
        self.assertEqual(self.service.room.keys.issued_count, 0)
        self.dispatch(self.b, 86)
        self.assertEqual(len(self.sent), 2)
        self.assertEqual(self.service.state['room_leave_requests'][-1]['rejected'], 'not_room_member')

    def test_rejoin_refreshes_both_rosters_and_enables_ready_then_current_host_start(self):
        self.dispatch(self.b, 86)
        self.tick(11)
        self.dispatch(self.b, 82, 136)
        self.dispatch(self.b, 585, 139)
        self.tick(12)
        self.tick(13)
        self.tick(14)
        self.assertEqual([(x['role'], x['alias']) for x in self.sent], [
            ('A', 'start'), ('A', 'slot_A'), ('B', 'ready'), ('B', 'roomlist_B'),
            ('B', 'join'), ('B', 'player_B'), ('A', 'guest_A'), ('B', 'slot_B'),
            ('B', 'guest_B'), ('B', 'table'), ('B', 'map')])
        self.assertTrue(self.b['map_response_sent'])
        self.assertEqual([x['role'] for x in self.service.peers()], ['A', 'B'])
        self.assertEqual(self.a['s2c_next_seq'], 82)
        self.assertEqual(self.b['s2c_next_seq'], 97)
        self.dispatch(self.a, 104)
        self.assertFalse(self.service.room.active)
        self.dispatch(self.b, 95)
        self.dispatch(self.a, 104)
        self.assertTrue(self.service.room.active)
        self.assertEqual(self.service.room.key, 12345)
        self.assertEqual(self.service.round['roster'], [
            {'role': 'A', 'slot': 0, 'identity': 1}, {'role': 'B', 'slot': 1, 'identity': 2}])
        self.assertEqual(self.service.room.keys.issued_count, 1)

    def test_early_join_or_changed_host_cannot_reestablish_membership(self):
        self.dispatch(self.b, 86)
        self.dispatch(self.b, 82)
        self.assertEqual(self.service.state['room_reentry_rejections'][-1]['reason'], 'lobby_refresh_pending')
        self.tick(11)
        self.a['closed'] = {'reason': 'eof'}
        self.dispatch(self.b, 82)
        self.assertEqual(self.service.state['room_reentry_rejections'][-1]['reason'], 'host_membership_changed')
        self.assertNotIn('create_response_sent', self.b)
        self.assertFalse(any(x['alias'] == 'join' for x in self.sent))

    def test_missing_profile_metadata_does_not_remove_guest_or_send_partial_cleanup(self):
        def body(alias):
            if alias == 'guest_A':
                raise ValueError('fixture does not declare sequence: guest_A')
            return b'fixture'
        self.service.profile.body = body
        self.dispatch(self.b, 86)
        self.assertEqual(self.service.state['room_leave_requests'][-1]['rejected'], 'unsupported_reentry_profile')
        self.assertIn('map_response_sent', self.b)
        self.assertEqual(self.sent, [])


if __name__ == '__main__':
    unittest.main()
