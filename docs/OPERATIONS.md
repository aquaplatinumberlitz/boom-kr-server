# KR server operations

Run one KR 00967 service instance per private state directory. Run the commands below as a dedicated Linux service user unless a build or firewall step calls for a different account.

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
       -p kr-service -p kr-tunnel --bin kr-service --bin kr-admin --bin kr-companion --bin kr-launch-identity)
   BINARY_DIR="$PWD/rust/target/$BUILD_TARGET/release"
   install -d -m 0700 local/operator
   ```

2. Complete [first install steps 1–4](#first-install), then package the serving recipe. Keep the builder's `manifest_sha256` in the handoff record, separate from the bundle. Transfer the verified bundle privately, preserving file modes; use a new installation directory for each version.

   ```bash
   python3 tools/rust_service_package.py --base "$SERVING_RECIPE" --binary-dir "$BINARY_DIR" \
     --target "$BUILD_TARGET" --out local/operator/server-new
   ```

   The builder packages pinned fixtures and profiles from the recipe. Relative fixture paths are resolved from the recipe's directory. The bundle contains no database, live sessions, or private TLS keys.

## First install

1. Start with a pinned DB-backed managed-room recipe (`profile_mode: database_zero`, managed player rounds and choices enabled). Keep its supported maps, roster and fixtures. Select authenticated relay and set peer, route, queue, rate and lease budgets for the planned deployment.

   ```bash
   SERVING_RECIPE="$PWD/local/operator/service.json"
   python3 tools/rust_phase1_config.py --base "$BASE_CONFIG" \
     --relay-bind "$RELAY_BIND" --relay-advertise "$RELAY_ADVERTISE" \
     --database "$STATE/accounts.sqlite" --launch-socket "$STATE/launch.sock" --out "$SERVING_RECIPE"
   ```

   Rebase any relative fixture paths when moving the recipe. The package runner sets DB and launch socket paths from `--state`. Make configuration changes in the recipe before building a new manifest.

2. Provision TLS for `public_launcher` and `prelogin_tls` under `diagnostic_two_lobby.account_admission`. For a new identity, run the helper once per distinct certificate with a new destination under a private directory:

   ```bash
   "$BINARY_DIR/kr-launch-identity" "$NEW_TLS_DIR"
   ```

   Pin each `certificate.der` in the recipe with `path`, `bytes` and `sha256`; distribute the matching public pins in the trusted player package. Provision the two matching `private.pk8` files separately. Never put private keys in either bundle.

3. For Internet access, set `network_scope: controlled_internet` and add the **source IPv4 addresses seen by the server** to top-level `allowed_peers`, retaining local peers needed by the profile. Allow those sources only at both host and upstream firewalls:

   | Listener | Inbound |
   | --- | --- |
   | `public_launcher.bind` | TCP/TLS for the account API |
   | `prelogin_tls.bind` | UDP/QUIC for the prelogin companion |
   | `authenticated_relay.bind` | UDP/QUIC for peer relay |

   Bind those listeners to a local interface; set `authenticated_relay.advertise` to the IPv4:port the companion can reach. Allow established replies. Keep native KR ports on loopback at both ends and the admin socket local. One relay listener carries all logical routes (`n × (n − 1)` per full mesh room); no public port range is needed. Players behind one NAT share the prelogin `max_per_ip` limit.

4. For each enabled TLS listener, set `session_ttl_seconds` and `session_idle_seconds` if the package defaults (28,800 s absolute / 1,800 s idle) do not fit. Keep `1 ≤ idle ≤ TTL ≤ 86400`. Room and match activity refresh idle; keepalives do not. The absolute TTL starts at login and is separate from QUIC timeouts.

5. Build the package, then set `PACKAGE` and `MANIFEST_SHA256` from the handoff. Initialize a new database once; for an existing deployment use the restore or upgrade steps.

   ```bash
   python3 "$PACKAGE/kr-package.py" verify --manifest-sha256 "$MANIFEST_SHA256"
   install -d -m 0700 "$STATE"
   install -m 0600 "$LAUNCHER_KEY" "$STATE/launcher.pk8"
   install -m 0600 "$PRELOGIN_KEY" "$STATE/prelogin.pk8"
   python3 "$PACKAGE/kr-package.py" init-db --manifest-sha256 "$MANIFEST_SHA256" --state "$STATE"
   ```

## Start, stop, and restart

1. Run in the foreground under a process supervisor or in a dedicated terminal. The runner takes the state lock and replaces itself with the service.

   ```bash
   python3 "$PACKAGE/kr-package.py" run --manifest-sha256 "$MANIFEST_SHA256" --state "$STATE"
   ```

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

3. Stop the old instance before switching over. Restore into a new absolute `NEW_STATE` with no destination database or SQLite sidecars. Provision the keys matching the package pins, then start with `--state "$NEW_STATE"`.

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
   python3 "$OLD_PACKAGE/kr-package.py" run --manifest-sha256 "$OLD_MANIFEST_SHA256" --state "$ROLLBACK_STATE"
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
Nút thêm máy chờ S1, chế độ offline chờ O1. Enter mở game khi máy đã chọn sẵn sàng; Esc đóng hộp thoại.
Tạo, đổi, xoá và khôi phục tài khoản dùng TLS của máy đã chọn. ID dùng thành công trong các form launcher được
nhớ riêng theo máy; mật khẩu và mã khôi phục không được lưu. LoginStage trong game vẫn xử lý đăng nhập.
Số ms là thời gian của yêu cầu trạng thái qua TLS. `release_url` là link HTTPS Google Drive tùy chọn trong recipe
Windows hoặc `launcher.json`; để trống thì ẩn **Tải bản mới**. Bấm link mở trình duyệt, launcher không tải hay cập nhật tự động.

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
