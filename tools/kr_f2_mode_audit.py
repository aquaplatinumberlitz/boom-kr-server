"""Kiểm kê F2 từ metadata đã lưu của KR 00967; không đọc pack hay gọi mạng."""

import argparse
from collections import Counter, defaultdict
import hashlib
import json
from pathlib import Path
import re
import sys

from content_census import CENSUS, MISSION, MODE_ROWS, ROOT, spans


CLASSIFICATION = ROOT / "docs/content/classification.json"
CATALOG = ROOT / "docs/evidence/content/f2-mode-catalog.json"
MARKDOWN = ROOT / "docs/content/f2-mode-catalog.md"
RULES_LOCATOR = "rust/crates/kr-service/src/room_modes/rules.json"
RULES = ROOT / RULES_LOCATOR
EXPECTED_MAPS = 122
EXPECTED_FAMILIES = 18
MODE_GROUPS = {
    "협공배틀 (đánh phối hợp)": "co_op",
    "부활대전 (hồi sinh)": "revive",
    "고슴도치 (nhím)": "hedgehog",
    "대장잡기 (bắt thủ lĩnh)": "leader_capture",
    "시한폭탄 (bom hẹn giờ)": "timebomb",
    "팝케이드 (Popcade)": "popcade",
    "눈싸움 (đấu ném tuyết; sự kiện cũ)": "snowball_event",
    "슈퍼파이터 (Super Fighter; sự kiện cũ)": "super_fighter",
    "래더 (Ladder)": "ladder",
    "Ghép trận UXMatching": "matchmaking",
    "Free 1v7": "free_1v7",
}
# Các nhóm RoomRules chỉ có hàng họ nhiệm vụ, chưa có hàng riêng trong bảng Mode.
FAMILY_ONLY_GROUPS = {
    "CAPong": "capong",
    "Clover": "clover",
    "Hamster": "farm_mission",
    "Rabbit": "farm_mission",
    "RescueET": "rescue_et",
    "Wriggle": "wriggle",
    "Zombie": "zombie",
    "Zzolman": "zzolman",
}
PRIVATE = re.compile(
    r"/home/|/Users/|ocid" r"1\.|BEGIN [A-Z0-9 ]*PRIVATE" r" KEY|"
    r"(?<![\d.])(?:10\.\d{1,3}|172\.(?:1[6-9]|2\d|3[01])|192\.168|198\.(?:1[89]))"
    r"(?:\.\d{1,3}){2}(?![\d.])"
)


def canonical(value):
    return (json.dumps(value, ensure_ascii=False, sort_keys=True,
                       separators=(",", ":"), allow_nan=False) + "\n").encode("utf-8")


def sha256(raw):
    return hashlib.sha256(raw).hexdigest()


def reject_private(text):
    if PRIVATE.search(text):
        raise ValueError("metadata chứa đường dẫn máy, địa chỉ nội bộ hoặc định danh riêng")


def classification_index(data):
    index = {}
    for row in (data or {}).get("rows", []):
        if row["kind"] not in {"map", "mode", "system"}:
            continue
        key = (row["kind"], row["name"])
        if key in index:
            raise ValueError("trùng khóa xếp loại theo loại và tên")
        index[key] = row
    return index


def grade(row):
    if row is None:
        return None
    return {key: row[key] for key in ("classification", "wave", "status") if key in row}


def slug(name):
    return re.sub(r"(?<=[a-z])(?=[A-Z])", "_", name).lower()


def drop_counts(maps):
    return {
        "maps_with_rows": sum(bool(row["drops"]) for row in maps),
        "rows": sum(len(row["drops"]) for row in maps),
        "positive_rows": sum(weight > 0 for row in maps for _, weight in row["drops"]),
        "distinct_positive_names": len({name for row in maps for name, weight in row["drops"] if weight > 0}),
    }


def placement_counts(maps):
    names = Counter(name for row in maps for name in row["placements"])
    return {"maps_with_rows": sum(bool(row["placements"]) for row in maps),
            "rows": sum(names.values()), "by_name": dict(names)}


