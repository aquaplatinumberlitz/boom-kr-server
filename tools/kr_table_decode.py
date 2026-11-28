"""Giải tĩnh các bảng dữ liệu ngoài `Attr/` của client KR 00967 thành JSON nhẹ.

Đọc pack bằng reader của `tools/content_census.py` (không viết lại), giải các entry XML mà CONTENT
còn ghi "chưa giải" và ghi JSON đã giải vào `docs/content/client-tables/`, kèm manifest pin SHA-256
của từng file pack nguồn và của từng entry đã giải mã. Không ghi .idx/.idd/.exe hay blob thô.

    python3 tools/kr_table_decode.py --client /home/admin/boom/clients/kr-00967 --generate
    python3 tools/kr_table_decode.py --check                      # không cần client (CI)
    python3 tools/kr_table_decode.py --check --client <dir>       # giải lại và so từng byte

Định dạng và mức chắc chắn của từng trường ở `docs/content/client-data-tables.md`. Tên trường lấy
nguyên từ file; nghĩa gameplay chỉ là `candidate` nếu chưa có tham chiếu chéo chứng minh.
"""
import argparse
import collections
import hashlib
import json
import re
import struct
import sys
import xml.etree.ElementTree as ET
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from content_census import Pack, item_keys, parse_item_attr, parse_map, sha256  # noqa: E402

ROOT = Path(__file__).resolve().parents[1]
OUT_DIR = ROOT / "docs/content/client-tables"
MANIFEST = "manifest.json"
DOC = ROOT / "docs/content/client-data-tables.md"
CENSUS = ROOT / "docs/content/census.json"
SCHEMA = "kr00967-client-tables-v1"
BEGIN, END = "<!-- BEGIN kr_table_decode -->", "<!-- END kr_table_decode -->"
SHEET_NS = "{urn:schemas-microsoft-com:office:spreadsheet}"


class DecodeError(ValueError):
    pass


# --- XML khoan dung ---------------------------------------------------------
# Các file script/bảng có thuộc tính không ngoặc kép, comment không đóng, ghi chú "#" ngoài thẻ và
# chữ hoa thường lẫn lộn, nên parser XML chuẩn từ chối. Scanner này chỉ tách thẻ; mọi chỗ lệch chuẩn
# được đếm vào `issues` để không giấu đi.
TOKEN = re.compile(
    r"<!--(.*?)(-->|\Z)|<\?.*?\?>|<!\[CDATA\[.*?\]\]>|<!DOCTYPE[^>]*>"
    r"|</\s*([A-Za-z_][\w:.-]*)\s*>"
    r"|<([A-Za-z_][\w:.-]*)((?:[^<>\"']|\"[^\"]*\"|'[^']*')*?)(/?)>", re.S)
ATTR = re.compile(r"([A-Za-z_][\w:.-]*)\s*=\s*(\"([^\"]*)\"|'([^']*)'|([^\s\"'=<>/]+))")
ENTITY = re.compile(r"&(lt|gt|amp|quot|apos|#\d+|#x[0-9a-fA-F]+);")


def unescape(value):
    def repl(match):
        name = match.group(1)
        if name.startswith("#x"):
            return chr(int(name[2:], 16))
        if name.startswith("#"):
            return chr(int(name[1:]))
        return {"lt": "<", "gt": ">", "amp": "&", "quot": '"', "apos": "'"}[name]
    return ENTITY.sub(repl, value)


class Node:
    __slots__ = ("tag", "attrs", "children", "notes", "comments")

    def __init__(self, tag, attrs):
        self.tag, self.attrs, self.children, self.notes, self.comments = tag, attrs, [], [], []

    def get(self, key, default=None):
        key = key.lower()
        return next((v for k, v in self.attrs if k.lower() == key), default)

    def attr_dict(self, issues=None):
        out = {}
        for key, value in self.attrs:
            if key in out:
                if issues is not None:
                    issues.setdefault("duplicate_attr", []).append([self.tag, key, value])
                continue
            out[key] = value
        return out

    def is_(self, tag):
        return self.tag.lower() == tag.lower()


def scan(text):
    """Trả (root ảo, issues). root.children là các phần tử cấp cao nhất."""
    root, issues = Node(None, []), collections.Counter()
    stack, pos, pending = [root], 0, []
    closed_root = False
    for match in TOKEN.finditer(text):
        chunk = text[pos:match.start()]
        pos = match.end()
        if chunk.strip():
            parent = stack[-1]
            lines = chunk.split("\n")
            if closed_root and parent is root:
                issues["after_root"] += 1
            for index, line in enumerate(lines):
                line = line.strip()
                if not line:
                    continue
                if not line.startswith("#"):
                    issues["stray_text"] += 1
                    continue
                if index == 0 and parent.children:  # cùng dòng với thẻ trước: ghi chú cho thẻ đó
                    parent.children[-1].notes.append(line[1:].strip())
                else:
                    pending.append(line[1:].strip())
        if match.group(1) is not None:
            if not match.group(2):
                issues["unterminated_comment"] += 1
            stack[-1].comments.append(match.group(1))
            if closed_root and len(stack) == 1:
                issues["after_root"] += 1
            continue
        if match.group(3):
            name = match.group(3).lower()
            for depth in range(len(stack) - 1, 0, -1):
                if stack[depth].tag.lower() == name:
                    if depth != len(stack) - 1:
                        issues["implicit_close"] += len(stack) - 1 - depth
                    del stack[depth:]
                    break
            else:
                issues["stray_close"] += 1
            if len(stack) == 1:
                closed_root = True
            continue
        if match.group(4) is None:
            continue  # khai báo <?xml?>, CDATA, DOCTYPE
        body = match.group(5)
        if ATTR.sub("", body).strip():
            issues["junk_in_tag"] += 1
        attrs = []
        for attr in ATTR.finditer(body):
            if attr.group(5) is not None:
                issues["unquoted_attr"] += 1
            value = next(v for v in attr.group(3, 4, 5) if v is not None)
            attrs.append((attr.group(1), unescape(value)))
        node = Node(match.group(4), attrs)
        node.notes, pending = pending, []
        if len(stack) == 1 and closed_root:
            issues["after_root"] += 1
        stack[-1].children.append(node)
        if not match.group(6):
            stack.append(node)
    if text[pos:].strip():
        issues["stray_text"] += 1
    if len(stack) > 1:
        issues["unclosed"] += len(stack) - 1
    return root, issues


