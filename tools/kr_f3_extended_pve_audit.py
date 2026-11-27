"""Sinh hoặc kiểm ma trận F3 mở rộng từ metadata đã lưu trong repo."""

import argparse
from collections import Counter, defaultdict
import hashlib
import ipaddress
import json
from pathlib import Path
import re


ROOT = Path(__file__).resolve().parents[1]
SCHEMA = "kre.f3-extended-pve.v1"
BUILD = "KR00967"
INPUTS = {
    "classification": "docs/content/classification.json",
    "census": "docs/content/census.json",
    "content": "docs/CONTENT.md",
    "catalog": "docs/evidence/content/f3-pve-catalog.json",
    "authority_matrix": "docs/evidence/content/f3-script-authority-matrix.json",
    "coverage_directive": "docs/evidence/owner-directives/full-content-coverage.json",
    "ida": "docs/evidence/content/f2-f5-f3-ida-anchors.json",
    "script_ida": "docs/evidence/content/f1e-dynamic-scripts-ida.json",
    "roadmap": "docs/PLAN.md",
}
JSON_OUT = "docs/evidence/content/f3-extended-pve-matrix.json"
MD_OUT = "docs/content/f3-extended-pve-matrix.md"

# Đây là câu hỏi và đối chứng đề xuất, không phải luật rút ra từ tên map.
FAMILY_PROBES = {
    "Antarctic": ("Điều gì kích hoạt đường chạy và qua màn?",
                  "So lần vượt mốc có trigger với lượt dừng trước mốc."),
    "DragonPalace": ("Điều kiện chết hoặc bị khóa có đổi kết quả màn không?",
                     "So cùng đoạn đường với và không có trạng thái bị khóa."),
    "EscapeMaze": ("Ai xác nhận lấy chìa, mở cửa và thoát mê cung?",
                   "Thử cùng cửa khi có và khi thiếu chìa; theo dõi cả hồi sinh."),
    "GolemRun": ("Ai kích hoạt đá rơi và điều kiện qua màn?",
                 "So lượt đi qua vùng có dấu trigger với lượt không vào vùng."),
    "IamWriggle": ("Cửa vào GameType 43 có tải đúng họ map và áp dụng cấu hình vật phẩm không?",
                   "So trạng thái vật phẩm trước và sau thao tác trong mode thật."),
    "MineExp": ("Ai kích hoạt laser và ghi nhận sát thương?",
                "So một lượt tiếp xúc laser với một lượt tránh được."),
    "Minibot": ("Ai cập nhật trạng thái Minibot tại trạm sửa?",
                "So cùng trạng thái trước và sau khi vào trạm với lượt đứng ngoài."),
    "SavetheEarth": ("Ai quyết định bắt mục tiêu hoặc cứu máy bay?",
                     "So lần hoàn tất mục tiêu với lượt thiếu điều kiện cuối."),
}

