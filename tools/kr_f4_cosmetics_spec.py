"""Sinh và kiểm ma trận projection F4 từ các nguồn đã commit, không đọc pack."""

import argparse
import ast
import hashlib
import ipaddress
import json
from pathlib import Path
import posixpath
import re
import sys


ROOT = Path(__file__).resolve().parents[1]
JSON_PATH = "docs/evidence/content/f4-cosmetics-projection-spec.json"
MD_PATH = "docs/content/f4-cosmetics-projection-spec.md"
CONTENT = "docs/evidence/content/"
WIRE = "rust/crates/kr-wire/src/"
SERVICE = "rust/crates/kr-service/"
INPUTS = {
    "protocol": "docs/protocol/f4-inventory-cosmetics.md",
    "architecture": "docs/architecture/f4-economy-service.md",
    "exporter": "tools/kr_f4_catalog_export.py",
    "preset": "server/profiles/f4-economy-standard.json",
    "catalog_summary": CONTENT + "f4-catalog-export-summary.json",
    "bc_summary": CONTENT + "f4-bc-spec-summary.json",
    "wave2": CONTENT + "f4-w2-ida-deepdive.json",
    "wave3": CONTENT + "f4-w3-ida-records.json",
    "wave6": CONTENT + "f4-w6-ida-own291-and-hud.json",
    "wave8": CONTENT + "f4-w8-ida-exp-175-f2.json",
    "progression": CONTENT + "f4a-progression-wine.json",
    "room_protocol": "docs/protocol/room-membership.md",
    "start_protocol": "docs/protocol/f1-aux-opcodes-and-classc.md",
    "preference_protocol": "docs/protocol/auth-session.md",
    "vehicle_protocol": "docs/protocol/vehicles.md",
    "account_wire": WIRE + "kr00967/account.rs",
    "player_wire": WIRE + "kr00967/player.rs",
    "room_wire": WIRE + "kr00967/room.rs",
    "kr_wire": WIRE + "kr00967.rs",
    "room_adapter": SERVICE + "src/room_wire.rs",
    "schema3": SERVICE + "migrations/0003_match_history.sql",
    "schema7": SERVICE + "migrations/0007_f4_progression_inventory_shop.sql",
    "schema8": SERVICE + "migrations/0008_f4_match_consumptions.sql",
    "inventory_store": SERVICE + "src/accounts/economy/inventory.rs",
    "consumption_store": SERVICE + "src/accounts/economy/consumption.rs",
    "session_cosmetics": SERVICE + "src/account_worker/economy/session_cosmetics.rs",
    "runtime_fixtures": SERVICE + "src/f4bc_runtime_tests.rs",
    "consumption_fixtures": SERVICE + "src/accounts/economy/tests/consumption.rs",
    "shop_fixtures": SERVICE + "src/web_shop/tests.rs",
}

WIRE_GAP = "unmeasured_wire_projection"
CAPTURE_GAP = "in_game_219_224_225_313_317_566_pending_capture"
INVENTORY_OPCODES = (219, 224, 225, 313, 314, 315, 316, 317)
SURFACES = ("kr_wire", "own291", "start121", "room_slot")
UNKNOWN = {"status": "unknown", "gap": WIRE_GAP}

