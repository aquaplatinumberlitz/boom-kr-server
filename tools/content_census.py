"""Kiểm kê nội dung client KR 00967 từ pack: map, item, phương tiện, nhân vật, script.

Chỉ đọc bản client gốc và ghi metadata (tên, số đếm, trường header); không ghi asset,
lưới ô, nội dung script hay decompile. Định dạng pack theo `boomkr-re/rust/krpack`
(PackFS__DecryptIdx/DecryptEntry); layout `Attr/*` và map theo `boom-re/docs/items.md`,
`boom-re/docs/map.md`, đã kiểm lại trên KR 00967 (header map KR dài hơn VN một dword,
có ba blob cuối và thêm version 1003).

    python3 tools/content_census.py --client /home/admin/boom/clients/kr-00967 --generate
    python3 tools/content_census.py --check          # không cần client

Phân nhóm nội dung và ghép wiki là suy từ tên, không phải bằng chứng runtime.
"""
import argparse
import collections
import hashlib
import itertools
import json
import operator
import re
import struct
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CENSUS = ROOT / "docs/content/census.json"
OUTPUT = ROOT / "docs/CONTENT.md"
MAGIC = 0x9C6C95CE
M32 = 0xFFFFFFFF


# --- pack -----------------------------------------------------------------
def decrypt_idx(data):
    n = len(data) // 4
    words = list(struct.unpack_from(f"<{n}I", data, 0))
    words[0] ^= words[0]  # khoá là dword đầu
    out = list(itertools.accumulate(words, operator.xor))
    last = out[-1] if out else 0
    tail = bytes(b ^ ((last >> (8 * j)) & 0xFF) for j, b in enumerate(data[n * 4:]))
    return struct.pack(f"<{n}I", *out) + tail


def parse_idx(buf):
    first = struct.unpack_from("<I", buf, 0)[0]
    count, off, extra = (struct.unpack_from("<I", buf, 8)[0], 12, True) if first == 0 else (first, 4, False)
    entries = []
    for _ in range(count):
        length = struct.unpack_from("<i", buf, off)[0]
        off += 4
        if length <= 0 or off + length > len(buf):
            raise ValueError("pack index: bad name length")
        name = buf[off:off + length].decode("utf-8", "replace")
        off += length
        offset, size, key = struct.unpack_from("<III", buf, off)
        off += 16 if extra else 12
        entries.append((name, offset, size, key))
    if len(entries) != count:
        raise ValueError("pack index: entry count mismatch")
    return entries


def decrypt_entry(data, key):
    n = len(data) // 4
    out, prev = [], key
    for i, x in enumerate(struct.unpack_from(f"<{n}I", data, 0)):
        if i == 0:
            prev = x ^ key ^ MAGIC
        else:
            poly = (i * i * (i + 966) * (i + 2020)) & M32
            prev = prev ^ x ^ poly ^ MAGIC
        out.append(prev)
    tail = bytes(b ^ ((prev >> (8 * j)) & 0xFF) for j, b in enumerate(data[n * 4:]))
    return struct.pack(f"<{n}I", *out) + tail


class Pack:
    def __init__(self, root, stem):
        self.idx, self.idd = root / f"{stem}.idx", root / f"{stem}.idd"
        self.order = parse_idx(decrypt_idx(self.idx.read_bytes()))
        self.entries = {name: (offset, size, key) for name, offset, size, key in self.order}

    def read(self, name):
        offset, size, key = self.entries[name]
        with self.idd.open("rb") as handle:
            handle.seek(offset)
            raw = handle.read(size)
        return decrypt_entry(raw, key) if key else raw


def sha256(path):
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for block in iter(lambda: handle.read(1 << 20), b""):
            digest.update(block)
    return digest.hexdigest()


# --- bảng Attr ------------------------------------------------------------
def text(buf, off):
    length = struct.unpack_from("<I", buf, off)[0]
    if not 0 < length < 96:
        raise ValueError(f"bad string length {length} at {off}")
    return buf[off + 4:off + 4 + length].decode("latin1").rstrip("\0"), off + 4 + length


