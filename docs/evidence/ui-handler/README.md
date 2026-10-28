# CP012 — handler UI và endpoint login KR

Owner Codex, issue14, review độc lập [được miễn](../checkpoints/review-waiver.json).
Nhóm run ngày28/10 JST dùng cùng copy SHA`70708abab77ebd1cfd230a35f4997b8e84330f74442ed77b0b74c7b03180c82c`,
không thêm patch binary so CP011. Raw ở `local/research/codex/ui-handler-r1/`; GDB/argv/harness,
run metadata, log, snapshot và SHA ở [manifest](../checkpoints/manifest.json).
Mỗi run dùng namespace user/net/mount/PID chỉ loopback, NSS files-only, Xvfb và prefix riêng tắt audio.
Đây là hành vi copy chẩn đoán, chưa có đối chứng vanilla Windows.

## Đường input đã đo

`kr_runtime/confirmed` trong phạm vi run, kết hợp `kr_static` cho call/RTTI:

1. Message đi qua pump `CD6681` hoặc `12569AF` tới callback `1259C00`; gate tại `1259C3D`
   trả1, vào dispatch `125A5A0`. Tên WinAPI là binding trên copy, chưa xác nhận import nguyên bản.
2. Mouse đi `125C250 → 1504563`, key/char đi `1504770 → 1504C10`; UI manager/root có thật trong
   snapshot. Không còn giữ giả thuyết mọi input bị mất trước custom handler.
3. RTTI vtable`16F105C` là `CCtrlButton`. Char trong phép thử đi tới callback`1508006` của button;
   ở hai tọa độ ô ID/password chưa quan sát edit control hoặc nội dung đổi. Không suy từ đó toàn
   client không hỗ trợ manual login hoặc hook omission là nguyên nhân.
4. Click xác nhận trúng button rectangle x351,y558,w99,h30; callback có x52,y16, handler`1509376`.
   Internal ID tại`+94` là7; **command ID khác**, getter`14FEBDF` đọc`+100`.
   Live callback`10FC540` nhận command`3EA` (=1002), interface this=`stage+AC`, rồi gọi
   `1101E20 → 1101BA0`. Đối chiếu primary vtable đơn thuần từng dẫn tới no-op; không dùng
   phỏng đoán đó thay callback thực trên interface.