# target: (catalog category, source group, slot name from evidence, meaning of slot)
# A group is not a KR slot. Keep source groups separate even when their SKU
# category is the same. Preserve runtime test tokens as fixtures only.
TARGETS = {
    "character": ("character", "FxContents/Attr/BomberAttr", "character", "preset"),
    "avatar": ("decoration", "CharTE/AvatarDeco", "cosmetic.avatar", "proposed"),
    "char_decoration": ("decoration", "CharTE/CharDeco", "cosmetic.char", "proposed"),
    "grade": ("decoration", "CharTE/GradeDeco", "cosmetic.grade", "conditional"),
    "bomb": ("decoration", "CharTE/BombDeco", "cosmetic.bomb", "proposed"),
    "id": ("decoration", "EmoTE/IDDeco", "cosmetic.id", "proposed"),
    "fishing": ("decoration", "CharTE/FishingDeco", None, "unassigned"),
    "ladder": ("decoration", "CharTE/LadderDeco", None, "unassigned"),
    "emote_slot": ("decoration", "EmoTE/Slot", None, "unassigned"),
    "emoticon": ("emotion", "EmoTE/Character", None, "unassigned"),
    "emote": ("emotion", None, "emote", "fixture"),
    "fanfare": ("emotion", "EmoTE/Fanfare", None, "unassigned"),
    "collection": ("collection", "fx/Pet", None, "unassigned"),
    "collection_panel": (None, "FxContents/ItemCollection", None, "unassigned"),
    "emoticon_effect": (None, "fx/Emoticon", None, "unassigned"),
    "consumable": ("consumable", "FxContents/Attr/ItemAttr", None, "unassigned"),
    "quick": ("consumable", None, "quick", "fixture"),
    "backup": ("consumable", None, "backup", "fixture"),
    "hat": (None, None, "head", "fixture"),
    "vehicle": (None, "FxContents/Attr/ItemAttr: Ship/*", None, "unassigned"),
    "balloon": (None, None, None, "unassigned"),
    "flag": (None, None, None, "unassigned"),
    "effect": (None, None, None, "unassigned"),
    "outfit": (None, None, None, "unassigned"),
    "ring": (None, None, None, "unassigned"),
}
TARGET_NOTES = {
    "character": "Khóa BomberAttr của nhân vật nền đã chọn. Chưa nối SKU trang bị với lựa chọn phòng.",
    "grade": "Chỉ đặt slot khi xác nhận có lựa chọn riêng. Cấp hồ sơ không cấp quyền sở hữu SKU.",
    "emote_slot": "Tên nhóm asset Slot chưa xác nhận có thanh gán hoặc enum trang bị.",
    "emoticon": "Chưa đo quyền phát, chủ thể và bề mặt render của peer.",
    "emote": "Slot fixture của Web Shop; không xác nhận trang bị để phát emoticon.",
    "collection": "Chỉ mẫu Pet/Pet_<số> được xuất. Sở hữu Pet chưa xác nhận tiến độ, mục tiêu hoặc thưởng.",
    "collection_panel": "Tài nguyên panel và constructor manager chưa xác nhận đồng bộ tiến độ.",
    "emoticon_effect": "Asset hiệu ứng ngoài exporter. Chưa ghép với SKU của EmoTE/Character.",
    "consumable": "Chỉ SKU ItemAttr được preset chọn; pickup trong trận nằm ngoài quyền túi đồ.",
    "quick": "Slot stack trong kiểm thử nội bộ; không có index hotbar KR đã đo.",
    "backup": "Kiểm thử thay catalog để hai slot cùng tham chiếu stack; không phải hotbar mặc định.",
    "hat": "test.hat dùng slot head trong fixture Rust; chưa có token hoặc phân nhóm hat của KR.",
    "vehicle": "Ship/* và route pickup đã đo thuộc F1-d; chưa xác nhận slot trang bị F4.",
}

SLOT_LABELS = {
    "preset": "preset",
    "proposed": "đề xuất",
    "conditional": "có điều kiện",
    "fixture": "fixture Rust",
    "unassigned": "chưa gán",
}
PRIVATE_MARKER = re.compile(
    r"(?i)(?:[/\\]home[/\\]admin(?:[/\\]|\b)|-----\s*BEGIN [A-Z0-9 ]*PRIVATE KEY"
    r"|PuTTY-User-Key-File-|['\"]?private[_ -]key['\"]?\s*[:=])"
)
IPV4 = re.compile(r"(?<![\d.])(?:[0-9]{1,3}\.){3}[0-9]{1,3}(?!\d|\.\d)")
IPV6 = re.compile(r"(?<![\w:])[a-f0-9]*:[a-f0-9:.]+(?![\w:])", re.I)


class SpecError(ValueError):
    """A diagnostic safe to show without echoing source data."""


def require(condition, message):
    if not condition:
        raise SpecError(message)


