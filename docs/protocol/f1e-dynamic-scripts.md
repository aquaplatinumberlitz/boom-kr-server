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
    - `307..=546`: subtracts `312` (`case 115` = wire `427`). Switch `v20` inside `0xE5D3A0` contains `case 38` -> `0xE6A3C0` and `case 43` -> `0xE73080` (`candidate` wire base `312` -> `350`/`355`, needs confirmation of `v20` entry branch in runtime/follow-up).
    - `v17 == 547`: `0xE69090`.
    - `v17 > 547`: subtracts `548` (`case 36` = wire `584` -> `0xE853D0`).
  - `default` / `LABEL_123` fallback: forwards packet to `CModeComponent::vt+64` (`v21[6941]`), then iterates `CMapScript*::vt+64` (`v21 + 6938`), then falls through to `sub_D5F5A0(a2)` (handling opcodes `7`, `84`, `106..109`, `150`, `151`, `218`, `421`, `428`).

## 2. Dynamic Family Contracts (`F1-e`)

| Family | Maps | Reader / Component (`kr_static`) | Authority | `C->S` Opcodes | `S->C` Dispatch |
| --- | --- | --- | --- | --- | --- |
| `space` | 13 | `CMapScriptChangeTile` (`0x634070`) | Host (`1240CF0` + `E78070`) | `228` (`0xE736C0`) | `0xE6A3C0` -> `0x483210` (`v20` `case 38`, `candidate` wire id) |
| `pangland` | 12 | `CMapScriptSprinkler` (`0xF126E0`, `<time>`) | Host (`1240CF0`) + Local UI | `148` (`0xE733D0`), `149` (`0xF158C0`), `233` (`0xE71C20`) | `unknown` (server relay / room bomb / sprinkler sync via Wine) |
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
   - `CGOBombSprinkler` (`0x1770248`) detonation in `0xF158C0` sends **`C->S 149`** when host (`sub_1240CF0 != 0`), including `u16, u32, u32, ...` when a target cell hits.

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

- Exact `S->C` wire opcode numbers inside `v20` (`case 38` = `0xE6A3C0`, `case 43` = `0xE73080`) and full field semantics for `C->S 132`, `148`, `149`, `180`, `229`, `233` need live KR Wine / Rust test confirmation before server broadcast implementation.
- `<RescueAliensScript>` vs `<CatchTailScript>` instantiation behavior in live `Fm_RescueET01..03` sessions, plus `Retro*` cloud/ride timing (`cloudappearloop` / `retroscript` / `rideinfo`), must be verified via Wine runtime capture.
- Exact `BigBattle` (`BSBig*`, `19x17`, no `<mapscript>`) vs `F1-e` dynamic room win/loss/result triggers (`S->C 196..199`) remain `unknown` statically for live server state transitions.
