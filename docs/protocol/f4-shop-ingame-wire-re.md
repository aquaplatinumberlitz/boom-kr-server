# F4 B1: wire shop trong trận và mỹ phẩm (IDA tĩnh)

Bản ghi đo tương ứng là [f4-shop-ingame-wire-re.json](../evidence/content/f4-shop-ingame-wire-re.json). Phân tích dùng IDA headless trên IDB KR 00967 có SHA-256 được pin trong bản ghi; mọi địa chỉ dưới đây là VA với image base `0x400000`. Đây là `kr_static/confirmed`: reader, thứ tự đọc, nhánh và lời gọi được thấy trong pseudocode/disassembly. `candidate` chỉ là nhãn nghiệp vụ để định hướng phép đo tiếp theo.

Liên hệ với phần túi đồ và mỹ phẩm hiện có ở [f4-inventory-cosmetics.md](f4-inventory-cosmetics.md). Các trường wire chưa có tên nghiệp vụ vẫn giữ `unknown`; không dùng chúng làm contract Rust.

## CShopStageNew và điều kiện vào shop

`CShopStageNew` có RTTI/vtable tại `0x178337c`, constructor `0x106df70`, được cấp tại `0x125d725` trong switch `sub_125D190` (`0x125d190`). Dispatcher stage S→C là `0x1072b10`; nó đọc opcode u16 qua reader và nhảy qua ba jump-table. Các case 229/232/236/237/345/446/447 đã được định vị trong bản ghi, nhưng chúng chỉ chứng minh handler/UI thông báo, không chứng minh request mua hàng.

Cổng vào `sub_125E540` (`0x125e540`) yêu cầu `sub_7A0180()!=0`, state `sub_1220480()!=0` (state 72/73/74), opcode đầu vào `121` và stage hiện tại `11`. Sau đó client gọi `sub_12562F0(12)` rồi `sub_125D190(...)`. `sub_125D190` đóng stage cũ khi stage id đổi và chọn constructor bằng jump-table tại `0x125d29a`. Đây là điều kiện stage/lobby quan sát được trong client; chưa có phép đo click nút sảnh hay server chấp nhận route.

## Reader chung và các hàm liên quan

`sub_11CE930` (`0x11ce930`) dùng cho 219/224 và đọc một record theo thứ tự: `u32` (`+68`), `u16` (`+64`), `u32` (`+68`), chuỗi mã hóa (`+80`, length-prefixed), 8 byte thô (`+88`, giải mã thành hai nhóm u32), `u32` (`+68`) rồi `u8` (`+60`). Record đích có các vị trí nội bộ đã đo, nhưng chưa biết cột nào là item id, số lượng, hạn hay serial.

`sub_AE6330` (`0xae6330`) là deserializer blob `UserSimpleInfo`, không phải writer shop. Sau phần tên, nó đọc các trường `u8/u8/u32/u8/u32/u16/u32/u32/u32/u8/u32`, chuỗi, `5*u16`, `2*u32`, `4*u32`, `u8` vào các offset nội bộ đã pin trong JSON. Ba block đuôi phụ thuộc guard `sub_7036F0`, `sub_7A0130`, `sub_48D8D0`; layout và ý nghĩa của chúng vẫn unknown.

## S→C trong CStage

Tất cả case dưới đây được dispatch tại `sub_D68C90` (`0xd68c90`). Không thấy guard lobby/room riêng trong chính switch; việc có được gọi hay không phụ thuộc stage/packet lifecycle bên ngoài.