def parse_item_attr(buf):
    total, _, ship_count = struct.unpack_from("<3I", buf, 0)
    off, ships, items = 12, [], []
    for _ in range(ship_count):
        seq = struct.unpack_from("<I", buf, off)[0]
        name, off = text(buf, off + 4)
        ships.append(dict(seq=seq, name=name, values=list(struct.unpack_from("<8i", buf, off))))
        off += 32
    while off < len(buf):
        name, off = text(buf, off)
        sprite, off = text(buf, off)
        weight, field160, group, *tail = struct.unpack_from("<8i", buf, off)
        off += 32
        items.append(dict(kind=len(items), name=name, sprite=sprite, weight=weight,
                          field160=field160, group=group, tail=tail))
    if off != len(buf):
        raise ValueError("ItemAttr: not EOF-exact")
    return dict(header_total=total, ships=ships, items=items)


def parse_bomber_attr(buf):
    off, rows = 0, []
    while off < len(buf):
        name, off = text(buf, off)
        rows.append(dict(name=name.split("/")[-1], values=list(struct.unpack_from("<11i", buf, off))))
        off += 44
    if off != len(buf):
        raise ValueError("BomberAttr: not EOF-exact")
    return rows


def parse_map_group(buf):
    off, groups = 4, []
    for _ in range(struct.unpack_from("<I", buf, 0)[0]):
        name, off = text(buf, off)
        count = struct.unpack_from("<I", buf, off)[0]
        off += 4
        maps = []
        for _ in range(count):
            entry, off = text(buf, off)
            maps.append(entry)
        groups.append(dict(name=name, maps=maps))
    if off != len(buf):
        raise ValueError("MapGroup: not EOF-exact")
    return groups


# --- map ------------------------------------------------------------------
def parse_map(buf):
    """Trả metadata của một map. `parse` = exact | partial (đuôi chưa hiểu)."""
    version, d1, d2, width, height = struct.unpack_from("<5i", buf, 0)
    block8 = struct.unpack_from("<8i", buf, 20)
    block4 = struct.unpack_from("<4i", buf, 52)
    spawn_count, off = block8[0], 80  # KR: 3 dword sau block4 (VN: 2)
    if version not in (1000, 1001, 1003) or not (0 < width <= 512 and 0 < height <= 512 and 0 <= spawn_count <= 64):
        raise ValueError(f"header version={version} size={width}x{height} spawns={spawn_count}")
    off += 8 * spawn_count
    rects = struct.unpack_from("<i", buf, off)[0]
    if not 0 <= rects <= 64:
        raise ValueError(f"rect count {rects}")
    off += 4 + 16 * rects
    cell = 2 if version == 1000 else 3
    cells = struct.unpack_from(f"<{width * height * cell}i", buf, off)
    off += width * height * cell * 4
    info = dict(version=version, d1=d1, d2=d2, width=width, height=height, spawns=spawn_count,
                header=[block8[1], block8[2], block8[7], *block4],
                object_cells=sum(1 for value in cells[1::cell] if value != -1),
                drops=[], placements=[], scripted=False, script_tags=[], script_funcs=[],
                script_triggers=[], parse="partial")
    try:
        count = struct.unpack_from("<i", buf, off)[0]
        off += 4
        if not 0 <= count <= 512:
            raise ValueError("drop count")
        for _ in range(count):
            name, off = text(buf, off)
            info["drops"].append([name, struct.unpack_from("<i", buf, off)[0]])
            off += 4
        _, count = struct.unpack_from("<2i", buf, off)
        off += 8
        if not 0 <= count <= 4096:
            raise ValueError("placement count")
        for _ in range(count):
            name, off = text(buf, off)
            info["placements"].append(name)
            off += 3  # x, y, xác suất %
        blobs = []
        for _ in range(3):
            if off + 4 > len(buf):
                break
            length = struct.unpack_from("<i", buf, off)[0]
            off += 4
            if not 0 <= length <= len(buf) - off:
                raise ValueError("blob length")
            blobs.append(buf[off:off + length])
            off += length
        if off != len(buf):
            raise ValueError("trailing bytes")
    except (ValueError, struct.error):
        return info
    info["parse"] = "exact"
    script = max(blobs, key=len).decode("cp949", "replace") if blobs else ""
    if "<" in script:
        info["scripted"] = True
        info["script_tags"] = sorted(set(re.findall(r"<([A-Za-z_]\w*)", script)))
        info["script_funcs"] = sorted({name.lower() for name in re.findall(r'\bfunc\s*=\s*"([^"]+)"', script)})
        info["script_triggers"] = sorted({name.lower() for name in
                                          re.findall(r'<Trigger[^>]*\btype\s*=\s*"([A-Za-z]\w*)"', script)})
    return info