def reject_private(body):
    # Fail on IPv4-looking dotted quads, including forms with leading zeroes.
    require(not PRIVATE_MARKER.search(body) and not IPV4.search(body),
            "nội dung chứa đường dẫn riêng, khóa riêng hoặc địa chỉ IP")
    for match in IPV6.finditer(body):
        try:
            ipaddress.IPv6Address(match[0].rstrip("."))
        except ValueError:
            continue
        raise SpecError("nội dung chứa địa chỉ IP")


def sha256(raw):
    return hashlib.sha256(raw).hexdigest()


def load_inputs(root):
    data, sources = {}, {}
    for key, relative in INPUTS.items():
        path = root / relative
        require(path.resolve() == path, f"nguồn phải là tệp trong repo: {relative}")
        raw = path.read_bytes()
        data[key] = raw.decode("utf-8")
        sources[key] = {"path": relative, "sha256": sha256(raw)}
    return data, sources


def exporter_scope(body):
    """Read only the exporter declarations; never import or run the pack exporter."""
    declarations = {
        statement.targets[0].id: statement.value
        for statement in ast.parse(body).body
        if isinstance(statement, ast.Assign) and len(statement.targets) == 1
        and isinstance(statement.targets[0], ast.Name)
    }
    categories = ast.literal_eval(declarations["CATEGORIES"])
    groups = declarations["OBJECT_GROUPS"]
    require(isinstance(groups, ast.Dict), "định dạng OBJECT_GROUPS đã đổi")
    result = {}
    for key, value in zip(groups.keys, groups.values):
        pack = ast.literal_eval(key)
        if isinstance(value, ast.DictComp):
            # CharTE uses {group: "decoration" for group in (...)}.
            require(len(value.generators) == 1, "định dạng nhóm exporter đã đổi")
            generator = value.generators[0]
            require(isinstance(value.key, ast.Name) and isinstance(generator.target, ast.Name)
                    and value.key.id == generator.target.id and not generator.ifs
                    and not generator.is_async, "định dạng nhóm exporter đã đổi")
            entries = dict.fromkeys(ast.literal_eval(generator.iter), ast.literal_eval(value.value))
        else:
            entries = ast.literal_eval(value)
        result.update({f"{pack}/{group}": category for group, category in entries.items()})
    return categories, result


def verify_anchors(data):
    # The matrix transcribes a few concrete Rust fields. Refuse regeneration
    # if their source shape changes; a new hash alone would not revalidate them.
    markers = {
        "protocol": ('id="f4c-cosmetics-contract"', 'id="f4b-inventory-callees-w2"',
                     "8` slot mỗi phần tử", "chưa chốt"),
        "architecture": ("lưu biên nhận Schema 8", "chưa nối handler tiêu hao hoặc projection"),
        "exporter": ('sku_id=f"character.', 'sku_id=f"consumable.', 'r"Pet/Pet_[0-9]+"'),
        "account_wire": ("pub fn own291(", "tail[11..13].copy_from_slice(&level.to_le_bytes())",
                         "out.extend(encode_bytes(&raw)?);",
                         "u64::from(wire_exp_permille_u32(exp_permille))"),
        "player_wire": ("stream[10 + length..] != [0; 98]", "body.extend_from_slice(&self.suffix)"),
        "room_wire": ("pub fn local_character92", "uint(&body[1..], Width::U16)",
                      "out[12..16].copy_from_slice", "prior & !0x1fff0000",
                      "u64::from(character) << 16", "pub fn start121_players",
                      "u16::from_le_bytes([row[3], row[4]])",
                      "selected.extend(player.character.to_le_bytes());", "as_chunks::<26>()"),
        "kr_wire": ("pub fn slot110", "crate::write_uint(u64::from(state), crate::Width::U32)"),
        "room_adapter": ("wire::player106_character(&body, character)", "wire::slot110(",
                         "read_uint(&p[12..16], Width::U32)"),
        "room_protocol": ('id="rust-selected-player-rounds"', "Character ở bit16..28"),
        "start_protocol": ("DC8A2A: 14E9910", "ghi vào"),
        "preference_protocol": ('id="rust-persistent-character"', "chưa phục hồi projection nhân vật trong291"),
        "vehicle_protocol": ("S→C 165", "1192700"),
        "schema3": ("CREATE TABLE player_preferences", "character_key INTEGER NOT NULL"),
        "schema7": ("CREATE TABLE player_inventory", "PRIMARY KEY(player_id,sku_id)",
                    "CREATE TABLE player_equipped", "PRIMARY KEY(player_id,slot)",
                    "FOREIGN KEY(player_id,sku_id) REFERENCES player_inventory(player_id,sku_id)"),
        "schema8": ("CREATE TABLE match_item_consumptions", "UNIQUE(player_id,idempotency_key)",
                    "FOREIGN KEY(match_id,player_id) REFERENCES match_players(match_id,player_id)"),
        "inventory_store": ("pub fn equip_item(", "item.equip_slot.as_deref() != Some(slot)",
                            "active: expires_at.is_none_or(|at| at > now)"),
        "consumption_store": ("fn consume_equipped_item_in_match(", "DELETE FROM player_equipped",
                              "INSERT INTO match_item_consumptions"),
        "session_cosmetics": ("struct O1SessionCosmetics", "item.enabled && item.equip_slot"),
        "runtime_fixtures": ('const QUICK: &str = "quick";', 'sku_id: "test.hat".into()',
                             'equip_slot: Some("head".into())'),
        "consumption_fixtures": ('equip_slot = Some("backup".into());',
                                 'equipped_slots, ["backup", "quick"]'),
        "shop_fixtures": ('item("emote.off", ShopCategory::Emotion, 10, Some("emote"), false)',),
    }
    for key, snippets in markers.items():
        require(all(snippet in data[key] for snippet in snippets),
                f"neo nguồn đã đổi; cần đối chiếu lại: {INPUTS[key]}")


