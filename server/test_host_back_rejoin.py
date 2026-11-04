"""Opt-in fixture membership policy; native compatibility is checked in isolated Wine."""
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

from server import probe_codec as codec
from server.probe_service import ProbeService, HOST_BACK_ALIASES


class HostBackRejoinTests(unittest.TestCase):
    def setUp(self):
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        self.out = Path(tmp.name)
        names = ['rank_Awin', 'promote_B', 'auth_C', 'roomlist_C', 'join_C',
                 'player_C', 'slot_C', 'guest_C', 'ready_C', *HOST_BACK_ALIASES]
        self.profile = SimpleNamespace(frames=dict.fromkeys(names), data={
            'items': [], 'initial_key': 12345, 'host_back_rejoin': {'mode': 'idle_A_to_B'}},
            raw=lambda _: b'', body=lambda _: b'fixture')
        self.service = ProbeService(self.profile, self.out, {})
        self.addCleanup(self.service.selector.close)
        self.a = self.member('A', 4)
        self.b = self.member('B', 8)
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
        self.service.send_raw = lambda *_: self.fail('reentry must use current recipient sequence')

    def member(self, role, ordinal):
        row = {'role': role, 'global_ordinal': ordinal, 'reply_ordinal': 4, 's2c_next_seq': 103}
        row.update({n: {'monotonic_ns': 1} for n in ['create_response_sent', 'player_response_sent',
                    'slot_response_sent', 'guest_response_sent', 'map_response_sent']})
        return row

    def dispatch(self, event, opcode, body=b'', sequence=106):
        frame = codec.encode(bytes(37), opcode, body, sequence)
        with patch('server.probe_service.time.monotonic_ns', return_value=10**10):
            self.service.receive_channel(event, frame, sequence, body)

    def tick(self, seconds):
        with patch('server.probe_service.time.monotonic_ns', return_value=seconds * 10**9):
            self.service.tick()

    def leave(self):
        self.dispatch(self.a, 86)

    def rejoin(self):
        self.leave()
        self.tick(11)
        self.dispatch(self.a, 82)
        self.dispatch(self.a, 585)
        for t in [12, 13, 14]:
            self.tick(t)

    def test_disabled_policy_keeps_legacy_rejection_and_membership(self):
        self.service.host_back_rejoin = False
        self.leave()
        self.assertEqual(self.service.state['room_leave_requests'][-1]['rejected'], 'unsupported_leave_membership_variant')
        self.assertIn('map_response_sent', self.a)
        self.assertEqual(self.sent, [])

    def test_leave_clears_only_a_and_promotes_b_without_transport_or_key_reset(self):
        self.leave()
        self.assertNotIn('create_response_sent', self.a)
        self.assertNotIn('map_response_sent', self.a)
        self.assertNotIn('closed', self.a)
        self.assertEqual(self.service.host_role, 'B')
        self.assertEqual([e['role'] for e in self.service.peers()], ['B'])
        self.assertEqual([(e['role'], e['alias']) for e in self.sent], [('B', 'start'), ('B', 'promote_B')])
        self.assertEqual(codec.read_fields(self.sent[0]['body'], [1, 4, 1]), [0, 1, 0])
        self.assertEqual((self.a['s2c_next_seq'], self.b['s2c_next_seq']), (103, 109))
        self.assertEqual(self.service.room.keys.issued_count, 0)
        self.assertEqual(self.a['last_room_leave']['survivor_connection'], 8)
        self.dispatch(self.a, 86)
        self.assertEqual(len(self.sent), 2)
        self.assertEqual(self.service.state['room_leave_requests'][-1]['rejected'], 'not_room_member')

    def test_rejoin_is_currentseq_guest_a_chief_b_then_ready_a_start_b(self):
        self.rejoin()
        self.assertEqual([(e['role'], e['alias']) for e in self.sent], [
            ('B', 'start'), ('B', 'promote_B'), ('A', 'ready'), ('A', 'roomlist_B'),
            ('A', 'join_A'), ('A', 'player_A_guest'), ('B', 'player_A_guest'),
            ('A', 'slot_A_guest'), ('A', 'guest_B_chief'), ('A', 'table'), ('A', 'map')])
        self.assertTrue(self.a['map_response_sent'])
        self.assertTrue(self.a['promoted_rejoin'])
        self.assertEqual((self.a['s2c_next_seq'], self.b['s2c_next_seq']), (127, 112))
        self.dispatch(self.a, 104)
        self.assertFalse(self.service.room.active)
        self.dispatch(self.b, 104)
        self.assertFalse(self.service.room.active)
        self.dispatch(self.a, 95)
        self.assertEqual([e['alias'] for e in self.sent[-2:]], ['ready_A', 'ready_A'])
        self.dispatch(self.b, 104)
        self.assertTrue(self.service.room.active)
        self.assertEqual(self.service.round['host_role'], 'B')
        self.assertEqual(self.service.room.keys.issued_count, 1)
        self.assertEqual([(e['role'], e['slot'], e['identity']) for e in self.service.round['roster']], [('A', 0, 1), ('B', 1, 2)])

    def test_refresh_pending_create_and_lobby_ready_do_not_restore_member(self):
        self.leave()
        self.dispatch(self.a, 82)
        self.assertEqual(self.service.state['room_reentry_rejections'][-1]['reason'], 'lobby_refresh_pending')
        self.tick(11)
        self.dispatch(self.a, 81)
        self.assertEqual(self.service.state['room_reentry_rejections'][-1]['reason'], 'promoted_room_requires_join')
        self.dispatch(self.a, 95)
        self.assertEqual(self.service.state['ready_rejections'][-1]['reason'], 'not_rejoined_member')
        self.assertNotIn('create_response_sent', self.a)
        self.assertNotIn('guest_ready_received', self.a)

    def test_changed_survivor_identity_blocks_old_membership_rejoin(self):
        self.leave()
        self.tick(11)
        self.b['global_ordinal'] = 12
        self.dispatch(self.a, 82)
        self.assertEqual(self.service.state['room_reentry_rejections'][-1]['reason'], 'host_membership_changed')
        self.assertNotIn('create_response_sent', self.a)
        self.assertFalse(any(e['alias'] == 'join_A' for e in self.sent))

    def test_active_or_nonempty_leave_does_not_change_host(self):
        self.dispatch(self.a, 86, b'extra')
        self.assertEqual(self.service.state['room_leave_requests'][-1]['rejected'], 'unsupported_leave_body')
        self.service.room.begin()
        self.leave()
        self.assertEqual(self.service.state['room_leave_requests'][-1]['rejected'], 'round_not_returned')
        self.assertEqual(self.service.host_role, 'A')
        self.assertIn('map_response_sent', self.a)
        self.assertEqual(self.sent, [])

    def test_return_refresh_uses_guest_a_state_instead_of_replacement_c(self):
        self.rejoin()
        self.service.round = {'index': 1, 'rank': {}, 'return': {'monotonic_ns': 1}}
        self.tick(20)
        self.assertEqual([e['alias'] for e in self.sent[-4:]], ['promote_B', 'promote_B', 'slot_A_guest', 'slot_A_guest'])

    def test_profile_optin_fails_before_owner_creation_on_missing_or_unsequenced_alias(self):
        self.profile.frames.pop('join_A')
        with self.assertRaisesRegex(ValueError, 'missing host Back fixtures'):
            ProbeService(self.profile, self.out, {})
        self.profile.frames['join_A'] = b''
        def body(alias):
            if alias == 'slot_A_guest':
                raise ValueError('fixture does not declare sequence: slot_A_guest')
            return b'fixture'
        self.profile.body = body
        with self.assertRaisesRegex(ValueError, 'does not declare sequence'):
            ProbeService(self.profile, self.out, {})


if __name__ == '__main__':
    unittest.main()
