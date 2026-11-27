"""Sinh và kiểm đặc tả F4-e từ nguồn đã commit, hoàn toàn offline."""

import argparse
import html
import ipaddress
import json
from pathlib import Path
import re
import sys
from urllib.parse import unquote


ROOT = Path(__file__).resolve().parents[1]
SCHEMA = "kre.f4-powerups-boosters.v1"
BUILD = "KR 00967"
CLASSIFICATION = "docs/content/classification.json"
DIRECTIVE = "docs/evidence/owner-directives/full-content-coverage.json"
IDA = "docs/evidence/content/f4-ida-clusters.json"
JSON_OUTPUT = "docs/evidence/content/f4-powerups-boosters-spec.json"
MD_OUTPUT = "docs/content/f4-powerups-boosters-spec.md"

# The Korean strings select the five roadmap rows introduced by PR #713.
# Static cases are adjacent evidence; no case is a verified F4-e route.
SYSTEMS = {
    "power_up_kits": {
        "title": "Bộ tăng sức mạnh",
        "terms": ("파워업 키트",),
        "cases": (("cstage_s2c", 418),),
        "wire": "Opcode và payload chọn, trang bị, tháo kit và chiếu chỉ số vào Start; ánh xạ SKU, slot và mã lỗi.",
        "formula": "Mức cộng theo từng kit, thứ tự cộng dồn, trần KR và cách xử lý kit hết hạn giữa trận.",
        "constraints": {"max_equipped_slots": 2, "enforce_room_stat_caps": True},
        "rules": [
            "Máy chủ cho phép tối đa 2 slot kit còn hiệu lực. Cấu hình có thể hạ giới hạn; trang bị quá giới hạn bị từ chối mà không xóa lựa chọn cũ.",
            "Khi phòng cho phép kit, máy chủ chốt trang bị hợp lệ lúc bắt đầu trận. `RoomRules` quyết định có áp dụng kit và giữ trần hiệu lực theo nhân vật, mode và map cho số bom, sức nổ và tốc độ; kit không nâng các trần đó.",
            "Phòng cấm kit dùng chỉ số gốc. Chưa chiếu bonus lên client cho đến khi đo được consumer và đơn vị chỉ số.",
        ],
        "next_check": "So cùng nhân vật và map khi đeo 0, 1, 2 kit, thử thêm kit thứ ba và thử phòng cấm kit; đối chiếu Start, HUD và chỉ số khi chạm trần.",
    },
    "boosters_all_maps_pass": {
        "title": "Vé ưu đãi và quyền chọn map",
        "terms": ("지존권", "EXP +50%", "루찌 2배"),
        "cases": (),
        "wire": "Opcode, payload, thời hạn và trạng thái bật vé, danh sách map hoặc màn thưởng, và cách chiếu phần thưởng cộng thêm.",
        "formula": "Điều kiện hưởng, VÀNG an ủi, lịch màn thưởng, gốc tính, cộng dồn, làm tròn và trần khi chốt EXP/Lucci.",
        "constraints": {
            "paid_entitlements": False,
            "combat_bonus": False,
            "ranked_bonus": False,
            "enforce_room_map_pool": True,
            "default_reward_percent": {"exp": 100, "lucci": 100},
        },
        "rules": [
            "Vé chỉ cấp miễn phí theo điều kiện công khai, áp dụng như nhau cho mọi người đủ điều kiện. Không bán vé hoặc dùng vé để mua lợi thế trong trận.",
            "Cấu hình máy chủ quyết định nhóm phòng thường, cách cấp, hạn dùng và tỷ lệ thưởng. Tỷ lệ mặc định là 100% thưởng gốc; EXP +50% và Lucci x2 chỉ là nhãn tham khảo.",
            "Mọi map nghĩa là mọi map máy chủ đã mở cho mode và đã qua kiểm tra `RoomRules`. Vé không vượt allowlist, roster, hạn chế sự kiện hay luật phòng và không ưu đãi xếp hạng.",
            "Nếu bật thưởng, máy chủ tính một lần lúc chốt trận qua `finish_match_settled`; client không tự quyết định mức thưởng.",
        ],
        "next_check": "So ván cùng kết quả có và không có vé, thử map hợp lệ và map bị chặn. Đo UI, thay đổi EXP/Lucci và kết quả khi phát lại; projection số dư hoặc tiến độ F4-a riêng lẻ không chứng minh có booster.",
    },
    "upgrades_enhancement": {
        "title": "Cường hóa và nâng cấp",
        "terms": ("강화",),
        "cases": (),
        "wire": "Opcode và payload xem công thức, gửi yêu cầu, báo kết quả và cập nhật vật phẩm, pet, bóng nước hoặc đồ RPG.",
        "formula": "Bảng công thức, nguyên liệu, chi phí, tỷ lệ thành công, mức cộng, trần cấp và xử lý nguyên liệu hoặc vật phẩm khi thất bại.",
        "constraints": {"allow_unconfigured_recipes": False},
        "rules": [
            "Công thức, nguyên liệu, chi phí, tỷ lệ và kết quả thất bại phải do máy chủ cấu hình theo từng loại vật phẩm. Không tự suy ra công thức giữa item, pet, bóng nước và đồ RPG.",
            "Trong chế độ thường có lưu túi, máy chủ kiểm quyền sở hữu và công thức rồi ghi kết quả cùng thay đổi túi trong một giao dịch. Phát lại cùng thao tác trả kết quả cũ, không bốc kết quả mới hoặc trừ hai lần.",
        ],
        "next_check": "Đo từng họ riêng, từ xem công thức tới một lượt thành công, thất bại và thiếu nguyên liệu. So UI, gói tin và thay đổi túi; một lần thành công không xác lập tỷ lệ KR.",
    },
    "gifts_coupons": {
        "title": "Tặng quà và đổi phiếu",
        "terms": ("선물하기", "쿠폰"),
        "cases": (("cshopstagenew_s2c", 236), ("cshopstagenew_s2c", 237), ("cshopstagenew_s2c", 345)),
        "wire": "Opcode và payload gửi, nhận và đổi; mã hóa, độ dài và nghĩa trường của mã phiếu, mã kết quả và cập nhật túi.",
        "formula": "Điều kiện nhận hoặc tặng, danh mục đổi, hạn, hạn mức, số lần đổi và phần thưởng KR.",
        "constraints": {"nexon_real_money_billing": False, "paid_currency": False, "o1_transferable": False},
        "rules": [
            "Chỉ tặng vật phẩm sở hữu hợp lệ hoặc đổi phiếu cục bộ do máy chủ cấp. Loại trừ hoàn toàn Nexon Cash, thanh toán tiền thật, xác minh hóa đơn và dịch vụ đổi phiếu Nexon.",
            "Máy chủ cấu hình SKU được phép tặng, thời hạn, hạn mức và grant. Chuyển quà hoặc đổi phiếu ghi thay đổi túi và biên nhận trong một giao dịch; phát lại không cấp thêm. Quyền phiên O1 không chuyển thành vật phẩm có thể tặng.",
        ],
        "next_check": "Dùng hai tài khoản và phiếu thử trên stub cục bộ; đo tặng, nhận, đổi hợp lệ, hết hạn, bị từ chối và phát lại. Ghép hành động với frame, UI và biến động túi.",
    },
    "second_password": {
        "title": "Mật khẩu cấp 2",
        "terms": ("2차 비밀번호",),
        "cases": (("cstage_s2c", 245), ("cstage_s2c", 377), ("cstage_s2c", 378)),
        "wire": "Opcode hai chiều và payload tạo, xác thực, khóa, mở khóa, đổi và tắt; quy cách mã hóa và mã lỗi. Các case ItemLock không xác nhận route mật khẩu cấp 2.",
        "formula": "Số lần sai, cửa sổ đếm, thời gian khóa, điều kiện reset và phạm vi thao tác được bảo vệ trong KR.",
        "constraints": {"max_failed_attempts": 5, "failure_window_seconds": 900, "cooldown_seconds": 300},
        "rules": [
            "Yêu cầu phiên đăng nhập chính hợp lệ. Quyền mở khóa chỉ có hiệu lực trong phiên; máy chủ thu hồi quyền khi đăng xuất và thu hồi ở mọi phiên khi đổi mật khẩu. Không ghi mật khẩu rõ vào log hay lưu trữ.",
            "Đếm lỗi theo tài khoản trên mọi phiên cho thao tác mở khóa, đổi hoặc tắt. Mặc định đề xuất: 5 lần sai trong cửa sổ 900 giây sẽ khóa thử lại 300 giây. Cấu hình cho phép 1–5 lần, cửa sổ 60–900 giây và khóa 60–900 giây.",
            "Thao tác khóa lại thu hồi quyền mở khóa của phiên. Thao tác sai không mở khóa. Kết nối lại không xóa bộ đếm; yêu cầu trong thời gian khóa bị từ chối mà không kéo dài khóa. Xóa bộ đếm khi xác thực đúng, hết cửa sổ đếm ngoài thời gian khóa hoặc hết thời gian khóa.",
        ],
        "next_check": "Đo tạo, khóa, mở đúng, mở sai, chạm giới hạn, đổi và tắt trên stub. Kiểm tra khi nối lại và khi hết khóa; chỉ giữ metadata đã loại bỏ dữ liệu xác thực.",
    },
}