def anchor(source, status, description, *refs, **fields):
    return {
        "source": source, "status": status, "description": description,
        "evidence": [{"input": key, "locator": locator} for key, locator in refs], **fields,
    }


def anchor_index(progression):
    return {
        "character92": anchor(
            "kr_runtime", "confirmed", "Client chọn nhân vật nền bằng ordinal cục bộ và khóa u16.",
            ("room_wire", "local_character92"), ("room_protocol", "Lựa chọn nhân vật, đội và lỗi phòng đầy"),
            direction="C->S", opcode=92, client_locator="sub_D61990", body="u8 local player, u16 character",
        ),
        "character_room_slot": anchor(
            "kr_runtime", "confirmed", "Roster 106 và cập nhật 110 dùng cùng khóa nhân vật trong state.",
            ("room_wire", "player106_character"), ("room_adapter", "player; slot"),
            ("kr_wire", "slot110"), ("room_protocol", "room-player; Lựa chọn nhân vật, đội và lỗi phòng đầy"),
            direction="S->C", opcodes=[106, 110], client_locator="sub_DC6700 (110)",
            fields={"player106": "packed u32 tại body[12..16]", "slot110": "u8 slot, packed u32, u8 zero",
                    "character": "bit 16..28 inclusive của packed u32"},
        ),
        "character_start121": anchor(
            "kr_runtime", "confirmed", "Khóa nhân vật nền theo slot, độc lập với spawn và team.",
            ("room_wire", "start121_characters; start121_players"),
            ("room_protocol", "rust-selected-player-rounds"), ("start_protocol", "f1-aux-character-keys"),
            direction="S->C", opcode=121, client_locator="0xDC8A2A → sub_1241250",
            fields={"layout": "hàng 26 byte trong blob Start đã giải mã",
                    "slot": "row[0]", "character": "u16 LE tại row[3..5]"},
        ),
        "own291_progression": anchor(
            "kr_runtime", "confirmed", "Đối chứng hồ sơ: bật opt-in để chiếu cấp và EXP; chưa định danh slot mỹ phẩm.",
            ("account_wire", "own291; own291_with_progression; local_own291_with_progression"),
            ("wave6", "f4a_own291_and_hud_mapping"), ("wave8", "f4a_own291_exp_permille"),
            ("progression", "observed_own291_frames; ui_screenshots"),
            direction="S->C", opcode=291, client_locator="sub_D2FEB0; sub_AE6330; sub_902A50",
            fields={"level": "u16 LE tại tail[11..13] tính sau tên trong blob",
                    "exp_permille": "u32 ngoài, ngay sau blob UserSimpleInfo"},
            run=progression["run"], service_sha256=progression["binaries"]["files"]["kr-service"],
        ),
        "player106_template": anchor(
            "design", "design", "PlayerTemplate giữ hậu tố opaque và nhánh hồ sơ rỗng; không gán nghĩa mỹ phẩm.",
            ("player_wire", "PlayerTemplate::parse_name_branch; PlayerTemplate::project_name"),
        ),
        "db_inventory": anchor(
            "design", "design", "Schema 8 giữ bảng sở hữu từ Schema 7; tài sản có hạn phải còn hiệu lực khi chiếu.",
            ("schema7", "player_inventory"), ("inventory_store", "InventoryItem.active"),
            migration=7, table="player_inventory", key=["player_id", "sku_id"],
        ),
        "db_equipped": anchor(
            "design", "design", "Slot là token catalog nội bộ. Hàng trang bị tham chiếu SKU sở hữu của đúng người.",
            ("schema7", "player_equipped"), ("inventory_store", "equip_item; list_equipped"),
            migration=7, table="player_equipped", key=["player_id", "slot"],
            owner_foreign_key=["player_id", "sku_id"],
        ),
        "db_character_preference": anchor(
            "design", "design", "Preference nhân vật nền opt-in, riêng với SKU đã equip và không cấp entitlement.",
            ("schema3", "player_preferences.character_key"), ("preference_protocol", "rust-persistent-character"),
            migration=3, table="player_preferences", field="character_key", key=["player_id"],
        ),
        "db_consumption": anchor(
            "design", "design", "Schema 8 lưu biên nhận tiêu hao standard; hết stack thì gỡ slot tham chiếu. O1 không ghi DB.",
            ("schema8", "match_item_consumptions"), ("architecture", "Session::consume_item_in_match"),
            ("consumption_store", "consume_equipped_item_in_match; consume_item"),
            migration=8, table="match_item_consumptions", key=["player_id", "idempotency_key"],
        ),
        "o1_session_cosmetics": anchor(
            "design", "design", "O1 giữ lựa chọn theo token trong bộ nhớ, không ghi SQLite.",
            ("session_cosmetics", "O1SessionCosmetics"), ("architecture", "WebShopAuth"),
        ),
        "f1_vehicle_pickup": anchor(
            "kr_static", "confirmed", "Đối chứng pickup phương tiện F1-d, tách khỏi quyền trang bị trong túi.",
            ("vehicle_protocol", "Nhặt và thay xe"), direction="S->C", opcode=165,
            client_locator="0xE5D570 → 0xE63260 → 0x1192700",
        ),
    }


