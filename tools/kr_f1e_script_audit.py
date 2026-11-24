"""Trích đặc tả tĩnh mapscript F1-e và đuôi nhị phân Soul từ pack KR 00967.

Chỉ đọc pack đã kiểm SHA-256. Kết quả là bản tóm tắt cấu trúc `kr_static`; nó không
chứng minh quyền xử lý, tick hay đồng bộ của server gốc. Bố cục đuôi theo
docs/protocol/terrain.md#f1a-ida-round4 (`EA1EE0 → vf+0x64 → 115C770`).
"""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import re
import struct
import sys
import xml.etree.ElementTree as ET

SCHEMA = 'kre-f1e-script-specs-v1'
PIN_NAMES = ('FxMap.idx', 'FxMap.idd', 'fx.idx', 'fx.idd')
SOUL_MAPS = ('EventForest00', 'EventPatrit00', 'ExBula01', 'ExBula02')
F1A_SCRIPT_MAPS = ('NXCL_Aqua01',)


def _numbered(prefix, numbers, width=2):
    return tuple(f'{prefix}{n:0{width}d}' for n in numbers)


# 95 map lớp C F1-e theo docs/content/f1-full-mechanic-clusters.md (không gồm 4 map AIBattle F3).
FAMILY_MAPS = {
    'aqua': _numbered('Aqua', range(1, 10)),
    'space': ('CaJJ_Space02', 'CaJJ_Space08', 'Fm_Space01', 'Fm_Space02',
              *_numbered('Space', range(1, 9)), 'WL_Space08'),
    'pangland': ('NXCL_PangLand15', *_numbered('PangLand', (0, 2, 3, 4, 6, 7, 11, 12, 15, 16)),
                 'Pangland14'),
    'wriggle': (*_numbered('CaJJ_Wriggle', range(1, 8)), 'Fm_Wriggle02', 'Fm_Wriggle04',
                *_numbered('RetroWriggleN', range(1, 6))),
    'retro_popup': tuple(m for family in ('Camp', 'Factory', 'Patrit')
                         for m in _numbered(f'Retro{family}N', range(1, 7))),
    'retro_forest': _numbered('RetroForestN', range(1, 7)),
    'retro_village': _numbered('RetroVillageN', range(1, 7)),
    'lovelyroom': _numbered('LovelyRoom', range(1, 11)),
    'rescue_et': _numbered('Fm_RescueET', range(1, 4)),
    'coloring': ('Fm_Coloring03',),
    'bigbattle': ('BSBigCamp08', 'BSBigPatrit14', 'BSBigVillage10'),
}
F1E_MAPS = tuple(name for names in FAMILY_MAPS.values() for name in names)

# Giới hạn an toàn của parser; dữ liệu thật nằm xa dưới các trần này.
MAX_GROUPS, MAX_POINTS, MAX_RECORDS, MAX_NAME = 256, 4096, 256, 64

LIMITS = [
    'Mọi số đếm, offset, hash và trường là cấu trúc tĩnh của pack (kr_static/confirmed); '
    'ý nghĩa nhóm điểm Soul và ba i32 cuối mỗi bản ghi vẫn unknown.',
    'Quyền server, tick, thứ tự thực thi, đồng bộ hai view, điểm và phần thưởng của mọi họ script là unknown.',
    'Parser chịu lỗi chỉ dùng để tóm tắt; không khẳng định client KR đọc Coloring hay comment theo cùng cách.',
]


class Element:
    __slots__ = ('tag', 'attrs', 'children')

    def __init__(self, tag, attrs):
        self.tag, self.attrs, self.children = tag, attrs, []

    def iter(self, *tags):
        wanted = {t.lower() for t in tags}
        for child in self.children:
            if not wanted or child.tag.lower() in wanted:
                yield child
            yield from child.iter(*tags)

    def get(self, name, default=None):
        """Tra attr không phân biệt hoa thường (pack trộn `type`/`Type`)."""
        return self.attrs.get(name.lower(), default)


