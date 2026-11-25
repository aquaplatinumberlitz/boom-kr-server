# F4-0: phân cụm kinh tế và đối chiếu wire KR 00967

Batch `p116` đối chiếu dump IDA của `CA.exe` KR 00967 với [catalog kinh tế F4-0](f4-economy-catalog.md),
[kiểm kê pack](../CONTENT.md#bảng-dữ-liệu-ngoài-attr) và [bảng phân loại](classification.md).
[JSON bằng chứng](../evidence/content/f4-ida-clusters.json) giữ hash, VA, chuỗi, entry cạnh vtable,
call site và khóa của từng đoạn decompile. Nguồn là client gốc `a52545c1…` và ảnh unpack đối chiếu
`cf72da11…` (image base `0x400000`; RVA = VA trừ image base).

`kr_static/confirmed` chỉ xác nhận case, lời gọi hoặc phần đọc thấy được tại VA đã ghi.
`candidate` là liên hệ cần đo tiếp; `unknown` là opcode hoặc trường chưa nối được. Dump có
413 opcode C→S phân biệt tại `sub_1164C10`, 225 case S→C của `CStage` (`0xD68C90`) và
20 case của `CShopStageNew` (`0x1072B10`). Đây là kích thước chỉ mục dùng để tra, không phải số gói
thuộc F4. Địa chỉ trong các bảng là VA; `u8/u16/u32` chỉ primitive sau giải mã theo
[codec KR](../protocol/f1-aux-opcodes-and-classc.md#f1-aux-codec). Đợt này chỉ đọc dump; không có lượt Wine
mới. Opcode, layout và stage của VN không được đưa vào baseline.

## F4-a: hồ sơ, EXP, cấp, Lucci và kết quả trận

| Điểm tựa KR | Phần đã xác nhận và giới hạn |
|---|---|
| `CStage` S→C `135`: case `0xD690CB` → `0xD6CED0` | Đọc `{ u8 local_idx, u32 remain_lucci }` qua slot reader `+0x3C/+0x44`, gọi `0x13E4950`. Chuỗi là `SelfUser[%d] set remain lucci=%d`. Lịch gửi và miền số dư hiển thị còn **unknown**. |
| `CStage` S→C `20`: case `0xD68DAC` → `0xD6E890` | Prefix gồm hai `u8` và bốn `u32`. Byte đầu là chỉ số local dùng chung; dword thứ hai đi vào cùng setter Lucci của `135`. Byte thứ hai đi vào `0x13E2380`; ba dword còn lại đi vào `0x13E2440`, `0x13E4BC0`, `0x13E4A30`. Không đặt tên EXP/cấp/Cash cho chúng. Guard `0xD95C30` mở nhánh đọc tiếp; dump dừng giữa nhánh, payload đầy đủ **unknown**. |
| `CStage` S→C `137` → `0xD6D1B0`, `217` → `0xD72F80` | `137` đọc `u8 count` rồi từng cặp `u8 local_idx, u8 value` qua setter chung với `20`. `217` đọc hai `u32` rồi gọi `0x13E3EB0`. Nghĩa các giá trị còn **unknown**. |
| `CStage` S→C `291` → `0xD73BB0` → `0xD2FEB0` | Tái dùng [CP183](../protocol/lobby-profile.md#own-profile-291): hồ sơ qua active/blob và các trường theo cờ. Có consumer EXP cho một trường ngoài blob; các số liệu hiển thị trong mẫu đã đo bằng 0. |
| `CGameStage` S→C `175`: `0xE5D686` → `0xE5F610` | Hai đoạn decompile dừng trước thân hàm. [CP023](../protocol/life-results.md#round-result) đã nối buffer kết quả: variant có header 16 byte với `+9 = 0xFF`, sau đó 16 slot; slot trống dài 1 byte, hàng active 131 byte, rank ở byte `+5` của hàng. Offset, độ rộng, delta hay total của EXP/Lucci và trường cấp trong kết quả vẫn **unknown**. |

Schema 7 có `PlayerProfile`, ledger và `EconomyPolicy` cấu hình thưởng thắng/thua/hòa, ngưỡng cấp
và điều kiện người rời trận. `finish_match_settled` ghi kết quả và thưởng trong cùng giao dịch; phát lại
cùng kết quả không thưởng hai lần. Worker hiện chỉ trả match record từ đường này.
[Serializer `175`](../../rust/crates/kr-wire/src/kr00967/battle.rs) chỉ điền rank vào các hàng đã biết;
[khung hồ sơ `291`](../../rust/crates/kr-wire/src/kr00967/account.rs) vẫn dùng số liệu 0.
Bảng ngưỡng wiki trong preset và số thưởng của server là cấu hình, chưa có bằng chứng đó là luật KR gốc.

[CP398](../protocol/life-results.md#f4a-profile-result-projection) đọc tiếp thân `20` và nối hai trường
đầu của hàng `175` với cùng setter hồ sơ. [CP399](../protocol/lobby-profile.md#f4a-profile-395) xác định
case `395` và tách scalar dùng trong panel RPG khỏi vùng lưu của `20/175`. Nghĩa EXP/cấp thường
và lịch phát vẫn cần đo.

Bước kiểm: lần từ setter của `20/135` và parser hàng `175` tới nơi hiển thị. Trên Wine, giữ một ván thưởng 0
làm đối chứng rồi đo một ván có một khoản thưởng khác 0 đã biết, đủ qua ngưỡng cấp. So cursor và render của
kết quả, sảnh/MyInfo sau khi quay lại và sau đăng nhập lại với ledger/profile. Kiểm giá trị vừa giới hạn
`u32`, chính sách khi số dư DB vượt giới hạn wire và tách tổng khỏi delta trước khi nối projection.
Chưa gán request C→S cho refresh ví/MyInfo hoặc chọn lịch phát `20/135/291`.

## F4-b: túi đồ, trang bị, tiêu hao và coupon

RTTI có `CItemInventoryNew` tại vtable `0x178E624`. Các case dưới đây cho điểm bám vào code túi đồ;
tên thao tác chính xác vẫn là **candidate** khi chỉ có call chain hoặc thông báo UI.

| Cụm | Dấu tĩnh và ranh giới |
|---|---|
| Chuyển dữ liệu đến manager | `CStage` S→C `219` → `0xD76860` → `0x11CEA10`; `224` → `0xD6D370` → `0x11CE4C0`; `225` → `0xD6D410` → `0x11CE6A0`. Chưa tách được snapshot, delta hoặc equip. |
| Các trường đầu của cập nhật | `CStage` `313/314/315/316` → `0xD779C0/0xD77BD0/0xD77D60/0xD77EF0`. JSON giữ prefix và các nhánh đọc có guard. `315` đọc thêm một giá trị 64 bit sau hai byte và ba dword; callee `0x11AECE0` có site tạo C→S `566` ở `0x11AFA5D`. Chưa biết nghĩa các trường, khi nào site chạy hoặc có phải ACK. |
| Thông báo túi, thời hạn và khóa | `CStage` `228` → `0xD85F80` có `CantUseLucciBox`; `317` → `0xD84410` có `PeriodChange_fail` và `PeriodChange_Success`. `245/377/378/418/419` có chuỗi về bán lại, khóa, hoặc hết chỗ. Các chuỗi không định danh request và không cho layout đầy đủ. |
| Coupon và thông báo ở shop | `CShopStageNew` S→C `236/237` → `0x1074130/0x1075900` có `UseItemCoupon` và `Each`; `236` có thêm `ValidFor`. `345` → `0x107E660` có `CrazyPark_ReceiveCouponGift`; `446/447` có chuỗi hết hạn và đã bán. Request nhập coupon, kiểu mã và quyền đổi vẫn **unknown**. |

[Schema 7](../../rust/crates/kr-service/migrations/0007_f4_progression_inventory_shop.sql) lưu SKU sở hữu,
số lượng, thời hạn và trang bị theo slot. `AccountService` đọc được túi và slot, hỗ trợ equip/unequip;
equip kiểm SKU có sở hữu, còn hạn và khớp slot của catalog. Grant vĩnh viễn, theo chồng hoặc có thời hạn đã
có ở lớp dữ liệu; API này chưa có tiêu hao từ túi tài khoản hay đổi coupon native. Đường tiêu hao vật phẩm
trong trận cần được đối chiếu riêng với số lượng đã lưu.

Bước kiểm: bám reader của `219/224/225` và lookup chung của `313–316`, rồi dùng một tài khoản túi trống và
một tài khoản có một SKU đã biết. Trên Wine, mở túi, trang bị/tháo một món, dùng một vật phẩm tiêu hao
và đăng nhập lại; giữ một món hết hạn hoặc không sở hữu làm đối chứng. Đo từng frame và kiểm cả UI lẫn
DB trước/sau. Đo luồng coupon riêng trên stub cục bộ, gồm một mã chấp nhận, một mã từ chối và lần gửi lặp,
sau khi đã biết layout và xác định nơi kiểm tra mã.

## F4-c: trang trí, emoticon và bộ sưu tập

| Cụm | Dấu tĩnh KR | Khoảng trống |
|---|---|---|
| Avatar / nhân vật / cấp | `CharTE/AvatarDeco`, `CharTE/CharDeco`, `CharTE/GradeDeco`; RTTI `CCharDecoMan`, vtable `0x174590C`. | Cạnh từ asset avatar tới manager, enum slot và trường trên snapshot còn **unknown**. |
| Bom | `CharTE/BombDeco`; RTTI `CBombDecoMan`, vtable `0x174425C`. | Biên các entry cuối và đường áp trang trí bom ở peer còn **unknown**. |
| ID / emoticon | `EmoTE/IDDeco`, `EmoTE/Character`, `fx/Emoticon`; RTTI `CIDDecoMan`, vtable `0x17958D4`. | Chưa có opcode gửi/chuyển tiếp emoticon hay ánh xạ token của ID deco lên UI. |
| Bộ sưu tập | `FxContents/ItemCollection`; RTTI `CCAItemCollectionMgr`, vtable `0x172C618`. | SKU `collection` từ asset Pet trong exporter là lựa chọn catalog. Chưa nối asset này với mục tiêu, tiến độ hoặc phần thưởng của collection KR. |

Schema 7 dùng chung SKU, slot và quyền sở hữu cho nhóm `decoration`, `emotion`, `collection`.
`CatalogAccess.unlock_all` cho phép phiên O1 tra SKU đang bật mà không ghi quyền sở hữu; `equip_item`
vẫn kiểm túi thật. Mô hình chưa có enum slot KR, luật bộ sưu tập hoặc cơ chế phát hình trang trí cho peer.

Bước kiểm: lần xref từ từng RTTI/asset tới lookup và render. Trên Wine dùng hai client, bật một đại diện
mỗi slot đã nối rồi đối chiếu view local và peer ở sảnh, phòng, trong trận và sau đăng nhập lại.
Thử một emoticon và một cập nhật collection riêng để phân biệt render cục bộ với thay đổi trạng thái
qua wire. Giữ opcode và layout hai chiều của cả cụm ở **unknown** cho tới khi có writer/reader và quan sát.

## F4-d: cửa hàng riêng và trình duyệt nhúng

| Lớp RTTI | VA vtable | Ranh giới |
|---|---|---|
| `CShopStageNew` | `0x178337C` | Có catalog case của dispatcher `0x1072B10`, nhưng decompile dispatcher là `null`. |
| `CShopConnectionMan`, `CShopItemMan`, `CShopOrderMan` | `0x17827FC`, `0x178E8B0`, `0x1782C58` | RTTI và entry là mốc tìm owner; chưa gán vai trò từng slot hoặc layout đơn hàng. |
| `CNxOverlayWebBrowser` | `0x1714E2C` | Cạnh từ shop tới browser chưa nối. Entry 14–16 của dump có dạng dữ liệu UTF-16; chỉ đếm chúng là entry thô, biên vtable phải kiểm lại. |

JSON giữ toàn bộ 20 opcode S→C của shop. `103/380/529` cũng có case ở `CStage`; tra cả dispatcher
và VA khi giải mã thay vì dùng số opcode riêng lẻ.

| Cạnh cần ưu tiên | Điều đã biết từ KR |
|---|---|
| S→C `103` → `0x1076820` | Đọc `u16`; một nhánh gọi `sub_1164C10(88)` tại `0x107689F`, ghi `u16` qua `0x14EBCF0`, `u32` qua `0x14EBD90`, rồi gửi qua `0x1164D50`. Tên hai trường và điều kiện xuất hiện trên wire còn **unknown**. |
| S→C `76` → `0xD3E170`; `264` → `0x107B8D0` | Handler `76` chứa bốn site tạo `74`; `264` gọi `0x11A5450`, nơi có site tạo `524`. Đây là cạnh tĩnh; chưa có cặp request/response cho giao dịch. |
| S→C `708` → `0x10766F0`; `711` → `0x1076570` | `708` đọc một `u32` rồi dùng giá trị 1–5 để hiện `CashExchange_Fail`. Chưa biết nghĩa từng mã. `711` dùng một phần state chung, nhưng có cảnh báo decompile `bad sp`; nhánh thành công là **candidate**. |
| Site C→S quanh shop | `504/505/512/535/549/571/573/574/575/580/586/637/726/750/752/758` là site đã thấy gọi builder. Liên hệ với shop dựa theo vùng VA, còn **candidate**; payload và tên hành động **unknown**. |

`ShopCatalog` dùng được cấu hình do exporter tạo. `AccountService::purchase_shop_item` tra SKU,
kiểm số dư, trừ Lucci và cấp vật phẩm nguyên tử, có khóa chống mua lặp theo người chơi.
Đây là nền cho web shop cục bộ theo [lộ trình](../PLAN.md#full-game-roadmap); chưa có wire đơn hàng
client KR, đồng bộ shop → túi/ví hay contract URL/callback của browser. Chuỗi Cash chỉ chứng minh
có UI tương ứng, chưa xác nhận luồng thanh toán gốc.

Bước kiểm: đo mở và đóng shop trên Wine, xác định guard kết nối, opcode và cursor theo dispatcher,
rồi lần xref từ UI tới `CShopOrderMan` và `CNxOverlayWebBrowser`. Nếu nối được browser, chỉ cho client
đi tới stub cục bộ theo allowlist, ghi route và callback. Sau khi biết đường cập nhật, thử một lần
mua thành công, một lần thiếu Lucci và một lần lặp cùng yêu cầu. Đối chiếu mỗi bước với biên nhận,
ledger và việc UI cập nhật lại túi/ví; mở được màn hoặc thấy lại số dư từ stub chưa đủ nghiệm thu.