def build_spec(data, sources):
    verify_anchors(data)
    categories, exporter_groups = exporter_scope(data["exporter"])
    catalog = json.loads(data["catalog_summary"])
    bc = json.loads(data["bc_summary"])
    defaults = json.loads(data["preset"])["category_defaults"]
    progression = json.loads(data["progression"])
    require(catalog["build"] == "KR00967" and all(
        doc["build"] == "KR 00967" for doc in (bc, progression, *(
            json.loads(data[key]) for key in ("wave2", "wave3", "wave6", "wave8")
        ))
    ), "các nguồn không cùng build KR 00967")

    counts = catalog["real_smoke"]["counts_by_category"]
    require(set(categories) == defaults.keys() == counts.keys()
            == {category for category, _, _, _ in TARGETS.values() if category},
            "chưa phủ hết category của exporter, preset hoặc tóm tắt F4")
    require(sum(counts.values()) == catalog["real_smoke"]["item_count"], "số SKU trong tóm tắt không khớp")
    require({category: value["equip_slot"] for category, value in defaults.items() if value["equip_slot"]}
            == {category: slot for category, _, slot, state in TARGETS.values() if state == "preset"},
            "preset có slot mới hoặc đã đổi; cần đối chiếu lại")
    require(exporter_groups == {
        group: category for category, group, _, _ in TARGETS.values()
        if group and group.split("/")[0] in {"CharTE", "EmoTE"}
    }, "chưa phủ hết nhóm ObjectTable của exporter")
    pack_counts = bc["f4c"]["pack_groups"]
    require(pack_counts.keys() <= {group for _, group, _, _ in TARGETS.values() if group},
            "chưa phủ hết nhóm pack F4-c")
    suggested = {group.split("/")[1]: slot + (" nếu có lựa chọn riêng" if state == "conditional" else "")
                 for _, group, slot, state in TARGETS.values() if state in {"proposed", "conditional"}}
    require(suggested == bc["schema7"]["suggested_cosmetic_slots"], "slot đề xuất F4-c đã đổi")

    cstage = bc["f4b"]["cstage_s2c"]
    cases = {case["opcode"]: case for case in cstage["cases"]}
    require(set(INVENTORY_OPCODES) <= cases.keys() and
            [case["opcode"] for case in bc["f4b"]["c2s"]] == [566], "neo route F4-b đã đổi")
    wave2 = {callee["name"]: callee for callee in json.loads(data["wave2"])["f4b_inventory_callees"]["callees"]}
    equip225, send566 = wave2["sub_11CE6A0"], wave2["sub_11AECE0"]
    require(equip225["caller_hint"] == "S->C 225" and "v7 * 8" in equip225["behavior"]
            and "vtable+80" in equip225["behavior"]
            and send566["site_566_va"].lower() == bc["f4b"]["c2s"][0]["call_site_va"].lower()
            and 'một `u8`' in send566["behavior"], "mốc tĩnh 225 hoặc 566 đã đổi")
    return {
        "schema": "kre.f4_cosmetics_projection_spec.v1",
        "build": "KR 00967",
        "client_exe_sha256": bc["binary"]["sha256"],
        "ida_image_sha256": bc["binary"]["ida_image_sha256"],
        "ida_image_base": bc["image_base"],
        "inputs": sources,
        "catalog": {
            "counts_by_category": counts,
            "default_equip_slots": {category: entry["equip_slot"] for category, entry in defaults.items()},
            "object_groups": exporter_groups,
        },
        "projection_context": {
            "own291": ["own291_progression"],
            "room_slot": ["player106_template"],
            "o1": ["o1_session_cosmetics"],
        },
        "gaps": {
            WIRE_GAP: {
                "status": "unknown",
                "description": "Chưa đo ánh xạ SKU/slot sang token và trường wire cho bề mặt này.",
            },
            CAPTURE_GAP: {
                "status": "unknown",
                "static_source": "kr_static",
                "description": "Có mốc tĩnh; còn chờ capture theo thao tác thật để chốt nghĩa, hướng áp dụng và guard.",
                "s2c": {str(opcode): cases[opcode]["handler_va"] for opcode in INVENTORY_OPCODES},
                "c2s": {"566": {"call_site_va": send566["site_566_va"], "static_body": "u8"}},
                "s2c_225": {
                    "entries_per_outer_element": 8,
                    "entry": "u32 + chuỗi (reader vtable +80)",
                    "slot_mapping": "unknown",
                    "reader_va": equip225["va"],
                },
                "evidence": [
                    {"input": "bc_summary", "locator": "f4b"},
                    {"input": "wave2", "locator": "f4b_inventory_callees"},
                    {"input": "wave3", "locator": "f4b_item_record_and_handlers"},
                ],
            },
        },
        "anchors": anchor_index(progression),
        "targets": {
            target: target_row(target, category, group, slot, state, pack_counts)
            for target, (category, group, slot, state) in sorted(TARGETS.items())
        },
    }


