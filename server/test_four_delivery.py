"""Modeled write failures at four-service entrypoints; TCP/default-timeout evidence is separate."""
from pathlib import Path
import importlib.util,sys,unittest
from unittest.mock import patch
from server import probe_codec as c
from server import test_four_bomb_policy as fixtures
from server import test_four_battle_policy as reports
from server import four_player_service as m
class RecipientTests(unittest.TestCase):
 def service(self):
  helper=fixtures.FourBombTests();self.addCleanup(helper.doCleanups)
  def make(*a,**k):return m.FourPlayerService(*a,**k,battle_outcome=True)
  with patch.object(fixtures,'FourPlayerService',make):service,sent=helper.service()
  return helper,service,sent
 def fail(self,service,role):
  def fail(_data):raise TimeoutError('controlled recipient '+role)
  service.connections[service.channels[role]['global_ordinal']]['socket'].sendall=fail
 def input(self,service,role,sub=17):
  event=service.channels[role];body=reports.Reports().body(role,sub,10)
  connection=service.connections[event['global_ordinal']];connection.update(buffer=bytearray(),sequence=40);event['chunks']=[]
  connection['socket'].recv=lambda _size:c.encode(bytes(37),127,body,40)
  service.receive(event['global_ordinal'])
 def test_partial_first_A_delivered_C_fails_B_stays_and_report_not_committed(self):
  _,service,_=self.service();pending=service.room.bomb();self.fail(service,'C');self.input(service,'B')
  self.assertEqual(service.state['membership_broken']['role'],'C');self.assertFalse(service.channels['B'].get('closed'))
  report=service.state['event_reports'][-1];self.assertEqual(report['rejected'],'peer_delivery_incomplete')
  self.assertEqual([q['role'] for q in report['forward170']],['A'])
  self.assertEqual(service.battle_reports.cursors,{});self.assertIsNone(service.battle_reports.life['B']['trap'])
  self.assertEqual(service.room.fence.rejection(pending),'round_closed')
 def test_tick_send_failure_is_contained_and_pending_second_timer_fenced(self):
  helper,service,_=self.service()
  for role in 'AB':
   body=helper.body(service.room,'AB'.index(role));service.receive_bomb(service.channels[role],c.encode(bytes(37),118,body,40),40,body)
  self.fail(service,'C');service.bomb_jobs=[(0,i,z) for _,i,z in service.bomb_jobs];service.tick()
  self.assertEqual(service.state['delivery_failures'][-1]['owner_entrypoint'],'tick')
  self.assertEqual(service.state['membership_broken']['role'],'C');self.assertNotIn('rank',service.round)
  service.tick();self.assertEqual(service.state['round_bombs'][-1]['timer_rejected'],'round_closed')
 def test_successful_current_report_still_commits_after_three_deliveries(self):
  _,service,_=self.service();self.input(service,'B')
  self.assertTrue(service.room.active);self.assertNotIn('delivery_failures',service.state)
  q=service.state['event_reports'][-1];self.assertEqual({x['role'] for x in q['forward170']},{'A','C','D'})
  self.assertIs(service.battle_reports.life['B']['trap'],q);self.assertIn('B',service.battle_reports.cursors)
if __name__=='__main__':unittest.main()
