"""Nối dữ liệu đã xuất với template wire có trong gói Linux."""
import argparse
import json
from pathlib import Path
import struct

from kr_pack_export import (F1_CHARACTERS, F1_FIXED_ITEMS, F1_HIDDEN_ITEMS, F2_ADAPTER,
                            PACK_SHA256, cache_identity, checked_cache, encode, load_f2_maps, sha)
from rust_map_profile import blob, raw_blob, table_row
from server import probe_codec as codec


# Cumulative Policy::allows sets from pickups.rs; numeric ItemUseKind values
# come from kr-wire's battle.rs. Both F1-d policies inherit F1-b extended only.
_BASIC = frozenset({0, 1, 2, 5})
_SHOE_PUNCH = _BASIC | {3, 4}
_EXTENDED = _SHOE_PUNCH | {*range(67, 73), 77, 143, 146, 224, 225}
_AUX = _EXTENDED | {148, 226, 227, 228, 229, 230}
_REMOTE_SENSOR = _AUX | {151, 152}
_REMOTE_DRILL = _REMOTE_SENSOR | {149, 150}
_VEHICLES = _EXTENDED | set(range(40, 45))
_ALL_SHIPS = _VEHICLES | {*range(45, 67), 331}
_FULL_ITEM_POLICIES = (
    ('report_derived_basic', _BASIC),
    ('report_derived_shoe_punch', _SHOE_PUNCH),
    ('report_derived_f1b_extended', _EXTENDED),
    ('report_derived_f1b_aux', _AUX),
    ('report_derived_f1b_remote_sensor', _REMOTE_SENSOR),
    ('report_derived_f1b_remote_drill', _REMOTE_DRILL),
    ('report_derived_f1d_vehicles', _VEHICLES),
    ('report_derived_f1d_all_ships', _ALL_SHIPS),
)
_FULL_ITEM_KINDS = _REMOTE_DRILL | _ALL_SHIPS

# characters.rs::CLASS_C_ROWS supplies stats internally. Only these four
# rows have maximum_power > 9 and must stay out of terrain rooms.
_CLASS_C_KEYS = frozenset((*range(4000, 4019), *range(4020, 4025),
                           *range(4027, 4031), *range(5000, 5005)))
_CLASS_C_POWER_TEN = frozenset({4006, 4018, 4029, 4030})
_SCOPES = ('g1', 'f1', 'f1-a', 'f1-full', 'f2-optin')
# Các preset runtime do operator điều khiển trong room_modes/runtime.rs.
# Họ map trong catalog không xác định preset của phòng.
_F2_RUNTIME_MODES = frozenset(('revive', 'timebomb', 'leader_capture',
                              'hedgehog', 'free_1v7', 'co_op'))


def enabled_characters(chars, *, uses_terrain=False):
    """Return the pack roster accepted by the selected service floor."""
    return sorted(
        int(key) for key, stats in chars.items()
        if not uses_terrain or stats['maximum_power'] <= 9
    )


def floor582(floor, *, width=15, height=13, scope='f1-a'):
    if (width, height) != (15, 13) and not (scope == 'f1-full' and (width, height) == (19, 17)):
        raise ValueError('Kích thước lưới floor không được hỗ trợ')
    lists = (floor['fixed'], floor['hidden'], [])
    if sum(map(len, lists)) > 64:
        raise ValueError('Floor vượt giới hạn')
    fixed = set(F1_FIXED_ITEMS.values())
    hidden = set(F1_HIDDEN_ITEMS.values())
    if scope == 'f1-full':
        fixed |= _FULL_ITEM_KINDS
        # Catalog::from_floor admits vehicles and ships in list0 only.
        hidden |= _REMOTE_DRILL
    inner = bytearray()
    index = 0
    for group, allowed in zip(lists, (fixed, hidden, ())):
        inner.extend(struct.pack('<H', len(group)))
        for y, x, kind in group:
            if not (0 <= y < height and 0 <= x < width and kind in allowed):
                raise ValueError('Floor F1 có ô hoặc item ngoài phạm vi')
            inner.extend(struct.pack('<IBBI', index, y, x, kind))
            index += 1
    if sum(kind == 145 for _, _, kind in floor['fixed']) > 1:
        raise ValueError('Floor F1 chỉ hỗ trợ một Kim cố định')
    return raw_blob(inner)[0]


