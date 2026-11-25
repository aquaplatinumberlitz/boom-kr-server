"""Đọc pack KR 00967 và tạo cache dữ liệu phục vụ. Chỉ dùng thư viện chuẩn."""
import argparse
from contextlib import contextmanager, ExitStack
import hashlib
import os
import json
from pathlib import Path
import re
import stat
import struct
import tempfile
import xml.etree.ElementTree as ET

from content_census import decrypt_idx, decrypt_entry, parse_idx, parse_item_attr, parse_bomber_attr, parse_map_group, text
from export_map_profile import FIELDS, decode_map, load_tile_table

# SHA-256 của toàn bộ các cặp pack gốc, theo census và đối chứng CP090.
PACK_SHA256 = {
    'FxMap.idx': '30fd16b63d13a4a474daeeae611cbd2329077fdc6d3c5846cd8d2ed02d490b9b',
    'FxMap.idd': '9ce6da28ec24901ee489bdffd024eaf93314e1194fe735bb97e1d2bdd2dd7222',
    'FxContents.idx': '56ded9cd49079a528abed3b9d9fa7182a1ddb2b63648a835c22134850af52886',
    'FxContents.idd': '1ea0b3eb8b96381cda23f41e9714f5b1ec32bbc3a36b6c5ca138a2075ce661dd',
    'fx.idx': 'bcf921cc1be51f1cbda081fa0077ea02cdb94a591c714253a3230b4447dc1ae5',
    'fx.idd': '4465e5250bb72c0b4bdebb5e65b769c9ec44153676b76cc91415561eddc02767',
}
G1 = ('Forest07', 'Patrit14', 'Village10', 'Camp08', 'Tomb12', 'Factory07')
# CP382: hai người, floor trống và tắt terrain cho phép thử. Không tự mở rộng họ map.
F1 = ('Camp01', 'Cemetery01', 'Cookie01', 'Desert00', 'Factory07', 'Forest02', 'Ice00',
      'Island01', 'Kfg02', 'Mine00', 'Ocean00', 'Patrit01', 'Tomb01', 'Village00', 'Xmas01')
MAPS = tuple(dict.fromkeys((*G1, *F1, 'ExVillage01')))
ITEMS = {0: 'Item/ItemBomb', 1: 'Item/ItemFire', 2: 'Item/ItemRoller', 3: 'Item/ItemShoe',
         4: 'Item/ItemPunch', 5: 'Item/ItemMaxPower', 145: 'Item/ItemNiddle'}
# G1 đã đo 582/124/165 cho Kim cố định, giới hạn một Kim mỗi catalog.
# Luồng lộ Kim từ thùng chưa được đo; chỉ các kind 0–5 vào bảng drop F1.
F1_HIDDEN_ITEMS = {name: kind for kind, name in ITEMS.items() if kind != 145}
F1_FIXED_ITEMS = {name: kind for kind, name in ITEMS.items()}
TERRAIN_SEEDS = dict(zip((*G1, 'ExVillage01'), (
    '6760c3bf47d75cda0273d3ba476c8d0e9a972f582c731606f8c2f4466c76555f',
    '5cda90d6e0b21a910352cddfc55c1c655ab60f8c167f044e967d92cf725e3dc9',
    'b531fb3c1b876a805c75cd329c4dc130a3b8f0b6609ff1635f6384606048dda2',
    '1d3274a4c5e6cb72c553351680a0592d2e1134c614f71d2efcb846382c92496e',
    '0e7152a3a10dee15dc6c9de100f87abe00ea6de77b22a90200934eb34d33419b',
    'e077fddcf2624eb50643b8c9e473627d2bf5d6fce765881cf409cc3387e79135',
    'c96dff8c4d148a229bef97ba8c3a279f1a1cac3516281de1689385353b972144',
)))
SCHEMA = 'kr00967-export-v2'
F1E_ADAPTER = 'f1e-adapter-r1'
_AQUA_MARKERS = frozenset(('NXCL_Aqua01', 'Aqua01', 'Aqua02', 'Aqua05', 'Aqua06', 'Aqua08'))
_SOUL_MAPS = frozenset(('EventForest00', 'EventPatrit00', 'ExBula01', 'ExBula02'))
# Tile15_Ground/Remain*: B9D8E0 ignores byte +61 and B9DEA0 word +62 < 10000 feeds
# only EA5A10 ground drawing and mode-2 CBulaComponent; ED2350 cancels only >= 10000.
_REMAIN_GROUND_D15 = {
    'Tile15_Ground/Remain00': 7405312,
    'Tile15_Ground/Remain01': 7536384,
    'Tile15_Ground/Remain02': 7536384,
    'Tile15_Ground/Remain03': 7012096,
    'Tile15_Ground/Remain04': 6946560,
    'Tile15_Ground/Remain05': 7601920,
    'Tile15_Ground/Remain06': 7470848,
    'Tile15_Ground/Remain07': 7470848,
    'Tile15_Ground/Remain08': 7077632,
    'Tile15_Ground/Remain09': 7077632,
    'Tile15_Ground/Remain_Green01': 65280,
    'Tile15_Ground/Remain_Green02': 65280,
}
# KR data[8] for Tile*_Box exploding blocks (+33 == 1 -> 1-cell cross blast on break).
_EXPLODING_BOX_FLAGS = {
    'Tile53_Box/Block30': 256,
    'Tile53_Box/Block31': 65792,
    'Tile53_Box/Block32': 65792,
    'Tile30_Box/JellyBombBlock00': 65792,
}
_RETRO_POPUPS = {
    f'Retro{family}N{n:02}': f'Retro/Retro{popup}Start'
    for family, popup in (('Camp', 'Camp'), ('Factory', 'Factory'), ('Patrit', 'Pirate'))
    for n in range(1, 7)
}
_LOVELY_ROOMS = frozenset(f'LovelyRoom{n:02}' for n in range(1, 11))
_BIG_BATTLES = frozenset(('BSBigCamp08', 'BSBigPatrit14', 'BSBigVillage10'))
_SPACE_MAPS = frozenset((
    'CaJJ_Space02', 'CaJJ_Space08', 'Fm_Space01', 'Fm_Space02', 'WL_Space08',
    *(f'Space{n:02}' for n in range(1, 9)),
))
_PANGLAND_MAPS = frozenset((
    'NXCL_PangLand15', 'PangLand00', 'PangLand02', 'PangLand03', 'PangLand04',
    'PangLand06', 'PangLand07', 'PangLand11', 'PangLand12', 'Pangland14',
    'PangLand15', 'PangLand16',
))
_AQUA_OBJECT_MAPS = frozenset(('Aqua03', 'Aqua04', 'Aqua07', 'Aqua09'))
_RETRO_SCRIPT_MAPS = {
    f'Retro{family}N{n:02}': (f'Retro/Retro{family}Start', tag)
    for family, tag in (('Forest', 'cloud'), ('Village', 'retroride'))
    for n in range(1, 7)
}
_RESCUE_ALIEN_MAPS = frozenset(f'Fm_RescueET{n:02}' for n in range(1, 4))
_RETRO_WRIGGLE_MAPS = frozenset(f'RetroWriggleN{n:02}' for n in range(1, 6))
_WRIGGLE_MAPS = _RETRO_WRIGGLE_MAPS | {
    *(f'CaJJ_Wriggle{n:02}' for n in range(1, 8)), 'Fm_Wriggle02', 'Fm_Wriggle04',
}
_COLORING_MAP = 'Fm_Coloring03'
_F1E_FOUR_SPAWN_MAPS = _RESCUE_ALIEN_MAPS | _WRIGGLE_MAPS | {_COLORING_MAP}
_F1E_NEW_SCRIPT_MAPS = _AQUA_OBJECT_MAPS | _RETRO_SCRIPT_MAPS.keys() | _F1E_FOUR_SPAWN_MAPS

