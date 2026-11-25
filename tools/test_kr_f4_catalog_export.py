"""Kiểm exporter F4 trên các pack tổng hợp, không cần client KR."""

from copy import deepcopy
import json
from pathlib import Path
import struct
from tempfile import TemporaryDirectory
from types import SimpleNamespace
import unittest
from unittest.mock import patch

import kr_f4_catalog_export as f4


def string(name):
    data = name.encode("ascii")
    return struct.pack("<I", len(data)) + data


def bomber(key):
    return string("Character/DuplicateName") + struct.pack("<11i", key, *([0] * 10))


def item(name, group):
    return string(name) + string("Item/Icon") + struct.pack("<8i", 0, 0, group, *([0] * 5))


def object_table(*names):
    index_end = 4 + sum(len(string(name)) + 4 for name in names)
    return (
        struct.pack("<I", len(names))
        + b"".join(
            string(name) + struct.pack("<I", 0xffffffff if "/" not in name else index_end + i)
            for i, name in enumerate(names)
        )
        + bytes(len(names))
    )


FIXTURES = {
    "FxContents": {
        "Attr/BomberAttr": bomber(101) + bomber(221),
        "Attr/ItemAttr": struct.pack("<3I", 2, 0, 0) + item("Lucci", 99) + item("Item/Example", 1),
    },
    "CharTE": {
        "ObjectTable/CharTEObjectTable": object_table("BombDeco", "BombDeco/Bomb01", "_ForITE/C01"),
    },
    "EmoTE": {
        "ObjectTable/EmoTEObjectTable": object_table("Character/C01_Emotion", "Slot/Slot01"),
    },
    "fx": {
        "ObjectTable/ObjectTable": object_table("Pet/Pet_001", "Pet/Pet_EffectSave", "Tile/Ground"),
    },
}


def synthetic_packs(game):
    pins = {}
    for stem, entries in FIXTURES.items():
        index = struct.pack("<3I", 0, 0, len(entries))
        data = b""
        for name, payload in entries.items():
            index += string(name) + struct.pack("<4I", len(data), len(payload), 0, 0)
            data += payload
        index += bytes(-len(index) % 4)
        words = [word for (word,) in struct.iter_unpack("<I", index)]
        encrypted = b"".join(struct.pack("<I", word ^ previous)
                             for word, previous in zip(words, [0, *words]))
        for suffix, raw in (("idx", encrypted), ("idd", data)):
            name = f"{stem}.{suffix}"
            (game / name).write_bytes(raw)
            pins[name] = f4.sha(raw)
    return pins


