"""Exercise the capture analyzer using synthetic events and checked-in evidence."""

from contextlib import redirect_stderr, redirect_stdout
import hashlib
import io
import json
from pathlib import Path
from tempfile import TemporaryDirectory
import unittest

from tools import kr_f2_f3_capture_analyzer as analyzer


def event(opcode, direction="C2S", role="host", **fields):
    return {"direction": direction, "opcode": opcode, "sender_role": role,
            "payload_hex": "00Aa", "ts_ms": 5, **fields}


def capture(*events):
    return analyzer.parse_capture(json.dumps(events).encode())


def encoded(value, width):
    return (value ^ analyzer.KR_UINT_MASK[width]).to_bytes(width, "big")


def fields(*items):
    return b"".join(encoded(value, width) for value, width in items)


class CaptureAnalyzerTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.evidence = analyzer.load_evidence()

    def report(self, a, b=None):
        return analyzer.analyze_captures(a, b, evidence=self.evidence)

    def test_json_array_and_jsonl_have_the_same_events(self):
        records = [event(80, map_name="Sample"), event(185, "S2C", "server", payload_hex="")]
        body = ("\n".join(json.dumps(row) + "\r\n" for row in records)).encode()
        normalized = analyzer.parse_capture(body)
        self.assertEqual(normalized.events, capture(*records).events)
        self.assertEqual(normalized.sha256, hashlib.sha256(body).hexdigest())
        self.assertEqual(analyzer.parse_capture(b"[]").events, ())

    def test_malformed_capture_is_rejected_with_safe_location(self):
        bad_fields = (
            ({"opcode": True}, "opcode"),
            ({"opcode": -1}, "opcode"),
            ({"direction": "S->C"}, "direction"),
            ({"sender_role": "team_a"}, "sender_role"),
            ({"ts_ms": 1.5}, "ts_ms"),
            ({"payload_hex": "001"}, "payload_hex"),
            ({"payload_hex": "0g"}, "payload_hex"),
            ({"map_name": 5}, "map_name"),
            ({"note": None}, "note"),
        )
        for fields, message in bad_fields:
            with self.subTest(fields=fields), self.assertRaisesRegex(ValueError, f"event 1: .*{message}"):
                capture(event(1, **fields) if "opcode" not in fields else {**event(1), **fields})
        for body, message in ((b"[null]", "event 1"), (b"[{}", "JSON array"),
                              (b"\n{bad", "line 2"), (b"\xff", "UTF-8")):
            with self.subTest(body=body), self.assertRaisesRegex(ValueError, message):
                analyzer.parse_capture(body)

    def test_exact_anchor_matches_require_the_correct_direction_and_wire_opcode(self):
        events = [event(op) for op in (185, 474, 633, 670)]
        events += [event(op, "S2C", "server") for op in (165, 185, 474, 633)]
        rows = {(r["direction"], r["opcode"]): r for r in self.report(capture(*events))["opcodes"]}
        ghost = rows["S2C", 185]["static_wire_anchor"]
        self.assertEqual((ghost["symbol"], ghost["handler_va"], ghost["hexrays_case"]),
                         ("RebirthToGhost", "0xe67470", 165))
        self.assertEqual(ghost["source"], "kr_static")
        for key in (("S2C", 165), ("C2S", 185), ("C2S", 474), ("S2C", 633)):
            self.assertIsNone(rows[key]["static_wire_anchor"])
        self.assertEqual(rows["S2C", 474]["static_wire_anchor"]["dispatcher_va"], "0xd68c90")
        self.assertEqual(rows["C2S", 633]["static_wire_anchor"]["feature"], "F5")
        self.assertEqual(rows["C2S", 670]["static_wire_anchor"]["caller_vas"], ["0x583530", "0x583e80"])

    def test_wave_m_valid_frames_get_bounded_candidate_summaries(self):
        item = fields((7, 4), (8, 2), (9, 4), (0, 2)) + b"\x00" * 8 + fields((10, 4), (11, 1))
        row_225 = fields((1, 1)) + b"".join(fields((100 + n, 4), (0, 2)) for n in range(8))
        records = [
            event(204, "S2C", "server", payload_hex=fields((7, 2), (3, 1)).hex()),
            event(132, payload_hex=fields((16, 4), (7, 2), (0, 2), (8, 4)).hex()),
            event(180, payload_hex=fields((7, 2), (4, 2), (11, 2), (12, 2)).hex()),
            event(219, "S2C", "server", payload_hex=fields((1, 1), (2, 1), (1, 4)).hex() + item.hex()),
            event(225, "S2C", "server", payload_hex=row_225.hex()),
            event(317, "S2C", "server", payload_hex=(
                fields((1, 1), (2, 1), (3, 4), (4, 4))
                + b"".join(encoded(n, 2) for n in range(8)) + encoded(9, 4)
            ).hex()),
            event(566, payload_hex=encoded(6, 1).hex()),
        ]
        rows = {(row["direction"], row["opcode"]): row
                for row in self.report(capture(*records))["opcodes"]}
        self.assertEqual(rows["S2C", 204]["frame_annotations"][0]["status"], "candidate_unmeasured")
        self.assertEqual(rows["S2C", 204]["frame_annotations"][0]["summary"],
                         {"object_id": 7, "owner_slot": 3})
        self.assertEqual(rows["C2S", 132]["frame_annotations"][0]["summary"]["kind"], 0)
        self.assertEqual(rows["C2S", 180]["frame_annotations"][0]["summary"]["u16_fields"], [11, 12])
        self.assertEqual(rows["S2C", 219]["frame_annotations"][0]["summary"]["groups"][0]["item_count"], 1)
        self.assertEqual(rows["S2C", 225]["frame_annotations"][0]["summary"]["row_count"], 1)
        self.assertEqual(rows["S2C", 317]["frame_annotations"][0]["summary"]["ok"], 1)
        self.assertEqual(rows["C2S", 566]["frame_annotations"][0]["summary"]["action_byte"], 6)
        self.assertEqual(rows["C2S", 566]["frame_candidate"]["static_provenance"]["issue"], "#809")

    def test_wave_m_truncated_frames_are_annotated_without_crashing(self):
        records = [
            event(204, "S2C", "server", payload_hex="00"),
            event(132, payload_hex=""),
            event(180, payload_hex="00"),
            event(219, "S2C", "server", payload_hex=encoded(1, 1).hex()),
            event(225, "S2C", "server", payload_hex=encoded(1, 1).hex()),
            event(317, "S2C", "server", payload_hex=encoded(1, 1).hex()),
            event(566, payload_hex=""),
        ]
        rows = {(row["direction"], row["opcode"]): row
                for row in self.report(capture(*records))["opcodes"]}
        for key in rows:
            with self.subTest(key=key):
                self.assertEqual(rows[key]["frame_annotations"][0]["decode_status"], "truncated")
                self.assertIn("error", rows[key]["frame_annotations"][0])

    def test_wave_m_out_of_range_fields_are_annotated(self):
        records = [
            event(204, "S2C", "server", payload_hex=fields((7, 2), (8, 1)).hex()),
            event(132, payload_hex=fields((1, 4), (7, 2), (0xFFFF, 2)).hex()),
            event(180, payload_hex=fields((7, 2), (6, 2), (13, 1), (0, 1)).hex()),
            event(219, "S2C", "server", payload_hex=encoded(65, 1).hex()),
            event(225, "S2C", "server", payload_hex=encoded(33, 1).hex()),
            event(317, "S2C", "server", payload_hex=fields((2, 1), (0, 1), (0, 4), (0, 4)).hex()),
        ]
        rows = {(row["direction"], row["opcode"]): row
                for row in self.report(capture(*records))["opcodes"]}
        for key in rows:
            with self.subTest(key=key):
                self.assertEqual(rows[key]["frame_annotations"][0]["decode_status"], "out_of_range")
                self.assertIn("error", rows[key]["frame_annotations"][0])

    def test_direct_classifier_rejects_bad_hex_as_data(self):
        annotation = analyzer.classify_frame("C2S", 566, "not-hex")
        self.assertEqual(annotation["decode_status"], "error")
        self.assertIn("payload_hex", annotation["error"])

    def test_jump_table_wire_annotation_uses_case_plus_twenty(self):
        body = fields((100, 4), (2, 1), (300, 4), (400, 4), (5, 1))
        annotation = analyzer.classify_frame("S2C", 197, body.hex())
        self.assertEqual(annotation["status"], "candidate_unmeasured")
        self.assertEqual(annotation["summary"]["case_index"], 177)
        self.assertEqual(annotation["summary"]["fields"], [100, 2, 300, 400, 5])
        self.assertEqual(annotation["decode_status"], "ok")

    def test_raw_and_shifted_case_matches_remain_static(self):
        cases = {9: "0xe6aa90", 80: "0xe6d730", 135: "0xe67640", 136: "0xe67770", 177: "0xe751f0"}
        opcodes = set(cases) | {case + 20 for case in cases} | {185}
        events = [event(op, "S2C", "server") for op in opcodes] + [event(135)]
        rows = self.report(capture(*events))["opcodes"]
        for row in rows:
            opcode = row["opcode"]
            with self.subTest(direction=row["direction"], opcode=opcode):
                self.assertEqual(row["authority_status"], "unknown")
                if row["direction"] == "C2S":
                    self.assertNotIn("f3_jump_table_check", row)
                    continue
                check = row["f3_jump_table_check"]
                self.assertEqual((check["dispatcher_va"], check["jump_table_va"]), ("0xe5d3a0", "0xe5de9c"))
                self.assertEqual((check["raw_opcode"], check["opcode_minus_20"]), (opcode, opcode - 20))
                self.assertEqual(check["wire_mapping_status"], "unknown")
                for key, case in (("raw_case_match", opcode), ("minus_20_case_match", opcode - 20)):
                    anchor = check[key]
                    if case in cases:
                        self.assertEqual((anchor["static_case_label"], anchor["handler_va"]), (case, cases[case]))
                        self.assertEqual(anchor["wire_mapping_status"], "unknown")
                    else:
                        self.assertIsNone(anchor)

    def test_reversed_roles_classify_each_direction_and_explain_the_observations(self):
        shared = [
            event(10, ts_ms=9), event(10, ts_ms=2), event(10, "S2C", "server"),
            event(11), event(11, role="guest"), event(12), event(16, role="guest"),
            event(17, "S2C", "server"), event(18, "S2C", "host"),
            event(19, map_name="MapA"), event(19, role="guest", map_name="MapB"),
        ]
        report = self.report(
            capture(*shared, event(13), event(14, map_name="MapA")),
            capture(*shared, event(13, role="guest"), event(14, map_name="MapB"), event(15),
                    event(17, "S2C", "host")),
        )
        self.assertEqual({(r["direction"], r["opcode"]): r["authority_candidate"] for r in report["opcodes"]}, {
            ("C2S", 10): "follows_host", ("S2C", 10): "server_originated",
            ("C2S", 11): "both_clients", ("C2S", 12): "follows_host",
            ("C2S", 13): "inconclusive", ("C2S", 14): "inconclusive", ("C2S", 15): "inconclusive",
            ("C2S", 16): "inconclusive", ("S2C", 17): "inconclusive", ("S2C", 18): "inconclusive",
            ("C2S", 19): "inconclusive",
        })
        self.assertEqual(report["captures"]["host_a"]["sender_role_layout"]["host"], "client_a")
        self.assertEqual(report["captures"]["host_b"]["sender_role_layout"]["host"], "client_b")
        self.assertEqual(report["opcodes"][0]["observations"]["host_a"], [
            {"map_name": None, "sender_role_counts": {"host": 2}, "first_ts_ms": 2, "last_ts_ms": 9},
        ])
        self.assertTrue(all(row["authority_status"] == "unknown" for row in report["opcodes"]))
        self.assertEqual(set(report["authority_questions"].values()), {"unknown"})
        self.assertEqual(report["authority_questions"]["monster_hp"], "unknown")

    def test_second_capture_is_optional_and_empty_is_not_evidence(self):
        host_a = capture(event(185, "S2C", "server"))
        single = self.report(host_a)
        self.assertNotIn("host_b", single["captures"])
        self.assertEqual(single["opcodes"][0]["authority_candidate"], "inconclusive")
        paired = self.report(host_a, capture())
        self.assertEqual(paired["captures"]["host_b"]["event_count"], 0)
        self.assertEqual(paired["opcodes"][0]["observations"]["host_b"], [])
        self.assertEqual(paired["opcodes"][0]["authority_candidate"], "inconclusive")

    def test_output_rejects_private_data_in_decoded_keys_and_values(self):
        markers = (
            "/" + "home/" + "admin/capture",
            "gateway " + ".".join(("10", "0", "0", "1")) + ".",
            ".".join(("172", "31", "1", "2")),
            ".".join(("192", "168", "1", "2")),
            ".".join(("198", "18", "1", "2")),
            "fd00" + "::1",
            "-----BEGIN RSA " + "PRIVATE KEY-----",
            "-----BEGIN " + "PRIVATE KEY-----",
            "OCID" + "1.synthetic",
        )
        for n, marker in enumerate(markers):
            with self.subTest(marker_index=n):
                with self.assertRaisesRegex(ValueError, "^report contains private data$"):
                    analyzer.render_report({marker: "value"})
                report = self.report(capture(event(1, map_name=marker)))
                with self.assertRaisesRegex(ValueError, "^report contains private data$"):
                    analyzer.render_report(report)

    def test_cli_writes_reproducible_report_and_drops_payload_and_note(self):
        with TemporaryDirectory() as temp, redirect_stdout(io.StringIO()) as stdout:
            host_a, host_b, out = (Path(temp) / name for name in ("a.json", "b.jsonl", "out.json"))
            row = event(633, note="capture note stays local", payload_hex="cafebabe", map_name="눈밭")
            host_a.write_text(json.dumps([row]), encoding="utf-8")
            host_b.write_text(json.dumps(row) + "\n", encoding="utf-8")
            args = ["--host-a", str(host_a), "--host-b", str(host_b)]
            analyzer.main(args)
            analyzer.main([*args, "--out", str(out)])
            self.assertEqual(out.read_bytes(), stdout.getvalue().encode())
            self.assertEqual(json.loads(stdout.getvalue())["opcodes"][0]["authority_candidate"], "follows_host")
            for private in (str(host_a), str(host_b), row["payload_hex"], row["note"]):
                self.assertNotIn(private, stdout.getvalue())

    def test_cli_refuses_to_write_or_echo_private_output(self):
        with TemporaryDirectory() as temp, redirect_stdout(io.StringIO()) as stdout, redirect_stderr(io.StringIO()) as stderr:
            host_a, out = (Path(temp) / name for name in ("a.json", "out.json"))
            marker = "/" + "home/" + "admin/capture"
            # The slash is escaped in the input JSON and decoded before screening.
            host_a.write_text(json.dumps([event(1, map_name=marker)]).replace("/", r"\/"), encoding="utf-8")
            with self.assertRaises(SystemExit) as code:
                analyzer.main(["--host-a", str(host_a), "--out", str(out)])
            self.assertEqual(code.exception.code, 1)
            self.assertIn("report contains private data", stderr.getvalue())
            self.assertNotIn(marker, stderr.getvalue())
            self.assertEqual(stdout.getvalue(), "")
            self.assertFalse(out.exists())

    def test_self_check(self):
        with redirect_stdout(io.StringIO()) as stdout:
            analyzer.main(["--self-check"])
        self.assertEqual(stdout.getvalue(), "F2/F3 capture analyzer self-check OK\n")


if __name__ == "__main__":
    unittest.main()
