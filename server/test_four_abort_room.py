from pathlib import Path
import copy,time,unittest
from unittest.mock import patch
from server import probe_codec as c
from server import test_four_bomb_policy as f
from server import four_player_service as m
class AbortTests(unittest.TestCase):
 def service(self,enabled=True):
  h=f.FourBombTests();self.addCleanup(h.doCleanups)
  def make(*a,**kw):
   profile=a[0];raw=profile.raw;body=profile.body
   profile.body=lambda name:c.pack_fields([0,0,0],[1,1,1]) if name=='return' else body(name)
   profile.raw=lambda name:c.encode(bytes(37),180,c.pack_fields([0,0,0],[1,1,1]),43) if name=='return' else raw(name)
   return m.FourPlayerService(*a,**kw,battle_outcome=True,abort_room=enabled)
  with patch.object(f,'FourPlayerService',make):s,sent=h.service()
  return h,s,sent
 def setup(self,s,role):
  e=s.channels[role];s.receive_channel(e,c.encode(bytes(37),585,b'',40),40,b'')
 def test_guest_pending_eof_unranked_return_gated_remove_and_fenced_timer(self):
  h,s,sent=self.service();self.setup(s,'A');body=h.body(s.room,3)
  s.receive_channel(s.channels['D'],c.encode(bytes(37),118,body,40),40,body)
  s.close_connection(s.channels['D']['global_ordinal'],'eof');s.tick()
  j=s.abort_room_job;self.assertEqual(set(j['returns']),set('ABC'));self.assertEqual(j['setup_requests'],{});self.assertNotIn('completed_ns',j)
  for role in 'AB':self.setup(s,role)
  s.tick();self.assertEqual(j['removed'],{})
  self.setup(s,'C');s.tick();self.assertIn('completed_ns',j);self.assertEqual(j['chief_role'],'A')
  for role in 'ABC':
   q=j['removed'][role]['D'];body=c.frame_body((s.out/q['fixture']).read_bytes(),q['seq']);self.assertEqual(c.read_fields(body,[1,4,1]),[3,4,0])
   self.assertEqual(set(j['refreshed'][role]),set('ABC'));self.assertFalse(s.channels[role].get('guest_ready_received'))
  before=[len(v) for v in sent.values()];s.tick();self.assertEqual(before,[len(v) for v in sent.values()])
  s.bomb_jobs=[(0,i,z) for _,i,z in s.bomb_jobs];s.tick();self.assertEqual(s.state['round_bombs'][0]['timer_rejected'],'round_closed');self.assertNotIn('broadcast158',s.state['round_bombs'][0]);self.assertNotIn('rank',s.round)
  self.assertFalse(any(q['opcode']==175 for e in s.peers() for q in e['dynamic_s2c']))
  s.receive_channel(s.channels['A'],c.encode(bytes(37),104,b'',43),43,b'');self.assertFalse(s.room.active)
 def test_default_keeps_prior_fence_only(self):
  _,s,_=self.service(False);s.close_connection(s.channels['D']['global_ordinal'],'eof');s.tick();self.assertIsNone(s.abort_room_job)
 def test_no_setup_wait_blocking_and_remaining_close_before_setup(self):
  _,s,_=self.service();s.close_connection(s.channels['D']['global_ordinal'],'eof');s.tick();j=s.abort_room_job
  s.tick_abort_room(j['created_ns']+16*10**9);self.assertEqual(set(j['waiting_setup_roles']),set('ABC'))
  s.close_connection(s.channels['C']['global_ordinal'],'eof')
  for role in 'AB':self.setup(s,role)
  s.tick();self.assertEqual(j['departed'],['D','C']);self.assertIn('completed_ns',j)
  self.assertEqual(set(j['removed']['A']),set('CD'));self.assertEqual(set(j['refreshed']['A']),set('AB'))
 def test_partial_return_send_failure_adds_recipient_and_retries_survivors(self):
  _,s,sent=self.service();s.close_connection(s.channels['D']['global_ordinal'],'eof')
  def fail(data):raise TimeoutError('recipientC')
  s.connections[s.channels['C']['global_ordinal']]['socket'].sendall=fail
  s.tick();self.assertEqual(s.abort_room_job['departed'],['D','C']);self.assertEqual(set(s.abort_room_job['returns']),set('AB'))
  for role in 'AB':self.setup(s,role)
  s.tick();self.assertIn('completed_ns',s.abort_room_job)
 def test_completed_abort_and_primary_loss_history_survive_cleanup_EOF(self):
  _,s,_=self.service();s.close_connection(s.channels['D']['global_ordinal'],'eof');s.tick()
  for role in 'ABC':self.setup(s,role)
  s.tick();job=copy.deepcopy(s.abort_room_job);primary=copy.deepcopy(s.state['membership_broken'])
  s.close_connection(s.channels['A']['global_ordinal'],'eof');s.tick()
  self.assertEqual(s.abort_room_job,job);self.assertEqual(s.state['membership_broken'],primary)
 def test_prebattle_close_does_not_infer_abort(self):
  _,s,_=self.service();s.room.finish();s.close_connection(s.channels['D']['global_ordinal'],'eof');s.tick();self.assertIsNone(s.abort_room_job)
if __name__=='__main__':unittest.main()
