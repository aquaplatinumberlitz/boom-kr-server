"""Sinh kiểm kê F6-0 từ census và xếp loại KR 00967 đã lưu."""

import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import re


ROOT = Path(__file__).resolve().parents[1]
CENSUS = ROOT / "docs/content/census.json"
CLASSIFICATION = ROOT / "docs/content/classification.json"
CATALOG = ROOT / "docs/evidence/content/f6-events-catalog.json"
MARKDOWN = ROOT / "docs/content/f6-events-catalog.md"
POLICY = {"source": "design", "status": "excluded_policy", "locator": "docs/PLAN.md#full-game-roadmap",
          "reason": "F6 bỏ ưu đãi phòng máy Nexon (PC-bang)."}
EXCLUDED = "PCRoomBenefit"
RELATED = {
    "Antarctic": "Tên gợi chủ đề mùa lạnh; bảng Mode xếp tạm vào F2.",
    "Bộ sưu tập item": "Đối chiếu kinh tế vật phẩm ở F4; chưa có bằng chứng dùng cho sự kiện F6.",
    "IamWriggle": "Tên có phần Wriggle chung với RetroWriggle; chưa xác định quan hệ.",
    "Monster": "Có nhóm MonsterSeason2; lộ trình giữ gameplay hai mùa ở F3.",
    "SnowyChamp": "Tên gợi chủ đề tuyết; bảng Mode xếp tạm vào F2.",
}
UNKNOWN = {"kr_wire": "unknown", "original_server_prices_and_rates": "unknown",
           "active_in_build": "unknown"}
PRIVATE = re.compile(
    r"/home/|/Users/|ocid" r"1\.|BEGIN [A-Z0-9 ]*PRIVATE" r" KEY|"
    r"(?<![\d.])(?:10\.\d{1,3}|172\.(?:1[6-9]|2\d|3[01])|192\.168|198\.(?:1[89]))"
    r"(?:\.\d{1,3}){2}(?![\d.])", re.IGNORECASE,
)


def canonical(value):
    return (json.dumps(value, ensure_ascii=False, sort_keys=True,
                       separators=(",", ":"), allow_nan=False) + "\n").encode("utf-8")


def sha256(raw):
    return hashlib.sha256(raw).hexdigest()


def reject_private(text):
    if PRIVATE.search(text):
        raise ValueError("metadata chứa đường dẫn máy, địa chỉ nội bộ hoặc định danh riêng")


def unique_index(rows, key):
    index = {}
    for row in rows:
        name = key(row)
        if name in index:
            raise ValueError("trùng khóa trong metadata hệ thống hoặc nhóm tài nguyên")
        index[name] = row
    return index


