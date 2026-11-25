"""Xuất cấu hình kinh tế và catalog F4 từ pack KR 00967, hoàn toàn offline."""

import argparse
from collections import Counter
import json
from pathlib import Path
import re
import struct
import sys

from content_census import parse_bomber_attr, parse_item_attr
from kr_pack_export import PACK_SHA256, Pack, encode, sha, verified_packs


STANDARD = Path(__file__).resolve().parents[1] / "server/profiles/f4-economy-standard.json"
F4_PACK_SHA256 = {
    **{name: digest for name, digest in PACK_SHA256.items() if name.startswith(("FxContents.", "fx."))},
    "CharTE.idx": "9095e77f111420b6ca7a65ba93326c1e9e2a20cb52ba051b7a6a0f753beea13e",
    "CharTE.idd": "045cef30a9f0a6fd5d161875e05d93917403a687991752f68f7ab87e9f5fd8fc",
    "EmoTE.idx": "ae2702ae84fc9733013bdc1b7cb883cc6f4194e80684fb6593771f8c7153aa4c",
    "EmoTE.idd": "256cb9277b22431c69f84eeb085eb91d706503bb888fadbce1b2efb5366e89cc",
}

# Các giới hạn Validate trong kr-service/src/accounts/economy.rs.
ECONOMY_AMOUNT_MAX = 1_000_000_000_000
LEVEL_MAX = 10_000
CATALOG_ITEMS_MAX = 100_000
I64_MAX = (1 << 63) - 1
U32_MAX = (1 << 32) - 1
CATEGORIES = ("character", "collection", "consumable", "decoration", "emotion")
OUTCOMES = ("win", "loss", "draw")
REWARD_FIELDS = ("exp", "lucci")
POLICY_FIELDS = (*OUTCOMES, "enabled", "grant_on_disconnect_or_left", "level_thresholds")
ITEM_DEFAULT_FIELDS = ("price_lucci", "grant", "equip_slot", "enabled")
TOKEN = re.compile(r"[A-Za-z0-9_.-]+")

OBJECT_TABLES = {
    "CharTE": "CharTEObjectTable",
    "EmoTE": "EmoTEObjectTable",
    "fx": "ObjectTable",
}
OBJECT_GROUPS = {
    "CharTE": {
        group: "decoration"
        for group in ("AvatarDeco", "BombDeco", "CharDeco", "FishingDeco", "GradeDeco", "LadderDeco")
    },
    "EmoTE": {"IDDeco": "decoration", "Slot": "decoration", "Character": "emotion", "Fanfare": "emotion"},
}


def fields(value, allowed, label, *, partial=False):
    if not isinstance(value, dict):
        raise ValueError(f"{label}: cần object JSON")
    extra = value.keys() - set(allowed)
    missing = set() if partial else set(allowed) - value.keys()
    if extra or missing:
        raise ValueError(f"{label}: khóa lạ {sorted(extra)}, thiếu khóa {sorted(missing)}")


def integer(value, minimum, maximum, label):
    if type(value) is not int or not minimum <= value <= maximum:
        raise ValueError(f"{label}: cần số nguyên trong khoảng {minimum}..{maximum}")


def boolean(value, label):
    if type(value) is not bool:
        raise ValueError(f"{label}: cần true hoặc false")


def token(value, limit, label):
    if not isinstance(value, str) or len(value) > limit or not TOKEN.fullmatch(value):
        raise ValueError(f"{label}: cần 1..{limit} ký tự ASCII thuộc [A-Za-z0-9_.-]")


def validate_policy(policy):
    fields(policy, POLICY_FIELDS, "economy_policy")
    for outcome in OUTCOMES:
        fields(policy[outcome], REWARD_FIELDS, outcome)
        for field in REWARD_FIELDS:
            integer(policy[outcome][field], 0, ECONOMY_AMOUNT_MAX, f"{outcome}.{field}")
    for field in ("enabled", "grant_on_disconnect_or_left"):
        boolean(policy[field], field)
    thresholds = policy["level_thresholds"]
    if not isinstance(thresholds, list) or not 1 <= len(thresholds) <= LEVEL_MAX:
        raise ValueError(f"level_thresholds: cần 1..{LEVEL_MAX} mốc")
    for threshold in thresholds:
        integer(threshold, 0, I64_MAX, "level_thresholds")
    if thresholds[0] != 0 or any(a >= b for a, b in zip(thresholds, thresholds[1:])):
        raise ValueError("level_thresholds: phải bắt đầu từ 0 và tăng nghiêm ngặt")


def validate_grant(grant, label):
    kind = grant.get("kind") if isinstance(grant, dict) else None
    if kind == "permanent":
        fields(grant, ("kind",), label)
    elif kind == "stackable":
        fields(grant, ("kind", "max_stack"), label)
        integer(grant["max_stack"], 1, U32_MAX, f"{label}.max_stack")
    elif kind == "timed":
        fields(grant, ("kind", "duration_seconds"), label)
        integer(grant["duration_seconds"], 1, ECONOMY_AMOUNT_MAX, f"{label}.duration_seconds")
    else:
        raise ValueError(f"{label}: kind phải là permanent, stackable hoặc timed")


