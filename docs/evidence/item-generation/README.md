# Old165/index reuse qua vòng và claim policy — CP042

Baseline CP041/[PR49](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/49) merge
`97320ca02115a6de538aa0f38b7a12fed971aa4c`. CP042 snapshot trước tích hợp trên nhánh
`kre/item-generation-runtime`; PR của nhánh giữ trạng thái tích hợp.
Codex tự kiểm, review độc lập [waived](../checkpoints/review-waiver.json).
[Contract duy nhất](../../PROTOCOL.md#item-generation), [bước tiếp/gate](../../RESEARCH.md#q-023).
AUD013/CP040 gần nhất, audit định kỳ kế CP044; không gate mới đóng.

Raw/recipe/harness/GDB/fixtures/native reports/log/capture/ảnh:
`local/research/codex/item-generation-r1/`. Controller tuần tự với lease trong namespace
user/net/mount/PID, loopback/no outbound/files-only NSS/local stub. A/B prefix/wineserver/display/
game-dir riêng; source binary/pack/DB bất biến. Không thêm patch/ép actor/stat bằng debugger;
copy kế thừa CP020 import/API/AC/guard và initialglobalreset, không vanilla Windows.

| Artifact | SHA256 |
|---|---|
| Runtime A/B CA-r2-stage-social.exe | `e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef` |
| Static CA-r2-login-logout.exe | `660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3` |
| modO raw memory/base400000 | `a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5` |

RVA=VA−400000; modO raw, PEcopies mapped theo section. `code-match.json`23vùng/4artifact.
Dùng nativecache/Unicorn/WineGDB, không cold REA/IDA hoặc quét toàn client. E63260/E2C660/E14DA0
và stat probes đọc lại từ evidence CP030/031; không đổi schema theo VN.

VN `boom-re/docs/oc_e-game.md` dòng444–455 và738–743 hướng phép đo pending→effect/peer.
`vn-reference.json` giữ path/hash/excerpt/limits. VN có opcode117→158/indexu16 và ghi rõ server
ACK mọi117/chưa kiểm tranh cùng index; không áp cách đó làm validator KR124→165/indexu32.
Các claim KR dưới đây dựa capture/callback/stat native của chính build KR.

Mỗi run hai vòng giữ8connections/identity/slot/session, actual95/104/121full129/99/clock;
582 một Bomb/kind0 hàng4/cột3. Vòng1 index17; vòng2 index18 ở control hoặc17 ở reuse/guarded.
Sau clock vòng2, trước input nhặt, replay payload165 vòng1 với **seq receiver hiện tại**.
Không replay nguyên frame vượt seq/CRC hoặc giả actual124. Native5826cases kiểm hai fixture;
catalog/drop/index/placement/fuse6s/outcome Alose/Bwin là emu_choice/design.

`unique-index`: old165 lookup item17=0, đọc12/12/errors0, capacityA1→1 cả hai view;
sau input có actual124/index18, current165/capacity1→2. `reused-index`: old165 tìm item17 mới,
effect/increment/capacityA1→2 trên cả A/B trước input; toàn capture chỉ một124 ở vòng1,
không124 ở vòng2. Đây là crossround semantic reuse, khác duplicate sau item đã xóa cùng vòng CP031.
Không suy client bắt buộc pending124 trước khi chấp nhận positive165 của variant này.

| Run / UTC31/10/2026 | Kết quả | run.json SHA256 |
|---|---|---|
| unique-index / 08:54:07 |Đổi index18:old165 không cộng; actual12418/current165 mới cộng. | `155f3418cff0ed788ef521b83b491504af13c571bde389025b0ddacf35d260a4` |
| reused-index / 08:58:57 |Index17:old165 cộng1→2 trước input, không124 vòng2. | `da2a10b32a7a38360104d6808cb3965a66fd13a0f666647397489973869d1c23` |
| guarded-index / 09:04:22 |Collector KeyError trước165; không consumer/policy proof. | `e8885a961b099c950fb8fc5182a539f54bb923cfac02d6ff5dd358145b05be2e` |
| guarded-fixed / 09:10:22 |Oldticket drop; actual12417/current165 mới cộng; đủ hai result/return. | `688f0bc7c716aee3ad54c2405aa43d3a762465055f5eb0dff5cfa803012647e3` |

[ItemClaims](../../../server/item_claims.py) là design server: catalog hiện tại + slot gắn socket,
context/index/hàng/cột/kind khớp mới giữ claim đầu, ticket xác nhận local gắn owner/generation
và consume một lần. Begin trên actualStart, finish khi policy chọn rank; claim/consume/send165
trong packet lock. Exact bytes module/tests đã dùng được archive `server-item_claims.py`/
`server-test_item_claims.py`; runtime ghi sourceSHA để không mất provenance khi code tiếp tục đổi.
Module không phân biệt được124 cũ có toàn field trùng catalog hiện tại, không kiểm distance/tick/
sessionkey/actorstate hoặc UDP; không gọi đây là toàn validator. Unit tests có case xác nhận giới hạn đó.

`guarded-index` là **collector lỗi**, không consumer165 failure: sau consume claim mới còn dòng truy cập
bảng legacy `claimed_items` đã bị thay, gây KeyError lúc actual124 đầu. Channel đóng, connection thêm
không được gán role lại gây KeyError thứ hai. Chưa165 nào được xử lý; giữ raw/harness lỗi nguyên.
`guarded_fixed` dùng bản harness mới chỉ bỏ truy cập legacy thừa, đo lại bằng label/prefix mới.
`guarded-fixed` hoàn tất: `stale_confirms` ghi `drop/stale_round/broadcast=[]`; sau input A có
actual124/index17 vòng2,165 hiện tại12/12/errors0/capacity1→2 cả A/B. Hai175/180/phòng vẫn đủ.
Đã đọc30ảnh ghép (unique10/reuse10/fixed10); không dựa ảnh tức thời để xác nhận stat.
**9129 kiểm cơ học pass**,4sealedruns gồm1failedcollector,597nativepacketcases/1140actualsecure
snapshot/getterchecks,6layoutnativecases/23ranges4artifacts và12unit tests (6mới+6RoundFence).
Số kiểm không phải tiến độ gameplay; checker không tự suy semantics hoặc đóng gate.

`parse_runtime.py` ghép165 theo thứ tự gửi/callback từng receiver; trước/sau mỗi callback có secure
actor+4A4/+50C/+4BC/+524, checksum/native52A350/sourcehash qua `parse_fields.py`.
Effect/increment có thể xuất hiện trước nhãn outer FOUND/MATCHED vì interface gọi lồng;
thứ tự tên probe không thay call graph. Probe `GEN_PENDING_REMOVE` chỉ hit cạnh xóa record thực,
không thấy nó không tự chứng minh toàn pending table rỗng; capture124 là nguồn riêng.
`derived/` replay mọi actualC2S/framer/CRCbad/partial; `native-packets.json` kiểm typed fields/blob
của actual124/127/95 và selected165/582/159/158/170/175/180. S2Cfixture không oracle server gốc.

Mỗi vòng một bomA→17/19 và170 đưa peerstate1 trước175,175outer302/inner292,180full3 về phòng.
Clock hai vòng reset stat/coordinates/state trong các field đã đo; UDPport/serialA0 giữ theo ca.
Không suy toàn entity/timer/peerqueue/generation an toàn. Ảnh ngay sau send có thể trước callback;
ghép ảnh trễ/trace/stat, không dùng icon biến mất hay full cursor làm proof stat tăng.

Recipe chính xác ở `<run>-recipe.json`, rawfile/size/SHA cuối run tại `<run>-hashes.json`;
checkpoint pin các index, reports và helper source. Không ghi binary/decompile/raw vào Git.
Tái lập bằng label/prefix mới, controller tuần tự, không đè sealedrun:

```bash
python3 local/research/codex/item-generation-r1/run_probe.py new-unique CA-r2-stage-social.exe 250 wine_probe_item_generation_unique.py local/research/codex/item-generation-r1/items.gdb
python3 local/research/codex/item-generation-r1/seal_run.py new-unique
python3 local/research/codex/item-generation-r1/run_probe.py new-reuse CA-r2-stage-social.exe 250 wine_probe_item_generation_reuse.py local/research/codex/item-generation-r1/items.gdb
python3 local/research/codex/item-generation-r1/run_probe.py new-guarded CA-r2-stage-social.exe 250 wine_probe_item_generation_guarded_fixed.py local/research/codex/item-generation-r1/items.gdb
```

Phạm vi còn thiếu: identical-old124/fullvalidator, pending159 tới trễ, timer abort/cancel/chain,
allocator/wrap/concurrent, UDPgrammar/peerqueue/fullreset/fullauth/cache/codec/vanilla/NAT/relay.
Không fullmatch/K5/K6a/LAN/Internet. Giữ raw CP030/031/032/041 nguyên làm lịch sử;
không sửa claim cũ thành đã kiểm qua vòng. Counter/audit ở [manifest](../checkpoints/manifest.json).
