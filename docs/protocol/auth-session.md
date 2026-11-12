# Đăng nhập, token, account và lease

[Mục lục](../PROTOCOL.md) · [Câu hỏi đang mở](../RESEARCH.md) · [Thứ tự việc](../PLAN.md#dashboard).

<a id="rust-account-backend"></a>
## Backend account Rust độc lập client — CP236 / RUST-M5 / KRE-074

Nguồn `design/design`, không là contract server KR gốc. [Implementation](../../rust/crates/kr-service/src/accounts.rs),
[migration](../../rust/crates/kr-service/migrations/0001_accounts.sql), [CLI](../../rust/README.md#account-admin-cli),
[bằng chứng](../evidence/rust-service/README.md#cp236). Tái dùng design account CP213/M5, M1 crate/lock/tests,
CP235 baseline transport. Không import VN wire/accountserver hoặc suy diagnostic137/291 là auth.

- SQLite bundled, schema1/applicationID `KRAC`, migration sourceSHA256 + so live schema với schema sinh từ đúngmigration
  (tables/indexes/triggers), quickcheck/FK/startupallocator; từ chối foreign/future/checksum/schema hỏng, không downgrade.
  Parent phải là directory riêng mode0700 dưới đường dẫn admin tin cậy, DBregular/nosymlink/mode0600; WAL/FULL,
  FK on/trusted_schema off, busy250ms. Không chống admin sửa DB/ancestorpath hoặc mã hóa DB.
- Login backend ASCII3..32 alnum/underscore, lowercaseunique; displayname ASCII3..15 là policy nội bộ, chưa nativeprojection.
  Account→player1:1/FK, wireID allocator1..u32MAX riêng slot/accountID, không tái cấp qua reopen; tên đổi theo version CAS.
  Chưa character/XP/inventory. CLI account lookup/rename là admin, không quyền client tự khaiaccountID.
- Password12..128bytes, randomsalt16bytes, Argon2id v19/m19456KiB/t2/p1/output32, PHC atrest; chỉ nhận đúng PHCparams/length
  trước verify, tránh KDFunbounded. Unknownuser chạy dummyKDF cùng cost. Tối đa2 hash đồng thời/process, vượt trảBusy;
  CLI tuần tự, chưa benchmark/load hoặc giới hạn đa process/per-account. Không log password/PHC/token trong diagnostics.
- Sessiontoken CSPRNG32bytes→64lowerhex, DB chỉ SHA256 của chuỗi token; absoluteTTL1..86400s, một active/account
  theo transaction+partialuniqueindex. Issuance recheck credentialversion/PHC/disabled dưới write lock sau KDF,
  reset/ban thắng in-flightverify; expiry/revoke/disable/reset chặn ownership. Authorize trả ownplayer chỉ khi tokenhash,
  playerID, credentialversion, bootepoch, UTC và disabled đồng thời hợp lệ. Không authenticated transportlease/idleTTL.
- `open` không revoke. `begin-instance` là hành động admin explicit: tăng persistentepoch và revoke mọi session cũ cùng
  audit; chưa tự gắn service startup hoặc chống hai gameprocess đồng thời. UTCwatermark ghi cả denial trước businessTX,
  đồng hồ lùi failclosed qua reopen, token đã thấy expired không revive. Không resetclock tự động; chỉnh clock cần runbook.
- Create account+player+allocator+audit, rename/version+audit, ban/unban/reset/revoke/bootepoch+audit atomic.
  Auditactor/reason do trusted localoperator khai, randomrequestID, UTC; audit lỗi thì rollback mutation, watermark có thể tăng.
  Không remoteRBAC/tamperproofaudit/adminretryidempotency hoặc rewardexactlyonce claim. SQL/FULL/busy/entropy lỗi không trả success.

26Rusttests =11account+10transport/model+5wire retainedvectors; 33CLIprocess/2persistentaccounts/9auditedmutations,
reopen/profileCAS/ownership/ban-reset-revoke/instancefence, SQLite integrity/FK và plaintext-atrest scan đạt.
Ca fault bao gồm realtwoDBconnections loginrace, reset giữa verify/commit, strictPHC/quota, audittriggerabort,
SQLite `max_page_count` FULL giữa audit (không phải OSdiskfull), busy250ms, entropyfault, future/checksum/dropindex,
expiry forward→rollback→reopen. Native M2 cũ giữ baselinebinary/hash; **không chạy native KR mới cho backendCLI**.
Run CLI1 phát hiện serde unitvariant bỏ qua unknownfield: giữfailure/raw, đổi init/begin thành emptystructvariant,
CLI2 strictnegative đạt; không thay input/source giữa hai run. Secretinput≤4096bytes/stdin-only, output token chỉ login
explicit; rawtoken chỉ syntheticlocal dưới private directory, không commit secretDB/binary/raw.

M5 chưa complete: boundedservingDB/hashworker, idleTTL/rate và lease/session binding vào các socket KR,
expiry/revoke/freshlogin native hai testaccounts/negative, DBrestartadmission/lifecycle/backuprestore/load còn thiếu.
Không expose password trên legacyTCP hoặc gán sourceIP thành owner. M6 inventory/progression/resultreward riêng.
Python serving/RE/reference giữ; [PLAN](../PLAN.md#current-order) giữ thứ tự. Internet thật chưa kiểm.


<a id="rust-account-worker"></a>
## Worker account/DB hữu hạn và login-result handoff — CP243 / M5 partial

[Worker](../../rust/crates/kr-service/src/account_worker.rs), [Store](../../rust/crates/kr-service/src/accounts.rs),
[metadata/acceptance](../evidence/rust-service/account-worker.json), [raw/recipe](../evidence/rust-service/README.md#cp243).
Nguồn `design/design`: tái dùng CP236/schema1/Argon2quota/Store clock-audit; không native function/wire RE mới.
M3 scoped CP242 giữ baseline, **không nối worker này vào socket KR hoặc diagnostic IP identity**.

`Worker::open(path, capacity, queue_ttl)` mở/schema-check Store trên một thread `kr-account-db`, không KDF/SQLite
trong Tokio task. Client clone dùng chung admission; 1–64 queue entries +1 executing, không tạo thread/job theo request.
`try_login`/`try_authorize` trả typed Pending; bounds name/password/token/TTL trước cấp phát bản secret copy,
full→Busy/closed→Closed; password/token worker sở hữu Zeroizing/noDebug. Mỗi worker tuần tự; hashquota2/process CP236
vẫn giữ. Không tự begin-instance/revoke hay khóa độc quyền DB/multipleprocess; caller là trusted internal code/UTC.

Job bị hủy hoặc quá queue TTL trước execution không gọi Store/cấp token. Đã chạy KDF/SQLite thì không giả rollback
bằng async timeout. Login commit chỉ coi đã giao khi caller poll `Pending::wait` và claim handoff dưới mutex;
**oneshot.send thành công chưa đủ**. Handoff timeout dùng cùng duration1ms–60s nhưng là giai đoạn riêng sau execution.
Caller bỏ result/không poll/claim thua expiry→thu hồi chính token vừa cấp cùng `abandoned_login` audit trong transaction.
Không thu hồi toàn account; old cleanup không hủy session mới hơn hoặc lùi UTC watermark. QueueExpired có thể ở queue
hoặc handoff; không là idleTTL/sessionexpiry/rate limit. API caller đã nhận Grant còn phải xử lý transport/session lifecycle.

Compensation/audit lỗi→rollback transaction, đóng admission mọi clone, shutdown trả root backend error; **session có thể
vẫn active**, cần operator revoke/begin-instance trước khôi phục phục vụ. Không tự ghi cleanup success. Worker panic cũng
đóng admission. Owner Drop chỉ khởi động close; `shutdown().await` đóng nhận việc, drain/cancel accepted jobs và join.
Không hard OS I/O/shutdown latency/load/SLO claim; syscall/KDF đang chạy không bị kill bằng timer.

49workspace tests (44service incl5worker +5wire); 5worker tests riêng chạy trên current-thread Tokio với real SQLite/KDF,
barrier chứng minh Tokio vẫn tiến khi DB thread giữ; bounded/full/oversized/queued cancel/expiry, caller chưa poll dù send
thành công, handoffexpiry→revoke→login lại, wrongplayer, stalecleanup/newsession/clock, auditabort/panic/startupfail/join.
Giữ5 DB quiescent/integrity/FK/schema1/active/audit đối chiếu; faultDB vẫn active1 đúng rollback và workerclosed, không recoveryclaim.
Testpanic cố ý nằm trong rawlog, không client/server runtime crash. Harness build/pin/testbinary/source unchanged/PIDgone.
Clippy r1 lỗi collapsible-if được sửa trước run nghiệm thu; logs giữ. Không native KR run mới cho backendlibrary này,
binary build mới không tự thành native-certified; M3 native baseline/frozenbinary vẫn ở CP242.

M5 vẫn thiếu idleTTL/per-account rate/authenticated multi-transport lease/nativeKRbridge; M6inventory/reward và M8restore/load
riêng. [PLAN](../PLAN.md#current-order) tiếp idleTTL/rate rồiM4, audit kếCP244. Internet thật chưa kiểm.

<a id="rust-account-idle-rate"></a>
## Account idle TTL và worker rate limit, schema2 — CP244 / M5 partial

[Store/migrate](../../rust/crates/kr-service/src/accounts.rs), [migration002](../../rust/crates/kr-service/migrations/0002_session_idle.sql),
[limiter](../../rust/crates/kr-service/src/account_worker/rate.rs), [metadata](../evidence/rust-service/account-idle-rate.json),
[raw/hash](../evidence/rust-service/README.md#cp244). Nguồn `design/design`, không contract KR gốc; không native RE mới.

Mở DB xác minh AppID/version/checksum và **actual schema cũ trước upgrade**, rồi 002/version/checksum/schema trong
transaction. 0001 source/hash giữ nguyên. Schema2 thêm nullable idle_timeout_seconds/last_seen_at +partialindex;
mọi old account/player/wireID/profile/PHC/session tokenhash/audit/UTC giữ nguyên, session cũ idleNULL dùng absoluteTTL.
Future/corrupt/checksum sai bị reject; SQLiteFULL trong002 rollback columns/version/data, retry khi đủpages đạt.
CLI init trả schema2 thực, không giữ literal1 sai. Không downgrade/deleteusers; oldbinary schema1 reject2 đúng chủ ý.

API `Store::login_with_idle`/`Client::try_login_with_idle`: idle1..86400s explicit cho **new session**. Chỉ authorize đúng
currenttoken+ownplayer+epoch+credentialversion/disabled+absoluteTTL+idlewindow mới ghi last_seen=UTC. Boundary now≥last+idle
bị từ chối; wrongplayer/token không gia hạn. Renew không vượt absoluteTTL; durable UTCwatermark và reopen không revive
expiredtoken khi clock lùi. New login transaction revoke idle-expired active trước cấp session khác; ban/revoke/reset vẫn thắng.
Không tự dùng socketalive/heartbeat tùy ý làm activity hoặc đổi luật phiên cũ; KRbridge phải chỉ gọi authorize tại đúng seam.

Worker admission dùng chung một mutex/limiter cho mọi Clientclone, casefold principal ASCII trước SHA256 key.
Fixedwindow monotonic: default5attempt/60s/1024principal; `open_with_rate` cho1..1000attempt,1..3600s,1..4096principal.
Full/closed queue không charge; acceptedjob dù caller cancel vẫn charge. Unknownprincipal cùng policy; tablefull chỉ prune
expired, không evict activekey để cho attacker đổi tên. RateLimited trước KDF. Store/adminCLI trusted không đi limiter;
rate process-local mỗi worker, reopen reset; không distributed/global/network/IP/device/production abuse defense.
Authorize chưa per-session/networkrate, queue vẫn hữu hạn; nativecredential/multi-transport ownership chưa nối.

55Rusttests:50service+5wire. 7worker cases/7retained schema2 DBs; clone/casefold/fullqueue-nocharge/cancelcharge/unknown/
fulltable/reopen và idle đúngowner/wrongplayer/externalrevoke. Limiter model kiểm exactwindow/clockreverse/noactiveeviction.
Migration harness độc lập:5 bản sao DB CP243 đã seal,10CLIinit mở lần đầu/lặp lại; so hash/count **mọi cột cũ ở mọi bảng
business**, schema1→2/checksum, oldidleNULL/FK/integrity; baseline không mở hoặc sửa, copy-only. SQL FULL ở unit test là
SQLite max_page_count, không OSdiskfull. 33CLI account regression trên schema2 riêng. Source/binary/input giữ từ preflight
đến seal/processgone; raw credentials chỉ private local. Không lấy Python/Rust đồng ý thay native authentication.

AUD070 quaCP244 đối chiếu CP241–244, sửa PLAN/RESEARCH còn gợi M3 chưa xong và phân biệt DBworker với subagent;
reviewwaived/Codexselfcheck, checker không xác nhận ngữ nghĩa/gate. M5 vẫn partial: nativeKRbridge/transportownership/
worker-to-serving thiếu; inventory/reward/restore/load riêng. M3nativebaselineCP242 giữ, rootbuild mới chưa nativecertified.
[PLAN](../PLAN.md#current-order) tiếp M4four/eight và M5bridge theo gap thực tế; audit kếCP248. Internet thật chưa kiểm.

<a id="login-auth"></a>
### Login auth trên copy — CP014

Nguồn raw/recipe/hash và các control/failure: [evidence login](../evidence/login-runtime/README.md).
Các facts dưới là `kr_runtime/confirmed` hoặc `kr_static/confirmed` trong phạm vi copy đã khóa,
không phải đăng nhập Nexon thật hoặc K2/K3 đã pass.

- Luồng tự động `11010F0` đọc config `token_1/token_2`; tĩnh chọn count0/1/2 theo chuỗi rỗng.
  Runtime `token_1:offline-probe` đi tới `13B07F0` với1; cùng copy không token có0.
  `1255C90` chỉ gọi builder khi count1..2. Tại CP014 chỉ token_1/count1; [CP214](#auth-token-request) đã đo cả count2 trên copy4b.
- `1255DC1→13ADE40` yêu cầu getter `13AD5B0==-1`, socket ready và count1/2; gọi base ctor
  `14EAD00` với opcode1. Capture connection3 có **C→S1/80 byte**, kế **C→S60/58 byte**, rồi C→S15.
  Native parser `14EF300` replay7 frame với seq40+3n, khớp CRC/payload; CRC-sai/partial từ chối.
  Hai u8 đầu payload auth được native reader đọc4 và1. Full schema/biến thể auth còn unknown.
- Chiều nhận của login socket: tại `14EF300`, hello có mode0/selector0/seq40; phản hồi kế có
  mode2/selector3/**seq43**. Fixture S→C4 result1 dùng43 vào `10FCBE0→10FEDA0`, guard
  `stage+2FD8=1`, decode1 rồi nhánh lỗi đặt dialog result1. Đây là thử nghiệm từ chối login.
  Fixture sai giả định seq40 trước đó vẫn qua dispatch nhưng decode2; CRC của mẫu1byte không
  đủ chứng minh plaintext/seq. Không dùng seq chiều gửi thay chiều nhận.
- Prefix S→C4 result0 có thứ tự đã đo: **outer u8 result, u8, u16, u8 playerCount, u64**.
  Native replay đọc0/0/1/1/0; runtime vào handler result0 và đọc khớp prefix/count local1.
  Tại CP014 body theo từng player/đuôi chưa đủ. [CP058](lobby-profile.md#auth-profile) đã sửa chuỗi và lấp
  nhánh count1/feature đang chạy tới callback errors0; chưa full auth semantics/variants hoặc S→C15.
  CP020 mở prefix tới trường ID local, xem [contract](load-phase.md#start-reply); không đóng full auth.
- Prefix13byte cố ý thiếu body vẫn dẫn tới C→S9/46byte. Cấp tiếp fixture S→C15 với seq46,
  blob reader chứa host loopback, ba u16=0, mode0 và tailu16=0: đã vào `1100590` và đo wait
  **0→2→1→3**, setter result3 từ`C4704E`, modal trả3. Không suy getter bounds hoặc schema hợp lệ
  từ việc client tiếp tục với dữ liệu thiếu. Các giá trị loopback/zero là `emu_choice`.
- Sau modal, slot`16EA598` trap ở`13BFD44`, arguments hwnd/msg403/1/0. Copy bind thử
  `PostMessageA` (`emu_choice/candidate`; `SendMessageA` còn alternative) cho attempt trả1 và tới
  xử lý message tiếp `1259C65→125AF18→13BFF43→13C2052`. Run này dừng ở trap`2265828`,
  slot`16EA8FC`, call`13CA429`/return`13CA42F`; đó là blocker CP014, được tiếp nối ở CP015 bên dưới.
  Đây là return của luồng chẩn đoán, không phải login hợp lệ, vào kênh/sảnh hoặc gate K3.

Recipe mới bind Toolhelp process/thread và các API window trên copy, giữ global1A966FC reset0 trong
bộ nhớ như CP012. Các API phục hồi là `emu_choice/candidate`; `GetWindowTextA` còn alternative ABI
`GetClassNameA` chưa loại trừ. Token là giá trị giả local. Không ép modal/state thành success;
không suy các patch/import/API này là tương đương vanilla hoặc là nguyên nhân duy nhất.

<a id="auth-token-request"></a>
### Token local trong C→S1 — CP214

Nguồn [ba run/recipe/hash và sửa oracle](../evidence/login-runtime/README.md#cp214),
[metadata](../evidence/login-runtime/token-request.json); KR00967/originalSHA trong metadata, offlinecopy4b/Wine10.0.
Nguồn cũ dùng lại: CP014 builder/count/guard, CP058 reply và CP181 codec/retry; VNlogin chỉ tham chiếu,
không chuyển username/password/opcode/layout. IDA unavailable không làm mất cache/provenance.

**`kr_runtime/confirmed`, chỉ nhánh config token giả đã đo:** `13B07F0` nhận1/2,
`13ADE40` lặp đúng count; `105D050=0`, `122AE50=1`, writer `14EC040` nhận chuỗi13byte/key11223344.
Hai token_1 cùng độ dài A/B cho frame80/body42, hai token cho frame95/body57, seq40 ở socket thứ ba.
Body bắt đầu hai typed u8=4,count; mỗi record tại offset2 rồi17 gồm typed u16=13 +13byte biến đổi.
Thêm token2 thêm15byte; suffix25byte bắt đầu17/count1 hoặc32/count2 giống nhau giữa ba run.
Không có username/password plaintext được xác nhận; các nhánh provider khác không đi qua trong ba run.

**`kr_static/confirmed`, emulation tách runtime:** copied sender ctor14EAD00/vtable17B0B7C,
`14EC040→14EC060→14ED340`, synthetic buffer/mode2 đã cho đủ15byte khớp bốn record capture.
Đây là biến đổi định dạng, không chứng minh cơ chế xác thực an toàn. Copied receiver14EF300/mode2-selector3/seq40
replay ba frame actual: return0/CRC khớp/body khớp; CRC-flip return7/error10/consumed0, partial return2/consumed0.
Prefixcheckoff và allocator synthetic theo recipe cũ; không phải backend thật kiểm credentials.

**Giới hạn:** input13byte/ASCII, một nhánh local config/copy đã sửa; chưa length16+ hoặc token2-only,
chưa giải nghĩa suffix hay các flag mở rộng. Không gửi authsuccess, không kiểm expiry/revocation/swappedchannel
và không chứng minh account→player/sessionownership. Tại CP214 request6/name+tail chưa được nối với token; [CP216](#auth-channel-echo) phục hồi layout/field36echo, chưa tokenownership.
Contract dùng trực tiếp để parse một nhánh có bounds; thiết kế backend auth/Argon2id/session vẫn là `design`.
Phép thử tiếp ở [RUST-Q1](../RESEARCH.md#rust-service-gaps); không sửa/đóng gate demo hoặc fullauth từ kết quả này.

<a id="auth-token-bounds"></a>
### Ranh giới chuỗi token và token2-only — CP215

[Hai run/recipe/hash](../evidence/login-runtime/README.md#cp215), [metadata](../evidence/login-runtime/token-bounds.json);
reuse CP214 writer/framer và cache11010F0 của CP014, không mở hàm mới hoặc port VNlogin.
`kr_runtime/confirmed` trên cùng offlinecopy4b/Wine10.0/localstub: một run hai input15/16byte cho count2,
writer14EC040 nhận MSVC size15/cap15 rồi size16/cap31. Actual C→S1 seq40 **frame100/body62**;
typedprefix4,2; record17byte ở offset2 và18byte ở offset19, suffix25byte bắt đầu37 và giống CP214.
Cả hai record khớp đầy đủ copied-native sender độc lập. Không có auth-success reply.

Run chỉ khai token_2 dài16byte: native settercount0, không gọi builder13ADE40, không writer token;
connection3 có bốn C→S15/CRC hợp lệ, không C→S1 hoặc60 trong cửa sổ42s.
Điều này khớp cache tĩnh11010F0: token_1 rỗng → count0; chỉ token_1 →1; cảhai →2.
Không tự gom token_2-only thành một record thứ nhất; đây chỉ đường tự động config đã đo, không mọi luồng UI/provider.

`kr_static/confirmed`/emulation riêng:13 vector helper tại độ dài0,1,3,4,7,8,13,15,16,17,31,32,64
trả recordlength=n+2/cursor và typedlength đúng; không thay crypto helper bằng encoderPython.
Actualframe100 replaynative return0/bodyCRCkhớp; CRCflip return7/error10/consumed0, partial return2/consumed0.
Vector0/64 không là proof runtime nhận token rỗng/64 hoặc maxinput.

**Phạm vi mới:** actualtoken ASCII13/15/16, configcount0/1/2 theo những tổ hợp đã đo.
Fulltokenbounds/encoding/provider/password/suffixsemantics/credentialverify/expiry/revoke/accountownership vẫn unknown.
Không thay công thức backendauth bằng obfuscation này; bước tiếp ở [RUST-Q1](../RESEARCH.md#rust-service-gaps).

<a id="auth-admission-prototype"></a>
### Adapter admission cục bộ — CP217

[Hai run/raw/hash](../evidence/login-runtime/README.md#cp217), [metadata](../evidence/login-runtime/admission-prototype.json).
Dùng lại CP214–216token/field36/layout6, result1 CP014 và Profile47 CP181; sourcehash/raw giữ riêng.
Native14ED340/token-key11223344 khớp **hàmcrypt hiện có** trên13copied-nativevectors+6capturedrecords,
không viết cryptor hoặc dựng golden mới làm oracle.

**`emu_choice/design` — prototype riêng trong local, chưa tích hợp server/Rust:** một account giả,
đối chiếu tokenC2S1 → cấp pendinglease10s mộtlần qua field36 → check C2S6ID/name/echo/sourceIP.
Token công khai cố định chỉ là input RE. Echo32bit là correlator của nhánh legacy, không credential đủmạnh.
Rate4 lần/peer/30s, max32peer,1024ticket budget không reuse trong process rồi failclosed là lựa chọn thử nghiệm.
Không DB/password/registration/Argon2id/accountAPI/sessionexpiry/persistentrestartfence hoặc productionauth.

**`kr_runtime/confirmed` trong Winecopy4b/namespace loopback:** inputtoken32byte đi qua writer native
(heapcapacity≥32), C2S1seq40/frame99/body61; suffix25byte giống CP214.
Input đúng: nativeauthresult0 → field36reader/setter/getter/serializer vàactualC2S6echo khớp lease cục bộ;
adapter nhận kênh, native840470entrymarkers vàactualC2S67 tiếp theo có nguồn. Entrycount không là số packet.
Input sai cùng32byte/cùngsource/profile: nativeauthresult1 từ reply lỗi CP014, không C2S6/67 và khônglease
trong45s. Run không force nativeauthstate hoặc functioncall. Serverevents là policy evidence; nativewriter/
actualframes/result và bước tiếp của client được đối chiếu riêng, không dùng logserver đơn lẻ.

**Policy emulator, không nativeacceptance:**24 kiểm với actualretainedframes và negativefieldmutations
cho expiry/revoke/replay/identity-name/source/oneowner/rate/collision/staleEOF. TạiCP217 chưa nativeexpired/revokedlease; [CP218](#auth-lease-eof-blocker) đã thử nhưng clientEOFfault,
chưa negativeacceptance; tạiCP217 chưa replay kernelTCP. [CP220](#auth-tcp-ownership) bổ sung adapterTCPdesign,
haiaccountKR thật/swappedchannel vẫn thiếu. Pendinglease expiry khác sessionexpiry.
**Giới hạn:** frontend32chars không entropy/securityproof; chưa fullauth/DB/accountresume/matchrestore hoặc gate mới.
Phép thử kế ở [RUST-Q1](../RESEARCH.md#rust-service-gaps); M0–M1 không phụ thuộc fullauthRE.

<a id="rust-account-bridge"></a>
## Cầu nối account → admission KR ở mức thư viện — CP253 / M5 partial

[Implementation](../../rust/crates/kr-service/src/account_bridge.rs), [parser](../../rust/crates/kr-service/src/account_bridge/wire.rs),
[metadata](../evidence/rust-service/account-bridge.json), [recipe/raw](../evidence/rust-service/README.md#cp253).
Nguồn **design/design** cho policy/backend; wire dùng contract KR đã đo CP214–217/225.
Không RE binary mới hoặc nhập VN auth. Store/schema2/KDF/session/worker/idleTTL/rate CP236/243/244 giữ lại.

`Bridge::issue` yêu cầu strong session64 và player ID do backend xác thực, không lấy peer IP làm principal.
Sau authorize qua worker, chỉ Player có name ASCII5/7/wireID khác0 được cấp frontend token CSPRNG16byte→32hex riêng.
Không cắt session64 xuống32 hoặc truyền password vào legacy TCP. FrontendToken không Debug/Clone/Serialize,
zeroize khi drop; registry chỉ giữ SHA256 frontend cùng bản sao zeroized backend session cần cho reauthorization.
Hai account cùng IP được kiểm riêng. Backend API/launcher là thiết kế công ty, không phải màn login/register mới trong KR.

Parser login chỉ count1/token32/body61, crypt có sẵn/key11223344; suffix opaque16 không là credential, u32zero/
peer IPv4 little-endian/u8zero là profile network CP225. Parser channel kiểm prefix1/255/count1/name5 hoặc7/ID/echo,
body36+name và bool optional-string-off tại name+31; 23 byte còn lại giữ opaque. Unsupported branch bị từ chối,
không đoán tail là sessionkey. Field36 echo32 chỉ correlator legacy, không đủ strong channel authentication.

Frontend chỉ dùng một lần trước DB await. Login tạo echo32 không0, không reuse tối đa1024 trong process;
`Login::redirected` chỉ được gọi sau write redirect thành công, trước đó drop hủy ownership. Channel phải khớp
current redirect/peer/owned ID/name/echo/connection khác login; một channel được giữ bằng RAII generation.
Capacity1..32, TTL launch/handoff hiện dùng chung1ms..10s từ issue; hết hạn không refresh khi chờ DB.
Sweep pending ở admission mới, bound phải check/drop; đây là library policy, chưa nghiệm thu startup native với TTL đó.
Không durable process/restart replay fence hoặc chống mọi topology/attacker đã biết echo.

Không mutex registry qua await DB/KDF. Mỗi issue/login/channel và `Channel::check` dùng worker
`try_authorize_current`: UTC được lấy **khi command chạy**, không lúc xếp hàng. Queue deadline/cancel kiểm trước
execution; generation/phase/deadline được đối chiếu lại sau await. Phiên hết hạn lúc chờ bị từ chối, không dùng thời
điểm enqueue để giữ quyền cũ. Revoke, profile/version đổi, DB/queue lỗi đều fail closed; failed channel check xóa đúng
generation trước trả lỗi. Drop của launcher/login/channel cũ không xóa generation mới. Authorization là check tại
transaction, không phải continuous revocation; listener tương lai phải chọn ranh giới authorize và fence queued work.
Session backend được mượn từ caller, bridge không cấp login session mới: cancel không revoke phiên của caller.
Own-player authorize đã chạy có thể renew idle dù caller hủy; không claim rollback hoặc cancel SQLite đang chạy.

**Kiểm design/library:** 7 ca,6 SQLite DB schema2/12 account-player, current-thread Tokio và worker thật;
same-IP hai owner, wrong player/unknown session/token, replay, wrong peer/name-ID/connection, redirect chưa delivered,
expiry launch/queued session, revoke trước login/channel, profile rename, cancellation ở issue/login/channel,
Busy/Closed, stale generation và real SQLite trigger write fault. TTL/generation điều khiển trong unit test tách với
UTC session-expiry thật khi queue đang pause. Chưa native wrong-owner/expiry/revoke/DBfault.
3 body native CP217 còn đúng complete pin được replay qua parser Rust: hai token-input SHA từ native writer và
một name/ID/echo từ channel native chain. Đây là **offline replay**, không native client chạy qua account backend.
Encoder synthetic trong unit test chỉ kiểm policy/negative; không là oracle độc lập.

**Failure/control:** compile lỗi coercion/API trước run được giữ; test cancel ban đầu giả định queue đã drain ngay
sau resume nên gặp Busy đúng thiết kế, sửa bằng barrier. R1 acceptance7ca/6DB pass nhưng source review phát hiện
bool optional-string chưa được constrain; R2 thêm guard/negative, giữ R1 source/raw và 23 byte opaque. Hai frozen
run đều hoàn tất/PIDgone trước đổi source. R2 source/hash freeze15,069s/65file kiểm; 80 workspace tests đạt.

Service listener/config/profile renderer/room/UDP vẫn diagnostic như CP252; binary rebuilt không là chứng nhận native M5.
Chưa launcher API, backend password network flow, profile writer account, socket/room/UDP invalidation hoặc full M5.
Theo [PLAN](../PLAN.md#current-order), nối adapter nhỏ và đo native hai account trước tuyên bố tương thích account.
M4 Free startup vẫn blocked, không retry từ bằng chứng thư viện này. Python RE/reference/rollback giữ.
Wine/LAN/WAN giả lập được chấp nhận; **Internet thật chưa kiểm**.

<a id="rust-account-serving"></a>
## Account lobby opt-in — CP254, native positive hai account cố định

[Source](../../rust/crates/kr-service/src/account_serving.rs), [metadata/limits](../evidence/rust-service/account-serving.json),
[raw](../evidence/rust-service/README.md#cp254), [thứ tự](../PLAN.md#current-order).
Policy backend **design/design**, wire reuse CP214–217/225, native positive **kr_runtime/confirmed** trong scope dưới.

`diagnostic_two_lobby.account_admission` chọn DB/private Unix socket/launchTTL; chỉ hai known zero-profile KRLocal/ID1,
KRGuest/ID2/version1. Projection lấy DB-owned wireID/name; sourceIP chỉ allowlist/profile fence. Own291 blob/name/98zero/
zero tail kiểm trước bind, profile khác/rename/version đổi/room/battle bị reject. Không successful unauthenticated fallback.
Backend session64 khác frontend CSPRNG128bit→32hex và field36 echo32. Echo/IP không strongcredential hoặc luật server gốc;
opaque16 và các field channel chưa rõ không được gọi sessionkey. Không thêm password/signup UI vào KR.

Unix launcher parent0700/socket0600/pathabsent; JSONline≤4096/session/player/peer,4concurrent/3sdeadline, workerqueue4/3s.
Only successful private write transfers launch retention; cancel/drop/writefail xóa đúng issued generation, không revoke phiên
backend của caller. Log không password/session/frontend/raw auth body. Launch30..120s/chọn60, handoff10s sau authorizedlogin;
CP252 WCP→login12,852s lowerbound làm10s trướclaunch không đủ, không suy TTLgốc. Register/process-local echo budget1024,
không durable restart/multi-process fence. C2S1/6 parser/count1/32/name7 và seq40 giữ contract đã đo.

Known login candidate3848 reuseCP234, channel3858 exact. Dynamic field36 tại86+nameLen, typedwriter/CRC/seq mới; native
C2S6 echo/name/ID được check trước claim, redirect chỉ publish sau write. Periodic1s `inspect_session` kiểm revoke/expiry/profile
mà không renew idle; current own67 mới là activity. Worker dùng UTC lúc command chạy; postawait phase/generation/deadline.
Unknown/auth/DB/queue error failclosed. Room/battle/UDP không mở trong mode này, chưa có account-owned game cleanup.

R4 bắt được actual87/body2 sau67 ở cảhai, hash21ddbbba… khớp encodedU16zero; **layout/semantics unknown**. CP254 chọn
chỉ ownedChannel/Lobby/exactbody, budget16/noACK/noeffect/noactivityrenew; wrongcontext/body/budget failclosed.
Python reference không87dispatch/noACK; không port semantic từ S2C87. NativeR5 tiếp tục render/lobbyhold8s với policy này,
không suy originalauthority hoặc mở mọi unknown opcode. 16ca TCP riêng kiểm replay/swap/expiry/revoke/87context-budget.

**Native R5:** 2SQLiteaccounts→privateAPI/backend64→frontend32→actual1→auth137/field36→actual6/67→own291128/render.
HaiKR/Wine10.0-20260425/copy4b không patch mới; hai netns/veth +rootbridge/noexternalroute/directTCP.
FullPCAP8connections/318TCP/18exactreplies/CRC-sequence-SHA, nativecursor137/128/errors0/render đúngKRLocal/KRGuest,
actualPopen/per-role32 và independentCAprocargv cảhai. Hai ảnh UI đã xem; SQLite2account/player/session integrity/FK.
100raw/41,852s/sourcefrozen/seal/cleanup0. DB/log/raw ở privateTMPFS/exporthash sau shutdown; không crashdurability/Btrfsfix.
82Rusttests/9library8DB3retainednativebody/16syntheticTCP/11startupguards; model/TCP khác native acceptance.

**Failures preserved:** baseexe e9 bị hashguard chặn trướccopy; dùng đúngCP252cmp4b. CLIcreate preparationBtrfs timeout10s,
khôngWine/PIDgone, không kết luậncausalI/O; nativeRAMDB giữdeadline10s. ControllerR2socketmismatch trướcclient/6raw;
R3capture thiếuIPB/A3848chưa configure/B87closure/100raw47,245s; R4known3848+captureB+denylog/105raw46,097s→87observable.
R5boundedpolicy mới pass; initialpostrunchecker thiếuhashlib được giữ rồi sửachecker, không rerun native chỉ để checkerpass.

M5 vẫn partial: native wrong-owner/channel swap/expiry/revoke/replay/fault/freshlogin/lifecycle và account room-game integration
chưa nghiệm thu. Không accountresume/matchrestore/generalprofile/fullauth/production. Python/diagnostic rollback giữ.
Wine/LAN/WAN giả lập chấp nhận; **Internet thật chưa kiểm**.

<a id="auth-channel-echo"></a>
### Producer C→S6 và echo từ reply đăng nhập — CP216

[Run/control/raw/hash](../evidence/login-runtime/README.md#cp216), [metadata](../evidence/login-runtime/channel-echo.json).
Reuse CP015redirect, CP058name/56fieldrecipe, CP181Profile47, CP184–186nameguard và CP214–215token;
cache CP014 đã có edge13AE944/95A/969, nay nối với nativecapture. Không dùng S→C6handler11004A0 làm producer.

**`kr_static/confirmed`, runtime chỉ count1/optional-string-off:** producer13AE740 gọi ctor6 tại13AE944,
serializer4CD040 qua14EC6E0, send13ADD10 tại13AE969. Serializer cho count1..2; body có
`u8 count,u8 opaque`, mỗi record `str_u16,u32 id,u32 aux,u64 opaque,u8 flag`;
đuôi chung `u8,u8,u32,u32,bool,u32`, và optionalstring dưới4CD450.
Tên/ID từ13B0A00/13B09C0, aux từ13B0D10; getter/setter13B0D10/13B0D30 dùng cùng box
`context+148h+0Ch*index`. U64 được caller11018DB lấy từ13C0EC0/localwrapper; chưa nối nguồn đó
với reply server hoặc gọi là sessionkey. Staticallowcount2 không phải runtimecount2 acceptance.

**`kr_runtime/confirmed` trên Winecopy4b/localfixture:** baseline và one-fieldcontrol đều tạo C2S6
seq40/frame81/body43, prefix1/255, tênKRLocal/ID1. Với tên7byte, aux chiếm body15..18;
suffix32byte cũ được phân thành17byte của record và15byte chung, không là blob hay8u32.
Chỉ đổi **S2C4 field36/u32/body93..96** từ0 sang marker trong fixture auth137 và cập nhậtCRC.
Native reader10FE043 → setter13B0D30/index0 → getter tại13AE834 → serializer4CD040 →
writer4CD111/C2S6 cùng marker, đúng thứ tự thời gian; bodyC2S6 chỉ đổi4byte15..18.
Đây là **echo field36 qua cache**; ý nghĩa gốc, TTL hoặc việc dùng như credential/ticket chưa biết.

**`kr_static/confirmed`/emulation riêng:** hai actualframe/native14EF300 +26 logicalreader/string checks
đến cursor43/errors0, haiCRC-flip return7/error10/consumed0 và hai partial return2/consumed0.
Prefixcheckoff/allocator synthetic theo recipe cũ; không xác thực tài khoản.

**Giới hạn:** S2C4successfixture là dữ liệu servergiả lập đã đo parser, không phải kiểm token/password.
Client count1/ASCII7name/currentcopy; count2/optionalstring/provider/encoding rộng chưa runtime.
Một field32bit echo không tự chứng minh quyền sở hữu session hoặc an toàn xác thực. Admission/lease/expiry/revoke/
rate limits là `design`, cần nativepositive/negative riêng trước tích hợp; chưa có Rust implementation.

<a id="auth-two-account"></a>
### Hai identity qua native riêng; wrong-binding TCP riêng — CP221

[Run/recipe/hash](../evidence/login-runtime/README.md#cp221), [metadata](../evidence/login-runtime/two-account.json).
Tái dùng CP214 tokenA/B, CP216 layout6, Profile47 auth_A/B và own_profile typedID/name. Adapter RE mới
trong local chọn account bằng token rồi dùng đúng auth_role/claim_role; bản CP217 giữ nguyên.
Không thêm packet grammar, cryptor hoặc sửa client/production/Rust. Echo/TTL/rate vẫn policy design.

**`kr_runtime/confirmed`:** hai run Wine riêng trên cùng copy4b; mỗi run có service mới chứa hai dummyaccount.
A: ID1/KRLocal; B: ID2/KRGuest. Cảhai token32 qua writer native, authresult0 → typedID/name/C2S6 echo
khớp field36 reader/setter/getter/serializer → actual67 sau admission. Native17 entry marker có nguồn,
không coi số entry là số packet. Hai run/119raw/sourcefrozen/cleanup0; không fault17C7F4C trong cửa sổ đo.
Đây là **hai positive riêng**, chưa hai client KR đồng thời hoặc swap/healthydenial/accountresume.

**`emu_choice/design`, socket Linux thật/driver tổng hợp:** hai source loopback riêng, A đang có owner4,
B có pendinglease; wrongID/name với ticketB hoặc correctBfields từ sourceA bị policy từ chối/EOF nhưng
không tiêu leaseB/xóa ownerA. B sau đó vẫn được nhận bằng captured6 đúng; cảhai owner nhận reply67.
Aidentity6 trên kênh B đã bind không remap owner; B EOF giữ A mở, A EOF cuối xóa session maps.
Fixed public ticket/re-encoded negative/sequence là test controls; không native wrongclaim-health/securityproof.
R1 hai ca dừng tại assertion driver coi map channel lịch sử là map active; service giữ closed events.
R2 kiểm active/session và actualopenA/closedB, không đổi behavior server. R1 source/raw giữ nguyên;
hai R2 mỗi34 scoped checks qua. Bốn TCP run/20raw/sourcefrozen/cleanup0.

Tổng139raw mới; không nâng gate/đóng epic. [EOF receiver blocker](#auth-tcp-ownership) vẫn mở,
healthydenial/twoactualconcurrent/swap/sessionexpiry/restartfence/DB/fullauth chưa đủ bằng chứng.
[Unknown/phép thử](../RESEARCH.md#rust-service-gaps), [thứ tự](../PLAN.md#current-order). CP220 complete/AUD064
không sửa; giao CP220–221 cùng PR292, kiểm lại exacthead mới; audit kếCP224. Internet thật chưa kiểm.

<a id="auth-two-actual"></a>
### Hai KR cùng service: identity/session riêng và duplicate account — CP222

[Recipe/run/hash](../evidence/login-runtime/README.md#cp222), [metadata](../evidence/login-runtime/two-actual.json).
Reuse CP197/200 direct virtualTCP/ownedstartup, CP221 account/token/layout6/nativeprobes; không mở SDK/VM.
Controller mới chỉ hai client, token argv riêng, không chờ tạo phòng/mở4guest. A/B prefix/display/netns và
CA/wineserver riêng; bridge root10.77.0.1, A.2/B.3, không route ra ngoài/TCP proxy/NAT/UDPimpairment.
Profile copy bỏ đúng `replacement_slot` do API admission_ips không hỗ trợ kết hợp này;47fixture wire/pin
không đổi, profile nguồn giữ nguyên. Không gameplay/room/replacement acceptance.

**Correction phạm vi suffix:** R1 hai token đúng nhưng CP217 policy hash nguyên suffix25 nên từ chối
`unsupported_login_suffix`; R2 diagnostic giữ denial và đo native writer/cursor. Cached builder13AE3BD
và native args/cursor xác nhận25byte = block16 + u32 + u32 + u8; cursor36→52→56→60 ở token32.
Cả A/B block16 khác nhau/biến đổi giữa run, semantics còn unknown; u32 đầu0, u32 kế có bytes IPv4 nguồn
đã quan sát, u8 cuối0. Full capture khớp `_encode_bytes(block16)+pack_fields([u32,u32,u8])` có sẵn.
Đây là layout/quan sát, không credential/deviceidentity/ý nghĩa server gốc. Suffix trùng ở CP214–221
chỉ trong các run đó; record cũ bất biến. R1/R2 failed admission giữ raw/limits, không gọi chúng success.

**`emu_choice/design` parser profile mới:** chỉ count1/token32/body61; opaque16 giữ metadata/hash,
không dùng authenticate; nhánh0/sourceIPv4/0 được kiểm bounds rồi normalize cho private policy cũ.
Token verify/account/oneuselease/owner vẫn riêng; IP client chỉ consistency với peer server quan sát,
không trở thành bằng chứng chủ account.10 policychecks gồm malformed/count/flags/IP/tokenfail tách nativeproof.
Không sửa CP217/221 code cũ; source/config/input frozen từng run.

**`kr_runtime/confirmed` R3:** hai actualKR Wine cùng service, tám TCP từ hai IP/stage1..4; native writer32/
auth0/field36chain/C2S6 đúng ID1-KRLocal và ID2-KRGuest/echo rồi actual67 mỗi bên. Hai CA/hai wineserver/
hai netns, PCAP TCP/directendpoint/sequence và hai sessionowner ở snapshot **trước cleanup** nối được;
cảhai giữ qua8s+ sau khi đủ nativeproof. Cleanup mới xóa sessions; không đọc snapshot cuối thành noowner.
R3=40.781146896s/100raw. Không suy TCPaccept/alive/serverlog thành nativeprogress.

**R4 duplicate-account control:** giữ service/policy/topology, chỉ tokenB đổi sang publictokenA.
A native0/6/67 vẫn là owner; B native1 từ reply lỗi1 đã đo CP014/217, không channel6/67 trong cửa sổ đo;
không session thứ hai, hai client không fault trong hold8s+. R4=40.251220213s/99raw/cleanup0.
Đây là loginnegative hẹp, **không** channelEOF healthydenial/45snegative/generalretry/expiry/revoke.

Tổng498raw bốn run đã hash/seal/cleanup; không fullauth/sessionexpiry/restartfence/DB/API/Rust/gate mới.
Hai account positive chưa đảo account giữa endpoint, chưa native wrong-channel swap. [Unknown](../RESEARCH.md#rust-service-gaps),
[thứ tự](../PLAN.md#current-order). CP220–221 đã merge PR292/f5caefb; AUD064qua220/kế224.
EOF receiver blocker giữ nguyên; Internet thật chưa kiểm.

<a id="auth-lease-eof-blocker"></a>
### Lease expired/revoked: từ chối được, lifecycle client lỗi — CP218

[Hai run/raw/hash](../evidence/login-runtime/README.md#cp218), [metadata](../evidence/login-runtime/lease-controls.json).
Reuse nguyên adapter/profile/copy/token32/GDB của CP217, chỉ derivedpolicy trigger: pendingTTL1ns hoặc
revoke ngay sau issue/trước gửi authreply. Đây là **`emu_choice/design` test**, không luật TTL/revoke servergốc.

**`kr_runtime/confirmed` phần quan sát:** cảhai nativeauthresult0, actualC2S6/field36echo đúng ticket đã cấp;
adapter từ chối ở pha claim, không tạo session, không gửi channel17 và không cóactual67 trong run.
Nativecontext/capture và policyevents táchnguồn; không suy từ serverlog/alive hoặc harnessexit0.
**Cảhai client SIGSEGV tại17C7F4C, stack122EA12←12541E0 sau EOF khoảng12s.**
Run kết thúc sớm khoảng40.6–40.8s wall, sau đó ownedcleanup0; không phải45s khỏe mạnh.
Vì vậy negative/lifecycle acceptance của admission **chưa đạt**, dù policydeny đã quan sát.

**`kr_static/confirmed` chỉ locator:** return122EA12 có directcall1257360; entry hàm đó nhảy2052077
vào vùng protected, không đọc linearbytes saujump thành semantics. Fault17C7F4C nằm tại field+0C
của RTTIrecord17C7F40, typenameATL CWin32Heap. Shape/name không chứng minh heap/ABI/SDK/Wine
là rootcause; chưa biết nguồn callback/target lỗi. Copy vàmodO khớp vùng đọc.

**Bước tiếp tại CP218 (đã có control/dispatch bổ sung ở [CP219](#auth-eof-dispatch)):** đối chiếu cache CP015channel17-timeout/CP180fatalCRC-EOF/currentcopyEOF,
rồi passive trace đúng callback/args tại1257360 và targetdata trước lỗi, hoặc matchedEOF control.
Chỉ thử gracefulerrorreply/binding/patchcopy khi có locator vàcontrol, không đoán opcode VN hoặc vá theo tên RTTI.
CP217valid/badtoken proof giữ scope riêng; không fullauth/sessionexpiry/restartfence/DB/Rust/gate mới.

<a id="auth-eof-dispatch"></a>
### EOF trước channel17: control và nguồn target gọi lỗi — CP219

[Bốn run/recipe/hash](../evidence/login-runtime/README.md#cp219), [metadata](../evidence/login-runtime/eof-controls.json).
Tái dùng CP218 adapter/Profile47/copy/token và cache CP015 timeout/CP180 EOF; không viết cryptor,
đổi source trong run hoặc mở đồ thị SDK. Chỉ local control và probe thụ động trên bản copy offline.

**`kr_runtime/confirmed`:** lease 10s được claim hợp lệ rồi intentional close trước17 vẫn fault;
C2S6 của control này trùng byte với ca expired CP218. Baseline auth field36=0, không admission policy,
cũng actualauth0/actual6echo0 rồi intentional close và cùng fault17C7F4C. Vì vậy expiry/revoke và
nonzero field36 **không cần thiết để tái hiện** trong các recipe đã đo. Zero control đổi adapter/authport;
không coi mọi wire/input trùng hoặc kết luận nguyên nhân duy nhất là EOF. Bốn run/188raw đều seal,
source frozen/cleanup0; khoảng39–40s kết thúc sớm, **native lifecycle vẫn FAIL**.

**Nguồn dispatch đã đo:** stack thô tại fault có top return12578AF, trong khi backtrace bỏ qua frame này.
Ba snapshot sau native send6 xác nhận `this` → vtable16ECAA8 → slot+50=17C7F40 → local `[ebp-60]`
→ `call [ebp-60]` tại12578AC. FaultEIP17C7F4C nằm12byte trong vùng target; không gọi EIP là target đầu.
Native bytes tại callsite khớp modO và copy. Runtime vtable96byte khớp copy; RTTI của actual`this` là
`CObject`, khác tên `CWin32Heap` ở RTTI target. Đây là chuỗi lấy operand/gọi lỗi, **chưa biết vì sao
receiver này được chọn**, lifetime sai hay callback nào đáng ra phải nhận. Không vá vtable/RTTI theo tên.

Các callback snapshot cũ ở122E9FA/FF/122EA0D đều **trước auth reply**, không chứng minh target trước lỗi.
Probe1257360 không hit sau send6 trong hai run mới dù marker send6 có; entry có protectedjump, không
suy đường nội bộ từ linear protected bytes. Context tĩnh trước12578AC chỉ candidate cho getter13AEBE0;
cần xác nhận native receiver selection trước khi giải nghĩa. [Câu hỏi/phép thử](../RESEARCH.md#rust-service-gaps),
[thứ tự duy nhất](../PLAN.md#current-order). CP217–218 complete giữ nguyên; PR290 đã mergef9f48d2.
Không nâng gate, fullauth hoặc production/Rust; Internet thật chưa kiểm giữ backlog riêng.

<a id="auth-tcp-ownership"></a>
### Ownership qua kernel TCP; receiver EOF còn blocked — CP220

[Recipe/run/hash](../evidence/login-runtime/README.md#cp220), [metadata](../evidence/login-runtime/transport-controls.json).

**`emu_choice/design`, integration bằng socket Linux thật:** giữ nguyên private adapter/policy CP217,
Profile47 và frame native đã lưu. Driver tổng hợp chạy trong user/net namespace chỉ có loopback,
không chạy client KR. Cấp fixed public ticket đúng echo trong captured6 để replay nguyên frame qua TCP;
duplicate6 và request67 sau duplicate chỉ sửa sequence bằng encoder đã có, không coi encoder là oracle.
R2 đo được: duplicate6 trên kênh đã nhận không thu hồi owner; replay nguyên captured6 qua kết nối mới
bị EOF, không thay/xóa owner; owner gốc vẫn nhận reply67 thực qua TCP; owner EOF xóa active/session/
connection maps. Đây là kiểm transport adapter, **không native client khỏe mạnh sau từ chối**.
Stale EOF của một phiên từng được nhận sau freshlogin mới chưa được đo; không ghép với policy24 cũ.

R1 dừng tại assertion do **driver** đọc typed u32 như raw little-endian; R2 dùng `codec.read_fields`
hiện có. Giữ nguyên run/source R1, không đổi contract hoặc gọi đây là lỗi KR. Hai run kernel TCP/
8 raw đã seal; R2 có22 scoped checks, không fullauth/security/entropy/restartfence proof.

**`kr_static/confirmed` prefix getter và `kr_runtime/confirmed` phần đã quan sát:** getter13AEBE0
đọc `context+0x48`, nhưng bốn return đã bắt là null ở caller khác; nhánh store125783B không hit trước
fault trong run. Không gán null getter là crash receiver. Watch local12D434 chỉ có armed marker,
không write snapshot/stat; Wine kết thúc với privileged instruction17C7F4C, GDB sau đó không còn
register. Phép watch **inconclusive**, không chứng minh không ghi hoặc remote watch được hỗ trợ.
Hai run native/94 raw seal/cleanup0, lifecycle vẫn FAIL. Tìm bounded write-encoding trong vùng cache
chỉ thấy candidate125783B đã biết; không phải proof thiếu write hoặc semantics protected VM.

**Điểm dừng nhánh receiver:** chưa có observable chọn receiver đủ để sửa, không lặp broad getter/
hardwarewatch hoặc mở protected VM/SDK graph. Giữ [chuỗi dispatch CP219](#auth-eof-dispatch) và blocker;
chỉ mở lại khi có cạnh/recipe lỗi native đã đo mới. [Phép RE độc lập](../RESEARCH.md#rust-service-gaps),
[thứ tự](../PLAN.md#current-order). CP219 đã merge PR291/09435d9. Không production/Rust integration,
không đóng #14/#285/gate; audit AUD064 qua220, kếCP224. Internet thật chưa kiểm.

<a id="auth-session-lifecycle"></a>
### Account đổi endpoint; lỗi sau17 và ranh giới retry — CP223

[Bảy run/raw/hash](../evidence/login-runtime/README.md#cp223), [metadata](../evidence/login-runtime/session-lifecycle.json).
Không lặp CP222 positive/duplicate hoặc EOF trước17. Phép đo tiếp từ nguồn/caller cụ thể; không SDK/VMgraph.

**`kr_runtime/confirmed` endpoint permutation:** cùng privatepolicy/profile, chỉ đảo token hợp lệ giữa
hai endpoint. Source.2 nhận accountB/ID2-KRGuest trước; source.3 nhận accountA/ID1-KRLocal sau;
nativeauth0/typed6/echo/actual67/hai session/PCAP/8s+hold nối được. Identity theo verifiedaccount ở hai
mapping đã đo, không theo fixedIP-label hay thứ tự nhận. Đây là validaccountremap, **không** rejected
wrong-channel-swap/securityproof. Run đã seal sauCP222, giờ giao batch này; không sửa recordCP222.

**Termination test `emu_choice/design`:** đóng Bchannel sau khi cảhai actual67 và chờ5s, release đúng
Bsession, Aowner giữ. Không gọi đây là luật TTL/revoke gốc hoặc implementation sessionexpiry.
Sau17, B có UI connectionlost/errorcode10 và mở connect mới tới10.77.0.1:3848; không SIGSEGV trong
cửa sổ20/30s hai ca đầu. Không suy “healthy/recovered” từ alive/FIN/UI: nativefreshlogin chưa qua.
Khác EOF trước17 bị17C7F4C tạiCP218–220; blocker đó giữ nguyên, không tổng quát hóa qua mọi pha.

**Prototype admission còn thiếu lifecycle:** ca đầu chặn Bconnect mới bởi budget4/source. Derivedpolicy
reset counterB sau oldsession release thì còn guard ordinal>8 `replacement_not_ready` chặn; guard cũ giữ
bất biến. Adapter riêng cho đúng Bcycle2 (A/unknownsource vẫn fenced, trần12) nhận connection9 nhưng
không C2S byte/auth0 lần2/6/67 mới. Không retag theo port hoặc giả token/accountresume thành công.
Các failedretry run giữ raw; one-timecounter/guard và optional ordinarydialogReturn đều là testpolicy/input.

**Probe cuối khoanh điểm dừng:** copiedIATconnect16EA778/recv16EA744/AsyncSelect16EA740 được quan sát
thụ động. NativeB newconnect caller14F489A←13AEEE5←13AD6C5←1255C0B←1255A69←125BE47;
recv cùng socket trả15byte đúng bootstrap fixture. Framer14EF300 mode0/selector0 consume15/ret0,
sequence40→43/available0; sau đó AsyncSelect message400/mask21. Đây là bytes/framer success, chưa
consumer/validation success: không secondauth/C2S6/67/newsession. Tiếp đúng hello consumer13AEFE0/
validationresult theo caller/cache, không lặp socketaccept/recv hoặc mở graph protected/SDK.

**Corrections giữ provenance:** ảnh `screen-10` R1 được đọc nhầm là sauEOF nhưng timestamp trước close;
ảnh cuối có modal. Report đầu bị pin làm input R2 giữ nguyên, thêm correction, raw không sửa.
Thao tác dự kiến create trong R2 bị errorUI chặn; không send81, không roomacceptance hoặc causalityclaim.
Probe connect đầu đặt watchrecv từ lần5 nhưng breakpoint start sau2bootstrap nên newconnect là lần3;
thiếu snapshot không là no-read. Probe mới chọn actualendpoint và giữ đầy đủ cả hai report.

Tổng1108raw/bảy run/sourcefrozen/ownedcleanup; nativeendpoint-remap positive tách retry **FAIL**.
Chưa genericfreshlogin/accountresume/sessionexpiry/revocation/restartfence/DB/API/Rust/fullauth/gate mới.
[Unknown](../RESEARCH.md#rust-service-gaps), [thứ tự](../PLAN.md#current-order); CP222 mergePR293/4bcabd8,
AUD064qua220/audit kếCP224. Internet thật chưa kiểm.

<a id="rust-account-native-lifecycle"></a>
## Account native negative/lifecycle — CP255, không đổi server CP254

[Metadata/limits](../evidence/rust-service/account-native-lifecycle.json), [raw](../evidence/rust-service/README.md#cp255),
[thứ tự](../PLAN.md#current-order). Nguồn **kr_runtime/confirmed** cho các quan sát hẹp dưới; debugger control là **design**.
Tái dùng CP216 reader10FE043→setter13B0D30/index0/caller10FE05F/value[ESP+8]→getter13AE834→serializer4CD040→wire6;
CP254 Rustbinary/copy4b/Wine10.0-20260425/3netns/directTCP/privateTMPFS/PCAP/UI/cursor/argv. Không hàm binary mới/SDKgraph.
Skill REA42 dùng phương pháp/evidence boundary; REA session metadata không là native Evidence, không mở/sửa DBnguồn.

**Wrong pending correlator:** A send1 gate trướcauth≤30s, B send6 giữ≤6s trong handoff10s. NativeBsetter ghi echo thực,
A setter thay đúng4byte argument sangB echo, giữ nguyên AID/name/authresult. Nativegetter/serializer/C2S6 corroborate,
fullPCAP AID1/KRLocal/echoB→chỉhello/no17-no291/noAchannelgrant; Bpending không mất, actualB6đúng→17/291/render/8shold.
Compoundpeer/ID-name/echo guards, không chứng minh mỗi nhánh độc lập. Không forceauthstate, gọi function hoặc đổi fileclient.
78raw41,686s/13exactreply; A noticeerror10 ởloginscene, B lobbypanelKRGuest. Không đãactiveBmatch hoặc graceful recovery.

**Revoke khi active:** cảhai đãnative137/291/render trước, realauditedCLI revokeaccountA; worker periodicinspect đóng
đúng currentchannelA với InvalidSession. SQLite revocation/audit và PCAPserverFIN đối chiếu; delay sauCLIreturn0,718181637s.
Bcurrentchannel/render/DBphiên giữ≥8s.79raw44,268s/18exactreply. UIA KRLocal còncachepanel+noticeerror10, không coi là logoutUIcomplete.

**Tự hết hạn:** nativeAbackendSession absoluteTTL60s/idleNULL, native2profiles trướcexpiry; không sửa DB để ép hết hạn.
PCAPserverFIN0,484608s sauexpires_at, SQLiteAexpired/revokedNULL/noauditrevoke, Bsession1200s/current/render/channel≥8s.
123raw70,445s/18exactreply. Đây là expiry backend session đang bound, không pendingfrontend/handoff/idleTTL.

Cả3run sourcefrozen/exit0/seal/cleanup0, fulltuple/CRC/seq/SHA và native137/291/producer/control/CAargv đối chiếu,
6ảnh cuối đã xem. A không có terminalsignal trong boundedhold, noticeerror10 chưa được đóng; không kiểm ordinaryReturn,
sameprocessretry/freshlogin/resume. Initialchecker syntax lỗi được giữ rồi sửa checker riêng; không rerun native vì checker.
Delay là số đoquietLAN/TMPFS, không load/SLO/alltopology. PrivateRAMDB/exporthash không Btrfsfix/crashdurability/backuprestore.

M5 còn freshlogin/replay/projector tổngquát và accountroom-game integration; fixed2zero-profile không fullaccountservice.
CP220/224 recovery blockers giữ nguyên; không mở SDK VM chỉ để đóng modal. Source/ID catalog giữscope cũ, không gateclosure.
K6c WAN giả lập không đổi; **Internet thật chưa kiểm**.

<a id="rust-account-fresh-login"></a>
## Account fresh process và replay request native — CP256, M5 vẫn mở

[Metadata/hash](../evidence/rust-service/account-fresh-login.json), [raw](../evidence/rust-service/README.md#cp256),
[ưu tiên account xuyên suốt](../evidence/owner-directives/account-basic-end-to-end.json), [PLAN](../PLAN.md#current-order).
Nguồn **kr_runtime/confirmed** cho quan sát hẹp; clock/backend policy **design/design**. Không binary RE mới/VN auth.

Cùng server/DB, hai native account lobby trước. Explicit admin revoke A, dừng đúng WineA/đợi current channel đóng;
CLI login cấp backend session mới, private launcher cấp frontend32 mới, process CA mới nhận auth137/own291128/render KRLocal.
PID/startticks/CAargv mới và namespace đúng đối chiếu; B giữ process/backend session/current channel KRGuest.
Sau fresh A claim, Python trong **cùng netns A** phát lại nguyên first1 và first6 đã bắt từ native A cũ ở chính run này.
PCAP byte SHA giống request cũ: first1 nhận result1/no identity; first6 không17/291/grant; không consume/xóa owner A mới.
A mới/B giữ≥8s sau replay, không FIN/RST trước completion. Replay sender không là native retry/errorUI.
Full PCAP/CRC-seq/hash/3native auth137-291-render/independent argv/SQLite2accounts3sessions/oldA revoked/newA-B valid
và hai ảnh cuối đã xem;122raw70,793556617s/28exactreply/seal/cleanup0/Wine10.0-20260425/copy4b/3netns/TMPFS/no externalroute.
Không fresh server, sameprocessretry, accountresume hoặc matchrestore; revoke/stop là setup rõ ràng, không graceful logout.

**Clock discrepancy và sửa:** Native R1/68raw42,544s sealed/cleanup0 dừng ở CLI re-login `ClockRollback`, không có fresh success.
CLI từng giữ UTC trước KDF; worker inspect B có thể tiến DB high-water trong lúc hash, làm timestamp publication cũ bị reject.
`Store::login_current` lấy UTC **sau khi giữ writer transaction**, cả initial observation và publication sau KDF/lock wait;
TTL/issued_at tính từ publication. Vẫn durable high-water/clock rollback guard, credential-version/PHC/ban/one-active/entropy fences.
Không clamp timestamp về max(last_utc,now), không sửa DB để test pass. API explicit supplied-time/worker login giữ semantics cũ.
CLI chỉ command login chọn đường current; concurrent create/reset/admin clock contention chưa nghiệm thu từ ca này.
Hai regression kiểm publication sau connection khác advance clock, TTL/identity/new token và actual backward/reset credential fences;
84workspace tests/fmt/clippy/lockedbuild và33CLIprocess/2persistentaccounts/9auditedmutations/cleanup0 đạt. Initial compile mismatch/private test seam được giữ trước runtimeR2.

**Giới hạn hiện hành:** account mode vẫn chỉ hai known zero profiles/sảnh, reject room/Battle. Theo owner mới, bước kế là nối
account→phòng cố định→basic→kết quả/reset→trận tiếp, kiểm identity/transport/membership và disconnect/expiry/revoke/stale fences.
Diagnostic Battle M3 và account lobby không tự hợp thành account Battle. Generic projection/M6inventory/M7multiroom/M8restore riêng;
không đưa productionrestore thành dependency M5. CP220/224 sameprocess recovery blockers giữ nguyên; không SDK/copy retry.
AUD074 quaCP256 là audit theo lịch cũ cuối cùng; [policy mới](../evidence/owner-directives/feature-group-audits.json) từ sauCP256 audit theo nhóm tính năng, không deadline CP260. K6c WAN giả lập; **Internet thật chưa kiểm**.

<a id="auth-retry-validation"></a>
### Retry qua hello/validation nhưng fatal trước token builder — CP224

[Ba run/raw/hash](../evidence/login-runtime/README.md#cp224), [metadata](../evidence/login-runtime/retry-validation.json).
Tái dùng privatepair/profile/policy/inputCP223, chỉ thêm observer thụ động; không đổi source giữa run.

**`kr_runtime/confirmed`:** sau controlledBclose, run exactsite ghi13AD79C (return13AEFE0),
1255C73 (return13AD6F0),1255A69 (return1255AC0),125BE47 (return1255A30) đềuEAX1,
managerphase2/socket mới. Vì thế điểm chặn nằm **sau** hello/validation đã đo, không phải framer hay
consumer hello chưa thành công. CP223 chỉ biết bytes/framer; đây là refinement mới, không sửa record cũ.
R1 quan sát new3848, R2 new3838 theo actualevent/endpoint; không ép stage theo port hoặc coi boolean
`post_reconnect` là bằng chứng: Ainitial cũng dùng3848. Join theo B/closed_ns và nativecallsite.

Cảba chưa có token count setter/builder sau close trong giới hạn probe, không C2S chunk ở newconnection9,
không secondauth/6/67/newsession; sau validation khoảng6s, nativeGDB dừngSIGABRT rồi sau detach Wine báo
unhandledillegalinstruction02260DF8/ProgramError. Không suy rootcause từ popupLoginError2000:0,
threadstack hoặc địa chỉ protectedcode. A giữ owner/openTCP, không đo gameplayprogress.

**Correction `design/confirmed` chất lượng quan sát:** log `connect-path-r2` đã pinCP223 cũng có
SIGABRT/illegalinstruction; report cũ bỏ sót vì harness chỉ nhận SIGSEGV và bỏ qua Bfailure sauclose.
CP223 vẫn ghi retryFAIL/chưahealthy, nhưng failure phải ghi rõ fatal; raw/seal/record không sửa.
Observer R1 dùng FinishBreakpoint ngayentry trướcprologue, returnlabel bị lệch một caller;
không lấy các label/result đó làm oracle. RunR2 đặt breakpoint đúng bốn returnsite ở trên, không ép return
hoặc gọi hàm native. Giữ cả hai report và correction. Harness tương lai phải nhận mọi fatalstop,
không dùng processalive/khôngSIGSEGV làm healthpass; không cần chạy lại để chứng minh fatal đã có.

462raw/ba run đã hash/sourcefreeze/ownedcleanup; fullretry **FAIL**. Chưa genericfreshlogin/accountresume,
sessionexpiry/revoke/restartfence/DB/API/fullauth/Rust hoặc gate mới. Tiếp [câu hỏi](../RESEARCH.md#rust-service-gaps)
theo [PLAN](../PLAN.md#current-order); không lặp hello/codec/EOF/permutation hoặc mở graphSDK/VM.

**Continuation đo riêng:** fallback Capstone read-only ba range ngắn125BE47/125BEBD/125BF8C, cùng bytes
copiedPE/modelmodO đã pin; không fullfunction/decompile. REA/IDA request timeout450s, export0record,
tiến trình phân tích do run tạo đã cleanup, targethash không đổi; không trình fallback như REAevidence.
Run3 nativeEAX1→125BEBD→1255F80→return125BEC9; localEBP-28=11 nên đi125BF8C là epilogue/ret,
không đi nhánh local==1, không kẹt trong handoff. Ý nghĩa/nguồn discriminator11 **unknown**; không gọi
nó là token count/stage/error cause. Chỉ nguồn biến/caller là câu hỏi tiếp, không quét calleesSDK/VM.
Cảba run vẫn fatal/không auth6-67 mới. CP223 mergePR294/51bbf41; AUD065qua224/kế228. Internet thật chưa kiểm.

<a id="login-retry-codec"></a>
### Login rejection hợp lệ và codec reset khi retry — CP181

**`kr_runtime/confirmed` cho copye9/Wine, một client**. Tái dùng exact fixture CP014
`login-error1-key43.bin`39byte/SHA9aeae7b7937ad0c54d968297c48fffb4d506087f7826b2e9fdde35d84ed3ac19,
native result1 đã đo; không dựng lại layout/opcode từ VN. [Run/controls/hash](../evidence/live-codec-stream/README.md#cp181).

| Route/phép đo | Native observation / phạm vi |
|---|---|
| Firstlogin rejection | `14EF300` consume39/ret0, seq43→46/payload1; `10FEDF1` result1/global0; `11044A7` waitreturn1/global0. Khác fatalCRCcontrol đóngnotice/global1/cleanup, không suy mọi lỗilogin tương đương. |
| Actual UIretry | Đóngnotice rồi confirm: `10FC540` command1002/caller1509531 → `1101E20` → `11049C0` caller1101D1A. Một actualCA LinuxPID và GDBinferior/WinePID01C4 xuyên suốt, không launchCA thứhai/callfunction épstate. |
| Newlogin TCP4 | Nativehello0/0seq40→43, rồi mode2/selector3; nativewriter C2S1/60 resetseq40/43. Ở control r1 stub coi ordinal4làchannel nên không trảauth. Adapter r2 map4làloginA, giữ body175 từ profile: result0, nativeauth137/137/errors54=0/58=0, modalreturn3/global0. |
| Redirect TCP5 | Nativehello0/0seq40→43 lầnthứ3 trongcùngprocess; channel17seq43→46 rồi241/91/14 tiếp46/49/52→55, bufferhết. Đây là codec/sessiontransport retry, không server xácthực cùngaccount hoặc persistenceidentity đã khôi phục. |

Lab adapter retag4/5 là **`design`**, chỉ sau actualr1 chứng minh4 gửi1/60, dùng nguyên47fixtureprofile;
không sửa productionservice/codec, không épreturn/global saufatal hoặc patchDWK. Prefix/cache kế thừa,
bootstrapglobalreset1A966FC=0 giữ recipe cũ, không originalWindows/coldcacheall. Native0/0packetfieldlogger
khác layout nên chỉ dùng socketstate/mode/seq/buffer; mode2nativepacket/cursor có scope trên.
Harnessturnày WINEDEBUGkhôngbậtwinsock: không có per-connectAPIdump. Attribution dựa initCA/
GDBinferior/actualLinuxPID duy nhất, exclusiveclientnamespace, nativeframeorder và servicebytes;
analyzer đầu đòiAPItrace không có đã sửa phạmvi, không bịa API hoặc gọi đóclientfailure.

CP010cleanup/CP012modal+UI/CP014result1, CP057modelboundary và CP180actualstream được tái dùng;
không chạy lại76split/primitive/gameplay. Raw lỗiHWwatchpoint, control stubmisclassification và
fatalsequence giữ riêng. Unknownfullgrammar/modes/prefix/headerauthority/FFFFFFFFallocation/
cache/accountresume không tự thànhconfirmed; [nghiệm thu K2profile](../PLAN.md#k2-acceptance) là quyếtđịnh
thủ công riêng. Tại CP181 K3 còn ở RESEARCH; CP186 đã nghiệm thu localprofile, thứ tự hiện hành ở PLAN.

<a id="auth-fresh-process"></a>
### Fresh login bằng process mới sau session cũ — CP225

[Recipe/raw/hash](../evidence/login-runtime/README.md#cp225), [metadata](../evidence/login-runtime/fresh-process.json).
Không sửa sameprocessfatalCP224; tách rõ freshserver/freshlogin/sameprocessreconnect/accountresume/matchrestore.

**`kr_runtime/confirmed` nguồn discriminator:** native125BD10 errorlocal10→getter12562C0 trên cùngobject
trả11 tại125BDE0→storeEBP-28=11 tại125BDE3→setter12562F0(request1)→return125BDF5.
Đây là prior-value snapshot trước connect/validation, không token count/framer state. Setterrequest không
là readback1. Staticgetter đọc wrapperobject+14, chưa giải wrapperencoding; họ selector đã nối stage11/
CLobbyStage ở nguồn cũ nhưng không gán nó làm nguyên nhân fatal hoặc chứng minh fix bằng forcedstate.
Các logCP223 retry-ack/connect/connect-r2 đều có fatal; ba EOF/budget đầu không có literalmarker trong
boundedscan, không suy khỏe từ không tìm thấy. Giữ mọi record/raw cũ.

**Freshprocess test `emu_choice/design`:** sau initialABnative0/67 và controlledBclose, controller chờ3s,
kiểm oldB chưa tiêu transportcycle2, cleanup đúng Bprefix/oldPID rồi mở winedbg/CA mới cùngcopy/prefix/token.
Aprocess/netns/owner giữ nguyên; không nativefunctioncall/resetstate. Service/privatepolicy/profile47pins/
credential/lease/ticketseen giữ nguyên, Bcycle2/12transportcap đã khai designCP223. Không luật logout/TTL gốc.

**Dương tính:** newB PID650 khácold309, A159 giữ; nativeauth0, ID2/KRGuest/field36chain→typedC2S6echo
đúng lease mới→actual67, channel12/session mới khácold8, oldsession đã bỏ, Achannel6/session không đổi.
Hai client/two wineserver/twoendpointnetns/directbridge/noexternalroute/PCAP joins/8s+hold đã kiểm.
ẢnhBfresh cuối là sảnh KR. Không suy nativegameplayprogress của A từ owner/openTCP/PID.

**Âm tính riêng:** initialB vẫn token đúng, **chỉ** tokenBfresh đổi thành32 ký tự sai. Nativewriter/frame
khớp input32; service credentialsdeny→native result1, không channel17/C2S6/67 hoặc Blease/session trong45s+;
A giữ owner/PID/openTCP. Ảnh KRlogin có credential-errornotice, không WineProgramError; fatalmarkerA/Bfresh
không có trong boundedhold. Đây là knownresult1negative của freshlogin, không healthyEOFdenial/swap/securityproof.

3relatedruns595raw/frozenpreflight/ownedcleanup: getterorigin154, freshpositive181, negative260.
Chưa fullauth/securefrontend/accountDB/sessionexpiry/revoke/restartfence/automaticresume/matchrestore/Rust/gate.
[PLAN](../PLAN.md#current-order) chuyển khoảng trống phòng tổng quát cho M2; không tiếp quét SDK/VM hoặc
lặp failedretry khi thiếu input/observable mới. CP224mergePR295/928f2fb; AUD065qua224/kế228. Internet thật chưa kiểm.

<a id="rust-account-basic"></a>
## Account-owned phòng cố định/basic xuyên suốt — CP258, M5 còn mở

[Metadata/hash](../evidence/rust-service/account-basic.json), [raw/recipe](../evidence/rust-service/README.md#cp258),
[owner scope](../evidence/owner-directives/account-basic-end-to-end.json), [PLAN](../PLAN.md#current-order).
Nguồn **kr_runtime/confirmed** cho các đường đã đo; authority/backendfence/timer vẫn **design/design** như M3/CP254.
Không RE lại binary/codec hoặc port VN. Reuse CP235–242 room/basic/reset/EOF; CP253–256 account/private launcher/ownership.

`account_admission` opt-in nay ghép `fixed_room` cho đúnghai knownzero profiles ID1/KRLocal vàID2/KRGuest/version1.
Backend Channel xác nhận quyền rồi chọn ID/name; không dùng arrival/IP làm principal. Riêng roommode còn kiểm identity↔peer
mapping đã configure để endpoint P2P cố định đúng; unknownpeer/profile/version/roomcapacity lỗi failclosed, khôngfallback diagnostic.
Native account cấpfrontend32 riêng từbackend64, actual1→137/field36→actual6/67→own291 render→roomSession cùng connection.
Roomslot/membershipvisit/round/recipient queue và UDPkey dùng cơ chế M3 có nguồn, không thêm wireepoch hoặc sessionkey giả.

Authorize qua boundedworker trước roomcommand, tick và duebatch; sau await recheck Session connection/membership/round,
không giữ runtime mutex qua DBawait. Command admission renewidle trướcroomvalidation; background/tick/batchinspect không renew.
DB/queue/expired/revoked failure trả lỗi→drop ownedSession detach/queue cleanup; drop bridge generation cũ không xóa owner mới.
Đây là check tại transaction boundary, khôngatomicDB+socketwrite: bytes đã ghi trướcrevoke khôngthu hồi được; khôngSLO
instantrevocation hoặc continuousglobalcredentialclaim. Opaque87 exactbody chỉ accountLobby và Session::in_lobby, khôngroom/Battle.

**Native positive:** realCLI2accounts/session→privateAPIfrontend→actualnativeauth137/own291128/render/CAargv/SQLiteownership,
room ownslot/ID/name/native106, hostBack/promotion/rejoin/Ready/loadingbarrier; cùngchannels qua haioppositeoutcomes vàStart3.
Nativepre175 `[1,0]`→`[0,1]`, rank2/1 đảo,175302outer/292inner/1803/refresh/keys12346–48/clock01:53;6ảnh đã xem.
68raw203,750626716s/1.466TCP/106exactreplies/1.266copiednativegetter checks; sourcefrozen/seal/cleanup0.

**Active host lifecycle:** B đã cùngaccount-ownedBattle, actualB118→159/nativeentity39 trướcfuse6, rồi realadminrevoke,
naturalbackendabsoluteTTL120s/idleNULL/noDBmutation hoặc ownedCA-BSIGKILL cóPID/startticks/namespace/map/hash guard.
ExactcurrentBchannelFIN/RST/SQLite-audit đốichiếu; A còncurrentaccount/channel→forfeit175/180→107ID2clear→110chief/Room≥4s,
không158 muộn hoặcfake19. Mỗi48raw/60exactreply: revoke92,806s/724TCP, expiry148,216s/820TCP, disconnect91,026s/708TCP.
Mỗi14copiednativegetter checks. Nativechiefgetter saureturn hếtbudget20/unobserved; claim full110+UIchief riêng.
**B saurevoke/expiry có gameSessionError(2000:0) và Winefatal**: không claimgracefullogout/modalclose/retry hoặcpeerBsurvival.

**Manualfresh B + vòng mới:** reuse revoke/pendingcleanup trênA; dừng đúng WineB, cấp backend mới/privatefrontend mới,
process B mới cùngID/profile/newchannel→rejoinslot/Ready→AStart→nativeAclock2/Bfreshclock1/currentkey12347, khôngold158/175180.
ActualoldB UDP frame từ chínhrun/epoch12346 được replay nguyênEthernet ởveth: nativeingress/queue/consumerkey56≠57→key_drop,
không apply đúngoldpayload; currentfreshBRightinput key57→nativeapply/xprogress. A/newB không FIN/RST trướccompletion,
SQLite2accounts3sessions/oldBrevoked/newB-Avalid/CAargv/PIDstartticks và2ảnhBattle01:45 kiểm.68raw172,400479137s/89exactreply/
146copiedgetter checks/cleanup0. Manualfreshprocess là flow mới; không sameprocessretry/automaticresume/matchrestore.
Keylow8wrap/compromisedcurrentpeer/spoofing không được giải từca này; UDP vẫn legacyP2P khôngstrongaccountauth.

**Kiểm tích hợp:**84workspace/fmt/clippy/lockedbuild,59account-basicTCP/realSQLite/privateAPI gồm oldchannel6 trênroguetransport
không xóa freshowner, rejoinfree-slot/survivormembership/currentnextkey và wrongslot/duplicate/stalelife/deadowner/bombfences;
16lobbyregression đạt. Synthetic/body-encoder khôngnativeoracle; proofnativetừngcase riêng ởtrên.
**Failures:** R1eagerroomprobe27raw36,128s Aillegalinstruction02260DF8/Bbefore67,17frameSHA giốngCP254positive/no servererror,
khôngcausalSDK/codecclaim. R2preparationGDBPythonSyntaxError/no runtime giữ; R3boundedcontrol cài roomobservers sauown291end
D30122/separateglobals trênCP254lobbyrecipe→positive, sameRust/clientfiles. Không nhậnđã tìmrootcause. Freshpostrunchecker syntax
lỗi giữ/newversionfix khôngnative rerun. Tấtcảrun đãseal/cleanup trướcmọi source/configđổi.

Nhóm **account-fixed2-basic-end-to-end** đạt scope trên và auditAUD076 theo policy mới; **M5 tổngthể vẫnpartial/open**:
generic DB-owned profile projection beyondIDs1/2/version1/name5-7 và nativeDBqueuefault/integration cònthiếu;
affectedclientrecovery cógiớihạn thực. Inventory/progression/reward M6, multiroom M7, restore/productionops M8 riêng.
PrivateTMPFS DB/raw/log/exporthash khôngBtrfs/crashdurability/backuprestore. Wine10.0-20260425/copy4b/3netns/virtualLAN/noexternalroute;
K6cWANgiảlập giữscope. VPS networkpreflight là việc riêng, khôngnativeInternetgame; **Internet thật chưa kiểm**.


<a id="rust-db-account-profiles"></a>
## Hồ sơ account từ DB và kiểm quyền khi DB bận

Nguồn: [M5b](../evidence/rust-service/m5b-dynamic-accounts.json) và
[bổ sung native recovery](../evidence/rust-service/m5b-native-recovery.json); acceptance ở
[PLAN](../PLAN.md#phase1-acceptance). Đây là lớp mở rộng của đường account/basic và phòng managed hiện có.

**Contract và policy được giữ.** `account_admission.profile_mode=database_zero` lấy player ID, wire ID và tên từ DB
sau xác thực session của chính account. Mỗi account mới không cần fixture, sửa config hoặc restart. Auth4, own291
và card106 sinh từ một bộ template zero-profile đã đo; tên nằm ở cả hai vị trí card106, 98 byte hồ sơ ban đầu bằng 0.
Tên ASCII chữ/số/gạch dưới dài 3–15 byte là policy backend; không suy nhánh Unicode, trang bị hoặc profile khác.
Mode `fixed` mặc định giữ fence cũ. `max_sessions` hữu hạn độc lập số fixture. Không dùng IP để chọn identity.

Launcher riêng cấp frontend một lần từ session và player đã kiểm. Địa chỉ nguồn transport và địa chỉ client báo
được lưu riêng theo chính owner; metadata báo khác phải do launcher có session cấp, không lookup một account khác
theo IP. Channel6 vẫn kiểm name/wire ID/echo trước claim. Room card, slot, host, Ready, Start, round và hàng đợi gửi
giữ membership/connection/visit; không thêm alias theo identity hoặc luật theo sức chứa.

**Làn kiểm quyền trong trận.** Khi dùng profile DB, service mở một worker WAL riêng, queue 64 và TTL hàng đợi 1 giây,
tách khỏi queue đăng nhập/admin. Mỗi kiểm vẫn đọc session/account hiện đã commit: revoke, disabled, credential,
profile, boot epoch, absolute expiry và idle deadline. Không dùng grant cache để bỏ qua lỗi DB. Không đợi khóa ghi
để đọc WAL; cập nhật watermark/idle chỉ bỏ qua lỗi writer busy, còn lỗi lưu trữ khác vẫn fail closed. Nếu khóa kéo
dài qua idle deadline thì session vẫn hết hạn, không hồi sinh. Instance profile DB tăng boot epoch trước readiness;
restart là fresh server/fresh login, không account resume hoặc match restore.

Khi queue đăng nhập đầy, queue hết TTL hoặc DB bận, API launcher trả `busy_retry` và reason tương ứng; CLI báo `Busy`.
Đây là lỗi rõ trước khi tạo process KR, không là bằng chứng modal lỗi DB trong KR. Lượt vượt admission của launcher
có thể bị đóng socket; duplicate pending owner vẫn bị từ chối. Client đã chơi tiếp tục kiểm quyền qua làn WAL.
Revoke/expiry/credential/profile/epoch đã commit hoặc lỗi DB thật vẫn tháo owner/generation như trước; không nâng
thành graceful logout/retry. Mật khẩu Argon2id và session hash/revocation tiếp tục thuộc backend công ty.

**Mức chứng minh.** Client KR thật trên Wine đã nhận tên dài 3/10/15 và ID 6/7/8 tạo bằng CLI sau readiness; profile
và roster native khớp DB, tạo/vào phòng rồi chơi hai trận tới kết quả, có reset ở giữa. Một lượt riêng có ba account
chung nguồn TCP, vẫn phân biệt đúng name/ID và membership trong native reader/UI. Đây là source mapping ở LAN cô lập,
không chứng minh Internet hoặc gameplay sau NAT. Trong ca writer lock 25 giây và queue account đầy, cả ba native
consumer tạo entity159 rồi nổ158; UDP thật tiếp tục apply. Rank diễn ra sau khi nhả khóa theo delay của recipe,
rồi reset/ván kế; không tuyên bố rank được đo trong lúc giữ khóa. Login mới lỗi rõ và login fresh thành công sau khi
DB hồi phục. Lượt bổ sung mở process KR thứ tư sau lỗi, nhận đúng tên/profile của account vừa cấp session
trong sảnh. CP284 chỉ có recovery ở CLI/session; quyết định đóng M5b phải đọc cùng bằng chứng native bổ sung này.
TCP wrong-password/ban/duplicate và source tests kiểm fence riêng, không thay oracle KR.

**Giới hạn còn lại.** M5b đạt acceptance giai đoạn 1, không đóng toàn bộ thiết kế M5 rộng. Hồ sơ nhân vật bền vững và
kết quả/thưởng 0 thuộc M6; EXP/tiền/inventory thuộc giai đoạn sau. M8 còn đóng gói, tải, restore và vận hành. Những
lượt recipe lỗi và GDB remote-disconnect lúc teardown giữ nguyên trong evidence. Lượt recovery gặp bounded unmount
busy sau export: 103 file raw được phục hồi nguyên byte chỉ sau khi namespace kết thúc, rồi seal; exit1 và lỗi
helper được giữ nguyên. C đóng debugger sau frame107 đã capture hơn 118ms sau completion. Không gọi teardown sạch.
Shared-NAT relay, reuse/restart và mọi topology Internet không suy từ ca chung nguồn TCP. Python vẫn giữ RE/reference.

<!-- protocol-topic-content-end -->