SYSTEMS = {
    "ca_hero": {
        "label": "CA Hero",
        "scope_text": "CA Hero (máu, kỹ năng, trang bị, thuốc, nhiệm vụ)",
        "features": {"hp": "HP", "skills": "Kỹ năng", "gear": "Trang bị",
                     "potions": "Thuốc", "quests": "Nhiệm vụ"},
        "mode_rows": ("크아 히어로 (CA Hero)",),
        "question": "Ai xác nhận thay đổi HP, hiệu ứng kỹ năng/trang bị, tiêu hao thuốc và tiến độ nhiệm vụ?",
        "probe": "So hai cấu hình trang bị, trúng/hụt và dùng/không dùng thuốc; kiểm nhiệm vụ trước, sau và sau đăng nhập lại.",
    },
    "ai_battle": {
        "label": "AIBattle (협공배틀)",
        "scope_text": "AIBattle (độ khó theo sao, mở nhân vật ẩn bằng điểm)",
        "features": {"star_difficulty": "Độ khó theo sao",
                     "hidden_character_unlock": "Nhân vật ẩn mở bằng điểm (협공배틀 포인트, 로두마니)"},
        "mode_rows": ("AIBattle",),
        "game_types": (16,),
        "question": "Mức sao đổi AI thế nào, điểm được tính ở đâu, ngưỡng mở là gì và khi nào 로두마니 xuất hiện?",
        "probe": "So hai mức sao trên cùng map; đối chiếu điểm và danh sách nhân vật trước/sau kết quả và sau đăng nhập lại.",
    },
    "hidden_catch": {
        "label": "Hidden Catch (히든 캐치)",
        "scope_text": "Hidden Catch (thường, thi đấu, chế độ máy xu 100 màn)",
        "features": {"standard": "Chế độ thường", "versus": "Chế độ thi đấu (vs)",
                     "arcade": "Chế độ máy xu, 100 màn theo lộ trình"},
        "mode_rows": ("히든 캐치 (Hidden Catch)",),
        "game_types": (28, 29),
        "question": "Ai xác nhận lựa chọn, điểm và kết quả trong từng mode? Đường vào, thứ tự và số màn máy xu là gì?",
        "probe": "So lựa chọn đúng/sai và lần qua màn ở chế độ thường và thi đấu; nếu mở được máy xu, đo luồng vào và chuyển màn riêng.",
    },
    "monster_season_2": {
        "label": "Monster mùa 2 và 몬스터대작전",
        "scope_text": "Monster mùa 2 dạng hành động, 몬스터대작전",
        "features": {"season_2_action": "Monster mùa 2 dạng hành động",
                     "monster_operation": "몬스터대작전"},
        "mode_rows": ("Monster mùa 2",),
        "question": "Ai quyết định AI, đợt quái, sát thương, qua màn và thưởng? 몬스터대작전 dùng cửa vào và map nào?",
        "probe": "Đo mẫu MonS2 với đối chứng đánh hụt và thiếu điều kiện qua màn; tìm cửa vào 몬스터대작전 riêng trước khi nối map.",
    },
}

# Chỉ sao chép trường cần cho báo cáo; dò thêm mẫu dữ liệu riêng trước khi ghi/kiểm.
PRIVATE_TEXT = re.compile(
    r"/(?:home|users)/|[a-z]:[\\/]+users[\\/]|ocid" r"1\.|\.internal\b|"
    r"\bBEGIN [A-Z0-9 ]*PRIVATE KEY\b|\b(?:gh[pousr]_|github_pat_|sk-(?:proj-)?)[A-Za-z0-9_-]{16,}|"
    r"\b(?:password|passwd|client_secret|api_key|access_token|secret)\b[\x22\x27]?\s*[:=]"
    r"|\bBearer\s+[A-Za-z0-9._~-]{12,}|://[^/\s:@]+:[^/\s@]+@",
    re.IGNORECASE,
)
ADDRESS = re.compile(r"(?<![\w.])(?:[0-9]{1,3}\.){3}[0-9]{1,3}(?![\w.])"
                     r"|(?<![\w:])[a-f0-9:]*:[a-f0-9:.]+(?![\w:])", re.IGNORECASE)


def require(condition, message):
    if not condition:
        raise ValueError(message)


def sha256(raw):
    return hashlib.sha256(raw).hexdigest()


def canonical_hash(data, *exclude):
    value = {key: item for key, item in data.items() if key not in exclude}
    raw = json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":"), allow_nan=False)
    return sha256((raw + "\n").encode("utf-8"))


def reject_private(text, locator):
    error = f"{locator}: phát hiện mẫu dữ liệu riêng"
    require(not PRIVATE_TEXT.search(text), error)
    for match in ADDRESS.finditer(text):
        try:
            ipaddress.ip_address(match[0])
        except ValueError:
            continue
        raise ValueError(error)


def scan_output(raw, locator):
    text = raw.decode("utf-8")
    reject_private(text, locator)
    if locator == JSON_OUT:
        # Quét cả chuỗi JSON đã giải mã để bắt đường dẫn hoặc IP bị escape.
        decoded = json.dumps(json.loads(text), ensure_ascii=False).replace(r"\\", "/").replace(r"\/", "/")
        reject_private(decoded, locator)


def repo_path(root, locator):
    path = root / locator
    require(path.resolve() == path, f"{locator}: không dùng liên kết đến file khác")
    return path


def load_inputs(root):
    data, sources = {}, {}
    for key, locator in INPUTS.items():
        raw = repo_path(root, locator).read_bytes()
        data[key] = json.loads(raw) if locator.endswith(".json") else raw.decode("utf-8")
        sources[key] = {"locator": locator, "sha256": sha256(raw)}
    return data, sources


