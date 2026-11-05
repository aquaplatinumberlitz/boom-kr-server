"""Storage/migration/history negative controls; synthetic data has no RE meaning."""
import copy
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

import checkpoint_store as store
import check_checkpoints as checker
from migrate_checkpoints import migrate
import test_checkpoints as legacy_fixtures


class StoreTests(unittest.TestCase):
    def setUp(self):
        # Reuse the fixture without discovering its TestCase a second time.
        fixture = legacy_fixtures.CheckpointsTest()
        fixture.setUp()
        self.addCleanup(fixture.doCleanups)
        self.root, self.v1 = fixture.root, fixture.manifest
        self.source = self.root / 'baseline.json'
        self.source.write_bytes(store.encoded(self.v1))
        self.target = self.root / store.MANIFEST
        self.target.parent.mkdir(parents=True)
        self.target.write_bytes(self.source.read_bytes())

    def migration(self):
        return migrate(self.source, self.root, 'a' * 40, 'docs/evidence/checkpoints/migration-v2.json')

    def test_all_objects_evidence_audits_and_counters_equal_rerun_keeps_bytes(self):
        report = self.migration()
        v2 = store.load(self.root)
        self.assertEqual(list(store.records(v2, self.root)), self.v1['checkpoints'])
        for key in self.v1.keys() - {'schema', 'checkpoints'}:
            self.assertEqual(v2[key], self.v1[key])
        self.assertEqual(checker.validate(v2, self.root, True)[0], [])
        before = {p: p.read_bytes() for p in self.target.parent.rglob('*.json')}
        self.assertEqual(self.migration()['record_mappings'], report['record_mappings'])
        self.assertEqual(before, {p: p.read_bytes() for p in before})
        # Entire objects, not just selected fields, remain equivalent.
        self.assertTrue(report['comparison']['all_inline_evidence_equal'])
        self.assertEqual(checker.signature_continuity(checker.history_signature(self.v1, self.root),
                                                      checker.history_signature(v2, self.root)), [])

    def test_migration_refuses_changed_existing_record_and_preserves_index(self):
        self.migration()
        v2 = store.load(self.root)
        record_path = self.root / v2['checkpoints'][0]['path']
        obj = json.loads(record_path.read_bytes())
        obj['findings'][0]['summary'] = 'changed complete finding'
        record_path.write_bytes(store.encoded(obj))
        before = self.target.read_bytes()
        with self.assertRaisesRegex(ValueError, 'overwrite changed record'):
            self.migration()
        self.assertEqual(self.target.read_bytes(), before)
        with self.assertRaisesRegex(ValueError, 'hash mismatch'):
            store.records(v2, self.root)[0]

    def test_selective_latest_view_reads_one_record_not_entire_history(self):
        v1 = copy.deepcopy(self.v1)
        for i in range(2, 4):
            cp = copy.deepcopy(v1['checkpoints'][0])
            cp.update(id=f'CP{i:03}', previous=f'CP{i-1:03}')
            v1['checkpoints'].append(cp)
        v1.update(last_id='CP003', completed_count=3)
        self.source.write_bytes(store.encoded(v1))
        self.target.write_bytes(self.source.read_bytes())
        self.migration()
        v2 = store.load(self.root)
        reader = store.file_reader(self.root)
        with patch.object(store, 'file_reader', return_value=reader) as factory:
            calls = []
            def observe(path):
                calls.append(path)
                return reader(path)
            factory.return_value = observe
            current = store.current_view(v2, self.root)
        self.assertEqual(len(calls), 1)
        self.assertTrue(calls[0].endswith('CP003.json'))
        self.assertEqual(current['checkpoints'][0]['id'], 'CP003')

    def external(self):
        self.migration()
        v2 = store.load(self.root)
        cp = store.records(v2, self.root)[0]
        evidence = cp['evidence']
        relative = 'docs/evidence/checkpoints/evidence/run-fixture.json'
        p = self.root / relative
        p.parent.mkdir()
        raw = store.encoded(dict(schema=store.EVIDENCE_SCHEMA, evidence=evidence))
        p.write_bytes(raw)
        cp['evidence'] = []
        cp['evidence_manifests'] = [dict(path=relative, sha256=hashlib.sha256(raw).hexdigest(),
                                         bytes=len(raw), count=len(evidence))]
        record_raw = store.encoded(cp)
        record_path = self.root / v2['checkpoints'][0]['path']
        record_path.write_bytes(record_raw)
        v2['checkpoints'][0] = store.record_reference(cp, v2['checkpoints'][0]['path'], record_raw)
        self.target.write_bytes(store.encoded(v2))
        return v2, cp, p

    def test_external_evidence_verified_and_corruption_or_missing_local_reported(self):
        v2, cp, path = self.external()
        self.assertEqual(checker.validate(v2, self.root, True)[0], [])
        self.assertEqual(checker.validate(v2, self.root, True)[2]['evidence_verified'], 1)
        local = self.root / 'local/sample.txt'
        local.write_text('bad')
        self.assertTrue(any('size/hash mismatch' in e for e in checker.validate(v2, self.root, True)[0]))
        local.unlink()
        self.assertEqual(checker.validate(v2, self.root, False)[2]['evidence_unavailable'], 1)
        self.assertTrue(any('UNVERIFIED' in e for e in checker.validate(v2, self.root, True)[0]))
        path.write_text('{}')
        with self.assertRaisesRegex(ValueError, 'hash mismatch'):
            list(store.evidence(cp, self.root))

    def test_wrong_reference_metadata_paths_counts_and_duplicate_ids_detected(self):
        v2, cp, path = self.external()
        bad = copy.deepcopy(v2)
        bad['checkpoints'][0]['lifecycle'] = 'in_progress'
        with self.assertRaisesRegex(ValueError, 'metadata mismatch'):
            store.records(bad, self.root)[0]
        bad = copy.deepcopy(v2)
        bad['checkpoints'][0]['path'] = '../escape'
        with self.assertRaisesRegex(ValueError, 'outside repository'):
            store.records(bad, self.root)[0]
        cp['evidence_manifests'][0]['count'] = 4
        with self.assertRaisesRegex(ValueError, 'count mismatch'):
            list(store.evidence(cp, self.root))
        cp['evidence_manifests'][0]['count'] = 1
        cp['evidence'] = list(store.evidence(cp, self.root))
        # Direct fixture v1 wrapper exercises duplicate IDs across inline/external.
        test = copy.deepcopy(self.v1)
        test['checkpoints'] = [cp]
        self.assertTrue(any('duplicate evidence' in e for e in checker.validate(test, self.root, True)[0]))

    def test_cli_summary_pages_and_verify_all_even_when_page_is_empty(self):
        self.external()
        command = ['python3', str(Path(__file__).with_name('checkpoints.py')),
                   '--root', str(self.root)]
        def run(*args):
            return subprocess.run(command + list(args), capture_output=True, text=True)
        latest = run('latest')
        self.assertEqual(latest.returncode, 0)
        self.assertEqual(json.loads(latest.stdout)['last_id'], 'CP001')
        show = json.loads(run('show', 'CP001').stdout)
        self.assertNotIn('evidence', show)
        summary = json.loads(run('evidence', 'CP001').stdout)
        self.assertEqual(summary['total'], 1)
        self.assertNotIn('rows', summary)
        page = json.loads(run('evidence', 'CP001', '--limit', '1').stdout)
        self.assertEqual(page['rows'][0]['id'], 'E1')
        self.assertEqual(run('evidence', 'CP001', '--limit', '201').returncode, 2)
        self.assertNotEqual(run('show', 'CP999').returncode, 0)
        (self.root / 'local/sample.txt').write_text('corrupt beyond selected page')
        result = run('evidence', 'CP001', '--offset', '99', '--limit', '1', '--verify', '--require-local')
        self.assertEqual(result.returncode, 1)
        self.assertEqual(json.loads(result.stdout)['rows'], [])
        self.assertFalse(json.loads(result.stdout)['verification']['ok'])

    def test_cadence_survives_migration_and_migration_cannot_reset_appended_counter(self):
        # Reuse legacy fixture helpers for a real fourth-checkpoint/audit boundary.
        fixture = legacy_fixtures.CheckpointsTest()
        fixture.setUp()
        self.addCleanup(fixture.doCleanups)
        self.root, self.v1 = fixture.root, fixture.manifest
        self.source = self.root / 'baseline.json'
        self.target = self.root / store.MANIFEST
        self.target.parent.mkdir(parents=True)
        for _ in range(3):
            fixture.append()
        self.source.write_bytes(store.encoded(self.v1))
        self.target.write_bytes(self.source.read_bytes())
        self.migration()
        self.assertTrue(any('MANUAL AUDIT DUE' in e for e in checker.validate(store.load(self.root), self.root)[0]))
        # A supplied periodic audit must retain its exact JSON/hash.
        fixture.audit('periodic', 'CP004')
        fixture.manifest.update(last_periodic_audit=4, next_periodic_audit='CP008')
        # Use a separate fixture tree to avoid replacing the already migrated index.
        self.v1.update(audits=fixture.manifest['audits'], last_manual_audit=fixture.manifest['last_manual_audit'],
                       last_periodic_audit=4, next_periodic_audit='CP008')
        index = store.load(self.root)
        index.update(audits=self.v1['audits'], last_manual_audit=self.v1['last_manual_audit'],
                     last_periodic_audit=4, next_periodic_audit='CP008')
        self.target.write_bytes(store.encoded(index))
        self.assertEqual(checker.validate(index, self.root)[0], [])
        fifth = copy.deepcopy(self.v1['checkpoints'][-1])
        fifth.update(id='CP005', previous='CP004')
        relative = 'docs/evidence/checkpoints/records/CP005.json'
        (self.root / relative).write_bytes(store.encoded(fifth))
        index['checkpoints'].append(store.record_reference(fifth, relative))
        index.update(last_id='CP005', completed_count=5)
        self.target.write_bytes(store.encoded(index))
        self.migration()
        self.assertEqual(store.load(self.root)['last_id'], 'CP005')

    def git(self, *args):
        subprocess.run(['git', '-C', str(self.root), '-c', 'user.name=Test',
                        '-c', 'user.email=test@example.invalid', *args], check=True,
                       capture_output=True)

    def commit(self, message):
        self.git('add', 'docs')
        self.git('commit', '-qm', message)

    def test_history_cross_schema_and_record_only_tampering_in_intermediate_commit(self):
        self.git('init', '-q')
        self.commit('v1')
        self.migration()
        self.commit('v2 same objects')
        v2 = store.load(self.root)
        self.assertEqual(checker.check_history(self.root, store.MANIFEST, v2), [])
        path = self.root / v2['checkpoints'][0]['path']
        before = path.read_bytes()
        cp = json.loads(before)
        cp['scope'] = 'tampered without index change'
        path.write_bytes(store.encoded(cp))
        self.commit('record only tamper')
        path.write_bytes(before)
        self.commit('restore record')
        errors = checker.check_history(self.root, store.MANIFEST, store.load(self.root))
        self.assertTrue(any('hash mismatch' in e for e in errors))

    def test_deleted_record_new_hash_rewrite_and_counter_break_rejected(self):
        self.git('init', '-q')
        self.commit('v1')
        self.migration()
        self.commit('v2')
        v2 = store.load(self.root)
        path = self.root / v2['checkpoints'][0]['path']
        path.unlink()
        with self.assertRaises(FileNotFoundError):
            checker.history_signature(v2, self.root)
        self.migration()
        cp = store.records(v2, self.root)[0]
        cp['scope'] = 'rewritten but repinned'
        raw = store.encoded(cp)
        path.write_bytes(raw)
        v2['checkpoints'][0] = store.record_reference(cp, v2['checkpoints'][0]['path'], raw)
        self.target.write_bytes(store.encoded(v2))
        self.commit('repinned old finding')
        self.assertTrue(any('completed record changed' in e for e in checker.check_history(self.root, store.MANIFEST, v2)))
        v2['completed_count'] = 0
        self.assertTrue(any('counter' in e for e in checker.check_history(self.root, store.MANIFEST, v2)))

    def test_external_manifest_only_tamper_is_seen_in_git_history(self):
        self.git('init', '-q')
        v2, cp, path = self.external()
        self.commit('new external evidence fixture')
        before = path.read_bytes()
        path.write_text('{}')
        self.commit('external manifest only tamper')
        path.write_bytes(before)
        self.commit('restore external manifest')
        self.assertTrue(any('hash mismatch' in e for e in checker.check_history(self.root, store.MANIFEST, v2)))


