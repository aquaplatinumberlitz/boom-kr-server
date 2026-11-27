"""Kiểm catalog F4 bằng file đã commit và bản sao tạm, không cần mạng hay pack."""

import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys
from tempfile import TemporaryDirectory
import unittest

from tools import kr_f4_progression_rewards_audit as audit


SCRIPT = "tools/kr_f4_progression_rewards_audit.py"


def seal(catalog):
    body = {key: value for key, value in catalog.items() if key != "catalog_sha256"}
    canonical = json.dumps(body, ensure_ascii=False, sort_keys=True, separators=(",", ":")) + "\n"
    catalog["catalog_sha256"] = hashlib.sha256(canonical.encode("utf-8")).hexdigest()
    return json.dumps(catalog, ensure_ascii=False).encode("utf-8")


class CatalogTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.raw = (audit.ROOT / audit.CATALOG).read_bytes()
        cls.expected = audit.build_catalog(audit.ROOT)

    def test_schema_and_body_integrity(self):
        audit.validate_catalog(self.raw, self.expected)
        catalog = json.loads(self.raw)
        self.assertEqual(catalog["schema"], "kre.f4-progression-rewards.v1")
        self.assertEqual(set(catalog["systems"]), {"titles", "level_up_rewards", "lucci_vault", "record_erasers"})
        for entry in catalog["systems"].values():
            self.assertEqual(entry["wire_contract_status"], "unknown")
            self.assertEqual(entry["reward_tables_status"], "unknown")
            self.assertEqual(entry["original_server_rules_status"], "unknown")
            self.assertIs(entry["server_configurable_default"]["enabled"], False)
            facts = set(entry["known_account_serving_facts"])
            self.assertLessEqual({"schema_7", "schema_8", "account_serving"}, facts)
            self.assertLessEqual(facts, set(catalog["server_foundation"]))
        catalog["systems"]["titles"]["wire_contract_status"] = "confirmed"
        with self.assertRaisesRegex(ValueError, "catalog_sha256"):
            audit.validate_catalog(json.dumps(catalog).encode(), self.expected)

    def test_cannot_reseal_unsupported_claims_or_defaults(self):
        cases = [
            (("schema",), "kre.f4-progression-rewards.v2"),
            (("systems", "extra_system"), {}),
            (("systems", "titles", "wire_contract_status"), "confirmed"),
            (("systems", "titles", "wire_opcode"), 1),
            (("systems", "level_up_rewards", "reward_tables_status"), "confirmed"),
            (("systems", "level_up_rewards", "server_configurable_default", "character_milestones"), {}),
            (("systems", "lucci_vault", "server_configurable_default", "base_lucci_cap"), 100),
            (("systems", "record_erasers", "server_configurable_default", "enabled"), True),
            (("systems", "record_erasers", "server_configurable_default", "allowed_scopes"), ["ladder"]),
            (("systems", "record_erasers", "server_configurable_default", "require_audit"), False),
            (("systems", "record_erasers", "server_configurable_default", "require_audit"), 1),
        ]
        for path, value in cases:
            with self.subTest(path=path, value=value):
                catalog = json.loads(self.raw)
                node = catalog
                for key in path[:-1]:
                    node = node[key]
                node[path[-1]] = value
                with self.assertRaisesRegex(ValueError, "schema|sai trường"):
                    audit.validate_catalog(seal(catalog), self.expected)

    def test_duplicate_json_keys_rejected(self):
        ambiguous = b'{"schema": null,' + self.raw.lstrip()[1:]
        with self.assertRaisesRegex(ValueError, "khóa JSON trùng"):
            audit.validate_catalog(ambiguous, self.expected)

    def test_leak_guard_rejects_paths_ips_and_credential_shapes(self):
        path = "/" + "/".join(("home", "fixture", "client"))
        for candidate in (path, path.replace("/", "%2F"), ".".join(map(str, (10, 0, 0, 1))),
                          ":".join(("fd00", "", "1")) + ".", "gh" + "p_" + "x" * 36,
                          "password" + "=" + "fixture-value"):
            with self.assertRaises(ValueError) as raised:
                audit.require_no_leaks(candidate)
            self.assertNotIn(candidate, str(raised.exception))
        catalog = json.loads(self.raw)
        catalog["systems"]["titles"]["unknowns"] = path
        escaped = seal(catalog).replace(path.encode(), path.replace("/", r"\u002f").encode())
        with self.assertRaisesRegex(ValueError, "đường dẫn"):
            audit.validate_catalog(escaped, self.expected)

    def test_cli_checks_standalone_repo_files_and_detects_drift(self):
        with TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            for name in (SCRIPT, audit.CLASSIFICATION, audit.CATALOG, audit.MARKDOWN,
                         *(fact[0] for fact in audit.FOUNDATION.values())):
                target = root / name
                target.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(audit.ROOT / name, target)

            def cli(option):
                return subprocess.run([sys.executable, "-I", str(root / SCRIPT), option], cwd=root,
                                      capture_output=True, text=True, timeout=10)

            json_path, md_path = root / audit.CATALOG, root / audit.MARKDOWN
            json_bytes, md_bytes = json_path.read_bytes(), md_path.read_bytes()
            for option in ("--check", "--write", "--check"):
                result = cli(option)
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertEqual((json_path.read_bytes(), md_path.read_bytes()), (json_bytes, md_bytes))

            for path, original in ((json_path, json_bytes), (md_path, md_bytes)):
                path.write_bytes(original + b"\n")
                result = cli("--check")
                self.assertEqual(result.returncode, 1)
                self.assertIn("lệch bản sinh", result.stderr)
                path.write_bytes(original)

            sample = b"/" + b"/".join((b"home", b"fixture", b"client"))
            md_path.write_bytes(md_bytes + sample)
            result = cli("--check")
            self.assertEqual(result.returncode, 1)
            self.assertIn("đường dẫn", result.stderr)
            self.assertNotIn(sample.decode(), result.stderr)
            md_path.write_bytes(md_bytes)

            source = root / audit.FOUNDATION["schema_8"][0]
            original = source.read_bytes()
            source.write_bytes(original + b"\n")
            result = cli("--check")
            self.assertEqual(result.returncode, 1)
            self.assertIn("nguồn đã kiểm kê", result.stderr)

            source.write_bytes(original.replace(b"CREATE TABLE match_item_consumptions", b"CREATE TABLE other"))
            result = cli("--write")
            self.assertEqual(result.returncode, 1)
            self.assertIn("thiếu locator", result.stderr)
            self.assertEqual((json_path.read_bytes(), md_path.read_bytes()), (json_bytes, md_bytes))


if __name__ == "__main__":
    unittest.main()