def enable_floor_policies(room, kinds, *, scope='f1-a'):
    if scope == 'f1-full':
        item_kinds = kinds - {145}
        unknown = item_kinds - _FULL_ITEM_KINDS
        if unknown:
            raise ValueError(f'Floor F1-full có item ngoài phạm vi: {sorted(unknown)}')
        if item_kinds:
            for policy, allowed in _FULL_ITEM_POLICIES:
                if item_kinds <= allowed:
                    room['item_policy'] = policy
                    break
            else:
                raise ValueError('Floor F1-full trộn item F1-b và F1-d không có item_policy chung')
    elif kinds & {3, 4}:
        room['item_policy'] = 'report_derived_shoe_punch'
    if 145 in kinds:
        room['life_actions'] = ('needle_rescue' if room.get('life_actions') in ('rescue', 'needle_rescue')
                                else 'needle')


def _check_f2_room_modes(room):
    policy = room.get('f2_modes', {})
    if not isinstance(policy, dict) or policy.keys() - {'runtime_overrides'}:
        raise ValueError('F2 opt-in chỉ hỗ trợ f2_modes.runtime_overrides')
    overrides = policy.get('runtime_overrides', {})
    if (not isinstance(overrides, dict) or overrides.keys() - {'free', 'manner'}
            or any(not isinstance(mode, str) or mode not in _F2_RUNTIME_MODES
                   for mode in overrides.values())):
        raise ValueError('Preset runtime F2 chưa hỗ trợ')
    if len(set(overrides.values())) > 1:
        raise ValueError('F2 opt-in cần cùng preset runtime cho các selector đã bật')


