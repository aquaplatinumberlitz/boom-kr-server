# Kiểm kê PvE và map có script F3-0

Sinh từ [census](census.json) và [xếp loại A/B/C](classification.json) của client `KR00967`. Phạm vi theo [lộ trình F3](../PLAN.md#full-game-roadmap) và [bảng nội dung](../CONTENT.md#mode). [Catalog JSON](../evidence/content/f3-pve-catalog.json) giữ tên từng map, nhãn script theo họ và hash nguồn.

Catalog có **699 map / 180 họ** (692 `exact`, 7 `partial`). Có **577 map / 162 họ** trong phạm vi chính và **122 map / 18 họ** để đối chiếu Quest / Mission.

Quyền thực thi trigger, quyền quyết định AI quái, quyền cấp thưởng và route KR wire đều là `unknown` ở mọi nhóm. Tag, tên tài nguyên và nhãn xếp loại chỉ giúp chọn mẫu kiểm.

## Phạm vi và dấu vết theo nhóm

Cột Script đếm có / không / chưa rõ từ cờ `scripted` của census. Hàng `exact` đọc trọn; hàng `partial` đọc một phần nên trạng thái script vẫn chưa rõ. Các tag là tên phân biệt hoa thường do census ghi ở `script_tags`; không thêm thẻ `<script>` vào danh sách nếu census chỉ ghi cờ. JSON giữ riêng tên thu được từ phần đã đọc của map `partial`.

| Nhóm | Họ | Map exact / partial | Script có / không / chưa rõ | Xếp loại mode / hệ thống (đợt, lớp) | Tài nguyên (số mục) |
|---|---:|---:|---:|---|---|
| Monster (hai mùa) | 61 | 368 / 0 | 368 / 0 / 0 | 몬스터모드 (monster) (F3, C), Monster mùa 2 (F3, C), Hệ thống Monster (F3, C) | `FxMonster/Monster` (1022), `FxMonster/MonsterSeason2` (236) |
| Quest / Mission | 18 | 115 / 7 | 95 / 20 / 7 | Nhiệm vụ chơi với máy (F3, C), Hệ thống Nhiệm vụ (F6, C) | `FxContents/DailyMission` (22), `FxDesktop/Quest` (59), `FxStage/Game:Mission` (2), `fx/CAMission` (20), `fx/MissionPong` (10) |
| RPG | 52 | 71 / 0 | 71 / 0 / 0 | RPG (F3, C), Hệ thống RPG (F3, C) | `FxContents/RPG` (134), `FxContents/RPGMyInfo` (29) |
| CA Hero | 1 | 53 / 0 | 26 / 27 / 0 | 크아 히어로 (CA Hero) (F3, C), Hệ thống CAHero (F3, C) | `FxContents/CAHero` (69) |
| Hidden Catch | 30 | 37 / 0 | 11 / 26 / 0 | 히든 캐치 (Hidden Catch) (F3, C), Hệ thống Hidden Catch (F3, C) | `FxHiddenCatch/HiddenCatch` (291), `FxHiddenCatch/Prepare_HC` (12) |
| Màn thưởng (Bonus) | 13 | 43 / 0 | 21 / 22 / 0 | Màn thưởng (F3, C) | `FxMap/Bonus` (16), `FxMap/BonusPrepare` (1) |
| AIBattle | 4 | 4 / 0 | 0 / 4 / 0 | AIBattle (F3, C), Hệ thống AIBattle (F3, C) | `FxContents/AIBattle` (10) |
| Hướng dẫn (Tutorial) | 1 | 1 / 0 | 1 / 0 / 0 | Hướng dẫn (F3, C), Hệ thống Hướng dẫn (F3, C) | `FxStage/Tutorial` (74) |

- Monster: `FxMonster/Monster` và `FxMonster/MonsterSeason2` là hai nhóm tài nguyên. Tiền tố `MonS2` đánh dấu 26 map thuộc 8 họ ứng viên mùa 2. Census chưa gán mùa cho từng map; các họ còn lại chưa xác nhận là mùa 1.
- Quest / Mission: các map `category=mission` đã có trong [F2-0](f2-mode-catalog.md). Census chưa nối các map đó với mode nhiệm vụ chơi với máy hoặc với tài nguyên Quest / Mission. Hệ thống Nhiệm vụ mang đợt F6 theo xếp loại hiện hành; `DailyMissionShop` thuộc sảnh nên không cộng vào số map.
- AIBattle lấy các họ `Fm_AIBtl*` từ nhóm biến thể đấu; Tutorial lấy đúng họ `Tutorial` từ nhóm sảnh. Số mục tài nguyên không được cộng vào số map.

## Dấu vết theo họ map

Cột Lớp dùng hàng `kind=map` của từng họ. Các đợt ở bảng xếp loại được giữ nguyên, kể cả họ dùng để đối chiếu. Cột Tag đếm số tên khác nhau trong phần đọc trọn và phần đọc một phần; tên cụ thể theo họ có trong JSON.

### Monster (hai mùa)

Tag thấy ở map đọc trọn: `Action`, `ActionList`, `ActionNumberMapping`, `Area`, `AreaList`, `BackMp3`, `Box`, `ChatBalloonPosition`, `CollisionAttr`, `FirePosition`, `FromObjNo`, `GageBarPosition`, `GameMapItemAttrTable`, `GamePlayAreaScript`, `HPAnger`, `HPMax`, `InitInfo`, `Item`, `ItemSkillAttrTable`, `ItemTable`, `ItemTables`, `Life`, `List`, `MapItemConfig`, `MapObject`, `MapUIScript`, `ObjCollisionSize`, `ObjName`, `ObjNo`, `ObjSize`, `OverrideInitInfo`, `PassType`, `PuzzleBox`, `PuzzleBoxScript`, `RankPoint`, `ScriptObject`, `ScriptObjectTemplate`, `ScriptObjectTemplates`, `ScriptObjects`, `Stage`, `StageClearScript`, `StageInfo`, `StageOption`, `StartPosition`, `Time`, `TimeActionTable`, `ToObjNo`, `TotalTime`, `Trigger`, `TriggerAction`, `TriggerTable`, `UIData`, `UILayer`, `UIObject`, `UIOption`, `action`, `bomb`, `camerascript`, `ground`, `mapscript`, `mapscriptoption`, `mode`, `move`, `state`, `time`, `wave`.

| Họ | Map exact / partial | Script có / không / chưa rõ | Tag exact / partial | Lớp (đợt) |
|---|---:|---:|---:|---|
| `Fm_MonGolem` | 3 / 0 | 3 / 0 / 0 | 12 / 0 | C (F3) |
| `Fm_MonSeal` | 3 / 0 | 3 / 0 / 0 | 11 / 0 | C (F3) |
| `Fm_MonSnowBear` | 3 / 0 | 3 / 0 / 0 | 16 / 0 | C (F3) |
| `Fm_MonTurtle` | 3 / 0 | 3 / 0 / 0 | 12 / 0 | C (F3) |
| `Fm_Monster` | 3 / 0 | 3 / 0 / 0 | 11 / 0 | C (F3) |
| `MonCity` | 9 / 0 | 9 / 0 / 0 | 11 / 0 | C (F3) |
| `MonGolem` | 27 / 0 | 27 / 0 / 0 | 12 / 0 | C (F3) |
| `MonNoviceGolem` | 2 / 0 | 2 / 0 / 0 | 12 / 0 | C (F3) |
| `MonNoviceSnow` | 2 / 0 | 2 / 0 / 0 | 16 / 0 | C (F3) |
| `MonNoviceTako` | 2 / 0 | 2 / 0 / 0 | 11 / 0 | C (F3) |
| `MonNoviceTurtle` | 2 / 0 | 2 / 0 / 0 | 12 / 0 | C (F3) |
| `MonPen` | 34 / 0 | 34 / 0 / 0 | 12 / 0 | C (F3) |
| `MonRobot` | 27 / 0 | 27 / 0 / 0 | 14 / 0 | C (F3) |
| `MonS2Fire` | 15 / 0 | 15 / 0 / 0 | 28 / 0 | C (F3) |
| `MonS2Sand` | 5 / 0 | 5 / 0 / 0 | 29 / 0 | C (F3) |
| `MonS2Sand01Boss` | 1 / 0 | 1 / 0 / 0 | 31 / 0 | C (F3) |
| `MonS2Sand02Boss` | 1 / 0 | 1 / 0 / 0 | 31 / 0 | C (F3) |
| `MonS2Sand03Boss` | 1 / 0 | 1 / 0 / 0 | 31 / 0 | C (F3) |
| `MonS2Sand04Boss` | 1 / 0 | 1 / 0 / 0 | 31 / 0 | C (F3) |
| `MonS2Sand05Boss` | 1 / 0 | 1 / 0 / 0 | 31 / 0 | C (F3) |
| `MonS2Sand06Boss` | 1 / 0 | 1 / 0 / 0 | 30 / 0 | C (F3) |
| `MonSeal` | 37 / 0 | 37 / 0 / 0 | 17 / 0 | C (F3) |
| `MonSnowBear` | 30 / 0 | 30 / 0 / 0 | 19 / 0 | C (F3) |
| `MonTurtle` | 27 / 0 | 27 / 0 / 0 | 12 / 0 | C (F3) |
| `MonUnlimited` | 10 / 0 | 10 / 0 / 0 | 17 / 0 | C (F3) |
| `MonWarsBBR` | 1 / 0 | 1 / 0 / 0 | 11 / 0 | C (F3) |
| `MonWarsBG` | 4 / 0 | 4 / 0 / 0 | 17 / 0 | C (F3) |
| `MonWarsBGB` | 1 / 0 | 1 / 0 / 0 | 12 / 0 | C (F3) |
| `MonWarsBLB` | 1 / 0 | 1 / 0 / 0 | 12 / 0 | C (F3) |
| `MonWarsBM` | 4 / 0 | 4 / 0 / 0 | 11 / 0 | C (F3) |
| `MonWarsBOB` | 1 / 0 | 1 / 0 / 0 | 16 / 0 | C (F3) |
| `MonWarsBPB` | 1 / 0 | 1 / 0 / 0 | 11 / 0 | C (F3) |
| `MonWarsBR` | 4 / 0 | 4 / 0 / 0 | 12 / 0 | C (F3) |
| `MonWarsBS` | 2 / 0 | 2 / 0 / 0 | 11 / 0 | C (F3) |
| `MonWarsBSB` | 1 / 0 | 1 / 0 / 0 | 11 / 0 | C (F3) |
| `MonWarsBT` | 4 / 0 | 4 / 0 / 0 | 11 / 0 | C (F3) |
| `MonWarsBTB` | 1 / 0 | 1 / 0 / 0 | 12 / 0 | C (F3) |
| `MonWarsHGB` | 1 / 0 | 1 / 0 / 0 | 12 / 0 | C (F3) |
| `MonWarsHH` | 2 / 0 | 2 / 0 / 0 | 10 / 0 | C (F3) |
| `MonWarsHLB` | 1 / 0 | 1 / 0 / 0 | 12 / 0 | C (F3) |
| `MonWarsHM` | 2 / 0 | 2 / 0 / 0 | 10 / 0 | C (F3) |
| `MonWarsHOB` | 1 / 0 | 1 / 0 / 0 | 11 / 0 | C (F3) |
| `MonWarsHP` | 5 / 0 | 5 / 0 / 0 | 10 / 0 | C (F3) |
| `MonWarsHPB` | 1 / 0 | 1 / 0 / 0 | 11 / 0 | C (F3) |
| `MonWarsHR` | 2 / 0 | 2 / 0 / 0 | 11 / 0 | C (F3) |
| `MonWarsHS` | 5 / 0 | 5 / 0 / 0 | 11 / 0 | C (F3) |
| `MonWarsHSB` | 1 / 0 | 1 / 0 / 0 | 11 / 0 | C (F3) |
| `MonWarsHTB` | 1 / 0 | 1 / 0 / 0 | 12 / 0 | C (F3) |
| `MonWarsNG` | 3 / 0 | 3 / 0 / 0 | 12 / 0 | C (F3) |
| `MonWarsNGB` | 1 / 0 | 1 / 0 / 0 | 12 / 0 | C (F3) |
| `MonWarsNLB` | 1 / 0 | 1 / 0 / 0 | 12 / 0 | C (F3) |
| `MonWarsNM` | 3 / 0 | 3 / 0 / 0 | 10 / 0 | C (F3) |
| `MonWarsNOB` | 1 / 0 | 1 / 0 / 0 | 11 / 0 | C (F3) |
| `MonWarsNP` | 3 / 0 | 3 / 0 / 0 | 10 / 0 | C (F3) |
| `MonWarsNPB` | 1 / 0 | 1 / 0 / 0 | 11 / 0 | C (F3) |
| `MonWarsNR` | 3 / 0 | 3 / 0 / 0 | 12 / 0 | C (F3) |
| `MonWarsNS` | 3 / 0 | 3 / 0 / 0 | 11 / 0 | C (F3) |
| `MonWarsNSB` | 1 / 0 | 1 / 0 / 0 | 11 / 0 | C (F3) |
| `MonWarsNT` | 3 / 0 | 3 / 0 / 0 | 11 / 0 | C (F3) |
| `MonWarsNTB` | 1 / 0 | 1 / 0 / 0 | 12 / 0 | C (F3) |
| `Monster` | 47 / 0 | 47 / 0 / 0 | 39 / 0 | C (F3) |

### Quest / Mission

Tag thấy ở map đọc trọn: `Action`, `ActionList`, `AppearActionList`, `AppearPosList`, `CatchTailScript`, `ColoringPattern`, `CustomTrigger`, `CustomTriggerTable`, `Domain`, `Domains`, `InitInfo`, `InitInfos`, `Item`, `ItemSkillAttrTable`, `ItemTable`, `ItemTables`, `MapConfig`, `MapItemConfig`, `MapScriptOption`, `Match`, `Matches`, `MissionGoal`, `Pos`, `RabbitCatchScript`, `ScriptObject`, `ScriptObjects`, `SmallBird`, `StageClearScript`, `Sweeping`, `Time`, `Trigger`, `TriggerTable`, `WriggleScript`, `ZzolmanScript`, `action`, `bomb`, `mapscript`, `mapscriptoption`, `snowyfieldscript`, `snowynpc`, `snowynpcs`, `state`, `time`, `trigger`, `wave`.

| Họ | Map exact / partial | Script có / không / chưa rõ | Tag exact / partial | Lớp (đợt) |
|---|---:|---:|---:|---|
| `Baseball` | 10 / 0 | 10 / 0 / 0 | 5 / 0 | C (F2?) |
| `Bula` | 0 / 7 | 0 / 0 / 7 | 0 / 0 | C (F2?) |
| `CAPong` | 5 / 0 | 0 / 5 / 0 | 0 / 0 | C (F2?) |
| `Clover` | 6 / 0 | 6 / 0 / 0 | 5 / 0 | C (F2?) |
| `Coloring` | 5 / 0 | 5 / 0 / 0 | 2 / 0 | C (F2?) |
| `Gumiho` | 11 / 0 | 11 / 0 / 0 | 2 / 0 | C (F2?) |
| `Hamster` | 8 / 0 | 8 / 0 / 0 | 12 / 0 | C (F2?) |
| `Kart` | 8 / 0 | 0 / 8 / 0 | 0 / 0 | C (F2?) |
| `PopcadeA` | 5 / 0 | 5 / 0 / 0 | 21 / 0 | C (F2?) |
| `PopcadeB` | 5 / 0 | 5 / 0 / 0 | 21 / 0 | C (F2?) |
| `PopcadeC` | 5 / 0 | 5 / 0 / 0 | 21 / 0 | C (F2?) |
| `Rabbit` | 6 / 0 | 6 / 0 / 0 | 12 / 0 | C (F2?) |
| `RescueET` | 3 / 0 | 3 / 0 / 0 | 16 / 0 | C (F2?) |
| `SnowyFields` | 18 / 0 | 13 / 5 / 0 | 29 / 0 | C (F2?) |
| `SuperFighter` | 2 / 0 | 0 / 2 / 0 | 0 / 0 | C (F2?) |
| `Wriggle` | 7 / 0 | 7 / 0 / 0 | 20 / 0 | C (F2?) |
| `Zombie` | 6 / 0 | 6 / 0 / 0 | 2 / 0 | C (F2?) |
| `Zzolman` | 5 / 0 | 5 / 0 / 0 | 21 / 0 | C (F2?) |

### RPG

Tag thấy ở map đọc trọn: `Action`, `ActionList`, `InitInfo`, `MonDieRewardProb`, `Move00`, `Move01`, `MoveMap`, `OverrideActionList`, `OverrideInitInfo`, `OverrideTriggerTable`, `RPGScript`, `ScriptObject`, `ScriptObjectTemplate`, `ScriptObjectTemplates`, `StageClearScript`, `Time`, `Trigger`, `TriggerTable`, `action`, `mapscript`, `mapscriptoption`.

| Họ | Map exact / partial | Script có / không / chưa rõ | Tag exact / partial | Lớp (đợt) |
|---|---:|---:|---:|---|
| `RPG` | 1 / 0 | 1 / 0 / 0 | 15 / 0 | C (F3) |
| `RPG01_B` | 1 / 0 | 1 / 0 / 0 | 10 / 0 | C (F3) |
| `RPG01_N` | 1 / 0 | 1 / 0 / 0 | 11 / 0 | C (F3) |
| `RPG01_S` | 1 / 0 | 1 / 0 / 0 | 10 / 0 | C (F3) |
| `RPG02_B` | 1 / 0 | 1 / 0 / 0 | 18 / 0 | C (F3) |
| `RPG02_N` | 1 / 0 | 1 / 0 / 0 | 11 / 0 | C (F3) |
| `RPG02_S` | 1 / 0 | 1 / 0 / 0 | 10 / 0 | C (F3) |
| `RPG03_B` | 1 / 0 | 1 / 0 / 0 | 10 / 0 | C (F3) |
| `RPG03_N` | 1 / 0 | 1 / 0 / 0 | 11 / 0 | C (F3) |
| `RPG03_S` | 1 / 0 | 1 / 0 / 0 | 10 / 0 | C (F3) |
| `RPG04_B` | 1 / 0 | 1 / 0 / 0 | 10 / 0 | C (F3) |
| `RPG04_N` | 2 / 0 | 2 / 0 / 0 | 11 / 0 | C (F3) |
| `RPG04_S` | 1 / 0 | 1 / 0 / 0 | 10 / 0 | C (F3) |
| `RPG0501_B` | 1 / 0 | 1 / 0 / 0 | 10 / 0 | C (F3) |
| `RPG0501_N` | 2 / 0 | 2 / 0 / 0 | 11 / 0 | C (F3) |
| `RPG0501_S` | 1 / 0 | 1 / 0 / 0 | 10 / 0 | C (F3) |
| `RPG05_B` | 1 / 0 | 1 / 0 / 0 | 10 / 0 | C (F3) |
| `RPG05_N` | 2 / 0 | 2 / 0 / 0 | 11 / 0 | C (F3) |
| `RPG05_S` | 1 / 0 | 1 / 0 / 0 | 10 / 0 | C (F3) |
| `RPG0601_B` | 1 / 0 | 1 / 0 / 0 | 10 / 0 | C (F3) |
| `RPG0601_N` | 2 / 0 | 2 / 0 / 0 | 11 / 0 | C (F3) |
| `RPG0601_S` | 1 / 0 | 1 / 0 / 0 | 10 / 0 | C (F3) |
| `RPG06_B` | 1 / 0 | 1 / 0 / 0 | 10 / 0 | C (F3) |
| `RPG06_N` | 2 / 0 | 2 / 0 / 0 | 11 / 0 | C (F3) |
| `RPG06_S` | 1 / 0 | 1 / 0 / 0 | 10 / 0 | C (F3) |
| `RPG0701_B` | 1 / 0 | 1 / 0 / 0 | 10 / 0 | C (F3) |
| `RPG0701_N` | 2 / 0 | 2 / 0 / 0 | 11 / 0 | C (F3) |
| `RPG0701_S` | 1 / 0 | 1 / 0 / 0 | 10 / 0 | C (F3) |
| `RPG07_B` | 1 / 0 | 1 / 0 / 0 | 10 / 0 | C (F3) |
| `RPG07_N` | 2 / 0 | 2 / 0 / 0 | 11 / 0 | C (F3) |
| `RPG07_S` | 1 / 0 | 1 / 0 / 0 | 10 / 0 | C (F3) |
| `RPG0801_B` | 1 / 0 | 1 / 0 / 0 | 10 / 0 | C (F3) |
| `RPG0801_N` | 3 / 0 | 3 / 0 / 0 | 11 / 0 | C (F3) |
| `RPG0801_S` | 1 / 0 | 1 / 0 / 0 | 10 / 0 | C (F3) |
| `RPG08_B` | 1 / 0 | 1 / 0 / 0 | 10 / 0 | C (F3) |
| `RPG08_N` | 3 / 0 | 3 / 0 / 0 | 11 / 0 | C (F3) |
| `RPG08_S` | 1 / 0 | 1 / 0 / 0 | 10 / 0 | C (F3) |
| `RPG0901_B` | 1 / 0 | 1 / 0 / 0 | 10 / 0 | C (F3) |
| `RPG0901_N` | 2 / 0 | 2 / 0 / 0 | 11 / 0 | C (F3) |
| `RPG0901_S` | 1 / 0 | 1 / 0 / 0 | 10 / 0 | C (F3) |
| `RPG09_B` | 1 / 0 | 1 / 0 / 0 | 10 / 0 | C (F3) |
| `RPG09_N` | 3 / 0 | 3 / 0 / 0 | 11 / 0 | C (F3) |
| `RPG09_S` | 1 / 0 | 1 / 0 / 0 | 10 / 0 | C (F3) |
| `RPG1001_B` | 1 / 0 | 1 / 0 / 0 | 15 / 0 | C (F3) |
| `RPG1001_N` | 2 / 0 | 2 / 0 / 0 | 16 / 0 | C (F3) |
| `RPG1001_S` | 1 / 0 | 1 / 0 / 0 | 15 / 0 | C (F3) |
| `RPG10_B` | 1 / 0 | 1 / 0 / 0 | 10 / 0 | C (F3) |
| `RPG10_N` | 3 / 0 | 3 / 0 / 0 | 11 / 0 | C (F3) |
| `RPG10_S` | 1 / 0 | 1 / 0 / 0 | 10 / 0 | C (F3) |
| `RPG11_B` | 1 / 0 | 1 / 0 / 0 | 18 / 0 | C (F3) |
| `RPG11_N` | 3 / 0 | 3 / 0 / 0 | 16 / 0 | C (F3) |
| `RPG11_S` | 1 / 0 | 1 / 0 / 0 | 15 / 0 | C (F3) |

### CA Hero

Tag thấy ở map đọc trọn: `mapscript`, `mapscriptoption`.

| Họ | Map exact / partial | Script có / không / chưa rõ | Tag exact / partial | Lớp (đợt) |
|---|---:|---:|---:|---|
| `CAhero` | 53 / 0 | 26 / 27 / 0 | 2 / 0 | C (F3) |

### Hidden Catch

Tag thấy ở map đọc trọn: `Action`, `ActionList`, `Domain`, `Domains`, `InitInfo`, `Item`, `ItemTable`, `ItemTables`, `MapScriptOption`, `Match`, `Matches`, `MissionGoal`, `RabbitCatchScript`, `ScriptObject`, `ScriptObjects`, `SmallBird`, `StageClearScript`, `Sweeping`, `Time`, `Trigger`, `TriggerTable`, `WriggleScript`, `action`, `bomb`, `ground`, `mapscript`, `mapscriptoption`, `mp3`, `state`, `time`, `trigger`, `wave`.

| Họ | Map exact / partial | Script có / không / chưa rõ | Tag exact / partial | Lớp (đợt) |
|---|---:|---:|---:|---|
| `Hdn_Alche_Aqua` | 1 / 0 | 1 / 0 / 0 | 14 / 0 | C (F3) |
| `Hdn_Alche_Cam` | 1 / 0 | 0 / 1 / 0 | 0 / 0 | C (F3) |
| `Hdn_Alche_Ceme` | 1 / 0 | 0 / 1 / 0 | 0 / 0 | C (F3) |
| `Hdn_Alche_Fac` | 1 / 0 | 0 / 1 / 0 | 0 / 0 | C (F3) |
| `Hdn_Alche_Kart` | 1 / 0 | 0 / 1 / 0 | 0 / 0 | C (F3) |
| `Hdn_Alche_Rab` | 1 / 0 | 1 / 0 / 0 | 17 / 0 | C (F3) |
| `Hdn_Alice` | 1 / 0 | 0 / 1 / 0 | 0 / 0 | C (F3) |
| `Hdn_BlackLucy` | 1 / 0 | 0 / 1 / 0 | 0 / 0 | C (F3) |
| `Hdn_Caffe` | 1 / 0 | 0 / 1 / 0 | 0 / 0 | C (F3) |
| `Hdn_Camp` | 1 / 0 | 0 / 1 / 0 | 0 / 0 | C (F3) |
| `Hdn_Clover` | 1 / 0 | 1 / 0 / 0 | 5 / 0 | C (F3) |
| `Hdn_DarkCastle` | 1 / 0 | 1 / 0 / 0 | 2 / 0 | C (F3) |
| `Hdn_Factory` | 1 / 0 | 0 / 1 / 0 | 0 / 0 | C (F3) |
| `Hdn_Ice` | 1 / 0 | 0 / 1 / 0 | 0 / 0 | C (F3) |
| `Hdn_Mine` | 2 / 0 | 0 / 2 / 0 | 0 / 0 | C (F3) |
| `Hdn_Ocean` | 1 / 0 | 0 / 1 / 0 | 0 / 0 | C (F3) |
| `Hdn_Patrit` | 2 / 0 | 0 / 2 / 0 | 0 / 0 | C (F3) |
| `Hdn_R_Camp` | 1 / 0 | 0 / 1 / 0 | 0 / 0 | C (F3) |
| `Hdn_R_Factory` | 1 / 0 | 0 / 1 / 0 | 0 / 0 | C (F3) |
| `Hdn_R_Forest` | 1 / 0 | 0 / 1 / 0 | 0 / 0 | C (F3) |
| `Hdn_R_Patrit` | 1 / 0 | 0 / 1 / 0 | 0 / 0 | C (F3) |
| `Hdn_R_Village` | 1 / 0 | 0 / 1 / 0 | 0 / 0 | C (F3) |
| `Hdn_Smash` | 1 / 0 | 0 / 1 / 0 | 0 / 0 | C (F3) |
| `Hdn_Square` | 1 / 0 | 0 / 1 / 0 | 0 / 0 | C (F3) |
| `Hdn_SuperFight` | 1 / 0 | 0 / 1 / 0 | 0 / 0 | C (F3) |
| `Hdn_Tomb` | 1 / 0 | 0 / 1 / 0 | 0 / 0 | C (F3) |
| `Hdn_Village` | 1 / 0 | 0 / 1 / 0 | 0 / 0 | C (F3) |
| `Hdn_Wriggle` | 1 / 0 | 1 / 0 / 0 | 20 / 0 | C (F3) |
| `Hidden` | 6 / 0 | 6 / 0 / 0 | 11 / 0 | C (F3) |
| `HiddenCatchTest` | 1 / 0 | 0 / 1 / 0 | 0 / 0 | C (F3) |

### Màn thưởng (Bonus)

Tag thấy ở map đọc trọn: `Action`, `ActionList`, `BonusMapEvent`, `BonusMapScript`, `GeneralScript`, `GhostPos`, `GlobalScript`, `InitInfo`, `ItemUsePerReward`, `Limit`, `MinCount`, `MinItemPerReward`, `MinItemUseCount`, `MinTotalItemCount`, `PresentBoxLimit`, `RewardItem`, `RewardItemList`, `ScriptObject`, `ScriptObjects`, `SpecialItem`, `StageClearScript`, `Time`, `Trigger`, `TriggerTable`, `UserItemConfig`, `action`, `mapscript`, `mapscriptoption`.

| Họ | Map exact / partial | Script có / không / chưa rõ | Tag exact / partial | Lớp (đợt) |
|---|---:|---:|---:|---|
| `Bonus` | 12 / 0 | 0 / 12 / 0 | 0 / 0 | C (F3) |
| `BonusBig` | 1 / 0 | 0 / 1 / 0 | 0 / 0 | C (F3) |
| `BonusBigRPG` | 2 / 0 | 2 / 0 / 0 | 7 / 0 | C (F3) |
| `BonusBigReward` | 13 / 0 | 13 / 0 / 0 | 13 / 0 | C (F3) |
| `BonusCP_18th` | 1 / 0 | 0 / 1 / 0 | 0 / 0 | C (F3) |
| `BonusCP_LINE` | 1 / 0 | 0 / 1 / 0 | 0 / 0 | C (F3) |
| `BonusCp` | 3 / 0 | 0 / 3 / 0 | 0 / 0 | C (F3) |
| `BonusExp` | 3 / 0 | 0 / 3 / 0 | 0 / 0 | C (F3) |
| `BonusLargeGourd` | 2 / 0 | 2 / 0 / 0 | 16 / 0 | C (F3) |
| `BonusMazeMap` | 2 / 0 | 2 / 0 / 0 | 4 / 0 | C (F3) |
| `BonusPCRoom` | 1 / 0 | 1 / 0 / 0 | 6 / 0 | C (F3) |
| `Bonus_16th` | 1 / 0 | 0 / 1 / 0 | 0 / 0 | C (F3) |
| `Bonus_19th` | 1 / 0 | 1 / 0 / 0 | 3 / 0 | C (F3) |

### AIBattle

Tag thấy ở map đọc trọn: không có.

| Họ | Map exact / partial | Script có / không / chưa rõ | Tag exact / partial | Lớp (đợt) |
|---|---:|---:|---:|---|
| `Fm_AIBtlDesert` | 1 / 0 | 0 / 1 / 0 | 0 / 0 | C (F3) |
| `Fm_AIBtlForest` | 1 / 0 | 0 / 1 / 0 | 0 / 0 | C (F3) |
| `Fm_AIBtlPatrit` | 1 / 0 | 0 / 1 / 0 | 0 / 0 | C (F3) |
| `Fm_AIBtlTomb` | 1 / 0 | 0 / 1 / 0 | 0 / 0 | C (F3) |

### Hướng dẫn (Tutorial)

Tag thấy ở map đọc trọn: `Action`, `ActionList`, `GlobalScript`, `Trigger`, `TriggerTable`, `TutorialScript`, `mapscript`, `time`.

| Họ | Map exact / partial | Script có / không / chưa rõ | Tag exact / partial | Lớp (đợt) |
|---|---:|---:|---:|---|
| `Tutorial` | 1 / 0 | 1 / 0 / 0 | 8 / 0 | C (F3) |

Phép đo tiếp theo: chọn một map có tag trigger và một map đối chứng, đối chiếu điểm đọc trên KR 00967 rồi đo native trong mạng cô lập để xác định bên phát sự kiện và bên áp dụng tác động. Đo riêng AI quái và cấp thưởng trước khi xác định route hay luật phía server.

## Tái lập

```bash
python3 tools/kr_f3_pve_audit.py
python3 tools/kr_f3_pve_audit.py --check
```

Công cụ chỉ đọc hai file JSON metadata. `input_sha256` băm byte gốc của từng input. `catalog_sha256` băm JSON sau khi bỏ `catalog_sha256` và `input_sha256`: UTF-8, khóa sắp tăng, không khoảng trắng và có một LF cuối.
