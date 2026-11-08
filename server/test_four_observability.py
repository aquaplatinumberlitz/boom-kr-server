from pathlib import Path
import hashlib,importlib.util,json,sys,unittest
from types import SimpleNamespace
from server import test_four_bomb_policy as f
from server import four_observability as m
class HealthTests(unittest.TestCase):
 def service(self):
  h=f.FourBombTests();self.addCleanup(h.doCleanups);s,_=h.service();return s
 def test_unique_run_ids_and_stable_source_build_identity(self):
  s=self.service();a=m.FourObservability(s);b=m.FourObservability(s)
  self.assertNotEqual(a.run_id,b.run_id);self.assertEqual(a.metadata['source_build_sha256'],b.metadata['source_build_sha256']);self.assertEqual(a.metadata['profile_sha256'],b.metadata['profile_sha256'])
 def test_phases_and_snapshot_hashes_and_runmetadata_unchanged(self):
  s=self.service();o=m.FourObservability(s);s.room.finish();s.state.pop('ready',None)
  for phase,changes in [('starting',{}),('ready',{'ready':True}),('active',{}),('degraded',{'membership_broken':{'role':'D'}}),('stopping',{}),('stopped',{'stopped':True})]:
   if phase=='active':s.room.begin()
   if phase=='stopping':s.closed=True
   s.state.update(changes);q=o.publish(s);self.assertEqual(q['phase'],phase)
   raw=(s.out/'service-state.json').read_bytes();saved=json.loads(raw);self.assertEqual(q['state_sha256'],hashlib.sha256(raw).hexdigest());self.assertEqual(q['snapshot_sequence'],saved['observability']['snapshot_sequence']);self.assertEqual(q['run_metadata_sha256'],hashlib.sha256((s.out/'run-metadata.json').read_bytes()).hexdigest())
  self.assertFalse(list(s.out.glob('*.tmp')))
 def test_transport_session_room_round_correlations(self):
  s=self.service();o=m.FourObservability(s);a=o.publish(s);self.assertEqual(len({e['correlation']['transport_id'] for e in s.events}),4)
  self.assertEqual({e['correlation']['session_id'] for e in s.events},{o.run_id+'/'+r for r in 'ABCD'})
  old=a['round_id'];s.room.finish();s.start_round();b=o.publish(s);self.assertNotEqual(b['round_id'],old);self.assertEqual(a['room_id'],b['room_id']);self.assertEqual(a['run_id'],b['run_id'])
 def test_reused_output_refuses_without_overwriting_existing_files(self):
  s=self.service();a=m.FourObservability(s);a.publish(s);old={p.name:p.read_bytes() for p in s.out.glob('*.json')};b=m.FourObservability(s)
  with self.assertRaises(FileExistsError):b.publish(s)
  self.assertEqual(old,{p.name:p.read_bytes() for p in s.out.glob('*.json')})

class HealthLifecycleTests(unittest.TestCase):
 service=HealthTests.service
 def test_round_metadata_uses_explicit_record_round_not_current_round(self):
  s=self.service();o=m.FourObservability(s)
  s.state['event_reports']=[dict(role='B',round=1),dict(role='A',round=2),dict(role='C')]
  o.publish(s);rows=s.state['event_reports']
  self.assertTrue(rows[0]['correlation']['round_id'].endswith('/round1'))
  self.assertTrue(rows[1]['correlation']['round_id'].endswith('/round2'))
  self.assertIsNone(rows[2]['correlation']['round_id'])
 def test_phase_history_is_bounded_and_total_does_not_reset(self):
  s=self.service();o=m.FourObservability(s)
  for i in range(50):
   s.closed=bool(i%2);o.stamp(s)
  self.assertEqual(o.transitions_total,50);self.assertEqual(len(o.transitions),32)
  self.assertEqual(o.sequence,50)
 def test_health_write_failure_does_not_skip_owned_socket_cleanup(self):
  from unittest.mock import patch
  from server import four_player_service as service_module
  s=self.service();opened=[];closed=[];original=service_module.socket.socket
  for i,row in s.connections.items():row['socket'].close=lambda i=i:closed.append(i)
  def track(*a,**kw):
   result=original(*a,**kw);opened.append(result);return result
  with patch.object(service_module.socket,'socket',side_effect=track),patch.object(s,'save',side_effect=OSError('controlled health IO failure')):
   with self.assertRaisesRegex(OSError,'controlled health'):s.run([0])
  self.assertTrue(opened);self.assertTrue(all(p.fileno()==-1 for p in opened))
  self.assertEqual(set(closed),set(s.connections));self.assertIsNone(s.selector.get_map());self.assertFalse(s.room.active)

if __name__=='__main__':unittest.main()
