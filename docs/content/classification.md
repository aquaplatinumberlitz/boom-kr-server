# Xếp loại nội dung KR 00967

Bảng sinh từ [kiểm kê pack](../CONTENT.md). **A**: chạy được với server hiện tại trong phạm vi ghi ở dòng
tương ứng; **B**: cần thêm dữ liệu, wire hoặc luật đã hiểu; **C**: còn ẩn số cần RE.
Bậc của mẫu không tự áp dụng cho cả họ. Dấu **—** là ngoài phạm vi. F1 đang mở; F2–F7 tiếp tục chờ.

Nguồn: số liệu pack và wiki được định tuyến ở CONTENT; đối chiếu `boomkr-re/docs/kr-data.md` §2, §6, §12–13
và `boom-re/docs/items.md`, `map.md` để định hướng, không nhập opcode/ID VN. IDA trên bản sao unpack 00967
(hash input/DB trong JSON) có xref `Attr/ItemAttr` tại VA `0x1196ABF`, `Attr/BomberAttr` tại
VA `0x13501B6`; đây chỉ là dấu vết reader tĩnh, không xác định wire, drop hoặc quyền server.
Tên trùng trong bảng Attr/Ship được giữ theo số dòng; wiki chỉ giúp ghép tên.
Lớp B của nhân vật dựa trên layout và luồng chọn đã biết; giá trị/cap từng dòng vẫn cần đối chiếu KR.

| Nhóm | A | B | C | Ngoài |
|---|---:|---:|---:|---:|
| Họ map | 1 | 85 | 247 | 0 |
| Nhóm item | 1 | 2 | 75 | 0 |
| Phương tiện | 0 | 0 | 28 | 0 |
| Nhân vật | 4 | 30 | 33 | 0 |
| Mode | 1 | 0 | 31 | 0 |
| Hệ thống | 4 | 5 | 63 | 1 |

Họ map F1: 1 A, 85 B, 22 C. Có 70 map đạt A ở cấp map.
[Kiểm F1-a với adapter pack](../evidence/rust-service/f1-pack-map-adapters.json) giữ danh sách map, phép thử nội dung và phạm vi họ đã nghiệm thu.
Bốn map trong số đó đã lặp lại trọn chu trình với cặp Windows/Wine ở [CP387](../evidence/checkpoints/records/CP387.json).

### Phép thử rẻ cho F1

Cột Mẫu chỉ định một map mỗi họ. [CP381](../evidence/checkpoints/records/CP381.json) giữ đối chứng
ban đầu. [CP382](../evidence/checkpoints/records/CP382.json) ghi
đối chứng chu trình. [CP384](../evidence/checkpoints/records/CP384.json) mở rộng mẫu trên
catalog dựng từ pack, qua TLS/relay trong LAN cô lập. Mẫu A có Start/Load, tương tác trong trận, kết quả
và về phòng. Cột quan sát chỉ ghi nội dung thấy hoặc đã đối chứng, kèm giới hạn nếu có. Giữ họ ở lớp B
nếu còn map chưa qua chu trình hoặc còn thiếu căn cứ về item, đồ rơi và địa hình đặc biệt của họ.
Route cũ chỉ được dùng theo phạm vi [ma trận G1](../evidence/rust-service/g1-acceptance-matrix.json).

## Họ map

