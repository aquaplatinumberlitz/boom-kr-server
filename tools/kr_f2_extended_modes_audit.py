"""Sinh và kiểm ma trận thiết kế F2 mở rộng từ các file trong repo."""
import argparse
from collections import Counter, defaultdict
import hashlib
import ipaddress
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
CLASSIFICATION = "docs/content/classification.json"
CENSUS = "docs/content/census.json"
CATALOG = "docs/evidence/content/f2-mode-catalog.json"
DIRECTIVE = "docs/evidence/owner-directives/full-content-coverage.json"
RULES = "rust/crates/kr-service/src/room_modes/rules.json"
ANCHORS = "docs/evidence/content/f2-f5-f3-ida-anchors.json"
PLAN = "docs/PLAN.md"
JSON_INPUTS = (CLASSIFICATION, CENSUS, CATALOG, DIRECTIVE, RULES, ANCHORS)
JSON_OUTPUT = "docs/evidence/content/f2-extended-modes-matrix.json"
MD_OUTPUT = "docs/content/f2-extended-modes-matrix.md"
SCHEMA = "kre.f2-extended-modes.v1"

INVARIANTS = (
    "Đưa luật mới vào RoomRules hoặc policy do RoomRules dùng; không thêm nhánh "
    "`capacity() == N`, cờ theo sức chứa hay alias theo identity vào lõi phòng.",
    "Tách số điểm spawn trong pack, sức chứa roster và hình thức thi đấu; "
    "không suy ra giá trị nào từ hai giá trị còn lại.",
    "Giữ wire và quyền quyết định chưa đo ở mức unknown; không dùng mặc định trận thường "
    "hoặc ID giao diện để điền chỗ trống.",
    "Kiểm tra quyền và chuyển trạng thái ở server theo policy; giữ luật sảnh, kênh "
    "và giải đấu ở lớp chọn policy trước khi tạo phòng.",
)

# Tên khớp bảng xếp loại; slot và phép đo là đề xuất thiết kế.
SYSTEM_SPECS = {
    "Đấu loạn 12 người (대난투)": (
        "big_brawl", "RoomRules.roster + map_policy + result",
        "Đo chọn chế độ, roster 12 người, map được phép và cách chốt điểm cá nhân."
    ),
    "Xếp hạng 1:1 theo mùa (크아짱)": (
        "ca_jjang", "RoomRules.roster + result; RankingSeason",
        "Đo ghép 1:1, giới hạn 1P và chuyển mùa; xác định nơi chốt điểm và bảng xếp hạng."
    ),
    "Luật phòng (노템전, 노샵, 기샵)": (
        "room_item_rules", "RoomRules.item_policy",
        "Đo từng preset khi tạo hoặc vào phòng; tách quyền mang đồ, dùng đồ và nhặt đồ trong trận."
    ),
    "Phòng ưu tiên (슈프림, 앞방 잡기)": (
        "supreme_rooms", "RoomRules.admission_policy; LobbyRoomDirectory",
        "Đo quyền đặt chỗ S01–S50 theo kênh, cách giữ và giải phóng chỗ."
    ),
    "Bỏ phiếu đuổi khỏi phòng (강퇴투표)": (
        "vote_kick", "RoomRules.moderation_policy",
        "Đo người có quyền mở và bỏ phiếu, điều kiện thông qua, thời hạn và quyền chống đuổi."
    ),
    "Siêu chủ phòng (슈퍼방장)": (
        "super_host", "RoomRules.host_permissions",
        "Đo nguồn cấp quyền, điều kiện bắt đầu ngay và việc chuyển quyền khi chủ rời phòng."
    ),
    "Kênh theo loại (채널)": (
        "channel_types", "RoomRules.admission_policy + item_policy; ChannelPolicy",
        "Đo điều kiện vào từng loại kênh và preset chuyển xuống phòng."
    ),
    "Giải đấu (토너먼트)": (
        "tournaments", "RoomRules.roster + result; TournamentPolicy",
        "Đo đường vào kênh luyện tập và giải, điều kiện dự giải, tiến vòng và chốt quà."
    ),
}

