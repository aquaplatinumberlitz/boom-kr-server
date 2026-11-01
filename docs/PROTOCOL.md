# KR 00967 — contract tương thích client gốc

Revision kr-legacy-contract-r1; cập nhật02/11/2026 JST (CP054).
**Contract có bằng chứng từng phần; chưa nghiệm thu full codec K2**.
Lộ trình và thứ tự task chỉ giữ ở [REWRITE_PLAN — KRE](PLAN.md).
Đây là contract của repo server KR. ADR0009 của bản viết lại là nguồn lịch sử để so sánh;
vị trí tài liệu nguồn ghi trong [REFERENCES](REFERENCES.md), không phải giao thức dùng cho client KR gốc.
Giới hạn trong mục mang CP ID là phạm vi tại thời điểm đo đó; bổ sung mới ghi ở mục kế tiếp,
không sửa raw/checkpoint cũ. Câu hỏi còn mở và phép kiểm tiếp giữ ở [RESEARCH](RESEARCH.md#q-023).

## 1. Định danh và nguồn chuẩn

Đọc/đo mới trên các target sau; đường dẫn cũ trong notebook tra bằng `/home/admin/boom/README.md`.

| Artifact | Đường dẫn hiện tại | SHA-256 đo 26/10 |
|---|---|---|
| KR gốc | `/home/admin/boom/clients/kr-00967/CA.exe` | `a52545c14b1fea423de655c3163d6360b628d61c1cd472b46fdbc8a9ffe15f4e` |
| KR patcher | `/home/admin/boom/clients/kr-00967/NewPatcher.exe` | `b59e4dce9ad23c439a7632ce04ecb2eed3481585bd2454100b05c9d7527f5c02` |
| VN gốc 116 | `/home/admin/boom/clients/vn-116/ca.exe` | `bce8644d4e783d1354ab4fc880e1ab9a4ab3325327028f37d99e5439e25f4bbd` |
| VN đã dùng RE/runtime | `/home/admin/boom/repos/boom-re/wine/Boom/ca_fixed_patched.exe` | `bc875ba5765cc112e924f46e75d0ace112904c7801126e46bc0abb1c9cb576b1` |

VN gốc và VN patched là **hai artifact khác nhau**; phương pháp/claim runtime có patch provenance.
Một file `.i64` có cùng tên chưa chứng minh input, image layout và các address khớp. Dump memory
phải ghi base/region; VA trong dump không tự chuyển thành offset file packed gốc.

Nguồn đọc ở lượt lập contract: `boom-re/docs/{protocol,oc_d-login,lobby,oc_e-game}.md`,
`boom-re/tools/{boomproto,fakeserver,verify_emu}.py`, `boom-re/server/src/{proto,session,packets,hub}.rs`;
`boomkr-re/docs/kr.md` §1, §7–§33 và `vn-vs-kr.md` §4. Đây là nguồn tham chiếu local,
không dùng các phần trăm ước lượng reuse/rewrite của notebook làm bằng chứng tương thích.
Kiến thức được nhập vào docs hiện hành:
[VN server reference](REFERENCES.md),
[VN battle runtime](REFERENCES.md),
[VN protocol](REFERENCES.md),
[KR flame/chain](REFERENCES.md),
[KR trap timer](REFERENCES.md),
[KR ghost/position](REFERENCES.md).

## 2. Ma trận tái sử dụng VN → KR

Mức `confirmed` dưới đây chỉ áp dụng nguồn/build/phạm vi ghi trong hàng, không tự xác nhận KR wire.

| Lớp | Điều đã có | Cách áp dụng / phần KR còn thiếu |
|---|---|---|
| Debug/input/IAT | VN có recipe GDB proxy, input dispatch/gate trace và đối chiếu IAT theo ABI | CP012 có command login; CP016 có callback/dialog/request Tạo phòng theo [contract](#room-create). Recipe, provenance và giới hạn tại [evidence reuse](evidence/vn-reuse/README.md). Không copy địa chỉ hoặc tự đổi GetAsyncKeyState. |
| Pack/map/sprite/ODT | Reader dùng được cho pack KR; inventory 13 pack/51.467 entry; có khác biệt header map và numeric ID | Dùng lại tool và dữ liệu KR. Điều này không xác nhận version, frame hoặc opcode mạng. |
| Framing/primitive | VN `CClientSocket` `A68850`, recv/process `A662E0`, send `A663E0`; encode/decode/CRC đã có kiểm binary | Dùng làm mẫu tìm class/call graph và mẫu fixture. Patch bootstrap có profile riêng [đã kiểm](evidence/bootstrap-wire/README.md); KR có [profile WCP hẹp](#wcp) và native capture/field checks tới104/121 theo [contract start](#start-reply); full codec/reset/bounds còn mở. |
| Crypto/seq | VN XOR `0x53351D9C`/step `0x63CAACE3`, CRC8/32, base/seq/delta, opcode substitution | Constants chỉ là VN confirmed. Không gán cho KR khi chưa đọc consumer/đo. Kiểm seq tăng trước/sau crypto và reset riêng từng socket/chiều. |
| Hello/version | VN server-first hello, rồi đổi mode; version 116 và packet bootstrap có thứ tự | Patch socket KR: mode0/selector0, BE16 version967, response local được chấp nhận theo [evidence](evidence/bootstrap-wire/README.md). Consumer WCP, initial0/0 và transition2/3 có bằng chứng theo [contract WCP](#wcp); full codec/seq và login/world role chưa xác nhận; không gộp socket. |
| Login/identity | VN có pipeline login-result/network-info và variant China khác layout | CP013–014 có dispatcher/auth; CP015 đi qua redirect/request6 và UI sảnh hết notice trên copy. [Contract sảnh](#channel-lobby); full auth/profile/cache và vanilla vẫn thiếu. |
| Cache/redirect/channel | VN cần tag blob; dữ liệu inner LE khác outer wire, cache path khởi tạo theo stage | Kiểm contract KR từ reader/consumer. Giữ test cold/warm cache; không copy blob VN hay replay flag “dùng cache” khi file không tồn tại. |
| Lobby/room/start | VN có request/reply, slot flags, map tự gửi lúc vào phòng, loading/clock nhiều pha | Dùng để xây checklist ca biên. KR có stage11 và17/241 theo [contract sảnh](#channel-lobby); [request81/response86](#room-create) đã đưa copy tới CPrepareStage và UI phòng chờ; [player/ownslot/controls](#room-player) đã đo trên copy, [map metadata/start104](#room-map) và [start121/scene nhân vật](#start-reply), [clock/input](#round-go), [nổ/HELP/về phòng](#bomb-cycle) đã đo; bảng kết quả chọn đã có [CP023](#round-result), hai client movement/bom đã đo ở [CP027](#peer-duo), join/Ready và chu kỳ pair ở [CP028](#duo-room-round); authority/reset đầy đủ còn thiếu. |
| Bomb/item/trap/result | VN `111→152→151`; `553`, pickup `117→158`; `120` subtype; kết quả `168→173` có runtime nguồn cũ | Chỉ mỏ neo VN. KR phải nối send/receive tới object/state tương ứng trước khi implement. Fuse/drop/cap policy emulator không phải luật server gốc. |
| Peer/movement | VN C→S92 trao IP/UDP port; S→C146 có peer table; runtime hai máy di chuyển đồng bộ qua UDP | Không giả định KR dùng cùng packet/MTUDP version hoặc không đổi peer contract. TCP-only demo không hoàn tất multiplayer khi peer chưa đồng bộ. |
| Kiến trúc/harness | VN có hub/session/round generation, golden Python↔Rust, control 1–2 client | Dùng pattern và bài học, không coi golden hai implementation cùng logic là oracle KR; không copy legacy server vào crates của rewrite. |
| Luật trận chung | Nhiều claim VN confirmed dùng được theo ADR 0002/0013 | Dùng để thiết kế/kiểm hành vi. Luật tương đương không chứng minh binary layout, field widths và wire numbering tương đương. |

## 3. Mỏ neo KR có sẵn và giới hạn

| Locator | Nguồn / trạng thái | Có thể dùng để tìm | Không kết luận từ mỏ neo |
|---|---|---|---|
| `NewPatcher!sub_4118E0`, xref literal `files.nfo2` tại `0x411AF0`, `0x411D1D` | `kr_static` / `confirmed` cho consumer đọc tĩnh, [evidence](evidence/bootstrap/README.md) | Caller/config manifest/download và nhánh lỗi bootstrap | Không phải handler game; chưa nối tới quyết định thành công của `CA!ProcessPatch` |
| `0x00B7A760`, sender packet **167** | `kr_static` / `confirmed`, R-200…R-202 | Lần ngược packet builder/primitive writer từ CGOGameBomber | Không tự gán semantics/layout VN120 hoặc S→C tương ứng |
| `0x00E67E70→0x00E84460`, packet **184 C→S** | `kr_static` / `confirmed` trong phạm vi status serialize | Writer, context/slot và status report | Không gọi đây là S→C trap event hoặc coi client report là quyết định server |
| Packet **183 S→C**, `CGOGhost→B334C0` | `kr_static` / `confirmed`, R-138 | Reader/dispatch/context, setter đơn vị ô 40px | Chưa chứng minh đây là movement sync của bomber sống; C→S148/149 chỉ candidate |
| `0x00E9C520` / `0x00E9CDC0`, message **27** | `kr_static` / `confirmed` theo R-139…R-141 | Tính vùng nổ/chain và đường gọi từ network event | Message27 là nội bộ; wire158/kind0 được bổ sung riêng ở [CP022](#bomb-cycle), chưa full chain/authority |
| `0x01196A70` | `kr_static`, xem [ItemAttr](REFERENCES.md) | Loader ItemAttr; pickup/layout đã có [contract KR riêng](#item-pickup) | Không suy weight thành %, hoặc mặc định drop placement do client chọn. `E9E660` là nhánh kind0 bom, không phải receiver pickup |

Các số opcode này là các mảnh đã có, không phải registry hoàn chỉnh. Tìm được writer từ 167/184
có thể mở được primitive codec trước K1; vẫn cần traffic thật để qua K2.

## 4. Record bắt buộc cho mỗi packet

Registry khóa theo `(build, direction, connection_role, stage, opcode)`; không gộp packet cùng số
khác chiều/state. Mỗi entry phải ghi:

- Artifact SHA, VA/RVA của caller/builder/handler/consumer, evidence ID và phạm vi static/runtime.
- Header/checksum/crypto mode; endian outer và nested blob; seq/key trước/sau packet.
- Field order, offset/cursor, width/count/optional guard/encoding và bounds; unknown giữ nguyên tên trung tính.
- State cần trước khi nhận, side effect, reply/ack/broadcast, redirect socket và thời hạn chờ.
- Mẫu raw→decoded→state có nguồn độc lập; hash capture, run config/patch/cache, trailing bytes và giới hạn.
- Nhãn `kr_static`/`kr_runtime`/`vn_static`/`vn_runtime`/`emu_choice`/`design` và trạng thái
  `confirmed`/`candidate`/`unknown`/`design`. Chưa biết giá trị thì không điền default như fact.

Golden codec cần oracle hàm KR thật hoặc capture, bao phủ size qua ranh giới dword, chuỗi cp949/Unicode
theo consumer, blob raw/nén, checksum sai, seq rollover/reconnect, partial/coalesced frame. Parser server
phải từ chối length/count vượt giới hạn trước allocation. Không fuzz client gốc bằng packet chưa có bounds.

### Phần K2a đã kiểm

[Emulation writer helpers](evidence/primitives/README.md) đã xác nhận plainBE16 `14A5B90`,
XORBE16 `14A5C90` (`0xA569`) và byteXOR `14A5C80` (`0x5A`) trong image unpack có hash đã khóa.
Các helper đạt hết miền input nhỏ và kiểm byte guard/return; **full framing/seq/checksum chưa nghiệm thu**; profile WCP hẹp xem [bên dưới](#wcp).
Review receiver ownership cũng sửa được cách gọi send guard: `13ADD10` dùng receiver từ `545E30`,
packet nằm trên stack, nên không truy nhầm setter `packet+3C` để tìm socket/framer.

<a id="wcp"></a>
### WCP trên kết nối sau ProcessPatch

Nguồn chuẩn cho contract hẹp này; raw/recipe/run/review snapshot ở [evidence WCP](evidence/wcp/README.md).
Target modO base400000 SHA a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5;
runtime ban đầu trên copy Wine R2 SHA7b3b435b5d8e110185359107376b736615ceabe9452cea8154d02c8628f44c29;
các copy chẩn đoán CP006–012 có SHA/patch riêng trong manifest, không tương đương R2 nguyên recipe.
Trạng thái review của tổng hợp ở [CP001](evidence/checkpoints/manifest.json); confirmed nói về loại/phạm vi phép đo, không tự nói đã qua gate.

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

CP006, review theo [manifest](evidence/checkpoints/manifest.json): trên copy reset error và
`multiserveridx:0`, debug/plain run mỗi run nhận 66 byte C→S trên socket thứ hai, SHA và recipe ở
[evidence](evidence/wcp-sequence/README.md). `1238530` tạo packet type0 và gửi qua `1164D50`;
consumer tiếp theo chờ opcode13. Tám response tổng hợp opcode `13,2,12,21,1,43,62,719`
đã được consumer Wine xử lý đủ để guard `12349E0` trả1 trong một run debug; một số payload rỗng
là `emu_choice`, không chứng minh tài nguyên/server gốc. Mode2/3 với header36 byte, XOR và CRC8
được kiểm hẹp bằng native oracle cùng negative checksum/partial và runtime; không suy full codec,
reconnect hay K2. Import NMCO identity chưa bind trên copy CP006; thử nghiệm tiếp
CP007–009 ở [evidence identity](evidence/nmco-identity/README.md), còn blocker/kiểm hiện hành
ở [Q-023](RESEARCH.md#q-023). Tại CP006 chưa có login/gameplay; các mốc sau có phạm vi riêng bên dưới.

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
hoàn tất1→3. [Bằng chứng đối chiếu VN](evidence/vn-reuse/README.md) giữ bytes, guard và giới hạn.
Tại CP013 chưa có runtime S→C4/15 hoặc C→S auth KR; CP014 bổ sung ở [contract login](#login-auth).
Full success schema vẫn thiếu; CP014 có thử S→C15 bên dưới. Không gộp S→C15 này với
bốn C→S15 đã capture ở CP012. Đây không phải toàn bộ endpoint/port,
full codec hoặc proof auth/reconnect K2.
Call/capture và giới hạn tại [evidence handler](evidence/ui-handler/README.md);
Contract username/password, token thật và mode/seq tổng quát vẫn unknown.

<a id="login-auth"></a>
### Login auth trên copy — CP014

Nguồn raw/recipe/hash và các control/failure: [evidence login](evidence/login-runtime/README.md).
Các facts dưới là `kr_runtime/confirmed` hoặc `kr_static/confirmed` trong phạm vi copy đã khóa,
không phải đăng nhập Nexon thật hoặc K2/K3 đã pass.

- Luồng tự động `11010F0` đọc config `token_1/token_2`; tĩnh chọn count0/1/2 theo chuỗi rỗng.
  Runtime `token_1:offline-probe` đi tới `13B07F0` với1; cùng copy không token có0.
  `1255C90` chỉ gọi builder khi count1..2. Chỉ nhánh token_1/count1 đã đo; token_2/count2 chưa kiểm.
- `1255DC1→13ADE40` yêu cầu getter `13AD5B0==-1`, socket ready và count1/2; gọi base ctor
  `14EAD00` với opcode1. Capture connection3 có **C→S1/80 byte**, kế **C→S60/58 byte**, rồi C→S15.
  Native parser `14EF300` replay7 frame với seq40+3n, khớp CRC/payload; CRC-sai/partial từ chối.
  Hai u8 đầu payload auth được native reader đọc4 và1. Full schema/biến thể auth còn unknown.
- Chiều nhận của login socket: tại `14EF300`, hello có mode0/selector0/seq40; phản hồi kế có
  mode2/selector3/**seq43**. Fixture S→C4 result1 dùng43 vào `10FCBE0→10FEDA0`, guard
  `stage+2FD8=1`, decode1 rồi nhánh lỗi đặt dialog result1. Đây là thử nghiệm từ chối login.
  Fixture sai giả định seq40 trước đó vẫn qua dispatch nhưng decode2; CRC của mẫu1byte không
  đủ chứng minh plaintext/seq. Không dùng seq chiều gửi thay chiều nhận.
- Prefix S→C4 result0 có thứ tự đã đo: **outer u8 result, u8, u16, u8 playerCount, u64**.
  Native replay đọc0/0/1/1/0; runtime vào handler result0 và đọc khớp prefix/count local1.
  Body theo từng player, đuôi và S→C15/network-info chưa đủ để tạo phản hồi thành công hợp lệ.
  CP020 mở prefix tới trường ID local, xem [contract](#start-reply); không đóng full auth.
- Prefix13byte cố ý thiếu body vẫn dẫn tới C→S9/46byte. Cấp tiếp fixture S→C15 với seq46,
  blob reader chứa host loopback, ba u16=0, mode0 và tailu16=0: đã vào `1100590` và đo wait
  **0→2→1→3**, setter result3 từ`C4704E`, modal trả3. Không suy getter bounds hoặc schema hợp lệ
  từ việc client tiếp tục với dữ liệu thiếu. Các giá trị loopback/zero là `emu_choice`.
- Sau modal, slot`16EA598` trap ở`13BFD44`, arguments hwnd/msg403/1/0. Copy bind thử
  `PostMessageA` (`emu_choice/candidate`; `SendMessageA` còn alternative) cho attempt trả1 và tới
  xử lý message tiếp `1259C65→125AF18→13BFF43→13C2052`. Run này dừng ở trap`2265828`,
  slot`16EA8FC`, call`13CA429`/return`13CA42F`; đó là blocker CP014, được tiếp nối ở CP015 bên dưới.
  Đây là return của luồng chẩn đoán, không phải login hợp lệ, vào kênh/sảnh hoặc gate K3.

Recipe mới bind Toolhelp process/thread và các API window trên copy, giữ global1A966FC reset0 trong
bộ nhớ như CP012. Các API phục hồi là `emu_choice/candidate`; `GetWindowTextA` còn alternative ABI
`GetClassNameA` chưa loại trừ. Token là giá trị giả local. Không ép modal/state thành success;
không suy các patch/import/API này là tương đương vanilla hoặc là nguyên nhân duy nhất.

<a id="channel-lobby"></a>
### Từ auth tới UI sảnh trên copy — CP015

`kr_static/confirmed` cho reader/dispatcher đã đọc và `kr_runtime/confirmed` cho run ghi rõ ở
[evidence](evidence/postwait/README.md). Các giá trị fixture là `emu_choice`, không phải dữ liệu
server gốc; vẫn kế thừa result0-prefix thiếu body và các patch CP006–014.

- Slot`16EA8FC` khớp cụm export `nmco3!NMCOGetFriendList`, ABI cdecl3. Copy bind hàm thật trả0,
  wrapper trả0 và client tiếp tục. Slot`16EA8C4` khớp `NMCOLogout`, được bind sau trap cleanup;
  chưa có probe return xác nhận riêng Logout. Mapping export theo base ưu tiên/cụm IAT là
  candidate về nguyên bản; descriptor/function đã gọi trên copy là fact runtime.
- Feature `795920` chọn `1107B00`, không phải nhánh yêu cầu stage10. Đã đo init`D665F0` AL1
  và special-success; không gọi đây là init thất bại. Predicate`10FA3A0(a,b)` đúng khi
  `a==2 || (a==1 && b==1)`. Hai getter`13B27C0/13B2810` đọc box tại context`+4C0/+4D8` theo slot.
  Run control có a/b=0/0, pending; global`1A31F30=0`, chưa chứng minh stale global là nguyên nhân.
- **S→C62**, subtype u8=1, playerSlot u8, status u8: base dispatcher`D68C90`→`D88280`→setter`13B27E0`.
  Fixture slot0/status2 với seq49 cập nhật a=2; predicate đổi1, client tự gửi C→S40 rồi **C→S5/47byte**.
  Native reader request5 đọc `[u8=10,u32=FFFFFFFF,u32=0]`. Ý nghĩa lịch sử của hai status còn unknown.
- **S→C7** qua`1101EA0` yêu cầu pending`stage+3748==1`; nhánh result1 tiêu
  `u8 result,u8 kind,u32 index,u32 ip,u16 port` và optional string/vf4C dưới guard`4CD450`.
  Fixture seq52/kind10/index0/loopback/3858 mở **socket4**, hello riêng và **C→S6/74byte**.
  Không mang seq socket3 sang socket4; C→S socket4 replay bắt đầu40+3n, nhận S→C17 seq43 sau hello.
- **S→C17**: `D6E230→D82020→840320→840470`. Serializer KR có
  `8×u16 SYSTEMTIME,u32,9×u8,u8 count,count×(u32,u32),7×u8`; nhánh count0 dài37byte
  đã kiểm bằng native serializer và runtime. Giá trị thời gian/zero là design; count>0/bounds chưa kiểm.
  Run vào request stage11; RTTI `CLobbyStage`, vtable`1766A2C+CC→DA1F30`.
- **S→C241**: base`D6E6B0` đọc blob inner `(u8,u16,u32,u8 count,count×u8)`,
  sau blob còn `u32,u32,u8,u32`. Fixture seq46/count0/zero chạy handler thật; `DA1F30` sau xử lý
  opcode241 gọi`DA5F90`, đặt lobby`+6738=1` và đóng notice`+61F8` nếu active`+61F4`.
  Đã đo ready trước0→sau1 và UI sảnh hết hộp chờ tại CP015. Nhãn result tại`DA5FF1`
  cần đọc đúng callsite/register: probe raw_stack4=16 của CP034 không phải result1; không dùng nó làm proof return.

Không gán request715 thành ACK kết nối: handler715/716 có literal BigBattle; stack polling ở
`DA824D`. Hộp chờ kênh thực là `CNoticeDlgWithoutButton`, getter ready`DA1510`, nguồn tạo`12D4C70`.
Tại CP015, hai click Tạo phòng chưa có request77 hoặc proof hit control; CP016 bên dưới nối callback/request81. `CTimerNoticeDlg`
cuối run nối về `125C63D`/`mainDisconnected` khi kết thúc probe, không phải lỗi tạo phòng đã xác nhận.
Đây là bằng chứng UI/receive path trên copy; profile/tên còn trống, chưa xác nhận auth đầy đủ, room,
gameplay, cold/warm cache hoặc gate K2/K3.

<a id="room-create"></a>
### Tạo phòng: input, danh sách kênh và request — CP016

Nguồn `kr_runtime/confirmed` trong các run nêu tại [evidence](evidence/room-input/README.md),
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
[evidence input](evidence/room-input/README.md). S→C81 locator base `D6A3BC→5D9E60` có guard,
không phải pairing đã xác nhận. CP017 xác định đường đang hoạt động dưới đây.

**CP017 — `kr_runtime/confirmed` trong phạm vi copy**, [raw/control/oracle](evidence/room-reply/README.md).
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

<a id="room-player"></a>
### Player, own-slot và controls phòng (CP018)

Nguồn **kr_static/kr_runtime confirmed trong phạm vi copy**; lựa chọn field/API **emu_choice/candidate**.
Recipe/control/correction/hash tại [evidence](evidence/room-player/README.md). VN docs về
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
CP020 cấp ID1 qua auth và chỉ ra giới hạn start của ID0 tại [contract start](#start-reply).

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

<a id="room-map"></a>
### Map metadata, filter và request start (CP019)

**kr_static/kr_runtime confirmed trong phạm vi diagnostic copy**; fields/default/scheduling là
**emu_choice**, không phải capture server gốc. [Evidence](evidence/room-map/README.md) khóa control,
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

`14EA760` trả stored-checksum; native blob giữ `405E70` và xác nhận **Adler32 seed0 trên inner67byte**:
control`66DE09B4`, enabled`66ED09B5`. Giả thuyết tính cả tag ban đầu bị bác bỏ (`80D60A10` khácnative).
Allocator/free tổng hợp, checksum/reader không bị thay return. Config dùng checksum native tương ứng.
Mẫu raw blob bên trong outer là `[18,u32LE length,selector0,inner]`; checksum này không tính tag/header.

Config mới thử đọc cache1090row/CRC`C5B1B1C1`, khácexpected nên từ chối rồi gửi528.
Sau wire238/count1, bảng dynamic-count lên1 qua`DFEF20`; cache ghi lại khi reload vẫn có checksum khác
trong cửa sổ đo. Đây là **giới hạn cache/persistence đang mở**, không suy cold/warm/reconnect đã qua.
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
bước tiếp hiện hành chỉ giữ ở [RESEARCH](RESEARCH.md#q-023). Không copy start501 của VN.
K2/K3/K4 và full identity/auth/user/cache/vanilla/multiplayer vẫn mở.

<a id="start-reply"></a>
### Phản hồi start và init CGameStage — CP020

Nguồn **kr_static/kr_runtime confirmed trong phạm vi copy/nhánh đã đo**;
[recipe/control/native/hash/correction](evidence/start-reply/README.md).
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
`u32,u32,u16,u16,u32 ID,string(empty),u8`, tổng34byte. Cạnh`10FD8AA→10FD906→13B09E0`
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

Tại CP020, bước tiếp là99/round/clock/input. CP021 bổ sung bên dưới; bước hiện hành ở [RESEARCH](RESEARCH.md);
identity/cache/full auth/codec/multiplayer và gates còn mở.

<a id="round-go"></a>
### Phase, clock map và input — CP021

**kr_static/kr_runtime confirmed trong phạm vi diagnostic copy**;
fixture/default/timing là **emu_choice**. [Evidence](evidence/round-go/README.md) khóa run/hash,
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
Bước hiện hành chỉ ở [RESEARCH](RESEARCH.md).
Chưa hoàn thành một trận, chưa multiplayer/fullauth/cache/vanilla hoặc qua K2–K5.

<a id="bomb-cycle"></a>
### Bom nổ, HELP và trở về phòng — CP022

**kr_static/kr_runtime confirmed theo copy offline**; fixture/time/policy là **emu_choice**.
[Evidence](evidence/bomb-calc/README.md) khóa năm run tuần tự, code/native/ảnh/hash và correction;
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
CP023 đã mở hết hai envelope và nối writer trap/death tại [contract dưới](#round-result).
Không chuyển enum VN11/13 hoặc tự gán vị trí4039 thành spawn hợp lệ; các127/167 khác vẫn cần kiểm riêng.
Report142 được nối tiếp trong [CP032](#hidden-items), không dùng giới hạn CP022 làm current unknown.

**180/branch thường:** dispatcher`E5D62B`, CGameStagevt176A470+2D8→`E64950`.
`1220550` còn có branch khác quaE72490, chưa kiểm. Mẫu branch thường đọc **bool (u8), u8, u8**:
virtual38→`14EB1F0` thực gọi getter3C và tiêu thụ một byte; rồi status và listflag.
Chọn0/0/0, seq91/payload3: `result180-full` đọc3/3/error54=0/error58=0,
`E652D8→12562F0(12)` và UI phòng có KRLocal/KRGuest/nút Bắt đầu vẽ lại.
Listflag0 bỏ list variant; ở CP022, reset flags/position đi qua handler nhưng chưa kiểm start trận kế.

Mẫu đầu2byte có framing và hai getter pass nhưng callback báo54=1, vẫn gọi stage12;
giữ làm short-body control, không gọi full valid result. Việc trở về phòng không tự chứng minh parse đúng.
Trong run CP022, stub phát180 sau158 đúng15s theo test policy. CP023 bên dưới thay trigger bằng
reported-event19 và thêm bảng kết quả175. Serial/dynamic echo, authority đầy đủ, hai client,
full auth/cache/vanilla và gate K2–K5 vẫn mở. Bước tiếp duy nhất ở [RESEARCH](RESEARCH.md).

<a id="round-result"></a>
### Event trap/death và bảng kết quả — CP023

**kr_static/kr_runtime confirmed trong phạm vi copy offline**; outcome/rank/timing của stub là
**emu_choice/design**, không phục hồi server gốc. [Evidence](evidence/round-repeat/README.md) khóa
run/native/code/ảnh/hash và các probe thất bại. Copy không thêm patch; chín vùng code so byte giữa
static copy, runtime copy và modO **memory-layout** tại RVA, không remap modO như PE disk image.

**C→S127:** writer chung `B53010` tạo opcode127; runtime trap tại`B78373` đẩy subtype17,
death tại`E6EFDF` đẩy19, cùng actor và virtual21C trỏB53010. Call`1164D50→13ADD10`
có return site`B7839F` (trap) và`E6F008` (death), đã bắt stack runtime.
Prefix chung **14byte**, gồm u8 subtype,u8 getter actor+584/v28,u16`B335C0`,u16`B335E0`,
u32`12580B0`,u32 actor/v35C. Getter slot/tick/field cuối còn giới hạn semantics;
không tự gán4039 thành vị trí map hợp lệ hoặc12580B0 thành milliseconds.
Sau prefix,`14EC120` nối buffer serialized có length và checksum; native`14EB410` giải đủ:

| Capture event127-direct | Prefix | Inner sau giải buffer | Cursor outer |
|---|---|---|---|
| frame49/seq187, trap17 |17,0,40,4039,1445,5 |8byte=`0200000000000000` |32/32 |
| frame53/seq199, death19 |19,0,40,4039,1654,0 |5byte=`ffffffff00` |29/29 |

Native framer/primitive/blob và Adler32(seed0) độc lập đều khớp. Inner tương ứng2u32 vàu32/u8;
chưa đặt tên tất cả field, chưa kiểm duplicate, stale, peer hoặc ai chấp nhận/correction event.
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
reply159 lúc đó vẫn dùng mẫu100/1/id1; echo/spawn được kiểm tiếp tại [CP024](#bomb-position). Request104 lần hai wire461 thay vì109; framing/CRC có kiểm,
body tăng chưa giải thích. Delay kind0→reported19 khoảng6.5s/0.9s khác nhau chưa giải thích;
không dùng thành công lặp UI để tuyên bố spatial/timer reset chính xác hoặc authority đã khôi phục.
Guard log state trong probe là pointer wrapper, không phải raw flag word; điều kiện bit ở trên
đến từ code tĩnh và input110 được consumer chấp nhận, không từ cách printf đó.

<a id="bomb-position"></a>
### Echo bom, nguồn spawn và map cụ thể — CP024

**kr_static/kr_runtime confirmed trong phạm vi copy offline**, encoder/profile/policy còn là
**emu_choice/design**. [Evidence](evidence/bomb-position/README.md) khóa năm run tuần tự,
10actual118,56native getter trên snapshot, control/ảnh/hash. Copy SHAe9bb47… không thêm patch;
13vùng code khớp static copy660447… và modO memory-layout a08950….

**Lịch sử CP024, chỉ bom đầu/serial0:** stub lúc đó chuyển field3/u32 của118 vào field2/u16
của159, giữ field3/u32 của159=0. Vì chỉ đo serial0, cả hai field đều0; đây là candidate chưa phân biệt
được serial. **CP025 bác cách ghép tổng quát này** bằng bom thứ hai/serial28, sửa ở
[contract serial](#bomb-serial); raw/checkpoint CP024 giữ nguyên. Hai byte hàng/cột vẫn được kiểm
độc lập. Byte thứ ba từng gán power candidate bị CP031 bác bằng Fire; xem [mapping power](#item-effects).
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
K5/K6, collision/item/chain/authority và peer vẫn mở; bước tiếp ở [RESEARCH](RESEARCH.md#q-023).

<a id="bomb-serial"></a>
### Nhiều bom, serial và ID — CP025

**kr_static/kr_runtime confirmed trong phạm vi copy offline**, [evidence](evidence/bomb-serial/README.md).
Bốn run108s tuần tự trên cùng copy SHAe9bb47…; không patch thêm. Request118 thực có serial0,28,56,
không phải giả định số tăng từng bom. Chưa giải cách sinh/rollover serial hoặc mọi optional flag.

Với118/base22/flag0, widths13field vẫn `[4,1,1,4,1,1,1,1,1,1,1,4,1]`.
159/base39 có widths17field `[1,1,2,4,4,4,1,1,1,4,1,2,2,1,8,1,1]` (index từ0):

| Nguồn thực118 | Đích159 | Giới hạn |
|---|---|---|
|field2/u8 owner |field1/u8 |owner0 trong các run này |
|field3/u32, record+0C |**field3/u32**, payload+4 |serial ghép local pool, đo0/28/56; không ép u16 |
|field4/5/u8 |field6/7/u8, payload+16/+17 |hàng/cột đã đo; không gộp field6 của118 thành power |
|**field10/u8, record+18** |**field8/u8**, payload+18 |Correction CP031 bằng Fire1→2; [bằng chứng](#item-effects). CP025 dùng field6=1 nên chưa phân biệt hai field |
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

<a id="kr-peer-entry"></a>
### Request99 và bảng peer KR153 — CP025–027

[Evidence CP026](evidence/peer153/README.md). **kr_runtime confirmed hẹp trên copy Wine**,
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
[Bước tiếp](RESEARCH.md#q-023).

<a id="peer-duo"></a>
### Hai client thật: movement và bom — CP027

**kr_runtime confirmed trong phạm vi harness Wine**, [recipe/control/hash](evidence/peer-duo/README.md).
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
production tại CP027**. CP028 đã nối [join/Ready thực](#duo-room-round); full auth/profile/cache vẫn thiếu, không gán cấpIDlocal thành login Nexon thành công.

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
[contract serial](#bomb-serial) và [correction](#item-effects). Các bom CP027 đều power1, chưa phân biệt hai field.
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
chưa đủ. Bước tiếp chỉ giữ ở [RESEARCH](RESEARCH.md#q-023); không tự đóng K4/K5/K6 từ mốc này.

<a id="duo-room-round"></a>
### Join, Ready và chu kỳ hai client — CP028

**kr_static/kr_runtime confirmed hẹp trên copy Wine**, [recipe/control/hash](evidence/duo-room-round/README.md).
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

<a id="peer-events"></a>
### Đồng bộ trap/death qua KR170 — CP029

**kr_static/kr_runtime confirmed trong phạm vi copy Wine**, [control/recipe/hash](evidence/peer-events/README.md).
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

<a id="item-pickup"></a>
### Item trên nền và xác nhận nhặt — CP030

**kr_static/kr_runtime confirmed trong phạm vi copy Wine**; [recipe/control/hash](evidence/item-pickup/README.md).
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

<a id="item-effects"></a>
### Bomb/Fire: stat, giới hạn và nguồn power — CP031

**kr_runtime/kr_static confirmed trong phạm vi diagnostic copy Wine**;
[control/native/snapshot/hash và audit](evidence/item-effects/README.md). CP030 mới kiểm layout/pickup;
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

<a id="item-families"></a>
### Nhóm item và kiến thức tái sử dụng — CP045

Đối chiếu theo câu hỏi Battle cơ bản; bảng dưới không là kiểm kê mọi cơ chế trong client.
Nguồn/hash/locator và bốn câu hỏi tái sử dụng tại [evidence](evidence/item-families/README.md).
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
| Dùng chủ động group23: Needle145↔117 và12 hàng KR cùng group |Kim Ctrl sau trap, dùng cạn; R169–185 không là bằng chứng kim |1190C20 amount>0 rồi nhánhgroup23/vf1C; variant cùng group là ứng viên tái dùng đường |[Kim145](#needle-selfescape)/[lần hai](#needle-repeat): pickup124/165→Ctrl×1→actual127/1 rồi0(kind145), peer170/1 đưa56→26; cạn không thoát lần hai. Inventory validator, multi-charge/variant/round-reset toàn túi chưa đo. |
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

<a id="hidden-items"></a>
### Gạch, item ẩn và khởi tạo lại vòng — CP032

**kr_static/kr_runtime confirmed trên diagnostic copy Wine**, không patch thêm hoặc ép stat/state.
[Recipe, controls, native, snapshot và hash](evidence/hidden-items/README.md).
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

<a id="bomb-chain"></a>
### Nổ dây chuyền hai bom — CP033

**kr_runtime/kr_static confirmed hẹp trên diagnostic copy Wine**, không patch thêm;
[controls/recipe/native/hash](evidence/bomb-chain/README.md). Dùng profile ExVillage01/278,
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

<a id="room-leave"></a>
### Rời phòng, chủ mới và chọn loại sảnh — CP034

**kr_runtime/kr_static confirmed trong phạm vi diagnostic Wine copy**;
[10 run, recipe, control và hash](evidence/room-leave/README.md). VN `docs/lobby.md` §0/§18
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
[Bước tiếp duy nhất](RESEARCH.md#q-023).

<a id="room-rejoin"></a>
### Rejoin và Start sau chuyển chủ — CP035

**kr_runtime/confirmed trong phạm vi copy/profile**, [hai run và hash](evidence/room-rejoin/README.md).
Không thêm binary patch/ép state; dùng lại channel14 loại1 và các contract [leave](#room-leave),
[join/Ready](#duo-room-round), [start](#start-reply)/[peer](#peer-duo). Fixture/policy là **emu_choice/design**.

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
[Bước tiếp](RESEARCH.md#q-023).

<a id="collision-rescue"></a>
### Va chạm enemy, rescue và report21 — CP036

**kr_runtime/kr_static confirmed hẹp trên diagnostic copy Wine**,
[7 run/control/native/hash/audit](evidence/collision-rescue/README.md). Không patch thêm/ép game state.
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
[Bước tiếp](RESEARCH.md#q-023).

<a id="needle-selfescape"></a>
### Kim tự thoát và đồng bộ peer — CP037

**kr_runtime/kr_static confirmed hẹp trên diagnostic copy Wine**,
[recipe, control và hash](evidence/needle-contact/README.md). Không thêm patch hoặc ép state actor.
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
[Bước tiếp duy nhất](RESEARCH.md#q-023).

<a id="needle-repeat"></a>
#### Bị nhốt lần hai khi hết kim — CP038

**kr_runtime confirmed hẹp**, [run/recipe/hash](evidence/needle-repeat/README.md).
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

<a id="contact-precedence"></a>
### Tranh chấp cứu/diệt: ba client thật và nhánh chọn — CP037

**kr_runtime confirmed hẹp; quy tắc dừng ở chạm hợp lệ đầu kiểm bằng kr_static/emulation**,
[run/recipe/hash](evidence/needle-contact/README.md). Ba client A/B/C có authID1/2/3,
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
complete; [AUD012](evidence/checkpoints/audits/AUD012.json) giữ correction/nguồn/giới hạn.
Checkpoint/raw trước đó không bị viết lại. Trạng thái runtime còn thiếu và thứ tự kiểm tiếp chỉ giữ
ở [RESEARCH](RESEARCH.md#q-023); recipe/số đo ở [evidence](evidence/needle-contact/README.md).
Không suy luật server gốc/validator, kết quả175, full match/K5/K6a/LAN/Internet từ các nhánh trên.

<a id="contact-inversion"></a>
#### Đối chứng đổi team slot1/2 — CP038

**kr_runtime confirmed hẹp**, [recipe/control/hash](evidence/needle-repeat/README.md).
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
độ khớp client gốc; ưu tiên tiếp result/disconnect/reset ở [RESEARCH](RESEARCH.md#q-023).

<a id="round-finish"></a>
### Trap/death nhất quán → kết quả → vòng kế và host EOF — CP039

**kr_runtime/kr_static confirmed hẹp trên copy Wine**, [recipe/control/hash](evidence/round-finish/README.md).
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
không serverauthoritygốc. [Bước tiếp](RESEARCH.md#q-023).

Chưa toàn reset/validator/generation/disconnectvariants/rejoin/fullauth/cache/codec/vanilla/
LAN/Internet hoặc K5/K6a. Đây là ca hai vòng thử có kết quả/statepeer nhất quán hẹp,
không tự đổi thành nghiệm thu toàn trận đầy đủ.

<a id="abort-rejoin"></a>
### Sau host EOF: client mới vào slot trống, chủ mới Start — CP040

**kr_runtime/confirmed hẹp trên diagnostic copy Wine**;
[recipe, các lượt chưa đạt, control và hash](evidence/abort-rejoin/README.md).
Không thêm patch hoặc ép game state. Kế thừa [forfeit/return](#round-finish),
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
[Bước tiếp duy nhất](RESEARCH.md#q-023).

<a id="round-generation"></a>
### Payload nổ cũ qua vòng mới và tái dùng ID — CP041

**kr_runtime/kr_static confirmed hẹp trên diagnostic copy Wine**;
[controls, recipe, native checks và hash](evidence/round-generation/README.md).
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

**Design server:** [RoundFence](../server/round_fence.py) giữ ticket theo instance owner phòng,
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
**Chưa nghiệm thu full match/K5/K6a/LAN/Internet**. [Bước tiếp](RESEARCH.md#q-023).

<a id="item-generation"></a>
### Xác nhận item cũ khi tái dùng index qua vòng — CP042

**kr_runtime/kr_static confirmed hẹp trên diagnostic copy Wine**;
[recipe/control/hash](evidence/item-generation/README.md). Không thêm patch hoặc ép actor/stat.
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

**Policy emulator:** [ItemClaims](../server/item_claims.py) kiểm catalog hiện tại cùng slot gắn socket,
context/index/hàng/cột/kind, giữ một winner và ticket xác nhận local gắn owner/generation.
`begin/finish` vô hiệu hóa ticket cũ; `consume` chỉ một lần dưới owner/lock trước phát165.
Đây là design cho job/xác nhận local chưa gửi, không field epoch phục hồi từ wire.
Request124 cũ trùng toàn bộ catalog hiện tại vẫn không thể được module phân biệt chỉ bằng các field đó;
distance/tick/session/actor-state, pending159, timer abort và UDP/peerqueue còn cần kiểm riêng.
Không gọi module này là toàn validator hoặc nghiệm thu K5/K6a/LAN/Internet.
[Bước tiếp](RESEARCH.md#q-023).

<a id="late-bomb-confirm"></a>
### 159 cũ tới khi serial local được dùng lại — CP043

**kr_runtime/kr_static confirmed hẹp trên diagnostic copy Wine**;
[recipe/control/hash](evidence/late-bomb-confirm/README.md). Không thêm patch/ép state.
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

Áp dụng [RoundFence](../server/round_fence.py) cho việc phát159 chưa gửi cần giữ ticket gốc từ
request/round ban đầu và kiểm `rejection()` dưới owner/lock ngay trước send. Không tạo ticket mới
từ ID/serial đang active để “hợp thức hóa” job cũ. Fence là policy local, không field epoch wire;
kiểm lifetime trước159 không tự chặn duplicate159 trong cùng generation hoặc packet đã phát/UDP.
Generation/ID wrap/abort timer và fullvalidator vẫn có scope riêng.
**Chưa fullmatch/K5/K6a/LAN/Internet**. [Bước tiếp](RESEARCH.md#q-023).

<a id="abort-timer"></a>
### Timer bom còn chờ khi host EOF — CP044

**kr_runtime confirmed trong phạm vi diagnostic copy Wine**;
[control, recipe và hash](evidence/abort-timer/README.md). Không thêm patch hoặc ép actor/state.
Kế thừa [forfeit/return/promotion](#round-finish) và [client thay thế](#abort-rejoin).
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
**Chưa fullmatch/K5/K6a/LAN/Internet**. [Bước tiếp](RESEARCH.md#q-023).

<a id="udp-generation"></a>
### UDP qua vòng mới: queue, drain và consumer game — CP045

**kr_runtime/kr_static confirmed hẹp** trên cùng diagnostic copy, không thêm patch/ép actor.
[Nguồn đã đối chiếu, recipe/hash và các ca chưa đạt](evidence/udp-generation/README.md).
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
[Nguồn đối chiếu, recipe, control/lỗi probe và hash](evidence/udp-actor/README.md).
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
rekey/storage trong emulation; không đổi counter/state của client runtime. [Giới hạn phép kiểm](evidence/udp-actor/README.md).
Giới hạn: một oldpacket/cùng endpoint/profile; chưa toàn peer grammar, freshness validator, loại gói
cũ theo generation hoặc giải pháp production. [Phép thử kế](RESEARCH.md#q-023).

**CP047 — handler, byte khóa và bộ lọc ngắn, kr_static/kr_runtime confirmed hẹp:**
[Nguồn/cache đã tái dùng, recipe/hash và các mức kiểm](evidence/udp-freshness/README.md).
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
không nâng sáu ca thành runtime wrap/security. Bộ cấp [UdpRoundKeys](../server/udp_round_keys.py)
là `design/emu_choice`: cấp một u32 mỗi acceptedStart, cùng khóa cho mọi member, không lặp low8
trong một lifetime, báo lỗi sau256 lần; caller giữ instance qua finish/abort và quản lifetime/roster.
Unit test không là runtime tích hợp module; các harness trên chọn12345/12346 trực tiếp. Roster/ID thay,
key reuse, packet type khác, giả mạo và peer lifetime mới vẫn cần xử lý riêng; không gọi khóa này là auth.

### Giới hạn và correction WCP lịch sử

Fixture opaque/zero/empty và prefix69 là emu_choice đã được consumer chấp nhận trong phép thử,
không phải byte server gốc. Nhánh flag1,codec frame mode2/selector3 ở phạm vi fixture CP006 có kiểm hẹp; seq/reconnect, semantics đầy đủ và
message400 handler vẫn unknown.

Correction CP001: WCP r1 raw mô tả sai mapping fields và suy caller1255350. Giữ raw nguyên:
đúng là socket+40=mode2,+44=selector3,+48=0 theo oracle; route run này là1255AC0.
1234E70 return1 chỉ cho biết1254D50()==0 ở lần đọc đó,có thể cả khi1255A30 trả0;không dùng
return1 của wrapper này thay proof consumer WCP.1255350 là đường tĩnh khác,chưa quan sát trong run này.
Absence WCP timeout không phải runtime proof;
dùng breakpoint đã nêu. Câu hỏi/thí nghiệm mới chỉ giữ ở [RESEARCH](RESEARCH.md#q-023).

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

Bảng này tổng hợp bằng chứng từng phần đến CP055; chưa phải toàn contract server gốc. Mỗi hàng khi
triển khai phải bổ sung riêng **creator, simulator, validator, broadcaster, peer-editable state**, với
locator/trace và trạng thái từng ô; những vai trò không nêu dưới đây hiện `unknown` cho KR.

| Hành vi | Đã thấy trong KR | Phần cần đóng trước demo dùng hành vi đó |
|---|---|---|
| Movement | CP027 hai client thấy movement hai chiều; [phạm vi](#peer-duo);183 vẫn chỉ ghost | Full schema input/update/correction, validator và quyền sửa vị trí của peer |
| Bomb place/ID | CP027 hai owner/ID chung1..4 tạo trên cả hai client; [phạm vi](#peer-duo); lặp/cũ CP025 | CP041 có [stale158/ID reuse và fence job local](#round-generation), CP043 có [old159/serial reuse](#late-bomb-confirm); allocator/rollover/concurrent/race/toàn generation và authority ngoài policy thử còn mở |
| Explode/chain | CP033 hai bom B liền kề từ một158/ID1, control cách hai ô/replay sau chain; [contract](#bomb-chain) | Nhiều owner/ba bom/vật cản/race/generation và authority server gốc còn thiếu |
| Block destroy/item reveal | CP032 list2→FixedBox,158→hai view vỡ/reveal→124/165/stat; report142 từ cả hai; [contract](#hidden-items) | Drop algorithm/server validator, variants/chain và phép loại transport riêng còn thiếu |
| Item placement/pickup | CP030 nhặt/tranh index; CP031/032 stat/itemẩn; CP045 [bảng nhóm/Roller/MaxPower hai view/reset](#item-families) | CP042 có [old165/index reuse](#item-generation); runtime tại max, KRspeed→px/s/TTL/activevariants, drop/đảo winner/toàn124validator/generation/reset còn thiếu |
| Trap/death | CP029/036; CP037 kim và tranh chấp cứu/diệt; [kim](#needle-selfescape), [precedence](#contact-precedence) | Validator/duplicate/stale/generation/variants, result21 và đối chứng thêm/đảo chiều còn thiếu |
| Result/reset | CP039 hai vòng statepeer/result/21/return và reset field hẹp; CP044 [timer ở EOF và vòng B/C mới](#abort-timer) | CP041 có [payload cũ khi ID reuse](#round-generation); arbitration/toàn generation/entity/timer/peer/seq không rò vẫn mở; phân biệt run chờ sai origin với ca hoàn tất |
| Room lifecycle | CP034 leave/promotion/EOF/return; CP035 rejoin/Ready; CP040 chủ B/C Start; CP044 [B/C bom/death/result/return sau abort](#abort-timer) | Bầu chủ/giữ ID-slot/forfeit là policy stub; guest EOF trả A về phòng/xóa B đã đo riêng; CP050 có [service EOF/cleanup](#service-eof); CP051 có [C mới/Start B trong service](#service-replacement); CP052 [guest setup EOF](#setup-eof) đã đo; host với peer chưa map/reconnect cùng account/disconnect variants/toàn reset còn mở |
| Peer lifecycle | CP027 actual99→153; CP045–047 [oldUDP actor/keyguard/endpointcontrol](#udp-generation) | CP048 [service fixture](#probe-service) đã tích hợp key policy hẹp; low8reuse/roster/UDPtype khác/fullgrammar/reliability/relay/reset còn mở. Không coi đổi cổng riêng lẻ là freshness guard hoặc keybyte là auth |

Gate K5/K6 cần kiểm matrix và đối chứng của hành vi thực sự dùng; từ28/10 Codex tự kiểm theo miễn review của chủ dự án. Có thể giữ unknown về
server gốc thất lạc nếu policy emulator được chọn `design`, client chấp nhận và ca consistency đạt;
không thể để unknown về đường đồng bộ cần thiết rồi tuyên bố multiplayer đã nghiệm thu.

<a id="probe-service"></a>
## Service fixture và phạm vi tích hợp — CP048

Nguồn/recipe/actual trace/native checks ở [evidence service](evidence/probe-service/README.md).
`server/probe_service.py` là tiến trình TCP riêng; `probe_codec.py` chỉ hỗ trợ profile mode2/selector3
đã đo, header37 byte và typed reader. Header37 tính cả byte prefix; mô tả header36 lịch sử WCP
không được dùng để bỏ byte prefix. `probe_room.py` serialize owner, gọi UdpRoundKeys một lần/Start,
RoundFence cho159/158 và ItemClaims cho124→165; cùng khóa153 cho hai peer. Đây là implementation
`emu_choice/design`, không phục hồi thêm field wire hoặc luật server gốc.

Auth/roster/map/rank dùng fixture có SHA; snapshot CP048 dùng identity A/B theo thứ tự bốn connection mỗi vai.
CP051 mở đường [A/B→B/C hẹp](#service-replacement), không đổi thành auth/membership tổng quát.
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
Tại snapshot CP048, service parse `t/k` nhưng chưa kiểm chúng; CP049 bổ sung [guard k](#pickup-round-key). Replay124 trùng catalog vẫn là khoảng
trống có thể đo; không tự nói allocator UDP đã giải C2S stale124. Kiểm codec mới so **typed prefix của
97 actual/native-checked frame**, ngoài354roundtrip cũ; native framer/blob/CRC có proof riêng.


**Khoảng trống service được thực thi sau refactor:** body actual124/native-checked của r2 vòng1
được đưa vào `receive_channel` sau begin/finish/begin (catalog17 giữ, key12346). ItemClaims chấp
nhận body cũ `k12345` và phát positive165; current124/k12346 kế tiếp bị từ chối do claim đã tiêu.
Control chỉ current124 được chấp nhận. Đây là `design/confirmed` về code service, **không** runtime
client KR hoặc validator server gốc; [recipe/hash](evidence/probe-service/README.md) giữ phân biệt.
Tại CP048 chưa thêm guard; CP049 bổ sung bên dưới, không gọi toàn stale124 đã giải. Phép đo tiếp ở [RESEARCH](RESEARCH.md#gap-floor-pickup).


<a id="pickup-round-key"></a>
### Guard124 theo khóa đã cấp — CP049

[Schema124/165 và consumer](#item-pickup) giữ nguyên. [Recipe/raw/hash/controls](evidence/stale124/README.md)
tái dùng CP030/042/047/048 và nguồn VN, không phục hồi lại receiver hoặc đổi layout/opcode theo VN.

`probe_service.receive_channel` kiểm **u32 k (field7)** bằng khóa đang cấp ở u32 thứ hai153 **trước
ItemClaims.claim/consume**. Khác khóa: ghi `round_key_mismatch`, không phát165/không chiếm claim.
Khớp khóa: tiếp origin/context/catalog/first-winner/one-consume hiện có. So sánh cảu32 là lựa chọn
`design`, không sao chép quy tắc low8 của [movement](#udp-generation) hoặc gọi k là auth.

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


<a id="service-eof"></a>
### Service EOF và loading chờ đủ99 — CP050

[Recipe/raw/hash/nguồn reuse/native checks](evidence/service-eof/README.md). Wire107/110/175/180
vẫn theo [room leave](#room-leave), [forfeit](#round-finish), [pending timer](#abort-timer);
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
timeout/latejoin/loadfail hoặc toàn gate K4. Giới hạn ở [evidence](evidence/service-eof/README.md).

Tại snapshot CP050, service chỉ hai identity/cap8/StartA; CP051 tích hợp
[C mới/roster-ID mới và B Start](#service-replacement). Giữ giới hạn CP050 như lịch sử.
Fullvalidator/auth/cache/codec/vanilla/K5/K6a/LAN/Internet giữ mở. Câu hỏi ở [RESEARCH](RESEARCH.md#gap-disconnect-replacement).


<a id="service-replacement"></a>
### Client mới thay slot sau EOF trên service độc lập — CP051

[Recipe, nguồn reuse, raw/hash và hai run](evidence/service-replacement/README.md). Tái dùng
[roster/rejoin](#room-rejoin), [B/C sau abort](#abort-timer), [EOF](#service-eof), không phục hồi
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
còn mở. Phép kiểm còn thiếu ở [RESEARCH](RESEARCH.md#gap-disconnect-replacement); thứ tự chỉ ở PLAN.


<a id="setup-eof"></a>
### Guest rời lúc setup trước map/Start — CP052

[Control/raw/hash/nguồn reuse](evidence/setup-eof/README.md); schema/consumer107/110 vẫn ở
[room leave](#room-leave), không thêm field wire hoặc phục hồi lại receiver. Policy `design`:
Join/create được chấp nhận (`create_response_sent`) xác lập membership độc lập với tiến độ nhận map.
Khi channel đóng trong phòng chưa active, cleanup theo marker đó; socket lobby/chưa Join không bị
coi là member. Đường active/đã có result vẫn theo [EOF CP050](#service-eof). Closed event bị bỏ khỏi
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
fullvalidator/reset hoặc gate. Phép kiểm tối thiểu ở [RESEARCH](RESEARCH.md#gap-disconnect-replacement).


<a id="host-setup"></a>
### Host EOF khi survivor chưa map — CP053

[Control/fix/recipe/hash](evidence/host-setup/README.md), tái dùng [107/110](#room-leave) và
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
codec/reset/timeout/sendfailure và K5/K6a/LAN/Internet giữ mở; [phép đo](RESEARCH.md#gap-disconnect-replacement).


<a id="prestart-replacement"></a>
### C mới vào trước vòng đầu sau setup EOF — CP054

[Control/fix/source/raw/hash](evidence/prestart-replacement/README.md); dùng lại [CP053 cleanup](#host-setup)
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

<a id="pickup-tick"></a>
### Nguồn và lifetime của t trong124 — CP055

**kr_static/kr_runtime confirmed trong scope Wine copy**, [recipe/hash/giới hạn](evidence/pickup-time/README.md).
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
forged-current-key và TCP replay admission còn mở; bước kiểm ở [RESEARCH](RESEARCH.md#gap-floor-pickup).
