# KR server operations

Chạy một instance KR 00967 trên mỗi thư mục state riêng. Các lệnh Linux chạy bằng tài khoản riêng của dịch vụ;
xem [gói Windows](#gói-server-windows) cho tài khoản và lệnh tương ứng.

## System requirements

- Linux x86-64 or AArch64 with a GNU/Linux ABI compatible with the package, Python 3.11+, and local disk for SQLite, logs, and snapshots.
- Build host: Rust via rustup (the repository pins the toolchain) and a C compiler and linker for the Linux target. Windows cross builds also need the x86-64 MinGW-w64 toolchain.
- Use absolute paths for `PACKAGE` (installed bundle) and `STATE` (short, private path outside the bundle). Keep state and backup directories at mode `0700`; keep the database, snapshots, and private keys at `0600`.

<a id="gói-linux-và-state-riêng"></a>
## Build and package

1. From the repository root, select `BUILD_TARGET`: `x86_64-unknown-linux-gnu` or `aarch64-unknown-linux-gnu`. Build with the lockfile.

   ```bash
   (cd rust && rustup target add "$BUILD_TARGET" &&
     cargo build --release --locked --target "$BUILD_TARGET" \
       -p kr-service -p kr-tunnel --bin kr-service --bin kr-admin --bin kr-companion --bin kr-load --bin kr-launch-identity)
   BINARY_DIR="$PWD/rust/target/$BUILD_TARGET/release"
   install -d -m 0700 local/operator
   ```

2. Chuẩn bị cấu hình và mẫu wire theo [bước 1–4](#first-install), rồi build gói. Ghi `manifest_sha256` từ builder
   vào bàn giao riêng. Khi chuyển gói, giữ mode file và dùng thư mục cài riêng cho mỗi phiên bản.

   ```bash
   python3 tools/rust_service_package.py --base "$SERVING_RECIPE" --binary-dir "$BINARY_DIR" \
     --target "$BUILD_TARGET" --out local/operator/server-new
   ```

   Builder chỉ lấy mẫu wire và cấu hình vận hành từ recipe; đường dẫn pin tương đối tính từ thư mục của recipe.
   Gói không chứa profile map, terrain, DB hay private key. Mặc định dùng sáu map G1. Dùng `--pack-scope f1` để chạy
   catalog mẫu CP382 ở phạm vi hai người, floor trống và tắt terrain. Trên Linux, dùng `--legacy-fixtures` để tái lập gói cũ khi rollback.

<a id="pack-f1-a"></a>
### Catalog thử F1-a

Trên Linux, build gói với `--pack-scope f1-a`, rồi thêm `--f1-map <tên>` khi chạy lệnh `export` hoặc `run`
cùng `--game-dir`. Lặp cờ cho 1–16 tên pack chính xác và giữ thứ tự khi dùng lại cache.
Catalog thử dùng hai client, terrain từ pack và floor theo [policy F1-a](protocol/items-stats.md#g1-scoped-drops).
Cache tách khỏi G1 và cấu hình thử CP382. Nếu một map không xuất được, chọn catalog khác; exporter không tự
thay bằng floor rỗng hoặc map khác. Giữ cấu hình G1 cho bản phục vụ đã nghiệm thu.

Để kiểm tự động một nhóm tối đa tám map qua TLS/relay, dùng:

```bash
python3 tools/test_kr_pack_install.py --binaries "$BINARY_DIR" --bwrap --scope f1-a \
  --game-dir "$GAME_DIR" --f1-map CaJJ_Clover01 --f1-map CaJJ_Clover02
```

Test dùng cấu hình, tài khoản và replay cadence tổng hợp trong namespace riêng. Nó kiểm đúng map, kết quả và
về phòng; vẫn cần client KR thật để kiểm hình, va chạm và nhặt item.

## Gói server Windows

Lõi dùng chung cho **Play with friends / Chơi với bạn bè** (O1) và gói tự dựng (S1). Dùng cùng recipe, mẫu wire
và exporter KR 00967 như gói Linux. Từ host build có MinGW-w64 x86-64:

```bash
(cd rust && rustup target add x86_64-pc-windows-gnu &&
  cargo build --release --locked --target x86_64-pc-windows-gnu \
    -p kr-service -p kr-tunnel --bin kr-service --bin kr-admin --bin kr-launch-identity)
mkdir -p local/operator
python3 tools/rust_service_package.py --base "$SERVING_RECIPE" \
  --binary-dir rust/target/x86_64-pc-windows-gnu/release \
  --target x86_64-pc-windows-gnu --out local/operator/server-windows
```

Trên Windows x64, chạy bằng tài khoản thường, với Python 3.11+ và ổ đĩa cục bộ hỗ trợ ACL. Dùng thư mục state
tuyệt đối nằm ngoài gói và game; thư mục cha đã tồn tại và do người vận hành tin cậy. Ví dụ PowerShell:

```powershell
python "$Package\kr-package.py" init-db --manifest-sha256 $ManifestSha256 --state "$State"
python "$Package\kr-package.py" export --manifest-sha256 $ManifestSha256 --state "$State" --game-dir "$GameDir"
python "$Package\kr-package.py" run --manifest-sha256 $ManifestSha256 --state "$State" --game-dir "$GameDir"
```

Runner tạo ACL riêng có quyền kế thừa cho tài khoản chạy và SYSTEM, rồi giữ khóa runner tới khi server thoát.
Server kiểm chủ sở hữu, ACL và reparse point cho state, DB và khóa TLS; không cấp quyền rộng thêm khi gặp state
không hợp lệ. Các tệp `launch.sock` và `accounts.admin.sock` trên Windows là thông tin kết nối riêng chứa bí mật
theo lượt chạy. Hai listener dùng TCP trên `127.0.0.1` và xác thực hai chiều; không chép các tệp này ra khỏi state.
Trên Linux vẫn dùng socket Unix, mode POSIX và SIGTERM như trước. Nếu Winsock thiếu tùy chọn UDP của Quinn,
hai endpoint QUIC Windows dùng socket từng datagram; khi đó cần địa chỉ bind cụ thể, ví dụ `127.0.0.1` cho
chạy cục bộ. Khi chạy dưới Wine, đặt `umask 077` trước lúc mở prefix riêng để file con có quyền riêng đúng
kỳ vọng của ACL; server từ chối file có quyền rộng.

Dừng bằng **Ctrl+C** trong console của runner. Bản PE cũng đăng ký **Ctrl+Break** trên Windows; dưới Wine chỉ xác nhận Ctrl+C trong console riêng.
Chờ exit 0 và `"stopped": true` trong `service-state.json` trước khi mở lại. Dùng `bin\kr-admin.exe` cho
`health`, `backup` và `verify-backup` như trên Linux. Đặt snapshot trong thư mục riêng có ACL; có thể tạo
thư mục bằng `kr-admin.exe private-dir <directory>`. Windows đồng bộ dữ liệu snapshot rồi công bố bằng thao
tác move có cờ write-through và không ghi đè.

Đây là gói vận hành lõi. Launcher một nút, phòng/mã mời, cài đặt không cần Python và nghiệm thu LAN/Internet
là các lát tiếp theo của O1/S1.

## First install

1. Chọn recipe wire đã pin dùng `profile_mode: database_zero`, bật managed player rounds và lựa chọn trong phòng.
   Giữ relay xác thực và đặt giới hạn peer, route, hàng đợi, rate và lease cho máy triển khai.

   ```bash
   SERVING_RECIPE="$PWD/local/operator/service.json"
   python3 tools/rust_phase1_config.py --base "$BASE_CONFIG" \
     --relay-bind "$RELAY_BIND" --relay-advertise "$RELAY_ADVERTISE" \
     --database "$STATE/accounts.sqlite" --launch-socket "$STATE/launch.sock" --out "$SERVING_RECIPE"
   ```

   Khi dời recipe, cập nhật đường dẫn mẫu wire tương đối. Runner đặt DB và socket theo `--state`.
   Đổi thiết lập vận hành trên recipe trước khi build manifest mới.

2. Bật `prelogin_tls` và, nếu cần, `public_launcher` trong `diagnostic_two_lobby.account_admission`.
   `init-db` tự tạo chứng chỉ và private key cho từng listener được bật trong state riêng, tại
   `prelogin_tls-identity/` và `public_launcher-identity/`. Chỉ đưa `certificate.der` vào gói người chơi; giữ private key
   trên máy chủ. Nếu phục hồi DB vào state mới, chạy `init-tls` để cấp danh tính mới. Để tiếp tục dùng danh tính hiện có,
   build với `--external-tls`, giữ pin công khai trong recipe và cài private key tương ứng vào
   `$STATE/prelogin.pk8` hoặc `$STATE/launcher.pk8` với mode `0600`.

3. For Internet access, set `network_scope: controlled_internet` and add the **source IPv4 addresses seen by the server** to top-level `allowed_peers`, retaining local peers needed by the profile. Allow those sources only at both host and upstream firewalls:

   | Listener | Inbound |
   | --- | --- |
   | `public_launcher.bind` | TCP/TLS for the account API |
   | `prelogin_tls.bind` | UDP/QUIC for the prelogin companion |
   | `authenticated_relay.bind` | UDP/QUIC for peer relay |

   Bind those listeners to a local interface; set `authenticated_relay.advertise` to the IPv4:port the companion can reach. Allow established replies. Keep native KR ports on loopback at both ends and the admin socket local. One relay listener carries all logical routes (`n × (n − 1)` per full mesh room); no public port range is needed. Players behind one NAT share the prelogin `max_per_ip` limit.

4. For each enabled TLS listener, set `session_ttl_seconds` and `session_idle_seconds` if the package defaults (28,800 s absolute / 1,800 s idle) do not fit. Keep `1 ≤ idle ≤ TTL ≤ 86400`. Room and match activity refresh idle; keepalives do not. The absolute TTL starts at login and is separate from QUIC timeouts.

5. Ở máy cài, đặt `PACKAGE` và `MANIFEST_SHA256` theo bàn giao; `GAME_DIR` trỏ tới thư mục game KR 00967.
   `STATE` là đường dẫn tuyệt đối ngắn nằm ngoài cả gói và thư mục game.

   ```bash
   python3 "$PACKAGE/kr-package.py" verify --manifest-sha256 "$MANIFEST_SHA256"
   python3 "$PACKAGE/kr-package.py" init-db --manifest-sha256 "$MANIFEST_SHA256" --state "$STATE"
   python3 "$PACKAGE/kr-package.py" export --manifest-sha256 "$MANIFEST_SHA256" --state "$STATE" --game-dir "$GAME_DIR"
   ```

   Export mở chỉ đọc ba cặp `FxMap`, `FxContents` và `fx` (`.idx`/`.idd`) và kiểm SHA-256 đầy đủ của bản gốc.
   Nếu thiếu hoặc không khớp, lấy lại đúng pack gốc 00967. Nếu cache báo sai hash, dừng service, bỏ riêng thư mục cache
   bị lỗi dưới `$STATE/pack-cache` rồi chạy lại export. Cache dùng chung cho hai preset, được nhận dạng theo hash
   pack và phiên bản exporter. Runner sẽ tự export khi chạy nếu cache chưa có. Giữ `init-db` cho DB mới; DB cũ dùng
   quy trình restore hoặc nâng cấp bên dưới.

## Start, stop, and restart

1. Run in the foreground under a process supervisor or in a dedicated terminal. The runner takes the state lock and replaces itself with the service.

   ```bash
   python3 "$PACKAGE/kr-package.py" run --manifest-sha256 "$MANIFEST_SHA256" --state "$STATE" --game-dir "$GAME_DIR"
   ```

   Với gói cũ từ `--legacy-fixtures`, bỏ `--game-dir`.

2. Identify the current start's `$STATE/runs/<id>/service` directory as `RUN_DIR`. Wait for health to pass before admitting players.
3. Schedule a maintenance window, stop new admissions at the perimeter, and let current rounds finish. Stop via the supervisor or send `kill -TERM "$SERVICE_PID"` to the current process. Wait for exit 0, `"stopped": true` in `service-state.json`, and listeners to release. Unfinished rounds are recorded as aborted.
4. Restart with the same `run` command after the old process exits. Use the new run directory and ask players to relaunch and log in again. After a forced stop, confirm the PID is gone first; boot revokes old sessions and aborts any journal entries still marked running.

<a id="server-health"></a>
## Health and monitoring

```bash
ADMIN="$PACKAGE/bin/kr-admin"
"$ADMIN" health "$RUN_DIR"
cat "$RUN_DIR/health.json"
```

- Alert on process exit or a failing health check. The CLI returns JSON and exits 0 when the service is ready and the healthy sample is at most 15 seconds old. Compare with the supervisor's process status: after SIGKILL, the last sample can still pass until it ages out.
- Read `health.json` locally; the service replaces it atomically every five seconds and emits `health_sample` in `rust-events.jsonl`. Watch lobby players, room phases and rounds, relay companions and reserved routes, and database probe errors. `lobby.online_players` counts players attached to the managed lobby.
- Track host CPU, RAM, disk space and file descriptors. Monitor backup age and `backup_failed` events.

<a id="m8-udp-workload"></a>
For load monitoring, measure relay latency and loss at the sender and receiver; `submitted_to_quic` is a handoff count. See the [offline UDP profiler](../tools/m8_udp_profile.py) for workload inputs.

For a mixed load test, set `delivery_buckets: true` in the private `kr-load`
config. Each `delivery` can then report `receive_buckets` with
`first_second` and consecutive one-second counts. Bucket indices share a
monotonic origin across rooms in the same driver invocation. Count only whole
buckets contained in the confirmed common active interval. The round totals
still include partial seconds and the final drain window.
`delivery_clock_sample` brackets each wall clock reading with driver offsets
near the start and end. Check clock alignment before comparing with observations
from another process or host; the samples alone do not establish that alignment.

<a id="m8-soak-collector"></a>
## Collect load and soak metrics

Run the [collector](../tools/m8_soak_collect.py) on the same Linux host after readiness, with access to the service's `/proc` entry and run directory. Use a new private output file. Align the driver with the sample windows: start load after 60 seconds, sustain it for 12 hours, then stop the driver and leave the service up for 60 seconds of recovery.

    python3 tools/m8_soak_collect.py --run "$RUN_DIR" --out "$PRIVATE_LOG/new-soak.jsonl" \
      --interval 5 --baseline 60 --load 43200 --recovery 60

Check the `finished` row and samples from all three phases. Compare CPU per core, RSS, FD, rooms, companions, routes and DB errors against the planned limits and driver logs. The collector does not generate traffic or measure end-to-end delivery.

<a id="server-admin-backup-rollback"></a>
## Accounts, rooms, and audit

Run `"$ADMIN" "$DB"` as the service user and send one JSON request on stdin with an `actor`, a short `reason`, and a `command`. OS access controls who can run admin operations. Keep credentials and recovery codes out of arguments, history, audit reasons, and logs.

```bash
DB="$STATE/accounts.sqlite"
"$ADMIN" "$DB" < "$PRIVATE_REQUEST" # e.g. create; keep the JSON file at mode 0600
printf '%s\n' '{"actor":"on-call","reason":"Check current rooms","command":{"op":"list-rooms"}}' | "$ADMIN" "$DB"
printf '%s\n' '{"actor":"on-call","reason":"Check room closures","command":{"op":"audit-list","filter":{"operation":"close_room"}}}' |
  "$ADMIN" "$DB"
```

| `command.op` | Fields and use |
| --- | --- |
| `create`, `lookup` | Create with `login`, `name`, `password`; look up with `login`. For secrets, feed JSON from a protected file or a prompt. |
| `kick`, `disable`, `enable` | Use `account_id`. Kick revokes current sessions; disable also blocks login; enable allows login again. |
| `password-reset`, `recovery-reset`, `delete` | Use `account_id`; password reset also takes `password` and invalidates the recovery code. Recovery reset returns a new code once on stdout. Delete clears credentials and profile but keeps the ID for history. These operations revoke sessions. |
| `list-rooms`, `close-room` | List current rooms. To close one, copy its entire object into `command.target` (`instance`, `epoch`, `room_id`, `session`, `generation`). This removes the room and aborts an unfinished round. Refresh the list for a stale target. |
| `audit-list` | Optional `command.filter`: `actor`, `operation` (e.g. `close_room`), `target_kind`, `target_id` (string), `result`, `since_utc`, `before_id`, `limit` (1–100, default 50). Pass the last row's ID as `before_id` for the next page. |

If a close-room reply is lost, check the audit and live room list before retrying. Close starts as `pending` and records `ok`, `rejected`, or `journal_unconfirmed`; a pending record left at boot becomes `interrupted`.

## Backup and restore

1. For scheduled snapshots, add `"backup": {"interval_seconds": 3600, "keep": 24}` under `diagnostic_two_lobby.account_admission`. Rebuild the package. The first snapshot runs after one interval; retention applies only to scheduled snapshots in `accounts.backups/`.
2. For a manual snapshot, set `SNAPSHOT` to a new file in a private backup directory. This is safe during service: SQLite snapshots committed data, including WAL changes. Keep verified snapshots in approved private storage; never use a raw copy of the live database.

   ```bash
   "$ADMIN" backup "$DB" "$SNAPSHOT"
   "$ADMIN" verify-backup "$SNAPSHOT"
   ```

3. Stop the old instance before switching over. Restore into a new absolute `NEW_STATE` with no destination database or SQLite sidecars. Nếu dùng chứng chỉ do gói tạo, chạy `init-tls --state "$NEW_STATE"` sau khi restore; hoặc chuyển danh tính TLS hiện dùng vào state mới. Với pin ngoài gói, cấp đúng private key. Khi chạy, dùng state mới và cùng `--game-dir`.

   ```bash
   python3 "$PACKAGE/kr-package.py" restore-db --manifest-sha256 "$MANIFEST_SHA256" \
     --state "$NEW_STATE" --snapshot "$SNAPSHOT"
   ```

   Check health, a fresh login and a test round before reopening access. Restore does not resume live sessions or rounds. Keep the old state for recovery or diagnosis.

## Upgrade, migrate, and roll back

1. Verify the new package and retain the old package, manifest hash, recipe and TLS identities. Stop the service cleanly, then take and verify a fresh manual snapshot.
2. Run migration explicitly with the new package before starting it. On a schema change the migrator verifies a snapshot named `accounts.sqlite.before-v<old>-to-v<new>-*.sqlite` before applying changes in a transaction. If migration fails, stop here and keep the logs and database.

   ```bash
   python3 "$NEW_PACKAGE/kr-package.py" migrate-db --manifest-sha256 "$NEW_MANIFEST_SHA256" --state "$STATE"
   ```

3. Provision the keys that match the new package, start it, and check health, a fresh login and a test round. If verification fails, stop it and provision the old keys for rollback.
4. On an unchanged, compatible schema, start the old package against the same state. After a schema increase, use the new package to restore the verified pre-upgrade snapshot into fresh `ROLLBACK_STATE`, then run the old package. There is no in-place downgrade.

   ```bash
   python3 "$NEW_PACKAGE/kr-package.py" restore-db --manifest-sha256 "$NEW_MANIFEST_SHA256" \
     --state "$ROLLBACK_STATE" --snapshot "$PRE_UPGRADE_SNAPSHOT"
   # Provision keys matching the old package before starting.
   python3 "$OLD_PACKAGE/kr-package.py" run --manifest-sha256 "$OLD_MANIFEST_SHA256" --state "$ROLLBACK_STATE" # thêm --game-dir nếu gói cũ xuất từ pack
   ```

   Repeat the health, login and round checks. Keep the upgraded state; post-snapshot changes are absent from the rollback database.

<a id="launcher-tls-đang-triển-khai"></a>
## Launcher and player package

1. Prepare a private player recipe with original KR 00967 `base`, optional `pack_stages` in KR → vi → en order, and all three `languages` (`vi`, `en`, `ko`). Supply the matching 949x LoginStage executables, data, fonts, video, `index_stage`, and image status. Pin file inputs by path, bytes and SHA-256. To serve localized account names to 949x-patched clients, set `patched_949x_names: true` in `account_admission`.
2. For Windows, prepare the private runtime recipe: reachable IPv4:port values for `server` and `prelogin_server`, native loopback `ports`, and pins for `launcher`, `companion`, `certificate` and `prelogin_certificate`.

   ```bash
   (cd rust && rustup target add x86_64-pc-windows-gnu &&
     cargo build --release --locked --target x86_64-pc-windows-gnu \
       -p kr-player -p kr-tunnel --bin kr-player --bin kr-prelogin)
   python3 tools/player_package.py --recipe "$PLAYER_RECIPE" \
     --windows-runtime "$WINDOWS_RECIPE" --out local/operator/player-windows-new
   ```

   Omit `--windows-runtime` for payload only. For the Linux launcher with Wine, supply a native `kr-prelogin` companion and set `wine` and `udp_backend: native` in `launcher.json`. The launcher uses a Wine prefix per language. Windows uses `udp_backend: portable`.

3. Deliver the bundle and manifest hash privately. On Windows, extract to writable NTFS and open `kr-player.exe` (optional shortcut script: `Tao-loi-tat.ps1`). Chọn ngôn ngữ, chọn một máy trong mục Máy chủ, bấm **Chơi**, rồi đăng nhập hoặc chơi thử ở màn đăng nhập của game. The launcher verifies the installation and waits for the companion; starting `CA.exe` directly skips the tunnel.
4. For scripted payload verification, build the `kr-player-install` binary and use a new installation directory:

   ```bash
   (cd rust && cargo build --release --locked -p kr-player --bin kr-player-install)
   rust/target/release/kr-player-install install "$PAYLOAD" "$PLAYER_MANIFEST_SHA256" "$NEW_INSTALLATION"
   rust/target/release/kr-player-install verify "$PAYLOAD" "$PLAYER_MANIFEST_SHA256" "$NEW_INSTALLATION"
   ```

<a id="launcher-art"></a>
### Ảnh nền tùy chọn của launcher

Khi dựng gói, `tools/player_package.py` lấy ảnh từ `local/assets/launcher/launch.png` theo mặc định.
Dùng `--launcher-art ĐƯỜNG_DẪN` để chỉ định ảnh khác. Khi có ảnh nguồn, máy dựng gói cần Python Pillow
(CI dùng `Pillow==13.0.0`).
Tool giữ tỉ lệ khi thu ảnh vào khung 960×720, ghi JPEG chất lượng 92 vào `payload/launcher/backdrop.jpg`,
và pin kích thước byte cùng SHA-256 vào trường `launcher_art` của `manifest.json`.
Khi phân phối gói có ảnh, dùng launcher dựng cùng phiên bản và giữ tệp JPEG tại vị trí trong payload.
Ảnh nguồn và JPEG xuất ra đều ở `local/` trên máy dựng gói.

Launcher phủ kín cửa sổ khi đổi kích thước, cắt ưu tiên mép trên và trái để giữ hàng logo phía dưới.
Cửa sổ mở ở khoảng 1100×820, tối thiểu khoảng 880×660; màn hình thấp dùng khung nhỏ hơn cùng tỉ lệ.
Launcher nhớ ngôn ngữ và chế độ. Mục Máy chủ hiện máy trong cấu hình gói; đặt `server_name` nếu cần tên riêng.
Nút thêm máy chờ S1, chế độ Chơi với bạn bè chờ O1. Enter mở game khi máy đã chọn sẵn sàng; Esc đóng hộp thoại.
Tạo, đổi, xoá và khôi phục tài khoản dùng TLS của máy đã chọn. ID dùng thành công trong các form launcher được
nhớ riêng theo máy; mật khẩu và mã khôi phục không được lưu. LoginStage trong game vẫn xử lý đăng nhập.
Số ms là thời gian của yêu cầu trạng thái qua TLS. `release_url` là link HTTPS Google Drive tùy chọn trong recipe
Windows hoặc `launcher.json`; để trống thì ẩn **Tải bản mới**. Bấm link mở trình duyệt, launcher không tải hay cập nhật tự động.

**Hướng dẫn** mở trực tiếp trong launcher, có mục lục và cuộn theo từng phần. Nút **?** mở So sánh hai chế độ;
**Cách bắt đầu** mở phần của tab đang chọn, còn liên kết ở lỗi mở phần xử lý tương ứng. Gói kèm cả ba tệp trong
`player/`; launcher nhúng cùng nguồn [vi](player/guide.vi.md), [en](player/guide.en.md), [ko](player/guide.ko.md)
lúc build và đọc được khi không có mạng.

Trong Cài đặt có đường dẫn thư mục game, đường mở log, nút kiểm tra và sửa tệp, và nút tạo lối tắt trên Windows.
Sửa tệp chỉ thay thư mục cài có biên nhận khớp gói, sau khi dựng và xác minh bản thay thế. Bản cài trước được
giữ cạnh thư mục game trong `.kr-player-*/previous`. Không sửa khi một launcher đang sở hữu game trong cùng
thư mục. Nếu thư mục không có biên nhận phù hợp hoặc bản dựng lỗi, dùng thư mục cài mới; không xoá bản cũ trước khi lấy log.
Launcher dùng nền gradient nếu manifest cũ không có ảnh hoặc ảnh bị thiếu, hỏng hay sai pin.
Ảnh là phần trang trí; lỗi ảnh không chặn bước kiểm và cài dữ liệu game.

## Troubleshooting

| Symptom | First checks |
| --- | --- |
| Startup fails or health stays red | Check the current process exit status, stderr and `rust-events.jsonl`. Check manifest, state ownership, socket path length, DB schema and port use (`ss -lntu`). Do not remove locks or sockets to bypass an active instance. |
| Login or relay is unreachable | Match both certificate pins to deployed keys. Check observed source IPv4, both firewalls, TCP API and UDP prelogin/relay reachability, relay advertisement, and shared NAT limits. If an address changes, update the allowlist and firewall, rebuild, and verify the new manifest. |
| Session ends during play | Check for timeout, kick, password reset, disable, or restart. Reopen the game through the launcher and sign in again. |
| Launcher cannot verify, install, or start the game | Check manifest and writable storage. Use a fresh installation directory for an incomplete install. Check free loopback ports and the latest log under launcher state `prelogin-logs/`. |
| DB probe or backup fails | Check `database_probe` (`busy`, `schema`, `epoch_mismatch`, `unavailable`), disk space, permissions and service logs. Verify a fresh snapshot after fixing the cause. If corruption is suspected, preserve the DB and sidecars and restore a verified snapshot into new state. |
