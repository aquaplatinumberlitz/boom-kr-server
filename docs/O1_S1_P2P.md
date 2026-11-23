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
một lần nhận ở đường prelogin cục bộ để mở Link đến relay loopback. Server giữ kiểm slot, route và epoch
hiện có; không nhét packet QUIC bên trong datagram iroh. Khi host dừng, mọi kết nối theo phòng bị đóng.
Gateway O1 hiện nhận đúng một đề nghị mở relay cho mỗi người kết nối. Nút chơi thử 2P trên cùng máy tạo
hai đề nghị, nên gateway kết thúc phiên trước khi vào sảnh. Đây là giới hạn riêng của Phòng riêng.

Mã `S1-1-<base64url>` chứa 32 byte node ID và SHA-256 của chứng thư TLS DER. Dán mã S1 vào ô mã phòng
sẽ chuyển sang tab Server; dán mã O1 vào ô mã mời Server sẽ chuyển sang Phòng riêng. Khi thêm Server,
launcher mở phiên S1, kiểm khóa nút và hash chứng thư rồi mới lưu tên và mã mời. Mỗi lần chọn lại Server,
launcher dùng mã đã lưu để tạo cầu loopback cho API tài khoản và companion prelogin. Dữ liệu tài khoản
thuộc Server; thao tác xóa trong launcher chỉ bỏ mã mời và gợi ý ID đã lưu trên máy người chơi.
Companion hiện chưa chuyển địa chỉ relay được gửi sau đăng nhập sang cầu UDP S1. Vì vậy phạm vi
launcher S1 ở đây là thêm, chọn, quản lý tài khoản và mở prelogin; chưa nghiệm thu chơi trận qua S1.

Kiểm transport: `cargo test --manifest-path rust/Cargo.toml --locked -p kr-p2p -p kr-tunnel`. Lab prototype
dùng `cargo build --manifest-path rust/Cargo.toml --locked -p kr-p2p --features lab --example udp_lab` rồi
`tools/o1_p2p_namespace.sh` với `direct` hoặc `relay`, binary đã build và thư mục output rỗng.
Tiêu chí nghiệm thu nằm trong [PLAN](PLAN.md#full-game-roadmap).