def target_row(target, category, group, slot, state, pack_counts):
    projection = {surface: UNKNOWN.copy() for surface in SURFACES}
    db = ["db_inventory"] if category or target == "hat" else []
    if state != "unassigned":
        db.append("db_equipped")
    if category == "consumable":
        db.append("db_consumption")
    context = []
    if target == "character":
        projection.update(kr_wire={"status": "confirmed", "anchor": "character92"},
                          room_slot={"status": "confirmed", "anchor": "character_room_slot"},
                          start121={"status": "confirmed", "anchor": "character_start121"})
        db.append("db_character_preference")
    if target == "vehicle":
        context.append("f1_vehicle_pickup")
    slot_evidence = {
        "preset": "preset",
        "proposed": "bc_summary",
        "conditional": "bc_summary",
    }.get(state)
    if state == "fixture":
        slot_evidence = {
            "hat": "runtime_fixtures", "emote": "shop_fixtures",
            "quick": "runtime_fixtures", "backup": "consumption_fixtures",
        }[target]
    return {
        "catalog_category": category,
        "source_group": group,
        "pack_entry_count": pack_counts.get(group),
        "slot": {"name": slot, "state": state, "input": slot_evidence},
        "projection": projection,
        "schema8_persistence": {"status": "design" if db else "unknown", "anchors": db},
        "inventory_capture_gap": CAPTURE_GAP,
        "context_anchors": context,
        "note": TARGET_NOTES.get(target),
    }


