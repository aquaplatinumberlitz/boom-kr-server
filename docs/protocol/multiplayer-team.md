# Bốn/tám người, đội và admission

[Mục lục](../PROTOCOL.md) · [Câu hỏi đang mở](../RESEARCH.md) · [Thứ tự việc](../PLAN.md#dashboard).

<a id="battle-eight-roster"></a>
### Capacity Battle, 8 slot và điều kiện chia đội — CP147/163

**CP147 lịch sử (`kr_runtime/confirmed`, một copy Wine với roster tổng hợp/selector0):**
`ExVillage01`/code278 nhận đủ slot0–7; Start đếm8 người/2 đội/capacity8. Hai ca
4–4 và1–7 với guest Ready đều trả Start-check0 và phát **C→S104 thực tế**. Ca9 người
trên cùng map đếm9/capacity8, trả5, không phát104. Đây là kiểm roster/Start phía client,
**chưa có8 client thật, start reply/loading/Battle8 người hoặc admission room-full của server**.
[Recipe, control thất bại, raw/hash và giới hạn](../evidence/eight-player/README.md).

Tái dùng CP019: dump runtime `CPlayMapInfoList`1151×52, SHA`49727b4d…`, getter`EDC890`
đọc record+24hex; code278 capacity8. Dump cũng có record capacity12/24: không biến giới hạn
của map đang kiểm thành giới hạn mọi mode/toàn client. VN `oc_a-room.md`/`lobby.md` chỉ
dẫn cách xét roster/Ready; field ledger106/110 và opcode104 của phép thử đều lấy từ KR.

**CP147: `kr_static/confirmed` + emulation có mô hình**, tại đó chưa có runtime của Tiêu chuẩn:
`DCAD80` duyệt16 slot ở nhánh này, đếm người và số nhóm khác rỗng; `DCB0DD` so số người
với capacity. Đường capacity thông thường tới`DCB0F2` trả5 khi vượt; các nhánh ngoại lệ
mode ở sau vẫn giữ giới hạn chưa kiểm. `123FA10` đọc manager+9C; nếu giá trị bằng1,
`DCB12C–DCB191` đòi mỗi nhóm có số người bằng thương nguyên players/teams, sai trả7.
14 ca emulation giữ native getter/phép chia/so sánh, mô hình singleton và truy cập bảng đếm:
4–4 và2–2–2–2 qua;1–7,3–5,3–4 bị chặn khi selector1. Selector0 bỏ kiểm cân bằng này;
hai ca runtime4–4/1–7 đều quan sát selector0 tại caller`DCB138`.

| Mô tả đối chiếu | Mức chứng minh và phần chưa có |
|---|---|
| Tự do cho đội lệch1–7; Tiêu chuẩn/Manner đòi các đội bằng nhau | CP147 selector0 runtime4–4/1–7; [CP162](#current-team-mode) nối actual UI/81/ACK/reset0/1; CP163 dưới kiểm selector1 actual8 slot4–4/negative1–7. Phạm vi normalBattle/copy, không enum mọi mode/server8. |
| Battle: nhốt, đồng đội cứu, địch diệt; kim tự thoát | Dùng lại [contact/cứu](life-results.md#collision-rescue), [kim](life-results.md#needle-selfescape), [hai kết quả/reset](life-results.md#rescue-two-results); không đo lại và không suy luật thắng tổng quát từ policy stub. |
| Nhiệm vụ4 người, quái/boss; Hồi sinh tái xuất hiện ngẫu nhiên và đối thủ được điểm | Tham khảo lịch sử/chủ dự án; **hành vi KR00967 chưa kiểm trong nhánh này**, không nhập luật respawn/score vào Battle thường hoặc mở quét mode trước mục tiêu chơi. |

Control đầu đặt guest flags2 thay vì flags4 đã thấy native check3/notice chưa Ready;
giữ nguyên raw. Ba run sửa input là run riêng, không ép return/count/đội từ debugger.
Các cảnh báo capacity lúc table/117 chưa tới được giữ riêng với kết quả click sau init.
Copy `94ee93dc…` dùng lại CP018/019, **khác copy `e9bb4766…` của CP146**; không cộng các run
thành integration/gate. Tại CP147 service chỉ có A/B/C/slot cố định; [CP208](#eight-session-preview) bổ sung adapter8 và tám actual clock. Đây không nâng kết quả lịch sử CP147 thành integration.
Nguồn câu hỏi và phép đo tiếp ở [RESEARCH](../RESEARCH.md#gap-roster-ready-host), thứ tự ở [PLAN](../PLAN.md#current-order).

**Bổ sung CP163 — `kr_runtime/confirmed`, cùng giới hạn một client/roster tổng hợp:**
[Recipe/raw/hash](../evidence/eight-player/README.md#cp163). Reuse nguyên CP147 Ready flags4/106/profile,
map278/capacity8/guard/getter/loopback recipe; chỉ echo selector1 từ **actual81** vào known86 offset7
đã nối CP162. Hai copy/prefix/run riêng, không set manager/count/đội/guard return từ debugger.
4–4: actual click→Start-check0/request104/send1/pending3. 1–7: actual click→7/không104;
getter `123FA10` tại caller`DCB138` đều1, nhánh reject`DCB185` thấy8players/2teams; ảnh notice
nêu số người các đội phải bằng nhau. Mỗi run đọc đủ8 callback196/196/errors0, map/count/capacity đúng.
Actual81→sent86 hash nối/native12 framer/string/primitive/CRCpartial cases và44 raw/native joins;
không dùng encoder tự sinh làm oracle. Copy94ee93dc khác productioncopye9bb4766, không ghép thành
integration8 người. Không121/loading/Battle8/UDPmesh/8client thật/room-full service; CP147 ninth5 và
selector0 controls giữ nguyên, không đo lại hoặc biến capacity8 của map thành giới hạn mọi mode.


<a id="four-slot-native-consumers"></a>
### Bốn slot Start121/peer153 trên copy hiện hành — CP199

`kr_runtime/confirmed` trong **hai actual KR Wine4b + hai actor tổng hợp**, không phải bốn client.
[Recipe/raw/hash](../evidence/peer-endpoints/README.md#cp199), [câu hỏi còn thiếu](../RESEARCH.md#gap-lobby-identity-followup),
[thứ tự](../PLAN.md#current-order). Source-first tái dùng:

| Nguồn đã có | Dùng trực tiếp | Phần CP199 đo thêm |
|---|---|---|
| [CP020 Start121](load-phase.md#start-reply), CP036 `collision-r1/prepare_friendly3.py` | Prefix48/slotCount+43/row26; slot/u8spawn/u16position/team; đúng blob KR | Four rows trên copy4b: inner152/152, outer181/181/errors0 ở cả hai client |
| [CP147/163 roster8/đội](#battle-eight-roster) | 106 ID/slot/state, Ready4/team bits; map278/capacity8 và guard đã đo | ID1–4, slot0–3, position0–3 qua native validation; không lặp capacity/teamguard |
| [CP026 peer153](udp-peer.md#kr-peer-entry), [CP198 A/B](#native-interleaved-startup) | Prefix35/key/26flags/3endpointtriples, actual A/B tuples/copy/input probes | Active0–3:133/133/errors0; mỗi view đăng ký **ba remote** đúng local/remoteID, bỏ localself |
| VN room/game reference qua các nguồn trên | Định hướng roster/Ready/P2P; không nhập opcode/ID/layout VN | Không codec/body RE mới; không nâng reference thành runtimeKR |

Hai hàng0/1 giữ nguyên; hàng2/3 chọn spawn/position2/3, team0/1. Actor tổng hợp ID3/4 có Ready4,
giữ tên/profile KRGuest của template; không có auth/291/session/socket C/D. Native ở mỗi view khởi tạo
bốn object riêng, clock snapshot có tọa độ0:(120,159),1:(120,399),2:(120,279),3:(200,279); checksum
wrapper được kiểm đọc thụ động, không gọi getter trong process hoặc ghi actor. Ảnh hiện bốn nhân vật.
153 giữ actual A/B mappedtuple; self và hai synthetic có ba endpoint(0,0). Native đăng ký ba remote
slots/IDs ở mỗi view; điều này không chứng minh delivery tới synthetic. Hai cửa sổ normalRight A/B có
actual ingress→key→apply→targetchange/currentprogress; chỉ A/B peer có runtime transport.

Task-local subclass chỉ thêm106/121/153; production service/bundle/copy/assets giữ nguyên. Verifier đầu
đòi cả localself vào peerregister đã bị bác bỏ: raw cho A remote1/2/3, B remote0/2/3; sửa expectation
cho đúng consumer đã biết, giữ nguyên failure/script/runtime. Không suy bốn session/mesh, luật thắng2v2,
result/reset4người hoặc K7 từ phép thử này. **Scope tiếp chọn bốn session A/B/C/D đồng thời, slots0–3,
IDs1–4, team0/1 xen kẽ, một room/ExVillage01/normalBattle**; đây là design tối thiểu, chưa implementation.
K6c Internet/K7 còn mở; điểm chặn là generalize role/identity/roster/Ready/99barrier/153 và kiểm realTCP
trước actual4runtime. Không làm lại schema/guard/codec đã giải.

<a id="four-session-native"></a>
### Bốn session KR thật: phòng/load/clock và movement — CP200

[Recipe/raw/hash/controls](../evidence/peer-endpoints/README.md#cp200), [câu hỏi còn thiếu](../RESEARCH.md#gap-lobby-identity-followup),
[thứ tự](../PLAN.md#current-order). Đây là bước mở rộng sau [CP199 hai actual + hai synthetic](#four-slot-native-consumers),
không sửa kết quả lịch sử của CP199. **Bốn actual KR00967 Wine4b**, bốn tree/prefix/netns/IP riêng trên bridge
cô lập; Wine/LAN giả lập được owner nghiệm thu. Không dùng Nexon hoặc route Internet.

`server/four_player_service.py` là entrypoint opt-in riêng, tái dùng codec/FixtureProfile/send/serializedowner/
loading-fence của service hai người. `four_player_profile.py` chỉ biến đổi nhánh zero-profile/rawblob/namesASCII5/7
đã phục hồi: A1/slot0/KRLocal/team0; B2/slot1/KRGuest/team1; C3/slot2/KRNew/team0; D4/slot3/KRPeer4/team1.
ID/tên/đội/sourceIP là **design/emu_choice**, không auth/account gốc. C đồng thời slot2 tách replacementCslot0.
Default hai người/replacement/bundle cũ giữ nguyên. `--source-ip` bắt đủ bốn IPv4 riêng; mỗi role bốn transport
trong lifetime process, không reset count khi EOF, reject source lạ/fifthtransport trước cấp channel/counter.

Room scope một ExVillage01/normal/Free0: A create, B/C/D join; mỗi view có own106/110 và ba remote106,
291 theo tên auth. Ba guest Ready95 phải có đúng boundedblob+desired1/tail0; context opaque vẫn chưa auth.
Host104 chỉ Start khi đủ bốn member và Ready, chỉ epoch1 ở milestone này. 121 bốn row181byte giữ tail/map;
barrier pin đủ bốn actual99 trước197/196/153. 153 mỗi view133byte/26flags, self0/ba remote theo **operatorIP +
actual99port**, không tin IP trong request99 để chọn route. EOF đóng room/fence, không tự chọn rank hai người.
Opcode gameplay/bom/event/result/reset chưa hỗ trợ ở entrypoint mới, có rejection rõ; không giả trận hoàn chỉnh.

| Mức chứng minh | Kết quả/giới hạn |
|---|---|
| `design`/implementation | 309 server tests, 200 realTCPchecks/16transport xen kẽ DCBADCBA…; ID/name/stage/seq/Ready/barrier/153/EOF controls. Encoder hoặc TCP driver không là oracle native. |
| `kr_runtime/confirmed` R2/R3 | Cả bốn request6/291 tên đúng, ownslot0–3, actual join/Ready/start,121181/181/inner152/152, bốn99/clock và153133/133/errors0/ba remoteIDs. R1 authreply1 sai header: fixture KR reply4, đã sửa/regression; raw giữ nguyên. |
| `kr_runtime/confirmed` R6 ít probe | Bốn normalOSinputwindows; currentcoordinates của mỗi origin thay đổi ở cả ba recipient (12 chiều). PCAP độc lập kv-a thấy sáu chiều có A; không toàn bộ12Ethernetedge. Không có key/E668ADprobe trong R6 nên không gọi đây là12nativeapplycall proof. |
| `candidate`/observer effect | R2/R3 nhiều hotprobe: đủ12 API source→peer/queue nhưng queue sau hiếm drain/C-Dapply chưa quan sát; cache14E5710 duyệt từ queue đầu. R6 cùng service/profile/topology giảm hotprobe đã có12currentprogress; D dùng Down thay Right bị terrain chặn. Chưa chứng minh một nguyên nhân duy nhất cho mọi queue/timing. |

R4 dừng vì observeroutput cố địnhA, cleanup/seal exit130; R5 observerpaths đã sửa nhưng finalrunJSON vướng
circularmetadata, giữ **partialseal riêng/không tạo run.json giả**. R6 chỉ sửa phase_id string và có seal đầy đủ.
Mọi run source/config frozen tới cleanup, bản copy/sourceDB/assets giữ nguyên; không ghi actor hoặc gọi hàm game.
**Tại CP200**, bốn client đã vào Battle và quan sát movement liên thông; lúc đó chưa bốn-player bom/trap/death/normal2v2outcome/
175/180/reset/Start2 hoặc hardening. [CP201–207](#four-netem-restart) đã bổ sung những scope được dẫn; K7/Internet vẫn mở. Phần hiện hành theo PLAN;
không port rankAB hoặc coi ownerWineacceptance là tự đóng gate.

<a id="rust-four-startup"></a>
## Rust bốn client: phòng, loading và current movement — CP245 / M4 partial

[Code](../../rust/crates/kr-service/src/room_io.rs), [metadata](../evidence/rust-service/four-startup.json),
[raw/hash](../evidence/rust-service/README.md#cp245). Tái dùng CP200–207 và committed Python builder
`four_player_profile.py` hash61690446… tại dc71e33; loại tám WIP khỏi baseline, không RE binary mới.
CP200 startup và CP201 stat/bom là contract đầu vào; Rust phải có native acceptance riêng, generator không là oracle.

Diagnostic roster chỉ2 hoặc4, IP operator→ID/tên cố định. Four: KRLocal/1/slot0, KRGuest/2/1,
KRNew/3/2, KRPeer4/4/3; team alternating0/1. **R3 actual81/86/87 là Manner1**, không gọi Free0
như CP200/202 cũ. Auth/profile291/106 có size theo hai name đã biết5/7: C135/126/200, A/B/D137/128/204.
R1 chỉ ra ancillary544 C/body8; CP200 raw đối chiếu cùngshape, name7/body10. Cho đúng knownsize/opaque,
không gán field/ACK;67 vẫn2, heartbeat15 vẫn31, budget/stage giữ. Mỗi peer106 batch có subject membership riêng.

Room admission/Ready/round/visit guards mở bounded4;121 outer181/inner152,153 body133/26flag rows,
own endpoint zero và ba actual99 peer tuples. D99 giữ read-only theo recipe cũ: first3 ACK ≥3s không GO;
fourth actual99 mới197/196/153/clock. Native bốn view fullcursor/errors0/IDs1–4/slots0–3, clock/spawn/ảnh đúng.
Normal OS input A/B/C Right, D Down mỗi0,25s:12 directed remote current-coordinate progress +4self;
7.128 copied-native52A350 getter checks trên read-only wrappers trong Unicorn riêng. Không có key/E668AD
apply-call probes mới; không coi copied emulation là Wine execution. PCAP kv-a chỉ sáu cạnh A, không cả12Ethernetedges.

Native R3 binary bab87fed…/client4b781fbc…/Wine10.0-20260425, private root+A/B/C/D netns/bridge/no default route,
16TCP transports/698records/112exactreplies nối fulltuple/C2Sseq/CRC/hash;64raw/105,413713755s/sourcefrozen/cleanup0.
R1 39raw fail544/8 trước fix; R2 43raw controller Ready/Start sai410,566 nên không95/104/99,
intentionalSIGINT130/finallycleanup, R3 dùng CP200605,526. Giữ failures/oldinputs; R3 dùng lại private prefix/game
sau seal R1, không coldprefix/freshinstall. Không đổi client patch: ABI-only copy CP187/200.

58workspace tests (53service+5wire),7fourTCP trên public exporter output và51twoTCP trên samefinalbinary đạt.
Public exporter thêm **sau seal R3**; native dùng private preparer, retained pre-run exporter hash4d1a3dea… kiểm
đúng originalbytes; không sửa preflight/raw thành current source. Generated peer tail chỉ template, runtime dựng từ99.
TwoTCPR1 unexpectedEOF/cleanupdeadline giữ; D-state Btrfs quan sát lúc cleanup không chứng minh nguyên nhân EOF;
sameoldbinaryR2 và finalR3 đều51pass sau inputcopy kết thúc, không tăng deadline/không claim I/O fix.

**Startup4 hiện reject Battle/terrain config trước bind và gameplay118/124/142**. 127 ngoài Battle vẫn bounded
opaque/noeffect; chưa fourlife/bom/results/reset/Start2/EOF, chưa tám/fullaccount/generalroom/production.
M3 hai client đã đạt CP242 giữ regression; M4 partial. [PLAN](../PLAN.md#current-order) tiếp fourownerbom/life170 đủ
ba recipient→rank4/outcomes/reset/Start3→abort/EOF, không clone two-player ranks/policy. M5backend đã mergeCP244,
KRbridge riêng; AUD070 qua244 không đổi/audit kế248. Internet thật chưa kiểm.

<a id="native-interleaved-startup"></a>
### Native startup xen kẽ A/B với binding opt-in — CP198

[Incoming policy CP197](#operator-incoming-binding), [recipe/hash](../evidence/peer-endpoints/README.md#cp198),
[phép thử kế](../RESEARCH.md#gap-lobby-identity-followup). `kr_runtime`, confirmed cho lịch launch/connection này.
Giữ nguyên source/bundle/copy4b/profile47/NAT/qdisc/input recipe CP197, chỉ launch B sau transport2 của A;
UI Join B chờ actual createconfirm A để không trộn admission với lỗi thứ tự room.
Actual accept order **A1,A2,B1,B2,A3,A4,B3,B4**, mỗi source/privateIP có role đúng/stage1..4/count4.
Native identity291/room/join/hostBack/rejoin và normal start đều trên các connection này; không từ log
process launch hoặc TCP accept đơn lẻ nâng thành login/gameplay.

Cùng run có hai normal B wins/Roller4→5→4/A5, Aselfbomb/trapdeath/B21/full175180/room/Ready2/
normal B Start3/ba clock/keys/195seed reset. Bốn peer windows/hai key có mapped API/PCAP exactSHA/
key/apply/target/currentprogress; copiedgetter emulation tách native runtime. Không đổi service/wire/fixture
để ép pass. Default legacy vẫn cần serializedstartup; opt-in không auth/sameNATIP nhiều account/C-D/
bốn–tám actualclient/generalaccountresume hoặc mọi interleaving. K6c Internet/K7 vẫn mở.
Phần còn thiếu cho nhiều người: service hiện hardcode hai live peers/role IDs/26slot153/rosterreply/preset;
không mở client thứ4 trước khi có roster/session/admission policy và kiểm fixture/nativeconsumer tương ứng.
PLAN chuyển sang source-first đánh giá những chỗ hardcode và chọn scope4người tối thiểu, không RE lại
codec/8slot/teamrules đã có CP147/163 hoặc mở mode/item phụ.

<a id="operator-incoming-binding"></a>
### Incoming A/B theo source IPv4 riêng, opt-in — CP197

**`design`/implementation**, tách khỏi identity/accountauth KR gốc. [Recipe/hash](../evidence/peer-endpoints/README.md#cp197),
[câu hỏi kế](../RESEARCH.md#gap-lobby-identity-followup), [cách chạy](../../tools/README.md#incoming-admission).
CP190 realTCP đã chứng minh globalordinal nhóm4 gán sai6/8 khi A/B xen kẽ. Không RE lại native wire:
`ProbeService.accept` nay có keyword `admission_ips`/CLI repeat `--admission-ip ROLE=IPv4` opt-in,
phải khai đúng A và B với hai IPv4 riêng. Đây là IP **nguồn TCP thực**, khác `--peer-ip` quảng bá153 sau NAT.
Không khai thì policy cũ/serial admission và freshC replacement giữ nguyên.

Khi bật, mỗi role có ordinal transport1..4 riêng trong lifetime process; globalordinal vẫn đếm accepted
connections. Unknownsource/fifthtransport bị đóng trước tạo event/channel hoặc tăng counter role;
rejection lưu reason design. Không đặt lại count khi EOF: không account resume/retry/generic reconnect.
Từ chối kết hợp replacement_slot, C/D và cùng IP hai role, tránh suy IP có thể tách account chung NAT.
Cấu hình lỗi bị chặn trước selector/listen/đọc fixture ở CLI; codec/fixture/ID/seq không đổi.

RealTCP/current47fixture controls: default ABABABAB vẫn6wrong/8; opt-in ABABABAB và BBAAAABB đều
0wrong/8. IP lạ và kết nối thứ5 bị reject, không ảnh hưởng role kia; tám unit regression kiểm real sockets/
default/negative/config và302server tests pass. New bundle có59pins/47alias/43fixture giữ đúng byte profile,
khác source service mới; launcher -I vẫn kiểm integrity. Manifest mới ghi defaultserial/opt-in scope.

`kr_runtime` confirmed riêng: cùng recipe native **startup tuần tự** đã có hai normal B wins/reset/Ready2/
normalBStart3/currentpeer/nativeflags/stat/key/life qua NAT/UDP qdisc trên bundle mới, đúng binding A/B
privateTCPsource và stages1..4/count4. Đây là compatibility runtime, **không native interleavedstartup**;
realTCP xen kẽ là control riêng; [CP198](#native-interleaved-startup) bổ sung native lịch AABBAABB. Không auth/token, nhiều client cùng sourceIP, C/D/bốn–tám actualclients,
NAT discovery/relay gốc/Internet gate. K6c/K7 giữ mở; native startup xen kẽ tại CP197 còn thiếu đã có [CP198](#native-interleaved-startup),
rồi mới mở admission/roster/peer nhiều client theo PLAN. Không lặp hai vòng trên sameconfig không có biến mới.

<a id="four-owner-bombs"></a>
### Bốn owner bom: baseline →118→159→158 — CP201

[Recipe/raw/hash/failedcontrols](../evidence/peer-endpoints/README.md#cp201), [câu hỏi kế](../RESEARCH.md#gap-lobby-identity-followup),
[thứ tự](../PLAN.md#current-order). **`kr_runtime/confirmed` cho bốn actualWine/copy4b/ExVillage01/Free0/epoch1**,
không phải normaloutcome/reset2v2. Tái dùng [CP200 bốn session](#four-session-native), [stat CP031](items-stats.md#item-effects),
[owner/serial159](bombs.md#bomb-position), [158kind0](bombs.md#bomb-cycle) và fence/capacity đã có; không nhập ID/opcode/layout VN.

**Baseline native bốn slot**, cả bốn view đọc cùng giá trị tại clock; 144 securegetters được kiểm lại bằng
hàm52A350 trong emulation riêng (không gọi hàm trong process):

| Slot/role | Capacity/max | Power/max | Speed/max |
|---|---|---|---|
| 0/A |1/10|1/7|5/7|
| 1/B |2/7|1/9|4/8|
| 2/C |1/8|1/5|5/8|
| 3/D |1/10|1/8|4/8|

Đây là char/profile đã chọn; không default mọi nhân vật/mode. Request-only run bắt normalSpace từ mỗi role,
actual118 có đúng currentkey/slot0–3/power1/serial/CRC, service khi đó reject118. Nhờ đo, D không bị gán capacity2
của B; maxima C/D cũng không sao chép AB. Old startup failures không sửa thành positive.

Opt-in `--bomb-profile` của service4 khai rõ capacities1/2/1/1, powers1 và fuse6s. `four_bomb_policy.py` là
**design/emu_choice**, kiểm member/currentepoch/currentkey/boundslot/knownlayout/mapbounds/configuredpower,
serial riêng từng owner, duplicate/conflict và reservation/capacity riêng. Chỉ acceptedrequest mới lấy wireID;
EOF/roomfinish vô hiệu mọi pendingticket. Không dùng report127 làm authority, không tính wholeterrain/drops/
chainquota/fullinventory hoặc nativepoollaw. Không khai bombprofile thì CP200reject118 giữ nguyên.

Run accepted4 có một normalSpace/role, actual118 A→ID1/B→2/C→3/D→4; 159broadcast4recipients mỗi bom.
Cả bốn nativecallback159 đọc39/39/errors0, trả entity khác0; protectedcoords khớp actual118cell×40/y+39.
Deadline6s phát158/count1/currentID/kind0; cả bốn callback158 đọc13/13/errors0 mỗi ID. Nativeentitycreation và
fullconsumer158 có nguồn riêng với service/capture; không encoder tự sinh làm oracle. ProbeE14360 thêm có
nhầm EBPargument tại functionentry, **không dùng matched_confirm_entity làm proof**; raw/script giữ nguyên,
không làm lại run chỉ để sửa probe ngoài acceptance này. Chưa chứng minh mọi damage/chain/ownerdeath.
316servertests/207realTCPcontrols kiểm4serials/duplicates/capacityD1-B2/16×159/16×158/EOFfence; đây là kiểm code,
không thay native. Fuse/guard/lựa chọnrank chưa là servergốc. **Tại CP201, life127/survivor/rank1754rows/180/reset/Ready2/
Start2 chưa được triển khai/nghiệm thu**; [CP202](#four-two-team-cycles) đã bổ sung trong scope2v2; không port AB rank hoặc biến selftrap/forfeit thành2v2normalending.

Startup controls trước baseline: R1/R2 C SIGABRT và R3 A đi HTTPcontent→inputJSONempty→throw`expected value`.
CP073 cache đã định danh content khác locale; chưa chứng minh mọi startupabort cùng cause. Candidatecopy8d7194fd
thêm guard chỉ inputsize0→nativeemptyoutput/ret8, nonempty giữ originalcontinuation;5Unicornbranch/ABImodels,
byte-exactrollback về4b. Runtimeguard thấy emptyinput/nativeoutsize0/cap15, sau đó cleanupSIGILL2260DF8;
cacheCP181 chỉ ra trap16EA064/UninstallDWKcandidate. **Guard không giải startup tổng quát, không đưa vào service/
recipe chính hoặc coi cleanuptrap làAPI cần bind.** Runbaselinepositive dùng original4b với coldframerprobes;
không thấy primaryfatal/contentthrow trongrun này, rootcauseintermittent vẫnunknown. Source/4b/DB/assetsgiữ nguyên.

<a id="rust-four-owner-bombs"></a>
## Rust bốn owner bom: opt-in, current native entity và nổ — CP246 / M4 partial

[Allocator](../../rust/crates/kr-service/src/bombs.rs), [adapter](../../rust/crates/kr-service/src/room_io.rs),
[metadata](../evidence/rust-service/four-owner-bombs.json), [raw/hash](../evidence/rust-service/README.md#cp246).
Tái dùng CP201/CP238/CP245; CP201 là Free0, **run Rust hiện hành là Manner1/cố định2v2**.
Không RE thân hàm mới hoặc import Python WIP. Consumer mới phải được đo trên Rust, không dùng generator làm oracle.

`fixed_room.four_owner_bombs=true` chỉ hợp lệ diagnostic4, bật capacity1/2/1/1, power1 và designfuse6s.
Startup4 không có flag giữ reject118/142; Battle/terrain config4 và124 vẫn reject. Ledger chỉ2 hoặc4 unique current
members, serial riêng owner, chỉ accepted lấy ID; reserve mọi spawn/deadline queues trước publish. Reservation nhả khi
**cả bốn TCP writes158 thành công**, không là nativeexecutionACK; member/visit/round/ticketscope cũ/EOF vô hiệu pending.
Wrongkey/owner/power2/duplicate/capacity không lấy serial-ID; B2 khác D1, ba writes không nhả reservation.
Chưa fouraccepteddeath nên không nhận dead-owner guard bốn đã đạt; life127 hiện chưa apply/forward.

CP201 raw đã có142body7/selfslot/aux37 hoặc-1 và167body22; Rusttwo whitelist1674/16/32 chưa đủ cho bốn.
R1 sau Abom đóng4channel tại actual16722, rồi controller screenshot timeout; giữ73raw/150,767s/exit1/cleanup0.
Newraw4frames22 và historicalCP20112frames22 đều CRC/seq checked. Chỉ thêm16722 cho fourbombopt-in,
noACK/effect/forward, giữ own64budget. Four142 chỉ knownsize/selfslot/13×15bounds/aux37/-1/currentArmedmember,
budget64/round/visit riêng, **không reveal/eligibleitem/response**. Không gán semantics field167; heartbeat/other shapes giữ.
Controller mới kiểm channelclosure sau mỗi7s ownerhold trước screenshot, không che lỗi gốc bằng ảnh; chưa giải mọi screenshot/I/Ostall.

Native R2 trên Wine10.0-20260425/copy4b781fbc…/private root+A–D netns/bridge/no default route, reusedcachedprefixes/newprocesses,
không patch mới/coldinstall. Bốn view cùng stat/max CP201: A1/10,1/7,5/7;B2/7,1/9,4/8;C1/8,1/5,5/8;D1/10,1/8,4/8
(capacity,power,speed). Actual normalSpace mỗi owner118/currentkey12346/slot0–3/power1/serial→ID1–4.
16native15939/39/errors0/entitynonzero/protectedcoords=(column×40,row×40+39);16native15813/13/errors0,
corrected lookup→kind0→cleanup cùng entity/currentID và delay≥5,8s. 7.196copied52A350getterchecks riêng trongUnicorn,
không gọi inferior/không wholebinaryoracle.902TCP/144exactreply/16transports fulltuple/C2Sseq/CRC/SHA joins;
84raw/139,844435953s/sourcefrozen/cleanup0,12actual142 chỉobserve. No165/170/175/180.

59Rusttests/14fourTCP gồm capacity/serial/all4completion/known142-167/noeffect;51twoTCP life-result-reset-EOF trênsamefinalbinary.
Two-regressionR1 chọn oldroomharness với rank-enabledconfig:41cases rồi immediate107 expectation sai delayedforfeit,
timeout6s/PIDgone/sourcefrozen; giữfailure, dùng existinglifeharnessR2/51pass, không tăngdeadline hoặc gọi server/IOfix.
Ảnh bốn cuối run có **own/peer trap-death khác nhau** khi127 chưa forward; không suy sharedlife/outcomes/fullM4.
[PLAN](../PLAN.md#current-order) tiếp170đủ3recipient/clear21fanout→rank4/opposite2v2/reset/Start3→scopedabort/EOF→8.
Rollback giữ cặp binary+config startup CP245; oldbinary không hiểu newflag. M5backend đã có/KRbridge riêng,
AUD070 qua244 không đổi/audit kế248. Internet thật chưa kiểm.

<a id="rust-four-life"></a>
## Rust life bốn người: 17/19 tới ba peer, completion clear21 — CP247 / M4 partial

[Life](../../rust/crates/kr-service/src/life.rs), [adapter](../../rust/crates/kr-service/src/room_io.rs),
[metadata](../evidence/rust-service/four-life.json), [raw/hash](../evidence/rust-service/README.md#cp247).
Tái dùng CP202 native report/170/state56→1, CP240 typed Rustlife và CP246 currentfourbom/Manner1;
committed Python reference, không WIPservice/VNwire hoặc RE thêm thân hàm.

`fixed_room.four_life_reports=true` cần diagnostic4 + four_owner_bombs; không tự bật rank/result/terrain.
Chỉ17/19/21 được admit theo boundslot/currentround/connectioncursor/reportbudget. Unsupportedsubtypes và report
sau accepted19 không đổi cursor/life/forward; death19 cần current17. Current accepteddeath chặn118 mới,
không lấy serial/ID. Ticks là freshness policy, không authenticated life/wireepoch/originalauthority.
Mỗi forward dùng **ba peer membership hiện hành**, không echo origin; reserve cả3 queue trước publish model.
Guard source/recipient/visit/round giữ; không dùng `.find` chỉ chọn một peer.

21 giữ prefix14→170prefix6. Pendingclear bind tập unique1 hoặc3recipientmemberships; chỉ đủ successful TCPwrites
mới đánh dấu once-per-trap. Duplicatecompletion/wrongvisit không tiêu thụ recipient khác; blast/newtrap/roundreset
vô hiệu clear cũ. Không coi writes là nativeexecutionACK; clear21 không hồi sinh accepteddeath hoặc mở inventory.
Unit test dừng sau2/3write vẫnvalid, duplicate/wrongvisit/stalecompletion/clear mới/blast/roundreset riêng;
TCP kiểm ba170clear đến đủ3peer, once/newtrap và2clientregression. **Chưa nativeclear21/rescue/needle bốn người**.

Native current Manner1/fixed2v2, Wine10.0-20260425/copy4b/private root+A–Dnetns/bridge/no default route, cachedprefixes/newprocesses.
A/C normalSpace tự tạo bom rồi actual12717→19; mỗi report có đúng3recipient170/prefix10 hoặc21/noorigin.
Remote native protectedstates56→1 ở mọi peer; freshreadonlysamples cả4view `[1,0,1,0]`, B/Dalive.
Không force19/actorwrite hoặc dùng serverlog làm proof. Rare CP202 FinishBreakpoint có cursor0 saureturn,
**không là full170parseproof**; native state transition và fullwirejoins là bằng chứng riêng.
874TCPrecords/140exactreplies/16transports/C2Sseq-CRC-SHA/fulltuple;76raw/134,675276088s/sourcefrozen/cleanup0.
7.240copied52A350getterchecks riêngUnicorn/counterreset, không wholebinaryoracle. Ảnh cả4 đã xem/clock01:06;
corpseanimation khác do chụp tuần tự, không pixelidentity/fullworldconsistency claim. Không175/180/result/reset.

60Rusttests (55service+5wire),17fourTCP/51twoTCP chạy tuần tự sau xác nhậnPIDgone,3actualprebindnegativeprofiles.
KernelR1 đủ17hànhvi nhưng cleanup5s+3s chưa reap PID885548; kiểm sau PIDgone, không có at-timewchan nên chưa rõcause.
TwoTCPR1 mở trước kiểm PIDcleanup4, giữ timinglimit; kernelR2 thêm PID/thread/wchan observations khi deadline,
17pass/exit0/PIDgone rồi twoR2/51pass. Không tăngdeadline hoặc nhận đã fix shutdown/Btrfs.
VerifierR1 nhầm mọi127 đều typedlife; một D127 sau ownedchannelEOF đi boundedabortdrop/noeffect. Giữ log/oldverifier,
chỉ sửa phân loại cleanup-tail, không sửa raw/run hoặc gọi healthyfourEOF/return đã nghiệm thu.

[PLAN](../PLAN.md#current-order) tiếp rank4/haiopposite2v2/reset/Start3/scopedabortEOF rồi8. Outcome phải xét
currentwitness/drain/settle8 và allrecipient life delivery; không clone rank2/forfeitpolicy hoặc để Ending làm mất170.
M4partial, M5backend đã có/nativebridge riêng. AUD070qua244/audit kế248; Internet thật chưa kiểm.

<a id="four-two-team-cycles"></a>
### Hai vòng 2v2 trên bốn client, kết quả/reset/Start3 — CP202

[Recipe/raw/hash](../evidence/peer-endpoints/README.md#cp202), [câu hỏi/giới hạn còn lại](../RESEARCH.md#gap-lobby-identity-followup),
[thứ tự duy nhất](../PLAN.md#current-order). **kr_runtime/confirmed** cho cùng bốn KR00967 Wine copy4b,
16TCP/4prefix/netns/IP riêng, ExVillage01/Free0, A+C team0/B+D team1; không Internet/K7 hoặc fullinventory.

| Nguồn đối chiếu trước nhánh | Đã dùng trực tiếp | Delta phải kiểm KR |
|---|---|---|
| [KR report/rank CP023](life-results.md#round-result), [peer CP028](life-results.md#peer-events), [drain CP102](life-results.md#drained-result-local-clear), CP201raw | 127/17,19blob;170prefix6/aux;175header16/row131/16slots/featureoff;1803; seq/key/fence | CP201chunks đã có4trap17/3death19; không chạy lại để khám phá envelope. Chuyển tới3peer thay vì một; 4 populated rows |
| VN `boom-re/docs/oc_e-game.md` §397–425, hash ởsource-first | RuntimeVN2 tham chiếu phân đội/survivor→result→room; tránh RE lại ý nghĩa thắng đội/chu kỳ | VN168/173/offset/layout không chuyển KR; winner/rank/economy server giả lập phải khai design |
| CP200/201 service4/native4 | Session/roster/121/153/currentstats/4owner118159158 đã có | Same4client hai normal2v2outcomes/reset/Ready2/Start2/Start3, không dùng forfeit hoặc syntheticguest |

`--battle-outcome` opt-in cùng `--bomb-profile` dùng `four_battle_policy.py`: chỉ17/19/21 đã biết,
currentmember/loaded/boundslot/knownblob, freshness riêng mỗi connection (giữ cursor qua Start), duplicate,
19 phải sau current17; accepted19 không bị17/21 sau đó hồi sinh. Sub39/40 vẫn unsupported/opaque.
17/19 dịch170 cho đúng ba peer, không phản xạ origin; commit marker/cursor sau send thành công.
21 chỉprefix14; nếu đã trap thì phát170/prefix6 tới ba peer, không tự mở lại inventory/life.

**Lựa chọn emulator/design:** đội chưa có accepted19 được coi còn người; chỉ kết thúc khi còn đúng một đội,
ít nhất một survivor21 sau latestdeath/latestblast/currenttrap, không còn pendingbombticket và chờ8s.
All-dead/tie, rescue18/needle/mode phụ không thuộc opt-in này. Rank cho cả đội thắng=1/đội thua=2, economy0;
552raw=16header+4×131+12empty, header9=FF tắt feature bổ sung. Native copiedparser đọc16→147→278→409→540→552;
đây chỉ emulation; runtime riêng bên dưới xác nhận toànconsumer. Không accountauth/authoritativealive/servergốc.

**Run cùng bốn client:** R1 actualSpace A/C→bombIDs1/2, accepted12717→19 mỗi người, B/D21 sau death/blast;
R2 actualSpace B/D→IDs3/4, A/C21. Mỗi17/19 có3recipient170; native remote actor đạt56 rồi1.
Ngay trước175, cả4view đồng thuận R1states[1,0,1,0]/R2[0,1,0,1];32protectedwrapper đã kiểm lại native52A350
trong emulation riêng. Native175 đọc562/562outer/errors0 và552/552inner/16rows, ranks[2,1,2,1] rồi[1,2,1,2].
Ảnh đã đọc có4dòng/tên đúng/LOSE-PERFECT đổi đội. Mỗi180 đọc3/3/errors0; refresh110 cả4slot tới4view,
actual95 củaBCD và104 củaA đưa Ready2/Start2 rồi Ready3/Start3/3nativeclocks mỗi view. Keys12345/12346/12347,
cùng channels, seq tiếp tục. Không patch actor/forced19/resulttimer giả hoặc AB rankclone.

**Giới hạn observer:** GDBFinishBreakpoint ở170 thấy cursor0 sau return dù protectedstates thay đổi đúng;
không lấy cursor này làm full170parseproof hoặc suy cause reset. Nativepre175state và175/180 in-functioncursors
là bằng chứng độc lập. ProbeE14360entryarg đã sửa ESP trên bản recipe mới; không sửa CP201raw.
**Tại CP202 còn thiếu:** deadowner118guard/fourportable; [CP203](#four-portable-guards) đã bổ sung. Restart/boundedparser/
backpressure/rate-limit/persistence/health và genericstale/fullgrammar/authority còn thiếu. Items/terrain/kim/cứu
của hai-client service không tự suy đã tích hợp bốn người. Không tự đóng K7 hoặc Internet từ hai normalcycles.

<a id="rust-four-results"></a>
## Rust bốn người: hai kết quả 2v2 đối nghịch, reset/Start3 — CP248 / M4 partial

[Result](../../rust/crates/kr-service/src/four_results.rs), [delivery](../../rust/crates/kr-service/src/life.rs),
[metadata](../evidence/rust-service/four-results.json), [raw/hash](../evidence/rust-service/README.md#cp248).
Nguồn CP202 nativeaccepted175/180/layout/rareprobes, CP247three170/currentManner1; không RE thêm hàm/VNwire/import WIP.

`fixed_room.four_results=true` cần diagnostic4/four_life_reports/four_owner_bombs. Không bật theo packet `rank` hai người;
legacyBattle2 vàterrain4cfg vẫn reject. Typed rank dùng16header (byte9FF) +4×131rows +12empty =552inner/562outer,
cả đội thắngrank1/thuarank2/economy0. Hai payload SHA khớp actualsealedCP202 fixtures; đây là **wire regression**,
không encoder tự chứng minh nativecompatibility. Return180known3bytes echo selector0/1.

Outcome chỉ khi đúng một team chưa có accepteddeath, latest=maxdeath-lastblast qua8s, không pendingbom,
có current21witness sau latest/currenttrap. All-dead/tie4 chưa hỗ trợ; không ghép rank_all_dead hai người.
17/19 giữ pendingdigest→tập distinctrecipientmemberships; cùng pending21, **mọi TCPwrites170 phải hoàn tất trước Ending**,
không làm mất170 còn queued khi guardphase đổi. Duplicate/wrongvisitcompletion không nhả recipient khác;
begin resetpending/life nhưng giữ connectioncursor; 21 bị blast/newtrap invalidate như CP247.
Reserve mọi queue trước publish; thành công writes không là nativeexecutionACK. Authority/fuse6/settle8/rank/economy là design.

Native cùng4 Wine10.0-20260425/copy4b/private root+A–Dnetns/bridge/no default route/currentManner1/fixed2v2:
R1 A/CnormalSpace→17/19→three170, nativepre175all4states[1,0,1,0], ranks[2,1,2,1]. R2 B/D cùngpath,
states[0,1,0,1]/ranks[1,2,1,2]. Mỗi view175outer562/562/errors0, inner552/552/16slots/rankmemory đúng và1803/3/errors0.
Currentwitness21 sau death/blast, drained158 và all170 trước rank được đối chiếu cùngstream/C2Sseq-CRC-SHA/fulltuple;
nativeprotected56→1/pre175states independent, không dùng serverlog-only. Rare170Finishcursor0 không là fullparseproof.
Normal175→5s180→3s4slot110refresh mỗi view; actual95BCD/104A/99×4 đưa Start2 rồi Start3 cùng16TCPtransports,
keys12346/12347/12348; peer153133/121181 parsers full mỗi vòng. Sau clock3hold6s lấy freshreadonlyall4state0.
48copied52A350getters kiểm32pre175+16Start3wrappers riêngUnicorn/counterreset; callbackclock ban đầu không dùng suy state0.
84raw/236,896244281s/sourcefrozen/cleanup0,1.691TCP/280exactreply. Recipe khai300s tổng cho2cycles, I/O/cleanupdeadlines giữ.

62Rusttests (57service+5wire),24fourTCP gồmoppositevectors/16refresh/Ready-Start3/oldreportnegative/currentclearpositive,
51twoTCP tuần tự/cùngfinalbinary,4processprebindrejects. VerifierR1 KeyError metadataevent thiếuelapsed_ns được giữ;
R2 chỉ phân vùng timestampedevents, giữretainedpre-run verifierhash/oldsource/log/raw. Không chạy lại native chỉ sửa checker.

Đã xem8rankshots+4Start3shots. B/C/D thấy bảng4tên/LOSE-PERFECT đổi đúng hai vector/zeroeconomy; **hai ảnh A vẫnBattle**,
không chứng minh Aresultpanelrender hoặc nguyên nhân timing. NativeA175/180/rankmemory/Ready-Start2-3 là proof riêng.
All4Start3ảnhclock01:53/correctnames/actorsalive; không pixelidentity/fullworldconsistency.

**M4 còn scoped host/guestEOF/unrankedabort→180/roomcleanup rồi tám**; fourresultflag không chọn two-forfeit,
tại CP248 activeEOF4 chưa có unranked180/native survivorroom proof; bổ sung scoped ở [CP249](#rust-four-abort). AUIrankcapturegap để đo muộn hơn trong plannedlifecycle,
không lặp hai cycles chỉ lấy ảnh. Nativeclear21/rescue/needle/terrain-inventory4/fullauth/generalroom/fullM4/production chưa đạt.
M5backend đã có, nativeownershipbridge riêng. AUD071 qua248/next252/Codexselfcheck/reviewwaived; checker không tự đóng gate.
CP247shutdown/Btrfsrootcause vẫnunknown/deadlinesgiữ; Internet thật chưa kiểm.

<a id="four-member-abort-room"></a>
### Bốn session: mất guest/host khi bom còn chờ, ba survivor về phòng — CP205

[Recipe/raw/hash](../evidence/peer-endpoints/README.md#cp205), [phần thiếu/phép thử](../RESEARCH.md#gap-lobby-identity-followup),
[thứ tự duy nhất](../PLAN.md#current-order). Source `kr_runtime` confirmed cho hai run riêng; luồng abort là `design`.

| Nguồn tái dùng / mức chứng minh | Dùng trực tiếp và phần khác biệt phải đo |
|---|---|
| CP022:180 ba byte `(0,0,0)` không cần175; CP034/174/196:107/slotclear/110 đã native. CP202–204: bốn slot/identity/585 và bundle | Dùng knownKRcodec/fixtures/VA; đo thêm ba consumer thực cùng run. Cslot2/ID3, Dslot3/ID4 theo fourprofile; không dùng legacyCslot0 hoặc rankAB |
| [VN oc_e-game/lobby](../REFERENCES.md): return→Room→setup→slot, `vn_runtime` trong nguồn cũ | Dùng kiến trúc pha; không mang VNopcode/offset/layout sang KR. Boundedread KRcũ không thêm proof fourEOF; không mở decompile mới |

Opt-in `--abort-room` cần `--battle-outcome`/bombprofile và returnfixture đúng3byte0/0/0.
Current loaded member EOF đóng/fence phòng, gửi180 không xếp hạng tới người còn lại; sau **585 mới của từng
recipient đã gửi180**, gửi107 sáu byte `[slot,u32ID,flag0]` cho departed và110 cho các survivor, Ready0,
chief theo slot nhỏ nhất. 585 là request setup quan sát được, **không authenticatedACK**. Chờ thiếu585 chỉ ghi
waitingroles sau15s, không block vòng lặp; không replay106 cũ khiến người đã rời xuất hiện lại.
Partialsend vẫn đóng đúng recipient/fence theo CP204. Giữ firstmembershipfailure và completedabortjob khi cleanup
đóng thêm connection; test bảo toàn lịch sử. Không bật mặc định hoặc mở Start với ba người.

- **GuestD:** bốn actualKR Wine copy4b/16TCP/private4virtualhosts/Free0/ExVillage01; DSpace→actual118→159 đủ4.
  Kill đúngD PID sau kiểm prefix/netns/startticks/exe còn5,723883969s trước fuse6. A/B/C native1803/3/errors0,
  actual585, 107base6/6→rewind0→end1/errors0, slot3ID4→0;9slot110 và ảnh Room đúng3tên/Achief.
  Quan sát15,567803164s sau fuse, pendingtimer=`round_closed`, không158 hoặc175.1258checks.
- **HostA:** run độc lập cùng topology/copy/profile, source cuối giữ ổn định completedjob/firstloss;
  ASpace/118/159→ownedAEOF còn5,730900202s trước fuse. B/C/D native180/107slot0ID1→0/585/9slot110/Room;
  Bchief getter`1240D5E=1`, C/D=0 và ảnh chiefB.15,216112527s sau fuse không158/175.1286checks.

Guestrun dùng source trước hai guard telemetry-cleanup; host và222TCP kiểm source cuối. Guestfinalsnapshot từng
appendA/Cdeparted khi cleanup **sau** observation; raw giữ nguyên, verifier phân biệt timestamp, không gọi đó là
lỗi gameplay trong cửa sổ đo. R1 lỗi harness absentdict `.get` trước connection đã giữ seal; R2 chỉ sửa default
kiểu dict, không clientpatch. Verifier đầu giả finalsnapshot chỉD đã thất bại; correction đọc closuretimestamp.

Giới hạn: abort phòng không là normal2v2win/forfeitlaw gốc. Toàn room vẫn fenced, chưa3memberStart,
freshmember/reconnect/accountresume/restart, native sendfailure/quota-roomrecovery hoặc full4item/terrain/kim/cứu.
Hai run abort không thay hai normaloutcomes CP202–204; K7/Internet giữ mở. Review waived; audit kếCP208.

<a id="rust-four-abort"></a>
## Rust bốn người: guest/host EOF về phòng không xếp hạng — CP249 / M4 partial

[Metadata](../evidence/rust-service/four-abort.json), [raw/hash](../evidence/rust-service/README.md#cp249),
[thứ tự tiếp](../PLAN.md#current-order). Tái dùng [CP205](#four-member-abort-room) và CP248; không thêm hàm native,
clientpatch hoặc wire VN. Contract consumer KR có nguồn; lựa chọn abort/fence là `design`, không luật server gốc.

`fixed_room.four_abort_room=true` cần `four_results` và profile bốn người/life/bombs. Exporter có
`--four-abort-room --four-results --four-life-reports --four-owner-bombs` cùng four-startup inputs.
Một member hiện hành mất khi Armed: fence ngay bom/pickup/life jobs, bỏ membership đó, chọn chief slot nhỏ nhất,
seal phòng không cho replacement hoặc Start ba người; gửi180 ba byte theo selector hiện hành tới ba survivor.
Không tạo175, deathreport hoặc thắng2v2/forfeit hai người. Mỗi180 phải ghi TCP xong trước585 của recipient đó;
585 chỉ setup quan sát được, không authenticatedACK/epoch. Đủ ba currentrecipient585 mới reserve toàn bộ queues,
gửi107 đúng `[slot,ID,0]` và ba110/Ready0 mỗi recipient; không replay106. Chỉ đủ ba refreshwrites mới complete.
Duplicate585 bị budget chặn và không thay recipient khác. Late work mang membership/round cũ không còn deliverable.
Nếu mất thêm member trước complete, đóng các channel còn lại, giữ firstjob; chỉ unit, chưa native đa mất kết nối.

Hai run riêng KR00967/Wine10.0-20260425/copy4b, currentManner1/fixed2v2, root+A–D netns/bridge/no default route:
D guest hoặc A host dùng OS Space→actual118→159 đủ4 rồi kill đúng ownedPID đã kiểm namespace/prefix/startticks/exe.
Cả ba survivor đọc1803/3/errors0, gửi585;107base6/6→rewind0→end1/errors0 và native ID4/slot3 hoặc ID1/slot0→0.
Wire9slot110/run và ảnh Room đúng ba tên; guestloss giữ A chief, hostloss Bchiefgetter1/C-D0 và UI BStart.
Hold ít nhất8s sau nativeRoom và11s sau kill, vượt fuse6; không158 hoặc175 trong trace. Giữ cùng16transports,
fulltuple/C2Ssequence/CRC/SHA joins; raw/export/hash và cleanup kiểm riêng, không suy từ alive/TCPaccept/logserver.

65Rusttests (60service+5wire),20guestTCP/20hostTCP,32normalfour và51twoTCP hồi quy,6prebind guards.
NormalTCPR1 còn pendingD158 khi quietlifecase nên thất bại; giữ raw/source và sửa bài kiểm đợi đúng158 đủ4 trước
lifeassertions. R2/R3 source/binary giữ nguyên nhưng exit1 vì logflush timeout events292/208; PIDgone/sourcefrozen,
R2 có threadwchan `reserve_bytes.cold`. Đây chưa là chẩn đoán causal Btrfs hoặc lỗi wire/EOF đã được chứng minh.

Đối chứng giữ nguyên Rustbinary/policy/2sflush/I/Ocleanupdeadlines, dùng tmpfs cho evidence sink:
32normalfour/51twoTCP đạt; raw chép về local và kiểm từng hash trước xóa temporarydir thuộc run.
Hai native run dùng **256MiB tmpfs mount riêng ở thư mục raw trong private mount namespace**; sau đóng mọi
Wine/service/capture, copy toàn raw, so size/SHA, unmount rồi đặt lại cùng localpath. `storage-export.json` pin từng file.
Storage là giới hạn môi trường được công khai; không đổi server logging, không che run lỗi hoặc chứng minh
Btrfs được sửa/durable logging/backuprestore/production. Kernelguards/malformed/wire proof vẫn tách khỏi native.

Chưa three-memberStart/replacement/reconnect/accountresume/matchrestore/generalrooms/nativequota-sendfailure,
loadingEOF/selectedEndingEOF/terrain4/rescue/needle/eight/fullM4. Không có normaloutcome trong hai EOFcontrols,
Aresultpanel screenshot gap CP248 giữ nguyên. M5backend có sẵn/nativebridge riêng; Internet thật chưa kiểm.
AUD071 qua248/audit kế252; Codex selfcheck, independentreviewwaived; checker không tự đóng gate.

<a id="four-portable-guards"></a>
### Guard input bốn người và bundle riêng đã chạy native — CP203

[Recipe/raw/hash](../evidence/peer-endpoints/README.md#cp203), [cách xuất/chạy](../../tools/README.md#four-player-bundle),
[gap kế](../RESEARCH.md#gap-lobby-identity-followup), [PLAN](../PLAN.md#current-order).
Reuse [CP202 report/result/reset](#four-two-team-cycles), [bundle hai người](rust-boundary.md#portable-probe-bundle) và CP201reservation;
không RE lại codec/127/rank hoặc coi profile hai người mặc nhiên là profile bốn người.

**design/confirmed trong kiểm source/TCP:** trước sửa, currentacceptedA19 rồiA118serialmới vẫn tăng wireID4→5/job0→1.
Guard mới chỉ khi opt-inbattle/currentgeneration có accepted19 của boundowner: reject`bomb_owner_reported_dead`
trước reservation/serial/ID/job/send. 247realTCPcontrols giữ0mutation ở rejection, B còn sống đặt được và A ở
newloadedgeneration đặt lại được; malformed127/sai slot/blob thiếu/extended21/duplicate/rollback không đổi
cursor/life hoặc phát peer/result. Newround vẫn giữ reportcursor, clear life/99/peer barrier; mặc định chưa bật
battlepolicy không thêm suy luận life. Đây không authenticatedlife hoặc originalserverauthority.

**Bundle four-client-battle riêng**, dùng pin47aliases/43fixturefiles của CP202, gồm knownfloor variant của profile4;
không đổi pin theo file đang có. APIexport_bundle dùng opt-infour_bomb_profile với đúng measuredpreset,
64runtimefiles gồm4service/policy/profile/launcher, hashmanifest. Launcher4 chọnfour_player_service,
gắn sẵnprofile+bombprofile+--battle-outcome; overrideprofile/policy, missingpolicyreference hoặc sai hash
bị từ chối trướclisten/output. Default2 vàpreset2 cũ giữ entrypointProbeService. Đây là integritypins,
không signedrelease hoặc networkisolation; client/assets/DB không đónggói.

**kr_runtime/confirmed riêng runR2:** actualservicePython-I chạy từ gói đã chuyển thư mục; khởi spawn vớicwd/tmp,
launcherchdir vào bundle. Tất cảpolicy_sources trongbundle/hashkhớp64pins, khôngimportsourcecheckout.
Cùng4KRWine4b/16TCP/4netns/ExVillage01/Free0 hoàn thành2normal2v2outcomes đổiđộithắng,175552inner/562outer,
1803/reset/Ready2Start2/Ready3Start3; cả4viewpre175life/ranks khớp, keys12345–12347,32nativegetteremulation riêng.
Không ghép các run; nativepositive không tự chứng minh maliciouspostdeathrequest, đã có TCPnegative riêng.

R1partial trướcBattle: C đi contentJSONthrow96A480/96A4EB/96E300,76.241s/309raw/cleanup0.
Một boundedrepeatR2 dùng byte-identicalbundle/source/copy/profile/policy, chỉ outputlabel/relocation khác đã đạt;
không gọi đây startupfix hoặc coldreliability, không sửa copy để ép qua, không sửa R1thànhpass.
Giới hạn4items/terrain/kim/cứu/fullauth/Internet/K7 và170Finishcursor0 giữ như CP202.
**Tại CP203 blocker kế còn modelrepro:** recipientA sendtimeout trong B17 lại closeB; [CP204](#four-transport-bounds) đã kiểm kernel/default2s và sửa. Khi đó chưa có kernelproof/fix. Tiếp đúng transportowner/bounds/rate-limit/restart/persistence/health theoPLAN,
không tự đóngK7 từ bundle hoặc hai vòng native.

<a id="four-transport-bounds"></a>
### Bốn người: recipient-safe transport, giới hạn I/O/rate và nativepositive — CP204

[Recipe/raw/hash/audit](../evidence/peer-endpoints/README.md#cp204), [gap kế](../RESEARCH.md#gap-lobby-identity-followup),
[PLAN](../PLAN.md#current-order). Reuse CP190/202/203 cùng codec/framebound65536, report17/19/21,
4profile/47aliases/43fixtures/2v2result/reset; không RE thêm nativefunction hoặc đổi wire/clientcopy.

**design/confirmed bằng source/model/realTCP:** baseline kernelqueueA32768byte sau inertfiller làm B17broadcast
sendtimeout0,2s rồi outerrun close nhầm B. Fouradapter nay bọc accept/receive/tick bằng recipient-aware exception;
đóng đúng recipient, fence room, không commit cursor/life của report gửi dở. Metadata giữ các recipient đã gửi,
`peer_delivery_incomplete` và failedconnection; không rollback byte đã tới peer. ActualTCP defaulttimeout2s
(đã đo gettimeout/restored) kết thúc khoảng2,01s: A đóng, B còn gửi một request kế được xử lý, Cpendingbom
`round_closed`, không result phụ. Ca bổ sung218checks đọc lại48byte170 ở hai TCPdriverD/C, CRC/seq/hashkhớp; không suy nativeconsumer hoặc broadcastatomic.
Unitcontrol bổ sung Cfail sauAdelivered vàtick158failure/pendingsecondtimer. Đây không queuednonblockingbackpressure,
không benchmark tải/Internet hoặc proof nativeclientabortcleanup.

**Budgets explicit emulator/design, không luật KR:** áp mỗi transport trong process lifetime, không reset theo Battle:

| Giới hạn | Giá trị/ý nghĩa |
|---|---|
| C→S raw / logchunks |1MiB /4096chunks; quá giới hạn đóng đúng connection, log reason/hash thay vì append tiếp |
| S→C attempted raw / sendcalls |2MiB /4096calls; đã attempt không hoàn quota vì kernel có thể gửi một phần |
| Chunk và decoded-frame rate |hai bucket độc lập, burst128/refill32/s; coalesced129frames không lách chunkbucket; không minttoken khi clocklùi |
| Admission từ source lạ/exhausted |giữ tail256 và totalcounter, không tăngevents/session/rolecounts |
| Parser |bound65536wire có sẵn; không nâng thành fullKRgrammar hoặc mặc địnhbufferclient1MiB |

Các quota hữu hạn dành demo fixed4/16transports; chạm lifetimequota cần processlifetime mới, không accountresume.
335servertests kiểm boundary/reject0countermutation/refill/independence/partialsend/fence/coalesced129/
tiny4097chunks/outputquota/admissiontail;214realTCPdefault2snonreader và247guard controls riêng.

**Integrity bundle:** kiểm cũ chỉ các row được khai có thể verify-onlypass khi bỏ pin của một source .py còn hiện diện.
Đối chứng copy đã giữ; launcher nay từ chối mọi server/*.py không có trong manifest trước importserver.
16bundletests/default2controls có positive và missingpinnegative;65files của gói mới chứa thêm budgetmodule.
Hash là integritypins, không signature/authentication.

**kr_runtime/confirmed riêng normalpositive:** gói65pins/Python-I/relocated/allpolicy_sourcesinbundle,
cùng4KRWine4b/16TCP/4netns/ExVillage01/Free0 hoàn thành hai normal2v2outcomes đổi đội thắng/175552inner562outer/
1803/reset/actualReady2Start2/Ready3Start3. Không budget/deliveryrejection trongrun; maxobservedpertransport
input8284bytes/99chunks/output6286bytes/64calls.32pre175securegetters kiểm lại native52A350 trongemulation riêng;
2552checks/cleanup0/sourcefrozen. Không dùng normalpositive thay maliciousTCPnegative hoặc ngược lại.

**Còn thiếu:** roomfence hiện chưa tự đưa ba nativepeer còn lại về phòng sauEOF/sendfailure/quota.
Tiếp actual4pendingownerEOF→three-survivor180/107/slotreset/nativeUI, không cloneABrank/giả normalwin;
sau đó health/runidentity/persistence/restart theoPLAN. Full4items/terrain/kim/cứu, accountauth/Internet/K7 vẫn mở.
[AUD060](../evidence/checkpoints/audits/AUD060.json) sửa stale currentheader/gap2v2/counter/catalog, không đónggate.

<a id="four-health-identity"></a>
### Health local, correlation và identity pin tối thiểu — CP206

[Recipe/raw/hash](../evidence/peer-endpoints/README.md#cp206), [phần thiếu](../RESEARCH.md#gap-lobby-identity-followup),
[PLAN](../PLAN.md#current-order). Phần này là `design` của emulator; nativepositive riêng bên dưới là `kr_runtime`.
Tái dùng `ProbeService.save/run`, existingpolicy_sources, fixedfourprofile/rolecounts, currentround, CP204budgets,
CP205firstloss/completedabort; không RE thêm hàm native, không nhập opcode/offset VN.

Fourservice tự ghi `run-metadata.json` một lần và `health.json` bên cạnh `service-state.json`:
UUID mỗi process; session`run/role`, transport`run/tcpOrdinal`, fixedroom`run/room1`, round`room/roundIndex`.
Correlations gắn vào transport/currentround và bomb/request/report có index khai báo; row thiếu index giữroundIDnull,
không gán ngầm round hiện tại. Runmetadata pin digest profile/config và danh sách sourcepolicy đã tải;
**source_build_sha256 là digest declaredpolicy_sources**, bundlelauncher được pin riêng trong bundlemanifest.
TargetKR00967 chỉ là khai báo, không attestation binary từ peer/accountauth.

Healthphase`starting/ready/active/degraded/stopping/stopped`; active nghĩa roomstateactive, không tự chứng minh gameplay.
Ringphase32+totalcounter, heartbeatmonotonic/sequence, số accepted/open transports/errors/roomstate.
State vàhealth mỗi file replaceatomic; **cặp hai file không là một transaction**, reader phải match sequence/hash
`state_sha256`. Healththam chiếurunmetadatahash bất biến. Không thêm HTTPlistener. Thư mục đã có runmetadata bị từ chối,
không đè lịch sử. Fourrunloop dùng existingselector/accept/receive/tick, ghi stopping rồi đóng owned sockets/listeners/
selector/fence trước finalstopped; lỗi healthwrite vẫn cleanup qua finally. Test faultIO kiểm resourceclose, không gate.

Opt-in `--identity-store PATH` lưu **đúng khai báo cố định A/B/C/D:ID1/2/3/4,slot0/1/2/3,name/team của fourprofile**.
Publishfile đầy đủ bằng exclusivelink từ temp cùngparent; racecreator cùngdata chỉ1winner. Existingbytes phải khớp
canonicalschema/pin, đọc tối đa16KiB; sai/truncated/oversized/conflict từ chối, không tựrepair/overwrite. Parent do operator
chuẩn bị. Đây là consistency/persistence qua **processrestart**, chưa accountdatabase/credential/resume/trận đang chơi
hoặc durability khi powerloss. Bỏflag thì identitystoredisabled, wireidentityfixedprofile vẫn theo contract cũ.

352servertests gồm11health/identity mới,16bundle/export,43checkpoint; 240realTCPsyntheticrequest/phase/hashcontrols.
Hai processPython thật/listenerloopback/19checks: SIGTERM→stopping/stopped/exit0/ownedcleanup; processmới có run/roomIDmới,
transportcounts0/budgets0, giữ nguyên identitybytes/hash vàsourcepolicy digest. Không native reconnect/matchrestore.

Native runcurrent67pinbundle/Python-I/relocated: cùng4actualKRWine4b/16TCP/4virtualhosts, hai normal2v2outcomes
đối nhau/1754rows/180/reset/Ready2Start2/Start3;2202gameplaychecks+208healthcorrelationchecks+32copiednativegetters riêng.
Identitypinprecreated được đọc nguyênbytes; metadata nguồn đều dướibundle. Nativephase`ready/active`ba lần, rồi cleanup
`degraded/stopping/stopped`; không delivery/budgetreject/errors trongrun. Starting/degradedpolicy có unit/TCP controls riêng,
không suy nativeca mà run chưa đo. VerifierV1 nhầm đòi2500checks vì số frozenfile khác; đã giữ fail/source và thay bằng
contract hai winner/key cụ thể, không dùng số checks làm mức tiến độ hoặc tự xác nhận ngữ nghĩa.

K7/Internet giữ mở. Còn nativefreshserverrestart dùng lại sameidentitypin + bốn-peer UDPimpairment theo PLAN;
fullaccountresume/3memberStart/4items/terrain/kim/cứu và8actualclients chưa được nghiệm thu.

<a id="four-netem-restart"></a>
### Samebundle/store, freshserver và bốn peer dưới UDP delay/loss — CP207

[Recipe/hash](../evidence/peer-endpoints/README.md#cp207), [K7đốichiếu](../PLAN.md#k7-evidence-review),
[câu hỏi kế](../RESEARCH.md#gap-lobby-identity-followup). `kr_runtime` confirmed trong profileđãđo; tham sốnetem/identity là`design`.
Reuse CP195UDP-onlyTC/CP206fourtopology/nativeoracles, **chính bundle67file vàidentitystore cũ**, không exportpackagekhác.
VNoc_e-game475–486 chỉ tham chiếu kiến trúc TCPtraoendpoint/P2P; không92/146/VNlayoutport hoặc hàm native mới.

Control4host riêng, mỗi case trong namespace mới/configcốđịnh trướctraffic: UDPbaseline768/768,
loss100%0/768, delay40±5ms/loss5%668/768; mỗi case48TCPecho đủ/median0,10–0,18ms. UDPbatchmedianbaseline~35,6ms,
delayloss~80,3–80,9ms **có đợi hếtbatch/đọcqueue**, không purewire/nativeping.118checks; counterfresh0 mỗi case,
case100%192drop/queue, case5%25drop/queue; không lẫn counter qua phase. Transportcontrol không KR/Internet.

Nativefreshserver sauCP206cleanup, giữ exactbundle/profile/config/policy digest/identitybytes-path-hash; newrunUUID/newroom,
16freshTCP/ordinal1..16/4stage mỗi role. Freshclientprocesses, không nối lại account/trận cũ.
Bốn recipientvethkv-a/b/c/d: rootprio/priomapall0/protocol17→netemparent1:3/delay40±5ms/loss5%seed194;
TCPbaseline,12add trước firstnativeTCPaccept, không đổi qdisc trongrun. Hai normal2v2outcomes đối nhau/1754rows/
180/reset/Ready2Start2/Start3:2200gameplay+208health+2528network-restartchecks/32getters riêng.
8OSmovementwindows/hai round có24directedremotecurrentcoordinateprogress; không thêmE668AD/key/ACKprobe.
Queue drop69/70/69/69/delayed1194/1210/1179/1181/backlog0; không perpacketdropattribution.
PCAP2412records ởkv-a đủ6A-edges/cảhai epoch, **không12Ethernetedges**; position24chiều là nguồn khác.

Currentsource cũng qua214realTCPkernel-nonreader/default2s/Bfollowup1/pendingfence/partialreportnocommit vớiFourrunloop mới.
VerifierV1 thiếu aliasb khi trỏ bundlecũ; giữscript/log/dependenterrors, chỉ sửa readerpath, không rerun/patch native.
OldCP200–206controller có textNATscaffold unused/stale: actualfourtopology làdirectbridge theo virtual-network.json;
CP207 sửa textscope, rawcũ giữ nguyên, không coi fourruns làNATproof.
CP206PIDlocator có câu nhápR1/R2: sourcechuẩnR2process-verification.json làPID1145289/1145292;1143576/1143577 thuộcR1.
Đính chính tạiCP207, không sửa completerecordCP206.

Giới hạn: privateLAN/sharedkernel/owneracceptedWine; không Internet/NATdiscovery/genericlossdistributions/accountresume/
3memberStart/full4items-terrain-kim-cứu/8actualclients. K7 giữ mở theo dependencyK6c; Codex đối chiếu thủ công,
script/counts không tựđónggate. Audit kếCP208, reviewwaived.

<a id="eight-session-preview"></a>
### Tám slot native và adapter tám session — CP208

[Recipe/raw/hash](../evidence/eight-player/README.md#cp208), [phần còn thiếu](../RESEARCH.md#gap-roster-ready-host),
[thứ tự duy nhất](../PLAN.md#current-order). Không đổi gate hoặc coi roster tổng hợp là tám client.

| Nguồn tái sử dụng | Mức chứng minh đã có | Delta phải kiểm |
|---|---|---|
| [CP147/163](#battle-eight-roster), CP020/026/036 và [CP199](#four-slot-native-consumers) | KR runtime: capacity8/Ready/teamguard; KR121 prefix48/row26, KR153 26 flags/ba tuple mỗi slot; bốn actor trên hai copy | Giữ nguyên layout KR; chỉ tăng count/row và active slot0–7. Không tìm lại codec/capacity/guard |
| [CP200–207](#four-session-native), current four profile/service/health/bundle | Bốn actual KR và controls; mỗi client cần **bốn transport**, auth reply4/u32 width4 không là số người | Parametrize roster/role/identity/Ready/99barrier; 32 transport tổng cộng. Không bật policy bom/rank4 cho tám |
| VN `oc_e-game.md`475–486, room/lobby dẫn trong [REFERENCES](../REFERENCES.md) | VN runtime cho kiến trúc TCP trao endpoint và P2P | Chỉ dùng kiến trúc; không chuyển opcode/offset/ID/layout VN |

**KR runtime đã xác nhận hẹp:** hai actual Wine A/B/currentcopy`4b781fb…`, sáu slot tổng hợp C–H.
Hai client đều đọc KR121 **285/285**, inner **256/256**, KR153 **205/205**, errors54/58=0;
ID1–8/slot và position0–7 qua validation, tám actor pointer khác0/khác nhau ở mỗi clock.
Sáu endpoint tổng hợp bằng0; không phải tám login/P2P/gameplay. 1709 kiểm raw/pin/native;
48 wrapper life/position từ clock khớp **copied native52A350/Unicorn**, tách khỏi quan sát runtime.
Run287,0916s/exit0/1376 raw/285 frozen/cleanup namespace0; điều kiện earlycompletion của controller
nhầm adjacency JSON nên hết bound, không thay input giữa run hoặc suy clock là normal match.
VerifierV1 đọc source hiện tại sau khi adapter được sửa nên báo drift; V2 đối chiếu đúng bản
`runtime-source/` đã pin trước run. Không sửa preflight/seal/raw hoặc tạo lại bytes cũ.

**Implementation/design:** opt-in `server/eight_player_service.py`, profile`eight-client-start` và bundle;
A–H/ID1–8/slot0–7/team0/1 xen kẽ/name5hoặc7byte là lựa chọn operator. Reuse four service qua profile
riêng, default ABCD còn kiểm hồi quy. Sources bound IP riêng, bốn transport/role, sequence độc lập,
seven remote106/153/selftuple0, đủ bảy Ready và tám actual99 mới mở barrier. RealTCP **487 checks/32 sockets**
trên adapter mới; negative unknown source/exhausted fifth transport/wrong name/early stage/malformed Ready/
missing lastReady/missing eighth99/zeroUDPport. Đây là C2S tổng hợp, không oracle native.
Eight preview từ chối bomb/outcome/abort-room/identity persistence policy; không giả rank hoặc cap E–H.
Gói68pins/47alias qua Python-I/verify/help; integrity không chứng nhận runtime hoặc chữ ký.

**KR runtime mới — tám client thật:** `eight-native-r2` chạy152,435188146s/exit0/870raw/276frozen/cleanup0;
Wine10.0-20260425/currentcopy4b, tám Wineprefix/display và CA PID riêng, chín netns root+A…H/IP10.77.0.2…9/
privatebridge/no default route. 32nativeTCP/fourstage mỗi role, actual291/name với cursor126(C)/128(còn lại),
ownslot0–7. Bảy Ready/8actual99 trước firstGO; cả8view đọc121285/inner256/153205 không lỗi, validation8ID/
position và8distinctactor/currentclock. 1468raw/nativejoins;192copiednative52A350 kiểm life/position riêng.
Ảnh tám view bổ sung callback/owner/raw; không dùng screenshot hoặc service log thay native proof.
RunR1 fail preclient topology assert9 trên list5, rồi finally unbound sync_service; raw/seal/namespace0 giữ riêng.
R2 chỉ sửa helper validation đủ9namespace và safe cleanup default trước service; production source không thay giữa run.
Service trực tiếp từ source snapshot; gói68pins mới chỉ được verify/help, **chưa native run trên gói8 relocated**.
Không fullmesh movement/bom/kết quả/reset8/1–7 actual/fullauth/Internet/K7 từ hai-client probe hoặc 32 TCP.

<a id="eight-mesh-owner-bombs"></a>
### Tám client: movement mesh, baseline stat và owner confirm/nổ — CP209

[Recipe/raw/hash](../evidence/eight-player/README.md#cp209), [khoảng trống](../RESEARCH.md#gap-roster-ready-host),
[thứ tự](../PLAN.md#current-order). Reuse [CP208](#eight-session-preview) tám actualclock/privatebridge,
CP200 lowobserver3wrapper/OSinput, CP201 rareclockstat/118/15939/15813/nativeentitycoords/fence;
[CP031](items-stats.md#item-effects) đã sửa power118.field10→159.field8. Không codec/Room8/teamguard/functionRE lại.
VNoc_e-game chỉ định hướng TCP/P2P và luật đội; không port opcode/ID/layout/capacity.

**KR runtime: mesh/stat/request, run riêng trước bật bom.** Cùng tám actual Wine4b/prefix/display/CA/netns/IP;
8OSinputwindows ABC→Right,D/H→Down,E/F/G→Left theo spawn/corridor. 64own+remote currentcoordinateprogress,
trong đó **56chiều remote** có baseline sau mởwindow/trước input và sample tăng/giảm đúng hướng trongwindow.
Observer chỉ đọc wrapper hiện tại, không probe E668AD/key-handler từng frame; không đo latency/gametick.
PCAP `kv-a` có2733record/**14chiều liên quan A**, không wirecapture toàn56; nativeprogress56 là nguồn khác.
Eightactual118 sau8Space có đúng currentkey/ownslot/power, preview từ chối confirm ở run này.
4610checks/704copiednative52A350/184,016344755s/891raw/277frozen/exit0/cleanup0.

Tám view đồng ý baseline hiệntại dưới đây; `kr_runtime/confirmed` **chỉ tám profile/character đang chọn**,
không mọi character/mode/runtimeatmax hoặc luật server gốc:

| Slot/role | Capacity/max | Power/max | Speed/max |
|---|---|---|---|
|0/A|1/10|1/7|5/7|
|1/B|2/7|1/9|4/8|
|2/C|1/8|1/5|5/8|
|3/D|1/10|1/8|4/8|
|4/E|2/9|1/6|4/8|
|5/F|1/6|1/7|5/9|
|6/G|1/6|2/7|5/8|
|7/H|1/9|2/8|4/8|

**Implementation/design:** `EightBombAdmission` kế thừa admit/expire đã kiểm; chỉ constructor fixedroles8
thay4, default`FourBombAdmission` vẫn4. Opt-in `--bomb-profile` ở EightPlayerService chỉ nhận đúng
`eight_current_baseline`: capacity1/2/1/1/2/1/1/1,power1/1/1/1/1/1/2/2,fuse6s. Không đoán E–H giốngB.
Reservation/placementbounds/dedupe/owner-slot/key/generation/EOFfence/fuse/broadcast là emulatorpolicy,
không authenticatedworldauthority. Default preview không bật bom; report/outcome/abort-room/identityresume vẫn tắt.
672realTCPsyntheticchecks/32sockets: đủ8recipient159 và158/ownerpower2/BEcapacity2/F1/duplicate-noID/
wrongpower/fencependingH khi C EOF; 360server và16bundle-preset tests qua. Healthmetadata pin eightbombpolicy.

**KR runtime: tám owner đặt/confirm/nổ, run khác sau controls.** Mỗiowner normalSpace rồi đợi7s;
actual118→ID1…8/serialowner→broadcast159 tới8view→entity khác0/correctcell→broadcast158 saufuse6s.
Cả8view đủ8confirm **39/39/errors0** và8blast **13/13/errors0**;64entity cùng tọa độ request,
128wrapper tọa độ khớp copiednative52A350. 2256joins/241,599624124s/1107raw/687frozen/exit0/cleanup0,
source/config freeze tới seal. G/H actual118/wire159 echo power2; không suy native raylength8 từ cursor,
exactblastentitylookup/powerconsumer8 chưa đo lại. Không dùng probeE14360entryEBP sai của CP201;
run mới bỏ site đó, giữ nguyên raw/report cũ.

Giới hạn: một epoch/fixedFree0/ExVillage01/current8profiles/privatebridge/Wineaccepted, không ghép hai run thành
một trận nghiệm thu. 127 vẫn bị từ chối; chưa life-forward8/reportguard/rank175eightrows/180/reset/Start2/
haioutcomes/1–7actual/reject9native/8portablebundle-native/Internet/K7. Tám bom đã nhận/nổ không là trận hoàn chỉnh.

<a id="rust-eight-baseline"></a>
## Rust tám người: baseline Manner4v4, mesh/bom/life/kết quả/reset — CP250 / M4 partial

[Metadata](../evidence/rust-service/eight-baseline.json), [raw/hash](../evidence/rust-service/README.md#cp250),
[thứ tự tiếp](../PLAN.md#current-order). Reuse [CP208](#eight-session-preview)/[CP209](#eight-mesh-owner-bombs)/
[CP210](#eight-two-team-cycles), sharedRustCP245–249; không RE thêm hàm/VNwire/currenteightWIP import.

Config tám diagnosticactors ID1–8/names/slots0–7 cố định; `eight_battle=true` bật known8bom/life/result,
startup producer `expand_eight_startup` giữ baseline hai/four độc lập và không import moduleeight đangWIP.
Exporter `--eight-startup-derived-dir` +8`--eight-startup-peer`; `--eight-battle` riêng. Fourflags/legacyBattle2/terrain8reject.
Known121285/rowinner256,153205/26slots/8active/selfzero/current99IP-port; đủ8Ready-ACK mới GO.
Queue mỗi8connection16 đủ10startupbatches, localpending16; hai/fourqueue8 giữ. Kernelburst585 phát hiện guard8cũ,
đã sửa cóđối chứng. Bombledger unique2/4/8recipient, life unique1/3/7membership; duplicate/wrongvisitcompletion
không nhả peer khác. Reserve toàn queues trước publish và đủ mọi170writes trước Ending; không TCPwrite=clientACK.

Stats current8profile theo CP209: capacities1,2,1,1,2,1,1,1; powers1,1,1,1,1,1,2,2; speeds5,4,5,4,4,5,5,4.
Native maxstats cũng được lưu trong clockwrappers; đây không mọi character/mode/maxcap. Fuse6/placement/reportauthority
là design. Reread sealedCP209C2S/CRC/seq cho142aux33/37/39/47/-1 và167sizes4/22/42, chỉ boundedobserve/noeffect,
không reveal/inventory/ACK. Terrain/item/kim/rescue8 không tự bật.

Outcome exactlyonealiveTeam/current21witness sau latestdeath-blast/settle8/bombdrain/all17-19-21recipientwrites.
Rank16header+8×131active+8empty=1072inner/1082outer; hai actualsealedCP210bodySHA là wire regression,
không encoder tự làm nativeoracle. Winningteamrank1/loser2/economy0, kể cả teammate đã chết; không all-dead/tie8.
175→5s1803→3s64slot110/Readyreset/cursorretained; đủ7Ready mới và8native99 mới Start2/3/key mới.
Eightloading/ArmedEOF fencejobs/sealroom/107110 không tạo175180; **chưa native bảy survivor về phòng**.
Không clone fourabort-room hoặc twoforfeit thành8.

**Native cùng8client** KR00967/Wine10.0-20260425/copy4b/private8netns+rootbridge/no default route/sharedkernel,
currentManner1/fixedalternating4v4/ExVillage01: ID/name/slots thật, 8actual99 (giữ H3s khi đủ7khôngGO),
121285/153205 đầy đủ/errors0;8OS.25s movementwindows có56remoteprogress/freshbaselines theo nativeprotectedcoords.
Mỗiowner actual118 đúngslot/key/power→64native159entity/full39→64native158/full13/lookupkind0-cleanup; stats khớp8views.
Hai outcome trong cùng32TCP/process/channels:

| Vòng/key | Actual accepteddeath/nativeall8pre175 | Rank theo team và reset |
|---|---|---|
|1/12346|A/C/E/G tự đặt bom; D chết thêm do bom lan: `[1,0,1,1,1,0,1,0]`;70peer170|Team1 thắng `[2,1,2,1,2,1,2,1]`; Dđãchết vẫn rank1 theo design team; full1751082/1072/16slots/1803/reset64|
|2/12347|B/D/F/H: `[0,1,0,1,0,1,0,1]`;56peer170|Team0 thắng `[1,2,1,2,1,2,1,2]`; full175/180/reset64/7Ready-Start3|

Không ép R1 thành vector CP210 cũ hoặc bỏ D khỏi proof. Mỗi accepted17/19 tới đúng7peer/native56→1;
cả8nativepre175state khớp actualdeadset và đúng một aliveTeam. Native175/180 fullincallback/rankmemory riêng;
Finish170cursor0afterreturn không fullparseproof. Key12348/Start3:8clocklần3, sau6s freshreadonlyall8state0.
5.019TCP/912exactreply/fulltuple-C2Sseq-CRC-SHA/156raw/384,236496429s/sourcefrozen/cleanup0.
53.616copied52A350checks riêngUnicorn/counterreset; không wholebinaryoracle.
Additionalcontrols xác nhận8nativeauth/291-render/roster-ownslots và3actual8ACK-beforeGO. UDPcapturer còn assertIP2..5,
threaddừng ởE: chỉ3frame A→B/C/D đã lưu/index3rows/launchAssertionError. **UDPcapture partial**, không14hoặc56wireedges
hay nativeUDPAPI-key/applyproof;56currentnativecoordinateprogress vàfullTCPcapture là proof độc lập. Không suy
exit0/sourcefrozen thànhcapturecomplete. Sửa UDPcapture trong plannedteamvariantnativecontrol, không lặp2cycles chỉcapture. Đã xem24ảnh qua3contact sheets:
all8boards đủ8tên/twooppositevectors/zeroeconomy vàStart3clock01:53/8actors/ownarrows; Aboard hiệnởrun8này,
không backfill ảnhfourCP248 hoặc pixelidentity/worldauthority.

**Failures/provenance:**69Rusttests/16eightTCP/5prebind. TCPR1 thiếu loginredirect đúngguard;R2queue8startupfull;
R3bombledgerroster2/4còn sót; giữsource/raw/export/PIDgone, sửa cónguồn trước runtime, R4synthetic16pass.
NativeR1total540 vượt inheritedhardcap480, argparseexit2 chưanamespace; R2nativeconfig lấy nhầmTCPephemeralports,
controlledSIGINT vào existingfinally/67raw-seal-cleanup0/captureguardfourIPs vàbaselinepredicate4 chưamở8.
R3H vàR4G đóng channel sau17/trước544/CALauncher, raw67/run/frozen/cleanup0;37byte17 khớp CP210H.
R4privateHcopyfromfourD không giải nguyên nhân. **Correction AUD072:** R5 đạt với per-role token13 ký tự,
giống actualCP210; scalar token32 trong recipe không được dùng khi launch. Claim “khôi phục per-role32” trước đây sai
([đối chiếu frozenconfig/run/launch](bootstrap-codec.md#argv-provenance-correction)); không kết luận argvchange hoặc causalSDKfix.
Không patch client mới/đụngbinary-assetsnguồn hoặc mở SDKVMgraph.

VerifierR1 giả onlyintendedlosersdead, R2 dùng markerFOUR_SLOT thayEIGHT_SLOT, R3 shadoworigin-slot trong
snapshotloop; giữoldversions/failurelogs/retainedpre-run verifierhash/raw. R4 sửa kiểm dựa actualaccepteddeathset,
đủ intendedowners/everycurrentrecipient/nativeall8states/rankbyteam và56remoteprogress, không rerun native.
Private256MiBtmpfs evidence sink/exportallsizeSHA sau ownedWine/service/capturecleanup; total480s,
I/O/2sflush/cleanupdeadlines giữ. Btrfsflush faults CP249 vẫnunknown; không durability/load/productionacceptance.

M4 còn RustfixedFree1v7/selectorpersistence/Mannerunbalancednegative vànative9admissionbridge/consumer trước
đối chiếu acceptance; unknownIP/socketreject không đủ9thnativefullroom. Không interactiveC2Steamchange,
generalrooms/native8EOF/fullitems/rescue/needle/auth/resume/fullM4. M5backend có sẵn/nativebridge riêng;
AUD071through248/next252/reviewwaived/checker không đóng gate. Internet thật chưa kiểm.


<a id="eight-two-team-cycles"></a>
### Tám actual client: hai kết quả 4v4, về phòng và Start3 — CP210

[Recipe/raw/hash](../evidence/eight-player/README.md#cp210), [khoảng trống](../RESEARCH.md#gap-roster-ready-host),
[thứ tự](../PLAN.md#current-order). **KR runtime/confirmed trong scope cốđịnh dưới đây**, không tựđóng K7/Internet.

Source-first reuse [CP202](#four-two-team-cycles) report17/19/21→170/guard/deadowner118/freshwitness/
drained-settle8/rank16slot/return180/reset; [CP209](#eight-mesh-owner-bombs) actual8profiles/mesh/ownerbom.
VN `oc_e-game.md`397–425 cho kiến trúc kết quả/đội/reset, không168/173/VNlayoutport; KR175 đã có header16/
flag1row131/flag0byte1/header9FF/16records. Raw8bom CP209 đã đủ17+19 của8owner: đọc lại và kiểm envelope,
không runtime lại để tìm schema. Copiednative175header+16recordparser nhận hai vector8rank đầy1072byte,
state/allocator mô hình và skip managerpostparse: **emulation riêng**, chưa là nativecallback.

**Implementation/design:** fixedEightBattleReports kế thừa guards/prepare/commit/outcome, fixed4v4team0/1 xen kẽ;
Four defaultABCD/2v2 giữ nguyên. `--battle-outcome` ở EightPlayerService cần measured8bombprofile, bool rõ;
defaultpreview/bomb-only vẫn có. Report17/19 forward đến7peer, khôngself; chỉcommit sau đủsend. Duplicate/
rollback/death-no-trap/afterdeath/earlyphase/deadowner118 và pendingfence tái dùng. Một đội còn sống,
không bom chờ, settle≥8s sau lastdeath/lastblast và fresh21 của survivor mới chọnrank; đây là **report-derived
emulatorpolicy**, khôngauthenticatedlife/originalworldauthority. Metadata config ghi outcome bật/tắt.

Rank8 giữ16slot: header16 +8×131active +8empty=**1072inner**, outer**1082**. Rank1 cho đội thắng/rank2
đội thua; economy0/featureoff là design. Sau5s gửi180/payload3 tới8client, rồi64slot110/resetReady;
Start2/3 cần7Ready mới và8actual99/barrier/key mới. 767realTCPsyntheticcontrols/32sockets có7peer170/
rank8/1808/refresh64/cursorretention/Start2beforeloadnegative/EOFpendingHfence;363server/16bundle pass.
Không dùng serializer tựsinh làm oracle runtime.

**Một run với cùng8actual KR Wine**:32TCP/tám prefix/display/CA-wineserver/IP/netns/privatebridge/no externalroute,
currentcopy4b/Wine10.0-20260425/ExVillage01/Free0. Hai round liên tiếp:

| Epoch/key | Native input/life | Native kết quả/reset |
|---|---|---|
|1/12345|NormalSpace A/C/E/G; actual12717→19 mỗiowner→7peer170/nativeprotected56→1; survivors B/D/F/H cófresh21|Winnerteam1; mọi view pre175state`1,0,1,0,1,0,1,0`,rank`2,1,2,1,2,1,2,1`;1751082/inner1072 và1803 đủ/errors0;phòng8slot/7Ready2/Start2|
|2/12346|NormalSpace B/D/F/H; actual17→19→7peer/native56→1; survivors A/C/E/G cófresh21|Winnerteam0; pre175state`0,1,0,1,0,1,0,1`,rank`1,2,1,2,1,2,1,2`;175/180 đủ;reset8slot/7Ready3/Start3|

Key12347/epoch3: cùng8view clock lần3; **không thirdoutcome**. Hai normalresult khôngabort/forfeit;
không ghép run khác. Các death xuấtphát từ selfbomb bằng OSinput cóchủđích, không actor/life writes hoặc
functioncall từ observer. Client/server source/config freeze tới seal/cleanup.4351raw/nativejoins và128
pre175state wrappers khớp copiednative52A350/Unicorn riêng;364,941389647s/2099raw/937frozen/exit0/namespace0.
PeerFinish ởreturn nhìn cursor0; **không** dùng nó làm full170parseproof. Nativeafterstate56→1 vàpre175
statevectors độclập;175/180 cóprobe tronghàm để kiểmcursor. Ảnh rank8tên/2vector/phòngreset/Start3 bổ sunglogs;
Start3 cótutorialoverlay, không suy popup là inputfailure hoặc round3 đã kết thúc.

Giới hạn: fixed4v4/Free0/oneExVillage01/8profiles/Wine/privatebridge/sharedkernel, no8items/terrain/kim/cứu/
1–7actual/Manneractual8/reject9native/generalroomauth/accountresume/nativeportable8/Internet/K7.
K5/K6a/K6b scope đã đạt giữ nguyên; không nânggate vì thêm8cycles. Bước kế theo PLAN là portable8/admission
và biến thểđội cótrigger, không quét lại toànclient.

<a id="eight-portable-battle"></a>
### Gói tám người đã relocation, hai kết quả/reset và admission có giới hạn — CP211

[Recipe/raw/hash](../evidence/eight-player/README.md#cp211), [cách xuất/chạy](../../tools/README.md#eight-battle-bundle),
[phần còn thiếu](../RESEARCH.md#gap-service-hardening), [thứ tự](../PLAN.md#current-order).
Reuse CP203portable4/CP2088startbundle/CP209exact8baseline/[CP210two4v4](#eight-two-team-cycles);
chỉ đo delta packaging/pinflags/relocation, không codec/report/rank/teambalanceRE lại.

**Implementation/design:** preset`eight-client-battle` giữ nguyên47fixturealiases/profile của`eight-client-start`,
thêm exactmeasured8bombprofile. Exporter vẫn pin/validate input, refuse overwrite và symlinkescape;
launcher EightPlayerService inject fixed`--profile`, `--bomb-profile eight-bomb-profile.json`, `--battle-outcome`.
Operator không override ba flag, sidecar/filedrift/missingrequired/unlistedPython bị chặntrước listen.
Profile khác hoặc Bcap2 ápnhầmE/GHpower1 bị từchối; test so presetbaseline với compiledBASELINE.
18bundle/preset tests qua; code server không đổi, dùng lại363server của CP210 với sourcehashmatching.

Gói mới **71pins/47aliases**, di chuyển từ`export/service` sang`relocated/service`, thưmục xuất cũ không còn.
`python -I .../run.py --verify-only` và767realTCPsyntheticcontrols chạy từmodule **tronggói** đã chuyển chỗ.
ProfileJSON so CP210 chỉ khác pathfixture, mọi byte47alias/cácfield còn lại bằng nhau. Policy-sourcehash,
config/identity/fuse/outcomeflag/io budgets đo trongnativeprocess **trùngCP210**; loaderprint modulepath
ởgói, actualspawnargv là Python-I/relocatedrun.py, không externalprofile/bomb/outcomeflags.
71filepins được đốichiếu trước và saurun. Integrity là localfilepin, không signature/remoteattestation.

**KR runtime/confirmed hẹp:** cùng8actualWine4b/32nativeTCP/private8hosts/Free0/ExVillage01, **service từgóiđãrelocation**
qua hai normal4v4outcomes ngượcđội/7peerlife/native56→1/rank8/1751082-inner1072/1803/phòngreset/
7Ready2Start2/7Ready3Start3 và3clock mỗi view.364,775602329s/2102raw/1000frozen/exit0/cleanup0;
4417gameplayjoins +112delivery/path/hashjoins,128pre175copiednative52A350 tách riêng.
Không ghép source-run CP210 thành packagedrun; nativepacket/state/175180 của chínhrun mới đủ như CP210.

**Admission:** packaged32TCPcontrol source`127.0.0.99` ngoài8operatorIP bị close mà khôngconsume role/stage;
fifthtransport củarole đã đủ4 cũng bị từchối. Đây là`design`/realTCPsynthetic, **không9thCA runtime hoặc
map-awaregeneralroom-full/auth**. Clientcapacity9negative CP147 giữgiớihạn một actual+synthroster;
CP208–211 có8actualplayers, không khái quát mọi mode/max12/24record trongdump thành24humanplayer.

Preparehelper đầu gọi FixtureProfile với relativefixturepaths từrepoCWD nên FileNotFoundError,
trong khi launcher đãverifypass vì chdirbundle. GiữV1script/observed-error note; sửa **readerhelper** để
so raw bằngexplicitpaths vàTCPconstructor chdirbundle/restoreCWD, **trướcfreeze/native**, không sửa gói/code
đểpass hoặc suy đây là nativefailure. Genericgameplayverifier khôngtựchứngnhậnportable; deliveryreport
độc lập nối argv/module/sourcehash/config/pins với nativecallbacks. Raw/seal/record cũ giữ nguyên.

Giới hạn: fixed8Free4v4/operatoridentity/oneknownmap/Wine/privatebridge, chưa actual1–7/Manner8/
interactivechange-team/generalroomauth/native9/full8items-terrain-kim-cứu/accountresume/Internet/K7.
Không thay gate từ packaging/demo; chỉ PLAN giao nhánh teamvariants kế. Audit kếCP212.

<a id="eight-team-variants"></a>
### Tám actualclient: Free 1–7 và Manner 4–4 qua hai kết quả/reset — CP213

Bổ sung delivery11/11: [source Python byte-identical/kiểm](../evidence/eight-player/team-code-delivery.json) được giao riêng; không đổi kết quả/copy/hash CP213 và không chứng minh Rust variants.

[Raw/recipe/hash](../evidence/eight-player/README.md#cp213), [câu hỏi/phép thử](../RESEARCH.md#gap-lobby-identity-followup),
[thứ tự duy nhất](../PLAN.md#current-order). `kr_runtime/confirmed` trong profile/copy dưới; team/rank/fuse policy `design`.

| Câu hỏi / nguồn tái sử dụng | Delta / kết quả KR mới |
|---|---|
| CP147 capacity8/Free4v4-1v7; CP162 actualselector81→86/87/180; CP163 Mannerbalancedguard8negative; CP2102118life/rank/reset | Không giải mã lại codec/175/rows/map/teamguard. Fixed operatorlayout4v4/1v7 theo instance, allowManner cần measured echo policy; không port VN89 hoặc mutate module-global roster |
| Cùng8actualFree1–7 | Copy4b/32TCP/8prefix-display-IP-netns/directbridge/ExVillage01/Free0. Vòng1 AactualSpace/selftrapdeath→B–H thắng; vòng2 B–HactualSpace/trapdeath→A thắng. Full8life-before175/7peer170/rank1751082-inner1072/1803/room/7Ready2–3/Start3.4344gameplayjoins/125modejoins/128copiedgetters/358.900s/2126raw/892pins/cleanup0 |
| Cùng8actualManner4–4 | Copy4b/current8/fixedalternatingteams/selector1; hai opposite4v4 outcomes/reset/Start3.4357gameplayjoins/124modejoins/128copiedgetters/369.936s/2102raw/929pins/cleanup0. Không suy Free→Manner chỉ từ caption hoặc serviceconfig |
| Modepersistence qua cả8view | Mỗi view có3native123F9C0setter: initial DB2A50(create86) hoặc8179BA(join87), haiE64A7B(return180), đúng0/1. Actualwirehash/typedfield/callchronology và3actualStartreturns0 độc lập. R1counts/rule80budget hết lúcjoin2..5; không gọi earlybalance_reject là final8rejection. R2filterfull8 trước budget, nativeFree8/2teams/selector0 đã thấy |
| Servicecontrols | TeamProfile instance riêng; rank/outcome chỉtwofixedfamilies, Mannerunbalanced104 reject trước roundID/key/allocation.713Free1v7/775Manner4v4/180Manner1v7 actualTCP controls (application C2S tổng hợp),366unit.13bundle+5preset tests; packaged8battle giữfixed4v4/khôngoverrideManner. Không coi synthetic104 lànativeclick |
| Startupcontrols/negative | Original4b FreeR1D và Manner1v7R1E/R2F đều preBattleSIGABRT96E300 family, raw giữpartial. R2Factual96A480inputsize0/cap15, parser96A4EB→throw96E2D0 expectedvalue; khớp cacheCP073201, không locale/teamfailure. Nguồn emptyHTTP/upstream còn unknown; guardempty-onlycandidate8d7194 có control riêng, không tự áp kết quả lên4b |

Protectedlife đọc native runtime;128getter/run dùng copied52A350/Unicorn tách riêng. OS Space/gameevents/currentkey,
drainedbombs/freshsurvivor21/settle8/rank/reset kiểm cùng-run; không ghép hai run thành một trận. Tutorial/ảnh rank chỉ
bổ sung, không screenshot-onlyproof. Legacy controller early_completion label nói4v4 ở cả Free1v7; raw giữ nguyên,
verifier mới đối chiếu đúng per-slotteam/death/rankvector, không dùng label làm oracle.

**Giới hạn:** operator fixedroster, chưa native interactiveC2Steamchange/generalteamfamilies/native9/fullroomauth;
full8items/terrain/kim/cứu/ability/raylength chưa tíchhợp. Fuse6/report-derivedlife/rank/zeroeconomy/settle8 là emulator
design, không servergốc/worldauthority. Manner1–7 nativeguard đã có CP163(mộtactual/tám tổnghợp); các8actualcopy4b
negative chưa tớiStart nên vẫn partial. Alternatecopy8d có8nativecount8/selector1/balancebranch/return7/no104,
1543checks/140.441s/755raw/508pins/cleanup0; emptyguardkhônghit nên không causalstartupfix hoặc runtimeemptybranchcoverage.
Code source/copy4bpositive giữ riêng, không giảm tiêu chí hoặc biến alternatecopy thành unmodifiedclientproof.
K6c WAN giả lập/K7 đã nghiệm thu CP212; Internet thật chưa kiểm, không dependency các gate đã đạt.

<a id="rust-eight-teams"></a>
### Rust fixed1v7 và native Manner lệch đội — CP251, M4 còn mở

[Metadata/hash/giới hạn](../evidence/rust-service/eight-teams.json), [recipe/raw](../evidence/rust-service/README.md#cp251),
[thứ tự](../PLAN.md#current-order). Tái dùng CP213 Free1v7/Manner/selector và CP250 Rust8baseline; Python source
CP213 đã giao nguyên bytes qua PR322/`83abe4b`, không nhập WIP. Không RE lại codec/rows/rank/guard.

`eight_team_layout="4v4"|"1v7"` chỉ hợp lệ khi opt-in `eight_battle`/8actor. Không khai báo giữ alternating4v4.
Room giữ vector đội bất biến khi có member; slot/profile106/110, measured121 teambyte mỗi row26 và outcome/rank
cùng dùng vector này. Rank1v7 khớp hai bodyhash native-accepted CP213 (wire regression, không tự là nativeRustoracle).
Chỉ hỗ trợ fixed operatorlayout, không C2S đổi đội. Manner1v7 synthetic104 được no-op reject trướcbegin/round/key/
pickups/queue publication, transport vẫn sống và Back86 dùng được; `start_rejection` ghi lý do. Free0 cho đội lệch;
report-derivedlife/drain/unique7writes/freshwitness/settle8/fuse6/zeroeconomy và cursorfences giữ nguyên.

**Native confirmed hẹp:** cùng8KR00967/Wine10.0-20260425/**alternatecopy8d7194…**, 32TCP/64native106/errors0,
8auth-own291/render/ownslot và64actual106teamfields1v7; actualUI Manner1/7Ready/Start→native count8/cap8/2teams/
selector1/balancebranch/return7; hold3s không actual104/99/Battlewire/clocks. 90raw/127,113194352s/cleanup0,
1.513TCP/266exactreplies/fulltuple-seq-CRC-SHA. Không gọi negative này là copy4b hoặc bằng chứng sửa startup.
Copy8d là guardempty-only đã đo trước đây; run này không đo guardbranchcoverage và không có causalfixclaim.
CP250 copy4b Manner4v4 hai oppositeoutcomes/reset/Start3 được tái sử dụng, không chạy lại.

**Free native còn thiếu:** hai run mới dừng trướcRoom, copy4bD post17 (66raw/52,509s), alternate8dG post17
(67raw/60,841s); giữ runerror/sourcefrozen/cleanup0. A–C hoặc A–F nativeauth/profile/render dương tính trước lỗi,
không native Freegameplay/selector0persist. Chưa rõ nguyên nhân cùng family CP250H/G; không suy Rustteamguard sai
hoặc copy8d sửa SDK/JSON. Sau hai lượt, không retry startup khi chưa có observable mới; PLAN chuyển native9gap độc lập.

71workspace tests/35eightTCP/8prebind/32normalfourTCP đạt; malformedlayout/wrongcount/flag combos từ chối trước bind,
1v7 rank từ CP213 raw và all-seven-deaths/currentwitness/roster-drift được kiểm riêng. HarnessR1 UnboundLocalError
trướcRoom giữ93raw/PIDgone; correctedr2 chỉ sửa testcontainer mutation. GuardcheckerR1 sai expectedreason giữstderr,
r2 kiểm đúng `invalid diagnostic actor binding`, không đổi server. UDPcapturer nay nhậnIP2..9 và giữ workerexception,
harness failclosed trong loop/sau stop nhưng cleanup/export vẫn chạy; injectedfaultcontrol đạt. Native negative không
BattleUDP nên **chưa có14Aedges/full56wire/nativeAPI-keyapplyproof mới**; không hồi quy CP250partialcapture.
TMPFS/exporthash không sửa Btrfs durability. Không native9fullroom/M4closure/M5ownership/fullgame/production;
K6c WAN giả lập giữ đã đạt scoped, **Internet thật chưa kiểm**.


<a id="current-team-mode"></a>
### Free/Manner, selector và giữ qua return180 — CP162

**`kr_runtime/confirmed` trong copy Wine/nhánh Battle thường, policy echo là `emu_choice/design`.**
[Run/recipe/control/hash](../evidence/peer-endpoints/README.md#cp162). Reuse CP147 getter `123FA10`/manager+9C,
Start balance và14 emulation cases, CP016/076 actual81 schema, CP017 ACK86 `DB29A0`, CP028 join87
`817820` và CP022 return180 `E64950`. VN lobby/oc_a-room chỉ hướng dẫn workflow/ready/đội;
không chuyển opcode/enum/offset/layout VN. Source CA SHA a52545c1…, Wine e9bb4766…,
static660447c2…/modela08950c8…, base400000, RVA=VA−base; full hashes tại evidence.

Click thật Free/Manner và ảnh checkbox/tooltip đối chứng: **u32 thứ hai sau hai string trong81 là0/1**;
15 trường đuôi còn lại không đổi so với actual Manner baseline. Đây là một field đã nối trong phần
trước kia để opaque; field5 không được suy thành capacity. Random/giá trị khác chưa đo. R2 Manner+9C=1
trước create và tại entry ACK; fixed86 ghi0 qua setter `123F9C0`, actual write tại`123F9FE`, caller`DB2A50`.
Consumer cached86 đọc selector ở payload offset7; guest87 đọc **offset5** qua`8179B5`/return`8179BA`
(second u8 sau slot/session, không offset4). R3 sửa86/87 giữ1 lúc join nhưng **failed persistence**:
return1 ghi cả hai+9C về0, stack`E64A7B`. Cache CP022 nối đó với byte thứ hai180 (offset1) sau bool0.
Hai outcomes của r3 không được gọi hai vòng Manner.

Opt-in `room_team_mode: {"mode":"create81_to_86_87"}` dùng bounded81/title≤32/CP949/empty-password
policy như [title](room-membership.md#current-room-title); chỉ chấp nhận selector0/1. Preflight successful86/87 và1803byte
bool0/listflag0 trước listening. Echo81 vào86/87, commit selector sau ACK thành công; malformed/unsupported
request không ACK/không commit title hoặc membership. Return180 chỉ thay selector, giữ bool/listflag và
sequence theo recipient. Default không khai báo giữ nguyên. C/promoted/recreate/general membership không
được coi đã hỗ trợ; known A/B policy từ chối alias ngoài create/join. Row91 mode metadata khác title
vẫn chưa phục hồi, không sửa opaque row hoặc khái quát nhánh compressed/list180.

R4 production/direct module, không hold165/active UDP replay: actual Manner81→A86/B87/B Back/rejoin87,
hai native outcomes/reset/Ready, cả hai setter tại return180 đều1, không write về0 trước Ready2;
actual Start caller`DCB138` đọc1 ở hai epoch. 7 actual sent86/87/180 frames được native framer/getters/
CRCpartial kiểm21 cases và giữ mọi byte khác ngoài selector. 40 mode joins; 75 pickup/46 cycle/670 peer
4 windows/14 claims/16 loading/26 entity reset/23 strict life kiểm riêng. Getter-model không Winecall/worldoracle.
257 server tests (10 mode tests) đạt; preset chỉ bật sau r4 positive, final export exact operational profile;
chưa runtime chính file preset mới. R4 A Start area trắng một ảnh cuối, B Ready và tên hai view đúng;
không suy persistent UI lỗi hoặc Start3 thành công. Không third match/8 client/fullmatch/K6a/LAN/Internet/
Windows/auth/original authority; natural CP157 vẫn unknown. Runtime8 slot selector1 đã được bổ sung ở [CP163](#battle-eight-roster); không lặp capacity8 hoặc gọi hai client1–1 là chứng minh mọi đội cân bằng.


<a id="mode-preset-input-blocker"></a>
### Committed mode-preset: first clock và input blocker — CP164 (lịch sử, CP165 giải scope mới)

**`kr_runtime/confirmed` cho phần đã quan sát; planned2cycles/Start3 còn `unknown`/chưa đạt.**
[Run/hash/audit](../evidence/peer-endpoints/README.md#cp164). Tái dùng CP160 normalStart3/export/native,
CP162 Manner81/86/87/180 và CP163 balanced8; không body RE/productioncode/preset/47pins change.
Hai freshcopy/privatehosts runs thật dùng exported file đúng committedpreset/exactCP162r4profile,
module `server.probe_service`, không hold165/activeUDPprobe. Actual Manner81→A86/B87/B Back/rejoin87,
selector1 và firstnative121129/clocks cảhai đã quan sát; đây là native runtime từ presetfile, **chỉ partial**.

R1 `alignmiss:R1-A-contest-approach`: targetY199±30,20freshneutral samples luân phiên159/234 sau
OS45ms pulses, không tới Fire. R2 chỉ đổi pulse15ms, cùng tolerance/20attempt/3freshsame/neutral3s:
6stable samples159/234 rồi `alignnotstable`, không hai normalcycles/Ready2/Start3. Actual input pacing
không đủ từ thời lượng xdotool; chưa chứng minh nguyên nhân native timing/queue/keyrelease/transport.
Không gọi service lỗi, không nới vùng đích/thời hạn hoặc phát report/command giả để đạt kế hoạch.
R1/r2 source frozen/cleanup extinct/rawseal, service errors0; đo lại wrapper getter120/36cases là model
riêng, không gamefunction call. Actual81/86/87 mỗi run12nativeframer/string/getter/CRCpartial controls;
18partial joins/run, không ghi completedsmoke hoặc gate từ những kiểm này. Giữ CP162 r4 thành công
hai vòng là scope riêng, không dùng nó backfill hai partial.

**Audit AUD049:** README→PLAN→PROTOCOL/authority→RESEARCH→evidence và raw CP161–164 đã đối chiếu.
Sửa stale next-title trong RESEARCH, unknown selector1 input trong PLAN, currentauthority đang gộp
oldpresetStart3 với mode mới; PR232/233/234 đã merge, CP164 pendingPR. Giữ complete records/runhash và
failedR3/CP157, auditthrough164/next168; không tự nghiệm thu gate. K4 currentmode chuyển chủ/vào lại
còn chưa đo: codeoptin chỉ create/join và từ chối promoted aliasjoin_A; prior CP157/129profile không
chứng minh currentmode. Fullauth/authority/K6a/fullmatch/Windows/LAN/Internet/8realclients còn mở.
Phép đo tiếp chỉ ở PLAN/RESEARCH: nativeinput/keyup/actorstep trace tối thiểu rồi thử lại khi có giả thuyết.


<a id="input-batch-gating"></a>
### Native batch, observer control và committed mode-preset Start3 — CP165

**`kr_static/kr_runtime/confirmed` trong scope copy Wine/debugger/recipe; không vanilla timing.**
[Raw/control/hash](../evidence/peer-endpoints/README.md#cp165). Sourcefirst CP082/094 neutral, CP123
keyup drift, CP150 stop-on-nativepickup, CP156 tolerancecontrol, CP164raw2partials; VNinput dispatch/gate
chỉ cách đo, không port offset/IAT/opcode. NativeIDA4body `E7C400/E7CE60/E7D350/E7C750` và cached
`E7D2B0/E2D130` dùng privateDBcopy; bodyhash khớp fullcopycf72/static660447/runtimee9bb/modOa089,
base400000/RVA=VA−base. MCP thiếu plugin, existinglocalIDA batch fallback riêng; không config/sourceDBchange.

Runtime finiteDown45ms: actualWndProc keyup xảy ra **trước** movingbatch A, nhưng batch đó có15 lần
native loopE7C48B→E7C750/worldvirtual1C; actorY159→164→…→234,5px/bước. State23 giữ qua loop,
sang15 sau loop trước virtualE2D130; không gán causality cho virtual khi state đã đổi trướcentry.
Getter4524models từ rawwrapper là emulation riêng. B thấy234 không thay proof exactUDPapply của CP145/155.
Đối chứng chỉ measurementBPgating, samepulse45ms/profile/copy: chờ3consecutive batches≤3 bounded5s,
nativecounts chủyếu1 và actor159→174 ởhai view;112BPenabledstates đượcrestoređúng. Native24gettermodels
và17relations riêng. Heavytrace có thể tạo thêm timingperturbation: đo/control này không retrospective
chứng minh mọi CP164input hoặc bug nativegốc; đủ giảthuyết cho harnessintegration mới.

Runpreset r3 dùng **actualcommitted47pinexport/exactCP162r4profile/directmodule**, không service/game/preset
change, nohold165/activeUDPprobe. Chỉ trong12alignmentwindows, đóng measurementbreakpoints khôngcần
và bỏ duplicatedcollisionprintf, giữ readonlynativefeedback; restoreexactenabledstates cảhai view trước
contest/Bomb. Giữ45ms/20attempt/3freshneutral/tolerance30approach,12return và neutral3s/known6sreturn;
không đổi target/actual124/native165 hoặc gọi gamefunction/viếtactor/count/tick.162gatingchecks/capturedacks
vàall4native165callbacks/view khôi phục proof pickup. Event chronology tronggatedwindows không đầy đủ;
không gọi đó là wholeinstructiontrace hoặc mất event nghĩa là không thực thi.

Cùngrun397.659234656s/exit0/1366raw/107frozen:2normaloutcomes/reset/Ready rồi normalA104/third121129/
99barrier/nativeclocks/render+8bothviews; thirdactive/nobomb/noaccepted17,19, no thirdoutcome.
Mode1 qua86/87/rejoin/2×180 vàStart3; title16byte heap31/native81/91/87 vàảnhcảhai giữ.75pickup/46cycle/
651peer4windows/14claims/24loading/26entityreset/23strictlife/26preset/42mode/21modeframercases;
648actor/420peer/2758flag+2730array/16life/330nativepacketmodels táchruntime/model. Raw/cleanupseal
đạt; reuse257server/4exporttests bằng exactsourcehash vì productionkhôngđổi. CP164partials không backfill.

TạiCP165 currentmodehostpromotion/rejoin còn thiếu và join_A87 bịpolicy từchối; [CP166](room-membership.md#current-mode-host-rejoin) giải idlepath sauđó. CP107/129/157oldprofile vẫn giữscope riêng. K6a/fullmatch/Windows/LAN/Internet/8actualclients/generalreset/
fullauth/originalauthority/naturalCP157 mở. AUD049qua164/kế168; bước tiếp duy nhất PLAN.


<a id="rust-ninth-admission-wip"></a>
<a id="rust-ninth-admission"></a>
### Visitor thứ chín trên Rust — CP252, phạm vi một KR thật và tám member TCP tổng hợp

[Metadata/SHA](../evidence/rust-service/ninth-admission.json), [recipe/raw](../evidence/rust-service/README.md#cp252),
[thứ tự](../PLAN.md#current-order). Source-first CP227 đã có native87/result2, nên không RE lại parser lỗi hoặc port VN opcode.

**Policy design:** opt-in `ninth_admission_probe` nhận chín identity lobby chẩn đoán nhưng phòng vẫn tám slot.
Visitor9 đã có identity tại lobby; khi idle room đủ tám member, request82 đúng context nhận87/result2 trước `enter`:
không membership/slot/visit/round mới và không broadcast sang tám member. Generic2 là “không thể vào phòng” đã đo;
chọn lý do capacity là design của server này, không xác nhận mã fullroom gốc. Probe chỉ hỗ trợ negative full idle,
không vacancy/replacement/general admission. Roomlist vẫn fixed cached card, không proof cập nhật occupancy động.
Fixture producer tạo auth4/own291/header cho KRPeer9 bằng các primitive đã đo, không thêm player_I hoặc slot9 vào phòng;
output producer không phải native oracle. Cấu hình sai số actor/flag/name bị từ chối trước bind.

**Native confirmed:** một KR00967/Wine10.0-20260425/copy4b trong host ảo riêng; root namespace giữ tám socket member
TCP tổng hợp. Hai netns/root+veth/bridge và tám loopback source127.0.0.2..9, không route ngoài. Tám member đã setup/Ready
trước native launch. Visitor9 đọc auth137/own291128/errors0, render KRPeer9, phát C2S6ID9 và actual82. Server trả87
onebyte2; native816B2C/8177D1 đọc1/1/errors0. OrdinaryReturn đóng notice, hold3s ở lobby; không585/Room106/slot grant.
Hai ảnh notice/lobby đã xem, raw/native/TCP tách nguồn. 19raw/35,407594284s/cleanup0/20TCPconnections/1.384records/
249exactreplies/fulltuple-seq-CRC-SHA. Không suy chín client native đồng thời hoặc nghiệm thu gameplay mới.

6 ca realTCP synthetic kiểm hai lần87 trên cùng channel, không member removal sau visitorEOF, Ready giữ nguyên,
Start đầu vẫn key12346/eightACK/peer205 không ninth. Hai unit kiểm no membership/round mutation, queuefull/wrongcontext;
73workspace tests/4prebind đạt. TCPR1 loader đòi nhầm roomplayeralias9 trước bind, giữ94raw/PIDgone, sửa để chỉ đòi tám
roomplayerfixtures. Không thêm slot9 để né lỗi. Source/tool và lockfile/toolchain giữ provenance.

**Failure/control:** nativeR1 đã tới error2branch1/1/errors0 nhưng controller chờ handler_end khi notice chưa đóng;
tới120s syntheticidle thì fail,17raw/cleanup0 giữ. R2 chỉ thêm ordinaryReturn theo CP227, không đổi server. Verifier đầu
đòi mọi close reason đềuEOF; R2 có7Brokenpipe khi đóng cùng lúc tám synthetic trong cleanup. Phân tích sau run giữ nguyên
lỗi, chứng minh tất cả9channelFIN/RST trong PCAP sau completion; fullTCPtuple/replyjoin vẫn đạt. Không gọi đây là sửa
shutdown, không rerun native chỉ để sửa checker.

**Argv:** run9 có actualperrole32 Popenlist và independentCAproccommandline, không scalar-only. Control Free mới cũng có
32Popenlist cả8, independentCAargv A–G gồmD, Hchưa được observer ghi trướccleanup. D vẫn post17/preRoom,66raw/48,888s;
khôngFree2outcomes/selector0persist hoặc causalSDKfix. [AUD072](bootstrap-codec.md#argv-provenance-correction) giữ correction riêng.
M4 vẫn mở; M5backend đã có/nativeownershipbridge kế tiếp độc lập. TMPFS/exporthash không sửa Btrfsdurability;
K6c WAN giả lập giữ scope đã đạt. **Internet thật chưa kiểm**.

<!-- protocol-topic-content-end -->
