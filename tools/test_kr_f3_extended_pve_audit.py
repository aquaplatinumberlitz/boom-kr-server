"""Kiểm phạm vi, nguồn chứng cứ và hai đầu ra F3 khi không có runtime."""

import hashlib
import json
from pathlib import Path
from shutil import copyfile
from tempfile import TemporaryDirectory
import unittest
from unittest.mock import patch

if __package__:
    from . import kr_f3_extended_pve_audit as audit
else:
    import kr_f3_extended_pve_audit as audit


class ExtendedPveTests(unittest.TestCase):
    def test_variants_and_pack_counts(self):
        matrix = audit.run(audit.ROOT, write=False)
        self.assertEqual(matrix["schema"], "kre.f3-extended-pve.v1")
        self.assertEqual(matrix["summary"], {
            "family_count": 8, "map_count": 17, "scripted_map_count": 17, "system_count": 4,
        })
        expected = {
            "Antarctic": (15, 101, 4, ["01"]),
            "DragonPalace": (15, 101, 4, ["01"]),
            "EscapeMaze": (29, 25, 8, ["00", "01", "02"]),
            "GolemRun": (15, 101, 4, ["01"]),
            "IamWriggle": (15, 13, 8, ["01", "02", "03"]),
            "MineExp": (15, 101, 4, ["01"]),
            "Minibot": (15, 13, 4, ["01", "02", "03", "04"]),
            "SavetheEarth": (15, 13, 4, ["01", "02", "03"]),
        }
        families = {row["family"]: row for row in matrix["families"]}
        self.assertEqual(families.keys(), expected.keys())
        for name, (width, height, spawns, suffixes) in expected.items():
            with self.subTest(family=name):
                row = families[name]
                self.assertEqual(row["wave"], "F3?")
                self.assertEqual(row["status"], "candidate")
                self.assertEqual(row["map_count"], len(suffixes))
                self.assertEqual(row["classification_script_unknown"], len(suffixes))
                self.assertEqual(row["variants"], [
                    {"name": name + suffix, "width": width, "height": height, "spawn_points": spawns,
                     "parse": "exact", "scripted": True, "mapscript_tag": True}
                    for suffix in suffixes
                ])
        self.assertEqual(families["Antarctic"]["same_name_mode_classification"]["wave"], "F2?")

    def test_scope_and_static_anchors_stay_separate_from_authority(self):
        matrix = audit.run(audit.ROOT, write=False)
        systems = {row["id"]: row for row in matrix["systems"]}
        expected = {
            "ca_hero": (53, {"hp", "skills", "gear", "potions", "quests"}),
            "ai_battle": (4, {"star_difficulty", "hidden_character_unlock"}),
            "hidden_catch": (37, {"standard", "versus", "arcade"}),
            "monster_season_2": (368, {"season_2_action", "monster_operation"}),
        }
        self.assertEqual(systems.keys(), expected.keys())
        for ident, (maps, keys) in expected.items():
            self.assertEqual(systems[ident]["catalog_group"]["map_count"], maps)
            self.assertEqual(systems[ident]["features"].keys(), keys)
        self.assertIn("로두마니", systems["ai_battle"]["features"]["hidden_character_unlock"])
        self.assertTrue(systems["ai_battle"]["hidden_character_scope"]["roadmap_only"])
        self.assertEqual(systems["hidden_catch"]["roadmap_arcade_stage_count"], 100)
        season = systems["monster_season_2"]["season_2_name_candidate"]
        self.assertEqual((season["map_count"], len(season["families"]), season["status"]), (26, 8, "candidate"))
        self.assertEqual(systems["monster_season_2"]["features"]["monster_operation"], "몬스터대작전")
        anchors = matrix["static_anchors"]
        self.assertEqual(anchors["controller_escapemazescript"]["class"], "CMapScriptEscapeMaze")
        self.assertEqual(anchors["game_type_43"]["symbol"], "CreateDlgGameTypeIamWriggle")
        self.assertEqual(anchors["stage_script"]["hexrays_case"], 80)
        self.assertNotIn("opcode", anchors["stage_script"])
        for row in matrix["families"] + matrix["systems"]:
            self.assertEqual(row["authority_status"], "unknown")
            self.assertEqual(row["kr_wire_route"], "unknown")
            self.assertLessEqual(set(row["static_anchor_ids"]), anchors.keys())
        self.assertEqual(matrix["wine_probe"]["rounds"], ["r2", "r3"])
        self.assertEqual(matrix["wine_probe"]["status"], "pending")

    def test_rejects_drift_between_sources(self):
        data, sources = audit.load_inputs(audit.ROOT)
        census_row = next(row for row in data["census"]["maps"] if row["name"] == "EscapeMaze00")
        census_row["spawns"] -= 1
        with self.assertRaisesRegex(ValueError, r"EscapeMaze: .*CONTENT\.md"):
            audit.build_matrix(data, sources)
        census_row["spawns"] += 1
        data["authority_matrix"]["summary"]["map_count"] = 1
        with self.assertRaisesRegex(ValueError, r"hash nội dung F3"):
            audit.build_matrix(data, sources)

    def test_offline_write_and_check_detect_hash_and_byte_drift(self):
        with TemporaryDirectory() as tmp, patch("socket.socket", side_effect=AssertionError("unexpected socket")):
            root = Path(tmp)
            # Bản thử chỉ có đầu vào đã lưu và thư mục đầu ra, không có Git hoặc dữ liệu runtime.
            for locator in audit.INPUTS.values():
                target = root / locator
                target.parent.mkdir(parents=True, exist_ok=True)
                copyfile(audit.ROOT / locator, target)
            audit.run(root, write=True)
            audit.run(root, write=False)
            for locator in (audit.JSON_OUT, audit.MD_OUT):
                self.assertEqual((root / locator).read_bytes(), (audit.ROOT / locator).read_bytes())

            report = root / audit.JSON_OUT
            raw = report.read_bytes()
            stored = json.loads(raw)
            digest = stored.pop("matrix_sha256")
            payload = json.dumps(stored, ensure_ascii=False, sort_keys=True, separators=(",", ":"))
            self.assertEqual(digest, hashlib.sha256((payload + "\n").encode()).hexdigest())
            self.assertEqual(stored["inputs"].keys(), audit.INPUTS.keys())
            for entry in stored["inputs"].values():
                self.assertEqual(entry["sha256"], hashlib.sha256((root / entry["locator"]).read_bytes()).hexdigest())
            report.write_bytes(raw.replace(digest.encode(), b"0" * 64))
            with self.assertRaisesRegex(ValueError, "SHA-256"):
                audit.run(root, write=False)
            report.write_bytes(raw)
            markdown = root / audit.MD_OUT
            markdown.write_bytes(markdown.read_bytes() + b"\n")
            with self.assertRaisesRegex(ValueError, "đầu ra đã cũ"):
                audit.run(root, write=False)
            markdown.write_text(".".join(map(str, (192, 168, 4, 5))), encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "mẫu dữ liệu riêng"):
                audit.run(root, write=False)

    def test_leak_filter_covers_paths_addresses_and_credential_shapes(self):
        fixtures = (
            "/" + "/".join(("home", "admin", "fixture")),
            ".".join(map(str, (10, 12, 34, 56))),
            ":".join(("fd00", "", "42")),
            "api" + "_key: " + "unit-fixture",
            "gh" + "p_" + "x" * 24,
        )
        for fixture in fixtures:
            escaped = "".join(f"\\u{ord(char):04x}" for char in fixture)
            with self.assertRaisesRegex(ValueError, "mẫu dữ liệu riêng"):
                audit.scan_output(f'{{"value": "{escaped}"}}'.encode(), audit.JSON_OUT)
            with self.assertRaisesRegex(ValueError, "mẫu dữ liệu riêng"):
                audit.scan_output(fixture.encode(), audit.MD_OUT)


if __name__ == "__main__":
    unittest.main()
