"""Sinh ma trận F5 mở rộng từ metadata KR 00967 đã lưu trong repo."""

import argparse
import hashlib
import ipaddress
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
INPUTS = {
    "classification": "docs/content/classification.json",
    "social_catalog": "docs/evidence/content/f5-social-catalog.json",
    "social_notes": "docs/content/f5-social-catalog.md",
    "ida": "docs/evidence/content/f2-f5-f3-ida-anchors.json",
    "coverage": "docs/evidence/owner-directives/full-content-coverage.json",
    "migration": "rust/crates/kr-service/migrations/0009_f5_social_foundation.sql",
    "roadmap": "docs/PLAN.md",
}
MATRIX = "docs/evidence/content/f5-extended-social-matrix.json"
MARKDOWN = "docs/content/f5-extended-social-matrix.md"
BUILD = "KR00967"
FAMILIES = {
    "LobbySquare": "Ứng viên cho sảnh quảng trường đi lại tự do.",
    "LobbyCaffe": "Khu cà phê; lộ trình nêu quà khi ngồi, chưa biết điều kiện và quà cụ thể.",
    "LobbyClothing": "Khu quần áo; chưa biết thao tác tại khu.",
    "LobbyJewelry": "Khu trang sức; chưa biết thao tác tại khu.",
    "LobbyPet": "Khu thú cưng; chưa biết thao tác tại khu.",
    "Square": "Ứng viên cho map quảng trường; chưa biết quan hệ chuyển cảnh với LobbySquare.",
}
ROADMAP_NAMES = {
    "Plaza": "Quảng trường (광장)",
    "Mail": "Thư (쪽지)",
    "ExchangeChat": "Phòng chat trao đổi (교환채팅방)",
    "Guild": "Huy hiệu và bang kín (길드 마크, 비공개 길드)",
}
ANCHORS = {
    "guild": {"S->C": [467, 474, 478], "C->S": [633]},
    "couple": {"S->C": [257, 273, 284, 640, 674], "C->S": [670]},
}
PRIVATE = re.compile(
    r"/(?:home|Users|root|var/lib|mnt|media|srv|tmp)/|\b[A-Z]:\\+Users\\+|"
    r"\bocid" r"1\.|-----BEGIN [A-Z0-9 ]*PRIVATE" r" KEY-----|"
    r"\b(?:gh[pousr]_[A-Za-z0-9]{16,}|github_pat_[A-Za-z0-9_]{16,}|"
    r"(?:AKIA|ASIA)[A-Z0-9]{16}|sk-[A-Za-z0-9_-]{16,}|"
    r"eyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+)\b|"
    r"""["']?\b(?:api[_-]?key|access[_-]?token|token|client[_-]?secret|password|secret)["']?"""
    r"""\s*[:=]\s*["']?[^\s"'\x60,}]+""",
    re.IGNORECASE,
)
IP_LITERAL = re.compile(
    r"(?<![\w.])(?:[0-9]{1,3}\.){3}[0-9]{1,3}(?!\w|\.[0-9])"
    r"|(?<![\w:])(?=[0-9A-Fa-f:.]*:[0-9A-Fa-f:.]*:)[0-9A-Fa-f:.]+(?![\w:])"
)


class AuditError(ValueError):
    """Lỗi có thông báo an toàn để in ra CLI."""


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def json_bytes(value, *, compact=False):
    options = {"separators": (",", ":")} if compact else {"indent": 2}
    return (json.dumps(value, ensure_ascii=False, sort_keys=True, allow_nan=False, **options)
            + "\n").encode("utf-8")


def reject_private(raw, label):
    body = raw.decode("utf-8")
    if PRIVATE.search(body):
        raise AuditError(f"{label}: có đường dẫn máy hoặc thông tin định danh/đăng nhập riêng")
    for match in IP_LITERAL.finditer(body):
        try:
            address = ipaddress.ip_address(match[0].rstrip("."))
        except ValueError:
            continue
        if isinstance(address, ipaddress.IPv6Address):
            address = address.ipv4_mapped or address
        # Bao gồm cả địa chỉ dùng chung CGNAT (is_private cũng là False).
        if not address.is_global or address.is_multicast:
            raise AuditError(f"{label}: có địa chỉ IP không công khai")


def repo_file(root, relative):
    path = root / relative
    # Không đọc qua liên kết ra ngoài worktree, kể cả khi một thư mục cha bị thay.
    for candidate in (path, *path.parents):
        if candidate == root:
            break
        if candidate.is_symlink():
            raise AuditError(f"{relative}: cần tệp thường trong worktree")
    if path.exists() and (not path.is_file() or path.stat().st_nlink != 1):
        raise AuditError(f"{relative}: cần tệp thường trong worktree")
    return path


