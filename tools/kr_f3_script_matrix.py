"""Sinh ma trận F3-1 từ bốn tệp JSON đã lưu, hoặc kiểm bằng --check."""

import argparse
from collections import Counter, defaultdict
import ipaddress
import json
import re

if __package__:
    from . import kr_f3_pve_audit as inventory
else:
    import kr_f3_pve_audit as inventory


ROOT = inventory.ROOT
ANCHORS = ROOT / "docs/evidence/content/f2-f5-f3-ida-anchors.json"
MATRIX = ROOT / "docs/evidence/content/f3-script-authority-matrix.json"
MARKDOWN = ROOT / "docs/content/f3-script-authority-matrix.md"
EXPECTED_MAPS, EXPECTED_FAMILIES = 699, 180
SYMBOLS = ("script_tags", "script_triggers", "script_funcs")
PARSES = ("exact", "partial")
SCRIPT_COUNTS = ("yes", "no", "unknown")
GRADES = ("A", "B", "C")
CASES = {9: "stage_quest", 80: "stage_script", 135: "stage_object_36",
         136: "stage_object_52", 177: "stage_timer_ai"}
FACTORY_INDICES = [*range(3), *range(4, 16)]
OTHER_LOCAL_IP = re.compile(
    r"(?<![\d.])(?:127\.\d{1,3}|169\.254|100\.(?:6[4-9]|[7-9]\d|1[01]\d|12[0-7]))"
    r"(?:\.\d{1,3}){2}(?![\d.])"
)
IPV6_LITERAL = re.compile(r"(?<![\w:])[a-f0-9:]*:[a-f0-9:]+(?![\w:])", re.IGNORECASE)

# Các câu hỏi áp dụng cho mọi map. Handler phía client chưa đủ để trả lời.
QUESTIONS = {
    "monster_spawn": (
        "Bên nào quyết định sinh quái và chuyển đợt?",
        "So thời điểm xuất hiện, mã đối tượng và đợt trên hai client với thông điệp đến/đi.",
    ),
    "monster_hp": (
        "Bên nào quyết định sát thương, HP còn lại và quái chết?",
        "So một lần đánh trúng với lần đánh hụt; ghi HP hoặc trạng thái quan sát được trước và sau, cùng nguồn cập nhật.",
    ),
    "trigger_execution": (
        "Bên nào quyết định điều kiện và hiệu ứng của trigger?",
        "Lặp cùng điều kiện khi đổi người tạo phòng; thêm lượt không kích hoạt và so dấu vết trên hai client.",
    ),
    "stage_clear": (
        "Bên nào xác nhận qua màn và bắt đầu chuyển màn?",
        "So lần đủ điều kiện qua màn với lần chưa đủ; ghi bên phát và bên áp dụng kết quả.",
    ),
    "stage_clear_rewards": (
        "Bên nào quyết định cấp thưởng và ghi nhận thưởng qua màn?",
        "So lượt được thưởng và lượt không đạt; ghi số dư hoặc túi đồ trước, sau và sau tái đăng nhập nếu có.",
    ),
    "monster_ai": (
        "Bên nào quyết định AI và trạng thái đòn quái?",
        "Đối chiếu cờ host và thứ tự quan sát tại guard đòn quái với chuyển trạng thái phía nhận.",
    ),
    "kr_wire_route": (
        "Các neo phía client xuất hiện ở route nào trong mỗi màn?",
        "Ghi hướng, thứ tự, mã gói và điểm gọi trong client; chỉ gắn route cho luồng thực sự quan sát được.",
    ),
}
GROUP_PROBES = {
    "monster": ("stage_timer_ai", "monster_attack_guard"),
    "quest_mission": ("stage_quest",),
    "rpg": ("stage_quest",),
    "ca_hero": ("stage_timer_ai",),
    "ai_battle": ("stage_timer_ai",),
}