def validate_sources(data, sources):
    census, grades, catalog, matrix = (data[key] for key in
                                      ("census", "classification", "catalog", "authority_matrix"))
    require(all(row["build"] == BUILD for row in (census, grades, catalog, matrix)), "sai build KR00967")
    artifacts = data["ida"]["source_artifacts"]
    require(census["client_exe_sha256"] == grades["client_sha256"] == artifacts["kr_ca_exe_sha256"]
            == catalog["static_metadata"]["client_exe_sha256"] == matrix["static_anchors"]["client_exe_sha256"]
            and grades["idb_sha256"] == artifacts["kr_full_i64_sha256"] == matrix["static_anchors"]["idb_sha256"],
            "hash client hoặc IDB không khớp")
    require(grades["census_sha256"] == catalog["input_sha256"]["census"] == sources["census"]["sha256"],
            "hash census không khớp")
    # F3-0/F3-1 cùng pin bản xếp loại cũ. Đối chiếu với bản hiện hành theo phạm vi.
    require(matrix["inputs"]["classification"]["sha256"] == catalog["input_sha256"]["classification"],
            "F3-0 và F3-1 dùng bản xếp loại khác nhau")
    for old, current in (("census", "census"), ("f3_pve_catalog", "catalog"), ("ida_anchors", "ida")):
        require(matrix["inputs"][old] == sources[current], f"nguồn F3-1 không khớp: {current}")
    require(catalog["catalog_sha256"] == canonical_hash(catalog, "catalog_sha256", "input_sha256")
            and matrix["matrix_sha256"] == canonical_hash(matrix, "matrix_sha256"),
            "hash nội dung F3-0 hoặc F3-1 không khớp")
    directive = data["coverage_directive"]
    require(directive["authority"] == "project_owner" and directive["plan"] == INPUTS["roadmap"] + "#full-game-roadmap"
            and "협공배틀 포인트/로두마니" in directive["supplement_2026_11_27_wiki"]["added"],
            "thiếu căn cứ phạm vi trong quyết định của chủ dự án")


def index(rows, key):
    result = {}
    for row in rows:
        value = key(row)
        require(value not in result, "khóa trùng trong metadata nguồn")
        result[value] = row
    return result


def authority(question, probe):
    return {"authority_status": "unknown", "kr_wire_route": "unknown",
            "question": question, "probe": probe}


def static_anchors(data):
    ida = data["ida"]
    f3 = ida["f3_pve"]
    source = INPUTS["ida"]
    require(f3["status"] == ida["f2_modes"]["status"] == "kr_static/confirmed", "thiếu xác nhận tĩnh IDA")
    anchors = {
        "script_factory": {"va": f3["script_factory_va"], "controller_range": f3["script_controller_range"]},
        "monster_attack_guard": {key: f3["monster_attack_authority"][key] for key in
                                 ("handlers", "host_flag_fn", "host_flag_expr", "notes")},
    }
    prior = data["authority_matrix"]["static_anchors"]["anchors"]
    require(prior["script_factory"]["va"] == anchors["script_factory"]["va"], "neo factory F3-1 không khớp")
    handlers = index(f3["cgamestage_handlers"], lambda row: row["opcode"])
    for name, anchor in prior.items():
        if "opcode" in anchor:
            case = anchor["opcode"]
            handler = handlers[case]
            require(anchor["handler_va"] == handler["handler_va"], "neo handler F3-1 không khớp")
            anchors[name] = {"hexrays_case": case, **{key: val for key, val in handler.items() if key != "opcode"}}
    for name in anchors:
        anchors[name].update(locator=source + "#f3_pve", source="kr_static", status="confirmed")

    mode_switch = ida["f2_modes"]
    for game_type in (16, 28, 29, 43):
        anchors[f"game_type_{game_type}"] = {
            "source": "kr_static", "status": "confirmed", "locator": source + "#f2_modes",
            "va": mode_switch["game_type_switch_va"], "client_game_type": game_type,
            "symbol": mode_switch["game_type_ids"][str(game_type)],
        }

    script_ida = data["script_ida"]
    factory = script_ida["factories"]["map_script_factory"]
    require(script_ida["source"] == "kr_static" and factory["status"] == "confirmed"
            and factory["va"].lower() == f3["script_factory_va"], "thiếu đối chiếu factory controller")
    by_tag = {}
    # Time cũng xuất hiện như trường dữ liệu phổ thông; tên đó không đủ để nối lớp.
    for entry in factory["entries"]:
        if entry["tag"] and entry["tag"].endswith("script"):
            tag = entry["tag"].casefold()
            by_tag[tag] = f"controller_{tag}"
            anchors[by_tag[tag]] = {
                **entry, "source": "kr_static", "status": "confirmed",
                "binary_sha256": script_ida["binary_sha256"],
                "locator": INPUTS["script_ida"] + "#map_script_factory",
            }
    return anchors, by_tag


