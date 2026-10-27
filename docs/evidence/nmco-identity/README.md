# CP007 — identity local, khởi tạo cửa sổ và blocker audio

Codex RE ngày 27/10/2026, issue14; trạng thái review/integration và SHA từng raw file ở
[manifest](../checkpoints/manifest.json). Các run là Wine 10 trên bản copy đã patch trong `local/`,
user/net/mount/PID namespace chỉ loopback, NSS files-only; không kết nối server thật. Kết quả không
khẳng định client nguyên bản, identity hợp lệ, login, sảnh, gameplay hay gate.
[CP006](../wcp-sequence/README.md) đã được review và merge PR16; fixture WCP 15 byte và tám
response tổng hợp 317 byte được giữ nguyên từ CP006. Raw packet/log chỉ lưu local.

## Phép thử nhóm

- `debug`: patch call `13CA139`, giả lập `EDX:EAX=0:1`; debugger tại `13CA13F` thấy cặp này,
  caller `13BE369` chạy tiếp tới trap slot `16EA8F0` (tên candidate `NMCOGetMyScreenName`).
  `13CA130` ghi cặp 64 bit và trả boolean; đây là giá trị **emu_choice**, không phải account thật.
- `debug-heap`: screen name giả thất bại/rỗng, gặp heap fault đọc `0x460`. Copy kế reset hai guard
  `19D8770/19D8790`; `debug-heap-progress` dùng heap handle hiện hành và đi tiếp tới trap
  slot `16EA8BC` (candidate `NMCOChangeMyInfo`). Hai run/prefix riêng, nên đây là quan sát có
  giới hạn và giả thuyết guard cũ, không chứng minh toàn bộ heap đã đúng.
- `debug-no-info` bỏ call change-info (cdecl, caller cleanup), hostname cục bộ không resolve trong
  NSS files-only; `debug-host` ánh xạ hostname sang loopback rồi đi tiếp tới trap slot `16EA654`.
  Hai run khác prefix/thời gian; dữ kiện phù hợp giả thuyết cấu hình hostname, không chứng minh
  đây là nguyên nhân duy nhất.
- Call site `14E2DDB`/`14E2E45` và các xref phù hợp `GetWindowLongA`/`SetWindowLongA`; copy
  thay import `16EA654`/`16EA66C` candidate. `debug-window` thấy getter trả style;
  `debug-metrics` thấy setter trở về, rồi gọi `16EA5C8` với 0/1; khi thử bind candidate
  `GetSystemMetrics`, kết quả là 1024/768. Chỉ xác nhận API đã thử trên copy, không chốt import gốc.
- `debug-metrics` gặp `nxoverlay.dll` ordinal2; `debug-overlay` bỏ hai call ordinal2/3 có caller
  cleanup và đi tiếp đến trap `16EA67C`. Candidate `GetClientRect` tại `80798F` trả1 và RECT
  `(0,0,800,600)` trong `debug-clientrect`, rồi tới `12382E7`. Candidate chưa phân biệt hoàn toàn
  với `GetWindowRect` bằng bằng chứng nguồn gốc. Bỏ overlay có thể bỏ side effect cần về sau.
- Sau đó `debug-clientrect` dừng ở assertion `mmdevapi_private.h:67` trong Wine. Thử độc lập
  `debug-null-audio` với ALSA default null, trace vẫn dừng assertion sau `alsa_create_stream`.
  Cả hai dùng copy/prefix riêng và debugger; chưa biết nguyên nhân gốc hoặc ảnh hưởng của debugger.

Các patch identity, name, heap, NMCO và overlay ghi before/after, SHA source/target, rollback trong
metadata local đã hash ở manifest. Các import thử được gắn vào PE copy, kèm script và metadata local;
không thay recipe R2 đã merge. [Verification](verification.json) là kiểm cấu trúc/byte/log hữu hạn,
không phải oracle ngữ nghĩa. Xem [RESEARCH](../../RESEARCH.md#q-023) cho phép kiểm tiếp.
