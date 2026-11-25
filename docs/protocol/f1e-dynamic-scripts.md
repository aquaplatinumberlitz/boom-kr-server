# F1-e Dynamic Map Scripts & Mode Contracts (`kr_static`)

- `status`: `confirmed` for client-side XML factories, host authority gates, and inspected packet builders/handlers; `candidate` / `unknown` items explicitly noted below.
- `source`: `kr_static` on KR `00967` (`kr-full.exe`, image SHA-256 `cf72da112308d9f1ca256df8089594c5f787875310933073bf6bd5ba6f70e63b`, image base `0x400000`).
- JSON evidence: [f1e-dynamic-scripts-ida.json](../evidence/content/f1e-dynamic-scripts-ida.json).
- Related specs: [terrain.md](terrain.md), [load-phase.md](load-phase.md), [f1-aux-opcodes-and-classc.md](f1-aux-opcodes-and-classc.md).

## 1. Script Factories, Mode Component & Dispatch

- **MapScript XML Factory (`0xEBA2D0`, `confirmed`)**:
  - Mode/map gate runs before script init (`if a4 != 130 || sub_E05060(v8) != 0`), then every created script initializes via `vt+16`.
  - Tag mapping (`0x176BDC0..0x176BDF8`):
    - `0` `<time>` -> `0x1013B80` -> `0xF126E0` (`CMapScriptSprinkler`, vtable `0x1770154`)
    - `1` `<stageclearscript>` -> `0x1013C00` -> `0xF16BC0` (`CMapScriptStageClear`)
    - `2` `<rabbitcatchscript>` -> `CMapScriptRabbitCatch`
    - `4` `<squarescript>` -> `CMapScriptSquare`
    - `5` `<lobbysquarescript>` -> `CMapScriptLobbySquare`
    - `6` `<wrigglescript>` -> `0x1013E00` -> `0xF9E7F0` (`CMapScriptWriggle`)
    - `7` `<generalscript>` -> `CMapScriptGeneral`
    - `8` `<rpgscript>` -> `CMapScriptRPG`
    - `9` `<larvascript>` -> `0x690FD0` (`CMapScriptLarva`)
    - `10` `<catchtailscript>` -> `0x62A100` (`CMapScriptCatchTail`)
    - `11` `<changetile>` -> `0x634070` (`CMapScriptChangeTile`, also invokes `sub_978BA0`)
    - `12` `<escapemazescript>` -> `0x6513B0` (`CMapScriptEscapeMaze`)
    - `13` `<bulascript>` -> `0x610760` (`CMapScriptBula`)
    - `14` `<repairminibotscript>` -> `0x6ABA20` (`CMapScriptRepairMinibot`)
    - `15` (numeric case 15) -> `0x10143B0` (`CMapScriptMagNest`)
- **Mode Factory (`0xEEF020`, `confirmed`)**:
  - Selects battle `CModeComponent` instance. Mode `36` constructs `CColoringComponent` (`0x637910`, via `sub_EEC850(36)`), separate from standard battle mode `0` (`CDefaultComponent`).
- **Host / Sender Authority Gate (`confirmed`)**:
  - Script actions guarded by `sub_1240CF0()` (checks local player slot flag `& 2`, i.e., host/captain slot) emit `C->S` reports; non-host clients execute only local visuals/timers or apply `S->C` broadcasts.
  - `sub_E78070(this)` additionally requires `E15110` in `{0, 2, 4, 5}` or `1243C40 == 1` before sending `C->S 228`.
