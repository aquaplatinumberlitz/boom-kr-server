# CP011 — UI login và giới hạn input

Các run đo27/10 UTC, đồng bộ CP011 ngày28/10 JST. Owner Codex, issue14; review độc lập được
[chủ dự án miễn](../checkpoints/review-waiver.json). Codex tự kiểm; đây không phải approval độc lập.
Counter, SHA raw và trạng thái tích hợp ở [manifest](../checkpoints/manifest.json).

## Mốc UI

Copy `CA-r2-no-overlay-tick.exe`, SHA `70708abab77ebd1cfd230a35f4997b8e84330f74442ed77b0b74c7b03180c82c`,
kế thừa chuỗi sửa CP006–010. Chỉ thay call6 byte `807B25` bằng NOP để bỏ ordinal9 candidate;
wrapper không push stack argument cho call, ý nghĩa return/caller và overlay về sau chưa xác nhận.
Tên ordinal9 là **candidate từ excluded-import report**, không phải tên API đo trực tiếp runtime.

- `no-overlay-tick`: ảnh `screen-06.png` cho thấy UI chọn server, ô ID/password và nút xác nhận.
  Không thấy fatal exception trong log35 s; screenshot là bằng chứng UI, process alive không dùng làm gate.
- `login-input`: focus/click/type dữ liệu giả bằng xdotool trong Xvfb riêng, chụp trước/sau xác nhận.
  Lệnh trả0 nhưng ảnh chưa cho thấy ô nhập đổi; không suy game đã nhận input.
- `login-held`: tăng mouse hold250 ms và key hold100 ms, vẫn chưa quan sát ô nhập đổi hoặc request mới.
  Hai run này chưa kiểm handler bằng breakpoint; phép GDB kế tiếp được tách rõ bên dưới.

Cả ba run chỉ thu66 byte C→S cũ. Chưa có auth, sảnh, gameplay hoặc gate K3; packet15 log vẫn
không có capture wire mới. Hook input đã bị bỏ, native key-state binding là candidate, nên chưa
xác định nguyên nhân input chưa có tác dụng. `CAWH.sys` trên host chỉ được đọc tĩnh: artifact
hash riêng có một export Hook tại RVA1006 với bytes `xor eax,eax; ret12`; vendor-original provenance
và vai trò thực của artifact này chưa xác nhận. Không dùng điều này chứng minh hook là nguyên nhân.

Recipe/argv, harness và patch before/after được hash trong manifest. Các run giữ namespace
user/net/mount/PID, loopback, NSS files-only và Xvfb; chỉ dùng identity giả với stub local.
Nguồn chuẩn câu hỏi tiếp ở [RESEARCH](../../RESEARCH.md#q-023).

## Bổ sung/correction theo review CP010

Report độc lập trên head `093fd416965a294e674d67483110a72ba7b80766` còn một P2 về provenance;
report giữ nguyên trong raw và [PR19](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/19#issuecomment-6437700488).
CP010 complete record không sửa findings; CP011 bổ sung/correct rõ:

- Hash toàn bộ `rebuild-r2-final.json`, output R2 SHA `7b3b435b…`, và excerpt `static-provenance.json`.
  `16EA4E8 → nxoverlay.dll #9` và `16EA064 → dwk.dll UninstallDWK` chỉ là excluded-import candidates.
- `critical-cleanup-proof.asm` ghi `14F11DC → [16EA2B0]`, binding candidate `InitializeCriticalSection`,
  và nhánh `14F127A` dùng `this+0C`. Không nâng mapping gốc thành confirmed chỉ vì copy đã chạy.
- Byte call `122E7E5 e856020000` trỏ `122EA40`; gọi đây là routine trên đường cleanup.
  Kết luận trước gọi chính routine này là destructor chưa có đủ chứng cứ và được rút lại.
- Rút mapping candidate `16EA5FC=RtlEnterCriticalSection` cho copy đang thử, dùng `SetClipboardData`
  theo ngữ cảnh clipboard. Việc này không rút quan sát WCP/packet0 cũ, cũng không khẳng định các run
  CP006–009 đã kiểm slot này. Những caller chưa chạy vẫn chưa kiểm.
- Attach WineDbg không cung cấp stack fault chính. Báo `FFF40200` được giữ raw, chưa gán chắc actor.
  Các `.S/.o/.bin` của recorder là trung gian; builder thêm replay/jump vào PE cuối. Snapshot giữ
  lần ghi cuối; hai run throw có một chuỗi first-chance exception mỗi run, với biểu diễn32/64 bit.

Các kiểm byte/hash/snapshot hữu hạn và docs check không tự xác nhận ngữ nghĩa, auth hoặc gate.
Chính sách miễn review chỉ đổi quy trình review/merge; các lỗi chất lượng đã biết vẫn phải sửa.

## Áp dụng tham chiếu VN sau hai phép input

[Evidence VN→KR](../vn-reuse/README.md) giữ nguồn chuẩn cho hai run GDB bổ sung:
đã đo message input tại DispatchMessage trên KR copy với audio tắt trong prefix riêng;
custom control vẫn chưa thay đổi và tổng C→S vẫn66 byte. Không còn giữ “chưa biết input có tới
API dispatch không” như câu hỏi mở; handler/hit-test phía sau vẫn unknown.

CP012 đã đo handler và command xác nhận; xem [mốc mới](../ui-handler/README.md). Phần trên là lịch sử CP011.