def map_families(data, controllers):
    grades = index((row for row in data["classification"]["rows"]
                    if row["kind"] == "map" and row["wave"] == "F3?"), lambda row: row["name"])
    require(grades.keys() == FAMILY_PROBES.keys(), "phạm vi tám họ map F3? đã đổi")
    previous = {row["family"] for row in data["catalog"]["families"]
                + data["authority_matrix"]["families"]}
    require(not grades.keys() & previous, "họ F3? đã xuất hiện trong danh mục F3-0/F3-1")
    census = defaultdict(list)
    for row in index(data["census"]["maps"], lambda row: row["name"]).values():
        if row["family"] in grades:
            census[row["family"]].append(row)
    content_rows = defaultdict(list)
    for line in data["content"].splitlines():
        if line.startswith("|"):
            cells = [cell.strip() for cell in line.split("|")[1:-1]]
            if len(cells) == 9 and cells[0] in grades:
                content_rows[cells[0]].append(cells[1:5])

    families = []
    for name, grade in sorted(grades.items()):
        rows = sorted(census[name], key=lambda row: row["name"])
        require(all(row["parse"] == "exact" and row["category"] == grade["category"] for row in rows),
                f"{name}: parse hoặc nhóm census đã đổi")
        count = len(rows)
        scripted = sum(row["scripted"] for row in rows)
        complex_script = sum(bool(set(row["script_tags"]) - {"mapscript", "StageClearScript"}) for row in rows)
        require((count, scripted, complex_script)
                == (grade["map_count"], grade["script_count"], grade["script_unknown"]),
                f"{name}: số map hoặc cờ script không khớp bảng xếp loại")
        spawns = sorted({row["spawns"] for row in rows})
        dimensions = sorted({(row["width"], row["height"]) for row in rows})
        require(len(spawns) == len(dimensions) == 1
                and content_rows[name] == [[str(count), f"{dimensions[0][0]}×{dimensions[0][1]}",
                                            str(spawns[0]), str(scripted)]],
                f"{name}: tổng hợp map, lưới, spawn hoặc script không khớp CONTENT.md")
        tags = sorted({tag for row in rows for tag in row["script_tags"]})
        matches = sorted({controllers[tag.casefold()] for tag in tags if tag.casefold() in controllers})
        anchor_ids = ["script_factory", *matches]
        if name == "IamWriggle":
            anchor_ids.append("game_type_43")
        family = {
            "family": name, "wave": grade["wave"], "classification": grade["classification"],
            "status": grade["status"], "census_category": grade["category"],
            "map_count": count, "script_count": scripted, "classification_script_unknown": complex_script,
            "script_tags": tags, "static_anchor_ids": anchor_ids,
            "variants": [{"name": row["name"], "width": row["width"], "height": row["height"],
                          "spawn_points": row["spawns"], "parse": row["parse"], "scripted": row["scripted"],
                          "mapscript_tag": "mapscript" in {tag.casefold() for tag in row["script_tags"]}}
                         for row in rows],
            **authority(*FAMILY_PROBES[name]),
        }
        mode = next((row for row in data["classification"]["rows"]
                     if row["kind"] == "mode" and row["name"] == name), None)
        if mode is not None:
            family["same_name_mode_classification"] = {key: mode[key] for key in ("wave", "classification", "status")}
        families.append(family)
    return families


