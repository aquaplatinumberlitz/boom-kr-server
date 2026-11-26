"""Kiểm ma trận F3-1 bằng JSON đã lưu, không đọc pack hay chạy Wine."""

from collections import Counter
from contextlib import redirect_stderr, redirect_stdout
from copy import deepcopy
import hashlib
import io
import json
from pathlib import Path
from tempfile import TemporaryDirectory
import unittest
from unittest.mock import patch

from tools import kr_f3_script_matrix as audit


def digest(payload):
    body = json.dumps(payload, ensure_ascii=False, sort_keys=True, separators=(",", ":")) + "\n"
    return hashlib.sha256(body.encode("utf-8")).hexdigest()


def repin_catalog(catalog):
    catalog["catalog_sha256"] = digest({key: value for key, value in catalog.items()
                                        if key not in {"catalog_sha256", "input_sha256"}})


class ScriptMatrixTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.data, cls.sources = audit.load_inputs()
        cls.matrix = audit.build_matrix(cls.data, cls.sources)
        cls.maps = {row["name"]: row for row in cls.matrix["maps"]}
        cls.families = {(row["stage_group"], row["family"]): row for row in cls.matrix["families"]}

    def test_catalog_scope_matches_every_map_and_family(self):
        expected = set()
        for family in self.data["f3_pve_catalog"]["families"]:
            grade = self.families[family["group"], family["family"]]["classification"]
            self.assertEqual(grade, family["classification"])
            for parse, names in family["maps"].items():
                expected.update(names)
                for name in names:
                    row = self.maps[name]
                    self.assertEqual((row["stage_group"], row["family"], row["parse"], row["classification"]),
                                     (family["group"], family["family"], parse, grade["classification"]))
        self.assertEqual(set(self.maps), expected)
        self.assertEqual(len(self.maps), len(self.matrix["maps"]))
        self.assertEqual(self.matrix["summary"], {
            "map_count": 699, "family_count": 180, "parse_counts": {"exact": 692, "partial": 7},
            "has_script": {"yes": 593, "no": 99, "unknown": 7},
            "family_classification_counts": {"A": 0, "B": 0, "C": 180},
        })
        self.assertEqual(len(self.families), 180)
        self.assertNotIn("Village10", self.maps)
        self.assertEqual(self.families["quest_mission", "Bula"]["classification"]["wave"], "F2?")
        mission = next(group for group in self.matrix["groups"] if group["id"] == "quest_mission")
        self.assertEqual((mission["scope"], mission["map_count"], mission["family_count"]),
                         ("cross_reference", 122, 18))

    def test_tags_are_observations_and_partial_does_not_mean_no(self):
        mon, ai = (self.maps[name] for name in ("Fm_MonSeal01", "Fm_AIBtlDesert05"))
        self.assertEqual((mon["has_script"], mon["mapscript_tag"], mon["literal_script_tag"]), ("yes", "yes", "no"))
        self.assertIn("StageClearScript", mon["script_tags"])
        self.assertIn("die", mon["script_triggers"])
        self.assertEqual((ai["parse"], ai["has_script"], ai["mapscript_tag"]), ("exact", "no", "no"))
        self.assertEqual(ai["script_tags"], [])
        partial = {row["name"] for row in self.maps.values() if row["parse"] == "partial"}
        self.assertEqual(partial, {"Bula00", "Bula01", "Bula02", "Bula03", "Bula04", "Bula15", "Bula16"})
        for name in partial:
            row = self.maps[name]
            self.assertEqual((row["has_script"], row["mapscript_tag"], row["literal_script_tag"],
                              row["script_cluster"]), ("unknown", "unknown", "unknown", "unresolved"))

        data = deepcopy(self.data)
        row = next(row for row in data["census"]["maps"] if row["name"] == "Bula00")
        row["script_tags"] = ["Trigger", "script"]
        result = audit.build_matrix(data, self.sources)
        bula = next(row for row in result["maps"] if row["name"] == "Bula00")
        self.assertEqual((bula["has_script"], bula["literal_script_tag"], bula["script_tags"]),
                         ("unknown", "yes", ["Trigger", "script"]))

    def test_abc_grades_follow_classification_without_granting_authority(self):
        data = deepcopy(self.data)
        catalog = data["f3_pve_catalog"]
        grades = {"CAhero": "A", "Tutorial": "B"}
        for row in data["classification"]["rows"]:
            if row["kind"] == "map" and row["name"] in grades:
                row["classification"] = grades[row["name"]]
        for row in catalog["families"]:
            if row["family"] in grades:
                row["classification"]["classification"] = grades[row["family"]]
        repin_catalog(catalog)
        changed = audit.build_matrix(data, self.sources)
        self.assertEqual(changed["summary"]["family_classification_counts"], {"A": 1, "B": 1, "C": 178})
        self.assertEqual(changed["authority"], self.matrix["authority"])

    def test_static_anchors_do_not_resolve_unknown_authority(self):
        evidence, authority = self.matrix["static_anchors"], self.matrix["authority"]
        self.assertEqual((evidence["source"], evidence["status"]), ("kr_static", "confirmed"))
        anchors = evidence["anchors"]
        factory = anchors["script_factory"]
        self.assertEqual((factory["va"], factory["controller_count"]), ("0xeba2d0", 15))
        self.assertEqual(factory["controller_indices"], [0, 1, 2, *range(4, 16)])
        self.assertEqual(
            {row["opcode"]: row["handler_va"] for row in anchors.values() if "opcode" in row},
            {9: "0xe6aa90", 80: "0xe6d730", 135: "0xe67640", 136: "0xe67770", 177: "0xe751f0"},
        )
        attack = anchors["monster_attack_guard"]
        self.assertEqual(attack["host_flag_fn"], "0x11656a0")
        self.assertIn("or matching attackSeq", attack["notes"])
        self.assertEqual(authority["scope"], "all_maps_and_stage_groups")
        self.assertEqual(set(authority["gaps"]), {"monster_spawn", "monster_hp", "trigger_execution", "stage_clear",
                                                   "stage_clear_rewards", "monster_ai", "kr_wire_route"})
        for gap in authority["gaps"].values():
            self.assertEqual(gap["status"], "unknown")
            self.assertTrue(gap["probe"])
        for group in self.matrix["groups"]:
            self.assertEqual(group["probe_anchors_status"], "candidate")
            self.assertLessEqual(set(group["probe_anchors"]), set(anchors))

    def test_wine_samples_are_exact_and_cover_modes_and_clusters(self):
        plan = self.matrix["wine_f3_2"]
        self.assertEqual((plan["source"], plan["run_status"]), ("design", "pending"))
        selected = plan["samples"]
        self.assertEqual(len({row["map"] for row in selected}), len(selected))
        primary = Counter(row["stage_group"] for row in selected if row["priority"] == "first")
        self.assertEqual(primary, {"monster": 2, "quest_mission": 1, "rpg": 1, "ca_hero": 2,
                                   "hidden_catch": 2, "bonus": 2, "ai_battle": 1, "tutorial": 1})
        self.assertEqual({row["script_cluster"] for row in selected},
                         {row["script_cluster"] for row in self.maps.values() if row["parse"] == "exact"})
        for sample in selected:
            row = self.maps[sample["map"]]
            self.assertEqual(row["parse"], "exact")
            self.assertGreater(row["spawn_points"], 0)
            self.assertEqual((sample["stage_group"], sample["script_cluster"]),
                             (row["stage_group"], row["script_cluster"]))
        self.assertEqual({row["map"] for row in selected if row["script_cluster"] == "no_script"},
                         {"CAhero07", "Hdn_Alche_Cam", "Fm_AIBtlDesert05"})

    def test_hash_covers_inputs_and_render_is_deterministic(self):
        result = json.loads(audit.render_json(self.matrix))
        files = {
            "f3_pve_catalog": "docs/evidence/content/f3-pve-catalog.json",
            "ida_anchors": "docs/evidence/content/f2-f5-f3-ida-anchors.json",
            "census": "docs/content/census.json",
            "classification": "docs/content/classification.json",
        }
        self.assertEqual(result["inputs"], {
            key: {"locator": name, "sha256": hashlib.sha256((audit.ROOT / name).read_bytes()).hexdigest()}
            for key, name in files.items()
        })
        recorded = result.pop("matrix_sha256")
        self.assertEqual(digest(result), recorded)
        self.assertIn(recorded, audit.render_markdown(self.matrix))
        # The join remains stable regardless of index traversal or map order within a family.
        data = deepcopy(self.data)
        data["census"]["maps"].reverse()
        data["classification"]["rows"].reverse()
        catalog = data["f3_pve_catalog"]
        catalog["families"].reverse()
        for family in catalog["families"]:
            family["maps"]["exact"].reverse()
        repin_catalog(catalog)
        regenerated = audit.build_matrix(data, self.sources)
        self.assertEqual(audit.render_json(regenerated), audit.render_json(self.matrix))
        self.assertEqual(audit.render_markdown(regenerated), audit.render_markdown(self.matrix))

    def test_bad_input_pairing_and_wrong_map_parse_fail(self):
        sources = deepcopy(self.sources)
        sources["classification"]["sha256"] = "0" * 64
        with self.assertRaisesRegex(ValueError, "hash đầu vào"):
            audit.build_matrix(self.data, sources)

        data = deepcopy(self.data)
        data["ida_anchors"]["f3_pve"]["status"] = "kr_static/candidate"
        with self.assertRaisesRegex(ValueError, "chưa có xác nhận tĩnh"):
            audit.build_matrix(data, self.sources)

        data = deepcopy(self.data)
        catalog = data["f3_pve_catalog"]
        family = next(row for row in catalog["families"] if row["family"] == "Bula")
        family["maps"]["partial"].remove("Bula00")
        family["maps"]["exact"].append("Bula00")
        repin_catalog(catalog)
        with self.assertRaisesRegex(ValueError, "không khớp tên, họ, nhóm hoặc trạng thái"):
            audit.build_matrix(data, self.sources)

    def test_generate_and_check_both_outputs_without_rewriting_stale_files(self):
        with TemporaryDirectory() as tmp, redirect_stdout(io.StringIO()), redirect_stderr(io.StringIO()):
            json_path, md_path = Path(tmp) / "matrix.json", Path(tmp) / "matrix.md"
            with patch.multiple(audit, MATRIX=json_path, MARKDOWN=md_path):
                audit.main([])
                originals = {path: path.read_bytes() for path in (json_path, md_path)}
                audit.main([])
                self.assertEqual({path: path.read_bytes() for path in originals}, originals)
                audit.main(["--check"])
                for path, original in originals.items():
                    with self.subTest(path=path.name):
                        path.write_bytes(b"stale")
                        with self.assertRaises(SystemExit) as exc:
                            audit.main(["--check"])
                        self.assertEqual(exc.exception.code, 1)
                        self.assertEqual(path.read_bytes(), b"stale")
                        path.write_bytes(original)

    def test_private_metadata_fails_before_output(self):
        private_values = ("/home/operator/source", "10.0.0.3", "172.31.0.3", "192.168.1.2",
                          "198." + "18.1.2", "127.0.0.1", "169.254.0.3", "100.64.1.3",
                          "fd00::123", "fe80::7", "::1",
                          "ocid" + "1.example", "BEGIN OPENSSH PRIVATE" + " KEY")
        data = deepcopy(self.data)
        row = next(row for row in data["census"]["maps"] if row["name"] == "Fm_MonSeal01")
        with TemporaryDirectory() as tmp, redirect_stdout(io.StringIO()):
            json_path, md_path = Path(tmp) / "matrix.json", Path(tmp) / "matrix.md"
            with patch.multiple(audit, MATRIX=json_path, MARKDOWN=md_path), patch.object(
                audit, "load_inputs", return_value=(data, self.sources)
            ):
                for i, value in enumerate(private_values):
                    row["script_funcs"] = [value]
                    output = io.StringIO()
                    with self.subTest(kind=i), redirect_stderr(output):
                        with self.assertRaises(SystemExit) as exc:
                            audit.main([])
                        self.assertEqual(exc.exception.code, 1)
                        self.assertNotIn(value, output.getvalue())
                        self.assertFalse(json_path.exists())
                        self.assertFalse(md_path.exists())
        self.assertNotIn("/home/", audit.render_json(self.matrix))
        self.assertNotIn("/home/", audit.render_markdown(self.matrix))


if __name__ == "__main__":
    unittest.main()
