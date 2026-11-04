# CP075 — Guard body cũ/trùng qua TCP thực

Baseline CP074/PR109 merge `f84823dd8745930713de1d99c5e7fb45d83ee50d`; KRE-038/[issue110](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/110).
Codex solo/tự kiểm; [review waived](../checkpoints/review-waiver.json), đã merge [PR111](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/111) tại `41f0fa9`.
[Contract/giới hạn](../../PROTOCOL.md#current-tcp-replay), [gap](../../RESEARCH.md#gap-diagnostic-service),
[thứ tự](../../PLAN.md#current-order). Raw/scripts/seals tại `local/research/codex/current-tcp-replay-r1/`.

**Confirmed hẹp:** service nhận 12 proxy-emitted replay qua socket channel thật/framer/current seq,
loại cả same-body và old-body của118/124/127(sub17,19). Nativewriter chỉ phát các frame nguồn;
proxy mã hóa lại nguyên body đã capture với sequence upstream mới. Không gọi replay là nativewriter
mới, nguyên frame cũ hoặc bằng chứng server Nexon. Hai starts/clocks, kết quả/return/refresh vòng1 và
hiệu ứng/state vòng2 đã thấy; **chưa đủ hai vòng kết quả/return** trong run này.

`reuse-question.json` pin CP049proxy thất bại trước67/Start và dispatcher124, CP063127/CP067118,
CP058/059auth/cache và CP069basicprofile/nativeoracles; VN protocol/oc_e-game/session, KR data,
boom-rs/l10n/archive đúng câu hỏi. Tái dùng codec/body/schema/writer/consumer/guard/clock, tránh RE lại.
Điều kiện mới là current auth137/string/cache/profile41 đã có positive A/B67; không chứng minh yếu tố
nào riêng lẻ đã giải lỗi proxy lịch sử. S2C forward nguyên byte, productionservice giữ nguyên.

Run `tcp-current`, UTC 2026-11-02T21:44:19Z, SHA256 run.json
`106053d85773109ae951f715adc8214c7760e7a328b78470a19fa9bd24add762`. Runtime A/B SHA
`e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef`;
static `660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3`;
modO raw/base400000 `a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5`, RVA=VA−400000.
Không sửa binary/DB/pack nguồn, không patch/force mới. Private user/net/mount/PID namespace,
chỉlo/no route/NSSfiles-only/lease đơn; channel3858→23858, bootstrap/auth/UDP thực giữ route local.

| Body giữ nguyên | Same-body vòng1/vòng2 | Body vòng1 gửi trong vòng2 |
|---|---|---|
|118|`duplicate_bomb_serial`|`bomb_round_key_mismatch`|
|124|`claim_not_current_or_already_consumed`|`round_key_mismatch`|
|127/sub17 và19|`duplicate_report_body`|`report_tick_not_forward`|

Tổng8same+4old. Old gửi sau actualA127/sub39 vòng2 đã nhận; key/tick/body không sửa. A upstream161
frame =149actualfront+12replay; B100actualfront/100upstream. Hai stream join exactbytes với các
TCP chunks service nhận, front/upstream seq riêng40+3n; trước injection frameactualforward exactbytes.
OriginSHA/bodySHA/upstreamSHA/sequence/timestamp/serviceentry ở`tcp-replays.json`/`verification.json`;
archive `proxy-*-client/upstream.bin` và `tcp-proxy.json`. Không nhầm TCPchunk là packetboundary.

Chỉ4actual118 cấpID1..4/4deadline jobs, mỗi client4native159 và4native158. Không ID/job từ replay.
Năm actual124 được nhận: vòng1 index17Roller/27Bomb; vòng2 17/27/28Fire; mỗi client đúng5native165.
Native secure getters trước/sau165: speed5→6/capacity1→2 và vòng2 power1→2; baseline reset về1/1/5.
Bom power lần lượt1,1,1,2. Không tái khẳng định hiddenstore/reveal/pointerjoin: `tcp.gdb` không có
probe đó; CP068 chỉ là contract tái dùng. Không CP069 ba hiệu ứng mỗi vòng (Fire vòng1 không nhặt).
Accepted actual127/17,19 mỗi vòng, B đúng4native170; R1 stateA1 trước175, R2 stateA1 tại writer19
và peer170_END. Không replay170, thêm tiêu hao hoặc kết quả từ reject; serviceerrors[]/khôngerrorstate,
CreateProcess/HTTP/stop marker trong window. Đây không fullvalidator/auth/distance/cap/freshserial.

Controller giới hạn360s (toànrun367,817s gồmsetup/cleanup), dừng4,549s sau actual19 vòng2; policy
rank chờ≥8s saudeath+có actualpeer21, return+5s/refresh+3s. Raw có firstrefresh, secondabort EOF khi
controller cleanup; không secondrank/return/refresh. Không giả đây là client từ chối result hoặc hạ
policy để làm pass. `analyze_completed.py` đã dừng ở assertion two-cycle (`KeyError early_completion`)
trong `integration-check.txt`; các verifier kế hoạch được giữ nguyên. `verification-scope-change.json`
ghi bản kiểm bounded riêng, pass chỉ cho điều đã đo; planned acceptance vẫn **partial**.

Kiểm bounded: 149transport joins/sideeffects, 2643integration relations,
825nativeframerupstream/789front,177actual-packet/282report-packet/12native153 cases và474secure
snapshots. Front/upstream model có input trùng nên không cộng thành proof độc lập. Native framer/
heap/gettercounter trong model chỉemulation; callback/state/effect thật có sourcehash riêng. Socketpair
bench chỉ kiểm sequence/forward regression, không native oracle. Codeproduction không đổi; reuse
baseline72server/18checkpoint tests; docs/catalog/artifacts/localhistory/diff kiểm bàn giao.
AUD022/CP072, audit periodic kếCP076. Chưa fullmatch/K5/K6a/LAN/Internet/vanilla/fullcodec/toàn127.

Recipeexact `tcp-current-recipe.json`; `run_probe.py tcp-current CA-r2-stage-social.exe 360
wine_probe_current_tcp_replay.py tcp.gdb` (đườngscript đầy đủ trong recipe). Seal raw saucomplete;
parsefields/lifecycle/hidden/derive_roles/parse_tcp/native_actual/native_reports/native153 rồi kiểm
bounded theo scope-change. Không chạy lại cùnglabel/overwrite; không lặp run chỉ để có ảnh đẹp.
Khoảng trống hai-cycle guard integration giữ rõ. Bước tạiCP075 (lịch sử): native Back86 đã có ởCP074 nhưng
service còn giữ membership/setup markers; đối chiếu nguồn và đo create/join lại cùngchannel trướcfix.

[CP076 bổ sung emptyroomBack/recreate](../back86-membership/README.md); CP075 hai-cycle guardacceptance vẫnpartial. Thứ tự hiện tại ởPLAN.

[CP085](../tcp-guard-cycles/README.md) có run mới ghép selectedTCPguard với hai result/reset; rawCP075 vẫnpartial như mô tả, không backfill từrunmới.
