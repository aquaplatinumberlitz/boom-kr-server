"""Deletion safety and evidence retention, without a VM or private deployment."""
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import native_run_cleanup as cleanup


class RunCleanupTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.run = self.root / 'local/research/codex/m8-test-run'
        self.run.mkdir(parents=True)
        self.disk = self.run / 'native-system.qcow2'
        self.disk.write_bytes(b'owned disposable overlay')
        self.seed = self.run / 'seed.img'
        self.seed.write_bytes(b'owned disposable seed')
        (self.run / 'guest.json').write_text('{"phase":"failed"}')
        (self.run / 'capture.pcap').write_bytes(b'evidence')
        (self.run / 'raw.dmp').write_bytes(b'not extracted')
        self.guards = patch.object(cleanup, 'reference_guards', return_value=(set(), set()))
        self.guards.start()
        self.addCleanup(self.guards.stop)

    def clean(self, **kwargs):
        return cleanup.cleanup(self.root, self.run, stopped=True, exports_complete=True, **kwargs)

    def test_removes_owned_disks_only_after_durable_manifest(self):
        original = Path.unlink
        observed = []

        def unlink(path, *args, **kwargs):
            manifest = self.run / 'run-cleanup-manifest.json'
            self.assertTrue(manifest.is_file())
            record = json.loads(manifest.read_text())
            self.assertIn(path.name, [r['path'] for r in record['files']])
            observed.append(path.name)
            return original(path, *args, **kwargs)

        with patch.object(Path, 'unlink', unlink):
            receipt = self.clean()
        self.assertEqual(set(observed), {'seed.img', 'native-system.qcow2'})
        self.assertEqual(receipt['deleted_apparent_bytes'], len(b'owned disposable overlayowned disposable seed'))
        for name in ('guest.json', 'capture.pcap', 'raw.dmp'):
            self.assertTrue((self.run / name).is_file())
        self.assertEqual(receipt['dumps_pending_extraction'], ['raw.dmp'])

    def test_uncertain_stop_or_export_preserves_disks(self):
        for stopped, exported in ((False, True), (True, False)):
            with self.subTest(stopped=stopped, exported=exported):
                other = self.run.with_name('m8-uncertain-' + str(stopped))
                other.mkdir()
                (other / 'seed.img').write_bytes(b'keep for recovery')
                receipt = cleanup.cleanup(self.root, other, stopped=stopped, exports_complete=exported)
                self.assertEqual(receipt['deleted'], [])
                self.assertTrue((other / 'seed.img').exists())

    def test_reference_and_pinned_directory_preserve_disks(self):
        relative = 'local/research/codex/m8-test-run'
        with patch.object(cleanup, 'reference_guards', return_value=({relative + '/seed.img'}, {relative})):
            receipt = self.clean()
        self.assertEqual(receipt['deleted'], [])
        self.assertTrue(self.disk.exists())
        self.assertTrue(self.seed.exists())

    def test_reusable_input_is_not_deleted(self):
        receipt = self.clean(inputs=[self.disk])
        self.assertEqual(receipt['deleted'], ['seed.img'])
        self.assertTrue(self.disk.exists())

    def test_fixture_pin_preserves_scratch_named_as_input(self):
        (self.run / 'fixture-config.json').write_text(json.dumps({
            'schema': 'kr-rust-cleanup-control-v1',
            'fixture': {'path': 'local/research/codex/m8-test-run/seed.img', 'sha256': '0' * 64}}))
        receipt = self.clean()
        self.assertEqual(receipt['deleted'], ['native-system.qcow2'])
        self.assertTrue(self.seed.exists())

    def test_symlink_and_hardlink_preserved(self):
        self.disk.unlink()
        target = self.root / 'source.qcow2'
        target.write_bytes(b'original')
        self.disk.symlink_to(target)
        os.link(self.seed, self.root / 'source.img')
        receipt = self.clean()
        self.assertEqual(receipt['deleted'], [])
        self.assertEqual(target.read_bytes(), b'original')

    def test_incomplete_reference_inventory_preserves_everything(self):
        with patch.object(cleanup, 'reference_guards', side_effect=ValueError('missing manifest')):
            receipt = self.clean()
        self.assertEqual(receipt['deleted'], [])
        self.assertIn('missing manifest', receipt['guard_error'])
        self.assertTrue(self.seed.exists())

    def test_manifest_write_failure_prevents_deletion(self):
        with patch.object(cleanup, 'write_json', side_effect=OSError('disk full')):
            with self.assertRaises(OSError):
                self.clean()
        self.assertTrue(self.seed.exists())
        self.assertTrue(self.disk.exists())

    def test_changed_file_is_preserved_after_manifest(self):
        original = cleanup.write_json

        def changed(path, value):
            original(path, value)
            if path.name == 'run-cleanup-manifest.json':
                self.seed.write_bytes(b'new content after hashing')

        with patch.object(cleanup, 'write_json', changed):
            receipt = self.clean()
        self.assertEqual(receipt['deleted'], ['native-system.qcow2'])
        self.assertTrue(self.seed.exists())

    def test_existing_receipts_and_non_m8_boundary_rejected(self):
        self.clean()
        with self.assertRaises(ValueError):
            self.clean()
        outside = self.root / 'local/research/codex/g1-other'
        outside.mkdir()
        with self.assertRaises(ValueError):
            cleanup.cleanup(self.root, outside, stopped=True, exports_complete=True)

    def test_storage_thresholds(self):
        for available in (79_999_999_999, 149_999_999_999):
            with patch.object(cleanup, 'storage', return_value={'available_bytes': available}):
                with self.assertRaises(RuntimeError):
                    cleanup.require_storage()
        with patch.object(cleanup, 'storage', return_value={'available_bytes': 150_000_000_000}):
            cleanup.require_storage()


if __name__ == '__main__':
    unittest.main()
