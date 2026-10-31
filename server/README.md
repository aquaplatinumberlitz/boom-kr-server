# Server tương thích KR

Code server mới thuộc thư mục này. Harness RE trong `local/` đã chạy các chu kỳ hai client
Wine theo [contract](../docs/PROTOCOL.md); chưa có server độc lập hoặc nghiệm thu toàn codec/hello.

`round_fence.py` là policy emulator cho vòng đời **job local**: ticket gắn instance chủ phòng,
generation vòng và lần cấp ID; chỉ job hiện tại được consume một lần. Caller phải giữ ticket gốc,
gọi `begin()` khi Start được chấp nhận, `finish()` khi kết thúc/hủy vòng và serialize mọi thao tác
cùng việc phát packet dưới owner/lock phòng. Timer còn giữ mỗi `wire_id` không được đổi sang ticket
mới khi chạy. `register()` không tự chọn ID; nó từ chối ID đang active hoặc ngoài1..65535.

Đây là design của server, không thêm field generation vào wire KR. Nó không kiểm request từ client,
deadline, chain đã xử lý tại client, UDP, packet đã gửi hoặc allocator wrap/quarantine. Các phạm vi đó
vẫn cần kiểm riêng; không gọi module này là toàn validator hoặc đủ đóng K5/K6.

Kiểm policy: `python3 -m unittest server.test_round_fence`.

Lộ trình và điều kiện mở probe server: [PLAN](../docs/PLAN.md).
