# Hai client KR: movement và bom — CP027

Codex solo/tự kiểm, review độc lập [waived](../checkpoints/review-waiver.json).
Baseline CP026/PR34 merge`2030e77ca938925a6368b9a225350ba7b409c98a`.
CP027 snapshot trước tích hợp; PR nhánh`kre/peer-duo-runtime` giữ trạng thái merge hiện tại.
Contract/giới hạn ở [PROTOCOL](../../PROTOCOL.md#peer-duo), task tiếp ở
[RESEARCH](../../RESEARCH.md#q-023), gate ở [PLAN](../../PLAN.md).
Audit gần nhất AUD007/CP024, kế CP028; không có gate mới đóng.

## Artifact và cô lập

Raw`local/research/codex/peer-duo-r1/`, ignored. Một run control rồi ba run pair tuần tự, mỗi run
user/net/mount/PID namespace chỉlo/no outbound/files-only NSS/local stub. Controller giữ lease;
hai client trong pair chung namespace nhưng prefix/wineserver/display/game-dir riêng. Không chạy
hai controller tranh lease. Bgame là copy riêng của copy local, không phải source client.

| Artifact | SHA256 |
|---|---|
| RuntimeA/B CA-r2-stage-social.exe |`e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef`|
| Static CA-r2-login-logout.exe |`660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3`|
| modO.bin raw memory-layout/base400000 |`a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5`|

Không sửa binary/pack/DB nguồn hoặc thêm patch. Kế thừa import/API/AC/guard copy CP020,
không vanilla. Native maprawmodO tại400000; RVA=VA−400000. Bảy vùng code khớp bốn image
(static, runtimeA, runtimeB, native); locator và byte/hash trong manifest.

## Run và kết quả

| Run / bắt đầu UTC30/10/2026 | Config | Quan sát và trạng thái |
|---|---|---|
| replace-198 /11:37:00 |95s, một client |153 thay198/seq82,97/97/errors0; clock1; ba bom;133UDP sink. Run hoàn tất.|
| duo-first /11:43:05 |185s, hai client, chỉ movement |ID1/2 qua auth; actual99ports9858/9720;153 full/clock1 mỗi bên; send/recv thật, ảnh movement hai chiều. Run hoàn tất.|
| duo-bombs /11:46:39 |185s, hai client, broadcastbom |Ports9218/9564;4actual118/ID1..4;4entity/4kind0/cleanup mỗi bên. **Runerror/exit1** do ảnh cuối timeout5s; raw giữ nguyên, không gọi toàn run pass.|
| duo-confirm /11:51:14 |180s, cùng phép thử, recorder sửa |Ports9237/9351;4bom/4entity/4kind0/cleanup mỗi bên; snapshot remote bomb; run hoàn tất không lỗi/captureerror.|

B vẫn đi create86→server push121 tạistage12 sau map/profile, chưa join/ready chuẩn. Cùng roster/map
trong harness không phải nghiệm thu phòng chung production/fullauth. Sink không dùng để thay clientB.
Không tự gửi175/180. Chưa fullround/item/trap/result/reset hai trận/disconnect, K6a/b/c chưa đạt.

Các recipe argv ở`<run>-recipe.json`; scripts/harness/GDB và inputfixtures được pin. Lặp dùng
runID/prefix mới, tạo Bgame-copy riêng nếu chưa có (không sửa source, không ghi đè raw):

```bash
python3 local/research/codex/peer-duo-r1/run_probe.py new-duo CA-r2-stage-social.exe 180 wine_probe_peer_duo_confirm.py local/research/codex/peer-duo-r1/duo.gdb
python3 local/research/codex/peer-duo-r1/seal_run.py new-duo
```

## Kiểm và đọc bằng chứng

**6331 kiểm cơ học pass**, gồm hash/isolation/lease/run liên tiếp, role/connection/identity/endpoint,
seq, callbacks bom và seven vùng code match. **180 ca native** frame/CRCbad/partial, stream/vector/
typed getters;106 chỉ prefix5byte, auth chỉ prefix34byte. Receiver alloc/C heap được mô hình hóa;
không dùng nativeframer thay runtimecallback. Mỗi connectionC2S được replay riêng cùng controls.

**180 successfulrecvfrom samples**,30/client/duorun:173×17byte,7×5byte. `parse_udp.py` trích đúng
nbyte từ memory dump API return, kiểm sourcepeer/port và checksum native14A5860(seed0,n−1).
API capture có file/hash, không phải pcap hoặc fullUDPgrammar/handshake/reliability/NAT proof.
Role views/round-index trong`derived/` và`*-round-derived.json` liên kết raw gốc, không sửa raw.

Đọc ảnh thủ công: `duo-first/B/seen-after-A-0-Right.png` và các ảnh tiếp cho thấy A đổi vị trí trênB;
`duo-first/A/seen-after-B-0-Right.png` cho thấy B đổi vị trí trênA. `duo-confirm/B/blast-1.png`
có bomA, `duo-confirm/A/blast-3.png` có bomB. Label`blast-N` là recipe trigger sau emit158, không
đồng nhất với frame nước nổ; actualkind0 được chứng minh bằng GDB. Authority/fullconsistency giữ mở.

REA/IDA coldrequestCP024 timeout/đóng, không lặp query không đổi. CP027 dùng codecache/hàm KR native,
Wine GDB và targeted13DE6D0/13E1820 edges; snippet1167AE0 lệch đầu là navigation, không dùng làm
kết luận. Native/wrapper/runtime/static/source limits, lỗi chuẩn bị harness trước đo và lỗi collector
được giữ trong notes. Không quét/devirt toànclient, không gọi worker.

## Hash metadata

Mỗi run có full file/size/SHA trong`<run>-hashes.json`; manifest pin index/raw quan trọng/API samples.

| File dưới peer-duo-r1 | SHA256 |
|---|---|
|verification.json|`d928ed31d264c6bfc518ffffa13de319972f2dfd2d9eefa38ed8a5d72ce3a720`|
|runtime-summary.json|`f02e4a81d6170f975a67a5d63b02a374fa2de2704ca61a72d9be271870793a48`|
|native-fields.json|`908e564ac2c36cfa233bd9122d1d9e342059314d73351656b6cda268f6de268c`|
|udp-api-captures.json|`834be96399276ae542e0ad09ab0219ffb817f9e29ba43abda6b235e7dd51ab8b`|
|code-match.json|`89ac13b30ac92dc1b95075165e8f520246c918db9eac1958caf38d103f990364`|
|measurement-notes.json|`fb33895e541b4f8a253e70dd990488cf2cce1ac26751742f4df970107eebd61b`|
|replace-198-hashes.json|`aafa495ad9ea4ce10c0815605ab801b0c5a41ffcf310cccd756667a041bb9b41`|
|duo-first-hashes.json|`bea57422b0937afe55b1b9b5dba95c320723f37d732546443f29c27f2431be78`|
|duo-bombs-hashes.json|`e124e947a71f0f63c293d15bcd90ab668dbe0fe305902053808fb9a147c0cbe0`|
|duo-confirm-hashes.json|`075455bc4c6b57563ee3852d24d3b74d6a89b45481a5445d5867548bf440c13a`|

Checkpoint/counter/delta ở [manifest](../checkpoints/manifest.json). Audit kế CP028.
