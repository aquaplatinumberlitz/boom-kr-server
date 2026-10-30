# KR 00967 — contract tương thích client gốc

Revision kr-legacy-contract-r1; cập nhật30/10/2026 JST (CP025).
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
| Framing/primitive | VN `CClientSocket` `A68850`, recv/process `A662E0`, send `A663E0`; encode/decode/CRC đã có kiểm binary | Dùng làm mẫu tìm class/call graph và mẫu fixture. Patch bootstrap có profile riêng [đã kiểm](evidence/bootstrap-wire/README.md); KR có [profile WCP hẹp](#wcp) và native capture/field checks tới104/121 theo [contract start](#start-reply); full codec/reset/bounds còn mở. |
| Crypto/seq | VN XOR `0x53351D9C`/step `0x63CAACE3`, CRC8/32, base/seq/delta, opcode substitution | Constants chỉ là VN confirmed. Không gán cho KR khi chưa đọc consumer/đo. Kiểm seq tăng trước/sau crypto và reset riêng từng socket/chiều. |
| Hello/version | VN server-first hello, rồi đổi mode; version 116 và packet bootstrap có thứ tự | Patch socket KR: mode0/selector0, BE16 version967, response local được chấp nhận theo [evidence](evidence/bootstrap-wire/README.md). Consumer WCP, initial0/0 và transition2/3 có bằng chứng theo [contract WCP](#wcp); full codec/seq và login/world role chưa xác nhận; không gộp socket. |
| Login/identity | VN có pipeline login-result/network-info và variant China khác layout | CP013–014 có dispatcher/auth; CP015 đi qua redirect/request6 và UI sảnh hết notice trên copy. [Contract sảnh](#channel-lobby); full auth/profile/cache và vanilla vẫn thiếu. |
| Cache/redirect/channel | VN cần tag blob; dữ liệu inner LE khác outer wire, cache path khởi tạo theo stage | Kiểm contract KR từ reader/consumer. Giữ test cold/warm cache; không copy blob VN hay replay flag “dùng cache” khi file không tồn tại. |
| Lobby/room/start | VN có request/reply, slot flags, map tự gửi lúc vào phòng, loading/clock nhiều pha | Dùng để xây checklist ca biên. KR có stage11 và17/241 theo [contract sảnh](#channel-lobby); [request81/response86](#room-create) đã đưa copy tới CPrepareStage và UI phòng chờ; [player/ownslot/controls](#room-player) đã đo trên copy, [map metadata/start104](#room-map) và [start121/scene nhân vật](#start-reply), [clock/input](#round-go), [nổ/HELP/về phòng](#bomb-cycle) đã đo; bảng kết quả chọn đã có [CP023](#round-result), authority/reset đầy đủ/hai client còn thiếu. |
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
team0/1 và nhánh start, không phải hai client hoặc ready/team roundtrip. Chưa gửi request start.
Một lần trước đó (`room-teams`) wineboot timeout35s trước client, không tính là phép đo game;
retry prefix mới giữ nguyên copy/fixture/harness.
Tại CP018, Handler117/DC70D0 mới là locator tĩnh và chưa có map/load/Battle.
CP019 bên dưới nối table/ack/filter; auth/profile/cache hoàn chỉnh và hai client vẫn chưa có.

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
Không chuyển enum VN11/13 hoặc tự gán vị trí4039 thành spawn hợp lệ; các127/142/167 khác vẫn cần kiểm riêng.

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
[contract serial](#bomb-serial); raw/checkpoint CP024 giữ nguyên. Echo owner và ba byte hàng/cột/
power candidate vẫn được kiểm độc lập. Các tọa độ/spawn/profile dưới đây giữ phạm vi CP024.

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
|field4/5/6/u8 |field6/7/8/u8, payload+16/+17/+18 |hàng/cột/power candidate như CP024 |
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
### Request99 và entry bảng peer KR — CP025

**kr_runtime confirmed hẹp:** `E7F301→13E1470→13DE7C0` dùng hostname/host lookup lấy tối đa hai IPv4;
`E7F33C→13E14A0→14E5670` lấy u16 ở networkbase+24. Hai run có breakpoint bind đo cùng cổng
với native99 (9751/9345); còn có socket bind khác, không gán cả hai cho movement.
IP trong namespace=127.0.0.1/0. Chưa cấp peer nên không thấy sendto dưới hook đã cài;
không suy không có UDP hoặc đồng bộ movement từ quan sát âm này.

**kr_static confirmed cho cấu trúc/cạnh, áp dụng runtime candidate:** dispatch153 tạiE5D4CE→E5E6D0
**gọi E825D0 không qua guard703A80**. Trong E825D0, đoạnE838…/E839… đọc relay IPv4/u16port/u8flag;
IP khác0→htonl→13E1140→14E57B0. ĐoạnE839A2 lặp **26slot**, mỗi slot đọc u8on;
on→**3×(u32IP,u16port)** tạiE83AB2/E83AC9, đóng thành array3×8byte, kiểm1243330(slot),
rồi E83B69→13E1190(slot,array,localID,slotID).13E1190 có consumer địa chỉ và gọi14E57B0.
Đây là suffix trong **body dài chưa phục hồi đủ**, không được gửi suffix trần như full153.
Endian/runtime HostCreate và movement/UDP framing/key/relay chưa được nghiệm thu.

KR532 cũng tớiE825D0 quaE5E820 nhưng guard703A80 trên ba artifact đồng nhất trả AL=0;
đường này không phải ứng viên mặc định để thử. Chưa quan sát153 hoặc532 trên client trong các run này.
VN §5a-6 (92/146,18slot) giúp nhận ra pattern peer; KR99/153/26slot và guard được đọc riêng,
không dùng offset opcode hay số slot VN. Phép kiểm tiếp giữ ở [RESEARCH](RESEARCH.md#q-023).

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

### Authority matrix hiện hành và gate review

Bảng này tổng hợp bằng chứng từng phần đến CP025; chưa phải toàn contract server gốc. Mỗi hàng khi
triển khai phải bổ sung riêng **creator, simulator, validator, broadcaster, peer-editable state**, với
locator/trace và trạng thái từng ô; những vai trò không nêu dưới đây hiện `unknown` cho KR.

| Hành vi | Đã thấy trong KR | Phần cần đóng trước demo dùng hành vi đó |
|---|---|---|
| Movement | Consumer dir/position có phạm vi lớp;183 đã nối ghost | Input player→mover/network, peer update/correction và ai chấp nhận vị trí |
| Bomb place/ID | CP025 ba bom/serial u320/28/56, ID1..3 và lặp/cũ hẹp; tọa độ/spawn CP024 | ID allocator/rollover/concurrent/race/round generation, creator/validator/broadcast và peer còn thiếu |
| Explode/chain | CP022158/kind0 có nước/HELP; calc tĩnh/message27 giữ phạm vi riêng | Fuse/clock owner/relay và cách bỏ lặp chain còn thiếu |
| Block destroy/item reveal | Calc vùng nổ không tự xác nhận authority phá block/drop | Ai quyết block/item, state từng peer và packet/event đồng thuận |
| Item placement/pickup | Loader/receiver có mỏ neo, chưa đủ contract | Kind/index theo build, ai đặt item, xác nhận pickup/race và broadcast |
| Trap/death | CP023 nối writer127/sub17/19 và full envelope14+blob8/5; HELP/local biến mất có ảnh | Semantics inner, validator/broadcast/peer/timeout/loại trùng còn thiếu |
| Result/reset | CP023175 hiện LOSE/PERFECT theo rank chọn; reported19→175→180/full3 về UI phòng | Arbitration/round identity/serial và reset/start lặp hai phía còn mở |
| Peer lifecycle | Chưa xác nhận KR giống MTUDP VN | Discovery/key/slot/endpoint/reliability/relay và peer được sửa state nào |

Gate K5/K6 cần kiểm matrix và đối chứng của hành vi thực sự dùng; từ28/10 Codex tự kiểm theo miễn review của chủ dự án. Có thể giữ unknown về
server gốc thất lạc nếu policy emulator được chọn `design`, client chấp nhận và ca consistency đạt;
không thể để unknown về đường đồng bộ cần thiết rồi tuyên bố multiplayer đã nghiệm thu.
