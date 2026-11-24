"""Hồi quy F1-e và phép so tùy chọn với pack KR chỉ đọc."""
import argparse
import hashlib
import json
from pathlib import Path
import struct
import subprocess
import tempfile
from types import SimpleNamespace
import unittest

import kr_pack_export as adapter

ROOT = Path(__file__).resolve().parents[1]
POPUP = ('<time ms="0"><popup name="Retro/RetroCampStart" targetcx="123" '
         'targetcy="88" darkness="100"/></time>')


def payload(body='', size=(15, 13), spawns=None, obj=-1):
    width, height = size
    if spawns is None:
        spawns = [(0, x) for x in range(7)] + [(width - 1, height - 1)]
    head = struct.pack('<20i', 1001, 0, 0, width, height, len(spawns), *([0] * 14))
    grid = struct.pack('<3i', -1, obj, -1) + struct.pack('<3i', -1, -1, -1) * (width * height - 1)
    script = f'<mapscript>{body}</mapscript>\0'.encode('cp949') if body else b''
    return (head + b''.join(struct.pack('<2i', *p) for p in spawns) + bytes(4)
            + grid + struct.pack('<6i', 0, 100, 0, 0, 0, len(script)) + script)


def read(name, raw, *, f1e=True):
    return adapter.read_map(SimpleNamespace(read=lambda _: raw), name, 1, f1=not f1e, f1e=f1e)


class F1eAdapterTests(unittest.TestCase):
    def test_lovely_room_uses_only_live_enablebomb_option(self):
        live = '<mapscriptoption enablebomb="0"/>'
        comment = '<!-- -- 주석 <ItemTable/><time ms="10"/><mapscriptoption enablebomb="1"/> -->'
        profile = read('LovelyRoom01', payload(live + comment))
        self.assertTrue(profile['scripted'])
        rules = adapter.f1_rules(profile, ([], {}), f1e=True)
        self.assertIs(rules['terrain']['disable_bombs'], True)
        self.assertEqual(rules['floor'], dict(fixed=[], hidden=[], omitted_placements=[]))
        for body in (comment, live.replace('"0"', '"1"'), live + '<time ms="10"/>', live + '<!--'):
            with self.subTest(body=body), self.assertRaisesRegex(ValueError, 'Script ngoài adapter'):
                read('LovelyRoom01', payload(body))

    def test_accepts_only_single_opening_popup_and_empty_marker(self):
        self.assertNotIn('disable_bombs', read('RetroCampN01', payload(POPUP)))
        self.assertNotIn('disable_bombs', read('Aqua01', payload('<StageClearScript/>')))
        with self.assertRaisesRegex(ValueError, 'Map ngoài phạm vi'):
            read('RetroCampN06', payload(POPUP), f1e=False)
        for name, body in [
            ('RetroCampN01', POPUP.replace('ms="0"', 'ms="1"')),
            ('RetroCampN01', POPUP + '<retroscript/>'),
            ('RetroCampN01', POPUP + '<cloud/>'),
            ('RetroCampN01', POPUP + '<rideinfo/>'),
            ('RetroCampN01', POPUP + '<Respawn/>'),
            ('RetroForestN01', POPUP),
            ('Aqua01', '<StageClearScript><TriggerTable/></StageClearScript>'),
        ]:
            with self.subTest(name=name, body=body), self.assertRaisesRegex(ValueError, 'Script ngoài adapter'):
                read(name, payload(body))

    def test_bigbattle_scope_and_f1a_cache_stay_separate(self):
        name = 'BSBigCamp08'
        raw = payload(size=(19, 17))
        self.assertEqual(len(read(name, raw)['cells']), 323)
        for candidate in (payload(), payload(size=(17, 19)),
                          payload(size=(19, 17), spawns=[(0, 0)] * 8),
                          payload(size=(19, 17), spawns=[(0, x) for x in range(8)] + [(18, 16)])):
            with self.assertRaisesRegex(ValueError, 'Map ngoài phạm vi'):
                read(name, candidate)
        with self.assertRaisesRegex(ValueError, 'Map ngoài phạm vi'):
            read(name, raw, f1e=False)
        with self.assertRaisesRegex(ValueError, 'Script ngoài adapter'):
            read(name, payload(POPUP, size=(19, 17)))
        self.assertNotEqual(adapter.cache_identity({}, [name], True, f1=True),
                            adapter.cache_identity({}, [name], True, f1e=True))

    def test_dynamic_boxes_and_bombproof_are_not_flattened(self):
        for map_name, body, tile, data in [
            ('Aqua01', '<StageClearScript/>', 'Tile30_Box/JellyBombBlock00',
             (2, 0, 0, 0, 0, 0, 0, 0, 65792, 0, 0, 0, 0, 0, 0, 0, 0, 0)),
            ('RetroCampN01', POPUP, 'Tile62_Passable/BombProof_Right01',
             (10, 0, 0, 0, 0, 0, 65536, 65536, 65536, 0, 0, 1, 0, 0, 0, 0, 0, 0)),
        ]:
            profile = read(map_name, payload(body, obj=0))
            with self.assertRaisesRegex(ValueError, tile):
                adapter.f1_rules(profile, ([tile], {tile: data}), f1e=True)

    def test_bigbattle_floor_cannot_emit_items_outside_legacy_floor_grid(self):
        profile = read('BSBigCamp08', payload(size=(19, 17)))
        profile['drops'] = [['Item/ItemBomb', 1]]
        profile['placements'] = [dict(name='Item/ItemFire', x=18, y=15, probability=100)]
        terrain = dict(width=19, height=17, kinds=[0] * 323)
        terrain['kinds'][16 * 19 + 17] = 2
        floor = adapter.f1_floor(profile, terrain, f1e=True)
        self.assertEqual(floor, dict(fixed=[], hidden=[], omitted_placements=profile['placements']))


