# F7 — Kiểm kê tĩnh và chính sách map tự tạo

Sinh bằng `python3 tools/kr_f7_custom_map_audit.py --write`; kiểm từng byte bằng `--check`.

## Kiểm kê KR 00967

Nguồn `kr_static`, trạng thái `confirmed` chỉ cho metadata trong [census](census.json). [Xếp loại](classification.json) giữ các họ map và hệ thống CustomMap ở lớp C, trạng thái `candidate`.
Census ghi 158 mục trong `fx/CustomMap`. Các tên bảng trong nhóm mang tiền tố `CustomMap`, `CustomTileIndexList` và `CustomTileList`; chưa giải định dạng các bảng này.

| Map trong `FxMap` | Lưới | Điểm xuất phát theo census | Lưới thuộc danh sách cho phép đề xuất |
|---|---|---:|---|
| `CustomMap15x13` | `15x13` | 12 | Có |
| `CustomMap15x27` | `15x27` | 12 | Không |
| `CustomMap21x21` | `21x21` | 12 | Có |
| `CustomMap31x13` | `31x13` | 12 | Có |
| `CustomMap31x27` | `31x27` | 12 | Không |

[Chỉ đạo CustomMap](../evidence/owner-directives/custom-map.json) và [lộ trình F7](../PLAN.md#full-game-roadmap) nêu năm map và ba lưới `15x13`, `21x21`, `31x13`. Census vẫn có năm map nhưng có thêm hai cỡ `15x27` và `31x27`. Đề xuất giữ đúng ba cỡ cho phép; hai cỡ còn lại chỉ ghi nhận trong kiểm kê.
Cả năm dòng census đều có script và ghi 12 điểm xuất phát. Metadata này không xác nhận khả năng chơi, vị trí hợp lệ của các điểm xuất phát hay số người của phòng. Các map gốc không mặc nhiên đạt validator dưới đây.

## Chính sách fail-closed đề xuất

Nguồn và trạng thái đều là `design`. F7 tắt mặc định; các quy tắc này dành cho dữ liệu đã chuẩn hóa của server.

- Chỉ nhận lưới `15x13`, `21x21`, `31x13` theo thứ tự rộng × cao. File đầu vào và payload chuẩn hóa phải có từ 1 đến 8192 byte; chặn đọc quá giới hạn và chặn payload trước khi parse. Chỉ nhận JSON UTF-8, không giải nén, không nhận khóa lặp hoặc trường lạ và không thực thi script hay nạp tài nguyên do người chơi cung cấp.
- Schema `kre-f7-normalized-map-v1` chỉ có `schema`, `width`, `height`, `tiles`, `spawns`. `tiles` phải có đúng `width * height` ô theo thứ tự hàng. Mỗi ID là số nguyên trong `0..65535` và phải nằm trong bảng tile tin cậy của server kèm cờ đi bộ được. Chưa có bảng được duyệt cho KR thì từ chối.
- Cần 2–8 điểm xuất phát khác nhau, đủ cho số người đang chơi khi chọn/nạp map. Mỗi điểm là `[x, y]` nguyên, bắt đầu từ 0, nằm trong lưới và trên ô đi bộ được. Dùng đường đi bốn hướng qua các ô đi bộ được để kiểm mọi điểm xuất phát cùng một vùng liên thông; không tính đi chéo hoặc phá chướng ngại.
- Tính SHA-256 của bảng tile dưới dạng mảng cặp `[id, walkable]` theo ID tăng dần. Sau đó băm đối tượng `{"map": <map>, "palette_sha256": <hash>}`. Cả hai bước dùng JSON gọn, khóa sắp xếp, UTF-8 và một LF cuối; giữ thứ tự tile và điểm xuất phát. Thay map hoặc bảng tile làm đổi pin; pin chỉ định danh dữ liệu, không cấp quyền.
- Server kiểm trước khi duyệt hoặc chia sẻ, giữ bản đã duyệt theo hash bất biến và kiểm lại trước khi chia sẻ, chọn hoặc nạp. Chỉ dùng pin đã duyệt do server giữ, kiểm quyền của người yêu cầu và kiểm lại đủ chỗ theo roster hiện hành khi chọn/nạp. Thiếu dữ liệu, thiếu quyền, sai hash hoặc vượt giới hạn thì từ chối; không chia sẻ, không đổi map của phòng và không gửi ACK thành công giả.

## Wire và cách đo tiếp

Upload và chọn map trên wire đều là `unmeasured`: chưa biết opcode, layout, định danh hay bên có quyền quyết định trong KR. Chưa chọn trình dựng trong client, công cụ web hay file chia sẻ. Giữ các route tắt cho tới khi RE tĩnh xác định đường xử lý và Wine cô lập với stub/server cục bộ đo luồng client từ mở UI đến upload, lựa chọn, nạp và bắt đầu ván, gồm phản hồi từ chối và trạng thái phòng. Không dùng schema chuẩn hóa làm layout wire KR.

## Tái lập

Mặc định chỉ đọc ba nguồn đã commit. [JSON bằng chứng](../evidence/content/f7-custom-map-spec.json) giữ pin SHA-256 của từng nguồn, client và spec. Có thể thêm `--pack-root <thư-mục-pack>` khi chạy `--write` hoặc `--check` để so cả hai index cùng các pin của `FxMap.idd`; riêng `fx.idd` chỉ có kích thước trong census. Kiểm pack là tùy chọn đọc tại chỗ, không thay đầu ra và không xác nhận client thực thi.

SHA-256 của spec chuẩn hóa (bỏ trường `spec_sha256`): `60dcafef80e81b2dd8cfcf4d1e613c0d5fabc6fb8e75775ef1826da440a9c78c`.
