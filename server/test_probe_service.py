import hashlib
import json
from pathlib import Path
import tempfile
import unittest

from server import probe_codec as codec
from server.probe_service import FixtureProfile


class FixtureProfileTests(unittest.TestCase):
    def write_profile(self, root, raw, sequence, *, digest=None):
        fixture = root / 'fixture.bin'
        fixture.write_bytes(raw)
        profile = root / 'profile.json'
        profile.write_text(json.dumps({'schema': 'kr-probe-profile-v1', 'fixtures': {
            'test': {'path': str(fixture), 'bytes': len(raw),
                     'sha256': digest or hashlib.sha256(raw).hexdigest(), 'sequence': sequence}}}))
        return profile

    def test_corrupt_fixture_is_rejected_without_rewriting_it(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            original = codec.encode(bytes(37), 159, b'fixture', 85)
            corrupt = original[:-1] + bytes([original[-1] ^ 1])
            path = self.write_profile(root, corrupt, 85, digest=hashlib.sha256(original).hexdigest())
            with self.assertRaisesRegex(ValueError, 'hash mismatch'):
                FixtureProfile(path)
            self.assertEqual((root / 'fixture.bin').read_bytes(), corrupt)

    def test_wrong_base_sequence_fails_before_listening(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            raw = codec.encode(bytes(37), 159, b'fixture', 85)
            path = self.write_profile(root, raw, 91)
            with self.assertRaisesRegex(ValueError, 'invalid fixture test'):
                FixtureProfile(path)