PRIVATE_TEXT = re.compile(
    r"/home/|/Users/|[a-z]:[\\/]+Users[\\/]|\bocid" r"1\.|"
    r"BEGIN [A-Z0-9 ]*PRIVATE" r" KEY|"
    r"\b(?:gh[pousr]_[a-z0-9]{20,}|github_pat_[a-z0-9_]{20,}|"
    r"AKIA[A-Z0-9]{16}|xox[baprs]-[a-z0-9-]{10,})\b|"
    r"\b(?:authorization|password|passwd|api[_-]?key|access[_-]?token|"
    r"client[_-]?secret|secret|token)[\"']?\s*[:=]\s*[\"']?[^\s\"'`|,}]+",
    re.IGNORECASE,
)
IP_LITERAL = re.compile(
    r"(?<![\w:])[0-9a-f]*(?::[0-9a-f]*){2,}(?:\.\d{1,3}){0,3}(?![\w:])|"
    r"(?<![\w.])(?:\d{1,3}\.){3}\d{1,3}(?!\w|\.\d)",
    re.IGNORECASE,
)


def sha256(raw):
    return hashlib.sha256(raw).hexdigest()


def reject_private(raw, path):
    body = raw.decode("utf-8")
    if PRIVATE_TEXT.search(body):
        raise ValueError(f"{path}: có dấu hiệu đường dẫn máy, định danh hoặc thông tin xác thực riêng")
    for match in IP_LITERAL.finditer(body):
        try:
            address = ipaddress.ip_address(match[0])
        except ValueError:
            continue
        if not address.is_global:
            raise ValueError(f"{path}: có địa chỉ IP nội bộ")


def require(condition, message):
    if not condition:
        raise ValueError(message)


def slot(name):
    return {"slot": name, "source": "design", "status": "design"}


