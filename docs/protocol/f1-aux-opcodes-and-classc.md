# F1 phụ trợ: opcode 138/139, item Ctrl chủ động, khóa nhân vật lớn và Ship bổ sung

Toàn bộ mục này là `kr_static` trên ảnh unpack KR 00967 `cf72da11…` (image base `0x400000`, RVA = VA trừ
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
client không thể gửi 3000. Policy Loading79 hiện hành chỉ nhận byte bằng khóa đã chọn. Nhánh 4000–4030 tương ứng
byte 160–190, còn 5000 thành 136. Lượt này không sửa Rust; cần đo native trước khi mở thêm khóa.

**Confirmed.** Ở `E0F770`, `AEDE60` luôn đúng. Nếu đối số 5 bằng 95 và khóa thuộc 4000–4030, hàm gọi `B3BDE0(0)`
rồi trả về trước khởi tạo actor và trước bộ đếm spawn. Đối số này là `E02A60(E04F00(map))` từ `E5A290`
(**candidate**: loại map/mode). Ở giá trị khác, các khóa đó đi đường spawn chung. Chưa thấy guard UI riêng cho
33 hàng lớp C ngoài kiểm sở hữu `11D2F20`.

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
