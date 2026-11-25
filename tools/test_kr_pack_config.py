"""Kiểm cấu hình pack và giải mã lại fixture floor582 đã xuất."""
import copy
import json
from pathlib import Path
import struct
import tempfile
import unittest

from kr_pack_config import configure, enable_floor_policies, floor582
from kr_pack_export import sha
from rust_map_profile import blob, raw_blob
from server import probe_codec as codec
from tools.test_kr_pack_install import HEADER, MAPS, fake_recipe


def read_floor(body):
    raw, end, _ = blob(body)
    if end != len(body):
        raise ValueError('Dữ liệu thừa sau blob floor')
    groups, at = [], 0
    for _ in range(3):
        count = struct.unpack_from('<H', raw, at)[0]
        at += 2
        end = at + count * 10
        groups.append(list(struct.iter_unpack('<IBBI', raw[at:end])))
        at = end
    if at != len(raw):
        raise ValueError('Dữ liệu thừa hoặc thiếu trong floor')
    return groups


class FullFloorTests(unittest.TestCase):
    def test_bigbattle_bounds_and_wire_rows(self):
        floor = dict(fixed=[[16, 18, 331]], hidden=[[15, 17, 150]])
        self.assertEqual(read_floor(floor582(floor, width=19, height=17, scope='f1-full')),
                         [[(0, 16, 18, 331)], [(1, 15, 17, 150)], []])
        for row, column in ((13, 14), (12, 15)):
            old_floor = dict(fixed=[[row, column, 1]], hidden=[])
            for options in ({}, {'scope': 'f1-full'}, {'width': 19, 'height': 17}):
                with self.subTest(row=row, column=column, options=options):
                    with self.assertRaises(ValueError):
                        floor582(old_floor, **options)
        for row, column in ((17, 18), (16, 19), (-1, 0), (0, -1)):
            with self.subTest(row=row, column=column):
                with self.assertRaises(ValueError):
                    floor582(dict(fixed=[], hidden=[[row, column, 149]]),
                             width=19, height=17, scope='f1-full')

    def test_full_floor_keeps_vehicles_ships_and_needle_visible_only(self):
        for kind in (40, 44, 45, 66, 331, 145):
            with self.subTest(kind=kind):
                self.assertEqual(read_floor(floor582(dict(fixed=[[1, 2, kind]], hidden=[]),
                                                     scope='f1-full'))[0], [(0, 1, 2, kind)])
                with self.assertRaises(ValueError):
                    floor582(dict(fixed=[], hidden=[[1, 2, kind]]), scope='f1-full')
        for kind in (6, 39, 73, 153, 231, 332):
            with self.subTest(kind=kind), self.assertRaises(ValueError):
                floor582(dict(fixed=[[1, 2, kind]], hidden=[]), scope='f1-full')
        with self.assertRaises(ValueError):
            floor582(dict(fixed=[[1, 2, 145], [1, 3, 145]], hidden=[]), scope='f1-full')

    def test_minimal_policy_for_every_admitted_kind(self):
        cases = (
            ('basic', (0, 1, 2, 5)),
            ('shoe_punch', (3, 4)),
            ('f1b_extended', (67, 68, 69, 70, 71, 72, 77, 143, 146, 224, 225)),
            ('f1b_aux', (148, 226, 227, 228, 229, 230)),
            ('f1b_remote_sensor', (151, 152)),
            ('f1b_remote_drill', (149, 150)),
            ('f1d_vehicles', tuple(range(40, 45))),
            ('f1d_all_ships', (*range(45, 67), 331)),
        )
        for policy, admitted in cases:
            for kind in admitted:
                with self.subTest(kind=kind):
                    room = {}
                    enable_floor_policies(room, {0, 5, kind}, scope='f1-full')
                    self.assertEqual(room, {'item_policy': 'report_derived_' + policy})
                    if not policy.startswith('f1d_'):
                        body = floor582(dict(fixed=[[1, 1, kind]], hidden=[[1, 2, kind]]),
                                        scope='f1-full')
                        self.assertEqual(read_floor(body), [[(0, 1, 1, kind)], [(1, 1, 2, kind)], []])

    def test_policy_combinations_and_separate_needle_actions(self):
        room = {'life_actions': 'rescue'}
        enable_floor_policies(room, {3, 44, 143, 145}, scope='f1-full')
        self.assertEqual(room, dict(item_policy='report_derived_f1d_vehicles', life_actions='needle_rescue'))
        room = {}
        enable_floor_policies(room, {3, 143, 145, 148, 149, 152}, scope='f1-full')
        self.assertEqual(room, dict(item_policy='report_derived_f1b_remote_drill', life_actions='needle'))
        room = {}
        enable_floor_policies(room, {145}, scope='f1-full')
        self.assertEqual(room, {'life_actions': 'needle'})
        for kinds in ({40, 149}, {44, 152}, {331, 148}):
            room = {'life_actions': 'rescue'}
            with self.subTest(kinds=kinds), self.assertRaisesRegex(ValueError, 'F1-b.*F1-d'):
                enable_floor_policies(room, kinds | {145}, scope='f1-full')
            self.assertEqual(room, {'life_actions': 'rescue'})
        with self.assertRaisesRegex(ValueError, '153'):
            enable_floor_policies({}, {153}, scope='f1-full')


