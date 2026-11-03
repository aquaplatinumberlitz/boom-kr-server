"""Export option tests only; native layout/UI evidence lives outside these tests."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

from export_probe_profile import FIXTURES


class AllDeadExportTests(unittest.TestCase):
    def test_option_adds_only_the_declared_optional_alias_and_pins_its_raw_bytes(self):
        # Exporter deliberately restricts all paths to this repository's local/.
        root = Path(__file__).resolve().parents[1]
        with tempfile.TemporaryDirectory(prefix='test-rank-export-', dir=root / 'local') as tmp:
            folder = Path(tmp)
            for relative, _ in FIXTURES.values():
                path = folder / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(b'existing-fixture')
            frame = bytearray(38)
            frame[5:9] = (175).to_bytes(4, 'little')
            candidate = folder / 'all-dead.bin'
            candidate.write_bytes(frame)
            command = [sys.executable, str(root / 'tools/export_probe_profile.py'),
                       '--source-root', str(folder)]
            normal, optional = folder / 'normal.json', folder / 'optional.json'
            subprocess.run(command + ['--out', str(normal)], check=True, capture_output=True)
            subprocess.run(command + ['--out', str(optional), '--all-dead-rank', str(candidate)],
                           check=True, capture_output=True)
            before, after = json.loads(normal.read_text()), json.loads(optional.read_text())
            self.assertNotIn('rank_all_dead', before['fixtures'])
            self.assertEqual(after['fixtures'].pop('rank_all_dead'), {
                'path': str(candidate), 'bytes': 38,
                'sha256': hashlib.sha256(frame).hexdigest(), 'sequence': 91})
            self.assertEqual(before, after)
            self.assertEqual(candidate.read_bytes(), frame)
            candidate.write_bytes(bytes(38))
            invalid = folder / 'invalid.json'
            result = subprocess.run(command + ['--out', str(invalid),
                                               '--all-dead-rank', str(candidate)],
                                    capture_output=True, text=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn('KR175 frame', result.stderr)
            self.assertFalse(invalid.exists())


if __name__ == '__main__':
    unittest.main()
