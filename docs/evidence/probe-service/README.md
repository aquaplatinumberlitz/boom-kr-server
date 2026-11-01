# CP048 — service fixture độc lập

Owner Codex; review độc lập waived theo [nguồn](../checkpoints/review-waiver.json), Codex tự kiểm;
không phải review độc lập. Kết quả native và giới hạn từng run được ghi dưới đây.
Contract/giới hạn: [PROTOCOL](../../PROTOCOL.md#probe-service).
Raw/recipe/hash: `local/research/codex/probe-service-r1/`; giữ failed integrated-r1,
sealed integrated-r2 và phép đo bổ sung integrated-r3. Metadata kiểm cuối được pin ở manifest CP048.

Baseline CP047/PR55 merge `5ffd9258ac52068f6ee42786fd4cdca2c4a2a382`, nhánh `kre/probe-service`.
KRE-005b/issue14; catalog KRE-011/issue56. CP048 là snapshot trước tích hợp PR; không gọi waived là approved.

## Nguồn tái sử dụng và phần tránh làm lại

`reuse-question.json`: đọc PROTOCOL/RESEARCH/evidence CP045–047, RoundFence/ItemClaims/UdpRoundKeys,
harness keyrotation/family; boomkr-re kr-data; boom-re oc_e-game/session.rs; boom-rs VN reference,
l10n router và archive KNOWLEDGE. Ledger giữ path/hash/locator, prooflevel, phần reuse/KR gap/phép đo.
Dùng lại layout/counter/peer/collector/native oracle, không RE lại E66680/queue/153/124/165 hoặc
Roller/MaxPower; không nhập nguyên code server VN. Code mới chỉ profile codec và owner/service hẹp.

354 actual frame đã có native oracle CP045/047 được kiểm roundtrip làm regression; **roundtrip đó
không là oracle**. `codec-native-comparison.json` bổ sung so typed prefix của97 frame hiện hành với
native reader độc lập; schema blob chỉ so prefix, native blob/cursor/checksum được kiểm riêng.
Không lấy constants giống VN làm proof KR.

| Artifact | SHA256 |
|---|---|
| Runtime A/B CA-r2-stage-social.exe | `e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef` |
| Static CA-r2-login-logout.exe | `660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3` |
| modO raw memory/base400000 | `a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5` |

RVA=VA−400000; không PE-remap modO. Source binary/DB/pack giữ nguyên, runtime copy không thêm patch
so CP020. Namespace user/net/mount/PID chỉlo/no IPv4 route/files-only NSS, lease một controller;
A/B prefix/wineserver/display/game-dir riêng. Wine diagnostic copy không là Windows nguyên bản.

## Các run có ngày và giới hạn riêng

| Run / UTC01/11/2026 | Kết quả | run.json SHA256 |
|---|---|---|
| integrated-r1 / 11:07:17 | Profile base seq bomb91 sai; checksum guard fail trước Wine/client. | `54ca7de2d298a21783059e6543bcfbac3c2056a27407c77259d6392ab4325d3f` |
| integrated-r2 / 11:08:42 | Hai actual124/bom/17/19/170, hai175/180 mỗi client; oldUDP guard/current apply. | `237645f5b4848f721b9cfbca91270fe953568b00f45972a975b3fe1c8c60696e` |
| integrated-r3 / 11:16:56 | Hai Start/clock; một124/bom/result/return.165/stat vòng1 và reset clock2; thiếu124/118 vòng2. | `2c5a5eb9a9b6a8c7ea8500821f88eb7e29c15da57838656154f0843fd84f18f9` |

Failed r1 giữ nguyên `profile.json`, harness/recipe/hash; profile-v2 sửa metadata base seq85,
không đổi bytes fixture. Loader kiểm SHA/size/CRC trước readiness. Không coi lỗi metadata là
client từ chối protocol; negative test bảo đảm sai seq bị bắt trước khi listen.

Service process riêng giữ socket/framing/seq/round state; `policy_sources` của run pin cả6module.
Cấp key một lần acceptedStart, cùng12345 ở vòng1/12346 ở vòng2 cho A/B; fence/bomb ID1→2,
ItemClaims nhận actual124/catalog và consume một lần trước165. Identity A/B theo accept-order,
authprefix/roster/map/rank/outcome/fuse6s/placement index17/kind2 đều là fixture/emu_choice/design,
không server Nexon gốc. Không full schema/identity hoặc account reconnect.

**kr_runtime/confirmed hẹp:** trong r2 hai client có121full129, clock, actual pickup/118/127,
170/17,19 và175outer302/inner292,180full3/về phòng hai lần. r3 chỉ một result/return;
input Down/Right/Space vòng2 có log gửi nhưng không actual124/118 tương ứng. Ảnh có tutorial overlay,
không đủ gán nguyên nhân cho focus/tutorial/probe/state hoặc gọi input đã được handler xử lý.
Giữ riêng outcome partial, không ghép r3 và r2 thành ca đo paired165 hai vòng.

R3 callback165 **12/12/errors0**, fields `[1,0,0,4,3,17,2,0]`,1191840 effect3→B48350;
secure actorA+5E8 **5→6**, max+5DC7, trên A/B. Stat actorB current4/max8 giữ trong snapshot.
Clock vòng2 cả A/B currentA về5. R2 cũng có clock1/2current5, nhưng GDB cũ chỉ marker
NEEDLE165_BEGIN, chưa GEN165_END/fields, nên không có cặp secure165 đầy đủ cho hai vòng r2.
Không suy lifetime/toàn túi/pxs hoặc runtime tại max.

Exact-old UDP B→A r2 SHA `15a17305e7ae2f7975de2c590da09844f8b1a89ade684dd480055035323dcf7d`,
source57795, round2/sample4: endpoint→queue/drain/game/output13→E66680 key59≠56;
không tail/apply, cùng actor61085430 state15/x120/y159 giữ nguyên. Current key56 tới apply.
R3 có đối chiếu riêng sample8/source58955, actor63F7EBB8 state24/x120/y159 giữ nguyên;
raw/hash trong replay-effects, không lẫn identity/pointer giữa run. low8 check không là full epoch/auth.

## Kiểm, tái lập và giới hạn

`verification.json`:3710 quan hệ cơ học,3sealedruns (gồm1failedbeforeclient),291native packet cases,
24native153cases,189actualUDPchecksumrows/126unique,876secure snapshots/getters,
97native-prefix codec comparisons. Native framer14EF300/typed/blob/CRC và153E8A340 giữ code;
allocation/world context là model, không server authority oracle. Getter52A350 kiểm checksum/value,
19D88D4starts0 và1A7E438reset0/read chỉ trong emulation để bỏ periodic rekey/storage, không đổi
client runtime. Checker không xác nhận ngữ nghĩa hoặc đóng gate từ số kiểm.

12ảnh nguồn đã đọc trong2composite, index/hash ở `visual-index.json`: hai view sau nhặt mỗi vòng và
final; r2 cuối ở phòng, r3 cuối còn trận. Ảnh tức thời/tutorial không thay callback/stat/nativebranch.
Raw cuối run pin `<run>-hashes.json`; profile/source/helper/report/snapshot pin qua manifest CP048.
Lượt verify đầu hiểu sai biểu diễn interfaces/routes trong metadata; sửa checker để đọc `[[1,lo]]`
và route chỉheader, không đổi raw hoặc kết luận isolation.

```bash
python3 local/research/codex/probe-service-r1/run_probe.py new-service CA-r2-stage-social.exe 300 wine_probe_external_service_v2.py local/research/codex/probe-service-r1/service-v2.gdb
python3 local/research/codex/probe-service-r1/seal_run.py new-service
```

Dùng label/prefix/out mới, không đè sealedrun. Profile-v2 cần fixture nguồn host-local đã có SHA.
`service.gdb` là probe r2, `service-v2.gdb` bổ sung165before/after/cursor cho r3; không viết lại probe cũ.
Service EOF hiện chỉ finish room, chưa forfeit/cleanup/promotion/newpeer trong tiến trình mới.
ItemClaims chưa kiểm t/k/distance/actorstate; actual124 r2 có k12345→12346 là quan sát, không
validator đã phục hồi. Các công việc còn lại và thứ tự ở [PLAN](../../PLAN.md#current-order),
[phép đo theo cơ chế](../../RESEARCH.md#feature-gaps). Không fullmatch/K5/K6a/LAN/Internet.


## Sau tổ chức docs: kiểm blocker stale124 ở service

`old124-reuse.json` đối chiếu CP030/042/047/048, cache E2E390/RVAA2E390
(writer E2E5AB đọc record+20), ItemClaims và VN session.rs117broadcastall. Nguồn/hash có ledger;
VN chỉ reference, không chuyển index/opcode/validator. Không mở body RE mới hoặc quét client.
`probe_old124.py` gọi chính `ProbeService.receive_channel` với body của actual124 r2 đã được native
framer/getter kiểm, sau room begin/finish/begin và catalog trùng. Broadcast được thay bằng recorder,
không mở socket/chạy Wine, không coi log đó là native client nhận165.

`old124-service-gap.json`: key room12346 nhưng old124 chứa k12345 vẫn consume claim17 và tạo165;
current124/k12346 sau đó bị từ chối `claim_not_current_or_already_consumed`. Control instance mới
chỉ current124 thì phát165 thành công. Đây là **design/confirmed về implementation gap**,
không kr_runtime, không semantics/validator server gốc. Source service SHA giữ đúng run r2/r3.
Guard k/actor/distance/tick và replay client thực còn ở [câu hỏi](../../RESEARCH.md#gap-floor-pickup),
ưu tiên theo PLAN; không trì hoãn tới full item inventory. Không sửa capture hoặc giả actual request mới.
