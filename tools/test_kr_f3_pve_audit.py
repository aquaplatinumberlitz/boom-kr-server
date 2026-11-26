"""Kiểm kê F3-0 bằng fixture và metadata đã lưu, không mở pack gốc."""

from copy import deepcopy
import hashlib
import io
import json
from contextlib import redirect_stderr, redirect_stdout
from pathlib import Path
from tempfile import TemporaryDirectory
import unittest
from unittest.mock import patch

from tools import kr_f3_pve_audit as audit


def map_row(name, family, category, **fields):
    return {"name": name, "family": family, "category": category, "parse": "exact", "scripted": False} | fields


def fixture():
    census = {
        "build": "KR00967",
        "client_exe_sha256": "fixture-client",
        "maps": [
            map_row("MonS2Fire01", "MonS2Fire", "monster", scripted=True, script_tags=["Trigger", "Time", "Time"],
                    script_triggers=["default"], script_funcs=["waitonly"]),
            map_row("MonS2Fire02", "MonS2Fire", "monster", parse="partial", scripted=True, script_tags=["action"]),
            map_row("Monster01", "Monster", "monster", scripted=True, script_tags=["mapscript"]),
            map_row("Bula01", "Bula", "mission", parse="partial"),
            map_row("PopcadeA01", "PopcadeA", "mission", scripted=True, script_tags=["Trigger", "trigger"]),
            map_row("RPG01", "RPG", "rpg", scripted=True, script_tags=["RPGScript"]),
            map_row("CAhero01", "CAhero", "rpg"),
            map_row("Hidden01", "Hidden", "hidden", scripted=True, script_tags=["mapscript"]),
            map_row("Bonus01", "Bonus", "bonus"),
            map_row("Fm_AIBtlForest01", "Fm_AIBtlForest", "battle_variant"),
            map_row("Tutorial01", "Tutorial", "lobby", scripted=True, script_tags=["TutorialScript"]),
            map_row("DailyMissionShop00", "DailyMissionShop", "lobby"),
            map_row("Battle01", "Battle", "battle_theme", scripted=True, script_tags=["OutsideScope"]),
            map_row("Fm_CAPong01", "Fm_CAPong", "battle_variant"),
        ],
        "systems": [
            {"name": "Monster", "groups": ["FxMonster/MonsterSeason2", "FxMonster/Monster"]},
            {"name": "Nhiệm vụ", "groups": ["FxDesktop/Quest"]},
        ],
        "resource_groups": [
            {"pack": "FxMonster", "name": "Monster", "entries": 5},
            {"pack": "FxMonster", "name": "MonsterSeason2", "entries": 3},
            {"pack": "FxDesktop", "name": "Quest", "entries": 2},
            {"pack": "FxMap", "name": "BonusPrepare", "entries": 1},
        ],
    }
    classification = {"rows": [
        dict(kind="map", name="MonS2Fire", map_count=2, classification="C", status="candidate", wave="F3"),
        dict(kind="map", name="Bula", map_count=1, classification="B", status="candidate", wave="F2?"),
        dict(kind="map", name="Bonus", map_count=1, classification="A", status="confirmed", wave="F3"),
        dict(kind="mode", name="Nhiệm vụ chơi với máy", classification="C", status="candidate", wave="F3"),
        dict(kind="system", name="Nhiệm vụ", classification="C", status="candidate", wave="F6"),
    ]}
    return census, classification


