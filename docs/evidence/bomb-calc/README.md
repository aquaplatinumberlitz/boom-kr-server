# CP022 — nổ, HELP và trở về phòng trên copy Wine

Contract duy nhất ở [PROTOCOL](../../PROTOCOL.md#bomb-cycle), tiến độ/gate ở [PLAN](../../PLAN.md),
câu hỏi và thứ tự tiếp ở [RESEARCH](../../RESEARCH.md).
Baseline `96a0e48efd76a50d174c889b18c1f8b9cfcb397e` (CP021/PR29 đã merge).
Codex solo RE/runtime/tự kiểm; review độc lập **waived** theo chủ dự án.
CP022 là snapshot trước tích hợp, PR nhánh `kre/bomb-calc-runtime` giữ trạng thái merge hiện tại.
AUD006 tại CP020; audit định kỳ tiếp **CP024**, không reset theo phiên.

Raw `local/research/codex/bomb-calc-r1/`. Record CP022 trong
[manifest](../checkpoints/manifest.json) khóa SHA/size script, fixture, native oracle, run/log/ảnh.
`*-hashes.json` phủ toàn output từng run sau khi `run.json` đã có end timestamp.
Không commit binary/disassembly/capture; checkpoint và raw CP021 trở về trước giữ nguyên.

## Target, giới hạn và recipe

Runtime copy CP020 `CA-r2-stage-social.exe`, SHA256
`e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef`; không thêm patch CP022.
Static copy `CA-r2-login-logout.exe`, SHA256
`660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3`.
Native oracle modO base400000 SHA256
`a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5`.
`static-runtime-code-match.json` so byte sáu vùng code/table. Locator là VA, RVA=VA−400000.
Các bind/import/API/guard kế thừa và giới hạn modified copy ở [CP020](../start-reply/README.md).
Không ép phase/timer/kết thúc từ debugger; stub chọn inputs và thời điểm thử.

Năm run **tuần tự**: prefix riêng, user/net/mount/PID namespace chỉ lo, không route outbound,
NSS files-only, peer loopback. Game-dir/cache chung không chuẩn hóa; cold/warm/reconnect còn mở.
Harness từ kind0 giữ `fcntl` lease **bên trong PID namespace**, đã đo held khi run cuối hoạt động;
ngăn các harness cùng áp dụng lease chạy chồng, không bảo vệ legacy harness chưa dùng lease.
Seal yêu cầu final run.json để tránh lặp lỗi recovery CP021. Không suy lease là proof protocol.

Mỗi `*-recipe.json` có argv; `run.json` có image/harness SHA/PID/network/timeline/capture.
Tái tạo dùng tên output/prefix **mới**, không ghi đè run cũ:

```bash
python3 local/research/codex/bomb-calc-r1/run_probe.py NEW-RUN CA-r2-stage-social.exe 98 wine_probe_result180_full.py local/research/codex/bomb-calc-r1/result180.gdb
python3 local/research/codex/bomb-calc-r1/verify.py
```

`vn-bomb-navigation.json` khóa hash VN oc_e-game/session: chọn nhánh kind0 và hướng result.
KR2u32/u8 thêm, subtype prefix17/19 và bool đầu180 được đo riêng; không áp opcode/enum offset.

## Các run và kiểm thủ công

| Run | Quan sát và giới hạn |
|---|---|
| calc-guard | E9C520 chạy trước159 ở stack đặt và sau158/kind1; kind1, getter100/1/power1, cell khác0, qua spread/end. Không phải early guard reject hoặc detonation. |
| kind0 |158/kind0/13byte đọc13/13/errors0, E14360 tìm cùng bom rồi E9E660. Ảnh sau bom biến mất, vật cản gần đó thay đổi; move Right tránh ô bom. |
| kind0-burst |Giữ local tại chỗ sau Space;18ảnh burst có nước/HELP/local trong bong bóng. Ảnh24/26 sau đó local biến mất/roster đổi. Native127 prefix17/19 chỉ là partial navigation, chưa full death contract. |
| result180 |Sau nổ/HELP, stub gửi180 sau15s. UI về phòng nhưng callback54=1 dù cursor2/2. Giữ short-body control, không gọi full valid result. |
| result180-full |Thêm bool đầu thành3byte; runtime3/3/errors0, tự gọi stage12 và UI phòng có KRLocal/KRGuest/nút Bắt đầu. Trigger15s là test policy, không tự quyết outcome. |

Ảnh chọn `kind0-burst/blast-00.png` trước hiệu ứng, `blast-01.png` nước/HELP,
`blast-07.png` bong bóng; `screen-24.png`/`screen-26.png` local biến mất.
Run cuối `blast-01.png` lặp lại nước/HELP, `screen-29.png` là UI phòng.
Đọc ảnh cùng log/capture/đúng run, không lấy alive/TCP accept/stage request riêng làm proof vòng trận.

## Verification và correction

`verification.json`: **1630 kiểm /5run pass** về thứ tự, isolation, pinned copy/harness,
hash raw, native replay mọi C→S với CRC sai/frame thiếu,9ca framing/3fixture mới,
getter native, runtime cursor/control/cleanup/stage và lease/burst completeness.
`runtime-summary.json` giữ markers, `measurement-notes.json` giữ giới hạn và sửa giả thuyết.
Kiểm chỉ đo được tính nhất quán; đọc code/ảnh/timeline thủ công riêng, không tự đóng gate.

Quan trọng: oracle framing/two-getter mẫu1802byte không chạy callback đầy đủ.
Getter virtual38 thực tiêu thụ một bool/u8 trước hai u8 tiếp; runtime đầu báo54=1 nhưng vẫn về12.
Mẫu sửa3byte mới có full cursor và không lỗi. Raw/mẫu đầu giữ nguyên như lịch sử.
Native context/buffer mô hình; phép kiểm emulation không thay runtime callback.

Nổ/HELP/về phòng đã có proof trên copy. Chưa bảng kết quả/rank175, arbitration tự động,
serial/dynamic echo/reset trận kế, full death/authority/collision/spawn (getter100 và prefix y4039 chưa hiểu),
item/pickup/chain đầy đủ, hai client/hai trận, full auth/profile/cache/vanilla hoặc pass K2–K5.
Không gọi đây là một trận hoàn chỉnh đã nghiệm thu. Bước tiếp giữ duy nhất ở RESEARCH.
