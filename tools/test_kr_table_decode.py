"""Kiểm tools/kr_table_decode.py bằng pack tổng hợp nhỏ; không cần client KR."""
from contextlib import redirect_stdout
import io
import json
from pathlib import Path
import struct
import sys
from tempfile import TemporaryDirectory
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parent))
import content_census  # noqa: E402
import kr_table_decode as decode  # noqa: E402

M32 = 0xFFFFFFFF


# --- mã hoá ngược của reader trong content_census ---------------------------
def encrypt_idx(plain, key=0x1234ABCD):
    """Ngược decrypt_idx: dword đầu bản rõ luôn là 0, mỗi dword sau XOR với dword trước."""
    n = len(plain) // 4
    words = struct.unpack_from(f"<{n}I", plain)
    out = [key] + [words[i] ^ words[i - 1] for i in range(1, n)]
    tail = bytes(b ^ ((words[-1] >> (8 * j)) & 0xFF) for j, b in enumerate(plain[n * 4:]))
    return struct.pack(f"<{n}I", *out) + tail


def encrypt_entry(plain, key):
    n = len(plain) // 4
    words = struct.unpack_from(f"<{n}I", plain)
    out = []
    for i, word in enumerate(words):
        if i == 0:
            out.append(word ^ key ^ content_census.MAGIC)
        else:
            poly = (i * i * (i + 966) * (i + 2020)) & M32
            out.append(word ^ words[i - 1] ^ poly ^ content_census.MAGIC)
    last = words[-1] if words else key
    tail = bytes(b ^ ((last >> (8 * j)) & 0xFF) for j, b in enumerate(plain[n * 4:]))
    return struct.pack(f"<{n}I", *out) + tail


def write_pack(root, stem, entries):
    """Pack có index dạng extra (dword đầu 0, 16 byte sau tên); entry lẻ được mã hoá bằng khoá."""
    idd, rows = b"", []
    for number, (name, data) in enumerate(entries.items()):
        key = 0 if number % 2 else 0x5A5A0000 + number
        raw = encrypt_entry(data, key) if key else data
        rows.append((name, len(idd), len(raw), key))
        idd += raw
    plain = struct.pack("<3I", 0, 0, len(rows))
    for name, offset, size, key in rows:
        encoded = name.encode()
        plain += struct.pack("<i", len(encoded)) + encoded + struct.pack("<4I", offset, size, key, 0)
    plain += b"\0" * (-len(plain) % 4)
    (root / f"{stem}.idx").write_bytes(encrypt_idx(plain))
    (root / f"{stem}.idd").write_bytes(idd)


def xml(text, encoding="cp949"):
    body = text.encode(encoding)
    return struct.pack("<I", len(body)) + body


def text_field(value):
    raw = value.encode() + b"\0"
    return struct.pack("<I", len(raw)) + raw


def map_payload(width, height, spawns, ground, objects):
    header = struct.pack("<20i", 1000, 0, 0, width, height, len(spawns), *([0] * 14))
    cells = b"".join(struct.pack("<2i", 7 if i in ground else -1, 9 if i in objects else -1)
                     for i in range(width * height))
    return (header + b"".join(struct.pack("<2i", x, y) for x, y in spawns) + struct.pack("<i", 0) + cells
            + struct.pack("<i", 0) + struct.pack("<2i", 100, 0))


def sheet(header, rows, notes=()):
    def row(cells):
        out = []
        for cell in cells:
            if cell is None:
                out.append("<Cell/>")
            else:
                kind = "Number" if isinstance(cell, (int, float)) else "String"
                out.append(f'<Cell><Data ss:Type="{kind}">{cell}</Data></Cell>')
        return "<Row>" + "".join(out) + "</Row>"
    body = row(notes) + row(header) + "".join(row(r) for r in rows)
    return xml('<?xml version="1.0"?>\n<Workbook xmlns="urn:schemas-microsoft-com:office:spreadsheet" '
               'xmlns:ss="urn:schemas-microsoft-com:office:spreadsheet">'
               '<DocumentProperties><Author>someone</Author></DocumentProperties>'
               f'<Worksheet ss:Name="Sheet1"><Table>{body}</Table></Worksheet>'
               '<Worksheet ss:Name="Sheet2"><Table><Row/></Table></Worksheet></Workbook>', "utf-8")


