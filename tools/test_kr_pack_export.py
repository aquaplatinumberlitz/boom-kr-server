"""Kiểm đuôi Bula theo bản ghi P228 và ranh giới xuất profile phòng."""

import json
from pathlib import Path
import struct
import sys
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parent))
import kr_pack_export as pack


class BulaTailTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        evidence = Path(__file__).resolve().parents[1] / pack.BULA_TAIL_EVIDENCE
        cls.rows = json.loads(evidence.read_bytes())['bula_tail']['maps']

    def fixture(self, name):
        # Phần trước đuôi là fixture; phần đuôi lấy từ byte pack đã lưu ở P228.
        header = struct.pack('<20i', 1000, 3, 2, 15, 13, 0, *([0] * 14))
        prefix = header + struct.pack('<i', 0) + struct.pack('<ii', -1, -1) * 195
        prefix += struct.pack('<iii', 0, 100, 0)
        expected = self.rows[name]
        raw = prefix + bytes.fromhex(expected['tail_hex'])
        return raw, dict(expected, payload_sha256=pack.sha(raw), payload_bytes=len(raw), tail_offset=len(prefix))

    def test_exact_tails_preserve_group_points_record_fields_and_eof(self):
        self.assertEqual(set(self.rows), pack.BULA_MAPS)
        for name in self.rows:
            with self.subTest(name=name):
                raw, expected = self.fixture(name)
                with patch.object(pack, '_bula_tail_reference', return_value={name: expected}):
                    profile, report = pack.validate_bula_tail(raw, name)
                self.assertEqual((profile['version'], profile['width'], profile['height']), (1000, 15, 13))
                self.assertEqual(report, expected)

    def test_malformed_tails_fail_closed(self):
        raw, expected = self.fixture('Bula00')
        offset = expected['tail_offset']
        name_at = offset + 4 + 4 + 4 * 8 + 4 + 4
        corruptions = {
            'short header': raw[:12],
            'truncated points': raw[:offset + 9],
            'truncated record': raw[:name_at + 11],
            'truncated script length': raw[:-1],
            'extra byte': raw + b'\0',
            'negative group count': raw[:offset] + struct.pack('<i', -1) + raw[offset + 4:],
            'excess point count': raw[:offset + 4] + struct.pack('<i', 2**31 - 1) + raw[offset + 8:],
            'negative record count': raw[:name_at - 8] + struct.pack('<i', -1) + raw[name_at - 4:],
            'zero name length': raw[:name_at - 4] + struct.pack('<i', 0) + raw[name_at:],
            'invalid CP949': raw[:name_at] + b'\xff' + raw[name_at + 1:],
            'interior NUL': raw[:name_at + 1] + b'\0' + raw[name_at + 2:],
            'unsigned script length past EOF': raw[:-4] + struct.pack('<I', 2**32 - 1),
        }
        with patch.object(pack, '_bula_tail_reference', return_value={'Bula00': expected}):
            for label, data in corruptions.items():
                with self.subTest(label=label), self.assertRaisesRegex(ValueError, 'Cấu trúc đuôi Bula'):
                    pack.validate_bula_tail(data, 'Bula00')

    def test_well_formed_tail_or_prefix_with_different_bytes_is_rejected(self):
        raw, expected = self.fixture('Bula00')
        with patch.object(pack, '_bula_tail_reference', return_value={'Bula00': expected}):
            for data in (raw[:-8] + struct.pack('<i', 1) + raw[-4:],
                         raw[:4] + struct.pack('<i', 4) + raw[8:],
                         raw[:-4] + struct.pack('<I', 1) + b'x'):
                with self.assertRaisesRegex(ValueError, 'không khớp bằng chứng P228'):
                    pack.validate_bula_tail(data, 'Bula00')
            with self.assertRaisesRegex(ValueError, 'ngoài phạm vi'):
                pack.validate_bula_tail(raw, 'Bula05')

    def test_room_profile_readers_reject_bula_before_reading_pack(self):
        with self.assertRaisesRegex(ValueError, 'chỉ dùng để kiểm tra tĩnh'):
            pack.read_map(None, 'Bula00', 0, f1=False)
        for read in (pack.read_f2_map, pack.read_f3_map):
            with self.subTest(reader=read.__name__), self.assertRaisesRegex(ValueError, 'chỉ dùng để kiểm tra tĩnh'):
                read(None, 'Bula00', 0, None, None)


class BaseballKartTerrainTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        evidence = Path(__file__).resolve().parents[1] / 'docs/evidence/content/f2-baseball-kart-tiles-re.json'
        cls.families = json.loads(evidence.read_bytes())['classification']

    def project(self, key, data, role):
        profile = dict(name='TileProbe', width=15, height=13, payload_sha256='', spawns=[],
                       cell_fields=['tile', 'object', 'overlay'],
                       cells=[[-1, -1, -1] for _ in range(195)])
        profile['cells'][16][profile['cell_fields'].index(role)] = 0
        return pack.f2_terrain_rules(profile, ([key], {key: data}))['terrain']

    def test_verified_records_project_only_their_static_footprint(self):
        for family in self.families.values():
            for key, row in family['static_verified'].items():
                data = tuple(row['tileinfo_i32'])
                kind = data[0]
                role = 'tile' if kind == 0 else 'object'
                expected = [0] * 195
                if kind == 1:
                    for y in range(2 - data[7], 2):
                        for x in range(1, 1 + data[6]):
                            expected[y * 15 + x] = 1
                elif kind == 2:
                    expected[16] = 2
                with self.subTest(key=key):
                    result = self.project(key, data, role)
                    self.assertEqual(result['kinds'], expected)
                    self.assertNotIn('ray_ports', result)
                    for changed, layer in ((data, 'overlay'), ((*data[:-1], data[-1] ^ 1), role)):
                        with self.assertRaisesRegex(ValueError, 'Tile F2 chưa ánh xạ'):
                            self.project(key, changed, layer)

    def test_mode_tiles_stay_rejected_even_if_their_tuple_looks_static(self):
        for family in self.families.values():
            for key, row in family['fail_closed_dynamic_or_special_physics'].items():
                data = tuple(row['tileinfo_i32'])
                role = 'tile' if data[0] == 0 else 'object'
                static = (family['static_verified']['Tile05_Ground/Road00']['tileinfo_i32']
                          if key.startswith('Tile70_Ground/') else
                          self.families['Kart']['static_verified'][
                              'Tile12_Ground/Sand00' if role == 'tile' else 'Tile12_Static/Signal00'
                          ]['tileinfo_i32'])
                with self.subTest(key=key):
                    for record in (data, tuple(static)):
                        with self.assertRaisesRegex(ValueError, 'Tile F2 chưa ánh xạ'):
                            self.project(key, record, role)

    def test_baseball_and_kart_stay_outside_room_export(self):
        for name in ('Baseball01', 'Kart02'):
            with self.subTest(name=name), self.assertRaisesRegex(ValueError, 'tile chưa ánh xạ'):
                pack.load_f2_maps([name])


if __name__ == '__main__':
    unittest.main()
