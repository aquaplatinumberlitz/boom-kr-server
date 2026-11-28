# P252 — Cổng Start phòng co-op F2/F3 (IDA KR 00967)

Phân tích này dùng IDB `kr-full.exe.i64` của build KR 00967, SHA-256
`6be15a1fccea0837ee95c50893345e9f13dc217b0fa570975b5c284f0d547fd1`. IDA
được chạy trên bản sao tạm; địa chỉ dưới đây là VA với image base `0x400000`.

## Kết quả chính

`r1-ready` không phải là server tự chuyển cảnh. Client chỉ gửi C→S 104 sau khi
`CPrepareStage` gọi `sub_DE0310`; hàm này gọi `sub_DCAD80` và chỉ build 104 khi
kết quả là `0` hoặc `1`. Với các lượt `MonSnowBear07`, `MonGolem01`,
`MonPen01`, `PopcadeB01`, `PopcadeC01` và `Zzolman01`, capture có 117 và 110
đầy đủ nhưng không có 104. Lượt đối chứng `Coloring01`, `Hamster01`,
`Gumiho01`, `SuperFighter16`, `Rabbit01`, `Wriggle01` có cùng chuỗi 117/110,
khách ở đội 3 và host gửi 104.

Đây là predicate tối thiểu cho lần thử `wine-f2f3-r5`:

* hai slot có identity hợp lệ, một slot host và một slot khách;
* cả hai slot đã nhận 106 và 110; byte trạng thái đã sẵn sàng là group
  `(packed_state >> 2) & 7 == 1`;
* phải gán team/color khác 0 theo profile mode. Trong capture lỗi, cả hai slot
  đều mang team 0; capture đối chứng dùng team 3 cho cả hai slot. Static code
  chưa chứng minh hai người phải ở hai đội khác nhau;
* chọn map phải hoàn tất 117 (result 0, local-map code và dynamic-index hợp
  lệ), rồi host mới bấm Start. Nếu mọi predicate qua, client gửi C→S 104;
  server sau đó phải trả Start121 và profile/slot hợp lệ để vào stage.

Gán team khác 0 là yêu cầu thực tế đã được đối chứng, nhưng nguyên nhân cụ thể của
từng lượt kẹt (mã lỗi 3, 6 hay 7) chưa được bắt breakpoint trên chính sáu lượt;
không gán popup hay lỗi server khi capture không có 104.

## IDA anchors và logic

`sub_57F520` (`0x57F520`) là parser `User_RoomInvited`, không phải bộ phân
loại map. Nó đọc các field `actionid`, `channelid`, `hostid`, `roomid`,
`roommode` (offset `+628`) và `roomtype` (offset `+692`) từ packet/string
reader rồi trả `this + 24`. Vì vậy symbol `roomtype` ở đây chỉ xác định vị trí
field; không thể dùng nó để suy ra F2/F3.

Bộ chọn số người/đội là `sub_138A940` (`0x138A940`), được gọi qua vtable
`CPrepareStage` tại `sub_DCAD80` (`0xDCAD80`). Mặc định nó đặt
`min_players=2,min_teams=2`; `sub_1389AD0` (`0x1389AD0`) nhận diện nhóm mã
ASCII `Y..c`. Sau đó các mã game-type/room-type đặc biệt được hạ ngưỡng:

| mã được thấy trong helper | min players | min teams |
|---:|---:|---:|
| 19 (Popcade), 16/32, 17, 14/22/33, 96/107/116/118/121/127/124/129/131/132/134/136/155/178 | 1 | 1 |
| 18/21/25, 20 | 2 | 2 |
| 243/175/137 | 2 | 1 |
| 159 | 4 | 1 |
| 158 | 4 | 4 |

Các giá trị chưa có map-record KR tương ứng vẫn là static candidate; không dùng
bảng này để phát minh roomtype wire mới.

