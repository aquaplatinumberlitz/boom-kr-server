<a id="f4d-web-shop-backend"></a>
# F4-d: backend HTTP cửa hàng web nhúng

Backend này là HTTP/1.1 bounded, chỉ phục vụ catalog kinh tế cục bộ của `kr-service` qua `AccountService` Schema 7. Cấu hình yêu cầu địa chỉ IPv4 cụ thể, danh sách peer cho phép, tối đa 64 kết nối, giới hạn header/body và thời gian xử lý. Không có kết nối HTTP ra ngoài; trang `/` tự chứa HTML, CSS và JavaScript, kèm CSP `connect-src 'self'`.

## Phiên và danh tính

`WebShopAuth` phát token ngẫu nhiên 32 byte, lưu SHA-256 digest trong bảng tối đa 1.024 phiên và gắn mỗi digest với một `player_id` dương. Token hết hạn theo `session_ttl_seconds`; có thể thu hồi theo token hoặc người chơi. So sánh digest chạy theo thời gian không phụ thuộc nội dung. Token chỉ nằm trong bộ nhớ của caller và không có `Debug`/`Display`; HTTP không chấp nhận `player_id` từ query hoặc JSON.

## HTTP contract

`GET /health` không cần đăng nhập và trả mode, số item catalog và `rewards_enabled`. `GET /` trả trang cửa hàng UTF-8. Các route `/api/v1/profile`, `/api/v1/catalog`, `/api/v1/inventory`, `/api/v1/purchase`, `/api/v1/equip` và `/api/v1/unequip` yêu cầu `Authorization: Bearer <token>`. Catalog lọc bằng `?category=character|decoration|emotion|consumable|collection`.

Chế độ thường đọc và ghi inventory/equip, trừ Lucci qua `purchase_shop_item`, và khóa replay bằng `request_id`. Replay cùng payload trả lại receipt; payload khác cùng khóa trả `409`. `unlock_all` chỉ cấp quyền dùng session từ catalog enabled, không đọc/ghi ownership và không trừ Lucci; mọi purchase ở chế độ này trả `409` với `unlock_all_no_purchase`.

Lỗi đầu vào trả `400`, token thiếu/sai/hết hạn trả `401`, thiếu Lucci trả `402`, xung đột ownership hoặc idempotency trả `409`, SKU không hợp lệ được trả `400`, queue worker bận/hết hạn trả `503`. Các response có `Cache-Control: no-store` và `X-Content-Type-Options: nosniff`.

Binary `kr-webshop --config <path>` đọc database, `web_shop` và economy profile JSON. Profile có thể là output trực tiếp của `tools/kr_f4_catalog_export.py` với `economy_policy`, `shop_catalog` và `summary`.

Phía native KR vẫn còn **unknown**: callback URL của `CShopStageNew`, cạnh nối tới `CNxOverlayWebBrowser`, và các opcode bridge/callback. Những điểm này chờ đo Wine; backend này không suy đoán wire opcode hay URL gốc. Xem thêm [cụm F4-d](../content/f4-mechanic-clusters.md#f4-d-cửa-hàng-riêng-và-trình-duyệt-nhúng).
