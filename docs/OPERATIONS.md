# Vận hành server KR: cấu hình relay

Đây là phần cấu hình mạng của tài liệu vận hành. Đóng gói, cấp tài khoản từ xa, giám sát, chạy dài, backup/restore và
nâng cấp/rollback còn theo [acceptance M8](PLAN.md#phase1-acceptance); không coi trang này là nghiệm thu vận hành.
Contract xác thực, route và giới hạn ở [UDP/companion](protocol/udp-peer.md#rust-authenticated-serving-relay).

## Gói Linux và state riêng

[Builder](../tools/rust_service_package.py) nhận binary cùng kiến trúc ELF64 và recipe đã pin, gom fixture/profile
thành đường dẫn tương đối và ghi manifest. Nó không copy DB/session, không tự đổi endpoint/budget và không tự xuất
lại toàn bộ dữ liệu từ pack. Bước này chưa thay acceptance build-from-pack, Linux sạch hoặc gói người chơi của M8.
Đầu ra chứa cấu hình hạ tầng và dữ liệu client, chỉ dựng trong `local/`, không commit hoặc phát hành lên GitHub.

```bash
python3 tools/rust_service_package.py --base "$BASE_CONFIG" --binary-dir "$BINARY_DIR" \
  --target "$BUILD_TARGET" --out local/operator/package-new
```

Giữ `manifest_sha256` từ kết quả builder làm hash bàn giao độc lập với gói. Chuyển thư mục nguyên mode sang đường
cài mới; target và Linux ABI phải tương thích, Python dùng standard library. State thuộc người chạy, quyền `0700`,
ở ngoài thư mục gói; socket cần đường dẫn ngắn. Runner kiểm manifest, file/mode và từ chối symlink/path escape.

```bash
python3 "$PACKAGE/kr-package.py" verify --manifest-sha256 "$EXPECTED_MANIFEST_SHA256"
python3 "$PACKAGE/kr-package.py" init-db --manifest-sha256 "$EXPECTED_MANIFEST_SHA256" --state "$PRIVATE_STATE"
python3 "$PACKAGE/kr-package.py" run --manifest-sha256 "$EXPECTED_MANIFEST_SHA256" --state "$PRIVATE_STATE"
```

`init-db` chỉ dùng cho DB mới; restore có quy trình riêng, không ghi đè DB bằng init. `run` re-root DB/socket vào
state, khóa instance của runner rồi exec service với cwd gói để fixture không cần repo/local. Admin dùng JSON qua
stdin của `bin/kr-admin`, không đặt mật khẩu/token trên argv. Kiểm namespace bằng
[smoke tool](../tools/smoke_rust_service_package.py) chỉ chứng minh repo bị che trên cùng kernel/thư viện host;
không gọi nó là máy Linux sạch, Internet/native client, restore, tải hoặc soak.

[Bằng chứng nền đóng gói](evidence/rust-service/m8-server-package.json) có kiểm x86 và ARM trên hai Linux độc lập.
Smoke cần `unshare`, `ip`, `mount`, `bwrap` và namespace được kernel cho phép; mỗi lượt chỉ mở loopback trong
network namespace riêng. Trên máy ARM, lượt dùng user namespace với tài khoản thường vướng quyền đọc
`/proc/self/mountinfo` của `bwrap`; bản smoke đạt dùng thư mục riêng do root sở hữu và `sudo`. Đây là giới hạn
của harness, chưa là quy trình chạy production bằng tài khoản service. Giữ raw lỗi trước, không thay cấu hình bảo
mật host để vượt lỗi. Smoke kiểm DB mới, từ chối init lại mà không đổi DB, từ chối runner thứ hai khi instance đầu
còn ready, rồi dừng có trật tự. Không suy client đăng nhập hoặc Internet từ readiness.

## Launcher TLS đang triển khai

GUI mới ở `kr-player` theo [chỉ đạo vi/en/ko](evidence/owner-directives/m8-multilang-launcher.json); bản ko phải là
exe đã vá 949x. [Builder người chơi](../tools/player_package.py) nhận recipe local có `base` là client KR gốc,
`pack_stages` theo thứ tự vi rồi en, và ba phần `languages`. Mỗi phần khai `encoding: 949x`, `data`, `fonts`,
`executable` và `video` được pin path/bytes/sha256, `index_stage` là base/vi/en và trạng thái `images`. Không lấy
idx en đã dựng riêng trên pack KR rồi ghép vào sau vi; builder từ chối khi prefix không khớp. Khi chưa có ảnh en,
phải khai rõ ảnh tạm và giữ giới hạn chưa đạt, không ghi thành gói cuối.

```bash
python3 tools/player_package.py --recipe "$PRIVATE_PLAYER_RECIPE" --out local/operator/player-payload-new
```

Giữ hash manifest độc lập với payload. Bộ cài Rust ráp pack, kiểm hash từng phần và toàn pack, dùng hard link
cho pack readonly/file chung, rồi copy exe/chữ/font/video/idx vào ba thư mục. Chỉ bản cài đủ hash mới có receipt;
thư mục cài dở hoặc có sẵn bị từ chối ghi đè. Có thể kiểm cùng installer không GUI:

```bash
kr-player-install install "$PAYLOAD" "$PLAYER_MANIFEST_SHA256" "$NEW_INSTALLATION"
kr-player-install verify "$PAYLOAD" "$PLAYER_MANIFEST_SHA256" "$NEW_INSTALLATION"
```

`kr-player "$GUI_CONFIG"` nhận config local gồm `server` (TLS endpoint), `certificate`, `launcher` (kr-launch),
`companion`, `bundle` (payload), `manifest_sha256`, `installation`, `state`, `port_budget`; Wine thêm `wine` là
path executable Wine. Path tương đối tính từ config. GUI kiểm/cài payload, nhớ ngôn ngữ trong state và hỏi status
qua pin TLS mỗi năm giây. Status chỉ là endpoint TLS và admission còn mở, không chứng nhận DB/trận khỏe.
Trước mỗi launch, GUI kiểm lại file của ngôn ngữ đã chọn; password chỉ truyền stdin cho owner `kr-launch`.
Đóng GUI hoặc nút dừng tạo cancel file riêng, owner dừng/join hai child rồi logout; không dùng kill để giả logout.
GUI cập nhật heartbeat file riêng mỗi nửa giây; nếu GUI chết hoặc heartbeat lỗi/không đổi quá 10 giây, owner
chạy cùng cleanup/logout. Mỗi launch có thư mục riêng nên heartbeat/cancel cũ không tác động launch kế.
Cấu hình GUI/CLI có `udp_backend` tùy chọn: `native` mặc định, `portable` dùng UDP một datagram qua Tokio,
bỏ ECN/GSO/ancillary cho adapter Windows đang gặp10040. Chọn rõ trong config riêng; không tự retry/fallback
sau lỗi xác thực. `peer_diagnostics` trên CLI chỉ xuất counters/mã OS đã lọc, không endpoint/token.
Wine có prefix riêng từng ngôn ngữ dưới state. Cài máy sạch, prefix hoàn chỉnh và vi/ko cùng phòng đọc chữ có dấu
vẫn cần native acceptance M8; ảnh GUI hay kiểm hash không thay ca này.

Server cho ba client đã vá có thể khai `account_admission.patched_949x_names: true` cùng `profile_mode: database_zero`.
DB giữ tên UTF-8, adapter đổi đúng 949x khi phát profile và kiểm channel; giới hạn là 3–15 byte trên wire.
Đây là opt-in theo deployment, không chọn encoding từ ngôn ngữ launcher hoặc IP. Để false/mặc định cho client chưa vá:
tên ngoài ASCII bị từ chối cấp frontend. Không tự sửa DB/đổi tên khi chuyển policy. [Contract biên tên](protocol/auth-session.md#rust-patched-account-names)
chưa có native acceptance cho tên Việt/Hàn; chat và tên phòng có ca riêng.

Giao diện [public launcher](protocol/auth-session.md#rust-public-launcher) opt-in của M8 hiện có kiểm loopback,
chưa là gói Internet/Wine/Windows đã nghiệm thu. Tạo identity bằng `kr-launch-identity "$NEW_PRIVATE_DIRECTORY"`;
command từ chối directory có sẵn. Giữ `private.pk8` trên server trong parent `0700`, file `0600`; phát certificate
qua kênh/gói tin cậy. Không tự tải pin từ cùng endpoint chưa xác thực hoặc ghi private key vào bundle. Khi đóng
gói config có launcher, builder gom certificate đã pin và đặt key thành `@STATE@/launcher.pk8`; operator cấp key
khớp riêng vào state trước chạy service. DB và key không được đổi/copy từ raw thí nghiệm đang dùng.

Player config khai endpoint TLS, certificate path, native IP, companion path, native program/arguments/directory
và port budget; toàn bộ config triển khai giữ local. `kr-launch "$PLAYER_CONFIG"` đọc JSON login/password qua stdin,
không qua argv; dùng wrapper nhập mật khẩu riêng trước khi bàn giao người chơi. Client mở companion bằng stdin,
đợi transport ready rồi mở native với frontend token hiện hành. Kết thúc/lỗi/signal dọn hai child rồi thử logout;
server không tới được thì báo lỗi và giữ TTL, không giả logout đã xác nhận. Gói Wine/Windows và Internet native
còn theo acceptance M8; không dùng `/bin/true` trong smoke làm bằng chứng chơi trận.

Harness Windows hiện dùng [owner](../tools/native_windows_candidate.py), [input QMP tham số hóa](../tools/native_windows_input.py)
và [installer](../tools/native_windows_install.ps1). Ca đa ngôn ngữ dùng [controller tham số hóa](../tools/native_windows_multilang.py)
và [guest script](../tools/native_windows_multilang.ps1), nhận vi/en/ko, manifest/hash và executable control đã khai.
Nó kiểm input đã cài trong runtime folder riêng, chụp trạng thái có hạn, dừng owner và xuất evidence; không coi
thư mục runtime này là ca cài sạch hoặc gói cuối. Lượt có `--socket-probe` là đối chứng debugger riêng.
Nếu nguồn Internet đổi, staging lấy đúng IPv4 quan sát từ SSH để thay `/32` ở kế hoạch rule và allowlist gói,
pin lại config/manifest rồi verify trước chạy; không mở rộng ACL để né lỗi. Mọi cấu hình, media, credential test, overlay và raw chỉ ở local;
owner giữ lease chung, VM/egress riêng và chờ bản cài kiểm hash xong mới bật runner VPS có hạn và rule test riêng.
Chọn executable theo hash đã đo, không mặc định tên `CA.exe` trong cây nhiều probe; bản copy từ ISO phải bỏ
ReadOnly trước khi ghi config/cache. Các lỗi chuẩn bị và sửa tương ứng ở metadata nền launcher, chưa là nghiệm thu KR.

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