def render_markdown(spec):
    def link(path):
        rel = posixpath.relpath(path, posixpath.dirname(MD_PATH))
        return f"[`{path}`]({rel})"

    def cell(value):
        return f"`{value}`" if value else "—"

    def refs(keys):
        return ", ".join(f"[`{key}`](#anchor-{key})" for key in keys) or "—"

    lines = [
        "# Ma trận projection mỹ phẩm F4 — KR 00967",
        "",
        "Sinh bằng `python3 tools/kr_f4_cosmetics_spec.py --write`; kiểm lại bằng `--check`.",
        f"Bản máy đọc: {link(JSON_PATH)}. SHA-256 nội dung chuẩn: `{spec['spec_sha256']}`.",
        "",
        "Ma trận dùng nguồn đã commit: exporter, preset, metadata KR, codec Rust và migration SQLite.",
        "Slot của catalog là token nội bộ (1–32 ký tự); preset chỉ gán `character`. Các slot đề xuất và",
        "fixture chỉ ghi lại phạm vi nguồn. Cột bề mặt ghi khoảng trống khi chưa biết token, trường,",
        "guard hoặc audience. Các SKU sở hữu riêng không được suy thành dữ liệu công khai cho peer.",
        "",
        "Các hàng `balloon`, `flag`, `hat`, `effect`, `outfit`, `vehicle` và `ring` giữ chỗ để theo dõi",
        "bề mặt cần đo. Chưa có ánh xạ xác nhận từ tên này sang nhóm SKU F4. Cột DB của `hat` chỉ nói",
        "về fixture `head`; cột DB của slot đề xuất chỉ áp dụng nếu catalog gán slot và SKU còn hiệu lực.",
        "",
        "## Ma trận theo slot và bề mặt",
        "",
        "| Target | Nhóm SKU / nguồn pack | Slot nội bộ | `kr_wire` | `own291` | `start121` | `room_slot` | DB ở Schema 8 |",
        "|---|---|---|---|---|---|---|---|",
    ]
    for target, row in spec["targets"].items():
        category, group = row["catalog_category"], row["source_group"]
        group_cell = " / ".join(cell(value) for value in (category, group) if value) or "—"
        slot = row["slot"]
        slot_cell = (cell(slot["name"]) + " (" + SLOT_LABELS[slot["state"]] + ")") if slot["name"] else "—"
        cells = [cell(target), group_cell, slot_cell]
        for surface in SURFACES:
            projection = row["projection"][surface]
            cells.append(refs([projection["anchor"]]) if "anchor" in projection else "[unknown](#wire-gap)")
        db = row["schema8_persistence"]["anchors"]
        lines.append("| " + " | ".join([*cells, refs(db) if db else "`unknown`"]) + " |")
    lines += ["", "### Giới hạn theo mục", ""]
    for target, row in spec["targets"].items():
        if row["note"]:
            context = " " + refs(row["context_anchors"]) + "." if row["context_anchors"] else ""
            lines.append(f"- `{target}`: {row['note']}{context}")
    lines += [
        "",
        '<a id="wire-gap"></a>',
        "## Khoảng trống wire và phép đo",
        "",
        f"- `{WIRE_GAP}`: {spec['gaps'][WIRE_GAP]['description']}",
        f"- `{CAPTURE_GAP}`: áp dụng cho route túi đồ F4, chưa gán opcode theo từng slot.",
        "  Đo riêng S→C `219/224/225/313/314/315/316/317` và C→S `566`. Nhánh tĩnh `225` đọc tám cặp",
        "  `u32 + chuỗi` mỗi phần tử ngoài; chưa gán tám vị trí đó cho bất kỳ tên slot nào.",
        "  Site gửi `566` trong phân tích tĩnh chưa xác nhận thao tác thật kích hoạt nó hoặc vai trò ACK.",
        "- Đối chiếu A/B từ baseline, đổi đúng một SKU đã biết rồi tháo, vào phòng muộn và tái đăng nhập.",
        "  So UI mỗi bên, cursor và guard của writer/reader, frame có hướng/opcode và DB trước/sau.",
        "  Kiểm riêng standard hết hạn/hết stack và O1 theo token. Lựa chọn O1 nằm trong bộ nhớ,",
        "  hết cùng token và không ghi SQLite.",
        "",
        "## Neo đã kiểm và đối chứng",
        "",
        f"Ảnh IDA: `{spec['ida_image_sha256']}` (base `{spec['ida_image_base']}`). "
        f"Client: `{spec['client_exe_sha256']}`.",
        "",
    ]
    for key, item in spec["anchors"].items():
        evidence = "; ".join(
            f"{link(spec['inputs'][ref['input']]['path'])} (`{ref['locator']}`)" for ref in item["evidence"]
        )
        fields = "; ".join(f"{field}: `{value}`" for field, value in item.get("fields", {}).items())
        body = " Body: " + cell(item["body"]) + "." if "body" in item else ""
        table = " Bảng " + cell(item["table"]) + ", khóa " + cell(", ".join(item["key"])) + "." if "table" in item else ""
        lines += [
            f'<a id="anchor-{key}"></a>',
            f"- `{key}` ({item['source']}/{item['status']}): {item['description']}{body}{table}",
            f"  {fields + '. ' if fields else ''}{evidence}.",
        ]
    lines += ["", "## SHA-256 nguồn đã commit", "", "| Nguồn | SHA-256 (byte gốc) |", "|---|---|"]
    for meta in sorted(spec["inputs"].values(), key=lambda entry: entry["path"]):
        lines.append(f"| {link(meta['path'])} | `{meta['sha256']}` |")
    return "\n".join(lines) + "\n"


