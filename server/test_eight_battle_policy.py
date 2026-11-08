"""Fixed eight-player result/reset controls; native acceptance is separate."""
from pathlib import Path
import copy
import struct
import tempfile
import unittest
from server import probe_codec as c
from server.eight_battle_policy import EightBattleReports, rank_body
from server.eight_bomb_policy import BASELINE
from server.eight_player_service import EightPlayerService
from server.probe_room import ProbeRoom
from server.room_title import _encode_bytes,_decode_bytes
from server.test_four_player_service import fixtures

ROLES='ABCDEFGH';SLOTS=dict(zip(ROLES,range(8)));TEAMS={r:i%2 for i,r in enumerate(ROLES)}

class EightReports(unittest.TestCase):
    def body(self,role,sub,tick):
        prefix=c.pack_fields([sub,SLOTS[role],160,159,tick,5 if sub==17 else 0],[1,1,2,2,4,4])
        if sub==21:return prefix
        inner=struct.pack('<II',2,0) if sub==17 else bytes.fromhex('ffffffff00')
        stream=b'\x18'+struct.pack('<I',len(inner)+1)+b'\0'+inner
        return prefix+c.pack_fields([len(stream)],[4])+_encode_bytes(stream)

    def test_eight_requires_all_losing_members_and_fresh_witness_after_drain(self):
        for winner in range(2):
            room=ProbeRoom([]);room.begin();p=EightBattleReports(room,SLOTS,TEAMS);p.begin()
            losers=[r for r in ROLES if TEAMS[r]!=winner];witness=next(r for r in ROLES if TEAMS[r]==winner)
            self.assertEqual(p.prepare(losers[0],self.body(losers[0],19,10),1)['rejected'],'death_no_current_trap')
            for role in losers:
                for sub,tick in [(17,20),(19,21)]:
                    q=p.prepare(role,self.body(role,sub,tick),20+ROLES.index(role))
                    self.assertEqual(len(q['forward_payload']),{17:10,19:21}[sub]);p.commit(q)
                self.assertIsNone(p.outcome(10**11,False))
            q=p.prepare(witness,self.body(witness,21,30),100);p.commit(q)
            self.assertIsNone(p.outcome(101,False));self.assertIsNone(p.outcome(10**11,True))
            q=p.outcome(10**11,False);self.assertEqual(q['winner_team'],winner)
            self.assertEqual(set(q['death_roles']),set(losers))
            self.assertIn('eight_4v4',q['policy'])
            self.assertEqual(p.prepare(witness,self.body(witness,21,30),101)['rejected'],'duplicate_report_body')
            room.finish();room.begin();p.begin()
            self.assertEqual(len(p.life),8);self.assertTrue(all(z['death'] is None for z in p.life.values()))
            self.assertEqual(p.prepare(witness,self.body(witness,21,29),102)['rejected'],'report_tick_not_forward')

    def test_rank_eight_rows_keep_sixteen_slot_envelope_and_fixed_teams(self):
        for winner in range(2):
            body=rank_body(winner,TEAMS);inner=_decode_bytes(body[4:])[6:]
            self.assertEqual((len(body),len(inner),inner[9]),(1082,1072,255))
            self.assertEqual([inner[16+i*131+5] for i in range(8)],[1 if i%2==winner else 2 for i in range(8)])
            self.assertEqual(inner[-8:],bytes(8))
        for winner,teams in [(True,TEAMS),(2,TEAMS),(0,dict.fromkeys(ROLES,0))]:
            with self.assertRaises(ValueError):rank_body(winner,teams)

    def test_eight_outcome_opt_in_is_explicit_and_captured_in_metadata(self):
        with tempfile.TemporaryDirectory() as folder:
            ips={r:'127.0.0.'+str(i+2) for i,r in enumerate(ROLES)}
            for outcome in [1,'true']:
                with self.assertRaises(ValueError):
                    EightPlayerService(fixtures(),Path(folder),ips,bomb_profile=BASELINE,battle_outcome=outcome)
            s=EightPlayerService(fixtures(),Path(folder),ips,bomb_profile=copy.deepcopy(BASELINE),battle_outcome=True)
            try:
                self.assertIsInstance(s.battle_reports,EightBattleReports)
                self.assertTrue(s.observability.metadata['config']['battle_outcome'])
                self.assertEqual(s.state['eight_outcome_policy']['source'],'design')
            finally:s.selector.close()
