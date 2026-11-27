# KR 00967: đặc tả F4-e

<!-- Sinh bằng python3 tools/kr_f4_powerups_spec.py --write. -->

Đặc tả dựa trên nguồn đã commit. Không có phép đo client mới hoặc hợp đồng wire F4-e.
[Dữ liệu có cấu trúc](../evidence/content/f4-powerups-boosters-spec.json) giữ xuất xứ và địa chỉ VA.
Phạm vi lấy từ [bảng xếp loại](classification.json), [lộ trình F4](../PLAN.md#full-game-roadmap) và
[chỉ đạo bao phủ nội dung](../evidence/owner-directives/full-content-coverage.json) của PR #713.

Các dấu `kr_static/confirmed` từ [F4-0](../evidence/content/f4-ida-clusters.json) và
[F4-b](../protocol/f4-inventory-cosmetics.md#f4b-inventory-contract) chỉ xác nhận case và chuỗi tĩnh.
Liên hệ với thao tác F4-e là `candidate`; không dùng các case này để bật codec.
Chính sách dưới đây là đề xuất `design`, `server_configurable` và mặc định tắt.
Điểm nối dự kiến dựa trên [AccountService và túi đồ](../architecture/f4-economy-service.md),
[projection kết quả F4-a](../protocol/life-results.md#f4a-profile-result-projection) và
[luật phòng](../../rust/crates/kr-service/src/rules.rs).

## 1. Bộ tăng sức mạnh

`파워업 키트`. Phạm vi `candidate`: Tăng chỉ số khởi đầu, đeo tối đa 2 bộ; chưa biết trường trang bị và giới hạn.

Dấu tĩnh: `CStage S→C 418 @ 0xd6af22`.

- Wire — `unknown`: Opcode và payload chọn, trang bị, tháo kit và chiếu chỉ số vào Start; ánh xạ SKU, slot và mã lỗi.
- Luật và công thức KR — `unknown`: Mức cộng theo từng kit, thứ tự cộng dồn, trần KR và cách xử lý kit hết hạn giữa trận.
- Máy chủ cho phép tối đa 2 slot kit còn hiệu lực. Cấu hình có thể hạ giới hạn; trang bị quá giới hạn bị từ chối mà không xóa lựa chọn cũ.
- Khi phòng cho phép kit, máy chủ chốt trang bị hợp lệ lúc bắt đầu trận. `RoomRules` quyết định có áp dụng kit và giữ trần hiệu lực theo nhân vật, mode và map cho số bom, sức nổ và tốc độ; kit không nâng các trần đó.
- Phòng cấm kit dùng chỉ số gốc. Chưa chiếu bonus lên client cho đến khi đo được consumer và đơn vị chỉ số.

Phép đo tối thiểu: So cùng nhân vật và map khi đeo 0, 1, 2 kit, thử thêm kit thứ ba và thử phòng cấm kit; đối chiếu Start, HUD và chỉ số khi chạm trần.

## 2. Vé ưu đãi và quyền chọn map

`지존권`, `EXP +50%`, `루찌 2배`. Phạm vi `candidate`: Chọn mọi map, VÀNG an ủi, thêm EXP và VÀNG, màn thưởng; server tự cấu hình vì game miễn phí.

- Wire — `unknown`: Opcode, payload, thời hạn và trạng thái bật vé, danh sách map hoặc màn thưởng, và cách chiếu phần thưởng cộng thêm.
- Luật và công thức KR — `unknown`: Điều kiện hưởng, VÀNG an ủi, lịch màn thưởng, gốc tính, cộng dồn, làm tròn và trần khi chốt EXP/Lucci.
- Vé chỉ cấp miễn phí theo điều kiện công khai, áp dụng như nhau cho mọi người đủ điều kiện. Không bán vé hoặc dùng vé để mua lợi thế trong trận.
- Cấu hình máy chủ quyết định nhóm phòng thường, cách cấp, hạn dùng và tỷ lệ thưởng. Tỷ lệ mặc định là 100% thưởng gốc; EXP +50% và Lucci x2 chỉ là nhãn tham khảo.
- Mọi map nghĩa là mọi map máy chủ đã mở cho mode và đã qua kiểm tra `RoomRules`. Vé không vượt allowlist, roster, hạn chế sự kiện hay luật phòng và không ưu đãi xếp hạng.
- Nếu bật thưởng, máy chủ tính một lần lúc chốt trận qua `finish_match_settled`; client không tự quyết định mức thưởng.

Phép đo tối thiểu: So ván cùng kết quả có và không có vé, thử map hợp lệ và map bị chặn. Đo UI, thay đổi EXP/Lucci và kết quả khi phát lại; projection số dư hoặc tiến độ F4-a riêng lẻ không chứng minh có booster.

## 3. Cường hóa và nâng cấp

`강화`. Phạm vi `candidate`: Nâng cấp item, pet, bóng nước và đồ RPG bằng nguyên liệu; chưa có công thức và tỉ lệ.

- Wire — `unknown`: Opcode và payload xem công thức, gửi yêu cầu, báo kết quả và cập nhật vật phẩm, pet, bóng nước hoặc đồ RPG.
- Luật và công thức KR — `unknown`: Bảng công thức, nguyên liệu, chi phí, tỷ lệ thành công, mức cộng, trần cấp và xử lý nguyên liệu hoặc vật phẩm khi thất bại.
- Công thức, nguyên liệu, chi phí, tỷ lệ và kết quả thất bại phải do máy chủ cấu hình theo từng loại vật phẩm. Không tự suy ra công thức giữa item, pet, bóng nước và đồ RPG.
- Trong chế độ thường có lưu túi, máy chủ kiểm quyền sở hữu và công thức rồi ghi kết quả cùng thay đổi túi trong một giao dịch. Phát lại cùng thao tác trả kết quả cũ, không bốc kết quả mới hoặc trừ hai lần.

Phép đo tối thiểu: Đo từng họ riêng, từ xem công thức tới một lượt thành công, thất bại và thiếu nguyên liệu. So UI, gói tin và thay đổi túi; một lần thành công không xác lập tỷ lệ KR.

## 4. Tặng quà và đổi phiếu

`선물하기`, `쿠폰`. Phạm vi `candidate`: Tặng item cho người khác và dùng phiếu đổi; bỏ phần tiền thật của Nexon.

Dấu tĩnh: `CShopStageNew S→C 236 @ 0x1072ddf`; `CShopStageNew S→C 237 @ 0x1072dfd`; `CShopStageNew S→C 345 @ 0x1072efc`.

- Wire — `unknown`: Opcode và payload gửi, nhận và đổi; mã hóa, độ dài và nghĩa trường của mã phiếu, mã kết quả và cập nhật túi.
- Luật và công thức KR — `unknown`: Điều kiện nhận hoặc tặng, danh mục đổi, hạn, hạn mức, số lần đổi và phần thưởng KR.
- Chỉ tặng vật phẩm sở hữu hợp lệ hoặc đổi phiếu cục bộ do máy chủ cấp. Loại trừ hoàn toàn Nexon Cash, thanh toán tiền thật, xác minh hóa đơn và dịch vụ đổi phiếu Nexon.
- Máy chủ cấu hình SKU được phép tặng, thời hạn, hạn mức và grant. Chuyển quà hoặc đổi phiếu ghi thay đổi túi và biên nhận trong một giao dịch; phát lại không cấp thêm. Quyền phiên O1 không chuyển thành vật phẩm có thể tặng.

Phép đo tối thiểu: Dùng hai tài khoản và phiếu thử trên stub cục bộ; đo tặng, nhận, đổi hợp lệ, hết hạn, bị từ chối và phát lại. Ghép hành động với frame, UI và biến động túi.

## 5. Mật khẩu cấp 2

`2차 비밀번호`. Phạm vi `candidate`: Lớp bảo vệ tài khoản trong game; chưa có wire tạo và xác thực.

Dấu tĩnh: `CStage S→C 245 @ 0xd6ac9c`; `CStage S→C 377 @ 0xd6abf3`; `CStage S→C 378 @ 0xd6ac05`.

- Wire — `unknown`: Opcode hai chiều và payload tạo, xác thực, khóa, mở khóa, đổi và tắt; quy cách mã hóa và mã lỗi. Các case ItemLock không xác nhận route mật khẩu cấp 2.
- Luật và công thức KR — `unknown`: Số lần sai, cửa sổ đếm, thời gian khóa, điều kiện reset và phạm vi thao tác được bảo vệ trong KR.
- Yêu cầu phiên đăng nhập chính hợp lệ. Quyền mở khóa chỉ có hiệu lực trong phiên; máy chủ thu hồi quyền khi đăng xuất và thu hồi ở mọi phiên khi đổi mật khẩu. Không ghi mật khẩu rõ vào log hay lưu trữ.
- Đếm lỗi theo tài khoản trên mọi phiên cho thao tác mở khóa, đổi hoặc tắt. Mặc định đề xuất: 5 lần sai trong cửa sổ 900 giây sẽ khóa thử lại 300 giây. Cấu hình cho phép 1–5 lần, cửa sổ 60–900 giây và khóa 60–900 giây.
- Thao tác khóa lại thu hồi quyền mở khóa của phiên. Thao tác sai không mở khóa. Kết nối lại không xóa bộ đếm; yêu cầu trong thời gian khóa bị từ chối mà không kéo dài khóa. Xóa bộ đếm khi xác thực đúng, hết cửa sổ đếm ngoài thời gian khóa hoặc hết thời gian khóa.

Phép đo tối thiểu: Đo tạo, khóa, mở đúng, mở sai, chạm giới hạn, đổi và tắt trên stub. Kiểm tra khi nối lại và khi hết khóa; chỉ giữ metadata đã loại bỏ dữ liệu xác thực.
