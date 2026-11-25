# F1 phụ trợ: opcode 138/139, item Ctrl chủ động, khóa nhân vật lớn và Ship bổ sung

Phần phân tích IDA là `kr_static` trên ảnh unpack KR 00967 `cf72da11…` (image base `0x400000`, RVA = VA trừ
`0x400000`). [Bằng chứng, hash và phạm vi](../evidence/content/f1-aux-opcodes-and-classc-ida.json) giữ VA và trạng
thái của từng kết luận. Không chạy Wine, client hay server trong lượt này. Hai mẫu wire `138`/`139` do brief cung cấp
chỉ được giải mã lại bằng writer tĩnh; chúng không tạo thêm bằng chứng runtime. Quyền xử lý của server gốc vẫn
**unknown** ở mọi mục. Danh sách cụm cần làm nằm ở [Full F1](../content/f1-full-mechanic-clusters.md).

<a id="f1-aux-codec"></a>
## Primitive dùng chung

`CSendPacket` (vtable `178DAFC`) ghi u8 qua `14EBC50 → 14EBC70 → 14A5C80` (XOR `0x5A`), u16 qua
`14EBCF0 → 14EBD10 → 14A5C90` (big-endian, XOR `0xA569`) và u32 qua `14EBD90`. Bộ đọc `CReceivedPacketBase`
(vtable `17B0BF4`) dùng slot `+0x3C` cho u8, `+0x40` cho u16, `+0x44` cho u32 và `+0x5C` cho blob có độ dài.
Blob gửi bằng `14EC120` là buffer tuần tự hóa có length và checksum như ở [peer events](life-results.md#peer-events).

Nhiều gói trong trận đi qua `CGameMsgCache` (vtable `17696E8`). Packet dùng chung nằm ở cache `+0x104` và
`E2D310` (slot `+0x14`) xả hàng đợi khi đã qua hơn 200 tick của đồng hồ client hoặc khi được ép. Byte đầu của các
gói này là `world_index` lấy từ `stage+0x1DF110` (`E150F0`); `E702D0 → E0E4D0` gán nó bằng chỉ số của world trong
danh sách stage. Gói `138`, `126` và `157` mở đầu bằng trường này; `118` đặt nó sau u32 serial. Trận một world
thường có giá trị 0, phù hợp với mẫu `138` live.

<a id="f1-aux-dart"></a>
## Dart: C→S 138 và S→C 172

**Confirmed.** Case group 21 (`1190F62`) gọi interface `actor+0x590/vf+0x14 → B4A780`. Hàm này đi từ ô của actor theo
hướng `vf+0x11C`, mỗi bước 40 px theo bảng `17929B0` (hướng 0/1/2/3 tương ứng cột −1, hàng −1, cột +1, hàng +1),
dừng khi rời map hoặc `EA0B80` báo chặn. Nó dựng hiệu ứng cục bộ qua `vf+0x374 → B5E660`, rồi `E2C950` ghi bản
ghi chờ cho slot. Guard `E2C9D0` chỉ cho bản ghi mới khi slot không còn bản ghi chờ và đã qua 200 tick.
Lần xả kế tiếp gọi `E2EB30`, gửi **9 byte**:

| Trường | Kiểu | Nguồn |
|---|---|---|
| `world_index` | u8 | `E150F0` |
| hằng 0 | u8 | Receiver bỏ qua gói nếu byte này khác 0. |
| `x_px` | u16 | `40 × cột` |
| `y_px` | u16 | `40 × hàng + 40` |
| `dir` | u8 | Receiver đổi 255 thành −1. |
| `distance_px` | u16 | Bội số 40 của quãng đã đi. |

Mẫu live `5a5aa5c9a7615aa5c9` giải ra `0, 0, 160, 520, 0, 160`: phi tiêu ở cột 4, hàng 12, bay sang trái 4 ô.
Vòng lặp ghi 200 nếu đi hết tới mép trái, nên 160 cho thấy bước tới cột 0 bị chặn; vật cản cụ thể chưa biết.

**Confirmed.** `S→C 172 → E64570` đọc đúng 9 byte theo cùng thứ tự, bỏ qua nếu `world_index` khác của mình hoặc byte
thứ hai khác 0, rồi tạo object 156 giống hiệu ứng cục bộ. Body không có slot nên peer không biết ai ném;
đường này chỉ dựng hình ảnh. **Candidate:** server chuyển tiếp nguyên 9 byte `138` thành `172`.

Nếu đường bay gặp bom hợp lệ (`E17AF0`), chính client ném gọi `bomb/vf+0xCC → B731D0` và gửi ngay
**C→S 122** `[u32 bomb_id, u16 hàng, u16 cột, u32 reason=0, u8 slot, u32 delay]`. Ở đây
`delay = max(0, 1200 × distance_px / 1000 − 2 × latency)`. Mã reason 4 của ô Pin đã có ở
[terrain](terrain.md#rust-camp08-boundary); mã 2 thuộc RemoteBomb bên dưới. Phản hồi server cho reason 0/2 chưa biết.

<a id="f1-aux-group11"></a>
## Vật đặt group 11: 139 → 173 và kích hoạt 126 → 168

**Confirmed.** `B4C1F0` (interface `actor+0x590/vf+0x08`) kiểm ô rồi gọi `B9DEF0` cục bộ. Sau đó nó gọi
`actor/vf+0x324 → B7A710 → E781B0` và gửi ngay **C→S 139** gồm bốn u8: `slot` (`m_SlotIdx`, `actor+0x594`),
`kind_flags`, `hàng`, `cột`. `kind_flags` là selector, OR `0x80` khi đối số thứ ba bật và OR `0x40` khi ô bị kẹp
theo trục. Biến thể có `a6` khác 0 đặt tối đa bốn ô chéo và gửi một gói cho mỗi ô. Mẫu live `5a5b5158` là slot 0,
Bond (selector 1), không cờ, hàng 11, cột 2.

**Confirmed.** `S→C 173 → E646E0` chỉ đọc ba u8 `kind_flags, hàng, cột`; không có slot hay `world_index`.
Kind là `kind_flags & 0x0F` (`E8FBB0`). Actor đang đứng đúng ô được ghi ô đó qua `B4CAB0`; trường này được dùng làm
ô miễn kích hoạt. Sau đó hàm gọi `B9DEF0`. Do đó gói `173` là body `139` bỏ byte slot; việc server làm đúng như vậy
vẫn là **candidate**. `B9DEF0` có nhánh 1 Bond, 2 Trap, 3 Banana, 4 Pin, 7 (biến thể Bond, timer 60/4) và 8.
`S→C 427/sub6 → E77740` mang danh sách cùng bộ ba nhưng chỉ áp dụng các mục kind 7 (**candidate**).

**Confirmed.** Ở `B77300` (GameBomber/CAHero), khi actor vào ô có vật đặt khác ô miễn, `1191F80` lọc theo kind:
1/3/7 cần `actor+0x584/vf+0x80 == 0` (cờ `m_fAbPassBox` của xe bay ở [F1-d](vehicles.md)), 2 cần `B540D0 == 0`, 4 không bao giờ kích hoạt, 5/6/8 luôn kích hoạt.
Qua lọc, `E2C780` xếp hàng (10 mục mỗi slot, khử trùng lặp) và `E2E610` gửi **C→S 126**
`[u8 world_index, u8 slot, u8 hàng, u8 cột]`. Lớp cơ sở `B525E0` áp dụng trực tiếp mà không gửi gói.

**Confirmed.** `S→C 168 → E63F10` đọc `u8 result, u8 world_index, u8 slot, u8 hàng, u8 cột` và luôn xóa mục chờ
khớp. Khi `result` khác 0 và world khớp, hàm gọi `1192040` cho actor của slot nếu ô còn vật; nếu không còn vật
thì gọi `E72870(slot, 4, −1)`. `1192040` xóa vật kind 1/2/3/7 bằng `B9E310(0)`. Bond gọi `B49460(1,12,0)`.
Trap tháo xe qua `vf+0x180` nếu đang cưỡi, nếu không thì nhốt actor ở tâm ô. Banana gọi `vf+0x130`; Pin không làm gì.
`B9E310` còn được gọi từ đường quét ô `EB9840` (sau `E9E660/E9EEE0`) và từ vtable bom, đều cục bộ.
Nghĩa đầy đủ của `result` và luật chọn owner của server gốc vẫn **unknown**.

<a id="f1-aux-ctrl"></a>
## Item Ctrl chủ động: Spring, Drill, RemoteBomb, Shield, Sensor

`B4D710` gọi switch dùng `1190C20`. Nếu thành công, nó dựng lệnh `AEE0D0` và `AEE300 → AEE3B0` gửi
**C→S 127/sub0** `[prefix 14 byte, u8, u16 kind, u8 lượng tiêu, u32]`. Vì vậy gói riêng của từng case (nếu có) luôn
đi trước sub0. Interface nhận case là `actor+0x590` (vtable `16F00E8` ở CAHero).

| Group | Case → hàm | Gói C→S | Receiver đã nối |
|---|---|---|---|
| 26 Spring | `1191063 → vf+0x28 → B4B5A0` | `127/sub2` + blob 3 byte `[hàng, cột, (distance<<4) \| dir]` | `170/sub2` đọc ba u8 inline sau header 6 byte → `E86DD0` → `vf+0x138(distance, dir, hàng, cột)` |
| 27 Drill | `119108B → vf+0x2C → B73C60 → B4BB40` | `127/sub3` + blob 7 byte `[u32 box_id, u8 hàng, u8 cột, u8 dir]` | `170/sub3` đọc blob → `E86520` |
| 28 RemoteBomb | `11910CF → vf+0x30 → B4BE10` | `118` với byte cờ `0x04` qua `E2C0E0`, gửi ở lần xả kế | Như [118](bombs.md#c2s118-admission) |
| 29 Shield | `11910F8 → vf+0x34 → B73DC0` | `127/sub5`, chỉ prefix, khi `actor/vf+0x1BC` trả khác 0 | `170/sub5 → E86670 → vf+0x1BC` |
| 30 Sensor | `119114A → vf+0x38 → B50110` | Không có gói riêng; case luôn trả thành công, chỉ gọi `EA8D80` hiển thị cục bộ khi guard qua | — |

**Confirmed (writer/reader).** Spring gửi blob nhưng `170/sub2` đọc ba byte trần, nên server phải bóc blob trước
khi phát lại. `170/sub0` cũng đọc blob trong khi `127/sub0` ghi trường trần. Cả hai đều không thể chuyển tiếp nguyên
body. Group 53 cũng dùng `vf+0x34`. Shield bật khiên bằng `actor/vf+0x1BC`; peer gọi cùng slot khi nhận `170/sub5`.

**Confirmed.** RemoteBomb kích nổ bằng phím đặt bom: `B518C0 → B4C0C0` duyệt bom của mình có cờ remote
(`B38070(4)`), đánh dấu bom chưa kích (`B38430`) rồi gọi `actor/vf+0x1C8 → B7A8A0 → B731D0`. Hàm gửi
**C→S 122** reason 2, `delay = max(0, 200 − 2 × latency)` và không đặt bom mới trong lần bấm đó.

<a id="f1-aux-character-keys"></a>
## Khóa nhân vật lớn hơn 255 và hàng lớp C

**Confirmed.** `Start 121` đọc khóa nhân vật trong hàng roster bằng u16 (`DC8A2A: 14E9910`, `movzx`) và ghi vào
record slot `+0x60` qua `1241250`. Wire vì vậy biểu diễn được khóa tới 65535. Giá trị đặc biệt `0x1FFF` được
`11D2F20` luôn chấp nhận và `13E52A0` xử lý riêng. Mục load-phase gọi trường này là position, còn Rust và F1-c dùng
nó làm khóa nhân vật; lượt này chỉ xác nhận độ rộng u16 và setter, không đổi tên trường.

**Confirmed.** Writer **C→S 79** là `D75D70`: `u8 count` (`13B07C0`), sau đó mỗi người chơi cục bộ có một u8.
Byte lấy từ `13E51F0`, nơi u16 đã lưu bị cắt còn `char`. Nhánh ánh xạ `1351C50` không bao giờ chạy vì
`703870()` luôn đúng, do `456AD0` luôn trả 0. `D7CC20` kiểm **byte đã cắt** bằng `11D2F20`. Nếu không qua, client
gửi **5** và ghi 5 lại làm lựa chọn cục bộ (`13E52A0`). `11D2F20` nhận khóa nhỏ hơn 12 qua `11D2CE0`, nhận
120–131 và 150/151 qua `1184F20`, còn khóa khác cần bản ghi sở hữu `13E5DA0/13E3B00`. `D75D70` được gọi từ handler
`121` (`DC92DF`) và từ đường chọn nhân vật `D61990`. Hai mẫu `5b58/5b59` đã ghi giải ra `(1,2)/(1,3)`; mẫu sảnh
`5b5f` giải ra `(1,5)`.

**Candidate rủi ro.** Với 3000–3004, client sẽ gửi `184–188` nếu byte cắt qua kiểm sở hữu, nếu không thì gửi 5;
client không thể gửi 3000. Tại PR #621, policy Loading79 chỉ nhận byte bằng khóa đã chọn. Nhánh 4000–4030 tương ứng
byte 160–190, còn 5000 thành 136. Các dự đoán byte trên client vẫn cần đo native.

**Confirmed.** Ở `E0F770`, `AEDE60` luôn đúng. Nếu đối số 5 bằng 95 và khóa thuộc 4000–4030, hàm gọi `B3BDE0(0)`
rồi trả về trước khởi tạo actor và trước bộ đếm spawn. Đối số này là `E02A60(E04F00(map))` từ `E5A290`
(**candidate**: loại map/mode). Ở giá trị khác, các khóa đó đi đường spawn chung. Chưa thấy guard UI riêng cho
33 hàng lớp C ngoài kiểm sở hữu `11D2F20`.

<a id="f1c-classc-rust"></a>
## F1-c: Rust opt-in cho 33 nhân vật lớp C

**Source:** `kr_static + emu_choice`. **Status:** `confirmed` trong kiểm thử Rust.
[Bằng chứng và cách kiểm](../evidence/rust-service/f1c-classc-rust.json).

Đặt `fixed_room.allow_class_c_characters: true` và thêm các khóa muốn mở vào
`managed_rooms.player_round.characters`. Cờ mặc định tắt; bật cờ vẫn chỉ mở các khóa được liệt kê.
Giữ khóa mặc định 0. Với cấu hình dùng `pack_stats`, giữ các hàng F1 thường trong bảng đó; sáu chỉ số của
nhân vật lớp C lấy trực tiếp từ các hàng `BomberAttr` đã đối chiếu. Chỉ số phải thỏa
`1 ≤ hiện tại ≤ tối đa ≤ 10` cho tốc độ, sức chứa và tầm bom. Giới hạn hiện có của địa hình vẫn
từ chối cấu hình có tầm bom tối đa 10.

Danh sách lớp C gồm 28 khóa `4000–4018`, `4020–4024`, `4027–4030` và 5 khóa `5000–5004`.
Các hàng đối chứng `4019/4025/4026` (C15/C08/C26) nằm ngoài 33 hàng nên vẫn bị từ chối.
`120–131`, `150/151` và `3000–3004` giữ nguyên phạm vi F1 thường.

C2S92 kiểm catalog trước khi đổi lựa chọn. S2C Start121 giữ nguyên khóa u16 theo slot.
C2S Loading79 vẫn đọc `[u8 count=1, u8 selected]` và chỉ ghi nhận một lần cho thành viên hiện hành trong
pha Loading. Với khóa lớp C đã chọn, Rust nhận byte thấp của khóa hoặc giá trị dự phòng 5 theo writer tĩnh
`D75D70 → D7CC20`. Gói này không đổi lựa chọn của server, không có ACK và không hoàn tất hàng rào tải trận.
Các khóa F1 thường vẫn dùng phép so khớp đầy đủ trước đây, kể cả khi bật cờ.

Chưa đo Wine cho 33 khóa này. Chưa xác nhận client cho chọn trong UI, byte thực tế khi qua kiểm sở hữu,
actor/skill trong trận, chế độ có đối số 95, hay hành vi server KR gốc. Kiểm thử dùng roster tổng hợp
để xác nhận lựa chọn, khóa Start121 và xử lý Loading79 của Rust; chưa mở dạng Loading79 nhiều người chơi cục bộ.

<a id="f1-aux-ship"></a>
## Ship bổ sung

**Confirmed cho đường đã quét.** Không có opcode lên xe riêng theo selector. Helper `B48020(actor, sel)` chỉ được
gọi với 12, 14, 15 và 16; tám caller của getter `B64F40/B48000` không so selector khác. Vì vậy 22 hàng bổ sung dùng
chung `165` và `127/sub16 → 170/sub16` như [F1-d](vehicles.md), trừ các phần dưới. RailCar 8, xe bay và Retro
chỉ khác dữ liệu hoặc hiệu ứng. Tank 12/14/15 chỉ thêm hoạt ảnh/âm thanh cục bộ (`B4A620`, `B514F0`).
Việc 14→15 không có opcode mới.

**Confirmed.** Hiệu ứng Ship 4–7 (`1191D20`) chỉ bật cờ cục bộ khi lên xe. Khi `1191DD0` xóa, hiệu ứng 5 (Tank 12,
MonS2Tank 14, MonS2UpgradeTank 15, Shark 16) gọi `vf+0x31C → B7A590`, gửi **C→S 167** `[u16 slot, u16 3]`.
Hiệu ứng 6 (Shark) gọi `vf+0x320 → B7A650`, gửi `[u16 slot, u16 4]`. Cả hai bỏ qua khi `actor/vf+0x1D8` khác 0.
Bố cục này khớp mẫu bom `167` ở terrain nếu u16 đầu là slot báo cáo và u16 thứ hai là mã (**candidate**).

**Confirmed.** Shark 16 (`B814C0` luôn đúng):
- Chạm bom khi `B55190` nhỏ hơn 3 (**candidate**: số bom đã nuốt): `E6F510 → E6FB80` gửi **C→S 219**
  `[u32 E704C0, u32 238, u8 0, u32 slot, u32 bomb_serial]`.
- Phím đặt bom gọi `B76CD0` (khoảng cách tối thiểu 0x1A tick). Nhánh nhả bom gửi `118` cờ 1 qua `E2C1F0`, rồi
  gửi `219 [u32, 238, u8 1, u32 slot]`.
- Khi tháo xe, `B73680` có 50% gọi `E6FF00`, xếp hàng **C→S 157**
  `[u8 world_index, u8 slot, u32, u16, u8, u8]` cho item tra bằng `1190880(actor, 55)`; 55 là kind item Shark
  (**candidate**: thả lại item).

Handler S→C và luật server cho 167/219/157 vẫn **unknown**.

<a id="f1bc-classc-sensor-wine"></a>
## Nghiệm thu Wine hai client cho F1-c và F1-b Sensor/Shield

**Source:** `kr_runtime`. Hai client KR 00967 chạy trên Wine trong namespace mạng cô lập, dùng `kr-service` và
`kr-prelogin` build `--release --offline --locked` từ `5d95458`, không sửa code server.
[Bằng chứng, hash, số gói và lệnh tái lập](../evidence/item-families/f1bc-classc-sensor-wine.json).

**F1-c lớp C, status `unknown` (bị chặn).** Cấu hình bật `fixed_room.allow_class_c_characters` và liệt kê
`4000`, `4013`, `5000`, `5001` (đủ ba nhóm f1_3, f1_4 và f1_6). Khóa được nạp sẵn vào `player_preferences`
(`emu_choice`), vì UI client không chọn được các khóa này. Kết quả **confirmed** trên cả bốn khóa, ở cả vai chủ
phòng (81) lẫn vai khách (82):
- `S→C 106` mang khóa ở nửa cao của trường u32 tại offset 12 của body, nửa thấp là đội. Ô phòng của client hiển
  thị sprite khác nhân vật mặc định.
- Sau 9–29 ms, client gửi **`C→S 92` = (local 0, khóa 0)**, tức là tự đổi về khóa 0 chứ không dùng giá trị dự phòng 5
  như `79`. Gói này đến trước `96/266`, khi phòng chưa setup xong. `kr-service` từ chối lựa chọn và đóng stream,
  client hiện lỗi mất kết nối (errorcode 10).
- Đối chứng khóa F1 thường `1` không sinh `92` và vào phòng bình thường.

Vì vậy chưa có `Start 121`, byte `C→S 79` thực tế, actor trong trận hay đường về phòng cho khóa lớp C. Nguyên nhân
**candidate** là client thiếu bản ghi sở hữu (`11D2F20`, `13E5DA0/13E3B00`) mà server hiện chưa cấp. Muốn mở lớp C
cho client gốc thì cần đường sở hữu nhân vật phía server hoặc cách xử lý `92` trước setup. Cả hai đều ngoài lượt
đo này.

**F1-b Sensor/Shield, status `confirmed` trên lát cắt Cookie07.** Lượt chạy dùng
`item_policy: "report_derived_f1b_remote_sensor"`, đặt `151/152` trong list1 của floor `582` nạp trước trận, chơi
hai ván và đổi vai giữa hai ván. Phím Ctrl phải gửi bằng sự kiện X11 `Control_L`; `SendInput` với VK 17 không tới game.
- Nhặt item theo luồng `C→S 124 → S→C 165` tới cả hai client. Ô Ctrl hiện Shield `×02` và Sensor `×06`.
- Shield: mỗi lần Ctrl, client gửi `127/sub5` (prefix 14 byte) rồi `127/sub0` `[u8 1, u16 151, u8 0, u32 0]`.
  `kr-service` chuyển `170/sub5` 6 byte, gồm byte subtype, slot và bốn byte đầu prefix, tới peer. Hiệu ứng khiên
  hiện đồng thời trên view A và view B theo cả hai chiều.
- Sensor: 6 gói `127/sub0` kind 152, không có gói `S→C`.
- Khi hết charge (Shield 2, Sensor 6), ô Ctrl trống và các lần Ctrl tiếp theo không tạo gói. Phiên vẫn sống, hai ván
  kết thúc `complete/elimination`, hai client thoát qua `86 → 241` và `room_count=0`.

**Giới hạn.** Vì client tự dừng gửi khi hết charge, chưa đo được cách server xử lý `127/sub0` thừa. Thời lượng khiên
và tác dụng Sensor chỉ được xem qua ảnh. Kết quả không bao hàm server KR gốc, map khác hoặc Internet thật.

<a id="f1c-classc-presetup92"></a>
### Xử lý gói `C→S 92=(0,0)` và `Loading 79` cho lớp C (`kr_runtime + emu_choice`)

Để tháo nút chặn `o12` và `o13` trên client thật (gửi `C→S 92=(local 0, key 0)` sau `S→C 106` cả trước lẫn sau `setup` khi nhận `106` của peer, và gửi `C→S 79 = (1, 0)` lúc vào trận), `kr-service` (`room_io.rs` và `characters.rs`) mở hai điều chỉnh thu hẹp dưới cờ `allow_class_c_characters`:
- `is_class_c_auto_reset`: bỏ qua an toàn mọi gói `92=(0,0)` trong `Phase::Idle` (cả trước và sau `setup`) cho thành viên đang giữ một khóa lớp C (`is_class_c`), trả `class_c_reset_ignored` mà không xóa khóa lớp C hay ngắt stream; mọi gói `92` chọn khóa khác `0` sau `setup` vẫn hoạt động bình thường, còn `92` trước `setup` khác vẫn bị từ chối fail-closed.
- `class_c_loading79_matches`: chấp nhận byte `0` ngoài byte thấp của khóa và giá trị dự phòng `5`.

<a id="f1-full-wine-accept-bc"></a>
## Nghiệm thu Wine hai client sau #641/#642: lớp C, Drill và RemoteBomb

**Source:** `kr_runtime`. Hai client KR 00967 trên Wine, namespace cô lập, `kr-service` chạy trực tiếp từ bộ binary
`--release --offline --locked` tại `8fe1301`, không proxy và không sửa code.
[Bằng chứng, hash, frame hex và lệnh tái lập](../evidence/item-families/f1-full-wine-accept.json).

**F1-c lớp C `4000`/`5000`, status `unknown` (bị chặn ở Loading 79).** Hai lượt đổi vai cho cùng kết quả:
- Guard `class_c_presetup_reset_ignored` của #642 có hiệu lực: gói `C→S 92=(0,0)` mà mỗi client gửi 4–9 ms sau
  `S→C 106` của chính mình, trước setup, không còn ngắt stream; phòng setup xong cho cả chủ phòng và khách.
- Client chủ phòng gửi lại `92=(0,0)` 1–8 ms sau khi nhận `106` của khách mới vào. Lúc đó chủ phòng đã setup nên
  server nhận như lựa chọn hợp lệ khóa `0`; `106` kế tiếp và journal đều ghi khóa `0` cho chủ phòng.
- `Start 121` tới khách mang khóa lớp C ở u16 tại byte 3 của hàng roster (`[0, 5000]` và `[0, 4000]`). View khách
  vào arena với sprite lớp C.
- Khách gửi `C→S 79` = (count 1, byte **0**), không phải byte thấp của khóa (`0x88`/`0xA0`) hay giá trị dự phòng 5.
  `class_c_loading79_matches` không khớp nên `observe_loading79` từ chối và server đóng stream của khách
  (`loading_aborted`). Muốn qua bước này, server cần chấp nhận byte `0` cho thành viên lớp C và xử lý `92=(0,0)`
  sau setup; cả hai là thay đổi code ngoài lượt đo.

**F1-b Drill `149`, status `confirmed` theo chiều B→A.** Nhặt `124 → 165` tới cả hai client, ô Ctrl `×03`. Phím
`Control_L` tạo `127/sub3` với blob `[u32 0xFFFFFFFF, hàng, cột, dir]` rồi `127/sub0` kind `149`; `kr-service`
chuyển `170/sub3` (header 6 byte cùng blob) tới peer sau khoảng 15 ms. Thùng biến mất trên cả view A và view B,
charge còn `×02`. Chiều A→B chưa đo được vì công cụ nhập không quay được hướng nhìn của A về phía thùng.

**F1-b RemoteBomb `150`, status `unknown` (bị chặn ở 118).** Ctrl gửi `127/sub0` kind `150` rồi `C→S 118` sau
khoảng 70 ms. Byte 21 của `118` là `0x04` như dự đoán tĩnh, nhưng trường `v[6]` (byte offset 12) bằng **2** thay
vì 1 ở cả vai chủ phòng và vai khách. `Request::parse_with_remote` đòi `v[6]=1` nên từ chối và đóng stream; chưa
có `159`, `122` hay `158` cho bom remote. Ý nghĩa của `v[6]=2` chưa RE.

<a id="f1-full-wine-accept-v2-bc"></a>
## Tái nghiệm thu lớp C, Drill và RemoteBomb sau #647 (p113)

`source: kr_runtime`. Hai client KR 00967 chạy trên Wine với `kr-service` trực tiếp, bản
`f1-full-rust-v2` (`69212df`). [Bằng chứng, frame hex và cách tái lập](../evidence/item-families/f1-full-wine-accept-v2.json).

**F1-c — `blocked` ở bước rời phòng.** Với `allow_class_c_characters` và lựa chọn đã lưu
`A:4000, B:5000`, các gói `92=(0,0)` quan sát trong pha chờ, trước và sau setup, không đổi khóa.
Cả hai kênh nhận `Start 121` có roster `[4000, 5000]` và gửi `Loading 79=(1,0)`.
Hai màn hình đều hiển thị cả hai sprite lớp C trong trận Tomb12. Cả hai nhận `175` và `180`;
SQLite ghi `complete/elimination`. Sau `180`, B lại gửi `92=(0,0)` trước khi server làm mới
phòng (`Phase::Ending`). Kênh B kết thúc với guard `room_request` ở opcode `92`, rồi kênh A mất
membership. Không có `86 → 241`. `room_count=0` sau ngắt phiên không đáp ứng tiêu chí thoát phòng sạch.

**F1-b RemoteBomb `150` — `confirmed` (cả hai vai chủ phòng A và khách B).** Trên `f1b-v2-retry`
(hai ván `Cookie07`): ở ván 1 (`r1`), A nhặt RemoteBomb `(12,4)` gửi `118` (`v6=2`) và nhận `159`
(`id=1, flags=4`), giữ `>= 7.5s` không tự nổ (`blasts=[]`), rồi nhấn Space gửi `C->S 122` và nhận
`S->C 158` (`id=1`). Ở ván 2 (`r2`), B vai khách nhặt `(11,12)` lặp lại đúng chuỗi `118 -> 159`
(`id=3, flags=4`) → `122 -> 158` (`id=3`). Cả hai ván ghi `complete/elimination`, rời phòng
`86 -> 241` và `final_room_count=0`.

**F1-b Drill `149` — `confirmed` chiều B→A; chiều A→B chưa đo.** Trong `f1b-v2-retry` ván 1 (`r1`),
B dùng Drill `(12,10)` gửi `127/sub3` và A nhận `170/sub3` qua server `f1-full-rust-v2` (tiếp tục xác nhận
chiều B→A sau #647). Ở ván 2 (`r2`), A nhặt Drill tại `(11,2)` nhưng thao tác kích `Control_L` rơi vào
`r2-a-drill-ctrl-missing` nên chiều A→B vẫn chưa đo được bằng công cụ nhập hiện tại.