# Cụm chỉ mô tả metadata. Kiểm dấu vết đặc thù trước StageClearScript chung.
CLUSTER_TAGS = {
    "monster_area_ui": ("GamePlayAreaScript", "MapUIScript"),
    "puzzle_box": ("PuzzleBoxScript",),
    "rpg_transition": ("RPGScript", "MoveMap"),
    "bonus_event": ("BonusMapScript",),
    "bonus_global": ("GeneralScript", "GlobalScript"),
    "tutorial": ("TutorialScript",),
    "quest_custom": ("ZzolmanScript", "CustomTrigger"),
    "rabbit_catch": ("RabbitCatchScript",),
    "wriggle": ("WriggleScript",),
    "catch_tail": ("CatchTailScript",),
    "snowy_field": ("snowyfieldscript",),
    "stage_clear": ("StageClearScript",),
    "timed_wave": ("time", "state", "wave"),
}
# Pin mẫu vì mỗi mẫu có một câu hỏi hoặc phép đối chứng cụ thể.
# Xác thực từng mẫu và độ phủ cụm bằng dữ liệu mỗi lần sinh.
PRIMARY_PROBES = (
    ("MonSeal01", "stage_clear", "So nhịp sinh quái, một lần thay đổi HP và qua màn khi đảo vai host."),
    ("MonS2Fire03", "monster_area_ui", "Tách trigger vùng và cập nhật quái khỏi camera/UI cục bộ."),
    ("SnowyFields07", "stage_clear", "Dò trigger qua màn ở nhóm đối chiếu; chưa biết map dùng cửa vào Quest nào."),
    ("RPG11_N_02", "rpg_transition", "Thử trigger đến gần và theo dõi chuyển bản đồ qua MoveMap."),
    ("CAhero01", "script_marker", "Dùng màn có dấu mapscript để dò nguồn cập nhật của chế độ."),
    ("CAhero07", "no_script", "Đối chứng cùng họ CAhero, census không ghi script."),
    ("Hdn_Alche_Aqua", "stage_clear", "So trigger và kết quả với mẫu Hidden Catch không có dấu script."),
    ("Hdn_Alche_Cam", "no_script", "Đối chứng để tách hiệu ứng của chế độ khỏi script trong map."),
    ("BonusLargeGourd01", "bonus_event", "So hoàn thành và không hoàn thành để tìm nguồn thông báo thưởng."),
    ("BonusBigRPG01", "bonus_global", "Đối chiếu cấu hình giới hạn vật phẩm với thưởng quan sát được."),
    ("Fm_AIBtlDesert05", "no_script", "Đối chứng AI trên map không có dấu script; theo dõi timer và trạng thái."),
    ("Tutorial01", "tutorial", "Kiểm trigger hướng dẫn; dùng một client nếu cửa vào chỉ hỗ trợ chơi đơn."),
)
FOLLOWUP_PROBES = (
    ("MonSnowBear22", "puzzle_box", "Đo tương tác PuzzleBox nếu mẫu Monster đầu chưa giải thích được trigger."),
    ("Baseball01", "timed_wave", "Dùng mốc thời gian và đợt làm đối chứng script của một màn F2."),
    ("Hamster01", "rabbit_catch", "Theo dõi điều kiện bắt mục tiêu và thông báo hoàn tất."),
    ("Wriggle01", "wriggle", "Theo dõi thay đổi vùng và tác động quan sát được trên mỗi client."),
    ("RescueET01", "catch_tail", "Kiểm controller thực sự được tạo và hành vi khi đạt mục tiêu."),
    ("SnowyFields01", "snowy_field", "Tách cập nhật NPC khỏi các hiệu ứng cục bộ của màn."),
    ("PopcadeA01", "quest_custom", "Theo dõi CustomTrigger ở màn có một vị trí spawn; không ép chạy hai client."),
)
PROBE_STEPS = (
    "Kiểm lease Wine và cô lập outbound trước khi chạy. Dùng bản sao KR 00967 đã pin hash, chỉ cho phép kết nối tới server và bộ thu cục bộ. Ghi phiên bản Wine, hash bản sao và mọi thay đổi nếu có.",
    "Xác minh cửa vào, map mà client thực sự tải và chế độ trên cả hai phía. Nếu cửa vào chưa có, ghi blocker; không dùng ép map hoặc ACK giả làm bằng chứng.",
    "Với chế độ hỗ trợ hai người, lấy một lượt A tạo phòng và một lượt B tạo phòng; lặp cùng thao tác và thêm đối chứng không kích hoạt. Màn chỉ chơi đơn chỉ đo hành vi cục bộ, không dùng để kết luận phần chia quyền.",
    "Đồng bộ dấu thời gian của hai client, capture cục bộ và phía server. Ghi riêng nguồn phát, bên áp dụng và bên ghi bền. Log emulator chỉ mô tả emulator; kết quả chỉ áp dụng cho luồng đã đo.",
)