| Nội dung | Đợt | Lớp | Số lượng / mẫu | Server cần làm |
|---|---|---|---|---|
| Aqua | F1 | C | 9 · `Aqua01` | Chưa biết quyền chạy script, kiểu ô, item hoặc kích thước chưa có profile. |
| Camp | F1 | B | 13 · `Camp01`; G1: Camp08 | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. **Camp01 (A):** Hai client Wine chọn map, Load, đặt bom, nhận kết quả và về phòng qua TLS/relay. **Camp08 (A):** Hai client KR trên Wine qua trọn ván với adapter pack F1-a. |
| Cemetery | F1 | B | 7 · `Cemetery01` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. **Cemetery01 (A):** Hai client Wine chọn map, Load, đặt bom, nhận kết quả và về phòng qua TLS/relay. |
| Cookie | F1 | B | 7 · `Cookie01` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. **Cookie01 (A):** Hai client Wine chọn map, Load, đặt bom, nhận kết quả và về phòng qua TLS/relay. **Cookie01 (A):** Hai client KR trên Wine qua trọn ván với adapter pack F1-a. **Cookie03 (A):** Hai client KR trên Wine qua trọn ván với adapter pack F1-a. **Cookie05 (A):** Hai client KR trên Wine qua trọn ván với adapter pack F1-a. **Cookie06 (A):** Hai client KR trên Wine qua trọn ván với adapter pack F1-a. |
| Desert | F1 | B | 7 · `Desert00` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. **Desert00 (A):** Hai client Wine chọn map, Load, đặt bom, nhận kết quả và về phòng qua TLS/relay. **Desert07 (A):** Hai client KR trên Wine qua trọn ván với adapter pack F1-a. **Desert14 (A):** Hai client KR trên Wine qua trọn ván với adapter pack F1-a. |
| Factory | F1 | B | 7 · `Factory00`; G1: Factory07 | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. **Factory07 (A):** Hai client Wine chọn map, Load, đặt bom, nhận kết quả và về phòng qua TLS/relay. **Factory07 (A):** Hai client KR trên Wine qua trọn ván với adapter pack F1-a. |
| Forest | F1 | B | 12 · `Forest02`; G1: Forest07 | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. **Forest02 (A):** Hai client Wine chọn map, Load, đặt bom, nhận kết quả và về phòng qua TLS/relay. **Forest07 (A):** Hai client KR trên Wine qua trọn ván với adapter pack F1-a. **Forest18 (A):** Hai client KR trên Wine qua trọn ván với adapter pack F1-a. **Forest24 (A):** Hai client KR trên Wine qua trọn ván với adapter pack F1-a. |
| Ice | F1 | B | 7 · `Ice00` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. **Ice00 (A):** Hai client Wine chọn map, Load, đặt bom, nhận kết quả và về phòng qua TLS/relay. **Ice06 (A):** Hai client KR trên Wine qua trọn ván với adapter pack F1-a. |
| Island | F1 | B | 15 · `Island01` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. **Island01 (A):** Hai client Wine chọn map, Load, đặt bom, nhận kết quả và về phòng qua TLS/relay. |
| Kfg | F1 | B | 3 · `Kfg02` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. **Kfg02 (A):** Hai client Wine chọn map, Load, đặt bom, nhận kết quả và về phòng qua TLS/relay. |
| LovelyRoom | F1 | C | 10 · `LovelyRoom01` | Chưa biết quyền chạy script, kiểu ô, item hoặc kích thước chưa có profile. |
| Mine | F1 | B | 7 · `Mine00` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. **Mine00 (A):** Hai client Wine chọn map, Load, đặt bom, nhận kết quả và về phòng qua TLS/relay. |
| Ocean | F1 | B | 7 · `Ocean00` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. **Ocean00 (A):** Hai client Wine chọn map, Load, đặt bom, nhận kết quả và về phòng qua TLS/relay. |
| PangLand | F1 | C | 10 · `PangLand00` | Chưa biết quyền chạy script, kiểu ô, item hoặc kích thước chưa có profile. |
| Pangland | F1 | C | 1 · `Pangland14` | Chưa biết quyền chạy script, kiểu ô, item hoặc kích thước chưa có profile. |
| Patrit | F1 | B | 8 · `Patrit01`; G1: Patrit14 | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. **Patrit01 (A):** Hai client Wine chọn map, Load, đặt bom, nhận kết quả và về phòng qua TLS/relay. **Patrit01 (A):** Hai client KR trên Wine qua trọn ván với adapter pack F1-a. **Patrit14 (A):** Hai client KR trên Wine qua trọn ván với adapter pack F1-a. **Patrit17 (A):** Hai client KR trên Wine qua trọn ván với adapter pack F1-a. |
| Space | F1 | C | 8 · `Space01` | Chưa biết quyền chạy script, kiểu ô, item hoặc kích thước chưa có profile. |
| Tomb | F1 | B | 5 · `Tomb01`; G1: Tomb12 | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. **Tomb01 (A):** Hai client Wine chọn map, Load, đặt bom, nhận kết quả và về phòng qua TLS/relay. **Tomb12 (A):** Hai client KR trên Wine qua trọn ván với adapter pack F1-a. |
| Village | F1 | B | 6 · `Village00`; G1: Village10 | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. **Village00 (A):** Hai client Wine chọn map, Load, đặt bom, nhận kết quả và về phòng qua TLS/relay. **Village16 (A):** Hai client KR trên Wine qua trọn ván với adapter pack F1-a. |
| Xmas | F1 | B | 14 · `Xmas01` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. **Xmas01 (A):** Hai client Wine chọn map, Load, đặt bom, nhận kết quả và về phòng qua TLS/relay. **Xmas03 (A):** Hai client KR trên Wine qua trọn ván với adapter pack F1-a. **Xmas06 (A):** Hai client KR trên Wine qua trọn ván với adapter pack F1-a. **Xmas15 (A):** Hai client KR trên Wine qua trọn ván với adapter pack F1-a. |
| forest | F1 | B | 1 · `forest21` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. **forest21 (A):** Hai client Wine vào trận, đặt bom, nhận kết quả và về phòng qua TLS/relay. Thấy cụm cây và hoa nhiều màu chia hai phía. **forest21 (A):** Hai client KR trên Wine qua trọn ván với adapter pack F1-a. |
| kfg | F1 | B | 3 · `kfg01` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. |
| mine | F1 | B | 1 · `mine23` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. |
| patrit | F1 | B | 1 · `patrit19` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. |
| BSBigCamp | F1 | C | 1 · `BSBigCamp08` | Chưa biết quyền chạy script, kiểu ô, item hoặc kích thước chưa có profile. |
| BSBigPatrit | F1 | C | 1 · `BSBigPatrit14` | Chưa biết quyền chạy script, kiểu ô, item hoặc kích thước chưa có profile. |
| BSBigVillage | F1 | C | 1 · `BSBigVillage10` | Chưa biết quyền chạy script, kiểu ô, item hoặc kích thước chưa có profile. |
| BSSmallCamp | F1 | B | 1 · `BSSmallCamp08` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. **BSSmallCamp08 (A):** Hai client Wine vào trận, đặt bom, nhận kết quả và về phòng qua TLS/relay. Thấy dải lưới và thùng trong vùng đấu thu nhỏ. |
| BSSmallPatrit | F1 | B | 1 · `BSSmallPatrit14` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. **BSSmallPatrit14 (A):** Hai client Wine vào trận, đặt bom, nhận kết quả và về phòng qua TLS/relay. Thấy tường bao và cột trong vùng đấu. **BSSmallPatrit14 (A):** Windows VM và Wine tải map, tới kết quả và về phòng; chỉ xác nhận chu trình. **BSSmallPatrit14 (A):** Hai client KR trên Wine qua trọn ván với adapter pack F1-a. |
| BSSmallVillage | F1 | B | 1 · `BSSmallVillage10` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. **BSSmallVillage10 (A):** Hai client Wine vào trận, đặt bom, nhận kết quả và về phòng qua TLS/relay. Thấy hàng cây bao quanh cụm nhà và thùng. |
| CaJJ_CAPong | F1 | B | 5 · `CaJJ_CAPong01` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. **CaJJ_CAPong01 (A):** Hai client Wine vào trận, đặt bom, nhận kết quả và về phòng qua TLS/relay. Thấy nền lưới với chướng ngại hình vuông và tròn. |
| CaJJ_Camp | F1 | B | 1 · `CaJJ_Camp08` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. **CaJJ_Camp08 (A):** Hai client Wine vào trận, đặt bom, nhận kết quả và về phòng qua TLS/relay. Thấy dải lưới và cột trong vùng đấu. |
| CaJJ_Clover | F1 | A | 5 · `CaJJ_Clover01` | Cả 5 map qua trọn ván với hai client trong LAN cô lập. Hai client thấy thùng thường vỡ ở CaJJ_Clover03 và Movable00 vỡ ở CaJJ_Clover02. Pack không có placement hay hàng rơi trọng số dương. Giới hạn: hai người, không suy mọi ô hoặc roster 8. **CaJJ_Clover01 (A):** Hai client Wine vào trận, đặt bom, nhận kết quả và về phòng qua TLS/relay. Thấy cụm chướng ngại hình cỏ bốn lá. **CaJJ_Clover01 (A):** Windows VM và Wine tải map, tới kết quả và về phòng; chỉ xác nhận chu trình. **CaJJ_Clover01 (A):** Hai client KR trên Wine qua trọn ván với adapter pack F1-a. **CaJJ_Clover02 (A):** Hai client KR trên Wine qua trọn ván với adapter pack F1-a. **CaJJ_Clover03 (A):** Hai client KR trên Wine qua trọn ván với adapter pack F1-a. **CaJJ_Clover04 (A):** Hai client KR trên Wine qua trọn ván với adapter pack F1-a. **CaJJ_Clover05 (A):** Hai client KR trên Wine qua trọn ván với adapter pack F1-a. |
| CaJJ_Factory | F1 | B | 1 · `CaJJ_Factory07` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. **CaJJ_Factory07 (A):** Hai client Wine vào trận, đặt bom, nhận kết quả và về phòng qua TLS/relay. Thấy lưới và cột có sọc trong vùng đấu. **CaJJ_Factory07 (A):** Hai client KR trên Wine qua trọn ván với adapter pack F1-a. |
| CaJJ_Forest | F1 | B | 1 · `CaJJ_Forest07` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. **CaJJ_Forest07 (A):** Hai client Wine vào trận, đặt bom, nhận kết quả và về phòng qua TLS/relay. Thấy ao ở góc dưới bên phải. |
| CaJJ_Ocean | F1 | B | 1 · `CaJJ_Ocean24` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. **CaJJ_Ocean24 (A):** Hai client Wine vào trận, đặt bom, nhận kết quả và về phòng qua TLS/relay. Thấy nền nước, cụm đá và các khối sáng. |
| CaJJ_Patrit | F1 | B | 1 · `CaJJ_Patrit14` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. **CaJJ_Patrit14 (A):** Hai client Wine vào trận, đặt bom, nhận kết quả và về phòng qua TLS/relay. Thấy các dãy thùng, cột và giàn ở giữa sân. |
| CaJJ_RetroCamp | F1 | B | 1 · `CaJJ_RetroCamp08` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. **CaJJ_RetroCamp08 (A):** Hai client Wine vào trận, đặt bom, nhận kết quả và về phòng qua TLS/relay. Thấy dải lưới và thùng gỗ; Fire đặt thử chưa có yêu cầu nhặt từ client. |
| CaJJ_RetroFactory | F1 | B | 1 · `CaJJ_RetroFactory07` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. **CaJJ_RetroFactory07 (A):** Hai client Wine vào trận, đặt bom, nhận kết quả và về phòng qua TLS/relay. Thấy cột và dải chướng ngại có sọc. **CaJJ_RetroFactory07 (A):** Hai client KR trên Wine qua trọn ván với adapter pack F1-a. |
| CaJJ_RetroForest | F1 | B | 1 · `CaJJ_RetroForest07` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. **CaJJ_RetroForest07 (A):** Hai client Wine vào trận, đặt bom, nhận kết quả và về phòng qua TLS/relay. Thấy ao ở góc dưới bên phải. |
| CaJJ_RetroPatrit | F1 | B | 1 · `CaJJ_RetroPatrit14` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. **CaJJ_RetroPatrit14 (A):** Hai client Wine vào trận, đặt bom, nhận kết quả và về phòng qua TLS/relay. Thấy cột và giàn ở giữa sân. |
| CaJJ_RetroVillage | F1 | B | 1 · `CaJJ_RetroVillage10` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. **CaJJ_RetroVillage10 (A):** Hai client Wine vào trận, đặt bom, nhận kết quả và về phòng qua TLS/relay. Thấy dãy nhà, cây và thùng dọc đường giữa. |
| CaJJ_Space | F1 | C | 2 · `CaJJ_Space02` | Chưa biết quyền chạy script, kiểu ô, item hoặc kích thước chưa có profile. |
| CaJJ_Tomb | F1 | B | 1 · `CaJJ_Tomb12` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. **CaJJ_Tomb12 (A):** Hai client Wine vào trận, đặt bom, nhận kết quả và về phòng qua TLS/relay. Thấy rãnh dài và cầu ngang. |
| CaJJ_Village | F1 | B | 1 · `CaJJ_Village10` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. **CaJJ_Village10 (A):** Hai client Wine vào trận, đặt bom, nhận kết quả và về phòng qua TLS/relay. Thấy các dãy nhà, cây và thùng ở đường giữa. |
| CaJJ_Wriggle | F1 | C | 7 · `CaJJ_Wriggle01` | Chưa biết quyền chạy script, kiểu ô, item hoặc kích thước chưa có profile. |
| EventCamp | F1 | B | 1 · `EventCamp00` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. **EventCamp00 (A):** Hai client Wine vào trận, đặt bom, nhận kết quả và về phòng qua TLS/relay. Thấy dãy rào và lưới bao quanh vùng giữa. |
| EventFactory | F1 | B | 1 · `EventFactory00` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. **EventFactory00 (A):** Hai client Wine vào trận, đặt bom, nhận kết quả và về phòng qua TLS/relay. Thấy cột có biểu tượng và dải có sọc. |
| EventForest | F1 | B | 1 · `EventForest00` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. |
| EventOcean | F1 | B | 1 · `EventOcean00` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. **EventOcean00 (A):** Hai client Wine vào trận, đặt bom, nhận kết quả và về phòng qua TLS/relay. Thấy các cầu gỗ nối vùng đất. |
| EventPatrit | F1 | B | 1 · `EventPatrit00` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. |
| ExBula | F1 | B | 2 · `ExBula01` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. |
| ExCamp | F1 | B | 2 · `ExCamp01` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. **ExCamp01 (A):** Hai client Wine vào trận, đặt bom, nhận kết quả và về phòng qua TLS/relay. Thấy dải lưới và chướng ngại hình lốp. |
| ExFactory | F1 | B | 2 · `ExFactory01` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. **ExFactory01 (A):** Hai client Wine vào trận, đặt bom, nhận kết quả và về phòng qua TLS/relay. Thấy các cột có biểu tượng quanh vùng đấu. |
| ExForest | F1 | B | 1 · `ExForest01` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. **ExForest01 (A):** Hai client Wine vào trận, đặt bom, nhận kết quả và về phòng qua TLS/relay. Thấy ao và cây lớn ở giữa. |
| ExPatrit | F1 | B | 2 · `ExPatrit01` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. **ExPatrit01 (A):** Hai client Wine vào trận, đặt bom, nhận kết quả và về phòng qua TLS/relay. Thấy cột tròn và công trình ở giữa. **ExPatrit02 (A):** Hai client KR trên Wine qua trọn ván với adapter pack F1-a. |
| ExTomb | F1 | B | 2 · `ExTomb01` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. **ExTomb01 (A):** Hai client Wine vào trận, đặt bom, nhận kết quả và về phòng qua TLS/relay. Thấy rãnh tối và cầu ở giữa. **ExTomb01 (A):** Windows VM và Wine tải map, tới kết quả và về phòng; chỉ xác nhận chu trình. |
| ExVillage | F1 | B | 2 · `ExVillage01` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. **ExVillage02 (A):** Hai client Wine vào trận, đặt bom, nhận kết quả và về phòng qua TLS/relay. Thấy cụm nhà, thùng và cụm hình cây ở giữa. **ExVillage01 (A):** Hai client KR trên Wine qua trọn ván với adapter pack F1-a. |
| Fm_AIBtlDesert | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Fm_AIBtlForest | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Fm_AIBtlPatrit | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Fm_AIBtlTomb | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Fm_CAPong | F1 | B | 3 · `Fm_CAPong01` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. **Fm_CAPong01 (A):** Hai client Wine vào trận, đặt bom, nhận kết quả và về phòng qua TLS/relay. Thấy nền lưới với chướng ngại hình vuông và tròn. |
| Fm_Camp | F1 | B | 1 · `Fm_Camp08` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. **Fm_Camp08 (A):** Hai client Wine vào trận, đặt bom, nhận kết quả và về phòng qua TLS/relay. Thấy nhân vật tiến đến cạnh thùng; có báo cáo 142, chưa xác nhận MaxPower lộ hoặc được nhặt. |
| Fm_Coloring | F1 | C | 1 · `Fm_Coloring03` | Chưa biết quyền chạy script, kiểu ô, item hoặc kích thước chưa có profile. |
| Fm_Patrit | F1 | B | 1 · `Fm_Patrit14` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. **Fm_Patrit14 (A):** Hai client Wine vào trận, đặt bom, nhận kết quả và về phòng qua TLS/relay. Thấy các dãy thùng, cột và giàn ở giữa sân. |
| Fm_RescueET | F1 | C | 3 · `Fm_RescueET01` | Chưa biết quyền chạy script, kiểu ô, item hoặc kích thước chưa có profile. |
| Fm_Space | F1 | C | 2 · `Fm_Space01` | Chưa biết quyền chạy script, kiểu ô, item hoặc kích thước chưa có profile. |
| Fm_Tomb | F1 | B | 1 · `Fm_Tomb22` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. **Fm_Tomb22 (A):** Hai client Wine vào trận, đặt bom, nhận kết quả và về phòng qua TLS/relay. Thấy dãy cột và cụm tượng bao quanh vùng giữa. |
| Fm_Village | F1 | B | 1 · `Fm_Village10` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. |
| Fm_Wriggle | F1 | C | 2 · `Fm_Wriggle02` | Chưa biết quyền chạy script, kiểu ô, item hoặc kích thước chưa có profile. |
| LJ_Camp | F1 | B | 1 · `LJ_Camp08` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. |
| LJ_Factory | F1 | B | 1 · `LJ_Factory07` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. |
| LJ_Forest | F1 | B | 1 · `LJ_Forest07` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. |
| LJ_Patrit | F1 | B | 1 · `LJ_Patrit14` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. |
| LJ_Tomb | F1 | B | 1 · `LJ_Tomb12` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. |
| LJ_Village | F1 | B | 1 · `LJ_Village10` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. |
| NL_Camp | F1 | B | 1 · `NL_Camp08` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. |
| NL_Factory | F1 | B | 1 · `NL_Factory07` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. |
| NL_Forest | F1 | B | 1 · `NL_Forest07` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. |
| NL_Patrit | F1 | B | 1 · `NL_Patrit14` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. |
| NL_Tomb | F1 | B | 1 · `NL_Tomb12` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. |
| NL_Village | F1 | B | 1 · `NL_Village10` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. |
| NXCL_Aqua | F1 | B | 1 · `NXCL_Aqua01` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. |
| NXCL_Camp | F1 | B | 1 · `NXCL_Camp08` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. |
| NXCL_Cookie | F1 | B | 1 · `NXCL_Cookie02` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. |
| NXCL_Desert | F1 | B | 1 · `NXCL_Desert00` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. |
| NXCL_Factory | F1 | B | 1 · `NXCL_Factory07` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. |
| NXCL_Forest | F1 | B | 1 · `NXCL_Forest07` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. |
| NXCL_Ice | F1 | B | 1 · `NXCL_Ice13` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. |
| NXCL_Mine | F1 | B | 1 · `NXCL_Mine01` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. |
| NXCL_Ocean | F1 | B | 1 · `NXCL_Ocean02` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. **NXCL_Ocean02 (A):** Hai client Wine vào trận, đặt bom, nhận kết quả và về phòng qua TLS/relay. Thấy nền nước, các khối sáng và cụm đá giữa. **NXCL_Ocean02 (A):** Windows VM và Wine tải map, tới kết quả và về phòng; chỉ xác nhận chu trình. |
| NXCL_PangLand | F1 | C | 1 · `NXCL_PangLand15` | Chưa biết quyền chạy script, kiểu ô, item hoặc kích thước chưa có profile. |
| NXCL_Patrit | F1 | B | 1 · `NXCL_Patrit14` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. |
| NXCL_Tomb | F1 | B | 1 · `NXCL_Tomb12` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. |
| NXCL_Village | F1 | B | 1 · `NXCL_Village10` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. |
| NoviceFactory | F1 | B | 1 · `NoviceFactory07` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. |
| NoviceForest | F1 | B | 1 · `NoviceForest07` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. |
| NoviceMine | F1 | B | 1 · `NoviceMine21` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. |
| NovicePatrit | F1 | B | 1 · `NovicePatrit14` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. |
| NoviceVillage | F1 | B | 1 · `NoviceVillage10` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. |
| RetroCampN | F1 | C | 6 · `RetroCampN01` | Chưa biết quyền chạy script, kiểu ô, item hoặc kích thước chưa có profile. |
| RetroFactoryN | F1 | C | 6 · `RetroFactoryN01` | Chưa biết quyền chạy script, kiểu ô, item hoặc kích thước chưa có profile. |
| RetroForestN | F1 | C | 6 · `RetroForestN01` | Chưa biết quyền chạy script, kiểu ô, item hoặc kích thước chưa có profile. |
| RetroPatritN | F1 | C | 6 · `RetroPatritN01` | Chưa biết quyền chạy script, kiểu ô, item hoặc kích thước chưa có profile. |
| RetroVillageN | F1 | C | 6 · `RetroVillageN01` | Chưa biết quyền chạy script, kiểu ô, item hoặc kích thước chưa có profile. |
| RetroWriggleN | F1 | C | 5 · `RetroWriggleN01` | Chưa biết quyền chạy script, kiểu ô, item hoặc kích thước chưa có profile. |
| WL_Camp | F1 | B | 1 · `WL_Camp08` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. |
| WL_Patrit | F1 | B | 1 · `WL_Patrit14` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. |
| WL_Space | F1 | C | 1 · `WL_Space08` | Chưa biết quyền chạy script, kiểu ô, item hoặc kích thước chưa có profile. |
| WL_Tomb | F1 | B | 1 · `WL_Tomb12` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. |
| WinningStreak_Patrit | F1 | B | 1 · `WinningStreak_Patrit14` | Thử ánh xạ mã map, spawn, ô và item bằng adapter G1; cần đối chiếu hành vi chưa có. |
| Bonus | F3 | C | 12 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| BonusBig | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| BonusBigRPG | F3 | C | 2 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| BonusBigReward | F3 | C | 13 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| BonusCP_18th | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| BonusCP_LINE | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| BonusCp | F3 | C | 3 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| BonusExp | F3 | C | 3 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| BonusLargeGourd | F3 | C | 2 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| BonusMazeMap | F3 | C | 2 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| BonusPCRoom | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Bonus_16th | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Bonus_19th | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Hdn_Alche_Aqua | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Hdn_Alche_Cam | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Hdn_Alche_Ceme | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Hdn_Alche_Fac | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Hdn_Alche_Kart | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Hdn_Alche_Rab | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Hdn_Alice | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Hdn_BlackLucy | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Hdn_Caffe | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Hdn_Camp | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Hdn_Clover | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Hdn_DarkCastle | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Hdn_Factory | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Hdn_Ice | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Hdn_Mine | F3 | C | 2 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Hdn_Ocean | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Hdn_Patrit | F3 | C | 2 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Hdn_R_Camp | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Hdn_R_Factory | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Hdn_R_Forest | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Hdn_R_Patrit | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Hdn_R_Village | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Hdn_Smash | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Hdn_Square | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Hdn_SuperFight | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Hdn_Tomb | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Hdn_Village | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Hdn_Wriggle | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Hidden | F3 | C | 6 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| HiddenCatchTest | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| CAPassShopMap | F6? | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| CustomMap15x | F7 | C | 2 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| CustomMap21x | F7 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| CustomMap31x | F7 | C | 2 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| DailyMissionShop | F6? | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| LobbyCaffe | F5? | C | 5 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| LobbyClothing | F5? | C | 2 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| LobbyJewelry | F5? | C | 2 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| LobbyPet | F5? | C | 2 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| LobbySquare | F5? | C | 9 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| LobbySquareFishingTest | F6? | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| MagicMarketmap | F6? | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| SelectionBox | F6? | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Square | F5? | C | 2 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Tutorial | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| shopmap | F4? | C | 2 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| tlshopmap | F4? | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Baseball | F2? | C | 10 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Bula | F2? | C | 7 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| CAPong | F2? | C | 5 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Clover | F2? | C | 6 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Coloring | F2? | C | 5 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Gumiho | F2? | C | 11 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Hamster | F2? | C | 8 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Kart | F2? | C | 8 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| PopcadeA | F2? | C | 5 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| PopcadeB | F2? | C | 5 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| PopcadeC | F2? | C | 5 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Rabbit | F2? | C | 6 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| RescueET | F2? | C | 3 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| SnowyFields | F2? | C | 18 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| SuperFighter | F2? | C | 2 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Wriggle | F2? | C | 7 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Zombie | F2? | C | 6 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Zzolman | F2? | C | 5 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Fm_MonGolem | F3 | C | 3 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Fm_MonSeal | F3 | C | 3 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Fm_MonSnowBear | F3 | C | 3 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Fm_MonTurtle | F3 | C | 3 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Fm_Monster | F3 | C | 3 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| MonCity | F3 | C | 9 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| MonGolem | F3 | C | 27 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| MonNoviceGolem | F3 | C | 2 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| MonNoviceSnow | F3 | C | 2 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| MonNoviceTako | F3 | C | 2 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| MonNoviceTurtle | F3 | C | 2 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| MonPen | F3 | C | 34 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| MonRobot | F3 | C | 27 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| MonS2Fire | F3 | C | 15 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| MonS2Sand | F3 | C | 5 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| MonS2Sand01Boss | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| MonS2Sand02Boss | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| MonS2Sand03Boss | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| MonS2Sand04Boss | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| MonS2Sand05Boss | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| MonS2Sand06Boss | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| MonSeal | F3 | C | 37 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| MonSnowBear | F3 | C | 30 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| MonTurtle | F3 | C | 27 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| MonUnlimited | F3 | C | 10 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| MonWarsBBR | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| MonWarsBG | F3 | C | 4 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| MonWarsBGB | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| MonWarsBLB | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| MonWarsBM | F3 | C | 4 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| MonWarsBOB | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| MonWarsBPB | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| MonWarsBR | F3 | C | 4 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| MonWarsBS | F3 | C | 2 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| MonWarsBSB | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| MonWarsBT | F3 | C | 4 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| MonWarsBTB | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| MonWarsHGB | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| MonWarsHH | F3 | C | 2 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| MonWarsHLB | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| MonWarsHM | F3 | C | 2 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| MonWarsHOB | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| MonWarsHP | F3 | C | 5 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| MonWarsHPB | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| MonWarsHR | F3 | C | 2 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| MonWarsHS | F3 | C | 5 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| MonWarsHSB | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| MonWarsHTB | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| MonWarsNG | F3 | C | 3 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| MonWarsNGB | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| MonWarsNLB | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| MonWarsNM | F3 | C | 3 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| MonWarsNOB | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| MonWarsNP | F3 | C | 3 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| MonWarsNPB | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| MonWarsNR | F3 | C | 3 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| MonWarsNS | F3 | C | 3 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| MonWarsNSB | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| MonWarsNT | F3 | C | 3 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| MonWarsNTB | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Monster | F3 | C | 47 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Antarctic | F3? | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Balansa | F2? | C | 5 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Baram | F2? | C | 4 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| BlackLucy | F2? | C | 9 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Clovertest | F2? | C | 3 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| ConfusedFight | F2? | C | 3 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| DarkCastle | F2? | C | 9 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| DarkVampire | F2? | C | 8 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| DragonPalace | F3? | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| EscapeMaze | F3? | C | 3 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Fm_BigBattle | F2? | C | 4 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Fm_BigBattleH_PreparePlay | F2? | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Fm_BigBattle_PreparePlay | F2? | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| GoldButterfly | F2? | C | 3 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| GolemRun | F3? | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Hiphopclub | F2? | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| IamWriggle | F3? | C | 3 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| MagNest | F2? | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| MineExp | F3? | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Minibot | F3? | C | 4 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| NewXmas | F2? | C | 6 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| OkongOfLegend | F2? | C | 3 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| SavetheEarth | F3? | C | 3 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Smash | F2? | C | 4 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Space_TW | F2? | C | 2 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| UserContest | F2? | C | 17 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| WhiteAngel | F2? | C | 2 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| WinningStreak_Colosseum | F2? | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Wriggletest | F2? | C | 4 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| CAhero | F3 | C | 53 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| RPG | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| RPG01_B | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| RPG01_N | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| RPG01_S | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| RPG02_B | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| RPG02_N | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| RPG02_S | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| RPG03_B | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| RPG03_N | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| RPG03_S | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| RPG04_B | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| RPG04_N | F3 | C | 2 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| RPG04_S | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| RPG0501_B | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| RPG0501_N | F3 | C | 2 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| RPG0501_S | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| RPG05_B | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| RPG05_N | F3 | C | 2 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| RPG05_S | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| RPG0601_B | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| RPG0601_N | F3 | C | 2 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| RPG0601_S | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| RPG06_B | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| RPG06_N | F3 | C | 2 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| RPG06_S | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| RPG0701_B | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| RPG0701_N | F3 | C | 2 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| RPG0701_S | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| RPG07_B | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| RPG07_N | F3 | C | 2 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| RPG07_S | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| RPG0801_B | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| RPG0801_N | F3 | C | 3 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| RPG0801_S | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| RPG08_B | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| RPG08_N | F3 | C | 3 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| RPG08_S | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| RPG0901_B | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| RPG0901_N | F3 | C | 2 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| RPG0901_S | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| RPG09_B | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| RPG09_N | F3 | C | 3 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| RPG09_S | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| RPG1001_B | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| RPG1001_N | F3 | C | 2 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| RPG1001_S | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| RPG10_B | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| RPG10_N | F3 | C | 3 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| RPG10_S | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| RPG11_B | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| RPG11_N | F3 | C | 3 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| RPG11_S | F3 | C | 1 | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |

## Nhóm item

| Nội dung | Đợt | Lớp | Số lượng / mẫu | Server cần làm |
|---|---|---|---|---|
| Item/ItemWinningNiddle | F1? | C | — | Tên được map F1 dùng nhưng chưa ghép được ItemAttr. |
| 1 · item nền | F1 | A | 4 | Có pickup và stat trong phạm vi G1. |
| 1 · thao tác | F1 | B | 2 | Có wire và luật mẫu G1; cần đối chiếu biến thể và điều kiện. |
| 1 · còn lại | F1 | C | 51 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 2 | F1 | C | 28 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 3 | F2–F6? | C | 1 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 4 | F2–F6? | C | 2 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 11 | F1 | C | 7 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 12 | F2–F6? | C | 7 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 13 | F2–F6? | C | 2 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 21 | F1 | C | 6 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 22 | F1 | C | 1 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 23 · thao tác | F1 | B | 1 | Có wire và luật mẫu G1; cần đối chiếu biến thể và điều kiện. |
| 23 · còn lại | F2–F6? | C | 11 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 24 | F1 | C | 4 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 25 | F2–F6? | C | 2 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 26 | F1 | C | 7 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 27 | F1 | C | 2 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 28 | F1 | C | 2 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 29 | F1 | C | 10 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 30 | F1 | C | 7 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 31 | F2–F6? | C | 2 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 32 | F2–F6? | C | 2 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 33 | F1 | C | 1 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 34 | F2–F6? | C | 1 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 41 | F2–F6? | C | 2 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 42 | F2–F6? | C | 2 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 45 | F2–F6? | C | 1 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 46 | F2–F6? | C | 11 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 47 | F2–F6? | C | 7 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 48 | F2–F6? | C | 9 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 49 | F2–F6? | C | 5 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 50 | F2–F6? | C | 5 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 51 | F2–F6? | C | 2 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 52 | F2–F6? | C | 2 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 53 | F2–F6? | C | 2 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 54 | F2–F6? | C | 1 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 55 | F2–F6? | C | 1 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 56 | F2–F6? | C | 1 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 60 | F2–F6? | C | 6 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 79 | F2–F6? | C | 1 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 80 | F2–F6? | C | 26 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 89 | F2–F6? | C | 1 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 91 | F2–F6? | C | 2 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 93 | F2–F6? | C | 1 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 94 | F2–F6? | C | 2 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 96 | F2–F6? | C | 1 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 97 | F2–F6? | C | 6 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 98 | F2–F6? | C | 11 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 99 | F1 | C | 12 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 100 | F2–F6? | C | 3 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 101 | F2–F6? | C | 2 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 102 | F2–F6? | C | 1 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 103 | F2–F6? | C | 2 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 104 | F2–F6? | C | 1 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 105 | F2–F6? | C | 12 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 106 | F2–F6? | C | 1 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 107 | F2–F6? | C | 1 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 108 | F2–F6? | C | 1 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 109 | F2–F6? | C | 1 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 110 | F2–F6? | C | 1 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 111 | F2–F6? | C | 1 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 113 | F2–F6? | C | 1 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 114 | F2–F6? | C | 1 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 115 | F2–F6? | C | 1 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 116 | F2–F6? | C | 1 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 117 | F2–F6? | C | 1 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 118 | F2–F6? | C | 1 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 119 | F2–F6? | C | 6 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 120 | F2–F6? | C | 1 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 121 | F2–F6? | C | 3 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 122 | F2–F6? | C | 1 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 123 | F2–F6? | C | 2 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 124 | F2–F6? | C | 2 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 125 | F2–F6? | C | 1 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 126 | F2–F6? | C | 1 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 127 | F2–F6? | C | 1 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |
| 1000 | F2–F6? | C | 1 | Chưa biết mapping, luật hoặc thẩm quyền cho các mục còn lại. |