def build_matrix(data, input_sha256, plan):
    classification, census, catalog, directive, rules, anchors = (data[path] for path in JSON_INPUTS)
    artifacts = anchors["source_artifacts"]
    client_hash = classification["client_sha256"]
    require(classification["build"] == census["build"] == catalog["build"] == "KR00967"
            and client_hash == census["client_exe_sha256"]
            == catalog["static_metadata"]["client_exe_sha256"] == artifacts["kr_ca_exe_sha256"]
            and classification["idb_sha256"] == artifacts["kr_full_i64_sha256"],
            "Nguồn không cùng client hoặc DB KR00967")
    require(classification["census_sha256"] == catalog["input_sha256"]["census"] == input_sha256[CENSUS],
            "Bảng xếp loại hoặc danh mục F2 không khớp census")
    require(directive["authority"] == "project_owner", "Thiếu chỉ đạo mở rộng phạm vi của chủ dự án")

    tentative = [row for row in classification["rows"] if row["kind"] == "map" and row["wave"] == "F2?"]
    target = [row for row in tentative if row["category"] == "other"]
    mission = {row["name"]: row["map_count"] for row in tentative if row["category"] == "mission"}
    require(len(target) == len({row["name"] for row in target}) == 21
            and len(tentative) == len(target) + len(mission),
            "Phạm vi F2? phải gồm 21 họ mới và các họ nhiệm vụ đã kiểm kê")
    require(mission == {row["family"]: row["map_count"] for row in catalog["families"]},
            "Các họ nhiệm vụ F2? phải thuộc danh mục F2 hiện có")

    rules_by_group = {row["group"]: row for row in rules}
    targets = {row["name"] for row in target}
    by_family = defaultdict(list)
    related = defaultdict(list)
    for row in census["maps"]:
        if row["category"] == "other" and row["family"] in targets:
            by_family[row["family"]].append(row)
    for mode in catalog["modes"]:
        for row in mode["related_families"]:
            if row["family"] in targets:
                related[row["family"]].append((mode["ruleset_group"], row["map_count"]))

    families = []
    for row in sorted(target, key=lambda row: row["name"]):
        name = row["name"]
        maps = by_family[name]
        require(len(maps) == row["map_count"] and sum(m["scripted"] for m in maps) == row["script_count"],
                "Số map hoặc script không khớp bảng xếp loại")
        grids = Counter((m["width"], m["height"]) for m in maps)
        spawns = Counter(m["spawns"] for m in maps)
        scripts = Counter("unknown" if m["parse"] != "exact" else "yes" if m["scripted"] else "no"
                          for m in maps)
        for group, count in related[name]:
            require(group in rules_by_group and count == len(maps),
                    "Liên kết họ map trong danh mục F2 không khớp số map hoặc bảng ModeRules")
        families.append({
            "family": name,
            "category": row["category"],
            "wave": row["wave"],
            "classification": row["classification"],
            "source": "kr_static",
            "status": row["status"],
            "map_count": len(maps),
            "spawn_counts": [{"spawns": n, "maps": count} for n, count in sorted(spawns.items())],
            "grid_sizes": [{"width": w, "height": h, "maps": count} for (w, h), count in sorted(grids.items())],
            "grid_class": "standard_15x13" if set(grids) == {(15, 13)} else
            "mixed" if (15, 13) in grids else "nonstandard",
            "script_count": row["script_count"],
            "script_unknown": row["script_unknown"],
            "script_presence": {key: scripts[key] for key in ("yes", "no", "unknown")},
            "room_rules": slot("RoomRules.map_policy"),
            "catalog_group_candidates": sorted({group for group, _ in related[name]}),
            "wire": "unknown",
        })

    roadmap = [row for row in classification["rows"]
               if row["kind"] == "system" and row["wave"] == "F2" and row.get("roadmap_only")]
    require(len(roadmap) == len(SYSTEM_SPECS) and {row["name"] for row in roadmap} == SYSTEM_SPECS.keys(),
            "Danh sách roadmap_only F2 đổi; cần đối chiếu slot đề xuất")
    systems = []
    for row in roadmap:
        key, design, next_check = SYSTEM_SPECS[row["name"]]
        systems.append({
            "id": key, "name": row["name"], "wave": row["wave"], "roadmap_only": True,
            "classification": row["classification"], "status": row["status"],
            "source": "design", "locator": CLASSIFICATION,
            "scope": row["reason"], "room_rules": slot(design), "next_check": next_check, "wire": "unknown",
        })
    require(any(line.startswith("| F2 ") and "map thưởng ladder" in line for line in plan.splitlines())
            and "ladder" in rules_by_group, "Thiếu mục map thưởng Ladder hoặc nhóm luật ladder")
    systems.append({
        "id": "ladder_bonus_map", "name": "Map thưởng Ladder", "wave": "F2", "classification": "C",
        "source": "design", "status": "candidate", "locator": PLAN + "#full-game-roadmap",
        "scope": "Mục riêng trong lộ trình F2. Chưa biết map, điều kiện xuất hiện hoặc phần thưởng.",
        "room_rules": slot("RoomRules.map_policy + result; LadderPolicy"),
        "next_check": "Đo cách client nhận lựa chọn map thưởng và điều kiện chuyển sang map đó; "
                      "đối chiếu kết quả và thưởng với một ván Ladder thường.",
        "wire": "unknown",
    })

    f2 = anchors["f2_modes"]
    matrix = {
        "schema": SCHEMA,
        "build": classification["build"],
        "source_artifacts": {key: artifacts[key] for key in (
            "kr_ca_exe_sha256", "kr_full_i64_sha256", "image_base"
        )},
        "input_sha256": input_sha256,
        "map_metadata": {
            "classification_locator": CLASSIFICATION + "#rows",
            "pack_locator": CENSUS + "#maps",
            "script_unknown_meaning": "Số map có tag ngoài mapscript và StageClearScript; cần đo quyền thực thi.",
            "script_presence_unknown_meaning": "Số map có parse khác exact.",
        },
        "summary": {
            "map_families": len(families),
            "maps": sum(row["map_count"] for row in families),
            "existing_mission_families_excluded": len(mission),
            "roadmap_only_systems": len(roadmap),
            "extended_systems": len(systems),
        },
        "invariants": INVARIANTS,
        "ida_context": {
            "locator": ANCHORS + "#f2_modes",
            "source": "kr_static", "status": "confirmed",
            "game_type_switch_va": f2["game_type_switch_va"],
            "map_group_visible": f2["map_group_visible"],
            "rtti_packet_namespaces": f2["rtti_packet_namespaces"],
            "limit": "Các neo chỉ dẫn đường kiểm; chưa gắn ID wire hoặc quyền quyết định cho các dòng trong ma trận.",
        },
        "map_families": families,
        "systems": sorted(systems, key=lambda row: row["id"]),
    }
    canonical = json.dumps(matrix, ensure_ascii=False, sort_keys=True, separators=(",", ":"), allow_nan=False)
    matrix["matrix_sha256"] = sha256((canonical + "\n").encode("utf-8"))
    return matrix