# --- phân nhóm (suy từ tên) -----------------------------------------------
THEMES = {  # họ map → mục wiki "맵" (chỉ ghép khi tên khớp rõ)
    "Village": "빌리지", "Forest": "포레스트", "Factory": "팩토리", "Camp": "캠프", "Patrit": "해적",
    "Tomb": "무덤", "Cemetery": "공동묘지", "Ice": "아이스", "Desert": "데저트", "Mine": "광산",
    "Ocean": "바다", "Aqua": "아쿠아", "Cookie": "쿠키", "PangLand": "팡랜드", "Space": "우주맵",
    "Island": "보물섬", "LovelyRoom": "포키의러블리룸", "Kfg": "켄터키", "Xmas": "크리스마스 (đã ngừng)",
}
MISSION = {  # wiki "미션 맵" và mode riêng có tên khớp rõ
    "Bula": "불라", "Kart": "카트", "Rabbit": "바니팜", "Hamster": "햄스터팜", "Wriggle": "꼬물이",
    "Zombie": "좀비", "Gumiho": "구미호", "Clover": "클로버", "Coloring": "색칠하기", "CAPong": "크아퐁",
    "Baseball": "프로야구", "Zzolman": "쫄맨 (đã ngừng)", "RescueET": "외계인구출 (đã ngừng)",
    "SuperFighter": "슈퍼파이터 (sự kiện)", "SnowyFields": "눈싸움 (sự kiện, chưa chắc)",
    "PopcadeA": "팝케이드", "PopcadeB": "팝케이드", "PopcadeC": "팝케이드",
}
MONSTER = {"MonGolem": "골렘대침공", "MonTurtle": "거북대격돌", "MonSeal": "물개의분노",
           "MonPen": "황제의귀환", "MonRobot": "로봇대공습"}
VARIANT_PREFIXES = ("Ex", "Retro", "Event", "Novice", "NXCL_", "LJ_", "NL_", "WL_", "BSBig", "BSSmall",
                    "CaJJ_", "Fm_", "WinningStreak_")
CATEGORIES = [
    ("battle_theme", "Map đấu thường theo theme"),
    ("battle_variant", "Biến thể của map đấu (tiền tố Ex/Retro/Event/Novice/NXCL/ladder…)"),
    ("mission", "Nhiệm vụ và mode riêng"),
    ("monster", "Monster mode"),
    ("rpg", "RPG và CA Hero"),
    ("hidden", "Hidden Catch"),
    ("bonus", "Màn thưởng"),
    ("lobby", "Sảnh, cửa hàng, tutorial, map tuỳ biến"),
    ("other", "Chưa phân nhóm"),
]


def family_of(name):
    return re.sub(r"_?\d+$", "", name)


def base_theme(family):
    """Tên theme gốc của một biến thể, nếu khớp rõ."""
    stem, stripped = family, True
    while stripped:
        stripped = False
        for prefix in VARIANT_PREFIXES + ("AIBtl",):
            if stem.startswith(prefix) and len(stem) > len(prefix):
                stem, stripped = stem[len(prefix):], True
    stem = re.sub(r"N$", "", stem) if "Retro" in family else stem
    for theme in list(THEMES) + list(MISSION):
        if stem.lower() == theme.lower():
            return theme
    return None


