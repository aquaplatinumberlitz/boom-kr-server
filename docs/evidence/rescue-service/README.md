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
CP116/PR186 đã merge `f33f3b8`, selfcheck/waived; không đóng issue14/gate.


<a id="cp117"></a>
## CP117 — Result/return sau cứu và blocker Start guard3

[Contract/giới hạn](../../PROTOCOL.md#rescue-return-start-guard),
[gap/phép thử](../../RESEARCH.md#gap-round-reset), [thứ tự](../../PLAN.md#current-order).
CP116/PR186 mergef33f3b8; source/runtimeprofile giữ nguyên, 190tests reuse theo sourcehash.
Source-first đọcCP083all-liveABaccepteddead/CP110actual585focus12/CP028boundedStartretry và
CP115–116rescue, existingDE0310/DCAD80cache. Không lặpwriter/codec/kim/ReadyRE trước đo khác biệt.

Raw `local/research/codex/rescue-reset-r1`…`rescue-reset-r4` và `rescue-start-guard-r1`;
mỗi root córecipe/preflighthash/runtime-source/capture/observation+seal. Singlelease/privateuser/net/
mount/PID/lo-only/noIPv4route/filesNSS; A/Bprefix/display riêng. Chỉ normalOS input/read-onlyprobes,
không đổi code/config giữa run, khôngmint report/endmatch hoặc forceactor. Binary/DB nguồn bất biến.

| Run | Kết quả và giới hạn | SHA run.json |
|---|---|---|
| `rescue-reset-r1/cycles` | 258.967659672s; Rescue→A/B19/native175180/room; Ready6s từrefresh chưa95. Partial. 1530 rawsealed. | `4c45c88653816b4fb6a09f4b42efd1f8d5573e3fcb007cfa5d251bb5930c7e45` |
| `rescue-reset-r2/cycles` | 227.738011608s; BLefttimed→actualbomb3,2; chỉB19/noresult; chưa kiểmReadyrecipe. 1297 rawsealed. | `2d33b1259799643f9b34074044cc1aa98cf068f59398a25771c4802eb5e95266` |
| `rescue-reset-r3/cycles` | 271.154490803s; HaiSpace→actualA/B19/result/room; actualB95 theo585focus12, mộtStartA chưa104. 1335 rawsealed. | `687d0abde52d28efa903798bbce28b5217acd352916c027e0ee8e20033284d7a` |
| `rescue-reset-r4/cycles` | 292.207587007s; Cùngroute; actualB95, tốiđa3StartA chưa104. Blocker rõ. 1646 rawsealed. | `bd37c3a7da62707e7eee53644f75becba58e1703ed2477c628993eeae202d820` |
| `rescue-start-guard-r1/cycles` | 297.138212744s; Read-onlyStarttrace: initialguard0/build104, returnedguard3/error/nobuild. Slot/cause chưađo. 1672 rawsealed. | `005862078acd5e42c246465ebe7f27a0dda3c62720f6119d4276996e6d149d69` |


`rescue-start-guard-r1/guard-observations.json` giữ source-bound nativeStarttrace và1.672raw;
`DCAD80-return3-focused.asm` đọc một thân hàm đã cachemộtphần, mở nốt cạnh runtime trả3,
modO SHAa08950…/base400000/RVA=VA−400000, không quétclient hoặc sửaDB.
`DE0310→DE0354/DE03A0→DE03F3→DE0422` đủ phân biệt pending/predicates/roomguard;
initial0→DE0463, returned3→table[3]=DE0434. Hai site trả3 DCB7DA/DCB7FD chưa actualhit,
slot/packedgroup/helper703A90/1240D70 cònunknown. `rea-availability.json` là inventorykhảdụngIDA,
không phải provideranalysis/REmới hoặc thay đổi config.

`actor-fields.json`/`native-reports.json` từngroot kiểmsecurefield/typedpacket bằng code nativecopy,
khác runtimecallback. `r1/returned-live-observation.png` preview chọnscreen99 theo lexicalsort là
**ảnh cũ, khôngcurrentstatus**; `returned-live-observation-latest.png` dùngmtimechohai phòng,
correctiongiữ trongpartial-verification. Khônggọiảnhcũ làhangproof.

Tại thời điểm CP117, read-onlyslotmeasurement `rescue-start-slot-r1` chưa xong; CP118 đã bổ sung actualslot2cause; không tính là
bằngchứng CP117 hoặc checkpointcomplete. Preparedpositive-docs draft ởr2 chưa chạy vàđược đánhdấu
unexecuted; không kếtluậnStart2/freshrescue từverifier/draft chưapass. Mọi partial giữnguyên.

Giới hạn: mộtchosenresult/return/actualReadyB saucứu, **chưaStart2**, chưafreshrescue/reset/2finishedrounds.
Sourceartifact/modO/currentclienthash vàknownVA giốngCP115–116. Outcome làpolicyemulator,
không fullmembership/auth/authority/fullmatch/vanilla/K5/K6a/LANInternet. CP117/PR187 đã merge `c2ca94d`;
AUD036qua116/kế120, recordcompletecũkhôngsửa.


<a id="start-slot"></a>
## Run slot trước migration CP118

Raw/recipe `local/research/codex/rescue-start-slot-r1`/`rescue-start-slot-r2`, singlelease/namespace/
lo-only/filesNSS/separateA/B/sourcefreeze như trên. `r1/quota-limitation.json`: 1.682raw, quota24 theo
kind bị ambient DCAD80call round0 dùng hết; **không dùng để kết luận slot saureturn**.
`r2` chỉ đổi observer scopeStartentry→switch và quota perstartindex, không đổi service/profile/client/fuse.
`slot-verification.json` pin1.693raw/sourcehash/cleanup; actualreturnedslot2group0/helper1/hostflag0→DCB7DA,
initialslot2group1. [Contract/giới hạn](../../PROTOCOL.md#rescue-start-slot). Phép đo này đã kết thúc/lưu
rawhash/cleanup trước migration theo yêu cầu chủ dự án; CP118 pin batchlist bên ngoài, không sửaCP117.
Giới hạn lịch sử CP118: khi đó chưa refreshfix/Start2/freshrescue; CP119/120 bên dưới bổ sung, việc hiện hành chỉ PLAN. Không wholewordenum/auth/gate.


<a id="cp119"></a>
## CP119 — Refresh synthetic2 mở Start2; R2 rescue còn partial

[Contract/giới hạn](../../PROTOCOL.md#rescue-slot-refresh), [phép thử](../../RESEARCH.md#gap-round-reset),
[PLAN](../../PLAN.md#current-order). SauCP118/PR189migrationmerged `dae23b7`, Codex tiếp ngayruntimefix
slot2đãlưu, không đo lại sourcecause. `local/research/codex/rescue-slot-refresh-r1/reuse-question.json`
đọc CP118actualslot/group/known106110layout/source/probes; newservice chỉ110readyrefresh saureturn.
`runtime-preflight.json` pin code/profile/client/probe; sameprofiletrue, source/config frozen trongrun,
namespace/lease/lo-only/no-route/filesNSS/A/Bprefix-display riêng và cleanup theo controller.

Run SHA `8db983a170ce9ca17bd86a1be7b204a9d9926b000a2c51ae8651f1abb9cde532`, 313.107640202s, exit1 vì R2freshrescue khôngđạtbound,
serviceerrors0. `start2-verification.json`: **1894** joins gồm **1840raw size/SHA**;
slot2group1/roomguard0/build104/actualreturnedA104/two121155/clocks/key12345→12346/native153
được xácnhận. FreshR2B17→19/no18 là **partial/fail plannedreset**, không sửa assertion thànhpass.
`actor-fields.json`, `native-reports.json`, `native-153.json` nativecopygetter/framer khác runtime;
`R2-contact-observation.png` manualhai view cònHELP/Apositionkhác, chưagán exactXY/tácnhân.
191server tests; oldCP118guard3control giữ nguyên. Không originalserverready law hoặc fullreset/gate.

CP118record/migrationreport nguyênbytes; #188 tổ chức đãđóng sauPR189và actualREresume,
localissue/task/PLANđồng bộtrongCP119, không sửa snapshotCP118. Audit kếCP120/AUD036.


<a id="cp120"></a>
## CP120 — Fresh R2 rescue theo current epoch, audit AUD037

[Contract/giới hạn](../../PROTOCOL.md#rescue-round2-feedback), [phép thử](../../RESEARCH.md#gap-round-reset),
[PLAN](../../PLAN.md#current-order). CP119/PR190 đãmerge`bfd526b`, code/profile khôngđổi.
`local/research/codex/rescue-R2-contact-r1/reuse-question.json` source-first CP119raw/CP111peerapply/
CP084freshfeedback/cachedgetter; khôngRElạicodec/Start/rescueconsumer hoặcportnumericVN.

Raw/recipe `rescue-R2-feedback-r1`/`rescue-R2-feedback-r2`, controller
`wine_probe_rescue_R2_feedback.py` và`wine_probe_rescue_R2_feedback_retry.py` trong local/research/codex.
Private namespace/lo-only/no-route/filesNSS/singlelease/two-prefix-display/sourcefreeze; rawsealed saucleanup.
r1failedBwineboot35s/noBattle/146raw đượcgiữ; cuốiscriptprepare.with_name lỗi trướcghi provenance,
input/preflight đã ghi vàshell vẫn chạy. Correctionr2 hoàn tấtprepare trướclaunch; baselineprefixcopies và
Winebootstartup60s riêng, khônggamedeadlinechange. Khônghạassertion hoặcgiảrawmissing.

Positive r2run SHA `e83b5d91de9eb43f5f86c3e2f223a9ce44d4209d69acfdc6abe30d1cb2efed7f`,
300,388426598s/exit0/error0, source/service/profilecùngCP119. `reset-verification.json`: **1863joins**
gồm**1782rawsizeSHA**, actualB95seq262/A104seq304/2keys/two121clocks/oneresultreturn/freshB17→18R2,
local/peer56→26/full1707/7/errors0. `actor-fields.json`621copiednativegetters;
`native-reports.json`213nativepacketcases, `native-153.json`12cases (valid/CRC/partial) táchWine.
`feedback-verification.json`1002capturedlive/peerfieldnative52A350checks; rawwrapper/data đượcpin,
emulationcounterreset chỉmodel, khôngsửa runtimeclient. `reset-visual.png`/`visual-review.json` manual.

**Khôngnângnghĩa feedback:** park sample304đạt279, Up4s khôngđạt159/actual18muộnhơn;
BlocalrescueentryA(120,189), B(120,159) làsnapshotkhôngfullcontactthreshold. PeerB64quota/touchonly,
park noack; fullUptrajectory/CP119causalinput-orUDPdifference cònunknown. Hai view finalAposition
khác khônglàm mất factBescape, cũngkhônglàprooffullmovementconsistency. Limits ởPROTOCOL.

MộtR1result vàR2activeescape, **chưaR2result/secondreturn/Ready saurescue**, khôngK5/K6a/fullmatch.
191server tests CP119reuse/sourceunchanged. AuditthủcôngAUD037quaCP120/kếCP124;
CP120selfcheck/chờPR/reviewwaived. CompleteCP119vàoldfailure/rawgiữ nguyên, externalbatchmanifest mới.
