"""Kiểm kê bốn hệ thống tiến trình F4 của KR 00967 từ file trong repo."""

import argparse
import hashlib
import ipaddress
import json
from pathlib import Path
import re
import sys
from urllib.parse import unquote


ROOT = Path(__file__).resolve().parents[1]
SCHEMA = "kre.f4-progression-rewards.v1"
BUILD = "KR00967"
CLASSIFICATION = "docs/content/classification.json"
CATALOG = "docs/evidence/content/f4-progression-rewards-catalog.json"
MARKDOWN = "docs/content/f4-progression-rewards-catalog.md"
SERVICE = "rust/crates/kr-service"
ECONOMY = f"{SERVICE}/src/accounts/economy"
ROADMAP_COMMIT = "27b93c0a1dd3a32820cd9176938046a3265117a8"

# Các locator chỉ kiểm sự hiện diện; hash phát hiện mọi thay đổi byte trong nguồn.
FOUNDATION = {
    "account_serving": (
        f"{SERVICE}/src/account_serving.rs",
        ("fn economy(&self)", "economy rewards require persisted matches",
         "economy requires database profiles", "pub project_own291_level", "pub emit_lucci_135"),
        "Cấu hình economy mặc định tắt thưởng trận. Bật thưởng cần `persist_matches`; "
        "thưởng, catalog có mục và các projection opt-in cần hồ sơ DB. "
        "Các projection cấp và số dư không xác nhận wire của bốn hệ thống này.",
    ),
    "schema_7": (
        f"{SERVICE}/migrations/0007_f4_progression_inventory_shop.sql",
        ("CREATE TABLE player_profiles", "exp INTEGER NOT NULL DEFAULT 0", "level INTEGER NOT NULL DEFAULT 1",
         "lucci INTEGER NOT NULL DEFAULT 0", "CREATE TABLE ledger_entries",
         "CREATE TRIGGER ledger_entries_append_only_delete", "CREATE TABLE player_inventory",
         "CREATE TABLE player_equipped"),
        "Schema 7 thêm hồ sơ theo `player_id` với EXP 0, cấp 1 và Lucci 0, "
        "ledger chỉ ghi thêm, kho SKU và slot gắn đồ.",
    ),
    "equip": (
        f"{ECONOMY}/inventory.rs", ("pub fn equip_item", "if !owned.active", "item.equip_slot.as_deref()"),
        "`equip_item` cần SKU thuộc người chơi, còn hiệu lực và đúng slot catalog. "
        "Đây là trang bị chung, chưa có slot hay danh mục danh hiệu riêng.",
    ),
    "economy_policy": (
        f"{ECONOMY}.rs", ("pub struct EconomyPolicy", "enabled: false", "pub fn level_for",
                          "pub const ECONOMY_AMOUNT_MAX"),
        "`EconomyPolicy` cấu hình EXP/Lucci theo kết quả trận và tính cấp từ mốc EXP tổng. "
        "`ECONOMY_AMOUNT_MAX` giới hạn một lượng cấu hình, không phải trần ví.",
    ),
    "settlement": (
        f"{SERVICE}/src/accounts/matches.rs", ("pub fn finish_match_settled", "Self::settle_tx"),
        "`finish_match_settled` ghi kết quả và quyết toán thưởng trận trong cùng giao dịch; "
        "`match_players.reward` của M6 vẫn bằng 0.",
    ),
    "ledger": (
        f"{ECONOMY}/ledger.rs", ("pub fn admin_grant_lucci", "pub(super) fn economy_audit",
                               ".checked_add(lucci_delta)", "INSERT INTO admin_audit"),
        "Số dư Lucci dùng số nguyên 64 bit có dấu; điều chỉnh dùng phép cộng kiểm tràn "
        "và từ chối số dư âm. `admin_grant_lucci` ghi ledger và `admin_audit` trong "
        "cùng giao dịch. Đây là đường điều chỉnh Lucci, chưa phải đường reset.",
    ),
    "lucci_projection": (
        "rust/crates/kr-wire/src/kr00967/account.rs",
        ("pub fn wire_lucci_u32", "unwrap_or(u32::MAX)", "u32 remain_lucci"),
        "Projection số dư opt-in dùng `remain_lucci` kiểu `u32` và chặn số lớn ở `u32::MAX` "
        "khi mã hóa; giới hạn hiển thị này không phải trần ví hoặc luật mở rộng két.",
    ),
    "schema_8": (
        f"{SERVICE}/migrations/0008_f4_match_consumptions.sql",
        ("CREATE TABLE match_item_consumptions", "REFERENCES match_players(match_id,player_id)",
         "CREATE TRIGGER match_item_consumptions_append_only_delete"),
        "Schema 8 thêm biên nhận tiêu hao trong trận theo người chơi và khóa chống lặp. "
        "Biên nhận chỉ ghi thêm và tồn tại sau khi hết stack; phạm vi là tiêu hao trong trận.",
    ),
}

