# Tái sử dụng nhóm item và delta KR — CP045

Baseline CP044/PR52 merge `b947a9d62507a9aef9625cc2c4586198c53ddbf1`.
Codex solo/tự kiểm, review độc lập waived; không đóng gate. Nhánh `kre/item-families-udp`.
[Contract/bảng đối chiếu duy nhất](../../PROTOCOL.md#item-families), [phép thử tiếp](../../RESEARCH.md#q-023).
Raw/copy/script/report/disassembly/log/ảnh giữ ở `local/research/codex/item-families-r1/`.

Trước nhánh mới đã đọc theo câu hỏi: PROTOCOL/RESEARCH/CP031/CP037 evidence và cache;
boomkr-re ItemAttr/kr-data/game-spec-web; boom-re items/oc_e-game; boom-rs R153 và R169–185,
boom-re-l10n docs migration/decomp/item_kinds; archive2026-10-26 KNOWLEDGE/README và report149.
`reuse-item-question.json` giữ path/hash, mức chứng minh, phần reuse/phải đối chiếu KR, khoảng trống/
phép đo tối thiểu. Bảng335KR/269VN đã parse CP037 được dùng lại, không RE loader/receiver/kim lần nữa.

Nguồn chính vẫn là artifact/runtime cùng build. R169–185 của VN đã giải Roller tăng point/kẹp max,
base/max/current/movement; không gọi toàn bộ Rollerunknown dựa dòng454 cũ. R150 gọi E9E660 là itempath
là nhận định lịch sử bị CP030 sửa; R153 table/loader vẫn dùng. boomkr-re kr-data có address VN và
fuse từ emulator; game-spec-web là web hint, không proof00967. Archive tóm tắt không thay rawsource.
boom-re-l10n269 hàng khớp VN sau chuẩn hóa−1/FFFFFFFF ở ba tail; không nguồn hành vi độc lập.

Runtime copy A/B SHA `e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef`;
static copy `660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3`;
modO rawmemory/base400000 `a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5`.
KR_ItemAttr SHA `a9b55540c85c21de108e8720b4148b2030a231d907f3709dcb06f7ef728fcdba`;
VN_ItemAttr `a98883ce35f3a509719db637f662f2ec90da64dd3c1faa0c9f7eb22c0de7dd1d`.
RVA=VA−400000, PEcopies mapped, modOraw. Nguồn binary/pack/DB không sửa; không patch thêm hoặc
ép actor/game state. Controller dùng lease/user-net-mount-PIDnamespace/chỉlo/nooutbound/files-only NSS.

Phép đo tối thiểu: reuse CP042 two-round/hai-client124/165+ItemClaims, không chèn old165;
chỉ đổi layout/allowlist kind2 hoặc5, đọc actualsecurefield trước/sau165/cảhai và clock vòng2.
1191840/effect3→vf14/B48350: current iface+5C=actor+5E8, max iface+50=actor+5DC.
Năm ca emulation giữ native increment/getter/setter/clamp, mô hình hóa local gateAD5BA0=false;
không dùng emulation thay runtime hoặc quy đổi tốc độ KR. `native-roller-clamp.json` giữ kết quả.


| Run / UTC31/10/2026 | Kết quả hẹp | run.json SHA256 |
|---|---|---|
| roller-rounds / 10:29:58 |Hai view current5→6/max7, đầu vòng2 reset5 rồi nhặt lên6; hai result/return.| `ebca4ee3e490af84bea1aa3a51949b4b68a691fe90c547e964b2159ca352bb15` |
| maxpower-rounds / 12:17:41 |Hai view power1→7/max7, actual118/159power7; đầu vòng2 reset1 rồi nhặt lên7.| `7672f4b0832d06874ebaa9df566734968d8c82b7a4c3e9636fd7da83e52e0215` |

Mỗi run hai actualA104/B95/121full129/99/clock và actualA124/index17/kind tương ứng mỗi vòng;
16512/12/errors0 cả hai. Callback1191840 nhận effect3 hoặc6 rồi ba0. Snapshots secure12bytewrapper/
12bytedata kiểm checksum và native52A350, không suy stat từ ảnh/icon. Stat slot1/capacity/Fire hoặc
movement ngoài effect mục tiêu giữ trong phạm vi đã chụp. Rank/return chỉ sau actual17/19 và peer170;
actorAstate1 cả hai trước175, hai175302/inner292 và180full3. Outcome/fuse/catalog là emulator choice.

`item-crosswalk.json` tái dùng CP037items.json/tablehash,335KR/269VN; `l10n-table-delta.json` giữ
ba khác biệt biểu diễn−1/FFFFFFFF, chưa là khác biệt bytes/behavior.267hàng chung giốngfield/weight,
hai khác dữ liệu là ItemCpGold/Pocket; không dùng chung tên để suy cơ chế. Bốnselector đầu của
five-tail group1 được dispatch; fifth-tail không tự là selector thứ5. Không kiểm kê mỗi row như một cơ chế.

`verify.py`:5406 kiểm quan hệ cơ học pass,2sealedruns,384native packet cases/816secure getterchecks;
`code-match.json`10ranges/4artifact.16ảnh đã đọc (mỗi run8), giữ sourcepath/hash ở visual-index.
Số kiểm không là tiến độ gameplay. `native_packets.py`/derived kiểm actualC2S theo socket/seq,
CRCbad/partial; synthetic receiver/allocation không là oracle server gốc. `parse_runtime.py` ghép165
với securefields và effectdispatch; không tự đóng gate. Rawfile/size/SHA cuối run ở `<run>-hashes.json`.

Tái lập tuần tự với label/prefix mới, giữ cùng isolation và không đè sealedrun:

```bash
python3 local/research/codex/item-families-r1/run_probe.py new-roller CA-r2-stage-social.exe 250 wine_probe_item_family_2.py local/research/codex/item-families-r1/family.gdb
python3 local/research/codex/item-families-r1/seal_run.py new-roller
python3 local/research/codex/item-families-r1/run_probe.py new-maxpower CA-r2-stage-social.exe 250 wine_probe_item_family_5.py local/research/codex/item-families-r1/family.gdb
```

Không đo TTL/expiration, runtime tại max, activevariants/inventory/resettoàntúi hoặc speed→px/s KR.
Không nâng ba loại cũ thành toàn kiến thức; VN R169–185 dùng được trực tiếp cho luật chung có giới hạn,
KR wire/offset/cap/profile phải kiểm riêng. Giữ AUD014/CP044, audit kếCP048; không fullmatch/K6a/LAN/Internet.
