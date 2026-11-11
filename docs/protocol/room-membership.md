# Phòng, roster và membership

[Mục lục](../PROTOCOL.md) · [Câu hỏi đang mở](../RESEARCH.md) · [Thứ tự việc](../PLAN.md#dashboard).

<a id="room-create"></a>
### Tạo phòng: input, danh sách kênh và request — CP016

Nguồn `kr_runtime/confirmed` trong các run nêu tại [evidence](../evidence/room-input/README.md),
`kr_static/confirmed` cho reader/dispatcher/native oracle. Cùng copy SHA`660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3`;
không thêm patch binary hoặc ép ready/index/level. Fixture và bounds là `emu_choice/design`.

- Click client-relative286,81 tới WndProc`1259C00→125C250→1504563`, scale1; button RTTI
  `CCtrlButton`, nhả chuột ở`1509525` gọi`810230`, interface`CLobbyRoomUIWnd+FC`, vtable`17152D4`.
  Command getter`14FEBDF`/member`+100` đọc **2011**; không dùng ID nội bộ/primary vtable thay callback.
  Command2011→`813060`; control có `+4EC74=0,+4EC70=0` và trả ngay.
- **S→C91**, u16 count=0 (seq49 ở channel socket sau17/241), UI dispatcher`810AC0`→`8150F0`:
  handler tự đặt`+4EC74=1`. Nhánh count>0 đọc blob, mỗi entry u16/u32 rồi object consumer;
  tại CP016 mới xác nhận count0. CP028 bổ sung một row không rỗng ở [contract join](#duo-room-round); các variant khác vẫn mở.
- Sau91, callback qua guard rồi kiểm level`1220360`; control chưa có bảng kênh chọn index−1,
  getter`13A2E10` tính`context+E4*index+84`, gây fault`52A219` trên secure-box ngoài hàng.
  Chưa quy lỗi này cho profile hoặc input; snapshot xác nhận index−1 trong run này.
- **S→C14**, base dispatcher`D68C90`→`D69208`→`13A2FD0`→cache`13A1DE0`.
  Nhánh outer u8=1 đọc blob; raw blob là `[18,u32LE length,storageSelector=0,innerFields]`.
  Byte selector nằm **bên trong** phần length. Thiếu selector đã gây reader length0/count26276;
  frame/CRC/first-field pass không xác nhận blob/schema. Nhánh selector1 là compressed, chưa kiểm.
  Inner có u16LE count, mỗi row theo`13A20D0`: string(u32LE byte-length),u32LE,
  7×u8,2×u16LE,4×u8,2×u32LE. Chỉ count1/name rỗng/kind10 đã thử; semantics/bounds còn thiếu.
  Native blob reader xác nhận count1/33byte inner. CP034 nối trường u32 cuối tới
  loại sảnh khi quay về tại [room lifecycle](#room-leave); các field còn lại giữ giới hạn CP016. Runtime copy bảng qua`13A28F0`,
  lookup`13A2C10` cập nhật selected0; getters min/max trả0/65535, check level trả1.
- Client tự mở `CCreateDlgRenew` (vtable`1752274`, modal caller`DB2170`). Click xác nhận
  client-relative343,479 làm modal trả3 →`DB3550→DB35E0`, builder`DB3617` chọn **opcode81**.
  Capture `create-confirm` socket4 có **C→S81/107byte**, seq88 trong chuỗi run này.
  Writer`840350`; parser KR + hai string getter +12u32 +4u8 tiêu hết payload69byte:
  string1 bytes`bec6b9abb3aa20bfc0bcbcbfe4` (cp949 UI title), string2 rỗng;
  u32 `[0,1,5,8191,0,0,0,FFFFFFFF,0,0,0,0]`; u8 `[0,0,0,0]`.
  Chưa gán tên/nghĩa mọi field từ số mặc định; password/title variants chưa kiểm.

Request77 từng được dùng làm mục tiêu tìm kiếm, **không phải opcode của đường tạo phòng đã đo**.
Tại **CP016**, chưa gửi response hoặc thấy room stage; kết quả lịch sử ở
[evidence input](../evidence/room-input/README.md). S→C81 locator base `D6A3BC→5D9E60` có guard,
không phải pairing đã xác nhận. CP017 xác định đường đang hoạt động dưới đây.

**CP017 — `kr_runtime/confirmed` trong phạm vi copy**, [raw/control/oracle](../evidence/room-reply/README.md).
CNormalLobbyStrategy vtable`17675D0`, virtual+2C=`DB29A0`; listener+8 là interface CLobbyStage+61E8,
vtable`1766D84` (chỉ5 method). S→C86 đi từ room UI dispatcher`810AC0→810B9C→815740`:
outer u8 result0 → `815797` → holder+128/getter`79E9C0` → strategy virtual+2C.

| Payload offset | Getter / giá trị đã đo | Phạm vi |
|---|---|---|
| 0 | u8 result=0 | Đọc ở815740 trước body strategy; chỉ success fixture đã runtime. |
| 1 | u16 room=1 | DB29B6, setter123FA30; giá trị1 là emu_choice. |
| 3 | u32=0 | DB29C7; tên/nghĩa trường chưa xác nhận. |
| 7,8,9 | 3×u8=0 | DB29D7/DB29EA/DB29FD; giữ tên trung tính. |
| 10 | bool=false, virtual+38 | DB2A10; native bool reader giải XOR byte rồi chuẩn hóa0/1, byte lưu false là5A. |

Fixture49byte/payload11 ở socket kênh4, incoming seq55 sau17/241/91/14 (43/46/49/52).
Stub chỉ gửi sau **C→S81 thực tế**, không gọi strategy hoặc setter stage từ debugger.
Native parser và mọi getter tiêu hết11byte, CRC-sai/partial bị từ chối; runtime cursor11/payload11
và lỗi0 đã đo riêng. `DB2AAC→12562F0(12)` rồi init`DC1AB0`, RTTI CPrepareStage/vtable`1767A80`.
Ba run với fixture86 thấy chuyển12; hai run probe init xác nhận class; ảnh có tám ô, chọn nhân vật/đội,
nút Sẵn sàng, **ô người chơi và map vẫn trống**. Chưa xác nhận slot/host/ready hoạt động, gameplay,
full auth/profile, cold/warm/reconnect, vanilla hoặc gate K3/K4.

S→C92 là nhánh khác: `DA2185` cần feature`7A0180` và region`1220480`; run reply92 đo1/0,
không tới `DABD40`/strategy hoặc12. Đây là negative trong cấu hình đã đo; không suy92 vô dụng mọi region.
CApp pending registry77..85 thực tế NULL tại request81, không dùng pending82 làm đường ACK đã có.

Sau chuyển12 client gửi **C→S585/42byte**, builder`11E1820`, một u32 từ context+7C
(native capture=0 trong ba run). Stack init`DC4AB1→11E1820→1164D50→13ADD10` đã quan sát;
semantics/version/catalog chưa xác nhận. Receiver CPrepareStage`DC51F0` có fallback`D5F5A0`;
static opcode106 → virtual+2B4=`D62750` → constructor`AE4000` → `AE3360` và `AE3950`.
Tại **CP017**, đây là locator **kr_static/confirmed**, player schema/hiệu ứng vẫn candidate, chưa gửi106/107.
CP018 dưới đây thử106 sau585; không xác nhận585 chỉ dùng để xin roster hoặc là pairing server gốc. Không gán map/user layout từ VN hay đoán opcode theo độ lệch.
Profile/auth-prefix và các giới hạn CP015/016 tiếp tục giữ mở.

<a id="room-list-lifecycle"></a>
### Nhiều row, cập nhật91 và xóa90 — CP226

[Recipe/raw/hash](../evidence/room-list/README.md), [metadata](../evidence/room-list/lifecycle.json).
Reuse CP016/028/161 row/header/blob/title/featureguards; VNlobby88/78 chỉ phương pháp, không portopcode/layout.
Private2accountservice/profile chỉ đổi roomlistA/B và bỏ single-rowtitlemode;45fixturepins khác giữ nguyên.
Rows/timing/listing là `emu_choice/design`; dưới đây là `kr_runtime/confirmed` consumer/input của client KR.

**Hai row:** session1/roomID101/RoomAlpha và session2/roomID202/RoomBeta, giữ22bytestail đã đo;
outer91u16count2+serializeduncompressedblob. Cảhai native8150F0/D9D720 inner81/81,
outer93/93/errors0, UI2cardsS02/S03 WAITING1/4. Doubleclick thường Arow1/Brow2 tự phát C2S82 typed
`[session,roomID,0,1,5]`=`[1,101,0,1,5]`/`[2,202,0,1,5]`. Adapter chỉ ghi82, **không gửi87**;
chưa nativejoin/membership/registry hoặc semantics mọi field/tail.

**91 cập nhật, không full replacement trong profile đã đo:** gửi count1/session2/ID303/RoomGamma ở
seq55 rồi count0/seq58. Native count2→1→0, inner81/41, outer93/53/2/errors0. Cảhai UI vẫn giữ Alpha,
đổi Beta→Gamma và count0 vẫn giữ hai card; actual82 A1/101, B2/303. Count0 không phải clear-all ở trạng thái
này; không tổng quát mọi feature/mode. Mỗi connection phải tiếp đúng sequence sau fixedsetup46/49/52.

**Correction giữ failure:** ca update đầu dùng `send()` default73 trong adapter vốn gửi fixedsetup nhưng
chưa set nextseq. Hai client đóng TCP/GDBremote sau frame, không consumerupdate/empty/click; run185raw
được giữ. Bản mới chỉ initialize missingnext55, giữbody/profile/input, đo được55/58 và native/UI/wire trên.
Không khẳng định nativeCRCreturn hoặc rootcause chỉ từ code/EOF. Remoteconnectionclosed trong cleanup bình
thường không là lỗi runtime; observer strict phải phân biệt mốc completion/cleanup, fatalmarker vẫn kiểm.

**90 đường xóa có ID guard:** jumptable810C80 map opcode90→810B78→8152C0, không phải92.
Outer `u16 count1,u16 count2`; blob1 nếucount1>0, blob2 nếucount2>0. Nhánh đo **count1=0,count2=1**,
blob2uncompressed có `u8tag0,u16LEsession,u32LEroomID` (7byte); outer21byte. Native lookup8117E0,
validobject D9D5D0 và object+5C==roomID mới vào815570→D9DE80(clearwrapper+38); cuối813B60refresh.
WrongID999/session2 gửi cảhai: nativeinner7/7/outer21/21/errors0, không matchingclearbranch, A vẫn chọn
row2/ID202. MatchingID202/session2 chỉ gửiB: branch hit vớirowobject thật, cardBeta mất/Alpha còn;
Bclickrow1→82/ID101. UIA2cards/B1card và nativeframes/PCAP đối chứng. Đây là guard của listcard,
**không** xác nhận server chống stalejoin/epoch/reuse hoặc membership authority.

Firststream90 (calleeD9DC10), secondstreamtag1 (D9D9A0), count>1/bounds/unknownsession/full/password/
mode/region khác chưa đo. Không coi clearedwrapper là xóa object/row khỏi mọi store; chỉ cardeffect đã kiểm.
4relatedruns545raw/sourcefreeze/ownedcleanup, gồm seq73failure; không clientpatch/nativefunctioncall/
SDK-VMgraph/Rust/production/gate. [Câu hỏi](../RESEARCH.md#rust-service-gaps), [thứ tự](../PLAN.md#current-order)
tiếp selected82→87session/context/title; CP225mergePR296/3bd1cdd, AUD065qua224/kế228. Internet thật chưa kiểm.

<a id="room-join-projection"></a>
### Chọn row → reply87 → context phòng; generic join error — CP227

[Recipe/raw/hash](../evidence/room-list/README.md#cp227), [metadata](../evidence/room-list/join-projection.json).
Source-first CP028817820/8169C0/known87 + CP226two-rowlist; không RE lại codec, không gọi hàm native.

**`kr_runtime/confirmed` projection dương tính:** A82(session1,ID101) nhận87(result0,slot0,session1,0,0,0),
B82(session2,ID202) nhận87(result0,slot1,session2,0,0,0). Native8117E0 lookup row khác0, rồi123FA30
đặt session,123FB70 lấy roomID từ row+5C,12409E0 đặt title lấy từ row: A1/101/RoomAlpha, B2/202/RoomBeta.
8179F3 ghi7/7/errors0, UI headerS02/S03 và đúng tên. Client phát585; adapter **không trả585**,
roster/UI ô player còn trống. Đây là context projection, không hai phòng hoạt động/membership/registry isolation.
Hai process có thể cùng virtual address cho context; không suy chúng cùng object.

**Phân biệt policy và lỗi native:** fixturebackend tuyên bố session2 hiện mang ID303 nhưng client còn row202
là `emu_choice/design` để thử reject trước membership. Native table8177E4: result1→816A62/resource
`WrongPassword`17158DC; result2→816B2C/resource `CantJoinRoom`17158EC. Mẫu result1 ban đầu đọc1/1/errors0,
UI nói sai mật khẩu nên **không phù hợp làm mã staleID**; giữ ca/report/input gốc và correction, không giả đây là
luật server gốc hoặc password đã hỗ trợ. Result2 là generic “không thể vào phòng”, không khẳng định lý do gốc.

**Route generic2 đã đo:** Bactual82/ID202 bị privatepolicy trả87/onebyte2 seq55; native error branch/handlerend
đọc1/1/errors0, UI thông báo không thể vào phòng; không setter session/title hoặc585 trước khi chấp nhận.
Ordinary Return đóng notice,91seq58 cập nhật row2/ID303/RoomGamma (53/53/errors0), ordinary center-card
click phát82(2,303,0,1,5),87success seq61 đi lookup/setters/7/7/errors0 tới session2/ID303/RoomGamma;
actual585 xuất hiện sau đó. Avalid101 tiếp projection riêng; cùng auth connection/session không bị đóng để retry.
Đây là client error/refresh/retry trên cùng channel, khác automaticloginresume bị fatal ởCP224.

**Correction input:** hai ca retry đầu click lại cùng edge730,170 không có callback2501/8132B0/new82,
không được tính recovery. Observer mới xác nhận callback đầu đi pending0/list1/vectorcount2/row202/
eligibility1/predicate1/send811AD0,87entry812070 đặt pending0; không có proof guard chặn retry.
Recipe ordinarymoveout300,450/.25s rồi click card-center600,150/150msdoubleclick có callback lần2/row303,
không widgetinvoke/forcedcallback/guardwrite. Nguyên nhân edgeclick không nhận callback vẫn unknown;
không gọi đây là sửa protocol hoặc lỗi logic client. Code1 inputcontrol giữ riêng trước ca code2 cuối.

5relatedruns802raw/sourcefreeze/ownedcleanup, gồm no-retry225raw+observer155raw failures và hai correction
control. Giữ mọi raw/source/report/checkpoint/audit cũ. Chưa reply585/roster/ownslot/currentroommembership/
roomstateisolation/Ready/start/UDP/registry/DB/fullauth/Rust/gate. [PLAN](../PLAN.md#current-order) tiếp roster
riêng từng room bằng consumer đã biết; không lặp list/context/error2/input controls. CP226mergePR297/ccab4cc,
AUD065qua224/kếCP228. Internet thật chưa kiểm.

<a id="room-local-roster"></a>
### Roster riêng của hai room context — CP228

[Recipe/raw/hash](../evidence/room-list/README.md#cp228), [metadata](../evidence/room-list/local-roster.json).
Source-first dùng CP020/026/199/200 và CP227; reuse merged `four_player_profile.player_body/slot_body/own_body`,
không dùng tám source WIP. Không có thân hàm native mới hoặc runtime mới ngoài run đã seal dưới đây.

**`emu_choice/design` input:** private adapter để cả hai reply87 slot0, A session1/ID101/RoomAlpha và
B session2/ID202/RoomBeta. Sau actual585 mới kiểm admission session và active owner tương ứng identity1/2,
rồi trả **chỉ own106/110/291 tới cùng connection** (seq58/61/64; body204/6/128). Mỗi phòng một member/chief,
team0. Các opcode sau projection khác chỉ record/ignore; không trả global map/peer/Ready/start/Battle.
Generated body là input thử; không là oracle độc lập hoặc luật server gốc.

**`kr_runtime/confirmed`:** cả hai client parse106 slot0/identity riêng; full204/204,110 full6/6,
291 producer128/128, errors0. Native13B1577 self-ID candidate khớp incoming1/2;124001C tìm local identity
index0;123FF10 setter nhận slot0. UI A RoomAlpha chỉ KRLocal ở ô đầu/chief; B RoomBeta chỉ KRGuest ở ô đầu/chief,
bảy ô khác trống. Giữ8.864759497s sau native roster; PCAP/two actual/two endpoints/authowners đã đối chiếu.
Không generated/routed member frame của phòng kia trong ca này; đây chưa là chứng minh mọi packet isolation.

**Correction marker:** probe raw kind `own_slot` tạiDC5925 ghi EAX1. Cache DC5890.asm:41 xác định
DC5920 gọi1241090 rồi DC5925 testEAX; đây là predicate return1, **không phải slotindex1**. Giữ label/raw/source;
slot0 được chứng minh bằng setter123FF10/106slot0/UI, còn own-match-index0 là index localidentity.
291 producer/name KRLocal/KRGuest đã đo; personal-render sau291 chưa đo vì client đã ở Prepare.
Map caption ExVillage01 từ cache, không là nghiệm thu map mới.

Một run50.735937193s/129raw/exit0,377 frozen pins, ownedcleanup0. Private fixed recipient guard chưa là
registry/membership lifecycle/sessionmove/general room security; production ProbeService vẫn globalroom.
Chưa Ready/start/UDP/gameplay/DB/fullauth/Rust/gate. [PLAN](../PLAN.md#current-order) chuyển atomic membership
theo room tuple/generation/account owner và native leave/rejoin controls, không thêm fixedprojection-only runs.
AUD066 quaCP228/kếCP232; record/audit cũ giữ nguyên. Internet thật chưa kiểm.

<a id="room-membership-registry"></a>
### Registry private, Back/rejoin và chuyển phòng — CP229

[Recipe/raw/hash](../evidence/room-list/README.md#cp229), [metadata](../evidence/room-list/membership-registry.json).
Source-first đối chiếu CP034/035/107 và CP226–228; merged four-profile builders/consumer106/110/291 giữ nguyên.
`boomkr-re@88e11ed/kr-data.md553–559` là oldVN UI hint/Bjoin chưa xác nhận; `boom-re@9ac2075/lobby.md603–615`
chỉ mẫu kiểm ghost/chief/per-room routing, không port opcode. Không thân hàm native mới/patch/SDK/VM graph.

**`emu_choice/design` registry:** một event-loop owner, không await trong mutation; room key `(session,roomID,
internal generation)` và member handle thêm identity/account-session/connection/slot/monotonic visit.
Join kiểm current owner/room trước cấp slot; duplicate cùng member idempotent, wrongowner/stale/full không mutation.
Leave/cleanup so exact handle, old cleanup không xóa visit mới. Trước delivery kiểm lại owner/current room/visit.
Public(session,ID) đã remove không reuse trong process vì native82 chưa có measured generation; tombstone không
là durable restart fence. 19 model scenarios (owner/full/duplicate/stale/ABA/cleanup) và7 move scenarios chỉ kiểm
policy, không originalserver law hoặc native compatibility. Atomic ở state transition event-loop, không DB/send transaction.

**`kr_runtime/confirmed` run R2:** hai actual native auth/session giữ; AAlpha1/ID101/slot0chief và BBeta2/ID202/
slot0chief. OrdinaryBBack113,581→actual86empty, registry bỏ đúng member B;1s sau private241/91 refresh normalLobby.
Ordinary center-card82Beta→87slot0/session2→585→106/110/291 đúng ID2/KRGuest/slot0/chief; visit mới khác visit cũ,
Acontext/roster1 giữ. Native87full7,106full204,110full6,291producer128/errors0; UI/PCAP/same8TCP/two authowners,
8s+hold. Old queued recipient handle kiểm false, không wire send — đây là policy observable, không native stale injection.

**`kr_runtime/confirmed` run R3:** sau initial hai room như trên, BBack→normalLobby rồi actual82(session1,ID101),
87slot1/session1→native context101/Alpha/ownslotsetter1→585. B nhận own106ID2/slot1/state0 (khách),110/291,
peer106ID1/slot0/state2 (chủ); A nhận thêm106ID2/slot1/state0, Aownslot0/chief giữ. Hai ảnh đều Alpha,
KRLocal ô đầu/chủ, KRGuest ô hai/khách; B có Ready, A có Start nhưng không bấm hoặc nghiệm thu Ready/start.
Native mọi106full204/110full6/291producer128/87full7/errors0; registry Alpha2members, Beta0, hai account sessions/channels giữ,
8s+hold. Capacity8 là policy, chỉ hai actual được đo; không native8/fullroom proof.

**Giới hạn/correction:** decoded actual585 ở cả hai run là4byte/u32zero, không observed room/epoch.
Captured internal member handle chặn queued reply cũ; bytes585 cũ giống request mới sau rejoin chưa thể phân loại stale.
Không thêm epoch vào wire hoặc tuyên bố replay protection tổng quát. Raw markerDC5925/predicate1 vẫn không là slotindex;
291 producer/name không personal-render. R3 controller kế thừa text “Aunchanged”: Acontext/account/channel giữ,
**roster A có thêm B**, không unchanged roster. Cards vẫn fixture occupancy1/4, không phản ánh Beta rỗng của registry.

R1 lỗi import repository trước service-ready/chưa mở client:0.20888104s/7raw/exit1; giữ source/raw/cleanup,
R2 sửa setup import trên bản riêng sau seal. R2+R3=336raw, cả batch343raw frozen/sealed/ownedcleanup0.
PCAP direct-parser fallback kiểm271+262record/14+16dynamicframe theo stream; REA TShark provider thiếu capability,
không có successful REA capture Evidence/không sửa toolconfig. Chưa production integration/general shared-leave/
EOF fanout/promotion/occupancy/roomquota/Ready/start/UDP/DB/fullauth/Rust/gate. [PLAN](../PLAN.md#current-order)
tiếp shared-membership cleanup bằng107/110 đã biết, không lặp join/projection. AUD066qua228/kế232. Internet thật chưa kiểm.

<a id="room-shared-cleanup"></a>
### Cleanup membership trong registry private — CP230

[Recipe/raw/hash](../evidence/room-list/README.md#cp230), [metadata](../evidence/room-list/shared-cleanup.json).
Reuse [107/110/normalLobby CP034](#room-leave), [rejoin/promotion CP035](#room-rejoin) và
[CP229 private registry](#room-membership-registry); không thân hàm native mới hoặc protocol redesign.

**`emu_choice/design` routing:** chụp current member handles trước leave/connection cleanup, detach exact handle
rồi chỉ gửi107/110 tới peer cùng room key/generation còn current/activeowner/đã roster. Body107 giữ slot/identity
người rời;110 dùng slot người sống và chief theo minslot còn lại. Notify không mutate membership, old captured
peer đã move/owner replacement không nhận;9policy scenarios kiểm adapter thật với fake send và accountC ở room khác.
AccountC chỉ model, không thirdnative/no-allpacket-isolation claim. Đây là policy bầu chủ, không luật server gốc.

**`kr_runtime/confirmed` ba ca riêng**, hai KR/Wine/copy4b/direct virtual LAN, cùng source/profile/probe đã frozen:
initial AAlpha/Bbeta rồi B chuyển Alpha theo CP229; sau đó chọn một trigger bằng config trước run.

| Trigger | Bằng chứng native và lifecycle |
|---|---|
| Guest B ordinaryBack | Actual86 lần2/bodyempty→107 tớiA(slot1,ID2,0), native ID2→0/UI ô B trống; A chiefgetter1 saucleanup, B241/91 về normalLobby, hai authowners/channels giữ |
| Guest B ownedprefix EOF | Bkill riêng/wire không thêm86 sau initialroommove, channelEOF/releaseBsession;107 tớiA/ID2→0/UIclear, Aowner giữ và khỏe8s+. Không claim Bhealthy hoặc UIBack |
| Host A ordinaryBack | ActualA86/bodyempty→107 tớiB(slot0,ID1,0), native ID1→0;110 cho Bslot1/state2/nativefull6/errors0, UI chiefbadge/Start và slotA trống; A241/91 về normalLobby, haiowners/channels giữ |

107 ở mỗi ca có **base6/6/errors0 → rewind0 → UI reread1/6**, intentional hai pass; so slot ID trước/sau đúng
người rời trong cùng remove-handler interval. Một107 cho survivor, không107 cho leaver. PCAP exact dynamicframe/
sequence/order và native/UI đối chiếu;20/18/20frame,302/274/304capture records;8s+hold sau nativecleanup.
Không bấm Ready/Start hoặc suy Startbutton thành Battle acceptance; dynamiccardoccupancy vẫn fixture1/4.

**Giới hạn observer:** host run getter1240D5E có14initial1+6guest0, hết budget20 trước promotion. **Không có
postpromotion getter sample**;110full6/state2 và UIchief/Start là evidence riêng. First verifier đòi sample đó
đã fail, giữ source và measurement-correction; host-specific verifier ghi unobserved và kiểm evidence khác,
không rewrite raw hoặc claim getter1. Guest cleanup hai ca có actual survivor getter1 sau107 riêng.

**Correction controller:** EOF R1 dừng khi tìm cửa sổ B đã chủ động kill;166raw/exit1/partial, không đạt8s hold.
Seal/cleanup trước khi tạo R2 chỉ bỏ windowlookup sau expectedEOF, giữ survivor health checks/adapter/profile/probe.
R2EOF187raw đạt. GuestBack189 + HostBack189 + EOF187 =565positive raw; cùng failure166 =731raw frozen/sealed/
ownedcleanup0. Chưa productionintegration/send-failure/backpressure/thirdnativeotherroom/generalroomquota/
restartfence/Ready/start/UDP/DB/fullauth/Rust/gate. [PLAN](../PLAN.md#current-order) tiếp module policy/reference và
seam theo session→membership→recipient, không thêm broad RE/phép lặp đã giải. AUD066qua228/kế232. Internet thật chưa kiểm.

<a id="room-player"></a>
### Player, own-slot và controls phòng (CP018)

Nguồn **kr_static/kr_runtime confirmed trong phạm vi copy**; lựa chọn field/API **emu_choice/candidate**.
Recipe/control/correction/hash tại [evidence](../evidence/room-player/README.md). VN docs về
CSetPlayerOnPreparePacket, profile tên, ID match và unlock controls giúp chọn probe; **không copy wire**:
VN103/106/337 dùng3byte state, KR đo106/110/585 và4byte state. Chưa nghiệm thu K3/K4.

**S→C106**, sau stage12: `DC51F0→D5F5A0→vf2B4/D62750→AE4000`.
RTTI xác nhận CSetPlayerOnPreparePacket/vt173CB4C, secondary+5C/vt173CB5C;
base CSetPlayerPacket/AE3360 và accessory CSlotAccessoryPacket/AE3950.
Fixture242byte, payload204; incoming seq58 sau create86 seq55, chỉ gửi sau C→S585 thực tế.
Chọn slot0, ID1 hoặc0, tên ASCII `KRLocal`, state raw4byte `02 01 00 00`.
Đây là lựa chọn thử, chưa gán byte thứ2 thành character1 theo layout VN.

| Offset payload mẫu | Reader/data | Kết quả trong scope |
|---|---|---|
| 0..11 | u8 slot,u32 ID,3u8,u32 | Base+8=slot,+C=ID; semantic ba u8/u32 giữ trung tính. |
| 12..15 | DecodeBuf4 | AE3474/virtual+58 đọc4, không dùng VN3. |
| 16..134 | blob `[18,u32LE length,rawSelector0,inner109]` | D2D310 đọc stringLE đầu `KRLocal`; native/runtime cursor109/109. |
| 135..158 | 3u8,string,string,blob rỗng | String outer dùng reader khác blob; blob thứ hai được đọc nhưng region52/82 đều0 nên không deserialize nhánh vùng. |
| 159..175 | u8 count0,4u32=0 | Feature70/11F/34 trên build; native/runtime base cursor176. |
| 176..203 | accessory headeru16, countA/Bu8=0,sentinelFF,u8=0,u32 count0,u8 countC0,2u32,boolfalse,u32,u16,stringrỗng | AE3950 đi đúng branch native và runtime, cursor204/204/error54=error58=0. Count lớn/optional variants chưa kiểm. |

Profile109 là layout KR theo D2D310, không đệm256byte từ VN: stringLE; widths
`u8,u8,u32,u8,u32,u16,4u32,u8,u32,stringLE,5u16,6u32,u8,5u32,stringLE,u8`.
Tên dài7byte, strings sau rỗng, scalar0; 5u32 optional phụ thuộc feature11F. Feature7A0130=false,
48D8D0=true cho string bổ sung; các wrapper/gate và limits chi tiết trong raw ledger.
Oracle native giữ input readers/control-flow nhưng mô hình hóa alloc/object setters/storage và
selected-region10; **runtime** xác nhận cursors/region0 riêng, không lấy mô hình làm proof nguyên object.

Trong recipe CP018, ID1 hiển thị nhân vật/tên/nhãn chủ phòng sau khi sửa import pointer-check, nhưng local user list
thực tế có count1/ID0. `13B1540` so incoming1→−1, `123FF70` không đặt own flag;
`DC5925` đo own_slot0. Gửi incoming0 theo giá trị vừa đo → lookup0/own_slot1;
client tự gửi thêm96/266, không ép own flag hoặc sửa danh sách ID từ debugger.
Đây là workaround dữ liệu của recipe auth chưa đủ, **không phải ID hợp lệ/stable của server gốc**.
NMCO code1 kế thừa không được đồng nhất với local login user-ID0.
CP020 cấp ID1 qua auth và chỉ ra giới hạn start của ID0 tại [contract start](load-phase.md#start-reply).

**S→C110**, `DC51F0→DC52D6→DC6700`: u8 slot,DecodeBuf4,u8; mẫu payload6/wire44,
seq61, state giống106/tail0, gửi sau106 khoảng2s để quan sát branch. Native getter và runtime
đọc6/6/errors0; slot của mình/state không đổi đưa pending+6224 từ1 về0, gọi vf+2F4/DDF330(1).
Ảnh cho nút Bắt đầu và nút map sáng. **111 là handler khác (DC6AF0)**; giả thuyết111 cũ bị bác bỏ
bằng jump table, primitive-only oracle không chứng minh dispatcher. Không khẳng định ready/team/character
roundtrip hoặc tương đương server gốc từ một mẫu110.

Hai generated import traps trong copy đã gặp thực tế và bind trên copy mới:
16EA2B4→kernel32!IsBadReadPtr (pointer,size60; return0 trong run),
16EA6A8→user32!GetKeyState (VK_MENU12, mask8000; return0 khi không giữ Alt).
Có byte before/after, control và rollback; **tên API gốc vẫn candidate** vì IsBadWritePtr/
GetAsyncKeyState có thể trùng signature. SIGILL tại22624DC/22645D8 là trap dựng khi rebuild,
không phải bằng chứng nhánh anti-tamper gốc. Copy mới chỉ thêm import, giữ nguyên handler/state code.

Click nút Bắt đầu với một player tới modal thông báo số người quá ít; chưa capture request start.
Map trống. Capture sau own match có C→S96/44byte và266/50byte; native getter thử lần lượt
u16/u32=[0,0] và3u32=[0,0,0] tiêu6/12byte, **schema/semantics cần nối writer**, chưa gọi là map request confirmed.
Run `room-guest` thêm KRGuest slot1/ID2/state ready nhưng cùng team8, click tới modal không đủ đội.
Run `teams-retry` dùng state self=`02 00 00 00`, guest=`24 00 00 00`,110 self cùng state;
bits5..9 của lowword đổi team8→0/1 theo consumer`D628E0`. UI đổi đỏ/vàng, đủ hai player/đội để
click tới thông báo **map hiện tại cho tối đa0 người tham gia**. Đây là bằng chứng runtime hẹp cho
team0/1 và nhánh start tại CP018, chưa ready roundtrip ở checkpoint đó. CP028 bổ sung [Ready95→110](#duo-room-round).
Một lần trước đó (`room-teams`) wineboot timeout35s trước client, không tính là phép đo game;
retry prefix mới giữ nguyên copy/fixture/harness.
Tại CP018, Handler117/DC70D0 mới là locator tĩnh và chưa có map/load/Battle.
CP019 bên dưới nối table/ack/filter; CP027–028 có hai client thật. Auth/profile/cache hoàn chỉnh vẫn thiếu.

<a id="current-room-title"></a>
### Tên phòng current81 → guest91 — CP161

**`kr_runtime/confirmed` trong hai copy Wine/private stub, `emu_choice/design` cho chính sách service.**
[Recipe/raw/hash](../evidence/peer-endpoints/README.md#cp161). Tái dùng CP016 request81/string,
CP028 row91/session/87 title context và CP076 inline15/heap16; không RE lại handler hoặc chuyển opcode,
ID/layout VN. Native `14EF300`/`14EB9E0` kiểm actual81, row91 `D9D720`/blob consumer kiểm actual91;
base400000, RVA=VA−base. Copy Wine SHA `e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef`.

Opt-in `room_title: {"mode":"create81_to_row91","max_bytes":32}` parse đúng hai typed string,
12u32+4u8 opaque. Chỉ nhận title CP949 hợp lệ, không control, 1–32 byte và password rỗng; **giới hạn32
và từ chối password là policy, không giới hạn UI/auth gốc đã phục hồi**. Preflight known empty row A và
single uncompressed row B trước listening. Request lỗi không ACK/không đổi membership; title commit
sau create ACK gửi thành công. Chỉ thay title/length trong row91; session, roomID,22 byte tail, sequence
và CRC của đường gửi được giữ/tính theo codec KR. Initial B91 dùng base49, dynamic Back/return-list dùng
sequence hiện hành. Mode tắt giữ đường cũ; không diễn giải52 byte opaque hoặc lấy title làm identity.

Run r1 title13 byte `아무나 오세요`, r2 title16 byte `저랑 게임하실분~`: actual81/native framer/string/CRC/
partial controls, initial và return91/native row đều đạt. R1 ảnh hai view **trước guest Back** tên trùng;
final B đen sau Back không phải proof còn ở phòng. R2 exported preset47 pins chạy module production,
không hold165/active replay, guest Back→91→rejoin87→Ready/Start, hai normal outcomes/reset/Ready;
ảnh cuối A/B tên trùng. Hai epoch Fire winner lần lượt A/B, mỗi epoch một actual claim và một effect/view;
strict56→1/other0 trước175 cả hai view. Getter emulation/model tách runtime, không oracle cả world.

247 server tests (10 title regression),4 export tests đạt; negative/truncation/password/control,
ACK failure không commit, exact row identity/tail, default và sequencing đều có kiểm. Preset bật policy
sau UI positive r1, r2 đo cùng operational profile; provenance-only update sau cleanup giữ nguyên
`profile` và exact frozen preset trong evidence. R2 không Start3/third match; C/recreate/promotion/title
variants/password/compressed/multi-row/full auth/authority/Windows/LAN/Internet/8 client/K6a chưa kiểm.
Natural transient CP157 vẫn unknown; không ghép run riêng thành gate/fullmatch.


<a id="current-291-room-integration"></a>
### Current hai client291/nameguard: phòng/load/start và giới hạn — CP187

[Nghiệm thu K4](../PLAN.md#k4-acceptance), [nguồn/hash](../evidence/lobby-identity/README.md#cp187).
R2 Winecopy4b781f… trên privatevirtualhosts có actualAB/8sockets, nativeproducer291 mỗirole128/128/errors0
và renderguard1/tênKRLocal/KRGuest. Request6 guard cuối nhận đúngname; mọi S2Crecipient seq43+3n liên tục.
Profile47fixture và policies đúngCP171 ngoại trừ optinown_profile; replay/hash/consumer cũ dùng lại.
Actualcreate81/Ajoin82B/hostA86→nativeB107ID1clear/chief1→A82rejoin/A95/B104; roster/map/phòngboth
khớp ảnh. Hai lần native121129/129 và clockboth, key12345/12346, loadingbarrier30scomplete6.079/6.790s
sau actual99both. First175302/302→nativecompare→1803/3→roomrefresh/Ready95→Start104/clock2.

**R2 vẫn partial:**308.016s/exit1/1120raw/frozen166/cleanup0; controller `alignnotstable:R2-A-hidden-row`,
targety159 nhưng retained stable samples299→281, rồi thiếu3neutralreads trongdeadline3s. Không nativecrash/
serviceerror hoặc frame175180failure; round2abortEOF khi cleanup giữ log. Chưa quy rootcause input/key/
physics/probe, chưa secondoutcome hoặc currentStart3. Không dùng verifier2cycles/Start3 cũ để ép pass:
chỉ verifierK4scopedroom/compare và sourcecriteria; fullcycle verifier được chuẩn bị nhưng không chạy/count.
OldCP1712outcomes/Start3 giữ là historicalproof, không backfillrun này. Tại CP187 K5/K6a mở; [CP188](life-results.md#current-basic-cycles-291) có current2outcomes mới và nghiệm thu K5scope, K6a vẫn mở.

<a id="duo-room-round"></a>
### Join, Ready và chu kỳ hai client — CP028

**kr_static/kr_runtime confirmed hẹp trên copy Wine**, [recipe/control/hash](../evidence/duo-room-round/README.md).
Không patch thêm vào copy SHAe9bb47…; hai client/prefix/display/game-dir riêng, chung namespace chỉlo.
Fixture/room/profile/Ready ACK/rank/timing là **emu_choice/design**, không phải policy server gốc.
VN `boom-re@9ac2075/server/src/packets.rs:228–273` hướng dẫn pattern count/blob/session/roomID;
KR dispatcher/schema/gates được kiểm riêng, không đổi opcode VN theo offset rồi coi tương thích.

**Room list91 không rỗng:** `810AC0→810B66→8150F0` đọc u16 count; count>0 mở stream,
mỗi row đọc u16LE session/u32LE roomID rồi`D9D720`. Row mẫu38byte:

```text
u16LE session, u32LE roomID, u32LE name_len, name_bytes
u8, u8, u8, u16LE, u8, u8, u8, u8, u32LE
9×u8 zero theo các guard/tail bên dưới
```

Tên`KRRoom`, session0/1, roomID1; các trường sau tên chọn`0,0,0,0,1,0,4,0,0`.
Pattern player-count1/capacity4/password0/playing0 có UI/card tương ứng; semantics mọi field khác
vẫn candidate/unknown. Tail đi`13704B0` (mandatoryu8,703AD0=false nên bỏ một byte,
A32020=true thêm một byte), D9DFD0=true/u8,A31FE0=true/bool,hai bool bắt buộc,
7FA4A0=true/u8,một u8 bắt buộc rồi`D9DEA0` count0. Không khái quát cho feature/mode khác.
Outer=`u16obf count1 + serializedblob(compression0 + row)`; full payload50.
Control thiếu byte cuối vẫn vẽ card và outer49/49/errors0, nhưng **inner37/37/error1**.
Ca sửa **inner38/38/error0**, doubleclick card tự phát request82; UI/outer pass không thay inner check.

**C→S82/join:** actual body11byte; native widths`[2,4,2,1,2]` trả`[session,1,0,1,5]`,
session0 hoặc1 khớp row. Trường u16zero sau roomID có thể là chuỗi password rỗng nhưng chưa nối
writer/nonempty; hai field cuối cũng chưa đặt tên. Không áp toàn contract VN78 lên KR82.
**S→C87/result0:** `810BAE→8169C0→strategy/v34 DB2B70` đọc u8slot,
`123FF10(slot)→12562F0(12)`; sau đó`817820` đọc u16session,2u8,bool khi`1220520()==0`.
Fixture7byte=`u8result0,u8slot,u16session,u8zero,u8zero,bool0`, runtime **7/7/errors0**.
Roomlookup`8117E0(session)` và setters`123FA30/123FB30/123FB70/12409E0` nối context/title.
B dùng slot1/ID2; các run sau ghi **session1 ở cả A(create86) và B(join87)**. Pair đầu giữ
sessionB0/A1 làm diagnostic control, không dùng nó chứng minh context phòng chung đã nhất quán.
Tại CP028, full room authority và password/error/leave/host variants chưa kiểm.
CP034–035 bổ sung [leave, chuyển chủ](#room-leave) và [rejoin/Start sau chuyển chủ](#room-rejoin)
trong phạm vi đã nêu; authority và các variant còn lại vẫn mở.

**C→S95/Ready thật:** B ban đầu state20/unready; click nút Ready tự tới`DDE760` rồi
`DDEAB7→1164C10(95)`, context`1264220→14EC120`, u8desired qua`14EBC50` vàu32tail
qua`14EBD90`, send`13ADD60`; success đặt pending4. Body đầu60byte gồm serializedbuffer
stored51byte +u8desired1 +u32tail0. Native framer/blob/checksum/getters đọc đủ; inner context chưa
được gán toàn semantics. Ở run cuối, body sau trận322byte, native decode context265→2845byte; không áp cố định60 cho mọi95 hoặc coi growth là lỗi reset đã biết.
Stub sau CRC hợp lệ broadcast110/slot1/state24/tail0, client tự clear pending và A click tự gửi104;
không ép state/ready/104 từ GDB. Bits2..4=1 là ready group; bit khác giữ theo profile đã đo.
ACK/policy này là lựa chọn emulator, chưa thử unready0/error/duplicate/race.

**Chuỗi pair vòng đầu:** A81→86, B82→87, B95→110 cả hai, A104→121 cả hai;
197/196/153 chỉ sau actual99 của cả hai. Seq dynamic riêng mỗi receiver bắt đầu73+3n;
Ready11073,12176,19779,19682,15385; bom/result tiếp tục counter, không chèn198.
Run`duo-join-ready` có4bom/4entity/4kind0 mỗi phía; session control nêu trên giữ nguyên raw.
Các run session1 dùng một bom A/stay, actual127/sub17/19 dẫn policy175→180 tới cả hai,
callback175outer302/302/inner292/292 và1803/3/errors0, cả hai chuyển stage12/UI phòng.
Đây kiểm consumer/chu kỳ local với outcome Alose/Bwin được chọn, chưa đồng thuận hoặc validation
trap/death phía peer. Run cuối`duo-round-restart` có **hai lần actual104 củaA,95 củaB,99/121/153/clock/bom/175/180
mỗi phía, cùng tám connection**; Aport9339/B9256 giữ qua hai99. Seq12176/109,
15385/118,17594/127,18097/130; toàn S→C73..136 bước3 có159/158/110 chen đúng counter.
Hai bomA serial0/0, stub cấp chungID1/2 qua hai stage. B không đặt bom trong runhai vòng này;
không ghép bomB của CP027/firstpair thành proof mọi event đều đã lặp hai vòng.
Ảnh cả hai cho cùng LOSEKRLocal/PERFECTKRGuest (highlight ownID), rồi UI phòng sau180lần hai.
Đây là **hai chu kỳ thử pair**, chưa fullmatch, reset không rò state, item/pickup hoặc gate K6a.
Bấm sau return cần retry hữu hạn:3clicksA→2request104,3clicksB→2request95. Hai run trước chỉ
đạt một vòng hoặc Ready lần hai; nguyên nhân click đầu bị bỏ vẫn unknown (focus/timing/state).
Không suy quan hệ nhân quả từ recipe thay nhiều yếu tố hoặc gọi retry là protocol fix.

<a id="room-leave"></a>
### Rời phòng, chủ mới và chọn loại sảnh — CP034

**kr_runtime/kr_static confirmed trong phạm vi diagnostic Wine copy**;
[10 run, recipe, control và hash](../evidence/room-leave/README.md). VN `docs/lobby.md` §0/§18
chỉ dẫn kiểm ghost slot/chief bit và ba biến thể sảnh; wire KR được kiểm riêng.
Không thêm patch binary hay ép stage/game state bằng debugger.

**Rời bằng UI:** click Back client-relative113,581 đi qua
`DDA160→12562F0(previousStage11)` (return`DDA32C`) rồi`1256100` tạo **C→S86**, body rỗng,
wire38byte, gửi qua`1164D50→13ADD10`. Seq theo socket`40+3n`, không cố định một giá trị.
Escape control không tạo86. Control Back không reply giữ ghost B trên A và leaver ở loading.
`123F820(1)` trên đường gửi reset room context; không gọi routine này đơn giản là pending setter.

**Xóa slot:** **S→C107** = `u8 slot,u32 id,u8 flag`, payload6/wire44.
`D5F5A0→vf2BC/DC62D0→D603A0` đọc đủ6/6/errors0;
`12400D0` so incoming ID với record rồi xóa ID về0. Manager+A8 chứa rows stride318hex,
ID tại row+C. Guest rời gửi107(slot1,id2,flag0) cho A; host rời gửi107(slot0,id1,flag0) cho B.
Runtime và ảnh cùng xác nhận ô người rời bị xóa. DC62D0 **rewind** packet tại DC6327
(cursor6→0) rồi đọc lại slot cho UI; cursor cuối1/6 là intentional second pass, không phải base parser đọc thiếu.
Không gửi107 cho leaver trong ca tự rời; nhánh tự bị đuổi/modal BannedByLeader/ChiefBanned/BannedBySuper chưa kiểm.

**Chuyển chủ:** sau A rời, stub gửi110 cho B: slot1, DecodeBuf4=`22 00 00 00`, tail0.
State B trước đó20hex; bit1 được đặt. Getter`1240CF0→1240D5E` của actual B đổi0→1,
slot A ID1→0, UI B có nút Start. Đây là promotion được client chấp nhận; chọn người còn lại làm chủ
là **design** của stub. Tại CP034 chưa đo click Start; CP035 bổ sung [rejoin/Start](#room-rejoin). Mọi luật bầu chủ server gốc còn thiếu.

**Ngắt guest:** kill wineserver thuộc riêng prefix B trong namespace; channel B quan sát EOF,
stub sau đó phát107(slot1,id2,0) cho A. A xóa record ID2→0/ô B; capture không có86.
Chỉ ca idle room; chưa host abrupt disconnect, active-match recovery, reconnect hoặc cleanup mọi peer/timer. CP035 đo rejoin cùng socket riêng dưới đây.

**Điểm chặn quay về sảnh:** controls `guest-return`/`host-return` vẫn loading sau refresh241/91.
Ca `guest-route` đo trực tiếp hai frame seq73/76 được framer chấp nhận: `115BA99` return0, `125DD40` app+F0
trỏ vtable1716740/**CLobbySquareStage**, vfCC=82A530; vì vậy không vào CLobbyStage/DA1F30.
Không suy CRC lỗi hoặc thiếu reply88 từ ảnh loading/không có log handler sảnh.

Factory`125D497` mặc định loại1; khi `13AD5B0` trả index−1/−2 thì không đọc bảng.
Khi index0, getter`13A1CB0` đọc secure value tại manager+D4+E4×index.
Reader`13A1FB0` đặt rowbase manager+14+E4×index; do đó field là **row+C0, u32LE cuối
bản ghi channel14** (reader`13A21DD→13A2250`), getter dùng byte thấp.
`125D4E5`: loại0 tạo Square; loại1 tạo **CLobbyStage**/vt1766A2C/vfCC=DA1F30.
Loại2 có nhánh riêng, chưa đo. Lần vào đầu dùng default1 lúc index−1 không bảo đảm lần quay về chọn cùng loại.

Ca sửa chỉ u32 cuối0→1 trong channel14, giữ các field cũ: guest/host ở hai run riêng
đều đo getter1→factory1→normal ctor→framer0→handler241/91; ảnh leaver hiện sảnh, hết loading.
Refresh gửi sau actual86 khoảng1s. Đây là recipe tương thích đã đo; chưa chứng minh 241/91 là tối thiểu
hoặc bắt buộc trong mọi cache path. Initial candidate đổi u8 thứ ba không đổi getter0;
rowbase+8 từng đoán sai được sửa thành+14, giữ nguyên run thất bại. Full-row emulation thiếu context
bị lỗi, không tính là proof; native blob oracle và runtime getter/factory là các bằng chứng riêng.

Creator/policy của107/110/normal-lobby fixture và thời gian refresh là **emu_choice/design**.
Không phục hồi authority server gốc hoặc đóng K4/K5/K6a; rejoin/start sau promotion được bổ sung ở CP035, còn disconnect trong trận,
collision/rescue/generation/toàn reset/full auth/cache/codec/LAN/Internet vẫn mở.
[Bước tiếp duy nhất](../RESEARCH.md#q-023).

<a id="back86-membership"></a>
### Back86 và tạo lại phòng sau BEOF — CP076

[Control/failed-fix/fix/raw/hash và recipe](../evidence/back86-membership/README.md), **kr_runtime/confirmed
hẹp** trên copy Wine/currentprofile. Reuse CP034/035 wire/stage/sequence và CP074 preclockcleanup;
VN lobby/session chỉ cho phương pháp, không opcode/layout. Không nativebody/patch/forcing mới.

Control service cũ: Back86empty→CLobbyStage/UI nhưng Create không81. Chỉ thêm knownrefresh241/91
ở wrapper tạo được actual81 lần2; oldcreate_response_sent chặn reply86, không585. Bản sửa tách
membership khỏi channel: chỉ A một mình sau BEOF và resultrefresh, lưu marker cũ/giữ TCPidentity/
sequence/roundallocator/jobs, refresh241/91 sau1s, nhận new81→dynamic86→actual585→106/110/238/117.
Fixed-v2 dừng14s trướcmap/ảnhđen là partial; followup cùngservice chờnative map+render12s.
Native86 đọc11byte,106 own-match1,117 map; ảnh phòng mộtKRLocal/Start/map, không B106 sau recreate.

**Implementation/design:** leave variants ngoài scope log reject/không đổi membership. Không tái tạo
keyallocator hoặc reset seq khi Back. Setup reentry dùng seq tiếp tục, bỏ fixture B đã ngắt. Bản sửa
đầu TypeError do create metadataNone; giữ run failure/controllererror/service.log. Export mới chỉ thêm
sequence gốc create55/player_A58/map70 đã có nativecontract; mọi fixture bytes/hash giữ nguyên.
Preflight bảy dynamicaliases trước leave; profile cũ thiếu sequence bị từ chối trước mutation.

Control/refresh-only/finalobserved khôngerror; firstfix TypeError và fixed-v2 partial/ECONNRESET saucontrollerstop giữriêng, không ghép thành proof ổn định mọi variant.
Native81 title có cả inline15 và heap16; getter/allocator model bounded đọc đúng cursor, không tự nâng
thành heap/runtime proof. Chưa guestleave/hostleavecònpeer/activeleave/rejoinpeer/accountresume/
password/capacity/fullmembership/fullreset/fullauth/fullcodec/vanilla/K4/K5/K6a/fullmatch/LANInternet.

<a id="guest-back-rejoin"></a>
### Guest Back/rejoin cùng channel với host còn phòng — CP077

[Control/fix/native/UI/raw/hash](../evidence/guest-back-rejoin/README.md), **kr_runtime/confirmed hẹp**.
Tái dùng CP034/035 guestleave/rejoin và CP076 currentseq/preflight; không nativebody mới/patch/forcing.
Controlwrapper chỉ refresh241/91, actualB86→82lần2 nhưng service giữoldmember/rejectvariant: AghostB/
no107, Bno87/585. Fixed production idleBleave khi Ahost/mapped: archiveBmarkers/giữTCPidentityseq,
107/110 cho A, Brefresh241/91; actual82→87full7/slot1/session1→585, AgetsB106 và BgetsA106/map.
NativeAID2→0/chief1 rồi profileB lại; nativeBown-match1/map. Hai liveTCPchannelordinal4/8/IDs1,2/
slots0,1 giữ, dynamicseq tiếp tục, không accountreconnect. Export chỉ khai báo knownseq metadata của
roomlist_B/join/player_B/guest_A/guest_B;41rawfixtures unchanged, preflight trước mutation.

ActualB95/StartA104 sau rejoin→121129/errors0/99/197196153/clock1 cảhai, rosterA-B/key12345 đầu.
Đây là **mộtStart/clock**, không trận/result/return hoặc hai vòng. Membership/chief/timing/fixture là
**design** được client chấp nhận trong ca hẹp; nativehandler/slot/profile/UI có evidence riêng.
Chưa hostleavecònpeer/activeleave/accountresume/generalroom/fullauth/fullcodec/fullvalidator/
fullreset/vanilla/K4/K5/K6a/fullmatch/LANInternet. Bước tại CP077 (lịch sử) là Battle cùng run sau rejoin; CP078 bổ sung kết quả. Thứ tự hiện hành chỉ ở [PLAN](../PLAN.md#current-order).

<a id="host-normal-back"></a>
### Host Back/chuyển chủ/rejoin trên service hiện hành — CP107

[Recipe/hash/control/fix](../evidence/host-normal-back/README.md), [gap](../RESEARCH.md#gap-room-membership),
[thứ tự](../PLAN.md#current-order). **kr_runtime/confirmed hẹp** copyWine/AB/local/ExVillage01;
implementation/promotion/refresh/authfixture là **design**, không luật server gốc.

Contract native đã có ở [CP034](#room-leave)/[CP035](#room-rejoin), đợt này không REthân hàm mới.
Currentservice control thực nhậnA86empty/wire38/seq139 nhưng reject unsupportedvariant;15s trước
controllercompletion không phát107, BkhôngxóaA. Teardown sau đó có EOF→107 sau completion183,815997ms,
không gán cho Back. Run startupSIGABRT đầu chưa tớiBack giữ riêng, không chứngminh lỗi doobserver.

Opt-in `host_back_rejoin: {"mode":"idle_A_to_B"}` và exporter `--host-back-rejoin` pin5fixtureCP035:
join_A87slot0/session1, player_A_guest106/state0, slot_A_guest110/state0, guest_B_chief106/state22,
ready_A110/state4. Defaultlegacy/profile42 cũ không đổi; currentopt-in47fixture. Preflight metadata/
sequence trước membershipmutation; chỉAhostidle/Bmappedlive vàrejoinđúng sameBconnection, không
activeleave/accountresume/generalroom. Giữchannel/ID1,2/slot0,1/seq/roomowner/keyallocator/jobhistory.

Fixedrun: actualA86→B107(slot0,id1,0), nativebase6/errors0→rewind0→UIrereadcursor1; slotID1→0,
localchiefB1. Arefresh241/roomlist_B91 vào normalLobby; Aactual82→87full7→585/own106ownmatch1/
chief0, B nhận lạiA106guest0. A95/110ready4 vàB104→121129/129/errors0 mỗi view→actual99→153→
clock một lần cảhai. CurrentS2Cseq liên tục; không gửi107 cho leaver hoặc coi B vẫn guest theo slot1.
Ảnh đã xem phù hợpAReady/BStart vàmaphaiactor, không thay actualrequest/readerproof.

42boundedruntimejoins/72typednativepacketmodelcases và perroleframernegatives,157servertests gồm8new/
exporter2tests. Return-refresh dùngguestAalias khiđãpromotedrejoin được modeltest; CP107 dừngclock,
chưa nativepostresult/reset/bom/item/haiBattlecycles trên vai tròBhost. CP106hai vòngAhost giữ scope
riêng, không ghép thành nghiệmthu saupromotion. Same-stage/globaltime/wholeauthority không suy thêm.
Bước kế chỉđo gap gameplay/reset này theoPLAN; auditCP108. Fullauth/codec/validator/authority/
activeleave/accountresume/generalroom/allreset/vanilla/fullmatch/K5/K6a/LANInternet vẫn mở.


<a id="room-rejoin"></a>

CurrentManner/title idlepromotion/rejoin vàlateStart3: [currentmodecontract](#current-mode-host-rejoin),
[two-cycle/Start3scope](#current-mode-promoted-start3). Các run cũ dưới đây giữ build/recipe/lịch sử riêng.

### Rejoin và Start sau chuyển chủ — CP035

**kr_runtime/confirmed trong phạm vi copy/profile**, [hai run và hash](../evidence/room-rejoin/README.md).
Không thêm binary patch/ép state; dùng lại channel14 loại1 và các contract [leave](#room-leave),
[join/Ready](#duo-room-round), [start](load-phase.md#start-reply)/[peer](udp-peer.md#peer-duo). Fixture/policy là **emu_choice/design**.

- `guest-rejoin`: B Back→86→sảnh, click lại row phòng→actual82 lần hai; S→C87/seq79
  đọc7/7/errors0, client tự gửi585. Refresh own106/seq82, own110/85, peer106/88,
  table238/91 và117/94 đưa B về phòng với hai profile; A đã xóa slot B nhận lại106.
  Roster/ID1,2/slot0,1/session1 và channel sockets giữ nguyên; không reconnect/auth lại.
- `promoted-rejoin`: A rời, B nhận107 xóa A và110/state22 thành chủ; A về sảnh rồi
  actual82→87(slot0/session1)→585. Cấp lại106/110 cho A với state0 (khách/team0),
  profile B state22 (chủ/team1), đồng thời gửi lại profile A cho B.
  A own-match1, local-chief getter0; B local-chief1. A click Ready→actual95→110/state4
  cho cả hai; B click Start→**actual104 từ B**, không dựa giả định chủ luôn slot0.
- Stub trả121 cho cả hai từ request đó: runtime **129/129/errors0**, stage13, actual99
  mỗi phía,197/196/153 rồi **clock một lần mỗi client**. Ảnh sau clock có hai actor đúng
  own-marker; không dùng UI/clock riêng lẻ thay chuỗi input/request/reader/init này.

Dynamic S2C seq tiếp tục trên từng socket, không reset/replay template cũ nguyên wire.
Thời gian refresh/giữ ID/slot/session, trạng thái0/4/22 và cách chọn chủ là policy emulator được
client chấp nhận trong ca này; chưa allocator/ID reuse/đổi slot/password/full variant/reconnect.
Ca guest control chỉ tới phòng; ca chủ mới chỉ chứng minh rejoin→Ready→Start→clock, không phải
full match/hai trận hoặc mọi peer/entity/timer reset. Host EOF/disconnect trong trận/collision/rescue/
stale-generation/full auth/cache/codec/K5/K6a/LAN/Internet còn mở.
[Bước tiếp](../RESEARCH.md#q-023).

<a id="abort-rejoin"></a>
### Sau host EOF: client mới vào slot trống, chủ mới Start — CP040

**kr_runtime/confirmed hẹp trên diagnostic copy Wine**;
[recipe, các lượt chưa đạt, control và hash](../evidence/abort-rejoin/README.md).
Không thêm patch hoặc ép game state. Kế thừa [forfeit/return](life-results.md#round-finish),
[107/110](#room-leave), [Join/Ready](#duo-room-round) và [Start sau promotion](#room-rejoin).
Các mô tả “chưa đo sau abort” ở CP039 là giới hạn tại checkpoint đó; kết quả bổ sung dưới đây
không mở rộng chúng thành reconnect/full match.

`replacement-retry` bắt đầu A/B, IDs1/2, slots0/1, session1. Sau actual A channel EOF,
stub chọn Alose/Bwin →175 cho B →180 →107 xóa slot0/ID1 →110 đưa B slot1 thành chủ.
Chỉ sau cleanup mới khởi động **client C mới**, auth prefix cấp ID3, own slot0 qua87;
runtime `13B09E0`, `123FF10`, `8179F3` xác nhận ID3/slot0/87 đọc7/7/errors0.
106 own-match1 và profile B chief được cấp cho C; B nhận106 slot0/ID3.
C gửi82/585/95 thật;110 Ready slot0/state4 phát cho B/C.
Tên hiển thị C vẫn KRLocal theo fixture; phải dùng ID/slot/socket để phân biệt với A.

**Start từ chủ mới:** lần bấm đầu chưa có104; sau một lần bấm lại có **actual104 từ B**.
B và C đều đọc121 **129/129/errors0**, init stage13, actual99,197/196/153 rồi clock.
B giữ channel socket và UDP port qua hai lần Start; C dùng socket/port riêng.
153 vòng mới chỉ dành B/C, remote endpoint lấy từ actual99 của peer tương ứng;
không phát tiếp cho A đã EOF. S2C seq B liên tục73…121, không reset khi thay peer;
C bắt đầu stream riêng. Input C Down/B Right sau153 và ảnh trước/sau cho thấy hai view
đều thấy cả hai di chuyển. Không phục hồi toàn UDP grammar/key/reliability chỉ từ ca này.

`replacement-first` lỗi collector `NameError recordroot` trước TCP, không là lỗi client/protocol.
`replacement-fixed` đã C Ready/B chief nhưng một lần bấm chưa có104 tới hết320s;
giữ nguyên như lượt chưa đạt Start. Run mới có retry giới hạn, không giả request104 hoặc sửa
state để đi tiếp. **Nguyên nhân khác biệt lần bấm đầu vẫn unknown**: chưa quy cho focus/timing/guard.
Ảnh ngay sau gửi cleanup có thể còn roster trước callback; dùng trace clear/chief và ảnh trễ để đối chiếu.

**Guest EOF trong trận, đo riêng:** `guest-EOF-control` chỉ kill prefix B, có actual B channel EOF;
A channel vẫn mở, input Right đổi vị trí, B vẫn hiện và không có175/180/107 tự động trong cửa sổ đo.
`guest-EOF-policy` chỉ sau EOF đúng B chọn Awin/Blose →175 cho A →180 →107(slot1,ID2,0)
rồi110 host A. A đọc175 outer302/302, inner292/292/errors0 và1803/3/errors0, về stage12;
107 base6/6 rồi wrapper rewind/final1, record ID2→0. Chief A vẫn1, ảnh trễ chỉ còn A trong phòng.
Actor B trên A vẫn state0 trước/sau175 và trước180, không có actual17/19;
đây là policy bỏ cuộc, không giả deathreport. Chưa thử A Start sau ca guest EOF hoặc cả hai mất kết nối.
Các tên metadata/screenshot kế thừa `host_disconnect`/`B-after-EOF-input` không xác định vai trò;
run này ghi role B bị kill và role A được input/quan sát.

Policy forfeit, identity3, tái dùng slot0/session1, profile, timing và fixture là **emu_choice/design**
được client chấp nhận trong ca này. C không phải A reconnect/resume; chưa full auth/cache,
ID allocator hoặc cleanup mọi entity/timer/queue/generation. Run dừng ở clock/movement trận mới,
chưa bom/death/result của B/C. **Chưa nghiệm thu full match/K5/K6a/LAN/Internet**.
[Bước tiếp duy nhất](../RESEARCH.md#q-023).

<a id="service-replacement"></a>
### Client mới thay slot sau EOF trên service độc lập — CP051

[Recipe, nguồn reuse, raw/hash và hai run](../evidence/service-replacement/README.md). Tái dùng
[roster/rejoin](#room-rejoin), [B/C sau abort](bombs.md#abort-timer), [EOF](load-phase.md#service-eof), không phục hồi
lại layout hoặc áp opcode/ID VN. Policy `design`: bốn connection/vai theo thứ tự A→B→C, tối đa12;
C chỉ được nhận khi channelA đã đóng, B còn sống/chủ và vòng cũ đã refresh. IDs1/2/3 tách với
slot0/1/0: C là identity mới, không A reconnect/account resume. Profile41fixture thêm bảy alias C
đã đo từ CP035/040/044; profile34 cũ thiếu alias thì fail trước listen.

C Join82→87/session1, own106/ID3 và push106 sang B bằng sequence B đang tiếp tục; C có sequence
riêng. Nonhost C Ready95→110/slot0, host hiện hành B104 chỉ được Start khi hai peer live đã setup,
guest ready và vòng trước refresh. Begin một lần/Start, key12346 cho B/C qua153 từ actual99;
A đóng không còn trong roster hoặc destination. Timer bom vòng cũ giữ ticket gốc và bị loại
`round_closed`, không158. Broadcast170/17,19 và điều kiện report21 dùng peer hiện tại; rank theo
slot chết (B/slot1 thua), refresh giữ Bchief và C unready. Đây là policy fixture, không auth,
arbitration/rank/host election của server gốc hoặc bộ membership tổng quát.

**kr_runtime/confirmed hẹp:** `replacement-lite` có actual C Join/Ready, native localID3/ownmatch,
B104, B/C121 đọc129/129/errors0 và clock; actual B118/key12346/serial1→ID2/159/158 tới B/C,
B17/19→C170 và C21. Secure actorB state1 ở cả B/C trước175;175full302/inner292 rồi180full3
và phòng/chiefB1/C0. Một forfeit A + một vòng B/C đã chơi, không hai trận đủ gate.
`replacement-first` chỉ tới Start/clock và C pickup, không B118 hoặc result; không ghép run.
Giảm probe79–84 ở ca lite,85–101 tự bật lại khi Start; chưa có đối chứng xác định nguyên nhân
input thiếu hoặc overhead/tutorial. Chưa chạy recipe tắt toàn UDP hoặc retry muộn đã chuẩn bị.

Fullvalidator/t/distance/actorstate/forged-current-key, roster/key lifetime/UDP grammar, account
reconnect/cache/fullcodec, timeout/loadfail/sendfailure/idleEOF/reset variants và K5/K6a/LAN/Internet
còn mở. Phép kiểm còn thiếu ở [RESEARCH](../RESEARCH.md#gap-disconnect-replacement); thứ tự chỉ ở PLAN.


<a id="setup-eof"></a>
### Guest rời lúc setup trước map/Start — CP052

[Control/raw/hash/nguồn reuse](../evidence/setup-eof/README.md); schema/consumer107/110 vẫn ở
[room leave](#room-leave), không thêm field wire hoặc phục hồi lại receiver. Policy `design`:
Join/create được chấp nhận (`create_response_sent`) xác lập membership độc lập với tiến độ nhận map.
Khi channel đóng trong phòng chưa active, cleanup theo marker đó; socket lobby/chưa Join không bị
coi là member. Đường active/đã có result vẫn theo [EOF CP050](load-phase.md#service-eof). Closed event bị bỏ khỏi
setup/peers; close lặp không gửi lại, host104 không Start với peer đã đóng. Ba regression kiểm thêm
before-own106/lobby-only/duplicate/tick, là kiểm code policy, không tự là runtime KR.

**kr_runtime/confirmed hẹp:** hai run cùng recipe, B actual82/own106 rồi prefixB đóng trước slot/map;
A đã setup và có B trong roster. `setup-baseline` không107, native A chưa clear, ảnh sauEOF còn B.
`setup-fixed` phát một107(slot1,id2,0) và110 cho A; native107 base6/6/errors0, rewind0/final1/6,
ID2→0 một lần, chiefA giữ1 và ảnh ô B biến mất. Không native Start121/clock hoặc175/180 ở cả hai;
không197/196/153 giả, không tiếp setup tới B đóng. UI attempt Start chỉ là input thử, không actual104.
Đây là dọn membership trước trận, không forfeit/death hoặc trận hoàn chỉnh. GDB dùng probe noUDP
đã sửa autoenable CP051; không suy điều đó ảnh hưởng gameplay từ ca không có trận.

Phạm vi còn thiếu rõ: host rời khi survivor B chưa map. `host-setup-gap.json` chạy code với
socketpair/fake sender thấy cleanup không recipient và tick sau vẫn chọn alias guest_B; đây là gap
`design/confirmed`, **chưa native runtime KR**. Không coi việc sửa guest đã giải mọi setup race,
preannounced fixture/general membership, sendfailure/timeout/loadfail/account reconnect/fullcodec/
fullvalidator/reset hoặc gate. Phép kiểm tối thiểu ở [RESEARCH](../RESEARCH.md#gap-disconnect-replacement).


<a id="host-setup"></a>
### Host EOF khi survivor chưa map — CP053

[Control/fix/recipe/hash](../evidence/host-setup/README.md), tái dùng [107/110](#room-leave) và
[guest setup EOF](#setup-eof). CP052 bench nay có runtime đúng boundary, không tìm lại consumer.
Policy `design`: chọn host mới từ live channel đã Join/create, độc lập với map progress. Member chưa
map nhận pending departure riêng; sau slot/guest/table/map ban đầu mới gửi107/110 qua sequence động.
Nhờ thứ tự này fixture guest cũ không ghi đè cleanup hoặc đảo sequence61/64/67/70→73/76. Recipient
đóng thì bỏ pending, tick không gửi tới closedpeer; người ở lobby/chưa Join không được bầu chủ.

**kr_runtime/confirmed hẹp:** A đã map, B actual82/own106 rồi AEOF trước Bslot/map. Control giữ ôA,
chiefB0, không107; bấm nút B phát Ready95/110 (không phải Start). Fixed chờ Bmap rồi một107(slot0,
id1,0)/seq73 và110/seq76: native base6/rewind/final1/errors0, ID1→0 và chiefB0→1; UI chỉ B/Start.
Hai run chưa121/clock/175/180, không phase197/196/153 hoặc gửi tới A đã đóng. Run đầu bị ngắt trước
B được giữ incomplete, không là protocol failure/control. Không thêm patch binary/force state.

Đây là cleanup trước trận, không account reconnect hoặc general membership/host election gốc.
C mới sau ca này vẫn bị admission đòi round.refresh dù chưa có vòng nào: gap code cần kiểm kế tiếp,
không coi B có nút Start là service đã nhận C/Start hoặc đã chơi được. Full auth/validator/cache/
codec/reset/timeout/sendfailure và K5/K6a/LAN/Internet giữ mở; [phép đo](../RESEARCH.md#gap-disconnect-replacement).


<a id="prestart-replacement"></a>
### C mới vào trước vòng đầu sau setup EOF — CP054

[Control/fix/source/raw/hash](../evidence/prestart-replacement/README.md); dùng lại [CP053 cleanup](#host-setup)
và [CP051 C/B Start](#service-replacement), không thêm wire/schema hoặc port ID VN. Policy `design`:
post-round refresh path giữ; trước vòng1 chỉ nhận C khi Achannel đóng, B live/mapped/currenthost,
không pending departure, room idle và service đã gửi xong107/110 cho A (`departures_sent`). Marker
là hoàn tất send của service, không ACK/native consume. Reject accept ghi reason/peer/ordinal/time,
không assign identity/bootstrap; cap12 giữ. Không mở admission cho active round/lobby/pending setup.

**kr_runtime/confirmed hẹp:** control `prestart-denied` sau B clear/chief vẫn từ chối actual TCP
ordinal9 `replacement_not_ready`; C chưa được auth/ID3, không round. Fixed cùng binary/profile/recipe
có12connection; native C ID3/slot0/ownmatch, actual82/95 và B104. B/C121full129/clock, generation1/
key12345, actual B118→ID1/159/158→B17/19→C170/C21; secure Bstate1 cả hai trước175,175full302/
inner292→180full3/phòng. Không A-B round/bom/forfeit trước đó, không packet tới A sauEOF. Một vòng
B/C đã chơi, không hai trận đủ gate. Auth/roster/rank/fuse vẫn fixture/policy; C3 không A1resume.

Native replay144packet/6vector153/210secure snapshots, source/heap/counter limits ở evidence.
Adapter153 ban đầu giả mọi channel có dynamic_s2c; A chưa vào round không có list nên KeyError,
đã sửa vắnglist→empty và rerun oracle, không sửa raw/giả pass. Ba policy regression cho
idle-before-first/setup/cleanup/active và first-key/roster/deny transport. Fullvalidator/t/actorstate/
distance/auth/cache/codec/membership/reset/vanilla/K5/K6a/LAN/Internet giữ mở; thứ tự chỉ ở PLAN.

<a id="rejoin-battle-cycles"></a>
### Battle sau guest rejoin: một vòng confirmed, hai vòng partial — CP078

[Raw/control/observer/helpercorrection/hash](../evidence/rejoin-battle-cycles/README.md),
**kr_runtime/confirmed hẹp** cho firstR1, acceptancehai vòng **unknown/partial**. ReuseCP077rejoin và
CP069basicUIroute, productionservice/profile giữ; không nativebody/patch/forcing/semanticreplay mới.
Firstsame-runBBack/82/87/585/roster→Ready95/Start104/clock→actual17Roller/27Bomb+2bomID1,2/power1,
paired165/statbothviews→17/19/B170/stateA1trước175full302→180full3/refresh. NoFire28/hiddenpointerjoin.
Saurefresh actualA585seq346/B585seq304(u32body0), oldplayer_response_sent khiếnservicekhông106mới;
khôngactualB95/A104, chưa chứng minh thiếu106 làcause. NativeownchiefA1/B0/110error0 đã thấy.

Followup intendedfocus/renderReady2 chưa được đo: controllerobserver100s từJoin dừng1,5065s sau
actual99cảhai, trướcminimum1971961532s, khôngnativeclock; noerror/EOF saucontrolledstop. Không gọi
là153bịreject/clientfailure. RawUIhelperB-labeledDISPLAY0image thực tếAhost (Winebootenvpicked),
loại khỏiBproof; giữfailedhelper/setup/correction. Không suy request từOSclick hoặc ownership từpath.
Plannedtwo-cycleassertfail giữ, boundedfirstpass không thay. Nativeframer/getter/heap làmodels, callback/
state thực cónguồn riêng. Chưa hairesultreturn/fullreset/fullmatch/K5/K6a/vanilla/LANInternet/auth/
codec/validator/hostlivepeer/activeleave/accountresume. Target tại CP078 (lịch sử) là 585/Ready; CP079 bổ sung counterexample. Thứ tự chỉ ở [PLAN](../PLAN.md#current-order). Service/rank policy không sửa trong CP078.

<a id="normal-battle-exit"></a>
### Normal guest Battle exit202→86, lobby và survivor return — CP125

**kr_runtime/kr_static confirmed trong scope; emulatorpolicy design**, [raw/recipe/hash](../evidence/battle-exit/README.md).
Source-first CP034 idleBack/CP044activeEOF/CP107idlehostBack/VNmembership concepts chỉ phần tái dùng;
không gọi chúng là proof normalBattleexit. Hai realWineAB/Ahost/Bguest/ExVillage01/currentterrain,
không đặt bom. Nút 나가기 client-relative715,578, không có confirmmodal trong hai ca đo.

Control182,674759821s: Bactual202seq202/payload22, sau đó86seq205/bodyempty, native13→11 qua
callerE724F0; service86reject`round_not_returned`. Bviewlobbyempty, A vẫnBattle/no175180 trong30s.
AEOF do cleanup **247,169634ms sau controllercompletion**, không gán cho click. Source/capture giữ.

Targetedstatic2newbodies **E72490/E84FF0** byte-match static/runtimeA/B/modO. E72490 gọiE84FF0,
123F820(1), normal12562F0(11)→1256100, không chờ reply202. E84FF0 guardstage+6E08, constructor
1164C10(op202), u8stage+6C58/u8min(getter13B07C0,2), mỗi record5u32/14EBD90, send1164D50.
Ca đo [0,1,0,0,0,0,0]/22byte; nghĩa cácu32 opaque. RawfirstsliceE72380 mởgiữainstruction không
được dùng; alignedfullbody/code-match giữ riêng. Không newnativebody khác hoặc sửa sourceDB/binary.
**202 khôngexitexclusive:** retainedCP106/121/123 có12example normalending; fixedA gửi202 từE60F5F
trước180, Bexit sendstackE724A1/E85241. Không dùng202mộtmình như leave request/death/authority.

`battle_exit: {"mode":"guest_B_202_86"}` opt-in/defaultoff. Chỉ Ahost/Bguest/currentbound/mapped/
live2real/unbombed, exclude syntheticrescueprofile. Stage valid202variant22/context0/count1/fiveopaqueu32,
không reply/finish/death. Subsequent86empty seqforward cùnggeneration mới releaseBmembership giữTCP,
finishfence/abort reasonnormal_Battle_exit. Validate reentryfixtures trước mutate; reject default/unsupported
không đổi member/fence. Chosenforfeit12s/rankA1B2→1805s→postreturn1071103s và B24191sau1s tái dùng
policy cũ, không luật Nexon hoặc body202đủ chứng minh chết. Không send107prepare-only lúcAstillBattle.

Fixed228,339129183s/exit0/error0: B202seq205→86seq208 được nhận, native11/24191/room91đủ50;
A175đủ302/native manager ranks1,2→180đủ3/stage12→107arrival/110đủ6, ảnh room chỉA/chief và Bởsảnh
córoomrowWAITING. KhôngEOF trướcboundcompletion, khôngactual/synthetic17/19; identity/channel khôngđóng.
Runtime107basecursor/IDfield không probe riêng ở đây; copiednative6byteparser+Preparearrival/UIslotempty
khác mức chứng minh đó. 929joins/820raw+84nativepacketcases; control877joins/790raw+69nativecases.
211server tests gồm9new regression; noBomb/role/context/count/reset/pair/duplicate/default/preflight.
Chưa rejoin/Start2 sauexit/activehost/postbomb/deadexit/generalroom/auth/fullmatch/K5/K6a/LANInternet.
Bước kế duy nhất PLAN; AUD039qua124/kế128, completehistory/raw cũ không sửa.


<a id="battle-exit-rejoin"></a>
### Same-channel rejoin/Ready/Start2 sau normal B Battle exit — CP126

**kr_runtime/confirmed hẹp**, [recipe/hash](../evidence/battle-exit/README.md#cp126).
Giữ CP125 service/profile/client/probes/fuse, chỉ thêm normalBdoubleclickroom sauAactual180/cleanup và
B24191→rejoin117+8s→ReadyB→AStart. Không mint82/95/104 hoặc reconnectaccount. Run243,779083202s/
exit0/error0: actualB202→86/currentoption/forfeitA175180/room, cùng8TCP connections/ID1,2/slot0,1.
Saucleanup: B82seq268→585seq283→95seq301, A104seq250 (seq từng socket, không so số giữa hai vai).
Bnative87 đủ7 lần3/ownslot1, actualmemberrecreateSHA khớp82, native121 đủ129 cảhai lần/key12345→
12346/actual99/peer153/clocks hai view; ảnh R2active. Chỉ mộtchosenresult/return trướcStart2.

1379joins/1095raw,135nativepacket/12peer153 copied-native cases riêng. Verifierđầu gomframes theo
roleA/B rồi đòi chronologicalorder nên fail; bản riêng sort theo capturemonotonic_ns, giữ nguyên
assertion/raw/failedscript. Actual82 tới127,662ms trướcOSdoubleclickend, trong actionwindow, không
làinputminted. Không dùng arrivalorder thay globalgametick. 211servertestsCP125 reuse/sourceunchanged.
Chưa Battle2completedresult/exitafterBomb/hostexit/deadexit/earlyjoinbarrier/accountresume/genericreset/
auth/fullmatch/K5/K6a/vanilla/LANInternet. Bước kế chỉ PLAN; AUD039/kế128.


<a id="postbomb-exit-rejoin"></a>
### Pending B bomb/normal exit/rejoin/Start2 cùng run — CP128

**kr_runtime/confirmed trong scope copy Wine; policy vẫn emu_choice/design**, [metadata/hash](../evidence/battle-exit/README.md#cp128).
Tái dùng nguyên service/profile pending-mode CP127 và recipe rejoin CP126; không thêm native function,
opcode hoặc sửa binary. Trong242,080503262s/exit0/errors0, actual B118/native159 đủ39 hai view →
normal B202/86/admit → finishfence/oldjob `round_closed`/no158/no terraincommit → A175302/1803/
postreturn cleanup và B241/91 → normal B82seq274/585seq289/95seq307 → A104seq250.
Seq chỉ so trên cùng socket; cùng8TCP connections/ID1,2/slot0,1, key12345→12346, hai native121 đủ129,
actual99/153 và hai clock trên mỗi client. `battle_exit202` đã được xóa trước Start2.

1385joins/1091raw,144copied-native packet và12peer153 cases tách khỏi runtime. Job bom R1 được giữ
tới deadline rồi reject; toàn capture không158, chỉ một native159/view trong R1. Ảnh R2 vẽ hai actor/
clock và không còn bom B tại ô9,3; **không probe native bomb pool/transport queue**, nên không kết luận
toàn bộ entity/UDP reset đã sạch. Giữ source/profile/fuse6 tới cleanup, không mint request/call/kill.
Chỉ một chosen forfeit/result/return rồi R2active, không hai completed matches/accountresume/fullmatch/
K5/K6a/vanilla/LANInternet. Host/dead/multiple/alreadyblast/earlyjoin/generalreset/authority vẫn thiếu.
Phạm vi lịch sử CP126/127 giữ nguyên; gap postbombexit+rejoin trong scope này đã giải ở CP128.
Bước tiếp chỉ PLAN; AUD040 qua128/kế132, review độc lập waived, gate giữ mở.


<a id="host-battle-exit"></a>
### Host A normal Battle exit/B promotion/Arejoin/Start2 — CP129

**kr_runtime/confirmed hẹp; emu_choice/design cho admission/result/cleanup**, [recipe/hash](../evidence/battle-exit/README.md#cp129).
Tái dùng CP125 E72490/E84FF0/20286, CP107 idlehost promotion/guestA fixtures và CP126–128 normalrejoin;
không thêm thân hàm native hoặc port số VN. Control199,685678163s: actual host A202seq190/22byte →
86seq193/empty/native11/E724F0, currentguestmode reject202membership/86round_not_returned; B không175/
180 trong45s, A sảnh trống/B còn Battle, noEOF trước completion. Có hai subtype39 tự phát, semantics
chưa gán; không gọi control là không có report. 202 vẫn không phải gói chỉ riêng quit: B gửi từ ending.

Opt-in `battle_exit: {"mode":"host_A_202_86"}` yêu cầu known `host_back_rejoin.mode=idle_A_to_B`/
5fixture guestA và preflight trước release. Chỉ Ahost/currentmappedlive2realAB/noBomb/noaccepted17,19;
202 context0/count1/fiveu32 opaque →86empty/forwardseq/samegeneration, kiểm lại scope tại86.
Release Amembership, giữ TCP, finishfence/abortA; reuse chosenforfeit Bwin175 sau12s/180 sau5s/
postreturn107+110chief sau3s, không gửi107 khi B còn Battle, không giả19 hoặc thêm202reply.
A nhận241/91(roomlistB) sau1s. Promotion tại cleanup sau Breturn; earlyAjoin trước Bchief bị reject,
sau cleanup reuse known CP10782/585/guestA95/B104. Oldguest/default giữ nguyên; mode mới chỉAhost,
không mode gộp mọi leave, hostpending/dead/multiple/synthetic/authority đều ngoài scope.

Fixed205,688539272s/noerror: actualA20286/admit/native11/A24191/full50/sảnh KRoomWAITING;
B175302/ranks2,1/1803/stage12/post107 slot0/id1→0, basecursor6/rewind0/UIreread1 khônglỗi,
nativechief1/full1106, UIphòng chỉB. NoEOF trướccompletion/noBomb/no17,18,19. Source/profile thêm
namedmode+5knownfixtures và readonly107/chiefprobes, nêu rõ nhiều thay đổi, không single-variableclaim.

Rejoin256,568428149s cùngsource/profile/probes: Aexit/B175180/107clear-chief → actualA82/585/95/B104,
cùng8TCP, ID1,2/slot0,1/key12345→12346, native121129/clocks2 mỗi view/actual99/153; marker202reset.
A87full7/ownmatch1/chief0, R2hostB. Tổng3run2936raw/3263joins, 288copied-native packet+12peer153
cases riêng, 220servertests/5new. Chỉ mộtforfeit rồiR2active, chưa2completedresults/fullmatch/K5/K6a/
Windowsvanilla/LANInternet/accountresume. Không đo UDPqueue/nativepool reset trong cácrun này.
Verifierfailures giảđịnh noeventreports và statehost_role khôngtồn tại được giữ; sửa riêng theo no17/18/19
và nativechief/currentR2host, không sửa raw hoặc bịa semantics39. Bước tiếp chỉPLAN, audit kếCP132.


<a id="current-mode-host-rejoin"></a>
### Current Manner/title: host Back, chuyển chủ và vào lại — CP166

**`kr_runtime/confirmed` hẹp trên hai copy Wine/private virtual hosts; service policy `design`.**
[Control/fix/raw/hash](../evidence/peer-endpoints/README.md#cp166), [câu hỏi](../RESEARCH.md#gap-room-membership).
Reuse CP107 idleA86/107clear/chief1/A82/87slot0session1/ReadyA/StartB, CP129/157 lifecycle, CP161 title,
CP162 selector86/87/180 và CP165 currentpreset. VN lobby/oc_a-room chỉ workflow/host-vs-slot;
KRlegacy không thêm contract mode. Không nativebody/offset/codec RE mới hoặc port opcode VN.

Control currentpreset: actualAhostBack86empty được chấp nhận, Bnative107ID1→0/chief1, Ađọc lobby91
đủ60byte/title; actualA82/seq169 bị `room_team_mode` từchối vì aliasjoin_A ngoài create/join.
KhôngACK87 choA/khôngclock trướccompletion. Đây là lỗi policy emulator đã tái hiện, không lỗi native.

Fix giữ existing promotedmembership/sameBconnection/live/map/refresh/opcode82 guards; chỉ cho join_A
echo currentselector giống join87. Combinedhost+mode profile preflight opcode87/success/full7byte trước
owner mutation. Bodyjoin_A nativefixture SHA79b35b72…/slot0/session1: chỉ offset5 đổi selector0→1;
identity/opaque bytes/sequence không đổi. C/generalrecreate/password/Random vẫn ngoài scope; không
lấy row91 title làm proof mode metadata. Không sửa gamebinary/pack/DB/actor/time hoặc gọi gamefunction.

Fixedrun164.328823171s/exit0: actualManner81→A86/B87 selector1; actualA86→B107/110chief1→A91title
đủ56byte→actualA82/currentseq87/full7/error0→585/ownmatch1/chief0→117→A95/B104→121full129/
actual99/153/clock1mỗi view vàrender+8s. Hai title tựsinh khácđộdài giữa run không so byte ngangrun;
trong từngrun refresh91 echo đúng actualcreate81 và giữ opaque rowtail. Nativeobserved modegetter1
cảhai tới completion; không suy mọi field hoặc wholeexecution. Nocurrentbomb/pickup/trap/death/result;
hai actualstartup subtype39 vẫn giữ, không xóa report để gọi noevent. ACK/body/getter emulation riêng.

514control/815fixed quan hệ gồm rawhash,9/21 copiednativeframer/reader valid/CRC/partial cases;
265server/4exporttests gồm negatives wronghost/staleconnection/refresh/create/no-selector/sendfailure/
preflightshape/error/opcode/continuingseq/opaque preservation. 418+668rawsealed/cleanupnamespaceextinct;
initialverifier giả định eventreportsrỗng bịsửa côngkhai, không sửa raw/productioncho pass.

TạiCP166 chưa đo hai trận/reset sau promotion trên currentmode; [CP167](#current-mode-promoted-cycles) đo sauđó. CP109oldprofile/CP165Ahost vẫn không ghép thành proof. [K4 acceptance/dependencies](../PLAN.md#k4-evidence-review), K6a/fullmatch/Windows/LAN/
Internet/8client/fullauth/originalauthority/generalreset vẫn mở. Bước tiếp chỉ ở PLAN.


<a id="current-mode-promoted-cycles"></a>
### Hai normal cycles/reset/Ready sau promotion trên currentmode — CP167

**Đính chính AUD050/CP168:** recordCP167 dùng nhầm tên Roller cho missingindex16. Actual124/165
là kind0/Bomb/capacityeffect đãphục hồi ởCP031; khôngkind2/Roller. Chỉ nhãnprose sai, raw/kind/hash/
missingcallback và cácverification khôngđổi. RecordCP167 bấtbiến giữlỗi đểtruynguồn.

**`kr_runtime/confirmed` hẹp, modifiedWine/privatevirtualhosts; policy vẫn `design`.**
[Raw/control/hash](../evidence/peer-endpoints/README.md#cp167), [gap](../RESEARCH.md#gap-room-membership).
Sourcefirst CP109/110 Bhost/Aguestcycles/UI vàCP111receiverprepost, CP165currentpreset/observergating,
CP166mode/title/membership. VN/legacybuildreferences dẫn lại reuse-question đãpin; không portopcode/
ID/offset hoặc đọc nativebody mới. Ghép đúng phần thiếu, giữ profileexport47pins/directproduction/
2actualcopies/currentManner/title/nohold165/noreplay. Chỉ OSReadyA/StartB thay role; actor slot/geometry/
inputtargets/tolerance/timeout/nativecriteria giữ CP165. R1/R2 là run riêng, không cộng thành4trận.

R1 378.305974972s/exit0 có hai native175302/1803/errors0/reset/actualReadyA/native110 cảhai, Bhost/
sameTCP/key12345→12346, mode1 quaACK/2×180/StartB; movement/life/entity195 có nguồn. Tuy nhiên
full4native165/view check **FAIL**: chỉ3callback/view; Bomb16/kind0/R1serviceconfirm broadcast trongfirstalignment
window ởcảhai khi observer bịtắt. Không dùng servicebroadcast chứngminh client đãchạymissingcallback;
Fire165cảhaivòng và snapshotstat có nguồn riêng. R1giữ nguyên raw/fail/partialcoverage, không backfill.

R2 chỉ đổi measurementgating: giữ enabledstate vốn có của ba rareprobe165 `E63260/E63309/E63E1A`
(begin/fields/end+readonlystats) trongalignment, còn lại vẫn giảm tải vàexactrestore. 24actualarms/2view
đều xácnhận3probeenabled; 12alignmentwindows/162restore+samecriteriachecks pass. Không thêm
nativefunction/data/timewrite, service/preset/sourcebinary/DB/packchange hoặc tăngtolerance/timeout.

Cùng R2 369.832789235s/exit0: actualA86→B107clear/chief1→A82/87Manner1→ReadyA/StartB hai lần;
hai barrier trước30s, key12345/12346; actualbomID1/A vàID2/B/158/full13 mỗi view, native175302/1803
hai lần và actualReadyA/native110 sau return2. Cảhai view có đủ4native165/full12/errors0; chỉwinnerFire
powerdelta/reset theo nativegetter, không doubleeffect; R1FirewinnerA rồiB, R2winnerB rồiB là observation,
không luật ưu tiên. Native195seed/reset/2entities/lifeagreement trước chosenrank/4peerwindows được kiểm
riêng; emulatorterrain/fuse/claims/report/rankguard không là originalworldauthority.

FinalR1ảnh cảhai phòng/AReady/BStartcolored; FinalR2 phòng/AReady/Bchief cóảnh, **BStart trắng1frame**.
Không suy persistenthang, không lấy R1 backfill R2 hoặc gọi BStart sauReady2 đã responsive; chưa click
Start3 tronghai run. [PLAN](../PLAN.md#current-order) kiểm actualBStart3/nativeclock/render+8 vàauditCP168.
Hai cycles/Ready nativeproof vẫn giữ đúngscope, không tự nâng K4/K5/K6a/fullmatch hoặc LAN/Internet/
8client/Windows/generalreset/fullauth/originalauthority. AUD049qua164, periodicnext168 không reset.


<a id="current-mode-promoted-start3"></a>
### Currentmode BStart3 sau hai normalcycles và audit — CP168

**`kr_runtime/confirmed` hẹp; controller input/placement/servicepolicy `design`.**
[Control/raw/hash](../evidence/peer-endpoints/README.md#cp168). Reuse CP150 stop-on-actualpickup principle,
CP165 Start3/observercontrol, CP166membership vàCP167currentmodeBhost2cycles/rare165. Không nativebody/
codec/item RE mới, không portopcode/ID/layoutVN hoặc đổi sourcebinary/pack/DB/service/preset.

Hai run controlR1/R2 fixed0,30s keys FAIL `ProductionPickup17notobservedR1`: firstnativeclock/mode1/
promotion đãtới, khôngFire124/bom/outcome/Ready2/Start3. R2thêmmeasurementwindow giảmhotBPs/keep3rare165,
nativefeedback16samples/view tronginput: ownA179→239, ownB387→343 (timestamp khácnhau); 192copied
nativegettercases xácnhận capturedwrappers. Không gọi làglobalendpositions/pxs hoặc nguyênnhânmọifailure.
Firecenter279 theo knownrow6; control chỉ cho thấy actualmovement vàmissingclaim, khôngservicebug.

R3thay fixedpulse bằngnormalOSkeyhold dừng theoactualFire124 hoặc5s cap, giữtarget/tolerance/20attempt/
3freshneutral/30sloading/claimuniqueness/native165/peerlifecycles. Đây là controllerdesign mới theofeedback,
**không** giữphím0,30s rồi gọioldcontrolpass hoặc nới oracle. Actualholdduration0,617006739s/0,595376959s;
mỗivòng1actualFireclaim/winnerA, native4callbacks165/view/full12/errors0/currentpower/reset cónguồn.
Không chứng minh2simultaneousnativecontenders/fairness từ1claim. Haiadditionalcontestmeasurementwindows
giữrare165/stat probes, disable writer/chronology khác rồiexactrestore trước4peerwindows vàbombs;
28actualarms/14groups, 12alignmentgroups/162checks và2contestgroups/58checks táchscope đầyđủ.

Run405.945854372s/exit0/currentexport47pins/directmodule/private2hosts: hai normal175302/1803/reset/
actualReadyA rồi normalB104/third121full129/actual99/153/barrier/nativeclock3 mỗi view vàrender8s;
Manner1 qua87/2×180/StartB. Thirdactive/key12347/noBomb/noaccepted17,19, khôngthirdnormaloutcome.
Ảnh BStartcolored trướcclick vàmap/timer1:50/twoactors+8s ởhai view đãxem. Newrun lateUIfunctional
cóproof; không retrospectiveR2whiteframecause hoặc proof mọi UI/timing/Windows/LAN/Internet.

AUD050 đốichiếu core→evidence→raw CP165–168, sửa navigation/pendingPR/knownpromotion/reset cũ,
giữ recordcomplete vàcontrols bấtbiến; auditnextCP172. Rà gate phát hiện **currentpresetfloor-only**:
khônghidden_items, revealed_hidden={} cả3generation. NativeBox/reveal142/hiddenpickup đãRE/đo tại
[CP068/069](life-results.md#basic-battle) ởprofilekhác, chưacurrentintegration; không coi thiếu là phảiRElạitoànitem.
Đây lànextgap cụthể phục vụK5; [PLAN](../PLAN.md#current-order) composeknown582/catalog/route/nativepreflight.
K4dependencies/K5/K6a/fullmatch/8client/generalreset/Windows/coldcache/codec-reset/originalauthority
giữmở. R3khôngactiveoldUDP/EOFfault, controlsCP155/167 khôngbackfillsame-runqualification.


<a id="replacement-profile"></a>
### Preset thay người và capacity theo slot trống — CP177

[Nguồn reuse, recipe/raw/hash/control](../evidence/peer-endpoints/README.md#cp177). Tái dùng
[CP051 Cslot0](#service-replacement), [CP175 binding/guard](networks.md#a-survivor-admission),
[CP176 Cslot1 gameplay](life-results.md#a-c-basic-cycles) và known121/118/159/158; không thêm wire/schema/ID VN
hoặc tìm lại consumer. `two-client-basic-replacement` là preset **tùy chọn**, chỉ thêm
`replacement_slot={mode:vacant_AB_slot,capacity:vacated_slot}` vào `two-client-basic`, giữ đúng47
fixture pin và mọi policy khác. Default hai preset cũ và flag `mode` đơn giữ behavior cũ/explicit C.

`design/confirmed` cho implementation: derived policy bắt buộc chỉ A/B `bomb_capacity`, từ chối C
khai báo cùng lúc, thiếu role hoặc policy không hỗ trợ. Ordinal9 sau cleanup/membership/host/map/
round fence hiện hành bind Cidentity3 vào slot của vai rời, sao **riêng** initial/maximum từ vai ấy
vào service instance và log `replacement_binding.capacity.source_role/initial/maximum/source`.
Không sửa profile input, A/B limits, global slot hoặc instance khác. Cslot0 lấy A1/10; Cslot1 lấy
B2/7. Terrain preflight cho A/B trong derived mode; old explicit policy vẫn cần C khi có capacity.
Maximum10/7 là lựa chọn operator; copy limits không tự chứng minh native maxima/exhaustion.

`kr_runtime/confirmed` hẹp: hai run riêng dùng đúng exported preset, copied Wine/private virtual
hosts/cùng physical host. C1: Bguest EOF trước trận đầu, Ahost/C3-slot1, nativeinitial2/power1/speed4;
C0: Ahost EOF trước trận đầu, Bpromotion/C3-slot0, nativeinitial1/power1/speed5. C từng run có actual
Join/Ready, currenthost Start1/full121/99/barrier/clock, C118 key12345/boundowner → ID1/power1,
native159/entity và158 xử lý cùngID1 một lần trên cả survivor/C. Mỗi run có48 quan hệ kiểm ngoài
raw/preflight; native model packet cases33/42 tách khỏi runtime. Không gộp thành hai vòng cùng run.

C0r2 dừng ở Bstartup SIGSEGV trongjypc.dll,6transports/trước C; rawseal/cleanup sạch, không là probe
thành công. C0r3 cùng cấu hình/fresh copies đi hết phép thử, **không** chứng minh đã sửa nguyên nhân
startup hoặc bác bỏ ca lỗi. C0r1 chỉ preparation assertion, chưa launch. Một số nhãn mô tả/ảnh của
harness C0 còn wording cũ Cslot1/A-survivor; code, ownership, namespace, host và verifier field/slot
đã đối chiếu tường minh. Không dùng nhãn đó làm proof cho sai vai.

284server tests/7exporter tests và actual pinned preflight kiểm policy; đó không là native evidence.
Không Ccapacity exhaustion/hai reservation/survival/peer movement/result/reset mới; CP176 giữ hai
AC normal cycles riêng. Không account resume/fullauth/general lifecycle/originalWindows/LAN/Internet/
8client/fullmatch hoặc tự đóng gate. [PLAN](../PLAN.md#current-order) tiếp acceptance/control thực tế.


<a id="rust-r1c-player-slots"></a>
## R1c — identity, slot và một template 106 — CP263

Nguồn **design/design**, kiểm source/unit/TCP, không thêm kết luận native KR. [Metadata/hash](../evidence/rust-service/r1c-player-slots.json),
[thứ tự/acceptance cuối R1](../PLAN.md#rust-r1). R1b đã merge PR337 tại baseline của mốc này.
Reuse layout KR106/string/profile đã đo tại [room-player](#room-player), Python `four_player_profile.py`
và các fixture CP018/020/026/199 đã có. 30 body trong năm config/14 pin khác nhau đều khớp phép biến đổi các field đã biết;
không có chênh lệch unknown cần RE thêm. Python là mẫu phương pháp; phép so input fixture không tự là native proof mới.

`PlayerId` là alias `u32` của identity backend/wire; **không phải newtype kiểm kiểu**. Membership vẫn gồm identity,
connection và visit. `RoomRules::from_config` chọn `SlotAllocation`: mặc định `legacy_identity` giữ slot=identity−1,
creator1 và wire của profile cũ; opt-in `first_vacant` lấy slot trống nhỏ nhất và host là người tạo. Team theo bảng slot
đã có. Zero ID, trùng identity/connection, đầy phòng, sai phase hoặc slot không hợp lệ bị chặn trước tăng visit/publish.
Rời phòng chuyển host theo slot như trước; tái dùng slot không làm membership/visit cũ có quyền trên member mới.
Policy này là design của server; không nhận là quy tắc server KR gốc. Unit dùng ID70000/42 để kiểm tách identity/slot,
full/duplicate atomicity, host transfer, vacancy reuse và stale membership/ACK.

Ở admission, một `PlayerTemplate` lấy prefix/suffix opaque từ fixture106 đầu; nhánh profile là name +98 byte zero,
hai tên inner/outer phải cùng nhau, tên ASCII 5/7 byte theo profile đang hỗ trợ. Chỉ thay identity/tên/slot/state/đội đã biết;
opaque bytes được giữ. Body sinh ra phải bằng từng fixture cũ; header cũng bằng sau bỏ riêng trường length do codec tính lại.
Lệch bất kỳ byte khác làm startup từ chối trước bind, kể cả hash input hợp lệ. Runtime cache projection theo PlayerId;
không còn lookup alias identity để gửi106/110. Alias legacy chỉ còn adapter nạp config cũ và donor header107 đã có;
việc bỏ phụ thuộc packet/header vào fixture tiếp tục ở R1d, không mở rộng lát cắt này.

94 Rust tests/fmt/clippy/locked build và 14 suite/305 ca TCP đạt. Trong đó 298 ca cũ được so với baseline CP262 đã seal:
3.189 frame phòng khớp identity/normalized visit/alias/opcode/sequence/size/full SHA. Giữ boundary receiver và raw tail như R1b;
không dùng thứ tự log giữa recipient độc lập làm contract. Bảy ca mới gồm năm pre-bind guard với pin hợp lệ và hai kiểm TCP:
B tạo phòng/host slot0, A identity1 ở slot1; sau B rời phòng, A giữ slot1 thành host, B vào lại slot0 thành khách.
Tên/ID/state ở cả hai roster được đọc đối chiếu. Đợt runner đầu mất executable mode khi chụp binary, chưa chạy service;
log giữ nguyên, sửa copier ở đợt mới mà không đổi source/config/input của service.

Listener/launcher vẫn giới hạn các profile diagnostic đã đo và bridge account hiện có; không tự phục vụ ID/profile tùy ý,
không mặc định scalar cho inventory/accessory hoặc chứng nhận profile tổng quát M5. Transport binding và nhiều session chung NAT
vẫn là R1f. N1 uplink di động/ISP-CGNAT chưa xác nhận; không đóng gate N1 hoặc sửa K6c/K7. Binary mới chưa được nghiệm thu native;
cuối R1 vẫn cần native 2/4/8/account. Sửa tên helper ghi nhầm `admit` ở mô tả R1b thành đúng `from_config`; record/audit/raw CP262 giữ nguyên.

<a id="rust-r1e-registry"></a>
## R1e — registry phòng, listing và batch origin — CP265

Nguồn **design/design**, source/unit/TCP; [metadata/hash](../evidence/rust-service/r1e-registry.json), [dashboard](../PLAN.md#dashboard).
Reuse [CP226 listing](#room-list-lifecycle), [CP229 registry](#room-membership-registry), CP230 cleanup và R1d builders;
Python prototype chỉ mẫu policy/session-room-generation/visit/tombstone, không port nguyên service hoặc wire VN.
Không RE lại native91/87/106/107/110 đã có; ba input91 CP226 (2/1/0row) tái tạo đúng bằng builder Rust mới.
Đây là fixture equivalence, không native mới. R1d/PR343 đã merge sau CI phục hồi; record/raw CP264 giữ nguyên.

`BoundProfiles.rooms` là registry đồng bộ `RoomId→Entry<Shared>`; mỗi `Shared` giữ model/RoomRules/member/queue riêng.
Key gồm session, ID và generation tăng; trùng ID/session, zero ID/session hoặc publictuple đã dùng bị chặn trước mutation.
Không reuse cùng publictuple trong process vì request82 chưa có epoch đã đo; tombstone không phải durable restart fence.
Chọn entry rồi mở channel dưới lock registry→room, không await trong đoạn đó. Listing91 tạo từ các entry hiện hành,
đọc title trong từng room dưới cùng thứ tự lock rồi bỏ lock trước socket await. Snapshot từng room không được coi
là transaction nhất quán toàn registry khi gameplay thay đổi. Giữ22byte opaque từ template, không tự đổi occupancy/quota.

Serving hiện vẫn đăng ký một entry1/1 từ `fixed_room` và chọn đúng entry đó sau lobby67. Đây là compatibility route
để giữ bytes profile2/4/8/account; **chưa** thêm operator multi-room config hay request81/82 tổng quát. M7 sẽ sử dụng
registry/API này để tạo/chọn/chuyển phòng; kết quả unit hai entry không là native hai phòng/gameplay đã giao.
Unit kiểm hai Session/inbox thực qua registry: table238 đúng recipient, đầy queueA không chiếm/chặn queueB,
đóngA không xóaB. Unit listing kiểm2entry đúng session/ID/title/opaque, đổiB không đổiA, gỡB/A→count1/0.
Native91count0 không tự clear-all; đường delete90/dynamic occupancy khi M7 cần acceptance riêng.

Phép thử cố ý truyền batch cache cũ sang Session phòng mới cho thấy guard Channel cũ tin inbox riêng;
**không phải** packet leak qua mạng đã quan sát. R1e gắn `(roomscope,recipientID,connection)` khi reserve/publish,
`valid` kiểm origin trước các guard membership/round cũ. Roomscope là token nội bộ riêng mỗi Runtime, không wirefield.
Batch từ room đã gỡ hoặc Session khác bị chặn kể cả các scalar tình cờ giống; policy auth/session vẫn ở service.
Retirement guard được thử tuần tự với live channel/queued batch; container generic không tự quyết authority, caller phải
kiểm channel/member/jobs và lock order. Không claim concurrent select/remove hoặc raw585/99 stale phân biệt được.

Giữ failed copied-source r4 (forced oldbatch accepted), r5 (fence đạt nhưng fixture unranked cũ tạo batch chưa publish),
r6 dùng cùng fence và cho fixture đi qua publish/inbox thật:91 unit của bản sao đạt. Không nới guard để qua test.
Source tích hợp thêm listing:101 workspace tests/fmt/clippy/locked build và14suite306 TCP đạt;3169 frame cũ +24
first-vacant khớp CP264 theo boundary receiver, unread cleanup tail giữ raw. Code/binary/input frozen, owned PID đã dọn.
Cache extern rlibs của probe được hash lúc compile; executable/copy source giữ riêng, không coi target cache mutable
là archive dependency bất biến. Build/lockfile/source của run serving đã snapshot riêng để tái lập.

R1e chỉ refactor nền theo acceptance của PLAN; không đóng fullR1/M7/M5 hoặc chứng nhận binary native.
Tiếp N1b tối đa hai native runs trước R1f, rồi R1g và native cuối R1 2/4/8/account. N1 one-side scope giữ; VPS chỉ local.

<a id="rust-r1f-session-binding"></a>
## R1f — identity/profile theo session và endpoint theo channel — CP269

Nguồn **design/design**, kiểm source/unit/TCP, chưa native candidate; [metadata/hash](../evidence/rust-service/r1f-session-binding.json),
[thứ tự và native cuối R1](../PLAN.md#rust-r1), [thiết kế relay tách riêng](../RUST_SERVICE_ARCHITECTURE.md#relay-endpoint-design).
Dùng lại frontend32/body61→issuedplayer/session DB→echo/channel/lease CP253–258, metadata address CP260 và99/153 builders
CP264; không wire mới hoặc port VN. Không còn khoảng trống layout cần RE cho refactor này.

`BoundProfiles.actors` keyed PlayerId, chỉ mục `legacy_peers` dùng đường diagnostic không auth. Account mode cho hai
profile đã biết cùng peer IP; allowlist phải khớp tập IP của cấu hình, không dùng số actor để buộc IP duy nhất.
Diagnostic legacy vẫn từ chối trùng endpoint và giữ binding cũ; không coi nó là xác thực tài khoản.

`Bridge::login_for_profile` parse reportedIPv4 như metadata không tin cậy, kiểm body61/token32/opaque shape, tìm đúng
issued frontend rồi mới lấy expectedaddress từ **player của lease đó**. So actualpeer với peer của token vẫn giữ;
wrong metadata/unknown profile không chọn actor khác hoặc lấy reportedIP làm quyền. Frontend một lần, generation,
DB authorize/version/expiry/revoke và channel identity/name/echo/connection kiểm như trước. Unknown name/ID/profileversion
vẫn từ chối; chưa projection cho mọi account. Guard fixedroom IP→identity cũ đã bỏ vì ownership từ lease.

Channel account giữ địa chỉ từ actual admitted transport, theo đúng connection; legacy giữ configuredpeer. Không sửa
map IP của player toàn cục khi một socket mới đến. `LegacyDirectPlan` chụp round/membership/slot/endpoint tại loading99,
chặn missing/duplicate/outside slot/address0/port0 và recipient khác visit/connection/round trước builder153. Slot độc lập
PlayerId; selfentry vẫn zero bởi builder cũ. Plan chỉ là snapshot nội bộ dưới room lock, không relay hay wireepoch;
publication/batch origin/recipient/currentmodel fences CP265 tiếp tục áp dụng sau đó.

104Rust tests đạt, gồm token-selected metadata hai account chungIP, wrongmetadata/unknownprofile không ăn frontend,
revokeA/Bhealthy và plan sharedIP/nonidentityslot/stale visit/connection/round/invalid rows. Fmt/clippy/lockedbuild đạt.
14suite/306TCP legacy đạt,3169 consumedroomframes+24firstvacant khớp CP265 theo identity/normalizedvisit/opcode/sequence/
length/fullSHA; unreadtail giữ riêng. Thêm59TCP cùng sourceIP với reportedmetadata khác nhau, xuyên account→phòng→
hai kết quả/reset/Start3/revokeA/fresh rejoin; packet153 được decode kiểm IP chung và cổng đúng slot. 16accountserving
negative/lifecycle cũ đạt. Source/tool/binary/input frozen, ownedPID cleanup, raw/hashes giữ; lỗi clippy ban đầu ở
fixturetest gọi sai tên method/map-iteration lint được giữ rồi sửa trước các run cuối, không đổi input đang chạy.

Các ca này là **TCP tổng hợp**, không hai KR native hoặc đồng bộ UDP sau NAT. Endpoint153 vẫn IP channel/cổng99 tự
báo; không học cổng, holepunch, relay hoặc chữa N1bnegative. M5generalprofile/nativeDBqueuefault, key256/portreuse/
restart và sameIPUDPspoof còn riêng. Chỉ nghiệm thu fullR1 sau native2/4/8/account; không nâng gate từ source/TCP.

<!-- protocol-topic-content-end -->