def partial_tail_diagnostic(row):
    reason = row.get("partial_reason")
    missing_tags = row.get("missing_tags")
    tail = row.get("tail")
    if reason is not None and not isinstance(reason, str):
        raise ValueError("partial_reason phải là chuỗi hoặc null")
    if missing_tags is not None and (not isinstance(missing_tags, list)
                                     or any(not isinstance(tag, str) for tag in missing_tags)):
        raise ValueError("missing_tags phải là danh sách tag hoặc null")
    if tail is not None and (
            not isinstance(tail, dict) or tail.keys() - {"offset", "remaining_bytes"}
            or any(type(value) is not int or value < 0 for value in tail.values())):
        raise ValueError("tail chỉ nhận offset và remaining_bytes không âm")
    return {
        "map": row["name"], "reason": reason, "missing_tags": missing_tags, "tail": tail,
        "observed": {
            "drop_rows": len(row["drops"]), "placement_rows": len(row["placements"]),
            **{key: row.get(key) for key in ("script_tags", "script_funcs", "script_triggers")},
        },
    }


def family_summary(name, maps, mode_refs, classification):
    exact = [row for row in maps if row["parse"] == "exact"]
    partial = sorted((row for row in maps if row["parse"] == "partial"), key=lambda row: row["name"])
    if len(exact) + len(partial) != len(maps):
        raise ValueError("trạng thái parse nằm ngoài exact/partial")
    if classification and classification.get("map_count", len(maps)) != len(maps):
        raise ValueError("số map theo họ không khớp với bảng xếp loại")
    wiki = sorted({row.get("wiki") or MISSION.get(name, "") for row in maps} - {""})
    group = FAMILY_ONLY_GROUPS.get(name)
    if group is None and len(mode_refs) == 1:
        group = mode_refs[0]["ruleset_group"]

    return {
        "family": name, "map_count": len(maps),
        "maps": [{key: row[key] for key in ("name", "code", "map_code") if key in row}
                 for row in sorted(maps, key=lambda row: row["name"])],
        "grid_sizes": sorted({(row["width"], row["height"]) for row in maps}),
        "spawn_range": [min(row["spawns"] for row in maps), max(row["spawns"] for row in maps)],
        "d1_d2_pairs": sorted({(row["d1"], row["d2"]) for row in maps}),
        "has_script": {"yes": sum(row["scripted"] for row in exact),
                       "no": sum(not row["scripted"] for row in exact), "unknown": len(partial)},
        "partial_maps": [row["name"] for row in partial],
        **({"partial_tail_diagnostics": [partial_tail_diagnostic(row) for row in partial]} if partial else {}),
        "drops": {"exact": drop_counts(exact), "partial_observed": drop_counts(partial)},
        "placements": {"exact": placement_counts(exact), "partial_observed": placement_counts(partial)},
        "classification": grade(classification),
        "grouping": {"wiki_labels": wiki, "mode_refs": sorted(row["mode"] for row in mode_refs),
                     "ruleset_group": group or slug(name),
                     "basis": "wiki_inferred" if wiki else "name_inferred", "status": "candidate"},
    }


def mode_summaries(census, by_family, mission, grades):
    resources = {f"{r['pack']}/{r['name']}": r for r in census.get("resource_groups", [])}
    systems = {r["name"]: r for r in census.get("systems", [])}
    declarations = {r["name"]: r["maps"] for r in census.get("map_groups", [])}
    maps = defaultdict(list)
    for row in census["maps"]:
        maps[row["name"]].append(row)

    def resource(key):
        row = resources.get(key, {})
        system = row.get("system") or None
        system_row = systems.get(system, {})
        return {"locator": key, "entries": row.get("entries"), "system": system,
                "system_ids": sorted(system_row.get("ids", [])),
                "system_classification": grade(grades.get(("system", system)))}

    def map_group(key):
        entries = declarations.get(key)
        # Preserve declaration order and syntax. Equality is only a census lookup.
        matches = (None if entries is None else [
            {"entry": entry, "category": maps[entry][0]["category"], "family": maps[entry][0]["family"]}
            for entry in entries if len(maps[entry]) == 1
        ])
        return {"locator": f"FxContents/Attr/MapGroup/{key}", "entries": None if entries is None else list(entries),
                "exact_census_matches": matches}

    rows = []
    for row in [*MODE_ROWS, {"name": "Free 1v7", "wave": "F2"}]:
        name = row["name"]
        families = set(row.get("families", []))
        families.update(family for family in by_family
                        if any(family.startswith(prefix) for prefix in row.get("family_prefixes", [])))
        if not (row["wave"] in {"F2", "F2?"} or families & mission.keys()):
            continue
        map_groups = row.get("map_groups", [])
        # Use the same fallback locators as content_census.render_modes.
        groups = row.get("groups", [])
        if not groups and (row.get("families") or map_groups):
            groups = ["FxMap/Map" if row.get("families") else "FxContents/Attr:MapGroup"]
        wiki = row.get("wiki")
        rows.append({
            "mode": name, "wave": row["wave"], "wire_mode": None,
            "ruleset_group": MODE_GROUPS.get(name, slug(name)),
            "grouping_basis": "wiki_inferred" if wiki else "roadmap" if name == "Free 1v7" else "name_inferred",
            "grouping_status": "candidate", "wiki": wiki,
            "classification": grade(grades.get(("mode", name))),
            "pack": {
                "resource_groups": [resource(key) for key in sorted(groups)],
                "detail_from_mode_table": row.get("detail"),
                "map_groups": [map_group(key) for key in sorted(map_groups)],
            },
            "related_families": [
                {"family": family, "map_count": len(by_family[family]),
                 "mission_map_count": len(mission.get(family, []))}
                for family in sorted(families)
            ],
        })
    return sorted(rows, key=lambda row: row["mode"])


