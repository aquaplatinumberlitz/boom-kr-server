# Ma trận hệ thống xã hội F5 mở rộng

Build `KR00967`: **6 họ, 22 map, 4 hệ thống mở rộng**. Còn `unknown` về hành vi của 22/22 script.

Đây là kiểm kê offline. Neo IDA tĩnh và tài nguyên pack chưa xác nhận luồng client, luật server hoặc payload KR; chưa có phép đo runtime mới.

Nguồn: [bảng xếp loại](classification.json), [catalog JSON](../evidence/content/f5-social-catalog.json), [kiểm kê F5-0](f5-social-catalog.md), [neo IDA](../evidence/content/f2-f5-f3-ida-anchors.json), [chỉ đạo phủ nội dung](../evidence/owner-directives/full-content-coverage.json), [Schema 9](../../rust/crates/kr-service/migrations/0009_f5_social_foundation.sql) và [lộ trình F5](../PLAN.md#full-game-roadmap). Hash đầu vào và định danh bản client nằm trong [ma trận JSON](../evidence/content/f5-extended-social-matrix.json).

## Các họ map quảng trường

| Họ map | Đợt / lớp | Map | Script chưa rõ / tổng | Vai trò dự kiến | Opcode KR |
|---|---|---:|---:|---|---|
| `LobbySquare` | `F5?` / C | 9 | 9/9 | Ứng viên cho sảnh quảng trường đi lại tự do. | `unknown` |
| `LobbyCaffe` | `F5?` / C | 5 | 5/5 | Khu cà phê; lộ trình nêu quà khi ngồi, chưa biết điều kiện và quà cụ thể. | `unknown` |
| `LobbyClothing` | `F5?` / C | 2 | 2/2 | Khu quần áo; chưa biết thao tác tại khu. | `unknown` |
| `LobbyJewelry` | `F5?` / C | 2 | 2/2 | Khu trang sức; chưa biết thao tác tại khu. | `unknown` |
| `LobbyPet` | `F5?` / C | 2 | 2/2 | Khu thú cưng; chưa biết thao tác tại khu. | `unknown` |
| `Square` | `F5?` / C | 2 | 2/2 | Ứng viên cho map quảng trường; chưa biết quan hệ chuyển cảnh với LobbySquare. | `unknown` |

## Hành vi cần đo và phần lưu trữ cần bổ sung

Schema 9 đã có bảng quan hệ xã hội, kiểm soát chat và thành viên bang. Các phần mở rộng dưới đây là đề xuất thiết kế và chưa có migration.

### Quảng trường (광장)

Sảnh đi lại tự do và các khu quần áo, trang sức, pet, cà phê; chưa có wire di chuyển và đồng bộ.

Wire `unknown`: Chuyển cảnh, danh sách người, di chuyển, tương tác và quà cà phê.

- `unknown`: Chưa rõ thời gian ngồi, vật phẩm và điều kiện nhận lại. Chưa có bằng chứng về quà ở các khu khác.

Mở rộng Schema 9 (`design`): Giữ hiện diện và vị trí theo phiên; nếu có quà bền, cần lưu lần nhận và chống cấp trùng khi ghi thưởng vào túi đồ.

Phép đo đề xuất trong mạng cô lập: Vào và rời từng họ; dùng hai client để thử thấy nhau di chuyển. Đối chiếu ngồi và không ngồi ở quán cà phê để xác định đường nhận quà.

### Thư (쪽지)

Gửi và nhận thư, hộp thư có giới hạn; chưa có wire và lưu trữ.

Wire `unknown`: Gửi, nhận, liệt kê, đọc, xóa và phản hồi khi hộp đầy.

- `unknown`: Có giới hạn hộp thư; chưa biết số thư tối đa và cách xử lý khi đầy.

Mở rộng Schema 9 (`design`): Lưu thư theo người gửi và người nhận, trạng thái đọc/xóa, chỉ mục hộp thư và kiểm giới hạn trong giao dịch ghi. Có thể dùng lại `player_blocks`.

Phép đo đề xuất trong mạng cô lập: Gửi cho người đang offline, kiểm sau đăng nhập lại; đo biên đầy, thử gửi thêm và thử lại sau khi xóa một thư.

### Phòng chat trao đổi (교환채팅방)

Kênh chat để trao đổi đồ giữa người chơi; chưa có wire.

Wire `unknown`: Tạo, tìm, tham gia, rời phòng và chuyển tin.

- `unknown`: Chưa rõ phòng dùng CHOCO hay có giao dịch đồ gắn với phòng.

Mở rộng Schema 9 (`design`): Thiết kế phòng, thành viên và hiện diện theo phiên. Chỉ xem xét lưu giao dịch đồ sau khi xác nhận có luồng đổi đồ riêng. Có thể dùng lại `player_blocks`, `social_mutes`, `chat_rate_windows`.

Phép đo đề xuất trong mạng cô lập: Tạo và tham gia từ hai client; đo chat, rời phòng và thử thao tác đổi đồ riêng để kiểm quan hệ với phòng chat.

### Huy hiệu và bang kín (길드 마크, 비공개 길드)

Đổi huy hiệu bang, bang kín duyệt đơn tối đa 7 ngày; chưa có wire.

Wire `unknown`: Xem/đổi huy hiệu, đổi quyền riêng tư, nộp đơn, duyệt, từ chối và hết hạn.

Neo IDA `kr_static/confirmed`: `S->C 467` (0x6d9340); `S->C 474` (0xd769d0); `S->C 478` (0xd76da0); `C->S 633` (0xd769d0). Các neo không xác nhận wire của toàn bộ thao tác mở rộng.

- `candidate`: Mốc tối đa theo bảng xếp loại; chưa biết lúc bắt đầu tính, biên hết hạn và hành vi khi hết hạn.

Mở rộng Schema 9 (`design`): Bổ sung tham chiếu huy hiệu, quyền hiển thị/gia nhập và đơn gia nhập có thời hạn, quyết định và người duyệt. Duyệt và ghi thành viên trong cùng giao dịch. Có thể dùng lại `guilds`, `guild_members`.

Phép đo đề xuất trong mạng cô lập: Đối chiếu đổi huy hiệu và nộp đơn bang kín với mời bang thường; thử quyền duyệt, hết hạn và trạng thái thành viên sau đăng nhập lại.

### Cặp đôi (커플)

Chưa có đủ wire, luật hoặc quyền quyết định của server trong KR 00967.

Wire `unknown`: Mời, chấp nhận, đồng bộ cặp, đổi trang bị và kết thúc quan hệ.

Neo IDA `kr_static/confirmed`: `S->C 257` (0x128d3c0); `S->C 273` (0xd87250); `S->C 284` (0x128d120); `S->C 640` (0xd88830); `S->C 674` (0xd7fa60); `C->S 670` (0x583530, 0x583e80). Các neo không xác nhận wire của toàn bộ thao tác mở rộng.

- `unknown`: Nhẫn đôi thuộc phạm vi khảo sát; chưa có ID, điều kiện trang bị hay quan hệ đã kiểm với Kim đôi trong lộ trình.
- `unknown`: Lộ trình nêu thưởng EXP; chưa biết mức, điều kiện và cách cộng dồn.
- `unknown`: Lộ trình nêu thưởng Lucci (VÀNG); chưa biết mức, điều kiện và cách cộng dồn.

Mở rộng Schema 9 (`design`): Lưu lời mời và quan hệ cặp, tham chiếu nhẫn nếu đo được, và liên kết thưởng EXP/Lucci với lần chốt kết quả để tránh ghi trùng.

Phép đo đề xuất trong mạng cô lập: Đo mời/chấp nhận/từ chối và đăng nhập lại; so hai ván có/không có cặp hoặc trang bị để tách hiệu ứng nhẫn khỏi thưởng EXP/Lucci.

## Tái lập offline

```bash
python3 tools/kr_f5_extended_social_audit.py --write
python3 tools/kr_f5_extended_social_audit.py --check
```

SHA-256 của JSON chuẩn hóa, bỏ trường `matrix_sha256`: `784e82006196d58d80e6b3b869e5c1759d35994f3dbc61ac72811885a78aa089`.
