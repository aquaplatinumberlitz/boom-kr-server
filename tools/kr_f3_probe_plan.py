"""Sinh thiết kế probe Wine F3-2 từ ma trận F3-1 và neo IDA đã lưu."""

import argparse
from collections import Counter
import json
from pathlib import Path
import re

if __package__:
    from . import kr_f3_script_matrix as f3
else:
    import kr_f3_script_matrix as f3


ROOT = Path(__file__).resolve().parents[1]
INPUTS = {
    "matrix": "docs/evidence/content/f3-script-authority-matrix.json",
    "ida_anchors": "docs/evidence/content/f2-f5-f3-ida-anchors.json",
}
PLAN = "docs/evidence/content/f3-wine-probe-plan.json"
DESIGN = "docs/content/f3-pve-authority-design.md"
PENDING = "pending_wine_capture"
EXPECTED = {"first": 12, "followup": 7}
WIRE_CAVEAT = (
    "Các số 9/80/135/136/177 là nhãn tĩnh từ nguồn IDA. Với mỗi frame bắt được, "
    "đối chiếu opcode giải mã từ wire, giá trị vào CGameStage::OnPacket, chỉ số jump table "
    "(kiểm phép trừ 20) và VA handler thực chạy. Không lấy nhãn làm opcode wire hoặc tự cộng 20."
)
IPV4_LITERAL = re.compile(r"(?<![\w.])(?:[0-9]{1,3}\.){3}[0-9]{1,3}(?![\w.])")

