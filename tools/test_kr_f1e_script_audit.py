"""Fixture tổng hợp cho audit F1-e; không cần pack KR 00967."""
from pathlib import Path
import struct
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parent))
from kr_f1e_script_audit import audit_map, audit_soul, family_of, map_tail_offset, parse_trailer, summarize_script


def sized(data):
    return struct.pack('<I', len(data)) + data


def map_prefix(version=1001):
    header = struct.pack('<20i', version, 0, 0, 2, 1, 1, *([0] * 14))
    spawns = struct.pack('<2i', 1, 0)
    rects = struct.pack('<5i', 1, 0, 0, 1, 0)
    grid = struct.pack('<6i', 0, 2, -1, 1, 2, 3) if version == 1001 else struct.pack('<4i', 0, 2, 1, -1)
    drops = struct.pack('<I', 1) + sized(b'Item/Test\0') + struct.pack('<i', 9)
    placements = struct.pack('<2i', 60, 1) + sized(b'Item/Other\0') + bytes((1, 0, 100))
    return header + spawns + rects + grid + drops + placements


class SoulTrailerTests(unittest.TestCase):
    def setUp(self):
        self.prefix = map_prefix(1000)
        self.groups = struct.pack('<II4iI2i', 2, 2, -2, 4, 7, -9, 1, 11, 2)
        self.records = (
            struct.pack('<I', 2)
            + sized(b'InnerSoul\0') + struct.pack('<BB3i', 255, 6, -8, 25, 1)
            + sized(b'OuterSoul\0') + struct.pack('<BB3i', 0, 27, 3, -4, 12)
        )
        self.payload = self.prefix + self.groups + self.records + bytes(4)

    def test_cursor_after_placement_and_exact_trailer(self):
        self.assertEqual(map_tail_offset(self.payload), len(self.prefix))
        info = audit_soul('Synthetic', self.payload, 8)
        self.assertEqual(info['groups'], [[[-2, 4], [7, -9]], [[11, 2]]])
        self.assertEqual(info['group_point_counts'], [2, 1])
        self.assertEqual(info['group_count'], 2)
        self.assertEqual(info['record_count'], 2)
        self.assertEqual(info['records'], [['InnerSoul', 255, 6, -8, 25, 1], ['OuterSoul', 0, 27, 3, -4, 12]])
        self.assertEqual(info['script_length_offset'], len(self.payload) - 4)
        self.assertEqual(info['script_length'], 0)

    def test_rejects_bad_name_length_and_point_count(self):
        name_offset = len(self.prefix + self.groups) + 4
        bad_name = self.payload[:name_offset] + struct.pack('<I', 0xffffffff) + self.payload[name_offset + 4:]
        with self.assertRaisesRegex(ValueError, 'Độ dài chuỗi sai'):
            parse_trailer(bad_name, len(self.prefix))
        points_offset = len(self.prefix) + 4
        bad_points = (self.payload[:points_offset] + struct.pack('<I', 0xffffffff)
                      + self.payload[points_offset + 4:])
        with self.assertRaisesRegex(ValueError, 'Số điểm sai'):
            parse_trailer(bad_points, len(self.prefix))

    def test_rejects_invalid_script_end(self):
        for invalid in (self.payload[:-1], self.payload + b'x', self.payload[:-4] + struct.pack('<I', 1) + b'x'):
            with self.subTest(size=len(invalid)), self.assertRaises(ValueError):
                audit_soul('Synthetic', invalid, 8)
        with self.assertRaisesRegex(ValueError, 'Placement vượt payload'):
            map_tail_offset(self.prefix[:-1])


