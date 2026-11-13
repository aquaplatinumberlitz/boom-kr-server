# UDP, peer, movement và stale fences

[Mục lục](../PROTOCOL.md) · [Câu hỏi đang mở](../RESEARCH.md) · [Thứ tự việc](../PLAN.md#dashboard).

<a id="kr-peer-entry"></a>
### Request99 và bảng peer KR153 — CP025–027

[Evidence CP026](../evidence/peer153/README.md). **kr_runtime confirmed hẹp trên copy Wine**,
không thêm patch vào SHAe9bb47…; native decoder trên modO/a08950… và code11vùng khớp ba artifact.
CP025 chỉ xác định153 tĩnh; CP026 đã dispatch153→E5E6D0→E825D0 trong bốn run tuần tự.

**Request99 (CP025):** `E7F301→13E1470→13DE7C0` lấy tối đa hai IPv4 qua hostname/hostlookup;
`E7F33C→13E14A0→14E5670` lấy u16networkbase+24. Bind9751/9345 khớp99 trong hai runCP025;
còn có socket bind khác, không gán cả hai cho movement. Trong namespace IP=127.0.0.1/0.

**Body153 đã đo**, index từ0, typed field dùng obfuscation theo codec KR; relay/header values ngoài
đích ghi vẫn là `emu_choice`. Không dùng suffix trần hoặc schema VN146 thay fullKR153:

| Phần | Decoder/giới hạn |
|---|---|
| Stream đầu | `E88790→vf5C/14EB410→14EB430`: u32obf stored length, rồi blob obf theo chunk4/2/1. Raw sau decode=`18` +u32LE length +compression0 +data. Ca đo: raw7byte=`18020000000000`; data=u8count0. Outer tổng11byte. `E8A340→E8A3F0` đọc count rồi count×u8 vào vector stage+6404. Chỉ count0/compression0 đã kiểm. |
| u32 đầu | Đưa vào `E13490`; chọn0, quan sát clock origin khởi tạo lại. Semantics game time candidate; chưa thử nonzero. |
| u32 thứ hai | Gán stage+63E0; CP026 chọn12345, CP047 đổi12346 vòng2 và kiểm byte khóa trong [movement handler](#udp-generation). Chưa đủ toàn UDP session/identity contract. |
| 2×u32, u8 | Stage+64D4/+64D8 nhân1000; chọn180/180 giây và flag0→secure+647C. Header kết thúc cursor28. |
| Relay u32IP/u16port/u8flag | Chọn0/0/0. Static IPnonzero quahtonl→13E1140→14E57B0; chưa kiểm relay runtime/endian. Không áp kết luận endian peer cho relay. |
| 26slot | Mỗi slot u8on. On≠0 đọc3×(u32IP,u16port); array3×8byte→13E1190 khi1243330(slot) đủ điều kiện. Ca đo slots0/1 có profile, localID1/guestID2. |

Payload61 (mọi slot tắt),79 (guest1 bật),97 (local0+guest1 bật) đều tới cuốiE83E53 với cursor=payload,
error54/error58=0. Stream cursor11, header28; không có read packet khác giữa header và relay trên
đường đã đo. Chưa kiểm mọi mode, vector khác rỗng, optional state hoặc handler ở stage khác.

**Slot bật/tắt có side effect**, không chỉ khai báo địa chỉ. ĐoạnE83A35/E83A89 gọi12400D0 với slot
tắt đang có profile. Hai run local0off không phát118 sau input; ca local0on/ba endpoint0 giữ guest
endpoint như cũ phát ba118, ghép159/158 nhưCP025. Ảnh alloff hiệnComputer0/1; không khái quát
đây là toàn semantics leave/disconnect hoặc nguyên nhân duy nhất của mọi input guard.

**Peer endpoint và UDP đã đo:** guestslot1→13E1190(localID1,slotID2)→14E57B0; ca cả hai bật có
HostCreate trả pointer khác0. `14E7370` lưu IP/port; sender`14E61F0` gọi **htonl tại14E63A8**,
htons tại14E63BC rồi sendtoIAT16EA798 tại14E63D9. Vì vậy **peer IPv4 typed u32=0x7f000001**
cho127.0.0.1, port14001 hostu16. Control0x0100007f tạo sockaddr1.0.0.127, hooksendto có hit nhưng
sink không nhận; bản sửa chỉ IP tạo sockaddr127.0.0.1, sendto trả byte count và sink có raw/hash thật.
`13E14C0` ghi envelope trước sendto; chưa gọi các datagram này là movement/handshake đã giải mã.

Bốn recipe **CP026** gửi153 sau198 để kiểm consumer; clock origin được khởi tạo lại. SinkCP026
không phải clientB. **CP027 đã kiểm153 thay198 và hai client thật** bên dưới; không dùng recipeCP026
làm chuỗi production. Relay/NAT, codec/key/slot đầy đủ, consistency/reset và K6 vẫn mở.

KR532 cũng tớiE825D0 quaE5E820 nhưng guard703A80 trên ba artifact trảAL0; giữ giới hạnCP025,
không thử532. VN §5a-6 và helper blob giúp định hướng; KRstream/153/26slot/endian được kiểm riêng.
[Bước tiếp](../RESEARCH.md#q-023).

<a id="peer-endpoint-ip"></a>
### IP endpoint153 và UDPsource tuple — CP138

**kr_runtime/confirmed scoped; tuple-cause ngoài ca đo là candidate**, [recipe/raw/hash/limits](../evidence/peer-endpoints/README.md).
Reuse153/99 layout CP027 và cached13DE880lookup/fallback14E57B0 CP045. Đổi duy nhất IP remote
trong153 sang127.0.0.2/3, giữ99port/key/seq/restbody: actualPCAP cho native127.0.0.1 gửi tớialias;
receiver cóAPIingress, nhưng hai cửa sổ input không cóE66680handlerpacket hoặc remoteprogress dù
origin di chuyển. Đây khônglostpacket trên đường gửi hoặc bằngchứng chắcchắn về exactfallback.

Ca positive chỉquảngbá ownedaliasproxy IP+port; forward nguyênUDPpayload từorigin proxy tuple đã
quảngbá tớiactualrecipient99port. Có actualingress→key/slot/handlerapply/targetchange vàfreshremote
progress ởhai input. 192selectedpeer samples trùng pointer setendpoint;155branch-site samples
đãquan sát missing0, không ghép sốquota thành eventpairs. Negativebranch chưa probe; khôngsuy
universal sourceIPauthentication hoặc trường hợp mọi NAT/portchange. Alias controls/proxy là design
đo, không luật server gốc hoặc relay production. Service vẫn hardcode127.0.0.1 trong production.

ADown lasttrackedcall before/afterY159, target204/distance45<threshold50; freshsample saucallY246
vàrawnativegetter246. Hai strictreaders before/across-call vẫnfail; scope endpoint dùngnativeapply/
target+laterfreshmemoryprogress, khôngsửa chúng thànhpass hay suy interpolationlaw. BRight current
progress nằm trong/giữa calls. Hai run chỉmộtclock/haiinput/no bom/result, khôngfullmatch hoặcCP137
UIproof. Loopbackalias cùnghost khôngLAN/NAT/Internet. Bước kế chỉPLAN: virtualhosts/IP riêng và
kiểm source tuple/directUDP; realLAN/K6b/Internet/gate giữunknown/open.


<a id="virtual-peer"></a>
### Direct native UDP giữa hai virtualhost — CP139

**kr_runtime/confirmed hẹp**, [recipe/raw/hash/ownership](../evidence/peer-endpoints/README.md#cp139).
Hai CA/wineserver thật thuộc netnsriêng/prefixriêng, kiểmactualPID trướcinput; rootstub/bridge là
namespace thứba, khôngdefault route/physicalinterface. TCPchild127→stub10.77.0.1 cóforward; UDP
đi directqua veth, sourceIP/port đúngactual99=.2:9753/.3:9832,153remoteIPpeer+actual99port.
C2S99fieldIP0 khớpIPvirtualNICvàsourceđãđo, fieldIP1 khôngđổi/opaque, khôngxácnhận publicIP/NAT.

140PCAPrecords được nốiexactpayload vớiactualnativeingress, key/slot/apply/target/currentprogress
củaR1ADown/BRight. Có4/6nativeapply/2targetchangesmỗiphase; currentpositions tiến trong/giữacalls.
Đây khôngchỉsocketaccept haynamespacefrontend. Parentlegacymetadataudp_proxy.mode=stale vẫngiữ
trongraw nhưng0forward/replay/serviceproxyports0; route đượcchứngminh bằngvethsrc/dst/actual99port,
khôngdựa vào nhãn.12C2S127/subtype39 cósemanticsunknown, khôngđổi thànhACK/result.

Runchỉclock/haiinput/no bom/result/reset2. Servicebind/address/TCPforwardlà shimdesign thực nghiệm,
chưa productionLANconfig/auth; mộtphysicalhost/two virtualnetns khôngrealLAN/K6b/Internet/K6c/NAT/
relay/fullmatch/K6a/gate. CP137finalUIdebt giữ. Bước kế PLAN ghép nativehairesults/reset trên topology
này; khôngghépruncũ thànhmộttrậnđãnghiệmthu.


<a id="peer-duo"></a>
### Hai client thật: movement và bom — CP027

**kr_runtime confirmed trong phạm vi harness Wine**, [recipe/control/hash](../evidence/peer-duo/README.md).
Không patch thêm. CopyA/B cùngSHAe9bb47…; prefix, wineserver, display và game-dir riêng, cùng một
namespace chỉlo/no outbound/files-only NSS. Codex solo/tự kiểm, review độc lập waived; chưa qua K6a.

**Chuỗi khởi tạo đã kiểm hẹp:** 197/seq76→196/79→**153/82 thay198**; kế tiếp S→C85+3n liên tục
trên từng socket. Control một client đọc97/97/errors0, clock origin một lần, ba118/159/158 và133UDP
vào sink. Ba run hai client cũng có một clock origin mỗi phía và không dispatch198. Không suy mọi
variant/stage/reset tương đương hoặc header timeout đã là cấu hình production.

**Identity và roster:** A nhậnID1, B nhậnID2 qua auth prefix34byte, không sửa identity/ownflag từ GDB.
A slot0/KRLocal, B slot1/KRGuest; B nhận106 của chính mình trước ởseq58,110slot1/61 rồi106A/64.
Cả hai nhận cùng map ExVillage01/code278 và121 đã đo. **Trong CP027**, A click tạo/start; B dùngcreate86
và server push121 sau map/profile tạistage12. **Chưa chứng minh join/ready chuẩn hoặc phòng chung
production tại CP027**. CP028 đã nối [join/Ready thực](room-membership.md#duo-room-round); full auth/profile/cache vẫn thiếu, không gán cấpIDlocal thành login Nexon thành công.

**Trao endpoint thực:** stub chờ cả hai99 trước khi phát197/196/153. Trong153, cả slot0/1 bật;
slot của máy mình có ba endpoint0 chẩn đoán; slot bên kia dùng `(7f000001,u16port từ99),(0,0),(0,0)`.
`13E1190` đăng ký đúng localID/remoteID và slot đối diện,14E57B0 trảptr; sendto/recvfrom thật hai chiều.
Các cặp cổng9858/9720,9218/9564,9237/9351 khớp actual99/source sockaddr ở ba run.

**Movement:** run`duo-first` điều khiển A rồi B bằng bàn phím trên display riêng. Ảnh B sau inputA
cho thấy KRLocal rời spawn/đổi vị trí; ảnh A sau inputB cho thấy KRGuest rời spawn/đổi vị trí.
Có recvfrom tại13DE717 ở cả hai phía, không dùng log stub hoặc hai actor trên một client thay bằng chứng.
180 sample UDP API memory được trích từ30 successfulrecvfrom đầu/client/run;173gói17byte và7gói5byte,
được kiểm checksum bằng native14A5860 với seed0, n−1byte. Chỉ trích đúng nbyte dù GDBdump17;
đây là API capture có hash, không phải pcap/full grammar. `13E1820→vf24/13E14C0` là wrapper gửi đã nối;
chưa phục hồi đủ codec movement/reliability/handshake/relay hoặc cơ chế kiểm vị trí phía nhận.

**Bom hai phía:** hai run`duo-bombs`/`duo-confirm` có bốn request118 thực, owner/serial lần lượt
`(0,0),(0,28),(1,1),(1,29)`. Không mặc định serial bắt đầu0 hoặc dùng chung giữa hai owner.
Stub tại CP027 echo serial u32/owner/hàng/cột và field6=1 sang159, cấp **ID chung1–4**.
Field6 khi đó còn là power candidate; CP031 đã sửa nguồn power thành field10 sau Fire, xem
[contract serial](bombs.md#bomb-serial) và [correction](items-stats.md#item-effects). Các bom CP027 đều power1, chưa phân biệt hai field.
broadcast159 rồi158/kind0 tới cả hai socket bằng seq riêng. Mỗi phía đọc4×39/39 và4×13/13/errors0,
trả4entity khác0, lookup IDs1/2/3/4 và4kind0/cleanup. Ảnh B có bomA, ảnh A có bomB.
Allocator thử và fuse4s/args/fieldzero là **emu_choice/design**, không phải luật server gốc hoặc
proof mọi duplicate/stale/race/chain/item/trap. Chỉ đặt lần lượt, chưa stress đồng thời/hai round.

`duo-bombs` bị timeout chụp ảnh cuối: runerror được giữ, metadata sender/fixture và callbacks đã có;
không gọi toàn run đó pass. `duo-confirm` sửa recorder lưu metadata trước cleanup, ảnh lỗi ghi riêng,
và thêm ảnh trễ sau Space/trigger158: run hoàn tất không lỗi, cùng4bom trên cả hai phía.
Tên ảnh`blast-N` chỉ thời điểm trigger snapshot sau emit158; callback GDB mới chứng minh thực thi,
không suy ảnh luôn bắt đúng frame nước nổ. Control/runtime cũ giữ nguyên hash.

**Giới hạn tại CP027:** chưa hai trận đầy đủ, đồng thuận trap/death/result/item, join/ready chuẩn,
reset/disconnect, LAN hoặc Internet. Authority matrix dưới đây còn phần validator/broadcast/peer-editable
chưa đủ. Bước tiếp chỉ giữ ở [RESEARCH](../RESEARCH.md#q-023); không tự đóng K4/K5/K6 từ mốc này.

<a id="current-peer-targets"></a>
### Native UDP áp tọa độ đích trên currentservice — CP098

[Nguồn/recipe/hash/corrections](../evidence/current-peer-targets/README.md).
Snapshot CP098; gap R2 và nhánh distance lớn được bổ sung ở [CP106](#current-movement-cycles), không sửa raw cũ.
**kr_runtime/confirmed hẹp** currentmodifiedWine/AB/ExVillage01/localstub/packet13selector0. Reuse
CP027/045–047E66680/key/body/52A350, CP070transport, CP097profile/proxy/two-cycle và CP096neutralinput.
Không quét grammar; finite internalbreakpoints chỉ bật trong OSinputwindows, tối đa64entry/96ingress
mỗi phase. NativeAPI13DE7AA capture17byte; payload `[3:-1]` khớp exactnativepacket13 tạiE66680;
keycompareE667BE/slot/actorpointer và applyE668AD→B4E6F0→E668B2 được nối cùng nativecall.

Run `apply`305,509s có hai currentcycles nhưng assertion “current4C/58 bằng packetargs ngay sau apply”
bị runtime bác bỏ: state đổi ngay trong một số call, tọa độ tiến dần trong các callback sau. Đây không
là packet không apply. CP046oldpayloadstate0/delta80snap không thành luật tức thời cho ordinarymovement.
R1hai chiều có changedstate/nativebody/uniqueingressjoin; R2-A-Down thiếu transition đầu nên ghi partial.

RE tĩnh có mục tiêu đọc thêm B4E6F0 VA`B4E6F0`/RVA`74E6F0`; bytes0x510 khớp static/runtimecopy/modO.
`B4E9E5…EA04` ghi argsx/y vào protected actor`+7BC/+7C8` qua55FD30; 529C60 wrapper gọi52A350.
`B4EA14` so `abs(x−currentX)+abs(y−currentY)` với `10×property` đọc từcontrolleractor+584/virtual38.
Nhánh lớn gọiactorvirtual+74với x/y, semantics/runtime branch lớn chưa đo trong đợt này; nhánh nhỏ có
state/direction/`+7EC` checks. Không gọi property làspeed/ms, không đặt thời gian hay luật interpolation.

Run `target`302,534s đọc thêm protected7BC/7C8/7EC và actualcomparison; 281relations,168copiednative
securegetters,20actualingressnativeCRC checks. R1B→A có xarg140/current120, delta20<threshold40,
`+7BC:120→140` trong call, `+4C` giữ120; R1A→B có yarg229/current224,delta5<threshold50,
`+7C8:179→229`, `+58` giữ224. Đây là nativearg/branch/targetstorage proof trên cùng actor, không
chỉ pairedpositions. Có8nonzeroerrorbelowthreshold examples; zero observedlarge/snapbranch.
Currentcoords tiến dần ở callback sau; nguyên lý/timeconstant/correctiontoànmotion vẫn chưa đủ.
R2B→A có changedtarget; R2A→B chỉ settledtarget/currentsample, không changedtarget/progress proof.

Cùng run vẫn14native159/11native158/view/48native195flags/processkhớpservice,12same-oldTCPguards
được từ chối/noextraID165170, hai175/180/refresh. R2secondextra sauownsecondarydeadline0,769238847s:
CP094earlyadmission inheritedassertion fail được giữ; peerquestion không dùng run này chứng minh
hai extras đều trước deadline. CP097earlyadmission positive giữ nguồn riêng. Jobs7/13/14round_closed/
no158/noterraincommit. Không chuyển những callback bịfence thành nativeprocess.

**Blocker quan sát:** R1pre175 AviewA state56 tại(80,159), BviewA state0 cùngtọađộ; Bactorstate1 cảhai.
R2Astate0/Bstate1/cùngtọađộ. `RANK175_BEGIN` cũ không có monotonicns, mỗi view chụp khi chính nó nhận175,
không phải snapshots cùngtick. **Correction AUD029/CP099:** activeguardservice không logA17, nhưng raw nativefront thực cóA17 sau
first175send1,596361ms; báo cáo cũ lọcservice_entry đã bỏ frame không activeadmission.
[Audit](../evidence/checkpoints/audits/AUD029.json). B17/19/current+replays đủ. Managerstorage
rowtypes1/1rank1/2 trướcreturn cảhai vòng/view. VerifierAstate0 fail được giữ; items/storagepass chỉ
chứng minh BombFire/reset/Bdeath/storage, không consistencyA. Chưa gán persistentdesync, rank sai,
report bị mất hay nguyên nhân cho observer. Bước tại CP098 (lịch sử) là nativeAtrap/report/
servicefinish/rank/175entry chronology; thứ tự hiện hành chỉ ở [PLAN](../PLAN.md#current-order); không dùng patch hoặc policy đổi để làm state nhất quán giả.

Earlierapply preparation metadata pin-update failedaftercodecompiled/shelllaunch; sourceboundary ghi
postlaunch. Targetrepeat code/profile/GDB/prefix/role/sourcepins đã chuẩn bị/kiểm trướclaunch. Không
binarypackDBsource/statewrite/gamecall. Fullauth/codec/location/distance/authority/fullvalidator/allreset/
UDPgrammar/reliability/NAT/vanilla/LANInternet/fullmatch/K5/K6a vẫn mở; AUD028/nextCP100.


<a id="current-udp-causality"></a>
### UDP và vị trí hai chiều: can thiệp trên service hiện hành — CP070

**kr_runtime/confirmed hẹp trên Wine diagnostic copy**, [recipe/raw/hash](../evidence/current-udp-causality/README.md).
Tái dùng actual99→153, native wrapper/getter và cached1256B3B; không sửa service, client, key hoặc
payload. Trong namespace private chỉ loopback, input hook drop chính xác cặp cổng UDP thực, hai
hướng đo tuần tự; counter dương và restore/delete được ghi. Profile CP065 distinct/floor, một round.

A→B bị drop34gói/1518byte: ownA(120,159)→(120,279), viewB của A giữ(120,159).
B→A bị drop37gói/1665byte: ownB(120,399)→(172,399), viewA của B giữ(120,399).
Sau restore, trước input mới, remote đã thấy vị trí cuối cut; sau input mới hai view khớp A(120,339),
B(200,399). Các mẫu là native state/coordinates đọc bằng secure getter, không kết luận từ ảnh hoặc
absence callback. Runtime đã tách paired-position agreement với phụ thuộc UDP trong cấu hình này.

Counter bao gồm toàn luồng UDP khớp rule, không định danh mỗi packet movement. Restore trước input
mới chưa phân biệt periodic send/retransmit/queue; hostclock không là clienttimebase/same-tick proof.
Run v2 counter dương nhưng thiếu paired phases giữ partial, v3 tăng sampling mới đủ phép kiểm.
Lỗi assert metadata của run đầu xảy ra trước nft, không quy cho client. Service/profile/guards giữ;
không bom/result/reset/2cycles ở phép này, không full UDP grammar/reliability/NAT/auth/fallback/LAN/
Internet/vanilla/K5/K6a. [OldUDP/key](#udp-generation) giữ nguồn CP045–047 riêng.

<a id="current-movement-cycles"></a>
### Movement/terrain/ending trong cùng hai vòng hiện hành — CP106

[Recipe/hash/phép kiểm](../evidence/current-movement-cycles/README.md); gap và phép đo tiếp ở
[RESEARCH](../RESEARCH.md#gap-peer-movement), thứ tự chỉ ở [PLAN](../PLAN.md#current-order).
**kr_runtime/confirmed hẹp**: modified Wine copy A/B, ExVillage01, local stub/current drained profile.
Reuse CP098 API13/key/getter/B4E6F0/B4EA14, CP103 fixed-role proxy/route, CP104–105 đúng ending phase.
Không đọc thân hàm mới, đổi production/profile/timer/packet hoặc ép state. VN chỉ tái dùng khái niệm
P2P/tách session-membership, không port opcode/layout/offset.

Observer host ghi ack sau sáu breakpoint thực đã bật; controller đợi ack và mẫu live mới ở cảhai view
trước pulse OS0,25s. Window giữ thêm tối đa3s sau input. Ack không là gói game hoặc đồng bộ gametick.

| Pha | Native apply ở recipient | Unique API join | Target đổi sau input | Current XY qua callback |
|---|---:|---:|---:|---|
| R1 A-Down→B |5|5|2| (120,159)→(120,214)→(120,284)→(120,274) |
| R1 B-Right→A |6|6|2| (120,399)→(200,399) |
| R2 A-Down→B |4|4|2| (120,159)→(120,204)→(120,344)→(120,284) |
| R2 B-Right→A |7|7|2| (120,399)→(200,399) |

Mỗi target-change có captured API17byte, payload13/keycompare/slot/args, cùng actor trước/sau và
protected+7BC/+7C8 bằng args. Đây bổ sung gap R2A của CP098; không suy nguyên nhân gap cũ do observer.
Có3 nonzero-error dưới threshold mà currentXY chưa đổi ngay và3 lần distance≥threshold currentXY
bằng args sau call: 44/40, **40/40**, 60/50. Cached static comparison phù hợp; chưa đo riêng callee
virtual74 hay toàn luật/timeconstant interpolation. Các sample không đơn điệu, không suy đường
đi/tốc độ chỉ từ hold hoặc các điểm thưa. Getter/CRC emulation và input timestamp giữ nguồn riêng.

Cùng run hai Start/clock/result175/return180/585/refresh:14actual118/159entities, **14actual158/view**,
60native map snapshots×195 flags khớp service prediction/process/boxdelta/reset seed; không job chưa
phát158 trong run này. Bốn Bomb/Fire pickups/stat1→2/baselineStart2; 8same/4old TCP replays bị loại,
không thêm ID/165/170. Đây là integration của bounded policy, không nguyên tắc server gốc.

Hai selected21 thực từ A được gửi prefix6 đến B; native reader6/errors0, gate1/calleeB43320(0,0,-1)
và56→26 cảhai vòng. CP103 có một gate-skip; distribution đó không áp cố định cho mọi run.
Entry175 vẫn có inequality; **ở native180 từng view**, A3/B1 và tọa độ khớp cảhai vòng
(R1 A80,159/B200,159; R2 A80,169/B200,159). Không simultaneous-globaltick/allfields proof hoặc luật
universal enum/winner. Phép kiểm này bổ sung qualification #166 theo correction CP104–105,
không sửa những assertionfail/raw lịch sử hoặc dùng same-entry175 làm toàn-ending criterion.

557 movement relations/360copied getters/44actualCRC checks;152integration/247transport/39item/
60drained/33peerhandler/59ending relations; emulation11.820mapgetters/11.700arrayaddresses riêng.
Analysis KeyError do nhầm path substitution được giữ và sửa bằng script riêng, không sửa raw/native.
CP106 đã merge PR173 tại `f67a160`; CP107/PR175 merge `ac6562e` bổ sung [current hostBack](room-membership.md#host-normal-back).
Bước host Back nêu tại CP106 là lịch sử; thứ tự hiện hành chỉ ởPLAN. Fullauth/codec/location/validator/
authority/general membership/allreset/vanilla/fullmatch/K5/K6a/LANInternet vẫn mở; cadence ởmanifest.


<a id="virtual-movement-cycles"></a>
### Movement hai chiều, item và hai lifecycle trên production virtualhosts — CP142

**kr_runtime/confirmed hẹp**, [raw/hash/recipe](../evidence/peer-endpoints/README.md#cp142).
Reuse finite observer/ADown-BRight/OS returnsetup CP139 và production CLI/profile/basic lifecycle
CP141. Run mới có bốn cửa sổ (hai hướng × hai vòng), actual CA+wineserver/namespace được kiểm trước
input, native ingress→key/slot→actor apply/protected target→fresh current position. Packet selector0
vẫn13byte; key byte so theo **low8(roundkey) XOR low8(profileID(slot))** đã phục hồi CP047, không
phải low8key đơn lẻ và không là auth. Có632 runtime joins và1356 source/wire/ownership joins;
26 recipient applies,8 target changes, cả bốn current progress trong/giữa calls.45 actual API CRC và
408 secure getters là emulation riêng.590 veth PCAP records, UDP direct, không shim/UDPproxy.

ADown thực tế nhặt Bomb index16/kind0 và Fire17/kind1 ở cả hai vòng. Actual124/currentkey/winnerA
→ native165full12/errors0 ở cả hai view; capacity/power1,1→2,1→2,2, clock2 trở lại1,1. B giữ baseline.
Bốn request/eight view-item joins/70 checks;1314 actor copied-getter checks tách riêng. Không phải
tranh item/fairness/location validator/fullinventory/max/TTL. Feedback snapshot giữ counterevent cũ;
chỉ GEN165_BEGIN/END exact context được dùng cho effect, không đếm mọi event/phase thành handler.

Cùng run có hai actual A118 tại3,3/power2 sauFire, IDs1→2/keys12345→12346, currentA17→19 bind
exactfreshtrapSHA, peer170/B21 dẫn chosen survivor rank2,1/native175302/1803/110 ở cả hai view;
195flags before/after/seedreset và clockstatbaseline được kiểm. Barrier25,901/25,755s<30s; không
suy chắc chắn mọi coldstart sẽ trước deadline. Normal ReadyB/StartA giữa vòng, normal BReady sau
return2 tạo actual95seq457/SHA`d028d7997f2fccd885e2a490528f7dcf1d13dd6382f4574b6588b0bc25da75c7`;
additional native110full6/errors0 cảhai trướccleanup107, render phòng/readybadge; không Start3.

Run r1 trước input dừng do composition harness bỏ mapping GDB B→roleB/pathB. Hai process cùng
outputA khiến thiếu ackB; không dùng đó làm lỗi gameplay/network.691raw giữ nguyên, r2 dùng mapping
đã có CP139 và output/prefix mới. Các draft checker thiếu XOR identity hoặc chọn snapshot lẫn feedback
đều giữ failedscript/log/correction; không có rawrewrite hoặc nativeRE mới chỉ để vượt check.

Đây là cùng-run movement/item/basic2ending/reset/Ready **trên modifiedWine và hai virtualhost cùng
máy**, chưa fullmatch/K5/K6a/physicalLAN/K6b/Internet/K6c. Chưa selected malformed/stale TCP trong
run mới hoặc disconnect giữa trận, không tự đóng consistency/authority/gate. Không giải general UDP
reliability/NAT/fullgrammar/fullinventory/CP137complexUI/sharedgeometry. IP1/subtype39 opaque;
rank/fuse/drop/address là emulator policy. Bước tiếp duy nhất ở PLAN; AUD043qua140/kếCP144.


<a id="basic-bidirectional"></a>
### Basic: hai owner bom, movement hai chiều và hai vòng — CP173

**`kr_runtime/confirmed` hẹp, controller/terrain/claims/fuse/rank là `design`.**
[Recipe/raw/hash và partial](../evidence/peer-endpoints/README.md#cp173). Source-first CP155/160/168
native peer windows/B118, CP170–172 exactbasicroute/probes/195/reset/normalBStart3; không nativebody/
opcode/ItemAttr/VNlayout RE mới, production/profile/guard giữ nguyên sau PR244.

R1 thêm ADown/BRight finitewindows rồi BSpace/Rightescape/return trước routeA:4nativepeerwindows
đạt651checks/420gettermodels vàBbomb hai epoch/native15939/15813/samepointer. Nhưng Roller được
nhặt26,60/26,87s trước firstAbomb, vòng2 A ở80,399/Up bị chặn và20pulse không tới159; chỉ1outcome,
thiếuFire/outcome2/Start3. Earlyspeed là giả thuyết gây lệch coarse route, không universal causal proof.
R2 chỉ đổi AUp: hai cycles/basic/reset/Start3 đạt1728firsttwo/560Start3checks, nhưng nativeAUp không
changedtarget trước hộp trên spawn nên endpoint progress assertion FAIL. Không coi blocked input
là movement pass hoặc ghép R1peer/R2cycles thành acceptance. Raw/failures giữ nguyên.

R3 đổi riêng Awindow sang Right/X120/LeftRight return trên row3col3..5floor, BRight row9 giữ;
noearlyRoller/noactorwrite/noGamecall/nofakepacket/noboundsrelaxation. Mỗi key bốn bước:
hai native windows A→B/B→A ngoài observerdisable → B118 tại9,3/power1/Rightescape8s/return120,399
→ knownAroute/3pickups/2Abombs/normaloutcome. Bpower1 không tới ô5 trong hành lang floor3..5.
Các tọa độ là lựa chọn controller trên map đã pin, không motionlaw gốc.

Final same8TCP/currentManner/title/promotedBhost, ba keys12345/12346/12347. Hai epoch mỗi epoch
3bom: B rồi A,A, IDs1..6/powers[1,1,2] mỗi vòng. Native159/158/currentID/samepointer/full39/full13,
195before/after tính tuần tự Bclear rồi hai A blasts và seedreset; không lấy virginworld của runA-only.
Ba pickups17/27/28 và native165/effects chỉA, baseline/stat/hiddeneligibility/currentcell142 reset,
A17→19/B21 và hai view56→1/other0 trước175,2normal175180/room/Ready. NormalBStart3 actual104/
12199/barrier trước30s/hai clocks thêm8s+/hai final Battle images. Không thirdbomb/outcome/EOFfault.

Fourfinitepeerwindows có explicit6probeACK mỗi view/freshbaseline trướcOSinput; actualingress17→
selector0/full13/key/slot→nativeapply/target7BC7C8change và freshcurrent4C58progress. Native distance
branches/getters độc lập; không buộc immediate snap khi consumer nội suy. Window direction/axis
theo actualRight của A/B, không đổi criterion cho blocked Up. Observerlight chỉ căn vị trí, restore
trước windows/bom; rare165+lookup giữ. Kết quả/count/hash chi tiết ở evidence, không native timinglaw.

Khoảng trốngBowner/bidirectional basic đã giải trong R3, không backfill hai partial. Gate/dependencies/
K5/K6a/fullmatch/Windows/LAN/Internet/8clients/auth/generalreset vẫn mở. Tiếp[PLAN](../PLAN.md#current-order)
currentbasic BhostEOF với pendingBbomb/cleanupA/promotion/fence; CP155156controls là nguồnreuse,
không lấy intentional endcleanup của R3 làm proof.142sameoldbytesafterlegitclear/fairness giữ giới hạn.


<a id="udp-generation"></a>
### UDP qua vòng mới: queue, drain và consumer game — CP045

**kr_runtime/kr_static confirmed hẹp** trên cùng diagnostic copy, không thêm patch/ép actor.
[Nguồn đã đối chiếu, recipe/hash và các ca chưa đạt](../evidence/udp-generation/README.md).
Tái dùng CP027 endpoint/actual99→153/nativeCRC và CP039–044 lifecycle; VN92/146 chỉ hướng peer
exchange/P2P, chưa có proof stale UDP. `boom-rs` ADR0009 là wire mới, không áp epoch của nó vào KR.

`ingress-rounds` giữ hai client/hai Start/clock/result/return trên cùng8connections; breakpoint sau
recvfrom, trước indirectcall **13DE7AA** ghi24datagram/client/round, tổng96sample.
Receiver vtable17A2BB0, callback thật **13DE880** cả hai vòng; receiver cùng pointer trong mỗi client
không chứng minh mọi state/queue giữ nguyên hoặc đã reset. Đây là actualAPIcapture, không pcap toàn luồng.

**Đường native có mục tiêu:**13DE880 lookup peer theo IP/u16port, thiếu thì thử14E57B0; kiểm fragment
length bằng14A58A0 và CRC14A5860(seed0 trên toàn frame) bằng0 rồi gọi **14E7340**.
Wrapper cộng peer+140 và truyền argument đầu **hằng0** vào **14E8280**.
Nhánh buffer kiểm pointer/length1..255, giữ lock, ghi length4byte +payload vào buffer+80,
cập nhật count+2080; guard argument đầu chỉ so khi khác0. Không gọi argument này là round hoặc
wireseq chỉ vì method có phép so; wrapper constant0 không proof toàn UDP không epoch.

**Drain:**14E8480 đọc record đầu, copy tối đa outputcapacity, dịch phần còn lại và cập nhật count;
directcall14E5742 nằm trong vòng duyệt peer, return14E5747.14E83A0 có clear count0 theo static;
chưa thấy runtime caller của clear ở roundboundary, không coi đã chứng minh reset từ body đó.

Proxy thử chỉ đổi endpoint153 sang hai socket loopback sở hữu bởi harness; forward nguyên datagram
đến UDPport thực từ99, dùng sourceport đúng endpoint của origin đã khai báo. Đây là policy thí nghiệm,
không productionrelay/NAT. `proxy-control` có159/entity/158kind0 nhưng A không17/19 hoặc result,
giữ làm ca chưa đạt vòng2. `centered-control` lỗi GDB `Invalid type combination in equality test`
tại14E85B7 do conveniencevar drain chưa khởi tạo; không lỗi wire/client. `centered-fixed` khởi tạo
vars và bỏ ADown trước Space để giữ ởspawn: hai bom/trap/death/statepeer1 trước175, hai175/180/phòng.
Không khẳng định nhân quả của partialrun chỉ từ thay input/probe; giữ tất cả raw.

Queueappend/drain đã quan sát trong control: payload15byte tăng count0→19, draincopy15 rồi count0;
payload3byte tăng0→7. Mẫu có nhiều record thì count khác0, không ép luôn0→19.
Admission/drain không tương đương actor đã áp dụng.

**Gói cũ nguyên vẹn:** `stale-round2` lấy actual B→A datagram17byte vòng1, replay một lần sau153
cả hai phía vòng2/trước input mới, cùng advertised sourceport. Wire SHA
`bbd38001fff30521da221b5c67a5b2389e3d95abab160cb85e376d9a063f4058`, inner15byte
`010100003b000132c8009f0000004b`; không có gói forward bình thường vòng2 cùng wirehash.
Round2/sample10: queue61FE6E00 count26→45, drain cùng payload/count57→38. Đây là bằng chứng
runtime gói cũ đã được nhận và lấy khỏi queue, không chỉ sendlog.

**Consumer game:** stack-return thật14E5747 và backtrace giới hạn dẫn tới13E0E3F trong13E0E00.
Prefix tĩnh đọc selector; selector1 đọc slot0..25. Gate từE8E960 điều khiển nhánh đọc4byte/
so với securefield tại context+18D4+slot×C: compare13E0F96, windowdrop13E0FB6,
store13E0FD7. Chưa đặt tên field là tick/key/epoch chỉ vì có phép so. Body clear chưa chứng minh
roundboundary; prefix consumer chưa phải toàn parser/downstream.

`stale-game-guard` chỉ tới vòng1/A17, không19/vòng2/replay; giữ partial, instrumentation overhead
là giả thuyết. Run kế `stale-game-lite` chỉ bật probe game từ Start2, tắt sau64 message/client:
actual old B→A wire SHA `dc261c4b21730298f962a78cc6cbe26fe64c2034c3828bd2feb6d9f7e8715246`,
inner `010100003b00013ac8009f00000048`, không trùng normalforward vòng2. Round2/sample6:
queue5C303B80 count19→38, drain45→26, gameconsumer nhận đúng inner15byte,
**slot1/gate0, return13**; không vào compare/windowdrop/store branch đã đặt probe.
Đó là giá trị return quan sát được, chưa chứng minh toàn13byte output hoặc actor nhận/apply.
Run này vẫn có hai17/19/statepeer1 trước175 và hai175/180/phòng; outcome/fuse là policy stub.

Tại CP045 chưa suy gói cũ sửa actor từ ảnh spawn/tutorial, queue hoặc return13. CP046 kiểm tiếp
downstream và actor dưới đây; round identity/grammar đầy đủ vẫn mở. Không tự đóng validator/reset/K6a.

**CP046 — output và actor, kr_static/kr_runtime confirmed hẹp:**
[Nguồn đối chiếu, recipe, control/lỗi probe và hash](../evidence/udp-actor/README.md).
Không RE lại endpoint/CRC/queue: dùng CP045, cache sender13E14C0 và VN P2P làm phương pháp;
nguồn cũ không chứng minh guard KR. Không thêm patch/ép state; cùng diagnostic copy Wine.

- `E8E960` tại build này trả AL0 vô điều kiện; 13byte machinecode khớp static/runtimeA/runtimeB/modO.
  Gate0 của CP045 có lời giải tĩnh, không còn cần tìm input bật nó. Không suy toàn wire không epoch.
- Tail13E0E00 dưới gate0 chép payload từ buffer+2 về đầu, trả length−2. Runtime caller thật125934B;
  output13byte khớp inner15byte đã bỏ selector/slot. `125937E→14EF010` là khởi tạo packet stream,
  không phải callback gameplay. Bước này chưa đổi field actor trong ca đã đo.
- Pump ngoài tại **1257B61** gọi virtual+B4, runtime target **125DEF0**, receiver vt17927FC;
  wrapper này chuyển tới receiver+F0/vfF4. Tại CP046 target vfF4 sâu hơn chưa được đo; CP047 bổ sung bên dưới.
  Nhánh130BC00 trong caller trung gian không hit ở control, không gọi nó là handler actor.
- `pump-stale`, vòng2/sample6: exact old B→A datagram17byte vòng1, wire SHA
  `2bee9f8cf7d5f12d3cbec10921e8d1c4e66ac1a10c1bf320b0b772417d9f0f9d`, không trùng normalforwardR2.
  Inner15byte `010100003b00015cc8009f00000043` → output/packet stream/callback13byte
  `00003b00015cc8009f00000043`. Cùng callback, cùng pointer actor slot1: secure **+4C/x120→200**,
  **+58/y159→159**, **+28/state15→0**; checksum và getter native52A350 xác nhận trước/sau.
  Cursor packet0→13. Đây là actor thực sự đổi field trong lúc xử lý oldpayload, vượt bằng chứng queue.
- Normalforward vòng2/sample8 kế tiếp làm **x200→120**, y159/state0 giữ nguyên. Return callback
  old=1/current=0 nhưng cả hai đổi tọa độ; không gọi0 là reject hoặc1 là accept từ quan sát này.
  Không gán tên tick/key/epoch cho field payload chỉ vì trông giống counter.

Control `pump-control` không replay, cùng24-sample probe/input (harness chỉ khác mode), có8callback
normalB đã match actualforward trước input vòng2: x120/y159 giữ nguyên trước/sau. Control có hai
bom/17/19/statepeer/result/return. Oldrun có hai Start/bom nhưng vòng2 chỉ17, không19/result; không
quy việc thiếu19 chỉ do replay và không ghép hai run thành ca fullmatch.
Getter oracle kiểm value/checksum native, mô hình riêng counter5000/55-read để bỏ periodic callback/
rekey/storage trong emulation; không đổi counter/state của client runtime. [Giới hạn phép kiểm](../evidence/udp-actor/README.md).
Giới hạn: một oldpacket/cùng endpoint/profile; chưa toàn peer grammar, freshness validator, loại gói
cũ theo generation hoặc giải pháp production. [Phép thử kế](../RESEARCH.md#q-023).

**CP047 — handler, byte khóa và bộ lọc ngắn, kr_static/kr_runtime confirmed hẹp:**
[Nguồn/cache đã tái dùng, recipe/hash và các mức kiểm](../evidence/udp-freshness/README.md).
Runtime125DF30 xác định vfF4 thật **E66680/vt176A470** trên cả hai client. Cache CP025 đã có body,
không quét lại client. Chỉ nhánh selector0/length13 đang dùng cho ca movement được phục hồi ở đây;
E66680 còn các nhánh khác, không lấy một nhánh làm toàn grammar.

| Phần payload13byte | Đọc/consumer đã đối chiếu |
|---|---|
| u16 selector | Nhánh0, tổng packetlength phải13. |
| u8 khóa | So tạiE667BE với `low8(stage+63E0) XOR low8(profileID(slot))`, profile getter1240370 đã có ở CP020. Không là xác thực hoặc u32 epoch đầy đủ. |
| u8 context | So với E150F0 trên context lấy từE703E0; chưa gán tên/đơn vị. |
| body8byte | byte0 chọn slot; byte2–3/4–5 là signed16 x/y ở call B4E6F0; byte6 đi vào state argument. Byte1/7 chưa giải nghĩa. |
| u8 cuối | So với byte tại stage+63EC+slot, rồi lưu trước apply; chưa gọi là clock/tick/time. |

CacheE83900 cho thấy init153 đặt24byte từ+63EC thànhFF; không suy đủ bounds26slot từ peerlist26.
Ở E66868–E66892, nếu incoming>stored thì store/apply; nếu incoming≤stored nhưng stored−incoming≥10
vẫn store/apply. Chỉ đoạn lùi0..9 bị nhánh E66884 bỏ. Vì vậy byte này không tự chống packet vòng cũ.
Mười ca emulation chạy đúng code compare/jump với context tổng hợp (biên9/10/wrap), tách với runtime.

- `handler-stale`: giữ khóa12345/cùng endpoint, exactold callback vẫn đổi x120→200; handler target
  E66680 được quan sát. Hai Start/bom/17/19/statepeer/result/return, không phải nghiệm thu toàn trận.
- `key-rotate`: **chỉ u32 thứ hai trong153 đổi12345→12346 ở vòng2**, cả hai client cùng khóa;
  endpoint và oldUDPbytes/CRC giữ nguyên. Decoder native153 đọc97/97/errors0, khóa đúng từng vòng.
  Exactold B→A wire SHA `12f19edc8462717b528f36ece3a4d5606bd4b128c78aeb3551628c44547b8291`
  vẫn qua endpoint/CRC/queue/drain/consumer/packetstream/E66680, round2/sample4 đọc đủ13byte;
  key wire59/expected56. Không tới tailcompare/B4E6F0; cùng actor giữ state15/x120/y159 trước/sau.
  Normalpacket key56 khớp và tới apply; có hai bom/17/19/statepeer/result/return. Đây là native guard
  cho packet đã đo, không suy sendlog hoặc return callback thành accept/reject.
- `rotate-old`:153 đổi cả hai proxyport vòng2 nhưng khóa vẫn12345; giữ socket cũ, replay trực tiếp
  tới client từ cổng B cũ60880, không đi qua bộ lọc generation của proxy. Native resolve peer khác0;
  exactold round2/sample4 key59 khớp, tail incoming69/stored86 (lùi17) vẫn store/apply,
  actor x120→200/y159 giữ nguyên. Có hai result/return. **Đổi endpoint riêng lẻ không chặn ca này.**
- `rotate-new`: oldpayload từ cổng mới47725 vẫn vào E66680; round2/sample4 key59 khớp,
  incoming70/stored86 (lùi16) store/apply, x120→200/y159 giữ nguyên. Hai result/return.
  Cả hai ca đổi cổng đều kiểm native endpoint/queue/actor, không dùng log proxy làm oracle.

Sáu ca emulation byte-key/XOR/compare gồm khóa12345 và12601 (`+256`) cho cùng byte kiểm;
không nâng sáu ca thành runtime wrap/security. Bộ cấp [UdpRoundKeys](../../server/udp_round_keys.py)
là `design/emu_choice`: cấp một u32 mỗi acceptedStart, cùng khóa cho mọi member, không lặp low8
trong một lifetime, báo lỗi sau256 lần; caller giữ instance qua finish/abort và quản lifetime/roster.
Unit test không là runtime tích hợp module; các harness trên chọn12345/12346 trực tiếp. Roster/ID thay,
key reuse, packet type khác, giả mạo và peer lifetime mới vẫn cần xử lý riêng; không gọi khóa này là auth.

<a id="udp-membership-lifetime"></a>
### Exact old UDP sau host remove107/rejoin — CP130

**kr_runtime/confirmed hẹp**, [metadata/hash/recipe](../evidence/udp-generation/README.md#cp130).
Reuse CP045queue/drain, CP047–048key branch, CP070direct endpoints, CP105API→consumer/securegetters
và CP129 hostexit/rejoin/currentservice. Không nativebody mới, không payload/CRC/key/state rewrite.
Raw IPv4/UDP transport chỉ loopback/owned endpoints trong private namespace, preflight synthetic;
native ingress/hash sau đó mới chứng minh client nhận. Source/profile/client/fuse giữ nguyên tới cleanup.

R3 actual R1A→B17byte `10010000003800006178009f0018000475`, SHA256
`cfca3fda346c8878609c87b8173b8c3ad7329e471781da06a10da59037e93bf1`;
replay nguyên payload từ actualR1A99 tới actualB99, kiểm port với archived`previous_membership.init99`.
Sau Bnative107clear ID1→0/chief1/Prepare, trướcArejoin: exacthash ingress13DE7AA/endpoint13DE949peer≠0,
inner15 vào14E8280queue/+19byte,14E8480drain/−19byte; **không exact E66680 consumer trong cửa sổ**.
Không suy queue đượcclear khi release membership hoặc endpoint đãbịhủy từ chỉ IDclear.

Sau Arejoin/Ready/BStart2/key12346, cùnghash lại ingress→endpoint→queue/drain→E66680entry/
E667BEcompare wire**56**/expected**59** → **E667C2key-drop**, khôngE668ADapply cho exactpayload.
Gói R2current cùng phase vẫn có13apply, phase positive sau đó31apply/fullcursor13; R1positive29apply.
Đây là mỏ neo gate key ở consumer, không identity/auth/wholeepoch: low8 reuse256/mode/roster-ID khác/
source endpoint mới và toànUDPgrammar chưa đủ. R1Right có thểbịblockởx120, không gán displacement
cho mọi callback; positive là native currentpacket apply/fullread, wrapper kiểm độc lập.

R3299,981878580s/exit0/errors0/probeerrors0/1112sealedraw/1226checks/852copiednative52A350wrappers,
135packet+12peer153 cases riêng; cùng8TCP/two121clocks/key12345→12346/oneforfeit+R2active.
Mỗi cửa sổ chỉ bật12knownreadonlyprobes sau **Barmedack**, cap128/kind/phase; ack là observerready,
không làreceipt. R1collectorKeyErrorinit99 sau membershiprelease trướcreplay giữ807raw/138getters.
R2sửa archivedport,256,2825s/1146raw/336getters/2sends nhưng khôngexactoldingress vì thiếuarmedproof;
không gọi làdrop. R3thêmack/window8s, giữ source/profile/payloadcontract; không khẳngđịnh mộtnguyênnhân
duynhất cho R2miss. Draftanalyzer slicing sửa riêng raw[1:-1]15 và[3:-1]13 theo actualnative bytes;
oldreport giữ, runtime/raw không sửa. Generalqueue/nativepool/fullmatch/K5/K6a/LANInternet vẫn mở.
Bước tiếp chỉPLAN; AUD040/kế132, reviewwaived khôngđónggate.


<a id="virtual-exact-old-udp"></a>
### Exact old UDP từ cùng endpoint trên direct virtualhosts — CP145

**kr_runtime/confirmed hẹp; AF_PACKET replay/offload policy là design.**
[Recipe/raw/hash/control](../evidence/peer-endpoints/README.md#cp145). Reuse CP047 key XORprofileID,
CP13012known readonly ingress/endpoint/queue/drain/consumer/drop probes và CP143–144 production
virtualprofiles. Không nativebody mới/port opcode/layout VN, không mở toànUDPgrammar.

R1 đã chạy hai normalcycles nhưng **không đạt mục tiêu key-drop cũ**: rootkv-a lưu frame trước
checksumoffload, bytes IP checksum hợp lệ/UDPchecksum không hợp lệ, exact replay không có exactnative
oldingress. Không suy KRdrop từ im lặng. Owned isolated-veth control với payload tự sinh: default
receive1/originalonly/checksumresidue≠0; txrxoff nhận2/capturedframechecksum0. Control chứng minh lỗi
bytes-only replay trong topology này, không nói mạng gốc làm mất UDP hoặc biến model thành KRproof.

R2 mới tắt tx/rx offload **trên bốn ownedveth trướcclientlaunch**, dùng rawsend socket riêng để capture
thấy cả original vàreplay. Source/profile/client/tuple/CRC/payload giữ, frame checksum tự hợp lệ,
không sửa header hoặc gamebytes. ActualR1B→AEthernet59byte/UDP17byte/SHA có hai exactPCAPcopies:
originalvòng1 và AF_PACKETinject sau153/clock2/currentkey. Đây không nativeB emission lầnhai.

A12probesarmed trướcsend; exactnativeingress source10.77.0.3/actual99port→registeredpeer≠0→inner15
vàoqueue tăng19/drain giảm19→selector0consumer13. NativeE667BE so **wire59/expected56**, slot1,
round2→**E667C2key-drop**, không exactE668ADapply/after. MAC/IP/port/UDPchecksum/KRCRC/payload
đều không rewrite. Cùng phase có13 currentnormal full13 apply; cửa sổ BRight sau đó có target/current
progress. Không gọi việc thiếu displacement là negativeoracle; keybranch đã được quan sát.

Cùng R2 có4normalmovementwindows/BombFireeffect/resetbothviews/8same4oldTCPguard/twoactualA118
power2/current17→19hash/survivor175180110/195keyIDreset/finalnormalReady95native110/render; không
Start3/guestEOF trong run này. Barrier25,603/25,038s<30 là run riêng, không sửa marginCP144 thành
stabilityfix. R1partial/control/R2 được giữ riêng; AUD044through144/next148.

Scope chỉ IDs1/2, keys12345→12346, stableactual99sourceIP/port/two virtualhosts samephysicalhost.
Low8wrap256, forgedcurrentkey/auth, UDPtypemode/reliability/fullgrammar/NAT/realLAN/Internet/fullmatch/
K6a/gates/CP137complexUI/sharedgeometry/fullinventory vẫn mở. TCP vẫn dùng childloopbackforward và
fixturelocalredirect; native sourceIP crossvirtual proof không đồng nghĩa bootstrap/login trực tiếp
trên LAN. Bước tiếp duy nhất PLAN là gỡ dependency TCPforward bằng đối chiếu endpoint chain.


<a id="report127-freshness"></a>
### Admission report127 theo kết nối — CP063

[Recipe/hash/control](../evidence/report127-freshness/README.md), [giới hạn còn mở](../RESEARCH.md#gap-contact-state).
Tái dùng [writer/prefix14](life-results.md#round-result), [CMainSystem logicalcounter](items-stats.md#pickup-tick), nativeframer/
state/stat và currentprofile; không thêm thân hàm native hoặc patch. Đối chiếu64 report active ở
CP061–062: tick không giảm trên từng kết nối qua hai Start, có **cùngtick/khácbody/tọa độ**.
Native getter kiểm prefix14 cả các report này và subtype34 ngoài active; không giải toàn tail39/34.
Field u32 tại offset6 là tick; field cuối actor/v35C **không** là k124. Không áp milliseconds hay clock chung hai peer.

**Đối chứng runtime:** wrapper giữ nguyên exact frame/body17/19/21 do client thật gửi ở vòng1 rồi
replay tại dispatcher sau actual39 của cùng role ở vòng2. Baseline nhận cả ba; old17/19 đặt marker
vòng2/phát170. Trước175 vòng2 native view A đọc actorA state0, viewB đọc state1; không có actualbom/
actual19 vòng2. Duplicate19 cùng vòng cũng thay deathreport/phát170 lần nữa. Đây là can thiệp một
nhóm replay, chưa cô lập đóng góp từng subtype; không khẳng định old21 là nguyên nhân của rank.
**Dispatcher replay bỏ qua socket/framer/sequence admission**, không chứng minh actual TCP replay lọt.
Run đầu clientB dừng login local trước phép thử giữ riêng, không tính là đối chứng thành công.

**Policy mới là design của emulator**, không phục hồi server gốc: sau prefix/slot và supported trap
validation, giữ `report127_cursor` riêng trên channel connection, không xóa ở Start. Với tick cũ,
`delta=(tick−last)&FFFFFFFF`; delta>=80000000 bị loại, gồm half-range mơ hồ. Delta0 cho body khác
đi tiếp; đúng bodySHA đã nhận ở tick hiện tại bị loại, kể cả wire sequence khác. Tick tiến thì thay
nhóm bodySHA. Không lấy124t đã chốt ở pending record hoặc tick peer khác làm watermark.
Report bị loại vẫn log lý do, không sửa marker/death, không phát170 và không được dùng làm21 hoàn tất vòng.

Run guard cùng recipe: **3 old replay +6 exact duplicate bị loại**, tất cả report actual trong active
được nhận; hai actual124/165/Roller5→6/reset, hai bom/17/19→170/21, nativeAstate1 cảhai view trước175,
hai kết quả/return/roomrefresh. Không suy toàn reset/K5/K6a từ hai chu kỳ này.
Clock wrap/half-range và freshconnection chỉ regression policy, không runtime wrap/reconnect proof.
Guard không nhận biết old report lần đầu khi thiếu anchor, payload sửa/tick tăng giả, actor/distance/auth,
reset counter cùng socket hoặc khoảng cách>=nửa miềnu32. Originalvalidator/đơn vị thời gian/transport
replay admission còn unknown; không dùng reportguard như authority state. Bước tiếp ở [PLAN](../PLAN.md#current-order).

<a id="current-tcp-replay"></a>
### Body cũ/trùng qua TCP channel thực — CP075

[Run/recipe/provenance/hash và kiểm planned-partial](../evidence/current-tcp-replay/README.md),
**kr_runtime/confirmed hẹp**, productionservice/currentauthcache/basicprofile giữ nguyên.
Channelproxy forward S2C byteforbyte, trước injection C2S nguyênframe; sau capture mã hóa lại body
nguyên bằng sequence upstream mới và resequence actualframes. Front nativewriter và proxyreplay có
nguồn riêng; upstream TCPchunk join exact archivedframes. Không gọi replay là nativewriter mới.

12replay (8samebody/4oldbody) của118/124/127sub17,19 đều log admission/reject đúng guard:
118 duplicate_bomb_serial/bomb_round_key_mismatch,124 claim_not_current_or_already_consumed/
round_key_mismatch,127 duplicate_report_body/report_tick_not_forward. Oldbody gửi sau actualA39
vòng2, không sửa key/tick. Không thêm ID/job/165/170 từ replay. Bốn bomactualID1..4, năm pickupactual
R1Roller/Bomb vàR2Roller/Bomb/Fire, delta native cảhai view và baselineR2; R2 actual17/19→peerstateA1.

**Phạm vi tích hợp partial:** haiStart/clock, R1result/return/refresh, R2effect/deathstate; controller
hết360s trướcR2rank (4,549s saudeath, policyrank≥8s). Không đủ hai chu kỳ kết quả/reset cùngrun.
Verifier planned giữ failure, bounded verifier ghi scope riêng. Không hiddenpointerjoin mới vì thiếu
probe, không baeffects mỗi vòng. CP049/063/067 dispatchercontrols lịch sử không tự biến thànhTCPproof.
Chưa toàn127/21/34/forgedcurrentkey/freshserial/actorauth/distance/cap/fullvalidator/fullcodec/vanilla/
fullmatch/K5/K6a/LANInternet. Không causalfix lỗi proxy cũ; reuseauth/cache là điều kiện mới, không isolate.

<a id="tcp-guard-cycles"></a>
### Selected TCP guards cùng hai chu kỳ native — CP085

[Recipe/negative/raw/hash](../evidence/tcp-guard-cycles/README.md), **kr_runtime/confirmed hẹp** trên
modifiedWine/local/currentprofile. Reuse CP075proxy exactsource/front-upstreamseq/bodyjoin và
CP080–084 normalinput/BombFire/threebomb/state/all-dead/resultreset. VNprotocol/session chỉ checklist;
không RE lại codec/schema/chain hoặc đổi outcome/timing để pass. CP075 run cũ vẫn partial tại mốc đó.

Final `cleanup-fixed`: A nativefront158 frame→upstream170 (12replay), B146→146; sequence40+3n
riêng mỗi stream, S2C nguyên byte. **8same+4oldbody** đã capture118/124/12717,19, chỉ mã hóa lại outer
currentseq; không phải nativewriter phát frame mới hoặc gói cũ vượt crypto. Oldbody được gửi sau
actualA127/sub39 vòng2. Same118 bị `duplicate_bomb_serial`, old118 `bomb_round_key_mismatch` trước
ID/job; same124 `claim_not_current_or_already_consumed`, old124 `round_key_mismatch` không165;
same127 `duplicate_report_body`, old127 `report_tick_not_forward` khôngforward170. Originbody/
front/upstreamSHA/currentseq/serviceentry được join exactTCPstream, nativeframer/typedfields riêng.

Cùng run sau guestBack/rejoin vẫn hai actualBombFirepickups/vòng, capacity/Fire1→2 mỗi view/reset1/1,
6actual118/159/158 ID1..6, rootA3,3/B3,5/Athird4or5,3/powers2,1,2. Native rootcallback process3,
ownlater khôngduplicate; actualaccepted17/19/cảhai state1 trước175, rank2/2 storage trước180/UIhaiLOSE,
121/clock/175full302/180full3/roomrefresh hai lần/key12345→12346. Sáudeadline/bốnitemconsume/
hairesult; mỗi recipient chỉ bốn165 và bốn170 phát từ actualacceptedreports. Probe hiện tại không có
native170 entrycounter: noextra170 là **emitted-side**, runtime statepeer trước175 và typed170model
kiểm riêng, không giả callbackcount. Encoder/proxy/heap/getteroracles không thay native proof.

**Lỗi teardown và fix có phạm vi:** first `tcp-cycles` có hairesultreset nhưng strict noerror fail:
`Broken pipe` dưới ordinal4 khi AEOF→cleanup gửi107/110 cho peer đang đóng. Errorrow khôngtimestamp,
AEOF timestamp sau earlycompletion là boundary theo source, không trace socket target trực tiếp.
Production `deliver_departure` bắt riêng BrokenPipe/Reset/Aborted/Timeout khi gửi107/110, ghi
`send_failures` có recipient/connection/op/errno/time/sent_prefix rồi close recipient idempotently.
Chỉ metadata gói đã sendall thành công giữ lại; không nuốt lỗi file/codec, không tuyên bố peer đã parse
một partialsend. Các đường send/broadcast khác chưa có recovery/backpressure tổng quát.

91servertests gồm real socketpair hai remoteclose trước readEOF và controlledreset sau107/110failure,
kiểm đúngrecipient/successprefix/no unsentmetadata/no swallowENOSPC. Repeat actualruntime giữ
profile/proxy/teardownorder/outcome/delay, serviceerrors[] và send_failures[]: **khônghit nhánh mới**,
không dùng repeat làm causalproof lỗi đã tái hiện/sửa runtime. Firstfailure giữ nguyên, modelproof
và runtimeintegration tách rõ. Không tự mở lại chỉ để tìm một cleanup race không chặn lượt hiện tại.

Khép bounded acceptance CP075 cho selectedvariants trong run mới, không viết lại partial/manifest075.
Chưa cap/distance/auth/freshserial/currentkeyforgery/toàn127/codec/authority/vanilla/LANInternet/
fullmatch/K5/K6a/fullreset. Tại CP085 bước tiếp là capacity/serial; CP086–088 đã đo, [PLAN](../PLAN.md#current-order) giữ việc hiện hành.


<a id="virtual-tcp-guard-cycles"></a>
### Selected TCP replay guard cùng virtual movement/two cycles — CP143

**kr_runtime/confirmed trong fault injection; proxy/validator/rank là design/emu_choice.**
[Recipe/raw/hash/giới hạn](../evidence/peer-endpoints/README.md#cp143). Reuse CP106/137 ReplayProxy,
CP142 profile/production/observers/input; chỉ đổi proxy bind10.77.0.1:3858→service23858 và chọn
actualA118/124/127:17,19 (A là bên chết hiện tại). Hai childTCPforward vẫn đích3858; **UDP direct**.
Proxy giữ client-front native frames riêng, re-encode upstream/currentouterseq khi cần, giữ body;
S2C forward nguyên byte/archive, không coi proxy frame là client native writer hoặc original auth.

Cùng run có **8 same body** (4 mỗi vòng) và **4 old body** (vòng1→2 sau actualA39 current marker).
Mỗi frame ghép originbodySHA/frontseq/upstreamseq+SHA/serviceguard/nativeframer/getters. Same118
bị `duplicate_bomb_serial`, old118 `bomb_round_key_mismatch`, không cấpID. Same124 bị
`claim_not_current_or_already_consumed`, old124 `round_key_mismatch`, không165. Same17/19 bị
`duplicate_report_body`, old17/19 `report_tick_not_forward`, không170. Đây là policy guard hẹp;
không suy report39 có nghĩa ACK hoặc chọn nó làm world oracle, chỉ dùng marker freshness của recipe.

Có87 guard/integration checks; A135 actualfrontframes→147upstream (12proxy replay), B141→141.
876 front framer emulation cases kiểm valid/CRC/partial; upstream typed getter/report model tách
khỏi runtime. Không thừa acceptedbombID (1,2), native159/158 mỗi view2,165 mỗi view4,175180 mỗi
view2;170 phát đúng report accepted, không replay effect. Policy begin/finish2/itemconsume4/
bombdeadline2. Raw source/config frozen và không service/controller error.

Cùng run vẫn bốn finite movement windows/nativekeyXORID/target/currentprogress, BombFire effects
1,1→2,1→2,2 và reset clock2 ở cảhai view, hai actualA118power2/currenttrapSHA/death/survivorresult/
175180110/195seed/keyIDreset. Barrier25,652/25,540s<30. FinalactualB95seq457/SHA
`c94d21262a645dfe9f2786dc065a88e8e322706a5949cccf79c0b8350948c914` ghép front/upstream,
additional native110full6/errors0 và ảnh readybadge cảhai; không Start3.

Giới hạn: exact captured same/old body, không arbitrary forgedcurrentkey/freshserial/location/tick/
identity/fullcodec/auth/fullUDPgrammar/reliability. Không actual guestEOF giữa trận trong run này;
shutdown cleanup cuối phép thử không phải disconnect acceptance. Không fullmatch/K5/K6a/physicalLAN/
NAT/Internet hoặc đóng CP137complexUI/sharedgeometry/fullinventory/authority debt. Operator IP/rank/
fuse/drop vẫn policy. Bước tiếp duy nhất PLAN; AUD043qua140, audit kếCP144.


<a id="peer21-ending-order"></a>
### Peer170/21, native17 mới sau result và proxy identity — CP103

[Recipe/hash/failedchecks](../evidence/peer21-ending-order/README.md), câu hỏi ở
[RESEARCH](../RESEARCH.md#gap-contact-state), thứ tự ở [PLAN](../PLAN.md#current-order).
**design** cho optionaltranslation/harness; **kr_runtime/confirmed hẹp** cho dispatch/gates/state/timing.
Không dùng scalar26/56 hoặc report21 riêng lẻ làm authoritativealive/winner/serverlaw.

**Optional drained translation:** sau admission/currentcursor,21 mới hơn current17 và latestsuccessful158,
đúngeventcurrentmember/active/chưaaccepted19 → service gửi **đúngbody[:6]** như170/sub21 tới oppositepeer.
Khôngselfecho, chỉ một successfulforward cho mỗi trap-frameSHA/generation; accepted17 mới tạoepoch mới,
Startcleartransitionmarker nhưng cursor127 giữ. Rejectedduplicate/stale/no17/closed/nopeer không gửi;
failedsend không commitmarker, không recoverytransaction tổng quát. Không xóa pickup_ineligible hoặc
accepteddeathhistory. ModeCP100/absentfield khôngđổi.149server tests+3proxyidentitytests+4kernel pass.

**Native receiver đã đo:** cachedCP036E85E92→E86CA0/prefix6, consumerarg/returnE85EA8; actual170/21
reader6/6/errors0 ở B cảhai vòng. R1 `E86CF3` gate1, `E86D0E` vf148=**B43320(0,0,-1)**, cùngactor
pre56/post26 tạiE86D11, callbackend trước B175entry. R2 gate0/actorAđã0, nhánhvf148 không chạy và
endstate vẫn0. Đây là **một clear thực + một gate-skip thực**, không bịa hai clears hoặc suy nohook
thành mấtpacket. Exactactual12721front/upstreamSHA/bodyprefix6/emitted170/nativeargs/cursors đượcjoin.

**Result boundary chưa đồng nhất:** cùngrun2native121/clock/175/180/refresh,14native159entities/
14actual158callbacks/view/60snapshots195flags/process/reset,0controller-serviceerrors. Trướcown175,
AviewA56; BviewA26 ởR1 và0 ởR2, cùngXY80,159; Bstate1 cảhai view. Perviewtimestamps khác nhau, không
làsimultaneousglobaltick. Fulltargetsame-stateassertion fail giữ; scopedreceiverbranches pass riêng.

Timelinepositive mới ởcảhai vòng: nativewriter21 state26 → actualaccepted21 → ranktick/first175send →
**nativeAstate56 mới** → nativewriter17/B78373→B53010→direct → actual17front/`inactive_reports` → own175entry.
Front17 sau first175send **108,344208ms /95,888487ms**, nhưng trước nativeA175entry. Exactnative17
prefix32/retainedframer/cursor/hash khớp, khôngactiveadmission/forward170/outcomechange. State56 mới
xuất hiện **sau send175** trong run này: không gán cho old17queuedtrướcfinish hoặc claim chỉ thêm
freshness/drain đủ ổnđịnh physics tới lúc175. Room.finish vẫn bracket tick→firstsend, không exactns.

Blocker tiếp là **endingphase/recipient/lifecycle**: cần đối chiếu native175→actorchanges/report34→180,
knownE6F370 stage+6220 và E86CA0 gates trước đổipolicy. Có thể state+28 đang animation/phase khác,
chưa kết luận persistentdesync/rankwrong hoặc ownACK21 làfix. Không arbitraryquietdelay/fusechange/
forcedstate/forwardlateclosedreport. NguồnCP036own-onlyACK21 không dừngperiodicreports vẫn giữ phạmvi.

**Harness correction có source:** WCP socket cũng có thể dùng3858. Proxycũ đếmconnection A/B theo
vị trí, nên WCP-A2 ở firstrunCP103 hoặcWCP-B2 ở failedCP102stable chiếmrolechannel; actualchannelB
thứba làm`IndexError` rồi upstream8reset104. Exactproxyupstreamtuple→serviceglobalordinal/replyordinal
vàlaunchertrace xác nhận; không phải kết luận codec/clientfault từstring104. CP102successrepeat được
kiểmtupleA4/B4đúng nênkhôngbác positivecũ. Không giả mọi lịch sử đã audittoànbộ.

`tools/probe_channel_identity.py` tìm đúng currentliveupstreamtuple/listenerport trong **hostfixture
metadata**, chưaauthenticatedidentity. WCP/nonchannel đi rawbytebridge, không tiêu rolechannel hoặc
mode2framer/replay.3tests+realTCPbench4connectionsWCP-A2→A4→WCP-B2→B4, exactretained66byteWCPecho
và2channelframes pass; newnativefixedrun chỉA4/B4 trênproxy, **không** giả đã hitWCPbranchnative.
AUD031manualcanonical→selectedfailed/successraw sửa startupunknown/roleassumption, giữCP102raw/manifest;
periodicCP104 khôngreset. Fullauth/codec/authority/allreset/vanilla/fullmatch/K5/K6a/LANInternet còn mở.

<a id="ending-udp-state-apply"></a>
### UDP state đi qua native175 trong cùng ending — CP105

[Recipe/hash/partialwindow](../evidence/ending-udp-state-apply/README.md); câu hỏi ở
[RESEARCH](../RESEARCH.md#gap-peer-movement), thứtựở[PLAN](../PLAN.md#current-order).
**kr_runtime/confirmed hẹp**, no production/profile/policy change. Reuse CP098API13/key/slot/applyargs/
securefields, CP104native175→3/34beforewrite; khôngportVNnumber hoặcREcodecs/wholeclientlạilần.

**Window đầu sau175** bắt21nativeapply ởB, stateargs0/56/3 qua actualvf204 target**F73610**, args
`(state,0x203F)`; nativeactorA3→0→56→3 vàprotected+5A0=1 giữ. Packet0/56 cóconsumerbuffer13/nativeargs/
setterproof nhưng matchingAPIkhôngởwindow175→180; strictAPIjoinfail đượcgiữ, khônggọiđóoldround.
State3 cóingressmatch; Aview khôngcapturematchingpacket13apply, khôngsuykhôngUDP.

**Window mở sớm** từ native158callbackindex6 (hai callback cuối của7job fixture) tới180, khôngresetcap/
phase tại175.20Bnativeapplysau175 có API17byte/contentmatch duy nhất trongfinitecapture→buffer13
selector0→nativekeywire56=expected56/slot0→B4E6F0 argsstatebyte10/x80/y164→B4EAA3vf204/F73610→after.
Có2statechanges:

| Packetstate | API so với B175entry | API→consumerbegin | Nativeactor+28/setter |
|---|---|---|---|
|56|trước122,629161ms|350,064088ms|3→56,sau175;vf204args(56,203F)|
|3|sau396,801990ms|3,508373ms|56→3;vf204args(3,203F)|

ExactAPIwireSHA/payloadSHA/port/actor/prepost/cursor13 đượcpin. API sourceport9951 khớp actualA99/
B153remoteendpoint, fullroundkey12345 vàexpectednativebyte56 cóscope riêng, khônggánbyteđó==low8fullkey/
authentication. Queuecrossing175đượcchứngminh bằngAPI trước/consumer sau; chưa timestampnetworksend
hayservergốc, chưapacketpreviousround/Startleak/wholequeueorder hoặcuniquepacketnonce.

Protected**+5A0đã1** trước/sau selectedsetters, nên nó khôngngăn **đườngapplyđãđo này**. Cached
B4EA77..B4EAA6 so currentstatev64 vớiarg1 rồi vf204(arg1,203F) khi khác, sauđó+7EC=1; khôngcósource
cho universalendingfreeze ngaytừ+5A0. Không tự václient/blockUDP hoặcthêmpolicyserverdựavàođó.
Bộ64entry/96APIcap giữ; A8/12APIđãđo theo run nhưngkhôngmatching13applywindow, ghiunknown thaynoUDP.

Hai runlàhai **one-cycle** riêng, khôngghép thành nghiệmthu2trận: đầu226,441s (129boundedchecks/
336copiednativefields, API0/56gap), preroll267,466s (140checks/308copiedfields/20exactapplies,
2changes56/3). Prerollkhôngtáihiệnstate0change nênkhôngbịaingress0proof. Cảhaiactual175180/returned585/
refresh,7current158,0errors,cảhaiA3/B1before180. Report34prefixY164 ởprerollkhớpnativewritersnapshot,
175entryAđã26khôngcầnupon17556clear; inheritedCP104XY159/clearassumption failures giữ/táchnewverifier.
285typedpacketmodelcasespreroll và282firstrun làemulationriêng; source/copy/pack/DBkhôngđổi.

Khép exactendingarg/queuegaptrongscope này; CP103entry175numericmismatch khônglàpersistentending
failure. Khôngđóng#166/K5/K6a từhai onecycle hoặcsameendingfield. Bước tại CP105 là currenttwo-cycle
movementtransitionpartial củaCP098; [CP106](#current-movement-cycles) đã bổ sung, thứ tự chỉ ởPLAN.
Fullidentity/auth/codec/authority/fullvalidator/allreset/vanilla/LANInternet/fullmatch còn mở;
AUD032qua104/next108. No ownACK/newquietdelay/fuse/stateforcing/gamecall/UDPcut trongCP105.


<a id="rust-movement-floor"></a>
## Rust movement và floor pickup — CP237 / RUST-M3 partial / KRE-074

[Code](../../rust/crates/kr-service/src/pickups.rs), [model/queue](../../rust/crates/kr-service/src/room_io.rs),
[metadata/raw](../evidence/rust-service/movement-floor.json), [recipe](../evidence/rust-service/README.md#cp237).
Tái dùng CP030/031/049/055/063 floor/writer/stat/key/logicaltick, CP142/189/191 currentUDP/nativeapply,
CP235fixedroom/queues. Không RE body/SDK mới hoặc port VNopcode. Nguồn legacy modO chỉ làm copiedgetter/CRC
đã có hash/locator, không nhập scope/wire clientRust cũ; [nguồn chuẩn](../REFERENCES.md).

Policy `design`: đọc catalog từ **cùng582 đã pin/gửi**, typedlength→knownserializedrawtag0→ba list;
list1 eligible, list2 luôn hidden-unrevealed, list3 nonempty/variant/duplicateindex/unsupportedkind từ chối trước bind.
Giới hạn64records/4096bytes/ExVillage01 13×15/kinds0..2; fixedroom vẫn pin46byte582, không general map/parser.
Không có catalog JSON thứhai dễ lệch. Actual12419byte theo [CP030](items-stats.md#item-pickup), flag0, ctx0, socket-boundslot,
currentArmed/153u32key, exactindex/cell/kind và chưa consumed mới có positive16512byte tới cảhai.
`t` chỉ metadata logicalcounter theo CP055, **không là serial/timestamp chốngreplay**; không bịa serial vào124.
Claim trên modelclone, reserve toàn recipientqueues rồi mới publishconsumed+enqueue; queuedsubject/member/round
fence trước send. Queuefull không grant/partialpublish; TCP đã ghi một phần không atomic/rollback.
Start mới resetclaims/key, leave/EOF clearactiveclaims; identicaloldkey reuse/wrap/fullauth/distance chưa được chứng minh.

`kr_runtime/confirmed` hẹp: R3 e9/Wine/privatevirtualLAN hai namespace/IP/wineserver, normalroom/Back/rejoin/
Ready/withheld99 barrier→spawn. ADown rồiBRight có 217vethUDPrecords/87nativeingress exactsource-IP/port/hash;
selector0/keyXORprofileID/apply13/cursor13, protectedtarget/currentpositions thay đổi đúngactor cảhai.
TCP368/58reply/alltuples-C2Sjoins; oneepoch/6shold sauhaiinput, haiFIN saucompletion. A124index17/kind2/key12346
→one165perview, nativeparsed đúngtuple, itemfound17,12/12/errors0, protectedactor+5E8 **5→6** cảhai.
696copied52A350getter và87copied14A5860CRC trên rawactual; modOa08950/counterreset/selected128bytehelpersmatchcopy
là emulation bổsung, không gamefunctioncall/wholefunctionequality. UI hai view đúnglocalarrow/peerposition/itemgone.
58raw cuối frozen/seal/cleanup0; R1 unsupported12451raw vàR2 unsupported12355raw giữ failure/positive subset riêng.

R2 quan sát actualB **123/body10** nhưng semantics còn `unknown`. Pythonreference không có case123/response;
Rust chọn `emu_choice/design` observe-only cho currentArmedmember, tối đa64/member/round, exactsize10,
**không ACK/effect/forward/inventory mutation**. Counter và opaqueinput không chứng minh request hợp lệ/auth/servergốc.
Boundedsearch hai snapshot cũ không tìm được123join, không kết luận opcode vắng ở client/traffic khác.
Không mở acceptmọiopcode/body hoặc tự trả packet. Chỉ RE field/response nếu thật sự chặn implementation tiếp.

30Rusttests/22kernelTCPcases kiểm wrongkey-slot-ctx-cell-kind/index/hidden/duplicate/onewinner/phase/reset/queuefull
và opaque123budget; synthetic negatives không là nativeattacks. Một kernelshutdownSIGTERM5s/SIGKILL2s timeout,
PID đã mất ở lần nhìn sau; giữ failure/instrumentstatus-thread-wchan, repeat22cases đạt **không là rootcausefix**.
M3 chưa bom/terrain/reveal/life/result-reset/twooutcomes/kim-cứu/capacity4-8; M5 chưa KRaccountbridge.
Python giữ serving/RE/reference, không hot-swap backend giữa run. Internet thật chưa kiểm.


<a id="rust-relay-endpoint-scope"></a>
## Ranh giới endpoint và relay opt-in

Nguồn **design/design** cho policy mới; **kr_runtime/confirmed** cho prototype hai người theo
[evidence](../evidence/rust-service/n1r-fixed2.json). [Module và ownership](../RUST_SERVICE_ARCHITECTURE.md#relay-endpoint-design),
[acceptance](../PLAN.md#rust-relay), [unknown](../RESEARCH.md#gap-relay-endpoint).
Không phục hồi hoặc tuyên bố đây là relay gốc. Default `LegacyDirect` giữ byte và policy đã nghiệm thu.

`EndpointPolicy::Relay` cần account admission và cấu hình `udp_relay` rõ ràng. Packet 153 gửi riêng từng recipient:
entry peer chứa socket relay dành cho hướng recipient → peer; self entry vẫn rỗng. Payload UDP được chuyển nguyên
byte từ socket hướng ngược tới tuple peer đã chủ động gửi vào socket đó. Không thêm token, field, snapshot hoặc
movement authority vào wire KR. Native đã đo source socket, key/helper và actor thay đổi ở hai chiều qua hai vòng.

Route gắn service generation, room key, round và hai membership/session generation/connection. Pool bind hoàn toàn
trước serving. Reservation và mọi queue recipient phải đủ trước publish; không publish một phần khi thiếu tài nguyên.
Pump kiểm account lease không gia hạn idle, rồi kiểm lại room/member/scope trước I/O không blocking. Reset, Ending,
leave, EOF, expiry hoặc revoke vô hiệu lease; không chuyển tuple sang phiên mới. Native EOF và stale/reset có đối
chứng; revoke và nhiều account chung IP mới được kiểm tổng hợp, không tự thành native acceptance cho các ca đó.

Giới hạn policy prototype: datagram tối đa 255 byte; queue, rate, TTL và số recv/send mỗi pump đều hữu hạn theo config.
Tuple đầu phải có IP nguồn của channel account và được pin, không tự rebind. Duplicate được chuyển nguyên nếu còn
budget; relay không deduplicate gameplay. Cổng dùng một lần trong instance, kể cả reservation đã hủy; low-byte key
cùng room key không được lặp. Hết pool hoặc key thì từ chối trước publish. Direct legacy không nhận guard này.

Native stale proof chỉ cho exact gói cũ tới cổng gốc đã nghỉ, cùng tuple sau NAT, có current positive sau đó; không
chứng minh gói cũ gửi vào cổng hiện hành hoặc cổng tái dùng bị lọc theo nội dung. Restart và port reuse chưa giải.
IP, CRC và low-byte key **không xác thực UDP**; spoof first datagram giữa người chung IP còn mở. Relay không sửa
identity, profile hoặc DB. Không suy native hai người thành relay bốn/tám người, mọi loại UDP hay tải vận hành.

Đường đo có một chặng Internet UDP và một phía tunnel qua SSH/TCP với SNAT kiểm soát; không đại diện mọi ISP/CGNAT.
Topology, kiểu NAT, tham số thực, raw/hash/build/copy và cleanup giữ local theo quy tắc hạ tầng riêng.
K6c/K7/N1 cũ và kết quả từng lượt giữ nguyên. M5b và native hồi quy cuối R1 đã có bằng chứng riêng;
N1R-b và đích dịch vụ nội bộ còn mở.


<a id="rust-authenticated-companion-foundation"></a>
### Companion xác thực: nền transport và phép kiểm client

Nguồn: [metadata](../evidence/rust-service/n1rb-companion-foundation.json). Đây là **design** bên ngoài wire KR,
với phép kiểm **kr_runtime/confirmed** chỉ cho endpoint và payload của hai client trong lab. Không gọi companion
là cơ chế của server gốc. N1R-b vẫn theo [acceptance](../PLAN.md#phase1-acceptance), chưa đạt từ nền này.

`kr-tunnel` là crate riêng chạy cạnh client: QUIC DATAGRAM qua Quinn/rustls/ring, certificate server được pin và
capability kiểm trên control stream có deadline. Không chấp nhận đường bỏ kiểm certificate hoặc 0-RTT thay xác thực.
Header route ID 128 bit và sequence 64 bit chỉ nằm trong companion, được tháo trước khi gửi UDP cho KR; không sửa
packet153 hoặc thêm field giả vào native payload. Sequence window 64 cho reordering hữu hạn và chặn duplicate/old.
ID route không đủ cấp quyền: adapter phục vụ còn phải đối chiếu authenticated owner, account/channel, room và round.

Proxy bind loopback, chuyển tối đa 255 byte giữ nguyên nội dung và source socket hướng ngược mà native153 đã chỉ.
Queue có giới hạn, dữ liệu chờ native tuple có TTL 500ms; tuple local đầu không tự rebind. Proxy không xác thực process
cục bộ. Mỗi task/socket thuộc owner; stop có deadline và đợi child, không để task rời owner. Capability/endpoint/cert
phải được phát qua launcher đã xác thực khi tích hợp; ví dụ lab chỉ có route/capability tĩnh và không được dùng làm
dịch vụ game. Công cụ lab khác với contract account và không thay Python RE/reference.

**Đã quan sát trên KR thật:** native153 nhận endpoint loopback; payload từ client A qua proxy A, QUIC hub và proxy B
tới native B giữ đúng byte, và chiều ngược tương tự. Native ingress/current-key/helper và tọa độ actor thay đổi được
đối chiếu với capture loopback hai đầu; capture mạng ngoài proxy ghi traffic QUIC. Hai kết quả trái nhau, reset và
Start tiếp giữ stat/terrain baseline. Một ingress cuối cửa sổ đo có packet joins nhưng chưa có helper hoàn tất;
không suy apply hoặc drop cho nó. Raw/hash, source freeze và cleanup đã seal trước mọi thay đổi tiếp.

**Chưa chứng minh:** capability thuộc đúng session/channel/room trong serving; control cấp proxy route từng vòng;
roster bốn/tám người, vacancy/departure và survivor; tái dùng cổng/key, restart và spoof giữa hai người chung IP công
khai. Sai capability, certificate pin, route và replay mới có kiểm transport tổng hợp; không tự thành native attack
acceptance. Chưa có nghiệm thu Internet, mọi loại UDP, tải hoặc đóng gói từ phép kiểm LAN cô lập này. Các run lỗi và
capture thất bại được giữ riêng trong metadata, không sửa lịch sử prototype.

<!-- protocol-topic-content-end -->
