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
F1A_MEMBERS_SHA256 = '92663b1d6e9697a2cc449d5f51782cbd7603cc11f51054aa53e85a517d6ea815'
FOUR_SPAWNS = [(1, x) for x in range(4)]


def payload(body='', size=(15, 13), spawns=None, obj=-1, overlay=-1):
    width, height = size
    if spawns is None:
        spawns = [(0, x) for x in range(7)] + [(width - 1, height - 1)]
    head = struct.pack('<20i', 1001, 0, 0, width, height, len(spawns), *([0] * 14))
    grid = struct.pack('<3i', -1, obj, overlay) + struct.pack('<3i', -1, -1, -1) * (width * height - 1)
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

    def test_new_xml_families_validate_the_live_tree(self):
        objects = ('<ScriptObjects><ScriptObject><InitInfo class="Mob"/>'
                   '<TriggerTable><Time until="1"><Trigger Type="Tick" ActionList="Start"/></Time></TriggerTable>'
                   '<ActionList name="Start"><Action name="Wait" time="1"/></ActionList>'
                   '</ScriptObject></ScriptObjects>')
        items = '<ItemTables><ItemTable><Item name="Item/ItemBomb" weight="1"/></ItemTable></ItemTables>'
        aqua = f'<StageClearScript>{items}{objects}</StageClearScript>'
        forest = (POPUP.replace('RetroCampStart', 'RetroForestStart')
                  + '<retroscript><InitInfos><cloud cloudappearloop="2000">'
                  '<Respawn row="1" col="1"/></cloud></InitInfos></retroscript>')
        village = (POPUP.replace('RetroCampStart', 'RetroVillageStart')
                   + '<retroscript><InitInfos><retroride><rideinfo ridetype="40"/>'
                   '</retroride></InitInfos></retroscript>')
        option = '<MapScriptOption playTime="90000"/>'
        rescue = f'{option}<RescueAliensScript><InitInfos><InitInfo/></InitInfos>{objects}</RescueAliensScript>'
        wriggle = (f'{option}<WriggleScript><Domains><Domain index="0"/></Domains>'
                   f'<Matches><Match vs="0"/></Matches>{items}<Sweeping/>{objects}</WriggleScript>')
        retro_wriggle = POPUP.replace('Retro/RetroCampStart', 'RetroWriggle/Y2KWriggleStart') + wriggle
        coloring = ('<ColoringPattern groupIdx="0" regensecond = 3 minboxcount=7 maxboxcount=20 '
                    'combocount=7 comboIntervalTime=3 comboRunTime=5 scoreLevelGap=5 maxScoreLevel=5 '
                    'timeLevelGap=30 maxTimeLevel=3/>')
        for name, body, spawns, malformed in (
            ('Aqua03', aqua, None, aqua.replace('<Action name', '<item name')),
            ('RetroForestN01', forest, None, forest.replace('Respawn', 'rideinfo')),
            ('RetroVillageN06', village, None, village.replace('rideinfo', 'Respawn')),
            ('Fm_RescueET01', rescue, FOUR_SPAWNS, rescue.replace('90000', '90001')),
            ('CaJJ_Wriggle07', wriggle, FOUR_SPAWNS, wriggle.replace('<Match ', '<Unknown ')),
            ('Fm_Wriggle02', wriggle, FOUR_SPAWNS, retro_wriggle),
            ('RetroWriggleN05', retro_wriggle, FOUR_SPAWNS, wriggle),
            ('Fm_Coloring03', coloring, FOUR_SPAWNS, coloring.replace('regensecond = 3', 'regensecond = frog')),
        ):
            with self.subTest(name=name):
                raw = payload(body, spawns=spawns)
                profile = read(name, raw)
                self.assertTrue(profile['scripted'])
                self.assertEqual(profile['adapter'], 'f1e-adapter-r1')
                self.assertFalse(profile.get('disable_bombs', False))
                with self.assertRaisesRegex(ValueError, 'Map ngoài phạm vi'):
                    read(name, raw, f1e=False)
                wrong_count = None if spawns else FOUR_SPAWNS
                duplicate = [(1, 1)] * len(profile['spawns'])
                for invalid_spawns in (wrong_count, duplicate):
                    with self.assertRaisesRegex(ValueError, 'Map ngoài phạm vi'):
                        read(name, payload(body, spawns=invalid_spawns))
                for invalid in (malformed, body + '<time ms="0"><item/></time>',
                                body + '<?unknown live?>', body + '<!--truncated'):
                    with self.assertRaisesRegex(ValueError, 'Script ngoài adapter'):
                        read(name, payload(invalid, spawns=spawns))

    def test_new_script_trailer_rejects_groups_and_records(self):
        from kr_f1e_script_audit import map_tail_offset

        raw = payload('<StageClearScript><ItemTables><ItemTable><Item/></ItemTable></ItemTables>'
                      '<ScriptObjects><ScriptObject><InitInfo/></ScriptObject></ScriptObjects></StageClearScript>')
        offset = map_tail_offset(raw)
        for trailer in (struct.pack('<3i', 1, 0, 0), struct.pack('<3i', 0, 1, 1) + b'a' + bytes(14)):
            candidate = raw[:offset] + trailer + raw[offset + 8:]
            with self.assertRaisesRegex(ValueError, 'Script ngoài adapter'):
                adapter.f1e_bombs_disabled(candidate, 'Aqua03', f1e=True)

    def test_new_tiles_only_project_on_f1e_with_matching_flags(self):
        ray = 0x01010101
        clear = 0, 0, 0, 0, 0, 0
        for tile, fields, kind, overlay in (
            ('Tile58_Deco/Cloud01', (4,) + (0,) * 15, 0, True),
            ('Tile61_Deco/Static05_2', (4,) + (0,) * 15, 0, True),
            ('Tile61_Passable/GreenHouse03', (3, *clear[1:], ray, ray, ray, 0, 0, 1, 0, 0, 0, 0), 0, False),
            ('TileTransparent_Passable/Passable02', (3, *clear[1:], ray, ray, ray, ray, 1, 1, 1, 0, 0, 0), 0, False),
            ('Tile20_Static/CenterStatic03', (1, *clear[1:], 1, 2, 1, 0, 0, 2010089472, 0, 0, 0, 0), 1, False),
            ('Tile64_Static/CenterStatic01', (1, *clear[1:], 1, 2, 1, 0, 0, 0, 0, 0, 0, 0), 1, False),
            ('Tile38_Box/NumBlock_09_X', (2, *clear[1:], 0, 0, 0, 2308, 0, 0, 0, 0, 0, 0), 2, False),
        ):
            with self.subTest(tile=tile):
                profile = read('RetroCampN01', payload(POPUP, spawns=[(2, x) for x in range(8)],
                                                       obj=-1 if overlay else 0, overlay=0 if overlay else -1))
                if kind == 1:
                    profile['cells'][15], profile['cells'][0] = profile['cells'][0], profile['cells'][15]
                fields = (*fields, 0, 0)
                projected = adapter.f1_terrain_rules(profile, ([tile], {tile: fields}), f1e=True)['terrain']
                self.assertEqual(projected['kinds'][0], kind)
                self.assertEqual(projected['kinds'][15], int(kind == 1))
                self.assertNotIn('ray_ports', projected)
                self.assertNotIn('ray_barriers', projected)
                self.assertNotIn('box_hits', projected)  # One hit is the default.
                with self.assertRaisesRegex(ValueError, 'Ô ngoài adapter F1-a'):
                    adapter.f1_terrain_rules(profile, ([tile], {tile: fields}))
                with self.assertRaisesRegex(ValueError, 'Ô ngoài adapter F1-e'):
                    adapter.f1_terrain_rules(profile, ([tile], {tile: (*fields[:12], 9, *fields[13:])}), f1e=True)

    def test_exploding_boxes_emit_flag_and_unverified_tiles_fail_closed(self):
        clear_spawns = [(1, x) for x in range(8)]
        for map_name, body, tile, data in [
            ('NXCL_Aqua01', '<StageClearScript/>', 'Tile30_Box/JellyBombBlock00',
             (2, 0, 0, 0, 0, 0, 0, 0, 65792, 0, 0, 0, 0, 0, 0, 0, 0, 0)),
            ('Island09', '', 'Tile53_Box/Block30',
             (2, 0, 0, 0, 0, 0, 0, 0, 256, 0, 0, 0, 0, 0, 0, 0, 0, 0)),
            ('Island06', '', 'Tile53_Box/Block32',
             (2, 0, 0, 0, 0, 0, 0, 0, 65792, 0, 0, 0, 0, 0, 0, 0, 0, 0)),
        ]:
            f1e = map_name != 'Island09' and map_name != 'Island06'
            profile = read(map_name, payload(body, spawns=clear_spawns, obj=0), f1e=f1e)
            rules = adapter.f1_rules(profile, ([tile], {tile: data}), f1e=f1e)
            self.assertEqual(rules['terrain']['exploding_boxes'][0], 1)
        coral = (3, 0, 0, 0, 0, 0, 0, 16843009, 16843009, 1, 0, 1, 0, 0, 0, 0, 0, 0)
        coral_profile = read('Aqua06', payload('<StageClearScript/>', spawns=clear_spawns, obj=0))
        coral_rules = adapter.f1_rules(coral_profile, (['Tile30_Static/Coral01'], {'Tile30_Static/Coral01': coral}), f1e=True)
        self.assertEqual(coral_rules['terrain']['kinds'][0], 0)
        with self.assertRaisesRegex(ValueError, 'spawn trên ô type 3'):
            adapter.f1_rules(read('Aqua06', payload('<StageClearScript/>', obj=0)),
                             (['Tile30_Static/Coral01'], {'Tile30_Static/Coral01': coral}), f1e=True)
        urchin = (2, 0, 0, 0, 0, 0, 255, 1, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0)
        urchin_prof = read('Aqua08', payload('<StageClearScript/>', spawns=clear_spawns, obj=0))
        urchin_rules = adapter.f1_rules(urchin_prof, (['Tile30_Box/Urchins00'], {'Tile30_Box/Urchins00': urchin}), f1e=True)
        self.assertEqual(urchin_rules['terrain']['kinds'][0], 1)
        self.assertNotIn('box_hits', urchin_rules['terrain'])
        with self.assertRaisesRegex(ValueError, 'Spawn không nằm trên ô trống'):
            adapter.f1_rules(read('Aqua08', payload('<StageClearScript/>', obj=0)),
                             (['Tile30_Box/Urchins00'], {'Tile30_Box/Urchins00': urchin}), f1e=True)
        bombproof = (10, 0, 0, 0, 0, 0, 65536, 65536, 65536, 0, 0, 1, 0, 0, 0, 0, 0, 0)
        bp_prof = read('RetroCampN01', payload(POPUP, spawns=clear_spawns, obj=0))
        bp_rules = adapter.f1_rules(bp_prof, (['Tile62_Passable/BombProof_Right01'],
                                              {'Tile62_Passable/BombProof_Right01': bombproof}), f1e=True)
        self.assertEqual(bp_rules['terrain']['ray_ports'][0], 0b0100)
        with self.assertRaisesRegex(ValueError, 'spawn trên ô type 10'):
            adapter.f1_rules(read('RetroCampN01', payload(POPUP, obj=0)),
                             (['Tile62_Passable/BombProof_Right01'],
                              {'Tile62_Passable/BombProof_Right01': bombproof}), f1e=True)
        for map_name, body, tile, data in [
            ('Aqua08', '<StageClearScript/>', 'Tile30_Box/Urchins00',
             (2, 0, 0, 0, 0, 0, 255, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0)),
            ('RetroCampN01', POPUP, 'Tile62_Passable/BombProof_Center01', bombproof),
        ]:
            profile = read(map_name, payload(body, spawns=clear_spawns, obj=0))
            with self.assertRaisesRegex(ValueError, tile):
                adapter.f1_rules(profile, ([tile], {tile: data}), f1e=True)

    def test_verified_soul_maps_and_remain_ground(self):
        width, height = 15, 13
        spawns = [(0, x) for x in range(8)]
        head = struct.pack('<20i', 1000, 0, 0, width, height, len(spawns), *([0] * 14))
        grid = struct.pack('<2i', -1, -1) * (width * height)
        soul_tail = struct.pack('<5i', 1, 1, 1, 2, 0) + struct.pack('<I', 0)
        raw = (head + b''.join(struct.pack('<2i', *p) for p in spawns) + bytes(4)
               + grid + struct.pack('<3i', 0, 60, 0) + soul_tail)
        for name in ('EventForest00', 'EventPatrit00', 'ExBula01', 'ExBula02'):
            prof = read(name, raw, f1e=False)
            self.assertFalse(prof['scripted'])
        with self.assertRaisesRegex(ValueError, 'map parser không đọc hết payload'):
            read('Village10', raw, f1e=False)
        for tile, d15 in [('Tile15_Ground/Remain00', 7405312), ('Tile15_Ground/Remain_Green01', 65280)]:
            data = (0, 0, 0, 0, 0, 0, 1, 1, 0, 0, -1, 0, 0, 0, 0, d15, 0, 0)
            prof = dict(name='ExBula01', width=15, height=13, payload_sha256='0' * 64,
                        spawns=[[x, 0] for x in range(8)], rectangles=[], cells=[(0, -1)] + [(-1, -1)] * 194,
                        drops=[], placement_header=60, placements=[dict(name='Item/ItemShoe', x=0, y=1, probability=100)])
            rules = adapter.f1_rules(prof, ([tile], {tile: data}))
            self.assertEqual(len(rules['floor']['fixed']), 1)
        with self.assertRaisesRegex(ValueError, 'nền/cờ'):
            adapter.f1_rules(prof, (['Tile15_Ground/Remain00'], {
                'Tile15_Ground/Remain00': (0, 0, 0, 0, 0, 0, 1, 1, 0, 0, -1, 0, 0, 0, 0, 999 << 16, 0, 0),
            }))
        bula_tile = (1, 0, 0, 0, 0, 0, 1, 1, 0, 10101, 0, 1243912, 2012964084, 2012754176, -1, 1243928, 0, 0)
        bula_prof = dict(name='ExBula01', width=15, height=13, payload_sha256='0' * 64,
                         spawns=[[x, 1] for x in range(8)], rectangles=[], cells=[(-1, 0)] + [(-1, -1)] * 194,
                         drops=[], placement_header=60, placements=[])
        bula_rules = adapter.f1_rules(bula_prof, (['Tile15_Static/Bf01_01'], {'Tile15_Static/Bf01_01': bula_tile}))
        self.assertEqual(bula_rules['terrain']['kinds'][0], 1)
        with self.assertRaisesRegex(ValueError, 'Tile15_Static/Bf01_01'):
            adapter.f1_rules(bula_prof, (['Tile15_Static/Bf01_01'], {
                'Tile15_Static/Bf01_01': (*bula_tile[:9], 10199, *bula_tile[10:]),
            }))

    def test_space_and_pangland_xml_rules_fail_closed(self):
        space_ok = '<changetile ms="20000"><position x="-1" y="0"/></changetile>'
        self.assertNotIn('disable_bombs', read('CaJJ_Space02', payload(space_ok)))
        pang_ok = '<limititemcount><item name="Item/ItemBomb"/></limititemcount><time ms="2000"><bomb/><wave/></time>'
        self.assertNotIn('disable_bombs', read('PangLand03', payload(pang_ok)))
        self.assertNotIn('disable_bombs', read('PangLand06', payload(pang_ok + '\n##special attack\n')))
        for name, body in [
            ('CaJJ_Space02', ''),
            ('CaJJ_Space02', '<changetile><other/></changetile>'),
            ('PangLand03', '<time ms="2000"><item name="Item/ItemBomb"/></time>'),
            ('PangLand03', '<ground/>'),
        ]:
            with self.subTest(name=name, body=body), self.assertRaisesRegex(ValueError, 'Script ngoài adapter'):
                read(name, payload(body))

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
    unlocked_8 = {'EventForest00', 'EventPatrit00', 'ExBula01', 'ExBula02',
                  'Island06', 'Island09', 'Island10', 'NXCL_Aqua01'}
    assert len(names) == 214, len(names)
    baseline_names = [n for n in names if n not in unlocked_8]
    assert len(baseline_names) == 206
    digest = hashlib.sha256()
    with adapter.verified_packs(game) as files:
        for start in range(0, len(baseline_names), 16):
            batch = baseline_names[start:start + 16]
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
    assert digest.hexdigest() == F1A_MEMBERS_SHA256, digest.hexdigest()

    specs = json.loads((ROOT / 'docs/evidence/content/f1e-scripts-extracted.json').read_bytes())
    result = {}
    with tempfile.TemporaryDirectory(prefix='kr-f1e-pack-') as tmp:
        for name, row in specs['maps'].items():
            family = row['family']
            try:
                cache, reused = adapter.export(game, Path(tmp), maps=[name], f1e=True)
            except ValueError as error:
                result[name] = dict(result='blocked', reason=str(error))
                continue
            assert not reused
            assert adapter.export(game, Path(tmp), maps=[name], f1e=True) == (cache, True)
            blob = (cache / f'{name}.json').read_bytes()
            profile = json.loads(blob)
            assert [profile['width'], profile['height']] == row['size']
            assert profile['payload_sha256'] == row['payload_sha256']
            assert profile['map_code'] == row['code'] and len(profile['spawns']) == row['spawns']
            assert len({tuple(pair) for pair in profile['spawns']}) == row['spawns']
            assert profile['adapter'] == adapter.F1E_ADAPTER and profile['scripted'] == row['script']['marker']
            assert profile.get('disable_bombs', False) == (family == 'lovelyroom')
            rules = (cache / f'{name}.rules.json').read_bytes()
            assert json.loads(rules)['terrain'].get('disable_bombs', False) == (family == 'lovelyroom')
            result[name] = dict(result='exported', profile_sha256=adapter.sha(blob), rules_sha256=adapter.sha(rules))
    blocked = {name for name, row in result.items() if row['result'] == 'blocked'}
    assert len(result) == 96, len(result)
    assert blocked == set(), {name: result[name]['reason'] for name in sorted(blocked)}
    f1e_members = {member: digest for name, row in result.items()
                   for member, digest in ((f'{name}.json', row['profile_sha256']),
                                          (f'{name}.rules.json', row['rules_sha256']))}
    f1e_digest = adapter.sha(''.join(f'{name}:{value}\n' for name, value in sorted(f1e_members.items())).encode())
    return dict(f1a_baseline=baseline_ref, f1a_equal=len(baseline_names), f1a_pending=len(names),
                f1a_unlocked=sorted(unlocked_8), f1a_members_sha256=digest.hexdigest(),
                f1e_exported=len(result), f1e_blocked=len(blocked), f1e_members_sha256=f1e_digest, maps=result)


if __name__ == '__main__':
    cli = argparse.ArgumentParser(description=__doc__)
    cli.add_argument('--pack-root', required=True, type=Path)
    cli.add_argument('--baseline-ref', default='c8b6ff056583c88e1bba78cda6a5fb0d5c289ae2')
    args = cli.parse_args()
    print(json.dumps(verify_pack(args.pack_root, args.baseline_ref), ensure_ascii=False, indent=2))