def require(condition, message):
    if not condition:
        raise ValueError(message)


def reject_private(body):
    inventory.reject_private(body)
    require(not OTHER_LOCAL_IP.search(body), "metadata chứa địa chỉ nội bộ")
    for match in IPV6_LITERAL.finditer(body):
        try:
            address = ipaddress.IPv6Address(match[0])
        except ipaddress.AddressValueError:
            continue
        require(not address.is_private, "metadata chứa địa chỉ nội bộ")


def load_inputs():
    paths = {"f3_pve_catalog": inventory.CATALOG, "ida_anchors": ANCHORS,
             "census": inventory.CENSUS, "classification": inventory.CLASSIFICATION}
    data, sources = {}, {}
    for key, path in paths.items():
        raw = path.read_bytes()
        data[key] = json.loads(raw)
        sources[key] = {"locator": path.relative_to(ROOT).as_posix(), "sha256": inventory.sha256(raw)}
    return data, sources


def static_anchors(data, sources):
    census, grades, catalog, ida = (data[key] for key in
                                   ("census", "classification", "f3_pve_catalog", "ida_anchors"))
    exe = census["client_exe_sha256"]
    artifacts = ida["source_artifacts"]
    require(census["build"] == grades["build"] == catalog["build"] == "KR00967", "sai build KR00967")
    require(grades["census_sha256"] == sources["census"]["sha256"]
            and catalog["input_sha256"] == {key: sources[key]["sha256"] for key in ("census", "classification")},
            "census hoặc xếp loại không khớp hash đầu vào F3-0")
    require(exe == grades["client_sha256"] == catalog["static_metadata"]["client_exe_sha256"]
            == artifacts["kr_ca_exe_sha256"] and grades["idb_sha256"] == artifacts["kr_full_i64_sha256"],
            "hash build của census, xếp loại, F3-0 hoặc IDA không khớp")
    catalog_body = {key: value for key, value in catalog.items() if key not in {"catalog_sha256", "input_sha256"}}
    require(catalog["catalog_sha256"] == inventory.sha256(inventory.canonical(catalog_body)),
            "hash nội dung F3-0 không khớp")

    f3 = ida["f3_pve"]
    require(f3["status"] == "kr_static/confirmed", "neo F3 chưa có xác nhận tĩnh")
    handlers = inventory.index_rows(f3["cgamestage_handlers"], lambda row: row["opcode"])
    attack = f3["monster_attack_authority"]
    require(handlers.keys() == CASES.keys() and f3["script_factory_va"] == "0xeba2d0"
            and attack["host_flag_fn"] == "0x11656a0" and "attackSeq" in attack["notes"],
            "neo factory, CGameStage hoặc guard đòn quái đã đổi")
    rows = {
        "script_factory": {
            "va": f3["script_factory_va"], "controller_range": f3["script_controller_range"],
            "controller_count": len(FACTORY_INDICES), "controller_indices": FACTORY_INDICES,
            "controller_indices_locator": "docs/protocol/f2-f5-f3-ida-anchors.md",
        },
        "monster_attack_guard": {
            "symbol": "RollAttack::MonsterAttackReq",
            **{key: attack[key] for key in ("handlers", "fallback_dispatcher", "host_flag_fn",
                                           "host_flag_expr", "notes")},
        },
    }
    for case, key in CASES.items():
        rows[key] = {"symbol": "CGameStage::OnPacket", **handlers[case]}
    return {
        "source": "kr_static", "status": "confirmed", "locator": sources["ida_anchors"]["locator"] + "#f3_pve",
        "client_exe_sha256": exe, "idb_sha256": artifacts["kr_full_i64_sha256"],
        "image_base": artifacts["image_base"], "anchors": rows,
        "limit": "Chỉ xác nhận điểm quan sát phía client; chưa nối neo với route hay quyền quyết định của từng màn.",
    }


