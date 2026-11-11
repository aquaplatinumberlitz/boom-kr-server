# Bom, serial, nổ và capacity

[Mục lục](../PROTOCOL.md) · [Câu hỏi đang mở](../RESEARCH.md) · [Thứ tự việc](../PLAN.md#dashboard).

<a id="c2s118-admission"></a>
### Admission request đặt bom118 — CP067

[Recipe/native/control/hash](../evidence/c2s118-admission/README.md); **kr_runtime/confirmed** cho
consequence trên copy, **design** cho guard. Tái dùng CP021/025 cachedwriter/fullu32serial/poolmatch,
CP065/066 profile/captures; không dùng oldS2C159/158 như C2Sadmissionproof hoặc thêm body native.
E2DF80/E2E03D lấy firstu32 từrecord+8, E2E0B3 serialu32 từ+0C;2vùng khớp4artifact. Actualrecord/
TCP snapshot giữ firstfield12345→12346 khớp key153 hiện hành quaStart, serial0 lặp. Constructor/key
source/serialgenerationwrap/authority chưa phục hồi; field0bằngkey không đồng nghĩa authentication.

**Control:** exactactual118duplicate vòng1 và oldbody sau current39vòng2 tại dispatcher khiến service
cấp extraID2/3 và timer. NativeA159 trả0 cho hai extra (không pending), B tạo cả hai remoteentity;
oldID3 còn vào kind0 khi timer158 tới ởB, không ởA. Đây là consequence cụ thể, không chỉ logallocate.
DuplicateID2 đã có entity nhưng callback158 khôngkind0lặp; chưa tách chain/samecell ở nhánh này.
Replay bypasssocket/framer/seqadmission; không actualTCP replayproof. Baseline chỉpartialround2.

**Policy caller:** sau layoutflag0/boundowner, log `bomb_requests`; so fullu32field0 với currentroomkey
trước reserve/ID/timer. Giữ mapserial→priorID/bodySHA riêng boundconnection/round, xóa ởStart.
Same serial/body bịduplicate; same serial/bodykhác bịconflict; không resend159 vì nativepending đã dùng.
Không áp serialmonotonic hoặc so tick127/124. Không thay RoundFence: oldtimer vẫn giữ ticketgốc.

Guardrun cùngprofile/3injection:loại2duplicate +1oldkey trướcID/jobs/159; haiactual118serial0quaStart
đượcID1/2, nativeentity cảhai, hai bom/state/result/return/refresh/errors0.64regression gồmfullkey+256/
newseqsamebody/conflict/perowner/roundreset/nonmonotonic/hardlimits; wrapnumeric chỉpolicytest,
không runtimeallocator/serialwrapproof. Chưa cap/geometry/actorstate/fullvalidator/auth/forgednewserial/
firstfieldconstructor/fullreset/K6a/LAN. Khoảng trống currenthiddenitems của CP067 đã được đo ở
[CP068](items-stats.md#current-hidden-items); thứ tự hiện hành chỉ ở [PLAN](../PLAN.md#current-order).

<a id="bomb-position"></a>
### Echo bom, nguồn spawn và map cụ thể — CP024

**kr_static/kr_runtime confirmed trong phạm vi copy offline**, encoder/profile/policy còn là
**emu_choice/design**. [Evidence](../evidence/bomb-position/README.md) khóa năm run tuần tự,
10actual118,56native getter trên snapshot, control/ảnh/hash. Copy SHAe9bb47… không thêm patch;
13vùng code khớp static copy660447… và modO memory-layout a08950….

**Lịch sử CP024, chỉ bom đầu/serial0:** stub lúc đó chuyển field3/u32 của118 vào field2/u16
của159, giữ field3/u32 của159=0. Vì chỉ đo serial0, cả hai field đều0; đây là candidate chưa phân biệt
được serial. **CP025 bác cách ghép tổng quát này** bằng bom thứ hai/serial28, sửa ở
[contract serial](#bomb-serial); raw/checkpoint CP024 giữ nguyên. Hai byte hàng/cột vẫn được kiểm
độc lập. Byte thứ ba từng gán power candidate bị CP031 bác bằng Fire; xem [mapping power](items-stats.md#item-effects).
Các tọa độ/spawn/profile dưới đây giữ phạm vi CP024.

**Tọa độ đã đo:** getter`B335C0/B335E0` đọc secure wrapper actor+4C/+58 qua`57A870→52A350`.
Snapshot gồm wrapper và vùng data trỏ tới; checksum được giải thủ công rồi gọi native52A350 độc lập.
Info159+18/+24 giữ hai field A/B của bộ ba vừa nêu; snapshot bom trả từE5ECA8 có các cặp sau:

| Profile/run |118 A/B |159 A/B động |Bom getter+4C/+58 |
|---|---|---|---|
| Antarctic/position-dynamic vòng1 |100/1 |100/1 |40/4039 |
| Antarctic/position-dynamic vòng2 |100/3 |100/3 |120/4039 |
| Village00 hai vòng |11/11 |11/11 |440/479 |
| ExVillage01 hai vòng |3/3 |3/3 |120/159 |

Các mẫu khớp`x=40*B,y=40*A+39`; A/B tương ứng hàng/cột trong phạm vi này.
Không áp công thức/thẩm định va chạm cho mọi mode từ bốn mẫu. AI interface`1775F08/CGOAIBattleBomber`
trả ởE14600 khác base actor; dump4C/+58 của interface đầu là sai, checksum fail và bị loại rõ.
Không suy địa chỉ pointer mới hoặc được tái dùng là reset state đầy đủ.

**Nguồn spawn đã nối:** `E57D76→12418A0` lấy index từ slot+7C đã được121 đặt0/1;
`CGameStage/v2D0→E59F30→E5A290→E0F6C0` đưa vector index tới`EAF910`.
Hàm này lấy point8byte từ mapinfo+368, ngoài khoảng trả0/0; runtime thấy slot0/1 hợp lệ,
slot2..15 index−1. `E0F770→vf/B3D170` khởi tạo **actual actor** tạiE0F970/E0F973:

| KR code/name |Point spawn0/1 từ map |Actor pixel0/1 sau init |
|---|---|---|
|0/Antarctic01 |(6,100)/(4,100) |(240,4039)/(160,4039) |
|1101/Village00 |(13,11)/(1,11) |(520,479)/(40,479) |
|278/ExVillage01 |(3,3)/(3,9) |(120,159)/(120,399) |

Như vậy100 có nguồn từ bảng map **trước actor init**, không tự chứng minh garbage/reset hỏng.
Snapshot local-playmap CP019 có fields15/101 cho Antarctic và15/13 cho hai map mới;
nhãn width/height và bounds toàn consumer còn giới hạn, không dùng4039 riêng để kết luận spawn sai.
VN §5a-7 chỉ gợi ý đường index→point→actor; index/name/layout KR được đo riêng.

**Profile map đồng bộ:** đổi238row/name/localcode và Adler32 inner→241,117ack,
121name/code và196code ở cả hai vòng; không chỉ đổi chữ UI. Village00/ExVillage01 có ảnh geometry,
spawn riêng,118/159,HELP,175 và180→UI phòng;121 lần lượt127/129byte,errors0.
Village00 có object giữa map giống UFO/boss: chưa chốt semantics Battle thường từ tên/ảnh.
Hai map mới đều báo19 khoảng6.46–6.49s sau kind0 ở hai vòng; Antarctic vòng hai vẫn khoảng0.9s.
Đây là số đo theo context, chưa chứng minh nhân quả hoặc mọi timer/reset đã đúng.
K5/K6, collision/item/chain/authority và peer vẫn mở; bước tiếp ở [RESEARCH](../RESEARCH.md#q-023).

<a id="bomb-serial"></a>
### Nhiều bom, serial và ID — CP025

**kr_static/kr_runtime confirmed trong phạm vi copy offline**, [evidence](../evidence/bomb-serial/README.md).
Bốn run108s tuần tự trên cùng copy SHAe9bb47…; không patch thêm. Request118 thực có serial0,28,56,
không phải giả định số tăng từng bom. Chưa giải cách sinh/rollover serial hoặc mọi optional flag.

Với118/base22/flag0, widths13field vẫn `[4,1,1,4,1,1,1,1,1,1,1,4,1]`.
159/base39 có widths17field `[1,1,2,4,4,4,1,1,1,4,1,2,2,1,8,1,1]` (index từ0):

| Nguồn thực118 | Đích159 | Giới hạn |
|---|---|---|
|field2/u8 owner |field1/u8 |owner0 trong các run này |
|field3/u32, record+0C |**field3/u32**, payload+4 |serial ghép local pool, đo0/28/56; không ép u16 |
|field4/5/u8 |field6/7/u8, payload+16/+17 |hàng/cột đã đo; không gộp field6 của118 thành power |
|**field10/u8, record+18** |**field8/u8**, payload+18 |Correction CP031 bằng Fire1→2; [bằng chứng](items-stats.md#item-effects). CP025 dùng field6=1 nên chưa phân biệt hai field |
|ID do stub cấp |field11/u16, payload+24 |chọn1/2/3; allocator/round generation/collision còn design |
|không suy từ118 serial |field2/u16, payload+2 |aux unknown, giữ0 trong ca sửa; không gọi field này serial |

`E5E989→65A320` đưa serial u32 vào info+00; `E5EAD1` đưa ID vào info+0C.
`E14600` đưa aux u16 vào info+B0, gọi `CGOAIBattleBomber/vf50→AEF700`.
Hai nhánh pool so `B37FC0(this+118→B3BB20)` với getter info+00 tại **AEF7AB/AEF814**;
run có probe mới đi nhánh AEF7AB, không phải AEF814. Match dẫn tới`B37880(ID)`;
aux+B0 đi`B38350` lưu secure+10C. Getter`B38370` đọc aux, không được dùng nó thay serial.

Control đặt serial28 vào aux u16 nhưng serial u32=0: callback vẫn39/39/errors0,
**E5ECA8 trả0**, không bind bom2; ảnh bom local prediction không chứng minh159 đã nhận/ghép đúng.
Ca sửa chỉ hai field này có ba118→159 trả entity, bind ID1..3 và ba158/kind0→cleanup.
Cùng pointer được tái dùng, ID/serial và nhánh thực thi mới là bằng chứng phân biệt vòng đời.
Fuse4s, ID, field159 còn lại và158args2/−1/FF là **emu_choice**, không phải luật server gốc.
Các run CP025 bỏ policy tự gửi175/180 để tập trung cùng vòng; không dùng chúng chứng minh kết quả trận.

**Control lặp/cũ ở tầng semantic**, mã hóa lại cùng payload bằng seq S→C mới liên tục (+3):

- 159 lặp sau0.5s: ba bản gốc trả entity/bind1..3; ba bản lặp trả0, đọc39/39/errors0.
- 158 lặp sau cleanup0.5s: lookup vẫn thấy ID, nhưng không vào E9E660/kind0 lần hai;
  tổng ba enter/cleanup cho sáu callback13/13/errors0. Không nói lookup0 hoặc crash-free là đủ idempotency.
- 159/158 của bom cũ gửi sau khi ID kế đã bind: hai159 cũ trả0;
  hai158 cũ đọc đủ nhưng không tới E1449C/E9E660; ba158 gốc vẫn enter/cleanup đúng ID1..3.
  Chỉ xác nhận đường đã đo; không chứng minh mọi race, stale round, ID reuse/wrap hoặc chain/item an toàn.

<a id="bomb-cycle"></a>
### Bom nổ, HELP và trở về phòng — CP022

**kr_static/kr_runtime confirmed theo copy offline**; fixture/time/policy là **emu_choice**.
[Evidence](../evidence/bomb-calc/README.md) khóa năm run tuần tự, code/native/ảnh/hash và correction;
copy stage-social SHAe9bb47… không thêm patch. Sáu vùng code/table so byte với static copy660447….
Đây bổ sung CP021: kết luận “chưa thấy nổ/về phòng” ở các run trước là lịch sử, không phải trạng thái hiện tại.

**158/kind1 và calc:** `calc-guard` thấyE9C520 ở stack đặt trước159 (`AF10E5→AEF528`) và
sau158/kind1 (`E5F5CF`). Kind getter1, các getter tọa độ100/1/power1, cell khác0 và qua spread;
không phải guard đầu loại request. Không gọi hàm tính này là detonation; vị trí100 không tự xác nhận tile hợp lệ.

**158/kind0:** u8 count1,u16 ID1,u8 kind0, **2u32,u8**; mẫu2/FFFFFFFF/FF,
seq88/payload13 đọc13/13/errors0. `E14360→E9E660` nhận đúng pointer bom đã được159 cấp ID1.
Nhánh này xây tập sự kiện rồi gọi cleanup virtual; burst runtime có nước ở ô bom, `HELP!`/local trong
bong bóng, bom biến mất và vật cản lân cận thay đổi. Run stay có local biến mất/roster đổi sau đó;
đây là quan sát hình ảnh, chưa full contract death/outcome/peer hoặc toàn authority phá block.
Các input thêm/delay2.5s là test choice, chưa fuse gốc hay đầy đủ nhánh kind/type.

Lịch sử CP022 chỉ đọc prefix10byte của127 sau blast; writer/tail lúc đó chưa phục hồi.
CP023 đã mở hết hai envelope và nối writer trap/death tại [contract dưới](life-results.md#round-result).
Không chuyển enum VN11/13 hoặc tự gán vị trí4039 thành spawn hợp lệ; các127/167 khác vẫn cần kiểm riêng.
Report142 được nối tiếp trong [CP032](items-stats.md#hidden-items), không dùng giới hạn CP022 làm current unknown.

**180/branch thường:** dispatcher`E5D62B`, CGameStagevt176A470+2D8→`E64950`.
`1220550` còn có branch khác quaE72490, chưa kiểm. Mẫu branch thường đọc **bool (u8), u8, u8**:
virtual38→`14EB1F0` thực gọi getter3C và tiêu thụ một byte; rồi trường thứ hai (CP022 gọi status, [CP162](multiplayer-team.md#current-team-mode) nối selector) và listflag.
Chọn0/0/0, seq91/payload3: `result180-full` đọc3/3/error54=0/error58=0,
`E652D8→12562F0(12)` và UI phòng có KRLocal/KRGuest/nút Bắt đầu vẽ lại.
Listflag0 bỏ list variant; ở CP022, reset flags/position đi qua handler nhưng chưa kiểm start trận kế.

Mẫu đầu2byte có framing và hai getter pass nhưng callback báo54=1, vẫn gọi stage12;
giữ làm short-body control, không gọi full valid result. Việc trở về phòng không tự chứng minh parse đúng.
Trong run CP022, stub phát180 sau158 đúng15s theo test policy. CP023 bên dưới thay trigger bằng
reported-event19 và thêm bảng kết quả175. Serial/dynamic echo, authority đầy đủ, hai client,
full auth/cache/vanilla và gate K2–K5 vẫn mở. Bước tiếp duy nhất ở [RESEARCH](../RESEARCH.md).

<a id="mixed-owner-chain"></a>
### Hai owner: control khoảng cách và blocker liền kề — CP064

**kr_runtime/confirmed hẹp cho control**, liền kề khác owner vẫn **unknown**;
[recipe/raw/hash/blocker](../evidence/mixed-owner-chain/README.md). Tái dùng CP033 EBE180/B32390/EB80E3/
ID mapping, currentprofile/guard127 và VN phương pháp nối ID/tầm lúc đặt; không thêm thân hàm native/
patch hoặc đổi productionservice. Hai timer6s gửi158 riêng ID1/2 là policy emulator, không thời hạn gốc.

Ba run `spaced-live`, `adjacent-live`, `adjacent-short` đều có actual118 A0 tại(3,3), B1 tại(3,5),
power1, serial0 riêng từng owner, service cấpID1/2. Giữ Right0.8/0.4/0.2s đều ra ô5 trong fullprobe;
**tên run và thời gian giữ phím không chứng minh liền kề**. Native159/getter kiểm owner/ID/cell.
Ở cảhai view, root158/ID1 chỉ state0/processID1; ID2 chỉ state0/process trong callback timer riêng.
Mỗi158 đọc13/13/errors0. Đây là control hai bom khác owner ngoài tầm1, không chain proof.

Pointer/ID remote nối bằng native159 BEGIN→ENTITY→END và exactwire/order/nativefields; local
IDsetter chỉ dùng khi đã hit, không giả nó chạy cho mọi entity remote. Nhãn row/col EB80E3 kế thừa
quy ước getter cũ và đảo so actual118; tọa độ chuẩn lấy actual118. Không sửa raw để đổi nhãn.

Run `adjacent-slim` giữ input0.2s và giảm read-only trace chỉ159/158/state/process, nhưng B báo
`nmcogame.dll SetLocaleAndRegion Error(2000:0)`, channel đóng saurequest6, chưa Battle/bom/replay.
Không quy lỗi cho slimprobe hoặc timing khi chưa có control nhân quả. Giả thuyết fullprobe làm
nhiễu keyup chỉ candidate; không hiệu chỉnh px/s/clock từ wallhold. Preparation đầu thiếuGDBfile
cũng được giữ riêng là lỗi trước Wine, không clientruntimefailure.

Ba ca chỉ một lượt bom, không hai trận/reset/fullmatch/K6a/LAN/authority gốc. CP033 chain một owner
vẫn nguồn hợp lệ; mixed-owner liền kề/ba bom/vật cản/list3 còn mở. Tránh lặp vô hạn input trên overlap
profile: CP065 đã [tích hợp distinct spawn](load-phase.md#distinct-spawn). Chỉ mở lại mixed-owner chain khi bố trí
native đủ điều kiện hoặc probe ổn định hơn; thứ tự hiện hành chỉ ở [PLAN](../PLAN.md#current-order).

<a id="mixed-owner-power"></a>
### Mixed-owner/power2: control ngoài tầm và blocker UI — CP071

[Raw/recipe/hash](../evidence/mixed-owner-power/README.md). **kr_runtime/confirmed hẹp**: currentservice/
CP065distinctspawn, floorFire17row6col3 nhận actual124/165 và Fire+4BC1→2 cảhai view. Run đầu actual
A118(3,3)/power2 và B118(6,5)/power1, owner0/1, IDs1/2; native root158 chỉ xử lýID1, ID2 tới timer riêng.
Đây là control ngoài tầm dự định, không chứng minh chain/radius2 hoặc phủ nhận chain khác owner.

CP070 next-step kỳ vọng Up1 đưa B tới hàng3 đã bị actualhàng6 sửa; không suy hold→cell hoặc pickup
row6 là điểm dừng. Retry read-only cached1256B3B/manualchecksum/native52A350 feedback cho normalUI
Up đưa B tới(120,159), cùng A trước Space, nhưng không request118 trong4s. Controller dừng bounded
failure; không native159/158/bom trong retry, không quy cause cho debugger/client/input. Hash/raw/lỗi
được giữ; không thử lại recipe thiếu điều kiện mới. Không thêm body/patch/service hoặc force state.

**Unknown:** hai owner trong tầm/adjacent1cell/ba bom/vật cản/reset/fullvalidator/authority/K5/K6a/
fullmatch/LANInternet/vanilla. [CP033](#bomb-chain) sameowner và [CP064](#mixed-owner-chain) control
power1 vẫn hợp lệ; historicalcontrols không cùngrun power-only intervention. Bước tiếp ở PLAN là
EOF sauStart trướcclock, không nâng gate từ Fire/feedback/encoder/probe pass.

<a id="mixed-input"></a>
### Hai owner trong tầm và root callback chain — CP081

[Reuse/raw/control/hash](../evidence/mixed-input/README.md), **kr_runtime/confirmed hẹp** trên current
modified Wine/profile CP071, production không đổi. CP027/064/071/VN cung cấp pipeline/checklist,
không numerical port. Source-first sửa bước tiếp CP080: CP071 live đã có B118; feedback thiếu Aroot
sau đưa B lên, trước BSpace. Giữ failure/causeunknown; không gọi B không đặt được bom tổng quát.

Run mới cùng normal input/Space0,15s/Fire profile, lean known read-only probes và durable action
facts: A124 Fire17/6,3→165, nativeA+4BC1→2 cảhai view; Bposition nativeY≤159. Actual118 Aowner0
(3,3)/power2/ID1, Bowner1(3,5)/power1/ID2. Native key down/up→writerE2DF80/callerE2D608/enqueue118
cảhai; không thêm retry/hold, patch/stateforcing/reply hoặc RE lại body. Không isolate nguyên nhân
historicalmissingAroot vì instrumentation/recording khác, không suy debugger/focus/guard đã sửa.

Native159 cursor39/errors0/pointer join theo actual wire owner/ID ở từng view. Trong root158 ID1,
cảhai view processID1 vàID2, secondary state0 qua callerEBE21A; chỉ rootkind0. Callback158 ID2 riêng
sau đó lookupID2 nhưng khôngkind0/process lại, cursor13/errors0. Đây là mixed-owner chain trong tầm2
trên geometry đã đo, không chỉ hai server timer. Reuse CP064 samegeometry/power1 root-only là control
lịch sử với khác biệt profile/timing, không phải same-run can thiệp chỉ power.

Chưa adjacent1cell/ba bom/vật cản/toànchain/authority/reset/two-owner hai chu kỳ/fullmatch/K5/K6a/
vanilla/LANInternet. Models framer/heap/getter chỉ corroborate bytes; policy fuse/drop/rank là design.
Run tại CP081 dừng sau hai bomdeadline+4s, không result/return. Bước ghép route hai chu kỳ đã
được [CP082](#mixed-cycles) đo; việc hiện tại chỉ ở [PLAN](../PLAN.md#current-order).


<a id="mixed-cycles"></a>
### Hai owner/chain qua hai kết quả và reset — CP082

[Recipe/raw/correction/hash](../evidence/mixed-cycles/README.md), **kr_runtime/confirmed hẹp** trên
modified Wine copy/current local service. Reuse CP080 rejoin/hoverpreparedReadyStart và CP081
Fire/feedback/actualpair/chain/ID joins; VN chỉ lifecycle/checklist. Profile measurement ghép metadata
membership CP077/080 và rawFirefloor CP071/081, không hiddenitems; rawschema/servicepolicy giữ.

Final aligned run: guestBack/rejoin rồi hai native121/clock/175/180/refresh cảhai view; cùng8TCP/
IDs1,2/slots0,1, key12345→12346. Mỗi vòng actualA124Fire17/6,3→165, native+4BC1→2 và baseline1
vòng2; actual118 A(3,3)power2/B(3,5)power1, globalIDs1,2 rồi3,4. Native159 cursor39/owner/ID/
power/pointer joins; root158 mỗi vòng processroot+secondary/state0 callerEBE21A, ownlater158 của
secondary khôngkind0/process lại. Mỗi158 cursor13/errors0, Astate1 trước175 ởcảhai view.
Pointer join yêu cầu cùng round/native159entity trước event, không lấy địa chỉpool là ID suốt lifetime.

**Controls và correction giữ riêng:** run đầu có chain nhưng cảhai17/19/no21, policy fixture đợi
other21 sau death nên chưa rank trong boundedwindow; Codex dừngSIGINT130/originalfinally, plannedfail.
Run escape sau có hai resultreset nhưng Aactual(4,3)/B(3,5) diagonal, không chain; strictverifierfail
giữ, sửa kết luận sơ bộ dựa intendedroute. Final thêm fresh nativeA feedbackY≤159 cho rootrow,
ngoài BUpY≤159/BDownY≥239. Tất cả là normalOSkeys/release theo read-only samples/known52A350,
không arbitraryholdincrease/forcing/nativefunctioncall/new106. Targetcoordinates không thay actual
request/state/callback; final chỉ confirmed sau kiểm. Historical CP071missingArootcause vẫnunknown.

Ca một survivor ở final có actualA17/19/peerreport21, nativeAstate1 trước175→180/reset theo policy
hiện tại. Không coi policy đó là luật server gốc hoặc có tie support. Bước cảhai chết tại CP082 đã
được [CP083](life-results.md#all-dead-result) đo theo policy có tên/giới hạn; việc hiện tại chỉ ở [PLAN](../PLAN.md#current-order). Chưa fullmatch/K5/K6a/
vanilla/LANInternet/fullauth/codec/validator/authority/toànreset/generalmembership/ba bom/vật cản/
allchain. Models nativeframer/getter/heap chỉ corroborate bytes/fields, khác nativecallbacks/runtime.


<a id="bomb-chain"></a>
### Nổ dây chuyền hai bom — CP033

**kr_runtime/kr_static confirmed hẹp trên diagnostic copy Wine**, không patch thêm;
[controls/recipe/native/hash](../evidence/bomb-chain/README.md). Dùng profile ExVillage01/278,
layout582 ba list rỗng để không có hiệu ứng item, mapping power118.field10→159.field8 đã sửa.

**Can thiệp:** B có capacity2/power1 trong profile, đặt hai bom actual118 owner1/serial1,29.
Bom đầu tại hàng9/cột3; control bom sau hàng7/cột3, ca liền kề hàng8/cột3.
Stub cấp ID1/2 tới cả A/B nhưng chỉ phát một158/kind0/ID1 sau fuse thử6s, không timer riêng cho ID2.
Control cách hai ô: chỉ bom đầu bị xử lý, bom2 vẫn hiện ở cả hai ảnh cuối.
Ca liền kề: cả hai bom được xử lý trong cùng158, ảnh hai phía không còn bom tại hai ô đó.

**Cạnh runtime:** `E145E2→E9E660` chỉ gọi một lần cho root ID1. Trong `E9ECB0`, root được
chọn và đưa vào output, candidate bom2 có selected0 ở nhánh đầu; không dùng riêng flag này để
phủ nhận chain. Nhánh mở rộng thực ghi trạng thái bom2 qua `B32390(0)` từ return `EBE21A`
trong helper `EBE180`, rồi nối bom đó vào danh sách. `B32390` ghi secure field+1C và reset+70/+7C;
không gán hàm này chỉ là visibility hoặc đồng nhất với toàn lifecycle removal.
`EB8000` tới cạnh `EB80E3` cho cả hai pointer bom trong ca liền kề, chỉ root trong control.
Pointer được ghép với hai entity do159 trả, không suy từ cùng hình nước nổ hoặc số entry kind0.
Các nhãn row/col trong raw probe từ getter B37D10/B37D30 đảo so với actual118; report chuẩn hóa theo
actual118: hàng9/8 hoặc9/7, cột3. Không dùng nhãn printf làm nguồn quy ước tọa độ.

**Replay sau chain:** gửi lại158/ID1 sau khoảng1s và158/ID2 sau2s, seq receiver mới liên tục.
Mỗi client đọc ba callback13/13/errors0; lookup vẫn có pointer ID1/2 nhưng không gọi lại E9E660,
không xử lý blast thêm hoặc lặp cạnh ghi trạng thái đã probe. Đây là kiểm hẹp sau chain trong cùng
round, không chứng minh stale generation/ID reuse/wrap, ba bom hoặc nhiều owner.

Creator/timer/broadcast ID1 là policy emulator; client tự mở rộng chain trong ca này, không cần
server phát riêng ID2. Không phục hồi thuật toán authority/validator server gốc hoặc loại trừ ảnh hưởng
UDP riêng. Chain qua tường/block, nhiều owner/ba bom/race, collision/rescue/reset/disconnect và
full auth/cache/codec còn mở; **chưa full match/K5/K6a/LAN/Internet**.

<a id="three-bomb-chain"></a>
### Ba bom A/B/A trong root callback qua hai vòng — CP084

[Recipe/nguồn/hash/audit](../evidence/three-bomb-chain/README.md), **kr_runtime/confirmed hẹp** trên
currentmodifiedWine/local. Reuse CP033/081–083 callback/state/ID/input/result và CP042/069 Bomb/
Firefloor/native582/securestats; VN/map/legacyserver chỉ checklist, không port opcode/offset/FIFO.
Không thêm nativebody/binarypatch/productioncode. Lựa chọn vật cản chưa đủ actualmap/control:
list2 của582 chỉ gắn item vào FixedBox hiện có, không phải lệnh tạo tường. Chọn ca ba bom tối thiểu.

Measurementprofile ghép hai knownfloor records16/4,3/kind0 và17/6,3/kind1; nativeframer/blob/
11ED590 đọc26inner/counts2,0,0 với CRC/partialcontrols. Emulation chưa proof liveinstall/effect.
Actual124 A nhặt cảhai mỗi vòng;165nativecapacity+4A4 vàFire+4BC lần lượt1→2/1→2 ở cảhai view,
clockvòng2 baseline1/1. BRight trướcUp tránh đi lại qua rootrow; thirdA Down/release trên fresh
nativeY≥179 rồiSpace; target/hold không thay actualrequestgeometry.

Cùng run sau guestBack/rejoin: keys12345→12346, IDs1,2,3 rồi4,5,6, owners0,1,0 mỗi vòng.
Actual118 Aroot(3,3)/power2, B(3,5)/power1; A thứhai **(5,3) vòng1, (4,3) vòng2**, power2.
Không gọi hai hình học giống hệt hoặc suy tọa độ ô từ một hostsample. Mỗi view native159 đủ39/
errors0 cho cảba trước root158. Trong rootcallback ID1 hoặc4: kind0 chỉroot, process cảba; hai
secondary state0 callerEBE21A. Hai ownlater158 đọc13/errors0, lookupđúngID nhưng khôngkind0/
process lặp. Đây là fork từ root theo hai hướng, chưa chuỗi sâu kiểu root→secondary→third ngoài
rootrange hoặc mixedowneradjacent/vật cản/allchain.

Cảhai actual17/19/state1 trước175, all-dead policyCP083/latestdeath8s, native175full302/180full3,
manager trước180 rank2/2, UI haiLOSE/cảhai view/hai vòng và finalroomrefresh. Originalrun normalexit0/
no controller/service/probe errors. Scopephân biệt raw/nativecallbacks với model/getter/oracles;
fixture/drop/fuse/outcome vẫn design, không fullmatch/K5/K6a/vanilla/LANInternet/fullauth/codec/
validator/authority/toànreset. [PLAN](../PLAN.md#current-order) tiếp gap guardTCP + hairesultreset.


<a id="recursive-chain"></a>
### Chain sâu ngoài tia bom gốc và so candidate terrain — CP122

**kr_runtime/confirmed hẹp**, [nguồn/recipe/hash](../evidence/three-bomb-chain/README.md#cp122).
Tái dùng CP033/084 native ID/setter/process/typed packets, CP094–096 terrain195flags và known
Bomb/Firefloor. VN chỉ hỗ trợ khái niệm/item checklist; KR opcode/field/power/cell đã đo riêng.
CP084 vẫn là fork lịch sử. Không nativebody mới, không production/binary/pack/DB/fuse change.

Run179,891290309s/exit0/error0, một epoch key12345, hai copy Wine đối địch/distinct spawn. Actual118
ownersA/B/A, IDs1/2/3, cells/power **(3,3,p2)/(3,5,p1)/(4,5,p2)**; native159 ba entity đủ39 trước
root158 trên từng view. Third khác cả hàng và cột root, ngoài tia root, nhưng kề Bsecondary.
Trong callback root158/ID1: chỉ một kind0 root, native process **1→2→3**, setter secondary2/3
state0/callerEBE21A. Later own158/ID2/3 lookup đúng ID, đủ13/lỗi0, không kind0/setter/process lặp.
Không dùng riêng số lần kind0 hoặc model tự sinh để suy chain. Rawprintf row/col cũ đảo; cell chuẩn
lấy actual118, pointer ghép native159 cùng epoch/trước callback.

Cả hai view có195flags clock/before/after từng158; root chỉ clear **(2,3),(2,5),(3,2),(3,6),(4,6)**,
khớp toàn195 phần tử với `TerrainShadow` trên pinned profile; later own callbacks không đổi flags.
Model process/release/terrain là `emu_choice/design`; native agreement này chỉ mở rộng scope đã đo,
không chứng minh FIFO/immutable/first-bomb-stop/sharedblocker/moving/universal map hoặc server gốc.
Normal A Down(targetY169) rồi Right(targetX190), actual118 là oracle cell; không teleport/forged118.

929 joins gồm774raw SHA,438 actor getter/150 packet/6 peer153 cases;14 flag snapshots có2758
native getter và2730 array-address checks là copied-native emulation riêng. Run dừng sau owncallback
bound, **không claim result/return/vòng2**, fullmatch/K5/K6a/LAN/Internet. Bước kế chỉ ở PLAN.


<a id="recursive-chain-cycles"></a>
### Chain sâu cùng survivor/result/reset hai epoch — CP123

**kr_runtime/confirmed hẹp**, [raw/recipe/control/hash](../evidence/three-bomb-chain/README.md#cp123).
Correction nguồn: **CP106 đã có currentterrain survivorA/B19/A21/native ranks1,2/result/reset2**;
CP122 PLAN/RESEARCH mô tả gap survivor hiện hành quá rộng. Phần mới ở đây chỉ deepthird ngoài tia
root được ghép cùng lifecycle hai epoch, không claim firstsurvivor hay sửa CP106/122 complete.

Run đầu300,247027271s có hai result/return nhưng R1third(4,5), **R2third(5,5)**: root4 chỉ process4,5,
bom6 nổ riêng; strict deepgeometry vòng2 fail giữ nguyên. Fresh Down sample189 trướckeyup nhưng lúc
Right có214→239; không gọi sample/target/hold là actualcell hoặc causalproof cho mọi input.
Retry300,138278672s/exit0/error0 chỉ thay firstthirdDown bằng normal pulse0,10s đã biết CP094;
Rightfeedback/laterADownescape299/profile/currentproduction/fuse6 không đổi. Actual118 hai vòng đều
**(3,3,p2)/(3,5,p1)/(4,5,p2)** ownersA/B/A, IDs1..3 rồi4..6, keys12345/12346. Mỗi view mỗi epoch
ba native159 đủ39 trướcroot158; process1→2→3 hoặc4→5→6 trongrootcallback, secondarycallerEBE21A,
laterown2/3/5/6 đọc13/lỗi0/no-op. All195flags/callback/clockseed reset khớp candidate cùng run.

ActualB17/19 và actualA21 sau death/latestown158, drained rank chọnA1/B2; native175 đủ302 và ở180
manager hai view ranks1/2 cả hai vòng,180 đủ3/roomrefresh/ảnh finalroom. **Start121 đủ129/129**
với hai realactors, không155 của synthetic3roster CP121. Verifier đầu dùng nhầm155 fail; bản sửa
riêng chỉ scope129, giữ script/logfail/raw/geometry/result assertions; không hạ tiêu chí native đọc đủ.

1453joins/1238raw,882 actor getter/348packet/12peer153;28flag snapshots có5516getter/5460arraychecks
là copied-native emulation riêng. Scope modifiedWine/currentpinnedmap/power1–2/normalinput/chosendrain
outcome; không chứng minh universalchain/wholemovement/auth/lifeauthority/fullmatch/K5/K6a/LAN/Internet.
AUD038 bổ sung sửa nguồn/gap ngay, không reset cadenceCP124. Bước tiếp chỉ ở PLAN.


<a id="capacity-fresh-serial"></a>
### Native third-Space control và serial-only divergence — CP086

[Recipe/nguồn/raw/hash](../evidence/capacity-admission/README.md), **kr_runtime/confirmed hẹp** trên
currentmodifiedWine/local. Reuse CP025serial/pool, CP031/042 Bombcapacity và CP084–085 normalroute/
knownfloor/nativefields/chain/results; VNoc_e-game có hai bom sauBomb, VNemulator111 handler chỉ
allocate/broadcast/timer, không tự mang capvalidator. Không tìm lại itemconsumer hoặc copy VNoffset.

Hai control `native-cap`/`capacity-watch`: actualBomb/Fire124165→Acurrent2/power2 (native securegetter);
clockbaseA1/max10, B2/max7, không giả B1. ActualA/B/A118 serial0/1/28, ba159 trướcfirst158, ownerA
có hai bom. ADown/release theo freshY≥239, nhưng actualSpace snapshot cóYcao hơn target; không
suy hold→cell. NativeSpace down/up được ghi với capacity2, vị trí khác trước và trướcfirstblast,
khôngwriterE2DF80/118 thứba củaA trong boundedwindow. Đây **không là serverreject hoặc causalcapguard**.
Sau nổ Bactual17/19, Aactual21, nativeAstate0/Bstate1 trước175/180/room: một survivorcycle mỗi run,
khác parentall-dead; không cố giữ policyoutcome cũ. Watch trên wrapperactor+4A4+8 chỉ arm thirdnative
keydown→keyup/max12reads, disarm ngay: **0reads**. Cached529C60→52A350 có đọc+8; không chứng minh
nativekhôngdùng cachedcapacity/pool/earlierguard, nguyên nhân chưa biết, không lặp unchangedwatch.

`fresh-serial`: sau cùng control, trướcfirstblast, localTCPproxy lấy **actualA118 serial28**, chỉ sửa
field3/u32→56 (giá trị serial đã quan sát CP025); khóa12345 và mọi field khác/cell giữ nguyên. Outer
seq upstream được mã hóa lại, S2C không đổi. Đây là **modified captured body**, không nativeplacement,
không gói cũ qua cryptoguard hoặc input trên ô mới; A đã ở vị trí khác nên không isolate cap vsdistance.
Service chỉ kiểm key/serial và cấpID4/deadlinejob/159 cho cảhai. Field generic `source_bomb.actual118`
trong logservice không làm mutation thành nativewriter; front/upstream/originSHA/phân loại proxy pin riêng.

Hai nativeviews đọc đủ15939/15813/errors0. ID4 `GEN159_ENTITY` A trả**0**, B trảnonzero; root158ID1
processIDs**1,2,3** ởA nhưng**1,2,3,4** ởB. Later own158 khôngprocesslặp; A không cóID4lookupreturn,
B cólookup. Không dùng timer/serviceaccept hoặc ảnh bom chồng để kết luận entity thànhcông/cause.
Đây là **divergence có trigger concrete**, chưa nativepoollimit/receiverguard giải đầyđủ. Ba run đều
có originalfinalrecords/khôngcontroller-serviceprobeerrors, nhưng roundreturn không xóa finding lệch.

**Tình trạng implementation:** CP086 chỉ đo, chưa thêm guard cap/cell/distance/freshserial. Cácpositive
CP084/085 giữ đúng normal/selectedreplay scope, không bị nâng thành fullvalidator. Tại CP086 bước tiếp là profile-scoped reservationguard trướcID/job (CP087 đã triển khai); source stat/max và acceptedBomb effect đã có, release
secondary sớm trong nativechain vẫnunknown phía service, phải ghi design/conservativewindow khi chọn.
Chưa fullmatch/K5/K6a/fullauth/fullcodec/fullvalidator/authority/vanilla/LANInternet/allreset.


<a id="capacity-reservations"></a>
### Optional profile capacity/reservation guard — CP087

[Recipe/nguồn/model/runtime](../evidence/capacity-reservations/README.md). **Implementation là design**,
compatibility **kr_runtime/confirmed hẹp** trên currentmodifiedWine/A-B/profile. Reuse CP086 exact
serial-onlyproxy/currentbody/negativeA0-Bentity và CP031/042/086 securecapacity/max/acceptedBomb;
VNsession111alloc khôngcapguard chỉ là reference tránh port thiếuvalidator. Không newnativebody/patch.

Optional metadata `bomb_capacity` là map role→`{initial,maximum}`; profile đo khai A1/max10, B2/max7.
A/B bắt buộc nếu khai, C chỉ khi có source/profile riêng; số nguyên1≤initial≤maximum≤255 là bounds
cấu hình, không law max255. Absentfield giữ behavior cũ **không bảo vệ cap**; không tựmigrate/profile
raw. Unsupportedconfiguredroster bị từ chối Start trướckey/generationadvance, không gán statC từA.
AcceptedStart reset trackedcapacity về initial; currentcatalog Bomb/kind0 đã claim+consume và phát165
mới increment+1/clamp, duplicate/stalekey/noncatalog khôngincrement. Không lấy stat từ requestpower
hoặc suy effect từ tên/icon; tracking là policy server, native165effect kiểm riêng.

Sau existingkey/serialdedupe, **trước ID/fence/job/159**, đếm jobs owner hiện hành/cùnggeneration/
fencevalid chưaretire. Count≥trackedcapacity → `bomb_capacity_reserved`, logcapacity/reservedIDs,
không reserve serial/ID/job/packet. Jobretire/finish/Startfence loại reservations theo lifetime đã có;
không tin clientreport để release. Ownerserialize vẫngiữ, profilekhôngkhai báo không đổi duplicate/
oldkeyreason. Đây là reservationbudget, chưa recoverednativepool hoặc toàncapacity/distance/auth.

Runtime `guarded` mộtvòng lặp sameCP086variant: serial56 bị loại vớiAcurrent2/reserved1,3; native159/
root158 cảhai có3entity/IDs, khôngID4, result175/180/room. `guarded-cycles` đo thêm **state reset mới**:
A tracked/nativecapacity1→2/B2 ởmỗi vòng, Fire1→2 vàbaseline1; keys12345/12346, actualA/B/AIDs1..3/
4..6. Proxy mỗi vòng giữ capturedfields/currentkey/cell, chỉ chọn field3+28 (design displacement,
không nativegeneratorlaw). Hai bản bị loại trướcID/job/159; reservations1,3 rồi4,6, khôngstaleR1/count
leak. Hai view đủ6nonzero159full39/6full15813; rootprocess3IDs/mỗi vòng, ownlater khôngduplicate;
121/clock/175full302/180full3/roomrefresh hai lần. Native field/getter, emittedbody/seq/model và policy
counter tách rõ; không suy noerrors thành fullmatch.

**Giới hạn còn hoạt động:** nativechain đã state0/processsecondary trước ownserverdeadline; điều đó
chưa chứng minh toàn bộ pool đã trả slot. Guard giữ reservation tới jobretire; CP088 đã đo một normal native replacement bị từ chối trong cửa sổ này. Tại CP087 chưa đo/sửa
release sớm, chưa đóng capacity/chainauthority hoặc mọi newserial/cell/distance/auth/character/mode.
98tests kiểm incrementonce/clamp/currentkey/serial/owner/jobretire/reset/fence/unsupportedroster/config;
resetactual hai vòng không là mọiEOF/errorvariant. Profile mới chưa có API tựtrích statbase từnative.
Phép đo cửa sổ đã có ở CP088 bên dưới; [PLAN](../PLAN.md#current-order) giữ việc hiện hành. AUD026 qua CP088, audit kế CP092.


<a id="early-chain-reservation-window"></a>
### Normal replacement sau early chain bị quota giữ chỗ — CP088

[Recipe/hash và nguồn tái dùng](../evidence/early-chain-window/README.md).
**kr_runtime/confirmed hẹp** trên current modified Wine copy/profile A-B. Giữ production CP087,
capacity A2/B2, fuse6s; chỉ trì hoãn input thirdA2s và dùng normal Space/Right/Space sau root callback
đã hoàn tất cả hai view. Không proxy mutation, ép state hoặc gọi hàm native.

Actual A/B/A118 tạo ID1/2/3; root158 ID1 process cả ba trong mỗi view, secondary3 state0 caller
`EBE21A`, ownlater158 không process lặp. Sau root observation, A phát118 serial56 ô(9,3), được
ID4/159 nonzero ở cả hai view, quota reserved[3]/capacity2. A phát118 serial84 ô(9,5), bị
`bomb_capacity_reserved` trước ID5 với reserved[3,4]/capacity2. Request này đến **1,91992808s**
trước own158/jobretire của ID3. Native writer/frontframe/request timestamps nối riêng, không dùng
input completion thay thời điểm phát; request thứ hai không phải mutant CP086.

Điều đã xác nhận: guard design CP087 chặn normal native request trong cửa sổ chain sớm; native
emission chứng minh đường input/request chạy được ở context này. **Không** tự chứng minh toàn pool
đã trả slot, original server phải accept mọi request đó hoặc native allocation của ID5 (chưa có159).
ID4 được process riêng; hai view đủ4nonzero159/full39 và4full158/13/errors0, một survivor result/
return/room cycle, không controller/service/probe errors. Không là hai vòng/fullmatch hoặc gate.

Tại CP088 chưa sửa code/release policy; CP089 bên dưới bổ sung policy giới hạn. Không release theo report hoặc khoảng cách Manhattan đơn thuần:
geometry/map blocker và chuỗi secondary phải có nguồn/giới hạn trước khi chọn design.
[PLAN](../PLAN.md#current-order) giữ thứ tự; [gap](../RESEARCH.md#gap-bomb-lifetime) giữ phép kiểm.
AUD026 sửa wording CP087 “đã giải phóng secondary” thành state0/processsecondary, chưa toàn pool;
raw/manifestCP087 giữ bất biến. Review waived/tự kiểm, chưa independent approval.


<a id="chain-capacity-release"></a>
### Release reservation trên đường clear trực tiếp của fixture — CP089

[Recipe/raw/hash/control/partial](../evidence/chain-capacity-release/README.md).
**Implementation: design**, không recovered server/pool law. **Compatibility: kr_runtime/confirmed hẹp**
trên modified Wine/current A-B profile. Tái dùng CP033/081/084/088 root-chain/159/118 và VN pool
reference; old KR static R139–141 có blocker theo class/hướng, không cho dùng khoảng cách đơn thuần.
Đọc bổ sung E9F220 (step/bounds/class/direction) và EB9710 (cell/ray state); bytes match static/runtime
copy và modO, không patch/call native. Body/cache/provenance ở evidence, không native whole-map oracle.

Optional `capacity_chain` yêu cầu `bomb_capacity`, pins `fixtures.map/start` SHA256 với actual profile
frames, `clear_cells` (cặp row/column duy nhất) và `maximum_power`. Config bounds0..255 cho cell,
1..255 cho power là design bounds, không native map size/max law. Absent metadata giữ CP087 behavior.
Profile đo chỉ khai (3,3)/(3,4)/(3,5)/(4,3)/(5,3), maximum_power2, đúng các đường direct đã có native
chain evidence; pin117/121 không tự chứng minh map state tổng quát. Không tự export/migrate profile cũ.

Sau **broadcast158 root thành công**, xét current-generation/fence-valid/unreleased pending jobs.
Root ở clear cell, power trong bound mới mở bốn tia; cell ngoài whitelist dừng. Chỉ secondary đầu
trên từng tia, một bomb không chồng/đủ power bound được bỏ reservation. Không recursive hoặc đoán
vùng vuông/moving/overlap/tile mới. Root khác owner vẫn có thể release secondary đúng current owner;
count quota vẫn tách owner. Không tin client report hoặc report-time để release.

Marker `capacity_release` có rootID/generation/time/source design. **Jobs, ticket và deadline6s giữ**,
secondary vẫn nhận own158 như trước; không dùng marker để consume fence hoặc tạo158 mới. Quota bỏ
jobs đã marker. Ownlater callback của secondary đã marker không mở lại release lên bom mới (tránh
chain giả từ job cũ). Finish/Start fence và capacity reset giữ; stale-generation root không release
current bomb. Unknown paths/power/ambiguous overlap vẫn giữ reservation, có thể còn conservative.

Runtime final có hai vòng: mỗi vòng actual A/B/A/A/A118 nhận5IDs, root process3 đầu cảhai view;
hai normal replacements trên hai ô khác nhau đến sau bothview root complete và trước ownsecondary
retirement, cảhai159 nonzero/full39, new bombs process đúng một lần, oldsecondary ownlater không
process lặp. Tổng10native159/10full158/13 mỗi view, 121/clock/175full302/180full3/roomrefresh hai lần,
keys12345/12346; native stats/getter và policy state reset kiểm riêng. Native E9F220 admits corridor
steps trong root callbacks; không suy đã đo mọi wall/class. Không controller/service/probe errors.

Các run trước giữ nguyên: thirdA(6,3) ngoài power2 chỉ native rootprocess1,2/policy chỉ releaseB2;
run hai vòng đầu chỉ8requests, run safe9requests, không đủ scoped acceptance dù có resultreset.
Trong run safe, Space thứ hai ởX195/Y399 trước native159END của replacement đầu; A17 đến muộn hơn.
Đây là **ordering**, chưa causal proof ACK/pool/movement guard. Final dùng Right theo fresh nativeX
feedback thay fixed.25s, không ép state, không tự gán nguyên nhân missing118 từ run trước.

105server models (7 ca mới) kiểm geometry/unknown gap/first-hit/overlap/profile pins/current owner/
fence/reset/preblast rejection và oldsecondary không release bom mới. Model không là native law;
first-hit là policy conservative. Phạm vi ngoài clear-cells/fullmap/dynamic terrain/recursive chain/
fullpool/cell-distance-auth/fullvalidator/fullcodec/authority/allreset/vanilla/LANInternet/fullmatch/
K5/K6a còn mở. [PLAN](../PLAN.md#current-order) giữ việc tiếp, AUD026/periodicCP092.


<a id="round-generation"></a>
### Payload nổ cũ qua vòng mới và tái dùng ID — CP041

**kr_runtime/kr_static confirmed hẹp trên diagnostic copy Wine**;
[controls, recipe, native checks và hash](../evidence/round-generation/README.md).
Không thêm patch/ép state. Hai client A/B, cùng8connections/IDs1,2/slots0,1/session1,
hai actual95/104/121/99/clock; mỗi vòng layout582 rỗng, một bom A rồi17→19,
170 đưa peer vềstate1 trước175/180/về phòng. Fuse6s và outcome Alose/Bwin là **emu_choice**.

**Phép thử phân biệt tầng:** giữ payload158 của bom vòng1, mã hóa lại bằng **seq S2C hiện tại**
cho từng receiver ở vòng2. Đây là replay semantic có CRC/seq hợp lệ, không raw packet replay
và không bằng chứng vượt kiểm seq/crypto. Gửi một lần trước bom mới và một lần sau khi159 mới
đã cấp entity/ID, trước timer6s hiện tại. Các158 đều đọc13/13/errors0 qua native và runtime.
Variant đã dùng là `u8 count=1,u16 ID,u8 kind=0,u32=2,u32=FFFFFFFF,u8=FF`;
không có field round riêng trong variant này. Không khái quát mọi optional flag/mode/transport.

| Ca | Runtime vòng2 ở cả A/B |
|---|---|
| `unique-control`: cấp ID1 rồi2 | Hai158 cũ/ID1 không tới lookup-return E1449C hoặc kind0; timer hiện tại/ID2 mới tới entity159 vòng2 và nổ. A17 sau mốc record server6,738s. |
| `reused-ID`: cấp ID1 rồi1 | Trước159 mới,158 cũ không tới lookup-return/kind0. Sau bind ID1 mới,158 cũ lookup đúng **entity159 vòng2**, đi E9E660/cleanup; A17 sau mốc record server3,644s, trước timer hiện tại6,109s. Timer đó sau cùng vẫn lookup entity nhưng không nổ lần hai. |
| `guarded-reuse`: ID1 rồi1, fence job local | Giữ control158 trước bind như trên; job cũ sau bind bị loại với `stale_round`, không phát packet. Timer hiện tại6,067s mới vào kind0 của entity vòng2; A17 sau mốc record server6,651s. Hai vòng vẫn có statepeer1 trước175 và180/phòng. |

Các khoảng trên lấy từ `created_ns` của record server sau phát159, không phải timestamp breakpoint bind.
Không dùng pointer lặp, ảnh bom prediction hoặc việc đọc packet đủ làm bằng chứng nổ.
159 trả entity khác0 cả hai; B37880 bind được probe ở đường local A, remote B kiểm bằng
entity159→lookup ID→kind0 cùng pointer. Trace từng158 ghép theo thứ tự TCP/seq riêng của receiver.
Ảnh tức thời sau gửi có thể trước effect; callback và ảnh trễ được đối chiếu riêng.
Trước bind không có kind0 chỉ trả lời ca đã đo, không chứng minh toàn heap/entity/timer đã xóa.

**Cạnh tĩnh giải thích:** E5F450 đọc ID/kind/args rồi E5F5CA→E14360.
E143A2/E143C3 tìm ID trong context hiện tại; không có record phù hợp thì trả sớm trước E1449C.
Probe E1449C sau lời gọi nhận pointer; E144E9→B323D0 kiểm trạng thái hiện tại trước nhánh kind0 tại
E145DD→E9E660. Nhánh state giải thích ứng viên cho duplicate không nổ thêm;
chưa phục hồi đầy đủ mọi getter/collection/optional context hoặc luật generation server gốc.

**Design server:** [RoundFence](../../server/round_fence.py) giữ ticket theo instance owner phòng,
generation và lần cấp ID; job chỉ consume một lần, `begin/finish` vô hiệu hóa job vòng cũ.
Caller giữ ticket gốc và serialize kiểm/phát packet cùng owner/lock. Không thêm field vào wire KR.
Trong lượt guarded, control packet trước bind cố ý bypass fence để đo consumer; job cũ sau bind
mới là phép kiểm fence. Không gọi đây là chặn mọi đường gửi trong harness.
Module chỉ chặn job local chưa gửi; không thay allocator/quarantine, deadline, chain cleanup,
validator C2S, gói đã gửi hoặc UDP. Kiểm unit và lượt runtime có fence được ghi ở evidence;
không coi log policy tự nó là proof consumer client.

State/coordinates/stat tại clock hai vòng, serialA0 và UDPport giữ nguyên đã kiểm hẹp;
không đồng nghĩa toàn reset/validator/stale-safe. Item index/165 qua vòng mới, pending159 tới trễ,
ID wrap, abort/cancel timer và peerqueue/UDP generation còn mở.
**Chưa nghiệm thu full match/K5/K6a/LAN/Internet**. [Bước tiếp](../RESEARCH.md#q-023).

<a id="late-bomb-confirm"></a>
### 159 cũ tới khi serial local được dùng lại — CP043

**kr_runtime/kr_static confirmed hẹp trên diagnostic copy Wine**;
[recipe/control/hash](../evidence/late-bomb-confirm/README.md). Không thêm patch/ép state.
Giữ ca hai client/8connections/IDs1,2/slots0,1/session1, hai Start/clock và layout582 rỗng,
mỗi vòng một actual118 của A có owner0/serial0. ID hiện tại do stub cấp1 vòng1,2 vòng2;
fuse6s/outcome/delay là **emu_choice/design**.

`no-old159` nhận159 hiện tại mỗi vòng, full39/39/errors0, A bindID1/2;158 cùng ID lookup đúng
entity159 và nổ cả hai. Hai actual17/19, peerstate1 trước175, hai175/180/phòng.
`old159-first` chỉ thêm payload159 vòng1, mã hóa lại seq hiện tại, sau actual118 vòng2 và ngay
trước159 hiện tại trên từng TCP stream. Cả hai159 vòng2 đọc39/39/errors0; không lỗi CRC/seq/parser.

| Receiver | 159 cũ ID1/serial0 | 159 hiện tại ID2/serial0 | Timer158 hiện tại ID2 |
|---|---|---|---|
| A, owner/local prediction |Compare localserial0/incoming0, bind ID1 vào entity dự đoán vòng2 |24 compare samples localserialFFFFFFFF/incoming0, entity return0, không bindID2 |Đọc đủ13 nhưng không tới lookup-return/kind0 |
| B, remote |Trả entity khác0 |Trả entity thứ hai khác0, khác pointer của159 cũ |Lookup ID2 đúng entity hiện tại, kind0/cleanup |

Vòng2 không có actualA17/19, không175/180 tiếp; ảnh cuối cả hai còn scene trận.
Ảnh B ngay sau timer có bubble cho A trong khi A không có; ảnh muộn có thể đổi animation/state,
chưa snapshot secure tại mốc đó nên không gán enum trap/death kéo dài cho remote từ ảnh.
Đây là ca **mất đồng bộ có chủ ý**, không hai trận hoàn tất. Không bịa deathreport hoặc sửa bind để ép kết quả.
ID tăng1→2 đã tránh old158 trong CP041 nhưng không đủ chống old159 khi serial local trùng.

**Giải thích tĩnh có mục tiêu:** handler159 E5E8F0 đưa serialu32 vào info+00/ID vào info+0C.
AEF700 localpool duyệt actor bomb, AEF7AB so getter B37FC0 với65A340(info+00);
nhánh khớp có đường AEF8DC→B37880 bindID. Runtime đo nhánh AEF7AB trên A như bảng;
không suy AEF814 hoặc mọi mode/pool cùng hành vi. Giá trịFFFFFFFF là getter sample trên các
candidate được xét sau159 cũ, chưa gán nghĩa cho toàn pending table hoặc phục hồi mọi reset guard.
Remote B đi đường khác; không bắt buộc B37880 phải hit để công nhận entity/lookup runtime.

`fenced-old159` giữ oldjob từ vòng1 nhưng kiểm ticket gốc ở generation2, nhận `stale_round`
và không phát159 cũ.159 hiện tại/ID2 trả entity/bind đúng A,158/ID2 kind0 trên cả hai;
hai actual17/19, peerstate1 trước175 và hai175/180/phòng như control. Đây là runtime áp dụng
policy local sẵn có, không sửa client hoặc phục hồi epoch native.

Áp dụng [RoundFence](../../server/round_fence.py) cho việc phát159 chưa gửi cần giữ ticket gốc từ
request/round ban đầu và kiểm `rejection()` dưới owner/lock ngay trước send. Không tạo ticket mới
từ ID/serial đang active để “hợp thức hóa” job cũ. Fence là policy local, không field epoch wire;
kiểm lifetime trước159 không tự chặn duplicate159 trong cùng generation hoặc packet đã phát/UDP.
Generation/ID wrap/abort timer và fullvalidator vẫn có scope riêng.
**Chưa fullmatch/K5/K6a/LAN/Internet**. [Bước tiếp](../RESEARCH.md#q-023).

<a id="abort-timer"></a>
### Timer bom còn chờ khi host EOF — CP044

**kr_runtime confirmed trong phạm vi diagnostic copy Wine**;
[control, recipe và hash](../evidence/abort-timer/README.md). Không thêm patch hoặc ép actor/state.
Kế thừa [forfeit/return/promotion](life-results.md#round-finish) và [client thay thế](room-membership.md#abort-rejoin).
Fuse, forfeit, identity/slot và thời gian gửi là **emu_choice/design**, không luật server gốc.

Hai ca `uncancelled-room`/`fenced-room` bắt đầu A/B IDs1/2, slots0/1, cùng namespace chỉlo.
B bấm Space → actual118 owner1/serial1, stub159 cấp ID1 cả hai; mỗi consumer đọc39/39 và trả entity.
Sau đó kill riêng wineserver prefix A; channel A có EOF thực. Caller gọi `RoundFence.finish()`
ngay khi quan sát EOF, trước175. Policy chọn Alose/Bwin,175→180→107 xóa A→110 chủ B như CP039.
Không có actual17/19 hoặc deathreport tổng hợp trong vòng abort này.

Timer cũ đến hạn40s sau record bom, khi B đã trở về phòng:

| Ca | Quyết định local và client |
|---|---|
| `uncancelled-room` |Cố ý bỏ qua ticket đã đóng, gửi một158/ID1 chỉ tới B. Framer nhận seq106, app125DD40 đọc current stage vtable1767A80/CPrepareStage, có PREPARE_RECV158; không vào E5F450/kind0. |
| `fenced-room` |Giữ ticket gốc; `rejection()` trả `round_closed`, `consume()` trảfalse, không gửi158. B vẫn đọc175outer302/inner292 và180full3, xóa slot0 ID1→0, chiefB1 và ở phòng cuối run. |

Control158 có native framer/getters/CRC/partial check; không có callback game nên **không gọi nó
là runtime game-reader13/13 hoặc bom đã nổ**. Không đo crash hay đổi state do gói muộn này.
Hình ngay sau send có thể trước callback; trace stage/consumer và ảnh trễ giữ vai trò riêng.
Ca control vẫn gọi finish nhưng cố ý bỏ qua kết quả tại deadline; đây là đối chứng đường gửi,
không bằng chứng module fence không hủy được job.

`finish()` phải nằm trong cùng owner/lock với kiểm ticket/phát packet; không chờ tới bảng rank mới
hủy vòng đã abort. Timer giữ ticket ban đầu, không lấy ticket hiện tại từ wireID khi chạy.
Module không tự phát hiện EOF hoặc quản lý roster/deadline; caller chịu trách nhiệm.
Job vẫn nằm trong scheduler tới deadline trong harness; fence ngăn phát packet, không xóa entry scheduler.

**Vòng mới với C:** `fenced-replacement` giữ oldtimer drop/round_closed trước Start mới;
A ID1 đã EOF/xóa slot0, C mới authID3/ownslot0 qua87full7. C actual82/95, B chủ actual104
cho B/C đọc121full129, actual99/197/196/153/clock/movement. B giữ channel cũ; C có bốn socket,
prefix và UDPport riêng. Không gọi đây là A reconnect/resume.

Sau input C Down/B Right rồi C Down/B Space, có actual118 mới từ B/owner1/serial1, stub cấp ID2
ở generation2. Timer6s hiện tại consume thành công,159full39/entity và158full13/lookupID2/kind0/
cleanup khớp cùng entity trên cả B/C. Local B bindID2; remote C kiểm entity→lookup, không đòi localbind.
B actual17/19 được chuyển170/17,19 cho C, đọc10/21byte/errors0; secure actorB state1 cả hai
**trước175**. Policy chờ C21 rồi chọn slot0win/slot1lose;175outer302/inner292 và180full3 cả hai,
ảnh C highlight KRLocal/PERFECT, B highlight KRGuest/LOSE; hai view về phòng.
Refresh110 giữ B chủ/state22, C khách/state0. Rank/outcome/forfeit vẫn do stub chọn.

Counter GDB của C chỉ1 vì đây là Start đầu của process C; đó là generation2 của phòng,
không thiếu Start hoặc reset counter của B. Các broadcast mới chỉ B/C, không gửi A đã EOF.
CP040 chỉ tới movement; đây là run mới mở rộng tới bom/death/result/return, không sửa lịch sử CP040.
Không suy đã giải fullvalidator/UDPqueue, gói đã gửi, reconnect cùng account hoặc mọi disconnect.
**Chưa fullmatch/K5/K6a/LAN/Internet**. [Bước tiếp](../RESEARCH.md#q-023).

<a id="battle-exit-pending-bomb"></a>
### Một bom B đang chờ khi normal guest exit — CP127

**kr_runtime/confirmed hẹp; emu_choice/design cho timer/forfeit/scope**, [metadata/hash](../evidence/battle-exit/README.md#cp127).
Reuse CP044 EOF fence, CP12520286/CP126rejoin và actual118/159/158/currentfuse6; không nativebody mới.
CP044 là EOF, không thay proof triggernormalUI. Control201,095886510s: actualBSpace→118/159đủ39 hai
view, B20286 trước158 **4,617702294s**, policyunbombed từchối202`unsupported_battle_exit_after_bomb`/
86`round_not_returned`; timer vẫn158/Afullgame13/Bstage11khônggame158. NoEOFtrướccompletion.
Control971joins/880raw giữ nguyên; không gọi Bstage11 receipt là full158reader hoặc crash proof.

Mode mới `battle_exit: {"mode":"guest_B_pending_202_86"}` giữ caunbombed và thêm **đúng1current
Bownedticket cònpending**, chưa158/timerreject, khôngaccepted17/19tronggeneration. Kiểm scope tại202
vàlại86, sauđó dùng đườngmembershiprelease/finishfence/abort/knownforfeit-lobby củaCP125.
Job không bị xóa ngay: tới deadline gặp`round_closed`, không158/no terraincommit. Cũ`guest_B_202_86`
vàdefaultoff khôngđổi. Multiple/Aowner/alreadyblast/trapped/dead/host/syntheticvariant vẫnunsupported.
Khôngcancel158đãsend, không đặt thêmwireepoch hoặc gán native/originalserverauthority cho localfence.

Fixed203,696107317s/exit0/serviceerrors0: mộtactualB118/159 trước20286/admit→timer`round_closed`/
không158 ởwire hoặc nativegame hai view, Aforfeit175302/ranks1,2/1803/prepare107arrival1106,
B24191/full50/lobby; no17/19 vànoEOFtrướccompletion. 1009joins/891raw/93copiednativepacket cases;
215server tests gồm4newticket/no158drop/race/multiple/nonowner/trap/defaultconfig. Model tách runtime.
B202fiveu32 lần này `[16066,0,0,0,1]`, khácunbombedzeros; vẫnopaque, không gán thời gian/bombcount.
Verifierkếthừa sozero sai giữfailure; bản riêng so đúng decodedcapture vớiservice/nativefields, không
đổi body/scope/criteria. Lần chạy bản mới thiếuPYTHONPATHcũng giữlog; runnerenvsửa, rawkhôngđổi.
Chưa postbombexit/rejoinStart2/hostexit/afterblast/multiple/deadexit/generalreset/auth/fullmatch/gates.
Bước kế chỉ PLAN, AUD039/kếCP128; sourcebinary/DB/pack/oldcomplete/raw bất biến.


<a id="rust-bombs"></a>
## Rust hai owner bom và deadline ticket — CP238 / RUST-M3 partial

[Codeallocator](../../rust/crates/kr-service/src/bombs.rs), [queueadapter](../../rust/crates/kr-service/src/room_io.rs),
[metadata/raw](../evidence/rust-service/bombs.json), [recipe](../evidence/rust-service/README.md#cp238).
Reuse CP024/028/037/044/171/174/188–189/PythonProbeRoom-RoundFence-ProbeService; CP237nativecopy/room/barrier.
Không RE nativefunction mới, không import VNopcode hoặcengineRustcũ. Wire11822/15939/15813 chỉvariant basicđãđo.

Policy`design`: currentArmed/socket-boundowner/153key; knownbasic118variant/rawshape, row0..12/col0..14/power1..2.
Khôngworldterrainvalidation ở lát cắt này; currentnativeclient là nguồn positive cell/input, syntheticoutsidebounds reject.
Serial `(membership,round,serial)` giữ digestfields: duplicate/conflict không cấp ID, không xóa serial sau nổ.
ID toànroom tăng1..65535/khôngresetquaStart/exhaustfail, allocationcounter riêng/checked; pendingcapacityA1/B2,
occupiedpendingcell guard. Capacitychưa chainrelease hoặcstatsmọichar; không lấy survivorreport làm authority.

Modelclone chuẩn bị ticket/ID/serial/reservation, reserve **cả spawn và deadline queue của cảhai** trước publish.
Mỗi delayedbatch giữ originalArcroomscope/round/allocation/ID/ownermembership/recipient; không lookup ID rồi lấyticketmới.
Recipient+subject+round kiểm trướcwrite; writecompleted mới consume **recipient receipt** mộtlần. Capacity chỉrelease
sauhaiTCPwrites158thànhcông, không là nativeexecutionACK/durableexactlyonce. EOF/leave/newround clearactive ledger;
partialwrite khôngrollbackbytes, đóng đúngtransport/vôhiệu membership/round. Nổ sau6s là policy thamchiếuPython,
không khẳng định fusegốc; queue/time/readiness deadlines giữbound. Nohot-swapbackend giữa run.

`kr_runtime/confirmed` hẹp R3: actualA118 rồiactualB118/đúngslot-serial-currentkey12346 → mỗi15939/39/errors0,
entitynonzero khác nhau;15813/13/errors0 lookupID1/2 **cùngentity159** trên từng view, kind0 rồicleanup.
Haiowner/same2process-channel/oneepoch/6shold sau cảhai, FINsaucompletion. FinalUI A bịtrap/HELP,Bstanding;
đây không là healthyactor/life-forward/rescue/death/resultacceptance, chỉbombconsumer+channelhold. WirePCAPalltuples/S2Cseq-SHA/C2Sjoins
vànativecallbacks tách serverlogs. ControllerOSSpace/escapeordinary, không gamecall/statepatch/clientpatch mới.
Snapshotkhi đanggiữphím ởR2 gây inputDown~39s; rawtimestampsgiữ, không nhận600msnhư dựkiến. R3bỏsnapgiữaphím,
luôn báo actualtiming; không suy causality từ sceneinput hoặc nhận đầy đủlife/gameplay.

R1nativebomApositive/44rawnhưng167/body32unsupportedclosesboth. R2haiownerbompositive/50rawnhưng167/body4
shortvariantclosesA tronghold; cảhaioverallfail/frozen-seal-cleanup0. **167semantics unknown**, Rustchọn
`emu_choice/design` currentArmedboundmember/exact4or32/budget64/member-round/noACK-effect-forward; Pythonunknowncase
noresponse chỉreferencepolicy, không recoveredserverbehavior. Không nhétbody4vào32/hayignoremọiunknownopcode.
1427byte knownlayout được ghi observedrejection, chưaterrain thì **khôngrevealhidden/khônggrant165hidden**.

32Rusttests/40kernelTCPcases kiểm malformed-knownvariant/key-owner/cell-power/duplicate-conflict/capacity/ID/tickets/
foreignroom/oldround-reusedmembership/tworeceiptonce/queuefailureall4entriesrollback/noID-serialreservation/EOFpending158.
Kerneltestlaunch từngracependingbuild rồireject167: giữfail/hash-atfinallyambiguous; sửa awaitbuildthực+hashtrướcPopen/
unchangedcheck,40casespass đúngfinalbinary. Syntheticnegatives không nativeattackproof. CP237shutdownrootcauseunknown
vẫn giữ, passingrepeatkhôngfixclaim. Preflight135sargreject trướcnamespace/clientrun cũnggiữ, dùng120srecipe mới.

M3 cònterrain/reveal/chain/life/result-reset/twooutcomes/kim-cứu/capacity4-8/accountbridge; khôngoriginalserverauthority/
fullservice/production. Pythongiữ reference/serving, [PLAN](../PLAN.md#current-order) giữ thứ tự. Internet thật chưa kiểm.


<!-- protocol-topic-content-end -->
