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

CP043 đo [159 cũ khi serial local trùng](../docs/PROTOCOL.md#late-bomb-confirm): caller cũng phải kiểm
ticket gốc trước phát159. `rejection()` chỉ kiểm lifetime; duplicate159 trong cùng generation cần
quy tắc riêng, không consume ticket timer chỉ để gửi ACK.

CP044 kiểm [timer còn chờ khi host EOF](../docs/PROTOCOL.md#abort-timer): caller gọi `finish()` ngay
khi quan sát abort dưới owner/lock; job cũ tới hạn bị loại `round_closed`. Module không tự theo dõi
socket/roster hoặc hủy timer trong scheduler; mỗi đường gửi vẫn phải kiểm ticket gốc.

Kiểm policy: `python3 -m unittest server.test_round_fence`.

`item_claims.py` giữ catalog item hiện tại, kiểm slot gắn socket/context/index/hàng/cột/kind,
chọn một claim đầu cho mỗi index và chỉ consume xác nhận local một lần. `begin/finish` làm
ticket cũ hết hiệu lực, kể cả khi dùng lại index. Caller serialize claim/consume/phát165 cùng owner phòng.
Đây là policy thử; chưa tự kiểm distance, thời gian, session key, actor state hoặc UDP. Nếu request cũ
có mọi field trùng catalog hiện tại và wire không có epoch đã xác nhận, module không phân biệt được nó;
không gọi đây là toàn validator chống stale124.

Kiểm cả hai policy: `python3 -m unittest server.test_round_fence server.test_item_claims`.

`udp_round_keys.py` là policy cấp khóa cho [nhánh movement đã đo CP047](../docs/PROTOCOL.md#udp-generation).
Owner phòng serialize `next_key()` một lần mỗi Start được chấp nhận, dùng cùng giá trị ở u32 thứ hai
của153 cho mọi client. Giữ instance khi kết thúc/hủy vòng; module không có reset/release tự động.
Sau256 giá trị byte thấp khác nhau, allocator báo lỗi; không quay vòng rồi giả định gói cũ đã hết hạn.
Chỉ tạo instance mới khi caller đã thiết lập lifetime peer mới an toàn; việc đóng transport/loại queue
để bảo đảm điều đó còn cần kiểm riêng. Roster/ID thay đổi cũng phải kiểm identity riêng.

Runtime CP047 kiểm12345→12346, stable IDs1/2 và một gói movement cũ. Module mới chỉ qua unit test,
chưa ghép vào server độc lập; các key khác/biênu32 là kiểm policy, không thêm bằng chứng runtime KR.
Không gọi đây là xác thực, toàn UDP validator hoặc bảo vệ mọi loại packet. Fixture RE CP047 chọn hai
giá trị trực tiếp qua153; runtime đó không tự chứng minh caller đã tích hợp module này.

Kiểm policy: `python3 -m unittest server.test_udp_round_keys`.

Lộ trình và điều kiện mở probe server: [PLAN](../docs/PLAN.md).
