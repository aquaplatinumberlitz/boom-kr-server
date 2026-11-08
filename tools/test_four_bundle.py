"""Portable four-service launcher controls. Synthetic fixtures are not a native oracle."""
import copy
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

from probe_bundle import export_bundle

SPEC=dict(mode='four_current_baseline',capacities={'A':1,'B':2,'C':1,'D':1},
          powers=dict.fromkeys('ABCD',1),fuse_seconds=6)


class FourBundleTests(unittest.TestCase):
    def setUp(self):
        tmp=tempfile.TemporaryDirectory();self.addCleanup(tmp.cleanup)
        self.root=Path(tmp.name);self.source=self.root/'source';self.source.mkdir()
        raw=b'packaging fixture, not native wire';p=self.source/'fixture.bin';p.write_bytes(raw)
        row=dict(path=str(p),bytes=len(raw),sha256=hashlib.sha256(raw).hexdigest(),sequence=None)
        self.profile=dict(schema='kr-probe-profile-v1',items=[],initial_key=12345,
            fixtures={n:dict(row) for n in ['bootstrap','rank_Awin','promote_B','auth_C','roomlist_C',
                                            'join_C','player_C','slot_C','guest_C','ready_C']})

    def call(self,out,*args):
        return subprocess.run([sys.executable,'-I',str(out/'run.py'),*args],cwd=self.root,
                              text=True,capture_output=True)

    def test_four_launcher_relocates_under_PythonI_and_requires_its_policy_reference(self):
        out=self.root/'bundle'
        manifest=export_bundle(self.profile,self.source,out,four_bomb_profile=SPEC)
        self.assertEqual(manifest['entrypoint'],'server.four_player_service')
        moved=self.root/'moved';shutil.move(out,moved);shutil.rmtree(self.source)
        self.assertEqual(self.call(moved,'--verify-only').returncode,0)
        result=self.call(moved,'--help')
        self.assertEqual(result.returncode,0);self.assertIn('--source-ip',result.stdout)
        p=moved/'bundle.json';j=json.loads(p.read_text())
        j['files']=[q for q in j['files'] if q['path']!='four-bomb-profile.json'];p.write_text(json.dumps(j))
        self.assertIn('incomplete bundle manifest',self.call(moved,'--verify-only').stderr)

    def test_unlisted_server_source_is_not_imported_or_reported_verified(self):
        out=self.root/'bundle';export_bundle(self.profile,self.source,out,four_bomb_profile=SPEC)
        p=out/'bundle.json';j=json.loads(p.read_text())
        j['files']=[q for q in j['files'] if q['path']!='server/four_input_limits.py'];p.write_text(json.dumps(j))
        result=self.call(out,'--verify-only')
        self.assertNotEqual(result.returncode,0);self.assertIn('unlisted bundle server source',result.stderr)

    def test_policy_tamper_and_override_fail_before_runtime_output(self):
        out=self.root/'bundle';export_bundle(self.profile,self.source,out,four_bomb_profile=SPEC)
        runtime=self.root/'not-created'
        for option in [['--bomb-profile','outside.json'],['--bomb-profile=outside.json']]:
            result=self.call(out,*option,'--out',str(runtime),'--port','0')
            self.assertNotEqual(result.returncode,0);self.assertFalse(runtime.exists())
        p=out/'four-bomb-profile.json';p.write_bytes(p.read_bytes()+b' ')
        result=self.call(out,'--out',str(runtime),'--port','0')
        self.assertNotEqual(result.returncode,0);self.assertIn('bundle file mismatch',result.stderr)
        self.assertFalse(runtime.exists())

    def test_unmeasured_or_bool_policy_is_not_silently_exported(self):
        for caps in [dict.fromkeys('ABCD',2),{'A':True,'B':2,'C':1,'D':1}]:
            policy=copy.deepcopy(SPEC);policy['capacities']=caps;out=self.root/'not-created'
            with self.assertRaisesRegex(ValueError,'only measured'):
                export_bundle(self.profile,self.source,out,four_bomb_profile=policy)
            self.assertFalse(out.exists())

    def test_eight_launcher_relocates_and_refuses_four_combat_policy(self):
        out=self.root/'eight'
        manifest=export_bundle(self.profile,self.source,out,eight_sessions=True)
        self.assertEqual(manifest['entrypoint'],'server.eight_player_service')
        moved=self.root/'moved';shutil.move(out,moved);shutil.rmtree(self.source)
        self.assertEqual(self.call(moved,'--verify-only').returncode,0)
        result=self.call(moved,'--help')
        self.assertEqual(result.returncode,0);self.assertIn('--source-ip',result.stdout)
        self.assertIn('--bomb-profile',result.stdout)
        with self.assertRaisesRegex(ValueError,'excludes four Battle'):
            export_bundle(self.profile,moved,self.root/'invalid',eight_sessions=True,four_bomb_profile=SPEC)
        p=moved/'bundle.json';j=json.loads(p.read_text())
        j['files']=[q for q in j['files'] if q['path']!='server/eight_player_profile.py']
        p.write_text(json.dumps(j))
        self.assertIn('incomplete bundle manifest',self.call(moved,'--verify-only').stderr)


if __name__=='__main__':
    unittest.main()
