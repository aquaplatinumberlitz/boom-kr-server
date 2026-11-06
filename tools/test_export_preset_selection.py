"""CLI selection and relocation checks; fixtures here are not a native oracle."""
import copy
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

import export_probe_profile as exporter


class PresetSelectionTests(unittest.TestCase):
    def test_replacement_preset_only_adds_explicit_vacancy_policy(self):
        basic = json.loads(exporter.PRESETS['two-client-basic'].read_text())['profile']
        replacement = json.loads(exporter.PRESETS['two-client-basic-replacement'].read_text())['profile']
        self.assertEqual(replacement.pop('replacement_slot'),
                         {'mode': 'vacant_AB_slot', 'capacity': 'vacated_slot'})
        self.assertEqual(replacement, basic)
        self.assertEqual(set(replacement['bomb_capacity']), {'A', 'B'})

    def test_cli_selects_exact_profile_and_rejects_drift_overwrite_and_flags(self):
        repo = Path(__file__).resolve().parents[1]
        with tempfile.TemporaryDirectory(prefix='test-presets-', dir=repo / 'local') as tmp:
            folder = Path(tmp)
            specs = {}
            expected = {}
            for name in exporter.PRESETS:
                root = folder / name
                root.mkdir()
                raw = name.encode()
                (root / 'fixture.bin').write_bytes(raw)
                profile = dict(schema='kr-probe-profile-v1', initial_key=12345,
                               item_layout=name, fixtures={'floor': dict(
                                   path='fixture.bin', bytes=len(raw),
                                   sha256=hashlib.sha256(raw).hexdigest(), sequence=73)})
                spec = root / 'spec.json'
                spec.write_text(json.dumps(dict(schema='kre-probe-preset-v1', profile=profile)))
                specs[name] = spec
                expected[name] = copy.deepcopy(profile)
                expected[name]['fixtures']['floor']['path'] = str(root / 'fixture.bin')
            with patch.dict(exporter.PRESETS, specs, clear=True):
                for name in specs:
                    root = specs[name].parent
                    out = root / 'export.json'
                    argv = ['export_probe_profile.py', '--source-root', str(root),
                            '--out', str(out), '--preset', name]
                    with self.subTest(name=name), patch.object(sys, 'argv', argv):
                        exporter.main()
                        self.assertEqual(json.loads(out.read_text()), expected[name])
                        before = out.read_bytes()
                        with self.assertRaises(FileExistsError):
                            exporter.main()
                        self.assertEqual(out.read_bytes(), before)
                    # Same-size drift must fail before creating output.
                    (root / 'fixture.bin').write_bytes(b'X' * len(name))
                    failed = root / 'drift.json'
                    argv[argv.index('--out') + 1] = str(failed)
                    with patch.object(sys, 'argv', argv):
                        with self.assertRaisesRegex(ValueError, 'size/hash mismatch'):
                            exporter.main()
                    self.assertFalse(failed.exists())
                    for extra in [['--item-layout', 'battle'], ['--spawn-layout', 'overlap'],
                                  ['--host-back-rejoin'], ['--host-battle-exit'],
                                  ['--all-dead-rank', str(root / 'fixture.bin')]]:
                        with patch.object(sys, 'argv', argv + extra):
                            with self.assertRaises(SystemExit) as error:
                                exporter.main()
                            self.assertEqual(error.exception.code, 2)
                        self.assertFalse(failed.exists())

    def test_shipped_cli_recognizes_both_presets_and_rejects_missing_fixtures(self):
        # Empty relocated roots exercise the shipped CLI and both committed specs.
        # Real pinned artifacts are validated separately by runtime preflight.
        repo = Path(__file__).resolve().parents[1]
        with tempfile.TemporaryDirectory(prefix='test-preset-cli-', dir=repo / 'local') as tmp:
            folder = Path(tmp)
            for name, spec_path in exporter.PRESETS.items():
                spec = json.loads(spec_path.read_text())
                root = folder / name
                root.mkdir()
                # Empty roots must never be silently converted into new trusted pins.
                out = root / 'profile.json'
                result = subprocess.run([
                    sys.executable, str(repo / 'tools/export_probe_profile.py'),
                    '--source-root', str(root), '--out', str(out), '--preset', name],
                    text=True, capture_output=True)
                self.assertNotEqual(result.returncode, 0)
                self.assertNotIn('invalid choice', result.stderr)
                self.assertFalse(out.exists())
                self.assertEqual(spec['schema'], 'kre-probe-preset-v1')


if __name__ == '__main__':
    unittest.main()
