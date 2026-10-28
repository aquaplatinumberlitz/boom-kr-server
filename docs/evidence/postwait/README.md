# Từ helper sau auth tới UI sảnh — CP015

Owner Codex, tự kiểm; review độc lập [được miễn](../checkpoints/review-waiver.json).
Baseline `cb9e0b5550aea9fbfe8564c989113284ccd8d57b` (CP014/PR22 đã merge).
Contract ở [PROTOCOL](../../PROTOCOL.md#channel-lobby), việc tiếp ở
[Q-023](../../RESEARCH.md#q-023); không tạo roadmap riêng hoặc đóng gate từ ảnh UI.

Raw: `local/research/codex/postwait-r1/`; file/size/SHA trong
[manifest CP015](../checkpoints/manifest.json), mỗi run có hash index kiểm toàn bộ raw.
12 run tuần tự trên game-dir copy chung; prefix/Xvfb/user/net/mount/PID namespace riêng,
chỉ loopback, không route IPv4, NSS files-only. CALog trước/sau giữ theo run; cache game-dir
không được chuẩn hóa giữa các run nên đây chưa phải cold/warm-cache gate.
IDA MCP không có database mở, REA native provider chưa cấu hình; dùng PE/Capstone phạm vi mục tiêu,
GDB proxy Wine và Unicorn như recipe trước, không sửa cấu hình tool hoặc DB nguồn.

## Copy và cách tái tạo

| Artifact local | SHA-256 | Thay đổi thêm |
|---|---|---|
| `CA-r2-login-postmsg.exe` | `de2df3ba68f2e7deff892186d51571e3eae7b613d7cb02227066015a2eb4789d` | Baseline CP014. |
| `CA-r2-login-friend-api.exe` | `7171659f64fedfe154b55ba0169c882248911fa80101ff1b4fcfe3cc3269e415` | Bind8FC→nmco3!NMCOGetFriendList. |
| `CA-r2-login-logout.exe` | `660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3` | Bind8C4→NMCOLogout sau trap cleanup; chưa có probe return riêng. |
| `nmco3.dll` trong game copy | `7e038696ec69db579f71e029da66f5b5500c56375ff77b24d93529ba7faf9493` | Export cluster/ABI đọc tĩnh; không sửa. |

`patch-friend-api.py`, `patch-logout.py` giữ before/after, slot/import/slack và rollback về copy trước.
Recipe kế thừa patch identity/API/heap/hook/overlay trước và reset memory global1A966FC=0;
không ép modal result, cờ ready hoặc stage. Không tương đương vanilla.
Metadata print đầu của friend patch ghi nhầm nhãn user32; PE descriptor thực là nmco3.dll,
đã sửa metadata và ghi correction, không sửa log lịch sử.

Runner `run_probe.py` thêm token local giả `token_1:offline-probe` và `multiserveridx:0`.
Ví dụ: label mới, exe `CA-r2-login-logout.exe`,52s, harness `wine_probe_lobby_ready241.py`,
commands `ready241.gdb`; argv đầy đủ trong recipe từng run. Harness/fixtures host-local được hash.
Rollback gồm chọn lại exe trước và runner/harness trong control tương ứng; không thay binary nguồn.

## Các run ngày28/10 UTC và delta

| Run | Phép đo/kết quả |
|---|---|
| `friend-api` | Hàm DLL thật trả0, wrapper trả0; không còn trap helper cũ. UI vẫn login, chưa đủ để suy transition. |
| `next-stage`, `stage-init` | Nhánh feature chọn inline1107B00, init trảAL1 và special-success; không yêu cầu stage10. Bác giả thuyết init-return-false. |
| `inline-block` | a/b=0/0, predicate0, flagsF4/F5=0/1, global1A31F30=0. Chưa có request5; không quy chặn này cho global stale1. |
| `flags62` | Handler thật nhận subtype1/slot0/status2, predicate đổi1, request40 và5 xuất hiện. Các status là giá trị thử có semantics chưa biết. |
| `redirect7` | Pending1, result1 và IP/port local được reader đọc đúng; socket4 nhận hello, gửi6/74byte; không có17 nên timeout/cleanup rồi trapLogout. |
| `channel17` | Serializer count0 đã kiểm native; S→C17 gửi ngay sau6, request stage11, RTTI CLobbyStage; vẽ sảnh nhưng còn notice nối kênh. |
| `lobby-wait`, `lobby-notice` | Tách polling715/BigBattle khỏi notice. Probe đầu đặt trùng breakpoint send nên không có stack715; probe sau bắt trực tiếp D9BCF0. Notice thật đi qua12D4C70, getter+6738. |
| `ready241` | Packet241 chạy base reader rồi DA5F90: ready0→1, đóng notice, notice trả1; ảnh screen-16 là UI sảnh không overlay. |
| `room-open`, `room-notice` | Click Tạo phòng được inject sau ready>2s; chưa có77 hoặc proof hit control. TimerNotice cuối run nối về app mainDisconnected khi dừng probe, không dùng làm lỗi tạo phòng. |

`channel17-timeout.gdb` chỉ được chuẩn bị, **không chạy**. Các run17 thành công ở trên không
sửa timeout30000; vẫn có nợ timeout/profile do result0-prefix thiếu body. Handler17 xuất hiện hai
lần trong log không phải hai frame wire; record stub chỉ gửi một fixture trên connection4.

## Fixture, oracle và giới hạn

S→C62(seq49) và7(seq52) ở login connection3; channel connection4 có hello riêng, S→C17(seq43),
241(seq46). Các field/default và lịch gửi là `emu_choice`. 241 được gửi sau marker C→S67 của stage
kênh để đo receive path; không coi đây là cặp request67/reply241 của server gốc. Không gửi716.
Prefix result0 vẫn cố ý thiếu user body; tên/profile còn trống dù UI đã hiện.

`verify-flags62.py`: native14EF300/u8 reader và18 case predicate10FA3A0.
`verify-channel17.py`: native frame rồi **đủ serializer840470/count0**, đúng8word thời gian;
count>0/bounds chưa đo. Fixture-oracle và capture-oracle dùng tên file riêng sau khi sửa collision
tên output; đã tái chạy phép kiểm với cùng input, không dùng capture replay thay serializer proof.
`replay-run.py` kiểm capture thật theo từng socket, seq40+3n reset riêng; CRC-sai/partial không consume.
`verify-fixtures.py` kiểm12 positive/negative/partial case; decoder KR là oracle, encoder VN chỉ là candidate.

`verify.py`: **1.014 assertions/12 run**, kiểm raw hash, source/copy/harness, marker, per-connection
capture/native replay và giới hạn input. Đây là kiểm nhất quán đo được; đọc code/ảnh/timeline vẫn thủ công.
Kiểm bàn giao: docs/checkpoints strict,5 artifact nguồn và git diff. Checkpoint review `waived`,
không có approval độc lập; trạng thái merge lấy từ PR. Counter tiếp CP016, **tới kỳ audit thủ công**.

Mốc đạt: UI sảnh và notice đóng bằng handler nhận thực trên copy. Chưa có profile/auth đầy đủ,
room create/join, trận, hai client, vanilla control hoặc gate K2/K3. Bước kế được giữ ở RESEARCH.