# Thao tác dương, thao tác đối chứng và dấu hiệu cần so. Tên tag chỉ gợi ý nơi quan sát.
TRIALS = {
    "MonSeal01": (
        "Đợi đợt đầu, đánh trúng một quái đã định danh một lần, rồi chơi tới khi UI báo kết quả.",
        "Cùng vị trí và thời điểm, đánh hụt; dừng trước đòn cuối và giữ hết cửa sổ quan sát.",
        "ID quái và đợt đầu, lần đổi HP hoặc trạng thái, lần chết cuối và mốc báo qua màn.",
    ),
    "MonS2Fire03": (
        "Cho người thao tác đi qua ranh vùng ứng viên một lần khi người còn lại đứng yên.",
        "Đi song song ngoài ranh vùng cùng khoảng thời gian và giữ nguyên vị trí người quan sát.",
        "Mốc qua ranh vùng, camera/UI từng bên và ID/trạng thái quái trước và sau.",
    ),
    "SnowyFields07": (
        "Qua cửa Quest/Mission đã xác minh, hoàn tất mục tiêu màn do UI hiển thị.",
        "Giữ thiếu mục tiêu cuối trong cửa sổ quan sát, kể cả khi đã thấy quái chết.",
        "Mốc đạt mục tiêu và mốc kết quả; đối chiếu với đường chạy StageClearScript nếu quan sát được.",
    ),
    "RPG11_N_02": (
        "Cho người thao tác tiến đến lối chuyển thấy trong game và dừng sau lần chuyển đầu tiên.",
        "Dừng ngay trước lối chuyển và chờ cùng khoảng thời gian.",
        "Mốc tiếp cận, lần chạy MoveMap nếu thấy, và map thực sự tải trên từng client.",
    ),
    "CAhero01": (
        "Chụp từ lúc tải màn, đứng yên lấy mốc rồi cho một người thực hiện một mục tiêu UI chỉ dẫn.",
        "Lặp lượt đứng yên hết cửa sổ và đối chiếu cùng thao tác trên CAhero07 nếu hai cửa vào tương đương.",
        "Controller hoặc dấu không quan sát được khi tải màn; mốc cập nhật chế độ và nguồn phát.",
    ),
    "CAhero07": (
        "Chụp từ lúc tải màn và lặp thao tác UI hợp lệ đã dùng trên CAhero01 nếu cửa vào tương đương.",
        "Giữ hai người đứng yên; nếu không thể lặp thao tác CAhero01, ghi cặp đối chứng bị chặn.",
        "Dấu cập nhật chung và dấu riêng của từng map; ghi nhánh quan sát được thay vì suy từ census không có tag.",
    ),
    "Hdn_Alche_Aqua": (
        "Thực hiện một lựa chọn hợp lệ rồi chơi tới kết quả Hidden Catch do UI báo.",
        "Để thiếu mục tiêu cuối và so với Hdn_Alche_Cam ở cùng chế độ nếu mở được.",
        "Mốc cập nhật điểm, mốc kết quả, và lời gọi script nếu quan sát được trên từng bên.",
    ),
    "Hdn_Alche_Cam": (
        "Lặp kiểu lựa chọn Hidden Catch của Hdn_Alche_Aqua ở cùng chế độ nếu mở được.",
        "Không thực hiện lựa chọn cuối; nếu khác chế độ, ghi cặp đối chứng bị chặn.",
        "Thay đổi điểm và kết quả trên từng client, kèm dấu điều phối chế độ hoặc script nếu thấy.",
    ),
    "BonusLargeGourd01": (
        "Hoàn tất mục tiêu màn thưởng, xem kết quả rồi đăng nhập lại để đọc số dư và túi đồ.",
        "Dừng trước mục tiêu cuối và đọc lại cùng các trường ở cùng mốc; dùng tài khoản thử riêng cho mỗi lượt.",
        "Thông báo qua màn, biến động phần thưởng thực tế và trạng thái lưu sau đăng nhập lại.",
    ),
    "BonusBigRPG01": (
        "Lặp một thao tác nhận vật phẩm mà UI cho phép tới khi thấy giới hạn hoặc hết cửa sổ đã định.",
        "Giữ cùng thời lượng nhưng bỏ lần nhận cuối; kiểm lại túi đồ ở cùng mốc.",
        "Số lần thử, số lần UI ghi nhận, và thay đổi túi đồ sau đăng nhập lại nếu có.",
    ),
    "Fm_AIBtlDesert05": (
        "Đợi hết khoảng đứng yên ban đầu, cho một người tiếp cận AI và ghi lần đổi trạng thái đầu.",
        "Giữ cả hai ngoài tầm tiếp cận trong cùng cửa sổ, đối chiếu ở cùng mốc timer.",
        "Giá trị timer, trạng thái AI và lần gọi handler 177 nếu thấy; giữ riêng số nhãn tĩnh và opcode đo được.",
    ),
    "Tutorial01": (
        "Theo UI tới bước dùng hoặc nhặt một vật phẩm được chỉ dẫn và ghi lần đổi bước.",
        "Dừng trước hành động được yêu cầu, rồi đợi cùng khoảng thời gian.",
        "Bước hướng dẫn trước/sau, lời gọi phía client và lưu lượng đi kèm; xác nhận riêng cửa vào chơi đơn nếu có.",
    ),
    "MonSnowBear22": (
        "Tương tác một lần với hộp ứng viên thấy trên màn và ghi trạng thái hộp trước và sau.",
        "Tiếp cận cùng vị trí nhưng không tương tác với hộp.",
        "ID và trạng thái hộp, dấu gọi PuzzleBox và hiệu ứng trên hai client nếu quan sát được.",
    ),
    "Baseball01": (
        "Đứng yên từ lúc bắt đầu đến qua mốc timer ứng viên hoặc hết cửa sổ đã định.",
        "Chụp cùng trạng thái trước mốc trong mỗi lượt; lặp ở lượt đảo host mà không thêm thao tác.",
        "Timer và ID đợt trước/sau mốc, cùng bên tạo thông báo nếu có.",
    ),
    "Hamster01": (
        "Hoàn thành một lần bắt mục tiêu theo UI; thử tới lần bắt cuối nếu UI công bố mục tiêu đếm.",
        "Thử hụt hoặc giữ thiếu lần bắt cuối trong cùng cửa sổ.",
        "ID mục tiêu, bộ đếm và mốc thông báo hoàn tất trước/sau thao tác.",
    ),
    "Wriggle01": (
        "Cho một người băng qua ranh vùng ứng viên một lần trong lúc người còn lại đứng yên.",
        "Đi dọc nhưng không qua cùng ranh vùng trong cùng cửa sổ.",
        "Mốc qua ranh, vùng trước/sau và hiệu ứng cục bộ hoặc đồng bộ.",
    ),
    "RescueET01": (
        "Ghi lần tải màn và controller nếu thấy, rồi hoàn thành một mục tiêu được UI chỉ dẫn.",
        "Lặp khởi động nhưng dừng trước khi hoàn thành mục tiêu.",
        "Khởi tạo controller hoặc dấu không quan sát được, lần mục tiêu xuất hiện và phản ứng khi đạt mục tiêu.",
    ),
    "SnowyFields01": (
        "Chọn một NPC thấy trong màn và tiếp cận một lần; ghi đối tượng từ lúc tải màn.",
        "Giữ nguyên ngoài vùng tiếp cận và ghi ở cùng mốc.",
        "ID/vị trí/trạng thái NPC và hiệu ứng riêng từng client; xác định thứ tự nếu có cập nhật mạng.",
    ),
    "PopcadeA01": (
        "Dùng cửa vào đã kiểm, thực hiện một mục tiêu UI chỉ dẫn và theo dõi CustomTrigger nếu quan sát được.",
        "Giữ thiếu mục tiêu vừa chọn trong cửa sổ; ghi riêng nếu cửa vào chỉ chấp nhận một người.",
        "Mốc mục tiêu, dấu gọi CustomTrigger và hiệu ứng trên từng client tham gia.",
    ),
}

