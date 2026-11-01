# CP052 — Guest EOF trước map và slot ma trong setup

Baseline CP051/[PR63](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/63) merge
`c764aa555d0318322a4f67ef0628e14d3e95d0c9`. KRE-015/[issue64](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/64),
phụ thuộc KRE-005b/#14 và CP034/050/051. Codex solo/tự kiểm, independent review
[waived](../checkpoints/review-waiver.json). [Contract](../../PROTOCOL.md#setup-eof),
[câu hỏi/phép đo tiếp](../../RESEARCH.md#gap-disconnect-replacement), [thứ tự duy nhất](../../PLAN.md#current-order).
Raw `local/research/codex/setup-eof-r1/`; snapshot khi ghi CP052: nhánh `kre/setup-eof`, chưa merge.

## Nguồn và khoảng trống cần đo

`reuse-question.json` pin path/hash/build/locator/prooflevel: current contract/evidence CP034/050/051,
source caller và cache/probe107/110/106; boomkr-re kr-data, boom-re lobby/session.rs (teardown line428),
boom-rs VN reference, l10n router và archive KNOWLEDGE. Dùng trực tiếp KR wire/IDclear/chief reader,
fixture41 và recipe Wine/lease; VN chỉ giúp nhận ra membership teardown khác gameplay stage.
Không chuyển opcode/offset/ID hoặc luật host của VN; không mở thân hàm binary mới/RE lại receiver.
Câu hỏi còn thiếu: guest đã Join và own106 nhưng chưa map, host có giữ slot ma hoặc nhận phase giả?

`baseline-bench.json` chạy code cũ bằng socketpair/fake send thấy không cleanup; đây là kiểm policy,
không native KR proof. `baseline/server/` và `fixed-source/` lưu exact bytes dùng hai run. Bản sửa chỉ
đổi inactive close từ marker map sang Join/create acceptance; schema/consumer giữ nguyên.

| Artifact | SHA256 |
|---|---|
| Runtime A/B copy | `e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef` |
| Static copy | `660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3` |
| modO raw/base400000 | `a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5` |
| Service control | `3dbaf35dd44211a9c3f43725e48fccb2ba89a614dcf4d84b87e5d7f1eda925eb` |
| Service fixed | `1bcd3e8892fd2a5149aa62ef59384ecbb337653556753019dfe2d302eb988ce6` |

RVA=VA−400000, không PE-remap modO. Source binaries/DB/pack không sửa; không patch/ép actor hoặc
phát request giả. Hai controller tuần tự, namespace user/net/mount/PID chỉlo/no route/files-only NSS,
lease bên trong. Prefix/display/wineserver/game-dir A/B riêng, game-dir/cache chung giữa hai recipe
không phải ma trận cold/warm cache. GDB `setup.gdb` bỏ autoenable UDP đã phát hiện ở CP051;
không có gameplay trong ca này, không suy tác dụng tới input/UDP từ việc ít probe.

## Runtime control/fix

| Run / UTC01/11/2026 | Kết quả | run.json SHA256 |
|---|---|---|
| setup-baseline /13:41:06 | B actual82/own106→EOF trước map, A vẫn thấy B, không107/clear. | `519d505485522323696f278ff84c66910035c51e2f4fa6d85d717d5b1d64279c` |
| setup-fixed /13:48:18 | Cùng boundary, A nhận107/110, native ID2→0/chief1, ô B biến mất. | `9350439da1121019ac49b5d2415868d802dd43317d9bdfba3dc5470ce689cb10` |

Mỗi recipe145s/8connection/IDs1,2/slots0,1/profile41. A đã map; controller kill prefixB sau
`player_response_sent` nhưng trước `slot_response_sent`/map; channel thực EOF. Baseline có native
106 guest/ownmatch0 trên A và ảnh sauEOF còn B; không REMOVE107/SLOT_CLEAR. Fixed phát đúng một
107(slot1,id2,0)/seq73 rồi110/seq76 chỉ A. D603A0 base6/6/errors0, DC6329 rewind0/final1/6 intentional;
12400D0/12401A9 so đúng actualID2 rồi xóa0. Chief getter1240D5E giữ1, ảnh chỉ A và Start.

Không có Start121/clock hoặc rank175/return180 native ở hai run, không197/196/153 hay bom/trận.
UI attempt Start có log input, không actual104 hoặc chứng minh native Start được thực hiện.
Không setup packet tới B sau close; duplicate/tick/lobby-only/before-own106 được kiểm bằng code
regression, **chưa runtime riêng**. Cleanup/host state là policy được client chấp nhận, không luật
server gốc. Không dùng ca không có trận làm gameplay/fullmatch/gate proof.

## Kiểm, audit và giới hạn

`verification.json`:931 quan hệ/hash,0error/0probeerror; `native-packets.json`:12cases từ actual82 và
S2C107/110 (valid/CRCbad/partial). Native14EF300/typed/CRC giữ code, receiver/heap là model;
replay role A/B giữ actual auth/channel framing, không oracle thế giới. `*-hashes.json` pin raw,
`visual-index.json` pin4ảnh đọc trong `setup-visual.png`. 39server tests gồm3case mới,7catalog và
18checkpoint tests; docs/catalog/artifacts/history/localhash/diff checks lưu cạnh raw. Checker chỉ
kiểm tính nhất quán đo được. [AUD016](../checkpoints/audits/AUD016.json) đọc core docs và so raw/
reports CP049–052; counter kếCP056, không đổi lịch checkpoint cũ hoặc gate.

`host-setup-gap.json`: code execution socketpair/fake sender cho hostEOF khi B chưa map, không
recipient107/110 và tick tiếp vẫn chọn alias guest_B. Đây là **design/confirmed về code**, chưa
quan sát KR runtime hoặc kết luận native B re-add A. Cần ca hostEOF đúng boundary và deferred
cleanup/current roster nếu được xác nhận. Fixture preannounce/general membership/account reconnect,
idleEOF/sendfailure/timeout/loadfail/fullvalidator/cache/codec/reset/Windows vanilla/LAN/Internet còn mở.

```bash
python3 local/research/codex/setup-eof-r1/run_probe.py new-label CA-r2-stage-social.exe 145 wine_probe_setup_eof.py local/research/codex/setup-eof-r1/setup.gdb
python3 local/research/codex/setup-eof-r1/seal_run.py new-label
```

Exact argv ở `<run>-recipe.json`; dùng label/prefix mới, không đè sealed run hoặc tranh controller.
