# F3 PvE: RE 65 họ lưới lớn của KR 00967

Tài liệu này chốt khảo sát tĩnh trên IDB KR 00967 và census pack F3. IDB được sao chép vào scratch trước khi đọc, không sửa bản nguồn. SHA-256 IDB là `6be15a1fccea0837ee95c50893345e9f13dc217b0fa570975b5c284f0d547fd1`. Bằng chứng máy và pack nằm ở [JSON](../evidence/content/f3-pve-large-grid-re.json).

## Phạm vi census

Census `kre.f2-f3-tile-grid-census.v1` ghi nhận 699 map thuộc 180 họ F3; 692 map giải mã chính xác và 7 map còn từng phần. Có 583 map `15x13`, còn 116 map thuộc 65 họ có lưới lớn. Các kích thước phân biệt và số ô là:

| Lưới | Ô | Số map |
|---|---:|---:|
| `15x13` | 195 | 583 |
| `15x51` | 765 | 2 |
| `20x12` | 240 | 12 |
| `20x13` | 260 | 1 |
| `20x15` | 300 | 3 |
| `20x16` | 320 | 1 |
| `20x18` | 360 | 2 |
| `20x19` | 380 | 2 |
| `20x20` | 400 | 3 |
| `20x22` | 440 | 2 |
| `20x25` | 500 | 4 |
| `21x12` | 252 | 2 |
| `21x16` | 336 | 1 |
| `21x17` | 357 | 2 |
| `21x19` | 399 | 1 |
| `21x20` | 420 | 2 |
| `21x21` | 441 | 1 |
| `22x22` | 484 | 1 |
| `23x17` | 391 | 1 |
| `23x23` | 529 | 1 |
| `24x24` | 576 | 8 |
| `25x12` | 300 | 5 |
| `25x13` | 325 | 1 |
| `25x15` | 375 | 3 |
| `25x17` | 425 | 2 |
| `25x19` | 475 | 1 |
| `25x20` | 500 | 7 |
| `25x22` | 550 | 1 |
| `25x25` | 625 | 12 |
| `29x29` | 841 | 3 |
| `33x29` | 957 | 1 |
| `37x29` | 1073 | 2 |
| `40x10` | 400 | 12 |
| `40x12` | 480 | 1 |
| `67x10` | 670 | 1 |
| `100x10` | 1000 | 12 |

Danh sách 65 họ và phân bổ kích thước đầy đủ được pin trong JSON để tránh lặp lại bảng dài ở đây. `15x13` không phải một biến thể của lưới lớn; `19x17` `BigBattle` hiện có trong Rust cũng không xuất hiện trong 65 họ này.

## Kết quả IDA

`CPlayMap` có vtable tại `0x176BFF0`, `CGamePlayMap` tại `0x1769720`. Decompiler cho thấy client dùng bước ô 40 pixel và chuyển vị trí pixel sang chỉ số ô bằng phép chia `/40`. Đường cập nhật/render của `CGamePlayMap` lấy bốn biên từ các trường `this[1924..1927]`, lặp theo nửa khoảng `[row_min,row_max)` và `[col_min,col_max)`, rồi gọi các đường vẽ tile, object, bomber và bomb. Vì vậy biên phải phụ thuộc `width` và `height` của map, không thể cố định 15×13.

Các trường `this[83]` và `this[84]` được cộng vào tọa độ render; đây là bằng chứng tĩnh cho offset camera/scroll ở client. Vtable slot 5 (`0xE9F830`) kiểm tra ô ground/GBB có thể đi được. Các đường cập nhật tiếp tục loại object ngoài bốn biên và gọi logic theo ô. Tên và vtable của `CMapActionDropItem`, `CMapActionBomb`, `CMapActionDestroyBox`, `CMapActionTileControl` và anchor `kCPlayMap_CalcBombDirectionExplosion` đều hiện diện. Điều này xác nhận các điểm móc client cho drop, phá ô, ray bomb và tile script; thân wire, quyền server và luật riêng từng map vẫn là candidate cho đến khi có capture.

Census cho thấy các ô chưa ánh xạ lặp lại nhiều nhất là hai tile tornado của `MonS2Desert` và `TileBonusMap_static/Tower00`. Placement chưa ánh xạ lặp lại gồm `MonS2Tank`, `MonS2UpgradeTank`, `Item/ItemSuperMan`, `Item/ItemHPNeedle`, nhóm `Item/Treasure*`, `BonusMapReward*`, `SkillThrowNuclearBomb` và một số `Lucci*`. Drop dương chưa ánh xạ được ghi nhận ở `Item/ItemDart`, `Item/ItemNiddle`, `Item/ItemRescueCoin`, `Item/ItemSensor`, `Item/ItemShield`, `Item/ItemWildGinseng`, `RemoteBomb` và `Spring`. Đây là kết quả pack census, không phải khẳng định mọi map đều chứa từng loại.

## Điều chưa được chứng minh

`u8` hay `u16` cho tọa độ wire chưa thể kết luận từ pass tĩnh này: client có arithmetic theo ô và object field, nhưng chưa thu được codec đầy đủ cho tất cả route large-grid. Số spawn chính xác của 116 map lớn cũng chưa có trong census; một control `CAhero01` xuất được 8 spawn nhưng là map `15x13`, không được dùng làm oracle cho lưới lớn. Cần một capture hai client và đọc loader/codec tương ứng để chốt hai trường này.

## Khoảng cách tới `terrain.rs`

`rust/crates/kr-service/src/terrain.rs` hiện chỉ chấp nhận `STANDARD_SIZE = (15,13)` và `BIG_BATTLE_SIZE = (19,17)`. Để mở thực thi F3 large-grid cần mở rộng cấu hình theo đúng từng cặp kích thước đã pin, cấp phát `width*height`, giữ lớp tile/object/overlay theo map, kiểm tra spawn và placement trong biên, rồi làm ray bomb, camera clipping và drop theo kích thước động. Không được ánh xạ ngầm lưới lớn vào một trong hai hằng số hiện tại. Các luật script và quyền MonAction vẫn phải đi qua contract đã đo; không suy quyền server từ vtable client.

Nguồn và trạng thái: kích thước, số map, payload hash và các địa chỉ IDA là `kr_static/confirmed`; độ rộng tọa độ wire, spawn large-grid, relay/quyền server và ngữ nghĩa đầy đủ của hook là `candidate` hoặc `unknown`. Đây là checkpoint tĩnh, chưa đóng gate runtime.