STEPS = [
    {"id": "prepare", "action": (
        "Kiểm lease Wine và cô lập outbound trước khi mở hai bản sao KR00967; chỉ cho server và bộ thu "
        "cục bộ. Dùng hai tài khoản thử A/B, pin hash client, cấu hình, phiên bản Wine và binary server; "
        "ghi patch nếu có. Đồng bộ thời gian và bật capture TCP/UDP kèm log Wine A/B và server trước khi tạo phòng."
    )},
    {"id": "qualify", "action": (
        "A tạo phòng qua UI của chế độ và chọn map nếu cửa vào cho phép; B vào và sẵn sàng, A bắt đầu. "
        "Xác minh vai host, chế độ và map thực tải trên cả hai client. Nếu sai map, chưa có cửa vào hoặc "
        "không vào được hai người, áp dụng blocked; không ép map hay dùng ACK giả."
    )},
    {"id": "baseline", "action": (
        "Ghi mốc tải màn, trạng thái đầu, mục tiêu UI, số dư và túi đồ. Cố định bản build, thiết lập phòng, "
        "người thao tác và vị trí nếu có thể. Ghi trước cửa sổ quan sát theo timer thấy được hoặc thời lượng cố định."
    )},
    {"id": "observe", "action": (
        "Ghi lần sinh quái và đổi đợt, ID quái, rồi đối chiếu lần quái tấn công với guard và trạng thái phía nhận nếu thấy. "
        "Nếu có quái đánh được, so một đòn trúng và một đòn hụt trong hai lượt mới; ghi HP hoặc trạng thái trước/sau. "
        "Thực hiện action và control của map trong các lượt mới, ghi trigger, qua màn/chuyển màn và thưởng nếu có. "
        "Nếu không thấy một mốc, ghi cửa sổ và để quyền tương ứng unknown."
    )},
    {"id": "swap", "action": (
        "Thoát và lập phòng mới: B tạo/bắt đầu, A vào/sẵn sàng. Lặp lượt action và control với cùng điều kiện "
        "và cùng người thao tác nếu UI cho phép; nếu đổi theo vai chơi, ghi biến bị lẫn và không kết luận quyền từ lượt đó."
    )},
    {"id": "correlate", "action": (
        "Ghép frame TCP/UDP với log Wine A/B, log server và mốc UI theo lượt, thời gian và ID đối tượng. "
        "Ghi riêng bên phát, server chuyển tiếp hay kiểm tra/đổi trạng thái, lần client áp dụng và bên ghi bền. "
        "Nếu có thưởng, ghi trước, sau và sau đăng nhập lại; lưu capture/log vào local/."
    )},
]

