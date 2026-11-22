"""Nối dữ liệu đã xuất với template wire có trong gói Linux."""
import json
import struct

from kr_pack_export import F1_FIXED_ITEMS, F1_HIDDEN_ITEMS, sha
from rust_map_profile import blob, raw_blob, table_row
from server import probe_codec as codec


def floor582(floor):
    lists = (floor['fixed'], floor['hidden'], [])
    if sum(map(len, lists)) > 64:
        raise ValueError('Floor vượt giới hạn')
    inner = bytearray()
    index = 0
    for group, allowed in zip(lists, (F1_FIXED_ITEMS, F1_HIDDEN_ITEMS, {})):
        inner.extend(struct.pack('<H', len(group)))
        for y, x, kind in group:
            if not (0 <= y < 13 and 0 <= x < 15 and kind in allowed.values()):
                raise ValueError('Floor F1 có ô hoặc item ngoài phạm vi')
            inner.extend(struct.pack('<IBBI', index, y, x, kind))
            index += 1
    if sum(kind == 145 for _, _, kind in floor['fixed']) > 1:
        raise ValueError('Floor F1 chỉ hỗ trợ một Kim cố định')
    return raw_blob(inner)[0]


def enable_floor_policies(room, kinds):
    if kinds & {3, 4}:
        room['item_policy'] = 'report_derived_shoe_punch'
    if 145 in kinds:
        room['life_actions'] = ('needle_rescue' if room.get('life_actions') in ('rescue', 'needle_rescue')
                                else 'needle')