COMMENT = re.compile(r'<!--.*?(?:-->|\Z)', re.S)
TAG = re.compile(r'<\s*(/?)\s*([A-Za-z_][\w.:-]*)((?:[^<>"\']|"[^"]*"|\'[^\']*\')*?)(/?)\s*>', re.S)
ATTR = re.compile(r'''\s+([A-Za-z_][\w.:-]*)\s*=\s*("[^"]*"|'[^']*'|[+-]?[0-9]+(?=\s|$))''')


def parse_script(text):
    """Tokenizer chịu lỗi: bỏ comment, nhận attr không nháy, đóng thẻ theo tên không phân biệt hoa thường."""
    comments = COMMENT.findall(text)
    root, stack = Element('#root', {}), []
    stack.append(root)
    stats = Counter(comments=len(comments))
    for match in TAG.finditer(COMMENT.sub('', text)):
        close, tag, body, self_closing = match.groups()
        if close:
            for depth in range(len(stack) - 1, 0, -1):
                if stack[depth].tag.lower() == tag.lower():
                    stats['mismatched'] += depth != len(stack) - 1
                    del stack[depth:]
                    break
            else:
                stats['mismatched'] += 1
            continue
        attrs = {}
        pos = 0
        for attribute in ATTR.finditer(body):
            key, value = attribute.groups()
            if body[pos:attribute.start()].strip() or key.lower() in attrs:
                raise ValueError(f'Attr sai hoặc trùng trong thẻ {tag}')
            pos = attribute.end()
            if value[0] in '"\'':
                value = value[1:-1]
            else:
                stats['unquoted_attrs'] += 1
            attrs[key.lower()] = value
        if body[pos:].strip():
            raise ValueError(f'Attr sai trong thẻ {tag}')
        element = Element(tag, attrs)
        stack[-1].children.append(element)
        if not self_closing:
            stack.append(element)
    stats['unclosed'] = len(stack) - 1
    return root, comments, {key: stats[key] for key in ('comments', 'unquoted_attrs', 'mismatched', 'unclosed')}


def num(value):
    if value is not None and re.fullmatch(r'[+-]?[0-9]+', value.strip()):
        return int(value)
    return value


def counted(values):
    """Đếm theo khóa chuỗi; attr vắng mặt ghi `(missing)` thay vì bị bỏ."""
    return dict(sorted(Counter('(missing)' if v is None else str(v) for v in values).items()))


def distinct(values):
    return sorted(set(values), key=lambda v: (isinstance(v, str), v))


def schedule(values):
    """Mốc ms: số mốc khác nhau, đầu/cuối và các bước khác nhau giữa hai mốc liền kề."""
    points = sorted(set(values))
    if not points:
        return None
    return dict(distinct=len(points), first=points[0], last=points[-1], timestamps=points,
                steps=distinct(b - a for a, b in zip(points, points[1:])))


# --- nhị phân ------------------------------------------------------------
def _i32(data, off):
    if not 0 <= off <= len(data) - 4:
        raise ValueError(f'Thiếu dword tại {off}')
    return struct.unpack_from('<i', data, off)[0], off + 4


def _string(data, off, limit=MAX_NAME, encoding='latin1'):
    length, off = _i32(data, off)
    if not 1 <= length <= limit or off + length > len(data):
        raise ValueError(f'Độ dài chuỗi sai tại {off - 4}: {length}')
    return data[off:off + length].rstrip(b'\0').decode(encoding), off + length


