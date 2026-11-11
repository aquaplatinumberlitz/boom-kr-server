# Câu hỏi RE của server KR

Đây là backlog riêng KRE. Các ID Q-022…Q-029 được giữ để truy nguồn; không thuộc gate V1 Rust.


Mở theo yêu cầu trực tiếp 26/10/2026. Đây là câu hỏi của **`KR-00967-EMU-1`**, không đổi câu hỏi/gate
V1 Rust. [Lộ trình KRE](PLAN.md) giữ thứ tự
task và điểm dừng; [contract](PROTOCOL.md) giữ ma trận/nguồn.
KRE-001/002/003 giữ backlog theo PLAN và issue hiện hành; owner khi mở nhánh: từ28/10 Codex solo RE/runtime và tự kiểm theo
[miễn review của chủ dự án](evidence/checkpoints/review-waiver.json). Report dsh/Claude cũ giữ lịch sử.
Review plan nguồn đã xong; các câu hỏi dưới đây vẫn open cho tới khi có evidence trả lời.

## Câu hỏi hiện hành

Thứ tự task chỉ ở [PLAN](PLAN.md#current-order). Tra phạm vi đã biết qua
[coverage/cây chức năng](COVERAGE.md); bảng dưới dẫn đến khoảng trống/phép đo theo cơ chế.
Nguồn cũ và run/checkpoint giữ trong phần lịch sử bên dưới và manifest; kết luận tại CP cũ không
được đọc thành trạng thái hiện tại. Tra mức RE/implementation/runtime/gate theo từng chức năng ở
[coverage](COVERAGE.md), contract/giới hạn tại link của từng cơ chế trong bảng gap. Không duy trì
một danh sách tiến độ theo CP khác tại đây; [README](../README.md) giữ mốc hiện tại,
[PLAN](PLAN.md#current-order) giữ thứ tự. Fullauthentication chưa nghiệm thu. K6c WAN giả lập/K7 đạt CP212 theo scope PLAN; **Internet thật chưa kiểm** và không dependency ngầm của các gate đã đạt.

| Câu hỏi / owner | Trạng thái | Khoảng trống và phép đo hiện hành |
|---|---|---|
| <a id="q-022"></a>Q-022 bootstrap / Codex tự kiểm | open | [ProcessPatch/control](#gap-process-patch), [WCP/config](#gap-wcp-config) |
| <a id="q-023"></a>Q-023 codec / Codex tự kiểm | open | [Socket/seq/mode/bounds](#gap-socket-codec), [service actual frame](#gap-diagnostic-service) |
| <a id="q-024"></a>Q-024 login/kênh / Codex tự kiểm | open | [Auth/identity](#gap-auth-identity), [sảnh](#gap-redirect-lobby), [cache](#gap-profile-cache) |
| <a id="q-025"></a>Q-025 phòng/load / Codex tự kiểm | open | [Tạo/join](#gap-create-join), [roster](#gap-roster-ready-host), [barrier](#gap-map-spawn-loading), [membership](#gap-room-membership) |
| <a id="q-026"></a>Q-026 authority/trận / Codex tự kiểm | open | [Bom](#gap-bomb-lifetime), [chain/block](#gap-blast-block-chain), [pickup validator](#gap-floor-pickup), [item](#gap-item-stats), [contact](#gap-contact-state), [result/reset](#gap-round-reset) |
| <a id="q-027"></a>Q-027 peer / Codex tự kiểm | open | [Movement](#gap-peer-movement), [freshness](#gap-udp-freshness), [relay/NAT](#gap-peer-relay), [EOF](#gap-disconnect-replacement) |
| <a id="q-028"></a>Q-028 profile/core / Codex tự kiểm | open | [Authority matrix](PROTOCOL.md#authority): audit creator/simulator/validator/broadcast/peer-editable; lựa chọn lai/shadow core là design, chưa là contract client |
| <a id="q-029"></a>Q-029 vận hành / Codex tự kiểm | open | [Hardening/demo nhiều máy](#gap-service-hardening); theo gate K7, không suy từ TCP accept |

<a id="gap-codec-retry-followup"></a>
### Codec retry đã đo CP181; giới hạn còn lại

[Contract/sequence native](PROTOCOL.md#login-retry-codec) giữ nguồn chuẩn: validresult1→UI1002→
sameprocessTCPloginretry→auth137cursor/errors0→channelreset. [Controls/hash](evidence/live-codec-stream/README.md#cp181)
giữ fatalCRCcleanup/WineDbgHWwatchpointfailure/stubordinalcontrol. Không lặp fatalclose hoặc stubDWK.
K2profile được nghiệm thu theo [PLAN](PLAN.md#k2-acceptance); không fullgrammar/allmodes/headerauthority/
accountresume. Widercodec chỉ mở khi có trigger cụ thể từ K3; không quét mọi variant trước sảnh.
Owner/tự kiểm Codex, review waived; AUD062 quaCP212, periodic audit kếCP216.

<a id="gap-lobby-identity-followup"></a>
### K6c WAN giả lập/K7 đã nghiệm thu scoped; tiếp biến thể team theo PLAN

[CP207contract](PROTOCOL.md#four-netem-restart), [raw/hash](evidence/peer-endpoints/README.md#cp207),
[K7đốichiếu](PLAN.md#k7-evidence-review), [thứtựduynhất](PLAN.md#current-order). Same67bundle/store sauCP206stop:
4nativefreshsessions/2normalopposite2v2outcomes/resetStart3 dưới4recipientUDP-only40±5ms/loss5%, TCPbaseline;
24remotecurrentprogress/hai epoch, PCAP6Aedges/cảhai epoch, qdiscdelay/drop/cleanup0. Current214kernelTCP vẫn đúng
recipient/Bfollowup1/fence/nocommit; khôngqueuednonblockingbenchmark.
ReuseCP195TC/CP206four/profile/health/copy, VN chỉ TCPendpoint/P2Parchitecture; khôngnativefunctionRE mới.
CP212 [owner directive](evidence/owner-directives/emulated-wan.json) thay môi trường K6c bằng WAN giả lập;
[đối chiếu từng tiêu chí](PLAN.md#k6c-acceptance) đủ và [K7 đánh giá lại độc lập](PLAN.md#k7-evidence-review) đạt scoped.
**Internet thật chưa kiểm**, backlog riêng bên dưới; không phải dependency K6c/K7. Không cần lặp các run đã đủ proof.
[CP208](PROTOCOL.md#eight-session-preview) đã dùng lại capacity/teamguard/codec/row26/peer26flags,
đo native8consumer trên hai actual + sáu synthetic và adapter32TCP/487checks; không tìm lại nhánh đã giải.
Tám actual auth/name291/ownslot/99/285-205/8actorsclock đã qua R2/1468joins/192getters/cleanup0.
[CP209](PROTOCOL.md#eight-mesh-owner-bombs) giải 56remoteprogress/8OSwindows, baseline stat8/8actual118
và run riêng8owner159-entity/158 đủ8view. Ecapacity2,G/Hpower2 là delta thật, không clone defaults4.
[CP210](PROTOCOL.md#eight-two-team-cycles) giải fixed8life→7peer/rank8/175180/reset và same8twoopposite4v4outcomes/
Ready2–3/Start3;4351joins/128copiedgetters. Reuse cached175parser và actual12717/19từCP209,
không codec/rowRE mới. Reportderivedlife/rank/economy0/settle8 vẫndesign, không worldauthority.
[CP211](PROTOCOL.md#eight-portable-battle) giải native8portable:71pins/47aliases/18bundle/767packagedTCP,
Python-I/relocation/modulepath/sourcehash-config byteproof vàsame8two4v4cycles/resetStart3/4417+112joins/cleanup0.
[Python source CP213](evidence/eight-player/team-code-delivery.json) được giao riêng trước Rust variants; không cần RE lại wire hoặc chạy lại Python positives.
[CP213](PROTOCOL.md#eight-team-variants) bổ sung actual8Free1–7 và Manner4–4, mỗi run hai oppositeoutcomes/reset/Start3;
4344+125 và4357+124gameplay/modejoins,128copiedgetters/run. Selector giữ0/1 qua86/87/180 ở cả8view;
không cần RE lại175/codec/knownbalancedguard. Fixedoperatorpreset không là interactiveC2Steamchange.
Ca8actualManner1–7 copy4b R1E/R2F preStartSIGABRT; R2FactualemptyJSONsize0/cap15→expectedvalue tại96A4EB/96E2D0,
tái dùngCP073201. Nguồn HTTPinput rỗng cònunknown, không quy cho locale/teams. Alternatecopy8d có8nativecount8/selector1/balancereject/return7/no104 vàcleanup0; guardkhônghit, khôngstartupfixclaim. Nativeguardreturn7/no104 đã có CP163
với mộtactual/tám slot tổng hợp giữ nguồn riêng; alternatecopy8d đã bổ sung8native negative, không gán lêncopy4b.
Nhánh emptycontrol khép hẹp; không còn active retry hoặc startupfixclaim. InteractiveC2Steamchange/KRopcode-layout
vẫn unknown, chỉ mở theo trigger RUST-M4/M7 trong PLAN; VNroom89 là seed, không portopcode/offset.
Tại CP213, đợt khảo sát ưu tiên planning-only; [owner implementation09/11](evidence/owner-directives/rust-implementation.json) đã mở code/nativeRust. Thứ tự hiện hành ở PLAN.
Unknownninthsource packagedTCPnegative khôngactual9CA/generalroomfull/auth. Full4/8items-terrain-kim-cứu/ability/
raylength/accountresume ngoài proof. AUD062qua212/kế216; Codex selfcheck/reviewwaived.

<a id="gap-real-internet"></a>
### Backlog riêng: Internet thật chưa kiểm

Owner cho [WAN giả lập](evidence/owner-directives/emulated-wan.json) nghiệm thu K6c; không cần externalendpoint
để đóng K6c/K7. Phần chưa biết: actual ISP/CGNAT/mapping-filtering/PMTU/firewall/topology Internet, NAT discovery
hoặc relay gốc. Khi chủ dự án ưu tiên backlog này, phép thử nhỏ nhất là hai endpoint do dự án kiểm soát,
khai topology/routing/NAT/impairments thực rồi đo directUDP hoặc relay đã chọn/nativeconsistency/two cycles/EOF.
Không gọi run cô lập cũ là Internet thật và không thêm phần này trở lại dependency demo đã nghiệm thu.
Owner/tự kiểm Codex; chỉ mở theo thứ tự PLAN, không dựng plan riêng.

<a id="rust-service-gaps"></a>
## Unknown chặn dịch vụ Rust — KRE-074, đang triển khai

[PLAN](PLAN.md#rust-service-milestones) giữ thứ tự/acceptance, [kiến trúc/ma trận](RUST_SERVICE_ARCHITECTURE.md)
giữ design/readiness; không mở toàn client trước port. Chủ nhiệm/tự kiểm Codex, review độc lập waived.
Nguồn khảo sát/pins ở [survey](evidence/rust-service/survey.json); CP212 gate demo không xác nhận fullauth/DB/economy.
[CP252 native9](PROTOCOL.md#rust-ninth-admission) đã có một KR thật/eightTCPsynthetics/actual82→native87generic2/
no585-noRoomgrant; không lặp genericparser hoặc ninthsocket-only negatives. Generalvacancy/replacement/9native cùng lúc
chưa đo và không suy từ probe. M4 còn Freehaioutcomes/selector0persist. [AUD072](PROTOCOL.md#argv-provenance-correction)
sửa scalar32unused/actualold13; control actual32 (independentCAargv A–G, Hkhôngobserved) vẫn Dpost17preRoom, không đủ
sửa startup. Không retry khi chưa có observable mới. PLAN chuyển M5 worker-to-KR/ownershipbridge độc lập với Free;
DB/KDF/session/worker/idleTTL/rate đã có; [CP253 admission library](PROTOCOL.md#rust-account-bridge) thêm owner/generation/oneuse32 và execution-time UTC. Còn listener/profilewriter/launcher API và native hai account; không làm lại backend hoặc quét SDKgraph. Accountresume/matchrestore khác freshlogin.
CP254 account listener/Unix launcher ở [account serving](PROTOCOL.md#rust-account-serving) đã có16ca TCP/82Rust/11guard và native2account positive/18exactTCP; native negative/lifecycle/account room-game còn thiếu. Next perPLAN, không mở room/game hoặc SDK graph từ kết quả TCP này.

Trạng thái lịch sử tại [CP251](PROTOCOL.md#rust-eight-teams): fixed1v7 code/TCP và native8Manner1v7negative trên8d; Freepositive2outcomes/selector0persist chưa có do copy4bD/8dGpost17preRoom. CùngCP250HGfamily/causalunknown, không retry hoặc SDKgraph khi chưa có observable mới; Tại mốc đó PLAN chuyển native9admissionbridge/consumer độc lập; CP252 phía trên đã đo gap này. Raw/hash/failedprebindchecker-harness được giữ, không nâng M4.

[CP231](PROTOCOL.md#rust-foundation) đã có M0/M1; [CP235](PROTOCOL.md#rust-m2-acceptance) chốt M2 scoped theo checklist:2actualfixedroom/lifecycle/barrier và healthyBsession sau EOF. [CP237](PROTOCOL.md#rust-movement-floor) có nativeUDP/movement hai chiều+floorRoller; [CP238](PROTOCOL.md#rust-bombs) thêmhaiownerbasicbom118/159/158; [CP239](PROTOCOL.md#rust-terrain) thêmterrain195flags/revealhidden27-28/ba165effects; [CP240](PROTOCOL.md#rust-life-results) thêm hai native outcome/reset/Ready2-Start3; [CP241](PROTOCOL.md#rust-return-cleanup) có activehostEOF→nativeRoom/chief/fence và dead-owner/drain kernelcontrols; [CP242](PROTOCOL.md#rust-m3-acceptance) chốt M3 scoped qua nativeitemreset/OScontest/exactoldUDP-currentpositive và queue controls; [CP243](PROTOCOL.md#rust-account-worker) có boundedworker/handoff/compensation; [CP244](PROTOCOL.md#rust-account-idle-rate) có idleTTL/rate/schema2; [CP245](PROTOCOL.md#rust-four-startup) có native4startup/room/loading/12remoteprogress; [CP246](PROTOCOL.md#rust-four-owner-bombs) thêmfour-ownerbom/knownstats/16native159158; [CP247](PROTOCOL.md#rust-four-life) có17/19→3peer170/native56→1/all4states và syntheticclear21completion; [CP248](PROTOCOL.md#rust-four-results) đã có same4opposite2v2/rank4-native175180/reset-Start3; [CP249](PROTOCOL.md#rust-four-abort) thêm scopedguest-hostEOF/unrankedreturn/threeRoom/chief; [CP250](PROTOCOL.md#rust-eight-baseline) có8Manner4v4/loading-mesh56-statbom8/life-rankreset; CP251 đã portfixedteams/nativeMannernegative và CP252native9bridge scope1KR+8synthetic; M4cònFree2outcomes/selector0persist/bridgeM5 riêng; [PLAN](PLAN.md#current-order) giữ thứ tự, không broadRE. Hai-client dùngfirstCRCrequest/endpoint/stagelease; WCPport biếnđổi đã giải theoCP006/103, khônglặpportonly. DiagnosticIP/ancillary shape policy chưa accountauthentication; unknownmode/header giữ giới hạn.

| Câu hỏi | Có thể chọn backend policy design ngay | Bắt buộc đo KR / phép thử nhỏ nhất |
|---|---|---|
| RUST-Q1 authbridge/ownership | Account→player schema, Argon2id/PHC, onegamesession, expiry/revoke/ban, CLI/HTTPS riêng; accountDB không chờ BattleRE | [CP225](PROTOCOL.md#auth-fresh-process): processB mới cùngaccount có native0/ID-name-echo6/67/newlease-session/Aowner8s; badfresh32token→native1/nochannel/noBsession/Aowner45s. Priorselector11fromgetter/storebound, setterrequest1notreadback. SameprocessfatalCP224/before17receiverCP220blocked; chỉ mở khi có input/observable mới hoặc milestonechặn. Freshlogin khôngresume. [CP236](PROTOCOL.md#rust-account-backend) đã có SQLite/Argon2id/account-player/profile/sessionexpiry-revoke-epoch/CLI backend, 11tests+33CLIprocess; CP243đãcóboundedworker/handoff/exacttokencompensation; CP244đãcóidleTTL/rate/schema2; CP253 đã có library oneuse frontend32/redirect/channel owner và worker executionUTC/7ca6DB/3nativebodyreplay. Chưa listener/profilewriter/launcher API/nativeKRbridge/fullauth; đo TCP negatives rồi freshKR2account/ownIDname/channel swap/expiry-revoke/replay/DBqueue failclosed; [PLAN](PLAN.md#current-order) tiếp M4native và M5bridge độc lập, không đợi fullauth/quétSDK/VM/lặphello |
| RUST-Q2 dynamicroom/profileprojection | Registry/roomquota/sessionowner/maxslots/API kiểm quyền, DBprofile/name độc lập | [CP230](PROTOCOL.md#room-shared-cleanup): private nativeguestBack/EOF/hostBack/107IDclear/110UIchief; CP229join/move đã có. Đủ consumer đầu vào M2; M0→M1 không đợi module Python/reference hoặc roomintegration. Theo owner mới, chỉ RE discrepancy chặn lát cắt; typed seam/partialsend khi port phòng cần. Không broad RE/lặp solvedroom. Productionglobalroom/dynamicoccupancy/quota/thirdnativeotherroom/failure-send/restartfence cònthiếu; raw585zero không wireepoch; hostpostgetter unobservedbudget20 |
| RUST-Q3 inventory/consumables | Definitions/revision/ownership/quantity/expiry/reservation/transaction; worlddrop khác persistentstack; disabled unknownkr_id | Native query/equip/use/request/confirm/count/effect/reset/freshlogin cùng1knownitem. Kim145fixturecredit không inventoryauth; 4/8integration đo khi mở |
| RUST-Q4 progression/rewards | Explicit nonredeemableformula/version, result/grantunique/ledger/outbox/backendtransactions. WikiEXP chỉ tham chiếu product | Actual175profile/resultfield→XP/currency/count/UI/relogin consumption; đo 0 và1delta/control/bounds, không portVN173/layout hoặc coi bảngwiki làfieldmapping |
| RUST-Q5 authority/freshness | Giữ fuse/drop/reportguard/fence policy đã đo; backendreward pending khi outcome chưa đủ tin cậy | CP242 exactoldUDP/currentpositive đã đo trên Rust; còn oldTCP/membership/IDreuse và phạm vi ngoài keywindow đã kiểm; full142distance/auth/low8wrap nếu validator/deployment cần. Không hứa server lọc mọiP2Ptraffic/fullworldauthority |
| RUST-Q6 operation/capacity | SQLiteWAL/backup-restore/adminaudit/config/metrics/queue limits design không cần mọi gameplayRE | [CP231](PROTOCOL.md#rust-foundation) foundation/14TCP/SIGTERM/rebind và [CP235](PROTOCOL.md#rust-m2-acceptance) nghiệm thu M2 scoped/healthyBsessionEOF; guardprofile trướcbind khácnativewrongversionUI. CP241activehostEOF đã có nativeRoom, loading/selectedEndingEOF và fullrecovery giữ giới hạn riêng; shutdownTCPtimeoutchưarõrootcause/CP237táihiệnkernel/PIDgonepostinspection/repeat22casesđạt, targetload/SLO/generalbackpressure/DBfault. CP236SQLite realbusy/FULL/audit/entropy/clock/schemafault có backendtests; CP243boundedworker/rawSQLite-threadfaults đã có; OSdiskfull/worker-nativeintegration/nativeDBfault/restore/SLO chưa đủ. CP240 đã có native life17/19/170 và hai175/180/reset/Start3; M3scopedacceptedCP242; loggerR4maskederror được sửa supervision, kernelR2flush timeout event141 chưa rõrootcause/repeat51TCP+nativeR5pass; không tăngdeadline. CP243worker/handoff đã có; CP244idleTTL/rate đã có; tiếp M4 và M5bridge riêng trong PLAN; opaque123/10 chỉobservepolicy/semanticsunknown, 167/body4-16-32 cũngboundedopaque/noresponseunknown;CP239142currentclearance/fullroster158mới mởđúnghidden27/28;old142saucurrentclear vẫnkhôngwireepoch. REchỉnếucầnresponse/field. REonlyportdiscrepancy. Accountresume/matchrestore khác freshlogin/connection; workload/RPO/RTO riêng  CP245fourstartup đã đo Manner1/2v2/4ACKbarrier/12remoteprogress; không lặp positive. CP246fourbom đã có; CP247giải gap12717/19→3peer170/native56→1/all4states; clear21fanoutđãkiểmTCP/nativeclear21chưađo. Không đo lại startup/bom/life17-19positive để tăngCP. CP2508baseline69tests/16eightTCP/native56mesh-8ownerbom-7peerlife/opposite4v4-reset-Start3 đã có; R1Dcollateraldead giữđúngproof. NativeR3H/R4Gpost17EOF causalunknown/R5priorargvpositive không SDKfix; UDPcapturepartial3Aframes/fourIPthreadguard,56nativeprogress/fullTCP độc lập; sửa capture trongmeaningfulteamvariantcontrol/no repeat2cycles chỉcapture; chỉ mở lại khi chặnvariant vớiobservablemới. 142/16722 chỉ boundednoeffect, không reveal/lifeacceptance; CP247clear21đãđợiđủrecipient/currentvisit; CP248outcome đã đợi allrecipientlife delivery/currentwitness/drain/latestdeath-blast/settle8 và có nativefull175180/2outcomes-Start3; CP249scopedguestD-hostA EOF/unranked180-post585-107IDclear/threeRoom/chief đã có; CP250known8baseline đã có; nextfixedFree1v7/selector/Mannernegative/native9admissionbridge, không twoforfeit/fullM4. Aảnh lúcrank chưa thấypanel, nativeAparser-rank-reset cóproof; đo Aảnh muộn trong plannedlifecycle nếu cần, không repeat2cycles. KernelR1shutdowndeadline/PIDgone vẫnchưarõcause. CP249normalR2/R3logflush292/208 tái hiện/R2threadreserve_bytes.cold; nguyênnhâncausalchưarõ. Explicittmpfs32four/51two/haiEOFnative/exporthashpass không là Btrfsfix/durablelog; không tăngdeadline hoặc rerunvôhạn, đo causalI/O tạiM8 khi cótrigger. Phép port/đo tối thiểu: CP201fourowner/capacity-power-fuse →170đủ3recipient/currentclear21 →CP248rank4/opposite2v2/reset/Start3đãcó →CP249abort/EOFportđãcó→CP250known8baselineđãcó→fixedteamvariants/native9admissionbridge. Publicexporter chỉkernel-tested; nativeprivatepreparer/R3retainedinputs khácversion đúngprovenance. BtrfsD-state lúccleanup twoTCPR1 không chứng minh nguyên nhânEOF; nếu tái hiện giữPID/thread/wchan, không tăngdeadline.|
| RUST-Q7 productpolicy còn mở | Chọn baseline mộtchannel/map/basic, diagnosticreward0 rồi internalnonredeemable, onegamesession/refusesecond, ephemeralrounds; khôngpayment/HA banđầu | Công thức XP/reward thật, starterinventory/currency/spend/expiry/refund/trust/abuse thresholds phải được product chốt trong issue trước M6expose; backend có thể dùng versioneddisabled/default0, không làm stall codec. Client có thể biểu đạt phần nào vẫn cần consumerproof |

Design nội bộ (hashing/store/transaction/scheduler/limits/adminAPI) không là khẳng định KR gốc. Wire/IDs/offsets,
authhandoff/actionconditions/effect/reset phải có build/locator/nativeproof trước expose. Không cần đợi fullRE
cho backend schema/tests hoặc port codec. Internet thật chưa kiểm, backlog riêng không dependency K6c/K7.
Đợt planning-only đã giao PR286/CP213; [owner09/11](evidence/owner-directives/rust-implementation.json) đã mở implementation Rust. M0→M1 không đợi fullauth/generalroom; chỉ mở RE khi implementation/native acceptance bị chặn. Thứ tự hiện hành chỉ ở [PLAN](PLAN.md#current-order).

<a id="feature-gaps"></a>
## Khoảng trống theo cơ chế

Đây là câu hỏi/phép đo tối thiểu, **không là thứ tự công việc**; thứ tự duy nhất ở [PLAN](PLAN.md#current-order).
Phạm vi đã biết/giới hạn nằm ở contract được catalog dẫn; không đọc `unknown` thành vắng mặt.
Bảng được [coverage](COVERAGE.md) trích tự động để tra theo chức năng, không sửa bản sinh.

| ID | Khoảng trống hiện hành | Phép kiểm tối thiểu |
|---|---|---|
| <a id="gap-process-patch"></a>KR-B001 | [CP179](PROTOCOL.md#bootstrap-recipe-scope): K1 đạt riêng offlineR2 từ3coldstart/API đúngPID/contract-controls; K0CP178 giữ. Không originalWindowsbootstrap hoặc3e9coldstart. | Cold3R2 đã giải, không đo lại. WiderWCP/launcher/config còn ở #2; currente9input/state bridge và livecodec theo PLAN/#3, chỉ mở bootstrapcause nếu có trigger mới. |
| <a id="gap-wcp-config"></a>KR-B002 | Fixture tổng hợp đã qua guard; oneclickuser/+E9C và dữ liệu/cache thật chưa đủ. | Chỉ mở consumer/config đang chặn runtime; so nhánh với và không tham số trên cùng copy, không đo lại +E78 đã giải. |
| <a id="gap-socket-codec"></a>KR-B003 | [CP181](PROTOCOL.md#login-retry-codec) actualsameprocessretry/reset/authcursor + CP180stream/CRC + CP057boundednativeoracle đủ K2profile theo PLAN. Fullgrammar/prefixguardon/modes/headerauth/capenforcement/FFFFFFFF giữ unknown. | Codec backlog theo trigger K3; không lặp solvedretry/closeonly/HWwatchpoint/76modelsplit. Accountresume thuộc identity/lifecycle, không tự thêm gate. |
| <a id="gap-auth-identity"></a>KR-L001 | [CP073](PROTOCOL.md#launcher-error-attribution): auxiliarylocale1,0 valid/init0→2000:0/noTCP; main currentA/Bactual67 positive. HistoricalmainSIGABRT/HTTPstartupcause vẫn unknown, không gánpopup là root. [CP217](PROTOCOL.md#auth-admission-prototype) valid/invalid32token cóproof; [CP223](PROTOCOL.md#auth-session-lifecycle) thêm endpoint-accountremap; [CP225](PROTOCOL.md#auth-fresh-process) freshprocesspositive/badtokennegative đã đo, sameprocessfatal/healthyEOFbefore17/fullidentityauthority vẫn mở. | Theo[PLAN](PLAN.md#current-order) tiếp RustM2haiidentity/phòng với bindings rõ; không mở lại authRE đã blocked hoặc tìm lại auth137/135/string/cache. Trace mainfail chỉ khi thực sự chặn. Providerinit sâu chỉ mở khi thực sự chặn main, chưa phải trọng tâm chơi. |
| <a id="gap-redirect-lobby"></a>KR-L002 | [CP186 K3local](PLAN.md#k3-acceptance) đạt; A291/nameguard/coldroute/versionnegative có nguồn. CP187actualAB291/currentroomintegration đủ K4scope; [CP200](PROTOCOL.md#four-session-native) bổ sung C/D291/name/ownslot. Caption/multichannel giữunknown. | [PLAN](PLAN.md#current-order): CP202–204 đã2v2outcomes/reset/bundle/bounds; tiếp lifecycle theoPLAN, không lặp ABCD291/room/start/NATtuple. Multichannel chỉ theo trigger. |
| <a id="gap-profile-cache"></a>KR-L003 | [CP186](PROTOCOL.md#channel-cold-route) cold3cache/warm có nguồn, K3local đạt. Năm retainedSSI giống clientnguồn; stageCRC0, mọicache/corrupt/reconnect/profile rộng cònunknown. | Reuse [scope/CRC](PROTOCOL.md#dynamic-cache-representation) và [coldroute](PROTOCOL.md#channel-cold-route); chỉ mở cachevariant khác khi K4/integration có trigger. Không lặp3file/CRC hoặc gọi coldroute là toànasset. |
| <a id="gap-create-join"></a>KR-R001 | [CP229](PROTOCOL.md#room-membership-registry) private nativejoin/move, [CP230](PROTOCOL.md#room-shared-cleanup) sharedcleanup có107/IDclear/promotionUI. | [PLAN](PLAN.md#current-order): trích policy thành module reference/owner-room-visit invariants, chọn seam trướcproductionintegration; không thêmRE consumer đãgiải. Globalround/jobs/host vẫn single-room; dynamiclistoccupancy/quota/restart/password/full/mode theo trigger |
| <a id="gap-roster-ready-host"></a>KR-R002 | [CP163](PROTOCOL.md#battle-eight-roster) actual selector1/8synthetic slots:4–4 check0/104,1–7 check7/no104, Ready/parser/map/capacity/getter có nguồn. CP162 actual Free/Manner/ACK/reset đã nối. | Không lặp guard/roster/map/input RE. CP165 actualpreset2cycles/Start3 đãđo; CP166 currentmodepromotion/rejoin đãtới clock; CP167đãhai cycleBhost; CP168BStart3/AUD050đãđo; CP171/172basiccycles/Start3 đã đo; tiếp PLAN theo gap thật; [CP208](PROTOCOL.md#eight-session-preview) có native8consumer trên2actual và adapter32TCP; tám actualclock/mesh56/stat8/ownerbom đã kiểm CP208209; CP210 có report-life/rank/reset/two4v4cycles8; [CP213](PROTOCOL.md#eight-team-variants) actualFree1–7/Manner4–4twooutcomes đã có; native8Manner1–7negative copy4b partialdoemptyJSONstartup; alternatecopy8d đãreturn7/no104/1543checks, emptyguardkhônghit nên khôngcause/fixproof. Còn interactiveC2Steamchange/generalroom-full/admission/Random/generalpromotion/row91 metadata. Theo PLAN; không coi sáu actor tổng hợp là sáu client. |
| <a id="gap-map-spawn-loading"></a>KR-R003 | [CP153](PROTOCOL.md#loading-hot-gate-contest): B32390 gate riêng khi load/restoreclock/0loadinghits; hai actual99/barrier5,94s/4,60s. CP151R2 historicalcause không bị tự sửa. | Scope loading recipe đã giải; không lặp writer/loader RE hoặc tăng30s. Generalcoldload/membership/timeout variants chỉ mở theo trigger; theo thứ tự hiện hành trong PLAN. |
| <a id="gap-room-membership"></a>KR-R004 | [CP230](PROTOCOL.md#room-shared-cleanup): guest86/EOF→A107ID2→0/base6-rewind0-UI1 vàhost86→B107ID1→0/110full6/UIchief; private registry/native2actual/565positive+166failure raw. Hostpostgetter unobservedbudget20, otherroomC exclusion9policycases only. | [PLAN](PLAN.md#current-order): reusable policy/reference and typed integration seam, recipient-specific sendfailure/partialsend/owner-visit fences beforeproduction. Native only if newimplementation/discrepancy; không lặp solved107/110/UI hoặc fullmembership/accountresume claim |
| <a id="gap-round-clock"></a>KR-F001 | CP155 baepoch actual99/nativeclock/barrier trước30s, B32390restore mỗi clock; same-run freshness/contest/EOF cónguồn. | Không RE lại99/load/clock; tiếp thứ tự hiện hành trong PLAN. Originalclock/timeoutauthority ngoài profile giữunknown. |
| <a id="gap-peer-movement"></a>KR-F002 | [CP176](PROTOCOL.md#a-c-basic-cycles) current A/C4nativewindows/2keys/target/currentprogress cùng2normalcycles; CP173 giữ AB scope. | FullUDP/auth/reliability/generalphysics/NAT discovery còn unknown; [CP194–195](PROTOCOL.md#udp-netem-cycles) đã đo current AB trên NAT tĩnh/UDP impairment. [CP200](PROTOCOL.md#four-session-native) thêm4actualcurrentmovement12chiều; [CP209](PROTOCOL.md#eight-mesh-owner-bombs) thêm8actual/56remoteprogress, gameplay4 theo PLAN, không lặp consumer/AC peer/solvedcodec/K6b đã đạt. |
| <a id="gap-udp-freshness"></a>KR-F003 | [CP155](PROTOCOL.md#direct-oldudp-contest-eof) exactR1Bframe→R2actualendpoint/queue/key59≠56/drop/noapply+11currentpositive vàfreshBRight trong cùng2contest/Ready/EOFrecipe. | Scope1key/currentIDs đã giải, không RE lại CP145consumer. low8wrap/forgedcurrentkey/fullgrammar/auth cònunknown theo trigger; tiếp thứ tự hiện hành trong PLAN. |
| <a id="gap-bomb-lifetime"></a>KR-F004 | [CP174](PROTOCOL.md#basic-host-b-eof) basicB/A/A IDs1..6/native195/reset rồithirdBID7pending/ownedBEOF/round_closed/no158; CP155scope cũ giữ. | [CP201](PROTOCOL.md#four-owner-bombs) có4owner/currentcapacity/159158broadcast/fencecontrols; [CP202](PROTOCOL.md#four-two-team-cycles) đã có2v2outcome/reset; [CP203](PROTOCOL.md#four-portable-guards) đã guarddeadowner118; transport/hardening tiếp theoPLAN. Genericpool/auth/races/low8wrap cònunknown, khônglặpknownfence. |
| <a id="gap-blast-block-chain"></a>KR-F005 | [CP172](PROTOCOL.md#hidden-cell-admission-current) basic Box/reveal27/28/142guard/3effects/reset/Start3 đã kiểm. | Generalmoving/order/allmaps/fullworldauthority unknown; không RE lại consumer. CP173 Bowner/native195 đã ghép; CP174BhostEOF đã kiểm; admission theoPLAN. |
| <a id="gap-floor-pickup"></a>KR-F006 | [CP172](PROTOCOL.md#hidden-cell-admission-current) terrain currentclearedcell/generation/commit/roster guard và native basic2cycles/reset/Start3 có nguồn; no-terrain giữ weak report scope. | Full142key/auth/distance/freshness/fairness/two-nativecontenders unknown. Same old142 sau legitimatecurrentclear chưa phân biệt; không gọi policy guard là original authority. CP173B118/bidirectional đã kiểm; CP174BhostEOF đã kiểm; admission theoPLAN. |
| <a id="gap-item-stats"></a>KR-F007 | [CP087](PROTOCOL.md#capacity-reservations) profileAinitial1max10/B2max7, acceptedBomb tăngtracked/native1→2 và resetthật hai vòng. Runtimeatmax/character/mode/pxs/TTL/fullreset khôngđủ. | [CP201](PROTOCOL.md#four-owner-bombs) đã đo C/Dstat/maxima trêncurrent4clock, không ápABdefaultquaD. Servertrackingdesign/nativepoollaw/runtimeatmax/othercharacters vẫnunknown; mởtheotrigger. |
| <a id="gap-needle"></a>KR-F008 | CP113opt-inonecredit/pair/peerescape; [CP114](PROTOCOL.md#needle-round-reset) currentR1secondtrapcạn/no1,0/actual19→resultreturn/Start2fresh145credit/escape. R2active, không2results. | Multi-charge/inventoryauth/generalresetvariants cònmởtheotrigger; khônglặpcurrentonecredit/depleted/reset. Rescue18độc lậpởcontactgap/PLAN. |
| <a id="gap-contact-state"></a>KR-F009 | [CP158](PROTOCOL.md#current-trap-udp-counter): native smallgap4 drop/largegap17 accept/controlledremote56→0→56→1; two observational runs không tái hiện CP157. | Exact naturalCP157state0packet/apply vẫn candidate/unknown; controlledcapability không retrospectivecause. Không lặp counter/body RE; nếu trigger cần, đo actual non-replayed packet/queue trướctrap. Hiện theo PLAN production baseline, không tự sửa client/claimbugfixed. |
| <a id="gap-ability-debuff"></a>KR-F010 | Shoe/Punch/debuff/TTL chỉ có bảng và một phần selector; không chuyển kết quả MaxPower sang mọi effect. | Chọn đúng ability cần cho Battle rồi trace1191840 selector/virtual tương ứng; đo consume/effect/peer/reset một case và control. |
| <a id="gap-mount"></a>KR-F011 | Group2 có nhánh riêng; điều kiện/effect/giới hạn/reset runtime KR chưa kiểm. | Chỉ khi fixture Battle cần mount: trace11928A2 tới consumer rồi một pickup/peer/reset; hiện loại khỏi fixture theo policy. |
| <a id="gap-active-other"></a>KR-F012 | Dart/Oxygen/Potion/Spring/Drill/RemoteBomb mới là phân loại, chưa chứng minh cùng đường kim. | Chọn một group đang cần, tìm consumer Ctrl/report/confirm; đo điều kiện, tiêu hao, hiệu ứng và peer, không quét từng row. |
| <a id="gap-timed-variants"></a>KR-F013 | Shield/Sensor/SJNeedle raw tail chưa chứng minh đơn vị/thời hạn/hành vi KR. | Trace consumer timer/group cần dùng, đo start/expiry/cancel/reset và peer với control; không suy ms từ tên hoặc raw số. |
| <a id="gap-mode-items"></a>KR-F014 | ItemAttr cho thấy row/group; presence cơ chế/mode guard/reward runtime vẫn candidate, ngoài Battle tối thiểu. | Chỉ mở group mode khi PLAN mở mode đó; bắt guard/consumer trước thử runtime, HPNeedle17 không là Needle145. |
| <a id="gap-peer-relay"></a>KR-F015 | [CP193](PROTOCOL.md#nat-endpoint-control) đã kiểm SNAT nhận thực tế/queue/drain và đối chứng153 riêng với mapped+DNAT/native helper/output/handler/current movement. | [CP194](PROTOCOL.md#nat-two-cycles) đã hai vòng NAT tĩnh/reset/Start3; [CP195](PROTOCOL.md#udp-netem-cycles) đã hai vòng dưới UDP delay/loss; [CP196](PROTOCOL.md#nat-netem-host-eof) bổ sung EOF/fence riêng; incomingadmission CP197198 đã đo; currentlifecycle/hardening theoPLAN. K6c WAN giả lập đạt CP212; [topology/giới hạn](PROTOCOL.md#emulated-wan-acceptance). NAT discovery/relay gốc/fullUDP ngoài scope; [Internet thật chưa kiểm](RESEARCH.md#gap-real-internet), backlog không chặn gate. |
| <a id="gap-result-return"></a>KR-X001 | [CP176](PROTOCOL.md#a-c-basic-cycles) Ahost/Cguest2normalresults/180/room/actualCReady2/native110; CP174thirdforfeit giữ riêng. | [CP198](PROTOCOL.md#native-interleaved-startup) đã ABnormal2cycles/reset/Start3; CP176khôngStart3 chỉ là phạm vi ACcũ. [CP210](PROTOCOL.md#eight-two-team-cycles) thêm8actual/2normal4v4/resetStart3 và [CP211](PROTOCOL.md#eight-portable-battle) packagednativeproof; authority/fullvariants vẫn mở, không ghép khác run thànhfull4inventory. |
| <a id="gap-round-reset"></a>KR-X002 | [CP176](PROTOCOL.md#a-c-basic-cycles) AC2keys/6entities/3Aeffects/195/stat/hiddeneligibility reset và Ready2 có nguồn. | ABStart3 đã có [CP198](PROTOCOL.md#native-interleaved-startup), ACscopeCP176giữ riêng; [CP202–204](PROTOCOL.md#four-transport-bounds) đãfourreset/Start2/3; wholeinventory/modes/low8wrap/generalreset còn thiếu, không lặp knownAB/ACcycles. |
| <a id="gap-disconnect-replacement"></a>KR-X003 | [CP177](PROTOCOL.md#replacement-profile) optional exported preset/capacity theo vacancy và actual C0/C1 initial/118/159158 đã kiểm; CP176 giữ AC2normalcycles riêng. | Maximum/exhaustion/general membership/account resume chưa biết; C0 startup SIGSEGV giữ nguyên, đối chứng pass không là rootcause. Theo PLAN acceptance/control, không lặp known C0/C1 entry hoặc ACcycles. |
| <a id="gap-diagnostic-service"></a>KR-S001 | [CP190](PROTOCOL.md#portable-probe-bundle) gói service/47fixture relocation đã actualKRWine/nativeidentity/room/load/EOFcontrol;  [CP177](PROTOCOL.md#replacement-profile) optional replacement preset/47pins/derived capacity/284server và7export tests; native C0/C1 actual118159158 đã đo. | Theo PLAN đối chiếu acceptance/dependency/control trước phép đo còn thiếu; nofullauth/physicalLANInternet8client/gate. Startup unknown không được suy thành capacity regression/fix. |
| <a id="gap-service-hardening"></a>KR-S002 | LAN giả lập đạtCP191; [CP193–196](PROTOCOL.md#nat-netem-host-eof) có NAT tĩnh/UDP delay-loss/two cycles/EOF. Interleaved admission sai6/8 đã có CP190. | [CP202203](PROTOCOL.md#four-two-team-cycles) đã4normal2v2/reset/bundle, [CP204](PROTOCOL.md#four-transport-bounds) córecipient-safe/bounds/rate; [CP205](PROTOCOL.md#four-member-abort-room) bổ sung native3survivorEOFcleanup; [CP206](PROTOCOL.md#four-health-identity) thêm health/identity/processrestart; [CP207](PROTOCOL.md#four-netem-restart) đã nativefreshrestart/fourUDPimpairment; [CP210211](PROTOCOL.md#eight-portable-battle) có8actualtwo4v4cycles/relocatedbundle; [K7 đạt scoped CP212](PLAN.md#k7-evidence-review); Internet thật chưa kiểm, không dependency. Teamvariants giữ mở theoPLAN. Theo PLAN, không tái mở NEXT-K7-FOUR-BATTLE-OUTCOME đã giải. Binding incoming là design riêng, không auth gốc. |


## Diễn biến lịch sử theo checkpoint

Các đoạn sau ghi trạng thái **tại checkpoint được nêu**, không phải bảng việc đang hoạt động.
Câu hỏi/kiểm tiếp đã được gom ở bảng phía trên; raw/checkpoint/evidence cũ giữ nguyên.

Cập nhật CP001: contract WCP ở [PROTOCOL](PROTOCOL.md#wcp),raw/review/integration ở
[evidence WCP](evidence/wcp/README.md). Kết luận “socket hai chưa nhận reply” chỉ áp cho
[ba run R2 silent](evidence/wine-r2/README.md),không còn là trạng thái mới nhất.
KRE-004/issue6 đã đóng trong scope Wine r1;KRE-005a/issue9 còn mở cho bước sau.

Q-023, CP006 (review theo [manifest](evidence/checkpoints/manifest.json)): `multiserveridx:0`
trong run copy đã kích hoạt setter `+E78`, tránh nhánh launcher và cho request đầu C→S;
`oneclickuser` liên hệ setter `+E9C` mới chỉ static, chưa đo runtime. Tám fixture S→C mode2/3
đưa guard trả1; hai copy thử NMCO lần lượt tới import init, status và `NMCOGetMyIdCode`.
Nguồn/giới hạn: [PROTOCOL](PROTOCOL.md#wcp), [evidence](evidence/wcp-sequence/README.md).

CP007: copy thay identity EDX:EAX bằng `0:1`, screen name rỗng, sửa heap guard và thử các import
cửa sổ; hai run debug dừng tại cùng thông báo Wine `mmdevapi` assertion. CP008: run plain với ALSA
null chạy audio và sau các import thử dừng tại slot `16EA638`. Khác biệt run chưa chứng minh debugger
gây assertion. Chi tiết/giới hạn tại [evidence](evidence/nmco-identity/README.md); review theo manifest.

CP009: thử tiếp message loop và thread import, client vẽ màn phân loại tuổi; run plain-thread
gặp lỗi ghi tại `45E920` tới địa chỉ `02677C9C`. Pointer/caller và nguyên nhân chưa xác định.
Chi tiết tại [evidence](evidence/nmco-identity/README.md).

CP010: guard của object chứa pointer cũ đã được thử reset; snapshot trực tiếp cho thấy pointer
được khởi tạo lại. Pre-throw snapshot xác định nhánh yêu cầu `CAWH.sys`; thử bỏ hàm cài hook và
các binding API tiếp theo đưa copy tới marker khởi tạo, rồi trap overlay ordinal9. Nguồn chuẩn
cho run/recipe/giới hạn: [evidence object state](evidence/object-state/README.md).

CP011: bỏ call overlay candidate ordinal9 trên copy cho UI login đầy đủ; hai phép click/type
(thường và giữ nút lâu hơn) chưa quan sát ô nhập thay đổi hoặc C→S mới. Callback `CAWH.sys`
trên host chỉ được đọc tĩnh, chưa xác nhận vai trò với hiện tượng input. Nguồn/giới hạn tại
[evidence UI](evidence/login-ui/README.md); cùng CP011 bổ sung provenance theo review CP010.

Áp dụng kiến thức VN trong CP011: GDB proxy bắt guard/IAT; run tắt audio trong prefix riêng
đo192 message chuột/phím tại DispatchMessage, nhưng chưa thấy nội dung ô nhập đổi hoặc C→S mới.
Nguồn và giới hạn tại [VN→KR](evidence/vn-reuse/README.md). Chưa chứng minh custom handler nhận
đúng control; GetAsyncKeyState vẫn là candidate KR, so VN chưa đủ để đổi thành GetKeyState.

CP012: đã nối mouse/key qua handler và click tới command1002. Hai tọa độ ID/password chưa
cho edit control; phím được chuyển tới button, không suy toàn bộ input hỏng. Đã tìm hostname login
riêng; run có ánh xạ local mở kết nối3 trước click, WCP poll/consumer trả1. Sau đó vào attempt
`11049C0` từ return`11013FE`, modal `CWaitForLoginDlg` trả1, chưa vào nhánh result3; attempt EAX0.
`modal-result` nối setter1 tới nhánh kiểm global1A966FC; bản dump có giá trị1. Reset riêng global
trong bộ nhớ run mới giữ được dialog đếm chờ, không thấy return trong cửa sổ đo.
Hai run reset thu276 byte mới=4 frame C→S15, nối guard→virtual→WinSock/capture; native parser
replay với seq40+3n khớp CRC cho bốn mẫu, negatives từ chối. Chưa xác nhận semantics auth.
Raw/recipe/timeline/giới hạn tại [evidence handler](evidence/ui-handler/README.md).

CP013 đối chiếu VN: static dispatcher KR đã nối S→C4/result0 và S→C15 tới các routine wait0→2→1,
rồi tick1→3; offset state KR+F4 khác VN+D8. Nguồn/call và giới hạn tại
[evidence VN→KR](evidence/vn-reuse/README.md). Chưa quan sát runtime hai handler hoặc đủ payload;
không dùng code VN `login_ok()` nguyên xi để đóng contract KR.

CP014 đã tìm trigger auth từ token local, capture request và thực thi handler phản hồi lỗi;
contract/chiều seq/prefix result0 tại [PROTOCOL](PROTOCOL.md#login-auth), control và các phép đo
bị loại ở [evidence login](evidence/login-runtime/README.md). CP013 đã merge PR21.
Nguồn count0 của auto-attempt đã giải; lifecycle kích hoạt stage tự động và global vẫn chưa đủ.

CP015 đã qua helper và nối flags62 → request5 → redirect/request6 → stage11/CLobbyStage;
S→C241 đóng notice, UI sảnh đã hiển thị trên copy. Contract/giới hạn ở
[PROTOCOL](PROTOCOL.md#channel-lobby), raw/control ở [evidence](evidence/postwait/README.md).
CP014 đã merge PR22. Không suy auth/profile đủ hoặc K3 từ UI này.

CP016 đã nối click → command2011 → guard91/bảng kênh14 → CCreateDlgRenew → capture81/107byte.
Contract và giới hạn ở [PROTOCOL](PROTOCOL.md#room-create), raw/control/correction tại
[evidence](evidence/room-input/README.md). CP015 đã merge PR23. Profile còn thiếu và chưa có phòng.

CP017 đã đo S→C86/result0 → strategy thật → CPrepareStage/stage12 và UI phòng chờ tám ô trống.
S→C92 bị region guard trong run, không phải đường success dùng cho cấu hình này. Contract duy nhất tại
[PROTOCOL](PROTOCOL.md#room-create), phép đo/correction tại [evidence](evidence/room-reply/README.md).
CP016 đã merge PR24; CP017 là thí nghiệm trên copy, tự kiểm/miễn review, tích hợp theo PR.

CP018 áp dụng VN đúng phạm vi: player/profile, own-ID match và unlock controls giúp chọn probe;
KR106/110/schema được phục hồi riêng. [Contract](PROTOCOL.md#room-player),
[raw/negative/correction](evidence/room-player/README.md). CP017 đã merge PR25.
Player `KRLocal` đã hiện; ID1 không match local list0, ID0 match own_slot1;110/pending0 cho nút Bắt đầu.
Hai import API candidate được bind thật trên copy mới. Full auth/identity vẫn thiếu, không coi ID0 là contract server.

CP019 đã nối writer96, config241/checksum, table238 và ack117; map capacity4 và filter mode0
đã qua bằng field+5D có control. Click thật tự gửi104/109byte, send1/pending3. Contract duy nhất ở
[PROTOCOL](PROTOCOL.md#room-map), [raw/correction](evidence/room-map/README.md).
CP018 đã merge PR26. Đây là mốc request start, chưa có loading/Battle hoặc gate mới.

CP020 nối start104 tới receiver KR121, slot validation và init CGameStage/stage13.
Auth prefix mở rộng có trường ID1, consumer tự cập nhật local-user list; player1 match own-slot.
ID0 workaround cũ không đủ để tham gia start. Contract/layout/correction tại
[PROTOCOL](PROTOCOL.md#start-reply), raw/native/control tại [evidence](evidence/start-reply/README.md).
CP019 đã merge PR27; CP020 có scene map/HUD/hai nhân vật trên diagnostic copy, chưa gameplay hoặc gate mới.

CP021 đã nối actual99 tới writer/init, đo197/196/mode0/198, clock/input và bom hiện sau118/159.
Nguồn chuẩn [PROTOCOL](PROTOCOL.md#round-go), raw/control tại [evidence](evidence/round-go/README.md).
CP020/PR28 đã merge. Đây là bước clock/di chuyển trên một client có guest tổng hợp, chưa vòng trận.

CP022 đã phân biệt calcE9C520 với nhánh158/kind0→E9E660; burst có nước/HELP và local actor
biến mất sau đó.180/full3 được đọc3/3/errors0 rồi stage12/UI phòng.
[Contract](PROTOCOL.md#bomb-cycle), [raw/correction](evidence/bomb-calc/README.md).
CP023 đã nối writer chungB53010 cho127/sub17/19 và giải full prefix14+blob8/5 bằng native;
175/292rawbyte đọc đủ và UI có LOSE/PERFECT. Stub chuyển từ fixed15s sang checksum-checked
reported19→rank chọn→180. Refresh110 host rồi guest-ready cho click thật phát104 lần hai;
run cuối có hai vòng scene/input/bom/HELP/kết quả/phòng trên cùng channel.
[Contract](PROTOCOL.md#round-result), [raw/control/correction](evidence/round-repeat/README.md).
CP023/PR31 đến CP026/PR34 đã merge. CP027 bổ sung hai client movement/bom runtime dưới đây; review độc lập waived, gates giữ mở.

CP024 đã echo owner/ba field từ actual118 sang159 cho bom đầu mỗi vòng; serial0 chưa phân biệt hai field.
CP025 có control serial28 bác mapping aux16, sửa serial u32 và ba bom liên tiếp0/28/56;
ID1..3, lặp/cũ hẹp và native getter được kiểm. [Contract](PROTOCOL.md#bomb-serial),
[evidence](evidence/bomb-serial/README.md).
Đã nối index spawn0/1→point từ map→actual actor.100/4039 có nguồn bảng Antarctic01;
Village00 và ExVillage01 có profile/map/spawn/geometry riêng và hai vòng thử với guest tổng hợp.
[Contract](PROTOCOL.md#bomb-position), [raw/control/correction](evidence/bomb-position/README.md).
Không dùng điều này thay nghiệm thu collision/reset/authority hoặc hai client.

CP030 đã có layout582/request124/confirm165, nhặt từ hai origin và một ca tranh cùng index;
nguồn chuẩn ở [PROTOCOL](PROTOCOL.md#item-pickup), [raw/control/hash](evidence/item-pickup/README.md).
Trap/death CP029 giữ [contract riêng](PROTOCOL.md#peer-events).
CP030/PR38 đã merge `1f70d4f`. CP031 đo hiệu ứng Bomb/Fire, duplicate sau xóa, native clamp và
sửa mapping power; [nguồn chuẩn/giới hạn](PROTOCOL.md#item-effects),
[raw/control/hash](evidence/item-effects/README.md). CP031/PR39 đã merge `4de4e2e`.
CP032 đo block/list2→reveal/124/165/stat và khởi tạo lại vòng; contract ở
[PROTOCOL](PROTOCOL.md#hidden-items), raw/control ở [evidence](evidence/hidden-items/README.md).
Phân biệt run chờ sai origin với run sửa policy; review độc lập waived.
CP032/PR40 đã merge `fe46beb`. CP033 đo chain hai bom liền kề/cách hai ô và replay sau chain;
[contract](PROTOCOL.md#bomb-chain), [raw/control](evidence/bomb-chain/README.md). Không nâng toàn gate.

CP033/PR41 đã merge `fbf4dc5`. CP034 đo leave/107/host110/guestEOF và giải
return loading bằng trường loại sảnh trong channel14; [contract duy nhất](PROTOCOL.md#room-leave),
[control/recipe/hash](evidence/room-leave/README.md). Không ghép các ca này thành full match.

CP034/PR42 đã merge `0779bc7`. CP035 đo B rejoin và A rejoin sau B promotion,
Ready A→Start B→121/99/clock hai phía; [contract](PROTOCOL.md#room-rejoin),
[evidence](evidence/room-rejoin/README.md). Chỉ mốc rejoin/start, chưa full match.

CP035/PR43 đã merge `1e78035`. CP036 đo enemy touch/no-touch, rescue18→170/18/state56→26
hai phía và movement tiếp; [contract](PROTOCOL.md#collision-rescue), [evidence](evidence/collision-rescue/README.md).
Ca rescue có hai client thật + C tổng hợp; control một đội/report21 không dùng làm full match.

CP036/[PR44](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/44) đã merge `7ab702f`.
CP037 thêm [kim tự thoát/peer](PROTOCOL.md#needle-selfescape) và
[cứu/diệt ba client thật + nhánh native](PROTOCOL.md#contact-precedence);
[recipe/control/hash](evidence/needle-contact/README.md). Không ghép các ca thành full match.

CP037/[PR45](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/45) đã merge `1e2ac9a`.
CP038 trả lời hẹp [trap lần hai khi hết kim](PROTOCOL.md#needle-repeat), có đối chứng hoán đổi team;
[raw/control/hash](evidence/needle-repeat/README.md). Kiểm exact-simultaneity runtime là backlog bổ sung,
không điều kiện chặn mốc trận đầy đủ; đây là quyết định ưu tiên của Codex, không tự xác nhận unknown.

CP038/[PR46](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/46) đã merge `1fb2c66`.
CP039 có [hai vòng statepeer/result/return, report21 suppression và hostEOFcontrol/policy](PROTOCOL.md#round-finish),
[recipe/hash](evidence/round-finish/README.md). Không còn coi các mốc hẹp này unknown;
semantics21/sub34, toàn reset/authority và disconnectvariants vẫn mở.

CP039/[PR47](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/47) đã merge `e19bc9e`.
CP040 đo [Start với client mới sau host EOF](PROTOCOL.md#abort-rejoin),
[recipe/control/hash](evidence/abort-rejoin/README.md). Đây là C identity mới3 vào slot0;
không coi là A reconnect/session resume. Giữ lượt collector lỗi và lượt bấm Start chưa tạo104 như lịch sử.

CP040/[PR48](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/48) đã merge `99d1835`.
CP041 có [stale158/ID reuse qua vòng](PROTOCOL.md#round-generation),
[controls/recipe/hash](evidence/round-generation/README.md). Không coi CRC/seq hợp lệ hoặc callback đầy đủ
là chống stale semantic. Fence cho job local là design server, không field generation phục hồi từ wire.

CP041/[PR49](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/49) đã merge `97320ca`.
CP042 có [old165/index reuse và stat trước124 mới](PROTOCOL.md#item-generation),
[control/hash](evidence/item-generation/README.md). Claim/ticket local là design; chưa giải được124 cũ
có toàn field khớp catalog mới. Không tìm lại receiver165 hoặc coi duplicate cùng vòng là proof qua vòng.

CP042/[PR50](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/50) đã merge `ed47c97`.
CP043 đo [159 cũ khi serial local trùng](PROTOCOL.md#late-bomb-confirm),
[recipe/control/hash](evidence/late-bomb-confirm/README.md). ID cấp tăng không đủ khi old159 khớp pending serial;
ca chèn gói cũ mất đồng bộ, không có result vòng2, phải giữ như outcome riêng.

CP043/[PR51](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/51) đã merge `f8569d8`.
CP044 trả lời hẹp [timer còn chờ khi host EOF](PROTOCOL.md#abort-timer),
[control/fence/vòng B/C mới](evidence/abort-timer/README.md). Job cũ bị loại ở deadline sau finish;
vòng mới có actual Bbom/trap/death/statepeer/result/return. Không phải A reconnect hoặc toàn reset.

CP044/[PR52](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/52) đã merge `b947a9d`.
CP045 bổ sung [nhóm item/nguồn cũ/Roller và MaxPower KR](PROTOCOL.md#item-families),
[recipe/hash](evidence/item-families/README.md). VN R169–185 đã có Rollerclamp/reset/movement;
không còn gọi nó unknown vì chỉ đọc oc_e-game cũ. [UDP cũ](PROTOCOL.md#udp-generation) đã tới
queue/drain và consumer game ở vòng2, slot1/gate0/return13; chưa chứng minh actorapply.
CP045 hoàn tất record/tự kiểm, review độc lập waived; [recipe/các control chưa đạt](evidence/udp-generation/README.md).
CP045/PR53 đã merge `6fe17b0`. CP046 đo tiếp [oldUDP tới actor](PROTOCOL.md#udp-generation):
native getter xác nhận x120→200 qua exactold callback, gói hiện tại sửa về120; no-replaycontrol
giữ x120/y159 ở8callback trước input, có hai result/return. Oldrun vòng2 có17 nhưng chưa19/result;
không quy toàn outcome chỉ do replay. CP046 hoàn tất record/tự kiểm, review độc lập waived.
E8E960 là false stub trong build này; không tiếp tìm input bật gate đã đo0.
CP046/PR54 đã merge `f85cda4`. CP047 theo target thật E66680, tái dùng cache CP025:
[byte khóa/tailfilter](PROTOCOL.md#udp-generation), [runtime key/endpoint](evidence/udp-freshness/README.md).
Đổi153key chặn oldmovement đã đo; endpointrotation riêng lẻ không chặn cả oldsource/newsource.
CP047 hoàn tất record/docs/tự kiểm, allocator unit-tested/pending runtime integration; review độc lập waived.
