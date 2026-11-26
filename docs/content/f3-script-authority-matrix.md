# Ma trận quyền thực thi script PvE F3-1

Nguồn `kr_static` của build `KR00967`: [F3-0](../evidence/content/f3-pve-catalog.json),
[neo IDA](../protocol/f2-f5-f3-ida-anchors.md), [census](census.json) và [xếp loại](classification.json).
[Ma trận JSON](../evidence/content/f3-script-authority-matrix.json) giữ hash của cả bốn đầu vào, từng map và từng họ.

Phạm vi: **699 map / 180 họ** (692 `exact`, 7 `partial`); lớp A/B/C theo họ: 0 / 0 / 180.
Script theo census: 593 có, 99 không ghi nhận, 7 chưa rõ.
Census ghi `<mapscript>`; không thấy thẻ literal `<script>` trong phần đã đọc. Ma trận giữ riêng trạng thái của hai thẻ, cờ script và tên tag, trigger, hàm quan sát được.
`exact` là đọc trọn theo parser census. Ở map `partial`, thiếu tag không chứng minh script vắng mặt.
Các nhóm màn là nhãn kiểm kê. Quest / Mission dùng để đối chiếu F2; chưa xác nhận cửa vào hay route của từng map.

[Nguồn tham chiếu](../REFERENCES.md): `boomkr-re/docs/kr-content.md` §5 mô tả tag đã trích từ pack KR; `boom-re/docs/map.md` mô tả blob XML của build VN. Hai nguồn chỉ giúp đối chiếu bố cục dữ liệu.

| Nhóm màn | Phạm vi | Họ | Map exact / partial | Script có / không ghi nhận / chưa rõ |
|---|---|---:|---:|---:|
| Monster (hai mùa) | F3 | 61 | 368 / 0 | 368 / 0 / 0 |
| Quest / Mission | Đối chiếu F2 | 18 | 115 / 7 | 95 / 20 / 7 |
| RPG | F3 | 52 | 71 / 0 | 71 / 0 / 0 |
| CA Hero | F3 | 1 | 53 / 0 | 26 / 27 / 0 |
| Hidden Catch | F3 | 30 | 37 / 0 | 11 / 26 / 0 |
| Màn thưởng (Bonus) | F3 | 13 | 43 / 0 | 21 / 22 / 0 |
| AIBattle | F3 | 4 | 4 / 0 | 0 / 4 / 0 |
| Hướng dẫn (Tutorial) | F3 | 1 | 1 / 0 | 1 / 0 / 0 |

## Điểm quan sát tĩnh và giới hạn

- Factory `0xEBA2D0` tạo 15 controller (case `0..2, 4..15`, dải `0x1013b80..0x1014280`). Chưa nối từng map với controller được tạo.
- `CGameStage::OnPacket` case `9` tại `0xe6aa90`: quest.
- `CGameStage::OnPacket` case `80` tại `0xe6d730`: script.
- `CGameStage::OnPacket` case `135` tại `0xe67640`: đọc hai u8, gọi vtable +36.
- `CGameStage::OnPacket` case `136` tại `0xe67770`: đọc hai u8, gọi vtable +52.
- `CGameStage::OnPacket` case `177` tại `0xe751f0`: đọc u32, u8, u32, u32, u8; hai trường timer nhân 1000.
- `RollAttack::MonsterAttackReq` (`0x6c1bc0`, `0xf3b740`, dispatcher dự phòng `0xf274e0`): kiểm cờ host `0x11656a0` hoặc `attackSeq` khớp trước trạng thái `34..37`.

Chỉ xác nhận điểm quan sát phía client; chưa nối neo với route hay quyền quyết định của từng màn.
Danh sách neo theo nhóm trong JSON là các điểm đặt quan sát ứng viên, chưa phải route của chế độ.
Case 135/136 chỉ là neo quan sát IEatObject/IAbilityObject; case 177 chưa đủ để gán quyền AI.
Guard đòn quái chưa chứng minh bên nào có quyền sinh quái, trừ HP, chạy trigger hoặc cấp thưởng.

| Quyền / route chưa biết | Trạng thái | Phép đo phân biệt |
|---|---|---|
| Bên nào quyết định sinh quái và chuyển đợt? | `unknown` | So thời điểm xuất hiện, mã đối tượng và đợt trên hai client với thông điệp đến/đi. |
| Bên nào quyết định sát thương, HP còn lại và quái chết? | `unknown` | So một lần đánh trúng với lần đánh hụt; ghi HP hoặc trạng thái quan sát được trước và sau, cùng nguồn cập nhật. |
| Bên nào quyết định điều kiện và hiệu ứng của trigger? | `unknown` | Lặp cùng điều kiện khi đổi người tạo phòng; thêm lượt không kích hoạt và so dấu vết trên hai client. |
| Bên nào xác nhận qua màn và bắt đầu chuyển màn? | `unknown` | So lần đủ điều kiện qua màn với lần chưa đủ; ghi bên phát và bên áp dụng kết quả. |
| Bên nào quyết định cấp thưởng và ghi nhận thưởng qua màn? | `unknown` | So lượt được thưởng và lượt không đạt; ghi số dư hoặc túi đồ trước, sau và sau tái đăng nhập nếu có. |
| Bên nào quyết định AI và trạng thái đòn quái? | `unknown` | Đối chiếu cờ host và thứ tự quan sát tại guard đòn quái với chuyển trạng thái phía nhận. |
| Các neo phía client xuất hiện ở route nào trong mỗi màn? | `unknown` | Ghi hướng, thứ tự, mã gói và điểm gọi trong client; chỉ gắn route cho luồng thực sự quan sát được. |