# Khóa BomberAttr f0 đã cắt cho trận thường. Nhánh f1=3 (kể cả các
# biến thể trùng tên C15/C08/C26) và nhóm đối thủ còn thiếu luật.
F1_CHARACTERS = frozenset((*range(12), *range(120, 132), 150, 151, *range(3000, 3005)))


def encode(value):
    return (json.dumps(value, ensure_ascii=False, separators=(',', ':'), sort_keys=True) + '\n').encode()


def sha(data):
    return hashlib.sha256(data).hexdigest()


def file_sha(stream):
    stream.seek(0)
    digest = hashlib.file_digest(stream, 'sha256').hexdigest()
    stream.seek(0)
    return digest


class Pack:
    """Giải mã đúng các entry được chọn trên chính descriptor đã băm."""
    def __init__(self, index, data):
        self.order = parse_idx(decrypt_idx(index.read()))
        self.data = data
        self.entries = {name: (offset, size, key) for name, offset, size, key in self.order}
        if len(self.entries) != len(self.order):
            # Pack thật có entry tên lặp ở các nhóm tham chiếu. Chỉ chấp nhận
            # entry được xuất khi tên của nó xuất hiện đúng một lần.
            seen = set()
            for name, *_ in self.order:
                if name in seen:
                    self.entries.pop(name, None)
                seen.add(name)

    def read(self, name):
        offset, size, key = self.entries[name]
        if not 0 < size <= 8 * 1024 * 1024:
            raise ValueError(f'Entry vượt giới hạn: {name}')
        self.data.seek(offset)
        raw = self.data.read(size)
        if len(raw) != size:
            raise ValueError(f'Entry bị cắt: {name}')
        return decrypt_entry(raw, key) if key else raw


def terrain_for(profile, table):
    """Chiếu các lớp seed đã đo, không áp dụng cho map F1 chưa đo terrain."""
    name = profile['name']
    names, tiles = table
    kinds, hits, ray, cancel = [0] * 195, [0] * 195, [0] * 195, []

    def tile(code):
        if code == -1:
            return '', None
        if not 0 <= code < len(names):
            raise ValueError(f'ID tile ngoài bảng: {name}')
        key = names[code]
        return key, tiles[key]

    def footprint(index, data):
        row, col = divmod(index, 15)
        width, height = data[6:8]
        if not (1 <= width <= 3 and 1 <= height <= 2):
            raise ValueError(f'Footprint chưa biết: {name}')
        for r in range(max(0, row - height + 1), row + 1):
            for c in range(col, min(15, col + width)):
                yield r * 15 + c

    for index, (ground, obj, *overlay) in enumerate(profile['cells']):
        if any(code != -1 for code in overlay):
            raise ValueError(f'Overlay chưa được đo: {name}')
        _, ground_data = tile(ground)
        obj_name, obj_data = tile(obj)
        if (name == 'Camp08' and ground_data is not None and ground_data[0] == 0
                and struct.unpack('<h', struct.pack('<i', ground_data[15])[2:])[0] >= 10_000):
            cancel.append(list(divmod(index, 15)))
        if name == 'Tomb12' and ground_data is not None and ground_data[11]:
            for covered in footprint(index, ground_data):
                ray[covered] = 1
        if obj_data is None:
            continue
        if obj_data[0] == 1:
            for covered in footprint(index, obj_data):
                if kinds[covered] == 2:
                    raise ValueError(f'Footprint chồng thùng: {name}')
                kinds[covered] = 1
        elif obj_data[0] == 2:
            if kinds[index]:
                raise ValueError(f'Ô thùng chồng địa hình: {name}')
            kinds[index] = 2
            hits[index] = max(1, obj_data[6]) if name == 'Camp08' else 1
        elif obj_data[0] not in (0, 3):
            raise ValueError(f'Lớp tile chưa biết: {obj_name}')
    seed = sha(bytes(kinds))
    if seed != TERRAIN_SEEDS[name]:
        raise ValueError(f'Terrain {name} không khớp seed đã đo: {seed}')
    terrain = dict(width=15, height=13, kinds=kinds, map_sha256=profile['payload_sha256'],
                   seed_sha256=seed, maximum_power=9)
    if name == 'Camp08':
        terrain['box_hits'] = hits
    if name == 'Tomb12':
        terrain['ray_barriers'] = ray
    result = dict(terrain=terrain)
    if cancel:
        result['bomb_cancellation'] = dict(map_sha256=profile['payload_sha256'], cells=cancel)
    return result


# Projection of the KR initializer (EA1EE0/9B0640) and the two
# dimension getters (7E7350/7E72B0). G1 profiles keep their pinned rules.
_GROUND = (0, 0, 0, 0, 0, 0)
_PASS_ALL = 0x01010101
_TYPE1_PIN_FLAGS = {
    'Tile16_Static/PinPillar01': 16843009,
    'Tile16_Static/PinPillar02': 256,
    'Tile16_Static/PinPillar03': 65536,
    'Tile16_Static/PinPillar04': 1,
    'Tile16_Static/PinPillar05': 16777216,
    'Tile62_Static/PinPillar': 16843009,
    'Tile62_Static/PinPillar_down': 256,
    'Tile62_Static/PinPillar_left': 65536,
    'Tile62_Static/PinPillar_right': 1,
    'Tile62_Static/PinPillar_up': 16777216,
}
# Ground offset 56 is copied by B9D8E0 and read only through B9E500, whose
# callers are bomber/NPC draw routines (alpha 128). No blocking or ray role.
_WATER_RENDER_GROUNDS = frozenset(('Tile07_Ground/Tile00', 'Tile07_Ground/Tile01'))
# Type-1 offset 32 sets map+1D48 via BB3FD0/EA7C90. Its only reader E9C490
# serves IUseAbleSpecialItem slot 10 (B4B5A0, the range-3 leap that tile
# names call "cantjump"); that special item is outside the F1 item scope.
_TYPE1_LEAP_BARRIERS = frozenset(('TileTransParent_Static/Static01', 'Tile19_Static/Block12'))
# Type 3 with offset 44 set enables the directional checks of E9F220/E9C520
# (E90A80). These pieces have identical uniform groups at offsets 24, 28, 32:
# all ones is clear, all zero cannot be entered or left by a ray (as 1D24).
_UNIFORM_PASSABLE_PIECES = frozenset(
    [f'Tile10_Passable/Building0{i}' for i in range(9)]
    + [f'Tile02_Passble/IceHouse0{i}' for i in range(9)])
