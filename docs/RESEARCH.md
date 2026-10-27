# Câu hỏi RE của server KR

Đây là backlog riêng KRE. Các ID Q-022…Q-029 được giữ để truy nguồn; không thuộc gate V1 Rust.


Mở theo yêu cầu trực tiếp 26/10/2026. Đây là câu hỏi của **`KR-00967-EMU-1`**, không đổi câu hỏi/gate
V1 Rust. [Lộ trình KRE](PLAN.md) giữ thứ tự
task và điểm dừng; [contract](PROTOCOL.md) giữ ma trận/nguồn.
KRE-001/002/003 đang thực hiện: theo chỉ đạo mới nhất27/10, Codex làm RE chính và runtime; Claude
Sonnet5.5 chỉ review độc lập. Report dsh/Claude RE cũ giữ làm nguồn lịch sử.
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

Bước tiếp: Codex xác định call site/ABI của import `16EA638`, thử bind trên copy theo evidence;
đối chứng Wine audio/debug khi cần. Theo dõi các side effect NMCO
và dữ liệu khởi tạo; không coi identity giả là ID server gốc hay cửa sổ là login thành công.
Tiếp tục đo mode2/3 hai chiều, seq/reconnect và nguồn `oneclickuser` khi cần. Claude chỉ review.

| ID / ưu tiên | Trạng thái | Owner / reviewer | Đã biết | Còn thiếu / bước đóng |
|---|---|---|---|---|
| Q-022 / KRE-P1 bootstrap | open | Codex RE / Claude reviewer độc lập | ProcessPatch local có debugger return1;recipe R2 đã merge;contract ở [PROTOCOL](PROTOCOL.md#wcp) | K1a ownership/control trên Windows local/effective token; K1b kiểm caller/config/IPC riêng CA và NewPatcher; K1c stub/patch theo evidence; gate ba cold start tới network attempt/state chờ hello, không lấy alive/không exit làm proof |
| Q-023 / KRE-P1 codec | open | Codex RE / Claude reviewer độc lập | Primitive có [oracle](evidence/primitives/README.md);WCP có bằng chứng hẹp tại [PROTOCOL](PROTOCOL.md#wcp) | Truy import hiện tại theo Q-023; tiếp tục identity và codec hai chiều,checksum/seq/reset và gate K2 |
| Q-024 / KRE-P1 login/kênh | open | Codex RE / Claude reviewer độc lập | VN có bootstrap-data, login/redirect/cache/kênh; các variant trong một binary đã khác layout | KR handler/field/order/encoding/local identity; cold/warm cache và keepalive vào sảnh K3; không gọi auth Nexon thật |
| Q-025 / KRE-P1 room/load | open | Codex RE / Claude reviewer độc lập | Asset map KR và UI Ready có locator; VN có bẫy map tự gửi/spawn trước khi load | Wire KR create/join/host/slot/ready/start/roster/map/spawn/ack; K4 hai client thấy cùng phòng và spawn đúng |
| Q-026 / KRE-P1 authority/trận | open | Codex RE / Claude reviewer độc lập | KR có consumer flame/chain, timer/send167/status184; VN mô hình lai | Phân quyền client/server/peer từng event; packet bom/pickup/trap/death/result/correction KR; K5 một chu kỳ hoàn chỉnh, không lấy packet183 ghost làm mover người sống |
| Q-027 / KRE-P1 peer | open | Codex RE / Claude reviewer độc lập | VN92/146 và UDP P2P có runtime; chưa xác nhận tương đương KR | Peer negotiation, endpoint, key/slot, protocol UDP và relay KR; movement hai chiều, cùng result và hai trận K6 |
| Q-028 / KRE-P2 profile/core | open | Codex / Claude reviewer độc lập | Core Rust và policy V1 có sẵn, nhưng client gốc không biết ADR0009 | Sau audit Q-026/027, quyết định giữ lai/relay/shadow core hay authoritative được client hỗ trợ; profile emu có provenance, không sửa V1 implicit |
| Q-029 / KRE-P2 vận hành | open | Codex / Claude reviewer độc lập | Runtime KR copy Wine hẹp có [evidence](evidence/wcp/README.md);chưa multiplayer/gameplay | K7 bốn client, malformed/stale/duplicate/race/disconnect/restart, bounds/resource limits, Linux package, patch manifest và README tái lập |

Tối đa hai lượt mỗi câu hỏi theo plan; gặp VM hoặc thiếu runtime thì ghi entry/boundary/thí nghiệm cần,
không đóng wire unknown bằng design. Không sửa trạng thái sang done chỉ vì đã có kế hoạch hoặc golden tự sinh.