## Mẫu ưu tiên Wine F3-2

Thiết kế thử, chưa chạy. Mỗi nhóm màn có 1–2 mẫu đầu; chỉ mở mẫu bổ sung khi cần phân biệt cụm tag khác.
Mẫu đều `exact` và có vị trí spawn trong census. Điều đó chưa xác nhận màn mở được hoặc chơi được với hai client.
Các cụm mô tả tag quan sát được; mẫu chung một cụm không đại diện cho route của các chế độ khác.
Tiền tố `MonS2` dùng chọn ứng viên mùa 2; không gán mùa 1 cho các họ còn lại.

| Ưu tiên | Nhóm màn | Map | Cụm dấu vết | Mục đích |
|---|---|---|---|---|
| Đầu | Monster (hai mùa) | `MonSeal01` | `stage_clear` | So nhịp sinh quái, một lần thay đổi HP và qua màn khi đảo vai host. |
| Đầu | Monster (hai mùa) | `MonS2Fire03` | `monster_area_ui` | Tách trigger vùng và cập nhật quái khỏi camera/UI cục bộ. |
| Đầu | Quest / Mission | `SnowyFields07` | `stage_clear` | Dò trigger qua màn ở nhóm đối chiếu; chưa biết map dùng cửa vào Quest nào. |
| Đầu | RPG | `RPG11_N_02` | `rpg_transition` | Thử trigger đến gần và theo dõi chuyển bản đồ qua MoveMap. |
| Đầu | CA Hero | `CAhero01` | `script_marker` | Dùng màn có dấu mapscript để dò nguồn cập nhật của chế độ. |
| Đầu | CA Hero | `CAhero07` | `no_script` | Đối chứng cùng họ CAhero, census không ghi script. |
| Đầu | Hidden Catch | `Hdn_Alche_Aqua` | `stage_clear` | So trigger và kết quả với mẫu Hidden Catch không có dấu script. |
| Đầu | Hidden Catch | `Hdn_Alche_Cam` | `no_script` | Đối chứng để tách hiệu ứng của chế độ khỏi script trong map. |
| Đầu | Màn thưởng (Bonus) | `BonusLargeGourd01` | `bonus_event` | So hoàn thành và không hoàn thành để tìm nguồn thông báo thưởng. |
| Đầu | Màn thưởng (Bonus) | `BonusBigRPG01` | `bonus_global` | Đối chiếu cấu hình giới hạn vật phẩm với thưởng quan sát được. |
| Đầu | AIBattle | `Fm_AIBtlDesert05` | `no_script` | Đối chứng AI trên map không có dấu script; theo dõi timer và trạng thái. |
| Đầu | Hướng dẫn (Tutorial) | `Tutorial01` | `tutorial` | Kiểm trigger hướng dẫn; dùng một client nếu cửa vào chỉ hỗ trợ chơi đơn. |
| Bổ sung | Monster (hai mùa) | `MonSnowBear22` | `puzzle_box` | Đo tương tác PuzzleBox nếu mẫu Monster đầu chưa giải thích được trigger. |
| Bổ sung | Quest / Mission | `Baseball01` | `timed_wave` | Dùng mốc thời gian và đợt làm đối chứng script của một màn F2. |
| Bổ sung | Quest / Mission | `Hamster01` | `rabbit_catch` | Theo dõi điều kiện bắt mục tiêu và thông báo hoàn tất. |
| Bổ sung | Quest / Mission | `Wriggle01` | `wriggle` | Theo dõi thay đổi vùng và tác động quan sát được trên mỗi client. |
| Bổ sung | Quest / Mission | `RescueET01` | `catch_tail` | Kiểm controller thực sự được tạo và hành vi khi đạt mục tiêu. |
| Bổ sung | Quest / Mission | `SnowyFields01` | `snowy_field` | Tách cập nhật NPC khỏi các hiệu ứng cục bộ của màn. |
| Bổ sung | Quest / Mission | `PopcadeA01` | `quest_custom` | Theo dõi CustomTrigger ở màn có một vị trí spawn; không ép chạy hai client. |

1. Kiểm lease Wine và cô lập outbound trước khi chạy. Dùng bản sao KR 00967 đã pin hash, chỉ cho phép kết nối tới server và bộ thu cục bộ. Ghi phiên bản Wine, hash bản sao và mọi thay đổi nếu có.
2. Xác minh cửa vào, map mà client thực sự tải và chế độ trên cả hai phía. Nếu cửa vào chưa có, ghi blocker; không dùng ép map hoặc ACK giả làm bằng chứng.
3. Với chế độ hỗ trợ hai người, lấy một lượt A tạo phòng và một lượt B tạo phòng; lặp cùng thao tác và thêm đối chứng không kích hoạt. Màn chỉ chơi đơn chỉ đo hành vi cục bộ, không dùng để kết luận phần chia quyền.
4. Đồng bộ dấu thời gian của hai client, capture cục bộ và phía server. Ghi riêng nguồn phát, bên áp dụng và bên ghi bền. Log emulator chỉ mô tả emulator; kết quả chỉ áp dụng cho luồng đã đo.

SHA-256 nội dung ma trận: `3d3d82b842a0bc65aa8d3e6869dd6175444a85717846c836390f73de86e86ada`. Hash tính trên JSON chuẩn hoá UTF-8, bỏ trường `matrix_sha256`.
Sinh lại: `python3 tools/kr_f3_script_matrix.py`; so sánh hai đầu ra: `python3 tools/kr_f3_script_matrix.py --check`.