def decode_text(data):
    """Chọn bảng mã: khai báo encoding nếu có, rồi UTF-8 chặt, rồi cp949 chặt."""
    head = re.match(rb"\s*<\?xml[^>]*encoding=[\"']([\w-]+)", data)
    if head:
        return data.decode(head.group(1).decode()), head.group(1).decode().lower()
    for encoding in ("ascii", "utf-8", "cp949"):
        try:
            return data.decode(encoding), encoding
        except UnicodeDecodeError:
            pass
    raise DecodeError("không giải được bảng mã")


def entry_payload(data, name):
    if len(data) < 5 or struct.unpack_from("<I", data)[0] != len(data) - 4:
        raise DecodeError(f"{name}: thiếu tiền tố độ dài u32")
    return data[4:]


def one_root(root, tag, name):
    elements = root.children
    if not elements or not elements[0].is_(tag):
        raise DecodeError(f"{name}: gốc phải là <{tag}>")
    return elements[0]


def require(node, tag, keys=None, name=""):
    if not node.is_(tag):
        raise DecodeError(f"{name}: gặp <{node.tag}>, chờ <{tag}>")
    if keys is not None:
        got = sorted(k.lower() for k, _ in node.attrs)
        if got != sorted(k.lower() for k in keys):
            raise DecodeError(f"{name}: <{node.tag}> có thuộc tính {got}, chờ {sorted(keys)}")
    return node


def to_int(value, name):
    try:
        return int(value.strip())
    except (AttributeError, ValueError):
        raise DecodeError(f"{name}: giá trị không phải số nguyên: {value!r}") from None


def leaf(node, name):
    if node.children:
        raise DecodeError(f"{name}: <{node.tag}> có phần tử con chưa hiểu")
    return node


# --- bộ giải theo bảng ------------------------------------------------------
def decode_skill_table(root, name, issues):
    table = one_root(root, "SkillTable", name)
    skills = []
    for skill in table.children:
        require(skill, "Skill", ["Type", "Name"], name)
        props = []
        for index, prop in enumerate(skill.children):
            require(prop, "Property", ["Idx", "Name"], name)
            stats = []
            for stat in prop.children:
                leaf(require(stat, "Stat", ["Level", "Value"], name), name)
                stats.append([to_int(stat.get("Level"), name), to_int(stat.get("Value"), name)])
            if to_int(prop.get("Idx"), name) != index:
                issues["idx_not_sequential"] = issues.get("idx_not_sequential", 0) + 1
            props.append(dict(idx=to_int(prop.get("Idx"), name), name=prop.get("Name"), stats=stats))
        skills.append(dict(type=to_int(skill.get("Type"), name), name=skill.get("Name"),
                           note=" ".join(skill.notes) or None, properties=props))
    commented = []
    for body in table.comments:
        inner, _ = scan(body)
        for node in inner.children:
            if node.is_("Skill"):
                commented.append(dict(type=to_int(node.get("Type"), name), name=node.get("Name")))
    return dict(skills=skills, commented_out=commented), len(skills)


def decode_equip_table(root, name, issues):
    table = one_root(root, "EquipItemTable", name)
    items = []
    for item in table.children:
        require(item, "EquipItem", ["attr1", "attr2", "Name"], name)
        stats = []
        for stat in item.children:
            leaf(require(stat, "Stat", ["Level", "Damage", "Defense", "Critical", "Miss"], name), name)
            stats.append({k.lower(): to_int(v, name) for k, v in stat.attrs})
        items.append(dict(attr1=to_int(item.get("attr1"), name), attr2=to_int(item.get("attr2"), name),
                          name=item.get("Name"), stats=stats))
    return dict(items=items), len(items)


def decode_gadget_table(root, name, issues):
    table = one_root(root, "GadgetTable", name)
    gadgets = []
    for skill in table.children:
        require(skill, "Skill", ["Type", "Name"], name)
        props = []
        for prop in skill.children:
            leaf(require(prop, "Property", ["Name", "Value"], name), name)
            props.append(dict(name=prop.get("Name"), value=to_int(prop.get("Value"), name),
                              note=" ".join(prop.notes) or None))
        gadgets.append(dict(type=to_int(skill.get("Type"), name), name=skill.get("Name"),
                            note=" ".join(skill.notes) or None, properties=props))
    return dict(gadgets=gadgets), len(gadgets)


def trigger_table(node, name, issues):
    times = []
    for time in node.children:
        require(time, "Time", None, name)
        triggers = [leaf(require(t, "Trigger", None, name), name).attr_dict(issues) for t in time.children]
        times.append(dict(attrs=time.attr_dict(issues), triggers=triggers))
    return dict(attrs=node.attr_dict(issues), times=times)


def action_list(node, name, issues):
    steps = []
    for step in node.children:
        if step.is_("ActionList") or step.is_("TriggerTable"):
            raise DecodeError(f"{name}: <{step.tag}> lồng trong ActionList")
        steps.append([leaf(step, name).tag, step.attr_dict(issues)])
    return dict(attrs=node.attr_dict(issues), steps=steps)


