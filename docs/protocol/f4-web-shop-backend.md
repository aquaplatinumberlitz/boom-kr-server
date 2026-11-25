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

<a id="f4d-shop-browser-xrefs-w3"></a>
## Caller của `CShopStageNew` và `CNxOverlayWebBrowser` (Wave 3)

Bổ sung từ [f4-w3-ida-records.json](../evidence/content/f4-w3-ida-records.json) (`kr_static/confirmed`):

- **`CShopStageNew` (`sub_106DF70`)**: được gọi khởi tạo tại `sub_111AA20`, `sub_11543A0` và `sub_125D190`. Case `232` (`sub_11A8F80`) thực tế là stub rỗng trong ảnh KR 00967 này; helper `sub_107BA70` khi `sub_1220830() == 0` gửi `C->S 541` (`0x21D`) kèm byte `*(this + 0xDD60)`. Các case `107` (`sub_107BB90`) và `124` (`sub_107BB00`) đều có nhánh gọi `sub_1243A30(1)`.
- **`CNxOverlayWebBrowser` (`sub_809A60` / `sub_80C9B0`)**: constructor chỉ thấy xref từ `sub_808990`, còn `sub_80C9B0` lưu ngữ cảnh vào `this` và gọi vtable `+48` trên `*(this+1)`. Cạnh kích hoạt UI cửa hàng từ sảnh giữ chờ nghiệm thu trực tiếp qua Wine.

<a id="f4d-shop-wine-probe-w3"></a>
### Kết quả probe Wine nút Cửa hàng (Wave 3)

Trong `f4-ui-probe-v1` ([f4-ui-probe-wine.json](../evidence/content/f4-ui-probe-wine.json)), thao tác click đơn vào nút `상점` tại `(290, 678)` trên `CookieB` làm bật tooltip `상점 가기`, nhưng `CA.exe` không chuyển sang `CShopStageNew`, không mở `CNxOverlayWebBrowser` và không phát `C->S` nào ngoài giữ kết nối sảnh (`715`, `15`). Nguyên nhân (điều kiện click trong Wine vs guard cục bộ kiểu `sub_1220830()` hoặc cờ ở gói vào sảnh) còn `unknown`.

<a id="f4d-shop-gates-w4"></a>
### Phân tích tĩnh gate `sub_1220830` và stage switch cửa hàng (Wave 4)

Bổ sung từ [f4-w4-ida-gates.json](../evidence/content/f4-w4-ida-gates.json) (`kr_static/confirmed`):

- **Hàm gate `sub_1220830` (`0x1220830`)**: kiểm tra đồng thời `sub_4961E0() != 0` (`sub_456AD0(26) == 0`) và `sub_1220030() == 130` (`sub_13AD5B0(sub_545E30())` trỏ `sub_13A2A10` -> `sub_57A830`). Hàm này được gọi ở `89` vị trí (35 ngoài dải UI/stage, 54 trong `0xD60000..0xD90000` và `0x1030000..0x1260000`), bao gồm `sub_107BA70` trước khi dựng `C->S 541`. Nghĩa của flag `26` và trạng thái `130` còn `unknown`.
- **Stage switch `sub_125D190` (`0x125D190`, gọi bởi `sub_125E540`)**: dọn stage cũ ở `*(this + 240)` và khởi tạo các stage con kế thừa `CShopStageNew` (`sub_106DF70`), gồm `CUserMarketStage` (`sub_111AA20` tại `0x125D857`) và `CMagicBookMarketStage` (`sub_11543A0` tại `0x125D943`). Bảng giá trị chuyển `a2` và điều kiện nút `상점` gọi tới `sub_125E540` vẫn chờ đo runtime.

<a id="f4d-shop-flag26-w5"></a>
### Giải mã `sub_456AD0`, gate `sub_125E540` và token UI cửa hàng (Wave 5)

Bổ sung từ [f4-w5-ida-ui-and-exp.json](../evidence/content/f4-w5-ida-ui-and-exp.json) (`kr_static/confirmed`):

- **`sub_456AD0` (`0x456AD0`) là hàm hằng `return 0;`**: cả `sub_4961E0()` (`0x4961E0`) lẫn `sub_844690()` (`0x844690`) đều luôn trả `true`; vì vậy `sub_1220830()` chỉ phụ thuộc `sub_1220030() == 130`. Ngược lại, `sub_76FBA0()` (`0x76FBA0`) trả `0` cố định và khóa nhánh `CUserMarketStage` (`0x125D857`) trong `sub_125D190`.
- **Điều kiện gọi `sub_125D190` từ `sub_125E540` (`0x125E540`, caller `sub_125DD40`)**: chỉ chạy `sub_12562F0(12)` rồi chuyển stage khi `sub_7A0180() != 0`, `sub_1220480()` (`sub_1220030()` thuộc `{72, 73, 74}`), `u16(packet) == 121`, và `sub_125D170(this) == 11`.
- **Token stage UI `sub_124D6A0` (`0x124D6A0`)**: ánh xạ `"Shop" -> 0`, `"ShopTab" -> 1`, `"ShopFreeStock" -> 2`, `"ShopStock" -> 3`, `"Prepare" -> 4`, `"MyPage" -> 5`, `"Square" -> 6`, `"LowWinRateChannel" -> 7`; chuỗi nút gồm `"GlobalUI/BtUserMarket"` (`0xBED910`), `"Dialog/BtMyPage"` (`0xC17FE0`/`0xC5EB80`) và `"ConfirmGoToMyPageNormal"` (`0xD73C50`). Cạnh nối từ nút `상점` sảnh tới `sub_125D190` vẫn chờ đo Wine.