def canonical(spec):
    return (json.dumps(spec, ensure_ascii=False, sort_keys=True, separators=(",", ":")) + "\n").encode("utf-8")


def generate(root):
    data, sources = load_inputs(root)
    spec = build_spec(data, sources)
    spec["spec_sha256"] = sha256(canonical(spec))
    outputs = {
        JSON_PATH: json.dumps(spec, ensure_ascii=False, sort_keys=True, indent=2) + "\n",
        MD_PATH: render_markdown(spec),
    }
    for body in outputs.values():
        reject_private(body)
    return {path: body.encode("utf-8") for path, body in outputs.items()}


def main(argv=None, *, root=ROOT):
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--write", action="store_true", help="ghi JSON và Markdown")
    mode.add_argument("--check", action="store_true", help="so từng byte với bản sinh lại")
    args = parser.parse_args(argv)
    try:
        root = root.resolve()
        outputs = generate(root)
        for relative, expected in outputs.items():
            path = root / relative
            require(path.resolve() == path, f"đích phải là tệp trong repo: {relative}")
            if args.write:
                path.write_bytes(expected)
            else:
                actual = path.read_bytes()
                reject_private(actual.decode("utf-8"))
                require(actual == expected, f"{relative} không khớp; chạy lại với --write")
    except SpecError as error:
        print(f"F4 cosmetics projection spec: {error}", file=sys.stderr)
        return 1
    except (OSError, ValueError, SyntaxError, KeyError, TypeError, AttributeError):
        # Raw parse exceptions can echo input data or absolute file paths.
        print("F4 cosmetics projection spec: thiếu tệp hoặc định dạng nguồn không hợp lệ", file=sys.stderr)
        return 1
    action = "updated" if args.write else "OK"
    print(f"F4 cosmetics projection spec: {action} (JSON + Markdown)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
