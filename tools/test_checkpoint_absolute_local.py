"""Compatibility for immutable host-local pins, without weakening raw hashes."""
import hashlib
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from check_checkpoints import verify_evidence_item, local_target


class AbsoluteLocalTests(unittest.TestCase):
    def setUp(self):
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        self.root = Path(tmp.name)
        self.path = self.root / 'local/run/raw.py'
        self.path.parent.mkdir(parents=True)
        self.raw = b'captured bytes\n'
        self.path.write_bytes(self.raw)
        self.pin = dict(id='E1', kind='repository', path=str(self.path),
                        bytes=len(self.raw), sha256=hashlib.sha256(self.raw).hexdigest())

    def verify(self, item=None, required=True):
        return verify_evidence_item(item or self.pin, self.root, required, 'CP187')

    def test_absolute_internal_raw_pin_keeps_record_and_reports_defect(self):
        before = dict(self.pin)
        errors, warnings, checked, missing = self.verify()
        self.assertEqual((errors, checked, missing), ([], 1, 0))
        self.assertEqual(self.pin, before)
        self.assertIn('declared kind=repository', warnings[0])
        self.assertIn('no Git fallback', warnings[0])

    def test_changed_raw_cannot_use_git_history_even_if_a_match_exists(self):
        self.path.write_text('changed\n')
        with patch('check_checkpoints.historical_checkpoint_source') as source, \
                patch('check_checkpoints.historical_repository_blob') as doc:
            self.assertTrue(self.verify()[0])
            source.assert_not_called()
            doc.assert_not_called()
        wrong_size = self.pin | {'bytes': 0}
        self.path.write_bytes(self.raw)
        self.assertTrue(self.verify(wrong_size)[0])

    def test_paths_outside_local_and_symlink_escape_remain_rejected(self):
        other = self.root / 'server/raw.py'
        other.parent.mkdir()
        other.write_bytes(self.raw)
        self.assertTrue(self.verify(self.pin | {'path': str(other)})[0])
        link = self.root / 'local/escape.py'
        link.symlink_to(other)
        self.assertTrue(self.verify(self.pin | {'path': str(link)})[0])
        self.assertTrue(self.verify(self.pin | {'path': str(self.root / 'local/../../elsewhere')})[0])
        with self.assertRaises(ValueError):
            local_target(self.root, str(self.path))

    def test_relative_paths_keep_strict_kind_validation(self):
        item = self.pin | {'path': 'local/run/raw.py'}
        self.assertTrue(self.verify(item)[0])
        errors, warnings, checked, missing = self.verify(item | {'kind': 'local'})
        self.assertEqual((errors, warnings, checked, missing), ([], [], 1, 0))

    def test_missing_host_file_is_unverified_and_required_fails(self):
        self.path.unlink()
        errors, warnings, checked, missing = self.verify()
        self.assertTrue(errors)
        self.assertEqual((checked, missing), (0, 1))
        self.assertTrue(any('LEGACY ABSOLUTE LOCAL PIN' in x for x in warnings))


if __name__ == '__main__':
    unittest.main()