SKILL = """<SkillTable>
\t# 배찌 스킬( 일격 )
\t<Skill Type="0" Name="DirectBomb">
\t\t<Property Idx="0" Name="Len"><Stat Level="1" Value="3"/><Stat Level="3" Value="4"/></Property>
\t</Skill>
<!--
\t<Skill Type="1" Name="CrossBomb"><Property Idx="0" Name="Len"><Stat Level="1" Value="3"/></Property></Skill>
-->
</SkillTable>"""
MON_ACTION = """<MonAction>
  <TriggerTable><Time until="1000">
    <Trigger ActionList="0" Type="default"/><Trigger type="Appear" ActionList="1"/>
  </Time></TriggerTable>
  <ActionList name="0" next="0"><Action name=setTriggerOption trigger="Appear"/><Wait time="5"/></ActionList>
  <ActionList name="1" next="-1"><action name="Say" say01="KeyA" say01="dup"/></ActionList>
</MonAction>
<!-- 닫히지 않은 주석"""
STAGE = """<mapscript><StageClearScript><ScriptObjects><ScriptObject>
<InitInfo type="BlueMon" image="Game/Blue"/>
<TriggerTable timeRelative="true"><Time until="10"><Trigger type="Default" ActionList="1"/></Time></TriggerTable>
<ActionList name="1" next="-1"><action name="Say" say01="KeyA"/></ActionList>
</ScriptObject></ScriptObjects></StageClearScript></mapscript>"""
CUSTOM_MAP = """<?xml version="1.0" ?>
<CustomMap>
 <Version><Value Version="1"/></Version>
 <MapSize><Value MapSize="196610"/></MapSize>
 <TeamInfo><Value TeamCount="2" TeamUserCount="1"/></TeamInfo>
 <StartPoints><Value TilePos="0"/><Value TilePos="3"/></StartPoints>
 <GroundTiles><Value Pos="0" ObjectName="T/G"/><Value Pos="1" ObjectName="T/G"/><Value Pos="3" ObjectName="T/H"/></GroundTiles>
 <ObjectTiles><Value Pos="1" ObjectName="T/B"/></ObjectTiles>
 <DecoTiles/><Character/>
 <Ban><Gadget><JumpUmbrella Value="0"/></Gadget></Ban>
 <Localdata><Value ThemeName_00="theme_00"/></Localdata>
</CustomMap>"""
TILE_INDEX = """<?xml version="1.0" encoding="UTF-8"?>
<CustomDataList><CustomTileIndexList>
<Tile Idx="0" Name="T/G"/><Tile Idx="1" Name="T/H"/><Tile Idx="2" Name="T/B" LimitCount="10"/><Tile Idx="3" Name="T/G"/>
</CustomTileIndexList><CustomItemList><Item Name="Item/ItemBomb"/></CustomItemList></CustomDataList>"""
TILE_LIST = """<?xml version="1.0" encoding="UTF-8"?>
<!-- 1 : 바닥 -->
<CustomMapInfo><TileInfo>
<Theme Name="Theme_00" IDFName="Theme/Village" Desc="빌리지">
<Tile Name="T/G" Desc="바닥" Category="1"/><Tile Name="T/B" Desc="블록" Category="2" Drawretio="120" interpolate="0,0"/>
</Theme></TileInfo>
<TemplateMapInfo><Map Name="CustomMap_Mini" IDFName="Theme/Village" SizeInfo="3x2" Desc="d" Default="1"/></TemplateMapInfo>
<ItemInfo><Item Name="Item/ItemBomb" Desc="d" Category="1" DetailDesc="d"/></ItemInfo>
<SpecialBlock><Tile Name="T/B" Desc="d" Category="1" DetailDesc="d" OverlayUrl="u"/></SpecialBlock>
<MonsterInfo><Monster Name="Game/Blue" Desc="d" Category="1"/></MonsterInfo>
</CustomMapInfo>"""
GADGET = """<GadgetTable>
\t# 점핑우산
\t<Skill Type="1" Name="JumpUmbrella">
\t\t<Property Name="cooltime" Value="3000"/>
\t\t<Property Name="arg0" Value="2000"/> # 점프 시간 (ms)
\t</Skill>
</GadgetTable>"""