`sub_DCAD80` quét 16 slot. Nó đếm slot hoạt động (`v70`) và số đội có người
(`v63`), lấy ngưỡng từ `sub_138A940`, rồi trả: `4` nếu thiếu người, `6` nếu
thiếu đội, `5` nếu vượt giới hạn map, `7` nếu selector Manner yêu cầu các đội
cân bằng hoặc mode-specific roster/team không hợp lệ, `2` nếu map/filter không
được phép, `12` nếu policy ngoài. Ở vòng kiểm cuối, `sub_1240A50` đọc packed
state; team là `(word >> 5) & 0x1f`, ready/group là `(word >> 2) & 7`.
`sub_1241090` kiểm tra slot thuộc local/đã có identity; slot không thuộc local
và group khác 1 đi qua `sub_703A90`/`sub_1240D70` và có thể trả `3` (AllMustReady
hoặc slot chưa sẵn sàng). `sub_DCBED0` (`0xDCBED0`) đổi mã trả về thành popup
`CantDecideMap`, `AllMustReady`, `SmallPlayerNo`, `SmallMapPlayer`,
`SmallTeamNo`, `TeamDoesntMatch` hoặc `PremiumOnly`.

Nút Start ở `sub_DE0310` (`0xDE0310`) trước hết yêu cầu
`sub_12201A0()!=0` (stage đúng) và `sub_12202A0(1)!=0` (nút/identity hợp lệ).
Kết quả 0/1 mới gọi `sub_1164C10(104)`, dựng map context qua
`sub_1264220`/`sub_1263F40`, và gửi bằng `sub_13ADD60`; mọi mã lỗi còn lại chỉ
gọi `sub_DCBED0` rồi trả về, nên không có C→S 104.

S→C 110 được dispatcher tại `sub_DC51F0` (`0xDC51F0`) chuyển tới
`sub_DC6700` (`0xDC6700`): đọc slot, 4-byte packed state và tail; helper ghi
team vào bit 5..9 và gọi `sub_DD7C50` để cập nhật UI. S→C 117 đi qua
`sub_DC70D0` (`0xDC70D0`), đọc result/local code/dynamic index 8 byte rồi cập
nhật map hiện tại. Start response 121 dùng dispatcher
`sub_DC51F0→sub_DC5366→sub_DC7F30`; profile-list và các slot được kiểm ở
`DD6190`/`sub_1240A50`. Đây là điều kiện sau 104, không phải cách bypass cổng
client.

## Phân loại map và giới hạn PvE

Các map probe có mã pack và spawn đã đo: `MonSnowBear07` 666/4,
`MonGolem01` 501/4, `MonPen01` 536/4, `PopcadeB01` 910/2,
`PopcadeC01` 915/4, `Zzolman01` 1146/4. Các control: `Coloring01` 213/8,
`Hamster01` 355/1, `Gumiho01` 344/2, `SuperFighter16` 1075/8,
`Rabbit01` 920/8, `Wriggle01` 1115/8. Đây là phân loại theo pack/map record;
`sub_57F520` không tạo ra các nhãn này.

`sub_DCAD80` không tự tạo AI slot. Với `Mon*`, Popcade và Zzolman, bốn spawn
trong map không làm bốn client hợp lệ; server phải đưa profile Start121 có
slot/identity/spawn phù hợp sau khi 104 được chấp nhận. Số AI, team AI và
script PvE sau Start121 chưa được chứng minh bởi lượt r1/r4; giữ `unknown`.

## Bằng chứng runtime và bước r5

Các capture đã giải CRC bằng decoder KR: lượt lỗi chỉ kết thúc ở 110; lượt
đối chứng có thêm host 104, S→C 121 và S→C 153. Cần chạy r5 với khách đổi
đội sang 3, giữ hai 110 và 117 như control, rồi kiểm theo thứ tự
`104 → 121 → 99 → 153`. Nếu vẫn thiếu 104, lấy return của `sub_DCAD80` và
slot/state gây lỗi; không giả lập 104 hoặc coi UI Ready là bằng chứng server
đã nhận Start.
