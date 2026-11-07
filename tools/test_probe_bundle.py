"""Relocation, real TCP startup, and drift refusal; no native protocol oracle."""
import hashlib
import json
from pathlib import Path
import shutil
import socket
import subprocess
import sys
import tempfile
import time
import unittest

from probe_bundle import export_bundle


class BundleTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.source = self.root / 'source'
        self.source.mkdir()
        raw = b'bootstrap fixture for packaging only'
        self.fixture = self.source / 'fixture.bin'
        self.fixture.write_bytes(raw)
        names = ['bootstrap', 'rank_Awin', 'promote_B', 'auth_C', 'roomlist_C',
                 'join_C', 'player_C', 'slot_C', 'guest_C', 'ready_C']
        pin = dict(path=str(self.fixture), bytes=len(raw),
                   sha256=hashlib.sha256(raw).hexdigest(), sequence=None)
        self.profile = dict(schema='kr-probe-profile-v1', items=[], initial_key=12345,
                            fixtures={name: dict(pin) for name in names})

    def test_bundle_relocates_and_runs_real_tcp_from_another_cwd_in_isolated_python(self):
        original = self.root / 'original'
        manifest = export_bundle(self.profile, self.source, original)
        self.assertEqual(manifest['fixture_aliases'], 10)
        self.assertEqual(manifest['fixture_files'], 1)
        relocated = self.root / 'moved'
        shutil.move(original, relocated)
        # The source fixtures disappear: the relocated bundle must be sufficient.
        self.fixture.unlink()
        script = relocated / 'run.py'
        result = subprocess.run([sys.executable, '-I', str(script), '--verify-only'],
                                cwd=self.root, text=True, capture_output=True, check=True)
        self.assertEqual(json.loads(result.stdout)['fixtures'], 10)
        out = self.root / 'runtime'
        with (self.root / 'launch.log').open('w') as log:
            process = subprocess.Popen([sys.executable, '-I', str(script), '--out', str(out),
                                        '--bind-ip', '127.0.0.1', '--port', '0'],
                                       cwd=self.root, stdout=log, stderr=subprocess.STDOUT)
            try:
                deadline = time.monotonic() + 5
                while time.monotonic() < deadline:
                    state = out / 'service-state.json'
                    if state.exists():
                        snapshot = json.loads(state.read_text())
                        if snapshot.get('listen_addresses'):
                            break
                    if process.poll() is not None:
                        self.fail((self.root / 'launch.log').read_text())
                    time.sleep(.02)
                else:
                    self.fail('owned packaged service did not publish a listener')
                with socket.create_connection(tuple(snapshot['listen_addresses'][0]), timeout=2) as client:
                    self.assertEqual(client.recv(4096), b'bootstrap fixture for packaging only')
            finally:
                process.terminate()
                try:
                    process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait(timeout=5)
        self.assertEqual(process.returncode, 0)
        self.assertTrue(json.loads((out / 'service-state.json').read_text())['stopped'])

    def test_drift_or_missing_fixture_is_rejected_before_any_output_is_created(self):
        self.fixture.write_bytes(b'X' * self.profile['fixtures']['bootstrap']['bytes'])
        out = self.root / 'not-created'
        with self.assertRaisesRegex(ValueError, 'hash mismatch'):
            export_bundle(self.profile, self.source, out)
        self.assertFalse(out.exists())
        self.fixture.unlink()
        with self.assertRaises(FileNotFoundError):
            export_bundle(self.profile, self.source, out)
        self.assertFalse(out.exists())

    def test_existing_output_is_never_overwritten_and_input_is_not_rebased_in_place(self):
        out = self.root / 'bundle'
        before = json.dumps(self.profile, sort_keys=True)
        export_bundle(self.profile, self.source, out)
        manifest = (out / 'bundle.json').read_bytes()
        with self.assertRaises(FileExistsError):
            export_bundle(self.profile, self.source, out)
        self.assertEqual((out / 'bundle.json').read_bytes(), manifest)
        self.assertEqual(json.dumps(self.profile, sort_keys=True), before)

    def test_fixture_drift_at_destination_prevents_listen_or_runtime_output(self):
        out = self.root / 'bundle'
        export_bundle(self.profile, self.source, out)
        fixture = next((out / 'fixtures').iterdir())
        fixture.write_bytes(b'X' * fixture.stat().st_size)
        runtime = self.root / 'must-not-exist'
        result = subprocess.run([sys.executable, '-I', str(out / 'run.py'), '--out', str(runtime),
                                 '--port', '0'], cwd=self.root, text=True, capture_output=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('bundle file mismatch', result.stderr)
        self.assertFalse(runtime.exists())

    def test_source_symlink_escape_is_rejected_before_output(self):
        external = self.root / 'outside.bin'
        external.write_bytes(self.fixture.read_bytes())
        self.fixture.unlink()
        self.fixture.symlink_to(external)
        out = self.root / 'must-not-exist'
        with self.assertRaisesRegex(ValueError, 'escapes source root'):
            export_bundle(self.profile, self.source, out)
        self.assertFalse(out.exists())

    def test_launcher_refuses_profile_override_before_runtime_output(self):
        out = self.root / 'bundle'
        export_bundle(self.profile, self.source, out)
        runtime = self.root / 'must-not-exist'
        for option in [['--profile', 'outside.json'], ['--profile=outside.json']]:
            result = subprocess.run([sys.executable, '-I', str(out / 'run.py'),
                                     *option, '--out', str(runtime), '--port', '0'],
                                    cwd=self.root, text=True, capture_output=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn('bundle fixes --profile', result.stderr)
            self.assertFalse(runtime.exists())


if __name__ == '__main__':
    unittest.main()
