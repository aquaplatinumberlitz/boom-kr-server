# Vận hành server KR

Runbook này dùng cho một instance Linux. Nghiệm thu gói người chơi, tải và chạy dài theo [acceptance M8](PLAN.md#phase1-acceptance).
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

<a id="server-admin-backup-rollback"></a>
## Quản trị, snapshot và đổi phiên bản

Chạy bằng tài khoản hệ điều hành quản lý service. Tách thư mục gói (đã kiểm manifest) khỏi state riêng `0700`.
Giữ gói cũ, hash manifest, cấu hình và cặp chứng thư/khóa TLS tương ứng. Khóa TLS nằm ngoài DB; cấp lại đúng khóa
khớp pin từ kho bí mật khi phục hồi. Cài lần đầu bằng `init-db`, phục hồi thì dùng `restore-db`.

`kr-admin DATABASE` đọc một JSON từ stdin. Khai `actor` là tên operator, `reason` là lý do ngắn; quyền truy cập do
hệ điều hành bảo vệ, còn actor là khai báo của operator. Không đưa mật khẩu, mã recovery hoặc token vào
argv, lý do, shell history hay log.

| `command.op` | Trường và hành vi |
|---|---|
| `create`, `lookup` | Tạo bằng `login`, `name`, `password`; tra cứu bằng `login`. |
| `disable`, `enable`, `kick` | Dùng `account_id`; khóa thu hồi phiên, mở khóa cho đăng nhập lại, kick chỉ thu hồi phiên (alias `revoke`). |
| `password-reset`, `recovery-reset` | Dùng `account_id`; reset mật khẩu cần thêm `password` và hủy mã recovery hiện có. Reset recovery phát mã mới qua stdout một lần. Cả hai thu hồi phiên. |
| `delete` | Dùng `account_id`; xóa login cũ, thông tin hồ sơ, password/recovery đang dùng và thu hồi phiên. Giữ ID đã vô hiệu để bảo toàn lịch sử trận/audit. |
| `list-rooms`, `close-room` | Liệt kê phòng hiện hành; truyền nguyên object đã chọn vào `target` gồm `instance`, `epoch`, `room_id`, `session`, `generation`. Close loại phòng khỏi danh sách, ngắt quyền phòng và hủy ván chưa kết thúc. Yêu cầu từ tiến trình cũ bị từ chối, kể cả khi restore dùng lại số epoch. |
| `audit-list` | `filter` tùy chọn: `actor`, `operation`, `target_kind`, `target_id` (chuỗi), `result`, `since_utc`, `before_id`, `limit` (1–100, mặc định 50). Sắp từ ID mới nhất; dùng ID cuối làm `before_id` cho trang kế. |

    printf '%s\n' '{"actor":"on-call","reason":"tra thao tác đóng phòng","command":{"op":"audit-list","filter":{"operation":"close_room","limit":20}}}' |
      "$PACKAGE/bin/kr-admin" "$PRIVATE_STATE/accounts.sqlite"

Audit account thành công commit cùng thay đổi, lỗi có mã riêng khi DB ghi được. Đóng phòng ghi `pending` trước
khi tác động, rồi ghi `ok`, `rejected` hoặc `journal_unconfirmed`; boot kế chuyển yêu cầu còn treo thành `interrupted`.
Nếu mất phản hồi, tra audit cùng khóa phòng trước khi thử lại. Khi DB không ghi được audit, sửa lỗi lưu trữ trước;
không coi lỗi CLI là bằng chứng thao tác đã chạy. Đóng phòng không yêu cầu kick toàn bộ tài khoản.

Đặt `"backup": {"interval_seconds": 3600, "keep": 24}` trong `diagnostic_two_lobby.account_admission` của recipe
rồi dựng và kiểm lại manifest. Lượt đầu chạy sau một chu kỳ; chỉ một snapshot tự động chạy một lúc.
Service ghi `backup_complete` hoặc `backup_failed` vào `rust-events.jsonl` và giữ N snapshot tự động mới nhất trong
`accounts.backups/`. Chỉ dọn sau khi snapshot mới đã qua kiểm; file thủ công và bản trước migration nằm ngoài
lịch dọn. Theo dõi dung lượng, tuổi bản cuối và sao chép bản snapshot đã kiểm sang nơi lưu riêng được phép.
Snapshot có dữ liệu tài khoản nhạy cảm; giữ quyền `0600`, thư mục `0700` và không đưa vào gói.

    "$PACKAGE/bin/kr-admin" backup "$PRIVATE_STATE/accounts.sqlite" "$PRIVATE_BACKUP/new.sqlite"
    "$PACKAGE/bin/kr-admin" verify-backup "$PRIVATE_BACKUP/new.sqlite"
    python3 "$PACKAGE/kr-package.py" restore-db --manifest-sha256 "$EXPECTED_MANIFEST_SHA256" \
      --state "$NEW_PRIVATE_STATE" --snapshot "$PRIVATE_BACKUP/new.sqlite"

Có thể backup khi service chạy. SQLite `VACUUM INTO` chỉ chụp dữ liệu đã commit, kể cả WAL; kiểm cả schema,
`integrity_check` và khóa ngoại trước khi công bố file. Không copy file DB live bằng `cp`. Restore yêu cầu DB đích
chưa tồn tại và không có sidecar; dùng state mới. Cài khóa TLS, khởi động với gói tương thích, kiểm readiness,
đăng nhập mới và vào một ván. Restore không tiếp tục phiên hay trận trong bộ nhớ; các ván chạy dở trong snapshot
được ghi hủy ở lần boot sau. Đo RPO từ thời điểm snapshot và RTO tới lúc có thể mở ván trên môi trường triển khai.

Nâng cấp: ngừng nhận người chơi, gửi SIGTERM (hoặc dừng bằng service manager), chờ exit 0 và `stopped: true`.
Trận đã hoàn tất giữ kết quả; trận dở được ghi hủy, không cấp thưởng. Nếu phải SIGKILL, xác nhận PID cũ đã hết rồi
khởi động lại; boot thu hồi phiên, dọn socket chỉ khi không còn listener, và hủy journal còn `running`.
Nếu không thấy readiness, kiểm exit status, `rust-events.jsonl`, quyền state, cổng bị chiếm và DB; giữ log trước khi
thử lại. Không xóa lock/socket để vượt lỗi chạy trùng.

1. Kiểm hash gói mới và cấu hình. Sau khi dừng bản cũ, tạo thêm snapshot thủ công và kiểm. Có thể chạy
   `kr-package.py migrate-db --manifest-sha256 … --state …` dưới gói mới trước khi chạy `run`. Migrator tự giữ bản
   `accounts.sqlite.before-v<old>-to-v<new>-*.sqlite` đã kiểm trước khi áp dụng thay đổi trong một transaction.
   Nếu snapshot không tạo hoặc không kiểm được, migration dừng.
2. Kiểm `ready: true`, đăng nhập mới và mở ván. Nếu thất bại, dừng bản mới và giữ DB, cấu hình và log của lần lỗi
   để đối chiếu. Nếu schema không đổi, dùng lại gói/cấu hình cũ tương thích. Nếu schema đã tăng, dùng bản trước
   migration để restore vào state mới và chạy gói cũ với cấu hình/khóa TLS tương ứng.
3. Migration `0001`–`0006` không có chiều ngược tại chỗ. Cụ thể binary schema 5 từ chối DB đã qua
   `0006_operator_audit` (schema 6). Dùng binary mới hơn để **restore**, sau đó dùng binary cũ để **run**;
   không chạy `migrate-db` hoặc `run` của gói mới lên state rollback. Dữ liệu phát sinh sau snapshot không có
   trong DB rollback; giữ state mới để xử lý riêng.

Nếu `backup_failed` lặp lại, kiểm dung lượng và quyền, tạo/kiểm snapshot thủ công rồi theo dõi lượt kế. Khi
nghi DB hỏng, giữ nguyên DB và sidecar để điều tra; phục hồi bản đã kiểm vào state khác. Luyện rollback bằng
[script](../tools/test_m8_binary_rollback.py) với hai thư mục binary; tùy chọn `--package` và hash manifest kiểm
thêm boot/dừng hai server loopback. Test Rust mở ván sau restore qua hai client TLS tổng hợp, không chạy game.

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
cho pack và tệp dùng chung không cần ghi. Tệp cấu hình, cache và CALog có bản riêng theo ngôn ngữ. Bộ cài bỏ
thuộc tính chỉ đọc trong cây game của cả ba ngôn ngữ. Chỉ bản cài đủ hash mới có receipt; thư mục cài dở hoặc có
sẵn bị từ chối ghi đè. Có thể kiểm cùng installer không GUI:

```bash
kr-player-install install "$PAYLOAD" "$PLAYER_MANIFEST_SHA256" "$NEW_INSTALLATION"
kr-player-install verify "$PAYLOAD" "$PLAYER_MANIFEST_SHA256" "$NEW_INSTALLATION"
```

Gói Windows dùng hai binary x86-64 từ target `x86_64-pc-windows-gnu`:

```bash
cargo build --manifest-path rust/Cargo.toml --release --locked --target x86_64-pc-windows-gnu \
  -p kr-player -p kr-tunnel --bin kr-player --bin kr-prelogin
python3 tools/player_package.py --recipe "$PRIVATE_PLAYER_RECIPE" \
  --windows-runtime "$PRIVATE_WINDOWS_RECIPE" --out local/operator/player-windows-new
```

Recipe Windows khai hai endpoint IPv4 có cổng (`server` cho API tài khoản, `prelogin_server` cho QUIC),
`ports` là danh sách cổng native loopback, và bốn pin path/bytes/sha256: `launcher`, `companion`, `certificate`,
`prelogin_certificate`. Dùng các exe tương thích LoginStage manual đã được kiểm ở ngôn ngữ định phát hành.
Giải nén vào thư mục ghi được trên NTFS, mở `kr-player.exe` hoặc chạy `Tao-loi-tat.ps1` để tạo lối tắt.
Lối tắt chỉ mở launcher. Chọn ngôn ngữ, bấm **Vào game**, rồi đăng nhập hoặc chơi thử trong game.

`launcher.json` khai các đường dẫn tương đối với file cấu hình. GUI kiểm và cài payload, lưu ngôn ngữ
trong state và hỏi status qua TLS. Trước khi chơi nó kiểm lại bản đã chọn, thử mở ghi các tệp `.ssd`, `.ini` và
CALog. Tệp cấu hình hoặc cache đã đổi được khôi phục từ payload có hash; CALog đang có được giữ. Launcher báo
lỗi quyền ghi trước khi mở game; quy trình không sửa ACL. Nếu kiểm tra đạt, launcher đợi companion sẵn sàng rồi
mở game.
Nút dừng hoặc đóng cửa sổ dừng hai tiến trình. Companion tự đóng đường TLS nếu bộ đếm heartbeat của
launcher không tăng trong 10 giây. Companion viết log tối đa 32 KiB trong thư mục của lượt chạy; launcher giữ tối đa bốn log
đã kết thúc dưới `state/prelogin-logs`. Log ghi cổng loopback, mốc TLS, số byte chuyển và nhãn kết thúc;
launcher thêm mã thoát tiến trình. Log không chứa địa chỉ máy chủ, credential hoặc nội dung gói.
Gói Windows đặt `udp_backend: portable`. Với launcher Linux gọi Wine, đặt `udp_backend: native` cho companion Linux,
khai đường dẫn chương trình qua trường `wine` và dùng prefix riêng theo ngôn ngữ.
Hai đầu TLS cần certificate pin riêng (có thể dùng cùng chứng thư đã phân phối nếu hai đầu dùng cùng identity).

Server cho ba client đã vá có thể khai `account_admission.patched_949x_names: true` cùng `profile_mode: database_zero`.
DB giữ tên UTF-8, adapter đổi đúng 949x khi phát profile và kiểm channel; giới hạn là 3–15 byte trên wire.
Đây là opt-in theo deployment, không chọn encoding từ ngôn ngữ launcher hoặc IP. Để false/mặc định cho client chưa vá:
tên ngoài ASCII bị từ chối cấp frontend. Không tự sửa DB/đổi tên khi chuyển policy. [Contract biên tên](protocol/auth-session.md#rust-patched-account-names)
chưa có native acceptance cho tên Việt/Hàn; chat và tên phòng có ca riêng.

Trạng thái nghiệm thu của [public launcher](protocol/auth-session.md#rust-public-launcher) nằm ở
[Bảng điều hành M8](PLAN.md#current-order). Tạo identity bằng `kr-launch-identity "$NEW_PRIVATE_DIRECTORY"`;
command từ chối directory có sẵn. Giữ `private.pk8` trên server trong parent `0700`, file `0600`; phát certificate
qua kênh/gói tin cậy. Không tự tải pin từ cùng endpoint chưa xác thực hoặc ghi private key vào bundle. Khi đóng
gói config có launcher và prelogin, builder gom certificate đã pin. Operator cấp key khớp vào
`launcher.pk8` và `prelogin.pk8` trong state trước khi chạy service. DB và key không được đổi/copy từ raw thí nghiệm đang dùng.

Gói Linux sinh `config.template.json` với `session_ttl_seconds: 28800` (8 giờ) và
`session_idle_seconds: 1800` (30 phút) cho từng listener TLS đã bật nếu recipe không ghi giá trị.
Giữ `1 ≤ idle ≤ TTL ≤ 86400` giây. Hoạt động trong phòng và trận gia hạn idle; keepalive và
các lượt kiểm nền giữ nguyên hạn. TTL tuyệt đối vẫn tính từ lần đăng nhập. Với gói launcher hiện hành,
khi phiên tài khoản hết hiệu lực, mở lại game từ launcher và đăng nhập lại khi thấy thông báo.
Đổi mật khẩu hoặc thu hồi phiên cũng có thể kết thúc phiên đang chơi.

Đường dự phòng cho chẩn đoán dùng cấu hình riêng trong local. `kr-launch "$PLAYER_CONFIG"` đọc JSON
login/password qua stdin; không có trong gói Windows mới. Client mở companion bằng stdin,
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

## Nghiệm thu gói qua Internet và WAN giả lập

Tạo recipe triển khai trước khi dựng manifest. Giữ các cổng KR nội bộ ở loopback VPS và loopback của companion;
chỉ mở ba cổng từ IPv4 nguồn của lượt thử ở cả firewall máy và lớp mạng VPS: API launcher TCP/TLS, prelogin UDP/QUIC
và relay UDP/QUIC. Dùng địa chỉ giao diện riêng cho bind của ba listener và địa chỉ client tới được cho relay
advertise. Client nhận pin chứng thư và cổng API/prelogin qua payload; địa chỉ relay đến từ offer xác thực.

Đặt `network_scope: controlled_internet` và thêm **IPv4 mà VPS quan sát** vào `allowed_peers`, đồng thời giữ các
peer nội bộ của profile. API launcher và prelogin đều kiểm danh sách này; firewall mở và TCP kết nối được vẫn
chưa đủ. Hai client chung IPv4 dùng chung hạn mức `max_per_ip` của prelogin. Giữ hạn phiên mặc định 8 giờ và idle
30 phút; hai hạn này độc lập timeout im lặng của QUIC. Nếu địa chỉ đổi, cập nhật recipe và rule của lượt thử, tạo
gói mới và kiểm manifest lại. Không sửa trực tiếp template đã có pin.

Khởi tạo state riêng, cấp hai key và account thử nghiệm qua stdin riêng. Sau readiness, xác nhận launcher đã nhận
`status` qua TLS và đợi prelogin TLS rồi mới thao tác trong game; chỉ chạy thao tác trận khi đồng hồ trên cả hai
màn hình đang đếm. Ghi RTT tới VPS riêng với bằng chứng relay. Kết thúc lượt bằng tín hiệu dừng cho runner đã tạo,
kiểm listener đã nhả rồi gỡ các rule có nhãn của lượt đó ở cả hai lớp.

[Tool WAN giả lập](../tools/m8_wan_namespace.sh) chạy lệnh kiểm trong namespace mới với server và Wine ở
`M8_WAN_SERVER`, còn QEMU user networking phải được mở trong `M8_WAN_EDGE_NS` qua `nsenter --net`. Router chỉ
chuyển các cổng của gói; prelogin và relay dùng hai dải cổng nguồn NAT riêng. Mỗi chiều đường Windows thêm 75 ms
và mất 3% độc lập. Khi cả hai đồng hồ đã chạy, hạ `wan0` trong namespace edge. Lúc khôi phục, bật lại link và
đặt lại route `M8_WAN_SERVER/32` qua router của namespace; hạ link có thể xóa route này. Kiểm journal, phiên,
UI người còn lại và đăng nhập mới sau khi đường trở lại.

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
| Launcher hiện hành | Unix socket riêng; gói người chơi dùng thêm một listener TCP/TLS và một listener UDP/QUIC prelogin |

Với gói launcher qua Internet, áp dụng ba cổng và allowlist ở mục trên. Nếu chạy một chế độ chẩn đoán riêng,
chỉ cho inbound tới listener mà chế độ đó cần.
Cho phép reply của flow hiện hành; không mở dải cổng proxy loopback hoặc dải 448 cổng public. Bind/advertise là hai
vai trò riêng: địa chỉ listener cục bộ phải bind được, địa chỉ quảng bá phải tới được từ companion. Certificate pin
và capability được phát qua launcher xác thực, không qua packet native hoặc argv.

Con số trên là ngân sách cấu trúc, chưa là benchmark. Không lấy preset hai route của phép thử làm cấu hình 50 người.
M8 phải chọn headroom, queues/rate/lease và đo CPU/RAM/độ trễ/rơi gói theo tải đích. Proxy endpoint cũ giữ riêng theo
session với budget hữu hạn; cạn budget dừng đóng, cần launcher mới. Ghi nhận và xử lý admission/expiry/revoke/DB fault
thay vì tự rebind identity hoặc mở quyền cho gói đầu không xác thực. Chi tiết firewall/topology thực không commit.
