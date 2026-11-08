# Kế hoạch server giả lập KR 00967

Nguồn chuẩn của project `boom-kr-server`; bản nhập từ plan đã được dsh1/dsh2 review tại `1eee46d`.
CP208: tám actual KR Wine đã qua auth/291/phòng/Ready/8ACK99/121285/153205/támactorclock, adapter32TCP có controls. Tiếp mesh movement và baseline stat E–H; audit AUD061 qua208/kế212. K6cInternet/K7/8fullmatch còn mở.


### Mục tiêu, baseline và phạm vi

Yêu cầu chủ dự án 26/10/2026: dùng kiến thức RE VN `VBNB_116` để dựng server giả lập cho
**chính `CA.exe` KR `00967`**. Đây là project độc lập `boom-kr-server`; không dùng scope, gate,
registry hoặc tỷ lệ tiến độ V1 của bản viết lại `boom-rs` làm điều kiện của repo này.

Revision kế hoạch **`KR-00967-EMU-1`**, trạng thái **K5 basic/K6a local và K6b LAN giả lập đã đạt trong scope; K6c Internet/K7 còn mở**.
Các mốc nguồn được phân biệt rõ:

- **Tài liệu tham chiếu lịch sử:** `boom-rs@dae5e1c717ad31b9e8d32a9b20a76d65942807eb`.
- **Plan KRE trước khi tách, đã review:** `boom-rs@1eee46d0ecf5c7f4eeedeadd7e258ef1cd64bf43`.
- **Snapshot khởi tạo repo mới:** `boom-kr-server@a19c8f68af68b01844c92a7870aa2a22eebf5f5f`;
  các sửa sau migration nằm trong lịch sử của chính repo này.
- **Nguồn RE:** `boom-re@9ac2075bbec0c16217741aaedea968cf685e39a9` và
  `boomkr-re@88e11ed30764c753a00f4f23a2f33dafadc4c890`.

Hash client và ma trận tái sử dụng:
[LEGACY_KR_PROTOCOL](PROTOCOL.md).

Đích đầu: bản copy client KR → bootstrap offline → đăng nhập identity cục bộ → một kênh/sảnh →
tạo/vào phòng → ready/start → Battle thường hai client → kết quả → về phòng → trận thứ hai.
Một map thật của KR, ưu tiên Village00 sau khi xác nhận loader/mode cho phép; nhân vật cơ bản theo
ID/Attr KR. Không áp index Village01, roster, drop override hoặc policy ready của V1 Rust lên client gốc.
Mở rộng bốn client chỉ sau khi hai client ổn. Shop, economy, guild, mode phụ và kết nối Nexon thật
để sau. Không dùng client Rust thay bằng chứng nghiệm thu `CA.exe`.

**Quyết định kỹ thuật:** ưu tiên chứng minh tương thích bằng service nhỏ và state machine theo
client; chưa ghép `boom-core` vào đường trận. VN có P2P và client tự tính một phần gameplay;
KR đã có consumer nổ/trap ở client nhưng chưa có bản đồ authority đầy đủ. Một server mới chạy được
với client Rust không chứng minh có thể điều khiển client KR bằng snapshot/command của ADR 0009.

### Tiến độ hiện hành theo bằng chứng

