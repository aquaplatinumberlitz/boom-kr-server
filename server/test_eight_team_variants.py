"""Fixed team families and selector policy controls, not native UI proof."""
from pathlib import Path
import struct
import tempfile
from types import SimpleNamespace
import unittest
from server import eight_player_profile as eight, probe_codec as c
from server.eight_battle_policy import EightBattleReports,rank_body
from server.eight_player_service import EightPlayerService
from server.probe_room import ProbeRoom
from server.room_title import _decode_bytes,_encode_bytes
from server.test_four_player_service import fixtures
from server import test_eight_battle_policy as report_fixtures

class Teams(unittest.TestCase):
    def test_profiles_are_independent_and_start_rows_and_room_flags_use_selected_teams(self):
        one=eight.TeamProfile('1v7');balanced=eight.TeamProfile('4v4');p=fixtures()
        # The generic stub leaves both row teams zero; the measured KR two-row
        # template is A0/B1 (CP020/208). Preserve those rows as the encoder does.
        template=bytearray(p.body('start'));size=c.read_fields(template[1:5],[4])[0]
        raw=bytearray(_decode_bytes(template[5:5+size]));raw[6+74+5]=1
        template[5:5+size]=_encode_bytes(raw)
        self.assertEqual([m['team'] for m in one.MEMBERS.values()],[0,1,1,1,1,1,1,1])
        for profile in [one,balanced]:
            body=profile.start_body(bytes(template));size=c.read_fields(body[1:5],[4])[0]
            inner=_decode_bytes(body[5:5+size])[6:]
            self.assertEqual([inner[48+26*i+5] for i in range(8)],[m['team'] for m in profile.MEMBERS.values()])
            for role,m in profile.MEMBERS.items():
                slot=profile.slot_body(p.body('player_B'),m,chief=role=='A',ready=role!='A')
                state=struct.unpack('<I',_decode_bytes(slot[1:5]))[0]
                self.assertEqual((state>>5)&7,m['team'])
        one.MEMBERS['C']['team']=0
        self.assertEqual(eight.MEMBERS['C']['team'],0)
        self.assertEqual(eight.TeamProfile('1v7').MEMBERS['C']['team'],1)
        for bad in ['other',None,[]]:
            with self.assertRaises(ValueError):eight.TeamProfile(bad)

    def test_one_seven_outcomes_require_every_losing_member_and_fresh_witness(self):
        bodies=report_fixtures.EightReports().body;teams=eight.TEAM_LAYOUTS['1v7']
        for winner in range(2):
            room=ProbeRoom([]);room.begin();p=EightBattleReports(room,dict(zip(eight.ROLES,range(8))),teams);p.begin()
            losers=[r for r in eight.ROLES if teams[r]!=winner];witness=next(r for r in eight.ROLES if teams[r]==winner)
            for role in losers:
                for sub,tick in [(17,20),(19,21)]:
                    q=p.prepare(role,bodies(role,sub,tick),30);self.assertNotIn('rejected',q);p.commit(q)
                self.assertIsNone(p.outcome(10**11,False))
            q=p.prepare(witness,bodies(witness,21,30),100);p.commit(q)
            self.assertIsNone(p.outcome(101,False));self.assertIsNone(p.outcome(10**11,True))
            result=p.outcome(10**11,False);self.assertEqual(result['winner_team'],winner)
            self.assertEqual(set(result['death_roles']),set(losers));self.assertIn('1v7',result['policy'])
            body=rank_body(winner,teams);inner=_decode_bytes(body[4:])[6:]
            self.assertEqual([inner[16+131*i+5] for i in range(8)],[1 if teams[r]==winner else 2 for r in eight.ROLES])

    def test_manner_requires_persistence_profile_and_unbalanced_start_does_not_mutate_round(self):
        with tempfile.TemporaryDirectory() as folder:
            ips={r:'127.0.0.'+str(i+2) for i,r in enumerate(eight.ROLES)}
            with self.assertRaisesRegex(ValueError,'selector echo'):
                EightPlayerService(fixtures(),Path(folder),ips,allow_manner=True)
            p=fixtures();p.data['room_team_mode']={'mode':'create81_to_86_87'}
            base_body,base_raw=p.body,p.raw
            bodies={'create':c.pack_fields([0]*11,[1]*11),'return':c.pack_fields([0,0,0],[1]*3)}
            p.body=lambda n:bodies[n] if n in bodies else base_body(n)
            p.raw=lambda n:c.encode(bytes(37),{'create':86,'join':87,'return':180}[n],p.body(n),43) if n in ['create','join','return'] else base_raw(n)
            s=EightPlayerService(p,Path(folder),ips,team_layout='1v7',allow_manner=True)
            try:
                s.room_team_selector=1;e=dict(role='A',global_ordinal=1,reply_ordinal=4)
                before=(s.room.generation,s.room.key,s.room._wire_id)
                s.receive_channel(e,c.encode(bytes(37),104,b'',40),40,b'')
                self.assertEqual(s.state['stage_rejections'][-1]['reason'],'manner_fixed_teams_unbalanced')
                self.assertFalse(s.room.active);self.assertEqual((s.room.generation,s.room.key,s.room._wire_id),before)
                self.assertEqual(s.observability.metadata['config']['accepted_room_selectors'],[0,1])
                self.assertEqual(s.observability.metadata['config']['operator_team_layout'],'1v7')
            finally:s.selector.close()
