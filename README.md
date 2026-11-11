# Boom KR Server

Server giả lập cho **the lastest client Crazy Arcade KR build00967 before end of service**, dựa trên kiến thức từ client
VN VBNB_116. 
Chú ý:
1- dự án không cung cấp client game
2- dự án chứa lượng lớn text và kiến thức RE được viết bằng agents, ko phải con người, nếu đọc không hiểu thì hãy hỏi AI giải thích. 
<img width="1024" height="1230" alt="fixed-contact" src="https://github.com/user-attachments/assets/739b9237-b44a-47f0-87a3-97ccd3c9f669" />

**Thư mục làm việc hiện tại trên host:** `/home/admin/boom/repos/boom-kr-server`.
Docs, tools, code và evidence mới đều ở đây. Không cần mở worktree của repo Rust cũ để làm project này.

```text
boom-kr-server/
  AGENTS.md                 quy tắc của project này
  docs/
    PLAN.md                 lộ trình K0–K7, task và gate
    features.json           chỉ mục cơ chế/ID/dependency/trạng thái có nguồn
    COVERAGE.md             bảng/cây/thẻ tra cứu sinh từ catalog
    PROTOCOL.md             contract KR, ma trận reuse VN→KR và authority
    RESEARCH.md             câu hỏi mở, owner và điều kiện đóng
    REFERENCES.md           nguồn RE cũ, provenance và bản đồ migration
    evidence/               metadata/hash; không chứa binary hoặc decompile
  tools/                    harness, probe và kiểm tra artifact/docs
  rust/                     kr-wire + kr-service; lát cắt Rust từng bước
  server/                   Python serving/RE/reference vẫn giữ
  local/                    raw/copy client/DB/capture; git-ignored
```

