# Từ input sảnh tới request tạo phòng — CP016

Owner Codex, issue14, tự kiểm theo [miễn review](../checkpoints/review-waiver.json).
Baseline `49b78b12d82711f3e686a45d11e77c94131a3d65` (CP015/PR23 đã merge).
Contract duy nhất ở [PROTOCOL](../../PROTOCOL.md#room-create), bước tiếp ở [Q-023](../../RESEARCH.md#q-023).

Raw: `local/research/codex/room-input-r1/`; manifest CP016 khóa size/SHA cho metadata, scripts,
log/capture/screenshot và mỗi hash index của run. Các file prefix không phải binary nguồn.
14 run ngày28/10 UTC chạy tuần tự: Wine namespace user/net/mount/PID, loopback-only/no IPv4 route,
NSS files-only, prefix/Xvfb riêng; game-dir chung giữ CALog trước/sau. Cache không chuẩn hóa,
không dùng các run này làm nghiệm thu cold/warm hoặc chứng minh nguyên nhân duy nhất.

## Recipe và artifact

Cùng exe `local/research/codex/wcp-guard-r1/game/CA-r2-login-logout.exe`, SHA
`660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3`; không patch binary mới.
Kế thừa các chỉnh sửa chẩn đoán CP006–015, token local giả và reset memory1A966FC=0 sau guard.
Không ép cờ enable/index/level/modal/stage. Source client/DB không đổi, không kết nối Nexon thật.
IDA list không có DB; REA session không mở target, provider native chưa cấu hình như CP015;
dùng GDB Wine, PE/Capstone mục tiêu và native Unicorn. Không đổi cấu hình tool.

Runner `run_probe.py` lưu argv từng run; harness mới nằm ở `local/research/codex/wine_probe_*`.
Ví dụ runner label mới, exe trên,46s, `wine_probe_create_confirm.py`, commands `create-confirm.gdb`.
Recipe/harness/hash trong manifest; copy và fixtures cũ chỉ đọc. Rollback là chạy harness control
`wine_probe_room_input.py` với cùng exe, không cấp91/14, rồi kết thúc namespace/prefix riêng.
Harness mới tắt WINEDEBUG nhiễu, chỉ click xác nhận tuổi một lần; nút phòng giữ1s thay0,25s.
Vì vậy không dùng khác biệt giữa harness cũ/mới như phép nhân quả tuyệt đối.

## Chuỗi đo và correction

| Run | Kết quả / giới hạn |
|---|---|
| `input-route` | Sảnh ready, chưa thấy entry trên primary CLobbyStage; không suy input mất. |
| `app-input`, `input-filter` | WndProc nhận down/up đúng tọa độ. Các filter đi qua; gate không đủ để quy cho profile. |
| `input-manager` | Scale1, UI tree tới `CCtrlButton` theo RTTI. Primary stage callback không phải đường nút này. |
| `button-notify`, `button-callback` | Nhả chuột tới owner callback810230 trên interface+FC CLobbyRoomUIWnd; command2011. Label `id` ở probe trước dùng+64, không phải command getter+100. |
| `room-enable` | Command2011→813060 đo enable0/busy0 và trả ngay; không có request tạo phòng. Parent runner exit143 sau khi harness đã ghi run.json; không gọi outer exit là pass. |
| `list91`, `level-snapshot` | Handler91/count0 đặt enable1; click vượt guard, sau đó fault52A219; snapshot selected−1 tại getter level. |
| `channel14`, `channel14-string`, `blob14` | Mẫu đầu thiếu selector raw/nén: outer flag1/cursor đủ vẫn có inner length0, count26276 và abort trong vòng row/string. **Mẫu bị bác bỏ**, không phải contract hợp lệ. |
| `channel14-raw` | Thêm selector0; handler thật nạp count1, lookup index0, level0..65535/check1; mở CCreateDlgRenew, ảnh screen-10. Không sửa cờ hoặc index. |
| `create-confirm` | Click xác nhận343,479; modal trả3, DB3550/DB35E0 và C→S81/107byte trên socket4. Chưa có response tạo phòng hoặc room stage. |

Breakpoint81524F được chuẩn bị lệch instruction trong control chưa nhận91, không hit; sửa815250
trước run list91. Không dùng vị trí sai làm quan sát. Ranges D691D0/DB232F bắt đầu giữa instruction
được thay bằng block đã xác định từ dispatcher/prologue; raw cũ giữ để truy lỗi phép đo.
Metadata fixture/runner và fixture-oracle/capture-oracle ban đầu trùng tên; đã tách `*-fixture.json`,
`*-fixture-oracle.json`, giữ runner/capture, tái tạo và assert byte fixture như cũ rồi chạy lại oracle.
Các correction chi tiết trong `measurement-corrections.json`; không sửa log run hoặc CP015 để làm khớp.

## Fixture và kiểm

Channel socket:17 seq43,241 seq46 như CP015;91 seq49;14 seq52. 91/14 gửi sau marker67 để đo,
không chứng minh67 là request ghép cặp cả ba response. Count0 room-list, một kênh kind10/index0,
level0..65535, name rỗng đều là lựa chọn emulator. Không kiểm count lớn, compressed/cacheflag0,
nonempty channel name, password/title variants hoặc server policy gốc.

`verify-list91.py`, `verify-channel14*.py`: native14EF300 + first field, positive/badCRC/partial;
**first-field pass không kiểm body**. `verify-blob14.py` trên mẫu sửa chạy complete outer/blob reader
đến inner33byte/count1; allocator/free mô hình hóa, side effect405E70 bỏ qua, không giả là oracle toàn cache.
Oracle đầu bị chặn bởi external HeapAlloc/size assertion của mẫu thiếu selector; giới hạn ghi riêng.
`replay-run.py` dùng capture thực, seq40+3n reset theo socket; CRC-sai/partial không consume.
`verify-create81.py` chạy parser và mọi field getter KR của capture107byte, hai SSO strings/12u32/4u8,
cursor69 bằng payload69. Tên/nghĩa mọi u32 vẫn chưa khẳng định.

`verify.py`: **2.072 kiểm tra /14 run pass**, raw hash, isolation/copy/harness, log markers và native cases;
đọc code, ảnh và timeline làm thủ công. 5 artifact nguồn kiểm nguyên vẹn. Không có review độc lập,
không tự đóng gate. AUD005 hoàn tất ở CP016, audit định kỳ kế CP020; trạng thái merge lấy từ PR.
