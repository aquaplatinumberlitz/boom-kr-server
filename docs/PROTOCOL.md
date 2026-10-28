# KR 00967 — contract tương thích client gốc

Revision kr-legacy-contract-r1; cập nhật28/10/2026 JST (CP016).
**Contract có bằng chứng từng phần; chưa nghiệm thu full codec K2**.
Lộ trình và thứ tự task chỉ giữ ở [REWRITE_PLAN — KRE](PLAN.md).
Đây là contract của repo server KR. ADR0009 của bản viết lại là nguồn lịch sử để so sánh;
vị trí tài liệu nguồn ghi trong [REFERENCES](REFERENCES.md), không phải giao thức dùng cho client KR gốc.

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
| Framing/primitive | VN `CClientSocket` `A68850`, recv/process `A662E0`, send `A663E0`; encode/decode/CRC đã có kiểm binary | Dùng làm mẫu tìm class/call graph và mẫu fixture. Patch bootstrap có profile riêng [đã kiểm](evidence/bootstrap-wire/README.md); KR có [profile WCP hẹp](#wcp) và native capture/field checks tới81; full codec/reset/bounds còn mở. |
| Crypto/seq | VN XOR `0x53351D9C`/step `0x63CAACE3`, CRC8/32, base/seq/delta, opcode substitution | Constants chỉ là VN confirmed. Không gán cho KR khi chưa đọc consumer/đo. Kiểm seq tăng trước/sau crypto và reset riêng từng socket/chiều. |
| Hello/version | VN server-first hello, rồi đổi mode; version 116 và packet bootstrap có thứ tự | Patch socket KR: mode0/selector0, BE16 version967, response local được chấp nhận theo [evidence](evidence/bootstrap-wire/README.md). Consumer WCP, initial0/0 và transition2/3 có bằng chứng theo [contract WCP](#wcp); full codec/seq và login/world role chưa xác nhận; không gộp socket. |
| Login/identity | VN có pipeline login-result/network-info và variant China khác layout | CP013–014 có dispatcher/auth; CP015 đi qua redirect/request6 và UI sảnh hết notice trên copy. [Contract sảnh](#channel-lobby); full auth/profile/cache và vanilla vẫn thiếu. |
| Cache/redirect/channel | VN cần tag blob; dữ liệu inner LE khác outer wire, cache path khởi tạo theo stage | Kiểm contract KR từ reader/consumer. Giữ test cold/warm cache; không copy blob VN hay replay flag “dùng cache” khi file không tồn tại. |
| Lobby/room/start | VN có request/reply, slot flags, map tự gửi lúc vào phòng, loading/clock nhiều pha | Dùng để xây checklist ca biên. KR có stage11 và17/241 theo [contract sảnh](#channel-lobby); [request81/response86](#room-create) đã đưa copy tới CPrepareStage và UI phòng chờ; [player/ownslot/controls](#room-player) đã đo trên copy, map/load/trận chưa có. |
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
| `0x00E9C520` / `0x00E9CDC0`, message **27** | `kr_static` / `confirmed` theo R-139…R-141 | Tính vùng nổ/chain và đường gọi từ network event | Message27 là nội bộ; handler wire explosion chưa định vị |
| `0x01196A70`, `0x00E9E660` | `kr_static`, xem [ItemAttr](REFERENCES.md) | Loader/receiver để đối chiếu index/kind/actor | Không suy weight thành %, hoặc mặc định ItemAttr/drop placement do client chọn |

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
ở [Q-023](RESEARCH.md#q-023). Không có login/gameplay.

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
  Đã đo trước0→sau1, setter result1 tại`DA5FF1`, notice trả1 và UI sảnh hết hộp chờ.

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
  mới xác nhận count0, chưa đủ contract danh sách phòng không rỗng.
- Sau91, callback qua guard rồi kiểm level`1220360`; control chưa có bảng kênh chọn index−1,
  getter`13A2E10` tính`context+E4*index+84`, gây fault`52A219` trên secure-box ngoài hàng.
  Chưa quy lỗi này cho profile hoặc input; snapshot xác nhận index−1 trong run này.
- **S→C14**, base dispatcher`D68C90`→`D69208`→`13A2FD0`→cache`13A1DE0`.
  Nhánh outer u8=1 đọc blob; raw blob là `[18,u32LE length,storageSelector=0,innerFields]`.
  Byte selector nằm **bên trong** phần length. Thiếu selector đã gây reader length0/count26276;
  frame/CRC/first-field pass không xác nhận blob/schema. Nhánh selector1 là compressed, chưa kiểm.
  Inner có u16LE count, mỗi row theo`13A20D0`: string(u32LE byte-length),u32LE,
  7×u8,2×u16LE,4×u8,2×u32LE. Chỉ count1/name rỗng/kind10 đã thử; semantics/bounds còn thiếu.
  Native blob reader xác nhận count1/33byte inner. Runtime copy bảng qua`13A28F0`,
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

ID1 hiển thị nhân vật/tên/nhãn chủ phòng sau khi sửa import pointer-check, nhưng local user list
thực tế có count1/ID0. `13B1540` so incoming1→−1, `123FF70` không đặt own flag;
`DC5925` đo own_slot0. Gửi incoming0 theo giá trị vừa đo → lookup0/own_slot1;
client tự gửi thêm96/266, không ép own flag hoặc sửa danh sách ID từ debugger.
Đây là workaround dữ liệu của recipe auth chưa đủ, **không phải ID hợp lệ/stable của server gốc**.
NMCO code1 kế thừa không được đồng nhất với local login user-ID0.

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
team0/1 và nhánh start, không phải hai client hoặc ready/team roundtrip. Chưa gửi request start.
Một lần trước đó (`room-teams`) wineboot timeout35s trước client, không tính là phép đo game;
retry prefix mới giữ nguyên copy/fixture/harness.
Handler117/DC70D0 (locator tĩnh) đọc resultu16 rồi nhánh map; cần kiểm bảng map/index/data trước fixture.
Chưa có map/load/Battle, auth/profile/cache hoàn chỉnh hoặc hai client.

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

### Authority matrix ban đầu và gate review

Đây là trạng thái RE trước runtime KRE, không phải contract server gốc đã khôi phục. Mỗi hàng khi
triển khai phải bổ sung riêng **creator, simulator, validator, broadcaster, peer-editable state**, với
locator/trace và trạng thái từng ô; những vai trò không nêu dưới đây hiện `unknown` cho KR.

| Hành vi | Đã thấy trong KR | Phần cần đóng trước demo dùng hành vi đó |
|---|---|---|
| Movement | Consumer dir/position có phạm vi lớp;183 đã nối ghost | Input player→mover/network, peer update/correction và ai chấp nhận vị trí |
| Bomb place/ID | Chưa có wire contract đầy đủ | Creator, ID/serial namespace, ack và ai phát cho các peer |
| Explode/chain | Client có calc `E9C520/E9CDC0`, message27 | Trigger wire/timer/clock owner, relay và cách bỏ lặp chain |
| Block destroy/item reveal | Calc vùng nổ không tự xác nhận authority phá block/drop | Ai quyết block/item, state từng peer và packet/event đồng thuận |
| Item placement/pickup | Loader/receiver có mỏ neo, chưa đủ contract | Kind/index theo build, ai đặt item, xác nhận pickup/race và broadcast |
| Trap/death | Client gửi167/status184 trong phạm vi R-200…202 | Ai chấp nhận kết quả, đường nhận của peer, timeout và loại trùng |
| Result/reset | Chưa có wire contract | Người quyết outcome, result/round identity và reset hai phía |
| Peer lifecycle | Chưa xác nhận KR giống MTUDP VN | Discovery/key/slot/endpoint/reliability/relay và peer được sửa state nào |

Gate K5/K6 cần kiểm matrix và đối chứng của hành vi thực sự dùng; từ28/10 Codex tự kiểm theo miễn review của chủ dự án. Có thể giữ unknown về
server gốc thất lạc nếu policy emulator được chọn `design`, client chấp nhận và ca consistency đạt;
không thể để unknown về đường đồng bộ cần thiết rồi tuyên bố multiplayer đã nghiệm thu.