def extended_systems(data, anchors):
    roadmap = [line for line in data["roadmap"].splitlines() if line.startswith("| F3 Chơi với máy có script |")]
    require(len(roadmap) == 1, "thiếu dòng F3 trong lộ trình")
    grades = index((row for row in data["classification"]["rows"] if row["kind"] in {"mode", "system"}),
                   lambda row: (row["kind"], row["name"]))
    catalog = index(data["catalog"]["groups"], lambda row: row["id"])
    matrix = index(data["authority_matrix"]["groups"], lambda row: row["id"])
    systems = []
    for ident, spec in SYSTEMS.items():
        require(spec["scope_text"] in roadmap[0], "phạm vi hệ thống F3 trong lộ trình đã đổi")
        group = "monster" if ident == "monster_season_2" else ident
        prior, evidence = catalog[group], matrix[group]
        counts = {key: prior[key] for key in ("map_count", "family_count", "has_script", "parse_counts")}
        require(all(evidence[key] == val for key, val in counts.items()), "số liệu nhóm F3-0/F3-1 không khớp")
        names = spec["mode_rows"]
        classifications = {name: {key: grades["mode", name][key] for key in ("wave", "classification", "status")}
                           for name in names}
        require(all(prior["mode_classifications"][name] == grade for name, grade in classifications.items()),
                "xếp loại mode F3-0 không khớp bảng hiện hành")
        ids = [*evidence["probe_anchors"], *(f"game_type_{num}" for num in spec.get("game_types", ()))]
        require(set(ids) <= anchors.keys(), "nhóm tham chiếu neo không có trong IDA")
        system = {
            "id": ident, "label": spec["label"], "status": "candidate",
            "features": spec["features"], "feature_scope_locator": INPUTS["roadmap"] + "#full-game-roadmap",
            "mode_classification": classifications, "static_anchor_ids": ids,
            "catalog_group": {"id": group, **counts, "resource_groups": prior["resource_groups"]},
            **authority(spec["question"], spec["probe"]),
        }
        if ident == "ai_battle":
            name = spec["features"]["hidden_character_unlock"]
            unlock = grades["system", name]
            require(unlock["roadmap_only"] and unlock["wave"] == "F3", "thiếu mục mở nhân vật trong bảng xếp loại")
            system["hidden_character_scope"] = {
                "classification_row": name, "roadmap_only": True, "note": unlock["reason"],
            }
        if ident == "hidden_catch":
            system["roadmap_arcade_stage_count"] = 100
        if ident == "monster_season_2":
            system["season_2_name_candidate"] = prior["season_2_name_candidate"]
            system["limit"] = "Nhóm Monster gồm cả hai mùa. Tiền tố MonS2 là dấu vết theo tên; chưa nối với 몬스터대작전."
        systems.append(system)
    return systems


def build_matrix(data, sources):
    validate_sources(data, sources)
    anchors, controllers = static_anchors(data)
    families = map_families(data, controllers)
    systems = extended_systems(data, anchors)
    used_anchors = {name for row in families + systems for name in row["static_anchor_ids"]}
    # Giữ neo quest làm điểm dò chung; chưa gán handler này cho nhiệm vụ CA Hero.
    used_anchors.add("stage_quest")
    matrix = {
        "schema": SCHEMA, "build": BUILD, "authority_status": "unknown", "inputs": sources,
        "static_metadata": {
            "source": "kr_static", "status": "confirmed", **data["ida"]["source_artifacts"],
            "fxmap_pack_hashes": data["catalog"]["static_metadata"]["pack_hashes"]["FxMap"],
            "f3_0_f3_1_classification_sha256": data["catalog"]["input_sha256"]["classification"],
            "limit": "Xác nhận metadata pack và điểm quan sát client. Phân nhóm F3? và liên hệ với neo vẫn là ứng viên.",
        },
        "roadmap_metadata": {"source": "design", "status": "design", "pr": 713,
                             "limit": "Mô tả tính năng từ lộ trình, đối chiếu theo quyết định bao phủ nội dung."},
        "summary": {
            "family_count": len(families), "map_count": sum(row["map_count"] for row in families),
            "scripted_map_count": sum(row["script_count"] for row in families), "system_count": len(systems),
        },
        "static_anchors": {name: anchors[name] for name in sorted(used_anchors)},
        "families": families, "systems": systems,
        "wine_probe": {
            "rounds": ["r2", "r3"], "status": "pending",
            "authority_status": "unknown", "source": "design",
            "plan": "docs/content/f3-pve-authority-design.md",
            "r1": {"locator": "docs/content/f2-f3-wine-probe-r1.md",
                   "limit": "Probe r1 chỉ tải mẫu CA Hero và AIBattle trong phòng roomtype=0, chưa đo quyền PvE."},
            "case_limit": "Nhãn case Hex-Rays không phải opcode; đối chiếu frame, jump table và VA handler.",
            "control": "Dùng cửa vào thật của mode, lặp với đối chứng và đảo host khi có hai client. "
                       "Ghi bên phát, bên áp dụng và bên ghi bền; ghi blocker nếu chưa vào được mode.",
        },
    }
    return {"matrix_sha256": canonical_hash(matrix), **matrix}


