# Terrain, block và cascade

[Mục lục](../PROTOCOL.md) · [Câu hỏi đang mở](../RESEARCH.md) · [Thứ tự việc](../PLAN.md#dashboard).

<a id="current-map-chain-state"></a>
### ExVillage01 static grid và native box-stop/clear state — CP090

[Recipe/source/hash và negativecontrol](../evidence/current-map-chain/README.md).
**kr_static/confirmed data**, **kr_runtime/confirmed hẹp** trên currentmodifiedWine/profile. Reuse KR
header80 (VN76), packindex key từdwordđầu/entryCRC, ObjectTable/TileInfo72 từnguồn cũ; không port VN
ID/offset/opcode hoặc profile collision Village00 thành mọi map. Extract đúng FxMap/Map/ExVillage01,
fx/ObjectTable vàTileInfo tại local; sourcepack/binary/DB không sửa, chưa mở rộng serverpolicyCP089.

Map payload SHA `aeb57ffdb27a1405bd2ae3d29f229174ae5d584fc84eaf4e87884fe8518f86a4`,1814bytes,
version1000/15columns×13rows,8spawn từoffset80, grid148..1708/exclusive,195cell×2i32. Targetmap dùng
11IDtile/object; numeric index của ObjectTable đúngKR. Raw TileInfo Data72 wordđầu: Ground0,
House1, Box2 trong subset này, lấy từ actualdata và classreference, không từ tên/icon.

| Nhóm dữ liệu targetmap | IDs KR | Mức/giới hạn |
|---|---|---|
| Nền |1938/1941/1942/1943/1947|staticDataword0; không tự là mọi flame-direction rule|
| House |1955/1956/1957|staticDataword1; native instance/classflag/direction chưa quan sát|
| Box |1932/1933/1934|staticDataword2;9cell native stop dưới đây, không mọiclass2/allmaps|

Read4smallhelpers byte-match static/runtimecopies/modO: E9F9B0 query có bounds/mapcontainer/type3/10/
active filter vàdynamiccast; E90A80 đọc `[object+AC]→+2C`; EA1230 tra map+1D24, EA0B10 tra map+1D54.
StaticDataword2 **không** đồng nhất với E90A80flag2 hoặc container membership. No newfunctioncall/
stateforcing; debugger chỉ đọc nativecalls đã chạy, không dùng guessed rawfield layout.

Runtime `tiles-gated`: rootA(3,3)/power2,B(3,5)/power1,lateA(5,3)/power2 vàtwo replacements(9,3)/(9,5).
Mỗi view có29 `EB9710` cellreturns:20 `(state1,return1)` trên actualstaticobj−1/EA0B10=0;
9 `(state2,return0)` nối preceding actual `EA0B10=1` callerEB975C và staticBox1932/3/4 tại
(3,2),(2,3),(2,5),(3,6),(5,2),(9,2),(10,3),(9,6),(10,5). Callbackindex1 hoặc4 giữ join riêng.
Ở các ca có preceding ray-step, **E9F220 accepted1 vẫn dẫn tới box-stop**: nhánh điều phối sau đó
mới gọi EB9710; không được lấy step1 làm toànraypass hoặc suy tia xuyênbox. Cellbox là điểm dừng,
không chứng minh flamevisual/damage/fullclass behavior từ return0 này.

Cả80E9F9B0queries/view đều **null**, EA1230 đều0, khônghit các explicitstopbranches E9F220 trongrun.
Đây là absence có boundary, không bác nhà/staticblocker; boxes nằm ở đường/set khác. Không có
nonnull objectcontext để nối TileInfo72→runtime template/flag. Tại CP090 house/directional flags/other sets và mutable terrain sauboxbreak chưa đo;
CP091 đã đo house/settransition, CP092 đo finiteflags/reset. Context/direction template khác vẫnunknown.

5actual118/159nonzero/full39 và5full15813/errors0, rootprocess1,2,3/newroot4,5/ownlaternoduplicate,
121clock175full302/180full3/roomrefresh một lần. 156targetrelations và secure/getter emulation kiểmriêng;
không two-cycle acceptance mới, CP089two-cyclepositive giữ scope cũ. Firstungated358,644s cóclock/key
nhưng no124/118/158; ảnhblack/faint, không đo được stop. Newprobe group chỉ bật158entry→END cho
repeat200,292s; overhead chỉcandidatecause, không coi firstnegative làruntimecollisionproof.

Chưa releaseexpansion/fullmap/recursive/moving/wholepool/validator/auth/codec/authority/allreset/
vanilla/LANInternet/fullmatch/K5/K6a. [PLAN](../PLAN.md#current-order) giữ việc tiếp; AUD026/nextCP092.


<a id="house-after-box"></a>
### Native house stop và mutable box flag — CP091

[Recipe/hash/oldsource reuse](../evidence/house-after-box/README.md), **kr_runtime/confirmed hẹp** trên
currentmodifiedWine/A-B/ExVillage01. Production CP089 không đổi; dùng lại CP090 map/data/helper/probe,
không thêm nativebody/patch/callfunction hoặc ép map. Thử normalUp saufirstroot, Space, Downescape.
Actual root mới là **(2,3)/serial56/ID4**, vượt target3,3 do input/feedback; giữactual118 làm nguồn,
không sửa nhãn cho khớp dự định. ID4/159 nonzero cảhai view trên ô từng cóBox1933/rawclass2.

Trong firstroot158/index1, ô(2,3) có actualEA0B10=1 callerEB975C/EB9710state2/return0; **sau xử lý
ngaytrong cùngcallback** EA0B10=0 callerEB9BBB. Firstnative142 mỗi view báo(2,3) trước newplacement,
service log `unknown_hidden_cell` vì ngoàihiddencatalog; đó không phải nativebox khôngbịphá vàkhông
được dùng report này làm worldstateoracle. CP090raw đã có transition1→0 này, CP091 chỉ nêu rõ nguồn cũ.
Không có EB9710state1 của **ownrootcell2,3** trong callback4; không bịa marker đó để gọi mutablepass.

Newroot158/index4 cóstepE9F220 accepted1 tới(1,3), actualEA0B10=1 callerEB975C→EB9710state2/return0
cảhai view. Staticcell làHouse1956/rawclass1: **cùng stop path bao gồm observedbox vàobservedhouse**,
không suy container1D54 chỉclass2 hoặc classword==E90A80flag. Cả61E9F9B0queries/view vẫnnull,
EA1230zero/noexplicitE9F220stopbranches; không có context/template/directionflag instancejoin.
Đường house vừa đo đã được giải qua nativeflag, nhánh template khác chưa đo nhưng không chặn bước này.

32targetrelations;4actual118/159/full39/errors0,rootprocess1,2,3/newroot4/laternoduplicate,
4full15813/121clock175302/1803/roomrefresh một lần, khôngcontroller/service/probeerrors. Model/parser
và securegetter tách với runtime; không mới two-cycle/fullmatchgate. Verifier đầu đoánstate1owncell
bịStopIteration; correctedactualsamecallbackset0/caller/lineorder giữraw/strictactualhouse/new159.

Tại CP091 bước tiếp là storage/snapshot (CP092 đã đo bên dưới); [PLAN](../PLAN.md#current-order) giữ việc hiện hành để phục vụ
terrain/release ngoài whitelist. Chưa mapwide/otherclasses/recursive/moving/reportfreshness/auth/
fullvalidator/fullcodec/authority/allreset/vanilla/LANInternet/fullmatch/K5/K6a; AUD026/nextCP092.


<a id="map-flags-snapshot"></a>
### Native map+1D54 flags và hai vòng reset — CP092

[Recipe/correction/hash/audit](../evidence/map-flags/README.md). **kr_static/confirmed storage hẹp**,
**kr_runtime/confirmed finite snapshot** trên currentmodifiedWine/ExVillage01/A-B. Production CP089
không đổi. Reuse map195cells/TileInfo/classdata CP090 và CP091 house/settransition; protectedgetter/
checksum đã có, không RE lại cipher/codec. ED1350 trả `*(container)+12*index` (không boundscheck trong
helper này). EA0B10 dùng map+1D54→ED1350→57A870→52A350; mỗi slot12byte là protectedwrapper.

Clockreceiver E13490 **là container khác CPlayMap**; E13290 đọc field+1DF06C để lấy actualmap.
First44snapshots trên wrongreceiver đềuchecksumfail/partial, originalrun vẫn hai nativecycles;
không gọi đây là flag/resetfailure hoặc lấy fallback guessedpointer. Newrun dùngsource-backedfield,
actual15×13/checksum/value0or1 rồi chụp195slots. Nativequery receiver/index và observedwrapperbind
khớpmap/array+12*index; clockmap không suy từ process hoặc tên asset. Không gọi hàm game/mutate state.

Final `fixed`: **44snapshots** = mỗi view22 (clock vàbefore/after5native158 mỗi vòng),195flags/mốc.
149seedones =126staticBox +23House,46obj−1clear trên **đúng map này**; toàn195values giống staticobj
presence tại cảhai clock. Sau root1/6 giảm149→144 (5Box), sau newroot4/9 giảm144→140 (4Box), later
ownsecondary callbacks không đổi thêm. Vòng1 thirdA(5,3) nên box(5,2) đổi; vòng2 thirdA(4,3) nênbox
(4,2) đổi, giữactual118geometry; các boxdeltakhác khớpcảhai view. House flags giữ1, clear không đổi.
Startvòng2 khôi phục **toàn195seedvalues**, không chỉ vài ô; không là mọiresetvariant/map/gamemode.

Snapshotraw pinpointer/wrapper/cipherdata/phase/begin-end_ns/sourceSHA; lấybuffer finite hiện hành,
không biết containercapacity/allocationlength chỉ từ ED1350. Các query chạy trong callback được kiểm
thuộc tập giá trị snapshotbefore/after, **không** tạo timestamp giả như đồngthờiquerry. Actual75queries/
55wrapperbinds mỗi view: bindcoveragepartial vì GDBfeedbackignore dùngmutable `$bpnum` nay làprobe
thêm cuối. Giữ marker thiếu, chỉ join observedbind→nextquery; không đổi raw để ép1:1. Địa chỉ toàn195
được kiểm bằng copiednativeED1350 model riêng, không giả mọi nativebind đều đãquan sát.

800targetrelations; copied52A350 **8668getterchecks** vàED1350 **8580addresschecks** (44×197/195)
là emulation trên runtimebytes, counter/rekeymodelreset không làm thayclientWine. 1152actorsecure
checks riêng. Hai nativecycles10actual118/159nonzero/full39,10full15813/errors0,121clock175302/1803/
roomrefresh hai lần, khôngcontroller/service/probeerrors; no wholematch/K5/K6a proof. Partial first
vàmetadata/marker verifierfailures giữ đầyđủ ởevidence, không sửa checkpoint trước.

**Tại CP092 chưa có implementation terrain mới**; [CP094](#terrain-policy) đã tích hợp optionalprofile có giới hạn. Mapflag là nativequeryinput thực tế trong cađo, chưa servergốc
worldauthority hoặc luật mọi class/template/direction. Servertracking phải explicitdesign/profilepin,
model so actualdeltas/chainorder trước tích hợp, không nhận142 nhưepoch/auth/worldoracle.
[PLAN](../PLAN.md#current-order) giữ thứ tự, AUD027qua092/nextCP096. Genericmap/moving/recursive/fullpool/
cell-distance-auth/fullvalidator/fullcodec/authority/allreset/vanilla/LANInternet/fullmatch/K5/K6a mở.


<a id="terrain-shadow"></a>
### Offline terrain candidate so native oracles đã lưu — CP093

[Source/reuse/hash và phép so](../evidence/terrain-shadow/README.md). **design/candidate**, kiểm bằng
**emulation/offline model**, không native runtime mới/productionintegration/recoveredserverlaw.
[tools/terrain_shadow.py](../../tools/terrain_shadow.py) độc lập vớiservice/legacy, không socket hoặc report.
Reuse CP090targetmap/TileInfo IDs, CP090–092nativecallbacks/queryendpoint và44snapshotsCP092 làm
**expectedoutputs độc lập**; không dùng model tự sinhexpected đểxácnhậnbehavior.

Model khai profilewidth/height/grid kinds0floor/1solid/2box/mapSHA256/power≤2. Externalcomparison
kiểmactualmapbytes SHA, rawgrid/classes/sourcehashpins; class khôngkhai báo fail rõ. mapSHA trong
class là metadata, không tự đọcfile/chứngthựcmap. Place nhận Bomb(id,owner,row,column,power), rejects
unknown/outside/overlap/solid/currentseenID trước mutation; chỉowner0/1/current simplegeometry.
Không model auth/capacity/distance hoặcmọi118variants; adapteractualfieldcheck boundedknownvalues.

Khi root hiện hànhblast, bốn tia tới range/power, immutableterrain **trong cả cascade**, gặp soliddừng,
boxdừng+ghiđểclear sau; firstoccupiedbomb dừng tia+queuechain. FIFO/directionorder/firstbombstop và
terrainimmutable là **model choices**, chưa universalnativeorderlaw/sharedblocker/chainvariants.
ProcessedIDs loại pending; latercallback ID đãprocess no-op, Start resetseed/pending/seenIDs. Không
suy mọi placement/chain đều đúng từcase đãmatch, không mở serverterrain tracking ởCP này.

Phépso432relations/38nativecallbacks từ CP0905callbacks×2view, CP0914×2, CP09210×2; rootdata từ
actualaccepted118/159 geometry, expectedprocessedIDs từnativepointer/callbackjoins. Tất cảactual159
phải nonzero/full39. NativeEA0B10query trongcallback được soendpointsetbefore/after, không fake
exacttime. CP092actual195before+195afterflags/callback vàboxdelta/resetseed khớp; CP091newroot2,3
trênoldboxclear→house1,3stop cũng khớp. Cùngraw có sẵn, khôngnewWine/nativeRE cho phầnđãgiải.
4testconstraints kiểm atomicreject/unknownprofile, boxstoptrongcascade→laterblastqua, solidhold,
oldcallback không đụngnewbomb/reset; tests không lànative oracle.

**Tại CP093 chưa tích hợp service**; [CP094](#terrain-policy) đã đo optionalpinnedterrain/guards với input
ngoài CP089clearcells. Việc hiện hành chỉ ở [PLAN](../PLAN.md#current-order); modelmatcholdraw không thay runtime production.
Genericclass/map/direction/moving/overlapvariants/recursiveorder/sharedblockers/fullvalidator/auth/
codec/authority/fullreset/vanilla/LANInternet/fullmatch/K5/K6a mở. AUD027/nextCP096.


<a id="terrain-policy"></a>
### Optional pinned terrain/capacity policy và native integration — CP094

[Recipe/source/hash/partial](../evidence/terrain-policy/README.md). **Implementation: design**,
**compatibility:kr_runtime/confirmed hẹp** currentmodifiedWine/A-B/ExVillage01/power1..2. Purekernel
CP093 chuyển về [server/terrain_model.py](../../server/terrain_model.py), toolterrain_shadow giữfacade/API;
body chỉđổidocstring và432oldnativecomparisons vẫnpass. Khônglegacycodecopy hoặc newnativebody.

Optional `bomb_terrain` yêu cầu capacityprofile **chỉA/B**, excludes `capacity_chain`. Khai width15,
height13,195kinds0floor/1solid/2box, maxpower1..2; `seed_sha256` kiểmbytes(kinds), `fixtures.map/start`
SHA khớprawactualframes. `map_sha256` là sourceprovenancemetadata đãkiểmactualpack tạiprep, **không**
signature/assetattestation hayserviceđọcpack. Absentmetadata giữCP089/defaultbehavior; oldprofile không
migrate. Invalidconfig failbeforeselector; Startresetworld/seed/pendingseenIDs vàtrackedcapacity.

Currentkey/serialduplicate giữlýdocũ; known118variant `[field1,6,7,8,9,11,12]=[0,1,0,0,0,1,0]`,
ownerboundslot/currentpower1..2/insidefloor/nooverlap mớiqua. Reject `bomb_terrain_variant` hoặc
`bomb_terrain_placement` **trướcID/job/serialreservation**, rồicurrentownerquota nhưCP087. Khôngcheck
nativeactorlocation/auth/distance/tấtcảcharacter/mode. Previewworldplace trêncopy, commitpendingterrain
chỉ sau successful159 broadcast; partialsendvẫnraise, existingID/serial trước159 khôngrollback.

Rootdeadline: previewcascade/currentterrain theoacceptedgeometry trêncopy trước158, chỉcommitflags/
processedpending/capacity_release sau broadcast158thànhcông. `terrain_blast` logsource/design/generation/
processed/cleared/before-after195flags/commit_ns. Secondaryprocessed được bỏquota, **giữjob/ticket/
own158deadline**; modelpendingkhôngcònsecondary nên ownlater khôngghostchainnewbomb. Fencestalejobs
khôngchạmworldvòngmới; currentFinish/Startreset giữ. Failed158 khôngcommitterrain/release, nhưngjob/
fence đãretire/consume trướcbroadcast theoarchitecturecũ; chưa whole-service transaction/recovery.
Không142reportworldoracle: blockreport ngoàiexistinghiddencatalog vẫn logreject, worldupdate từpolicyroot.

113server tests (8ca policy mới) cùng4kernelconstraints, atomicreject/profilepins/owner/quota/currentkey/
duplicate/cell/overlap/power/oldsecondary/newround/reset/failed159and158commitboundaries; sourceclock
fixturefirstfailure giữriêng. FirstnewWine12entities nhưng controllerindex==ID bịsai saujobfence vàR1
extra118 sausecondarydeadline, **partial**. Repeatlean giữproduction/profile/input/delay/fuse6, bỏknown
highfrequencytile/rayprobes, nối actualGEN_LOOKUPwireID→callbackindex→END. Không sửaraw đểmakepass.

Finaltwo-cycle `lean`: mỗi vòng7actualnative118/159, IDs1..7/8..14, keys12345/6. Rootnormal3,3→B3,5/
A4,3; lowerpair(9,3)/(9,5) cáchinput2s. Lowerroot4/11 chain5/12 **ngoàioldclearcells**;
AthoátUp/freshfeedback vàactualextras serial112/140 được nhận budget[] rồi[6]/[13], request thứhai
trướcsecondaryown158 **1,631533938s/1,400993019s**, bothviewactualwriters/159nonzero/full39.
ExtraR1cells(3,5),(3,3),R2(3,5),(3,2) theoactual118; oldbox3,2đãclear nênplacementnhận.

14nativeentities nhưng **12full15813callbacks/view**, không14lầnnổ: root6 process6,7; root13 chỉ13
(vìactual14ở3,2 ngoài tầm2 từ3,5). Ownjobs7/14 bị `round_closed`, no158/terraincommit; ID14 chưa
nativeprocess trướcreturndisposal, khôngbịtôthànhsuccess. Serverpredictedsets vàall195before/afterflags/
boxdeltas khớp52nativephase snapshots; seed/reset cảhai vòng, native121clock175302/1803/roomrefresh2,
no controller/service/probeerrors. 136relations/10244copiedgetters/10140addresschecks/1182actorsecure;
model/helperemulation/native/runtime/sourceboundary tách riêng. Chưa allreset/wholematch/gate.

Scopedmodelchoicesimmutablecascade/firstbombstop/FIFO/directrays stillcandidate beyondmeasuredinputs.
[CP095](#opposite-box-control) bổ sung actualboxgiữa2bomb/laterfreshpaircontrol cho khoảng này;
ca đo đó không chứng minh toàn bộ modelorder. Việc hiện hành chỉ ở [PLAN](../PLAN.md#current-order). Genericmap/template/direction/moving/recursiveorder/auth/codec/
fullvalidator/authority/vanilla/LANInternet/fullmatch/K5/K6a cònmở; AUD027/nextCP096.


<a id="opposite-box-control"></a>
### Hai bom đối diện thùng thật — CP095

Nguồn [recipe/hash](../evidence/opposite-box-control/README.md), scope **kr_runtime/confirmed hẹp**:
copy Wine đã sửa/cùng profile A–B/ExVillage01/local stub. Source binary/pack/DB giữ nguyên.
Tái dùng map grid/hash và getter/consumer/callback CP090–094; bảng 121 và spawn CP024/065;
KR flame-chain cũ cùng VN lifecycle/report làm reference cơ chế. Không chuyển số opcode, offset,
ID hay layout VN sang KR. New fixture chỉ đổi slot1 spawn-index ở inner offset76 từ1 sang4;
index4 trong map là `(x11,y3)`. Native frame/blob/typed-reader model trả0, inner100/outer129;
actual clock ở cả hai view: A X120/Y159 và B X440/Y159. Model fixture và runtime tách riêng.

Run `blocked`: B đặt `(3,9)` power1 phá đường qua Box `(3,8)`; input thường đưa B tới `(3,7)`.
A đã nhận Fire, đặt root `(3,5)` power2; giữa hai bom là actual Box `(3,6)`. Actual118/159 IDs1..4:
B1 `(3,9)`, A2 `(3,5)`, B3 `(3,7)`, A4 `(3,5)`. Cả bốn native159 trả pointer khác0/full39
ở mỗi view. B3 đã có entity trước callback A2; callback A2 chỉ process2, không3, và snapshot195flags
trước/sau cho Box3,6 `1→0` ở cả hai view. B3 sau đó process riêng trong callback own158/full13.
Service dự đoán processed/195flags khớp ba callback đã gửi; đây là kiểm tương thích, không là luật server gốc.

Run đầu **partial**: `oppositeboxmovementtargetmiss:A-later-escape`, chưa callback A4 trước cleanup.
A4 đặt trước own158 của B3 chỉ0,724395057s trong khi fuse service6s; không thể dùng A4 để chứng minh
chain chính B3. Fuse6s là emu_choice. Verifier kế hoạch ban đầu fail được giữ; verifier bounded chỉ
xác nhận phần trên, không sửa raw/lỗi controller thành success. Repeat tách riêng chờ B3 owncallback,
rồi đặt A4 trước B5 mới trên hai ô cũ để kiểm đường đã thông.

Hai repeat trung gian `freshpair`/`aligned` đều partial trước cặp chính vì B không tới ô cần đặt.
Mẫu mới sau nhả phím chưa đủ chứng minh đã dừng: lần aligned đọc Y163 rồi bước Left thấy Y187/X360.
Run `stable` yêu cầu ba mẫu tọa độ mới giống nhau sau thời gian neutral250ms, pulse OS45ms để căn ô;
đây là điều kiện controller, không là contract tick/độ trễ game. B được đưa tới ô mới trước khi bắt đầu
fuse A4. Actual118 vẫn là điều kiện hình học, không dùng tọa độ feedback thay request.

Run `stable` hoàn tất194,715s, không controller/service/probeerrors. Actual IDs1B `(3,9)` /2A `(3,5)` /
3B `(3,7)` /4A `(3,5)` /5B `(3,7)`, power1/2/1/2/1. Mỗi view năm159pointerkhác0/full39;
root2 chỉ process2, Box3,6 flag1→0, B3 sau đó process dưới owncallback3. B3 đã process trong owncallback trước khi đặt
cặp mới; B5 được nhận trước deadline A4 **4,753103927s**. Callback A4 process4,5 ở cả hai view,
Box3,6 đãclear trước callback4. Bốn native158/full13/view; own158 của5 không quan sát trước dừng phép đo,
không gọi năm entity là năm callback hoặc chứng minh mọi disposal/reset. 18snapshot195flags (9/view)
và processedsets của bốn rootdeadline khớp service CP094 không đổi. Verifier runtime49relations,
3546copiednativegetters/3510addresschecks là mức emulation riêng; thêm6isolation/source/profilechecks.
Phép đo kết thúc chủ động, không xác nhận result/fullcycle/gate.

Ca hẹp này không chứng minh immutable terrain cho mọi recursive cascade, FIFO, shared blocker,
first-bomb-stop tổng quát, direction/template/moving bomb hoặc genericmap. Auth/distance/fullvalidator/
fullreset/fullmatch/K5/K6a/Windows nguyên bản/LAN/Internet vẫn mở.


<a id="adjacent-owner-control"></a>
### Hai owner sát một ô trên ô thùng đã phá — CP096

[Recipe/reuse/hash/audit](../evidence/adjacent-owner-control/README.md). **kr_runtime/confirmed hẹp**,
currentmodifiedWine/ExVillage01/A-B/localstub, production CP094 không đổi. Tái dùng14sources và toàn
CP095profile/spawn4/nativeclear/stable-neutral input/getter/framer; chỉ target B của cặp cuối đổi từ
X280/column7 sang X240/column6. Không nativebody/patch/gamecall/force/packDBsource edit. Bảng121/
codec/map đã giải không được tính là RE mới.

Setup native1B `(3,9)`, 2A `(3,5)`/power2 và3B `(3,7)`/power1 giữ đối chứng blocked và clearBox3,6;
B3 process owncallback trước cặp mới. New actual118: A4 `(3,5)` owner0/power2, B5 **`(3,6)`** owner1/
power1, cách1ô. Sourcegrid ô3,6 là object1932/rawclass2 ban đầu; snapshot trước callback4 flag0,
không suy floor từ tên/icon. Cảhai view: native159 của5 full39/nonzero hoàn tất **trước** callback4,
latest same-round binding pointer khớp nativeprocess5. Setter state0 của5 caller`EBE21A` trước process;
callback root4 process `[4,5]`. B5 đến trước rootdeadline theo timestamp service, không dùng hold làm tick.

Run196,679s; năm native118/159/view, bốn158/full13/errors0/view, không controller/service/probeerrors.
195before/afterflags/processedsets cả bốn callback khớp service; 18snapshots/3546copiednativegetters/
3510addresschecks; verifier49relations +10adjacentbinding/geometrychecks +6source/isolationchecks.
Các mức runtime/getter-emulation/hash/test tách riêng. Own158 của5 chưa quan sát trước chủ động dừng;
không là fullresult/reset, firstbombstop/FIFO universal hoặc server gốc authority.

[AUD028](../evidence/checkpoints/audits/AUD028.json) sửa stale CP095PR, authority-index/roleboundary,
CP092/093historical “chưa tích hợp” và catalog implementationchain `unknown→partial` theo optionalpolicy
đã có; không nâng gate. [PLAN](../PLAN.md#current-order) ghép currentterrain/selectedTCPguards/hai chu kỳ
cùng run tiếp. Fullauth/codec/fullvalidator/location/distance/genericmap/recursive/sharedblocker/moving/
allreset/Windowsvanilla/LANInternet/fullmatch/K5/K6a vẫn mở; periodic sauCP100.


<a id="shared-blocker"></a>
### Shared blocker hai tia trong một root cascade — CP131

**kr_runtime/confirmed hẹp; hai model là design**, [metadata/hash](../evidence/three-bomb-chain/README.md#cp131).
Reuse CP090/095 actualmap/native195/boxcontrol, CP122–123recursivechain/currentmodel và VNgame/items
chỉ làm reference. Không body/codec/mapformat mới, không List2tạo tường/actorforce/terrainwrite.
MapSHAaeb57ffd…/seed/source/profile/client/fuse6 giữ nguyên tới cleanup.

R3 actual Aopening118 **(3,4)p2/ID1** → native158clears(2,4),(3,2),(3,6), full195 hai view khớp;
(2,3) và(2,2) vẫn1. Dàn vị trí trước fuse, actual Broot**(3,3)p1/ID2**, Asecondary**(2,4)p2/ID3**,
Abridge**(3,4)p2/ID4**, cả159full39 trướcroot158. Native hai view cùngprocess **2→4→3** trong
rootcallback, rootstate0callerE9EDFF, secondary4/3callerEBE21A; full15813/errors0.
Laterown158IDs3,4 chỉlookup/no kind0/process/state, không nổ trùng. Sau root: sharedbox**(2,3)=0**,
diagnostic**(2,2)=1**, all195flags khớp immutablecandidate; counterpart mutabledepthfirst
(clearboxngaygiữatraversal) dự đoán(2,2)=0 nên **bị bác cho input này**. Không tự suy toàn luật
globalFIFO/BFS/immutable hoặc thời điểm flag chuyển từ1→0: hiện mớibefore/afterroot snapshots.

R3198,045742258s/exit0/errors0/703raw/849checks,18nativeflagsnapshots/3546copiednativegetter/
3510arraychecks/153packet+6peer153 cases riêng; actual17sau blast cảA/B, không19/result/return/fullmatch.
Hai preparation/run trước giữ nguyên: R1thiếuimportre ởhelperwaitroot sauopening118/159, trướcfuse,
671raw/collectorerror; R2thêmimportonly, openingnative195đúng nhưngpostrootRighttarget160feedback
thấy225→280/Upblocked, khôngsecondary/bridge,675raw/partial. R3đổi dàn vị trí trướcfuse vàrootowner/
power, không singlevariableclaim hoặc sửafailedraw. NormalOSkeys/read-onlyfeedback; actual118quyếtđịnh
geometry, khôngduration/target. Runtimecopye9bb4766/static660447/modOa08950/base400000/RVA=VA−400000.
NextchỉPLAN: knownprocessphaseflag timing để giải nguyênnhân scopedcounterhypothesisfail, không
mởmọi chain/moving/validator/auth/gate. AUD040/kế132; reviewwaived khôngđóngK5/K6a.


<a id="shared-flag-timing"></a>
### Sharedbox flag tại process-entry và root return — CP132

**kr_runtime/confirmed cho bốn mốc/case**, [recipe/hash](../evidence/three-bomb-chain/README.md#cp132).
Reuse nguyên CP131R3source/profile/client/map/fuse6/inputgeometry, chỉ thêm readonly2flagwrappers
ở knownEB80E3/process-entry (root2/bridge4/secondary3 trong mainroot158index2) vàE5F5FD/rootreturn.
Actual118 vẫn openingA3,4p2/ID1, Broot3,3p1/ID2, Asecondary2,4p2/ID3, Abridge3,4p2/ID4;
nativeprocess2→4→3/full15939/full15813/laterown3,4noops/all195flags vẫn khớp cảhai view.

| Phase trong callback2 | Shared(2,3) | Diagnostic(2,2) | Phạm vi |
|---|---|---|---|
|process-entryroot2|1|1|hai client/ptrIDjoin159|
|process-entrybridge4|1|1|chưa đo mọi query ởbêntrong|
|process-entrysecondary3|1|1|cònflag khi vào process3|
|root158return|0|1|sau xử lýroot trướccallback thoát|

Kết luận hẹp: chưa thấy flag2,3clear tại ba process-entry, đãclear tạirootreturn; loại cách giải thích
“flagđã0trướckhi vào secondaryprocess” cho case này. **Không** suy flag chỉđổi sau mọi ray/cảcallback,
native dùng globalimmutable/FIFO hoặc mapflagduynhất quyếtđịnhray. [CP091](#house-after-box) có
queryEA0B10=1 rồi0 trong cùngcallback geometrykhác; giữ nguyên, không mâu thuẫn với bốn mẫu này.
Actualquery của sharedcell/caller/return trong geometryCP132 chưa bắt, bước kế duy nhấtPLAN.

Run209,060284766s/exit0/controller/service/probeerror0/696raw/843basechecks,18flagsnapshots/
3546native52A350/3510ED1350checks,8phase-samples/16nativephasegetterchecks,153packet+6peer153models.
Read-onlywrapperraw/cipher/checksum/gdbrolepaths riêng, không gamecall/statewrite/nativecounterchange.
ActualA17/B17 sau blast, không19/rank/return/no2rounds/fullmatch/K5/K6a/vanilla/LANInternet/gate.
Observer overhead khácCP131 được nêu, actualgeometry/order đã kiểm lại; không timer/fuse/sourceđổi giữa run.
AUD041 periodicqua132/kế136, reviewwaived; oldcomplete/raw giữ nguyên.


<a id="shared-cell-state"></a>
### Shared traversal cellstate chặn query lặp — CP133

**kr_runtime/confirmed hẹp + kr_static/cachedbranch**, [recipe/hash](../evidence/three-bomb-chain/README.md#cp133).
SourcefirstCP132phaseflags/CP091query/CP092storage, cachedEA0B10vàEB9710; no newnativebody.
Hai run cùngproduction/profile/client/map/fuse6/inputgeometry CP132, first thêmEA0B70queryreturn
chỉô(2,3)/(2,2) maincallback2/cap32; second thêmEB9719entry/EB982Creturn samecellfilter/cap32.
Cached2ranges EB9710/EA0B10 byteequal runtimeA/B/static/modO, không dùng equality làm runtimeproof.

Runquery217,712989226s/exit0/noerrors/716raw/864basechecks: hai view mỗi phía chỉEA0B10(2,3)=1
caller**EB975C**, rồi0caller**EB9BBB**cleanup. Khôngquery2,2trongwindow; không suyglobalabsence.
Map/currentlinear33/wrapper/nativeEAX khớp securegetter độc lập. Phasemarker vẫnsharedflag1trước
process2/4/3,0return. Temporal“latestprocess” chỉcontext, chưa gán calltree từthời gian.

Followup216,732498767s/exit0/noerrors/712raw/861base+821celljoins: nativeEB9710 root2 vàsecondary3
thực sự dùng **cùngcellstatepointer** ở ô(2,3), cùngmap, hai entry/returnstackframes/thread khớp.
Rootentry state**0**, EA0B10query1 bêntrong đúngstackwindow, returnstate**2**, EAX**0**.
Secondaryentry state**2**, returnstate**2**/EAX**0**, khôngEA0B10querytrongstackwindow.
CachedEB9710 **EB971Ccmp[cellstate],2→EB9721…EB9745return0** đi trước EA0B10callEB9757;
nhánh query1 đặtstate2tạiEB9763. Runtimeentry/return/queryjoin xác nhận shortcircuit trong case này,
không phải chỉ suy từmodel. Sausecondaryreturn mớiquery0cleanupEB9BBB, trướclastrootreturn.

Đây làcellstate của traversal, không nativeactortrappedstate hayallmapimmutablesnapshot. Không
chứng minh generalglobalFIFO/BFS/everyfirstbombstop/moving/allmaps, thờiđiểm/instructionxóaflagmap,
hoặcserverauthority. All195flags/native2→4→3/actualgeometry/laterownnoops vẫnkhớp cảhairun.
Mỗi run18flagsnaps/3546copiedgetters3510arraychecks/153packet6peer153,4querygetters và16phasegetters
riêng; modelgetters táchWineexecution. Actual17sau blast, không19/result/return/two-round/fullmatch/
K5/K6a/vanilla/LANInternet/gate. Khôngclientfunctioncall/statewrite/sourcebinary/DB/packmutation.
Bước kế chỉPLAN ghép result/reset2 bằng knownpolicy/actualreports, không mởmọi chainvariant.
AUD041qua132/kế136, reviewwaived; completedraw/checkpoint history bất biến.


<a id="shared-cascade-cycles"></a>
### Shared cascade/actual all-dead/result/return/reset hai vòng — CP134

**kr_runtime/confirmed hẹp; result/fuse/profile là emu_choice/design**, [recipe/hash](../evidence/three-bomb-chain/README.md#cp134).
Reuse CP131–133actualopening/sharedgeometry/native195/cellstatemechanism, CP084all-deadfixture đãcó
trongprofile vàCP123ReadyB/StartA. Giữ source/profile/client/map/fuse6/inputroute, bỏ early-stop sau
rootđểchờ report chết thật; dùng minimalCP131probes, khôngđo lạimọi query/cellstate hoặcgọi hàm game.

Same-run378,168184386s/exit0/errors0: R1Aopening3,4p2/ID1 rồiBroot3,3p1/ID2/Asecondary2,4p2/ID3/
Abridge3,4p2/ID4; R2samegeometry/powers/owners/IDs5,6,7,8. Nativeprocessroot2→4→3 rồi6→8→7,
full15939/full15813/laterownnoops vàfull195before/afterflags mỗi callback khớp hai view. A/Bactual17
rồi19 mỗi epoch, khôngfake19/forfeit; chosenall-deadrank2/2 sauactualboth19, native175302/ranks2,2/
1803/room1106 hai lần/cảhai view. R1return→actualB95/A104→121129/clock2/key12346; cùng8TCP.

Toàn195seed reset ởclock2,24clockstatvalues khớp A1/1/5 vàB2/1/4 (capacity/power/logicalspeed)
ởhai view/hai epoch; actualA124 Bomb/Fire lại hai vòng. Đây khôngfullinventory/character/mode/pxs/TTL
reset law. FinalUIhai phòng cóAhost/Bguest/Start-Ready, không thêmReady2 hoặcStart3chưađo.
1212sealedraw/1454cycleschecks/36flagsnaps7092copied52A350/7020ED1350arraychecks/5178actorfields/
354packet12peer153models; overlappingchecks không là các proof độc lập hoặc phần trăm tiến độ.
Modelgettercounter reset chỉemulation, khôngWine. Sourcebinary/DB/pack/oldcomplete/raw giữ nguyên.
Hai chu kỳ thử hoàn tất cùngrun/profilemodifiedWine/localstub, không tựnghiệmthu fullmatch/K5/K6a/
vanilla/LANInternet/authority/allreset. Bước kế chỉPLAN connectedstalledload; không mởchainvariantvôhạn.
AUD041qua132/kế136, reviewwaived khônggateclosure.


<a id="terrain-tcp-guard-cycles"></a>
### Terrain và TCP replay guards cùng hai chu kỳ — CP097

[Recipe, nguồn tái dùng và hash](../evidence/terrain-tcp-guard-cycles/README.md).
**kr_runtime/confirmed trong scope copy Wine/A-B/ExVillage01/localstub**; production/profile CP094
không đổi. Reuse CP094route/terrain/native195flags, CP085proxy/frame/body/seq joins và CP096neutral
input. VN lifecycle/report-confirm giúp định hướng; VN session probe cấp khóa cố định12345 nên không
chứng minh KR generation. Không copy opcode/layout/key VN, không thêm nativebody hoặc sửa pack/DB.

Run đầu `cycles` có hai nativecycles/14entities/12callbacks/view/52flagsnaps nhưng proxy A-only
không capture17/19 do report nhốt/chết thực sự từ B: chỉ7replay, target còn thiếu. `peer` đổi selector
proxy sang **A118/124, B127/sub17,19**, giữ body đã capture và chỉ resequence outer C2S như recipe cũ;
đã thấy bốn samebody nhưng controller lỗi Left sau Up overshoot, chỉ6entities vòng1. Raw/failures giữ.
Run `stable` tái dùng ba mẫu neutral giống nhau/pulse45ms để căn A hàng3 trước extras; lowerpair
inputgap2→3s, fuse vẫn6s. Điều kiện geometry/timing/guard không đổi để làm pass.

Run cuối306,328s có12replay thực qua TCP: mỗi vòng một samebody cho118/124/17/19, cộng bốn body
vòng1 gửi lại ở vòng2 sau actual39 của chính role. Nativefront/clientseq và upstream/serverseq/SHA/
bodyorigin/typedfields được nối riêng; proxy-emitted replay không là native writer emission. Rejects:
118same=`duplicate_bomb_serial`, old=`bomb_round_key_mismatch`;124same=`claim_not_current_or_already_consumed`,
old=`round_key_mismatch`;127same=`duplicate_report_body`, old=`report_tick_not_forward`.
Không replay nào thêm ID/broadcast165/forward170. Accepted124/118/127 đều nối actualnativefront.
A nhận bốn165 (Bomb/Fire hai vòng), nativecapacity/Fire1→2, Start2 trở về1/1 ở cảhai view.
ReportB17/19 được nhận và phát170 tớiA; tổng170 chỉ bằng bốn acceptedremote reports. Probe không có
native170entrycounter, nên không suy từng callback170 đã được quan sát từ count hay logservice.

14acceptednative118/159 nonzero/full39 ở mỗi view; **11native158/full13/view**, không12 hay14:
jobs7/13/14 bị `round_closed` trước deadline, không158/terraincommit; cảba chưa nativeprocess trước
roomdispose. Actualextra7/14 ở `(3,2)`, xa3ô so vớiroot6/13 `(3,5)`, không tự gắn expectedchain.
48snapshot195flags/processedsets khớp service;9.456copiednativegetters/9.360arraychecks là emulation
riêng. Verifier12callback dự kiến fail được giữ; correction nối đúng actualbroadcast158→nativecallbacks
và exactunsent/fencereasons, không tạo callback giả hoặc sửa raw. Hai121/clock/175302/1803/refresh;
trước175 cảhai view có Astate0/Bstate1, rankstorage rowtypes1/1 và ranks1/2 trước return ở cảhai vòng.
Đó là scope observed/profile, không suy enum/authority server gốc hoặc mọi reset.

128terrainrelations,219transportjoins,43nativeeffect/storagechecks,12sourcechecks; các tập không độc lập.
Stable metadata preparation bị FileNotFoundError ở bước chép reuse JSON **sau khi controller/GDB/profile/
prefix đã tạo**; shell tiếp chạy. Metadata được ghi lại sau launch và nói rõ boundary, không giả preseal;
originalfinal harnessSHA/policy_source/profile pins khớp file hiện hành. Fullmatch/K5/K6a/fullauth/codec/
location/distance/forgedfreshbody/fullvalidator/genericterrain/allreset/vanilla/LANInternet vẫn mở.
[PLAN](../PLAN.md#current-order) tiếp finite native movementapply hai chiều trong currentrun; không dùng
pairedsamples hoặc counter tổng thay packet→callback→actor proof. AUD028/nextCP100.


<a id="rust-terrain"></a>
## Rust boundedterrain/reveal/ba basicitems — CP239 / M3partial

[Code](../../rust/crates/kr-service/src/terrain.rs), [claims](../../rust/crates/kr-service/src/pickups.rs),
[raw/metadata](../evidence/rust-service/terrain.json), [recipe](../evidence/rust-service/README.md#cp239).
SourcefirstCP172/174 vàPythonTerrainShadow/load_terrain/current142guard; CP238ledger-recipientcompletion và
CP237floorcatalog. Không binaryfunction/wirelayoutRE mới, không importVNopcode/engineRewrite.

Policy`design`: optionaltrustedterrain spec từcùngPythonprofile/export,15×13/195kinds0..2/maxpower1..2,
seedSHA vàrawmap/startframeSHA khớp trước bind; mapassetdigest là provenance dooperator khai, không attestation.
Candidategeometry kiểmfloor/solid/box/pendingoccupancy trướcpublish IDserial/reservation. Cascade dùng immutable
snapshot cho cảroot/secondary/raystop; clearbox mộtlần. Chainsecondary own158 sau đó có thểkhôngdelta nhưreference,
khôngreleasecapacitysớm hoặc nhận mọichainvariantđãnative.

Sau158write thànhcông **đủ originalroster**, roomowner commitblast/clearcells cùnggeneration/rootID.
Partialrecipientwrite khôngclearance; EOF/newround resetpending/clearance/claims.142known7bytes kiểmboundslot,
auxFFFFFFFF/knownhidden cell/currentgeneration+committedclearance; duplicate idempotent. Chỉđúngôhidden từcùng582
mới chuyểneligible, rồi124samekey/catalog/onewinner→165; không cấp broadworlddistance/authauthority.
Clientlogicalcounter khôngserverclock/serial; **old142bytes saulegitimatecurrentclear khôngphânbiệt**, giữCP172limit.
Bombpickupkind0 cộngpendingcapacity theo designAmax10/Bmax7 hiệncó; khônggeneralcharacterstats/mọiinventory.

`kr_runtime/confirmed` hẹp trênKR00967/e9/Wine/privatevirtualLAN: actualAbomb118/root1, hai158native;
maparray195cells mỗiview tạiclock/before158/after158: trước bằngseed, sau đúng195flags theoPythonkernelđộc lập
vớiactual118, chỉclear(2,3)/(3,2). Không lấyRust-generatedexpectedlàmoracle.
142reveal đúngô saufullroster158commit/duplicate, actualA124indices17→28→27→mỗiview3native165/parsedtuple,
itemfound,12/12/errors0; protectedactorfields **5E8 5→6 (Roller17),4BC1→2(Fire28),4A4 1→2(Bomb27)**.
1314copiednative52A350getters xácminhrawflags/statwrappers táchruntime; rawmodO/counterreset/emulationlimits cũgiữ.
TCP410/66exactreplies/alltuples-C2Sjoins,6shold/FINsaucompletion;54raw/86.998s frozen-retained-seal/cleanup0.
UI hai viewlocalarrow/peer/cells/itemgone/cùngtimer; không gamefunctioncall/statewrite/clientpatch mới.

35Rusttests/43kernelTCP kiểmgeometry/block-chain/reset/clearance, invalidclaim/phase/dupe,
**first158recipient chưa mởclearance, secondwrite mớicommit**, hidden124trước142nogrant/142duplicate/once165,
oldgeneration/EOFpending/queuefail khôngmấtIDserial. Negative/modeltests khôngnativeattackproof.
New180sexperimentbudget khai báo trước run cho readonlyfeedbackroute; actual86.998s, nominal45ms OS pulsekhông
phải gametick/physicslaw, không tăng service shutdowndeadline. Compile helperLowerHex/privatevisibility đãsửa,
failedlogs giữ; không có runtime ởbinarychưabuildxong. Source/config/inputfrozen trongrun.

M3 cònlife/report-derived17-19-21→170/results175/180/reset/twooutcomes/oldpacket-currentpositive controls.
Không nativechainvariants/allmaps/generalphysics/auth/kim-cứu/capacity4-8/fullM3/fullservice/production.
Opaque123/167 policy/semanticsunknown vàshutdownrootcauseunknown giữ. Pythonserving/RE/reference,
[PLAN](../PLAN.md#current-order) duy nhất giữ thứtự. CP240auditrequired; Internet thật chưa kiểm.

<!-- protocol-topic-content-end -->


<a id="rust-player-terrain"></a>
## Terrain và ba item nền theo account/roster hiện hành trên Rust

[Ma trận, recipe và hash](../evidence/rust-service/g1-terrain-players.json) ghi nghiệm thu G1 lát 4.
Tái dùng [kernel/claim KR đã đo](#rust-terrain), [item effects](items-stats.md#item-effects) và catalog nhân vật hiện hành.
Bật `fixed_room.item_policy = "report_derived_basic"` cùng account và `managed_rooms.player_round`; mặc định tắt.
Policy nằm trong `RoomRules`, không chọn hành vi theo sức chứa hoặc identity. Python giữ công cụ RE/reference.

Mỗi member có stat hiện hành từ nhân vật đã chọn. Bomb/Fire/Roller tăng capacity/power/speed một mức và kẹp
max của catalog nhân vật; reset/bắt đầu vòng mới nạp lại baseline. Không đổi stat catalog dùng chung.
Request118 phải có power đúng stat hiện hành; capacity và maximum lấy từ nhân vật, không từ slot hay tên account.
Validator đo được cho phép owner0..7 và power tới9 khi terrain profile cho phép; profile cũ giữ giới hạn riêng.
Native mới chỉ chứng minh power1→2; không gọi source bounds tới9 là native mọi tầm nổ đã đạt.

Terrain vẫn là shadow policy 15×13, floor/solid/box và pending occupancy. Chỉ commit blast sau158 của toàn bộ
recipient hiện hành đã hoàn tất. Nếu recipient cuối rời đi sau những người còn lại đã gửi xong, commit cho survivor;
không để bom chờ vô hạn hoặc dùng completion cũ vào vòng mới. Khi không còn member, kết thúc ledger/terrain/claim.
Report142 phải có bound slot, known hidden cell và clearance đúng vòng hiện hành trước khi mở item. Trong policy
item opt-in, aux là u32 opaque, không chọn item/index và không tự cấp clearance; giá trị khác FFFFFFFF có budget64.
Guard chỉ nhận FFFFFFFF trước đây đã chặn report thật trên Village10; [bằng chứng và sửa policy hẹp](../evidence/rust-service/g1-village10-terrain.json)
không phục hồi nghĩa aux hoặc authority server gốc. Đường bomb-only vẫn chỉ quan sát, không effect. Raw142 không có
epoch xác thực: body cũ sau current clearance hợp lệ vẫn không phân biệt được. Đây không phải xác thực vị trí hoặc authority server gốc.

124 kiểm current membership/slot/key/catalog, một winner và phase; công bố claim/stat cùng reservation toàn bộ
hàng đợi165. Hàng đợi đầy không tiêu item, cộng stat hoặc chiếm serial/ID dở. Duplicate không cộng lần hai.
Guard/queue/timer dùng room/round scope và ownership hiện hành, không cho batch của phòng khác hoàn tất thay.

**Native trên Wine/LAN ảo:** các roster2/4/8 có ba item Roller17, hidden Fire28 và Bomb27; cùng một binary Rust,
account và companion xác thực. Mọi view đọc165 đủ12byte/errors0 và thấy actor0 nhân vật1 tăng Roller4→5,
Fire1→2, capacity2→3. Mỗi vòng có118 power1 trước Fire và power2 sau đó,159 đọc đủ39byte/entity đúng tọa độ,
158 đọc đủ13byte/lookup đúng ID; hai bảng kết quả và reset hoàn tất. Đầu vòng2 stat trở về baseline ở mọi view.
Các report142 lặp không tạo thêm165/stat delta. Snapshot effect chỉ có slot0/1; clock kiểm mọi actor hiện diện.

Native array195flags so với kernel Python độc lập được dẫn bằng actual118 và thứ tự158. Đã đo đủ các blast vòng1,
và seed/root vòng2 theo quota đã khai; geometry cuối vòng2 chưa được chụp. Python model dùng owner0 cho ray math
khi owner cao hơn1, không xác minh quyền owner; ownership/serial/power/tọa độ đã đối chiếu riêng qua capture và
native consumer. Getter emulation trên snapshot tách khỏi runtime; log server chỉ nối ID/sequence.

**Giới hạn:** ExVillage01 và ba item nền, không sáu map ladder hoàn chỉnh, mọi item hoặc mọi nhân vật.
Native item trên sparse roster, maximum gameplay tới9, native EOF của lát mới và nội dung khác chưa có proof mới;
source controls không thay thế native. M8 còn Internet transport mới, tải và vận hành. Không mở lại K6c/K7.


<a id="rust-selected-map-terrain"></a>
## Terrain riêng theo map được chọn và lát Forest07

[Ma trận, raw/hash, recipe và giới hạn](../evidence/rust-service/g1-map-terrain.json) giữ bằng chứng của lát này.
`fixed_room.map_choices[*].terrain` là tùy chọn cùng policy item nền đã có. Mỗi spec kiểm map payload SHA của
profile pack và hash frame map/start của chính lựa chọn đó. Root `terrain`, nếu cùng được khai, phải trùng choice0;
không bắt buộc lặp spec ở root. Những hash này kiểm input đồng bộ, không chứng nhận dữ liệu operator là server gốc.

Phòng mới clone seed của choice0. Current Idle host chọn96 thì candidate room thay terrain, kết thúc claims cũ và
xóa last blast; sau reservation đầy đủ mới công bố model cùng selected map, floor catalog và revision. Hàng đợi đầy
không đổi nửa map. Choice không terrain xóa shadow cũ. Chọn lại cùng index vẫn có revision mới để chặn batch117 cũ.
Từng phòng giữ state riêng; Start của vòng mới nạp seed/catalog/stat hiện hành, không tái dùng clearance map trước.
Unknown tile không tự chuyển thành floor; spec có ô chưa phân loại bị từ chối trước bind.

**Forest07 có footprint đa ô.** Profile tĩnh từng ô đã bỏ sót năm ô chặn quanh `Well00`: anchor ở row12/column12,
vùng thực tế là row11..12/column12..14. Hai client native với binary đã nghiệm thu và terrain tắt xác nhận cùng
195flags. Đây là sửa ngoại suy candidate, không sửa pack hoặc exporter; mẫu dữ liệu static class1 được đối chiếu
với vùng chặn thực tế. Không áp footprint3×2 này cho mọi object có tên Static hoặc mọi map.

Lát gameplay mới dùng tám account KR/Wine, từng spawn pack hiện hành và terrain riêng của Forest07. Slot6 gần giếng
đi bằng phím thật, phá thùng, nhặt Roller ẩn đã cấu hình và tăng speed5→6 trên tám view. Sau actual root row11/column11,
power1, cả sáu ô giếng vẫn chặn trong khi thùng ở10,11 và12,11 bị phá. Hai vòng có tất cả195flags trước/sau18blast,
reader159/158 đầy đủ, hai rank/reset và baseline vòng2 đúng. So geometry bằng kernel Python độc lập từ actual118;
không dùng log Rust hay encoder tự sinh làm oracle. Getter emulation trên snapshot tách khỏi native runtime.

Roster2/4 có route riêng phá thùng và nhặt item ở11,2, hai kết quả/reset và baseline stat mới trên mọi view.

**Phạm vi còn thiếu:** route cạnh giếng chỉ một hướng/power1 với roster8; không suy mọi sparse roster hoặc toàn bộ map đã đo.
Một Roller ẩn cố định là policy thử nghiệm, không phục hồi phân phối drop,
trọng số ItemAttr hoặc mọi item trong bảng Forest07. Catalog fallback Factory07 chưa có terrain/native mới ở run này.
Sáu map/nhóm item/nhân vật và M8 giữ acceptance riêng; không tự đóng G1 hoặc giai đoạn1 từ lát này.
[Phạm vi roster hiện hành](../evidence/owner-directives/g1-map-roster-limits.json) cho Village10/Tomb12 là 2–6 người;
bốn map còn lại là 2–8 người. Đây là quyết định nghiệm thu, không phục hồi wire/giới hạn gốc hoặc sửa kết quả cũ.


<a id="rust-patrit-terrain"></a>
## Lát terrain Patrit14 theo map hiện hành

[Bằng chứng và giới hạn](../evidence/rust-service/g1-patrit-terrain.json) tái dùng đúng implementation per-map
đã có. Lượt tám người bổ sung measured167/body52 vào observation có budget của managed player rounds;
không ACK, sửa world/stat hoặc gán semantics. Hai/bốn người dùng binary trước delta, tám người dùng binary sau delta.
Seed riêng của Patrit14 giữ overlay cho MastTileA tại row6/columns6..8; hai view native xác nhận cùng vùng
chặn, trong đó hai ô ground candidate trước đây thực tế bị chặn. Data tĩnh có mẫu3/1 tương quan với footprint,
nhưng chưa là contract chung cho mọi object đa ô. Profile xuất và pack gốc giữ nguyên.

Các roster account 2/4/8 đi bằng phím thật từ spawn pack, phá thùng theo năm root, nhận Roller ẩn cấu hình tại4,8,
và thấy stat actor1 tăng5→6 trên mọi view. Actual root5,8/power1 giữ ba ô Mast chặn đồng thời phá hai thùng cạnh,
lặp cả hai vòng. Toàn195flags trước/sau mỗi blast được so từ actual input với kernel độc lập; reader/entity,
owner/serial/location/power, rank/reset và baseline spawn/stat vòng sau được kiểm riêng. Getter emulation trên
snapshot tách khỏi runtime. Lượt source/TCP/config trước vẫn giữ giới hạn, không thay native evidence.

**Giới hạn:** route từ phía trên/column8 và power1, roster contiguous đã ghi; chưa mọi sparse roster hoặc toàn bộ map.
Một Roller cố định là policy thử nghiệm, không phục hồi phân phối drop hoặc các item còn lại. Factory07 fallback
không có terrain trong lượt này. Sáu map/item/nhân vật và M8 tiếp tục theo acceptance PLAN, không đóng toàn bộ G1.


<a id="rust-village10-herb"></a>
## Village10: Herb và prototype theo số spawn của map

[Bằng chứng, recipe và giới hạn](../evidence/rust-service/g1-village10-herb.json) tách seed hai client khỏi
phép đo tia nổ trên sáu account KR/Wine. Mọi ô Herb có flag seed 0; các ô còn lại khớp phân loại ứng viên.
Trong route đã đo, actor slot 4 nhặt Fire cấu hình rồi đặt ba bom power 2 tại row0/columns5,6,8. Native query
trên Herb0,5 trả object với context dữ liệu lớp3 và flag0; lần nổ sau không còn object tại ô đó. Tia từ0,8 đi qua
Herb mới0,9, rồi dừng ở nhà0,10 bằng cellstate2/result0. Nhà vẫn chặn; flags trước/sau mọi blast trên mọi view
khớp kernel Python độc lập dẫn từ actual118. Không lấy flag0 riêng lẻ làm bằng chứng tia đi qua.

Loader nhận thêm số dòng prototype trong khoảng2–8 chỉ cho DB-owned managed player rounds, cấp slot trống và
không dùng legacy Battle. Template, packet set và ordinal spawn vẫn phải nhất quán với profile pack. Parser Start
nhận shape hữu hạn2–8; profile legacy giữ guard riêng. Tên cố định ở fixture chẩn đoán không làm chủ identity người
chơi. Native sáu người xác minh Start/spawn/Fire và ba blast của binary mới; bằng chứng roster3/6 cũ được dẫn riêng.
Không thêm cờ hoặc nhánh runtime theo từng sức chứa. Công cụ xuất map giữ nguyên và ô chưa đo vẫn không được đoán.

Quan sát Herb ban đầu chưa bật terrain shadow. [Lát terrain tích hợp](../evidence/rust-service/g1-village10-terrain.json)
đã bật seed đó cho roster contiguous2/4/6, cùng một binary mới, hai kết quả/reset. Actor0 ở roster2/4 đi từ spawn gốc,
phá bốn thùng hành lang bằng power1, rồi nhặt Fire tại0,6. Actor4 ở roster6 đi từ spawn riêng tới cùng Fire; power tăng1→2
và cả ba roster phá theo roots0,5/0,6/0,8, nhặt Roller ẩn cấu hình tại0,8, tăng speed và reset về baseline ở mọi view.
Tất cả195flags trước/sau mỗi blast được so với kernel Python độc lập dẫn từ actual118; reader/entity/ownership kiểm
bằng capture/native riêng, getter emulation tách khỏi runtime. Các chỉ số hoặc kết quả không suy từ log server.

Report thật trên ô Roller có aux0, nên guard FFFFFFFF-only cũ từ chối claim dù client đã reveal. Policy item opt-in
nay dùng known cell và current committed clearance cho mọi aux, vẫn kiểm ownership/budget. Đây là lựa chọn công ty;
không gán nghĩa cho aux0/3/5 hoặc biến report thành world authority. Lượt thất bại trước giữ nguyên raw/exit1.
Fire đặt sẵn và Roller ẩn là input thử nghiệm, chưa là phân phối drop gốc. [Catalog tám/sáu spawn và sparse6](load-phase.md#rust-map-spawn-capacity)
đã có hai vòng mới với slot0/2/4/5/6/7, Fire/Roller và spawn/stat ở mọi view. Native/capture kiểm cả20 vụ nổ, nhưng
budget snapshot40 chỉ giữ195flags trước/sau19 vụ; cặp cuối vòng hai còn thiếu và không được suy từ mô hình. Lượt cũ
contiguous2/4/6 có đầy đủ flags trong phạm vi riêng. Chưa nghiệm thu toàn khoảng2–6, toàn route/hướng/item hoặc luật
server gốc; không giảm sức chứa bốn map tám spawn xuống sáu.

Run tia nổ trả exit1 do unmount busy sau khi đã export. Lỗi giữ nguyên; tất cả file export được so đúng hash,
namespace đã hết process rồi mới phục hồi vị trí raw và seal. Không gọi exit1 là exit0. Lượt seed cũng ghi riêng
hai output cache `fxdynmapinfo.ssd` thay đổi khớp bảng238 thực nhận; không sửa preflight để che thay đổi input.
Getter emulation tách khỏi native; không có nghiệm thu Windows, Internet hoặc G1 đầy đủ từ lát này.


<a id="rust-camp08-boundary"></a>
## Camp08: seed, điều kiện report bom và giới hạn phục vụ

[Bằng chứng seed và blocker ban đầu](../evidence/rust-service/g1-camp08-boundary.json) giữ Start/spawn/195 flags
trên hai account KR/Wine, getter emulation và các run lỗi. Ô 12,11 có flag chặn dù candidate là ground, cạnh
MissileBase; nguồn footprint và ray đã được đo riêng dưới đây. Camp08 chỉ có các route private đã đo; chưa mở
preset gameplay tổng quát.

[Đối chứng report122](../evidence/rust-service/g1-camp08-report122.json) đã so ô thường 6,0 với ô 6,2 từng được gắn
nhãn tĩnh PinGround. Cả hai trả object `CGOTileGround` ở hai view; predicate lần lượt là `[0,FFFFFFFF]` và `[1,4]`.
Do đó tên PinGround chưa xác định loại object hoặc nguyên nhân report. Actual TCP của owner có 118, rồi 122/17 byte,
rồi 167/12 byte; report122 khớp writer `B731D0`, layout `[u32,u16,u16,u32,u8,u32]` và giá trị
`FFFFFFFF,6,2,4,255,0`. Getter/protected property vẫn FFFFFFFF trước local effect, không được ghép tùy ý với ID server.

Rust có `BombReport122` và parser kiểm đúng 17 byte trong `kr-wire`; fixture lấy nguyên body native và giữ đủ u32
sentinel. [Bằng chứng cờ khởi tạo và policy phục vụ](../evidence/rust-service/g1-camp08-cancellation.json) tách nguồn
client khỏi lựa chọn server. Tại khởi tạo, `EA1EE0` gọi `1363D90` sao 72 byte TileInfo; `ED2350` kiểm type 0 và
signed short tại offset 62 lớn hơn hoặc bằng 10000, rồi `EA5080` ghi cờ map. Native hai view xác nhận toàn bộ 195
cờ khởi tạo: chỉ các ô `(2,5)`, `(6,2)`, `(6,12)`, `(10,9)` bật. TileInfo của chúng là `Tile16_Ground/PinGround01`,
code 2820, footprint 1×1. Đây là bằng chứng từ dữ liệu và consumer khởi tạo, không suy luật từ tên tile.

Policy Rust là **design**: mỗi map choice có `bomb_cancellation` tùy chọn, ràng buộc SHA-256 payload map, tọa độ
không trùng và nằm trong 13×15. C2S118 vẫn phải qua current membership, round key, owner/slot, power, serial,
capacity và cell guards. Ô bật policy chỉ tiêu thụ serial; không cấp wire ID, không chiếm quota, không phát 159
hoặc hẹn 158. Tối đa 64 serial bị hủy cho mỗi membership trong một vòng; Start mới xóa tombstone.

C2S122 chỉ được quan sát cho variant `FFFFFFFF,row,column,4,255,0`, tại ô được bật của map đang chọn và current
member. Armed có ngân sách 64 report mỗi người mỗi vòng; Ending/Idle sau kết quả dùng ngân sách tail hiện hành.
Report không tạo ACK, không sửa terrain, không hoàn tất hoặc giải phóng bất kỳ allocation nào. Vì packet không
mang serial/epoch, không chứng minh được raw freshness; ngay cả report cũ cùng ô cũng không có quyền tác động bom
mới. Chuyển map sang choice không có policy giữ fail-closed122. Tests tự động kiểm isolation, serial, map, member,
ngân sách và reset; nghiệm thu native binary mới được ghi riêng trong evidence, không suy từ tests.

Phân tích tĩnh có mục tiêu nối predicate qua `B3B5D0` tới `EA08E0`. Native đã phân biệt nhánh cờ map:
ô 6,0 có scalar và getter trả 0, ô 6,2 trả 1 ở cả hai view. Virtual `+0xD0` của CGOGameBomb là `B73290`;
actual167 có layout `[u16,u16,u32,u16,u16]`, giá trị `0,1,29,6,2` ở A và `1,1,29,6,2` ở B. Getter serial native
trả 29 ở cả hai. Không gọi field đầu là owner bom: hai giá trị khác nhau cho cùng bom do B đặt.

Cả owner và peer đi qua writer167 rồi `E9E660` với cùng serial/tọa độ. Owner còn property FFFFFFFF lúc vào writer,
peer đã có ID server 2; property của owner còn đổi cục bộ trước cleanup, chưa gán semantics network ID.
Đây là effect client tự thực hiện, không chứng minh response server gốc. Các lượt đối chứng trước policy Rust
dừng có chủ đích sau denial122 của binary cũ.
Policy map-owned cancellation đã kiểm native hai account: mỗi vòng có một placement bị hủy, actual122 được quan sát,
không có 159/158 cho placement đó; bom thường trước và sau vẫn hoàn tất consumer ở cả hai view. Hai kết quả/reset
và Start thứ ba đã chạy. Lượt đầu lỗi recipe do thử di chuyển B sau local effect gây nhốt/chết; raw giữ nguyên,
lượt sau dùng A còn sống cho positive. Không suy toàn bộ tác động PinGround lên nhân vật hoặc response server gốc.
Các terrain/item và đủ roster Camp08 còn mở;
[khoảng trống](../RESEARCH.md#gap-camp08-bomb-report) giữ phép nghiệm thu kế.

[Thùng nhiều hit và route Rust](../evidence/rust-service/g1-camp08-durable-boxes.json) đã đo StrongBox01 tại `(7,2)`
chặn sau bốn hit và hết chặn ở hit thứ năm; StrongBox02 tại `(8,3)` hết chặn ở hit thứ hai. Các hit dùng power 1 và
ô đặt thường, cả hai view có đủ consumer 159/158. Actual object là `CGOTileMoveAbleBox` từ `E9FF80`;
`B9F460` nạp dword TileInfo tại offset 24 vào protected counter `+0xD8`, 0 đổi thành 1. `B9FDD0(0)` giảm counter,
chỉ xóa chặn khi về 0. Guard tĩnh `BA01C0` giữ giá trị từ 255 trở lên; variant này chưa kiểm runtime.
Native initializer và từng counter trước/sau hit được đối chiếu riêng. Lớp game-box `B9C760/+0xC4` là ứng viên sai nhánh, không dùng để chứng nhận StrongBox.

Terrain Rust thêm `box_hits` tùy chọn: vắng thì mỗi ô box có một hit như trước; có thì đủ 195 giá trị, box nhận
1–254 và ô khác nhận 0. Đây là policy design hữu hạn. Mỗi callback hiện hành trừ một lần cho ô được ray đánh dấu,
chỉ cấp clearance sau hit cuối đã gửi đủ mọi recipient; callback lặp hoặc timer của secondary đã xử lý không trừ
thêm. Ray vẫn dừng tại thùng ở hit cuối, Start mới khôi phục seed/count và xóa clearance. Config vẫn ràng buộc map,
seed và frame map/start. Ngân sách, queue, generation và membership guards giữ nguyên.

Private route với binary mới đã qua hai vòng: hành vi 5/2 hit, hidden Bomb ở `(7,2)`, hidden Fire ở `(8,3)`,
nhặt/165 ở hai view, power 1→2, bom thường, hai kết quả/reset và Start thứ ba. 18 bom thường hoàn tất consumer ở
cả hai view; DB ghi hai trận hoàn tất và trận thứ ba hủy khi cleanup, thưởng 0. Hai item là dữ liệu test design,
không phải xác suất drop server gốc. Seed của những ô khác chỉ đối chiếu cờ khởi tạo cho diagnostic này; không coi
lượt đó là nghiệm thu Herb/Line/MissileBase/PinPillar hoặc toàn terrain/roster Camp08.

Lượt tích hợp sau main chứa thay đổi L1/account-auth dùng binary mới, giữ cùng source kernel và client G1 949.
Route hai vòng/item/kết quả/reset/Start kế được kiểm lại trên binary kết hợp; metadata giữ riêng hash hai service.
211 tests trên source kết hợp đạt. Kết quả này không mở acceptance localization hoặc đổi wire client G1.

[Route phục vụ tám account](../evidence/rust-service/g1-camp08-serving-roster.json) đã dùng cùng binary kết hợp,
terrain có hit 5/2 và tám spawn gốc. Hai vòng có 24 bom thường, hidden Bomb/Fire được nhặt bởi B, power 1→2;
cả tám view hoàn tất consumer 159/158, bảng kết quả tám người, return/reset và Start thứ ba. Từng callback
được so toàn bộ 195 cờ với model hữu hạn độc lập từ actual C2S118; các clock mới khôi phục cờ và chỉ số ban đầu.
Journal của lượt chạy có hai trận hoàn tất, trận thứ ba hủy và thưởng 0. Đây là route riêng, không chứng nhận
mọi ô, hướng, item hoặc các roster còn lại của Camp08. Output và DB trên tmpfs chỉ là đối chứng chẩn đoán,
không thay nghiệm thu lưu bền/Internet/vận hành M8.

[Ma trận roster 2/4/8](../evidence/rust-service/g1-camp08-roster-routes.json) bổ sung route bốn account,
hai đội AB/CD với slot và spawn gốc. Có 20 bom, hai vòng StrongBox 5/2 và hidden Bomb/Fire, bốn view hoàn tất
consumer/kết quả/return, Start thứ ba và 43 snapshot × 195 cờ mỗi view khớp model độc lập từ actual C2S118.
Journal có hai trận hoàn tất, trận thứ ba hủy và thưởng 0. Đây là coverage của route đã chọn, không mọi ô,
hướng, sparse roster, item hoặc nhân vật của Camp08.

Lượt bốn account đầu mất mẫu vị trí B sau ba bom đúng tọa độ, trong khi A còn phát mẫu. Không thấy fatal,
probe error, QueueExpired hoặc lỗi service trong phạm vi log đã giữ; nguyên nhân vẫn unknown. Lượt tiếp có
thu process/thread/focus khi stale và một đối chứng focus giới hạn, nhưng không kích hoạt các nhánh đó và route
đạt. Không suy đây là lỗi đã được sửa hoặc một chẩn đoán focus; tmpfs vẫn chỉ là đối chứng chẩn đoán.

Lượt tám account đầu dừng trước nhặt item: hai actual C2S118 phát từ `(8,1)` thay vì `(7,1)`, nên StrongBox01
còn hai hit. Cờ native của cả tám view khớp model theo tọa độ thực tế. Harness đã chấp nhận lệch tâm khi rẽ;
lượt đạt siết căn tâm và nhịp phím, không sửa luật terrain hoặc timeout service. Raw và input frozen của lượt lỗi
được giữ nguyên.

[Đối chứng Herb và Line](../evidence/rust-service/g1-camp08-passable.json) đã tách khả năng đi qua của nhân vật
khỏi tia nổ. Trên hai view, Herb01 `(0,2)`, Herb02 `(0,5)`/`(0,8)` và Line02 `(0,7)` đều trả
`CGOTilePassable`. Tia power 2 từ `(0,6)` đi qua Line02 tới Herb02 `(0,8)`; Line vẫn tồn tại trong callback.
Cờ seed và cờ sau nổ của Line02 bằng 0, nhưng A bị chặn khi đi từ `(0,6)` sang `(0,7)` sau lần nổ đó:
35 mẫu cuối giữ tọa độ `(240,39)` dù có input Right. Vì vậy flag 0 hoặc type 3 không chứng minh đi qua tự do.
Herb01 đã có ca đi vào khi còn nguyên; Herb02 `(0,5)` có ca đi vào trước lần nổ tại chính ô đó.

[Đối chứng Line01](../evidence/rust-service/g1-camp08-serving-roster.json) đã dùng tia power 2 từ `(1,6)`
đi qua Line01 còn nguyên `(1,7)` tới thùng `(1,8)` ở cả hai view. Line01 là `CGOTilePassable`, nhóm byte
24–27 bằng 0 và 32–35 bằng 1; ô Line trả cell state 1/result 1 và cờ vẫn bằng 0. Bước ray từ Line tới thùng
được chấp nhận; thùng trả state 2/result 0, cờ đổi từ 1 xuống 0 sau callback. Vì thế state 2/result 0 chỉ nói
tia dừng tại ô đó trong phép đo, không tự phân biệt thùng phá được với static. Run này không đo trực tiếp
nhân vật đi vào Line01 hoặc getter di chuyển của nó. Fire cung cấp power 2 là dữ liệu test design.

Consumer tĩnh của lớp này phân biệt các nhóm byte trong TileInfo: `BA0860` đọc byte `24 + direction`;
`BA0CE0` đọc `28 + direction`; `BA0E30` đọc `32 + direction`. `BA0D70` và `BA0EC0` đổi sang hướng đối diện
trước khi gọi hai getter sau. Tia nổ đã biết đi qua `BA0F80`/`BA0FC0` tới nhóm 32–35; nhóm này đều bằng 1
ở Herb và Line02 đã đo. Nhóm 24–27 bằng 1 ở Herb và 0 ở Line02; nhóm 28–31 của Line02 là `1,0,1,0`.
Không dùng dword offset 24 làm footprint hoặc một scalar chung cho type 3. Ý nghĩa di chuyển của từng getter
đã có đối chứng native return giới hạn: khi A gửi Right về Line02 còn nguyên `(0,7)`, `BA0860` trả 0 ở epilogue
cho direction 0/2 tại A và direction 0 tại B, khớp byte 24–27. Ba lần input không vượt khỏi ô `(0,6)`;
Herb01 đi vào khi còn nguyên trước bom đầu. [Bằng chứng đối chứng](../evidence/rust-service/g1-camp08-footprint-predicate.json)
giữ raw, sáu bom power 2 hoàn tất ở hai view và phạm vi riêng của từng phép đo.

Các lượt đối chứng tiếp bị lỗi trước khi tới negative Line hoặc trước khi cài probe direct. Giá trị wrapper dùng
`FinishBreakpoint` lệch với body tĩnh không được dùng làm contract. Raw và nguyên nhân chưa rõ được giữ nguyên;
không lặp route lỗi nếu chưa có điều kiện mới. Lượt đạt dùng DB thử nghiệm trên tmpfs sau khi log hai lượt trước ghi
`Backend(QueueExpired)` trước return/reset và lỗi observer đọc actor cũ. DB đóng rồi export đúng bytes và unmount;
đây là đối chứng cô lập storage, không chứng minh nguyên nhân sâu hoặc durability/M8. Ray MissileBase/PinPillar có bằng chứng riêng bên dưới;
Line01 có đối chứng ray riêng bên trên; hướng khác của Line02, coverage roster và item còn lại vẫn mở.
Không mở preset Camp08 từ lát này.

Ba object type 1 đã có [đối chứng initializer/footprint](../evidence/rust-service/g1-camp08-footprint-predicate.json)
ở hai view: PinPillar03 `(1,11)`, PinPillar04 `(11,3)` và MissileBase02 `(12,10)` đều là `CGOTileStatic`,
virtual initializer `BA1350`. Object giữ nguyên dữ liệu TileInfo 72 byte; hai getter `7E7350`/`7E72B0` trả width/height.
Type 0/1 đọc dword offset 24/28; type 2/3/10 trả 1, type 4 trả 0. Không lấy counter type 2 hoặc nhóm byte type 3
làm kích thước. Predicate `9B0640` bật cho type 1/2, hoặc type 0 có dword offset 44 khác 0.
Loop khởi tạo mở rộng từ `row-height+1..row`, `column..column+width-1`, rồi gọi `EA4D90` và `EA7C20` với value 1
cho type 1. Native ghi đúng các call này: hai PinPillar 1×1; MissileBase 2×1 phủ `(12,10)` và `(12,11)`.
Cờ ban đầu tại các ô này bằng 1, các ô đối chứng cạnh chúng vẫn bằng 0. Đây là contract khởi tạo/footprint;
không suy vật lý nhân vật, phân phối item, reset hoặc map tổng quát từ initializer.

[Ray tĩnh và ô phủ](../evidence/rust-service/g1-camp08-static-rays.json) đã có hai route riêng. Từ trái, PinPillar04
`(11,3)` và anchor MissileBase `(12,10)` trả cell state 2/result 0 ở hai view. Tia power 4 đi qua Line02 `(12,7)`
và Herb `(12,9)` rồi dừng tại MissileBase. Từ phải/trên, tám account dùng spawn gốc: H kiểm PinPillar03 `(1,11)`,
E nhặt Fire cấu hình rồi kiểm covered-cell MissileBase `(12,11)`. Cả tám view có đủ bốn consumer 159/158;
cell state 2/result 0 chặn tại hai ô, không đi tiếp từ ô phủ sang anchor `(12,10)`. Cờ của cả bốn ô static giữ 1
trước/sau mọi blast trong hai route. Đây là đối chứng cho hard-stop kind 1 hiện có, không phải loại ô mới của Rust.

Một lượt đầu timeout ở Wine preboot; lượt khác có `log flush timeout at event 287` trước bom đầu dù DB đã trên
tmpfs. Route đạt dùng output tmpfs theo phương pháp đã có: cleanup, export so từng size/SHA, unmount rồi seal.
Lượt tám người đầu hoàn tất native nhưng UDP capture kế thừa allowlist hẹp bị lỗi, nên overall exit 1 và giữ riêng.
Lượt sửa capture dùng nguyên bản tám người đã nghiệm thu đạt exit 0. Các đối chứng storage/capture không là
durability, sửa production hoặc M8; không thay timeout. Line01 và route serving tám account có đối chứng riêng bên trên; coverage
item/roster còn lại vẫn theo PLAN. Không coi bốn bom có tám recipient là nghiệm thu toàn gameplay Camp08 tám người.


<a id="rust-factory07-terrain"></a>
## Factory07: khởi tạo và bốn loại thùng

[Bằng chứng Factory07](../evidence/rust-service/g1-factory07-terrain.json) tách adapter/basic khỏi terrain.
Native hai account trên Wine với terrain Rust tắt đã đối chiếu đủ 195 ô: 1D54 có 162 ô bằng 1, gồm 53 static
và 109 thùng theo profile; 1D24 có toàn số 0. Tên/code 281, hai spawn đầu trong tám spawn gốc và dữ liệu pack
được kiểm riêng. Cờ khởi tạo không tự chứng minh khả năng đi qua, độ bền thùng hoặc phân phối item.

Sáu placement power 1 tại `(1,0)`, `(2,0)`, `(2,1)`, `(2,2)`, `(2,3)`, `(2,4)` đã có actual C2S118 và
consumer 159/158 hoàn tất trong cả hai view. Toàn bộ 195 ô trước/sau từng callback khớp chín thùng mất cờ,
đưa tổng 1D54 từ 162 xuống 153; 1D24 không đổi. Các sample có BoxBlue, BoxSky, PaperBox00 và Safety00.
Đây là bằng chứng phá ở hit đầu trong các ca đã đo, không phải suy từ tên hoặc dword 24 bằng 0.

Blue/Sky ở sample là `CGOTileFixedBox`. Probe `B9C849` thấy dữ liệu nguồn và counter C4 bằng 0 trước nhánh
default của initializer; trước hit counter bằng 1, sau hit bằng 0 và return bằng 1. Body `B9C760` đã cache
giải thích nhánh đổi 0 thành 1 sau vị trí probe. Generic lookup ở nhánh box thứ hai trả null sau mutation;
không coi null đó là thiếu bằng chứng về thùng đã bị phá.

Paper/Safety ở sample là `CGOTileMoveAbleBox`, với virtual `+64/+68/+74` lần lượt là `B33250/B33270/B334C0`.
Probe C4 của FixedBox không áp dụng cho lớp này; lượt control không đo được counter đó và không nhập giá trị 0
thay unknown. Verifier frozen đã thất bại do đồng nhất thời điểm counter và lớp; bản kiểm quan sát riêng giữ
nguyên mọi predicate TCP/195 ô, ghi rõ phần counter còn thiếu. RTTI chưa chứng minh thao tác đẩy thùng đã chạy.

Route Rust hai account dùng kernel terrain đã có, cùng payload map/fixture binding và hit count mặc định 1;
không sửa source hoặc binary để mở map. Bomb 701 ẩn tại `(2,0)`, Fire 700 tại `(1,4)` là vị trí **design**.
Native 142 chỉ reveal sau current clearance đã commit, actual 124/165 tiêu thụ item, rồi A có bom power 2.
Mỗi vòng chạy lại sáu root power 1 đã đo, thêm root `(1,0)` power 2 và B tự đặt bom ở spawn `(0,14)`.
Cả hai view giữ static `(1,1)` chặn nhân vật và ray; không suy mọi hướng hoặc hành vi đẩy từ ca âm này.

Probe actual MoveAbleBox đã dùng offset D8 và các site cache `B9F568/B9FDD0/BA004F`, không dùng C4 của
FixedBox. Paper/Safety có raw source counter 0 trước default, counter 1 trước hit, sau hit 0/return 1 trong
cả hai vòng. Toàn bộ 195 cờ của hai lớp khớp actual input ở mọi callback; hai rank/return/reset và clock thứ ba
khôi phục thùng, counter, spawn và stat. DB journal có hai trận complete đúng map, thưởng 0; Start cuối được
đóng cancelled khi cleanup. Đây là route hai người, chưa nghiệm thu Factory07 cho roster 2–8 hoặc mọi nội dung.

Lượt integration đầu dừng có khai báo sau một vòng vì quickclick Ready không tạo request 95 mới. Ảnh vẫn ở phòng
chờ; heartbeat mang stage Lobby không chứng minh client đã rời phòng. Lượt kế dùng helper settle/hold đã có,
hoàn tất hai vòng và giữ exit 0. Lượt kế thiếu file note review trước freeze do lỗi chuẩn bị; note sau launch
ghi rõ giới hạn quy trình, không được gọi là prelaunch review. Raw/input frozen và verifier thất bại giữ nguyên;
không suy Internet, tải, độ bền lưu dữ liệu hoặc M8 từ output/DB tmpfs chẩn đoán.

[Route bốn/tám account và các control static](../evidence/rust-service/g1-factory07-roster-routes.json)
dùng cùng kernel, spawn gốc và hai đội. Mọi recipient có cờ terrain khớp actual placement ở mỗi callback,
hai kết quả/reset và Start kế. Native 165 tìm thấy đúng item, đọc đủ payload và thay đổi stat của A:
Bomb 2→3, Fire 1→2; sáu stat của actor khác giữ nguyên. Clock kế khôi phục stat và thùng; journal đầy đủ
player, đúng Factory07 và thưởng 0. Report frozen ghi nhầm trường `native_items` thành danh sách counter
do dùng lại biến sau các predicate item. Report/verifier cũ giữ nguyên; bản kiểm bổ sung từ sealed raw
lưu đúng records item và kiểm lại toàn bộ tuple, reader, item lookup và secure stat, không chạy native lại.

Ray tại các sample Block00/01/02 có getter 1D54 trả 1 và cell trả trạng thái 2/result 0 trong tám view qua
hai vòng. Glass00 được kiểm riêng bằng slot 2 từ spawn `(12,1)`: actor tới `(12,2)`, thử đi lên Glass `(11,2)`
rồi quay lại. Fire 702 hiện trên nền tại `(12,2)` là design, native 165 tăng power 1→2 ở slot 2 trong cả bốn view.
Bom power 2 tại `(12,2)` dừng ở Glass; Blue `(10,2)` phía sau còn nguyên, Blue `(12,3)` bên cạnh vỡ.
Đủ 195 cờ của hai lớp được kiểm trước/sau, terrain tắt ở đối chứng rồi bật trong lượt tích hợp Rust.
Đây là một hướng/anchor Glass, không phải nghiệm thu thao tác đẩy hoặc mọi hướng trên map.

Lượt tích hợp cài đúng probe initializer đã có nguồn trước load map. Bốn sample Glass00/Block00/01/02
đều là `CGOTileStatic`, kích thước 1×1; 72 byte tại initializer và qua context AC sau init khớp nguồn pack.
Không nhập offset AC này sang Ground hoặc MoveAbleBox. Lượt đối chứng trước đó thiếu lệnh cài probe nên
không có initializer records; verifier frozen thất bại được giữ, bản quan sát chỉ xác minh movement/ray/stat.
Fixture Fire hiện đầu tiên cũng được sửa đúng thứ tự list trước freeze; file ứng viên lỗi chưa được chạy và giữ nguyên.

Lượt Glass Rust giữ wrapper exit 0 nhưng có lỗi K2/GDB ở B và lỗi fatal GDB ở C. Trong từng log, trước lỗi
có peer event mang timestamp lớn hơn controller completion, nên xác minh riêng các quan sát đã hoàn tất
trước đuôi lỗi. Verifier tổng và bản sửa chỉ bao phủ B đều được giữ; bản quan sát tiếp kiểm đủ hai role.
Không suy root cause, debugger shutdown sạch, phục hồi hoặc M8 từ những quan sát này.

<a id="rust-tomb12-boundary"></a>
## Tomb12: hai lớp cờ và route phục vụ

[Bằng chứng khởi tạo](../evidence/rust-service/g1-tomb12-initial.json) giữ hai account KR trên Wine, hai spawn
đầu trong sáu spawn gốc và hai view có cùng 195 cờ: 65 ô bằng 1, 130 ô bằng 0. Source type 1/type 2 tạo mask
ứng viên khớp toàn bộ cờ khởi tạo, nhưng cờ 1 chưa phân biệt static/thùng và cờ 0 không chứng minh đi qua tự do.

Tại các sample đã chọn, Stone00, WaterwayH04–H08 và bridge00 đều là `CGOTileGround`, virtual initializer
`B9D8E0`. TileInfo 72 byte tại callsite khớp pack, hai getter truyền kích thước Stone 1×1, Waterway/bridge 2×1.
Hai sample có cả mã ground/object bằng -1 cũng tạo object `CGOTileGround`; chưa suy hành vi của toàn bộ 52 ô
đó. Sáu GraveStone01/03 là `CGOTileStatic` với source data, footprint 1×1 và call ghi cờ đã quan sát.

Probe ban đầu đọc `+AC` theo lớp type 1/3, nhưng trường này bằng 0 ở các Ground đã chọn. Không gọi nó là con trỏ
TileInfo của Ground hoặc xác nhận dữ liệu đã được chép tới offset đó. Nơi lưu TileInfo của Ground
vẫn unknown; không cần suy offset đó để dùng lớp chặn tia đã đo riêng bên dưới. Native di chuyển, drop/pickup
và hai vòng/reset cho toàn phạm vi 2–6 người vẫn cần acceptance theo ma trận route. Chưa mở map tổng quát từ bằng chứng khởi tạo.

Run gốc trả exit 1 vì output tmpfs unmount busy sau khi export đã được so từng size/SHA; seal thường không tìm
thấy `cycles/run.json`. Raw giữ tại locator export, đã được xác minh riêng cùng input frozen và không còn PID
trong namespace gốc. DB và host bind đã unmount trong run; không chứng nhận output unmount thành công.
Giữ nguyên lỗi wrapper, verifier frozen và giới hạn chẩn đoán; không suy độ bền hoặc M8 từ bản export này.


[Bằng chứng tia và lớp chặn](../evidence/rust-service/g1-tomb12-ray-barriers.json) phân biệt **map+1D54** với
**map+1D24**. Trong hai ca power 3, WaterwayH06 ở anchor `(11,1)` và H07 ở ô phủ `(0,13)` có cờ 1D54 bằng 0,
nhưng `EA1230` trả 1; `E9F220` dừng ở nhánh `next_set1230` trước khi xử lý ô. Moss sau footprint còn nguyên.
Giả thuyết tia đi xuyên chỉ từ mask 1D54 bị bác; verifier theo giả thuyết đó và raw được giữ nguyên.

Nguồn tĩnh được đối chiếu bytes với client thực: `EA7B90` ghi wrapper secure trong container map+1D24 qua
`DEA970` và `55FD30`; `529C60` đọc qua decoder đã biết. Ground type 0 có dword tại offset 44 khác 0 được
initializer `EA1EE0` chọn qua `9B0640`, rồi ghi footprint từ `row-height+1..row`, `column..column+width-1`.
Đây là bằng chứng tĩnh; bằng chứng thực thi riêng có đủ 96 call từ return site `EA31AD` và giá trị sau ghi bằng 1
ở mỗi view. Toàn bộ 195 wrapper được đọc và kiểm checksum lúc clock và trước/sau hai callback nổ, khớp mask
96 ô bật từ Waterway. Các ô bridge trong mask này bằng 0; điều đó chưa chứng minh di chuyển qua bridge.

Đối chứng Stone có root `(8,0)`, power 3: tia xử lý Moss `(7,0)` và cờ 1D54 đổi 1→0; hướng qua Waterway
`(8,1)` dừng trước ô. Root `(0,14)` có ca âm ở Waterway `(0,13)`. Cả hai view giữ nguyên mask 1D24 trong
khi 1D54 chỉ mất cờ của đúng thùng Stone. Fire dùng để cấp power là input thiết kế có native pickup, không phải
phân phối rơi gốc. Đây là hai ca chẩn đoán, chưa survival, kết quả/reset hoặc gameplay toàn map.

Rust nhận `terrain.ray_barriers` tùy chọn, tách khỏi `kinds` và `box_hits`: thiếu hoặc rỗng tương đương 195 số 0;
nếu có thì phải đủ 195 giá trị 0/1 và dùng cùng binding map/fixture hiện hành. Tia dừng trước ô có mask, không
đánh thùng hoặc kích chain phía sau; origin có mask thì không phát tia. Mask giữ nguyên khi reset, còn kinds/hit
count được khởi tạo lại. Đây là policy projection có giới hạn, không tự đổi movement hay guard đặt bom.
Binary Rust mới đã chạy cùng hai ca Stone/Waterway với mask bật trong cấu hình riêng, hai view có đúng thay đổi
1D54 và mask 1D24 bất biến. Đây là nghiệm thu tích hợp hữu hạn; chưa đọc trực tiếp shadow clearance từ service,
chưa nghiệm thu chain hoặc bom đặt tại origin có mask trên Tomb12. Reset có bằng chứng riêng trong route
phục vụ bên dưới. Unit test chỉ kiểm hồi quy policy, không thay bằng chứng native.


[Ma trận route phục vụ](../evidence/rust-service/g1-tomb12-serving-rosters.json) đã xác minh route roster 2/4/6 account thực qua
hai vòng/reset và Start kế. A mở liên tục hành lang Stone cột 0, nhặt Fire/Bomb ẩn, đi qua anchor và ô phủ bridge
`(1,1)/(1,2)` trong cả hai view, rồi nổ power 2 qua hai ô đó và làm vỡ thùng `(0,0)`. Các lần thử Waterway từ
`(11,0)` và `(0,14)` bằng phím thường không đưa actor sang ô kề. Từng snapshot cả hai lớp cờ khớp input native;
clock kế khôi phục thùng và sáu stat theo nhân vật. DB ghi hai trận hoàn tất, trận kế hủy khi đóng client, thưởng 0.
Đây là route contiguous 2/4/6 đã đo; chưa mọi ô/hướng, sparse roster hoặc toàn bộ item/G1. Lượt sáu người giữ
overall exit 1: UDP capture dừng trước clock đầu do allowlist bốn máy, không có trace UDP gameplay; GDB B lỗi sau controller completion, output
unmount busy sau export đúng bytes. Các quan sát TCP/native, hai lớp cờ, bridge và DB được xác minh riêng; không
coi đó là capture UDP đầy đủ hay debugger/output cleanup sạch. Không chạy lại chỉ để biến exit 1 thành 0.

<a id="f1a-adapter-expansion"></a>
## F1-a: mở rộng adapter tile đã đo sang 163/214 map (#559)

Từ nền [F1-a ban đầu](../evidence/rust-service/f1-pack-map-adapters.json) (155/214 map), `tools/kr_pack_export.py::f1_terrain_rules`
mở rộng nhận thêm các biến thể cùng cấu trúc của các tile đã đo trong KR 00967: `Tile16_Static/PinPillar01..05` và
`Tile62_Static/PinPillar{,_down,_left,_right,_up}` (type 1 giữ nguyên cờ hướng đã pin trong bảng `_TYPE1_PIN_FLAGS`),
`Tile62_Passable/PoliceLine01..04` (type 3 cùng ngữ nghĩa passable shadow như `Tile16_Passble/Line*`), và
`Tile10_Passable/BasicBlock_pass00..01` (type 3 có ba nhóm pass-all, tách các dword render/cache đuôi ngoài `EA1EE0`).
Các cơ chế chưa có hợp đồng (băng chuyền `TileConv/*`, đường ray `Tile08_Ground/Rail00`, cờ `Tile07_Ground/*` / `Tile42_*`,
và 4 map lỗi parser đuôi payload) tiếp tục bị từ chối rõ ràng.

Tám map vừa được mở (`CaJJ_RetroCamp08`, `Camp04`, `Camp05`, `Camp12`, `Camp15`, `Camp17`, `Cemetery01`, `Cemetery19`)
đều chạy đạt qua `tools/f1_pack_matrix.py` hai client TLS/relay `kr-load --scope f1-a` (`8/8 passed`), nâng tổng số
map F1-a hợp lệ lên **163/214** (51 map còn từ chối adapter). [Biên nhận và SHA-256](../evidence/rust-service/f1a-adapter-expansion.json).
Đây là bằng chứng tổng hợp (`kr_static + emu_choice`); không tạo checkpoint native và không tự nâng phân loại họ map.
