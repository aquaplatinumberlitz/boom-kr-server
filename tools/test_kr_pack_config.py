"""Kiểm cấu hình pack và giải mã lại fixture floor582 đã xuất."""
import copy
import json
from pathlib import Path
import struct
import subprocess
import sys
import tempfile
import unittest

TOOLS = Path(__file__).resolve().parent
sys.path.insert(0, str(TOOLS))

from kr_pack_config import configure, enable_floor_policies, floor582
from kr_pack_export import F2_ADAPTER, PACK_SHA256, SCHEMA, cache_identity, load_f2_maps, sha
from rust_map_profile import blob, raw_blob, table_row
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
        self.root = root = Path(temp.name)
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
        return read_floor(self.read_packet(choice['packets']['floor']))

    def read_packet(self, pin):
        frame = Path(pin['fixture']['path']).read_bytes()
        self.assertEqual(sha(frame), pin['fixture']['sha256'])
        self.assertEqual(len(frame), pin['fixture']['bytes'])
        return codec.frame_body(frame, pin['sequence'])

    def prepare_legacy_scopes(self):
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

    def test_legacy_scopes_keep_their_source_floor_and_opt_ins(self):
        self.prepare_legacy_scopes()
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

    def test_all_legacy_config_and_wire_bytes_match_baseline(self):
        self.prepare_legacy_scopes()
        self.write_map(MAPS[1], code=21, hidden=[[4, 5, 1]], terrain=True)
        self.write_map('BigBattle', width=19, height=17, code=22, fixed=[[16, 18, 149]], terrain=True)
        # Hash config in insertion order and every generated file at 6a5a67a.
        cases = (
            ('g1', 'a3be94da358a2726031bf9f1749bc03622d788498e957b05a3a68a95e08b1572'),
            ('f1', '1309ce03050705691af713a7e0b2f77933ae5a48053c2b5479bd5d936a5b363b'),
            ('f1-a', '08ed591eeac2db48d4880288a42078fe84b468fdf7900787ce711f79a103c19e'),
            ('f1-full', '9f4d7b2d5f39d143d5f59a6f2a3980b828918c83a1723e69c7d80be7e4bb952f'),
        )
        for scope, expected in cases:
            with self.subTest(scope=scope):
                out = self.out / scope
                out.mkdir()
                cfg = copy.deepcopy(self.recipe)
                maps = (*MAPS, 'BigBattle') if scope == 'f1-full' else MAPS
                configure(cfg, self.bundle, self.cache, out, maps, scope, unlock_all=True, allow_class_c=True)
                encoded = json.dumps(cfg, ensure_ascii=False, separators=(',', ':'))
                data = encoded.replace(json.dumps(str(self.root))[1:-1], '<temp>').encode()
                digests = {'config.json': sha(data), **{p.name: sha(p.read_bytes()) for p in out.iterdir()}}
                self.assertEqual(sha(json.dumps(digests, sort_keys=True).encode()), expected)

    def write_f2_cache(self, *names):
        evidence = load_f2_maps(names)
        for i, (name, row) in enumerate(evidence.items()):
            profile = dict(name=name, map_code=1143 if name == 'Zombie04' else 40 + i,
                           adapter=F2_ADAPTER, f2_family=row['family'],
                           payload_sha256=row['payload_sha256'], version=row['version'],
                           scripted=row['scripted'], width=15, height=13, rectangles=[],
                           spawns=[[x, 1] for x in range(8 if name == 'Zombie04' else 4)],
                           drops=[['Item/ItemFire', 100]], placements=[])
            kinds = [0] * 195
            kinds[30 + i] = 2
            terrain = dict(width=15, height=13, kinds=kinds, map_sha256=row['payload_sha256'],
                           seed_sha256=sha(bytes(kinds)), ray_ports=[10] + [15] * 194)
            self.write_json(name + '.json', profile)
            self.write_json(name + '.rules.json', {'terrain': terrain})
        self.pin_f2_cache(names, evidence)

    def pin_f2_cache(self, names, evidence=None):
        if evidence is None:
            evidence = load_f2_maps(names)
        members = ['tables.json', *(name + ext for name in names for ext in ('.json', '.rules.json'))]
        self.write_json('manifest.json', dict(
            schema=SCHEMA, identity=cache_identity(PACK_SHA256, names, True, f2_maps=evidence),
            packs=PACK_SHA256, files={name: sha((self.cache / name).read_bytes()) for name in members}))

    def test_f2_static_maps_bind_their_terrain_and_keep_an_empty_floor(self):
        names = ('CAPong02', 'CAPong01')
        self.write_f2_cache(*names)
        recipe_room = self.recipe['diagnostic_two_lobby']['fixed_room']
        operator_policy = {'runtime_overrides': {'free': 'timebomb', 'manner': 'timebomb'}}
        recipe_room.update(f2_modes=operator_policy, allow_f1e_dynamic_scripts=True,
                           item_policy='report_derived_f1d_vehicles', life_actions='needle_rescue')
        config = copy.deepcopy(self.recipe)
        configure(config, self.bundle, self.cache, self.out, names, 'f2-optin')
        two = config['diagnostic_two_lobby']
        room = two['fixed_room']
        self.assertEqual(room['f2_modes'], operator_policy)
        self.assertIs(room['allow_f1e_dynamic_scripts'], False)
        self.assertEqual(room['item_policy'], 'report_derived_basic')
        self.assertEqual(room['life_actions'], 'disabled')
        self.assertEqual(room['map_profile'], room['map_choices'][0]['map_profile'])
        self.assertEqual(room['terrain'], room['map_choices'][0]['terrain'])
        self.assertEqual(len(room['map_choices']), 2)
        for i, (name, choice) in enumerate(zip(names, room['map_choices'], strict=True)):
            with self.subTest(name=name):
                code = 40 + i
                pin = choice['map_profile']
                profile = Path(pin['path']).read_bytes()
                self.assertEqual((len(profile), sha(profile)), (pin['bytes'], pin['sha256']))
                serving = json.loads(profile)
                self.assertEqual((serving['name'], serving['map_code'], serving['scripted']), (name, code, False))
                self.assertNotIn('adapter', serving)
                self.assertNotIn('drops', serving)
                terrain = choice['terrain']
                self.assertEqual(terrain['map_sha256'], serving['payload_sha256'])
                self.assertEqual(terrain['seed_sha256'], sha(bytes(terrain['kinds'])))
                self.assertEqual(terrain['kinds'][30 + i], 2)
                self.assertEqual(terrain['ray_ports'][0], 10)
                self.assertEqual(terrain['maximum_power'], 9)
                packets = choice['packets']
                self.assertEqual(terrain['fixtures'],
                                 {key: packets[key]['fixture']['sha256'] for key in ('map', 'start')})
                self.assertEqual(codec.read_fields(self.read_packet(packets['map']), [2, 2, 4]), [0, code, i])
                self.assertEqual(codec.read_fields(self.read_packet(packets['play']), [1, 4, 2])[-1], code)
                start, _, _ = blob(self.read_packet(packets['start']), 1)
                self.assertEqual(struct.unpack_from('<HI', start, 20), (code, len(name)))
                self.assertEqual(start[26:26 + len(name)], name.encode())
                self.assertEqual(struct.unpack_from('<IB', start, 26 + len(name)), (i, 0))
                self.assertEqual(self.read_choice_floor(choice), [[], [], []])
        table, _, _ = blob(self.read_packet(room['packets']['table']))
        self.assertEqual(struct.unpack_from('<H', table), (2,))
        size = (len(table) - 2) // 2
        for i, name in enumerate(names):
            row = b'\x01\x00' + table[2 + i * size:2 + (i + 1) * size]
            strings, code_at = table_row(row)
            self.assertEqual(strings[1][2], name.encode())
            self.assertEqual(struct.unpack_from('<H', row, code_at), (40 + i,))
        for actor in two['actors']:
            ready = self.read_packet(actor['profile']['packets']['ready'])
            _, end, _ = blob(ready)
            self.assertEqual(codec.read_fields(ready[end:], [4, 4, 1, 4])[0], raw_blob(table)[1])

    def test_f2_zombie_single_map_retains_client_script_marker_without_f1_hooks(self):
        self.write_f2_cache('Zombie04')
        self.recipe['diagnostic_two_lobby']['fixed_room']['f2_modes'] = {
            'runtime_overrides': {'free': 'revive'}}
        room, _ = self.configure(('Zombie04',), 'f2-optin')
        profile = json.loads(Path(room['map_profile']['path']).read_bytes())
        self.assertEqual((profile['adapter'], profile['map_code'], len(profile['spawns'])),
                         ('f2f3-probe-r1', 1143, 8))
        self.assertEqual(room['map_choices'], [])
        self.assertTrue(profile['scripted'])
        self.assertIs(room['allow_f1e_dynamic_scripts'], False)
        self.assertEqual(room['f2_modes'], {'runtime_overrides': {'free': 'revive'}})
        self.assertEqual(self.read_choice_floor(room), [[], [], []])

    def test_f2_rejects_unknown_unsupported_and_mixed_maps(self):
        cases = (
            (('NeverExported',), 'NeverExported'),
            (('Baseball01',), 'Baseball01'),
            (('CAPong01', 'CAPong01'), '1–16'),
            (('CAPong01', 'SnowyFields04'), 'không trộn họ map'),
        )
        for names, message in cases:
            with self.subTest(names=names), self.assertRaisesRegex(ValueError, message):
                self.configure(names, 'f2-optin')
        self.assertEqual(list(self.out.iterdir()), [])

    def test_f2_rejects_unverified_cache_and_invalid_profile_or_rules(self):
        name = 'Zombie04'
        cases = (
            ('.json', {'width': 19, 'height': 17}, True, '15x13'),
            ('.json', {'adapter': 'f1e-adapter-r1'}, True, 'bằng chứng F2'),
            ('.json', {'payload_sha256': '0' * 64}, True, 'bằng chứng F2'),
            ('.json', {'scripted': False}, True, 'bằng chứng F2'),
            ('.json', {'map_code': 1142}, True, 'probe Rust'),
            ('.json', {'spawns': [[1, 1], [1, 1]]}, True, 'Spawn F2'),
            ('.rules.json', {'floor': {'fixed': [[1, 1, 1]], 'hidden': []}}, True, 'địa hình tĩnh'),
            ('.rules.json', {'terrain': {'width': 15, 'height': 13, 'kinds': [0] * 194}},
             True, 'không khớp profile'),
            ('.json', {'map_code': 1142}, False, 'sai hash'),
        )
        for suffix, update, repin, message in cases:
            with self.subTest(update=update.keys(), repin=repin):
                self.write_f2_cache(name)
                member = name + suffix
                original = json.loads((self.cache / member).read_bytes())
                self.write_json(member, dict(original, **update))
                if repin:
                    self.pin_f2_cache((name,))
                with self.assertRaisesRegex(ValueError, message):
                    self.configure((name,), 'f2-optin')
        (self.cache / 'manifest.json').unlink()
        with self.assertRaisesRegex(ValueError, 'manifest'):
            self.configure((name,), 'f2-optin')
        self.assertEqual(list(self.out.iterdir()), [])

    def test_f2_rejects_native_or_mixed_runtime_presets(self):
        self.write_f2_cache('CAPong01')
        room = self.recipe['diagnostic_two_lobby']['fixed_room']
        for policy, message in (
            ({'create_overrides': {'free': 'revive'}}, 'runtime_overrides'),
            ({'runtime_overrides': {'free': 'snowy_fields'}}, 'chưa hỗ trợ'),
            ({'runtime_overrides': {'free': 'revive', 'manner': 'timebomb'}}, 'cùng preset'),
        ):
            with self.subTest(policy=policy), self.assertRaisesRegex(ValueError, message):
                room['f2_modes'] = policy
                self.configure(('CAPong01',), 'f2-optin')
        self.assertEqual(list(self.out.iterdir()), [])

    def test_f2_cli_accepts_opt_in_and_fails_cleanly_for_an_unknown_map(self):
        self.write_f2_cache('SnowyFields04')
        (self.bundle / 'config.template.json').write_text(json.dumps(self.recipe), encoding='utf-8')
        command = [sys.executable, str(TOOLS / 'kr_pack_config.py'), '--bundle', str(self.bundle),
                   '--cache', str(self.cache), '--scope', 'f2-optin']
        dest = self.root / 'cli'
        result = subprocess.run([*command, '--out', str(dest), '--map', 'SnowyFields04'],
                                capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        config = json.loads((dest / 'config.json').read_bytes())
        room = config['diagnostic_two_lobby']['fixed_room']
        self.assertNotIn('f2_modes', room)
        profile = json.loads(Path(room['map_profile']['path']).read_bytes())
        self.assertEqual(profile['name'], 'SnowyFields04')
        failed = self.root / 'unknown'
        result = subprocess.run([*command, '--out', str(failed), '--map', 'UnknownMap'],
                                capture_output=True, text=True)
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertIn('UnknownMap', result.stderr)
        self.assertNotIn('Traceback', result.stderr)
        self.assertFalse((failed / 'config.json').exists())


if __name__ == '__main__':
    unittest.main()