def read_inputs(root):
    data = {key: repo_file(root, path).read_bytes() for key, path in INPUTS.items()}
    # Chỉ đưa dòng F5 vào dấu vân tay; sửa đợt khác trong PLAN không làm ma trận cũ đi.
    rows = re.findall(rb"^\| F5 [^\r\n]*", data["roadmap"], re.M)
    if len(rows) != 1:
        raise AuditError("Cần đúng một dòng F5 trong lộ trình")
    data["roadmap"] = rows[0] + b"\n"
    return data


def select(rows, names, label, *, key="name"):
    selected = [row for row in rows if row[key] in names]
    result = {row[key]: row for row in selected}
    if len(result) != len(selected) or result.keys() != set(names):
        raise AuditError(f"{label}: thiếu hoặc trùng mục")
    return result


def unknown_wire(operations):
    return {"status": "unknown", "opcodes": "unknown", "unmeasured": operations}


def rule(value, note, *, status="unknown", unit=None):
    data = {"status": status, "value": value, "note": note}
    if unit:
        data["unit"] = unit
    return data


def storage(note, *reuse):
    return {"status": "design", "reuse_candidates": list(reuse), "extension": note}


def static_anchors(ida, group):
    rows = []
    for direction, key, locations in (
        ("S->C", "cstage_handlers", ("handler_va",)),
        ("C->S", "c2s_anchors", ("caller_va", "caller_vas")),
    ):
        found = sorted((r for r in ida[key] if r["group"] == group), key=lambda r: r["opcode"])
        if [r["opcode"] for r in found] != ANCHORS[group][direction]:
            raise AuditError(f"Neo IDA {group} không khớp phạm vi")
        for row in found:
            vas = [va for field in locations for va in
                   ([row[field]] if isinstance(row.get(field), str) else row.get(field, []))]
            if not vas or any(not re.fullmatch(r"0x[0-9a-f]+", va) for va in vas):
                raise AuditError(f"Neo IDA {group} thiếu VA hợp lệ")
            rows.append({"direction": direction, "opcode": row["opcode"], "vas": vas,
                         **({"ui_anchor": row["ui_anchor"]} if "ui_anchor" in row else {})})
    return {"source": "kr_static", "status": "confirmed", "anchors": rows}


