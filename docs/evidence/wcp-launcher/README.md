# CP005 — Điều kiện rẽ sang launcher

Phạm vi [CP005](../checkpoints/manifest.json), review theo manifest. Contract ở
[PROTOCOL](../../PROTOCOL.md#wcp); câu hỏi tiếp ở [RESEARCH](../../RESEARCH.md#q-023).

Codex chạy lại variant CP004 SHA `60e981698ed8c4adb3ca070a9612212d7cefa14d29453eb895513367cf197ce7`,
hai ô error được reset trong copy, dưới Wine debugger với prefix copy đã dùng và harness hai reply.
Run `local/research/codex/wcp-launcher-r1/debug` có lo-only, không route IPv4 và NSS files-only.
Consumer tại `13AD79C` trả1; tại các return breakpoint `123238D`, `12323A4`, `12323B4`,
EAX lần lượt `1,0,0`; breakpoint `12323BA` được hit. `CALauncher` được khởi chạy, CALog ghi
exitcode9 và sau đó debugger ghi UD2. Không quan sát guard `1232468` trong run này.

Static trong modO: `D95CB0` gọi `456AD0(0x9f)` rồi đảo điều kiện, `456AD0` trả AL=0;
`122AE50` và `122AF30` qua hai wrapper khác cùng gọi `52A350` với offset đối tượng `+E78`, `+E9C`.
Ý nghĩa các giá trị, đầu vào/khởi tạo và tính hợp lệ vẫn unknown. Các đoạn byte vùng nhánh/hàm
trên variant khớp modO theo `static-byte-compare.json`; hash raw nằm trong checkpoint.
Code `12322E0`/`1234E40`/`1232458` lưu từ WCP r2 được liên kết trực tiếp ở CP005.

CP004 dùng chữ "copy thử zero ... cho qua" quá mạnh: hai run khác nhau về prefix và timing,
cổng và breakpoints; CP005 thu hẹp thành các quan sát theo từng run. Một phần nhánh if/ordering
là static/inference, không phải breakpoint đã đo mọi lệnh. Không khẳng định patch đơn lẻ gây ra
mọi khác biệt; không suy thành Windows vanilla hoặc login. Baseline debug-baseline có breakpoint tại `1232385`/`12323BA`: tới `12364A0` và không hit hai
điểm đó trước khi hết quan sát; dialog/debugger có thể chặn tiến trình, nên negative hit chỉ áp
cho run này. Baseline và variant vẫn không có prefix/port/timing hoàn toàn khớp.