def map_tail_offset(data):
    """Đi qua header, spawn, rect, lưới, drop và placement; trả offset đuôi."""
    version, _, _, width, height = struct.unpack_from('<5i', data, 0)
    spawn_count = struct.unpack_from('<i', data, 20)[0]
    if version not in (1000, 1001, 1003) or not (0 < width <= 512 and 0 < height <= 512
                                                  and 0 <= spawn_count <= 64):
        raise ValueError(f'Header map sai: {version} {width}x{height} {spawn_count}')
    off = 80 + 8 * spawn_count
    rects, off = _i32(data, off)
    if not 0 <= rects <= 64:
        raise ValueError(f'Số rect sai: {rects}')
    off += 16 * rects
    stride = 2 if version == 1000 else 3
    off += 4 * width * height * stride
    drops, off = _i32(data, off)
    if not 0 <= drops <= 512:
        raise ValueError(f'Số drop sai: {drops}')
    for _ in range(drops):
        _, off = _string(data, off, 95)
        off += 4
    _, off = _i32(data, off)
    placements, off = _i32(data, off)
    if not 0 <= placements <= 4096:
        raise ValueError(f'Số placement sai: {placements}')
    for _ in range(placements):
        _, off = _string(data, off, 95)
        off += 3
    if off > len(data):
        raise ValueError('Placement vượt payload')
    return off


def parse_trailer(data, off):
    """Đuôi sau placement: nhóm điểm, bản ghi tên + u8 x/y + 3 i32, rồi u32 độ dài script.

    Yêu cầu khít tuyệt đối: script phải kết thúc đúng ở cuối payload.
    """
    start = off
    count, off = _i32(data, off)
    if not 0 <= count <= MAX_GROUPS:
        raise ValueError(f'Số nhóm sai tại {start}: {count}')
    groups = []
    for _ in range(count):
        points, off = _i32(data, off)
        if not 0 <= points <= MAX_POINTS or off + 8 * points > len(data):
            raise ValueError(f'Số điểm sai tại {off - 4}: {points}')
        groups.append([list(struct.unpack_from('<2i', data, off + 8 * i)) for i in range(points)])
        off += 8 * points
    count, off = _i32(data, off)
    if not 0 <= count <= MAX_RECORDS:
        raise ValueError(f'Số bản ghi sai tại {off - 4}: {count}')
    records = []
    for _ in range(count):
        name, off = _string(data, off, encoding='cp949')
        if not name or '\0' in name:
            raise ValueError('Tên bản ghi Soul rỗng hoặc có NUL bên trong')
        if off + 14 > len(data):
            raise ValueError(f'Bản ghi bị cắt tại {off}')
        records.append([name, *struct.unpack_from('<2B3i', data, off)])
        off += 14
    if off + 4 > len(data):
        raise ValueError(f'Thiếu độ dài script tại {off}')
    length = struct.unpack_from('<I', data, off)[0]
    off += 4
    if off + length != len(data):
        raise ValueError(f'Độ dài script {length} tại {off - 4} không khít payload {len(data)}')
    return dict(offset=start, groups=groups, records=records, script_offset=off - 4,
                script=data[off:off + length])


# --- tóm tắt theo họ -----------------------------------------------------
def family_of(name):
    for family, names in FAMILY_MAPS.items():
        if name in names:
            return family
    if name in F1A_SCRIPT_MAPS:
        return 'aqua'
    raise KeyError(f'Map ngoài danh sách F1-e: {name}')


def _triggers(root):
    return counted(t.get('type') for t in root.iter('Trigger'))


def _objects(root):
    objects = list(root.iter('ScriptObject'))
    infos = [o for s in objects for o in s.children if o.tag.lower() == 'initinfo']
    return dict(count=len(objects), types=counted(i.get('type') for i in infos),
                images=counted(i.get('image') for i in infos),
                table_count=sum(1 for _ in root.iter('TriggerTable')),
                tables=[
                    dict(info=_attrs(i, ('type', 'class', 'pos', 'image')), trigger_tables=[
                        [_attrs(t) | {'triggers': [_attrs(x) for x in t.iter('Trigger')]}
                         for t in table.iter('Time')]
                        for table in s.iter('TriggerTable')
                    ])
                    for s in objects for i in s.children if i.tag.lower() == 'initinfo'
                ])


def _attrs(element, keys=None):
    return {k: num(v) for k, v in element.attrs.items() if keys is None or k in keys}


def _actions(root):
    actions = list(root.iter('Action'))
    return dict(names=counted(a.get('name') for a in actions),
                funcs=counted(a.get('func') for a in actions if a.get('func') is not None))


def _require(condition, family, what):
    if not condition:
        raise ValueError(f'Nội dung không khớp họ {family}: {what}')