def validate_rule_group_parity(catalog, rules):
    expected = {row["group"]: row["wave"] for row in rules}
    modes = catalog["modes"]
    mode_groups = {row["ruleset_group"]: row["wave"] for row in modes}
    families = {row["family"]: row for row in catalog["families"]}
    family_groups = {FAMILY_ONLY_GROUPS[name]: "F2?" for name in families if name in FAMILY_ONLY_GROUPS}
    if (not expected or len(expected) != len(rules) or len(mode_groups) != len(modes)
            or mode_groups.keys() & family_groups.keys() or expected != (mode_groups | family_groups)):
        raise ValueError("tập nhóm luật hoặc đợt trong catalog không khớp bảng F2")
    if len({row["mode"] for row in modes}) != len(modes):
        raise ValueError("trùng tên mode trong catalog F2")

    if len(families) != len(catalog["families"]):
        raise ValueError("trùng tên họ map trong catalog F2")
    links = defaultdict(dict)
    for mode in modes:
        for ref in mode["related_families"]:
            family = ref["family"]
            mission_count = families[family]["map_count"] if family in families else 0
            if ref["mission_map_count"] != mission_count:
                raise ValueError(f"số map mission của liên kết {family} không khớp")
            if mission_count:
                if mode["mode"] in links[family]:
                    raise ValueError(f"trùng liên kết mode với họ {family}")
                links[family][mode["mode"]] = mode["ruleset_group"]
    for family, row in families.items():
        grouping = row["grouping"]
        refs = links.get(family, {})
        group = grouping["ruleset_group"]
        if family in FAMILY_ONLY_GROUPS:
            group_matches = not refs and group == FAMILY_ONLY_GROUPS[family]
        elif refs:
            group_matches = set(refs.values()) == {group}
        else:
            group_matches = group not in expected
        if sorted(grouping["mode_refs"]) != sorted(refs) or not group_matches:
            raise ValueError(f"nhóm luật hoặc liên kết mode của họ {family} không khớp")
    return {"source": "design", "status": "design", "locator": RULES_LOCATOR,
            "mode_count": len(mode_groups), "linked_mission_families": len(links)}


