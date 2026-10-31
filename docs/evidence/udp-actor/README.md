# UDP vòng cũ tới actor — CP046

Baseline CP045/[PR53](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/53) merge
`6fe17b0665f952964abe4360116b621f3ae56919`; nhánh `kre/udp-actor-apply`, task KRE-005b/issue14.
Codex solo/tự kiểm, independent review waived; không gate closure.
[Contract duy nhất](../../PROTOCOL.md#udp-generation), [phép thử tiếp](../../RESEARCH.md#q-023).
Raw/copy/ASM/helper/log/ảnh: `local/research/codex/udp-actor-r1/`.

Trước nhánh đã đối chiếu PROTOCOL/RESEARCH/CP045/runtime-summary/consumerprefix;
boomkr-re kr-data189 trỏ VN; boom-re oc_e-game475–486; boom-rs VN_LEGACY_SERVER_REFERENCE29;
l10n docs migration và archive KNOWLEDGE75. Cache CP025 sender13E14C0 có cùng optionalfield gate.
`reuse-question.json` giữ nguồn/hash/locator/build, prooflevel, reuse/KRdelta/gap/minimumprobe.
Dùng lại endpoint/CRC/proxy/two-round/securegetter; không tìm lại queue, không copy VN wire/offset.
Nguồn cũ chỉ chứng minh P2P/movement theo phạm vi riêng, chưa KR actorapply/generationguard.

Runtimecopy A/B SHA `e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef`;
staticcopy `660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3`;
modO raw/base400000 `a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5`.
PEcopies mapped, RVA=VA−400000. Không sửa sourcebinary/pack/DB, không thêm patch hoặc ép actor/gate.
Controller tuần tự/lease/user-net-mount-PIDnamespace, chỉlo/nooutbound/files-onlyNSS;
prefix/game-dir/display/wineserver A/B riêng. Đây là diagnostic copy kế thừa CP020, không vanillaWindows.

Đường đo:13E110A output/caller →125937E target14EF010/packetstream → outerpump1257B61/vB4,
actualtarget125DEF0/vt17927FC. Probe có round/sample/thread guard; trước/sau callback đọc wrapper/data
secure actor+28/+4C/+58; parser giải checksum độc lập rồi native52A350 đọc lại từng snapshot.
`actor-apply.json` ghép exactoldhash/queue/drain/game/output/packetstream/pump và securevalues;
`downstream-summary.json` giữ line/loghash/pairing/cursor/target; raw không sửa theo kết luận.

Static đọc theo cạnh runtime: consumer tail, alignedcaller1259344, E8E960 falsehelper, target14EF010;
backtrace mới dẫn tới outer1258937/pump1257AF7, runtimepump dẫn tới125DEF0. Boundedwindow không là
wholefunction oracle; hai start92F0/9346 giải sai alignment đã bị loại trước khi dùng kết luận.
`code-match.json`:8ranges/4artifacts, kể cả falsehelper13byte; byte equality không tự chứng minh execution.
Không mở wholeclient/wholehandler; targetvfF4 sâu hơn và field grammar còn chưa đo.

Các ca giữ riêng:

- downstream-stale: pre-callstack buffer/length bị đảo trong probe; đọc0xD/KR_STOP cả A/B, không replay.
  Hai Start nhưng chỉ một result/return; controllererrorNone không là probehealthy.
- downstream-fixed: sửa stackread, outputold13byte/packetstream khớp. Hai Start, một bom/result/return;
  input phím vòng2 có log nhưng không actual118, không gọi hai vòng hoàn chỉnh. Overhead là giả thuyết.
- stage-control: hai actualbom/17/19/result/return; nhánh130BC00 không hit. A probe còn đọc actor sau
  result/return và gặp freedpointer0xFF00418B/KR_STOP; giữ lỗi đo, không gọi client/protocolfailed.
- pump-stale: đặt probe đúng outerpump, 24nonemptygame messages/client từStart2; chỉ đọc actor có
  vtable1775B40, reset con trỏ quan sát khiStart, không sửa actor. Hai bom/haiStart; vòng2 có17 nhưng
  chưa19/result. Spacehold1s và inputvòng2 chờ12s (sau measurement), không chỉnh oldpacket/timingreplay.
- pump-control: cùng actor-pump.gdb, cùng input/harness trừ modecontrol không replay; hai bom/17/19/
  statepeer/result/return, không lỗiGDB.8nativecallback normalB trướcinput giữ x120/y159; không wholematchgate.

| Run / UTC31/10/2026 | run.json SHA256 |
|---|---|
| downstream-stale / 14:23:22 | `7802ccf595aad631808ab85b95fb7b5cbaa2be24867ea029cbdd700aed9e4c5b` |
| downstream-fixed / 14:26:52 | `537bf08de0ec70d1a30f1c3ed8571924dc15864def0e5ef015be9d627707de6c` |
| stage-control / 14:31:11 | `34aad11535f20aa7b4313121189cf4818ab3e0e1e6b3cc6842e47d2550c674a8` |
| pump-stale / 14:36:26 | `dc592fc22f0c9118d950c4c69983674d2a8a3f10eb1ab5bebfc1e746d39a4e8d` |
| pump-control / 14:41:15 | `61d5bddd745ee5546c14a830655b4baa1ea991d17044a81a202e4042681bcd76` |

Old datagram `10010100003b00015cc8009f000000438f`, SHA
`2bee9f8cf7d5f12d3cbec10921e8d1c4e66ac1a10c1bf320b0b772417d9f0f9d`, originB9556 →
advertisedproxy41460 → A9648; source/bytes/CRC không đổi. Replay một lần2s sau153 cả hai vòng2,
trước input mới; không normalforwardR2 nào trùng wirehash. Round2/sample6:
queue64F68C78 count26→45, drain19→0, game15→output13→packetstream→pump cursor0→13.
Actor641E9198/slot1 securefield trước/sau và currentpacket kế ở `actor-apply.json`; contract giữ ý nghĩa.
Return1/0 không nhãn accept/reject. Không suy nguyên nhân thiếu19 từ chỉ ca replay này.

Tái lập tuần tự với label/prefix mới; không đè sealedrun:

```bash
python3 local/research/codex/udp-actor-r1/run_probe.py new-stale CA-r2-stage-social.exe 260 wine_probe_udp_actor_stale.py local/research/codex/udp-actor-r1/actor-pump.gdb
python3 local/research/codex/udp-actor-r1/seal_run.py new-stale
python3 local/research/codex/udp-actor-r1/run_probe.py new-control CA-r2-stage-social.exe 260 wine_probe_udp_actor_control.py local/research/codex/udp-actor-r1/actor-pump.gdb
python3 local/research/codex/udp-actor-r1/seal_run.py new-control
```

Recipe argv/harnessSHA, per-run hashindex và helpers giữ trong rawroot; raw/hash ghi ngay sau đo.
Không originalserveroracle, fullgrammar/freshnessvalidator, productionrelay hoặc fullmatch/K6a/LAN/Internet.
AUD014/CP044 và chu kỳ audit giữ nguyên; audit định kỳ kếCP048.

Kiểm cuối: **16623** quan hệ cơ học/5sealedcontrollers,669native packet cases/2443secure snapshots;
434nativeUDPCRCrows/287unique,8ranges/4artifacts;12policytests/artifacts5/5 pass.
`visual-index.json`16ảnh đã đọc qua2composite: hai client/run trướcinputvòng1, sau probevòng2,
sau bombinputvòng2 và cuối run. Oldrun cuối còn bubble, control về phòng; ảnh không thay nativeproof
hoặc tự xác định nguyên nhân khác outcome. Hai lỗiGDB ở run trước không bị làm mất khỏi raw/check.

**Giới hạn oracle getter:** batch đầu tới snapshot2353 gọi allocator77C15DE0 ngoài mô hình.
Giả thuyết counter5000 bị bác bỏ khi reset19D88D4 vẫn lặp lỗi cùng snapshot. Tail native52A44D–52A4D1
cho thấy counter1A7E438 mỗi55read gọi52A860/rekey; oracle cuối đặt19D88D4=0 lúc bắt đầu (<5000read)
và1A7E438=0 trước mỗi getter để chỉ kiểm read/value/checksum/pointerbyte, bỏ periodic rekey/storage.
Không patch machinecode getter, sửa captureddata hoặc đổi bất kỳ counter/state của client runtime.
`measurement-notes.json` giữ lần lỗi/giả thuyết bị bác bỏ/correction; không gọi đây là fullnative
setter/allocator/lifecycle coverage. Các run runtime và hash đều bất biến.

CP046 hoàn tất record/docs/tự kiểm; tích hợp qua PR nhánh, independent review waived.
Không K5/K6a/fullmatch/LAN/Internet mới đóng; audit định kỳ kếCP048.