def build_matrix(data):
    grades, catalog, ida, coverage = (json.loads(data[key]) for key in
                                      ("classification", "social_catalog", "ida", "coverage"))
    if (grades["schema"], catalog["schema"], ida["schema"], coverage["schema"]) != (
        "kre-content-classification-v1", "kre-f5-social-catalog-v1",
        "boom.f2f5f3.ida_anchors.v1", "kre-owner-directive-v1",
    ):
        raise AuditError("Schema đầu vào không hỗ trợ")
    client_hash = grades["client_sha256"]
    artifacts = ida["source_artifacts"]
    f5 = ida["f5_social"]
    if (grades["build"] != BUILD or catalog["build"] != BUILD
            or grades["source"] != "kr_static"
            or (catalog["source"], catalog["status"]) != ("kr_static", "confirmed")
            or f5["status"] != "kr_static/confirmed"
            or grades["idb_sha256"] != artifacts["kr_full_i64_sha256"]
            or not re.fullmatch(r"0x[0-9a-f]+", f5["cstage_dispatcher_va"])
            or any(not re.fullmatch(r"[0-9a-f]{64}", digest) for digest in
                   (client_hash, grades["idb_sha256"]))
            or client_hash != catalog["client_sha256"] or client_hash != artifacts["kr_ca_exe_sha256"]):
        raise AuditError("Build, hash client/IDA hoặc mức bằng chứng nguồn không khớp")
    # Catalog F5-0 ghim bản classification của lần xuất trước, không phải bản hiện tại.
    catalog_digest = catalog.pop("catalog_sha256")
    if catalog_digest != sha256(json_bytes(catalog, compact=True)):
        raise AuditError("SHA-256 của catalog F5-0 không khớp")

    rows = grades["rows"]
    map_rows = [r for r in rows if r["kind"] == "map" and r["wave"] == "F5?"]
    families = select(map_rows, FAMILIES, "Họ map F5?")
    if len(map_rows) != len(families):
        raise AuditError("Phạm vi họ map F5? đã đổi")
    systems = select((r for r in rows if r["kind"] == "system" and r["wave"] == "F5"),
                     (*ROADMAP_NAMES.values(), "Couple"), "Hệ thống F5")
    resources = select(catalog["subsystems"], ("Guild", "Couple"), "Catalog F5-0", key="id")
    notes, roadmap = (data[key].decode("utf-8") for key in ("social_notes", "roadmap"))
    added = coverage["supplement_2026_11_27_wiki"]["added"]
    if (coverage["authority"] != "project_owner"
            or not {"교환채팅방", "길드 마크/비공개 길드"}.issubset(added)
            or not all(f"| `{key}` |" in notes for key in resources)
            or "Schema 9" not in notes
            or not all(term in roadmap for term in ("광장", "quà khi ngồi quán cà phê", "EXP/VÀNG"))):
        raise AuditError("Các ghi chú nền hoặc phạm vi lộ trình F5 đã đổi")

    map_families = []
    for name, role in FAMILIES.items():
        row = families[name]
        counts = {key: row[key] for key in ("map_count", "script_count", "script_unknown")}
        if (row["category"] != "lobby" or row["base"] != name
                or any(type(count) is not int for count in counts.values())
                or not 0 <= counts["script_unknown"] <= counts["script_count"] <= counts["map_count"]
                or not counts["map_count"]):
            raise AuditError(f"Số liệu map hoặc script không hợp lệ: {name}")
        map_families.append({
            "name": name, "source": "kr_static", "status": row["status"],
            "wave": row["wave"], "classification": row["classification"], **counts,
            "role": role, "role_status": "candidate", "kr_wire": unknown_wire("Vào, rời và dùng map."),
        })

    def feature(key, *, wire, rules, schema_9, next_measurement, evidence):
        row = systems[ROADMAP_NAMES.get(key, key)]
        return {
            "id": key, "name": row["name"] if key != "Couple" else "Cặp đôi (커플)",
            "status": row["status"], "classification": row["classification"],
            "classification_reason": row["reason"], "source_refs": ["classification", *evidence],
            "kr_wire": unknown_wire(wire), "rules": rules, "schema_9": schema_9,
            "next_measurement": next_measurement,
        }

    cap = re.search(r"tối đa ([0-9]+) ngày", systems[ROADMAP_NAMES["Guild"]]["reason"])
    if cap is None:
        raise AuditError("Không tìm thấy cửa sổ duyệt bang kín trong bảng xếp loại")
    plaza = feature(
        "Plaza", wire="Chuyển cảnh, danh sách người, di chuyển, tương tác và quà cà phê.",
        rules={"cafe_gift": rule("unknown", "Chưa rõ thời gian ngồi, vật phẩm và điều kiện nhận lại. "
                                           "Chưa có bằng chứng về quà ở các khu khác.")},
        schema_9=storage("Giữ hiện diện và vị trí theo phiên; nếu có quà bền, cần lưu lần nhận "
                         "và chống cấp trùng khi ghi thưởng vào túi đồ."),
        next_measurement="Vào và rời từng họ; dùng hai client để thử thấy nhau di chuyển. "
                         "Đối chiếu ngồi và không ngồi ở quán cà phê để xác định đường nhận quà.",
        evidence=["roadmap"],
    )
    extended = [
        feature(
            "Mail", wire="Gửi, nhận, liệt kê, đọc, xóa và phản hồi khi hộp đầy.",
            rules={"inbox_cap": rule("unknown", "Có giới hạn hộp thư; chưa biết số thư tối đa "
                                               "và cách xử lý khi đầy.", unit="messages")},
            schema_9=storage("Lưu thư theo người gửi và người nhận, trạng thái đọc/xóa, "
                             "chỉ mục hộp thư và kiểm giới hạn trong giao dịch ghi.", "player_blocks"),
            next_measurement="Gửi cho người đang offline, kiểm sau đăng nhập lại; đo biên đầy, "
                             "thử gửi thêm và thử lại sau khi xóa một thư.",
            evidence=["roadmap"],
        ),
        feature(
            "ExchangeChat", wire="Tạo, tìm, tham gia, rời phòng và chuyển tin.",
            rules={"trade_binding": rule("unknown", "Chưa rõ phòng dùng CHOCO hay có giao dịch "
                                                   "đồ gắn với phòng.")},
            schema_9=storage("Thiết kế phòng, thành viên và hiện diện theo phiên. Chỉ xem xét lưu "
                             "giao dịch đồ sau khi xác nhận có luồng đổi đồ riêng.",
                             "player_blocks", "social_mutes", "chat_rate_windows"),
            next_measurement="Tạo và tham gia từ hai client; đo chat, rời phòng và thử "
                             "thao tác đổi đồ riêng để kiểm quan hệ với phòng chat.",
            evidence=["coverage", "roadmap"],
        ),
        feature(
            "Guild", wire="Xem/đổi huy hiệu, đổi quyền riêng tư, nộp đơn, duyệt, từ chối và hết hạn.",
            rules={"private_approval_max": rule(
                int(cap[1]), "Mốc tối đa theo bảng xếp loại; chưa biết lúc bắt đầu tính, "
                "biên hết hạn và hành vi khi hết hạn.", status="candidate", unit="days",
            )},
            schema_9=storage("Bổ sung tham chiếu huy hiệu, quyền hiển thị/gia nhập và đơn gia nhập "
                             "có thời hạn, quyết định và người duyệt. Duyệt và ghi thành viên "
                             "trong cùng giao dịch.", "guilds", "guild_members"),
            next_measurement="Đối chiếu đổi huy hiệu và nộp đơn bang kín với mời bang thường; "
                             "thử quyền duyệt, hết hạn và trạng thái thành viên sau đăng nhập lại.",
            evidence=["coverage", "social_catalog", "social_notes", "ida", "roadmap"],
        ),
        feature(
            "Couple", wire="Mời, chấp nhận, đồng bộ cặp, đổi trang bị và kết thúc quan hệ.",
            rules={
                "rings": rule("unknown", "Nhẫn đôi thuộc phạm vi khảo sát; chưa có ID, điều kiện "
                                         "trang bị hay quan hệ đã kiểm với Kim đôi trong lộ trình."),
                "exp_bonus": rule("unknown", "Lộ trình nêu thưởng EXP; chưa biết mức, điều kiện và cách cộng dồn."),
                "lucci_bonus": rule("unknown", "Lộ trình nêu thưởng Lucci (VÀNG); chưa biết mức, "
                                              "điều kiện và cách cộng dồn."),
            },
            schema_9=storage("Lưu lời mời và quan hệ cặp, tham chiếu nhẫn nếu đo được, và liên kết "
                             "thưởng EXP/Lucci với lần chốt kết quả để tránh ghi trùng."),
            next_measurement="Đo mời/chấp nhận/từ chối và đăng nhập lại; so hai ván có/không có "
                             "cặp hoặc trang bị để tách hiệu ứng nhẫn khỏi thưởng EXP/Lucci.",
            evidence=["social_catalog", "social_notes", "ida", "roadmap"],
        ),
    ]
    for feature_row in extended:
        key = feature_row["id"]
        if key in resources:
            feature_row["static_ida"] = static_anchors(f5, key.lower())
            feature_row["resource_groups"] = [
                {field: group[field] for field in ("entries", "idx_sha256", "locator")}
                for group in resources[key]["resource_groups"]
            ]

    tables = sorted(re.findall(rb"^CREATE TABLE ([a-z_]+) \(", data["migration"], re.M))
    tables = [table.decode("ascii") for table in tables]
    if any(not set(row["schema_9"]["reuse_candidates"]).issubset(tables) for row in extended):
        raise AuditError("Bảng tái dùng đề xuất không còn trong Schema 9")
    matrix = {
        "schema": "kre.f5-extended-social.v1", "build": BUILD, "client_sha256": client_hash,
        "source": "design", "status": "design",
        "limits": "Đây là kiểm kê offline. Neo IDA tĩnh và tài nguyên pack chưa xác nhận "
                  "luồng client, luật server hoặc payload KR; chưa có phép đo runtime mới.",
        "inputs": {key: {"path": path, "sha256": sha256(data[key])}
                   for key, path in INPUTS.items()},
        "ida": {"idb_sha256": grades["idb_sha256"], "cstage_dispatcher_va": f5["cstage_dispatcher_va"]},
        "schema_9": {"baseline_tables": tables, "extensions_status": "design"},
        "summary": {"map_families": len(map_families), "maps": sum(r["map_count"] for r in map_families),
                    "scripts": sum(r["script_count"] for r in map_families),
                    "scripts_unknown": sum(r["script_unknown"] for r in map_families),
                    "extended_systems": len(extended)},
        "map_families": map_families, "plaza": plaza, "systems": extended,
    }
    matrix["inputs"]["roadmap"]["locator"] = "#full-game-roadmap, dòng F5 (UTF-8 và LF cuối dòng)"
    matrix["matrix_sha256"] = sha256(json_bytes(matrix, compact=True))
    return matrix


