"""Kiểm đặc tả và lệnh F4-e bằng dữ liệu đã commit, không cần catalog ngoài repo."""

from copy import deepcopy
from ipaddress import IPv4Address, IPv6Address
import json
from pathlib import Path
from shutil import copyfile
import subprocess
import sys
import tempfile
import unittest

from tools import kr_f4_powerups_spec as spec


SCRIPT = "tools/kr_f4_powerups_spec.py"
INPUTS = (spec.CLASSIFICATION, spec.DIRECTIVE, spec.IDA)
OUTPUTS = (spec.JSON_OUTPUT, spec.MD_OUTPUT)


class CLITests(unittest.TestCase):
    def setUp(self):
        tmp = self.enterContext(tempfile.TemporaryDirectory())
        self.root = Path(tmp) / "checkout"
        for relative in (SCRIPT, *INPUTS, *OUTPUTS):
            dest = self.root / relative
            dest.parent.mkdir(parents=True, exist_ok=True)
            copyfile(spec.ROOT / relative, dest)

    def cli(self, option):
        # No .git metadata, raw evidence, sibling repositories or repo cwd.
        return subprocess.run(
            [sys.executable, str(self.root / SCRIPT), option],
            cwd=self.root.parent, capture_output=True, text=True, encoding="utf-8", timeout=10,
        )

    def test_check_and_write_agree_with_checked_in_artifacts(self):
        expected = {path: (self.root / path).read_bytes() for path in OUTPUTS}
        result = self.cli("--check")
        self.assertEqual(result.returncode, 0, result.stderr)
        for path in OUTPUTS:
            (self.root / path).unlink()
        result = self.cli("--write")
        self.assertEqual(result.returncode, 0, result.stderr)
        for path in OUTPUTS:
            self.assertEqual((self.root / path).read_bytes(), expected[path])

    def test_check_rejects_drift_without_rewriting_either_artifact(self):
        for relative in OUTPUTS:
            with self.subTest(relative=relative):
                path = self.root / relative
                original = path.read_bytes()
                changed = original + b"\n"
                path.write_bytes(changed)
                result = self.cli("--check")
                self.assertEqual(result.returncode, 1, result.stderr)
                self.assertIn("lệch nội dung sinh", result.stderr)
                self.assertEqual(path.read_bytes(), changed)
                path.write_bytes(original)

    def test_check_rejects_encoded_private_details_in_both_artifacts(self):
        address = str(IPv4Address(0x0A000105))
        fake_path = "/".join(("", "home", "tester", "fixture"))
        json_path = self.root / spec.JSON_OUTPUT
        original = json_path.read_text(encoding="utf-8")
        escaped = "".join(f"\\u{ord(char):04x}" for char in fake_path)
        json_path.write_text(original.replace('"task": "F4-e"', f'"task": "{escaped}"'), encoding="utf-8")
        result = self.cli("--check")
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertIn("rò rỉ: đường dẫn máy cá nhân", result.stderr)
        self.assertNotIn(fake_path, result.stderr)
        json_path.write_text(original, encoding="utf-8")

        md_path = self.root / spec.MD_OUTPUT
        with md_path.open("a", encoding="utf-8") as output:
            output.write(address.replace(".", "%2E") + "\n")
        result = self.cli("--check")
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertIn("rò rỉ: địa chỉ IP", result.stderr)
        self.assertNotIn(address, result.stderr)

    def test_write_rejects_leak_in_source_before_touching_outputs(self):
        path = self.root / spec.CLASSIFICATION
        data = json.loads(path.read_text(encoding="utf-8"))
        row = next(row for row in data["rows"] if "파워업 키트" in row["name"])
        dummy_value = "fixture" + "-value"
        row["reason"] += f" api_key={dummy_value}"
        path.write_text(json.dumps(data), encoding="utf-8")
        expected = {relative: (self.root / relative).read_bytes() for relative in OUTPUTS}
        result = self.cli("--write")
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertIn("rò rỉ: thông tin xác thực", result.stderr)
        self.assertNotIn(dummy_value, result.stderr)
        self.assertEqual({relative: (self.root / relative).read_bytes() for relative in OUTPUTS}, expected)


class PolicyTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.artifact = json.loads((spec.ROOT / spec.JSON_OUTPUT).read_text(encoding="utf-8"))

    def policy(self, system_id):
        return deepcopy(self.artifact["systems"][system_id]["policy"])

    def test_all_five_systems_separate_evidence_unknowns_and_design(self):
        self.assertEqual(self.artifact["schema"], "kre.f4-powerups-boosters.v1")
        self.assertEqual(set(self.artifact["systems"]), {
            "power_up_kits", "boosters_all_maps_pass", "upgrades_enhancement",
            "gifts_coupons", "second_password",
        })
        for name, entry in self.artifact["systems"].items():
            with self.subTest(name=name):
                spec.validate_policy(name, entry["policy"])
                self.assertEqual(entry["roadmap"]["status"], "candidate")
                self.assertEqual(entry["gaps"]["wire"]["status"], "unknown")
                self.assertEqual(entry["gaps"]["formula"]["status"], "unknown")
                for anchor in entry["static_anchors"]:
                    self.assertEqual(anchor["association_status"], "candidate")
                    self.assertEqual(anchor["wire_status"], "unknown")

    def test_rejects_policy_presented_as_measured_or_enabled(self):
        name = "power_up_kits"
        for key, value in (("source", "kr_static"), ("status", "confirmed"),
                           ("server_configurable", False), ("enabled_by_default", True)):
            with self.subTest(key=key):
                policy = self.policy(name)
                policy[key] = value
                with self.assertRaisesRegex(ValueError, "policy phải là design"):
                    spec.validate_policy(name, policy)

    def test_rejects_caps_billing_and_unbounded_failures(self):
        for name, key, bad in (
            ("power_up_kits", "max_equipped_slots", 3),
            ("power_up_kits", "max_equipped_slots", True),
            ("power_up_kits", "enforce_room_stat_caps", False),
            ("boosters_all_maps_pass", "paid_entitlements", True),
            ("boosters_all_maps_pass", "ranked_bonus", True),
            ("boosters_all_maps_pass", "combat_bonus", True),
            ("boosters_all_maps_pass", "enforce_room_map_pool", False),
            ("boosters_all_maps_pass", "default_reward_percent", {"exp": 150, "lucci": 200}),
            ("upgrades_enhancement", "allow_unconfigured_recipes", True),
            ("gifts_coupons", "nexon_real_money_billing", True),
            ("gifts_coupons", "paid_currency", True),
            ("gifts_coupons", "o1_transferable", True),
            ("second_password", "max_failed_attempts", 0),
            ("second_password", "max_failed_attempts", 6),
            ("second_password", "failure_window_seconds", 901),
            ("second_password", "cooldown_seconds", 0),
            ("second_password", "cooldown_seconds", 901),
        ):
            with self.subTest(name=name, key=key, bad=bad):
                policy = self.policy(name)
                policy["constraints"][key] = bad
                with self.assertRaisesRegex(ValueError, key):
                    spec.validate_policy(name, policy)


class LeakTests(unittest.TestCase):
    def test_rejects_private_ip_host_and_credential_patterns(self):
        for value in (
            str(IPv4Address(0xC0A80105)),
            f"[{IPv6Address((0xFD << 120) + 1)}]",
            "https://fixture." + "internal",
            "-----BEGIN " + "PRIVATE KEY-----",
            "Authorization: Bearer " + "fixture",
            "https://user:fixture" + "@example.org",
        ):
            with self.subTest(value=value):
                with self.assertRaisesRegex(ValueError, "rò rỉ"):
                    spec.reject_leaks({"nested": [value]})
        spec.reject_leaks("https://example.org/docs; CStage 418 @ 0xd6af22")


if __name__ == "__main__":
    unittest.main()