def render_markdown(matrix):
    summary = matrix["summary"]
    lines = [
        "# Ma trận F3 mở rộng cho KR 00967", "",
        "Tài liệu sinh từ metadata đã lưu. [JSON](../evidence/content/f3-extended-pve-matrix.json) giữ hash, "
        "biến thể, cờ script, neo tĩnh và đối chứng cho từng mục.", "",
        f"Có **{summary['map_count']} map thuộc {summary['family_count']} họ `F3?`**, gồm "
        f"**{summary['scripted_map_count']} map có script**, và **{summary['system_count']} hệ thống mở rộng**. "
        "Tám họ vẫn là ứng viên theo [bảng xếp loại](classification.md) và "
        "[quyết định bao phủ nội dung](../evidence/owner-directives/full-content-coverage.json).", "",
        "## Tám họ map xếp tạm", "",
        "Tên từng biến thể, lưới và số vị trí xuất phát lấy từ [census](census.json), "
        "đối chiếu với [kiểm kê nội dung](../CONTENT.md). Số vị trí trong file chưa xác định số người được vào phòng. "
        "Hai cột script phân biệt cờ hiện diện trong file và số map có tag bổ sung mà bảng xếp loại "
        "ghi ở `script_unknown`. Các họ này nằm ngoài [catalog F3-0](f3-pve-catalog.md). "
        "Riêng `Antarctic`, hàng mode cùng tên ở `F2?`; hàng họ map vẫn ở `F3?`.", "",
        "| Họ | Map | Biến thể | Lưới | Spawn/map | Có script / có tag bổ sung | Neo riêng |",
        "|---|---:|---|---|---:|---|---|",
    ]
    anchors = matrix["static_anchors"]
    for family in matrix["families"]:
        first = family["variants"][0]
        names = ", ".join(f"`{row['name']}`" for row in family["variants"])
        evidence = []
        for name in family["static_anchor_ids"][1:]:
            anchor = anchors[name]
            symbol = anchor.get("class") or anchor["symbol"]
            va = anchor.get("ctor_va") or anchor.get("wrapper_va") or anchor["va"]
            evidence.append(f"`{symbol}` (`{va}`)")
        lines.append(f"| {family['family']} | {family['map_count']} | {names} | "
                     f"{first['width']}×{first['height']} | {first['spawn_points']} | "
                     f"{family['script_count']} / {family['classification_script_unknown']} | "
                     f"{', '.join(evidence) or '—'} |")
    lines += [
        "", "Factory chung ở `0xeba2d0`. Ghép tên tag script với lớp controller để chọn điểm quan sát. "
        "Cần thấy controller được tạo trong mode thật mới xác nhận đường chạy.", "",
        "## Hệ thống mở rộng theo PR #713", "",
        "Các mục tính năng lấy từ [lộ trình F3](../PLAN.md#full-game-roadmap). "
        "Số lượng và neo nhóm lấy từ [catalog F3-0](f3-pve-catalog.md) và "
        "[ma trận F3-1](f3-script-authority-matrix.md). Chúng mô tả cả nhóm, "
        "chưa phân map theo từng nhánh mode hoặc tính năng.", "",
        "| Hệ thống | Tính năng trong lộ trình | Map / họ trong catalog | Tài nguyên pack (số mục) |",
        "|---|---|---|---|",
    ]
    for system in matrix["systems"]:
        group = system["catalog_group"]
        features = "; ".join(system["features"].values())
        resources = ", ".join(f"`{row['locator']}` ({row['entries']})" for row in group["resource_groups"])
        lines.append(f"| {system['label']} | {features} | {group['map_count']} / {group['family_count']} | {resources} |")
    monster = next(row for row in matrix["systems"] if row["id"] == "monster_season_2")
    season = monster["season_2_name_candidate"]
    lines += [
        "", f"Tiền tố `MonS2` cho **{season['map_count']} map ứng viên thuộc {len(season['families'])} họ**. "
        "Catalog Monster gồm cả hai mùa; chưa nối các map ứng viên với 몬스터대작전. "
        "Số 100 màn của Hidden Catch là mô tả trong lộ trình, "
        "chưa được kiểm bằng danh sách map hay luồng chơi máy xu.", "",
        "## Neo tĩnh và câu hỏi chờ Wine r2/r3", "",
        "[Neo IDA](../protocol/f2-f5-f3-ida-anchors.md) ghi GameType client "
        + ", ".join(f"`{row['client_game_type']}` (`{row['symbol']}`)"
                    for row in anchors.values() if "client_game_type" in row)
        + ". Các liên hệ với mode và route phải kiểm trong Wine.", "",
        "Nhãn đã lưu của `CGameStage::OnPacket`: "
        + ", ".join(f"case `{row['hexrays_case']}` → `{row['handler_va']}`"
                    for row in sorted((row for row in anchors.values() if "hexrays_case" in row),
                                      key=lambda row: row["hexrays_case"]))
        + ". " + matrix["wine_probe"]["case_limit"] + " Case quest chưa nối riêng với CA Hero.", "",
        "Mỗi mục giữ `authority_status: unknown` và route KR `unknown`. "
        "Guard `RollAttack::MonsterAttackReq` có kiểm cờ host hoặc `attackSeq`. "
        "Phân quyền HP, AI, sinh quái, trigger, qua màn và thưởng vẫn chưa đo. "
        "[Probe r1](f2-f3-wine-probe-r1.md) chỉ tải mẫu CA Hero/AIBattle trong phòng `roomtype=0`.", "",
        "| Mục | Câu hỏi còn mở | Đối chứng đề xuất |",
        "|---|---|---|",
    ]
    for row in matrix["families"] + matrix["systems"]:
        lines.append(f"| {row.get('family') or row['label']} | {row['question']} | {row['probe']} |")
    lines += [
        "", matrix["wine_probe"]["control"] + " Xem [thiết kế probe F3-2](f3-pve-authority-design.md).", "",
        "Sinh lại: `python3 tools/kr_f3_extended_pve_audit.py --write`. "
        "Kiểm ngoại tuyến: `python3 tools/kr_f3_extended_pve_audit.py --check`.", "",
        f"SHA-256 nội dung: `{matrix['matrix_sha256']}`. Hash dùng JSON UTF-8 với khóa sắp tăng, "
        "dấu phân tách gọn và một ký tự xuống dòng cuối, bỏ trường `matrix_sha256`.", "",
    ]
    return "\n".join(lines).encode("utf-8")


