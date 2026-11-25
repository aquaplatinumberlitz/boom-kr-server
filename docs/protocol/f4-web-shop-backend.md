<a id="f4d-web-shop-backend"></a>
# F4-d: backend HTTP cửa hàng web nhúng

Backend này là HTTP/1.1 bounded, chỉ phục vụ catalog kinh tế cục bộ của `kr-service` qua `AccountService` Schema 7. Cấu hình yêu cầu địa chỉ IPv4 cụ thể, danh sách peer cho phép, tối đa 64 kết nối, giới hạn header/body và thời gian xử lý. Không có kết nối HTTP ra ngoài; trang `/` tự chứa HTML, CSS và JavaScript, kèm CSP `connect-src 'self'`.

## Phiên và danh tính

`WebShopAuth` phát token ngẫu nhiên 32 byte, lưu SHA-256 digest trong bảng tối đa 1.024 phiên và gắn mỗi digest với một `player_id` dương. Token hết hạn theo `session_ttl_seconds`; có thể thu hồi theo token hoặc người chơi. So sánh digest chạy theo thời gian không phụ thuộc nội dung. Token chỉ nằm trong bộ nhớ của caller và không có `Debug`/`Display`; HTTP không chấp nhận `player_id` từ query hoặc JSON.

Khi nhúng vào `kr-service` qua trường tùy chọn `account_serving.web_shop`, cấu hình bắt buộc `profile_mode = database_zero` và `web_shop.unlock_all == account_serving.unlock_all_catalog`. Launcher công khai (`public_launcher`) hỗ trợ `{"op":"web_shop_token"}` để cấp token sau khi kiểm tra session hợp lệ của đúng `player_id`, và tự thu hồi toàn bộ web shop token của người chơi khi `{"op":"logout"}` thành công.

## HTTP contract

`GET /health` không cần đăng nhập và trả mode, số item catalog và `rewards_enabled`. `GET /` trả trang cửa hàng UTF-8. Các route `/api/v1/profile`, `/api/v1/catalog`, `/api/v1/inventory`, `/api/v1/purchase`, `/api/v1/equip` và `/api/v1/unequip` yêu cầu `Authorization: Bearer <token>`. Catalog lọc bằng `?category=character|decoration|emotion|consumable|collection`.

Chế độ thường đọc và ghi inventory/equip, trừ Lucci qua `purchase_shop_item`, và khóa replay bằng `request_id`. Replay cùng payload trả lại receipt; payload khác cùng khóa trả `409`. `unlock_all` chỉ cấp quyền dùng session từ catalog enabled, không đọc/ghi ownership và không trừ Lucci; mọi purchase ở chế độ này trả `409` với `unlock_all_no_purchase`.

Lỗi đầu vào trả `400`, token thiếu/sai/hết hạn trả `401`, thiếu Lucci trả `402`, xung đột ownership hoặc idempotency trả `409`, SKU không hợp lệ được trả `400`, queue worker bận/hết hạn trả `503`. Các response có `Cache-Control: no-store` và `X-Content-Type-Options: nosniff`.

Binary `kr-webshop --config <path>` đọc database, `web_shop` và economy profile JSON. Profile có thể là output trực tiếp của `tools/kr_f4_catalog_export.py` với `economy_policy`, `shop_catalog` và `summary`.

Phía native KR vẫn còn **unknown**: callback URL của `CShopStageNew`, cạnh nối tới `CNxOverlayWebBrowser`, và các opcode bridge/callback. Những điểm này chờ đo Wine; backend này không suy đoán wire opcode hay URL gốc. Xem thêm [cụm F4-d](../content/f4-mechanic-clusters.md#f4-d-cửa-hàng-riêng-và-trình-duyệt-nhúng).

<a id="f4d-shop-entry-ida-w2"></a>
## Phân tích tĩnh `CShopStageNew` và các manager cửa hàng (Wave 2)

Tóm tắt tĩnh được lưu tại [f4-w2-ida-deepdive.json](../evidence/content/f4-w2-ida-deepdive.json) (`kr_static/confirmed`):

- **`CShopStageNew::OnPacket` (`0x1072B10`)**: phân phối qua ba jump table (`jpt_1072B7C` từ `76` gồm 189 case đến `345`, `jpt_1072BB9` từ `380` gồm 68 case đến `529`, `jpt_1072BE3` từ `534` gồm 178 case) cùng các nhánh lẻ tại `345` (`0x1072EFC`) và `529` (`0x1072E51`).
- **Case đã thấy trong disassembly đầu hàm**:
  - `103` -> `sub_1076820`: đọc `u16 v14` (`vtable+64`), kiểm tra `sub_135E950(v14)`; ở nhánh chính dựng `C->S 88` chứa `u16` (`sub_1243940`) và `i32`/`u32` (`sub_12439D0`), gửi qua `sub_1164D50` và gọi `sub_12562F0(v14)`.
  - `107` -> `sub_107BB90`; `124` -> `sub_107BB00`.
  - `229` (`0x1072C96`): qua guard `sub_850D20`/`sub_BFABF0`/`sub_D972B0`/`sub_1073160`, gọi `sub_107BA70` + `sub_11D0070`; khi `sub_1220830() == 0` thì gửi `C->S 512` (`0x200`) bắt đầu bằng `u8 = 2` và byte `[this + 0xDD60]`.
  - `232` (`0x1072C20`): chuẩn bị dữ liệu qua `sub_13B40F0`/`sub_125E150`/`[this + 0xDD60]` rồi gọi `sub_11A8F80` (hàm mang xref tới chuỗi `URL_CashCharge_CN` tại `0x11A93B2`).
- **`sub_10766F0` / `sub_1076570`**: `sub_10766F0` đọc `u32` `1..=5` để bật `CashExchange_Fail` (`Dialog_CashExchange`); `sub_1076570` gọi `sub_13E4B50(v10, v11)` và reset hộp thoại cửa hàng (`sub_9671C0`).
- **Mốc F4-a liên quan (`CStage` setter)**: `sub_13E4950` (`0x13E4950`, Lucci clamp `0..=sub_13E5150`, 25 xrefs gồm `sub_D6CBA0`), `sub_13E4790` (`0x13E4790`, grade/level kiểm ngưỡng `< 60`, 15 xrefs gồm `sub_D6CBD0` và `sub_D6CFF0`). Nghĩa từng tham số/field của `sub_D6CBD0`/`sub_D6CFF0` và luồng mở `CNxOverlayWebBrowser` vẫn **unknown** cho tới khi đo Wine.