def spec_aqua(root, family):
    stages = list(root.iter('StageClearScript'))
    _require(len(stages) == 1, family, 'cần đúng một StageClearScript')
    if not any(root.iter('ScriptObject')):
        _require(not stages[0].children and not stages[0].attrs, family, 'marker có nội dung')
        return dict(variant='stage_clear_marker_only', reader='CMapScriptStageClear')
    _require(any(stages[0].iter('TriggerTable')), family, 'thiếu TriggerTable')
    return dict(variant='stage_clear_objects', reader='CMapScriptStageClear', objects=_objects(root),
                triggers=_triggers(root), actions=_actions(root),
                until=distinct(num(t.get('until')) for t in root.iter('Time')),
                item_tables=len(list(root.iter('ItemTable'))))


def spec_space(root, family):
    tiles = list(root.iter('changetile'))
    _require(tiles, family, 'thiếu changetile')
    positions = [p for t in tiles for p in t.children if p.tag.lower() == 'position']
    return dict(changetile=len(tiles), positions=len(positions),
                full_lines=sum(1 for p in positions if -1 in (num(p.get('x')), num(p.get('y')))),
                ms=schedule(num(t.get('ms')) for t in tiles),
                dangertime=distinct(num(t.get('dangertime')) for t in tiles),
                tilecategory=distinct(t.get('tilecategory') for t in tiles),
                action_types=distinct(t.get('changepostfix') for t in tiles))


def pangland_event(block):
    counts = counted(node.tag.lower() for node in block.children if node.tag.lower() in ('wave', 'bomb', 'state'))
    sprinkler = sum(node.tag.lower() == 'wave' and node.get('name') == 'wave/SprinklerBomb'
                    for node in block.children)
    if sprinkler:
        counts['sprinkler_bomb'] = sprinkler
    return _attrs(block, ('ms', 'type')) | counts


def spec_pangland(root, family):
    top = next(root.iter('mapscript'), root)
    times = [t for t in top.children if t.tag.lower() == 'time']
    waves = counted(w.get('name') for w in root.iter('wave'))
    _require('wave/SprinklerBomb' in waves, family, 'thiếu wave/SprinklerBomb')
    bombs = list(root.iter('bomb'))
    limit = next(root.iter('limititemcount'), None)
    return dict(time_nodes=len(times), ms=schedule(num(t.get('ms')) for t in times),
                events=[pangland_event(t) for t in times],
                time_types=counted(t.get('type') for t in times if t.get('type')),
                children=counted(c.tag for t in times for c in t.children),
                waves=waves,
                bombs=dict(nodes=len(bombs),
                           total_explicit_count=sum(int(b.get('count')) for b in bombs if b.get('count') is not None),
                           missing_count=sum(b.get('count') is None for b in bombs),
                           power=distinct(num(b.get('power')) for b in bombs),
                           state=counted(num(b.get('state')) for b in bombs),
                           targeted=sum(1 for b in bombs if b.get('targetcol') is not None)),
                tile_states=counted(num(s.get('state')) for s in root.iter('state')),
                limit_items=None if limit is None else dict(
                    default=num(limit.get('default')),
                    items={i.get('name'): num(i.get('count')) for i in limit.iter('item')}))


def spec_wriggle(root, family):
    script = next(root.iter('WriggleScript'), None)
    _require(script is not None, family, 'thiếu WriggleScript')
    sweeping = next(script.iter('Sweeping'), None)
    return dict(play_time=distinct(num(o.get('playTime')) for o in root.iter('MapScriptOption')),
                objects=_objects(script), triggers=_triggers(script), actions=_actions(script),
                domains=len(list(script.iter('Domain'))),
                domain_rows=[_attrs(d) for d in script.iter('Domain')],
                matches=[{k: num(v) for k, v in sorted(m.attrs.items())} for m in script.iter('Match')],
                sweeping=None if sweeping is None else dict(sorted(sweeping.attrs.items())),
                items=distinct(i.get('name') for i in script.iter('Item')),
                popup=distinct(p.get('name') for p in root.iter('popup')))


