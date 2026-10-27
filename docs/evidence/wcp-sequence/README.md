# CP006 — request đầu và chuỗi response thử

RE Codex ngày27/10/2026, review theo [manifest](../checkpoints/manifest.json). Contract ở
[PROTOCOL](../../PROTOCOL.md#wcp), blocker/kiểm tiếp tại [RESEARCH](../../RESEARCH.md#q-023).
Mọi runtime dưới đây là client copy sửa trên Wine; chưa có login, sảnh, gameplay hay gate K2.
Binary gốc/DB không sửa. Các raw packet/log có thể chứa dữ liệu phiên, giữ trong `local/`, không commit.

## Các run quan trọng

Harness private `wine_probe_wcp_args.py` nhận tham số client; biến thể `wine_probe_wcp_next.py` gửi
response thử sau C→S đầu tiên trên socket thứ hai. Cả hai giữ user/net/mount/PID namespace, chỉ
loopback, không route IPv4 và NSS files-only. Prefix là các copy đã dùng; hash và run ID ở manifest.
Fixture WCP 15 byte và bản reset error kế thừa [CP004](../wcp-guard/README.md).

- `debug-args` và `plain-args`: copy reset error SHA `60e981698ed8c4adb3ca070a9612212d7cefa14d29453eb895513367cf197ce7`,
  tham số `multiserveridx:0`; mỗi run thu 66 byte C→S, cùng SHA-256
  `ba1ed0f5373e023c288c889488b6e1186d097959407ef931fa87582dd48fe43f`. Debug thấy
  `1236138 EAX=1` sau tra key, setter `123614E`, rồi `12323A4 EAX=1`; `1232468` guard được gọi.
  Không có response sau WCP nên poll `12386D8 EAX=0`. `plain-noarg` với prefix copy riêng thu 0 byte
  và tới launcher/exit9; so sánh này có giới hạn prefix, timing và cách chạy, không phải control khớp hoàn toàn.
- `debug-next`: một fixture opcode13 S→C, poll `12386D8 EAX=1`, `1238770 EAX=13`, vào `CD3660`;
  poll tiếp theo `12387EB EAX=0` vì stub không gửi thêm. Body rỗng chỉ để thử đường đi.
- `debug-three`, `debug-four`: fixture opcode13/2/12 rồi21; debugger xác nhận opcode và consumer
  từng bước. `debug-eight` dùng payload rỗng opcode43, vào fallback và gặp exception cấp phát;
  không xem đây là response đúng.
- `debug-eight-v2`: tám fixture `13,2,12,21,1,43,62,719` coalesced; opcode43 dùng flag1/count0
  để tránh fallback rỗng. Breakpoint ở các giai đoạn sau xác nhận 43,62,719 và guard `123246D EAX=1`,
  `12324D4` được hit. Sau đó gặp trap `2265790` (slot `16EA8B0`, candidate NMCO init).
  Fixture là lựa chọn emulator, không phải capture server thật; không có đối chứng Windows vanilla.
- `debug-nmco`: trên copy mới, hai call NMCO init/callback được NOP, giữ stack cleanup; guard vẫn
  trả1, các breakpoint sau hai call được hit, rồi gặp trap `2265800` (slot `16EA8E8`, candidate
  NMCOGetMyStatus). `debug-nmco-status`: thêm patch trả EAX=0 ở call status; runtime tới
  `13CA17F EAX=0`, sau đó trap `22657F8` (slot `16EA8E4`, candidate NMCOGetMyIdCode).
  Return0 là `emu_choice`; cả hai patch là chẩn đoán, chưa review như recipe phân phối.

## Codec, đối chứng và giới hạn

Reader native `14EF300`/vtable `17B0BF4` và socket tổng hợp ở mode2/selector3 được chạy trong
Unicorn. Từng fixture hợp lệ trả0 và giữ opcode/payload; checksum sai trả7 và frame thiếu byte trả2.
Dùng parser/crypto VN làm giả thuyết; hằng số XOR trong KR được đọc tại `14ED2B0/14ED340`.
Các response riêng dùng seed43,46,…,64 theo socket sequence đã đo/kiểm; `eight-v2-responses.bin`
317 byte SHA `5930ad297fc2c1540d8cbd894e88d3c1d2831bd4565dfb8ad96644568b87ad38`.
Native oracle có socket/object tổng hợp và hook allocator; một run client đã tiêu thụ chuỗi. Không
suy framing đầy đủ, multiple connection, reset/reconnect, semantics dữ liệu hay server gốc.

`multiserveridx` được tra tại `1236133` và nhánh gán `+E78` tại `123614E`; runtime hỗ trợ quan hệ
cho tham số đã thử. `oneclickuser` đi tới setter `+E9C` theo static tại `123622C`, chưa có phép đo
runtime riêng. Nhãn và ý nghĩa từng field còn giới hạn. `13CA130` nhận ID 64 bit ở EDX:EAX và caller
`13BE364` lưu kết quả; giá trị hợp lệ/nguồn identity và tác động khi giả lập vẫn unknown.

Patch `nmco-patch.json`/`nmco-status-patch.json` ghi before/after, SHA, rollback và side effect;
slot candidate lấy từ report builder, cần kiểm ABI/consumer độc lập trước khi làm recipe.
Những lần thử rỗng bị crash giữ nguyên như lịch sử, không lặp thành trạng thái hiện tại.
