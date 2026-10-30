# Item trên nền, nhặt và tranh item — CP030

Baseline CP029/PR37 đã merge tại `99f0ea92713166bc54ed3fec23d7b5ed59b6d93f`.
Codex solo/tự kiểm; review độc lập [waived](../checkpoints/review-waiver.json).
CP030 là snapshot trước tích hợp, PR nhánh `kre/item-pickup-runtime` giữ trạng thái merge hiện tại.
Contract ở [PROTOCOL](../../PROTOCOL.md#item-pickup), bước tiếp ở
[RESEARCH](../../RESEARCH.md#q-023), gate ở [PLAN](../../PLAN.md).
AUD008/CP028 gần nhất; audit định kỳ kế CP032. Không gate mới đóng.

## Artifact, phạm vi và tái lập

Raw/scripts/fixture/disassembly/report ở `local/research/codex/item-pickup-r1/`;
harness `local/research/codex/wine_probe_items_*.py`. Mỗi controller giữ lease trong namespace
user/net/mount/PID, chỉ loopback/không route, NSS files-only và stub local. Hai client có
prefix/wineserver/display/game-dir riêng; bốn controller chạy tuần tự. Không sửa source binary/pack/DB.

| Artifact | SHA256 |
|---|---|
| Runtime A/B CA-r2-stage-social.exe | `e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef` |
| Static CA-r2-login-logout.exe | `660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3` |
| modO.bin, raw memory layout/base400000 | `a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5` |

RVA=VA−400000; modO không PE-remap. Bảy vùng code khớp bốn artifact trong `code-match.json`.
Không thêm patch; vẫn kế thừa copy chẩn đoán CP020 và initial memory guard reset, không vanilla.

| Run / bắt đầu UTC 30/10/2026 | Thời gian đo | Kết quả và giới hạn |
|---|---|---|
|floor-control / 14:12:18|150s|582 đọc đủ và vẽ hai item ở cả A/B; đường Right/Left không qua item, không124.|
|near-control / 14:15:13|150s|A gửi124/index17 nhưng không165; item hiện lại khi rời ô. B chưa gửi124 cho placement gần gạch.|
|pickup-ack / 14:19:02|160s|B nhặt Fire/index18 rồi A nhặt Bomb/index17; mỗi165 tới cả hai, khớp actor/index, xóa pending và item.|
|pickup-race / 14:22:17|160s|A/B cùng124/index19 trước reply; policy chọnA, một165/slot0 tới cả hai; pending và item được xóa.|

Exit0/no collector error không thay mục tiêu gameplay. Hai control giữ nguyên cả phần chưa đạt pickup.
Recipe argv pin ở `<run>-recipe.json`, GDB và harness có hash; toàn raw pin theo file/size/SHA ở
`<run>-hashes.json`. Dùng label/prefix mới, không ghi đè run đã seal hoặc tranh lease:

```bash
python3 local/research/codex/item-pickup-r1/run_probe.py new-race CA-r2-stage-social.exe 160 wine_probe_items_race.py local/research/codex/item-pickup-r1/items-ack.gdb
python3 local/research/codex/item-pickup-r1/seal_run.py new-race
python3 local/research/codex/item-pickup-r1/derive_roles.py new-race
```

## Đọc bằng chứng

- `near-control/{A,B}/final-both.png` và actual124 của A: item17 hiện lại sau khi rời ô,
  không có165. A dùng cùng placement/input trong `pickup-ack`; riêng vị trí item B đổi từ hàng10
  sang hàng8 theo ảnh map. Không suy toàn can thiệp chỉ có một biến ở B.
- `pickup-ack/client*.log`: cả hai đọc165 đầy đủ12/12/errors0, context0 khớp, actor lookup theo
  slot0/1, interface thực `F73D30`, tìm đúng item và qua kiểm tọa độ/kind. Pending của mỗi origin
  được xóa; final-both cả hai không còn item17/18. Chưa đo +capacity/+range từ icon hoặc method hit.
- `pickup-race`: hai input bắt đầu cách 0.880ms; hai actual124
  cách 336.444ms và đều tới trước ACK. Stub giữ request đầu,
  chờ khoảng2s, phát một xác nhận slot0/index19 cho A/B; request còn lại ghi `already-reserved-index`.
  Cả hai có một165, cùng slot/index và xóa pending local. Ảnh `race-before.png` có item giữa hai
  spawn; `final-both.png` không còn item. Một ca tranh index không nghiệm thu toàn validator/stat/reset.

**4251 kiểm cơ học pass** cho bốn run: sealed hash, isolation/lease/sequential, source harness,
seq S2C riêng, replay C2S từng role/kết nối bằng framer KR với negative CRC/partial, actual request,
field/hash/timeline và callback. **57 ca native actual frame**,
**3 ca layout** và **9 ca165 reject/context-mismatch**.
Framer/blob/typed getters/`11ED590` thật; allocator được model. Native reject cases model cache/world/
lookup/context và không chạy semantic positive effect. Runtime callback/ảnh là nguồn riêng;
không dùng encoder của stub làm oracle server gốc.

VN `oc_e-game.md` phần vật phẩm/two-client và `server/src/session.rs:361,484` tại baseline
`boom-re@9ac2075bbec0c16217741aaedea968cf685e39a9` chỉ dẫn cách thử layout trước start và
report→confirm. KR582/124/165 và index32bit được nối từ dispatcher/writer/reader/native/runtime riêng;
không copy VN553/117/158 hoặc index16bit. REA cold IDA CP024 timeout/closed không lặp query cũ;
lượt này dùng cache/Capstone/native Unicorn/Wine GDB, không nhận là decompile mới.

Tọa độ record item là hàng/cột; point spawn từ map có quy ước riêng, không chuyển nhãn giữa hai kiểu.
`E9E660` đã quan sát là nhánh kind0 bom, không phải receiver pickup. Cạnh effect mới dừng ở
`F73D30→B52840→E14DA0`; chưa đo stat/cap. Placement/first-arrival/2s delay là emu_choice/design.
Item ẩn/phá block/drop, biến thể loại item, duplicate/stale/generation/reconnect/reset/disconnect,
full auth/cache/codec và LAN/Internet vẫn mở; **chưa full match hoặc K5/K6a**.

## Hash metadata

| File dưới item-pickup-r1 | SHA256 |
|---|---|
| verification.json | `da68421d12078c6ca0e205a1bd803bf323bf3fe348b7812666d67aef2a37f82f` |
| runtime-summary.json | `b153a9c05ae47150396b761095d0eaf99912163129861067632dafb387a6ecb6` |
| native-layout.json | `ce955e27e635f2d55f33a6f85828f8818e39f0f63e8eabc75fd26effbce2edda` |
| native-pick.json | `11097ee699bdf1a29000764f843487ef6c57ccbfa5da99d9b67bd2592e200886` |
| native-actual.json | `541231ccd41ac91e40647d0ce811b1e3581573c84d4cc8de619d619b72174c48` |
| code-match.json | `c6aa5662493b636e6a9ca690e7547a3487db784733ebf7f94a7315c4281cf713` |
| measurement-notes.json | `6365cda62ed0a949ce0b248b745a32a9aadca151c093b9fb7497ffe190fef458` |
| floor-control-hashes.json | `d41e01ed2200ee0f72b3fbae418a7c71634dfdb19e359819bcbba2c7022ee84d` |
| near-control-hashes.json | `683ccdd3efaf7a490f99a94ff82d836e6320d426eea37d5912a00799e1202d56` |
| pickup-ack-hashes.json | `31b07ec7a0ef10bd190d033cbada72ec19b4e82d604dfe968f5763e189bdf795` |
| pickup-race-hashes.json | `68e1d294820988d913bc71acf49349b240bc755486952e879441bdc41e03e81f` |

Checkpoint/counter/delta ở [manifest](../checkpoints/manifest.json); audit kế CP032.
