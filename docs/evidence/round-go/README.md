# CP021 — clock, input và bom hiện trên map local

Contract ở [PROTOCOL](../../PROTOCOL.md#round-go), tiến độ/gate ở [PLAN](../../PLAN.md),
phép thử tiếp ở [RESEARCH](../../RESEARCH.md#q-023). Baseline
`9b02ab160bebe3806bc5054156fa84a18ee3abe6`: CP020/PR28 đã merge.
Codex trực tiếp RE/runtime và tự kiểm; review độc lập **waived** theo chủ dự án.
CP021 ghi trước tích hợp; PR nhánh `kre/round-go-runtime` giữ trạng thái merge hiện tại.
AUD006 tại CP020; audit định kỳ tiếp CP024.

Raw nằm trong `local/research/codex/round-go-r1/`. Record CP021 trong
[manifest](../checkpoints/manifest.json) khóa SHA/size của recipe, script, log, capture, oracle và ảnh.
`*-hashes.json` phủ output mỗi run, riêng159 dùng final seal sau recovery như correction bên dưới.
Không commit binary/disassembly/capture.

## Target và recipe

Runtime giữ nguyên copy CP020 `CA-r2-stage-social.exe`, SHA256
`e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef`.
Không thêm patch ở CP021. Patch/import/API/debugger guard kế thừa cùng giới hạn tại
[evidence CP020](../start-reply/README.md); không suy tương đương client vanilla.
Static copy `CA-r2-login-logout.exe` SHA256
`660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3`.
`static-runtime-code-match.json` so byte bốn vùng code đang dùng; native oracle modO base400000 SHA256
`a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5`.
Locator là VA, RVA=VA−400000. Dùng GDB/PE/Capstone/Unicorn hiện có, không sửa DB nguồn.

11 run hoàn tất:9 run không chồng thời gian, cặp159/retry là exploratory sau recovery.
Mỗi run có prefix riêng, user/net/mount/PID namespace chỉ loopback, không route outbound,
NSS files-only và stub local. Game-dir/cache dùng chung **không chuẩn hóa**; chưa kiểm cold/warm.
`run.json` ghi copy/harness/network/PID/timeline và byte TCP; `*-recipe.json` giữ argv chính xác.
Ví dụ tái tạo bằng **tên run/output mới**, không chạy đè raw cũ:

```bash
python3 local/research/codex/round-go-r1/run_probe.py NEW-RUN CA-r2-stage-social.exe 78 wine_probe_round_begin.py local/research/codex/round-go-r1/round-begin.gdb
```

`vn-round-reuse.json` khóa nguồn VN về loading/GO và clock map: snapshot lobby/packets,
`boom-re/server/src/session.rs`, `boom-re/docs/oc_e-game.md`.
Chúng hướng dẫn chọn cạnh, không chứng minh opcode/layout/authority KR hoặc tạo VN runtime mới.

## Nhóm phép đo

| Run | Kết quả và giới hạn |
|---|---|
| round-control | Baseline start121; bắt actual99 stack ở13ADD10→E7F4DD→E59587. Native capture đọc đủ23byte. Chưa gửi GO. |
| go-play | Thêm197 rồi196/mode0; đọc14/14 và7/7, phase0→2→3, bỏ pause stage clock. Ảnh timer00:00, chưa input. |
| go-time30-input | Đổi riêng timeB của197 từ0→30, thử Right/Left/Space. Vẫn timer00:00; chưa thấy di chuyển. Probe hot getter sai context không có CLOCK_SAMPLE; absence không chứng minh clock đứng. |
| begin198-input | Thêm198/13byte sau196, khởi tạo clock map running1. Ảnh timer01:56→01:50→01:47; local đổi vị trí sau Left, guest đứng nguyên. Capture118/60byte sau Space, chưa phản hồi. |
| bomb-request | Input Up/Space/Right/Space; lặp lại clock và local di chuyển lên. Có118/60byte, nhưng probe13ADD10 chỉ bắt132, chưa bắt118; không gọi đây là proof đặt bom. |
| request118-direct | Thêm probe13ADD60; capture vẫn có118, cả hai probe cấp cao không bắt118. Đây là giới hạn điểm chặn, không phủ nhận frame. |
| request118-winsend | Winsock send IAT16EA78C bắt118/60byte; stack flush qua E2D654/E7D2DB, không phải điểm tạo record. |
| request118-queue | Probe13ADCC0 bắt118 tạiE2E373; writerE2DF80/1164D80, native13getter đọc22byte. |
| reply159 | Recovery ngắt điều phối; client thực tế vẫn hoàn tất, chồng với retry. Chỉ exploratory, không independent repeat. |
| reply159-rerun |159 đọc39/39/errors0 và trả entity khác0; ảnh chưa thấy bom riêng biệt. Chồng thời gian với run159, chỉ exploratory. |
| explode158 |Run riêng sau cả hai159;159 trả CGOGameBomb, bước Right khỏi vị trí thì thấy bom.158/kind1 đọc4/4/tìm cùng pointer và vàoE9C520, nhưng ảnh sau vẫn còn bom. |

Ảnh `begin198-input/screen-16.png` có timer01:56 nhưng actor chưa vẽ ở frame đó;
không suy mọi frame đã load xong. `input-after-Right.png` timer01:51/local x392,y619;
`input-after-space.png` timer01:50/local x199,y619; `screen-18.png` timer01:47.
`bomb-request/input-after-space-7.png` timer01:51/local x392,y539, guest không đổi vị trí.
Đọc ảnh thủ công xác nhận clock và input/di chuyển **local**, chưa xác nhận getter tọa độ/collision/peer sync.

## Kiểm, lịch sử và giới hạn

`verify.py` kiểm hash, isolation, copy/harness, thứ tự run, marker/cursor và native replay mọi frame capture.
Mỗi frame có ca CRC sai và frame thiếu byte. Sáu fixture197/196/197-time30/198/159/158 có18ca framing;
`native-round-fields.json`, `reply159-native-fields.json`, `explode158-native-fields.json` kiểm typed getters;
`captured99-native-fields.json` kiểm9getters/cursor23 và `capture118-native-fields.json` kiểm13getters/cursor22.
Oracle gọi code KR trong context receiver/buffer mô hình, không thay bằng chứng callback runtime.
`verification.json` và `runtime-summary.json` giữ **4026 kiểm /11run**,9run không overlap, cùng cảnh báo
2run exploratory và3preliminary hash entries không thể kiểm nội dung cũ. Không tự xác nhận ngữ nghĩa hoặc đóng gate.

`measurement-notes.json` giữ cả phép đo âm tính và sửa giả thuyết: range E7F3B0 bắt đầu giữa instruction;
range E73760 vượt ret sang table/hàm khác; linear scan field bị data cắt nên không exhaustive.
Không dùng full-consumption do getter tự gọi để khẳng định callback nếu thiếu runtime cursor.
Static B7A760 dựng167/constant6 chưa có runtime hit và chưa phải request bom đã xác nhận.

CP021 thay trạng thái CP020 “clock/input chưa kiểm” bằng bằng chứng clock map/input local;
không sửa record/raw lịch sử. Phase3 và stage-clock unpause ở hai run đầu chưa đủ gameplay clock:
198 khởi tạo **clock map khác pointer**; kết luận giới hạn theo chuỗi đầu vào đã đo.
Fixture/default/timing là emu_choice; đã thấy bom ở một ca local nhưng chưa proof nổ, kết quả, về phòng, trận hoàn chỉnh,
multiplayer, full auth/profile/cache hoặc vanilla. Gate K2–K5 vẫn mở.


## Bom, calc và correction recovery

Contract duy nhất về118/159/158 ở PROTOCOL. Bằng chứng quan trọng của run riêng `explode158`:
`input-after-Right-9.png` có bom nước sau khi local rời vị trí; log trả entity62C84D10,
vt16FC0CC/RTTI CGOGameBomb.158/kind1 tìm lại cùng pointer/id1 và vào calcE9C520.
`screen-19.png` timer01:32 và `screen-23.png` timer01:20 vẫn còn bom. Chỉ entry calc đã chứng minh,
không gọi là nổ thành công. Không có blast/trap/result/full match; bước tiếp là guard/state/clock calc.

`reply159-interrupted.json` là ghi nhận ban đầu **đã bị bác bỏ**: sau daemon restart, không thấy
launcher/final run.json nên suy client đã dừng. Final run.json xuất hiện sau đó chứng minh client
vẫn chạy. `runtime-recovery-correction.json` giữ timeline/chồng lấn và giới hạn;
không dùng cặp159/retry làm independent runtime proof. Run `explode158` bắt đầu sau khi cả hai kết thúc.

Không ghi đè seal sơ bộ `reply159-hashes.json`:3entry client.log/xvfb.log/launcher.log tiếp tục đổi
vì run chưa kết thúc. Nội dung trước đó không được giữ riêng nên các hash nội dung cũ **không thể kiểm**.
`reply159-final-hashes.json` khóa raw cuối và được verifier kiểm; seal sơ bộ được giữ như metadata lỗi,
không giả làm toàn raw bất biến. Các raw/checkpoint CP020 trở về trước không sửa.
