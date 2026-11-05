# CP135 — Barrier loading còn TCP và deadline tùy chọn

[Contract/giới hạn](../../PROTOCOL.md#loading-timeout), [câu hỏi](../../RESEARCH.md#gap-map-spawn-loading),
[thứ tự duy nhất](../../PLAN.md#current-order). Baseline CP134/PR205 merge `c2b2de4`;
Codex tự kiểm, [review độc lập waived](../checkpoints/review-waiver.json). #14 và các gate giữ mở.

## Tái sử dụng đúng câu hỏi

`local/research/codex/connected-load-stall-r1/reuse-question.json` pin nguồn/hash: CP050 actual99
hold10s/release, CP072/074 preclock EOF/cleanup, current barrier trong service; VN lobby/oc_e-game
và KR kr-data. Dùng lại wrapper/framer/native121/phase/clock, recipe namespace và normal UI.
VN chỉ tham chiếu khái niệm session/loading; không chuyển opcode, layout hoặc policy timeout.
Control mới chỉ đo khoảng trống **TCP còn mở nhưng B99 chưa dispatch**, không RE lại loader/99/153.
`loading-timeout-r1/reuse-question.json` nối control với consumer175/180/107/110 và probes slot-clear/
chief đã đo tại CP074; không phân tích thân hàm mới hoặc gọi hàm game.

## Hai run tuần tự, seal trước đổi source/config

| Run local/research/codex/ | Kết quả | run.json SHA256 |
|---|---|---|
| connected-load-stall-r1/cycles | 181,662s/exit0; actualB99 giữ46,349s tới mốc đo; both121129, A99, không197/196/153/clock. 781 raw,896 quan hệ. | `5e87eaa428cb52c0c992dde8c43e1cb57483cb7245a5e8a91ffac3d3738b7e07` |
| loading-timeout-r1/cycles | 180,260s/exit0; deadline sau121 đạt30,043s, đóng riêng B/connection8; A175302→1803→107/ID2→0→110/chief1→UI phòng. 787 raw,898 quan hệ. | `afb906fbef71d0b66dd1579388712b7d5504ae02f640b0ca886247ba166e36c9` |

Control wrapper giữ **frame99 thực do B gửi**, không giả client chưa load. Sáu mẫu `/proc/PID/net/tcp`
độc lập trong34,814–44,839s sauhold cho cả hai channel ESTABLISHED; log service không đóng channel
trước mốc46,349s. Mẫu sau mốc đo có AEOF do cleanup, không dùng nó làm nguyên nhân stall.
Ảnh cuối control: hai scene/timer02:00; nativeclock chưa chạy. Không suy vô hạn từ cửa sổ45s.

Fixed thêm duy nhất `loading_timeout` vào profile mới và policy production; không sửa profile/raw cũ.
B99 tới sauStart, bị giữ10,855s trước close; **30s tính từ121**, không từhold. Không release99,
không kill client từ controller trước phép đo. A đọc đủ175302/1803/errors0; bằng chứng107/base6→rewind0→
wrapper1, ID2→0/chief1 nằm **sau180 hiện tại**, không nhầm lần107 của idle Back/rejoin trướcStart.
Ảnh A chỉKRLocal/nútStart/Btrống; B có dialog mất kết nối/error10 như dự kiến. Không gửi result tới B
đã đóng, không actual17/19/bom/clock ở hai run. Forfeit/rank là policy đã chọn, không một trận đã chơi.

## Recipe và phép kiểm

Mỗi thư mục giữ `cycles-recipe.json`, controller path/argv, `runtime-preflight.json`, server snapshot,
`launch-exit.json`, `raw-seal.json`; prefix/display riêng, user/net/mount/PID namespace chỉlo/no-route/
NSSfiles-only/mộtlease. Chuỗi: preflight→run→cleanup→kiểm toàn input SHA→seal→derive; không sửa
source/config giữa run. Copy runtime A/B `e9bb4766…`, static `660447c2…`, modO `a08950c8…`;
SHA đầy đủ ở checkpoint; base400000/RVA=VA−400000. Không thay binary/DB/pack nguồn.

`verify_control.py`:896 quan hệ, `verify_fixed.py`:898; copied-native framer/getters valid/CRC/partial
51 và66 cases tách khỏi Wine callbacks. `visual-review.json` pin bốn ảnh đã đọc. 226server tests gồm
6regressions deadline/boundary/defaultoff/bothmissing/roundreset/replacement/fence. Docs/catalog/issues/
requiredlocalhistory/diff kiểm lúc bàn giao; checker không tự xác nhận ngữ nghĩa/gate.

CP135 đã merge [PR206](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/206) tại `57b3b53`.
Positive normal sau đó được ghi riêng ởCP136 bên dưới; raw/history CP135 giữ nguyên.
Không native load failure/network loss/original timeout/fullmatch/K5/K6a/vanilla/LANInternet proof.


<a id="cp136"></a>
## CP136 — Positive hai vòng với deadline bật; AUD042

Raw `local/research/codex/loading-timeout-positive-r1`, recipe/controller/preflight/runtime-source/
raw-seal và `reuse-question.json`. Dùng nguyên CP134 normal controller/geometry/client/map/fuse/probes,
chỉ thêm optional30s trong profile; không hold wrapper, không RE lại chain hoặc đọc thân hàm mới.
Run374,592s/exit0/1209raw, SHA256 run.json
`1fb9db23bbfedcc8ef291f071f01189b21f97270872b1fb208c682711fad3a47`.

`loading-verification.json`:24 quan hệ: generation1/2 pin cùng A4/B8, hai actual99 resolve barrier
sau15,165/16,188s, đều trước30s; result sau deadline69,641/64,060s. Không timeout/abort/close trước
mốc đo. Start2 deadline mới, key12345→12346. `cycles-verification.json`:1453 joins dùng lại exact
CP134 geometry/native195/sharedcascade/actualboth19/rank2,2/175180/secondroom assertions; không coi
đây là RE cơ chế mới. Copied-native354packet/12peer153,4542actorgetters,7092flaggetters/7020array
là model checks riêng. Hai ảnhfinal-both đã đọc: AhostStart/BReady/cùngphòng, chưa Start3.
Source/config giữ nguyên tới cleanup, raw seal trước chuyển việc; không sourcebinary/DB/packmutation.

CP135 checkpoint draft chưa commit ban đầu thiếu evidence[]/method enum; bản bị reject, index/log và
correction metadata được giữ ở `loading-timeout-r1/schema-validation/`, auditCP136 đối chiếu. Chỉ
sửa metadata trước firstcommit; không sửa findings/conclusion/raw hoặc record complete trong Git.
CP135 finalcheck204587evidence/0missing/0errors;226servertests reuse vì production không đổi sauPR206.

CP136/AUD042 self-check trong thay đổi hiện tại, audit qua136/kế140. Khép scoped positive loading;
PLAN chuyển sang tích hợp normal movement/survivor/result-drain với current-trap guard và deadline
đã biết. Không suy current all-dead profile đã bật life/result guards: nó chưa bật hai option đó.
Hai run fault/control và run positive là các thí nghiệm riêng; không ghép thành fullmatch/K5/K6a/
vanilla/LANInternet hoặc toàn bộ thời gian load/membership đã được nghiệm thu.