class CatalogExportTest(unittest.TestCase):
    def setUp(self):
        self.preset = f4.load_json(f4.STANDARD)
        self.preset["consumables"] = {"Item/Example": 77}
        self.packs = {stem: SimpleNamespace(read=entries.__getitem__) for stem, entries in FIXTURES.items()}

    def build(self, overrides=None, **options):
        return f4.build_export(self.packs, self.preset, {} if overrides is None else overrides, **options)

    def test_standard_schema_and_source_mapping(self):
        result = self.build()
        policy = result["economy_policy"]
        catalog = result["shop_catalog"]
        f4.validate_policy(policy)
        f4.validate_catalog(catalog)
        self.assertEqual(set(result), {"economy_policy", "shop_catalog", "summary"})
        self.assertFalse(policy["enabled"])
        self.assertEqual(policy["level_thresholds"][:3], [0, 400, 1000])
        self.assertEqual(policy["level_thresholds"][345], 5_835_900)
        self.assertEqual(policy["level_thresholds"][575], 34_412_900)
        self.assertEqual(policy["level_thresholds"][-1], 78_000_000)
        self.assertEqual(result["summary"], {
            "counts_by_category": dict(character=2, collection=1, consumable=1, decoration=2, emotion=1),
            "level_count": 720, "unlock_all": False, "catalog_sha256": f4.sha(f4.encode(catalog)),
        })
        items = {row["sku_id"]: row for row in catalog["items"]}
        self.assertEqual(list(items), [
            "character.101", "character.221", "collection.fx.Pet.Pet_001", "consumable.1",
            "decoration.CharTE.BombDeco.Bomb01", "decoration.EmoTE.Slot.Slot01",
            "emotion.EmoTE.Character.C01_Emotion",
        ])
        self.assertEqual(items["consumable.1"]["price_lucci"], 77)
        self.assertEqual(items["character.221"]["equip_slot"], "character")
        self.assertEqual(items["character.221"]["grant"], {"kind": "permanent"})

    def test_override_precedence_and_grants(self):
        override = {
            "economy_policy": {
                "enabled": False,
                "win": {"exp": 234, "lucci": 56},
                "loss": {"lucci": 3},
                "draw": {"exp": 4},
                "grant_on_disconnect_or_left": True,
                "level_thresholds": [0, 125, 500],
            },
            "items": {
                "character.101": {"price_lucci": 121, "disabled": True, "grant": {
                    "kind": "timed", "duration_seconds": 3600}, "equip_slot": None},
                "consumable.1": {"grant": {"kind": "stackable", "max_stack": 50}},
            },
        }
        result = self.build(override, enable_economy=True, levels=3)
        self.assertEqual(result["economy_policy"], {
            "enabled": True, "win": {"exp": 234, "lucci": 56}, "loss": {"exp": 30, "lucci": 3},
            "draw": {"exp": 4, "lucci": 50}, "grant_on_disconnect_or_left": True,
            "level_thresholds": [0, 125, 500],
        })
        self.assertEqual(result["summary"]["level_count"], 3)
        items = {row["sku_id"]: row for row in result["shop_catalog"]["items"]}
        self.assertEqual(items["character.101"], {
            "sku_id": "character.101", "category": "character", "price_lucci": 121, "enabled": False,
            "grant": {"kind": "timed", "duration_seconds": 3600}, "equip_slot": None,
        })
        self.assertEqual(items["consumable.1"]["grant"], {"kind": "stackable", "max_stack": 50})
        self.assertTrue(self.build({"economy_policy": {"enabled": True}})["economy_policy"]["enabled"])
        with self.assertRaisesRegex(ValueError, "--levels"):
            self.build(override, levels=2)

    def test_o1_zeroes_all_categories_and_preserves_disabled_items(self):
        override = {"items": {"character.101": {"price_lucci": 123, "disabled": True}}}
        result = self.build(override, unlock_all=True)
        items = result["shop_catalog"]["items"]
        self.assertEqual({item["category"] for item in items}, set(f4.CATEGORIES))
        self.assertTrue(all(item["price_lucci"] == 0 for item in items))
        self.assertFalse(items[0]["enabled"])
        self.assertTrue(result["summary"]["unlock_all"])
        self.assertNotEqual(result["summary"]["catalog_sha256"], self.build()["summary"]["catalog_sha256"])

    def test_level_count_can_truncate_or_extend_preset(self):
        self.assertEqual(self.build(levels=1)["economy_policy"]["level_thresholds"], [0])
        extended = self.build(levels=10000)["economy_policy"]["level_thresholds"]
        self.assertEqual(len(extended), 10000)
        self.assertEqual(extended[719:722], [78_000_000, 78_600_000, 79_200_000])
        for levels in (0, 10001):
            with self.subTest(levels=levels), self.assertRaisesRegex(ValueError, "--levels"):
                self.build(levels=levels)

    def test_policy_rejects_invalid_schema_and_accepts_rust_limits(self):
        f4.validate_policy(self.build({"economy_policy": {
            "level_thresholds": [0, 2**63 - 1], "win": {"exp": 10**12, "lucci": 10**12},
        }})["economy_policy"])
        invalid = [
            {"win": {"exp": -1}}, {"loss": {"lucci": 10**12 + 1}}, {"draw": {"exp": True}},
            {"draw": {"lucci": 2.5}}, {"win": {"xp": 1}}, {"enabled": 1},
            {"grant_on_disconnect_or_left": "false"}, {"level_thresholds": []}, {"level_thresholds": [1]},
            {"level_thresholds": [0, 10, 10]}, {"level_thresholds": [0, 10, 9]},
            {"level_thresholds": [0, 2**63]}, {"level_thresholds": [0, True]},
            {"level_thresholds": list(range(10001))},
        ]
        for fields in invalid:
            with self.subTest(fields=list(fields)), self.assertRaises(ValueError):
                self.build({"economy_policy": fields}, enable_economy=True)

    def test_catalog_rejects_invalid_schema_and_accepts_rust_limits(self):
        result = self.build({"items": {
            "character.101": {"price_lucci": 10**12, "equip_slot": "A" * 32,
                              "grant": {"kind": "timed", "duration_seconds": 10**12}},
            "consumable.1": {"grant": {"kind": "stackable", "max_stack": 2**32 - 1}},
        }})
        item = result["shop_catalog"]["items"][0]
        f4.validate_catalog({"items": [dict(item, sku_id="A" * 64)]})
        invalid = [
            {"sku_id": ""}, {"sku_id": "A" * 65}, {"sku_id": "sku/invalid"}, {"sku_id": "sku.é"},
            {"category": "unknown"}, {"equip_slot": ""}, {"equip_slot": "A" * 33}, {"equip_slot": "bad:slot"},
            {"price_lucci": -1}, {"price_lucci": 10**12 + 1}, {"enabled": 1},
            {"grant": {"kind": "permanent", "max_stack": 1}}, {"grant": {"kind": "unknown"}},
            {"grant": {"kind": "stackable"}}, {"grant": {"kind": "stackable", "max_stack": 0}},
            {"grant": {"kind": "stackable", "max_stack": 2**32}},
            {"grant": {"kind": "timed", "duration_seconds": 0}},
            {"grant": {"kind": "timed", "duration_seconds": 10**12 + 1}},
        ]
        for fields in invalid:
            with self.subTest(fields=fields), self.assertRaises(ValueError):
                f4.validate_catalog({"items": [dict(item, **fields)]})
        with self.assertRaisesRegex(ValueError, "sku_id trùng"):
            f4.validate_catalog({"items": [item, item]})
        with self.assertRaisesRegex(ValueError, "tối đa"):
            f4.validate_catalog({"items": [item] * 100001})

    def test_unknown_or_invalid_overrides_fail_including_o1_price(self):
        for override in ({"extra": 1}, {"items": []}, {"items": {"absent": {"disabled": True}}},
                         {"items": {"character.101": {"disabled": 1}}},
                         {"items": {"character.101": {"price_lucci": -1}}},
                         {"items": {"character.101": {"category": "emotion"}}}):
            with self.subTest(override=override), self.assertRaises(ValueError):
                self.build(override, unlock_all=True)

    def test_object_index_rejects_payload_offsets_inside_index(self):
        with self.assertRaisesRegex(ValueError, "offset"):
            buf = struct.pack("<I", 1) + string("Pet/Pet_001") + struct.pack("<I", 4)
            list(f4.object_names(buf))

    def test_cli_reads_synthetic_packs_and_outputs_deterministically(self):
        with TemporaryDirectory() as tmp:
            root = Path(tmp)
            game = root / "game"
            game.mkdir()
            pins = synthetic_packs(game)
            standard, overrides = root / "standard.json", root / "override.json"
            standard.write_bytes(f4.encode(self.preset))
            overrides.write_text('{"items":{"consumable.1":{"price_lucci":101}}}', encoding="utf-8")
            input_args = ["--game-dir", str(game), "--overrides", str(overrides), "--enable-economy", "--levels", "2"]
            with patch.multiple(f4, STANDARD=standard, F4_PACK_SHA256=pins):
                outputs = [root / f"catalog{i}.json" for i in range(2)]
                for out in outputs:
                    f4.main(input_args + ["--out", str(out)])
                a, b = [out.read_bytes() for out in outputs]
                self.assertEqual(a, b)
                self.assertNotIn(str(root).encode(), a)
                payload = json.loads(a)
                self.assertEqual(payload["economy_policy"]["level_thresholds"], [0, 400])
                self.assertTrue(payload["economy_policy"]["enabled"])
                self.assertEqual(next(row["price_lucci"] for row in payload["shop_catalog"]["items"]
                                      if row["sku_id"] == "consumable.1"), 101)
                with self.assertRaisesRegex(ValueError, "ngoài thư mục game"):
                    f4.write_output(game / "new.json", game, a)
                self.assertFalse((game / "new.json").exists())
                with self.assertRaises(FileExistsError):
                    f4.write_output(outputs[0], game, b"new")
                self.assertEqual(outputs[0].read_bytes(), a)
                self.assertEqual(f4.sha((game / "FxContents.idd").read_bytes()), pins["FxContents.idd"])

    def test_json_overrides_reject_duplicate_keys(self):
        with TemporaryDirectory() as tmp:
            path = Path(tmp) / "overrides.json"
            path.write_text('{"items":{"a":{"disabled":false,"disabled":true}}}', encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "khóa trùng"):
                f4.load_json(path)


if __name__ == "__main__":
    unittest.main()
