# Item, stat, pickup và kim

[Mục lục](../PROTOCOL.md) · [Câu hỏi đang mở](../RESEARCH.md) · [Thứ tự việc](../PLAN.md#dashboard).

<a id="item-families"></a>
### Nhóm item và kiến thức tái sử dụng — CP045

Đối chiếu theo câu hỏi Battle cơ bản; bảng dưới không là kiểm kê mọi cơ chế trong client.
Nguồn/hash/locator và bốn câu hỏi tái sử dụng tại [evidence](../evidence/item-families/README.md).
ItemAttr KR335/VN269 đã parse từ CP037:269 tên chung,267 hàng giống group/field160/five-tail/weight;
hai hàng khác là KR247/VN213 `ItemCpGold`, KR248/VN214 `ItemCpPocket`. Cùng field không chứng minh
cùng hiệu ứng. `boom-re-l10n/decomp/item_kinds.json` là cùng bảng VN269, ba số−1 khác biểu diễn
unsignedFFFFFFFF; không coi là ba cơ chế mới hoặc một nguồn runtime độc lập.

`boom-rs` R169–185 bổ sung **VN static/runtime confirmed** cho Roller: selector3, tăng1/kẹp max,
base/max/current, movement amount và đường cong đo trên actor cụ thể. Dùng lại kiến thức và cách kiểm;
offset/ID/consumer/wire KR vẫn phải đối chiếu. `oc_e-game.md:454` ghi unknown cũ; dòng463 đã có
Bomb/Fire runtime, evidence R169–185 mới hơn đã có Roller. Không đọc unknown lịch sử thành hiện trạng.
`archive/KNOWLEDGE.md` và bundle là bản đồ nguồn; câu tóm tắt33px/s/cấp hoặc fuse2,5s không thay
giới hạn capture gốc. `boom-rs` wire/authority/scope V1 là design khác project này.

| Nhóm / ví dụ KR↔VN | Đã biết từ VN | KR static/emulation | KR runtime / phần còn thiếu cho demo dùng nhóm đó |
|---|---|---|---|
| Stat tích lũy, group1: Bomb0/Fire1/Roller2 |Bomb/Fire effect và nhiều bom; R169–185 Roller+1 logical point/kẹp max/reset/consumer theo fixture VN |1192700/group1 gọi1191840 cho **bốn** selector đầu; fifth-tail không phải selector thứ5. Effect1→vf4,2→vf8,3→vf14; CP031 increment/clamp; Roller B48350 current iface+5C/max+50 |Bomb/Fire đã [đo stat/peer](#item-effects), [reset hẹp](#hidden-items). CP045 Roller5→6/max7 cả hai view, đầu vòng2 về5 rồi nhặt lên6; không chuyển rate VN thành px/s KR. Không tiêu hao kiểu Ctrl; item nền bị xóa sau165. |
| Ability/debuff cùng group1: Shoe3/Punch4/MaxPower5, Hollow6, Skeleton7/8, MoonWalk9, Superman10, GreenDevil11 |Tên/selector và render đã có; tránh suy Shoe là tăng speed, Roller mới là nhánh tăng point đã đo |Selector4/5/6 dùng các virtual khác ở1191840; không RE lại124/165 cho từng bản ghi |CP045 MaxPower1→7/max7 và118/159power7, reset đầu vòng2 về1. Ném/đá, immunity, thời hạn/debuff/mode cần consumer/case riêng trước dùng. Tên/icon hoặc số0 trong table không chứng minh vô hạn. |
| Mount/biến hình group2: Airship40↔34/Owl41↔35/Turtle42↔36… |Table và tên/variant có sẵn |Group2 là nhánh khác11928A2; không áp increment group1 |Chưa runtime KR trong đợt này; exclude khỏi fixture Battle tối thiểu theo policy thử, không gọi client không hỗ trợ. |
| Dùng chủ động group23: Needle145↔117 và12 hàng KR cùng group |Kim Ctrl sau trap, dùng cạn; R169–185 không là bằng chứng kim |1190C20 amount>0 rồi nhánhgroup23/vf1C; variant cùng group là ứng viên tái dùng đường |[Kim145](life-results.md#needle-selfescape)/[lần hai](life-results.md#needle-repeat): pickup124/165→Ctrl×1→actual127/1 rồi0(kind145), peer170/1 đưa56→26; cạn không thoát lần hai. Inventory validator, multi-charge/variant/round-reset toàn túi chưa đo. |
| Chủ động khác: Dart143↔115(g21), Oxygen144↔116(g22), Potion147↔119(g25), Spring148↔120(g26), Drill149↔121(g27), RemoteBomb150↔122(g28) |Table và đường Ctrl/nhận xét cũ; không runtime từng loại |Group/field160 chỉ phân loại dữ liệu; không gọi field160 luôn là số lần dùng |Điều kiện, effect, tiêu hao/report0/confirm, peer và reset chưa được kiểm theo từng nhóm; dùng Needle làm phương pháp, không copy kết quả. |
| Có thời hạn/variant: Shield151↔123(g29,raw2000/1000), Sensor152↔124(g30,raw30000/5000), SJNeedle161↔133(g46) |Shield được ghi2000ms là locator/hypothesis; SJ/needle variants khác group |Raw tail giống không đủ xác nhận đơn vị, duration hoặc cùng nhánh145; cần consumer timer/group |Chưa runtime KR duration/expiration/consume/peer/reset; không gán max/giây từ tên/field. |
| Mode/event/reward: HPNeedle17(g1/effect27), GoldFlag19(effect99), Lucci67↔50(g99), HCHint300↔266(g1000), RPG… |Catalog cũ cho phép tránh nhầm HPNeedle17 với Needle145 |Cùng tên “Needle” không cùng group/effect; R153 loader/group table khác biệt với effectconsumer |Không đưa vào ca Battle cơ bản hiện hành; mode guard/serverreward/variants còn unknown, không RE mọi row trước K6a. |

Nhóm pickup dùng đường [582/124/165](#item-pickup) đã đo; đối chiếu KR indexu32, không dùng VN indexu16.
Confirm broadcast có CRC/cursor đúng chưa đủ chứng minh effect; đo securefield/native consumer/peer và
đầu vòng kế. Placement/drop/first-arrival/fuse của stub là `emu_choice/design`, weight không là phần trăm.
Nguồn R150 cũ gọi E9E660 là item path đã bị [CP030 sửa](#item-pickup): đó là bom kind0;
R153 loader/table vẫn dùng được, không lặp tìm pickup/stat từ nhánh nổ.

**Delta KR runtime CP045, hai run riêng `roller-rounds`/`maxpower-rounds`:** mỗi run hai client thật,
hai Start/clock trên cùng8connections/IDs1,2/slots0,1/session1. Mỗi vòng một item nền index17/hàng4/
cột3, A Down tới actual124, ItemClaims/current165 phát cả hai, không chèn gói cũ hoặc sửa state.
Sau nhặt A Space thật, bom nổ→A17/19→170 peerstate1 trước175/180/phòng hai vòng. Placement,
catalog/fuse6s/rank/first-arrival là policy emulator, không luật server gốc.

- Roller2: actual165full12 →1191840 effect3/0/0/0, receiver actor+58C/vt17760BC →B48350.
  Secure **actor+5E8/current5→6**, **+5DC/max7→7** trên cả A/B; capacity/Fire/max và stat slot1
  được chụp giữ nguyên. Native B48350 dùng636620 tăng1 rồi so current với max, vượt thì54B610 chép max;
  năm ca emulation giữ machinecode tăng/kẹp, chỉ model local gateAD5BA0=false. Runtime tại max chưa đo.
  Cặp field/consumer tương ứng VN R173 nhưng không copy offset658/64C hay định lượng px/s từ VN.
- MaxPower5: effect6/0/0/0 →vfC/B472A0. Secure **actor+4BC/power1→7** và **+524/max7→7**
  cả hai; capacity/movement được chụp giữ nguyên. Actual118.field10=7,159.field8=7 ở hai vòng;
  không suy capacity/speed cũng lên max hoặc mọi feature branch từ tên MaxPower.
- Clock vòng2 reset current Roller về5 và power về1 trước pickup mới; sau nhặt lặp cùng delta.
  Hai run có hai175outer302/inner292 và180full3 mỗi client. Đây là reset của field đã đo, không
  toàn túi/entity/UDP hoặc kiểm mọi item. Không có input Ctrl/127sub0 trong hai ca này; chưa đo TTL/
  expiration trong cùng vòng. Không gán “vĩnh viễn” từ việc giữ qua các mốc hoặc table tail0.

<a id="item-pickup"></a>
### Item trên nền và xác nhận nhặt — CP030

**kr_static/kr_runtime confirmed trong phạm vi copy Wine**; [recipe/control/hash](../evidence/item-pickup/README.md).
Giữ cùng binary chẩn đoán SHAe9bb47… và profile ExVillage01/278, hai identity/slot riêng, không patch thêm.
VN hướng dẫn cách thử layout trước start và report→confirm; opcode, index32bit và reader KR dưới đây
được phục hồi riêng. Không áp VN553/117/158 hoặc index16bit sang KR.

**Layout trước load:** prepare dispatcher `DC51F0→DC5378`, **S→C582**, đọc serialized blob rồi
`D824D0`. Guard `5725B0` thực trả1; `11ED590` đọc từng record. Inner sau giải blob gồm:

```text
u16LE n1; n1 × {u32LE index, u8 row, u8 column, u32LE kind}
u16LE n2; n2 × cùng record
u16LE n3; n3 × cùng record
```

Tại CP030 chỉ list1 là item trên nền đã quan sát; list2/3 đều rỗng trong các run đó.
CP032 kiểm tiếp [list2 giấu trong gạch](#hidden-items); list3 vẫn chưa có ca nonempty.
Mẫu hai item tiêu26byte inner/36byte payload; mẫu một item tiêu16/26byte, errors0 cả hai.
Framer/blob/getters và record reader native kiểm độc lập; runtime guard/list install/map render là
nguồn riêng. Tag raw0 trong recipe đã được reader chấp nhận, không phải capture server gốc.
Item record dùng hàng/cột, khác point spawn map có quy ước riêng; không chuyển nhãn tọa độ giữa hai kiểu.
Kind0 vẽ Bomb, kind1 vẽ Fire ở profile này; icon không chứng minh đã cộng số bom/tầm nổ.

**Request/confirm:** actual **C→S124** từ `E2E390` (stack thực `E2E5EC`) có19byte:

```text
u8 ctx, u8 slot, u8 row, u8 column, u32 index, u16 kind, u32 t, u32 k, bool flag
```

Đây là typed outer fields qua primitive KR, không phải raw LE của inner blob. Writer lấy record+0,
slot từ argument, record+0C/+10/+08/+14/+1C/+20/+24 tương ứng. Nguồn `t` đã phục hồi ở
[CP055](#pickup-tick); `k` có [guard hẹp CP049](#pickup-round-key), không là auth. Ca CP030 ctx0, flag0, k12345 do profile peer đã cấp.
Native replay actual frame theo seq/socket xác nhận CRC, độ dài và giá trị; không dựng request từ encoder.

**S→C165**: `E5D570→E63260(packet,0)`, payload12byte:

```text
u8 ok, u8 ctx, u8 slot, u8 row, u8 column, u32 index, u16 kind, bool flag
```

`E2C660` xóa pending theo ctx/index trước kiểm ok/context. Native reject/context-mismatch chỉ kiểm
header/nhánh từ chối với cache/world model; positive effect lấy từ runtime. `ok!=0` và context khớp
mới đi tiếp lookup actor theo slot; interface actor+588 gọi `F73D30`, sau đó tìm item, kiểm tọa độ/kind
trước phần xử lý tiếp. Cạnh tĩnh `F73D30→B52840→E14DA0` tại CP030 mới là mỏ neo; CP031 đã nối
và đo [stat Bomb/Fire cùng giới hạn](#item-effects).
Không gửi nguyên124 làm reply: thêm ok, bỏ haiu32 t/k, mã hóa lại165 theo seq của từng receiver.

**Control và can thiệp:** `near-control` có A gửi124/index17/row4/col3/kind0 nhưng không165;
item hiện lại sau khi rời ô. `pickup-ack` giữ input/placement A đó; B được đặt Fire tại ô hàng8/cột3
thay vị trí gần gạch ở control. Cả B rồi A gửi124 thật; mỗi xác nhận165 phát tới cả hai, đọc12/12/errors0,
context0 khớp, đúng actor/item/tọa độ/kind; pending local được xóa và ảnh cuối không còn hai item.
Đây là can thiệp có đối chứng cho A; B đổi placement nên không tuyên bố một biến duy nhất cho cả run.

**Một ca tranh index:** `pickup-race` đặt Bomb/index19 ở hàng6/cột3, input Down(A)/Up(B) đồng thời.
Cả hai gửi124 cho cùng index trước ACK; stub giữ request đến trước, chờ2s rồi chỉ phát một positive165
slot0/index19 tới A/B. Request B ghi bị từ chối do đã giữ index. Cả hai có một callback165 đầy đủ,
cùng slot thắng, xóa pending của chính mình và không còn item ở ảnh cuối. Đã kiểm một ca/two requests;
chưa đảo thứ tự winner, duplicate replay, stale/round generation, stat chỉ cộng một lần hoặc mọi race.

| Vai trò cho hành vi đã dùng | Bằng chứng/giới hạn |
|---|---|
| Creator | Stub chọn layout/index/kind; client tự phát124 khi chạm item. Không phục hồi thuật toán drop server gốc. |
| Simulator | Client dựng item, giữ pending và xử lý165/interface. Tại CP030 chưa đo stat/cap; kết quả Bomb/Fire mới ở [CP031](#item-effects). |
| Validator | Harness kiểm CRC, slot theo origin và allowlist index/tọa độ/kind; giữ một index cho request đầu. Đây là design hẹp, thiếu kiểm room/round/generation/timing đầy đủ. |
| Broadcaster | Stub mã hóa165 riêng từng socket và gửi cả origin lẫn peer; hai consumer đã quan sát. |
| Peer-editable state | Report124 chỉ được chấp nhận theo slot gắn socket trong harness. Toàn grammar/quyền UDP và chống sửa state ngoài đường này còn unknown. |

Placement/first-arrival/delay2s là **emu_choice/design**, không luật server gốc. Stat và duplicate sau
xóa được kiểm tiếp hẹp ở [CP031](#item-effects); item ẩn/phá block/drop, variants, malformed/stale/
reset/disconnect và full auth/cache/codec còn mở. Không ghép riêng mốc này với CP029 thành nghiệm thu
full match/K5/K6a hoặc LAN/Internet.

<a id="current-pickup-race"></a>
### Tranh index thật trên serializedservice — CP062

**kr_runtime/confirmed hẹp**, [recipe/hash/giới hạn](../evidence/current-pickup-race/README.md).
Tái dùng CP030race, CP061profile41/floor17/Roller/overlapspawn/currentkey và ItemClaims; không newwire/
nativebody/patch/servicecode. NormalUI A/B cùngDown0.30s qua barrier, không mintreport bằng encoder.
Wrapperlocal chỉ giữ **delivery165 trong2s**, productionclaim/consume/key/reportguard/winnerlogic nguyên.
Đây là canthiệp đo để cả hai nativewriter có thời gian gửi, không latency/arbitrationpolicy servergốc.

Cùng8connections/2cycles: mỗi vòng **hai actualTCP124** index17/hàng4/cột3/kind2/currentkey,
loser đến trước165delivery khoảng1.92s/v1 và1.96s/v2. Production first-arrivalclaim thắng:

| Vòng/key | Request tới trước / sau | Native165 cùngslot trênA/B | SecureRoller trước→sau ở cả2view |
|---|---|---|---|
|1/12345|A / B|một165/receiver,slot0|A5→6, B4→4|
|2/12346|B / A|một165/receiver,slot1|A5→5, B4→5|

Không có165cho requestloser; cùngpayload positive165 gửi origin/peer một lần, callback12/errors0.
Winner đảo tự nhiên theo actualarrival, không forcewinner. Clockv2/statbefore165 trởbaseline A5/B4;
claim reusedindex17 còn nhận được trước khi reserve/consume mới. Hai bom/stateA1/peerstate trước175,
2rank/return/room; không auth/fullvalidator/whole reset hoặc K6a/LAN/Internet từ sốcallback.

Production claim được consume **trước** queue165; wrapper flush sau2s chỉ nếu room/generation hiện hành,
discard khácvòng nhưng ca discard chưa runtime riêng. Không thay firstarrival bằng công bằng/thờiđiểm
va chạm tuyệt đối; normalUIbarrier không chứng minh physics cùngtick. CP030oldharness và CP062current
service là hai scope có liên kết, không nâng mọi duplicate/stale/transport/mode biến thể. Khoảng trống report127 của CP062 được tiếp ở [CP063](udp-peer.md#report127-freshness); thứ tự hiện hành chỉ ở [PLAN](../PLAN.md#current-order).

<a id="distinct-spawn-race"></a>
### Tranh item thật giữa hai spawn riêng — CP066

**kr_runtime/confirmed hẹp**, [recipe/raw/hash](../evidence/distinct-spawn-race/README.md).
Tái dùng CP030 midpoint/Down-Up1s, CP062 claim/deliveryhold/nativeeffect, CP065 currentprofile/
mapspawn0/1. Không thêm body native/patch/productionservicecode. Chỉ datafixture582 và catalog
row4→6, giữ index17/col3/kind2; native11ED590 kiểm fullinner16/1floorrecord/2list rỗng, không dropalgorithm.

NormalUI ADown/BUp1s qua barrier, không mintrequest; wrapper giữ outgoingpositive1652s nhưCP062,
productionclaim/consume/currentkey/reportguard không đổi. Cùngrun8connections,2Start/clock và native
spawn/reset A120159/B120399 cảhai view, hai key12345→12346. Mỗi vòng có **hai actualTCP124** cùng
index17/row6/col3/kind2/currentkey trước delivery165. Firstarrival claim thắng một lần, loser không165;
secureRoller chỉwinner+1 cảhai view, vòng2reset baseline A5/B4 trước nhặt mới. Winner/order cụ thể ở
bảng evidence, không force đảo winner hoặc gọi đồng bộphím là cùng physicstick/fairnessservergốc.

Sau165, normalUIBRight/Aspace cho hai bom/17/19→B170/B21/stateA1bothbefore175/hairesult/return/room.
Đây là same-run integration của ca tranhitem/distinctspawn/two-cycle đã đo, không đủ fullmovement/
transportcausality/actorauth/distancevalidator/fullreset/K6a/LAN/Internet. Delay2s là canthiệp đo;
không suy productionlatency hoặc TCP replay admission. Wrapperdiscardqueued165khácgeneration chưa
runtime riêng, policytiming/fuse/rank/placement/authfixture vẫn design.

Tại CP066, C→S118 admission còn thiếu; [CP067](bombs.md#c2s118-admission) đã đo control/fix, không coi
oldS→C159/158 hoặc k124 là proofrequest auth. Thứ tự hiện hành chỉ ở PLAN.

<a id="pickup-round-key"></a>
### Guard124 theo khóa đã cấp — CP049

[Schema124/165 và consumer](#item-pickup) giữ nguyên. [Recipe/raw/hash/controls](../evidence/stale124/README.md)
tái dùng CP030/042/047/048 và nguồn VN, không phục hồi lại receiver hoặc đổi layout/opcode theo VN.

`probe_service.receive_channel` kiểm **u32 k (field7)** bằng khóa đang cấp ở u32 thứ hai153 **trước
ItemClaims.claim/consume**. Khác khóa: ghi `round_key_mismatch`, không phát165/không chiếm claim.
Khớp khóa: tiếp origin/context/catalog/first-winner/one-consume hiện có. So sánh cảu32 là lựa chọn
`design`, không sao chép quy tắc low8 của [movement](udp-peer.md#udp-generation) hoặc gọi k là auth.

**kr_runtime/confirmed hẹp trên Wine copy:** control service CP048 nhận lại body actual124 vòng1
(k12345) qua dispatcher ở vòng2/key12346 rồi phát165 trước input mới; native secure Roller5→6
cả A/B, không actual124 vòng2. Service có guard loại body cũ; sau input **actual TCP124/k12346**
vẫn phát165, callback12/12/errors0 và Roller5→6 cả hai. Clock vòng2 về5; hai run đều có hai
bom/17/19/peerstate1 trước175/180/phòng. Điều này chứng minh consequence của dispatch/165 và guard
hẹp; body chèn bypass C2S transport, **chưa kiểm TCP admission của replay**. Không giả nó là request
mới do native writer phát. Hai proxy setup dừng trước phép đo được giữ riêng, nguyên nhân unknown.

Không còn coi “service chưa kiểm k” là trạng thái hiện hành. [Nguồn t](#pickup-tick) đã giải ở CP055;
policy dùng t, distance/actorstate, request giả đã
đổi k hiện tại, mode/variant, key lifetime/roster/otherUDP và fullvalidator vẫn mở. Equality không xác
thực sender; fuse/drop/rank/auth/profile còn fixture. Không gate K5/K6a/LAN/Internet mới đóng.


<a id="pickup-tick"></a>
### Nguồn và lifetime của t trong124 — CP055

**kr_static/kr_runtime confirmed trong scope Wine copy**, [recipe/hash/giới hạn](../evidence/pickup-time/README.md).
Tái dùng VN time/session producer làm hướng truy nguồn và cached KR writer, không chuyển địa chỉ/đơn vị VN.
`t` là **bộ đếm cập nhật logic riêng của CMainSystem**, chốt khi tạo pending record; không là đồng hồ
server hoặc số mili giây tính từ đầu trận. Physical timebase/đơn vị của nguồn thời gian vẫn unknown.

Native `E2C450` gọi `E2F2D0→60FDE0(global19D9920)→12580B0(+100)→123DD10(+10)→65A340→65A5D0`;
protected member **CMainSystem+110**. RTTI cùng hash runtime: vtable179234C, COL18076F4,
type descriptor19D089C, `.?AVCMainSystem@@`. `12580B0` đã có trong cache bootstrap cũ; CP055 nối nó
với124. Getter khi globalnull trả0. Constructor `E36F20` ghi u32t vào record+1C, k vào+20;
writer `E2E390/E2E583` đọc lại; actual TCP124 giữ cùng giá trị. Temporary stack record được copy vào
vector heap, không đòi pointer giống nhau. Gọi collision lặp không nhất thiết tạo thêm record.

Setter stack runtime phân biệt tăng thực (`D54743←123DCAB←1256B3B`) với periodic getter rekey
(`65A734`, giữ giá trị). Native `123DC20`: unsigned delta chia30, số bước xử lý kẹp15,
flag timer+20 có thể kẹp1; counter timer+10 cộng số bước đã xử lý, reference timer+4 tiến30×quotient
**chưa kẹp**.17 ca emulation kiểm arithmetic/wrap với time source/protected storage/callback model;
không chứng minh full scheduler hoặc30ms/33Hz. Counter/rekey globals chỉ được model/reset trong
emulation, không sửa runtime. Pointer timebase vào Wine kernelbase chưa khớp exact export.

| Run (hai vòng liên tiếp) | actual A124 t/k vòng1 | vòng2 |
|---|---|---|
| clock-trace |2215/12345|4365/12346|
| clock-late (input đầu4→12s) |2558/12345|4698/12346|

Trong từng run/role, cùng CMainSystem tiếp tục qua Start vòng2; A/B có counter khác nhau tại
Start/clock tương ứng. Chỉ chứng minh lifetime qua hai vòng trong cùng process, chưa logout/reconnect
hoặc mọi reset.45 snapshot native getter,7 constructor cases,4 chuỗi init→record→writer→TCP;
context của snapshot là tag **trước đó**, không đếm tag context lặp thành writer mới.

Service chỉ log `client_tick` cùng request; không thêm ngưỡng thời gian/tự nhận auth hay chống replay.
Hai run dùng source CP054 lưu `runtime-source/` **trước** thay đổi logging; dispatcher bench với
bốn actual frame kiểm code mới, broadcast sockets stubbed, không gọi bench là runtime mới.
[Guard k](#pickup-round-key) giữ nguyên. Original-server validator/time policy, distance/actorstate,
forged-current-key và TCP replay admission còn mở; bước kiểm ở [RESEARCH](../RESEARCH.md#gap-floor-pickup).

<a id="pickup-report-state"></a>
### Guard124 theo report nhốt/chết của fixture — CP056

[Recipe/hash/control/giới hạn](../evidence/pickup-state/README.md); nguồn `kr_runtime/confirmed` cho
consequence trên Wine copy, **`design`** cho policy service. Tái dùng124/165,17/19→170, securestate/
Roller và [t/key](#pickup-tick); không RE lại login/item IDs/clock hoặc mang layout VN sang KR.

**Native tĩnh có mục tiêu:** caller `B77300` (subobject actor+584) tìm floor item tại ô actor rồi
kiểm `B49FE0` (protected actor+618) trước `B7B440→E2C450`. `B7B440` entry `B323D0` đọc **item+1C**,
`B8C970` kiểm item properties; vf390 trên vtable actor1775B40 là `55FE90` trảtrue.
Không gọi các guard này là enum sống/chết của actor. Runtime alive có +618=0; chưa nối field này với
nhốt/chết hoặc toàn eligibility upstream. `B40F30` tại mainvf108(-1) xét các khoảng27..30/39..42;
chưa chứng minh toàn update/state rule. Static bytes8body chính khớp runtimecopy; cạnh vf108 bổ sung
được pin riêng. Candidate khác EE5A30/1124C40 không mở rộng thành contract mọi class/mode.

**Control `state-control2`:** giữ item17/Roller ở4,3; thêm spare18/19 ở8,7/8,8, ngoài đường input/bom.
ActualA124 nhặt17→165→Roller5→6. Sau **actual127/17**, securestateA56: chèn request từ body CP055
nhưng **đổi index/hàng/cột và k hiện tại**, giữ t2215, tại production dispatcher. Service CP055 nhận
index18/phát165; native callback12/12/errors0 và Roller6→7 **cả A/B**, lặp cả hai vòng.
Sau actual19/state1, index19 cũng được nhận/phát165; Roller đã max7 nên7→7, **không** có proof tăng stat
ở actor chết. Không coi đây là gói mới do native writer phát hoặc replay qua TCP; request chèn cách xa
actor cũng chưa tách riêng distance khỏi eligibility. Chỉ chứng minh service dựa key/catalog vẫn nhận nó.

**Policy mới:** sau report17 hợp lệ trong layout fixture hoặc19 từ slot khớp connection, service ghi
`pickup_ineligible` cùng generation/report SHA/sequence. Sau [guard k](#pickup-round-key), trước claim/
consume,124 từ connection đó bị `actor_report_ineligible`, không165/không chiếm item. Marker không phụ
thuộc có peer để forward; Start kế xóa nó. Đây là report-derived/conservative fixture policy, không
state authority/auth hoặc full original validator. Fixture Roller hiện chưa hỗ trợ rescue/self-escape;
không áp marker chỉ xóa ở Start sang service kim/cứu mà chưa nối report1/18 và kiểm transition.

**`state-guard`:** cùng bốn injected frame/SHA/keys12345/12346, cả bốn bị loại; không callback165 cho
spare18/19. Input vòng1 không tạo actual124 (nguyên nhân chưa gán cho guard); vòng2 có **actualTCP124**
index17/key12346, native165 full12 và Roller5→6 cả A/B, chứng minh reset marker trong scope này.
Hai run đều2Start/clock/175outer302/180full3, stateA1 trước175 cả hai view. Không hai trận nghiệm thu.

Wrapper measurementv1 bị outer127 ghi đè provenance label của injection; final verifier nối **exact
injection SHA/fields/trigger**,2 actual124 ở control/1 ở fixed vòng2; raw không sửa. V1 source đã archive;
v2 chỉ sửa setdefault label, chưa runtime.5regression + actual-capture dispatcher bench kiểm peer còn
claim được item bị A từ chối, keyguard trước actor và Start/reset. Không dùng số test đóng gate.
Distance/auth/transport/malformed/stale127 hoặc report giả, rescue/self-escape/general eligibility vẫn mở.

<a id="item-effects"></a>
### Bomb/Fire: stat, giới hạn và nguồn power — CP031

**kr_runtime/kr_static confirmed trong phạm vi diagnostic copy Wine**;
[control/native/snapshot/hash và audit](../evidence/item-effects/README.md). CP030 mới kiểm layout/pickup;
CP031 theo cạnh đang chạy, không thay binary hoặc ép stat bằng debugger.

**Đường thực thi:**165→`F73D30→B52840→E14DA0→1190A40`; custom-effect getter trả0,
đi `1192700`. Group1 chọn `11927AE`, gọi `1191840` bốn effect number. Bomb/kind0 có effect1,
Fire/kind1 có effect2; ba số còn lại là0 ở hai loại này. Receiver runtime là actor+58C,
vtable17760BC. Effect1→v4/B47180→context actor+4A0/vA4/**AF06F0**;
effect2→v8/B47210→cùng context/vA0/**AF0690**. Không lấy interface+588 của165 làm receiver stat.

| Stat secure của actor | Offset wrapper | Phạm vi đã đo |
|---|---|---|
| Capacity hiện tại / max |+4A4 / +50C |Slot0 ban đầu1/10, sau Bomb2/10; slot1 giữ2/7 |
| Fire/power hiện tại / max |+4BC / +524 |Slot1 ban đầu1/9, sau Fire2/9; slot0 giữ1/7 |

Hai client chụp cùng giá trị trước/sau từng165 và lúc clock bắt đầu; checksum giải độc lập rồi gọi
native `529C60→52A350` trên wrapper/data đã capture. Chỉ profile hai slot hiện tại; chưa nối tên
asset/character tới mọi base/max. Đây là phạm vi CP031; Roller/MaxPower được bổ sung tại
[CP045](#item-families), reset stat hẹp ở [CP032](#hidden-items). Shoe và toàn reset vẫn chưa đo.

**Phép tăng/kẹp native:** AF06F0 dùng context+4/+6C; AF0690 dùng+1C/+84.
`636620` đọc secure current rồi cộng1/ghi bằng52A860; sau đó so signed current với max.
Nếu vượt, `54B610` chép max về current và trả0; trong giới hạn trả1. Mười ca emulation chạy code
increment/getter/setter/clamp/RNG thật với context/box storage tổng hợp, gồm tại max và current>max.
Counters/callback định kỳ của instrumentation được khởi tạo0; chưa mô phỏng rollover các counters.
Đây là **emulation** của nhánh kẹp; runtime mới kiểm pickup dưới max, chưa ca nhặt dư tại max.

**Control hành vi:** cùng layout hai item và ba lần bấm Space xen Right cho mỗi slot, fuse thử30s
để mọi input kết thúc trước158 đầu. Không165: A có một118, B có hai; có165: A có hai, B vẫn hai.
Mọi bom có actual118/native replay và159 trả entity ở cả hai. Không lấy việc thiếu request thứ ba
riêng lẻ làm proof trần; snapshot stat và nhánh kẹp là các bằng chứng riêng. Tại B, field10 của118
tăng1→2 sau Fire. Không suy hai actor có cùng capacity ban đầu từ cùng recipe.

**165 gửi lặp:** sau mỗi positive165 khoảng1s, gửi lại cùng payload bằng seq receiver mới liên tục.
Cả hai vẫn đọc12/12/errors0 nhưng stat không tăng lần hai; mỗi loại chỉ đi increment một lần.
`E14DA0` tra item trước effect; trong nhánh item không còn, trả sớm. Ca đo giới hạn hai index đã bị
xóa trong cùng round, không chứng minh mọi stale/round-generation/reuse index đều an toàn.

**Correction power118→159:** mapping cũ dùng field6 của118 như power chỉ vì cả field6 vàfield10
đều1 trước pickup. Fire phân biệt: field6 vẫn1, **field10/u8 tại payload+16, nguồn record+18**, lên2.
Writer `E2E1D1` serialize record+18; không dùng record+30 (nguồn field6) thay nó.
Harness `capacity-power` đổi nguồn **118.field10→159.field8/u8 tại payload+18**;
owner/serial/hàng/cột/ID giữ cách ghép đã kiểm. Native đọc packet độc lập, và `E9C58C` trên cả hai
client đọc power2 cho bom B tại6/3,6/4; bom A tại3/3,3/4 giữ1. Trước correction chỉ có proof wire
đang lấy nhầm field, chưa probe calc của peer để gán toàn bộ ảnh hưởng runtime cũ.
Raw/checkpoint/harness CP024–030 và các run mapping cũ giữ nguyên làm lịch sử/control; không sửa hash cũ.

Emulator vẫn chọn placement/index, fuse30s, broadcast, timing/duplicate và outcome. Nhánh tăng stat
và clamp là code client đã chạy, không tự phục hồi authority server gốc. Item ẩn/phá block/reveal,
runtime nhặt tại max, speed, validator/disconnect/full auth/cache/codec còn mở;
reset stat/item được kiểm tiếp hẹp ở [CP032](#hidden-items), chưa toàn lifecycle round;
**chưa full match/K5/K6a/LAN/Internet**.

<a id="hidden-items"></a>
### Gạch, item ẩn và khởi tạo lại vòng — CP032

**kr_static/kr_runtime confirmed trên diagnostic copy Wine**, không patch thêm hoặc ép stat/state.
[Recipe, controls, native, snapshot và hash](../evidence/hidden-items/README.md).
VN chỉ hướng dẫn thử list2 trước start và theo dõi report vỡ gạch; schema/consumer KR được kiểm riêng.

**List2 của582:** sau giải blob có counts `[0,2,0]`, hai record
`index27/row2/col3/kind0` và `index28/row3/col2/kind1`;26byte inner được đọc đủ.
`D824D0→1245700` lưu list2 ở context+3E8; getter `1245780` được gọi từ `EA46E2`, rồi
`EA46F8→EB56E0` duyệt list2 trước list1. Với hai ô trên ExVillage01/code278, lookup đầu
`E9FCF0` trả tile, hai lookup thay thế trả0. `B9C8D0` qua guard `11902B0`, ghi kind vào
tile+B0 và index vào tile+AC. Cả A/B đo đúng tọa độ/index/kind, vtable1747724;
RTTI static khớp `CGOTileFixedBox`. Không suy mọi loại block/map đều chấp nhận list2.

**Control và reveal:** cùng input A đặt bom3/3 rồi rời ô, quay lại hai ô gạch; B giữ ở spawn.
Control không phát158: bom/gạch còn nguyên, không124/142/165 hoặc hit hàm tăng stat đã probe trong cửa sổ đo.
Ca có158/kind0 sau fuse thử6s: hai gạch biến mất và hai item hiện trên cả hai view;
A thực gửi124/index27 rồi28, stub xác nhận165 tới cả hai. Snapshot secure/native xác nhận
capacity và Fire của slot0 lần lượt1→2 ở cả hai; các max và stat slot1 đã đo giữ nguyên.
Không cần chuyển tiếp report142 để hai view reveal trong recipe này; chưa có phép tắt UDP riêng.

**Đường phá/reveal có mục tiêu:** `CGOTileFixedBox/B9CEC0` có nhánh giảm secure durability,
ẩn tile và lấy object nền `E9F360`; `118FBF0(index)`/`B8CC50(kind)` kiểm trước cạnh
`B9D0B4→B8B5B0(game,index,kind,...)`. Runtime đo hai destination vtable1746A4C/RTTI `CGOItem`;
pointer từng index khớp đúng item tìm thấy sau đó trong165 ở cả hai client.
Không dùng entry `E9DFC0` của đường đặt item nền làm proof reveal: probe entry đó không hit trong ca list2.

**C→S142**, writer `E72720`, construct tại `E727FF`, serialize `E7281B/28/35/41`:
`u8 row,u8 column,u8 selectedSlot,u32 aux`;7byte payload, khác VN135/u16 đuôi.
Hai client đều report cùng ô2/3 và3/2 nhưng selectedSlot lần lượt0/1, không phải luôn bomb owner0.
Hàm chọn slot bằng guard/local-slot registry; aux được caller `B9D11C` truyền−1 trong ca này.
Native framer/getters kiểm actual frame theo seq/socket; chưa generalize aux hoặc validator/server reply.

**Vòng kế:** run `hidden-rounds` có hai121/clock/layout trên cùng tám connection; hai tile/index
được gắn lại, cùng hai124/165 mỗi vòng. Snapshot đầu vòng2 ở cả hai đã trở lại capacity1/Fire1
cho slot0; sau nhặt lại lên2/2. Serial bomA lặp0/28 ở mỗi vòng, ID stub chọn1–4;
power request/broadcast là1 trước Fire,2 sau Fire. Đây là reset của các field/đối tượng đã đo,
không phải chứng minh toàn entity/timer/peer/seq không rò.
Run này chỉ có175/180 vòng1: harness kế thừa policy chờ B reported19 vòng2 trong khi input điều khiển A;
actual A17/19 vòng2 vẫn capture. Run sửa policy phải được đối chiếu riêng, không gọi ca này hai vòng hoàn tất.
`hidden-rounds-fixed` chỉ đổi điều kiện origin của policy thành A/A: hai reported17/19 của A dẫn tới
170 gửi B,175/180 mỗi client hai lần, cùng tám connection/seq liên tục và ảnh kết quả/phòng vòng2.
Hai lần layout/reveal/nhặt, stat reset và serial0/28/power1→2 vẫn khớp. Không coi sửa policy stub
là sửa bug client hoặc chứng minh mọi reset/validator đã đạt.

Placement/index/fuse6s, giữ một ACK/index/round, broadcast, rank và timing vẫn **emu_choice/design**.
Chưa phục hồi thuật toán random drop/server authority; list3, block variants, chain/rescue, runtime tại max/
speed, stale/generation/disconnect/full auth/cache/codec và K5/K6a/LAN/Internet còn mở.

<a id="current-hidden-items"></a>
### Item ẩn trong currentservice — CP068

[Recipe/hash/partial/retry/audit](../evidence/current-hidden-items/README.md). **kr_runtime/confirmed**
cho integration Winecopy, **design** cho catalog/reportgate. Tái dùng CP032exact582/counts0/2/0/
FixedBox/revealconsumer/142/124165/nativeeffects/knownUI; không thêm thân hàm native/patch hoặc RE lại
receiver/IDs/clock. Currentprofile giữ distinctspawn/cleanauth/cache/key+serial/reportguards.

`hidden_items` là catalog optional riêng với floor`items`, merged vào Room claimcatalog; duplicateindex
fail.124 kiểmcurrentkey rồi hiddeneligibility trướcclaim;142 có7byte đã phục hồi, kiểmboundselectedslot,
auxFFFFFFFF và knowncell. Sau ít nhấtmộtcurrent158 đã gửi, report mởindex tươngứng, duplicateidempotent,
Startreset. Không forward142, không servercollision/replayauth; old/forged142saucurrent158 vẫn ngoài
policy, currentblast bất kỳ không chứng minh ô báo đãphá. Floorprofilekhônghidden giữbehavior.

Run đầu2cyclesreveal/result chỉ1pickup27vòng2, khôngfulleffects. Retry normalUIroute có haiactual27/28
mỗi vòng,142 từhairole trướcpickup. Nativebothviews FixedBox(tile,index,kind)→revealcommit CGOItem→
165lookupsamepointer; capacity/FireA1/1→2/1→2/2, reset1/1vòng2;4actual118/159158/power1→2/serial0,28,
Astate1bothbefore175/hairesult/return/refresh/errors0. Không dùng cursor/ảnh hoặc runpartial thayproof.
Nativeframer/blob/11ED590/gettermodel tách khỏi runtime; counts0/2/0/inner26 kiểm, chưa mọilist2/mode.

Exporter `--item-layout hidden` giữ exactCP032fixture27/28 vàcatalog;defaultfloor giữ,41alias chỉfloor
khác. Đây là deterministicfixtureplacement, không thuật toán drop gốc, không tựmigrate profilecũ.
72serverregressions kiểmguardnegative/reset/forgerylimit; actualcapturebench khôngruntime mới.
Chưafullvalidator/reset/auth/K5/K6a/LAN/Internet. Nghiệm thu profile dùnghànhvi nào vẫn theo PLAN/gates.

<a id="item-generation"></a>
### Xác nhận item cũ khi tái dùng index qua vòng — CP042

**kr_runtime/kr_static confirmed hẹp trên diagnostic copy Wine**;
[recipe/control/hash](../evidence/item-generation/README.md). Không thêm patch hoặc ép actor/stat.
Theo [item165](#item-pickup)/[stat Bomb](#item-effects), đặt một Bomb/kind0 tại hàng4/cột3
mỗi vòng, index17 vòng1; vòng2 đổi18 hoặc giữ17. Hai client A/B giữ8connections/IDs1,2/slots0,1,
hai actual95/104/121/99/clock. Mỗi vòng một bom A rồi17/19,170 đồng bộ peerstate1 trước175/180/phòng.
Placement/index/outcome/fuse6s là **emu_choice/design**.

Sau clock vòng2, trước input/actual124 mới, stub mã hóa lại payload165 xác nhận vòng1 bằng seq
hiện tại của từng receiver. Payload12byte là `ok1,ctx0,slot0,row4,col3,index17,kind0,flag0`.
Đây là semantic replay từ stub, không replay nguyên frame vượt CRC/seq và không request client giả.

| Ca | Quan sát ở cả hai client |
|---|---|
| `unique-index`: vòng2 index18 |165 cũ đọc12/12/errors0, item17 lookup0, capacityA giữ1→1, không increment. Sau A Down có actual124/index18;165 hiện tại làm capacity1→2. |
| `reused-index`: vòng2 index17 |165 cũ tìm item17 hiện tại, chạy effect/increment, capacityA1→2 trước input nhặt. Toàn capture không có124 vòng2; chỉ có124 vòng1. |
| `guarded-fixed`: vòng2 index17, claim policy local |Ticket xác nhận cũ bị loại `stale_round`, không phát165. Sau input có actual124/index17,165 hiện tại đọc12/12 và capacityA1→2 cả hai; hai vòng vẫn kết quả/về phòng. |

Stat là secure actor+4A4 trước/sau từng callback, checksum và native52A350 kiểm độc lập;
max/power/capacityB đã đo giữ nguyên. Clock vòng2 reset capacityA về1 trước replay.
Không dùng riêng full cursor, icon biến mất hoặc log stub làm bằng chứng cộng stat.

**Cạnh đã kiểm:** E63260 đọc packet, E63331→E2C660 thử xóa pending theo ctx/index,
sau đó mới kiểm ok/context và actor. Với variant đã đo, việc không có124 mới không chặn effect
khi item hiện tại khớp. Interface→F73D30/B52840/E14DA0 tìm item bằng index; E636D9 kiểm pointer,
E63712…E6374F kiểm hàng/cột và E637F9 qua nhánh match. Probe effect có thể xảy ra trước nhãn
FOUND/MATCHED của outer handler do interface gọi lồng; không dùng thứ tự tên probe thay call graph.
Chưa suy mọi variant đều bỏ qua pending hoặc mọi tọa độ/flag/kind sẽ được chấp nhận.
Lượt `guarded-index` trước bản fixed lỗi collector `claimed_items` trước165, không phải client/protocol
từ chối; giữ raw và harness lỗi như lịch sử tại evidence, không gộp thành ca runtime thành công.

**Policy emulator:** [ItemClaims](../../server/item_claims.py) kiểm catalog hiện tại cùng slot gắn socket,
context/index/hàng/cột/kind, giữ một winner và ticket xác nhận local gắn owner/generation.
`begin/finish` vô hiệu hóa ticket cũ; `consume` chỉ một lần dưới owner/lock trước phát165.
Đây là design cho job/xác nhận local chưa gửi, không field epoch phục hồi từ wire.
Request124 cũ trùng toàn bộ catalog hiện tại vẫn không thể được module phân biệt chỉ bằng các field đó;
distance/tick/session/actor-state, pending159, timer abort và UDP/peerqueue còn cần kiểm riêng.
Không gọi module này là toàn validator hoặc nghiệm thu K5/K6a/LAN/Internet.
[Bước tiếp](../RESEARCH.md#q-023).

<a id="needle-service"></a>
### Kim145 trong current service: credit và report-pair opt-in — CP113

[Metadata/raw/control/hash](../evidence/needle-service/README.md); [gap](../RESEARCH.md#gap-needle).
**design/implementation** trênmodeltrước, **kr_runtime/confirmed hẹp** sau currentcopyrun; không
servergốc/inventoryauth. CP037038nguồnnative táidùng trực tiếp, khôngnativebody/codec mới hoặc portVN117.

Profile có `needle_escape: {"mode":"needle_145_report_pair"}` mới bật. Chỉ đúngmột catalogkind145
(kểcảhiddenlist), Start đòihai live mappedA/B. Khôngoption giữdefault, không tựbật theo tênitem.
Accepted124 đúngcurrentclaim/key/catalog/boundmember, hai165sendxong mớicredit1 cho boundconnection/
currentgeneration. Dedupeclaim cũ ngăncreditlặp. Credit không làaccountinventory hoặc locationvalidation.

Actual1 chỉcommonprefix14; actual0 đúng22 với typedtail widths1/2/1/4=`1,145,0,0`.
Sau freshnessread-only, admission đòicurrenttrap17/noaccepted19/credit1 và đúngliveboundmember.
1 chỉstagecandidate cógeneration/trapSHA/report1SHA/seq/prefix6/tick; khôngconsume hoặcforward.
0 phảikhớpcandidate trap/slot/XY/logicaltick; haioldcontrolsCP037038cùngtick nhưng rulepairing là
designvariant, khôngguaranteemọi mode. Actor_field5→6 không đòikhớp. Service gửi170/sub1 exactprefix6
của1tớipeer, rồi mớicommitcursor0/credit1→0/clear17/pending. No170/sub0 guessed.

Thiếukim/sai_shape/cặp/tick/XY/crossconnection/dead khôngforward/unblock/consume hoặcthaycursor.
Accepted17/19mới hủypending; accepted19history khôngbị17sauđóche. Start/acceptedleave/EOF xóastate
kim của member (survivor trongroundabort khôngadmitkhiinactive); connectionreportcursor giữCP063.
Sendfailure chưa consume/clear/cursorcommit, nhưngrealserviceexception vẫnclose/abort; directmodelretry
không lànetworkrecoveryproof. Rescue18 vàreport21 không tựclearinventory/pickupmarker.

Runtime199,834777498s/SHA6074f0ea… trêncopyWine/profile47+CP037needlefloor/index41/AB/BchiefAguest:
actual124/165→17→1→0/tick3220/slot0XY120309, report1frontseq274/service280 vàreport0front277/service283
khácnhauvìTCPproxy replay; only170/sub1Bseq112/full6. A local56→26/Bpeer56→26/reader6/6/lỗi0;
actualnative afterRightA/B XY200309, ôkimtrống vàsecondCtrlkhông1/0 mới. Ledger source/design logs1credit/
1consume, defaultoff/onactualoldpairmodels36joins/175server tests. Nativegetters162+12/typed108models
khácactualcallbacks. Noresult/secondStart/depletedsecondtrap trênnewservice trongrun này.

Giới hạn lịch sử CP113: run này chưa có Start2/depleted. Phần đó được đo tiếp ở
[CP114](#needle-round-reset), rescue A/aux1 ở[CP115](life-results.md#rescue-service); multi-charge vẫn unknown.
Fullinventory/forgedfirstseen/location/authority/auth/fullmatch/K5/K6a/vanilla/LANInternet vẫn mở.


<a id="needle-round-reset"></a>
### Kim cạn và freshcredit sau Start2 trên current service — CP114

**kr_runtime/confirmed hẹp**, [metadata/raw/hash](../evidence/needle-service/README.md#cp114).
Không code/profile/fuse/client/probe mới: giữ CP113option và floor145, reuse CP038depletedcontrol/
CP110BhostreturnedReadyAStartB. Chỉ normalOSinput thêmSpace/emptyCtrl và knownStart2.

Run339,135766083s/SHA9356d26d11f03676d1ccdc388ccb06f317f2d0b4c9d381c2a3e82893616825db:
R1actual124145→17→1→0/credit1→0/peer1701/nativeA,B56→26. SauRight/Space thứhai A nhận17,
Ctrlrỗng không1/0 mới, rồi **actual19** trước chosenrank175/return180/585/110/fullnativeerrors0.
Khôngsuytimer/gamephysics law từkhoảngchờCtrl hoặc giả19do service tạo.

CùngTCP/IDslot/Bchief/Aguest, actualAReady95/BStart104→R2native121129/clock/currentkey12346.
Old124bodykey12345 replay bịloại trướccredit/165. Fresh124index41/kind145/key12346 đượcclaim/lạihai165;
A nativeCtrl×1, report17→1→0 mới/cặpđúngXYtick, ledgercredit1→0 lần2 vàpeer1701/nativeA,B56→26.
Duplicate124hai vòng khôngcredit thêm. Native4lầnescape-after26/hairole×hairound, localpairsource/nativegetters
/cursor/hashvàảnh tách ledger; modelStartclear không tựlàoracleclient.

Chỉ **một result/return hoàn tất**, R2 dừngactive sauescape. HaiStart/hai lần dùngkim khác hai trậnđãkếtthúc;
không nâng K5/K6a/fullmatch/resetmọibiếnthể. Baactualbomb/3complete158/view; reportsequenceR1
17,1,0,17,19 và R2 17,1,0. 2.082joins gồm1.977rawhash, copiednativefields408/typed264models riêng.
Nguồn CP113oneescape giữnguyên; oldCP038depletedvà CP114currentreset khácscope rõ.

Bước kế tại thời điểm CP114 là rescue18/profile; đã được giải trong scope fixed A/aux1 ở
[CP115](life-results.md#rescue-service), chiều B/aux0 ở[CP116](life-results.md#rescue-reverse). Thứ tự hiện hành chỉ ở PLAN.


<a id="direct-item-feedback"></a>
### Fire xác nhận trước input kế và bom hai owner trên directservice — CP150

**`kr_runtime/confirmed` hẹp**, [recipe/control/raw/hash](../evidence/peer-endpoints/README.md#cp150).
Reuse CP082freshfeedback/CP106native165/currentstats/CP149allTCPdirect và map/item KR đã biết;
không tìm lại nativebody/opcode/codec. Productioncode/profile/timeout30s/actorbinary giữ nguyên.
Chỉ controllercopy kiểm currentactual124index17/kind1 +native165 ởcảhai view trước khi đặt bom;
nếu chưa có, dùng boundednormalOSfeedback, không fake124/165/gamefunction/writeactor.
Giữ20pulse/.045s/3sneutral và5sdeliverywait; stopkhi nativepickupđãcó, không tăng hạn đểpass.

Hai run được giữ riêng. R1partial199.741s có actualFire124/165/effect trước extraalignment,
nhưng đòi tọa độ tâmY279/tolerance4 rồi20pulse dao động259..319 gâyalignmiss; chưaSpace/kếtquả.
Đây là thất bại điều kiện kết thúc phép đo, khônglàFirefail. R2newcopy/freeze364.479s/exit0/
1348raw hoàn tất: cảhai vòng A nhặtBomb16+Fire17,165cursor12/errors0 ởhai view,
copygettercapacity/power`1/1→2/1→2/2`; clockvòng2 vềbaseline. R1Fiređãcótrướchelper,
R2alignment dừngtrên actual124+hai native165; không đòi đứng đúngpixel sau khi đãnhặt.
Không suy đã giải nguyênnhân CP149R1miss hoặc hiệu ứng timing/proxy/packetloss.

Vòng1nativeA118/159owner0/ID1/3,3/power2; vòng2nativeB118/159owner1/ID2/9,3/power1
(BkhôngnhặtFire). Native158/process/195flags ởhai view khớp power/geometrythực; currentA17→19/
B21 vòng1, B17→19/A21 vòng2. Native175/180 decodedrank2,1→1,2, resultreset hai lần,
finalReady95/native110/noStart3. **664vethPCAPrecords**,4peerwindows/currentkey/target/currentprogress,
tấtcả8TCPdirectservicepeerA.2/B.3/noTCPproxy/UDPproxy. Render4ảnh sau reset/Ready đãxem.

3537lifecycle/1490wire/499peer/70pickup/16inputgate/25Ready/55directTCP joins;
models1668actor/324peergetter/2364flaggetter+2340array/294typedpacket/297report/12native153/
42UDPCRC táchruntime. Hai loadingbarrier27.376/27.838s<30; khôngcoldstartstabilityclaim.
Currentcredit/claims/result vẫn emulatorpolicy/authoritylimits, không luật server gốc/toàninventory.
CP150 khôngxóa CP149partial/CP150R1failure, chưafullmatch/K6a/realLAN/Internet/8clientsthật.
[PLAN](../PLAN.md#current-order) tiếp acceptance còn thiếu cho hai client trực tiếp, ưu tiên actualcontested
pickup theoCP030/062 với claimlogicgiữ và boundedmeasurementdelivery, không RElại 124/165.


<a id="direct-contested-pickup"></a>
### Hai nativeclaim cùngFire17 trên distinctspawn/directroute — CP151 partial

**`kr_runtime/confirmed` R1**, [recipe/raw/partial](../evidence/peer-endpoints/README.md#cp151).
Reuse CP030/062actualtwo124 vàpositive165delay2s, CP150currentdistinctspawn/profile/claims/directroute;
không mang overlapspawn/Roller củaCP062 sangKRcurrentprofile. Wrapper`runtime_hold.py` sao nguyên
CP062: chỉ delaypositive165, productionclaim/key/consume nguyên, provenance là **design measurement**.
NormalOSfreshfeedback approachA199/B359 rồi DownA/UpB0.30s và release; khôngfake124/forcewinner.

R1/key12345 cóhai actual124 cùngindex17/kind1/cell6,3: A tới trướcđượcnhận, B bị
`claim_not_current_or_already_consumed`. Hold2.032375181s; Brequest tới trướcdelivery1.692042763s.
Mỗi receiverchỉmộtFire165/full12/errors0/slot0, nativegetterA power1→2/Bpower1giữ,
capacityA/B2giữ;45contestjoins. R1cóhai peerwindows/A118ID1power2/158/195/currentA17→19/B21/
native175rank2,1/180/110reset và Start2. Đây **không** là luật fairness/arbitrationserver gốc.

**Blocker:** R2cảhai nhận121full129 vàinit2actor, nhưngkhôngactual99 trênTCPchanneltrướchạn30s;
serviceđóngcảhai vìloading_timeout. KhôngnativeclockR2/contestR2/endingR2/finalReady2.
Serviceghi forfeit rank/return/refresh R2 với **broadcastrỗng**, khôngphải kếtquảnative lần2.
Harness exit0/`virtual_two_returns_native_observer_timeout` chỉnói cleanupxong; không2cyclespass.
Nguyênnhân thiếu99 chưa xácđịnh; khôngquy delay165/networkloss/clientdeadlock từcorrelation.

Run353.595s/1204raw/407vethrecords;905partialjoins/298peerjoins/2R1windows;
models1176actor/192peergetter/1182flaggetter+1170array/171typedpacket/174report/6native153/22UDPCRC
riêng. Initialverifier thiếucontext165 nên trùng snapshot; sửajoinđúngcallback/context, raw giữ.
Initialpartialchecker giảR2khôngrank bịbácbỏ bởi emptyforfeit; raw/model/policy/nativereceipt phânbiệt.
Source/config/binary khôngđổi giữarun;30sdeadlinegiữ, capture/process/namespacecleanup/hashđầyđủ.

[PLAN](../PLAN.md#current-order) tiếp trace nativewriter99 đãbiết`E7F2A0`/guardstage+6410 và
actualsend`13ADD10` với monotonic timestamp từ121 tới deadline, khôngraise timeout/force99/clock.
CP150hai cycles/BombFire/ABowner vẫnnguồnriêng; CP151R1khôngbackfillR2/gate/fullmatch/LAN/8clients.


<a id="current-basic-hidden"></a>
### Basicprofile ghép trên currentmode/Bhost: native reveal và ba hiệu ứng — CP169

**`kr_runtime/confirmed` hẹp cho copyprofile thử; placement/reportgate/controller là `design`.**
[Raw/partials/hash](../evidence/peer-endpoints/README.md#cp169), [gap](../RESEARCH.md#gap-floor-pickup).
Reuse CP032/068/069 exact582/list2/FixedBox/reveal142/124165/effectfamily, CP095actualbox/currentterrain,
CP165boundednativefeedback vàCP166/168Manner/title/Bhost. VNitems/oc_e-game chỉworkflow/family đãpin;
không portopcode/ID/offset/enum hoặc RE lạiconsumer/ItemAttr/thân hàm native. Sourcebuild e9bb4766…/
static660447…/modOa08950…/base400000 giữ, khôngsourcebinary/pack/DBchange.

Copyprofile ghép từCP168current47pin: **46alias giữexact**, chỉfloor582 thayexactCP069counts1/2/0/
inner36; floorRollerindex17,row4col3,kind2; hiddenBomb27,row2col3,kind0/Fire28,row3col2,kind1.
items/hidden_items/item_layoutcatalogtươngứng, mode/title/terrain/network/capacity/guards khôngđổi.
Nativeframer/blob/11ED590 valid/CRC/partial3cases vàconstructorpass trướcWine; list2gắn vàoactualFixedBox,
khôngtạo tường mới. Committedpreset/default vẫnfloor16/17; không gọi profilecopy làpresetđãmigrate.

R1knownCP069OSroute chỉactual17+27; R2thêmfeedbackmissing28nhưngoldrouteđãnhặt28/bỏ27 nênchỉ17+28,
khôngchạyfallback. Cảhai view của từngrun đãnativeinstallFixedBox(vt1747724)/reveal27+28→CGOItem
(vt1746A4C), actual142bothroles; các effect cónguồnriêng, khônggộphai partial thành3effects.
R3freshcopy7connections/Aroom/Blogin3-nochannel4 tới330sbound/nohiddenruntime; exit0/khôngerror
khôngacceptance, không gánstartupfailurechohidden/membership hoặcbypassauth. R4freshprefixcùnglogic
retry/R3nguồn giữ; nonewnativebody/handlerfix. R2generatorindentfallback sửa **trướclaunch**/refreeze,
candidatecontrollercũgiữlocal, khôngsource/configchangegiữarun.

CùngR4 192.983476401s/exit0: actualMannercreate/Bhostpromotion/Arejoin/ReadyAStartB/12199clock1;
normalSpace/158 pháactual2Box→native27/28reveal/4actual142→actualA12417,27,28/currentkey12345→
native165full12/errors0/3callbacks/view/lookup27,28 đúngpointerreveal. Nativegetterbothviews chỉA tăng
Roller5→6/capacity1→2/Fire1→2, Bcapacity2/power1/Roller4 giữ; đây là familyeffect đãRE, khôngmớiluậtitem.
Missing28 dùngpeer_alignread-onlyArowY159/X80,tolerance12/20attempt/3freshneutral/45ms, giữ3rare165+
lookupE636D9/statprobes, exactrestore4viewphaseacks. Only2alignmentgroupshit; missing27fallbackkhônghit
trongR4, không giảmọinhánhcontroller đãkiểm.

176integrationjoins/122effectjoins/60capturednativeframer/gettermodels; pointer/cell/typedreportSHA và
securegetters là bằngchứng, ảnhfinal2view chỉrender. 721raw/109frozen/namespacecleanupsealed.
Chỉepoch1/oneblast/3effects, **chưa normaloutcome/reset/epoch2** trênprofileghép. Originalrandomdrop/
distance/auth/hiddenreportfreshness/validator giữgiớihạnCP068; khônglấy142 làmworldoracle.
Next[PLAN](../PLAN.md#current-order)ghépknownfinalSpace/result/reset/twoepochs, khôngRElạinativeconsumer.
K4dependencies/K5/K6a/fullmatch/Windows/LAN/Internet/8client giữmở; auditAUD050qua168/next172.


<a id="hidden-cell-admission-current"></a>
### Hidden142: ràng buộc ô đã phá trong bounded terrain — CP172

**Guard là `emu_choice/design`, native compatibility là `kr_runtime/confirmed` hẹp.**
[Recipe/raw/hash](../evidence/peer-endpoints/README.md#cp172). Source-first CP068 weak report policy,
CP069/170/171 basic/native195 và `terrain_blast.cleared`/commit có sẵn; không mới native body/opcode,
không suy hành vi từ asset/VNlayout. Trước CP172, known142 chỉ cần có anycurrent158, kể cả blast
không phá ô được báo; test cũ giữ giới hạn này cho profile không có terrain.

Khi profile có pinned terrain, sau slot/aux/knowncell/anycurrentblast guard, service đòi một bomb
đúng generation có `terrain_blast.generation` hiện tại, `cleared` chứa đúng ô, commit>0 trước report,
và receipt158 đúng toàn bộ roster. Metadata terrain chỉ được commit sau broadcast thành công;
pending/partial/oldgeneration/unrelated cell không mở eligibility. Report hợp lệ lưu `terrain_clearance`
source design/generation/bombid/commit; duplicate vẫn setdefault/idempotent,124/165 không đổi.
Không terrain giữ lower bound lịch sử, không nâng mọi profile thành validated world.

Năm test mới kiểm unrelated/pending→positive, two-peer duplicate/two cells/onepickup, reset/old
generation, partialbroadcast và roster/commit order;270server tests đạt. Đây là policy tests, không
native oracle hoặc runtime negative attack controls. 142 vẫn không có recovered roundkey:
**cùng old bytes sau legitimate currentcellclear không phân biệt được**; auth/distance/fullfreshness mở.

Fresh modifiedWine/privatehosts/directproduction cùng exact basic export CP171/47pins, chỉ delta
service guard; source/config frozen→cleanup/seal. Run406,400966485s/exit0/1438raw/158frozen. Hai
native cycles/keys12345→12346, bốn A bombs IDs1..4/power1,2,1,2, ba pickup/effect mỗi epoch trênhai
view, native stat/hiddeneligibility/195 reset và56→1/other0 before175 giữ. Ba barriers
6,459173952/6,455755669/6,374629005s trước30s; normalBStart3/key12347/native12199/hai clocks thêm
9,229294342s/render hai view. Không thirdbomb/outcome; cleanupEOF sau completion0,409834981s.

16actual142 gồm **8accepted hidden-cell reports** (4already-revealed other-view duplicates),
8outsidecatalog reports bị `unknown_hidden_cell`, không phải 16accepted hoặc injected negatives.
Accepted đều gắn rootbomb1/3 đã clear ô trong currentepoch/fullroster/commitbeforereport; không mở
eligibility phụ/không thêm165effect.52admissionjoins,1736first-two và419Start3/stream/observerchecks;
180capturednative packet/552securefields,life23/16getters;4adaptivealignmentgroups/8viewphaseACK/
rare165lookup enabled/exactrestore. Initial counting assumption16accepted được giữ và sửa về8+8;
raw/runtime/guard/expectedvalues không đổi để pass. Không causalclaim về số report/input timing.

AUD051 qua172 đối chiếu sources/raw và sửa stale current status/next-step; next auditCP176.
Không K5/K6a/fullmatch/dependency/Windows/LAN/Internet/8client. [PLAN](../PLAN.md#current-order) tiếp
B118 và peer windows hai chiều trên đúng basic run; oldfloor proof giữ riêng. Generalphysics/drop/
mode/inventory/auth và old142saulegitclear không được giải bởi guard này.


CP172 cũng giữ được pin source lịch sử CP166 khi service thay đổi: checker xác minh đúng source
Python tại commit lần đầu công bố record v2, gồm exact declaration/path/size/SHA và externalmanifest
nếu có. CP166/source `81b0c8d3…` khớp commit `09759d0`; không sửa complete record/evidence. Raw/
binary/local không dùng fallback; source chỉ có v1 hoặc không khớp khi công bố v2 vẫn lỗi.34checker
schema/history tests đạt; đây là versioned provenance, không native semantics/gate. [Quy tắc](../evidence/checkpoints/README.md).


<!-- protocol-topic-content-end -->
