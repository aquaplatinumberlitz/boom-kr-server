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

Bước tiếp: Codex nối C→S96 (native candidate u16/u32),266 và handler117 tới writer/consumer map;
phục hồi bảng map, index/room type và bounds tối thiểu trước khi gửi fixture. Docs VN về bảng map,
nhóm và room-type/filter chỉ là mỏ neo. Start một người đã bị chặn số người; thử roster giả cùng stub
chỉ kiểm nhánh, không chứng minh hai client/multiplayer. Sau map mới mở ready/start/load có mục tiêu.
Roster giả hai đội đỏ-vàng đã đi qua kiểm số người/đội và tới thông báo map tối đa0 người (teams-retry).
Ưu tiên phục hồi map capacity/data; không ép return kiểm map. KR111 hypothesis đã sửa thành110
từ jump table; incomplete run và một wineboot timeout trước client không tính success.
Đồng thời hoàn thiện result0/user/cache, các biến thể channel14/91 và cursor/blob15/241,
lifecycle global1A966FC, oneclickuser/+E9C, token_2/count2 và full codec/reset.
Review độc lập được miễn; AUD005 ở CP016, audit định kỳ kế CP020. Chưa thêm gate.

| ID / ưu tiên | Trạng thái | Owner / reviewer | Đã biết | Còn thiếu / bước đóng |
|---|---|---|---|---|
| Q-022 / KRE-P1 bootstrap | open | Codex RE/tự kiểm | ProcessPatch local có debugger return1;recipe R2 đã merge;contract ở [PROTOCOL](PROTOCOL.md#wcp) | K1a ownership/control trên Windows local/effective token; K1b kiểm caller/config/IPC riêng CA và NewPatcher; K1c stub/patch theo evidence; gate ba cold start tới network attempt/state chờ hello, không lấy alive/không exit làm proof |
| Q-023 / KRE-P1 codec | open | Codex RE/tự kiểm | CP015 có socket kênh mới, native replay theo từng kết nối, schema17/count0 và nhận241 | Full result0/codec/reset theo bước trên; gate K2 còn mở |
| Q-024 / KRE-P1 login/kênh | open | Codex RE/tự kiểm | UI sảnh hết notice bằng17/241 trên copy, profile/tên còn trống | Full identity/user/cache và cold/warm/reconnect K3; không gọi auth Nexon thật |
| Q-025 / KRE-P1 room/load | open | Codex RE/tự kiểm | CP018 có player106, own ID match,110 bật controls và nút Bắt đầu; map trống | Writer96/266, bảng map/117, room type/filter, join/ready/start/roster/spawn/ack; K4 hai client thấy cùng phòng và spawn đúng |
| Q-026 / KRE-P1 authority/trận | open | Codex RE/tự kiểm | KR có consumer flame/chain, timer/send167/status184; VN mô hình lai | Phân quyền client/server/peer từng event; packet bom/pickup/trap/death/result/correction KR; K5 một chu kỳ hoàn chỉnh, không lấy packet183 ghost làm mover người sống |
| Q-027 / KRE-P1 peer | open | Codex RE/tự kiểm | VN92/146 và UDP P2P có runtime; chưa xác nhận tương đương KR | Peer negotiation, endpoint, key/slot, protocol UDP và relay KR; movement hai chiều, cùng result và hai trận K6 |
| Q-028 / KRE-P2 profile/core | open | Codex/tự kiểm | Core Rust và policy V1 có sẵn, nhưng client gốc không biết ADR0009 | Sau audit Q-026/027, quyết định giữ lai/relay/shadow core hay authoritative được client hỗ trợ; profile emu có provenance, không sửa V1 implicit |
| Q-029 / KRE-P2 vận hành | open | Codex/tự kiểm | Runtime KR copy Wine hẹp có [evidence](evidence/wcp/README.md);chưa multiplayer/gameplay | K7 bốn client, malformed/stale/duplicate/race/disconnect/restart, bounds/resource limits, Linux package, patch manifest và README tái lập |

Tối đa hai lượt mỗi câu hỏi theo plan; gặp VM hoặc thiếu runtime thì ghi entry/boundary/thí nghiệm cần,
không đóng wire unknown bằng design. Không sửa trạng thái sang done chỉ vì đã có kế hoạch hoặc golden tự sinh.
