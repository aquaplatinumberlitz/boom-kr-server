"""Kiểm định chính sách vận hành F6 cho KR 00967, hoàn toàn offline."""

import argparse
from datetime import datetime
import json
from pathlib import Path
import re
import sys


ROOT = Path(__file__).resolve().parents[1]
CATALOG = ROOT / "docs/evidence/content/f6-events-catalog.json"
EXTENDED_SPEC = ROOT / "docs/evidence/content/f6-extended-events-spec.json"
DISABLED_PROFILE = ROOT / "server/profiles/f6-events-disabled.json"
RUST_POLICY = ROOT / "rust/crates/kr-service/src/f6_policy.rs"
PROFILE_SCHEMA = "kre.f6-events-policy.v1"
SYSTEM_COUNT = 19
EXCLUDED = "PCRoomBenefit"
UTC_TIMESTAMP = re.compile(r"[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z")
RATE_CAP_MAX = {"per_account_per_hour": 60, "global_per_minute": 600}
MAP_FAMILY_NAMES = frozenset({
    "CAPassShopMap", "DailyMissionShop", "LobbySquareFishingTest", "MagicMarketmap", "SelectionBox",
})
RUST_ALLOWLIST = re.compile(
    r"^[ \t]*const SYSTEM_NAMES\s*:\s*\[\s*&str\s*;\s*([0-9]+)\s*\]"
    r"\s*=\s*\[([^\]]*)\]\s*;", re.MULTILINE,
)
RUST_NAME = r'"([^"\\\r\n]+)"'


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"khóa JSON trùng: {key!r}")
        result[key] = value
    return result


def reject_constant(value):
    raise ValueError(f"hằng JSON không hợp lệ: {value}")


def read_text(path):
    try:
        if any(part.is_symlink() for part in (path, *path.absolute().parents)):
            raise ValueError("không chấp nhận symlink")
        return path.read_text(encoding="utf-8")
    except (OSError, ValueError) as exc:
        raise ValueError(f"{path.name}: {exc}") from exc


def read_json(path):
    raw = read_text(path)
    try:
        return json.loads(raw, object_pairs_hook=unique_object,
                          parse_constant=reject_constant)
    except (ValueError, RecursionError) as exc:
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
        if name in MAP_FAMILY_NAMES and enabled:
            raise ValueError(f"{label}: họ map F6? phải bị vô hiệu hóa trong v1")

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


def require_same_names(actual, expected, label):
    if actual != expected:
        raise ValueError(f"{label}: tập in_scope không khớp catalog F6; "
                         f"dư {sorted(actual - expected)}, thiếu {sorted(expected - actual)}")


def rust_system_names(source):
    # Chỉ đọc khai báo mảng literal hiện tại; cú pháp khác cần được đối chiếu lại.
    matches = RUST_ALLOWLIST.findall(source)
    if len(matches) != 1:
        raise ValueError("f6_policy.rs: cần đúng một khai báo SYSTEM_NAMES")
    size, body = matches[0]
    if not re.fullmatch(rf"\s*{RUST_NAME}(?:\s*,\s*{RUST_NAME})*\s*,?\s*", body):
        raise ValueError("f6_policy.rs: SYSTEM_NAMES phải là mảng tên literal")
    entries = re.findall(RUST_NAME, body)
    names = set(entries)
    if EXCLUDED in names:
        raise ValueError(f"f6_policy.rs: SYSTEM_NAMES không được chứa {EXCLUDED}")
    if size != str(SYSTEM_COUNT) or len(names) != len(entries) or len(names) != SYSTEM_COUNT:
        raise ValueError(f"f6_policy.rs: SYSTEM_NAMES cần đúng {SYSTEM_COUNT} tên khác nhau")
    return names


def spec_rows(spec, section, count, *, key="name", **expected):
    rows = spec.get(section)
    label = f"đặc tả F6.{section}"
    if not isinstance(rows, list) or len(rows) != count:
        raise ValueError(f"{label}: cần đúng {count} mục")
    result = {}
    for row in rows:
        name = row.get(key) if isinstance(row, dict) else None
        if not isinstance(name, str) or not name.strip() or name in result:
            raise ValueError(f"{label}: {key} thiếu, không hợp lệ hoặc bị trùng")
        for field, value in expected.items():
            actual = row.get(field)
            if (actual is not value if isinstance(value, bool) else actual != value):
                raise ValueError(f"{label}.{name}: {field} phải là {value!r}")
        result[name] = row
    return result


