"""Kiểm định chính sách vận hành F6 cho KR 00967, hoàn toàn offline."""

import argparse
from datetime import datetime
import json
from pathlib import Path
import re
import sys


ROOT = Path(__file__).resolve().parents[1]
CATALOG = ROOT / "docs/evidence/content/f6-events-catalog.json"
PROFILE_SCHEMA = "kre.f6-events-policy.v1"
SYSTEM_COUNT = 19
EXCLUDED = "PCRoomBenefit"
UTC_TIMESTAMP = re.compile(r"[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z")
RATE_CAP_MAX = {"per_account_per_hour": 60, "global_per_minute": 600}


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"khóa JSON trùng: {key!r}")
        result[key] = value
    return result


def reject_constant(value):
    raise ValueError(f"hằng JSON không hợp lệ: {value}")


def read_json(path):
    try:
        return json.loads(path.read_text(encoding="utf-8"), object_pairs_hook=unique_object,
                          parse_constant=reject_constant)
    except (OSError, ValueError, RecursionError) as exc:
        raise ValueError(f"{path.name}: {exc}") from exc


def catalog_systems(catalog):
    if (not isinstance(catalog, dict) or catalog.get("schema") != "kre.f6-events-catalog.v1"
            or catalog.get("build") != "KR00967" or not isinstance(catalog.get("systems"), list)):
        raise ValueError("catalog F6: cần schema v1, build KR00967 và danh sách systems")
    scopes = {}
    for row in catalog["systems"]:
        if not isinstance(row, dict):
            raise ValueError("catalog F6: mỗi hệ thống phải là object JSON")
        name, scope = row.get("system"), row.get("scope")
        if not isinstance(name, str) or not name or name in scopes:
            raise ValueError("catalog F6: tên hệ thống thiếu, không hợp lệ hoặc bị trùng")
        if scope not in ("in_scope", "related_candidate", "excluded_policy"):
            raise ValueError(f"catalog F6: scope không hợp lệ cho {name!r}")
        scopes[name] = scope
    if scopes.get(EXCLUDED) != "excluded_policy":
        raise ValueError(f"catalog F6: {EXCLUDED} phải mang scope excluded_policy")
    names = frozenset(name for name, scope in scopes.items() if scope == "in_scope")
    if len(names) != SYSTEM_COUNT:
        raise ValueError(f"catalog F6: cần đúng {SYSTEM_COUNT} hệ thống in_scope")
    return names


def require_fields(value, expected, label):
    if not isinstance(value, dict):
        raise ValueError(f"{label}: cần object JSON")
    if EXCLUDED in value:
        raise ValueError(f"{label}: {EXCLUDED} bị cấm theo excluded_policy")
    extra = value.keys() - set(expected)
    missing = set(expected) - value.keys()
    if extra or missing:
        raise ValueError(f"{label}: khóa lạ {sorted(extra)}, thiếu khóa {sorted(missing)}")


def utc_timestamp(value, label):
    if isinstance(value, str) and UTC_TIMESTAMP.fullmatch(value):
        try:
            return datetime.fromisoformat(value[:-1] + "+00:00")
        except ValueError:
            pass
    raise ValueError(f"{label}: cần ngày giờ UTC hợp lệ dạng YYYY-MM-DDTHH:MM:SSZ")


def validate_policy(profile, catalog):
    names = catalog_systems(catalog)
    require_fields(profile, ("schema", "default_enabled", "systems"), "profile")
    if profile["schema"] != PROFILE_SCHEMA:
        raise ValueError(f"schema: cần {PROFILE_SCHEMA}")
    if profile["default_enabled"] is not False:
        raise ValueError("default_enabled: phải là false")

    systems = profile["systems"]
    require_fields(systems, names, "systems")
    for name in sorted(names):
        entry = systems[name]
        label = f"systems.{name}"
        require_fields(entry, ("enabled", "start_utc", "end_utc", "rate_caps"), label)
        enabled = entry["enabled"]
        if type(enabled) is not bool:
            raise ValueError(f"{label}.enabled: cần true hoặc false")

        start, end = entry["start_utc"], entry["end_utc"]
        if enabled or start is not None or end is not None:
            start_time = utc_timestamp(start, f"{label}.start_utc")
            end_time = utc_timestamp(end, f"{label}.end_utc")
            if start_time >= end_time:
                raise ValueError(f"{label}: cần start_utc < end_utc")

        caps = entry["rate_caps"]
        require_fields(caps, RATE_CAP_MAX, f"{label}.rate_caps")
        for key, maximum in RATE_CAP_MAX.items():
            cap = caps[key]
            if type(cap) is not int or not 0 <= cap <= maximum:
                raise ValueError(f"{label}.rate_caps.{key}: cần số nguyên trong khoảng 0..{maximum}")


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", required=True, type=Path, metavar="PROFILE.json",
                        help="đọc và kiểm định profile F6")
    args = parser.parse_args(argv)
    try:
        validate_policy(read_json(args.check), read_json(CATALOG))
    except ValueError as exc:
        print(f"F6: {exc}", file=sys.stderr)
        return 1
    print(f"F6: profile hợp lệ ({SYSTEM_COUNT} hệ thống)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
