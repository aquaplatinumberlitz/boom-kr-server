# KR153 và UDP sink — CP026

Codex solo RE/code/runtime/tự kiểm; review độc lập [waived](../checkpoints/review-waiver.json).
Baseline CP025/PR33 merge`80bc6efb8f462513dd14225eabf17dfa4878588a`.
CP026 là snapshot trước tích hợp; PR nhánh`kre/peer153-runtime` giữ trạng thái merge hiện tại.
Contract chỉ giữ ở [PROTOCOL](../../PROTOCOL.md#kr-peer-entry), bước tiếp ở
[RESEARCH](../../RESEARCH.md#q-023), gate ở [PLAN](../../PLAN.md).
AUD007/CP024 gần nhất; audit định kỳ kế CP028. Không có gate mới đóng.

## Recipe và artifact

Bốn run Wine tuần tự, user/net/mount/PID namespace chỉlo, không outbound route, files-only NSS,
stub local; lease bên trong namespace. Prefix/runID riêng. Raw ở`local/research/codex/peer153-r1/`.
Không sửa source binary/pack/DB, không patch thêm vào copy CP020. Vẫn là copy chẩn đoán có sửa
import/API/AC/guard theo [CP020](../start-reply/README.md), không phải vanilla.

| Artifact | SHA256 |
|---|---|
| Runtime CA-r2-stage-social.exe |`e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef`|
| Static CA-r2-login-logout.exe |`660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3`|
| modO.bin memory-layout base400000 |`a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5`|

VA/RVA trong manifest; RVA=VA−400000. Native oracle maprawmodO tại400000, không PE-remap.

## Kết quả có đối chứng

| Run ID / bắt đầu UTC30/10/2026 | Config / thay đổi | Quan sát |
|---|---|---|
| empty-peer153 /11:18:17 |105s;15361byte, stream rỗng,26slot tắt |Đọc61/61/errors0; input có nhấn nhưng không118; ảnhComputer0/1.|
| guest-peer153 /11:20:21 |105s;15379byte, bậtguest1,local0tắt; IP0100007f/14001 |Đọc79/79; register1/IDs1,2 vàHostCreate; không118/sink0.|
| both-peer153 /11:22:44 |100s;15397byte, giữlocal0bật/3endpoint0; guest như trên |Đọc97/97; HostCreate trảptr; ba118/159/158; sendtohit nhưng sockaddr1.0.0.127,sink0.|
| fixed-peer153 /11:24:41 |95s; chỉ sửa peerIP7f000001 |Đọc97/97; ba118/159/158; sockaddr127.0.0.1:14001, sendto trả17, **125datagram17byte** sink nhận từ127.0.0.1:9788.|

Sink chỉ nhận/lưu, không trả lời. Chưa có clientB, handshake/UDP receive/movement sync, relay/NAT hoặc
trận hoàn chỉnh. Datagram chưa được giải mã ý nghĩa. Hook chỉ log10sendto đầu/20envelope đầu;
capture sink độc lập là bằng chứng delivery. Không dùng sendto-entry hoặc HostCreate thay delivery.
Bốn ca gửi153 sau198; clock origin khởi tạo lại, chưa chọn chuỗi production. Local zero-endpoint và
các field header/relay/vector rỗng là emu_choice, không suy đây là schema server gốc hoàn chỉnh.
Ảnh fixed sau input cho thấy tênKRLocal/KRGuest và vị trí local khác spawn; không phải hai client.

Recipe argv tại`<run>-recipe.json`, harness/GDB được pin trong manifest. Nếu lặp dùng ID/prefix mới:

```bash
python3 local/research/codex/peer153-r1/run_probe.py new-label CA-r2-stage-social.exe 95 wine_probe_peer153_fixed.py local/research/codex/peer153-r1/peer153-fixed.gdb
python3 local/research/codex/peer153-r1/seal_run.py new-label
python3 local/research/codex/peer153-r1/replay-run.py new-label
```

## Kiểm và phạm vi nguồn

`native_all.py`:12ca,4fixture×valid/CRCbad/partial. Hàm native14EF300, stream/vector decoder và getters
kiểm đủ body; receiver allocation/C heap malloc/free được mô hình hóa. Không emulate full game state.
`replay-run.py` kiểm mọi frame C→S thực theo từng connection/seq cùng hai negative cases.
`verify.py`:HASH/run isolation/sequential/lease, cursor/header/26slot, actual118, registration/HostCreate,
sendto/sink và11vùng code khớp static/runtime/native. Tổng **1598kiểm cơ học pass**,12ca native riêng.
Đọc code/log/ảnh và interpretation do Codex tự kiểm; script không xác nhận semantics hoặc đóng gate.

REA/IDA cold requestCP024 đã timeout và đóng; không lặp query không đổi. CP026 dùng Capstone/hàm KR
native/Wine GDB theo các cạnh153→stream→peer→sender. Không nhận kết quả decompile chưa có.
VN`boom-re/docs/oc_e-game.md` §5a-6 và helperblob là reference, không copyopcode/slot/key.

Thử native ban đầu thiếu length/obfuscation và thiếu heap model được giữ trong raw failure outputs;
không gửi các chuẩn bị lỗi đó live. Snippet14E5862 lệch instruction giữ là navigation; aligned14E585D
và caller14E61F0 dùng cho kết luận. Sai peerIP là control runtime có ngày/run/hash, đã sửa bằng
htonl thực/capture; CP025 chỉ để endian unknown, không ghi đè checkpoint cũ. Checker routes nhận
/proc/net/route có header nhưng không có entry; kiểm đúng chỉlo/header-only. Notes giữ chi tiết.

## Hash bằng chứng

Full file/size/SHA mỗi run ở`<run>-hashes.json`; manifest pin index và raw quan trọng, gồm125UDP.

| File dưới peer153-r1 | SHA256 |
|---|---|
|verification.json|`20aad96352d6ad8b117ef34009d194e3c27010790d439196eab46d7bb1be5d4f`|
|runtime-summary.json|`db3f08a31fcad7804d24e9501e07265b9b3a97f70378bff290cfa3b72f05303c`|
|native-all.json|`e1367a0c6b2aa59086f89325110ed75a438e57a6b381e3ee3e98f7c54c11ab10`|
|code-match.json|`700f0cb5e70f60606a829565d57f075ae806f2da3c089dbe56c88be1111fd8f3`|
|measurement-notes.json|`dfe7ca19320ce4eb3ed2bfeeaac67fb346b13658835aebdbf09ddcaf79dc9f46`|
|empty-peer153-hashes.json|`82ad4904e4d546e198cab338bf68a2b05976a0628d6477104b54b60b4ee0962e`|
|guest-peer153-hashes.json|`a241b3f3a037fec88f1461085bc7d21c1350d802b23b865e201d3572fa31809c`|
|both-peer153-hashes.json|`849c76d720ef689a50be1e4b60982c975ec304d738ef1e6667263481383d7134`|
|fixed-peer153-hashes.json|`60e2fcbb430635ab5377ef680d8e05d20e76a5c4bfbff98541b546521f2f592e`|

Checkpoint/delta/counter ở [manifest](../checkpoints/manifest.json). Audit kếCP028.
