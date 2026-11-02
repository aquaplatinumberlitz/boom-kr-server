# CP058 — Auth/profile string và đuôi đã đọc sạch

Baseline CP057/[PR75](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/75) merge `956f31b`;
KRE-021/[issue76](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/76), Codex solo/tự kiểm,
[review waived](../checkpoints/review-waiver.json); trạng thái tích hợp ở [PR77](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/77). [Contract](../../PROTOCOL.md#auth-profile),
[chỉ mục56field](field-shape.json), [gap](../../RESEARCH.md#gap-auth-identity), [PLAN](../../PLAN.md#current-order).
Raw/privateDB/scripts/capture `local/research/codex/auth-profile-r1/`.

`reuse-question.json` pin source/hash/locator/proof: CP014/020 IDprefix/cached53reader candidates,
KR106 `put_str` đã native/runtime kiểm, CP057framer, VN `oc_d-login.md` perplayer consumer/31fields,
lobby cachechannel2/protocol; boomkr-re/boom-rs reference/l10n/archive. Dùng lại cách theo setter/getter,
không chuyển31fields/layout/opcode VN; tránh RE lại bootstrap/login trigger/codec/player IDs. Cache
CP0191090row/CRC là consumer khác, không gộp authprofile với game-dir assets/cache.

| Run / UTC02/11/2026 /180s | Native auth callback A/B | run.json SHA256 |
|---|---|---|
| auth-short /11:24:58 |34/34/errors54=1; lỗi đầu string tại29; ID1/2 đã được set trước. |`07dc2fe1a834db57aa72b64f21651cbc9b0977a31fb49ad65ff0d32cface03d3`|
| auth-full /11:28:27 |125/125/errors54=1;53direct fields sạch, thiếu helper tails. |`9c8261dcfec1c2d88d7dc08c08c65cc56928955da2b6b928b969c563c1e90274`|
| auth-tail /11:32:31 |133/133/errors54=1;2listcount sạch, thiếu53E230 count. |`c07df71a7ce1148d1e38142e9ac2cc3c8b711791ae4b3bd3ef496768d74d843f`|
| auth-final /11:36:07 |137/137/errors54=0/errors58=0;53direct+3count rỗng. |`d77056145360bc162508d44c87b8e3a6bd6d138ebcc31cd859357947f00a6780`|

Controller tuần tự/lease, namespace user/net/mount/PID chỉlo/no route/files-only NSS. A/B prefix/display/
wineserver/game-dir riêng; shared game-dir đã dùng các run trước nên **không cold/warm cache matrix**.
`cache-before-final/` giữ10file/channel2/multisvr/dynmap-dynimg snapshot/hash, chỉ filepresence,
không proof consumer đọc/ghi. Không thêm patch hoặc forceID/state/packetfunction calls trong runtime;
preexistingdiagnosticcopy/harness globalreset theo recipe cũ, không Windows vanilla.
Runtime SHA `e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef`;
static `660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3`, modO **raw/base400000**
`a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5`; RVA=VA−400000, không PE-remap raw.
`runtime-source/` pin service/codec chính xác bốn run; binary/DB/pack nguồn không đổi.

REA/IDA MCP known thiếuida_nexus: tiếp batchIDAPython **privateDB CP055**, không sửa config hoặc
bịa REA EvidenceID. `analysis-ledger.json`:4body pass1, concreteexiterr/forwardedge mở2body pass2;
`body-comparisons.json` sáu vùng code khớp runtimecopy. `10FD540`, string `14EB9E0→14EBA90` và
helper`12F6A60→1386790`, `53E230`; không quét toàn client.

Control chỉ ra oldprefix dùng typedu32zero tại stringname, thành u16length33A3hex; native đặt error54,
cursor29 không tiến, u8head kế=204. `native-short.json` A/B/C độc lập xác nhận, ID proof CP020 vẫn đúng.
Fix dùng **KR106 u16string encoding đã đo**; prefix đúng qua name/head32+namelength. Thiếu helper
counts được mở theo **nhánh runtime đang chạy**, không nối thêm randompadding để làm pass.
`native-final.json`:3candidate roles/168typedfield+string checks/cursor/errors0, C native-only.
Unknown values0/emptylists/IDs/names là design, không serverdefaults hoặc toàn successschema.

`auth-reads.json` pin locator/sourcehash. Short/full trace chỉ directcaller10FD..10FED; tail/final mở
all readers cùng packet nên thấy3nestedbool→u8 calls. Final59entries=56logicalfields; counthelper ở
125/129/133, callbacks137/errors0 cả A/B. **Actual native C→S6**81byte mỗi role chứa KRLocal/KRGuest;
framer/CRC và string reader độc lập tại offset2 kiểm tên, không phải fullopcode6schema/UI/profileproof.

`verification.json`:3573relations/hash,2085nativeframer valid/CRCbad/partial cases trên actual4runs,
168finalfield nativechecks, actualname6A/B; no controller/service/probe errors. Short/full/final có2Start/
clock nhưng1rank/return; tail chỉ1Start/clock,0rank/return. Không ghép run thành trận nghiệm thu hoặc
quy mọi input/timing khác cho authfixture. Header/string/tail khác qua các run là phép sửa recipe liên tiếp,
không one-variable causality cho toàn gameplay.

Exporter41fixtures chuyển3authalias sang `auth-{A,B,C}-final.bin`/seq43; `export-check.json` và
FixtureProfile kiểm cả41byteframes trùng exact profile-final,38alias khác giữ nguyên. ProfileJSON cũ
không migrate. A/B runtime, C135byte/tên5byte chỉ native; cần chạy C trước acceptance membershipprofile mới.
Không đổi server/codec; baseline50server+18checkpoint regressions CP056 được tái dùng, không gọi test mới.
Catalog/docs/artifacts/localhistory/diff kiểm theo workflow; AUD018 serious-contradiction ngay lỗi string,
periodic vẫnCP056, nextCP060.

Các module build_full/native_short/native_full/native_final/native_names/parse_auth/verify chạy bằng
`python3 -m local.research.codex.auth-profile-r1.<module>` từ repo root. Run mới dùng run_probe.py,
label/prefix mới,180s, wine_probe_auth_final.py/auth-tail.gdb; seal_run.py sau completion. Lỗi adapter
ban đầu metadata authseq=None đã đối chiếu measuredS4seq43, không đổi raw; nativeverify chạy trực tiếp
thiếu import path được chạy lại modulemode. Không fake các lỗi adapter thành native/client failure.

Tiếp named-cache present/missing trên copy riêng theo PLAN; cache/coldwarm/reconnect/token_2/field
meaning/featurecount2/fullauth/vanilla/K2/K3/K5/K6a/LAN/Internet vẫn mở. Không dừng ở checkpoint.
