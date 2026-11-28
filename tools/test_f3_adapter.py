"""Kiểm adapter F3 và hồi quy byte của các bản xuất KR 00967."""
from contextlib import redirect_stderr, redirect_stdout
import hashlib
import io
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import Mock, patch

import kr_pack_export as adapter
from f1_pack_matrix import inventory
from kr_f2_f3_tile_census import default_game_dir
from tools.test_f2_adapter import BASEBALL_WALL, BLOCKED_BASEBALL_KART, EXPANDED_FAMILIES, STATIC_GROUND, payload

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / 'docs/evidence/content'
CENSUS_NAME = 'f2-f3-tile-grid-census.json'
CATALOG_NAME = 'f3-pve-catalog.json'
GAME = default_game_dir()
SNOW_BEAR_STATIC_MAPS = ('MonSnowBear07', 'MonSnowBear23', 'MonSnowBear29')


def run_cli(*args):
    out, err = io.StringIO(), io.StringIO()
    with (patch.object(sys, 'argv', ['kr_pack_export.py', *args]),
          redirect_stdout(out), redirect_stderr(err)):
        try:
            adapter.main()
        except SystemExit as exc:
            return exc.code, err.getvalue()
    return 0, json.loads(out.getvalue())


class F3AdapterTests(unittest.TestCase):
    def test_cli_rejects_unknown_or_unready_maps_and_invalid_lists(self):
        with tempfile.TemporaryDirectory() as tmp:
            cache = Path(tmp) / 'cache'
            prefix = ('--game-dir', tmp, '--cache', str(cache))
            for names, message in (
                (('NoSuchMap',), 'NoSuchMap'),
                (('Factory07',), 'Factory07'),
                (('Baseball01',), 'Baseball01'),
                (('Baseball03',), 'Baseball03'),
                (('Kart02',), 'Kart02'),
                (('Hdn_SuperFight',), 'Hdn_SuperFight'),
                (('MonSnowBear01',), 'MonSnowBear01'),
                (('Fm_MonSnowBear01',), 'Fm_MonSnowBear01'),
                (('MonNoviceSnow01',), 'MonNoviceSnow01'),
                (('Zombie04',), 'Zombie04'),
                (('MonS2Fire03',), 'MonS2Fire03'),
                (('MonS2Sand06Boss',), 'MonS2Sand06Boss'),
                (('BonusBigReward12',), 'BonusBigReward12'),
                (('Bula00',), 'Bula00'),
                (('Bonus01',), 'Bonus01'),
                (('Tutorial01', 'Tutorial01'), '1–16'),
                (tuple(f'MonSeal{n:02}' for n in range(1, 18)), '1–16'),
                (('../Tutorial01',), '1–16'),
            ):
                with self.subTest(names=names):
                    code, err = run_cli(*prefix, *(arg for name in names for arg in ('--f3-map', name)))
                    self.assertEqual(code, 1, err)
                    self.assertIn(message, err)
                    self.assertFalse(cache.exists())

    def test_admits_only_the_static_maps_of_mon_snow_bear(self):
        selected = adapter.load_f3_maps(SNOW_BEAR_STATIC_MAPS)
        self.assertEqual({name: row['family'] for name, row in selected.items()},
                         dict.fromkeys(SNOW_BEAR_STATIC_MAPS, 'MonSnowBear'))
        for n in range(1, 31):
            name = f'MonSnowBear{n:02}'
            if name not in selected:
                with self.subTest(name=name), self.assertRaisesRegex(ValueError, name):
                    adapter.load_f3_maps((name,))

    def test_cli_requires_cache_and_rejects_each_other_adapter(self):
        for args in (
            ('--game-dir', 'game', '--prebuilt-out', 'prebuilt'),
            ('--check-prebuilt', 'prebuilt'),
            *(('--game-dir', 'game', '--cache', 'cache', flag, 'OtherMap')
              for flag in ('--f1-map', '--f1e-map', '--f2-map')),
        ):
            with self.subTest(args=args):
                code, err = run_cli(*args, '--f3-map', 'Tutorial01')
                self.assertEqual(code, 2, err)
                self.assertIn('--f3-map', err)

    def test_census_and_catalog_pins_gate_f3(self):
        with tempfile.TemporaryDirectory() as tmp:
            evidence = Path(tmp)
            catalog = (EVIDENCE / CATALOG_NAME).read_bytes()
            census = json.loads((EVIDENCE / CENSUS_NAME).read_bytes())
            catalog_path, census_path = evidence / CATALOG_NAME, evidence / CENSUS_NAME
            with patch.object(adapter, '_F2_EVIDENCE', evidence):
                census_path.write_bytes(adapter.encode(census))
                catalog_path.write_bytes(catalog + b'\n')
                with self.assertRaisesRegex(ValueError, 'không khớp KR 00967'):
                    adapter.load_f3_maps(('Tutorial01',))
                catalog_path.write_bytes(catalog)
                for name in ('Tutorial01', 'MonSnowBear07'):
                    census['maps'][name]['terrain_constraints'] = ['rectangles']
                census_path.write_bytes(adapter.encode(census))
                with self.assertRaisesRegex(ValueError, 'không khớp KR 00967'):
                    adapter.load_f3_maps(('Tutorial01',))
                del census['body_sha256']
                census_path.write_bytes(adapter.encode(dict(census, body_sha256=adapter.sha(adapter.encode(census)[:-1]))))
                for name in ('Tutorial01', 'MonSnowBear07'):
                    with self.subTest(name=name), self.assertRaisesRegex(ValueError, 'Metadata map F3'):
                        adapter.load_f3_maps((name,))

    def test_f3_geometry_rejects_unmapped_tiles_and_blocked_spawns(self):
        ground = (0, 0, 0, 0, 0, 0, 1, 1, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0)
        box = (2, 0, 0, 0, 0, 0, 0, 1) + (0,) * 10
        tiles = {'Tile42_Ground/Tile00': ground, 'Tile26_Box/Basket': box}
        table = (list(tiles), tiles)

        def read(raw, digest=None):
            evidence = dict(family='Tutorial', payload_sha256=digest or adapter.sha(raw),
                            version=1001, scripted=False)
            levels = Mock()
            levels.read.side_effect = {'Map/Tutorial01': raw}.__getitem__
            return adapter.read_f3_map(levels, 'Tutorial01', 7, evidence, table)

        raw = payload(cells={6: (0, 1, -1)})
        profile, rules = read(raw)
        self.assertEqual((profile['adapter'], profile['f3_family'], profile['map_code']),
                         ('f3-adapter-r1', 'Tutorial', 7))
        kinds = [0] * 195
        kinds[6] = 2
        self.assertEqual(rules, {'terrain': dict(width=15, height=13, kinds=kinds,
                                                 map_sha256=adapter.sha(raw),
                                                 seed_sha256=adapter.sha(bytes(kinds)))})
        for change, error in (
            (dict(cells={6: (0, 2, -1)}), 'Địa hình F3'),
            (dict(cells={77: (0, 1, -1)}), 'ô trống'),
            (dict(spawns=((2, 5), (2, 5))), 'spawn'),
            (dict(width=19), 'Bố cục map F3'),
        ):
            with self.subTest(change=change), self.assertRaisesRegex(ValueError, error):
                read(payload(**change))
        with self.assertRaisesRegex(ValueError, 'không khớp SHA-256'):
            read(raw, '0' * 64)

    def test_unbreakable_box_is_solid_but_dynamic_objects_stay_unmapped(self):
        tornado = (3, 0, 0, 0, 0, 0, *(0x01010101,) * 3, 16777473,
                   0, 1, 0, 1, 1, 5000, 17862522, 13741456)
        tiles = {
            'Tile29_Box/R2_UnbreakableBox': (2, 0, 0, 0, 0, 0, 255, *(0,) * 11),
            'Tile33_Passble/SnowBoom': (
                3, 0, 0, 0, 0, 0, *(0x01010101,) * 4, 1, 1, 0, 0, 0, 0, 20484810, 112516296),
            'Tile33_Passble/Pass_Down': (
                3, 0, 0, 0, 0, 0, *(0x01010101,) * 4, 0, 1, 0, 0, 0, 0, 20484810, 140088616),
            'Tile23_Static/Temple08': (1, 0, 0, 0, 0, 0, 7, 1, 1, 0, 1, 2010089472, *(0,) * 6),
            'TileBonusMap_static/Tower00': (1, 0, 0, 0, 0, 0, 1, 1, 0, 999, *(0,) * 8),
            'Tile31_Passable_MonS2Desert/Tornado_Day_01': tornado,
            'Tile31_Passable_MonS2Desert/Tornado_Night_01': tornado,
        }
        table = (list(tiles), tiles)

        def project(code):
            raw = payload(cells={i: (-1, code if i == 6 else -1, -1) for i in range(195)})
            levels = Mock()
            levels.read.return_value = raw
            evidence = dict(family='MonSnowBear', payload_sha256=adapter.sha(raw), version=1001, scripted=False)
            _, rules = adapter.read_f3_map(levels, 'GeometryProbe', 7, evidence, table)
            return rules['terrain']

        terrain = project(0)
        self.assertEqual(terrain['kinds'][6:8], [1, 0])
        self.assertNotIn('box_hits', terrain)
        for code, key in enumerate(table[0][1:], start=1):
            with self.subTest(tile=key), self.assertRaisesRegex(ValueError, key):
                project(code)

    def test_baseball_and_kart_tiles_do_not_fall_back_to_generic_rules(self):
        def project(role, key, data):
            cell = (0, -1, -1) if role == 'tile' else (-1, 0, -1)
            raw = payload(cells=dict.fromkeys(range(195), (-1, -1, -1)) | {6: cell})
            evidence = dict(family='Baseball', payload_sha256=adapter.sha(raw), version=1001, scripted=False)
            _, rules = adapter.read_f3_map(Mock(read=Mock(return_value=raw)), 'ModeProbe', 7,
                                           evidence, ([key], {key: data}))
            return rules['terrain']

        key = 'Tile70_Static/Static28'
        terrain = project('object', key, BASEBALL_WALL)
        self.assertEqual([i for i, kind in enumerate(terrain['kinds']) if kind], [6, 7, 8, 9])
        self.assertEqual(terrain['kinds'][6:10], [1] * 4)
        for i, word in enumerate(BASEBALL_WALL):
            changed = (*BASEBALL_WALL[:i], word ^ 1, *BASEBALL_WALL[i + 1:])
            with self.subTest(word=i), self.assertRaisesRegex(ValueError, key):
                project('object', key, changed)

        for role, key, data in BLOCKED_BASEBALL_KART:
            if role == 'tile':
                generic = STATIC_GROUND
            elif key.startswith('Tile12_Passable/'):
                generic = (3, 0, 0, 0, 0, 0, *(0x01010101,) * 3, *(0,) * 9)
            else:
                generic = (1, 0, 0, 0, 0, 0, 1, 1, *(0,) * 10)
            for record in (data, generic):
                with self.subTest(tile=key, stock=record == data), self.assertRaisesRegex(ValueError, key):
                    project(role, key, record)

    def test_transparent_tile_allows_rays_but_cannot_host_a_spawn(self):
        key = 'TileTransparent_Passable/Passable01'
        record = (3, 0, 0, 0, 0, 0, 0, 0x01010101, 0x01010101, 0, 1, 1, 0, 112, 0, 0,
                  1980460265, 1980460433)
        profile = dict(name='TileProbe', width=15, height=13, rectangles=[], spawns=[],
                       cells=[(-1, 0, -1)], payload_sha256='')
        table = ([key], {key: record})
        terrain = adapter.f1_terrain_rules(profile, table, f1e=True)['terrain']
        self.assertEqual(terrain['kinds'][0], 0)
        self.assertNotIn('ray_barriers', terrain)
        self.assertNotIn('ray_ports', terrain)
        with self.assertRaisesRegex(ValueError, 'spawn'):
            adapter.f1_terrain_rules(dict(profile, spawns=[(0, 0)]), table, f1e=True)
        # A record which would otherwise match the generic all-open rule
        # cannot pass under this name.
        changed = (*record[:6], *(0x01010101,) * 3, 0, 0, 0, *record[12:])
        with self.assertRaisesRegex(ValueError, key):
            adapter.f1_terrain_rules(profile, ([key], {key: changed}), f1e=True)