# Tách điều kiện chứng minh quyền khỏi kết quả phép thử: một dấu hiệu có thể đo được
# mà chưa đủ phân biệt bên quyết định, hoặc cho kết quả khác giả thuyết.
AUTHORITY_CRITERIA = {
    "monster_spawn": "Đối chiếu ID/đợt/kích hoạt và nơi tạo trạng thái mới với lần phía còn lại áp dụng.",
    "monster_ai": "Ghép lần quái đổi trạng thái với cờ host, attackSeq và đường nhận; guard riêng lẻ không đủ.",
    "monster_hp": "So đòn trúng/hụt theo cùng ID; phân biệt dự đoán hoặc hoạt ảnh với HP/chết đã áp dụng.",
    "trigger_execution": "So action/control khi đảo host; tách thay đổi UI cục bộ khỏi hiệu ứng được bên khác áp dụng.",
    "stage_clear": "So đủ và thiếu mục tiêu; phân biệt UI báo thắng với bên quyết định kết quả hoặc chuyển màn.",
    "stage_clear_rewards": "So hoàn tất và chưa hoàn tất; ghép thông báo, biến động tài khoản và ghi bền.",
    "kr_wire_route": WIRE_CAVEAT,
}
BLOCKED = (
    "Nếu thiếu cửa vào, mục tiêu hoặc lượt 2P có thể so sánh, ghi bước và lý do bị chặn. "
    "Với cửa vào đã xác minh chỉ chơi đơn, lưu dấu vết cục bộ để tham khảo; giữ quyền host/server unknown."
)


def load_inputs(root=ROOT):
    data, sources = {}, {}
    for key, path in INPUTS.items():
        raw = (root / path).read_bytes()
        data[key] = json.loads(raw)
        sources[key] = {"locator": path, "sha256": f3.inventory.sha256(raw)}
    return data, sources


def static_anchors(matrix, ida, sources):
    f3.require(matrix["inputs"]["ida_anchors"] == sources["ida_anchors"],
               "ma trận F3-1 không khớp hash neo IDA")
    evidence, artifacts = matrix["static_anchors"], ida["source_artifacts"]
    f3.require(matrix["build"] == "KR00967" and ida["scope"] == "static_kr00967_anchors"
               and evidence["client_exe_sha256"] == artifacts["kr_ca_exe_sha256"]
               and evidence["idb_sha256"] == artifacts["kr_full_i64_sha256"]
               and evidence["image_base"] == artifacts["image_base"], "build nguồn không khớp")
    pve = ida["f3_pve"]
    f3.require(pve["status"] == "kr_static/confirmed" and
               (evidence["source"], evidence["status"]) == ("kr_static", "confirmed"),
               "neo F3 chưa có xác nhận tĩnh")
    dispatcher, = {row["dispatcher"] for row in ida["f2_modes"]["cgamestage_handlers"]}
    anchors = {}
    for name, row in evidence["anchors"].items():
        if name == "script_factory":
            f3.require(row["va"] == pve["script_factory_va"], "neo factory không khớp")
            anchors[name] = {**row, "controller_selection_status": "unknown"}
        elif name == "monster_attack_guard":
            attack = pve["monster_attack_authority"]
            f3.require(all(row[key] == attack[key] for key in
                           ("handlers", "fallback_dispatcher", "host_flag_fn", "host_flag_expr", "notes"))
                       and "*(this + 316)" in attack["notes"], "neo đòn quái không khớp")
            anchors[name] = {**row, "attack_seq_expr": "*(this + 316)"}
        else:
            case = row["opcode"]
            handler, = (entry for entry in pve["cgamestage_handlers"] if entry["opcode"] == case)
            f3.require(f3.CASES.get(case) == name and all(row[key] == value for key, value in handler.items()),
                       "case CGameStage không khớp")
            anchors[name] = {
                **{key: value for key, value in row.items() if key != "opcode"},
                "dispatcher_va": dispatcher, "static_case_label": case, "raw_wire_opcode": "unknown",
                "jump_table_index": "unknown", "wire_status": PENDING,
            }
    f3.require(anchors.keys() == {"script_factory", "monster_attack_guard", *f3.CASES.values()},
               "thiếu neo F3")
    return {**evidence, "anchors": anchors, "wire_caveat": WIRE_CAVEAT}


