# Kiểm kê quà tiến trình, danh hiệu, két Lucci và xóa thành tích F4

Phạm vi: bốn mục `roadmap_only` loại C trong [bảng xếp loại](classification.md) và
[lộ trình F4](../PLAN.md#full-game-roadmap) từ PR #713, theo
[chỉ đạo kiểm kê toàn bộ nội dung](../evidence/owner-directives/full-content-coverage.json).
Tên tiếng Hàn là neo tra cứu từ bảng xếp loại KR 00967; chúng không xác nhận hệ thống chạy được.
[Catalog JSON](../evidence/content/f4-progression-rewards-catalog.json) giữ hash và locator.

Opcode và cấu trúc gói của cả bốn mục chưa biết (`wire_contract_status: unknown`).
Bảng quà và luật server gốc cũng chưa biết (`reward_tables_status` và `original_server_rules_status` đều là `unknown`).
`server_configurable_default` là chính sách thiết kế đề xuất: tắt, chưa điền bảng và giới hạn.
Đây không phải cấu hình mà `account_serving` chấp nhận hoặc quy tắc của server gốc.
Chỉ xác nhận bảng và giới hạn gốc khi có bằng chứng KR tương ứng.

## Nền Rust đã đối chiếu

Các mục dưới đây được xác nhận trong mã nguồn dự án (`design`).
Xem thêm [kiến trúc runtime economy](../architecture/f4-economy-service.md).

- [`account_serving`](../../rust/crates/kr-service/src/account_serving.rs): Cấu hình economy mặc định tắt thưởng trận. Bật thưởng cần `persist_matches`; thưởng, catalog có mục và các projection opt-in cần hồ sơ DB. Các projection cấp và số dư không xác nhận wire của bốn hệ thống này.
- [`schema_7`](../../rust/crates/kr-service/migrations/0007_f4_progression_inventory_shop.sql): Schema 7 thêm hồ sơ theo `player_id` với EXP 0, cấp 1 và Lucci 0, ledger chỉ ghi thêm, kho SKU và slot gắn đồ.
- [`equip`](../../rust/crates/kr-service/src/accounts/economy/inventory.rs): `equip_item` cần SKU thuộc người chơi, còn hiệu lực và đúng slot catalog. Đây là trang bị chung, chưa có slot hay danh mục danh hiệu riêng.
- [`economy_policy`](../../rust/crates/kr-service/src/accounts/economy.rs): `EconomyPolicy` cấu hình EXP/Lucci theo kết quả trận và tính cấp từ mốc EXP tổng. `ECONOMY_AMOUNT_MAX` giới hạn một lượng cấu hình, không phải trần ví.
- [`settlement`](../../rust/crates/kr-service/src/accounts/matches.rs): `finish_match_settled` ghi kết quả và quyết toán thưởng trận trong cùng giao dịch; `match_players.reward` của M6 vẫn bằng 0.
- [`ledger`](../../rust/crates/kr-service/src/accounts/economy/ledger.rs): Số dư Lucci dùng số nguyên 64 bit có dấu; điều chỉnh dùng phép cộng kiểm tràn và từ chối số dư âm. `admin_grant_lucci` ghi ledger và `admin_audit` trong cùng giao dịch. Đây là đường điều chỉnh Lucci, chưa phải đường reset.
- [`lucci_projection`](../../rust/crates/kr-wire/src/kr00967/account.rs): Projection số dư opt-in dùng `remain_lucci` kiểu `u32` và chặn số lớn ở `u32::MAX` khi mã hóa; giới hạn hiển thị này không phải trần ví hoặc luật mở rộng két.
- [`schema_8`](../../rust/crates/kr-service/migrations/0008_f4_match_consumptions.sql): Schema 8 thêm biên nhận tiêu hao trong trận theo người chơi và khóa chống lặp. Biên nhận chỉ ghi thêm và tồn tại sau khi hết stack; phạm vi là tiêu hao trong trận.

## Danh hiệu (칭호)

Nền đã có: `account_serving`, `schema_7`, `equip`, `schema_8`.
Chưa biết ID, điều kiện mở khóa, trường gắn danh hiệu và cách client nhận thông báo khi đạt.

Mặc định đề xuất: `{"enabled": false, "equip_slot": null, "notify_on_unlock": false, "unlock_table": null}`.
Chỉ gắn danh hiệu đã mở khóa. Nếu bật thông báo, chỉ phát sau lần mở khóa mới đã commit; thao tác gắn lại, retry hoặc đăng nhập lại không phát thêm.

Bước kiểm KR trong mạng cô lập: Đối chiếu màn danh hiệu trước và sau khi đạt, gắn và đăng nhập lại.

## Quà lên cấp và rương lên cấp (레벨업 보물상자)

Nền đã có: `account_serving`, `schema_7`, `economy_policy`, `settlement`, `schema_8`.
Chưa biết lượng Lucci khi lên cấp, mốc và quà rương theo từng nhân vật, hoặc đường nhận quà. Bảng EXP theo kết quả trận không phải bảng quà lên cấp.

Mặc định đề xuất: `{"character_milestones": null, "enabled": false, "level_up_lucci_table": null}`.
Cấu hình riêng tiền lên cấp và rương theo cặp nhân vật, mốc cấp; mỗi quyền nhận chỉ cấp một lần, kể cả khi nhảy nhiều cấp hoặc retry.

Bước kiểm KR trong mạng cô lập: Kiểm mốc trước và sau khi lên cấp trên hai nhân vật; tách tiền trận, tiền lên cấp và rương.

## Két VÀNG và giới hạn VÀNG (루찌 금고)

Nền đã có: `account_serving`, `schema_7`, `economy_policy`, `ledger`, `lucci_projection`, `schema_8`.
Chưa biết trần Lucci gốc, lượng tăng mỗi lần, số lần mở rộng và cách xử lý tiền vượt trần.

Mặc định đề xuất: `{"base_lucci_cap": null, "enabled": false, "expansion_step": null, "max_expansions": null, "max_lucci_cap": null}`.
Khi triển khai, yêu cầu trần cơ sở và bước tăng dương, giới hạn số lần mở rộng và trần cuối; kiểm phép nhân/cộng không tràn, không vượt trần hoặc giới hạn lưu trữ. Không dùng giới hạn một khoản thưởng làm trần ví.

Bước kiểm KR trong mạng cô lập: Kiểm số dư ở sát trần và phản hồi khi mở rộng két hoặc nhận tiền vượt trần.

## Xóa thành tích (전적 지우개, 래더 재도전권)

Nền đã có: `account_serving`, `schema_7`, `ledger`, `schema_8`.
Cấp, thắng/thua và điểm ladder là phạm vi ứng viên; chưa biết mục nào ứng với từng vật phẩm, giá trị sau reset hay quan hệ với mùa ladder.

Mặc định đề xuất: `{"allowed_scopes": [], "enabled": false, "require_audit": true}`.
Từ chối khi thiếu scope, quyền, vật phẩm hợp lệ hoặc audit. Nếu được triển khai, tiêu vé, cập nhật đúng scope và ghi audit theo tác nhân, mục tiêu, trạng thái trước/sau và khóa chống lặp trong cùng giao dịch; lỗi thì rollback toàn bộ. Giữ lịch sử trận, ledger và biên nhận cũ; Schema 8 không cấp quyền reset ngoài trận.

Bước kiểm KR trong mạng cô lập: Đo riêng từng vật phẩm và so cấp, thắng/thua, điểm ladder trước và sau; kiểm vật phẩm hết hạn.

## Tái lập

```bash
python3 tools/kr_f4_progression_rewards_audit.py --write
python3 tools/kr_f4_progression_rewards_audit.py --check
```

Cả hai lệnh chỉ đọc nguồn đã có trong repo và không cần mạng, pack hay checkout bản địa hóa.
`catalog_sha256` băm toàn bộ nội dung JSON trừ chính trường hash: UTF-8, khóa sắp tăng,
không khoảng trắng giữa các phần tử, một LF cuối. `--check` kiểm cả hai file theo từng byte.
SHA-256 của catalog: `5c9f0cc549432ae9be5d99b5c261b59b4e3df84789812d3d54ba89c8f6119241`.
