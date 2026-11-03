# CP076 — Back86 và tạo lại phòng trên cùng channel

Baseline CP075/PR111 merge `41f0fa9b5cf3f5f5e303d64cc0edf6ce73587699`;
KRE-039/[issue112](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/112).
Codex solo/tự kiểm, [review waived](../checkpoints/review-waiver.json), tích hợp theo [PR113](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/113).
[Contract và giới hạn](../../PROTOCOL.md#back86-membership), [câu hỏi](../../RESEARCH.md#gap-room-membership),
[thứ tự duy nhất](../../PLAN.md#current-order). Raw/scripts/seals: `local/research/codex/back86-membership-r1/`.

Tái dùng CP034 Back86/refresh241,91/loại sảnh, CP035 rejoin/sequence, CP050/052–054/074 cleanup,
CP016 typed81 và CP017–019 sequence fixture. `reuse-question.json`/`oracle-reuse-addendum.json` pin
nguồn KR/VN lobby/session, boom-rs/l10n/archive theo câu hỏi. Không RE lại binary writer/consumer/
auth/cache/clock; VN chỉ cho cách tách room membership khỏi TCP session, không port opcode/layout.

| Run / UTC | Kết quả có giới hạn | SHA256 run.json |
|---|---|---|
|back-control / 2026-11-02T21:59:22Z|Service cũ, Back86/UIsảnh nhưng Create không dialog/không81; chưa chứng minh81 bị bỏ qua.|`71479991bd751b37161b275bd29077af57d70618d5dd79aaecd2f8946380751c`|
|back-refresh / 2026-11-03T11:06:06Z|Wrapper chỉ refresh241/91: có81 lần2, marker cũ còn, không reply86/585.|`ea7a34e11d9fa62689ef1dbf47ccfe60c7a3fb0b0f141cb584d71cd0e342be8c`|
|back-fixed / 2026-11-03T11:10:27Z|Bản sửa đầu dừng vì sequence=None ở metadata create; TypeError trongservice.log/controllererror, không clientfailure.|`ae1adeefe44b7715c05cd6de02f4fbf027fda11b95e2e8baf89181b99b006be3`|
|back-fixed-v2 / 2026-11-03T11:15:41Z|Metadata/preflight đã sửa: actual81→86→585→106/110; dừng14s trướcmap/ảnhđen; partial,ECONNRESET saucontrollerstop.|`ed8a6f3c7f5c4a5774cbcdb65e71f1b9a3694f1c4e53ec0c486986ff1365fa61`|
|back-observed / 2026-11-03T11:20:30Z|Cùngservice/profilev2, chờnative map rồi render12s:81→86→585→106/110/238/117, phòngmộtKRLocal/Start/map.|`797493e9097db2f307f833695428536929da7762f90929063e8334e31a86725e`|

Cả5run chỉ một Start/preclock/actualB99hold→BEOF, không197/196/153/clock/bom/pickup/death; 175/180 là
forfeit design. NativeA107/ID2→0/chief1 là prerequisite tái dùng. Privateuser/net/mount/PID/chỉlo/
no route/NSSfiles-only/mộtlease; prefix/game/display riêng, nguồn binary/pack/DB giữ nguyên.
RuntimeA/B SHA `e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef`;
static `660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3`;
modO raw/base400000 `a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5`; RVA=VA−400000.

Bản sửa chỉ A ở một mình sau BEOF/resultcleanup: archive marker membership cũ, giữ channel/identity/
sequence/round/keyallocator/job history. Refresh241/91 rồi new81 dùng dynamicsequence; không gửi lại
fixture guestB đã ngắt. Biến thể không hỗ trợ được log, không đổi membership. Profile cũ thiếu sequence
bị từ chối trước mutation; export mới khai báo create55/player_A58/map70, **41fixture bytes/hash giữ**.
Không sửa fixtureJSON/raw cũ. `profile-metadata-correction.json`/`failed-fix.json` giữ nguyên cause,
source snapshot và điều kiện sửa; Fakeprofile test ban đầu không bắt được None, đã thêm realFixtureProfile.

Typed81 tái dùng native string/int getters: title hiện tại có SSO15 và heap16/cap31, nên assertion
SSO-only cũ không áp mọi title. Fixed-v2 cònECONNRESET227,734ms saucontrollerstop, giữserviceerror; không suy clientrejectroom. Nativegetter/allocator model đọc theo length/capacity có bounds và
cursor đầy đủ; không là native heap/runtime mới. Native86 success đọc11/11 và room/profile/map thực
thi riêng; ảnh được đọc thủ công, không dùng pixel/nonblack để kết luận. Chi tiết ở verification và
visual-observation.json; frame/time/SHA joins ở request-summary.json. Không claim nativeIDB sau
recreate từ getter chưa cài; proof không ghost là không gửiB106 + ảnh/phòng chỉKRLocal.

Kiểm: 4215relations; 1734nativeframer capture/CRCbad/partial,
240typedpacket cases, 120secure snapshots;79server/18checkpoint/7catalog tests,
docs/artifacts5/5/localhistory/diff. Models emulation và callback/UIruntime tách riêng. Run failed-fix
vẫn ghi failure, serviceerrors[] không phủ controllererror/traceback. Chưa guestleave/hostleavecònpeer/
activeleave/generaljoin/accountresume/fullauth/fullcodec/fullreset/fullmatch/K4/K5/K6a/LAN/Internet.

Recipeexact `<label>-recipe.json`, seal `<label>-hashes.json`. Control dùng controller
wine_probe_back86_membership.py, refresh dùng _back86_refresh.py, fix-v2 dùng _back86_membership_v2.py; observed dùng _back86_membership_observed.py;
run_probe.py label/CA-r2-stage-social.exe/320/controller/cleanup.gdb. Không overwrite label hoặc
chạy chồnglease. Collect→derive_roles→parse_fields/lifecycle→native_packets→verify sau completed.
AUD023 đọc thủ công quaCP076, periodic kếCP080. Tiếp guest Back/rejoin khi A còn trong phòng,
tái dùng CP035/control hiện tại, không nhận emptyroomcreate là membership tổng quát hoặc trận chơi.