## Phương tiện

| Nội dung | Đợt | Lớp | Số lượng / mẫu | Server cần làm |
|---|---|---|---|---|
| Airship | F1 | C | 1 | Chưa có wire lên/xuống và quyền cập nhật tốc độ/va chạm của phương tiện. |
| Owl | F1 | C | 2 | Chưa có wire lên/xuống và quyền cập nhật tốc độ/va chạm của phương tiện. |
| Turtle0 | F1 | C | 3 | Chưa có wire lên/xuống và quyền cập nhật tốc độ/va chạm của phương tiện. |
| Turtle1 | F1 | C | 4 | Chưa có wire lên/xuống và quyền cập nhật tốc độ/va chạm của phương tiện. |
| AirshipBroken | F1 | C | 5 | Chưa có wire lên/xuống và quyền cập nhật tốc độ/va chạm của phương tiện. |
| Octopus0 | F1 | C | 6 | Chưa có wire lên/xuống và quyền cập nhật tốc độ/va chạm của phương tiện. |
| Octopus1 | F1 | C | 7 | Chưa có wire lên/xuống và quyền cập nhật tốc độ/va chạm của phương tiện. |
| RailCar | F1 | C | 8 | Chưa có wire lên/xuống và quyền cập nhật tốc độ/va chạm của phương tiện. |
| Chicken | F1 | C | 9 | Chưa có wire lên/xuống và quyền cập nhật tốc độ/va chạm của phương tiện. |
| Kart | F1 | C | 10 | Chưa có wire lên/xuống và quyền cập nhật tốc độ/va chạm của phương tiện. |
| Kart | F1 | C | 11 | Chưa có wire lên/xuống và quyền cập nhật tốc độ/va chạm của phương tiện. |
| Tank | F1 | C | 12 | Chưa có wire lên/xuống và quyền cập nhật tốc độ/va chạm của phương tiện. |
| Larva | F1 | C | 13 | Chưa có wire lên/xuống và quyền cập nhật tốc độ/va chạm của phương tiện. |
| MonS2Tank | F1 | C | 14 | Chưa có wire lên/xuống và quyền cập nhật tốc độ/va chạm của phương tiện. |
| MonS2UpgradeTank | F1 | C | 15 | Chưa có wire lên/xuống và quyền cập nhật tốc độ/va chạm của phương tiện. |
| Shark | F1 | C | 16 | Chưa có wire lên/xuống và quyền cập nhật tốc độ/va chạm của phương tiện. |
| Dragon | F1 | C | 17 | Chưa có wire lên/xuống và quyền cập nhật tốc độ/va chạm của phương tiện. |
| Rabbit | F1 | C | 18 | Chưa có wire lên/xuống và quyền cập nhật tốc độ/va chạm của phương tiện. |
| ItemBat | F1 | C | 19 | Chưa có wire lên/xuống và quyền cập nhật tốc độ/va chạm của phương tiện. |
| Rudolph | F1 | C | 20 | Chưa có wire lên/xuống và quyền cập nhật tốc độ/va chạm của phương tiện. |
| RetroFrog | F1 | C | 21 | Chưa có wire lên/xuống và quyền cập nhật tốc độ/va chạm của phương tiện. |
| RetroSnail | F1 | C | 22 | Chưa có wire lên/xuống và quyền cập nhật tốc độ/va chạm của phương tiện. |
| RetroPig | F1 | C | 23 | Chưa có wire lên/xuống và quyền cập nhật tốc độ/va chạm của phương tiện. |
| RetroDuck | F1 | C | 24 | Chưa có wire lên/xuống và quyền cập nhật tốc độ/va chạm của phương tiện. |
| RetroChicken | F1 | C | 25 | Chưa có wire lên/xuống và quyền cập nhật tốc độ/va chạm của phương tiện. |
| BalansaCap | F1 | C | 26 | Chưa có wire lên/xuống và quyền cập nhật tốc độ/va chạm của phương tiện. |
| RetroMouse | F1 | C | 27 | Chưa có wire lên/xuống và quyền cập nhật tốc độ/va chạm của phương tiện. |
| MagTurtle | F1 | C | 28 | Chưa có wire lên/xuống và quyền cập nhật tốc độ/va chạm của phương tiện. |