def _popups(root):
    return [dict(ms=num(t.get('ms')), popup=p.get('name')) for t in root.iter('time')
            for p in t.children if p.tag.lower() == 'popup']


def spec_retro_popup(root, family):
    popups = _popups(root)
    _require(popups and not any(root.iter('retroscript')), family, 'popup-only')
    return dict(reader='CMapScriptPopUp', popups=popups)


def spec_retro_forest(root, family):
    clouds = list(root.iter('cloud'))
    _require(clouds, family, 'thiếu cloud')
    return dict(reader='CMapScriptPopUp', popups=_popups(root),
                cloud=[{k: num(v) for k, v in sorted(c.attrs.items())} for c in clouds],
                respawns=len(list(root.iter('Respawn'))),
                respawn_positions=[_attrs(r) for r in root.iter('Respawn')],
                user_counts=[_attrs(u) for u in root.iter('numberofusers')])


def spec_retro_village(root, family):
    rides = list(root.iter('rideinfo'))
    _require(rides, family, 'thiếu rideinfo')
    return dict(reader='CMapScriptPopUp', popups=_popups(root),
                ride_types=[num(r.get('ridetype')) for r in rides],
                user_counts=[_attrs(u) for u in root.iter('numberofusers')])


def spec_lovelyroom(root, family, comments=()):
    options = list(root.iter('mapscriptoption'))
    _require(len(options) == 1 and options[0].get('enablebomb') == '0', family, 'thiếu enablebomb=0')
    commented = Counter()
    tables, events = [], []
    for comment in comments:
        inner, _, _ = parse_script(comment[4:].removesuffix('-->'))
        commented.update(e.tag for e in inner.iter())
        tables.extend(dict(attrs=_attrs(t), items=[_attrs(i) for i in t.iter('Item')])
                      for t in inner.iter('ItemTable', 'limititemcount'))
        events.extend(dict(ms=num(t.get('ms')), states=[_attrs(s) for s in t.iter('state')])
                      for t in inner.iter('time'))
    return dict(options=[{k: num(v) for k, v in o.attrs.items()} for o in options],
                live_tags=distinct(e.tag for e in root.iter()),
                commented_tags=dict(sorted(commented.items())), comment_only=dict(tables=tables, events=events))


def spec_rescue_et(root, family):
    script = next(root.iter('RescueAliensScript'), None)
    _require(script is not None, family, 'thiếu RescueAliensScript')
    init = next((i for i in script.children if i.tag.lower() == 'initinfos'), None)
    params = {} if init is None else {k: num(v) for i in init.children for k, v in i.attrs.items()}
    return dict(play_time=distinct(num(o.get('playTime')) for o in root.iter('MapScriptOption')),
                init=dict(sorted((k, v) for k, v in params.items() if not k.startswith('pos'))),
                init_positions=sum(1 for k in params if k.startswith('pos')),
                objects=_objects(script), triggers=_triggers(script), actions=_actions(script),
                appear_positions=len(list(script.iter('Pos'))),
                appear_actions=len([a for s in script.iter('AppearActionList') for a in s.children]),
                init_position_fields={k: v for k, v in params.items() if k.startswith('pos')},
                appearance_positions=[_attrs(r) for b in script.iter('AppearPosList') for r in b.iter('Pos')],
                appearance_events=[_attrs(r) for b in script.iter('AppearActionList') for r in b.iter('Action')])


def spec_coloring(root, family):
    patterns = list(root.iter('ColoringPattern'))
    _require(patterns, family, 'thiếu ColoringPattern')
    return dict(patterns=[{k: num(v) for k, v in sorted(p.attrs.items())} for p in patterns])


def spec_bigbattle(root, family):
    _require(not root.children, family, 'có script')
    return {}


SPECS = {name[5:]: fn for name, fn in globals().items() if name.startswith('spec_')}


