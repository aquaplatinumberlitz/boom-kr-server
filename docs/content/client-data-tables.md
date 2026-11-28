# Bảng dữ liệu client ngoài `Attr/` (KR 00967)

Tài liệu này mô tả định dạng các bảng mà [kiểm kê nội dung](../CONTENT.md#bảng-dữ-liệu-ngoài-attr) trước đây ghi
"chưa giải". JSON đã giải nằm ở [`client-tables/`](client-tables/manifest.json), do `tools/kr_table_decode.py` sinh từ
client gốc chỉ đọc. Manifest pin SHA-256 của mọi file pack đã đọc và của từng entry đã giải mã. Repo không chứa `.idx`,
`.idd`, `.exe` hay blob thô ([quyền dữ liệu pack](../evidence/owner-directives/pack-data-rights.json)).

```bash
python3 tools/kr_table_decode.py --client /home/admin/boom/clients/kr-00967 --generate
python3 tools/kr_table_decode.py --check                  # CI: manifest, hash, khối sinh tự động, đối chiếu census
python3 tools/kr_table_decode.py --check --client <dir>   # giải lại từ client và so từng byte
```

<!-- BEGIN kr_table_decode -->

Bảng sinh bởi `python3 tools/kr_table_decode.py --generate`; không sửa tay giữa hai dấu mốc.

| Bảng | Entry nguồn | Byte nguồn | Số dòng giải được | Lệch chuẩn XML | JSON |
|---|---|---:|---:|---|---|
| [skill-tables](#skilltable) | `FxContents:RPG/SkillTable`, `FxContents:RPG/SnowBallSkillTable`, `FxContents:RPG/ZzolmanSkillTable`, `FxMonster:MonsterSeason2/SkillTable` | 14,618 | 15 skill | không | [skill-tables.json](client-tables/skill-tables.json) |
| [equip-item-table](#equipitemtable) | `FxContents:RPG/EquipItemTable` | 2,444 | 6 item | không | [equip-item-table.json](client-tables/equip-item-table.json) |
| [gadget-table](#gadgettable) | `FxContents:SandBox/GadgetTable` | 786 | 4 gadget | không | [gadget-table.json](client-tables/gadget-table.json) |
| [stage-clear-event-npc](#stageclearevent) | `FxDesktop:ClientData/StageClearEventNpc`, `FxDesktop:ClientData/StageClearEventNpcBoss` | 14,302 | 4 ScriptObject | không | [stage-clear-event-npc.json](client-tables/stage-clear-event-npc.json) |
| [mon-action](#monaction) | `FxMap:MapScriptTemplate/MonAction*` (364 entry) | 568,455 | 364 template | after_root 1, duplicate_attr 1, unquoted_attr 265, unterminated_comment 1 | [mon-action.json](client-tables/mon-action.json) |
| [custom-maps](#custommap) | `fx:CustomMap/CustomMap_*` (15 entry) | 421,714 | 15 map | stray_text 1 | [custom-maps.json](client-tables/custom-maps.json) |
| [custom-tile-index-list](#customtileindexlist) | `fx:CustomMap/CustomTileIndexList` | 116,890 | 2,219 tile | không | [custom-tile-index-list.json](client-tables/custom-tile-index-list.json) |
| [custom-tile-list](#customtilelist) | `fx:CustomMap/CustomTileList` | 230,865 | 1,896 tile theme | không | [custom-tile-list.json](client-tables/custom-tile-list.json) |
| [pet-table](#pettable) | `fx:Pet/PetTable` | 104,517 | 94 dòng | không | [pet-table.json](client-tables/pet-table.json) |
| [pet-random-table](#petrandomtable) | `fx:Pet/PetRandomTable` | 184,454 | 276 dòng | không | [pet-random-table.json](client-tables/pet-random-table.json) |
| [speaker-table](#speakertable) | `fx:ObjectTable/SpeakerTable` | 174,151 | 63 dòng | không | [speaker-table.json](client-tables/speaker-table.json) |
| [xml-init](#xmlinit) | `fx:XML/init` | 116 | 3 phần tử | không | [xml-init.json](client-tables/xml-init.json) |

Pack nguồn (SHA-256 đầy đủ trong [manifest](client-tables/manifest.json)):

- `CharTE`: idx `9095e77f111420b6…`, idd `045cef30a9f0a6fd…` (368,661,975 byte).
- `EmoTE`: idx `ae2702ae84fc9733…`, idd `256cb9277b22431c…` (275,290,407 byte).
- `FxAsset`: idx `44a55df7ccae641a…`, idd `5f528247fecdb996…` (155,592 byte).
- `FxCharacter`: idx `39dca93599a06129…`, chỉ đọc index.
- `FxContents`: idx `56ded9cd49079a52…`, idd `1ea0b3eb8b96381c…` (59,759,874 byte).
- `FxDesktop`: idx `92b7f8b91aa926c2…`, idd `fcc57c2cacc9f037…` (54,003,285 byte).
- `FxHiddenCatch`: idx `3d6dccf3abe4dac9…`, chỉ đọc index.
- `FxMap`: idx `30fd16b63d13a4a4…`, idd `9ce6da28ec24901e…` (55,897,232 byte).
- `FxMonster`: idx `9522083b51eac71f…`, idd `6e15a7da569cf58c…` (79,552,572 byte).
- `FxSound`: idx `ad7172b1feab63ec…`, chỉ đọc index.
- `FxStage`: idx `4544d8d29a6b344c…`, chỉ đọc index.
- `FxTile`: idx `14f62d1bd283829c…`, chỉ đọc index.
- `fx`: idx `bcf921cc1be51f1c…`, idd `4465e5250bb72c0b…` (87,822,170 byte).

<!-- END kr_table_decode -->

## Cách đọc và mức chắc chắn

Mọi entry trong bảng trên có cùng vỏ: `u32` little-endian bằng kích thước entry trừ 4, sau đó là văn bản XML. Quy tắc
này đúng cho cả 404 entry và trùng với cách census nhận diện XML, nên ghi `kr_static`. Bảng mã được chọn theo khai báo
`<?xml encoding?>`, rồi thử UTF-8 chặt, rồi cp949 chặt. Bảng SpreadsheetML, `CustomTileList` và `CustomMap_Patrit14` là
UTF-8; bảng RPG, `GadgetTable`, `StageClearEventNpc*` và chín template `MonAction` là cp949; phần còn lại thuần ASCII.

Các file script không phải XML chuẩn. Có thuộc tính không ngoặc kép, comment không đóng, dòng ghi chú bắt đầu bằng `#`
nằm ngoài thẻ và tên thẻ hoặc thuộc tính lẫn chữ hoa thường. Tool dùng một scanner khoan dung, đếm mọi chỗ lệch chuẩn
vào `issues` của từng entry và dừng hẳn khi gặp phần tử hoặc thuộc tính chưa nằm trong layout đã mô tả dưới đây. Vì vậy
"đã giải" nghĩa là mọi phần tử và thuộc tính đã được tách. Nghĩa gameplay của từng trường được ghi riêng theo ba mức:

- `kr_static`: bytes chứng minh, ví dụ giá trị nhất quán trên mọi dòng hoặc khớp tham chiếu chéo tới dữ liệu pack khác.
- `candidate`: hợp lý theo tên trường, ghi chú trong file hoặc phân bố giá trị nhưng chưa có tham chiếu chứng minh.
- `unknown`: chưa có căn cứ; ghi lý do và phép kiểm cần làm.

Client so một số tên không phân biệt hoa thường. Với khoá `ClientString` và tên skill/property, mức là `kr_static`
vì parser đổi tên sang chữ thường trước khi lưu hoặc tra ([nơi đọc](#loaders)). Với tên khác, mức vẫn là `candidate`;
căn cứ là map tham chiếu `MapScriptTemplate/MonAction_Rkuo_01` trong khi entry tên `MonAction_RKuo_01`, và
`ThemeName_00="theme_05"` trỏ theme `Theme_05`. Hai quy ước sau là `candidate`. Nội dung trong `<!-- -->` bị bỏ qua;
căn cứ là cả 60 item `SkillName` của map Zzolman đều trỏ skill duy nhất không nằm trong comment. Dòng `#` là ghi chú của
người làm dữ liệu; JSON giữ chúng ở trường `note` để đọc nghĩa, không coi là dữ liệu client dùng.

Số liệu tham chiếu chéo dưới đây lấy từ `crossrefs` và `stats` trong [manifest](client-tables/manifest.json).
Không có entry nào trong 13 pack nhắc tên các bảng này; hàm nạp trong `CA.exe` được ghi ở mục [Nơi client đọc bảng](#loaders).
Ba mục cuối tài liệu trả lời [nguồn chữ thoại `Say`](#say-source), [nơi đọc bảng](#loaders) và
[bên chạy `MonAction`](#monaction-runtime), dựa trên IDA tĩnh.

<a id="skilltable"></a>
## `RPG/SkillTable`, `RPG/SnowBallSkillTable`, `RPG/ZzolmanSkillTable`, `MonsterSeason2/SkillTable`

Cây: `SkillTable` > `Skill{Type, Name}` > `Property{Idx, Name}` > `Stat{Level, Value}`. JSON ghi mỗi skill là
`{type, name, note, properties: [{idx, name, stats: [[level, value]]}]}`, kèm `commented_out` cho skill nằm trong comment.

| Trường | Kiểu | Mức | Căn cứ |
|---|---|---|---|
| `Skill.Name` | chuỗi | `kr_static` là khoá tra cứu | Map khai `skilltablename` và item `SkillName`/`SkillLevel`; 4/4 tên trên 130 item đều có trong đúng bảng, kể cả `RectNuclearBombBoss` của `RPG/SkillTable` |
| `Skill.Type` | số nguyên | `candidate` (mã skill) | Duy nhất trong mỗi bảng; `RectNuclearBomb` có cùng `Type=2` ở hai bảng |
| `Property.Idx` | số nguyên | `kr_static` là thứ tự `0..n-1` | Liên tục trong mọi skill |
| `Property.Name` | chuỗi | `kr_static` tên; `candidate` đơn vị | Có `Len`, `Damage`, `Cooltime`, `Power`, `Shape`, `Distance`, `Stuntime`, `Speed`, `Duration`, `Miss`, `Defense`. `Cooltime`, `Duration` và `Stuntime` có giá trị 1000–38000 nên có thể là mili giây; `Len` và `Distance` có thể là số ô |
| `Stat.Level` | số nguyên | `kr_static` cho mức 1; `candidate` cho ngưỡng cấp | Item map dùng `SkillLevel=1` và mức 1 có trong bảng. `RPG/SkillTable` có các mốc 1–45 giống cấp nhân vật; mọi property của cùng skill dùng chung danh sách mốc |
| `Stat.Value` | số nguyên | `kr_static` giá trị, nghĩa theo `Property.Name` | — |

`RPG/SkillTable` có 12 skill (`Type` 0–9, 12, 13). Ghi chú cp949 gán skill cho 배찌 (Bazzi) hoặc 우니 (Woonie) và cho boss
진격의 물개 (hải cẩu) hoặc 진격의 꼬물이 (ấu trùng); việc gán này là `candidate`. `RPG/ZzolmanSkillTable` chỉ có
`RectNuclearBomb` hoạt động, chín skill còn lại nằm trong comment. `RPG/SnowBallSkillTable` có `BigSnowBall` và
`MonsterSeason2/SkillTable` có `ThrowNuclearBomb`; cả hai chỉ có mức 1.

<a id="equipitemtable"></a>
## `RPG/EquipItemTable`

Cây: `EquipItemTable` > `EquipItem{attr1, attr2, Name}` > `Stat{Level, Damage, Defense, Critical, Miss}`, mọi giá trị là số
nguyên. Có 6 item, mỗi item có 5 mốc `Level` 0–4. `attr1` 0, 1, 2 khớp một-một với tiền tố tên `MASK`, `Armor`, `Weapon`;
`attr2` 0, 1 khớp với hậu tố `_0`, `_1`. Hai quan hệ này là `kr_static` trên cả 6 dòng. Nghĩa `attr1` là ô trang bị và
`attr2` là bậc vẫn là `candidate`. Không map nào trong pack tham chiếu bảng này. Exe cũng không chứa tên file hay tên
trường `Critical`, nên có thể bản 00967 không đọc bảng này (`candidate`, xem [nơi đọc](#loaders)).

<a id="gadgettable"></a>
## `SandBox/GadgetTable`

Cây: `GadgetTable` > `Skill{Type, Name}` > `Property{Name, Value}`. Có 4 gadget là `JumpUmbrella`, `RocketBoots`,
`SuperMagnetic` và `ReflectShield`, với `Type` 1–4. Bốn tên này khớp đúng bốn phần tử con của `Ban/Gadget` trong
`CustomMap_*`, nên tên là `kr_static`. Property gồm `cooltime` và `arg0`, riêng `SuperMagnetic` có thêm `arg1`. Ghi chú
cùng dòng nêu nghĩa: thời gian nhảy (ms), quãng lướt (ô), tầm và tốc độ bắn (ms mỗi ô), thời gian phản xạ (ms). Nghĩa và
đơn vị này là `candidate` vì căn cứ duy nhất là ghi chú trong file.

<a id="stageclearevent"></a>
## `ClientData/StageClearEventNpc`, `ClientData/StageClearEventNpcBoss`

Đây là đoạn `<mapscript>` > `StageClearScript` > `ScriptObjects` > `ScriptObject`. Mỗi `ScriptObject` có `InitInfo`, một
`TriggerTable` và các `ActionList`, cùng bộ từ vựng với script trong map (census đếm `InitInfo` ở 477 map). Mỗi entry có
hai object: điều phối ẩn `type="DynamicScript"` và NPC `BlueMon` tạo bằng `CreateObj class="555"`. Bản Boss chỉ khác ở hai
chỗ: trigger `BossIn` thay `FirstMonDie`, và trước khi tạo NPC tại `BossPos` có chờ 1000. Ảnh `Game/MonSealSmall` và
`Game/BlueMon` có trong ObjectTable (2/2). Cả 14 khoá `say01..03` (`BlueMonSay_*`) không có trong
`ClientData/ClientString` mà có trong `csysstr.ssd` ([nguồn chữ thoại](#say-source)). Thuộc tính `InitInfo` và tham số action
được giữ dạng chuỗi; nghĩa từng thuộc tính là `candidate` theo tên.

<a id="monaction"></a>
## `MapScriptTemplate/MonAction_*` (364 template)

Cây: `MonAction` > `TriggerTable{timeRelative?}` > `Time{until}` > `Trigger{Type, ActionList}`, cùng các
`ActionList{name, next}` chứa bước `Action{name, …}`, hoặc `Wait{time}` ở 5 chỗ. JSON giữ mỗi bước là `[thẻ, {thuộc tính}]`,
đúng thứ tự và đúng cách viết hoa thường trong file. Mỗi template có đúng một `TriggerTable` với một `Time`.

- Vai trò template là `kr_static`. 82 map tham chiếu template qua `monAction="MapScriptTemplate/<tên>"` với 2.108 lần
  và 343 tên khác nhau. Cả 343 tên đều có entry; chỉ `MonAction_Rkuo_01` khớp khi bỏ qua hoa thường. Có 342/364
  template được dùng; 22 template không map nào trỏ tới, gồm các bản `*_Test` (danh sách trong manifest).
- `Trigger.ActionList` trỏ `ActionList.name`, còn `next` trỏ danh sách chạy tiếp, với `-1` là dừng. Quan hệ trỏ này là
  `candidate`, dựa trên tên và giá trị số. Loại trigger gồm `default` (364, mỗi template một), `selfdamaged`,
  `selfreleased`, `selfdie`, `appear`, `approach`, `bossanger`, `selflocked`, `bossdamaged`, `lastmonleft` và `die`.
- Tên bước theo số lần: `pattern`, `wait`, `setstate`, `createobj`, `attack`, `playanim`, `settriggeroption`, `move`,
  `say`, `turn`, `createmonmissile`, `specialattack`, `dropitem`. Số đếm và danh sách `func` ở `stats.mon-action`.
  Hành vi từng bước là `unknown` khi chưa RE bộ diễn giải AI của client. Server chỉ cần dữ liệu này nếu tự mô phỏng quái.
- Bước `Say` có 64 giá trị `say01..03` khác nhau, không giá trị nào có trong `ClientString`. 51 giá trị là khoá của
  `csysstr.ssd`; 12 giá trị còn lại là chữ viết thẳng ([nguồn chữ thoại](#say-source)).
- Lệch chuẩn: 265 thuộc tính không ngoặc kép ở 130 template, như `name=setTriggerOption`.
  `MonAction_12_SM_Red_Hidden_Boxmon01` lặp thuộc tính `pos01`; JSON giữ giá trị đầu, còn client chọn giá trị nào là
  `unknown`. `MonAction_00_SM_MonEgg2_00` có comment không đóng sau thẻ gốc. Cả hai file này đều không được map nào dùng.
  Comment đó là ghi chú hướng dẫn trigger `Appear` và `Approach`: `Appear` cần `setTriggerOption`, còn `Approach` chọn
  tầm nhìn `rectangle` hoặc `straight`. Ghi chú này là căn cứ `candidate` cho nghĩa hai trigger.

<a id="custommap"></a>
## `CustomMap/CustomMap_*` (15 map mẫu)

Cây: `CustomMap` > các mục theo thứ tự ghi ở `sections`. JSON gom mỗi lớp ô thành `palette` và các cặp `[pos, chỉ số
palette]`.

| Mục | Mức | Căn cứ |
|---|---|---|
| `MapSize/Value@MapSize` = `(width << 16) \| height` | `kr_static` | Khớp tên của 9 template và kích thước của 6 map gốc cùng tên (`Camp08`, `Factory07`, `Forest07`, `Patrit14`, `Tomb12`, `Village10`) |
| `StartPoints/Value@TilePos`, `*Tiles/Value@Pos` = `y * width + x` | `kr_static` | Ở 6 map gốc, điểm xuất phát trùng spawn của map theo đúng thứ tự (Tomb12/Village10 có 6 spawn, mẫu có thêm 2 điểm). Tập vị trí `GroundTiles` và `ObjectTiles` trùng ô khác `-1` của lớp nền và lớp vật thể |
| `*Tiles/Value@ObjectName` | `kr_static` | 183/183 tên có trong `CustomTileIndexList` |
| `TeamInfo/Value@TeamCount`, `@TeamUserCount` | `kr_static` bất biến; `candidate` nghĩa | Tích hai giá trị bằng số điểm xuất phát ở 15/15 map; nghĩa là số đội × người mỗi đội |
| `Version/Value@Version` | `candidate` | 9 template có giá trị 1; 6 map chuyển từ map gốc có giá trị 2 và có thêm mục `MonTiles` |
| `Ban/Gadget/<tên>@Value` | `kr_static` tên; `unknown` giá trị | Tên khớp `GadgetTable`; mọi giá trị đều là 0. `31x27_01` có `Ban/Item/Attr3_3` chưa rõ nghĩa |
| `Localdata/Value@ThemeName_NN` | `kr_static` tham chiếu; `candidate` nghĩa | 36/36 giá trị khớp `Theme_NN` của `CustomTileList` khi bỏ qua hoa thường (4 khớp đúng chữ); có thể là các theme mở sẵn trong editor |
| `DecoTiles`, `FixedItems`, `MonTiles`, `Character`, `Env`, `Time` | `unknown` | Rỗng ở cả 15 map nên chưa có layout phần tử con; tool dừng nếu gặp nội dung |

`CustomMap_Patrit14` có một ký tự lạc `ㄴ` trong `ObjectTiles`, được đếm là `stray_text`.

<a id="customtileindexlist"></a>
## `CustomMap/CustomTileIndexList`

Cây: `CustomDataList` > `CustomTileIndexList` > `Tile{Idx, Name, LimitCount?}` và `CustomItemList` > `Item{Name}`.
`Idx` liên tục 0–2218 (`kr_static`). 2.214 tên khác nhau đều có trong `fx/TileInfo/TileInfoList`. Năm tên xuất hiện hai lần
(`duplicate_names`). `LimitCount="10"` có ở đúng 15 tile `TileCustom/Mon_*`, trùng 15 dòng `MonsterInfo` có ghi
"(10개 배치 제한)" (giới hạn đặt 10 cái). Nghĩa giới hạn số lần đặt trong editor là `kr_static`: hàm đặt ô từ chối khi số đã
đặt đạt `LimitCount` và hiện chuỗi `CustomMap_MonsterLimitCount` ([nơi đọc](#loaders)). 65 tên
item đều khớp `Attr/ItemAttr` sau chuẩn hoá tên như census. `Idx` có thể là mã tile khi lưu hoặc gửi map tự tạo (F7), nhưng
đây là `unknown` khi chưa có capture hay RE đường lưu.

<a id="customtilelist"></a>
## `CustomMap/CustomTileList`

Cây: `CustomMapInfo` > `TileInfo` > `Theme{Name, IDFName, Desc}` > `Tile{Name, Desc, Category, Drawretio?, interpolate?}`,
cùng các mục `TemplateMapInfo/Map`, `ItemInfo/Item`, `SpecialBlock/Tile` và `MonsterInfo/Monster`. JSON giữ mỗi phần tử
là dict thuộc tính, kèm `key_counts` để thấy thuộc tính tuỳ chọn. Comment đầu file được giữ ở `legend`.

- 42 theme với 1.896 tile. Mọi tên tile có trong `CustomTileIndexList` và 36/36 `IDFName` có entry `FxMap/Theme/*`
  (`kr_static`). `Category` 1–4 theo chú giải trong file là 바닥 (nền), 블록 (khối), 단단 (cứng) và 특수 (đặc biệt).
  Chú giải là `kr_static` văn bản; hành vi va chạm tương ứng là `candidate`.
- `TemplateMapInfo` có 15 dòng ứng với 15 `CustomMap_*`. `SizeInfo` khớp `MapSize` ở 13 dòng; hai mẫu `31x27_*` ghi
  `31x37` nhưng có 837 = 31 × 27 ô nền, nên `MapSize` là nguồn đúng. `Default=1` chỉ có ở `15x13_01`.
- `ItemInfo` (63 dòng, đều khớp `Attr/ItemAttr`), `SpecialBlock` (127) và `MonsterInfo` (31) đều có tên tra được trong
  TileInfoList, ObjectTable hoặc index pack. `Desc` và `DetailDesc` là mô tả tiếng Hàn cho người dùng editor.
  `OverlayUrl` trỏ ảnh gif ngoài client. `Drawretio`, `interpolate`, `MonType` và `collisionSize` là `candidate` theo tên.

<a id="pettable"></a>
## `Pet/PetTable`

Excel 2003 XML (SpreadsheetML). Ở `Sheet1`, dòng 1 là mô tả tiếng Hàn, dòng 2 là tên cột và từ dòng 3 là dữ liệu.
Client đọc từng cột theo đúng tên ở dòng 2 (`kr_static`, [nơi đọc](#loaders)), nên thứ tự cột không mang nghĩa.
`Sheet2` và `Sheet3` rỗng. JSON lấy kiểu theo `ss:Type` (`Number` thành số, `String` giữ chuỗi), giữ mô tả cột ở
`description` và bỏ phần `DocumentProperties` (tên người sửa file) cùng style. Bảng có 94 dòng × 14 cột.

- `PetType` là số duy nhất phủ đủ 1–94. `TemplateName` khớp 94/94 tên trong ObjectTable. Hai trường này là `kr_static`.
- `AbilityType`, `SayIdx`, `Ability_Special_*`, `Upgrade_Appearance` và `Ability_Couple` là chuỗi dạng danh sách
  `[a, b, c]`, nối bằng ` & ` khi có nhiều nhóm. Tool giữ nguyên chuỗi. Mô tả cột cho cú pháp và mã liệt kê, ví dụ
  `AbilityType` = `[Type, Prop, UseCount]` với Type 1 cứu bằng kim, 2 khiên, 3 EXP, 4 tiền (루찌), 5 hút tiền, 6 rải tơ.
  Các nghĩa lấy từ mô tả này là `candidate`.
- `BonusExpStyle`, `BonusLucciStyle`, `Ability_LucciCollect` và `HasTeamColor` là mã liệt kê theo mô tả cột (`candidate`).
  Tên `BlackAlpaca` dùng cho hai `PetType` (45 và 84).

<a id="petrandomtable"></a>
## `Pet/PetRandomTable`

Cùng định dạng, có 276 dòng × 9 cột: `RandomType`, `RandomIdx`, năm cột năng lực như `PetTable` và `Rate`. `RandomIdx`
duy nhất trong mỗi `RandomType`. Tổng `Rate` của từng `RandomType` 1–6 đều bằng 100000 (`kr_static`), nên `Rate` có thể
là trọng số trên 100000 (`candidate`). Bảng không có cột trỏ `PetType`, nên quan hệ giữa `RandomType` và pet hay vật phẩm
là `unknown`. 85 ô `BonusExpStyle` và 84 ô `BonusLucciStyle` có kiểu `String` (ví dụ `"-1"`); JSON giữ đúng kiểu gốc.

<a id="speakertable"></a>
## `ObjectTable/SpeakerTable`

Cùng định dạng, có 63 dòng × 25 cột; dòng mô tả rỗng. `Index` liên tục 0–62 (`kr_static`). Mọi tên tài nguyên đều tra
được trong ObjectTable hoặc index pack (`kr_static`): `MarkIcon` (2 tên), `BothSideImg` (10) và 42 tên ở
`BothSideTemplate`/`StickerTemplate1..3`. Các cột `BGColor` và `FontColor1..12` là chuỗi `"R, G, B"` (`candidate`).
`FontTwinkleTick` và `BothSideImgFrame` là số (`candidate` theo tên). `Shortcut` là lệnh chat gõ phím Latin theo bàn phím
Hàn, ví dụ `/ghkrtjd` ứng với 확성 (loa); nghĩa này là `candidate`. `Name` là tên hiển thị tiếng Hàn.

<a id="xmlinit"></a>
## `XML/init`

`init` > `nation{name="korea"}` và hai `info{name, attr0}` với `couple` và `quest` cùng `attr0="1"`. Đây có thể là cờ
bật tính năng cặp đôi và nhiệm vụ cho bản Hàn (`candidate`). Exe không chứa chuỗi `XML/init` hay `nation`, nên có thể
bản 00967 không đọc file này (`candidate`); tác dụng là `unknown`.

<a id="say-source"></a>
## Nguồn chữ thoại của bước `Say`

Mức `kr_static` (IDA tĩnh trên bản unpack 00967, IDB `6be15a1f…`). Bước `Say` không mang chữ mà mang khoá
`FxClientString`. Client tra khoá trong kho FxClientString; kho này nạp từ bản cache `csysstr.ssd` cạnh `CA.exe` và
được server cập nhật qua gói bảng chuỗi. Cách tra đổi khoá sang chữ thường, và nếu không thấy thì hiện nguyên giá trị.

| Hàm | Vai trò |
|---|---|
| `0xF17C30` | Factory bước của script: tên `say` tạo `CMapActionSay` (`0xF530C0`). |
| `0xF53180` | Parse: gom mọi thuộc tính có tên bắt đầu bằng `say` (`say01`, `say02`, …). |
| `0xF53320` | Chạy: chọn ngẫu nhiên một giá trị rồi gửi `C->S 132` (u32, u16 mã đối tượng, u16 0, chuỗi); không tự hiện bong bóng. |
| `0x61CE90` | Nhận thông điệp đồng bộ của đối tượng; loại 1 đọc chuỗi, tra khoá qua `0x52A500`, rồi đặt chữ cho bong bóng (`0xB6B700`). |
| `0x13950D0`, `0x1394D20` | Tra khoá FxClientString: đổi sang chữ thường, không thấy thì trả nguyên chuỗi vào. |
| `0x1394670`, `0x1394770`, `0x13948B0` | Nạp `csysstr.ssd` ("Load FxClientString cache") và cập nhật khi server gửi bảng mới. |

Đối chiếu dữ liệu: 14/14 khoá của `StageClearEventNpc*` và 51/64 giá trị của `MonAction` có trong `csysstr.ssd` (15 chỉ khớp
khi bỏ qua hoa thường); 1 giá trị rỗng. 12 giá trị còn lại là chữ viết thẳng: ký hiệu (`!`, `?`, `ZZZ`, `@#%$&~`) và ba câu
tiếng Hàn. Ba câu này chỉ nằm trong `MonAction_Monkey_Test` và `MonAction_Sancho_Test`, hai template không map nào dùng.

Ảnh hưởng bản địa hoá: chữ thoại NPC và quái đang dùng đều thuộc `csysstr.ssd`. Catalog `boom-re-l10n` (`l10n/catalog.csv`,
nguồn `csysstr.ssd`) đã có bản `vi` và `en` cho cả 65 khoá. Không có câu thoại tiếng Hàn nào trong `MapScriptTemplate` lộ ra
khi chơi, nên không phải dịch thêm file pack. Một giới hạn còn mở (`candidate`): server có gói cập nhật bảng chuỗi; nếu server
gửi bảng gốc tiếng Hàn thì cache dịch có thể bị ghi đè.

<a id="loaders"></a>
## Nơi client đọc bảng

Đã tìm được hàm đọc cho 10/12 nhóm bảng (IDA tĩnh, chỉ ghi địa chỉ và vai trò).

| Bảng | Hàm | Vai trò |
|---|---|---|
| Bốn `SkillTable` | `0x14005C0`, `0x1400800`, `0x13FB510`, `0x13FBE10`, `0x13FBF90`, `0xEB2BD0` | `CSkillTableMan` tra bảng theo tên (mặc định `RPG/SkillTable`); parse `Skill`, `Property`, `Stat`; map đọc `skilltablename` |
| `EquipItemTable` | không thấy | Exe không chứa tên file hay `Critical` |
| `GadgetTable` | `0x53B4C0`, `0x53B7D0` | Nạp `SandBox/GadgetTable`; parse `Property` (có đọc tên `cooltime`) |
| `StageClearEventNpc*` | `0xF1B130` | Nạp `ClientData/StageClearEventNpc` và bản `Boss` |
| `MonAction_*` | `0x7450C0`, `0x7443E0` | `_parseMonActionTemplate`; đọc thuộc tính `monAction` của map |
| `CustomMap_*` | `0x508370`, `0x5091F0`, `0x4621C0` | Đọc và ghi XML map tự tạo; đọc `TemplateMapInfo` |
| `CustomTileIndexList` | `0x507980`, `0x507DD0`, `0x99CD70` | Nạp danh sách ô; đọc `LimitCount`; editor từ chối đặt khi vượt giới hạn |
| `CustomTileList` | `0x461A70` | Nạp danh sách theme và ô của editor |
| `PetTable` | `0x1408CA0` | Đọc từng cột theo tên ở dòng 2 |
| `PetRandomTable` | `0x1409440` | Đọc theo tên cột, kể cả `RandomType` và `Rate` |
| `SpeakerTable` | `0xBA78E0` | Đọc theo tên cột (`FontColor%d`, `StickerTemplate%d`, …) |
| `XML/init` | không thấy | Exe không chứa `XML/init` hay `nation` |

Trường được nâng lên `kr_static` từ code: tra tên skill và property không phân biệt hoa thường (parser đổi sang chữ
thường); `Property.Idx`, `Stat.Level` và `Stat.Value` được đọc thành số nguyên; nghĩa giới hạn đặt của `LimitCount`; cột
SpreadsheetML được đọc theo tên. Đơn vị skill/gadget, nghĩa `Rate` và các mã liệt kê của pet chưa được chứng minh bằng code
dùng, nên giữ `candidate`.

<a id="monaction-runtime"></a>
## Bên chạy `MonAction`

Kết luận: client chạy các bước `MonAction`, nhưng chỉ trên client đang giữ quyền điều khiển đối tượng quái. Server phải giao
quyền đó và chuyển tiếp kết quả cho các client khác, không phải diễn giải từng bước. Đường code dưới đây là `kr_static`.

| Hàm | Vai trò |
|---|---|
| `0x73C790` → `0x115D440` → `0x115DD30` | Parser StageClear tạo quái `CSyncGameObject` và đăng ký với chủ `-1`, tức chưa phải đối tượng của máy này. |
| `0xE5D3A0` → `0x115CB10` → `0x115D2E0` | `CGameStage::OnPacket` chuyển gói không thuộc bảng chính cho bộ quản lý đồng bộ; gói `204` (u16 mã đối tượng, u8 slot) giao quyền cho slot cục bộ ("Delegate ObjId[%d] to Local SlotIdx %d"). |
| `0xB91F60` → `0x1017890` → `0x1018090` | Nhịp của quái chỉ chạy danh sách bước khi cờ đối tượng từ xa (`0x11656A0`) bằng 0. |
| `0xF53320` và các bước trong `0xF50DD0..0xF572E0` | Kết quả bước được gửi đi bằng `C->S 132`/`180` ([F1-e](../protocol/f1e-dynamic-scripts.md)); `Say` không tự hiện trên máy gửi. |

Phần `candidate`: server gốc giao quyền cho client chủ phòng hay cho client khác, và server chuyển tiếp `132` thành
thông điệp đồng bộ loại 1 mà `0x61CE90` đọc. Cả hai cần capture Wine hai client. Không thấy kiểm cờ chủ phòng
`0x1240CF0` trên đường chạy bước.

## Ẩn số còn mở

- Ngữ nghĩa từng loại bước `MonAction` và `StageClearEventNpc*` (tham số, đơn vị). Cần RE từng lớp `CMapAction*` hoặc
  capture PvE; ghi tiếp ở F3.
- Slot server gốc giao quyền quái (gói `204`) và cách server chuyển tiếp `C->S 132`/`180`. Cần capture Wine hai client.
- Vai trò `Idx` của `CustomTileIndexList` khi lưu hoặc gửi map tự tạo, và layout các mục rỗng của `CustomMap`. Cần
  capture F7 hoặc RE đường lưu map.
- `EquipItemTable` và `XML/init` không có hàm đọc trong exe 00967; nghĩa `Rate`/`RandomType` của `PetRandomTable` cần
  đọc code dùng bảng sau hàm nạp `0x1409440`.
- Giá trị ô trong lớp nền và lớp vật thể của file map nhị phân (ví dụ `1932` ứng với `Tile00_Box/BoxGray`) ánh xạ một-một
  sang tên tile ở 6 map so được (93 giá trị, không xung đột) nhưng không phải chỉ số `TileInfoList`. Bảng gốc của các giá
  trị này là `unknown` và thuộc định dạng map, ngoài phạm vi tài liệu này.