def validate_item_values(item, label):
    integer(item["price_lucci"], 0, ECONOMY_AMOUNT_MAX, f"{label}.price_lucci")
    if item["equip_slot"] is not None:
        token(item["equip_slot"], 32, f"{label}.equip_slot")
    boolean(item["enabled"], f"{label}.enabled")
    validate_grant(item["grant"], f"{label}.grant")


def validate_catalog(catalog):
    fields(catalog, ("items",), "shop_catalog")
    items = catalog["items"]
    if not isinstance(items, list) or len(items) > CATALOG_ITEMS_MAX:
        raise ValueError(f"shop_catalog.items: cần danh sách tối đa {CATALOG_ITEMS_MAX} mục")
    seen = set()
    for item in items:
        fields(item, ("sku_id", "category", *ITEM_DEFAULT_FIELDS), "shop_catalog.items[]")
        sku = item["sku_id"]
        token(sku, 64, "sku_id")
        if sku in seen:
            raise ValueError(f"sku_id trùng: {sku}")
        seen.add(sku)
        if item["category"] not in CATEGORIES:
            raise ValueError(f"{sku}.category: nhóm không hợp lệ")
        validate_item_values(item, sku)


def unique_object(pairs):
    value = {}
    for key, item in pairs:
        if key in value:
            raise ValueError(f"JSON có khóa trùng: {key}")
        value[key] = item
    return value


def invalid_constant(value):
    raise ValueError(f"JSON có hằng số không hợp lệ: {value}")


def load_json(path):
    with path.open(encoding="utf-8") as stream:
        return json.load(stream, object_pairs_hook=unique_object, parse_constant=invalid_constant)


def object_names(buf):
    """Đọc chỉ mục tên/offset theo boomkr-re/docs/kr-data.md §1; bỏ qua blob."""
    count, = struct.unpack_from("<I", buf)
    offset = 4
    entries = []
    for _ in range(count):
        size, = struct.unpack_from("<I", buf, offset)
        start = offset + 4
        end = start + size
        if size == 0 or end + 4 > len(buf):
            raise ValueError("ObjectTable: tên chỉ mục rỗng hoặc bị cắt")
        position, = struct.unpack_from("<I", buf, end)
        if position != U32_MAX:
            entries.append((buf[start:end].decode("cp949"), position))
        offset = end + 4
    for name, position in entries:
        if not offset <= position < len(buf):
            raise ValueError("ObjectTable: offset nằm ngoài phần dữ liệu object")
        yield name


def catalog_rows(packs, defaults, consumables):
    attr = packs["FxContents"]
    for row in parse_bomber_attr(attr.read("Attr/BomberAttr")):
        # Tên hiển thị có thể lặp; khóa f0 phân biệt các dòng.
        yield dict(defaults["character"], sku_id=f"character.{row['values'][0]}", category="character")

    remaining = set(consumables)
    for row in parse_item_attr(attr.read("Attr/ItemAttr"))["items"]:
        name = row["name"]
        if name not in consumables:
            continue
        if name not in remaining:
            raise ValueError(f"ItemAttr: tên consumable bị trùng: {name}")
        remaining.remove(name)
        item = dict(defaults["consumable"], sku_id=f"consumable.{row['kind']}", category="consumable")
        if consumables[name] is not None:
            item["price_lucci"] = consumables[name]
        yield item
    if remaining:
        raise ValueError(f"ItemAttr: thiếu consumable đã chọn: {sorted(remaining)}")

    for stem, table in OBJECT_TABLES.items():
        for name in object_names(packs[stem].read(f"ObjectTable/{table}")):
            group, sep, leaf = name.partition("/")
            if not sep or not leaf:
                continue
            if stem == "fx":
                # Chỉ mẫu Pet có số; bỏ hiệu ứng và biến thể animation.
                category = "collection" if re.fullmatch(r"Pet/Pet_[0-9]+", name) else None
            else:
                category = OBJECT_GROUPS[stem].get(group)
            if category is not None:
                sku = f"{category}.{stem}.{name.replace('/', '.')}"
                yield dict(defaults[category], sku_id=sku, category=category)


