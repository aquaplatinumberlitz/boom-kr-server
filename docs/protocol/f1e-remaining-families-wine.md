# F1-e: Wine hai client có di chuyển và đặt bom

Phép đo `p111` dùng KR `00967`, hai client Wine và bản release `kr-service@8fe1301`. Đây là phép đo wire của trận thường
(`roomtype=0`, đọc từ Start `S→C 121` ở cả hai client). [JSON evidence](../evidence/item-families/f1e-5fam-active-wine.json)
giữ lệnh tái lập, hash, kiểm kê frame TCP và UDP, timestamp theo peer `153`, body hex mẫu và biên nhận dọn scratch từng map.

## Phép đo và cách đọc

Ở mỗi map, cả A và B dùng `native-input.exe` để nhấn hai phím dọc ngược chiều, mỗi phím 300 ms, rồi nhấn Space và né
bom. Có hai đợt trong cửa sổ trận. Runner dùng ô bom thực tế trong `C→S 118` và tọa độ UDP để chọn đường né dựa trên
địa hình của pack. Mỗi game copy đọc `FxMap` đã kiểm hash; phía server dùng fixture terrain và floor rỗng
(`emu_choice`). Xem [phép chiếu terrain riêng](terrain.md#f1e-95-complete) để phân biệt hai phạm vi.

Một đợt xác nhận di chuyển khi CRC UDP đạt và thấy tọa độ actor thay đổi tại điểm phát lẫn điểm nhận. Việc đặt bom
cần `C→S 118` của client sau Space; `S→C 159` cùng owner, serial và ô, rồi `158` cùng ID là bằng chứng phản hồi
của fixture. `r1-mid` chụp ngay sau lượt Space thứ hai, trước lúc né. Ảnh là đối chứng hiển thị; dùng wire để
phân biệt bom chưa kịp lên hình với phím không phát gói.

Observer ghi frame từ hai hướng và giữ lại `C→S` ngoài allowlist sau `153`, không chèn gói trả lời. TCP được ráp
và kiểm CRC từ capture rồi đối chiếu với proxy. UDP nội được kiểm CRC8 ở loopback; bản mã QUIC ngoài loopback được
lưu riêng. Mốc `t153_ms=0` là `S→C 153` đầu tiên của từng client, nên hai dòng A và B có mốc riêng.

## Kết quả theo map

Trên cả sáu map, cả hai đợt của A và B đều có UDP đổi tọa độ và đúng một `C→S 118` sau Space
(`confirmed`). Cả hai cửa sổ nhận `159` và `158` khớp từng bom. TCP và proxy khớp từng frame; toàn bộ
UDP nội vượt kiểm CRC. Đã xem `frame-review.png` của cả sáu map và kiểm `scratch_removed=true` trong
mọi biên nhận dọn. `Fm_RescueET01` được chạy bù bằng cùng catalog sau khi lượt đầu hết thời gian chờ mở phòng.

| Map | Ảnh `r1-mid` / `r1-end` | Giới hạn wire và cơ chế |
|---|---|---|
| `Fm_Coloring03` | Bom ở mép trái, nhân vật đổi ô; HUD trận thường không có điểm Coloring. | `C→S 229` và `S→C 186/351` có trạng thái `not_emitted_in_roomtype0`. |
| `Aqua03` | Thấy bom và các đối tượng dạng cá; cả hai né sang ô khác. | `C→S 132/180` và `S→C 176/178/198/199` có trạng thái `not_emitted_in_roomtype0`. StageClear vẫn `unknown`. |
| `CaJJ_Wriggle01` | Hai bom thấy rõ ở đợt đầu; ảnh giữa trận có chồng hình nhân vật, còn wire phân biệt hai ID bom. | Chưa thấy opcode riêng có thể gán cho Wriggle; domain, match và tính điểm vẫn `unknown`. |
| `Fm_RescueET01` | Bom ở hai phía đối diện, hai nhân vật về phía góc sau lúc né. | `C→S 180` có trạng thái `not_emitted_in_roomtype0`; tương tác cứu alien hoặc lên tàu vẫn `unknown`. |
| `RetroForestN03` | Popup đầu trận có hình mây; hai nhân vật đặt bom rồi đổi vị trí dọc hành lang ngoài. | Ảnh chụp rời rạc không thấy cloud. XML có `cloudappeartime=10000` và `cloudappearloop=30000`; chưa biết mốc bắt đầu của script. Đồng bộ cloud vẫn `unknown`. |
| `RetroVillageN01` | Popup có hình vật cưỡi; ảnh trận thấy bom và đổi vị trí, chưa thấy cưỡi vật. | A phát một `C→S 123` dài 10 byte, giữa lệnh mũi tên và lệnh Space ở đợt hai. [CP237](udp-peer.md#rust-movement-floor) từng thấy cùng opcode và độ dài trên `ExVillage01`. Chưa thể gán gói cho thao tác ride. |

`C→S 15` xuất hiện cả trước và sau `153`, cũng có trong đối chứng thụ động; proxy chỉ giữ gói này sau `153`.
`C→S 172` và `S→C 172` đều vắng mặt. `S→C 170/sub19` được xác nhận ở A trên Coloring, Aqua, Wriggle và
Rescue ET sát lúc dừng namespace, cùng log `native_stream_eof` và cùng body 21 byte. Hai map Retro không có gói đó.
Các mẫu này không chứng minh một sự kiện riêng của script.

## Đối chứng thời gian của RetroForest

Hai client đứng yên trong `RetroForestN03` với cùng fixture `roomtype=0`. Video cả hai phía bao phủ cả mốc
10 giây và 40 giây tính từ peer `153`. Cả hai hiện popup giới thiệu; các khung đã xem sau popup không thấy cloud.
Sau `153`, TCP chỉ có các báo cáo `C→S 15`, `127/sub39` và `190`; UDP nội chỉ có selector 0. Kết quả này
giới hạn ở ca thụ động, và chưa nối đồng hồ `153` với thời điểm script thực sự bắt đầu. JSON giữ video, tần suất
xem mẫu, kiểm kê wire và biên nhận dọn riêng của đối chứng.

## Nguồn mode ứng viên của Coloring

Bản client KR 00967 đọc byte `roomtype` trong blob `S→C 121` ngay sau `dynamicindex`. Tại
`0xDC864B`, reader `0xDC7F30` chuyển byte này qua setter `0x1243BE0` vào context `+0x23C`.
`0xE7F830` đọc lại qua `0x1243C00`: giá trị này là mode ứng viên, có thể bị lọc hoặc thay theo map
và ngữ cảnh. Lời gọi ảo ở `0xE7FE9B` đi qua `CGameStage` slot `+0x30C` đến `0xE7FED0`, rồi vào
factory `0xEEF020` (`kr_static/confirmed`). Chưa có Wine xác nhận fixture đặt byte `36` đi hết đường
khởi tạo hoặc phát `229`.

## Giới hạn và phép đo tiếp

Một opcode vắng mặt chỉ có trạng thái `not_emitted_in_roomtype0` trong ca đo này. Với `S→C`, phép đo chỉ cho biết
fixture đã gửi gì. `S→C 196` và `197` vẫn xuất hiện trước `153` như các gói khởi tạo trận; sự có mặt của chúng
không chứng minh rằng script Aqua, Wriggle hay Rescue ET hoạt động. Các nhóm báo cáo, telemetry và gói đến lúc đóng
kênh được ghi riêng trong JSON với hướng và thời điểm, không dùng làm bằng chứng cơ chế riêng của map.

[Đối chiếu IDA và XML cùng build](f1e-dynamic-scripts.md) đặt Coloring ở mode `36` và xác định các nhánh ứng viên
`132` và `180` cho StageClear và Rescue ET. Cả `RetroForestN03` và `RetroVillageN01` **có** thẻ
`<retroscript>` trong pack đã pin; forest có `cloudappearloop`, village có `rideinfo`. Consumer và đồng bộ các
cơ chế đó vẫn `unknown`. Các phép đo tiếp cần mode `36` cho Coloring, điều kiện kích hoạt script object cho
Aqua/Wriggle/Rescue ET. Với RetroForest, cần tìm điều kiện khởi tạo consumer hoặc một fixture có cloud quan sát
được trước khi mở lượt đo mới. Với RetroVillage, cần đối chứng trước và sau một tương tác ride đã quan sát trên
client. Chưa có cơ sở suy luật của server KR gốc.
