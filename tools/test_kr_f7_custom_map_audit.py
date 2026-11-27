"""Kiểm F7 bằng nguồn đã commit và fixture cục bộ, không cần pack game."""

from contextlib import redirect_stderr, redirect_stdout
import hashlib
import io
import json
from pathlib import Path
from tempfile import TemporaryDirectory
import unittest

from tools import kr_f7_custom_map_audit as audit


def map_document(width=15, height=13):
    return {"schema": "kre-f7-normalized-map-v1", "width": width, "height": height,
            "tiles": [0] * (width * height), "spawns": [[0, 0], [width - 1, height - 1]]}


def json_bytes(value):
    return json.dumps(value, indent=2).encode("utf-8")


class InventoryTests(unittest.TestCase):
    def setUp(self):
        temp = TemporaryDirectory()
        self.addCleanup(temp.cleanup)
        self.root = Path(temp.name)
        for relative in audit.INPUTS.values():
            path = self.root / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes((audit.ROOT / relative).read_bytes())
        for relative in (audit.JSON_PATH, audit.MD_PATH):
            (self.root / relative).parent.mkdir(parents=True, exist_ok=True)

    def cli(self, *args):
        stderr = io.StringIO()
        with redirect_stdout(io.StringIO()), redirect_stderr(stderr):
            code = audit.main(args, root=self.root)
        return code, stderr.getvalue()

    def test_write_check_are_deterministic_and_reject_any_stale_output(self):
        self.assertEqual(self.cli("--check")[0], 1)
        self.assertEqual(self.cli("--write"), (0, ""))
        products = {name: (self.root / name).read_bytes() for name in (audit.JSON_PATH, audit.MD_PATH)}
        self.assertEqual(self.cli("--write"), (0, ""))
        self.assertEqual(self.cli("--check"), (0, ""))
        for name, content in products.items():
            with self.subTest(name=name):
                path = self.root / name
                self.assertEqual(content, path.read_bytes())
                path.write_bytes(content + b"\n")
                code, message = self.cli("--check")
                self.assertEqual(code, 1)
                self.assertIn(name, message)
                self.assertEqual(path.read_bytes(), content + b"\n")
                path.write_bytes(content)

    def test_input_pins_and_schema_keep_static_design_and_wire_distinct(self):
        self.assertEqual(self.cli("--write"), (0, ""))
        spec = json.loads((self.root / audit.JSON_PATH).read_bytes())
        self.assertEqual((spec["schema"], spec["feature"], spec["build"]),
                         ("kre-f7-custom-map-spec-v1", "F7", "KR00967"))
        self.assertEqual(set(spec["inputs"]), {"census", "classification", "owner_directive"})
        for source in spec["inputs"].values():
            self.assertEqual(source["sha256"], hashlib.sha256((self.root / source["path"]).read_bytes()).hexdigest())
        pin = spec.pop("spec_sha256")
        canonical = json.dumps(spec, ensure_ascii=False, sort_keys=True, separators=(",", ":")) + "\n"
        self.assertEqual(pin, hashlib.sha256(canonical.encode("utf-8")).hexdigest())
        self.assertIn(pin, (self.root / audit.MD_PATH).read_text(encoding="utf-8"))
        inv, policy, wire = (spec[k] for k in ("inventory", "policy", "wire"))
        self.assertEqual((inv["source"], inv["scope"], inv["resource_group"], inv["resource_entries"], inv["map_count"]),
                         ("kr_static", "committed_pack_census_metadata_only", "fx/CustomMap", 158, 5))
        self.assertEqual(inv["classification"]["map_families"], {"CustomMap15x": 2, "CustomMap21x": 1, "CustomMap31x": 2})
        self.assertEqual({row["name"]: row["grid_allowed_by_design"] for row in inv["maps"]}, {
            "CustomMap15x13": True, "CustomMap15x27": False, "CustomMap21x21": True,
            "CustomMap31x13": True, "CustomMap31x27": False,
        })
        self.assertEqual((policy["source"], policy["authority"]), ("design", "server"))
        self.assertEqual(policy["allowed_grids"], [[15, 13], [21, 21], [31, 13]])
        self.assertEqual((policy["max_file_bytes"], policy["max_payload_bytes"]), (8192, 8192))
        self.assertEqual(set(policy["required_fields"]), set(map_document()))
        self.assertEqual(policy["validate_before"], ["share", "select", "load"])
        self.assertTrue(policy["pinning"]["require_approved_pin_on_share_select_load"])
        self.assertTrue(policy["fail_closed"])
        self.assertFalse(policy["enabled_by_default"] or wire["enabled"] or wire["synthetic_ack"])
        self.assertEqual((wire["status"], wire["upload"], wire["selection"]), ("unmeasured",) * 3)
        self.assertEqual(set(spec["pack_pins"]["fx.idd"]), {"bytes"})

    def test_rejects_census_that_no_longer_matches_classification(self):
        census = self.root / audit.INPUTS["census"]
        census.write_bytes(census.read_bytes() + b"\n")
        code, message = self.cli("--write")
        self.assertEqual(code, 1)
        self.assertIn("SHA-256 census", message)
        self.assertFalse((self.root / audit.JSON_PATH).exists())

    def test_optional_pack_verification_is_read_only_and_checks_pins(self):
        pack = self.root / "operator-pack"
        pack.mkdir()
        census_path = self.root / audit.INPUTS["census"]
        classification_path = self.root / audit.INPUTS["classification"]
        census, classification = (json.loads(path.read_bytes()) for path in (census_path, classification_path))
        for stem in ("FxMap", "fx"):
            idx, idd = (f"fixture {stem} {ext}".encode() for ext in ("idx", "idd"))
            (pack / f"{stem}.idx").write_bytes(idx)
            (pack / f"{stem}.idd").write_bytes(idd)
            census["packs"][stem]["idx_sha256"] = hashlib.sha256(idx).hexdigest()
            census["packs"][stem]["idd_bytes"] = len(idd)
            if stem == "FxMap":
                census["packs"][stem]["idd_sha256"] = hashlib.sha256(idd).hexdigest()
        raw = json_bytes(census)
        census_path.write_bytes(raw)
        classification["census_sha256"] = hashlib.sha256(raw).hexdigest()
        classification_path.write_bytes(json_bytes(classification))
        self.assertEqual(self.cli("--write"), (0, ""))
        self.assertEqual(self.cli("--check", "--pack-root", str(pack)), (0, ""))
        self.assertEqual(self.cli("--write", "--pack-root", str(pack)), (0, ""))
        self.assertEqual(self.cli("--check"), (0, ""))
        fx_map = pack / "FxMap.idd"
        original = fx_map.read_bytes()
        fx_map.write_bytes(b"x" * len(original))
        code, message = self.cli("--write", "--pack-root", str(pack))
        self.assertEqual(code, 1)
        self.assertIn("SHA-256 pack: FxMap.idd", message)
        self.assertNotIn(str(pack), message)
        fx_map.write_bytes(original)
        (pack / "fx.idd").write_bytes(b"")
        code, message = self.cli("--check", "--pack-root", str(pack))
        self.assertEqual(code, 1)
        self.assertIn("kích thước pack: fx.idd", message)
        self.assertEqual(self.cli("--check"), (0, ""))