def summarize_script(script, family):
    text = script.rstrip(b'\0').decode('cp949')
    root, comments, stats = parse_script(text)
    try:
        ET.fromstring(text.strip())
        strict = True
    except ET.ParseError:
        strict = False
    marker = len(root.children) == 1 and root.children[0].tag.lower() == 'mapscript'
    if script and (not marker or stats['mismatched'] or stats['unclosed']):
        raise ValueError('Blob có thẻ không khớp hoặc thiếu root mapscript')
    extra = dict(comments=comments) if family == 'lovelyroom' else {}
    spec = SPECS[family](root, family, **extra)
    summary = dict(bytes=len(script), marker=marker, xml_strict=strict and bool(text.strip()),
                   root=distinct(c.tag for e in root.children for c in e.children) if marker else [], **stats)
    if script:
        summary['sha256'] = hashlib.sha256(script).hexdigest()
    return summary, spec


def tile_summary(layers, names):
    out = {}
    for label, layer in zip(('ground', 'object', 'overlay'), layers):
        codes = sorted({c for c in layer if c != -1})
        if any(not 0 <= c < len(names) for c in codes):
            raise ValueError(f'ID tile ngoài ObjectTable ở lớp {label}')
        categories = Counter(names[c].split('/', 1)[0] for c in codes)
        out[label] = dict(variants=len(codes), codes=codes, categories=dict(sorted(categories.items())))
    return out


def audit_map(name, raw, code, names):
    from kr_pack_export import decode_map
    profile = decode_map(raw)
    off = map_tail_offset(raw)
    if raw[off:off + 8] != bytes(8):
        raise ValueError(f'Hai trường đầu sau placement phải rỗng: {name}')
    trailer = parse_trailer(raw, off)
    family = family_of(name)
    script, spec = summarize_script(trailer['script'], family)
    entry = dict(code=code, family=family, scope='F1-a' if name in F1A_SCRIPT_MAPS else 'F1-e',
                 payload_sha256=hashlib.sha256(raw).hexdigest(), bytes=len(raw),
                 version=profile['version'], size=[profile['width'], profile['height']],
                 spawns=len(profile['spawns']), trailer_offset=trailer['offset'],
                 script_length_offset=trailer['script_offset'],
                 soul_groups=len(trailer['groups']), soul_records=len(trailer['records']),
                 script=script, tiles=tile_summary(zip(*profile['cells']), names), spec=spec)
    return entry


def audit_soul(name, raw, code):
    off = map_tail_offset(raw)
    trailer = parse_trailer(raw, off)
    if trailer['script']:
        raise ValueError(f'Map Soul có script khác rỗng: {name}')
    groups, records = trailer['groups'], trailer['records']
    return dict(code=code, payload_sha256=hashlib.sha256(raw).hexdigest(), bytes=len(raw),
                trailer_offset=trailer['offset'], script_length_offset=trailer['script_offset'],
                script_length=0, end_aligned=True,
                group_count=len(groups), record_count=len(records),
                group_point_counts=[len(g) for g in groups], groups=groups,
                record_names=counted(r[0] for r in records),
                records=records,
                record_xy_is_group_first_point=len(groups) == len(records) and all(
                    g and g[0] == r[1:3] for g, r in zip(groups, records)))


# --- pack ----------------------------------------------------------------
def extract(pack_root):
    from kr_pack_export import PACK_SHA256, Pack, file_sha, load_tile_table, verified_packs
    pins = {name: PACK_SHA256[name] for name in PIN_NAMES}
    with verified_packs(Path(pack_root).resolve(strict=True), pins) as files:
        levels = Pack(files['FxMap.idx'], files['FxMap.idd'])
        names, _ = load_tile_table(Pack(files['fx.idx'], files['fx.idd']))
        order = [entry for entry, *_ in levels.order if entry.startswith('Map/')]
        codes = {entry: code for code, entry in enumerate(order)}
        if len(codes) != len(order) or len(set(F1E_MAPS)) != 95 or len(F1E_MAPS) != 95:
            raise ValueError('Map trùng hoặc phạm vi F1-e không đủ 95 map')

        def read(name):
            return levels.read('Map/' + name), codes['Map/' + name]

        maps = {name: audit_map(name, *read(name), names) for name in (*F1E_MAPS, *F1A_SCRIPT_MAPS)}
        soul = {name: audit_soul(name, *read(name)) for name in SOUL_MAPS}
        if any(file_sha(stream) != pins[name] for name, stream in files.items()):
            raise ValueError('Pack đã đổi trong lúc đọc')
    return build(pins, maps, soul)


