# Authority, phạm vi service và ranh giới Rust

[Mục lục](../PROTOCOL.md) · [Câu hỏi đang mở](../RESEARCH.md) · [Thứ tự việc](../PLAN.md#dashboard).

<a id="rust-port-boundary"></a>
### Ranh giới contract khi chuyển Rust — design CP213

[Thiết kế/ma trận readiness](../RUST_SERVICE_ARCHITECTURE.md), [thứ tự/acceptance duy nhất](../PLAN.md#rust-service-milestones),
[unknown/RE tối thiểu](../RESEARCH.md#rust-service-gaps), [provenance](../evidence/rust-service/README.md).
Đây là kế hoạch, **chưa có implementation/nativeRust**. Contract KR giữ ở các mục bên dưới, không sao chép
opcode/layout thành spec Rust riêng. Chọn policy đã nghiệm thu có revision, không đổi authority trong port.
Gate Python scoped K0–K7 không chứng minh mọi feature/catalog/fullgame hoặc accountauth/inventory/reward.
AccountDB/backendadmin/transaction là design công ty; adapter credential/session/projection cần bằng chứng KR.
Không nhập wireInputCommand/Snapshot/authoritativeengine của boom-rs hoặc opcodeVN. Python RE/reference
vẫn giữ; parityPython↔Rust/encoder tựsinh không oracle độc lập, nativeend-to-endRust là acceptance riêng.
Wine/LANgiảlập/K6cWANgiảlập giữ quyết địnhowner; Internet thật chưa kiểm, khôngdependencyngầm.

## 5. Authority và profile emulator

Record cho từng hành vi: ai tạo sự kiện, ai mô phỏng, ai chấp nhận/loại, ai broadcast, state nào peer
có thể sửa. VN đã cho thấy client tự di chuyển, nổ lan, vỡ gạch, trap/death và rescue; server emu cấp
ID bom/timer nổ/xác nhận pickup, peer mang movement. KR có bằng chứng client consumer nhưng chưa đủ
để chốt toàn authority. Gắn `unknown` phần chưa đọc, không dùng “server authoritative” làm mặc định wire.

Profile `kr-emu-dev` dự kiến tách khỏi `v1-dev.toml`: build/map/roster/packet-profile, timer/policy và
provenance từng giá trị. Drop Bomb/Fire/Roller do Rust V1 chọn, fuse2,5s của emulator VN, timeout15/10/5s
và host-ready của V1 đều là quyết định riêng; không tự chuyển thành KR native behavior. Server chỉ
enforce điều client có thể biểu đạt/nhận được, validation shadow giữ log discrepancy trước khi cưỡng chế.

Client gốc tự vẽ và phát âm thanh. Tái sử dụng ODT/assets giúp đối chiếu scene/actor/map, không yêu cầu
viết lại UI để hoàn thành nhánh tương thích. Những patch bootstrap/AC nếu cần chạy offline phải được
liệt kê và phân biệt với nguyên bản trong mọi demo.

<a id="authority"></a>
### Authority matrix hiện hành và gate review

Bảng này dẫn nguồn bằng chứng từng phần; chưa phải toàn contract server gốc. Mỗi hàng khi
triển khai phải bổ sung riêng **creator, simulator, validator, broadcaster, peer-editable state**, với
locator/trace và trạng thái từng ô; những vai trò không nêu dưới đây hiện `unknown` cho KR.

| Hành vi | Bằng chứng theo scope đang dùng | Giới hạn/kiểm còn thiếu theo scope sử dụng |
|---|---|---|
| Movement | [CP188 currentAB291](life-results.md#current-basic-cycles-291); [CP176 A↔C hai key](life-results.md#a-c-basic-cycles); [CP159/160 production](#exported-preset-runtime):4nativepeerwindows/actualapply/target/currentprogress, nohold165/activeUDPprobe; [CP070](udp-peer.md#current-udp-causality) giữcontrol riêng | [Bốn-view CP200](multiplayer-team.md#four-session-native) đã có currentprogress12chiều; fullschema/auth/reliability và NAT discovery còn unknown; [NAT tĩnh/UDP delay-loss](networks.md#udp-netem-cycles) có scope riêng, không fullgrammar trước mọi demo hẹp |
| Bomb place/ID | [CP188 currentB/A/A](life-results.md#current-basic-cycles-291); [CP176 C/A owners](life-results.md#a-c-basic-cycles); [CP160](#exported-preset-runtime) actual118/nativeID1/A/power2,ID2/B/power1/full159158/195flagreset; [CP156](networks.md#direct-host-eof-acceptance) pendinghostEOF/fence scope riêng | Boundedrole/key/serial/quota/geometry làpolicy; actorlocation/fullauth/allpool/races unknown |
| Explode/chain | CP033 một owner; CP064 [power1/cách hai ô](bombs.md#mixed-owner-chain), CP071 [power2 ngoài tầm/feedback chưa118](bombs.md#mixed-owner-power) giữ partial lịch sử; CP081 [power2/distance2 mixed-owner](bombs.md#mixed-input), CP084 [ba bom fork hai vòng](bombs.md#three-bomb-chain); CP094 [optional pinned terrain](terrain.md#terrain-policy), CP095 [actualbox/laterfreshpair](terrain.md#opposite-box-control) có native callback/195flag proof | [CP096adjacent1cell](terrain.md#adjacent-owner-control) và [CP122 chain sâu](bombs.md#recursive-chain) đã đo trong ca hẹp; [CP133 shared cellstate/query](terrain.md#shared-cell-state) và [CP134 hai shared cycles](terrain.md#shared-cascade-cycles) đã giải ca cố định; FIFO/firstbombstop/immutable tổng quát/race và authority server gốc chưa đủ |
| Block destroy/item reveal | [CP176 A/C reveal/reset](life-results.md#a-c-basic-cycles); [CP172](items-stats.md#hidden-cell-admission-current) optional basic export/terrain-cell142 guard; hai native cycles/reveal27/28/3effects/reset/Start3 | Chỉ bounded terrain policy; không recovered142key/auth/distance/originaldrop/generalmap. Không terrain giữ weak anycurrent158 scope cũ |
| Item placement/pickup | [CP159/160](#production-baseline) productionimmediate165/1or2actualclaims/oneeffect/nativewinnerpower/reset; [CP155](networks.md#direct-oldudp-contest-eof) hold2sactualcontenders là control riêng | Full124distance/auth/generation/inventory/max/speedTTL/activevariants chưa đủ; delay2s không còn trongproductionbaseline |
| Trap/death | [CP188 currentAB](life-results.md#current-basic-cycles-291); [CP176 A/C56→1/other0](life-results.md#a-c-basic-cycles); [CP159/160](#exported-preset-runtime) strict56→1/other0beforerank2views/2epochs; [CP157natural](networks.md#current-host-ui-exit)/[CP158controlledoverwrite](life-results.md#current-trap-udp-counter) giữ causal limits | Snapshot khôngglobalsametick; samekeycounterheuristic cho controlledremoteoverwrite; naturalCP157cause unknown, nofullvalidator |
| Result/reset | [CP188 current2outcomes/Start3](life-results.md#current-basic-cycles-291); [CP176 A/C2normal/Ready2](life-results.md#a-c-basic-cycles); [CP171](load-phase.md#basic-preset-start3) optional basic export hai outcomes/reset/Ready rồi normal BStart3/native12199/clock/render; CP168 floor riêng | Không trận ba outcome; allinventory/timer/seq/peer/resetvariants/originalarbitration và gate còn mở |
| Room lifecycle | [CP175 Cslot1](networks.md#a-survivor-admission)/[CP176 ACreset](life-results.md#a-c-basic-cycles); [CP161 title](room-membership.md#current-room-title), [CP162 mode/return](multiplayer-team.md#current-team-mode) có2normalcycles/Ready; [CP165 currentpreset](multiplayer-team.md#input-batch-gating) guestBack/rejoin/2outcomes/Ready/Start3 đã đo; CP164partial giữ lịch sử. CP155–157 EOF/UI controls giữ profile riêng | [CP168](room-membership.md#current-mode-promoted-start3) currentmodeBhost2cycles/Ready/BStart3 đãđo; R1/R2partial vàoldwhiteframe giữriêng. Accountresume/halfclose/loss/generalreplace chưa đủ; không lấy oldprofileexit làm currentmode proof |
| Peer lifecycle | [CP159/160](#exported-preset-runtime)actual99/153/currentdirectIPs/ports; [CP155oldUDP](networks.md#direct-oldudp-contest-eof)/[CP158samekeycounter](life-results.md#current-trap-udp-counter) controls riêng | Key/counterbyte khôngauth/universalfreshness; [LANgiảlập](networks.md#emulated-lan-acceptance)/[NATtĩnh](networks.md#nat-two-cycles) đã đo; low8wrap/reliability/fullgrammar/relay/NATdiscovery/Internet unknown |

Trong **optional profile hiện hành**, bảng dưới ghi vai trò emulator, không gán cho server gốc.
Giới hạn và phép đo nằm ở link contract; không dùng mỗi vai trò này làm gate acceptance.

| Cơ chế/profile | Creator | Simulator | Validator của service | Broadcaster | Peer-editable state / giới hạn |
|---|---|---|---|---|---|
| [Hidden basic582/142/124165](items-stats.md#hidden-cell-admission-current) | Emulator catalog/list2 gắn existingFixedBox; native158/reveal rồi142/124 | NativeBox/item165/effect và bounded terrain model | 124key/serial/consumed;142slot/cell/aux và với terrain: currentgeneration/clearedcell/committedblast/fullroster before report; no-terrain giữ anycurrent158 | Service165 tới channel của roster theo claim | Identical old142 sau legitimate currentclear vẫn có thể nhận; fullauth/distance/freshness/originalserverauthority unknown. Reset và duplicate idempotence có scope CP172 |
| [Place159](terrain.md#terrain-policy) | Native Space tạo118 | Native159 tạo/bind entity ở cả view | Role/key/serial/quota + knownvariant/power/floor/nonoverlap theo profile | Service cấp ID/phát159 | Client gửi cell/power; chưa xác thực actorlocation/distance/fullauth, không tự coi field là tin cậy |
| [Explode158/terrain](terrain.md#terrain-policy) | Timer6s của profile là emu_choice | Native158/process/chain/mapflags; service dùng bounded terrain model | Model/profile pin/kinds/power≤2/currentgeneration; semantic modelchoices có giới hạn | Service phát158 rồi commit world/release | Peer movement có thể ảnh hưởng contact/report; report142 không làm worldoracle. Original server validator/order vẫnunknown |
| [Trap/death/result](life-results.md#result-freshness-guard) | Native local simulation tạo127/subtype | Native170/175/180 nhận effect/UI | Guard key/slot/reportstate/selectedsame-oldbody; optional result freshness/drain/21 sau17 và opposite-peer170 theo [CP102](life-results.md#drained-result-local-clear)/[CP103](udp-peer.md#peer21-ending-order); firstseen/forgedforward/location vẫn thiếu | Service170/result/return theo policy | Report/body do client gửi; acceptedreport không là authoritative world proof. [Limits](udp-peer.md#tcp-guard-cycles) và RESEARCH giữ mở |
| [Peer endpoint/directUDP](udp-peer.md#virtual-peer) | Native99 báoIP0/UDPport; configđo chọnIP153 | NativeWinsock/lookup/handler/actor; actualCA+wineserverNS kiểmđộc lập | Currenttuple/key/slot trongcađãđo; no fullIPauth/NAT law | Stub153traobảng; nativeUDPđi direct | IP0khớpNIC/source trongvirtualcase; IP1opaque; [CP141 cấu hình tường minh](networks.md#service-address-config) thay measurement shim; operator policy, không là NAT discovery/auth; [mapping tĩnh](networks.md#nat-two-cycles) đã kiểm riêng. Hai basic cycles không là movement gate |
| [Four2v2 CP202–204](multiplayer-team.md#four-transport-bounds) | NativeSpace/127, operatorprofile/ID/team, timer/rank theo design | Native159158/170/175180; khônginferiorfunctioncall | Currentrole/key/slot/serial/capacity/reporthistory/deadguard và transportbudgets làdesign; no originalauth/distance | Bom/rank/180 tới4; report170 tới3peer, partialfailure fence/khôngsemanticcommit | Cell/status/power/report do client gửi; không authenticatedworld. Native4twooutcomes/reset/Start3 không suy4item/terrain/kim/cứu hoặc3survivorEOFcleanup (bổ sung riêng[CP205](multiplayer-team.md#four-member-abort-room)) |
| [Channel endpoint](networks.md#service-channel-endpoint) | NativeC2S5 yêu cầu redirect | KR1101EA0 nhận7 rồi nativeconnect | Service kiểm operatorIPv4/port và variant7 hẹp; không quyết định account/channel authority gốc | Service giữ raw mặc định hoặc thay typedIP/port | IP/port do operator đặt; không tin init99/identity client; rootTCPguardproxy còn riêng trong CP148 |

Gate K5/K6 cần kiểm matrix và đối chứng của hành vi thực sự dùng; từ28/10 Codex tự kiểm theo miễn review của chủ dự án. Có thể giữ unknown về
server gốc thất lạc nếu policy emulator được chọn `design`, client chấp nhận và ca consistency đạt;
không thể để unknown về đường đồng bộ cần thiết rồi tuyên bố multiplayer đã nghiệm thu.

<a id="probe-service"></a>
## Service fixture và phạm vi tích hợp — CP048

Nguồn/recipe/actual trace/native checks ở [evidence service](../evidence/probe-service/README.md).
`server/probe_service.py` là tiến trình TCP riêng; `probe_codec.py` chỉ hỗ trợ profile mode2/selector3
đã đo, header37 byte và typed reader. Header37 tính cả byte prefix; mô tả header36 lịch sử WCP
không được dùng để bỏ byte prefix. `probe_room.py` serialize owner, gọi UdpRoundKeys một lần/Start,
RoundFence cho159/158 và ItemClaims cho124→165; cùng khóa153 cho hai peer. Đây là implementation
`emu_choice/design`, không phục hồi thêm field wire hoặc luật server gốc.

Auth/roster/map/rank dùng fixture có SHA; snapshot CP048 dùng identity A/B theo thứ tự bốn connection mỗi vai.
CP051 mở đường [A/B→B/C hẹp](room-membership.md#service-replacement), không đổi thành auth/membership tổng quát.
Bootstrap/hello/auth/redirect có state riêng theo connection nhưng chưa schema tổng quát/cache/
reconnect đầy đủ. Kết quả/fuse/drop và forfeit vẫn là policy. Harness chỉ còn Wine/input/measurement/
UDP proxy; không sở hữu TCP state hay timer gameplay. Chi tiết run và prooflevel giữ ở evidence.

Các ca tích hợp CP048 chỉ áp dụng copy chẩn đoán Wine và stable IDs1/2/slots0/1. Không tự chứng minh
Windows nguyên bản, LAN/Internet, full124validator, toàn reset/grammar/auth hoặc K5/K6a.
Contract cơ chế vẫn ở các anchor tương ứng, không đổi chúng thành claim full match từ log service.


**Quan sát tích hợp CP048 (`kr_runtime/confirmed`, giới hạn theo run):** `integrated-r2` có hai
actual124/kind2, hai bom/17/19/170, hai175/180 mỗi client trên cùng8connection; exact-old UDP vòng1
đến E66680 ở vòng2, byte59≠56, secure actor giữ nguyên; normalpacket key56 tới apply.
`integrated-r3` kiểm thêm165full12 và Roller5→6/max7 cả hai view, clock vòng2 về5; run này chỉ một
124/118/result/return, input vòng2 không tạo request tương ứng trong cửa sổ đo. Không ghép thành
một run có hai165 paired-secure hoặc suy nguyên nhân thiếu request. Recipe/hash/native/raw ở evidence.

Trong actual124 của r2, `k`=12345 ở vòng1,12346 ở vòng2 tương ứng u32 thứ hai153 đã cấp;
đây là quan sát trên hai vòng/keys đó, không chứng minh toàn semantics hoặc validator server gốc.
Tại snapshot CP048, service parse `t/k` nhưng chưa kiểm chúng; CP049 bổ sung [guard k](items-stats.md#pickup-round-key). Replay124 trùng catalog vẫn là khoảng
trống có thể đo; không tự nói allocator UDP đã giải C2S stale124. Kiểm codec mới so **typed prefix của
97 actual/native-checked frame**, ngoài354roundtrip cũ; native framer/blob/CRC có proof riêng.


**Khoảng trống service được thực thi sau refactor:** body actual124/native-checked của r2 vòng1
được đưa vào `receive_channel` sau begin/finish/begin (catalog17 giữ, key12346). ItemClaims chấp
nhận body cũ `k12345` và phát positive165; current124/k12346 kế tiếp bị từ chối do claim đã tiêu.
Control chỉ current124 được chấp nhận. Đây là `design/confirmed` về code service, **không** runtime
client KR hoặc validator server gốc; [recipe/hash](../evidence/probe-service/README.md) giữ phân biệt.
Tại CP048 chưa thêm guard; CP049 bổ sung bên dưới, không gọi toàn stale124 đã giải. Phép đo tiếp ở [RESEARCH](../RESEARCH.md#gap-floor-pickup).


<a id="rust-fixed-room"></a>
### Rust fixedroom → host Back/rejoin/Ready → loading barrier/spawn — CP234

[Code/recipe](../../rust/README.md), [raw/hash](../evidence/rust-service/README.md#cp234),
[metadata](../evidence/rust-service/fixed-room.json). KRE-074/#285, tiếp [CP233](lobby-profile.md#rust-two-lobby); **M2 chưa đóng**.
Reuse CP016–020/028/034/035/161 room/cache/map/start/Ready, CP229/230visit/cleanup và currentPython
`room_title`/`room_team_mode`/`four_player_profile`/`probe_service`. Không hàm native/SDK/clientpatch mới.

**`design` implementation:** opt-in `fixed_room` trong twoactorprofile; `room::Room` serializes owner/slot/host/
Ready/setup/visit/round, `room_io` adapter pin fixtures, `room_wire` sửa đúng measuredID/slot/state/selector/title/
endpointfields. ID1slot0/ID2slot1, một phòng/session1/room1; title echo bounded1..32/control-free opaqueCP949carrier,
không Unicode normalization hoặc fullCP949validation; selector0/1 theoactual81/86/87. Listcardoccupancy/capacityfield
vẫnfixture, không chứng minhquota8/dynamiccount/generalroom/password. Các header/bodyunknown giữopaque.

`Room` clone/validate trước commit. Reserve queueentry cho mọi recipient trước publishstate/batches; queue8/recipient,
pendingtimer16/socket, replyseq riêng43+3n. Timer/batch chứa exactmembership(connection,visit), optionalround vàpeer
subject; kiểm lại trước gửi, project106/110 theo host/Ready hiện hành để scheduledstatecũ không hồi sinh peer/host.
KhôngatomicTCPbroadcast: partialwrite/sendfailure vẫn phải cleanup recipient; queuereservationtest khôngthaynative.
Own585→106,110 sau2s, peer106 sau3s (rosterpairde-dup), table238/map117 sau4s. Reuse fixture timing, không originaltiming.
Native96/body6,266/body12 và Armed190/empty chỉ observed/no reply;127/body1..4096 sauArmed chỉ observe, khôngack/
forward/grant/gameplayeffect. Không imlặngcoiBattle đãport. SourceIPoperator còn là diagnosticbinding, khôngaccountauth.

HostBack86empty→remove107(slot,ID,0)/survivor110 giữteam/chief và1s241/91 refresh; samechannelrejoin82 choA slot0guest
khiBslot1host. Start104 chỉ currenthost/two members/setup/guestReady, rosterroundsnapshot;99/body23/currentmembership/
UDPport!=0 đủcảhai mới197,1s196,2s153 có IPoperator và actual99ports/currentroundkey. **Visit/round chỉ fence internalwork**:
raw585zero và99 chưa có measuredvisit/roundtoken, khôngclaim nhậnra mọi ACK cũ trênwire. Roundabort/timeout/recovery
chưa đầyđủ; không phát cancel/result giả. Channel/logincandidateports khai rõ: R1actualCRCvalidlogin1/body42 dùng3848,
profile `login_candidate_ports` bổ sung theo measuredcandidate endpoints; no acceptorder/authenticationassumption.

**`kr_runtime/confirmed` R5:** hai actuale9copy/Wine10/2CA/2wineserver/3netns/vethbridge/noexternalroute/files-onlyNSS.
Auth137/291 đúng tên; ordinarycreate81(titleCP949“초보자 환영!”,selector1)→86, B91/82→87, native106204/204/errors0
trêncảhai với ID1slot0/ID2slot1, ownslotsetter0/1;1106/6/errors0. HostAordinaryBack→Bnative107ID1→0/chiefgetter1,
UIA vềsảnh/BStartbadge; A82rejoin→ownslot0guest/peerroster, A95Ready110, B104Start→121129/129/errors0.
NativeB99producer được observer giữ **3.363547654s**, A99 đã tới; controller/PCAP không197 hoặcclock khi thiếuB.
Releasefiletheodeclaredrecipe→actualB99 rồi197cảhai/196/153, nativeclock/spawnslot0/1, UIhaiActor/đúngownarrow/ExVillage01,
giữ8.278646918s. PCAP334record/allacceptedtuples/56exactS2Creplies/allacceptedC2Sseq-op-len, capturepartial/overlap
checks không báo thiếu. RootbinarySHA`554fa5b318ab58bfa93bd27933674d8ab73cd739fe586cb89d6515105766a04b` đúngfinalbuild.
Chưa đo movement/bom/terrain/life/result/reset trênRust; thấyspawn không là BattleM3.

**Failures/corrections giữ nguyên:** R1strictlogin3838 bỏ sótactualA3848→23rawFAIL; R2ASCIIonlytitle + OSinputkhôngđổi
customdialog→27rawFAIL; R3create/106đãđọc nhưng chưa port96/266→25rawFAIL; R4positivephòng/clock nhưng190empty làm
channelđóng, controllerthiếucheckchannelvàBheldmarkerJSONspaces→fallback20s release→37rawFAILhealthyhold. R5fixknown
ancillary/parserhealthcheck→40rawpositive scope trên. Tổng5run152raw frozen/seal/cleanup0/exactinputversionsretained.
StrictallclosureEOF verifier củaR5 fail giữ: sauearlycompletion8.279hold, ownedWinecleanupAEOF→roundIdle→Bfinal12714
bịreject; clientFIN tới sauearly0.216/0.273s, khôngđóng trướchold. Scopedverifier chứngminh đườngtrướcownedcleanup;
**không healthyactive-peerabort hoặc gracefulpostholdteardownclaim**. Không sửa raw/controllerreturn0 thànhsemanticproof.

14Rusttests(5wire+3Flow+2binding+3room+1queue),8syntheticroomTCPcase và16binding/14M1regression riêngnativeproof.
Một lầnroomTCPcleanup timeout (SIGTERM5s/killwait2s) giữraw/log, chưa rõrootcause; ba repeat cùngbinaryđạt, khônggọiđãfix.
[PLAN](../PLAN.md#current-order) cònchốtversion/negative+activepeerEOF/late-reportcleanup trướcfullM2, audit kếCP236.
Pythonserving/RE/reference/8WIP byteexact, CP233PR304mergeb10952f; K0–K7 cũkhôngđổi. **Internet thật chưa kiểm**.

<a id="rust-m2-acceptance"></a>
### Rust M2: nghiệm thu đường hai identity/phòng/loading và survivor sau EOF — CP235

[Checklist](../evidence/rust-service/m2-acceptance.json), [raw/hash](../evidence/rust-service/README.md#cp235),
[code](../../rust/README.md). M2 được nghiệm thu phạm vi diagnostic KR00967/Wine/một phòng/hai actor;
KRE-074/#285 vẫn mở cho dịch vụ Rust, K0–K7 cũ giữ nguyên phạm vi. [CP234](#rust-fixed-room) giữ các failure
và kết luận lịch sử; thay đổi mới giải khoảng trống late-report survivor, không sửa record/run cũ.

**`design`:** khi member rời ở Loading/Armed, model lưu roster chính xác `(identity,connection,visit)` của vòng
vừa hủy, chuyển Idle và clear Ready. Chỉ survivor còn current và thuộc roster đó được observe/drop thêm tối đa64
request99/127/190 có shape đã khai. Không gửi ACK/effect/forward/GO/result; member mới không kế thừa allowance,
Start mới xóa record aborted. Log ghi `dropped_after_abort`; quá budget hoặc sai membership vẫn bị từ chối.
Đây là policy chống đóng nhầm survivor do report đang tới, không là original authority hoặc replay protection
cho raw99/585 không có measured round/visit token. [CP234 queue/subject fences](#rust-fixed-room) giữ nguyên.

Trước bind, `load_config` chỉ nhận bootstrap15/WCP317 có SHA của profile KR00967 đã chạy native và kiểm toàn bộ
fixture pin/schema/opcode/identity. Sáu ca config sai, kể cả profile khác tự cập nhật hash, phải fail trước listen.
**Guard này là design/operator config**, không khẳng định đã gửi native wrong-version error hoặc đo UI của client
build khác. Các negative CRC/shape/duplicate/socket/stage/identity dùng kernelTCP/model; positive compatibility
vẫn dựa trên KR thật, không lấy encoder hoặc Python↔Rust agreement làm oracle.

**`kr_runtime/confirmed` run EOF:** cùng e9copy/Wine10/virtualLAN/2CA/2wineserver/3netns/no default route;
đường CP234 phòng/Back/rejoin/Ready/121129/barrier giữ đúng. B99 hold3.360600112s, chỉ đủ hai actual99 mới có
197/196/153 và clock/spawn. Recipe dừng riêng Wine prefix A sau clock; B giữ cùng PID/startticks/channel,
107 đọc đủ/native ID1→0; chín native127 sau EOF khớp chín actual report được drop, không phát GO hay gameplay
reply mới. B UI vẫn KRGuest, không error modal, timer1:56→1:48 và giữ8.230957675s sau hành động EOF.
PCAP345record/56exactS2Creplies/alltuplestreams/C2Sseq-op-len khớp; B FIN sau completion, A FIN sau declaredkill.
44raw frozen/seal/cleanup0/exactinputretention. Binary đã chạy/finalbuild cùng SHA
`ab06ae3f8d00e7baa3146ff503f3d11086df5e31819208025dcf53ca1c18992e`.

**Giới hạn quan trọng:** roster/sidebar xóa A nhưng sprite đỏ của A còn trên map. Chỉ nghiệm thu healthy B
session/roster/clock/report cleanup, chưa fullscene actor deletion/forfeit/result/return hoặc game disconnect.
A bị dừng chủ động nên không claim A healthy. IP/name/ID vẫn diagnostic, chưa accountauth/DB/generalroom/capacity8.
Một TCPshutdown timeout ở CP234 không có đủ PID evidence để xác định rootcause; ba repeat cùng binary cũ,
12race trên baseline CP234 và12race trên bản mới đạt. Giữ anomaly và instrumentation PID/thread/wchan ở M7;
không coi không tái hiện là đã sửa hoặc đạt productionSLO.

**Đối chiếu M2 thủ công:** checklist10criteria dẫn từng bằng chứng và loại kiểm: nativebootstrap/name/slots/
room/Back/chief/rejoin/Ready/loading/spawn; kernelTCP negatives; prelisten profileguard; queue/modelfences;
nativeBsurvivor; rollback/Python/WIP/freeze và operationalbounds trong scope. 15Rusttests,6profileguards,
12shutdownraces,8roomTCP,16binding,14M1regression đạt. Các phần chưa đo ở trên được nêu trong acceptance,
không đổi tiêu chí thành fullgame/fullservice hoặc đóng epic. [PLAN](../PLAN.md#current-order) chuyển M3 theo
contract đã có, bước đầu native UDP/movement hai chiều; RE chỉ discrepancy chặn port. Audit kếCP236.
**Internet thật chưa kiểm**; K6c WAN giả lập giữ đúng quyết định chủ dự án.

<a id="current-basic-contest-reset"></a>
### Basic hiện hành: tranh Roller, baseline từng vai và hai kết quả/reset — CP189

**kr_runtime/confirmed trên copy Wine4b781f…/privatevirtualhosts; authority policy vẫn design.**
[Recipe/hash/partials](../evidence/peer-endpoints/README.md#cp189), [K6a acceptance](../PLAN.md#k6a-acceptance).
Tái dùng CP155/159160 nativeclaim/oneeffect/immediate165, CP188 basic/profile/peer/life/reset và CP175
navigation; không thân hàm native mới, không port VN opcode/layout hoặc RE lại ItemAttr/wiki.

Focused `k6-current-basic-contest-r1` là production service, không hold165/replay. Hai OSkeys tiến vào
Roller17/kind2/row4col3; actual124 từ cả B và A/cùng key12345, B thắng, A bị consumedreject; một165
full12/errors0 mỗi view. Native secure getter xác nhận **B speed4→5, A giữ5**, capacity/fire không đổi.
Analyzer ban đầu giả cảhai baseline5 đã FAIL; source/log giữ nguyên, đối chiếu snapshot CP188 cũng
có A5/B4 rồi sửa đúng baseline từng vai, không sửa raw/game/profile để pass. Đây là stat của fixture
hiện hành, không suy speed gốc mọi char hoặc law/wiki khác. Focusedrun không kết quả/reset/EOF.

`k6-current-contest-reset-r1` giữ partial: hai claim/Awins/Bconsumed nhưng20 nhịp Breturn chỉ đạt
375 khi đích399/tolerance12, chưa bom/kết quả. Nativefeedback vẫn dịch chuyển, không suy stall/keybug.
R2 thêm normalBDown.5 trên hành lang đã biết trước finefeedback20pulse/.045/tol12; không tăng bounds
hoặc gọi gamefunction/writeactor. Helper giữ variant neutral6s cho labelcontest-returnsetup đã có CP159,
global3s không đổi; phân biệt với mô tả sơ bộ3s trong source-first củar1.

R2 cùng source/profile/copy47pins/291/nameguard/immediate165, hai epoch/keys12345→12346. Mỗi vòng
hai OSkeys có1hoặc2actual124 và mộtwinner/confirm/view, native +1speed đúng vai, nativebaseline A5/B4
trước pickup vòng2; không fixedwinner hoặc fairnessclaim. Sau B về y399/A159, normalASpace/power1
cho actual159/158/IDs1,2 rồi A17→19/B21, hai view loser56→1/other0 trước175; đủ175302/1803/errors0,
roomrefresh/Ready95/BStart2 và Ready2. Bốn peerwindows/two keys và nativegetters cùng-run có report riêng.
R1 A thắng (hai claim), R2 B thắng (một claim): chứng minh reset A6→5 trước vòng2; không Start3
sau B thắng nên B5→4 sau thắng còn chưa đo runtime, giữ giới hạn variant.
Không hidden27/28pickup/fullthreeeffects hoặc Bbomb/EOF/oldUDPinjection trongrun này; những proof đó
ở [CP188](life-results.md#current-basic-cycles-291)/CP155, không ghép thành cùng một run.

Codex đối chiếu tiêu chí/authority: **K6a đạt scope hai actualclient cùnghost/offline/basic đã pin**,
[quyết định/giới hạn](../evidence/peer-endpoints/k6a-acceptance.json). K6bLAN/K6cInternet/K7, bốn/tám actualclient,
fullauth/fullcodec/allmaps/items/generalreset/142freshness/distance/originalauthority còn mở. GateK6
trong catalog vẫn open cho các môi trường ngoài K6a; script không tự xác nhận semantics hoặc acceptance.

<a id="current-pair-round"></a>
### Hai chu kỳ A/B trên currentprofile — CP061

**kr_runtime/confirmed hẹp trên Winecopy**, [recipe/raw/hash/phép kiểm](../evidence/current-pair-round/README.md).
Tái dùng two-cyclepath CP048/055/056 và profile41 CP058–060; không newnativebody/patch/servicecode.
Cùng8connections/IDs1,2/slots0,1, actualStart/Ready/99/121/153/clock hai lần; keys12345→12346.
Mỗi vòng **actualA124→165 cả hai**, secureRollerA5→6, vòng2reset5 trước nhặt mới;
Bcurrent4 giữ. Mỗi vòng A118, service cấp ID1→2,159/158→A17/19→B170/B21; stateA1 ở cả hai
trước175, Bkhôngstate1,175302/1803/errors0/phòng/refresh hai lần. Reportmarker17/19 vòng1 không chặn
actualpickup vòng2; claim/catalog/key/stat được reset hẹp, không toànentity/queue/transport/reset proof.

Hai nativeview cùng tọa độ tại clock và trước175: đầu vòng cả haiactor(120,159); sau input,
A=(120,289) vòng1/(120,279) vòng2, B=(200,159) cả hai, khớp ở A/B.
Đây là đồng thuận **sampled movement hai actor trong run**, không fullgrammar/pxs/correction/UDPcausality.
AuthA/B137/errors0/namedcacheaccepted/no528 giữ; sau chỉnh UI delay12s input2round không forcefunction.
Không dùng processalive/encoder tựsinh hoặc2rank để nâng fullmatch/K6a/LAN/Internet.

Tại CP061 chưa có currentservice hai người cùng tranh index/đảo winner; [CP062](items-stats.md#current-pickup-race)
đã đo scope này với delivery165 giữ2s. Duplicate theo actual transport/fullfairness vẫn mở;
CP030 oldharness giữ provenance riêng, thứ tự hiện hành chỉ ở PLAN.

<a id="portable-probe-bundle"></a>
### Service và fixture có thể chuyển thư mục — CP190

**design/implementation; kr_runtime confirmed chỉ trên copy Wine/privatevirtualhosts trong recipe đã pin.**
[Recipe/hash/giới hạn](../evidence/peer-endpoints/README.md#cp190), [cách dùng](../../tools/README.md#portable-bundle).
Tái dùng exporter/preset47fixture CP159/171 và profile/currentnative đường CP188; không RE thân hàm native
mới, không sửa wire/client/assets/sourceDB. `export_probe_profile.py --preset ... --bundle` thêm nhánh
opt-in, `--out` trở thành thư mục mới trong `local/`; thiếu preset, output có sẵn hoặc source pin sai bị từ chối.

Gói chứa profile với path tương đối,43 fixture files cho47alias (dedup theoSHA),13 module runtime Python
nguyênbytes, initializer và launcher. Không client/asset/DB/test/cache. Manifest ghi size/SHA của59file,
contextcommit là baseline khi export, không tự hàm ý source đã merge. Đây là kiểm drift, không chữ ký tin cậy.
Launcher kiểm manifest/path/hash/profile trước listen, cố định profile đã gói, tự cwd/sys.path vào rootgói;
chạy được bằng Python `-I` từ cwdkhác/đường dẫn cóspace. Profile override và drift bị từ chối. Partialexport
không có finalmanifest nên launcher fail, không xóa dấu vết hoặc overwrite output.

Unitcontrol chuyển gói, xóa fixture nguồn tổng hợp rồi actualTCPbootstrap/SIGTERM/cleanup; nativecontrol
riêng dùng đúng47fixture/policy/13sourcebytes sau relocation, Python-I tải module **trong gói** (startupJSON).
Hai actualKR copies nhận291128/128/errors0/names/renderguard1 và request6guard, S2Csequence43+3n;
room/hostBack/rejoin/Ready/firstnative12199clock. BSpaceID1→ownedBEOF còn5.197152337s trước6sfuse;
A đọc175302/1803/107ID2clear/chief110/room, pendingjobround_closed/no158pastdue+4s.
Run187.247701577s/690raw/frozen229/exit0/cleanup0;1046EOF/relocationchecks+66identity/seqchecks.
Không two-normalcycles/Start2/Bnhậnforfeit hoặc physicalLANproof; K6a đã có ở CP189, không nâng K6b/K7.

Tiền kiểm `accept` TCP sauCP189 được gom CP190: AAAABBBB đúng8/8, ABABABAB sai6/8 role/stage.
`--peer-ip` là outgoing153config; admission vẫn globalordinal/4, không incomingidentitybinding. Recipe LAN
đầu cần startup tuần tự như K6a. Tại CP190 chưa có máy vật lý thứhai; CP191 chủ dự án chấp nhận [LANgiảlập](networks.md#emulated-lan-acceptance).
Bundle không thiết lập isolation,
routing/NSS/firewall hoặc triển khai tự động, fullauth/NAT/relay và resetvariants vẫn theo RESEARCH.

<a id="production-baseline"></a>
### Hai vòng bằng service production, không hold165/replay — CP159

**`kr_runtime/confirmed` hẹp; policy vẫn `emu_choice/design`.**
[Recipe/hash/checks](../evidence/peer-endpoints/README.md#cp159). Tái dùng CP157 currentdirect profile,
knowncodec/consumer/actor/terrain/reset và CP158 counterlimits. Không thân hàm RE mới. Chỉ thay
measurement wrapper bằng `python3 -m server.probe_service`, bỏ active oldUDP replay, dừng sau native
return2/actualReady95/110. Profile/game/30s/loadinggate giữ nguyên, 96 frozeninputs; không source/config
change giữa run. Không tắt native observer/copy patches rồi suy tương đương Windows nguyên bản.

Cùng run:8actualTCP/2client/wineserver privatehosts/directchannel/UDP, key12345→12346/2native12199clocks,
4movementwindows/nativeapply/currentprogress. **Không** trường measurement/hold165, không active replay.
Fire item17: R1 chỉ actual A124; R2 actualA124 nhận/B124 muộn rejectconsumed. Mỗi item/view chỉ1native165,
full12/errors0, chỉApower1→2, statbaseline reset hai vòng. Service broadcast delay đo0,2508–0,42968ms
(đoạn code service, không latency mạng/SLA/originalfairness), khác 2shold của phép tranh chấp cũ.
HaiOSkeys không bắt buộc phát2request khi một client đã thấyconfirm và itembiến mất; không nới luật
claim/key/consume để đạt. 75pickupchecks/12actualC2Sjoins/2214nativeactorchecks tách theo source.

BomactualID1/A/power2 vàID2/B/power1;158/159full/195flags/reset khớp scope26checks. Native12717→19/
opposite21/chosenrank hai vòng; strict lifecycle checker23checks ở run này cóloser56→1/other0before175
cảhai view/hai vòng,16copiedgettermodel riêng. Điều này không bác transientCP157 hoặc controlCP158.
Hai native175full302/180full3/110full6/errors0, chosenrank2,1→1,2; BReady95 thật/native110 sau return2.
Không EOF/thirdforfeit/Start3 trongrun. ẢnhAStart rõ tạifocused4/native110, mộtframe ngayđóngrun trắngnút;
không suy persistentrenderfailure hayStart3thànhcông từ cácframe. CP110 knownvisualbehavior giữnguồnriêng.

**Tái tạo saucleanup:** [preset metadata](../../server/profiles/two-client-battle.json) gồm policy và47localfixture
relativepaths/size/SHA/seq; không chứa binary/asset. `tools/export_probe_profile.py --preset two-client-battle`
xác minh pin rồi chỉ rebasepath dưới source-root, từ chối input đổiSHA/size/symlinkthoátroot, không tự
rehashđểtinfileđổi. Exportobject bằng exactprofile đãđo, productionservicepreflight47fixtures/4exporttests pass;
**runtime dùng exportedfile chưa đo tại CP159**, đó là bước kế. Default/options cũ giữ hànhvi.
Fixtureauth/room/map/rank vẫn synthetic/policy, template không phát client/DB/pack hoặc fullauth.

CP158/PR229 đã merge259282b; CP159 self-check/reviewwaived/chờPR. GateK6a/fullmatch/Windows/LAN/
Internet/8client/sharedgeometry/generalreset/originalauthority không nâng. Audit AUD047qua156/kếCP160.
[PLAN](../PLAN.md#current-order) smoke exportedpreset/normalStart3 + periodicmanualaudit; tránh mở lại
itemconsumer/counterRE. Acceptance theo đúng environment/behaviors measured, không tổngsốchecks.


<a id="exported-preset-runtime"></a>
### Pinned preset đã chạy, Start3 thật và AUD048 — CP160

**`kr_runtime/confirmed` scope copyWine/productionpolicy**;
[run/recipe/hash](../evidence/peer-endpoints/README.md#cp160). Runtime service argv chỉ đúng fileprofile xuất
bằng `--preset two-client-battle`; object/47fixturepins đúng preset đã commit ởPR230/merge2a7332c.
Productionmodule trực tiếp, khônghold165/activeUDPprobe,98inputs frozen, no source/configchange.
Tái dùng CP159native/encoder/getter/cycle và CP110/156normalStartbutton, khôngbodyRE mới.

Cùngrun 394,801002246s/exit0/1313raw: hai normalcycles/actualReady2 rồi normalAStart3/actual104,
key12347/native121129/full99/barrier trước30s/nativeclock cảhai, khôngbomb/accepted17,19 ởvòng3;
render+8s cảhai view cómap/2actor/flooritems/timer1:50. Hai normal175302/1803, **khôngthirdoutcome**.
EOF/abortgeneration3 xuất hiện trongownedcleanup **saucompletion** khoảng358,7ms; khônghostfault/UIexit
proof hoặc thirdforfeit. NativeStart3 chứng minh nút hoạt động trongrun này, khôngwholeUI/nointermittentbug.

75pickupchecks: actualFireAonly mỗi2key/currentone165/view/onlywinnerpower/reset; khôngrequire2reports
khi khônghold. InheritedCP159checker ban đầu cốđịnh counts1rồi2 FAIL vìrawlà1rồi1; giữinheritedsource/log,
checktheo plannedcriterion1or2/oneeffect/currentkey, không sửa serviceguard/runtime để pass. Latencyservice
observed0,239821–0,41538ms khôngSLA. 44cycles/689peer4windows/24loadinggate/26entity195flags/23strictlife;
2184actor/444peer/2758flag+2730array/16life/321nativepacketmodels tách source/runtime/emulation.
Native195seeds ởStart3 khớp roundseed, currentstrict56→1 hai view/hai vòng không bác naturalCP157.

**Lịch sử CP160: gap UI/service khi đo, đã sửa phạm vi hẹp tại [CP161](room-membership.md#current-room-title):** actualC2S81title làcp949 “매너겜 필수” trongrun,
khác row91guest fixture “KRRoom”. Host giữ tên từdialog/context, guest nhận fixedrow, hai view ảnhphòng
khác tên. Native14EF300/14EB9E0/12u32/4u8 đọcfull67byte, CRC/partial controls đạt; không suy hai phòng
hoặcroomID khác nhau. Reuse [CP016request81](room-membership.md#room-create), [CP028row91/87](room-membership.md#duo-room-round),
[CP076inline/heap](room-membership.md#back86-membership); VN chỉpattern/encodingreference, không port opcode/layout.
Phần cần làm là titleecho trongknownrow/session/currentmembership vớibounds/hash/log, không RE lại
handler hoặc khai thácunknownfield. Giới hạn implementation/negativecase/runtime ở RESEARCH/PLAN.

**AUD048 manual:** đọc README→PLAN→authority/PROTOCOL→RESEARCH→evidence/raw CP157–160. Sửa authority
currentrows cònCP156/hold2s/UIpending, cập nhật PR230merge/presetnativeproof/nextactualroomtitlegap,
giữ lịch sửCP157failure/CP158controlledcausality/CP159immediaterenderframe. Counterqua160/kế164.

Đối chiếu K5/K6a: localtwoactualclients/identityslot/2waymovement/2ownerbomb/pickup/result/reset/Start3
có evidence đúngprofile; nativeflags195 choactualboxclear vàseedreset trongruncurrent, không mặcđịnh
fullgeometry/hiddenitem/drop. Hostleave controls CP155–157 là cácscope riêng, khônggộpthànhmộttrận.
Roomtitleconsistency cònfixpending tại CP160 (CP161 giải known variant); naturalCP157cause/wholequeue-seq/fullauth/originalauthority/Windows/
LAN/Internet/8clients/generalreset/sharedgeometry/CP137complexUI giữunknown. **Khôngđóng K6a/fullmatch**
từpreset/positivecontrols/đếmassertion; cũng không đòi fullservergốc before mọi localpolicy demo.


<a id="rust-m3-acceptance"></a>
## Rust M3 Battle basic hai client: nghiệm thu có giới hạn — CP242

[Checklist thủ công](../evidence/rust-service/m3-acceptance.json), [metadata](../evidence/rust-service/m3-controls.json),
[recipe/raw/hash](../evidence/rust-service/README.md#cp242). Quyết định Codex tự đối chiếu dưới review waiver:
**RUST-M3 đạt scope** hai identity diagnostic/KR00967/Wine/LAN giả lập/ExVillage01/basic ba item.
CP237 đã có movement hai chiều; CP238 hai owner bom; CP239 terrain/ba effects; CP240 hai outcome đối nghịch;
CP241 pending-hostEOF→survivor Room/IDclear/chief/no158. Mốc mới chỉ đo khoảng trống, không nghiệm thu lại mọi positive.

`kr_runtime/confirmed`: hai epoch đều actual124→mỗi view đúng một165/index17/28/27, effects Roller4→5 trên B,
Fire1→2/Bomb1→2 trên A; clock1/2/3 baseline stat và195 flags/view khớp map seed. Hai OS input ADown/BUp đồng thời,
B thắng Roller cả hai vòng, **mỗi vòng chỉ một actual124 cho index17**; two-claimant/duplicate thuộc kernel control riêng.
Bốn actual118/two owners→159full39/entity-ID→158full13/kind0/cleanup/terrain khớp model Python độc lập.
Hai vòng mới cùng B chết, hai view175outer302/inner292/rank1-2/1803/refresh/Ready/Start3/hold6s;
không gọi run này là hai outcome đối nghịch: điều đó đã có CP240.

Gói17byte B→A ở vòng1 được capture và replay **toàn Ethernet frame không sửa byte** ở vòng2: PCAP có hai bản
byte-identical với checksum IPv4/UDP đúng; native ingress nhận, inner13byte key khác current→key-drop/noapply.
Cửa sổ riêng BRight hiện tại có native API/key/apply/peerposition thay đổi (sáu packet hai vòng).
Không suy replay auth/current-key forgery/low8 wrap đã giải. 1.006TCP/126reply/alltuple-C2Ssequence/CRC/SHA joins,
4.746 copied52A350 getters tách khỏi Wineexecution. 84raw/306,382s/frozen119inputs/cleanup0.

`design`: test outcome queue giữ Armed và không phát cho ai khi recipientB không reserve đủ3 entries;
retry phát175/180/refresh đúng một lần, giữ selector1, chỉ Idle sau hai refresh; oldrefresh/outcome không mở vòng mới.
44Rusttests/51kernelTCP negative/lifecycle đạt; negative synthetic không là chứng minh native tấn công.
Logger writer được supervise trong main select, lỗi gốc ưu tiên sau cleanup, kèm write/flush/eventordinal;
fault sink error/pending và clean drain kiểm riêng. Giữ write/flush2s, emit200ms/shutdown3s.
R4 native bị unavailable che sink error; kernelR2 bản sửa thấy flush timeout event141. Inputcopy lớn đồng thời là
context, chưa đủ causalproof. RepeatR3/nativeR5 đạt; **chưa rõ nguyên nhân flush stall**, không nhận đã sửa I/O latency.

Không fullgame/fullauth/inventory/reward/generalroom/capacity4–8/kim-cứu/allmaps/matchrestore/production;
loadingEOF/selectedEndingEOF vẫn giới hạn riêng. K0–K7 lịch sử/K6c WAN giả lập không đổi. Internet thật chưa kiểm.
[PLAN](../PLAN.md#current-order) tiếp boundedworker/idleTTL/rate M5 độc lập rồi M4; audit kếCP244.

<a id="rust-pr329-integration"></a>
## Đồng bộ PR329/RoomRules sau account/basic — CP259

[Chỉ đạo ưu tiên mới](../evidence/owner-directives/pr329-integration-order.json),
[metadata/build](../evidence/rust-service/pr329-integration.json), [PLAN](../PLAN.md#current-order).
Nguồn **design/design**; các phép kiểm source, build và TCP được phân biệt với native.

Nhánh `rust/m5-account-basic` fast-forward tới main `1b8d6f2`, gồm PR329/`073700e` và PR330/CP258.
Sáu cờ của Runtime nay nằm trong `r.rules.*`; `life_enabled`, `team_results`, `owner_bombs` thuộc `RoomRules`.
Cấu hình JSON giữ nguyên. Không thêm cờ theo sức chứa, alias identity hoặc nhánh capacity mới.

Sau gộp thật: fmt, clippy, 86 test workspace (81 service + 5 wire), locked build và 59 ca account TCP đạt.
143 frame phản hồi phòng khớp alias/opcode/sequence/size/SHA với CP258 theo từng identity.
Đây là kiểm TCP tổng hợp. Native CP258 vẫn pin binary trước R1a; hồi quy native 2/4/8/account ở cuối R1.
WIP projector 5 file được giữ bằng snapshot và stash; 88 test Rust/17 ca TCP đạt, chưa native hoặc merge.

Ưu tiên mới: N1 → R1b–g → M7 → G1 song song M6 thu hẹp → M8. N1 phải đo cổng 99 và tuple UDP sau NAT,
từng chiều, đích native trả lời (nguồn quan sát hoặc 153), hai NAT khác nhau và ca chung NAT.
Kết luận (a) trực tiếp, (b) học endpoint hoặc (c) relay cần bằng chứng theo acceptance.
12 payload nonce qua WG đóng gói trong SSH/TCP và UDP trực tiếp 0/12 không quyết định được ba nhánh này.
Chủ dự án đã tạo policy và dynamic group. Fresh signer 11/11 12:22 UTC vẫn trả `GetVnic 404
NotAuthorizedOrNotFound`; cấu hình domain/matching rule/policy/root compartment còn cần đối chiếu.
Chưa đọc được ACL, chưa sửa cloud ACL hoặc chạy native N1. Rust ARM64 trên VPS đã locked build và
86 test đạt từ archive main `1b8d6f2`; không mở game listener. Build/test không chứng minh UDP Internet hoặc native game.

Đích giai đoạn 1 là dịch vụ nội bộ 50 người/8 phòng và Internet thật; dài hạn giữ đầy đủ gameplay.
N1 là dependency của đích mới; scope K6c/K7 bằng WAN giả lập giữ nguyên.
Map/item G1, M4 Free 1v7 và thưởng ngoài mức 0 chờ chủ dự án. **Internet thật chưa kiểm** ở mức native game.

<a id="rust-r1b-policies"></a>
## R1b — chính sách phòng có tên, profile cũ giữ wire — CP262

Nguồn **design/design** cho cách tổ chức source; không RE hoặc đổi authority KR gốc.
[Metadata/hash/kiểm](../evidence/rust-service/r1b-policies.json), [thứ tự/acceptance](../PLAN.md#rust-r1).
Reuse R1a/PR329, CP259–260 source và ownWIP ba file đã giữ trong stash; source của các đường phòng khớp
baseline đó trước khi khôi phục. Projector stash riêng không được pop hoặc ghép vào refactor.

`RoomRules::from_config` chuyển JSON cũ thành `ResultPolicy` (Disabled/Duel/Teams), `DeparturePolicy`
(Leave/Forfeit/UnrankedReturn/FenceActive), `BombPolicy` (Basic/Disabled/OwnerReports), `LifePolicy`
(Observe/DuelReports/TeamReports) và `TeamPolicy` (Any/BalancedManner). Các lựa chọn compatibility của
profile2/4/8 giữ ý nghĩa cũ; không thêm trường `four_*`/`eight_*`, alias identity hoặc nhánh sức chứa mới
trong packet handler. Các điều kiện legacy theo profile được normalize ở admission trong `rules.rs`.
Cờ diagnostic `ninth_admission_probe` đã có vẫn giữ; chưa phải general admission/registry.

Fixture `rank` tiếp tục cung cấp bytes, **không bật luật tính kết quả/rời trận**. `tick` chọn `ResultPolicy`,
lifecycle chọn `DeparturePolicy`, báo life chọn `LifePolicy`; điều kiện Manner cân đội chọn `TeamPolicy`.
Query `life_enabled()` giữ ý nghĩa legacy TeamReports cho phần model chung; `reports_enabled()` gồm
cả DuelReports. Test negative có fixture rank/mature result nhưng ResultPolicy::Disabled giữ Armed/không output,
sau đó policy Duel mới đi đường queue error cũ. Không gán design này là authority của server gốc.

Kiểm cùng hai binary trước/sau:90Rust/fmt/clippy/lockedbuild,12suite239TCP và59account TCP mỗibinary.
13suite/298ca/3189roomframes khớp identity/normalized visit/alias/opcode/sequence/size/fullSHA; không so thứ tự
phát toàn cục giữa các recipient độc lập. Các guard lỗi, pending-bomb EOF, stale/duplicate/reset và ownership
trong bộ hiện có vẫn được assert; đây là syntheticTCP/differential, không native KR mới.

Hai lần so ban đầu phát hiện khác ở **unread finally-close notifications**: eight-team có2remove107 thêm;
ninth khác7vs1remove107 trong đợt đóng cuối. Không loại tùy ý theo alias hoặc nhận nguyên log là giống.
Thêm phép ghi receiver.next_sequence ngay trước finally, giữ reference cả H đã EOF và visitor9, rồi chỉ chạy lại
hai ca đó trên hai binary không đổi. Tất cả frame đã được driver đọc khớp; explicit H EOF/visitor-denial/fences
vẫn trong acceptance. Raw cuối giữ cả tail không đọc; phần tail không được claim byte-identical.
Bốn người cũng ghi boundary tương ứng. Các tool chỉ bổ sung phép đo, không đổi input/response/assertion của service.

Không đổi codec/header/packet/field51 policy, room identity/slot/host/team assignment, số phòng hoặc binding theo IP;
R1c–g còn lại theo PLAN. N1 native2account/gameplay đã đo tạiCP261 vẫn gắn source/binary ARMCP260;
R1b candidate chưa chạy native và không thừa kế certification đó. Cuối R1 vẫn phải chạy native2/4/8/account.
N1 mobile/ISP-CGNAT còn chưa xác nhận; sharedNAT fixedIP limit cần R1f. Không đóng N1/fullR1/M5/M4 hoặc production.

<a id="rust-r1d-wire-builders"></a>
## R1d — header chuẩn và builder theo opcode — CP264

Nguồn **design/design**, kiểm source/unit/TCP; [metadata/hash](../evidence/rust-service/r1d-wire-builders.json),
[dashboard/thứ tự](../PLAN.md#dashboard). Dùng lại CP263 layout106/slot, CP202/210/213 rank/return, cùng pure serializers
của bombs/pickups/life/room_wire hiện có; không mở RE consumer mới hoặc port wire VN. Survey50 frame có cùng header
sau bỏ opcode/length mà encoder vốn ghi lại; helper Rust tái tạo đủ50 frame với body do Python decoder cấp.
Đây là input fixture equivalence, không oracle native mới.

`kr-wire::kr00967` giữ một header đã pin cho profile mode2/selector3 và các body builder:
room81/82/86/87/91/106/107/110/121/153/585, bomb158/159, pickup165, report127→170, rank175/return180.
Builder chỉ nhận field/template; không nhận account/session/room model/timer hoặc quyết định authority.
Profile106 zero/optional limits giữ nguyên; byte opaque vẫn từ template có pin, không tự gán nghĩa.
Service giữ policy admission, identity/slot/team, winner/settle, queue/bomb/life/generation fences, lựa chọn endpoint và kết quả.
`room_wire` chỉ còn adapter membership→field, `four_results` giữ winner; helper field-byte dùng ở wire crate.

Alias JSON cũ được chuyển một lần thành `Template` có tên tại `room_packets::normalize_legacy`. Runtime không lookup
fixture alias hay mượn header từ `start`/`player_A`; packet mới có opcode tường minh, header chuẩn và body do builder cấp.
Các body opaque dùng lại giữ trong enum template; việc nạp JSON vẫn kiểm hash/opcode/size và không đổi input format.
Header không thuộc profile bị từ chối trước bind, kể cả mọi fixture cùng một header lạ với hash hợp lệ.
Log alias để tra evidence giữ nguyên; nó không chọn packet bytes. Chưa full grammar/modes khác hoặc general profile.

96 test Rust/fmt/clippy/locked build và14 suite/306 ca TCP đạt; source/binary/input cố định, cleanup từng suite đạt.
298 ca cũ có3169 frame đọc được khớp identity/normalized visit/alias/opcode/sequence/size/fullSHA với CP263;
ca first-vacant thêm24 frame khớp. Sáu pre-bind guards +hai slot lifecycle là8 ca trong suite mới.
Phạm vi nhỏ hơn3189 báo trước do dùng actual receiver boundary cho cả eight-manner và manner-negative ở lần so này:
6 và14 notification đóng cuối của baseline chưa được driver đọc. Giữ nguyên raw cả hai bên, không loại theo opcode/alias.
Ca eighth EOF và visitor negative vẫn được assert; không claim unread tail byte-identical.

WIP foundation ban đầu truyền slot đã encode vào builder110 dẫn đến encode hai lần; run lỗi/raw đã giữ, cleanup hoàn tất.
Sửa adapter decode slot trước builder, thêm test đúng điểm nối,43 TCP/56 frame đối chứng đạt trước khi mở chuyển toàn bộ.
Bộ đầy đủ hiện đạt trên source sửa; không sửa raw hoặc record mốc cũ để đạt checker. Main PR339–341 đã gộp sau seal run,
Rust source không đổi, kiểm sau gộp đạt; CI dùng toàn module catalog test đã được sửa ở main.
Binary mới chưa native, không thừa kế CP261 hoặc các gate cũ. R1e/N1b/R1f/R1g và native cuối R1 còn theo dashboard.

<a id="documentation-checkpoint-identity"></a>
## Danh tính checkpoint thuần tài liệu — correction CP268

Nguồn **design/design**, kiểm tool/schema; [metadata](../evidence/rust-service/documentation-identity.json),
[workflow](../evidence/checkpoints/README.md). CP267 khai `artifacts=[]` vì không có binary/runtime mới; kiểm lịch sử đầu
xác minh424095 pin nhưng lỗi schema "artifact identity and locators/run IDs required". Không sửa complete record,
audit hoặc evidence CP267 để vượt checker, không gọi lượt lỗi là đạt.

Checker nhận danh tính từ tài liệu chuẩn có pin khi artifacts đúng mảng rỗng và **mọi** finding có source design,
method documentation, status design/candidate/unknown. Mỗi canonical phải là docs/*.md có kind repository/size/SHA
trong evidence; mỗi finding phải dẫn ít nhất một tài liệu đã pin. Locator và các kiểm hash/history/continuity/audit/
docs_sync/evidence-reference giữ nguyên. In warning riêng không chứng nhận implementation/runtime.
Kr_runtime/kr_static, method runtime/static/emulation/test, status confirmed hoặc thiếu canonical pin vẫn không được
miễn artifact. Sourcepin tài liệu không chứng minh nội dung đúng; audit thủ công/acceptance vẫn giữ vai trò cũ.
57 test checkpoint gồm positive/docs-pinned và negative hash/source/method/status/canonical/locator đạt.
Đây là correction công cụ, không có contract KR hoặc runtime/acceptance mới; thiết kế CP267 và gate cũ giữ phạm vi.

<a id="rust-r1g-admission"></a>
## R1g — ngân sách transport và allowlist — CP270

Nguồn **design/design**, kiểm source/unit/TCP; [metadata/hash](../evidence/rust-service/r1g-admission.json),
[native/việc tiếp](../PLAN.md#rust-r1). Dùng lại semaphore/accepted queue/allowlist/channellease đã có, R1fprofile theo
session và các byte fixtures đã seal; không contract wire/RE mới hoặc tự port protocol VN.

`network::admission_budget` kiểm `max_connections`1..1024 và `allowed_peers`1..256entry trước bind. Đây là giới hạn
operator design; không là giới hạn server KR gốc hoặc benchmark an toàn. Các field/default/giá trị cấu hình cũ
không đổi; operator có thể chọn256transport cho đích khoảng200+headroom, nhưng tải thực thuộc M8.
Accept queue32 và logger256 vẫn giữ budget riêng, private launcher/DB worker/profile count không bị tự nâng theo
max_connections. Không suy trần socket cho phép50account hoặc8phòng khi M5generalprofile/M7 chưa có.

Bỏ điều kiện số IP allowlist bằng số actor và so tập allowlist bằng tập profile. Từng actor configuredpeer vẫn phải
được allow; diagnostic duplicateactor endpoints vẫn bị từ chối. IP allowlisted thêm không tự chọn identity: diagnostic
không có mapping đóng với `unmapped diagnostic actor`; account cần frontend/session/DB/channellease và knownprofile.
R1f currentchannelendpoint/LegacyDirectPlan vẫn giữ, không NATlearning/relay, không cờ hay nhánh sức chứa mới.

105Rust tests/fmt/clippy/lockedbuild đạt. Native-tested fixtures chỉ làm input độc lập;14suite/306TCP legacy đạt,
3169consumedroomframes+24firstvacant khớp CP269 theo fullSHA/sequence/opcode/identity/normalizedvisit; unreadtail giữ.
10ca admission kiểm65TCPtransport cùng giữ/hello thực nhận,66thbị chặn, một EOF trả permit rồi admit lại, source bị
cấm không chiếm permit, bounds0/1025/empty/257peers bị chặn trước bind, allowlist3IP/profile2 không cấp identity cho
IP không mapped. Số socket không phải số người KR; hello/TCP/proclog không được dùng làm native gameplay proof.

Lượt hồi quy đầu chạy sai cwd: binaryR1g nhưng snapshotrepository là R1f; raw/cleanup giữ cùng addendum, không dùng
để chứng nhận source. Lượt2 chạy đúng worktree, có cwdassert và pin producer, source/tool/binary/fixture giữ nguyên
đến seal; ownedPID cleanup đạt. Không sửa lượt cũ hoặc lấy reportpass của nó thay bằng chứng source đúng.
R1f PR347 đã merge; fullR1 vẫn cần native2/4/8/account. Relay/currenttwoNAT, M5generalprofile/nativeDBqueuefault,
M7multiroom, key/portreuse/restart và workload50/8roomM8 chưa chứng nhận. K6c/K7/N1 scope cũ giữ nguyên.

**Correction CI CP271:** test race hai kết nối vẫn phải có đúng một grant; bên thua được trả `ActiveSession` hoặc
`Busy` theo backend hiện hành (SQLite writer wait 250ms; KDF cũng có budget). Log CI cũ chỉ chứng minh assertion
đếm `ActiveSession` thất bại sau assertion một grant đã đạt, không in lỗi thực của bên thua; không gán chắc nguyên
nhân SQLite cho run đó. Test mới chỉ in tên outcome/error đã redacted, retry sau hai thread phải là `ActiveSession`.
Một connection khác giữ `BEGIN IMMEDIATE` ép `Busy` hữu hạn (<2s); rollback rồi retry phải là `ActiveSession`.
Kiểm một session/token hash và không plaintext vẫn giữ. Đây là sửa test, không đổi timeout, auth policy hoặc runtime;
không thay CP270/raw cũ hay chứng nhận native DB-queue fault của M5. [Metadata](../evidence/rust-service/r1g-ci-correction.json).
<!-- protocol-topic-content-end -->