def classify(family):
    lower = family.lower()
    if lower.startswith(("lobby", "custommap")) or lower in (
            "square", "shopmap", "tlshopmap", "capassshopmap", "dailymissionshop", "magicmarketmap",
            "selectionbox", "tutorial"):
        return "lobby", ""
    if lower.startswith("hdn_") or lower in ("hidden", "hiddencatchtest"):
        return "hidden", "히든캐치"
    if lower.startswith("bonus"):
        return "bonus", ""
    if lower.startswith(("mon", "fm_mon")):
        return "monster", MONSTER.get(family, MONSTER.get(family.replace("Fm_", ""), ""))
    if lower.startswith("rpg") or lower == "cahero":
        return "rpg", "크아 히어로" if lower == "cahero" else ""
    for theme, wiki in THEMES.items():
        if lower == theme.lower():
            return "battle_theme", wiki
    for theme, wiki in MISSION.items():
        if lower == theme.lower():
            return "mission", wiki
    base = base_theme(family)
    if base and family.startswith(VARIANT_PREFIXES):
        return "battle_variant", f"biến thể của {base}"
    return "other", ""


# --- dựng census ----------------------------------------------------------
def build(client):
    packs, sources = {}, {}
    for idx in sorted(client.glob("*.idx")):
        pack = Pack(client, idx.stem)
        packs[idx.stem] = pack
        groups = collections.Counter(name.split("/")[0] for name, *_ in pack.order)
        sources[idx.stem] = dict(entries=len(pack.order), idx_sha256=sha256(pack.idx),
                                 idd_bytes=pack.idd.stat().st_size, groups=dict(sorted(groups.items())))
    for stem in ("FxMap", "FxContents"):
        sources[stem]["idd_sha256"] = sha256(packs[stem].idd)
    attr = packs["FxContents"]
    item_attr = parse_item_attr(attr.read("Attr/ItemAttr"))
    characters = parse_bomber_attr(attr.read("Attr/BomberAttr"))
    map_groups = parse_map_group(attr.read("Attr/MapGroup"))

    maps, failed = [], []
    fx_map = packs["FxMap"]
    for name, *_ in fx_map.order:
        if not name.startswith("Map/"):
            continue
        short = name.split("/", 1)[1]
        try:
            info = parse_map(fx_map.read(name))
        except (ValueError, struct.error) as error:
            failed.append(dict(name=short, error=str(error)))
            continue
        category, wiki = classify(family_of(short))
        maps.append(dict(name=short, family=family_of(short), category=category, wiki=wiki, **info))

    drop_maps, place_maps = collections.Counter(), collections.Counter()
    for entry in maps:
        drop_maps.update({name for name, weight in entry["drops"] if weight > 0})
        place_maps.update(set(entry["placements"]))
    # Tên trong file map lệch hoa thường/tiền tố so với ItemAttr (ItemAirShip, Item/RedDevil, Item/Bomb…).
    # Ghép theo tên chuẩn hoá để đếm; cách client phân giải tên chưa được đo.
    def keys(name):
        base = name.split("/")[-1].lower()
        return {name.lower(), base, base[4:] if base.startswith("item") else "item" + base}

    lookup = {}
    for item in item_attr["items"]:
        item["maps_drop"] = item["maps_placed"] = 0
        for key in keys(item["name"]) | {item["sprite"].lower()}:
            lookup.setdefault(key, item)
    unknown = []
    for name in sorted(set(drop_maps) | set(place_maps)):
        item = lookup.get(name.lower()) or next((lookup[key] for key in sorted(keys(name)) if key in lookup), None)
        if item is None:
            unknown.append(name)
            continue
        item["maps_drop"] += drop_maps.get(name, 0)
        item["maps_placed"] += place_maps.get(name, 0)
    themes = sorted(name.split("/", 1)[1] for name, *_ in fx_map.order if name.startswith("Theme/"))
    return dict(
        schema="kre-content-census-v1", build="KR00967", client_exe_sha256=sha256(client / "CA.exe"),
        limits="Metadata tĩnh từ pack. category/wiki suy từ tên; không phải bằng chứng runtime hay wire.",
        packs=sources, maps=maps, maps_failed=failed, map_groups=map_groups, themes=themes,
        ships=item_attr["ships"], items=item_attr["items"], item_header_total=item_attr["header_total"],
        map_item_names_not_in_item_attr=[dict(name=name, maps_drop=drop_maps.get(name, 0),
                                              maps_placed=place_maps.get(name, 0)) for name in unknown],
        characters=characters,
    )