SYSTEMS = {
    "titles": {
        "name": "Danh hiệu (칭호)",
        "client_labels_ko": ["칭호"],
        "known_account_serving_facts": ["account_serving", "schema_7", "equip", "schema_8"],
        "unknowns": "Chưa biết ID, điều kiện mở khóa, trường gắn danh hiệu và cách client nhận thông báo khi đạt.",
        "server_configurable_default": {
            "enabled": False, "unlock_table": None, "equip_slot": None, "notify_on_unlock": False,
        },
        "design_guard": "Chỉ gắn danh hiệu đã mở khóa. Nếu bật thông báo, chỉ phát sau lần mở khóa mới "
                        "đã commit; thao tác gắn lại, retry hoặc đăng nhập lại không phát thêm.",
        "next_probe": "Đối chiếu màn danh hiệu trước và sau khi đạt, gắn và đăng nhập lại.",
    },
    "level_up_rewards": {
        "name": "Quà lên cấp và rương lên cấp (레벨업 보물상자)",
        "client_labels_ko": ["레벨업 보물상자"],
        "known_account_serving_facts": ["account_serving", "schema_7", "economy_policy", "settlement", "schema_8"],
        "unknowns": "Chưa biết lượng Lucci khi lên cấp, mốc và quà rương theo từng nhân vật, "
                    "hoặc đường nhận quà. Bảng EXP theo kết quả trận không phải bảng quà lên cấp.",
        "server_configurable_default": {
            "enabled": False, "level_up_lucci_table": None, "character_milestones": None,
        },
        "design_guard": "Cấu hình riêng tiền lên cấp và rương theo cặp nhân vật, mốc cấp; "
                        "mỗi quyền nhận chỉ cấp một lần, kể cả khi nhảy nhiều cấp hoặc retry.",
        "next_probe": "Kiểm mốc trước và sau khi lên cấp trên hai nhân vật; tách tiền trận, tiền lên cấp và rương.",
    },
    "lucci_vault": {
        "name": "Két VÀNG và giới hạn VÀNG (루찌 금고)",
        "client_labels_ko": ["루찌 금고"],
        "known_account_serving_facts": [
            "account_serving", "schema_7", "economy_policy", "ledger", "lucci_projection", "schema_8",
        ],
        "unknowns": "Chưa biết trần Lucci gốc, lượng tăng mỗi lần, số lần mở rộng và cách xử lý tiền vượt trần.",
        "server_configurable_default": {
            "enabled": False, "base_lucci_cap": None, "expansion_step": None,
            "max_expansions": None, "max_lucci_cap": None,
        },
        "design_guard": "Khi triển khai, yêu cầu trần cơ sở và bước tăng dương, giới hạn số lần mở rộng "
                        "và trần cuối; kiểm phép nhân/cộng không tràn, không vượt trần hoặc "
                        "giới hạn lưu trữ. Không dùng giới hạn một khoản thưởng làm trần ví.",
        "next_probe": "Kiểm số dư ở sát trần và phản hồi khi mở rộng két hoặc nhận tiền vượt trần.",
    },
    "record_erasers": {
        "name": "Xóa thành tích (전적 지우개, 래더 재도전권)",
        "client_labels_ko": ["전적 지우개", "래더 재도전권"],
        "known_account_serving_facts": ["account_serving", "schema_7", "ledger", "schema_8"],
        "unknowns": "Cấp, thắng/thua và điểm ladder là phạm vi ứng viên; chưa biết mục nào ứng với "
                    "từng vật phẩm, giá trị sau reset hay quan hệ với mùa ladder.",
        "server_configurable_default": {"enabled": False, "allowed_scopes": [], "require_audit": True},
        "design_guard": "Từ chối khi thiếu scope, quyền, vật phẩm hợp lệ hoặc audit. Nếu được triển khai, "
                        "tiêu vé, cập nhật đúng scope và ghi audit theo tác nhân, mục tiêu, trạng thái trước/sau "
                        "và khóa chống lặp trong cùng giao dịch; lỗi thì rollback toàn bộ. "
                        "Giữ lịch sử trận, ledger và biên nhận cũ; Schema 8 không cấp quyền reset ngoài trận.",
        "next_probe": "Đo riêng từng vật phẩm và so cấp, thắng/thua, điểm ladder trước và sau; kiểm vật phẩm hết hạn.",
    },
}

