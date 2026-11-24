# F1-d: Phương tiện cưỡi trong KR 00967

Kết quả dưới đây là `kr_static`. [Bằng chứng, hash và cách tái lập](../evidence/item-families/f1d-vehicles-static.json)
ghi VA của ảnh giải nén (RVA = VA − `0x400000`). Xem [kiểm kê gốc F1-d](items-stats.md#f1d-vehicles)
cho tên, trọng số và đối chiếu VN. Bài này cập nhật các nhánh client trong bảng đó. Chưa có lượt Wine
F1-d; các kết luận tĩnh chỉ áp dụng cho đường mã ghi rõ bên dưới. Quyền và luật của server gốc vẫn chưa biết.

## Nhặt và thay xe

**Confirmed.** Trên đường floor item thường, `S→C 165` đi qua `E5D570 → E63260`. Sau khi kiểm
kết quả, context và tìm actor theo slot, interface `actor+0x588` dẫn vào `B52840` (qua `F73D30`
ở lớp CAHero), rồi `E14DA0 → 1190A40 → 1192700` nếu item qua các guard. Case group 2
tại `11928A2` cast sang `CGOBomber`, lấy selector từ tail đầu của ItemAttr bằng `64DB00(0)`.
Trong nhánh xe trống, `1192957` gọi `actor/vf+0x17C → B479E0`, rồi `B978B0` khởi tạo
`CGOShip` nhúng tại `actor+0x804`. Lớp CAHero còn có nhánh đổi kind `40→44`, `43→42`
trước lookup ở `F73D30`; điều kiện xuất hiện của nhánh đó trong trận thường chưa đo.

`actor+0x584` là địa chỉ interface, không phải biến boolean: slot `+0x94` dẫn tới
`B47E30`, đọc validity qua `B323D0(ship)` (secure `ship+0x1C`). Nếu đang cưỡi,
`B48000 → B64F40` đọc selector hiện tại từ `ship+0x90`. `1370710` đòi cả
`1370670(old,new)` và `old=14, new=15`; chỉ cặp này gọi `actor+0x590/vf+0x0C`
tại `1192920`. `B4C7A0` kiểm validity, selector cũ và cặp tương thích, rồi
`B4C9D0 → B978B0` nạp lại xe, giữ trạng thái hoạt ảnh nếu cờ truyền vào bật.
Bản thân `1370670` có cả `3→4`, `5→1` và `6→7`, nhưng ba cặp này không qua
guard của pickup. Cả năm kind 40–44 đều bị từ chối thay xe ở đây, kể cả Turtle0 gặp Turtle1
và AirshipBroken gặp Airship. Kết quả này nói về trạng thái xe; việc phát claim hoặc giữ/xóa item
phải kiểm riêng. Airship còn bật cờ chặn nhặt được mô tả dưới đây.

## Tám số trong `Ship/*`

`1196A70` đọc `Attr/ItemAttr` với stride 192 cho mỗi hàng Ship. Selector nằm trước tên.
Tám số thô bên dưới nằm sau tên, theo thứ tự 1–8. `1197900` tra bằng selector, và trả hàng đầu
nếu không khớp; fallback của lookup không cấp thêm kind hợp lệ.

| Kind | Selector | Tên | Tám số thô |
|---|---:|---|---|
| 40 | 1 | Airship | `[2,10,0,1,2,0,0,0]` |
| 41 | 2 | Owl | `[1,5,0,0,0,0,0,0]` |
| 42 | 3 | Turtle0 | `[1,2,0,0,0,0,0,0]` |
| 43 | 4 | Turtle1 | `[1,9,0,0,0,0,0,0]` |
| 44 | 5 | AirshipBroken | `[2,2,0,1,2,0,0,0]` |

| Số | Offset trong hàng → trong `CGOShip` | Consumer và giới hạn |
|---|---|---|
| 1 | `+60 → +A8` | `B98680`: `B978B0` lưu boolean `raw == 2`. `80D8A0` và `B65250` phân biệt xe bay và xe đi bộ, có nhánh render riêng ở `B57340`. |
| 2 | `+6C → +B4` | `B98660(0) → B64FD0 → B46940`: tốc độ chính khi đang cưỡi. |
| 3 | `+78 → +C0` | `B98660(1) → B64F60`: tốc độ thay thế nếu đối số getter khác 0 và trường này khác 0; nếu bằng 0, dùng số 2. Đây là lựa chọn giá trị, không phải số cộng vào tốc độ. |
| 4 | `+84 → +CC` | Mã hiệu ứng thứ nhất; ở hai Airship, mã 1 đi qua `1191D20 → B49FA0` và bật `m_fAbNotEatItem` (`actor+0x618`). `B77300/B525E0` đọc qua `B49FE0` để chặn đường nhặt thường. |
| 5 | `+90 → +D8` | Mã hiệu ứng thứ hai; ở hai Airship, mã 2 đi qua `1191D20 → B49F40` và bật `m_fAbPassBox` (`actor+0x600`). Hàm nhận cờ bật 1, không chép số 2 thô vào actor. |
| 6–7 | `+9C/+A8 → +E4/+F0` | Hai vị trí tiếp theo của cùng danh sách hiệu ứng, đọc bằng `6B5860(2/3)`. Cả năm hàng dùng mã 0 (không hiệu ứng). |
| 8 | `+B4 → +FC` | Vị trí thứ năm của danh sách. `B978B0` sao chép, `1191DD0` duyệt cả năm khi dọn; đường pickup tại `1192962–11929E5` chỉ bật bốn vị trí đầu. Giá trị ở cả năm hàng là 0. |

**Confirmed.** Getter tốc độ `actor+0x584/vf+0x38 → B46940` dùng trị của Ship nếu xe
còn hợp lệ và `actor+0x918` không âm; nhánh buff hoặc trạng thái đặc biệt có thể ghi đè.
`B58B60` khởi tạo hai giá trị secure `actor+0x2998=15` và `+0x29A4=50`.
Lúc lên, `B47A90` chép 15 vào `+0x918`; lúc tháo, `B47C41` ghi số đối của 50 vào
cùng trường. Debug dump `B4F070` gọi trường này là `m_nShipRideCount`. Đây là bộ đếm
chuyển trạng thái, khác với getter tốc độ. `B47ABF` và `B47C70` đều gọi `vf+0x264`
với đối số dương `(15,0,0)` và `(50,0,0)`. **Unknown:** đích của slot là
`B784A0` ở GameBomber/CAHero và `B546C0` ở lớp cơ sở. Cả hai nhảy đến vùng không có
byte trong DB (`201554F`, `2012FA6`); chưa xác lập tác dụng của helper hay tốc độ theo pixel/giây.

## Va chạm lúc cưỡi

**Confirmed cho predicate đã nối.** `B52200`, `B3F7A0`, `B5BD40/B5C040` lấy cờ
qua `actor+0x584/vf+0x80 → B49F80` và gọi `EA0B80` với `a3 = !m_fAbPassBox`.
Với `a4=0` (slot `actor/vf+0x1DC → B51880` trả 0 ở ba lớp đã đối chiếu), Airship
và AirshipBroken dùng nhánh `+1D3C` và hai phép kiểm object động `EA1540/EA1780`;
Owl/Turtle dùng `EA1930/EA1AE0`, tức mask `+1D60/+1D84`. Hai nhánh đều kiểm biên map
và `EA1230` trước, rồi caller còn kiểm tile directional qua `E9F9B0`. Do đó không có
bypass toàn ô: Waterway có mask `+1D24` vẫn bị chặn tại predicate này; ô tĩnh type 1 có
cờ `+1D3C` cũng chặn Airship. Xem [cờ khởi tạo và đối chứng Waterway](terrain.md#rust-tomb12-boundary).
Với bom, `EA1540` duyệt mảng `E12C80` của tất cả slot, cùng mảng mà `E14360 → E17650` dùng
để tìm bom cho [158](bombs.md#bomb-serial). Nó chặn nếu có bom valid ở ô đích. Nhánh đi bộ
chặn khi `+1D60` bật, cùng mask mà `E17BD0` kiểm trước khi tra bom theo ô.
**Candidate:** Airship qua thùng phá được nào mà chỉ bật lớp cản dành cho xe đi bộ.
**Unknown:** ma trận theo loại thùng, biến thể bom, cách thoát ô đang đứng trên bom của mình, và ô có
nhiều lớp chồng nhau. Các kết luận trên chỉ nói về predicate xét ô đích.

## Trúng đòn, tháo và reset

**Confirmed.** Tick stage `E7C7B0 → E6F160` xét hit bằng `E15230 → 1269E80 → E15280`, rồi gọi
`actor/vf+0x244` tại `E6F338`. Ở GameBomber/CAHero, `B77FD0 → B5E120` chọn case 1
khi xe valid và `m_nShipRideCount >= 0`, nếu qua guard trước đó. Case 1 gọi `vf+0x180`
tại `B78182` thay vì chuyển sang nhánh nhốt; lớp cơ sở `B53AD0` cũng có case 1 tương ứng.
Đường này hấp thụ lần hit hợp lệ tại client và không chờ một gói xác nhận tháo. `B73680 → B47B80`
đặt bộ đếm âm và gọi `ship/vf+0x60 → B97EE0` tại `B47D45` để cập nhật hoạt ảnh.
Tick `B44000` kéo số âm về 0 rồi gọi `B47EB0` tại `B448A2`, tắt Ship qua
`ship/vf+0x0C(0)` và gỡ hiệu ứng qua `1191DD0`. `B73680` còn kiểm ô chồng thùng bằng
`E9FCF0/E9FF80` và có thể chỉnh vị trí về ô của actor. Chưa đo khoảng thời gian hay trạng thái ở lần hit kế.

Các caller đã phân loại gồm handler hit `B77FD0/B53AD0`, `S→C 170/sub16 → E866F0`,
`170/sub10 → E87A00 → B74B30`, `S→C 732 → E6A820 → B57230`,
tile effect `1192040/case 2`, và các đường đổi trạng thái `B42880/B48610`.
Bản quét VA giữ thêm các biến thể lớp. Chưa tìm thấy đường phím tự xuống trong đường input thường;
việc không có đường phím ở mọi mode vẫn **unknown**.

**Confirmed.** `B79DC0 → B58B60 → B3D1D0` tạo/khởi tạo lại actor, gọi `ship/vf+0x0C(0)`
tại `B3D56E` và đặt `+0x918/+0x924` về 0. `B3DCB0` (actor `vf+0xA4`) và
`B7B100` (GameBomber `vf+0x338`) cũng thực hiện reset này. `+0x584` vẫn là interface;
điều trở về 0 là kết quả getter `+0x94`. [Start 121](load-phase.md#start-reply) đổi sang
`CGameStage` ở `DC937B`. Đường spawn `E59F30 → E5A290 → E0F6C0 → E0F770` gọi
`B3D170` tại `E0F970`; ba lớp đã đối chiếu đều vào `B3CC80` (GameBomber qua `B762A0`,
CAHero qua `F73190 → B762A0`), rồi gọi `actor/vf+0xA0 → B3D1D0` tại `B3CC9A`.
Chưa chốt lúc nào actor cũ đang cưỡi bị tắt ở mọi đường kết thúc. Opcode 136 đi vào default
trong hai dispatcher room/game đã kiểm, nên chưa gán nó vai trò reset.

## Đồng bộ hai view

- **Confirmed tĩnh:** `S→C 381` vào `E68BB0`, nhưng `703A80` luôn trả AL=0; đi ra trước
  parser. Ở đường tích cực của `165`, lookup theo slot cho phép callback nhặt tác động actor của
  peer. Chưa đo peer lên xe từ `165` trên Wine hoặc policy server phát gói.
- **Confirmed tĩnh:** Sau khi đã bắt đầu tháo, `B73680` kiểm `actor/vf+0x1D8 == 0`,
  rồi `B73704/B73710 → B53010` ghi **`C→S 127/sub16`** và `B73719 → 1164D50` gửi.
  Thân writer dùng prefix 14 byte đã ghi ở [127](life-results.md#round-result), không nối blob ở callsite này.
  Receiver **`S→C 170/sub16`** đọc đúng header `u8 subtype, u8 slot, u16 x, u16 y` theo
  getter KR (tọa độ sign-extend), không đọc trường sau header. `E866F0` tra actor, yêu cầu
  Ship valid và counter không âm, đặt tọa độ rồi gọi cùng `vf+0x180`.
- **Candidate:** Phát `165` cho hai view để lên và chuyển tiếp `127/sub16 → 170/sub16`
  để tháo. Cần đối chứng để biết peer có tự tháo từ mô phỏng tia hay cần `170` trong route được thử.
  **Unknown:** Không thấy selector/trạng thái tháo trong consumer pose đã xem (`B4E6F0`);
  [UDP peer](udp-peer.md#peer-duo) chưa được loại trừ như đường bổ sung. Không gửi lại nguyên body
  `127` làm `170`. Chưa có bằng chứng parity server, cũng chưa mở hỗ trợ Rust từ record này.

Đo tiếp bằng hai client cô lập. `p94`: giữ cùng nhân vật, lấy mốc đi bộ rồi đo cả năm kind,
một lượt thử nhặt chồng trên xe mặt đất; so quan sát với `165` chỉ gửi tới người nhặt và gửi cả hai.
Dùng một đường đối chứng với ô trống, thùng phá được, ô tĩnh, bom (đổi owner) và Waterway; ghi
getter tốc độ và vị trí chứ không chỉ ảnh. `p95`: một lần trúng tia khi đi bộ, một lần khi đang cưỡi;
so chặn/chuyển riêng `170/sub16` trong khi giữ UDP, ghi `127` thật, setter/tick và hai view;
nếu peer tự tháo, cô lập thêm tác động của UDP hoặc tia. Thử phím riêng khi không có đòn và
giữ một xe tới hết vòng để đo trước/sau Start kế.
