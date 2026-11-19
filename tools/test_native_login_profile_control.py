"""Pin/boundary tests for the explicit loopback profile diagnostic."""
import hashlib
import json
from pathlib import Path
import struct
import tempfile
import unittest

import native_login_profile_control as control


class ProfileControlTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.local = self.root / 'local'
        self.local.mkdir()
        self.fixture = self.local / 'reply.bin'
        self.raw = bytearray(39)
        struct.pack_into('<I', self.raw, 5, 4)
        struct.pack_into('<I', self.raw, 13, 1)
        self.fixture.write_bytes(self.raw)
        self.declaration = self.local / 'control.json'
        self.row = dict(path='local/reply.bin', bytes=len(self.raw),
                        sha256=hashlib.sha256(self.raw).hexdigest(), sequence=43)
        self.save()

    def save(self):
        self.declaration.write_text(json.dumps(dict(fixture=self.row)))

    def test_exact_fixture_is_pinned_and_default_source_stays_unchanged(self):
        raw, pin = control.load_reply(self.declaration, self.root)
        self.assertEqual(raw, self.raw)
        self.assertEqual(pin['sha256'], self.row['sha256'])
        source = Path(__file__).with_name('native_login_capture.cs').read_text()
        generated = control.capture_source(source, raw)
        self.assertEqual(Path(__file__).with_name('native_login_capture.cs').read_text(), source)
        with self.assertRaises(ValueError):
            control.capture_source(generated, raw)

    def test_later_fixture_replacement_does_not_inherit_a_valid_pin(self):
        changed = self.raw.copy()
        changed[-1] ^= 1
        self.fixture.write_bytes(changed)
        with self.assertRaises(ValueError):
            control.load_reply(self.declaration, self.root)

    def test_external_declaration_traversal_and_symlink_escape_are_rejected(self):
        outside = self.root / 'outside.json'
        outside.write_bytes(self.declaration.read_bytes())
        with self.assertRaises(ValueError):
            control.load_reply(outside, self.root)
        self.row['path'] = 'local/../outside.json'
        self.save()
        with self.assertRaises(ValueError):
            control.load_reply(self.declaration, self.root)
        self.row['path'] = 'local/escape.bin'
        (self.local / 'escape.bin').symlink_to(outside)
        self.save()
        with self.assertRaises(ValueError):
            control.load_reply(self.declaration, self.root)

    def test_wrong_sequence_or_frame_opcode_length_and_oversize_fail_closed(self):
        self.row['sequence'] = 40
        self.save()
        with self.assertRaises(ValueError):
            control.load_reply(self.declaration, self.root)

        self.row['sequence'] = 43
        for offset, value in [(5, 15), (13, 2)]:
            changed = self.raw.copy()
            struct.pack_into('<I', changed, offset, value)
            self.fixture.write_bytes(changed)
            self.row['sha256'] = hashlib.sha256(changed).hexdigest()
            self.save()
            with self.assertRaises(ValueError):
                control.load_reply(self.declaration, self.root)
        self.fixture.write_bytes(b'\0' * 65537)
        self.row.update(bytes=65537, sha256=hashlib.sha256(self.fixture.read_bytes()).hexdigest())
        self.save()
        with self.assertRaises(ValueError):
            control.load_reply(self.declaration, self.root)

    def test_second_slot_notification_requires_its_own_sequence_and_redirect_advance(self):
        continuation = {}
        for name, opcode, sequence in [('netinfo', 15, 46), ('flags', 62, 49),
                                       ('flags2', 62, 52), ('redirect', 7, 55)]:
            raw = self.raw.copy()
            struct.pack_into('<I', raw, 5, opcode)
            path = self.local / (name + '.bin')
            path.write_bytes(raw)
            continuation[name] = dict(path='local/' + path.name, bytes=len(raw),
                sha256=hashlib.sha256(raw).hexdigest(), sequence=sequence)
        data = dict(fixture=self.row, continuations=continuation)
        self.declaration.write_text(json.dumps(data))
        frames, pins = control.load_continuations(self.declaration, self.root)
        self.assertEqual(set(frames), {'netinfo', 'flags', 'flags2', 'redirect'})
        self.assertEqual(pins['redirect']['sequence'], 55)
        continuation['redirect']['sequence'] = 52
        self.declaration.write_text(json.dumps(data))
        with self.assertRaises(ValueError):
            control.load_continuations(self.declaration, self.root)



if __name__ == '__main__':
    unittest.main()