def build_catalog(census, classification):
    systems = unique_index(census["systems"], lambda row: row["name"])
    groups = unique_index(census["resource_groups"], lambda row: f"{row['pack']}/{row['name']}")
    grades = unique_index((row for row in classification["rows"] if row["kind"] == "system"),
                          lambda row: row["name"])
    scoped = {name for name, row in grades.items() if row["wave"] == "F6"}
    if grades.get(EXCLUDED, {}).get("wave") != "ngoài phạm vi":
        raise ValueError("xếp loại không còn tách PCRoomBenefit khỏi phạm vi F6")
    if not scoped:
        raise ValueError("xếp loại không có hệ thống F6")

    rows = []
    packs = set()
    for name in sorted(scoped | RELATED.keys() | {EXCLUDED}):
        system, grade = systems[name], grades[name]
        refs = []
        for locator in sorted(set(system["groups"])):
            group = groups[locator]
            if group["system"] != name:
                raise ValueError("nhóm tài nguyên không khớp hệ thống trong census")
            packs.add(group["pack"])
            refs.append({"locator": locator, "entries": group["entries"]})
        if not refs or sum(ref["entries"] for ref in refs) != system["entries"]:
            raise ValueError("số mục tài nguyên không khớp hệ thống trong census")

        scope = "in_scope" if name in scoped else "related_candidate"
        row = {
            "system": name, "scope": scope,
            "classification": {key: grade[key] for key in ("classification", "wave", "status")},
            "static_metadata": {"entries": system["entries"], "resource_groups": refs,
                                "wiki_labels": sorted(system["wiki"])},
            **UNKNOWN,
        }
        if name == EXCLUDED:
            row.update(scope="excluded_policy", policy=POLICY.copy())
        elif scope == "related_candidate":
            row["scope_note"] = RELATED[name]
        rows.append(row)

    counts = Counter(row["scope"] for row in rows)
    catalog = {
        "schema": "kre.f6-events-catalog.v1", "build": census["build"],
        "static_metadata": {
            "source": "kr_static", "status": "confirmed",
            "locator": "docs/content/census.json: systems, resource_groups",
            "client_exe_sha256": census["client_exe_sha256"],
            "pack_hashes": {name: {key: value for key, value in census["packs"][name].items()
                                   if key in {"idx_sha256", "idd_sha256"}} for name in sorted(packs)},
        },
        "scope_locator": "docs/content/classification.json: rows[kind=system]",
        "summary": {"census_systems": len(systems), **dict(counts)},
        "systems": rows,
    }
    raw = canonical(catalog)
    reject_private(raw.decode("utf-8"))
    return {"catalog_sha256": sha256(raw), **catalog}


def render_json(catalog):
    # Một hệ thống mỗi dòng, như catalog F2-0.
    lines = ["{"]
    for key, value in sorted(catalog.items()):
        if key == "systems":
            rows = ",\n".join("    " + canonical(row).decode().rstrip() for row in value)
            lines.append(f'  "{key}": [\n{rows}\n  ],')
        else:
            lines.append(f'  "{key}": {canonical(value).decode().rstrip()},')
    lines[-1] = lines[-1].removesuffix(",")
    return "\n".join([*lines, "}", ""])


