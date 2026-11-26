# Neo IDA tĩnh KR 00967 cho F2, F5 và F3

Tài liệu này ghi nhận các điểm neo tĩnh (`kr_static/confirmed`) trích từ bản sao cục bộ của DB IDA nguồn KR `00967` (`image base 0x400000`, `kr-full.exe.i64` SHA-256 `6be15a1fccea0837ee95c50893345e9f13dc217b0fa570975b5c284f0d547fd1`, `CA.exe` SHA-256 `a52545c14b1fea423de655c3163d6360b628d61c1cd472b46fdbc8a9ffe15f4e`). Dữ liệu máy gốc đã đối chiếu trong [PROTOCOL](../PROTOCOL.md) và [f2-f5-f3-ida-anchors.json](../evidence/content/f2-f5-f3-ida-anchors.json).

Các kết quả IDA dưới đây là neo tĩnh cho công tác phân tích tiếp theo. Neo tĩnh có thể xác định opcode trong bảng dispatch, nhưng chưa xác nhận đường đi của gói tin trong ván thật.

## 1. F2 — Chế độ đấu (`GameType`, `RebirthToGhost`, RTTI)

Đối chiếu với [danh mục F2-0](../content/f2-mode-catalog.md):

- **`MapGroupVisible` (`0xE04CB0` / `0xE04DD0`):** ghi và đọc các mục 2 byte theo khóa cấu hình `MapGroupVisible`.
- **Bảng `GameType` trong hộp thoại tạo phòng (`sub_C6C080` @ `0xC6C080`):** ánh xạ ID số nguyên của chế độ chơi sang chuỗi giao diện:
  - `0` → `Default`, `10` → `Rehab`, `11` → `Hedgehog`, `12` → `FlagHunt`, `13` → `TimeBomb`, `14`/`17` → `NpcBattle`, `16` → `AIBattle`, `18`/`21`/`25` → `SuperFighter` (qua điều kiện `sub_703830`), `19` → `Popcade`, `20` → `SnowBallFight`, `27` → `IamBoss`, `28` → `HiddenCatch_NormalMode`, `29` → `HiddenCatch_CompetitionMode`, `43` → `IamWriggle`, `100` → `All`.
- **`CGameStage::OnPacket` (`0xE5D3A0`) wire opcode `185` (Hex-Rays `case 165`, `sub_E67470` @ `0xE67470`):** `0xE5D3A0` đánh chỉ mục bảng nhảy `0xE5DE9C` theo `opcode - 20`, rồi nhảy qua `0xE5DDC4`. Wire opcode **`185`** (`0xE5D765` → `0xE67470`) đọc `u8` (`+60`), tra tên nhân vật qua `sub_12403A0`, phát chuỗi `RebirthToGhost` và gọi vtable `+84` qua danh sách thực thể (`this + 27752`); wire opcode `165` đi tới `0xE5D570` → `0xE63260` (handler nhặt vật phẩm). Đính chính này dựa trên ảnh đã unpack có SHA-256 `cf72da112308d9f1ca256df8089594c5f787875310933073bf6bd5ba6f70e63b` (khớp [f2_modes.rs](../../rust/crates/kr-wire/src/kr00967/f2_modes.rs)); chưa có capture sống cho `185`.
- **RTTI gói tin hiện đại:** ghi nhận các namespace `WeeklyLadderPacket`, `ConfusedFightPacket` và lớp gói `PTimebombClient` (`CGameClientPacket`), `PTimebombServer` (`CGameServerPacket`), `PWeeklyLadderUserInfoResult` (`CGameServerPacket`). Giá trị opcode số cụ thể trên wire vẫn là `unknown` cho tới khi đo capture sống.

<a id="f2-runtime-adapter"></a>
### F2-4 — Adapter runtime được bật riêng

Phần này là `design`, có kiểm thử tích hợp Rust. Đặt `"f2_modes": {"runtime_overrides": {"free": "revive"}}` ở cấu hình phòng để gắn adapter vào các phòng `Free` mới có `round_profile` hoặc `player_round` do service quản lý. Khóa `manner` áp dụng tương tự cho `Manner`. `create_overrides` vẫn dùng cổng kiểm native cũ; hai loại override không được dùng cho cùng selector. Adapter dùng roster và bước tải của trận thường với `roomtype=0`; đường mở mode F2 gốc vẫn chưa đo.

Dùng `kr-admin` với lệnh cục bộ `f2-runtime`, cùng `target` do `list-rooms` trả về và số `round` từ trạng thái phòng `active` trong mẫu health. Trường `command` nhận `{"command":"start","round":1,"setup":{"mode":"revive","team_budget":2,"player_budget":1,"duration_ms":500,"score_limit":3}}`, rồi nhận `{"command":"apply","round":1,"sequence":1,"event":{"action":"defeat","slots":[1]}}`. Chuỗi sự kiện bắt đầu từ 1; `advance_to` nhận `milliseconds` tính từ đầu ván. Trong `revive`, `defeat` chuyển người chơi sang ghost và gửi `185` với slot qua hàng đợi của phòng. `respawn` chỉ đổi trạng thái mô phỏng. `timebomb` và `leader_capture` cũng chỉ trả kết quả mô phỏng, không có opcode mới.

