"""Pinned preset relocation must reject drift instead of silently trusting it."""
import hashlib
import json
from pathlib import Path
import tempfile
import unittest

from export_probe_profile import load_preset


class PresetTests(unittest.TestCase):
    def test_relocation_preserves_pin_and_rejects_changed_fixture(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            raw = b'fixture before drift'
            fixture = root / 'captured.dat'
            fixture.write_bytes(raw)
            entry = dict(path='captured.dat', bytes=len(raw),
                         sha256=hashlib.sha256(raw).hexdigest(), sequence=73)
            preset = root / 'preset.json'
            preset.write_text(json.dumps(dict(schema='kre-probe-preset-v1',
                                              profile=dict(fixtures={'start': entry}))))
            exported = load_preset(root, preset)
            self.assertEqual(exported['fixtures']['start'], entry | {'path': str(fixture)})
            self.assertEqual(json.loads(preset.read_text())['profile']['fixtures']['start'], entry)
            fixture.write_bytes(b'X' * len(raw))
            with self.assertRaisesRegex(ValueError, 'size/hash mismatch'):
                load_preset(root, preset)

    def test_symlink_outside_source_root_is_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            folder = Path(tmp)
            root = folder / 'source'
            root.mkdir()
            external = folder / 'outside.dat'
            external.write_bytes(b'outside')
            (root / 'captured.dat').symlink_to(external)
            preset = root / 'preset.json'
            preset.write_text(json.dumps(dict(schema='kre-probe-preset-v1', profile=dict(
                fixtures={'start': dict(path='captured.dat', bytes=7,
                                       sha256=hashlib.sha256(b'outside').hexdigest(), sequence=73)}))))
            with self.assertRaisesRegex(ValueError, 'escapes source root'):
                load_preset(root, preset)


if __name__ == '__main__':
    unittest.main()
