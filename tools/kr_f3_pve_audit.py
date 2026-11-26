"""Sinh catalog F3-0 từ metadata KR 00967 đã lưu, hoặc kiểm bằng --check."""

import argparse
from collections import Counter, defaultdict
import hashlib
import json
from pathlib import Path
import re


ROOT = Path(__file__).resolve().parents[1]
CENSUS = ROOT / "docs/content/census.json"
CLASSIFICATION = ROOT / "docs/content/classification.json"
CATALOG = ROOT / "docs/evidence/content/f3-pve-catalog.json"
MARKDOWN = ROOT / "docs/content/f3-pve-catalog.md"
GROUPS = {
    "monster": ("Monster (hai mùa)", ("몬스터모드 (monster)", "Monster mùa 2"), "Monster"),
    "quest_mission": ("Quest / Mission", ("Nhiệm vụ chơi với máy",), "Nhiệm vụ"),
    "rpg": ("RPG", ("RPG",), "RPG"),
    "ca_hero": ("CA Hero", ("크아 히어로 (CA Hero)",), "CAHero"),
    "hidden_catch": ("Hidden Catch", ("히든 캐치 (Hidden Catch)",), "Hidden Catch"),
    "bonus": ("Màn thưởng (Bonus)", ("Màn thưởng",), None),
    "ai_battle": ("AIBattle", ("AIBattle",), "AIBattle"),
    "tutorial": ("Hướng dẫn (Tutorial)", ("Hướng dẫn",), "Hướng dẫn"),
}
AUTHORITY = dict.fromkeys(("trigger_execution", "monster_ai", "rewards", "kr_wire_route"), "unknown")
PRIVATE = re.compile(
    r"/home/|/Users/|ocid" r"1\.|BEGIN [A-Z0-9 ]*PRIVATE" r" KEY|"
    r"(?<![\d.])(?:10\.\d{1,3}|172\.(?:1[6-9]|2\d|3[01])|192\.168|198\.(?:1[89]))"
    r"(?:\.\d{1,3}){2}(?![\d.])"
)


def canonical(value):
    return (json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":"),
                       allow_nan=False) + "\n").encode("utf-8")


def sha256(raw):
    return hashlib.sha256(raw).hexdigest()


def reject_private(body):
    if PRIVATE.search(body):
        raise ValueError("metadata chứa đường dẫn máy, địa chỉ nội bộ hoặc định danh riêng")


def select_group(row):
    category, family = row["category"], row["family"]
    if category == "rpg":
        return "ca_hero" if family == "CAhero" else "rpg"
    if category == "battle_variant" and family.startswith("Fm_AIBtl"):
        return "ai_battle"
    if category == "lobby" and family == "Tutorial":
        return "tutorial"
    return {"monster": "monster", "mission": "quest_mission", "hidden": "hidden_catch",
            "bonus": "bonus"}.get(category)


def index_rows(rows, key):
    index = {}
    for row in rows:
        name = key(row)
        if name in index:
            raise ValueError("trùng khóa trong metadata")
        index[name] = row
    return index


def grade(row):
    if row is None:
        return None
    if row["classification"] not in {"A", "B", "C"}:
        raise ValueError("xếp loại nằm ngoài A/B/C")
    return {key: row[key] for key in ("classification", "wave", "status")}


def parse_counts(maps):
    counts = Counter(row["parse"] for row in maps)
    if counts.keys() - {"exact", "partial"}:
        raise ValueError("trạng thái parse nằm ngoài exact/partial")
    return {key: counts[key] for key in ("exact", "partial")}


def observations(maps):
    counts = parse_counts(maps)
    exact = [row for row in maps if row["parse"] == "exact"]
    partial = [row for row in maps if row["parse"] == "partial"]
    yes = sum(row["scripted"] for row in exact)

    def symbols(rows):
        return {field: sorted({value for row in rows for value in row.get(field, [])})
                for field in ("script_tags", "script_triggers", "script_funcs")}

    return {"map_count": len(maps), "parse_counts": counts,
            "has_script": {"yes": yes, "no": len(exact) - yes, "unknown": len(partial)},
            "script_symbols": {"exact": symbols(exact), "partial_observed": symbols(partial)}}