def build_client(root, mon_action=MON_ACTION):
    item_attr = struct.pack("<3I", 1, 0, 0) + text_field("Item/ItemBomb") + text_field("Bomb") + struct.pack("<8i", *range(8))
    objects = [b"Pet/Pet_001", b"Dialog/SpeakerIcon", b"Game/Blue"]
    object_table = struct.pack("<I", len(objects)) + b"".join(
        struct.pack("<I", len(n)) + n + struct.pack("<I", 0) for n in objects)
    write_pack(root, "FxContents", {
        "Attr/ItemAttr": item_attr,
        "RPG/SkillTable": xml(SKILL), "RPG/SnowBallSkillTable": xml(SKILL), "RPG/ZzolmanSkillTable": xml(SKILL),
        "RPG/EquipItemTable": xml('<EquipItemTable><EquipItem attr1="0" attr2="1" Name="MASK_1">'
                                  '<Stat Level="0" Damage="0" Defense="50" Critical="0" Miss="0"/></EquipItem></EquipItemTable>'),
        "SandBox/GadgetTable": xml(GADGET)})
    write_pack(root, "FxMonster", {"MonsterSeason2/SkillTable": xml(SKILL)})
    write_pack(root, "FxDesktop", {
        "ClientData/StageClearEventNpc": xml(STAGE), "ClientData/StageClearEventNpcBoss": xml(STAGE),
        "ClientData/ClientString": xml("<DocumentElement><Elem><key>KeyA</key><string>x</string></Elem></DocumentElement>")})
    write_pack(root, "FxMap", {
        "Map/Mini": map_payload(3, 2, [(0, 0)], {0, 1, 3}, {1}) + text_field('x monAction="MapScriptTemplate/monaction_a" '
                                                                                 'skilltablename="rpg/SnowBallSkillTable" '
                                                                                 'SkillName="DirectBomb" SkillLevel=3'),
        "MapScriptTemplate/MonAction_A": xml(mon_action), "MapScriptTemplate/MonAction_B": xml(MON_ACTION),
        "Theme/Village": b"\0" * 8})
    write_pack(root, "fx", {
        "TileInfo/TileInfoList": xml('<TileInfo><Tile Name="T/G" Data="00"/><Tile Name="T/H" Data="00"/>'
                                     '<Tile Name="T/B" Data="00"/></TileInfo>'),
        "ObjectTable/ObjectTable": object_table,
        "CustomMap/CustomMap_Mini": xml(CUSTOM_MAP, "ascii"), "CustomMap/CustomTheme": b"\2\0\0\0sprite",
        "CustomMap/CustomTileIndexList": xml(TILE_INDEX, "utf-8"), "CustomMap/CustomTileList": xml(TILE_LIST, "utf-8"),
        "Pet/PetTable": sheet(["PetType", "Name", "TemplateName"], [[1, "Rabbit", "Pet/Pet_001"]], ["", "", "mô tả"]),
        "Pet/PetRandomTable": sheet(["RandomType", "Rate"], [[1, 60000], [1, 40000.0]]),
        "ObjectTable/SpeakerTable": sheet(
            ["Index", "MarkIcon", "BothSideImg", "BothSideTemplate", "StickerTemplate1", "StickerTemplate2",
             "StickerTemplate3"], [[0, "Dialog/SpeakerIcon", None, None, None, None, None]]),
        "XML/init": xml('<init>\n\t<nation name="korea"/>\n\t<info name="couple" attr0="1"/>\n</init>', "ascii")})
    (root / "CA.exe").write_bytes(b"MZ fixture")


