# Kết quả preflight và probe Wine 2-client F2/F3 đợt 1 (`r1`)

Tài liệu này ghi nhận kết quả đo thực tế (`r1`) trên client gốc KR `00967` qua Wine 2-client cho các map `F2` và `F3-2`, phối hợp với [thiết kế đo quyền PvE F3-2](f3-pve-authority-design.md) và công cụ [kr_f2_f3_capture_analyzer.py](../../tools/kr_f2_f3_capture_analyzer.py). Dữ liệu JSON có kiểm định ở [f2-f3-wine-probe-r1.json](../evidence/content/f2-f3-wine-probe-r1.json).

## 1. Kết quả preflight tĩnh trên pack KR `00967`

- **Nhóm `15x13` không có script rời, pass `decode_map` và `f1_terrain_rules` (12 map):**
  - `F3`: `CAhero07` (`code 100`), `Fm_AIBtlDesert05` (`code 287`).
  - `F2`: `CAPong01`..`CAPong05`, `SnowyFields04`..`SnowyFields06`, `SnowyFields17`..`SnowyFields18`.
- **Bị allowlist `F1-e` trong `kr-service/src/map_profile.rs` chặn trước bind (dù giải mã lưới `15x13` và terrain đạt):** `MonSeal01`, `SnowyFields07`, `Zombie04`, `CAhero01`, `RescueET01`, `SnowyFields01`, `Clover01`.
- **Nút chặn bố cục/header ngoài lưới chuẩn:** `MonS2Fire03` (`40x10`), `RPG11_N_02` (`25x25`), `Tutorial01`.
- **Nút chặn tile ngoài adapter `F1-a` (mẫu đại diện):** `Baseball01` (`Tile70_Ground/Tile04`), `Coloring01` (`Tile38_Box/NumBlock_05_X`), `SuperFighter16` (`Tile23_Static/Temple01`), `Gumiho01` (`Tile65_Static/Static06`), `Hamster01` (`Tile69_Static/Object10`), `Kart02` (`Tile12_Ground/Race13`).

## 2. Kết quả chạy Wine 2-client trực tiếp (`roomtype=0`, `f1-full-rust-v4`)

| Map | Mã map | Khung TCP | Sự kiện chuẩn hóa | Opcode phân biệt | Kết quả & ghi chú |
|---|---|---|---|---|---|
| `CAhero07` | `100` | `130` | `140` | `40` | Tải arena 2P sạch (`exit 0`, dọn scratch `4.63 GB`); giữ baseline `roomtype=0`. |
| `Fm_AIBtlDesert05` | `287` | `130` | `140` | `40` | Tải arena 2P sạch (`exit 0`, dọn scratch `4.63 GB`); giữ baseline `roomtype=0`. |
| `SnowyFields04` | `1048` | `133` | `143` | `40` | Tải arena 2P sạch (`exit 0`, dọn scratch `4.63 GB`); giữ baseline `roomtype=0`. |

**Kết luận đo đạc:** Cả 3 map tải ổn định trong phòng 2P chuẩn `roomtype=0`, xác nhận cần mở đường tạo phòng theo `GameType`/`roomtype` phi-0 và mở allowlist cho map PvE có script trong `map_profile.rs` để thu được các gói quyền host-vs-server. Mọi cơ chế thắng/thua F2 và quyền PvE giữ `unknown`.