def build_catalog(census, classification=None, *, expected_maps=EXPECTED_MAPS,
                  expected_families=EXPECTED_FAMILIES, rules=None):
    by_family, mission = defaultdict(list), defaultdict(list)
    map_names = Counter(row["name"].casefold() for row in census["maps"])
    for row in census["maps"]:
        by_family[row["family"]].append(row)
        if row["category"] == "mission":
            if map_names[row["name"].casefold()] != 1:
                raise ValueError("trùng tên map trong phạm vi mission (không phân biệt hoa thường)")
            mission[row["family"]].append(row)

    total_maps = sum(map(len, mission.values()))
    if total_maps != expected_maps or len(mission) != expected_families:
        raise ValueError(f"cần {expected_maps} map / {expected_families} họ; "
                         f"nhận {total_maps} map / {len(mission)} họ")
    grades = classification_index(classification)
    modes = mode_summaries(census, by_family, mission, grades)
    families = [family_summary(name, maps,
                               [row for row in modes if any(link["family"] == name
                                                             for link in row["related_families"])],
                               grades.get(("map", name)))
                for name, maps in sorted(mission.items())]
    counts = Counter((row["classification"] or {}).get("classification", "unknown") for row in families)
    if counts.keys() - {"A", "B", "C", "unknown"}:
        raise ValueError("lớp của họ mission nằm ngoài A/B/C")
    catalog = {
        "schema": "kre.f2-mode-catalog.v1", "build": census["build"],
        "static_metadata": {"source": "kr_static", "status": "confirmed",
                            "locator": "docs/content/census.json: maps[category=mission], map_groups, resource_groups",
                            "client_exe_sha256": census.get("client_exe_sha256"),
                            "pack_hashes": {
                                name: {field: value for field, value in census["packs"][name].items()
                                       if field in {"idx_sha256", "idd_sha256"}}
                                for name in ("FxMap", "FxContents", "FxStage", "CharTE")
                                if name in census.get("packs", {})
                            }},
        "summary": {
            "total_maps": total_maps, "family_count": len(families), "duplicate_map_entries": 0,
            "classification_counts": {key: counts[key] for key in ("A", "B", "C", "unknown")},
            "has_script": {key: sum(row["has_script"][key] for row in families)
                           for key in ("yes", "no", "unknown")},
        },
        "families": families, "modes": modes,
    }
    if rules is None:
        rules, _ = read_json(RULES)
    catalog["rule_group_parity"] = validate_rule_group_parity(catalog, rules)
    raw = canonical(catalog)
    reject_private(raw.decode("utf-8"))
    return {"catalog_sha256": sha256(raw), **catalog}


def render_json(catalog):
    # One family or mode per line; keep headers easy to scan.
    lines = ["{"]
    for key, value in sorted(catalog.items()):
        if key in {"families", "modes"}:
            lines.append(f'  "{key}": [')
            lines += ["    " + canonical(row).decode().rstrip() + "," for row in value]
            if value:
                lines[-1] = lines[-1].removesuffix(",")
            lines.append("  ],")
        else:
            lines.append(f'  "{key}": {canonical(value).decode().rstrip()},')
    lines[-1] = lines[-1].removesuffix(",")
    return "\n".join([*lines, "}", ""])


