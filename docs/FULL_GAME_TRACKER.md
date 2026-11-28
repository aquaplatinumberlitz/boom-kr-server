# Theo dõi tiến độ Full-Game (`Pha A` + `Pha B`)

> Bảng theo dõi tổng quan 2 pha hướng tới **Full-Game Parity (`F4 → F2 → F5 → F3 → F6/F7`)** cho `boom-kr-server`, được liên kết trực tiếp từ [PLAN.md#dashboard](PLAN.md#dashboard). Lịch sử PR/SHA chi tiết giữ tại [PLAN.md](PLAN.md) và sổ điều phối cục bộ gitignored `local/runs/codex-orchestrator/STATE.md`.
>
> **Cập nhật gần nhất:** 29/11 — **Waves L–Q (`PR #805`–`#831`) đã merge** (kết hợp khung Pha A + IDA tĩnh `B1`–`B4` + analyzer `#825` + Wine `r3`/`r4` đạt 6 mode F2 `#830` + cổng start co-op `#831`); đang verify tuần tự Wave O/P/Q và chạy **Wave R (`p253`–`p258`)** (`8` làn Codex song song).

## Nguyên tắc 2 Pha (A + B)

- **Pha A — Khung hệ thống (Backend / DB / WebShop / CLI / `kr-wire` tĩnh / Pack):** Đưa toàn bộ API quản trị, Schema 7–9, kiểm định policy fail-closed, `WebShop` read-only + hành động cục bộ, `kr-admin` ops, và bộ xuất map/table tĩnh lên mức hoàn chỉnh, bảo đảm M0–F1 mặc định luôn byte-identical.
- **Pha B — Chơi thật Full-Game trong `CA.exe` (IDA + Wine capture + `room_io.rs` sống + Nghiệm thu client thật):** Xác minh trên `CA.exe` + Wine capture hai chiều, nối wire sống vào lõi phòng (`room_io.rs` / `kr-wire`) và nghiệm thu client thật cho 4 cụm lớn (`B1..B4`). Mọi neo IDA chưa có Wine capture đều giữ nhãn `candidate`/`unmeasured`.

---

## Pha A — Phần khung (`A1..A7`)

| Hạng mục | Phạm vi | Trạng thái sau Wave L | Chốt đã merge / Điểm cần lưu ý |
|---|---|---|---|
| **`A1` (`F4`)** | Kinh tế, Schema 7/8, 9 policy mở rộng, WebShop, `kr-admin`, neo shop | **Gần hoàn tất khung** | `kr-wire` `F4_SHOP_ANCHORS`/`Stage20ProfileSetter` (`PR #805`, `#826`), `kr-admin f4-erase-record`/`f4-upgrade-item` (`PR #808`), `f4-redeem-coupon`/`f4-evaluate-title` (`PR #803`), `F4SecurityKitsRuntime` (`PR #796`) |
| **`A2` (`F2`)** | Danh mục `122` map/`18` họ, `room_modes*`, pack `f2-optin`, tile | **Đã mở rộng lớn** | `room_modes` `15` họ/`97` map + evaluators (`PR #800`, `#806`, `#829`), `F2_MODE_ANCHORS`/`Bula` (`#820`), `Bula` inspect (`#818`), `supported_f2f3_probe` (`#819`), `40` tile thiếu (`#807`), audit tests (`#827`) |
| **`A3` (`F3`)** | `699` map/`180` họ, `F3PveRegistry` sống, WebShop, pack `f3-optin` | **Đã mở rộng lớn** | `CP405 MonAction` + bảng nhảy `0xE5DE9C`/`204`/`132`/`180` (`PR #799`, `#824`), multi-stage & `Owner::open` (`PR #801`, `#812`, `#817`), `/api/v1/f3/pve-status` (`PR #802`), `49` tile thiếu (`#807`) |
| **`A4` (`F5`)** | Schema 9, `account_worker/social`, `choco_chat.rs`, CLI/Web | **Đang khép khung** | Khung core sống qua `#780`/`#787`; đã merge `F5NoteInbox`/couple/plaza runtime snapshot (`PR #815`) và `CHOCOPacket` anchors (`PR #828`); đang mở rộng CLI (`p238`) và live wiring (`p249`) |
| **`A5` (`F6`)** | 19 hệ thống core + 25 mở rộng (trừ `PCRoomBenefit`), `f6_runtime` | **Đang khép khung** | Core live `/api/v1/events-status` + CLI (`#759`/`#777`); đã merge `F6EventStatusSnapshot` + extended claims (`PR #816`); đang wiring `account_serving` (`p249`) & `kr-admin` (`p238`) |
| **`A6` (`F7`)** | Spec `F7`, `kr-wire/custom_map.rs`, `CustomMapStore`, WebShop & CLI | **Gần hoàn tất khung** | Đã thêm `CustomMapStore::policy()` + `GET /api/v1/f7/custom-maps-policy` + panel `textContent` (`PR #813`), `f7-custom-maps-list` (`#786`); đang verify `Type 666` (`p245`) & store ops (`p258`) |
| **`A7` (Pack/DB/UI)** | Bảng dữ liệu `CP404`/`CP405`, `boom-re-l10n`, `kr-admin` | **Đạt nền CI** | `tools/kr_table_decode.py --check` (`PR #783`), `f2-f3-tile-grid-census` [tài liệu](content/f2-f3-tile-grid-census.md), `boom-re` `d41d95d` |

---

## Pha B — Chơi thật Full Game trong `CA.exe` (`B1..B4`)

| Cụm | Nội dung trong `CA.exe` | RE tĩnh (IDA / Pack) | Wine capture & Nối wire sống (`room_io.rs`) | Còn lại để đóng |
|---|---|---|---|---|
| **`B1` (`F4` in-game)** | Shop trong trận, túi đồ `219`/`224`/`225`/`313..317`, request `566`, hiển thị trang phục/trang trí/pet | **Đã xong bước IDA nền** ([f4-shop-ingame-wire-re.md](protocol/f4-shop-ingame-wire-re.md), `PR #809`), liên kết [f4-inventory-cosmetics.md](protocol/f4-inventory-cosmetics.md) | Chưa capture thao tác shop/trang bị thật trong phòng; chưa nối projection mỹ phẩm/pet vào `own291`/`room_io` | Capture thao tác mua/đeo/pet trên Wine -> xác nhận byte action `566` -> nối `kr-wire`/`room_io` -> nghiệm thu 2 client |
| **`B2` (`F2` in-game)** | Mã mode & luật thắng thật cho `Zombie`, `Coloring`, `Hamster`, `Popcade`, `Gumiho`, `SuperFighter`, `Baseball`, `Kart`, `Bula` | **Đã xong `Bula`, `Baseball`/`Kart` tiles & start gate** ([Bula `#810`](protocol/f2-bula-tail-and-mode-wire-re.md), [Baseball/Kart `#822`](protocol/f2-baseball-kart-tiles-re.md), [cổng start `#831`](protocol/f2-f3-coop-room-start-gate-re.md)) | **Wine `r3`/`r4` (`#830`)** đạt **6 mode F2** (`Coloring01`, `Hamster01`, `Gumiho01`, `SuperFighter16`, `Rabbit01`, `Wriggle01`) + analyzer `#825`; `Popcade`/`Zzolman` kẹt ở `r1-ready` do `team=0` (`sub_DCAD80`) | Truy vết nguồn `room_type`/`team=0` (`p256`) -> chạy `wine-f2f3-r5` chọn team 3 -> đóng `Baseball`/`Kart`/`Bula` & nối luật thắng sống |
| **`B3` (`F3` PvE — nặng nhất)** | Bảng nhảy `0xE5DE9C`, giao quyền `MonAction` (`204`/`132`/`180`), spawn quái/boss, chuyển màn và 65 họ map lưới lớn | **Đã xong RE `0xE5DE9C` + `MonAction` + lưới lớn + start gate** ([MonAction `#811`](protocol/f3-pve-monaction-wire-re.md), [65 họ lưới lớn `#823`](protocol/f3-pve-large-grid-re.md), [cổng start `#831`](protocol/f2-f3-coop-room-start-gate-re.md)) | `Mon*` chạy qua `r3`/`r4` nhưng dừng tại `r1-ready` do `team=0` trước `C->S 104`; `terrain` opt-in 35 kích thước (`p250`) chờ verify | Truy vết `room_type`/`team=0` (`p256`) -> chạy co-op `wine-f2f3-r5` bắt `104 -> 121 -> 153 -> 204/132/180` -> nối `MonAction`/boss/lưới lớn |
| **`B4` (`F5`/`F6`/`F7` + `L`/`O1`/`S1`)** | `CHOCOPacket` (chat/bạn bè/bang/quảng trường), sự kiện trong client, load CustomMap vào phòng | **Đã xong RE tĩnh `B4`** ([f5-f6-f7-ingame-wire-re.md](protocol/f5-f6-f7-ingame-wire-re.md), `PR #821`: `CHOCOPacket`, `F6` event opcodes, `F7` Type `666` subtype `19..38`) | Chưa capture socket `CHOCOPacket` sống, packet sự kiện in-game hay tải map tự tạo vào phòng (`kr-wire` neo `F5`/`F6`/`F7` `#828`/`p246`/`p245`) | Capture Wine chat/quảng trường/sự kiện/CustomMap -> nối socket/room & nghiệm thu `L`/`O1`/`S1` |

---

## Theo dõi Wave đang chạy

- **Wave L–Q (`#805`–`#831`):** Đã merge hoàn chỉnh (khung Pha A + RE tĩnh `B1`–`B4` + analyzer `#825` + Wine `r3`/`r4` `6` mode `#830` + cổng start co-op `#831`).
- **Hàng đợi verify & Wave P/Q còn lại:** `p232` (`WebShop`), `p238` (`kr-admin`), `p245` (`F7`), `p246` (`F6`), `p250` (`F3` lưới lớn) đang verify tuần tự; `p248` (`f4_security_kits`) & `p249` (`F5/F6` snapshot) đang chạy.
- **Wave R (`p253`–`p258`):** Đang chạy `8` làn Codex song song gồm đồng bộ tile `Baseball`/`Kart` & `f2-mode-catalog` (`p253`), `adapter.rs` (`p254`), `f3_pve_runtime` (`p255`), IDA nguồn `room_type`/`team=0` (`p256`), IDA projection mỹ phẩm/pet `B1` (`p257`), và `custom_map_store` (`p258`).