## Nhân vật

| Nội dung | Đợt | Lớp | Số lượng / mẫu | Server cần làm |
|---|---|---|---|---|
| C00 | F1 | A | 1 | Dùng profile nhân vật G1. |
| C01 | F1 | A | 2 | Dùng profile nhân vật G1. |
| C03 | F1 | A | 3 | Hai client Wine chọn nhân vật, nhặt tới trần, nhận kết quả và về phòng trên Forest24; ván kế đổi nhân vật và xác nhận reset chỉ số. |
| C05 | F1 | A | 4 | Hai client Wine chọn nhân vật, nhặt tới trần, nhận kết quả và về phòng trên Forest24; ván kế đổi nhân vật và xác nhận reset chỉ số. |
| C04 | F1 | B | 5 | Đối chiếu key Room/Battle, cap và quyền chọn; không suy key từ tên. |
| C06 | F1 | B | 6 | Đối chiếu key Room/Battle, cap và quyền chọn; không suy key từ tên. |
| C02 | F1 | B | 7 | Đối chiếu key Room/Battle, cap và quyền chọn; không suy key từ tên. |
| C07 | F1 | B | 8 | Đối chiếu key Room/Battle, cap và quyền chọn; không suy key từ tên. |
| C14 | F1 | B | 9 | Đối chiếu key Room/Battle, cap và quyền chọn; không suy key từ tên. |
| C15 | F1 | B | 10 | Đối chiếu key Room/Battle, cap và quyền chọn; không suy key từ tên. |
| C16 | F1 | B | 11 | Đối chiếu key Room/Battle, cap và quyền chọn; không suy key từ tên. |
| C23 | F1 | B | 12 | Đối chiếu key Room/Battle, cap và quyền chọn; không suy key từ tên. |
| C24 | F1 | B | 13 | Đối chiếu key Room/Battle, cap và quyền chọn; không suy key từ tên. |
| C10 | F1 | B | 14 | Đối chiếu key Room/Battle, cap và quyền chọn; không suy key từ tên. |
| C11 | F1 | B | 15 | Đối chiếu key Room/Battle, cap và quyền chọn; không suy key từ tên. |
| C12 | F1 | B | 16 | Đối chiếu key Room/Battle, cap và quyền chọn; không suy key từ tên. |
| C13 | F1 | B | 17 | Đối chiếu key Room/Battle, cap và quyền chọn; không suy key từ tên. |
| C17 | F1 | B | 18 | Đối chiếu key Room/Battle, cap và quyền chọn; không suy key từ tên. |
| C08 | F1 | B | 19 | Đối chiếu key Room/Battle, cap và quyền chọn; không suy key từ tên. |
| C09 | F1 | B | 20 | Đối chiếu key Room/Battle, cap và quyền chọn; không suy key từ tên. |
| C22 | F1 | B | 21 | Đối chiếu key Room/Battle, cap và quyền chọn; không suy key từ tên. |
| C26 | F1 | B | 22 | Đối chiếu key Room/Battle, cap và quyền chọn; không suy key từ tên. |
| C18 | F1 | B | 23 | Đối chiếu key Room/Battle, cap và quyền chọn; không suy key từ tên. |
| C19 | F1 | B | 24 | Đối chiếu key Room/Battle, cap và quyền chọn; không suy key từ tên. |
| C20 | F1 | B | 25 | Đối chiếu key Room/Battle, cap và quyền chọn; không suy key từ tên. |
| C21 | F1 | B | 26 | Đối chiếu key Room/Battle, cap và quyền chọn; không suy key từ tên. |
| C25 | F1 | B | 27 | Đối chiếu key Room/Battle, cap và quyền chọn; không suy key từ tên. |
| C27 | F1 | B | 28 | Đối chiếu key Room/Battle, cap và quyền chọn; không suy key từ tên. |
| C28 | F1 | B | 29 | Đối chiếu key Room/Battle, cap và quyền chọn; không suy key từ tên. |
| C10 | F1 | B | 30 | Đối chiếu key Room/Battle, cap và quyền chọn; không suy key từ tên. |
| C29 | F1 | B | 31 | Đối chiếu key Room/Battle, cap và quyền chọn; không suy key từ tên. |
| N01 | F1 | C | 32 | Chưa rõ key, quyền chọn hoặc modifier ngoài nhóm C. |
| N02 | F1 | C | 33 | Chưa rõ key, quyền chọn hoặc modifier ngoài nhóm C. |
| N03 | F1 | C | 34 | Chưa rõ key, quyền chọn hoặc modifier ngoài nhóm C. |
| N04 | F1 | C | 35 | Chưa rõ key, quyền chọn hoặc modifier ngoài nhóm C. |
| N05 | F1 | C | 36 | Chưa rõ key, quyền chọn hoặc modifier ngoài nhóm C. |
| N06 | F1 | C | 37 | Chưa rõ key, quyền chọn hoặc modifier ngoài nhóm C. |
| N07 | F1 | C | 38 | Chưa rõ key, quyền chọn hoặc modifier ngoài nhóm C. |
| N08 | F1 | C | 39 | Chưa rõ key, quyền chọn hoặc modifier ngoài nhóm C. |
| N09 | F1 | C | 40 | Chưa rõ key, quyền chọn hoặc modifier ngoài nhóm C. |
| N10 | F1 | C | 41 | Chưa rõ key, quyền chọn hoặc modifier ngoài nhóm C. |
| N11 | F1 | C | 42 | Chưa rõ key, quyền chọn hoặc modifier ngoài nhóm C. |
| N12 | F1 | C | 43 | Chưa rõ key, quyền chọn hoặc modifier ngoài nhóm C. |
| N13 | F1 | C | 44 | Chưa rõ key, quyền chọn hoặc modifier ngoài nhóm C. |
| N14 | F1 | C | 45 | Chưa rõ key, quyền chọn hoặc modifier ngoài nhóm C. |
| N15 | F1 | C | 46 | Chưa rõ key, quyền chọn hoặc modifier ngoài nhóm C. |
| N16 | F1 | C | 47 | Chưa rõ key, quyền chọn hoặc modifier ngoài nhóm C. |
| N17 | F1 | C | 48 | Chưa rõ key, quyền chọn hoặc modifier ngoài nhóm C. |
| N18 | F1 | C | 49 | Chưa rõ key, quyền chọn hoặc modifier ngoài nhóm C. |
| N19 | F1 | C | 50 | Chưa rõ key, quyền chọn hoặc modifier ngoài nhóm C. |
| C15 | F1 | B | 51 | Đối chiếu key Room/Battle, cap và quyền chọn; không suy key từ tên. |
| N20 | F1 | C | 52 | Chưa rõ key, quyền chọn hoặc modifier ngoài nhóm C. |
| N21 | F1 | C | 53 | Chưa rõ key, quyền chọn hoặc modifier ngoài nhóm C. |
| N22 | F1 | C | 54 | Chưa rõ key, quyền chọn hoặc modifier ngoài nhóm C. |
| N23 | F1 | C | 55 | Chưa rõ key, quyền chọn hoặc modifier ngoài nhóm C. |
| N24 | F1 | C | 56 | Chưa rõ key, quyền chọn hoặc modifier ngoài nhóm C. |
| C08 | F1 | B | 57 | Đối chiếu key Room/Battle, cap và quyền chọn; không suy key từ tên. |
| C26 | F1 | B | 58 | Đối chiếu key Room/Battle, cap và quyền chọn; không suy key từ tên. |
| N25 | F1 | C | 59 | Chưa rõ key, quyền chọn hoặc modifier ngoài nhóm C. |
| N26 | F1 | C | 60 | Chưa rõ key, quyền chọn hoặc modifier ngoài nhóm C. |
| N27 | F1 | C | 61 | Chưa rõ key, quyền chọn hoặc modifier ngoài nhóm C. |
| N28 | F1 | C | 62 | Chưa rõ key, quyền chọn hoặc modifier ngoài nhóm C. |
| Z01 | F1 | C | 63 | Chưa rõ key, quyền chọn hoặc modifier ngoài nhóm C. |
| B01 | F1 | C | 64 | Chưa rõ key, quyền chọn hoặc modifier ngoài nhóm C. |
| B02 | F1 | C | 65 | Chưa rõ key, quyền chọn hoặc modifier ngoài nhóm C. |
| W01 | F1 | C | 66 | Chưa rõ key, quyền chọn hoặc modifier ngoài nhóm C. |
| Z02 | F1 | C | 67 | Chưa rõ key, quyền chọn hoặc modifier ngoài nhóm C. |

