# CP024 — echo bom, nguồn spawn và profile map KR

Contract duy nhất ở [PROTOCOL](../../PROTOCOL.md#bomb-position); tiến độ/gate ở [PLAN](../../PLAN.md),
bước kế ở [RESEARCH](../../RESEARCH.md#q-023). Baseline
`561acc5fbf036806c4cceea91e9355dba1c08f76`: CP023/PR31 đã merge.
Codex solo RE/runtime/tự kiểm, review độc lập **waived**. CP024 là snapshot trước tích hợp,
PR nhánh `kre/bomb-position-runtime` giữ trạng thái merge hiện tại.
[AUD007](../checkpoints/audits/AUD007.json) audit định kỳ CP024; audit kế **CP028**.

Raw `local/research/codex/bomb-position-r1/`; [manifest](../checkpoints/manifest.json) khóa SHA/size
script/fixture/oracle/log/ảnh/dependency. Mỗi `*-hashes.json` phủ output run sau final run.json.
Không sửa raw/completed checkpoint cũ hoặc đưa binary/disassembly/capture vào Git.

## Target, công cụ và recipe

Runtime `CA-r2-stage-social.exe` SHA256
`e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef`;
static `CA-r2-login-logout.exe` SHA256
`660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3`;
modO memory-layout/base400000 SHA256
`a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5`.
Không thêm patch. Import/API/guard chẩn đoán kế thừa [CP020](../start-reply/README.md).
`static-runtime-code-match.json` so13vùng code; VA/RVA=VA−400000.

REA mở đúng target qua IDA headless nhưng query B335C0 timeout450s, không có dossier; close hoàn tất.
`rea-route.json` ghi giới hạn. Dùng Capstone/PEfile, Wine GDB và Unicorn hiện có để đo; không đổi
cấu hình/DB nguồn hoặc dùng timeout làm kết luận RE. Các lượt mở thêm theo cạnh thực:
record118→159 wrappers; spawn getter→point→actor; map table đã capture→hai profile thử.

Năm run145s **tuần tự**, mỗi prefix/output riêng, namespace user/net/mount/PID chỉ lo/no route,
NSS files-only, peer loopback; lease giữ trong namespace. Game-dir/cache dùng chung, chưa cold/warm.
Không ép stage/104/timer hoặc gọi getter trong client bằng debugger; snapshot được đọc rồi replay native ngoài process.
`*-recipe.json` giữ argv, `run.json` giữ hash/isolation/capture/timeline. Tái tạo với tên mới:

```bash
python3 local/research/codex/bomb-position-r1/run_probe.py NEW-RUN CA-r2-stage-social.exe 145 wine_probe_bomb_exvillage.py local/research/codex/bomb-position-r1/spawn-track.gdb
python3 local/research/codex/bomb-position-r1/verify.py
```

VN `oc_e-game.md` §5a-7 chỉ hướng index spawn→point→actor; KR getter12418A0/EAF910/B3D170
và code map0/1101/278 đo riêng. `map-candidates.json` khóa snapshot1151×52 từ CP019;
không nhập index954 VN hoặc suy tên UI là asset thật.

## Nhóm run và ảnh

| Run | Kết quả / giới hạn |
|---|---|
| position-control | Hai vòng recipe CP023 giữ159fixed; record118+10/+14 là100/1 rồi100/3, callback159 vẫn full39. |
| position-dynamic | Echo actual118 kiểm CRC/flag0→159; hai vòng full39. Info100/1→100/3, bomb getter40/4039→120/4039. Bốn snapshot parent/interface sai không dùng. |
| spawn-track | Nối getter slot0/1→point map6,100/4,100→actual actor240/4039,160/4039;16snapshot hợp lệ/native. Hai vòng có result/return. |
| village-dynamic | Profile1101/Village00 đồng bộ241/238/117/121/196; spawn13,11/1,11, actor520/479,40/479. Actual118 và15911/11; hai vòng/HELP/result/return. Geometry có object giống UFO/boss giữa map, chưa chốt mode Battle thường. |
| exvillage-dynamic | Profile278/ExVillage01 đồng bộ, spawn3,3/3,9, actor120/159,120/399; actual118/1593/3, bom120/159. Hai vòng/HELP/result/return; guest vẫn tổng hợp. |

Ảnh đã đọc: `village-dynamic/input-after-space-8.png`/`second-after-space.png` geometry/actor;
`exvillage-dynamic/second-after-space.png` actor/bom trên map,
`screen-31.png` HELP, `screen-33.png` LOSE/PERFECT, `screen-36.png`/`screen-38.png` UI phòng ExVillage01.
`screen-34.png` đen lúc chuyển cảnh, không dùng làm proof return; callback/log và ảnh sau đối chiếu riêng.
Ảnh observation elapsed lấy trước screenshot/thao tác, không phải timestamp chính xác effect.

## Verification, correction và giới hạn

`verification.json`: **2788 kiểm /5run**, gồm hash mọi output/isolation/lease/copy/harness,
native replay mọi C→S và CRC sai/partial, **66ca framing/22fixture** (14profile map và8reply động),
10actual118/base22, native159/base39, **56getter52A350 trên snapshot thật**,13byte ranges,
spawn index/point và hai vòng callback đầy đủ mỗi run.
Script chỉ kiểm tính nhất quán đo được; code/ảnh/timeline được đọc thủ công cho audit.

`measurement-notes.json` giữ các correction: parent159 là AI interface, không phải base actor;
wrapper checksum fail không phải giá trị tọa độ; có window static sai alignment và vtable method khác
setter/getter; REA timeout không có pseudocode. Native52A350 kiểm lại manual decode độc lập.

100 là point của map Antarctic trước actor init; table có fields15/101, nhãn width/height vẫn có giới hạn.
Không gọi4039 là corruption hoặc bằng chứng spawn hợp lệ chỉ từ số đó. Profile map mới dùng name/code
và checksum nhất quán. Village00/ExVillage01 mỗi vòng blast→reported19 khoảng6.46–6.49s;
Antarctic vòng hai khoảng0.9s vẫn chưa giải thích nhân quả. Pointer có thể được tái dùng dù actor re-init.

Chỉ bom đầu mỗi vòng, serial candidate0/id1, branch flag0; chưa nhiều bom/allocator/dedup,
va chạm/item/pickup/chain/authority/reset đầy đủ, peer/hai client, LAN/Internet, full auth/cache/vanilla
hoặc gate mới. Mốc nghiệm thu mạng K6a/K6b/K6c được làm rõ trong PLAN, đều chưa đạt.
