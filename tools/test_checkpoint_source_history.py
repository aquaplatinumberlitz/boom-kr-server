"""Publication-bound source recovery tests; no RE/acceptance certification."""
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile
import unittest

import check_checkpoints as checker
import checkpoint_store as store


class SourceHistoryTests(unittest.TestCase):
    def setUp(self):
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        self.root = Path(tmp.name)
        self.git = ['git', '-C', str(self.root), '-c', 'user.name=test',
                    '-c', 'user.email=test@example.invalid']
        subprocess.run(self.git + ['init', '-q'], check=True)

    def commit(self):
        subprocess.run(self.git + ['add', '.'], check=True)
        subprocess.run(self.git + ['commit', '-qm', 'synthetic publication'], check=True)

    def fixture(self, relative='server/service.py', external=False, broken_reference=False):
        path = self.root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        raw = b'original fixture bytes\n'
        path.write_bytes(raw)
        item = dict(id='E1', kind='repository', path=relative, bytes=len(raw),
                    sha256=hashlib.sha256(raw).hexdigest())
        record = dict(id='CP001', lifecycle='complete', evidence=[item])
        if external:
            ep = 'docs/evidence/checkpoints/evidence/CP001.json'
            data = store.encoded(dict(schema=store.EVIDENCE_SCHEMA, evidence=[item]))
            p = self.root / ep
            p.parent.mkdir(parents=True, exist_ok=True)
            p.write_bytes(data)
            record.update(evidence=[], evidence_manifests=[dict(
                path=ep, bytes=len(data),
                sha256='f' * 64 if broken_reference else hashlib.sha256(data).hexdigest(), count=1)])
        record_path = self.root / 'docs/evidence/checkpoints/records/CP001.json'
        record_path.parent.mkdir(parents=True, exist_ok=True)
        record_path.write_text(json.dumps(record))
        self.commit()
        return item, path

    def verify(self, item):
        return checker.verify_evidence_item(item, self.root, True, 'CP001')

    def test_inline_pin_recovers_exact_source_at_record_publication(self):
        item, path = self.fixture()
        path.write_text('new working source\n')
        errors, warnings, checked, missing = self.verify(item)
        self.assertEqual((errors, checked, missing), ([], 1, 0))
        self.assertTrue(any('HISTORICAL SOURCE VERIFIED' in w for w in warnings))

    def test_external_pin_requires_original_evidence_manifest_hash(self):
        item, path = self.fixture('tools/tool.py', external=True)
        path.write_text('new source\n')
        self.assertEqual(self.verify(item)[0], [])
        # Verify publication bytes, not an edited working evidence list.
        ep = self.root / 'docs/evidence/checkpoints/evidence/CP001.json'
        ep.write_text('{}')
        self.assertEqual(self.verify(item)[0], [])

    def test_later_matching_source_does_not_repair_a_different_publication_pin(self):
        item, path = self.fixture()
        later = b'later source\n'
        path.write_bytes(later)
        self.commit()
        requested = item | dict(bytes=len(later), sha256=hashlib.sha256(later).hexdigest())
        path.write_text('third version\n')
        self.assertTrue(self.verify(requested)[0])
        self.assertTrue(self.verify(item | {'sha256': 'f' * 64})[0])

    def test_matching_old_source_without_publication_blob_is_not_enough(self):
        item, path = self.fixture()
        path.write_text('new working source\n')
        # A different checkpoint cannot claim CP001's birth/source as its own.
        errors = checker.verify_evidence_item(item, self.root, True, 'CP002')[0]
        self.assertTrue(errors)

    def test_bad_publication_evidence_manifest_hash_cannot_rescue_source(self):
        item, path = self.fixture(external=True, broken_reference=True)
        path.write_text('new working source\n')
        self.assertTrue(self.verify(item)[0])

    def test_binary_local_or_unpublished_source_never_uses_fallback(self):
        item, path = self.fixture('server/client.bin')
        path.write_text('changed binary fixture\n')
        self.assertTrue(self.verify(item)[0])
        self.assertIsNone(checker.historical_checkpoint_source(
            self.root, 'local/source.py', item['sha256'], item['bytes'], 'CP001'))
        source = self.root / 'server/not-published.py'
        source.write_text('current\n')
        self.assertTrue(self.verify(item | {'path': 'server/not-published.py'})[0])

    def test_catalog_metadata_requires_exact_publication_not_later_matches(self):
        item, path = self.fixture('docs/features.json')
        later = b'{"later": true}\n'
        path.write_bytes(later)
        self.commit()
        path.write_text('{"current": true}\n')
        errors, warnings, checked, missing = self.verify(item)
        self.assertEqual((errors, checked, missing), ([], 1, 0))
        self.assertTrue(any('HISTORICAL METADATA DOC VERIFIED' in w for w in warnings))
        requested = item | dict(bytes=len(later), sha256=hashlib.sha256(later).hexdigest())
        self.assertTrue(self.verify(requested)[0])
        self.assertTrue(checker.verify_evidence_item(item, self.root, True, 'CP002')[0])

    def test_issue_snapshot_metadata_keeps_external_manifest_authentication(self):
        item, path = self.fixture(
            'docs/evidence/issue-reconciliation/github-snapshot.json', external=True)
        path.write_text('{"current": true}\n')
        self.assertEqual(self.verify(item)[0], [])
        self.assertTrue(self.verify(item | {'sha256': 'f' * 64})[0])

    def test_issue_review_metadata_can_evolve_after_publication(self):
        item, path = self.fixture('docs/evidence/issue-reconciliation/review.json')
        path.write_text('{"current": true}\n')
        self.assertEqual(self.verify(item)[0], [])

    def test_other_json_evidence_cannot_use_metadata_history_fallback(self):
        item, path = self.fixture('docs/evidence/rust-service/runtime.json')
        path.write_text('{"changed": true}\n')
        self.assertTrue(self.verify(item)[0])


if __name__ == '__main__':
    unittest.main()