def read_json(root, path):
    try:
        return json.loads((root / path).read_text(encoding="utf-8"))
    except (OSError, ValueError, RecursionError):
        raise ValueError(f"{path}: không đọc được JSON") from None


def validate_policy(system_id, policy):
    """Check the server-only guardrails; these are not runtime configuration."""
    if (policy["source"] != "design" or policy["status"] != "design"
            or policy["server_configurable"] is not True
            or policy["enabled_by_default"] is not False):
        raise ValueError(f"{system_id}: policy phải là design, cấu hình được và mặc định tắt")

    constraints = policy["constraints"]
    expected = SYSTEMS[system_id]["constraints"]
    if constraints.keys() != expected.keys():
        raise ValueError(f"{system_id}: lệch trường ràng buộc policy")
    bounds = {
        "max_equipped_slots": (0, 2),
        "max_failed_attempts": (1, 5),
        "failure_window_seconds": (60, 900),
        "cooldown_seconds": (60, 900),
    }
    required_on = {"enforce_room_stat_caps", "enforce_room_map_pool"}
    for key, value in constraints.items():
        if key in bounds:
            low, high = bounds[key]
            valid = type(value) is int and low <= value <= high
        elif key == "default_reward_percent":
            valid = (isinstance(value, dict) and value.keys() == {"exp", "lucci"}
                     and all(type(rate) is int and rate == 100 for rate in value.values()))
        else:
            valid = value is (key in required_on)
        if not valid:
            raise ValueError(f"{system_id}: ràng buộc policy không hợp lệ: {key}")


