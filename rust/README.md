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


## M3 movement/floor trên native KR — CP237

[Contract/giới hạn](../docs/PROTOCOL.md#rust-movement-floor), [raw/hash](../docs/evidence/rust-service/README.md#cp237).
Cùng fixedroom profile đã pin: catalog đọc từ582list1,124currentmember/key/ctx/itemexact→165mộtlần/cảhai;
list2 cònhidden, khônginventory/reward. NativeADown/BRight/PCAP→key/apply/currentpeerprogress, floorRoller5→6
trên hai view; M3bom/terrain/life/result-reset/twooutcomes chưa có.
`123/10` chỉ opaqueobserve64/member/Armedround/noACK-effect-forward; semanticsunknown, không hỗtrợmọiopcode.

```bash
python3 tools/test_rust_room.py --binary rust/target/debug/kr-service --config local/native-fixed-room.json --out local/new-floor-policy-check --pickup-checks
```

22kernelTCPcases/30Rusttests là policyregression, tách nativeWineproof và copiedgetter/CRC checks.
Native recipe giữ source/binary/config frozen tới seal; không dùng syntheticinputs làm oracle. Shutdown timeout
cũ/CP237kernelrepeat vẫn chưa rõ rootcause, PID/thread/wchan được ghi khi tái hiện; không tăng deadline để pass.


## Haiownerbom/allocator — CP238

[Contract/giới hạn](../docs/PROTOCOL.md#rust-bombs), [raw/hash](../docs/evidence/rust-service/README.md#cp238).
Known11822byte/currentkey-boundowner/serial/cellbounds/capacityA1B2→15939/ID,6s→15813/kind0;
ticketroomscope-round-allocation-owner-recipient giữ xuyênqueue, EOF/roundchangefence/noIDreusequaStart.
Nativehaiowner/entitylookup/kind0/cleanup chỉscopehẹp, chưaterrain/hiddenreveal/life/result-reset/fullM3.
167/body4hoặc32 opaqueobservebudget64/currentArmedmember/noresponse/semanticsunknown;142knownshape chỉghi
rejectionnoreveal đến khi portterrain. KhôngplaceholderACK hoặcnormal-successcho unknownwire.

```bash
python3 tools/test_rust_room.py --binary rust/target/debug/kr-service --config local/native-fixed-room.json --out local/new-bomb-policy-check --pickup-checks --bomb-checks
```

40kernelTCPcases+32Rusttests tách nativeproof; toolpinhashbinarytrướcPopen vàso lạisau cleanup.
Đợibuildxong trướclaunch; freeze tấtcảinput/source/config/binary trongrun, khôngthaylivebinary.


## Boundedterrain/hiddenitems — CP239

[Contract](../docs/PROTOCOL.md#rust-terrain), [raw/hash](../docs/evidence/rust-service/README.md#cp239).
Optional`fixed_room.terrain` lấy từtrustedPython`bomb_terrain`: seed/map/startSHA,15×13/maxpower2 được kiểm trướcbind.
Geometrycandidate trướcIDcommit;fullroster158writecompletion mớicommitclearance;142đúngcurrentclearedhidden cell
mớirelease124/165. Hiddencatalog vẫn từcùng582. Nativeoneepoch/195flags/3effects đã đo, chưareset/life/results/fullM3.

```bash
python3 tools/test_rust_room.py --binary rust/target/debug/kr-service --config local/native-fixed-room.json --out local/new-terrain-check --pickup-checks --bomb-checks --terrain-profile local/python-profile.json
```

35Rusttests/43TCPcases tách nativeacceptance. Negative/model agreement khônglà originalserverauthority.
Không hot-swap backend giữa activeconnection;chờ source/config/binary/rawseal/cleanup trướcđổi input.

<a id="life-result-checks"></a>
## Life/result/reset có giới hạn — CP240

[Contract/limits](../docs/PROTOCOL.md#rust-life-results), [bằng chứng](../docs/evidence/rust-service/README.md#cp240).
`export_rust_profile.py` thêm optional `fixed_room.battle` khi Python profile khai đúng `death_lifecycle=current_trap`
và `result_freshness=post_blast_untrapped_drained`; bốn captured fixtures phải đúng hash/sequence/opcode/size trước bind.
Giữ profile diagnostic IP/identity; SQLite backend hiện chưa nối vào KR login.

Kiểm TCP policy (config/profile local đã pin, chọn output mới mỗi lần):
```bash
python3 tools/test_rust_life.py --binary rust/target/debug/kr-service --config CONFIG.json --battle-profile PROFILE.json --idle-seconds 20 --out LOCAL_NEW_DIR
```
Native Wine hai outcome/reset/Start3 đã có; kernel negatives/all-dead/peer21 không thay native proof.
M3 còn control item reset/contest, oldUDP/currentpositive, dead-owner/queue và hostEOF scene return theo PLAN.
Internet thật chưa kiểm.

## Active host EOF/forfeit — CP241

[Contract/limits](../docs/PROTOCOL.md#rust-return-cleanup), [raw/recipe](../docs/evidence/rust-service/README.md#cp241).
Battleprofile đã pin bật activeEOFforfeit12s→175→5s180→3s107/110; không active Battle-exit hoặc matchrestore.
`test_rust_life.py` thêm dead-owner/pendingbomb-freshsurvivor/EOFforfeit controls; `--idle-seconds 30` kiểm absoluteidle.
Các giới hạn native/synthetic và M3 còn thiếu theo PLAN. Internet thật chưa kiểm.


## CP242: M3 đạt scope basic hai client

[Acceptance](../docs/PROTOCOL.md#rust-m3-acceptance), [raw/hash](../docs/evidence/rust-service/README.md#cp242).
Hai epoch ba item/effect/reset, exact oldUDP native key-drop rồi current-positive, Start3/hold6s; 84raw/cleanup0.
44tests/51TCP; logger giữ sink rooterror, flush stall vẫn unknown/deadline unchanged. CP237–241 giữ native
movement/oppositeoutcomes/hostEOF. Không fullgame/auth/production; accountDB CP236 có, worker/idleTTL/rate/nativebridge còn riêng.
[PLAN](../docs/PLAN.md#current-order) giữ thứ tự; Internet thật chưa kiểm.


## Account worker nội bộ — CP243

`account_worker::Worker::open(db, capacity, queue_ttl).await` → `client()` →
`client.try_login(name, password, ttl, trusted_utc)?.wait().await`; authorize tương tự với token/ownplayer.
Typed seam, chưa KR socket/API. Gọi `shutdown().await` để đóng admission mọi clone/drain/join; Drop không là joined cleanup.
Queue1–64+one executing, queue/handoff expiry; `Pending::wait` phải claim result trước handoff deadline.
Result bỏ sau commit chỉ revoke token đó/audit; audit lỗi đóng worker/session có thể active, cần operator recovery.
[Contract/limits](../docs/PROTOCOL.md#rust-account-worker), [raw/hash](../docs/evidence/rust-service/README.md#cp243).
Kiểm với `python3 tools/test_rust_account_worker.py --out local/<new-private-dir>`:5worker cases/5retainedSQLiteDBs,
source/testbinary pinned/PIDgone. Đây là backenddesign, không nativeKRbridge/idleTTL/rate/fullM5.


## Schema2 idle TTL và login rate — CP244

[Contract/limits](../docs/PROTOCOL.md#rust-account-idle-rate). `Worker::open` dùng rate5/60s/1024principal;
`open_with_rate(..., RatePolicy)` đổi policy trong bounds. `try_login_with_idle(..., ttl, idle, utc)` bật idle cho phiên mới;
`try_login`/adminCLI giữ absolute-only. Chỉ correct ownplayer authorize gia hạn; chưa gắn KR socket.
Schema1 tự migrate transaction lên2, giữ mọi old rows/0001hash/oldidleNULL; CLIinit báo2. Không downgrade về binaryschema1.
Rate per-worker, sharedclones, không global/durable; fullqueue không charge, admittedcancel charge.
`test_rust_account_worker.py` giữ7schema2DBs; `test_rust_idle_migration.py --binary rust/target/debug/kr-admin
--baseline local/research/codex/rust-account-worker-acceptance-r1 --out local/<new-private-dir>` kiểm5copyDB cũ,
10CLIopens và full old-row conservation. Baseline CP243 không sửa; raw không publish.


## Bốn client startup — CP245

[Contract/giới hạn](../docs/PROTOCOL.md#rust-four-startup), [raw/hash](../docs/evidence/rust-service/README.md#cp245).
Với đủ các flags two-lobby/fixed-room ở trên, thêm:

```text
--four-startup-derived-dir local/new-four-inputs
--four-startup-peer-c 10.77.0.4 --four-startup-peer-d 10.77.0.5
```

Exporter tạo directory mới dưới local/, pin generated frames/provenance dùng committed four_player_profile.py,
giữ hai-client path. Chỉ diagnostic4/knownnames/IP, không auth; bỏ Battle/terrain vì chưa port bốn.
Chạy service cùng cách hai-client, topology cô lập theo recipe. Public export output đã kiểm7TCPcases;
native R3 dùng privatepreparer trước exporter enhancement, không gọi mọi generated input đã native-tested.

```bash
python3 tools/test_rust_four_room.py --binary rust/target/debug/kr-service \
  --config local/four-config.json --out local/new-four-tcp
```

Native scope: Manner1/đội alternating2v2/room/4ACKbarrier/clock/currentprogress12; chưa fourBattle/results/reset/EOF.
58workspace tests; two-client51TCPregression giữ. PLAN tiếp fourownerbom/life/rank4/abort rồi tám.


## Bốn owner bom — CP246

[Contract/giới hạn](../docs/PROTOCOL.md#rust-four-owner-bombs), [raw/hash](../docs/evidence/rust-service/README.md#cp246).
Thêm `--four-owner-bombs` vào exporter với đủ four-startup flags để đặt `fixed_room.four_owner_bombs=true`.
Chỉ capacity1/2/1/1/power1/designfuse6; không bật Battle/terrain/life/results. Không flag giữ startup4 cũ.

```bash
python3 tools/test_rust_four_room.py --binary rust/target/debug/kr-service \
  --config local/four-config.json --owner-bombs --out local/new-four-bomb-tcp
```

14syntheticcases khác nativeproof: CP246 actual4Wine/copy4b/Manner1 đã có118→16native159entity→16native158/kind0/cleanup,
knownstats/7.196copiedgetters riêng;59Rusttests/51twoTCPregression. Four142/16722 chỉ finite/noeffect;
own-peer trap/death chưa đồng bộ vì127 chưa forward, chưa fullM4. Rollback dùng matchedCP245binary+startupconfig,
không đưa newflag cho oldbinary. PLAN tiếp life3recipient/rank4/reset/EOF rồi tám; M5bridge riêng.


## Life bốn người — CP247

[Contract/giới hạn](../docs/PROTOCOL.md#rust-four-life), [raw/hash](../docs/evidence/rust-service/README.md#cp247).
Thêm `--four-life-reports` cùng `--four-owner-bombs` và đủfour-startup flags của exporter;
đặt fixed_room.four_life_reports=true. Chỉ17/19/21→ba peer, không rank/result/terrain/generalaccount.

```bash
python3 tools/test_rust_four_room.py --binary rust/target/debug/kr-service \
  --config local/four-config.json --life-reports --out local/new-four-life-tcp
```

Native4 Manner1/copy4b/Wine đã có A/C17→19/12recipient170/remote56→1/all4states[1,0,1,0].
Clear21đủ3TCPwrites mới complete có unit/TCPproof, chưa nativeclear21/rescue/needle.60workspace/17fourTCP/51twoTCP đạt.
Harness giữ PID/thread/wchan khi deadline; timeout cũ chưa rõcause. Rollback matchedCP246binary+config khôngnewflag.
PLAN tiếp rank4/haioutcome-reset/Start3/abortEOF rồi8; M5bridge riêng.


## Kết quả 2v2 bốn người — CP248

[Contract/giới hạn](../docs/PROTOCOL.md#rust-four-results), [raw/hash](../docs/evidence/rust-service/README.md#cp248).
Thêm `--four-results` với `--four-life-reports --four-owner-bombs`/đủfour-startup flags; fixed_room.four_results=true.
Typedrank4/16slots/zeroeconomy; currentwitness/bombdrain/allrecipient170writes/settle8; all-dead/tie4 unsupported.

```bash
python3 tools/test_rust_four_room.py --binary rust/target/debug/kr-service \
  --config local/four-config.json --results --out local/new-four-results-tcp
```

62workspace/24fourTCP/51twoTCP/4prebind đạt; native same4/currentManner1 hai opposite2v2/full175180/reset/Start3.
Ảnh bảng điểm B/C/D đủ2vectors; Aảnh vẫnBattle, chỉ nativeAparser/rank/resetproof riêng. Chưa healthyfourEOF/eight/fullM4.
Rollback matchedCP247binary+config khôngnewflag; không dùng two-rank fixture/config cho4. PLAN tiếp unrankedabort/EOF rồi8,
M5bridge riêng; audit kế252. Internet thật chưa kiểm.


## Abort không xếp hạng bốn người — CP249

[Contract/giới hạn](../docs/PROTOCOL.md#rust-four-abort), [metadata](../docs/evidence/rust-service/four-abort.json).
Exporter thêm `--four-abort-room`, cần `--four-results --four-life-reports --four-owner-bombs`/four-startup inputs.
Đây là opt-in một active memberloss:180→đủ3postreturn585→107departed/3survivor110 mỗi view; không175 hoặc106 replay.
Phòng sealed sau abort; không Start3người/replacement/resume. Default profile chưa tự bật flag này.

```bash
python3 tools/test_rust_four_room.py --binary rust/target/debug/kr-service \
  --config local/four-config.json --abort-role D --out local/new-four-guest-abort-tcp
```

Đổi D thành A để kiểm hostloss.20guest/20hostTCP/65Rusttests/32normalfour/51two/6guards đạt; hai native run có
native180107/IDclear/Room/chief. Evidence sink dùng tmpfs riêng/exporthash; Btrfsflush lỗi giữ nguyên/chưa sửa,
không nghiệm thu vận hành bền vững. Rollback binary/config CP248 không flag mới, giữ Pythonreference; không hot-swap activeconnection.
PLAN tiếp8, M5nativebridge riêng. Internet thật chưa kiểm.


## Baseline tám người — CP250

[Contract/giới hạn](../docs/PROTOCOL.md#rust-eight-baseline), [evidence](../docs/evidence/rust-service/eight-baseline.json).
Exporter `--eight-startup-derived-dir` với8`--eight-startup-peer` (hai/fixed-room input), `--eight-battle` opt-in;
không trộn fourexpansion/flags/terrain. Không import currenteightWIP; fixedalternatingteams/knownzero profiles.

```bash
python3 tools/test_rust_eight_room.py --binary rust/target/debug/kr-service \
  --config local/eight-config.json --out local/new-eight-tcp
```

69Rusttests/16eightTCP/5guards; native cùng8Manner4v4/32TCP/56mesh/statbom8/life7peer/rankresetStart3.
R1Dcollateraldead được giữ, winningteamrank1; không deterministicmovement/exactCP210vectorclaim.
Native/tmpfsrawexporthash, Btrfsloggingdurabilityunknown. M4cònfixedFree1v7/selector/unbalancednegative/native9admission;
không fullM4/8EOFreturn/fullauth. Rollback matchedCP249binary+profilekhôngeightflag; không hot-swap activeconnections.
Internet thật chưa kiểm.


CP251: fixed-room config nhận `eight_team_layout: "4v4"|"1v7"` chỉ với `eight_battle`; exporter có
`--eight-team-layout` tương ứng. Default4v4 giữ. 106/110/121/rank cùng theo layout; Manner lệch đội reject104
trước roundallocation, không phát errorwire tự đoán. [Contract/nativecopy8d/Freeblocked](../docs/PROTOCOL.md#rust-eight-teams).
Đã kiểm TCP Free1v7/Manner4v4 và Manner1v7 `--selector 1 --manner-negative` trong `test_rust_eight_room.py`;
native Free2outcomes còn thiếu, không nhận fullM4/auth. Python team reference đã giao riêng PR322. Internet thật chưa kiểm.


CP252: fixed-room config có `ninth_admission_probe: true` với chín diagnostic lobby actors và `eight_battle`;
phòng vẫn tám slot. Exporter có `--ninth-admission-probe-peer`/`--ninth-admission-derived-dir`, tạo visitor KRPeer9;
không có roomplayer_I. `test_rust_ninth_admission.py` kiểm sáu ca TCP; `rust_ninth_probe.py` cung cấp tám member tổng hợp
cho native harness. [Native scope/giới hạn](../docs/PROTOCOL.md#rust-ninth-admission): một KR visitor, tám member TCP,
không chín client native hoặc fullM4/auth. Freeactual32control vẫn startupblocked; M5bridge độc lập tiếp theo trong PLAN.

<a id="account-admission-bridge"></a>
## Admission account ở mức thư viện, CP253

`account_bridge::Bridge` dùng bounded account worker hiện có để kiểm strong backend session/own Player,
cấp frontend32 riêng và chuyển một lần login → redirect-delivery → channel generation.
`Channel::check` authorize lại và fail closed; source IP hoặc field36 echo không tự tạo quyền.
[Contract/giới hạn](../docs/PROTOCOL.md#rust-account-bridge), [raw/hash](../docs/evidence/rust-service/README.md#cp253).
Chưa gắn listener/profilewriter/launcher API. TTL dùng chung10s cần kiểm startup khi nối native;
không coi replay parser ngoại tuyến là KR đã dùng backend account, full authentication hoặc M5 complete.

```bash
python3 tools/test_rust_account_bridge.py --out local/new-account-bridge-check \
  --oracle-manifest local/retained-native-bridge-oracle.json
```

Manifest oracle phải pin raw native đã đo, expected token SHA/name-ID-echo từ nguồn độc lập;
example `check_account_bridge_wire` chỉ decode stdin hữu hạn, không listener hoặc log plaintext token.
Test lưu DB/raw vào directory riêng0700 và kiểm source/hash/PIDgone; password/DB/token không commit.
Python và diagnostic Rust serving giữ vai trò hiện tại. Native end-to-end M5 vẫn theo PLAN.

## Account lobby opt-in CP254

[Contract/limits](../docs/PROTOCOL.md#rust-account-serving), [metadata](../docs/evidence/rust-service/account-serving.json).
Rust `account_admission` dùng DB/private Unix socket và hai known zero-profile, session64→frontend32 riêng, không password trênTCP.
82Rust/9library8DB3retainednativebody/16TCP/11startupguards; nativeR5 có haiKR/Wine/copy4b/137-291-render/argv/18exactTCP.
Raw ở `local/research/codex/rust-account-native-r5/native-pair`, 100file/41,852s/cleanup0 và native-verification/UIreview ởparent.
PrivateTMPFS DB/log/raw/exporthash, khôngBtrfsdurabilityfix. NativeR2/R3/R4fail và preparationfail giữ nguồn/raw.
`tools/test_rust_account_serving.py --binary <kr-service> --admin <kr-admin> --config <two-profile> --out <new-private-dir>`
kiểm synthetic TCP, khôngnativeoracle. Accountmode reject room/battle/unknownprofile. [CP255](../docs/PROTOCOL.md#rust-account-native-lifecycle) đã kiểm native wrong-pending-correlator/revoke/absolute expiry; freshlogin/replay/generalprofile/accountroom-game/fullM5 còn thiếu.
Python/diagnostic rollback giữ. Internet thật chưa kiểm.