class ScriptTests(unittest.TestCase):
    def summarize(self, name, source):
        return summarize_script(f'<mapscript>{source}</mapscript>'.encode('cp949') + b'\0', family_of(name))

    def test_coloring_handles_unquoted_numbers_and_skips_comments(self):
        meta, spec = self.summarize('Fm_Coloring03', '''
            <!-- -- 주석 <changetile ms=80/> <ColoringPattern groupIdx=99/> -->
            <ColoringPattern groupIdx="0" regensecond = 3 gain=-5 desc="keep x=17 > 3" />
        ''')
        self.assertFalse(meta['xml_strict'])
        self.assertEqual(meta['comments'], 1)
        self.assertEqual(meta['unquoted_attrs'], 2)
        self.assertEqual(spec['patterns'], [{'groupidx': 0, 'regensecond': 3, 'gain': -5, 'desc': 'keep x=17 > 3'}])
        with self.assertRaises(ValueError):
            self.summarize('Fm_Coloring03', '<ColoringPattern groupIdx=3x/>')

    def test_space_counts_positions_and_distinct_times(self):
        _, spec = self.summarize('Space01', '''
            <changetile ms="140" changepostfix="fall" dangertime="80" tilecategory="Test">
                <position x="-1" y="12"/></changetile>
            <changetile ms="100" changepostfix="rise" dangertime="80" tilecategory="Test">
                <position/><position/></changetile>
            <!-- <changetile ms="9000"><position/><position/></changetile> -->
            <CHANGEtile ms="140" changepostfix="fall" dangertime="60" tilecategory="Test"><position/></CHANGEtile>
        ''')
        self.assertEqual((spec['changetile'], spec['positions'], spec['full_lines']), (3, 4, 1))
        self.assertEqual(spec['ms']['timestamps'], [100, 140])
        self.assertEqual(spec['ms']['steps'], [40])
        self.assertEqual(spec['dangertime'], [60, 80])
        self.assertEqual(spec['action_types'], ['fall', 'rise'])

    def test_pangland_keeps_time_blocks_and_explicit_bomb_count(self):
        _, spec = self.summarize('PangLand00', '''
            <time ms="0"><state state="1"/></time>
            <time ms="350" type="random"><wave name="wave/SprinklerBomb"/><bomb count="2" power="1" state="3"/>
                <bomb power="3" state="3"/><state state="3"/></time>
            <time ms="350"><state state="4"/></time>
        ''')
        self.assertEqual(spec['events'], [
            {'ms': 0, 'state': 1},
            {'ms': 350, 'type': 'random', 'wave': 1, 'sprinkler_bomb': 1, 'bomb': 2, 'state': 1},
            {'ms': 350, 'state': 1},
        ])
        self.assertEqual(spec['bombs']['total_explicit_count'], 2)
        self.assertEqual(spec['bombs']['missing_count'], 1)

    def test_aqua_marker_and_wriggle_object_tables(self):
        _, marker = self.summarize('NXCL_Aqua01', '<StageClearScript><!-- <ScriptObject/> --></StageClearScript>')
        self.assertEqual(marker['variant'], 'stage_clear_marker_only')
        obj = '''
            <ScriptObject><InitInfo type="TestMon" pos="7,3" class="42"/>
                <TriggerTable><Time until="89">
                    <Trigger Type="Default" ActionList="7"/><Trigger Type="SelfLocked" ActionList="8"/>
                    <Trigger Type="SelfReleased" ActionList="8"/>
                </Time></TriggerTable><Action func="absolute2"/>
            </ScriptObject>
        '''
        _, aqua = self.summarize('Aqua03', f'<StageClearScript>{obj}</StageClearScript>')
        self.assertEqual(aqua['variant'], 'stage_clear_objects')
        self.assertEqual(aqua['actions']['funcs'], {'absolute2': 1})
        _, wriggle = self.summarize('CaJJ_Wriggle01', f'''
            <WriggleScript><Domain index="0" gridRect="0,0,6,12"/>
                <Match vs="2"/><Sweeping haggardClass="300"/>{obj}</WriggleScript>
        ''')
        self.assertEqual(wriggle['domain_rows'], [{'index': 0, 'gridrect': '0,0,6,12'}])
        self.assertEqual(wriggle['matches'], [{'vs': 2}])
        self.assertEqual(wriggle['triggers'], {'Default': 1, 'SelfLocked': 1, 'SelfReleased': 1})
        self.assertEqual(wriggle['objects']['table_count'], 1)
        self.assertEqual(wriggle['objects']['tables'][0]['trigger_tables'][0][0], {
            'until': 89, 'triggers': [
                {'type': 'Default', 'actionlist': 7}, {'type': 'SelfLocked', 'actionlist': 8},
                {'type': 'SelfReleased', 'actionlist': 8},
            ],
        })

    def test_retro_distinguishes_popup_cloud_and_rides(self):
        popup = '<time ms="5"><PopUp name="Retro/Demo"/></time>'
        _, basic = self.summarize('RetroCampN01', popup)
        self.assertEqual(basic['popups'], [{'ms': 5, 'popup': 'Retro/Demo'}])
        _, cloud = self.summarize('RetroForestN01', popup + '''
            <retroscript><cloud cloudappeartime="100">
            <numberofusers usercount="2" count="1"/><Respawn col="7" row="2"/>
            </cloud></retroscript>
        ''')
        self.assertEqual(cloud['cloud'], [{'cloudappeartime': 100}])
        self.assertEqual(cloud['respawn_positions'], [{'col': 7, 'row': 2}])
        self.assertEqual(cloud['user_counts'], [{'usercount': 2, 'count': 1}])
        _, rides = self.summarize('RetroVillageN01', popup + '<retroscript><rideinfo ridetype="61"/></retroscript>')
        self.assertEqual(rides['ride_types'], [61])

    def test_lovelyroom_keeps_comment_only_data_out_of_live_summary(self):
        _, spec = self.summarize('LovelyRoom01', '''
            <mapscriptoption enablebomb="0"/>
            <!-- -- inactive <ItemTable tablename="Lucci">
                <Item name="LucciGold" weight="10"/></ItemTable>
                <time ms="50"><state type="item" state="1"/></time>
            -->
        ''')
        self.assertEqual(spec['options'], [{'enablebomb': 0}])
        self.assertEqual(spec['live_tags'], ['mapscript', 'mapscriptoption'])
        self.assertEqual(spec['comment_only']['tables'], [
            {'attrs': {'tablename': 'Lucci'}, 'items': [{'name': 'LucciGold', 'weight': 10}]},
        ])
        self.assertEqual(spec['comment_only']['events'], [{'ms': 50, 'states': [{'type': 'item', 'state': 1}]}])

    def test_rescue_extracts_appearance_tables(self):
        _, spec = self.summarize('Fm_RescueET01', '''
            <RescueAliensScript><InitInfos><InitInfo initAppearCount="3" pos00="2,7"/></InitInfos>
                <ScriptObject><InitInfo type="Alien"/></ScriptObject>
                <ScriptObject><InitInfo type="BadAlien"/></ScriptObject>
                <ScriptObject><InitInfo type="TailAlien"/></ScriptObject>
                <ScriptObject><InitInfo type="Spaceship"/>
                    <AppearPosList><Pos col="5" row="9"/></AppearPosList>
                    <AppearActionList><Action appeartime="200" duration="500"/></AppearActionList>
                </ScriptObject>
            </RescueAliensScript>
        ''')
        self.assertEqual(spec['objects']['types'], dict.fromkeys(('Alien', 'BadAlien', 'TailAlien', 'Spaceship'), 1))
        self.assertEqual(spec['init_position_fields'], {'pos00': '2,7'})
        self.assertEqual(spec['appearance_positions'], [{'col': 5, 'row': 9}])
        self.assertEqual(spec['appearance_events'], [{'appeartime': 200, 'duration': 500}])

    def test_bigbattle_without_blob_reports_grid_and_all_layers(self):
        profile = audit_map('BSBigCamp08', map_prefix() + bytes(12), 80, ['a', 'b', 'c', 'd'])
        self.assertEqual(profile['size'], [2, 1])
        self.assertEqual(profile['spawns'], 1)
        self.assertEqual({layer: row['codes'] for layer, row in profile['tiles'].items()},
                         {'ground': [0, 1], 'object': [2], 'overlay': [3]})
        self.assertEqual(profile['script']['bytes'], 0)
        self.assertFalse(profile['script']['marker'])
        with self.assertRaisesRegex(ValueError, 'Hai trường đầu'):
            audit_map('BSBigCamp08', map_prefix() + sized(b'x') + bytes(8), 80, ['a', 'b', 'c', 'd'])
        with self.assertRaises(KeyError):
            family_of('Fm_AIBtlTomb22')


if __name__ == '__main__':
    unittest.main()
