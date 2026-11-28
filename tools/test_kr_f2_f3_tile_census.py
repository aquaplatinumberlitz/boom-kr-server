"""Kiểm census F2/F3 bằng map tổng hợp và đầu ra đã lưu, không đọc pack."""

from contextlib import redirect_stderr, redirect_stdout
import io
import json
from pathlib import Path
import shutil
import struct
import sys
from tempfile import TemporaryDirectory
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parent))
import kr_f2_f3_tile_census as census


def named(name):
    raw = name.encode("ascii") + b"\x00"
    return struct.pack("<i", len(raw)) + raw


def payload(width=15, height=13, *, cells=(), placements=(), drops=(), spawns=(), tail=b"", version=1001):
    stride = 2 if version == 1000 else 3
    header = struct.pack("<20i", version, 2, 4, width, height, len(spawns), *([0] * 14))
    spawn_rows = b"".join(struct.pack("<2i", x, y) for x, y in spawns)
    grid = b"".join(struct.pack(f"<{stride}i", *cell) for cell in cells)
    grid += struct.pack(f"<{stride}i", *([-1] * stride)) * (width * height - len(cells))
    drop_rows = b"".join(named(name) + struct.pack("<i", weight) for name, weight in drops)
    placement_rows = b"".join(named(name) + struct.pack("<3B", x, y, probability)
                              for name, x, y, probability in placements)
    return (header + spawn_rows + struct.pack("<i", 0) + grid
            + struct.pack("<i", len(drops)) + drop_rows
            + struct.pack("<2i", 100, len(placements)) + placement_rows + tail)


