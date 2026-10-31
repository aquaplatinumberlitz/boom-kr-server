# Hai bom liền kề, control và replay — CP033

Baseline CP032/[PR40](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/40) merge
`fe46beb76291e5dda2d2fdd14c97e40b7cbe0602`. CP033 snapshot trước tích hợp; PR nhánh
`kre/chain-runtime` giữ trạng thái merge hiện tại. Codex solo/tự kiểm, review độc lập
[waived](../checkpoints/review-waiver.json). Contract duy nhất ở
[PROTOCOL](../../PROTOCOL.md#bomb-chain), next/gate ở [RESEARCH](../../RESEARCH.md#q-023)/[PLAN](../../PLAN.md).
AUD010/CP032 gần nhất; audit định kỳ kế CP036. Không gate mới đóng.

## Artifact và recipe

Raw/script/fixture/disassembly/report: `local/research/codex/chain-r1/`. Sáu controller tuần tự,
lease trong namespace user/net/mount/PID chỉ loopback/không outbound/files-only NSS/local stub;
A/B có prefix/wineserver/display/game-dir riêng. Không patch thêm hoặc ép state/hàm game bằng GDB;
source binary/pack/DB giữ nguyên. Copy diagnostic kế thừa CP020 import/API/AC/guard và initial reset,
không vanilla.

| Artifact | SHA256 |
|---|---|
| Runtime A/B CA-r2-stage-social.exe |`e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef`|
| Static CA-r2-login-logout.exe |`660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3`|
| modO.bin raw memory/base400000 |`a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5`|

RVA=VA−400000; không PE-remap modO. `code-match.json` khóa8 vùng khớp bốn artifact,
gồm kind0/collection/blast entry/helper/setter/writer/reader. Chỉ phạm vi đã pin, không toàn call graph.

| Run / bắt đầu UTC31/10/2026 | Đo | Kết quả và giới hạn |
|---|---|---|
|spaced-control /01:47:23|175s|B hai bom cách hai ô; chỉ event ID1, bom2 còn ở cả hai ảnh cuối.|
|adjacent-first /01:52:56|175s|Hai bom liền kề, một event ID1, hai bom không còn; entry kind0 một lần chưa đủ mô tả chain.|
|adjacent-trace /01:57:31|175s|Collection ban đầu root selected1/secondary0; không dùng cờ này thay toàn đường mở rộng.|
|adjacent-replay /02:01:36|175s|Blast process cả hai pointer; duplicate ID1 và stale ID2 sau chain đọc đủ nhưng không lặp kind0/processing.|
|spaced-trace /02:05:36|175s|Chỉ root được ghi state0 và xử lý blast, bom2 còn; cùng probe với ca state dưới.|
|adjacent-state /02:09:24|175s|Secondary setter từ EBE21A, cả hai pointer được xử lý; replay không lặp setter/processing.|

Recipe argv pin ở `<run>-recipe.json`, toàn raw file/size/SHA ở `<run>-hashes.json`.
Tái lập bằng label/prefix mới; không ghi đè raw đã seal hoặc chạy controller tranh lease:

```bash
python3 local/research/codex/chain-r1/run_probe.py new-chain CA-r2-stage-social.exe 175 wine_probe_chain_replay.py local/research/codex/chain-r1/chain-state.gdb
python3 local/research/codex/chain-r1/seal_run.py new-chain
python3 local/research/codex/chain-r1/derive_roles.py new-chain
```

## Đọc bằng chứng và giới hạn

- `run.json`:hai actual118 của B, source fields/power/serial và broadcast159 cho A/B. Chỉ bom1 có
  original158; replay nằm riêng trong dynamic S2C và dùng seq mới liên tục, không có timer ID2.
- `native-actual.json`:276 ca native framer/getters/empty-layout blob/actual118/142, fixture159/158;
  CRC sai/partial được từ chối. Allocator tổng hợp; không emulate toàn game hoặc lấy encoder làm oracle.
- `adjacent-state/client*.log`:159 trả hai pointer, root kind0 một lần; setter B32390(0) trên từng
  pointer từ hai caller riêng, EB80E3 xử lý cả hai. `spaced-trace` cùng probe chỉ có root.
  Lookup replay vẫn có ID/pointer nhưng không vào kind0 hoặc các cạnh đã probe lần nữa.
- Đọc14ảnh qua `spaced-visual.png`, `adjacent-first-visual.png`, `final-visual.png`:ca cách hai ô giữ
  bom2 ở cả hai; ca liền kề/replay/state không còn tại hai ô. Không dùng số lần entry kind0 hoặc ảnh
  riêng lẻ thay runtime setter/processing/control.
- `E9ECB0` chỉ là collection đầu: secondary selected0 không phủ nhận chain; probe E9EA74 không hit,
  cạnh runtime thật nằm ở helper EBE180. Nhãn raw `CHAIN_VISIBILITY` chỉ tên probe, code setter còn
  ghi secure+1C/reset+70/+7C; chưa full lifecycle. Nhãn row/col raw từ hai getter đảo với actual118,
  report chuẩn hóa theo actual request và vị trí đã đo. Không viết lại raw để sửa nhãn.
- Read EBE1B0 ban đầu bắt giữa instruction; bỏ các byte giải sai, đọc lại từ prologue EBE180.
  Không tính phần giải sai là kiến thức. Cache/Capstone/Unicorn/Wine GDB được dùng; không lặp cold
  REA IDA đã timeout CP024 hoặc nhận là decompile mới.

**7106 kiểm cơ học pass**,6run sealed,276 ca native actual packet; source/lease/isolation/seq/hash,
origin/placement/callback/state-setter/processing/replay được đối chiếu. Script kiểm quan hệ khai báo,
không xác nhận ngữ nghĩa/gate hoặc thay review độc lập.

VN chỉ giúp hướng thử event nổ và kiểm hai client; KR158/helper/state/field đều đọc/đo riêng.
Fuse6s, empty layout, ID allocator và broadcast/replay là **emu_choice/design**, không server gốc.
Chưa mixed-owner/ba bom/vật cản/race/generation/collision/rescue/disconnect/full auth/cache/codec;
**chưa full match/K5/K6a/LAN/Internet**. Ca leave tiếp ở root khác, không tính vào evidence CP033.

## Hash metadata

| File dưới chain-r1 | SHA256 |
|---|---|
| verification.json | `fcaf934e1e3dc22cb82c023f448b7029eccd84f52f021950fec49ea94733b764` |
| runtime-summary.json | `735111808caef0d9435583f7982a5b1ea51d9eb425cb2283e4d346614dafa63a` |
| native-actual.json | `d69bed7d464bd61d8659c6c40fd1ee0f9e88703d10484998785ff26ad8c51414` |
| code-match.json | `286d576cbefc0218d0f631707e1f9a513bddc82a948e8f5b1c094ad7bb4fe320` |
| measurement-notes.json | `84b4354e3a31a601649cec5acdec63f688d425a119f62e5ceed4f7975fa67e5d` |
| spaced-control-hashes.json | `7a0fb1742d2dc34baf5320fff4131df3502b06c026eb91c40675006924040712` |
| adjacent-first-hashes.json | `35fdde4d902e16f0e23123ee7cb589467c65112309ee52811f2f277cd5e8bf2f` |
| adjacent-trace-hashes.json | `1146e9dd599fbaf9f83169ac3c64740be20720861047976624741ff0a10ab4e5` |
| adjacent-replay-hashes.json | `9d0a3a041cb791b328f3082087ec191f3ebb42c09a7800501a01f9f84491634d` |
| spaced-trace-hashes.json | `5a2a282506d5f70574cb301a05b21689a85b16b8d8b82a4cb06f2e5529dda284` |
| adjacent-state-hashes.json | `b1a09a4582de721057f433013bbdeb10b2bb1b2878c2f3ae47164bff33c9d832` |

Checkpoint/counter/delta ở [manifest](../checkpoints/manifest.json); audit kế CP036.