def build_catalog(census, classification):
    grades = index_rows((row for row in classification["rows"] if row["kind"] in {"map", "mode", "system"}),
                        lambda row: (row["kind"], row["name"]))
    systems = index_rows(census.get("systems", []), lambda row: row["name"])
    resources = index_rows(census.get("resource_groups", []), lambda row: f"{row['pack']}/{row['name']}")
    by_group = defaultdict(list)
    by_family = defaultdict(list)
    name_counts = Counter(row["name"].casefold() for row in census["maps"])
    for row in census["maps"]:
        group = select_group(row)
        if group is not None:
            if name_counts[row["name"].casefold()] != 1:
                raise ValueError("trùng tên map trong phạm vi kiểm kê (không phân biệt hoa thường)")
            by_group[group].append(row)
            by_family[(group, row["family"])].append(row)

    families = []
    for (group, name), maps in sorted(by_family.items()):
        classification_row = grades.get(("map", name))
        if classification_row and classification_row["map_count"] != len(maps):
            raise ValueError("số map theo họ không khớp bảng xếp loại")
        families.append({
            "family": name, "group": group,
            "census_categories": sorted({row["category"] for row in maps}),
            **observations(maps),
            "maps": {key: sorted(row["name"] for row in maps if row["parse"] == key)
                     for key in ("exact", "partial")},
            "classification": grade(classification_row),
        })

    groups = []
    for group, (label, mode_names, system) in GROUPS.items():
        system_row = systems.get(system, {})
        locators = system_row.get("groups", [])
        if group == "bonus":
            locators = [key for key in ("FxMap/Bonus", "FxMap/BonusPrepare") if key in resources]
        maps = by_group[group]
        counts = Counter((row["classification"] or {}).get("classification", "unknown")
                         for row in families if row["group"] == group)
        entry = {
            "id": group, "label": label, **observations(maps),
            "scope": "cross_reference" if group == "quest_mission" else "f3_core",
            "family_count": sum(counts.values()),
            "classification_counts": {key: counts[key] for key in ("A", "B", "C", "unknown")},
            "mode_classifications": {name: grade(grades.get(("mode", name))) for name in mode_names},
            "system_classification": ({"name": system, "grade": grade(grades.get(("system", system)))}
                                      if system else None),
            "resource_groups": [{"locator": key, "entries": resources[key]["entries"]}
                                for key in sorted(locators)],
            "grouping_status": "candidate",
            "authority": AUTHORITY.copy(),
        }
        if group == "monster":
            season2 = [row for row in maps if row["family"].startswith("MonS2")]
            entry["season_2_name_candidate"] = {
                "family_prefix": "MonS2", "status": "candidate",
                "map_count": len(season2), "parse_counts": parse_counts(season2),
                "families": sorted({row["family"] for row in season2}),
            }
        groups.append(entry)

    selected = [row for maps in by_group.values() for row in maps]
    core = [row for key, maps in by_group.items() if key != "quest_mission" for row in maps]
    catalog = {
        "schema": "kre.f3-pve-catalog.v1", "build": census["build"], "authority": AUTHORITY.copy(),
        "static_metadata": {
            "source": "kr_static", "status": "confirmed",
            "locator": "docs/content/census.json: maps, systems, resource_groups",
            "classification_locator": "docs/content/classification.json: rows",
            "client_exe_sha256": census.get("client_exe_sha256"),
            "pack_hashes": {name: {key: row[key] for key in ("idx_sha256", "idd_sha256") if key in row}
                            for name, row in census.get("packs", {}).items()
                            if name in {"FxMap", "FxMonster", "FxHiddenCatch", "FxContents",
                                        "FxDesktop", "FxStage", "fx"}},
        },
        "summary": {
            "map_count": len(selected), "parse_counts": parse_counts(selected), "family_count": len(families),
            "f3_core": {"map_count": len(core), "parse_counts": parse_counts(core),
                        "family_count": sum(row["group"] != "quest_mission" for row in families)},
        },
        "groups": groups, "families": families,
    }
    raw = canonical(catalog)
    reject_private(raw.decode("utf-8"))
    return {"catalog_sha256": sha256(raw), **catalog}


