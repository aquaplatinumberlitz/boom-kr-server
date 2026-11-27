"""Kiểm phạm vi, phân mức bằng chứng và tái lập F5 khi không có dữ liệu ngoài repo."""

import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

if __package__:
    from . import kr_f5_extended_social_audit as audit
else:
    import kr_f5_extended_social_audit as audit


class EvidenceTests(unittest.TestCase):
    def setUp(self):
        self.data = audit.read_inputs(audit.ROOT)

    def test_exact_f5_scope_and_unknown_wire(self):
        result = audit.build_matrix(self.data)
        self.assertEqual(result["schema"], "kre.f5-extended-social.v1")
        families = result["map_families"]
        self.assertEqual({row["name"]: row["map_count"] for row in families}, {
            "LobbySquare": 9, "LobbyCaffe": 5, "LobbyClothing": 2,
            "LobbyJewelry": 2, "LobbyPet": 2, "Square": 2,
        })
        self.assertEqual(result["summary"], {
            "map_families": 6, "maps": 22, "scripts": 22, "scripts_unknown": 22,
            "extended_systems": 4,
        })
        self.assertTrue(all((row["wave"], row["classification"], row["status"]) ==
                            ("F5?", "C", "candidate") for row in families))

        systems = {row["id"]: row for row in result["systems"]}
        self.assertEqual(set(systems), {"Mail", "ExchangeChat", "Guild", "Couple"})
        for row in (*families, result["plaza"], *systems.values()):
            with self.subTest(name=row["name"]):
                self.assertEqual(row["kr_wire"]["opcodes"], "unknown")
                self.assertEqual(row["kr_wire"]["status"], "unknown")
        for row in (result["plaza"], *systems.values()):
            self.assertEqual(row["schema_9"]["status"], "design")

        inbox_cap = systems["Mail"]["rules"]["inbox_cap"]
        self.assertEqual((inbox_cap["status"], inbox_cap["value"]), ("unknown", "unknown"))
        approval = systems["Guild"]["rules"]["private_approval_max"]
        self.assertEqual((approval["status"], approval["value"], approval["unit"]),
                         ("candidate", 7, "days"))
        rules = systems["Couple"]["rules"]
        self.assertEqual(set(rules), {"rings", "exp_bonus", "lucci_bonus"})
        self.assertTrue(all(rule["value"] == "unknown" for rule in rules.values()))

    def test_static_anchor_directions_are_separate_from_wire(self):
        result = audit.build_matrix(self.data)
        systems = {row["id"]: row for row in result["systems"]}
        for key, expected in {
            "Guild": {"S->C": {467, 474, 478}, "C->S": {633}},
            "Couple": {"S->C": {257, 273, 284, 640, 674}, "C->S": {670}},
        }.items():
            with self.subTest(system=key):
                static = systems[key]["static_ida"]
                self.assertEqual((static["source"], static["status"]), ("kr_static", "confirmed"))
                for direction, opcodes in expected.items():
                    self.assertEqual({a["opcode"] for a in static["anchors"]
                                      if a["direction"] == direction}, opcodes)
                for row in static["anchors"]:
                    self.assertTrue(row["vas"])

        ida = json.loads(self.data["ida"])
        ida["f5_social"]["c2s_anchors"].pop()  # Couple 670 must not disappear silently.
        self.data["ida"] = json.dumps(ida).encode()
        with self.assertRaisesRegex(audit.AuditError, "Neo IDA couple"):
            audit.build_matrix(self.data)

    def test_output_digest_and_input_fingerprints(self):
        matrix = json.loads((audit.ROOT / audit.MATRIX).read_bytes())
        fingerprint = matrix.pop("matrix_sha256")
        canonical = json.dumps(matrix, ensure_ascii=False, sort_keys=True, separators=(",", ":")) + "\n"
        self.assertEqual(fingerprint, hashlib.sha256(canonical.encode()).hexdigest())
        self.assertEqual({key: value["sha256"] for key, value in matrix["inputs"].items()}, {
            key: hashlib.sha256(raw).hexdigest() for key, raw in self.data.items()
        })
        self.assertIn(fingerprint, (audit.ROOT / audit.MARKDOWN).read_text(encoding="utf-8"))