class PveAuditTests(unittest.TestCase):
    def test_scope_resources_and_classifications(self):
        catalog = audit.build_catalog(*fixture())
        groups = {row["id"]: row for row in catalog["groups"]}
        families = {row["family"]: row for row in catalog["families"]}
        self.assertEqual({key: row["map_count"] for key, row in groups.items()}, {
            "monster": 3, "quest_mission": 2, "rpg": 1, "ca_hero": 1, "hidden_catch": 1,
            "bonus": 1, "ai_battle": 1, "tutorial": 1,
        })
        self.assertEqual(catalog["summary"], {
            "map_count": 11, "family_count": 10, "parse_counts": {"exact": 9, "partial": 2},
            "f3_core": {"map_count": 9, "family_count": 8, "parse_counts": {"exact": 8, "partial": 1}},
        })
        self.assertEqual(groups["monster"]["season_2_name_candidate"], {
            "family_prefix": "MonS2", "status": "candidate", "map_count": 2, "families": ["MonS2Fire"],
            "parse_counts": {"exact": 1, "partial": 1},
        })
        self.assertEqual(groups["monster"]["resource_groups"], [
            {"locator": "FxMonster/Monster", "entries": 5}, {"locator": "FxMonster/MonsterSeason2", "entries": 3},
        ])
        mission = groups["quest_mission"]
        self.assertEqual(mission["scope"], "cross_reference")
        self.assertEqual(mission["resource_groups"], [{"locator": "FxDesktop/Quest", "entries": 2}])
        self.assertEqual(mission["mode_classifications"]["Nhiệm vụ chơi với máy"]["wave"], "F3")
        self.assertEqual(mission["system_classification"]["grade"]["wave"], "F6")
        self.assertEqual(mission["classification_counts"], {"A": 0, "B": 1, "C": 0, "unknown": 1})
        self.assertEqual(families["Bula"]["classification"]["wave"], "F2?")
        self.assertIsNone(families["PopcadeA"]["classification"])
        self.assertEqual(families["Bonus"]["classification"]["classification"], "A")
        self.assertTrue(set(families).isdisjoint({"DailyMissionShop", "Battle", "Fm_CAPong"}))

    def test_parse_and_script_observations_remain_separate(self):
        catalog = audit.build_catalog(*fixture())
        families = {row["family"]: row for row in catalog["families"]}
        season = families["MonS2Fire"]
        self.assertEqual(season["parse_counts"], {"exact": 1, "partial": 1})
        self.assertEqual(season["maps"], {"exact": ["MonS2Fire01"], "partial": ["MonS2Fire02"]})
        self.assertEqual(season["has_script"], {"yes": 1, "no": 0, "unknown": 1})
        self.assertEqual(season["script_symbols"], {
            "exact": {"script_tags": ["Time", "Trigger"], "script_triggers": ["default"],
                      "script_funcs": ["waitonly"]},
            "partial_observed": {"script_tags": ["action"], "script_triggers": [], "script_funcs": []},
        })
        self.assertEqual(families["Bula"]["has_script"], {"yes": 0, "no": 0, "unknown": 1})
        self.assertEqual(families["Bonus"]["has_script"], {"yes": 0, "no": 1, "unknown": 0})
        self.assertEqual(families["PopcadeA"]["script_symbols"]["exact"]["script_tags"], ["Trigger", "trigger"])

    def test_no_authority_or_wire_inferred_from_tags_and_grades(self):
        catalog = audit.build_catalog(*fixture())
        expected = {"trigger_execution": "unknown", "monster_ai": "unknown",
                    "rewards": "unknown", "kr_wire_route": "unknown"}
        self.assertEqual(catalog["authority"], expected)
        for group in catalog["groups"]:
            with self.subTest(group=group["id"]):
                self.assertEqual(group["authority"], expected)
        self.assertIn("là `unknown` ở mọi nhóm", audit.render_markdown(catalog))

    def test_serialization_order_and_hash(self):
        census, classification = fixture()
        catalog = audit.build_catalog(census, classification)
        reordered = deepcopy(census)
        for rows in reordered.values():
            if isinstance(rows, list):
                rows.reverse()
        reordered["systems"][-1]["groups"].reverse()
        reordered["maps"][-1]["script_tags"].reverse()
        other = audit.build_catalog(reordered, {"rows": classification["rows"][::-1]})
        self.assertEqual(audit.render_json(catalog), audit.render_json(other))
        self.assertEqual(audit.render_markdown(catalog), audit.render_markdown(other))
        payload = json.loads(audit.render_json(catalog))
        digest = payload.pop("catalog_sha256")
        canonical = json.dumps(payload, ensure_ascii=False, sort_keys=True, separators=(",", ":")) + "\n"
        self.assertEqual(hashlib.sha256(canonical.encode()).hexdigest(), digest)

    def test_generate_is_idempotent_and_check_detects_stale_output(self):
        catalog = audit.build_catalog(*fixture())
        with TemporaryDirectory() as tmp, redirect_stdout(io.StringIO()), redirect_stderr(io.StringIO()):
            json_path, md_path = Path(tmp) / "out.json", Path(tmp) / "out.md"
            with patch.multiple(audit, CATALOG=json_path, MARKDOWN=md_path), patch.object(
                audit, "load_catalog", return_value=catalog
            ):
                audit.main([])
                first = json_path.read_bytes(), md_path.read_bytes()
                audit.main([])
                self.assertEqual(first, (json_path.read_bytes(), md_path.read_bytes()))
                audit.main(["--check"])
                md_path.write_text("cũ", encoding="utf-8")
                with self.assertRaises(SystemExit) as error:
                    audit.main(["--check"])
                self.assertEqual(error.exception.code, 1)
                self.assertEqual(md_path.read_text(encoding="utf-8"), "cũ")

    def test_private_metadata_rejected(self):
        for value in ("/home/operator/game", "/Users/operator/game", "10.0.0.3", "172.31.0.3",
                      "192.168.0.3", "198." + "18.1.3", "ocid" + "1.example",
                      "-----BEGIN RSA PRIVATE" + " KEY-----"):
            census, classification = fixture()
            census["maps"][0]["script_tags"].append(value)
            with self.subTest(value=value), self.assertRaisesRegex(ValueError, "định danh riêng"):
                audit.build_catalog(census, classification)

    def test_inconsistent_metadata_rejected(self):
        census, classification = fixture()
        with self.assertRaisesRegex(ValueError, "trùng tên map"):
            audit.build_catalog(census | {"maps": census["maps"] + [
                map_row("mons2fire01", "Outside", "other"),
            ]}, classification)
        census["maps"][0]["parse"] = "invalid"
        with self.assertRaisesRegex(ValueError, "exact/partial"):
            audit.build_catalog(census, classification)
        census["maps"][0]["parse"] = "exact"
        classification["rows"][0]["map_count"] = 1
        with self.assertRaisesRegex(ValueError, "số map theo họ"):
            audit.build_catalog(census, classification)

    def test_classification_must_be_from_same_census(self):
        census, classification = fixture()
        classification.update(build=census["build"], client_sha256=census["client_exe_sha256"],
                              census_sha256="stale")
        with TemporaryDirectory() as tmp:
            census_path, class_path = Path(tmp) / "census.json", Path(tmp) / "class.json"
            census_path.write_text(json.dumps(census), encoding="utf-8")
            class_path.write_text(json.dumps(classification), encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "census_sha256"):
                audit.load_catalog(census_path, class_path)

    def test_saved_metadata_covers_all_requested_families(self):
        catalog = audit.load_catalog()
        groups = {row["id"]: row for row in catalog["groups"]}
        self.assertEqual({key: (row["parse_counts"]["exact"], row["parse_counts"]["partial"], row["family_count"])
                          for key, row in groups.items()}, {
            "monster": (368, 0, 61), "quest_mission": (115, 7, 18), "rpg": (71, 0, 52), "ca_hero": (53, 0, 1),
            "hidden_catch": (37, 0, 30), "bonus": (43, 0, 13), "ai_battle": (4, 0, 4), "tutorial": (1, 0, 1),
        })
        self.assertTrue(all(row["classification"] is not None for row in catalog["families"]))
        self.assertIn("RewardItem", groups["bonus"]["script_symbols"]["exact"]["script_tags"])
        self.assertIn("TutorialScript", groups["tutorial"]["script_symbols"]["exact"]["script_tags"])


if __name__ == "__main__":
    unittest.main()