def script_cluster(parse, has_script, tags):
    if parse == "partial":
        return "unresolved"
    if has_script == "no":
        return "no_script"
    return next((key for key, required in CLUSTER_TAGS.items() if set(required) <= tags), "script_marker")


def count_keys(entries, field, keys):
    counts = Counter(row[field] for row in entries)
    return {key: counts[key] for key in keys}


def summarize(maps):
    return {"map_count": len(maps), "parse_counts": count_keys(maps, "parse", PARSES),
            "has_script": count_keys(maps, "has_script", SCRIPT_COUNTS)}


def join_catalog(data):
    catalog, census, classification = (data[key] for key in ("f3_pve_catalog", "census", "classification"))
    groups = inventory.index_rows(catalog["groups"], lambda row: row["id"])
    catalog_families = inventory.index_rows(catalog["families"], lambda row: (row["group"], row["family"]))
    candidates = [row for row in census["maps"] if inventory.select_group(row) is not None]
    census_maps = inventory.index_rows(candidates, lambda row: row["name"].casefold())
    grades = inventory.index_rows((row for row in classification["rows"] if row["kind"] == "map"),
                                  lambda row: row["name"])

    maps, families, seen = [], [], set()
    for (group, family), entry in sorted(catalog_families.items()):
        grade = inventory.grade(grades.get(family))
        require(grade is not None and grade == entry["classification"], "xếp loại họ F3 không khớp")
        require(group in groups and entry["maps"].keys() == set(PARSES), "họ F3 không có nhóm hoặc trạng thái hợp lệ")
        family_maps = []
        for parse in PARSES:
            for name in entry["maps"][parse]:
                folded = name.casefold()
                require(folded not in seen and folded in census_maps, "map F3 trùng hoặc không có trong census")
                seen.add(folded)
                row = census_maps[folded]
                require((row["name"], row["family"], row["parse"], inventory.select_group(row))
                        == (name, family, parse, group), "map F3 không khớp tên, họ, nhóm hoặc trạng thái census")
                symbols = {key: sorted(set(row[key])) for key in SYMBOLS}
                tags = {tag.casefold() for tag in symbols["script_tags"]}
                has_script = ("unknown" if parse == "partial" else "yes" if row["scripted"] else "no")
                exact_absence = "no" if parse == "exact" else "unknown"
                family_maps.append({
                    "name": name, "family": family, "stage_group": group, "census_category": row["category"],
                    "classification": grade["classification"], "parse": parse, "has_script": has_script,
                    "mapscript_tag": "yes" if "mapscript" in tags else exact_absence,
                    "literal_script_tag": "yes" if "script" in tags else exact_absence,
                    "script_cluster": script_cluster(parse, has_script, set(symbols["script_tags"])),
                    **symbols,
                    "width": row["width"], "height": row["height"], "spawn_points": row["spawns"],
                })
        counts = summarize(family_maps)
        require(all(entry[key] == value for key, value in counts.items())
                and grades[family]["map_count"] == len(family_maps), "số liệu họ F3 không khớp census hoặc xếp loại")
        families.append({"family": family, "stage_group": group, "classification": grade, **counts})
        maps.extend(family_maps)

    require(seen == census_maps.keys() and len(maps) == EXPECTED_MAPS and len(families) == EXPECTED_FAMILIES,
            "phạm vi F3 không đủ 699 map / 180 họ")
    require(catalog["summary"]["parse_counts"] == summarize(maps)["parse_counts"]
            and catalog["summary"]["map_count"] == EXPECTED_MAPS
            and catalog["summary"]["family_count"] == EXPECTED_FAMILIES,
            "tổng F3-0 không khớp ma trận")
    by_group, families_by_group = defaultdict(list), defaultdict(list)
    for row in maps:
        by_group[row["stage_group"]].append(row)
    for row in families:
        families_by_group[row["stage_group"]].append(row)

    summaries = []
    for key, group in groups.items():
        group_maps, group_families = by_group[key], families_by_group[key]
        counts = summarize(group_maps) | {"family_count": len(group_families)}
        require(all(group[field] == value for field, value in counts.items()), "tổng nhóm F3-0 không khớp ma trận")
        probes = ["script_factory", "stage_script"] if counts["has_script"]["yes"] else []
        if any(row["parse"] == "exact" and "ScriptObject" in row["script_tags"] for row in group_maps):
            probes.extend(("stage_object_36", "stage_object_52"))
        probes.extend(GROUP_PROBES.get(key, ()))
        summaries.append({
            "id": key, "label": group["label"], "scope": group["scope"], "stage_group_status": group["grouping_status"],
            **counts, "probe_anchors": probes, "probe_anchors_status": "candidate",
        })
    return sorted(maps, key=lambda row: (row["stage_group"], row["family"], row["name"])), families, summaries


