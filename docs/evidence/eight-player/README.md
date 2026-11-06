# CP147 — Roster8, capacity và nhánh cân bằng đội

[Contract duy nhất/giới hạn](../../PROTOCOL.md#battle-eight-roster),
[câu hỏi](../../RESEARCH.md#gap-roster-ready-host), [thứ tự](../../PLAN.md#current-order).
Baseline `c671301` gồm CP146 chưa merge; nhánh này giữ nguyên CP146 và mọi source/raw/history.
Owner/reviewer Codex, [review độc lập waived](../checkpoints/review-waiver.json); chưa đóng issue14/gate.

Nguồn đối chiếu trước nhánh: CP018106/110/Ready field ledger, CP019 local map dump/getterEDC890,
CP020121/slot26 để tránh suy roster manager thành giới hạn người; `boomkr-re/docs/game-spec-web.md`
§2–3 (nguồn web, không runtime KR), `boom-re/docs/oc_a-room.md` và nguồn lobby trong REFERENCES.
`reuse-question.json` pin source/build/SHA/locator, điều đã biết, phần dùng trực tiếp/phải kiểm KR và
phép đo tối thiểu. Không tìm lại codec/loader hoặc chuyển opcode/layout VN. Dump1151×52 đã đủ trả
capacity278; nhánh mới chỉ đo roster/Ready/Start và phần thiếu ở balance guard.

Raw ở `local/research/codex/eight-player-r1/`; recipe JSON từng run ghi command đầy đủ.
`prepare.py` tái dùng builder106 CP018 vào đường mới, `prepare_variants.py` chuẩn bị toàn bộ variant
trước chạy tuần tự; **không đổi source/config giữa run**. `wine_probe_eight_*.py` giữ namespace recipe
CP019: user/net/mount/PID riêng, chỉ lo/no route, NSS files-only, stub loopback, prefix và game copy
riêng. Mỗi run khởi động một CA thật; slot còn lại tổng hợp. Không dùng production probe_service
hoặc thay state manager/count/Start return qua debugger. GDB giữ init suppression1A966FC=0 của
recipe CP019, vì vậy đây vẫn là copy chẩn đoán; không coi là Windows nguyên bản.

| Run | Kết quả và thời gian có cả setup/cleanup | run.json SHA256 |
|---|---|---|
| `four-four` | 76.004s; check3; actual104=0 | `47f60f0c5951c8f44126a73eeb5f09a74702e0e777f2ef9ea5cfb33ea39b53ed` |
| `ready-four-four` | 75.373s; check0; actual104=1 | `f4d5a5cf10d144e6d8fcc173491d4925d6c5936782d362bbdd18e1a51b299a1f` |
| `ready-one-seven` | 75.376s; check0; actual104=1 | `a0ec0dbebd71beef3d9f3fde9a31adfe156f6833717d45eeb93e2b674943dbe1` |
| `ready-nine` | 75.669s; check5; actual104=0 | `35296b220acb3f9f4a2161e36369f219c6616aaa2037ff51da25bfacd5a32d97` |

Control `four-four` dùng guest flags2 (sai vai trò Ready): callback đọc đủ8 profile nhưng click trả3,
notice yêu cầu Ready, không104. Sửa **input** guest flags4 từ ledger cũ và chạy prefix/game mới;
không sửa log/cấu hình control. Ba run mới đều parser106 cursor196/196/errors0, callback xong từng
slot; tên KR0..KR7 (ca9 thêmKR8) là dữ liệu thiết kế. Ảnh `ready-four-four/run/screen-18.png`
cho tám ô/4đỏ4vàng, host và bảy Ready. Cơ chế/giới hạn không suy riêng từ ảnh.

`verification.json`: **527 joins** cho isolation, thứ tự run đã end trước run kế, SHA executable/harness/
commands, slot/ID/cursor/callback, actual117/238, count/capacity và kết quả **click**. Log còn các check
capacity theo tick/trước table đến; không gộp chúng với kết quả click sau117. Captured104 từ hai ca8
được replay bằng native14EF300 và CRC-sai/partial controls; encoder tự sinh chỉ là fixture. `replay_capture.py`
tái dùng CP019 receiver model; allocator/prefix-off/seq40+3n được khai rõ, không là oracle full auth.

`verify_balance.py`/`balance-emulation.json`:14 ca native blockDCB12C–DCB191, getter123FA10 giữ native;
chỉ singleton544520 và count-index657960 mô hình. `start-guard-instructions.json` và `room-getters.json`
giữ static window/hash/VA (base400000; RVA=VA−400000), không gọi bounded window là toàn hàm.
Bytes guard/getter được đối chiếu nguyên với modO/static. REA `analyze_function(DCAD80)` timeout450s,
`rea-timeout.json` giữ lỗi; không báo có dossier thành công hoặc bỏ mất phép đo runtime để chờ tool.

**Hash/build:** sourceCA `a52545c1…` bất biến; runtime `CA-r2-room-keystate.exe`
`94ee93dc7978a1ab22f25ec877e29ddcab254ca761f2eac22534421e2dd628e8` dùng lại CP018/019;
static copy `660447c2…`, modO `a08950c8…`, map dump `49727b4d…` đều có locator/hash đầy đủ ở
[manifest evidence CP147](../checkpoints/evidence/CP147-eight-player.json).
`raw-seal.json` hash **296 output files** sau `run.json`/end/cleanup; game/prefix loại khỏi raw index,
executable pin riêng. Cleanup wineserver đúng prefix rồi process groups và exit namespace của recipe;
không process của run còn lại. Không binary/asset/decompile được đưa vào Git.

Chưa8 client thật,121/load/Battle8, UDP mesh/room-full service/reset8, tên/input selector1,
Manner1–7 negative runtime, hay Mission/Respawn. Service roleA/B/C và slot cố định vẫn là blocker
mở rộng; chưa port luật web hoặc nhận8 synthetic rows thành integration. Kiểm requiredlocal/history,
docs/catalog/issues và diff ghi tại `checks/`; semantic/gate do người đối chiếu, script không tự nâng.
AUD044 qua144, audit định kỳ kế **CP148**. Bước tiếp theo duy nhất ở PLAN; không mở plan theo phiên.