- **S->C Battle Script & Component Dispatch (`0xE5D3A0`, `confirmed`)**:
  - Reads wire opcode `v17 = (**a2)(a2)`. Observed range branches:
    - `v17 <= 305`: subtracts `20` (e.g., `case 156` = wire `176` -> `0xE77D20`; `case 158` = wire `178` -> `0xE77DD0`; `case 176..179` = wire `196..199` -> `0xE73760`, `0xE751F0`, `0xE74410`, `0xE72EC0`).
    - `v17 == 306`: `0xE684E0`.
    - `307..=546`: subtracts `312` (`case 115` = wire `427`). Trong switch `v20`, `case 38` là wire `350` → `0xE6A3C0`, còn `case 43` là wire `355` → `0xE73080`. Cả hai ánh xạ đã được xác nhận tĩnh; `350` còn được kiểm với hai client qua relay shadow trong [p108](#f1e-space-pangland-wine). Chưa xác định `355` có phải phản hồi cho `233` hay không.
    - `v17 == 547`: `0xE69090`.
    - `v17 > 547`: subtracts `548` (`case 36` = wire `584` -> `0xE853D0`).
  - `default` / `LABEL_123` fallback: forwards packet to `CModeComponent::vt+64` (`v21[6941]`), then iterates `CMapScript*::vt+64` (`v21 + 6938`), then falls through to `sub_D5F5A0(a2)` (handling opcodes `7`, `84`, `106..109`, `150`, `151`, `218`, `421`, `428`).

## 2. Dynamic Family Contracts (`F1-e`)

| Family | Maps | Reader / Component (`kr_static`) | Authority | `C->S` Opcodes | `S->C` Dispatch |
| --- | --- | --- | --- | --- | --- |
| `space` | 13 | `CMapScriptChangeTile` (`0x634070`) | Host (`1240CF0` + `E78070`) | `228` (`0xE736C0`) | `350` -> `0xE6A3C0` -> `0x483210` (đo tại [p108](#f1e-space-pangland-wine)) |
| `pangland` | 12 | `CMapScriptSprinkler` (`0xF126E0`, `<time>`) | Host (`1240CF0`) + Local UI | `148` (`0xE733D0`) đã đo; `149` (`CGOTileStaticLucci::0xF158C0`) và `233` (`0xE71C20`) mới xác nhận tĩnh | `159` / `158` nhận trên cả hai client với shadow; phản hồi server KR gốc còn `unknown` |
| `retro_*` | 30 | `CMapActionPopUp` (`0x9EA760`) -> `CMapScriptPopUp` (`0xB2AF70`) | Local fade/render inside `<time>` (`0xF14E20`) | None in `CMapScriptPopUp` (`0xB2B140`) | None in `CMapScriptPopUp`; see `unknown` XML tags below |
| `aqua` | 9 (4 active) | `CMapScriptStageClear` (`0xF16BC0`, `StageClearScriptParser::parseScriptObjectTemplates` `0x73C4D0`) | `CScriptObject` action / host events | `132`, `180` (`CScriptObject` actions) | `176` (`0xE77D20`), `178` (`0xE77DD0`), `196..199`, `584` |
| `wriggle` | 14 | `CMapScriptWriggle` (`0xF9E7F0`, `0xF9EF90`) | `CScriptObject` delegation (no direct `CSendPacket` in `0xF9E700..0xFA4500`) | Via `CScriptObject` (`unknown` full subset) | Via `CMapScriptWriggle` / `CScriptObject` (`vt+36`) & `0xE77D20`/`0xE77DD0` |
| `rescue_et` | 3 | `CMapActionRescueAliens*` (`0x6287A0..0x62A0B0`) / `CMapScriptCatchTail` (`0x62A100`) | `CScriptObject` / host AI | `180` (`sub` in `{0, 3, 4, 9, 10}`) | Shared `CScriptObject` / battle channels (`unknown` exact subset) |
| `coloring` | 1 | `CColoringComponent` (`0x637910`, mode `36`) | Mode component (`0xEEF020`) | `229` (`0x639350`) | `186` (`0x639210`), `351` (`0x639280`) via `0x638340` |

## 3. Per-Family Wire & Tick Details (`kr_static`)

1. **`space` (`CMapScriptChangeTile`, 13 maps, `confirmed`)**:
   - `0x6341A0` parses timed tile change entries keyed by `ms / 30`. `0x634B50` ticks and fires `CMapActionChangeTile::vt+16` (`0x633A70`).
   - Host passes `sub_1240CF0()` and `sub_E78070()`, then `sub_E736C0` emits **`C->S 228`** serialized by `sub_483060`: 4 strings (`+0`, `+24`, `+48`, `+72`), 4 `u32` (`+96..+108`), `u32 count`, `count * (u32, u32)` cell coords, `u8 (+112)`, `u8 (+113)`.
   - Mirror reader **`sub_483210`** is called by **`0xE6A3C0`** (`0xE5D3A0` `v20` `case 38`), which decodes the same struct and triggers `sub_E9E270` when active match state (`sub_E13290 != 0`) holds.

2. **`pangland` (`CMapScriptSprinkler`, 12 maps, `confirmed`)**:
   - `<time>` instantiates `CMapScriptSprinkler` (`0xF126E0`), NOT `CMapScriptPopUp`.
   - `0xF12810` parses `<ms>` events into queue `this+152` sorted by `ms / 30`:
     - `0xF12D90` (`airplane`) -> `0xF15DE0` / `0xF15E90`: host calls `sub_E73610`.
     - `0xF12F10` -> `0xF15F30` / `0xF16030`: host calls `sub_E733D0` -> **`C->S 148`**: `u32, u8 count, u16, u16, u8, u32, u16, count * (u8 row, u8 col)`.
     - `0xF13850` -> `0xF163E0` / `0xF16520`: host calls `sub_E71C20` -> **`C->S 233`**: `string, u16, u16, u8 count, count * (u8 row, u8 col)`.
     - `0xF141E0` (`0xF16910`), `0xF14890` (`ground`, modifies local `tile+104` in `0xF16220`), `0xF14B10` (`0xD1CAD0`), `0xF14C80` (`0xD1D710`), and `0xF14E20` (`CMapActionPopUp` `0x9EA760` -> `0xE7BF00` -> `CMapScriptPopUp` `0xB2AF70`) are client-local.
   - `0xF158C0` là hàm của tile `CGOTileStaticLucci` (RTTI của vtable `0x17703E0`, slot `+0x94`). Hàm chuyển state 1 → 2 khi cờ thuộc tính tile bằng 1. Host (`0x1240CF0`) phát **`C->S 149`** nếu lọc còn đúng một ô đích. Bom `CGOBombSprinkler` có vtable riêng `0x1770248`.

3. **`retro_popup` / `retro_forest` / `retro_village` (30 maps, `confirmed` + `unknown`)**:
   - Client `.rdata` contains `Retro` HUD/monster/audio assets (`Game/RetroBug`, `Game/RetroVirus`, `Wave/Npc_Retro*`, `RetroWriggle/*`), and `<popup>` actions run via `CMapScriptSprinkler::0xF14E20` -> `CMapActionPopUp` -> `CMapScriptPopUp::0xB2B140` (local fade/render via `0x14B7700`/`0x14B77E0`/`0x14B7E50`, zero network I/O).
   - `cloudappearloop`, `retroscript`, and `rideinfo` XML literals are absent as plain ASCII/UTF-8 literals in client `.rdata`/`.text` (`unknown` whether handled by server-only script engine, ignored XML attributes, or renamed parser tags; requires Wine capture).

4. **`aqua` StageClear (9 maps: 4 object-scripted + 5 marker-only, `confirmed`)**:
   - `0xF17BD0` calls `0x73C4D0` (`StageClearScriptParser::parseScriptObjectTemplates`).
   - `0xF18700` ticks `CScriptObject` entries at `this+24` (`vt+20`, occasional local idle roll `sub_15625C2() % 995 < 10`); `0xF176B0` forwards script events to active `CScriptObject::vt+36`.
   - Script objects (`0x629940`, `0xF50DD0..0xF572E0`, `0x1013A40`) send **`C->S 132`** (`u32 E704C0, u16 npc_id, u16 sub in {0,21,27,32,33,35,37}, ...`) and **`C->S 180`**.
   - Incoming **`S->C 176`** (`0xE77D20`) reads `u16 script_id`, matches `sub_EE7360(script) == script_id` (`*(script + 16)`), and dispatches `vt+196`; **`S->C 178`** (`0xE77DD0`) spawns scripted bombs via `sub_E9E090` (`u16 x, u16 y, u16 count`, groups of `u16 type, u16 n, n*(i32, i16, i16)`). `S->C 196..199` handle match/object/stage state.

5. **`wriggle` (14 maps, `confirmed`)**:
   - `0xF9EF90` parses `sweeping`, `domains`, `matches`, `itemtables`, and `scriptobjects`.
   - `0xF9E700..0xFA4500` contains zero direct `CSendPacket` (`0x1164C10`) or `0xE7*` calls; tick (`0xF9F4F0`) and event forwarder (`0xF9F590`) delegate exclusively to `CScriptObject::vt+20` / `vt+36`.

6. **`rescue_et` (3 maps, `confirmed` XML/factory mismatch)**:
   - `Fm_RescueET01..03` ship `<RescueAliensScript>`, whereas client factory `0xEBA2D0` maps `<catchtailscript>` -> `0x62A100` (`CMapScriptCatchTail`), which itself parses `getontheshipsec` / `huntermontype` / `rescueplanetype` at `0x62A9B0`.
   - `CMapActionRescueAliensPattern` / `State` / `Wait` (`0x6287A0..0x62A0B0`) send **`C->S 180`** (`u16 npc_id, u16 sub in {0, 3, 4, 9, 10}, ...`).

7. **`coloring` (`Fm_Coloring03`, 1 map, `confirmed`)**:
   - Runs as `CColoringComponent` (`0x637910`, mode `36` in `0xEEF020`), NOT `CMapScript*` in `0xEBA2D0`.
   - Sends **`C->S 229`** (`0x639350`) and handles **`S->C 186`** (`0x639210`) & **`S->C 351`** (`0x639280`) directly in `0x638340`.

## 4. Known Limits & Runtime Verification Required (`unknown`)

- Xem [p108](#f1e-space-pangland-wine) cho opcode `S->C` trong `v20` và layout đã đo của `C->S 148`. Ngữ nghĩa đầy đủ của `C->S 132`, `148`, `149`, `180`, `229`, `233` vẫn cần đối chiếu trước khi triển khai broadcast.
- `<RescueAliensScript>` vs `<CatchTailScript>` instantiation behavior in live `Fm_RescueET01..03` sessions, plus `Retro*` cloud/ride timing (`cloudappearloop` / `retroscript` / `rideinfo`), must be verified via Wine runtime capture.
- Exact `BigBattle` (`BSBig*`, `19x17`, no `<mapscript>`) vs `F1-e` dynamic room win/loss/result triggers (`S->C 196..199`) remain `unknown` statically for live server state transitions.

<a id="f1e-space-pangland-wine"></a>
## 5. Space và PangLand trên Wine hai client (p108)

`source: kr_runtime`. Hai client KR 00967 dùng bản Wine M8 đã pin, cùng trận thường
(`121` có roomtype 0, A host slot 0, B guest slot 1). Cả hai bản sao dùng nguyên cặp
`FxMap.idx` / `FxMap.idd` gốc, kiểm SHA-256 trước khi chạy. Catalog cục bộ chỉ chiếu
hình học và gửi floor rỗng; binary `kr-service` CP397 giữ nguyên. Xem
[bằng chứng p108](../evidence/item-families/f1e-space-pangland-wine.json) để đối chiếu frame,
offset, CRC, thời gian tính từ `S->C 153` và ảnh hai view.

| Map và chiều | Trạng thái | Kết quả |
| --- | --- | --- |
| `CaJJ_Space02` (167), `C->S 228` | `confirmed` | A phát theo các mốc XML từ 20 đến 32 giây. Body theo `0x483060`: bốn chuỗi có độ dài `u16` theo thứ tự `tilecategory, groundodd, groundeven, changepostfix`; bốn `u32` là `dangertime, dangerAlpha, dangerSpeed, dangerColor`; `u32 count`, các cặp `(u32 x, u32 y)`, rồi hai `u8`. Mốc đầu gửi `(0xffffffff, 0)` cho cả hàng 0, không gửi 15 tọa độ riêng. |
| `CaJJ_Space02`, `S->C 350` | `confirmed` với relay cục bộ | Proxy shadow dùng nguyên body `228` do A phát, đóng lại frame `350` theo sequence và CRC của từng kênh. Cả hai client hiện hàng tile rơi. Ở lượt đối chứng không proxy, binary CP397 đóng channel của A khi nhận `228`; B không hiện hàng 0 rơi. |
| `NXCL_PangLand15` (871), `C->S 148` | `confirmed` | A phát ở đợt `<time ms="2000">`. Body theo `0xE733D0`: `u32` nguồn script (đo `0xffffffff`), `u8 count`, `u16 y`, `u16 x`, `u8 power`, `u32 damage`, `u16 state`, rồi `count * (u8 row, u8 col)`. |
| `PangLand03` (887), `C->S 148` | `confirmed` | A phát ở các đợt XML 12, 16, 20 và 24 giây. Hai đợt 16 và 20 giây dùng chung layout với đợt đầu của `NXCL_PangLand15` và có danh sách nhiều ô. |
| `PangLand03`, `S->C 159` rồi `158` | `confirmed` với relay cục bộ | Đối chứng chỉ chặn `148` thì không thấy bom. Lượt thử chọn một `148` có ô đích `(row 7, col 2)`, chiếu tọa độ và power thành `159` (owner 25, ID thử 901), rồi gửi `158` cùng ID. Cả A và B hiện bom bay tới ô đích và hiện vụ nổ. |
| PangLand, `C->S 149` và `233` | `unknown` trong runtime | Không thấy `149` khi nổ bom sprinkler thử, hoặc khi bom thường nổ cạnh tile mang asset `Lucci00` trong khoảng `begingold`. Sampler chỉ đọc không tìm được instance `CGOTileStaticLucci` khớp ở hai client trong lần quét của lượt đối chứng. Hai script được chọn không có `<item>` bên trong `<time>` để kích writer `233`. |

`source: kr_static`, `status: confirmed` cho ánh xạ dispatcher: `0xE5D426`
trừ `0x138` khỏi opcode. Chỉ số 38 đọc selector 7 tại `0xE5E022`;
entry `0xE5DFB8` trỏ `0xE5D885`, gọi `0xE6A3C0` rồi đến reader
`0x483210`. Chỉ số 43 là `355` (`0xE5E027` → `0xE5DFBC` →
`0xE5DC61` → `0xE73080`); ánh xạ tĩnh này không chứng minh đó là phản
hồi cho `233`. Writer `149` tại `0xF159D4`–`0xF15A5D` ghi
`u16 chọn từ bảng Lucci, u32 x, u32 y, u8 row, u8 col` ở nhánh một ô đích; layout tĩnh của
`233` tại `0xE71C20` là `string itemtablename, u16 x, u16 y, u8 count, count * (u8 row, u8 col)` trên đường XML.
Hai layout đó chưa được đối chiếu trên wire. Reader `159` tại `0xE5E8F0` vào
`0xE14600` với owner 25, dùng slot được `0xE0F490` / `0xE12540` dành cho
`CGOBombSprinkler`. Reader `158` tại `0xE5F450` dùng ID để tìm bom cho lượt nổ.
Không suy chính sách chuyển tiếp của server KR gốc từ các gói shadow.

Kiểm pack gốc KR 00967 (`kr_static`): không tìm thấy `<item>` là con trực tiếp
của `<time>` trong các entry `Map/` của `FxMap`. Phép kiểm bỏ comment và đối
chiếu bằng cả parser thuộc tính lẫn lượt đọc tên thẻ cho script sai cú pháp.
Kết quả chưa đưa ra map gốc để kích writer `233`.

Asset `Tile17_Static/Lucci00` xuất hiện trong cả map PangLand và map ngoài họ này,
nhưng mã asset chưa xác định class runtime. `CMapActionState` đọc `<state>` qua
`0xF141E0`; trong build này hai nhánh `lucci` và `item` từ `0xF16910` gọi
`0xF169A0` và `0xF169C0`, đều trả 1 mà không tạo hay đổi trạng thái tile.
Lần theo binary chưa tìm được cạnh gọi trực tiếp tới constructor
`CGOTileStaticLucci` (`0xF15690`); còn mở khả năng có đường gọi gián tiếp.
Một mốc XML `begingold` kèm asset `Lucci00` chưa đủ để chọn lượt đo `149`.

Writer `233` có hai call site trực tiếp đã xác minh: `CMapActionItem::0xF16520`
gọi tại `0xF165A6`, còn `0x6D8DD0` gọi tại `0x6D8EAF` khi state là 1 và
client là host. Chưa tìm thấy tham chiếu vào `0x6D8DD0` trong các dạng địa chỉ
đã quét, nên chưa xác định được đường kích hoạt thứ hai. Factory tile tĩnh
`0xEBDAD0` có nhánh `CGOTileStaticFishing` và nhánh mặc định `CGOTileStatic`;
cả hai dùng `0xBA14A0` tại slot `+0x94`, khác writer Lucci `0xF158C0`.
Các đường này chưa cung cấp fixture KR gốc để đo thêm `149` hoặc `233`.

<a id="f1e-space-pangland-rust"></a>
## 6. Triển khai opt-in Rust và Adapter cho Space & PangLand (`allow_f1e_dynamic_scripts`)

`source: kr_runtime + emu_choice`, `status: candidate` (bằng chứng tổng hợp trong
[f1e-space-pangland-rust.json](../evidence/rust-service/f1e-space-pangland-rust.json)).

- **Adapter pack & catalog (`tools/kr_pack_export.py`, `map_profile.rs`):** Mở rộng allowlist `f1e-adapter-r1` nhận đủ **13 map `Space`** (`<mapscript><changetile><position/>...</changetile></mapscript>`) và **12 map `PangLand`** (`<ground/>`, `<limititemcount><item/>`, `<time><bomb|ground|mp3|state|wave/></time>`, từ chối `<time><item>` fail-closed).
- **Wire codecs (`kr-wire::kr00967`):** Thêm `space_change_tile228` / `space_change_tile350` (khớp byte-for-byte frame `CaJJ_Space02` từ `p108`) và `sprinkler148` / `sprinkler_spawn159` (`SPRINKLER_BOMB_OWNER = 25`, khớp byte-for-byte frame `pang-159` từ `p108`).
- **Service handler (`room_io.rs`, opt-in `allow_f1e_dynamic_scripts = false` mặc định):**
  - `C->S 228`: Chỉ nhận từ host trong pha `Armed` (trần `512` sự kiện/ván), validate cấu trúc rồi broadcast nguyên body dưới opcode `S->C 350` tới mọi đại diện transport.
  - `C->S 148`: Chỉ nhận từ host trong pha `Armed`, cấp phát `bomb ID` không trùng với `118` của người chơi (`bombs::Ledger::allocate_script_id`), phát ngay `S->C 159` (`owner = 25`) và hẹn `S->C 158` sau `2200 ms` dưới `Guard::ScriptBomb`.
  - Pha `Ending`/`Idle` ghi nhận an toàn (`observe_closed` / `observe_after_abort`) không ngắt kênh; `C->S 149` và `233` tiếp tục giữ đóng cho tới khi có bằng chứng runtime bổ sung. Xem kết quả chạy trực tiếp bên dưới.

Đối chứng trực tiếp trên binary release `ae37ac9` (#638), với opt-in bật và
không dùng proxy shadow (`source: kr_runtime`, `status: confirmed` cho blocker):
A gửi `228` trên `CaJJ_Space02` và `148` trên `PangLand03`; channel của A
kết thúc với `native_stage_guard` trước reply dự kiến. `ingame_tls/room.rs`
chỉ giao opcode cho phòng nếu `room_io::Session::handles` nhận; hàm này chưa liệt kê
`148/228` ở commit đo. Hai nhánh ở `Session::packet` chưa được nghiệm thu qua
đường native TLS. [Bằng chứng p108](../evidence/item-families/f1e-space-pangland-wine.json)
giữ pin build, capture, CRC và ảnh của lượt đối chứng.
Mỗi kênh A/B của p108 có một người chơi local. Với cấu hình này,
`local_command` giữ nguyên body sau dispatch; đường ghi phản hồi native
đóng frame theo opcode và sequence của kênh. Không mở rộng kết luận sang
hai người chơi dùng chung một kênh.

Kiểm ngoại tuyến cho thấy codec Rust nhận các body `148/228` đã bắt. Sau khi
nối dispatch, cần kiểm hai giới hạn của nhánh `148` (`status: candidate` cho
kết quả runtime). Mỗi ô đặt trước hai batch trong inbox của mỗi kênh. Với
phòng hai slot của p108, inbox có 8 chỗ; riêng một gói 6 hoặc 7 ô đã đo ở
`PangLand03` sẽ cần 12 hoặc 14 chỗ, kể cả khi inbox trống.
`NXCL_PangLand15` gửi từng ô và dồn nhiều gói trong chưa đến một chu kỳ poll;
nếu đưa được cả đợt vào hàng đợi, số timer cần giữ đồng thời cũng vượt trần
16 batch pending của kênh có một client. Đây là đối chiếu mã với nhịp capture.
Lần đo native cần phân biệt `room_recipient_queue` khi đặt chỗ với
`room_timer_queue` khi giữ batch, rồi kiểm đủ reply và hủy timer khi đổi vòng.

Kiểm thêm trần 512 sự kiện mỗi ván bằng một lượt dài. Trong XML gốc,
`PangLand03`, `PangLand11` và `PangLand16` đều khai báo tổng số ô bom
vượt trần. Nếu mỗi action phát đủ ô đã khai báo, handler hiện tại sẽ bỏ
gói đầu tiên vượt trần ở các mốc tương ứng 171, 96 và 168 giây. Đây là
dự đoán từ XML và mã; client có thể lọc ô hoặc kết thúc ván sớm hơn.

<a id="f1e-space-pangland-native-wine"></a>
## 7. Space và PangLand trên `kr-service` trực tiếp sau #643

`source: kr_runtime`. Hai client KR 00967 trên Wine, binary release `8fe1301` không sửa code, không proxy shadow,
`allow_f1e_dynamic_scripts` bật và profile xuất bằng `tools/kr_pack_export.py --f1e-map`. Bản sao game dùng cặp
`FxMap` gốc như p108. [Bằng chứng, frame hex, hash và cleanup](../evidence/item-families/f1-full-wine-accept.json).

| Map và chiều | Trạng thái | Kết quả |
| --- | --- | --- |
| `CaJJ_Space02`, `C->S 228` → `S->C 350` | `confirmed` | A gửi 10 gói `228` (từ mốc 20 s). `Session::handles` giao gói cho phòng và server phát 10 gói `350` tới A và B sau khoảng 90 ms, body trùng từng byte; không còn `native_stage_guard` ở `228`. Ảnh hai view cho thấy cùng các hàng ngoài đã rụng. |
| `CaJJ_Space02`, kết thúc ván | `unknown` (bị chặn) | Ở 46,3 s, A nhận `170/sub19` rồi cả A và B gửi `C->S 172` (12 byte). Opcode này chưa có trong `Session::handles` nên server đóng cả hai kênh bằng `native_stage_guard`; ván bị hủy `no_players`. Ý nghĩa của `172` chưa RE. |
| `PangLand03`, `C->S 148` | `unknown` (bị chặn) | Ở mốc XML 12 s, A gửi 8 gói `148`, mỗi gói một ô, trong khoảng 1,2 ms. Server nhận 4 gói; gói kế tiếp làm kênh A kết thúc với `room_recipient_queue`, sau đó kênh B kết thúc với `retired_membership`. B chỉ nhận một `159` owner 25, không có `158`; cả hai client hiện errorcode 10. |

Bản gộp của #643 chỉ gộp các ô **trong một gói** `148`; chuỗi gói một ô ở mốc 12 s vẫn đặt hai batch mỗi gói vào
inbox 8 chỗ của mỗi kênh, khớp dự đoán `room_recipient_queue` ở mục 6. Hai đợt nhiều ô 16/20 s, hủy timer khi
đổi vòng và trần 512 sự kiện chưa đo vì ván dừng ở 12 s.
