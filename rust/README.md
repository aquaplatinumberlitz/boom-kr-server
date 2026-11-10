# Rust foundation KR00967

CP236 thêm backend account/DB qua [CLI](#account-admin-cli), chưa gắn vào native login adapter.
[Contract/giới hạn](../docs/PROTOCOL.md#rust-account-backend); accountDB độc lập gameplay và không đóng toàn M5.

Đợt implementation được mở bởi [owner09/11](../docs/evidence/owner-directives/rust-implementation.json).
[PLAN](../docs/PLAN.md#current-order) giữ thứ tự; [contract/limits](../docs/PROTOCOL.md#rust-foundation)
và [evidence](../docs/evidence/rust-service/README.md#cp231) giữ kết luận có nguồn.

`kr-wire` không có dependency: profile mode2/selector3, header37 giữ opaque, crypt/CRC, typed unsigned fields,
framing tối đa64KiB và seq theo connection. Giới hạn64KiB là design; không phải maximum của client gốc.
`kr-service` là binary Tokio/Linux có profile **hello → login error → retry** (M1) và opt-in
**fixed local identity1/KRLocal → channel → sảnh → profile291** (M2partial). Chưa accountauthentication,
gameplay. [Phòng/lifecycle/barrier CP234](../docs/PROTOCOL.md#rust-fixed-room) đã có nativepositive; M2 đã nghiệm thu scoped tại [CP235](../docs/PROTOCOL.md#rust-m2-acceptance). Opt-in `diagnostic_two_lobby` đã đo hai client KR thật tới sảnh với ID/tên riêng. Hai transport đầu dùng mapping WCP của recipe CP181;
mọi login ở error-only profile nhận lỗi1. Local-lobby profile chỉ projector đã khai, không kiểm credential. Không dùng arrival order làm thiết kế identity production.

Build/kiểm từ root repo:

```bash
cargo fmt --manifest-path rust/Cargo.toml --all --check
cargo clippy --manifest-path rust/Cargo.toml --workspace --all-targets --locked -- -D warnings
cargo test --manifest-path rust/Cargo.toml --workspace --locked
cargo build --manifest-path rust/Cargo.toml --workspace --locked
```

Rust1.99.0 và lockfile được pin. Tokio cho I/O, semaphore/queue/timeout/signal; serde/JSON cho config và
structured logs; SHA-256 kiểm input fixture trước bind. Không import server/clientengine từ boom-rs/VN.
Binary không kèm client/asset/binary fixture. Từ fixture Python đã giữ cục bộ, export **path/size/hash**, không sinh wire:

```bash
python3 tools/export_rust_profile.py --python-profile local/profile.json \
  --login-error local/login-error1-key43.bin --port 3838 --port 3848 --out local/rust-profile.json
rust/target/debug/kr-service --config local/rust-profile.json --out local/new-rust-run
```

Recipe native cô lập ở evidence dùng port3808..3998/step10 theo CP181; config chỉ cho loopback/allowlist.
Giữ source/binary/config/input frozen từ trước run tới seal; không hot-swap backend. Rollback chọn Python
baseline đã pin trước run mới sau cleanup. Python vẫn serving/RE/reference; không rút khỏi đường game hiện tại.

Giới hạn: admission tối đa64connection, acceptqueue32, logqueue256, buffer64KiB/socket, write/read/log timeout,
shutdown SIGTERM/SIGINT hữu hạn; không unbounded per-packet tasks. Ghi opcode/seq/size, không raw credential/token
hoặc hash auth body. Request60/body20 và15/body31 sau rejection chỉ được ignore có budget64 theo traceCP181,
không khẳng định ý nghĩa. Malformed/CRC/unknownstage/duplicate login không thay seq hoặc tạo success giả.
Chưa load/SLO/DB/UDP/Internet; **Internet thật chưa kiểm**.

Opt-in local lobby (profile/body inputs đã giữ trong evidence; không kèm game asset):

```bash
python3 tools/export_rust_profile.py --python-profile local/profile.json   --login-error local/login-error1-key43.bin --local-lobby-own-profile local/own-profile-A-body.bin   --port 3838 --port 3848 --port 3858 --out local/rust-lobby.json
rust/target/debug/kr-service --config local/rust-lobby.json --out local/new-lobby-run
```

[Nativeproof/giới hạn CP232](../docs/PROTOCOL.md#rust-local-lobby): auth137 và291128/nativepersonalrender/UIKRLocal
trên một actualWineclient. `lobby::Flow` kiểm stage/shape/identity trướcadvance,2negative statetests;
14kernelTCP M1regression vẫn pass. Nhu cầu hai actual/phòng/barrier/Back-rejoin giữ ở PLAN, M2 chưa hoàn tất.
First2WCP/3login/4channel arrivalmapping chỉ lab single-client; không mở cho generalaccounts/Internet.
Profilezero/name/level/EXP là diagnosticpolicy, không durable account/profile hoặc production.

Hai-client diagnostic (CP233):

```bash
python3 tools/export_rust_profile.py --python-profile local/profile.json \
  --login-error local/login-error1-key43.bin --two-lobby-own-profile-a local/own-A.bin \
  --two-lobby-own-profile-b local/own-B.bin --two-lobby-redirect local/redirect7-private.bin \
  --diagnostic-bind 10.77.0.1 --diagnostic-peer-a 10.77.0.2 --diagnostic-peer-b 10.77.0.3 \
  --port 3838 --port 3848 --port 3858 --out local/two-lobby.json
rust/target/debug/kr-service --config local/two-lobby.json --out local/new-two-lobby-run
python3 tools/test_rust_lobby.py --binary rust/target/debug/kr-service \
  --config local/two-lobby.json --out local/new-binding-tcp-check
```

Redirect7 phải là frame đã pin cho endpoint đã chọn; exporter chỉ pin fixture hiện có, không suy fieldunknown.
Recipe native lắng nghe candidate3808..3998/step10. WCP có thể dùng login/channelport: type0/body28 sauCRC
chọnWCP; type1/type6 cộng endpoint+IPoperator+redirectlease chọnlogin/channel, socketrỗng khôngclaim.
[Contract/limits](../docs/PROTOCOL.md#rust-two-lobby), [raw/nativeproof](../docs/evidence/rust-service/README.md#cp233).
Hai source IP chỉ diagnosticbinding, chưa accountauthentication. 10Rusttests/16bindingTCP/14M1regression táchnativeproof;
phòng/Ready/barrier/Back-rejoin giữ thiếu ở PLAN; chưa fullM2. Python serving/RE/reference còn giữ.

Phòng cố định opt-in (CP234): thêm `--fixed-room-python-profile local/profile.json`,
`--fixed-room-listing local/room91-one.bin`, `--fixed-room-title RustRoom` vào exporter two-lobby.
Đây là titleinitial/fixturepin; actualcreate81 echo titlebytes bounded vào listing91 vàselector0/1 vào86/87.
`login_candidate_ports` liệt kê cổnglogin đã chấpnhận (native có3838/3848); `channel_port` vẫn exactredirect3858.
Không source/config/input change giữa run. [Proof/limits](../docs/PROTOCOL.md#rust-fixed-room),
[raw](../docs/evidence/rust-service/README.md#cp234); recordM2 chưa complete.

```bash
python3 tools/test_rust_room.py --binary rust/target/debug/kr-service \
  --config local/fixed-room-config.json --out local/new-room-tcp-check
```

Roommodel/queue/wire adapters ở `kr-service/src/room.rs`, `room_io.rs`, `room_wire.rs`;14Rusttests và8kernelTCPcases.
SyntheticTCPinputs khácnativeevidence. Queue8/pending16/visit-roundsubjectfences khôngraw-oldACKproof.
127/190sauArmed chỉobserve/noeffect; khôngBattle. SauownedcleanupA, Bfinalreport có thể bịdeny; activepeerabort chưađạt.
Python vẫnserving/RE/reference; audit kếCP236.

M2 đã nghiệm thu scoped tại [CP235](../docs/PROTOCOL.md#rust-m2-acceptance);
[checklist/giới hạn](../docs/evidence/rust-service/m2-acceptance.json), [run EOF](../docs/evidence/rust-service/README.md#cp235).
Sau member EOF ở Loading/Armed, chỉ current survivor thuộc captured roster được drop thêm64 report99/127/190,
không GO/effect/forward/result; member mới và Start mới không kế thừa. Guard exact bootstrap/WCP profile trướcbind
không là native wrong-version UI. Sprite A còn sau rosterclear; fullgame/result/return thuộc M3.

```bash
python3 tools/test_rust_profiles.py --binary rust/target/debug/kr-service \
  --config local/fixed-room-config.json --out local/new-profile-check
python3 tools/test_rust_shutdown.py --binary rust/target/debug/kr-service \
  --config local/fixed-room-config.json --out local/new-shutdown-check --iterations 12
```

Các lệnh trên kiểm config/kernelTCP, không thay native KR evidence. Shutdown tool giữ PID/thread/wchan khi deadline
lỗi; timeout cũ chưa biết nguyên nhân. Python serving/RE/reference còn giữ; PLAN tiếp M3, audit kếCP236.


<a id="account-admin-cli"></a>
## Account/DB CLI nội bộ — CP236

Binary `rust/target/debug/kr-admin <đường-dẫn-DB>`; parent đã tồn tại mode0700 dưới đường dẫn admin tin cậy,
DB mới mode0600. Không dùng thư mục repo/publicstorage. `cargo build --workspace --locked` tạo cả hai binary;
`cargo run` mặc định vẫn `kr-service`. Backend sync, chưa được gọi trong Tokio room/network handler.

Mỗi lần chạy nhận **một JSON object từ stdin**, tối đa4096bytes; không đưa password/token vào argv/shellhistory/log.
Dùng trusted local terminal/helper có secretinput riêng; login trả token một lần ởstdout, coi output là secret.
Format (chỉ mô tả field, không là credential thật):

```json
{"actor":"local-admin","reason":"internal provisioning","command":{"op":"init"}}
```

Mọi request có `actor`, `reason`, `command`. Trường không khai báo bị reject; lỗi chỉ mã enum ởstderr/exit1.
`actor/reason` là lời khai adminlocal, không RBAC; không ghi secret trong reason. Account/user không truy cập CLI trực tiếp.

| command.op | Field bổ sung trong command | Kết quả |
|---|---|---|
| init | không | schema1; mở lại không revoke |
| begin-instance | không | epoch mới + revoke toàn session cũ + audit; chỉ gọi khi bắt đầu instance mới |
| create | login, name, password | account/player/wireID bền vững |
| lookup | login | player và profileversion; admin-only |
| login | login, password, ttl_seconds | ownplayer, expires_at, secret token; chưa KRlogin |
| authorize | token, player_id | ownplayer hoặc InvalidSession |
| rename | account_id, name, expected_version | CAS profileversion tăng; không đổi player/wireID |
| revoke | account_id | revoke sessions/audit |
| disable / enable | account_id | ban/unban/audit; enable không revive token cũ |
| password-reset | account_id, password | credentialversion tăng/revoke/audit |

Policy: login3..32 ASCIIalnum/underscore/lowercaseunique; name3..15 cùng alphabet; password12..128bytes.
AbsoluteTTL1..86400s; chưa idleTTL, authnetworkratelimit, restore, clientbridge hoặc multiplayerauth.
Không tự downgrade/repairDB hoặc fallback unauthenticated khi lỗi. Clockrollback failclosed, cần xử lý clock chủ động.
`begin-instance` manual chưa là exclusive gameinstance lock; không chạy hai service owner cùng store.

```bash
python3 tools/test_rust_accounts.py --binary rust/target/debug/kr-admin --out local/new-account-check
```

33process CLI/realSQLite là backendacceptance; 11Rustaccounttests thêm concurrency/expiry/clock/reset-race/fault.
Giữ failures và raw/hash/private synthetic token outputs theo evidence; không dùng teststore cho production.
Dynamic KR account/identity projection cần measured authbridge và native positive/negative trước expose.