class ValidationTests(unittest.TestCase):
    palette = {0: True, 1: False}

    def check_map(self, document, **options):
        return audit.validate_payload(json_bytes(document), self.palette, **options)

    def test_only_exact_allowlisted_grid_and_complete_tiles(self):
        for width, height in ((15, 13), (21, 21), (31, 13)):
            with self.subTest(grid=(width, height)):
                self.assertRegex(self.check_map(map_document(width, height)), r"^[a-f0-9]{64}$")
        for width, height in ((15, 27), (31, 27), (13, 15), (22, 21), (0, 13)):
            with self.subTest(grid=(width, height)), self.assertRaisesRegex(audit.AuditError, "lưới"):
                self.check_map(map_document(width, height))
        short = map_document()
        short["tiles"].pop()
        with self.assertRaisesRegex(audit.AuditError, "tile"):
            self.check_map(short)
        for tile in (-1, 2, 65536, True, 0.0):
            with self.subTest(tile=tile), self.assertRaisesRegex(audit.AuditError, "tile"):
                doc = map_document()
                doc["tiles"][1] = tile
                self.check_map(doc)
        with self.assertRaisesRegex(audit.AuditError, "bảng tile"):
            audit.validate_payload(json_bytes(map_document()), {})

    def test_spawn_count_bounds_integrity_and_cardinal_reachability(self):
        for spawns in ([], [[0, 0]], [[x, 0] for x in range(9)], [[0, 0], [0, 0]],
                       [[0, 0], [15, 0]], [[0, 0], [0, -1]], [[0, 0], [True, 1]]):
            with self.subTest(spawns=spawns), self.assertRaisesRegex(audit.AuditError, "xuất phát"):
                self.check_map(dict(map_document(), spawns=spawns))
        with self.assertRaisesRegex(audit.AuditError, "không đủ điểm"):
            self.check_map(map_document(), player_count=3)
        doc = dict(map_document(), spawns=[[0, 0], [1, 1]])
        doc["tiles"][0] = 1
        with self.assertRaisesRegex(audit.AuditError, "ô chặn"):
            self.check_map(doc)
        doc["tiles"][0] = 0
        doc["tiles"][1] = doc["tiles"][15] = 1
        with self.assertRaisesRegex(audit.AuditError, "không thể đi bộ"):
            self.check_map(doc)
        doc["tiles"][15] = 0
        self.check_map(doc, player_count=2)

    def test_file_payload_and_strict_schema_rejections(self):
        good = audit.canonical(map_document())
        limit = good.ljust(8192, b" ")
        self.assertEqual(audit.validate_payload(limit, self.palette), audit.validate_payload(good, self.palette))
        for payload in (b"", limit + b" ", b"\xff", b"[]", good.replace(b'"width":', b'"width":15,"width":')):
            with self.subTest(prefix=payload[:30]), self.assertRaises(audit.AuditError):
                audit.validate_payload(payload, self.palette)
        for field, value in (("schema", "other"), ("script", "code"), ("width", 15.0), ("width", True)):
            with self.subTest(field=field, value=value), self.assertRaises(audit.AuditError):
                self.check_map(dict(map_document(), **{field: value}))
        with TemporaryDirectory() as directory:
            path = Path(directory) / "map.json"
            path.write_bytes(limit)
            self.assertEqual(audit.validate_map_file(path, self.palette), audit.validate_payload(good, self.palette))
            path.write_bytes(limit + b" ")
            with self.assertRaisesRegex(audit.AuditError, "8192 byte"):
                audit.validate_map_file(path, self.palette)

    def test_pin_is_canonical_and_approved_map_rechecks_roster_and_palette(self):
        doc = map_document()
        pin = self.check_map(doc)
        reordered = json_bytes(dict(reversed(list(doc.items()))))
        self.assertEqual(audit.validate_approved_payload(reordered, {1: False, 0: True},
                                                         approved_sha256=pin, player_count=2), pin)
        for palette, approved, count in ((self.palette, None, 2), (self.palette, "a" * 64, 2),
                                          (self.palette, pin, None), (self.palette, pin, 3), ({0: True, 1: True}, pin, 2)):
            with self.subTest(palette=palette, approved=approved, count=count), self.assertRaises(audit.AuditError):
                audit.validate_approved_payload(reordered, palette, approved_sha256=approved, player_count=count)
        doc["tiles"][1] = 1
        self.assertNotEqual(self.check_map(doc), pin)
        with self.assertRaisesRegex(audit.AuditError, "hash map"):
            self.check_map(doc, expected_sha256=pin)


if __name__ == "__main__":
    unittest.main()
