# Ma trận thiết kế F2 mở rộng — KR 00967

Phạm vi gồm **21 họ / 91 map** mới được xếp tạm `F2?` trong PR #713, cùng **9 mục mở rộng**. 18 họ nhiệm vụ cũng có nhãn `F2?` và đã có trong [danh mục F2](f2-mode-catalog.md).

Nguồn phạm vi là [bảng xếp loại](classification.md) và [chỉ đạo bao phủ nội dung](../evidence/owner-directives/full-content-coverage.json). Số liệu map lấy từ [census đã lưu](census.json). Hash của đầu vào và build nằm trong [ma trận JSON](../evidence/content/f2-extended-modes-matrix.json).

## Bất biến thiết kế

- Đưa luật mới vào RoomRules hoặc policy do RoomRules dùng; không thêm nhánh `capacity() == N`, cờ theo sức chứa hay alias theo identity vào lõi phòng.
- Tách số điểm spawn trong pack, sức chứa roster và hình thức thi đấu; không suy ra giá trị nào từ hai giá trị còn lại.
- Giữ wire và quyền quyết định chưa đo ở mức unknown; không dùng mặc định trận thường hoặc ID giao diện để điền chỗ trống.
- Kiểm tra quyền và chuyển trạng thái ở server theo policy; giữ luật sảnh, kênh và giải đấu ở lớp chọn policy trước khi tạo phòng.

## Họ map xếp tạm `F2?`

Tất cả các họ dưới đây ở lớp C, trạng thái `candidate`. Số trong ngoặc là số map ở mỗi mức spawn hoặc kích thước lưới. Cột script ghi số map **có / không / chưa đọc đủ**; cột cần đo đếm map có tag ngoài `mapscript` và `StageClearScript`, chưa xác định quyền thực thi.

Slot `RoomRules.map_policy` là đề xuất cho dữ liệu lưới, layout spawn và cổng thực thi script. Nhóm có tên là liên kết ứng viên theo danh mục và [bảng ModeRules](../../rust/crates/kr-service/src/room_modes/rules.json). Chưa có liên kết ứng viên thì đo chọn map và tạo phòng trước khi gán nhóm.

| Họ | Map | Spawn (số map) | Lưới (số map) | Script | Cần đo script | Slot RoomRules | Nhóm ứng viên | Wire |
|---|---:|---|---|---|---:|---|---|---|
| `Balansa` | 5 | 8 (5) | 15×13 (5); chuẩn | 0 / 5 / 0 | 0 | `RoomRules.map_policy` | — | `unknown` |
| `Baram` | 4 | 8 (4) | 15×13 (4); chuẩn | 0 / 4 / 0 | 0 | `RoomRules.map_policy` | — | `unknown` |
| `BlackLucy` | 9 | 8 (9) | 15×13 (9); chuẩn | 0 / 9 / 0 | 0 | `RoomRules.map_policy` | — | `unknown` |
| `Clovertest` | 3 | 8 (3) | 15×13 (3); chuẩn | 3 / 0 / 0 | 3 | `RoomRules.map_policy` | — | `unknown` |
| `ConfusedFight` | 3 | 12 (3) | 15×13 (3); chuẩn | 1 / 2 / 0 | 1 | `RoomRules.map_policy` | `confused_fight` | `unknown` |
| `DarkCastle` | 9 | 8 (9) | 15×13 (9); chuẩn | 9 / 0 / 0 | 9 | `RoomRules.map_policy` | — | `unknown` |
| `DarkVampire` | 8 | 8 (8) | 15×28 (1), 15×34 (1), 15×35 (2), 15×40 (1), 15×42 (2), 15×44 (1); ngoài chuẩn | 7 / 1 / 0 | 7 | `RoomRules.map_policy` | — | `unknown` |
| `Fm_BigBattle` | 4 | 24 (4) | 15×13 (1), 30×26 (1), 45×26 (1), 45×39 (1); hỗn hợp | 4 / 0 / 0 | 4 | `RoomRules.map_policy` | `big_battle` | `unknown` |
| `Fm_BigBattleH_PreparePlay` | 1 | 24 (1) | 15×13 (1); chuẩn | 0 / 1 / 0 | 0 | `RoomRules.map_policy` | `big_battle` | `unknown` |
| `Fm_BigBattle_PreparePlay` | 1 | 24 (1) | 15×13 (1); chuẩn | 0 / 1 / 0 | 0 | `RoomRules.map_policy` | `big_battle` | `unknown` |
| `GoldButterfly` | 3 | 8 (3) | 15×13 (3); chuẩn | 0 / 3 / 0 | 0 | `RoomRules.map_policy` | — | `unknown` |
| `Hiphopclub` | 1 | 8 (1) | 15×13 (1); chuẩn | 0 / 1 / 0 | 0 | `RoomRules.map_policy` | — | `unknown` |
| `MagNest` | 1 | 8 (1) | 15×13 (1); chuẩn | 1 / 0 / 0 | 1 | `RoomRules.map_policy` | — | `unknown` |
| `NewXmas` | 6 | 8 (6) | 15×13 (6); chuẩn | 6 / 0 / 0 | 6 | `RoomRules.map_policy` | — | `unknown` |
| `OkongOfLegend` | 3 | 2 (3) | 15×13 (3); chuẩn | 0 / 3 / 0 | 0 | `RoomRules.map_policy` | — | `unknown` |
| `Smash` | 4 | 12 (4) | 15×16 (2), 20×13 (1), 20×19 (1); ngoài chuẩn | 0 / 4 / 0 | 0 | `RoomRules.map_policy` | — | `unknown` |
| `Space_TW` | 2 | 8 (2) | 15×13 (2); chuẩn | 2 / 0 / 0 | 2 | `RoomRules.map_policy` | — | `unknown` |
| `UserContest` | 17 | 2 (1), 6 (1), 8 (15) | 15×13 (17); chuẩn | 0 / 17 / 0 | 0 | `RoomRules.map_policy` | — | `unknown` |
| `WhiteAngel` | 2 | 8 (2) | 15×13 (2); chuẩn | 2 / 0 / 0 | 2 | `RoomRules.map_policy` | — | `unknown` |
| `WinningStreak_Colosseum` | 1 | 4 (1) | 15×13 (1); chuẩn | 0 / 1 / 0 | 0 | `RoomRules.map_policy` | — | `unknown` |
| `Wriggletest` | 4 | 4 (4) | 15×13 (4); chuẩn | 4 / 0 / 0 | 4 | `RoomRules.map_policy` | — | `unknown` |

