# Item ẩn trong gạch, reveal và vòng kế — CP032

Baseline CP031/[PR39](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/39) đã merge
`4de4e2ef2e0de4bba4c6a365cf669b97db618790`. CP032 là snapshot trước tích hợp; PR nhánh
`kre/hidden-item-runtime` giữ trạng thái merge hiện tại. Codex solo/tự kiểm,
review độc lập [waived](../checkpoints/review-waiver.json).
Contract duy nhất ở [PROTOCOL](../../PROTOCOL.md#hidden-items), next/gate ở
[RESEARCH](../../RESEARCH.md#q-023)/[PLAN](../../PLAN.md).

## Artifact và recipe

Raw/script/fixture/disassembly/report: `local/research/codex/hidden-item-r1/`.
Năm controller chạy tuần tự, giữ runtime lease trong namespace user/net/mount/PID chỉ loopback,
không route outbound, NSS files-only và stub local. A/B có prefix/wineserver/display/game-dir riêng.
Không thêm patch, không ép stat/stage hoặc gọi hàm game từ GDB; source binary/pack/DB giữ nguyên.
Copy kế thừa CP020 import/API/AC/guard và initial reset, không vanilla.

| Artifact | SHA256 |
|---|---|
| Runtime A/B CA-r2-stage-social.exe |`e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef`|
| Static CA-r2-login-logout.exe |`660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3`|
| modO.bin raw memory/base400000 |`a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5`|

RVA=VA−400000, không PE-remap modO. `code-match.json` khóa11 vùng khớp bốn artifact,
gồm list/parser/install/storage, reveal, writer142 và code stat/pickup/power kế thừa.

| Run (UTC30/10/2026,31/10 JST) | Đo | Kết quả/giới hạn |
|---|---|---|
|hidden-explode|155s|Counts0/2/0, hai gạch vỡ/item hiện cả A/B; A thực nhặt27/28, cả hai áp stat.|
|hidden-no158|155s|Giữ input/layout nhưng không158: bom/gạch còn, không142/124/165; đo actual tile storage trên cả hai.|
|hidden-rounds|320s|Hai lần load/reveal/nhặt/stat/serial; chỉ175/180 vòng1 vì harness chờ sai origin ở vòng2. Không gọi hai chu kỳ hoàn tất.|
|hidden-trace|155s|Nối cạnh reveal tới đúng CGOItem pointer được165 tìm thấy; writer142 runtime ở cả hai.|
|hidden-rounds-fixed|320s|Chỉ sửa policy origin A/B→A/A; hai175/180/phòng cả hai phía, reset stat/item/serial/power lặp đúng scope.|

Recipe argv ở `<run>-recipe.json`, toàn raw file/size/SHA ở `<run>-hashes.json`.
Mỗi run/prefix mới dùng label mới; không ghi đè raw đã seal hoặc chạy controller song song:

```bash
python3 local/research/codex/hidden-item-r1/run_probe.py new-hidden CA-r2-stage-social.exe 320 wine_probe_hidden_rounds_fixed.py local/research/codex/hidden-item-r1/hidden-rounds.gdb
python3 local/research/codex/hidden-item-r1/seal_run.py new-hidden
python3 local/research/codex/hidden-item-r1/derive_roles.py new-hidden
```

## Đọc bằng chứng

- `hidden582-fixture.json` pin counts/index/row/column/kind của list2; `native-actual.json`
  giữ framer/blob/11ED590 thật, CRC sai/partial, actual124/118/142 và fixture165/159/158.
  S2C là emulator choice; actual C2S và runtime callback là nguồn riêng.
- `hidden-no158/client*.log`: install và store trong `CGOTileFixedBox`; hai index không thành
  floor item chỉ từ parse/install. Đối chiếu ảnh control với ảnh blast và sau Up/Left của positive.
- `hidden-trace/client*.log`: `HIDDEN_REVEAL_COMMIT` và `ITEM165_FOUND` cùng pointer/index,
  destination RTTI `CGOItem`; `BLOCK142_WRITE` nối tới captured142. Canonical offsets/schema chỉ ở contract.
- `stat-states.json`: wrapper/data capture, slot→actor, checksum giải độc lập và native529C60→52A350;
  phase0 lúc clock, phase1/2 trước/sau165. Hai vòng được gắn marker riêng; không suy reset từ pointer mới.
- `hidden-rounds`: actual A17/19 vòng2 đã có nhưng inherited alternating policy chờ B; giữ raw partial.
  Ca sửa chỉ đổi điều kiện origin trong harness mới, không sửa client hoặc data cũ để làm check pass.
- Entry `E9DFC0` ban đầu là giả thuyết reveal từ floor-list; probe không hit, đường thực là B9CEC0
  tới B8B5B0. Hai breakpoint queue tại cùng địa chỉ có lệnh continue trước đã che probe142 thêm sau;
  không dùng absence đó làm negative report. Probe mới ở E72849 và capture/native xác nhận writer.
  Một read ASM tail ban đầu bắt giữa instruction tại B9C980 không được dùng; bản đọc lại từ entry khóa đúng byte.

VN `boom-re/docs/oc_e-game.md` §5a-5d gợi ý list2/FixedBox và report phá gạch; KR index32bit,
writer142/7byte và object/stat được nối riêng. Không copy VN553/135/117/158 hoặc u16 đuôi.
Chỉ dùng cached bytes/Capstone/native Unicorn/Wine GDB; không lặp REA cold IDA đã timeout ở CP024,
không nhận là decompile mới.

Placement/index/fuse6s/broadcast/ACK/rank là **emu_choice/design**. Chưa random drop/server validator,
list3/loại block khác/chain/rescue/stale/generation/disconnect hoặc full auth/cache/codec.
Reset chỉ những field/đối tượng đã đo; **chưa full match/K5/K6a/LAN/Internet**.

## Kiểm, hash và audit

**8674 kiểm cơ học pass**, **429 ca native packet**, **496 snapshot stat**, **40 actual report142**.
Đã đọc18ảnh qua ba contact sheet và đối chiếu log/code/native/field. Native allocator tổng hợp;
script chỉ kiểm tính cơ học, không xác nhận ngữ nghĩa/gate. AUD010/CP032 periodic; audit kế CP036.

| File dưới hidden-item-r1 | SHA256 |
|---|---|
| verification.json | `5f82c0128f82c432ba889dfb2bf214b75b5cee117f4eb53fd62959373961dc97` |
| runtime-summary.json | `2ea82393aa502b752152db7f7d33ec04130c6667452cbb6b722de95ce87935e2` |
| native-actual.json | `ebede52abfed03c8c619f5a16aea3251a299e5f27377a0d1f947d4eb1f2580e1` |
| stat-states.json | `15f6c7f7d2e5076bd341fbbcba7df4fbe29e76315d5be7e4ee32e382229252bf` |
| block142.json | `32f1ce27bcfac9416e25f8e69a85b32a3ddf9a6be8829b11f454f3e2bffb8e89` |
| tile-rtti.json | `f44b2df2f803be682f358c25e0b538019b6d622db4b589059c1afdc6329ccd0a` |
| code-match.json | `4e9829ffb933fba5cf1b546052fde08509e929d78e5c99376672b735f09c94cb` |
| measurement-notes.json | `f2c5da3336de83d13da3c7c0eccd5d758a501a6149825d8639922f962c4dc2de` |
| hidden-explode-hashes.json | `dcd4cf79cc34a7d6a61dc0848c3531f5b047ad7a17a83f2309456517d02caa95` |
| hidden-no158-hashes.json | `c8f62300b67e150c93c8ff3024a97758724c59fdac134c40cd940fb786ef3f5f` |
| hidden-rounds-hashes.json | `150669caba20ff3e446324bf0ec7ff6a26517c4d83cad8850286677c870afb98` |
| hidden-trace-hashes.json | `b4a3c61d119bf619baf69844a19caeb73df225989e9a3db6c34264f1c217a1db` |
| hidden-rounds-fixed-hashes.json | `01207968ee23f17bfcc71c802cedbaf84e3879b2a183e89eedc32ec006cd78d1` |

Checkpoint/counter/audit ở [manifest](../checkpoints/manifest.json).
