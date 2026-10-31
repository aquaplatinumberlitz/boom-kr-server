# Rejoin và Start sau chuyển chủ — CP035

Baseline CP034/[PR42](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/42) merge
`0779bc71b6a23d9262e3d4bec522ef18b3681b5c`. CP035 snapshot trước tích hợp;
PR nhánh `kre/rejoin-runtime` giữ trạng thái merge. Review độc lập [waived](../checkpoints/review-waiver.json),
Codex tự kiểm. [Contract duy nhất](../../PROTOCOL.md#room-rejoin), [next/gate](../../RESEARCH.md#q-023).
AUD010/CP032 gần nhất; audit kế CP036. Không gate mới đóng.

Raw/script/fixtures/report ở `local/research/codex/rejoin-r1/`. Hai controller tuần tự giữ lease trong
namespace user/net/mount/PID, chỉ lo/không route outbound/files-only NSS/stub local. A/B prefix,
wineserver/display/game-dir riêng. Không thêm binary patch/ép game state; diagnostic copy kế thừa
CP020 import/API/AC/guard và initial global reset, không vanilla. Source binary/pack/DB giữ nguyên.

| Artifact | SHA256 |
|---|---|
| Runtime A/B CA-r2-stage-social.exe | `e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef` |
| Static CA-r2-login-logout.exe | `660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3` |
| modO raw/base400000 | `a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5` |

RVA=VA−400000; modO không PE-remap.15 vùng code khớp4 artifact; không thân hàm mới.

| Run / bắt đầu UTC31/10/2026 | Kết quả |
|---|---|
| guest-rejoin / 2026-10-31T03:52:14Z | B rời/vềsảnh/click82→87 lần2→585/106/110/238/117; hai profile/phòng, cùng sockets. |
| promoted-rejoin / 2026-10-31T03:55:51Z | A rời/Bchủ/A vào lại làkhách; ReadyA95→StartB104→121full cảhai/99/clock. |

Recipe argv ở `<run>-recipe.json`, mọi rawfile/size/SHA ở `<run>-hashes.json`.
Tái lập bằng label/prefix mới, không ghi đè run đã seal hoặc tranh runtime lease:

```bash
python3 local/research/codex/rejoin-r1/run_probe.py new-promoted CA-r2-stage-social.exe 190 wine_probe_room_promoted_rejoin.py local/research/codex/rejoin-r1/promoted.gdb
python3 local/research/codex/rejoin-r1/seal_run.py new-promoted
python3 local/research/codex/rejoin-r1/derive_roles.py new-promoted
```

`run.json` khóa actual82/585/95/104, dynamic recipientseq/fixturehash/peerendpoint;
`client*.log` khóa87/7byte, own-match/chief,121/129byte/stage13/clock. `derived/` replay mọi
actual auth/channel C2S bằng native framer/CRCbad/partial; `native-actual.json` thêm typed82/86/107/110; `native-fields.json` đọc đủ87 và95
(blob context +u8desired1 +u32tail0), không dùng schema95/u8 đã thử sai trong helper ban đầu. Allocation tổng hợp; không fullgame/authority. `promoted-fixtures.json` giữ candidate
lúc chuẩn bị, kết quả runtime nằm contract/report. `rejoin-visual.png`: đã đọc6 ảnh profile/role/
clock của hai phía, đối chiếu input/capture/reader logs. `measurement-notes.json` giữ giới hạn.

**2605 kiểm cơ học pass**,2 sealedruns,48 nativeactual cases;
15ranges/fourartifacts match. Hash report/script/artifact/raw index pin trong
[CP035 manifest](../checkpoints/manifest.json); checker không suy ngữ nghĩa hoặc đóng gate.

Rejoin cùng ID/slot/session/socket, chưa reconnect/ID reuse/đổi slot/password/bounds. Ca chủ mới tới
clock, không fullmatch/hai trận hoặc mọi cleanup/reset. Chính sách bầu chủ và state0/4/22 là
emu_choice/design; handler/read/role/start/clock thực thi có trace riêng. HostEOF/disconnect trong trận,
collision/rescue/generation/toàn reset/fullauth/cache/codec/vanilla/relay/NAT/K5/K6a/LAN/Internet còn mở.