# --- render ---------------------------------------------------------------
ITEM_GROUPS = {  # theo PROTOCOL#item-families; nhóm khác để trống
    1: "Chỉ số và năng lực nhặt trong trận (Bomb/Fire/Roller, Shoe, Punch, MaxPower…)",
    2: "Phương tiện cưỡi và biến hình",
    21: "Phi tiêu (Dart)", 22: "Bình oxy", 23: "Kim và biến thể", 24: "Máy quét (Scanner)",
    25: "Thuốc (Potion)", 26: "Lò xo (Spring)", 27: "Khoan (Drill)", 28: "Bom điều khiển từ xa",
    29: "Khiên (Shield)", 30: "Cảm biến (Sensor)", 99: "Tiền Lucci rơi trong trận",
}
MEASURED = {  # trạng thái đã có trong PROTOCOL/PLAN, không phải kết quả của tool này
    "Item/ItemBomb": "đã đo native (2/4/8 người)", "Item/ItemFire": "đã đo native",
    "Item/ItemRoller": "đã đo native", "Item/ItemMaxPower": "đã đo native (CP045)",
    "Item/ItemNiddle": "đã đo native trên Python (kim), chưa port Rust",
}


def spans(values):
    values = sorted(set(values))
    return str(values[0]) if len(values) == 1 else f"{values[0]}–{values[-1]}"


