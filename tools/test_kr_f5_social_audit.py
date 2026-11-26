"""Kiểm hành vi catalog, fixture tổng hợp và hai đầu ra đã lưu."""

import hashlib
import json
from pathlib import Path
import subprocess
import sys
from tempfile import TemporaryDirectory
import unittest

if __package__:
    from .kr_f5_social_audit import (
        CATALOG, CENSUS, CLASSIFICATION, MARKDOWN, SYSTEMS, build_catalog, render_json, render_markdown,
    )
else:
    from kr_f5_social_audit import (
        CATALOG, CENSUS, CLASSIFICATION, MARKDOWN, SYSTEMS, build_catalog, render_json, render_markdown,
    )


SCRIPT = Path(__file__).with_name("kr_f5_social_audit.py")
CENSUS_HASH = "b" * 64
CLASS_HASH = "c" * 64


def fixture():
    sys_groups = [
        ("FxDesktop/Chatting", "FxDesktop/WhisperLog"),
        ("FxDesktop/GlobalUI:BtMessenger",),
        ("FxDesktop/Etc:Guild", "FxDesktop/Guild"),
        ("fx/Couple",),
        ("FxContents/UXCAJJangRatingMark",),
    ]
    resources, systems, rows = [], [], []
    packs = {}
    for (_, name, _), groups in zip(SYSTEMS, sys_groups):
        for locator in groups:
            pack, group = locator.split("/")
            packs[pack] = {"idx_sha256": "d" * 64}
            resources.append({"pack": pack, "name": group, "entries": len(resources) + 1, "system": name})
        systems.append({"name": name, "groups": list(groups),
                        "entries": sum(row["entries"] for row in resources if row["system"] == name)})
        rows.append({"kind": "system", "name": name, "wave": "F5", "classification": "C",
                     "status": "candidate", "groups": list(groups)})
    return (
        {"schema": "kre-content-census-v1", "build": "KR00967", "client_exe_sha256": "a" * 64,
         "resource_groups": resources, "systems": systems, "packs": packs},
        {"schema": "kre-content-classification-v1", "build": "KR00967", "source": "kr_static",
         "client_sha256": "a" * 64, "census_sha256": CENSUS_HASH, "rows": rows},
    )


def audit(census, grades, *, census_hash=CENSUS_HASH):
    return build_catalog(census, grades, census_hash=census_hash, classification_hash=CLASS_HASH)


class SocialAuditTests(unittest.TestCase):
    def test_five_subsystems_and_unknown_wire(self):
        census, grades = fixture()
        grades["rows"][0]["classification"] = "B"
        grades["rows"][0]["opcode"] = 123
        result = audit(census, grades)
        self.assertEqual(result["summary"], {"subsystems": 5, "resource_groups": 7, "entries": 28})
        self.assertEqual([r["id"] for r in result["subsystems"]],
                         ["Chatting", "Messenger", "Guild", "Couple", "Ranking"])
        self.assertEqual([r["entries"] for r in result["subsystems"]], [3, 3, 9, 6, 7])
        self.assertTrue(all(r["kr_wire"]["status"] == "unknown" for r in result["subsystems"]))
        self.assertEqual(result["subsystems"][0]["classification"]["classification"], "B")
        self.assertEqual(result["subsystems"][1]["resource_groups"][0], {
            "locator": "FxDesktop/GlobalUI:BtMessenger", "entries": 3, "idx_sha256": "d" * 64,
        })
        self.assertNotIn("opcode", render_json(result))

    def test_canonical_hash_and_order(self):
        census, grades = fixture()
        result = audit(census, grades)
        for rows in (census["systems"], census["resource_groups"], grades["rows"]):
            rows.reverse()
        for row in census["systems"] + grades["rows"]:
            row["groups"].reverse()
        self.assertEqual(render_json(result), render_json(audit(census, grades)))
        digest = result.pop("catalog_sha256")
        payload = json.dumps(result, ensure_ascii=False, sort_keys=True, separators=(",", ":")) + "\n"
        self.assertEqual(digest, hashlib.sha256(payload.encode()).hexdigest())

    def test_source_hash_build_schema_and_scope_mismatch(self):
        with self.assertRaisesRegex(ValueError, "build hoặc hash"):
            audit(*fixture(), census_hash="e" * 64)
        for key, value, error in (
            ("build", "KR00968", "build hoặc hash"),
            ("schema", "unknown", "schema"),
            ("client_sha256", "e" * 64, "build hoặc hash"),
            ("rows", [], "phạm vi"),
        ):
            with self.subTest(key=key):
                census, grades = fixture()
                grades[key] = value
                with self.assertRaisesRegex(ValueError, error):
                    audit(census, grades)

    def test_duplicate_and_misaligned_groups_rejected(self):
        census, grades = fixture()
        census["resource_groups"].append(census["resource_groups"][0])
        with self.assertRaisesRegex(ValueError, "trùng khóa"):
            audit(census, grades)
        census, grades = fixture()
        grades["rows"][0]["groups"][0] = "FxDesktop/Other"
        with self.assertRaisesRegex(ValueError, "không khớp"):
            audit(census, grades)
        census, grades = fixture()
        census["resource_groups"][0]["entries"] = 123
        with self.assertRaisesRegex(ValueError, "số mục"):
            audit(census, grades)

    def test_unsafe_locators_rejected_even_when_census_agrees(self):
        census, grades = fixture()
        for row in (census["systems"][0], grades["rows"][0]):
            row["groups"][0] = "/" + "home" + "/operator"
        with self.assertRaisesRegex(ValueError, "locator"):
            audit(census, grades)

    def test_checked_in_outputs_match_and_do_not_leak(self):
        result = subprocess.run([sys.executable, SCRIPT, "--check"], capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("5 phân hệ", result.stdout)
        raw = CATALOG.read_text(encoding="utf-8")
        self.assertEqual(json.loads(raw)["summary"], {"subsystems": 5, "resource_groups": 7, "entries": 127})
        md = MARKDOWN.read_text(encoding="utf-8")
        self.assertEqual(md, render_markdown(json.loads(raw)))
        self.assertNotRegex(raw + md, r"/" r"home/|198" r"\.18\.|ocid" r"1\.|PRIVATE KEY")
        self.assertEqual(md.count("\x60unknown\x60"), 5)

    def test_cli_output_and_safe_destination(self):
        with TemporaryDirectory() as tmp:
            json_out, md_out = Path(tmp) / "test.json", Path(tmp) / "test.md"
            cli = [sys.executable, SCRIPT, "--census", CENSUS, "--classification", CLASSIFICATION]
            result = subprocess.run(cli + ["--out", json_out, "--markdown-out", md_out],
                                    capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(json_out.read_bytes(), CATALOG.read_bytes())
            self.assertEqual(md_out.read_bytes(), MARKDOWN.read_bytes())
            result = subprocess.run(cli + ["--check", "--out", json_out, "--markdown-out", md_out],
                                    capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            md_out.write_text("stale")
            result = subprocess.run(cli + ["--check", "--out", json_out, "--markdown-out", md_out],
                                    capture_output=True, text=True)
            self.assertEqual(result.returncode, 1)
            result = subprocess.run(cli + ["--out", CENSUS], capture_output=True, text=True)
            self.assertEqual(result.returncode, 1)
            self.assertIn("riêng với đầu vào", result.stderr)


if __name__ == "__main__":
    unittest.main()
