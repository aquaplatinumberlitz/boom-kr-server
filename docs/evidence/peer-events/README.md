# Trap/death của actor phía peer — CP029

Codex solo/tự kiểm; review độc lập [waived](../checkpoints/review-waiver.json).
Baseline CP028/PR36 merge`79acacddbbaff2854dfbd74d947a9e58dab79e48`.
CP029 snapshot trước tích hợp; PR nhánh`kre/peer-events-runtime` giữ trạng thái merge hiện tại.
Contract duy nhất ở [PROTOCOL](../../PROTOCOL.md#peer-events), việc tiếp ở
[RESEARCH](../../RESEARCH.md#q-023), gate ở [PLAN](../../PLAN.md).
AUD008/CP028 gần nhất; audit định kỳ kếCP032. Không gate mới đóng.

## Artifact và recipe

Raw`local/research/codex/peer-events-r1/`, scripts probe cạnh bên dưới`local/research/codex/`.
Mỗi run user/net/mount/PID namespace chỉlo/no outbound/files-only NSS/localstub; controller giữ lease,
hai clientprefix/display/wineserver/game-dir riêng. Dùng Bcopy local CP028, không sửa sourcebinary/
pack/DB hoặc thêm patch. Các sửa import/API/AC/guard CP020 và memoryguardreset cũ vẫn kế thừa;
không vanilla hoặc proof authNexon thật.

| Artifact | SHA256 |
|---|---|
| RuntimeA/B CA-r2-stage-social.exe |`e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef`|
| Static CA-r2-login-logout.exe |`660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3`|
| modO.bin raw memory-layout/base400000 |`a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5`|

RVA=VA−400000; map modO raw, không PE-remap.10 vùngcode khớp static/runtime/native.

| Run / bắt đầu UTC30/10/2026 | Config | Kết quả/giới hạn |
|---|---|---|
|baseline /13:28:43|195s|Không118/trap/death sauSpace; probe dàyB33270/B41150, UI có tutorial. Mục tiêu chưa đạt.|
|control-lean /13:33:03|195s|BỏB33270 nhưngB41150 vẫnhitdày trong tick; không118. Mục tiêu chưa đạt.|
|control-targeted /13:37:41|215s|Hai vòngAdeath; trước175: ownAstate1/remoteA trênB56. Không170.|
|forward170 /13:42:37|215s|Hai vòngA→B170; remoteA0→56→1 trước175; cả hai175/180/phòng hai lần.|
|alternating /13:46:39|240s|A→B vòng1,B→A vòng2; state56→1 đúngslot. Chạm hạn trước175/180vòng2, chỉ mộtchu kỳkếtquả/phòng.|
|alternating-long /13:52:25|320s|A→B vòng1/B→A vòng2, remote56→1 trước175, hai175/180/phòng cảhai; cùng8connections.|

Runexit0/noerror chỉ là recorder hoàn tất. Hai run đầu không đạt bom/trap; alternating240s không có
175/180lần hai dù đã đo state đảo chiều. Không dùng tên recipe/exitcode thay callback/hash/state.
GDBteardown remoteclosed/XIO giữ raw, không tự coi clientcrash đã định vị.

Recipe argv/GDB/harness pin trong manifest, full rawfile/size/hash ở`<run>-hashes.json`.
Tái lập bằng runID/prefix mới; không ghi đè raw đãseal hoặc chạy song song controller tranhlease:

```bash
python3 local/research/codex/peer-events-r1/run_probe.py new-events CA-r2-stage-social.exe 320 wine_probe_peer_events_alternating_long.py local/research/codex/peer-events-r1/events-targeted.gdb
python3 local/research/codex/peer-events-r1/seal_run.py new-events
python3 local/research/codex/peer-events-r1/derive_roles.py new-events
```

## Đọc và kiểm bằng chứng

**12611 kiểm cơ học pass**, 6 run sealed/hash/isolation/sequential/lease, từng roleC2S native
replay với CRCbad/partial, seq S2C riêng/khôngreset, slot/origin/translation/timeline vàcallbacks.
**22 ca native handler tổng hợp**, **246 ca native frame từ actualcapture/fixture**,
**340 snapshotstate thật** kiểm checksum thủ công rồi native52A350. Giá trị actor+28
được snapshot tại actualreport/170begin/end/rankbegin, không đọc sai AIinterface hoặc gọi hàm game bằng GDB.
Nativemodel có world/actor lookup/guard/endpoints/heap/free/HeapAlloc, giữ framer/checksum/blob/
consumer E85B60/E86780/E86910/emptyvector thật; không emulate toàn actor/game/state authority.

Control so before175 ownA1/remoteA56 cả hai vòng. Can thiệp remote170 state56→1 có sauhandler và
trước175, không suy từ cùng rank.17 có thể đã được physics/UDPđưa remote56 ởcontrol; chưa phép loại
transport nên không gán duy nhấtUDP hoặc physics. Bkhông báo17/19 choactorA là bình thường theo
capture ownslot, không dùng absenceC2S thay peerstate. Delayreported19→rank8s là lựa chọn thử để quan sát.

Đọc ảnh`control-targeted/{A,B}/event-19-r1-late.png`: ownAdeathanimation/skull, BremoteA bubble.
`forward170/{A,B}/event-19-r1-late.png`: BremoteA không còn bubble trước175; frame animation local
khác thời điểm không làm bằng chứng state getter vô hiệu, không yêu cầu ảnh giống pixel.
Run320s đủ hai175/180/phòng; actor B state1 trên A trước175. Ở vòng2 trên A, actor A
có state0 sau170/17, state20 sau170/19, rồi0 trước175. Không khẳng định actor còn lại luôn0
hoặc bất biến qua handler; cần control riêng để gán nguyên nhân cho state20.
Ảnhcảhai vòng2 rankPERFECTKRLocal/LOSEKRGuest; outcome làpolicyđổiAwin/Blose từreportedB19.
Ảnhphòngcuối giữbothnames/hostReady. Chỉ cùng8connection vàstatecyclehẹp, chưa noleak/gate.
Các ảnh đảo chiều vòng2/rank/phòng vàgiới hạn thời gian được khóa theo run trong rawindex.

B41150 từng gắnnhãn`ACTOR_DEATH_V110` là navigation sai nghĩa: hàm cũng được gọiE7CC16 mỗi tick.
Probe mới chỉ ghi cạnhE86A17 từ170 vàsnapshot, không xem genericfunctionhit làdeathproof. Heavyprobe/
Spacefirstmiss giữ lịch sử, inputretry0.8s chỉ khi chưa118; không kết luận causalfix từnhiều thay đổi.
Native/snapshot/report lỗi chuẩn bị giữ notes, không sửa oldsealedrun/CP028 để passcheck.

VN docs `oc_e-game.md` phần rescue/two-client vàserver`session.rs:388` là patternreference:
clientreportownslot/serverchosenoutcome, không xác định KRopcode/enum/layout. KR170dispatch/header/
guard/slot/state được đọc và đo riêng. REA coldIDA CP024 timeout/closed không lặpunchangedrequest;
checkpoint này dùng cache/Capstone/nativeUnicorn/WineGDB, không nhận decompile chưa có.

Còn thiếu reportvalidation/originslot/generation/duplicate/stale/timeout/rescue/chain/item/pickup,
resetnoleak/hostleave/disconnect/fullauth/cache/codec/UDPgrammar/relay/NAT/LAN/Internet; **chưa K6a**.
Policy rank/fuse/timer/remoteonlyforward là emu_choice/design, không serverauthoritygốc.

## Hash metadata

| File dưới peer-events-r1 | SHA256 |
|---|---|
|verification.json|`aeafb11651bc76d656b8e816f8480f0bc9e830f440ae29994ace05ce7ca92d85`|
|runtime-summary.json|`66c44a0a1ed5f388a24af42d0725e2c9c23722d5943e5fe84f42844505674f3e`|
|native-events.json|`ed6cc4e13a8c94b4f9b738cf0696e22ecde352d2ce80e56d5a9f35bf79fd30ab`|
|native-actual.json|`d420d57c79aa82902fd866133ce8f0a7f6117fef7b8156002f5f9ba067c56e64`|
|actor-states.json|`4414b59f440c52a196d972f39ac7a752cfddd7cbf8e2944b60f6ddc17ed55b6b`|
|code-match.json|`1a01b44c11c3f17fa4db5095abffb962465a53c0515784a96061aebbbe8bfbb0`|
|static-ledger.json|`6500a5f07a71489dc8387ef74a31b768da56e3a2799d5e672eb3f598c9415d8f`|
|measurement-notes.json|`390be07f8f4b57877b054b98cee698151c7339dc4ded264e79b7538dd917c94e`|
|baseline-hashes.json|`7dd00b7c2751ba885454c6671e8a4226be6227d479944248e945d7137d77a86b`|
|control-lean-hashes.json|`93b8fd77dcb1f3b2072be4443ab0274b12489202ee27d1bf7df417a2fd452b33`|
|control-targeted-hashes.json|`0657483861c511f2aa0759edb64c53e0cc1864052b300d4dfff681acc088267e`|
|forward170-hashes.json|`be49b32bbb6820eeae8909020a4d5b289942e01ab19f92e0274d7ecb4f631fe7`|
|alternating-hashes.json|`c4573df664858e04b3ab5a2079e71e66f12ff6b7b4dbdea55894dc4ef78c51c7`|
|alternating-long-hashes.json|`2efda9a395fe4c8104765e7d06c2128d48a89e647e7197965a40b36467c74707`|

Counter/delta ở [manifest](../checkpoints/manifest.json). Audit kếCP032.