class ScannerTests(unittest.TestCase):
    def test_lenient_scan_counts_deviations(self):
        root, issues = decode.scan('<A x=1 y="2">\n# note\n<b/> # same line\n</a><!-- open')
        self.assertEqual(root.children[0].attrs, [("x", "1"), ("y", "2")])
        self.assertEqual(root.children[0].children[0].notes, ["note", "same line"])
        self.assertEqual(issues["unquoted_attr"], 1)
        self.assertEqual(issues["unterminated_comment"], 1)
        self.assertEqual(issues["after_root"], 1)

    def test_entities_and_bare_ampersand(self):
        root, _ = decode.scan('<a v="&lt;&amp;&#65;" w="@#%$&~"/>')
        self.assertEqual(root.children[0].attr_dict(), {"v": "<&A", "w": "@#%$&~"})

    def test_payload_requires_length_prefix(self):
        with self.assertRaises(decode.DecodeError):
            decode.entry_payload(b"\x05\0\0\0<a/>", "x")

    def test_unknown_structure_is_rejected(self):
        root, _ = decode.scan('<SkillTable><Skill Type="0" Name="x"><Other/></Skill></SkillTable>')
        with self.assertRaises(decode.DecodeError):
            decode.decode_skill_table(root, "t", {})

    def test_sheet_index_gaps(self):
        data = ('<Workbook xmlns="urn:schemas-microsoft-com:office:spreadsheet" '
                'xmlns:ss="urn:schemas-microsoft-com:office:spreadsheet"><Worksheet ss:Name="S"><Table>'
                '<Row/><Row><Cell><Data ss:Type="String">A</Data></Cell><Cell><Data ss:Type="String">B</Data></Cell>'
                '<Cell><Data ss:Type="String">C</Data></Cell></Row>'
                '<Row><Cell ss:Index="2"><Data ss:Type="Number">2.5</Data></Cell></Row>'
                '</Table></Worksheet></Workbook>').encode()
        body, rows = decode.decode_sheet(data, "s", {})
        self.assertEqual(rows, 1)
        self.assertEqual(body["sheets"][0]["rows"], [[None, 2.5, None]])


