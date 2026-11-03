# CP077 — Guest Back/rejoin khi host vẫn ở phòng

Baseline CP076/PR113 merge `e791e964f18e092d9fbd6737dd0ee916456755ce`;
KRE-040/[issue114](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/114).
Codex solo/tự kiểm, [review waived](../checkpoints/review-waiver.json), tích hợp theo [PR115](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/115).
[Contract và giới hạn](../../PROTOCOL.md#guest-back-rejoin), [câu hỏi](../../RESEARCH.md#gap-room-membership),
[thứ tự](../../PLAN.md#current-order). Raw/scripts/seals ở `local/research/codex/guest-back-rejoin-r1/`.

Nguồn trước RE: CP034 BBack86/107slot1ID2→0/Achief/refresh24191/normal-lobbykind1, CP035 B82/87/
585/rejoin/Ready và CP076 archive/currentseq/preflight/observer. Typed82/87/95/99/107/110/106 dùng lại
nativegetter/framer đã phục hồi. `reuse-question.json` pin KR data/VN lobby/session/boom-rs/l10n/archive;
VN chỉ giúp tách TCPsession và member, không copy opcode/ID/offset/layout. Không đọc lại thân hàm
native/writer/auth/cache/clock/locale; chỉ integration service khác với harness lịch sử.

| Run / UTC | Kết quả | SHA256 run.json |
|---|---|---|
|guest-control / 2026-11-03T11:31:30Z|Wrapper chỉ knownrefresh241/91 giữ productionmembership: actualB86 rồi82 lần2, service rejectvariant, AghostB/no107 và Bno87/585.|`f511f2b516d4eff99415999e14ec69faa71068871d6c0066ba9f82097e12a958`|
|guest-fixed / 2026-11-03T11:35:59Z|Service sửa: actualB86→A107/ID2→0/chief1; B82→87slot1/session1→585/profile/map, A nhận lạiB106; actualB95/A104→native121/99/clock cảhai.|`c4b7f965a0ca3a621ea0fbefbd122a54a1a701a615fb0b5c86f7b14d39ed3149`|

Hai run privateuser/net/mount/PID/chỉlo/no route/NSSfiles-only/lease đơn, A/Bprefix/game/display riêng.
Không EOF/99hold/Start trong control. Fixed chỉ mộtStart/clock sau rejoin, không bom/pickup/death/
result/return/hai trận. Sourcebinary/pack/DB nguyên; không patch/hook/force state mới.
RuntimeSHA `e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef`;
static `660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3`;
modO raw/base400000 `a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5`, RVA=VA−400000.

Implementation **design** mở thêm guest B idle với đúng Ahost/mapped còn lại: preflight profile rồi
archive marker B/giữ channel/seq/IDslot, gửi107/110 cho A, refresh241/91 cho B. New82 chỉ nhận sau
refresh và A vẫn mapped/live;87/106/110/238/117 dùng currentseq, B585 làm A nhận lạiB106 và B nhận
profileA trước map. Không khôi phục slot B bằng fixture đã rời trong khoảng lobby. Generalhostleave/
activeleave/accountresume vẫn reject/unknown; policy chọnhost/timing/authfixture không server gốc.

Exporter khai báo knownsequence roomlist_B49/join55/player_B58/guest_A64/guest_B64, raw41fixture
không thay hash. Privatecontrol đã córoomlist_B49 cho measurementrefresh; fixed thêm4sequence cònthiếu.
FixtureJSON/raw cũ giữ; metadata mới có preflight trước mutation. Control và fixed provenance của
refresh khác nhau: controlwrapper giữ membership, fixed production service thực sự cập nhật membership.

NativeA107 đọc6/rewind/final1/errors0/ID2→0/chief1; rejoin106B own-match0 trên A, nativeB87 full7/
slot1/session1→own106 own-match1/map117. Cùng8TCP/ID1,2/slot0,1, perreceiverseq tiếp tục, không auth/
reconnect mới. ActualB95/StartA104→121full129/99/197196153/clock mỗi phía1lần, keyđầu12345/rosterA-B.
Ảnh control/fixed được đọc thủ công; không lấy nútReady/OSinput hay renderedlobby làm requestproof.

Kiểm 1786relations, 648nativeframer CRC/partial/capture,
84typedpacket và 36secure snapshots;83server tests,18checkpoint/7catalog,
docs/artifacts5/5/localhistory/diff. Nativecallback/UIruntime riêng với receiver/heap/gettermodel emulation.
Chưa fullmatch/hairesultreturn/toànreset/fullauth/fullcodec/fullvalidator/hostlivepeer/activeleave/
accountresume/password/capacity/K4/K5/K6a/vanilla/LANInternet. Gate không tự đóng.

Recipeexact `<label>-recipe.json`, seal `<label>-hashes.json`; run_probe.py label/CA-r2-stage-social.exe/
240(control) hoặc300(fixed)/controller/guest.gdb. Control _guest_back_control.py dùng refresh_guest.py;
fixed _guest_back_fixed.py chạy productionservice, profile-fixed.json. Collect/derive/parse/native_packets/
verify sau completed, không overwrite label/chạy chồnglease. AUD023/CP076, periodic kếCP080.

Tiếp theoPLAN: guestleave/Ready/reset chỉ đo variant làm blocker; ưu tiên cùng-run Battle sau rejoin
để kiểm hiệu ứng/bom/17,19/result/return và nextkey, dùng route CP069/075 đã biết. Không gọi clock
sau rejoin là trận hoàn chỉnh, không quét lại toànclient hoặc mọi leavevariant trước tích hợp.
