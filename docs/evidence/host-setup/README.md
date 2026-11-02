# CP053 — Host rời khi survivor còn setup

Snapshot CP053: blocker C trước vòng đầu đã đo/fix ở [CP054](../../PROTOCOL.md#prestart-replacement); thứ tự hiện hành ở PLAN.

Baseline CP052/PR65 `3617e69a3536cac9ac9f8a05ae826cbb0cc803be`; KRE-016/[issue66](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/66).
Codex solo/tự kiểm, review [waived](../checkpoints/review-waiver.json). [Contract](../../PROTOCOL.md#host-setup),
[câu hỏi](../../RESEARCH.md#gap-disconnect-replacement), [thứ tự](../../PLAN.md#current-order).
Raw `local/research/codex/host-setup-r1/`; snapshot trước merge trên `kre/host-setup`. Trạng thái tích hợp ở [PR67](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/67).

`reuse-question.json` pin source/hash/build và scope: CP052 host-setup-gap, CP034/050/051107/110/
IDclear/chief, profile41/probe; boomkr-re, VN lobby/session, boom-rs reference, l10n/archive. Tái dùng
consumer/fixtures/method membership cleanup, không port wire VN hoặc mở thân hàm binary mới.
Cần đo native consequence của host EOF lúc Bown106/chưa map, rồi cleanup sau initial setup.
[Yêu cầu chủ dự án làm liên tục](../owner-directives/continuous-work.json) đã ghi vào AGENTS/PLAN,
commit `bb640f7`; giữ nguyên evidence/audit/PR và gate, không dừng ở checkpoint boundary.

| Run / UTC01/11 | Kết quả | run.json SHA256 |
|---|---|---|
| host-setup-baseline | Turn bị ngắt, chỉ tới A;96rawfiles giữ trong interruption.json. Không finalrun/control/protocol failure. | Không có finalrun |
| host-setup-baseline-r2 /14:08:11 | AEOF lúc Bsetup, B giữ A/chief0; bấm nút B gửi95/110Ready, không107. | `d97f812b7af945c2f9b33708a2911990e059881fd4899bec4edbd75380fa91c6` |
| host-setup-fixed /14:15:57 | Host logicalB, pending B; Bmap xong107/110, nativeID1→0/chief0→1. | `611923168ca47d227b11bf3da408b6d9765e9f80dccc919d76aca1f303e80425` |

Cùng recipe145s/profile41, runtime A/B SHA `e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef`;
static `660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3`, modO raw/base400000
`a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5`, RVA=VA−400000.
Source binaries/DB/pack nguyên vẹn, không thêm patch/force state. Namespace user/net/mount/PID chỉlo,
no route/files-only NSS/runtime lease, controller tuần tự; prefix/display/game-dir/wineserver A/B riêng.
Shared game-dir/cache giữa recipe không là cold/warm matrix. `baseline/server/` SHAservice
`1bcd3e8892fd2a5149aa62ef59384ecbb337653556753019dfe2d302eb988ce6`, `fixed-source/` SHAservice
`26eb81a733e4f2f55047d0f28a53074be9a18cd53808a28764b52212924994a9` giữ exact code.

A đã map; Bactual82/own106 rồi kill prefixA trước Bslot/map. Fixed chọn chủ từ accepted member,
queue departure riêng cho B; sau raw slot61/guest64/table67/map70 mới107/seq73→110/seq76. Native107
base6/errors0, rewind0/final1/6,12400D0/12401A9 actualID1→0; chiefgetter1240D5E0→1. Ảnh B hết ôA,
Start thay Ready. Không native121/clock/175/180 hoặc phase197/196/153, không gửi A đã đóng.
UI input không là proof actual104 hoặc gameplay. Đóng survivor trước flush/lobby-only/duplicate/order
là3regression mới, chưa runtime riêng; tổng42server tests. Deferred cleanup là emulator policy, không
server gốc/general membership. Recipient sendfailure/partialflush và mọi host-election variant chưa đo.

`verification.json`:932quan hệ/hash; `native-packets.json`:18valid/CRCbad/partial cases trên actual82/
95 và107/110. Native14EF300/typed/blob giữ code, receiver/heap là model; không automatic semantic/gate
proof. Hai run full được seal bằng `*-hashes.json`; lượt interrupted giữ riêng, không giả complete.
Bốn ảnh trong `visual.png` đã đọc, hashes tại visual-index. 7catalog/18checkpoint tests cùng docs/
artifacts/history/localhash/diff kiểm cạnh raw. AUD016/CP052 giữ, audit kếCP056.

Blocker tiếp: admission C đòi round.refresh dù chưa có round; B đã clear/chief chưa đủ chứng minh
có C hoặc Start. Theo PLAN tiếp C trước trận đầu, rồi actual bom/peerstate/result/return. Auth/cache/
codec/fullvalidator/reset/general membership/K5/K6a/LAN/Internet giữ mở.

```bash
python3 local/research/codex/host-setup-r1/run_probe.py new-label CA-r2-stage-social.exe 145 wine_probe_host_setup.py local/research/codex/host-setup-r1/host-setup.gdb
python3 local/research/codex/host-setup-r1/seal_run.py new-label
```

Exact argv `<run>-recipe.json`, label/prefix mới, không đè raw đã seal hoặc chạy controller chồng nhau.