| Opcode | Case → handler | Layout đã đo | Tác động tĩnh và nhãn candidate |
|---|---|---|---|
| `20` | `0xd68dac → 0xd6e890` | `u8, u8, u32, u32, u32, u32`; nhánh `sub_D95C30` đọc thêm `u8 + 4*u32 + string` | Setter profile/wallet; không gán shop semantics |
| `219` | `0xd693b7 → 0xd76860 → 0x11cea10` | outer `u8`; mỗi group `u8 index + u32 count`; mỗi record là `sub_11CE930` | Snapshot nhóm inventory; chèn vào `this + 61320*index + 36`, không reset snapshot |
| `224` | `0xd68fa9 → 0xd6d370 → 0x11ce4c0` | outer `u8`; mỗi group `u8 index + u32 count`; mỗi record là `sub_11CE930` | Reset `this + 20*index + 123852` và `this + 61320*index + 36`, áp trạng thái sau vòng; candidate snapshot thay thế |
| `225` | `0xd68fbb → 0xd6d410 → 0x11ce6a0` | outer `u8`; xóa 8 slot/group; mỗi slot `u32 item id + chuỗi mã hóa` | Candidate snapshot 8 slot equip/cosmetic; ý nghĩa chuỗi và slot chưa chốt |
| `313` | `0xd6ab2d → 0xd779c0` | `u8, u32, u32, u32, u16, u8`; dword kỳ hạn/số lượng `0` đổi thành `100000`, rồi nhân `24` | Lookup catalog, dựng record 196B, insert; candidate add/update item |
| `314` | `0xd6ab3f → 0xd77bd0` | `u8, u32, u32, u32` | Lookup record hiện có; cập nhật giá trị và kỳ hạn qua `24*x` hoặc đường xóa tương ứng; candidate item delta |
| `315` | `0xd6ab51 → 0xd77d60` | `u8, u8, u32, u32, u32, i64/u64` | Lookup catalog, cập nhật record rồi gọi `sub_11AECE0(..., 27, ...)`; candidate kết quả thao tác item |
| `316` | `0xd6ab63 → 0xd77ef0` | `u8, u32, u32`; có `i64/u64` ở nhánh `sub_1220830` và cờ catalog | Trừ trường record `+96`; khi hết gọi `sub_11CFD50` và `sub_1094080`; candidate erase/consume |
| `317` | `0xd6af10 → 0xd84410` | `u8 ok, u8 slot, u32 old, u32 item`; nếu ok đọc `16` byte (`8*u16`) và `u32` | Cập nhật hạn qua `sub_11CB960`, hiển thị `PeriodChange_Success/fail`; candidate period/upgrade response |

Các nhãn “purchase/equip/unequip/gift/erase/upgrade” chưa được xác nhận riêng theo opcode. Đặc biệt, không được suy `313/314/315/316/317` là request C→S chỉ vì chúng cập nhật manager phía client.

## C→S 566

Trong `sub_11AECE0` (`0x11aece0`), nhánh tại `0x11af978` chỉ đi tiếp khi `sub_1199D00(v217)==35` (`0x23`) và lookup catalog `sub_11E0E40` thành công. Bitmask tại record `+0x38` lần lượt gọi `sub_13E4790(a3,1)`, `sub_13E2380(a3,0)` và `sub_13E2440(a3,0)` cho các bit `1`, `0x200`, `0x400`. Sau đó builder `sub_1164C10(566)` tại `0x11afa52` ghi **đúng một byte u8** từ context `[ebx+8]` qua `sub_14EBC50` và gửi tại `0x11afa79` bằng `sub_1164D50`.

`S→C 315` gọi `sub_11AECE0(v12, 27, v7, &v8, 0, v13)`: byte gửi là u8 đầu tiên của 315; `27` là context riêng, không phải sub-opcode wire. IDA không cho thấy bảng giá trị byte ánh xạ purchase/equip/unequip/gift/erase/upgrade, cũng không cho thấy guard room/lobby tại site 566. Vì vậy các tên thao tác chỉ là `candidate`, còn action map, route stage và ACK vẫn `unknown`.

Server-side route acceptance vẫn cần live Wine captures với thao tác UI thật, capture frame hai chiều và đối chứng trạng thái. Phân tích tĩnh này không chứng minh client đã gửi trong một phiên chạy và không chứng minh server gốc chấp nhận `566`.