<a id="f4d-webshop-wine-e2e"></a>
## Kiểm chứng E2E Wine + `public_launcher` TLS + `web_shop` nhúng (Wave 5)

Bằng chứng thực nghiệm tại [f4d-webshop-wine-e2e.json](../evidence/content/f4d-webshop-wine-e2e.json) (`kre_runtime/e2e_verified`, baseline `0df8d81`) xác nhận toàn bộ vòng khép kín trên bản build `f4-rust-v2`:

- **Ván Wine 2 người chơi (`Tomb12`)**: `CA.exe` (A/B) hoàn tất trận loại trực tiếp (`206` frame TCP khớp CRC, `1867` gói UDP, `room_count=0` sau khi rời phòng). `kr-service` đẩy `S->C 135` tăng `0 -> 3500` Lucci cho `CookieA` (thắng, `+500 EXP` lên level 2) và `0 -> 30` Lucci cho `CookieB` (thua, `+30 EXP` level 1).
- **Giới hạn một phiên hoạt động & thoát sạch**: Khi hai phiên Wine còn mở, `public_launcher` (`TLS 1.3` ALPN `kre-launch/1`) từ chối `login` cho `CookieA` với `denied_or_busy`. Sau khi tắt sạch `kr-prelogin` qua `cancel_file` và `wineserver -k` (`native_tls_owner_ended` ok x2), `CookieA` đăng nhập lại và đổi `web_shop_token` thành công.
- **`web_shop` nhúng (`8.316` mục, `mode=standard`)**: Kiểm tra HTML `/` không chứa `innerHTML` và giữ `history.replaceState` xóa fragment; `POST /api/v1/purchase` mua `character.0` (`3000` Lucci) với `request_id` idempotent (`500` Lucci còn lại, retry trả `replayed: true`), `equip` -> `inventory` -> `unequip` sạch, `CookieB` (`30` Lucci) bị từ chối `402` (`request_failed`), và `logout` thu hồi token `CookieA` (`401`). SQLite giữ đúng `player_profiles`, `ledger_entries` (`match_reward` x2 + `shop_purchase`), `player_inventory` (`character.0`) và `player_equipped` rỗng.

<a id="f4d-shop-jumptable-w6"></a>
## Các khối stage của `sub_125D190` và caller `sub_124D6A0` (Wave 6)

Từ [f4-w6-ida-own291-and-hud.json](../evidence/content/f4-w6-ida-own291-and-hud.json) (`kr_static/confirmed`):

- **`sub_125D190` (`0x125D190`, `0x125D29A jmp near ptr word_205DA52`)**: chuyển qua bảng nhảy gián tiếp tới các khối dựng `CShopStageNew` (`0x106DF70`), `CChattingStage` (`0xD556A0`), `CExchangeStage` (`0x10DF8A0`), `CUserMarketStage` (`0x111AA20`, khóa bởi `sub_76FBA0() == 0`), `CSquareStage` (`0x1129180`, qua `sub_BE0B90()`), `CMagicBookMarketStage` (`0x11543A0`, qua `sub_844690()`), và `CCustomMapStage` (`0x98C340`), cùng các nhánh nội bộ `sub_D9F3F0`, `sub_79BB10`, `sub_125E330`, `sub_125E240`.
- **Caller của `sub_124D6A0` (`0x124D6A0`)**: gồm constructor `CFxMessage` (`0x124D240`), bộ nạp bảng chuỗi `sub_124D5D0` và `sub_D77390`.

<a id="f4d-o1-unlock-all-e2e"></a>
## Kiểm chứng E2E chế độ O1 `--unlock-all` (Wave 6)

Bằng chứng thực nghiệm tại [f4d-o1-unlock-all-e2e.json](../evidence/content/f4d-o1-unlock-all-e2e.json) (`kre_runtime/o1_unlock_all_verified`, build `f4-rust-v2`) chạy `kr-service` với `unlock_all_catalog: true`, `web_shop.unlock_all: true` và catalog `8.316` mục xác nhận:

- **Truy cập O1 theo phiên**: `public_launcher` (`TLS 1.3` ALPN `kre-launch/1`) đăng nhập và cấp `web_shop_token`; `GET /health` trả `mode="unlock_all"`, `catalog_items=8316`; `GET /api/v1/catalog?category=character` trả đủ `67/67` SKU với `usable=true`, `unlocked_by_session=true`, `quantity=null`, `expires_at=null`.
- **Không trừ Lucci và không ghi DB**: `POST /api/v1/purchase` bị chặn ở `409` `unlock_all_no_purchase`, `GET /api/v1/inventory` rỗng, `logout` thu hồi token về `401`, và SQLite sau tắt `kr-service` giữ `0` dòng trên cả `ledger_entries`, `player_inventory`, `player_equipped` và `player_profiles`.
