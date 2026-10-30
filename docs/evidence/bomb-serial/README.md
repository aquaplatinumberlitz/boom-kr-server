# Serial bom và entry peer KR — CP025

Codex solo RE/code/runtime, self-check; review độc lập waived theo [nguồn](../checkpoints/review-waiver.json).
Baseline CP024/PR32 đã merge `d14c708e47a5a80851e1c2e752d658e6bc934ac3`.
CP025 là snapshot trước tích hợp; PR nhánh `kre/bomb-serial-runtime` là nguồn trạng thái merge hiện tại.
Contract/field/limits chỉ giữ ở [PROTOCOL](../../PROTOCOL.md#bomb-serial), peer ở
[entry153](../../PROTOCOL.md#kr-peer-entry); task tiếp ở [RESEARCH](../../RESEARCH.md#q-023),
gate/acceptance ở [PLAN](../../PLAN.md). AUD007/CP024 gần nhất; audit định kỳ kế CP028.

## Phạm vi và artifact

Bốn Wine run tuần tự, mỗi recipe cấu hình108s, prefix riêng, namespace user/net/mount/PID,
chỉ lo/no outbound route/files-only NSS/local stubs. Lease bên trong namespace; chạy tuần tự.
Không đổi source binary/pack/DB; không patch thêm vào copy CP020 ở checkpoint này.
Raw/reports/asm/ảnh dưới `local/research/codex/bomb-serial-r1/`, ignored.

| Artifact | SHA256 |
|---|---|
| Runtime copy CA-r2-stage-social.exe |`e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef`|
| Static copy CA-r2-login-logout.exe |`660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3`|
| Native oracle modO.bin, memory-layout/base400000 |`a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5`|

Kế thừa sửa import/API/AC/guard của [CP020](../start-reply/README.md), không phải vanilla.
Native oracle map raw memory-layout tại400000; không PE-remap. RVA=VA−400000.

## Các run và kết quả có đối chứng

| Run ID | Recipe thay đổi/phép đo | Kết quả runtime |
|---|---|---|
|multi-sequential |CP024 mapping, ba lần nhấn Space/di chuyển; ID tăng; bỏ175/180 tự động |Hai118, serial0/28;159 đọc đủ nhưng entity trả nonzero/0; ID2 không được bind. Ảnh local prediction không thay proof callback.|
|serial-corrected |Chuyển serial từaux16 sangu32 đúng; input/map/policy còn lại như control |Ba118 serial0/28/56, ba159 entity/bind ID1..3, ba158 tới kind0/cleanup.|
|duplicates-peer |Sau mỗi159/158, gửi lại payload sau0.5s bằng seq mới; probe hai compare branch/bind/sendto |Ba159 gốc trả entity, ba lặp trả0. Sáu158 nhưng ba kind0/cleanup; ID lookup của bản lặp vẫn nonzero. IP/port99 nối actualbind9751.|
|stale-id-reuse |Gửi payload159/158 cũ0.5s sau khi ID kế bind, seq mới |Hai159 cũ trả0; hai158 cũ không tới probe lookup-return/kind0; ba bản gốc vẫncleanup. Samepointer tái dùng cho IDs1..3. Port99/bind9345.|

Không có client B thật; guest tổng hợp. Không gửi153/532 trong các run này; peer suffix được RE tĩnh,
không gọi đây là peer runtime hoặc movement sync. Không tự gửi175/180, nên không có nghiệm thu
result/reset ở CP025. Snapshot cuối input có local gần bom/bị chặn hướng đi, không suy mọi va chạm
hoặc thoát bom thành công. ID/fuse4s/zeros/args là emu_choice; các gate giữ mở.

## Kiểm và recipe

Các recipe argv gốc ở `<run>-recipe.json`; harness sources và GDB được hash trong manifestCP025.
Nếu lặp runtime dùng **run ID/prefix mới**, không ghi đè raw đã seal:

```bash
python3 local/research/codex/bomb-serial-r1/run_probe.py new-label CA-r2-stage-social.exe 108 wine_probe_bomb_multi_serial.py local/research/codex/bomb-serial-r1/serial.gdb
python3 local/research/codex/bomb-serial-r1/seal_run.py new-label
python3 local/research/codex/bomb-serial-r1/replay-run.py new-label
```

`verify-native.py` gọi14EF300 và typed getters native cho actual99/118 và tất cả159/158 fixtures,
kèm CRC sai/frame thiếu cho reply. `replay-run.py` kiểm mọi frame C→S trên từng connection cùng hai
negative cases. Native receiver/heap/prefix-check được mô hình hóa, không thay callback runtime.
`verify.py` kiểm hash/index, isolation/lease/sequential, echo/serial, ID/callback timelines, port99/bind,
seq S→C liên tục,16 vùng code đồng nhất static/runtime/native,153/532 dispatch và guardAL0.

Tổng **1649 phép kiểm cơ học/4run pass**, gồm **111 ca native typed/framing**.
Kiểm đọc code/log/ảnh do Codex thực hiện riêng; checker không xác nhận semantics/gate.
CP024 REA/IDA cold analysis đã timeout, session đóng; checkpoint này dùng code cache/hàm KR native
và Wine GDB, không lặp query hoặc nhận kết quả decompile chưa có.

## Correction và giới hạn nguồn

CP024 chỉ đo serial0; mapping u32→aux16 lúc đó là candidate. CP025 control có serial28 bác cách dùng
tổng quát này, không sửa record/raw completed cũ. Probe kế thừa ghi nhãn `serial` cho aux16 trong hai
run đầu; hai run sau đổi nhãn, contract dựa native/static/compare thực. AEF814 không hit không có nghĩa
không so serial: nhánh AEF7AB được probe và hit ở hai run sau.

VN `boom-re/docs/oc_e-game.md` §5a-6 là reference pattern peer; không phải KR oracle. Targeted xref
chỉ là encoded call/pointer candidate, không phủ indirect graph. Những lần thử snippet lệch instruction
hoặc decode rỗng và lỗi đặt lower-bound307 thay312 giữ trong notes/raw; kết luận dispatch dùng aligned
cachedE5D3A0 và table byte đọc lại. Các cạnh mở thêm bám actual99→manager→13E1190→E825D0;
không quét/devirt toàn client. Header153, endian/HostCreate runtime, UDP codec/key/slot và hai client
còn thiếu; không gửi suffix trần hay mặc định KR532 bypassguard.

## Hash bằng chứng

Full raw list/size/hash mỗi run ở `<run>-hashes.json`; manifest checkpoint pin các index này và các
run/log/fixture trực tiếp. Hash hiện tại sau khi run hoàn tất:

| File local (dưới bomb-serial-r1) | SHA256 |
|---|---|
|run-index.json|`1c605b15c6785d1518ff575bde5031a27eb438f01120fadb068ae4c2f72f3341`|
|verification.json|`9d59640e21e10bb577ba72e2f4974e2aae406c394f99ea2cdf914f5e4e1a378b`|
|native-fields.json|`3f8aabcad548597e8d475b1caa5c719f0d2d7a6fc5fd6ba4efdd76364b5c37cf`|
|code-match.json|`316224b24c92296e3efc63d2acdae5382fbe6a54c6f583a2dda8cac10e6818ad`|
|runtime-summary.json|`2298e4bc08d2a60c7d8175ac169b57db565b7f2e412a2f7911b712d692fd7d3b`|
|measurement-notes.json|`c9113f03e93e90a0942880c055968b1b2249e1305c1a60058932379f6856d0b3`|
|peer-table-dispatch.json|`e936a5fce0e98b4d952e5d4cddbdcebf1158cc8eace42aad78d7d3c9b5b371fe`|
|multi-sequential-hashes.json|`8ab2d91c9e13aec5d0acb851d50c6425e471289d76be3b45ee56a13e4caf53ab`|
|serial-corrected-hashes.json|`b06104cfda5976296fb70b88247ad529a1fd052f8ea413b9532a906805794391`|
|duplicates-peer-hashes.json|`b41f0d538595ddb3a65d58adf9fc4364ea73f706377db541c926abd5ae70017e`|
|stale-id-reuse-hashes.json|`1753426f6938a2605e2f2c17fca8dc607cd612462affeb16eef05af422fe1d50`|

Counter/delta/evidence đầy đủ ở [manifest](../checkpoints/manifest.json). Chưa có audit mới tại CP025; chu kỳ tiếpCP028.