def build_spec(root):
    classification = read_json(root, CLASSIFICATION)
    directive = read_json(root, DIRECTIVE)
    ida = read_json(root, IDA)
    if ida["build"] != BUILD or ida["schema"] != "kre.f4_ida_clusters.v1":
        raise ValueError("Cần bằng chứng tĩnh F4-0 của KR 00967")
    if (classification["build"] != BUILD.replace(" ", "")
            or classification["client_sha256"] != ida["binary"]["sha256"]):
        raise ValueError("Bảng xếp loại và bằng chứng tĩnh phải cùng client KR 00967")

    artifact = {
        "schema": SCHEMA,
        "task": "F4-e",
        "build": BUILD,
        "scope": "Đặc tả dựa trên nguồn đã commit. Không có phép đo client mới hoặc hợp đồng wire F4-e.",
        "sources": {
            "roadmap": {"classification": CLASSIFICATION, "plan": directive["plan"],
                        "pr": 713, "commit": "27b93c0a1dd3a32820cd9176938046a3265117a8"},
            "owner_directive": {"path": DIRECTIVE, "recorded_utc": directive["recorded_utc"]},
            "kr_static": {"path": IDA, "build": BUILD, "image_base": ida["image_base"],
                          "client_sha256": ida["binary"]["sha256"],
                          "ida_image_sha256": ida["binary"]["ida_image_sha256"]},
            "inventory_contract": "docs/protocol/f4-inventory-cosmetics.md#f4b-inventory-contract",
            "economy_architecture": "docs/architecture/f4-economy-service.md",
            "result_projection": "docs/protocol/life-results.md#f4a-profile-result-projection",
            "room_rules": "rust/crates/kr-service/src/rules.rs",
        },
        "systems": {},
    }
    for system_id, entry in SYSTEMS.items():
        rows = [row for row in classification["rows"]
                if row["kind"] == "system" and row["wave"] == "F4" and entry["terms"][0] in row["name"]]
        if (len(rows) != 1 or rows[0]["classification"] != "C"
                or rows[0]["status"] != "candidate" or rows[0].get("roadmap_only") is not True):
            raise ValueError(f"{system_id}: cần đúng một dòng roadmap F4 loại C, candidate")
        anchors = []
        for group, opcode in entry["cases"]:
            wave = ida["waves"]["F4-b" if group == "cstage_s2c" else "F4-d"]
            records = [row for row in wave[group] if row["opcode"] == opcode]
            if len(records) != 1 or records[0]["status"] != "confirmed" or records[0]["wire_status"] != "unknown":
                raise ValueError(f"{system_id}: dấu tĩnh {group}/{opcode} cần đối chiếu lại")
            row = records[0]
            anchors.append({
                "source": "kr_static", "status": "confirmed", "association_status": "candidate",
                "wire_status": "unknown", "direction": "S→C",
                "dispatcher": "CStage" if group == "cstage_s2c" else "CShopStageNew",
                "opcode": opcode, "case_va": row["case_va"], "strings": row["strings"], "limit": row["limit"],
            })
        policy = {
            "source": "design", "status": "design", "server_configurable": True,
            "enabled_by_default": False, "constraints": entry["constraints"], "rules": entry["rules"],
        }
        validate_policy(system_id, policy)
        artifact["systems"][system_id] = {
            "title": entry["title"], "kr_terms": entry["terms"],
            "roadmap": {"source": "roadmap", "status": "candidate",
                        "name": rows[0]["name"], "note": rows[0]["reason"]},
            "static_anchors": anchors,
            "gaps": {key: {"status": "unknown", "detail": entry[key]} for key in ("wire", "formula")},
            "policy": policy,
            "next_check": entry["next_check"],
        }
    return artifact


