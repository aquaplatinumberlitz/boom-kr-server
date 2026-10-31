# Câu hỏi RE của server KR

Đây là backlog riêng KRE. Các ID Q-022…Q-029 được giữ để truy nguồn; không thuộc gate V1 Rust.


Mở theo yêu cầu trực tiếp 26/10/2026. Đây là câu hỏi của **`KR-00967-EMU-1`**, không đổi câu hỏi/gate
V1 Rust. [Lộ trình KRE](PLAN.md) giữ thứ tự
task và điểm dừng; [contract](PROTOCOL.md) giữ ma trận/nguồn.
KRE-001/002/003 đang thực hiện: từ28/10 Codex solo RE/runtime và tự kiểm theo
[miễn review của chủ dự án](evidence/checkpoints/review-waiver.json). Report dsh/Claude cũ giữ lịch sử.
Review plan nguồn đã xong; các câu hỏi dưới đây vẫn open cho tới khi có evidence trả lời.

Cập nhật CP001: contract WCP ở [PROTOCOL](PROTOCOL.md#wcp),raw/review/integration ở
[evidence WCP](evidence/wcp/README.md). Kết luận “socket hai chưa nhận reply” chỉ áp cho
[ba run R2 silent](evidence/wine-r2/README.md),không còn là trạng thái mới nhất.
KRE-004/issue6 đã đóng trong scope Wine r1;KRE-005a/issue9 còn mở cho bước sau.

<a id="q-023"></a>
Q-023, CP006 (review theo [manifest](evidence/checkpoints/manifest.json)): `multiserveridx:0`
trong run copy đã kích hoạt setter `+E78`, tránh nhánh launcher và cho request đầu C→S;
`oneclickuser` liên hệ setter `+E9C` mới chỉ static, chưa đo runtime. Tám fixture S→C mode2/3
đưa guard trả1; hai copy thử NMCO lần lượt tới import init, status và `NMCOGetMyIdCode`.
Nguồn/giới hạn: [PROTOCOL](PROTOCOL.md#wcp), [evidence](evidence/wcp-sequence/README.md).

CP007: copy thay identity EDX:EAX bằng `0:1`, screen name rỗng, sửa heap guard và thử các import
cửa sổ; hai run debug dừng tại cùng thông báo Wine `mmdevapi` assertion. CP008: run plain với ALSA
null chạy audio và sau các import thử dừng tại slot `16EA638`. Khác biệt run chưa chứng minh debugger
gây assertion. Chi tiết/giới hạn tại [evidence](evidence/nmco-identity/README.md); review theo manifest.

CP009: thử tiếp message loop và thread import, client vẽ màn phân loại tuổi; run plain-thread
gặp lỗi ghi tại `45E920` tới địa chỉ `02677C9C`. Pointer/caller và nguyên nhân chưa xác định.
Chi tiết tại [evidence](evidence/nmco-identity/README.md).

CP010: guard của object chứa pointer cũ đã được thử reset; snapshot trực tiếp cho thấy pointer
được khởi tạo lại. Pre-throw snapshot xác định nhánh yêu cầu `CAWH.sys`; thử bỏ hàm cài hook và
các binding API tiếp theo đưa copy tới marker khởi tạo, rồi trap overlay ordinal9. Nguồn chuẩn
cho run/recipe/giới hạn: [evidence object state](evidence/object-state/README.md).

CP011: bỏ call overlay candidate ordinal9 trên copy cho UI login đầy đủ; hai phép click/type
(thường và giữ nút lâu hơn) chưa quan sát ô nhập thay đổi hoặc C→S mới. Callback `CAWH.sys`
trên host chỉ được đọc tĩnh, chưa xác nhận vai trò với hiện tượng input. Nguồn/giới hạn tại
[evidence UI](evidence/login-ui/README.md); cùng CP011 bổ sung provenance theo review CP010.

Áp dụng kiến thức VN trong CP011: GDB proxy bắt guard/IAT; run tắt audio trong prefix riêng
đo192 message chuột/phím tại DispatchMessage, nhưng chưa thấy nội dung ô nhập đổi hoặc C→S mới.
Nguồn và giới hạn tại [VN→KR](evidence/vn-reuse/README.md). Chưa chứng minh custom handler nhận
đúng control; GetAsyncKeyState vẫn là candidate KR, so VN chưa đủ để đổi thành GetKeyState.

CP012: đã nối mouse/key qua handler và click tới command1002. Hai tọa độ ID/password chưa
cho edit control; phím được chuyển tới button, không suy toàn bộ input hỏng. Đã tìm hostname login
riêng; run có ánh xạ local mở kết nối3 trước click, WCP poll/consumer trả1. Sau đó vào attempt
`11049C0` từ return`11013FE`, modal `CWaitForLoginDlg` trả1, chưa vào nhánh result3; attempt EAX0.
`modal-result` nối setter1 tới nhánh kiểm global1A966FC; bản dump có giá trị1. Reset riêng global
trong bộ nhớ run mới giữ được dialog đếm chờ, không thấy return trong cửa sổ đo.
Hai run reset thu276 byte mới=4 frame C→S15, nối guard→virtual→WinSock/capture; native parser
replay với seq40+3n khớp CRC cho bốn mẫu, negatives từ chối. Chưa xác nhận semantics auth.
Raw/recipe/timeline/giới hạn tại [evidence handler](evidence/ui-handler/README.md).

CP013 đối chiếu VN: static dispatcher KR đã nối S→C4/result0 và S→C15 tới các routine wait0→2→1,
rồi tick1→3; offset state KR+F4 khác VN+D8. Nguồn/call và giới hạn tại
[evidence VN→KR](evidence/vn-reuse/README.md). Chưa quan sát runtime hai handler hoặc đủ payload;
không dùng code VN `login_ok()` nguyên xi để đóng contract KR.

CP014 đã tìm trigger auth từ token local, capture request và thực thi handler phản hồi lỗi;
contract/chiều seq/prefix result0 tại [PROTOCOL](PROTOCOL.md#login-auth), control và các phép đo
bị loại ở [evidence login](evidence/login-runtime/README.md). CP013 đã merge PR21.
Nguồn count0 của auto-attempt đã giải; lifecycle kích hoạt stage tự động và global vẫn chưa đủ.

CP015 đã qua helper và nối flags62 → request5 → redirect/request6 → stage11/CLobbyStage;
S→C241 đóng notice, UI sảnh đã hiển thị trên copy. Contract/giới hạn ở
[PROTOCOL](PROTOCOL.md#channel-lobby), raw/control ở [evidence](evidence/postwait/README.md).
CP014 đã merge PR22. Không suy auth/profile đủ hoặc K3 từ UI này.

CP016 đã nối click → command2011 → guard91/bảng kênh14 → CCreateDlgRenew → capture81/107byte.
Contract và giới hạn ở [PROTOCOL](PROTOCOL.md#room-create), raw/control/correction tại
[evidence](evidence/room-input/README.md). CP015 đã merge PR23. Profile còn thiếu và chưa có phòng.

CP017 đã đo S→C86/result0 → strategy thật → CPrepareStage/stage12 và UI phòng chờ tám ô trống.
S→C92 bị region guard trong run, không phải đường success dùng cho cấu hình này. Contract duy nhất tại
[PROTOCOL](PROTOCOL.md#room-create), phép đo/correction tại [evidence](evidence/room-reply/README.md).
CP016 đã merge PR24; CP017 là thí nghiệm trên copy, tự kiểm/miễn review, tích hợp theo PR.

CP018 áp dụng VN đúng phạm vi: player/profile, own-ID match và unlock controls giúp chọn probe;
KR106/110/schema được phục hồi riêng. [Contract](PROTOCOL.md#room-player),
[raw/negative/correction](evidence/room-player/README.md). CP017 đã merge PR25.
Player `KRLocal` đã hiện; ID1 không match local list0, ID0 match own_slot1;110/pending0 cho nút Bắt đầu.
Hai import API candidate được bind thật trên copy mới. Full auth/identity vẫn thiếu, không coi ID0 là contract server.

CP019 đã nối writer96, config241/checksum, table238 và ack117; map capacity4 và filter mode0
đã qua bằng field+5D có control. Click thật tự gửi104/109byte, send1/pending3. Contract duy nhất ở
[PROTOCOL](PROTOCOL.md#room-map), [raw/correction](evidence/room-map/README.md).
CP018 đã merge PR26. Đây là mốc request start, chưa có loading/Battle hoặc gate mới.

CP020 nối start104 tới receiver KR121, slot validation và init CGameStage/stage13.
Auth prefix mở rộng có trường ID1, consumer tự cập nhật local-user list; player1 match own-slot.
ID0 workaround cũ không đủ để tham gia start. Contract/layout/correction tại
[PROTOCOL](PROTOCOL.md#start-reply), raw/native/control tại [evidence](evidence/start-reply/README.md).
CP019 đã merge PR27; CP020 có scene map/HUD/hai nhân vật trên diagnostic copy, chưa gameplay hoặc gate mới.

CP021 đã nối actual99 tới writer/init, đo197/196/mode0/198, clock/input và bom hiện sau118/159.
Nguồn chuẩn [PROTOCOL](PROTOCOL.md#round-go), raw/control tại [evidence](evidence/round-go/README.md).
CP020/PR28 đã merge. Đây là bước clock/di chuyển trên một client có guest tổng hợp, chưa vòng trận.

CP022 đã phân biệt calcE9C520 với nhánh158/kind0→E9E660; burst có nước/HELP và local actor
biến mất sau đó.180/full3 được đọc3/3/errors0 rồi stage12/UI phòng.
[Contract](PROTOCOL.md#bomb-cycle), [raw/correction](evidence/bomb-calc/README.md).
CP023 đã nối writer chungB53010 cho127/sub17/19 và giải full prefix14+blob8/5 bằng native;
175/292rawbyte đọc đủ và UI có LOSE/PERFECT. Stub chuyển từ fixed15s sang checksum-checked
reported19→rank chọn→180. Refresh110 host rồi guest-ready cho click thật phát104 lần hai;
run cuối có hai vòng scene/input/bom/HELP/kết quả/phòng trên cùng channel.
[Contract](PROTOCOL.md#round-result), [raw/control/correction](evidence/round-repeat/README.md).
CP023/PR31 đến CP026/PR34 đã merge. CP027 bổ sung hai client movement/bom runtime dưới đây; review độc lập waived, gates giữ mở.

CP024 đã echo owner/ba field từ actual118 sang159 cho bom đầu mỗi vòng; serial0 chưa phân biệt hai field.
CP025 có control serial28 bác mapping aux16, sửa serial u32 và ba bom liên tiếp0/28/56;
ID1..3, lặp/cũ hẹp và native getter được kiểm. [Contract](PROTOCOL.md#bomb-serial),
[evidence](evidence/bomb-serial/README.md).
Đã nối index spawn0/1→point từ map→actual actor.100/4039 có nguồn bảng Antarctic01;
Village00 và ExVillage01 có profile/map/spawn/geometry riêng và hai vòng thử với guest tổng hợp.
[Contract](PROTOCOL.md#bomb-position), [raw/control/correction](evidence/bomb-position/README.md).
Không dùng điều này thay nghiệm thu collision/reset/authority hoặc hai client.

CP030 đã có layout582/request124/confirm165, nhặt từ hai origin và một ca tranh cùng index;
nguồn chuẩn ở [PROTOCOL](PROTOCOL.md#item-pickup), [raw/control/hash](evidence/item-pickup/README.md).
Trap/death CP029 giữ [contract riêng](PROTOCOL.md#peer-events).
CP030/PR38 đã merge `1f70d4f`. CP031 đo hiệu ứng Bomb/Fire, duplicate sau xóa, native clamp và
sửa mapping power; [nguồn chuẩn/giới hạn](PROTOCOL.md#item-effects),
[raw/control/hash](evidence/item-effects/README.md). CP031/PR39 đã merge `4de4e2e`.
CP032 đo block/list2→reveal/124/165/stat và khởi tạo lại vòng; contract ở
[PROTOCOL](PROTOCOL.md#hidden-items), raw/control ở [evidence](evidence/hidden-items/README.md).
Phân biệt run chờ sai origin với run sửa policy; review độc lập waived.
CP032/PR40 đã merge `fe46beb`. CP033 đo chain hai bom liền kề/cách hai ô và replay sau chain;
[contract](PROTOCOL.md#bomb-chain), [raw/control](evidence/bomb-chain/README.md). Không nâng toàn gate.

CP033/PR41 đã merge `fbf4dc5`. CP034 đo leave/107/host110/guestEOF và giải
return loading bằng trường loại sảnh trong channel14; [contract duy nhất](PROTOCOL.md#room-leave),
[control/recipe/hash](evidence/room-leave/README.md). Không ghép các ca này thành full match.

CP034/PR42 đã merge `0779bc7`. CP035 đo B rejoin và A rejoin sau B promotion,
Ready A→Start B→121/99/clock hai phía; [contract](PROTOCOL.md#room-rejoin),
[evidence](evidence/room-rejoin/README.md). Chỉ mốc rejoin/start, chưa full match.

CP035/PR43 đã merge `1e78035`. CP036 đo enemy touch/no-touch, rescue18→170/18/state56→26
hai phía và movement tiếp; [contract](PROTOCOL.md#collision-rescue), [evidence](evidence/collision-rescue/README.md).
Ca rescue có hai client thật + C tổng hợp; control một đội/report21 không dùng làm full match.

CP036/[PR44](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/44) đã merge `7ab702f`.
CP037 thêm [kim tự thoát/peer](PROTOCOL.md#needle-selfescape) và
[cứu/diệt ba client thật + nhánh native](PROTOCOL.md#contact-precedence);
[recipe/control/hash](evidence/needle-contact/README.md). Không ghép các ca thành full match.

CP037/[PR45](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/45) đã merge `1e2ac9a`.
CP038 trả lời hẹp [trap lần hai khi hết kim](PROTOCOL.md#needle-repeat), có đối chứng hoán đổi team;
[raw/control/hash](evidence/needle-repeat/README.md). Kiểm exact-simultaneity runtime là backlog bổ sung,
không điều kiện chặn mốc trận đầy đủ; đây là quyết định ưu tiên của Codex, không tự xác nhận unknown.

**Bước tiếp, Codex RE/tự kiểm:**

1. **NEXT-DUO-ROUND:** rời phòng/xóa ghost slot/promotion/guest EOF/về sảnh đã đo hẹp ở CP034;
   CP035 đã đo **rejoin qua click/request thật và Start sau promotion tới clock**. Ưu tiên đo actual21 cùng175/180: report có ngừng, UI kết quả và return/reset có nhất quán hai phía không;
   sau đó host EOF/disconnect trong trận trước nghiệm thu full match.
   Kiểm entity/timer/peer/seq/generation/reset trước K6a; giữ riêng mốc request104/reader121/99/clock CP035, không suy full match từ đó.
2. **NEXT-ITEM-PICKUP:** CP037 kim tự thoát/peer, CP038 có [trap lần hai khi hết kim](PROTOCOL.md#needle-repeat).
   Exact sameupdate/both-eligible runtime, đảo thứ tự danh sách và Ctrl cạnh tranh là kiểm bổ sung,
   không chặn first full match. Tiếp validator/stale/generation/toàn reset sau report21/result và disconnect;
   multi-charge/duplicate/aux khác và các mode giữ scope riêng. CP032 item ẩn và
   stat đầu vòng kế, CP033 chain một owner đã có bằng chứng; không tìm lại receiver. Runtime tại max/speed,
   chain nhiều owner/ba bom/vật cản, list3/loại block khác/đảo winner/stale index qua round còn thiếu.
   Giữ118.field10→159.field8. Report142 từ cả hai không tự chứng minh authority server gốc.
3. Receiver170/17,19 và mismatch death hẹp có lời giải CP029; CP036 bổ sung rescue/enemy-touch/timeout hẹp; CP037 có ba client thật và native collection dừng ở chạm hợp lệ đầu.
   Không tìm lại ID kim VN117/KR145 hoặc170/sub1; sameupdate/listorder/aux khác/variants là backlog kiểm bổ sung.
   Fullauth/profile/cache/reconnect/codec/vanilla và UDPgrammar/key/reliability/handshake/relay/NAT còn mở;
   104/95bodygrowth, serialrollover/allocator/concurrent/optionalflags, Antarctic timing giữ scope riêng.
   Cùngrank không thay deathstate; không có gate K5/K6a/LAN/Internet mới đóng.

CP029–032 raw/record bất biến; CP032 giữ control không158 và ca hai vòng chờ sai origin làm lịch sử.
Không patch thêm/ép state bằng GDB. `E9E660` là nhánh kind0 bom, không phải receiver pickup;
layout/request/confirm và stat Bomb/Fire mới đã đo riêng. Clamp native chưa thay runtime tại max.
Hai item ẩn đã đo không phục hồi thuật toán random drop hoặc toàn validator. Rank/outcome/fuse/timing
và policy giữ index cho request đầu là lựa chọn emulator,
chưa server authority gốc. Authprefix34byte/ID1/2,15/241variants,
oneclickuser/+E9C/token_2/count2/vanilla còn mở. AUD011/CP036 audit định kỳ hoàn tất; AUD012 sửa inference nháp CP037, audit định kỳ kế CP040.

| ID / ưu tiên | Trạng thái | Owner / reviewer | Đã biết | Còn thiếu / bước đóng |
|---|---|---|---|---|
| Q-022 / KRE-P1 bootstrap | open | Codex RE/tự kiểm | ProcessPatch local có debugger return1;recipe R2 đã merge;contract ở [PROTOCOL](PROTOCOL.md#wcp) | K1a ownership/control trên Windows local/effective token; K1b kiểm caller/config/IPC riêng CA và NewPatcher; K1c stub/patch theo evidence; gate ba cold start tới network attempt/state chờ hello, không lấy alive/không exit làm proof |
| Q-023 / KRE-P1 codec | open | Codex RE/tự kiểm | CP015 có socket kênh mới, native replay theo từng kết nối, schema17/count0 và nhận241 | Full result0/codec/reset theo bước trên; gate K2 còn mở |
| Q-024 / KRE-P1 login/kênh | open | Codex RE/tự kiểm | UI sảnh hết notice17/241; CP020 phục hồi inputID local, tên/profile phòng có fixture, chưa full auth/sảnh identity | Full identity/user/cache và cold/warm/reconnect K3; không gọi auth Nexon thật |
| Q-025 / KRE-P1 room/load | open | Codex RE/tự kiểm | CP028 join/Ready;CP034 leave/promotion/EOF/return;CP035 rejoin/newhostStart, [phạm vi](PROTOCOL.md#room-rejoin) | Password/error/unready/load-barrier variants, host abruptEOF/disconnect trong trận/reset/104growth/cache, nghiệm thu K4 |
| Q-026 / KRE-P1 authority/trận | open | Codex RE/tự kiểm | CP032/033/036;CP037 kim/ba client/precedence có giới hạn, [phạm vi](PROTOCOL.md#contact-precedence) | Drop algorithm, block/chain/rescue variants, runtime tại max/speed/validator/report21/result; K5 một chu kỳ hoàn chỉnh, không lấy packet183 ghost làm mover người sống |
| Q-027 / KRE-P1 peer | open | Codex RE/tự kiểm | CP027 movement/bom, CP029 trap/death, CP030 tranh item, CP032 reveal/stat và vòng kế; [giới hạn](PROTOCOL.md#hidden-items) | Full grammar/key/slot/reliability/relay, stale/generation/disconnect/variants và toàn reset trước K6 |
| Q-028 / KRE-P2 profile/core | open | Codex/tự kiểm | Core Rust và policy V1 có sẵn, nhưng client gốc không biết ADR0009 | Sau audit Q-026/027, quyết định giữ lai/relay/shadow core hay authoritative được client hỗ trợ; profile emu có provenance, không sửa V1 implicit |
| Q-029 / KRE-P2 vận hành | open | Codex/tự kiểm | CP029 có statepeer sau170 trước175 và hai chu kỳ thử; [giới hạn](PROTOCOL.md#peer-events); chưa fullmatch/K6a | K7 bốn client, malformed/stale/duplicate/race/disconnect/restart, bounds/resource limits, Linux package, patch manifest và README tái lập |

Tối đa hai lượt mỗi câu hỏi theo plan; gặp VM hoặc thiếu runtime thì ghi entry/boundary/thí nghiệm cần,
không đóng wire unknown bằng design. Không sửa trạng thái sang done chỉ vì đã có kế hoạch hoặc golden tự sinh.