def build_policy(standard, override, *, enable_economy, levels, extra_level_exp):
    fields(override, POLICY_FIELDS, "overrides.economy_policy", partial=True)
    policy = dict(standard, **override)
    for outcome in OUTCOMES:
        if outcome in override:
            fields(override[outcome], REWARD_FIELDS, f"overrides.{outcome}", partial=True)
            policy[outcome] = dict(standard[outcome], **override[outcome])
    # Kiểm giá trị từ override trước khi cờ CLI thay thế.
    validate_policy(policy)
    if enable_economy:
        policy["enabled"] = True
    if levels is not None and levels != len(policy["level_thresholds"]):
        if "level_thresholds" in override:
            raise ValueError("--levels phải bằng số mốc level_thresholds trong overrides")
        thresholds = policy["level_thresholds"][:levels]
        extra_levels = levels - len(thresholds)
        if extra_levels > 0:
            first = thresholds[-1]
            thresholds.extend(first + extra_level_exp * i for i in range(1, extra_levels + 1))
        policy["level_thresholds"] = thresholds
        validate_policy(policy)
    return policy


def build_export(packs, preset, overrides, *, enable_economy=False, unlock_all=False, levels=None):
    fields(preset, ("economy_policy", "extra_level_exp", "category_defaults", "consumables"), "standard")
    validate_policy(preset["economy_policy"])
    integer(preset["extra_level_exp"], 1, I64_MAX, "extra_level_exp")
    defaults = preset["category_defaults"]
    fields(defaults, CATEGORIES, "category_defaults")
    for category, default in defaults.items():
        fields(default, ITEM_DEFAULT_FIELDS, category)
        validate_item_values(default, category)
    consumables = preset["consumables"]
    if not isinstance(consumables, dict):
        raise ValueError("consumables: cần object JSON")
    for name, price in consumables.items():
        if price is not None:
            integer(price, 0, ECONOMY_AMOUNT_MAX, f"consumables.{name}")
    fields(overrides, ("economy_policy", "items"), "overrides", partial=True)
    changes = overrides.get("items", {})
    if not isinstance(changes, dict):
        raise ValueError("overrides.items: cần object JSON theo sku_id")
    if levels is not None:
        integer(levels, 1, LEVEL_MAX, "--levels")
    policy = build_policy(preset["economy_policy"], overrides.get("economy_policy", {}),
                          enable_economy=enable_economy, levels=levels, extra_level_exp=preset["extra_level_exp"])

    items = sorted(catalog_rows(packs, defaults, consumables), key=lambda item: item["sku_id"])
    by_sku = {item["sku_id"]: item for item in items}
    for sku, patch in changes.items():
        if sku not in by_sku:
            raise ValueError(f"overrides.items: sku_id không có trong catalog: {sku}")
        fields(patch, ("price_lucci", "grant", "equip_slot", "disabled"), sku, partial=True)
        for field, value in patch.items():
            if field == "disabled":
                boolean(value, f"{sku}.disabled")
                by_sku[sku]["enabled"] = not value
            else:
                by_sku[sku][field] = value
    catalog = {"items": items}
    validate_catalog(catalog)
    if unlock_all:
        for item in items:
            item["price_lucci"] = 0
    counts = Counter(item["category"] for item in items)
    return {
        "economy_policy": policy,
        "shop_catalog": catalog,
        "summary": {
            "counts_by_category": {category: counts[category] for category in CATEGORIES},
            "level_count": len(policy["level_thresholds"]),
            "unlock_all": unlock_all,
            "catalog_sha256": sha(encode(catalog)),
        },
    }


def export(game, overrides, **options):
    preset = load_json(STANDARD)
    with verified_packs(game, F4_PACK_SHA256) as files:
        packs = {
            stem: Pack(files[f"{stem}.idx"], files[f"{stem}.idd"])
            for stem in ("FxContents", *OBJECT_TABLES)
        }
        return build_export(packs, preset, overrides, **options)


def write_output(out, game, data):
    if out.resolve().is_relative_to(game.resolve()):
        raise ValueError("--out phải nằm ngoài thư mục game")
    # Tạo mới, tránh ghi vào pack hoặc inode hiện có qua liên kết.
    with out.open("xb") as stream:
        stream.write(data)


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--game-dir", type=Path, required=True, help="Thư mục KR 00967, chỉ đọc")
    parser.add_argument("--overrides", type=Path, help="File JSON cấu hình ghi đè")
    parser.add_argument("--out", type=Path, help="Đường dẫn file mới; mặc định ghi stdout")
    parser.add_argument("--enable-economy", action="store_true", help="Bật thưởng trận")
    parser.add_argument("--unlock-all", action="store_true", help="Preset O1: đặt mọi giá về 0")
    parser.add_argument("--levels", type=int, help=f"Số cấp từ 1 đến {LEVEL_MAX}")
    args = parser.parse_args(argv)
    try:
        overrides = load_json(args.overrides) if args.overrides is not None else {}
        data = encode(export(args.game_dir, overrides, enable_economy=args.enable_economy,
                             unlock_all=args.unlock_all, levels=args.levels))
        if args.out is None:
            sys.stdout.buffer.write(data)
        else:
            write_output(args.out, args.game_dir, data)
    except (OSError, ValueError, KeyError, IndexError, struct.error) as exc:
        parser.exit(1, f"Không thể xuất catalog F4: {exc}\n")


if __name__ == "__main__":
    main()