def render_markdown(spec):
    lines = [
        "# KR 00967: đặc tả F4-e",
        "",
        "<!-- Sinh bằng python3 tools/kr_f4_powerups_spec.py --write. -->",
        "",
        spec["scope"],
        f"[Dữ liệu có cấu trúc](../evidence/content/{Path(JSON_OUTPUT).name}) giữ xuất xứ và địa chỉ VA.",
        "Phạm vi lấy từ [bảng xếp loại](classification.json), [lộ trình F4](../PLAN.md#full-game-roadmap) và",
        "[chỉ đạo bao phủ nội dung](../evidence/owner-directives/full-content-coverage.json) của PR #713.",
        "",
        "Các dấu `kr_static/confirmed` từ [F4-0](../evidence/content/f4-ida-clusters.json) và",
        "[F4-b](../protocol/f4-inventory-cosmetics.md#f4b-inventory-contract) chỉ xác nhận case và chuỗi tĩnh.",
        "Liên hệ với thao tác F4-e là `candidate`; không dùng các case này để bật codec.",
        "Chính sách dưới đây là đề xuất `design`, `server_configurable` và mặc định tắt.",
        "Điểm nối dự kiến dựa trên [AccountService và túi đồ](../architecture/f4-economy-service.md),",
        "[projection kết quả F4-a](../protocol/life-results.md#f4a-profile-result-projection) và",
        "[luật phòng](../../rust/crates/kr-service/src/rules.rs).",
    ]
    for index, system in enumerate(spec["systems"].values(), 1):
        terms = ", ".join(f"`{term}`" for term in system["kr_terms"])
        lines.extend(["", f"## {index}. {system['title']}", "",
                      f"{terms}. Phạm vi `candidate`: {system['roadmap']['note']}"])
        if system["static_anchors"]:
            cases = "; ".join(
                f"`{a['dispatcher']} {a['direction']} {a['opcode']} @ {a['case_va']}`"
                for a in system["static_anchors"]
            )
            lines.extend(["", f"Dấu tĩnh: {cases}."])
        lines.extend([
            "",
            f"- Wire — `unknown`: {system['gaps']['wire']['detail']}",
            f"- Luật và công thức KR — `unknown`: {system['gaps']['formula']['detail']}",
            *(f"- {rule}" for rule in system["policy"]["rules"]),
            "",
            f"Phép đo tối thiểu: {system['next_check']}",
        ])
    return "\n".join(lines) + "\n"