def _f2_inputs(cache, maps):
    width, height = 15, 13
    cell_count = width * height
    if any(not isinstance(name, str) for name in maps):
        raise ValueError('Tên map F2 phải là chuỗi')
    evidence = load_f2_maps(maps)
    if len({row['family'] for row in evidence.values()}) != 1:
        raise ValueError('F2 opt-in không trộn họ map trong cùng phòng')
    identity = cache_identity(PACK_SHA256, maps, True, f2_maps=evidence)
    # checked_cache dùng cờ f1 để yêu cầu rules cho mọi map của bản xuất.
    manifest = checked_cache(cache, identity, PACK_SHA256, maps, measured=True, f1=True)

    def standard_grid(value):
        return all(type(value.get(key)) is int and value[key] == expected
                   for key, expected in (('width', width), ('height', height)))

    def read(member):
        data = (cache / member).read_bytes()
        if sha(data) != manifest['files'][member]:
            raise ValueError(f'File cache sai hash: {member}')
        return json.loads(data)

    choices, codes = {}, set()
    for name, expected in evidence.items():
        profile = read(name + '.json')
        if not isinstance(profile, dict):
            raise ValueError(f'Profile F2 không hợp lệ: {name}')
        if not standard_grid(profile):
            raise ValueError(f'Lưới F2 phải là 15x13: {name}')
        if (profile.get('name') != name or profile.get('adapter') != F2_ADAPTER
                or profile.get('f2_family') != expected['family']
                or profile.get('payload_sha256') != expected['payload_sha256']
                or profile.get('version') != expected['version']
                or profile.get('scripted') is not expected['scripted']
                or profile.get('rectangles') != []):
            raise ValueError(f'Profile không khớp bằng chứng F2: {name}')
        code, spawns = profile.get('map_code'), profile.get('spawns')
        if type(code) is not int or not 0 <= code <= 65535 or code in codes:
            raise ValueError(f'Mã map F2 không hợp lệ hoặc bị trùng: {name}')
        codes.add(code)
        if (not isinstance(spawns, list) or not 2 <= len(spawns) <= 8
                or any(not isinstance(pair, list) or len(pair) != 2
                       or any(type(v) is not int for v in pair)
                       or not (0 <= pair[0] < width and 0 <= pair[1] < height) for pair in spawns)
                or len(set(map(tuple, spawns))) != len(spawns)):
            raise ValueError(f'Spawn F2 không hợp lệ: {name}')

        rules = read(name + '.rules.json')
        if not isinstance(rules, dict) or rules.keys() != {'terrain'}:
            raise ValueError(f'Bản xuất F2 chỉ hỗ trợ địa hình tĩnh: {name}')
        terrain = rules['terrain']
        if (not isinstance(terrain, dict)
                or terrain.keys() - {'width', 'height', 'kinds', 'map_sha256', 'seed_sha256', 'ray_ports'}
                or not standard_grid(terrain)):
            raise ValueError(f'Địa hình F2 phải là lưới 15x13 đã xuất: {name}')
        kinds, ports = terrain.get('kinds'), terrain.get('ray_ports', [])
        if (not isinstance(kinds, list) or len(kinds) != cell_count
                or any(type(k) is not int or not 0 <= k <= 2 for k in kinds)
                or not isinstance(ports, list) or len(ports) not in (0, cell_count)
                or any(type(p) is not int or not 0 <= p <= 15 for p in ports)
                or terrain.get('map_sha256') != expected['payload_sha256']
                or terrain.get('seed_sha256') != sha(bytes(kinds))
                or any(kinds[y * width + x] for x, y in spawns)):
            raise ValueError(f'Địa hình không khớp profile F2: {name}')

        # Profile Rust hiện chỉ nhận marker probe cho Zombie04. Các profile
        # tĩnh dùng schema chuẩn, không chuyển metadata item/script thành luật.
        serving = dict(name=name, map_code=code, width=width, height=height, spawns=spawns,
                       payload_sha256=expected['payload_sha256'], scripted=expected['scripted'])
        if serving['scripted']:
            if (name, code, len(spawns)) != ('Zombie04', 1143, 8):
                raise ValueError(f'Profile script F2 chưa có trong phạm vi probe Rust: {name}')
            serving['adapter'] = 'f2f3-probe-r1'
        choices[name] = (serving, dict(terrain, maximum_power=9))
    return choices, read('tables.json')['characters']


