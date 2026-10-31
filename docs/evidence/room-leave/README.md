# Rời phòng, chuyển chủ, guest EOF và loại sảnh — CP034

Baseline CP033/[PR41](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/41) merge
`fbf4dc5ada459578a76280cd7197d5e20698ee9d`. CP034 snapshot trước tích hợp; PR nhánh
`kre/leave-runtime` giữ trạng thái merge. Codex solo/tự kiểm, review độc lập
[waived](../checkpoints/review-waiver.json). [Contract duy nhất](../../PROTOCOL.md#room-leave),
[next/gate](../../RESEARCH.md#q-023)/[PLAN](../../PLAN.md). AUD010/CP032; audit kế CP036.

## Artifact và recipe

Raw/script/fixture/disassembly/report: `local/research/codex/leave-r1/`.
Mười controller tuần tự, lease trong namespace user/net/mount/PID chỉ loopback/không route outbound,
files-only NSS/stub local. A/B prefix/wineserver/display/game-dir riêng. Không thêm binary patch,
không ép game state; copy kế thừa CP020 import/API/AC/guard và initial global reset, không vanilla.
Source binary/pack/DB giữ nguyên.

| Artifact | SHA256 |
|---|---|
| Runtime A/B CA-r2-stage-social.exe | `e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef` |
| Static CA-r2-login-logout.exe | `660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3` |
| modO raw memory/base400000 | `a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5` |

RVA=VA−400000. `code-match.json`:15 vùng khớp bốn artifact; modO không PE-remap.
`stage-rtti.json` khóa type/vtable. Chỉ phạm vi đã pin, không toàn call graph.

| Run / UTC31/10/2026 | Kết quả và giới hạn |
|---|---|
| guest-escape / 2026-10-31T02:13:56Z | Escape không86/phòng giữ nguyên. |
| guest-back / 2026-10-31T02:24:00Z | Back86 rỗng; peerghost còn/leaverloading. |
| guest-leave / 2026-10-31T03:24:09Z | 107 xóa ô trênA; Bloading, probe trước/sau chưa có. |
| guest-return / 2026-10-31T03:27:13Z | 107 base6/6→rewind0→reread1; refresh vẫnloading. |
| host-return / 2026-10-31T03:30:36Z | 107 xóaA,110promoteB; Arefresh vẫnloading. |
| guest-disconnect / 2026-10-31T03:34:30Z | Kill prefixB→channelEOF→107A; no86/ID2→0. |
| guest-route / 2026-10-31T03:37:12Z | Framer0 nhưng actualappstageSquare/vf82A530. |
| guest-lobby1 / 2026-10-31T03:40:45Z | Đổi u8 thứba vẫngetter0/Square; candidatebịbác. |
| guest-kind1 / 2026-10-31T03:43:38Z | Đổi lastu32→1: getter/factory1/normalhandler và Bsảnh. |
| host-kind1 / 2026-10-31T03:46:49Z | Cùnglastu32→1: Avềsảnh, BslotAcleared/chief1. |

Argv pin ở `<run>-recipe.json`; mọi raw file/size/SHA ở `<run>-hashes.json`.
Tái lập với label/prefix mới, không ghi đè run đã seal hoặc chạy controller tranh lease:

```bash
python3 local/research/codex/leave-r1/run_probe.py new-kind1 CA-r2-stage-social.exe 135 wine_probe_room_guest_kind1.py local/research/codex/leave-r1/leave-kind.gdb
python3 local/research/codex/leave-r1/seal_run.py new-kind1
python3 local/research/codex/leave-r1/derive_roles.py new-kind1
```

## Đọc bằng chứng

- `run.json` ghi actual86/empty/CRC,107 cho peer,110promotion hoặc guest EOF sau kill riêng prefix;
  S2C dynamic seq liên tục theo recipient và filehash. Không có trận/input trong nhóm này.
- `derived/`: lọc role từ raw seal, replay mọi actual auth/channel C2S bằng native framer với CRCbad/partial.
  `native-actual.json`: getters body86/107/110 và negatives; allocation tổng hợp, không full game/authority.
- `guest-return/client.log` và các ca sau: base107 đọc6/6/errors0, rewind0, cuối1/6 intentional;
  slot manager ID→0. `guest-leave` có probe ít hơn; không backfill log cũ.
- `host-kind1/client-B.log`: ID1→0,110slot1/chiefgetter0→1; ảnh có Start.
  `guest-disconnect`: channel B EOF trước broadcast107A và no86; idle room, không active-match recovery.
- `guest-route/client-B.log`: framerreturn0/241 và91, appstage vtable1716740/vf82A530;
  `stage-rtti.json` xác nhận CLobbySquareStage. `guest-kind1`/`host-kind1`:getter1/factory1/
  normalctor/DA1F30/241/91 và ảnh sảnh. Timing/copy/profile này, không claimrefreshminimal.
- `channel14-lobby1.*` là candidate **thất bại**, u8 thứba/rowbase8 đoán sai;
  `channel14-kind1.*` sửa từ baseline chỉ lastu32 dựa13A1FB0 rowbase14/getter13A1CB0 manager+D4.
  Native blob oracle tiêu44/count1/inner33, dùng allocation tổng hợp và skip checksum/storage.
  Full-row oracle fault149D134 do context thiếu; giữ report lỗi, **không tính pass**.
- Đã đọc ảnh trong back-control4/remove-only2/host-return4/control4/positive4 qua các composite;
  ảnh ghép nằm local. So với slot/getter/dispatch logs, không dùng UI riêng lẻ để đóng gate.
  Raw còn nhiều ảnh định kỳ chưa được đếm là manual review.
- VN `boom-re/docs/lobby.md` §0/§18 giúp chọn lifecycle/variants; không port opcode/schema theo số.
  Dùng cached asm/Capstone/Unicorn/WineGDB; không lặp cold REA/IDA timeout CP024.
  Các read mở giữa instruction và nhãn stack/result sai có correction trong `measurement-notes.json`.

**9111 kiểm cơ học pass**,10 run sealed,54 ca native actual packets;15 ranges/bốn artifact.
`verification.json`/`runtime-summary.json` khóa kết quả; `measurement-notes.json` giữ giới hạn/correction.
Hash từng raw/recipe/report/script/artifact pin trong [manifest CP034](../checkpoints/manifest.json).
Checker kiểm hash/counter/link/timeline, không tự xác nhận ngữ nghĩa hoặc đóng gate.

Chỉ room idle và các biến thể đã nêu. Stub bầu chủ/removal/refresh là design; chưa rejoin/start sau
promotion/host abrupt disconnect/active-match cleanup, collision/rescue/generation/toàn reset;
full auth/cache/codec/vanilla/UDPgrammar/relay/NAT còn mở. Chưa full match/K5/K6a/LAN/Internet.
