# P228 — B2: đuôi `Bula` và mã mode F2

Hồ sơ này chốt phần dữ liệu tĩnh của build KR `00967` và ghi rõ phần wire còn thiếu. IDA DB được pin bằng SHA-256 `6be15a1fccea0837ee95c50893345e9f13dc217b0fa570975b5c284f0d547fd1`; `CA.exe` có SHA-256 `a52545c14b1fea423de655c3163d6360b628d61c1cd472b46fdbc8a9ffe15f4e`. Hash bốn pack và toàn bộ byte đuôi nằm trong [evidence JSON](../evidence/content/f2-bula-tail-and-mode-wire-re.json). Nguồn đọc lại là `tools/export_map_profile.py` và `tools/kr_f1e_script_audit.py`; các hash reader cũng được pin trong JSON.

## Đuôi `group_record_tail`

Bảy map `Bula` đều là lưới `15×13`, `version=1000`, và có `script_length=0`. `map_tail_offset` đi qua header, spawn, rectangle, cell buffer, drop và placement; từ offset đó, parser khít EOF theo bố cục sau (little-endian):

```text
i32 group_count
i32 point_count; point_count * (i32 x, i32 y)       # lặp theo nhóm
i32 record_count
lặp record:
  i32 name_byte_length; name_byte_length byte CP949
  u8 x; u8 y; i32 field0; i32 field1; i32 field2
u32 script_length; script_length byte script
```

Phần sau tên dài 14 byte (`u8,u8,3*i32`). Byte tên được giữ nguyên trong `tail_hex`; chuỗi được giải mã CP949 chỉ để đọc. Cấu trúc đã xác nhận không tự cho biết ý nghĩa hình học, ba trường `i32`, thứ tự thực thi, điểm, phần thưởng hoặc quyền server.

| Map | Offset đuôi | Byte đuôi | Nhóm | Bản ghi | Nội dung bản ghi `(name,x,y,[i32])` |
| --- | ---: | ---: | ---: | ---: | --- |
| `Bula00` | 1952 | 160 | 1 | 4 | `OuterSoul(14,12,[0,3,1])`; `InnerSoul(5,2,[2,2,0])`; `InnerSoul(3,8,[2,2,0])`; `InnerSoul(4,4,[0,2,0])` |
| `Bula01` | 1952 | 292 | 3 | 5 | `OuterSoul(0,0,[0,3,3])`; `InnerSoul(0,5,[2,2,1])`; `InnerSoul(14,5,[0,2,2])`; `InnerSoul(3,12,[2,2,0])`; `InnerSoul(11,0,[0,2,0])` |
| `Bula02` | 1952 | 356 | 3 | 5 | `OuterSoul(0,0,[0,3,3])`; `InnerSoul(5,4,[0,2,1])`; `InnerSoul(9,4,[2,2,2])`; `InnerSoul(3,12,[2,2,0])`; `InnerSoul(11,12,[0,2,0])` |
| `Bula03` | 1951 | 204 | 3 | 3 | `OuterSoul(0,0,[0,3,3])`; `InnerSoul(0,4,[0,2,1])`; `InnerSoul(14,6,[0,2,2])` |
| `Bula04` | 1952 | 188 | 1 | 5 | `OuterSoul(0,0,[0,3,1])`; `InnerSoul(2,5,[2,2,0])`; `InnerSoul(12,7,[0,2,0])`; `InnerSoul(0,8,[3,2,0])`; `InnerSoul(14,2,[0,2,0])` |
| `Bula15` | 1952 | 396 | 4 | 4 | `InnerSoul(5,0,[0,2,1])`; `InnerSoul(9,0,[0,2,2])`; `InnerSoul(5,12,[0,2,3])`; `InnerSoul(9,12,[0,2,4])` |
| `Bula16` | 1936 | 140 | 2 | 2 | `OuterSoul(0,0,[0,3,1])`; `OuterSoul(14,0,[0,3,2])` |

