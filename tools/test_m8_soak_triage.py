import json
import tempfile
import unittest
from pathlib import Path

from m8_soak_triage import classify, load


class TriageTest(unittest.TestCase):
    def test_classifies_explicit_orderly_close(self):
        report = classify([{
            "event": "native_tls_owner_ended", "ok": False,
            "guard": "owner_lifecycle", "classification": "expected_peer_close",
            "opcode": None, "subtype": None,
        }])
        self.assertEqual(report["counts"], {"expected_peer_close": 1})
        self.assertEqual(report["unexpected"], 0)

    def test_legacy_terminal_delivery_tail_is_expected(self):
        events = [{"event": "native_tls_owner_ended", "ok": False,
                   "guard": "owner_lifecycle", "_line": 10},
                  {"event": "health_sample", "at_utc": 10, "_line": 9}]
        driver = [{"event": "delivery", "passed": False, "at_unix_ms": 9000, "_line": 9}]
        report = classify(events, driver)
        self.assertEqual(report["counts"], {"expected_peer_close": 1})

    def test_non_terminal_error_remains_gate(self):
        events = [{"event": "native_tls_owner_ended", "ok": False,
                   "guard": "account_storage", "_line": 2}]
        report = classify(events, [])
        self.assertEqual(report["counts"], {"unexpected": 1})

    def test_load_ignores_malformed_lines(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "events.jsonl"
            path.write_text('{"event":"x"}\nnot-json\n')
            self.assertEqual(len(load(path)), 1)


if __name__ == "__main__":
    unittest.main()