def run(root, *, write):
    root = root.resolve()
    matrix = build_matrix(*load_inputs(root))
    outputs = {
        JSON_OUT: (json.dumps(matrix, ensure_ascii=False, sort_keys=True, indent=2) + "\n").encode("utf-8"),
        MD_OUT: render_markdown(matrix),
    }
    paths = {name: repo_path(root, name) for name in outputs}
    for name, raw in outputs.items():
        scan_output(raw, name)
    if write:
        for name, raw in outputs.items():
            paths[name].write_bytes(raw)
    else:
        for name, raw in outputs.items():
            current = paths[name].read_bytes()
            scan_output(current, name)
            if name == JSON_OUT:
                stored = json.loads(current)
                require(stored.get("matrix_sha256") == canonical_hash(stored, "matrix_sha256"),
                        f"{name}: SHA-256 nội dung không khớp")
            require(current == raw, f"{name}: đầu ra đã cũ; chạy --write")
    return matrix


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--write", action="store_true")
    mode.add_argument("--check", action="store_true")
    args = parser.parse_args()
    try:
        matrix = run(ROOT, write=args.write)
    except (OSError, KeyError, TypeError, IndexError, json.JSONDecodeError, UnicodeError):
        parser.exit(1, "F3 mở rộng: thiếu file hoặc metadata sai định dạng.\n")
    except ValueError as exc:
        parser.exit(1, f"F3 mở rộng: {exc}\n")
    print(f"F3 mở rộng OK: {matrix['summary']['map_count']} map; {matrix['matrix_sha256']}")


if __name__ == "__main__":
    main()
