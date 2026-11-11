# Map, Ready, loading và phase

[Mục lục](../PROTOCOL.md) · [Câu hỏi đang mở](../RESEARCH.md) · [Thứ tự việc](../PLAN.md#dashboard).

<a id="room-map"></a>
### Map metadata, filter và request start (CP019)

**kr_static/kr_runtime confirmed trong phạm vi diagnostic copy**; fields/default/scheduling là
**emu_choice**, không phải capture server gốc. [Evidence](../evidence/room-map/README.md) khóa control,
recipe/hash và lỗi phép đo. Copy CP018 SHA94ee93… không thêm patch. VN chỉ dẫn tách config/table/ack,
không copy VN231/234/113, row128 hoặc index954. KR đo238/241/117, row120 và local playmap stride52.

- **C→S96**: writer`DDED40`, opcode60hex, u16 local-playmap code từ`E01930(index)`,u32 dynamic-index;
  gửi qua`13ADD60`, success đặt pending+6224=1 và disable controls. Không thấy ở probe13ADD10 không
  phủ nhận capture; CP018 width-candidate nay nối writer KR. Chưa kiểm đầy đủ select/reply roundtrip.
- **S→C117**: `DC70D0`, resultu16=0,u16 localcode,u32 dynamicindex. Native đầy8byte; runtime cùng cursor8/errors0,
  setter`DFF9F0` kiểm bounds dynamic-count. Ack0/0 khi count0 vẫn không cấp dữ liệu map; count1 mới có row.
- **S→C241**: `D6E6B0`, blob raw-selector0 với inner8byte `{u8,u16,u32,u8 count0}`, rồi
  `u32 dynamicCRC,u32 stageCRC,u8,u32`. Native full blob/tail tiêu31byte payload; dynamicCRC vào
  manager+0C qua`DFE850`. Mẫu mới sửa raw-selector/blob theo bài học CP016; UI-ready side effect các run
  CP015–018 là lịch sử đã quan sát, không dùng chúng làm proof full blob cũ đúng schema.
- **S→C238**: base`D68C90→D69278→DFE9A0→DFEB10`, outer blob. Inner u16LE count, rows:
  `u32,u8,stringLE,stringLE,2u8,stringLE,stringLE,2u8,u32,3u8,u16,u8,u32`.
  Row memory120byte; u16 tại+68 là local-playmap code (`E01930`), không phải VN+112.
  Native frame/blob, checksum và mọi row getter tiêu67byte của mẫu count1/strings ngắn;
  runtime parserCRC khớp/count1/return1/innercursor67=67. Count lớn/compression của238/bounds chưa kiểm.

`14EA760` trả stored-checksum; native blob giữ `405E70` và xác nhận **Adler32 seed0 trên inner67byte ở selector0**:
control`66DE09B4`, enabled`66ED09B5`. Giả thuyết tính cả tag ban đầu bị bác bỏ (`80D60A10` khácnative).
Allocator/free tổng hợp, checksum/reader không bị thay return. Config dùng checksum native tương ứng.
CP059 xác định checksum tổng quát nằm trên storage selector+representation **trước** inflate; selector0
zero-prefix/seed0 cho cùng giá trị như inner, không suy checksum ấy áp nguyên sang selector1.
Mẫu raw blob bên trong outer là `[18,u32LE length,selector0,inner]`; checksum này không tính tag/header.

Config mới thử đọc cache1090row/CRC`C5B1B1C1`, khácexpected nên từ chối rồi gửi528.
Sau wire238/count1, bảng dynamic-count lên1 qua`DFEF20`; cache ghi lại khi reload vẫn có checksum khác
trong cửa sổ đo. Đây là snapshot CP019; [CP059](lobby-profile.md#dynamic-cache-representation) đã giải lệch representation
cho cache này. Fullcache/reconnect vẫn mở, không suy cold/warm toàn client đã qua.
Raw snapshot/cache hash ở evidence; không coi cache kế thừa là server-origin oracle.

`CPlayMapInfoList` runtime có1151 record/stride52, snapshot tại init12. Index0 là **Antarctic01**,
capacity lấy record+24hex qua`EDC890`=4; ví dụ Camp01 index178, khác index VN104.
Fixture row hiển thị chữ **Village01 là emu_choice**, code0 vẫn trỏ Antarctic01; chưa có proof load
Village01/geometry hoặc preview map. Không dùng nhãn UI để gán map index/build.

**Map filter/start:** mẫu đầu row+5D=0 (inner offset52), ack/table có capacity4 nhưng start-check
`DCAD80` đo players2/teams2/capacity4, gọi `E00540→E06F80(mode0)→E061A0` trả0;
notice `CantDecideMap` từ`DCC032`. Predicate`E02C00` đọc row+5D, nhánh`E0622E` đòi khác0.
Mẫu enabled chỉ đổi inner52=1 và checksum/config tương ứng: runtime đọc row5D=1,
default-predicate1, các filter tiếp đi qua, selectable1; click check tại`DE03F3` trả0.
Không sửa roomtype/mode, ép filter return hoặc count/capacity trong debugger. Chỉ count1/mode0/
roomtype0/row field này đã đo; không khẳng định ý nghĩa tổng quát của byte+5D.

**C→S104**, actual click→`DE0310` builder`DE0463`→`13ADD60` return`DE0574`:
run `map-enabled`, socket4/frame28, seq124, wire109/payload71, send1/pending3. Client vẫn ở stage12;
chưa gửi response start hoặc quan sát loading/Battle. Parser KR/native CRC và mọi outer field
đọc hết71byte: compressed blob outer51byte (storage-selector1, native inflate ra inner265byte),
`u32 selected-dynamic=0,u16 localcode=0,u32 current-dynamic=0,u8=0,u8=2,u32=0`.
Map fields nối writer`E006F0`; context265byte từ`1264220` và u8/u32 còn giữ semantics giới hạn.
Native inflate/checksum giữ nguyên, allocator mô hình arena1MiB/mỗi request≤64KiB;
không gán inner context thành roster/spawn/seed đủ contract. Width/cursor đúng chưa đủ reply/load.

Trạng thái tại CP019 dừng sau104. Receiver/scene đã được bổ sung ở [CP020](#start-reply);
bước tiếp hiện hành chỉ giữ ở [RESEARCH](../RESEARCH.md#q-023). Không copy start501 của VN.
K2/K3/K4 và full identity/auth/user/cache/vanilla/multiplayer vẫn mở.

<a id="start-reply"></a>
### Phản hồi start và init CGameStage — CP020

Nguồn **kr_static/kr_runtime confirmed trong phạm vi copy/nhánh đã đo**;
[recipe/control/native/hash/correction](../evidence/start-reply/README.md).
Baseline CP019/PR27; các giá trị ID/seed/time/slot và lịch gửi là **emu_choice**, không phải server gốc.
VN normal-start117/auto501 và mô tả load chỉ dùng định hướng; dispatcher KR tự xác nhận
`DC51F0→DC5366→DC7F30` cho **S→C121**, không đoán opcode từ độ lệch VN.
KR120→DC7DE0 là nhánh status/error u8, không phải success body đã thử.

**121, nhánh outer profile-list flag0:** outer u8=0, blob raw-selector0. Inner của mẫu roomtype0:
raw SYSTEMTIME16byte, u32 seed, u16 localmapcode, stringLE mapname, u32 dynamicindex,
u8 roomtype,u8,u8 slotCount,3u8,i8 itemCount(count0); mỗi slot:
`u8 slot,u8 visibility,u8 spawn,u16 position,u8 team,u8,u8 feature138bool,i8 partner,4u32,u8 optionalFlag0`.
Runtime đo hai row26byte, inner100/100. Spawn setter`1241850` ghi slot+7C;
position setter`1241250` ghi slot+60, getter`1241290`. Đây là điểm khác VN/u8 position.
Tên map được so với CPlayMapInfoList qua`DC8470`, dynamic-index được kiểm`E00240`;
mẫu dùng đúng Antarctic01/code0/index0. Chưa có proof geometry/load từ hai check này.

Outer sau blob (các guard đang chạy trên copy): u8 optionalBlobFlag0,u8 time120,
i8 eventStage−1,u8,u16 count0,blob inner rỗng,2u8. Native và runtime đều tiêu
**129/129byte/errors0** với mẫu cuối (wire167); flag/profile/count/region khác chưa kiểm.
Bốn byte u32 dư của mẫu đầu không được callback đọc: giữ lịch sử cursor103/107 ở mẫu guest-only,
không dùng native gọi getter tự chọn để xác nhận schema callback. Mẫu cuối bỏ phần dư theo runtime.
Mẫu đầu cũng đặt nhầm thứ tự/nhãn spawn/position; runtime slot1 position256 đã bác bỏ,
control sửa layout đo position1. Các raw cũ giữ nguyên và đánh dấu candidate bị thay thế.

**Slot validation** `DD6190`: getter`1240370` đọc record+0C (ID), getter`1241290` đọc position.
Trong run, manager+A4=26, vòng partner/validation ở121 đã kiểm slot0..23.
ID!=0 đi nhánh active, position−1 bị loại; ID0 đi nhánh empty, position>=0 đòi thêm flag
`1243200` (vector+ B0/stride24hex) khác0. Slot0 của recipe cũ có ID0/position0/flag0 nên trả0,
client tự về stage11. Đây là giới hạn ID0 workaround CP018, không phải lỗi map checksum.
Bỏ slot0 khỏi row list giữ position−1 cho qua để cô lập downstream; run guest-only không chứng minh
own-player tham gia hoặc gameplay. Sau đó đã kiểm riêng identity khác0 bên dưới.

**Nối ID từ auth bằng input:** mở rộng prefix S→C4 cũ13byte thêm
`u32,u32,u16,u16,u32 ID`, rồi4byte số0 được recipe cũ **định dùng** làm chuỗi rỗng vàu8, tổng34byte.
[CP058](lobby-profile.md#auth-profile) xác nhận4byte đó **không** là chuỗi rỗng hợp lệ: reader cần u16length,
set err54 tại cursor29; proof setterID trước lỗi vẫn giữ, không dùng prefix này làm successbody sạch. Cạnh`10FD8AA→10FD906→13B09E0`
ghi context local-user+140/index0. Native đọc đúng bounded prefix; runtime đo setter ID1,
KR106 playerID1→lookup0/ownflag1; không sửa local-list/ownflag trong debugger.
Full body auth/profile/tail vẫn thiếu: chỉ chứng minh nguồn và lifecycle ID trong recipe local này,
không xác nhận login đầy đủ hoặc dùng ID1 làm policy server gốc.

Mẫu start121 hai player (ID1/ID2), position0/1, spawn0/1 qua kiểm slot rồi gọi
`12562F0(13)` từ`DC937B`. Object init đo tại`E572D9`, vtable`176A470`, RTTI
`.?AVCGameStage@@`. Sau init client gửi **C→S99/61byte**; ý nghĩa99/load ACK chưa xác nhận.
Stage/class identity và init entry riêng chưa chứng minh geometry/Battle. Các run tiếp `start121-social`
và `start121-lean` có ảnh thực tế: **scene map băng, HUD roster/timer02:00; probe gọn có hai nhân vật,
local có mũi tên, guest labelKRGuest**. Đây là proof đã dựng scene/nhân vật trong phạm vi hình ảnh;
chưa đo input/di chuyển, clock chạy, spawn tọa độ hợp lệ bằng getter, bom hoặc chu kỳ trận.
Code0/keyAntarctic01 được so tên thành công; exact geometry variant/asset path chưa trace.
Không suy playable hoặc qua K4/K5 từ ảnh này.

Copy CP018 SHA94ee93… dừng ở generated-import trap22646D4/caller125A54B.
Bản copy tiếp bind16EA6CC→user32!GetMenuItemCount candidate, vòng xóa menu có API
win32u!NtUserGetSystemMenu/NtUserDeleteMenu đã bind từ trước; runtime count9, init đi tiếp.
Sau đó trap22657A8/caller13CA2F9 là slot16EA8BC, cluster preferred-export và ABI5cdecl
khớp nmco3!NMCOChangeMyInfo candidate. Các binding là **emu_choice/candidate**, không ép return;
source copy/patch byte/hash/rollback nằm tại evidence. Runtime bind export NMCO trả0 và client tiếp
C→S15; run social có **GDB tự crash** sau đó (host backtrace, không phải client signal được định vị).
Run lean cùng copy/fixture lặp lại init/ảnh hai nhân vật, không log signal; stub teardown cuối run có
CALog disconnect/error10, không dùng làm kết luận timeout steady-state. Không sửa DLL/client nguồn.

Tại CP020, bước tiếp là99/round/clock/input. CP021 bổ sung bên dưới; bước hiện hành ở [RESEARCH](../RESEARCH.md);
identity/cache/full auth/codec/multiplayer và gates còn mở.

<a id="round-go"></a>
### Phase, clock map và input — CP021

**kr_static/kr_runtime confirmed trong phạm vi diagnostic copy**;
fixture/default/timing là **emu_choice**. [Evidence](../evidence/round-go/README.md) khóa run/hash,
negative, native ledger và ảnh. Không thêm patch client so với copy stage-social CP020 SHAe9bb47…;
code của bốn vùng đang đọc đã so byte bằng nhau với static copy660447….
VN189/190/191 và ví dụ thời gian chỉ hướng dẫn; dispatcher KR tự nối196/197/198, không suy opcodeoffset.

**C→S99**, writer`E7F2A0`, actual stack`E7F4DD→E59587→E57F33→E57C11` tới`13ADD10`;
guard once stage+6410. Constructoropcode63hex, writes2u32,u16,u32 rồi hai local-user entries:
entry có user ghiu16 position +3u8 từ actor; entry ngoài localcount ghiu32 FFFFFFFF.
Capture61byte/payload23 trên socket4/seq136 của `round-control`: native getter sequence
`u32,u32,u16,u32,u16,3u8,u32` đọc `[2130706433,0,9867,682740161,0,1,1,5,FFFFFFFF]`.
Hai u32/u16 đầu lấy context`D97010→13E1470/13E14A0`, giống endpoint; chưa đủ hợp đồng peer/loaded ACK.
Không dùng absence ở13ADD60 của CP020 để phủ nhận capture99.

**S→C197**, dispatcher`E5D3A0→E5D79B→E751F0`: u32,u8,2u32,u8 flag;
mẫu flag1 dài14byte được đọc14/14/errors0, flag1 lấy hai actor local cho vector nội bộ.
Handler gọi`E78620(2)`, pause clock stage+6B34, đặt2timefields+64D4/+64D8 rồi nhân1000.
Giá trị12345/kind0/time120/timeB0 hoặc30/flag1 là lựa chọn thử, không gán semantics mọi field.
Flag0 cần thêm count/list; nhánh đó chưa runtime.

**S→C196/mode0**, `E5D741→E73760`: u8 mode,u32 vào stage+63E0,u16 mapcode;
mẫu7byte/cursor7/errors0. Nhánhmode0 kiểm phase+6C44==2, gọi`E75A50→E78620(3)` và
`E0DF10` bỏ pause trên clock stage+6B34. Chưa kiểm mode1/2/4/load barrier/variants.
Hai run chỉ197/196 dù tới phase3/unpause vẫn cho timer00:00; không coi state này đủ clock gameplay.
Probe getter tần suất cao trong time30-control không log CLOCK_SAMPLE; absence đó không chứng minh clock đứng.

**S→C198**, `E5D7AD→E74410`:3u32,u8, mẫu13byte đọc13/13/errors0.
Không nhân1000 như197: mẫu gameTime0,limit120000,timeB30000,flag0 là emu_choice.
Handler đặt+64AC=1, timefields, gọi`E13490→E0DC60` khởi tạo clock ở play-map+1DF088;
runtime đo origin khác0/game_time0/running1. Đây là clock khác pointer stage+6B34 vừa unpause.
Sau197seq76,196seq79,198seq82 trên channel socket, ảnh timer01:56→01:50→01:47;
không ép phase, timer hoặc API return từ debugger.

`begin198-input` focus cửa sổ và thử Right/Left/Space. Ảnh trước/sau cho local actor đổi vị trí
sau Left, guest giữ vị trí; run tiếp Up/Space/Right/Space lặp lại clock và local đổi vị trí theo input.
Đây là **input/di chuyển local trên một client**; chưa tọa độ getter/collision/peer sync hoặc authority matrix đầy đủ.
Space có outgoing118/60byte được capture cùng các127/132/190. Các opcode phụ chưa có semantics đầy đủ.
StaticB7A760 dựng167/u16actor,u16constant6 là mỏ neo khác, chưa có runtime hit, không phải bomb request đã xác nhận.

**C→S118**, actual enqueue`13ADCC0←E2E373`, writer`E2DF80` tạo opcode76hex qua`1164D10`
và gọi`1164D80`. Probe13ADD10/13ADD60 không bắt118; actual Winsock send qua IAT16EA78C
bắt60byte ở stack`14F50AE←14EF29B←E2D654←E2D163←E7D2DB←E7C4F3`.
`13ADC90` gọi virtual flush; stack flush không phải stack tạo record ban đầu.
Base payload22: `u32,u8,u8,u32,7u8,u32,u8` từ record+08,+00, owner argument,+0C,
+10,+14,+30,+34,+38,+3C,+18,+1C,+40. Native getter đọc đủ
`[12345,0,0,0,100,1,1,0,0,0,1,1,0]` của capture frame46/seq178.
Flag record+40 &3 khác0 còn3u8,u16,u32 từ+20,+24,+28,+2C,+44; nhánh này chưa runtime.
Không gán field+10=100 thành tọa độ tile chỉ từ nhãn parser VN.

**S→C159**, dispatcher`E5D51D→E5E8F0`, mẫu branch context0/slot0/flag0 đọc39/39/errors0.
Thứ tự mẫu:2u8,u16,3u32,3u8,u32,u8,2u16,u8,u64,2u8.
Context được so với play-map; slot được kiểm trước`E14600`.
Run `reply159-rerun` có entity return61BDD300 khác0, nhưng ảnh sau Space và timer01:40/01:37
chưa cho thấy bom riêng biệt. Entity return khác0 không chứng minh đã đặt bom lên map.
Tại CP021, ID1/zeros/first-record serial0 là emu_choice; CP024 bổ sung echo bên dưới.
General schema, nhiều bom/ID và nhánh flag/type khác0 vẫn mở.
Cặp159/retry chồng thời gian sau daemon recovery nên chỉ exploratory, không dùng làm independent repeat.
Run riêng `explode158` sau khi cả hai kết thúc xác nhận full39, entity có vt16FC0CC/
RTTI `.?AVCGOGameBomb@@`; sau Space rồi Right, ảnh cho thấy bom nước tại vị trí vừa rời.
Đây là proof **bom hiện trên map trong một ca local**, chưa đầy đủ serial/ID/duplicate/authority lifecycle.

**S→C158**, dispatcher`E5D565→E5F450`: u8 count, mỗi entryu16 ID,u8 kind.
Mẫu count1/id1/kind1/seq88/payload4 được đọc4/4/errors0,`E14360` tìm lại **cùng pointer bom**
và tới`E9C520` qua nhánhkind1. Ảnh timer01:32/01:20 vẫn còn bom; không có proof blast/block/trap.
Entry calc không tự chứng minh nổ hoặc xóa bom. Static kind0 còn đọc **2u32,u8**, khác mẫu VN151;
nhánhkind0 chưa runtime. ID1/kind1 và delay2.5s sau159 là lựa chọn test, không phải fuse/authority gốc.


Tại CP021, bước tiếp là guard/state/clock calc và về phòng; CP022 bên dưới bổ sung kết quả.
Bước hiện hành chỉ ở [RESEARCH](../RESEARCH.md).
Chưa hoàn thành một trận, chưa multiplayer/fullauth/cache/vanilla hoặc qua K2–K5.

<a id="distinct-spawn"></a>
### Spawn riêng trên profile service hiện hành — CP065

**kr_runtime/confirmed hẹp trên copy Wine**, [recipe/native/hash](../evidence/distinct-spawn/README.md).
Tái dùng [CP024 index→map point→actor](bombs.md#bomb-position) và exactfixture ExVillage01/code278, không
RE lại loader hoặc đổi point map. VN §5a-7 giúp checklist init một lần/per-slot index, không opcode/
layout/offset VN. Profile overlap trước đây là lựa chọn phục vụ contact/race, không vắng cơ chế spawn riêng.

Body121 knownfixture riêng và overlap đều129byte; khác một byte typed ở bodyoffset86, native inner
ledger **slot1 spawn tại offset76:0→1**. Slot0 giữ0; seed/map/room/roster/position/outertail không đổi.
Native frame/blob/getter tiêu inner100/outer129/errors0. Profile41 chỉ aliasstart đổi,40alias còn lại
khớp byte; không thêm schema hoặc derive spawn từ tên/mapindex VN. Hai fixture/checkpoint cũ giữ hash.

Run currentproductionservice/guard127, không replaywrapper:8connections/IDs1,2/slots0,1, haiStart/
clock/keys12345→12346. Nativeinit point A(3,3)/B(3,9); clock cảhai nativeview A(120,159),B(120,399)
ở **cả hai vòng**. Sau normalUI cảhai view cùng sampledcoordinate từng actor trước175; không fullUDP
schema/correction/transportcausality/pxs. HaiA124/165/Roller5→6/reset5, A118IDs1→2/159158/A17/19→
B170/B21, nativeAstate1bothbefore175, hai175/180/roomrefresh. Auth/cache/currentkey/reportguard giữ.
Không gộp CP062overlaprace thành tranhitem distinctspawn cùngrun; không fullmatch/K4/K6a/LAN/Internet.

Exporter mặc định **distinct** dùng exactCP024`exvillage121.bin`; `--spawn-layout overlap` giữ
`collision-r1/overlap121.bin` cho RE. Metadata`spawn_layout` là design khai báo fixture, không fieldwire.
Không tự migrate profileJSON cũ: export path mới rồi dùng hash. Đã đo A/B với distinct; default này
chưa là B/C/accountresume/roster tổng quát hoặc chọn spawn ngẫu nhiên theo seed. Sourcebinary/pack/DB giữ.

Giới hạn còn ở [RESEARCH](../RESEARCH.md#gap-map-spawn-loading); thứ tự chỉ ở [PLAN](../PLAN.md#current-order).

<a id="service-eof"></a>
### Service EOF và loading chờ đủ99 — CP050

[Recipe/raw/hash/nguồn reuse/native checks](../evidence/service-eof/README.md). Wire107/110/175/180
vẫn theo [room leave](room-membership.md#room-leave), [forfeit](life-results.md#round-finish), [pending timer](bombs.md#abort-timer);
không phục hồi lại receiver hoặc chuyển opcode VN. `close_connection` idempotent, đánh dấu đóng/
unregister rồi loại connection khỏi peers/setup; channel EOF lúc active gọi finish ngay trong owner,
đặt abort và giữ ticket timer gốc. Tới deadline ticket bị loại `round_closed`, không phát158.

Policy fixture cho hai người:12s sau EOF chọn rank survivor thắng,5s sau180,3s sau107 xóa slot
người rời và110 cho survivor; B được promote nếu A rời, A giữ host nếu B rời. Đây là
`design/emu_choice`, không death/event19 hoặc thuật toán server gốc. Kết quả đã chọn trước EOF
không bị thay bằng một chuỗi rank mới. Profile34fixture tại CP050 yêu cầu rankAwin/Blose và promoteB có SHA;
profile32 cũ phải export mới, không sửa lịch sử. Send failure/idle-EOF/roster race chưa kiểm runtime.

**kr_runtime/confirmed hẹp:** hai run riêng host/guest EOF khi actual bomB còn chờ6s; mỗi survivor
175full302/inner292,180full3 rồi107base6/rewind/final1, ID người rời về0; chiefB0→1 hoặc chiefAgiữ1.
Không postEOF senddead/158 hay actual/synthetic17/19; secure actor trước rank không state1.
Đây là forfeit/cleanup đã kiểm trong service, không trận đã chơi xong hoặc account reconnect.
UI helper bấm tiếp nút một người tạo modal cuối; snapshot sau cleanup và native getter giữ phạm vi.

**Giữ99:** wrapper giữ dispatch frameTCP99 thật củaB10,002s; A đã init99. Service không phát
197/196/153 trong khoảng đó; sau release phát đúng phase, cả hai có clock native và actualinput/
bom/result/return vòng1. Run185s có Start2/clock2 nhưng chỉ một result/return; không hai trận.
Đây là kiểm policy đã chờ đủ99 với can thiệp có log; không networkloss/native-server loadpolicy,
timeout/latejoin/loadfail hoặc toàn gate K4. Giới hạn ở [evidence](../evidence/service-eof/README.md).

Tại snapshot CP050, service chỉ hai identity/cap8/StartA; CP051 tích hợp
[C mới/roster-ID mới và B Start](room-membership.md#service-replacement). Giữ giới hạn CP050 như lịch sử.
Fullvalidator/auth/cache/codec/vanilla/K5/K6a/LAN/Internet giữ mở. Câu hỏi ở [RESEARCH](../RESEARCH.md#gap-disconnect-replacement).


<a id="loading-timeout"></a>
### Loading barrier và deadline tùy chọn — CP135

**kr_runtime/confirmed trong fault injection; policy là emu_choice/design.**
[Recipe, raw/hash và phép kiểm](../evidence/loading-timeout/README.md). Control current service giữ
frame99 thực của B ở dispatcher46,349s, A99 đã dispatch; cả hai đọc121129/errors0, không có
197/196/153/nativeclock. TCP được quan sát ESTABLISHED trước cleanup; hai client đã load/vẽ map.
Đây không bằng chứng client gốc không load, network loss hoặc timeout của server Nexon.

Profile cũ **mặc định không có deadline**. Profile mới có thể khai:
`"loading_timeout": {"mode":"close_missing_init99","seconds":30}`; seconds là số nguyên1–300,
giá trị30s do emulator chọn. Sau khi phát121, service pin generation, từng role/connection và deadline.
Một owner tick thấy mọi init99 đã dispatch thì resolve `barrier_complete` trước khi xét expiry;
đã resolve/round inactive/generation khác không chạy lại. Channel bị thay hoặc mất membership thì
resolve `membership_changed`, không đóng nhầm replacement. Đây không validator epoch trên wire.

Nếu hết hạn mà barrier còn thiếu, chỉ đóng các channel được pin chưa cóinit99, với reason
`loading_timeout`. Đường close hiện hành invalidates fence ngay rồi dùng abort/forfeit175→180→
107/110 cleanup; không mint99, death hoặc một cancellation opcode mới. Fixed đã đóng riêng B8
ởStart+30,043s; A đọc175302/1803/errors0,107 xóaID2→0 rồi110/chief1 và hiện phòng/nútStart.
B hiện dialog mất kết nối/error10. Callback cleanup được kiểm sau180 hiện tại; không nhầm idleBack107.
Forfeit/rank/30s là lựa chọn emulator, không báo cáo trận đã chơi hoặc luật server gốc.

Giới hạn: runtime mới kiểm B99 bị giữ với A99 thật/hostA/hai copy Wine; cả hai thiếu, replacement,
đổi vòng/defaultoff/boundary có regression model, chưa phải runtime mọi variant. Đường bình thường với policy bật có positive CP136 bên dưới. Không K5/K6a/fullmatch/vanilla/LANInternet/general recovery.


<a id="loading-timeout-positive"></a>
### Positive loading deadline qua hai vòng — CP136

**kr_runtime/confirmed hẹp**, [recipe/hash/phép kiểm](../evidence/loading-timeout/README.md#cp136).
Exact normal CP134 controller/profile, thêm duy nhất loading_timeout30s, không hold wrapper.
Cùng run có hai actualA/B99, barrier_complete ởgeneration1/2 sau15,165/16,188s, native121129/
197/196/153/clock hai lần mỗi view; cùng channelA4/B8. Trận tiếp tục qua deadline, actualA/B17→19
mỗi vòng, chosen all-dead rank2,2/full175180 và về phòng hai lần; không loading_timeout/abort hoặc
channelclose trước mốc đo. Start2 pin deadline mới/key12346; callback/195flags geometry được kiểm
bằng assertions đã có ởCP134, không phục hồi thêm cơ chế native.

Runtime này chứng minh policy tương thích đường normal của **profile này**. Nó chưa bật
`death_lifecycle` hoặc `result_freshness`; các guard đã có từCP124/106 nhưng tích hợp chung với
current deadline/survivor/movement cần phép kiểm riêng theo PLAN. Không suy cùng enum/field hoặc
các kết quả khác run là proof một profile tích hợp. Source/code production không đổi sauCP135;
226server tests reuse, local/raw/hash và auditAUD042 qua136. Fullmatch/K5/K6a/vanilla/LANInternet/
originalauth/serverauthority và genericloading/membership vẫn mở.


<a id="preclock-eof"></a>
### EOF sauStart, trước clock — CP072

[Raw/recipe/failures/audit](../evidence/preclock-eof/README.md), **kr_runtime/confirmed hẹp** cho forfeit/
return trướcclock, **unknown** cho native slotcleanup. Production service/profile41/guards giữ;
wrapper giữ actualB99 tại dispatch≤30s, discard sau actualchannelEOF. A99 đã nhận, hai native121
full129 nhưng chưa197/196/153/clock. BEOF tronghold, A175full302/180full3/stage12; không bom/death.

107/110 được phát nhưng chưa có native107/IDclear trước stopservice-refresh+3s; AECONNRESET xảy ra
sau controlledstop, giữ rawerror. Không suy gửi107 là xóa slot; không quy thiếu trace cho engine/
debugger. Observerretry chờnativecleanup≤30s lại bị NMCO startupabort trướcStart, không đo EOF.
Cả firstB/thirdA UI SetLocaleAndRegion2000:0/SIGABRT giữ, cùng loại CP064; causeunknown.

Forfeit/rank/timing/dispatchhold là **design**, không death/originalserver. Không thêm body/patch/
servicefix/force state; không gameplay/2cycles/fullreset/hostEOFvariant/accountresume/timeout/loadfail/
K4/K5/K6a/fullmatch/LANInternet/vanilla. Next actualblocker: NMCO locale caller/input/result,
không lặp input/proxy hoặc reopen175/180receiver đã đo. Sau startup ổn mới hoàn tất native107.

<a id="preclock-cleanup"></a>
### Native107 và UI sau preclockEOF — CP074

[Raw/recipe/native/image/hash](../evidence/preclock-cleanup/README.md), **kr_runtime/confirmed hẹp** trên
currentWinecopy/profile41/service giữ nguyên. Reuse CP072actualB99hold/realEOF/175180, CP034Back/
107slotclear/chief vàCP073attributionprobes; không nativebody/patch/forcing/providerfix.
Hai run: accepted121129/actualA99/B99held→realBEOF/discard, không197196153/clock/death/bom.
A175full302→180full3/stage12→107base6/rewind0/final1/errors0; slot1ID2→0,110full6/chief1afterclear.

Firstnativeobserver10,485s sau servicesendcleanup có đủtrace nhưng ảnhđen. Followupfocus/chờ4/8/12s
renderphòng mộtKRLocal/ôBtrống/nútStart; blackframe trước không là failedcleanup/hangproof, chưa tách
focus vs wait causality. NormalBack113581→actual86empty/seq205/nativeframer0 và UIsảnh; servicechưa
explicit86 membership reset/recreate handler, không fulljoin/profile/cache/list/rejoinproof.
Serviceerrors0/khôngnativefallback/errorstate tronghai window; không suy sửa được lỗistartuphistorical.

Forfeit/timing/hold là **design**, không servergốc/death. CP072missingnativecleanup được đo bổ sung,
không viết lại history hoặc suy causalroot oldreset/blackframe. Chưa accountresume/hostvariant/
timeout/loadfail/wholemembership/fullreset/gameplay/2cycles/K4/K5/K6a/fullmatch/LANInternet/vanilla.
Bước tại CP074 (lịch sử) là selected TCP guard admission; CP075 bên dưới bổ sung kết quả. Thứ tự hiện hành chỉ ở [PLAN](../PLAN.md#current-order).

<a id="return585-ready"></a>
### 585 sau return và Ready2 — CP079

[Counterexample/cache/native/rawinterruption/hash](../evidence/return585-ready/README.md).
**kr_runtime/confirmed hẹp:** CP069 hai vòng cóactualB95/A104 dùkhông106mới sau585. Thiếu106
khôngnecessarycause trongmeasuredprofile; không thêmwrapper106/persistentfix chỉvìCP078 có585.
CP078Readyclick trướcactual585 khácCP069, timingcause vẫncandidate. ReuseknownReadyguardcached.

Run mới chỉlog/native snapshottrướcgiánđoạn: oneR1/rejoin/effects/resultreturn, nativeReadyentryđầu
pending0/local1/count1/host0/build1; sau180enablearg1/pending0,ảnhBReadyUI đúngprofile nhưngkhông
Readyentry2/noactualB95/A104. Đây làcandidateblocker trướccallback, chưa làReadyguardrejectproof.
Toolshellsessionexit143 trước ghi originalrun.json, causeunknown; finalactionreturncode/roleenv/phase/
isolationPIDmanifestunknown. Niêmphongraw/service-statelast riêng, khôngdựngrun.jsongốc hoặc gọi
laststate.errors[] làfinalerrorfree. Sourcecode/profile/fixture/binarygiữ, nofunctionforcing/replay/newpatch.

Counterexample cóhash/finalrun gốc; tracegiánđoạn cóscope riêng, không nângnativeproof/model/gate.
[PLAN](../PLAN.md#current-order) đoWndProc/inputfilter/button→Ready vớiprogressmetadata lưu ngay,
khôngặp550s/vòng2chờvôhạn/đổireplythiếubằngchứng. Fullmatch/2cycles/currentreset/gates/LAN/
Internet/vanilla/fullauth/codec/validator/membershipvariant giữmở.


<a id="ready-input"></a>
### Input Ready/Start sau return và hai chu kỳ sau rejoin — CP080

[Source-first/control/run/hash/correction](../evidence/ready-input/README.md),
**kr_runtime/confirmed hẹp** trên modified Wine copy/current profile. CP016 cung cấp input path,
CP028 lịch sử retry, CP069/079 counterexample không cần106 mới; VN chỉ checklist, không numeric port.

So cùng run: click Ready đầu qua WndProc1259C00/filter→button1509376 command3500,
mouse-up state1→release1509629/delegate1→callbackDD3940→DDE760→actual95. Click sau return
cũng nhận down/up đúng tọa độ, enabled1/gateA8=1/style21, filter không consume; nhưng delegate3
chen trước up và state0, không callback. Đây là boundary trước Ready guard, không pending/local/
count/host/throttle từ chối hoặc thiếu reply106 đã chứng minh.

Watch A/B **+148/+14C**: down instruction1509711 đặt state1 (watch post-PC150971B);
message nội bộ **0x464** tới button, instruction1509438 chép saved+14C=0 về state+148
(post-PC150943E), trước mouse-up. Stack1504E2E→150475D nối đúng helper1504D0A. Helper tĩnh
có đổi control giữ ở manager+24, gửi old465/new464 khi khác, rồi lưu handle mới; không gán
ý nghĩa Win32 cho các message nội bộ hoặc suy nguyên nhân Windows vanilla. Một thân helper mới,
range0x300 khớp static/runtime/modO; consumer/writer còn lại tái dùng cache.

Control **ordinary OS UI/design**: đưa pointer ra500,450 rồi vào605,526/chờ0,5s mỗi bước trước
nhấn1s. 464 xảy ra trước down; watch state1→release0, không restore chen giữa. Cả BReady/AStart
sau return đi callback ngay và actual95/104 một lần, không retry/new106/service/profile/client patch
hoặc native forcing. Watch-run trước có B retry95 nhưng AStart đầu vẫn mất và chỉ một vòng;
không ghép run đó thành hai vòng. Không gọi UI recipe là sửa contract server hoặc universal timing fix.

**Cùng-run integration confirmed hẹp:** guest Back/rejoin rồi hai native121/clock/175/180/refresh
ở cảhai view, tám TCP/IDs1,2/slots0,1/sequence tiếp tục, key12345→12346. Mỗi vòng actual124
Roller17/Bomb27/Fire28, native165 delta hai phía và baseline vòng2; actual118 ID1..4/power1,2,1,2,
159/158 tương ứng, stateA1 trước175. Hidden-pointer join không đo mới. Native framer/getter/heap
models chỉ corroborate bytes/fields, khác native callbacks/runtime. Policy rank/drop/fuse/host là design.

Old-storage watch sau Start2/stage13 có hai stack-unwind truncations mỗi role ở post-PC152590D;
raw/failure giữ, parser/checker tách đúng warning khỏi probe evaluation error. Không suy free/destructor
hoặc UI state từ địa chỉ sau lifetime; phép đo kế phải tháo watch lúc rời UI. Native trace vẫn đủ hai
chu kỳ; planned assertion/stat không bỏ. Chưa fullmatch/K5/K6a/vanilla/LAN/Internet/full auth/codec/
validator/authority/reset/general membership; [PLAN](../PLAN.md#current-order) chuyển khoảng trống bom
khác owner, không tiếp nhánh106 hoặc lặp Ready cause.


<a id="return2-visual"></a>
### Render/Ready sau return2 trên Bhost — CP110

**kr_runtime/confirmed hẹp**: [recipe/raw/hash và giới hạn](../evidence/promoted-outcome/README.md#cp110).
Reuse CP074 focus/wait/windowPID và CP080 Readywriter/hover, CP109sameprofile47/knownroute;
không nativebody/codec mới, không port numeric VN. Baseline PR179/merge8f3ea2f, review waived.

Run457,296629636s/SHA `b289d1d10257302ae73c06571b054e414303bf9d541d1922af044c874e76bc11`:
hai native121/clock/175/180/view, Aguest/Bchief/sameTCP/12entities. **Sau return2** stage12 gọi→
PrepareInitDC1AB0→EnableDDF330(arg1,pending0)→hai110DC6ADC/role đủ6/6/errors0.
Ảnhunfocused0đen; mốc4 và12s đãhiện phòng cảhai trướcthao tác focus mới. Window16777217ở
DISPLAY0/1 trỏ actualgamePID136/489/gameA/B, không phải cửa sổ launcher. Không tách nguyênnhân
focus trước đó/thời gian; không suy ảnhđenCP109 là enginehang hoặc đãsửa lỗi runtime.

Sau mốc focus/wait, normalCP080hover AReady→DDE760(currentPrepare/pending0)→DDEAB7→actualC2S95
**seq868**, body425bytes giữopaque/hash theo currentnativefront/upstream; cảhai native110 đọc6/6
sau95 và UIhiện ready. Không mặcđịnh95emptybody: assertionempty củascriptphântích đầu đãfail,
được giữ cùngscriptcũ; mới chỉ xác nhận exactopaque425/bodySHA/currentseq, chưa thêm layout95.
Không Start trận3, không biến95/110 thành fullauthority hoặcnghiệmthu gate.

Hai runpartial trước giữriêng:225,616s Up8s chỉcachedsample, final158 hoàn tất trướcbound0,643s;
204,173s capacity6s cũng chỉcachedsample. Run3 giữprofile/service/fuse; thêm read-onlykey/frame/
end158E5F5FD tạicửa sổ feedbackA vàboundcapacity/Up12s. 611edgeevents/key439/frame167/end1585,
không forcefield/gamecall. Đây là điều kiện quan sát khác, không causalfix/motionlaw/đổi tốc độ.

**Giới hạn tích hợp:** strict4phase kếthừa fail `currentpositionprogressacrosslatercallbacks R1-A-Down`;
CP110 không thaythế CP109run4phase proof, không gọi cảrunCP110 là mọiintegrationpass. Nativefield
getters/framer/typedpacket/flagreader chỉ emulation khácUI/native95. 4.686scopedchecks gồm4.640raw
size/SHA vàjoins, không ngữ nghĩa/gateoracle. RESEARCH/PLAN giữ gapreceiver/samplemuộn; fullauth/
codec/validator/authority/allreset/vanilla/K5/K6a/fullmatch/LANInternet còn mở.


<a id="receiver-prepost"></a>
### Receiver pre/post giải missing-progress của CP110 — CP111

**kr_runtime/confirmed hẹp từ raw đã có**, đối chiếu nguồn trướcphân tích: CP098/106knownAPI/key/
B4E6F0/B4EA14/getter52A350, CP110actualpeerB/framewindow/freshsamples. Không runtime mới/nativebody/
codepolicy mới; [metadata/hash](../evidence/promoted-outcome/README.md#cp111), [gap](../RESEARCH.md#gap-peer-movement).

Oldstrict chỉappend **before XY** từngapply. R1A-Down→B có4callbacks, trước cảbốnY159; callbackcuối
trong chínhraw đổi currentY159→219/target219, distance60/threshold50 (snapbranch), sauđó
freshrecipientpostinputY219→294. Vìkhôngcó callback kếtiếp trongwindow, thốngkê before-only vẫnchỉ159.
Đây giải thích **assertion thiếu before-state khác**, không phải nativekhôngdịchchuyển. Khôngquota64hit,
không cầnchạyY389/Ready/observerlại hoặc suy chínhobserver lànguyênnhân lịch sử.

Phépkiểm mới giữuniqueAPI17→packet13/key/slot/args/actor/prepost/target/distancebranch vàfreshpostinput
ởcảhai view; chỉ đổi câu hỏi thành currentprogress **trong hoặc giữa actualapplies**.
4pha: R1A-DownB4apply/1targetchange/Y159→219; R2A-DownB6/1/Y159→219;
R1B-RightA6/2/X120→164→200; R2B-RightA6/2/X120→168→200.
556relations/360copiednativegetters/49actualCRC; fullprepost securewords/data pinned.

Tách branchcoverage: runnày có4distance≥threshold snap, **0nonzero-error-below-thresholddeferred**.
OldCP106 cócảhai branch vẫn giữriêng; oldstrict/CP110failure và firstnewanalysis mandatorydeferred
failure giữ nguyên. Newscopedproof không báo oldstrictpass hay chứngminh toàninterpolationlaw.
CùngrawCP110cóUI/native95/110 vàcurrentcoordinate4pha theođịnhnghĩa mới, khônglấytoạđộ từảnh hoặc
hai runriêng. Vẫn khôngfullvalidator/auth/authority/allreset/vanilla/K5/K6a/fullmatch/LANInternet.


<a id="rescue-return-start-guard"></a>
### Sau cứu: result/return có, Start2 bị room guard3 — CP117

**kr_runtime/confirmed hẹp**, [recipe, control, raw/hash](../evidence/rescue-service/README.md#cp117).
Giữ CP116 service/profile/client/fuse; fixedfriendlyAB + synthetic2, B cứu/di chuyển rồi normalASpace/
BSpace tại chỗ. ActualA/B17→19, existing `all_live_AB_accepted_dead_rank2` chọn175/180; native175/180
và hai phòng trở lại được quan sát. Đây là **emu_choice**, không originaltie/teamrule hoặc “mọi actor chết”:
slot2 đối địch tổng hợp vẫn không là client thứ ba.

Nhóm run phân biệt rõ: r1 dùngReady6s từservicerefresh chưa95; r2 timedBLeft cho actualbomb(3,2)/power1,
chỉB19, chưaresult nên không kiểmReady; r3 đổi sanghaiSpace→cảhai19/result/room, dùng cảhai
actualreturned585/focus12s→**B95 thật**, mộtStartA không104. r4 giữrecipe và tốiđa3Startclick vẫnkhông104.
Raw/ảnh/logpartial giữ nguyên; không đổi outcome/fuse hoặc tạo report/state để qua.

Run có breakpoint chỉ đọc tại site Start đã cache: lần đầu `DE0310` pending0, predicate12201A0/12202A0
đều trả1, `DCAD80` trả0, tớiDE0463build104. Sau return/ReadyB, actualStarthandler vẫn vào,
pending0/predicate1,1 nhưng **DCAD80 trả3** →switch tạiDE0422 chọnDE0434/error, khôngbuild104.
Đây định danh nhánh runtime thực, không quy lỗi cho thiếufocus, socket hay actorC bằng suy đoán.

RE tĩnh có mục tiêu từ cùngmodOa08950 cho thấy hai site trả3 tạiDCB7DA/DCB7FD trong vòngslot0..15;
đường đọc packedstate qua1240A50, group `(word>>2)&7`, helper703A90 và1240D70. Chưa quan sát
slot/state/helper nào gây nhánh nên **nguyên nhân cụ thể unknown**. Không gán code3 thành “C chưaReady”.
Site/nativebody cólocator/hash trong evidence; không sửa DB/sourcebinary. Phép đo kế đọc đúng
slot/state/return3site, rồi mới chọn refresh roster/policy cần thiết.

CP117 chưaStart2/freshrescue/reset saucứu; chỉ mộtresult/room/actualReadyB và nativeblocker,
không fullmatch/K5/K6a/vanilla/LANInternet. 190tests CP116 reuse do service khôngđổi. CP117/PR187 đã merge `c2ca94d`;
AUD036 qua116/kế120. ChuỗiCP114kimStart2 giữ scope riêng, không dùng lấp bằng chứng rescue.


<a id="rescue-start-slot"></a>
### Slot gây guard3 sau return — run lưu trước migration CP118

**kr_runtime/confirmed hẹp**, [metadata/hash](../evidence/rescue-service/README.md#start-slot).
Run `rescue-start-slot-r2` dùng probe chỉ đọc trong actualStartentry→switch, khắc phục observerquota
cũ hết trước câu hỏi. InitialStart loopslot1/slot2 đều group1. ReturnedStart slot1 group1, **slot2 group0**,
703A90return1,1240D70return0, actualPC**DCB7DA**→DCAD80return3/error trước104.
Raw packedword phần cao cònopaque; chỉ group `(word>>2)&7` đã đọc, không gọi toànword là enum.

Slot2 là actor đối địch tổng hợp, không có client thật phát Ready. Phép thử kế là bounded diagnostic
roster refresh110/slot2/state0x24 saureturn từ layout đã có, có control/native guard0/actual104/
Start2freshrescue mới xác nhận fix. **Chưa chạy fix**, không suy law server gốc hoặc ghi Start2 đã qua.
Raw/hash/cleanup được lưu xong trước đợt migration; source/profile không đổi giữa run. CP118 chính là
refactor kho checkpoint; recordCP117/failedobserverraw giữ nguyên, gate/auditcounter không reset.


<a id="rescue-slot-refresh"></a>
### Diagnostic slot2 ready refresh mở lại Start2 — CP119

[Metadata/control/hash](../evidence/rescue-service/README.md#cp119). **design/emu_choice**: chỉ khi
`rescue_contact` đã bật với fixed-profile, sau return gửi thêm110/slot2/state0x24/tail0 tới hai peer,
ngoài110host/guest hiện có. Đây là ready của actor tổng hợp, không giả clientC hoặc actualC95.
Defaultoff không thêmgói. Không sửa stateclient/binary, không đổi profile/fuse/outcome.

**kr_runtime/confirmed hẹp**: controlCP118slot2group0/DCB7DAguard3 so với run này slot2group1,
roomguard0/build104 và actualA104 sauB95, R2native121155/clock/haipeer153key12345→12346.
Mộtresult/return hoàn tất rồiStart2, không hai trậnhoànchỉnh/gate. 191server tests qua.

**Phần chưa đạt giữ rõ:** R2B17→19, không18 trong bound dù có normalAUp; runexit1/
`current rescue integration not observed before bound`, serviceerrors0. Không gọi freshrescue/reset
đãgiải từ việc quaStart. ẢnhR2A/B sauchạm cònHELP và vịtríA khác nhau, cần đối chiếu nativecurrentXY/
input/peerapply, khôngsuy tọađộ hoặc nguyênnhân từảnh. Bước kế theo PLAN, không nới guard báo chết/
consume/report đểche lỗi. Scope2realWineAB+synthetic2; auth/authority/fullmatch/LANInternet chưađạt.


<a id="loading-observer-timing"></a>
### Đường load trước writer 99 và ảnh hưởng observer — CP152

**`kr_runtime/confirmed` cho các mốc đo trên copy Wine; `kr_static/confirmed` cho instruction đã đọc.**
[Recipe, từng run, hash và giới hạn](../evidence/peer-endpoints/README.md#cp152).
Tái dùng CP021 writer/once-guard, CP024 loader/spawn và CP030/062/150/151 claims/profile;
không RE lại codec, 124/165 hoặc thuật toán nhặt item. Production source/profile/deadline **30s**,
copy SHA `e9bb4766…` và measurement delay165 **2s** giữ nguyên trong từng run.

Trace đầu đo được writer `E7F2A0` vào với guard `+6410=0`, caller `E59587`, sau native121
**33,713s/33,835s** (A/B); send entry99 theo sau khoảng36ms. Cả hai channel đã đóng trước đó;
không có actualTCP99 sau121. Guard1 xuất hiện ở các tick sau, không phải guard chặn lần gửi đầu.
Đây là **R1 của run mới**, không xác định nguyên nhân R2 CP151. Dừng SIGINT có `finally` cleanup,
exit130; không ghi là runtime pass. Trace entry lặp195 lần/view được thay bằng cạnh build chỉ đi khi guard0.

Đọc có mục tiêu `E59550..E59597`: `E5955A` reset guard0, memset `+6414/0x68`, rồi gọi vtable`+308`
đưa tới writer đã biết. `E57C20` gọi `+2D0` trước reset/writer. Các vùng byte đã so khớp giữa
static SHA `660447…` và hai copy runtime; cửa sổ tuyến tính không phải toàn CFG/decompilation.
Dùng lại cache CP024 `E59F30/E5A290/E0F6C0` để đặt mốc trả về, tránh đọc lại các thân hàm này.

Run milestones kế có actual99/barrier R1 ở29,276s; riêng `+2D0` mất28,391/28,375s. Run dừng
`alignmiss:R1-A-contest-approach`:20 mẫu neutral dao động Y159↔214/234 so với199±12, chưa Fireclaim.
Run sau vẫn timeout ở R1: đoạn tới `E5A01A` khoảng18,286/18,215s, tới `E5A094` thêm11,139/11,301s,
khởi tạo actor tới `E5A0DE` thêm1,780/1,795s; sau đó reset/build/send. Thay tolerance staging12→20
**chưa được kiểm** trong run đó vì không tới clock; không báo là đã sửa input hoặc hoàn tất contest.

**Observer control:** `Chronology.stop()` trước clock vẫn bị gọi ở bốn state hook dù nó bỏ mọi
observation do `_chrono_ready!=round`. Đợt đối chứng giữ A nguyên, B chỉ tắt bốn breakpoint này
khi load và khôi phục tại nativeclock; ghi hit counter ở hai cạnh `+2D0`. Kết quả theo từng role:

| Role/vòng | State hook lúc load | Thời gian +2D0 | ActualTCP99 | Nativeclock |
|---|---|---:|---:|---|
| A/1 | bật, control | 32.291877s | 0 | không |
| B/1 | tắt 4, treatment | 27.302141s | 1 | không |

Cả A/B còn **929 hit tại breakpoint `B32390`** trong đoạn `+2D0`; script chỉ log khi actor thuộc
ba bomb pointer đang theo dõi, nhưng lúc load cả ba pointer bằng0. Đây là observer đã có từ trước;
không cần các stop đó cho gameplay vì chưa có bom. `gdb.breakpoints()` chỉ trả public sites trong
ca này, không trả hit counter của bốn Python internal hook; không suy counter internal bằng0.
B có actual99 trước hạn, A gửi muộn; cả hai chưa clock. Chưa kiểm đường khôi phục hook tại clock.
Hit counter là số stop debugger, không phải số lần game đổi state. A/B khác role và chỉ một ca đối
chứng; tương quan thời gian chưa là chứng minh toàn bộ nguyên nhân CP151 hoặc thời gian client nguyên bản.
Không force99/clock/guard, không gọi native function để vượt load. Các run trước giữ nguyên và không
ghép thành hai vòng hoàn tất. Tranh item vòng2/nativeReady2 vẫn cần đúng cùng-run proof; K6a/fullmatch,
LAN/Internet/8 client thật và authority gốc còn mở. [Phép thử tiếp](../RESEARCH.md#gap-map-spawn-loading),
[thứ tự duy nhất](../PLAN.md#current-order).


<a id="loading-hot-gate-contest"></a>
### Bỏ stop observer thừa khi load, tranh Fire qua hai key và hai kết quả — CP153

**`kr_runtime/confirmed` trong cùng run copy Wine**, [raw/recipe/hash](../evidence/peer-endpoints/README.md#cp153).
Tái dùng CP152 public hit counter929/view, CP021 writer/guard, CP024 loader, CP030/062/151 pickup
và CP150 profile/claims/directroute. Không thêm thân hàm native hoặc thay source binary/pack/DB.

Recipe mới chỉ gate breakpoint `B32390` từ Start121 tới nativeclock, cả hai view; ba pointer bom
được assert bằng0 trước gate. Trong `+2D0`, hit delta của breakpoint này bằng0; tại clock khôi phục
trạng thái enabled trước đó, rồi giữ quan sát gameplay. Bốn internalChronology state hook vẫn bật
như control r3 của CP152. Không gọi hàm game, ép99/clock, ghi guard/actor hoặc tăng deadline30s.
Production/profile/claim/consume/key giữ nguyên; delaypositive1652s vẫn là **design measurement**.

| View/vòng | +2D0 (s) | Actual99 trước hạn | Clock/restoreB32390 |
|---|---:|---|---|
| A/1 | 4.172151 | có | có |
| A/2 | 3.775976 | có | có |
| B/1 | 4.210599 | có | có |
| B/2 | 3.718414 | có | có |

Hai barrier lần lượt5,944500s và4,598209s; không deadline close/abort. Đây là thời gian của copy
có debugger/recipe này, không phải thời gian chuẩn client nguyên bản hoặc chứng minh toàn bộ nguyên
nhân CP151R2. CP152 giữ nguyên các run late99/alignmiss/control; không backfill bằng run mới.

NormalOS staging199/359±20 và DownA/UpB0.30s đã đi qua hai vòng; mỗi vòng cả A/B thực sự gửi124
cho Fire17/kind1/cell6,3/currentkey trước delivery165. B đến trước cả hai lần; A bị reject consumed.
Mỗi receiver chỉ một Fire165/full12/errors0/slot1; securegetter xác nhận **chỉ B power1→2**, A giữ1,
capacityA/B2 giữ. R2 trước pickup mới có baseline1, không mang power2 của R1 sang. Đây là policy
first-arrival của emulator và thí nghiệm hold165; chưa phục hồi luật fairness/auth/validator vị trí gốc.

| Vòng/key | Winner | Hold165 (s) | Loser tới trước delivery (s) | Bom native/current | Rank native A/B |
|---|---|---:|---:|---|---|
| 1/12345 | B | 2.012065 | 1.893150 | owner0/ID1/power1 | 2/1 |
| 2/12346 | B | 2.024936 | 1.882167 | owner1/ID2/power2 | 1/2 |

Cùng run:6 actualitemclaims được nối SHA/seq/body từ TCP capture (Bomb16A + haiFireclaim mỗi vòng),
4 pha peer input có actualnativeapply/target/currentcoordinateprogress; hai currenttrapSHA→19 và
opposite survivor21 trước chosenrank. Cả hai đọc đủ175302/1803/errors0 hai lần,110 sau return2 và
actualReady95/native110 cuối. Hai ảnh Ready đã xem: hai user KRLocal/KRGuest, AStart/BReady và guest
Ready badge. Lưới8 slot **không** là8 client thật. Không Start3/disconnect hoặc ca stale trong run này.

16 gate checks/84 contest checks/43 lifecycle checks/14 captureclaim checks;1155 loading assertions gồm
raw hashes;2484 captured secure fields và324 peer getters bằng native52A350 model;499 peer assertions/
4 windows. Model allocator/rekey-counter riêng như nguồn cũ, không phải native runtime authority.
Fullworld/physics/identity/malformed/stale/generalreset/CP137complexUI/K6a/fullmatch/LAN/Internet/
8 client thật vẫn mở. Hai epoch contest và recipe loading blocker đã giải trong scope này; parent issue14
và gate giữ mở. [PLAN](../PLAN.md#current-order) tiếp acceptance còn thiếu, đặc biệt actualpendingguestEOF
trên channel allTCPdirect thay route proxy CP144; [câu hỏi](../RESEARCH.md#gap-disconnect-replacement).


<a id="basic-preset-start3"></a>
### Preset basic được export: hai vòng và normal BStart3 — CP171

**`kr_runtime/confirmed` trong scope modified Wine/private virtual hosts; policy là `emu_choice/design`.**
[Recipe/raw/hash](../evidence/peer-endpoints/README.md#cp171). [PR242](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/242)
thêm `two-client-basic`, pin exact profile CP170/47 fixture; 46 alias như default. CLI chọn preset,
rebase path, giữ size/SHA/sequence/policy; default `two-client-battle` và legacy export không đổi.
Sáu exporter tests và real fixture/production constructor preflight chỉ là kiểm implementation.

Fresh run dùng **chính file export** qua `python -m server.probe_service`, hai CA/wineserver/private
hosts, cùng8TCP; source/config giữ nguyên qua cleanup/seal. 402,663072569s/exit0/1443raw/156frozen.
Hai vòng Bhost sau idle ABack/promotion/rejoin; actual121/99/barriers6,277452087s và6,237059530s
trước30s; keys12345→12346, bốn A-owner bomb IDs1..4/power1,2,1,2. Mỗi vòng reveal27/28/actual142
trước124/165full12/errors0/lookup đúng CGOItem; order[17,28,27] và[17,27,28]. ChỉA có Roller5→6,
capacity1→2/Fire1→2; baseline A1/1/5 và B2/1/4 tái lập ở epoch2, cùng hiddeneligibility và195 seed.
Native159/158 samepointer/currentID/full39/full13/all195 flags được đối chiếu như CP170.

Hai native175/180/result/room/Ready; cảhai view native life56→1/other0 trước175, không global tick.
Sau actualReadyA95/native110both, normal BStart tạo **actual104 thứ ba**, key12347/native121/99,
barrier6,326163042s trước30s, cảhai native clock chạy thêm9,146193006s trước completion. Không bom
hoặc outcome trận ba. Cleanup EOF/abort sau completion0,401806744s, không được coi disconnect control.
Hai ảnh cuối đã xem có map/actor riêng/timer01:50; ảnh riêng không thay native execution/consensus.

1741 first-two checks gồm rawhash/native joins,180 captured-native packet cases/552 securefield models;
life23joins/16nativegetters;417 exported-file/Start3/stream/observer checks. Năm alignmentgroups/
10view-phase ACK, bốn rare165+lookup probes giữ enabled và exact restoration; bounded modegetter
observations1, không trace toàn setter/mode. Số group phụ thuộc đường OS input thực tế; không hạ
bound/đổi target hoặc giả request để đạt. VN/KR reuse vẫn theo CP170, không mới opcode/body/layout.

Phân tích ban đầu phát hiện parser gán `CAP_SPACE_SNAPSHOT` vào lastGEN165_END do event/phase vẫn
giữ giá trị pickup cuối. Thêm marker ngữ cảnh Space để tách đúng nguồn; giữ **mọi** snapshot/native
getter/checksum và criterion unique GEN165 effect. Initialscript/log/fields và correction giữ trong raw
analysis; không sửa runtime/raw/expectedvalues để pass. Assertion độc lập Start3 cũng sửa boundary:
chỉ cấm abort trước completion, không nhầm intentional cleanup EOF là active-run abort.

Đã giải optional preset/export/nativeBStart3 trong scope này; không gộp profile cũ thành acceptance.
Fullauth/drop/distance/142freshness/generalreset/Windows/LAN/Internet/8actualclients/K5/K6a/fullmatch/
dependencies vẫn mở. Tiếp [PLAN](../PLAN.md#current-order): guard142 đúng current clearedcell là policy
candidate có thể tận dụng terrain model/native195 đã kiểm;8actual142 đều có currentcellclear trước nó.
142 không có recovered roundkey; cùng oldbyte sau đúng currentclear vẫn indistinguishable, không
tự tuyên bố fullfreshness/auth. Gate không tự đóng, audit kế CP172.


<a id="a-c-first-bomb"></a>
### Cslot1 thực phát118 và A/C nhận cùng bom/nổ — CP176 (run đơn)

Run đơn C bom đầu dưới đây được gom với hai vòng A/C vào CP176; phạm vi từng run giữ riêng.
[Recipe/raw/hash](../evidence/peer-endpoints/README.md#cp176-in-progress). Tái dùng đúng entry prestart,
C ID3/slot1/native initial2/profile2/7 và toàn bộ codec/159/158 đã có; không mở hàm native mới.
VN item families/actor-slot checklist và KR legacy đã đối chiếu, không chuyển opcode/layout/ID.

`kr_runtime/confirmed` hẹp: normal CSpace0.4/Right0.8 sau8s clock, PID/startticks/namespace/mapped
image được kiểm lại trước input. C phát118 slot1/fullkey12345/serial1, service nhận một bom
ID1/owner1/row9-col3/power1;159 và158 chỉ tới A/C. C2S sequence/CRC/typed fields qua native oracle,
hai client tạo entity và xử lý cùng ID1 đúng một lần;23 relationship checks/33native packet cases.
Khóa118 là field đầu, không nhầm với serial/tail. Fuse6s là policy giữ nguyên, quan sát tới due+4s.
Run197.706s/626raw đã cleanup và seal; không C17/19 trong scope quan sát nhưng chưa dùng điều đó
làm bằng chứng độc lập rằng C sống/di chuyển đúng. Generic parser cũ đòi dynamic_s2c của B đã rời
và verifier ban đầu đọc nhầm locator key; đã sửa analyzer theo actual A/C và contract118 đã biết,
giữ log lỗi cũ, không sửa raw hoặc chạy lại client để làm report pass.

Đây là C bom đầu, không kiểm maximum7/hai reservation/giải phóng capacity, hai chiều peer,
A+C result/reset/vòng hai hoặc gate. Bước kế ở [PLAN](../PLAN.md#current-order): tái dùng entry nhanh,
known A hidden-route/bom/effects/life và current Cslot1, gom hai vòng normal/Ready/reset hoặc blocker
thực tế rồi hoàn tất CP176 và audit định kỳ. CpreReady render vẫn unknown từ CP175.


<!-- protocol-topic-content-end -->