def build(pins, maps, soul):
    f1e = [m for m in maps.values() if m['scope'] == 'F1-e']
    families = counted(m['family'] for m in f1e)
    return dict(
        schema=SCHEMA, task='p101-f1e-script-specs', build='KR 00967', source='kr_static',
        status='confirmed', semantics='unknown', server_authority='unknown', tick_and_sync='unknown',
        packs=dict(sorted(pins.items())),
        reproduction='python3 tools/kr_f1e_script_audit.py --pack-root <thư mục KR 00967> '
                     '--check docs/evidence/content/f1e-scripts-extracted.json',
        layout=dict(
            tail='i32 group_count; group_count × (i32 point_count, point_count × (i32 x, i32 y)); '
                 'i32 record_count; record_count × (u32 name_len, name, u8 x, u8 y, i32, i32, i32); '
                 'u32 script_len; script (cp949, mapscript)',
            client_reader='EA1EE0 → 118FC30 → vf+0x64 → 115C770 (docs/protocol/terrain.md#f1a-ida-round4)',
            note='96 map trong tập script có hai dword đầu bằng 0 sau placement; độ dài script khít tới EOF.'),
        counts=dict(f1e_maps=len(f1e), f1e_with_mapscript_marker=sum(m['script']['marker'] for m in f1e),
                    f1e_without_script=sum(not m['script']['bytes'] for m in f1e),
                    f1e_strict_xml_rejected=sorted(n for n, m in maps.items()
                                                   if m['script']['bytes'] and not m['script']['xml_strict']),
                    f1a_script_maps=len(maps) - len(f1e), soul_maps=len(soul), families=families),
        limits=LIMITS, soul=soul, maps=maps)


def dump(data):
    """JSON ổn định: mỗi map/Soul nằm trên một dòng để diff gọn."""
    def compact(value):
        return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(',', ':'))

    lines = []
    for key, value in sorted(data.items()):
        if key in ('maps', 'soul'):
            inner = ',\n'.join(f'  {compact(k)}: {compact(v)}' for k, v in sorted(value.items()))
            lines.append(f' {compact(key)}: {{\n{inner}\n }}')
        else:
            lines.append(f' {compact(key)}: {compact(value)}')
    return '{\n' + ',\n'.join(lines) + '\n}\n'


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument('--pack-root', type=Path, required=True, help='Thư mục client KR 00967, chỉ đọc')
    target = parser.add_mutually_exclusive_group(required=True)
    target.add_argument('--out', type=Path, help='Ghi JSON trích xuất')
    target.add_argument('--check', type=Path, help='Trích lại và so khớp từng byte với JSON đã commit')
    args = parser.parse_args(argv)
    try:
        pack_root = args.pack_root.resolve(strict=True)
        if args.out is not None:
            out = args.out.resolve()
            if (out.is_relative_to(pack_root) or out.suffix != '.json'
                    or (out.exists() and (not out.is_file() or out.stat().st_nlink != 1))):
                raise ValueError('--out phải là file .json thường bên ngoài pack')
        data = dump(extract(pack_root)).encode('utf-8')
        if args.check is not None:
            if args.check.read_bytes() != data:
                raise ValueError(f'{args.check}: khác bản trích lại từ pack')
            print(f'OK: {args.check} khớp pack')
        else:
            out.write_bytes(data)
            print(f'Đã ghi {args.out}')
    except (OSError, ValueError, KeyError, struct.error) as exc:
        parser.exit(1, f'Không thể trích F1-e: {exc}\n')


if __name__ == '__main__':
    sys.path.insert(0, str(Path(__file__).resolve().parent))
    main()
