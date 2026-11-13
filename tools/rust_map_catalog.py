"""Dựng catalog chọn map từ profile pack; candidate cần native UI/consumer proof.

Chỉ tạo fixture trong local, giữ template/policy hiện có và floor rỗng trên map
mới. Không chỉnh exporter, client hoặc pack; bảng nhiều row có budget hữu hạn.
"""
import argparse
import copy
import json
from pathlib import Path
import sys
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from tools import rust_map_profile as adapter
from server import probe_codec as codec
from server.room_title import _encode_bytes


def catalog(base, profiles, out):
    if not 2 <= len(profiles) <= 16 or len({p['map_code'] for p in profiles}) != len(profiles):
        raise ValueError('Catalog cần2..16mã map riêng')
    variants = []
    rows = []
    def save(pin, body, path):
        parent = adapter.pinned(pin)
        frame = codec.encode(parent[:37], pin['opcode'], body, pin['sequence'])
        path.write_bytes(frame)
        pin['fixture'] = dict(path=str(path.resolve()), bytes=len(frame), sha256=adapter.sha(frame))
    for index, profile in enumerate(profiles):
        directory = out / ('map-' + str(index)); directory.mkdir(mode=0o700)
        value, report = adapter.adapt(base, profile, directory)
        packets = value['diagnostic_two_lobby']['fixed_room']['packets']
        table, _, _ = adapter.blob(codec.frame_body(adapter.pinned(packets['table']), packets['table']['sequence']))
        row = bytearray(table[2:]); row[:4] = index.to_bytes(4, 'little'); rows.append(row)
        pin = packets['map']; body = codec.pack_fields([0, profile['map_code'], index], [2, 2, 4]); save(pin, body, directory/'selected-map.bin')
        pin = packets['start']; body = codec.frame_body(adapter.pinned(pin), pin['sequence']); inner, end, _ = adapter.blob(body, 1)
        name_size = int.from_bytes(inner[22:26], 'little'); dynamic_at = 26 + name_size
        inner = inner[:dynamic_at] + index.to_bytes(4, 'little') + inner[dynamic_at+4:]
        raw, _ = adapter.raw_blob(inner); save(pin, body[:1] + raw + body[end:], directory/'selected-start.bin')
        (directory/'provenance.json').write_text(json.dumps(report, ensure_ascii=False, indent=2)+'\n')
        variants.append(value)
    value = variants[0]; two = value['diagnostic_two_lobby']; room = two['fixed_room']
    table, crc = adapter.raw_blob(len(rows).to_bytes(2, 'little') + b''.join(rows))
    save(room['packets']['table'], table, out/'catalog-table.bin')
    for index, actor in enumerate(two['actors']):
        pin = actor['profile']['packets']['ready']; body = codec.frame_body(adapter.pinned(pin), pin['sequence']); _, end, _ = adapter.blob(body)
        tail = codec.read_fields(body[end:], [4, 4, 1, 4]); tail[0] = crc
        save(pin, body[:end] + codec.pack_fields(tail, [4, 4, 1, 4]), out/('ready-'+str(index)+'.bin'))
    choices = []
    for index, (variant, profile) in enumerate(zip(variants, profiles)):
        p = out/('profile-'+str(index)+'.json'); data=(json.dumps(profile, ensure_ascii=False, indent=2)+'\n').encode(); p.write_bytes(data)
        choices.append(dict(map_profile=dict(path=str(p.resolve()), bytes=len(data), sha256=adapter.sha(data)), packets={k: copy.deepcopy(variant['diagnostic_two_lobby']['fixed_room']['packets'][k]) for k in ['map','start','play','floor']}))
    room.update(map_choices=choices, map_profile=choices[0]['map_profile'])
    room.pop('terrain', None)
    return value, dict(source='design', status='candidate', maps=[dict(name=p['name'],code=p['map_code'],dynamic_index=i)for i,p in enumerate(profiles)],metadata_crc=crc,limits='Generated input only; native multi-row metadata/117/UI/Start proof required. No terrain/drop/item or broader map acceptance inferred.')


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--base',type=Path,required=True);parser.add_argument('--profile',type=Path,action='append',required=True);parser.add_argument('--out',type=Path,required=True);args=parser.parse_args()
    out=args.out.resolve();local=(Path(__file__).resolve().parents[1]/'local').resolve()
    if not out.is_relative_to(local) or out==local:parser.error('Đầu ra chỉ trong local/')
    out.mkdir(mode=0o700,parents=True,exist_ok=False)
    value,report=catalog(json.loads(args.base.read_text()),[json.loads(p.read_text())for p in args.profile],out)
    (out/'config.json').write_text(json.dumps(value,indent=2)+'\n');(out/'provenance.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n');print('Đã dựng candidate map catalog; cần native proof theo PLAN.')


if __name__=='__main__':main()