_TENT_PIECES = frozenset((
    'Tile01_Passable/TentBlue', 'Tile01_Passable/TentOrange', 'Tile01_Passable/TentRed',
    'Tile16_Passble/ArmyTent01', 'Tile16_Passble/ArmyTent02',
))
_TENT_PORTS = 0x01000100
# Ground offset 52 is copied to CGOTileGround+0xD4 (B9D8E0) and read only via
# B9E4E0. Every live caller selects a riding speed (B46940: Ship +192 or +180)
# or draws a riding effect; no bomb, ray, flag-grid or placement consumer.
_RAIL_GROUNDS = frozenset(f'Tile08_Ground/Rail0{i}' for i in range(7))
# Ground offset 32 is a conveyor step in pixels, +36 the entry mask and +40 the
# exit direction (B9DB70/B9DBF0, KR direction order W/N/E/S). Only bomber
# movement, idle animation and AI read them; CGOBomb and the ray code do not.
_CONVEYOR_STEP = 2
# Word offset 62 below 10000 does not cancel bombs (ED2350). Values 1/2 are read
# only by ground drawing (EA5A10) and CBulaComponent (mode 2 in EEF020).
_PONG_GROUND_WORDS = {'Tile42_Ground/TileRed': 1, 'Tile42_Ground/TileBlue': 2}
# Static offset 36 is read only through BA1420: CBulaComponent (10001..10199),
# CBonusMapLadderWinnerComponent (999) and B50370, which has no reference.
_TYPE1_UNREAD_OFFSET36 = frozenset(('Tile42_Static/Block03',))
_TYPE1_BULA_OFFSET36 = {
    'Tile15_Static/Bf01_01': 10101,
    'Tile15_Static/Bf01_02': 10102,
    'Tile15_Static/Bf01_03': 10103,
    'Tile15_Static/Bf01_04': 10104,
    'Tile15_Static/Bf02_02': 10102,
    'Tile15_Static/Bf02_03': 10103,
    'Tile15_Static/Bf02_04': 10104,
    'Tile15_Static/Bf03_01': 10101,
    'Tile15_Static/Bf03_04': 10104,
    'Tile15_Static/Bf04_01': 10101,
    'Tile15_Static/Bf04_02': 10102,
    'Tile15_Static/Bf04_03': 10103,
    'Tile15_Static/Bf04_04': 10104,
    'Tile15_Static/Bf05_01': 10101,
    'Tile15_Static/Bf05_02': 10102,
    'Tile15_Static/Bf05_03': 10103,
    'Tile15_Static/Bf05_04': 10104,
}
_RETRO_CONVEYORS = {'Tile59_Ground/Conveyor00': (4, 2), 'Tile59_Ground/Conveyor01': (1, 0),
                   'Tile60_Ground/Conveyor02': (2, 1), 'Tile60_Ground/Conveyor03': (1, 0),
                   'Tile60_Ground/Conveyor04': (8, 3), 'Tile60_Ground/Conveyor05': (4, 2)}
_RETRO_DECORATORS = frozenset((
    'Tile60_Deco/Cloud00', 'Tile60_Deco/Cloud01', 'Tile60_Deco/Deco03',
    'Tile58_Deco/Cloud00', 'Tile58_Deco/Cloud01', 'Tile61_Deco/Static04_2', 'Tile61_Deco/Static05_2',
))
_F1E_LEAP_BARRIERS = frozenset((
    'Tile20_Static/CenterStatic00', 'Tile20_Static/CenterStatic01',
    'Tile20_Static/CenterStatic02', 'Tile20_Static/CenterStatic03',
    'Tile61_Static/GreenHouse01', 'Tile61_Static/GreenHouse02',
    'Tile64_Static/CenterStatic00', 'Tile64_Static/CenterStatic01', 'Tile64_Static/Static00',
))
_F1E_OPEN_PASSABLES = {
    'Tile61_Passable/GreenHouse03': (3, 0, 0, 0, 0, 0, *([_PASS_ALL] * 3), 0, 0, 1, 0, 0, 0, 0),
    'Tile61_Passable/GreenHouse04': (3, 0, 0, 0, 0, 0, *([_PASS_ALL] * 3), 0, 0, 1, 0, 0, 0, 0),
    'TileTransparent_Passable/Passable02': (3, 0, 0, 0, 0, 0, *([_PASS_ALL] * 4), 1, 1, 1, 0, 0, 0),
}
_F1E_COLORING_BOXES = {
    'Tile38_Box/LineBlock_D': 1541,
    'Tile38_Box/LineBlock_X': 263,
    'Tile38_Box/LineBlock_Y': 262,
    'Tile38_Box/NumBlock_09_X': 2308,
    'Tile38_Box/NumBlock_09_XY': 2306,
    'Tile38_Box/NumBlock_09_Y': 2307,
}
_RETRO_GROUND_102 = frozenset(f'Tile60_Ground/Tile{i:02}' for i in range(3, 12))
_RETRO_TENTS = frozenset(('Tile62_Passable/ArmyTent00', 'Tile62_Passable/ArmyTent01'))
_RETRO_BOMBPROOF_PORTS = {
    'Tile62_Passable/BombProof_Left01': (1, 0b0001),
    'Tile62_Passable/BombProof_Up01': (256, 0b0010),
    'Tile62_Passable/BombProof_Right01': (65536, 0b0100),
    'Tile62_Passable/BombProof_Down01': (16777216, 0b1000),
}


