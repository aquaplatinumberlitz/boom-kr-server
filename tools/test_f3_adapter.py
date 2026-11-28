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
from tools.test_f2_adapter import payload

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / 'docs/evidence/content'
CENSUS_NAME = 'f2-f3-tile-grid-census.json'
CATALOG_NAME = 'f3-pve-catalog.json'
GAME = default_game_dir()


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
                (('Hamster08',), 'Hamster08'),
                (('CAhero07',), 'CAhero07'),
                (('SnowyFields04',), 'SnowyFields04'),
                (('MonS2Fire03',), 'MonS2Fire03'),
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
                census['maps']['Tutorial01']['terrain_constraints'] = ['rectangles']
                census_path.write_bytes(adapter.encode(census))
                with self.assertRaisesRegex(ValueError, 'không khớp KR 00967'):
                    adapter.load_f3_maps(('Tutorial01',))
                del census['body_sha256']
                census_path.write_bytes(adapter.encode(dict(census, body_sha256=adapter.sha(adapter.encode(census)[:-1]))))
                with self.assertRaisesRegex(ValueError, 'Metadata map F3'):
                    adapter.load_f3_maps(('Tutorial01',))

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


@unittest.skipUnless(all((GAME / name).is_file() for name in adapter.PACK_SHA256), 'Thiếu pack KR 00967')
class StockPackTests(unittest.TestCase):
    def test_cli_exports_all_allowlisted_f3_maps_in_batches(self):
        census = json.loads((EVIDENCE / CENSUS_NAME).read_bytes())
        families = {name: row['family'] for row in census['families']['F3']
                    if row['priority'] == 'terrain_ready' for name in row['maps']}
        self.assertEqual(len(families), 398)
        self.assertEqual(len(set(families.values())), 98)
        names = sorted(families)
        with tempfile.TemporaryDirectory() as tmp:
            for start in range(0, len(names), 16):
                batch = names[start:start + 16]
                args = ('--game-dir', str(GAME), '--cache', tmp,
                        *(arg for name in batch for arg in ('--f3-map', name)))
                code, result = run_cli(*args)
                self.assertEqual(code, 0, result)
                self.assertFalse(result['reused'])
                cache = Path(result['cache'])
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
                        spawns = profile['spawns']
                        self.assertEqual(len(set(map(tuple, spawns))), len(spawns))
                        for x, y in spawns:
                            self.assertTrue(0 <= x < 15 and 0 <= y < 13)
                            self.assertEqual(kinds[y * 15 + x], 0)

    def test_legacy_exports_match_pre_f3_bytes(self):
        # Hash thành viên theo lô đã sắp xếp, gồm cả tables và manifest,
        # từ exporter 4f3f1a88 trên pack KR 00967 gốc. F1-a gồm cả 8 map thêm sau mốc 206.
        f1a = sorted(name for name, row in inventory(GAME).items() if row['status'] == 'pending')
        f1e = sorted(json.loads((EVIDENCE / 'f1e-scripts-extracted.json').read_bytes())['maps'])
        f2 = sorted(name for family, names in adapter._F2_FAMILIES.items()
                    if family not in ('Coloring', 'SuperFighter') for name in names)
        cases = (
            ('f1', f1a, 214, 'cf3fa3b3a89bd64c9a180d0a15b60d41c3bdb97a82809e03c3825fb4cdf36e1a'),
            ('f1e', f1e, 96, 'f926dd350359fa53b777857d155d04bfa0887763407096609033d92553ca4a70'),
            ('f2', f2, 38, '14af89ab5ca7384d22463edf61b8f0781217b803a839f8e6eab99bc490a903f8'),
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
        new_families = {'Bonus', 'BonusLargeGourd', 'Bonus_19th', 'Hdn_Alice',
                        'Hdn_BlackLucy', 'Hidden', 'Coloring', 'SuperFighter'}
        old_f3 = sorted(name for row in census['families']['F3']
                        if row['priority'] == 'terrain_ready' and row['family'] not in new_families
                        for name in row['maps'])
        with tempfile.TemporaryDirectory() as tmp:
            digest = hashlib.sha256()
            for start in range(0, len(old_f3), 16):
                cache, reused = adapter.export(GAME, Path(tmp), maps=old_f3[start:start + 16], f3=True)
                self.assertFalse(reused)
                for path in sorted(cache.iterdir()):
                    digest.update(f'{path.name}:{adapter.sha(path.read_bytes())}\n'.encode())
            self.assertEqual((len(old_f3), digest.hexdigest()),
                             (380, '1135320ecbe67bbb0f5ebfcedfefa465017a3bb2fcf592f63351971416eed156'))


if __name__ == '__main__':
    unittest.main()