def validate_extended_spec(spec, rust_source, names):
    if (not isinstance(spec, dict) or spec.get("schema") != "kre.f6-extended-events-spec.v1"
            or spec.get("build") != "KR00967"):
        raise ValueError("đặc tả F6: cần schema v1 và build KR00967")

    excluded = spec_rows(spec, "excluded_policy", 1, scope="excluded_policy", kind="system",
                         enabled=False, enableable=False)
    if excluded.keys() != {EXCLUDED}:
        raise ValueError(f"đặc tả F6.excluded_policy: cần đúng mục {EXCLUDED}")
    scoped = spec_rows(spec, "in_scope", SYSTEM_COUNT, scope="in_scope", kind="system", enabled=False)
    roadmap = spec_rows(spec, "roadmap_only", 25, key="id", scope="roadmap_only", enabled=False)
    groups = spec_rows(spec, "roadmap_classification", 7, kind="system", roadmap_only=True)
    maps = spec_rows(spec, "map_families", 5, kind="map", wave="F6?")
    summary = spec.get("summary")
    for section, rows in (("excluded_policy", excluded), ("in_scope", scoped), ("roadmap_only", roadmap),
                          ("roadmap_classification_systems", groups), ("map_families", maps)):
        count = summary.get(section) if isinstance(summary, dict) else None
        if type(count) is not int or count != len(rows):
            raise ValueError(f"đặc tả F6.summary.{section}: không khớp danh sách")
    if not isinstance(summary, dict) or summary.get("maps") != len(maps):
        raise ValueError("đặc tả F6.summary.maps: không khớp danh sách")

    roadmap_names = set()
    for identifier, row in roadmap.items():
        name = row.get("name")
        if not isinstance(name, str) or not name.strip() or name in roadmap_names:
            raise ValueError(f"đặc tả F6.roadmap_only.{identifier}: name thiếu, không hợp lệ hoặc bị trùng")
        roadmap_names.add(name)

    reserved = roadmap.keys() | roadmap_names | groups.keys()
    if any(name.casefold() == EXCLUDED.casefold() for name in reserved | scoped.keys() | maps.keys()):
        raise ValueError(f"đặc tả F6: {EXCLUDED} chỉ được xuất hiện trong excluded_policy")

    map_refs = set()
    for identifier, row in roadmap.items():
        ref = row.get("classification_ref")
        ref_name = ref.get("name") if isinstance(ref, dict) else None
        if isinstance(ref_name, str):
            if ref.get("kind") == "map" and ref_name in maps:
                map_refs.add(ref_name)
                continue
            if ref.get("kind") == "system" and (ref_name in scoped or ref_name in groups):
                continue
        raise ValueError(f"đặc tả F6.roadmap_only.{identifier}: classification_ref không hợp lệ")
    if map_refs != maps.keys():
        raise ValueError("đặc tả F6.map_families: mọi họ map F6? phải có mục roadmap_only")
    for name, row in (*roadmap.items(), *groups.items(), *maps.items()):
        if row.get("enabled", False) is not False or row.get("enableable", False) is not False:
            raise ValueError(f"đặc tả F6: {name} phải giữ tắt trong v1")

    # SelectionBox là tên hệ thống hợp lệ và cũng là tên họ map riêng.
    blocked = reserved | (maps.keys() - {"SelectionBox"})
    overlap = blocked & (scoped.keys() | names)
    if overlap:
        raise ValueError(f"đặc tả F6: mục roadmap_only hoặc họ map F6? vào in_scope: {sorted(overlap)}")
    require_same_names(scoped.keys(), names, "đặc tả F6.in_scope")
    require_same_names(rust_system_names(rust_source), names, "f6_policy.rs SYSTEM_NAMES")


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", type=Path, metavar="PROFILE.json",
                        help="đọc và kiểm định profile F6")
    parser.add_argument("--check-extended", action="store_true",
                        help="đối chiếu thêm đặc tả F6 mở rộng, profile mặc định và Rust")
    parser.add_argument("profile", nargs="?", type=Path, metavar="PROFILE.json",
                        help="profile F6 cần kiểm định, tương đương --check")
    args = parser.parse_args(argv)
    if args.check is None and args.profile is None:
        parser.error("the following arguments are required: --check")
    if args.check is not None and args.profile is not None:
        parser.error("chỉ định một profile F6 qua --check hoặc đối số vị trí")
    try:
        profile = read_json(args.check or args.profile)
        catalog = read_json(CATALOG)
        validate_policy(profile, catalog)
        if args.check_extended:
            spec = read_json(EXTENDED_SPEC)
            rust_source = read_text(RUST_POLICY)
            catalog_names = catalog_systems(catalog)
            validate_extended_spec(spec, rust_source, catalog_names)
            spec_names = {row["name"] for row in spec["in_scope"]}
            require_same_names(set(profile["systems"]), spec_names, "profile và đặc tả F6")
            require_same_names(rust_system_names(rust_source), spec_names, "Rust và đặc tả F6")
            map_names = {row["name"] for row in spec["map_families"]}
            if set(profile["systems"]) & (map_names - {"SelectionBox"}):
                raise ValueError("profile: họ map F6? không được xuất hiện trong systems")
            validate_policy(read_json(DISABLED_PROFILE), catalog)
    except ValueError as exc:
        print(f"F6: {exc}", file=sys.stderr)
        return 1
    print(f"F6: profile hợp lệ ({SYSTEM_COUNT} hệ thống)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
