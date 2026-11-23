# Transport O1 và S1

O1 dùng iroh/QUIC. Launcher của người tạo giữ khóa nút tạm cho mỗi phòng và chạy server Rust trên
loopback. Companion của mỗi người mở cổng TCP/UDP loopback cho client KR. DNS-SD trên mDNS tìm phòng
LAN; đường LAN hoạt động khi không có Internet. Thuộc tính TXT tùy chọn `host` mang tên nhân vật để
launcher hiện tên cạnh mã phòng. Đây là nhãn hiển thị; kiểm khóa nút và admission vẫn giữ nguyên.
Khi gói có dịch vụ tra mã, O1 dùng relay công cộng
Number 0 để tìm đường và iroh có thể chuyển sang UDP trực tiếp. Người chơi không cấu hình port forwarding.

Mã phòng có dạng `O1-1-K7M2-4QX8-7RDT-25PH`. Phần sau tiền tố là 80 bit đầu của SHA-256 có miền
riêng trên khóa công khai của phòng. Launcher kiểm mã theo khóa nút trước khi kết nối; iroh chứng minh
quyền giữ khóa riêng. Người tạo nhận peer qua một stream admission có mã phòng, SHA-256 của manifest gói
người chơi và tên khách. Mã là quyền vào phòng; người cùng LAN có thể xem mã qua discovery. Không gửi mã
công khai nếu chỉ muốn chơi với bạn bè. Mã khác phiên bản hoặc manifest khác bị từ chối trước LoginStage.

Tra mã Internet là ứng viên phát hành: launcher đăng bản ghi Nostr hết hạn chứa khóa nút, relay và hạn dùng,
mã hóa ChaCha20-Poly1305 bằng khóa HKDF từ mã phòng. Chỉ mục dùng miền SHA-256 khác. Bản ghi không chứa IP
LAN hay tên khách. Peer trực tiếp thấy IP của nhau; nhà vận hành relay/mailbox có thể thấy IP phía kết nối và
metadata. Cần kiểm relay/mailbox công cộng giữ và phục vụ bản ghi theo hạn trước khi phát hành.

Launcher phía tạo giữ gateway. Gateway mở prelogin QUIC tới server cục bộ bằng chứng thư DER do gói tạo
và giữ pin trong máy người tạo. Chứng thư này không cần phát cho người vào vì phía họ xác thực đầy đủ khóa
nút iroh qua mã. Gateway chỉ cho phép các selector cổng cố định. Iroh stream mang luồng native và control
của companion; iroh DATAGRAM mang `Packet` game, gồm route ID, sequence và UDP KR. Gateway dùng capability
một lần nhận ở đường prelogin cục bộ để mở Link đến relay loopback. Gateway chấp nhận một hoặc hai đề
nghị loopback. Khi chọn 2P trên cùng máy, mỗi người có một stream control riêng; gateway và companion
phân luồng datagram theo thứ tự đề nghị. Framing 1P giữ nguyên. Server vẫn kiểm slot, route và epoch;
chỉ số luồng không cấp quyền người chơi. Khi host dừng, mọi kết nối theo phòng bị đóng.
Hai slot trên cùng một client dùng chung khung chat dưới tên P1. Dùng hai client để chat dưới hai tên riêng.
Ở Phòng riêng, người chơi vẫn thấy khung ID và mật khẩu của `CA.exe` nhưng có thể để trống.
Ẩn khung riêng cho chế độ này là việc tiếp theo.

Mã `S1-1-<base64url>` chứa 32 byte node ID và SHA-256 của chứng thư TLS DER. Dán mã S1 vào ô mã phòng
sẽ chuyển sang tab Server; dán mã O1 vào ô mã mời Server sẽ chuyển sang Phòng riêng. Khi thêm Server,
launcher mở phiên S1, kiểm khóa nút và hash chứng thư rồi mới lưu tên và mã mời. Mỗi lần chọn lại Server,
launcher dùng mã đã lưu để tạo cầu loopback cho API tài khoản và companion prelogin. Dữ liệu tài khoản
thuộc Server; thao tác xóa trong launcher chỉ bỏ mã mời và gợi ý ID đã lưu trên máy người chơi.
Companion mở Link gameplay qua cổng relay loopback của chính phiên S1. Mỗi Link vẫn dùng chứng thư và
capability từ đề nghị sau đăng nhập. Cầu giữ cùng một socket nguồn cho cả hai Link nếu server gửi hai đề
nghị. Bài kiểm Rust dùng relay iroh riêng, tắt đường IP trực tiếp giữa hai nút và xác nhận UDP đi hai chiều
trên cả hai Link. Phép kiểm này không thay nghiệm thu một ván native qua launcher S1.

Kiểm transport: `cargo test --manifest-path rust/Cargo.toml --locked -p kr-p2p -p kr-tunnel`. Lab prototype
dùng `cargo build --manifest-path rust/Cargo.toml --locked -p kr-p2p --features lab --example udp_lab` rồi
`tools/o1_p2p_namespace.sh` với `direct` hoặc `relay`, binary đã build và thư mục output rỗng.
Tiêu chí nghiệm thu nằm trong [PLAN](PLAN.md#full-game-roadmap).
