# F2: probe Wine hai client r3/r4

Sáu lượt tới `started-ok`: controller ghi nhận start, hai client báo tải, phân phối thông tin peer và journal mở đúng map. Bảy lượt còn lại dừng trước start và không có `capture-norm.jsonl`. Đây là bằng chứng `kr_runtime` cho đường khởi động trên client KR `00967` với server cục bộ. Luật chơi riêng của mode, quyền quyết định trạng thái và kết quả đủ ván vẫn `unknown`.

Nguồn pin là `local/cache/kr-bin/f2f3-probe-r3/bins.json`: `git_head` `72635de6bdbf22f75b31f42e7bc4503572370f40` và SHA-256 của `kr-service` `6e2e3a40a889a0dd36b88c039c212d60f6d59b1414d2c4dc9446c3a2de99862f`. SHA-256 của binary trên đĩa khớp manifest và `identities.json` của cả hai đợt. Các lượt dùng Wine `wine-10.0-20260425 (CachyOS)` và cùng hash bản sao client cho A và B, ghi trong [bằng chứng JSON](../evidence/item-families/f2-wine-probe-r3-r4.json).

## Sáu capture phân tích

Chạy [analyzer](../../tools/kr_f2_f3_capture_analyzer.py) riêng cho từng capture ở chế độ `--host-a`. Mỗi lượt có cửa sổ quan sát cấu hình 25 giây. Cột TCP và sự kiện chuẩn hóa lấy từ `batch-summary.json`; `event_count` của analyzer khớp số sự kiện chuẩn hóa. Cột `C→S 15` tính cả các frame bị observer giữ lại.

| Lượt | Mã map | Frame TCP | Sự kiện chuẩn hóa | Cặp chiều/opcode | C→S `15` (tổng / giữ lại) |
|---|---:|---:|---:|---:|---:|
| `r3/Coloring01` | 213 | 135 | 147 | 40 | 35 / 12 |
| `r3/Hamster01` | 355 | 133 | 145 | 41 | 33 / 12 |
| `r3/Gumiho01` | 344 | 134 | 146 | 40 | 35 / 12 |
| `r3/SuperFighter16` | 1075 | 136 | 148 | 41 | 37 / 12 |
| `r4/Rabbit01` | 920 | 134 | 146 | 40 | 35 / 12 |
| `r4/Wriggle01` | 1115 | 133 | 145 | 41 | 33 / 12 |

Observer giữ lại 6 frame C→S `15` từ mỗi client sau S→C `153`; mỗi body dài 31 byte. Các frame bị giữ lại vẫn có trong capture chuẩn hóa. Vì vậy số sự kiện chuẩn hóa cao hơn số frame TCP đúng 12 ở mỗi lượt. Analyzer đếm `15` theo chiều và vai trò, chưa giải mã body của opcode này.

| S→C opcode | Frame quan sát / mẫu mỗi lượt | Body | Kết quả analyzer và anchor tĩnh |
|---|---:|---:|---|
| `110` | 12 / 8 | 6 byte | `opaque_static_layout`, case 90, `0xe5e2c0`. |
| `153` | 2 / 2 | 97 byte | `opaque_static_layout`, case 133, `0xe5e6d0`. |
| `196` | 2 / 2 | 7 byte | `opaque_static_layout`, case 176, `0xe73760`. |
| `197` | 2 / 2 | 14 byte | `ok`, case 177, `0xe751f0`; cả hai mẫu ở mỗi lượt có các trường `[12345, 0, 120, 30, 1]` theo thứ tự `u32, u8, u32, u32, u8`. |
| `170` | 1 / 1 ở `Hamster01` và `Wriggle01` | 21 byte | `opaque_static_layout`, case 150, `0xe85b60`; không thấy ở bốn capture còn lại. |

Analyzer giới hạn tám mẫu mỗi cặp chiều/opcode. Các case trên dùng `opcode - 20` theo [bảng nhảy tĩnh](f3-pve-monaction-wire-re.md); giá trị `ok` cho biết analyzer đọc được các trường, không xác nhận client chạy handler hay ý nghĩa của từng trường. Sáu capture đều có C→S `99`, S→C `121` và `153` (mỗi opcode hai frame/lượt). Không thấy S→C `185` hoặc `204`, hay C→S `132` hoặc `180`, trong cửa sổ đã đo. Không có lượt đổi host, nên kết quả về quyền quyết định vẫn `inconclusive` / `unknown`.

## Lượt dừng trước khi bắt đầu trận

- **`r4/CAPong01`:** timeout khi chuyển `native-lobby` → `host-room-setup`. Controller ghi đã vào sảnh; service ghi đã nhận lựa chọn phòng `81`, nhưng không ghi thiết lập phòng. `service/rust-events.jsonl` ghi hai lần kết thúc owner với `account_worker_queue_expired`. Đây là trục trặc ở pha vào phòng của lượt này, trước yêu cầu chọn map. Chưa có lượt thử lại trong hai batch để kiểm tra tính nhất thời hoặc xác định nguyên nhân gốc.
- **`r3/MonSnowBear07` và `r4/Zzolman01`, `r4/PopcadeB01`, `r4/PopcadeC01`, `r4/MonGolem01`, `r4/MonPen01`:** controller đã qua `r1-map-request` và `r1-ready`, rồi dừng trước `r1-start`. Service ghi S→C `110` cho hai luồng, không có yêu cầu start `104` hay lần ghi S→C `121`. Log có `native_stream_eof`; chưa xác định EOF là nguyên nhân hay xảy ra khi đóng lượt. Không có capture chuẩn hóa để phân tích các lượt này.

`batch-summary.json` ghi controller trả exit `1` cho cả bảy lượt trên. `cleanup.json` của cả 13 lượt đều ghi `scratch_removed: true`, `namespace_ended: true`, `namespace_exit: 0`; đây là kết quả dọn dẹp của namespace. Tổng `deleted_bytes` là `60 155 781 334` byte.

## Tái lập phân tích

Từ gốc checkout, đặt `PROBE_ROOT` thành thư mục chứa hai đợt nếu dữ liệu nằm ngoài worktree. Nguồn mặc định là `local/runs/f2f3-wine-probe`; JSON giữ hash của hai `batch-summary.json`, hash từng capture và số đếm opcode. Lệnh sau chỉ đọc capture và in báo cáo:

```bash
probe_root="${PROBE_ROOT:-local/runs/f2f3-wine-probe}"
for run in r3/{Coloring01,Hamster01,Gumiho01,SuperFighter16} r4/{Rabbit01,Wriggle01}; do
  python3 tools/kr_f2_f3_capture_analyzer.py --host-a "$probe_root/$run/capture-norm.jsonl"
done
```

Để phân biệt lỗi thao tác UI với lỗi xử lý start, phép đo tiếp theo cần đối chiếu thao tác sau `r1-ready` và việc gửi/nhận yêu cầu start với một map đã tới `started-ok`, trên cùng binary. Kiểm tra riêng `CAPong01` ở bước thiết lập phòng trước khi quy vấn đề cho map.