LOCAL_PATH = re.compile(r"(?i)(?<![\w/])(?:/(?:home|root|tmp|var|mnt|users)/|[a-z]:[/\\]|\\\\)")
IP_LITERAL = re.compile(r"(?<![\w:])(?:\d{1,3}(?:\.\d{1,3}){3}|[\da-f]*:[\da-f:.]*:[\da-f:.]*)(?![\w:])",
                        re.IGNORECASE)
CREDENTIAL = re.compile(
    r"-----BEGIN [A-Z ]*PRIVATE KEY-----|\b(?:gh[pousr]_|github_pat_|sk-(?:proj-)?)[a-z0-9_-]{12,}"
    r"|\b(?:AKIA|ASIA)[A-Z0-9]{16}\b|[a-z]+://[^/\s:@]+:[^/\s@]+@"
    r"|\b(?:api[_-]?key|password|passwd|secret|access[_-]?token)[\"']?\s*[:=]\s*[\"']?[^\s\"'`]{4,}"
    r"|\bBearer\s+[a-z0-9_.-]{12,}",
    re.IGNORECASE,
)


def require_no_leaks(content):
    """Không đưa nội dung nghi nhạy cảm vào thông báo lỗi."""
    for value in (content, unquote(content)):
        if LOCAL_PATH.search(value) or CREDENTIAL.search(value):
            raise ValueError("phát hiện đường dẫn máy hoặc thông tin xác thực")
        for match in IP_LITERAL.finditer(value):
            try:
                ipaddress.ip_address(match[0].rstrip("."))
            except ValueError:
                continue
            raise ValueError("phát hiện địa chỉ IP")


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError("khóa JSON trùng")
        result[key] = value
    return result


def read_json(raw):
    def reject_constant(_value):
        raise ValueError("hằng JSON không hợp lệ")

    return json.loads(raw.decode("utf-8"), object_pairs_hook=unique_object, parse_constant=reject_constant)


def sha256(raw):
    return hashlib.sha256(raw).hexdigest()


def encode_json(value, *, pretty=False):
    options = {"indent": 2} if pretty else {"separators": (",", ":")}
    return (json.dumps(value, ensure_ascii=False, sort_keys=True, allow_nan=False, **options) + "\n").encode("utf-8")


def repo_file(root, path):
    file = root / path
    try:
        resolved = file.resolve()
    except RuntimeError:
        raise ValueError(f"liên kết không hợp lệ: {path}") from None
    if resolved != file:
        raise ValueError(f"không dùng file qua liên kết: {path}")
    return file