def script_body(node, name, issues):
    """TriggerTable + ActionList của một ScriptObject hoặc một MonAction."""
    out = dict(trigger_tables=[], action_lists=[])
    for child in node.children:
        if child.is_("TriggerTable"):
            out["trigger_tables"].append(trigger_table(child, name, issues))
        elif child.is_("ActionList"):
            out["action_lists"].append(action_list(child, name, issues))
        elif child.is_("InitInfo"):
            out["init_info"] = leaf(child, name).attr_dict(issues)
        else:
            raise DecodeError(f"{name}: <{child.tag}> chưa hiểu trong script")
    return out


def decode_mon_action(root, name, issues):
    return script_body(one_root(root, "MonAction", name), name, issues), 1


def decode_stage_clear(root, name, issues):
    script = one_root(root, "mapscript", name)
    objects = []
    for clear in script.children:
        require(clear, "StageClearScript", [], name)
        for group in clear.children:
            require(group, "ScriptObjects", [], name)
            for obj in group.children:
                require(obj, "ScriptObject", [], name)
                objects.append(script_body(obj, name, issues))
    return dict(script_objects=objects), len(objects)


TILE_LAYERS = ("GroundTiles", "ObjectTiles", "DecoTiles", "FixedItems", "MonTiles")
EMPTY_ONLY = ("Character", "Env", "Time")


def single_value(section, keys, name):
    if len(section.children) != 1:
        raise DecodeError(f"{name}: <{section.tag}> phải có đúng một <Value>")
    return leaf(require(section.children[0], "Value", keys, name), name)


def decode_custom_map(root, name, issues):
    custom = one_root(root, "CustomMap", name)
    out = dict(sections=[c.tag for c in custom.children], layers={}, empty_sections=[])
    for section in custom.children:
        tag = section.tag
        if tag == "Version":
            out["version"] = to_int(single_value(section, ["Version"], name).get("Version"), name)
        elif tag == "MapSize":
            raw = to_int(single_value(section, ["MapSize"], name).get("MapSize"), name)
            out.update(map_size=raw, width=raw >> 16, height=raw & 0xFFFF)
        elif tag == "TeamInfo":
            value = single_value(section, ["TeamCount", "TeamUserCount"], name)
            out.update(team_count=to_int(value.get("TeamCount"), name),
                       team_user_count=to_int(value.get("TeamUserCount"), name))
        elif tag == "StartPoints":
            out["start_points"] = [to_int(leaf(require(v, "Value", ["TilePos"], name), name).get("TilePos"), name)
                                   for v in section.children]
        elif tag in TILE_LAYERS:
            palette, cells = [], []
            for value in section.children:
                leaf(require(value, "Value", ["Pos", "ObjectName"], name), name)
                tile = value.get("ObjectName")
                if tile not in palette:
                    palette.append(tile)
                cells.append([to_int(value.get("Pos"), name), palette.index(tile)])
            if cells:
                out["layers"][tag] = dict(palette=palette, cells=cells)
            else:
                out["empty_sections"].append(tag)
        elif tag in EMPTY_ONLY:
            if section.children or section.attrs:
                raise DecodeError(f"{name}: <{tag}> có nội dung chưa hiểu")
            out["empty_sections"].append(tag)
        elif tag == "Ban":
            ban = {}
            for group in section.children:
                ban[group.tag] = {leaf(i, name).tag: to_int(i.get("Value"), name) for i in group.children}
            out["ban"] = ban
        elif tag == "Localdata":
            out["localdata"] = single_value(section, None, name).attr_dict(issues)
        else:
            raise DecodeError(f"{name}: mục <{tag}> chưa hiểu")
    if out.get("width", 0) * out.get("height", 0) == 0:
        raise DecodeError(f"{name}: thiếu MapSize")
    return out, 1


def decode_tile_index_list(root, name, issues):
    data = one_root(root, "CustomDataList", name)
    out = {}
    for section in data.children:
        if section.is_("CustomTileIndexList"):
            tiles, limits = [], {}
            for index, tile in enumerate(section.children):
                leaf(require(tile, "Tile", None, name), name)
                keys = sorted(k.lower() for k, _ in tile.attrs)
                if keys not in (["idx", "name"], ["idx", "limitcount", "name"]):
                    raise DecodeError(f"{name}: <Tile> có thuộc tính {keys}")
                if to_int(tile.get("Idx"), name) != index:
                    raise DecodeError(f"{name}: Idx không liên tục tại {index}")
                tiles.append(tile.get("Name"))
                if tile.get("LimitCount") is not None:
                    limits[str(index)] = to_int(tile.get("LimitCount"), name)
            seen = collections.Counter(tiles)
            out.update(tiles=tiles, limit_count=limits,
                       duplicate_names=sorted(n for n, c in seen.items() if c > 1))
        elif section.is_("CustomItemList"):
            out["items"] = [leaf(require(i, "Item", ["Name"], name), name).get("Name") for i in section.children]
        else:
            raise DecodeError(f"{name}: mục <{section.tag}> chưa hiểu")
    return out, len(out.get("tiles", []))


TILE_LIST_SECTIONS = {"TemplateMapInfo": "Map", "ItemInfo": "Item", "SpecialBlock": "Tile", "MonsterInfo": "Monster"}


def uniform_rows(nodes, tag, name, issues):
    """Các phần tử lá cùng tên; `key_counts` đếm số dòng có từng thuộc tính (thuộc tính tuỳ chọn)."""
    rows, counts = [], collections.Counter()
    for node in nodes:
        row = leaf(require(node, tag, None, name), name).attr_dict(issues)
        counts.update(row.keys())
        rows.append(row)
    return dict(key_counts=dict(counts), rows=rows)


