# Kiến thức VN dùng cho KR — CP011

Đối chiếu ngày28/10 JST, owner Codex, tự kiểm theo [miễn review](../checkpoints/review-waiver.json).
Đây là evidence tái sử dụng có mục tiêu cho [Q-023](../../RESEARCH.md#q-023), không phải plan mới.
Nguồn tham chiếu `boom-re@9ac2075bbec0c16217741aaedea968cf685e39a9`; snapshot tài liệu/script,
đường dẫn gốc, hash và raw runtime KR được khóa trong [manifest](../checkpoints/manifest.json).
Không chạy lại client VN trong lượt này; các kết quả runtime VN là lịch sử từ tài liệu nguồn.

## Áp dụng ngay tại điểm chặn hiện tại

| Kiến thức/nguồn VN | Áp dụng KR và mức chứng minh |
|---|---|
| `docs/runtime.md` mục Debug bản dump, `tools/wine_gdb.sh`: proxy `winedbg --gdb` thay frontend tương tác | Đã thử trên KR copy: cả hai run bắt VA`123246D`, EAX=`1`, đọc slot`16EA5A8` ra`7B49D500`. `kr_runtime/confirmed` trong phạm vi breakpoint và đọc bộ nhớ; chưa kiểm hardware watchpoint hoặc mọi thao tác GDB. |
| `tools/xvfb_login.sh`, `wine_gdb.sh`: chờ sau click, retry vì thời gian startup khác; `tools/gdb/input_{dispatch,gates}.gdb`: truy message→handler→gate | Harness KR dùng Xvfb/namespace/prefix riêng, thêm chờ0,4s và tối đa3 lần thử; tọa độ lấy từ UI KR. Run tắt audio thu192 event tại entry DispatchMessage: down/up chuột9/9, keydown/up60/60, char54. Đã tới API dispatch; custom handler, focus control và hit-test chưa được chứng minh. |
| `tools/fix_iat.py`, `decomp/iat_resolved.json`: IAT tự khôi phục có alias sai, phải đối chiếu ABI/caller | VN phân biệt fake-slot GetKeyState`16AD3D0` và GetAsyncKeyState`16AD4AC`. Tại nhánh action VN, call`8DCE84` dùng Async; KR có call tương ứng cần điều tra ở`12272D9`. `vn_static/confirmed` cho VN, `candidate` cho đối sánh KR. Chưa có cơ sở đổi binding KR sang GetKeyState chỉ vì input chưa hoạt động. |

Raw ở `local/research/codex/vn-reuse-r1/`, harness riêng
`local/research/codex/wine_probe_vn_gdb{,_noaudio}.py`. Copy KR không đổi so với
[mốc UI](../login-ui/README.md), SHA`70708abab77ebd1cfd230a35f4997b8e84330f74442ed77b0b74c7b03180c82c`.

- `gdb-input`,27/10 23:31:46 UTC: guard/IAT đọc được, Wine audio assertion rồi inferior exit3;
  chưa đến thao tác input. Không gọi debugger hoàn toàn ổn định từ run này.
- `gdb-input-noaudio`,27/10 23:35:22 UTC: cùng copy, prefix mới đặt Wine Drivers/Audio rỗng
  trước launch; không thấy audio assertion trong cửa sổ70,17s. Ba lần thao tác có message thật
  tại DispatchMessage; ảnh vẫn không cho thấy nội dung ô nhập đổi, C→S vẫn tổng66 byte cũ.
  Đây là lựa chọn chẩn đoán `emu_choice/design`, không phải audio nguyên bản hoặc chứng minh
  nguyên nhân duy nhất của lỗi. Debugger làm thay đổi timing.

Các event chuột đầu có lParam`01F201C2` (x450,y498); nút xác nhận có`023E0193` (x403,y574).
Tọa độ đã tới message không chứng minh chúng trúng custom control. Điểm chặn đã thu hẹp từ
“xdotool trả0 nhưng chưa biết có event” sang đường sau DispatchMessage; bước kiểm duy nhất
được duy trì ở [RESEARCH](../../RESEARCH.md#q-023). Không có request auth mới hoặc gate mới tại CP011.
CP012 đã nối tới handler/button/command UI; cập nhật ở [evidence handler](../ui-handler/README.md).

## Dùng ở bước login/sảnh kế tiếp

`docs/protocol.md` mục Đăng nhập và chọn kênh, `docs/oc_d-login.md`, `docs/lobby.md` cung cấp
các giả thuyết có thứ tự, **chưa phải contract KR**:

| Bài học VN | Cách tiết kiệm công RE KR |
|---|---|
| Bootstrap-data trước login; manual request VN là C→S1/58, result S→C4, network info S→C15 | Đặt breakpoint từ thao tác xác nhận tới builder/send và từ receiver tới transition; xác định chiều/socket/stage trước khi gán nghĩa. C→S15 keepalive VN khác S→C15 network info; log packet15 KR không tự chứng minh auth. |
| VN cũng có adapter China với layout khác ngay trong một binary; playerCount và secure boxes có width riêng | Lập field ledger từ reader KR, kiểm cursor/width/điều kiện; dùng tên lỗi, chuỗi và call graph làm mỏ neo. Không sao chép payload VN. |
| Blob VN cần tag, inner endian khác outer; cache path chỉ có sau transition; gửi sớm từng gây CAppException | Truy initializer/consumer KR và cache cold/warm trước khi gửi blob kênh. Đây là checklist tránh đo sai stage, không phải kết luận KR có cùng tag/layout. |

Constants crypto, opcode, địa chỉ, offset, stage ID và tọa độ VN chưa được mang sang KR như fact.
Pack/map tooling đã có trong [ma trận reuse](../../PROTOCOL.md); chưa cần mở rộng gameplay khi input/login còn chặn.
Raw binary/packet/decompile nằm local; Git chỉ lưu metadata, recipe và hash.

## CP013 — tái sử dụng luồng login VN vào điểm chặn KR

Đối chiếu28/10 JST, baseline KR `2bcee732659bc4170eb869a491bdb0744e680d0b` (CP012/PR20 đã merge).
Nguồn VN hiện hành được đọc thêm: README trạng thái19/10, `docs/oc_d-login.md`, `protocol.md`,
`lobby.md` kể cả errata, `oc_e-game.md`, `tools/fakeserver.py`, `server/src/{session,packets}.rs`.
Snapshot/hash ở `local/research/codex/auth-request-r1/vn-login-references/` và manifest CP013.
Không chạy lại VN hoặc tạo runtime KR mới trong lượt đối chiếu này.

VN đã có bằng chứng lịch sử login→kênh→sảnh→phòng→trận hai client trong README và các report/run
được trỏ ở `oc_e-game.md`. Những đoạn ghi “hai client đứng yên” là mốc trước khi tìm peer UDP;
không dùng chúng làm trạng thái hiện tại. Code Rust/session mới là reference đang chạy, Python
fakeserver là đặc tả/golden có giá trị mẫu; các zero/default hoặc identity giả vẫn là lựa chọn emulator.

### Kết quả áp dụng vào KR

| Đường VN | Đối chiếu KR và giới hạn |
|---|---|
| S→C4 kết quả login → dialog0→2; `612CB1`, state`+D8` | Vtable `CLoginStage` `1788C64+CC → 10FCBE0`; opcode getter, trừ4 rồi bảng byte/jump: opcode4→`10FCC46`→`10FEDA0`, result0 gọi`10FD540`. Call`10FEBA5` truyền`stage+2C04` vào`C47260`, state`+F4` từ0→2; sau đó tạo C→S9. `kr_static/confirmed` cho đường và bytes; chưa đọc đủ field/result0 hoặc quan sát runtime handler. |
| S→C15 thông tin mạng → dialog2→1; `612CD0`; timer1→3 `612CEF` | Cùng dispatcher: opcode15→`10FCC6A`→`1100590`. Handler đọc blob, nhánh đọc string+3u16, mode1 có2string/mode2 cóu16, cuối tiêu u16; nếu `stage+2FD8!=0`, call`1100785` tới`C472A0(stage+2C04)` đặt2→1. `C472E0` chỉ đặt1→3; tick gọi sau điều kiện thời gian. `kr_static/confirmed` cho nhánh/call; schema/guard/encoding/blob đầy đủ và runtime vẫn cần kiểm. |
| Login VN cần cả hai response đúng thứ tự và đúng playerCount | KR có cùng mỏ neo state nhưng lệch offset và nhiều variant. Dùng field ledger VN để đặt breakpoint/cursor KR; chưa copy `login_ok()` VN thành KR fixture hoặc coi state3 là auth hợp lệ. |

Bảng opcode được giải từ **dispatcher KR**, không lấy số4/15 VN làm bằng chứng KR. Raw
`login-dispatch-map.json`, `handler-wrappers-corrected.asm`, `vn-kr-login-compare.asm` và
`wait-state-transitions.asm` ghi call/bytes. Artifact KR copy SHA`70708abab…`, VN patched SHA`bc875ba5…`;
full SHA trong manifest. Synthetic fixture/copy từ CP012 giữ giới hạn cũ.

Có một kết quả tìm kiếm phụ phải loại: script tìm SEH prologue không thấy match trước call`10FCC71`,
đã xuất range sai do chỉ số−1; file cũ giữ như lịch sử failed extraction, không dùng làm evidence
về function extent. Tìm lại ordinary prologue`10FCBE0`, kiểm exact call và vtable+CC mới là nguồn.
Phép tìm `push1` gần constructor general packet cũng không xác nhận auth opcode1 KR.

### Thứ tự tận dụng để tiến login/sảnh

Nguồn chuẩn bước tiếp ở [Q-023](../../RESEARCH.md#q-023): từ mỏ neo mới đo reader/handler và
request auth KR trên đúng socket. Chưa thấy auth request thì chưa dùng C→S15 đã capture làm trigger
“login thành công”. VN request1/58 là tham chiếu riêng của VN; contract C→S auth KR vẫn unknown.

Sau login, VN giúp tiết kiệm các vòng thử đã biết: số tab kênh đến từ login result; danh sách/status
kênh phải đúng category/online; cache phải có tag và được khởi tạo path trước khi gửi; redirect cần
socket/hello/seq mới và identity xuyên socket. Sau sảnh, dùng chuỗi phòng/load/ready/start, loading
barrier và UDP peer làm checklist. Opcode, field width, numeric ID, map/spawn và authority vẫn phải
đối chiếu consumer/runtime KR. Các bước này không tự được xác nhận chỉ vì VN đã chạy được trận.
