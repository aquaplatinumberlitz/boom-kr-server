"""Đối chiếu đặc tả F6 với dữ liệu repo và policy Rust; không chạy client."""

import copy
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess
import sys
from tempfile import TemporaryDirectory
import unittest

from tools import kr_f6_extended_events_audit as audit


RUST_POLICY = "rust/crates/kr-service/src/f6_policy.rs"
SCRIPT = "tools/kr_f6_extended_events_audit.py"


def identity(row):
    return row["kind"], row["name"]


def read_json(path):
    return json.loads(path.read_bytes())


def run_cli(root, mode, *, cwd=None):
    return subprocess.run(
        [sys.executable, str(root / SCRIPT), mode],
        cwd=cwd or root, capture_output=True, text=True, check=False,
    )


class ExtendedF6AuditTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.spec = read_json(audit.ROOT / audit.EVIDENCE)
        cls.classification = read_json(audit.ROOT / audit.INPUTS["classification"])
        cls.catalog = read_json(audit.ROOT / audit.INPUTS["catalog"])
        cls.rust = (audit.ROOT / RUST_POLICY).read_text(encoding="utf-8")

    def test_checked_outputs_are_current(self):
        result = run_cli(audit.ROOT, "--check")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("19 in_scope, 25 roadmap_only", result.stdout)

    def test_write_is_reproducible_with_only_docs_and_check_never_repairs(self):
        with TemporaryDirectory() as directory:
            cwd = Path(directory)
            root = cwd / "repo"
            for relative in [SCRIPT, *audit.INPUTS.values()]:
                target = root / relative
                target.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(audit.ROOT / relative, target)
            expected = {relative: (audit.ROOT / relative).read_bytes()
                        for relative in (audit.MARKDOWN, audit.EVIDENCE)}

            for _ in range(2):
                result = run_cli(root, "--write", cwd=cwd)
                self.assertEqual(result.returncode, 0, result.stderr)
                for relative, body in expected.items():
                    self.assertEqual((root / relative).read_bytes(), body)

            for relative, body in expected.items():
                with self.subTest(relative=relative):
                    target = root / relative
                    stale = body + b"\n"
                    target.write_bytes(stale)
                    result = run_cli(root, "--check", cwd=cwd)
                    self.assertEqual(result.returncode, 1)
                    self.assertIn(relative, result.stderr)
                    self.assertEqual(target.read_bytes(), stale)
                    target.write_bytes(body)

            (root / audit.EVIDENCE).unlink()
            result = run_cli(root, "--check", cwd=cwd)
            self.assertEqual(result.returncode, 1)
            self.assertIn(audit.EVIDENCE, result.stderr)
            self.assertFalse((root / audit.EVIDENCE).exists())

    def test_policy_is_exact_rust_allowlist_and_disjoint_from_extended(self):
        declaration = re.search(
            r'const SYSTEM_NAMES: \[&str; (\d+)\] = \[(.*?)\];', self.rust, re.S,
        )
        self.assertIsNotNone(declaration)
        rust_names = [json.loads(line.strip().removesuffix(","))
                      for line in declaration[2].strip().splitlines()]
        self.assertEqual(int(declaration[1]), 19)
        self.assertEqual(len(set(rust_names)), len(rust_names))
        self.assertEqual(len(rust_names), 19)

        policy = [row["name"] for row in self.spec["in_scope"]]
        catalog = [row["system"] for row in self.catalog["systems"] if row["scope"] == "in_scope"]
        self.assertCountEqual(policy, rust_names)
        self.assertCountEqual(policy, catalog)
        extended = self.spec["roadmap_only"]
        self.assertEqual(len({row["id"] for row in extended}), len(extended))
        self.assertEqual(len({row["name"] for row in extended}), len(extended))
        self.assertFalse(set(policy) & {row[field] for row in extended for field in ("id", "name")})
        self.assertEqual({row["policy_context"] for row in extended if "policy_context" in row},
                         {"Pet", "Nhiệm vụ", "Câu cá", "LuckyCard", "MagicMarket"})

    def test_classification_completeness_by_kind_and_name(self):
        source_rows = self.classification["rows"]
        expected = [row for row in source_rows if row["wave"] in ("F6", "F6?")]
        covered = self.spec["in_scope"] + self.spec["roadmap_classification"] + self.spec["map_families"]
        self.assertCountEqual(map(identity, covered), map(identity, expected))

        for key, predicate in (
            ("in_scope", lambda row: row["kind"] == "system" and not row.get("roadmap_only")),
            ("roadmap_classification", lambda row: row["kind"] == "system" and row.get("roadmap_only")),
            ("map_families", lambda row: row["kind"] == "map"),
        ):
            with self.subTest(bucket=key):
                fields = ("kind", "name", "wave", "classification", "status")
                if key == "map_families":
                    fields += ("map_count", "script_count", "script_unknown")
                project = lambda rows: [tuple(row[field] for field in fields) for row in rows]
                self.assertCountEqual(project(self.spec[key]), project(filter(predicate, expected)))

        refs = {identity(row["classification_ref"]) for row in self.spec["roadmap_only"]}
        self.assertEqual(refs & {identity(row) for row in self.spec["in_scope"]},
                         {("system", name) for name in ("Nhiệm vụ", "Câu cá", "LuckyCard", "MagicMarket")})
        self.assertEqual(refs - {identity(row) for row in self.spec["in_scope"]},
                         {identity(row) for row in self.spec["roadmap_classification"] + self.spec["map_families"]})
        self.assertIn(("map", "SelectionBox"), refs)
        self.assertIn(("system", "SelectionBox"), set(map(identity, self.spec["in_scope"])))
        self.assertEqual(self.spec["summary"], {
            "in_scope": 19, "roadmap_classification_systems": 7, "map_families": 5,
            "maps": 5, "roadmap_only": 25, "excluded_policy": 1,
        })
        shared = self.spec["shared_dependencies"]
        self.assertCountEqual(map(identity, shared),
                              (identity(row) for row in source_rows if row["wave"] == "F4/F6"))
        self.assertTrue(shared)
        for row in shared:
            self.assertIs(row["enables_f6"], False)

    def test_exclusion_is_absolute(self):
        match = re.search(r'const EXCLUDED_SYSTEM: &str = "([^"]+)";', self.rust)
        self.assertIsNotNone(match)
        self.assertEqual(match[1], "PCRoomBenefit")
        [row] = self.spec["excluded_policy"]
        self.assertEqual(identity(row), ("system", match[1]))
        self.assertEqual(row["scope"], "excluded_policy")
        self.assertIs(row["enabled"], False)
        self.assertIs(row["enableable"], False)
        self.assertNotIn("id", row)
        self.assertIn("kể cả khi enabled: false", row["rule"])

        for bucket in ("in_scope", "roadmap_only", "roadmap_classification", "map_families", "shared_dependencies"):
            self.assertNotIn("pcroombenefit", json.dumps(self.spec[bucket]).casefold())

    def test_fails_closed_on_classification_drift(self):
        inputs = audit.read_inputs(audit.ROOT)
        changes = (
            ("promote excluded", lambda rows: next(r for r in rows if r["name"] == "PCRoomBenefit").update(
                wave="F6", roadmap_only=True)),
            ("remove excluded", lambda rows: rows.remove(next(r for r in rows if r["name"] == "PCRoomBenefit"))),
            ("reuse exclusion name", lambda rows: rows.append(dict(
                next(r for r in rows if r["name"] == "PCRoomBenefit"), name="pcroombenefit"))),
            ("add system", lambda rows: rows.append(
                {"kind": "system", "name": "NewEvent", "wave": "F6", "roadmap_only": True,
                 "classification": "C", "status": "candidate"})),
            ("remove map", lambda rows: rows.remove(next(r for r in rows if r["name"] == "CAPassShopMap"))),
            ("duplicate map", lambda rows: rows.append(next(r for r in rows if r["name"] == "CAPassShopMap"))),
        )
        for label, change in changes:
            with self.subTest(change=label):
                modified = copy.deepcopy(self.classification)
                change(modified["rows"])
                with self.assertRaises(audit.AuditError):
                    audit.build_spec({**inputs, "classification": json.dumps(modified).encode()})

    def test_rejects_catalog_without_hard_exclusion_even_with_valid_hash(self):
        modified = copy.deepcopy(self.catalog)
        excluded = next(row for row in modified["systems"] if row["system"] == "PCRoomBenefit")
        excluded["scope"] = "related_candidate"
        payload = {key: value for key, value in modified.items() if key not in ("input_sha256", "catalog_sha256")}
        canonical = json.dumps(payload, ensure_ascii=False, sort_keys=True, separators=(",", ":")) + "\n"
        modified["catalog_sha256"] = hashlib.sha256(canonical.encode()).hexdigest()
        with self.assertRaisesRegex(audit.AuditError, "PCRoomBenefit"):
            audit.build_spec({**audit.read_inputs(audit.ROOT), "catalog": json.dumps(modified).encode()})

    def test_rows_carry_disabled_defaults_and_design_gates(self):
        defaults = self.spec["defaults"]
        self.assertIs(defaults["enabled"], False)
        self.assertIsNone(defaults["start_utc"])
        self.assertIsNone(defaults["end_utc"])
        self.assertEqual(defaults["rate_caps"], {"per_account_per_hour": 0, "global_per_minute": 0})
        self.assertEqual(set(self.spec["constraints"]), {"no-real-money-monetization", "no-gambling-for-money"})
        self.assertEqual(self.spec["limits"]["kr_wire"], "unknown")
        self.assertEqual(self.spec["limits"]["wine_acceptance"], "not_run")
        for scope in ("in_scope", "roadmap_only"):
            for row in self.spec[scope]:
                with self.subTest(scope=scope, name=row["name"]):
                    self.assertEqual(row["scope"], scope)
                    self.assertIs(row["enabled"], False)
                    for field in ("quota", "idempotency") + (("wine_probe",) if scope == "roadmap_only" else ()):
                        self.assertTrue(row[field].strip())


if __name__ == "__main__":
    unittest.main()