def build_plan(data, sources):
    matrix = data["matrix"]
    f3.require(matrix["schema"] == "kre.f3-script-authority-matrix.v1"
               and data["ida_anchors"]["schema"] == "boom.f2f5f3.ida_anchors.v1", "schema đầu vào không hỗ trợ")
    digest = f3.inventory.sha256(f3.inventory.canonical(
        {key: value for key, value in matrix.items() if key != "matrix_sha256"}
    ))
    f3.require(digest == matrix["matrix_sha256"], "hash nội dung ma trận F3-1 không khớp")
    anchors = static_anchors(matrix, data["ida_anchors"], sources)
    groups = f3.inventory.index_rows(matrix["groups"], lambda row: row["id"])
    maps = f3.inventory.index_rows(matrix["maps"], lambda row: row["name"])
    samples = f3.inventory.index_rows(matrix["wine_f3_2"]["samples"], lambda row: row["map"])
    gaps = matrix["authority"]["gaps"]
    f3.require(samples.keys() == TRIALS.keys() and Counter(s["priority"] for s in samples.values()) == EXPECTED,
               "phạm vi mẫu F3-2 đã đổi; cần cập nhật phép thử")
    f3.require(gaps.keys() == AUTHORITY_CRITERIA.keys() and all(gap["status"] == "unknown" for gap in gaps.values()),
               "câu hỏi quyền trong F3-1 đã đổi; cần đối chiếu evidence")
    questions = {
        name: {"status": "unknown", "capture_status": PENDING,
               "axis": "wire_mapping" if name == "kr_wire_route" else "host_vs_server",
               "question": gap["question"], "discriminator": AUTHORITY_CRITERIA[name]}
        for name, gap in gaps.items()
    }

    probes = []
    for name, sample in samples.items():
        row, group = maps[name], groups[sample["stage_group"]]
        f3.require(row["parse"] == "exact" and row["spawn_points"] > 0 and
                   all(row[key] == sample[key] for key in ("stage_group", "script_cluster")),
                   f"mẫu {name} không khớp census trong ma trận")
        f3.require(group["probe_anchors_status"] == "candidate"
                   and set(group["probe_anchors"]) <= anchors["anchors"].keys(), "neo ứng viên không khớp")
        action, control, observable = TRIALS[name]
        probes.append({
            **sample,
            "static_map": {key: row[key] for key in (
                "family", "parse", "has_script", "mapscript_tag", "script_tags",
                "script_triggers", "script_funcs", "spawn_points",
            )},
            "scope": group["scope"],
            "anchor_observation": {
                "status": "candidate", "registry_ref": "static_anchors", "refs": group["probe_anchors"],
            },
            "wire_status": PENDING,
            "authority_status": "unknown",
            "question_refs": sorted(questions),
            "experiment": {"procedure_ref": "two_client_procedure", "action": action, "control": control},
            "acceptance": {
                "observable": observable,
                "pass": ("Có mốc trước/sau cho dấu hiệu trên trong lượt action và control của cả hai vai host. "
                         "Ghép được với capture/log hoặc có dấu vết cục bộ riêng; ghi cả kết quả giống nhau nếu đo được."),
                "fail": ("Đã vào đúng map 2P nhưng thiếu mốc cần so, thiếu đối chứng hoặc không ghép được "
                         "dấu hiệu với client tương ứng."),
                "blocked": BLOCKED,
            },
        })

    return {
        "schema": "kre.f3-wine-probe-plan.v1", "build": matrix["build"], "source": "design",
        "status": "design", "run_status": PENDING, "inputs": sources, "summary": EXPECTED,
        "static_anchors": anchors,
        "static_map_metadata": {**matrix["static_metadata"], "locator": INPUTS["matrix"] + "#maps"},
        "two_client_procedure": STEPS, "authority_questions": questions,
        "authority_decision_rule": (
            "Chỉ gán quyền cho đường chạy đã đo nếu phân biệt được lần tạo hoặc chấp nhận trạng thái "
            "với chuyển tiếp và hiển thị. Đảo host và đối chứng phải loại được giải thích còn lại. "
            "Khi server cục bộ có nhánh đối chứng rõ, tách chuyển tiếp khỏi áp dụng và so trạng thái; "
            "nếu còn nhiều giải thích, ghi unknown dù phép thử có timeline đạt. "
            "Log emulator chỉ mô tả hành vi emulator; không suy quyền của server Nexon."
        ),
        "probes": probes,
    }


