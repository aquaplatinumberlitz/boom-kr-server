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
Q-023, CP004 (review theo [manifest](evidence/checkpoints/manifest.json)): R2 đọc error1/msg1958 có sẵn trong image, gọi 12364A0 trước
12349E0. Copy thử zero hai ô cho qua nhánh đó, sau đó đến 12323BA→1258150, chạy CALauncher
và ghi error9/msg0x9a1; khi thoát gặp UD2 tại 226459C. Xem [contract](PROTOCOL.md#wcp) và
[evidence/giới hạn](evidence/wcp-guard/README.md). Chưa có proof cho nguyên nhân MW_SEP(10).

Bước tiếp: Codex kiểm nguồn flag d95cb0 và các điều kiện 122AE50/122AF30 bằng phép đo hẹp;
đối chiếu launcher path và điều kiện khởi tạo hợp lệ trước khi quyết định test copy tiếp. Không
coi zero lỗi là fix sản phẩm hoặc giả định guard12349E0 đã được gọi. Nếu qua early exit mới đo
guard và notification400/mode2/selector3; storage u32 cần oracle khác0. Claude chỉ review.

| ID / ưu tiên | Trạng thái | Owner / reviewer | Đã biết | Còn thiếu / bước đóng |
|---|---|---|---|---|
| Q-022 / KRE-P1 bootstrap | open | Codex RE / Claude reviewer độc lập | ProcessPatch local có debugger return1;recipe R2 đã merge;contract ở [PROTOCOL](PROTOCOL.md#wcp) | K1a ownership/control trên Windows local/effective token; K1b kiểm caller/config/IPC riêng CA và NewPatcher; K1c stub/patch theo evidence; gate ba cold start tới network attempt/state chờ hello, không lấy alive/không exit làm proof |
| Q-023 / KRE-P1 codec | open | Codex RE / Claude reviewer độc lập | Primitive có [oracle](evidence/primitives/README.md);WCP có bằng chứng hẹp tại [PROTOCOL](PROTOCOL.md#wcp) | Đo nhánh launcher/early exit theo Q-023;rồi guard,codec2/3,checksum/seq/reset/request kế cho K2 |
| Q-024 / KRE-P1 login/kênh | open | Codex RE / Claude reviewer độc lập | VN có bootstrap-data, login/redirect/cache/kênh; các variant trong một binary đã khác layout | KR handler/field/order/encoding/local identity; cold/warm cache và keepalive vào sảnh K3; không gọi auth Nexon thật |
| Q-025 / KRE-P1 room/load | open | Codex RE / Claude reviewer độc lập | Asset map KR và UI Ready có locator; VN có bẫy map tự gửi/spawn trước khi load | Wire KR create/join/host/slot/ready/start/roster/map/spawn/ack; K4 hai client thấy cùng phòng và spawn đúng |
| Q-026 / KRE-P1 authority/trận | open | Codex RE / Claude reviewer độc lập | KR có consumer flame/chain, timer/send167/status184; VN mô hình lai | Phân quyền client/server/peer từng event; packet bom/pickup/trap/death/result/correction KR; K5 một chu kỳ hoàn chỉnh, không lấy packet183 ghost làm mover người sống |
| Q-027 / KRE-P1 peer | open | Codex RE / Claude reviewer độc lập | VN92/146 và UDP P2P có runtime; chưa xác nhận tương đương KR | Peer negotiation, endpoint, key/slot, protocol UDP và relay KR; movement hai chiều, cùng result và hai trận K6 |
| Q-028 / KRE-P2 profile/core | open | Codex / Claude reviewer độc lập | Core Rust và policy V1 có sẵn, nhưng client gốc không biết ADR0009 | Sau audit Q-026/027, quyết định giữ lai/relay/shadow core hay authoritative được client hỗ trợ; profile emu có provenance, không sửa V1 implicit |
| Q-029 / KRE-P2 vận hành | open | Codex / Claude reviewer độc lập | Runtime KR copy Wine hẹp có [evidence](evidence/wcp/README.md);chưa multiplayer/gameplay | K7 bốn client, malformed/stale/duplicate/race/disconnect/restart, bounds/resource limits, Linux package, patch manifest và README tái lập |

Tối đa hai lượt mỗi câu hỏi theo plan; gặp VM hoặc thiếu runtime thì ghi entry/boundary/thí nghiệm cần,
không đóng wire unknown bằng design. Không sửa trạng thái sang done chỉ vì đã có kế hoạch hoặc golden tự sinh.
