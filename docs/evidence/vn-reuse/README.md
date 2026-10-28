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
