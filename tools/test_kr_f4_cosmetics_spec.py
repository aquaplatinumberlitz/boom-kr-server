"""Kiểm ma trận F4 chỉ với nguồn trong repo và bản sao tạm của các nguồn đó."""

from contextlib import redirect_stderr, redirect_stdout
import hashlib
import io
import json
from pathlib import Path
import re
from tempfile import TemporaryDirectory
import unittest

from tools import kr_f4_cosmetics_spec as spec


class CosmeticsSpecTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.outputs = spec.generate(spec.ROOT)
        cls.matrix = json.loads(cls.outputs[spec.JSON_PATH])
        cls.targets = cls.matrix["targets"]

    def test_category_groups_and_slots_keep_their_evidence_scope(self):
        by_category = {row["catalog_category"] for row in self.targets.values()} - {None}
        self.assertEqual(by_category, {"character", "collection", "consumable", "decoration", "emotion"})
        self.assertEqual(set(self.matrix["catalog"]["object_groups"]), {
            "CharTE/AvatarDeco", "CharTE/BombDeco", "CharTE/CharDeco", "CharTE/FishingDeco",
            "CharTE/GradeDeco", "CharTE/LadderDeco", "EmoTE/IDDeco", "EmoTE/Slot",
            "EmoTE/Character", "EmoTE/Fanfare",
        })
        self.assertEqual(
            {(row["slot"]["name"], row["slot"]["state"])
             for row in self.targets.values() if row["slot"]["name"]},
            {("character", "preset"), ("cosmetic.avatar", "proposed"), ("cosmetic.char", "proposed"),
             ("cosmetic.grade", "conditional"), ("cosmetic.bomb", "proposed"), ("cosmetic.id", "proposed"),
             ("emote", "fixture"), ("quick", "fixture"), ("backup", "fixture"), ("head", "fixture")},
        )
        for target in ("balloon", "flag", "effect", "outfit", "vehicle", "ring"):
            with self.subTest(target=target):
                row = self.targets[target]
                self.assertIsNone(row["slot"]["name"])
                self.assertIsNone(row["catalog_category"])
                self.assertEqual(row["schema8_persistence"], {"status": "unknown", "anchors": []})
        self.assertEqual(self.targets["vehicle"]["context_anchors"], ["f1_vehicle_pickup"])

    def test_only_base_character_has_measured_projection(self):
        confirmed = {
            (target, surface): value["anchor"]
            for target, row in self.targets.items()
            for surface, value in row["projection"].items()
            if value["status"] == "confirmed"
        }
        self.assertEqual(confirmed, {
            ("character", "kr_wire"): "character92",
            ("character", "room_slot"): "character_room_slot",
            ("character", "start121"): "character_start121",
        })
        for target, row in self.targets.items():
            self.assertEqual(set(row["projection"]), {"kr_wire", "own291", "start121", "room_slot"})
            self.assertEqual(row["inventory_capture_gap"], "in_game_219_224_225_313_317_566_pending_capture")
            for surface, value in row["projection"].items():
                if (target, surface) not in confirmed:
                    self.assertEqual(value, {"status": "unknown", "gap": "unmeasured_wire_projection"})
        anchors = self.matrix["anchors"]
        self.assertEqual(anchors["character_start121"]["fields"]["character"], "u16 LE tại row[3..5]")
        self.assertEqual(anchors["character_room_slot"]["fields"]["character"],
                         "bit 16..28 inclusive của packed u32")
        self.assertEqual(set(anchors["own291_progression"]["fields"]), {"level", "exp_permille"})
        gap = self.matrix["gaps"]["in_game_219_224_225_313_317_566_pending_capture"]
        self.assertEqual(set(gap["s2c"]), {"219", "224", "225", "313", "314", "315", "316", "317"})
        self.assertEqual(set(gap["c2s"]), {"566"})
        self.assertEqual(gap["s2c_225"]["entries_per_outer_element"], 8)
        self.assertEqual(gap["s2c_225"]["slot_mapping"], "unknown")

    def test_db_preference_equipment_and_consumption_are_separate(self):
        anchors = self.matrix["anchors"]
        self.assertEqual(
            [(anchors[key]["migration"], anchors[key]["table"], anchors[key]["key"])
             for key in ("db_character_preference", "db_equipped", "db_consumption")],
            [(3, "player_preferences", ["player_id"]), (7, "player_equipped", ["player_id", "slot"]),
             (8, "match_item_consumptions", ["player_id", "idempotency_key"])],
        )
        self.assertIn("db_character_preference", self.targets["character"]["schema8_persistence"]["anchors"])
        self.assertNotIn("db_consumption", self.targets["hat"]["schema8_persistence"]["anchors"])
        self.assertEqual(self.targets["quick"]["schema8_persistence"]["anchors"],
                         ["db_inventory", "db_equipped", "db_consumption"])
        self.assertEqual(self.matrix["projection_context"]["o1"], ["o1_session_cosmetics"])

    def test_sha256_pins_and_checked_in_outputs(self):
        matrix = json.loads(self.outputs[spec.JSON_PATH])
        for entry in matrix["inputs"].values():
            source = spec.ROOT / entry["path"]
            self.assertEqual(hashlib.sha256(source.read_bytes()).hexdigest(), entry["sha256"])
        claimed = matrix.pop("spec_sha256")
        canonical = json.dumps(matrix, ensure_ascii=False, sort_keys=True, separators=(",", ":")) + "\n"
        self.assertEqual(hashlib.sha256(canonical.encode("utf-8")).hexdigest(), claimed)
        self.assertIn(claimed.encode("ascii"), self.outputs[spec.MD_PATH])
        for path, data in self.outputs.items():
            self.assertEqual((spec.ROOT / path).read_bytes(), data, path)

    def test_cli_roundtrip_detects_stale_inputs_and_both_stale_outputs(self):
        with TemporaryDirectory() as directory:
            root = Path(directory)
            for entry in self.matrix["inputs"].values():
                path = root / entry["path"]
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes((spec.ROOT / entry["path"]).read_bytes())
            (root / spec.MD_PATH).parent.mkdir(parents=True, exist_ok=True)

            def run(arg):
                stderr = io.StringIO()
                with redirect_stdout(io.StringIO()), redirect_stderr(stderr):
                    result = spec.main([arg], root=root)
                return result, stderr.getvalue()

            self.assertEqual(run("--write"), (0, ""))
            self.assertEqual(run("--check"), (0, ""))
            for relative, expected in self.outputs.items():
                path = root / relative
                self.assertEqual(path.read_bytes(), expected)
                path.write_bytes(expected + b"\n")
                result, diagnostic = run("--check")
                self.assertEqual(result, 1)
                self.assertIn(relative, diagnostic)
                self.assertEqual(path.read_bytes(), expected + b"\n")
                path.write_bytes(expected)

            source = root / self.matrix["inputs"]["architecture"]["path"]
            source.write_bytes(source.read_bytes() + b"\n")
            self.assertEqual(run("--check")[0], 1)
            self.assertEqual(run("--write"), (0, ""))
            self.assertEqual(run("--check"), (0, ""))
            self.assertNotEqual((root / spec.JSON_PATH).read_bytes(), self.outputs[spec.JSON_PATH])

    def test_anchor_change_requires_review_even_in_write_mode(self):
        data, sources = spec.load_inputs(spec.ROOT)
        data["room_wire"] = data["room_wire"].replace(
            "u16::from_le_bytes([row[3], row[4]])", "u16::from_le_bytes([row[4], row[5]])",
        )
        with self.assertRaisesRegex(spec.SpecError, "room.rs"):
            spec.build_spec(data, sources)

    def test_new_exporter_group_requires_matrix_row(self):
        data, sources = spec.load_inputs(spec.ROOT)
        data["exporter"] = re.sub(r'"FishingDeco"', '"FishingDeco", "NewDeco"', data["exporter"], count=1)
        with self.assertRaisesRegex(spec.SpecError, "ObjectTable"):
            spec.build_spec(data, sources)

    def test_private_patterns_are_rejected_without_echo(self):
        for body in (
            "/home/" + "admin/private",
            "http://" + ".".join(("203", "0", "113", "42")),
            "https://[" + "2001:db8::4" + "]:443",
            "http://[" + "::1" + "]:443",
            "-----BEGIN " + "OPENSSH PRIVATE KEY-----",
            '"private_' + 'key": "redacted"',
        ):
            with self.subTest(kind=body.split()[0][:5]):
                with self.assertRaises(spec.SpecError) as error:
                    spec.reject_private(body)
                self.assertNotIn(body, str(error.exception))
        spec.reject_private("KR 00967; SHA-256; OpenSSH public key; KRE 1.2.3")


if __name__ == "__main__":
    unittest.main()
