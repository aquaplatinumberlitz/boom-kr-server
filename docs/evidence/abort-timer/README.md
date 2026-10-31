# Timer đang chờ khi abort và client thay thế — CP044

Baseline CP043/[PR51](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/51) merge
`f8569d8822b62fb2f917f00fb415b3c4e3771cdb`. CP044 snapshot trước tích hợp trên nhánh
`kre/abort-timer-runtime`; PR của nhánh giữ trạng thái tích hợp. Codex tự kiểm;
review độc lập [waived](../checkpoints/review-waiver.json).
[Contract duy nhất](../../PROTOCOL.md#abort-timer), [bước tiếp/gate](../../RESEARCH.md#q-023).
Audit định kỳ [AUD014/CP044](../checkpoints/audits/AUD014.json), audit kế CP048; không gate mới đóng.

Raw/recipe/harness/GDB/capture/log/native reports/ảnh: `local/research/codex/abort-timer-r1/`.
Controller tuần tự với lease trong namespace user/net/mount/PID, chỉ loopback/no outbound/files-only NSS.
Mỗi client prefix/wineserver/display/game-dir riêng. Binary/pack/DB nguồn bất biến;
không thêm patch hoặc ép actor/game state. Copy kế thừa CP020 imports/API/AC/guard và initial
reset global đã ghi recipe, không vanilla Windows.

| Artifact | SHA256 |
|---|---|
| Runtime A/B/C CA-r2-stage-social.exe | `e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef` |
| Static CA-r2-login-logout.exe | `660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3` |
| modO raw memory/base400000 | `a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5` |

RVA=VA−400000; PEcopies mapped theo section, modO raw memory. `code-match.json`19vùng/5artifact
kể cả app dispatch/159/158/lookup/result. Dùng WineGDB/Unicorn và cache cũ; không cold REA/IDA
hoặc quét toàn client. Byte equality không proof runtime.

| Run / UTC31/10/2026 | Kết quả | run.json SHA256 |
|---|---|---|
| uncancelled-room / 09:59:34 |B về phòng/chủ, old158 tới preparestage, không game158/kind0.| `018f10e740440b05563ec05428940c4ba8e3c78d921fe67e37c8f93c019b86a6` |
| fenced-room / 10:03:18 |Oldtimer drop/round_closed, không158; B về phòng/chủ.| `c0094fd72c737304207b12225742a9780b0ef2b2cd4a3a28d8acfb878bb20957` |
| fenced-replacement / 10:07:03 |Oldtimer drop; C mới vào, B Start/bom/trap/death, B/C result/return.| `25b0d0ccb5cbc6a3c725abc042ca52142fa2ea79b7dd815e005a8eb40d036948` |

B actual118 sau clock, owner1/serial1, ID1 do stub cấp;159 full39/entity cả hai. Sau khoảng2s kể từ
server record, controller kill chỉ prefix A, quan sát actual EOF đúng channel rồi finish fence.
Policy175/180/107/110 chỉ gửi B; không giả17/19. Oldfuse40s do thí nghiệm chọn, không fuse gốc.
Control vẫn finish nhưng cố ý bỏ qua rejection khi gửi; fenced dùng ticket gốc/consume trong lock.
Control old158 tới sau roomcleanup14,232s, app stage vt1767A80/PREPARE_RECV158; không hit E5F450
hoặc kind0. Không gọi đó là crash, game mutation, hoặc runtime full13reader.

`fenced-replacement` tái dùng fixtures C của CP040, khởi động C sau roomcleanup; actual ID3/slot0,
82/87full7/95, B chief gửi104 sau một retry hữu hạn. Không quy nguyên nhân click đầu cho focus/
timing/guard. B/C121full129/99/clock rồi input movement và bom mới từ B; ID2/owner1/serial1,
fuse6s,159full39/entity và158full13/lookup/kind0/cleanup khớp entity mỗi phía.
Actual B17/19→170 gửi C, runtime10/21byte/errors0; secure actorB state1 cả hai trước175.
Chờ actual C21 rồi chọn slot0win/slot1lose;175outer302/inner292,180full3 và110 refresh đúng Bchief/
Cunready. Ảnh rank highlight C/KRLocal/PERFECT và B/KRGuest/LOSE; ảnh cuối cả hai ở phòng.
Một forfeit và một vòng B/C đã chơi, **không hai trận hoàn chỉnh hoặc A reconnect/resume**.

`server-round_fence.py` lưu exact bytes module đã dùng; module server không thay đổi trong lượt này.
Caller finish tại EOF trong packet lock; scheduled entry vẫn giữ đến deadline rồi kiểm ticket/drop,
không gọi đây là xóa scheduler entry. `active_game_peers` loại EOF A khỏi broadcasts; harness CP040
chỉ đo movement, không coi nó vốn đã kiểm các đường bom/result mới. Không thêm wire epoch.

`derive_roles.py`/`replay-run.py` native14EF300 kiểm mọi actual C2S theo từng socket/seq40+3n,
CRCbad/partial negatives. `native-packets.json` giữ typed/blob/checksum của selected actual118/127/95/82
và S2C159/158/170/175/180/582/107/110/106. 87 kiểm full7 ở runtime; native106 chỉ prefix5byte
slot/ID, không full profile.
Framer/getters giữ code KR, allocation/receiver model; S2C fixtures do stub chọn không là oracle server gốc.
`parse_fields.py` giải actual secure actor state/coordinates/stat với checksum và native52A350.
Abort actorB trước175 vẫn0, vòng B/C mới là1 cả hai; không dùng forfeit làm proofdeath.
`parse_runtime.py` tách app dispatch/prepare158 với game158 và client-local counter:
C first Start=1 tương ứng roomgeneration2, B có haiStart. Seq từng socket liên tục; không postEOF sendA.
Metadata `bootstrap_reply_limit=8` là nhãn kế thừa; collector thực dùng `reply_claimed<12` và run
replacement có12connection/4mỗi role. Dùng actual tcp_events/role và harnessSHA, giữ raw nhãn cũ.

29ảnh nguồn đã đọc: control6/fenced5/replacement18; `visual-index.json` giữ source path/hash.
Ảnh gần mốc roomcleanup ở replacement có thể còn roster cũ vì controller chặn trong Cwineboot;
nearest-screen không bảo đảm sau callback. Dùng trace107 ID1→0/chiefB1 và ảnh cuối, phân biệt C ID3
với tên KRLocal trùng A. Ảnh ngay sau send/blast không tự chứng minh đúng frame effect.

**5741 kiểm cơ học pass**,3sealedruns/213nativepacketcases/310actualsecure getterchecks,
19ranges/5artifact,12policy regressiontests và sourceartifacts5/5. Counter không phải tiến độ gameplay.
Helper/report/index/rawhash pin trong [CP044 manifest](../checkpoints/manifest.json).
Rawfile/size/SHA tại `<run>-hashes.json`; argv chính xác tại `<run>-recipe.json`.

Tái lập bằng label/prefix mới, từng controller tuần tự; không đè sealedrun:

```bash
python3 local/research/codex/abort-timer-r1/run_probe.py new-control CA-r2-stage-social.exe 190 wine_probe_abort_timer_control.py local/research/codex/abort-timer-r1/timer.gdb
python3 local/research/codex/abort-timer-r1/seal_run.py new-control
python3 local/research/codex/abort-timer-r1/run_probe.py new-fenced CA-r2-stage-social.exe 190 wine_probe_abort_timer_fenced.py local/research/codex/abort-timer-r1/timer.gdb
python3 local/research/codex/abort-timer-r1/run_probe.py new-replacement CA-r2-stage-social.exe 320 wine_probe_abort_timer_replacement.py local/research/codex/abort-timer-r1/timer.gdb
```

AUD014 đọc cả core docs theo thứ tự, evidence CP040–043 và reports/raw mới, sửa bảng tiến độ/Q026
stale và ghi PR51 merge. Giữ CP042 collector failure/CP043 intentional disruption đúng lịch sử;
không đổi checkpoint cũ. KếCP048 theo counter. Full auth/cache/codec/server độc lập/vanilla/account
reconnect/validator/UDPqueue/LAN/Internet và K5/K6a vẫn mở.
