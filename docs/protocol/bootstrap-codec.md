# Bootstrap, WCP và codec

[Mục lục](../PROTOCOL.md) · [Câu hỏi đang mở](../RESEARCH.md) · [Thứ tự việc](../PLAN.md#dashboard).

<a id="bootstrap-recipe-scope"></a>
### Phạm vi recipe bootstrap đã đối chiếu — CP179

`kr_runtime/confirmed` trong **recipe offline R2**: ba run lịch sử cùng exe7b3b435…/harnessa1e7abe…/
Wine10.0/reply75be47…/hostmap/listeners, mỗi process/game/prefix riêng, `init_peb` PID01a0/01ac/0174
gắn hai nativeconnect loopback cùng thứ tự endpoint với listener. Reply15byte chỉ trên socket1;
socket2 có attempt sau reply trong10s đầu từngrun, zeroC2S. Native nonblocking status `c00000a3`
không là connect đồng bộ thành công; Wine timestamp và Python monotonic không tự đồng nhất clock.
Observation15/10/10s giữ nguyên; từ templateprefix đã init, không ba Wineinstall hoặc coldallcache.

`kr_static/confirmed` về bytes: R2 và e9 hiện dùng khớp vùng VA1234000..1239000 và14ED000..14F0000
(end exclusive, base400000/RVA=VA−base). R1→R2 vùng đầu chỉ khác5byte NGinit omission12358E7;
framer/fields vùng sau không đổi. R1 nativeProcessPatchreturn1 và silent/badguard vẫn là control
riêng; không gán chúng thành3coldstarts. E9 dài thêm1024byte, khác644byte/85range trong phần chung,
có thêm patch import/state/NMCO/heap/overlay/identity; equal-region không chứng minh whole-program
tương đương, quan hệ nhân quả từngpatch hoặc cold3e9. Candidate import names giữ candidate.

Nguồn reuse: [Wine R2 và CP179](../evidence/wine-r2/README.md#cp179), [bootstrap controls](../evidence/bootstrap-wire/README.md),
[e9/CP020](../evidence/start-reply/README.md), [CP014](../evidence/login-runtime/README.md), [CP015](../evidence/postwait/README.md).
Không cần RE lại primitive/opcode/VNlayout hoặc chạy lại cold3R2/gameplay. `design/confirmed` về
quyết định nghiệm thu dự án chỉ ở [PLAN](../PLAN.md#k1-acceptance); full K2/live/reset/auth/cache vẫn mở.
Windows nguyên bản CP178 có ownership/control; không có bootstrap pass hoặc exactPCcause mới.

<a id="wcp"></a>
### WCP trên kết nối sau ProcessPatch

Nguồn chuẩn cho contract hẹp này; raw/recipe/run/review snapshot ở [evidence WCP](../evidence/wcp/README.md).
Target modO base400000 SHA a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5;
runtime ban đầu trên copy Wine R2 SHA7b3b435b5d8e110185359107376b736615ceabe9452cea8154d02c8628f44c29;
các copy chẩn đoán CP006–012 có SHA/patch riêng trong manifest, không tương đương R2 nguyên recipe.
Trạng thái review của tổng hợp ở [CP001](../evidence/checkpoints/manifest.json); confirmed nói về loại/phạm vi phép đo, không tự nói đã qua gate.

| Claim | Nguồn/phương pháp | Trạng thái và giới hạn |
|---|---|---|
| Socket115B840 qua13AED40;13AD6F0 đặt initial mode/selector0/0 từ1A7C684/688,response14ED440,poll13AD820 | kr_static;asm/call chain | confirmed route; không suy cả game có cùng codec |
| Mode0/selector0:prefix1+BE16 length+payload; guard u8 phải0,tiếp2×u16,u32,string cóu16 byte-length,u8 flag | kr_static+emulation14EF300/13AEFE0 | confirmed nhánh flag khác1; ý nghĩa2×u16 chưa biết, không áp version967 của ProcessPatch lên WCP |
| Nhánh flag khác1 áp mode2/selector3 qua globals1A7C694/698;u32 chuyển tới socket virtual24 | kr_static;native oracle u32=0 | confirmed thứ tự gọi; vai trò seed candidate. Oracle+40/+44/+48=[2,3,0];+48=0 chưa chứng minh lưu seed (input và nền đều0);không phải measurement fields live |
| Poll/consumer chấp nhận fixture ở socket hai | kr_runtime;debugger13AD782EAX1 và13AD79CEAX1 | confirmed đúng run debug trước CP006; chưa full hello/codec nghiệm thu; C→S mới xem CP006 |
| Sau consumer:WSAAsyncSelect(socket,hWnd,msg400,mask21) thành công | kr_static+kr_runtime Wine trace | confirmed notification READ/CLOSE; không chứng minh toàn client chỉ chờ server |
| Caller run thực:1255AC0→1255A30→1234E70→1231BD0;literal ConnectToMultiServerSingleTime | kr_static đối chiếu backtrace runtime | confirmed route; vai trò multi-server candidate,login/world chưa xác nhận |
| Sau connect 1254D50 trả error từ global 1A3398C; 1234E70 gọi 12364A0 nếu khác0 | kr_static + kr_runtime Wine R2, CP004 | Trong run R2 error1/msg1958 có sẵn khi debug bắt đầu và sau consumer; dừng tại 12364A0, chưa quan sát guard12349E0. Giá trị dump không chứng minh lỗi vanilla. |
| Trong run copy thử với hai u32 error bằng0, quan sát 1232385 và early exit qua 1258150 | kr_runtime + kr_static CP004 | Thử nghiệm riêng; code ghi error9/msg0x9a1, launcher khởi động, tiếp đó UD2; không đạt login. Nhánh guard12349E0 chưa quan sát tới. |

CP006, review theo [manifest](../evidence/checkpoints/manifest.json): trên copy reset error và
`multiserveridx:0`, debug/plain run mỗi run nhận 66 byte C→S trên socket thứ hai, SHA và recipe ở
[evidence](../evidence/wcp-sequence/README.md). `1238530` tạo packet type0 và gửi qua `1164D50`;
consumer tiếp theo chờ opcode13. Tám response tổng hợp opcode `13,2,12,21,1,43,62,719`
đã được consumer Wine xử lý đủ để guard `12349E0` trả1 trong một run debug; một số payload rỗng
là `emu_choice`, không chứng minh tài nguyên/server gốc. Mode2/3 với header36 byte, XOR và CRC8
được kiểm hẹp bằng native oracle cùng negative checksum/partial và runtime; không suy full codec,
reconnect hay K2. Import NMCO identity chưa bind trên copy CP006; thử nghiệm tiếp
CP007–009 ở [evidence identity](../evidence/nmco-identity/README.md), còn blocker/kiểm hiện hành
ở [Q-023](../RESEARCH.md#q-023). Tại CP006 chưa có login/gameplay; các mốc sau có phạm vi riêng bên dưới.

CP007 dùng identity `1` và screen name rỗng như **emu_choice**, không bổ sung contract wire hay
chứng minh identity server gốc. Các run chỉ chứng minh phạm vi đường chạy trên copy đã sửa;
ngữ nghĩa login và nội dung khởi tạo vẫn unknown.

CP010 có log tạo packet type15 sau marker khởi tạo trên copy; capture stub vẫn chỉ có 66 byte
C→S trước đó. Chưa có wire/layout hoặc vai trò packet15 được khôi phục từ quan sát này.
CP011 vẽ UI login nhưng hai phép input chưa tạo thêm C→S; điều này chưa bổ sung contract auth.

CP012 (`kr_runtime/confirmed`, copy có patch): command UI1002 và luồng tự động trước click đều
được tách bằng breakpoint/timeline. Resolver login yêu cầu `dream-login-ca.nexon.com`, thấy port3838/3848;
NSS riêng ánh xạ về loopback cho kết nối thứ ba. Trên run cấp cùng fixture hello cho ordinal3,
đã đo `13AD782`/`13AD79C` trả1 sau guard. Trước reset global, tổng C→S chỉ66 byte của connection2. Hai run reset global1A966FC trong bộ nhớ
thu thêm4 frame×69 byte C→S15 trên connection3. Native parser14EF300 replay bốn capture với
seq40,43,46,49 khớp CRC/payload; CRC-sai và partial bị từ chối. Header37+payload31+CRC1 đúng cho
bốn mẫu mode2/selector3 đang thử; synthetic receiver tắt prefix check, không phải server C→S.
Semantics auth/keepalive và field payload chưa xác nhận.
CP013 bổ sung đường **S→C** bằng static dispatcher KR: `10FCBE0` ánh xạ4→`10FEDA0` (result0→`10FD540`)
và15→`1100590`; hai handler nối `stage+2C04` tới wait state`+F4` qua`C47260/C472A0`, tick`C472E0`
hoàn tất1→3. [Bằng chứng đối chiếu VN](../evidence/vn-reuse/README.md) giữ bytes, guard và giới hạn.
Tại CP013 chưa có runtime S→C4/15 hoặc C→S auth KR; CP014 bổ sung ở [contract login](auth-session.md#login-auth).
Full success schema vẫn thiếu; CP014 có thử S→C15 bên dưới. Không gộp S→C15 này với
bốn C→S15 đã capture ở CP012. Đây không phải toàn bộ endpoint/port,
full codec hoặc proof auth/reconnect K2.
Call/capture và giới hạn tại [evidence handler](../evidence/ui-handler/README.md);
Contract username/password, token thật và mode/seq tổng quát vẫn unknown.

### Giới hạn và correction WCP lịch sử

Fixture opaque/zero/empty và prefix69 là emu_choice đã được consumer chấp nhận trong phép thử,
không phải byte server gốc. Nhánh flag1,codec frame mode2/selector3 ở phạm vi fixture CP006 có kiểm hẹp; seq/reconnect, semantics đầy đủ và
message400 handler vẫn unknown.

Correction CP001: WCP r1 raw mô tả sai mapping fields và suy caller1255350. Giữ raw nguyên:
đúng là socket+40=mode2,+44=selector3,+48=0 theo oracle; route run này là1255AC0.
1234E70 return1 chỉ cho biết1254D50()==0 ở lần đọc đó,có thể cả khi1255A30 trả0;không dùng
return1 của wrapper này thay proof consumer WCP.1255350 là đường tĩnh khác,chưa quan sát trong run này.
Absence WCP timeout không phải runtime proof;
dùng breakpoint đã nêu. Câu hỏi/thí nghiệm mới chỉ giữ ở [RESEARCH](../RESEARCH.md#q-023).

### Helper K2a biệt lập (lịch sử); oracle/stream phục hồi sau đó

[Emulation writer helpers](../evidence/primitives/README.md) đã xác nhận plainBE16 `14A5B90`,
XORBE16 `14A5C90` (`0xA569`) và byteXOR `14A5C80` (`0x5A`) trong image unpack có hash đã khóa.
Các helper đạt hết miền input nhỏ và kiểm byte guard/return. Đây là snapshot helper banđầu;
[WCP/profile0/0→2/3](#wcp), [actualframe/stream/CRC/seq/resetmodel CP057](#codec-stream-boundaries)
và capturednativeframes phía sau có mức chứng minh riêng. Không hiểu giới hạn helper là thiếu toànbộ
framinghiệnnay; fullmodes/prefix/live reconnect/bounds vẫn mở. GateK2profile đã đạtCP181 theo PLAN, không toàngrammar.
Review receiver ownership cũng sửa được cách gọi send guard: `13ADD10` dùng receiver từ `545E30`,
packet nằm trên stack, nên không truy nhầm setter `packet+3C` để tìm socket/framer.

<a id="codec-stream-boundaries"></a>
### Codec incremental trên actual frame — CP057

**kr_static/confirmed qua emulation**, [source/hash/recipe/model](../evidence/codec-boundaries/README.md).
Tái dùng cached `14EF300`/RVA10EF300 và oracle CP012–056, rawmodO base400000; không thêm decompile
hoặc chạy malformed input trên client Wine. VN chỉ hướng cách kiểm socket/seq/retain, không port header.
Actual A channel capture CP056 `state-guard`, hai frame liên tiếp đầu6/67, seq40/43, frame đầu74byte.

- 76 scenario native: chia frame đầu ở mọi vị trí0..74 và coalesce cả hai. Partial trả2, giữ nguyên
 bytes/seq; đủ frame trả0, tăng3. Coalesced native memmove giữ **đúng byte frame thứ hai**, call tiếp
 seq43→46. Payload native khớp decoded body của actual frames; không dùng encoder roundtrip làm oracle.
- 75 scenario `ProbeService.receive`/socketpair: cùng actual bytes qua chia/coalesce; mỗi logical
 connection riêng40→43→46, buffer hết, dispatch recorder nhận đúng frame/body. Đây là OS socketpair
 bench với handler recorder, **không** mới Wine runtime/fragmented liveTCP/auth integration.
- 14 boundary case native hoàn tất: CRC sai hoặc seq sai trả7, không consume/tăng seq; partial length
 giữ buffer/seq. Cap native được **cấp tường minh trong model**, exact framesize pass/onebelow trả5;
 chưa phục hồi cấu hình cap thật. 64KiB của service là `design`, kiểm length trước chờ/allocate body.
 Native có case headerlength lớn trả2/7 trong model nên không sao chép nguyên giới hạn/overflow sang server.
- HeaderlengthFFFFFFFF dừng tại allocation guard của oracle cũ (>1000hex buffer); **unknown** phần native
 sau đó. Không chạy lại bằng cấp allocation lớn, không gọi đây client crash hoặc kết quả full bounds.
- 4 fixture đổi sequence/reencrypt từ actual plaintext được native decoder kiểm u32+3 wrap ở FFFFFFFE/
 FFFFFFFF/0/3; đó là fixture biến đổi, không nativewriter capture.1 scenario synthetic fresh→warm replay→
 fresh reset: cùng frame40 pass→CRC7 ở seq43→pass sau reset socket40; chưa live logout/reconnect/cache.

Service pop tách frame rồi frame_body kiểmCRC; native lỗiCRC giữ buffer. Production run policy đóng
connection lỗi đó (connection khác giữ riêng), không là native resync/recovery. Không đổi code codec/service
ở CP057. Prefix guard socket+AC vẫn tắt trong model như oracle trước; mode khác, nguyên bảnWindows,
full reset/reconnect/header-auth và string/blob boundary theo consumer còn mở, **K2 chưa nghiệm thu**.

<a id="live-codec-stream"></a>
### Codec trên socket Wine thật: split, coalesced và CRC — CP180

**`kr_runtime/confirmed` trên copy e9bb476…**, một client, Wine10.0, namespace chỉlo/noIPv4routes/
NSSfiles-only, game/prefix/cache kế thừa; không Windowsvanilla hoặc coldallcache. Tái dùng CP057
nativeframer/methodology VN, CP058auth137 và current47fixture serviceprofile; chỉ đổi write scheduling
trong subclass đo. Production server/codec không đổi. [Recipe/raw/hash/audit](../evidence/live-codec-stream/README.md).

| Phép kiểm | Native observation / giới hạn |
|---|---|
| Auth175byte gửi13 rồi162 sau0,6s | `14EF300`: thực nhận13, ret2, available13/seq43 giữ; nhận đủ175 ret0/available0/seq46/op4/payload137; callback`10FD540..10FED80` cursor137/137/error54=0/error58=0. Có50pollpartial, không50case độc lập. |
| Netinfo+flags ghép110byte | Native15 tiêu thụ69, còn41, seq46→49; native62 tiêu thụ41, còn0, seq49→52. Đây là buffer thật/callback, không suy từ số write. |
| Ready+roomlist+channellist ghép191byte | Native241→91→14, available191→122→82→0, seq46→49→52→55. Kết quả ở copy/fixtures đã đo; không toàn opcodegrammar. |
| Control authCRC chỉ lật1bit cuối | Native ret7/packet+58=10, available175 và seq43 không đổi; không thấy authcallback hoặc TCP thứ4 trong run42s. CALog có `GetPacket Err=7`, CrcGen234/CrcPosted235 và C2S59; ý nghĩa59 ngoài path này chưa khôi phục. Không suy native tựresync hoặc cơ chế disconnect từ riêng ret7. |
| Redirect sang channel socket | Actualordinal3/4 có hai hello0/0: seq40→43 rồi mode2/selector3; channel17 bắt đầuseq43. Objectaddress có thể tái dùng, không là cùng TCPconnection. Chứng minh reset trên redirectpath, không retry sau lỗi/accountresume hoặc freshprocessreconnect. |
| Cấu hình socket thật | Các socket đo có prefixguard`+AC=0`, maxsize`+6C=1048576`. Đây là giá trị quan sát, không proof enforcement cap/overflow/FFFFFFFF hoặc prefixguard bật. Header37 gồm prefix1; CALog HeaderSize36 là phần còn lại. |

Locator base400000: framerVA14EF300/RVA10EF300; authbegin10FD540/end10FED80. Native GDB probes
đọc entry/return; giữ diagnosticglobalreset1A966FC=0 từ recipe cũ. Không épreturn/callinferiorfunction/
dispatch hoặc thêm binarypatch. Logger ghi fixed packetoffset cả mode0: các **packetfield ở mode0
không có nghĩa** do layoutkhác; chỉ dùng socketmode/seq/buffer/ret tại đó. Raw không sửa để xóa dữ liệu này.
Mode2typed packet/cursor đủ trong phạm vi trên; UI sảnh được nhìn nhưng chưa proof displayidentity/K3.
Các run67,719462273s/48,911328033s gồm setup/cleanup; observationnominal60s/42s. Hai run không overlap,
service stopped/errors0/probeerrors0/sourcepins unchanged và prefix-ownedcleanup có hash.

`design` cho fixtureidentity/token/emptylists và scheduling; không authauthority servergốc.
FullK2 vẫn mở về retry/reconnect sau lỗi, prefix/mode/bounds chưa đo; [RESEARCH](../RESEARCH.md#gap-socket-codec)
giữ phép kiểm, [PLAN](../PLAN.md#current-order) giữ duy nhất thứ tự. Không lặp76modelsplit hoặc ACgames.

<a id="codec-fatal-close-followup"></a>
### Đường đóng thông báo lỗi codec — controls CP181

`kr_runtime/confirmed` trong `k2-retry-reset-r1/close-route-r2`, copye9/Wine cùng CRCnegative:
sau thao tác đóng notice đã đo entry`1503DFD` với stackreturn`1254CED`, rồi `11044A7` trảEAX1/
global1A966FC=1; entrycleanup`1259EC0` cũng global1/caller125CA9F, sau đó SIGILL2260DF8 với
return1259F11. Không thấy callbackcommand1002 hoặc TCP thứ4 trong run này. Tái dùng locator/modal
CP012 và trapreport/cleanupCP010: slot16EA064 có tên **candidate** `dwk.dll!UninstallDWK`.
Đây là sequence observed trên copy, không tênAPI original đã được resolve hoặc mọiCRC đều cùngroute.

Thao tác đóng notice ở run này không chứng minh retry login; không sửa globalvề0/stubcleanup để
tự gọi reconnectthànhcông. `retry-r1` có cleanuptrap nhưng ít probe hơn giữ riêng; `close-route-r1`
hardwarewatchpoint làm WineDbg internalcrash14001D25A, không kết luận client từ ca đó. R2 bỏ HWwatchpoint
và dùng functionbreakpoints; source/config freeze, không newbinarypatch/forcedfunctionreturn.
Raw/hash/failedmeasurement ở [metadata đợt đang đo](../evidence/live-codec-stream/retry-in-progress.json),
[câu hỏi tiếp](../RESEARCH.md#gap-codec-retry-followup), thứ tự duy nhất PLAN. Chưa K2retry/accountresume/gate.

<a id="rust-foundation"></a>
### Rust foundation và native login-error/retry — CP231

[Code/cách chạy](../../rust/README.md), [recipe/raw/hash](../evidence/rust-service/README.md#cp231),
[metadata](../evidence/rust-service/foundation.json). Owner09/11 đã mở implementation; CP213planning-only là lịch sử.
Rust reimplement measured [codec/retry CP181](auth-session.md#login-retry-codec), không import server VN hoặc boom-rs.

**Implementation/policy `design`:** cùng repo hai crate `kr-wire`/`kr-service`, Rust1.99.0/lockfile/Linux.
Wire mode2/selector3/header37 opaque, unsignedtypedfields/crypt/CRC8, seq40+3n riêng mỗi receive connection;
decode lỗi không advance. Buffer/frame tối đa64KiB là diagnosticbound, không maximumKR. Hello15/WCPsecond317/
error1frame39 pin size/SHA trước bind; error opcode4/payload1/seq43 được Rust encode exact với native-testedfixture.
Diagnostic mapping hai transport đầu là WCP, các login sau trả result1; **không authentication/accountownership**.
Ancillary60/body20 và15/body31 sau rejection có trong CP181; Rust ignore không reply/budget64, không gán nghĩa.
Unknownstage/duplicate1/CRC/length lỗi failclosed. Chưa mode/header/prefix grammar tổng quát.

Tokio owned listener tasks/connection receiver, semaphore≤64, acceptqueue32/logqueue256/buffer64KiB,
readidle/write/log/shutdown deadlines; không spawn per-packet timer tasks hoặc unbounded sendqueue.
M1 chỉ ghi response nhỏ trực tiếp có writetimeout, chưa đo generalbackpressure/SLO; chưa cần actor-room/writerqueue
cho gameplay. Structuredlog có build/version/boot/connection/stageevent/seq/size/hashpinnedreply, không rawtoken/
credential/authbodydigest. Hashwrong/invalidconfig/bindconflict fail before readiness; SIGTERM/SIGINT drain/close.

**Kiểm khác loại, không thay nhau:**5wiretests dùng2capturedC2S6/544 và plaintext nativeemulation CP057 cố định,
every split/coalesce/CRC/incompleteEOF/bounds/unsigned examples; maxframe/wrap là designregression tự tạo,
không nativeKR oracle mới.14kernelTCP/process cases dùng capturedC2S1 làm input: split/exactreply, ancillary
keepopen, badCRC/no reply, oversize/headerbound, partialEOF/idledeadline, duplicate/coalescedstageclose,
quota/freshconnectionreset/SIGTERM/rebind/hashfail/bindconflict. Python codec chỉ đối chứng; đồng ý không proofcontract.

**`kr_runtime/confirmed` finalR3:** copye9/Wine10.0/inheritedglobalreset/privateuser-net-mount-PID/lo-only/nooutbound.
Cùng CA LinuxPID144 qua firstlogin→ordinary close-notice+confirm(command1002)→newlogin. Hai nativehello
mode0/selector0 seq40→43/15consumed; hai frame4/result1 mode2/selector3 seq43→46/payload1/ret0,
handler10FEDF1 ghi result1/global0 hai lần. PCAP108record/toànselectedstreams: exacthello/WCPsecond/error1,
typed actualC2S và Rustseq/op/len khớp, login connections3/4 đều resetC2Sseq40; giữ9.517034768s sau result2,
ảnh cuối normal credential-error notice. Mode0packetobject offsets trong retainedprobe sai layout, chỉ dùngsocketstate.
Hai lần đều từ chối login; **chưa authsuccess/channel/lobby/room/Battle/accountresume** trên Rust.

**Failure/correction bảo toàn:** R1 senterror nhưng đóngsocket ngayactual60/body20→native không có result1,
NoticeDisconnected10/global1/cleanup/SIGILL;53raw/FAIL, không gọi đã đọc lỗi. R2 giữ ancillary theoCP181,
native2result1/UIretry có nhưngcapturelo thu unrelatedlocaltraffic chạm12000record,63raw/partialcapture, không đủtracejoin.
R3 chỉ lọc capture theo portKR, cùng Rustbinary/inputprofile,54raw/complete. Tổng3run170raw/frozen/seal/ownedcleanup0.
R3 cuối raw có Targetdisconnected/framerentry và GDBinternal-error khi teardown; finalcontrollerobservations launcher
alive/CA-PID/UI giữ trước ownedWinecleanup; không healthydebugger-aftercleanup claim. Retainfirststrictverifierfailure/
scopedverifier/observerreview; không sửa raw. Servicebinary finalbuild SHA giữ đúng native-testedbinary.

**M0/M1 acceptance thủ công đạt scope trên**, không đổi K0–K7/fullgame/fullservice. M0archive exactPython784a0d8/
server-tools/privateCP181adapter/47fixturepins giữ rollback, támsourceWIP byte-exact/excluded; không clientbundle phát hành.
Tokio/serdeJSON/SHA-256 dependencies có lý do tương ứng I/O-config/log-inputpins, lockfile/toolchain đã kiểm.
[PLAN](../PLAN.md#current-order) tiếp M2localidentity→sảnh→fixedroom bằng contract đã có; Python serving/RE/reference giữ,
chỉ RE discrepancy chặn port. Quota/rate/fairness/DB/UDP/load/SLO/internet/production chưa nghiệm thu. Audit kếCP232.
**Internet thật chưa kiểm**.

<a id="original-windows-control"></a>
### Control original KR00967 trên Windows VM — CP178

[Recipe/raw/hash và phép đo](../evidence/peer-endpoints/README.md#cp178). `kr_static/confirmed`:
CA sourceSHAa525… có RT_MANIFEST/ID1/524bytes/SHAe388… yêu cầu `requireAdministrator`, đọc bằng
Windows LoadLibraryEx(DATAFILE|IMAGE_RESOURCE)/FindResource, không gọi entrypoint. REA fulltree trả
range không file-backed, không có partialsuccess; targeted native resource read giải đúng câu hỏi.
NewPatcher IDs1–3 không thấy manifest; không suy mọi resource/name/language đều vắng mặt.

`kr_runtime/confirmed` trong Windows10 x64 VM hiện hành: sourceCAa52545…/NewPatcherb59e4d…,
187file/1861590835bytes mỗi fullcopy, base/game trên CNTFS/DriveType3. Original source không sửa;
current5source/dump/DBhash khớp, source187tree kiểm trước/sau. Native run từ fixedC, không UNC;
share chỉ cấp script/evidence. CA-onlyPID1328, NewPatcher-only9656, BothNP11136/CA7876; ETWEvent1
khớp PID/parent/creationticks/image với controller, tokenhighS-1-16-12288/elevated1. Read-only token/
PEB32 query có cwd `C:\BoomKR-CP178\game\`, moduleAPI và kernelImageLoad khớp base/size theoPID;
CA base400000/size31842304. Không port VA/offset VN hoặc đọc sourceDB bằng worker.

CA có modal `??? ??` class#32770 và newCALog46bytes/InitBeforePatch→ProcessPatch; NewPatcher có
modal `Nexon`/embeddedIE. Both quan sát đồng thời hai PID/modal riêng. Childtext CA vẫn questionmarks;
ý nghĩa/rootcause/ACP/provider/launcherIPC và exactPC/return tại điểm chặn chưa đo. Không coi title, alive hoặc noTCPsample là
bootstrap/login. Control quan sát mỗi recipe một lần, không ba coldstart cùng recipe đạt K1.

`design/confirmed` cho isolation/harness: IPv4/6guestFORWARD DROP và DNSproxy guard giữ trong run;
native PowerShell negativeconnect chỉ tới owned host sentinel172.18.0.1:39177 timeout, hai actualSYN
mỗi bridgecapture, không sameflow trêneth0/không listeneraccept. PCAP có background/management,
ETW chỉprocess/image/thread; không fullnetworkAPI/registry/fileauthority. Kill chỉ ownPID/start/path/
hashguard; 0remainingowned/externalchild/newdriver, ETWstop0, sourcefrozen, exactfirewallrestore.
Không chứng minh toàn registry/driver state hoặc unobserved bareWindows tương đương.

NewPatcher-r1 preparation lỗi robocopylogoption, chưa launchclient; v3quote argument sửa lỗi và
normalize separator, giữ failedsource/raw. Before-restorediff của v2 có falsepositive nested paths,
không coi là nativeclientmutation. Analyzer ban đầu giả extra blankline CALog; raw đúng hai dòng
CRLF, parser marker/cursor không bị đổi; giữ các log lỗi. Không sửa binary/raw để pass.

Source diagnostic đã đo được giữ byte-identical ở [windows_control.ps1](../../tools/windows_control.ps1)
và [windows_console.py](../../tools/windows_console.py). Lease/capture/fullcopy và thư mục output mới
phải chuẩn bị theo recipe có hash trước khi gọi control; PS không tự cấu hình firewall.

[PLAN](../PLAN.md#k0-k2a-acceptance) giữ quyết định nghiệm thu K0/K2a do Codex đối chiếu thủ công/waiver;
verifier12314checks phần lớn packetintegrity không tự đóng gate. K2a tái dùng oracleKR cũ, không
runtimecodec mới ở Windows. K1/fullK2/K3–K7/fullmatch/LANInternet8clients tiếp mở; gameplayWineCP176
và replacementCP177 giữ nguồn riêng. Sourcefact/limits ở đây, thứ tự chỉ ở PLAN.


<a id="argv-provenance-correction"></a>
### Correction argv tại AUD072 — không sửa CP250/251 complete hoặc raw

[Đối chiếu và SHA](../evidence/rust-service/argv-provenance.json). Source harness ghi cả scalar `client_args` lẫn
`role_args`, nhưng launch Wine dùng `config.roles[role].args`. Frozenconfig và `run.role_args` của CP250R3/R4/R5
và ba run CP251 đều có token13 ký tự; scalar token32 chỉ là metadata/CLI không được chuyển vào lệnh launch.
ActualCP210 dùng scalar13 trên harness cũ. Do đó claim R5 “khôi phục đúng per-role32 từ CP210” không đúng.
Bằng chứng native gameplay/Mannernegative và raw/hash/cleanup giữ nguyên; chúng không chứng minh input32 hay causalfix.

Đây là observable mới của harness/input, không giải thích SDK/JSON hoặc bắt buộc token phải dài32. CP217 có native
frontendinput32 dương tính riêng, nên được thử boundedFree1v7 với **actualper-role32** cùng source/copy/profile khác
không đổi, có preflight và kiểm actuallaunchargv. Không randomretry/copycycling hoặc RE SDKgraph. Native9 vẫn là bước
độc lập trước trong PLAN. AUD072 seriouscontradiction không reset periodic auditCP252.

<a id="launcher-error-attribution"></a>
### Attribution popup launcher và lỗi main — CP073

[Recipe/argv/ASLR/control/hash](../evidence/nmco-locale-stability/README.md). **kr_runtime/confirmed hẹp**:
standaloneCALauncher, cả emptyargv lẫn2CAargs, không gameTCP: nmcogame nativeexportRVA1D140/actualbase
7A540000 nhận1,0/validation1/internalinit0/errors2000:0, hiển thị đúng popup. Provider/launcher là bytes
KRnguồn/privatecopies, không patchedstub. InvalidID/code1000 và missingexport3000 là nhánh tĩnh khác.

Mainnegative actualTCPclose sau6/trước17 khiến CreateProcessA gọiCALauncher.exe/emptycmd/flags20,
return1258231, rồi mainSIGSEGV17C7F4C. Không khẳng định đây là cause của historicalSIGABRT96A.
HaiA-only+duo/currentprofileactual67 positive, targetlocale/HTTP/fallback probes không hit trongwindow.
96A155→96A480 là HTTPJSON `contentsUrl`, không tự nhận là locale caller từstack. ExactUIstring thuộc
auxiliaryCALauncher, oldprocesssnapshot cũng cóprocessriêng. **Unknown:** gốc startupinterrupt/
HTTPfatal/history và providerinitinternals; popup đồng thời không chứng minh causalroot của main.

Preferredbaseprobe lỗiASLR và genericargvclaim sai giữraw/correction; actualbaseMZ/exportbytes và
emptyargvcontrol đã đo riêng. Không locale/client/provider/servicefix/force success, không stability/
fullauth/K5/K6a/fullmatch/LANInternet/vanilla proof. Bước kế tiếp preclocknative107 observer với
fallback/errorstate diagnostics; không bypassauxiliary locale để giả mainổnđịnh.

<!-- protocol-topic-content-end -->