## Hệ thống F2 mở rộng

Tám mục có cờ `roadmap_only` trong bảng xếp loại. Map thưởng Ladder nằm riêng trong [lộ trình F2](../PLAN.md#full-game-roadmap). Các slot mô tả nơi gắn policy khi có đủ bằng chứng. Chưa có bằng chứng gắn Big Brawl 12P với nhóm pack `big_battle` hoặc gắn map thưởng với một map cụ thể trong LadderMapGroup.

| Hệ thống | Phạm vi ứng viên | Slot thiết kế | Wire | Phép đo còn thiếu |
|---|---|---|---|---|
| Đấu loạn 12 người (대난투) | Đấu xếp hạng cá nhân 12 người trên map lớn, có kênh và điểm riêng; chưa có wire và luật điểm. | `RoomRules.roster + map_policy + result` | `unknown` | Đo chọn chế độ, roster 12 người, map được phép và cách chốt điểm cá nhân. |
| Xếp hạng 1:1 theo mùa (크아짱) | Đấu 1:1 tính điểm theo mùa, chỉ 1P; chưa có wire mùa và bảng điểm. | `RoomRules.roster + result; RankingSeason` | `unknown` | Đo ghép 1:1, giới hạn 1P và chuyển mùa; xác định nơi chốt điểm và bảng xếp hạng. |
| Kênh theo loại (채널) | Kênh tân thủ, sự kiện, không item, luyện tập và giải đấu; chưa biết luật vào kênh. | `RoomRules.admission_policy + item_policy; ChannelPolicy` | `unknown` | Đo điều kiện vào từng loại kênh và preset chuyển xuống phòng. |
| Map thưởng Ladder | Mục riêng trong lộ trình F2. Chưa biết map, điều kiện xuất hiện hoặc phần thưởng. | `RoomRules.map_policy + result; LadderPolicy` | `unknown` | Đo cách client nhận lựa chọn map thưởng và điều kiện chuyển sang map đó; đối chiếu kết quả và thưởng với một ván Ladder thường. |
| Luật phòng (노템전, 노샵, 기샵) | Phòng không item, không đồ mua hoặc chỉ đồ cơ bản; chưa biết cờ phòng trên wire. | `RoomRules.item_policy` | `unknown` | Đo từng preset khi tạo hoặc vào phòng; tách quyền mang đồ, dùng đồ và nhặt đồ trong trận. |
| Siêu chủ phòng (슈퍼방장) | Quyền chủ phòng mở rộng như bắt đầu ngay; chưa biết nguồn cấp và giới hạn. | `RoomRules.host_permissions` | `unknown` | Đo nguồn cấp quyền, điều kiện bắt đầu ngay và việc chuyển quyền khi chủ rời phòng. |
| Phòng ưu tiên (슈프림, 앞방 잡기) | Phòng S01–S50 mỗi kênh cho người có quyền; chưa có wire tạo và giữ phòng. | `RoomRules.admission_policy; LobbyRoomDirectory` | `unknown` | Đo quyền đặt chỗ S01–S50 theo kênh, cách giữ và giải phóng chỗ. |
| Giải đấu (토너먼트) | Giải đấu có kênh luyện tập, giới hạn người vừa vô địch và quà; chưa có wire và luật. | `RoomRules.roster + result; TournamentPolicy` | `unknown` | Đo đường vào kênh luyện tập và giải, điều kiện dự giải, tiến vòng và chốt quà. |
| Bỏ phiếu đuổi khỏi phòng (강퇴투표) | Bỏ phiếu đuổi người chơi và chống đuổi; chưa có wire và luật bỏ phiếu. | `RoomRules.moderation_policy` | `unknown` | Đo người có quyền mở và bỏ phiếu, điều kiện thông qua, thời hạn và quyền chống đuổi. |

## Giới hạn bằng chứng và cách kiểm

[Neo IDA F2](../protocol/f2-f5-f3-ida-anchors.md) cung cấp hướng lần từ `MapGroupVisible`, switch `GameType` và RTTI. Cần đo client KR trong mạng cô lập, giữ đối chứng khi đổi từng lựa chọn, rồi xác định payload, bên phát sự kiện, quyền chốt kết quả và bước reset khi phù hợp. Ma trận giữ toàn bộ wire của các dòng ở `unknown`.

SHA-256 nội dung JSON: `b034a4508ed186d938857ddcee8f3a76c389825423522dc733540beddd498d39`. Băm UTF-8 sau khi bỏ trường `matrix_sha256`, sắp khóa tăng, dùng dấu phân cách JSON không kèm khoảng trắng và thêm một LF cuối.

```bash
python3 tools/kr_f2_extended_modes_audit.py --write
python3 tools/kr_f2_extended_modes_audit.py --check
```

Hai lệnh chỉ dùng file trong repo. `--check` kiểm SHA-256 theo byte của cả hai đầu ra và rà dấu hiệu lộ đường dẫn máy, IP nội bộ hoặc thông tin xác thực.
