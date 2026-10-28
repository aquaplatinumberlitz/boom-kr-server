# Phản hồi tạo phòng và init phòng chờ — CP017

Owner Codex, KRE-005b/issue14; tự kiểm, [miễn review độc lập](../checkpoints/review-waiver.json).
Baseline `a4d28a9af3837710813652a3ab79f3907590027e` (CP016/PR24 đã merge).
[PROTOCOL](../../PROTOCOL.md#room-create) giữ contract duy nhất;
[RESEARCH](../../RESEARCH.md#q-023) giữ câu hỏi/phép kiểm tiếp. Không có gate mới.

Raw `local/research/codex/room-reply-r1/`; checkpoint manifest khóa size/SHA của script,
static locator, oracle, metadata và hash index bao phủ mọi file run. Năm run đầu ngày28/10 UTC
khoảng08h, run prepare-cursor ngày28/10 UTC23h (29/10 JST). Đây là cùng nhóm mốc,
không gọi các run cách nhau nhiều giờ là kiểm cold/warm cache. Chưa có PR khi ghi record;
trạng thái tích hợp lấy từ PR, CP016/history không sửa.

## Artifact, recipe và rollback

Cùng diagnostic copy `local/research/codex/wcp-guard-r1/game/CA-r2-login-logout.exe`, SHA
`660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3`; không patch exe/DB mới.
Kế thừa identity1/tên rỗng và các thay đổi CP006–015; memory1A966FC=0 sau guard123246D.
Không ép stage, modal, enable/index/level hoặc region guard. Chỉ stub local sau request đã capture.
Source binary/DB bất biến; năm artifact nguồn kiểm SHA/size nguyên vẹn.

Runner `run_probe.py` ghi argv, mỗi run prefix/Xvfb/private user/net/mount/PID namespace,
loopback-only/no route/NSS files-only. Shared game-dir có CALog before/run; cache không chuẩn hóa.
REA/IDA provider vẫn chưa sẵn như CP015/016; tiếp tục GDB Wine, PE/Capstone mục tiêu và native
Unicorn, không sửa cấu hình REA/IDA. VN chỉ đọc snapshot đã có, không chạy client/server VN.

Tái lập với label mới, không ghi đè raw đã khóa:

```sh
python3 local/research/codex/room-reply-r1/run_probe.py replay-new CA-r2-login-logout.exe 44 wine_probe_create_reply86.py local/research/codex/room-reply-r1/prepare-cursor.gdb
```

Harness `wine_probe_create_reply86.py` nối CP016 nhưng chỉ gửi reply86.bin một lần sau C→S81
trên connection4. Rollback: harness control `wine_probe_create_confirm.py` không trả create,
cùng copy rồi kết thúc namespace/prefix của run. Giữ nguyên game/source và raw lịch sử.

## Run và bằng chứng

| Run / thời gian đo | Quan sát / giới hạn |
|---|---|
| receiver /44s | Không trả create; capture81, đo CNormalLobbyStrategy và listener CLobbyStage+61E8; không stage12. |
| registry /42s | Registry77..85 ở1A34478 thực tế NULL khi gửi81; không stage12. Không suy mọi opcode/region không có pending handler. |
| reply92 /50s | Candidate92 tới lobby; feature=1/region=0, không strategy success/stage12. Mẫu bool ban đầu có lỗi dự đoán, xem correction; body chưa được runtime reader này thực thi. |
| reply86 /48s | Actual UI handler86/result0→DB29A0, đọc fields và yêu cầu12. Ảnh screen-11 có UI phòng chờ, ô/map trống. |
| prepare-init /46s | Lặp fixture86, thực sự vào DC1AB0/vt1767A80; stack request585 nối về init. Cursor probe chưa chạy vì duplicate breakpoint. |
| prepare-cursor /44s | Lần ba tới12, init class; cursor11/payload11/error54=error58=0; request585 context+7C=0. screen-13 có UI phòng chờ. |

Thời gian không gồm wineboot/khởi động harness. C→S81 dài105–110byte giữa các run
(CP016 chỉ khóa mẫu107byte); không coi mọi title/variant có cùng kích thước hoặc đã kiểm schema.
Captures dùng parser KR thật, không lấy stub log thay bằng chứng handler hoặc UI.
Chưa có player trong ô, map, ready/start hiệu lực, load/Battle, multiplayer hoặc full login/profile/cache.

## Oracle, hash và correction

- Reply86 fixture49byte SHA `f7d21cf19f44a9d4c2a17d55c1090af1e2145689fc17e39c2e0e299574c2a4e2`;
  metadata lựa chọn field là emu_choice/candidate, hiệu ứng handler/stage là kr_runtime/confirmed
  trong scope copy. Không coi fixture là capture server gốc.
- `verify-reply86.py`, `verify-reply92.py`, `verify-reply92-false.py`: tổng9 native cases,
  frame14EF300 và mọi primitive body/cursor, CRC-sai/partial. 92-false chỉ emulation, không runtime.
- `replay-run.py`: tất cả frame C→S socket3/4 của6 run, seq40+3n reset từng connection;
  positive/CRC-sai/partial.585 payload một u32 được native getter đọc=0 trong ba positive run.
  Mô hình receiver/buffer/allocator và tắt prefix check; không chứng minh server semantics hoặc full codec.
- `verify.py`: **1.343 kiểm tra /6 run pass**, hash/isolation/unchanged-copy, runtime markers,
  all captured frame cases, full fixture fields. Đọc code, timeline và ảnh thủ công riêng.
- `measurement-corrections.json`: giữ lỗi bool92, launch không bị chặn bởi assertion đầu,
  probe cursor trùng, label REPLY92 còn sót ở run86, dump vượt cuối vtable và ranges lệch instruction.
  Không đổi kết luận để làm pass: ghi kỳ vọng bool đúng từ reader, tách fixture false và chạy lại native;
  negative92 không nâng thành body success. Cursor chỉ được xác nhận ở run sửa riêng.

Bài học VN dùng đúng phạm vi: snapshot `auth-request-r1/vn-login-references/docs/lobby.md`
có request77/reply83→strategy→stage12; giúp lần theo strategy/listener. KR đo81/86 độc lập,
không cộng hằng số opcode hoặc dùng nguyên payload VN. Locator106/player và request585 chỉ dẫn
thí nghiệm tiếp trong RESEARCH; chưa có schema đủ để gửi player/map fixture.

CP017 complete nghĩa record/docs đủ, review waived; chưa chứng minh K3/K4. AUD005 giữ nguyên ở
CP016; audit định kỳ kế CP020. Check docs/strict manifest/history và artifact verify theo workflow.
