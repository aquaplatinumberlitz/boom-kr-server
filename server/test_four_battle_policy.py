"""Report-derived 2v2 controls; synthetic envelopes reuse CP201 native shapes."""
import struct
import unittest
from server.probe_room import ProbeRoom
from server import probe_codec as c
from server import four_battle_policy as m
from server.room_title import _encode_bytes, _decode_bytes
TEAMS={'A':0,'B':1,'C':0,'D':1};SLOTS=dict(zip('ABCD',range(4)))
class Reports(unittest.TestCase):
 def setUp(self):
  self.room=ProbeRoom([]);self.room.begin();self.policy=m.FourBattleReports(self.room,SLOTS,TEAMS)
  self.policy.begin()
 def body(self,role,sub,tick):
  if sub==21:return c.pack_fields([sub,SLOTS[role],160,159,tick,0],[1,1,2,2,4,4])
  # Captured CP201 inner17=(2,0), inner19=(FFFFFFFF,0entries), KR feature-off blob.
  inner=struct.pack('<II',2,0) if sub==17 else bytes.fromhex('ffffffff00')
  blob=b'\x18'+struct.pack('<I',len(inner)+1)+b'\0'+inner
  return c.pack_fields([sub,SLOTS[role],160,159,tick,5 if sub==17 else 0],[1,1,2,2,4,4])+c.pack_fields([len(blob)],[4])+_encode_bytes(blob)

 def admit(self,r,s,tick,ns):
  report=self.policy.prepare(r,self.body(r,s,tick),ns);self.assertNotIn('rejected',report);self.policy.commit(report);return report
 def test_native_report_shapes_and_three_peer_payloads(self):
  for role in 'ABC':
   self.assertEqual(len(self.admit(role,17,100,10)['forward_payload']),10)
   self.assertEqual(len(self.admit(role,19,110,20)['forward_payload']),21)
  self.assertEqual(len(self.admit('D',17,100,10)['forward_payload']),10)
 def test_no_first_death_win_no_stale_witness_pending_or_early_result(self):
  self.admit('B',21,10,1)
  self.admit('A',17,10,10);self.admit('A',19,11,20)
  self.assertIsNone(self.policy.outcome(10**11,False))
  self.admit('C',17,10,30);self.admit('C',19,11,40)
  self.assertIsNone(self.policy.outcome(10**11,False))
  self.admit('B',21,11,50)
  self.assertIsNone(self.policy.outcome(51,False));self.assertIsNone(self.policy.outcome(10**11,True))
  z=self.policy.outcome(10**11,False);self.assertEqual(z['winner_team'],1);self.assertEqual(z['death_roles'],['A','C'])
 def test_reject_untrapped_death_wrongslot_duplicate_rollback_and_shape(self):
  self.assertEqual(self.policy.prepare('A',self.body('A',19,10),1)['rejected'],'death_no_current_trap')
  self.admit('A',17,20,1)
  self.assertEqual(self.policy.prepare('A',self.body('A',17,20),2)['rejected'],'duplicate_report_body')
  self.assertEqual(self.policy.prepare('A',self.body('A',17,19),2)['rejected'],'report_tick_not_forward')
  self.assertEqual(self.policy.prepare('B',self.body('A',17,21),2)['rejected'],'report_slot_mismatch')
  self.assertEqual(self.policy.prepare('A',self.body('A',21,21)+b'x',2)['rejected'],'unsupported_report21_shape')
  self.admit('A',19,21,2)
  self.assertEqual(self.policy.prepare('A',self.body('A',21,22),3)['rejected'],'report_after_accepted_death')
 def test_latest_blast_invalidates_survivor(self):
  for r in 'AC':self.admit(r,17,10,10);self.admit(r,19,11,20)
  self.admit('B',21,10,30);self.policy.last_blast_ns=40
  self.assertIsNone(self.policy.outcome(10**11,False))
  self.admit('B',21,11,50);self.assertEqual(self.policy.outcome(10**11,False)['winner_team'],1)
 def test_no_cursor_commit_before_delivery_and_persistent_cursor_reset_life(self):
  report=self.policy.prepare('A',self.body('A',17,10),10)
  self.assertEqual(self.policy.cursors,{});self.assertIsNone(self.policy.life['A']['trap'])
  self.policy.commit(report);self.room.finish();self.assertIsNone(self.policy.outcome(10**11,False))
  self.room.begin();self.policy.begin();self.assertIsNone(self.policy.life['A']['trap'])
  self.assertEqual(self.policy.prepare('A',self.body('A',17,9),20)['rejected'],'report_tick_not_forward')
  self.admit('A',17,11,20)
 def test_all_dead_is_unsupported_not_arbitrary_winner(self):
  for r in 'ABC':self.admit(r,17,10,10);self.admit(r,19,11,20)
  # Use the same measured death envelope with the bound D slot, no native D19 claim.
  self.admit('D',17,10,10);b=bytearray(self.body('A',19,11));b[1:2]=c.pack_fields([3],[1]);q=self.policy.prepare('D',bytes(b),20);self.policy.commit(q)
  self.assertIsNone(self.policy.outcome(10**11,False))
 def test_both_rank_vectors_match_and_bad_roster_fails_closed(self):
  for team in (0,1):
   body=m.rank_body(team,TEAMS);raw=_decode_bytes(body[4:]);inner=raw[6:]
   self.assertEqual(len(inner),552);self.assertEqual(inner[9],255)
   self.assertEqual([inner[16+131*i+5] for i in range(4)],[1 if i%2==team else 2 for i in range(4)])
  with self.assertRaises(ValueError):m.rank_body(2,TEAMS)
  with self.assertRaises(ValueError):m.FourBattleReports(self.room,SLOTS,dict.fromkeys('ABCD',0))
if __name__=='__main__':unittest.main()
