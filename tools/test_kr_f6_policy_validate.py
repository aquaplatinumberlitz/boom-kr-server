"""Kiểm profile mặc định, giới hạn schema và mã thoát của công cụ F6-1."""

from contextlib import redirect_stderr, redirect_stdout
from copy import deepcopy
import io
import json
from pathlib import Path
import tempfile
import unittest

from tools.kr_f6_policy_validate import CATALOG, ROOT, main, validate_policy


PROFILE = ROOT / "server/profiles/f6-events-disabled.json"
START = "2027-01-01T00:00:00Z"
END = "2027-01-08T00:00:00Z"


class PolicyTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.catalog = json.loads(CATALOG.read_text(encoding="utf-8"))
        cls.baseline = json.loads(PROFILE.read_text(encoding="utf-8"))

    def setUp(self):
        self.profile = deepcopy(self.baseline)
        self.systems = self.profile["systems"]
        self.entry = self.systems["CAPass"]

    def validate(self):
        validate_policy(self.profile, self.catalog)

    def assert_rejected(self, message):
        with self.assertRaisesRegex(ValueError, message):
            self.validate()

    def test_default_profile_is_complete_and_off(self):
        self.validate()
        self.assertIs(self.profile["default_enabled"], False)
        expected = {row["system"] for row in self.catalog["systems"] if row["scope"] == "in_scope"}
        self.assertEqual(len(expected), 19)
        self.assertEqual(self.systems.keys(), expected)
        for name, entry in self.systems.items():
            with self.subTest(system=name):
                self.assertIs(entry["enabled"], False)
                self.assertIsNone(entry["start_utc"])
                self.assertIsNone(entry["end_utc"])
                self.assertEqual(entry["rate_caps"], {"per_account_per_hour": 0, "global_per_minute": 0})

    def test_explicit_opt_in_with_bounded_schedule_and_caps(self):
        self.entry.update(enabled=True, start_utc=START, end_utc=END)
        self.entry["rate_caps"] = {"per_account_per_hour": 60, "global_per_minute": 600}
        self.validate()
        self.entry["enabled"] = False
        self.validate()

    def test_rejects_default_on_and_non_boolean_flags(self):
        for value in (True, 0, "false", None):
            with self.subTest(default_enabled=value):
                self.profile["default_enabled"] = value
                self.assert_rejected("default_enabled")
        self.profile["default_enabled"] = False
        self.entry["enabled"] = 1
        self.assert_rejected(r"CAPass\.enabled")

    def test_rejects_pc_bang_even_when_off(self):
        for enabled in (False, True):
            with self.subTest(enabled=enabled):
                self.systems["PCRoomBenefit"] = {**self.entry, "enabled": enabled}
                self.assert_rejected("PCRoomBenefit.*excluded_policy")

    def test_catalog_cannot_reclassify_pc_bang(self):
        catalog = deepcopy(self.catalog)
        next(row for row in catalog["systems"] if row["system"] == "PCRoomBenefit")["scope"] = "in_scope"
        next(row for row in catalog["systems"] if row["system"] == "CAPass")["scope"] = "related_candidate"
        del self.systems["CAPass"]
        self.systems["PCRoomBenefit"] = self.entry
        with self.assertRaisesRegex(ValueError, "PCRoomBenefit.*excluded_policy"):
            validate_policy(self.profile, catalog)

    def test_rejects_reversed_and_empty_windows_even_when_off(self):
        for end in (START, "2026-12-31T23:59:59Z"):
            with self.subTest(end=end):
                self.entry.update(start_utc=START, end_utc=end)
                self.assert_rejected("start_utc < end_utc")

    def test_rejects_missing_partial_and_noncanonical_windows(self):
        self.entry["enabled"] = True
        self.assert_rejected("start_utc")
        self.entry.update(enabled=False, start_utc=START)
        self.assert_rejected("end_utc")
        for start in ("2027-02-29T00:00:00Z", "2027-01-01T00:00:00+09:00", "2027-01-01", 123):
            with self.subTest(start=start):
                self.entry.update(start_utc=start, end_utc=END)
                self.assert_rejected("start_utc")

    def test_rejects_out_of_range_and_non_integer_caps(self):
        caps = self.entry["rate_caps"]
        for key, maximum in (("per_account_per_hour", 60), ("global_per_minute", 600)):
            for value in (-1, maximum + 1, True, 1.0, "1"):
                with self.subTest(cap=key, value=value):
                    caps[key] = value
                    self.assert_rejected(rf"rate_caps\.{key}")
            caps[key] = 0

    def test_rejects_missing_system_and_required_fields(self):
        for obj, key in ((self.systems, "Nhiệm vụ"), (self.profile, "default_enabled"),
                         (self.entry, "start_utc"), (self.entry["rate_caps"], "global_per_minute")):
            with self.subTest(key=key):
                value = obj.pop(key)
                self.assert_rejected(f"thiếu khóa.*{key}")
                obj[key] = value

    def test_rejects_extra_keys_at_every_level(self):
        for obj, key in ((self.profile, "force"), (self.systems, "Antarctic"),
                         (self.entry, "opcode"), (self.entry["rate_caps"], "per_second")):
            with self.subTest(key=key):
                obj[key] = 0
                self.assert_rejected(f"khóa lạ.*{key}")
                del obj[key]

    def test_rejects_unknown_schema_and_non_object_caps(self):
        self.profile["schema"] = "kre.f6-events-policy.v2"
        self.assert_rejected("schema")
        self.profile["schema"] = self.baseline["schema"]
        self.entry["rate_caps"] = []
        self.assert_rejected(r"CAPass\.rate_caps: cần object JSON")

    def test_cli_returns_failure_for_invalid_json(self):
        raw = PROFILE.read_text(encoding="utf-8")
        cases = (
            (raw.replace('"default_enabled": false', '"default_enabled": true, "default_enabled": false'),
             "khóa JSON trùng"),
            (raw.replace('"per_account_per_hour": 0', '"per_account_per_hour": NaN', 1),
             "hằng JSON không hợp lệ"),
            (raw.rstrip()[:-1], "Expecting"),
        )
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "profile.json"
            for data, message in cases:
                with self.subTest(error=message):
                    path.write_text(data, encoding="utf-8")
                    with redirect_stdout(io.StringIO()) as stdout, redirect_stderr(io.StringIO()) as stderr:
                        self.assertEqual(main(["--check", str(path)]), 1)
                    self.assertEqual(stdout.getvalue(), "")
                    self.assertIn(message, stderr.getvalue())


if __name__ == "__main__":
    unittest.main()