def configure(config, bundle, cache, out, maps, scope, *, unlock_all=False, allow_class_c=False):
    """Gắn fixture map; F2 chỉ dùng runtime khi template khai báo preset."""
    two = config['diagnostic_two_lobby']
    room = two['fixed_room']
    packets = room['packets']
    f2 = scope == 'f2-optin'
    if f2:
        _check_f2_room_modes(room)
        f2_maps, chars = _f2_inputs(cache, maps)

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
    if f2:
        if inner[30 + old_size] != 0:
            raise ValueError('F2 runtime cần mẫu Start roomtype 0')
        if len(maps) == 1 and len(f2_maps[maps[0]][0]['spawns']) < count:
            raise ValueError('Map F2 không đủ spawn cho mẫu Start')

    # Floor packets are server policy. Preserve a floor only for its original
    # map and G1 mode; do not copy the CP382 diagnostics into other maps.
    floors = {original_name(old_start): load(packets['floor'])[1]}
    for choice in room['map_choices']:
        floors[original_name(choice['packets']['start'])] = load(choice['packets']['floor'])[1]

    rows, choices, floor_kinds = [], [], set()
    scripted = False
    for index, name in enumerate(maps):
        if f2:
            profile, terrain = f2_maps[name]
            path = out / f'map-{index}-profile.json'
            data = encode(profile)
            path.write_bytes(data)
        else:
            path = cache / (name + '.json')
            data = path.read_bytes()
            profile = json.loads(data)
        if scope == 'f1-full' and profile.get('scripted') is True:
            scripted = True
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
        if f2:
            applied = dict(terrain=terrain)
        else:
            applied = (json.loads(rules.read_bytes())
                       if scope == 'f1-full' or (scope in ('g1', 'f1-a') and rules.is_file()) else {})
        floor = floors.get(name, raw_blob(bytes(6))[0]) if scope == 'g1' else raw_blob(bytes(6))[0]
        if scope in ('f1-a', 'f1-full'):
            plan = applied.pop('floor')
            if scope == 'f1-full':
                floor = floor582(plan, width=profile['width'], height=profile['height'], scope=scope)
            else:
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
            if scope != 'f1-full' or 'terrain' in applied:
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
    # Rust's selectable catalog requires 2–16 choices. A single F2 map
    # uses the standalone profile, packet and terrain pins.
    room.update(map_choices=[] if f2 and len(choices) == 1 else choices,
                map_profile=choices[0]['map_profile'])
    if 'terrain' in choices[0]:
        room['terrain'] = choices[0]['terrain']
    if scope in ('f1-a', 'f1-full'):
        enable_floor_policies(room, floor_kinds, scope=scope)
    if scripted:
        room['allow_f1e_dynamic_scripts'] = True
    if f2:
        # Giữ floor trống và không bật các route script F1-e của host.
        room.update(item_policy='report_derived_basic', life_actions='disabled',
                    allow_f1e_dynamic_scripts=False)

    # Chế độ thường giữ danh sách đã chọn trong recipe. O1 mở thêm các
    # nhân vật trong pack mà mọi map của phòng có thể dùng.
    if not f2:
        chars = json.loads((cache / 'tables.json').read_bytes())['characters']
    round_config = two['managed_rooms']['player_round']
    uses_terrain = 'terrain' in room or any('terrain' in choice for choice in choices)
    if unlock_all:
        # Match the service's measured terrain scope; retain the original
        # stats so a supported character plays with the same stats as the client.
        if scope == 'f1-full':
            chars = {key: stats for key, stats in chars.items() if int(key) in F1_CHARACTERS}
        round_config['characters'] = enabled_characters(chars, uses_terrain=uses_terrain)
    ordinary = round_config['characters']
    if scope == 'f1-full':
        ordinary = [key for key in ordinary if key not in _CLASS_C_KEYS]
        round_config['characters'] = ordinary
        if allow_class_c:
            room['allow_class_c_characters'] = True
            class_c = _CLASS_C_KEYS - _CLASS_C_POWER_TEN if uses_terrain else _CLASS_C_KEYS
            round_config['characters'] = ordinary + sorted(class_c)
    round_config['pack_stats'] = {str(key): chars[str(key)] for key in ordinary}
    if scope in ('f1', 'f1-a'):
        two['managed_rooms']['capacity'] = 2


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bundle', required=True, type=Path, help='Thư mục chứa fixture nguồn')
    parser.add_argument('--config', type=Path,
                        help='Template JSON (mặc định: config.template.json trong bundle). '
                             'F2 runtime dùng f2_modes.runtime_overrides trong fixed_room.')
    parser.add_argument('--cache', required=True, type=Path, help='Thư mục cache do exporter trả về')
    parser.add_argument('--out', required=True, type=Path, help='Thư mục mới cho config.json và fixture')
    parser.add_argument('--map', required=True, action='append',
                        help='Tên map; với F2 lặp lại đúng thứ tự --f2-map khi xuất')
    parser.add_argument('--scope', choices=_SCOPES, default='g1')
    parser.add_argument('--unlock-all', action='store_true')
    parser.add_argument('--allow-class-c', action='store_true')
    args = parser.parse_args()
    try:
        source = args.config if args.config is not None else args.bundle / 'config.template.json'
        config = json.loads(source.read_bytes())
        out = args.out.absolute()
        out.mkdir()
        configure(config, args.bundle.resolve(), args.cache.absolute(), out, args.map, args.scope,
                  unlock_all=args.unlock_all, allow_class_c=args.allow_class_c)
        (out / 'config.json').write_bytes(
            (json.dumps(config, ensure_ascii=False, separators=(',', ':')) + '\n').encode())
    except (OSError, ValueError, KeyError, TypeError, IndexError, struct.error) as exc:
        parser.exit(1, f'Không thể tạo cấu hình pack KR 00967: {exc}\n')


if __name__ == '__main__':
    main()