def render_markdown(matrix):
    count = matrix["summary"]
    lines = [
        "# Ma trận hệ thống xã hội F5 mở rộng", "",
        f"Build `{matrix['build']}`: **{count['map_families']} họ, {count['maps']} map, "
        f"{count['extended_systems']} hệ thống mở rộng**. "
        f"Còn `unknown` về hành vi của {count['scripts_unknown']}/{count['scripts']} script.", "",
        matrix["limits"], "",
        "Nguồn: [bảng xếp loại](classification.json), [catalog JSON](../evidence/content/f5-social-catalog.json), "
        "[kiểm kê F5-0](f5-social-catalog.md), [neo IDA](../evidence/content/f2-f5-f3-ida-anchors.json), "
        "[chỉ đạo phủ nội dung](../evidence/owner-directives/full-content-coverage.json), "
        "[Schema 9](../../rust/crates/kr-service/migrations/0009_f5_social_foundation.sql) và "
        "[lộ trình F5](../PLAN.md#full-game-roadmap). "
        "Hash đầu vào và định danh bản client nằm trong [ma trận JSON](../evidence/content/f5-extended-social-matrix.json).",
        "", "## Các họ map quảng trường", "",
        "| Họ map | Đợt / lớp | Map | Script chưa rõ / tổng | Vai trò dự kiến | Opcode KR |",
        "|---|---|---:|---:|---|---|",
    ]
    for row in matrix["map_families"]:
        lines.append(f"| `{row['name']}` | `{row['wave']}` / {row['classification']} | {row['map_count']} | "
                     f"{row['script_unknown']}/{row['script_count']} | {row['role']} | "
                     f"`{row['kr_wire']['opcodes']}` |")
    lines += ["", "## Hành vi cần đo và phần lưu trữ cần bổ sung", "",
              "Schema 9 đã có bảng quan hệ xã hội, kiểm soát chat và thành viên bang. "
              "Các phần mở rộng dưới đây là đề xuất thiết kế và chưa có migration.", ""]
    for row in (matrix["plaza"], *matrix["systems"]):
        lines += [f"### {row['name']}", "", row["classification_reason"], "",
                  f"Wire `{row['kr_wire']['opcodes']}`: {row['kr_wire']['unmeasured']}", ""]
        if "static_ida" in row:
            anchors = row["static_ida"]["anchors"]
            labels = "; ".join(f"`{r['direction']} {r['opcode']}` ({', '.join(r['vas'])})" for r in anchors)
            lines += [f"Neo IDA `kr_static/confirmed`: {labels}. "
                      "Các neo không xác nhận wire của toàn bộ thao tác mở rộng.", ""]
        for gap in row["rules"].values():
            lines.append(f"- `{gap['status']}`: {gap['note']}")
        reuse = row["schema_9"]["reuse_candidates"]
        suffix = " Có thể dùng lại " + ", ".join(f"`{table}`" for table in reuse) + "." if reuse else ""
        lines += ["", f"Mở rộng Schema 9 (`design`): {row['schema_9']['extension']}{suffix}", "",
                  f"Phép đo đề xuất trong mạng cô lập: {row['next_measurement']}", ""]
    lines += [
        "## Tái lập offline", "",
        "```bash", "python3 tools/kr_f5_extended_social_audit.py --write",
        "python3 tools/kr_f5_extended_social_audit.py --check", "```", "",
        f"SHA-256 của JSON chuẩn hóa, bỏ trường `matrix_sha256`: `{matrix['matrix_sha256']}`.", "",
    ]
    return "\n".join(lines).encode("utf-8")


