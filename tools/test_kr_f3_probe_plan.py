"""Kiểm thiết kế F3-2 chỉ bằng hai nguồn JSON trong repo."""

from contextlib import redirect_stderr, redirect_stdout
from copy import deepcopy
import io
import json
from pathlib import Path
from tempfile import TemporaryDirectory
import unittest

from tools import kr_f3_probe_plan as probe


class ProbePlanTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.data, cls.sources = probe.load_inputs()
        cls.plan = probe.build_plan(cls.data, cls.sources)
        cls.maps = {row["map"]: row for row in cls.plan["probes"]}

    def test_preserves_f3_1_samples_and_map_evidence(self):
        expected = {
            "first": {"MonSeal01", "MonS2Fire03", "SnowyFields07", "RPG11_N_02", "CAhero01", "CAhero07",
                      "Hdn_Alche_Aqua", "Hdn_Alche_Cam", "BonusLargeGourd01", "BonusBigRPG01",
                      "Fm_AIBtlDesert05", "Tutorial01"},
            "followup": {"MonSnowBear22", "Baseball01", "Hamster01", "Wriggle01", "RescueET01",
                         "SnowyFields01", "PopcadeA01"},
        }
        self.assertEqual(len(self.maps), len(self.plan["probes"]))
        for priority, names in expected.items():
            self.assertEqual({row["map"] for row in self.maps.values() if row["priority"] == priority}, names)
        self.assertEqual(self.maps["MonSeal01"]["static_map"]["family"], "MonSeal")
        self.assertIn("die", self.maps["MonSeal01"]["static_map"]["script_triggers"])
        self.assertIn("MoveMap", self.maps["RPG11_N_02"]["static_map"]["script_tags"])
        self.assertEqual(self.maps["CAhero07"]["static_map"]["has_script"], "no")
        self.assertEqual(self.maps["CAhero07"]["static_map"]["family"], self.maps["CAhero01"]["static_map"]["family"])
        self.assertEqual(self.maps["SnowyFields07"]["scope"], "cross_reference")
        self.assertEqual(self.maps["PopcadeA01"]["static_map"]["spawn_points"], 1)

    def test_anchor_case_labels_are_not_wire_opcodes(self):
        static = self.plan["static_anchors"]
        self.assertEqual((static["source"], static["status"]), ("kr_static", "confirmed"))
        anchors = static["anchors"]
        self.assertEqual(anchors["script_factory"]["va"], "0xeba2d0")
        guard = anchors["monster_attack_guard"]
        self.assertEqual((guard["symbol"], guard["host_flag_expr"], guard["attack_seq_expr"]),
                         ("RollAttack::MonsterAttackReq", "*(this + 157)", "*(this + 316)"))
        stage = [row for row in anchors.values() if row.get("symbol") == "CGameStage::OnPacket"]
        self.assertEqual({row["static_case_label"]: row["handler_va"] for row in stage},
                         {9: "0xe6aa90", 80: "0xe6d730", 135: "0xe67640", 136: "0xe67770", 177: "0xe751f0"})
        for row in stage:
            self.assertEqual(row["dispatcher_va"], "0xe5d3a0")
            self.assertEqual((row["raw_wire_opcode"], row["jump_table_index"]), ("unknown", "unknown"))
            self.assertEqual(row["wire_status"], "pending_wine_capture")
            self.assertNotIn("opcode", row)
        self.assertIn("kiểm phép trừ 20", static["wire_caveat"])
        for sample in self.maps.values():
            observation = sample["anchor_observation"]
            self.assertEqual(observation["status"], "candidate")
            self.assertLessEqual(set(observation["refs"]), anchors.keys())
        self.assertIn("monster_attack_guard", self.maps["MonSeal01"]["anchor_observation"]["refs"])

    def test_authority_stays_unknown_and_all_maps_have_controls(self):
        questions = self.plan["authority_questions"]
        self.assertEqual({key for key, row in questions.items() if row["axis"] == "host_vs_server"},
                         {"monster_spawn", "monster_ai", "monster_hp", "trigger_execution", "stage_clear",
                          "stage_clear_rewards"})
        self.assertTrue(all((row["status"], row["capture_status"]) == ("unknown", "pending_wine_capture")
                            for row in questions.values()))
        steps = {row["id"]: row["action"] for row in self.plan["two_client_procedure"]}
        self.assertIn("A tạo phòng", steps["qualify"])
        self.assertIn("B tạo/bắt đầu", steps["swap"])
        self.assertIn("TCP/UDP", steps["prepare"])
        self.assertEqual(len({row["acceptance"]["observable"] for row in self.maps.values()}), len(self.maps))
        for row in self.maps.values():
            self.assertEqual((row["authority_status"], row["wire_status"]), ("unknown", "pending_wine_capture"))
            self.assertEqual(set(row["question_refs"]), questions.keys())
            self.assertNotEqual(row["experiment"]["action"], row["experiment"]["control"])
            self.assertIn(row["experiment"]["procedure_ref"], self.plan)
            self.assertTrue(all(row["acceptance"][result] for result in ("pass", "fail", "blocked")))
        self.assertIn("chỉ chơi đơn", self.maps["Tutorial01"]["acceptance"]["blocked"])

    def test_rejects_mismatched_sources(self):
        data = deepcopy(self.data)
        data["matrix"]["wine_f3_2"]["samples"].pop()
        with self.assertRaisesRegex(ValueError, "hash nội dung ma trận F3-1"):
            probe.build_plan(data, self.sources)

        sources = deepcopy(self.sources)
        sources["ida_anchors"]["sha256"] = "0" * 64
        with self.assertRaisesRegex(ValueError, "không khớp hash neo IDA"):
            probe.build_plan(self.data, sources)

    def set_up_isolated_root(self, root):
        for location in probe.INPUTS.values():
            path = root / location
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes((probe.ROOT / location).read_bytes())
        (root / probe.DESIGN).parent.mkdir(parents=True)

    def test_write_and_check_are_deterministic_with_only_two_inputs(self):
        with TemporaryDirectory() as temp, redirect_stdout(io.StringIO()):
            root = Path(temp)
            self.set_up_isolated_root(root)
            paths = (root / probe.PLAN, root / probe.DESIGN)
            probe.main(["--write"], root=root)
            first = [path.read_bytes() for path in paths]
            self.assertEqual(json.loads(first[0]), self.plan)
            probe.main(["--write"], root=root)
            probe.main(["--check"], root=root)
            self.assertEqual([path.read_bytes() for path in paths], first)
            self.assertEqual({path.relative_to(root).as_posix() for path in root.rglob("*") if path.is_file()},
                             {*probe.INPUTS.values(), probe.PLAN, probe.DESIGN})

    def test_check_rejects_drift_in_either_output_without_overwriting(self):
        with TemporaryDirectory() as temp, redirect_stdout(io.StringIO()):
            root = Path(temp)
            self.set_up_isolated_root(root)
            probe.main(["--write"], root=root)
            for location in (probe.PLAN, probe.DESIGN):
                with self.subTest(location=location), redirect_stderr(io.StringIO()) as error:
                    path = root / location
                    original = path.read_bytes()
                    stale = original + b"\n"
                    path.write_bytes(stale)
                    with self.assertRaises(SystemExit) as exit_info:
                        probe.main(["--check"], root=root)
                    self.assertEqual(exit_info.exception.code, 1)
                    self.assertIn(location + " chưa đồng bộ", error.getvalue())
                    self.assertEqual(path.read_bytes(), stale)
                    path.write_bytes(original)


if __name__ == "__main__":
    unittest.main()