def wine_plan(maps, groups):
    by_name = {row["name"]: row for row in maps}
    probes, covered = [], set()
    for priority, selections in (("first", PRIMARY_PROBES), ("followup", FOLLOWUP_PROBES)):
        for name, cluster, focus in selections:
            row = by_name.get(name)
            require(row is not None and row["parse"] == "exact" and row["script_cluster"] == cluster
                    and row["spawn_points"] > 0 and name not in covered, "mẫu F3-2 không còn đủ điều kiện")
            covered.add(name)
            probes.append({"priority": priority, "map": name, "stage_group": row["stage_group"],
                           "script_cluster": cluster, "focus": focus})
    counts = Counter(probe["stage_group"] for probe in probes if probe["priority"] == "first")
    require(all(1 <= counts[row["id"]] <= 2 for row in groups), "mỗi nhóm cần 1–2 mẫu ưu tiên")
    require({row["script_cluster"] for row in maps if row["parse"] == "exact"}
            == {probe["script_cluster"] for probe in probes}, "chưa phủ hết cụm script đọc trọn")
    return {"source": "design", "status": "design", "run_status": "pending",
            "steps": PROBE_STEPS, "samples": probes}


def build_matrix(data, sources):
    anchors = static_anchors(data, sources)
    maps, families, groups = join_catalog(data)
    result = {
        "schema": "kre.f3-script-authority-matrix.v1", "build": data["census"]["build"], "inputs": sources,
        "static_metadata": {"source": "kr_static", "status": "confirmed",
                            "limit": "Xác nhận dấu vết đã lưu, không xác nhận đường chạy hay nghĩa của tag."},
        "static_anchors": anchors,
        "summary": {**summarize(maps), "family_count": len(families),
                    "family_classification_counts": {grade: sum(row["classification"]["classification"] == grade
                                                                for row in families) for grade in GRADES}},
        "authority": {
            "scope": "all_maps_and_stage_groups",
            "source": "kr_static",
            "gaps": {key: {"status": "unknown", "question": question, "probe": probe}
                     for key, (question, probe) in QUESTIONS.items()},
            "limit": "Chưa phân ranh giữa server và host-client; không có phép đo F3-2.",
        },
        "groups": groups, "families": families, "maps": maps, "wine_f3_2": wine_plan(maps, groups),
    }
    raw = inventory.canonical(result)
    reject_private(raw.decode("utf-8"))
    return {"matrix_sha256": inventory.sha256(raw), **result}