def render_json(catalog):
    # Giữ mỗi họ một dòng để diff cho thấy chính xác họ nào thay đổi.
    lines = ["{"]
    for key, value in sorted(catalog.items()):
        if key in {"families", "groups"} and value:
            rows = ",\n".join("    " + canonical(row).decode().rstrip("\n") for row in value)
            lines.append(f'  "{key}": [\n{rows}\n  ],')
        else:
            lines.append(f'  "{key}": {canonical(value).decode().rstrip()},')
    lines[-1] = lines[-1].removesuffix(",")
    return "\n".join([*lines, "}", ""])


def render_markdown(catalog):
    summary = catalog["summary"]
    core = summary["f3_core"]
    groups = {row["id"]: row for row in catalog["groups"]}
    mission = groups["quest_mission"]
    season = groups["monster"]["season_2_name_candidate"]
    lines = [
        "# Kiểm kê PvE và map có script F3-0", "",
        "Sinh từ [census](census.json) và [xếp loại A/B/C](classification.json) của client "
        f"`{catalog['build']}`. Phạm vi theo [lộ trình F3](../PLAN.md#full-game-roadmap) và "
        "[bảng nội dung](../CONTENT.md#mode). [Catalog JSON](../evidence/content/f3-pve-catalog.json) "
        "giữ tên từng map, nhãn script theo họ và hash nguồn.", "",
        f"Catalog có **{summary['map_count']} map / {summary['family_count']} họ** "
        f"({summary['parse_counts']['exact']} `exact`, {summary['parse_counts']['partial']} `partial`). "
        f"Có **{core['map_count']} map / {core['family_count']} họ** trong phạm vi chính và "
        f"**{mission['map_count']} map / {mission['family_count']} họ** để đối chiếu Quest / Mission.", "",
        "Quyền thực thi trigger, quyền quyết định AI quái, quyền cấp thưởng và route KR wire đều "
        "là `unknown` ở mọi nhóm. Tag, tên tài nguyên và nhãn xếp loại chỉ giúp chọn mẫu kiểm.", "",
        "## Phạm vi và dấu vết theo nhóm", "",
        "Cột Script đếm có / không / chưa rõ từ cờ `scripted` của census. Hàng `exact` đọc trọn; "
        "hàng `partial` đọc một phần nên trạng thái script vẫn chưa rõ. "
        "Các tag là tên phân biệt hoa thường do census ghi ở `script_tags`; không thêm thẻ `<script>` "
        "vào danh sách nếu census chỉ ghi cờ. JSON giữ riêng tên thu được từ phần đã đọc của map `partial`.", "",
        "| Nhóm | Họ | Map exact / partial | Script có / không / chưa rõ | Xếp loại mode / hệ thống (đợt, lớp) | Tài nguyên (số mục) |",
        "|---|---:|---:|---:|---|---|",
    ]

    def parse_cell(row):
        return " / ".join(str(row["parse_counts"][key]) for key in ("exact", "partial"))

    def script_cell(row):
        return " / ".join(str(row["has_script"][key]) for key in ("yes", "no", "unknown"))

    for row in groups.values():
        classifications = dict(row["mode_classifications"])
        if system := row["system_classification"]:
            classifications[f"Hệ thống {system['name']}"] = system["grade"]
        grades = ", ".join(f"{name} ({value['wave']}, {value['classification']})" if value else
                           f"{name} (chưa xếp loại)" for name, value in classifications.items())
        resources = ", ".join(f"`{item['locator']}` ({item['entries']})" for item in row["resource_groups"])
        lines.append(f"| {row['label']} | {row['family_count']} | {parse_cell(row)} | {script_cell(row)} | "
                     f"{grades} | {resources or '—'} |")
    lines += [
        "",
        f"- Monster: `FxMonster/Monster` và `FxMonster/MonsterSeason2` là hai nhóm tài nguyên. "
        f"Tiền tố `MonS2` đánh dấu {season['map_count']} map thuộc {len(season['families'])} họ "
        "ứng viên mùa 2. Census chưa gán mùa cho từng map; các họ còn lại chưa xác nhận là mùa 1.",
        "- Quest / Mission: các map `category=mission` đã có trong [F2-0](f2-mode-catalog.md). "
        "Census chưa nối các map đó với mode nhiệm vụ chơi với máy hoặc với tài nguyên Quest / Mission. "
        "Hệ thống Nhiệm vụ mang đợt F6 theo xếp loại hiện hành; `DailyMissionShop` thuộc sảnh nên không cộng vào số map.",
        "- AIBattle lấy các họ `Fm_AIBtl*` từ nhóm biến thể đấu; Tutorial lấy đúng họ `Tutorial` từ nhóm sảnh. "
        "Số mục tài nguyên không được cộng vào số map.",
        "", "## Dấu vết theo họ map", "",
        "Cột Lớp dùng hàng `kind=map` của từng họ. Các đợt ở bảng xếp loại được giữ nguyên, "
        "kể cả họ dùng để đối chiếu. Cột Tag đếm số tên khác nhau trong phần đọc trọn và phần đọc một phần; "
        "tên cụ thể theo họ có trong JSON.", "",
    ]
    for key, group in groups.items():
        tags = group["script_symbols"]["exact"]["script_tags"]
        lines += [f"### {group['label']}", "",
                  "Tag thấy ở map đọc trọn: " + (", ".join(f"`{tag}`" for tag in tags) or "không có") + ".", "",
                  "| Họ | Map exact / partial | Script có / không / chưa rõ | Tag exact / partial | Lớp (đợt) |",
                  "|---|---:|---:|---:|---|"]
        for row in catalog["families"]:
            if row["group"] != key:
                continue
            grade_row = row["classification"]
            grade_cell = f"{grade_row['classification']} ({grade_row['wave']})" if grade_row else "chưa xếp loại"
            tag_counts = " / ".join(str(len(row["script_symbols"][parse]["script_tags"]))
                                    for parse in ("exact", "partial_observed"))
            lines.append(f"| `{row['family']}` | {parse_cell(row)} | {script_cell(row)} | {tag_counts} | {grade_cell} |")
        lines.append("")
    lines += [
        "Phép đo tiếp theo: chọn một map có tag trigger và một map đối chứng, đối chiếu điểm đọc "
        "trên KR 00967 rồi đo native trong mạng cô lập để xác định bên phát sự kiện và bên áp dụng "
        "tác động. Đo riêng AI quái và cấp thưởng trước khi xác định route hay luật phía server.", "",
        "## Tái lập", "",
        "```bash",
        "python3 tools/kr_f3_pve_audit.py",
        "python3 tools/kr_f3_pve_audit.py --check",
        "```", "",
        "Công cụ chỉ đọc hai file JSON metadata. `input_sha256` băm byte gốc của từng input. "
        "`catalog_sha256` băm JSON sau khi bỏ `catalog_sha256` và `input_sha256`: UTF-8, khóa sắp tăng, "
        "không khoảng trắng và có một LF cuối.", "",
    ]
    return "\n".join(lines)