Client không có quyền phát sự kiện F2. Kết quả F2 trả về cho điều khiển cục bộ, không phát `175`/`180` hoặc ghi thưởng; dùng `close-room` để đóng phòng thử. Các chế độ chưa có luật phù hợp tiếp tục bị từ chối. Chưa nghiệm thu Wine cho adapter này.

## 2. F5 — Xã hội (`CStage`, `CHOCO`, Bang hội, Cặp đôi)

- **Kiến trúc Chat / Thì thầm (`CChattingStage` & `CHOCOPacket`):**
  - `CChattingStage::OnResConnectToSvr` (`0xD58760`) đọc `u8 ok (+60), u8 (+60), u32 (+68), u32 ip (+68), u16 port (+64)` và gọi `sub_13AD4D0(...)` để kết nối tới server chat riêng (`CHOCOPacket`: `P_CHOCO_CHATMSG`, `P_CHOCO_GUILDCHATMSG`, `P_CHOCO_SHOUTMSG`, `P_CHOCO_WHISPERMSG`, `P_CHOCO_TEAMCHATMSG`, `P_RES_CHOCO_EVENT`).
  - Lệnh gõ chat (`sub_426F00` @ `0x426F00`) đăng ký `/whoami`, `/userinfo`, `/who`, `/where`, `/all`, `/guild`, `/whisper` cùng hàm Nexon Messenger `CNMCRSendChatMessageFunc` (`0xAB8017`) / `CNMCSSendChatMessageFunc` (`0xAB8047`).
- **Handler Bang hội (`CStage::OnPacket` @ `0xD68C90`):**
  - `S->C 467` → `0x6D9340`.
  - `S->C 474` → `0xD769D0` (đọc `str, str, u32, u32, str`; nếu bật chặn mời `sub_122A310(5)`, gửi trả `C->S 633` qua `sub_1164C10(633)`; ngược lại mở `ReceiveGuildInvite`).
  - `S->C 478` → `0xD76DA0` (`RefuseGuildInvite`).
- **Handler Cặp đôi (`CStage::OnPacket` @ `0xD68C90`):**
  - `S->C 257` (`0x128D3C0`), `273` (`0xD87250`), `284` (`0x128D120`), `640` (`0xD88830`), `674` (`0xD7FA60`), kèm `C->S 670` (`0x583530` / `0x583E80`) và `CCoupleManager`/`CCoupleHelper` (`0x1233140`, `0x12843AE`, `0x12844AE`, `0x12867F5`).
- Toàn bộ payload đầy đủ của Kết bạn, Bang hội (tạo/danh sách/giải tán), Xếp hạng và framing `CHOCOPacket` vẫn giữ `unknown`.

## 3. F3 — PvE và quyền thực thi quái/script

Đối chiếu với [danh mục F3-0](../content/f3-pve-catalog.md):

- **Factory script (`0xEBA2D0`):** khởi tạo 15 bộ điều khiển script (`0..2, 4..15` → `0x1013B80..0x1014280`).
- **Handler `CGameStage::OnPacket` (`0xE5D3A0`):** Hex-Rays `case 9` (`0xE6AA90`, quest), `case 80` (`0xE6D730`, script), `case 135` (`0xE67640`, `u8, u8` → `IEatObject`/`IAbilityObject` `+36`), `case 136` (`0xE67770`, `u8, u8` → `+52`), `case 177` (`0xE751F0`, đọc `u32, u8, u32, u32, u8`, nhân 2 trường thời gian với `1000`). **Lưu ý:** các nhãn `case` này xuất từ switch Hex-Rays ở `0xE5D3A0`; cần đối chiếu bảng nhảy `0xE5DE9C` (`opcode - 20`) trên assembly trước khi nối wire codec thật.
- **Bằng chứng quyền chủ phòng cho đòn quái (`RollAttack::MonsterAttackReq`):** `sub_6C1BC0` (`0x6C1BC0`) và `sub_F3B740` (`0xF3B740`, nhánh mặc định `0xF274E0`) kiểm tra `sub_11656A0(this - 46)` (`return *(this + 157)`, cờ host) hoặc `attackSeq` khớp (`*(this + 316) == v5`) trước khi thực thi trạng thái `34..37`. Phân ranh đầy đủ giữa quyền server và client-host cho sinh quái, trừ máu và chốt thưởng qua màn hiện giữ `unknown`.