def read_repo_file(root, path):
    return repo_file(root, path).read_bytes()


def build_catalog(root):
    raw = read_repo_file(root, CLASSIFICATION)
    classification = read_json(raw)
    if (not isinstance(classification, dict)
            or classification.get("schema") != "kre-content-classification-v1"
            or classification.get("build") != BUILD or classification.get("source") != "kr_static"
            or not isinstance(classification.get("client_sha256"), str)
            or not re.fullmatch(r"[a-f0-9]{64}", classification["client_sha256"])
            or not isinstance(classification.get("rows"), list)):
        raise ValueError("bảng xếp loại không hợp lệ cho KR 00967")

    systems = {}
    for system_id, spec in SYSTEMS.items():
        rows = [row for row in classification["rows"]
                if isinstance(row, dict) and row.get("name") == spec["name"]]
        if len(rows) != 1 or rows[0].get("roadmap_only") is not True or any(
                rows[0].get(key) != value for key, value in
                (("kind", "system"), ("wave", "F4"), ("classification", "C"), ("status", "candidate"))):
            raise ValueError(f"{system_id}: cần đúng một hàng ứng viên F4 loại C, roadmap_only")
        systems[system_id] = {
            **spec, "source": "kr_static", "status": "candidate", "roadmap_only": True, "classification": "C",
            "wire_contract_status": "unknown", "reward_tables_status": "unknown",
            "original_server_rules_status": "unknown",
        }

    facts = {}
    for fact_id, (path, locators, note) in FOUNDATION.items():
        source = read_repo_file(root, path)
        if any(locator.encode("utf-8") not in source for locator in locators):
            raise ValueError(f"{fact_id}: thiếu locator trong mã nguồn đã kiểm kê")
        facts[fact_id] = {
            "fact": note, "path": path, "locators": list(locators), "sha256": sha256(source),
            "source": "design", "status": "confirmed",
        }
    body = {
        "schema": SCHEMA, "build": BUILD, "wave": "F4", "roadmap_commit": ROADMAP_COMMIT,
        "classification_sha256": sha256(raw), "client_sha256": classification["client_sha256"],
        "server_foundation": facts, "systems": systems,
    }
    return {**body, "catalog_sha256": sha256(encode_json(body))}


def validate_catalog(raw, expected):
    require_no_leaks(raw.decode("utf-8"))
    catalog = read_json(raw)
    if not isinstance(catalog, dict) or catalog.get("schema") != SCHEMA:
        raise ValueError("schema catalog F4 không hợp lệ")
    digest = catalog.pop("catalog_sha256", None)
    body_bytes = encode_json(catalog)
    require_no_leaks(body_bytes.decode("utf-8"))
    if digest != sha256(body_bytes):
        raise ValueError("catalog_sha256 không khớp nội dung JSON")
    # So dạng chuẩn để khóa toàn bộ trường, kể cả kiểu bool khác kiểu int.
    if encode_json({**catalog, "catalog_sha256": digest}) != encode_json(expected):
        raise ValueError("catalog F4 sai trường, chính sách hoặc nguồn đã kiểm kê")


