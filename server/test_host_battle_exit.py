"""Bounded opt-in host exit policy; not an original KR authority oracle."""
import copy
from unittest.mock import patch
import unittest
from server import test_host_back_rejoin as fixture
from server import test_battle_exit as exit_fixture
from server.probe_service import ProbeService


class HostBattleExitTests(unittest.TestCase):
    setUp = fixture.HostBackRejoinTests.setUp
    member = fixture.HostBackRejoinTests.member
    tick = fixture.HostBackRejoinTests.tick
    dispatch = fixture.HostBackRejoinTests.dispatch
    send_request = exit_fixture.BattleExitTests.send_request
    body = exit_fixture.BattleExitTests.body

    def begin(self):
        self.service.battle_exit = True
        self.service.battle_exit_mode = 'host_A_202_86'
        self.service.start_round()
        self.sent.clear()

    def pair(self):
        self.send_request(202,self.body,event=self.a)
        self.send_request(86,event=self.a,seq=103)

    def test_host_pair_defers_promotion_until_survivor_returns_keeps_transport(self):
        self.begin()
        self.pair()
        left=self.a['last_room_leave']
        self.assertEqual(left['variant'],'active_host_A')
        self.assertEqual(left['survivor_connection'],8)
        self.assertNotIn('closed',self.a)
        self.assertNotIn('map_response_sent',self.a)
        self.assertIs(self.service.channels['A'],self.a)
        self.assertFalse(self.service.room.active)
        self.assertEqual(self.service.host_role,'A')
        self.assertEqual(self.sent,[])  # prepare107 must not reach Battle
        self.tick(11)
        self.assertEqual([(q['role'],q['alias']) for q in self.sent],[('A','ready'),('A','roomlist_B')])
        self.tick(22)
        self.assertEqual((self.sent[-1]['role'],self.sent[-1]['alias']),('B','rank'))
        self.assertNotIn('death_event',self.a)
        self.tick(27)
        self.assertEqual((self.sent[-1]['role'],self.sent[-1]['alias']),('B','return'))
        self.tick(30)
        self.assertEqual(self.service.host_role,'B')
        self.assertEqual([(q['role'],q['alias']) for q in self.sent[-2:]],[('B','start'),('B','promote_B')])
        self.assertEqual(self.service.round['abort']['role'],'A')
        self.assertEqual([q['role'] for q in self.service.peers()],['B'])

    def test_pair_requires_current_forward_seq_live_mapped_host_and_rechecks_race(self):
        self.begin()
        self.send_request(86,event=self.a)
        self.assertTrue(self.service.room.active)
        self.send_request(202,self.body,event=self.a,seq=106)
        self.send_request(86,event=self.a,seq=103)
        self.assertTrue(self.service.room.active)
        self.service.state['round_bombs'].append({'round':1})
        self.send_request(86,event=self.a,seq=109)
        self.assertTrue(self.service.room.active)
        self.assertEqual(self.sent,[])
        self.service.state['round_bombs'].clear()
        self.a['battle_exit202']['generation']=0
        self.send_request(86,event=self.a,seq=112)
        self.assertTrue(self.service.room.active)
        self.a.pop('battle_exit202')
        for event in [self.b,copy.deepcopy(self.a)]:
            self.send_request(202,self.body,event=event)
            self.assertEqual(self.service.state['battle_exit_reports'][-1]['rejected'],'unsupported_battle_exit_membership')

    def test_no_bombs_or_accepted_life17_19_and_no_release_on_missing_fixture(self):
        self.begin()
        for subtype in [17,19]:
            self.service.state['event_reports']=[{'round':1,'subtype':subtype}]
            self.send_request(202,self.body,event=self.a)
            self.assertNotIn('battle_exit202',self.a)
        self.service.state['event_reports'].clear()
        self.send_request(202,self.body,event=self.a)
        body=self.profile.body
        self.profile.body=lambda name: (_ for _ in ()).throw(ValueError('missing')) if name=='join_A' else body(name)
        self.send_request(86,event=self.a,seq=103)
        self.assertEqual(self.service.state['room_leave_requests'][-1]['rejected'],'unsupported_reentry_profile')
        self.assertIn('map_response_sent',self.a)
        self.assertTrue(self.service.room.active)
        self.assertEqual(self.sent,[])

    def test_host_rejoin_waits_actual_cleanup_and_reuses_guestA_ready_StartB(self):
        self.begin();self.pair();self.tick(11)
        self.dispatch(self.a,82,sequence=106)
        self.assertNotIn('create_response_sent',self.a)
        self.assertEqual(self.service.state['room_reentry_rejections'][-1]['reason'],'host_membership_changed')
        self.tick(22);self.tick(27);self.tick(30)
        self.dispatch(self.a,82,sequence=109)
        self.dispatch(self.a,585,sequence=112)
        for t in [31,32,33]:self.tick(t)
        self.assertTrue(self.a['promoted_rejoin'])
        self.assertTrue(self.a['map_response_sent'])
        self.dispatch(self.a,104,sequence=115)
        self.assertFalse(self.service.room.active)
        self.dispatch(self.a,95,sequence=118)
        self.dispatch(self.b,104,sequence=121)
        self.assertTrue(self.service.room.active)
        self.assertEqual(self.service.round['host_role'],'B')
        self.assertEqual(self.service.room.key,12346)
        self.assertNotIn('battle_exit202',self.a)

    def test_constructor_requires_known_host_profile_and_preserves_guest_modes(self):
        for mode in ['host_A_202_86','guest_B_202_86','guest_B_pending_202_86']:
            self.profile.data['battle_exit']={'mode':mode}
            svc=ProbeService(self.profile,self.out,{})
            self.addCleanup(svc.selector.close)
            self.assertEqual(svc.battle_exit_mode,mode)
        self.profile.data['battle_exit']={'mode':'host_A_202_86'}
        self.profile.data.pop('host_back_rejoin')
        with self.assertRaisesRegex(ValueError,'host Battle exit requires'):
            ProbeService(self.profile,self.out,{})


if __name__=='__main__':
    unittest.main()