class GenerateCheckTests(unittest.TestCase):
    def setUp(self):
        self.tmp = TemporaryDirectory()
        base = Path(self.tmp.name)
        self.client, self.repo = base / "client", base / "repo"
        self.client.mkdir()
        (self.repo / "docs/content").mkdir(parents=True)
        build_client(self.client)
        self.out = self.repo / "docs/content/client-tables"
        self.doc = self.repo / "docs/content/client-data-tables.md"
        self.doc.write_text(f"# Tài liệu\n\nVăn bản tay.\n\n{decode.BEGIN}\n{decode.END}\n\nĐuôi tay.\n")
        self.patch = patch.multiple(decode, ROOT=self.repo, OUT_DIR=self.out, DOC=self.doc,
                                    CENSUS=self.repo / "docs/content/census.json")
        self.patch.start()
        with redirect_stdout(io.StringIO()):
            decode.write(decode.build(self.client))

    def tearDown(self):
        self.patch.stop()
        self.tmp.cleanup()

    def load(self, name):
        return json.loads((self.out / name).read_text())

    def test_decoded_tables(self):
        skills = self.load("skill-tables.json")["entries"]
        self.assertEqual(len(skills), 4)
        self.assertEqual(skills[0]["skills"][0]["note"], "배찌 스킬( 일격 )")
        self.assertEqual(skills[0]["skills"][0]["properties"][0]["stats"], [[1, 3], [3, 4]])
        self.assertEqual(skills[0]["commented_out"], [{"type": 1, "name": "CrossBomb"}])
        gadget = self.load("gadget-table.json")["entries"][0]["gadgets"][0]
        self.assertEqual(gadget["properties"][1], {"name": "arg0", "value": 2000, "note": "점프 시간 (ms)"})
        mon = self.load("mon-action.json")["entries"]
        self.assertEqual([e["entry"] for e in mon], ["MapScriptTemplate/MonAction_A", "MapScriptTemplate/MonAction_B"])
        self.assertEqual(mon[0]["issues"]["unquoted_attr"], 1)
        self.assertEqual(mon[0]["issues"]["duplicate_attr"], [["action", "say01", "dup"]])
        self.assertEqual(mon[0]["action_lists"][0]["steps"][1], ["Wait", {"time": "5"}])
        cmap = self.load("custom-maps.json")["entries"]
        self.assertEqual(len(cmap), 1)  # CustomTheme là sprite, không phải XML
        self.assertEqual((cmap[0]["width"], cmap[0]["height"]), (3, 2))
        self.assertEqual(cmap[0]["layers"]["GroundTiles"], {"palette": ["T/G", "T/H"], "cells": [[0, 0], [1, 0], [3, 1]]})
        self.assertEqual(cmap[0]["empty_sections"], ["DecoTiles", "Character"])
        index = self.load("custom-tile-index-list.json")["entries"][0]
        self.assertEqual((index["limit_count"], index["duplicate_names"]), ({"2": 10}, ["T/G"]))
        tiles = self.load("custom-tile-list.json")["entries"][0]
        self.assertEqual(tiles["legend"], ["1 : 바닥"])
        self.assertEqual(tiles["themes"][0]["tiles"]["key_counts"]["Drawretio"], 1)
        pet = self.load("pet-table.json")["entries"][0]["sheets"]
        self.assertEqual(pet[0]["columns"][2], {"name": "TemplateName", "description": "mô tả"})
        self.assertEqual(pet[1], {"name": "Sheet2", "columns": [], "rows": []})
        self.assertNotIn("someone", (self.out / "pet-table.json").read_text())

    def test_manifest_pins_and_crossrefs(self):
        manifest = self.load("manifest.json")
        self.assertEqual(set(manifest["packs"]), {"FxContents", "FxDesktop", "FxMap", "FxMonster", "fx"})
        self.assertEqual(manifest["packs"]["fx"]["idd_sha256"], content_census.sha256(self.client / "fx.idd"))
        refs = manifest["crossrefs"]
        self.assertEqual(refs["mon_action_map_refs"]["case_insensitive_only"], ["MapScriptTemplate/monaction_a"])
        self.assertEqual(refs["mon_action_map_refs"]["unreferenced"], ["MapScriptTemplate/MonAction_B"])
        self.assertEqual(refs["skill_table_map_refs"]["item_refs"][0]["level_present"], True)
        compare = refs["custom_map"]["map_compare"][0]
        self.assertTrue(all(compare[k] for k in ("size_equal", "spawns_are_prefix", "ground_positions_equal",
                                                 "object_positions_equal")))
        self.assertEqual(refs["custom_map"]["localdata_theme_names"]["exact_case"], 0)
        self.assertEqual(refs["script_strings"]["mon-action"]["resolved"], 1)
        self.assertEqual(refs["sheets"]["pet_random_rate_sum_by_type"], {"1": 100000})
        self.assertEqual(manifest["stats"]["mon-action"]["mixed_case"]["action"], {"Action": 2, "action": 2})
        text = self.doc.read_text()
        self.assertTrue(text.startswith("# Tài liệu\n\nVăn bản tay.") and text.endswith("Đuôi tay.\n"))

    def test_check_without_and_with_client(self):
        with redirect_stdout(io.StringIO()) as output:
            decode.check()
            decode.check(self.client)
        self.assertIn("Client tables OK", output.getvalue())

    def test_check_detects_tampering(self):
        path = self.out / "xml-init.json"
        path.write_text(path.read_text().replace("korea", "korez"))
        with self.assertRaises(SystemExit):
            decode.check()

    def test_check_detects_stale_doc_and_extra_file(self):
        self.doc.write_text(self.doc.read_text().replace("| [xml-init]", "| [xml-initX]"))
        with self.assertRaises(SystemExit):
            decode.check()
        self.doc.write_text(self.doc.read_text().replace("| [xml-initX]", "| [xml-init]"))
        (self.out / "raw.idd").write_bytes(b"x")
        with self.assertRaises(SystemExit):
            decode.check()

    def test_check_with_client_detects_drift(self):
        # JSON vẫn khớp manifest; chỉ phép giải lại từ client phát hiện pack nguồn đã đổi.
        build_client(self.client, MON_ACTION.replace('until="1000"', 'until="2000"'))
        with redirect_stdout(io.StringIO()):
            decode.check()
        with self.assertRaises(SystemExit):
            decode.check(self.client)


if __name__ == "__main__":
    unittest.main()