def markdown(matrix):
    summary = matrix["summary"]
    lines = [
        "# Ma trận thiết kế F2 mở rộng — KR 00967",
        "",
        f"Phạm vi gồm **{summary['map_families']} họ / {summary['maps']} map** mới được xếp tạm "
        f"`F2?` trong PR #713, cùng **{summary['extended_systems']} mục mở rộng**. "
        f"{summary['existing_mission_families_excluded']} họ nhiệm vụ cũng có nhãn `F2?` "
        "và đã có trong [danh mục F2](f2-mode-catalog.md).",
        "",
        "Nguồn phạm vi là [bảng xếp loại](classification.md) và "
        "[chỉ đạo bao phủ nội dung](../evidence/owner-directives/full-content-coverage.json). "
        "Số liệu map lấy từ [census đã lưu](census.json). "
        "Hash của đầu vào và build nằm trong [ma trận JSON](../evidence/content/f2-extended-modes-matrix.json).",
        "",
        "## Bất biến thiết kế",
        "",
        *(f"- {line}" for line in matrix["invariants"]),
        "",
        "## Họ map xếp tạm `F2?`",
        "",
        "Tất cả các họ dưới đây ở lớp C, trạng thái `candidate`. "
        "Số trong ngoặc là số map ở mỗi mức spawn hoặc kích thước lưới. "
        "Cột script ghi số map **có / không / chưa đọc đủ**; cột cần đo đếm map có tag "
        "ngoài `mapscript` và `StageClearScript`, chưa xác định quyền thực thi.",
        "",
        "Slot `RoomRules.map_policy` là đề xuất cho dữ liệu lưới, layout spawn và cổng thực thi script. "
        "Nhóm có tên là liên kết ứng viên theo danh mục và "
        "[bảng ModeRules](../../rust/crates/kr-service/src/room_modes/rules.json). "
        "Chưa có liên kết ứng viên thì đo chọn map và tạo phòng trước khi gán nhóm.",
        "",
        "| Họ | Map | Spawn (số map) | Lưới (số map) | Script | Cần đo script | Slot RoomRules | Nhóm ứng viên | Wire |",
        "|---|---:|---|---|---|---:|---|---|---|",
    ]
    grid_class = {"standard_15x13": "chuẩn", "mixed": "hỗn hợp", "nonstandard": "ngoài chuẩn"}
    for row in matrix["map_families"]:
        spawns = ", ".join(f"{v['spawns']} ({v['maps']})" for v in row["spawn_counts"])
        grids = ", ".join(f"{v['width']}×{v['height']} ({v['maps']})" for v in row["grid_sizes"])
        scripts = " / ".join(str(row["script_presence"][key]) for key in ("yes", "no", "unknown"))
        groups = ", ".join(f"`{group}`" for group in row["catalog_group_candidates"]) or "—"
        lines.append(f"| `{row['family']}` | {row['map_count']} | {spawns} | {grids}; "
                     f"{grid_class[row['grid_class']]} | {scripts} | {row['script_unknown']} | "
                     f"`{row['room_rules']['slot']}` | {groups} | `{row['wire']}` |")

    lines.extend([
        "",
        "## Hệ thống F2 mở rộng",
        "",
        "Tám mục có cờ `roadmap_only` trong bảng xếp loại. "
        "Map thưởng Ladder nằm riêng trong [lộ trình F2](../PLAN.md#full-game-roadmap). "
        "Các slot mô tả nơi gắn policy khi có đủ bằng chứng. "
        "Chưa có bằng chứng gắn Big Brawl 12P với nhóm pack `big_battle` "
        "hoặc gắn map thưởng với một map cụ thể trong LadderMapGroup.",
        "",
        "| Hệ thống | Phạm vi ứng viên | Slot thiết kế | Wire | Phép đo còn thiếu |",
        "|---|---|---|---|---|",
    ])
    for row in matrix["systems"]:
        fields = (row["name"], row["scope"], f"`{row['room_rules']['slot']}`",
                  f"`{row['wire']}`", row["next_check"])
        lines.append("| " + " | ".join(value.replace("|", "&#124;") for value in fields) + " |")
    lines.extend([
        "",
        "## Giới hạn bằng chứng và cách kiểm",
        "",
        "[Neo IDA F2](../protocol/f2-f5-f3-ida-anchors.md) cung cấp hướng lần từ "
        "`MapGroupVisible`, switch `GameType` và RTTI. "
        "Cần đo client KR trong mạng cô lập, giữ đối chứng khi đổi từng lựa chọn, "
        "rồi xác định payload, bên phát sự kiện, quyền chốt kết quả và bước reset khi phù hợp. "
        "Ma trận giữ toàn bộ wire của các dòng ở `unknown`.",
        "",
        f"SHA-256 nội dung JSON: `{matrix['matrix_sha256']}`. Băm UTF-8 sau khi bỏ trường "
        "`matrix_sha256`, sắp khóa tăng, dùng dấu phân cách JSON không kèm khoảng trắng và thêm một LF cuối.",
        "",
        "```bash",
        "python3 tools/kr_f2_extended_modes_audit.py --write",
        "python3 tools/kr_f2_extended_modes_audit.py --check",
        "```",
        "",
        "Hai lệnh chỉ dùng file trong repo. `--check` kiểm SHA-256 theo byte của cả hai đầu ra "
        "và rà dấu hiệu lộ đường dẫn máy, IP nội bộ hoặc thông tin xác thực.",
        "",
    ])
    return "\n".join(lines)