def verify_pack(game, baseline_ref):
    from f1_pack_matrix import inventory

    baseline = {}
    source = subprocess.check_output(['git', 'show', f'{baseline_ref}:tools/kr_pack_export.py'], cwd=ROOT)
    exec(compile(source, 'baseline_kr_pack_export.py', 'exec'), baseline)
    names = sorted(name for name, row in inventory(game).items() if row['status'] == 'pending')
    assert len(names) == 206
    digest = hashlib.sha256()
    with adapter.verified_packs(game) as files:
        for start in range(0, len(names), 16):
            batch = names[start:start + 16]
            old_id = baseline['cache_identity'](adapter.PACK_SHA256, batch, True, f1=True)
            assert old_id == adapter.cache_identity(adapter.PACK_SHA256, batch, True, f1=True), batch

            def encode_batch(exporter, encode):
                for f in files.values():
                    f.seek(0)
                return {name: encode(value) for name, value in exporter(files, batch, True, f1=True)}

            old = encode_batch(baseline['outputs'], baseline['encode'])
            new = encode_batch(adapter.outputs, adapter.encode)
            assert old == new, batch
            for name, data in sorted(new.items()):
                digest.update(f'{name}:{adapter.sha(data)}\n'.encode())

    specs = json.loads((ROOT / 'docs/evidence/content/f1e-scripts-extracted.json').read_bytes())
    result = {}
    with tempfile.TemporaryDirectory(prefix='kr-f1e-pack-') as tmp:
        for name, row in specs['maps'].items():
            family = row['family']
            if row['spec'].get('variant') != 'stage_clear_marker_only' and family not in {
                'retro_popup', 'lovelyroom', 'bigbattle'
            }:
                continue
            try:
                cache, reused = adapter.export(game, Path(tmp), maps=[name], f1e=True)
            except ValueError as error:
                tile = {'aqua': 'JellyBombBlock00', 'retro_popup': 'BombProof_'}.get(family)
                assert tile and tile in str(error), (name, error)
                result[name] = dict(result='blocked', reason=str(error))
                continue
            assert not reused
            assert adapter.export(game, Path(tmp), maps=[name], f1e=True) == (cache, True)
            blob = (cache / f'{name}.json').read_bytes()
            profile = json.loads(blob)
            assert [profile['width'], profile['height']] == row['size']
            assert profile['payload_sha256'] == row['payload_sha256']
            assert profile['map_code'] == row['code'] and len(profile['spawns']) == row['spawns']
            rules = (cache / f'{name}.rules.json').read_bytes()
            assert json.loads(rules)['terrain'].get('disable_bombs', False) == (family == 'lovelyroom')
            result[name] = dict(result='exported', profile_sha256=adapter.sha(blob), rules_sha256=adapter.sha(rules))
    blocked = {name for name, row in result.items() if row['result'] == 'blocked'}
    assert len(result) == 37 and blocked == {
        'NXCL_Aqua01', 'Aqua01', 'Aqua02', 'Aqua05', 'Aqua06', 'Aqua08',
        *(f'RetroCampN{n:02}' for n in range(1, 6)),
    }, result
    return dict(f1a_baseline=baseline_ref, f1a_equal=len(names), f1a_members_sha256=digest.hexdigest(), maps=result)


if __name__ == '__main__':
    cli = argparse.ArgumentParser(description=__doc__)
    cli.add_argument('--pack-root', required=True, type=Path)
    cli.add_argument('--baseline-ref', default='c8b6ff056583c88e1bba78cda6a5fb0d5c289ae2')
    args = cli.parse_args()
    print(json.dumps(verify_pack(args.pack_root, args.baseline_ref), ensure_ascii=False, indent=2))
