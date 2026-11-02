# CP063 — Report127 cũ/trùng và admission theo kết nối

Baseline CP062/PR85 merge `46cbbfd1e9c2f5b01133fbff8d6f104c52d6ab0f`;
KRE-026/[issue86](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/86).
Codex solo/tự kiểm; [review waived](../checkpoints/review-waiver.json), tích hợp theo [PR87](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/87).
[Contract/policy/giới hạn](../../PROTOCOL.md#report127-freshness),
[gap](../../RESEARCH.md#gap-contact-state), [thứ tự tiếp](../../PLAN.md#current-order).
Raw/script/capture/snapshot ở `local/research/codex/report127-freshness-r1/`.

`reuse-question.json` pin nguồn/build/locator: CP023 B53010 commonprefix, CP055 CMainSystem+110/
12580B0, CP056 marker, CP061–062 currentprofile/captures; boomkr-re, VN oc_e-game §5a-5c,
boom-rs/l10n/archive. VN giúp mô hình report/confirm/state/result, không cung cấp opcode/layout/
timeunit/validator KR. Tránh RE lại clock/writer/consumer/auth/cache; không thêm thân hàm native/patch.
`prior-order.json`:64 report active có tick không giảm từng kết nối, cùngtick/khác tọa độ đã có;
`native-prefix.json` kiểm getterprefix14 cả subtype39 và34 ngoài active, không giải toàn tail.

Runtime copy A/B `e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef`;
static copy `660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3`;
modO raw/base400000 `a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5`,
RVA=VA−400000. Bản nguồn/pack/DB không sửa; artifacts5/5 đã verify.

| Run02/11 UTC / run.json SHA256 | Kết quả và scope |
|---|---|
|control-current / `d3c17a22f3c1662f265d581f6b7f123b92ff21b89c19a59fa4f19ecb701198e0`|B mở connection7/reply3 rồi EOF trướcC2S; UI báo login không nối được, không tới report127. Giữ thất bại, không tính là control thành công.|
|control-warm / `1e7dec677d57da8093d5821d636be2e814ff4c5c40db4353fe823f84a264d42e`|Baseline snapshot nhận7 replay:3 duplicate vòng1,3 old report vòng2,1 duplicate21 vòng2. Old17/19 đặt marker mới/phát170; rank2 không có bom/death19 actual mới, actorA state0 ởviewA/state1 ởviewB trước175.|
|guard-current / `48ca7cd0f3471228a21c42511c83e9cfbd220031779bd06a7af31747e2663bec`|3 old +6 duplicate bị loại trước sideeffect;30 actual active report được nhận. Hai pickup/stat/reset/bom/death/result/return/refresh, native actorA state1 ở cảhai view trước175.|

Recipe hiện hành profile41/gamecopy CP059, namespace user/net/mount/PID chỉloopback/không route/
NSS files-only, một runtimelease/controller; prefix/display/wineserver riêng từng client, run tuần tự.
Retry dùng **bản copy prefix đã hoàn tất CP062** và UI confirm24→35s; hai biến đổi frontend cùng lúc,
không xác định nguyên nhân failure ban đầu. Control/fix dùng cùng frontend retry, policy-source riêng:
`control-source/` baseline bất biến và `runtime-source/` guard. Guard dừng bình thường sau refresh2+3s;
không kill namespace để giả completed. Không có controller/service/probe error ở run guard.

`runtime_replay.py` giữ exact actualTCP frame17/19/21 vòng1; gọi original dispatcher cho duplicate
ngay sau report thật và old reports sau actual39 cùng role vòng2. Provenance `actual_native_tcp`/
`dispatcher_replay` tách riêng; originSHA/fields/trigger/marker/death before-after được lưu. Không
mint competitor hoặc giả injection là TCP/nativewriter mới; bypass socket/framer/sequence admission.
Control là can thiệp nhóm replay; không kết luận old21 riêng gây rank. `bench.json` dùng actual127
với socket/broadcast stub và pickup synthetic xác nhận marker poisoning→guard reject; không runtime.

`verification.json`: **4270 relations**/source/seals/SHA,126 native actual127prefix,708 guard actualframer
cases,237 packet/12 native153,762 secure snapshots (354control+408guard). Native framer/getter/CRC/
blob/vector dùng binary KR với receiver/heap/counter model; không force counter runtime. 57 server
regression và18 checkpoint tests pass;7catalog/docs/local-history/diff kiểm lúc bàn giao. Checker không
suy ngữ nghĩa hoặc đóng gate. AUD019/CP060, periodic kế CP064.

Tái tạo với label/prefix mới: `run_probe.py LABEL CA-r2-stage-social.exe 330 wine_probe_report127_guard.py report.gdb`
(đường GDB tuyệt đối hoặc relative từrepo root); `seal_run.py LABEL` sau completion; parsers/verify ở
modulemode từrepo root. Control dùng `wine_probe_report127_control_late.py`, baseline snapshot;
prefix warmcopy/argv/hash trong preparation/recipes/seals. Không overwrite raw hoặc chạy chồng.

Policy tick/bodySHA là **design** hẹp, không xác thực state/server gốc. Equal distinctbody giữ;
wrap/half-range/freshconnection chỉ regression. Firstseenold/forgedforward/reset sameconnection/
actualTCP admission/fullvalidator/K5/K6a/LAN/Internet vẫn mở. Tiếp mixed-owner chain/control cách ô,
không lặp source t/k/auth/cache hoặc mở mọi biến thể replay trước khi vào chơi.
