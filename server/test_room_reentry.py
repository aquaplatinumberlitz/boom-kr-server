"""Fixture membership/sequence regressions; native compatibility needs runtime evidence."""
from pathlib import Path
from types import SimpleNamespace
import tempfile
import unittest
from unittest.mock import patch

from server import probe_codec as codec
from server.probe_service import ProbeService, FixtureProfile


class RoomReentryTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        names = ['rank_Awin', 'promote_B', 'auth_C', 'roomlist_C', 'join_C',
                 'player_C', 'slot_C', 'guest_C', 'ready_C']
        opcodes = {'ready': 241, 'roomlist_A': 91, 'create': 86, 'player_A': 106,
                   'slot_A': 110, 'guest_A': 106, 'table': 238, 'map': 117}
        raw = lambda name: codec.encode(bytes(37), opcodes.get(name, 0), b'test', 55)
        profile = SimpleNamespace(frames=dict.fromkeys(names),
                                  data={'items': [], 'initial_key': 12345},
                                  raw=raw, body=lambda name: b'test')
        self.service = ProbeService(profile, Path(temporary.name), {})
        self.addCleanup(self.service.selector.close)
        self.a = {'role': 'A', 'global_ordinal': 4, 'reply_ordinal': 4,
                  'create_response_sent': {'monotonic_ns': 1},
                  'player_response_sent': {'monotonic_ns': 1},
                  'slot_response_sent': {'monotonic_ns': 1},
                  'guest_response_sent': {'monotonic_ns': 1},
                  'map_response_sent': {'monotonic_ns': 1},
                  'departures_sent': ['B'], 's2c_next_seq': 103}
        self.b = {'role': 'B', 'reply_ordinal': 4, 'create_response_sent': True,
                  'closed': {'reason': 'eof'}}
        self.service.events.extend([self.a, self.b])
        self.service.channels.update(A=self.a, B=self.b)
        self.service.room.begin()
        self.ticket = self.service.room.bomb()
        self.service.room.finish()
        self.service.round = {'index': 1, 'refresh': {'monotonic_ns': 1}}
        self.raw_sends = []
        self.service.send_raw = lambda event, frame: self.raw_sends.append((event['role'], frame))

    def dispatch(self, opcode, body=b'', sequence=205):
        frame = codec.encode(bytes(37), opcode, body, sequence)
        with patch('server.probe_service.time.monotonic_ns', return_value=10**10):
            self.service.receive_channel(self.a, frame, sequence, body)

    def tick(self, seconds):
        with patch('server.probe_service.time.monotonic_ns', return_value=seconds * 10**9):
            self.service.tick()

    def test_leave_archives_membership_without_resetting_channel_or_round_history(self):
        room = self.service.room
        self.dispatch(86)
        self.assertNotIn('create_response_sent', self.a)
        self.assertNotIn('map_response_sent', self.a)
        self.assertEqual(self.service.peers(), [])
        visit = self.a['room_visits'][0]
        self.assertEqual(visit['previous_membership']['departures_sent'], ['B'])
        self.assertEqual(visit['seq'], 205)
        self.assertIs(self.service.room, room)
        self.assertEqual((room.generation, room.key, room.keys.issued_count), (1, 12345, 1))
        self.assertEqual(room.fence.rejection(self.ticket), 'round_closed')
        self.assertEqual((self.a['global_ordinal'], self.a['s2c_next_seq']), (4, 103))
        self.assertNotIn('closed', self.a)
        self.dispatch(86)
        self.assertEqual(self.service.state['room_leave_requests'][-1]['rejected'], 'not_room_member')
        self.assertEqual(len(self.a['room_visits']), 1)

    def test_recreate_uses_continuing_sequence_and_skips_closed_guest_profile(self):
        self.dispatch(86)
        self.tick(11)
        self.dispatch(81, b'actual-body-placeholder', 208)
        self.dispatch(585, b'profile-request', 211)
        self.tick(12)  # simulated receive timestamps are 10s
        self.tick(13)
        self.tick(14)
        ops = [int.from_bytes(raw[5:9], 'little') for _, raw in self.raw_sends]
        self.assertEqual(ops, [241, 91, 86, 106, 110, 238, 117])
        self.assertTrue(self.a['guest_response_sent']['skipped_closed_guest'])
        self.assertEqual([role for role, _ in self.raw_sends], ['A'] * 7)
        for index, (_, raw) in enumerate(self.raw_sends):
            self.assertEqual(codec.frame_body(raw, 103 + index * 3), b'test')
        self.assertEqual(self.a['s2c_next_seq'], 124)
        self.assertTrue(self.a['map_response_sent'])
        self.assertEqual([peer['role'] for peer in self.service.peers()], ['A'])
        before = list(self.raw_sends)
        self.dispatch(81, b'duplicate')
        self.dispatch(104)
        self.tick(20)
        self.assertEqual(self.raw_sends, before)
        self.assertFalse(self.service.room.active)
        self.assertEqual(self.service.room.keys.issued_count, 1)

    def test_refresh_pending_does_not_accept_create_or_stale_profile_request(self):
        self.dispatch(86)
        self.dispatch(81)
        self.dispatch(585)
        self.assertNotIn('create_response_sent', self.a)
        self.assertEqual(self.raw_sends, [])
        self.assertEqual(self.service.state['room_reentry_rejections'][0]['reason'], 'lobby_refresh_pending')
        self.tick(11)
        self.tick(15)
        self.assertEqual(len(self.raw_sends), 2)

    def test_unsupported_body_and_live_member_leave_keep_membership(self):
        self.dispatch(86, b'extra')
        self.assertEqual(self.service.state['room_leave_requests'][-1]['rejected'], 'unsupported_leave_body')
        self.b.pop('closed')
        self.dispatch(86)
        self.assertEqual(self.service.state['room_leave_requests'][-1]['rejected'], 'unsupported_leave_membership_variant')
        self.assertIn('create_response_sent', self.a)
        self.assertEqual(self.raw_sends, [])

    def real_profile(self, *, missing_create_sequence):
        import hashlib
        import json
        from tools.export_probe_profile import FIXTURES
        root = self.service.out
        fixtures = {}
        for name in ['ready', 'roomlist_A', 'create', 'player_A', 'slot_A', 'table', 'map']:
            sequence = FIXTURES[name][1]
            self.assertIsInstance(sequence, int)
            path = root / (name + '.bin')
            raw = codec.encode(bytes(37), 86 if name == 'create' else 0, b'test', sequence)
            path.write_bytes(raw)
            fixtures[name] = {'path': str(path), 'bytes': len(raw),
                              'sha256': hashlib.sha256(raw).hexdigest(),
                              'sequence': None if name == 'create' and missing_create_sequence else sequence}
        path = root / 'profile.json'
        path.write_text(json.dumps({'schema': 'kr-probe-profile-v1', 'fixtures': fixtures}))
        return FixtureProfile(path)

    def test_old_profile_missing_dynamic_sequence_is_rejected_before_leave_mutation(self):
        self.service.profile = self.real_profile(missing_create_sequence=True)
        with self.assertRaisesRegex(ValueError, 'does not declare sequence: create'):
            self.service.profile.body('create')
        self.dispatch(86)
        request = self.service.state['room_leave_requests'][-1]
        self.assertEqual(request['rejected'], 'unsupported_reentry_profile')
        self.assertIn('create_response_sent', self.a)
        self.assertNotIn('room_visits', self.a)
        self.tick(12)
        self.assertEqual(self.raw_sends, [])

    def test_exported_sequences_work_with_real_fixture_reader_and_dynamic_recreate(self):
        self.service.profile = self.real_profile(missing_create_sequence=False)
        self.dispatch(86)
        self.tick(11)
        self.dispatch(81)
        self.dispatch(585)
        self.tick(12)
        self.tick(13)
        self.tick(14)
        self.assertEqual(len(self.raw_sends), 7)
        self.assertEqual([codec.frame_body(frame, 103 + index * 3)
                          for index, (_, frame) in enumerate(self.raw_sends)], [b'test'] * 7)
        self.assertTrue(self.a['map_response_sent'])

    def test_active_or_not_returned_round_leave_does_not_release_member(self):
        self.service.room.begin()
        self.dispatch(86)
        self.assertEqual(self.service.state['room_leave_requests'][-1]['rejected'], 'round_not_returned')
        self.service.room.finish()
        self.service.round.pop('refresh')
        self.dispatch(86)
        self.assertEqual(self.service.state['room_leave_requests'][-1]['rejected'], 'round_not_returned')
        self.assertIn('create_response_sent', self.a)
        self.assertNotIn('room_visits', self.a)


if __name__ == '__main__':
    unittest.main()