class TileCensusTests(unittest.TestCase):
    def setUp(self):
        self.names = ["Tile/Ground", "Tile/NovelGround", "Tile/NovelObject", "Tile/NovelOverlay", "Tile/NoInfo"]
        basic = (0, 0, 0, 0, 0, 0, 1, 1, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0)
        self.audit = census.TileAudit((self.names, {
            self.names[0]: basic, self.names[1]: (99, *basic[1:]),
            self.names[2]: (98, *basic[1:]), self.names[3]: (97, *basic[1:]),
        }))

    def test_standard_and_custom_grids_decode_independently_of_size(self):
        for width, height in [(15, 13), (19, 17), (22, 22), (25, 25), (40, 10)]:
            with self.subTest(width=width, height=height):
                row = census.scan_map(payload(width, height, cells=[(1, -1)], version=1000), self.audit)
                self.assertEqual(row["decode"], "exact")
                self.assertEqual(row["grid"], [width, height])
                self.assertEqual(row["unmapped_tiles"], {"Tile/NovelGround": {"tile": "outside_f1e_rule"}})

    def test_decode_errors_preserve_only_supported_metadata(self):
        short, future, truncated = (census.scan_map(raw, self.audit) for raw in
                                    (b"", payload(version=2000), payload(19, 17)[:120]))
        for row in (short, future):
            self.assertEqual((row["decode"], row["layout"]), ("decode_error", "header"))
            self.assertNotIn("grid", row)
        self.assertEqual((truncated["decode"], truncated["layout"], truncated["grid"]),
                         ("decode_error", "cell_buffer", [19, 17]))
        self.assertNotIn("unmapped_tiles", truncated)

    def test_unknown_tail_and_truncated_floor_keep_recoverable_keys(self):
        raw = payload(cells=[(1, -1, -1)], placements=[("CustomPickup", 0, 1, 100)])
        row = census.scan_map(raw + b"?", self.audit)
        self.assertEqual((row["decode"], row["layout"], row["tail_bytes"]), ("partial", "unrecognized_tail", 1))
        self.assertIsNone(row["scripted"])
        self.assertEqual(row["unmapped_placements"], ["CustomPickup"])
        truncated = census.scan_map(raw[:-1], self.audit)
        self.assertEqual((truncated["decode"], truncated["layout"]), ("partial", "floor_rows_unreadable"))
        self.assertEqual(truncated["unmapped_tiles"], row["unmapped_tiles"])
        self.assertIsNone(truncated["unmapped_placements"])

    def test_recognized_group_tail_does_not_become_exact(self):
        tail = struct.pack("<4i", 1, 1, 0, 1) + struct.pack("<2i", 0, 0)
        row = census.scan_map(payload(tail=tail), self.audit)
        self.assertEqual((row["decode"], row["layout"]), ("partial", "group_record_tail"))
        self.assertEqual((row["tail_groups"], row["tail_records"]), (1, 0))
        self.assertFalse(row["scripted"])

    def test_collects_all_layers_and_separates_items_from_tiles(self):
        row = census.scan_map(payload(
            cells=[(1, 2, 3), (0, 4, 22), (1, 2, -1)],
            placements=[("Item/ItemBomb", 0, 4, 100), ("CustomPickup", 1, 4, 100),
                        ("Tile/NovelObject", 2, 4, 100)],
            drops=[("CustomDrop", 1), ("ZeroWeight", 0), ("Item/ItemFire", 3)],
        ), self.audit)
        self.assertEqual(row["unmapped_tiles"], {
            "Tile/NovelGround": {"tile": "outside_f1e_rule"},
            "Tile/NovelObject": {"object": "outside_f1e_rule", "placement": "outside_f1_floor_keys"},
            "Tile/NovelOverlay": {"overlay": "outside_f1e_rule"},
            "Tile/NoInfo": {"object": "missing_tile_info"},
            "#22": {"overlay": "outside_object_table"},
        })
        self.assertEqual(row["unmapped_placements"], ["CustomPickup", "Tile/NovelObject"])
        self.assertEqual(row["unmapped_positive_drops"], ["CustomDrop"])

    def test_mapped_terrain_can_still_block_spawn_with_unknown_tiles_present(self):
        self.names.append("Tile/Box")
        self.audit.tiles["Tile/Box"] = (2, 0, 0, 0, 0, 0, 1, 1, *([0] * 10))
        row = census.scan_map(payload(cells=[(1, -1, -1), (0, 5, -1)], spawns=[(1, 0)]), self.audit)
        self.assertEqual(row["unmapped_tiles"], {"Tile/NovelGround": {"tile": "outside_f1e_rule"}})
        self.assertEqual(row["terrain_constraints"], ["combined_terrain"])

    def test_ranks_families_by_union_and_excludes_layout_and_terrain_constraints(self):
        def row(*keys, grid=(15, 13), decode="exact", constraints=()):
            return dict(grid=list(grid), decode=decode, unmapped_tiles={key: {} for key in keys},
                        terrain_constraints=constraints)

        catalogs = {"F2": {"Many": ["A", "B"], "Three": ["C", "D"], "One": ["E"], "Ready": ["F"],
                           "Custom": ["G", "H"], "Partial": ["I"], "Constrained": ["J"], "BadCode": ["K"]}}
        maps = {
            "A": row("a", "b", "c"), "B": row("c", "d"), "C": row("a", "b"), "D": row("b", "c"),
            "E": row("x"), "F": row(), "G": row("a"), "H": row(grid=(19, 17)),
            "I": row("x", decode="partial"), "J": row("x", constraints=["rectangles"]), "K": row("#999"),
        }
        families = census.build_body(catalogs, {}, maps)["families"]["F2"]
        self.assertEqual([(r["family"], r["priority"]) for r in families],
                         [("Ready", "terrain_ready"), ("One", "quick_unlock"), ("Three", "quick_unlock"),
                          ("Many", "many_tiles"), ("BadCode", "terrain_constraints"),
                          ("Constrained", "terrain_constraints"), ("Partial", "needs_layout"),
                          ("Custom", "custom_grid")])
        self.assertEqual(families[2]["unmapped_tiles"], ["a", "b", "c"])
        self.assertEqual(families[3]["unmapped_tiles"], ["a", "b", "c", "d"])