def load_catalog(census_path=CENSUS, classification_path=CLASSIFICATION):
    census_raw, classification_raw = census_path.read_bytes(), classification_path.read_bytes()
    census, classification = json.loads(census_raw), json.loads(classification_raw)
    census_hash = sha256(census_raw)
    for key, expected in (("build", census["build"]), ("census_sha256", census_hash),
                          ("client_sha256", census["client_exe_sha256"])):
        if classification[key] != expected:
            raise ValueError(f"xếp loại không khớp census: {key}")
    return build_catalog(census, classification) | {
        "input_sha256": {"census": census_hash, "classification": sha256(classification_raw)}
    }


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="so sánh đầu ra với bản đã lưu")
    args = parser.parse_args(argv)
    try:
        catalog = load_catalog()
        outputs = [(CATALOG, render_json(catalog)), (MARKDOWN, render_markdown(catalog))]
        for path, body in outputs:
            reject_private(body)
            if not args.check and (path.is_symlink() or
                                   (path.exists() and (not path.is_file() or path.stat().st_nlink != 1))):
                raise ValueError("đầu ra phải là file thường, không phải liên kết")
        for path, body in outputs:
            raw = body.encode("utf-8")
            if args.check:
                if path.read_bytes() != raw:
                    raise ValueError(f"{path.name} chưa đồng bộ; chạy lại công cụ để sinh file")
            else:
                path.write_bytes(raw)
        summary = catalog["summary"]
        print(f"F3-0 OK: {summary['map_count']} map / {summary['family_count']} họ; {catalog['catalog_sha256']}")
    except (OSError, TypeError, ValueError, KeyError) as exc:
        parser.exit(1, f"Không thể kiểm kê F3-0: {exc}\n")


if __name__ == "__main__":
    main()
