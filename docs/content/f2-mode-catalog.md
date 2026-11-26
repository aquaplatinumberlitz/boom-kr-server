# Kiểm kê mode và map nhiệm vụ F2

Nguồn: [kiểm kê map và pack](../CONTENT.md), [bảng Mode](../CONTENT.md#mode), và [xếp loại A/B/C](classification.md).
[Catalog JSON](../evidence/content/f2-mode-catalog.json) giữ tên từng map, số đếm, locator tài nguyên và hash đầu vào.
Kết quả cho KR00967: **122 map / 18 họ**, không trùng tên map.
Census nhận diện script ở **95** map, không thấy ở **20** map đọc trọn và chưa rõ ở **7** map đọc một phần.
Xếp loại sơ bộ ở cấp họ: 0 A, 0 B, 18 C, 0 chưa có xếp loại.

## Metadata tĩnh theo họ

Phạm vi là các hàng `category=mission` do census phân theo tên. Kích thước lưới, số vị trí spawn và `d1/d2` lấy từ header; chúng chưa xác định sức chứa phòng, đội hay mã mode.
Cột Rơi ghi số map có hàng / tổng số hàng / số tên khác nhau có trọng số dương. Cột Đặt sẵn ghi số map có hàng / tổng số hàng, giữ cả tên lặp.
Hai cột này chỉ tính map đọc trọn. JSON giữ riêng các hàng thu được từ map đọc một phần; số 0 ở phần đó chưa chứng minh không có dữ liệu.
Cột Nhóm là nhãn ứng viên từ wiki hoặc tên họ. Lớp tham chiếu hàng `kind=map` của bảng xếp loại, không kế thừa từ lớp mode.

| Họ | Map | Lưới | Spawn | Script có/không/chưa rõ | d1/d2 | Rơi | Đặt sẵn | Lớp | Nhóm ứng viên |
|---|---:|---|---|---|---|---|---|---|---|
| Baseball | 10 | 15×13 | 8 | 10/0/0 | 2/4 | 10/120/0 | 1/95 | C | `baseball` · 프로야구 |
| Bula | 7 | 15×13 | 6–8 | 0/0/7 | 3/2, 4/2 | 0/0/0 | 0/0 | C | `bula` · 불라 |
| CAPong | 5 | 15×13 | 4 | 0/5/0 | 2/2 | 5/95/7 | 0/0 | C | `capong` · 크아퐁 |
| Clover | 6 | 15×13 | 8 | 6/0/0 | 2/4 | 6/72/0 | 0/0 | C | `clover` · 클로버 |
| Coloring | 5 | 15×13 | 4 | 5/0/0 | 2/2 | 5/59/20 | 5/297 | C | `coloring` · 색칠하기 |
| Gumiho | 11 | 15×13 | 8 | 11/0/0 | 8/1, 12/1 | 11/220/10 | 0/0 | C | `gumiho` · 구미호 |
| Hamster | 8 | 15×13 | 8 | 8/0/0 | 4/2 | 8/16/0 | 1/4 | C | `farm_mission` · 햄스터팜 |
| Kart | 8 | 15×13 | 4–8 | 0/8/0 | 8/1 | 8/71/13 | 8/109 | C | `kart` · 카트 |
| PopcadeA | 5 | 15×13 | 1 | 5/0/0 | 1/1 | 5/105/18 | 1/4 | C | `popcade` · 팝케이드 |
| PopcadeB | 5 | 15×13 | 2 | 5/0/0 | 1/2 | 5/105/18 | 1/4 | C | `popcade` · 팝케이드 |
| PopcadeC | 5 | 15×13 | 4 | 5/0/0 | 1/4 | 5/105/18 | 1/4 | C | `popcade` · 팝케이드 |
| Rabbit | 6 | 15×13 | 8 | 6/0/0 | 4/2 | 6/15/3 | 2/28 | C | `farm_mission` · 바니팜 |
| RescueET | 3 | 15×13 | 4 | 3/0/0 | 2/2 | 3/15/5 | 3/30 | C | `rescue_et` · 외계인구출 (đã ngừng) |
| SnowyFields | 18 | 15×13 | 4–8 | 13/5/0 | 1/4, 2/2, 2/4, 8/1 | 17/293/33 | 9/192 | C | `snowy_fields` · 눈싸움 (sự kiện, chưa chắc) |
| SuperFighter | 2 | 15×13 | 8 | 0/2/0 | 2/4 | 2/32/11 | 2/13 | C | `super_fighter` · 슈퍼파이터 (sự kiện) |
| Wriggle | 7 | 15×13 | 4 | 7/0/0 | 2/2 | 7/161/7 | 1/4 | C | `wriggle` · 꼬물이 |
| Zombie | 6 | 15×13 | 8 | 6/0/0 | 8/1 | 6/120/10 | 2/20 | C | `zombie` · 좀비 |
| Zzolman | 5 | 15×13 | 4 | 5/0/0 | 1/4 | 5/105/18 | 1/4 | C | `zzolman` · 쫄맨 (đã ngừng) |

## Chẩn đoán phần đuôi đọc một phần

Chỉ dùng hàng `parse=partial` trong phạm vi mission của census. Dấu — là metadata chưa có.
Số hàng đã đọc và danh sách tag rỗng không xác nhận phần đuôi đã đọc hết hay map không có script.

| Họ / map | Hàng rơi / đặt sẵn đã đọc | Tag đã ghi | Lý do | Tag thiếu | Phần đuôi |
|---|---:|---|---|---|---|
| Bula / `Bula00` | 10/0 | [] | — | — | — |
| Bula / `Bula01` | 10/0 | [] | — | — | — |
| Bula / `Bula02` | 10/0 | [] | — | — | — |
| Bula / `Bula03` | 11/0 | [] | — | — | — |
| Bula / `Bula04` | 10/0 | [] | — | — | — |
| Bula / `Bula15` | 10/0 | [] | — | — | — |
| Bula / `Bula16` | 10/0 | [] | — | — | — |

JSON chỉ sao chép mã map nếu census có trường riêng. Tên map hoặc hậu tố số không được dùng làm mã wire.

## Đối chiếu mode, tài nguyên và phạm vi

Nhãn wiki và đối ứng theo tên trong [bảng Mode](../CONTENT.md#mode) giúp chọn mẫu đo cho `RoomRules`. Số mục tài nguyên và khai báo `Attr/MapGroup` là metadata tĩnh. Số mục của một nhóm pack (ví dụ `FxMap/Map`) là của cả nhóm, không phải số tài nguyên riêng của mode.
Họ ngoài `mission` và các khai báo có mẫu như `Tomb?` không cộng vào tổng 122 map. Popcade có map trong tổng kiểm kê và vẫn mang đợt F3 theo bảng Mode.
[Bảng ModeRules](../../rust/crates/kr-service/src/room_modes/rules.json) khớp nhóm và đợt của 18 mode cùng nhóm luật của 6 họ mission có liên kết. Đây là hợp đồng thiết kế cho override do vận hành chọn; các mode vẫn bị chặn khi tạo hoặc bắt đầu trận vì chưa có dispatch KR đã đo.
Các trường thắng, hòa, hồi sinh, đồng hồ và reset mô tả phần cần xác định cho từng mode. `unknown` giữ chỗ cho phần chưa rõ, không kế thừa luật trận thường.

| Mode | Nhóm ứng viên | Đợt / lớp mode | Dấu vết trong pack | Họ map (thuộc mission / tổng) |
|---|---|---|---|---|
| Airplane | `airplane` | F2? / C | `FxStage/Game` (538), `Game/AirplaneBigWave`, `Game/AirplaneRetro` | — |
| Antarctic | `antarctic` | F2? / C | `FxContents/Antarctic` (15) | — |
| Baseball | `baseball` | F2? / C | `FxStage/Game` (538), `Game/Baseball_Image`, `Game/BaseballTip` | `Baseball` (10/10) |
| BigBattle | `big_battle` | F2? / C | `FxStage/Game` (538), `Game/BigBattle_*` | `Fm_BigBattle` (0/4), `Fm_BigBattleH_PreparePlay` (0/1), `Fm_BigBattle_PreparePlay` (0/1) |
| ConfusedFight | `confused_fight` | F2 / C | `FxContents/ConfusedFight` (50) | `ConfusedFight` (0/3) |
| Free 1v7 | `free_1v7` | F2 / C | Chưa ghép | — |
| Ghép trận UXMatching | `matchmaking` | F2 / C | `FxContents/UXMatching` (163) | — |
| SnowyChamp | `snowy_champ` | F2? / C | `FxContents/SnowyChamp` (6) | — |
| SnowyFields | `snowy_fields` | F2? / C | `FxMap/Map` (1151) | `SnowyFields` (18/18) |
| 고슴도치 (nhím) | `hedgehog` | F2 / C | Chưa ghép | — |
| 눈싸움 (đấu ném tuyết; sự kiện cũ) | `snowball_event` | F2 / C | Chưa ghép | — |
| 대장잡기 (bắt thủ lĩnh) | `leader_capture` | F2 / C | Chưa ghép | — |
| 래더 (Ladder) | `ladder` | F2 / C | `CharTE/LadderDeco` (89), `FxContents/Ladder` (9), `FxContents/WeeklyLadder` (12), `FxContents/Attr/MapGroup/LadderMapGroup` (6/6 tên khớp) | — |
| 부활대전 (hồi sinh) | `revive` | F2 / C | `FxContents/Attr:MapGroup` (1), `FxContents/Attr/MapGroup/ReBirthMapGroup` (0/2 tên khớp) | — |
| 슈퍼파이터 (Super Fighter; sự kiện cũ) | `super_fighter` | F2 / C | `FxMap/Map` (1151) | `SuperFighter` (2/2) |
| 시한폭탄 (bom hẹn giờ) | `timebomb` | F2 / C | Chưa ghép | — |
| 팝케이드 (Popcade) | `popcade` | F3 / C | `FxMap/Map` (1151) | `PopcadeA` (5/5), `PopcadeB` (5/5), `PopcadeC` (5/5) |
| 협공배틀 (đánh phối hợp) | `co_op` | F2 / C | Chưa ghép | — |

Họ map `SnowyFields` giữ nhóm riêng `snowy_fields`; nhãn ném tuyết của wiki chưa chứng minh đối ứng với mode sự kiện `snowball_event`. Các họ có chủ đề như `Zombie` hoặc `Gumiho` cũng chưa có đối ứng với hồi sinh, nhím hay bắt thủ lĩnh.

Cột Dấu vết ghi số mục `Attr/MapGroup` khớp nguyên tên duy nhất trong census trên tổng số mục khai báo. JSON giữ nguyên thứ tự và ký tự của khai báo; chưa diễn giải cách client xử lý `?`.
Khi đo Ladder, dùng một map trùng tên đã có trong [phạm vi nghiệm thu G1](../evidence/rust-service/g1-acceptance-matrix.json), chẳng hạn `Factory07`, làm đối chứng trận thường. Trước khi chọn map thử hồi sinh, đo cách client xử lý các khai báo như `Tomb?` và `Cemetery?`.

## Phép đo còn thiếu cho F2

1. Dùng IDA trên build KR 00967 để lần từ UI chọn mode hoặc `Attr/MapGroup` đến reader/writer ([điểm neo tĩnh Wave 8a](../evidence/content/f4-w8-ida-exp-175-f2.json): `MapGroupVisible` `0xe04cb0`/`0xe04dd0`, `Dlg_LB-Create_Ladder` `0xc541e0`, `Dlg_LB-Create_Mission` `0xc8df50`, `roomtype` `0x57f520`). Đo native trong mạng cô lập với cùng map, roster, đội và sức chứa; chỉ đổi lựa chọn mode. Nếu client không mở được lựa chọn Ladder, ghi điểm dừng ở UI hoặc luồng ghép trận trước khi thử gán giá trị wire.
   - Đối chiếu [tạo phòng 81/86](../protocol/room-membership.md#room-create) và [luồng join](../protocol/room-membership.md#duo-room-round). [Selector Free/Manner đã đo](../protocol/room-membership.md#current-mode-host-rejoin) là biến đối chứng về đội; chưa có bằng chứng đây là trường chọn mode F2.
   - Đối chiếu [map và Start 96/117/104](../protocol/load-phase.md#room-map) với [Start 121](../protocol/load-phase.md#start-reply). Giữ riêng mã map, dynamic index và byte `roomtype` đã đọc trong mẫu trận thường; cần xác định client thực sự đọc trường nào khi chuyển luồng.
2. Đo hai client qua ít nhất một đường thắng và một đối chứng không kích hoạt của mode đã nhận diện. Ghi bên phát sự kiện, trạng thái đội/người chơi, điều kiện kết thúc, kết quả và reset ván kế. Với map có script, xác định bên thực thi trigger và quyền áp dụng tác động trước khi thêm `RoomRules`.
3. Lần riêng chọn map của Ladder/hồi sinh và luồng UXMatching trước khi đo luật trận. Kiểm parser phần đuôi trên các map đọc một phần trước khi dùng bảng rơi, placement hay script của chúng.

## Tái lập

```bash
python3 tools/kr_f2_mode_audit.py --generate
python3 tools/kr_f2_mode_audit.py --check
```

`catalog_sha256` băm object JSON sau khi bỏ chính trường hash và `input_sha256`: UTF-8, khóa sắp tăng, không khoảng trắng, một LF cuối. `input_sha256` băm byte gốc của từng file nguồn.
Công cụ đọc census, bảng xếp loại và bảng ModeRules, chạy offline và không cần pack, IDA hay client. Nếu census bổ sung `partial_reason`, `missing_tags` hoặc `tail` (`offset`, `remaining_bytes`), công cụ sao chép chúng vào chẩn đoán.
Khi dùng `--census` khác, chọn `--classification` tương ứng hoặc `--no-classification` để giữ lớp chưa rõ. Dùng `--out` và `--markdown-out` để chọn file đầu ra riêng.