def render(census):
    maps, items = census["maps"], census["items"]
    families = collections.defaultdict(list)
    for entry in maps:
        families[entry["family"]].append(entry)
    scripted = [entry for entry in maps if entry["scripted"]]
    exact = sum(1 for entry in maps if entry["parse"] == "exact")
    total_maps = len(maps) + len(census["maps_failed"])
    used = [item for item in items if item["maps_drop"] or item["maps_placed"]]
    lines = [
        "# Kiểm kê nội dung client KR 00967",
        "",
        "Sinh từ [census.json](content/census.json) bằng `python3 tools/content_census.py --generate`; không sửa tay.",
        "",
        "Tài liệu này trả lời câu hỏi **\"full game gồm những gì\"** bằng dữ liệu tĩnh trong pack của client.",
        "Nó không chứng minh client hay server làm gì với từng mục. Cột nhóm nội dung và cột wiki là **suy từ tên**;",
        "cột \"server phải làm gì\" để trống cho tới khi có ca chạy thử. Thứ tự công việc vẫn chỉ ở",
        "[PLAN](PLAN.md#delivery-target).",
        "",
        "## Tóm tắt",
        "",
        "| Hạng mục | Số lượng | Ghi chú |",
        "|---|---|---|",
        f"| Mục trong 13 pack | {sum(pack['entries'] for pack in census['packs'].values()):,} | chỉ mục đọc trọn |",
        f"| Map | {total_maps:,} thuộc {len(families)} họ tên | {exact:,} đọc trọn file, "
        f"{len(maps) - exact} đọc tới hết lưới ô, {len(census['maps_failed'])} lỗi |",
        f"| Map có script | {len(scripted):,} | {exact - len(scripted):,} map đọc trọn và không có script |",
        f"| Map có bảng rơi item riêng | {sum(1 for entry in maps if entry['drops']):,} | "
        "trọng số nằm trong file map |",
        f"| Map có item đặt sẵn | {sum(1 for entry in maps if entry['placements']):,} | vị trí và xác suất trong file map |",
        f"| Dòng item trong `Attr/ItemAttr` | {len(items)} | {len(used)} dòng được ít nhất một map dùng |",
        f"| Phương tiện trong `Attr/ItemAttr` | {len(census['ships'])} | mục `Ship/*` |",
        f"| Dòng nhân vật trong `Attr/BomberAttr` | {len(census['characters'])} | "
        f"{len({row['name'] for row in census['characters']})} tên khác nhau |",
        f"| Theme trong `FxMap/Theme` | {len(census['themes'])} | gồm cả icon và mục ngẫu nhiên |",
        "",
        "Ba điều rút ra cho server:",
        "",
        "1. **Phần lớn file map mang sẵn một bảng tên item kèm trọng số.** `boom-re/docs/map.md` đọc khối này là bảng rơi",
        "   item của map; đây là nguồn tốt nhất hiện có cho tỉ lệ rơi theo từng map, tốt hơn wiki. Ba điều chưa đo: bên",
        "   nào dùng bảng trong trận thật (client tự sinh hay server gốc gửi xuống), bảng kết hợp thế nào với trọng số",
        "   mặc định trong `Attr/ItemAttr` (Village00 chỉ liệt kê tiền Lucci), và vì sao ExVillage01 đang chạy native",
        "   không có bảng, chỉ có bốn item đặt sẵn.",
        f"2. **Chỉ {len(used)} trong {len(items)} dòng item được file map nhắc tới.** Các dòng còn lại chưa rõ được cấp qua đường",
        "   nào (script, phần thưởng, cửa hàng, sự kiện). Thứ tự ưu tiên nên lấy từ bảng \"item được map dùng\" bên dưới.",
        f"3. **{exact - len(scripted):,} map không có script.** Đây là nhóm có khả năng chỉ cần server gửi đúng ID map và bảng",
        "   item; điều đó là giả thuyết cho tới khi thử một map mỗi họ.",
        "",
        "## Map theo nhóm nội dung",
        "",
        "Cột: họ tên (bỏ số cuối), số map, kích thước lưới, số điểm spawn, số map có script, số item khác nhau có",
        "trọng số lớn hơn 0 trong bảng rơi của họ, cặp `d1/d2` trong header (chưa rõ nghĩa), mục wiki nếu tên khớp rõ.",
        "",
    ]
    by_category = collections.defaultdict(list)
    for family, entries in families.items():
        by_category[entries[0]["category"]].append((family, entries))
    for key, title in CATEGORIES:
        rows = sorted(by_category.get(key, []), key=lambda row: (-len(row[1]), row[0].lower()))
        if not rows:
            continue
        count = sum(len(entries) for _, entries in rows)
        lines += [f"### {title} — {count} map, {len(rows)} họ", "",
                  "| Họ | Map | Lưới | Spawn | Có script | Item rơi | d1/d2 | Wiki |", "|---|---|---|---|---|---|---|---|"]
        for family, entries in rows:
            sizes = sorted({f"{e['width']}×{e['height']}" for e in entries})
            size = sizes[0] if len(sizes) == 1 else f"{sizes[0]} +{len(sizes) - 1} cỡ"
            dvals = sorted({f"{e['d1']}/{e['d2']}" for e in entries})
            drops = {name for e in entries for name, weight in e["drops"] if weight > 0}
            lines.append(f"| {family} | {len(entries)} | {size} | {spans(e['spawns'] for e in entries)} | "
                         f"{sum(1 for e in entries if e['scripted'])} | {len(drops)} | {', '.join(dvals)} | "
                         f"{entries[0]['wiki']} |")
        lines.append("")
    if census["maps_failed"]:
        lines += ["Map chưa đọc được header: " + ", ".join(row["name"] for row in census["maps_failed"]) + ".", ""]
    partial = [entry["name"] for entry in maps if entry["parse"] != "exact"]
    if partial:
        lines += ["Map chỉ đọc tới hết lưới ô (đuôi file có cấu trúc riêng, bảng rơi và script chưa lấy được): "
                  + ", ".join(partial) + ".", ""]
    lines += [
        "### Nhóm map do client khai báo (`Attr/MapGroup`)",
        "",
        "| Nhóm | Số map | Map |", "|---|---|---|",
    ]
    for group in census["map_groups"]:
        lines.append(f"| {group['name']} | {len(group['maps'])} | {', '.join(group['maps'])} |")
    lines += [
        "",
        "## Item",
        "",
        "### Item được map dùng",
        "",
        "Xếp theo số map có item trong bảng rơi với trọng số lớn hơn 0. \"Đặt sẵn\" là số map đặt item ở vị trí cố định. Tên trong file map",
        "được ghép với `Attr/ItemAttr` không phân biệt hoa thường và tiền tố `Item`. Cột trạng thái lấy từ",
        "[PROTOCOL](PROTOCOL.md#item-families), không phải kết quả của tool này.",
        "",
        "| Kind | Tên | Nhóm | Trọng số mặc định | Map có trong bảng rơi | Map đặt sẵn | Trạng thái |",
        "|---|---|---|---|---|---|---|",
    ]
    for item in sorted(used, key=lambda row: (-row["maps_drop"], -row["maps_placed"], row["kind"])):
        lines.append(f"| {item['kind']} | {item['name']} | {item['group']} | {item['weight']} | {item['maps_drop']} | "
                     f"{item['maps_placed']} | {MEASURED.get(item['name'], '')} |")
    missing = census["map_item_names_not_in_item_attr"]
    if missing:
        lines += ["", f"{len(missing)} tên item trong file map không khớp tên nào trong `Attr/ItemAttr` "
                  "sau khi đã ghép không phân biệt hoa thường và tiền tố `Item`; số trong ngoặc là số map có trong bảng rơi/đặt sẵn: "
                  + ", ".join(f"{row['name']} ({row['maps_drop']}/{row['maps_placed']})" for row in missing) + "."]
    lines += [
        "",
        "### Toàn bộ dòng item theo nhóm",
        "",
        "Nhóm là trường `group` trong bảng. Tên nhóm theo PROTOCOL khi đã biết; để trống khi chưa có nguồn.",
        "",
        "| Nhóm | Số dòng | Được map dùng | Ý nghĩa đã biết | Ví dụ |", "|---|---|---|---|---|",
    ]
    groups = collections.defaultdict(list)
    for item in items:
        groups[item["group"]].append(item)
    for group, rows in sorted(groups.items(), key=lambda row: (-len(row[1]), row[0])):
        sample = ", ".join(row["name"].split("/")[-1] for row in rows[:4]) + ("…" if len(rows) > 4 else "")
        lines.append(f"| {group} | {len(rows)} | {sum(1 for row in rows if row['maps_drop'] or row['maps_placed'])} | "
                     f"{ITEM_GROUPS.get(group, '')} | {sample} |")
    lines += [
        "",
        f"### Phương tiện — {len(census['ships'])} mục `Ship/*`",
        "",
        ", ".join(ship["name"].split("/")[-1] for ship in census["ships"]) + ".",
        "",
        "## Nhân vật",
        "",
        "Mỗi dòng `Attr/BomberAttr` có 11 số. Theo `boomkr-re/docs/kr-data.md`, đây là chỉ số nhân vật (bom, tầm nổ,",
        "tốc độ và giá trị tối đa); ánh xạ từng cột cho KR vẫn theo nguồn đó, tool này không xác nhận lại.",
        "",
        "| Tiền tố | Số dòng | Tên |", "|---|---|---|",
    ]
    prefixes = collections.defaultdict(list)
    for row in census["characters"]:
        prefixes[row["name"][0]].append(row["name"])
    for prefix, names in sorted(prefixes.items(), key=lambda row: -len(row[1])):
        lines.append(f"| {prefix} | {len(names)} | {', '.join(names)} |")
    lines += [
        "",
        "## Mode",
        "",
        "Pack không có một bảng liệt kê mode; dưới đây là hai danh sách đặt cạnh nhau, **chưa ghép từng dòng**.",
        "Mã mode trên wire (trường mode khi tạo phòng) và luật thắng của từng mode phải lấy từ RE client.",
        "",
        "- **Theo wiki (trang 게임 모드):** 일반모드 (thường), 몬스터모드 (monster), 협공배틀 (đánh phối hợp), bốn kiểu",
        "  게임타입: 부활대전 (hồi sinh), 고슴도치 (nhím), 대장잡기 (bắt thủ lĩnh), 시한폭탄 (bom hẹn giờ); 팝케이드,",
        "  혼자놀기 (chơi một mình); hai mode sự kiện cũ 눈싸움 và 슈퍼파이터.",
        "- **Nhóm giao diện theo mode trong `FxContents`:** "
        + ", ".join(f"{name} ({count})" for name, count in sorted(census["packs"]["FxContents"]["groups"].items())
                    if name != "Attr") + ".",
        "- **Pack riêng:** FxHiddenCatch (Hidden Catch), FxMonster (quái mùa 1 và mùa 2, tàu).",
        "- **Theo số map ở trên:** " + ", ".join(
            f"{title.split(' (')[0].lower()} {sum(len(e) for _, e in by_category.get(key, []))}"
            for key, title in CATEGORIES if by_category.get(key)) + ".",
    ]
    tags, funcs, triggers = collections.Counter(), collections.Counter(), collections.Counter()
    for entry in scripted:
        tags.update(entry["script_tags"])
        funcs.update(entry["script_funcs"])
        triggers.update(entry["script_triggers"])
    lines += [
        "",
        "## Script map",
        "",
        f"{len(scripted):,} map có `<mapscript>`. Số trong ngoặc là số map dùng. Script do client đọc; phần nào cần server",
        "tham gia (đồng bộ trigger, quái, phần thưởng) chưa được đo.",
        "",
        "- **Thẻ:** " + ", ".join(f"{name} ({count})" for name, count in tags.most_common(40)) + ".",
        "- **Hàm của Action:** " + ", ".join(f"{name} ({count})" for name, count in funcs.most_common()) + ".",
        "- **Loại Trigger:** " + ", ".join(f"{name} ({count})" for name, count in triggers.most_common(30)) + ".",
        "",
        "## Điền cột \"server phải làm gì\"",
        "",
        "Mỗi họ map và mỗi nhóm item cần một ca chạy thử trên client thật để xếp vào một trong ba loại:",
        "",
        "- **A — chỉ cần dữ liệu:** server gửi đúng ID và bảng item, client tự lo phần còn lại.",
        "- **B — dùng packet đã có:** nhặt, tranh item, báo sống/chết như đã đo; kiểm một ca mỗi nhóm.",
        "- **C — cần packet hoặc trạng thái mới:** phải RE trước khi làm.",
        "",
        "Chưa họ nào được xếp loại ngoài `ExVillage` (ExVillage01 đã chạy native theo PLAN). Thứ tự thử đề xuất:",
        "map đấu thường không script trước, sau đó các item được nhiều map dùng nhất mà chưa đo.",
        "",
    ]
    return "\n".join(lines)


