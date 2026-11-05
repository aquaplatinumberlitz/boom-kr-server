# Rescue18 trong diagnostic service — CP115

Nguồn chuẩn: [contract/giới hạn](../../PROTOCOL.md#rescue-service),
[câu hỏi/phép thử](../../RESEARCH.md#gap-contact-state), [thứ tự](../../PLAN.md#current-order).
Codex tự kiểm; review độc lập [waived](../checkpoints/review-waiver.json). CP115 đã merge [PR185](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/185), `80e4d48`;
CP114 đã merge [PR184](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/184), `989dd3a`.
Không sửa record complete hoặc raw cũ; gate giữ mở.

## Tái sử dụng và phần khác biệt

- `local/research/codex/rescue-service-r1/reuse-question.json`: source/hash/locator của
  CP036 `collision-r1/rescue-kept`, CP037 `contact-race-r1`, docs KR/VN và current service/exporter.
  [Collision/rescue cũ](../collision-rescue/README.md), [contact](../needle-contact/README.md)
  đã có actual18/blob10/aux1, peer170/18 và native callback/state; không RE lại handler/codec.
- `actual-pair-verification.json`: 15 joins default-off/on trên exact actual17/18 cũ,
  output body so với gói170 đã được native chấp nhận; không dùng encoder tự sinh làm oracle.
- `native-aux-routing.json`: hai case chạy đoạn native đã cache, getter mô hình trả1/2
  qua `E6ED03 → E6EDF4 → E6EF10 → E6F060` không đổi. Mô hình list/gate/profile,
  **không phải runtime aux2 hoặc xác thực rescuer**. Hash `modO`:
  `a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5`.
- VN `boom-re/docs/oc_e-game.md` chỉ hướng dẫn ca chạm/no-touch; KR opcode/layout/aux
  lấy từ KR. `boomkr-re/docs/kr-data.md` không thay bằng chứng runtime team/roster.
- Phần mới: service opt-in, hash guard cho đúng roster/team/control121, khai báo106 trước
  Start và giữ slot2 trong153. Slot2/ID3/team1 là **actor tổng hợp**, không là client C thật.

## Run và recipe

Raw/recipe riêng, cùng private user/net/mount/PID namespace, lo-only/no routes/files-only NSS,
lease độc quyền, hai prefix/display; source/config giữ nguyên trong từng run. Source binary/DB không đổi.

| Run | Kết quả và giới hạn | Metadata/raw |
|---|---|---|
| `rescue-service-r1/cycles` | B actual Ready95, thao tác StartA chưa có104. Dừng SIGINT đúng Python controller, `finally` cleanup/lưu raw; exit130. 749 raw được seal. Đây là phép chuẩn bị thất bại, không phải rescue. | `stop-initial-run.json`, `initial-run-seal.json`, `cycles-recipe.json`, `runtime-source/` |
| `rescue-service-r2/cycles` | 162,694902590s, error0; sau chuyển106 về phòng trước Ready/Start, native121/clocks, A17→18 và peer170/18; dừng khi đang Battle sau cứu/di chuyển. Không kết quả/reset. | `runtime-preflight.json`, `cycles-recipe.json`, `cycles/run.json`, `cycles/service-state.json` |

Run thành công SHA:
`315b34e75b8b24f0774b522cd91e4a59c88cd514b76ccf157d93a0e8d7c67ed2`.
Peer170/18 seq103 SHA:
`8c51eab19f4ac13c32de43b309a0e6e4efda86ed3106108c868ca63055fc30b9`.
Artifact runtime A/B SHA:
`e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef`.
Static copy SHA:
`660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3`.
VA/RVA: image base400000, RVA=VA−400000. Controller
`local/research/codex/wine_probe_rescue_service_ready.py`; helpers/probe và production snapshot
ở `rescue-service-r2/runtime-source/`, source/hash trước run trong `runtime-preflight.json`.

## Kiểm và giới hạn

- `runtime-verification.json`: **1.000 joins**, gồm **920 raw size/SHA** sau cleanup,
  front/upstream TCP nguyên byte (không replay), actual A17 seq193/A18 seq199,
  matching trap epoch, một170/18, active153 slots0/1/2 với slot2 ba endpoint0, hai client thật.
- `actor-fields.json`: **147** captured fields kiểm bằng native secure getter52A350 trên copy.
  Native A `E6F060` trước56, `E6F10A` sau26; B trước dispatch56,
  `E86820(slot0,120,159,0,1) → E86892` sau26, full170 reader7/7/errors0 tạiE863CA.
- `native-reports.json`: **69** native framer/typed/blob cases; `native-153.json`: **6** cases.
  Allocation/storage của emulation là mô hình, không là một runtime thứ ba.
- `rescue-visual.png`/`visual-review.json`: thủ công đối chiếu HELP trước chạm và A đã đi sang
  phải sau cứu trên hai view. Không khẳng định exact XY sau di chuyển từ ảnh.
- `rescue-service-r1/server-tests-retry.txt`: **188 tests**, gồm13 rescue policy cases.
  Lỗi gửi giữ trap/cursor/pending/credit trong direct model; lifecycle thật vẫn đóng/abort theo service.

Khác biệt trước/sau106 chỉ giải recipe hiện tại; chưa định danh đầy đủ nguyên nhân native chặn Start.
Không có aux0/aux2 runtime, cứu chiều B, kết quả sau cứu/Start2, đa inventory, distance/life authority,
ba client thật, vanilla Windows, fullmatch/K5/K6a/LAN/Internet. Phép thử kế ở RESEARCH; không tạo plan riêng.


<a id="cp116"></a>
## CP116 — Reverse rescue B/aux0

Contract ở[PROTOCOL](../../PROTOCOL.md#rescue-reverse), next test ở
[RESEARCH](../../RESEARCH.md#gap-contact-state). Source-first dùng đúng CP115/control CP036,
VN chỉ hướng khái niệm teammate; không thêm native body hoặc đổi profile/binary/DB nguồn.
`local/research/codex/rescue-reverse-r1/reuse-question.json` giữ sources/hash/known/reuse/gap/minimum.

| Run | Kết quả | SHA run.json |
|---|---|---|
| `rescue-reverse-r1/cycles` | 163.670107134s; actualB17seq157→B18seq163/aux0, currentA-onlyguard rejection/no170; local56→26, peerAview vẫnHELP. **Control**, không integrationpass. 915raw sealed. | `a3a0f2b345fab0d87f7c6b7c322365559a3187ea90d68dc1d77ed1e27f44188c` |
| `rescue-reverse-r2/cycles` | 163.271083062s; measuredreverseguard/170 A/nativeboth56→26/full7/7, movementvisual. 920raw sealed. Oneactivecycle, no result/reset. | `a59808d7093005ce4ef92e96e3eb7ea75e0b1c0a4fb93e672af6878be8c1f04b` |


Mỗi root có `cycles-recipe.json`, `runtime-preflight.json`, `runtime-source/`, capture và native probes.
Controller `local/research/codex/wine_probe_rescue_reverse.py`/`wine_probe_rescue_reverse_forward.py`;
singlelease/private namespace/lo-only/no-route/filesNSS và A/B prefix/display riêng. Không thay
source/config giữa run; control hoàn tất/lưu raw trước sửa service rồi chạy positive riêng.

Control: `control-verification.json`, `control-visual.png`, `visual-review.json`, 120 capturedfields/
66 copiednativepacketcases. Positive: `runtime-verification.json`, `actor-fields.json`,
`native-reports.json`, `native-153.json`, `reverse-visual.png`/`visual-review.json`.
**1000** positivejoins gồm **920** rawsizeSHA; 
raw thật/native front/upstream joined, không replay. Native getter/framer là emulation riêng.
190tests/15rescue methods; oppositeaux/directioninvalid/failedsend/credit/cursor guards giữ.
Source/runtimeartifact/hash/VA/RVA chung với CP115 ở trên. Peer170/18 SHA:
`60d5aadf20cf46e9a8fd8fafbb2e7ccddb60fc5011fcc253f294fd14e46d891e`.


Chưa kết quả/return/Ready/Start2 sau rescue trong hai run. Next reuse CP083 all-liveAB accepteddead
rank2 là **emulator policy**, profile có syntheticactor nên phải đo compatibility, không gọi originaltie/teamrule.
AUD036 thủ công đọc docs→raw/control/positive/code và sửa stale current-status; audit kếCP120.
CP116 chờ PR, selfcheck/waived; không đóng issue14/gate.