def render_json(matrix):
    lines = ["{"]
    for key, value in sorted(matrix.items()):
        if key in {"groups", "families", "maps"}:
            rows = ",\n".join("    " + inventory.canonical(row).decode().rstrip("\n") for row in value)
            lines.append(f'  "{key}": [\n{rows}\n  ],')
        else:
            lines.append(f'  "{key}": {inventory.canonical(value).decode().rstrip()},')
    lines[-1] = lines[-1].removesuffix(",")
    return "\n".join([*lines, "}", ""])


def render_markdown(matrix):
    summary = matrix["summary"]
    parse, script = summary["parse_counts"], summary["has_script"]
    grade = summary["family_classification_counts"]
    lines = [
        "# Ma trận quyền thực thi script PvE F3-1", "",
        "Nguồn `kr_static` của build `KR00967`: [F3-0](../evidence/content/f3-pve-catalog.json),",
        "[neo IDA](../protocol/f2-f5-f3-ida-anchors.md), [census](census.json) và [xếp loại](classification.json).",
        "[Ma trận JSON](../evidence/content/f3-script-authority-matrix.json) giữ hash của cả bốn đầu vào, từng map và từng họ.", "",
        f"Phạm vi: **{summary['map_count']} map / {summary['family_count']} họ** "
        f"({parse['exact']} `exact`, {parse['partial']} `partial`); lớp A/B/C theo họ: "
        f"{grade['A']} / {grade['B']} / {grade['C']}.",
        f"Script theo census: {script['yes']} có, {script['no']} không ghi nhận, {script['unknown']} chưa rõ.",
        "Census ghi `<mapscript>`; không thấy thẻ literal `<script>` trong phần đã đọc. "
        "Ma trận giữ riêng trạng thái của hai thẻ, cờ script và tên tag, trigger, hàm quan sát được.",
        "`exact` là đọc trọn theo parser census. Ở map `partial`, thiếu tag không chứng minh script vắng mặt.",
        "Các nhóm màn là nhãn kiểm kê. Quest / Mission dùng để đối chiếu F2; chưa xác nhận cửa vào hay route của từng map.", "",
        "[Nguồn tham chiếu](../REFERENCES.md): `boomkr-re/docs/kr-content.md` §5 mô tả tag đã trích từ pack KR; "
        "`boom-re/docs/map.md` mô tả blob XML của build VN. Hai nguồn chỉ giúp đối chiếu bố cục dữ liệu.", "",
        "| Nhóm màn | Phạm vi | Họ | Map exact / partial | Script có / không ghi nhận / chưa rõ |",
        "|---|---|---:|---:|---:|",
    ]
    labels = {}
    for row in matrix["groups"]:
        label = labels[row["id"]] = row["label"]
        scope = "Đối chiếu F2" if row["scope"] == "cross_reference" else "F3"
        parse, script = row["parse_counts"], row["has_script"]
        lines.append(f"| {label} | {scope} | {row['family_count']} | {parse['exact']} / {parse['partial']} | "
                     f"{script['yes']} / {script['no']} / {script['unknown']} |")
    anchors = matrix["static_anchors"]["anchors"]
    factory = anchors["script_factory"]
    attack = anchors["monster_attack_guard"]
    lines.extend([
        "", "## Điểm quan sát tĩnh và giới hạn", "",
        f"- Factory `0x{int(factory['va'], 16):X}` tạo {factory['controller_count']} controller "
        f"(case `0..2, 4..15`, dải `{factory['controller_range']}`). Chưa nối từng map với controller được tạo.",
    ])
    for case, key in CASES.items():
        row = anchors[key]
        detail = {9: "quest", 80: "script", 135: "đọc hai u8, gọi vtable +36",
                  136: "đọc hai u8, gọi vtable +52",
                  177: "đọc u32, u8, u32, u32, u8; hai trường timer nhân 1000"}[case]
        lines.append(f"- `CGameStage::OnPacket` case `{case}` tại `{row['handler_va']}`: {detail}.")
    lines.extend([
        f"- `{attack['symbol']}` ({', '.join(f'`{va}`' for va in attack['handlers'])}, "
        f"dispatcher dự phòng `{attack['fallback_dispatcher']}`): kiểm cờ host "
        f"`{attack['host_flag_fn']}` hoặc `attackSeq` khớp trước trạng thái `34..37`.",
        "", matrix["static_anchors"]["limit"],
        "Danh sách neo theo nhóm trong JSON là các điểm đặt quan sát ứng viên, chưa phải route của chế độ.",
        "Case 135/136 chỉ là neo quan sát IEatObject/IAbilityObject; case 177 chưa đủ để gán quyền AI.",
        "Guard đòn quái chưa chứng minh bên nào có quyền sinh quái, trừ HP, chạy trigger hoặc cấp thưởng.",
        "", "| Quyền / route chưa biết | Trạng thái | Phép đo phân biệt |", "|---|---|---|",
    ])
    for gap in matrix["authority"]["gaps"].values():
        lines.append(f"| {gap['question']} | `{gap['status']}` | {gap['probe']} |")
    lines.extend([
        "", "## Mẫu ưu tiên Wine F3-2", "",
        "Thiết kế thử, chưa chạy. Mỗi nhóm màn có 1–2 mẫu đầu; chỉ mở mẫu bổ sung khi cần phân biệt cụm tag khác.",
        "Mẫu đều `exact` và có vị trí spawn trong census. Điều đó chưa xác nhận màn mở được hoặc chơi được với hai client.",
        "Các cụm mô tả tag quan sát được; mẫu chung một cụm không đại diện cho route của các chế độ khác.",
        "Tiền tố `MonS2` dùng chọn ứng viên mùa 2; không gán mùa 1 cho các họ còn lại.", "",
        "| Ưu tiên | Nhóm màn | Map | Cụm dấu vết | Mục đích |", "|---|---|---|---|---|",
    ])
    plan = matrix["wine_f3_2"]
    for row in plan["samples"]:
        tier = "Đầu" if row["priority"] == "first" else "Bổ sung"
        lines.append(f"| {tier} | {labels[row['stage_group']]} | `{row['map']}` | "
                     f"`{row['script_cluster']}` | {row['focus']} |")
    lines.append("")
    lines.extend(f"{i}. {step}" for i, step in enumerate(plan["steps"], 1))
    lines.extend(["", f"SHA-256 nội dung ma trận: `{matrix['matrix_sha256']}`. "
                  "Hash tính trên JSON chuẩn hoá UTF-8, bỏ trường `matrix_sha256`.",
                  "Sinh lại: `python3 tools/kr_f3_script_matrix.py`; so sánh hai đầu ra: "
                  "`python3 tools/kr_f3_script_matrix.py --check`.", ""])
    return "\n".join(lines)


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="so sánh cả JSON và Markdown với bản đã lưu")
    args = parser.parse_args(argv)
    try:
        matrix = build_matrix(*load_inputs())
        outputs = ((MATRIX, render_json(matrix)), (MARKDOWN, render_markdown(matrix)))
        for path, body in outputs:
            reject_private(body)
            if not args.check and (path.is_symlink() or
                                   (path.exists() and (not path.is_file() or path.stat().st_nlink != 1))):
                raise ValueError("đầu ra phải là file thường, không phải liên kết")
        for path, body in outputs:
            raw = body.encode("utf-8")
            if args.check:
                require(path.is_file() and path.read_bytes() == raw,
                        f"{path.name} chưa đồng bộ; chạy lại công cụ để sinh file")
            else:
                path.write_bytes(raw)
    except (OSError, TypeError, KeyError, AttributeError):
        parser.exit(1, "Không thể sinh ma trận F3-1: không đọc/ghi được file hoặc cấu trúc metadata sai.\n")
    except ValueError as exc:
        parser.exit(1, f"Không thể sinh ma trận F3-1: {exc}\n")
    summary = matrix["summary"]
    print(f"F3-1 OK: {summary['map_count']} map / {summary['family_count']} họ; {matrix['matrix_sha256']}")


if __name__ == "__main__":
    main()
