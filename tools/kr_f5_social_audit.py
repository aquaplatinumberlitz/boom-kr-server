"""Kiểm kê F5-0 từ census và bảng xếp loại đã lưu của KR 00967."""

import argparse
import hashlib
import json
from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[1]
CENSUS = ROOT / "docs/content/census.json"
CLASSIFICATION = ROOT / "docs/content/classification.json"
CATALOG = ROOT / "docs/evidence/content/f5-social-catalog.json"
MARKDOWN = ROOT / "docs/content/f5-social-catalog.md"
SYSTEMS = (
    ("Chatting", "Chat", "Thì thầm, danh sách chặn và báo cáo lạm dụng"),
    ("Messenger", "Bạn bè / Messenger", "Lời mời, danh sách bạn và trạng thái hiện diện"),
    ("Guild", "Bang hội", "Danh sách, thành viên và quyền quản lý bang"),
    ("Couple", "Couple", "Lời mời, xác nhận và trạng thái cặp đôi"),
    ("Ranking", "Xếp hạng CAJJang", "Yêu cầu bảng hạng và dữ liệu trả về"),
)
HASH = re.compile(r"[0-9a-f]{64}")
LOCATOR = re.compile(r"[A-Za-z0-9_]+/[A-Za-z0-9_]+(?::[A-Za-z0-9_]+)?")
PRIVATE = re.compile(
    r"/" r"home/|/Users/|ocid" r"1\.|BEGIN [A-Z0-9 ]*PRIVATE" r" KEY|"
    r"(?<![\d.])(?:10\.\d{1,3}|172\.(?:1[6-9]|2\d|3[01])|192\.168|198\.(?:1[89]))"
    r"(?:\.\d{1,3}){2}(?![\d.])"
)


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def canonical(data):
    return (json.dumps(data, ensure_ascii=False, sort_keys=True,
                       separators=(",", ":"), allow_nan=False) + "\n").encode("utf-8")


def reject_private(text):
    if PRIVATE.search(text):
        raise ValueError("đầu ra chứa metadata riêng")


def index(rows, key):
    result = {}
    for row in rows:
        name = key(row)
        if name in result:
            raise ValueError("trùng khóa trong metadata")
        result[name] = row
    return result


def build_catalog(census, classification, *, census_hash, classification_hash):
    if (census["schema"] != "kre-content-census-v1"
            or classification["schema"] != "kre-content-classification-v1"):
        raise ValueError("schema metadata không hỗ trợ")
    client_hash = census["client_exe_sha256"]
    if (census["build"] != "KR00967" or classification["build"] != census["build"]
            or classification["source"] != "kr_static"
            or classification["client_sha256"] != client_hash
            or classification["census_sha256"] != census_hash
            or not all(HASH.fullmatch(value) for value in
                       (client_hash, census_hash, classification_hash))):
        raise ValueError("build hoặc hash đầu vào không khớp")

    resources = index(census["resource_groups"], lambda row: row["pack"] + "/" + row["name"])
    census_systems = index(census["systems"], lambda row: row["name"])
    grades = index((row for row in classification["rows"]
                    if row["kind"] == "system" and row["wave"] == "F5"), lambda row: row["name"])
    if grades.keys() != {name for _, name, _ in SYSTEMS}:
        raise ValueError("phạm vi phân hệ F5 trong bảng xếp loại đã đổi")

    packs = census["packs"]
    seen = set()
    subsystems = []
    for subsystem, name, gap in SYSTEMS:
        grade, system = grades[name], census_systems[name]
        locators = set(grade["groups"])
        if (not locators or len(locators) != len(grade["groups"])
                or sorted(grade["groups"]) != sorted(system["groups"])
                or locators & seen):
            raise ValueError("nhóm tài nguyên F5 trùng hoặc không khớp census")
        seen.update(locators)
        groups = []
        for locator in sorted(locators):
            if not LOCATOR.fullmatch(locator):
                raise ValueError("locator tài nguyên không hợp lệ")
            row = resources[locator]
            count = row["entries"]
            pack_hash = packs[row["pack"]]["idx_sha256"]
            if (row["system"] != name or type(count) is not int or count < 1
                    or not HASH.fullmatch(pack_hash)):
                raise ValueError("nhóm tài nguyên F5 không hợp lệ")
            groups.append({"locator": locator, "entries": count, "idx_sha256": pack_hash})
        total = sum(row["entries"] for row in groups)
        if type(system["entries"]) is not int or total != system["entries"]:
            raise ValueError("số mục phân hệ F5 không khớp census")
        if grade["classification"] not in {"A", "B", "C"} or grade["status"] not in {
            "confirmed", "candidate", "unknown", "design",
        }:
            raise ValueError("mức xếp loại F5 không hợp lệ")
        subsystems.append({
            "id": subsystem, "system": name, "entries": total, "resource_groups": groups,
            "classification": {key: grade[key] for key in ("classification", "status", "wave")},
            "kr_wire": {"status": "unknown", "unverified": gap},
        })

    catalog = {
        "schema": "kre-f5-social-catalog-v1", "scope": "F5-0", "source": "kr_static",
        "status": "confirmed", "build": census["build"], "client_sha256": client_hash,
        "inputs": {"census_sha256": census_hash, "classification_sha256": classification_hash},
        "limits": "Chỉ xác nhận locator và số mục từ census; không suy wire, luật hoặc quyền server.",
        "summary": {"subsystems": len(subsystems), "resource_groups": len(seen),
                    "entries": sum(row["entries"] for row in subsystems)},
        "subsystems": subsystems,
    }
    catalog["catalog_sha256"] = sha256(canonical(catalog))
    return catalog


