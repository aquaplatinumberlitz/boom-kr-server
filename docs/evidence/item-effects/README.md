# Hiệu ứng Bomb/Fire và correction power — CP031

Baseline CP030/[PR38](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/38) đã merge
`1f70d4fdfd707099bfabdfbafa8bed69ae9be9af`. CP031 là snapshot trước tích hợp;
PR nhánh `kre/item-effects-runtime` giữ trạng thái merge hiện tại. Codex solo/tự kiểm,
review độc lập [waived](../checkpoints/review-waiver.json).
Contract chỉ giữ ở [PROTOCOL](../../PROTOCOL.md#item-effects), bước tiếp ở
[RESEARCH](../../RESEARCH.md#q-023), gate ở [PLAN](../../PLAN.md).
AUD009 audit ngay do mâu thuẫn nguồn power; audit định kỳ vẫn CP032. Không gate mới đóng.

## Artifact và tái lập

Raw/scripts/fixture/disassembly/report: `local/research/codex/item-effects-r1/`.
Năm controller chạy tuần tự, giữ `.kr-wine-runtime.lock` bên trong namespace user/net/mount/PID;
chỉ loopback, không route outbound, NSS files-only, stub local. A/B dùng prefix/wineserver/display/
game-dir riêng. Không sửa source binary/pack/DB hoặc thêm patch; copy vẫn kế thừa CP020 import/API/
AC/guard và initial memory reset, không vanilla. Không gọi hàm game hoặc ép stat bằng debugger.

| Artifact | SHA256 |
|---|---|
| Runtime A/B CA-r2-stage-social.exe | `e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef` |
| Static CA-r2-login-logout.exe | `660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3` |
| modO.bin, raw memory layout/base400000 | `a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5` |

RVA=VA−400000; không PE-remap modO. `code-match.json` pin 12 vùng khớp bốn artifact,
gồm effect chain, hai hàm tăng stat, secure getter/setter, clamp, writer118 và calc bom.

| Run / bắt đầu UTC 30/10/2026 (31/10 JST) | Đo | Kết quả và giới hạn |
|---|---|---|
| effect-path / 22:33:27 |150s|Hai actual124, hai165/client; nối custom0→group1/effect1,2. Chưa đặt bom để kiểm capacity.|
| capacity-control / 22:36:33 |170s|Giữ hai124 nhưng không165; cùng chuỗi input, A có một118/B hai; ảnh cuối vẫn Battle.|
| capacity-ack / 22:39:42 |170s|Có165, A hai118/B hai; receiver thật, mỗi stat increment một lần,159 trả entity cả hai.|
| capacity-duplicate / 22:43:04 |170s|Gửi lặp165 sau1s bằng seq mới;144 snapshot, không tăng stat lần hai; mapping power cũ giữ làm lịch sử.|
| capacity-power / 22:46:18 |170s|Chỉ đổi nguồn power từ118.field6 sangfield10 trong harness;144 snapshot và calc power hai phía khớp Fire.|

Mỗi run có `<run>-recipe.json` pin argv; `<run>-hashes.json` pin raw file/size/SHA.
Các run đầu dùng mapping cũ, không sửa raw/harness/checkpoint đã complete. Tái lập bằng label/prefix
mới để không ghi đè bằng chứng hoặc tranh lease:

```bash
python3 local/research/codex/item-effects-r1/run_probe.py new-power CA-r2-stage-social.exe 170 wine_probe_item_capacity_power.py local/research/codex/item-effects-r1/capacity-power.gdb
python3 local/research/codex/item-effects-r1/seal_run.py new-power
python3 local/research/codex/item-effects-r1/derive_roles.py new-power
```

## Đọc bằng chứng

- `client.log`/`client-B.log`: từ165, effect number tới actual receiver actor+58C, context actor+4A0,
  exact AF06F0/AF0690 và return1. Probe bó hẹp theo cạnh, không lấy generic method hit làm proof stat.
- `stat-states.json`:288 snapshot của hai run cuối, lúc clock và trước/sau bốn165 mỗi client.
  Spawn slot→actor, wrapper/data/checksum và native529C60→52A350 được đối chiếu. Giá trị và offset
  chỉ giữ ở contract; không suy tên character hoặc base/max của toàn roster.
- `native-caps.json`:10 ca native increment/clamp, có current==max và current>max; secure storage/
  context tổng hợp, counters instrumentation khởi tạo0, RNG/code secure thật. Chưa runtime pickup-at-cap.
  `native-dispatch.json`:4 ca actual1191840, receiver/vtable endpoint được model; riêng runtime mới nối
  item→effect và hàm tăng thật. Lần chuẩn bị đầu dùng synthetic address chưa map bị lỗi trước phép đo;
  sửa sang vùng heap đã map, không sửa kết luận để bỏ failure.
- `runtime-summary.json`:noack A1/B2 so với ACK A2/B2 dưới cùng ba Space xen Right; fuse30s do emulator
  chọn để input kết thúc trước158. Không dùng thiếu request thứ ba làm proof cap vì geometry/pool có thể chặn.
- `capacity-duplicate`/`capacity-power`:bốn165 đầy đủ/client nhưng chỉ hai effect/increment, mỗi loại một.
  Duplicate là cùng payload, seq mới; chỉ kiểm hai index đã xóa trong cùng round, chưa reuse/generation.
- Correction: actual118.field10 tăng theo Fire, field6 giữ1. `capacity-power` forward field10→159.field8;
  `E9C58C` nhận power2 tại6/3,6/4 và power1 tại3/3,3/4 trên cả hai. Không suy physical blast range đầy đủ
  hoặc toàn tác động của mapping cũ khi chưa có probe tương ứng ở run cũ.
- Đã đọc ảnh `capacity-after-B.png` và `final-both.png` của A/B trong control/ACK/power (12 ảnh,
  contact sheet `visual-review.png`):control có một bom A/hai bom B và còn Battle; ACK/power có hai
  bom mỗi bên và về phòng. Ảnh là kiểm scene/context; snapshot và native mới là nguồn stat.

**7094 kiểm cơ học pass**, **357 ca native actual packet**, **288 snapshot stat**, **10 ca clamp**
và **4 ca dispatch**. Framer/getters/blob/11ED590 thật, có CRC sai/partial; allocator và context được
model như report ghi. Script kiểm hash/seq/timeline/field/state khai báo, không xác nhận ngữ nghĩa/gate.
Native S2C dùng fixture emulator; actual C2S được capture riêng từng role/kết nối.

VN `oc_e-game.md` phần passive effect và `KR_RE_BOMBERATTR.md` chỉ giúp chọn stat/cạnh kiểm;
offset, effect dispatch, getter, request power và receiver KR được kiểm riêng. REA cold IDA CP024
đã timeout/closed; không lặp query cũ. Lượt này dùng cache/Capstone/Unicorn/Wine GDB, không gọi là
decompile mới. Item ẩn/phá block/reveal, runtime tại max/speed, validator/reset/disconnect/full auth/
cache/codec còn mở; **chưa full match/K5/K6a/LAN/Internet**.

## Hash metadata

Hash report, recipe, raw index và scripts được pin trong [manifest CP031](../checkpoints/manifest.json).
Các raw index bao phủ toàn run; counter/audit cũng ở manifest. Bảng dưới được lập từ file thực sau kiểm.

| File dưới item-effects-r1 | SHA256 |
|---|---|
| verification.json | `56b31e2142f66f50ca901aa3580ea734bcc9608c4bf01e8ee35337c7df4f2683` |
| runtime-summary.json | `dac5c1104d22e27aed97a10ced4d024d4925b3d9ededd8d3ac3fa02d5cfd0fd6` |
| native-actual.json | `238ddb1c62594c64e68f4bda92b608d0ca5c85df793574bf0143ab76a5db590d` |
| stat-states.json | `af50c7b707a999e36b660097ab380100106b4c1307fe21160c1334e806a7ac06` |
| native-caps.json | `021f4b666b4b3d4d26eb559dee3499667ac2ec5969da2cab34be3a5b42d1ff57` |
| native-dispatch.json | `8988f3944ac85ac33248aef26f68ec858d0a02ea558b3c9732475b0a8ac04112` |
| code-match.json | `0883dc92d6437f312b230efee87e27e817c18cf19749db31a7f76f1805035f32` |
| measurement-notes.json | `f3df23aa7ccdc57b85e921731417f6b04838b2f7125a5b1d80167261b0449c74` |
| effect-path-hashes.json | `be21755376a9bf489bb74771c506bd338eaffb8d9451d18f48a2f6f7bef32003` |
| capacity-control-hashes.json | `8790357901dfd3aab6197b65d4abc984f4e6f4cbd173185c41861392e1c647cd` |
| capacity-ack-hashes.json | `501bd8b8cd7dcf5243665318f422971ea465270a435f8fb586e22e703108a054` |
| capacity-duplicate-hashes.json | `f6c036e44f43331ccdd4f833694add25a672de5047fd045a4c03352bc4d011aa` |
| capacity-power-hashes.json | `f685b51e52e2436138f40ba078894785978dd9694b68463feced465792a2d2ea` |
