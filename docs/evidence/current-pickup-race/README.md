# CP062 — Hai client tranh cùng item qua hai vòng

Baseline CP061/PR83 merge `c70a4e226d99ab623d9cc2ccba782d8a2f2ffb32`;
KRE-025/[issue84](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/84).
Codex solo/tự kiểm; [review waived](../checkpoints/review-waiver.json), tích hợp chờ PR.
[Contract và giới hạn](../../PROTOCOL.md#current-pickup-race),
[câu hỏi còn mở](../../RESEARCH.md#gap-contact-state), [thứ tự tiếp](../../PLAN.md#current-order).

Raw: `local/research/codex/current-pickup-race-r1/`. `reuse-question.json` pin nguồn/build/locator:
CP030 phương pháp giữ delivery165 để có hai native report; CP061 profile41/floor17/Roller/stat/reset;
ItemClaims và guard124 hiện hành; VN oc_e-game/items cho phương pháp report/confirm, không chép opcode/
layout. Nguồn boomkr-re/boom-rs/l10n/archive được đối chiếu. Tránh RE lại writer/consumer/auth/cache,
không thêm thân hàm native, patch hoặc sửa service production.

Run `race-current`, **2026-11-02T12:39:46Z**,360s; `race-current/run.json` SHA256
`06310de8be8ef90fd0cde9247119e0563bcf71925d1cd82b02115d122363d1a3`.
Runtime copy SHA `e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef`;
static copy `660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3`;
modO raw/base400000 `a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5`,
RVA=VA−400000, không PE-remap raw. Bản nguồn/pack/DB giữ nguyên.

Recipe: private A/B game copies CP059, prefix/display/wineserver riêng; một controller/runtimelease,
namespace user/net/mount/PID chỉ loopback, không route, NSS files-only. `runtime_hold.py` subclass chỉ
hoãn positive165 khoảng2s, claim/consume/key/reportguard giữ nguyên (`runtime-source/` pin exactcode).
`wine_probe_current_pickup_race.py` cho cả hai Down0.30s qua UI barrier clock+12s, chờ165 rồi BRight/
Aspace để kết thúc vòng. Không mint124/forcewinner; bounded space retries cài nhưng không dùng.

Bốn **actualTCP124**, cùngindex17/row4/col3/kind2. Vòng1/key12345 A tới trước, B bị
`claim_not_current_or_already_consumed`; vòng2/key12346 B tới trước, A bị cùng lý do. Mỗi vòng chỉ
một165/receiver đúng winner slot, native callback12/errors0. SecureRoller cảhai view: vòng1 A5→6/B4;
vòng2 baseline A5/B4 rồi A5/B5. Winner đảo theo actualarrival. Hold lần lượt2.002629184s và2.045519253s;
loser đã tới trước delivery1.916875127s và1.959364665s. Hai Start/clock/bom/stateA1/175/return/room;
controller/service/probe errors0. Kết luận kr_runtime/confirmed chỉ trong recipe này.

`verification.json` pass **2124 relations**,885 native actualframer cases,243 packet/12 native153/
408 secure snapshots; exact request SHA/report/nativefield được nối theo vòng/winner. Seal
`race-current-hashes.json` giữ1649 file. Heap/receiver/gettercounter model chỉ ở emulation, không force
client runtime. 50 server/18 checkpoint regressions CP056 dùng lại vì productioncode không đổi;
catalog/docs/artifact/local-history/diff kiểm khi bàn giao. AUD019/CP060, periodic kế CP064.

Tái tạo với label mới: `run_probe.py LABEL CA-r2-stage-social.exe 360 wine_probe_current_pickup_race.py race.gdb`,
seal sau completion bằng `seal_run.py`, verify modulemode từ repo root. Không overwrite raw/chạy chồng.
Delay2s là can thiệp đo, không policy latency server gốc; UI barrier không chứng minh cùng physics tick,
firstarrival là emulator choice, không fairness/auth/fullvalidator/whole reset/K6a/LAN/Internet.
Wrapper discard queue khác generation chưa được runtime riêng. Tiếp report127 stale/duplicate có thể
làm sai marker/state/result; không nâng gate hoặc mở rộng mọi race từ hai vòng này.
