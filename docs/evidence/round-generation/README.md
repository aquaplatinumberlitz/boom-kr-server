# Stale158 qua vòng mới và fence job local — CP041

Baseline CP040/[PR48](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/48) merge
`99d1835cad8e2d99dec301b88f504e61356f09fc`. CP041 snapshot trước tích hợp trên nhánh
`kre/round-generation-runtime`; PR của nhánh giữ trạng thái tích hợp.
Codex tự kiểm, review độc lập [waived](../checkpoints/review-waiver.json).
[Contract duy nhất](../../PROTOCOL.md#round-generation), [bước tiếp/gate](../../RESEARCH.md#q-023).
AUD013/CP040 gần nhất, audit định kỳ kế CP044; không gate mới đóng.

Raw/recipe/harness/GDB/fixtures/asm/capture/log/native reports/ảnh:
`local/research/codex/generation-r1/`. Ba controller tuần tự, lease trong namespace
user/net/mount/PID, loopback/no outbound/files-only NSS/local stub. A/B prefix, wineserver,
display và game-dir riêng. Không sửa source binary/pack/DB; không thêm patch/ép state.
Copy chẩn đoán kế thừa CP020 import/API/AC/guard và initial global reset, không vanilla Windows.

| Artifact | SHA256 |
|---|---|
| Runtime A/B CA-r2-stage-social.exe | `e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef` |
| Static CA-r2-login-logout.exe | `660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3` |
| modO raw memory/base400000 | `a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5` |

RVA=VA−400000; modO raw, PE copies map theo section. `code-match.json`19vùng/4artifact.
Dùng cache/Capstone/Unicorn/WineGDB, không lặp cold REA/IDA timeout CP024. Read E14440 mở giữa
instruction nên decode rỗng bị loại; đọc lại aligned E14360 rồi E14593. Builder GDB typo E5E800
đã sửa thành entry159 E5E8F0 trước runtime đầu; không có phép đo từ địa chỉ sai đó.

| Run / UTC31/10/2026 | Kết quả | run.json SHA256 |
|---|---|---|
| unique-control / 08:26:23 | ID1→2, hai old158 không kind0; timer mới nổ; hai result/return. | `07592ae3e58881d9b2779eaccd6cc3f243940fe66a5c67e88f988de57f2a3aa1` |
| reused-ID / 08:31:03 | ID1→1, old158 sau bind nổ đúng entity mới trước timer; hai result/return. | `4790d11f20b9960ac3cadf150bac692062a25e7d369c1a7d3aa2b2d68ef4e0d4` |
| guarded-reuse / 08:35:40 | ID1→1, job cũ sau bind drop/stale_round; timer hiện tại nổ; hai result/return. | `324a987f878abc09b74bc2c5b395dff7f27d7e816b7952bf52be6879eb4e4864` |

Hai vòng đều actual95/104/121full129/99/197/196/153/clock, cùng8connections, một bom A mỗi vòng,
layout582 rỗng; serial A0/0, không item/kim. Payload118 thực echo owner/serial/hàng/cột/power
theo CP031, ID do stub chọn. 158 cũ được mã hóa lại theo seq hiện tại từng receiver; CRC/seq hợp lệ.
Không chứng minh replay nguyên wire vượt crypto. `unique` và `reuse` chỉ đổi allocator ID;
giữ cùng harness logic/GDB/fixture/input và thời gian thử. `guarded` thêm module design server.

Lượt guarded import [RoundFence](../../../server/round_fence.py), runtime ghi path/SHA nguồn.
`server-round_fence.py`/`server-test_round_fence.py` lưu đúng bytes đã dùng để giữ provenance khi code
server tiếp tục phát triển. `begin()` trên actual Start được chấp nhận; `register()` giữ ticket cho
timer; `consume()` trước phát158 trong packet lock; `finish()` khi policy chọn rank.
Control old158 trước bind cố ý bypass guard để đo consumer, không là ca fence thất bại.
Sau bind, old ticket generation1 bị loại ở generation2, `broadcast=[]`; timer current vẫn gửi/consumer chạy.
Module không thay deadline/C2S/UDP/chain/allocator wrap hoặc gói đã gửi.

`round-fence-tests.txt`: **6 behavioral tests pass**: old round/reused ID, duplicate/same-round
reallocation, finish/no-active-round, khác room-owner instance, active-ID collision và invalid-ID
không phá job hiện tại. Đây là kiểm design server, không oracle KR.

`parse_runtime.py` ghép từng callback158 với metadata gửi theo thứ tự TCP/seq mỗi receiver;
lookup ID và pointer kind0 phải khớp entity159 vòng đó. B37880 chỉ hit đường local A;
không bắt buộc nó hit remote B rồi sửa raw để pass. Remote B có full159/entity/lookup/kind0 riêng.
`parse_fields.py` kiểm actual secure state/coordinates/stat/checksum bằng native52A350;
clock hai vòng khớp các field đã đo và actorA1 cả hai trước175. Không suy toàn heap reset.
`native-packets.json`/`derived/` giữ actualC2S và selectedS2C nativeframer/getters/blob/CRCbad/partial;
S2C policy fixtures không là server gốc oracle.

Ảnh ghép đã đọc: unique12/reused10/guarded10. `blast-1.png` trong ca tái dùng ID là snapshot lần gửi
timer cuối (vòng2); ảnh tên đó bị thay trong lúc run, không dùng làm lịch sử hai lần nổ.
Ảnh tuần tự `screen-N`/GDB/capture lưu riêng; wire fixtures mới có round trong filename để tránh đè.
Ảnh ngay sau send có thể trước callback; trace và ảnh trễ HELP/result/phòng được ghép đối chiếu.
Không suy từ riêng ảnh, pointer lặp, inputreturn0 hoặc full cursor rằng gameplay đúng.

Recipe argv chính xác tại `<run>-recipe.json`, mọi rawfile/size/hash cuối run tại `<run>-hashes.json`.
**7416 kiểm cơ học pass**,3sealedruns/552nativepacketcases/888actualsecure snapshot/getterchecks;
19ranges/4artifact,6unit tests,32ảnh đọc thủ công. Counter kiểm không phải tiến độ gameplay.
Script/report/index/rawhash được pin trong [CP041manifest](../checkpoints/manifest.json).

Tái lập với label/prefix mới, controller tuần tự, không đè raw đã seal:

```bash
python3 local/research/codex/generation-r1/run_probe.py new-unique CA-r2-stage-social.exe 240 wine_probe_generation_unique.py local/research/codex/generation-r1/generation.gdb
python3 local/research/codex/generation-r1/seal_run.py new-unique
python3 local/research/codex/generation-r1/run_probe.py new-reuse CA-r2-stage-social.exe 240 wine_probe_generation_reuse.py local/research/codex/generation-r1/generation.gdb
python3 local/research/codex/generation-r1/run_probe.py new-guarded CA-r2-stage-social.exe 240 wine_probe_generation_guarded.py local/research/codex/generation-r1/generation.gdb
```

Allocator/fuse6s/replay delay/outcome/guard là emu_choice/design. Không phục hồi thuật toán
generation server gốc, không coi mọi packet có epoch. Pending159/item165 với index reuse,
timer abort/cancel/chain, ID wrap/concurrent và UDP/peerqueue/full auth/cache/codec/vanilla vẫn mở.
Không fullmatch/K5/K6a/LAN/Internet. Giữ CP025/039/040 raw như lịch sử, không mở rộng phạm vi cũ.
