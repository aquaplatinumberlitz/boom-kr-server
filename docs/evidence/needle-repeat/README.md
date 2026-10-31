# Hết kim ở trap kế và đối chứng đổi team — CP038

Baseline [CP037/PR45](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/45) merge
`1e2ac9a26c5c1e036ed02faaf9a3bc043f72a7bb`. CP038 là snapshot trước tích hợp;
PR nhánh `kre/contact-inventory-next` là nguồn trạng thái tích hợp.
Codex solo/tự kiểm, review độc lập [waived](../checkpoints/review-waiver.json).
[Contract kim](../../PROTOCOL.md#needle-repeat), [va chạm](../../PROTOCOL.md#contact-precedence),
[thứ tự công việc](../../RESEARCH.md#q-023). AUD012 đã hoàn tất; audit định kỳ kế CP040.

Raw/script/fixture/native report/log/ảnh ở `local/research/codex/needle-repeat-r1/`
và `local/research/codex/contact-symmetry-r1/`. Các controller tuần tự, lease trong namespace
user/net/mount/PID, chỉlo/no outbound/files-only NSS/local stub. Mỗi client prefix/wineserver/display/
game-dir riêng. Không thêm patch hoặc ép state. Copy kế thừa CP020 import/API/AC/guard và initialglobalreset,
không vanilla Windows; binary/pack/DB nguồn giữ nguyên.

| Artifact | SHA256 |
|---|---|
| Runtime A/B/C | `e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef` |
| Static CA-r2-login-logout.exe | `660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3` |
| modO memory/base400000 | `a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5` |

RVA=VA−400000; modO raw, không PE-remap. `needle-repeat-r1/code-match.json` kiểm11 vùng/4artifact;
`contact-symmetry-r1/code-match.json` kiểm12 vùng/5artifact kể cả C.
Cạnh item-use đi từ callsite runtime CP037 B4D999→1190C20;
read1190F00/B4D600 mở giữa instruction bị loại, entry1190C20 đọc lại đúng. Không suy từ prefix sai.
`native-use-gate.json` có6 ca native1190C20/countguard, itemtable/group23/interface callback model;
amount tổng hợp, chưa capture count argument của Ctrl rỗng. Không coi đây là server inventory validator.

`repeat-ctrl` có một Start/clock, một pickup145, hai bom cùng vị trí trong cùng trận.
`repeat-visual.png` đã đọc6ảnh: ô trống trước bom2, HELP trước/sau Ctrl2 trên A/B, cuốiA biến mất trên cả hai.
Ảnh cuối ghép native state1/actual19; không gọi chỉ ảnh biến mất là death proof.

Đối chứng `swapped-symmetric` **không tới Start**: A initialguest106 được gửi bằng fixtureseq58,
nhưng stream cần64; A chưa map, không three-roster/Ready/121/99/clock/contact.
RooterrorNone không làm nó pass. `native-guest-seq.json` dùng native14EF300 với key64:
fixture58 trả7/CRC, retime64 trả0; body decode giữ nguyên/hash tại `guest-seq-fix.json`.
`swapped-fixed` dùng harness/fixture mới; raw/harness lỗi giữ nguyên.
Các state106/110 và team121 hoán đổi B1/C0, A0; IDs/slots không đổi.
Native121 inner126/outer155 kiểm độc lập trong `swapped121-native-fields.json`.
Input cùng hướng/giờ không phải proof cùng lượt va chạm hợp lệ; movement profile cũng khác CP037, không là can thiệp chỉ một biến team.
Ảnh `swapped-visual.png` đã đọc6ảnh trước/sau input; có thể còn HELP trong animation/GDB/callback, không dùng riêng ảnh tức thời phủ nhận trace death sau đó.

Recipe argv ở `<run>-recipe.json`, mọi rawfile/size/SHA tại `<run>-hashes.json`.
`derived/` native replay mọi actualC2S với CRCbad/partial; `native-packets.json` giữ getter/blob/checksum
cho actual118/124/127 và S2C159/158/165/170 theo scope từng root. S2C là lựa chọn stub, không oracle server gốc.
`actor-fields.json` là secure actor+28/+4C/+58 thực/checksum/native52A350 và sourcehash.
`runtime-summary.json`/`verification.json` chỉ kiểm khai báo cơ học; semantic/ảnh đọc riêng.

Tái lập dùng label/prefix mới, giữ controller tuần tự và không ghi đè run đã seal:

```bash
python3 local/research/codex/needle-repeat-r1/run_probe.py new-repeat CA-r2-stage-social.exe 185 wine_probe_needle_repeat.py local/research/codex/needle-repeat-r1/repeat.gdb
python3 local/research/codex/needle-repeat-r1/seal_run.py new-repeat
python3 local/research/codex/contact-symmetry-r1/run_probe.py new-swap CA-r2-stage-social.exe 205 wine_probe_contact_symmetry_swapped_fixed.py local/research/codex/contact-symmetry-r1/contact.gdb
python3 local/research/codex/contact-symmetry-r1/seal_run.py new-swap
```

Quyết định ưu tiên của Codex khi chủ dự án hỏi có cần ca simultaneous runtime: giữ nó làm kiểm bổ sung
độ khớp client gốc, không biến thành gate riêng hoặc săn timing kéo dài. Native first-eligible là cơ sở
policy thử có provenance; chưa khẳng định thứ tự cố định mọi mode hoặc cả hai cùng đủ điều kiện runtime.
Sau nhóm này ưu tiên report21/result→return180 và reset/disconnect cho ca trận đầy đủ.
Unknown chưa được coi solved chỉ vì chuyển ưu tiên. K5/K6a/LAN/Internet/full auth/cache/codec/validator/
server inventory/duplicate/stale/generation/toàn reset còn mở. Timing/fuse/placement/ACK/translation là emu_choice/design.

| Run / UTC31/10 | Kết quả | run.json SHA256 |
|---|---|---|
| repeat-ctrl / 2026-10-31T06:45:59Z | Một145/hai trap cùng trận;Ctrl2ôtrống không tựthoát,19sau6,150s. | `c28a7265ee73a42b192867ee77892bfa57cbdddcbabf99b26d9bc383cc000820` |
| swapped-symmetric / 2026-10-31T06:49:22Z | Sai seq Aguest106; chưaStart, không contactproof. | `bed2afeb64d9fb80a23092942ffed290f332a1b535351eb26302fdadb85be004` |
| swapped-fixed / 2026-10-31T06:53:45Z | Đổi team0/1/0;tick405Bpredicate0/deathslot1→19. | `f67a285a1a13abd62e81802d204617661b087e80bf27d909be43e540da87a3ee` |

Kiểm cơ học: repeat **2126** checks/222nativepacket/426actualsnapshot/6itemgatecases; contact **3442** checks/2sealedruns(1failedfixture-beforeStart)/39nativepacket/135actualsnapshot,2nativeframerseqcontrols và155byteStartfixture.
Checker kiểm khai báo/hash/callback/seq; một run lỗi vẫn được giữ là lỗi. Không suy tiến độ gameplay từ sốcheck.
Hash rawindex/native/script/ảnh pin trong [CP038](../checkpoints/manifest.json).
