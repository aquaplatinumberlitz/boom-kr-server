"""Kiểm phạm vi, bằng chứng và chế độ kiểm ma trận F2 khi offline."""
import json
from pathlib import Path
import tempfile
import unittest

from tools import kr_f2_extended_modes_audit as audit


class ExtendedModesTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.matrix = json.loads(audit.render_outputs(audit.ROOT)[audit.JSON_OUTPUT])

    def test_exact_scope_of_pr_713(self):
        rows = self.matrix["map_families"]
        self.assertEqual(self.matrix["schema"], "kre.f2-extended-modes.v1")
        self.assertEqual([row["family"] for row in rows], [
            "Balansa", "Baram", "BlackLucy", "Clovertest", "ConfusedFight", "DarkCastle",
            "DarkVampire", "Fm_BigBattle", "Fm_BigBattleH_PreparePlay", "Fm_BigBattle_PreparePlay",
            "GoldButterfly", "Hiphopclub", "MagNest", "NewXmas", "OkongOfLegend", "Smash",
            "Space_TW", "UserContest", "WhiteAngel", "WinningStreak_Colosseum", "Wriggletest",
        ])
        self.assertEqual(sum(row["map_count"] for row in rows), 91)
        self.assertEqual(self.matrix["summary"]["existing_mission_families_excluded"], 18)
        self.assertEqual(
            {row["id"] for row in self.matrix["systems"] if row.get("roadmap_only")},
            {"big_brawl", "ca_jjang", "room_item_rules", "supreme_rooms", "vote_kick",
             "super_host", "channel_types", "tournaments"},
        )
        ladder, = [row for row in self.matrix["systems"] if not row.get("roadmap_only")]
        self.assertEqual(ladder["id"], "ladder_bonus_map")
        self.assertEqual(ladder["locator"], "docs/PLAN.md#full-game-roadmap")

    def test_pack_facts_and_slot_remain_separate(self):
        families = {row["family"]: row for row in self.matrix["map_families"]}
        battle = families["Fm_BigBattle"]
        self.assertEqual(battle["spawn_counts"], [{"spawns": 24, "maps": 4}])
        self.assertEqual([(g["width"], g["height"]) for g in battle["grid_sizes"]],
                         [(15, 13), (30, 26), (45, 26), (45, 39)])
        self.assertEqual(battle["grid_class"], "mixed")
        self.assertEqual(battle["catalog_group_candidates"], ["big_battle"])
        self.assertEqual(battle["script_presence"], {"yes": 4, "no": 0, "unknown": 0})
        self.assertEqual(battle["script_unknown"], 4)
        self.assertEqual(families["Smash"]["grid_class"], "nonstandard")
        self.assertEqual(families["Smash"]["catalog_group_candidates"], [])
        self.assertEqual(families["UserContest"]["spawn_counts"],
                         [{"spawns": 2, "maps": 1}, {"spawns": 6, "maps": 1}, {"spawns": 8, "maps": 15}])

        brawl, = [row for row in self.matrix["systems"] if row["id"] == "big_brawl"]
        self.assertIn("12 người", brawl["scope"])
        self.assertNotIn("catalog_group_candidates", brawl)
        for row in [*families.values(), *self.matrix["systems"]]:
            self.assertEqual(row["wire"], "unknown")
            self.assertEqual(row["room_rules"]["status"], "design")
        self.assertTrue(any("capacity() == N" in note for note in self.matrix["invariants"]))

    def test_offline_write_check_and_stale_outputs(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for path in (*audit.JSON_INPUTS, audit.PLAN):
                target = root / path
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes((audit.ROOT / path).read_bytes())

            digests = audit.run(root, check=False)
            self.assertEqual(audit.run(root, check=True), digests)
            for path in digests:
                with self.subTest(path=path):
                    target = root / path
                    original = target.read_bytes()
                    target.write_bytes(original + b"\n")
                    with self.assertRaisesRegex(ValueError, Path(path).name):
                        audit.run(root, check=True)
                    self.assertEqual(target.read_bytes(), original + b"\n")
                    target.write_bytes(original)

            # Rò rỉ trong bản đã lưu phải được báo trước lỗi hash, không in giá trị.
            host_path = b"/" + b"home/" + b"admin/" + b"placeholder"
            target = root / audit.MD_OUTPUT
            target.write_bytes(target.read_bytes() + host_path)
            with self.assertRaisesRegex(ValueError, "thông tin xác thực riêng") as error:
                audit.run(root, check=True)
            self.assertNotIn(host_path.decode(), str(error.exception))

    def test_leak_check(self):
        for raw in (b"endpoint:" + b".".join((b"192", b"168", b"0", b"1")) + b".",
                    b"[" + b"fd12" + b"::1]",
                    b"client_" + b"secret=" + b"placeholder",
                    b"BEGIN " + b"PRIVATE " + b"KEY",
                    b"ocid" + b"1.instance.placeholder"):
            with self.subTest(kind=raw[:3]):
                with self.assertRaises(ValueError):
                    audit.reject_private(raw, "sample")
        audit.reject_private(b"KR00967; 0xC6C080; 15x13; S01-S50; schema v1.2.3", "sample")


if __name__ == "__main__":
    unittest.main()
