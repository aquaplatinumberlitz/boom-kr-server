# CP054 — C thay A trước trận đầu

Baseline CP053/[PR67](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/67) merge
`75606eb0353e7ef51d6a41e6bf6ac56b6cb56ad7`; KRE-017/[issue68](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/68).
Codex solo/tự kiểm, review [waived](../checkpoints/review-waiver.json). [Contract](../../PROTOCOL.md#prestart-replacement),
[câu hỏi](../../RESEARCH.md#gap-disconnect-replacement), [thứ tự duy nhất](../../PLAN.md#current-order).
Raw `local/research/codex/prestart-replacement-r1/`; snapshot trước merge trên `kre/prestart-replacement`. Trạng thái tích hợp ở [PR69](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/69).

`reuse-question.json` pin hash/source/locator/proof: CP053 cleanup, CP040/044/051 C3/BStart/current
roster/170/result, native framer/getter/153; VN lobby/session method, boomkr-re, boom-rs, l10n/archive.
Dùng lại fixture41/Cgame-dir/probes; không RE lại wire/login/consumer hoặc port opcode/ID/layout VN.
Gap mới chỉ admission index0 và cùng-run B/C sau hostEOF trước trận đầu; C là identity3, không Aresume.

| Run / UTC01/11 | Kết quả | run.json SHA256 |
|---|---|---|
| prestart-denied /14:29:37,210s | Sau Bclear/chief, TCP ordinal9 của C bị replacement_not_ready;8accepted transports, C chưa ID3/round. | `95e45aa9c8c6808cff006148812551900ce4b90dac4832f0ffc91e246204f785` |
| prestart-fixed /14:35:51,360s |12connections/C3/JoinReady/B104, firstkey12345/BbombID1/statepeer/result/return. | `9b56e4ec05c932ec09d8ddbe4ce8801c52999189d94abe89616392d26db99ca0` |

Control chỉ thêm admission rejection logging, không đổi predicate; fixed giữ logging và mở điều kiện
idle index0 sau cleanup đã gửi/mappedB/hostB/Aclosed/no pending; post-round refresh path giữ.
`departures_sent` là marker service đã send xong, không native ACK. Lý do từ chối không assign role
hoặc gửi bootstrap. Code boundary/firstkey/roster/deny transport có3test mới;45server tests tổng.
`baseline/server/` service SHA `a498c2e6018adf6aabaee4d13639e7f3b05669d10d92d40496c3305a2b7eadac`,
`fixed-source/` SHA `6b10bf9ee788c243e47f71a666b458870d48155a3755a6bcb88b61359e3191fe` giữ exact code.

Runtime A/B/C copy SHA `e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef`;
static `660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3`, modO raw/base400000
`a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5`, RVA=VA−400000.
Không sửa binary/pack/DB nguồn hoặc thêm patch/force state. Controller tuần tự có lease trong
namespace user/net/mount/PID chỉlo/no route/files-only NSS;3prefix/display/game-dir/wineserver.
Shared game-dir/cache không là ma trận cold/warm. Direct endpoints từ actual99, không quảng bá proxy.

Fixed: AEOF lúc Bown106/chưa map; Bmap→107clearA/chief1 rồi mới bootC. Native C ID3/slot0/ownmatch,
actual82/95 và B104;121full129/errors0/clock mỗi phía. Chỉ một generation1,153key12345 cho B/C,
không vòng A/B/bom/forfeit cũ. Actual B118→globalID1/159/158 chỉ B/C, B17/19→C170/C21; secure
actorB state1 trên B/C trước175.175outer302/inner292/errors0,180full3/errors0 rồi phòng, Bchief1/C0.
Một B/C playedround, không hai trận đầy đủ; rank/fuse/roster/auth vẫn fixture/policy.

`verification.json`:2528 quan hệ/hash;144native packet cases,6native153cases,210secure snapshots
của hai run. Native14EF300/typed/blob/vectorE8A340/secure52A350 giữ code; heap/receiver là model;
getter oracle counters chỉ trong emulation như CP051, không đổi counter runtime. Replay control
không có Cacceptedsocket, ghi rõ không native auth replay C. Adapter153 cũ giả mọi channel có
listdynamic; A chưa có153 gây KeyError, sửa missinglist→empty/rerun6cases, không sửa raw để giả pass.
Bốn ảnh `visual.png` đã đọc/hash tại visual-index; seal raw `<run>-hashes.json`.45server+7catalog+
18checkpoint tests cùng docs/catalog/artifacts/history/localhash/diff. Không tự xác nhận semantics/gate.

```bash
python3 local/research/codex/prestart-replacement-r1/run_probe.py new-label CA-r2-stage-social.exe 360 wine_probe_prestart_replacement.py local/research/codex/prestart-replacement-r1/service.gdb
python3 local/research/codex/prestart-replacement-r1/seal_run.py new-label
```

Exact argv/hash cạnh raw, label/prefix mới/controller tuần tự. Input retry hữu hạn có log; late45s/60s
chỉ chạy nếu thiếu118, không giả nguyên nhân tutorial/focus/overhead. Chưa account reconnect/general
membership/fullvalidator/cache/codec/reset/Windows vanilla/K5/K6a/LAN/Internet. AUD016/nextCP056 giữ.
Tiếp ngay validator124 nguồn field t theo PLAN và [chỉ đạo chủ dự án](../owner-directives/continuous-work.json).
