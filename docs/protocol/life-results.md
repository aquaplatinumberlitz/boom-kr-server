# Trap, death, cứu, kết quả và reset

[Mục lục](../PROTOCOL.md) · [Câu hỏi đang mở](../RESEARCH.md) · [Thứ tự việc](../PLAN.md#dashboard).

<a id="peer-events"></a>
### Đồng bộ trap/death qua KR170 — CP029

**kr_static/kr_runtime confirmed trong phạm vi copy Wine**, [control/recipe/hash](../evidence/peer-events/README.md).
Giữ SHA runtimee9bb47…/static660447…/modOa08950…, không patch thêm. Cùng namespace chỉlo,
files-only NSS, lease và hai prefix/display/game-dir riêng. Đây là broadcast policy thử của emulator;
chưa phục hồi validator hoặc toàn authority server gốc.

**Điểm lệch đã đo:** `control-targeted` không gửi170. Cả hai có bom/kind0 và actorA/slot0 đạt
state56, khớp ảnh bong bóng và reported127/sub17. Sau actual127/sub19, localAstate1 nhưng actorA
trênB vẫn56 **ngay trước callback175** ở cả hai vòng. Getterstate là`B3F4D0→B33250→57A870`,
secure actor+28; wrapper/data được chụp và gọi native52A350 độc lập, không suy state từ lời stub.
Không suy trap56 trênB chỉ do UDP hoặc chỉ do physics: hai đường cùng tồn tại, chưa có phép loại riêng.
State1/56 là tương quan với hành vi/ảnh trong profile này, chưa enum chung mọi class/mode.

**Receiver đã nối:** dispatcher`E5D3A0`, byte table`E5DE9C`/jump table`E5DDC4` đưa
**S→C170→E5D5B6→E85B60**. C→S127 là report khác; không có handler127 riêng trong bảng
CGameStage này, chưa loại fallback strategy. Không chuyển tiếp nguyên127 hoặc áp subtype VN nguyên xi.
Header170 gồm **u8subtype,u8slot,u16x,u16y**, x/y sign-extend khi truyền consumer.

| Subtype170 đã đo | Body sau header6 | Consumer/giới hạn |
|---|---|---|
|17 |u32aux; tổng10byte |`E85D1B→E86780`, lookupactor`E12C40`, guard`B40D60` đọc secure+5A0; nếu0 thì vf74(tọa độ),vf144 rồi`E72C10(slot,aux)`. Không đồng nhất vf144 với hàm setter: nhánh có thêm tác động, proof là snapshot state thực.|
|19 |serializedblob; tổng21byte ở ca này |`E85DE2→E86910`, `1403D00/1403E90` đọc u32 +u8count +entries. Ca đo inner5=`FFFFFFFF00`, không entries. vf74; vf1D8=`F73950→1243330(slot)` gate cho vf110/B41150,`E72A50` và`13E13D0`; cuối`E72B80`. Chưa variant inner khác hoặc mọi gate.|

**Chuyển report đã kiểm hẹp:** nhận actual127, kiểm CRC theo seq/socket, lấy6byte đầu (sub/slot/x/y).
Với17, giải serializedblob sau prefix14, inner8 gồm2u32LE; lấy **u32 thứ hai** làmaux170.
Cùng giá trị đi`E72C10` từ localwriter`B783B7`; ý nghĩa tên field còn unknown. Với19, giữ serializedtail
sau prefix14, bỏ haiu32 timestamp/field cuối của prefixC2S; mã hóa lại thành170 theo seq receiver.
Chỉ gửi client đối diện, không phản xạ lại origin. Clock/identity/seq/ID shared giữ theo chuỗi CP028.
Không coi client report là sự kiện đã được server xác thực; auth/slot/generation/duplicate/stale còn cần validator.

**Can thiệp thực:** `forward170` có hai vòng cùng pair. TrênB,170/17 đưa actorA từ0 sang56,
10/10/errors0;170/19 đưa56→1,21/21/errors0. `E86A03` thấy vf1D8=1,
cạnh`E86A17` sau vf110 và`E72B80` được quan sát; state1 đã có trước175, không lấy bảng rank
làm proofdeath. ActorBslot1 không bị đổi sang1 trong handler này. Cả hai vẫn175/180 đọc đủ và về phòng.
Run`alternating` kiểm A→B vòng1 và B→A vòng2: A đọc170/17,19 cho slot1/x120/y399,
state actorB đạt56→1, actorA vẫn0 tại mốc đó. Bom owner0/serial0 rồi owner1/serial1 có IDchung1/2.
Run 240s chạm hạn sau death vòng2, chỉ một175/180; giữ ca partial, không gọi hai vòng hoàn tất.
`alternating-long` giữ cùng protocol/recipe, thời gian đo320s: đủ hai175/180 đọc302/3byte và UI phòng
mỗi phía, cùng8connections. Cả hai chiều actor tương ứng đã state1 trước175; vòng2 rankA1/B2 được
stub chọn theo B reported19, ảnh cả hai có PERFECT KRLocal/LOSE KRGuest. Trên A ở vòng2,
actor A có state0 sau170/17 nhưng state20 sau170/19, rồi0 trước175; actor B có state1 sau170/19.
Không khẳng định actor còn lại luôn0 hoặc bất biến trong lúc xử lý; cần control riêng để gán nguyên nhân
cho biến đổi state của actor A. Đây là kiểm consumer/policy thử, không chứng minh ranking/economy
server gốc hoặc tổng quát không rò state.
Không suy broadcast này đủ mọi trận/mode hoặc cả trap đồng bộ chỉ qua một transport.

**Gate peer có side effect riêng:**`13E13D0` chỉ ghi network+170+EC×slot=1 trong code đã đọc;
`13E1400` ghi0 và cập nhật time. Không tự gọi đó là disconnect/đóng socket hoặc toàn lifecycle removal.
Vì170/19 có nhánh này, phép kiểm reset/peer vòng kế là bắt buộc; chưa tự đóng gate từ parse/callback.

**Giới hạn probe và corrections:** hai run đầu không có118 sau Space dù đã scene/clock;
probeB33270 và đặc biệt B41150/bt4 rất dày. B41150 còn chạy từE7CC16 trong tick thường,
nhãn cũ`ACTOR_DEATH_V110` trong raw **không chứng minh death**. Run gọn chỉ probe cạnhE86A17
trong170 và snapshot tại mốc; có một retry Space0.8s nếu chưa118. Giữ ca chưa đạt mục tiêu,
không khẳng định nguyên nhân input chỉ từ recipe thay nhiều yếu tố. Native emulation giữ framer/
handler/blob/vector nhưng model world/actor/guard/endpoints; runtime state/ảnh là nguồn kiểm riêng.

<a id="round-result"></a>
### Event trap/death và bảng kết quả — CP023

**kr_static/kr_runtime confirmed trong phạm vi copy offline**; outcome/rank/timing của stub là
**emu_choice/design**, không phục hồi server gốc. [Evidence](../evidence/round-repeat/README.md) khóa
run/native/code/ảnh/hash và các probe thất bại. Copy không thêm patch; chín vùng code so byte giữa
static copy, runtime copy và modO **memory-layout** tại RVA, không remap modO như PE disk image.

**C→S127:** writer chung `B53010` tạo opcode127; runtime trap tại`B78373` đẩy subtype17,
death tại`E6EFDF` đẩy19, cùng actor và virtual21C trỏB53010. Call`1164D50→13ADD10`
có return site`B7839F` (trap) và`E6F008` (death), đã bắt stack runtime.
Prefix chung **14byte**, gồm u8 subtype,u8 getter actor+584/v28,u16`B335C0`,u16`B335E0`,
u32`12580B0`,u32 actor/v35C. Tại CP023, getter slot/tick/field cuối còn giới hạn semantics; nguồn tick đã nối ở [CP055](items-stats.md#pickup-tick);
không tự gán4039 thành vị trí map hợp lệ hoặc12580B0 thành milliseconds.
Sau prefix,`14EC120` nối buffer serialized có length và checksum; native`14EB410` giải đủ:

| Capture event127-direct | Prefix | Inner sau giải buffer | Cursor outer |
|---|---|---|---|
| frame49/seq187, trap17 |17,0,40,4039,1445,5 |8byte=`0200000000000000` |32/32 |
| frame53/seq199, death19 |19,0,40,4039,1654,0 |5byte=`ffffffff00` |29/29 |

Native framer/primitive/blob và Adler32(seed0) độc lập đều khớp. Inner tương ứng2u32 vàu32/u8;
chưa đặt tên tất cả field. Tại CP023 chưa kiểm duplicate/stale/peer; [CP063](udp-peer.md#report127-freshness) bổ sung admission policy hẹp, chưa authority/correction tổng quát.
Đây thay phần prefix10/candidate của CP022 bằng envelope đầy đủ và cạnh writer thực;
không biến báo cáo client thành death được server xác thực.

**S→C175:** dispatcher`E5D686→E5F610`, compressed/serialized buffer→manager stage+6C24.
Header`E3EB20→E4ACB0→E4BA00→E4AE00` tiêu16rawbyte;
record`E3EB40→E4ACE0→E4BA20→E4AEF0` duyệt16slot/stride1B0.
Trong variant đã đo: header offset9=FF (signed−1 ởstate+5EA8) tắt phần feature60F8D0 thêm7byte/row;
flag0 là1byte, flag1 record131byte. Không áp131cho mọi header/feature branch.
Mẫu292rawbyte: header16 +2row131 +14empty; cursor native16→147→278→292,
runtime inner292/292 và outer302/302/error54=0/error58=0.
Chọn local rank2 tại raw21, guest rank1 tại raw152; ảnh client có **LOSE KRLocal / PERFECT KRGuest**.
Rank/stat/EXP/lucci zero và guest tổng hợp là lựa chọn thử, chưa hợp đồng economy hay kết quả trận thật.
Mẫu175306byte ban đầu sai row placement là navigation failure, chưa gửi cho client.

**Policy thử theo báo cáo:** ở các run repeat2, stub xác minh checksum của captured127/subtype19,
đợi1s rồi phát175,5s sau phát180/full3. Seq175=91,180=94; counter tiếp tục trên cùng channel.
Đây thay fixed15s của CP022; chỉ chọn local-loss/guest-win để kiểm consumer/UI, chưa arbitration
hoặc authority validation. Callback180 vẫn3/3/errors0 rồi UI phòng.

**Start lại:** run repeat2/repeat2-hold bấm Start sau return không tớiDE0310.
Run repeat2-refresh gửi lại own-slot110/seq97; lần bấm sau tớiDE0310 nhưng`DCAD80` trả3,
không gửi104. Nhánh`DCB744..DCB81C` kiểm group bits2..4 của state người chơi bằng1;
180 đã clear các bit này bằngFFE3. Run`repeat2-ready` thêm110/slot1/state24 ởseq100,
sau own110/seq97: start-check0, click tự phát104 lần hai,121/seq103 đọc129/129, stage13/scene mới.
197/196/198 seq106/109/112 đọc đủ14/7/13byte và phase3;159/seq115 có pointer bom mới,
158/kind0/seq118 có HELP,127/19→175/seq121 có LOSE/PERFECT rồi180/seq124→UI phòng lần hai.
Hai vòng trên cùng channel4, không reconnect/reset wire counter và không ép104/stage từ debugger.
Ảnh second-after-space,screen30/31/33 tương ứng scene/input,HELP,bảng kết quả,phòng.
Đây là hai vòng thử một client/guest tổng hợp, chưa hai client hoặc nghiệm thu toàn K5/K6.

Trong run CP023, hai capture118 đều base22/serial candidate0, tọa độ bytes100/1→100/3;
reply159 lúc đó vẫn dùng mẫu100/1/id1; echo/spawn được kiểm tiếp tại [CP024](bombs.md#bomb-position). Request104 lần hai wire461 thay vì109; framing/CRC có kiểm,
body tăng chưa giải thích. Delay kind0→reported19 khoảng6.5s/0.9s khác nhau chưa giải thích;
không dùng thành công lặp UI để tuyên bố spatial/timer reset chính xác hoặc authority đã khôi phục.
Guard log state trong probe là pointer wrapper, không phải raw flag word; điều kiện bit ở trên
đến từ code tĩnh và input110 được consumer chấp nhận, không từ cách printf đó.

<a id="collision-rescue"></a>
### Va chạm enemy, rescue và report21 — CP036

**kr_runtime/kr_static confirmed hẹp trên diagnostic copy Wine**,
[7 run/control/native/hash/audit](../evidence/collision-rescue/README.md). Không patch thêm/ép game state.
VN `oc_e-game.md` §cứu đồng đội/chọc vỡ hướng dẫn ca chạm và no-touch; KR report/receiver/field
được kiểm riêng, không áp số subtype VN12/13. Rank175/return180 bị tắt trong cả nhóm này.

**Bố trí:** ExVillage01/code278, A/B cùng spawn index0 → actual actor point(3,3) ở cả hai;
position0/1 vẫn riêng. B Down0,8s ra khỏi bom, A Space, stub từ actual118 phát159 và158/kind0
ID1/fuse6s; B Up0,8s sau report17 hoặc đứng yên ở control. Actor+4C/+58 được kiểm bằng snapshot
secure/checksum/native52A350; các giá trị pixel không suy từ nhãn ảnh.

**Enemy touch:** A/B team0/1. `enemy-first` có A17→19 cách **2,322s**;
`enemy-no-touch` cùng placement/Down/Space nhưng không Up có **6,180s**. A19 có trước lúc
Up kết thúc ở ca chạm; ảnh và trace xác nhận chuỗi, chưa nội suy mọi collision/timer/mode hoặc
loại UDP riêng. Không coi thời gian này là fuse/timeout server gốc.

**Một đội là control không đạt rescue:** `friendly-first` đổi team B trong121 nhưng roster hai actor
chỉ còn team0: report21 đã bắt đầu trước17 khoảng11,174s; chỉ có17, không18/19 trong cửa sổ đo.
UI hai phía không cùng trạng thái. Không dùng ca này để phủ nhận rescue hoặc chứng minh timeout bất tử.

**Thêm đối lập tổng hợp:** `friendly-three`/`rescue-forward`/`rescue-kept` thêm profile slot2/ID3/team1
và row121 thứ ba; **vẫn chỉ hai client thật A/B**. A/B team0, C spawn2/position2.
121 đọc155/155/errors0, native inner126/outer155. Hai ca đầu peer153 tắt slot2 khiến profileID3→0,
actor trở thành Computer3; giữ làm giới hạn/control. `rescue-kept` bật slot2/ba endpoint0 để giữ
profile, không clearID3; C không có socket/client người chơi thứ ba, không nghiệm thu ba client.
Cách bố trí này cho phép đo rescue trong ca cụ thể, không chứng minh toàn roster/authority của C.

**C→S127/sub18:** A actualwriter `E6F060→E6F0D1` gọi common`B53010` với subtype18,
serialize qua1403A00/1403A40 rồi`14EC120`. Payload **28byte** = commonprefix14 +serializedblob:
raw `18 05 00 00 00 00 01 00 00 00`, inner4byte=`01 00 00 00` trong các ca đã đo.
Native framer/getters/blob/checksum giữ code thật, allocation tổng hợp. Giá trị inner1 đi từ
argument`E6F060+10`; quan hệ với rescuer slot1 là **candidate**, chưa đổi rescuer/aux hoặc width ngoài0..255.
Tại commonwriter, state A vẫn56; sau local vf148 tại`E6F10A`, snapshot là26.
Không gọi snapshot trước serialize là state đã giải thoát.

**S→C170/sub18:** prefix `u8sub,u8slot,u16x,u16y`, thêm **u8aux**; tổng7byte.
`E85D71→E85D9B→E86820(slot,x,y,0,aux)`; vf1D8, vf74, vf148 rồiE71840/E72CA0.
Stub lấy prefix6 actual127/sub18, giải inneru32=1 rồi encode thànhu8=1, chỉ gửi B với seq mới.
Trong `rescue-forward`/`rescue-kept`, entry slot0/x120/y159/arg4=0/aux1, reader **7/7/errors0**;
B actorA trước56 → sauvf148/end26, A local sauvf14826. Control`friendly-three` không forward18
để B actorA vẫn56 đến ảnh cuối trong khi A đã thoát. `rescue-kept` còn thử A Right sau4s;
ảnh B cũng thấy A rời ô. Đây là đồng bộ giải thoát trong scope này, không validator/toàn authority.

**Report21 chưa giải hết:** actual127/sub21 chỉ commonprefix14, không blob; writer
`E6F370→E6F4CE` đẩy21, sendE6F4E3. Ở enemy no-touch, B bắt đầu21 khoảng0,381s sau A19;
ca một đội bắt đầu trước17. Static170/sub21→E85E92/E86CA0 đọc đúng prefix6 và gọi coordinate/v148
cùng các gate. Ca `enemy-ack21` phản xạ own-only170/21 lần đầu: runtime đọc6/6/errors0,
nhưng vẫn có49report21 sau reply. Vì vậy **không gọi đây là ACK đủ để chấm dứt report hoặc kết thúc trận**.
Semantics/validator/thuật toán result của21 còn unknown; cần theo nhánh đang chạy trước tích hợp result.

Nhãn `EVENT127_WRITER subtype` ở run enemy-first đọc nhầm pointer$esp+4;
report thực được giải từ capture, các ca sau dùng$esp+8. ReadE6F430 mở giữa instruction bị loại,
reread từ entryE6F370. State26/56 chỉ trong actor/profile này; không enum chung mọi mode.
Creator/layout/spawn/team/C tổng hợp/fuse/broadcast/translation là **emu_choice/design**;
chưa rescue đảo chiều/aux khác/enemy3actor/no-touch3actor/duplicate/stale/generation/result/disconnect/
toàn reset/full auth/cache/codec/vanilla/LAN/Internet/K5/K6a.
[Bước tiếp](../RESEARCH.md#q-023).

<a id="contact-precedence"></a>
### Tranh chấp cứu/diệt: ba client thật và nhánh chọn — CP037

**kr_runtime confirmed hẹp; quy tắc dừng ở chạm hợp lệ đầu kiểm bằng kr_static/emulation**,
[run/recipe/hash](../evidence/needle-contact/README.md). Ba client A/B/C có authID1/2/3,
ownslot0/1/2, prefix/wineserver/display/game-dir riêng; A/B team0, C team1, cùng spawn index0
point(3,3), position0/1/2 riêng. C là client thật với actual82→87slot2→95, không actor tổng hợp CP036.
Ba phía đọc121**155/155/errors0**, actual99 và clock một lần, 153 giữ profileC/peer endpoint thực.
Tên C vẫn trùng KRGuest; identity được kiểm bằng ID/slot/trace. Đây là ca RE ba client, chưa gate K7.

**Phân biệt input với chạm:** `race-first` bấm B Up/C Left lệch khoảng0,289ms, nhưng tại A deathentry
B(120,239), C(150,159), A(120,159). A17→19 sau3,100s; C gần hơn, không phải bằng chứng cùng chạm.
`race-Bfirst` cho C input trễ0,7s: tại tick428 có cờ cứu1/diệt0, rescuer slot1,
B(120,187), C(160,159), A(120,159); gọi E6F060 rồi actual18. Hai ca không có outcome ngược
trong cửa sổ đo; kết quả không khái quát cho mọi độ trễ/mode/packet ordering.

**Nhánh đang chạy:** backtrace nối update**E7C7B0→E6E440**. Đường trapped duyệt danh sách
actor đủ điều kiện; khi tìm thấy chạm hợp lệ, **E6EE3F nhảy thẳng E6EE66, dừng vòng quét**.
**1269B20** kiểm diện tích giao ≥400 và candidate vf1F0 hoặc bằng nhóm getter**60A3D0**
(wrapper actor+9C). Return1 đặt cờ cứu ở E6EDFA; return0 đặt cờ diệt ở E6EE18.
Getter runtime của A/B/C là0/0/1, khớp team121 trong profile này; vf1F0/special mode còn giới hạn.

**Cùng đủ điều kiện trong một lượt quét:** bốn ca native collection/predicate
**E6EAB3–E6EE66 +1269B20**, với cả B/C ở vùng giao hợp lệ, cho B trước→cứu, C trước→diệt;
đổi thứ tự danh sách đổi kết quả. Self actor bị bỏ qua; actor hợp lệ thứ hai không được xét.
Actorlist/order/positions/group/gates/rectangle math/profiling là model trong emulation;
không dùng nó chứng minh runtime cùng chạm hoặc thứ tự danh sách mọi mode.
`race-order` trace bốn lượt sau17, tick372–375: thứ tự slot**0→1→2** (A→B→C) trong ca này.
Tick389 A(120,159),B(120,275),C(120,159); predicate C trả0, cờ diệt1/cứu0/deathslot2 rồi report19. B vẫn xa, không phải cả hai cùng chạm.
`race-aligned-limited` tick426 vẫn chỉ B đủ điều kiện, cờ cứu1/diệt0; getter0/0/1 và tọa độ kiểm riêng.
Tại CP037 chưa bắt được cả B/C cùng đủ điều kiện trong một lần quét trên client thật, chưa đảo slot/team;
thứ tự trên không được nâng thành luật ưu tiên slot cố định mọi mode.

Tại nhánh cuối **E6EEA7**, cờ diệt khác0 gọi stage vf2DC; nếu diệt0/cứu khác0 mới gọi vf2E0.
Runtime xác nhận **vf2DC=E6EF50** (report19), **vf2E0=E6F060** (report18).
Sáu ca native final branch có ca hai cờ cùng bật chọn diệt, nhưng **đây là input tổng hợp;
chưa chứng minh trạng thái hai cờ cùng bật có thể đạt được trên trapped path thông thường**.
Không dùng kết quả đó làm luật “cùng chạm thì chết”: vòng quét đã thoát ở chạm hợp lệ đầu.
18 ca native predicate kiểm threshold/nhóm/vf1F0, không thay full world/authority.

Nhận định nháp CP037 “gom hết va chạm rồi diệt luôn thắng” đã bị bác bỏ trước khi checkpoint
complete; [AUD012](../evidence/checkpoints/audits/AUD012.json) giữ correction/nguồn/giới hạn.
Checkpoint/raw trước đó không bị viết lại. Trạng thái runtime còn thiếu và thứ tự kiểm tiếp chỉ giữ
ở [RESEARCH](../RESEARCH.md#q-023); recipe/số đo ở [evidence](../evidence/needle-contact/README.md).
Không suy luật server gốc/validator, kết quả175, full match/K5/K6a/LAN/Internet từ các nhánh trên.

<a id="contact-inversion"></a>
#### Đối chứng đổi team slot1/2 — CP038

**kr_runtime confirmed hẹp**, [recipe/control/hash](../evidence/needle-repeat/README.md).
`swapped-fixed` giữ IDs1/2/3 và slots0/1/2, đổi team A0/B1/C0 trong106/110/121;
getter runtime khớp0/1/0. Ba phía Start155/155/errors0 và clock một lần.
B/C cùng được input Down rồi Up, không xem cùng phím là cùng đủ điều kiện va chạm.
Trace bốn lượt đầu sau17 vẫn slot0→1→2; tick405 predicate của B trả0,
flags diệt1/cứu0/deathslot1, gọi E6EF50 rồi actual19. CP037 Bteam0 đã có rescue từ slot1;
**không hardcode slot1 luôn cứu**. Movement profile ca này cũng khác CP037 nên không dùng
hai run để khẳng định mọi khác biệt chỉ do team hoặc quy tắc luôn ưu tiên slot nhỏ nhất.

Lượt trước `swapped-symmetric` dừng trước Start do fixture Aguest106 sai seq58 thay64;
không có kết luận va chạm. Native framer key64 kiểm oldfixture→7/CRC, retime64→0;
body giữ nguyên. Sửa harness/fixture mới, không sửa raw lỗi/nguồn client.
Ca runtime cả hai cùng đủ điều kiện và các thứ tự khác vẫn unknown. Đây là kiểm bổ sung cho
độ khớp client gốc; ưu tiên tiếp result/disconnect/reset ở [RESEARCH](../RESEARCH.md#q-023).

<a id="rescue-service"></a>
### Rescue18 opt-in trên current service — CP115

[Wire/consumer CP036](#collision-rescue) được tái dùng;
[metadata, source/hash, recipe và kiểm](../evidence/rescue-service/README.md).
**kr_runtime/confirmed hẹp**: hai copy Wine thật A/B; fixed roster A/B team0/sharedspawn0 và
actor đối địch tổng hợp slot2/ID3/team1. Không đại diện ba client hoặc roster/team tổng quát.

`rescue_contact: {"mode":"friendly_AB_synthetic2_aux1"}` là **design/emu_choice**, mặc định tắt.
Hash body của sáu fixture start121/playerB/guestA/slotB/readyB/synthetic106 phải khớp control CP036;
không kết hợp host Back/promotion, initial key12345. Service gửi synthetic106 khi cả A/B đã map,
trước Ready/Start;153 khai báo0/1/2, slot2 ba endpoint0. Start yêu cầu live A/B, chiefA và marker106
đã gửi; đây là supported-profile policy, không full membership/authentication.

Admission18 kiểm freshness cùng connection trước side effect; chỉ targetA/slot0 với đúng28byte,
blob10/header `180500000000`/inneru32=1. Phải có current accepted17, không accepted19 trên hai member,
peerB không có current marker mắc bóng/chết. Chưa kiểm contact distance hoặc xác thực rescuer;
aux1 chỉ là biến thể đã đo, không truncate u32 bất kỳ thànhu8. Các guard này là lựa chọn emulator.

Gửi một170/18 prefix6 của report18 +u8(1) tới B. Chỉ sau send thành công mới commit cursor,
bỏ đúng trap marker và needle pending nếu có; **không tiêu credit145**. Rejection không đổi cursor/trap.
Start reset trap theo lifecycle hiện có; chưa có runtime kết quả/Start2 sau cứu ở nhánh này.

Run162,695s: actual A17 seq193 → A18 seq199; TCP front/upstream nguyên byte, không chèn report.
Native A56→26 tại local callback; B `E86820(slot0,120,159,0,1)`56→26, full7/7/errors0;
ảnh sau normalRight cho thấy A di chuyển trên hai view. Một run trước chưa phát104 khi106 bị đặt
sauStart đã được dừng có cleanup/seal, giữ lịch sử; không suy riêng timing106 là toàn nguyên nhân.
188 tests tại CP115; [PR185](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/185) đã merge `80e4d48`, review waived.
Giới hạn chiều ngược/aux0 của CP115 được đo tiếp ở[CP116](#rescue-reverse); aux2/full authority/gates giữ mở.

Native copied-model tái dùng CP037 xác nhận getter ởE6ED03 trả1/2 được giữ quaframe−6C→−70 và
E6EF10 đẩy vàoarg3/vf2E0(E6F060). List/gate/profile getter được mô hình hóa; **runtime aux2 unknown**,
quan hệ authenticated rescuer chưa chứng minh. Không chuyển opcode/offset VN sang KR.


<a id="rescue-reverse"></a>
### Cứu chiều B mắc bóng / A chạm — CP116

Tái dùng [fixed-profile rescue service](#rescue-service),
[control/positive/raw/hash](../evidence/rescue-service/README.md#cp116). **kr_runtime/confirmed hẹp**:
đổi vai thao tác normalOS, A Down/ B Space/ A Up/ B Right; không đổi client/profile/team121/153/probe.
Control trên service CP115 cho actual B17→B18 với **inneru32=0**, local E6F060(arg2=1,aux=0)
56→26/E6F10A; A-only guard từ chối, không170. Ảnh cửa sổ A vẫn HELP trong khi B đã thoát/đi sang phải.
Không suy trạng thái peer vĩnh viễn56 hoặc mọi đường UDP từ ảnh.

Service mở đúng target B/slot1 và aux0 tới peerA, giữ legacy tên option
`friendly_AB_synthetic2_aux1` để profile CP115 tiếp tục dùng được. Bản này hỗ trợ hai cặp đã đo:
A/aux1 hoặc B/aux0. Envelope vẫn28/blob10/header180500000000, aux phải đúng slot của member đối diện;
không nhận aux2 hoặc tùy ý truncateu32. Freshness/matching17/no19/peer-eligible/send-success-commit
và không tiêu credit145 giữ nguyên. Đây là design policy cho fixed roster, không xác thực contact/identity.

Positive sau thay đổi: actual B17→18/aux0, một170/18 tới A;
E86820(slot1,120,159,0,0) trước56→E86892 sau26/full1707/7/errors0. B local56→26,
ảnh sau normalBRight cho thấy B thoát/di chuyển trên cả hai view. 190 tests tại CP116; [PR186](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/186) đã merge `f33f3b8`.
Cả hai run chỉ một vòng đang active; chưa result/Start2 sau cứu, generalmembership/aux2/fullauthority,
vanilla/fullmatch/K5/K6a/LAN/Internet. Audit định kỳ AUD036 quaCP116, kếCP120.


<a id="needle-selfescape"></a>
### Kim tự thoát và đồng bộ peer — CP037

**kr_runtime/kr_static confirmed hẹp trên diagnostic copy Wine**,
[recipe, control và hash](../evidence/needle-contact/README.md). Không thêm patch hoặc ép state actor.
VN `boom-re/docs/oc_e-game.md` mục dùng vật phẩm Ctrl hướng phép thử nhặt→bị nhốt→Ctrl và
pattern report1 rồi0; ID/opcode/receiver được kiểm riêng trên KR.

**Kim KR:** bảng `KR_ItemAttr.bin` có `Item/ItemNiddle` kind**145**, offset11664, group23,
field160=1. VN cùng tên kind117; `ItemHPNeedle` kind17/group1 là hàng khác.
Parser đọc335 hàng KR/269 hàng VN, chỉ subset kim được runtime xác nhận, không khái quát toàn schema.
Fixture582 list1 chọn index41/hàng4/cột3/kind145; actual124 trả đúng index/kind,
165 hai phía đưa kim vào ô Ctrl×01 trên A. Placement/drop và ACK là **emu_choice**, không luật drop gốc.

**Local tự thoát:** `needle-fixed` và `needle-forward` có A actual17→Ctrl→**127/sub1** body14
(commonprefix, không blob) rồi **127/sub0** body22. Sub0 thêm
`u8=1,u16kind=145,u8=0,u32=0` sau commonprefix14. Nativeframer/getters/checksum đọc đủ;
không chỉ dựa encoder của stub. Writer subtype1 `B53010`, caller**E70EF1**, trong**E70E90**;
sub0 caller**AEE818**. E70E90 gửi1 trước vf148: state A tại writer vẫn56,
**E70F15 sau vf148 là26**, khớp state tại writer0. Không có19 trong cửa sổ hai ca này.
Ảnh cho thấy thoát bong bóng/ô kim trống; A tiếp tục Right và B thấy vị trí mới.

**Peer:** switch170/sub1 tại**E85DA6** gọi
**E86820(slot,x,y,1,-1)**, chỉ đọc prefix6=`u8sub,u8slot,u16x,u16y`.
`needle-forward` lấy đúng prefix6 từ CRC-checked actual127/1, gửi B với seq mới;
entry arg4=1/auxFFFFFFFF, reader**6/6/errors0**, B actorA trước56→sau vf14826→end26.
Local A sau vf14826. `needle-fixed` không forward1: A đã thoát nhưng B vẫn giữ actorA56/ảnh bubble.
Native dispatcher controls giữ E85B60/E86820, chỉ model world/actor/effect endpoints;
không dùng chúng thay bằng chứng actor runtime.

`needle-first` là **collector thất bại**: handler124 cũ trùng allowlist gây KeyError41/đóng socket;
không phải bằng chứng client không hỗ trợ kim. Raw lỗi giữ nguyên; `needle-fixed` loại handler trùng.
Tại CP037, Ctrl lần hai trong `needle-forward` khi đã thoát/ô trống không phát thêm1/0, nhưng chưa thử
**bị nhốt lần hai khi hết kim**, duplicate/stale/server inventory authority hoặc Ctrl cạnh tranh
với rescue/enemy touch. Không gửi170/sub0 đoán để suy inventory peer đã đồng bộ đầy đủ.

State26/56 chỉ trong profile này; timing/fixture/fuse/translation là lựa chọn emulator.
Hai client thật, không full match/K5/K6a/LAN/Internet hoặc vanilla Windows.
[Bước tiếp duy nhất](../RESEARCH.md#q-023).

<a id="needle-repeat"></a>
#### Bị nhốt lần hai khi hết kim — CP038

**kr_runtime confirmed hẹp**, [run/recipe/hash](../evidence/needle-repeat/README.md).
`repeat-ctrl` giữ cùng hai client/socket/identity và một Start/clock, một pickup145/165, hai bom A
ở cùng ô hàng6/cột3. Report A đúng thứ tự **17→1→0→17→19**; không phải hai trận.
Lần đầu Ctrl đưa local/peer56→26 như CP037. Sau đó chờ hơn8s, đặt bom lần hai;
ảnh ô Ctrl đã trống. Ctrl sau trap lần hai1,455s không có thêm report1/0 hoặc entry E70E90;
ảnh sau Ctrl vẫn bubble, A19 sau trap6,150s. Cả hai client kết thúc actorA state1;
170/17 đọc10/10,170/19 đọc21/21/errors0. Không suy inputreturn0 chứng minh mọi input đã được xử lý.

Native **1190C20** từ callsite B4D999 có guard1190C59: argument amount≤0 trả0 trước effect;
nhánh group23 tại1190FCC gọi interface vf1C. Sáu ca native kiểm kind/amount với itemtable/group/
interface model. **Chưa đọc amount argument của chính lần Ctrl rỗng**, không đặt tên field inventory
hoặc dùng emulation thay quan sát hết kim. Timing/fuse là stub choice; một ca không phục hồi mọi
mode/timeout hoặc server inventory validator. Duplicate/stale/multi-charge và Ctrl cạnh tranh touch còn mở.

<a id="death-lifecycle-policy"></a>
### Corpus17→escape→fresh17→19 và admission19 opt-in — CP124

[Recipe/raw/source/hash](../evidence/life-admission/README.md), **kr_runtime confirmed cho corpus/
run trong scope; emu_choice/design cho guard**, không server gốc hoặc full lifeauthority.
Tái dùng CP112 blob/cursor, CP113–121 kim/rescue/reset và CP122–123 directTCP/nativeframer.
12 run đã seal cho64 actualreport:33trap17/20death19/11escape0 hoặc18;14death nối proxyfront,
6death nối directsocketchunks, tất cả có frame/seq/bodySHA/typednativeparser đã lưu. 6127checks;
không RE lại writer/codec/nativebody. Cả20death có17 còn mở theo role/epoch bookkeeping;6death sau
escape đều có17 mới. Corpus chọn lọc không phủ mọi mode/deathcause, không coi marker là worldoracle.

`death_lifecycle: {"mode":"current_trap"}` opt-in trên profile. Với19 hợp envelope/freshness,
trước cursor/needle state/marker/death/forward: đòi connection là current bound mapped live member,
không accepted19 cùnggeneration, marker17 cùnggeneration. Reject ghi reason riêng nhưng không
poisoncursor, forward hoặc mutate actor/inventory. Accepted19 ghi `source:design/generation/trap_sha256`
rồi giữ đường cũ. Không option giữdefault cũ. Một forged17 rồi19 vẫn có thể qua; không location,
distance/timer/auth/genericmode oracle. Existing17/19 sendfailure vẫn exceptionclose/abort, cursor/
marker có thể đã commit trước send; option này không thêm whole-service transaction.

Replay full captured124/127 vào recordingdispatchmodel defaultoff vsopt-in:2229checks,20accepted19/
forwardpayload/rejection khớp. 202server tests gồm11new test firstseenhightick/nocursorpoison/current17/
escape0/18/fresh17/reset/member/accepteddeathhistory/malformed/sendfailure/default/config. Model tách Wine.

Runtime control270,594767466s có R1rescue/fresh17/death2/hashlinks/result/return/Start2; R2park5s
nativeY159→224 chưa279, lỗi controller trước Bbomb/trap/death.1566raw/1651joins giữ, không guardcausality.
Retry355,750586207s/exit0/error0 giữ guard/profile/client/fuse/probes, chỉ parkwindow5→9s/target279
không đổi: haiB17→18/aux0/full1707/nativeboth56→26, sauđó mỗi epoch freshA/B17→19/4guardtraplinks,
chosenrank2/175180/return2/actualReadyB, noStart3.2246basejoins/2130raw+89guardjoins;726actorgetter/
312packet/12peer153 là copied-native emulation riêng. Park đạt299; Up4s399→334→269→194 **chưa159**,
actual18 sau đó; peerquota/fulltrajectory/contactthreshold/causalfix vẫn giới hạn như CP120.
Hai realWineAB/fixedteam/syntheticopponent2/chosentwooutcomes; không fullmatch/K5/K6a/LAN/Internet.
AUD039 định kỳ qua124/kế128; sourcebinary/pack/DB và completeCP001123 không sửa. Bước kế ở PLAN.


<a id="death-envelope-admission"></a>
### Envelope report17/19 trước admission — CP112

[Recipe/hash/control/runtime](../evidence/firstseen-admission/README.md). Tái dùng CP023fullblob,
CP063connectioncursor, CP062claim/actualrace, CP067key/serial, CP075/085TCP và CP110capture;
VN chỉ report/confirm pattern, không portnumeric. **kr_static/emulation** native14EB410 đối chứng cấu trúc;
**design/implementation** guard thuộc emulator, không xác thực chết hoặc server gốc.

C→S127 subtype19 variant đãđo có commonprefix14 +typedu32N=11 +raw11 obfuscated4/4/2/1;
raw=`18 06 00 00 00 00` +inner5byte. Đã có inner `0000000000` và`ffffffff00`, không đặt nghĩa enum.
Subtype17 giữN14/header`18 09 00 00 00 00`/inner8. Helper `supported_event_blob` kiểm exactbody29/32,
N/header trước logacceptedreport/cursor/marker/death/forward; inner tiếp tục opaque. Gói19 hợp lệ
được forward nguyên tail trong170 sauprefix6,17 giữ translationu32 từraw+10.

Baseline nhận prefix-only/truncated/lengthsai và setdeath/phát170. NativeframeCRC vẫn hợp vì góiđộtbiến
đãencode lại, nhưng nativeblobreader readerror54=1. Native extra1 đọc29/30/lỗi0; service exactbodyguard
là policyvariant hỗtrợ, không native-rejectionclaim. Compressed/otherenvelope variant chưa kiểm.
Error mới đi theo service exceptionclose/abort, không silence-drop trong trận hoặc synthesize death.
Unit/directdispatcher có thể tiếp sauexception chỉ làmodel, không mô tả lifecycle socket thật.

**kr_runtime/confirmed hẹp** đối chứngcopyWine/sourceguard mới: run438,200581995s/SHA399d12f6…,
profile47/ExVillage01/Bchief/Aguest/sameTCP. 4actual17/2actual19/12entity+12complete158/view,
2native121clock175180/2outcomesreset, postreturn2actualA95seq823→native110cảhai/errors0/UIready.
Guard malformed chỉ kiểm offline; khôngclaimnativeclientđãphátmalformed. 33oldcaptures qua guard,
161server tests/34matrixjoins/4socketcases/5nativeblobcases riêng, không cộng thànhauthorityproof.

**Phần chưa có:** validfirstseen19 vẫn setdeath mà không trustedlocation/life-state;124 dùng boundslot/
context/exactcatalog/currentkey/oneconsume vàreported-ineligible, khôngactorposition/inventoryauth.
Đó là giới hạn lịch sử CP112: CP113–121 đã tích hợp kim/rescue, marker được clear khi commit0/18.
[CP124](#death-lifecycle-policy) thêm optional19 currenttrap prerequisite; default vẫn nhận firstseen
hợp envelope. Không tự clear marker bằng21 hoặc gọi reported17 là trustedlife/location. Fullauth/
codec/authority/gates/LANInternet vẫn mở; việc tiếp chỉ ở PLAN.


<a id="trap-report-rank-order"></a>
### Kết quả dựa trên report21 cũ và A17 đến sau rank — CP099

[Recipe/oldsourcehash/corrections](../evidence/trap-report-rank-order/README.md),
[serious audit AUD029](../evidence/checkpoints/audits/AUD029.json).
**kr_runtime/confirmed hẹp cho frame/timing; design/currentservice implementation cho finishpolicy.**
Không production/profile/clientpackDB change trong CP099. Reuse14sources CP023B53010/trapB78373/
deathE6EFDF, CP029170/state28/B33270, CP056reportguard, CP085actualTCP, CP098raw và VNconcept.

**Correction nghiêm trọng của CP098:** đoạn “noA17front” sai vì reportanalysis lọc `service_entry` khácnull.
Đây là activeadmissionlog, không đầy đủ nativefront. Raw `target/proxy-A-126-client.bin` vàupstream cùngbody
thật có127/sub17 `[17,0,80,159,4483,5]`, blob8byte`0200000000000000`, nativeframer/typedreader đủ32.
ExactnativefrontSHA/body/outerseq và reencodedupstream khớp retainedcapture. Report tới proxyfront
sau first175send **1,596361ms**, sau `rank.monotonic_ns` **2,417852ms**, khôngactiveadmissionlog.
Service nhận127 chỉ khi `room.active`; saufinish report không vào receive_event/forward170. Không
lấy missingadmissionlog thành khôngnativewriter/phátframe. CP098completedmanifest/raw bất biến;
CP099delta và canonical sửa lại, không viết lại thí nghiệm cũ.

OldR1root6broadcast158 trước ranktick319,9274ms (ownA/receiver timestamp); latestrecipient158 trước
ranktick319,84042ms. LastacceptedA21 trước latestsuccessful158 **105,479175ms**; policy hiện hành chỉ
đòi bất kỳ other21 sau B19 +8s. Nó không đòi21sau latestblast hoặc invalidatecandidate khi mới trap.
Do đó source và raw xác định trigger dùng report cũ qua event thayworld; chưa originalserveroutcomelaw.
Room.finish nằm sau ticknow và trước first175send, là một **bracket**, không gán exactfinishtimestamp.
NativeA56/175entry của oldCP098 vẫn thiếu hosttimestamp, chưa thứ tự nativewriter/175 hoặc persistent
state divergence. Không kết luận rankenum sai hoặc causalobservereffect chỉ từ source/lateframe.

Newread-onlychronology đặt timestamp ởB33270/B332B0(state+28)/postwrite, B53010sub17/19/21,
queue/direct/throttle127, knowntrap/deathreturn, native170/175/180entry. First`trace` probe gán nhầm
E5E8F0(159)là180 rồi tắt sau bom đầu: **partialobserver**, không suy report/175absence. `fixed` sửa180
đúngE64950; `control` thêmOSalignsecondary3,3/gap1s, `gated` bỏhighfrequency probes trướcextra6 để giảm
observeroverheadcandidate. Không runtimegamecall/statewrite/injectreport/đổirankpolicy. Clock→180
nativechronology capture được nhưng ba run đúng **không tái hiện A56**: A0R1/A12R2cảhai view tại175.
Negativecontrol không bác oldlateA17 hoặc chứng minh overhead là nguyênnhân. `gated` còn BadFDerror
connection8 khi cleanup, giữqualitypartial, không nói serviceerrorfree.

Không cộng các check thành cấp chứng minh: fixed3167/control3449/gated507chronorelationships và
2178/2364/390copiednativeactorgetters riêng. Các run có native2resultreset/currententities/195flags
nhưng chưa repeatpositiveoldA56chain. Moduleimportanalysisfirstfail và wrongsiteobserver được giữ;
chỉ evidence/claims đo được nhận. AUD029manualcanonical→rawoldframes sửaabsence/sourcefilter, periodic
CP100 khôngreset. Bước optionalresultfreshnessguard/regression/nativecontrol tại thời điểm CP099 đã
được thực hiện ở [CP100](#result-freshness-guard); [PLAN](../PLAN.md#current-order) giữ bước hiện hành,
không arbitraryquietdelay hoặc gán law gốc. Fullidentity/auth/codec/authority/allreset/vanilla/LANInternet/
fullmatch/K5/K6a vẫn chưa nghiệmthu.

<a id="result-freshness-guard"></a>
### Guard kết quả tùy chọn sau blast — CP100

[Recipe/hash và reuse](../evidence/result-freshness/README.md); câu hỏi/phép đo tiếp ở
[RESEARCH](../RESEARCH.md#gap-result-return), thứ tự duy nhất tại [PLAN](../PLAN.md#current-order).
**design/implementation** cho policy; **kr_runtime/confirmed hẹp** cho native control trên copy.
Không thay luật server gốc bằng suy luận từ report21 hoặc test tự sinh.

Profile khai báo `result_freshness: {"mode": "post_blast_untrapped"}`; vắng field giữ behavior cũ,
khác schema báo lỗi trước selector/runtime. Sau mỗi broadcast158 hoàn tất thành công, service lưu
`result_last_blast` với generation/ID/latest recipient-send timestamp. Lỗi/partialsend không commit
mốc mới; job cũ bị fence không làm đổi vòng mới. Start thay round/clear eligibility17/19; cursor127
qua Start vẫn giữ contract trước. Cả own158 của secondary dù native có thể no-op cũng invalidate21:
đây là lựa chọn bảo thủ, không native law hay quiettimer mới.

Candidate21 phải là report đã admission, đúng currentmember/currentgeneration, sau death và
**sau** latestsuccessful158; timestamp bằng không đủ. Candidate đã accepted17/19 trong generation
không được coi là survivor. Duplicate/stale rejected không tạo transition eligibility. Delay8s từ death
và rankalias cũ giữ nguyên. Cảhai accepteddead dùng optionalrank2/2 fixture CP083/latestdeath;
thiếu alias thì explicitpending, không gán falsewinner. Profile này **không có reset eligibility khi
cứu/kim**; chưa áp guard ngầm cho mode/harness có rescue/selfescape.

127 tới khi vòng không active được ghi `inactive_reports` (role/generation/seq/time/frame/bodySHA/
body/prefix14 nếu đủ). Đây là envelope **chưa admission**, không cursoradvance/forward170/lifestate/
outcome. Prefix ngắn vẫn giữ metadata; không lấy nó làm packet đã semanticvalidate. Khắc phục gap
observability CP099, không biến log này thành authentic/authoritative report.

**Kiểm mô hình:** 124server tests/4terrain kernel tests pass. Script dùng exact retained CP098
B19/A21/root118/lateA17 bodies+seq và timestamps: legacy chốt175 tại oldranktick, guard vẫnactive và
admit actualoldA17. Đây là mô hình service tối thiểu (emptyterrain/newID1, collapsed recipientsendtime),
không native recurrence hoặc mô phỏng đủ thế giới CP098. Reuse source tránh tìm lại writer/codec/rank.

**Native control cùng run:** 341,282s, hai actual121/clock/175/180/refresh mỗi view; 14native159entities,
10successful158 callbacks/view, 44snapshots195flags/process/reset khớp boundedterrain model. Jobs
6/7/13/14 bị `round_closed`, không158/terraincommit, không bịa14callbacks hay mọi entity đãprocess.
12selectedTCPsame-oldreplays bị reject/noextraID165170. Bốn BombFire pickups/delta/reset, rankstorage
1/2 trước180 đều có nguồn riêng. NativeAstate0R1/12R2 vàB1 cảhai view ở175; không A56recurrence.
Candidate21 tới sau last158 lần lượt2934,963410ms/639,368130ms; 2inactive native127/sub34 khớp exactfront/
upstream, không activeadmission. Ba cửa sổ old21→new158 đều **trước** deathdelay8s: run này chứng minh
compatibility, không chứng minh guard đã cứu đúng oldrace bằng native. Exactoldtrace được kiểm model riêng.

Giới hạn: guard chỉ xét blast đã gửi, không guarantee không còn pendingbomb trước175; futurebombs
có thể bị fence khi finish. NativeA56/writer→rank exactoldchain chưa tái hiện; negativecontrol không
bác oldlateA17. Report/life authority, forgedforward/location/fullauth/codec/allreset/rescuevariants/
vanilla/fullmatch/K5/K6a/LANInternet vẫn mở. AUD030quaCP100/nextCP104. BadFD cleanup là trigger độc lập,
không gán olderrno9 cho một stack nếu chưa có trace.

<a id="selector-closed-readiness"></a>
### Key selector còn sau cleanup đóng recipient — CP101

[Recipe/hash](../evidence/selector-closed-readiness/README.md), **design/currentservice implementation**,
không là hành vi binary KR/server gốc. Tái dùng CP085departure-send-failure và CP099gatederrno9;
không tìm lại packet/lifeenum hoặc port asyncio/session VN thành Pythonselector.

Real Linux socketpair A/B đều remoteclose → actualselector trả cùng batch [A,B]. Xử lý AEOF trước
kích hoạtcleanup gửi107 tớiB, EPIPE được CP085 ghi đúngrecipientB rồi closeB. KeyB đã nằm trongbatch
vẫn được dispatch; `receive(B)` trước sửa gọi recv trên fd−1 → **errno9**. Đảo thứ tự tương tự vớiA.
Đây là đường lỗi xác nhận trong service model; oldCP099gated chỉ có errorstring/ordinal8, **chưa đủ
stack để gán chắc cùng đường**. CP100native errorfree cũng không bác hoặc tái hiện olderror.

`receive` nay trả ngay nếu event đã có `closed` do serializedowner đóng; không gọi recv/advancecursor/
parse/report/outcome. Không bắt/nuốtEBADF chung: descriptor bị đóng ngoài lifecycle mà event chưa
marked vẫn raise OSError. Cleanup/attribution/idempotency/successprefix và lỗi fileENOSPC giữ nguyên.

129server tests pass, gồm 5ca mới: actualcachedbatch hai thứ tự; repeatedclosedread no mutation;
AEOF cùng Blivebytes vẫn đọc đúng; unmarkedbadfd vẫn lỗi; actualrunloop dispatch/save/finally không
spuriouserrorrow. Pre-fix2tests fail trực tiếp tại recv/errno9, source/reproduction giữlocal/hash.
Không chạy Wine lại: nhánh thay đổi chỉ already-closedno-op, live path có positivecontrol; native
controlCP100 là baseline lịch sử riêng, không giả newnativecausalfix/fullmatch/gate.

Giới hạn: chưa generalbackpressure/sendpartialrecovery/EOFvariant/accountresume/authority/fullreset/
vanilla/LANInternet/K5/K6a; bước integration tiếp chỉ ở PLAN. AUD030qua100/next104.

<a id="drained-result-local-clear"></a>
### Drain job hiện hành và transition local trước report21 — CP102

[Recipe/hash/failures](../evidence/drained-result-local-clear/README.md); gap/phép đo ở
[RESEARCH](../RESEARCH.md#gap-contact-state), thứ tự tại [PLAN](../PLAN.md#current-order).
**design** cho optionalfinishpolicy; **kr_runtime/confirmed hẹp** cho nativepath/timing/state.
Không coi report21 là authenticatedalive hoặc luật winner gốc.

**Reuse và trigger:** CP100jobs6/7/13/14 đã có nonzero native159entity cảhai view, chưa observedprocess,
creation+6s earliestdeadline sau175 khoảng0,679–4,900s. Deadline thật được schedule saucreated_ns,
nên đây là bound, không exactduetime. Chưa sampled nativebombstate tại175. Exact retainedCP100body/
seq/time service model phân biệt nondrainedfinish vàdrainedwait hai vòng; emptyterrain/noquota/newIDs/
collapsedrecipienttime là model, không fullworldreplay. VNsession finish<=1team là policy reference,
không luật chờbom; không nhập VNopcode/enum/timer.

**Mode mới:** `result_freshness: {"mode":"post_blast_untrapped_drained"}` là thử nghiệm độc lập.
Trước survivordecision phải hết currentgeneration/unfenced jobs; capacityrelease sau chain không
retire ownsecondarytimer. Mỗi successful158 lại invalidateold21 theo CP100; không quiettimer/fuse mới.
All-dead vẫn dùng CP083latestdeath/knownrank2fixture riêng, không bắt buộc drain vì không còn survivor.
Absentfield và modeCP100 giữbehavior; Start/fence/currentmember/failedsendboundaries giữ nguyên.

**Blocker đã đo trong bản thử đầu:** 7bom/7successfulnative158 nhưng permanent17exclusion giữ vòng
mở dù A phát265accepted21 sau17, noA19/no175180. ActualAstate56→26 trước21; không report1/0/18,
chỉ Bomb/Fire đã nhặt, không ép state/gamecall. Không gọi run491s/0errors này là fulltargetpass.

**Nativepath định vị đúng câu hỏi:** reuse CP036cached `E6F370` cho thấyinterface+98gate ởE6F48F,
callactorvf148 tạiE6F4AA rồiwriter21E6F4DD. Targetedread-onlyrun đo gate1, actualcallee**B43320**,
args **(0,0,-1)**, cùngactor pre56/post26 tạiE6F4AD, rồiactualwriter21state26→nativefront→serviceadmission.
12mechanicaljoins/18copiednative52A350fields; hai static/runtimePE/modOrange bytes matched, B43320
range180 riêngpinVA/RVA/SHA, không đoán toànfunction. Đây là localtransition đo được, không semantics
mọi21/character/mode hoặc nguyênnhân relay/desync. Không dùng own-onlyACK21CP036 làm peerproof.

Trong mode drained, resultcandidate21 được phép sau **accepted17 mới nhất** và latestsuccessful158;
**accepted19currentgeneration vẫn loại vĩnh viễn** kể cả sau đó có17/21. Không xóa pickup_ineligible:
quyền nhặt/rescue/kim/inventory không tự mở từ report21. ModeCP100 vẫn permanent17/19exclusion.
Timestamp added vào marker chỉ provenance; fullreportauth/location/firstseen vẫnunknown.141tests gồm
pendingfuturejobs/oldfence/reset/failedsend/chainreleased-ownjob/fresh21post17/laterblast/restale/
death19→17→21/missingall-deadalias và legacy. Guarddeathhistory cuối thêm saulaunch; live source pin
và finalsource khác biệt được ghi `postlaunch-hardening.json`; variant đó chỉ model, không giả đãnative.

**Currentrepeat 348,660s:** hai native121/clock/175/180/refresh mỗi view,14native159entities và14actual158
callbacks/view,60snapshots195flags/process/seedreset khớpterrain; nofencedunsentjobs.8same+4oldTCPbody
replays reject/noextraID165170; actualA17 mỗi vòng tạo2remote170 thêm choB, không copy expectedB-only
CP100. Có6mature21/future-jobwindows khideathdelay8s đã đủ; newmode vẫn chưa rank, chờ đủjobs rồi
fresh21. Native clearpath56→26/gate1/B43320(0,0,-1) trước21 lặp cảhai vòng.

**Blocker chưa giải:** pre175 AviewA=26, BviewA=56, cùngXY80,159 trong cảhai vòng; Bstate1 ở cảhai view.
Perviewnative175timestamps riêng, không gánsameglobaltick/persistentuniversaldesync. Servicehiệnchỉ
forward17/19 nên accepted21localclear chưa cópeer17021; cần đo actualreceiverE86CA0/gate/vf148/prepost
vàreaderfull trước175. Không nâng hai cycles/rankstorage thành trậnđồngbộ/fullmatch/K5/K6a.

Giữ inputfailure B118absence và startupreset104 trướcC2S6/clock, controlledstop/repeat raw/limits riêng.
StableBcolumn200 bằngOSfeedback là inputcontrol, không gánfocus/time nguyênnhân hoặc serverfix.
No sourcebinary/pack/DBchange; currentmodifiedWine/AB/ExVillage01/localstub. Originalauthority/
auth/codec/fullvalidator/allreset/vanilla/LANInternet vàfullmatch/gates còn mở; AUD030qua100/next104.

<a id="ending34-state-boundary"></a>
### Handler175 tự chuyển state và report34 trước lần ghi — CP104

[Recipe/hash/reuse](../evidence/ending34-state-boundary/README.md), gap ở
[RESEARCH](../RESEARCH.md#gap-result-return), thứ tự duy nhất ở [PLAN](../PLAN.md#current-order).
**kr_runtime/confirmed hẹp** và **kr_static/confirmed cho boundedmatchedranges**. Không production/
profile/newpacketpolicy change; không gánstate3 là universalalive/victory hoặc report34 làauthorityACK.

Source-first15refs currentCP103raw/CP023175/CP036E6F370/CP083result/VNconcept/KRdata. Existingraw2cycles
CP103: A34front khoảng129,092/139,416ms sauown175entry, Astate3/Bstate1cảhai view tại180. Chưa cóexact
writer34/caller, nên chỉ thêm B53010sub34/backtrace vàstackstate3/26sau175 trongobserver đã có; không
lặpwholecodec/contact/kim hoặc dùngown-onlyACKnegative thànhpositive. Onecycleboundedstop sauactual
175/180/returned585/refresh cảhai, không giảtwo-cycle/fullmatch.

**Nativepath đã đo đúng:** ownA175entry state56 → nativecallactorvf148 trong175 tạireturn**E603E6**,
callee path **B43320/B43427** → state26. Sau đó175callactorvf13C tại**E6049F** → **B74060/B7409A →
B42880/B428F9** → statewriterB51CF4/B33270 đặt3. B cũngqua cùng175callpath đặt3 từ26. NativeB sauđó
có3→0→56→3 ngắn, setters56/3 cóstack **B4EAA6←E668B2←125DF33**, là cạnh UDPapplyđã biết; exactpacket/
argstate/time củawindow này còn thiếu, không gọi ngayoldUDP hoặcnewgenerationfailure.

**Report34 không nằm sau completedstate3:** trongpathtrên A gọi B53010 subtype**34** tạireturn
**B51CCD**, snapshot cònstate26; rồinativeframeđiTCP, sauđóB51CEF gọi B33270 vàghi3 tạipostB51CF4.
Matchedstaticrange **B51CB0** cópush22/callvf21C/send1164D50 **trước** callB33270, khớpruntimeorder.
Body chỉcommonprefix14: **[34,0,80,159,4800,6]** trongrun, khôngblob; timestamp4800làcounterđãbiết,
fieldcuối6chưagánmeaning. Exactactualnativefront/upstream SHA/seq/framer/typedcursor14/errors0 khớp.
Serviceđãclosed nên chỉinactiveenvelope, khôngadmission/forward170/outcomechange. B khôngphátown34
trongboundedpathnày; khôngsuymọi remote sẽ khôngcó34.

**Quan sátboundary:** A175entry56→26→34writerstate26→3; B175entry26→3→0→56→3; trướcnative180
cảhaiA3/B1. Direct175→state3stack vàstaticB42880setprotected+5A0=1/callvf204(state3,aux922) làsource,
nhưng tênphase/gatefull vẫnunknown. B74060cònbranchcallD1CD10 vớimessage1A8; không gán tênhiệuứng.
Report/before/after lànhữngphasekhácnhau: CP102/103same-state-atown175entryassertion **khôngđủ**
làmstandardtoànending hoặcpersistentdesync/rankwrong. Strictfail lịch sử vẫnđúngnumericboundary,
khôngxóa/bịa pass; CP104bổsungprovedhandlerconvergence/unknowntransientUDP. #166/gate vẫnopen.
Không cần thêmownorigin21ACK đểđạtconvergenceởrunđãđo; chưathửownACKcontrol/lawrecipientmọi mode.

Run225,299s,1actual121/clock/175/180/returned585 mỗi view/7current158jobs,0controller-serviceerrors.
27mechanicalendingjoins/60copiednative52A350fields,282typedpacketvalid/badCRC/partialcases làemulation
riêng; sourcebinary/pack/DBgiữnguyên. Static4boundedruntime-ledrangesE60371/B74060/B42880/B51CB0
matchedstatic/runtimePE/modO;base400000/RVA=VA−400000. Firstinterior-byte decodes/firstlinearretmislabel
đượcgiữ/correct, khôngdùnglàwholefunction/exitproof. No newgamecalls/stateforcing/fuse/quietdelay.

Proxyfixedtrongrun còn bridgebootstrap **ordinal1/WCP-A-1** trên3858: exact15bytehello/nguyênbyte,
không C2S/channelframer; actualA4/B4vẫnroleđúng. Đây lànativebranch1proofhẹp, khôngđánhđồngbench
WCP2/66byte vớinewrun. AUD032manualcanonical→raw/CP101–104/source/auditcadence qua104, next108.
Fullidentity/auth/codec/authority/fullvalidator/allreset/endingUDPvariants/vanilla/fullmatch/K5/K6a/
LANInternet vẫnchưa nghiệmthu; nexttargetedpost175UDPstateapply ở PLAN.

<a id="round-finish"></a>
### Trap/death nhất quán → kết quả → vòng kế và host EOF — CP039

**kr_runtime/kr_static confirmed hẹp trên copy Wine**, [recipe/control/hash](../evidence/round-finish/README.md).
`finish-first` giữ8connection/cùngIDs1/2/slots0/1/session1; actual95 củaB/104củaA,121full129/129,
99/197/196/153/clock hai lần. Vòng1 một145pickup, hai bom A:17→1→0→17→19;
vòng2 phát582list rỗng, một bom A:17→19. Không có1/0 vòng2. Không thêm patch/ép state.

**Kết quả đã gắn state peer:** trước175, actorA local/remote đều1 ở cả hai vòng, khác control
CP028/029 chưaforward170. Stub chọn outcome Alose/Bwin và chỉ phát175 sauactual19vàB21;
175outer302/302/inner292/292/errors0,1803/3/errors0, cả hai vềstage12/phòng.
Sau room110 refresh, click95/104 thực khởi động vòng2. Ảnh hai lần cùngLOSEKRLocal/PERFECTKRGuest;
ownhighlight theoID. Outcome/delay/fuse/roster/ACK là emulatorchoice, chưa server arbitration gốc.

**Report21 hẹp:** mỗi vòng7actualB21 trước175, không có21 sau175 tới hết run;
last21→175 là0,801s/0,630s,175→180 là5,077s/5,080s. Runtime writer E6F370 và callback rank
được trace; không chỉ log stub. ActualB127/sub34 xuất hiện khoảng0,448s/0,446s sau175,
callerB51CCD; semantics **unknown**, không đặt tên ACK. Chuỗi175/180 không thay toàn validator21.

**Reset đã đo:** clock vòng1/2 có state/coordinates(+28/+4C/+58), stat(+4A4/+50C/+4BC/+524)
khớp riêng từng actor/client; ownstate24/remote15 lúc clock và spawn(120,159) trongprofile này.
Ba actual118/159/158 liên tục; UDPport từ99 giữ nguyên qua hai vòng, wireseq riêng liên tục không reset.
Sự khớp các field/port không chứng minh mọi entity/timer/peerqueue/generation đã reset.
Aownavatar ở phòng là silhouette trắng nhưng BthấyAđỏ; cosmetic/profile/animationcause cònunknown,
không gọi mọi cache/profileđúng. Scene/gameactor/input vòng2 và statepeer được kiểm riêng.

**Host EOF:** `host-EOF-control` kill wineserver chỉprefixA sauclock; channelAEOF thật,
Bchannel vẫn mở và inputRight đổi vị trí/ảnh, actorA còn hiện; không tự result/return trongcửa sổđo.
Không cósynthetic107/175/180/19 ởcontrol. EOFkhông đồng nghĩa death được client tự xử lý.
`host-EOF-policy` giữ đối chứng riêng: EOF đúng Achannel→chờ12s→175choB→5s→180choB→
107xóaA/110promoteB ởphòng. B175full302/inner292,180full3,107base6 rồi wrapperrewind/final1,
slot0ID1→0, localchiefB→1/nútStart. 107chỉ gửi sau vềCPrepareStage; không giả định107vào trận đã xử lý.
ActorA trênB state0 trước/sau175 vàtrước180, không cóactual17/19; **forfeitpolicykhông giảlậpcái chết**.
Ảnh cuốiBởphòngmộtmình/chủmới; chưaStart/rejoin hoặc cảhai disconnect. Đây là **emu_choice/design**,
không serverauthoritygốc. [Bước tiếp](../RESEARCH.md#q-023).

Chưa toàn reset/validator/generation/disconnectvariants/rejoin/fullauth/cache/codec/vanilla/
LAN/Internet hoặc K5/K6a. Đây là ca hai vòng thử có kết quả/statepeer nhất quán hẹp,
không tự đổi thành nghiệm thu toàn trận đầy đủ.

<a id="all-dead-result"></a>
### Kết quả cả hai bị loại, rank2/2 — CP083

[Recipe/nguồn/raw/hash](../evidence/all-dead-result/README.md), **kr_runtime/confirmed hẹp** trên
current modified Wine/local service. CP082 đã đo cảhai actual17/19/no21, policy chờ other21 không
ra rank trong boundedrun. Tái dùng KR CP023/039 raw175/180, header16/row131 và lossrank2/native
consumers; VN oc_e-game/result chỉ gợi checklist all-dead/result/reset, không cung cấp enum/layout KR.

Candidate raw292 chỉ đổi byte152 của rank175-v2 từ rank1→2; row0 đã rank2, không invent drawenum.
Native header E3EB20 +16 rawrecord E4ACE0 (allocator/sidelist modeled) đọc đủ292, rankstorage+8 ở
hai record đều2; nativeframer/actualtyped175 full302 corroborate bytes. Emulation không proof UI/
manager runtime. Gói175 gửi bằng seq hiện hành mỗi receiver, raw/hash pin riêng; không copy opcode VN.

**Policy design:** chỉ đúng hai live mapped A/B, mỗi vai có accepted subtype19/death_event cùng
round/role/slot hiện tại; chờ8s sau death cuối rồi finish/all-dead rank2/2. Trong lúc chờ không dùng
stray21 để chọn winner theo death đầu. Alias optional `rank_all_dead`; profile cũ thiếu alias ghi
`all_dead_result_unsupported` một lần và giữ pending, không tự suy winner. Return5s/refresh3s và
single-survivor cần actualother21 giữ như trước. Duplicate death không dời grace; begin vòng mới
xóa marker; đây vẫn là report-based fixture policy, chưa authority/actor/distance validator đầy đủ.
Exporter chỉ pin path/size/hash/seq91, kiểm basic175 trước output; không tự chứng nhận semantic.

Final `postparse` run sau guest Back/rejoin: hai keys12345→12346, IDs1..4, actualA3,3/power2 và
B3,5/power1, mỗi vòng Fire1→2/baseline1, root158 xử lý cảroot+secondary/ownlater khôngduplicate ở
hai view. Cảhai actual17/19 accepted; state+28=1 của cảhai actor trước175 trong mỗi view, không
actual21 sau deathcuối→rank. Native175 full302/180full3/roomrefresh hai lần. Trước native180 mỗi
view đọc manager row0/1 type1/rank2; UI mỗi vòng/cảhai client hiện **LOSE!!** hai hàng; cuối room
có KRLocal/KRGuest. Đây là representation “cùng bị loại”, không kết luận nguyên bản có tieenum.

**Probe limitation giữ rõ:** first `all-dead` run đủ lifecycle nhưng strictplanned fail vì không có
marker E3EDBE sau postparse. Repeat chuyển read-only probe tới callback branchjoin E5F7F8 và
trước existing180; chỉ mẫu **before180** xuất hiện, branchjoin cũng không có mẫu. Không suy hai
địa chỉ đó đã chạy, không gọi đây là trace đúng nhánh postparse. Native storage trước180 + actual
blob/UI là proof dương hiện có; nguyên nhân khônghit hai probe vẫnunknown, chỉ trace khi chặn
integration cần thiết. First failure/raw giữ, không hạ assertion hoặc tự tạo trace để pass.

Chưa fullmatch/K5/K6a/vanilla/LANInternet/fullauth/codec/validator/authority/toànreset/generalmembership/
ba bom/vật cản/allchain. Thứ tự duy nhất ở [PLAN](../PLAN.md#current-order), câu hỏi ở
[RESEARCH](../RESEARCH.md#gap-result-return); CP082 single-survivor và CP083 all-dead không cộng thành gate.


<a id="promoted-battle-cycles"></a>
### Battle sau Bpromotion: partial tại input-control — CP108

[Recipe/hash/strictfail/boundedproof](../evidence/promoted-battle-cycles/README.md),
[gap](../RESEARCH.md#gap-room-membership), [thứ tự](../PLAN.md#current-order).
**kr_runtime/confirmed hẹp** R1 trên modifiedWine/AB/ExVillage01/local/CP107opt-inprofile47;
**unknown** haiBattlecycles/ending/reset saupromotion. Hai partialruns khôngghép thành hai trận.

Reuse CP106currentAPI/key/target/route/terrain/guard và CP107A86/B107/chief1/A82/guest0/ReadyA95/
StartB104. Khôngnewnativebody/codecRE hoặcđảoactororigin theo chủ; production/profile/fuse/IDslot/
geometry/items khôngđổi. r1controller3sneutralBstability timeout trước118; r2chỉmởneutralbound6s,
vẫn3equalfreshsamples, quaBcolumnalignment. Khôngkhẳngđịnh thaywindowlàcausalfix củađợtr1.

Mỗirun R1có uniqueAPI→packet13/key/args→targetchange/currentXYprogress cảhai chiều, haiBombFire
pickup/native165callback12/errors0. r1/R1:223relations/144copiedgetters/18CRC; r2/R1:337/216/27.
r2statBomb/Fire1→2 cảhai view có8copiednativegetterjoins, chưa secondStartbaseline/reset.

r2 actual118/159IDs1..3/ownerA,B,A/cells(3,3),(3,5),(4,3), native159full39/3entity/view. Hai service158
được phát ID1/2; **nativeA first158 begin/end=1/0, B=0/0** khi controllerstop, khônghai callback hoàn tất.
Before158flagsA đãđọc nhưng khôngafter; full-completion chainverifierfail giữ. Ba195flagsnapshots/
591copiedgetters/585arrayaddresses chỉclockA/B vàAbefore158, không chứngminhterraincommit/resetmatch.

Capacityphase bắtđầu1796500329337866ns; cachedsample9,306s trướcstart khôngfresh. Hai freshsample
Astate23/Y294 lúc1796501086073221 vàY339 lúc1796502153465798; targetY≥389 chưađạt khirelease/
controllerbound~3,049s. FirstA158 đangcallback cuốiwindow; không175180/report17/19/bằngchứngAtrap.
Khônggán hỏnghostrole/transport/observer từtimeout. Bound6s/freshfeedback/handlerboundary là phép thử
kế theoPLAN, khôngsửathời gian nổ/policy hoặc ép actor đạt target.

48boundedchecks chỉpartial/controlscope; strict4phase/two-cycle remainsfailed. Oldemptyround_history
assumptionfail đượcgiữ: controllerarchivepartialroundonexitkhôngphảicompletedresult. Scopedverifier
kiểmkhôngrank/return, không xoáraw. Source/profilesealed/checks/hash riêng, 157server tests reuse.
AUD033manualperiodic qua108/kế112 sửa stale navigation/PRstatus, không đổi scientifichistory.
Snapshot CP108: CP107/PR175 đãmerge; CP108 khi đó partial/chờPR/task176open. Hiện CP108/PR177 đãmerge và CP109 dưới đây giải phạm vi hai native cycles; không sửa raw/record CP108. Fullauth/codec/validator/authority/general
membership/fullreset/vanilla/fullmatch/K5/K6a/LANInternet vẫn mở.


<a id="promoted-outcome"></a>
### Hai nativeoutcome/reset sau host đổi vai — CP109

[Recipe/hash/phases/giới hạn](../evidence/promoted-outcome/README.md); [gap](../RESEARCH.md#gap-result-return),
[thứ tự/issue](../PLAN.md#current-order). **kr_runtime/confirmed hẹp** modifiedWine/AB/local/ExVillage01/
CP107profile47/sameTCP/Bhost/Aguest, production/timer/profile/binarypackDB không đổi.

CP108capacity3sboundkhôngđủ: runmớiY294→339→399 tại3,130s/release3,179s, firstA158đãcomplete;
khôngsuy mọioldfailure cócausenàychỉtừrepeat. Upbound4→8 choR1nativeY119 sau4,245s, R1endingreturn.
R2runpartial bắtactualA17/native56→53/freshY302→258; aliveescapeY199 khôngđạt trong8s.
Reuse CP036actual17trap/report19 vàCP083existingoutcomepolicy; controllercuối **nhảUp khiacceptedfreshA17
sauinputstart/cùnground**, khôngtiếphaiSpacephụ, chờnativeoutcome. Không épstate/report/fuse/coordinate.

Finalrun382,456695112s cùnghai175180/585/refresh/actualReadyA95/StartB104/hostB/IDs1,2/slots0,1:
R1IDs1..7/R2IDs8..12, **12actual159entities/12complete158 mỗi view**,52map snapshots×195flags/process/
boxdelta/seedreset khớp. BốnBombFire165/stat1→2/secondStartbaseline1, 8same/4oldTCPbody11812412717,19
bịloại/khôngID165170thêm; proxywriteremissionkhácactualfront. FourUDPphases uniqueAPI/key/args/
targetchange/currentprogress. 500peer/132integration/232transport/25membershiprelations,324copied
peergetters/40CRC/10244mapgetters/10140addressmodelchecks riêng, khôngcộngthànhproofnghĩa.

Cảhai outcome là **post_blast_untrapped_drained**, cóB19/A21; all-deadaliasprofileđãcó nhưngkhônghit.
KhônggọiA17làAdeath hoặc2extrasđãđặt. **Native180selectedstate/XY** hai view khớpA3/B1:
R1A80,164/B200,159, R2A200,317/B200,159. Nativepostreturn110Bchief22/Aguest0/ReadyA/BStart2cycles
đãkiểm; perview175entrykhôngphảiglobaltick/equalitycriterion. ID/host/policy lựa chọnemulator.

#176narrowacceptancehai nativecycles cóbằngchứng; review/PRdelivery theo [reconciliation](../evidence/issue-reconciliation/README.md),
khôngđónggate. Ảnh R1returnedroom đãthấyReady/Start; finalsecondreturn haiảnhđen nên render/input
responsive vẫn **unknown**. Khônggọilàhanghoặcfullmatch; kếreuseCP074focus/wait/owner/nativePrepare/
Readywriter theoPLAN. Fullauth/codec/authority/fullvalidator/generalroom/allreset/Windowsvanilla/
K5/K6a/LANInternet/fullmatch vẫn mở. Giữ cả4rawrun/startup/partialfailures; sourcefirstreuse/typedprefix/
missinganalysisartifactcorrections ghiởevidence, khôngsửa scientificraw/historicalcheckpoint.


<a id="current-basic-cycles-291"></a>
### Basic hiện hành sau291/importABI: hai outcomes/reset, peer và EOF — CP188

**kr_runtime/confirmed trong copyWine4b781f…/privatevirtualhosts; inputcontroller/fuse/drop/forfeit là design.**
[Recipe/run/hash/partials](../evidence/peer-endpoints/README.md#cp188), [K5 scope](../PLAN.md#k5-acceptance).
Tái dùng CP1712cycles, CP1734peerwindows/Bowner, CP174EOF, CP175entrycolumn, CP187own291/copyABI;
không thân hàm native mới, không chuyển opcode/layout VN/wiki. Profile47fixture/policy đúng basicexport
đã đo, chỉ optinown291/nameguard; client.image nguyên byte, importcorrection giữ nguyên.

`k5-neutral-keyup-r2`:413.403716917s/1450raw/frozen167/exit0; hai normaloutcomes/reset/Ready2 rồi normal
BStart3, native121/99/clock cảhai, quan sát clock3≥9.1409s, không thirdbomb/outcome. A nhặt17/27/28,
nativebaeffects/195seed/stat/key reset; một165/view/index/epoch.16neutralwindows R1 đủ3freshstable và
OSkeyup0; **R2coarse route đã nhặt item nên không chạy exact R2-hidden-row cũ**. Không suy oldstuckkey
rootcause/fix. R1 trước đó bị gián đoạn ở wineboot−15/preclient/zeroTCP, recovery12raw/frozen167/cleanup;
không coi preclient interruption là gameplay negative hoặc quy nguyên nhân reboot/SIGTERM.

`k6-current-peer-cycles-r1`:453.875873934s/1486raw/frozen167/exit0; currentABnative291/names128/128/errors0/
renderguard1, hai full175302/1803/errors0 mỗi client, Ready2 rồi Start3. Bốn finiteRight windows A→B/B→A
qua2keys có actualUDP/uniqueingress/nativeapply/targetchange/currentprogress và getter model432checks.
Mỗi vòng B,A,A Space thật tạo3bom/IDs1..6/powers1,1,2, matchednative15939/15813/sameentity/195flags;
hidden27/28 từ actualblock/reports142 và floor17 có ba nativeeffect/statbaseline reset. Hai view loser56→1/
other0 trước175 trong cả2epoch; snapshots không global-sametick. Nativecapturedfield/getteremulation và
servicepolicy tách nguồn. Không intentionalEOF/oldUDP/contenders trong positive run này.

`k6-current-host-eof-r1`:374.766890422s/1158raw/frozen167/exit1, một normalending rồi
**alignmiss:R2-A-hidden-row**.20neutralattempts đều OSkeyup0/3freshstable, A vẫn x80/y399 khi targety159;
chưa tới intentionalfault. CP175 đã có entrycolumn120 trước upwardfallback, nhưng recipeCP174 bị dùng
lại thiếu bước đó. Đây là inputroute gap có nguồn, không chứng minh lỗi physics/transport/keyrelease.
R2 chỉ thêm đúng boundedentrycolumn CP175 trước hidden-row, giữ20attempt/.045/tol12/3s/30sload và
source/config/copy/fixtures;465.448559303s/1492raw/frozen167/exit1. Entrycolumn120→upwardfallback đã
qua, hai normaloutcomes/reset/Ready2/Start3 và pendingID7 được đo; **ownershipimagepath cũ bị FileNotFound
trước kill**, nên không intentionalEOF. First-two verifier kiểm riêng phần đã quan sát, giữ error/exit1.

`k6-current-host-eof-short-r2` rút gọn chỉ phép EOF: initialroom/hostBack/rejoin/firstload/clock→BSpace→
ownedBkill trước6sfuse; survivingAforfeit175180 rồi107slot1/id2clear/110chief1/room. Pendingjob bị
round_closed, không158 qua due+4s; B đã dừng không chứng minh nhậnforfeit. Short-r1 trước đó KeyError game_dir trước fault được giữ riêng. Short-r2 lấy ownership image từ
exactargs.game_dir siblingB/args.exe, đối chiếu SHA/mapping và giữ mọi PID/ns/prefix/startticks/cmd guard.
Run này **không có normaloutcome/two-reset/Start2**; hai normalcycles đến từ positive run riêng và R2
trước patherror, không ghép chúng thành một run.
Không Start4/UIBack/accountresume/generalEOF; forfeit của control không phải trận bình thường.

Tại CP188 Codex rà [authority](rust-boundary.md#authority) và nghiệm thu K5basic; currentbasiccontest khi đó còn thiếu.
[CP189](rust-boundary.md#current-basic-contest-reset) bổ sung ca mới và K6a scope, không sửa kết quả run CP188.
OldCP155 exactoldUDP/controlledcontest và CP159production immediate165 giữ scope/codeprofile riêng,
không giả là newlymeasured hoặc same-run với currentbasic. Fullauth/distance/142freshness/originaldrop/
resetvariants/allmaps/fullcodec/vanillaWindows/physicalLAN/Internet/4–8actualclient vẫn unknown/backlog.

<a id="basic-battle"></a>
### Một profile floorRoller + hiddenBomb/Fire — CP069

[Recipe/hash/native](../evidence/basic-battle/README.md), **kr_runtime/confirmed hẹp** cho integration,
**design** cho placement/rank/fuse/reportguards. Tái dùng CP065floor17/Roller và CP068hidden27/28/
142/reveal/capFire, không mở consumer/thân hàm native mới hoặc sửa service/binary.
582counts1/2/0/inner36 ghép exact3record đãbiết; nativeblob/11ED590/fullcursor/CRCpartial kiểm trước run.

Một run8connections/2cycles/distinctspawn, actual12417→27→28 mỗivòng/currentkey. Nativebothviews
A(capacity,Fire,Roller)1/1/5→1/1/6→2/1/6→2/2/6, reset1/1/5 trước lần2; B giữ. HaiFixedBox mỗi vòng
stored→revealCGOItem→165lookup samepointer;142 hairole trướchiddenpickup.4actual118/IDs1..4/serial0,28/
power1→2, Astate1bothbefore175/haikếtquả/return/refresh/errors0. Same-runthreeeffects, không gộprun cũ.

Exporter `--item-layout battle` chọn knowncombinedfixture/catalog;floor/hidden/defaultfloor giữ;
profileJSONcũ khôngtựmigrate. Đây là basicdiagnosticBattle profile, không toànitem/dropalgorithm/
fullmovementauthority/chainvariants/fullvalidator/reset/K5/K6a/LANInternet. Originalbinary/DB giữ.
Phép kiểm tiếp [PLAN](../PLAN.md#current-order) tách nativepositionagreement với UDPtransportcausality;
không cần quét toàngrammar trước đúng observable còn thiếu.

<a id="rescue-round2-feedback"></a>
### Fresh rescue sau result/return/Start2 — CP120

**kr_runtime/confirmed hẹp**, [raw/control/hash](../evidence/rescue-service/README.md#cp120).
Giữ CP119 service/profile/fuse/client; đổi normalR2input sang read-only feedback/release và finite
peerapplywindow; bỏ probe Start đã giải. Mộtattempt dựngprefixB quá35s, noBattle/146raw giữnguyên;
retryprefixcopybaseline/Winebootstartup60s, không đổi deadline gameplay/guard/outcome.

Run300,388426598s: R1B17→18/aux0, sau normalASpace/BSpace actualA/B17→19, chosen175/180/
room; actualB95seq262/A104seq304, R2native121155/clock/153key12346 (R1key12345), cùngconnections.
FreshR2B17seq322→18seq331/aux0, một170A/full7/7/errors0, B local56→26 vàApeer56→26 lần hai.
KhôngacceptedR2death19 trước rescue, marker/death reset cho phépadmission theo service. Đây chỉ
**mộtchosenresult/return hoàn tất + R2activeescape**, không hai trận hoàn chỉnh/toànreset/K5/K6a.

**Feedback/peer giới hạn:** parkA nhảDown khi sampleY304 vượt target279; mẫu đầu lúcUpY399,
trong4s chỉ thấy399→339, **không đạt target159**. Saukeyup, actualB18 vẫn xuất hiện khi nativeBworld
snapshotA(120,189), Btrap(120,159). Không suy ngưỡngchạm30 hoặc feedbacktarget gâyrescue.
PeerBwindow nhận64packet/quota, keywire59=expected59, cótargetchange/modelprepost, khôngđủ toàn
trajectory. Observerpark khôngready/khôngsamples, touchquotaexhausted; không gọi đầy đủ mọi peer.

CP119controlR2B17→19/no18 giữ nguyên; runmới khácOSrelease/probe/sourceboot nên chưa định danh
causalfix hoặc loại trừ UDP/input/timing/state. Capturedsecurefield/1002getter+621snapshotcheck là
copied-native emulation riêng, không runtime thứba. Sourcebinary/DB/pack không đổi.
Phép thử kế của CP120 đã có kết quả [CP121](#rescue-two-results); giới hạn feedback/causal ở trên vẫn giữ. Fullauthority/vanilla/LANInternet/gates giữ mở.


<a id="rescue-two-results"></a>
### Hai vòng cứu → kết quả → về phòng và ReadyB lần hai — CP121

**kr_runtime/confirmed trong scope**, [recipe/hash](../evidence/rescue-service/README.md#cp121).
Run357,734178034s/exit0/error0, giữ nguyên CP119 service/profile/fuse/client và các probe hữu hạn
của CP120. Delta chỉ normal A Space/B Space sau R2 escape và recipe return/Ready đã biết;
không phát hộ report18/19/118 hoặc ép actor/outcome. Hai epoch key12345/12346, native121 đủ155,
clock/153 mới; mỗi epoch actual B17→18/aux0, một peer170A đủ7/7/lỗi0, native B local/A peer56→26.
Sau mỗi escape mới có actual A/B17→19; service chọn `all_live_AB_accepted_dead_rank2`, cả hai đọc
175/180, về phòng. Sau return2, **actual B95 seq439**, frame proxy/client/server cùng SHA,
không104/Start3. Ảnh hai view đã vẽ phòng và nhãn ready; ảnh không là oracle tọa độ.

Đây là **hai chu kỳ rescue/result/return có bằng chứng cùng run** với hai client Wine sửa bản copy,
đội cố định + đối thủ tổng hợp slot2. Result/rank và roster là `emu_choice`, không luật server gốc.
771 captured actor getter checks,312 native packet cases và12 peer153 cases là emulation trên code
copy riêng; không thay thế runtime semantics. Giới hạn feedback/peer quota và CP119 negative ở
[CP120](#rescue-round2-feedback) giữ nguyên, không kết luận causal fix/contact threshold/fullmovement.
General reset/inventory/authority/auth/vanilla/LAN/Internet và K5/K6a vẫn mở; không ghép thành fullmatch.
Bước kế duy nhất ở [PLAN](../PLAN.md#current-order).


<a id="integrated-survivor"></a>
### Normal survivor/movement với current-trap và deadline — CP137

**kr_runtime/confirmed hẹp**, [raw/hash/recipe và các run partial](../evidence/integrated-survivor/README.md).
Reuse CP106 normalprofile/resultdrain/controller/probes, chỉ thêm death_lifecycle/current_trap và
loading_timeout30s; productioncode giữCP135. R1 cùngrun cóactualA/B99/barrier26,612/25,803s trước
hạn, hai nativeclock,4pha peer movement,14actual118/159/158 mỗi view/195flags/boxdelta/seedreset,
BombFire/statreset/selectedsame-oldTCPguards. Hai actualB19 chỉ được nhận saucurrentB17, report
metadata pin exacttrapSHA/currentgeneration. B19/A21 sauA17/localclear56→26 dẫn currentdrained
survivorresult,175/180 vànativePrepare/110/reset hai vòng. Trước180 hai view cùngselectedA3/B1/XY;
không globaltick/toànworldauthority. Đây là tổ hợp policyprofile mới đã đo, không sum ba run cũ.

**Giới hạn UI:** hai ảnh cuốiR1 đen; native180/110 không đủ gọi finalroom usable. R2/R3 thử observer
CP110 nhưng bị chặn trướcreturn2 bởi boundedinput (Y374<389 rồi cachedUp sample); raw/exit1 giữ.
Y389 đã đạt ởR3, không suycapacity/inputtiming law hoặc wait/focus causalfix. Cảhai partial không
thaythế R1nativeintegration hoặc bổ sung finalUIproof. Originalauth/authority/generalreset/Windows/
fullmatch/K5/K6a/LANInternet vẫn mở. Hướng độc lập kế theoPLAN: servicehardcodedpeerIP127.0.0.1,
153advertisedIP/sourceendpoint cần đo trước mạng nhiều máy; loopbackalias không làLAN/NAT/relay.


<a id="virtual-basic-cycles"></a>
### Hai basic bomb/result/reset cycles trên virtualhosts — CP140

**kr_runtime/confirmed hẹp; fuse/rank/addressshims là emu_choice/design**, [raw/hash/recipe/audit](../evidence/peer-endpoints/README.md#cp140).
Run basic cùnghai actualCA+wineserver/netnsriêng/directvethUDP/TCPforwardstub, giữcurrentprofile/
current-trap/deadline/resultdrain. Reuse CP061oneA Space/stay method, khôngportRoller/layoutVN.
Hai actualA118 tại3,3/power1/currentkeys12345→12346/IDs1→2, native15939/15813/processcurrentID ở
cảhai view,195flags/seedreset/clockstatcapacity-power-logicalspeedkhớp hai epochs. Hai actualA17→19
đượcguardbindexactfreshtrapSHA, peer170/B21 sauactualdeath dẫnchosensurvivorrank2,1, native175302/
1803/errors0 vàroom110hai lần. ReadyB/StartAgiữa2cycles, barrier15,300/15,332s trước30s, noEOF.

Đây là **hai basic cycles cùngrun**, khôngphảisharedcascade/all-dead/fourphasemovement/itempickup
hoặcwholematchvalidator. Trướcđóvirtualsharedrun chỉ1survivorreturn/Start2rồialignmiss; bridge4,4
khác3,4vàkhôngA19. Bảnnàygiữpartial, khônggáncausaldoIP/guardhoặctiming. Khôngépdesiredoutcome.

ActualwindowPID/prefixkhớpCAowner; sausecondnative180/110cảhai đãhiệnphòngAStart/BReady ởnativegate,
unfocused4/12vàfocused4. Khôngthử95Ready saureturn2, khôngchứngminhfocuscầnthiết/cócausalfix,
khôngxoáCP137complex-routeUIdebt. Vethtraffic526records actual99IPporthai epochsdirect, nhưngnative
movementhandler4phasestừCP139làrunkhác; khôngghépchúngthànhfullmatch/K6a. AUD043qua140/kế144.
Serviceproductionbind/153IPvẫnhard127; private-addressshims hiện làmeasurementdesign. RealLAN/K6b/
Internet/K6c/NAT/relay/fullauth/authority/generalreset/gates mở; field99IP1/subtype39 vẫnunknown.


<a id="current-trap-udp-counter"></a>
### Counter UDP cùng vòng và protected trap — CP158

**`kr_runtime/confirmed` cho các control; traffic phát lại là `design`.**
[Run/recipe/hash](../evidence/peer-endpoints/README.md#cp158). Source-first: CP157 natural transient,
[CP104/105](udp-peer.md#ending-udp-state-apply), [counter đã phục hồi](udp-peer.md#udp-generation) và cached E66680-prefix/
B4E6F0. Tái dùng native6observer/AF_PACKET/capture/offload; chỉ thêm hai read-only BP tại E66868/E66884,
không đọc thân hàm mới, không chuyển opcode/layout VN. Counter không phải timestamp hoặc identity auth.

Hai quan sát không can thiệp traffic trước trap: first56 window B/R1 có27ingress/16apply,
14uniqueAPIjoins/2thiếuAPI trước arm; preroll159 có54ingress/40apply,39joins/1thiếuAPI. Cap96API/64entry
chưa hết. Preroll thấy18apply0 **trước** trap và22apply56 sau trap; cảhai lượt B đi56→1, không tái hiện
CP157. Kết quả âm tính không bác raw56→0→56→1 của CP157 hoặc chứng minh observer đã sửa lỗi.

Control replay nguyên frame neutral A→B trong cùng key12345/byte56/slot0, source IP/port từ actual99,
MAC/IP/UDP/payload/CRC giữ nguyên. R3 thấy exact native ingress sau trap, không thấy apply; chưa đủ
định nhánh. R4 thêm hai BP xác nhận **incoming152/stored156**, key56=expected56, đi **E66884** bỏ và
không apply. R5 chọn frame cũ khác từ capture dựa trên receiver counter quan sát (không sửa counter):
**incoming140/stored157**, lùi17, không đi nhánh bỏ; actual13byte/fullcursor/keypass tới B4E6F0,
protected+28 **56→0**. Stack0: B33286←B51CF4←F73882←B4EAA6←E668B2←125DF33.
Packet56 mới kế tiếp thực sự apply0→56, sau đó actualdeath19 đưa1; chuỗi B56→0→56→1, đoạn0
**61,280385ms**. Native175 entry hai view vẫn loser1/other0. 24 native getter-model kiểm snapshot
trước/sau replay và fresh56; model không gọi hàm game trong Wine.

Điều này corroborate đúng luật byte tại stage+63EC+slot đã có: incoming>stored hoặc stored−incoming≥10
được store/apply; lùi0..9 bị bỏ. Không gọi đó là universal freshness/antireplay. Keyguard qua vòng
CP145/155 vẫn nguồn riêng: oldkey59≠current56 khác với frame hiện tại key56 nhưng counter cũ.
Không tự vá client hoặc thêm validator server từ control này; native peer state vẫn có thể bị input
cùng key chỉnh trong scope đã đo, không phải bằng chứng authority server gốc.

**Giới hạn causal:** chỉ control R5 chứng minh frame lựa chọn gây ghi0 trong run đó. Chưa nối được
exact packet/stack cho lần0 tự nhiên CP157, nên nguyên nhân CP157 vẫn `candidate/unknown`; không suy
resurrection, natural packet reorder, queue-crossing-trap hoặc bug đã được sửa. PCAP bindkv-a chỉ bắt
original Ethernet A→B; replay qua kv-b được nối bằng senderbufferSHA và exact BAPI wire/sourceport,
**không** có independent second Ethernet PCAP. Initial verifier assumption hai PCAP copies bị sửa
rõ, không đổi raw cho check pass. R4/R5 mỗi lượt một normalcycle; R1–3 mỗi lượt2normalcycles+thirdUIforfeit,
không ghép thành một trận. Profile/production/nativebinary/30s/2s165/loadinggate giữ nguyên/97inputs mỗi run.

CP158 gom5lượt cùng câu hỏi, không checkpoint/PR cho từng thử. CP157/PR228 mergef44fd66; reviewwaived,
AUD047 qua156/kế160; không nâng K6a/fullmatch/LAN/Internet/8clients. Nhánh này đã có giới hạn đo được;
[PLAN](../PLAN.md#current-order) ưu tiên baseline bằng service production, không delay165/replay chủ động.
Natural transient giữ trong [RESEARCH](../RESEARCH.md#gap-contact-state), chỉ mở lại khi trigger/acceptance cần.


<a id="current-basic-cycles"></a>
### Copybasicprofile currentmode/Bhost: hai nativecycles/reset/Ready — CP170

**`kr_runtime/confirmed` hẹp cho exactcopyprofile; fuse/claims/142/rank vẫn `design`.**
[Run/recipe/hash](../evidence/peer-endpoints/README.md#cp170). Sourcefirst CP069oldbasic2cycles/CP168
currentBhostReadyAStartB/CP169exact582/catalog/route/165lookup; chỉ ghép phần thiếu, không portVNopcode/
layout hoặcRElạiItemAttr/consumer. CùngsourceWine e9bb4766…/static660447…/modOa08950…/base400000;
no sourcebinary/pack/DBchange, no service/profile change, 46/47aliases nhưCP169, defaultpreset giữfloor.

Một run388.409514005s/exit0/1316raw/109frozen/privatehosts/cleanupseal: actualMannercreate→idleAhostBack/
Bpromotion/Arejoin/ReadyAStartB; hai actual121/99/barriers6,234301688s/6,362700964s trước30s.
Epoch1actual12417→28→27, epoch217→27→28; mỗi epoch2FixedBox27/28(vt1747724)→nativeCGOItem
(vt1746A4C)→actual142bothroles→hidden124/165full12/errors0/lookupđúngrevealedpointer. 3callback165/view/
epoch, chỉA Roller5→6/capacity1→2/Fire1→2, B2/1/4 giữ; **baselineA1/1/5/B2/1/4 táilậpởepoch2**.
Hiddeneligibility reportmap tạo mới mỗi key12345→12346, báo142currenttrướchiddenclaim; khôngsuy fresh/
authentic toàn142 vìschema chưacókey. NormalASpace trước/sau3pickup tạo4actual118/IDs1..4/power1→2
mỗi epoch, sameACK159pointer→processed158ID/full39/full13 vànative195before/after/seedreset khớp
boundedterrainmodel. Geometry/effectrules của model giữ `emu_choice`, không originalworldoracle.

Cảhai epoch Aactual17→19/B21→chosenrank; readonlynativewatch/capturedsecurefields chứngminh56→1
vàother0 trướcnative175 ởcảhai view (khôngglobaltick). Hai native175302/1803/errors0/roomrefresh;
actualReadyA/StartB giữaepoch vàactualReadyA/native110both saureturn2. Actual180body[0,1,0]4frames/
12copiednativecases/boundedmodegetterreads1; khôngcósetter2trace mới hoặcclaimwholemodeexecution.
Same8transports/noearlyEOF, chưaintentionaldisconnect/oldUDPprobe/two-nativecontenders/malformedtest.

1614checks gồmrawhash/typedjoins/reset,174capturednativepacket/516securefieldmodels riêngruntime;
life23joins/16nativegetters và7alignmentgroups/14viewphaseack/exactrestore/4rare165lookupretained.
ParserCP069cũ gáncollision snapshot vào lastGEN165_END gâyjointrùng; initialscript/log/report giữ,
finaldùngparserCP168đãcóevent/phase/strictSTAT_SNAPSHOT, không xóa raw hoặcđổicriterionchopass.
Final2frame đãxem córoom/AReady/Bchief, **BStarttrắng1frame**; khôngpersistenthang hoặcStart3proof.
DefaultfloorCP168Start3 khôngbackfillbasiccopyprofile.

Đã giải currentcopybasicprofile2cycles/reset trongscope này, chưafullmatch/K5/K6a/dependencies/LAN/
Internet/8client/Windows/fullauth/generalreset/originalauthority. Next[PLAN](../PLAN.md#current-order)
pinexactprofile thànhoptionalpreset/export rồinativeBStart3 trênactualexportedfile; giữdefaultunchanged.
AuditAUD050qua168/nextCP172. Khônggộp CP069oldprofile hoặcCP168floor thành nghiệmthu mới.


<a id="a-c-basic-cycles"></a>
### Ahost/Cguest: hai normal basic cycles, peer hai chiều và reset — CP176

[Recipe/raw/hash](../evidence/peer-endpoints/README.md#cp176). `kr_runtime/confirmed` hẹp trên copy Wine,
private virtual hosts: source-first CP175 prestart BguestEOF/Ahost→C3/slot1, C bom đầu và CP173/174
A hidden-route/peer/bombs/effects/reset. Chỉ chuyển vai đo Bslot1 sang Cslot1, Ready guest A sang C,
Start host B sang A; không new opcode/body, không phân tích lại native consumer hoặc import wire VN.
Code/profile default không đổi; production source đúng CP175, C2/7 declared, native initial2 khớp.

Run `A-C-cycles-r1`466.655250s/1079raw/163frozen/exit0/cleanup0PID. Prestart B đã rời, không ABgame
hoặc forfeit. A/C roster ID1-slot0/ID3-slot1, hai keys12345/12346. Mỗi key actual C118 rồi hai A118,
owner1/0/0, IDs1–3 rồi4–6, power1/1/2; native159/158/entity/195 flags/currentID và reset cùng hai view.
Actual142 từ A/C tới hai ô hidden27/28, ba actual A124 (17/27/28), native165/full reader/effects chỉ
ở A; native Cslot1 stats2/1/4 giữ, A baseline và hidden eligibility dựng lại qua Start2.

Bốn cửa sổ current ARight/CRight qua hai key: actual ingress/key/target/arguments/current-coordinate
progress trên recipient. Không dựa encoder tự sinh hoặc input return0. Native A local trap56→dead1,
C nhận170 và report21; cả hai protected actor snapshots trước175 có A1/C0. Hai normal175/180/native
full readers/errors0, room refresh, CReady95/AStart2 và CReady95 sau return2/native110 cả hai.
312 quan hệ basic không tính raw hash giữ như verifier cũ; tổng1391=1079raw+312. 228packet native
model cases/810 secure fields, peer708 checks/456 getters, life23/16. Models là loại kiểm riêng,
không gọi Unicorn getter là gamecall runtime hoặc original server authority.

Raw ảnh return2-native-gate đen cả A/C; sau normalfocus/actualReady, final ảnh cả hai vẽ phòng,
redA/yellowC. AStart control trắng dưới hover chưa phải proof hỏng hoặc thành công Start3; không
input Start3. CpreReady black ở CP175 và startup partial R3 không được giải thích ngược từ run này.
Hai kết quả/reset và peer có native proof; hiện tượng render trước focus vẫn là giới hạn quan sát.

Analyzer được chọn lại A/C thay AB (B đã rời/chưa có gameplay), giữ log StopIteration/missing
chronology-B ban đầu; không sửa raw hay nới core criteria. Guest/host control/prefix/namespace/PID
và current C owner trước input đã phân biệt; C giữ hostảo B nhưng không identity B/account resume.
Cstartup120s chỉ đến firstclock; observation hai vòng có total600s, loading30/fuse6/20pulse/.045/
tolerance12/neutral3/restore3 giữ nguyên, không kéo deadline gameplay để làm pass.

**Giới hạn:** cùng một physical host/private namespaces, modified client/local identities; fuse,
drop/claim/result vẫn policy. Không contest/oldUDP injection/disconnect trong hai ACepochs, không
capacitymaximum/hai reservation/generalreset/thirdStart, không originalWindows/fullauth/physicalLAN/
Internet/8clients/fullmatch hoặc tự đóng K0–K7/K5/K6a. [Authority](rust-boundary.md#authority) và [acceptance](../PLAN.md#basic-acceptance-review)
giữ dependency/control còn thiếu; [PLAN](../PLAN.md#current-order) chọn profile/capacity theo vacancy tiếp.


<a id="rust-life-results"></a>
## Rust report-derived life, kết quả và reset — CP240 / M3 partial

[Code](../../rust/crates/kr-service/src/life.rs), [owner/queues](../../rust/crates/kr-service/src/room_io.rs),
[metadata](../evidence/rust-service/life-results.json), [recipe/raw](../evidence/rust-service/README.md#cp240).
Tái dùng CP023/028 envelopes và translation, CP102/103 cursor/drained21, CP188/189 life/rank/reset;
không RE opcode/helper hoặc nhập wire VN. Python vẫn reference/rollback, backend M5 độc lập.

`design`: optional Battle profile chỉ bật với current-trap/death và post_blast_untrapped_drained, bốn fixture đã pin.
127 prefix14, trap17 raw14 và death19 raw11/envelope đã đo; biến thể khác fail closed trước cursor.
Cursor riêng connection: serial arithmetic u32, duplicate body cùng tick, tối đa64 body/tick và4096 report/vòng.
Start xóa trap/death/forward marker, **giữ cursor**; body cũ đã thấy không thành report mới. First-seen/forged-forward
vẫn thiếu authentication/wire epoch; không dùng tick của peer khác hoặc cached124 làm đồng hồ chung.
19 phải có17 hiện hành, không được chết lại;17 mới không xóa death history. 17/19 chuyển170 sang peer, không selfecho.
21 mới hơn current trap và latest successful full-roster158 (nếu đã có blast) có thể clear peer một lần/trap;
marker chỉ complete sau TCP write;
21 không mở lại quyền nhặt. Trap/death khóa124 theo report. Unknown subtype chỉ bounded observation, không world effect.

Result chọn sau death8s, survivor21 mới hơn death/trap/latest successful158 và hết pending bomb jobs.
All-dead dùng policy rank2 riêng, đợi latest death8s; không suy tie enum/luật server gốc. Reserve đủ sáu queue entries
cho hai recipient trước publish Ending:175 ngay,180 sau5s, hai110 refresh sau3s nữa. Complete refresh đủ roster mới Idle;
Ready/Start/late callbacks có membership/round fences. Full TCP write không là native execution ACK/atomic delivery.

Native post-result202/22 chỉ current Ending/closed member, prefix0/1 và budget; không active Battle-exit implementation.
Native180 gửi lại585/zero: bounded no-response khi kết quả đã chọn/closed, giữ110 refresh, không cấp visit/setup mới.
167 body16 đã thấy ở run lỗi R1; chỉ thêm đúng envelope này vào4/32 bounded opaque policy, **semantics vẫn unknown**.
Polling result dùng100ms; idle read deadline tuyệt đối giữ từ lần đọc bytes, không bị poll reset.

`kr_runtime/confirmed` trên KR00967/e9/Wine/virtualLAN: vòng1 actualA118→17→19, B21; vòng2 B118→17→19,A21.
Hai view state loser1/other0 trước175, native175 đủ outer302/inner292/errors0, ranks2/1 đảo đúng; hai180 đủ3/errors0,
refresh/actualReady/Start tới clock3 và giữ6s. Ảnh return2 A có vùng nút Ready trắng;
later actual95/Start3 đã đo, không gọi đã giải nguyên nhân render. Ba clock seed195 khớp, stat baseline theo vai; không consumed-item-reset proof.
TCP745 records/108 exact replies và tất cả connection/C2S-sequence joins; 66 raw/201.905s sealed/cleanup0.
Native-tested binary `a4798db20fb0797d523226a70b0b51ee9b4c264db3f1aae1d6448e5dcd0cf229`. Copied52A350 getter checks riêng, không Wine gamecall/oracle toàn binary.
PeerFinish cursor sau return có giới hạn cũ; không lấy nó làm full170 parse proof. Native pre175 state và175 parser là evidence riêng.

40 Rust tests/44 kernelTCP: malformed blob, slot/death prerequisites, duplicate/stale qua reset/current positive,
closed202/585 không đổi kết quả, hai winner và all-dead, once peer21, pickup vẫn blocked, EOF/generation/absolute idle20s.
Kernel/all-dead/peer21 positives không tự trở thành native control. R1 thiếu167/16, R2 thiếu202, R3 thiếu post180585;
46+46+48+50 raw failures giữ nguyên, mỗi run seal/cleanup trước sửa. R4 chỉ có một ending/return: click ngắn không95;
R5 giữ nguyên binary, dùng input CP188 focus12s/off-hover/heldmouse1s; budget240s khai trước run cho bốn thao tác return,
service/loading/write/shutdown deadline không đổi. Verifier đầu so boolean flags với kinds0/1/2 sai, sửa đúng projection
`int(kind!=0)` đã có CP239 rồi kiểm đủ195×3×2 từ cùng raw; log fail giữ, không đổi oracle hoặc runtime. Không thay source/config giữa run hoặc nới deadline để pass.

M3 chưa đủ acceptance: native consumed-item reset/contest; exact oldUDP/current positive trên Rust; dead-owner/queue cases;
hostEOF với bom chờ phải đưa survivor về phòng (107/110 giữa scene chưa đủ). Chưa kim/cứu/fullauth/fullservice,
không nativeall-dead/peer21clear/trận3 outcome/chain variants/generalphysics. [PLAN](../PLAN.md#current-order) giữ bước tiếp.
K0–K7 scope cũ giữ nguyên/K6c WAN giả lập. Internet thật chưa kiểm. AUD069 qua240/audit kế244.

<a id="rust-return-cleanup"></a>
## Rust active host EOF với bom chờ: forfeit và về phòng — CP241 / M3 partial

[Owner/queue](../../rust/crates/kr-service/src/room_io.rs), [model](../../rust/crates/kr-service/src/room.rs),
[metadata](../evidence/rust-service/return-cleanup.json), [recipe/raw](../evidence/rust-service/README.md#cp241).
Tái dùng CP188short-r2/CP189 activeEOF/nativeforfeit-180-107-110, Python `close_connection/tick/deliver_departure`;
CP240 Ending/currentmembership/recipientqueues/native175302-1803. Không RE nativefunction/wirelayout mới.

`design`: activeArmed channel EOF khi Battleprofile bật → leave/fence bom/terrain/claims ngay, giữ survivor Ending.
Không dựng report19 hoặc coi disconnect là native chết. Resolve rank theo slot departed, reserve đủ3queues/survivor
trước publish:12s forfeit175,17s180 (selector phòng),20s107 xóa departed ID rồi110 survivor/chief. Fullroster refresh
xong mới Idle. Recipient/round guards chặn job bom cũ; partial TCPwrite không rollback/native ACK.
GuestEOF giữ host, hostEOF promote survivor. Native control mới chỉ hostB→A; kernel riêng guestA→B.
LoadingEOF và EOF trong Ending đã chọn vẫn đường cũ/có giới hạn, không nhận generalrecovery/matchrestore.
118 từ accepted-dead member bị reject trước ID/serial/reservation; report-derived death không authoritativeworld.

`kr_runtime/confirmed`: hai actualKR00967/4b781f/Wine/privateLAN/cùngrootRust như CP240. B đã là host sau idleA Back/
rejoin/ReadyA/StartB; actualB118→native159 full39 ởhai view. Fault chỉ CA-B PID có namespace/prefix/startticks/
hostexe/cmd/image mapping/SHA khớp trước SIGKILL, trước6s fuse; service/A không restart. Không fake TCPdeath/127.
A đọc forfeit175 outer302/inner292/errors0/rank1-2,1803/errors0; sau180 native107 xóa ID2 về0,110/chief1 và
UIA một slot còn lại/chủ phòng. Getterchief sau return không đo vì budget20 đã hết trong initialroom/rejoin;
không nhận getter1,110full/UIchief là nguồn riêng. Hold4s saucleanup, không158 qua originaldue; B đã dừng không nhậnforfeit.
TCP361/60 exact replies/alltuple-C2Ssequence joins; 48 raw/89.914s frozen/sealed/cleanup0.
Binary `bff20c8890f5415847b7f3a872e1d9cd292f3150f64599c55889e6e00a0beb40`; copiednativegetter riêng từ rawwrappers, không Winegamecall/wholebinaryoracle.

41Rusttests/50kernelTCP: dead-owner118noID/serial; matureddeath waits pendingbom;158 invalidateold21/fresh21positive;
activeguestEOF fences survivor-owned pendingbomb, delayedforfeit/180/107110/idle30s. Không lấy syntheticdeath/EOF làm
nativehostproof; native không bao gồm guestEOF/đồng thờifault/rejoin mới. R1 startupfail25raw đã seal: Bchannel6→17 rồi
EOF trước67/sảnh/phòng/fault; nguyên nhân chưa rõ. R2 tới A175302/292/rank1-2 rồi SIGSEGV/kernelbase trước180,46raw đã seal. StackD2F64A←E5FC07
khớp lỗi importABI CP187 đã giải: copye9 bindCompareStringA6args sai thay vì thunk2args. R3 dùng toolcopy CP187
`patch_wine_compare.py` để tạo tên mới SHA4b781f…, giữ e9 nguyên; `.image` không đổi, chỉ14byte importrepair.
Không RE lại ABI hoặc gọi prior e9pass là ABI đúng. R3 control mới đo targetpath, không sửa lịch sử CP235–240.

M3 chưa đóng: native consumed-item reset/contest; exactoldUDP ở vòng mới + currentpositive; outcome/nonreader/queue
control còn thiếu theo [PLAN](../PLAN.md#current-order). Optionalkim/cứu/capacity4-8/accountbridge/fullservice ngoài ca này.
K0–K7 scope cũ/K6c WAN giả lập giữ nguyên. Internet thật chưa kiểm. AUD069 qua240/audit kếCP244.

<!-- protocol-topic-content-end -->
