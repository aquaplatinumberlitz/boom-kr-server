# Kiểm kê lưới và tile chưa ánh xạ của F2/F3

[JSON kiểm kê](../evidence/content/f2-f3-tile-grid-census.json) ghi kết quả `kr_static` của build `KR00967` cho 699 map riêng biệt; hai catalog có 122 map chung.
Nguồn phạm vi là [F2](f2-mode-catalog.md) và [F3](f3-pve-catalog.md). Mỗi map có bản ghi riêng; mỗi họ giữ hợp các khóa còn thiếu từ mọi map của họ.

| Catalog | Map | Họ | Giải mã đủ | Đọc một phần | Lỗi giải mã | Lưới khác 15x13 | Loại tile còn thiếu |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| F2 | 122 | 18 | 115 | 7 | 0 | 0 | 40 |
| F3 | 699 | 180 | 692 | 7 | 0 | 116 | 49 |

## Lưới và đuôi map

| Lưới | F2 | F3 |
| --- | ---: | ---: |
| `15x13` | 122 | 583 |
| `15x51` | 0 | 2 |
| `20x12` | 0 | 12 |
| `20x13` | 0 | 1 |
| `20x15` | 0 | 3 |
| `20x16` | 0 | 1 |
| `20x18` | 0 | 2 |
| `20x19` | 0 | 2 |
| `20x20` | 0 | 3 |
| `20x22` | 0 | 2 |
| `20x25` | 0 | 4 |
| `21x12` | 0 | 2 |
| `21x16` | 0 | 1 |
| `21x17` | 0 | 2 |
| `21x19` | 0 | 1 |
| `21x20` | 0 | 2 |
| `21x21` | 0 | 1 |
| `22x22` | 0 | 1 |
| `23x17` | 0 | 1 |
| `23x23` | 0 | 1 |
| `24x24` | 0 | 8 |
| `25x12` | 0 | 5 |
| `25x13` | 0 | 1 |
| `25x15` | 0 | 3 |
| `25x17` | 0 | 2 |
| `25x19` | 0 | 1 |
| `25x20` | 0 | 7 |
| `25x22` | 0 | 1 |
| `25x25` | 0 | 12 |
| `29x29` | 0 | 3 |
| `33x29` | 0 | 1 |
| `37x29` | 0 | 2 |
| `40x10` | 0 | 12 |
| `40x12` | 0 | 1 |
| `67x10` | 0 | 1 |
| `100x10` | 0 | 12 |

Kiểu đuôi chưa được `decode_map` đọc đủ: `group_record_tail` (7 map).
Với đuôi khác bố cục thường, công cụ giữ lưới và chỉ giữ các hàng vật phẩm nếu đọc trọn phần đó. Bộ đọc đuôi nhóm/bản ghi chỉ xác nhận cấu trúc; chưa xác nhận tác dụng các bản ghi.
Cả bảy map `Bula` khớp từng byte với [bằng chứng P228](../protocol/f2-bula-tail-and-mode-wire-re.md). Có thể xem JSON bằng `python3 tools/kr_pack_export.py --game-dir <thu-muc-pack> --inspect-bula Bula00`. Census vẫn ghi `partial` theo bộ đọc thường; các adapter phòng chưa nhận `Bula`.

## Họ ưu tiên theo số tile còn thiếu

Chỉ xếp `quick_unlock` khi mọi map trong họ đều giải mã đủ, có lưới 15x13, có từ một đến ba khóa tile khác nhau và không có lỗi ràng buộc địa hình đã thấy. Các họ được xếp theo số tile rồi theo tên trong mỗi nhóm ưu tiên.

| Catalog | Địa hình đủ | Ưu tiên | Trên ba tile | Ràng buộc địa hình | Thiếu bố cục | Lưới khác |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| F2 | 15 | 0 | 2 | 0 | 1 | 0 |
| F3 | 106 | 2 | 6 | 0 | 1 | 65 |

| Catalog | Họ ưu tiên | Map | Tile còn thiếu | Khóa đặt sẵn / drop dương còn thiếu |
| --- | --- | ---: | --- | ---: |
| F3 | `Bonus` | 12 | `TileBonusMap_static/Tower00` | 14 / 0 |
| F3 | `Hdn_SuperFight` | 1 | `Tile23_Static/Temple08` | 0 / 11 |

Mỗi mã ô được nối từ `ObjectTable/ObjectTable` sang `TileInfo/TileInfoList` trong pack `fx`, rồi kiểm riêng theo lớp nền, vật thể hoặc overlay bằng `f1_terrain_rules(f1e=True)`. Các tile Baseball và Kart đã xác nhận tĩnh còn phải khớp tên, lớp và đủ 18 số trong [bằng chứng KR 00967](../protocol/f2-baseball-kart-tiles-re.md). Công cụ cũng thử các lớp đã ánh xạ cùng nhau để phát hiện ràng buộc như footprint chồng nhau.
Khóa đặt sẵn và drop có trọng số dương được đối chiếu riêng với danh sách khóa F1. Khóa vật phẩm không tính vào số tile; một khóa đặt sẵn hoặc drop trùng tên trong TileInfo được ghi thêm theo lớp tương ứng.

Đối chiếu KR 00967 xét đủ 18 số của 49 khóa còn thiếu trong hai catalog. Có 0 khóa trùng tuple với khóa đã ánh xạ ở cùng lớp trên lưới 15x13. JSON giữ các tuple theo khóa và lớp cùng SHA-256 của tập này và của `TileInfo/TileInfoList`.
Bảy tile nền Baseball có cờ lap. Nền đường đua, ô tăng tốc và lốp của Kart có cờ hoặc marker riêng. Hai họ vẫn chưa đủ địa hình; census không mở script Baseball hoặc luật đua Kart. `Temple08` khác `Temple11` ở cờ +40, còn `Tower00` có marker +36 bằng 999. Các tile SnowBear và Tornado vẫn chờ xác nhận hành vi riêng.

Xếp hạng là `candidate` cho công việc về địa hình. Điều kiện phục vụ như vị trí, xác suất vật phẩm, allowlist script, chế độ phòng và quyền PvE chưa được xác nhận ở đây. Ví dụ, `MonS2Fire03` và `RPG11_N_02` đọc được với lưới lớn; `Tutorial01` đọc được với lưới 15x13. Những kết quả đó mô tả dữ liệu pack; các giới hạn phục vụ vẫn theo [probe Wine r1](f2-f3-wine-probe-r1.md).

Sinh lại từ pack gốc: `python3 tools/kr_f2_f3_tile_census.py --generate` (có thể chỉ định `--game-dir`). Kiểm hash thân JSON, phạm vi hai catalog và định dạng cả hai đầu ra: `python3 tools/kr_f2_f3_tile_census.py --check`. Lệnh kiểm chỉ đọc tệp trong repo.
