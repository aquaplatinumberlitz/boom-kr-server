"""Kiểm adapter pack F2 bằng fixture tổng hợp và catalog trong repo."""
from contextlib import redirect_stderr, redirect_stdout
from functools import partial
import io
import json
from pathlib import Path
import struct
import sys
import tempfile
import unittest
from unittest.mock import patch

TOOLS = Path(__file__).resolve().parent
sys.path.insert(0, str(TOOLS))

import kr_pack_export as adapter
from kr_f2_f3_tile_census import default_game_dir
from tools.test_f1e_adapter import payload as f1e_payload
from tools.test_kr_pack_install import fake_game, fake_pair, string

GAME = default_game_dir()
EXPANDED_FAMILIES = {
    'Hamster': tuple(f'Hamster{n:02}' for n in range(1, 9)),
    'SnowyFields': tuple(f'SnowyFields{n:02}' for n in range(1, 19)),
    'Zombie': tuple(f'Zombie{n:02}' for n in range(1, 7)),
    'PopcadeA': tuple(f'PopcadeA{n:02}' for n in range(1, 6)),
    'PopcadeB': tuple(f'PopcadeB{n:02}' for n in range(1, 6)),
    'PopcadeC': tuple(f'PopcadeC{n:02}' for n in range(1, 6)),
    'Gumiho': tuple(f'Gumiho{n:02}' for n in range(1, 12)),
}


def payload(*, width=15, height=13, cells=None, spawns=((2, 5), (2, 7), (12, 5), (12, 7)), extra=b''):
    cells = {} if cells is None else cells
    header = struct.pack('<20i', 1001, 2, 2, width, height, len(spawns), *([0] * 14))
    return (header + b''.join(struct.pack('<2i', *p) for p in spawns) + bytes(4)
            + b''.join(struct.pack('<3i', *cells.get(i, (0, -1, -1))) for i in range(width * height))
            + struct.pack('<6i', 0, 100, 0, 0, 0, 0) + extra)


class F2SelectionTests(unittest.TestCase):
    def test_admits_every_map_of_the_three_terrain_ready_families(self):
        for family, names in EXPANDED_FAMILIES.items():
            with self.subTest(family=family):
                for start in range(0, len(names), 16):
                    batch = names[start:start + 16]
                    selected = adapter.load_f2_maps(batch)
                    self.assertEqual({name: row['family'] for name, row in selected.items()},
                                     dict.fromkeys(batch, family))


@unittest.skipUnless(all((GAME / name).is_file() for name in adapter.PACK_SHA256), 'Thiếu pack KR 00967')
class F2StockPackTests(unittest.TestCase):
    def test_exports_expanded_families_and_projects_footprints_and_ray_ports(self):
        with tempfile.TemporaryDirectory() as tmp:
            terrains = {}
            for family, names in EXPANDED_FAMILIES.items():
                for start in range(0, len(names), 16):
                    batch = names[start:start + 16]
                    cache, reused = adapter.export(GAME, Path(tmp), maps=batch, f2=True)
                    self.assertFalse(reused)
                    for name in batch:
                        with self.subTest(name=name):
                            profile = json.loads((cache / f'{name}.json').read_bytes())
                            self.assertEqual((profile['f2_family'], profile['width'], profile['height']),
                                             (family, 15, 13))
                            rules = json.loads((cache / f'{name}.rules.json').read_bytes())
                            self.assertEqual(set(rules), {'terrain'})
                            terrains[name] = rules['terrain']

        # Object01 at (row 6, column 6) covers three columns and two rows.
        hamster = terrains['Hamster02']['kinds']
        for row in (5, 6):
            self.assertEqual(hamster[row * 15 + 6:row * 15 + 9], [1, 1, 1])
        ice_house = terrains['SnowyFields08']
        self.assertEqual(ice_house['kinds'][81:84], [0, 0, 0])
        self.assertEqual(ice_house['ray_ports'][81:84], [0, 0b1111, 0])
        zombie = terrains['Zombie02']
        self.assertEqual((zombie['kinds'][22], zombie['ray_ports'][22]), (0, 0b1010))


