# CP004 — WCP và trạng thái lỗi của bản dump

Thí nghiệm ngày 27/10/2026, Codex RE/runtime; trạng thái independent review theo
[checkpoint](../checkpoints/manifest.json). Contract hiện hành ở [PROTOCOL](../../PROTOCOL.md#wcp),
câu hỏi và phép đo kế tiếp ở [RESEARCH](../../RESEARCH.md#q-023). Đây là bản copy Wine đã sửa,
không chứng minh hành vi client Windows nguyên bản.

## Recipe và đối chứng

Dùng harness private `local/research/codex/wine_probe_wcp.py` SHA-256
`8c61e88f537f6dcfc995b64835f394923c517d7edd2b891b04eafecf249fc860` với fixture hai reply
`69000c00000003c700000000000000` SHA-256
`75be47b3067e2b1e9d2cf99b5a7eb7f949a074482409fb3eafc1a553cec53cd1`. Mỗi run thực thi trong
user/network/mount/PID namespace, chỉ loopback, không route IPv4, NSS files-only. Stub và client
chạy tuần tự; game/prefix là copy trong `local/`. Metadata/hash từng raw trong manifest CP004.

| Run | Artifact / điều kiện | Quan sát | Giới hạn |
|---|---|---|---|
| debug2/debug4 | R2 SHA `7b3b435b5d8e110185359107376b736615ceabe9452cea8154d02c8628f44c29`, prefix đã dùng | WCP consumer `13AD79C` EAX=1; `1254D50` EAX=1, `1254D60` EAX=0x7a6; `12364A0` stack args 1/0x7a6. Debug4 dừng tại call `1236702` sau đó. | Phạm vi Wine + debugger; không chứng minh cửa sổ hiển thị cụ thể có cùng nội dung ở breakpoint; không đo thành công login. |
| debug-variant | Copy R2 SHA `60e981698ed8c4adb3ca070a9612212d7cefa14d29453eb895513367cf197ce7`; hai u32 tại VA `1A3398C`/`1A33990` từ `1/1958` thành `0/0`; prefix copy sau run cũ | consumer EAX=1, getter error=0, đến `1232385`; watchpoint dừng sau ghi `9/0x9a1` tại `1254CF6/1254CFF`, backtrace `1258286→12323EB`. CALauncher được Wine khởi chạy, CALog có `EndOfInit! exitcode=9`; sau đó exception UD2 `226459C`. | Một lần debug; prefix copy không phải fresh control giống hệt, debugger và launcher có thể ảnh hưởng. Không tới breakpoint `1232468`; không chứng minh xử lý WCP kế tiếp hay login. |

File `patch-experiment.json` ghi byte trước/sau, offset/RVA, SHA, rollback dùng R2 nguyên vẹn và
nguy cơ che lỗi thật. Đã kiểm hai file cùng size, chỉ ba byte khác trong hai u32 nói trên. Không
đưa patch vào builder/harness đã merge. Không xóa lỗi live hay sửa binary gốc. Debug3 dùng cú pháp
watch thiếu type nên thất bại; debug4 dùng typed watchpoint. Plain-control bị ngắt, thiếu `run.json`,
không dùng làm đối chứng. Breakpoints/watchpoints không hit chỉ là quan sát trong phạm vi run/điều
kiện debugger, không chứng minh không có ghi bởi thread khác.

## Đối chiếu tĩnh

`modO` SHA `a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5` là memory-layout
base `0x400000`. Hai u32 đã là `1/1958` trong modO, kr-full và file R2; builder tạo image từ memory
snapshot. `1254D50`/`1254D60` đọc hai ô đó; `1254CC0` ghi chúng. Với variant, nhánh
`1232385→12323BA→1258150` khớp backtrace `12323EB`, sau đó `1258150` ghi `9/0x9a1` sau call
`16EA1D0`. Quan sát start CALauncher không chứng minh kết quả hay thành công của launcher.
`12349E0` gọi `1238530`; code có các nhánh trả0 cho hai kết quả, nhưng chưa gọi được trong các run này.
Không gán nguyên nhân MW_SEP(10) hoặc UD2 khi chưa đo. REA/IDA decompiler trả lỗi
`unreadable_output`; assembly là Capstone đọc modO, report lỗi lưu trong local.
