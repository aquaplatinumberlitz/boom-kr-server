# CP071 — Fire/power2, control ngoài tầm và blocker đặt bom

Baseline CP070/[PR101](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/101) merge
`b0ddb1271c9027392d1f90631cd6b2a1538b884d`; KRE-034/[issue102](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/102).
Codex solo/tự kiểm, [review waived](../checkpoints/review-waiver.json); tích hợp chờ PR.
[Contract/giới hạn](../../PROTOCOL.md#mixed-owner-power), [gap](../../RESEARCH.md#gap-blast-block-chain),
[thứ tự](../../PLAN.md#current-order). Raw/script/log/capture/seal: `local/research/codex/mixed-owner-power-r1/`.

**Câu hỏi partial:** chưa có hai bom khác owner nằm trong tầm root2. Không biến verifier pass về
hash/quan hệ thành chain thành công. Hai run có ý nghĩa được gom một checkpoint, không thử input tiếp.

`reuse-question.json` pin CP033/064 helpers EBE180/B32390/EB80E3/callback/ID mapping; CP066 route và
floor582, CP031/068 Fire+1/power-at-placement, CP070 UDP hiện hành. KR kr-data, VN items/oc_e-game
§5a-5d, boom-rs VN reference, l10n/archive dùng theo provenance. kr-data radius1cell/power dẫn VN,
không là phép đo KR độc lập; không suy asset/weight thành hành vi. Tránh RE lại itemconsumer/writer,
clock/auth/cache/UDP/nativebody. Sửa duy nhất data floor17row6col3 kindRoller2→Fire1 so CP066,
không165hold wrapper; production service/currentguards/CP065 distinctspawn giữ nguyên.
Native582 framer/blob/11ED590 đọc đủ16inner/counts1/0/0, CRC sai/partial bị loại trước run.

Runtime A/B SHA `e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef`;
static `660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3`;
modO raw/base400000 `a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5`.
RVA=VA−400000; không PE-remap raw. Không sửa source binary/pack/DB/patch hoặc ép hàm/state.
Hai controller tuần tự, mỗi run private user/net/mount/PID namespace, chỉlo/no route/files-only NSS,
một lease; gamecopyCP059/prefixwarmCP062/display/wineserver riêng; actual99→153/directUDP.

| Run / UTC02/11 | Kết quả | run.json SHA256 |
|---|---|---|
|power-live /17:35:23|A nhặtFire/power2, rootA(3,3), secondaryB(6,5)/power1; ngoài tầm dự định, từng timer chỉ xử lý bom riêng ở cảhai view.|`96651fe754676b1e782a654517cd5b6f6aae9f9de28034eaeb29e1b0ce590edb`|
|power-feedback /17:47:46|Native feedback đưa B tới cùng tọa độ A trước Space; không actual118 trong cửa sổ4s; controller báo bounded input failure. Không chain experiment.|`a68f87e27499213f54a51cbd46a8ec280140ad383a4ba48724cd824ec0893f47`|

Cảhai run: actualA124 row6/col3/index17/kind1, một165/receiver; native Fire actor+4BC1→2 ở cảhai
view. Run đầu có actual118 owner0/1, IDs1/2, power2/1, serial0 riêng; native159full39 và158full13/errors0.
Root158 ID1 chỉ process/state root, secondary chỉ trong callbackID2; không nằm trong hình học mục tiêu.
CP070 next-step dự kiến Up1 đưa B tới row3 đã bị **actualrow6** sửa; CP066 chỉ chứng minh đường từng
run, không hold→tọa độ/boundary tổng quát. Midpoint row6 là ô đi qua, không điểm dừng.

Retry đổi điều kiện bằng feedback, không chỉ đổi hold: gate bật sampling read-only ở cached1256B3B,
GDB Python đọc wrappers/checksum, ghi hostJSON atomic từng view; controller giữ Up bình thường,
nhả khi BnativeY≤159, tối đa12s. Các JSON feedback được đối chiếu native52A350/secure dump cùngns,
không gọi hàm game/ghi memory. Mẫu cuối viewB thấy cảhai(120,159), trước UIspace0.15; vẫn không118.
Không quy nguyên nhân cho probe, input, actorstate hoặc engine; mã lỗi là assertion điều kiện đo của
controller. Teardown debugger disconnect xảy ra sau lỗi, không là native crash proof. Không có159/158
ở retry; không lấy input thành request hoặc tọa độ thành bom. `verify_positive.py` giữ điều kiện proof
đã dự kiến; `verify.py` kiểm đúng partial và lỗi, không làm nới điều kiện để nhận chain.

`verification.json`: 1926 quan hệ/seals/source, 768 nativeframer cases,
132 native packet cases, 264 secure-field snapshots. Model heap/receiver/gettercounter chỉ
emulation; native getter/checksum code nguyên. Service/export không đổi, baseline72server/18checkpoint
tests giữ; catalog7/docs/artifacts5/5/requiredlocalhistory/diff kiểm khi bàn giao. AUD021/CP068,
periodic kếCP072. Hai actual run không hai trận/kết quả/reset; không fullmatch/K5/K6a/LAN/Internet.

Recipe exact tại `<label>-recipe.json`, seal mọi raw `<label>-hashes.json`, preparation/reuse/feedback-retry;
run_probe.py với label/prefix mới CA-r2-stage-social.exe300, wine_probe_mixed_power.py/mixed.gdb hoặc
wine_probe_mixed_power_feedback.py/mixed-feedback.gdb. Feedback GDB path/gate/JSON phải đổi theo
label (Bcommand đổi live-A→live-B). Không ghi đè raw hoặc chạy chồng controller/lease. Saucomplete:
seal,parse_fields/parse_chain,derive_roles,native_packets,verify. Không tự lặp recipe đã bị blocker.

Mixed-owner in-range/adjacent/ba bom/vật cản/authority/reset vẫn mở; CP033 same-owner giữ confirmed,
CP064 power1 ngoài tầm giữ control lịch sử, không phải cùngrun intervention. Bước tiếp độc lập:
EOF sau acceptedStart nhưng trước đủ99/clock; tái dùng CP050 delayed99/native175180107 và CP052–054
setup, đo survivor return/cleanup thật trước khi đổi policy. Không tìm lại packet consumer hoặc mở
mọi lifecycle race; CP072 phải audit thủ công README→PLAN→PROTOCOL→RESEARCH→raw/evidence.