class F2AdapterTests(unittest.TestCase):
    def setUp(self):
        temp = tempfile.TemporaryDirectory()
        self.addCleanup(temp.cleanup)
        root = Path(temp.name)
        self.game, self.cache, self.evidence = root / 'game', root / 'cache', root / 'evidence'
        fake_game(self.game)
        self.evidence.mkdir()
        evidence = TOOLS.parent / adapter._F2_EVIDENCE
        for name in ('f2-mode-catalog.json', 'f2-f3-tile-grid-census.json'):
            (self.evidence / name).write_bytes((evidence / name).read_bytes())
        self.census_path = self.evidence / 'f2-f3-tile-grid-census.json'
        self.census = json.loads(self.census_path.read_bytes())
        self.census.pop('body_sha256')
        evidence_patch = patch.object(adapter, '_F2_EVIDENCE', self.evidence)
        evidence_patch.start()
        self.addCleanup(evidence_patch.stop)

        ground = (0, 0, 0, 0, 0, 0, 1, 1, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0)
        static = (1, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0)
        tiles = {
            'Tile42_Ground/Tile00': ground,
            'Tile42_Static/Block01': static,
            'Tile26_Box/Basket': (2, 0, 0, 0, 0, 0, 0, 1) + (0,) * 10,
            'Tile42_Static/UnknownKind': (11, *static[1:]),
        }
        names = [*tiles, 'MissingTileInfo']
        xml = ('<Tiles>' + ''.join(f'<Tile Name="{name}" Data="{struct.pack("<18i", *data).hex()}"/>'
                                   for name, data in tiles.items()) + '</Tiles>').encode()
        fake_pair(self.game, 'fx', {
            'ObjectTable/ObjectTable': struct.pack('<I', len(names))
            + b''.join(string(name) + bytes(4) for name in names),
            'TileInfo/TileInfoList': struct.pack('<I', len(xml)) + xml,
        })
        self.raw = payload(cells={6: (0, 1, -1), 17: (0, 2, -1)})
        self.write_maps(self.raw)

    def write_census(self):
        digest = adapter.sha(adapter.encode(self.census)[:-1])
        self.census_path.write_bytes(adapter.encode(dict(self.census, body_sha256=digest)))

    def write_maps(self, raw):
        fake_pair(self.game, 'FxMap', {
            'Map/SyntheticOne': self.raw,
            'Map/Aqua01': f1e_payload('<StageClearScript/>', obj=1, spawns=[(x, 1) for x in range(8)]),
            'Map/CAPong01': raw,
            'Map/CAPong02': self.raw,
        })
        self.pins = {p.name: adapter.sha(p.read_bytes()) for p in self.game.iterdir()}
        self.census['maps']['CAPong01']['payload_sha256'] = adapter.sha(raw)
        self.census['maps']['CAPong02']['payload_sha256'] = adapter.sha(self.raw)
        self.write_census()

    def invoke(self, *args):
        out, err = io.StringIO(), io.StringIO()
        argv = ['kr_pack_export.py', '--game-dir', str(self.game), *args]
        with (patch.object(sys, 'argv', argv), redirect_stdout(out), redirect_stderr(err),
              patch.object(adapter, 'export', partial(adapter.export, pins=self.pins))):
            try:
                adapter.main()
            except SystemExit as exc:
                return exc.code, err.getvalue()
        return 0, json.loads(out.getvalue())

    def run_f2(self, *names):
        return self.invoke('--cache', str(self.cache), *(part for name in names for part in ('--f2-map', name)))

    def test_cli_exports_two_maps_and_reuses_verified_cache(self):
        error = AssertionError('F2 không gọi adapter F1')
        with (patch.object(adapter, 'read_map', side_effect=error),
              patch.object(adapter, 'f1_rules', side_effect=error),
              patch.object(adapter, 'terrain_for', side_effect=error)):
            code, result = self.run_f2('CAPong02', 'CAPong01')
            self.assertEqual(code, 0, result)
            self.assertFalse(result['reused'])
            cache = Path(result['cache'])
            self.assertEqual(self.run_f2('CAPong02', 'CAPong01'), (0, dict(result, reused=True)))
        profile = json.loads((cache / 'CAPong01.json').read_bytes())
        self.assertEqual((profile['width'], profile['height']), (15, 13))
        self.assertEqual(profile['adapter'], 'f2-adapter-r1')
        self.assertEqual((profile['name'], profile['map_code'], profile['f2_family']), ('CAPong01', 2, 'CAPong'))
        self.assertEqual(profile['spawns'], [[2, 5], [2, 7], [12, 5], [12, 7]])
        self.assertEqual(profile['cell_fields'], ['tile', 'object', 'overlay'])
        self.assertEqual(len(profile['cells']), 195)
        self.assertEqual(profile['cells'][17], [0, 2, -1])
        rules = json.loads((cache / 'CAPong01.rules.json').read_bytes())
        expected = [0] * 195
        expected[6], expected[17] = 1, 2
        self.assertEqual(rules['terrain']['kinds'], expected)
        self.assertEqual(rules['terrain']['map_sha256'], profile['payload_sha256'])
        self.assertNotIn('floor', rules)

    def test_cli_rejects_selection_outside_f2_scope(self):
        cases = (
            (('CAPong01', 'CAPong01'), '1–16'),
            (tuple(f'Map{i:02}' for i in range(17)), '1–16'),
            (('NoSuchMap',), 'NoSuchMap'),
            (('Factory07',), 'Factory07'),
            (('Aqua01',), 'Aqua01'),
            (('Bula00',), 'chưa được giải mã đầy đủ'),
            (('Baseball01',), 'tile chưa ánh xạ'),
        )
        for names, message in cases:
            with self.subTest(names=names):
                code, err = self.run_f2(*names)
                self.assertEqual(code, 1, err)
                self.assertIn(message, err)
                self.assertFalse(self.cache.exists())

    def test_cli_exports_each_new_terrain_ready_family(self):
        families = {
            'Clover01': 'Clover', 'Rabbit01': 'Rabbit', 'RescueET01': 'RescueET',
            'Wriggle01': 'Wriggle', 'Zzolman01': 'Zzolman',
            **{f'Coloring{n:02}': 'Coloring' for n in range(1, 6)},
            'SuperFighter16': 'SuperFighter', 'SuperFighter19': 'SuperFighter',
        }

        def selected(names):
            return {name: dict(family=families[name], payload_sha256=adapter.sha(self.raw),
                               version=1001, scripted=False) for name in names}

        fake_pair(self.game, 'FxMap', {
            'Map/SyntheticOne': self.raw,
            'Map/Aqua01': f1e_payload('<StageClearScript/>', obj=1, spawns=[(x, 1) for x in range(8)]),
            'Map/CAPong01': self.raw,
            'Map/CAPong02': self.raw,
            **{f'Map/{name}': self.raw for name in families},
        })
        self.pins = {p.name: adapter.sha(p.read_bytes()) for p in self.game.iterdir()}
        with patch.object(adapter, 'load_f2_maps', side_effect=selected):
            code, result = self.run_f2(*families)
        self.assertEqual(code, 0, result)
        cache = Path(result['cache'])
        for name, family in families.items():
            profile = json.loads((cache / f'{name}.json').read_bytes())
            self.assertEqual((profile['adapter'], profile['f2_family']), ('f2-adapter-r1', family))
            self.assertTrue((cache / f'{name}.rules.json').is_file())

    def test_cli_rejects_other_adapters_and_prebuilt_with_f2(self):
        for args in (('--cache', str(self.cache), '--f1-map', 'Factory07'),
                     ('--cache', str(self.cache), '--f1e-map', 'Aqua01'),
                     ('--prebuilt-out', str(self.cache))):
            with self.subTest(args=args):
                code, err = self.invoke(*args, '--f2-map', 'CAPong01')
                self.assertEqual(code, 2, err)
                self.assertIn('--f2-map', err)

    def test_export_rejects_unmapped_tiles_in_every_layer(self):
        for cell in ((3, -1, -1), (0, 3, -1), (0, -1, 1), (0, 4, -1), (5, -1, -1), (-2, -1, -1)):
            with self.subTest(cell=cell):
                self.write_maps(payload(cells={6: cell}))
                code, err = self.run_f2('CAPong02', 'CAPong01')
                self.assertEqual(code, 1, err)
                self.assertIn('Tile F2 chưa ánh xạ: CAPong01 (6,0)', err)
                self.assertEqual(list(self.cache.iterdir()), [])

    def test_export_rejects_unsupported_payloads_and_spawns(self):
        for raw, message in (
            (payload(width=19), '15x13'),
            (self.raw + b'\x00', 'không đọc hết payload'),
            (payload(spawns=((15, 0), (1, 1))), 'spawn'),
            (payload(spawns=((1, 1), (1, 1))), 'spawn'),
            (payload(cells={77: (0, 1, -1)}), 'ô trống'),
        ):
            with self.subTest(message=message):
                self.write_maps(raw)
                code, err = self.run_f2('CAPong01')
                self.assertEqual(code, 1, err)
                self.assertIn(message, err)

    def test_payload_pin_and_census_checks_also_apply_to_cache_reuse(self):
        self.assertEqual(self.run_f2('CAPong01')[0], 0)
        row = self.census['maps']['CAPong01']
        row['payload_sha256'] = '0' * 64
        self.write_census()
        code, err = self.run_f2('CAPong01')
        self.assertEqual(code, 1, err)
        self.assertIn('không khớp SHA-256', err)
        row['grid'] = [19, 17]
        self.write_census()
        code, err = self.run_f2('CAPong01')
        self.assertEqual(code, 1, err)
        self.assertIn('15x13', err)

    def test_legacy_exports_match_baseline_without_loading_f2_evidence(self):
        # Hash các member (gồm cả manifest) của fixture tổng hợp tại df0859e.
        cases = (
            ('--f1-map', 'SyntheticOne',
             'a2226c1b821b0cc53b9e0495849424ab1ab1bae9a50129db0c3224eb8ab98eee'),
            ('--f1e-map', 'Aqua01',
             '2e4bfce87c22a702c8ec870b32de41aea157e8bf761c857fd17af99e81e9806e'),
        )
        with patch.object(adapter, 'load_f2_maps', side_effect=AssertionError('F2 cần được bật riêng')):
            for flag, name, expected in cases:
                with self.subTest(flag=flag):
                    code, result = self.invoke('--cache', str(self.cache), flag, name)
                    self.assertEqual(code, 0, result)
                    digests = {p.name: adapter.sha(p.read_bytes()) for p in Path(result['cache']).iterdir()}
                    self.assertEqual(adapter.sha(adapter.encode(digests)), expected)


if __name__ == '__main__':
    unittest.main()