@unittest.skipUnless(all((GAME / name).is_file() for name in adapter.PACK_SHA256), 'Thiếu pack KR 00967')
class StockPackTests(unittest.TestCase):
    def test_cli_exports_all_allowlisted_f3_maps_in_batches(self):
        census = json.loads((EVIDENCE / CENSUS_NAME).read_bytes())
        families = {name: row['family'] for row in census['families']['F3']
                    if row['priority'] == 'terrain_ready' and row['family'] != 'Zombie'
                    for name in row['maps']}
        self.assertEqual(len(families), 503)
        self.assertEqual(len(set(families.values())), 105)
        for family, maps in EXPANDED_FAMILIES.items():
            if family != 'Zombie':
                self.assertEqual({name for name, label in families.items() if label == family}, set(maps))
        names = sorted(families)
        digest = hashlib.sha256()
        with tempfile.TemporaryDirectory() as tmp:
            for start in range(0, len(names), 16):
                batch = names[start:start + 16]
                args = ('--game-dir', str(GAME), '--cache', tmp,
                        *(arg for name in batch for arg in ('--f3-map', name)))
                code, result = run_cli(*args)
                self.assertEqual(code, 0, result)
                self.assertFalse(result['reused'])
                cache = Path(result['cache'])
                for path in sorted(cache.iterdir()):
                    digest.update(f'{path.name}:{adapter.sha(path.read_bytes())}\n'.encode())
                if start == 0:
                    self.assertEqual(run_cli(*args), (0, dict(result, reused=True)))
                for name in batch:
                    with self.subTest(name=name):
                        profile = json.loads((cache / f'{name}.json').read_bytes())
                        rules = json.loads((cache / f'{name}.rules.json').read_bytes())
                        row = census['maps'][name]
                        self.assertEqual((profile['name'], profile['adapter'], profile['f3_family']),
                                         (name, 'f3-adapter-r1', families[name]))
                        self.assertEqual((profile['width'], profile['height'], len(profile['cells'])), (15, 13, 195))
                        self.assertEqual((profile['payload_sha256'], profile['version'], profile['scripted']),
                                         (row['payload_sha256'], row['version'], row['scripted']))
                        self.assertLessEqual(rules.keys(), {'terrain', 'bomb_cancellation'})
                        terrain = rules['terrain']
                        kinds = terrain['kinds']
                        self.assertEqual((terrain['width'], terrain['height'], len(kinds)), (15, 13, 195))
                        self.assertEqual(terrain['map_sha256'], row['payload_sha256'])
                        self.assertEqual(terrain['seed_sha256'], adapter.sha(bytes(kinds)))
                        self.assertNotIn('maximum_power', terrain)
                        if name == 'SnowyFields08':
                            self.assertEqual(terrain['ray_barriers'][81:84], [1, 0, 1])
                        spawns = profile['spawns']
                        self.assertEqual(len(set(map(tuple, spawns))), len(spawns))
                        for x, y in spawns:
                            self.assertTrue(0 <= x < 15 and 0 <= y < 13)
                            self.assertEqual(kinds[y * 15 + x], 0)
        # Toàn bộ 503 map F3 đã xuất trên pack gốc trước p215 (43fe285).
        self.assertEqual(digest.hexdigest(), '8943e363fb0be9063ed601391d81cbed36535bf18f86dc79616372ef10e1cae2')

    def test_cli_exports_static_mon_snow_bear_maps(self):
        census = json.loads((EVIDENCE / CENSUS_NAME).read_bytes())
        with tempfile.TemporaryDirectory() as tmp:
            code, result = run_cli('--game-dir', str(GAME), '--cache', tmp,
                                   *(part for name in SNOW_BEAR_STATIC_MAPS for part in ('--f3-map', name)))
            self.assertEqual(code, 0, result)
            cache = Path(result['cache'])
            for name in SNOW_BEAR_STATIC_MAPS:
                with self.subTest(name=name):
                    profile = json.loads((cache / f'{name}.json').read_bytes())
                    rules = json.loads((cache / f'{name}.rules.json').read_bytes())
                    self.assertEqual((profile['f3_family'], profile['width'], profile['height']),
                                     ('MonSnowBear', 15, 13))
                    self.assertEqual(profile['payload_sha256'], census['maps'][name]['payload_sha256'])
                    self.assertTrue(profile['scripted'])
                    self.assertEqual(set(rules), {'terrain'})
                    self.assertEqual(len(rules['terrain']['kinds']), 195)
                    self.assertTrue(all(rules['terrain']['kinds'][y * 15 + x] == 0
                                        for x, y in profile['spawns']))

    def test_legacy_exports_match_baseline_bytes(self):
        # Hash thành viên theo lô đã sắp xếp, gồm cả tables và manifest,
        # từ exporter ef285c1 trên pack KR 00967 gốc. F1-a gồm cả 8 map thêm sau mốc 206.
        f1a = sorted(name for name, row in inventory(GAME).items() if row['status'] == 'pending')
        f1e = sorted(json.loads((EVIDENCE / 'f1e-scripts-extracted.json').read_bytes())['maps'])
        f2 = [name for family, names in adapter._F2_FAMILIES.items()
              if family not in EXPANDED_FAMILIES for name in names]
        f2 = sorted([*f2, *(f'SnowyFields{n:02}' for n in (4, 5, 6, 17, 18)), 'Zombie04'])
        cases = (
            ('f1', f1a, 214, 'cf3fa3b3a89bd64c9a180d0a15b60d41c3bdb97a82809e03c3825fb4cdf36e1a'),
            ('f1e', f1e, 96, 'f926dd350359fa53b777857d155d04bfa0887763407096609033d92553ca4a70'),
            ('f2', f2, 45, 'db5545dadb32ed62b5bbaba98f9745a6bbb7f9997463239ef71a8a2f6d64f147'),
        )
        with (tempfile.TemporaryDirectory() as tmp,
              patch.object(adapter, 'load_f3_maps', side_effect=AssertionError('F3 phải bật riêng'))):
            for flag, names, count, expected in cases:
                with self.subTest(flag=flag):
                    self.assertEqual(len(names), count)
                    digest = hashlib.sha256()
                    for start in range(0, count, 16):
                        cache, reused = adapter.export(GAME, Path(tmp), maps=names[start:start + 16], **{flag: True})
                        self.assertFalse(reused)
                        for path in sorted(cache.iterdir()):
                            digest.update(f'{path.name}:{adapter.sha(path.read_bytes())}\n'.encode())
                    self.assertEqual(digest.hexdigest(), expected)

        census = json.loads((EVIDENCE / CENSUS_NAME).read_bytes())
        legacy_expanded = {*EXPANDED_FAMILIES, 'CAhero'}
        old_f3 = sorted(name for row in census['families']['F3']
                        if row['priority'] == 'terrain_ready' and row['family'] not in legacy_expanded
                        for name in row['maps'])
        with tempfile.TemporaryDirectory() as tmp:
            digest = hashlib.sha256()
            for start in range(0, len(old_f3), 16):
                cache, reused = adapter.export(GAME, Path(tmp), maps=old_f3[start:start + 16], f3=True)
                self.assertFalse(reused)
                for path in sorted(cache.iterdir()):
                    digest.update(f'{path.name}:{adapter.sha(path.read_bytes())}\n'.encode())
            self.assertEqual((len(old_f3), digest.hexdigest()),
                             (398, '3d5fc41d0cc6cbfebb518abc7a5846560c7b6be07169e0b3ef906b374525cc20'))


if __name__ == '__main__':
    unittest.main()