| Task/mốc | Trạng thái theo nguồn dẫn, cập nhậtCP208 | Nguồn chuẩn |
|---|---|---|
| KRE-004/issue6 | Đã review/merge PR7 và đóng issue:recipe r1 đến local accept;không phải full K1 | [Wine r1](evidence/wine/README.md) |
| Bootstrap reply | PR8 đã merge;debugger ProcessPatchreturn1,không ép success | [bootstrap evidence](evidence/bootstrap-wire/README.md) |
| KRE-005a/issue9,R2 | PR10 đã merge:3 cold start cửa sổ/second accept;issue9 còn mở | [R2 evidence](evidence/wine-r2/README.md) |
| WCP hai-reply | Run CP001 là lịch sử chưa C→S; CP006 đã đo request đầu trên copy tham số hóa | [PROTOCOL](PROTOCOL.md#wcp),[WCP evidence](evidence/wcp/README.md) |
| KRE-005b/issue14 | CP202–207four2v2/bundle/bounds/EOF/health/freshserver/UDPimpairment đã merge; CP208adapter8/32TCP/támactualnativeclock có runtime. K6cInternet/K7/fullauth/authority/phần ngoài scope giữ mở; task ở current-order | [Q-023](RESEARCH.md#q-023), [service](evidence/probe-service/README.md) |
| K0 | Đã nghiệm thu identity/harness tại CP178: originalWindowsVM fullfixedcopy/PID/token/cwd/window/ETW/isolation/cleanup; không bootstrap/gameplay success | [Acceptance](#k0-k2a-acceptance) |
| K1 | Đạt CP179 cho recipe offline R2: 3 coldstart/API đúng PID/contract/patch controls; không e9cold3 hoặc originalWindows bootstrap | [Acceptance](#k1-acceptance) |
| K2a/K2 | K2a đạt CP178; K2 đạt profileoffline CP181 với native sameprocessretry/reset/cursor/stream và boundednativeoracles. Không toàncodec/accountauthority | [Acceptance](#k2-acceptance),[contract](PROTOCOL.md#login-retry-codec) |
| K3 | Đạt CP186 trong scope A/local/Wine/một kênh normal; cold3cache/version/identity/stage và nguồn CP015/016/034/058/181–185 đã đối chiếu. Không fullauth/Windows/multichannel-caption hoặc hai-client291 | [Nghiệm thu](#k3-acceptance), [contract](PROTOCOL.md#k3-version-identity-controls) |
| K4 | CP187 đạt riêng phòng/load/start local currentAB291/copyABIcorrected; oneoutcome/reset/Start2, không gameplay2outcomes | [Nghiệm thu](#k4-acceptance) |
| K5 | Basic local đạt CP188; current2cycles/peer/Bowner/EOFcontrol riêng, designauthority được rà. Không fullinventory/mode/map | [Nghiệm thu](#k5-acceptance) |
| K6a | Local hai actualAB/basic/cùnghost đạt CP189; nativecontestedindex/reset/life và CP188peer/Bowner/EOF cóscope riêng. Không physicalLAN/Internet | [Nghiệm thu](#k6a-acceptance) |
| K6b | Đạt CP191 cho LANgiảlập hai netns/IP/NIC/actualKR theo owner; physicaltwohosts không required | [Nghiệm thu](#k6b-acceptance) |
| K6c/K7 | CP195 actualKR hai vòng/reset/Start3 qua NAT+UDP delay/loss đã đo; chưa NAT discovery/relay gốc/Internetgate; CP205–207 guest-hostEOF/health/identity/freshserver/fourUDPimpairment có nguồn; K7 giữ dependencyK6cInternet mở, xem k7-evidence-review. K6a vàK6bgiảlập giữscopeđạt theoowner | [Current-order](#current-order), [authority](PROTOCOL.md#authority), [gap](RESEARCH.md#gap-lobby-identity-followup) |

<a id="current-order"></a>
### Thứ tự công việc hiện hành

CP048 tổ chức catalog/service fixture, CP049 bổ sung guard124 trong scope hẹp;
[KRE-011/issue56](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/56) giữ acceptance của refactor.
KRE-005b/issue14 vẫn mở. Các bước dưới là **thứ tự ưu tiên**, không tự thay dependency/nghiệm thu K0–K7.

**Yêu cầu chủ dự án01/11:** [làm liên tục qua các checkpoint](evidence/owner-directives/continuous-work.json);
xong checkpoint/commit/PR thì chuyển ngay việc kế tiếp trong thứ tự này, không tự dừng/chờ nhắc.
Một blocker chỉ kết thúc nhánh đo tương ứng; ghi rõ rồi tiếp việc độc lập có thể làm, không tự đóng gate.

KRE-073/[#188](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/188) đã giao
[PR189](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/189)/merge`dae23b7`, CP118 vàđóng;
selectiveCLI/historicalmigration xong, không còn taskactive tổ chức. CP119 đãtiếpactualRE sau merge.
Yêu cầu kiểm8 người có kết quả hẹp [CP147/163](PROTOCOL.md#battle-eight-roster), không thay gate:
Free0/Manner1 input/ACK/reset đã nối CP162; selector1 actual4–4/1–7 đã đo. Không lặp roster/dump/guard. Service8 peer
và trận8 client thật là phần mở rộng sau hai/bốn client ổn ở bước5, chưa được đánh dấu hoàn thành.
Nguồn luật mới [wiki KR được chủ dự án tin cậy](REFERENCES.md#owner-trusted-namu-wiki) dùng trước
khi hỏi lại ý nghĩa stat/item/mode; chỉ đo phần khác biệt KR00967. Nguồn này phục vụ gate đang mở,
không tạo nhánh kiểm kê wiki hoặc mở mode phụ trước K3/K4/K5/K6a.

1. **KRE-005b — bước mạng tiếp theo sau K6b LAN giả lập:**
   Theo [chỉ đạo mới](evidence/owner-directives/emulated-lan.json), hostgiảlập/netns được chấp nhận thay
   hai máy vật lý. `NEXT-K6B-LAN-PREFLIGHT` khép CP191 theo [acceptance](#k6b-acceptance): actualKR/
   distinctNIC-IP-netns/directTCPUDP/nativePCAP/2normalcycles/rejoin/EOFcontrol có nguồn; không yêu cầu
   cung cấp physicalendpoint mới. [Portablebundle](PROTOCOL.md#portable-probe-bundle) và [reset B](PROTOCOL.md#emulated-lan-acceptance)
   đã córuntimecontrol, không làm lại các ca đó.
   `NEXT-K6C-ROUTED-PREFLIGHT` khép phần2subnet/router CP192: transportforward0negative→1positive và
   actualKR2cycles/reset/Start3/PCAPnativeendpoint đã cónguồn. Khônglặp routing/hàngđợi/codec đãgiải.
   `NEXT-K6C-NAT-TUPLE` khép phần tuple/control ở CP193: [đối chứng native](PROTOCOL.md#nat-endpoint-control)
   đã đo nguồn sau SNAT, helper trả0 với153 riêng và khác0/output13/apply/current progress với153 mapped+DNAT.
   `NEXT-K6C-MAPPED-CYCLES` khép integration CP194: [hai vòng NAT tĩnh](PROTOCOL.md#nat-two-cycles)
   đã có normal outcomes/reset/Ready2/Start3 và native peer/capture từng epoch. Không lặp tuple/codec/queue.
   `NEXT-K6C-NETEM-CONTROL` khép ca UDP40±5ms/loss5% CP195: [transport controls và native hai vòng](PROTOCOL.md#udp-netem-cycles)
   có counter/capture/timing/key/state/result/reset, TCP baseline; không phải Internet gate.
   `NEXT-K6C-NAT-EOF` khép control hẹp CP196: [pending B EOF/A native cleanup/fence](PROTOCOL.md#nat-netem-host-eof),
   r1 probe partial và r2 positive giữ riêng; không account resume/general reconnect/Internet gate.
   `NEXT-K7-ADMISSION-CONTROL` khép scope CP197: [opt-in incoming IP](PROTOCOL.md#operator-incoming-binding),
   realTCP interleaving/unknown/exhaustion/default controls và native serial-start two-cycles trên bundle mới.
   `NEXT-K7-NATIVE-INTERLEAVE` khép lịch CP198: [actual A1,A2,B1,B2,A3,A4,B3,B4](PROTOCOL.md#native-interleaved-startup)
   đúng role/stage/identity/room và hai normalcycles/reset/Start3 trên samebundle/source/profile.
   `NEXT-K7-FOUR-PLAYER-SCOPE` khép phạm vi chọn CP199: [native four-slot consumers](PROTOCOL.md#four-slot-native-consumers)
   trên hai actualKR + hai actor tổng hợp đã có full121/153/fouractor/ABpeer. Reuse CP020/026/036/147/163,
   không lặp codec/guard. Chọn tối thiểu4session A/B/C/D/slots0–3/IDs1–4/team0/1 xen kẽ/mộtroom/
   ExVillage01/normalBattle; không auth gốc/mode/item expansion hoặc bốnclientgate từ synthetic.
   `NEXT-K7-FOUR-SESSION-CONTROL` khép scope phòng/load/input CP200: [entrypoint4riêng/native4](PROTOCOL.md#four-session-native),
   16realTCP/session/ID/name/seq/Ready/4ACK99barrier/3remote153 và bốn actualWine vàoBattle/currentmovement12chiều
   đã có evidence. Cslot2/ID3 đồng thời khác replacementCslot0; default hai người/bundle cũ giữ nguyên.
   `NEXT-K7-FOUR-BATTLE-OUTCOME` khép scope CP202: [cùng4actual2v2/1754rows/180/reset/Start3](PROTOCOL.md#four-two-team-cycles),
   17/19→3peer/native56→1, survivor21postdeath/blast/drain, hai đội thắng ở2normalcycles cónguồn. Khônglặp knownreport/rank/room/codec.
   `NEXT-K7-FOUR-INPUT-GUARDS` khép CP203: [post19owner118/negative0mutation/alive+newroundpositive](PROTOCOL.md#four-portable-guards),
   247realTCPcontrols/325servertests; không còn gapđặtbomaccepteddead củaCP202. Genericfullauth/stale vẫnngoàiphạmvi.
   `NEXT-K7-FOUR-PORTABLE` khép scope CP203: same4profile/preset47aliases/64hashpins/relocation/Python-I,
   actual4normal2v2haioutcomes/reset/Start3 trênbundleR2; R1knownCstartupcontentpartial giữriêng, khôngstartupfix.
   `NEXT-K7-FOUR-TRANSPORT-HARDENING` khép scope CP204: [recipient-safe/default2s/partialreportfence/IO-log-ratebudgets](PROTOCOL.md#four-transport-bounds),
   214actualTCPnonreader/247guard/335server/16bundle vàcurrentnative4hai2v2outcomes/reset/Start3. Khôngqueuednonblocking/lawservergốc.
   `NEXT-K7-FOUR-EOF-ROOM-CLEANUP` khép scope CP205: [guestD/hostA pendingEOF→three-native180/107IDclear/Room/Bchief](PROTOCOL.md#four-member-abort-room),
   1258+1286checks/twoownedruns/pastfuse+15s/no158175.222TCP/341server; không rankAB/normalwin/3memberStart/accountresume.
   `NEXT-K7-FOUR-HEALTH-IDENTITY` khép scope CP206: [localhealth/run-session-room-round/digests/identitypin/processrestart](PROTOCOL.md#four-health-identity),
   352server/240TCP/19twoactualprocesscontrols;67pinbundle/current4native2normaloutcomes/reset/Start3 vẫn qua2202+208checks/32getters.
   Processrestart không là nativeaccountresume; nativefreshserver/sameidentitystore vàfourUDPimpairment cònthiếu.
   `NEXT-K7-FOUR-NETEM-RESTART` khép scope CP207: [samebundle/store/freshserver/4UDPqdiscs/native4twooutcomes/resetStart3](PROTOCOL.md#four-netem-restart),
   118transportcontrols/24remoteprogress/2412PCAP/2200+208+2528checks/32getters; currentFourrunloop vẫn214kernelcontrols.
   [K7acceptancereview](#k7-evidence-review) giữcriteria, không tựđónggate. K6ccontrolledInternetendpoint chưa có
   trong môi trường/bằngchứng; Wine/LANgiảlập đượcownerchấpnhận nhưngkhônglàInternetwaiver.
   `NEXT-EIGHT-SESSION-SCOPE` khép scope hẹp CP208: [native8consumer trên2actual+6synthetic/adapter32TCP](PROTOCOL.md#eight-session-preview).
   Không lặp capacity/teamguard/codec hoặc coi 32TCP tổng hợp là8client. Policy bom/rank4 không bật cho8.
   `NEXT-EIGHT-NATIVE-CLOCK` khép scope CP208: [8actualKR/32TCP/own291-slot/8ACK99/285-205/8actorsclock](PROTOCOL.md#eight-session-preview),
   1468raw/nativejoins/192copiedgetters/9netns/870raw/cleanup0. R1preclientfail giữriêng, R2sourcefrozen/pass.
   `NEXT-EIGHT-MESH-STATS` hiện hành: reuse R2/knownlowobserver/current4statlocators; normalOSinput từng
   owner trong8cửasổ, đo56directedremoteprogress/currentkey/PCAP và baseline capacity/max/power/max/speed/max
   ởcả8view, rồi actual118 mỗiowner khi policy8 vẫn từchối confirm. Không clone cap B2 sang E–H.
   Sau controls này mới8owner159/158/report/rank8/haioutcomes/reset; reject9/4–4/1–7 theo bước5.
   Không mở quét mode/item khác; gói8 mới chỉ verify/help, chưa native relocationproof.
   Bốn-player item/terrain/kim/cứu chưa tíchhợp; giữgiớihạn, không coi hai-clientproof là4playerproof.
   Quyết định thứ tự: NAT/two-cycles/UDP impairment/EOF và incoming admission đã có proof; CP202đã có gameplay2v2haioutcomes/reset/Start3; CP203–207 hardening
   bốn client đã có bằngchứng theo scope; nhánh8 độc lập đang làm, chưa nghiệm thu K7. K6c vẫn cần endpoint Internet do project kiểm soát và acceptance giữ nguyên,
   không dùng mô phỏng để tự đóng gate hoặc thêm các biến thể NAT không có blocker.
   AUD061 periodic qua208; audit thủ công kế CP212, không reset counter.
   Wine thay Windows và LAN giả lập theo [directive](evidence/owner-directives/wine-accepted.json);
   [#14](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/14) tiếp tục mở.
   <!-- active-issue: 14 --> AUD061 qua208, audit kế **CP212**.

2. **KR-B003/KR-F006 — nợ có giới hạn:** [CP057](PROTOCOL.md#codec-stream-boundaries) đã đo stream/
   seq/bounds trong model, không mở lại76split hoặc cấp buffer lớn chỉ để vượt allocationguardFFFFFFFF.
   Full modes/prefix guard/live reset, distance/auth/transport/kim-cứu của reportguard giữ trong RESEARCH;
   chỉ mở đúng variant gây blocker integration. Không lặp nguồn t/k/forgedbody hoặc mọi EOF.
3. **Battle tối thiểu theo trigger sau integration:** CP084 có ba bom fork, CP095 có actualbox control, CP096 có adjacentmixedowner; CP122–123 deep recursion vàCP131sharedblocker ca hẹp đã đo; genericmoving/order/allmaps còn thiếu. List2 chỉ gắn item vào FixedBox đã có, không tạo tường mới; ca vật cản phải có actualmap/tile/control. Runtime max/speed hoặc ability khi ca Battle dùng
   nhóm đó. Tái dùng [item families](PROTOCOL.md#item-families), không quét mọi row hoặc tìm lại kim/Roller.
4. **Lifecycle còn thiếu:** account reconnect/general membership/idleEOF/sendfailure/timeout/loadfail/
   reset variants chưa đủ; chỉ mở ca có trigger cụ thể khi chặn integration/acceptance, không kiểm
   kê hết mọi race trước validator/auth. C mới không là account resume; một vòng B/C không K6a.
5. **Nghiệm thu local rồi nhiều máy:** gom cùng-run consistency/malformed/stale/reset/disconnect theo
   K5/K6a và [authority](PROTOCOL.md#authority); sau đó K6b LAN, K6c Internet rồi K7 bốn client/hardening.
   Sau baseline hai/bốn client, mở rộng service roster/slot/session/Ready/barrier/peer tới8;
   kiểm8 client thật, reject người thứ9 theo map,4–4/1–7 và kiểu cân bằng sau khi input KR đã phục hồi,
   kết quả/reset hai vòng. Không coi CP147 đã nghiệm thu phần mở rộng này.
   Không ghép các run riêng thành một trận đã nghiệm thu. LAN giả lập đã đạt CP191; Internet/relay gốc còn unknown. Nhánh admission engineering K7 ở bước1 được làm độc lập; nghiệm thu K7 vẫn giữ dependency/scope trong gate.

[Bằng chứng service](evidence/probe-service/README.md) phân biệt run đủ hai chu kỳ với run partial.
Policy fuse/drop/rank/forfeit vẫn là emulator choice. K0 đã nghiệm thu theo đối chiếu thủ công CP178; K2a tĩnh đã đạt. K1 đạt recipe R2 tại CP179; K2 đạt profileoffline CP181; K3 local đạt CP186; K4 local đạt CP187; K5 basic local đạt CP188; K6a CP189/K6b giả lập CP191 đạt scope riêng, K6c/K7 còn mở; checker không tự đóng gate.
Chu kỳ audit/counter tại [manifest](evidence/checkpoints/manifest.json), không tạo plan theo phiên.

<a id="issue-order"></a>
### Issue hiện hành và backlog theo nguồn chuẩn

Bảng này chỉ phân công/thứtự; acceptance/PR/bằngchứng ởissue. [Catalog](COVERAGE.md) dẫn featureID→issue,
[reconciliation cóngày](evidence/issue-reconciliation/README.md) giữ lýdođóng/thaythế, khônglàplan thứhai.

| Thứ tự/scope | Issue | Trạng thái/phần còn thiếu |
|---|---|---|
| Hiện hành RE | [#14/KRE-005b](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/14) | K5 basic/K6a/K6b giả lập đã đạt trong scope. CP193–196 NAT/UDP delay-loss/EOF có nguồn; CP197incoming opt-in có controls; CP207native4freshrestart/UDPimpairment đã kiểm; K7/K6cInternet mở theo dependency; CP2088consumer/32TCP đã đo; támactualclock đã qua; tiếp NEXT-EIGHT-MESH-STATS độc lập, audit kếCP212. K6c/K7/fullauth/authority/general variants giữ mở. |
| Control theo trigger/gate | [#1/KRE-001](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/1), [#2/KRE-002](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/2) | CP178 K0 originalWindowscontrol/ownership đã đủ, #1 đã đóng qua PR250; #2 giữ widerWCP/launcherconfig; K1 R2 đã đạt CP179, không lặp cold3. |
| Codec backlog/identity hiện hành | [#3/KRE-003](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/3), [#9/KRE-005a](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/9) | Fullcodec/livebounds vàintermittentmainstartupcause cònthiếu; khônglặpprimitive/auxiliarylocale. |

### Tra theo chức năng

[Cây/coverage/dependency sinh từ catalog](COVERAGE.md) thay bản đồ lặp theo checkpoint tại đây.
Catalog chỉ dẫn về contract/câu hỏi/evidence; không giữ một thứ tự công việc khác.
Lịch sử từng checkpoint bất biến ở [manifest](evidence/checkpoints/manifest.json).

<a id="k7-evidence-review"></a>
### Đối chiếu K7 hardening/demo — CP207

Codex đọc **criteriaK7 giữ nguyên** trong bảnggate/dependencygraph, đối chiếu thủ công sources dưới;
[reviewwaived](evidence/checkpoints/review-waiver.json), [metadatareview/raw](evidence/peer-endpoints/README.md#cp207).
**K7 vẫn mở** vì dependencyK6cInternet chưa đạt/thiếu project-controlledexternalendpoint. Các hàng là proof theo scope,
không autoacceptance toàn sản phẩm hoặc suy full4mechanics/8client/auth; không dùng %.

| Criteria hiện hữu | Bằng chứng / giới hạn |
|---|---|
| Bốnclient/Linuxpackage/freshlaunch/hai-và-bốn táilập | CP190/191/198two, CP202–207four; CP206→207same67bundle/store/newserver/16freshTCP/twooutcomes/resetStart3. CurrentProbeServicebyteSHA bằngCP198 nên không lặp RE/two-core khôngđổi. Wine4b/virtualhosts/fixedprofile, không coldreliability/fullauth |
| Boundedparser/backpressure/rate/malformed-duplicate-stale | CP203247guard/CP204finiteIO-rate/default2s214TCP/CP206352tests240TCP; CP207current214kernelTCP. Finiteblocking/failclosed/partialreportfence, không fullgrammar/nonblockingqueue/publicloadbenchmark |
| Minimumidentitypersistence/log-health/build-run-session-room-round | CP206exactoperatoridentitypin/19processrestart/UUID/digests/context/hashpair; CP207samephysicalbundle/store/freshnative/208health. Không accountdatabase/resume/remoteattestation/powerlossdurability; pairfiles cần sequence/hash |
| Loss/jitter/serverrestart | CP195twoNATUDP, CP2074recipientUDPqdiscs/118transportcontrols và4actualtwooutcomes/24remoteprogress/freshserver/newUUID. UDP40±5ms/loss5%/TCPbaseline, không Internet/hardwarediversity/oldclientresume |
| Authority/clientcopylimits/dependency | [Authority](PROTOCOL.md#authority), [copy/scope](PROTOCOL.md#four-two-team-cycles), [health](PROTOCOL.md#four-health-identity), [netem](PROTOCOL.md#four-netem-restart). Design/report-derivedlife/rank/fuse/budgets côngkhai; full4item-terrain-kim-cứu ngoài proof. K6cInternet chưađạt, không đóngK7 |

### Bằng chứng lịch sử khi lập plan (26/10)

- Tài liệu KR cũ chưa chứng minh qua `ProcessPatch`. §31 của `boomkr-re/docs/kr.md` ghi một run CA
  còn sống (45 thread/~75 MB), snapshot `netstat` không có kết nối; dòng `EndOfInit! exitcode=1` là
  log run trước. Không gọi đây là trạng thái runtime hiện tại khi chưa chạy lại; snapshot không có
  kết nối cũng chưa loại trừ connect ngắn đã xảy ra. §33 tách dialog **Nexon của NewPatcher** và
  dialog **CA** có title `??? ??`; quan hệ nhân quả với `ProcessPatch` vẫn unknown. Trạng thái unpack
  khác nhau giữa run/dump; không dùng entropy `.themida` để kết luận toàn bộ code game chưa đọc được.
- REA mới đọc được `NewPatcher.exe`: consumer manifest `sub_4118E0` phân biệt `files.nfo`/`files.nfo2`,
  `.gz` và delta; có đường báo lỗi mạng/file. Đây là cạnh RE tĩnh mới cho bootstrap, **chưa** chứng minh
  nguyên nhân `ProcessPatch` lỗi, IPC với CA hoặc schema patch service đầy đủ. Xem
  [evidence bootstrap](evidence/bootstrap/README.md).
- VN đã có codec kiểm bằng hàm binary và server chạy 1–2 client. Dùng lại cách kiểm và kiến trúc
  thăm dò; mọi key/seq/opcode/schema KR cần chứng minh riêng. Không có “port VN sang KR bằng đổi version”.
- `kr.md` §12 tìm **bảng handler VM Themida**, không phải kiểm kê dispatcher packet KR. Không suy
  “không có dispatcher mạng” từ kết quả âm tính về VM. Message nội bộ 27 cũng không phải wire opcode.

<a id="basic-acceptance-review"></a>
<a id="k5-acceptance"></a>
### Nghiệm thu K5 basic local và phần thiếu K6a — CP188

Codex đối chiếu thủ công **tiêu chí K5 giữ nguyên**, [review waived](evidence/checkpoints/review-waiver.json);
[decision/nguồn/giới hạn](evidence/peer-endpoints/k5-acceptance.json), [contract](PROTOCOL.md#current-basic-cycles-291).
**K5 đạt trong scope basic A trên hai actualAB/copyWine4b781f…/privatevirtualhosts/Manner/ExVillage01,
optionalbasic+291/nameguard.** Không toàn inventory/mode/map/auth hoặc nguyên bảnWindows.

| Acceptance giữ nguyên / phần liên quan K6 | Đối chiếu bằng chứng và giới hạn |
|---|---|
| Clock thật, move, bom, phá block, item cơ bản, trap/dead, result/room | Hai currentrun độc lập đều hai normalcycles/reset/Ready2/Start3. Nativeactual118/159158/currentIDs/195flags/hidden142/124/165/baeffects mỗi key; strict56→1/other0 trước175 ở hai view. Không encoder tự sinh hoặc syntheticguest |
| ID bom ổn định, không doubleconfirm, không kẹt GO/load | Peer run IDs1..6/B,A,A; samepointer/matched159158/fullcursor, one165/view/index/epoch; three30sbarriers sau99both; native121/clock3. No allpools/timers/resetvariants claim |
| Authority từng hành vi thực dùng | [Matrix](PROTOCOL.md#authority) đã Codex rà creator/simulator/validator/broadcaster/peer-editable; native input/simulation/peer state tách service policy6sfuse/drop/rank/terrain/guards. Originalauth/distance/142freshness/arbitration unknown; design không đổi thành luật gốc |
| Trace/capture cùng run và dependencies | RunSHA/seals/inputfreeze/ownedcleanup/nativepayload/getter/capture liên kết từngrun; K0–K4 scoped giữ đạt. Import-onlycopy/.imageidentical đã control CP187 |
| K6a A↔B/hai owner/hai outcomes/reset | Currentpeer run có4nativewindows/2keys/currentprogress/B,A,A và2results/Start3; không physicalLAN hoặc fulltransportgrammar |
| K6a disconnect/host cleanup | CurrentEOF focusedfirstepoch riêng có pendingB/ownedkill/A107ID2clear/chief110/room và round_closed pastdue+4s. Forfeit của control không thay một normalending |
| K6a tranh item/freshness | CP155 controlledcontest+exactoldUDP và CP159160 immediateproduction giữ source riêng. **Tại CP188 currentbasic291 contest chưa đo**; [CP189 đã bổ sung](#k6a-acceptance), không ghép thành cùng-run. Tại CP188 K6a còn mở; CP189 nghiệm thu K6a hẹp, K6b/c/K7 vẫn mở |

CP187 R2partial và currentEOF r1partial không bị sửa thành pass. Neutralkeyrelease có positive nhưng
không chứng minh nguyên nhân oldnotstable; entrycolumn reuseCP175 có control riêng. Không đổi gatecriteria.

<a id="gates"></a>
### Dependency và gate

Từ CP192, [ownerWineacceptance](evidence/owner-directives/wine-accepted.json) cho dùngWine thayWindows
thật ởmọi nghiệmthu project; [LANgiảlập](evidence/owner-directives/emulated-lan.json) thayphysicaltwohosts
choK6b. Các yêu cầu WindowsVM/ETW/fixedNTFS trongđịnhnghĩa hoặc bảngđốichiếu cũ bên dưới giữprovenance
của controlđãđo, không là blocker đòi chạyWindowsmới. Mỗi run mới vẫn pinmôi trường/copy/patch/isolation/
ownedPID/capture/nativebehavior; không suy đổimôi trườngnghiệmthu thành proof vềAPI chưađo.

```mermaid
flowchart LR
  K0["K0 Identity + harness · đạt CP178"] --> K1["K1 Qua bootstrap · R2 đạt CP179"]
  K0 --> K2a["K2a RE codec tĩnh · đạt CP178"]
  K1 --> K2["K2 Hello + codec · profile đạt CP181"]
  K2a --> K2
  K2 --> K3["K3 Login + sảnh"]
  K3 --> K4["K4 Phòng + load map"]
  K4 --> K5["K5 Trận 1 client"]
  K5 --> K6["K6 2 client + 2 trận"]
  K6 --> K7["K7 4 client + đóng gói"]
```

| Mốc | Công việc và đầu ra | Gate bắt buộc trước khi mở phần phụ thuộc |
|---|---|---|
| **K0: identity/harness** | Manifest source/copy/dump/DB: SHA, image base, RVA, section hoặc memory-layout, patch và config; DB dùng copy/private workspace, không ghi DB nguồn/shared. Windows VM là control bootstrap chính: copy đầy đủ lên ổ local fixed, cwd cạnh exe/DLL, kiểm manifest `requireAdministrator` và effective token; không dùng share/UNC làm đường chạy control. Mỗi run ghi PID/parent/start-time/image SHA/command line, window owner/class/title, CALog mới, log/capture cùng run ID. | Khớp artifact KR/VN; không ghi vào `clients/`; không nhầm CA/NewPatcher/CAWH hoặc log run cũ. Không suy thiếu DLL chỉ từ dialog (inject từng gây cùng lỗi giả). Có lệnh tái tạo và cleanup chỉ PID thuộc run. |
| **K1: bootstrap** | Ba bước K1a ownership/control → K1b contract/caller → K1c stub/patch có đối chứng, theo định nghĩa dưới bảng. NewPatcher manifest là một giả thuyết để kiểm, không phải route đã chốt. | Ba cold start của cùng recipe đi xa hơn điểm chặn được định danh, tới **network attempt local/state chờ hello**, có event API/capture gắn đúng PID; CALog marker mới nếu có. Không dùng alive/không exit/không còn dòng ProcessPatch làm proof. Màn login thuộc K3; không đòi server trả hello trước K2. |
| **K2a: bước con RE tĩnh** | Tìm primitive reader/writer, framer, checksum/crypto/seq và mode/hello từ **KR**, dùng mỏ neo167/184/183 hoặc socket call graph. Fixture hàm KR thật (emulation nếu đủ context), kiểm cursor/endian/bounds, chiều/socket và reset; lưu phần chưa mở được. | Có oracle độc lập cho phần codec probe K2 cần; không lấy encoder tự sinh golden làm proof. Không đòi registry toàn game trước hello; nếu primitive cần thiết chưa có oracle thì K2 chưa mở, ghi checkpoint Codex. |
| **K2: codec/hello** | RE `send/recv` → buffer/frame → primitive reader/writer → checksum/crypto/seq → callback hello/version; lập profile KR riêng theo từng kết nối và chiều. Fixture dùng hàm KR thật hoặc quan sát KR; probe server chỉ có accept/hello/log. | Client nhận hello, gửi request kế tiếp; parse đủ byte, checksum/seq đúng qua nhiều frame và reconnect. Kiểm boundary length, chia/gộp TCP, đổi mode sau hello và reset state. Golden do encoder tự sinh chỉ là regression. |
| **K3: login/kênh/sảnh** | State machine bootstrap-data → identity local → auth result → redirect/chọn kênh → lobby. RE consumer cache/version và gói tối thiểu theo thứ tự; cấp ID ổn định giữa socket login/channel, ghi encoding chuỗi. Patch/game/redirect socket có codec state riêng theo lifecycle KR đã đo. | Cold cache, cache thiếu và warm cache đều có kết quả rõ; UI identity/kênh đúng, keepalive không timeout. Kiểm gửi đúng byte nhưng sai socket/stage bị từ chối có log; không gửi blob trước stage/cache-path init hoặc tái dùng seq của socket trước. Sai version/identity lỗi rõ. |
| **K4: phòng/load** | RE room list/create/join/slot/host/ready/start; map name/id/spawn/roster/seed/placement; loading barrier và điều kiện chuyển scene. Mỗi field có provenance và ghi vào `kr-emu-dev`; không trộn KR native với `emu_choice`/`design` hoặc áp V1 Rust implicit. Mỗi request có reply/broadcast theo stage. | Hai client thấy cùng phòng/slot/host; map đúng payload KR; spawn khác nhau trên ô hợp lệ; một người chưa load không làm người kia vào trận sai state. Kiểm host rời, join/leave và start lặp; reviewer kiểm source/design của các field cần để load. |
| **K5: trận một client** | Chạy clock thật; di chuyển, đặt/nổ bom, phá block, hiện/nhặt item cơ bản, trapped/dead, kết quả và về phòng. Hoàn thiện authority matrix theo contract §5; fuse/drop/timeout không khôi phục được là `design`, có profile riêng. | Chu kỳ trận hoàn chỉnh bằng client KR; cùng bom có ID ổn định, không xác nhận pickup lặp, không kẹt GO/loading. **Reviewer duyệt authority matrix** cho từng hành vi đang dùng: creator/simulator/validator/broadcast/peer-editable state; unknown lịch sử được ghi giới hạn và policy design tương thích có kiểm, không mặc định server-authoritative. Trace client và capture cùng run. |
| **K6: hai client** | RE peer-discovery, session key/slot, UDP packet/reliability/relay nếu có; chọn P2P local hoặc relay theo consumer thật. Đồng bộ hai chiều, bom/item/trap/result rồi reset round. | A di chuyển B thấy và ngược lại; A đặt bom B thấy; tranh item chỉ một kết quả; hai máy đồng thuận trapped/death/result. Hai trận liên tiếp không rò entity/timer/seq/peer. Disconnect/host rời không treo peer còn lại. Reviewer cập nhật authority matrix bằng ca peer thực; unknown làm thiếu consistency của demo thì chưa qua gate. |
| **K7: hardening/demo** | Mở bốn client, bounded parser, backpressure, rate limit, persistence identity tối thiểu, log/health, loss/jitter và server restart. Đóng gói server Linux cùng config/harness hướng dẫn. | Fresh launch theo README; hai và bốn client tái lập; malformed/duplicate/stale input không nhân kết quả hoặc làm crash server; log xác định được build/run/session/room/round. Báo giới hạn authority và sửa client cần thiết. |

<a id="k0-k2a-acceptance"></a>
### Nghiệm thu K0 và bước tĩnh K2a tại CP178

Codex đối chiếu thủ công từng acceptance ở bảng trên, [review waived](evidence/checkpoints/review-waiver.json);
decision/nguồn/giới hạn tại [evidence CP178](evidence/peer-endpoints/README.md#cp178). Không đổi tiêu chí,
không dùng checker hoặc số checkpoint để đóng gate. **K0 đạt về identity/harness; K2a đạt phần tĩnh
cần cho probe K2. Tại CP178 K1/fullK2 và K3–K7 còn mở.** K1 có nghiệm thu hẹp ở CP179; K2 profile đã đạt CP181 bên dưới, widercodec vẫn backlog. PR/Git giữ delivery status; PR250 đã merge tại `2f2b87f`.

| Acceptance giữ nguyên | Đối chiếu bằng chứng | Quyết định / giới hạn |
|---|---|---|
| K0 source/copy/dump/DB SHA/layout/private provenance | Current5KRartifactverify + VNoriginal/patchedreference hash; source187file tree, fullfixedC copies187/187/hash; old private analysis provenance giữ nguyên | Đạt identity; không suy image memory nguyên bản tương đương copy Wine/model |
| K0 fixedlocal/cwd/admin | CNTFS/DriveType3, CA RT_MANIFEST1 requireAdministrator, actualPEB32cwd/TokenQuery+kernelEvent1high/elevated | Đạt trong Windows10 VM; không bareWindows/gameplay claim |
| K0 PID/parent/start/image/cmd/window/newlog/capture | CA1328, NP9656, Both11136/7876; native+ETW exactstartticks/modulebase; modal đúng PID, freshCALog/PCAP/ETW | Đạt ownership; CA cómarkerProcessPatch/modal, exactPC/return chưa đo; title không là rootcause hoặc K1success |
| K0 isolation/recipe/cleanup | IPv4/6DROP/DNSproxy, owned local sentinelSYN trênbridge/khôngeth0/khôngaccept; inputfreeze/ownedPIDguards/exactfirewallrestore | Đạt harness; không file/registry/networkcontract đầy đủ hoặc máy pristine |
| K2a writer/reader/framer/CRC/crypto/seq/hello | KRnativehelpers/exhaustive, WCP0/0→2/3, CP006/014 actualnativeframe, CP057stream/bounds/resetmodel, CP058strings/cursor/errors0 | Đạt oracle cho probe; model prefixguardoff/cap/FFFFFFFF/fullmodes/livereset vẫn giới hạn K2 |
| Dependency | K0 đã đạt; K2a cho chạy song song K1 theo kế hoạch gốc | K2 vẫn đòi K1 và livecodec acceptance; không đóng toàn KRE-003/#3 |

AUD053 ad-hoc qua178 không reset periodic176/kế180. KRE-001/#1 đã đóng sau PR250 đúng scope identity/control;
KRE-002/#2 bootstrap, KRE-003/#3 fullcodec và #14 giữ mở. Thứ tự duy nhất ở current-order.

<a id="k1-acceptance"></a>
### Nghiệm thu K1 recipe offline R2 tại CP179

Codex tự đối chiếu **tiêu chí K1 giữ nguyên** ở bảng gate; [review waived](evidence/checkpoints/review-waiver.json).
**K1 đạt cho recipe `post-bootstrap-r2` SHA7b3b435… trên Wine đã pin**, dựa ba coldstart tuần tự
`defproc-1`, `final-cold-2`, `final-cold-3`. [Nguồn/recipe/hash](evidence/wine-r2/README.md#cp179),
[contract/giới hạn](PROTOCOL.md#bootstrap-recipe-scope). Record CP179 giữ snapshot trước PR/merge.

| Tiêu chí giữ nguyên | Bằng chứng đối chiếu / giới hạn |
|---|---|
| K0/control và contract/caller | K0 CP178/PR250; native R1 `1234875/EAX1`, equal-version và silent/badguard controls. Không ghép các control thành cold3 hoặc suy Windowsmodalcause. |
| Ba coldstart cùng recipe vượt blocker tới local network attempt/state chờ hello | Cùng SHA R2/harness/Wine/reply/hostmap/listeners; `init_peb` và native `connect` cùng PID01a0/01ac/0174; reply socket1 trước localattempt socket2. Tất cả milestone nằm trong10s đầu; thời gian quan sát15/10/10s giữ nguyên, không là thay client input. |
| Recipe/stub/patch/rollback | Game/prefix copy riêng từ template đã init; không ba Wine install mới. Builder/NGinit omission/imports/negativecontrols có nguồn; không ép ProcessPatchreturn. Harness cleanup riêng prefix/children, run end và isolation có raw. |
| Giới hạn nghiệm thu | Chỉ offline R2; không originalWindowsbootstrap, không3e9coldstarts hoặc coldallcache. R2/e9 khớp vùng bootstrap/framer, nhưng e9 thêm patch/state nên không suy whole-program tương đương. Socket2silent/zeroC2S/timeout không là login hoặc K2. |

Tại CP179 K2 còn mở; CP181 đã nghiệm thu profile K2. KRE-002/#2 widerWCP/config và #3 widercodec giữ backlog; K3–K7 mở.
Thứ tự công việc duy nhất ở current-order, không thay record complete hoặc tiêu chí để đóng gate.
Periodic audit kế CP180, AUD053 ad-hoc không reset lịch.

<a id="k2-acceptance"></a>
### Nghiệm thu K2 codec/hello profile offline tại CP181

Codex tự đối chiếu từng acceptance K2 ở bảng gate, [review waived](evidence/checkpoints/review-waiver.json),
không đổi9dòng tiêu chí. **K2 đạt cho profile offline copye9/Wine đã pin**, mode0/selector0→mode2/
selector3/prefixguard0, header37 và các frame thực dùng trên route probe. [Contract](PROTOCOL.md#login-retry-codec),
[recipe/hash/controls](evidence/live-codec-stream/README.md#cp181). Decision không do checker/sốCP/UI.

| Acceptance giữ nguyên | Bằng chứng / giới hạn đối chiếu |
|---|---|
| Dependency K0/K1/K2a và oracle KR | CP178K0/K2a, CP179K1R2; e9 actualnativebootstrap/hello có nguồn riêng CP180/181. Helper/nativeconsumer/capturedwriter là oracle, không tự sinhgolden hoặc3e9coldstarts. |
| Hello/request kế, đủ byte/checksum/seq qua nhiều frame và reconnect | CP180 nativeactualsplit/coalesced/CRCret7; CP181result1 hợp lệ→UI1002→actualTCP4loginretry→137cursor/errors0→TCP5channel. C2S1/60 ở cảhai login reset40/43; S2Cauthseq43→46, typedbody/cursor native sạch. |
| Boundarylength | CP05714nativecase hoàn tất: partial/length/CRC/wrongseq/capexact-vs-onebelow; modelcaps rõ. CP180actual13/175partial→full và leftbuffer; CP18139byteresult1/175auth consumeall. ModelFFFFFFFFblocked, full1MiBenforcement/overflow chưa biết, không ghi làpasscases. |
| Chia/gộpTCP, mode sauhello và resetstate | CP180 split13/162/coalesced2+3frames; CP181same CA/GDBinferior/actualLinuxPID,3nativehello0/0seq40→43, mode2/3seq43 tại login/retry/channel. Không đồng nhất objectaddress tái dùng với cùngTCPconnection. |

Gate này là codec/hello profile thực dùng, **không toàn grammar/headerauthority/allmodes/prefixguardon,
accountresume hoặc K3identity/cache/UI**. Labadapter chỉ map ordinal4/5 theo route đã quan sát, không
recoveredaccountpolicy hoặc productiongeneralreconnect. Native boundarycaps/modelguard và WinSockAPI
trace thiếu trongharness giữ rõ giới hạn. KRE-003/#3 còn backlog broadercodec; K3–K7/fullmatch/LAN/
Internet chưa nghiệm thu. CP181 snapshot trước PRdelivery, periodic audit vẫn kếCP184.

<a id="k3-evidence-review"></a>
<a id="k3-acceptance"></a>
### Nghiệm thu K3 profile local tại CP186

Codex đối chiếu thủ công **tiêu chí K3 giữ nguyên**, [review waived](evidence/checkpoints/review-waiver.json).
**K3 đạt cho một A, Winecopy e9 offline/identity ID1–KRLocal/một kênh normal index0**, pinned47fixtures
và opt-in291/request528/request6-name guard. [Quyết định/nguồn/giới hạn](evidence/lobby-identity/k3-acceptance.json),
[contract](PROTOCOL.md#k3-version-identity-controls), [run/hash](evidence/lobby-identity/README.md#cp186).
Đây không là fullauth, originalWindows hoặc nghiệm thu hai-client291/K4.

| Tiêu chí giữ nguyên | Đối chiếu và giới hạn nghiệm thu |
|---|---|
| Cold / thiếu / warm cache có kết quả rõ | CP186 trước run cả channel2_000/dynamic-map/stage-map đều vắng; native channel/dynamic rebuild, warm nhận lại. CP059/185 missing→write→warm/CRC đã có. Năm SSI resource giữ nguyên hash client nguồn; cold ở đây là cache route đã quan sát, không xóa asset/toànclient. Stage-map vắng chỉ CRC0. |
| Identity/kênh trên UI và ID giữa socket | CP058/182 ID1/authgetter/request6; CP183/184 producer291/panelKRLocal/roomID1. CP016 selected0/level1 và CP034 kind1→normalfactory/UI tái dùng; CP186 A vẫn tới đúng sảnh/panel sau guard. Named row đọc đúng9byte, **chưa** chứng minh caption vẽ trên UI hoặc warm deserialize trước14; baseline một sảnh/emptycaption được giữ, không đa kênh. |
| Keepalive / lifecycle codec | CP181 sameprocessretry/reset; CP184 hai run80s A panel→room/map; CP185/186 bounded65s channel và actualC2S. Không timeout trong cửa sổ đã đo, không SLA/resume tổng quát. |
| Đúng bytes nhưng sai socket/stage bị từ chối có log | Tám AF_UNIX/publicreceive cases, nativeframer độc lập chấp nhận mọi frame, exactreason/no reply/no ứng dụng marker; positivesA/B/C và sequence riêng. Frames sai là input dựng từ capture, policy emulator, không original auth law. |
| Sai version/identity lỗi rõ | Bootstrap65535 nativeguard0/return0/ErrCannotLogin;966 compare0→NGMPatchError→OSACK→nativeReturn0→ErrCannotLogin(5:1754). Không tự gán errorcode/rootcause. Identity profile/ID/name sai fail trướclisten; request6 name/shape sai logged/rejected theo localpolicy. CP181 giữ genericautherror/retry riêng, không gán là version/credential mapping. |

Không suy K3 thành fullaccount/token/resume, WCPu16/versionfield auth, updater thành công, arbitrarycache/
multichannel, B/Cactual291, K4 hoặc fullmatch/LAN/Internet. Các phần đó giữ backlog theo trigger;
không mở lại namedcache/version/selector đã giải chỉ để tăng sốCP. K0/K1/K2 dependency profile đã đạt.

<a id="k4-evidence-review"></a>
<a id="k4-acceptance"></a>
### Nghiệm thu K4 phòng/load/start local tại CP187

Codex đối chiếu thủ công **tiêu chí K4 giữ nguyên**, reviewwaived; [quyết định/scope/nguồn](evidence/lobby-identity/k4-acceptance.json),
[contract/runtime](PROTOCOL.md#current-291-room-integration), [run/hash](evidence/lobby-identity/README.md#cp187).
**K4 đạt cho hai actualAB, Winecopy4b781f…/offline/currentbasicpreset+291/nameguard/normalManner/mộtmap**.
Đây là room/map/load/startgate, không K5/K6a hoặc hai outcome. Run hiện có một outcome/reset/Start2,
sau đó inputcontroller alignmentfail vòng2; giữ exit1/cleanupabort, không làm checkpass bằng xóa lỗi.

| Tiêu chí giữ nguyên | Đối chiếu / giới hạn |
|---|---|
| Hai client cùng phòng/slot/host, create/join | Native291128/128/errors0/name/panelKRLocal/KRGuest; currentcreate/join/teamACK1 và native106/110/117. A86→slotID1clear/Bchief1→A82rejoin; ảnh cảhai và actualC2S/sequence khớp. Reuse CP161/162/166; không đa phòng/password/fullaccountauth. |
| Map/roster/spawn hợp lệ | CP065 native121/spawn0/1 và CP171195flags/knownmap tái dùng; current47fixture/policies byteidentical ngoại trừ own_profile, actual121129/129 và clockboth hai lần. Không suy mọi map/8clients. |
| Chờ người chưa load | CP135/136 connectedhold/deadline negative+positive và CP153 observercontrol giữ phạm vi. Currenthai30sbarriers chỉ complete sau actual99củacảhai;6.079/6.790s, không thay deadline. Không tự claim currentBhostdeadline variant mới. |
| Host rời, join/leave, Start lặp | CurrentactualAhostBack/Bpromotion/Arejoin; A95/B104→firstnormal175180/roomreset→A95/B104→nativeclock2. Hai starts là Start lặp; currentStart3/secondoutcome chưa đo. CP168/171Start3 cũ giữ historical riêng. |
| Provenance/authority/dependency | K0/K1/K2/K3profile đã đạt; copyimportABI-only correction/native.imageunchanged và currentABidentity cónguồn. Codexmanual/selfcheck theo waiver, field/timeouts/rank/hostpolicy vẫn design. Không originalWindows/fullauth/fullmatch/LANInternet. |

Giới hạn inputalignment vòng2 thuộc bước K5/K6a; không đổi tiêu chí hoặc gọi bộ2outcomes đã thành công.
Phạm vi feature ngoài profile vẫn partial/unknown; `gate=passed` không là `presence/allimplementation=complete`.

<a id="k6a-acceptance"></a>
### K6a hai client cùng host — đạt scope basic local tại CP189

Tiêu chí K6a giữ nguyên: identity/slot riêng trong mạng thử cô lập; hai chiều movement/bom,
cùng trap/death/result và hai trận liên tiếp, không dùng guest tổng hợp thay client B.

Codex tự đối chiếu **tiêu chí K6 giữ nguyên cho bước K6a**, reviewwaived;
[decision theo từng tiêu chí](evidence/peer-endpoints/k6a-acceptance.json),
[contract/limits](PROTOCOL.md#current-basic-contest-reset), [recipe/hash](evidence/peer-endpoints/README.md#cp189).
K0–K5 scoped giữ đạt. CurrentCP189 có hai contestedcycles/normalending/Ready2, per-role statreset,
strict life và4peerwindows; CP188 currentBowner/hiddenbasic2cycles/Start3 và pendinghostEOF là control
riêng, oldCP155exactoldUDP cũng giữ riêng. Đã đối chiếu source/profile/copyconsumer continuity;
không gọi chúng cùng một run, không suy fulloriginalauthority.

**K6a đạt cho hai actualAB trên cùnghost/privatevirtualhosts/Winecopy4b/basic+291/nameguard/Manner/ExVillage01.**
Không fullinventory/maps/resetvariants hoặc originalWindows. CatalogK6 vẫn open ngoài các scope đã nghiệm thu.
Bước LAN được đối chiếu tiếp bên dưới:

<a id="k6b-acceptance"></a>
### K6b LAN giả lập — đạt tại CP191 theo điều kiện môi trường mới

[Owner directive](evidence/owner-directives/emulated-lan.json) thay yêu cầu hai máy vật lý bằng hai host
mô phỏng; giữ yêu cầu actualKR/endpoint/port/2normalcycles/consistency/disconnect/rejoin. [Decision theo
mỗi tiêu chí](evidence/peer-endpoints/k6b-acceptance.json), [contract/limits](PROTOCOL.md#emulated-lan-acceptance).
Đối chiếu thủ công CP189 raw/native/capture 2cycles và CP190EOFcontrol riêng;2CA/2wineserver trong
hai netns/NIC/IP .2/.3, bridge .1/directTCPUDP,1282PCAPrecords→nativepeer4windows/2keys. NormalAhostBack/
rejoin cólog, pendingBEOF/Acleanup/fence riêng. Không resume killedBaccount hoặc allfailurevariants.
**K6b đạt đúng scope LANgiảlập/sharedphysicalhost/kernel**; không có phép đo physicalLAN hoặc Internet mới.
CP191 cũng xác nhận BpostwinRollerreset hai lần quaStart2/3 trênportablebundle; không mở rộng gateitem/map.
Lịch sửCP189190 không sửa; quyết định nghiệmthu mới phát sinh sau chỉ đạoowner, scriptskhông tựđónggate.

Các bước chưa nghiệm thu:

- **K6c — hai mạng Internet:** kiểm endpoint/NAT và P2P hoặc relay được chọn; hai bên cùng state/result,
  hai trận liên tiếp, latency/loss/disconnect được ghi. Chỉ endpoint thử do dự án kiểm soát trong allowlist;
  giữ block Nexon/live auth. Tại CP028 NAT/relay/Internet còn chưa kiểm; CP193–196 đã có [NAT tĩnh/UDP impairment/EOF](PROTOCOL.md#nat-netem-host-eof), chưa Internet/NAT discovery/relay gốc.

**Ghi chú khi lập kế hoạch26/10 (lịch sử, không phải task hiện hành):**
K2a chạy song song K1 bằng dump/DB có sẵn. Nếu K1 thất bại, kết quả K2a vẫn là RE tĩnh và K2 chưa
qua gate. Không dựng cả lobby trước khi có hello KR thật. Không hứa ngày xong K6 khi K1/K2 còn
unknown; ước lượng lại sau từng gate từ số câu hỏi đã đóng và lượt đo thực tế.

**Điều kiện network từ K0/K1 trở đi:** cô lập outbound của VM/client trước khi chạy; chỉ cho host
local stub/capture theo allowlist. Không có traffic tới Nexon/NGM/live auth thật là điều kiện pass.
Nếu quan sát attempt tới endpoint thật, giữ egress block, lưu API/log/pcap rồi dừng run để sửa local
redirect/config theo contract; không nới egress hoặc fetch patch CDN để vượt gate. Attempt bị chặn
chưa phải traffic đã tới server thật, nhưng run đó không được dùng làm demo local hoàn chỉnh.

**Ba bước của K1:**

- **K1a — ownership/control:** kiểm ba recipe CA đơn lẻ, NewPatcher đơn lẻ và CA+NewPatcher trên cùng
  bản copy/VM; recipe được chọn để qua gate K1 phải có **ba cold start riêng của recipe đó**. Ghi process tree,
  module CA tại base tương ứng, cửa sổ đúng PID và event file/registry/network bằng quan sát ngoài
  process (ETW/Procmon khi có). Kết quả phải phân biệt exit, loop, alive-stall và modal dialog. Đây là
  characterization; chứng minh stall giúp đóng bước này nhưng **không** cho K1 qua gate network.
- **K1b — contract:** truy `CA!ProcessPatch` riêng và caller/config `NewPatcher!4118E0`, ghép command
  line/IPC/token/file/return mà CA thật sự chờ. Chỉ kết nối hai đường khi có evidence; có thể loại bỏ
  giả thuyết NewPatcher nếu không nằm trên route đã quan sát. Không đoán patch schema từ literal NFO.
- **K1c — thử nghiệm:** điều phối chọn stub local đúng contract trước; nếu không đủ thì patch bản copy
  theo KRE-D01. Giữ control plain, chỉ đổi một yếu tố mỗi ca, rồi kiểm gate ba cold start. Khi cả hai
  lượt không có cạnh/observable mới, dừng nhánh và Codex ghi checkpoint để đổi phép đo.

**Quyết định vận hành DESIGN KRE-D01 (26/10/2026):** theo quyền offline-preservation và chỉ đạo toàn
quyền mới của chủ dự án, Codex được chọn stub launcher/patch service, loại bỏ/đổi tên thành phần AC
và patch **file trên bản copy offline** của CA/NewPatcher khi có bằng chứng cần thiết; không phải xin
lại quyết định đó. Trước mỗi thí nghiệm ghi target/hash, module/process ownership, lý do, offset/RVA,
byte trước/sau, file bị loại/đổi tên, giả thuyết side effect và recipe rollback. Control dùng bản copy
plain đầy đủ, cùng VM/config/cache và log mới; so với ca sửa một yếu tố. Chọn quan sát ngoài process
trước vì inject/hook CA từng làm nhiễu. Không lặp DLL/IAT/prologue hook đã thất bại nếu chưa có điều
kiện mới; patch memory/hook xâm lấn không thuộc recipe mặc định. Runtime cô lập khỏi Nexon, không
phân phối binary/AC; lựa chọn policy này không biến kết quả modified-client thành vanilla behavior.

### Phương pháp khi mở nhánh và điểm dừng

Khi mở một nhánh, áp dụng phương pháp tương ứng; backlog hiện hành chỉ ở mục thứ tự phía trên.

1. **Nhánh bootstrap:** K1a xác định process/cửa sổ và điểm chờ; sau đó kiểm riêng `CA!ProcessPatch`
   và giả thuyết `NewPatcher` consumer/caller/config → tham số/IPC → điều kiện thành công. Đọc request
   thực trước khi dựng HTTP/DNS/launcher stub. Không đoán `version=967`
   là toàn bộ handshake và không lấy URL có trong binary làm endpoint đã quan sát.
2. **Wire từ hai đầu:** xref socket API và serializer từ phía gửi; đi ngược handler nhận từ các
   mỏ neo KR đã biết. Ghép cặp theo field order/width/consumer, không theo số opcode gần nhau.
   Chỉ mở những wrapper/VM chặn hello hoặc packet đang cần, không devirt toàn Themida.
3. **Phục hồi state machine:** ghi state trước/sau, guard, request→reply/broadcast, timer, cache,
   socket nào và byte nào được tiêu thụ; kiểm từng chuyển cảnh rồi mới thêm gói.
4. **Authority trước core:** phân biệt vị trí hiển thị, mô phỏng local, báo cáo client, xác nhận server
   và peer. Không ghép handler KR183 dành cho `CGOGhost` thành movement packet của bomber sống.

Mỗi câu hỏi tối đa **hai lượt**, mỗi lượt mặc định **8 request phân tích / 4 thân hàm mới**;
caller/callee đọc từ cache cùng hash không tính như phát hiện. Hết lượt không có cạnh mới:
ghi entry/blocker, vùng đã tìm, dữ liệu runtime còn thiếu và thí nghiệm tiếp; điều phối tự chuyển
hướng có bằng chứng. Bootstrap/codec đang unknown không được “giải” bằng design packet rồi đánh dấu
tương thích. Design được dùng cho policy server mất nguồn sau khi client đã chấp nhận wire contract.

Runtime dùng bản copy, chỉ network cục bộ; không sửa binary/pack/DB gốc. Linux giữ CI/build server.
Theo chỉ đạo26/10,ưu tiên Wine;recipe/merge ở bảng hiện hành.
Các run socket hai silent giữ trong evidence từng recipe;probe WCP sau đó có phạm vi riêng tại
[PROTOCOL](PROTOCOL.md#wcp). Không ghi kết quả mới thành plan khác.
Windows VM cô lập giữ vai trò
đối chứng nguyên bản; lỗi Wine không tự bác kết quả Windows. Không mở lại CI/đóng gói
Windows/macOS của Rust. Patch offline theo KRE-D01, phạm vi triển khai vẫn theo từng task KRE.

### Kiến trúc server đề xuất và mức tái sử dụng

Triển khai tại `server/` trong **chính repo `boom-kr-server`**; tool ở `tools/`, docs ở `docs/`.
Các repo VN/KR/Rust cũ chỉ là nguồn tham chiếu. Không thêm code/doc mới vào chúng để tránh hai nguồn chuẩn.

```text
CA.exe KR (bản copy) ── TCP ── KrWire/PacketReader ── KrSession/Stage ── lobby/room/round
         │                         │                    │                   │
         └── UDP peer đã đo; relay ? ──────┴────────────────────┴── authority audit ┘
                                   └── raw + decoded trace → fixture/replay
```

| Thành phần | Thiết kế / cách tái sử dụng |
|---|---|
| `kr-wire` (đề xuất) | Buffer TCP có giới hạn, codec state riêng từng socket/chiều, packet registry `(build,direction,stage,opcode)`, reader có cursor và báo trailing bytes. Dùng VN làm reference/fixture control; constants KR vào profile có nguồn. |
| `kr-session` / server (đề xuất) | Auth local, redirect token một lần, ID account/session/slot tách nhau; state machine idempotent, timer gắn round generation. SQLite chỉ cần khi identity contract ổn; không thiết kế economy sớm. |
| Room/round | Một owner cập nhật mỗi phòng, broadcast theo roster, phân biệt request/ack/result. Tái dùng ý tưởng generation guard từ VN; policy ready/host/timeout phải khớp consumer KR hoặc ghi design. |
| UDP | P2P local đã đo ở CP027–028 qua actual99→153 và UDP receive. NAT tĩnh/LANgiảlập đã có CP191–196; relay/NATdiscovery/Internet còn unknown, không đọc thiết kế banđầu thành chưa đo LAN. Không dùng 3 port login VN làm endpoint movement. |
| `boom-core` | Ứng viên dùng sau audit authority: shadow validation trước; chỉ phát kết quả authoritative khi chứng minh client có đường nhận/correction phù hợp. Nếu không có, giữ mô hình lai để đạt demo tương thích và ghi rõ giới hạn. |
| Content | Dùng reader pack/map/ODT đã có và inventory KR; map/item/character numeric ID có crosswalk theo build. Client gốc tự render/audio; không port renderer mới để giải wire. |

### Danh mục task lịch sử và phân công

Thứ tự/trạng thái hiện hành ở issue-order; danh mục dưới giữ phạm vi lịch sử. KRE-001/#1 đã đóng, #2/#3 backlog. Các task thuộc repo này: **KRE-001 [#1](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/1)**,
**KRE-002 [#2](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/2)**,
**KRE-003 [#3](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/3)**. Issue cũ boom-rs đã được
transfer, không giữ hai bảng việc. Mỗi task có baseline, owner/reviewer, file và acceptance theo AGENTS.

| Task | Câu hỏi / đầu ra | Dependency | Owner đề xuất / reviewer |
|---|---|---|---|
| KRE-001 | Khóa target/copy/dump/DB, chạy control và manifest | — | Codex / tự kiểm (review miễn28/10) |
| KRE-002 | CA ProcessPatch: response/caller/guard; bootstrap local và điểm chặn sau đó; NewPatcher là nhánh tham chiếu | KRE-001 | Codex RE/code/runtime / tự kiểm (review miễn28/10) |
| KRE-003 | KR socket→framer→reader/writer; constants và fixture độc lập | KRE-001; gate runtime cần KRE-002 | Codex RE / tự kiểm (review miễn28/10) |
| KRE-004 | Wine unpack/patch copy + harness, issue6/PR7 đã xong trong scope tới local accept; chưa nghiệm thu toàn K1 | KRE-001/002 | Codex / tự kiểm (review miễn28/10) |
| KRE-005 | Probe server hello, version/mode/seq, qua K2 | KRE-003/004 | Codex / tự kiểm (review miễn28/10) |
| KRE-006 | Login/channel/cache→lobby và K3 | KRE-005 | Codex RE/code / tự kiểm (review miễn28/10) |
| KRE-007 | Room/roster/map/spawn/loading và K4 | KRE-006 | Codex RE/code / tự kiểm (review miễn28/10) |
| KRE-008 | Bomb/item/trap/result, authority map và K5 | KRE-007 | Codex RE/code / tự kiểm (review miễn28/10) |
| KRE-009 | UDP/peer hoặc relay, đồng bộ hai client và K6 | KRE-008; có thể RE static từ KRE-003 | Codex RE/code / tự kiểm (review miễn28/10) |
| KRE-010 | Race/disconnect, bốn client, Linux packaging và K7 | KRE-009 | Codex / tự kiểm (review miễn28/10) |
| KRE-017 / issue68 | Admission C trước round1 sau host setup EOF; B/C played round CP054; baseline75606eb; tự kiểm/integration theo [PR69](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/69) | KRE-005b/CP040/051/053 | Codex / tự kiểm (waived) |
| KRE-016 / issue66 | Host EOF khi B chưa map: deferred cleanup/promotion CP053; baseline3617e69; tự kiểm/integration theo [PR67](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/67) | KRE-005b/CP034/050–052 | Codex / tự kiểm (waived) |
| KRE-015 / issue64 | Guest setup EOF/slot ma: control + native fix CP052, audit AUD016; baselinec764aa5; tự kiểm/integration theo [PR65](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/65) | KRE-005b/CP034/050/051 | Codex / tự kiểm (waived) |
| KRE-014 / issue62 | Service A/B→B/C: C identity3, B Start và một vòng bom/statepeer/result/return CP051; baseline9c6ac92; tự kiểm/integration theo [PR63](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/63) | KRE-005b/CP040/044/050 | Codex / tự kiểm (waived) |
| KRE-013 / issue60 | Service EOF/forfeit/cleanup + hold99 CP050; baselinea6b62cd; integration theo [PR61](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/61) | KRE-005b/CP039–044 | Codex / tự kiểm (waived) |
| KRE-012 / issue58 | Guard124 old/current, native client effect CP049; baseline4d77eaf; integration theo [PR59](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/59) | KRE-005b/CP048 | Codex / tự kiểm (waived) |
| KRE-011 / issue56 | Catalog/cây/coverage và consistency check CP048; baseline5ffd925; PR57 đã merge/issue56 đóng | KRE-005b/current evidence | Codex / tự kiểm (waived) |

Từ chỉ đạo28/10, Codex solo RE/code/runtime và tự kiểm; không gọi dsh/Claude nữa.
Review độc lập được chủ dự án miễn theo [nguồn](evidence/checkpoints/review-waiver.json).
Các tiêu chí evidence/verification/gate và PR vẫn giữ; yêu cầu reviewer trong plan trước đây
được thay bằng tự kiểm theo AGENTS, không tự coi gate đã pass. Report/approval cũ giữ provenance.
Một owner mỗi target/runtime, DB riêng; raw giữ local và checkpoint theo mốc/blocker.



Trong lượt lập kế hoạch26/10 (lịch sử), sau khi mở quyền và sửa alias cwd cũ, hai session đã research/review độc
lập. Findings vòng đầu đã được xử lý; **dsh1 và dsh2 APPROVE nội dung plan ở `5e9ebb64`** theo phạm vi
bootstrap và protocol/authority. Bằng chứng review xem [hồ sơ](evidence/bootstrap/README.md).
Đây là duyệt kế hoạch, không phải đã chạy KR hoặc nghiệm thu gate K0–K7; lỗi điều phối ban đầu không
phải kết quả RE phủ định.
Câu hỏi cần đóng: [Q-022…Q-029](RESEARCH.md).
