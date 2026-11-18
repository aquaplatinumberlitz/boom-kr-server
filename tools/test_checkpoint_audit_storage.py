"""Audit storage migration must preserve exact same-revision evidence."""
import copy
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile
import unittest

import check_checkpoints as checker
import checkpoint_store as store


class AuditStorageTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.git = ['git', '-C', str(self.root), '-c', 'user.name=test',
                    '-c', 'user.email=test@example.invalid']
        subprocess.run(self.git + ['init', '-q'], check=True)
        self.audit = dict(id='AUD001', trigger='feature_group',
                          completed_through='CP001', findings=['retained evidence'])
        self.path = 'docs/evidence/checkpoints/audits/AUD001.json'
        self.raw = store.encoded(self.audit)
        target = self.root / self.path
        target.parent.mkdir(parents=True)
        target.write_bytes(self.raw)
        self.manifest = dict(schema=store.V2, version=2, checkpoints=[],
                             last_id=None, completed_count=0, audits=[self.audit])

    def commit(self):
        target = self.root / store.MANIFEST
        target.write_bytes(store.encoded(self.manifest))
        subprocess.run(self.git + ['add', '.'], check=True)
        subprocess.run(self.git + ['commit', '-qm', 'audit storage fixture'], check=True)

    def reference(self):
        return dict(id='AUD001', path=self.path, sha256=hashlib.sha256(self.raw).hexdigest())

    def test_complete_embedded_audit_migrates_without_changing_raw_or_history(self):
        self.commit()
        self.manifest['audits'] = [self.reference()]
        self.commit()
        self.assertEqual(checker.check_history(self.root, store.MANIFEST, self.manifest), [])

    def test_inline_needs_exact_same_revision_file_and_cannot_borrow_later_file(self):
        (self.root / self.path).unlink()
        self.commit()
        (self.root / self.path).write_bytes(self.raw)
        self.manifest['audits'] = [self.reference()]
        self.commit()
        self.assertTrue(checker.check_history(self.root, store.MANIFEST, self.manifest))

    def test_inline_body_and_external_identity_must_match_entire_object(self):
        changed = copy.deepcopy(self.audit)
        changed['findings'] = ['changed conclusion']
        (self.root / self.path).write_bytes(store.encoded(changed))
        with self.assertRaisesRegex(ValueError, 'embedded audit differs'):
            checker.historical_audit_references(self.manifest, self.root)

    def test_reference_bad_hash_or_partial_reference_cannot_fall_back_to_inline(self):
        for row in [self.reference() | dict(sha256='f' * 64),
                    self.audit | dict(path=self.path),
                    self.reference() | dict(bytes=len(self.raw) + 1)]:
            with self.subTest(row=row), self.assertRaises(ValueError):
                checker.historical_audit_references(dict(audits=[row]), self.root)

    def test_audit_file_only_tampering_and_reference_reordering_remain_errors(self):
        self.commit()
        self.manifest['audits'] = [self.reference()]
        self.commit()
        (self.root / self.path).write_bytes(self.raw + b'\n')
        self.commit()
        self.assertTrue(checker.check_history(self.root, store.MANIFEST, self.manifest))
        before = checker.historical_audit_references(dict(audits=[self.audit]), self.root,
                                                    read=lambda _: self.raw)
        self.assertTrue(checker.signature_continuity(
            dict(checkpoints=[], audits=before), dict(checkpoints=[], audits=[])))


if __name__ == '__main__':
    unittest.main()