def decode_tile_list(root, name, issues):
    info = one_root(root, "CustomMapInfo", name)
    out = dict(legend=[c.strip() for c in root.comments])
    for section in info.children:
        if section.is_("TileInfo"):
            themes = []
            for theme in section.children:
                require(theme, "Theme", ["Name", "IDFName", "Desc"], name)
                themes.append(dict(theme.attr_dict(issues), tiles=uniform_rows(theme.children, "Tile", name, issues)))
            out["themes"] = themes
        elif section.tag in TILE_LIST_SECTIONS:
            out[section.tag] = uniform_rows(section.children, TILE_LIST_SECTIONS[section.tag], name, issues)
        else:
            raise DecodeError(f"{name}: mục <{section.tag}> chưa hiểu")
    rows = sum(len(t["tiles"]["rows"]) for t in out.get("themes", []))
    return out, rows


def decode_init(root, name, issues):
    init = one_root(root, "init", name)
    return dict(elements=[[leaf(c, name).tag, c.attr_dict(issues)] for c in init.children]), len(init.children)


def sheet_value(data):
    if data is None:
        return None
    kind, text = data.get(SHEET_NS + "Type"), "".join(data.itertext())
    if kind == "Number":
        number = float(text)
        return int(number) if number.is_integer() else number
    if kind in ("String", None):
        return text
    raise DecodeError(f"kiểu ô chưa hiểu: {kind}")


def sheet_rows(table):
    rows = []
    for row in table.findall(SHEET_NS + "Row"):
        index = row.get(SHEET_NS + "Index")
        while index is not None and len(rows) < int(index) - 1:
            rows.append([])
        cells = []
        for cell in row.findall(SHEET_NS + "Cell"):
            at = cell.get(SHEET_NS + "Index")
            while at is not None and len(cells) < int(at) - 1:
                cells.append(None)
            if cell.get(SHEET_NS + "MergeAcross") or cell.get(SHEET_NS + "Formula"):
                raise DecodeError("ô gộp hoặc công thức chưa hỗ trợ")
            cells.append(sheet_value(cell.find(SHEET_NS + "Data")))
        while cells and cells[-1] is None:
            cells.pop()
        rows.append(cells)
    return rows


def decode_sheet(data, name, issues):
    """Excel 2003 XML (SpreadsheetML): dòng 1 mô tả, dòng 2 tên cột, từ dòng 3 là dữ liệu."""
    root = ET.fromstring(data)
    if root.tag != SHEET_NS + "Workbook":
        raise DecodeError(f"{name}: không phải SpreadsheetML")
    sheets, primary = [], 0
    for sheet in root.findall(SHEET_NS + "Worksheet"):
        rows = sheet_rows(sheet.find(SHEET_NS + "Table"))
        if not any(rows):
            sheets.append(dict(name=sheet.get(SHEET_NS + "Name"), columns=[], rows=[]))
            continue
        header = rows[1]
        if not header or not all(isinstance(h, str) and h for h in header) or len(set(header)) != len(header):
            raise DecodeError(f"{name}: dòng 2 không phải tên cột")
        notes = rows[0] + [None] * (len(header) - len(rows[0]))
        body = rows[2:]
        if any(len(r) > len(header) for r in body):
            raise DecodeError(f"{name}: dòng dữ liệu dài hơn header")
        columns = [dict(name=h, description=notes[i] or None) for i, h in enumerate(header)]
        sheets.append(dict(name=sheet.get(SHEET_NS + "Name"), columns=columns,
                           rows=[r + [None] * (len(header) - len(r)) for r in body]))
        primary = primary or len(body)
    return dict(sheets=sheets), primary


# --- danh mục bảng ----------------------------------------------------------
# status: decoded = mọi phần tử/thuộc tính đã tách, cấu trúc lạ sẽ làm tool dừng; nghĩa từng trường
# ghi ở tài liệu với mức kr_static/candidate/unknown.
TABLES = [
    dict(out="skill-tables", anchor="skilltable", decoder=decode_skill_table, unit="skill",
         entries=[("FxContents", "RPG/SkillTable"), ("FxContents", "RPG/SnowBallSkillTable"),
                  ("FxContents", "RPG/ZzolmanSkillTable"), ("FxMonster", "MonsterSeason2/SkillTable")]),
    dict(out="equip-item-table", anchor="equipitemtable", decoder=decode_equip_table, unit="item",
         entries=[("FxContents", "RPG/EquipItemTable")]),
    dict(out="gadget-table", anchor="gadgettable", decoder=decode_gadget_table, unit="gadget",
         entries=[("FxContents", "SandBox/GadgetTable")]),
    dict(out="stage-clear-event-npc", anchor="stageclearevent", decoder=decode_stage_clear, unit="ScriptObject",
         entries=[("FxDesktop", "ClientData/StageClearEventNpc"), ("FxDesktop", "ClientData/StageClearEventNpcBoss")]),
    dict(out="mon-action", anchor="monaction", decoder=decode_mon_action, unit="template",
         prefix=("FxMap", "MapScriptTemplate/MonAction")),
    dict(out="custom-maps", anchor="custommap", decoder=decode_custom_map, unit="map",
         prefix=("fx", "CustomMap/CustomMap_")),
    dict(out="custom-tile-index-list", anchor="customtileindexlist", decoder=decode_tile_index_list, unit="tile",
         entries=[("fx", "CustomMap/CustomTileIndexList")]),
    dict(out="custom-tile-list", anchor="customtilelist", decoder=decode_tile_list, unit="tile theme",
         entries=[("fx", "CustomMap/CustomTileList")]),
    dict(out="pet-table", anchor="pettable", decoder=decode_sheet, unit="dòng", sheet=True,
         entries=[("fx", "Pet/PetTable")]),
    dict(out="pet-random-table", anchor="petrandomtable", decoder=decode_sheet, unit="dòng", sheet=True,
         entries=[("fx", "Pet/PetRandomTable")]),
    dict(out="speaker-table", anchor="speakertable", decoder=decode_sheet, unit="dòng", sheet=True,
         entries=[("fx", "ObjectTable/SpeakerTable")]),
    dict(out="xml-init", anchor="xmlinit", decoder=decode_init, unit="phần tử",
         entries=[("fx", "XML/init")]),
]


