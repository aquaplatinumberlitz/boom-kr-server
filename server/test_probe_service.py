import hashlib
import json
from pathlib import Path
import tempfile
import unittest

from server import probe_codec as codec
from server.probe_service import FixtureProfile, ProbeService
from server.probe_room import ProbeRoom
from server.item_claims import ItemSpec


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


class PickupRoundKeyTests(unittest.TestCase):
    def service(self):
        # Exercise the actual dispatch/claim path. The fixture profile and TCP
        # transport are irrelevant to this policy regression; record broadcasts.
        service = ProbeService.__new__(ProbeService)
        service.hidden_items = {}
        service.room = ProbeRoom([ItemSpec(17, 4, 3, 2)])
        service.room.begin()
        service.room.finish()
        service.room.begin()
        service.state = {'item_requests': [], 'policy_events': []}
        self.outputs = []
        def broadcast(name, label, **kwargs):
            self.outputs.append(kwargs['body'])
            return []
        service.broadcast = broadcast
        return service

    def pickup(self, service, key, *, bound_role='A', slot=0, index=17):
        body = codec.pack_fields([0, slot, 4, 3, index, 2, 1234, key, 0],
                                 [1, 1, 1, 1, 4, 2, 4, 4, 1])
        # Opcode is read from the frame header by dispatch.
        frame = codec.encode(bytes(37), 124, body, 100)
        service.receive_channel({'role': bound_role}, frame, 100, body)
        return service.state['item_requests'][-1]

    def test_old_round_key_cannot_take_new_claim_before_current_request(self):
        service = self.service()
        old = self.pickup(service, 12345)
        self.assertEqual(old['rejected'], 'round_key_mismatch')
        self.assertEqual(self.outputs, [])
        current = self.pickup(service, 12346)
        self.assertNotIn('rejected', current)
        self.assertEqual(len(self.outputs), 1)
        self.assertEqual(codec.read_fields(self.outputs[0], [1, 1, 1, 1, 1, 4, 2, 1]),
                         [1, 0, 0, 4, 3, 17, 2, 0])
        self.assertEqual(self.pickup(service, 12346)['rejected'],
                         'claim_not_current_or_already_consumed')
        self.assertEqual(len(self.outputs), 1)

    def test_current_key_does_not_replace_origin_or_catalog_checks(self):
        service = self.service()
        self.assertEqual(self.pickup(service, 12346, bound_role='B')['rejected'],
                         'claim_not_current_or_already_consumed')
        self.assertEqual(self.pickup(service, 12346, index=999)['rejected'],
                         'claim_not_current_or_already_consumed')
        self.assertNotIn('rejected', self.pickup(service, 12346))
        self.assertEqual(len(self.outputs), 1)

    def test_full_u32_guard_rejects_same_low_byte_without_taking_claim(self):
        service = self.service()
        # Movement uses only low8; the chosen C2S policy compares the full
        # measured u32 k. This regression does not claim native authentication.
        mismatch = self.pickup(service, 12346 + 256)
        self.assertEqual(mismatch['rejected'], 'round_key_mismatch')
        self.assertEqual(self.outputs, [])
        self.assertNotIn('rejected', self.pickup(service, 12346))
        self.assertEqual(len(self.outputs), 1)