def render_markdown(catalog):
    summary = catalog["summary"]
    scripts = summary["has_script"]
    counts = summary["classification_counts"]
    lines = [
        "# Kiểm kê mode và map nhiệm vụ F2",
        "",
        "Nguồn: [kiểm kê map và pack](../CONTENT.md), [bảng Mode](../CONTENT.md#mode), "
        "và [xếp loại A/B/C](classification.md).",
        "[Catalog JSON](../evidence/content/f2-mode-catalog.json) giữ tên từng map, số đếm, "
        "locator tài nguyên và hash đầu vào.",
        f"Kết quả cho {catalog['build']}: **{summary['total_maps']} map / {summary['family_count']} họ**, "
        "không trùng tên map.",
        f"Census nhận diện script ở **{scripts['yes']}** map, không thấy ở **{scripts['no']}** map đọc trọn "
        f"và chưa rõ ở **{scripts['unknown']}** map đọc một phần.",
        f"Xếp loại sơ bộ ở cấp họ: {counts['A']} A, {counts['B']} B, {counts['C']} C, "
        f"{counts['unknown']} chưa có xếp loại.",
        "",
        "## Metadata tĩnh theo họ",
        "",
        "Phạm vi là các hàng `category=mission` do census phân theo tên. Kích thước lưới, số vị trí spawn và "
        "`d1/d2` lấy từ header; chúng chưa xác định sức chứa phòng, đội hay mã mode.",
        "Cột Rơi ghi số map có hàng / tổng số hàng / số tên khác nhau có trọng số dương. "
        "Cột Đặt sẵn ghi số map có hàng / tổng số hàng, giữ cả tên lặp.",
        "Hai cột này chỉ tính map đọc trọn. JSON giữ riêng các hàng thu được từ map đọc một phần; "
        "số 0 ở phần đó chưa chứng minh không có dữ liệu.",
        "Cột Nhóm là nhãn ứng viên từ wiki hoặc tên họ. Lớp tham chiếu hàng `kind=map` của bảng xếp loại, "
        "không kế thừa từ lớp mode.",
        "",
        "| Họ | Map | Lưới | Spawn | Script có/không/chưa rõ | d1/d2 | Rơi | Đặt sẵn | Lớp | Nhóm ứng viên |",
        "|---|---:|---|---|---|---|---|---|---|---|",
    ]
    for row in catalog["families"]:
        dims = ", ".join(f"{w}×{h}" for w, h in row["grid_sizes"])
        pairs = ", ".join(f"{d1}/{d2}" for d1, d2 in row["d1_d2_pairs"])
        scripts = "/".join(str(row["has_script"][key]) for key in ("yes", "no", "unknown"))
        drop = row["drops"]["exact"]
        placed = row["placements"]["exact"]
        grade = (row["classification"] or {}).get("classification", "—")
        grouping = row["grouping"]
        wiki = ", ".join(grouping["wiki_labels"])
        group = f"`{grouping['ruleset_group']}`" + (f" · {wiki}" if wiki else "")
        lines.append(
            f"| {row['family']} | {row['map_count']} | {dims} | {spans(row['spawn_range'])} | "
            f"{scripts} | {pairs} | {drop['maps_with_rows']}/{drop['rows']}/{drop['distinct_positive_names']} | "
            f"{placed['maps_with_rows']}/{placed['rows']} | {grade} | {group} |"
        )
    lines.append("")
    partial = [(family["family"], row) for family in catalog["families"]
               for row in family.get("partial_tail_diagnostics", [])]
    if partial:
        lines += [
            "## Chẩn đoán phần đuôi đọc một phần",
            "",
            "Chỉ dùng hàng `parse=partial` trong phạm vi mission của census. Dấu — là metadata chưa có.",
            "Số hàng đã đọc và danh sách tag rỗng không xác nhận phần đuôi đã đọc hết hay map không có script.",
            "",
            "| Họ / map | Hàng rơi / đặt sẵn đã đọc | Tag đã ghi | Lý do | Tag thiếu | Phần đuôi |",
            "|---|---:|---|---|---|---|",
        ]
        for family, row in partial:
            seen = row["observed"]
            lines.append(
                f"| {family} / `{row['map']}` | {seen['drop_rows']}/{seen['placement_rows']} | "
                f"{metadata_cell(seen['script_tags'])} | {metadata_cell(row['reason'])} | "
                f"{metadata_cell(row['missing_tags'])} | {metadata_cell(row['tail'])} |"
            )
        lines.append("")
    parity = catalog["rule_group_parity"]
    lines += [
        "JSON chỉ sao chép mã map nếu census có trường riêng. Tên map hoặc hậu tố số không được dùng làm mã wire.",
        "",
        "## Đối chiếu mode, tài nguyên và phạm vi",
        "",
        "Nhãn wiki và đối ứng theo tên trong [bảng Mode](../CONTENT.md#mode) giúp chọn mẫu đo cho `RoomRules`. "
        "Số mục tài nguyên và khai báo `Attr/MapGroup` là metadata tĩnh. Số mục của một nhóm pack "
        "(ví dụ `FxMap/Map`) là của cả nhóm, không phải số tài nguyên riêng của mode.",
        "Họ ngoài `mission` và các khai báo có mẫu như `Tomb?` không cộng vào tổng 122 map. "
        "Popcade có map trong tổng kiểm kê và vẫn mang đợt F3 theo bảng Mode.",
        f"[Bảng ModeRules](../../{RULES_LOCATOR}) khớp nhóm và đợt của {parity['mode_count']} mode "
        f"cùng nhóm luật của {parity['linked_mission_families']} họ mission có liên kết. "
        "Đây là hợp đồng thiết kế cho override do vận hành chọn; các mode vẫn bị chặn khi tạo hoặc bắt đầu trận "
        "vì chưa có dispatch KR đã đo.",
        "Các trường thắng, hòa, hồi sinh, đồng hồ và reset mô tả phần cần xác định cho từng mode. "
        "`unknown` giữ chỗ cho phần chưa rõ, không kế thừa luật trận thường.",
        "",
        "| Mode | Nhóm ứng viên | Đợt / lớp mode | Dấu vết trong pack | Họ map (thuộc mission / tổng) |",
        "|---|---|---|---|---|",
    ]
    for row in catalog["modes"]:
        pack = row["pack"]
        evidence = [f"`{entry['locator']}` ({entry['entries'] if entry['entries'] is not None else 'chưa rõ'})"
                    for entry in pack["resource_groups"]]
        for entry in pack["map_groups"]:
            counts = ("chưa rõ" if entry["entries"] is None else
                      f"{len(entry['exact_census_matches'])}/{len(entry['entries'])} tên khớp")
            evidence.append(f"`{entry['locator']}` ({counts})")
        if pack["detail_from_mode_table"]:
            evidence.append(pack["detail_from_mode_table"])
        related = ", ".join(f"`{ref['family']}` ({ref['mission_map_count']}/{ref['map_count']})"
                            for ref in row["related_families"])
        grade = (row["classification"] or {}).get("classification", "—")
        lines.append(f"| {row['mode']} | `{row['ruleset_group']}` | {row['wave']} / {grade} | "
                     f"{', '.join(evidence) or 'Chưa ghép'} | {related or '—'} |")
    lines += [
        "",
        "Họ map `SnowyFields` giữ nhóm riêng `snowy_fields`; nhãn ném tuyết của wiki chưa chứng minh "
        "đối ứng với mode sự kiện `snowball_event`. "
        "Các họ có chủ đề như `Zombie` hoặc `Gumiho` cũng chưa có đối ứng với hồi sinh, nhím hay bắt thủ lĩnh.",
        "",
        "Cột Dấu vết ghi số mục `Attr/MapGroup` khớp nguyên tên duy nhất trong census trên tổng số mục khai báo. "
        "JSON giữ nguyên thứ tự và ký tự của khai báo; chưa diễn giải cách client xử lý `?`.",
        "Khi đo Ladder, dùng một map trùng tên đã có trong "
        "[phạm vi nghiệm thu G1](../evidence/rust-service/g1-acceptance-matrix.json), chẳng hạn `Factory07`, "
        "làm đối chứng trận thường. Trước khi chọn map thử hồi sinh, đo cách client xử lý "
        "các khai báo như `Tomb?` và `Cemetery?`.",
        "",
        "## Phép đo còn thiếu cho F2",
        "",
        "1. Dùng IDA trên build KR 00967 để lần từ UI chọn mode hoặc `Attr/MapGroup` đến reader/writer "
        "([điểm neo tĩnh Wave 8a](../evidence/content/f4-w8-ida-exp-175-f2.json): `MapGroupVisible` "
        "`0xe04cb0`/`0xe04dd0`, `Dlg_LB-Create_Ladder` `0xc541e0`, "
        "`Dlg_LB-Create_Mission` `0xc8df50`, `roomtype` `0x57f520`). "
        "Đo native trong mạng cô lập với cùng map, roster, đội và sức chứa; chỉ đổi lựa chọn mode. "
        "Nếu client không mở được lựa chọn Ladder, ghi điểm dừng ở UI hoặc luồng ghép trận trước khi thử gán giá trị wire.",
        "   - Đối chiếu [tạo phòng 81/86](../protocol/room-membership.md#room-create) và "
        "[luồng join](../protocol/room-membership.md#duo-room-round). [Selector Free/Manner đã đo]"
        "(../protocol/room-membership.md#current-mode-host-rejoin) là biến đối chứng về đội; "
        "chưa có bằng chứng đây là trường chọn mode F2.",
        "   - Đối chiếu [map và Start 96/117/104](../protocol/load-phase.md#room-map) với "
        "[Start 121](../protocol/load-phase.md#start-reply). Giữ riêng mã map, dynamic index và byte `roomtype` "
        "đã đọc trong mẫu trận thường; cần xác định client thực sự đọc trường nào khi chuyển luồng.",
        "2. Đo hai client qua ít nhất một đường thắng và một đối chứng không kích hoạt của mode đã nhận diện. "
        "Ghi bên phát sự kiện, trạng thái đội/người chơi, điều kiện kết thúc, kết quả và reset ván kế. "
        "Với map có script, xác định bên thực thi trigger và quyền áp dụng tác động trước khi thêm `RoomRules`.",
        "3. Lần riêng chọn map của Ladder/hồi sinh và luồng UXMatching trước khi đo luật trận. "
        "Kiểm parser phần đuôi trên các map đọc một phần trước khi dùng bảng rơi, placement hay script của chúng.",
        "",
        "## Tái lập",
        "",
        "```bash",
        "python3 tools/kr_f2_mode_audit.py --generate",
        "python3 tools/kr_f2_mode_audit.py --check",
        "```",
        "",
        "`catalog_sha256` băm object JSON sau khi bỏ chính trường hash và `input_sha256`: "
        "UTF-8, khóa sắp tăng, không khoảng trắng, một LF cuối. "
        "`input_sha256` băm byte gốc của từng file nguồn.",
        "Công cụ đọc census, bảng xếp loại và bảng ModeRules, chạy offline và không cần pack, IDA hay client. "
        "Nếu census bổ sung `partial_reason`, `missing_tags` hoặc `tail` "
        "(`offset`, `remaining_bytes`), công cụ sao chép chúng vào chẩn đoán.",
        "Khi dùng `--census` khác, chọn `--classification` tương ứng hoặc `--no-classification` "
        "để giữ lớp chưa rõ. Dùng `--out` và `--markdown-out` để chọn file đầu ra riêng.",
        "",
    ]
    return "\n".join(lines)


