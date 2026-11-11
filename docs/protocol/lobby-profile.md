# Sảnh, identity, cache và profile

[Mục lục](../PROTOCOL.md) · [Câu hỏi đang mở](../RESEARCH.md) · [Thứ tự việc](../PLAN.md#dashboard).

<a id="channel-lobby"></a>
### Từ auth tới UI sảnh trên copy — CP015

`kr_static/confirmed` cho reader/dispatcher đã đọc và `kr_runtime/confirmed` cho run ghi rõ ở
[evidence](../evidence/postwait/README.md). Các giá trị fixture là `emu_choice`, không phải dữ liệu
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

<a id="k3-version-identity-controls"></a>
### Version lỗi và ràng buộc identity local — CP186

[Nghiệm thu K3/scope](../PLAN.md#k3-acceptance), [nguồn/raw/hash](../evidence/lobby-identity/README.md#cp186).
**kr_runtime confirmed cho bootstrap ProcessPatch**, không là versionfield auth hoặc u16 WCP:

| Input/run | Quan sát và giới hạn |
|---|---|
| invalid-r1 | CLI chỉ file65535 nhưng actual serviceprofile vẫn fixture967; nativeguard1/equalbranch/return1. Control không tới client, không tính negative. |
| invalid-r2 | Profile riêng thực sự pin65535: `1236A02` remote65535/guard0, caller`1234875` return0; UI ErrCannotLogin, chỉ bootstrapconnection. |
| mismatch-r1 | Version966 qua guard1, `1236A5E` compare0, `1236AC8` PreparePatchreturn1. Hiện NGMPatch Process Error(967:1); chưa quan sát ProcessPatchreturn trước cleanup. Không suy crash hoặc login thành công. |
| mismatch-r2/r3 | Một OSACK trên Error modal; r2 nativeReturn0 nhưng xdotoolrc1 được giữ là tool ambiguity. R3 focus/keydown/keyup đều0, native`1234875` return0 nằm giữa timestamps input; sau đó ErrCannotLogin(5:1754). WindowPID129 và process list cùng run. Không suy ý nghĩa error5/1754 hoặc updater thành công. |

Chỉ thay BE16version reply+6; guard/opaque/emptystring/tail giữ. Cached ProcessPatch callsites tái dùng,
không ép branch/return hay gọi gamefunction. GDB script inherited có thể in phase sau remoteclose;
nhãn đó **không** là proof native bootstrapguard, chỉ dùng version-PC events thực sự đã hit.

**emu_choice/design** cho profile `own_profile.mode=native_291`: CP058/060 actualrequest6 có u16string
tại body+2, A/B43byte và C41byte. Guard chỉ kiểm boundedshape và tên tương ứng role đã pin, cùng
binding auth ID/name fail-fast CP184. Hai byte đầu và tail32byte còn opaque; không gọi đây là
account/token verification. Wrongname/shape logged/rejected trước17, không set identity/ready/seqmarker.
Profile default-off giữ nguyên. 294 servertests và8case AF_UNIX vào public`receive` dùng frame nativeframer
chấp nhận độc lập, wrongname string cũng được reader native đọc đúng; positivesA/B/C gửi17/43, next46.
Các input sai được dựng từ capture, không phải nativeclient tự gửi wrongcredential. RuntimeA warm
chứng minh guard cuối nhận actualKRLocal và vẫn tới sảnh/own291. Đây là snapshot CP186; actualAB291 đã đo [CP187](room-membership.md#current-291-room-integration), C291 còn unknown.

K3 nghiệm thu localprofile theo quyết định thủ công, **không** fullauth, mọiversion/updater,
Windowsnguyênbản hoặc fullmatch. Widermapping/cache/caption giữ question theo trigger trong RESEARCH.

<a id="channel-cold-route"></a>
### Cold cache route kênh và tên hàng — CP186

**kr_runtime confirmed** trên một A/Winecopy e9/offline; [recipe/hash](../evidence/lobby-identity/README.md#cp186).
Tái dùng CP016/034 channelblob/row/factory và CP059/185 SSI/CRC/528, không thân hàm native mới.
Trước `channel-cold-r1`, cả `channel2_000.ssd`, `fxdynmapinfo.ssd`, `fxstagemap.ssd` vắng trên copy
(stage-map đã vắng từ batch trước). Loader`14EA7C0` đọc channel0 hai lần, dynamic0 và stage0;
actual6→17/241/91/14/291, panelKRLocal, actual528→238/58 và native ghi dynamic76byte SHA6a4745… .
Native ghi channel54byte SHA`4c7204159c08ee70ffc18fed7f8ffc47099a5f8ab648b6fa05035a6f8939621a`.
Warm cùng copy nhận channel1 hai lần/dynamic1, stage0, không528 và không238; finalnameguard nhận actual6.
Năm SSI string/filter/fxmg/spcc giữ bytehash giống client nguồn; đây là cold **cache route**, không coldmọiasset/cache.
Ready241 đọc đúng sau blob14byte: cacheCRCdynamic`804314B9`, stageCRC0, u8=0, u32=0; không lấy8byte cuối
làm haiCRC vì lệch boundary. Stage-map không được khôi phục và chỉ có scope CRC0.

**emu_choice:** candidate channel14 đổi duy nhất string rỗng thành ASCII`KRLocal-0`, giữ rowtail/kind/index.
Cached body`13A20D0`/`13A1FB0` khớp từng byte trên e9. Native`13A21E7` trả1/name9byte đúng và
`13A2099` trả rows1 trong warmrun, sau phase lobbyready/fresh14. Không lặp full-row oracle lỗi CP034;
đây là object thật trong runtime. Không chứng minh tên caption đã vẽ hoặc warm cache đã phục hồi row
trước14; UI header vẫn không được gán là channelname. Gate baseline một sảnh normal dùng chứng cứ
selected0/level1 CP016 và kind1→normalfactory/UI CP034, không caption đa kênh.

<a id="lobby-cache-route"></a>
### Cache trước sảnh và service request528 — CP185

**kr_runtime confirmed cho recipe A/offline/Wine copy e9**, [run/hash/recipe](../evidence/cache-boundary/README.md#cp185).
Tái sử dụng CP059 parser/representation/CRC/payload và CP184 own291/counter; không đọc thân hàm native mới.
Loader`14EA7C0` từ debugger attach ghi15 cặp entry/return: chuỗi/static, channel2_000,
dynamic-map và stage-map; đây không là inventory mọi Win32file/API/cachefamily.

| Đối chứng | Quan sát native và giới hạn |
|---|---|
| Giữ cache | Dynamic-map và stage-map fileload1; auth→sảnh→own291, không528. |
| Bỏ đúng hai file trên copy, service CP184 | Cả hai fileload0; actual528/seq67/body0/CRCđúng; panelKRLocal vẫn hiện. Không238 và không tái tạo file trong65s. Đây là thiếu sót service, không bằng chứng client cần cache để hiện sảnh. |
| Cùng copy còn thiếu, service CP185 | Actual528→238/seq58 từ pinned compressed table; native handler`DFE9A0`→writer`DFEA58` trả1, tạo dynamiccache76byte SHA`6a4745c6010378a50ab4bf34827bbe122d7eb270c78051391cd21bd7378b1053`. |
| Warm sau run ghi file | Native`DFE8D0→DFE98D` trả1; không528, không238 được service gửi, panelKRLocal hiện. Stage-map vẫn vắng; các cache channel/static giữ. |

**emu_choice/design:** chỉ profile opt-in native291 bật đường528rỗng→table238 sau ready_response_sent.
Payload reuse chính representation CP059, dùng `send`/`s2c_next_seq` hiện có: own291/55→cache238/58→next61.
Warm giữ next58 vì không request. Sai login socket, trước ready hoặc body không hỗ trợ bị log/reject;
profile default-off giữ hành vi cũ. Không thêm server law/account authority hoặc tự gán ý nghĩa request có body.
292 model tests kiểm sequence sau replycache→create, rejection và default-off; native file/write/warm mới là oracle cache.

`count` trong probe mới là raw manager+14, **không sốrow**, như lưu ý CP059. `fxstagemap.ssd` vắng ở đây
chỉ dưới stageCRC0 của ready241; chưa chứng minh handler/request cho stageCRC khác0. Không toànclientcold,
corruptcache, nativeB/C, phòng/Battle trong batch này, versionerror hoặc K3gate. Giữ [checklist](../PLAN.md#k3-evidence-review).

<a id="dynamic-cache-representation"></a>
### Cache dynamic-map và checksum representation — CP059

**kr_runtime/kr_static confirmed trong scope một named-cache**, [source/recipe/hash](../evidence/cache-boundary/README.md).
Tái dùng CP019/DFE850/DFE8D0/DFEB10 và VN cache-method, giữ auth mới CP058. A/B dùng **hai game-dir
reflink riêng**; chỉ xóa `fxdynmapinfo.ssd` trên bản thử, các cache khác giữ lại. Không gọi fresh prefix
hoặc missing một file là cold toàn client; không sửa source binary/DB/pack/cache nguồn.

Nguồn lệch đã giải: `14EA7C0` kiểm SSI file-header checksum rồi `14EAB40→14E9560` nạp blob.
`14E9560` lưu `405E70(seed0, storageSelector+encodedRepresentation)` **trước** copy/inflate.
Selector0 =0+inner; với seed0 prefixzero không đổi checksum. Selector1 =1+u32LE uncompressedSize+zlib;
checksum **không** là Adler của plaintext sau inflate. Nativeblob oracle và cache/runtime xác nhận.

Ở profile ExVillage01/278,69byte inner của wire238 và cache **giống từng byte**; khác representation:
raw-wire checksum `8F400A8B`, compressed SSD `804314B9`. Cache76byte/SHA6a4745… có outerSSIchecksum
riêng `97201510`; không lẫn outerfile-integrity với checksum mà config241 quảng bá. Cả haiCRC là field
metadata, không auth/sessionkey. Không hardcode chúng thành hằng mọi map/profile.

| Run / mỗi role A/B | Namedfile/load | Sau config241 | Kết quả wire/cache |
|---|---|---|---|
| cache-present (raw241/238) |file có, actual804314B9≠expected8F400A8B, loader0|actual C→S528 rỗng|raw238 parsecount1; ghi compressedfile76byte.|
| cache-missing (cùng rawprofile) |file bị xóa, fileload0/loader0|actual528|raw238 ghi lại cùng file.|
| compressed-missing |file bị xóa, expected804314B9|actual528|selector1/238 nativeinflate69/CRCkhớp; ghi file76byte.|
| compressed-warm (freshprefix/file giữ) |fileload1/CRC804314B9 khớp; parse69/69/return1|**không528**|cache được chấp nhận trước238 tới; service vẫn gửi238 vô điều kiện sau đó.|

C→S528 tại các ca fallback là actual frame38byte/bodyrỗng/seq67 được nativeframer/CRC kiểm. Warm không
có request528 ở cả hai; không dùng chỉ processalive/TCPaccept để suy cache thành công. Rawtag
`CP59_CACHE_LOAD_END count` đọc manager+14 là **raw pointer chưa định nghĩa**, không là sốrow;
count1 lấy từ nativeu16/parser local ở DFEB8C. `CP59_CACHE_WRITE_BEGIN` tại DFE9A0 là handler238/update;
return14EA940=1 cộng snapshot file mới là proofghi. Warm thêm read-only parserexitprobe, không force return.

Fix servicefixture: gửi **exact compressed blob mà client đã ghi**, quảng bá checksum của representation
đó ở241; native238inflate69/241CRC/cursor/errors kiểm độc lập. Exporter đổi **ready/table**,39alias khác
kể cả auth giữ nguyên;41frames trùng profile đã đo. Không sửa codec/service hoặc cache fileformat/client.
Các file ghi rồi mở lại có cùng SHA; metadata/recipe dùng hash, không publish cache/assets/binary.

Bốn run đều authA/B137/errors0 nhưng gameplay/lifecycle còn partial, không chứng minh2trận hoặc K6a.
Fullprofile/cachefamily/cacheversion/corrupt/delete/reconnect/Windowsvanilla vẫn mở. Cauth mới tại
CP059 chỉ native, [CP060](#current-profile-c) đã đo integrationC; không đo lại CRC/cachefile này.

<a id="auth-profile"></a>
### Success/profile count1 đã đọc sạch trên copy — CP058

**kr_runtime/confirmed** cho callback/requests trong môi trường đo, **design** cho values fixture;
[recipe/control/hash/audit](../evidence/auth-profile/README.md), [chỉ mục56field](../evidence/auth-profile/field-shape.json).
Không đăng nhập server thật hoặc phục hồi authority/token/account verification. Tái dùng VNoc_d-login
thứ tự consumer, **KR106 put_str đã kiểm**, CP020 IDprefix, CP057 framer; không chuyển31field/layout VN.

**Sửa kết luận cũ:** CP02034byte gán đúng ID1 trước `10FD90B→13B09E0`, nhưng tên đầu tiên tại
`10FD8CE→packet.vf4C` **không** đọc sạch. `14EB9E0→vf70/14EBA90` đọc **u16 length + encoded bytes**.
Recipe cũ dùng u32zero (`96CA5395`) tại cursor29; native đọc thành length33A3hex, vượt payload34,
đặt packet+54=1 và không tiến cursor. Getteru8 kế đọc head204 từ byte96. Native replay oldA/B/C và
live A/B xác nhận; ID/ownmatch/các scene cũ vẫn là quan sát đúng, không biến chúng thành fullauth proof.
Empty string đúng là2byte typedu16zero; prefix qua name/head là **32+length(name)**, không34 cố định.

**Đường count1/feature đã đo:** outer resultu8=0 trước `10FD540`;53lần đọc trực tiếp gồm header và
một player, chỉ mục trỏ callsite/type/meaning đã có. Ordinal9u32 là local ID;10string là tên,11u8head;
12/13strings thứ hai/ba còn unknown semantics. Các field khác giữ tên trung tính; dùng0/empty là design,
không default wire gốc. Sau53fields còn consumer phụ: `10FEC8A→12F6A60→1386790` đọc countu32 tại125,
conditional AE25E0 đọc countu32 tại129; `53E230` đọc countu32 tại133 (offset với tên7byte trong ca này).
Ba count0 chỉ kiểm nhánh rỗng, chưa nonzeroentries/featureoff/playerCount2 hoặc profile thật.

| Run fixture / mỗi phía A/B | Callback cursor/payload | err54/58 | Giới hạn |
|---|---|---|---|
| auth-short (CP020 prefix) |34/34|1/0| Lỗi bắt đầu ở string/cursor29, cursor cuối bằng size không là parse thành công. |
| auth-full (sửa chuỗi+53fields) |125/125|1/0| Getter trực tiếp sạch nhưng thiếu consumer phụ, không gọi fullbody. |
| auth-tail (thêm2count) |133/133|1/0| Còn thiếu count53E230. |
| auth-final |137/137|0/0|53directfields+3count rỗng, hai role/count1/featureprofile hiện hành. |

Trace final có59getter entries vì3boolwrapper gọi lạiu8 bên trong; đó là **56field logic**, không59field
độc lập. A/B đều ID1/2, sau đó **actual C→S6/81byte** có tên KRLocal/KRGuest; nativeframer/CRC và
native string reader tại offset2 xác nhận. Chỉ chọn offset trong emulation để kiểm string, không fullop6schema.
Run180s có2Start/clock nhưng chỉ1rank/return, không hai trận đầy đủ hoặc K6a. Các run dùng shared game-dir/
cache đã có; fresh Wine prefix không là coldcache. Không thêm patch/force ID/state trong run.

Exporter41fixture nay chọn authA/B-final (137byte payload), authC-final (135byte/tên5byte) thay malformed
prefix; byte xuất khớp exact profile đã chạy. Tại CP058 **C mới chỉ native reader/emulation**; [CP060](#current-profile-c) đã đo runtimeC với
fixture mới. Không tự migrate profile JSON đã export; chúng giữ hash/path cũ cho reproducibility.
Service/codec không đổi. Fullmeaning/profilevalues/cache/cold-warm/missing/reconnect/token_2/allregions/
Windowsvanilla/authauthority/K2/K3/K5/K6a/LAN/Internet còn mở. AUD018 audit ngay lỗi string, cadence kếCP060.

<a id="local-name-model"></a>
### Local ID/name và guard panel sảnh — CP182

`kr_static/confirmed` cho tám body đã so byte với e9, `kr_runtime/confirmed` cho hai copy Wine:
setter`13B09E0` ghi ID theo slot tại manager+140h+4×index; `13B0A60` nhận tên, slotwrapper16byte
ở manager+18h+10h×index. Getter`13B0A00` chọn index0/1, gọi`1293080`: helper giải từng ký tự,
free/reallocate plaintextcache khi đọc, trảcharpointer; ngoài0/1 dùng fallback. Đây là scope code,
không accountauthority hoặc mọi encoding. VA/RVA theo base400000; metadata hash ở
[evidence đợt đang đo](../evidence/lobby-identity/in-progress.json).

Runtime sau auth: slot0 ID1, setter nhận7byteASCII`KRLocal`. Tại returnPC **13B0A4D** trước epilogue,
bảy observation thuộc sáu caller đều trả đúngbytesKRLocal, IDslot0=1/slot1=0. Actualrequest6 của cảhai
run byte-identical81byte/SHA`2cfebd3ad9a4935bb144090c4244c67ec9200ca54d4a9da71ecef00004a8bdf4`;
reuse nativeKRframer/stringreader tại offset2 đọcKRLocal/errors0. Không giả tên bị mất trong auth.
`identity-r1` dùng GDBFinishBreakpoint cólowEAX/stackdata nên giữcandidate/excluded khỏi kết luận
getternames; `identity-r2` dùng exactreturnPC và helperbody để loại nhiễu phép đo, không sửa raw.

Consumer đã đối chiếu: `1229DA0` dùng tên làm keyDenyWhisper; `13D41E0` tạo request544 chứa tên;
`13AE740` dùng tên/ID trong connectionrecord. Chúng không chứng minh own-panel sảnh đã render; consumer panel được nối ở đoạn CP182 bên dưới.
Candidategetterxref`DA8A70` có nhánh Tournament và không được quan sát trong run; không suy UIđích
chỉ từ vùngaddress. Hai lượt đầu/8body chưa xác định consumer own-panel; nhánh UI runtime tiếp dưới đây, không mở mọi186xref.
VN loginreference xác nhận phương phápprimaryname→op6/settings/whisper; str2shop/str3unknown
không được chuyển thành nickname KR. Không đổi field/packet tùy đoán. UIidentity/cachematrix/K3
vẫn mở; [câu hỏi tiếp](../RESEARCH.md#gap-lobby-identity-followup), PLAN giữ thứ tự duy nhất.

**Consumer panel đã nối bằng runtime CP182.** VN `lobby.md` (mục danh sách người chơi trái)
phân biệt store danh sách với auth manager, nhưng hai ô cấp/kinh nghiệm bản thân vẫn chưa kiểm ở VN.
Dùng lại cách phân biệt data-source, không chuyển opcode/ID/layout. KR RTTI/static xác định
`CLobbySquareLeftSideUI`; run `panel-r1` chỉ thấy `CLobbyRoomUIWnd` constructor, không thấy các
callback Square đã chọn trong60s. Quan sát âm có giới hạn, không chứng minh class vắng mặt.
`panel-r2` tại ready`DA5F90` xác nhận widget **`CLobbyStage+5E5D8 = CUserListUIWnd`**, vtable172B748.

Static byte-equal với copye9: virtual`901390` gọi **`9022D0`** để vẽ record bản thân; lookup
`7FA5D0→global19DB454`, `D2FE90(store,index)=store+6404h+220h×index` (544byte/record).
Tại`902312` có record thì đọc getter529C60 trên **record+1FCh**; tại`902329`, giá trị0 đi nhánh
`90232D→902A32`, bỏ phần vẽ tên/cấp/exp. Nhánh khác đọc std::string ở record+4 để vẽ tên,
không gọi auth getter13B0A00. Ý nghĩa tổng quát của field+1FC chưa đặt tên hoặc xem là accountID.

`panel-r4`, sau ready: hai hit từcaller9013BE tại902329 có đúng widget stage+5E5D8,
selected0, recordkhác0, **decodedguard=0**. Hai dump544byte có std::string record+4
size0/cap15. Đường auth/getter đúngKRLocal và recordpanel rỗng là hai nguồn dữ liệu riêng;
tại CP182 chưa biết producer; [CP183](#own-profile-291) bên dưới đã nối S→C291, không sửa record/raw cũ.
Probe string gặp lỗi đọc0byte trong GDB; raw SSO size/cap xác nhận rỗng, không coi là clienterror.
`D7A260` tĩnh là consumer đổi level có so tên auth rồi truy record này; chưa xác định producer
profile tổng quát hoặc opcode/contract để gửi bổ sung.

Controls giữ nguyên: `panel-r3` đặt probe trướcready dừng C++exception/SIGABRT trướcDA5F90,
channel chỉ gửi6; harness exit0 chỉ là cleanup. `panel-r4` hoãn probe tớiready có observation trên;
không suy installationtiming là nguyên nhân/fix. PrivateDB load inputhashNone có log riêng;
freshcopy analysis được dùng, không sửa DB/binary nguồn. Decompiler badSP/None chỉ là gợi ý,
assembly và16body-bytecomparison giữ phạm vi. [Recipe/run/hash](../evidence/lobby-identity/README.md).
Tại CP182 K3 vẫn mở; CP186 nghiệm thu localprofile sau cold-route/version/identity controls, bước tiếp duy nhất ở PLAN.

<a id="own-profile-291"></a>
### Profile bản thân ở sảnh: S→C291 → CUserMap → panel — CP183

**`kr_static/confirmed` và `kr_runtime/confirmed` trong copye9/Wine/offline**;
[recipe/run/hash](../evidence/lobby-identity/README.md#cp183). Tái dùng VNUserMap/list-vs-ownpanel để
chọn vùngstore; opcode291/record544/layout/consumer dưới đây đọc từ **KR**, không chuyển từ VN.

Dispatcher`D68C90` case291/callsiteD68ED8→`D73BB0`→`D2FEB0` cập nhật store global19DB454.
D2FEB0 duyệt sốslotlocal từ13B07C0; với từngslot đọc u8active. active0 bỏ phần còn lại của slot;
active!=0 đọc blob, reset record tạistore+6404h+220h×slot rồi virtual+8 deserialize qua`D30BF0→AE6330`.
AE6330 có cùng thứ tựfieldreader với profile109KR đã đo CP018/D2D310; reuse **inner109 nguyên bytes**,
SHA`a208ec5e86cf01ca513c1330708ebc5ac16f8966a2cf6e6ee11f95b6a90ed065`, không copylayoutmemory.
D30BF0 sau đọc profile đi qua renderhelper và ghi1 vào boxguardrecord+1FC; đây là **cờ đã nạp
trong consumer này**, không accountID. Recordtên std::string+4.

Với active slot, sau blob còn u32→record+208box (consumerexp), u8→record+214box;
feature106 (`6CF9C0`) thêm3u8→13E4C40/70/A0. Feature48 (`7A0180`) chỉ cập nhật state từ record,
không thêm bytewire ở consumer này. Những numericsemantic ngoài consumer giữ unknown;
profile/scalars0 là lựa chọnemulator. Các nhánhfeature khác/multilocal/count2/encodingnonASCII chưa đo.

| Phép thử trong cùng run `producer-r1` | Native/store/UI |
|---|---|
| Trướcinject | actualownwidget stage+5E5D8, renderguard0 vàtênrỗng |
| Control291seq55/u8active0 | payload1/cursor1/error54=error58=0; recordname vẫnrỗng |
| Positive291seq58/u8active1/blob109/u32/u8/3u8 | payload128/cursor128/error54=error58=0; record544byte cóKRLocal |
| Render saubpositive | Haihit902329/caller9013BE guard1, đúngwidget/nameKRLocal; ảnhscreen19 hiệnKRLocal/Lv.0/00.0% |

Actualinput qua socketchannel, không gọi gamefunction hoặc viết record/guard bằng debugger. Native
consumer/cursor/rawrecord/ảnh là bằng chứng độc lập, không lấy codecroundtrip làmoracle. Control→positive
chứng minh route input này đủ điền record/vẽ tên trên profile được đo; không chứng minh servergốc
phải dùng schedule2giây, producerkhác vắngmặt hoặc mọi field là0. Run60s, cleanup/hash/inputfreezeđủ.

Labservice chỉ gửi thêm hai frame sau bộready241/room91/channel14, productionservice/47fixtures
không đổi. **Chưa nối create/map sau khi thêm291**: các fixtureinitseq55..70 và dynamic73 phải được
đối chiếu lại khi integration, không chènraw rồi tái dùng sequence. K3/cachematrix/wrongstage/version/
identityauthority/Windows/LAN/Internet/K6a vẫn mở. Bước tiếp duy nhất [PLAN](../PLAN.md#current-order).

<a id="own-profile-integration"></a>
### Opt-in identity291 và sequence sảnh → phòng/map — CP184

`design/emu_choice` cho policy service, `kr_runtime/confirmed` cho hai run A được ghi rõ trong
[evidence](../evidence/lobby-identity/README.md#cp184). `--own-profile-291` của exporter thêm
`own_profile={mode:native_291}`; không đổi pin47 fixture hoặc mặc định của preset cũ.
`server/own_profile.py` chỉ hỗ trợ auth một local player, ID/name role A1/KRLocal, B2/KRGuest,
C3/KRNew và inner profile zero-valued đã đo. Unsupported identity/shape/profile bị từ chối trước listen.

Helper lấy blob từ player106 đã pin, giữ98byte tail profile đã đo, thay riêng stringLE tên bằng
field name auth đã kiểm CP058 (ID ở body25, u16 string ở29). C room fixture cũ là KRLocal dù authC
là KRNew; sao chép mù blob sẽ sai tên. Model xác nhận A/B payload128 và C126; **runtime mới A**.
Đây là binding identity cục bộ theo fixture, không auth server Nexon hay xác thực account gốc.

Opt-in dùng bộ đếm `s2c_next_seq`/send/send_setup đã có từ channel17seq43, ready46/list49/channel14seq52,
own291seq55, create86seq58/player106seq61/slot110seq64/guest106seq67/table238seq70/map117seq73,
next76 cho dynamic. Không thêm tracker sequence song song; reentry dùng tiếp counter hiện tại.
Default-off giữ nguyên bytes/schedule cũ, có regression. Parser guard/log sai stage/socket là design;
model kiểm receive/dispatch với bytes hợp lệ, không suy law hoặc runtime negative KR.

`integrated-A-r1` đã native name/ownrender rồi OS click-create/confirm tới room/map.
Sau cleanup/hash mới thêm guard/log, giữ source phiên bản r1 trong local; `integrated-A-r2`
đo lại đúng code có guard, cùng route/native framer ret0/sequence43…73 và producer291128/128/error0.
Framer exit cursor0 là **trước dispatch body**, không chứng nhận fullbody của mọi packet;
291 có probe consumer riêng. Ảnh trước create hiện KRLocal; ảnh cuối room hiện ID/name/map.
Guest trong ảnh là fixture tổng hợp, không client B thật. Hai run80s, inheritedcache, không game/start/rejoin.

290 server tests/7 exporter tests và392 quan hệ raw/source/native kiểm scope này; không đóng gate.
Các lỗi gọi test sai module/importpath được giữ, discovery đúng đã pass. Probe tái dùng đổi prefix
producer sang K3_FRAMER do global note, kind vẫn phân biệt; raw không sửa. [Checklist K3](../PLAN.md#k3-evidence-review)
giữ phần thiếu cache/version/channel/negative, không mở lại item/mode/race ngoài gate.

<a id="own-profile-result-compare"></a>
### Own291 mở đường result-name compare và sửa import trên copy — CP187

**kr_static/kr_runtime confirmed trong scope copy**, [recipe/control/hash](../evidence/lobby-identity/README.md#cp187).
R1 currentAB/e9 đi sạch291/panel/hostBack/rejoin/load/first175302/302 rồi cảhai SIGSEGV/kernelbase,
stack`D2F64A→E5FC07`. Đây là partialrun210.898s/exit0/854raw/frozen168/cleanup0; exit0 không resultpass.
Tái dùng CP183`D2F440`: ownrecord+1FC loadedguard1 mới cho phép compare tại`D2F644`(hai pointer strings).
Fullcaller`E5F610` đọc thêm có mục tiêu; rankfixture không được sửa. Oldguard0 từng bỏ qua nhánh này.

RebuiltIAT`16EA1CC` bị bind trực tiếp **CompareStringA(6args)**, trong khi originaldumpedthunk
`67B00AD` dựnglocale400/flags40000001/len-1 trước gọiCompareStringA, rồi chuyểnreturn1/2/3→-1/0/1,
callee-cleans8. Export gốc `kernel32!lstrcmpiA/7774EAA0` có cùng argument/returnrelation trong oracle;
APIreturn0 đều dừng tại SEHhelper chưa bao phủ. Resolver cũ `wine-imports-r4.json` ghi stackdelta−40 và
candidate endpoint, **không proof directABI**. Lỗi kiểu này tương tự bài học lstrcpyA16EA160, không quét toàn IAT.

**emu_choice:** `tools/patch_wine_compare.py` chỉ nhận privatecopySHAe9bb… và tạo file mới
SHA`4b781fbcea21e0f6294c729b63be86ab2953d60ad83b0d0c9219359f826107ed`, đổi hint/name tại
fileoffset31857478 từCompareStringA sanglstrcmpiA; `.image`31838208byte giữ nguyên,14byte khác ở `.repair`.
Không sửa guard/result/name/gamelogic, không overwritecopye9/source. Tool tái tạo đúngSHA và từchối
input khác/alreadycorrected/outputngoàilocal/outputđãcó; rollback dùng lại filee9. Không sửa recipeR1/R2 lịch sử.

R2 cùng profile/server/inputrecipe, thêm passivecompareprobes: actualstringsKRLocal/KRGuest hợp lệ,
nativecomparisonreturn0/±1 đúngsign và sp+8. Qua175/180 một lần trên cảhai rồi Start2; không lặp SIGSEGV.
Đây là control runtime cho ABIcorrection, không chứng minhmọiAPIfailure/locale hoặc originalWindows.

<a id="current-profile-c"></a>
### Integration C với profile auth/cache mới — CP060

**kr_runtime/confirmed trong Winecopy**, [recipe/raw/hash/giới hạn](../evidence/current-profile-c/README.md).
Dùng profile41 do exporter CP059 tạo, không thêm packet/schema/patch/servicecode. Tái dùng CP054
AEOFtrướcround1→Bcleanup→C3/ownmatch/JoinReady/BStart, không làm controlEOF mới hoặc gọi C là Aresume.
A/B riênggame-dir CP059, C reflinkcopy riêng; cache hiện hữu được nhận, không cold/missingcacheC proof.

Trong cùng run12actualconnections: A/B auth137/137, C **135/135/errors54=0/errors58=0**, ID3 qua
native setter. ActualC6/79byte có **KRNew** (A/B6/81byte KRLocal/KRGuest), nativeframer/CRC/stringgetter
xác nhận. C106/ownmatch1/slot0, actual82/95, Bactual104. C/B namedcache loader1/cursor69/không528;
đó là localprofile/parser/cachepath, không credential/serverauthority/accountverification.

Chỉ một generation1/key12345, rosterB(ID2/slot1)/C(ID3/slot0); B118→globalbombID1→159/158,
B17/19→C170/C21. Secure actorBstate1 trên cả B/C **trước175**, actorC không bị gán chết từ rank.
175callback302/errors0,180callback3/errors0, roomrefresh Bchief/Cguest. A đã đóng trước round đầu,
không có A/Bround/bomb/forfeit trước đó. Đây là **một vòng B/C**, chưa hai trận nghiệm thu/K5/K6a/LAN.

Run có một serviceerror Bchannel `Connection reset by peer` ở đoạn cuối: timestamp sau175/180/
refresh/final-roomcapture, <1s trướccontrollerend; C EOF ngay trước. Giữ raw/error, không gọi run error-free,
không suy đây là gameplaydisconnect hoặc đã nghiệm thu reset recovery. Không sửa error để làm checkpass.
759nativeframer cases,135packet/6native153/210secure snapshots và1865relations chỉ kiểm bằng chứng hẹp.
No newstaticbody; originalsourcehash giữ. Theo PLAN tiếp hai vòng A/B với **currentprofile** trong cùngrun;
AUD019periodicCP060, cadence kếCP064. Gate/auth/cache/validator/reset tổng quát vẫn mở.

<a id="rust-local-lobby"></a>
### Rust diagnostic identity → channel → sảnh → profile291 — CP232

[Code/cách chạy](../../rust/README.md), [raw/hash](../evidence/rust-service/README.md#cp232),
[metadata](../evidence/rust-service/local-lobby.json). Đây là **M2partial**, không fullM2 hoặc fullauthentication.
Reuse CP058 auth137, CP183/184291 producer/render, CP185/186 channel/sảnh và CP231 foundation;
Python `own_profile`/`probe_service`/47fixturepins giữ reference, không RE nativefunction mới/portserverVN.

**`design` implementation:** opt-in `diagnostic_local_lobby`, mutually exclusive với error-only M1profile.
Typed Flow stage Login→Info→Redirect→LoginDone và Channel→Ready→Lobby; kiểm shape trước advance,
wrongstage/duplicate không advance. Fixed identity1/KRLocal/name/ID trong C2S6 là diagnostic binding,
login input chỉ shape/tokenlength branch known, **không xác thực credential/provider/account**.
Hai WCPtransports đầu/transport3login/4channel là lab recipe single-client, chưa dùng cho2actual hoặc production.

Pin raw/header/body/seq của auth4/43, netinfo15/46, flags62/49, redirect7/52, channel17/43,
ready241/46, list91/49, channel14/52;291body128 từ measuredzero-profile builder, opaqueheader từplayer106.
Rust load/check every size/SHA/CRC/opcode/bodylength trước bind, mỗi connection S2Cseq43+3n riêng;
không replay nguyên frame khi seq đổi hoặc diễn giải field unknown. Ancillary knownshapes ignore có budget256,
không silentlygrant unknown request. Runtime chỉ response nhỏ/bounded foundation M1 giữ; chưa genericwriterqueue/load.

**`kr_runtime/confirmed` hai build location:** stagingR2 và rootbinaryR3 đều actualKR/e9copy/Wine10.0,
privateuser-net-mount-PID/lo-only/noIPv4route/files-onlyNSS/inheritedglobalreset, không patch mới/callfunction.
Native auth result0/global0, reader137/137/errors0; actual9→netinfo/flags,5→redirect, newchannel6→17,
actual67→241/91/14/291. Own291 producer128/128/errors0, storednameKRLocal, personalrenderguard1/matchingwidget
và UI normalLobby KRLocal/level0/EXP0/sixemptycards. FinalR3 PCAP160record/9exactS2Creplies và mọiactualC2S
seq/op/len khớp Rustlog, hold9.362070214s sau nativeprofile/render. Không suy socketalive hoặc login-result thànhfullauth.
Rootbinary SHA`6d758286c51025a05acf25b4b4d3ff2ebc4ca119dc8471e74c8af58f03f7dde6` là bản final đã chạy;
staging buildpath khác cho hash khác, giữ pin riêng, không gán stagingbinarySHA cho rootbuild.

**Control/correction:** R1hardcoded netinfoopcode18 trong validator nhưng fixtureKRpin là15 (đã có trongsourcefirst).
Fail beforelisten/service-ready/chưa launchclient;0.22990233s/6raw, không clientfailure/REblocker. NewR2 sửa đúng
declaration theo knownbytes, không fixture/opcodeguess;38raw positive. R3rootbinary47raw positive; tổng3run91raw/
frozen/seal/ownedcleanup0.7Rusttests (5wireoracle/designregression +2state/identitynegatives) và14kernelTCP M1
regression pass; nativeproof riêng. Inputversioncopies/raw/control preserved, no source/config change trongrun.

M2 còn hai actual identities/fixedroom/ownslot/host/Ready/loadingbarrier/Back-rejoin và negative lifecycle trênRust;
chưa Battle/UDP/DB/accountownership/fullauth/roomregistry/fullservice/production. [PLAN](../PLAN.md#current-order)
tiếp explicitdiagnostic bindings trước2actual, không mở authSDK/generalroom/broadRE. Pythonserving/RE/reference
và8WIP giữ; CP231PR302mergedbc55356, AUD067periodicqua232/kế236. Internet thật chưa kiểm.

<a id="rust-two-lobby"></a>
### Rust hai diagnostic identity với binding theo request/endpoint/stage — CP233

[Code/run](../../rust/README.md), [raw/hash](../evidence/rust-service/README.md#cp233),
[metadata](../evidence/rust-service/two-lobby.json). M2partial nối tiếp [CP232](#rust-local-lobby), chưa phòng hoặc fullauth.
Tái dùng CP058 authA/B, CP183291, CP186channel/sảnh, CP006WCPtype0/body28 và correctionCP103WCPportbiếnđổi;
Python `own_profile`/`channel_redirect_frame` giữ reference. Không RE thêm hàm/SDK/clientpatch.

**`design` policy:** opt-in `diagnostic_two_lobby`, mutuallyexclusive với M1 và singlelobby. Config bind/allowlist chỉ
loopback/RFC1918; hai actor pin ID1/KRLocal và ID2/KRGuest, nguồnIP khác nhau. Bootstrap15 gửi trước; socket rỗng
đóng không claimidentity. Request đầu phải frameCRC/seq40 hợp lệ: type0/body28→WCP317 opaque trên candidateports;
1→login chỉ đúng loginport;6→channel chỉ đúng channelport. Không còn global/per-actor acceptordinal để chọn identity
hoặc stage. WCPbody chưa genericparser, một validrequest/actor/process là labbound; profile cũ giữ scope lịch sử.

Login/Channel dùng typedFlow CP232 với ID/name theoactor. Registry giữ generation, một login/channel reservation;
redirect grant10s chỉ sau gửi7 thành công; channel chỉ bind sau C2S6 khớpID/tên rồi consumegrant. Sai identity chưa
consumegrant; channelboundEOF/duplicatepacketclose thu hồi session, loginEOF sau redirect giữgrant tớiTTL.
LeaseDrop đối chiếu generation/connection để không xóa owner mới. Các quy tắc này là localdesign, **IP không xác
thực account/credential**; không password/provider/sessionsecurity/DB/resume. Flow vẫn boundedtimeouts/ancillarybudget.

**`kr_runtime/confirmed`:** sameRustservice, hai actuale9copy/Wine10 trên hai netns/veth + bridge10.77.0.1,
A10.77.0.2/B10.77.0.3, ba networknamespace khác nhau/2CA/2wineserver/no default routes/files-onlyNSS.
Hai nativeauth137/137/result0/global0/errors0, C2S6 tự echo ID1/KRLocal và ID2/KRGuest; actual67 tới241/empty91/14/291.
Mỗi producer291128/128/errors0 lưu đúngtên, personalrenderguard1/matchingwidget; UI screen14 cảhai đúngtên.
PCAP162record: támtuplestreams hoàn chỉnh/18exactS2Creplies/allC2Sseq-op-len khớpRustlog; WCPtype0 thật đã dùng3838,
không nhầm với login1 cũng3838. Hold8.710398283s sau cảhaiprofile/render, rootbinarySHA
`bdf0c70d5a5366fd40d6e071e2f87f7660ab05881799880a6343b28148362d8e` không đổi sau finalbuild.

**Failure/correction:** R1portonlyWCP3848 giảđịnh sai: nativeWCP0/body28 tới3838 và bịroutelogin;193raw/FAIL giữ.
Seal/cleanup trước sửa. R2classifyfirstrequest theo contract đã có,98rawpositive; tổng291raw frozen/sealed/cleanup0.
UI review banđầu chọn screen9 bằngstringorder/Bstartupblack, sau sửa chọn numericlatest14/cảhaiLobby; raw giữ nguyên.
10Rusttests +16kernelTCPbindingnegatives +14M1regression đạt; syntheticinputs không independentnativeoracle.
REA packetinspection không có TSharkproviderconfigured; ordinary retainedstdlibparser kiểm selectedunfragmentedIPv4TCP
coverage/overlap/seq/CRC/bytejoins, không gán là REAEvidence. Native observer/PCAP/UI có nguồn riêng.

**Còn thiếu:** fixedroom/ownslot/host/Ready/loadingbarrier/Back-rejoin và native lifecyclephòng trênRust;
chưa Battle/UDP/DB/accountauthentication/generalroom/fullM2/fullservice/production. K0–K7 cũ giữ đúngscope.
[PLAN](../PLAN.md#current-order) tiếp portfixedroom, không thêm broadRE; Pythonserving/reference và8WIP byteexact.
CP232PR303merge5b3ef81, AUD067qua232/audit kếCP236. **Internet thật chưa kiểm**.

<!-- protocol-topic-content-end -->
