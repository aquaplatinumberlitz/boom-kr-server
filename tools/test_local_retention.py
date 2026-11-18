"""Retention plan/apply and PRUNED gate status. Synthetic data, no RE meaning."""
import contextlib
import hashlib
import io
import json
import os
from pathlib import Path
import time
import unittest

import check_checkpoints as checker
import checkpoint_evidence as lean
import checkpoint_store as store
import local_retention as retention
import test_checkpoint_evidence_lean as lean_fixture


class RetentionTest(unittest.TestCase):
    def setUp(self):
        # Reuse the lean-evidence fixture without inheriting (and re-running) its tests.
        fixture = lean_fixture.LeanEvidenceTest('setUp')
        fixture.setUp()
        self.addCleanup(fixture.doCleanups)
        self.fixture, self.root = fixture, fixture.root
        self.write, self.validate, self.migrated = fixture.write, fixture.validate, fixture.migrated
        self.write('local/research/codex/run-a/game-A/client.exe', 'synthetic game copy\n')
        self.write('local/research/codex/run-a/prefix-A/drive_c/x.reg', 'synthetic prefix\n')
        self.write('local/research/codex/run-a/scratch.bin', 'intermediate\n')
        self.write('local/research/codex/run-a/crash.dmp', 'dump\n')
        self.write('local/research/codex/unused-r1/out.log', 'never referenced\n')
        fixture = self.write('local/research/codex/fixture-r1/frame.bin', 'pinned frame\n')
        data = fixture.read_bytes()
        self.write('local/research/codex/fixture-r1/service.json', json.dumps({
            'schema': 'kr-rust-diagnostic-v1', 'fixture': {
                'path': 'local/research/codex/fixture-r1/frame.bin', 'bytes': len(data),
                'sha256': hashlib.sha256(data).hexdigest()}}))
        self.migrated()

    def dry_run(self, days=0, now=None):
        # Fixture files are created just now; the guard is off (0) unless a test exercises it.
        with contextlib.redirect_stdout(io.StringIO()):
            path, digest, summary, report = retention.plan(self.root, False, True, 'local/retention', days, now)
        return path, digest, report

    def test_dry_run_classes_and_pins(self):
        path, digest, report = self.dry_run()
        actions = {a['path']: a['class'] for a in report['actions']}
        self.assertEqual(actions['local/research/codex/run-a/game-A'], 'disposable_copy')
        self.assertEqual(actions['local/research/codex/run-a/prefix-A'], 'disposable_copy')
        self.assertEqual(actions['local/research/codex/run-a/crash.dmp'], 'disposable_dump')
        self.assertEqual(actions['local/research/codex/run-a/scratch.bin'], 'disposable_intermediate')
        self.assertEqual(actions['local/research/codex/unused-r1'], 'disposable_unreferenced_run')
        self.assertNotIn('local/research/codex/fixture-r1', actions)
        self.assertIn('local/research/codex/fixture-r1/frame.bin', report['pinned']['fixture_pins'])
        self.assertIn('local/research/codex/fixture-r1/service.json', report['pinned']['configs'])
        self.assertFalse(any(p.startswith('local/evidence-manifests') for p in actions))
        self.assertTrue(all(p.startswith('local/') for p in actions))
        self.assertTrue(summary_exists(path))
        self.assertEqual(report['actions'][0]['action'], 'delete')
        # Dry run deletes nothing.
        self.assertTrue((self.root / 'local/research/codex/run-a/game-A/client.exe').exists())

    def test_apply_requires_approved_hash_and_records_pruned_evidence(self):
        path, digest, report = self.dry_run()
        with self.assertRaisesRegex(ValueError, 'approved hash'):
            retention.apply(self.root, path, 'f' * 64)
        result = retention.apply(self.root, path, digest)
        self.assertGreater(result['deleted'], 0)
        self.assertFalse((self.root / 'local/research/codex/unused-r1').exists())
        self.assertTrue((self.root / 'local/research/codex/fixture-r1/frame.bin').exists())
        self.assertEqual(self.validate()[0], [])

    def test_gate_pruned_is_reported_but_unregistered_missing_still_fails(self):
        target = self.root / 'local/runs/b/y.txt'
        target.unlink()
        self.assertTrue(any('UNVERIFIED unavailable host evidence local/runs/b/y.txt' in e for e in self.validate()[0]))
        registry = {'schema': lean.PRUNED_SCHEMA, 'authorization': 'test', 'entries': [{
            'path': 'local/runs/b', 'class': 'disposable_intermediate', 'reason': 'synthetic',
            'date': '2026-11-18', 'plan_sha256': 'a' * 64}]}
        self.write(lean.PRUNED, json.dumps(registry))
        errors, warnings, stats = self.validate()
        self.assertEqual(errors, [])
        self.assertEqual(stats['evidence_pruned'], 1)
        self.assertTrue(any('PRUNED 1 host-local' in w for w in warnings))
        # A pruned path that still exists is verified normally.
        other = self.root / 'local/runs/b/x.txt'
        other.write_text('tampered\n')
        self.assertTrue(any('size/hash mismatch local/runs/b/x.txt' in e for e in self.validate()[0]))
        other.unlink()
        self.assertEqual(self.validate()[0], [])
        # Other missing files are still errors.
        (self.root / 'local/research/codex/run-a/four.txt').unlink()
        self.assertTrue(any('run-a/four.txt' in e for e in self.validate()[0]))
        registry['entries'][0]['path'] = 'local/evidence-manifests'
        self.write(lean.PRUNED, json.dumps(registry))
        self.assertTrue(any('local-pruned' in e for e in self.validate()[0]))

    def test_recent_unit_is_keep_recent_and_old_unreferenced_stays_disposable(self):
        later = time.time() + 30 * 86400  # every fixture file is 30 days old at this clock
        self.write('local/research/codex/wip-r2/notes.txt', 'in progress, unreferenced\n')
        wip = self.root / 'local/research/codex/wip-r2/notes.txt'
        os.utime(wip, (later - 3600, later - 3600))  # touched an hour ago (mtime)
        path, digest, report = self.dry_run(7, later)
        actions = {a['path']: a['class'] for a in report['actions']}
        self.assertNotIn('local/research/codex/wip-r2', actions)
        self.assertEqual(actions['local/research/codex/unused-r1'], 'disposable_unreferenced_run')
        self.assertEqual(report['keep_recent_days'], 7)
        self.assertEqual(report['totals']['keep_recent']['files'], 1)
        self.assertEqual([u['unit'] for u in report['keep_recent_units']], ['local/research/codex/wip-r2'])
        self.assertGreater(report['keep_recent_units'][0]['apparent_bytes'], 0)
        # Without the future clock the whole tree is fresh (ctime is now): everything unpinned is kept.
        _, _, fresh = self.dry_run(7)
        self.assertEqual(fresh['actions'], [])
        self.assertIn('local/research/codex/fixture-r1/frame.bin', fresh['pinned']['fixture_pins'])

    def test_keep_recent_days_zero_disables_guard(self):
        _, _, report = self.dry_run(0)
        self.assertNotIn('keep_recent', report['totals'])
        self.assertEqual(report['keep_recent_units'], [])
        self.assertIn('local/research/codex/unused-r1', {a['path'] for a in report['actions']})
        parser_default = retention.DEFAULT_KEEP_RECENT_DAYS
        self.assertEqual(parser_default, 7)

    def test_apply_rechecks_guard_and_skips_unit_touched_after_planning(self):
        later = time.time() + 30 * 86400
        path, digest, report = self.dry_run(7, later)
        self.assertIn('local/research/codex/unused-r1', {a['path'] for a in report['actions']})
        touched = self.root / 'local/research/codex/unused-r1/out.log'
        os.utime(touched, (later - 60, later - 60))  # became recent after planning
        result = retention.apply(self.root, path, digest, later)
        self.assertTrue(touched.exists())
        self.assertEqual(result['skipped_recent_count'], 1)
        self.assertEqual(result['skipped_recent'][0]['path'], 'local/research/codex/unused-r1')
        self.assertFalse((self.root / 'local/research/codex/run-a/game-A').exists())  # untouched units still go
        self.assertGreater(result['deleted'], 0)

    def test_apply_rejects_plan_without_guard(self):
        path, digest, report = self.dry_run(0)
        del report['keep_recent_days']
        path.write_text(json.dumps(report))
        with self.assertRaisesRegex(ValueError, 'keep_recent_days'):
            retention.apply(self.root, path, retention.sha256_file(path))

    def bulk(self, runs, files):
        """Referenced runs: one kept log plus many disposable intermediates, so every file is its own action."""
        for run in range(runs):
            for number in range(files):
                self.write(f'local/research/codex/bulk-{run}/sub{number % 5}/f{number}.bin', f'x{number}\n')
            self.write(f'local/research/codex/bulk-{run}/keep.txt', 'kept\n')
            for sub in range(5):  # small tooling in each subdirectory stops directory collapse
                self.write(f'local/research/codex/bulk-{run}/sub{sub}/tool.py', 'pass\n')
        for run in range(runs):
            row = self.fixture.row(f"B{run}", 'local', f'local/research/codex/bulk-{run}/keep.txt')
            record = self.fixture.record(f"CP9{run:02d}", 'origin', [row])
            self.fixture.put(record)
        self.fixture.commit("bulk records")

    def test_apply_scans_once_and_deletes_exactly_disposable(self):
        self.bulk(3, 60)
        path, digest, report = self.dry_run()
        self.assertGreater(len([a for a in report['actions'] if 'bulk-' in a['path']]), 150)
        calls = {'activity': 0, 'refs': 0}
        real_activity, real_refs = retention.unit_activity, retention.checkpoint_references

        def counting_activity(*args):
            calls['activity'] += 1
            return real_activity(*args)

        def counting_refs(*args):
            calls['refs'] += 1
            return real_refs(*args)
        retention.unit_activity, retention.checkpoint_references = counting_activity, counting_refs
        self.addCleanup(setattr, retention, 'unit_activity', real_activity)
        self.addCleanup(setattr, retention, 'checkpoint_references', real_refs)
        result = retention.apply(self.root, path, digest)
        units = {retention.unit_of(a['path']) for a in report['actions']}
        self.assertEqual(calls['refs'], 1)
        self.assertEqual(calls['activity'], len(units))  # once per unit, not once per action
        self.assertEqual(result['deleted'], len(report['actions']))
        for run in range(3):
            base = self.root / f'local/research/codex/bulk-{run}'
            self.assertEqual(sorted(p.name for p in base.rglob('*') if p.is_file()), ['keep.txt'] + ['tool.py'] * 5)
        self.assertTrue((self.root / 'local/research/codex/fixture-r1/frame.bin').exists())
        self.assertFalse((self.root / 'local/research/codex/unused-r1').exists())

    def test_apply_skips_unit_when_referenced_file_deletion_is_not_allowed(self):
        # An action whose target covers a pinned path must be skipped, not deleted (O(1) ancestor check).
        path, digest, report = self.dry_run()
        report['actions'].append({'path': 'local/research/codex/fixture-r1', 'class': 'disposable_copy',
                                  'action': 'delete', 'files': 2, 'apparent_bytes': 1})
        path.write_text(json.dumps(report))
        result = retention.apply(self.root, path, retention.sha256_file(path))
        self.assertTrue((self.root / 'local/research/codex/fixture-r1/frame.bin').exists())
        self.assertTrue(any(s['reason'] == 'not an allowed delete action' for s in result['skipped']))


def summary_exists(plan_path):
    return Path(str(plan_path)[:-5] + '.md').exists()


if __name__ == '__main__':
    unittest.main()