class OfflineCheckTests(unittest.TestCase):
    def offline_check(self, root):
        stdout, stderr = io.StringIO(), io.StringIO()
        with (patch.object(census, "default_game_dir", side_effect=AssertionError("consulted client path")),
              patch.object(census, "verified_packs", side_effect=AssertionError("opened client packs")),
              redirect_stdout(stdout), redirect_stderr(stderr)):
            try:
                census.main(["--check"], root=root)
            except SystemExit as exc:
                return exc.code, stderr.getvalue()
        return 0, stdout.getvalue()

    def test_checked_in_outputs_pass_offline_and_cover_both_catalogs(self):
        code, message = self.offline_check(census.ROOT)
        self.assertEqual(code, 0, message)
        body = json.loads((census.ROOT / census.JSON_OUT).read_bytes())
        self.assertEqual([(s["maps"], s["families"]) for s in body["summary"].values()], [(122, 18), (699, 180)])
        self.assertEqual(len(body["maps"]), 699)
        for name, key in {"Baseball01": "Tile70_Ground/Tile04", "Gumiho01": "Tile65_Static/Static06",
                          "Kart02": "Tile12_Ground/Race13"}.items():
            self.assertIn(key, body["maps"][name]["unmapped_tiles"])
            self.assertGreater(len(body["maps"][name]["unmapped_tiles"]), 1)
        for name in ("Coloring01", "SuperFighter16", "Hamster01", "Zombie01", "SnowyFields08"):
            self.assertEqual(body["maps"][name]["unmapped_tiles"], {})
        self.assertEqual(body["maps"]["Hdn_SuperFight"]["unmapped_tiles"],
                         {"Tile23_Static/Temple08": {"object": "outside_f1e_rule"}})
        for name in ("MonSnowBear01", "Fm_MonSnowBear01", "MonNoviceSnow01"):
            self.assertIn("Tile33_Passble/SnowBoom", body["maps"][name]["unmapped_tiles"])
            self.assertIn("Tile33_Passble/Pass_Down", body["maps"][name]["unmapped_tiles"])

    def test_rejects_stale_digest_and_noncanonical_json_or_markdown(self):
        with TemporaryDirectory() as temporary:
            root = Path(temporary)
            paths = [*census.CATALOGS.values(), census.JSON_OUT, census.MARKDOWN_OUT,
                     *(Path("tools") / f"{reader}.py" for reader in census.READERS)]
            for path in paths:
                (root / path).parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(census.ROOT / path, root / path)
            json_path, md_path = root / census.JSON_OUT, root / census.MARKDOWN_OUT
            original = json_path.read_bytes()
            stored = json.loads(original)
            stored["maps"]["Baseball01"]["scripted"] = False
            json_path.write_text(json.dumps(stored), encoding="utf-8")
            code, message = self.offline_check(root)
            self.assertEqual(code, 1)
            self.assertIn("SHA-256", message)

            json_path.write_bytes(original + b"\n")
            code, message = self.offline_check(root)
            self.assertEqual(code, 1)
            self.assertIn(census.JSON_OUT.name, message)

            json_path.write_bytes(original)
            md_path.write_bytes(md_path.read_bytes() + b"\n")
            code, message = self.offline_check(root)
            self.assertEqual(code, 1)
            self.assertIn(census.MARKDOWN_OUT.name, message)

    def test_rejects_host_paths_and_ip_literals_without_echoing_them(self):
        samples = ["/".join(("", "home", "example-user", "fixture")),
                   ".".join(map(str, (192, 168, 7, 23))), ":".join(("fd12", "", "abcd"))]
        for sample in samples:
            with self.assertRaises(census.CensusError) as error:
                census.reject_private(f"trước `{sample}` sau")
            self.assertFalse(sample in str(error.exception), "diagnostic echoed input")
        census.reject_private("FxMap/Map/Baseball01 và lưới 15x13.")


if __name__ == "__main__":
    unittest.main()
