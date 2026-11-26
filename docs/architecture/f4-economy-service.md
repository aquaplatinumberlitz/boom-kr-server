# Kiến trúc runtime F4 economy

`Session::consume_item_in_match` là hook nội bộ cho hành động đã được luật trận
chấp nhận. Hook lấy người chơi từ kết nối đã xác thực, kiểm lượt `Armed` hiện
hành và tạo khóa cùng digest từ ID hành động do server cấp. Luật trận vẫn quản
lý hiệu ứng, giới hạn dùng và chống lặp cho O1.

Standard yêu cầu `economy_policy.enabled` và `persist_matches`. Journal cung cấp
ID của trận đã commit. Qua hàng đợi có giới hạn của `AccountService`, cùng một
giao dịch kiểm phiên và slot đã trang bị, trừ stack và lưu biên nhận Schema 8.
Retry khớp trong lượt trả biên nhận gốc, kể cả khi stack đã hết. Khóa phòng được
nhả trước khi chờ worker; hook kiểm lại lượt và thành viên trước khi trả kết quả.

Với `unlock_all_catalog`, hook kiểm quyền stackable từ catalog và hạn tuyệt đối
của phiên trong bộ nhớ, không dùng journal hay SQLite cho lần tiêu hao.
`WebShopAuth` giữ `O1SessionCosmetics` riêng theo token. Public launcher cấp
token O1 qua kiểm phiên chỉ đọc, chặn thời hạn theo phiên nguồn và thu hồi khi
logout. Các lựa chọn chỉ tồn tại trong bộ nhớ và hết hiệu lực cùng token.

Phạm vi là dịch vụ Rust và Web Shop (`design`), xác minh bằng kiểm thử Rust.
Các đường wire hiện hành giữ nguyên; chưa nối handler tiêu hao hoặc projection
mỹ phẩm KR khi chưa có capture tương ứng.