`register-proc` đặt breakpoint RegisterClassEx sau guard không thấy registration trong cửa sổ đo;
không suy không có registration. Static site`125BC96/125BCA1` đặt callback`1259C00`;
probe `wndproc` kiểm execution của chính callback này. Các asm chỉ là range/call site có mục tiêu,
không kiểm kê toàn binary. Bước tiếp chỉ giữ ở [Q-023](../../RESEARCH.md#q-023).

## Endpoint và WCP sau UI

| Nhóm run | Quan sát và giới hạn |
|---|---|
| `login-gates`, `login-connect`, `endpoint-dns` | Handler xác nhận tới kiểm kết nối; `1101CB2` AL1, precheck AL0, `11047D0` trả0. `1255AEA` status−1; endpoint resolver`13A8300` trả0 trước connect. gethostbyname yêu cầu `dream-login-ca.nexon.com`, trảNULL trong namespace chưa ánh xạ hostname này. Port output thấy3838/3848. Không gọi hàm này đơn thuần là validator identity: call graph bao gồm endpoint/connect. |
| `dream-local-silent` | Chỉ thêm ánh xạ hostname trên về127.0.0.1 trong NSS riêng, giữ copy và stub hai hello cũ. DNS có kết quả, resolver trả1 và connect local thứ ba được capture; kết nối thứ ba chưa có C→S. Prefix/timing khác, không phải đối chứng nhân quả tuyệt đối của mọi hành vi. |
| `dream-local-hello` | Stub gửi thêm cùng fixture15 byte cho connection ordinal3, không gửi tám response khởi tạo cho connection này. Capture có connection3 và4, vẫn chỉ66 byte cũ ở connection2. Log WCP failed về sau không được gán ngược cho connection3 khi thiếu locator. |
| `auth-after-hello` | Breakpoint sau guard đo WCP poll`13AD782` và consumer`13AD79C` đều EAX1, tiếp vào`11049C0`, selected0. Chưa có C→S mới. Lần WCP timeout sau đó là phép connect tiếp không được cấp hello; không trộn hai kết nối. |

**Thứ tự thời gian:** trong `dream-local-silent`, accept3 xảy ra27,050s tính từ start record;
click-confirm đầu32,964s. Trong `dream-local-hello`, accept3=27,090s, click đầu33,038s;
accept4=33,081s. Vì vậy connection3 thuộc luồng tự khởi động trước click trong các run này;
chưa xác nhận điều kiện gây auto-login. Việc click gọi command1002 được chứng minh bằng probe
riêng, không dùng nó giải thích connection3. `multiserveridx:0`, identity giả và dữ liệu tổng hợp vẫn
là các điều kiện kế thừa; `oneclickuser/+E9C` chưa đo runtime.

Không có thêm request auth trên wire, auth success, sảnh/gameplay hoặc gate mới.
Thông số fixture/mode và nguồn chuẩn wire vẫn ở [PROTOCOL](../../PROTOCOL.md#wcp).

## Điểm chặn hiện hành

Run `auth-dialog` đo thêm: `11049C0` được gọi từ return`11013FE` trong luồng trước input;
đi qua cuối config socket`1255DCC`, vào`1104410`. Member`stage+2C04` có vtable`1751764`,
RTTI `CWaitForLoginDlg`, vf`+98` trỏ`15037F7`. Sau call modal tại`11044A4`, breakpoint
`11044A7` đo EAX1; static tại`11044B7` chỉ vào nhánh kế khi result3. Hàm attempt quay về với
EAX0 tại`1104A26`. Chưa giải vì sao dialog trả1 hoặc ý nghĩa đầy đủ các return code.

Cùng run có các entry `13ADD10` từ caller`1164D6D`, nhưng không có entry WinSock `send` mới
trong phạm vi breakpoint và capture vẫn chỉ66 byte trên connection2. Entry của hàm gửi logic
không chứng minh đã gửi wire; chưa kiểm hết đường `WSASend` hoặc queue. Không gán các entry này
thành request auth. Source binary/DB không đổi; tất cả khác biệt môi trường/fixture ghi trong recipe.

Blocker tại run `auth-dialog` (trước phép reset global): truy lifecycle/OnInit/modal-result và data dependency của
`CWaitForLoginDlg`, rồi nối tới packet builder/queue/socket. Không ép result3 hoặc tự dựng login-success
để đóng gate. Audit định kỳ CP012 đọc core docs và đối chiếu các raw trên; nguồn counter ở manifest.

## Phép kiểm global làm modal đóng sớm

`modal-result` đo result được reset0 tại entry modal, rồi update một vòng và setter nhận1
với return`15039A5`. Static nhánh`150398C` kiểm global`1A966FC`; image chẩn đoán đã có u32=1.
Setter toàn cục`1503DFD` ghi1, nhưng caller/ý nghĩa lifecycle gốc chưa xác nhận. Không gọi global này
là trạng thái auth hoặc khẳng định đây là giá trị lỗi duy nhất của dump.

`modal-global-zero` đặt **bộ nhớ run copy** tại`1A966FC` từ1 về0 sau guard; không sửa file client.
Log xác nhận giá trị trước/sau, modal reset0 và cập nhật; không thấy setter1 từ`15039A5` hoặc
return modal trong cửa sổ đo. Ảnh`screen-09.png` hiển thị dialog đếm chờ login (07).
Đây là `emu_choice/design` có bằng chứng `kr_runtime/confirmed` cho khác biệt được quan sát;
chưa chứng minh login thành công. Capture hoàn tất: run này có276 byte C→S mới trên connection3; lifecycle của global, state0 và nguồn
request auth tiếp vẫn cần truy. Hoàn tác bằng kết thúc prefix/run; không có thay đổi source binary/DB.

## Packet15 trên wire sau phép reset

Cả `modal-global-zero` và `send-zero` có **276 byte mới trên connection3**, là4 frame×69 byte;
connection2 vẫn giữ66 byte request khởi tạo. Dòng “chỉ66” ở các nhóm trước không áp dụng hai run này.
Trong `send-zero`: `13ADD21` có guard0/socketobjNULL trước connect; sau WCP, guard1 và socketobj
khác0. Packet snapshot có opcode15; virtual send target`14EF650`, WinSock send socket`29C` len69,
khớp capture. Điều này nối log builder với wire trong đúng run, chưa gán semantics auth.

Bốn frame có header mode2/selector3 đang thử: prefix1 byte +9 u32LE (36 byte), payload31 và CRC8.
Key55 cố định bị native parser từ chối; key40 chỉ đúng frame đầu. Với key40,43,46,49, hàm KR thật
`14EF300` trên synthetic receiver tiêu thụ69 byte/frame, opcode15, CRC khớp và packet_error0.
Mỗi frame có negative: đổi CRC→return7/error10/không consume; cắt byte→return2/không consume.
Payload native được so với decrypt VN và trùng trong bốn mẫu. Giữ cả oracle thất bại như lịch sử.
Đây là `kr_runtime` capture kết hợp `kr_static`/emulation native, không dùng encoder tự sinh làm oracle.
Synthetic receiver tắt prefix-check và không phải server C→S thật; seq40+3n chỉ confirmed cho
chuỗi bốn mẫu này, chưa kiểm rollover, full reconnect, auth field hoặc server dispatch.

State dialog quan sát vẫn0, chưa có response auth hoặc chuyển sảnh. K2 đầy đủ/K3 chưa đạt.