def decode_entry(table, stem, entry, raw):
    payload = entry_payload(raw, entry)
    issues = {}
    if table.get("sheet"):
        body, rows = table["decoder"](payload, entry, issues)
        encoding = "utf-8"
    else:
        text, encoding = decode_text(payload)
        root, counted = scan(text)
        issues.update(sorted(counted.items()))
        body, rows = table["decoder"](root, entry, issues)
    head = dict(pack=stem, entry=entry, bytes=len(raw), sha256=hashlib.sha256(raw).hexdigest(),
                encoding=encoding, issues=issues, rows=rows)
    return dict(head, **body)


def select_entries(table, packs):
    if "prefix" in table:
        stem, prefix = table["prefix"]
        return [(stem, name) for name, *_ in packs[stem].order if name.startswith(prefix)
                and packs[stem].read(name)[4:5] == b"<"]
    return table["entries"]


def compact(value):
    return json.dumps(value, ensure_ascii=False, separators=(",", ":"))


def dump_output(table, entries):
    head = dict(schema=SCHEMA, table=table["out"], unit=table["unit"])
    lines = ",\n".join(compact(e) for e in entries)
    return "{" + compact(head)[1:-1] + ',"entries":[\n' + lines + "\n]}\n"


# --- tham chiếu chéo ----------------------------------------------------------
def object_table_names(data):
    count, off, names = struct.unpack_from("<I", data)[0], 4, []
    for _ in range(count):
        length = struct.unpack_from("<I", data, off)[0]
        names.append(data[off + 4:off + 4 + length].decode("cp949", "replace"))
        off += 8 + length
    return names


def map_cells(buf, info):
    off = 80 + 8 * info["spawns"]
    rects = struct.unpack_from("<i", buf, off)[0]
    off += 4 + 16 * rects
    cell = 2 if info["version"] == 1000 else 3
    values = struct.unpack_from(f"<{info['width'] * info['height'] * cell}i", buf, off)
    spawns = [list(struct.unpack_from("<2i", buf, 80 + 8 * i)) for i in range(info["spawns"])]
    layer = lambda k: {i for i in range(info["width"] * info["height"]) if values[i * cell + k] != -1}  # noqa: E731
    return spawns, layer(0), layer(1)


def resolve_counts(names, *universes):
    pool = set()
    for universe in universes:
        pool |= {n.lower() for n in universe}
    names = sorted(set(names))
    missing = [n for n in names if n.lower() not in pool]
    return dict(distinct=len(names), resolved=len(names) - len(missing), missing=missing)