def f1_terrain_rules(profile, table, *, f1e=False):
    name = profile['name']
    if profile['rectangles']:
        raise ValueError(f'Vùng chữ nhật ngoài adapter F1-a: {name}')
    width, height = profile['width'], profile['height']
    size = width * height
    kinds, hits, exploding, ray, cancel = [0] * size, [0] * size, [0] * size, [0] * size, []
    ray_ports = [0b1111] * size
    names, tiles = table

    def fail(index, role, key):
        y, x = divmod(index, width)
        adapter = 'F1-e' if f1e else 'F1-a'
        raise ValueError(f'Ô ngoài adapter {adapter} của {name} ({x},{y}): {role} {key}')

    def lookup(code, index, role):
        if code == -1:
            return '−1', None
        if not 0 <= code < len(names) or names[code] not in tiles:
            fail(index, role, code)
        key = names[code]
        return key, tiles[key]

    def footprint(index, data, key):
        y, x = divmod(index, width)
        w, h = data[6:8]
        if not (1 <= w <= width and 1 <= h <= height):
            fail(index, 'footprint', key)
        for row in range(max(0, y - h + 1), y + 1):
            for col in range(x, min(width, x + w)):
                yield row * width + col

    def ground(index, key, d):
        # Empty ground uses the default CGOTileGround seen in Tomb12.
        if d is None:
            return
        factory_flag = (d[12:15] == (1, 0, 0) and
                        (key == 'Tile03_Ground/Tile07' or (f1e and key == 'Tile59_Ground/Tile05')))
        water_flag = key in _WATER_RENDER_GROUNDS and d[12:15] == (0, 0, 1)
        rail_flag = key in _RAIL_GROUNDS and d[12:15] == (0, 1, 0)
        conveyor = ((key.startswith('TileConv/') or
                     (f1e and _RETRO_CONVEYORS.get(key) == d[9:11])) and d[8] == _CONVEYOR_STEP
                    and d[9] in (1, 2, 4, 8) and d[10] in (0, 1, 2, 3)
                    and d[6:8] == (1, 1) and not any(d[11:15]))
        if (d[:6] != _GROUND or (d[8:11] != (0, 0, -1) and not conveyor)
                or (any(d[12:15]) and not (factory_flag or water_flag or rail_flag))
                or d[15] & 0xFF):
            # A nonzero byte at offset 60 selects CGOTileLap in EB48E0.
            fail(index, 'nền', key)
        if not (1 <= d[6] <= width and 1 <= d[7] <= height and d[11] in (0, 1)):
            fail(index, 'nền/footprint', key)
        if d[15] >> 16 and _PONG_GROUND_WORDS.get(key) != d[15] >> 16:
            # EB48E0's special ground for word 102 is gated by B64870
            # (constant false). ED2350 only cancels at word +62 >= 10000.
            retro = (f1e and d[:15] == (*_GROUND, 1, 1, 0, 0, -1, 0, 0, 0, 0)
                     and d[15] == 102 << 16
                     and key in _RETRO_GROUND_102)
            remain = (d[:15] == (*_GROUND, 1, 1, 0, 0, -1, 0, 0, 0, 0)
                      and _REMAIN_GROUND_D15.get(key) == d[15])
            if retro or remain:
                return
            if (key not in ('Tile16_Ground/PinGround01', 'Tile62_Ground/PinTile00',
                            'Tile62_Ground/PinTile01') or d[15] >> 16 != 10_000):
                fail(index, 'nền/cờ', key)
            cancel.append(list(divmod(index, width)))
        if d[11]:
            for cell in footprint(index, d, key):
                ray[cell] = 1

    spawn_cells = {y * width + x for x, y in profile['spawns']}
    for index, (base, obj, *overlay) in enumerate(profile['cells']):
        for code in overlay:
            if code == -1:
                continue
            if not f1e:
                fail(index, 'overlay', overlay)
            key, d = lookup(code, index, 'overlay')
            # EA1EE0 puts third-layer decorators in a separate render list.
            if key not in _RETRO_DECORATORS or d[:16] != (4,) + (0,) * 15:
                fail(index, 'overlay', key)
        ground(index, *lookup(base, index, 'nền'))
        key, d = lookup(obj, index, 'vật thể')
        if d is None:
            continue
        if d[0] == 0 and d[6:8] == (1, 1) and not d[11]:
            ground(index, key, d)
        elif d[:6] == (1, 0, 0, 0, 0, 0) and (
                d[8:11] == (0, 0, _TYPE1_PIN_FLAGS.get(key, 0))
                or ((key in _TYPE1_LEAP_BARRIERS or (f1e and key in _F1E_LEAP_BARRIERS))
                    and d[8:11] == (1, 0, 0) and not any(d[12:]))
                or (key in _TYPE1_UNREAD_OFFSET36 and d[8:11] == (0, 1, 0) and not any(d[11:]))
                or (d[6:9] == (1, 1, 0) and d[10] == 0 and _TYPE1_BULA_OFFSET36.get(key) == d[9])):
            # EA1EE0 writes the two flags for type 1 after BA1350, using
            # the dimensions and type regardless of the trailing dwords.
            for cell in footprint(index, d, key):
                if kinds[cell] == 2:
                    fail(cell, 'footprint chồng thùng', key)
                kinds[cell] = 1
        elif (d[:6] == (2, 0, 0, 0, 0, 0) and d[7] in (0, 1)
              and (d[8] == 0 or _EXPLODING_BOX_FLAGS.get(key) == d[8])
              and d[9:15] == (0,) * 6 and 0 <= d[6] <= 254):
            if kinds[index] != 0:
                fail(index, 'thùng chồng footprint', key)
            kinds[index], hits[index] = 2, max(1, d[6])
            if d[8]:
                exploding[index] = 1
        elif (f1e and key == 'Tile30_Box/Urchins00'
              and d[:16] == (2, 0, 0, 0, 0, 0, 255, 1, 16, 0, 0, 0, 0, 0, 0, 0)):
            # EA4332/EB61D1 -> ED23E0(tile, -1) -> EA4E40 sets bit 0x10; hit=255
            # never breaks from bomb blasts, so project as indestructible solid.
            if kinds[index] != 0:
                fail(index, 'thùng chồng footprint', key)
            kinds[index] = 1
        elif (f1e and key in _F1E_COLORING_BOXES
              and d[:16] == (2, 0, 0, 0, 0, 0, 0, 0, 0, _F1E_COLORING_BOXES[key], 0, 0, 0, 0, 0, 0)):
            if kinds[index] != 0:
                fail(index, 'thùng chồng footprint', key)
            kinds[index], hits[index] = 2, 1
        elif f1e and _F1E_OPEN_PASSABLES.get(key) == d[:16]:
            # All four ray ports stay open; no static occupied flag.
            pass
        elif (d[:6] == (3, 0, 0, 0, 0, 0) and d[6:11] == (_PASS_ALL,) * 3 + (0, 0)
              and d[11:13] == (0, 0)):
            # The Herb route confirms type 3 with the three all-ones groups:
            # the layer neither blocks the ray nor sets the occupied flag.
            pass
        elif (key in ('Tile53_Passable/Flower01', 'Tile53_Passable/Flower02')
              and d[:16] == (3, 0, 0, 0, 0, 0, _PASS_ALL, _PASS_ALL, _PASS_ALL,
                             0, 0, 1, 0, 0, 0, 0)):
            # 9B0640 checks offset 44 only for ground. These type-3 variants
            # have the same all-ones ray bytes as Herb at offsets 32–35.
            pass
        elif ((key.startswith('Tile16_Passble/Line') or key.startswith('Tile62_Passable/PoliceLine'))
              and d[:7] == (3, 0, 0, 0, 0, 0, 0)
              and d[7] in (65537, 16777472) and d[8:13] == (_PASS_ALL, 16777217, 1, 1, 0)):
            # The G1 shadow keeps these cells clear; native movement can still
            # be blocked. This is not a projection of directional movement.
            pass
        elif (key in ('Tile10_Passable/BasicBlock_pass00', 'Tile10_Passable/BasicBlock_pass01')
              and d[:6] == (3, 0, 0, 0, 0, 0)
              and d[6:11] == (_PASS_ALL, _PASS_ALL, _PASS_ALL, 0, 0)
              and d[11] == 0):
            # BasicBlock_pass01 has the same three passable groups as Line*;
            # its trailing render/cache words are opaque to EA1EE0 and are
            # deliberately excluded from the terrain projection.
            pass
        elif (key in _UNIFORM_PASSABLE_PIECES and d[:6] == (3, 0, 0, 0, 0, 0)
              and d[6] == d[7] == d[8] and d[6] in (0, _PASS_ALL)
              and d[9:13] == (0, 0, 1, 0) and d[13] not in (100, 101)):
            # EB4C10 still builds CGOTilePassable (offset 4 is 0, offset 52
            # is not 101; 6D29B0 tests 100). Offsets 56-68 are not read.
            if d[6] == 0:
                if index in spawn_cells:
                    fail(index, 'spawn trên ô type 3 bị chặn', key)
                ray[index] = 1
        elif ((key in _TENT_PIECES or (f1e and key in _RETRO_TENTS))
              and d[:13] == (3, 0, 0, 0, 0, 0, *([_TENT_PORTS] * 3), 0, 0, 1, 0)
              and d[13] not in (100, 101)):
            # BA0E30 reads byte +32 + direction; the byte 1 opens that side
            # for leaving and, via BA0EC0, entering. 17929B0/EB7190 give KR
            # direction order W/N/E/S, so bits 1 and 3 are north and south.
            # The value stays in KR bit order; see terrain.md#f1a-ida-round5.
            ray_ports[index] = 0b1010
        elif (f1e and key == 'Tile59_Passable/Shower03'
              and d[:16] == (3, 0, 0, 0, 0, 0, *([_TENT_PORTS] * 3), 16843008, 0, 1, 0, 101, 0, 1000)):
            # EB4C10's Tower branch for +52=101 is dead (B64870 returns
            # false). CGOTilePassable uses the same ray ports as Tent.
            ray_ports[index] = 0b1010
        elif (f1e and key in ('Tile30_Static/Coral00', 'Tile30_Static/Coral01')
              and d[:16] == (3, 0, 0, 0, 0, 0, 0, _PASS_ALL, _PASS_ALL, 1, 0, 1, 0, 0, 0, 0)):
            # EB4C10 -> E90A80/BA0E30: +24 == 0 blocks player movement while
            # +32 == 0x01010101 keeps all four bomb-ray sides open (0b1111).
            if index in spawn_cells:
                fail(index, 'spawn trên ô type 3 bị chặn', key)
        elif (f1e and key in _RETRO_BOMBPROOF_PORTS
              and d[:16] == (10, 0, 0, 0, 0, 0,
                             *([_RETRO_BOMBPROOF_PORTS[key][0]] * 3),
                             0, 0, 1, 0, 0, 0, 0)):
            # EB4C10 case 0xA -> 6D3C00 CGOTilePassableBroken inherits CGOTilePassable
            # (1747CEC); BA0E30 opens only the single non-zero byte side in +32..+35.
            if index in spawn_cells:
                fail(index, 'spawn trên ô type 10 bị chặn', key)
            ray_ports[index] = _RETRO_BOMBPROOF_PORTS[key][1]
        else:
            fail(index, 'vật thể/cờ', key)

    if any(kinds[y * width + x] != 0 for x, y in profile['spawns']):
        raise ValueError(f'Spawn không nằm trên ô trống: {name}')
    terrain = dict(width=width, height=height, kinds=kinds, maximum_power=9,
                   map_sha256=profile['payload_sha256'], seed_sha256=sha(bytes(kinds)))
    if any(ray):
        terrain['ray_barriers'] = ray
    if any(ports != 0b1111 for ports in ray_ports):
        terrain['ray_ports'] = ray_ports
    if any(hit > 1 for hit in hits):
        terrain['box_hits'] = hits
    if any(exploding):
        terrain['exploding_boxes'] = exploding
    rules = dict(terrain=terrain)
    if cancel:
        rules['bomb_cancellation'] = dict(map_sha256=profile['payload_sha256'], cells=cancel)
    return rules