def render_json(plan):
    # Mỗi map là một khối, giữ các trường như tag và neo tham chiếu trên cùng một dòng.
    def inline(value):
        return json.dumps(value, ensure_ascii=False, sort_keys=True, allow_nan=False)

    fields = []
    for key, value in sorted(plan.items()):
        if key == "probes":
            rows = [",\n".join(f"      {inline(field)}: {inline(item)}"
                               for field, item in sorted(row.items())) for row in value]
            value = "[\n    {\n" + "\n    },\n    {\n".join(rows) + "\n    }\n  ]"
        elif key == "authority_questions":
            value = "{\n" + ",\n".join(f"    {inline(name)}: {inline(row)}"
                                        for name, row in sorted(value.items())) + "\n  }"
        else:
            value = json.dumps(value, ensure_ascii=False, sort_keys=True, indent=2, allow_nan=False)
            value = value.replace("\n", "\n  ")
        fields.append(f"  {inline(key)}: {value}")
    return "{\n" + ",\n".join(fields) + "\n}\n"


def render_markdown(plan):
    counts = plan["summary"]
    case_labels = ", ".join(str(case) for case in f3.CASES)
    return f"""# Thiết kế đo quyền PvE F3-2

Nguồn mẫu là [ma trận quyền thực thi script F3-1](f3-script-authority-matrix.md) và
[neo IDA F2/F5/F3](../protocol/f2-f5-f3-ida-anchors.md) của build `{plan['build']}`.
[Ma trận probe JSON](../evidence/content/f3-wine-probe-plan.json) giữ hash nguồn,
thao tác, đối chứng và tiêu chí cho **{counts['first']} map ưu tiên và {counts['followup']} map bổ sung**.
Đây là thiết kế thử; hành vi trên wire là `unknown` / `{PENDING}`.

Factory `0xEBA2D0` và guard `RollAttack::MonsterAttackReq` là neo tĩnh. Guard đọc cờ host
`*(this + 157)` hoặc so `attackSeq` tại `*(this + 316)`; chưa cho biết bên nào quyết định HP,
đợt quái hay kết quả. Các số {case_labels} tại `CGameStage::OnPacket` (`0xE5D3A0`) là nhãn tĩnh.
{WIRE_CAVEAT}

1. Kiểm lease Wine, cô lập outbound và bật capture TCP/UDP cùng log Wine của cả hai client và log server.
   Dùng bản sao KR 00967 đã pin hash và server cục bộ. A tạo phòng, B vào, A bắt đầu; xác minh map cả hai tải.
2. Thực hiện thao tác và đối chứng của từng map trong các lượt mới. B tạo và bắt đầu phòng ở lượt đảo host.
   Giữ cùng người thao tác nếu cửa vào cho phép. Thu dấu sinh quái, đòn đánh, HP, trigger, qua màn và thưởng
   nếu có. Chỉ thử mẫu bổ sung khi cần phân biệt cụm script khác.
3. Giữ frame, log và dấu UI theo thời gian và ID đối tượng trong `local/`. Phân biệt bên phát, bên áp dụng
   và bên ghi bền. Nếu cửa vào chưa có hoặc chỉ hỗ trợ chơi đơn, ghi giới hạn thay vì ép chạy 2P.
   Dọn bản sao và môi trường tạm theo [quy định sau mỗi lượt](../evidence/owner-directives/run-cleanup.json).

| Quyền còn `unknown` | Cách phân biệt |
|---|---|
| Sinh quái, chuyển đợt và đòn quái | Ghép ID quái, đợt và guard với thông điệp đầu tiên và lần đổi trạng thái ở phía nhận, rồi so khi đảo host. |
| Sát thương và HP | So đòn trúng/hụt theo cùng ID; tìm nơi quyết định HP hoặc quái chết, tách khỏi hoạt ảnh. |
| Trigger và chuyển màn | So thao tác đủ và thiếu điều kiện; tách thay đổi UI cục bộ khỏi sự kiện được bên khác áp dụng. |
| Qua màn và thưởng | So lượt hoàn tất với lượt còn thiếu mục tiêu; ghi biến động tài khoản trước, sau và sau đăng nhập lại. |

Nếu dấu vết đi theo vai host, kiểm nó là lệnh đề xuất hay trạng thái bên nhận áp dụng. Nếu frame đến từ
server, kiểm server chuyển tiếp hay tự tính/kiểm tra kết quả. Một probe có thể đạt tiêu chí thu dấu vết mà
vẫn chưa phân được quyền. Log server cục bộ mô tả emulator; kết quả không xác nhận luật của server Nexon.

Hướng thiết kế (`design`): sau khi đo, tách đề xuất của client, quyết định của chế độ trong `RoomRules`,
và việc ghi thưởng qua dịch vụ tài khoản. Chỉ đưa một luồng vào server sau khi có đối chứng cho quyền của
luồng đó. Nếu thiết kế cần ghi thưởng, gắn giao dịch với kết quả ván đã kiểm và chống ghi lặp;
công thức thưởng, giá trị gói và hành vi chưa đo tiếp tục giữ `unknown`.

Sinh lại: `python3 tools/kr_f3_probe_plan.py --write`. Kiểm hai đầu ra từ JSON đã lưu:
`python3 tools/kr_f3_probe_plan.py --check`.
"""


