# Theo dõi tiến độ Full-Game (`Pha A` + `Pha B`)

> Bảng theo dõi tổng quan 2 pha hướng tới **Full-Game Parity (`F4 → F2 → F5 → F3 → F6/F7`)** cho `boom-kr-server`, được liên kết trực tiếp từ [PLAN.md#dashboard](PLAN.md#dashboard). Lịch sử PR/SHA chi tiết giữ tại [PLAN.md](PLAN.md) và sổ điều phối cục bộ gitignored `local/runs/codex-orchestrator/STATE.md`.
>
> **Cập nhật gần nhất:** 28/11 — **Wave L (`PR #805`–`#808`, `#812`, `#813`) ĐÃ HOÀN TẤT 6/6**; đồng thời **3/3 RE tĩnh IDA Wave M (`#809` B1, `#810` B2, `#811` B3)** đã merge song song với phần khung backend/CLI/WebShop/pack.

## Nguyên tắc 2 Pha (A + B)

- **Pha A — Khung hệ thống (Backend / DB / WebShop / CLI / `kr-wire` tĩnh / Pack):** Đưa toàn bộ API quản trị, Schema 7–9, kiểm định policy fail-closed, `WebShop` read-only + hành động cục bộ, `kr-admin` ops, và bộ xuất map/table tĩnh lên mức hoàn chỉnh, bảo đảm M0–F1 mặc định luôn byte-identical.
- **Pha B — Chơi thật Full-Game trong `CA.exe` (IDA + Wine capture + `room_io.rs` sống + Nghiệm thu client thật):** Xác minh trên `CA.exe` + Wine capture hai chiều, nối wire sống vào lõi phòng (`room_io.rs` / `kr-wire`) và nghiệm thu client thật cho 4 cụm lớn (`B1..B4`). Mọi neo IDA chưa có Wine capture đều giữ nhãn `candidate`/`unmeasured`.

---

## Pha A — Phần khung (`A1..A7`)

| Hạng mục | Phạm vi | Trạng thái sau Wave L | Chốt đã merge / Điểm cần lưu ý |
|---|---|---|---|
| **`A1` (`F4`)** | Kinh tế, Schema 7/8, 9 policy mở rộng, WebShop, `kr-admin`, neo shop | **Gần hoàn tất khung** | `kr-wire` neo shop `F4_SHOP_ANCHORS` (`PR #805`), `kr-admin f4-erase-record`/`f4-upgrade-item` (`PR #808`), `kr-admin f4-redeem-coupon`/`f4-evaluate-title` (`PR #803`), chia sẻ `F4SecurityKitsRuntime` sống (`PR #796`) |
| **`A2` (`F2`)** | Danh mục `122` map/`18` họ, `room_modes*`, pack `f2-optin`, tile | **Đã mở rộng lớn** | `room_modes` đủ `15` họ / `97` map (`PR #800`), `F2ModesStatusSnapshot` (`PR #806`), tile thiếu F2 giảm **`65 → 40`** sau mở `Tile70_Static` (`PR #807`); giữ fail-closed `Baseball`/`Kart`/`Bula` |
| **`A3` (`F3`)** | `699` map/`180` họ, `F3PveRegistry` sống, WebShop, pack `f3-optin` | **Đã mở rộng lớn** | `CP405 MonAction` trong `kr-wire` (`PR #799`), `Active::f3_pve_registry()` + `Owner::open` sống (`PR #801`, `#812`), `/api/v1/f3/pve-status` (`PR #802`), tile thiếu F3 giảm **`74 → 49`** (`PR #807`) |
| **`A4` (`F5`)** | Schema 9, `account_worker/social`, `choco_chat.rs`, CLI/Web | **Đang khép khung** | Khung core sống qua `#780`/`#787`; đang bổ sung `F5NoteInbox`/couple/plaza runtime snapshot ở Wave M (`p229`) và CLI/Web |
| **`A5` (`F6`)** | 19 hệ thống core + 25 mở rộng (trừ `PCRoomBenefit`), `f6_runtime` | **Đang khép khung** | Core live `/api/v1/events-status` + CLI (`#759`/`#777`); đang hoàn thiện `F6EventStatusSnapshot` & claims mở rộng ở Wave M (`p230`) |
| **`A6` (`F7`)** | Spec `F7`, `kr-wire/custom_map.rs`, `CustomMapStore`, WebShop & CLI | **Gần hoàn tất khung** | Đã thêm `CustomMapStore::policy()` + `GET /api/v1/f7/custom-maps-policy` + panel WebShop (`PR #813`); `f7-custom-maps-list` (`#786`) |
| **`A7` (Pack/DB/UI)** | Bảng dữ liệu `CP404`/`CP405`, `boom-re-l10n`, `kr-admin` | **Đạt nền CI** | `tools/kr_table_decode.py --check` (`PR #783`), `f2-f3-tile-grid-census` [tài liệu](content/f2-f3-tile-grid-census.md), `boom-re` `d41d95d` |

---

## Pha B — Chơi thật Full Game trong `CA.exe` (`B1..B4`)

| Cụm | Nội dung trong `CA.exe` | RE tĩnh (IDA / Pack) | Wine capture & Nối wire sống (`room_io.rs`) | Còn lại để đóng |
|---|---|---|---|---|
| **`B1` (`F4` in-game)** | Shop trong trận, túi đồ `219`/`224`/`225`/`313..317`, request `566`, hiển thị trang phục/trang trí/pet | **Đã xong bước IDA nền** ([f4-shop-ingame-wire-re.md](protocol/f4-shop-ingame-wire-re.md), `PR #809`), liên kết [f4-inventory-cosmetics.md](protocol/f4-inventory-cosmetics.md) | Chưa capture thao tác shop/trang bị thật trong phòng; chưa nối projection mỹ phẩm/pet vào `own291`/`room_io` | Capture thao tác mua/đeo/pet trên Wine -> xác nhận byte action `566` -> nối `kr-wire`/`room_io` -> nghiệm thu 2 client |
| **`B2` (`F2` in-game)** | Mã mode & luật thắng thật cho `Zombie`, `Coloring`, `Hamster`, `Popcade`, `Gumiho`, `SuperFighter`, `Baseball`, `Kart`, `Bula` | **Đã xong RE `Bula` + neo F2** ([f2-bula-tail-and-mode-wire-re.md](protocol/f2-bula-tail-and-mode-wire-re.md), `PR #810`: `28` record `group_record_tail`, switch `0xC6C080`, `S->C 185`) | Wine `r1`/`r2` mới load 2P `Zombie04`; `r3` multi-map bị chặn ở `map_profile::supported_f2f3_probe` (đang mở khóa ở Wave N) | Mở rộng `map_profile` -> chạy Wine multi-map qua các họ [F2 catalog](content/f2-mode-catalog.md) -> chốt mã mode + tile `Baseball`/`Kart` -> nối luật thắng sống |
| **`B3` (`F3` PvE — nặng nhất)** | Bảng nhảy `0xE5DE9C`, giao quyền `MonAction` (`204`/`132`/`180`), spawn quái/boss, chuyển màn và 65 họ map lưới lớn | **Đã xong RE `0xE5DE9C` + `MonAction`** ([f3-pve-monaction-wire-re.md](protocol/f3-pve-monaction-wire-re.md), `PR #811`: owner gate `0xB91F60`, full bảng nhảy `opcode=case+20`) | Chưa capture 2 client PvE sống cho `204`/`132`/`180`; lưới mới hỗ trợ `15x13`/`19x17` | Capture 2P PvE xác nhận thuật toán chọn `owner_slot` + body `132`/`180` -> mở rộng lưới lớn & tile -> nối `MonAction`/boss/chuyển màn |
| **`B4` (`F5`/`F6`/`F7` + `L`/`O1`/`S1`)** | `CHOCOPacket` (chat/bạn bè/bang/quảng trường), sự kiện trong client, load CustomMap vào phòng | Đang RE tĩnh IDA (`p233`: `CHOCOPacket`, `F6` event opcodes, `F7` map transfer) trên nền [F6 catalog](content/f6-events-catalog.md) & [F7 spec](content/f7-custom-map-spec.md) | Chưa capture socket `CHOCOPacket` sống, packet sự kiện in-game hay tải map tự tạo vào phòng | Hoàn tất IDA RE `B4` -> capture Wine chat/quảng trường/sự kiện/CustomMap -> nối socket/room & nghiệm thu `L`/`O1`/`S1` |

---

## Theo dõi Wave đang chạy

- **Wave L (`#805`–`#808`, `#812`, `#813`):** Hoàn tất 6/6 PR khung + chuẩn hóa đồng bộ dashboard.
- **Wave M (`#809`–`#811`, `p229`–`p231`):** 3/3 RE tĩnh (`B1`/`B2`/`B3`) đã merge; 3 làn Rust (`p229` F5 runtime, `p230` F6 runtime, `p231` F3 multi-stage & `MonAction` runtime) đã chạy xong và đang verify tuần tự.
- **Wave N (`p232`–`p239`):** Đang chạy 8 làn song song phủ IDA `B4` (`p233`), mở khóa probe `map_profile` (`p234`), `Bula` tail + static tiles (`p235`), `kr-wire` `F4`/`F3`/`F2` (`p236`/`p237`/`p239`), `kr-admin` (`p238`) và `WebShop` (`p232`).
