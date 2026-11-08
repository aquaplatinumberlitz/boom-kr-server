from concurrent.futures import ThreadPoolExecutor
import copy,importlib.util,json
from pathlib import Path
import tempfile,unittest
from server.four_player_profile import MEMBERS
from server import four_identity_store as m
class IdentityTests(unittest.TestCase):
 def directory(self):
  d=tempfile.TemporaryDirectory();self.addCleanup(d.cleanup);return Path(d.name)
 def test_first_publish_and_restart_preserve_same_hash_and_ids(self):
  p=self.directory()/'identities.json';a=m.pin_identities(p,MEMBERS);old=p.read_bytes();b=m.pin_identities(p,MEMBERS)
  self.assertTrue(a['created']);self.assertFalse(b['created']);self.assertEqual(a['sha256'],b['sha256']);self.assertEqual(old,p.read_bytes())
  j=json.loads(old);self.assertEqual([j['members'][r]['identity'] for r in 'ABCD'],[1,2,3,4])
 def test_existing_corrupt_changed_truncated_or_oversized_store_refused_unchanged(self):
  p=self.directory()/'identities.json';changed=copy.deepcopy(MEMBERS);changed['D']['identity']=99
  for raw in [b'',b'{}',m.identity_bytes(changed),b'x'*16385,m.identity_bytes(MEMBERS)[:-5]]:
   p.write_bytes(raw)
   with self.subTest(length=len(raw)),self.assertRaises(ValueError):m.pin_identities(p,MEMBERS)
   self.assertEqual(p.read_bytes(),raw)
 def test_concurrent_equal_creators_one_winner_complete_bytes_no_temporary_tail(self):
  p=self.directory()/'identities.json'
  with ThreadPoolExecutor(max_workers=8) as ex:out=list(ex.map(lambda _:m.pin_identities(p,MEMBERS),range(16)))
  self.assertEqual(sum(q['created'] for q in out),1);self.assertEqual(len({q['sha256'] for q in out}),1)
  self.assertEqual(p.read_bytes(),m.identity_bytes(MEMBERS));self.assertFalse(list(p.parent.glob('*.tmp')))
 def test_missing_parent_is_not_silently_created(self):
  p=self.directory()/'missing'/'identities.json'
  with self.assertRaises(FileNotFoundError):m.pin_identities(p,MEMBERS)
  self.assertFalse(p.parent.exists())
if __name__=='__main__':unittest.main()
