# CP066 — Actual tranh item trên distinctspawn qua hai vòng

Baseline CP065/PR91 merge `fbdf5f32bd44ddc347c21fad362a7532277682ef`;
KRE-029/[issue92](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/92).
Codex solo/tự kiểm; [review waived](../checkpoints/review-waiver.json), tích hợp chờ PR.
[Contract/giới hạn](../../PROTOCOL.md#distinct-spawn-race), [gap](../../RESEARCH.md#gap-floor-pickup),
[thứ tự](../../PLAN.md#current-order). Raw/script/capture ở `local/research/codex/distinct-spawn-race-r1/`.

`reuse-question.json` pin CP030 midpoint/Down-Up1s, CP062 serializedclaim/deliveryhold/nativeeffect,
CP065 currentprofile/spawn/reset, KR/VNdocs/itemfamilies/boom-rs/l10n/archive. Dùng lại method+582layout,
không opcode/offset/ID/time/authority VN. Không RE thêm nativebody/clock/auth/cache/itemconsumer hoặc
patch/client/serviceproduction. Thay một datafixture: floor582index17/col3/kind2 row4→6/catalognhưnhau,
spawn0/1 giữ nguyên. Native11ED590 kiểm16byteinner/counts1/0/0/record17,6,3,2; negativesCRC/partial.
Không phục hồi dropalgorithm bằng placement này. `prepare.py` mới không execute scriptCP030 vốn sửa raw cũ.

Runtimecopy SHA `e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef`;
static `660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3`;
modO raw/base400000 `a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5`,
RVA=VA−400000. Sourcebinary/pack/DB giữ; namespace user/net/mount/PID chỉlo/no route/NSSfiles-only,
một runtimelease/controller, prefix/display/wineserver riêng/warmcopyCP062/privategamesCP059.
Profile41 CP065 đổi aliasfloor/catalogrow,40alias còn lại giữ exactbytes. Runtime-source pin service.

Run `distinct-race` 2026-11-02T16:24:54Z, run.json SHA256
`f6456a6e76847ed0230973828c08a898eabe0d82a46dbf7390f0cd228531c040`.
8actualconnections/IDs1,2/slot0,1, twoStart/clock, nativeclockA120159/B120399 cảhai view/vòng.
NormalUI ADown/BUp1s qua barrier clock+12s theoCP030; actualTCP124 bắt thật, không mint/gọi writer.
`runtime_hold.py` reuseCP062 chỉ hoãn positive1652s, productionclaim/consume/key/reportguard giữ;
wrapper join exactrequestSHA/winner/receiver và generation/due/flush. Saupositive, BRight/Aspace hoàn
thành vòng. Mỗi vòng haiactual124 cùngindex17/row6col3/kind2/keyhiện hành trước165delivery.

|Vòng/key|Winner / loser|Hold / loser tới trước delivery|NativeRoller cảhai view|
|---|---|---|---|
|1/12345|A / B|2.020268s / 1.801668s|{'0': 5, '1': 4}→{'0': 6, '1': 4}|
|2/12346|A / B|2.025158s / 1.461968s|{'0': 5, '1': 4}→{'0': 6, '1': 4}|

Mỗi vòng chỉ một165/receiver/slotwinner, callback12/errors0, loser không confirm/statdelta;
round2baselineA5/B4 và itemindex17 dùng lại được. Winnerkhôngforce/khôngđòiđảowinner để làm pass.
Saupickup có haiA118/globalID1→2/159158/17/19→B170/B21, nativeAstate1bothbefore175,
hai175302/1803/errors0/roomrefresh. Controller/service/probe errors0, normalstop sau refresh2+3s.

`verification.json`: 1517 relations/hash/seals/source, 777 nativeactualframer,
243 packet/12 native153/408 secure snapshots. Heap/receiver/gettercounter models chỉ emulation,
không runtimeforcing. Productioncode không đổi:57server/18checkpoint baselineCP063/065reuse;
7catalog/docs/artifacts5/5/localhistory/diff kiểm bàn giao. AUD020/CP064,nextCP068.

Reproduce label/prefix mới qua run_probe.py CA-r2-stage-social.exe360/wine_probe_distinct_spawn_race.py/
race.gdb; prepareknownfloor/profile/hash trước, seal saucompletion, derive/native/verify modulemode
repo root. Không overwrite raw/chạy chồng. Hold2s là measurementperturbation, không policy latency/
arbitration/fairness gốc; UIbarrier không exactphysics simultaneity. Queue discard khácgeneration chưa
runtime riêng, actualTCP replay admission/actorauth/distance/fullreset/K6a/LAN/Internet còn mở.

Same-run distinctspawn/tranhitem/two-cycle là kết quả trong recipe, không ghép gate từ hai rank.
Tiếp C2S118 freshness/duplicate trước cấpID/timer theoPLAN, đối chiếu writer/capture/control;
không dùng oldS2C159/158 hoặc giá trị firstfield bằngk124 thay proofrequest admission/authentication.
