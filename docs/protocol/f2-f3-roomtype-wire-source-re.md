# P256 — nguồn `game_type`/`room_type`, màu đội và cổng Start

Phân tích tĩnh dùng build KR 00967, IDB SHA-256
`6be15a1fccea0837ee95c50893345e9f13dc217b0fa570975b5c284f0d547fd1`, image base
`0x400000`. IDA chỉ đọc bản sao của IDB. Kết quả runtime là so sánh có giới hạn
của các lượt Wine r3/r4; không phải server KR gốc.

## Chuỗi nguồn selector

`sub_57F520` (`0x57F520`) chỉ là parser `User_RoomInvited`: `roommode` ở
`this+628` và `roomtype` ở `this+692`. Nó không phân loại map. Dispatcher
`sub_DC51F0` (`0xDC51F0`) nhận packet 97/106/110/117 theo opcode; nhánh 117 gọi
`sub_DC70D0` (`0xDC70D0`). Hàm này đọc `result:u16`, `local_map_code:u16`,
`dynamic_index:u32`, rồi cập nhật map record. `sub_DFEB10` (`0xDFEB10`) đọc
`fxdynmapinfo` rows; row giữ local code ở offset record `+68` và mode byte ở
record `+40`/`+41` (được đọc qua `sub_E02A60`, `sub_E02C50`, `sub_E02F60`). Vì
vậy `S->C 117` chọn record, nhưng không tự đặt team.

`sub_DE1F20` (`0xDE1F20`) là caller duy nhất của `sub_138A940` (`0x138A940`):
`a3` là `game_type`, `a4=sub_1243C00(sub_544520())` là room-mode selector,
`a5` là policy/phase. `sub_1389AD0` (`0x1389AD0`) trả 1 đúng cho mã ASCII
`'Y'..'c'` (89..99); `sub_138A940` mặc định `(min_players,min_teams)=(2,2)`.
Các case tĩnh đã thấy: game type `243,175,137` cho `(2,1)`, `159` cho `(4,1)`,
`158` cho `(4,4)`, `107,96,116,118,121,127,124,129,131,134,132,136,155,178`
cho `(1,1)`, còn `a4=14,22,33,16,32,17,19,39` cho `(1,1)`; `a4=18,21,25,20`
cho `(2,2)`. Đây là các nhánh helper, không phải phép gán map-family.

Bộ nhãn UI `sub_C6C080` (`0xC6C080`) xác nhận các game-type selector công khai:
`19=Popcade`, `18/21/25=SuperFighter`, `36/37/38` và `39` là các selector khác
đi vào room-type 4/19. `sub_AEC920` (`0xAEC920`) nối game-type với map-record
mode; `sub_E02410` (`0xE02410`) trả nhóm hạn chế theo record mode. Không có
đường tĩnh chứng minh `Mon*`, `PopcadeB/C` hay `Zzolman` tự sinh một màu đội từ
FxMap payload; gán họ map vào mọi ID còn lại là unknown.

## Packed state, Start và mã trả về

`sub_DE28A0` (`0xDE28A0`) xử lý S→C 106 và khi tạo slot gọi `sub_D62950(slot, position, team, player_blob, 1)`; nó lấy team từ packed word của profile/identity trước khi UI hiển thị. `sub_DC6700` (`0xDC6700`) xử lý S→C 110. Packed word được đọc/ghi qua
`sub_1240A20`/`sub_124B7B0` (`0x124B7B0`): bit0 là active, bits2..4 là ready
**group**, bits5..9 là team/color `(word >> 5) & 0x1f`, bit10 là cờ phụ; word
kế tiếp giữ position `&0x1fff`. `sub_DCACC0` (`0xDCACC0`) là đường cập nhật
team từ client/UI: đọc slot rồi đặt `word = (word & 0xfc1f) | ((team & 0x1f)<<5)` và
đi qua `sub_D62B00` (`0xD62B00`). Đây là thao tác cần dùng khi bấm team/color. Trong live recipe, nút team của client phát yêu cầu C→S 110 và server phản hồi S→C 110; body request vẫn opaque trong scope này. 106/110 từ server chỉ phản ánh giá trị đã lưu.

`sub_DCAD80` (`0xDCAD80`) quét 16 slot, đếm player và số team có người,
đọc ngưỡng từ `sub_138A940`, rồi kiểm identity/ready/map. Mã chính: `4` thiếu
player, `6` thiếu team, `5` vượt capacity/giới hạn mode, `7` vi phạm cân bằng hoặc
roster mode-specific, `2` map/filter/policy không cho phép, `12` policy phụ; vòng
slot chưa ready có thể trả `3`. Các nhánh này là điều kiện, không phải quan sát
return cụ thể của sáu lượt r3/r4.

`sub_DE0310` (`0xDE0310`) chỉ gửi C→S 104 (`sub_1164C10(104)` rồi builder
`sub_1264220`/`sub_13ADD60`) khi `sub_DCAD80` trả `0` hoặc `1`. Mọi mã trên đi
`sub_DCBED0` (`0xDCBED0`) để báo lỗi và không phát 104. Sau 104, dispatcher
`sub_DC51F0` đưa S→C 121/153; đó là hậu điều kiện, không phải bypass client.

## Đối chiếu r3/r4 và recipe r5

Các map dừng: `MonSnowBear07(666)`, `MonGolem01(501)`, `MonPen01(536)`,
`PopcadeB01(910)`, `PopcadeC01(915)`, `Zzolman01(1146)`. Chúng đều nhận 117
với `result=0` và code/index đúng, rồi 110; không có 104. Control
`Coloring01(213)`, `Hamster01(355)`, `Gumiho01(344)`, `SuperFighter16(1075)`,
`Rabbit01(920)`, `Wriggle01(1115)` có cùng 117/110 rồi host phát 104, nhận
121/153. Giải mã 110 cho thấy các lượt dừng có hai slot team `0`, còn control có
team `3`; đây là khác biệt runtime đã quan sát, không chứng minh hai client phải
khác team. `sub_DCAD80` return cụ thể chưa được bắt trong các lượt cũ.

Để thử `wine-f2f3-r5`, giữ identity/slot hợp lệ, chờ 106 với team/color được profile đặt, chờ 110 với
`ready_group=1`, chờ 117 `result=0`, rồi gửi thao tác team qua đường 110/đường
UI tương ứng để cả hai slot dùng màu mode-accepted khác 0 (control dùng team 3).
Không giả AI slot: `sub_DCAD80` không tạo slot AI; Start121 phải chứa profile/slot
PvE do server cung cấp. Kiểm chuỗi bắt buộc `104 -> 121 -> 153` và ghi thêm
`204/132/180` nếu server mở nhánh Mon/PvE. Nếu vẫn thiếu 104, cần bắt return
`sub_DCAD80` và packed words ngay trước nút Start; không suy mã lỗi từ EOF.

Giới hạn còn lại: ID map-family trong row `+40/+41`, luật team đối nghịch, slot AI
và ý nghĩa PvE của Start121 chưa được xác nhận bằng runtime r5.