class ConfigureTests(unittest.TestCase):
    def setUp(self):
        temp = tempfile.TemporaryDirectory()
        self.addCleanup(temp.cleanup)
        root = Path(temp.name)
        self.bundle, self.cache, self.out = root / 'bundle', root / 'cache', root / 'out'
        self.recipe = fake_recipe(self.bundle)
        self.recipe['diagnostic_two_lobby']['fixed_room']['map_choices'] = []
        self.cache.mkdir()
        self.out.mkdir()
        self.stats = {'0': {'maximum_power': 7}, '1': {'maximum_power': 9},
                      '3001': {'maximum_power': 8}, '3002': {'maximum_power': 10}}
        self.write_json('tables.json', {'characters': self.stats})

    def write_json(self, name, value):
        (self.cache / name).write_text(json.dumps(value), encoding='utf-8')

    def write_map(self, name, *, width=15, height=13, code=20, scripted=False,
                  fixed=(), hidden=(), terrain=False):
        self.write_json(name + '.json', dict(name=name, map_code=code, spawns=[[1, 1], [2, 2]],
                                             width=width, height=height, scripted=scripted))
        rules = dict(floor=dict(fixed=fixed, hidden=hidden))
        if terrain:
            rules['terrain'] = dict(width=width, height=height, kinds=[0] * (width * height))
        self.write_json(name + '.rules.json', rules)

    def configure(self, maps, scope, **kwargs):
        config = copy.deepcopy(self.recipe)
        configure(config, self.bundle, self.cache, self.out, maps, scope, **kwargs)
        two = config['diagnostic_two_lobby']
        return two['fixed_room'], two['managed_rooms']

    def read_choice_floor(self, choice):
        pin = choice['packets']['floor']
        frame = Path(pin['fixture']['path']).read_bytes()
        self.assertEqual(sha(frame), pin['fixture']['sha256'])
        return read_floor(codec.frame_body(frame, pin['sequence']))

    def test_legacy_scopes_keep_their_source_floor_and_opt_ins(self):
        two = self.recipe['diagnostic_two_lobby']
        two['managed_rooms']['capacity'] = 4
        packets = two['fixed_room']['packets']
        pin = packets['floor']
        path = self.bundle / 'original-floor'
        body = raw_blob(struct.pack('<HIBBIHH', 1, 0, 2, 3, 1, 0, 0))[0]
        frame = codec.encode(HEADER, 582, body, pin['sequence'])
        path.write_bytes(frame)
        pin['fixture'] = dict(path=str(path), bytes=len(frame), sha256=sha(frame))
        self.write_map(MAPS[0], scripted=True, fixed=[[5, 6, 3], [5, 7, 145]], terrain=True)
        cases = (
            ('g1', 4, True, 'report_derived_basic', [[(0, 2, 3, 1)], [], []]),
            ('f1', 2, False, 'report_derived_basic', [[], [], []]),
            ('f1-a', 2, True, 'report_derived_shoe_punch', [[(0, 5, 6, 3), (1, 5, 7, 145)], [], []]),
        )
        for scope, capacity, terrain, policy, expected_floor in cases:
            with self.subTest(scope=scope):
                room, managed = self.configure(MAPS[:1], scope, allow_class_c=True)
                self.assertEqual(self.read_choice_floor(room), expected_floor)
                self.assertEqual('terrain' in room, terrain)
                self.assertEqual(room['item_policy'], policy)
                self.assertEqual(room.get('life_actions'), 'needle' if scope == 'f1-a' else None)
                self.assertNotIn('allow_f1e_dynamic_scripts', room)
                self.assertNotIn('allow_class_c_characters', room)
                self.assertEqual(managed['capacity'], capacity)
                self.assertEqual(managed['player_round'], dict(characters=[0, 1],
                                                               pack_stats={k: self.stats[k] for k in ('0', '1')}))

    def test_full_config_combines_bigbattle_rules_and_terrain_limited_roster(self):
        self.write_map(MAPS[0], fixed=[[12, 14, 143]])
        self.write_map('BigBattle', width=19, height=17, code=21, scripted=True, terrain=True,
                       fixed=[[16, 18, 149]], hidden=[[16, 17, 152]])
        # Exported stats do not govern Class C admission or terrain filtering.
        self.write_json('tables.json', {'characters': dict(self.stats, **{
            '4000': {'maximum_power': 10}, '4006': {'maximum_power': 1}, '4019': {'maximum_power': 7},
        })})
        room, managed = self.configure((MAPS[0], 'BigBattle'), 'f1-full', unlock_all=True, allow_class_c=True)
        self.assertIs(room['allow_f1e_dynamic_scripts'], True)
        self.assertIs(room['allow_class_c_characters'], True)
        self.assertEqual(room['item_policy'], 'report_derived_f1b_remote_drill')
        first, big = room['map_choices']
        self.assertEqual(self.read_choice_floor(first), [[(0, 12, 14, 143)], [], []])
        self.assertEqual(self.read_choice_floor(big), [[(0, 16, 18, 149)], [(1, 16, 17, 152)], []])
        self.assertNotIn('floor', big)
        self.assertEqual((big['terrain']['width'], big['terrain']['height']), (19, 17))
        self.assertEqual(big['terrain']['fixtures'],
                         {key: big['packets'][key]['fixture']['sha256'] for key in ('map', 'start')})
        ordinary = [0, 1, 3001]
        class_c = sorted(set(range(4000, 4031)) - {4006, 4018, 4019, 4025, 4026, 4029, 4030})
        selected = managed['player_round']
        self.assertEqual(selected['characters'], ordinary + class_c + list(range(5000, 5005)))
        self.assertEqual(selected['pack_stats'], {str(key): self.stats[str(key)] for key in ordinary})

    def test_class_c_is_optional_and_power_ten_is_allowed_without_terrain(self):
        self.write_map(MAPS[0], scripted=1)
        room, managed = self.configure(MAPS[:1], 'f1-full')
        self.assertNotIn('allow_f1e_dynamic_scripts', room)
        self.assertNotIn('allow_class_c_characters', room)
        self.assertEqual(managed['player_round']['characters'], [0, 1])
        room, managed = self.configure(MAPS[:1], 'f1-full', allow_class_c=True)
        self.assertIs(room['allow_class_c_characters'], True)
        class_c = sorted(set(range(4000, 4031)) - {4019, 4025, 4026}) + list(range(5000, 5005))
        self.assertEqual(managed['player_round']['characters'], [0, 1] + class_c)
        self.assertEqual(managed['player_round']['pack_stats'], {k: self.stats[k] for k in ('0', '1')})

    def test_incompatible_map_choices_fail_with_one_room_policy(self):
        self.write_map(MAPS[0], fixed=[[1, 2, 149]])
        self.write_map(MAPS[1], code=21, fixed=[[1, 2, 40]])
        with self.assertRaisesRegex(ValueError, 'F1-b.*F1-d'):
            self.configure(MAPS, 'f1-full')


if __name__ == '__main__':
    unittest.main()