class OfflineCLITests(unittest.TestCase):
    def setUp(self):
        temp = self.enterContext(tempfile.TemporaryDirectory())
        self.root = Path(temp) / "fixture"
        self.script = self.root / "tools" / "kr_f5_extended_social_audit.py"
        for relative in (*audit.INPUTS.values(), self.script.relative_to(self.root)):
            target = self.root / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(audit.ROOT / relative, target)

    def run_cli(self, mode, *, seed="1"):
        # No .git, local/, Wine, external tools or other project files. Deny socket
        # use and child processes, and invoke the script from outside the fixture.
        runner = (
            "import runpy,sys\n"
            "def guard(event, args):\n"
            "    if event.startswith(('socket.', 'subprocess.', 'os.exec', 'os.spawn', 'os.posix_spawn')) "
            "or event == 'os.system':\n"
            "        raise RuntimeError('CLI must run offline')\n"
            "sys.addaudithook(guard)\n"
            "sys.argv = sys.argv[1:]\n"
            "runpy.run_path(sys.argv[0], run_name='__main__')\n"
        )
        return subprocess.run(
            [sys.executable, "-c", runner, str(self.script), mode],
            cwd=self.root.parent, env={"PYTHONHASHSEED": seed, "PYTHONIOENCODING": "utf-8"},
            capture_output=True, text=True, encoding="utf-8", check=False, timeout=10,
        )

    def test_cli_regenerates_exact_bytes_and_rejects_drift(self):
        for seed in ("1", "2"):
            result = self.run_cli("--write", seed=seed)
            self.assertEqual(result.returncode, 0, result.stderr)
            for relative in (audit.MATRIX, audit.MARKDOWN):
                self.assertEqual((self.root / relative).read_bytes(), (audit.ROOT / relative).read_bytes())
        result = self.run_cli("--check")
        self.assertEqual(result.returncode, 0, result.stderr)
        for relative in (audit.MATRIX, audit.MARKDOWN):
            with self.subTest(relative=relative):
                target = self.root / relative
                original = target.read_bytes()
                target.write_bytes(original + b"\n")
                result = self.run_cli("--check")
                self.assertEqual(result.returncode, 1)
                self.assertIn(relative, result.stderr)
                self.assertEqual(target.read_bytes(), original + b"\n")
                target.write_bytes(original)

    def test_cli_leak_scan_on_checked_outputs_and_before_write(self):
        result = self.run_cli("--write")
        self.assertEqual(result.returncode, 0, result.stderr)
        samples = (
            (audit.MARKDOWN, "/".join(("", "home", "admin", "fixture"))),
            (audit.MATRIX, ".".join(("10", "42", "17", "9")) + "."),
            (audit.MARKDOWN, "[fd" + "12::5]"),
            (audit.MATRIX, "password" + " = fixture_value"),
        )
        for relative, marker in samples:
            with self.subTest(relative=relative):
                target = self.root / relative
                original = target.read_bytes()
                target.write_bytes(original + marker.encode())
                result = self.run_cli("--check")
                self.assertEqual(result.returncode, 1)
                self.assertIn("có ", result.stderr)
                self.assertNotIn(marker, result.stderr)
                target.write_bytes(original)

        target = self.root / audit.INPUTS["classification"]
        grades = json.loads(target.read_bytes())
        mail = next(row for row in grades["rows"] if row["name"] == "Thư (쪽지)")
        mail["reason"] += " " + samples[0][1]
        target.write_text(json.dumps(grades, ensure_ascii=False), encoding="utf-8")
        result = self.run_cli("--write")
        self.assertEqual(result.returncode, 1)
        self.assertNotIn(samples[0][1], result.stderr)
        for relative in (audit.MATRIX, audit.MARKDOWN):
            self.assertEqual((self.root / relative).read_bytes(), (audit.ROOT / relative).read_bytes())


if __name__ == "__main__":
    unittest.main()
