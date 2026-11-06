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
[PLAN](PLAN.md#current-order) giữ thứ tự. Fullauthentication/cache/K5/K6a vẫn chưa nghiệm thu.

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

<a id="feature-gaps"></a>
## Khoảng trống theo cơ chế

Đây là câu hỏi/phép đo tối thiểu, **không là thứ tự công việc**; thứ tự duy nhất ở [PLAN](PLAN.md#current-order).
Phạm vi đã biết/giới hạn nằm ở contract được catalog dẫn; không đọc `unknown` thành vắng mặt.
Bảng được [coverage](COVERAGE.md) trích tự động để tra theo chức năng, không sửa bản sinh.

| ID | Khoảng trống hiện hành | Phép kiểm tối thiểu |
|---|---|---|
| <a id="gap-process-patch"></a>KR-B001 | Đối chứng Windows nguyên bản/ownership và recipe cùng cold start chưa nghiệm thu. | Tái dùng recipe có hash; ba cold start đúng một recipe, gắn network attempt với PID; so Windows control khi mở nhánh bootstrap. |
| <a id="gap-wcp-config"></a>KR-B002 | Fixture tổng hợp đã qua guard; oneclickuser/+E9C và dữ liệu/cache thật chưa đủ. | Chỉ mở consumer/config đang chặn runtime; so nhánh với và không tham số trên cùng copy, không đo lại +E78 đã giải. |
| <a id="gap-socket-codec"></a>KR-B003 | CP057 native stream splits/coalesce/seq và socketpair khớp; prefixguardoff/modelcap,FFFFFFFF allocationmodel blocked, full modes/string/blob/reset/live reconnect chưa nghiệm thu. | Reuse [scope/model](PROTOCOL.md#codec-stream-boundaries); không chạy lại76split hoặc tăng allocator để làm pass. Khi consumer auth/cache gây trigger mới, đo đúng field/mode/prefix/lifecycle trên actual connection; fresh synthetic socket không là reconnect thật. |
| <a id="gap-auth-identity"></a>KR-L001 | [CP073](PROTOCOL.md#launcher-error-attribution): auxiliarylocale1,0 valid/init0→2000:0/noTCP; main currentA/Bactual67 positive. HistoricalmainSIGABRT/HTTPstartupcause vẫn unknown, không gánpopup là root. Fullidentity/token/accountauthority còn mở. | TheoPLAN trace actualerrorstate/caller nếu mainfail; không bypasslocale hoặc tìm lại auth137/135/string/cache. Providerinit sâu chỉ mở khi thực sự chặn main, chưa phải trọng tâm chơi. |
| <a id="gap-redirect-lobby"></a>KR-L002 | UI sảnh copy đã có; thông tin user/cache/version và variants chưa đủ K3. | Kiểm identity hiển thị cùng request6/17/241 trên cold/warm/missing cache, log stage/seq; không suy từ cửa sổ. |
| <a id="gap-profile-cache"></a>KR-L003 | CP059 namedfxdynmapinfo missing→write→warm đã nhận; CRC selector+representation trướcinflate giải lệchraw/nén. Toàncache/profile/version/corrupt/reconnect vẫn mở. | Reuse [scope/CRC](PROTOCOL.md#dynamic-cache-representation); không lặp namedcache này. Chỉ mở cachefile/variant khác khi native consumer hoặc acceptance có trigger; freshprefix không là coldallcache. |
| <a id="gap-create-join"></a>KR-R001 | [CP161](PROTOCOL.md#current-room-title) current A81→B91 title/Back/rejoin/UI hai view và hai vòng production/Ready đã đo;32 byte/empty-password/known single row là policy. | Password/full/error/general room authority, C/recreate/generalpromotion ngoài [CP166/168](PROTOCOL.md#current-mode-promoted-start3)/multi-row còn mở theo trigger. Không RE lại known81/91 hoặc lấy title làm identity. |
| <a id="gap-roster-ready-host"></a>KR-R002 | [CP163](PROTOCOL.md#battle-eight-roster) actual selector1/8synthetic slots:4–4 check0/104,1–7 check7/no104, Ready/parser/map/capacity/getter có nguồn. CP162 actual Free/Manner/ACK/reset đã nối. | Không lặp guard/roster/map/input RE. CP165 actualpreset2cycles/Start3 đãđo; CP166 currentmodepromotion/rejoin đãtới clock; CP167đãhai cycleBhost; CP168BStart3/AUD050đãđo; CP171/172basiccycles/Start3 đã đo; tiếp PLAN theo gap thật; general roster8/service admission/8peer/Battle8/Random/C/generalpromotion/row91 metadata còn thiếu sau local baseline. |
| <a id="gap-map-spawn-loading"></a>KR-R003 | [CP153](PROTOCOL.md#loading-hot-gate-contest): B32390 gate riêng khi load/restoreclock/0loadinghits; hai actual99/barrier5,94s/4,60s. CP151R2 historicalcause không bị tự sửa. | Scope loading recipe đã giải; không lặp writer/loader RE hoặc tăng30s. Generalcoldload/membership/timeout variants chỉ mở theo trigger; theo thứ tự hiện hành trong PLAN. |
| <a id="gap-room-membership"></a>KR-R004 | [CP175](PROTOCOL.md#a-survivor-admission) C3/slot1/Join/Ready/Start; [CP176](PROTOCOL.md#a-c-basic-cycles) A/C2normal/reset/Ready2. | CinitialpreReadyblack/return2beforefocusblack giữ giới hạn; finalroom vẽ cả hai. Generalmembership/accountresume/laterStart3 và profilecapacity theo vacancy còn mở. |
| <a id="gap-round-clock"></a>KR-F001 | CP155 baepoch actual99/nativeclock/barrier trước30s, B32390restore mỗi clock; same-run freshness/contest/EOF cónguồn. | Không RE lại99/load/clock; tiếp thứ tự hiện hành trong PLAN. Originalclock/timeoutauthority ngoài profile giữunknown. |
| <a id="gap-peer-movement"></a>KR-F002 | [CP176](PROTOCOL.md#a-c-basic-cycles) current A/C4nativewindows/2keys/target/currentprogress cùng2normalcycles; CP173 giữ AB scope. | FullUDP/auth/reliability/generalphysics/NAT unknown; không đo lại currentconsumer hoặc AC peer, theo PLAN profile/capacity delivery. |
| <a id="gap-udp-freshness"></a>KR-F003 | [CP155](PROTOCOL.md#direct-oldudp-contest-eof) exactR1Bframe→R2actualendpoint/queue/key59≠56/drop/noapply+11currentpositive vàfreshBRight trong cùng2contest/Ready/EOFrecipe. | Scope1key/currentIDs đã giải, không RE lại CP145consumer. low8wrap/forgedcurrentkey/fullgrammar/auth cònunknown theo trigger; tiếp thứ tự hiện hành trong PLAN. |
| <a id="gap-bomb-lifetime"></a>KR-F004 | [CP174](PROTOCOL.md#basic-host-b-eof) basicB/A/A IDs1..6/native195/reset rồithirdBID7pending/ownedBEOF/round_closed/no158; CP155scope cũ giữ. | Genericpool/auth/multiple/races/low8wrap khác unknown; khôngRElạicurrentpendingfence. Admission theoPLAN. |
| <a id="gap-blast-block-chain"></a>KR-F005 | [CP172](PROTOCOL.md#hidden-cell-admission-current) basic Box/reveal27/28/142guard/3effects/reset/Start3 đã kiểm. | Generalmoving/order/allmaps/fullworldauthority unknown; không RE lại consumer. CP173 Bowner/native195 đã ghép; CP174BhostEOF đã kiểm; admission theoPLAN. |
| <a id="gap-floor-pickup"></a>KR-F006 | [CP172](PROTOCOL.md#hidden-cell-admission-current) terrain currentclearedcell/generation/commit/roster guard và native basic2cycles/reset/Start3 có nguồn; no-terrain giữ weak report scope. | Full142key/auth/distance/freshness/fairness/two-nativecontenders unknown. Same old142 sau legitimatecurrentclear chưa phân biệt; không gọi policy guard là original authority. CP173B118/bidirectional đã kiểm; CP174BhostEOF đã kiểm; admission theoPLAN. |
| <a id="gap-item-stats"></a>KR-F007 | [CP087](PROTOCOL.md#capacity-reservations) profileAinitial1max10/B2max7, acceptedBomb tăngtracked/native1→2 và resetthật hai vòng. Runtimeatmax/character/mode/pxs/TTL/fullreset khôngđủ. | Reuse source/currentstat; servertracking làdesign, chưa nativepoollaw. Không port base/max quaC/characterchưađo; khivào nhánh max/ability phải consumer/control riêng. |
| <a id="gap-needle"></a>KR-F008 | CP113opt-inonecredit/pair/peerescape; [CP114](PROTOCOL.md#needle-round-reset) currentR1secondtrapcạn/no1,0/actual19→resultreturn/Start2fresh145credit/escape. R2active, không2results. | Multi-charge/inventoryauth/generalresetvariants cònmởtheotrigger; khônglặpcurrentonecredit/depleted/reset. Rescue18độc lậpởcontactgap/PLAN. |
| <a id="gap-contact-state"></a>KR-F009 | [CP158](PROTOCOL.md#current-trap-udp-counter): native smallgap4 drop/largegap17 accept/controlledremote56→0→56→1; two observational runs không tái hiện CP157. | Exact naturalCP157state0packet/apply vẫn candidate/unknown; controlledcapability không retrospectivecause. Không lặp counter/body RE; nếu trigger cần, đo actual non-replayed packet/queue trướctrap. Hiện theo PLAN production baseline, không tự sửa client/claimbugfixed. |
| <a id="gap-ability-debuff"></a>KR-F010 | Shoe/Punch/debuff/TTL chỉ có bảng và một phần selector; không chuyển kết quả MaxPower sang mọi effect. | Chọn đúng ability cần cho Battle rồi trace1191840 selector/virtual tương ứng; đo consume/effect/peer/reset một case và control. |
| <a id="gap-mount"></a>KR-F011 | Group2 có nhánh riêng; điều kiện/effect/giới hạn/reset runtime KR chưa kiểm. | Chỉ khi fixture Battle cần mount: trace11928A2 tới consumer rồi một pickup/peer/reset; hiện loại khỏi fixture theo policy. |
| <a id="gap-active-other"></a>KR-F012 | Dart/Oxygen/Potion/Spring/Drill/RemoteBomb mới là phân loại, chưa chứng minh cùng đường kim. | Chọn một group đang cần, tìm consumer Ctrl/report/confirm; đo điều kiện, tiêu hao, hiệu ứng và peer, không quét từng row. |
| <a id="gap-timed-variants"></a>KR-F013 | Shield/Sensor/SJNeedle raw tail chưa chứng minh đơn vị/thời hạn/hành vi KR. | Trace consumer timer/group cần dùng, đo start/expiry/cancel/reset và peer với control; không suy ms từ tên hoặc raw số. |
| <a id="gap-mode-items"></a>KR-F014 | ItemAttr cho thấy row/group; presence cơ chế/mode guard/reward runtime vẫn candidate, ngoài Battle tối thiểu. | Chỉ mở group mode khi PLAN mở mode đó; bắt guard/consumer trước thử runtime, HPNeedle17 không là Needle145. |
| <a id="gap-peer-relay"></a>KR-F015 | CP155 currentallTCPdirect/oldUDPnegative+currentpositive/2contests/4peerwindows/EOF3 cùngrun. | RealLAN/NAT/Internet/relay/fullUDP chưa đo; theo thứ tự hiện hành trong PLAN, không gọi virtualhosts là physicalLAN. |
| <a id="gap-result-return"></a>KR-X001 | [CP176](PROTOCOL.md#a-c-basic-cycles) Ahost/Cguest2normalresults/180/room/actualCReady2/native110; CP174thirdforfeit giữ riêng. | Originalarbitration/UI/generalreset chưa đủ; noStart3/thirdoutcome. Không coi ảnh đen trước focus là rootcause; finalroom đã render. |
| <a id="gap-round-reset"></a>KR-X002 | [CP176](PROTOCOL.md#a-c-basic-cycles) AC2keys/6entities/3Aeffects/195/stat/hiddeneligibility reset và Ready2 có nguồn. | Wholeinventory/modes/low8wrap/generalreset/thirdStart còn thiếu theo trigger, không lặp knownACcycles. |
| <a id="gap-disconnect-replacement"></a>KR-X003 | [CP177](PROTOCOL.md#replacement-profile) optional exported preset/capacity theo vacancy và actual C0/C1 initial/118/159158 đã kiểm; CP176 giữ AC2normalcycles riêng. | Maximum/exhaustion/general membership/account resume chưa biết; C0 startup SIGSEGV giữ nguyên, đối chứng pass không là rootcause. Theo PLAN acceptance/control, không lặp known C0/C1 entry hoặc ACcycles. |
| <a id="gap-diagnostic-service"></a>KR-S001 | [CP177](PROTOCOL.md#replacement-profile) optional replacement preset/47pins/derived capacity/284server và7export tests; native C0/C1 actual118159158 đã đo. | Theo PLAN đối chiếu acceptance/dependency/control trước phép đo còn thiếu; nofullauth/physicalLANInternet8client/gate. Startup unknown không được suy thành capacity regression/fix. |
| <a id="gap-service-hardening"></a>KR-S002 | Chưa LAN/Internet/bốn client/restart/backpressure/rate limit/persistence đủ K7. | Sau local gate có evidence, thử hai máy theo K6b rồi loss/disconnect/restart; không lấy loopback accept làm multiplayer. |


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