def f1_floor(profile, terrain, *, f1e=False):
    """Chính sách nghiệm thu: placement chắc chắn và tối đa một drop có trọng số."""
    # 118FC30 -> 11ED860 -> 11EB960 applies placement_header as a default-weight
    # percent multiplier when per-map weights omit an item; fixed placements ignore it.
    if profile['placement_header'] not in (60, 70, 100) and profile['placements']:
        raise ValueError(f"Placement header chưa biết: {profile['name']}")
    kinds, width, height = terrain['kinds'], terrain['width'], terrain['height']
    # The existing 582 config and pickup consumers still have a 15×13
    # coordinate contract. Only the terrain and bomb ledger use the full grid.
    floor_width, floor_height = (min(width, 15), min(height, 13)) if f1e else (width, height)
    fixed, hidden, omitted, occupied = [], [], [], set()
    needle_placed = False
    occupied.update(map(tuple, profile['spawns']))
    for row in profile['placements']:
        name, x, y = row['name'], row['x'], row['y']
        if not (0 <= x < width and 0 <= y < height and 0 <= row['probability'] <= 100):
            raise ValueError(f"Placement ngoài giới hạn: {profile['name']}")
        kind = F1_FIXED_ITEMS.get(name)
        tile = kinds[y * width + x]
        if (kind is None or row['probability'] != 100 or tile != 0 or (x, y) in occupied
                or x >= floor_width or y >= floor_height
                or (kind == 145 and needle_placed)):
            omitted.append(row)
            continue
        fixed.append([y, x, kind])
        needle_placed |= kind == 145
        occupied.add((x, y))
    # This seed and density are design choices. Retain weighted pack row order
    # and duplicates; do not interpret excluded entries as another item.
    weights = [(F1_HIDDEN_ITEMS[name], weight) for name, weight in profile['drops']
               if name in F1_HIDDEN_ITEMS and weight > 0]
    boxes = [i for i, kind in enumerate(kinds)
             if kind == 2 and i % width < floor_width and i // width < floor_height]
    total = sum(weight for _, weight in weights)
    if boxes and total and not terrain.get('disable_bombs'):
        seed = bytes.fromhex(profile['payload_sha256'])
        y, x = divmod(boxes[int(sha(seed + b'cell'), 16) % len(boxes)], width)
        roll = int(sha(seed + b'item'), 16) % total
        for kind, weight in weights:
            if roll < weight:
                hidden.append([y, x, kind])
                break
            roll -= weight
    if len(fixed) + len(hidden) > 64:
        raise ValueError(f"Floor vượt giới hạn: {profile['name']}")
    return dict(fixed=fixed, hidden=hidden, omitted_placements=omitted)


def _f1e_popup(node, name):
    if node.tag != 'time' or node.attrib != {'ms': '0'} or len(node) != 1:
        return False
    popup = node[0]
    return (popup.tag == 'popup' and not len(popup) and popup.get('name') == name
            and popup.attrib.keys() == {'name', 'targetcx', 'targetcy', 'darkness'}
            and all(re.fullmatch(r'[0-9]+', popup.get(key)) for key in ('targetcx', 'targetcy', 'darkness')))


_F1E_OBJECT_CHILDREN = {
    'ScriptObjects': ('ScriptObject',),
    'ScriptObject': ('InitInfo', 'TriggerTable', 'ActionList'),
    'TriggerTable': ('Time',), 'Time': ('Trigger',), 'ActionList': ('Action', 'action'),
    'ItemTables': ('ItemTable',), 'ItemTable': ('Item',),
}
# These fields are inert. Exact values are checked only for the outer control
# nodes; nothing here is compiled into server-side actions.
_F1E_SCRIPT_FIELDS = {
    'InitInfo': ('centerbase collisionattr collisionbase collisionsize boss chatpos class diedropitemprob '
                 'diedropitemtable firepos fromstart gagebarpos image life lucci passtype pos type '
                 'noinfluencegameend hide item lockedattr avoidduration createappearsec getontheshipscore '
                 'getontheshipsec initappearcount killcnttobebadalien maxappearcount'),
    'Time': 'until', 'Trigger': 'actionlist type', 'ActionList': 'name next',
    'Action': 'class name pos00 func pos timeout time duration event userdist wait speed appeartime',
    'action': 'name say01 say02 selfdamage speed collisionbase collisionsize duration event func wait',
    'ItemTable': 'tablename index weight', 'Item': 'eateffect name weight',
    'cloud': 'cloudappearloop cloudappeartime cloudattacktime cloudmovetime',
    'numberofusers': 'count usercount', 'Respawn': 'col row', 'rideinfo': 'ridetype', 'Pos': 'col row',
    'Domain': 'gridrect index',
    'Match': ('comboruntime combostartkill combostarttime initialappearcount initialappeardelay '
              'sweepinggoldscore sweepingscore vs'),
    'Sweeping': 'combohaggardclass goldwriggleclass haggardclass wriggleclass',
}
_COLORING_FIELDS = frozenset(('groupIdx', 'regensecond', 'minboxcount', 'maxboxcount', 'combocount',
                              'comboIntervalTime', 'comboRunTime', 'scoreLevelGap', 'maxScoreLevel',
                              'timeLevelGap', 'maxTimeLevel'))
_F1E_OBJECT_LABELS = frozenset((
    '#샤크_부활', '#투명몬1', '#외계인', '#외계인꼬리', '#우주선', '#우주인',
    '#까칠이01', '#까칠이02 콤보용', '#꼬물이', '#황금꼬물이', '=' * 75, '=' * 96,
))


def _f1e_inert_tree(node, grammar):
    allowed = set(_F1E_SCRIPT_FIELDS.get(node.tag, '').split())
    if node.tag in ('Domain', 'InitInfo'):
        allowed.update(f'pos{i:02}' for i in range(18 if node.tag == 'InitInfo' else 12))
    fields = {key.lower() for key in node.attrib}
    if not fields <= allowed or len(fields) != len(node.attrib):
        return False
    for chunk in (node.text, *(child.tail for child in node)):
        if chunk and chunk.strip() and (node.tag != 'ScriptObjects' or any(
                line.strip() not in _F1E_OBJECT_LABELS for line in chunk.splitlines() if line.strip())):
            return False
    children = grammar.get(node.tag, ())
    return bool(len(node)) == bool(children) and all(
        child.tag in children and _f1e_inert_tree(child, grammar) for child in node)


def _f1e_coloring_xml(source):
    # Only quote decimal values of known fields in the one shipped leaf. Match
    # quoted strings as tokens so their contents never get rewritten.
    pattern = re.fullmatch(r'(<mapscript>\s*<ColoringPattern)(\s+[^<>]*?)(/\s*>\s*</mapscript>)', source)
    if pattern is None:
        return source
    fields = '|'.join(sorted(_COLORING_FIELDS))
    attrs = re.sub(rf'''"[^"]*"|'[^']*'|(\s+(?:{fields})\s*=\s*)([0-9]+)(?=\s|$)''',
                   lambda match: f'{match[1]}"{match[2]}"' if match[1] else match[0], pattern[2])
    return pattern[1] + attrs + pattern[3]


def _f1e_new_script(root, name):
    def no_text(node):
        return not (node.text or '').strip() and not (node.tail or '').strip()

    if not no_text(root) or any((child.tail or '').strip() for child in root):
        return False
    tags = tuple(child.tag for child in root)
    if name == _COLORING_MAP:
        return (tags == ('ColoringPattern',) and not len(root[0]) and no_text(root[0])
                and root[0].attrib.keys() == _COLORING_FIELDS
                and all(re.fullmatch(r'[0-9]+', value) for value in root[0].attrib.values()))
    if name in _AQUA_OBJECT_MAPS:
        return (tags == ('StageClearScript',)
                and tuple(child.tag for child in root[0]) == ('ItemTables', 'ScriptObjects')
                and _f1e_inert_tree(root[0], {**_F1E_OBJECT_CHILDREN,
                                             'StageClearScript': ('ItemTables', 'ScriptObjects')}))
    if name in _RETRO_SCRIPT_MAPS:
        popup, tag = _RETRO_SCRIPT_MAPS[name]
        return (tags == ('time', 'retroscript') and _f1e_popup(root[0], popup)
                and all(no_text(node) for node in root[0].iter())
                and _f1e_inert_tree(root[1], {
                    'retroscript': ('InitInfos',), 'InitInfos': (tag,),
                    tag: ('numberofusers', 'Respawn' if tag == 'cloud' else 'rideinfo'),
                }))

    option = 0
    if name in _RETRO_WRIGGLE_MAPS:
        if (tags[:1] != ('time',) or not _f1e_popup(root[0], 'RetroWriggle/Y2KWriggleStart')
                or not all(no_text(node) for node in root[0].iter())):
            return False
        option = 1
    if name in _RESCUE_ALIEN_MAPS:
        script = 'RescueAliensScript'
        children = ('InitInfos', 'ScriptObjects')
        grammar = {'InitInfos': ('InitInfo',), 'AppearPosList': ('Pos',), 'AppearActionList': ('Action',),
                   'ScriptObject': (*_F1E_OBJECT_CHILDREN['ScriptObject'], 'AppearPosList', 'AppearActionList')}
    else:
        script = 'WriggleScript'
        children = ('Domains', 'Matches', 'ItemTables', 'Sweeping', 'ScriptObjects')
        grammar = {'Domains': ('Domain',), 'Matches': ('Match',)}
    return (tags[option:] == ('MapScriptOption', script) and not len(root[option])
            and root[option].attrib == {'playTime': '90000'} and no_text(root[option])
            and tuple(child.tag for child in root[-1]) == children
            and _f1e_inert_tree(root[-1], {**_F1E_OBJECT_CHILDREN, **grammar, script: children}))


def f1e_bombs_disabled(raw, name, *, f1e=False):
    """Accept only the observed script shapes, without executing any script."""
    from kr_f1e_script_audit import map_tail_offset, parse_trailer

    def unsupported():
        return ValueError(f'Script ngoài adapter F1-e: {name}')

    trailer = parse_trailer(raw, map_tail_offset(raw))
    script = trailer['script']
    if trailer['groups'] or trailer['records']:
        raise unsupported()
    if name in _BIG_BATTLES:
        if script:
            raise unsupported()
        return False
    new_script = f1e and name in _F1E_NEW_SCRIPT_MAPS
    if not script or (not new_script and name not in (
            _AQUA_MARKERS | _LOVELY_ROOMS | _RETRO_POPUPS.keys() | _SPACE_MAPS | _PANGLAND_MAPS)):
        raise unsupported()
    source = script.rstrip(b'\0').decode('cp949')
    if new_script:
        # ElementTree drops declarations, processing instructions and unused
        # namespace attributes. Reject these before checking the live tree.
        xml = re.sub(r'<!--.*?-->', '', source, flags=re.S)
        if re.search(r'<[!?]|\sxmlns(?::[\w.-]+)?\s*=', xml):
            raise unsupported()
        if name == _COLORING_MAP:
            source = _f1e_coloring_xml(source.strip())
    if name in _LOVELY_ROOMS:
        # The shipped comment contains an internal "--" (invalid XML). Drop
        # only closed comments before requiring an exact live tree.
        source = re.sub(r'<!--.*?-->', '', source, flags=re.S)
    try:
        root = ET.fromstring(source.strip())
    except ET.ParseError as error:
        raise unsupported() from error
    if root.tag != 'mapscript' or root.attrib or not len(root):
        raise unsupported()
    if new_script:
        if not _f1e_new_script(root, name):
            raise unsupported()
        return False
    if name in _PANGLAND_MAPS:
        # Shipped PangLand06 contains a "##special attack" marker line beside <time>.
        for node in root.iter():
            for chunk in (node.text, node.tail):
                text_chunk = (chunk or '').strip()
                if text_chunk and not text_chunk.startswith('#'):
                    raise unsupported()
        if all(
            (ch.tag == 'ground' and not len(ch))
            or (ch.tag == 'limititemcount' and all(sub.tag == 'item' and not len(sub) for sub in ch))
            or (ch.tag == 'time' and all(sub.tag in {'bomb', 'ground', 'mp3', 'state', 'wave'} and not len(sub) for sub in ch))
            for ch in root
        ) and any(ch.tag == 'time' for ch in root):
            return False
        raise unsupported()
    if any((node.text or '').strip() or (node.tail or '').strip() for node in root.iter()):
        raise unsupported()
    if name in _SPACE_MAPS:
        if all(ch.tag == 'changetile' and all(sub.tag == 'position' and not len(sub) for sub in ch) for ch in root):
            return False
        raise unsupported()
    if len(root) != 1:
        raise unsupported()
    node = root[0]
    if name in _AQUA_MARKERS and node.tag == 'StageClearScript' and not node.attrib and not len(node):
        return False
    if name in _LOVELY_ROOMS and node.tag == 'mapscriptoption' and node.attrib == {'enablebomb': '0'} and not len(node):
        return True
    if name in _RETRO_POPUPS and _f1e_popup(node, _RETRO_POPUPS[name]):
        return False
    raise unsupported()


def _decode_soul_map(data):
    """Parse Soul map header/grid plus verified group/record trailer to exact EOF."""
    from kr_f1e_script_audit import map_tail_offset, parse_trailer

    tail_off = map_tail_offset(data)
    trailer = parse_trailer(data, tail_off)
    if trailer['script'] or not (trailer['groups'] or trailer['records']):
        raise ValueError('map parser không đọc hết payload')
    version, _, _, width, height, spawn_count = struct.unpack_from('<6i', data, 0)
    off = 80
    spawns = [list(struct.unpack_from('<2i', data, off + 8 * i)) for i in range(spawn_count)]
    off += 8 * spawn_count
    rect_count = struct.unpack_from('<i', data, off)[0]
    off += 4
    rects = [list(struct.unpack_from('<4i', data, off + 16 * i)) for i in range(rect_count)]
    off += 16 * rect_count
    stride = 2 if version == 1000 else 3
    cells = [list(struct.unpack_from(f'<{stride}i', data, off + i * stride * 4))
             for i in range(width * height)]
    off += width * height * stride * 4
    drop_count = struct.unpack_from('<i', data, off)[0]
    off += 4
    drops = []
    for _ in range(drop_count):
        item_name, off = text(data, off)
        drops.append([item_name, struct.unpack_from('<i', data, off)[0]])
        off += 4
    placement_header, count = struct.unpack_from('<2i', data, off)
    off += 8
    placements = []
    for _ in range(count):
        item_name, off = text(data, off)
        x, y, probability = struct.unpack_from('<3B', data, off)
        off += 3
        placements.append(dict(name=item_name, x=x, y=y, probability=probability))
    if off != tail_off:
        raise ValueError('map parser không khớp offset đuôi Soul')
    return dict(version=version, width=width, height=height, spawns=spawns,
                rectangles=rects, cell_fields=list(FIELDS[:stride]), cells=cells,
                drops=drops, placement_header=placement_header,
                placements=placements, scripted=False)


def read_map(levels, name, code, *, f1, f1e=False):
    raw = levels.read('Map/' + name)
    profile = _decode_soul_map(raw) if (f1 and not f1e and name in _SOUL_MAPS) else decode_map(raw)
    profile.update(name=name, map_code=code, payload_sha256=sha(raw))
    spawns = profile['spawns']
    width, height = (19, 17) if f1e and name in _BIG_BATTLES else (15, 13)
    if ((profile['width'], profile['height']) != (width, height)
            or (not f1e and profile['scripted'] and not (f1 and name == 'NXCL_Aqua01'))
            or not 2 <= len(spawns) <= 8
            or (f1e and len(spawns) != (4 if name in _F1E_FOUR_SPAWN_MAPS else 8))
            or any(not (0 <= x < width and 0 <= y < height) for x, y in spawns)
            or ((f1 or f1e) and len({tuple(pair) for pair in spawns}) != len(spawns))):
        raise ValueError(f'Map ngoài phạm vi phục vụ: {name}')
    if f1e:
        profile['adapter'] = F1E_ADAPTER
        if f1e_bombs_disabled(raw, name, f1e=True):
            profile['disable_bombs'] = True
    elif f1 and name == 'NXCL_Aqua01' and f1e_bombs_disabled(raw, name):
        raise ValueError(f'Map ngoài phạm vi phục vụ: {name}')
    return profile


def f1_rules(profile, table, *, f1e=False):
    rules = (terrain_for(profile, table) if profile['name'] in TERRAIN_SEEDS
             else f1_terrain_rules(profile, table, f1e=f1e))
    if f1e and profile.get('disable_bombs'):
        rules['terrain']['disable_bombs'] = True
    rules['floor'] = f1_floor(profile, rules['terrain'], f1e=f1e)
    return rules


def outputs(files, maps, measured, f1=False, *, f1e=False):
    def pack(name):
        return Pack(files[name + '.idx'], files[name + '.idd'])

    attr = pack('FxContents')
    all_items = parse_item_attr(attr.read('Attr/ItemAttr'))['items']
    items = [all_items[i] for i in ITEMS]
    if any(row['name'] != ITEMS[row['kind']] for row in items):
        raise ValueError('ItemAttr có ID khác KR 00967')
    characters = {}
    seen = set()
    for entry in parse_bomber_attr(attr.read('Attr/BomberAttr')):
        key, group, speed, max_speed, cap, max_cap, power, max_power, *_ = entry['values']
        if key in seen or not 0 <= key <= 0x1fff:
            raise ValueError('BomberAttr có key trùng hoặc nằm ngoài wire KR 00967')
        seen.add(key)
        if key in F1_CHARACTERS:
            if not (0 <= group <= 8 and group not in (3, 4, 6)
                    and 1 <= speed <= max_speed <= 10 and 1 <= cap <= max_cap <= 10
                    and 1 <= power <= max_power <= 10):
                raise ValueError('BomberAttr có nhóm hoặc chỉ số ngoài phạm vi trận thường')
            characters[key] = dict(speed=speed, maximum_speed=max_speed, capacity=cap,
                                   maximum_capacity=max_cap, power=power, maximum_power=max_power)
    if not {0, 1} <= characters.keys():
        raise ValueError('BomberAttr thiếu khóa nền 0 hoặc 1')
    groups = parse_map_group(attr.read('Attr/MapGroup'))
    yield 'tables.json', dict(items=[{k: row[k] for k in ('kind', 'name', 'weight')} for row in items],
                              characters=characters,
                              map_groups=[dict(row, maps=[m for m in row['maps'] if m.rsplit('/', 1)[-1] in maps])
                                          for row in groups])
    levels = pack('FxMap')
    order = [name for name, *_ in levels.order if name.startswith('Map/')]
    table = load_tile_table(pack('fx')) if measured or f1 or f1e else None
    for name in maps:
        entry = 'Map/' + name
        profile = read_map(levels, name, order.index(entry), f1=f1, f1e=f1e)
        profile['pack_items'] = {row['name']: row['kind'] for row in items}
        yield name + '.json', profile
        if f1 or f1e:
            yield name + '.rules.json', f1_rules(profile, table, f1e=f1e)
        elif measured and name in TERRAIN_SEEDS:
            yield name + '.rules.json', terrain_for(profile, table)


def expected_members(maps, measured, f1=False):
    return {'tables.json', *(name + '.json' for name in maps),
            *(name + '.rules.json' for name in maps if f1 or (measured and name in TERRAIN_SEEDS))}


def checked_cache(path, identity, pins, maps, measured, f1=False):
    if path.is_symlink() or not path.is_dir():
        raise ValueError('Thiếu cache hợp lệ; hãy chạy export')
    manifest_path = path / 'manifest.json'
    if (manifest_path.is_symlink() or not manifest_path.is_file()
            or manifest_path.stat().st_size > 65_536):
        raise ValueError('Thiếu manifest cache; hãy chạy export')
    manifest = json.loads(manifest_path.read_bytes())
    expected = expected_members(maps, measured, f1)
    if (not isinstance(manifest, dict) or manifest.get('identity') != identity
            or manifest.get('schema') != SCHEMA or manifest.get('packs') != pins
            or not isinstance(manifest.get('files'), dict) or set(manifest['files']) != expected
            or {p.name for p in path.iterdir()} != expected | {'manifest.json'}):
        raise ValueError('Manifest cache không khớp bản xuất')
    for name in expected:
        member = path / name
        if member.is_symlink() or not member.is_file() or member.stat().st_size > 65_536:
            raise ValueError(f'File cache không hợp lệ: {name}')
        if sha(member.read_bytes()) != manifest['files'][name]:
            raise ValueError(f'File cache sai hash: {name}; hãy bỏ cache lỗi rồi export lại')
    return manifest


def cache_identity(pins, maps, measured, f1=False, *, f1e=False):
    key = dict(schema=SCHEMA, packs=pins, maps=maps, measured=measured)
    if f1e:
        key['adapter'] = F1E_ADAPTER
    elif f1:
        key['adapter'] = 'f1-a-terrain-r5'
    return sha(encode(key))


@contextmanager
def verified_packs(game, pins=PACK_SHA256, pack_files=None):
    with ExitStack() as stack:
        files = {}
        for name, digest in pins.items():
            path = game / name if pack_files is None else pack_files[name]
            if not path.is_file() or path.is_symlink():
                raise ValueError(f'Thiếu pack KR 00967: {name}')
            stream = stack.enter_context(path.open('rb'))
            if not stat.S_ISREG(os.fstat(stream.fileno()).st_mode) or file_sha(stream) != digest:
                raise ValueError(f'Pack không khớp KR 00967 (SHA-256): {name}')
            files[name] = stream
        yield files


def export(game, cache, *, pins=PACK_SHA256, maps=MAPS, measured=True, f1=False, f1e=False, pack_files=None):
    """Chỉ dùng pins khác qua API kiểm thử; catalog thử có giới hạn."""
    if f1 and f1e:
        raise ValueError('Chỉ chọn một adapter F1-a hoặc F1-e')
    if (f1 or f1e) and (not 1 <= len(maps) <= 16 or len(set(maps)) != len(maps)
               or any(not 1 <= len(name) <= 64 or not name.isascii()
                      or not all(c.isalnum() or c == '_' for c in name) for name in maps)):
        raise ValueError(f'{"F1-e" if f1e else "F1-a"} cần 1–16 tên map KR khác nhau')
    game = game.resolve(strict=True)
    if cache.is_symlink():
        raise ValueError('Cache không được là symlink')
    cache = cache.resolve()
    if cache.is_relative_to(game):
        raise ValueError('Cache phải nằm ngoài thư mục game chỉ đọc')
    identity = cache_identity(pins, maps, measured, f1, f1e=f1e)
    dest = cache / identity
    with verified_packs(game, pins, pack_files) as files:
        if dest.exists() or dest.is_symlink():
            checked_cache(dest, identity, pins, maps, measured, f1 or f1e)
            return dest, True
        cache.mkdir(mode=0o700, parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(prefix='.export-', dir=cache) as work:
            temp = Path(work) / 'data'
            temp.mkdir(mode=0o700)
            hashes = {}
            for name, value in outputs(files, maps, measured, f1, f1e=f1e):
                data = encode(value)
                (temp / name).write_bytes(data)
                hashes[name] = sha(data)
            if any(file_sha(stream) != pins[name] for name, stream in files.items()):
                raise ValueError('Pack đã đổi trong lúc xuất')
            (temp / 'manifest.json').write_bytes(encode(dict(schema=SCHEMA, identity=identity,
                                                             packs=pins, files=hashes)))
            checked_cache(temp, identity, pins, maps, measured, f1 or f1e)
            try:
                temp.rename(dest)
            except OSError:
                if not dest.is_dir():
                    raise
                checked_cache(dest, identity, pins, maps, measured, f1 or f1e)
                return dest, True
    return dest, False


def prebuilt(path, *, pins=None, maps=None, measured=True):
    """Kiểm bản JSON xuất sẵn đi kèm repo/gói, không cần thư mục game.

    Bản này là đúng thư mục export mặc định (G1, F1 và ExVillage01), được phép
    commit theo docs/evidence/owner-directives/pack-data-rights.json. Hằng số
    được đọc lúc gọi để gói thử với pack giả vẫn kiểm theo pin của chính nó.
    """
    pins = PACK_SHA256 if pins is None else pins
    maps = MAPS if maps is None else tuple(maps)
    path = Path(path)
    if path.is_symlink():
        raise ValueError('Dữ liệu JSON đóng sẵn không được là symlink')
    checked_cache(path, cache_identity(pins, maps, measured), pins, maps, measured)
    return path.resolve(strict=True)


def refresh_prebuilt(game, out):
    """Xuất lại dữ liệu JSON đóng sẵn từ client KR chỉ đọc (dev/build)."""
    out = Path(out)
    if out.is_symlink() or (out.exists() and not out.is_dir()):
        raise ValueError('Đích dữ liệu JSON đóng sẵn phải là thư mục thường')
    out.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='.pack-data-', dir=out.parent) as work:
        cache, _ = export(game, Path(work) / 'cache')
        if out.exists():
            for child in out.iterdir():
                if child.is_symlink() or not child.is_file() or child.suffix != '.json':
                    raise ValueError(f'Thư mục dữ liệu chỉ chứa file JSON phẳng: {child.name}')
            for child in out.iterdir():
                child.unlink()
        else:
            out.mkdir()
        for source in sorted(cache.iterdir()):
            (out / source.name).write_bytes(source.read_bytes())
    return prebuilt(out)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--game-dir', type=Path, help='Thư mục game KR 00967, chỉ đọc')
    target = parser.add_mutually_exclusive_group(required=True)
    target.add_argument('--cache', type=Path, help='Cache theo identity của bản xuất')
    target.add_argument('--prebuilt-out', type=Path,
                        help='Ghi đè thư mục dữ liệu JSON đóng sẵn, ví dụ server/pack-data')
    target.add_argument('--check-prebuilt', type=Path, help='Chỉ kiểm dữ liệu JSON đóng sẵn')
    adapter = parser.add_mutually_exclusive_group()
    adapter.add_argument('--f1-map', action='append', help='Tên map F1-a chính xác; lặp lại tối đa 16 lần')
    adapter.add_argument('--f1e-map', action='append', help='Tên map F1-e chính xác; lặp lại tối đa 16 lần')
    args = parser.parse_args()
    if (args.check_prebuilt is None) != (args.game_dir is not None):
        parser.error('--game-dir bắt buộc khi xuất và không dùng với --check-prebuilt')
    if (args.f1_map or args.f1e_map) and args.cache is None:
        parser.error('--f1-map và --f1e-map chỉ dùng với --cache')
    try:
        if args.check_prebuilt is not None:
            path, reused = prebuilt(args.check_prebuilt), True
        elif args.prebuilt_out is not None:
            path, reused = refresh_prebuilt(args.game_dir, args.prebuilt_out), False
        else:
            path, reused = export(args.game_dir, args.cache, maps=args.f1_map or args.f1e_map or MAPS,
                                  f1=bool(args.f1_map), f1e=bool(args.f1e_map))
    except (OSError, ValueError, KeyError, IndexError, struct.error, ET.ParseError) as exc:
        parser.exit(1, f'Không thể xuất pack KR 00967: {exc}\n')
    print(json.dumps(dict(cache=str(path), reused=reused)))


if __name__ == '__main__':
    main()
