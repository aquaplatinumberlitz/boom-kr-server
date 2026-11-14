"""Dựng fixture map nhất quán từ profile pack, không là oracle native.

Giữ nguyên template/policy đã đo. Map mới dùng metadata raw-selector0 và floor
rỗng; terrain/drop/item của map mới cần nghiệm thu G1 riêng trước khi mở.
"""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import struct
import sys
import zlib

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from server import probe_codec as codec
from server.room_title import _decode_bytes, _encode_bytes


def sha(data):
    return hashlib.sha256(data).hexdigest()


def pinned(pin):
    data = Path(pin['fixture']['path']).read_bytes()
    if len(data) != pin['fixture']['bytes'] or sha(data) != pin['fixture']['sha256']:
        raise ValueError('Fixture nguồn không khớp pin')
    return data


def blob(body, at=0):
    if len(body) < at + 4:
        raise ValueError('Blob thiếu length')
    size = codec.read_fields(body[at:at+4], [4])[0]
    end = at + 4 + size
    if end > len(body):
        raise ValueError('Blob vượt body')
    stream = _decode_bytes(body[at+4:end])
    if len(stream) < 6 or stream[0] != 24 or int.from_bytes(stream[1:5], 'little') != len(stream)-5:
        raise ValueError('Blob sai storage header')
    if stream[5] == 0:
        inner = stream[6:]
    elif stream[5] == 1 and len(stream) >= 10:
        expected = int.from_bytes(stream[6:10], 'little')
        if expected > 4096:
            raise ValueError('Metadata inflate vượt budget')
        decoder = zlib.decompressobj()
        inner = decoder.decompress(stream[10:], expected + 1)
        if len(inner) != expected or not decoder.eof or decoder.unused_data or decoder.unconsumed_tail:
            raise ValueError('Metadata inflate sai length/tail')
    else:
        raise ValueError('Blob selector chưa hỗ trợ')
    return inner, end, stream


def raw_blob(inner):
    storage = b'\0' + inner
    stream = bytes([24]) + struct.pack('<I', len(storage)) + storage
    return codec.pack_fields([len(stream)], [4]) + _encode_bytes(stream), zlib.adler32(storage, 0) & 0xffffffff


def table_row(inner):
    if inner[:2] != b'\1\0' or len(inner) > 4096:
        raise ValueError('Chỉ metadata một map đã đo')
    at = 7  # count/u32/u8; strings are raw little-endian inside the blob.
    strings = []
    for group in range(2):
        for _ in range(2):
            if at + 4 > len(inner):
                raise ValueError('Metadata string thiếu length')
            size = int.from_bytes(inner[at:at+4], 'little')
            end = at + 4 + size
            if size > 64 or end > len(inner):
                raise ValueError('Metadata string vượt budget')
            strings.append((at, end, inner[at+4:end]))
            at = end
        at += 2
    code_at = at + 4 + 3
    if code_at + 7 != len(inner):
        raise ValueError('Metadata row/tail sai schema')
    return strings, code_at