def render_markdown(catalog):
    summary = catalog["summary"]
    lines = [
        "# Kiểm kê sự kiện và kinh tế theo mùa F6-0",
        "",
        "Nguồn: [Hệ thống của client](../CONTENT.md#hệ-thống-của-client), [xếp loại A/B/C](classification.md) "
        "và [phạm vi F6](../PLAN.md#full-game-roadmap). "
        "[Catalog JSON](../evidence/content/f6-events-catalog.json) giữ locator và hash đầu vào.",
        f"Trong {summary['census_systems']} hệ thống của {catalog['build']}, bảng dưới có "
        f"**{summary['in_scope']} hệ thống được xếp vào F6**, "
        f"{summary.get('related_candidate', 0)} hệ thống đối chiếu và {summary['excluded_policy']} mục bị loại.",
        "Con số khoảng 25 trong PLAN là ước lượng phạm vi. Các mục đối chiếu giữ nguyên đợt trong bảng xếp loại. "
        "Dwarf/libcef dùng chung F4/F6 nhưng không có hàng hệ thống riêng trong census.",
        "",
        "Số mục và locator là metadata tĩnh từ index pack. Chúng cho biết tài nguyên có mặt, "
        "chưa cho biết client có mở tính năng hay server xử lý thế nào. Cả ba trường: wire KR, giá trị/giá/tỷ lệ "
        "của server gốc và trạng thái còn chạy hay đã ngừng trong build 00967 đều là `unknown` cho từng hệ thống.",
        "",
        "| Hệ thống | Phạm vi / đợt xếp loại | Nhóm tài nguyên (số mục) | Ghi chú |",
        "|---|---|---|---|",
    ]
    for row in sorted(catalog["systems"], key=lambda row: (row["scope"] != "in_scope", row["scope"], row["system"])):
        refs = ", ".join(f"`{group['locator']}` ({group['entries']})"
                         for group in row["static_metadata"]["resource_groups"])
        note = row.get("policy", {}).get("reason", row.get("scope_note", "—"))
        lines.append(f"| {row['system']} | `{row['scope']}` / {row['classification']['wave']} | {refs} | {note} |")
    lines += [
        "",
        "`DailyMission` nằm trong hàng Nhiệm vụ; các nhóm `Quest`, `CAMission` và `MissionPong` vẫn giữ nguyên "
        "theo census. Câu cá gồm tài nguyên trang trí, vật thể và giao diện trao đổi; DTT có nhóm `DTT_seasonShop`.",
        "[Bảng Mode](../CONTENT.md#mode) nhận diện MagicMarket là hệ chế tạo từ sảnh. "
        "Nguồn tĩnh chưa xác định công thức. Bảng Pet trong census cho biết có dữ liệu XML, "
        "chưa xác định tỷ lệ của server gốc.",
        "",
        "Khi mở từng hệ thống F6, lần từ tên nhóm tới UI và bộ đọc/ghi của build KR 00967. "
        "Kiểm đường mở UI và trao đổi dữ liệu bằng client trong mạng cô lập với server cục bộ. "
        "Cần nguồn KR riêng để xác nhận giá, thưởng và tỷ lệ gốc. Nếu không có nguồn, giữ `unknown`; "
        "giá trị tự chọn cho server giả lập phải ghi riêng là quyết định thiết kế.",
        "",
        "## Tái lập",
        "",
        "```bash",
        "python3 tools/kr_f6_events_audit.py",
        "python3 tools/kr_f6_events_audit.py --check",
        "```",
        "",
        "Công cụ chỉ đọc `census.json` và `classification.json`; không cần client, pack hay mạng. "
        "`input_sha256` băm byte gốc của hai file. `catalog_sha256` băm object JSON bỏ hai trường hash đó: "
        "UTF-8, khóa sắp tăng, không khoảng trắng, một LF cuối.",
        "",
    ]
    return "\n".join(lines)


def read_json(path):
    raw = path.read_bytes()
    return json.loads(raw), sha256(raw)


def validate_classification(census, census_hash, classification):
    for key, expected in (("build", census["build"]), ("census_sha256", census_hash),
                          ("client_sha256", census["client_exe_sha256"])):
        if classification[key] != expected:
            raise ValueError(f"xếp loại không khớp census: {key}")


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="so sánh với JSON và Markdown trong repo")
    args = parser.parse_args(argv)
    try:
        census, census_hash = read_json(CENSUS)
        classification, class_hash = read_json(CLASSIFICATION)
        validate_classification(census, census_hash, classification)
        catalog = build_catalog(census, classification)
        catalog["input_sha256"] = {"census": census_hash, "classification": class_hash}
        outputs = [(CATALOG, render_json(catalog)), (MARKDOWN, render_markdown(catalog))]
        for path, body in outputs:
            reject_private(body)
            if not args.check and (path.is_symlink() or
                                   (path.exists() and (not path.is_file() or path.stat().st_nlink != 1))):
                raise ValueError("đầu ra phải là file thường, không phải liên kết")
        for path, body in outputs:
            raw = body.encode("utf-8")
            if args.check:
                if not path.is_file() or path.read_bytes() != raw:
                    raise ValueError(f"{path.name} chưa đồng bộ; chạy lại công cụ sinh catalog")
            else:
                path.write_bytes(raw)
        summary = catalog["summary"]
        print(f"F6-0 OK: {summary['in_scope']} F6, {summary.get('related_candidate', 0)} đối chiếu, "
              f"{summary['excluded_policy']} loại theo chính sách; {catalog['catalog_sha256']}")
    except (OSError, ValueError, KeyError, TypeError) as exc:
        parser.exit(1, f"Không thể kiểm kê F6-0: {exc}\n")


if __name__ == "__main__":
    main()