def configure(config, bundle, cache, out, maps, scope, *, unlock_all=False):
    two = config['diagnostic_two_lobby']
    room = two['fixed_room']
    packets = room['packets']

    def load(pin):
        spec = pin['fixture']
        frame = (bundle / spec['path']).read_bytes()
        if len(frame) != spec['bytes'] or sha(frame) != spec['sha256']:
            raise ValueError('Mẫu wire trong gói không khớp pin')
        return frame, codec.frame_body(frame, pin['sequence'])

    def save(pin, body, stem):
        parent, _ = load(pin)
        frame = codec.encode(parent[:37], pin['opcode'], body, pin['sequence'])
        path = out / (stem + '.bin')
        path.write_bytes(frame)
        return dict(pin, fixture=dict(path=str(path), bytes=len(frame), sha256=sha(frame)))

    def start_parts(pin):
        _, body = load(pin)
        inner, end, _ = blob(body, 1)
        size = struct.unpack_from('<I', inner, 22)[0]
        row_at = 37 + size
        if not 1 <= size <= 64 or row_at > len(inner):
            raise ValueError('Tên map trong Start nguồn không hợp lệ')
        count = inner[row_at - 5]
        if not 2 <= count <= 8 or len(inner) != row_at + count * 26:
            raise ValueError('Roster mẫu Start không hợp lệ')
        return body, inner, end, size, count

    def original_name(pin):
        _, inner, _, size, _ = start_parts(pin)
        return inner[26:26 + size].decode('ascii')

    # Use the first metadata row; the other source rows may have different name lengths.
    _, table_body = load(packets['table'])
    table, end, _ = blob(table_body)
    if end != len(table_body) or not 1 <= int.from_bytes(table[:2], 'little') <= 16:
        raise ValueError('Bảng map mẫu không hợp lệ')
    at = 7
    for _ in range(2):
        for _ in range(2):
            size = struct.unpack_from('<I', table, at)[0]
            if size > 64:
                raise ValueError('Tên trong bảng map mẫu dài quá giới hạn')
            at += 4 + size
        at += 2
    original = b'\x01\x00' + table[2:at + 14]
    names, code_at = table_row(original)
    old_start = packets['start']
    body, inner, end, old_size, count = start_parts(old_start)
    if (names[1][2] != inner[26:26 + old_size]
            or original[code_at:code_at + 2] != inner[20:22]):
        raise ValueError('Bảng map và Start của mẫu wire không khớp')

    # Floor packets are server policy. Preserve a floor only for its original
    # map and G1 mode; do not copy the CP382 diagnostics into other maps.
    floors = {original_name(old_start): load(packets['floor'])[1]}
    for choice in room['map_choices']:
        floors[original_name(choice['packets']['start'])] = load(choice['packets']['floor'])[1]

    rows, choices, floor_kinds = [], [], set()
    for index, name in enumerate(maps):
        path = cache / (name + '.json')
        data = path.read_bytes()
        profile = json.loads(data)
        short = name.encode('ascii')
        code = profile['map_code']
        if profile['name'] != name:
            raise ValueError('Tên map không khớp cache')
        row = bytearray(original)
        row[2:6] = struct.pack('<I', index)
        row[code_at:code_at + 2] = struct.pack('<H', code)
        a, b, _ = names[1]
        row[a:b] = struct.pack('<I', len(short)) + short
        rows.append(row[2:])

        adjusted = bytearray(inner)
        row_at = 37 + old_size
        n = min(count, len(profile['spawns']))
        if any(adjusted[row_at + 26 * i] != i or adjusted[row_at + 26 * i + 2] != i
               for i in range(count)):
            raise ValueError('Mẫu Start cần slot và spawn ordinal liền nhau')
        adjusted[row_at - 5] = n
        del adjusted[row_at + 26 * n:]
        adjusted[26 + old_size:30 + old_size] = struct.pack('<I', index)
        adjusted[20:26 + old_size] = struct.pack('<HI', code, len(short)) + short
        start, _ = raw_blob(adjusted)
        _, play = load(packets['play'])
        value = codec.read_fields(play, [1, 4, 2])
        value[2] = code
        rules = cache / (name + '.rules.json')
        applied = json.loads(rules.read_bytes()) if scope in ('g1', 'f1-a') and rules.is_file() else {}
        floor = floors.get(name, raw_blob(bytes(6))[0]) if scope == 'g1' else raw_blob(bytes(6))[0]
        if scope == 'f1-a':
            plan = applied.pop('floor')
            floor = floor582(plan)
            floor_kinds.update(kind for group in ('fixed', 'hidden') for _, _, kind in plan[group])
        new = {}
        for key, payload in [('map', codec.pack_fields([0, code, index], [2, 2, 4])),
                             ('start', body[:1] + start + body[end:]),
                             ('play', codec.pack_fields(value, [1, 4, 2])), ('floor', floor)]:
            new[key] = save(packets[key], payload, f'map-{index}-{key}')
        choice = dict(map_profile=dict(path=str(path), bytes=len(data), sha256=sha(data)), packets=new)
        if applied:
            choice.update(applied)
            choice['terrain']['fixtures'] = {k: new[k]['fixture']['sha256'] for k in ('map', 'start')}
        choices.append(choice)

    table, crc = raw_blob(struct.pack('<H', len(rows)) + b''.join(rows))
    for i, actor in enumerate(two['actors']):
        ready = actor['profile']['packets']['ready']
        _, payload = load(ready)
        _, ready_end, _ = blob(payload)
        tail = codec.read_fields(payload[ready_end:], [4, 4, 1, 4])
        tail[0] = crc
        actor['profile']['packets']['ready'] = save(
            ready, payload[:ready_end] + codec.pack_fields(tail, [4, 4, 1, 4]), f'ready-{i}')
    packets['table'] = save(packets['table'], table, 'map-table')
    packets.update(choices[0]['packets'])
    room.update(map_choices=choices, map_profile=choices[0]['map_profile'])
    if 'terrain' in choices[0]:
        room['terrain'] = choices[0]['terrain']
    if scope == 'f1-a':
        enable_floor_policies(room, floor_kinds)

    # F1 mở khóa theo key pack; G1 giữ danh sách đã nghiệm thu.
    chars = json.loads((cache / 'tables.json').read_bytes())['characters']
    round_config = two['managed_rooms']['player_round']
    if unlock_all:
        round_config['characters'] = sorted(map(int, chars))
    round_config['pack_stats'] = {str(key): chars[str(key)] for key in round_config['characters']}
    if scope in ('f1', 'f1-a'):
        two['managed_rooms']['capacity'] = 2