def main(argv=None, *, root=ROOT):
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--check", action="store_true", help="kiểm đầu ra đã lưu bằng hai nguồn JSON trong repo")
    mode.add_argument("--write", action="store_true", help="sinh lại hai đầu ra")
    args = parser.parse_args(argv)
    try:
        plan = build_plan(*load_inputs(root))
        outputs = {
            PLAN: render_json(plan),
            DESIGN: render_markdown(plan),
        }
        for name, body in outputs.items():
            f3.reject_private(body)
            f3.require(not IPV4_LITERAL.search(body), "đầu ra chứa địa chỉ IP")
            for match in f3.IPV6_LITERAL.finditer(body):
                # IPv6 hợp lệ chứa ít nhất hai dấu ':'. Loại cả địa chỉ công cộng.
                f3.require(match[0].count(":") < 2, "đầu ra chứa địa chỉ IP")
            path = root / name
            if args.write:
                f3.require(not path.is_symlink() and
                           (not path.exists() or (path.is_file() and path.stat().st_nlink == 1)),
                           f"{name} phải là tệp thường")
            else:
                f3.require(path.is_file() and path.read_bytes() == body.encode("utf-8"),
                           f"{name} chưa đồng bộ; chạy lại với --write")
        if args.write:
            for name, body in outputs.items():
                (root / name).write_bytes(body.encode("utf-8"))
    except (OSError, KeyError, TypeError, ValueError) as exc:
        parser.exit(1, f"F3-2: {exc}\n")
    print(f"F3-2 OK: {plan['summary']['first']} priority + {plan['summary']['followup']} supplementary probes; "
          f"{PENDING}; {'wrote' if args.write else 'checked'} 2 outputs.")


if __name__ == "__main__":
    main()