def render_json(catalog):
    return json.dumps(catalog, ensure_ascii=False, sort_keys=True, indent=2, allow_nan=False) + "\n"


def render_markdown(catalog):
    summary = catalog["summary"]
    lines = [
        "# Kiểm kê tài nguyên xã hội F5-0", "",
        "Nguồn: [kiểm kê pack](../CONTENT.md), [census JSON](census.json) và "
        "[bảng xếp loại A/B/C](classification.md). "
        "[Catalog JSON](../evidence/content/f5-social-catalog.json) giữ hash đầu vào và IDX nguồn.",
        f"Build `{catalog['build']}` có **{summary['subsystems']} phân hệ, "
        f"{summary['resource_groups']} nhóm, {summary['entries']} mục** trong phạm vi F5-0.", "",
        "Số mục lấy từ index pack. Tên và số mục giúp chọn điểm bắt đầu kiểm tra; "
        "chúng chưa xác nhận mã, bố cục gói tin, quy tắc hay quyền quyết định của server KR.", "",
        "| Phân hệ | Nhóm tài nguyên (số mục) | Lớp | Wire F5 cần xác nhận |",
        "|---|---|---|---|",
    ]
    for row in catalog["subsystems"]:
        groups = ", ".join(f"`{g['locator']}` ({g['entries']})" for g in row["resource_groups"])
        wire = row["kr_wire"]
        lines.append(f"| `{row['id']}` | {groups} | {row['classification']['classification']} | "
                     f"`{wire['status']}`: {wire['unverified']} |")
    lines += [
        "",
        "Chat sảnh/phòng đã có [bằng chứng L1](../protocol/lobby-profile.md#rust-patched-room-chat) "
        "trong phạm vi client vá 949x. Bằng chứng đó chưa xác nhận thì thầm, chat bang hội hoặc chặn.",
        "Hàng `Ranking` kiểm kê nhóm CAJJang theo bảng xếp loại; tài nguyên Ladder thuộc phạm vi F2.",
        "",
        "Schema 9 dùng giới hạn nội bộ: mỗi người có tối đa 100 bạn, 100 tài khoản chặn và "
        "100 lời mời đang chờ cho mỗi chiều gửi hoặc nhận. Chặn ở một chiều ngăn gửi riêng ở cả hai chiều.",
        "Chat cho phép tối đa 512 byte mỗi tin và 5 tin mỗi cửa sổ UTC 10 giây chung cho chat và thì thầm. "
        "Audit lưu lý do, số byte và số tin đã được duyệt trong cửa sổ; không lưu nội dung tin nhắn.",
        "Bang hội cho gia nhập trực tiếp, tối đa 100 người và mỗi người chỉ thuộc một bang. "
        "Tên bang dùng chữ cái ASCII, chữ số và khoảng trắng; khóa duy nhất không phân biệt hoa thường.",
        "Bảng hạng nội bộ dùng thứ tự cấp, EXP và số trận thắng đã hoàn tất giảm dần, rồi ID người chơi tăng dần. "
        "Couple hiện chỉ có kiểm kê tài nguyên.",
        "",
        "Để xác nhận wire F5, cần đối chiếu điểm gọi tĩnh của từng thao tác với capture KR 00967 "
        "trong mạng cô lập, có thao tác đối chứng và trạng thái trước/sau.", "",
        "## Tái lập", "", "```bash", "python3 tools/kr_f5_social_audit.py --generate",
        "python3 tools/kr_f5_social_audit.py --check", "```", "",
    ]
    return "\n".join(lines)


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--census", type=Path, default=CENSUS)
    parser.add_argument("--classification", type=Path, default=CLASSIFICATION)
    parser.add_argument("--out", type=Path, help="đầu ra JSON")
    parser.add_argument("--markdown-out", type=Path, help="đầu ra Markdown")
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--generate", action="store_true", help="ghi JSON và Markdown vào repo")
    mode.add_argument("--check", action="store_true", help="kiểm tra hai bản sinh sẵn")
    args = parser.parse_args(argv)

    try:
        census = args.census.read_bytes()
        grades = args.classification.read_bytes()
        catalog = build_catalog(json.loads(census), json.loads(grades), census_hash=sha256(census),
                                classification_hash=sha256(grades))
        outputs = [(args.out or (CATALOG if args.generate or args.check else None), render_json(catalog))]
        md = args.markdown_out or (MARKDOWN if args.generate or args.check else None)
        if md is not None:
            outputs.append((md, render_markdown(catalog)))
        used = {p.resolve() for p in (args.census, args.classification)}
        for (path, body), suffix in zip(outputs, (".json", ".md")):
            reject_private(body)
            if path is None:
                continue
            resolved = path.resolve()
            if path.suffix != suffix or resolved in used or path.is_symlink() or (
                    path.exists() and (not path.is_file() or path.stat().st_nlink != 1)):
                raise ValueError("đầu ra phải là file thường riêng với đầu vào")
            used.add(resolved)
        for path, body in outputs:
            raw = body.encode("utf-8")
            if path is None:
                sys.stdout.buffer.write(raw)
            elif args.check:
                if not path.is_file() or path.read_bytes() != raw:
                    raise ValueError("catalog chưa đồng bộ; chạy lại với --generate")
            else:
                path.write_bytes(raw)
        if args.check:
            print(f"F5-0 OK: {catalog['summary']['subsystems']} phân hệ; {catalog['catalog_sha256']}")
    except (OSError, ValueError, KeyError, TypeError) as exc:
        parser.exit(1, f"Không thể kiểm kê F5-0 ({type(exc).__name__}): "
                       + (str(exc) if type(exc) is ValueError else "kiểm tra đầu vào và đầu ra") + "\n")


if __name__ == "__main__":
    main()