def render_outputs(root):
    sources = {path: (root / path).read_bytes() for path in (*JSON_INPUTS, PLAN)}
    matrix = build_matrix({path: json.loads(sources[path]) for path in JSON_INPUTS},
                          {path: sha256(raw) for path, raw in sources.items()},
                          sources[PLAN].decode("utf-8"))
    return {
        JSON_OUTPUT: (json.dumps(matrix, ensure_ascii=False, sort_keys=True, indent=2) + "\n").encode("utf-8"),
        MD_OUTPUT: markdown(matrix).encode("utf-8"),
    }


def run(root, *, check):
    outputs = render_outputs(root)
    digests = {path: sha256(raw) for path, raw in outputs.items()}
    for path, raw in outputs.items():
        reject_private(raw, path)
    if check:
        current = {path: (root / path).read_bytes() for path in outputs}
        for path, raw in current.items():
            reject_private(raw, path)
        stale = [path for path, raw in current.items() if sha256(raw) != digests[path]]
        require(not stale, "Chưa đồng bộ; chạy lại --write: " + ", ".join(stale))
    else:
        for path, raw in outputs.items():
            (root / path).write_bytes(raw)
    return digests


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    action = parser.add_mutually_exclusive_group(required=True)
    action.add_argument("--write", action="store_true", help="Ghi JSON và Markdown")
    action.add_argument("--check", action="store_true", help="Kiểm SHA-256 và nội dung riêng tư")
    args = parser.parse_args()
    try:
        digests = run(ROOT, check=args.check)
    except OSError as exc:
        parser.exit(1, f"Không thể đọc hoặc ghi ma trận F2: {exc.strerror}\n")
    except (ValueError, KeyError, TypeError) as exc:
        parser.exit(1, f"Không thể kiểm kê F2: {exc}\n")
    for path, digest in digests.items():
        print(f"{digest}  {path}")


if __name__ == "__main__":
    main()