Các điểm của từng nhóm và toàn bộ byte `tail_hex`, payload hash, tail hash được pin theo từng map trong JSON. Không có placement đọc được sau phần drop trong bảy map. Đây là `kr_static/confirmed` cho bytes/schema. Neo loader chung của `CA.exe` là `0xEA1EE0 → 0x118FC30` (drop/placement), sau đó gọi slot ảo `+0x64` tại `EA287D`; slot `CGamePlayMap` đi tới reader Soul `0x115C770`, tra tên tại `0x11676E0` rồi chuyển record tới `0x115DCF0`. Call-chain này xác nhận cách client đi từ map loader tới record reader, nhưng chưa xác nhận timing, object, wire hoặc quyền server riêng của từng Bula.

## Neo IDA và F2

- `MapGroupVisible`: writer `0xE04CB0`, reader `0xE04DD0`, khóa cấu hình cùng tên; neo hiện hành mô tả cặp `u16` và chỉ dùng để lần đường chọn map/nhóm.
- `roomtype`: `sub_57F520` tại `0x57F520`, nằm trong đường tạo phòng. Chưa phục hồi được trường số wire từ neo này.
- Switch `GameType`: `sub_C6C080` tại `0xC6C080` cho các nhãn/UI ID `0 Default`, `10 Rehab`, `11 Hedgehog`, `12 FlagHunt`, `13 TimeBomb`, `14/17 NpcBattle`, `16 AIBattle`, `18/21/25 SuperFighter`, `19 Popcade`, `20 SnowBallFight`, `27 IamBoss`, `28/29 HiddenCatch`, `43 IamWriggle`, `100 All`. Đây là `kr_static/confirmed` cho switch/label; không phải mã wire và không gán trực tiếp cho họ map.
- `CGameStage::OnPacket` tại `0xE5D3A0` dùng jump table `0xE5DE9C` theo chỉ số `opcode - 20`. Neo đã xác nhận `S→C 185` tới `0xE67470` (`RebirthToGhost`, đọc `u8` slot và gọi entity vtable `+84`); nhãn Hex-Rays `165` tới `0xE63260` là consumer pickup. Các nhãn `9` (quest), `80` (script), `135/136` (object callback) và `177` (trường thời gian) là neo tĩnh cần binding capture/assembly trước khi gọi là raw wire của mode.
- RTTI có `WeeklyLadderPacket`, `ConfusedFightPacket`, `PTimebombClient`, `PTimebombServer` và `PWeeklyLadderUserInfoResult`; số wire sống và quyền chốt kết quả vẫn `unknown`.

Ma trận 15 họ F2 đã xuất (`CAPong`, `Clover`, `Coloring`, `Gumiho`, `Hamster`, `PopcadeA..C`, `Rabbit`, `RescueET`, `SnowyFields`, `SuperFighter`, `Wriggle`, `Zombie`, `Zzolman`) cùng `Baseball`, `Kart`, `Bula` được giữ trong trường `f2_family_matrix.rows` của JSON. Mỗi họ giữ `wire_mode=unknown` và `round_objective_wire=unknown`; `PopcadeA..C` chỉ có candidate UI ID `19`, còn `SuperFighter` có candidate `18/21/25`. Không dùng tên map, hậu tố số hoặc nhãn UI để tự sinh mã wire.

## Giới hạn runtime

Chưa chạy Wine capture ở lượt này. IDA tĩnh và pack bytes không chứng minh client tải mode, gửi packet, chạy vòng, xử lý objective, chốt thắng/hòa, reset ván kế hoặc tương thích server. Bước tiếp theo phải chạy bản sao offline với outbound allowlist, giữ capture/raw/hash và cleanup; chỉ sau đó mới nâng các hàng `candidate` thành contract wire. Không đưa đường dẫn máy hoặc thông tin hạ tầng riêng vào hồ sơ.
