"""Đọc pack KR 00967 và tạo cache dữ liệu phục vụ. Chỉ dùng thư viện chuẩn."""
import argparse
from contextlib import ExitStack
import hashlib
import os
import json
from pathlib import Path
import stat
import struct
import tempfile
import xml.etree.ElementTree as ET

from content_census import decrypt_idx, decrypt_entry, parse_idx, parse_item_attr, parse_bomber_attr, parse_map_group
from export_map_profile import decode_map, load_tile_table

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
TERRAIN_SEEDS = dict(zip((*G1, 'ExVillage01'), (
    '6760c3bf47d75cda0273d3ba476c8d0e9a972f582c731606f8c2f4466c76555f',
    '5cda90d6e0b21a910352cddfc55c1c655ab60f8c167f044e967d92cf725e3dc9',
    'b531fb3c1b876a805c75cd329c4dc130a3b8f0b6609ff1635f6384606048dda2',
    '1d3274a4c5e6cb72c553351680a0592d2e1134c614f71d2efcb846382c92496e',
    '0e7152a3a10dee15dc6c9de100f87abe00ea6de77b22a90200934eb34d33419b',
    'e077fddcf2624eb50643b8c9e473627d2bf5d6fce765881cf409cc3387e79135',
    'c96dff8c4d148a229bef97ba8c3a279f1a1cac3516281de1689385353b972144',
)))
SCHEMA = 'kr00967-export-v1'


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


def outputs(files, maps, measured):
    def pack(name):
        return Pack(files[name + '.idx'], files[name + '.idd'])

    attr = pack('FxContents')
    all_items = parse_item_attr(attr.read('Attr/ItemAttr'))['items']
    items = [all_items[i] for i in ITEMS]
    if any(row['name'] != ITEMS[row['kind']] for row in items):
        raise ValueError('ItemAttr có ID khác KR 00967')
    characters = {}
    for entry in parse_bomber_attr(attr.read('Attr/BomberAttr')):
        if entry['name'] in ('C00', 'C01'):
            key = int(entry['name'][1:])
            if key in characters or entry['values'][0] != key:
                raise ValueError('BomberAttr có key trùng hoặc sai KR 00967')
            speed, max_speed, cap, max_cap, power, max_power = entry['values'][2:8]
            characters[key] = dict(speed=speed, maximum_speed=max_speed, capacity=cap,
                                   maximum_capacity=max_cap, power=power, maximum_power=max_power)
    if set(characters) != {0, 1}:
        raise ValueError('BomberAttr thiếu C00 hoặc C01')
    groups = parse_map_group(attr.read('Attr/MapGroup'))
    yield 'tables.json', dict(items=[{k: row[k] for k in ('kind', 'name', 'weight')} for row in items],
                              characters=characters,
                              map_groups=[dict(row, maps=[m for m in row['maps'] if m.rsplit('/', 1)[-1] in maps])
                                          for row in groups])
    levels = pack('FxMap')
    order = [name for name, *_ in levels.order if name.startswith('Map/')]
    table = load_tile_table(pack('fx')) if measured else None
    for name in maps:
        entry = 'Map/' + name
        raw = levels.read(entry)
        profile = decode_map(raw)
        profile.update(name=name, map_code=order.index(entry), payload_sha256=sha(raw),
                       pack_items={row['name']: row['kind'] for row in items})
        if (profile['width'], profile['height']) != (15, 13) or profile['scripted'] or not 2 <= len(profile['spawns']) <= 8:
            raise ValueError(f'Map ngoài phạm vi phục vụ: {name}')
        yield name + '.json', profile
        if measured and name in TERRAIN_SEEDS:
            yield name + '.rules.json', terrain_for(profile, table)


def expected_members(maps, measured):
    return {'tables.json', *(name + '.json' for name in maps),
            *(name + '.rules.json' for name in maps if measured and name in TERRAIN_SEEDS)}


def checked_cache(path, identity, pins, maps, measured):
    if path.is_symlink() or not path.is_dir():
        raise ValueError('Thiếu cache hợp lệ; hãy chạy export')
    manifest_path = path / 'manifest.json'
    if (manifest_path.is_symlink() or not manifest_path.is_file()
            or manifest_path.stat().st_size > 65_536):
        raise ValueError('Thiếu manifest cache; hãy chạy export')
    manifest = json.loads(manifest_path.read_bytes())
    expected = expected_members(maps, measured)
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


def cache_identity(pins, maps, measured):
    return sha(encode(dict(schema=SCHEMA, packs=pins, maps=maps, measured=measured)))


def export(game, cache, *, pins=PACK_SHA256, maps=MAPS, measured=True, pack_files=None):
    """Các tham số pins/maps khác chỉ được dùng qua API trong kiểm thử."""
    game = game.resolve(strict=True)
    if cache.is_symlink():
        raise ValueError('Cache không được là symlink')
    cache = cache.resolve()
    if cache.is_relative_to(game):
        raise ValueError('Cache phải nằm ngoài thư mục game chỉ đọc')
    identity = cache_identity(pins, maps, measured)
    dest = cache / identity
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
        if dest.exists() or dest.is_symlink():
            checked_cache(dest, identity, pins, maps, measured)
            return dest, True
        cache.mkdir(mode=0o700, parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(prefix='.export-', dir=cache) as work:
            temp = Path(work) / 'data'
            temp.mkdir(mode=0o700)
            hashes = {}
            for name, value in outputs(files, maps, measured):
                data = encode(value)
                (temp / name).write_bytes(data)
                hashes[name] = sha(data)
            if any(file_sha(stream) != pins[name] for name, stream in files.items()):
                raise ValueError('Pack đã đổi trong lúc xuất')
            (temp / 'manifest.json').write_bytes(encode(dict(schema=SCHEMA, identity=identity,
                                                             packs=pins, files=hashes)))
            checked_cache(temp, identity, pins, maps, measured)
            try:
                temp.rename(dest)
            except OSError:
                if not dest.is_dir():
                    raise
                checked_cache(dest, identity, pins, maps, measured)
                return dest, True
    return dest, False


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--game-dir', type=Path, required=True)
    parser.add_argument('--cache', type=Path, required=True)
    args = parser.parse_args()
    try:
        path, reused = export(args.game_dir, args.cache)
    except (OSError, ValueError, KeyError, IndexError, struct.error, ET.ParseError) as exc:
        parser.exit(1, f'Không thể xuất pack KR 00967: {exc}\n')
    print(json.dumps(dict(cache=str(path), reused=reused)))


if __name__ == '__main__':
    main()