def audit(root, *, write):
    matrix = build_matrix(read_inputs(root))
    outputs = {MATRIX: json_bytes(matrix), MARKDOWN: render_markdown(matrix)}
    paths = {name: repo_file(root, name) for name in outputs}
    for name, expected in outputs.items():
        reject_private(expected, name)
    if write:
        for name, expected in outputs.items():
            paths[name].write_bytes(expected)
    else:
        for name, expected in outputs.items():
            actual = paths[name].read_bytes()
            reject_private(actual, name)
            if actual != expected:
                raise AuditError(f"{name}: không khớp bản tái sinh; chạy lại với --write")
    return matrix


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--write", action="store_true", help="sinh JSON và Markdown trong worktree")
    mode.add_argument("--check", action="store_true", help="kiểm tra nội dung và rò rỉ dữ liệu riêng")
    args = parser.parse_args(argv)
    try:
        matrix = audit(ROOT, write=args.write)
    except AuditError as exc:
        parser.exit(1, f"F5: {exc}\n")
    except (OSError, ValueError, KeyError, TypeError):
        parser.exit(1, "F5: không đọc được dữ liệu kiểm kê hoặc đầu ra\n")
    print(f"F5 OK: {matrix['summary']['map_families']} họ map; SHA-256 {matrix['matrix_sha256']}")


if __name__ == "__main__":
    main()