def adapt(config, profile, out):
    value = copy.deepcopy(config)
    if (profile['scripted'] or (profile['width'], profile['height']) != (15, 13)
            or not 2 <= len(profile['spawns']) <= 8
            or len(profile['cells']) != 195
            or len(profile['payload_sha256']) != 64):
        raise ValueError('Profile map ngoài nhánh G1 hiện hành')
    name = profile['name'].encode('ascii')
    code = profile['map_code']
    if not 1 <= len(name) <= 64 or not all(chr(b).isalnum() or b == 95 for b in name) or not 0 <= code <= 65535:
        raise ValueError('Tên hoặc mã map không hợp lệ')
    room = value['diagnostic_two_lobby']['fixed_room']
    packets = room['packets']
    changes = []

    def save(alias, pin, body):
        source = pinned(pin)
        before = codec.frame_body(source, pin['sequence'])
        if before == body:
            changes.append(dict(alias=alias, changed=False, sha256=sha(source)))
            return
        data = codec.encode(source[:37], pin['opcode'], body, pin['sequence'])
        path = out / (alias + '.bin')
        path.write_bytes(data)
        pin['fixture'] = dict(path=str(path.resolve()), bytes=len(data), sha256=sha(data))
        changes.append(dict(alias=alias, changed=True, parent_sha256=sha(source), sha256=sha(data)))

    pin = packets['table']
    before = codec.frame_body(pinned(pin), pin['sequence'])
    inner, end, storage = blob(before)
    if end != len(before):
        raise ValueError('Metadata dư body')
    strings, at = table_row(inner)
    old_name = strings[1][2]
    old_code = int.from_bytes(inner[at:at+2], 'little')
    changed_map = (old_name, old_code) != (name, code)
    table_body = before
    if changed_map:
        old = strings[1]
        adjusted = bytearray(inner)
        adjusted[at:at+2] = code.to_bytes(2, 'little')
        adjusted[old[0]:old[1]] = len(name).to_bytes(4, 'little') + name
        table_body, crc = raw_blob(adjusted)
    else:
        crc = zlib.adler32(storage[5:], 0) & 0xffffffff
    save('table', pin, table_body)
    for index, actor in enumerate(value['diagnostic_two_lobby']['actors']):
        pin = actor['profile']['packets']['ready']
        body = codec.frame_body(pinned(pin), pin['sequence'])
        _, end, _ = blob(body)
        tail = codec.read_fields(body[end:], [4, 4, 1, 4])
        tail[0] = crc
        save('ready-' + str(index), pin, body[:end] + codec.pack_fields(tail, [4, 4, 1, 4]))
    pin = packets['start']
    body = codec.frame_body(pinned(pin), pin['sequence'])
    inner, end, _ = blob(body, 1)
    size = int.from_bytes(inner[22:26], 'little')
    if inner[26:26+size] != old_name or int.from_bytes(inner[20:22], 'little') != old_code:
        raise ValueError('Start nguồn khác map metadata')
    rows_at = 37 + size
    count = inner[rows_at - 5]
    if not 2 <= count <= 8 or len(inner) != rows_at + count * 26:
        raise ValueError('Start nguồn sai schema roster')
    for index in range(count):
        if inner[rows_at + index * 26 + 2] >= len(profile['spawns']):
            raise ValueError('Spawn Start vượt điểm spawn trong pack')
    inner = inner[:20] + code.to_bytes(2, 'little') + len(name).to_bytes(4, 'little') + name + inner[26+size:]
    start, _ = raw_blob(inner)
    save('start', pin, body[:1] + start + body[end:])
    for alias, widths, index in [('map', [2, 2, 4], 1), ('play', [1, 4, 2], 2)]:
        pin = packets[alias]
        fields = codec.read_fields(codec.frame_body(pinned(pin), pin['sequence']), widths)
        if fields[index] != old_code:
            raise ValueError('Mã nguồn giữa các packet không nhất quán')
        fields[index] = code
        save(alias, pin, codec.pack_fields(fields, widths))
    if changed_map:
        # No unmeasured transfer of ExVillage01 floor coordinates/drop policy.
        floor, _ = raw_blob(bytes(6))
        save('floor', packets['floor'], floor)
        room.pop('terrain', None)
    return value, dict(source='design', status='candidate', map=profile['name'], code=code,
                       metadata_crc=crc, changes=changes, changed_map=changed_map,
                       limits='Generated input, not native oracle. New map has empty floor and terrain disabled; no item/drop/physics acceptance. Spawn selectors refer to exported pack spawn order; no coordinates forced.')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for arg in ['base', 'profile', 'out']:
        parser.add_argument('--' + arg, type=Path, required=True)
    args = parser.parse_args()
    local = (Path(__file__).resolve().parents[1] / 'local').resolve()
    if not args.out.resolve().is_relative_to(local) or args.out.resolve() == local:
        parser.error('Đầu ra chỉ được nằm trong local/')
    args.out.mkdir(mode=0o700, parents=True, exist_ok=False)
    try:
        profile = json.loads(args.profile.read_text())
        result, report = adapt(json.loads(args.base.read_text()), profile, args.out)
        data = args.profile.read_bytes()
        result['diagnostic_two_lobby']['fixed_room']['map_profile'] = dict(
            path=str(args.profile.resolve()), bytes=len(data), sha256=sha(data))
        (args.out / 'config.json').write_text(json.dumps(result, indent=2) + '\n')
        (args.out / 'provenance.json').write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n')
    except (KeyError, ValueError, OSError, struct.error) as error:
        parser.exit(1, 'Lỗi dựng profile map: ' + str(error) + '\n')
    print('Đã tạo candidate map input; cần kiểm byte baseline và nghiệm thu native theo PLAN.')


if __name__ == '__main__':
    main()
