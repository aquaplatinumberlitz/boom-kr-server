# CP023 — writer event, bảng kết quả và hai vòng thử trên Wine

Contract duy nhất ở [PROTOCOL](../../PROTOCOL.md#round-result); thứ tự tiếp ở
[RESEARCH](../../RESEARCH.md#q-023), task/gate ở [PLAN](../../PLAN.md).
Baseline `5eafd6698eb85ca301040b30d568d940fc3e2eff`, CP022/PR30 đã merge.
Codex solo RE/runtime/tự kiểm; review độc lập **waived**. CP023 là snapshot trước tích hợp,
PR nhánh `kre/round-repeat-runtime` giữ trạng thái merge hiện tại. AUD006 ở CP020; kế **CP024**.

Raw `local/research/codex/round-repeat-r1/`; [manifest](../checkpoints/manifest.json) khóa
script/fixture/native oracle/log/ảnh và phụ thuộc bằng SHA/size. `*-hashes.json` phủ mọi output từng
run sau end timestamp; không thay raw/checkpoint CP022 trở về trước. Binary/disassembly/capture không commit.

## Target và recipe

Runtime copy `CA-r2-stage-social.exe`, SHA256
`e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef`;
static copy `CA-r2-login-logout.exe`, SHA256
`660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3`;
modO memory-layout base400000, SHA256
`a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5`.
Không thêm patch binary; import/API/guard chẩn đoán kế thừa [CP020](../start-reply/README.md).
`static-runtime-code-match.json` so chín vùng code tại VA/RVA=VA−400000.

Tám run **tuần tự**, prefix/output riêng, harness giữ lease trong namespace; user/net/mount/PID
namespace chỉ lo/no route, NSS files-only, peer loopback. Cache/game-dir dùng chung, không suy cold/warm.
`run.json` ghi hash/capture/timeline; `*-recipe.json` có argv. Dùng tên mới khi tái tạo:

```bash
python3 local/research/codex/round-repeat-r1/run_probe.py NEW-RUN CA-r2-stage-social.exe 145 wine_probe_repeat2_ready.py local/research/codex/round-repeat-r1/repeat2-ready.gdb
python3 local/research/codex/round-repeat-r1/verify.py
```

Harness cuối mở bound150s để quan sát vòng hai, run145s; không ép stage/104/timer bằng debugger.
Stub vẫn phát fixture được chọn; không phải server đầy đủ. VN `oc_e-game`/`packets.rs` chỉ dẫn hướng
result/blob/death; `vn-result-navigation.json` khóa nguồn, không áp opcode hoặc enum offset.

## Run và kiểm thủ công

| Run | Kết quả / giới hạn |
|---|---|
| event127 | Baseline full180; probe13ADD10 trùng breakpoint có continue nên không bắt127, không phải absence proof. |
| event127-direct | Bỏ breakpoint trùng: bắt stack trap17/death19; hai frame được giải hết bằng native prefix/blob/checksum. |
| rank175-v2 | Lỗi import screen00 timeout trước TCP/observations; final metadata/cleanup/raw giữ nguyên, không tính là phép kiểm175. |
| rank175-v2-retry |175 outer302/302/inner292/292/errors0; screen22 có LOSE/PERFECT, sau180/full3 về phòng. Trigger còn theo thời điểm thử. |
| repeat2 | Đổi trigger sang checksum-checked reported19→175→180; OS click tức thời trận hai chưa phát104. |
| repeat2-hold | Giữ chuột1s vẫn chỉ có callback Start đầu; không thêm104. Hai writer virtual21C thực bằngB53010. |
| repeat2-refresh | Sau return gửi own110/seq97; click sau tới callback, start-check3, chưa104. |
| repeat2-ready | Thêm guest-ready110/seq100: start-check0, actual104 lần hai, đủ121/197/196/198/159/158/175/180, hai scene/HELP/score/return trên cùng channel4. |

Ảnh đã đọc: `rank175-v2-retry/screen-22.png` bảng kết quả đầu;
`repeat2-ready/second-after-space.png` scene/input vòng hai,
`screen-30.png` HELP, `screen-31.png`/`screen-32.png` bảng kết quả vòng hai,
`screen-33.png`/`screen-35.png` phòng sau vòng hai.
Log có hai pointer actor/bomb/manager khác nhau và hai lần full cursor/phase3; native replay capture
có hai104, hai99, hai118. Không dùng alive/click/encoder/stage request riêng để kết luận.
Observation elapsed lấy trước ảnh/thao tác, không phải timestamp chính xác của mỗi effect.

`repeat118-native-fields.json` đọc hai actual118: serial candidate0 cả hai, tọa độ100/1→100/3;
reply159 vẫn fixed100/1/id1. Request104 lần hai wire461 thay vì109, framing/CRC pass nhưng body growth
chưa giải thích. Delay blast→reported19 khác6.5s/0.9s. Đây là giới hạn spatial/reset/timer thực,
không gọi hai vòng thử thành K5/K6 hoặc multiplayer hoàn chỉnh.

## Verification và correction

`verification.json`: **2892 kiểm /8run**, gồm hash mọi output, isolation/lease/copy/harness,
native replay C→S với CRC sai/frame thiếu, **84ca framing/28fixture**, full127 envelope/blob/checksum,
header+16record175, byte-match chín vùng code, timeline reported19 và hai vòng callback/capture.
`runtime-summary.json` giữ markers; `measurement-notes.json` giữ lỗi probe/correction/giới hạn.
Kiểm script là cơ học; code/ảnh/timeline được đọc thủ công riêng, không tự suy semantics hay đóng gate.

Các correction chính: prefix127 là14byte thay partial10; blob native cần allocator mapping đúng;
header175 signedFF tắt extra7byte/row, record131 và292raw thay draft306 chưa gửi.
Probe manager với synthetic unconstructed state thất bại không phải lỗi runtime client.
modO là memory-layout, không remap như PE disk image để so byte. Guard printf đọc pointer wrapper,
không dùng pointer làm raw state. Draft guest-state sửa field obfuscation trước runtime.
Các draft/disassembly window sai alignment và probe thất bại giữ provenance trong raw/notes.

Một client modified/guest tổng hợp, policy local-loss/guest-win đã chọn theo báo cáo19.
Chưa arbitration/validation/dedup, inner-field semantics đầy đủ, spawn/collision/reset/timer chính xác,
item/pickup/chain đầy đủ, hai client/peer, full auth/cache/vanilla hoặc gate mới.
