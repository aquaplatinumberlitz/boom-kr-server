# CP056 — Current-key forgery sau report nhốt/chết

Baseline CP055/[PR71](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/71) merge
`76766e78c840da02f068ba2e3607536cf2fb00f4`; KRE-019/[issue72](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/72).
Codex solo/tự kiểm, [review waived](../checkpoints/review-waiver.json), chờ PR.
[Contract/giới hạn](../../PROTOCOL.md#pickup-report-state), [gap](../../RESEARCH.md#gap-floor-pickup),
[thứ tự duy nhất](../../PLAN.md#current-order). Raw `local/research/codex/pickup-state-r1/`.

`reuse-question.json` pin nguồn/hash/locator/proof: KR030/029/037/049/055/actual124/127/170,
cachedE2C450/1243330/B52840; VN oc_e-game§5a-d/items; boomkr-re data/kr; boom-rs VN reference,
l10n migration/archive. Reuse native framer/getters/stat/clock/fixture/isolation, không RE lại login,
124/165/kim/itemkind hoặc mang opcode/layout VN. Khoảng trống mới là eligibility và forgedcurrentkey.

| Run / UTC01/11/2026 /300s | Kết quả | run.json SHA256 |
|---|---|---|
| state-control | GDB generation lỗi disable concatenation, SIGINT cleanup; không eligibility/control proof. | seal riêng, không dùng làm control |
| state-control2 /21:56:36 |2actual124+4semanticforgeries nhận/165; sau17/state56 Roller6→7 cả hai view, sau19 tại max7 không delta.2cycles. |`6565406b8337ea9f1668e30001ecb4f9caeac8517c2c02c3a84c3dbecb0a124f`|
| state-guard /22:02:07 |4forgeries bị loại/no165; round2actual124/key12346→165/5→6 cả A/B;2cycles. |`3e4313e8481ab80a8cdacdd8e7bad67be8880d97d6f4df3f694c0c5f0be384a7`|

Hai run hoàn chỉnh cùng profile41+spare18/19, direct99/noUDP replay, input/bom path cũ. Prefix/display/
wineserver/game-dir A/B riêng, namespace user/net/mount/PID, chỉlo/no route/files-only NSS/runtime lease,
controller tuần tự. Copy SHA `e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef`;
static `660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3`;
modO **raw**/base400000 `a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5`;
RVA=VA−400000, không PE-remap raw. Source binaries/DB/packs nguyên vẹn, privateIDA DB từ CP055.
`control-source/`, `fixed-source/` khớp policy_sources từng run; không đổi source trong run.

Static fallback: known IDA MCP thiếu ida_nexus, dùng batchIDAPython privateDB, không sửa config;
không bịa REA EvidenceID. `analysis-ledger.json`:4body pass1/itemguards,4body pass2/collisioncallers;
edge mainvf108/B40F30 mở thêm sau vtable/runtime collision. +618 không tự gán state; `body-comparisons`
kiểm8body runtimecopy, bổ sung vf108 có hash riêng. Không mở hết các producer class/mode.

Semanticforgery lấy actualCP055body/carrier, **đổi targetindex/hàng/cột và k hiện tại**, giữ t2215;
không gọi old unchanged packet hoặc nativewriter request. Trigger actual127/17 hoặc19 từ cùng actor;
production dispatcher bypass C2S TCP, S→C165 đi live native. Fixed round1 không actual124, không gán
nguyên nhân cho guard; round2actual124 là proof reset/living path. Cả hai role cả hai run đều2Start/
clock/175/180, stateA1 trước175, không probe/service error; không fullmatch/K5/K6a.

`runtime_dispatch-v1.py` là exact source đã đo: outer127 reentrancy ghi đè provenance label injection
thành actual_tcp_client. **Raw không sửa**; `verify.py` nối exact measurement.injections SHA/fields/
trigger, phân biệt2actual+4injected control và1actual(round2)+4injected fixed. V2 `setdefault` chỉ
sửa metadata ở localtool, chưa runtime. `measurement-notes.json` ghi lỗi GDB/seqadapter/provenance;
không bỏ lượt lỗi để tạo lịch sử sạch. Bench đầu dùng seqCP056 cho carrierCP055 bịCRC reject,
sau đối chiếu originalseq/native source đã sửa adapter, không sửa native/raw.

`verification.json`:3854 quan hệ/hash,483native packet cases,1004secure snapshots/native52A350;
receiver/heap là model, getter periodic counters chỉ reset **trong emulation** như CP049, không runtime.
50server tests (5mới), actual-capture dispatcher bench: report17/19→forged124 controlaccept/fixedreject,
peer vẫnclaim được item bị A từ chối và actual124 vòng2 qua Start. Policy là design theo report,
không native authority/auth hoặc full parser/transport. Fixture Roller chưa rescue/selfescape;
không áp marker đến Start sang kim/cứu mà chưa nối1/18. Dead165 ở control đang max7 nên không proof tăng.

```bash
python3 local/research/codex/pickup-state-r1/run_probe.py new-label CA-r2-stage-social.exe 300 wine_probe_pickup_state.py local/research/codex/pickup-state-r1/state.gdb
python3 local/research/codex/pickup-state-r1/seal_run.py new-label
python3 local/research/codex/pickup-state-r1/verify.py
```

Tên/prefix mới/controller tuần tự; recipe/argv/seals cạnh raw. Runtime đã đo dùng wrapperv1 archived,
wrapperv2 cho lần sau; không giả v2 là source của hai run. AUD017/CP056 đọc docs→reports/raw và sửa
stale summary/hostsetup/nguồnt/nextstep; audit kếCP060. Tiếp native codec boundaries theo PLAN.
