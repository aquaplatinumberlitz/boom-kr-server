"""Measured guest202/86 profile policy; neither native nor original-server oracle."""
import copy
from pathlib import Path
from unittest.mock import patch
import unittest

from server import probe_codec as codec
from server import test_guest_rejoin as fixture
from server.probe_service import ProbeService


class BattleExitTests(unittest.TestCase):
    setUp = fixture.GuestRejoinTests.setUp
    tick = fixture.GuestRejoinTests.tick
    body = codec.pack_fields([0, 1, 0, 0, 0, 0, 0], [1, 1, 4, 4, 4, 4, 4])

    def begin(self, enabled=True):
        self.service.battle_exit = enabled
        self.service.start_round()
        self.sent.clear()

    def send_request(self, opcode, body=b'', event=None, seq=100):
        event = self.b if event is None else event
        with patch('server.probe_service.time.monotonic_ns', return_value=10**10):
            self.service.receive_channel(event, codec.encode(bytes(37), opcode, body, seq), seq, body)

    def test_default_keeps_active86_rejected_and202_ignored(self):
        self.begin(False)
        self.send_request(202, self.body)
        self.send_request(86, seq=103)
        self.assertNotIn('battle_exit202', self.b)
        self.assertEqual(self.service.state['room_leave_requests'][-1]['rejected'], 'round_not_returned')
        self.assertTrue(self.service.room.active)
        self.assertEqual(self.sent, [])

    def test_normal_pair_releases_membership_keeps_channel_and_returns_survivor(self):
        self.begin()
        self.send_request(202, self.body)
        staged = self.b['battle_exit202']
        self.assertEqual(self.sent, [])
        self.assertTrue(self.service.room.active)
        self.send_request(86, seq=103)
        left = self.b['last_room_leave']
        self.assertEqual(left['variant'], 'active_guest_B')
        self.assertEqual(left['report202_sha256'], staged['frame_sha256'])
        self.assertFalse(self.service.room.active)
        self.assertNotIn('map_response_sent', self.b)
        self.assertNotIn('closed', self.b)
        self.assertIs(self.service.channels['B'], self.b)
        self.assertEqual(self.service.round['abort']['reason'], 'normal_Battle_exit')
        self.assertNotIn('death_event', self.b)
        self.assertEqual(self.sent, [])  # no prepare-only107 while A in Battle
        self.tick(11)
        self.assertEqual([(q['role'], q['alias']) for q in self.sent], [('B', 'ready'), ('B', 'roomlist_B')])
        self.tick(22)
        self.assertEqual(self.sent[-1]['role'], 'A')
        self.assertEqual(self.sent[-1]['alias'], 'rank_Awin')
        self.tick(27)
        self.assertEqual(self.sent[-1]['alias'], 'return')
        self.assertEqual(self.sent[-1]['role'], 'A')
        self.tick(30)
        self.assertTrue(self.service.round['refresh'])
        self.assertEqual(self.service.host_role, 'A')
        self.assertEqual([q['role'] for q in self.service.peers()], ['A'])
        self.assertEqual(self.service.room.keys.issued_count, 1)

    def test_active86_requires_pair_current_generation_and_forward_sequence(self):
        self.begin()
        self.send_request(86)
        self.assertEqual(self.service.state['room_leave_requests'][-1]['rejected'], 'round_not_returned')
        self.send_request(202, self.body, seq=106)
        staged = self.b['battle_exit202']
        self.send_request(86, seq=103)
        self.assertEqual(self.service.state['room_leave_requests'][-1]['rejected'], 'round_not_returned')
        staged['generation'] = 0
        self.send_request(86, seq=109)
        self.assertTrue(self.service.room.active)
        self.assertEqual(self.sent, [])

    def test_malformed_wrong_context_count_or_variant_does_not_stage(self):
        self.begin()
        for body in [self.body[:-1], self.body+b'\0', codec.pack_fields([1, 1], [1, 1])+self.body[2:],
                     codec.pack_fields([0, 2], [1, 1])+self.body[2:]]:
            self.send_request(202, body)
            self.assertEqual(self.service.state['battle_exit_reports'][-1]['rejected'], 'unsupported_battle_exit_body')
            self.assertNotIn('battle_exit202', self.b)
        self.assertEqual(self.sent, [])

    def test_other_fields_opaque_and_duplicate_report_cannot_replace_pair(self):
        self.begin()
        body = self.body[:2]+codec.pack_fields([1, 2, 3, 4, 0xFFFFFFFF], [4]*5)
        self.send_request(202, body)
        prior = copy.deepcopy(self.b['battle_exit202'])
        self.assertEqual(prior['opaque_fields'], [1, 2, 3, 4, 0xFFFFFFFF])
        self.send_request(202, self.body, seq=103)
        self.assertEqual(self.service.state['battle_exit_reports'][-1]['rejected'], 'battle_exit_already_staged')
        self.assertEqual(self.b['battle_exit202'], prior)

    def test_host_replacement_unmapped_or_closed_and_after_any_bomb_unsupported(self):
        self.begin()
        fake = copy.deepcopy(self.b)
        for event in [self.a, fake]:
            self.send_request(202, self.body, event)
            self.assertEqual(self.service.state['battle_exit_reports'][-1]['rejected'], 'unsupported_battle_exit_membership')
        self.b['map_response_sent'] = False
        self.send_request(202, self.body)
        self.assertNotIn('battle_exit202', self.b)
        self.b['map_response_sent'] = {'monotonic_ns': 1}
        self.service.state['round_bombs'].append({'round': 1})
        self.send_request(202, self.body)
        self.assertEqual(self.service.state['battle_exit_reports'][-1]['rejected'], 'unsupported_battle_exit_after_bomb')

    def test_bomb_after202_prevents86_and_start_discards_pair(self):
        self.begin()
        self.send_request(202, self.body)
        self.service.state['round_bombs'].append({'round': 1})
        self.send_request(86, seq=103)
        self.assertEqual(self.service.state['room_leave_requests'][-1]['rejected'], 'round_not_returned')
        self.service.room.finish()
        self.service.start_round()
        self.assertNotIn('battle_exit202', self.b)

    def test_missing_reentry_fixture_rejected_before_membership_or_fence_change(self):
        self.begin()
        self.send_request(202, self.body)
        before = copy.deepcopy(self.b)
        def missing(alias):
            if alias == 'ready':
                raise ValueError('missing fixture')
            return b'fixture'
        self.service.profile.body = missing
        self.send_request(86, seq=103)
        self.assertEqual(self.service.state['room_leave_requests'][-1]['rejected'], 'unsupported_reentry_profile')
        self.assertEqual(self.b, before)
        self.assertTrue(self.service.room.active)
        self.assertEqual(self.sent, [])

    def test_option_only_accepts_named_non_rescue_profile(self):
        p = self.service.profile
        for value in [True, None, {'mode': 'all'}, {'mode': 'guest_B_202_86', 'extra': 1}]:
            p.data['battle_exit'] = value
            with self.assertRaisesRegex(ValueError, 'battle_exit'):
                ProbeService(p, Path('/tmp'), {})
        p.data['battle_exit'] = {'mode': 'guest_B_202_86'}
        s = ProbeService(p, Path('/tmp'), {})
        self.addCleanup(s.selector.close)
        self.assertTrue(s.battle_exit)
        p.data['rescue_contact'] = {'mode': 'friendly_AB_synthetic2_aux1'}
        with self.assertRaisesRegex(ValueError, 'battle_exit'):
            ProbeService(p, Path('/tmp'), {})


if __name__ == '__main__':
    unittest.main()
