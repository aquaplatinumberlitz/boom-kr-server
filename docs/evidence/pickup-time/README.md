# CP055 — Nguồn t124 và giới hạn validator

Baseline CP054/PR69 merge `69df506cfd32c2898abb53f83d4b1e14a4c287f0`;
KRE-018/[issue70](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/70), Codex solo,
review [waived](../checkpoints/review-waiver.json), tự kiểm/chờ PR.
[Contract](../../PROTOCOL.md#pickup-tick), [gap/phép thử](../../RESEARCH.md#gap-floor-pickup),
[thứ tự duy nhất](../../PLAN.md#current-order). Raw `local/research/codex/pickup-time-r1/`.

`reuse-question.json` pin nguồn/hash/locator: CP030/049/actual124/cachedE2E390, VN `oc_e-game.md`
444–451 producer6EFBB8/6EFBCA/pendingtable21, boomkr-re data, boom-rs reference, l10n/archive.
VN cung cấp cách lần producer→record→writer, không xác nhận đơn vị/layout KR. Tránh RE lại login,
124/165/consumer/stat/k guard; `12580B0` tái dùng bootstrap cache rồi nối vào chuỗi mới.

| Run / UTC01/11/2026 /300s | actual A124 (t,k) vòng1→2 | run.json SHA256 |
|---|---|---|
| clock-trace /21:18:33 |(2215,12345)→(4365,12346)|`21be3904669210a59db265802a13e19689657b2e8874850bdce1176ca4bb5e07`|
| clock-late /21:24:43 |(2558,12345)→(4698,12346)|`00a77cca0042e1dab3206a798f65a5d5f485e73e5047a33cfeca185784393882`|

Controller tuần tự có lease; namespace user/net/mount/PID, chỉlo/no route/files-only NSS.
Cùng hai copy A/B/profile41/direct99 endpoints; không UDP replay, không thêm patch/force state.
Run2 đổi input đầu4→12s và đọc pointer timebase; debugger overhead không chứng minh time unit.
Cả hai run A/B đều2Start/clock/rank/return, không probe/service error. B clearA ở teardown controller,
không dùng làm gameplay disconnect mới. Shared game-dir/cache không chứng minh cold/warm matrix.
`runtime-source/` giữ exact service CP054 của cả hai run, **trước** logging-only `client_tick`.
`verify.py` bench bốn actual124 với code mới/broadcast sockets stubbed; không gọi đó native runtime.

Runtime SHA `e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef`;
static `660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3`;
modO **raw memory**, base400000 `a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5`;
RVA=VA−400000, không PE-remap raw. Original input kr-full.exe `cf72da112308d9f1ca256df8089594c5f787875310933073bf6bd5ba6f70e63b`,
DB `ab960c78a366845a117d1f3abfbe6e9e0e92fbf3a3ce14fb103794e02a355e27`, copy riêng `ida/`.

REA closed/Ghidra chưa cấu hình; IDA MCP private open thất bại `ida_nexus` missing.
Không sửa config/package; batch IDAPython trên **private DB** thành công bằng
`/mnt/m2sata/ida-pro-9.4-linux/idat -A -L<absolute-log> -S<absolute-script> <private.i64>`.
JSON/asm/decompile ở `ida/` là shell/IDA evidence, không bịa REA Evidence ID.
`analysis-ledger.json` ghi các lượt bounded/edge reopen; E2C780 là candidate bị loại (cache khác20byte).
Chỉ main CGameMsgCache vf0..9 được attribution, secondary dump có vùng RTTI/class kế cận.
Các body-byte-comparisons pin bytes đối chiếu static/runtime copy; RTTI cùng runtimehash độc lập DB label.

`verification.json`:3108 quan hệ/hash,4 getter→constructor→record→writer→actualTCP;
45 native getter snapshots/82events;474 native packet cases;7native constructor cases;
17 native123DC20 arithmetic/wrap cases. Time source/protected store/callback trong tick-flow là model,
không full scheduler; getter checksum/rekey counters chỉ model/reset **trong emulation**.
14 setter samples là rekey,34 là update; không lẫn write cùng giá trị thành tick mới.
Parser snapshot `context` giữ tag trước đó: chỉ exact CP55_WRITER_RECORD events là writer;
constructor stack và vector heap khác pointer, join theo fields/t/k/index.

```bash
python3 local/research/codex/pickup-time-r1/run_probe.py new-label CA-r2-stage-social.exe 300 wine_probe_pickup_time.py local/research/codex/pickup-time-r1/clock.gdb
python3 local/research/codex/pickup-time-r1/seal_run.py new-label
python3 -m local.research.codex.pickup-time-r1.verify
```

Tên/prefix mới, một controller; run2 dùng wine_probe_pickup_time_late.py/clock-api.gdb.
Hash raw/recipe/argv nằm trong seals và checkpoint; không cần lặp run chỉ để kiểm logging.
Nguồn t đã giải trong scope này; physical timebase API/unit, broader lifetime và original-server policy
còn unknown. Không thêm threshold, không tự đóng validator/auth/cache/codec/reset/K5/K6a/LAN/Internet.
Tiếp actorstate/forged-current-key theo PLAN; audit kế CP056 (AUD016 throughCP052 giữ).
