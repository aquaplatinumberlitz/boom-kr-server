# CP106 — Movement hai chiều qua hai vòng và ending cùng run

KRE-069/[issue172](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/172), baseline CP105/
[PR171](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/171)
`dc30e4ff49e325cb9488e4be7eb4befadd107f9b`. Codex tự kiểm, [review waived](../checkpoints/review-waiver.json),
CP106 chờ PR. [Contract/giới hạn](../../PROTOCOL.md#current-movement-cycles),
[câu hỏi](../../RESEARCH.md#gap-peer-movement), [thứ tự](../../PLAN.md#current-order).
AUD032 qua CP104; audit định kỳ kế CP108. Không nâng gate.

Raw/report/recipe: `local/research/codex/current-movement-cycles-r1/`; controller copy
`local/research/codex/wine_probe_current_movement_cycles.py`. `cycles-recipe.json` giữ exact argv.
Run `cycles`: 362,995406370s, SHA256 run.json
`e5c9a897c3c43a7efd480b3f3cd85b59837b8b5a95d619d0d23fab8d256dadce`.
`cycles-hashes.json`/`sealed-verification.json`: 2.186 raw files kiểm size/SHA; không sửa sau seal.

Source-first `reuse-question.json` 13 nguồn và `reuse-current-addendum.json` pin docs hiện hành:
CP098 API13/key/slot/B4E6F0/B4EA14/securegetter, CP103 fixed-role proxy/profile/route hai vòng,
CP104–105 ending phase; KR cũ `kr-data.md` chỉ rõ giới hạn đo TCP, VN `oc_e-game.md` §5a-6 dùng
khái niệm direct P2P và tách TCP/session. Build/VA/layout KR không lấy từ VN.
Không đọc thân hàm mới, không tìm lại codec/endpoint/clock/item/reset/endingqueue.

45 file pin trước launch, 30 server files khớp source CP103; profile cùng SHA
`30faa848cf5e5cf3e4e450a9ee38368cc9be8fb333f7bfeee944e5873da1966c`.
Nativecopy A/B SHA `e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef`;
staticcopy `660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3`;
modO raw/base400000 `a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5`;
RVA=VA−400000, không PE-remap raw. Binary/pack/DB nguồn không đổi.
Private user/net/mount/PID namespace, lo-only/no routes/files-only NSS, một lease/controller;
A/B prefix/display riêng. Chỉ bổ sung host-only ack sau sáu breakpoint đã bật và mẫu mới hai view
trước input; cùng pulse OS 0,25s, observation sau input tối đa3s. Không gamecall/statewrite/UDP rewrite.

`analyze_targets_strict.py`/`peer-verification.json`: 557 quan hệ, cả bốn pha có target đổi sau input,
unique API17→packet13→native key/slot/args→+7BC/+7C8 và currentXY tiến qua callback sau.
360 copied-native52A350 getters (gồm readiness wrappers), 44 actual-ingress nativeCRC checks tách
emulation khỏi runtime. `snap-branch-examples.json`: ba lần distance≥threshold có currentXY bằng args
sau call, gồm equality40/40; không đo callee riêng hoặc suy công thức/thời gian interpolation.

`runtime-verification.json`: 152 quan hệ, 14 actual118/159 entities và14 actual158/view, 60 native map
snapshots ×195 flags khớp prediction/process/box delta; reset seed hai vòng, không job chưa gửi.
11.820 copied native getters +11.700 array address checks là emulation riêng.
`transport-verification.json`:247 joins, 8 same/4 old TCP replay bị loại/no extraID/165/170;
`items-storage-verification.json`:39 checks, bốn pickup Bomb/Fire, stat1→2/baselineStart2 cảhai view;
`drained-verification.json`:60 quan hệ policy/actual21 sau17/latest158/local56→26;
`peer21-branches-verification.json`:33 quan hệ, lần này B gate1/56→26 cảhai vòng, không bê distribution
một-clear/một-skip của CP103 sang run mới; 12 copied-native getter checks.
`semantic-boundaries-verification.json`:59 quan hệ, cùng A3/B1 và XY ở native180 từng view/cảhai vòng;
XY của A ở vòng2 là80,169, không ép Y159. Entry175 vẫn khác ở một số field; không global-tick proof.
534 native-packet/537 typed-report model cases giữ riêng, không gọi là native gameplay callbacks.

Giữ `drained-check.txt`/`verify_drained.py`: analysis path replacement đã đổi nhầm key target thành
cycles và gây KeyError; bản riêng `verify_drained_current.py` chỉ sửa root/directory, vẫn kiểm actual
callee B43320/args/pre56/post26. `analysis-correction.json` ghi rõ; raw không sửa và không chạy lại
Wine để che lỗi tool. Preparation pin nhầm tên file terrain trước launch cũng giữ correction;
prelaunch cuối đã kiểm đúng terrain_model/profile và mọi hash.

Nguồn cũ tránh toàn bộ vòng RE codec/consumer/item/ending mới. CP098 R2 A-Down partial được khép ở
run này, không sửa history CP098 hoặc khẳng định observer timing là nguyên nhân của gap cũ.
#166 có qualification cùng run theo mốc native180, thay tiêu chí entry175 đã bị CP104–105 bác bỏ;
việc tích hợp chỉ qua PR. Full identity/auth/codec/validator/authority/general membership/all reset/
vanilla/fullmatch/K5/K6a/LAN/Internet còn mở. Bước kế theo PLAN: normal host Back còn guest live,
reuse107/110/refresh/currentseq; không lặp host EOF hoặc guest Back đã biết.