def render_markdown(catalog):
    lines = [
        "# Kiểm kê quà tiến trình, danh hiệu, két Lucci và xóa thành tích F4",
        "",
        "Phạm vi: bốn mục `roadmap_only` loại C trong [bảng xếp loại](classification.md) và",
        "[lộ trình F4](../PLAN.md#full-game-roadmap) từ PR #713, theo",
        "[chỉ đạo kiểm kê toàn bộ nội dung](../evidence/owner-directives/full-content-coverage.json).",
        "Tên tiếng Hàn là neo tra cứu từ bảng xếp loại KR 00967; chúng không xác nhận hệ thống chạy được.",
        "[Catalog JSON](../evidence/content/f4-progression-rewards-catalog.json) giữ hash và locator.",
        "",
        "Opcode và cấu trúc gói của cả bốn mục chưa biết (`wire_contract_status: unknown`).",
        "Bảng quà và luật server gốc cũng chưa biết (`reward_tables_status` và `original_server_rules_status` đều là `unknown`).",
        "`server_configurable_default` là chính sách thiết kế đề xuất: tắt, chưa điền bảng và giới hạn.",
        "Đây không phải cấu hình mà `account_serving` chấp nhận hoặc quy tắc của server gốc.",
        "Chỉ xác nhận bảng và giới hạn gốc khi có bằng chứng KR tương ứng.",
        "",
        "## Nền Rust đã đối chiếu",
        "",
        "Các mục dưới đây được xác nhận trong mã nguồn dự án (`design`).",
        "Xem thêm [kiến trúc runtime economy](../architecture/f4-economy-service.md).",
        "",
    ]
    for fact_id, fact in catalog["server_foundation"].items():
        lines.append(f"- [`{fact_id}`](../../{fact['path']}): {fact['fact']}")
    for system in catalog["systems"].values():
        refs = ", ".join(f"`{ref}`" for ref in system["known_account_serving_facts"])
        default = json.dumps(system["server_configurable_default"], ensure_ascii=False, sort_keys=True)
        lines += [
            "", f"## {system['name']}", "",
            f"Nền đã có: {refs}.",
            system["unknowns"],
            "",
            f"Mặc định đề xuất: `{default}`.",
            system["design_guard"],
            "",
            f"Bước kiểm KR trong mạng cô lập: {system['next_probe']}",
        ]
    lines += [
        "", "## Tái lập", "",
        "```bash",
        "python3 tools/kr_f4_progression_rewards_audit.py --write",
        "python3 tools/kr_f4_progression_rewards_audit.py --check",
        "```",
        "",
        "Cả hai lệnh chỉ đọc nguồn đã có trong repo và không cần mạng, pack hay checkout bản địa hóa.",
        "`catalog_sha256` băm toàn bộ nội dung JSON trừ chính trường hash: UTF-8, khóa sắp tăng,",
        "không khoảng trắng giữa các phần tử, một LF cuối. `--check` kiểm cả hai file theo từng byte.",
        f"SHA-256 của catalog: `{catalog['catalog_sha256']}`.",
        "",
    ]
    return "\n".join(lines).encode("utf-8")


def run(root, *, write):
    expected = build_catalog(root)
    json_bytes, markdown_bytes = encode_json(expected, pretty=True), render_markdown(expected)
    for output in (json_bytes, markdown_bytes):
        require_no_leaks(output.decode("utf-8"))

    if write:
        outputs = ((repo_file(root, CATALOG), json_bytes), (repo_file(root, MARKDOWN), markdown_bytes))
        for file, data in outputs:
            file.write_bytes(data)
    else:
        actual_json = read_repo_file(root, CATALOG)
        validate_catalog(actual_json, expected)
        actual_markdown = read_repo_file(root, MARKDOWN)
        require_no_leaks(actual_markdown.decode("utf-8"))
        if actual_json != json_bytes or actual_markdown != markdown_bytes:
            raise ValueError("JSON hoặc Markdown lệch bản sinh; kiểm thay đổi rồi chạy --write")


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--write", action="store_true", help="sinh JSON và Markdown")
    mode.add_argument("--check", action="store_true", help="kiểm JSON và Markdown đã commit")
    args = parser.parse_args(argv)
    try:
        run(ROOT, write=args.write)
    except OSError:
        print("F4: không thể đọc hoặc ghi file kiểm kê trong repo", file=sys.stderr)
        return 1
    except (UnicodeError, RecursionError, json.JSONDecodeError):
        print("F4: dữ liệu hoặc JSON không hợp lệ", file=sys.stderr)
        return 1
    except ValueError as exc:
        print(f"F4: {exc}", file=sys.stderr)
        return 1
    print(f"F4: {len(SYSTEMS)} hệ thống, hợp đồng wire chưa xác định")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
