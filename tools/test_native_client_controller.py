import json
from pathlib import Path
import sqlite3
from tempfile import TemporaryDirectory
import time
import unittest
from unittest.mock import patch

from native_client_controller import Server, prepare, run


class ControllerTest(unittest.TestCase):
    def setUp(self):
        self.temp = TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.events = self.root / "rust-events.jsonl"
        self.db = self.root / "account.sqlite"
        (self.root / "service-state.json").write_text('{"ready":true,"stopped":false}')
        self.health(at=int(time.time()))
        self.server = Server(self.root, self.db)

    def health(self, at, boot="123"):
        (self.root / "health.json").write_text(json.dumps({
            "schema": "kr-health-v1", "healthy": True, "at_utc": at, "boot": boot, "epoch": 1,
        }))

    def test_two_distinct_peer_markers_and_exact_journal_key(self):
        marker = {"event": "native_tls_room_peer_written", "opcode": 153, "stream_id": 2}
        self.events.write_text(json.dumps(marker) + "\n")
        with sqlite3.connect(self.db) as db:
            db.execute("CREATE TABLE matches(room_id,room_session,round,state,reason)")
            db.execute("INSERT INTO matches VALUES(1,2,256,'complete','elimination')")
        self.server.events.poll()
        start = {"begin": 1}
        gate = {"events": [{"after": "begin", "match": {"event": marker["event"]},
                            "count": 2, "distinct": "stream_id"}],
                "journal": {"room_id": 1, "room_session": 1, "round": 256, "state": "complete"}}
        with self.events.open("a") as f:
            f.write((json.dumps(marker) + "\n") * 2)
            f.write(json.dumps(dict(marker, stream_id=3)))  # writer not finished
        self.assertFalse(self.server.matches(gate, start))
        with self.events.open("a") as f:
            f.write("\n")
        self.assertFalse(self.server.matches(gate, start))  # stale room generation
        with sqlite3.connect(self.db) as db:
            db.execute("INSERT INTO matches VALUES(1,1,256,'complete','elimination')")
        self.assertTrue(self.server.matches(gate, start))

    def test_stale_health_times_out_without_input_and_restart_is_rejected(self):
        self.health(at=100)
        steps = [("wait", 1, {}, [["helper", "123", "key", "32", "100"]])]
        with patch("native_client_controller.send") as send, patch(
                "native_client_controller.time.monotonic", side_effect=[0, 2]):
            with self.assertRaisesRegex(TimeoutError, "no input"):
                run(self.server, steps, lambda row: None)
            send.assert_not_called()
        self.health(at=int(time.time()))
        self.assertTrue(self.server.ready())
        self.health(at=int(time.time()), boot="124")
        with self.assertRaisesRegex(ValueError, "restarted"):
            self.server.ready()

    def test_unicode_and_holds_are_validated_before_input(self):
        payload = self.root / "text"
        payload.write_text("Việt 한😀", encoding="utf-8")
        plan = {"steps": [{"id": "ready", "gate": {}, "actions": [
            {"op": "click", "xy": [30, 41], "hold_ms": 220},
            {"op": "unicode", "text_file": "text"}]}]}
        prefix = ["wine", "/example/input.exe", "55"]
        commands = prepare(plan, self.root, prefix)[0][3]
        self.assertEqual(commands[0], prefix + ["click", "30", "41", "220", "100"])
        self.assertEqual(bytes.fromhex(commands[1][-2]).decode("utf-16le"), "Việt 한😀")
        plan["steps"][0]["actions"][0]["hold_ms"] = 2001
        with self.assertRaises(ValueError):
            prepare(plan, self.root, prefix)


if __name__ == "__main__":
    unittest.main()
