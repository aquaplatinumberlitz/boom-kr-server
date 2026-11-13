# Vận hành server KR: cấu hình relay

Đây là phần cấu hình mạng của tài liệu vận hành. Đóng gói, cấp tài khoản từ xa, giám sát, chạy dài, backup/restore và
nâng cấp/rollback còn theo [acceptance M8](PLAN.md#phase1-acceptance); không coi trang này là nghiệm thu vận hành.
Contract xác thực, route và giới hạn ở [UDP/companion](protocol/udp-peer.md#rust-authenticated-serving-relay).

## Chọn cấu hình phục vụ

Giai đoạn 1 dùng **`AuthenticatedRelay`**. `LegacyDirect` và relay UDP prototype giữ làm cấu hình chẩn đoán riêng.
Binary yêu cầu `--config`, không tự chọn một địa chỉ hạ tầng hoặc biến config chẩn đoán thành config phục vụ.
Tạo preset bằng [tool cấu hình](../tools/rust_phase1_config.py), từ baseline account DB và managed player round
đã đo. Tool giữ nguyên map, item, sức chứa và budgets; nó chọn relay xác thực, bỏ pool prototype và yêu cầu chỉ rõ
DB/socket riêng. Fixture dùng khi đóng gói phải theo manifest hash; việc loại phụ thuộc `local/` khỏi gói thuộc M8.

Thông tin endpoint, đường truy cập, DB và cấu hình triển khai thực chỉ giữ trong `local/`. Tạo thư mục riêng, chọn
DB/socket dành cho vận hành; không chạy lại trực tiếp config trỏ vào DB/raw đã seal của một phép thử.

```bash
python3 tools/rust_phase1_config.py --base "$BASE_CONFIG" \
  --relay-bind "$RELAY_BIND" --relay-advertise "$RELAY_ADVERTISE" \
  --database "$ACCOUNT_DATABASE" --launch-socket "$PRIVATE_LAUNCH_SOCKET" \
  --out local/operator/service.json
kr-service --config local/operator/service.json --out local/operator/run
```

Tool tạo file mới mode `0600`, không ghi đè và từ chối đường ra ngoài `local/`. Chuẩn bị quyền riêng của thư mục,
DB và launcher theo contract account; SQLite từ bản sao phải giữ quyền `0600`. Mỗi lần service mở instance mới
phải đăng nhập mới qua launcher; không tái dùng capability, pin hoặc session trước restart như một cơ chế resume.
Chỉ công bố readiness sau khi TCP, launcher và UDP listener đã bind thành công. Đóng có trật tự và xác nhận port
đã nhả trước khi khởi động mới; Quinn có drain bất đồng bộ hữu hạn, không giả định `close()` đồng bộ nhả socket.

## Cổng và firewall

| Tài nguyên | Công thức / cấu hình cho tám phòng tám người |
|---|---|
| Listener QUIC của server | Một cổng UDP được cấu hình; tái dùng giữa các phòng/vòng và sau restart |
| Route logic | `n × (n − 1)` mỗi phòng; tối đa 448 cho tám phòng tám người |
| Peer QUIC | Một connection mỗi companion; tối đa 64 khi mọi slot đều đầy |
| Proxy tại một client | Bảy socket loopback mỗi snapshot tám người, ngoài socket UDP native và QUIC |
| TCP game/bootstrap | Các cổng trong config đã chọn; độc lập số route logic |
| Launcher hiện hành | Unix socket riêng; giao diện public/TLS và gói người chơi thuộc M8 |

Firewall chỉ cho inbound tới UDP listener và TCP game/bootstrap đã chọn, theo allowlist/phạm vi triển khai.
Cho phép reply của flow hiện hành; không mở dải cổng proxy loopback hoặc dải 448 cổng public. Bind/advertise là hai
vai trò riêng: địa chỉ listener cục bộ phải bind được, địa chỉ quảng bá phải tới được từ companion. Certificate pin
và capability được phát qua launcher xác thực, không qua packet native hoặc argv.

Con số trên là ngân sách cấu trúc, chưa là benchmark. Không lấy preset hai route của phép thử làm cấu hình 50 người.
M8 phải chọn headroom, queues/rate/lease và đo CPU/RAM/độ trễ/rơi gói theo tải đích. Proxy endpoint cũ giữ riêng theo
session với budget hữu hạn; cạn budget dừng đóng, cần launcher mới. Ghi nhận và xử lý admission/expiry/revoke/DB fault
thay vì tự rebind identity hoặc mở quyền cho gói đầu không xác thực. Chi tiết firewall/topology thực không commit.