def metadata_cell(value):
    if value is None:
        return "—"
    return canonical(value).decode().strip().replace("|", "&#124;").replace("<", "&lt;")


def read_json(path):
    raw = path.read_bytes()
    return json.loads(raw), sha256(raw)


def validate_classification(census, census_hash, classification):
    for key, expected in (("build", census["build"]), ("census_sha256", census_hash),
                          ("client_sha256", census.get("client_exe_sha256"))):
        if key in classification and classification[key] != expected:
            raise ValueError(f"xếp loại không khớp census: {key}")


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--census", type=Path, default=CENSUS, help="census JSON (mặc định: bản trong repo)")
    grades = parser.add_mutually_exclusive_group()
    grades.add_argument("--classification", type=Path, help="xếp loại JSON (mặc định: bản trong repo nếu có)")
    grades.add_argument("--no-classification", action="store_true", help="bỏ đối chiếu xếp loại")
    parser.add_argument("--out", type=Path, help="file catalog JSON; mặc định in ra stdout")
    parser.add_argument("--markdown-out", type=Path, help="file tóm tắt Markdown")
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--generate", action="store_true", help="ghi lại JSON và Markdown trong repo")
    mode.add_argument("--check", action="store_true", help="so sánh JSON và Markdown với bản sinh lại")
    args = parser.parse_args(argv)

    try:
        census, census_hash = read_json(args.census)
        class_path = None if args.no_classification else args.classification
        if not args.no_classification and class_path is None and CLASSIFICATION.is_file():
            class_path = CLASSIFICATION
        classification, class_hash = read_json(class_path) if class_path else (None, None)
        if classification is not None:
            validate_classification(census, census_hash, classification)
        rules, rules_hash = read_json(RULES)
        catalog = build_catalog(census, classification, rules=rules)
        catalog["input_sha256"] = {"census": census_hash, "classification": class_hash, "rules": rules_hash}
        default_targets = args.generate or args.check
        out = args.out or (CATALOG if default_targets else None)
        markdown = args.markdown_out or (MARKDOWN if default_targets else None)
        outputs = [(out, ".json", render_json(catalog))]
        if markdown is not None:
            outputs.append((markdown, ".md", render_markdown(catalog)))
        inputs = {path.resolve() for path in (args.census, class_path, RULES) if path is not None}
        for path, suffix, body in outputs:
            reject_private(body)
            if path is not None and (path.suffix != suffix or path.resolve() in inputs):
                raise ValueError(f"đầu ra phải là file {suffix} riêng với đầu vào")
            if path is not None and not args.check and (path.is_symlink() or
                    (path.exists() and (not path.is_file() or path.stat().st_nlink != 1))):
                raise ValueError("đầu ra phải là file thường, không phải liên kết")
        for path, _, body in outputs:
            raw = body.encode("utf-8")
            if args.check:
                if not path.is_file() or path.read_bytes() != raw:
                    raise ValueError(f"{path.name} chưa đồng bộ; chạy lại với --generate")
            elif path is None:
                sys.stdout.buffer.write(raw)
            else:
                path.write_bytes(raw)
        if args.check:
            print(f"F2 OK: {EXPECTED_MAPS} map / {EXPECTED_FAMILIES} họ; {catalog['catalog_sha256']}")
    except (OSError, ValueError, KeyError, TypeError) as exc:
        parser.exit(1, f"Không thể kiểm kê F2: {exc}\n")


if __name__ == "__main__":
    main()
