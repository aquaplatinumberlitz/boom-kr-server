"""Lean evidence storage: migration equivalence and tamper controls. Synthetic data, no RE meaning."""
import contextlib
import copy
import hashlib
import io
import json
from pathlib import Path
import subprocess
import tempfile
import unittest

import check_checkpoints as checker
import checkpoint_evidence as lean
import checkpoint_store as store
import checkpoints as cli
import evidence_lookup
from migrate_checkpoint_evidence import migrate, REPORT

DOCS = (*checker.CORE_DOCS, 'docs/evidence/README.md')


class LeanEvidenceTest(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        for name in DOCS:
            self.write(name, 'synthetic schema test\n')
        self.write('docs/evidence/sample.json', '{"synthetic": true}\n')
        files = ['local/research/codex/run-a/one.txt', 'local/research/codex/run-a/two.txt',
                 'local/research/codex/run-a/deep/three.txt', 'local/research/codex/run-a/four.txt',
                 'local/runs/b/x.txt', 'local/runs/b/y.txt', 'local/runs/b/z.txt',
                 'local/single/only.txt']
        rows = [self.row('R1', 'repository', 'docs/evidence/sample.json')]
        for number, name in enumerate(files, 2):
            self.write(name, f'synthetic raw {number}\n')
            rows.append(self.row(f'R{number}', 'local', name))
        self.git('init', '-q')
        self.rows = rows
        record = self.record('CP001', 'origin', rows)
        self.manifest = {
            'schema': store.V2, 'version': 2, 'last_id': 'CP001', 'completed_count': 1,
            'audit_interval': 4, 'last_periodic_audit': 0, 'next_periodic_audit': 'CP004',
            'last_manual_audit': None, 'checkpoints': [], 'audits': [],
            'task_states': [{'id': 'T1', 'state': 'open', 'canonical': 'docs/PLAN.md', 'reference': 'test'}]}
        self.put(record)
        self.commit('CP001 legacy storage')

    # -- fixture helpers ---------------------------------------------------------------
    def write(self, name, text):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)
        return path

    def row(self, ident, kind, name):
        data = (self.root / name).read_bytes()
        return {'id': ident, 'kind': kind, 'path': name, 'bytes': len(data),
                'sha256': hashlib.sha256(data).hexdigest()}

    def record(self, ident, previous, rows):
        return {
            'id': ident, 'previous': previous, 'lifecycle': 'complete', 'recorded_utc': '2026-11-17T00:00:00Z',
            'question': 'schema fixture', 'scope': 'no RE claims', 'context_commit': 'a' * 40,
            'artifacts': [{'name': 'fixture', 'sha256': 'b' * 64}], 'locators': ['fixture'],
            'owner': 'author', 'reviewer': 'reviewer', 'evidence': rows,
            'findings': [{'summary': 'fixture', 'source': 'design', 'status': 'design', 'method': 'documentation',
                          'limits': 'schema only', 'canonical': 'docs/PROTOCOL.md', 'evidence': ['R1', 'R3']}],
            'verification': [{'check': 'fixture', 'result': 'pass', 'evidence': ['R2'], 'limits': 'schema only'}],
            'delta': {'added': ['fixture'], 'corrected': [], 'superseded': []}, 'blockers': [],
            'next_steps': [], 'review': {'status': 'pending', 'as_of_utc': '2026-11-17T00:00:00Z',
                                         'reference': 'test', 'scope': 'schema'},
            'docs_sync': [{'path': p, 'state': 'updated', 'reason': 'fixture'} for p in DOCS]}

    def put(self, record, position=None):
        if (self.root / store.MANIFEST).exists():
            self.manifest = store.load(self.root)  # the migration rewrites the index
        relative = f'docs/evidence/checkpoints/records/{record["id"]}.json'
        raw = store.encoded(record)
        self.write(relative, '').write_bytes(raw)
        reference = store.record_reference(record, relative, raw)
        if position is None:
            self.manifest['checkpoints'].append(reference)
        else:
            self.manifest['checkpoints'][position] = reference
        self.manifest.update(last_id=self.manifest['checkpoints'][-1]['id'],
                             completed_count=len(self.manifest['checkpoints']))
        self.write(str(store.MANIFEST), '').write_bytes(store.encoded(self.manifest))

    def git(self, *args):
        return subprocess.run(['git', '-c', 'user.name=test', '-c', 'user.email=test@example.invalid',
                               '-C', str(self.root), *args], check=True, capture_output=True, text=True).stdout

    def commit(self, message):
        self.git('add', '-A', 'docs')
        self.git('commit', '-qm', message)
        return self.git('rev-parse', 'HEAD').strip()

    def validate(self, require_local=True):
        return checker.validate(store.load(self.root), self.root, require_local)

    def current(self):
        index = store.load(self.root)
        return index, store.records(index, self.root)[0]

    def migrated(self):
        before = self.validate()
        stats = migrate(self.root, 'HEAD')
        self.commit('lean evidence migration')
        return before, stats

    # -- migration equivalence -----------------------------------------------------------
    def test_migration_keeps_gate_result_identity_and_is_idempotent(self):
        _, legacy = self.current()
        legacy_digest = lean.logical_digest(legacy, self.root)
        before, stats = self.migrated()
        self.assertEqual(stats['records_migrated'], 1)
        self.assertEqual(stats['rows_moved'], 5)  # run-a minus cited R2/R3 (2), runs/b (3); single stays
        after = self.validate()
        self.assertEqual(before[0], [])
        self.assertEqual(after[0], [])
        self.assertEqual(before[2], after[2])
        self.assertTrue(any('LEGACY EVIDENCE STORAGE' in w for w in before[1]))
        self.assertEqual([w for w in before[1] if 'LEGACY' not in w], after[1])
        _, packed = self.current()
        self.assertEqual(lean.logical_digest(packed, self.root), legacy_digest)
        self.assertEqual(sorted(map(lean.canonical, lean.all_rows(packed, self.root))),
                         sorted(map(lean.canonical, self.rows)))
        self.assertEqual({r['id'] for r in packed['evidence']}, {'R1', 'R2', 'R3', 'R9'})
        self.assertEqual([p['run'] for p in packed['evidence_runs']], ['local/research/codex/run-a', 'local/runs/b'])
        self.assertEqual(checker.check_history(self.root, checker.MANIFEST, store.load(self.root)), [])
        snapshot = {p: p.read_bytes() for p in self.root.rglob('*') if p.is_file() and '.git' not in p.parts}
        self.assertEqual(migrate(self.root, 'HEAD')['records_migrated'], 0)
        self.assertEqual(snapshot, {p: p.read_bytes() for p in snapshot})
        report = json.loads((self.root / REPORT).read_bytes())
        self.assertEqual(report['records'][0]['logical_sha256'], legacy_digest)

    def test_legacy_external_manifest_lists_are_migrated_and_deleted(self):
        index, record = self.current()
        relative = 'docs/evidence/checkpoints/evidence/CP001-list.json'
        raw = store.encoded({'schema': store.EVIDENCE_SCHEMA, 'evidence': record['evidence']})
        self.write(relative, '').write_bytes(raw)
        external = dict(record, evidence=[], evidence_manifests=[{'path': relative, 'bytes': len(raw),
                                                                  'sha256': hashlib.sha256(raw).hexdigest(),
                                                                  'count': len(record['evidence'])}])
        self.put(external, 0)
        self.commit('external list')
        digest = lean.logical_digest(external, self.root)
        self.assertEqual(digest, lean.logical_digest(record, self.root))  # storage-independent identity
        before, stats = self.migrated()
        self.assertEqual(stats['evidence_files_deleted'], 1)
        self.assertFalse((self.root / relative).exists())
        after = self.validate()
        self.assertEqual((after[0], after[2]), ([], before[2]))
        self.assertEqual(lean.logical_digest(self.current()[1], self.root), digest)
        self.assertEqual(checker.check_history(self.root, checker.MANIFEST, store.load(self.root)), [])

    # -- tamper controls (each must fail exactly like the legacy list) ---------------------
    def test_altered_byte_in_local_file_detected(self):
        self.migrated()
        path = self.root / 'local/runs/b/y.txt'
        path.write_bytes(path.read_bytes().replace(b'raw', b'RAW'))
        self.assertTrue(any('size/hash mismatch local/runs/b/y.txt' in e for e in self.validate()[0]))

    def test_missing_and_renamed_local_files_detected(self):
        self.migrated()
        (self.root / 'local/runs/b/y.txt').rename(self.root / 'local/runs/b/renamed.txt')
        errors, warnings, stats = self.validate()
        self.assertTrue(any('UNVERIFIED unavailable host evidence local/runs/b/y.txt' in e for e in errors))
        errors, warnings, stats = self.validate(require_local=False)
        self.assertEqual(errors, [])
        self.assertEqual(stats['evidence_unavailable'], 1)

    def test_extra_or_altered_manifest_row_detected(self):
        self.migrated()
        _, record = self.current()
        path = self.root / record['evidence_runs'][1]['path']
        raw = path.read_bytes()
        extra = lean.canonical(self.row('R99', 'local', 'local/runs/b/x.txt')).encode() + b'\n'
        path.write_bytes(raw + extra)
        self.assertTrue(any('evidence run manifest' in e and 'size/hash mismatch' in e for e in self.validate()[0]))
        path.write_bytes(raw.replace(b'"bytes":', b'"bytes": ', 1))
        self.assertTrue(any('size/hash mismatch' in e for e in self.validate()[0]))
        path.write_bytes(raw)
        self.assertEqual(self.validate()[0], [])

    def test_extra_file_in_run_directory_is_not_evidence_in_either_format(self):
        # Parity: legacy lists never pinned unlisted files either; this is not a regression.
        self.write('local/runs/b/unlisted.txt', 'new\n')
        self.assertEqual(self.validate()[0], [])
        self.migrated()
        self.assertEqual(self.validate()[0], [])

    def test_missing_manifest_is_unverified_not_silent(self):
        self.migrated()
        _, record = self.current()
        pointer = record['evidence_runs'][0]
        (self.root / pointer['path']).unlink()
        errors = self.validate()[0]
        self.assertTrue(any('UNVERIFIED unavailable host evidence run manifest' in e for e in errors))
        errors, warnings, stats = self.validate(require_local=False)
        self.assertEqual(errors, [])
        self.assertEqual(stats['evidence_unavailable'], pointer['count'])
        self.assertTrue(any('run manifest' in w for w in warnings))
        # Restore rebuilds the exact bytes from the pre-migration list in Git.
        with contextlib.redirect_stdout(io.StringIO()):
            self.assertEqual(evidence_lookup.main(['--root', str(self.root), 'restore']), 0)
        self.assertEqual(self.validate()[0], [])

    def test_altered_repository_file_detected(self):
        self.migrated()
        self.write('docs/evidence/sample.json', '{"synthetic": false}\n')
        self.assertTrue(any('size/hash mismatch docs/evidence/sample.json' in e for e in self.validate()[0]))

    def test_record_pointer_or_noncanonical_packing_detected(self):
        self.migrated()
        index, record = self.current()
        changed = copy.deepcopy(record)
        changed['evidence_runs'][0]['sha256'] = 'f' * 64
        changed['evidence_runs'][0]['path'] = lean.manifest_path('f' * 64)
        self.put(changed, 0)
        self.assertTrue(any('run manifest' in e for e in self.validate()[0]))
        # Moving a run back inline keeps every row verifiable but is not the canonical form.
        unpacked = copy.deepcopy(record)
        unpacked['evidence'] += lean.read_run(record['evidence_runs'][1], self.root)
        unpacked['evidence_runs'] = unpacked['evidence_runs'][:1]
        self.put(unpacked, 0)
        self.assertTrue(any('not the canonical packing' in e for e in self.validate()[0]))

    def test_history_still_rejects_changed_complete_record_after_migration(self):
        self.migrated()
        _, record = self.current()
        changed = copy.deepcopy(record)
        changed['findings'][0]['summary'] = 'rewritten conclusion'
        self.put(changed, 0)
        errors = checker.check_history(self.root, checker.MANIFEST, store.load(self.root))
        self.assertTrue(any('completed record changed' in e for e in errors))
        # Swapping a run for a different, internally valid list (one row dropped) is a content change.
        changed = copy.deepcopy(record)
        rows = lean.read_run(record['evidence_runs'][1], self.root)[:-1]
        pointer, raw = lean.pack({}, rows)[1][0]
        lean.write_manifest(self.root, pointer, raw)
        changed['evidence_runs'][1] = pointer
        self.put(changed, 0)
        self.assertEqual(self.validate()[0], [])
        errors = checker.check_history(self.root, checker.MANIFEST, store.load(self.root))
        self.assertTrue(any('completed record changed' in e for e in errors))

    def test_checkpoint_storage_pinned_by_later_record_stays_verifiable(self):
        cp1_raw = (self.root / 'docs/evidence/checkpoints/records/CP001.json').read_bytes()
        pin = {'id': 'S1', 'kind': 'repository', 'path': 'docs/evidence/checkpoints/records/CP001.json',
               'bytes': len(cp1_raw), 'sha256': hashlib.sha256(cp1_raw).hexdigest()}
        second = self.record('CP002', 'CP001', [self.rows[0], pin])
        second['findings'][0]['evidence'] = ['R1', 'S1']
        second['verification'][0]['evidence'] = ['S1']
        self.put(second)
        self.commit('CP002 pins CP001 storage')
        self.assertEqual(self.validate()[0], [])
        self.migrated()
        errors, warnings, _ = self.validate()
        self.assertEqual(errors, [])
        self.assertTrue(any('HISTORICAL CHECKPOINT STORAGE VERIFIED' in w for w in warnings))
        index, _ = self.current()
        cp2 = store.records(index, self.root)[1]
        cp2['evidence'][1]['sha256'] = 'e' * 64
        self.put(cp2, 1)
        self.assertTrue(any('size/hash mismatch docs/evidence/checkpoints/records/CP001.json' in e
                            for e in self.validate()[0]))

    # -- writer helper and lookup ------------------------------------------------------
    def test_pack_evidence_helper_for_new_checkpoint(self):
        self.migrated()
        record = self.record('CP002', 'CP001', self.rows)
        packed = lean.pack_evidence(record, self.root)
        self.assertNotIn('evidence_manifests', packed)
        self.assertEqual(len(packed['evidence_runs']), 2)
        self.put(packed)
        errors, _, stats = self.validate()
        self.assertEqual(errors, [])
        self.assertEqual(stats['evidence_verified'], 2 * len(self.rows))

    def test_selective_cli_counts_and_verifies_run_rows(self):
        self.migrated()
        out = io.StringIO()
        with contextlib.redirect_stdout(out):
            code = cli.main(['--root', str(self.root), 'evidence', 'CP001', '--verify', '--require-local'])
        result = json.loads(out.getvalue())
        self.assertEqual((code, result['total'], result['verification']['verified']), (0, 9, 9))
        pointer = self.current()[1]['evidence_runs'][0]
        (self.root / pointer['path']).unlink()
        out = io.StringIO()
        with contextlib.redirect_stdout(out):
            code = cli.main(['--root', str(self.root), 'evidence', 'CP001', '--verify', '--require-local'])
        result = json.loads(out.getvalue())
        self.assertEqual((code, result['verification']['unavailable']), (1, pointer['count']))

    def lookup(self, *args):
        out = io.StringIO()
        with contextlib.redirect_stdout(out), contextlib.redirect_stderr(io.StringIO()):
            code = evidence_lookup.main(['--root', str(self.root), *args])
        return code, [json.loads(line) for line in out.getvalue().splitlines() if line.startswith('{')]

    def test_lookup_by_path_sha_id_cp_and_history(self):
        source = self.git('rev-parse', 'HEAD').strip()
        self.migrated()
        code, rows = self.lookup('--verify', 'path', 'local/runs/b')
        self.assertEqual((code, sorted(r['id'] for r in rows)), (0, ['R6', 'R7', 'R8']))
        self.assertTrue(all(r['verify'] == 'ok' and 'evidence-manifests' in r['stored_in'] for r in rows))
        code, rows = self.lookup('path', str(self.root / 'local/research/codex/run-a/deep/three.txt'))
        self.assertEqual([r['id'] for r in rows], ['R4'])
        code, rows = self.lookup('sha', self.rows[6]['sha256'][:12])
        self.assertEqual([r['id'] for r in rows], ['R7'])
        code, rows = self.lookup('--cp', 'CP001', 'id', 'R3')
        self.assertEqual(rows[0]['stored_in'], 'docs/evidence/checkpoints/records/CP001.json')
        code, rows = self.lookup('--from-history', source, 'path', 'local/runs/b/x.txt')
        self.assertEqual(rows[0]['stored_in'], 'docs/evidence/checkpoints/records/CP001.json')
        self.assertEqual(self.lookup('path', 'local/nothing')[0], 1)


if __name__ == '__main__':
    unittest.main()
