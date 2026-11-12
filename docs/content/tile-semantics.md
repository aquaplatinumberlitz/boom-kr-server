# Ngữ nghĩa tile từ dữ liệu tĩnh KR 00967

Nguồn `kr_static`: `fx/ObjectTable/ObjectTable`, SHA-256 `a68fd5ffcc815b24b5c5199e9910f10d14f1ac6f30aba4c237a23f4e279608dc`, và `fx/TileInfo/TileInfoList`, SHA-256 `51b738448f5bb0c9dc54c0ef6782bd612561660f256717dc5df1d59c2f5ec795`. Map ExVillage01 có SHA-256 `aeb57ffdb27a1405bd2ae3d29f229174ae5d584fc84eaf4e87884fe8518f86a4`. Tham khảo cấu trúc trong `boomkr-re/docs/kr-data.md` mục 1 và `boom-re/docs/map.md` mục 1; tài liệu VN là nguồn tham khảo, không chứng minh loader KR.

## Ánh xạ và quy tắc ứng viên

Mỗi mã không âm ở trường `tile`, `object` và `overlay` được tra theo vị trí bắt đầu từ 0 trong danh sách tên ObjectTable, kể cả mục thư mục; tên đó nối chính xác với thuộc tính `Name` trong TileInfoList. Ví dụ object 1932, 1933, 1934 lần lượt là `Tile00_Box/BoxGray`, `BoxOrange`, `BoxRed`; 1955–1957 là ba nhà `Tile00_Static/HouseBlue`, `HouseRed`, `HouseYellow`. Tra trực tiếp cùng chỉ số trong thứ tự XML cho tên khác, không phù hợp seed. Đây là ánh xạ `kr_static`, trạng thái **candidate**: sự phù hợp toàn bộ seed không chứng minh đường lookup thực thi của client.

Quy tắc dùng các dword little-endian đầu trong `Data` 72 byte, tính offset từ 0. Nền chỉ nhận mẫu dword tại offset 0–40 là `(0, 0, 0, 0, 0, 0, 1, 1, 0, 0, -1)`. Tên nền chứa `pin` bị loại theo tên vì có thể chỉ địa hình đặc biệt; đây là bộ lọc bảo thủ dựa trên tên, không phải trường đã giải. Trường `object` bằng -1 hoặc có cùng mẫu nền cho loại 0. Object có mẫu 11 dword `(1, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0)` cho loại 1; mẫu `(2, 0, 0, 0, 0, 0, 0, 0 hoặc 1, 0, 0, 0)` cho loại 2. Overlay khác -1, nền thiếu hoặc mẫu khác đều chưa rõ. Các dword và tên `Box`/`Static` tương quan với loại, nhưng chưa giải ý nghĩa của từng trường hay chứng minh vật lý từ chúng.

Seed CP094 là policy `design` pin theo quan sát `kr_runtime` giới hạn trên ExVillage01: loại 0 là trống, 1 là chặn cố định, 2 là thùng phá được. Cả chế độ cũ và quy tắc mới khớp **195/195**: 46 trống, 23 chặn, 126 thùng. Đây là kiểm tra duy nhất có oracle cho quy tắc; các map khác chỉ là ngoại suy `candidate`, không phải hành vi đã đo hoặc luật server gốc. Không suy mọi biến thể cùng mẫu đều có hành vi giống nhau.

## Sáu map ladder

Mỗi số là số ô; `chưa rõ` không được gán terrain. Mỗi map có 195 ô.

| Map | Trống ứng viên | Chặn ứng viên | Thùng ứng viên | Chưa rõ |
|---|---:|---:|---:|---:|
| Village10 | 40 | 46 | 91 | 18 |
| Camp08 | 61 | 2 | 82 | 50 |
| Factory07 | 33 | 53 | 109 | 0 |
| Forest07 | 30 | 56 | 108 | 1 |
| Tomb12 | 26 | 6 | 59 | 104 |
| Patrit14 | 52 | 4 | 138 | 1 |

Các tên và số ô chưa đo hành vi trong sáu map: `Tile00_Passable/Herb00` 18; `Tile16_Passble/Herb01` 10, `Herb02` 12, `Line01` 6, `Line02` 7; `Tile16_Box/StrongBox01` 4, `StrongBox02` 4; `Tile16_Static/MissileBase02`, `PinPillar03`, `PinPillar04` mỗi tên 1; `Tile16_Ground/PinGround01` 4; `Tile05_Static/Well00` 1; `Tile04_Static/MastTileA` 1. Trong Tomb12, `Tile09_Ground/WaterwayH04` 3, `WaterwayH05` 4, `WaterwayH06` 31, `WaterwayH07` 8, `WaterwayH08` 2, `bridge00` 4 và 52 ô có cả hai mã -1 đều chưa rõ. Các tên rút gọn sau dấu phẩy dùng cùng thư mục ngay trước đó. Không thấy loại đẩy, băng chuyền hay cổng dịch chuyển trong các ô chưa rõ này; chưa suy hành vi của chúng ở map khác.