def dump(census):
    """JSON hợp lệ, mỗi phần tử của danh sách lớn một dòng để diff đọc được."""
    def compact(value):
        return json.dumps(value, ensure_ascii=False, separators=(",", ":"), sort_keys=True)

    parts = []
    for key in sorted(census):
        value = census[key]
        if isinstance(value, list) and value and isinstance(value[0], dict):
            body = ",\n".join(compact(row) for row in value)
            parts.append(f"{compact(key)}:[\n{body}\n]")
        else:
            parts.append(f"{compact(key)}:{compact(value)}")
    return "{\n" + ",\n".join(parts) + "\n}\n"


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--client", type=Path, help="thư mục client KR 00967 gốc (chỉ đọc)")
    parser.add_argument("--generate", action="store_true")
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    if args.generate:
        if not args.client:
            parser.error("--generate cần --client")
        census = build(args.client)
        CENSUS.parent.mkdir(parents=True, exist_ok=True)
        CENSUS.write_text(dump(census))
        OUTPUT.write_text(render(census))
        print(f"wrote {CENSUS.relative_to(ROOT)} and {OUTPUT.relative_to(ROOT)}")
    elif args.check:
        census = json.loads(CENSUS.read_text())
        if OUTPUT.read_text() != render(census):
            raise SystemExit("CONTENT stale: run python3 tools/content_census.py --client <dir> --generate")
        print(f"Content census OK: {len(census['maps'])} maps, {len(census['items'])} items; static metadata only.")
    else:
        parser.error("chọn --generate hoặc --check")


if __name__ == "__main__":
    main()
