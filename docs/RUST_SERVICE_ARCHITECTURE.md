# Dịch vụ Rust tương thích KR00967

Revision **kr-service-design-1**, 08/11/2026, design cho triển khai tiếp;
[KRE-074/#285](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/285).
Đợt khảo sát CP213 đã hoàn tất; [owner09/11](evidence/owner-directives/rust-implementation.json) đã mở implementation.
Trạng thái/thứ tự hiện hành chỉ ở PLAN; readiness bên dưới giữ nguồn của survey và bổ sung qua contract/evidence mới.
[PLAN](PLAN.md#rust-service-milestones) giữ **duy nhất thứ tự milestone/acceptance/rollback**.
[PROTOCOL](PROTOCOL.md#authority) giữ contract/authority; [RESEARCH](RESEARCH.md#rust-service-gaps) giữ unknown/phép RE.
Tài liệu này giữ quyết định kiến trúc và ma trận sẵn sàng, không là current-status hoặc plan thứ hai.

## Tra nhanh cho quyết định của công ty

| Câu hỏi | Kết luận của đợt nghiên cứu |
|---|---|
| Bắt đầu Rust ở đâu? | Chốt baseline Python rồi viết codec KR và vòng đời connection. Dùng client thật kiểm hello, lỗi đăng nhập và retry trước khi mở rộng sảnh/phòng/trận |
| Chuyển được gì ngay? | Profile codec đã đo, các packet tối thiểu của đường offline, cơ chế phòng/trận cơ bản, ticket/epoch, endpoint exchange và kết quả/reset. Chuyển đúng phạm vi bằng chứng, giữ policy hiện tại |
| Chưa đủ phần nào? | Xác thực tài khoản và quyền sở hữu identity qua nhiều socket; phòng tổng quát; inventory/progression/phần thưởng trên client; nhiều item/mode chưa có runtime proof |
| Python còn làm gì? | Giữ bản tham chiếu phục vụ được, công cụ RE và đối chứng. Chỉ rút khỏi đường phục vụ từng lát cắt khi Rust đã qua native acceptance tương ứng |
| Cần xây gì để vận hành? | Account/DB, session expiry/revocation, hồ sơ và inventory, transaction kết quả/phần thưởng, quản trị có audit, giám sát, backup đã thử restore, deploy/rollback và kiểm tải |
| Kiến trúc chọn gì? | Cùng repo, hai crate, một process Linux; Tokio và queue hữu hạn; owner phòng xử lý tuần tự; SQLite ở giai đoạn nội bộ. Các lựa chọn này là design, không phải luật server gốc |
| RE tiếp theo để làm gì? | Chỉ đo khoảng trống chặn lát cắt. Backend account/DB có thể làm độc lập; bridge để client KR sử dụng account vẫn cần phép đo riêng |
| Khi nào đủ bàn giao? | Theo tiêu chí demo, dịch vụ nội bộ và phát hành rộng hơn trong PLAN. Chưa ước lượng ngày hoặc số checkpoint; Internet thật chưa kiểm |

## Quyết định và căn cứ

| Quyết định design | Lý do / điều kiện xem lại |
|---|---|
| Viết Rust từng lát cắt; Python giữ RE/đối chứng | Đã có native contracts/policy hẹp; không chờ full RE. Lần chuyển đầu giữ semantics, thay authority/policy thành task riêng |
| Cùng repo boom-kr-server, rust/Cargo.toml, rust/crates/kr-wire và kr-service | Contract/evidence/tool/issue cùng lịch sử; giữ server/ Python. Chỉ hai crate: wire thuần và service lib+binary. **Implementation đã mở theo owner09/11; dùng rust/ cùng repo** |
| Một process Linux, nhiều listener/stage, mỗi room một owner serialize | Hiện một phòng/2–8 người; không cần microservices/IPC. Chỉ tách process khi có đo tải, ownership hoặc failure isolation cụ thể |
| Tokio I/O, bounded queues theo byte và message, writer riêng mỗi socket | Nonreader không chặn phòng, stage/seq/order theo connection. Không spawn unsupervised task cho từng timer |
| Giữ direct UDP peer và endpoint exchange đã chứng minh | Native tự tính movement/geometry một phần. Không thay bằng TCP-only hoặc authoritative snapshots của boom-rs. Mapping NAT tĩnh là một profile, không discovery/relay đã phục hồi |
| SQLite WAL/local persistent volume, một service instance | Transaction/unique/index/backup đơn giản. DB worker blocking riêng, queue hữu hạn. PostgreSQL chỉ khi có multiwriter/HA/load; domain không khóa vào SQLite |
| Backend account/session/inventory/reward là design; adapter KR riêng | Local identity/login-result không xác thực đầy đủ. Backend làm sớm, client binding chỉ bật sau khi đo ownership/token/multi-connection |
| Giữ semantics/profile Python trong lần chuyển đầu | Fuse/drop/report-derivedlife/rank/terrain vẫn design; shadow discrepancy trước enforce validator mới |
| KR crypt/CRC không là TLS hoặc credential protection | Credential mới qua CLI được bảo vệ hoặc web HTTPS design. Không nhét password/token vào unknown header/field |
| Không import nguyên crate/server VN hoặc boom-rs | Khác build/wire/authority. Dùng pattern/test lessons, reimplement từ KR contract có provenance |

M1 đã pin trong Cargo.lock/toolchain: Tokio1.53.3 cho boundedI/O/timeout/signal, serde1.0.229/serde_json1.0.154 cho config/log,
sha2 0.11.1 cho fixtureprovenance. kr-wire không dependency. bytes là transitiveTokio; chưa cần trực tiếp.
CP236 đã thêm/build/test/merge Argon2id (`argon2=0.6.0`), SQLite bundled (`rusqlite=0.40.2`),
`getrandom=0.4.3`, `zeroize=1.9.1` trong lockfile. [Backend account/DB](PROTOCOL.md#rust-account-backend)
và CLI có CP236/11accounttests/33CLIprocess; CP243 thêm boundedworker/handoff/compensation (5worker cases/5retainedDBs). CP244 thêm schema2/idleTTL/per-worker rate; chưa nối worker vào KR serving/nativeKRbridge hoặc restore.
Tracing/metrics và HTTP framework chỉ thêm khi có nhu cầu; CLI hiện có không phải web/API đã phục vụ.
`encoding_rs` chỉ thêm khi có consumer cp949 cần đối chiếu. Linux build/native proof đã có ở M1–M3 theo PLAN;
chưa release package/SBOM/license audit hoặc vận hành production.

## Nguồn đã khảo sát và quyết định tái sử dụng

[Survey hash từng file/commit/snapshot](evidence/rust-service/survey.json); raw ở
local/research/codex/rust-service-survey-r1/. Khảo sát theo câu hỏi, không claim đọc mọi DB/raw.

| Nguồn/locator | Dùng trực tiếp / chỉ mẫu | Phần phải đối chiếu và công việc tránh được |
|---|---|---|
| Repo này main c0c7b9478dcf5b639a76f67ee51a5f6b927cb0f9: PROTOCOL/RESEARCH/catalog; codec/service/room/fence/claims/health/store/tests/export/harness; CP178–212 | KR contract và sealed native/copy/model fixtures theo scope trực tiếp làm port target. Python là differential reference, policy vẫn design | Tránh RE codec/round/bom/rank lại. Opaque fixture headers không fullgrammar; source WIP không merged |
| Team WIP và [CP213 evidence](evidence/eight-player/README.md#cp213), frozen source | Free1–7/Manner4–4 ca native dương tínhs và alternatecopyManner1–7negative có hash riêng, dùng lập milestone8 | Code chưa main; baselineRust đầu chọn mergedtwo/fourprofile. Không bằng chứng native trên Rust hoặc interactiveC2Steamchange |
| boomkr-re@88e11ed30764c753a00f4f23a2f33dafadc4c890: kr.md, kr-data.md, kr-content.md, data/kr/bomber_attr.json và exp_table.json | Data locator/consumer seed, không port toàn bảng | Mixed VN/KR/web và HP?; wiki khác thời điểm00967. Phải đối chiếu field/units/IDs. Tránh giải lại pack/Attr/inventory metadata; correction theo nguồn mới |
| boom-re@9ac2075bbec0c16217741aaedea968cf685e39a9: items, oc_e-game, protocol, lobby, oc_d-login, oc_a-room; Rust proto/session/hub | VN luật/ý nghĩa theo provenance; connectioncodec/hub/input/runtime recipe chỉ mẫu | VN92/146,111/152/151,room89/loginlayout không KR99/153,118/159/158. Hub queue không giới hạn/single-room/arrivalID không production. Tránh tìm lại P2Parchitecture/itemfamilies/phương pháp trace |
| boom-rs@86cee6f0588c9a2e56d1547f8867a887970b1ebe: routerAGENTS/planningbắtbuộc/ARCHITECTURE/ADR0009-0011; server auth/identity/sqlite/session/secret | Mẫu accountnormalization/Argon2id/quota/sessionrevoke/migration/negative/log-health; không claim chạy tests repo đó lượt này | Postcard/InputCommand/Snapshot và authoritativeengine dành clientRustmới, không KRadapter. Tránh lỗi phiên khóa sau EOF/restart và registerquota khôngđếm; không nhập V1scope/map/4cap/khôngeconomy |
| boom-re-l10n@308dc28309c1c5ec982c2da56fb060be07d20cde: migrationREADME và decomp/item_kinds.json VN269 | Reference/deduplicatepath; itemfamilies đã nhập nguồn hiện hành | Không thêm catalog/contract song song; không kiểm kê từng icon/row lại |
| Archive26/10 README/KNOWLEDGE; [wiki owner-trusted](REFERENCES.md#owner-trusted-namu-wiki), mode/EXPtext | Archive điều hướng claim gốc; wiki chọn luật/stat/Free-Manner/progression sản phẩm | ObjectTable mô tả cũ đã bị consumer mới sửa; ưu tiên nguồn hiện hành. Wiki không wire/TTL/maxunits/rewardformula00967; tránh RE ý nghĩa luật chung, vẫn đo khác biệt build |

Hash chứng minh đã đọc bytes nào, không tự xác nhận semantic/runtime. Quy tắc nguồn boom-rs đã đọc để
khảo sát các repo thuộc router; scope/gate của dự án đó không trở thành yêu cầu của server KR.

## Ma trận hiện có → Rust → còn thiếu

Mức: S=KRstaticpartial; E=emulationchecked; N=native KR/Wine/copyđãkhai; T=syntheticapplication/realTCP;
D=emulator/thiết kế backend; V=VNreference. Chúng không thay nhau. Presence/mức chi tiết ở [COVERAGE](COVERAGE.md).
LAN/WAN netns theo [scope mạng](PROTOCOL.md#emulated-wan-acceptance), không nâng internetunknown.
Sẵn sàng hẹp là port profile có giới hạn, không fullsubsystem. KRE-005b/#14 demo/RE, KRE-003/#3 codec rộng,
KRE-002/#2 bootstrap rộng; KRE-074/#285 là epic triển khai theo PLAN; ma trận survey tạiCP213 giữ mốc nguồn, runtimeRust/M2 scoped ở [contract](PROTOCOL.md#rust-m2-acceptance).

| Feature/KRE → nguồn chuẩn | Presence, bằng chứng; implementation / integration / merge / gate | Chuyển Rust và giới hạn | RE tối thiểu / nghiệm thu Rust |
|---|---|---|---|
| KR-B001/B002; #2/#14 → [WCP](PROTOCOL.md#wcp), [K1](PLAN.md#k1-acceptance) | present; V/S/E/N; R2 offline qua 3 cold start. Impl=partial; integration=partial; main; K0/K1 scoped | Chuyển listener local và reply đã pin. Giữ provenance bản copy; không viết patcher/AC mới. Launcher/config rộng còn unknown | Không RE lại R2. Rust chạy đúng copy qua 3 cold start tới network/state đã định, có PID/API/isolation/cleanup; chỉ mở config nếu nó chặn slice |
| KR-B003; #3 → [codec/retry](PROTOCOL.md#login-retry-codec), CP057/180/181 | present; V/S/E/N/T; hello mode0→2/3, header37. Impl=partial; integration=partial; main; K2 profile | Sẵn sàng chuyển codec thuần, framing, CRC, typed fields, seq theo socket/chiều. Header chưa hiểu giữ opaque; không claim toàn mode/prefix/auth | Dùng oracle native cũ. Rust kiểm split/coalesce/bounds/EOF/CRC/seq reset và native retry cùng process. Encoder tự sinh không là oracle |
| KR-L001; #14 → [auth](PROTOCOL.md#login-auth), [291](PROTOCOL.md#own-profile-integration) | present; V/S/E/N/T; auth137/error1/name291. Impl=partial; integration=partial; main; K3 hẹp; chưa full authentication | Chuyển identity diagnostic trước. Backend account/DB làm độc lập; [CP216 echo/layout6](PROTOCOL.md#auth-channel-echo) đã đo, còn credentialverification, quyền sở hữu identity và liên kết nhiều socket chưa đủ | Đo route credential/token local, sai/expired/revoked/replayed token và tráo channel giữa hai account. Rust phải bind đúng player từ session, không chỉ gửi login-result thành công |
| KR-L002/L003 → [sảnh/cache](PROTOCOL.md#channel-lobby), [cold route](PROTOCOL.md#channel-cold-route) | present; V/S/E/N; một channel, cold/missing/warm/version negative. Impl=partial; integration=partial; main; K3 scoped | Chuyển redirect và cache profile đã đo. Multi-channel, caption, toàn bộ cache và room list động còn thiếu | Rust chạy cold/missing/warm và lỗi version/stage trên native. Chỉ RE row/list/update/page khi làm lobby động |
| KR-R001/R002/R004 → [room](PROTOCOL.md#room-player), [mode/host](PROTOCOL.md#current-team-mode) | present; V/S/E/N/T; fixed roster 2/4/8, Ready, host Back/rejoin. Impl=partial; integration=partial; main; K4 scoped. Delta team CP213 là WIP chưa merge | Chuyển owner phòng, membership, slot, host, Ready và các reply đã đo; roster cố định trước. Password/full/kick/chọn đội tương tác chưa đủ | Rust hai native thấy cùng phòng/slot/host, rejoin/Ready/start lặp đúng. Team UI phải có KR OS-click→C2S→consumer; VN89 chỉ là đầu mối |
| KR-R003/F001 → [start/clock](PROTOCOL.md#start-reply), [hot gate](PROTOCOL.md#loading-hot-gate-contest) | present; S/E/N; map278/ExVillage01, 99, wait-all, clock/input recipe. Impl=partial; integration=partial; main; K4/K5 scoped | Chuyển loading barrier, deadline, membership snapshot và round epoch. Không ép state/clock mới vào client | Rust 99 của cả hai→GO/native clock; thiếu/trễ ACK, EOF khi load, timeout/reset. Map khác chỉ mở khi có đúng consumer/dữ liệu |
| KR-F002/F015 → [peer](PROTOCOL.md#peer-duo), [WAN](PROTOCOL.md#emulated-wan-acceptance) | present; V/S/E/N; 99→153, direct UDP; CP209 có mesh8/56 chiều. Impl=partial; integration=partial; main; K6a/b/c scoped; Internet unknown | Sẵn sàng chuyển endpoint exchange. Giữ UDP trực tiếp giữa client; không thay bằng snapshot server. NAT tĩnh là design; full UDP/discovery/relay/auth chưa đủ | Native Rust A↔B: capture→API/key/apply→tọa độ tiến. Thử subnet/NAT/UDP delay-loss/EOF theo recipe. Chỉ RE relay/discovery khi deployment cần |
| KR-F003/X002 → [key/fence](PROTOCOL.md#udp-generation), [old UDP](PROTOCOL.md#direct-oldudp-contest-eof) | present; S/E/N/T+D; exact old frame bị loại ở vòng mới, có current positive. Impl=partial; integration=partial; main; K6 scoped | Chuyển ticket/epoch do server sở hữu và giữ giới hạn 256 byte-key trước khi fail closed. Không suy queue cũ tự sạch | Rust old callback/timer/ID reuse, UDP cũ/current positive, TCP queue của membership cũ. Phải đo trước khi nới low8 wrap hoặc grammar |
| KR-F004 → [bom](PROTOCOL.md#bomb-serial), [four owners](PROTOCOL.md#four-owner-bombs) | present; V/S/E/N/T; 2/4/8 owner, cap/power riêng, 159/158/fence. Impl=partial; integration=partial; main; K5 scoped | Chuyển validation118, reservation/ID, confirm/explode/timer. Fuse6 và admission cap giữ design hiện tại; vị trí actor/full pool/races còn thiếu | Rust hai native đặt bom cả hai owner, ID/entity trùng khớp, cap1 khác cap2, nổ/reset/EOF khi bom chờ. Không clone stat B sang mọi slot hoặc đổi fuse trong port |
| KR-F005/F006/F007 → [terrain/basic](PROTOCOL.md#hidden-cell-admission-current), [families](PROTOCOL.md#item-families) | present; V/S/E/N hẹp, hai người/3 basic effects/reset. Impl=partial; integration=partial; main; K5 scoped. Chưa tích hợp đầy đủ terrain/item4/8 | Chuyển pinned Box/list2, reveal142, claim124/165, generation và stat cap đã biết. Không tự xác nhận drop weights, geometry tổng quát, distance/auth/fairness/freshness142 | Rust basic hai vòng: effect thật, tranh item một kết quả, reset. Map/vật cản/validator mới phải đo delta trước enforce; không lấy winner synthetic làm proof |
| KR-F008/F009 → [kim reset](PROTOCOL.md#needle-round-reset), [cứu](PROTOCOL.md#rescue-two-results) | present; V/S/E/N hẹp theo pair/single credit/report. Impl=partial; integration F008=unknown/F009=partial ngoài ca dẫn nguồn; main; K5 scoped; không proof4/8 | Chuyển riêng sau basic; credit/report policy giữ design. Client tính contact/trap/death một phần; fixture145 không là inventory authentication | Rust native pair: cứu hai chiều, chạm địch/chết, kim hết credit, reset. Không mở lại consumer đã giải; 4/8 integration, inventory credit và race khác cần proof tương ứng |
| KR-F010–F014 → [families](PROTOCOL.md#item-families) | present/candidate theo ID; V static/S partial; E/N unknown. Impl=unknown; integration=unknown; delivery=unknown. K5 passed không xác nhận các nhóm này | Chưa đủ chuyển hành vi. Có thể giữ taxonomy/backend definitions nhưng disable effect chưa rõ; không suy từ tên/icon | Mỗi family cần consumer + một ca dùng/không đủ điều kiện/consume/expire/peer/reset. Chọn theo sản phẩm, không RE mọi row như cơ chế độc lập |
| KR-X001/X002/X003 → [result](PROTOCOL.md#current-basic-cycles-291), [four abort](PROTOCOL.md#four-member-abort-room) | present; V/S/E/N/T; 175/180, rank/room/reset/Start3, host/guest EOF. Impl=partial; integration=partial; main; K5/K6/K7 scoped | Chuyển result builder, life/witness/settle/abort/fence hiện tại. Chưa durable results/rewards; ba survivor về phòng có fence không nghĩa ba người chơi tiếp | Rust hai normal results/reset/Start3; EOF không late158; bốn người kiểm host/guest rớt và survivor cleanup/promotion. Fresh server, reconnect, account resume, match restore phải tách |
| KR-S001/S002 → [health/identity](PROTOCOL.md#four-health-identity), [restart](PROTOCOL.md#four-netem-restart) | Native presence của S002=unknown; D/T/N cho service. Impl=partial; integration=partial; main; K7 scoped. JSON pin/67bundle/health/new server/4actual đã đo | Chuyển metadata/budgets/deadline/package. JSON pin giữ identity operator không phải account DB. Rust build/native M1–M3 đã có; chưa package/restore/load acceptance | Rust package Linux chạy sau relocation, hash/config đúng, fail trước listen với input lỗi; nonreader/quota/fault/shutdown và fresh server/same DB/native4 dưới impairment |
| Backend account/profile/inventory/reward/admin; KRE-074 → [gaps](RESEARCH.md#rust-service-gaps) | CP236 account/profile/session/SQLite/adminCLI đã triển khai/test/merge; CP254 có native fixed2account lobby bridge, native negatives/accountroom-game/generalprofile, inventory/reward và ops chưa đủ, M5 vẫn partial. UI/asset presence không proof toàn hệ thống | Backend design làm độc lập qua CLI/API; không gửi unknown wire. Catalog30 feature không phải danh mục đầy đủ của production service | DB thật: concurrent/transaction/replay/crash/restore. Mỗi client projection/login/equip/balance/reward cần KR observation và negative trước expose |


## Module và dòng dữ liệu

Cấu trúc dưới **đề xuất**, chưa tạo:

    rust/Cargo.toml, Cargo.lock, rust-toolchain.toml
    rust/crates/kr-wire/src/
      primitives, frame, codec, sequence, kr00967/{bootstrap,login,room,battle}
    rust/crates/kr-service/src/
      main, config, transport/{tcp,udp_endpoints}, connection, session
      channel, lobby, room, round, policies/{bomb,item,terrain,life,result}
      application/{accounts,profiles,inventory,rewards}
      store/{sqlite,migrations}, admin, telemetry
    rust/crates/kr-wire/tests/{native_vectors,stream}
    rust/crates/kr-service/tests/{lifecycle,differential,store_faults}
    server/     Python giữ nguyên
    tools/      Wine/RE/probe/selectivecheckpoint/differential drivers
    docs/       contract và nguồn chuẩn chung
    local/      copy/raw/snapshots/DB/trace, không Git

kr-wire không Tokio/DB/domain; parse trả lỗi+bounds, không tự authorizecommand. kr-service có library
cho deterministiclifecycle và binaryLinux. Chưa cần kr-game/kr-storecrate, engine/render/clientRustmới
hoặc boom-coredependency. Unknownfield giữ kiểu opaque+hash profile/limits, không tự đặt tên auth/session.

CP237 đã thêm [floor pickup](PROTOCOL.md#rust-movement-floor) từcùng582pin vào roomowner/clone+reservequeues,
không nhập DB/inventory/reward vào worlditem. NativeUDP/movement/floorRoller đã đo; [CP238](PROTOCOL.md#rust-bombs) thêmroomledger/monotonicID/membership-round-allocationticket/fuse6/tworecipientwrites; [CP239](PROTOCOL.md#rust-terrain) thêmterraincandidate+fullroster158commit/currentcell142/hiddenclaims; life/result-reset cònthiếu.
Opaque123/10 chỉboundedobserve, không contractsemantics/authority; giữ unknown trong RESEARCH.

Dòng dữ liệu: nativeTCP→reader/codec mỗi connection/direction→stage/session adapter→owner phòng→
policy/ticket timer→boundedwriter→native. Applicationcommands/resultIDs→boundedDB worker→transaction/outbox;
adminCLI/HTTPSdesignAPI đi qua cùng application layer. PeerUDPdirect theo153, không phải Rustsnapshot.

Roomowner xử lý start/leave/claim/report/timer tuần tự, không giữ lock qua await.
RoomId/MembershipGeneration/RoundId/AllocationTicket/ConnectionId khác wirelow8key/bombID.
Queueentry gắn recipient connection/session/membership/epoch; kiểm lại trước bắt đầusend, không đổi
ticket cũ thành mới. Partialwrite đã ra wire không rollback bytes: mark uncertain delivery, fence round,
đóng đúng recipient và cleanup theo recipe. Boundedqueue/timeout là design mới phải negative/native
regression, không tự suy tương đương Pythonfiniteblocking2s.

Reader có byte/frame/rate/decodebudget/maximumallocation, seqriêng; validframesai stage không đổi
state hoặc phân phối sangaccountkhác. Timer dùng monotonictime/injectedclock; expiry/auditpersistUTC,
clockrollback không phục hồi revoked session. Shutdown ngừngaccept/fence/boundeddrain/revokeinstance/
closeownedtransports/DB. UDP không có fullepoch đã hiểu thì server không hứa lọc mọi P2Poldpacket;
giữ measuredkeyrotation/exhaustion/peerlifetime và nativeoldframecontrol, không “đãclearqueue” giả proof.

## Account, identity và dữ liệu bền vững

CP236 hiện thực lát cắt design dưới bằng [backend account](PROTOCOL.md#rust-account-backend) và
[CLI](../rust/README.md#account-admin-cli); schema1 chỉ account/player-name-version/session/runtime-state/audit,
không xem toàn bảng design phía dưới đã triển khai. Identity persistence/backend authorization đã testSQLite,
CP243đãcóboundedworker/handoff/exacttokencompensation, CP244 schema2/idleTTL/per-worker rate; CP254 fixed2native account lobby/private launcher đã có, còn native negatives/lifecycle/accountroom-game/generalprofile/character/XP/inventory/ledger/outbox/restore. M5 độc lập M3 theo PLAN.

Tách ba biên: **backend nội bộ design**, **admin/web/launcher interface mới design**, **wire KR phục hồi**.
CLIcreateaccount hoặc webHTTPS không là màn đăng ký/login gốc đã chứng minh. Localfixtureauth137/291,
acceptorder/sourceIP không chứng minh chủaccount. Một gamesession có nhiều bootstrap/login/channel
connections; đóng bootstrap bình thường không revoke cảsession. Chọn policy ban đầu một gamesession/account,
refuse secondactive; bind transports bằng authenticatedlease, không tin accountID tự khai.

Backendaccount/Argon2id/DB/profile/admin có thể xây song song gameplayport. Bridgeclient cần local
experiment bằng credential/token côngty; nếu KRchỉnhậnprovidertoken, launcherlocal design chỉ được cấp
token sau khi chứng minh routenhận/kiểm/bindKR. Không nhét field vào37header hoặc tự thêm clientUI.
Nếu chưa có bridgean toàn, gateaccount-backedclient giữ blocked; prototype ghi diagnosticunauthenticated.

| Dữ liệu design / invariant | Ghi/đọc và kiểm quan trọng |
|---|---|
| accounts: id, normalized_login UNIQUE, password_phc, disabled, credential_version, created_at | Argon2id/saltrandom/PHCparams-version; benchmarkmemory-time/quota, boundedhashworkers. Dummyverifyunknownuser/uniformfailure. Không log plaintext/PHC/token; reset/adminaudit, entropyfailurefailclosed |
| players: id, account_id UNIQUE FK, kr_wire_id UNIQUE, display_name, profile_version | Account→persistentplayeridentity; A..H/slot khôngaccountID. Namelength/encoding theoconsumerKR, DBtext khác wirebytes; displayname khôngownership |
| sessions: id, accountFK, token_hash UNIQUE, boot_id, expires_at, revoked_at, credential_version; ephemeraltransportleases | CSPRNGsecret/hashatrest/absolute-idleTTLdesign; rotate/revoke/ban/logout/rate. Oneactive transactional, crashbootrevokeabandonedleases. Không passwordtrênlegacyTCP khi protectedroute chưađo |
| profiles: player_id, character, xp, level_policy_version, revision | KRpinnedcharacter/stat; wikiXPthreshold chọnversiondesign, không giả720levels/XPfielddisplay00967. Haiaccount/changespersist/freshserver/nativeprojection sauwiremapping |
| item_definitions: version, internal_id, kr_id?, family, limits; inventory_stacks: player/item/quantity CHECK>=0/revision; equipment | Persistentitem khác worlddropkind/index và perroundcredit. Unknownkr_idnullable/disabled. Concurrentequip/spend kiểmowner/version/quantity/expiry trongtransaction |
| matches: id, boot_id, room, round_nonce UNIQUE, policy_hash, status; results: match_id UNIQUE, evidence_digest, outcome_version | StablematchID trướcoutcome/broadcast, không chỉpeerkey/clientreport. Abort/uncertain khôngreward; report-derivedlife chưa đủ trustedmoney |
| reward_grants: UNIQUE(match_id,player_id,reward_slot), policy_version bất biến; ledger: entry_id,grant_id UNIQUE,delta,balance_revision; outbox | Acceptedresult+grant+balance+consumption+outbox mộttransaction; crash/retry khôngdoublecredit. Outboxatleastonce, dedupegrant/revision; DB idempotency khác việc packet được giao đúng một lần |
| admin_audit: actor,operation,target,reason,before-afterhash,UTC,request_id UNIQUE | Ban/unban/reset/reconcile, correction bằngcompensatingentry khôngdeleteledger. Auditfail chặn mutation cần audit; logsbounded/redacted |

Không lấy công thức rewardgốc từ EXPtable/wiki. Portđầu giữ reward0/profile0explicit;
durabledryrunreward/admingrant là design versionriêng. Bật rewardthật phải chốt formula/eligibility/currencycap/
spend/expiry/refund và đo KRprofile/result/inventoryconsumer. Clientreport không đủ nguồn tiền tin cậy;
trước validatorđủ chỉ nonredeemableinternal/audit/reconcile. Exactlyonce ở đây là logicalgrant theoDBkey,
không hứa TCPpacket đến client đúngmộtlần.

Idempotency key không được thay khi đổi công thức thưởng: policy version được pin vào match và kết quả lúc
finalize, không là cách tạo grant mới cho cùng match/player/reward slot. Retry bằng binary/config mới phải đọc
quyết định đã lưu; nếu khác policy thì từ chối, không tự tính lại. Sửa thưởng đã cấp dùng compensating entry có
request ID riêng và audit. Acceptance gồm replay cùng match sau đổi policy version để phát hiện double grant.

Migrationversion/checksum/forwardonly, khóaoneinstance/migrator, failbeforelisten với futureversion/corrupt/
identity sai; khôngdowngrade tựđộng. SQLiteonlinebackup hoặc quiesce+WALcheckpoint; không copy riêngfile DB
đangWAL rồi gọi backupđủ. Thử restore thưmục mới→migration/foreignkey/integrity/counts/ledgerbalance/
noduplicategrants/nativefreshlogin-profile-newmatch, đoRPO/RTO. Binaryrollback chỉcompatibleschema hoặc
restore đã kiểm; không xóa ledger để oldbinaryboot.

DBunavailable/busy/diskfull khônggiảlogin/purchase/rewardsuccess. Ephemeralround có thể xong display với
uncommittedstatus nhưng rewardfenced; internalservice cần durablecompletionpolicy/boundedpending/outbox.
Trong chế độ dịch vụ bền vững, commit kết quả/grant/outbox trước khi xác nhận hoàn tất có thưởng;
DB lỗi thì ngừng admission, giữ pending hữu hạn hoặc kết thúc không thưởng bằng flow đã kiểm trên KR.
Không gửi normal durable-success rồi chỉ giữ kết quả trong RAM. Demo không thưởng có thể dùng policy khác
được ghi rõ. Crashtrướccommit retry samematchkey; saucommit/trướcnotify readresult/outbox khôngnewgrant. Reconnect
sửa projectiontừDB sauauthmới, khôngresumeactor/timer/match mặcđịnh.

## Vận hành và phạm vi sản phẩm

Configvalidatebeforebind: listeners/allowlist/profile-fixtureSHA/portstage/revision policy/queue-rate-timeout/
đường dẫn DB/adminsocket/identitypolicy; secretsfilepermission/envriêng ngoàiGit/log. Logcorrelation build/git/config/
fixturehash/boot/session/connection/channel/room/round/match/grant/disconnectcause; redactcredentials/token/
rawauthbody. RawRE có retention/access riêng. Metricsboundedcardinality: connections/rooms/byte trong queue/
eventloop-timerlag/lỗi frame/DBlatency/rewardpending/outboxattempt/authfailrate, không label mọiaccount.
Health tách liveness và DB-config-listenerreadiness, readinessfail ngừngadmission; không healthloggameproof.

Adminđầu localCLI/Unixsocketpermissions/allowlistedoperations; HTTPS/auth/RBAC/CSRF khiweb. Tốithiểu
list/disable/ban-unban/resetpassword/revoke/reconcile/read-onlyroom-log/backup/restore và audit. Không arbitrarySQL/
remoteeval. Linuxartifact lockedtoolchain/deps/configschema/SBOM-license/SHA/pinnedfixturemanifest khôngclient/
asset/secretDB, sampleconfig/runbook; relocatable/nonroot/ownedstorage/SIGTERM-timeout-SIGKILL đo.
CI fmt/clippy/tests/oracle/storefault/package smoke; ghi đúngcheckcontexts thực. Staging→controlledrollout→
rollbackrehearsal, drain trướcupgrade; interruptedroundkhông cấp thưởng, khôngzero-downtime/HA/khôi phục trậnpromise.

Ưu tiên: Battlebasic/internalaccounts trước; chatroom khiwire/rate/moderationđủ; friends/whisper/social
sau stability; shop/premium/economymởrộng sauinventoryledger; guild/mission/boss/revival/events/modesphụ
cầnconsumer/runtime riêng. Wiki giúp chọn luật, không mở toàncatalog thànhdependency cơ bản.
Không MMOsharding/payment/HA mặcđịnh. Điều kiện demo/internal/widerrelease ở [PLAN](PLAN.md#rust-service-exit-criteria).

## Chiến lược Python ↔ Rust và chứng minh

Pinmain/copy/profile/inputrecipe/runhash; CP213WIP chỉchọn sau delivery riêng. Capture/replay normalize
nondeterministicclock/IDs bằngwhitelist có lýdo, không xóa khácbiệt opcode/seq/key/ownership/errors/results.
Native/binaryoracle inputspinned; Rustcapture có hash buildriêng. RegeneratedRustfixture chỉregression.
Differential codec/stream/errors/seqreset và command→state→outbound/timer/rejectkhông đổi state/lifecycle/
actualsocketpeer không đọc/ghi một phần/lỗi DB. Haiimplementationđồngý chỉparity, khôngproofKRcontract.

NativeKRparse/cursor/CRC/actualC2S/actorstate/OSinput/currentprogress/result-room/two-roundreset trênRust
mới chứngminhcompatibilityscoped. NativegetterUnicorn táchruntime. Không chạyhaiwriter cùngDB/room/port;
shadowread-only/replay khôngdoublebroadcast/award. Handoff cảprocess hoặcroom tạiended/emptyepoch, không đổi
backend trongrun. RetirePythonserving từngslice sau acceptanceRust/native/negative; harnessRE vẫngiữ.
RollbackPythonpinned độc lập, schemaRust khôngghi đè fixedpinPython. Backendauth/DBdesign không lấyPythonhiệncó
làmoracle. Policy/authoritythayđổi làtaskriêng saubaselineport.

| Khái niệm | Proof riêng, không thay nhau |
|---|---|
| Freshserver | BootID/process/listeners/storemới, khôngoldtimer/queue/lease; chưaclientauth |
| Freshlogin | Nativeclient/sessionmới authlocal→identity→channel; khôngreconnect/resumeprocesscũ |
| TCPreconnect | Socketmới/codecseqreset/statebind/timeout/lateoldconnectionfenced; khôngtựkhôi phục phiên tài khoản |
| Accountresume | Credential/token hợp lệ/ownership/expiry/revoke/oldtransportfencing đãđoquaKR |
| Matchrestore | Durableworld/timer/RNG/epoch/peerrekey/nativejoin-loading; ngoài scopeđầu, khôngsuytừDBaccountpersist |

Không hứa ngày/sốCP/phầntrăm. Ướclượng sau khi lát cắt đầu có đotải/reliability/REbridge, nêu assumptions.
Internet thật chưa kiểm; Internet/ISP/CGNAT làbacklogriêng, khôngdependencyngầm K6c/K7 đãnghiệmthu.