PRIVATE_PATTERNS = {
    "đường dẫn máy cá nhân": r"[/\\](?:home|users|root)[/\\]|(?<!\w)[a-z]:(?:\\|/(?!/))",
    "định danh hạ tầng": r"\b(?:ocid1\.[\w.-]+|[\w.-]+\.(?:local|internal|lan|private))\b",
    "thông tin xác thực": (
        r"-----BEGIN [ A-Z]*PRIVATE KEY-----|\bBearer\s+[\w./~+\-=]+|"
        r"\bAuthorization\s*:\s*Basic\s+[a-z0-9+/=]+|"
        r"\b(?:gh[pousr]_|github_pat_|sk-|xox[baprs]-|AKIA|ASIA)[a-z0-9_-]{12,}|"
        r"://[^/\s@]+:[^/\s@]+@|"
        r"""\b(?:password|passwd|secret|client_secret|api[_-]?key|access[_-]?token|refresh[_-]?token|token)["']?\s*[:=]\s*(?:"[^"\r\n]+"|'[^'\r\n]+'|[^\s"',}\x60]+)"""
    ),
}
IP_LITERAL = re.compile(
    r"(?<![\w:])(?:[\da-f]{0,4}:){2,}[\da-f:.]*(?![\w:])|"
    r"(?<![\w.])(?:[0-9]{1,3}\.){3}[0-9]{1,3}(?!\w|\.[0-9])",
    re.I,
)


def reject_leaks(value):
    """Scan text and decoded JSON fields without echoing a detected value."""
    if isinstance(value, dict):
        for key, item in value.items():
            reject_leaks(key)
            reject_leaks(item)
    elif isinstance(value, (list, tuple)):
        for item in value:
            reject_leaks(item)
    elif isinstance(value, str):
        decoded = html.unescape(unquote(value))
        for label, pattern in PRIVATE_PATTERNS.items():
            if re.search(pattern, decoded, re.I):
                raise ValueError(f"rò rỉ: {label}")
        for match in IP_LITERAL.finditer(decoded):
            try:
                address = ipaddress.ip_address(match[0].rstrip("."))
            except ValueError:
                continue
            if not address.is_global:
                raise ValueError("rò rỉ: địa chỉ IP nội bộ hoặc dành riêng")


def run(root, write):
    spec = build_spec(root)
    reject_leaks(spec)
    outputs = {
        JSON_OUTPUT: (json.dumps(spec, ensure_ascii=False, indent=2, allow_nan=False) + "\n").encode("utf-8"),
        MD_OUTPUT: render_markdown(spec).encode("utf-8"),
    }
    for output, expected in outputs.items():
        try:
            reject_leaks(expected.decode("utf-8"))
            if not write:
                current = (root / output).read_bytes()
                reject_leaks(current.decode("utf-8"))
                if output == JSON_OUTPUT:
                    reject_leaks(json.loads(current))
                if current != expected:
                    raise ValueError("lệch nội dung sinh; chạy lại với --write")
        except (OSError, ValueError, RecursionError) as exc:
            # OS errors can contain absolute paths; decode errors can expose bytes.
            detail = exc if type(exc) is ValueError else "không đọc được artifact hợp lệ"
            raise ValueError(f"{output}: {detail}") from None
    if write:
        for output, expected in outputs.items():
            (root / output).write_bytes(expected)


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    action = parser.add_mutually_exclusive_group(required=True)
    action.add_argument("--write", action="store_true", help="sinh JSON và Markdown")
    action.add_argument("--check", action="store_true", help="kiểm nội dung và rò rỉ, không ghi file")
    args = parser.parse_args(argv)
    try:
        run(ROOT, args.write)
    except (ValueError, OSError, KeyError, TypeError) as exc:
        detail = exc if type(exc) is ValueError else "không truy cập được file hoặc cấu trúc nguồn F4-e"
        print(f"F4-e: {detail}", file=sys.stderr)
        return 1
    print(f"F4-e: {'đã sinh' if args.write else 'đã kiểm'} JSON và Markdown (5 hệ thống)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