## Mode

| Nội dung | Đợt | Lớp | Số lượng / mẫu | Server cần làm |
|---|---|---|---|---|
| 일반모드 (thường) | F1 | A | — | RoomRules hiện chạy lát trận thường G1. |
| 몬스터모드 (monster) | F3 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| 협공배틀 (đánh phối hợp) | F2 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| 부활대전 (hồi sinh) | F2 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| 고슴도치 (nhím) | F2 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| 대장잡기 (bắt thủ lĩnh) | F2 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| 시한폭탄 (bom hẹn giờ) | F2 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| 팝케이드 (Popcade) | F3 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| 혼자놀기 (chơi một mình) | F3 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| 눈싸움 (đấu ném tuyết; sự kiện cũ) | F2 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| 슈퍼파이터 (Super Fighter; sự kiện cũ) | F2 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| 래더 (Ladder) | F2 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| 크아 히어로 (CA Hero) | F3 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| 히든 캐치 (Hidden Catch) | F3 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| AIBattle | F3 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Baseball | F2? | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Airplane | F2? | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| BigBattle | F2? | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| ConfusedFight | F2 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| RPG | F3 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| SandBox | F3? | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Antarctic | F2? | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| IamWriggle | F3? | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| IamBoss | F3? | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| SnowyFields | F2? | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| SnowyChamp | F2? | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Ghép trận UXMatching | F2 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Free 1v7 | F2 | C | — | Lượt M4 âm; không retry ở bước 0, chưa biết khác biệt luật gốc. |
| Hướng dẫn | F3 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Màn thưởng | F3 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Monster mùa 2 | F3 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Nhiệm vụ chơi với máy | F3 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |

## Hệ thống

| Nội dung | Đợt | Lớp | Số lượng / mẫu | Server cần làm |
|---|---|---|---|---|
| Cửa hàng | F4 | C | — | Chưa xác định hợp đồng túi đồ và giao dịch; màn nhúng dựng bằng web server. |
| Bang hội | F5 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Bạn bè / Messenger | F5 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Nhiệm vụ | F6 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Ladder | F2 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Xếp hạng CAJJang | F5 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Sự kiện theo tiền tố Event | F6 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Chat | F5 | B | — | Chat sảnh/phòng đã đo cho L1; whisper, bạn bè và giới hạn lạm dụng chưa đo. |
| Exchange | F6 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Level | F4 | B | — | Dùng hồ sơ M6 và bảng kinh nghiệm tham chiếu; cần chính sách thưởng và kiểm trường hiển thị. |
| PCRoomBenefit | ngoài phạm vi | — | — | Ngoài phạm vi theo quyết định bỏ ưu đãi Nexon. |
| Sự kiện TakeCareOfEvent | F6 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| CAPass | F6 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| MagicMarket | F6 | C | — | Pack có UI; boomkr-re không tìm thấy công thức trong phạm vi khảo sát. |
| Bộ sưu tập item | F4 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Hồ sơ / MyInfo | F4 | B | — | Có projection tài khoản M5/M6; chưa đầy đủ profile và dữ liệu túi đồ. |
| Chọn nhân vật | F4 | B | — | Có chọn và lưu cho nhân vật đã cho phép; quyền sở hữu/chỉ số mở rộng còn thiếu. |
| Trang trí avatar | F4 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Trang trí nhân vật | F4 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Trang trí bom | F4 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Trang trí cấp | F4 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Cảm xúc / Emoticon | F4 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Trang trí ID | F4 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Nhân vật | F1 | B | — | Có Room/Battle selection và chỉ số nền; đối chiếu key và cap cho từng nhóm. |
| Item | F1 | C | — | Tách theo nhóm ItemAttr; có lát A/B cho item nền đã đo, phần còn lại chưa biết. |
| Map | F1 | C | — | Tách theo họ; profile thường dùng adapter G1, script và nhóm đặc biệt chưa biết. |
| Ô map | F1 | C | — | Đã biết một số movement/ray flags; cần biết ý nghĩa các ô chưa đo. |
| Monster | F3 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Hidden Catch | F3 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| RPG | F3 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| CAHero | F3 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| SandBox | F3 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| ConfusedFight | F2 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| AIBattle | F3 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Antarctic | F2 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| HoneyButterChip | F6 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| DIYBomb | F6 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| MineExplore | F6 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| IamWriggle | F3 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| IamBoss | F3 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| SnowyChamp | F2 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| LuckyCard | F6 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Ghép trận UXMatching | F2 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Sảnh | giai đoạn 1 | A | — | Đã có luồng vào sảnh/phòng qua TLS; Square và các màn bên chưa phân loại. |
| Đăng nhập | giai đoạn 1 | A | — | Đã có companion TLS và admission tài khoản nội bộ, không gồm NGM. |
| Hướng dẫn | F3 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Kết quả | giai đoạn 1 | A | — | Kết quả/journal trận thường đã có, thưởng hiện là 0. |
| Trận đấu | F1 | A | — | Lõi RoomRules trận thường đã chạy trong phạm vi G1. |
| Pet | F6 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| CustomMap | F7 | C | — | Đợt F7 theo quyết định 27/11; hình thức chốt sau. Chưa biết upload, validation và quyền điều khiển. |
| Câu cá | F6 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Couple | F5 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| GrowSprout | F6 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| DTT | F6 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Partners | F6 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| LineFriends | F6 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| MonWars | F6 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| RetroWriggle | F6 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| SelectionBox | F6 | C | — | Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967. |
| Túi đồ | F4 | C | — | Chưa có ledger, projection túi đồ và wire đồng bộ quyền sở hữu. |
| Web nhúng: Dwarf/libcef | F4/F6 | C | — | Đã chốt dựng web ở server; chưa biết hợp đồng từng màn. |
| Quảng trường (광장) | F5 | C | — | Sảnh đi lại tự do và các khu quần áo, trang sức, pet, cà phê; chưa có wire di chuyển và đồng bộ. |
| Kênh theo loại (채널) | F2 | C | — | Kênh tân thủ, sự kiện, không item, luyện tập và giải đấu; chưa biết luật vào kênh. |
| Giải đấu (토너먼트) | F2 | C | — | Giải đấu có kênh luyện tập, giới hạn người vừa vô địch và quà; chưa có wire và luật. |
| Siêu chủ phòng (슈퍼방장) | F2 | C | — | Quyền chủ phòng mở rộng như bắt đầu ngay; chưa biết nguồn cấp và giới hạn. |
| Thư (쪽지) | F5 | C | — | Gửi và nhận thư, hộp thư có giới hạn; chưa có wire và lưu trữ. |
| Danh hiệu (칭호) | F4 | C | — | Đạt và gắn danh hiệu, có thông báo khi đạt; chưa biết điều kiện từng danh hiệu. |
| Cường hóa và nâng cấp (강화) | F4 | C | — | Nâng cấp item, pet, bóng nước và đồ RPG bằng nguyên liệu; chưa có công thức và tỉ lệ. |
| Tặng quà và phiếu (선물하기, 쿠폰) | F4 | C | — | Tặng item cho người khác và dùng phiếu đổi; bỏ phần tiền thật của Nexon. |
| Mật khẩu cấp 2 (2차 비밀번호) | F4 | C | — | Lớp bảo vệ tài khoản trong game; chưa có wire tạo và xác thực. |
| Bốc thăm và đậu may mắn (뽀끼뽀끼, 럭키빈) | F6 | C | — | Bốc thăm bằng chìa khóa và đổi đậu may mắn lấy quà; tỉ lệ do server cấu hình. |
| Chợ sách phép (마법거래소) | F6 | C | — | Người chơi đăng bán và mua sách phép; chưa có wire và luật giá. |
| Mini-game sự kiện (빙고, 주사위, 퀴즈) | F6 | C | — | Bingo nhân vật, nhiệm vụ xúc xắc và câu đố; chưa có wire và phần thưởng. |