def crossrefs(packs, outputs):
    """Đối chiếu tên/ID giữa bảng đã giải và dữ liệu pack khác; kết quả là bằng chứng kr_static."""
    fx, fx_map, contents = packs["fx"], packs["FxMap"], packs["FxContents"]
    by = {t["out"]: outputs[t["out"]] for t in TABLES}
    objects = set()
    for stem, entry in (("fx", "ObjectTable/ObjectTable"), ("CharTE", "ObjectTable/CharTEObjectTable"),
                        ("EmoTE", "ObjectTable/EmoTEObjectTable"), ("FxAsset", "ObjectTable/FxAssetObjectTable")):
        if stem in packs and entry in packs[stem].entries:
            objects |= set(object_table_names(packs[stem].read(entry)))
    entries = {n for p in packs.values() for n, *_ in p.order}
    tile_info = set(re.findall(r'<Tile Name="([^"]+)"', fx.read("TileInfo/TileInfoList").decode("cp949", "replace")))
    strings = set()
    if "FxDesktop" in packs:
        strings = set(re.findall(r"<key>([^<]*)</key>", packs["FxDesktop"].read("ClientData/ClientString")
                                 .decode("cp949", "replace")))
    item_names = set()
    for item in parse_item_attr(contents.read("Attr/ItemAttr"))["items"]:
        item_names |= item_keys(item["name"]) | {item["sprite"]}
    themes = {n for n in entries if n.startswith("Theme/")}
    out = {}

    # MonAction: map tham chiếu qua thuộc tính monAction="MapScriptTemplate/<tên>".
    templates = {e["entry"] for e in by["mon-action"]}
    lower = {t.lower(): t for t in templates}
    refs, maps_with_ref = collections.Counter(), set()
    skill_tables, skill_refs = collections.Counter(), collections.Counter()
    for name, *_ in fx_map.order:
        if not name.startswith("Map/"):
            continue
        text = fx_map.read(name).decode("cp949", "replace")
        found = re.findall(r'monAction\s*=\s*"([^"]+)"', text)
        if found:
            maps_with_ref.add(name)
        refs.update(found)
        tables = sorted({t.lower() for t in re.findall(r'skilltablename\s*=\s*"([^"]+)"', text, re.I)})
        skill_tables.update(tables)
        for skill, level in re.findall(r'SkillName\s*=\s*"([^"]*)"\s+SkillLevel\s*=\s*"?(\d+)', text, re.I):
            skill_refs[(tuple(tables), skill, int(level))] += 1
    exact = [r for r in refs if r in templates]
    folded = [r for r in refs if r not in templates and r.lower() in lower]
    used = {lower[r.lower()] for r in refs if r.lower() in lower}
    out["mon_action_map_refs"] = dict(
        maps=len(maps_with_ref), references=sum(refs.values()), distinct=len(refs), exact=len(exact),
        case_insensitive_only=sorted(folded), missing=sorted(r for r in refs if r.lower() not in lower),
        templates=len(templates), templates_referenced=len(used), unreferenced=sorted(templates - used))

    # SkillTable: map khai skilltablename và item SkillName/SkillLevel.
    skills = {}
    for entry in by["skill-tables"]:
        key = f"{entry['entry'].split('/')[-2]}/{entry['entry'].split('/')[-1]}".lower()
        skills[key] = {s["name"]: {lvl for p in s["properties"] for lvl, _ in p["stats"]} for s in entry["skills"]}
    rows = []
    for (tables, skill, level), count in sorted(skill_refs.items()):
        hit = [t for t in tables if skill in skills.get(t, {})]
        rows.append(dict(tables=list(tables), skill=skill, level=level, items=count, resolved=bool(hit),
                         level_present=any(level in skills[t][skill] for t in hit)))
    out["skill_table_map_refs"] = dict(skilltablename=dict(sorted(skill_tables.items())), item_refs=rows)

    # CustomMap: kích thước, điểm xuất phát và vị trí ô so với map gốc cùng tên trong FxMap.
    index_list = by["custom-tile-index-list"][0]
    known_tiles = set(index_list["tiles"])
    compare, layer_names = [], set()
    for entry in by["custom-maps"]:
        for layer in entry["layers"].values():
            layer_names |= set(layer["palette"])
        base = entry["entry"].split("CustomMap_", 1)[1]
        if f"Map/{base}" not in fx_map.entries:
            continue
        buf = fx_map.read(f"Map/{base}")
        info = parse_map(buf)
        spawns, ground, objs = map_cells(buf, info)
        width = entry["width"]
        starts = [[p % width, p // width] for p in entry["start_points"]]
        pos = lambda k: {c[0] for c in entry["layers"].get(k, {}).get("cells", [])}  # noqa: E731
        compare.append(dict(map=base, size_equal=[info["width"], info["height"]] == [entry["width"], entry["height"]],
                            map_spawns=len(spawns), start_points=len(starts),
                            spawns_are_prefix=starts[:len(spawns)] == spawns,
                            ground_positions_equal=pos("GroundTiles") == ground,
                            object_positions_equal=pos("ObjectTiles") == objs))
    theme_names = [th["Name"] for th in by["custom-tile-list"][0].get("themes", [])]
    local_themes = [v for e in by["custom-maps"] for v in e.get("localdata", {}).values()]
    local_exact = sum(1 for v in set(local_themes) if v in theme_names)
    team_equal = sum(1 for e in by["custom-maps"] if e["team_count"] * e["team_user_count"] == len(e["start_points"]))
    in_range = sum(1 for e in by["custom-maps"]
                   if all(0 <= c[0] < e["width"] * e["height"] for layer in e["layers"].values() for c in layer["cells"])
                   and all(0 <= p < e["width"] * e["height"] for p in e["start_points"])
                   and all(len({c[0] for c in layer["cells"]}) == len(layer["cells"]) for layer in e["layers"].values()))
    templates_info = by["custom-tile-list"][0].get("TemplateMapInfo", {}).get("rows", [])
    sizes = {e["entry"].split("/", 1)[1]: f"{e['width']}x{e['height']}" for e in by["custom-maps"]}
    out["custom_map"] = dict(
        map_compare=compare, maps=len(by["custom-maps"]),
        team_count_x_user_count_equals_start_points=team_equal,
        positions_in_range_and_unique=in_range,
        localdata_theme_names=dict(resolve_counts(local_themes, theme_names), exact_case=local_exact),
        template_map_info=dict(rows=len(templates_info),
                               name_and_size_equal=sum(1 for r in templates_info if sizes.get(r.get("Name")) == r.get("SizeInfo")),
                               theme_resolved=resolve_counts([r["IDFName"] for r in templates_info], themes)),
        layer_names_in_index_list=resolve_counts(layer_names, known_tiles),
        ban_gadgets_in_gadget_table=resolve_counts(
            [g for e in by["custom-maps"] for g in e.get("ban", {}).get("Gadget", {})],
            [g["name"] for g in by["gadget-table"][0]["gadgets"]]))

    tile_list = by["custom-tile-list"][0]
    theme_tiles = [t["Name"] for th in tile_list.get("themes", []) for t in th["tiles"]["rows"]]
    out["custom_tiles"] = dict(
        index_list_in_tile_info=resolve_counts(index_list["tiles"], tile_info),
        index_items_in_item_attr=resolve_counts(index_list.get("items", []), item_names),
        theme_tiles_in_index_list=resolve_counts(theme_tiles, known_tiles),
        theme_idf_in_fxmap=resolve_counts([th["IDFName"] for th in tile_list.get("themes", [])], themes),
        special_blocks_resolved=resolve_counts([r["Name"] for r in tile_list.get("SpecialBlock", {}).get("rows", [])],
                                               tile_info, objects, entries),
        monsters_resolved=resolve_counts([r["Name"] for r in tile_list.get("MonsterInfo", {}).get("rows", [])],
                                         tile_info, objects, entries),
        items_in_item_attr=resolve_counts([r["Name"] for r in tile_list.get("ItemInfo", {}).get("rows", [])], item_names))

    say = collections.Counter()
    for key in ("stage-clear-event-npc", "mon-action"):
        for entry in by[key]:
            for body in entry.get("script_objects", [entry]):
                for lst in body["action_lists"]:
                    for _, attrs in lst["steps"]:
                        for k, v in attrs.items():
                            if re.fullmatch(r"say\d+", k, re.I):
                                say[(key, v)] += 1
    images = [o["init_info"].get("image") for e in by["stage-clear-event-npc"] for o in e["script_objects"]]
    out["script_strings"] = {
        key: resolve_counts([v for (k, v) in say if k == key], strings) for key in ("stage-clear-event-npc", "mon-action")}
    out["script_strings"]["stage_clear_images"] = resolve_counts(images, objects, entries)

    def sheet_column(key, column):
        sheet = by[key][0]["sheets"][0]
        index = [c["name"] for c in sheet["columns"]].index(column)
        return [r[index] for r in sheet["rows"] if isinstance(r[index], str) and r[index]]
    out["sheets"] = dict(
        pet_template_in_objects=resolve_counts(sheet_column("pet-table", "TemplateName"), objects, entries),
        speaker_mark_icon=resolve_counts(sheet_column("speaker-table", "MarkIcon"), objects, entries),
        speaker_both_side_img=resolve_counts(sheet_column("speaker-table", "BothSideImg"), objects, entries),
        speaker_templates=resolve_counts(
            sheet_column("speaker-table", "BothSideTemplate") + sum(
                (sheet_column("speaker-table", f"StickerTemplate{i}") for i in (1, 2, 3)), []), objects, entries))
    sheet = by["pet-random-table"][0]["sheets"][0]
    names = [c["name"] for c in sheet["columns"]]
    sums = collections.Counter()
    for row in sheet["rows"]:
        sums[row[names.index("RandomType")]] += row[names.index("Rate")]
    out["sheets"]["pet_random_rate_sum_by_type"] = {str(k): v for k, v in sorted(sums.items())}
    return out


def script_stats(entries):
    """Đếm loại trigger, tên/func của bước và các cách viết hoa thường của cùng một tên."""
    triggers, steps, names, funcs = (collections.Counter() for _ in range(4))
    spellings = collections.defaultdict(collections.Counter)
    for entry in entries:
        for body in entry.get("script_objects", [entry]):
            for table in body["trigger_tables"]:
                for time in table["times"]:
                    for trigger in time["triggers"]:
                        for key in trigger:
                            spellings["Trigger@" + key.lower()][key] += 1
                        kind = next((v for k, v in trigger.items() if k.lower() == "type"), "")
                        triggers[kind.lower()] += 1
            for lst in body["action_lists"]:
                for tag, attrs in lst["steps"]:
                    spellings[tag.lower()][tag] += 1
                    steps[tag.lower()] += 1
                    for key, value in attrs.items():
                        spellings["@" + key.lower()][key] += 1
                        if key.lower() == "name":
                            names[value.lower()] += 1
                        elif key.lower() == "func":
                            funcs[value.lower()] += 1
    order = lambda c: dict(sorted(c.items(), key=lambda kv: (-kv[1], kv[0])))  # noqa: E731
    return dict(triggers=order(triggers), step_tags=order(steps), step_names=order(names), step_funcs=order(funcs),
                mixed_case={k: dict(sorted(v.items())) for k, v in sorted(spellings.items()) if len(v) > 1})


# --- dựng, kiểm và tài liệu ---------------------------------------------------
# Pack có dữ liệu được đọc: bảng nguồn, map, Attr/ItemAttr, ClientString và các ObjectTable. Index của
# mọi pack đều được đọc (tên entry dùng để đối chiếu), nên manifest pin idx của mọi pack.
REQUIRED_PACKS = {stem for t in TABLES for stem, _ in t.get("entries", [t.get("prefix")])} | {"FxMap", "fx", "FxContents"}
OPTIONAL_PACKS = {"CharTE", "EmoTE", "FxAsset", "FxDesktop"}


def build(client):
    packs = {p.stem: Pack(client, p.stem) for p in sorted(client.glob("*.idx"))}
    missing = sorted(REQUIRED_PACKS - set(packs))
    if missing:
        raise SystemExit(f"client thiếu pack: {missing}")
    outputs, files = {}, {}
    for table in TABLES:
        decoded = [decode_entry(table, stem, entry, packs[stem].read(entry))
                   for stem, entry in select_entries(table, packs)]
        outputs[table["out"]] = decoded
        files[f"{table['out']}.json"] = dump_output(table, decoded)
    pins = {}
    for stem, pack in sorted(packs.items()):
        pins[stem] = dict(idx_bytes=pack.idx.stat().st_size, idx_sha256=sha256(pack.idx))
        if stem in REQUIRED_PACKS | OPTIONAL_PACKS:
            pins[stem].update(idd_bytes=pack.idd.stat().st_size, idd_sha256=sha256(pack.idd))
    exe = client / "CA.exe"
    manifest = dict(
        schema=SCHEMA, build="KR00967", generator="tools/kr_table_decode.py",
        client_exe_sha256=sha256(exe) if exe.is_file() else None,
        limits="JSON giải tĩnh từ pack; tên trường lấy nguyên từ file. Nghĩa gameplay ở tài liệu, không phải bằng chứng runtime hay wire.",
        packs=pins,
        outputs=[dict(path=name, table=t["out"], anchor=t["anchor"], unit=t["unit"],
                      **({"prefix": ":".join(t["prefix"])} if "prefix" in t else {}),
                      entries=[f"{e['pack']}:{e['entry']}" for e in outputs[t["out"]]],
                      source_bytes=sum(e["bytes"] for e in outputs[t["out"]]),
                      rows=sum(e["rows"] for e in outputs[t["out"]]),
                      issues=dict(sorted(sum((collections.Counter({k: (len(v) if isinstance(v, list) else v)
                                                                   for k, v in e["issues"].items()})
                                              for e in outputs[t["out"]]), collections.Counter()).items())),
                      bytes=len(files[name].encode()), sha256=hashlib.sha256(files[name].encode()).hexdigest())
                 for t in TABLES for name in [f"{t['out']}.json"]],
        crossrefs=crossrefs(packs, outputs),
        stats={key: script_stats(outputs[key]) for key in ("mon-action", "stage-clear-event-npc")})
    files[MANIFEST] = json.dumps(manifest, ensure_ascii=False, indent=1) + "\n"
    return files


def render_block(manifest):
    lines = [BEGIN, "",
             "Bảng sinh bởi `python3 tools/kr_table_decode.py --generate`; không sửa tay giữa hai dấu mốc.", "",
             "| Bảng | Entry nguồn | Byte nguồn | Số dòng giải được | Lệch chuẩn XML | JSON |",
             "|---|---|---:|---:|---|---|"]
    for output in manifest["outputs"]:
        sources = output["entries"]
        shown = ", ".join(f"`{s}`" for s in sources) if "prefix" not in output else (
            f"`{output['prefix']}*` ({len(sources)} entry)")
        issues = ", ".join(f"{k} {v}" for k, v in output["issues"].items()) or "không"
        lines.append(f"| [{output['table']}](#{output['anchor']}) | {shown} | {output['source_bytes']:,} | "
                     f"{output['rows']:,} {output['unit']} | {issues} | [{output['path']}](client-tables/{output['path']}) |")
    lines += ["", "Pack nguồn (SHA-256 đầy đủ trong [manifest](client-tables/manifest.json)):", ""]
    for stem, pin in manifest["packs"].items():
        idd = f", idd `{pin['idd_sha256'][:16]}…` ({pin['idd_bytes']:,} byte)" if "idd_sha256" in pin else ", chỉ đọc index"
        lines.append(f"- `{stem}`: idx `{pin['idx_sha256'][:16]}…`{idd}.")
    lines += ["", END]
    return "\n".join(lines)


def check(client=None):
    errors = []
    manifest_path = OUT_DIR / MANIFEST
    if not manifest_path.is_file():
        raise SystemExit(f"thiếu {manifest_path.relative_to(ROOT)}; chạy --generate")
    manifest = json.loads(manifest_path.read_text())
    expected = {f"{t['out']}.json" for t in TABLES} | {MANIFEST}
    present = {p.name for p in OUT_DIR.iterdir()}
    if present != expected:
        errors.append(f"client-tables: file lệch danh mục: thừa {sorted(present - expected)}, thiếu {sorted(expected - present)}")
    if manifest.get("schema") != SCHEMA:
        errors.append("manifest: schema lệch")
    for output in manifest["outputs"]:
        path = OUT_DIR / output["path"]
        if not path.is_file():
            continue
        data = path.read_bytes()
        if len(data) != output["bytes"] or hashlib.sha256(data).hexdigest() != output["sha256"]:
            errors.append(f"{output['path']}: size/SHA-256 lệch manifest")
            continue
        body = json.loads(data)
        if body.get("schema") != SCHEMA or [f"{e['pack']}:{e['entry']}" for e in body["entries"]] != output["entries"]:
            errors.append(f"{output['path']}: schema hoặc danh sách entry lệch manifest")
        if sum(e["rows"] for e in body["entries"]) != output["rows"]:
            errors.append(f"{output['path']}: số dòng lệch manifest")
    if CENSUS.is_file():
        census = json.loads(CENSUS.read_text())["packs"]
        for stem, pin in manifest["packs"].items():
            for key in ("idx_sha256", "idd_sha256"):
                if key in census.get(stem, {}) and census[stem][key] != pin[key]:
                    errors.append(f"manifest: {stem} {key} khác census.json (khác build?)")
    text = DOC.read_text() if DOC.is_file() else ""
    if BEGIN not in text or END not in text or text[text.index(BEGIN):text.index(END) + len(END)] != render_block(manifest):
        errors.append(f"{DOC.relative_to(ROOT)}: khối sinh tự động lệch; chạy --generate")
    if client:
        for name, content in build(client).items():
            if (OUT_DIR / name).read_text() != content:
                errors.append(f"{name}: khác bản giải lại từ client")
    if errors:
        raise SystemExit("\n".join(errors))
    rows = sum(o["rows"] for o in manifest["outputs"])
    scope = "giải lại từ client và so từng byte" if client else "không cần client; kiểm manifest/hash/tài liệu"
    print(f"Client tables OK: {len(manifest['outputs'])} bảng, {rows:,} dòng; {scope}.")


def write(files):
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    for path in OUT_DIR.iterdir():
        if path.name not in files:
            path.unlink()
    for name, content in files.items():
        (OUT_DIR / name).write_text(content)
    manifest = json.loads(files[MANIFEST])
    text = DOC.read_text() if DOC.is_file() else f"# Bảng dữ liệu client\n\n{BEGIN}\n{END}\n"
    start, end = text.index(BEGIN), text.index(END) + len(END)
    DOC.write_text(text[:start] + render_block(manifest) + text[end:])


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--client", type=Path, help="thư mục client KR 00967 gốc (chỉ đọc)")
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--generate", action="store_true")
    mode.add_argument("--check", action="store_true")
    args = parser.parse_args()
    if args.generate:
        if not args.client:
            parser.error("--generate cần --client")
        write(build(args.client))
        print(f"wrote {OUT_DIR.relative_to(ROOT)}/ and {DOC.relative_to(ROOT)}")
    else:
        check(args.client)


if __name__ == "__main__":
    main()