**Đích bàn giao giai đoạn 1 (chốt 11/11):** dịch vụ nội bộ bền vững, người chơi qua Internet thật, khoảng 50 người đồng
thời, chế độ thường với nhiều map và item phổ biến. Dài hạn vẫn hướng tới gameplay đầy đủ.
[Đích, việc còn lại và thứ tự](docs/PLAN.md#delivery-target).

## Tra nhanh và trạng thái hiện tại

**CP259: đã đồng bộ main PR329/PR330 và chuyển ưu tiên sang N1 theo chỉ đạo mới.**
[Quyết định/giữ WIP](docs/evidence/owner-directives/pr329-integration-order.json), [kiểm sau gộp](docs/evidence/rust-service/pr329-integration.json).
Fmt, clippy, 86 test Rust, locked build và 59 ca account TCP đạt; 143 frame phòng khớp hash với CP258.
Bằng chứng native CP258 vẫn gắn binary đã đo, chưa tự chứng nhận binary sau `RoomRules`.
WIP projector được giữ trong snapshot/stash, chưa chạy native hoặc merge; thứ tự ở [PLAN](docs/PLAN.md#current-order).
Policy và dynamic group OCI đã tạo, nhưng lần kiểm mới 11/11 12:22 UTC vẫn bị `GetVnic 404 NotAuthorizedOrNotFound`; cần đối chiếu cấu hình group/policy đã lưu. VPS đã build/test Rust ARM64 (86 test), chưa mở game listener hoặc chạy native N1. Chưa có kết luận (a)/(b)/(c). Internet thật chưa kiểm.

### Mốc trước — phạm vi CP258 vẫn giữ nguyên


**CP258: account → phòng cố định → hai trận basic/kết quả/reset → Start3 đã chạy trên hai KR thật.**
[Contract/giới hạn](docs/PROTOCOL.md#rust-account-basic), [raw/hash](docs/evidence/rust-service/README.md#cp258).
Account sở hữu đúng ID/profile/channel/slot; revoke/expiry/disconnect khi bom chờ đưa survivor về phòng, không nổ muộn.
Manualfresh B dùng process/backend/frontend mới vào lại slot và trận mới; UDP nguyên bản cũ bị native key-drop, currentmovement áp dụng.
5run/280raw/375exactTCP/cleanup0;84Rust/59account-basicTCP/16lobbyregression. AffectedB revoke/expiry có lỗi phiên/Winefatal,
**chưa graceful recovery; M5 tổng thể vẫn mở** (generalprofile/nativeDBqueuefault). M6/M7/M8 riêng, không dependency lát cắt này.
Audit nhóm AUD076; không lịch4CP. Internet thật chưa kiểm; VPS preflight riêng chưa là nativeInternetE2E.

### Các mốc trước — trạng thái tại thời điểm ghi

**CP256: fresh login account trên KR thật và chặn replay request cũ đã đạt; M5 vẫn mở.**
[Contract/giới hạn](docs/PROTOCOL.md#rust-account-fresh-login), [raw/hash](docs/evidence/rust-service/README.md#cp256).
A mới/process-session-frontend mới vào đúng KRLocal, B giữ phiên; hai request native cũ phát lại bị từ chối, giữ sảnh8s.
122raw/70,794s/28exactTCP/cleanup0;84Rusttests. Sửa false ClockRollback ở CLI login khi worker tiếp tục kiểm phiên.
**Chưa account → phòng → Battle → kết quả/reset → trận tiếp**; đây là ưu tiên kế theo [chỉ đạo](docs/evidence/owner-directives/account-basic-end-to-end.json).
AUD074 quaCP256; [chỉ đạo mới](docs/evidence/owner-directives/feature-group-audits.json) bỏ lịch4CP, audit khi xong nhóm tính năng. Internet thật chưa kiểm.


**CP255: ba ca account native âm tính/lifecycle đã đạt, M5 vẫn mở.**
[Contract/giới hạn](docs/PROTOCOL.md#rust-account-native-lifecycle), [raw/hash](docs/evidence/rust-service/README.md#cp255).
A dùng correlator đang chờ của B bị từ chối, B vẫn claim đúng; revoke A khi đang ở lobby và phiên A tự hết hạn60s
đều đóng đúng channel A, B giữ8s. Ba run/280raw/49exactTCP/cleanup0; A có notice error10, chưa kiểm đóng modal/retry.
Không đổi server đã nghiệm thu CP254/PR326; chưa generic profile, freshlogin/replay/room-game hoặc fullM5.
Next theo PLAN, audit kếCP256. Internet thật chưa kiểm.


**CP254: hai KR thật đã đăng nhập vào sảnh bằng hai account SQLite trên adapter Rust.**
[Contract/giới hạn](docs/PROTOCOL.md#rust-account-serving), [raw/hash](docs/evidence/rust-service/README.md#cp254).
Native auth137/own291128/ID-name-echo/render/argv đều đối chiếu,18 reply TCP khớp;100raw/41,852s/cleanup0.
82 Rust tests,9ca thư viện/8DB,16ca TCP và11guard startup đạt. Hai profile cố định, chưa room/game hoặc full M5.
Next: native negative/lifecycle theo PLAN; M4 Free vẫn blocked. Internet thật chưa kiểm.


**CP253: M5 đã có thư viện nối session/account vào admission KR; chưa nối listener.**
[Contract/giới hạn](docs/PROTOCOL.md#rust-account-bridge), [raw/hash](docs/evidence/rust-service/README.md#cp253).
Session backend cấp token launcher riêng, dùng một lần; chuyển login → redirect → channel theo owner/generation,
kiểm lại DB khi thực thi. 7 ca thư viện/6 DB thật và 3 body native CP217 phát lại ngoại tuyến đạt; 80 test Rust đạt.
Chưa có native KR chạy qua cầu nối này, chưa full authentication hoặc đóng M5. Service diagnostic giữ hành vi cũ.

CP252 đã merge [PR324](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/324)/`e17b78b`.
PLAN tiếp adapter listener/profile/launcher nhỏ rồi native hai tài khoản, failure/ownership/lifecycle; không làm lại DB/KDF.
M4 vẫn thiếu Free 1v7 native và dừng retry tới khi có observable mới. Audit kế CP256. **Internet thật chưa kiểm**.


**CP252: Rust đã kiểm native visitor thứ chín bị từ chối vào phòng đầy; M4 vẫn mở.**
[Contract và giới hạn](docs/PROTOCOL.md#rust-ninth-admission), [raw/hash](docs/evidence/rust-service/README.md#cp252).
Một client KR thật làm visitor identity9, tám member TCP tổng hợp. Request82 nhận lỗi87 mã2, native đọc1/1/errors0;
đóng notice bằng Return, vẫn ở sảnh, không nhận slot/phòng. 19raw/35,408s/cleanup0;1.384TCP/249reply được đối chiếu.
73 test Rust, 6 ca TCP và4 guard trước bind đạt. Đây không phải chín client native cùng lúc hoặc general admission.

AUD072 sửa claim argv CP250: scalar32 không được launch cho từng role; thực tế cũ là token13 ký tự.
Control mới xác nhận argv32 thật nhưng Free vẫn dừng ở D sau17 trướcphòng (66raw/48,888s); không claim startupfix.
M4 còn Freehaioutcomes/selector0nativepersist. Tiếp M5 worker-to-KR/ownership bridge độc lập theo PLAN;
backend account/SQLite/worker/idleTTL/rate đã có. AUD073qua252/audit kếCP256. Internet thật chưa kiểm.


**CP251: Rust có fixed1v7 và native negative Manner lệch đội; M4 vẫn mở.**
[Contract/giới hạn](docs/PROTOCOL.md#rust-eight-teams), [raw/hash](docs/evidence/rust-service/README.md#cp251).
Tám KR thật/Wine/copy8d: native8người/2đội/selector1→nhánh lệch đội→return7, không104/Battle;
90raw/127,113s/cleanup0/1.513TCP/266reply. Free1v7 đã có code/TCP, nhưng hai lượt native dừng sau17 trướcphòng
(copy4bD/copy8dG); chưa có Freehaioutcomes/selector0nativepersist. Giữ failedraw, không retry vôhạn.
71Rusttests/35eightTCP/8prebind/32fourTCP đạt. Python team reference CP213 đã merge [PR322](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/322).
CP250 merge [PR321](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/321)/`de84217`; Manner4v4 haioutcomes/Start3 giữscope đãđạt.
Tiếp native9admissionbridge/consumer độc lập; Free startup chỉ mở lại khi có observable mới. M5backend đã có/bridge riêng;
AUD071qua248/audit kếCP252. Internet thật chưa kiểm.
CP251 đã merge [PR323](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/323)/`9b37b14`. Nhánh admission9 kế tiếp đã có WIP guard/model tests, chưa integratedTCP/native hoặc PR mới.
AUD072 sửa provenance argv: token32 ở CLI chung chưa tới lệnh launch từng role; [correction](docs/PROTOCOL.md#argv-provenance-correction). Bằng chứng gameplay cũ giữ nguyên, không claim startupfix.


**CP250: tám client KR thật đã chơi qua hai kết quả 4v4 và Start3 trên Rust.**
[Contract/giới hạn](docs/PROTOCOL.md#rust-eight-baseline), [raw/hash](docs/evidence/rust-service/README.md#cp250).
Cùng8/Wine/Manner1/32TCP: loading8,56chiều movement,stat/bom8owner,7peerlife/native1751082-inner1072/1803,
64slotreset/7Ready/Start2–3.156raw/384,236s sealed/cleanup0;5.019TCP/912reply/53.616copiedgetters riêng.
Đã xem24ảnh: cả8bảng điểm hai đội thắng đối nghịch vàStart3clock01:53. Vòng1 Dchết thêm do bom lan, giữ đúng nativevector.
69Rusttests/16eightTCP/5guards; hồi quy2/4 ghi riêng. TMPFSevidence/exporthash, Btrfsflushchưa sửa.
M4 còn fixedFree1v7/selector và Mannernegative cùng native9admissionbridge; M5nativebridge riêng, không fullgame/auth.
CP249 merge [PR320](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/320)/`9fdb57b`; AUD071qua248/kế252. Internet thật chưa kiểm.

Python reference cho fixed4v4/1v7 và selector được giao riêng từ WIP CP213, giữ nguyên source đã đo: [provenance/giới hạn](docs/evidence/eight-player/team-code-delivery.json). Không phải nghiệm thu Rust variants.


**CP249: Rust đã đưa ba survivor về phòng sau guest/host EOF khi bom còn chờ.**
[Contract và giới hạn](docs/PROTOCOL.md#rust-four-abort), [raw/hash](docs/evidence/rust-service/README.md#cp249).
Hai run KR thật/Wine/Manner1: guestD mất→A/B/C về phòng; hostA mất→B/C/D về phòng và B lên chủ.
Native180 đủ3byte, actual585, 107 xóa đúng ID và9slot110/run; không175 hoặc158 muộn. 128raw đã seal/cleanup0.
65Rusttests/20guestTCP/20hostTCP/32normalfourTCP/51twoTCP/6prebind đạt. Log/capture native và hồi quy dùng tmpfs
riêng rồi xuất raw/hash vềlocal; hai lỗi flushBtrfs được giữ, **chưa sửa/đạt vận hành bền vững**.
M4 còn tám client; M5backend đã có/nativebridge riêng. CP248 merge [PR319](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/319)/`b58abf8`.
AUD071 qua248/audit kế252. Internet thật chưa kiểm.

**CP248: cùng bốn KR thật đã qua hai kết quả 2v2 đối nghịch, reset và Start3 trên Rust.**
[Contract/giới hạn](docs/PROTOCOL.md#rust-four-results), [raw/hash](docs/evidence/rust-service/README.md#cp248).
Manner1: nativepre175states đảo `[1,0,1,0]`→`[0,1,0,1]`; cả4 đọc đủ562outer/552inner/16rankrows và1803.
ActualReady/Start2–3/cùng16transports/keys12346–48; sauStart3hold6s/freshall4state0.84raw/236,896s sealed/cleanup0,
1.691TCP/280reply/48copiedgetters riêng.62Rusttests/24fourTCP/51twoTCP/4prebind đạt.
Ảnh B/C/D có bảng điểm hai vòng; Aảnh vẫnBattle, nativeAparser/rank/reset có proof riêng, chưa nhận AUIpanelcapture.
M4 còn scopedhost/guestEOF rồi tám, không fullgame/auth. CP247 merge [PR318](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/318)/`d6cbf9e`.
AUD071 qua248/audit kế252; M5backend đã có/nativebridge riêng. Internet thật chưa kiểm.

**CP247: Rust đã đồng bộ trap/death tới ba peer trên bốn KR thật; M4 còn kết quả/reset.**
[Contract/giới hạn](docs/PROTOCOL.md#rust-four-life), [raw/hash](docs/evidence/rust-service/README.md#cp247).
A/C tự đặt bom→17/19→12recipient170; native remote56→1 và cả4view cùng `[1,0,1,0]`.
76raw/134,675s sealed/cleanup0,874TCP/140reply/16transports;7.240copiedgetters riêng.
60Rusttests/17fourTCP/51twoTCP/3prebind đạt; clear21 đủ3recipient mới complete đã kiểm synthetic, chưa nativeclear21.
KernelcleanupR1 giữ lỗi/PIDgone và verifiercleanup-tail correction giữ raw; không nhận đã fix nguyên nhân shutdown.
CP246 merge [PR317](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/317)/`1b3bddf`.
Tiếp rank4/hai outcome2v2/reset/Start3/abortEOF rồi8 trong PLAN; M5bridge riêng. Audit kếCP248. Internet thật chưa kiểm.

**CP246: bốn owner đã đặt/nổ bom qua Rust trên client KR thật; M4 còn life/result/reset.**
[Contract/giới hạn](docs/PROTOCOL.md#rust-four-owner-bombs), [raw/hash](docs/evidence/rust-service/README.md#cp246).
Manner1/cố định2v2: actual118 từng slot→ID1–4→16native159entity/full39 và16native158/full13/lookup-kind0-cleanup.
Stat capacity1/2/1/1/power1 khớp bốn view, fuse6;7.196copiedgetters.84raw/139,844s sealed/cleanup0,902TCP/144reply.
59Rusttests/14fourTCP/51twoTCP đạt. R1fail16722/73raw giữ; four142/167 chỉ boundedobserve/noeffect.
Ảnh own/peer trap-death còn lệch vì127 chưa forward: chưa sharedlife/outcome/reset/EOF hoặc fullM4/tám.
CP245 đã merge [PR316](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/316)/`d0c2ba3`;
M5backend đã có, nativebridge riêng. AUD070 qua244/audit kế248; Internet thật chưa kiểm.

**CP245: bốn client KR thật đã qua phòng/loading và movement trên Rust; M4 còn battle.**
[Contract/giới hạn](docs/PROTOCOL.md#rust-four-startup), [raw/hash](docs/evidence/rust-service/README.md#cp245).
Run Manner1/đội cố định2v2: bốn ID/slot/tên đúng, ba ACK chưa GO, ACK thứ tư mới clock/spawn;
12 hướng remote current progress và7.128 copied getters. 64raw/105,414s sealed/cleanup0,698TCP/112reply.
58Rusttests,7fourTCP/51twoTCP đạt; hai native failure39+43raw giữ nguyên. Chưa bom/life/result/reset/EOF bốn người.
M5 backend account/SQLite/worker/idleTTL/rate đã merge [PR315](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/315)/`e601ca7`;
chưa KR account bridge/ownership. AUD070 qua244, audit kế248; Internet thật chưa kiểm.

**CP244: M5 backend có idle TTL và rate limit; chưa nối account vào client KR.**
[Contract](docs/PROTOCOL.md#rust-account-idle-rate), [raw/hash](docs/evidence/rust-service/README.md#cp244).
Schema2 migration giữ mọi hàng/cột cũ trên5 bản sao DB CP243/10lần mở; session cũ giữ absoluteTTL.
Session mới opt-in idleTTL, chỉ own-player authorize gia hạn; rate process-local dùng chung worker handles.
55Rusttests,7worker cases/7DBs; accountCLI regression33process. M5 vẫn partial: nativeKRbridge/transportownership chưa đủ.
CP243 merge [PR314](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/314)/`dc71e33`; M3 scoped đạt CP242.
AUD070 quaCP244, audit kếCP248. PLAN chuyển M4 bốn rồi tám, bridge M5 riêng. Internet thật chưa kiểm.

**CP243: M5 có worker DB/hash với queue hữu hạn, chưa nối đăng nhập KR.**
[Contract/giới hạn](docs/PROTOCOL.md#rust-account-worker), [raw/hash](docs/evidence/rust-service/README.md#cp243).
Tái dùng backend CP236: SQLite/KDF trên thread riêng, cancel/expiry khi chờ, handoff login và thu hồi đúng token
nếu caller không nhận. Lỗi audit thu hồi đóng worker/giữ rooterror; không giả cleanup đã thành công.
5 ca worker/5 SQLite DB giữ lại, toàn workspace49 tests; chưa idleTTL/rate/native ownership bridge, M5 vẫn partial.
M3 basic hai client đã nghiệm thu [CP242](docs/PROTOCOL.md#rust-m3-acceptance)/[PR313](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/313),
merge `da1b925`. Tiếp idleTTL/rate M5 rồi M4 theo PLAN; audit kếCP244. Internet thật chưa kiểm.

**CP242: M3 đạt phạm vi Battle basic hai client KR trên Rust/Wine/LAN giả lập.**
[Acceptance thủ công](docs/PROTOCOL.md#rust-m3-acceptance), [raw/hash](docs/evidence/rust-service/README.md#cp242).
Hai vòng nhặt lại ba item/reset, exact UDP cũ bị native key-drop rồi current movement apply; Start3/hold6s.
84 raw/306,382s sealed/cleanup0, 1.006 TCP/126 reply, 4.746 copied getters; 44 Rust tests/51 TCP đạt.
Đối nghịch outcome lấy CP240, hostEOF lấy CP241; không fullgame/auth/production. Logger nay giữ lỗi sink gốc,
flush stall còn chưa rõ nguyên nhân. M5 backend đã có CP236; tiếp worker/idleTTL/rate độc lập trước M4.
CP241 merge [PR312](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/312)/`08a9a0d`.
Audit kếCP244. Internet thật chưa kiểm.

**CP241: Rust đã đưa survivor về phòng sau host EOF khi bom đang chờ.**
[Contract](docs/PROTOCOL.md#rust-return-cleanup), [raw/hash](docs/evidence/rust-service/README.md#cp241).
B host đặt bom rồi dừng đúng PID đã kiểm; A nhận forfeit175→180→107 xóa ID2→110/chief và UI phòng,
không158 qua fuse+4s. 48 raw/89.914s sealed/cleanup0; 41Rusttests/50TCP đạt.
M3 còn item consumed-reset/contest, exact UDP cũ + current positive và queue controls; không fullservice.
CP240 merge [PR311](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/311)/`c7dd615`.
M5 SQLite/account backend đã có CP236, native bridge còn riêng. Audit kếCP244. Internet thật chưa kiểm.

**CP240: Rust đã chạy hai kết quả đối nghịch, về phòng và Start3 trên hai client KR thật.**
[Contract/giới hạn](docs/PROTOCOL.md#rust-life-results), [raw/hash](docs/evidence/rust-service/README.md#cp240).
Vòng1 A chết/B thắng; vòng2 B chết/A thắng: hai view cùng state1/0 trước175, đọc đủ302/292 byte và rank2/1 đảo đúng.
Hai180/refresh/Ready rồi clock trận3 giữ6 giây; 66 raw đã seal/cleanup0. M3 **còn mở**: consumed-item reset/contest,
exact UDP cũ + current positive, dead-owner/queue controls và hostEOF đưa peer về phòng. Không full game/accountauth.
CP239 đã merge [PR310](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/310)/`35bb05f`.
AUD069 qua240, audit kế CP244. [PLAN](docs/PLAN.md#current-order) giữ thứ tự; M5 backend/SQLite đã có CP236,
cầu nối KR vẫn riêng. Python và tám WIP giữ nguyên. Internet thật chưa kiểm.

**CP239: Rust đã có terrain/reveal và ba hiệu ứng item basic trên hai KR thật.**
[Contract/giới hạn](docs/PROTOCOL.md#rust-terrain), [raw/hash](docs/evidence/rust-service/README.md#cp239).
Cảhai195flags khớp kernel thamchiếu trên actual118: chỉô2,3 và3,2clear;142đúngô sau158commitmởhidden27/28.
Actual124→mỗi view ba165full12/errors0/founditem: Roller5→6,Fire1→2,Bomb1→2;1314copiedgetters/410TCP/66reply.
54raw frozen/seal/cleanup0;35Rusttests/43TCPcases đạt. M3 **còn life/result-reset/haioutcome**, không fullgame.
CP238 merge [PR309](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/309)/`7ecfba3`;
[PLAN](docs/PLAN.md#current-order) tiếp17/19/21→170→175/180/reset; CP240 phải audit địnhkỳ.
Python/8WIP/accountDB giữ. Internet thật chưa kiểm.

**CP238: hai client KR thật đã đặt và nổ bom qua Rust.**
[Contract/giới hạn](docs/PROTOCOL.md#rust-bombs), [raw/hash](docs/evidence/rust-service/README.md#cp238).
A/B actual118→159full39/entity,158full13/lookupđúngID1-2/kind0/cleanup trêncảhai; fuse6 và6shold sauhai bom.
TCP419/66reply khớp; 48rawfinal frozen/seal/cleanup0.32Rusttests/40TCPcases đạt.
Hai run lỗi44+50raw giữ nguyên; `167/4,32` chỉ boundedopaqueobserve/semanticsunknown,142 chưa mởhiddeneligibility.
M3 vẫn thiếu terrain/reveal/life/result-reset/haioutcome. CP237 merge [PR308](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/308)/`70f00d0`.
[PLAN](docs/PLAN.md#current-order) tiếp boundedterrain/reveal rồi life/results/reset, audit kếCP240.
Python/8WIP/accountDB giữ; Internet thật chưa kiểm.

**CP237: Rust đã có movement hai chiều và nhặt Roller trên client KR thật.**
[Contract/giới hạn](docs/PROTOCOL.md#rust-movement-floor), [raw/hash](docs/evidence/rust-service/README.md#cp237).
Hai cửa sổ ADown/BRight có PCAP217UDP→87ingress native→key/apply→peerposition; TCP368/58reply khớp.
Actual124/index17→một165 mỗi view, native12/12/errors0/itemfound và stat5→6. 58raw run cuối/cleanup0;
51+55raw hai run lỗi giữ nguyên. 30Rusttests/22TCPcases/696copiedgetters/87nativeCRC đạt.
M3 **còn dở**: bom/terrain/life/result-reset/haioutcome chưa port; `123/10` chỉ opaqueobserve policy có budget,
chưa biết semantics. Shutdown timeout cũ tái hiện ở kerneltest, repeatinstrumented đạt nhưng chưa rõ nguyên nhân.
CP236 account/DB đã merge [PR307](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/307)/`9748dcb`;
[PLAN](docs/PLAN.md#current-order) tiếp allocator/bom theo contract đã đo. Audit kếCP240. Python/8WIP giữ.
Internet thật chưa kiểm.

**CP236: M5 đã có lát cắt account/DB độc lập gameplay.**
[Contract backend](docs/PROTOCOL.md#rust-account-backend), [CLI](rust/README.md#account-admin-cli),
[bằng chứng](docs/evidence/rust-service/README.md#cp236): SQLite migration/checksum/schema thực, Argon2id,
account→player/wireID bền vững, profile tên/version, session TTL/revoke/ban/reset/boot epoch, admin audit cùng transaction.
26 Rust tests (11 account,10 service,5 wire) và33 process CLI/2account/9audit đạt; không lưu plaintext password/token vào DB.
Một ca CLI unknownfield đầu tiên lỗi, đã giữ raw/seal và sửa strict variant trước run mới.
Đây là backend công ty, **chưa nối đăng nhập KR**; M5 chưa đạt đầy đủ, chưa inventory/reward/idleTTL/worker/restore.
CP235 merge [PR306](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/306)/`bb673a4`;
[PLAN](docs/PLAN.md#current-order) tiếp M3 UDP/movement, M5 không cần chờ Battle và ngược lại.
AUD068 qua236, audit kếCP240. Python/WIP giữ nguyên. Internet thật chưa kiểm.

**CP235: M2 đạt trong phạm vi hai identity cục bộ, một phòng và loading trên Wine.**
[Đối chiếu acceptance](docs/PROTOCOL.md#rust-m2-acceptance), [bằng chứng](docs/evidence/rust-service/README.md#cp235),
[cách chạy](rust/README.md). Sau khi dừng riêng A, B giữ nguyên process/channel, native xóa ID của A,
chín report muộn được drop và UI/clock tiếp tục 8,231 giây. PCAP345/56reply khớp; 44raw frozen/seal/cleanup0.
15Rusttests,6guard trước bind,12shutdown race,8roomTCP,16binding và14M1 đạt. Guard version là policy hỗ trợ
profile KR00967 trước bind; chưa đo UI lỗi version. Sprite A còn trên map sau EOF và timeoutshutdown cũ chưa rõ
nguyên nhân được giữ thành giới hạn M3/M7; không nhận là fullBattle/accountauth/fullservice.
CP234 đã merge [PR305](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/305)/`f15a2ba`.
[PLAN](docs/PLAN.md#current-order) chuyển M3 Battle theo contract đã có; audit kế CP236. Python và WIP giữ nguyên.
Internet thật chưa kiểm.


**CP234: Rust đã đưa hai KR thật từ phòng qua Back/rejoin/Ready tới loading và spawn.**
[Contract/giới hạn](docs/PROTOCOL.md#rust-fixed-room), [evidence](docs/evidence/rust-service/README.md#cp234),
[code/run](rust/README.md): ownID1/slot0 và ID2/slot1, hostA Back→B nativeclearID1/chief→A rejoin/Ready→B Start;
121129/129/errors0, giữ nativeB99 hơn3s khôngGO, rồi haiACK→197/196/153→clock/spawn/UI cảhai, giữ8.279s.
PCAP334record/56reply khớp; finalbinary554fa5b3… đã chạy. Nămrun152raw frozen/seal/cleanup0 giữ failures/corrections;
14Rusttests/8TCPphòng/16binding/14M1regression. Có một shutdown test timeout chưa tái hiện ởba lần lặp, giữ rõ.
**M2 chưa đóng:** còn chốt negative/version và cleanup sau active-peer EOF; Bfinalreport bị từ chối sau ownedteardownA,
không healthyabortclaim. CP233 merge [PR304](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/304)/`b10952f`;
[PLAN](docs/PLAN.md#current-order) tập trung phần còn thiếu M2 trước BattleM3. Audit kếCP236; Python/WIP giữ. Internet thật chưa kiểm.


**CP233: hai client KR thật đã vào sảnh trên cùng service Rust với ID/tên riêng.**
[Contract/giới hạn](docs/PROTOCOL.md#rust-two-lobby), [code/run](rust/README.md),
[evidence](docs/evidence/rust-service/README.md#cp233): KRLocal/ID1 và KRGuest/ID2 đều auth137/137,
291128/128/errors0, personalrender/UI đúng tên; PCAP162 record/18reply khớp, giữ8.710s.
Binding dùng IP operator + request đầu kiểmCRC + endpoint/stage/redirectlease; bỏ acceptcounter ở profile hai-client.
Run đầu phân loại WCP chỉ theo port thất bại193raw, sửa theo CP006/103 rồi98raw positive; cả hai frozen/seal/cleanup0.
10Rusttests,16TCP binding cases và14M1 regression đạt. **M2 vẫn dở**: phòng chung/ownslot/host/Ready/loadingbarrier/
Back-rejoin còn thiếu; diagnosticidentity chưa accountauthentication. CP232 merge
[PR303](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/303)/`5b3ef81`; audit kếCP236.
[PLAN](docs/PLAN.md#current-order) tiếp fixedroom bằng contract có sẵn, Python/WIP giữ nguyên. Internet thật chưa kiểm.


**CP232: Rust đã đưa một KR thật tới sảnh với identity cục bộ và profile đúng tên.**
[Code/run](rust/README.md), [contract/limits](docs/PROTOCOL.md#rust-local-lobby),
[evidence](docs/evidence/rust-service/README.md#cp232): auth0/native137/137, channel17/241/91/14 rồi291
native128/128/errors0/nameKRLocal, personal renderguard1 và UI sảnhKRLocal; PCAP160record/9exactreplies,
giữ9.362s. Root binary đúng hash đã chạy;7Rusttests/fmt/clippy/build và14TCP M1regression đạt.
M2 **chưa hoàn tất**: còn hai-client/phòng/barrier/Back-rejoin; diagnosticidentity chưa accountauthentication.
Ba run startupfailure6raw + staging38 + final47 =91raw frozen/seal/cleanup, giữ correctionopcode15.
CP231 merge [PR302](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/302)/`bc55356`;
AUD067 qua232/kế236. [PLAN](docs/PLAN.md#current-order) tiếp phần còn thiếu M2, RE chỉ khi port bị chặn.
Python serving/RE/reference và támWIP giữ nguyên. Internet thật chưa kiểm.


**CP231: đã viết và chạy lát cắt Rust đầu tiên trên client KR thật.**
[Rust/build/run](rust/README.md), [contract/limits](docs/PROTOCOL.md#rust-foundation),
[evidence](docs/evidence/rust-service/README.md#cp231): cùng CA PID144 trên Wine nhận result1 hai lần,
ordinary UI retry mở connection mới/resetseq40; native mode2/selector3/43→46, PCAP108record khớp byte,
giữ9.517s sau result lần2. M0 baseline Python/47pins,5wire tests (vector native giữ sẵn) và14kernelTCP cases đạt;
fmt/clippy/locked build pass. Hai run trước giữ failure53raw/capturepartial63raw; final54raw, tổng170raw seal/cleanup.
Đây là **hello/error/retry**, chưa login thành công/sảnh/phòng/fullauth hoặc game trên Rust. Python giữ serving/RE/reference.
CP230 merge [PR301](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/301)/`115c6b8`; audit kếCP232.
[PLAN](docs/PLAN.md#current-order) chuyển M2 local identity→sảnh→fixedroom, RE chỉ khi port bị chặn. Internet thật chưa kiểm.


**CP230: cleanup phòng chung đã có native Back, EOF và chuyển chủ trong prototype.**
[Contract/limits](docs/PROTOCOL.md#room-shared-cleanup), [run/hash](docs/evidence/room-list/README.md#cp230):
guest Back hoặc EOF → A nhận107, native ID2→0/ô B biến mất; host Back → B nhận107/ID1→0,
110 đọc đủ và UI B thành chủ. Ba ca565raw đạt scope hẹp;166raw lỗi controller EOF giữ riêng/correction,
batch731raw frozen/seal/cleanup. Getter sau hostpromotion chưa quan sát vì budget20; không suy đã đo getter1.
CP229 đã merge [PR300](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/300); audit kếCP232.
[Owner đã mở implementation Rust](docs/evidence/owner-directives/rust-implementation.json);
[PLAN](docs/PLAN.md#current-order) ưu tiên M0→M1 ngay, RE chỉ khi chặn lát cắt. CP230 chưa có Rust/gate mới.
Internet thật chưa kiểm.


**CP229: prototype registry đã qua Back/rejoin và chuyển B sang phòng A bằng client thật.**
[Contract/limits](docs/PROTOCOL.md#room-membership-registry), [run/hash](docs/evidence/room-list/README.md#cp229):
B Beta→Back→sảnh→Beta lại đúng ownID/slot0/chief; ca riêng Beta→Alpha đưa B vào slot1/khách,
A giữ slot0/chủ và cả hai nhận đúng peer106. Cùng auth/channel, native87/106/110/291 đọc đủ, PCAP và UI khớp.
Hai run336raw cùng7raw startup-failure đã seal/cleanup; 19 policy scenarios và7 move scenarios tách khỏi native proof.
Registry private có owner/room/visit fences; chưa ghép production, shared-leave/EOF fanout và occupancy card còn thiếu.
CP228/AUD066 đã merge [PR299](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/299); audit kếCP232.
[PLAN](docs/PLAN.md#current-order) giữ bước tiếp; chưa Rust/gate mới. Internet thật chưa kiểm.


**CP228: hai phòng riêng đã hiện đúng một player tại slot0/chief trên mỗi client.**
[Contract/limits](docs/PROTOCOL.md#room-local-roster), [run/hash](docs/evidence/room-list/README.md#cp228):
sau actual585, private adapter trả riêng106/110/291 cho account đã bind; native đọc204/6/128 byte không lỗi,
own-ID khớp1/2, setter slot0 và UI Alpha/KRLocal, Beta/KRGuest. 129raw đã seal/hash/cleanup, giữ8.86s.
Marker cũ `own_slot=1` là predicate, không phải chỉ số slot1; personal-render sau291 chưa đo.
CP227 đã merge [PR298](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/298).
AUD066 qua228/kế232. [PLAN](docs/PLAN.md#current-order) tiếp registry/membership có generation và owner;
chưa coi fixed projection là registry hoặc isolation tổng quát, chưa Rust/gate mới. Internet thật chưa kiểm.


**CP227: chọn phòng đã nối đúng session, ID và tên; lỗi join có đường quay lại chọn tiếp.**
[Contract/limits](docs/PROTOCOL.md#room-join-projection), [năm run/hash](docs/evidence/room-list/README.md#cp227):
hai KR chọn row khác nhau → reply87 → native lookup/context đúng RoomAlpha/RoomBeta, reader7/7 không lỗi.
Control staleID dùng lỗi2 “không thể vào phòng” → đóng thông báo → refresh → chọn ID303 → RoomGamma.
Hai lần click retry không tạo callback và ca dùng lỗi1 “sai mật khẩu” được giữ kèm correction;
recipe click giữa card đã đo lại. 802raw seal/hash/cleanup; chưa trả585/roster/membership/registry/Rust/gate.
CP226 merge [PR297](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/297)/`ccab4cc`;
AUD065 qua224/kế228. [PLAN](docs/PLAN.md#current-order) tiếp roster riêng cho từng phòng.


**CP226: danh sách hai phòng, cập nhật và xóa card đã có native evidence.**
[Contract/limits](docs/PROTOCOL.md#room-list-lifecycle), [bốn run/hash](docs/evidence/room-list/README.md):
hai KR đọc count2 và chọn đúng session/roomID; S→C91 update1/empty0 giữ row khác, không xóa toàn danh sách.
S→C90 tag0 chỉ xóa khi session/roomID khớp; wrongID giữ card. Nativecursor/UI/typed82/PCAP đối chiếu,
545raw seal/hash/cleanup; run đầu gửi counter73 sai giữ nguyên, bản sửa55/58 đã đo. Chưa join87/membership/
registry/Rust/gate mới. CP225 merge [PR296](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/296)/`3bd1cdd`;
AUD065 qua224/kế228. [PLAN](docs/PLAN.md#current-order) tiếp row selection→session lookup/join projection.


**CP225: process B mới đăng nhập lại được; cùng process sau mất kênh vẫn lỗi.**
[Contract/limits](docs/PROTOCOL.md#auth-fresh-process), [ba run/hash](docs/evidence/login-runtime/README.md#cp225):
cleanup B cũ rồi mở KR mới → native0/ID2-name-echo6/67/newlease-session, A cùngPID/owner giữ8s+;
chỉ đổi tokenB mới sang sai → native1/nochannel/noBlease-session/Aowner45s+. Getter trả11 được lưu trước
setter(request1), chưa đồng nghĩa sửa sameprocessresume. 595raw seal/hash/cleanup; không fullauth/Rust/gate mới.
CP224 merge [PR295](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/295)/`928f2fb`;
AUD065 qua224/audit kế228. [PLAN](docs/PLAN.md#current-order) chuyển khoảng trống phòng tổng quát,
không tiếp quét SDK/VM hoặc lặp failedretry.


**CP224: hello và validation thành công; retry sau mất kênh bị fatal error.**
[Contract/correction](docs/PROTOCOL.md#auth-retry-validation), [ba run/hash](docs/evidence/login-runtime/README.md#cp224):
breakpoint đúng return13AD79C/1255C73/1255A69/125BE47 đều trả1, manager phase2;
chưa tokenbuilder/auth6-67/session mới, sau đó SIGABRT/Wine illegalinstruction. CP223 đã ghi retryFAIL
nhưng bỏ sót fatal trong log; record/raw cũ giữ nguyên. Observer unwind đầu được thay bằng exactsite,
462raw seal/hash/cleanup; continuation cho local11→epilogue, không dừng trong handoff. CP223 merge [PR294](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/294)/`51bbf41`.
[Thứ tự hiện hành](docs/PLAN.md#current-order); không fullauth/Rust/gate mới. AUD065 quaCP224/audit kếCP228.


**CP223: identity theo account khi đảo endpoint; retry sau mất kênh chưa đạt.**
[Contract/giới hạn](docs/PROTOCOL.md#auth-session-lifecycle), [bảy run/hash](docs/evidence/login-runtime/README.md#cp223):
control token đảo source vẫn đúng native ID/name/echo/67 và session; đóng B sau17 có native error10/newconnect,
Aowner giữ. CP223 mới khoanh bytes/framer; [CP224](docs/PROTOCOL.md#auth-retry-validation) bổ sung validation thành công và fatal error.
1108raw đã seal/hash/cleanup, corrections ảnh/probe giữ nguồn cũ; không fullauth/sessionexpiry/Rust/gate mới.
CP222 đã merge [PR293](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/293)/`4bcabd8`; audit kếCP224.
[Thứ tự hiện hành](docs/PLAN.md#current-order); không lặp control đã đo.

**CP222: hai KR cùng service đã bind hai identity/session riêng; duplicate account bị native từ chối.**
[Contract/giới hạn](docs/PROTOCOL.md#auth-two-actual), [bốn run/hash](docs/evidence/login-runtime/README.md#cp222):
R3 native0/C2S6 đúng1/2/echo/actual67 cảhai,2CA/2wineserver/2netns/PCAP/8s+hold; R4 tokenB=A cho native1/nochannel,
ownerA giữ nguyên. Hai run denial ban đầu giữ raw; suffix25 có biến môi trường, parser profile mới ghi design rõ.
498raw đã kiểm/seal/cleanup; chưa fullauth/healthyEOFdenial/Rust/gate mới. CP220–221 đã merge
[PR292](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/292)/`f5caefb`; audit kếCP224.
[Thứ tự hiện hành](docs/PLAN.md#current-order); control đảo endpoint đã có ở CP223.

**CP221: A/B đã đi qua native đúng identity; wrong-binding TCP giữ lease/owner.**
[Contract/giới hạn](docs/PROTOCOL.md#auth-two-account), [run/hash](docs/evidence/login-runtime/README.md#cp221):
hai positive Wine riêng ID1/KRLocal và ID2/KRGuest có auth0/C2S6 echo/actual67;119raw đã kiểm.
Bốn TCP driver run/20raw kiểm wrongID/name/sai source không tiêu lease hoặc xóa owner; R1 driver failures giữ nguyên.
Chưa hai KR đồng thời/swap/healthydenial/fullauth/Rust; EOF blocker giữ rõ. CP220–221 giao chung
[PR292](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/292), trạng thái tích hợp theo PR;
AUD064 qua220/audit kế224. [Bước tiếp](docs/PLAN.md#current-order).

**CP220: adapter giữ owner qua TCP duplicate/replay; native EOF lifecycle vẫn chưa đạt.**
[Contract/giới hạn](docs/PROTOCOL.md#auth-tcp-ownership), [run/hash](docs/evidence/login-runtime/README.md#cp220):
TCP driver tổng hợp kiểm owner tiếp tục nhận reply sau duplicate/replay và owner EOF giải phóng session;
không dùng kết quả này thay client KR. Getter/watch receiver chưa có observable đủ để sửa; hai native run vẫn lỗi.
Bốn run/102 raw mới đã seal/hash/cleanup; R1 driver error giữ nguyên, R2 dùng codec sẵn có.
CP219 đã merge [PR291](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/291)/`09435d9`;
AUD064 qua220, audit kếCP224; #14/#285 vẫn mở, chưa Rust/fullauth. [Bước tiếp](docs/PLAN.md#current-order).

**CP219: đã khoanh nguồn dispatch lỗi sau EOF, chưa giải lifecycle âm tính.**
[Contract/giới hạn](docs/PROTOCOL.md#auth-eof-dispatch), [bốn run/hash](docs/evidence/login-runtime/README.md#cp219):
lease hợp lệ hoặc baseline field36=0 vẫn cùng fault; native vtable+50 → target17C7F40 → call12578AC có proof.
188 raw kiểm đủ, cleanup0; native lifecycle vẫn FAIL, receiver selection/rootcause chưa biết.
CP217–218 đã merge [PR290](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/290)/`f9f48d2`;
không đổi gate/Rust/source production, audit kếCP220. [Bước tiếp](docs/PLAN.md#current-order).

**CP218: policy chặn lease expired/revoked, nhưng lifecycle client sau EOF chưa đạt.**
[Blocker/giới hạn](docs/PROTOCOL.md#auth-lease-eof-blocker), [hai run/hash](docs/evidence/login-runtime/README.md#cp218):
validtoken/native0/actual6echo rồi localdeny; cảhai SIGSEGV17C7F4C qua122EA12/12541E0.
Raw/stack seal/cleanup0; harnessreturn0 không làclienthealthy. Rootcause chưa biết, chưa negativeacceptance/fullauth.
CP217valid/badtoken giữ scope riêng; giaoCP217–218 cùngPR, audit kếCP220. Bước tiếp ở [PLAN](docs/PLAN.md#current-order).

**CP217: adapter RE riêng đã phân biệt token đúng/sai bằng client KR thật.**
[Contract/giới hạn](docs/PROTOCOL.md#auth-admission-prototype), [hai run/hash](docs/evidence/login-runtime/README.md#cp217):
input32chars đúng → native0/C2S6echo/actual67; sai → native1/khôngkênh trong45s, cảhai seal/cleanup0.
24expiry/replay/revoke/swap checks mới policyemulator; chưa fullauth/DB/sessionexpiry hoặc production/Rustintegration.
CP216 đã merge [PR289](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/289)/`af16278`;
trạng thái tích hợp CP217 theo PR, AUD063through216/audit kếCP220. Bước tiếp duy nhất ở [PLAN](docs/PLAN.md#current-order).

**CP216: đã xác nhận authfield36 đi qua cache tới C2S6; chưa xác thực tài khoản.**
[Contract/layout/giới hạn](docs/PROTOCOL.md#auth-channel-echo), [hai run/hash](docs/evidence/login-runtime/README.md#cp216):
control một u32+CRC, native reader→setter→getter→serializer→wire khớp; C2S6 chỉ đổi4bodybytes.
Count1/ASCII/currentcopy, không gọi field echo32bit là ticket antoàn hoặc fullauth. Chưa Rust/sourcepolicychange.
CP215 đã merge [PR288](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/288)/`d3b85df`;
CP216 trạng thái tích hợp theo PR, AUD063 qua216/audit kếCP220. Thứ tự duy nhất ở [PLAN](docs/PLAN.md#current-order).

**CP215: token15/16 đã qua native SSO/heap và wire; token2-only cho count0.**
[Contract/giới hạn](docs/PROTOCOL.md#auth-token-bounds), [hai run/hash](docs/evidence/login-runtime/README.md#cp215):
frame100/body62, record17/18byte khớp copied-native,13 helpervectors emulation và CRC/partial controls riêng.
Run token2-only không gọi authbuilder trong cửa sổ đo; cảhai seal/cleanup0. Chưa xác thực tài khoản/Rust.
CP214 đã merge [PR287](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/287)/`83de46c`;
trạng thái tích hợp CP215 theo PR, audit kếCP216. Bước tiếp duy nhất ở [PLAN](docs/PLAN.md#current-order).

**CP214: đã nối token local tới request đăng nhập native KR; chưa xác thực tài khoản.**
[Contract/giới hạn](docs/PROTOCOL.md#auth-token-request), [ba run/hash](docs/evidence/login-runtime/README.md#cp214):
Winecopy4b/count1–2, bốn record token khớp native writer, ba frame actual +CRC/partial controls; cleanup0.
Chưa tokenverify/expiry/revocation/crosssocketownership hoặc Rustcode. PR286/CP213 đã giao kế hoạch Rust;
CP214 đã merge [PR287](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/287); audit tại mốc đó kếCP216. Thứ tự tiếp chỉ ở [PLAN](docs/PLAN.md#current-order).

**Kế hoạch Rust đã lập tại CP213; chưa triển khai Rust.**
[Lát cắt đầu và thứ tự milestone](docs/PLAN.md#rust-service-milestones):
codec/connection → localidentity/sảnh/phòng → Battlehai người/reset → bốn/tám theo contract;
account/DB làm độc lập sớm, sau đó inventory/result-reward và vận hành.
[Thiết kế, ma trận readiness và nguồn tái sử dụng](docs/RUST_SERVICE_ARCHITECTURE.md),
[survey/hash](docs/evidence/rust-service/README.md). Đề xuất cùng repo, rust/ với hai crate kr-wire/kr-service,
một process Linux; Python giữ RE/reference/differential. Không tạo Cargo/repo/worktree Rust trong đợt này.
Authbridge/token/ownership và inventory-reward projection cần đo KR riêng; gate demo không fullservice.

**CP213: tám KR Wine đã hoàn thành hai kết quả đối nghịch/reset/Start3 ở Free 1–7 và Manner 4–4.**
[Contract/giới hạn](docs/PROTOCOL.md#eight-team-variants), [raw/hash](docs/evidence/eight-player/README.md#cp213).
Free1–7:4344gameplay+125modejoins; Manner4–4:4357+124,128copiedgetters mỗi run; selector0/1 giữ qua86/87/180.
Fixedoperatorroster, chưa interactiveC2Steamchange. Các ca Manner1–7 copy4b dừng trước Start do startup; ca támactual trên alternatecopy8d đã trả7/no104,
1543checks/140.441s/cleanup0; guardJSONkhônghit nên khôngstartupfixclaim. CP163 mộtactual/tám tổnghợp giữriêng.
K6c WAN giả lập/K7 giữ đạt scope CP212; **Internet thật chưa kiểm**. CP213 docs/evidence được giao qua PR; code teamvariants vẫn WIP chưa merge, audit kếCP216.
Bước kế duy nhất ở [PLAN](docs/PLAN.md#current-order); không lặp codec/rank/teamguard đã đo.

**CP212: K6c WAN giả lập đạt; K7 hardening/demo bốn client đạt trong scope đã đối chiếu. Internet thật chưa kiểm.**
[Chỉ đạo chủ dự án](docs/evidence/owner-directives/emulated-wan.json) thay duy nhất môi trường K6c;
[acceptance K6c](docs/PLAN.md#k6c-acceptance) và [đánh giá lại từng tiêu chí K7](docs/PLAN.md#k7-evidence-review)
dùng bằng chứng native đã seal: CP193 endpoint negative/positive, CP195 hai trận qua NAT/UDP40±5ms/loss5%,
CP196 host EOF/cleanup/fence và CP203–207 đóng gói/bounds/identity/health/freshserver/bốn client.
[Topology/tham số/giới hạn](docs/PROTOCOL.md#emulated-wan-acceptance), [raw/hash/decision](docs/evidence/peer-endpoints/README.md#cp212).
Không chạy lại phần đã đủ bằng chứng; không nghiệm thu Internet/ISP/CGNAT thực tế, full game/account auth hoặc mọi mode/map/item.
Các giới hạn đó là backlog riêng; tám client không phải điều kiện K6c/K7. Codex tự kiểm, review độc lập waived;
CP212 đã merge [PR284](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/284)/`c0c7b94`, AUD062 qua212/audit kếCP216. Công việc kế tiếp duy nhất ở [PLAN](docs/PLAN.md#current-order).

Các đoạn CP bên dưới ghi **trạng thái tại checkpoint nêu tên**, kể cả K6c/K7 từng còn mở; quyết định hiện hành ở trên.

**CP211: gói server tám người đã chạy sau relocation và hoàn tất hai vòng 4v4/reset/Start3 trên tám KR Wine thật.**
71filepins/47fixturealiases, profile/bom/outcomeflags cốđịnh;
[contract/giới hạn](docs/PROTOCOL.md#eight-portable-battle), [cách xuất/chạy](tools/README.md#eight-battle-bundle),
[raw/hash](docs/evidence/eight-player/README.md#cp211). Gói không chứaclient/assets; LANgiảlập/Wine, chưaInternet/K7.
CP210 đã merge [PR282](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/282)/`9202d77`; CP211 đã merge [PR283](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/283)/`936693d`.
Tại CP211 audit kếCP212; tiếp actualteamvariants theo PLAN, không lặp codec/rank đãgiải.

**CP210: cùng tám KR Wine đã hoàn thành hai vòng 4v4, hai đội lần lượt thắng, kết quả → phòng → Ready → Start3.**
[Contract/scope](docs/PROTOCOL.md#eight-two-team-cycles), [raw/hash](docs/evidence/eight-player/README.md#cp210).
Native rank8/1751082-inner1072/180/reset vàlife17→19→7peer có nguồn; không item/cứu/kim8 hoặc Internet/K7.
CP209 đã merge [PR281](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/281)/`bebb1d0`; CP210 đã merge PR282.
AUD061 qua208/audit kếCP212; bước kế ở PLAN. Đây là scope8client4v4 tối thiểu, khôngtoànbộgame.

**CP209: tám KR Wine đã có movement đủ56chiều remote và cả tám owner đặt/confirm/nổ bom ở mọi view.**
[Contract/stat/giới hạn](docs/PROTOCOL.md#eight-mesh-owner-bombs), [raw/hash](docs/evidence/eight-player/README.md#cp209).
Ecapacity2,G/Hpower2 đo riêng; TạiCP209 127/rank/reset8 chưa tíchhợp; [CP210](docs/PROTOCOL.md#eight-two-team-cycles) bổ sung hai normal4v4cycles.
CP208 đã merge [PR280](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/280)/`8a99a72`; CP209 đã merge PR281.
AUD061 qua208/audit kếCP212; K6cInternet/K7 còn mở. Bước kế ở PLAN.

**CP208: tám client KR Wine thật đã login → sảnh → phòng → load → Battle/clock.**
Tám prefix/display/netns/IP/CA PID riêng,32TCP/own-slot0–7/8actual99 và full native121285/153205;
[contract/giới hạn](docs/PROTOCOL.md#eight-session-preview), [raw/hash](docs/evidence/eight-player/README.md#cp208).
TạiCP208 chưa movement mesh/bom/kếtquả/reset8; [CP209](docs/PROTOCOL.md#eight-mesh-owner-bombs) bổ sung mesh và bom, kếtquả/reset8 còn thiếu. Bước kế duy nhất ở [PLAN](docs/PLAN.md#current-order).
CP207 đã merge [PR279](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/279)/`7ffed54`; CP208 đã merge PR280;
review waived, AUD061 qua208/audit kếCP212. K6c Internet/K7 vẫn mở.

**CP207: bốn KR Wine đã qua hai vòng 2v2/reset/Start3 dưới UDPdelay/loss, server mới dùng lại samebundle vàidentitystore.**
[Contract/giới hạn](docs/PROTOCOL.md#four-netem-restart), [raw/hash](docs/evidence/peer-endpoints/README.md#cp207),
[K7đốichiếu](docs/PLAN.md#k7-evidence-review). Internet/K6c/K7 vẫn mở; không accountresume/8clientproof.
CP206 đã merge [PR278](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/278)/`4d136ba`; audit kếCP208.

**CP206: có health/run-session-room-round ID và identity pin qua restart process; bốn KR Wine vẫn qua hai vòng 2v2/reset/Start3.**
[Contract/giới hạn](docs/PROTOCOL.md#four-health-identity), [raw/hash](docs/evidence/peer-endpoints/README.md#cp206).
TạiCP206 còn nativefreshrestart/fourUDPimpairment; [CP207](docs/PROTOCOL.md#four-netem-restart) đã bổ sung; bước kế ởPLAN. K7/Internet chưa đóng; auditCP208.
CP205 đã merge [PR277](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/277)/`bf165f9`.

**CP205: guest hoặc host rớt giữa Battle, ba client KR Wine còn lại đã về phòng; host rớt thì B nhận chủ phòng.**
[Contract/giới hạn](docs/PROTOCOL.md#four-member-abort-room), [raw/hash](docs/evidence/peer-endpoints/README.md#cp205).
Không rank giả hoặc nổ bom muộn; phòng vẫn fenced, chưa chơi tiếp ba người/reconnect. Tại CP205 còn health/runidentity/restart; [CP206](docs/PROTOCOL.md#four-health-identity) đã thêm controls, bước hiện hành
trong PLAN; K7/Internet còn mở, audit kếCP208. CP204 đã merge [PR276](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/276)/`d78bc63`.

**CP204: sửa recipient khi gửi lỗi và thêm giới hạn I/O/log/rate; bốn KR Wine vẫn qua hai vòng 2v2/reset/Start3 trên gói mới.**
[Contract/giới hạn](docs/PROTOCOL.md#four-transport-bounds), [raw/hash](docs/evidence/peer-endpoints/README.md#cp204).
Tại CP204 còn nativecleanup ba survivor; [CP205](docs/PROTOCOL.md#four-member-abort-room) đã bổ sung guest/host; tiếp health/restart/identity. K7/Internet vẫn mở; AUD060qua204/kếCP208.

**CP203: gói bốn người đã chạy từ thư mục chuyển chỗ và qua hai vòng 2v2/reset/Start3 bằng KR Wine.**
[Contract/guard/giới hạn](docs/PROTOCOL.md#four-portable-guards), [cách xuất/chạy](tools/README.md#four-player-bundle).
Tại CP203: guard sau accepteddeath đã có, còn transport/rate; CP204 bổ sung phần đó. Lifecycle/restart/persistence/health/K7/Internet theo PLAN.

**CP202: cùng bốn KR Wine đã hoàn thành hai vòng 2v2, hai đội lần lượt thắng, kết quả/reset và Start3.**
[Contract/giới hạn](docs/PROTOCOL.md#four-two-team-cycles), [raw/hash](docs/evidence/peer-endpoints/README.md#cp202).
Tại CP202 còn guard/đóng gói4; CP203 đã bổ sung. K7/Internet vẫn mở; bước tiếp transport/hardening theo PLAN. Review waived; tại CP202 audit kếCP204, mốc hiện hành theo CLIlatest.

[Coverage và cây chức năng](docs/COVERAGE.md) trả lời: cơ chế nào hiện diện, mức RE/implementation/
runtime nào có nguồn, còn thiếu gì và phép kiểm nào giải quyết. Mỗi ID dẫn thẳng contract/giới hạn,
evidence/hash và câu hỏi; [PLAN](docs/PLAN.md#current-order) giữ duy nhất thứ tự công việc.
Đây là scope đường bootstrap → login → sảnh → phòng → Battle → kết quả/reset, không kiểm kê toàn client.
Tra checkpoint: `python3 tools/checkpoints.py latest`, `show CPxxx`, `evidence CPxxx`
(summary/page hoặc `--verify --require-local`); [kho v2/migration](docs/evidence/checkpoints/README.md#kho-checkpoint-v2-và-cách-đọc-chọn-lọc)
giữ record cũ nguyên object. Không cần đọc manifest lớn.

Hai copy KR Wine đã chạy login/sảnh/phòng/Battle và các chu kỳ kết quả/reset có bằng chứng;
**K5 basic local đạt CP188; K6a local đạt CP189; K6b LAN giả lập đạt CP191; K6c WAN giả lập/K7 đạt CP212. Internet thật chưa kiểm**. [Movement/terrain hai vòng](docs/PROTOCOL.md#current-movement-cycles),
[kim/reset](docs/PROTOCOL.md#needle-round-reset), [cứu hai chiều và hai kết quả](docs/PROTOCOL.md#rescue-two-results),
[deep chain/reset](docs/PROTOCOL.md#recursive-chain-cycles) giữ phạm vi và giới hạn tại nguồn chuẩn.

Các mốc dưới ghi trạng thái **tại checkpoint nêu tên**; next-step hiện hành chỉ ở PLAN.

**K0 và bước tĩnh K2a đạt CP178; K1 đạt riêng recipe Wine R2 offline tại CP179**, đã merge
[PR250](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/250)/`2f2b87f` và
[PR251](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/251)/`2e6dae4`.
[Phạm vi nghiệm thu](docs/PLAN.md#k1-acceptance) không bao gồm bootstrap Windows nguyên bản hoặc ba coldstart e9.

**CP181: K2 đạt cho profile codec/hello offline đã đo**, sau đối chiếu thủ công từng tiêu chí
giữ nguyên. Native retry trong cùng process: result1 hợp lệ → command1002 → login socket mới →
auth137/137/errors0 → channel socket mới; hello/mode/seq reset được đo trên từng connection.
[Nghiệm thu/giới hạn](docs/PLAN.md#k2-acceptance), [contract](docs/PROTOCOL.md#login-retry-codec),
[run/hash](docs/evidence/live-codec-stream/README.md#cp181). CP180 split/coalesced/CRCcontrol đã merge
[PR252](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/252) tại`fe242cc`.

**Tại CP191–211 K6c/K7 còn mở (đã nghiệm thu ở CP212); K6b LANgiảlập đạtCP191 theoowner. K3 local CP186, K4 local CP187, K5 basic CP188 và K6a local CP189 đã đạt riêng scope**,
[nghiệm thu/giới hạn](docs/PLAN.md#k3-acceptance). Fullgrammar/unknownheader/prefix/mode và
accountauthority chưa được khôi phục đầy đủ; K2 không nghiệm thu các phần đó. Thứ tự tiếp ở
[PLAN](docs/PLAN.md#current-order): nhánh mạng/admission đã có CP192–198; bốn session vào Battle tại CP200, CP202đã có2v2haioutcomes/reset/Start3; tiếp hardening theo PLAN, Internet thật giữ backlog riêng theo CP212.
CP181 đã merge [PR253](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/253) tại `5a154e5`.
**CP184: identity291 đã tích hợp opt-in vào service.** Hai run A đã hiện tên ở sảnh rồi tạo phòng,
nhận roster/map; native framer nhận sequence43…73 liên tục, next76. [Contract/giới hạn](docs/PROTOCOL.md#own-profile-integration),
[run/hash](docs/evidence/lobby-identity/README.md#cp184). Đây là một client thật với guest tổng hợp,
không thay bằng chứng hai client hoặc nghiệm thu trận. CP183 đã merge [PR255](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/255)
tại`5c1f3fb`; CP184 đã merge [PR256](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/256) tại`1f8018a`. Tại CP184 K3 còn mở; [CP186 đã đối chiếu nghiệm thu](docs/PLAN.md#k3-acceptance) cho profile hẹp.

**CP185: cache dynamic-map đã có đường request528→reply238 trong service opt-in.**
Đối chứng thiếu file, native ghi lại và warm không yêu cầu528 có [evidence/giới hạn](docs/PROTOCOL.md#lobby-cache-route).
Chưa cold toàn client; CP186 bổ sung cold route kênh và version/identity/negative cho nghiệm thu K3 hẹp. [Wiki KR do chủ dự án tin cậy](docs/REFERENCES.md#owner-trusted-namu-wiki)
được dùng cho luật/stat/char/map/mode trước khi mở RE gameplay; [bảng dùng lại](docs/PROTOCOL.md#trusted-wiki-reuse) giữ phần phải đối chiếu00967.

**CP186: K3 đạt trong scope một A/một kênh normal offline.** Bảy run và tám ca wire kiểm cache route,
version lỗi, native identity và guard request6; [contract/giới hạn](docs/PROTOCOL.md#k3-version-identity-controls).
Không fullaccountauth, không caption kênh có tên hoặc hai client291 đã nghiệm thu.
**CP187: K4 đạt trong scope phòng/load/start hai actualAB.** Đã đo identity291 cả hai, hostBack/
rejoin, một kết quả/reset và Start2; [nghiệm thu/giới hạn](docs/PLAN.md#k4-acceptance).
Copy riêng sửa importABI đã qua lỗi kết quả; [control/recipe](docs/PROTOCOL.md#own-profile-result-compare).
Run CP187 vẫn là lịch sử partial ở input vòng2; [CP188](docs/PROTOCOL.md#current-basic-cycles-291) đã có hai outcome mới, không sửa kết quả run cũ.
**CP188: K5 basic local đạt** theo [tiêu chí giữ nguyên](docs/PLAN.md#k5-acceptance).
CurrentAB291 có hai normal outcome/reset, hai chiều peer và hai owner bom; run riêng đã đo pending
B-host EOF/A cleanup/fence. [Contract/giới hạn](docs/PROTOCOL.md#current-basic-cycles-291),
[recipe/hash và partial giữ nguyên](docs/evidence/peer-endpoints/README.md#cp188).
Tại CP188 còn currentbasiccontest; CP189 bổ sung bên dưới, không thay kết quả lịch sử.
CP187 đã merge [PR259](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/259) tại `e57e6283`.
**CP189: K6a basic local đạt** sau currentcontesteditem/two-normalcycles/reset/nativepeer/life
và đối chiếu CP188Bowner/EOF riêng; [scope/giới hạn](docs/PLAN.md#k6a-acceptance),
[recipe/raw/hash](docs/evidence/peer-endpoints/README.md#cp189). Hai claim thật trên production,
mộtwinner/effect/view; baseline stat riêng A5/B4. Không physicalLAN/Internet/bốn–tám actualclient.
CP188 đã merge [PR260](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/260) tại`2ea548bb`.
**CP190: service/47fixture đã đóng thành gói chuyển thư mục**, kiểm bằng Python-I/realTCP và hai KR
Wine copy trên service đã relocation. [Cách dùng](tools/README.md#portable-bundle),
[nguồn/giới hạn](docs/PROTOCOL.md#portable-probe-bundle). Tại CP190 còn yêu cầu2máyvậtlý;
CP191owner thay bằng LANgiảlập, startup vẫn tuần tự.
CP189 đã merge [PR261](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/261) tại`1559ad57`.
**CP191: K6b LANgiảlập đạt theo chỉ đạo chủ dự án**, dùng hai actualKR trong netns/IP/NIC riêng;
[decision/giới hạn](docs/PLAN.md#k6b-acceptance). Không còn blocker phải có hai máy vật lý.
BpostwinRollerreset4→5→4 đã xác nhận qua hai normaloutcomes/Start2/3 trênportablebundle;
[nguồn](docs/PROTOCOL.md#emulated-lan-acceptance). K6cInternet/K7 vẫn mở.
CP190 đã merge [PR262](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/262) tại`b7e83f46`.
**CP192: actualKR Wine đã qua hai subnet/router** với hai normalcycles/reset/Start3 và endpoint/nativeUDP
có capture; [contract/giới hạn](docs/PROTOCOL.md#routed-Wine-network). Chưa NAT/relay/Internetgate.
[Owner chấp nhận Wine thay Windows](docs/evidence/owner-directives/wine-accepted.json), không cần máy Windows.
CP191 đã merge [PR263](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/263) tại`56a08dd6`.
AUD057–060 periodic qua192/196/200/204 là lịch sử; AUD062 qua212, audit kế **CP216**. [Review waived](docs/evidence/checkpoints/review-waiver.json).

**CP193: đã đo đối chứng NAT trên hai KR Wine thật.** Endpoint riêng làm consumer trả0; khai báo tuple
đã ánh xạ + DNAT khôi phục handler, key và movement hai chiều. [Contract/giới hạn](docs/PROTOCOL.md#nat-endpoint-control),
[run/hash](docs/evidence/peer-endpoints/README.md#cp193). Tại CP193 chưa hai chu kỳ NAT; [CP194](docs/PROTOCOL.md#nat-two-cycles) bổ sung, K6c Internet vẫn mở.
CP192 đã merge [PR264](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/264) tại`1a6daad0c17d718bc86574162bf2946bca1ed800`.
CP193 đã merge [PR265](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/265) tại`8acf57c81a727892c04e735c28a20a5319d70fd9`; review waived; tại CP193 audit kếCP196, hiện theo counter manifest.

**CP194: hai KR Wine đã hoàn thành hai vòng Battle bình thường/reset/Ready2/Start3 qua mapping NAT tĩnh.**
[Phạm vi và giới hạn](docs/PROTOCOL.md#nat-two-cycles), [raw/hash](docs/evidence/peer-endpoints/README.md#cp194).
CP194 đã merge [PR266](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/266) tại`9f4ebf53d29d22092b0971c7fb34e07dbcb2cce1`; chưa nghiệm thu Internet/K6c.

**CP195: hai vòng KR qua NAT vẫn hoàn tất/reset/Start3 với UDP delay40±5ms/loss5% đã kiểm counter.**
[Phạm vi và giới hạn](docs/PROTOCOL.md#udp-netem-cycles), [raw/hash](docs/evidence/peer-endpoints/README.md#cp195).
CP195 đã merge [PR267](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/267) tại`fe329b0b4da66aa4138c0e1a242da6ab3033d4a2`; TCP baseline, chưa Internet gate.

**CP196: pending B-host EOF qua NAT/UDP impairment đã kiểm A về phòng/ID clear/chief và fence bom chờ.**
[Contract/giới hạn](docs/PROTOCOL.md#nat-netem-host-eof), [run/hash](docs/evidence/peer-endpoints/README.md#cp196).
Run r1 partial do Python probe assertion giữ nguyên; r2 bounded probe pass. Tại CP196: AUD058 qua196/audit kếCP200; mốc hiện hành theo CLIlatest.
CP196 đã merge [PR268](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/268) tại`e842347b7cac073c6822b2251096747710abc170`; K6c Internet/K7 còn mở.

**CP197: incoming admission A/B theo IP nguồn riêng đã có opt-in**, realTCP xen kẽ0wrong/8 và native
startup tuần tự vẫn qua hai trận/reset/Start3 trên bundle mới. [Contract/giới hạn](docs/PROTOCOL.md#operator-incoming-binding),
[recipe/hash](docs/evidence/peer-endpoints/README.md#cp197). CP197 đã merge [PR269](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/269) tại`4c3f6ec246b3e0fcec017592aa6bd34f832ba6b9`. Tại CP197 native startup xen kẽ còn thiếu; [CP198](docs/PROTOCOL.md#native-interleaved-startup) đã bổ sung. Chưa bốn client/K7.

**CP198: hai KR native startup xen kẽ A1,A2,B1,B2,A3,A4,B3,B4**, đúng role/stage/identity, đã hoàn tất
hai normal cycles/reset/Start3 trên gói admission hiện hành. [Scope/giới hạn](docs/PROTOCOL.md#native-interleaved-startup),
[raw/hash](docs/evidence/peer-endpoints/README.md#cp198). Tại CP198: K6c Internet/K7/bốnclient chưa đạt, audit lúcđóCP200; currentfourproof ở CP202–204.

**CP199: copy KR hiện hành đã đọc Start121/peer153 bốn slot và khởi tạo bốn actor** ở cả hai view.
[Phạm vi](docs/PROTOCOL.md#four-slot-native-consumers), [raw/hash](docs/evidence/peer-endpoints/README.md#cp199):
hai actual client + hai actor tổng hợp, A/B peer còn hoạt động. Chưa bốn actual client, kết quả/reset2v2 hoặc K7.
Blocker chuyển sang service bốn session/identity/roster/Ready/barrier/mesh; thứ tự chỉ giữ ở PLAN.

**CP200: bốn KR Wine thật đã login/sảnh/phòng/load vào Battle**, đúng bốn identity/ownslot; run ít probe
quan sát movement từ từng client ở ba client còn lại. [Phạm vi/controls](docs/PROTOCOL.md#four-session-native),
[recipe/hash](docs/evidence/peer-endpoints/README.md#cp200). Service bốn session riêng; chưa bom/kết quả/reset2v2,
Start2 hoặc đóng K7. CP199 đã merge [PR271](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/271)/`ae329187`.
Tại CP200: AUD059 qua200/kếCP204; hiện AUD060qua204/kếCP208. Run lỗi/header/probe/metadata giữ raw riêng, không sửa thành pass.

**CP201: bốn owner A/B/C/D đã đặt bom và nhận confirm/nổ ở cả bốn KR Wine view.**
[Contract/baseline/giới hạn](docs/PROTOCOL.md#four-owner-bombs), [raw/hash](docs/evidence/peer-endpoints/README.md#cp201).
Capacity đã đo1/2/1/1, power1; native15939/39/entity và15813/13/errors0 đủ bốn bom/bốn view. Chưa kết quả/reset2v2.
CP200 đã merge [PR272](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/272)/`cd958c95`; audit kế CP204.

Đích ban đầu hai client/hai trận đã đạt trong scope K5/K6a; bốn client và hardening tiếp theo PLAN.

## Bắt đầu

Đọc [AGENTS](AGENTS.md), [PLAN](docs/PLAN.md), rồi [PROTOCOL](docs/PROTOCOL.md) và
[RESEARCH](docs/RESEARCH.md) theo task.

```bash
python3 tools/feature_catalog.py
python3 tools/check_docs.py
python3 tools/check_checkpoints.py --require-local --require-history
python3 tools/artifacts.py verify
```

`artifacts.py verify` dùng manifest **host-local** trong `docs/evidence/artifacts.json`; client/DB
không nằm trong Git nên clone sạch trên máy khác chưa thể chạy lệnh này. Khi chuyển máy, dùng
`--manifest /path/to/local-artifacts.json` với đường dẫn nguồn và hash đã xác nhận tại máy đó.

Client nguồn giữ ở `/home/admin/boom/clients/kr-00967`; không sửa hoặc commit. Artifact và database
đã unpack ở repo RE cũ chỉ là đầu vào tham chiếu; mỗi phép thử dùng bản copy riêng trong `local/`.
Nguồn/commit và giới hạn bằng chứng: [REFERENCES](docs/REFERENCES.md).
