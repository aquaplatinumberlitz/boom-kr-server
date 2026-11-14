# Lịch sử trạng thái theo checkpoint

Chuyển nguyên văn từ README ngày 12/11. Đây là trạng thái **tại thời điểm ghi** của từng mốc, không phải hiện trạng;
hiện trạng ở [Bảng điều hành](PLAN.md#dashboard). Mốc mới rời khỏi README thì nối vào đầu danh sách dưới.

## Các mốc trước

**CP299: Patrit14 đã có route terrain/item với account roster 2/4/8 qua hai ván/reset.**
[Bằng chứng và giới hạn](evidence/rust-service/g1-patrit-terrain.json) giữ đối chiếu spawn, thùng,
Roller cấu hình và MastTileA. Chủ dự án đã chấp nhận [Village10/Tomb12 2–6 người, bốn map còn lại 2–8](evidence/owner-directives/g1-map-roster-limits.json).
G1 và M8 còn theo [Bảng điều hành](PLAN.md#dashboard); phân phối drop và item chưa đo vẫn chưa được mở.


**CP297: terrain và ba item nền đã chạy trên Rust với roster account 2/4/8 người.**
Bomb/Fire/Roller, hidden reveal, stat theo nhân vật và bom power mới đã được đối chiếu client KR/Wine qua hai vòng
kết quả/reset. [Contract và giới hạn](protocol/terrain.md#rust-player-terrain),
[bằng chứng](evidence/rust-service/g1-terrain-players.json). **G1 vẫn mở:** tiếp nội dung sáu map, nhóm item còn lại
và nhân vật; M8 theo [Bảng điều hành](PLAN.md#dashboard).

**CP296: kim và cứu đã chạy trên Rust với account và peer KR thật.**
Hai client kiểm nhặt/tiêu kim, Ctrl khi cạn và kim mới sau reset. Ba client kiểm đồng đội cứu, cả hai peer nhận clear,
kim không bị tiêu bởi cứu và hai vòng kết quả/reset. [Contract và giới hạn](protocol/life-results.md#rust-needle-rescue),
[bằng chứng](evidence/rust-service/g1-life-actions.json). **G1 vẫn mở:** tiếp terrain/item nền, sáu map và roster,
nhân vật theo acceptance; M8 còn Internet transport mới và vận hành theo [Bảng điều hành](PLAN.md#dashboard).

**CP295: chủ phòng đã chọn map qua UI KR; peer và trận dùng đúng map mới.**
Catalog Factory07/Village10 có metadata nhất quán; native đổi map giữa các ván, đọc đúng spawn và kết quả/reset,
journal ghi đúng map. [Contract và giới hạn](protocol/load-phase.md#rust-room-map-selection),
[bằng chứng](evidence/rust-service/g1-map-selection.json). **G1 vẫn mở:** còn kim/cứu, terrain/item, map và roster
đầy đủ theo acceptance. M6 đã giao; M8 còn Internet transport mới và vận hành theo [Bảng điều hành](PLAN.md#dashboard).

**CP294: M6 đạt phạm vi giai đoạn 1 — lưu lịch sử trận và nhân vật, thưởng 0.**
Kết quả trên KR khớp DB; restart và đăng nhập mới giữ nguyên lịch sử và nhân vật. Writer-lock, retry, rớt giữa trận,
kill sau Start/kết quả và CLI có đối chứng riêng. [Contract và giới hạn](protocol/life-results.md#rust-match-journal-serving),
[bằng chứng](evidence/rust-service/m6-journal-serving.json). Giai đoạn 1 còn G1 và M8 theo
[Bảng điều hành](PLAN.md#dashboard); inventory/EXP/tiền tệ thuộc giai đoạn sau.

**CP293: lựa chọn nhân vật đã được lưu và khôi phục trên client KR thật qua account DB.**
Client chọn Dizni, chơi hai ván; sau restart và đăng nhập mới, phòng/Start nhận lại nhân vật mà không cần chọn lại.
[Contract và giới hạn](protocol/auth-session.md#rust-persistent-character),
[bằng chứng](evidence/rust-service/m6-character-serving.json). **M6 vẫn mở:** còn nối lưu trận/kết quả vào serving,
commit trước reply và nghiệm thu durability trên đường chơi. G1 và M8 tiếp theo [Bảng điều hành](PLAN.md#dashboard).

**CP292: M6 đã có backend và worker lưu lịch sử trận, preference nhân vật và thưởng 0.**
SQLite có khóa trận ổn định, retry không trùng, cancellation sau restart và kiểm kill ngay sau commit; CLI đọc lịch sử
account/phòng. [Contract/giới hạn](protocol/life-results.md#rust-match-store-backend),
[bằng chứng](evidence/rust-service/m6-match-store.json). **M6 chưa đóng:** còn nối commit trước kết quả trên wire,
lưu nhân vật từ phòng và native relogin/persistence. G1 adapter Factory07 đã giao; chọn map, terrain/item và các map khác
vẫn mở theo [Bảng điều hành](PLAN.md#dashboard). M8 còn triển khai Internet transport mới và vận hành.

**CP291: sửa kiểm provenance source test tích hợp, giữ nguyên bằng chứng Factory07.**
Checker nay kiểm source test Rust đã pin bằng đúng declaration và blob tại commit công bố checkpoint gốc;
[giới hạn/correction](evidence/checkpoints/integration-test-source-correction.json). Không sửa checkpoint/raw cũ.
Lát adapter G1 đã có byte baseline và native Factory07; [contract](protocol/load-phase.md#rust-map-profile-adapter)
giữ giới hạn terrain/item, chọn map và các map còn lại. M6 backend đang ở nhánh riêng, chưa nối native persistence.
Theo [Bảng điều hành](PLAN.md#dashboard), tiếp G1 song song M6 rồi M8; transport mới chưa kiểm Internet thật.

**CP290: adapter map đã qua byte baseline và trận Factory07 trên client KR thật.**
Profile pack giữ nguyên packet ExVillage01; Factory07 đã qua lookup tên, loading, spawn, hai kết quả/reset và vòng tiếp
trên Wine. [Contract/giới hạn](protocol/load-phase.md#rust-map-profile-adapter),
[bằng chứng](evidence/rust-service/g1-map-adapter.json). Đây là lát adapter của G1; terrain, item, chọn map trong phòng
và các map còn lại vẫn mở. M6 đang làm backend trên nhánh riêng, chưa nghiệm thu đường kết quả/persistence.
Theo [Bảng điều hành](PLAN.md#dashboard), tiếp G1 song song M6 rồi M8; transport mới chưa kiểm Internet thật.

**CP289: N1R-b đạt các tiêu chí hành vi của relay nhiều người trong phạm vi đã đo.**
Native bốn/tám người, roster thưa và survivor đã có bằng chứng; bổ sung đăng nhập mới từ DB sau restart,
đối chứng gói cũ, chung nguồn IP và test QUIC mạo nhận route với gói hiện hành. [Ma trận acceptance/giới hạn](evidence/rust-service/n1rb-controls-acceptance.json),
[contract](protocol/udp-peer.md#rust-authenticated-serving-relay), [cấu hình relay](OPERATIONS.md).
G1/M6/M8 còn mở theo [Bảng điều hành](PLAN.md#dashboard). Transport xác thực mới chưa đo Internet thật;
M8 phải kiểm đường triển khai đó theo đích giai đoạn 1. Hạ tầng và raw giữ local; [lịch sử](HISTORY.md).

**CP288: relay xác thực đã có nghiệm thu tám người, roster thưa và chơi tiếp sau rớt mạng.**
Client KR thật trên Wine đã qua trận tám người, một người rời trước ván kế và một endpoint rớt giữa trận;
sáu người còn lại dùng bom, tới kết quả/reset và vào trận tiếp. [Contract/giới hạn](protocol/udp-peer.md#rust-authenticated-serving-relay),
[bằng chứng](evidence/rust-service/n1rb-native-rosters.json). **N1R-b vẫn mở** cho reuse với stale/current,
restart và giả mạo chung IP theo [Bảng điều hành](PLAN.md#dashboard). G1/M6/M8 còn mở;
packet 59 ở lượt lỗi đầu vẫn chưa rõ nguyên nhân. Hạ tầng/raw giữ local; [lịch sử](HISTORY.md).

**CP287: relay có xác thực đã nối vào account/phòng và chạy bốn client KR thật.**
Hai ván có kết quả trái nhau, reset và clock ván thứ ba đã được đối chiếu bằng capture, native state và getter.
[Contract/giới hạn](protocol/udp-peer.md#rust-authenticated-serving-relay),
[bằng chứng](evidence/rust-service/n1rb-owned-serving-four.json). **N1R-b chưa đạt**: còn tám người,
roster sau rời/rớt mạng, restart và đối chứng giả mạo chung IP theo [Bảng điều hành](PLAN.md#dashboard).
G1/M6/M8 còn mở. Hạ tầng thực và raw chỉ giữ local; [lịch sử](HISTORY.md).

**CP286: đường companion có xác thực đã có phép kiểm tương thích hai client KR thật.**
UDP giữ nguyên byte qua QUIC tới native consumer; hai kết quả/reset và Start tiếp đã được đối chiếu bằng capture,
native state và getter. [Phạm vi/giới hạn](protocol/udp-peer.md#rust-authenticated-companion-foundation),
[bằng chứng](evidence/rust-service/n1rb-companion-foundation.json). Đây là nền transport với route tĩnh trong lab;
**N1R-b chưa đạt**. Tiếp nối quyền account/phòng, roster bốn/tám người và các ca reuse/restart/spoof theo
[Bảng điều hành](PLAN.md#dashboard). M5b đã giao PR #379; G1/M6/M8 còn mở.
Hạ tầng thực và raw chỉ giữ local. Lịch sử tại [HISTORY](HISTORY.md).

**CP285: M5b đã đủ đường account mới và đăng nhập KR sau khi DB/hàng đợi phục hồi.**
Ba account tạo khi service đang chạy đã chơi hai trận; ca chung nguồn TCP giữ đúng danh tính. Lượt bổ sung xác nhận
process KR thứ tư đăng nhập mới và nhận hồ sơ DB sau lỗi; CP284 trước đó mới kiểm phục hồi ở backend.
[Contract/giới hạn](protocol/auth-session.md#rust-db-account-profiles),
[bằng chứng bổ sung](evidence/rust-service/m5b-native-recovery.json). M5b đạt acceptance giai đoạn 1; relay nhiều
người, G1, M6 và M8 còn mở. Tiếp N1R-b theo [Bảng điều hành](PLAN.md#dashboard).
Hạ tầng thực và raw chỉ giữ local. Lịch sử tại [HISTORY](HISTORY.md).

**CP284: M5b đã đạt hồ sơ account từ DB và đường chơi cơ bản cho account tạo khi service đang chạy.**
Ba account mới đã qua sảnh/phòng và hai trận liên tiếp; một lượt riêng kiểm chung nguồn TCP mà không lẫn danh tính.
Trong ca DB bận và hàng đợi account đầy, cả ba client vẫn tạo bom/nổ; đăng nhập mới bị từ chối rõ ràng và phục hồi
sau lỗi. [Contract và giới hạn](protocol/auth-session.md#rust-db-account-profiles),
[bằng chứng](evidence/rust-service/m5b-dynamic-accounts.json). M5b đạt phạm vi giai đoạn 1; relay nhiều người,
G1, M6 và M8 còn mở. Tiếp N1R-b theo [Bảng điều hành](PLAN.md#dashboard).
Hạ tầng thực và raw chỉ giữ local. Lịch sử tại [HISTORY](HISTORY.md).

**CP283: M7e đã có roster hiện hành và người còn lại tiếp tục sau rớt mạng giữa trận trên client KR thật.**
Start/stat dùng nhân vật đã chọn; đã kiểm ba người, sáu người, slot thưa và hai lần mất endpoint với roster4→3→2,
kết quả/reset và ván kế. [Contract và giới hạn](protocol/room-membership.md#rust-selected-player-rounds),
[bằng chứng](evidence/rust-service/m7e-selected-rosters.json). M7 đạt phạm vi đã đo; hồ sơ account bất kỳ và
native DB/hàng đợi là việc tiếp theo ở M5b theo [Bảng điều hành](PLAN.md#dashboard).
Hạ tầng thực và raw chỉ giữ local. Lịch sử tại [HISTORY](HISTORY.md).

**CP282: M7d đã đạt ownership account/transport/membership trong nhiều phòng managed.**
Native revoke giữa trận chỉ ảnh hưởng phòng của account đó; phòng kia tiếp tục tới kết quả. Các ca transport trùng,
sai quyền/phòng, expiry và DB-busy đã kiểm riêng trên SQLite/TCP. [Contract và giới hạn](protocol/room-membership.md#rust-managed-account-ownership),
[bằng chứng](evidence/rust-service/m7d-account-ownership.json). Client bị revoke có thể abort; native DB-queue fault
và hồ sơ account bất kỳ còn thuộc M5b. Tiếp M7e → M5b theo [Bảng điều hành](PLAN.md#dashboard); toàn M7 chưa đóng.
Hạ tầng thực và raw chỉ giữ local. Lịch sử tại [HISTORY](HISTORY.md).

**CP281: M7c đã có lựa chọn nhân vật/đội, Ready và lỗi phòng đầy trên client KR thật.**
Lát cắt phòng chờ hỗ trợ Dao0/Dizni1 và nút đội A0/B3; peer thấy đúng, đổi lựa chọn xóa Ready rồi có thể Ready lại.
Ba client đã kiểm full-error/vacancy không slot ma. [Contract/giới hạn](protocol/room-membership.md#rust-managed-player-choices),
[bằng chứng](evidence/rust-service/m7c-room-choices.json). Lựa chọn hiện chưa ghép battle profile; Start/stat theo
nhân vật, roster tự do và rời giữa trận thuộc M7e. Tiếp M7d → M7e → M5b theo [Bảng điều hành](PLAN.md#dashboard).
Hạ tầng thực và raw chỉ giữ local. Lịch sử tại [HISTORY](HISTORY.md).

**CP280: phòng managed Rust đã có lifecycle native cho roster 2/4/8 trong phạm vi cấu hình đã đo.**
Hai phòng bốn người và một phòng tám người đã qua kết quả/reset/trận tiếp; EOF/bom chờ được cách ly với phòng còn chơi.
[Contract và giới hạn](protocol/room-membership.md#rust-managed-team-rosters),
[bằng chứng](evidence/rust-service/m7b-team-rosters.json). M7b đạt tập profile này; M7c/d, hồ sơ account tổng quát,
relay nhiều người và vận hành còn mở. Thứ tự ở [Bảng điều hành](PLAN.md#dashboard).
Hạ tầng thực và raw chỉ giữ local. Lịch sử tại [HISTORY](HISTORY.md).

**CP279: hai phòng duel Rust đã chạy độc lập trên bốn client KR thật.**
Room chưa đầy vẫn Ready/loading, kết quả/reset và mở trận tiếp theo roster thực; EOF/bom chờ của phòng này được
cách ly trong khi phòng kia tiếp tục. [Contract và giới hạn](protocol/room-membership.md#rust-managed-duel-lifecycle),
[bằng chứng](evidence/rust-service/m7b-duel-lifecycle.json). Chỉ lát cắt duel được nhận; M7b còn roster/layout
khác, M7c/d và M5 tổng quát còn mở. Thứ tự ở [Bảng điều hành](PLAN.md#dashboard).
Hạ tầng thực và raw chỉ giữ local. Lịch sử tại [HISTORY](HISTORY.md).

**CP278: đã sửa kiểm provenance cho source Rust và census tại checkpoint công bố.**
[Quy tắc và giới hạn](evidence/checkpoints/README.md#publication-bound-source-history): tìm đúng blob tại lần công bố,
không sửa record hoặc raw cũ. Occupancy phòng đã có [native proof](protocol/room-membership.md#rust-room-population)
ở CP277. M7b còn lifecycle trận và cách ly round/UDP; toàn M7 và profile account bất kỳ chưa đạt.
Tiếp lifecycle M7b theo [Bảng điều hành](PLAN.md#dashboard).

**CP277: danh sách phòng Rust đã hiển thị số người và sức chứa từ membership thật.**
Client KR trên Wine/LAN giả lập đã thấy một phòng 1/8 → 2/8 → 1/8 và phòng kia giữ 1/8; người cuối rời xóa đúng phòng.
[Contract và giới hạn](protocol/room-membership.md#rust-room-population),
[bằng chứng](evidence/rust-service/m7b-room-population.json). Chỉ phần occupancy đã đạt; M7b còn lifecycle trận
và cách ly round/UDP. M7a giữ nguyên phạm vi đã nghiệm thu; toàn M7 và profile account bất kỳ chưa đạt.
Tiếp lifecycle M7b theo [Bảng điều hành](PLAN.md#dashboard).

**CP276: M7a đã nghiệm thu tạo, chọn và rời nhiều phòng bằng account trên client KR thật.**
Bốn client trên Wine/LAN giả lập đã dùng hai phòng riêng, đúng slot/chủ phòng, chuyển phòng, EOF/revoke và xóa phòng rỗng.
[Contract và giới hạn](protocol/room-membership.md#rust-managed-room-selection),
[bằng chứng](evidence/rust-service/m7a-room-selection.json). Gameplay phòng động, occupancy và profile account
bất kỳ còn mở; toàn M7 chưa đạt. Tiếp M7b theo [Bảng điều hành](PLAN.md#dashboard).

**CP275: Village10 lookup đúng mã 1104, nhưng profile cũ chưa đủ để load map.**
[Contract và giới hạn](protocol/load-phase.md#village10-map-selection): cần đồng bộ metadata; chưa có trận Village10.
R1 đã đạt hồi quy native, relay đạt prototype hai người. M5 tổng quát, relay bốn/tám người và vận hành còn mở.
Bước tiếp là acceptance M7 theo [Bảng điều hành](PLAN.md#dashboard); lịch sử tại [HISTORY](HISTORY.md).
Hạ tầng thực và raw chỉ giữ local.

**CP270: R1g đã kiểm trần admission và allowlist độc lập số profile.**
[Contract/giới hạn](PROTOCOL.md#rust-r1g-admission), [evidence](evidence/rust-service/r1g-admission.json).
Cấu hình cho phép1–1024transport và1–256entry allowlist; vẫn kiểm source/lease/profile riêng. Giá trị cấu hình cũ giữ nguyên.
105Rust/fmt/clippy/lockedbuild đạt;306TCPlegacy/3169+24frame khớp CP269;10ca admission đạt, gồm65socket giữ đồng thời,
kết nối66 bị chặn và phục hồi permit. Đây chưa phải kiểm50người chơi KR hoặc native candidate.
R1f đã merge PR347; R1g đang bàn giao PR. Tiếp relay opt-in và native cuối R1 theo [dashboard](PLAN.md#dashboard).
Full M5/M7/M8 còn mở; hạ tầng chỉ local. Mốc cũ ở [HISTORY](HISTORY.md).


**CP269: R1f đã tách account profile/endpoint khỏi IP chọn identity, kiểm source/TCP đạt.**
[Contract/giới hạn](PROTOCOL.md#rust-r1f-session-binding), [evidence](evidence/rust-service/r1f-session-binding.json).
Profile lấy từ PlayerId của frontend/session lease; hai account chung IP vẫn giữ profile và membership riêng.
104 test Rust/fmt/clippy/locked build đạt;306 ca TCP legacy và3169+24frame khớp CP265; thêm59shared-IP và16account serving.
Endpoint direct theo channel hiện hành, vẫn dùng cổng99; chưa relay/NATlearning hoặc native shared-NAT gameplay.
Thiết kế/schema CP267–268 đã merge; R1f đang bàn giao riêng. Tiếp R1g, relay opt-in và native cuối R1 theo
[dashboard](PLAN.md#dashboard). Full M5 còn mở; thông tin VPS chỉ local. Mốc cũ ở [HISTORY](HISTORY.md).


**CP268: sửa checker cho checkpoint thiết kế có tài liệu chuẩn đã pin.**
CP267 là design/docs, không có binary mới; lần kiểm lịch sử phát hiện schema cũ bắt artifact cho cả trường hợp đó.
[Correction/giới hạn](PROTOCOL.md#documentation-checkpoint-identity), [evidence](evidence/rust-service/documentation-identity.json).
Record CP267 đã commit giữ nguyên. Checker chỉ nhận danh tính từ docs chuẩn có SHA khi mọi finding là design/documentation,
status design/candidate/unknown; runtime/static/test vẫn cần artifact. 57 test checkpoint đạt, không native/gate claim.
[Thiết kế endpoint](RUST_SERVICE_ARCHITECTURE.md#relay-endpoint-design) giữ nguyên; tiếp R1f theo
[dashboard](PLAN.md#dashboard). Relay chưa triển khai; scope N1/K6c/K7/M5 không đổi. Hạ tầng chỉ local.
Mốc cũ ở [HISTORY](HISTORY.md).


**CP267: đã chốt thiết kế endpoint/relay sau N1b; tiếp R1f session binding.**
[Thiết kế/module/lifecycle](RUST_SERVICE_ARCHITECTURE.md#relay-endpoint-design),
[phạm vi](PROTOCOL.md#rust-relay-endpoint-scope), [evidence/reuse](evidence/rust-service/relay-endpoint-design.json).
Giữ direct legacy; relay opt-in dùng socket từng hướng, forward nguyên payload từ reverse socket là phương án cần đo.
R1f và relay implementation là hai lát cắt riêng theo [dashboard](PLAN.md#dashboard).
Chưa triển khai relay hoặc có native proof mới. N1b âm đã giao [PR345](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/345);
N1 một phía tới được/K6c/K7 giữ scope. M5 tổng quát và shared-NAT còn mở. Thông tin VPS chỉ local.
Mốc cũ ở [HISTORY](HISTORY.md).


**CP266: N1b đã đo đủ hai lượt; UDP trực tiếp thất bại khi cả hai phía đổi cổng.**
[Đối chiếu native/capture và giới hạn](PROTOCOL.md#rust-n1b-dual-nat), [raw/hash](evidence/rust-service/n1b-dual-nat.json).
Hai KR/Wine vẫn đi qua account/phòng, hai kết quả/reset và Start3 qua TCP, nhưng không có ingress/helper movement UDP
trong các cửa sổ đo. Lượt đối chứng thấy cả hai cổng sau NAT khác packet99; native chỉ gửi tới endpoint153,
không có UDP tới service để học endpoint. Đây là kết quả âm có bằng chứng, chưa phải hỗ trợ hai NAT hoặc relay.
R1e đã merge [PR344](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/344). Tiếp thiết kế relay/endpoint trước R1f
ở [dashboard](PLAN.md#dashboard); không chạy lượt N1b thứ ba. N1 một phía tới được/K6c WAN giả lập giữ scope.
Shared-NAT, full M5 và native cuối R1 vẫn chưa đạt. Hạ tầng chỉ local; mốc cũ ở [HISTORY](HISTORY.md).


**CP265: R1e nền registry và listing đã kiểm, profile cũ giữ bytes.**
[Contract/giới hạn](PROTOCOL.md#rust-r1e-registry), [evidence](evidence/rust-service/r1e-registry.json).
Registry giữ các room runtime riêng; listing91 dựng từ entry/title hiện hành, batch có origin/recipient fence.
101 test Rust/fmt/clippy/locked build và306 ca TCP đạt;3169 frame cũ +24 first-vacant khớp CP264.
Hai room/queue/listing được kiểm unit; serving vẫn phòng fixed cũ, chưa tạo/chọn nhiều phòng tổng quát M7.
R1d đã merge PR343. CP265/AUD082 đang bàn giao; [dashboard](PLAN.md#dashboard) tiếp N1b trước R1f.
N1 vẫn đạt đúng scope một phía tới được; N1b/shared-NAT/native cuối R1/fullM5 chưa đạt. Hạ tầng chỉ local.
Mốc cũ ở [HISTORY](HISTORY.md).


**CP264: R1d header chuẩn và builder theo opcode đã kiểm; N1 được ghi nhận đạt phạm vi đã chốt.**
[Contract R1d](PROTOCOL.md#rust-r1d-wire-builders), [bằng chứng](evidence/rust-service/r1d-wire-builders.json).
96 test Rust/fmt/clippy/locked build;306 ca TCP đạt,3169 frame phòng cũ và24 frame first-vacant khớp baseline CP263.
Alias JSON chỉ còn adapter nạp cấu hình; runtime dùng template có kiểu/opcode và header chuẩn. Chưa native binary mới.
[N1](PROTOCOL.md#rust-n1-acceptance) đạt **ít nhất một phía tới được theo địa chỉ server báo**, dựa CP261 và xác nhận4G
của chủ dự án. N1b hai phía NAT đổi cổng còn mở; shared-NAT gameplay và full M5 vẫn chưa đạt.
R1c đã merge [PR338](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/338). Audit nhóm AUD081; R1d đang bàn giao PR.
Việc kế tiếp theo [Bảng điều hành](PLAN.md#dashboard): R1e → N1b → R1f/R1g → native cuối R1.
Mốc cũ ở [HISTORY](HISTORY.md). Thông tin hạ tầng giữ local.



**CP263: R1c đã tách player identity khỏi luật cấp slot và dùng chung template 106.**
[Contract/giới hạn](PROTOCOL.md#rust-r1c-player-slots), [bằng chứng](evidence/rust-service/r1c-player-slots.json).
Mặc định legacy giữ byte; `first_vacant` cấp slot độc lập, người tạo là host. Runtime không chọn fixture theo alias identity.
94 test Rust, 305 ca TCP và 3.189 frame phòng cũ khớp baseline CP262; 30 fixture 106 không có chênh lệch ngoài trường đã biết.
Ca TCP mới kiểm player 2 tạo phòng, player 1 vào slot 1, chuyển host/vào lại và năm guard trước bind.
[PR337](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/337) đã merge R1b; tiếp R1d theo PLAN.
Chưa chứng nhận native binary mới, chưa general profile/full M5 hoặc shared-NAT; **N1 vẫn mở** điều kiện uplink.

**CP262: R1b đã chuyển luật phòng sang chính sách có tên, giữ byte của ba profile hiện có.**
[Contract/giới hạn](PROTOCOL.md#rust-r1b-policies), [bằng chứng](evidence/rust-service/r1b-policies.json).
`ResultPolicy`, `DeparturePolicy`, `BombPolicy`, `LifePolicy`, `TeamPolicy` thay các cờ nội bộ; fixture `rank` chỉ cấp bytes.
90 test Rust/fmt/clippy/locked build,298ca TCP mỗi binary và3.189frame phòng khớp byte/hash với baseline.
JSON cũ giữ nguyên; chưa đổi identity/slot/registry/transport của R1c–g hoặc nghiệm thu native lại binary mới.
Tiếp R1c; **N1 vẫn mở** vì chưa xác nhận môi trường di động/ISP-CGNAT, R1f vẫn phải sửa ca chung NAT.

**CP261: hai KR thật đã qua account → phòng/basic → hai kết quả đối nghịch/reset → Start3 trên đường Internet đã đo.**
[Contract/giới hạn](PROTOCOL.md#rust-internet-observation), [bằng chứng](evidence/rust-service/internet-observation.json).
Native login field51 điều khiển ngưỡng chờ channel: fixture0 ghi đè cache thành0 gây timeout; control30000 vào sảnh ở cảhai.
Control là policy server được khai rõ, clientcopy4b giữ nguyên. Runtime ARM/source giữ CP260; frontendTTL120 là config đã hỗ trợ.
752TCP/106reply/91raw,78nativeUDP ingress/77helper-apply/78join các capture,2.286copiedgetters,cleanupđạt.
Cổng A tự báo99 khác cổng sau NAT; B thực gửi về nguồn quan sát được, không chỉ cổng153. Ca chungNAT: A giữ sảnh8s,
B đúng frontend/session nhưng bị binding theoIP/metadata từ chối; R1f còn phải sửa.
**N1 chưa đóng:** chưa có nguồn xác nhận uplink di động/CGNAT của ISP như môi trường tối thiểu trong PLAN; không suy từ IP.
Kết quả chỉ chứng nhận đường đã đo; chưa chứng nhận mọi NAT/ISP hoặc sharedNATplay. K6c/K7scopegiữ; detailsVPS chỉlocal.

**CP260: tiền kiểm Internet đã thông; native account qua NAT nhận auth thành công, N1 vẫn mở.**
[Contract và giới hạn](PROTOCOL.md#rust-controlled-network), [bằng chứng](evidence/rust-service/controlled-network.json).
Rust có chế độ mạng controlled với allowlist và mapping metadata login tường minh; quyền account/session vẫn theo transport thật.
90 test Rust, hai bộ 59 ca account TCP, 9 guard mạng và 5 guard metadata đạt. Hai KR/Wine nhận auth result0 qua mạng thật,
nhưng lỗi chờ channel trước own291/phòng. R7 xác nhận modal timeout/đóng client xảy ra trước HTTP JSON rỗng;
lỗi HTTP là nhánh phát sinh sau đó, chưa phải nguyên nhân mất channel. Guard cũ cũng chưa giúp vào sảnh.
Chưa có 99/153, UDP gameplay, hai kết quả/reset hoặc ca chung NAT; chưa chọn direct/học endpoint/relay.
Thông tin hạ tầng/raw giữ riêng trong `local/`. **Internet thật chưa kiểm ở mức game KR**; K6c/K7 WAN giả lập giữ nguyên.

**CP259: đã đồng bộ main PR329/PR330 và chuyển ưu tiên sang N1 theo chỉ đạo mới.**
[Quyết định/giữ WIP](evidence/owner-directives/pr329-integration-order.json), [kiểm sau gộp](evidence/rust-service/pr329-integration.json).
Fmt, clippy, 86 test Rust, locked build và 59 ca account TCP đạt; 143 frame phòng khớp hash với CP258.
Bằng chứng native CP258 vẫn gắn binary đã đo, chưa tự chứng nhận binary sau `RoomRules`.
WIP projector được giữ trong snapshot/stash, chưa chạy native hoặc merge; thứ tự ở [PLAN](PLAN.md#current-order).


### Mốc trước — phạm vi CP258 vẫn giữ nguyên


**CP258: account → phòng cố định → hai trận basic/kết quả/reset → Start3 đã chạy trên hai KR thật.**
[Contract/giới hạn](PROTOCOL.md#rust-account-basic), [raw/hash](evidence/rust-service/README.md#cp258).
Account sở hữu đúng ID/profile/channel/slot; revoke/expiry/disconnect khi bom chờ đưa survivor về phòng, không nổ muộn.
Manualfresh B dùng process/backend/frontend mới vào lại slot và trận mới; UDP nguyên bản cũ bị native key-drop, currentmovement áp dụng.
5run/280raw/375exactTCP/cleanup0;84Rust/59account-basicTCP/16lobbyregression. AffectedB revoke/expiry có lỗi phiên/Winefatal,
**chưa graceful recovery; M5 tổng thể vẫn mở** (generalprofile/nativeDBqueuefault). M6/M7/M8 riêng, không dependency lát cắt này.
Audit nhóm AUD076; không lịch4CP. Internet thật chưa kiểm; VPS preflight riêng chưa là nativeInternetE2E.

### Các mốc trước

Đã chuyển sang [docs/HISTORY.md](HISTORY.md). Hiện trạng và việc kế tiếp ở [Bảng điều hành](PLAN.md#dashboard).



**CP256: fresh login account trên KR thật và chặn replay request cũ đã đạt; M5 vẫn mở.**
[Contract/giới hạn](PROTOCOL.md#rust-account-fresh-login), [raw/hash](evidence/rust-service/README.md#cp256).
A mới/process-session-frontend mới vào đúng KRLocal, B giữ phiên; hai request native cũ phát lại bị từ chối, giữ sảnh8s.
122raw/70,794s/28exactTCP/cleanup0;84Rusttests. Sửa false ClockRollback ở CLI login khi worker tiếp tục kiểm phiên.
**Chưa account → phòng → Battle → kết quả/reset → trận tiếp**; đây là ưu tiên kế theo [chỉ đạo](evidence/owner-directives/account-basic-end-to-end.json).
AUD074 quaCP256; [chỉ đạo mới](evidence/owner-directives/feature-group-audits.json) bỏ lịch4CP, audit khi xong nhóm tính năng. Internet thật chưa kiểm.


**CP255: ba ca account native âm tính/lifecycle đã đạt, M5 vẫn mở.**
[Contract/giới hạn](PROTOCOL.md#rust-account-native-lifecycle), [raw/hash](evidence/rust-service/README.md#cp255).
A dùng correlator đang chờ của B bị từ chối, B vẫn claim đúng; revoke A khi đang ở lobby và phiên A tự hết hạn60s
đều đóng đúng channel A, B giữ8s. Ba run/280raw/49exactTCP/cleanup0; A có notice error10, chưa kiểm đóng modal/retry.
Không đổi server đã nghiệm thu CP254/PR326; chưa generic profile, freshlogin/replay/room-game hoặc fullM5.
Next theo PLAN, audit kếCP256. Internet thật chưa kiểm.


**CP254: hai KR thật đã đăng nhập vào sảnh bằng hai account SQLite trên adapter Rust.**
[Contract/giới hạn](PROTOCOL.md#rust-account-serving), [raw/hash](evidence/rust-service/README.md#cp254).
Native auth137/own291128/ID-name-echo/render/argv đều đối chiếu,18 reply TCP khớp;100raw/41,852s/cleanup0.
82 Rust tests,9ca thư viện/8DB,16ca TCP và11guard startup đạt. Hai profile cố định, chưa room/game hoặc full M5.
Next: native negative/lifecycle theo PLAN; M4 Free vẫn blocked. Internet thật chưa kiểm.


**CP253: M5 đã có thư viện nối session/account vào admission KR; chưa nối listener.**
[Contract/giới hạn](PROTOCOL.md#rust-account-bridge), [raw/hash](evidence/rust-service/README.md#cp253).
Session backend cấp token launcher riêng, dùng một lần; chuyển login → redirect → channel theo owner/generation,
kiểm lại DB khi thực thi. 7 ca thư viện/6 DB thật và 3 body native CP217 phát lại ngoại tuyến đạt; 80 test Rust đạt.
Chưa có native KR chạy qua cầu nối này, chưa full authentication hoặc đóng M5. Service diagnostic giữ hành vi cũ.

CP252 đã merge [PR324](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/324)/`e17b78b`.
PLAN tiếp adapter listener/profile/launcher nhỏ rồi native hai tài khoản, failure/ownership/lifecycle; không làm lại DB/KDF.
M4 vẫn thiếu Free 1v7 native và dừng retry tới khi có observable mới. Audit kế CP256. **Internet thật chưa kiểm**.


**CP252: Rust đã kiểm native visitor thứ chín bị từ chối vào phòng đầy; M4 vẫn mở.**
[Contract và giới hạn](PROTOCOL.md#rust-ninth-admission), [raw/hash](evidence/rust-service/README.md#cp252).
Một client KR thật làm visitor identity9, tám member TCP tổng hợp. Request82 nhận lỗi87 mã2, native đọc1/1/errors0;
đóng notice bằng Return, vẫn ở sảnh, không nhận slot/phòng. 19raw/35,408s/cleanup0;1.384TCP/249reply được đối chiếu.
73 test Rust, 6 ca TCP và4 guard trước bind đạt. Đây không phải chín client native cùng lúc hoặc general admission.

AUD072 sửa claim argv CP250: scalar32 không được launch cho từng role; thực tế cũ là token13 ký tự.
Control mới xác nhận argv32 thật nhưng Free vẫn dừng ở D sau17 trướcphòng (66raw/48,888s); không claim startupfix.
M4 còn Freehaioutcomes/selector0nativepersist. Tiếp M5 worker-to-KR/ownership bridge độc lập theo PLAN;
backend account/SQLite/worker/idleTTL/rate đã có. AUD073qua252/audit kếCP256. Internet thật chưa kiểm.


**CP251: Rust có fixed1v7 và native negative Manner lệch đội; M4 vẫn mở.**
[Contract/giới hạn](PROTOCOL.md#rust-eight-teams), [raw/hash](evidence/rust-service/README.md#cp251).
Tám KR thật/Wine/copy8d: native8người/2đội/selector1→nhánh lệch đội→return7, không104/Battle;
90raw/127,113s/cleanup0/1.513TCP/266reply. Free1v7 đã có code/TCP, nhưng hai lượt native dừng sau17 trướcphòng
(copy4bD/copy8dG); chưa có Freehaioutcomes/selector0nativepersist. Giữ failedraw, không retry vôhạn.
71Rusttests/35eightTCP/8prebind/32fourTCP đạt. Python team reference CP213 đã merge [PR322](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/322).
CP250 merge [PR321](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/321)/`de84217`; Manner4v4 haioutcomes/Start3 giữscope đãđạt.
Tiếp native9admissionbridge/consumer độc lập; Free startup chỉ mở lại khi có observable mới. M5backend đã có/bridge riêng;
AUD071qua248/audit kếCP252. Internet thật chưa kiểm.
CP251 đã merge [PR323](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/323)/`9b37b14`. Nhánh admission9 kế tiếp đã có WIP guard/model tests, chưa integratedTCP/native hoặc PR mới.
AUD072 sửa provenance argv: token32 ở CLI chung chưa tới lệnh launch từng role; [correction](PROTOCOL.md#argv-provenance-correction). Bằng chứng gameplay cũ giữ nguyên, không claim startupfix.


**CP250: tám client KR thật đã chơi qua hai kết quả 4v4 và Start3 trên Rust.**
[Contract/giới hạn](PROTOCOL.md#rust-eight-baseline), [raw/hash](evidence/rust-service/README.md#cp250).
Cùng8/Wine/Manner1/32TCP: loading8,56chiều movement,stat/bom8owner,7peerlife/native1751082-inner1072/1803,
64slotreset/7Ready/Start2–3.156raw/384,236s sealed/cleanup0;5.019TCP/912reply/53.616copiedgetters riêng.
Đã xem24ảnh: cả8bảng điểm hai đội thắng đối nghịch vàStart3clock01:53. Vòng1 Dchết thêm do bom lan, giữ đúng nativevector.
69Rusttests/16eightTCP/5guards; hồi quy2/4 ghi riêng. TMPFSevidence/exporthash, Btrfsflushchưa sửa.
M4 còn fixedFree1v7/selector và Mannernegative cùng native9admissionbridge; M5nativebridge riêng, không fullgame/auth.
CP249 merge [PR320](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/320)/`9fdb57b`; AUD071qua248/kế252. Internet thật chưa kiểm.

Python reference cho fixed4v4/1v7 và selector được giao riêng từ WIP CP213, giữ nguyên source đã đo: [provenance/giới hạn](evidence/eight-player/team-code-delivery.json). Không phải nghiệm thu Rust variants.


**CP249: Rust đã đưa ba survivor về phòng sau guest/host EOF khi bom còn chờ.**
[Contract và giới hạn](PROTOCOL.md#rust-four-abort), [raw/hash](evidence/rust-service/README.md#cp249).
Hai run KR thật/Wine/Manner1: guestD mất→A/B/C về phòng; hostA mất→B/C/D về phòng và B lên chủ.
Native180 đủ3byte, actual585, 107 xóa đúng ID và9slot110/run; không175 hoặc158 muộn. 128raw đã seal/cleanup0.
65Rusttests/20guestTCP/20hostTCP/32normalfourTCP/51twoTCP/6prebind đạt. Log/capture native và hồi quy dùng tmpfs
riêng rồi xuất raw/hash vềlocal; hai lỗi flushBtrfs được giữ, **chưa sửa/đạt vận hành bền vững**.
M4 còn tám client; M5backend đã có/nativebridge riêng. CP248 merge [PR319](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/319)/`b58abf8`.
AUD071 qua248/audit kế252. Internet thật chưa kiểm.

**CP248: cùng bốn KR thật đã qua hai kết quả 2v2 đối nghịch, reset và Start3 trên Rust.**
[Contract/giới hạn](PROTOCOL.md#rust-four-results), [raw/hash](evidence/rust-service/README.md#cp248).
Manner1: nativepre175states đảo `[1,0,1,0]`→`[0,1,0,1]`; cả4 đọc đủ562outer/552inner/16rankrows và1803.
ActualReady/Start2–3/cùng16transports/keys12346–48; sauStart3hold6s/freshall4state0.84raw/236,896s sealed/cleanup0,
1.691TCP/280reply/48copiedgetters riêng.62Rusttests/24fourTCP/51twoTCP/4prebind đạt.
Ảnh B/C/D có bảng điểm hai vòng; Aảnh vẫnBattle, nativeAparser/rank/reset có proof riêng, chưa nhận AUIpanelcapture.
M4 còn scopedhost/guestEOF rồi tám, không fullgame/auth. CP247 merge [PR318](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/318)/`d6cbf9e`.
AUD071 qua248/audit kế252; M5backend đã có/nativebridge riêng. Internet thật chưa kiểm.

**CP247: Rust đã đồng bộ trap/death tới ba peer trên bốn KR thật; M4 còn kết quả/reset.**
[Contract/giới hạn](PROTOCOL.md#rust-four-life), [raw/hash](evidence/rust-service/README.md#cp247).
A/C tự đặt bom→17/19→12recipient170; native remote56→1 và cả4view cùng `[1,0,1,0]`.
76raw/134,675s sealed/cleanup0,874TCP/140reply/16transports;7.240copiedgetters riêng.
60Rusttests/17fourTCP/51twoTCP/3prebind đạt; clear21 đủ3recipient mới complete đã kiểm synthetic, chưa nativeclear21.
KernelcleanupR1 giữ lỗi/PIDgone và verifiercleanup-tail correction giữ raw; không nhận đã fix nguyên nhân shutdown.
CP246 merge [PR317](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/317)/`1b3bddf`.
Tiếp rank4/hai outcome2v2/reset/Start3/abortEOF rồi8 trong PLAN; M5bridge riêng. Audit kếCP248. Internet thật chưa kiểm.

**CP246: bốn owner đã đặt/nổ bom qua Rust trên client KR thật; M4 còn life/result/reset.**
[Contract/giới hạn](PROTOCOL.md#rust-four-owner-bombs), [raw/hash](evidence/rust-service/README.md#cp246).
Manner1/cố định2v2: actual118 từng slot→ID1–4→16native159entity/full39 và16native158/full13/lookup-kind0-cleanup.
Stat capacity1/2/1/1/power1 khớp bốn view, fuse6;7.196copiedgetters.84raw/139,844s sealed/cleanup0,902TCP/144reply.
59Rusttests/14fourTCP/51twoTCP đạt. R1fail16722/73raw giữ; four142/167 chỉ boundedobserve/noeffect.
Ảnh own/peer trap-death còn lệch vì127 chưa forward: chưa sharedlife/outcome/reset/EOF hoặc fullM4/tám.
CP245 đã merge [PR316](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/316)/`d0c2ba3`;
M5backend đã có, nativebridge riêng. AUD070 qua244/audit kế248; Internet thật chưa kiểm.

**CP245: bốn client KR thật đã qua phòng/loading và movement trên Rust; M4 còn battle.**
[Contract/giới hạn](PROTOCOL.md#rust-four-startup), [raw/hash](evidence/rust-service/README.md#cp245).
Run Manner1/đội cố định2v2: bốn ID/slot/tên đúng, ba ACK chưa GO, ACK thứ tư mới clock/spawn;
12 hướng remote current progress và7.128 copied getters. 64raw/105,414s sealed/cleanup0,698TCP/112reply.
58Rusttests,7fourTCP/51twoTCP đạt; hai native failure39+43raw giữ nguyên. Chưa bom/life/result/reset/EOF bốn người.
M5 backend account/SQLite/worker/idleTTL/rate đã merge [PR315](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/315)/`e601ca7`;
chưa KR account bridge/ownership. AUD070 qua244, audit kế248; Internet thật chưa kiểm.

**CP244: M5 backend có idle TTL và rate limit; chưa nối account vào client KR.**
[Contract](PROTOCOL.md#rust-account-idle-rate), [raw/hash](evidence/rust-service/README.md#cp244).
Schema2 migration giữ mọi hàng/cột cũ trên5 bản sao DB CP243/10lần mở; session cũ giữ absoluteTTL.
Session mới opt-in idleTTL, chỉ own-player authorize gia hạn; rate process-local dùng chung worker handles.
55Rusttests,7worker cases/7DBs; accountCLI regression33process. M5 vẫn partial: nativeKRbridge/transportownership chưa đủ.
CP243 merge [PR314](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/314)/`dc71e33`; M3 scoped đạt CP242.
AUD070 quaCP244, audit kếCP248. PLAN chuyển M4 bốn rồi tám, bridge M5 riêng. Internet thật chưa kiểm.

**CP243: M5 có worker DB/hash với queue hữu hạn, chưa nối đăng nhập KR.**
[Contract/giới hạn](PROTOCOL.md#rust-account-worker), [raw/hash](evidence/rust-service/README.md#cp243).
Tái dùng backend CP236: SQLite/KDF trên thread riêng, cancel/expiry khi chờ, handoff login và thu hồi đúng token
nếu caller không nhận. Lỗi audit thu hồi đóng worker/giữ rooterror; không giả cleanup đã thành công.
5 ca worker/5 SQLite DB giữ lại, toàn workspace49 tests; chưa idleTTL/rate/native ownership bridge, M5 vẫn partial.
M3 basic hai client đã nghiệm thu [CP242](PROTOCOL.md#rust-m3-acceptance)/[PR313](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/313),
merge `da1b925`. Tiếp idleTTL/rate M5 rồi M4 theo PLAN; audit kếCP244. Internet thật chưa kiểm.

**CP242: M3 đạt phạm vi Battle basic hai client KR trên Rust/Wine/LAN giả lập.**
[Acceptance thủ công](PROTOCOL.md#rust-m3-acceptance), [raw/hash](evidence/rust-service/README.md#cp242).
Hai vòng nhặt lại ba item/reset, exact UDP cũ bị native key-drop rồi current movement apply; Start3/hold6s.
84 raw/306,382s sealed/cleanup0, 1.006 TCP/126 reply, 4.746 copied getters; 44 Rust tests/51 TCP đạt.
Đối nghịch outcome lấy CP240, hostEOF lấy CP241; không fullgame/auth/production. Logger nay giữ lỗi sink gốc,
flush stall còn chưa rõ nguyên nhân. M5 backend đã có CP236; tiếp worker/idleTTL/rate độc lập trước M4.
CP241 merge [PR312](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/312)/`08a9a0d`.
Audit kếCP244. Internet thật chưa kiểm.

**CP241: Rust đã đưa survivor về phòng sau host EOF khi bom đang chờ.**
[Contract](PROTOCOL.md#rust-return-cleanup), [raw/hash](evidence/rust-service/README.md#cp241).
B host đặt bom rồi dừng đúng PID đã kiểm; A nhận forfeit175→180→107 xóa ID2→110/chief và UI phòng,
không158 qua fuse+4s. 48 raw/89.914s sealed/cleanup0; 41Rusttests/50TCP đạt.
M3 còn item consumed-reset/contest, exact UDP cũ + current positive và queue controls; không fullservice.
CP240 merge [PR311](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/311)/`c7dd615`.
M5 SQLite/account backend đã có CP236, native bridge còn riêng. Audit kếCP244. Internet thật chưa kiểm.

**CP240: Rust đã chạy hai kết quả đối nghịch, về phòng và Start3 trên hai client KR thật.**
[Contract/giới hạn](PROTOCOL.md#rust-life-results), [raw/hash](evidence/rust-service/README.md#cp240).
Vòng1 A chết/B thắng; vòng2 B chết/A thắng: hai view cùng state1/0 trước175, đọc đủ302/292 byte và rank2/1 đảo đúng.
Hai180/refresh/Ready rồi clock trận3 giữ6 giây; 66 raw đã seal/cleanup0. M3 **còn mở**: consumed-item reset/contest,
exact UDP cũ + current positive, dead-owner/queue controls và hostEOF đưa peer về phòng. Không full game/accountauth.
CP239 đã merge [PR310](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/310)/`35bb05f`.
AUD069 qua240, audit kế CP244. [PLAN](PLAN.md#current-order) giữ thứ tự; M5 backend/SQLite đã có CP236,
cầu nối KR vẫn riêng. Python và tám WIP giữ nguyên. Internet thật chưa kiểm.

**CP239: Rust đã có terrain/reveal và ba hiệu ứng item basic trên hai KR thật.**
[Contract/giới hạn](PROTOCOL.md#rust-terrain), [raw/hash](evidence/rust-service/README.md#cp239).
Cảhai195flags khớp kernel thamchiếu trên actual118: chỉô2,3 và3,2clear;142đúngô sau158commitmởhidden27/28.
Actual124→mỗi view ba165full12/errors0/founditem: Roller5→6,Fire1→2,Bomb1→2;1314copiedgetters/410TCP/66reply.
54raw frozen/seal/cleanup0;35Rusttests/43TCPcases đạt. M3 **còn life/result-reset/haioutcome**, không fullgame.
CP238 merge [PR309](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/309)/`7ecfba3`;
[PLAN](PLAN.md#current-order) tiếp17/19/21→170→175/180/reset; CP240 phải audit địnhkỳ.
Python/8WIP/accountDB giữ. Internet thật chưa kiểm.

**CP238: hai client KR thật đã đặt và nổ bom qua Rust.**
[Contract/giới hạn](PROTOCOL.md#rust-bombs), [raw/hash](evidence/rust-service/README.md#cp238).
A/B actual118→159full39/entity,158full13/lookupđúngID1-2/kind0/cleanup trêncảhai; fuse6 và6shold sauhai bom.
TCP419/66reply khớp; 48rawfinal frozen/seal/cleanup0.32Rusttests/40TCPcases đạt.
Hai run lỗi44+50raw giữ nguyên; `167/4,32` chỉ boundedopaqueobserve/semanticsunknown,142 chưa mởhiddeneligibility.
M3 vẫn thiếu terrain/reveal/life/result-reset/haioutcome. CP237 merge [PR308](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/308)/`70f00d0`.
[PLAN](PLAN.md#current-order) tiếp boundedterrain/reveal rồi life/results/reset, audit kếCP240.
Python/8WIP/accountDB giữ; Internet thật chưa kiểm.

**CP237: Rust đã có movement hai chiều và nhặt Roller trên client KR thật.**
[Contract/giới hạn](PROTOCOL.md#rust-movement-floor), [raw/hash](evidence/rust-service/README.md#cp237).
Hai cửa sổ ADown/BRight có PCAP217UDP→87ingress native→key/apply→peerposition; TCP368/58reply khớp.
Actual124/index17→một165 mỗi view, native12/12/errors0/itemfound và stat5→6. 58raw run cuối/cleanup0;
51+55raw hai run lỗi giữ nguyên. 30Rusttests/22TCPcases/696copiedgetters/87nativeCRC đạt.
M3 **còn dở**: bom/terrain/life/result-reset/haioutcome chưa port; `123/10` chỉ opaqueobserve policy có budget,
chưa biết semantics. Shutdown timeout cũ tái hiện ở kerneltest, repeatinstrumented đạt nhưng chưa rõ nguyên nhân.
CP236 account/DB đã merge [PR307](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/307)/`9748dcb`;
[PLAN](PLAN.md#current-order) tiếp allocator/bom theo contract đã đo. Audit kếCP240. Python/8WIP giữ.
Internet thật chưa kiểm.

**CP236: M5 đã có lát cắt account/DB độc lập gameplay.**
[Contract backend](PROTOCOL.md#rust-account-backend), [CLI](../rust/README.md#account-admin-cli),
[bằng chứng](evidence/rust-service/README.md#cp236): SQLite migration/checksum/schema thực, Argon2id,
account→player/wireID bền vững, profile tên/version, session TTL/revoke/ban/reset/boot epoch, admin audit cùng transaction.
26 Rust tests (11 account,10 service,5 wire) và33 process CLI/2account/9audit đạt; không lưu plaintext password/token vào DB.
Một ca CLI unknownfield đầu tiên lỗi, đã giữ raw/seal và sửa strict variant trước run mới.
Đây là backend công ty, **chưa nối đăng nhập KR**; M5 chưa đạt đầy đủ, chưa inventory/reward/idleTTL/worker/restore.
CP235 merge [PR306](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/306)/`bb673a4`;
[PLAN](PLAN.md#current-order) tiếp M3 UDP/movement, M5 không cần chờ Battle và ngược lại.
AUD068 qua236, audit kếCP240. Python/WIP giữ nguyên. Internet thật chưa kiểm.

**CP235: M2 đạt trong phạm vi hai identity cục bộ, một phòng và loading trên Wine.**
[Đối chiếu acceptance](PROTOCOL.md#rust-m2-acceptance), [bằng chứng](evidence/rust-service/README.md#cp235),
[cách chạy](../rust/README.md). Sau khi dừng riêng A, B giữ nguyên process/channel, native xóa ID của A,
chín report muộn được drop và UI/clock tiếp tục 8,231 giây. PCAP345/56reply khớp; 44raw frozen/seal/cleanup0.
15Rusttests,6guard trước bind,12shutdown race,8roomTCP,16binding và14M1 đạt. Guard version là policy hỗ trợ
profile KR00967 trước bind; chưa đo UI lỗi version. Sprite A còn trên map sau EOF và timeoutshutdown cũ chưa rõ
nguyên nhân được giữ thành giới hạn M3/M7; không nhận là fullBattle/accountauth/fullservice.
CP234 đã merge [PR305](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/305)/`f15a2ba`.
[PLAN](PLAN.md#current-order) chuyển M3 Battle theo contract đã có; audit kế CP236. Python và WIP giữ nguyên.
Internet thật chưa kiểm.


**CP234: Rust đã đưa hai KR thật từ phòng qua Back/rejoin/Ready tới loading và spawn.**
[Contract/giới hạn](PROTOCOL.md#rust-fixed-room), [evidence](evidence/rust-service/README.md#cp234),
[code/run](../rust/README.md): ownID1/slot0 và ID2/slot1, hostA Back→B nativeclearID1/chief→A rejoin/Ready→B Start;
121129/129/errors0, giữ nativeB99 hơn3s khôngGO, rồi haiACK→197/196/153→clock/spawn/UI cảhai, giữ8.279s.
PCAP334record/56reply khớp; finalbinary554fa5b3… đã chạy. Nămrun152raw frozen/seal/cleanup0 giữ failures/corrections;
14Rusttests/8TCPphòng/16binding/14M1regression. Có một shutdown test timeout chưa tái hiện ởba lần lặp, giữ rõ.
**M2 chưa đóng:** còn chốt negative/version và cleanup sau active-peer EOF; Bfinalreport bị từ chối sau ownedteardownA,
không healthyabortclaim. CP233 merge [PR304](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/304)/`b10952f`;
[PLAN](PLAN.md#current-order) tập trung phần còn thiếu M2 trước BattleM3. Audit kếCP236; Python/WIP giữ. Internet thật chưa kiểm.


**CP233: hai client KR thật đã vào sảnh trên cùng service Rust với ID/tên riêng.**
[Contract/giới hạn](PROTOCOL.md#rust-two-lobby), [code/run](../rust/README.md),
[evidence](evidence/rust-service/README.md#cp233): KRLocal/ID1 và KRGuest/ID2 đều auth137/137,
291128/128/errors0, personalrender/UI đúng tên; PCAP162 record/18reply khớp, giữ8.710s.
Binding dùng IP operator + request đầu kiểmCRC + endpoint/stage/redirectlease; bỏ acceptcounter ở profile hai-client.
Run đầu phân loại WCP chỉ theo port thất bại193raw, sửa theo CP006/103 rồi98raw positive; cả hai frozen/seal/cleanup0.
10Rusttests,16TCP binding cases và14M1 regression đạt. **M2 vẫn dở**: phòng chung/ownslot/host/Ready/loadingbarrier/
Back-rejoin còn thiếu; diagnosticidentity chưa accountauthentication. CP232 merge
[PR303](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/303)/`5b3ef81`; audit kếCP236.
[PLAN](PLAN.md#current-order) tiếp fixedroom bằng contract có sẵn, Python/WIP giữ nguyên. Internet thật chưa kiểm.


**CP232: Rust đã đưa một KR thật tới sảnh với identity cục bộ và profile đúng tên.**
[Code/run](../rust/README.md), [contract/limits](PROTOCOL.md#rust-local-lobby),
[evidence](evidence/rust-service/README.md#cp232): auth0/native137/137, channel17/241/91/14 rồi291
native128/128/errors0/nameKRLocal, personal renderguard1 và UI sảnhKRLocal; PCAP160record/9exactreplies,
giữ9.362s. Root binary đúng hash đã chạy;7Rusttests/fmt/clippy/build và14TCP M1regression đạt.
M2 **chưa hoàn tất**: còn hai-client/phòng/barrier/Back-rejoin; diagnosticidentity chưa accountauthentication.
Ba run startupfailure6raw + staging38 + final47 =91raw frozen/seal/cleanup, giữ correctionopcode15.
CP231 merge [PR302](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/302)/`bc55356`;
AUD067 qua232/kế236. [PLAN](PLAN.md#current-order) tiếp phần còn thiếu M2, RE chỉ khi port bị chặn.
Python serving/RE/reference và támWIP giữ nguyên. Internet thật chưa kiểm.


**CP231: đã viết và chạy lát cắt Rust đầu tiên trên client KR thật.**
[Rust/build/run](../rust/README.md), [contract/limits](PROTOCOL.md#rust-foundation),
[evidence](evidence/rust-service/README.md#cp231): cùng CA PID144 trên Wine nhận result1 hai lần,
ordinary UI retry mở connection mới/resetseq40; native mode2/selector3/43→46, PCAP108record khớp byte,
giữ9.517s sau result lần2. M0 baseline Python/47pins,5wire tests (vector native giữ sẵn) và14kernelTCP cases đạt;
fmt/clippy/locked build pass. Hai run trước giữ failure53raw/capturepartial63raw; final54raw, tổng170raw seal/cleanup.
Đây là **hello/error/retry**, chưa login thành công/sảnh/phòng/fullauth hoặc game trên Rust. Python giữ serving/RE/reference.
CP230 merge [PR301](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/301)/`115c6b8`; audit kếCP232.
[PLAN](PLAN.md#current-order) chuyển M2 local identity→sảnh→fixedroom, RE chỉ khi port bị chặn. Internet thật chưa kiểm.


**CP230: cleanup phòng chung đã có native Back, EOF và chuyển chủ trong prototype.**
[Contract/limits](PROTOCOL.md#room-shared-cleanup), [run/hash](evidence/room-list/README.md#cp230):
guest Back hoặc EOF → A nhận107, native ID2→0/ô B biến mất; host Back → B nhận107/ID1→0,
110 đọc đủ và UI B thành chủ. Ba ca565raw đạt scope hẹp;166raw lỗi controller EOF giữ riêng/correction,
batch731raw frozen/seal/cleanup. Getter sau hostpromotion chưa quan sát vì budget20; không suy đã đo getter1.
CP229 đã merge [PR300](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/300); audit kếCP232.
[Owner đã mở implementation Rust](evidence/owner-directives/rust-implementation.json);
[PLAN](PLAN.md#current-order) ưu tiên M0→M1 ngay, RE chỉ khi chặn lát cắt. CP230 chưa có Rust/gate mới.
Internet thật chưa kiểm.


**CP229: prototype registry đã qua Back/rejoin và chuyển B sang phòng A bằng client thật.**
[Contract/limits](PROTOCOL.md#room-membership-registry), [run/hash](evidence/room-list/README.md#cp229):
B Beta→Back→sảnh→Beta lại đúng ownID/slot0/chief; ca riêng Beta→Alpha đưa B vào slot1/khách,
A giữ slot0/chủ và cả hai nhận đúng peer106. Cùng auth/channel, native87/106/110/291 đọc đủ, PCAP và UI khớp.
Hai run336raw cùng7raw startup-failure đã seal/cleanup; 19 policy scenarios và7 move scenarios tách khỏi native proof.
Registry private có owner/room/visit fences; chưa ghép production, shared-leave/EOF fanout và occupancy card còn thiếu.
CP228/AUD066 đã merge [PR299](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/299); audit kếCP232.
[PLAN](PLAN.md#current-order) giữ bước tiếp; chưa Rust/gate mới. Internet thật chưa kiểm.


**CP228: hai phòng riêng đã hiện đúng một player tại slot0/chief trên mỗi client.**
[Contract/limits](PROTOCOL.md#room-local-roster), [run/hash](evidence/room-list/README.md#cp228):
sau actual585, private adapter trả riêng106/110/291 cho account đã bind; native đọc204/6/128 byte không lỗi,
own-ID khớp1/2, setter slot0 và UI Alpha/KRLocal, Beta/KRGuest. 129raw đã seal/hash/cleanup, giữ8.86s.
Marker cũ `own_slot=1` là predicate, không phải chỉ số slot1; personal-render sau291 chưa đo.
CP227 đã merge [PR298](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/298).
AUD066 qua228/kế232. [PLAN](PLAN.md#current-order) tiếp registry/membership có generation và owner;
chưa coi fixed projection là registry hoặc isolation tổng quát, chưa Rust/gate mới. Internet thật chưa kiểm.


**CP227: chọn phòng đã nối đúng session, ID và tên; lỗi join có đường quay lại chọn tiếp.**
[Contract/limits](PROTOCOL.md#room-join-projection), [năm run/hash](evidence/room-list/README.md#cp227):
hai KR chọn row khác nhau → reply87 → native lookup/context đúng RoomAlpha/RoomBeta, reader7/7 không lỗi.
Control staleID dùng lỗi2 “không thể vào phòng” → đóng thông báo → refresh → chọn ID303 → RoomGamma.
Hai lần click retry không tạo callback và ca dùng lỗi1 “sai mật khẩu” được giữ kèm correction;
recipe click giữa card đã đo lại. 802raw seal/hash/cleanup; chưa trả585/roster/membership/registry/Rust/gate.
CP226 merge [PR297](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/297)/`ccab4cc`;
AUD065 qua224/kế228. [PLAN](PLAN.md#current-order) tiếp roster riêng cho từng phòng.


**CP226: danh sách hai phòng, cập nhật và xóa card đã có native evidence.**
[Contract/limits](PROTOCOL.md#room-list-lifecycle), [bốn run/hash](evidence/room-list/README.md):
hai KR đọc count2 và chọn đúng session/roomID; S→C91 update1/empty0 giữ row khác, không xóa toàn danh sách.
S→C90 tag0 chỉ xóa khi session/roomID khớp; wrongID giữ card. Nativecursor/UI/typed82/PCAP đối chiếu,
545raw seal/hash/cleanup; run đầu gửi counter73 sai giữ nguyên, bản sửa55/58 đã đo. Chưa join87/membership/
registry/Rust/gate mới. CP225 merge [PR296](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/296)/`3bd1cdd`;
AUD065 qua224/kế228. [PLAN](PLAN.md#current-order) tiếp row selection→session lookup/join projection.


**CP225: process B mới đăng nhập lại được; cùng process sau mất kênh vẫn lỗi.**
[Contract/limits](PROTOCOL.md#auth-fresh-process), [ba run/hash](evidence/login-runtime/README.md#cp225):
cleanup B cũ rồi mở KR mới → native0/ID2-name-echo6/67/newlease-session, A cùngPID/owner giữ8s+;
chỉ đổi tokenB mới sang sai → native1/nochannel/noBlease-session/Aowner45s+. Getter trả11 được lưu trước
setter(request1), chưa đồng nghĩa sửa sameprocessresume. 595raw seal/hash/cleanup; không fullauth/Rust/gate mới.
CP224 merge [PR295](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/295)/`928f2fb`;
AUD065 qua224/audit kế228. [PLAN](PLAN.md#current-order) chuyển khoảng trống phòng tổng quát,
không tiếp quét SDK/VM hoặc lặp failedretry.


**CP224: hello và validation thành công; retry sau mất kênh bị fatal error.**
[Contract/correction](PROTOCOL.md#auth-retry-validation), [ba run/hash](evidence/login-runtime/README.md#cp224):
breakpoint đúng return13AD79C/1255C73/1255A69/125BE47 đều trả1, manager phase2;
chưa tokenbuilder/auth6-67/session mới, sau đó SIGABRT/Wine illegalinstruction. CP223 đã ghi retryFAIL
nhưng bỏ sót fatal trong log; record/raw cũ giữ nguyên. Observer unwind đầu được thay bằng exactsite,
462raw seal/hash/cleanup; continuation cho local11→epilogue, không dừng trong handoff. CP223 merge [PR294](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/294)/`51bbf41`.
[Thứ tự hiện hành](PLAN.md#current-order); không fullauth/Rust/gate mới. AUD065 quaCP224/audit kếCP228.


**CP223: identity theo account khi đảo endpoint; retry sau mất kênh chưa đạt.**
[Contract/giới hạn](PROTOCOL.md#auth-session-lifecycle), [bảy run/hash](evidence/login-runtime/README.md#cp223):
control token đảo source vẫn đúng native ID/name/echo/67 và session; đóng B sau17 có native error10/newconnect,
Aowner giữ. CP223 mới khoanh bytes/framer; [CP224](PROTOCOL.md#auth-retry-validation) bổ sung validation thành công và fatal error.
1108raw đã seal/hash/cleanup, corrections ảnh/probe giữ nguồn cũ; không fullauth/sessionexpiry/Rust/gate mới.
CP222 đã merge [PR293](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/293)/`4bcabd8`; audit kếCP224.
[Thứ tự hiện hành](PLAN.md#current-order); không lặp control đã đo.

**CP222: hai KR cùng service đã bind hai identity/session riêng; duplicate account bị native từ chối.**
[Contract/giới hạn](PROTOCOL.md#auth-two-actual), [bốn run/hash](evidence/login-runtime/README.md#cp222):
R3 native0/C2S6 đúng1/2/echo/actual67 cảhai,2CA/2wineserver/2netns/PCAP/8s+hold; R4 tokenB=A cho native1/nochannel,
ownerA giữ nguyên. Hai run denial ban đầu giữ raw; suffix25 có biến môi trường, parser profile mới ghi design rõ.
498raw đã kiểm/seal/cleanup; chưa fullauth/healthyEOFdenial/Rust/gate mới. CP220–221 đã merge
[PR292](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/292)/`f5caefb`; audit kếCP224.
[Thứ tự hiện hành](PLAN.md#current-order); control đảo endpoint đã có ở CP223.

**CP221: A/B đã đi qua native đúng identity; wrong-binding TCP giữ lease/owner.**
[Contract/giới hạn](PROTOCOL.md#auth-two-account), [run/hash](evidence/login-runtime/README.md#cp221):
hai positive Wine riêng ID1/KRLocal và ID2/KRGuest có auth0/C2S6 echo/actual67;119raw đã kiểm.
Bốn TCP driver run/20raw kiểm wrongID/name/sai source không tiêu lease hoặc xóa owner; R1 driver failures giữ nguyên.
Chưa hai KR đồng thời/swap/healthydenial/fullauth/Rust; EOF blocker giữ rõ. CP220–221 giao chung
[PR292](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/292), trạng thái tích hợp theo PR;
AUD064 qua220/audit kế224. [Bước tiếp](PLAN.md#current-order).

**CP220: adapter giữ owner qua TCP duplicate/replay; native EOF lifecycle vẫn chưa đạt.**
[Contract/giới hạn](PROTOCOL.md#auth-tcp-ownership), [run/hash](evidence/login-runtime/README.md#cp220):
TCP driver tổng hợp kiểm owner tiếp tục nhận reply sau duplicate/replay và owner EOF giải phóng session;
không dùng kết quả này thay client KR. Getter/watch receiver chưa có observable đủ để sửa; hai native run vẫn lỗi.
Bốn run/102 raw mới đã seal/hash/cleanup; R1 driver error giữ nguyên, R2 dùng codec sẵn có.
CP219 đã merge [PR291](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/291)/`09435d9`;
AUD064 qua220, audit kếCP224; #14/#285 vẫn mở, chưa Rust/fullauth. [Bước tiếp](PLAN.md#current-order).

**CP219: đã khoanh nguồn dispatch lỗi sau EOF, chưa giải lifecycle âm tính.**
[Contract/giới hạn](PROTOCOL.md#auth-eof-dispatch), [bốn run/hash](evidence/login-runtime/README.md#cp219):
lease hợp lệ hoặc baseline field36=0 vẫn cùng fault; native vtable+50 → target17C7F40 → call12578AC có proof.
188 raw kiểm đủ, cleanup0; native lifecycle vẫn FAIL, receiver selection/rootcause chưa biết.
CP217–218 đã merge [PR290](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/290)/`f9f48d2`;
không đổi gate/Rust/source production, audit kếCP220. [Bước tiếp](PLAN.md#current-order).

**CP218: policy chặn lease expired/revoked, nhưng lifecycle client sau EOF chưa đạt.**
[Blocker/giới hạn](PROTOCOL.md#auth-lease-eof-blocker), [hai run/hash](evidence/login-runtime/README.md#cp218):
validtoken/native0/actual6echo rồi localdeny; cảhai SIGSEGV17C7F4C qua122EA12/12541E0.
Raw/stack seal/cleanup0; harnessreturn0 không làclienthealthy. Rootcause chưa biết, chưa negativeacceptance/fullauth.
CP217valid/badtoken giữ scope riêng; giaoCP217–218 cùngPR, audit kếCP220. Bước tiếp ở [PLAN](PLAN.md#current-order).

**CP217: adapter RE riêng đã phân biệt token đúng/sai bằng client KR thật.**
[Contract/giới hạn](PROTOCOL.md#auth-admission-prototype), [hai run/hash](evidence/login-runtime/README.md#cp217):
input32chars đúng → native0/C2S6echo/actual67; sai → native1/khôngkênh trong45s, cảhai seal/cleanup0.
24expiry/replay/revoke/swap checks mới policyemulator; chưa fullauth/DB/sessionexpiry hoặc production/Rustintegration.
CP216 đã merge [PR289](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/289)/`af16278`;
trạng thái tích hợp CP217 theo PR, AUD063through216/audit kếCP220. Bước tiếp duy nhất ở [PLAN](PLAN.md#current-order).

**CP216: đã xác nhận authfield36 đi qua cache tới C2S6; chưa xác thực tài khoản.**
[Contract/layout/giới hạn](PROTOCOL.md#auth-channel-echo), [hai run/hash](evidence/login-runtime/README.md#cp216):
control một u32+CRC, native reader→setter→getter→serializer→wire khớp; C2S6 chỉ đổi4bodybytes.
Count1/ASCII/currentcopy, không gọi field echo32bit là ticket antoàn hoặc fullauth. Chưa Rust/sourcepolicychange.
CP215 đã merge [PR288](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/288)/`d3b85df`;
CP216 trạng thái tích hợp theo PR, AUD063 qua216/audit kếCP220. Thứ tự duy nhất ở [PLAN](PLAN.md#current-order).

**CP215: token15/16 đã qua native SSO/heap và wire; token2-only cho count0.**
[Contract/giới hạn](PROTOCOL.md#auth-token-bounds), [hai run/hash](evidence/login-runtime/README.md#cp215):
frame100/body62, record17/18byte khớp copied-native,13 helpervectors emulation và CRC/partial controls riêng.
Run token2-only không gọi authbuilder trong cửa sổ đo; cảhai seal/cleanup0. Chưa xác thực tài khoản/Rust.
CP214 đã merge [PR287](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/287)/`83de46c`;
trạng thái tích hợp CP215 theo PR, audit kếCP216. Bước tiếp duy nhất ở [PLAN](PLAN.md#current-order).

**CP214: đã nối token local tới request đăng nhập native KR; chưa xác thực tài khoản.**
[Contract/giới hạn](PROTOCOL.md#auth-token-request), [ba run/hash](evidence/login-runtime/README.md#cp214):
Winecopy4b/count1–2, bốn record token khớp native writer, ba frame actual +CRC/partial controls; cleanup0.
Chưa tokenverify/expiry/revocation/crosssocketownership hoặc Rustcode. PR286/CP213 đã giao kế hoạch Rust;
CP214 đã merge [PR287](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/287); audit tại mốc đó kếCP216. Thứ tự tiếp chỉ ở [PLAN](PLAN.md#current-order).

**Kế hoạch Rust đã lập tại CP213; chưa triển khai Rust.**
[Lát cắt đầu và thứ tự milestone](PLAN.md#rust-service-milestones):
codec/connection → localidentity/sảnh/phòng → Battlehai người/reset → bốn/tám theo contract;
account/DB làm độc lập sớm, sau đó inventory/result-reward và vận hành.
[Thiết kế, ma trận readiness và nguồn tái sử dụng](RUST_SERVICE_ARCHITECTURE.md),
[survey/hash](evidence/rust-service/README.md). Đề xuất cùng repo, rust/ với hai crate kr-wire/kr-service,
một process Linux; Python giữ RE/reference/differential. Không tạo Cargo/repo/worktree Rust trong đợt này.
Authbridge/token/ownership và inventory-reward projection cần đo KR riêng; gate demo không fullservice.

**CP213: tám KR Wine đã hoàn thành hai kết quả đối nghịch/reset/Start3 ở Free 1–7 và Manner 4–4.**
[Contract/giới hạn](PROTOCOL.md#eight-team-variants), [raw/hash](evidence/eight-player/README.md#cp213).
Free1–7:4344gameplay+125modejoins; Manner4–4:4357+124,128copiedgetters mỗi run; selector0/1 giữ qua86/87/180.
Fixedoperatorroster, chưa interactiveC2Steamchange. Các ca Manner1–7 copy4b dừng trước Start do startup; ca támactual trên alternatecopy8d đã trả7/no104,
1543checks/140.441s/cleanup0; guardJSONkhônghit nên khôngstartupfixclaim. CP163 mộtactual/tám tổnghợp giữriêng.
K6c WAN giả lập/K7 giữ đạt scope CP212; **Internet thật chưa kiểm**. CP213 docs/evidence được giao qua PR; code teamvariants vẫn WIP chưa merge, audit kếCP216.
Bước kế duy nhất ở [PLAN](PLAN.md#current-order); không lặp codec/rank/teamguard đã đo.

**CP212: K6c WAN giả lập đạt; K7 hardening/demo bốn client đạt trong scope đã đối chiếu. Internet thật chưa kiểm.**
[Chỉ đạo chủ dự án](evidence/owner-directives/emulated-wan.json) thay duy nhất môi trường K6c;
[acceptance K6c](PLAN.md#k6c-acceptance) và [đánh giá lại từng tiêu chí K7](PLAN.md#k7-evidence-review)
dùng bằng chứng native đã seal: CP193 endpoint negative/positive, CP195 hai trận qua NAT/UDP40±5ms/loss5%,
CP196 host EOF/cleanup/fence và CP203–207 đóng gói/bounds/identity/health/freshserver/bốn client.
[Topology/tham số/giới hạn](PROTOCOL.md#emulated-wan-acceptance), [raw/hash/decision](evidence/peer-endpoints/README.md#cp212).
Không chạy lại phần đã đủ bằng chứng; không nghiệm thu Internet/ISP/CGNAT thực tế, full game/account auth hoặc mọi mode/map/item.
Các giới hạn đó là backlog riêng; tám client không phải điều kiện K6c/K7. Codex tự kiểm, review độc lập waived;
CP212 đã merge [PR284](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/284)/`c0c7b94`, AUD062 qua212/audit kếCP216. Công việc kế tiếp duy nhất ở [PLAN](PLAN.md#current-order).

Các đoạn CP bên dưới ghi **trạng thái tại checkpoint nêu tên**, kể cả K6c/K7 từng còn mở; quyết định hiện hành ở trên.

**CP211: gói server tám người đã chạy sau relocation và hoàn tất hai vòng 4v4/reset/Start3 trên tám KR Wine thật.**
71filepins/47fixturealiases, profile/bom/outcomeflags cốđịnh;
[contract/giới hạn](PROTOCOL.md#eight-portable-battle), [cách xuất/chạy](../tools/README.md#eight-battle-bundle),
[raw/hash](evidence/eight-player/README.md#cp211). Gói không chứaclient/assets; LANgiảlập/Wine, chưaInternet/K7.
CP210 đã merge [PR282](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/282)/`9202d77`; CP211 đã merge [PR283](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/283)/`936693d`.
Tại CP211 audit kếCP212; tiếp actualteamvariants theo PLAN, không lặp codec/rank đãgiải.

**CP210: cùng tám KR Wine đã hoàn thành hai vòng 4v4, hai đội lần lượt thắng, kết quả → phòng → Ready → Start3.**
[Contract/scope](PROTOCOL.md#eight-two-team-cycles), [raw/hash](evidence/eight-player/README.md#cp210).
Native rank8/1751082-inner1072/180/reset vàlife17→19→7peer có nguồn; không item/cứu/kim8 hoặc Internet/K7.
CP209 đã merge [PR281](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/281)/`bebb1d0`; CP210 đã merge PR282.
AUD061 qua208/audit kếCP212; bước kế ở PLAN. Đây là scope8client4v4 tối thiểu, khôngtoànbộgame.

**CP209: tám KR Wine đã có movement đủ56chiều remote và cả tám owner đặt/confirm/nổ bom ở mọi view.**
[Contract/stat/giới hạn](PROTOCOL.md#eight-mesh-owner-bombs), [raw/hash](evidence/eight-player/README.md#cp209).
Ecapacity2,G/Hpower2 đo riêng; TạiCP209 127/rank/reset8 chưa tíchhợp; [CP210](PROTOCOL.md#eight-two-team-cycles) bổ sung hai normal4v4cycles.
CP208 đã merge [PR280](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/280)/`8a99a72`; CP209 đã merge PR281.
AUD061 qua208/audit kếCP212; K6cInternet/K7 còn mở. Bước kế ở PLAN.

**CP208: tám client KR Wine thật đã login → sảnh → phòng → load → Battle/clock.**
Tám prefix/display/netns/IP/CA PID riêng,32TCP/own-slot0–7/8actual99 và full native121285/153205;
[contract/giới hạn](PROTOCOL.md#eight-session-preview), [raw/hash](evidence/eight-player/README.md#cp208).
TạiCP208 chưa movement mesh/bom/kếtquả/reset8; [CP209](PROTOCOL.md#eight-mesh-owner-bombs) bổ sung mesh và bom, kếtquả/reset8 còn thiếu. Bước kế duy nhất ở [PLAN](PLAN.md#current-order).
CP207 đã merge [PR279](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/279)/`7ffed54`; CP208 đã merge PR280;
review waived, AUD061 qua208/audit kếCP212. K6c Internet/K7 vẫn mở.

**CP207: bốn KR Wine đã qua hai vòng 2v2/reset/Start3 dưới UDPdelay/loss, server mới dùng lại samebundle vàidentitystore.**
[Contract/giới hạn](PROTOCOL.md#four-netem-restart), [raw/hash](evidence/peer-endpoints/README.md#cp207),
[K7đốichiếu](PLAN.md#k7-evidence-review). Internet/K6c/K7 vẫn mở; không accountresume/8clientproof.
CP206 đã merge [PR278](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/278)/`4d136ba`; audit kếCP208.

**CP206: có health/run-session-room-round ID và identity pin qua restart process; bốn KR Wine vẫn qua hai vòng 2v2/reset/Start3.**
[Contract/giới hạn](PROTOCOL.md#four-health-identity), [raw/hash](evidence/peer-endpoints/README.md#cp206).
TạiCP206 còn nativefreshrestart/fourUDPimpairment; [CP207](PROTOCOL.md#four-netem-restart) đã bổ sung; bước kế ởPLAN. K7/Internet chưa đóng; auditCP208.
CP205 đã merge [PR277](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/277)/`bf165f9`.

**CP205: guest hoặc host rớt giữa Battle, ba client KR Wine còn lại đã về phòng; host rớt thì B nhận chủ phòng.**
[Contract/giới hạn](PROTOCOL.md#four-member-abort-room), [raw/hash](evidence/peer-endpoints/README.md#cp205).
Không rank giả hoặc nổ bom muộn; phòng vẫn fenced, chưa chơi tiếp ba người/reconnect. Tại CP205 còn health/runidentity/restart; [CP206](PROTOCOL.md#four-health-identity) đã thêm controls, bước hiện hành
trong PLAN; K7/Internet còn mở, audit kếCP208. CP204 đã merge [PR276](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/276)/`d78bc63`.

**CP204: sửa recipient khi gửi lỗi và thêm giới hạn I/O/log/rate; bốn KR Wine vẫn qua hai vòng 2v2/reset/Start3 trên gói mới.**
[Contract/giới hạn](PROTOCOL.md#four-transport-bounds), [raw/hash](evidence/peer-endpoints/README.md#cp204).
Tại CP204 còn nativecleanup ba survivor; [CP205](PROTOCOL.md#four-member-abort-room) đã bổ sung guest/host; tiếp health/restart/identity. K7/Internet vẫn mở; AUD060qua204/kếCP208.

**CP203: gói bốn người đã chạy từ thư mục chuyển chỗ và qua hai vòng 2v2/reset/Start3 bằng KR Wine.**
[Contract/guard/giới hạn](PROTOCOL.md#four-portable-guards), [cách xuất/chạy](../tools/README.md#four-player-bundle).
Tại CP203: guard sau accepteddeath đã có, còn transport/rate; CP204 bổ sung phần đó. Lifecycle/restart/persistence/health/K7/Internet theo PLAN.

**CP202: cùng bốn KR Wine đã hoàn thành hai vòng 2v2, hai đội lần lượt thắng, kết quả/reset và Start3.**
[Contract/giới hạn](PROTOCOL.md#four-two-team-cycles), [raw/hash](evidence/peer-endpoints/README.md#cp202).
Tại CP202 còn guard/đóng gói4; CP203 đã bổ sung. K7/Internet vẫn mở; bước tiếp transport/hardening theo PLAN. Review waived; tại CP202 audit kếCP204, mốc hiện hành theo CLIlatest.

[Coverage và cây chức năng](COVERAGE.md) trả lời: cơ chế nào hiện diện, mức RE/implementation/
runtime nào có nguồn, còn thiếu gì và phép kiểm nào giải quyết. Mỗi ID dẫn thẳng contract/giới hạn,
evidence/hash và câu hỏi; [PLAN](PLAN.md#current-order) giữ duy nhất thứ tự công việc.
Đây là scope đường bootstrap → login → sảnh → phòng → Battle → kết quả/reset, không kiểm kê toàn client.
Tra checkpoint: `python3 tools/checkpoints.py latest`, `show CPxxx`, `evidence CPxxx`
(summary/page hoặc `--verify --require-local`); [kho v2/migration](evidence/checkpoints/README.md#kho-checkpoint-v2-và-cách-đọc-chọn-lọc)
giữ record cũ nguyên object. Không cần đọc manifest lớn.

Hai copy KR Wine đã chạy login/sảnh/phòng/Battle và các chu kỳ kết quả/reset có bằng chứng;
**K5 basic local đạt CP188; K6a local đạt CP189; K6b LAN giả lập đạt CP191; K6c WAN giả lập/K7 đạt CP212. Internet thật chưa kiểm**. [Movement/terrain hai vòng](PROTOCOL.md#current-movement-cycles),
[kim/reset](PROTOCOL.md#needle-round-reset), [cứu hai chiều và hai kết quả](PROTOCOL.md#rescue-two-results),
[deep chain/reset](PROTOCOL.md#recursive-chain-cycles) giữ phạm vi và giới hạn tại nguồn chuẩn.

Các mốc dưới ghi trạng thái **tại checkpoint nêu tên**; next-step hiện hành chỉ ở PLAN.

**K0 và bước tĩnh K2a đạt CP178; K1 đạt riêng recipe Wine R2 offline tại CP179**, đã merge
[PR250](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/250)/`2f2b87f` và
[PR251](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/251)/`2e6dae4`.
[Phạm vi nghiệm thu](PLAN.md#k1-acceptance) không bao gồm bootstrap Windows nguyên bản hoặc ba coldstart e9.

**CP181: K2 đạt cho profile codec/hello offline đã đo**, sau đối chiếu thủ công từng tiêu chí
giữ nguyên. Native retry trong cùng process: result1 hợp lệ → command1002 → login socket mới →
auth137/137/errors0 → channel socket mới; hello/mode/seq reset được đo trên từng connection.
[Nghiệm thu/giới hạn](PLAN.md#k2-acceptance), [contract](PROTOCOL.md#login-retry-codec),
[run/hash](evidence/live-codec-stream/README.md#cp181). CP180 split/coalesced/CRCcontrol đã merge
[PR252](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/252) tại`fe242cc`.

**Tại CP191–211 K6c/K7 còn mở (đã nghiệm thu ở CP212); K6b LANgiảlập đạtCP191 theoowner. K3 local CP186, K4 local CP187, K5 basic CP188 và K6a local CP189 đã đạt riêng scope**,
[nghiệm thu/giới hạn](PLAN.md#k3-acceptance). Fullgrammar/unknownheader/prefix/mode và
accountauthority chưa được khôi phục đầy đủ; K2 không nghiệm thu các phần đó. Thứ tự tiếp ở
[PLAN](PLAN.md#current-order): nhánh mạng/admission đã có CP192–198; bốn session vào Battle tại CP200, CP202đã có2v2haioutcomes/reset/Start3; tiếp hardening theo PLAN, Internet thật giữ backlog riêng theo CP212.
CP181 đã merge [PR253](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/253) tại `5a154e5`.
**CP184: identity291 đã tích hợp opt-in vào service.** Hai run A đã hiện tên ở sảnh rồi tạo phòng,
nhận roster/map; native framer nhận sequence43…73 liên tục, next76. [Contract/giới hạn](PROTOCOL.md#own-profile-integration),
[run/hash](evidence/lobby-identity/README.md#cp184). Đây là một client thật với guest tổng hợp,
không thay bằng chứng hai client hoặc nghiệm thu trận. CP183 đã merge [PR255](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/255)
tại`5c1f3fb`; CP184 đã merge [PR256](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/256) tại`1f8018a`. Tại CP184 K3 còn mở; [CP186 đã đối chiếu nghiệm thu](PLAN.md#k3-acceptance) cho profile hẹp.

**CP185: cache dynamic-map đã có đường request528→reply238 trong service opt-in.**
Đối chứng thiếu file, native ghi lại và warm không yêu cầu528 có [evidence/giới hạn](PROTOCOL.md#lobby-cache-route).
Chưa cold toàn client; CP186 bổ sung cold route kênh và version/identity/negative cho nghiệm thu K3 hẹp. [Wiki KR do chủ dự án tin cậy](REFERENCES.md#owner-trusted-namu-wiki)
được dùng cho luật/stat/char/map/mode trước khi mở RE gameplay; [bảng dùng lại](PROTOCOL.md#trusted-wiki-reuse) giữ phần phải đối chiếu00967.

**CP186: K3 đạt trong scope một A/một kênh normal offline.** Bảy run và tám ca wire kiểm cache route,
version lỗi, native identity và guard request6; [contract/giới hạn](PROTOCOL.md#k3-version-identity-controls).
Không fullaccountauth, không caption kênh có tên hoặc hai client291 đã nghiệm thu.
**CP187: K4 đạt trong scope phòng/load/start hai actualAB.** Đã đo identity291 cả hai, hostBack/
rejoin, một kết quả/reset và Start2; [nghiệm thu/giới hạn](PLAN.md#k4-acceptance).
Copy riêng sửa importABI đã qua lỗi kết quả; [control/recipe](PROTOCOL.md#own-profile-result-compare).
Run CP187 vẫn là lịch sử partial ở input vòng2; [CP188](PROTOCOL.md#current-basic-cycles-291) đã có hai outcome mới, không sửa kết quả run cũ.
**CP188: K5 basic local đạt** theo [tiêu chí giữ nguyên](PLAN.md#k5-acceptance).
CurrentAB291 có hai normal outcome/reset, hai chiều peer và hai owner bom; run riêng đã đo pending
B-host EOF/A cleanup/fence. [Contract/giới hạn](PROTOCOL.md#current-basic-cycles-291),
[recipe/hash và partial giữ nguyên](evidence/peer-endpoints/README.md#cp188).
Tại CP188 còn currentbasiccontest; CP189 bổ sung bên dưới, không thay kết quả lịch sử.
CP187 đã merge [PR259](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/259) tại `e57e6283`.
**CP189: K6a basic local đạt** sau currentcontesteditem/two-normalcycles/reset/nativepeer/life
và đối chiếu CP188Bowner/EOF riêng; [scope/giới hạn](PLAN.md#k6a-acceptance),
[recipe/raw/hash](evidence/peer-endpoints/README.md#cp189). Hai claim thật trên production,
mộtwinner/effect/view; baseline stat riêng A5/B4. Không physicalLAN/Internet/bốn–tám actualclient.
CP188 đã merge [PR260](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/260) tại`2ea548bb`.
**CP190: service/47fixture đã đóng thành gói chuyển thư mục**, kiểm bằng Python-I/realTCP và hai KR
Wine copy trên service đã relocation. [Cách dùng](../tools/README.md#portable-bundle),
[nguồn/giới hạn](PROTOCOL.md#portable-probe-bundle). Tại CP190 còn yêu cầu2máyvậtlý;
CP191owner thay bằng LANgiảlập, startup vẫn tuần tự.
CP189 đã merge [PR261](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/261) tại`1559ad57`.
**CP191: K6b LANgiảlập đạt theo chỉ đạo chủ dự án**, dùng hai actualKR trong netns/IP/NIC riêng;
[decision/giới hạn](PLAN.md#k6b-acceptance). Không còn blocker phải có hai máy vật lý.
BpostwinRollerreset4→5→4 đã xác nhận qua hai normaloutcomes/Start2/3 trênportablebundle;
[nguồn](PROTOCOL.md#emulated-lan-acceptance). K6cInternet/K7 vẫn mở.
CP190 đã merge [PR262](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/262) tại`b7e83f46`.
**CP192: actualKR Wine đã qua hai subnet/router** với hai normalcycles/reset/Start3 và endpoint/nativeUDP
có capture; [contract/giới hạn](PROTOCOL.md#routed-Wine-network). Chưa NAT/relay/Internetgate.
[Owner chấp nhận Wine thay Windows](evidence/owner-directives/wine-accepted.json), không cần máy Windows.
CP191 đã merge [PR263](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/263) tại`56a08dd6`.
AUD057–060 periodic qua192/196/200/204 là lịch sử; AUD062 qua212, audit kế **CP216**. [Review waived](evidence/checkpoints/review-waiver.json).

**CP193: đã đo đối chứng NAT trên hai KR Wine thật.** Endpoint riêng làm consumer trả0; khai báo tuple
đã ánh xạ + DNAT khôi phục handler, key và movement hai chiều. [Contract/giới hạn](PROTOCOL.md#nat-endpoint-control),
[run/hash](evidence/peer-endpoints/README.md#cp193). Tại CP193 chưa hai chu kỳ NAT; [CP194](PROTOCOL.md#nat-two-cycles) bổ sung, K6c Internet vẫn mở.
CP192 đã merge [PR264](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/264) tại`1a6daad0c17d718bc86574162bf2946bca1ed800`.
CP193 đã merge [PR265](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/265) tại`8acf57c81a727892c04e735c28a20a5319d70fd9`; review waived; tại CP193 audit kếCP196, hiện theo counter manifest.

**CP194: hai KR Wine đã hoàn thành hai vòng Battle bình thường/reset/Ready2/Start3 qua mapping NAT tĩnh.**
[Phạm vi và giới hạn](PROTOCOL.md#nat-two-cycles), [raw/hash](evidence/peer-endpoints/README.md#cp194).
CP194 đã merge [PR266](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/266) tại`9f4ebf53d29d22092b0971c7fb34e07dbcb2cce1`; chưa nghiệm thu Internet/K6c.

**CP195: hai vòng KR qua NAT vẫn hoàn tất/reset/Start3 với UDP delay40±5ms/loss5% đã kiểm counter.**
[Phạm vi và giới hạn](PROTOCOL.md#udp-netem-cycles), [raw/hash](evidence/peer-endpoints/README.md#cp195).
CP195 đã merge [PR267](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/267) tại`fe329b0b4da66aa4138c0e1a242da6ab3033d4a2`; TCP baseline, chưa Internet gate.

**CP196: pending B-host EOF qua NAT/UDP impairment đã kiểm A về phòng/ID clear/chief và fence bom chờ.**
[Contract/giới hạn](PROTOCOL.md#nat-netem-host-eof), [run/hash](evidence/peer-endpoints/README.md#cp196).
Run r1 partial do Python probe assertion giữ nguyên; r2 bounded probe pass. Tại CP196: AUD058 qua196/audit kếCP200; mốc hiện hành theo CLIlatest.
CP196 đã merge [PR268](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/268) tại`e842347b7cac073c6822b2251096747710abc170`; K6c Internet/K7 còn mở.

**CP197: incoming admission A/B theo IP nguồn riêng đã có opt-in**, realTCP xen kẽ0wrong/8 và native
startup tuần tự vẫn qua hai trận/reset/Start3 trên bundle mới. [Contract/giới hạn](PROTOCOL.md#operator-incoming-binding),
[recipe/hash](evidence/peer-endpoints/README.md#cp197). CP197 đã merge [PR269](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/269) tại`4c3f6ec246b3e0fcec017592aa6bd34f832ba6b9`. Tại CP197 native startup xen kẽ còn thiếu; [CP198](PROTOCOL.md#native-interleaved-startup) đã bổ sung. Chưa bốn client/K7.

**CP198: hai KR native startup xen kẽ A1,A2,B1,B2,A3,A4,B3,B4**, đúng role/stage/identity, đã hoàn tất
hai normal cycles/reset/Start3 trên gói admission hiện hành. [Scope/giới hạn](PROTOCOL.md#native-interleaved-startup),
[raw/hash](evidence/peer-endpoints/README.md#cp198). Tại CP198: K6c Internet/K7/bốnclient chưa đạt, audit lúcđóCP200; currentfourproof ở CP202–204.

**CP199: copy KR hiện hành đã đọc Start121/peer153 bốn slot và khởi tạo bốn actor** ở cả hai view.
[Phạm vi](PROTOCOL.md#four-slot-native-consumers), [raw/hash](evidence/peer-endpoints/README.md#cp199):
hai actual client + hai actor tổng hợp, A/B peer còn hoạt động. Chưa bốn actual client, kết quả/reset2v2 hoặc K7.
Blocker chuyển sang service bốn session/identity/roster/Ready/barrier/mesh; thứ tự chỉ giữ ở PLAN.

**CP200: bốn KR Wine thật đã login/sảnh/phòng/load vào Battle**, đúng bốn identity/ownslot; run ít probe
quan sát movement từ từng client ở ba client còn lại. [Phạm vi/controls](PROTOCOL.md#four-session-native),
[recipe/hash](evidence/peer-endpoints/README.md#cp200). Service bốn session riêng; chưa bom/kết quả/reset2v2,
Start2 hoặc đóng K7. CP199 đã merge [PR271](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/271)/`ae329187`.
Tại CP200: AUD059 qua200/kếCP204; hiện AUD060qua204/kếCP208. Run lỗi/header/probe/metadata giữ raw riêng, không sửa thành pass.

**CP201: bốn owner A/B/C/D đã đặt bom và nhận confirm/nổ ở cả bốn KR Wine view.**
[Contract/baseline/giới hạn](PROTOCOL.md#four-owner-bombs), [raw/hash](evidence/peer-endpoints/README.md#cp201).
Capacity đã đo1/2/1/1, power1; native15939/39/entity và15813/13/errors0 đủ bốn bom/bốn view. Chưa kết quả/reset2v2.
CP200 đã merge [PR272](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/272)/`cd958c95`; audit kế CP204.

Đích ban đầu hai client/hai trận đã đạt trong scope K5/K6a; bốn client và hardening tiếp theo PLAN.



<a id="plan-order-before-topic-refactor"></a>
## Các đoạn lịch sử chuyển khi tách contract theo chủ đề

Các đoạn dưới giữ nguyên lời tại thời điểm ghi; thứ tự và câu hỏi hiện hành chỉ ở PLAN/RESEARCH.

### Các dòng cũ sau Bảng điều hành


CP263: R1c slot allocation và template106 chung đã kiểm 94 Rust/305 TCP/3189 frame cũ/30 fixture. Audit nhóm AUD080; NEXT R1d header chuẩn và opcode builders. R1b đã merge PR337. N1 uplink pending, R1f shared-NAT và native cuối R1 2/4/8/account giữ dependency/giới hạn.
CP262: R1b typedpolicies đãkiểm90Rust/298TCPmỗibinary/3189exactroomframes, JSON/wirelegacy2/4/8giữ. AuditnhómAUD079; NEXT R1c identity/slot/template106. N1physicaluplinkfactpending/OPEN, khôngẩnR1f/M8dependency; nativefinalR1 2/4/8/account vẫnchưađo. ProjectorWIPgiữnguồnriêng.
CP261: actualInternetpath2KR/account/profile/room/basic2oppositeoutcomes/reset/Start3 đođạt;99postNATmismatch/Bobserved-source reply/nativeUDP2chiều cóproof. SharedNATnegativeBdenied/Aheld8s xácnhậnR1f gap. N1OPEN: mobile/ISP-CGNAT môi trường tốithiểu chưaxácnhận; khôngsuytừIP/agentlabel. NEXT chốtprovenanceuplink vàfield51policyimplementation; R1brefactorWIP cóthểtiếpđộc lập theođiềukiện accountmerge củaowner, chưabỏN1dependencyR1f/M8.
CP260: controlled network/mapping metadata account có90Rust/hai59TCP/9network+5metadata guards; native auth0 trên hai KR qua NAT. Lỗi chờ channel có trước HTTP JSON rỗng; N1OPEN, chưa direct/learning/relay. NEXT đo timer/recv trước modal2 tại1113AA0, đối chiếu packet17 trên PCAP; không dựng HTTPstub như causalfix. R1bWIP/projector giữ nguyên; không đóng nhóm/gate.
CP258: account-owned fixed2/basic end-to-end + revoke/expiry/disconnect + manualfresh rejoin/oldUDPdrop-currentpositive đã đạt scope;5run280raw375exactTCP/cleanup0. M5OPEN/generalprofile/nativeDBqueuefault/recoverylimits; AUD076auditnhóm, không lịch4CP. Internet thật chưa kiểm.



### Thứ tự và diễn biến trước khi rút current-order

<a id="current-order"></a>
### Thứ tự công việc hiện hành

**Thứ tự theo [đích giai đoạn 1](PLAN.md#delivery-target), chốt 11/11:** (1) hoàn tất lát cắt account → phòng cố định → trận
ở mục 1 dưới đây; (2) đo Internet thật [RUST-N1](PLAN.md#rust-n1); (3) tái cấu trúc lõi phòng [RUST-R1](PLAN.md#rust-r1);
(4) RUST-M7; (5) [RUST-G1](PLAN.md#rust-g1) song song RUST-M6 thu hẹp; (6) RUST-M8. Từ nay lát cắt mới **không thêm** cờ
`four_*`/`eight_*`, alias theo identity hay nhánh `capacity() == N` mới vào `room_io.rs`; luật mới đặt trong `RoomRules`.

**Kế hoạch Rust đã giao CP213/PR286 `4f9a963`; owner đã mở implementation theo [chỉ đạo mới09/11](evidence/owner-directives/rust-implementation.json). M0/M1 đã giao, M2 đã nghiệm thu phạm vi hai identity/một phòng/loading; RE chỉ theo blocker của lát cắt.**
[KRE-005b/#14](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/14) giữ RE/reference backlog theo trigger; KR-L001 giữ blocker có nguồn;
<!-- active-issue: 14 --> [KRE-074/#285](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/285) giữ epic chuyển Rust. <!-- active-issue: 285 -->
K0–K7 đạt đúng scope, K6c WAN giả lập; **Internet thật chưa kiểm** là backlog riêng.
Theo [policy mới](evidence/owner-directives/feature-group-audits.json), không deadline audit theo sốCP. Nhóm công việc đang ưu tiên
là các lát cắt M5 dưới đây; **account-owned fixedroom/basic end-to-end** hoàn tất scope CP258/AUD076. Audit khi nhóm kế hoàn tất, hoặc ngay khi có
mâu thuẫn nghiêm trọng. Checkpoint trung gian không tự phát sinh audit. Counter/audit cũ giữ lịch sử.

Team source CP213 đã giao riêng qua PR322/`83abe4b`, giữ nguyên bytes đã đo; [provenance/kiểm](evidence/eight-player/team-code-delivery.json). Baseline Rust cũ không hồi quy theo delivery này.

1. **NEXT-RUST-FIRST-SLICE / M5 worker-to-KR và ownership bridge / M4 Free startup bị chặn / KRE-074/#285**:
   [CP236](PROTOCOL.md#rust-account-backend) đã giao lát cắt backend M5 bằng SQLite thật/CLI: account,
   password/session, identity/profile/version và audited admin. Không cần chờ gameplay để viết backend;
   cũng không lấy KR authbridge chưa giao làm dependency chặn M3. Codex solo luân phiên hai nhánh công việc,
   không có subagent hoặc hai thí nghiệm runtime đồng thời; DBworker là thread trong backend. [CP243](PROTOCOL.md#rust-account-worker) đã có boundedworker/handoff/compensation; [CP244](PROTOCOL.md#rust-account-idle-rate) thêm idleTTL/rate; M5 còn authlease nhiều transport/worker-to-KR và native bridge;
   inventory/progression/resultreward ở M6, restore đầy đủ ở M8. [CLI/giới hạn](../rust/README.md#account-admin-cli).
   [CP235](PROTOCOL.md#rust-m2-acceptance)/[checklist](evidence/rust-service/m2-acceptance.json) chốt M2:
   bootstrap→identity/sảnh→fixedroom/ownslots→hostBack/chief→rejoin/Ready→native2actualloadingbarrier/spawn;
   survivor sau ownedpeerEOF giữ process/channel/clock và late report drop có giới hạn. Không làm lại M2 positive.
   [CP237](PROTOCOL.md#rust-movement-floor) đã đo **UDP/movement hai chiều và floorRoller124→165**:
   PCAP/nativeAPI-key/apply/currentprogress, native165/5→6 cảhai. Không làm lại hai cửa sổ đã đủ proof chỉ để tăngCP.
   [CP238](PROTOCOL.md#rust-bombs) đã có **haiowner118→159→158**/serial-ID1-2/nativeentity-kind0-cleanup,
   fuse6/reservation/ticket-round-recipientfences; không lặp positive chỉđể thêmCP.
   [CP239](PROTOCOL.md#rust-terrain) đã đo **terrain195flags/reveal142/hidden27-28/ba165effects**
   trênhaiKR/Wine: currentclearedcell/committed158fullroster→onewinneritem27/28, Roller5→6/Fire1→2/Bomb1→2.
   Không lặp lại nativepositive chỉđể thêmCP; oneepoch này chưa native reset/chainvariants/generalphysics.
   [CP240](PROTOCOL.md#rust-life-results) đã có report17/19→170, hai outcome đối nghịch/175-180/refresh/Ready2-Start3
   trên hai KR/Wine; 44 kernelTCP gồm cursor qua vòng mới/closed202-585/idle deadline. Native peer21clear/all-dead chưa chạy.
   [CP241](PROTOCOL.md#rust-return-cleanup) đã đo hostB EOF sau actual118/159 trước fuse: A nhận forfeit175/180,
   native107ID2→0/110chief/UI phòng/4s hold và no158; dead-owner118/noID, drain/fresh21/EOF kiểm riêng trong50kernelTCP.
   [CP242](PROTOCOL.md#rust-m3-acceptance)/[acceptance](evidence/rust-service/m3-acceptance.json) **chốt M3 đúng scope**:
   consumed-item reset/ba effects ở hai epoch/OScontest, exact UDP cũ native key-drop/noapply + currentpositive,
   Start3/hold6s và atomic outcome queues. Dùng CP237–241 cho movement hai chiều/oppositeoutcomes/hostEOF đã đủ;
   không lặp lại hay broadRE. Native contest chỉ một actual124/index17/epoch, hai claimant kiểm kernel riêng.
   [CP243](PROTOCOL.md#rust-account-worker) đã giao worker DB/hash/thread riêng/queue1–64+one active,
   cancel/queueexpiry/handoff/compensate exact token/audit và rootfault/join, 5retainedDBs/49tests.
   [CP244](PROTOCOL.md#rust-account-idle-rate) đã giao schema2/idleTTL opt-in và per-principal rate worker
   (process-local),55tests/7workerDBs/5schema1copies-10CLIopens/33CLIregression; 0001 hash và mọi old rows giữ.
   [CP245](PROTOCOL.md#rust-four-startup) đã có bốn native startup/room/loading/currentmovement:
   Manner1/cố định2v2, ownID-slot1–4/0–3, đủ4ACK mớiGO,12remoteprogress;64raw/cleanup0/58tests/7fourTCP/51twoTCP.
   [CP246](PROTOCOL.md#rust-four-owner-bombs) đã có four-owner118→16native159entity/full39→16native158/full13/
   lookup-kind0-cleanup, currentManner1 knownstats/capacity1-2-1-1/power1/fuse6/7.196copiedgetters;84raw/cleanup0.
   Opt-in four_owner_bombs giữ startup mặc định;142aux37/-1 và16722 chỉ boundedobserve/noeffect, không reveal/life.
   [CP247](PROTOCOL.md#rust-four-life) đã có A/C actual17/19→12recipient170/native56→1/all4states[1,0,1,0],
   76raw/cleanup0/7.240copiedgetters; clear21 đủ3currentrecipients mới complete kiểm17fourTCP, nativeclear21 chưa đo.
   Gap own-peertrap/death CP246 được giải đúng path này; không lặp positive. Accepteddeath4 nay chặn118/afterdeathreports,
   kiểm TCP; chưa native dead-ownernegative. [CP248](PROTOCOL.md#rust-four-results) đã có cùng4haiopposite2v2/
   rank4native175562outer-552inner-16rows/1803/reset/actualReady-Start2-3/keys12346–48/all4alive sau6sStart3;
   84raw/cleanup0/1691TCP280reply. Outcome đợi allrecipient17/19/21writes/currentwitness/bombdrain/latestdeath-blast/settle8;
   ranktyped4/hashkhớpCP202không là nativeoracle. Không lặp hai normalcycles/codecRE.
   [CP249](PROTOCOL.md#rust-four-abort) đã port CP205 scoped guestD/hostA EOF khi bom còn chờ:
   ba native180/actual585/107 đúng IDclear/9slot110/Room; hostA mất thì B lên chủ, không175/158 muộn.
   `four_abort_room` opt-in; roomsealed/no3memberStart-replacement-resume, secondlossfailclosed chỉunit.
   [CP250](PROTOCOL.md#rust-eight-baseline) đã có Rust8currentManner1/fixed4v4/cùng32transports:
   actual8ACKbarrier/56remoteprogress/current8capacity-power-speed/8owner118→64native159-158/life7peer/
   native1751082outer1072inner16rows/1803/64refresh/7Ready2–3/Start3/freshall8state0hold6s.
   R1actualdead[1,3,4,5,7] (Dcollateral), R2[2,4,6,8]; đúngteamrankđối nghịch, không tô thànhCP210exactvectors.
   156raw/384,236s/5019TCP912reply/53616copiedgetters riêng;24ảnh tám bảng điểm2vòng/Start3đãxem.
   [CP251](PROTOCOL.md#rust-eight-teams) đã port fixed1v7 cùng106/110/121/rank/outcome và guardManner trướcbegin;
   71tests/Free1v7TCP16/Manner4v4TCP16/Mannerforged104negative3/8prebind/32fourregression đạt.
   Native8Manner1v7/copy8d/count8-cap8-2teams/selector1/balancebranch/return7/no104,90raw/cleanup0 đã có.
   **Free1v7 native haioutcomes/selector0persist vẫn bị chặn ở startup**: copy4bD52,509s/66raw và copy8dG60,841s/67raw
   đều post17 trướcRoom, không phải policyteamfailure đã xác định. Cùng family CP250H/G; không repeat8startup/SDKgraph
   hoặc đổi copy để retry khi chưa có observable mới. Hai lượt sourcefrozen/seal/cleanup giữ nguyên.
   [CP252](PROTOCOL.md#rust-ninth-admission) đã có bridge visitor9: một KR thật/Wine/copy4b, tám member TCP tổng hợp,
   validlobby/C2S6ID9/actual82/full87generic2/native1of1-errors0/ordinaryReturn/hold3s/no585-noRoomgrant.
   19raw/35,408s/cleanup0/1.384TCP249reply,73Rusttests/6TCP/4prebind đạt; không gọi là9native hoặc generalroomadmission.
   NativeR1thiếuReturn nên modalwait→120sidle/runfail giữ17raw; R2chỉ sửa controller. Verifiercleanup-tail correction giữ
   7Brokenpipe trong syntheticteardown; PCAP chứng minh mọi channelFIN/RST sau completion, không xóa lỗi hay rerun native.
   AUD072 sửa claimCP250restore32: actualper-role13 cũ, scalar32khônglaunch. Control actualperrole32 mới/cùngcopy4b/
   currentbinarydefault-offninth vẫn Dpost17 trướcRoom (66raw/48,888s/cleanup0). IndependentCAargv A–G gồmD, Hchưa quan sát;
   không nativeFreegameplay/causalSDKfix. **Dừng Free retry** tới khi có observable call/return/error mới; không SDKgraph/copycycling.
   [CP253](PROTOCOL.md#rust-account-bridge) đã giao **thư viện admission account**, tái dùng Store/worker hiện có:
   session64 → frontend32 riêng → login một lần → actual redirect-delivery seam → channel owner/generation.
   Worker lấy UTC lúc thực thi, check expiry/revoke/profile ở issue/login/channel; không mutex qua await.
   7 ca/6 DB thật/3 body CP217 replay ngoại tuyến/80 workspace tests; cancellation/queue Busy/Closed/stale/drop/fault có kiểm.
   Tại CP253 chưa có listener, writer profile account, launcher API hoặc native M5; không suy parser replay thành authentication.
   **CP254: account listener opt-in và hai account native lobby đã có.** [Contract](PROTOCOL.md#rust-account-serving),
   [metadata](evidence/rust-service/account-serving.json); CP253 merge PR325/f09c59c. SQLite2accounts/private Unix API/session64→
   frontend32/actualC2S1→ownedID-name/field36→C2S6/67→native137/291/render và independentCAargv cảhai đã đối chiếu.
   100raw41,852s/cleanup0/318TCP/18exactreplies; hai ảnh đã xem. DB/log/raw privateTMPFS/exporthash, khôngBtrfsdurabilityfix.
   82workspace tests/9library8DB3retained-nativebody/16syntheticTCP/11startupguard đạt; native negatives khác TCP controls.
   LaunchTTL60/handoff10/periodicinspect1s không renew idle; current67 activity mới renew. Exactopaque87/body2 chỉaccountLobby,
   budget16/noACK-noeffect, semanticsunknown. Knownlogin3848 từCP234, không mở mọi port hoặc unknown opcode.
   Fixed KRLocal/ID1 vàKRGuest/ID2/zero-profile/version1, reject profilekhác/rename/room/battle, không fallback theo IP.
   [CP255](PROTOCOL.md#rust-account-native-lifecycle) đã có **ba native account negative/lifecycle** trên sameCP254Rust/4b:
   controlled AID1/nameA+pendingB echo→no17/no291/nochannelgrant; B6hold≤6s→release→nativeown291/render/8shold.
   Counter chỉ4byte setterargument/index0/callerđãđoCP216, A send1gate trướcauth giữhandoff10s; không forceauthstate/functioncall.
   ActiveA revoke→currentchannelFIN và naturalbackendabsoluteTTL60s→FIN, B giữ8s/DBsessioncurrent; measured0,718s/0,485s,
   chỉ quietTMPFS/LANrun, khôngSLO. 78+79+123raw/49exactTCP/nativeoracles/CAargv/SQLite-audit/6UIảnh kiểm, cleanup0.
   NativeA chỉ noticeerror10/boundednoterminal, **chưa modalclose/retry/recovery**; CP220/224 blockers không bị rút.
   PR326/CP254 đã merge5dcb9af; không Rustsourcechange. Native counter semantics là declared input fault/compoundguard,
   không originalclientnormalbehavior, independentbranchcoverage hoặc alreadyactiveBmatch proof. Backend/C2S6echo32khácstrong64.
   **CP258 đã nghiệm thu nhóm account-owned fixed2/basic xuyên suốt**, theo [chỉ đạo](evidence/owner-directives/account-basic-end-to-end.json)
   và [contract/limits](PROTOCOL.md#rust-account-basic): create→session→private launcher→native đúngprofile/channel/slot→fixedroom→
   basic2oppositeoutcomes→175180/reset/Start3. Normal68raw203,751s/106exactreply; ba hostB revoke/expiry/disconnect
   khiownbombpending→survivorA175180/107ID2clear/110chief/Room/no158pastdue, mỗi48raw/60reply. NativeB revoke/expiry
   hiệngameSessionError2000/Winefatal; chỉserver/survivorfenceđạt, khônggracefulrecovery. Khôngsửa raw hoặcgọi fatal là logoutsuccess.
   Manualfresh B process/backend/frontend mới saurevoke-stop→sameID/newchannel/rejoinslot/Ready/AStart/current12347;
   exactoldBUDP12346/nativekeydrop/noapply, freshBcurrentmovementapply/progress.68raw172,400s/89exactreply/cleanup0.
   Tổng5run280raw375exactTCP;84Rust/59account-basicTCP/16lobbyregression. Oldchannel6roguereplay/newmembership/nextkey
   kiểmTCP riêng, khôngsuy thành nativehostilepeer/strongUDPauth. Group auditAUD076, không lịchcheckpointcốđịnh.
   **Ưu tiên theo [chỉ đạo PR329 của chủ dự án](evidence/owner-directives/pr329-integration-order.json):**
   nhóm account/basic hai người đã merge PR330 tại `1b8d6f2`; nhánh `rust/m5-account-basic` đã đồng bộ main,
   gồm PR329/R1a. WIP projector gồm 5 file, 88 test Rust và 17 ca TCP được giữ trong snapshot và stash riêng;
   chưa chạy native hoặc merge. Không tiếp tục projector trước N1/R1.

   **[RUST-N1](PLAN.md#rust-n1): đã có native quan sát trên đường Internet, còn mở điều kiện môi trường.**
   [CP261](PROTOCOL.md#rust-internet-observation) xác nhận field51/u16/body121 của auth137 đi qua setter/cache194/
   getter13B4030→dialog460. Fixture0 gâyactualtimeout trước17; control30000 và frontendTTL120 (config hiện có)
   cho hai native account/own291/room/basic/haioutcomes/reset/Start3, không patchthreshold/client.
   Native99/153 soactualpostNAT: Aportremap, Btrả lời vềnguồnquan sát; UDP2chiều/nativehelper/apply và3clock cóproof.
   Không cần relay hoặc serverhọcport trong đường đã đo; không suy thành mọiNAT/holepunch/supportInternetphổquát.
   Ca cùngNAT actual2native/2correctfrontend/sharedpeer: Aown291rendergiữ8s, Bauth1/nogrant do lookupmetadata theoIP;
   đưa vàoR1f session/lease transportidentity, không coi đây là sharedNATgameđạt.
   **Còn thiếu trước đóngN1 đúng acceptance:** xácnhận uplinkA là diđộng/CGNAT thật hoặc đonative ởuplinkphùhợp.
   Nguồn đã lưu chỉ xácnhận Internet/publictuple/NATđổicổng; nhãnagent cũ mobile/CGNAT khôngcóprovenance bịloại.
   Ownerquestion đangchờ; không tự đổimôi trường hoặc mở thêmISP/topology. Chưa tuyênbốN1passed.
   Phầnquan sátkiếntrúc đã đủ cho đườnghiệncó; R1b→R1e táicấutrúc khôngđổiwire độc lập cóthểtiếptục theoowner
   choR1bắtđầusauaccountmerge; R1f/M8 vẫn giữN1/mạngchungNAT limits. Không làm lạiJSONguard/SDK/provider rộng.
   Field51control chỉ ởfixturelocal; phép triểnkhai policytườngminh vàoadapter cần giữdefault/nativebinaryprovenance
   và acceptance riêng, không tựđổi toànzeroauthprofile thànhoriginaldefault. Raw/địnhdanh/cấuhìnhVPS chỉlocal.

   R1b đã merge PR337; R1c đã qua source/TCP/differential CP263. Tiếp R1d→R1g theo bảng, giữ byte các profile cũ.
   R1a đã merge PR329/`073700e`; N1 vẫn mở điều kiện uplink, R1f/M8 giữ dependency đó.
   R1f dùng kết quả N1. Cuối R1 mới chạy native hồi quy 2/4/8 người và đường account theo acceptance.
   M5 tổng thể còn projection hồ sơ tổng quát, native DB/queue-fault và giới hạn recovery; giữ WIP để dùng
   ở R1c/R1f hoặc phần M5 sau N1/R1. Tiếp theo là M7 → G1 song song M6 thu hẹp → M8.
   Ba quyết định đã được chủ dự án chốt tại [phạm vi giai đoạn 1](evidence/owner-directives/phase1-scope-decisions.json):
   G1 dùng sáu map LadderMapGroup và item tương ứng; Free 1v7 là giới hạn đã biết, không chặn đích;
   M6 lưu lịch sử trận bền vững, thưởng 0, kim cố định theo policy mỗi vòng. Không hỏi lại các quyết định này.
   M4 vẫn mở do Free1v7haioutcomes/selector0nativepersist; native9 criterion có bằng chứng scope riêng, không tự đóng cả M4.
   Không nâng baseline8 thành fullM4/terrain-items8/auth. Shared8EOF chỉmodel/TCPfence/no175180, chưa7nativeRoomreturn.
   Queue8inbox16 giữlocalpending16/2-4inbox8; unique7life/8bomballwrites. CP209sealedraw142/167 khácfour đã đối chiếu,
   chỉ boundednoeffect/semanticsunknown. R1args540>hardcap480prelaunchrejected;R2wrongnativeports/controller4scope/captureIPs
   controlledstop-seal;R3H/R4Gnativepost17EOF causeunknown. R5exactprior8argvpositive, không SDKcausalfix/authclaim.
   Verifiercollateral/EIGHT_SLOT/meshvariablecorrections giữoldsource/log/inputhash/raw, không rerun native.
   UDPcapturerR5 còn4IPguard/threadstop/3Aoutgoingframes, fullTCP+56nativeprogress độc lập; không nhận14/56UDPwireproof.
   Trong plannedteamvariantnativecontrol mởcaptureIP2..9/supervisethreadfailure và đo14Aedges, không rerun2solvedcycles.
   Ảnh A rank CP248 vẫnBattle/no panel; nativeA175/180/rank/reset đã đủ proof riêng. EOFcontrols không cần normaloutcome
   nên không chạy thêm chỉ lấy ảnh; CP250meaningful8control đã có Aboardcảhai vòng, không backfill ảnhfourCP248 hoặc nhận fullUIclaim.
   R1normalTCP race pendingD158/quiet được sửa bằng explicitdrain, không đổi service. R2/R3 samebinary logflush timeout
   events292/208, raw/PIDgone/sourcefrozen giữ; R2threadreserve_bytes.cold chỉ observable, nguyên nhân chưa kết luận.
   32normalfour+51two và hai native EOF dùng explicittmpfs evidence sinks/export-hashverify saucleanup;
   giữ2sflush/I/Odeadlines/failclosed. Không coi đây là sửa Btrfs, durability/load/SLO hoặc nghiệm thu production.
   Không tiếp tục rerun Btrfs vô hạn; điều tra causalI/O ở M8 theo trigger, không chặn portknown8contract.
   KernelCP247R1 cleanupdeadline/PIDgone giữ, R2instrumentedpass/51twoTCP tuần tự; nguyên nhân shutdown chưa rõ/deadline giữ.
   Rank4typed riêng đã có; legacy `battle` schema hai người vàterraincfg4 vẫn reject. Không dùng chosen-forfeit hai người cho abort bốn.
   M5 native ownership bridge/worker-to-KR độc lập; chỉ RE route/token/multi-socket thật sự chặn binding.
   Không làm lại startup positive, không chờ fullauth/SDK/generalroom.
   Queue TTL/handoff khác idleTTL/sessionexpiry/rate; typed backend đã đạt không chứng minh KR authentication.
   LoadingEOF/EOF khi Ending đã chọn có giới hạn riêng, không suy mọi disconnect. Logger supervision giữ lỗi gốc;
   kernelR2 flush timeout khi inputcopy chạy đồng thời, repeatR3/R5pass, rootcause chưa rõ/deadline giữ nguyên.
   Giữ report-derived/designauthority; M5backend độc lập, không chờ fullauth/SDKRE.
   Opaque167/body4,16hoặc32 giữArmedmember/budget64/noACK-effect-forward/semanticsunknown; không mởmọiopcode.
   Catalog lấy từcùng582 đã pin, hiddenlist2 chỉeligible saucurrentclearance+142đúngô; `123/10` là observedopaque có budget64/Armedmember,
   noACK-effect-forward, semanticsunknown. Không lấy nativepositive để suy originalauthority/fullmovementlaw hoặc chức năng ngoài scope M3 đã nghiệm thu CP242.
   Chọn backend/config trước run, freeze đến seal/cleanup; Python giữ reference/serving và rollback.
   Sprite/scene sau active hostEOF đã có control CP241; không backfill CP235/240107-110 giữa Battle thành fullreturn.
   Native wrong-version error UI chưa kiểm; guard unsupportedoperatorprofile trước bind đã đủ scope M2 được nêu rõ.
   Shutdown timeout CP234 chưa rõ rootcause, tái hiện kernelCP237/SIGKILLwait rồiPIDgone;22caserepeatinstrumented đạt; giữ phép đo PID/thread
   khi tái hiện ở M7, không tăng deadline hoặc gọi đã fix. Không mở fullauth/SDK/generalroom/broadRE làm dependency.
   Pythonrollback giữ; team CP213 delivery riêng; AUD071 quaCP248 audit README→PLAN→PROTOCOL→RESEARCH→evidence/code/raw CP245–248, audit kếCP252.
2. **NEXT-AUTH-SESSION-BINDING — blocked sameprocessrecovery**: [CP224](PROTOCOL.md#auth-retry-validation)
   fullretryfatal sauhello/validation1; [CP225](PROTOCOL.md#auth-fresh-process) getter12562C0 trả11→storeEBP-28
   trước setter12562F0(request1), errorlocal10. FreshprocessBnewlease/sessionpositive/invalidtokennegative đã đo;
   không lặp getter/hello/codec/failedretry hoặc quét SDK/VM. Chỉ mở automaticresume khi có observable/input
   mới có nguồn hoặc thực sự chặn milestone. Sessionexpiry/revoke/restartfence/DB qua CLIbackend đã có CP236; KRbridge/API phục vụ chưa đạt; freshlogin
   không là accountresume/matchrestore. Phần backenddesign/port readiness không cần chờ mọiclientRE.
3. **NEXT-AUTH-CHANNEL-EOF — blocked về observable receiver selection**:
   [CP219](PROTOCOL.md#auth-eof-dispatch) đã đo native bad-dispatch; [CP220](PROTOCOL.md#auth-tcp-ownership)
   getter13AEBE0 trả null ở caller khác, selected-store125783B không hit; hardwarewatch không snapshot,
   không có proof nguồn chọn receiver. Hai followup đã seal; không lặp broad getter/watch/EOF controls.
   Chỉ mở lại khi có cạnh native mới hoặc graceful error recipe có nguồn; không patch vtable/RTTI,
   linear protected decode hoặc mở SDK/VM graph. Chuyển việc độc lập ở mục1 trong lúc blocker này còn mở.
4. **NEXT-ROOM-REGISTRY — backlog theo implementation trigger**: [CP229](PROTOCOL.md#room-membership-registry)/
   [CP230](PROTOCOL.md#room-shared-cleanup) đã đo private sharedjoin/move/guestBack/EOF/hostBack/IDclear/promotionUI.
   Module Python/reference extraction hoặc typed session→membership→recipient seam chỉ làm khi port M2 cần;
   không là dependency M1. Default ProbeService vẫn globalroom/round/jobs/host, chưa multiroom production.
   Otherroom cleanup exclusion mới modelonly/no thirdnative, listoccupancy fixture, hostpostgetter unobservedbudget20;
   known110/UIchief đã có nên không lặp để thêm cùng kết luận. Quota/restartfence/password/full/mode/round theo room
   giữ trigger riêng. Internalqueuedfence không là raw585zero wire-replay protection. Audit kếCP244.


[Owner design](evidence/owner-directives/rust-service.json), [kiến trúc/readiness](RUST_SERVICE_ARCHITECTURE.md),
[unknown/phép đo](RESEARCH.md#rust-service-gaps). Codex solo/tự kiểm, review waived; CP235 M2scopedaccepted/CP242 M3scopedaccepted; M4four/eight tiếp và M5bridge riêng giữ ở mục1, M0/M1 giữ scope, không nâng K0–K7/fullservice; AUD071 qua248/audit kếCP252.


### Diễn biến RE theo checkpoint

## Diễn biến lịch sử theo checkpoint

Các đoạn sau ghi trạng thái **tại checkpoint được nêu**, không phải bảng việc đang hoạt động.
Câu hỏi/kiểm tiếp đã được gom ở bảng phía trên; raw/checkpoint/evidence cũ giữ nguyên.

Cập nhật CP001: contract WCP ở [PROTOCOL](PROTOCOL.md#wcp),raw/review/integration ở
[evidence WCP](evidence/wcp/README.md). Kết luận “socket hai chưa nhận reply” chỉ áp cho
[ba run R2 silent](evidence/wine-r2/README.md),không còn là trạng thái mới nhất.
KRE-004/issue6 đã đóng trong scope Wine r1;KRE-005a/issue9 còn mở cho bước sau.

Q-023, CP006 (review theo [manifest](evidence/checkpoints/manifest.json)): `multiserveridx:0`
trong run copy đã kích hoạt setter `+E78`, tránh nhánh launcher và cho request đầu C→S;
`oneclickuser` liên hệ setter `+E9C` mới chỉ static, chưa đo runtime. Tám fixture S→C mode2/3
đưa guard trả1; hai copy thử NMCO lần lượt tới import init, status và `NMCOGetMyIdCode`.
Nguồn/giới hạn: [PROTOCOL](PROTOCOL.md#wcp), [evidence](evidence/wcp-sequence/README.md).

CP007: copy thay identity EDX:EAX bằng `0:1`, screen name rỗng, sửa heap guard và thử các import
cửa sổ; hai run debug dừng tại cùng thông báo Wine `mmdevapi` assertion. CP008: run plain với ALSA
null chạy audio và sau các import thử dừng tại slot `16EA638`. Khác biệt run chưa chứng minh debugger
gây assertion. Chi tiết/giới hạn tại [evidence](evidence/nmco-identity/README.md); review theo manifest.

CP009: thử tiếp message loop và thread import, client vẽ màn phân loại tuổi; run plain-thread
gặp lỗi ghi tại `45E920` tới địa chỉ `02677C9C`. Pointer/caller và nguyên nhân chưa xác định.
Chi tiết tại [evidence](evidence/nmco-identity/README.md).

CP010: guard của object chứa pointer cũ đã được thử reset; snapshot trực tiếp cho thấy pointer
được khởi tạo lại. Pre-throw snapshot xác định nhánh yêu cầu `CAWH.sys`; thử bỏ hàm cài hook và
các binding API tiếp theo đưa copy tới marker khởi tạo, rồi trap overlay ordinal9. Nguồn chuẩn
cho run/recipe/giới hạn: [evidence object state](evidence/object-state/README.md).

CP011: bỏ call overlay candidate ordinal9 trên copy cho UI login đầy đủ; hai phép click/type
(thường và giữ nút lâu hơn) chưa quan sát ô nhập thay đổi hoặc C→S mới. Callback `CAWH.sys`
trên host chỉ được đọc tĩnh, chưa xác nhận vai trò với hiện tượng input. Nguồn/giới hạn tại
[evidence UI](evidence/login-ui/README.md); cùng CP011 bổ sung provenance theo review CP010.

Áp dụng kiến thức VN trong CP011: GDB proxy bắt guard/IAT; run tắt audio trong prefix riêng
đo192 message chuột/phím tại DispatchMessage, nhưng chưa thấy nội dung ô nhập đổi hoặc C→S mới.
Nguồn và giới hạn tại [VN→KR](evidence/vn-reuse/README.md). Chưa chứng minh custom handler nhận
đúng control; GetAsyncKeyState vẫn là candidate KR, so VN chưa đủ để đổi thành GetKeyState.

CP012: đã nối mouse/key qua handler và click tới command1002. Hai tọa độ ID/password chưa
cho edit control; phím được chuyển tới button, không suy toàn bộ input hỏng. Đã tìm hostname login
riêng; run có ánh xạ local mở kết nối3 trước click, WCP poll/consumer trả1. Sau đó vào attempt
`11049C0` từ return`11013FE`, modal `CWaitForLoginDlg` trả1, chưa vào nhánh result3; attempt EAX0.
`modal-result` nối setter1 tới nhánh kiểm global1A966FC; bản dump có giá trị1. Reset riêng global
trong bộ nhớ run mới giữ được dialog đếm chờ, không thấy return trong cửa sổ đo.
Hai run reset thu276 byte mới=4 frame C→S15, nối guard→virtual→WinSock/capture; native parser
replay với seq40+3n khớp CRC cho bốn mẫu, negatives từ chối. Chưa xác nhận semantics auth.
Raw/recipe/timeline/giới hạn tại [evidence handler](evidence/ui-handler/README.md).

CP013 đối chiếu VN: static dispatcher KR đã nối S→C4/result0 và S→C15 tới các routine wait0→2→1,
rồi tick1→3; offset state KR+F4 khác VN+D8. Nguồn/call và giới hạn tại
[evidence VN→KR](evidence/vn-reuse/README.md). Chưa quan sát runtime hai handler hoặc đủ payload;
không dùng code VN `login_ok()` nguyên xi để đóng contract KR.

CP014 đã tìm trigger auth từ token local, capture request và thực thi handler phản hồi lỗi;
contract/chiều seq/prefix result0 tại [PROTOCOL](PROTOCOL.md#login-auth), control và các phép đo
bị loại ở [evidence login](evidence/login-runtime/README.md). CP013 đã merge PR21.
Nguồn count0 của auto-attempt đã giải; lifecycle kích hoạt stage tự động và global vẫn chưa đủ.

CP015 đã qua helper và nối flags62 → request5 → redirect/request6 → stage11/CLobbyStage;
S→C241 đóng notice, UI sảnh đã hiển thị trên copy. Contract/giới hạn ở
[PROTOCOL](PROTOCOL.md#channel-lobby), raw/control ở [evidence](evidence/postwait/README.md).
CP014 đã merge PR22. Không suy auth/profile đủ hoặc K3 từ UI này.

CP016 đã nối click → command2011 → guard91/bảng kênh14 → CCreateDlgRenew → capture81/107byte.
Contract và giới hạn ở [PROTOCOL](PROTOCOL.md#room-create), raw/control/correction tại
[evidence](evidence/room-input/README.md). CP015 đã merge PR23. Profile còn thiếu và chưa có phòng.

CP017 đã đo S→C86/result0 → strategy thật → CPrepareStage/stage12 và UI phòng chờ tám ô trống.
S→C92 bị region guard trong run, không phải đường success dùng cho cấu hình này. Contract duy nhất tại
[PROTOCOL](PROTOCOL.md#room-create), phép đo/correction tại [evidence](evidence/room-reply/README.md).
CP016 đã merge PR24; CP017 là thí nghiệm trên copy, tự kiểm/miễn review, tích hợp theo PR.

CP018 áp dụng VN đúng phạm vi: player/profile, own-ID match và unlock controls giúp chọn probe;
KR106/110/schema được phục hồi riêng. [Contract](PROTOCOL.md#room-player),
[raw/negative/correction](evidence/room-player/README.md). CP017 đã merge PR25.
Player `KRLocal` đã hiện; ID1 không match local list0, ID0 match own_slot1;110/pending0 cho nút Bắt đầu.
Hai import API candidate được bind thật trên copy mới. Full auth/identity vẫn thiếu, không coi ID0 là contract server.

CP019 đã nối writer96, config241/checksum, table238 và ack117; map capacity4 và filter mode0
đã qua bằng field+5D có control. Click thật tự gửi104/109byte, send1/pending3. Contract duy nhất ở
[PROTOCOL](PROTOCOL.md#room-map), [raw/correction](evidence/room-map/README.md).
CP018 đã merge PR26. Đây là mốc request start, chưa có loading/Battle hoặc gate mới.

CP020 nối start104 tới receiver KR121, slot validation và init CGameStage/stage13.
Auth prefix mở rộng có trường ID1, consumer tự cập nhật local-user list; player1 match own-slot.
ID0 workaround cũ không đủ để tham gia start. Contract/layout/correction tại
[PROTOCOL](PROTOCOL.md#start-reply), raw/native/control tại [evidence](evidence/start-reply/README.md).
CP019 đã merge PR27; CP020 có scene map/HUD/hai nhân vật trên diagnostic copy, chưa gameplay hoặc gate mới.

CP021 đã nối actual99 tới writer/init, đo197/196/mode0/198, clock/input và bom hiện sau118/159.
Nguồn chuẩn [PROTOCOL](PROTOCOL.md#round-go), raw/control tại [evidence](evidence/round-go/README.md).
CP020/PR28 đã merge. Đây là bước clock/di chuyển trên một client có guest tổng hợp, chưa vòng trận.

CP022 đã phân biệt calcE9C520 với nhánh158/kind0→E9E660; burst có nước/HELP và local actor
biến mất sau đó.180/full3 được đọc3/3/errors0 rồi stage12/UI phòng.
[Contract](PROTOCOL.md#bomb-cycle), [raw/correction](evidence/bomb-calc/README.md).
CP023 đã nối writer chungB53010 cho127/sub17/19 và giải full prefix14+blob8/5 bằng native;
175/292rawbyte đọc đủ và UI có LOSE/PERFECT. Stub chuyển từ fixed15s sang checksum-checked
reported19→rank chọn→180. Refresh110 host rồi guest-ready cho click thật phát104 lần hai;
run cuối có hai vòng scene/input/bom/HELP/kết quả/phòng trên cùng channel.
[Contract](PROTOCOL.md#round-result), [raw/control/correction](evidence/round-repeat/README.md).
CP023/PR31 đến CP026/PR34 đã merge. CP027 bổ sung hai client movement/bom runtime dưới đây; review độc lập waived, gates giữ mở.

CP024 đã echo owner/ba field từ actual118 sang159 cho bom đầu mỗi vòng; serial0 chưa phân biệt hai field.
CP025 có control serial28 bác mapping aux16, sửa serial u32 và ba bom liên tiếp0/28/56;
ID1..3, lặp/cũ hẹp và native getter được kiểm. [Contract](PROTOCOL.md#bomb-serial),
[evidence](evidence/bomb-serial/README.md).
Đã nối index spawn0/1→point từ map→actual actor.100/4039 có nguồn bảng Antarctic01;
Village00 và ExVillage01 có profile/map/spawn/geometry riêng và hai vòng thử với guest tổng hợp.
[Contract](PROTOCOL.md#bomb-position), [raw/control/correction](evidence/bomb-position/README.md).
Không dùng điều này thay nghiệm thu collision/reset/authority hoặc hai client.

CP030 đã có layout582/request124/confirm165, nhặt từ hai origin và một ca tranh cùng index;
nguồn chuẩn ở [PROTOCOL](PROTOCOL.md#item-pickup), [raw/control/hash](evidence/item-pickup/README.md).
Trap/death CP029 giữ [contract riêng](PROTOCOL.md#peer-events).
CP030/PR38 đã merge `1f70d4f`. CP031 đo hiệu ứng Bomb/Fire, duplicate sau xóa, native clamp và
sửa mapping power; [nguồn chuẩn/giới hạn](PROTOCOL.md#item-effects),
[raw/control/hash](evidence/item-effects/README.md). CP031/PR39 đã merge `4de4e2e`.
CP032 đo block/list2→reveal/124/165/stat và khởi tạo lại vòng; contract ở
[PROTOCOL](PROTOCOL.md#hidden-items), raw/control ở [evidence](evidence/hidden-items/README.md).
Phân biệt run chờ sai origin với run sửa policy; review độc lập waived.
CP032/PR40 đã merge `fe46beb`. CP033 đo chain hai bom liền kề/cách hai ô và replay sau chain;
[contract](PROTOCOL.md#bomb-chain), [raw/control](evidence/bomb-chain/README.md). Không nâng toàn gate.

CP033/PR41 đã merge `fbf4dc5`. CP034 đo leave/107/host110/guestEOF và giải
return loading bằng trường loại sảnh trong channel14; [contract duy nhất](PROTOCOL.md#room-leave),
[control/recipe/hash](evidence/room-leave/README.md). Không ghép các ca này thành full match.

CP034/PR42 đã merge `0779bc7`. CP035 đo B rejoin và A rejoin sau B promotion,
Ready A→Start B→121/99/clock hai phía; [contract](PROTOCOL.md#room-rejoin),
[evidence](evidence/room-rejoin/README.md). Chỉ mốc rejoin/start, chưa full match.

CP035/PR43 đã merge `1e78035`. CP036 đo enemy touch/no-touch, rescue18→170/18/state56→26
hai phía và movement tiếp; [contract](PROTOCOL.md#collision-rescue), [evidence](evidence/collision-rescue/README.md).
Ca rescue có hai client thật + C tổng hợp; control một đội/report21 không dùng làm full match.

CP036/[PR44](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/44) đã merge `7ab702f`.
CP037 thêm [kim tự thoát/peer](PROTOCOL.md#needle-selfescape) và
[cứu/diệt ba client thật + nhánh native](PROTOCOL.md#contact-precedence);
[recipe/control/hash](evidence/needle-contact/README.md). Không ghép các ca thành full match.

CP037/[PR45](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/45) đã merge `1e2ac9a`.
CP038 trả lời hẹp [trap lần hai khi hết kim](PROTOCOL.md#needle-repeat), có đối chứng hoán đổi team;
[raw/control/hash](evidence/needle-repeat/README.md). Kiểm exact-simultaneity runtime là backlog bổ sung,
không điều kiện chặn mốc trận đầy đủ; đây là quyết định ưu tiên của Codex, không tự xác nhận unknown.

CP038/[PR46](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/46) đã merge `1fb2c66`.
CP039 có [hai vòng statepeer/result/return, report21 suppression và hostEOFcontrol/policy](PROTOCOL.md#round-finish),
[recipe/hash](evidence/round-finish/README.md). Không còn coi các mốc hẹp này unknown;
semantics21/sub34, toàn reset/authority và disconnectvariants vẫn mở.

CP039/[PR47](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/47) đã merge `e19bc9e`.
CP040 đo [Start với client mới sau host EOF](PROTOCOL.md#abort-rejoin),
[recipe/control/hash](evidence/abort-rejoin/README.md). Đây là C identity mới3 vào slot0;
không coi là A reconnect/session resume. Giữ lượt collector lỗi và lượt bấm Start chưa tạo104 như lịch sử.

CP040/[PR48](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/48) đã merge `99d1835`.
CP041 có [stale158/ID reuse qua vòng](PROTOCOL.md#round-generation),
[controls/recipe/hash](evidence/round-generation/README.md). Không coi CRC/seq hợp lệ hoặc callback đầy đủ
là chống stale semantic. Fence cho job local là design server, không field generation phục hồi từ wire.

CP041/[PR49](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/49) đã merge `97320ca`.
CP042 có [old165/index reuse và stat trước124 mới](PROTOCOL.md#item-generation),
[control/hash](evidence/item-generation/README.md). Claim/ticket local là design; chưa giải được124 cũ
có toàn field khớp catalog mới. Không tìm lại receiver165 hoặc coi duplicate cùng vòng là proof qua vòng.

CP042/[PR50](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/50) đã merge `ed47c97`.
CP043 đo [159 cũ khi serial local trùng](PROTOCOL.md#late-bomb-confirm),
[recipe/control/hash](evidence/late-bomb-confirm/README.md). ID cấp tăng không đủ khi old159 khớp pending serial;
ca chèn gói cũ mất đồng bộ, không có result vòng2, phải giữ như outcome riêng.

CP043/[PR51](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/51) đã merge `f8569d8`.
CP044 trả lời hẹp [timer còn chờ khi host EOF](PROTOCOL.md#abort-timer),
[control/fence/vòng B/C mới](evidence/abort-timer/README.md). Job cũ bị loại ở deadline sau finish;
vòng mới có actual Bbom/trap/death/statepeer/result/return. Không phải A reconnect hoặc toàn reset.

CP044/[PR52](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/52) đã merge `b947a9d`.
CP045 bổ sung [nhóm item/nguồn cũ/Roller và MaxPower KR](PROTOCOL.md#item-families),
[recipe/hash](evidence/item-families/README.md). VN R169–185 đã có Rollerclamp/reset/movement;
không còn gọi nó unknown vì chỉ đọc oc_e-game cũ. [UDP cũ](PROTOCOL.md#udp-generation) đã tới
queue/drain và consumer game ở vòng2, slot1/gate0/return13; chưa chứng minh actorapply.
CP045 hoàn tất record/tự kiểm, review độc lập waived; [recipe/các control chưa đạt](evidence/udp-generation/README.md).
CP045/PR53 đã merge `6fe17b0`. CP046 đo tiếp [oldUDP tới actor](PROTOCOL.md#udp-generation):
native getter xác nhận x120→200 qua exactold callback, gói hiện tại sửa về120; no-replaycontrol
giữ x120/y159 ở8callback trước input, có hai result/return. Oldrun vòng2 có17 nhưng chưa19/result;
không quy toàn outcome chỉ do replay. CP046 hoàn tất record/tự kiểm, review độc lập waived.
E8E960 là false stub trong build này; không tiếp tìm input bật gate đã đo0.
CP046/PR54 đã merge `f85cda4`. CP047 theo target thật E66680, tái dùng cache CP025:
[byte khóa/tailfilter](PROTOCOL.md#udp-generation), [runtime key/endpoint](evidence/udp-freshness/README.md).
Đổi153key chặn oldmovement đã đo; endpointrotation riêng lẻ không chặn cả oldsource/newsource.
CP047 hoàn tất record/docs/tự kiểm, allocator unit-tested/pending runtime integration; review độc lập waived.


### Mốc README trước khi tách contract theo chủ đề

**CP271: sửa kỳ vọng test tranh chấp SQLite khi bàn giao R1g.**
CI của PR348 đã phát hiện test chỉ chấp nhận `ActiveSession`, trong khi backend có đường trả `Busy` hữu hạn.
Test giữ yêu cầu chỉ một grant, chấp nhận đúng hai lỗi đó, kiểm lại `ActiveSession` sau tranh chấp và ép khóa SQLite
để kiểm `Busy`/phục hồi. Không đổi runtime; [correction và bằng chứng](evidence/rust-service/r1g-ci-correction.json).

R1g CP270 đã kiểm trần admission và allowlist độc lập số profile.
[Contract/giới hạn](PROTOCOL.md#rust-r1g-admission), [evidence](evidence/rust-service/r1g-admission.json).
Cấu hình cho phép1–1024transport và1–256entry allowlist; vẫn kiểm source/lease/profile riêng. Giá trị cấu hình cũ giữ nguyên.
105Rust/fmt/clippy/lockedbuild đạt;306TCPlegacy/3169+24frame khớp CP269;10ca admission đạt, gồm65socket giữ đồng thời,
kết nối66 bị chặn và phục hồi permit. Đây chưa phải kiểm50người chơi KR hoặc native candidate.
R1f đã merge PR347; R1g đang bàn giao PR. Tiếp relay opt-in và native cuối R1 theo [dashboard](PLAN.md#dashboard).
Full M5/M7/M8 còn mở; hạ tầng chỉ local. Mốc cũ ở [HISTORY](HISTORY.md).



### Mô tả R1 trước khi cập nhật trạng thái sau PR348

<a id="rust-r1"></a>
**RUST-R1 — tái cấu trúc lõi phòng trước khi xây tiếp.**
Lý do, theo code tại `78daa71`: cả service chỉ có **một phòng toàn cục** (`BoundProfiles.room: Option<Shared>`,
`lib.rs`); identity chính là slot (`Room::enter` yêu cầu identity 1..=capacity, slot = identity − 1, chỉ identity 1 được
tạo phòng, `room.rs`); mỗi identity cần một fixture 106 riêng theo alias `player_A..H` (`room_io.rs`); mỗi sức chứa bật
bằng cờ riêng (`four_owner_bombs`, `four_life_reports`, `four_results`, `four_abort_room`, `eight_battle`,
`ninth_admission_probe`) và có ba đường rời phòng, hai đường tính kết quả khác nhau; một số nhánh chọn theo việc
"có fixture `rank` hay không"; header khung được mượn từ fixture bất kỳ (ví dụ header của `start` cho packet 180);
client được nhận diện theo IP nguồn. M7, G1 và M6 đều phải sửa đúng các chỗ này.

Nguyên tắc: **không thêm tính năng, không đổi byte trên wire** của ba profile đã nghiệm thu (2, 4, 8 người). Mỗi bước
một PR nhỏ. Không gộp R1 với RE mới.

| Bước | Nội dung | Kiểm |
|---|---|---|
| R1a | Gom luật phòng vào một kiểu `RoomRules` (`rules.rs`): sức chứa, sáu cờ, bảng bom mặc định, cỡ hàng đợi, kiểm chéo cấu hình. Chỉ chuyển chỗ, không đổi logic | fmt, clippy, cargo test |
| R1b | Đổi cờ thành chính sách có tên (`ResultPolicy`, `DeparturePolicy`, `BombPolicy`, `LifePolicy`); bỏ công tắc "có fixture `rank`". Ba profile chọn đúng tổ hợp chính sách cũ | Như R1a, thêm các bộ TCP `tools/test_rust_{room,four_room,eight_room,life}.py` khớp byte |
| R1c | Tách identity khỏi slot: `PlayerId` lấy từ session, slot theo luật cấp phát, host là người tạo. Một template 106 chung, vá identity/tên/slot/đội. Profile cũ giữ luật "slot = identity − 1" | Như R1b; đối chiếu byte với tám fixture 106 hiện có, field lệch ngoài phần đã hiểu ghi unknown trong RESEARCH |
| R1d | Một header chuẩn đã pin và builder body theo opcode trong `kr-wire` (module `kr00967`); packet không còn phụ thuộc alias fixture | Như R1b |
| R1e | Registry phòng `RoomId → phòng`; listing 91 dựng từ registry; mỗi phòng một `RoomRules`. Giữ mutex đồng bộ, không giữ lock qua await | Như R1b, thêm unit test hai phòng không lẫn recipient |
| R1f | Transport lấy identity từ session/lease thay cho IP nguồn; endpoint 153 theo kết quả N1 | Như R1b, thêm ca hai session cùng IP |
| R1g | Trần kết nối và `allowed_peers` thành cấu hình theo tải đích (50 người, khoảng 200 transport) | Unit test giới hạn; kiểm tải thật thuộc M8 |

**R1b — CP262:** các cờ nội bộ được chuyển thành Result/Departure/Bomb/Life/TeamPolicy trong `RoomRules`;
JSON compatibility được normalize tại admission. Không chọn authority từ sự tồn tại của fixture `rank`.
90Rust/fmt/clippy/lockedbuild; baseline vàcandidate đều239caTCP2/4/8 +59account;3189roomframes/13suite
khớp identity/visit/alias/opcode/sequence/size/fullSHA. Các ca8team/ninth cóunreadfinallytail được đoriêng
bằngreceiverboundary; rawkhácbanđầugiữ. N1nativeR10vẫngắnARMCP260/copy4b, khôngtựcertifyR1bbinary.
AuditnhómAUD079; [contract/evidence](PROTOCOL.md#rust-r1b-policies). NEXT R1c, nativecuốiR1vẫntheoacceptancedưới.

**R1c — CP263:** `SlotAllocation::LegacyIdentity` giữ mặc định; `FirstVacant` cấp slot trống độc lập với
PlayerId, người tạo là host. Một template106 tái tạo đúng cả 30 fixture đã pin; runtime cache theo PlayerId,
không lookup alias identity. 94 Rust/305 TCP đạt; 298 ca cũ và 3.189 frame phòng khớp baseline CP262.
Bộ mới có B tạo phòng/A vào slot1, chuyển host/rejoin và năm guard hợp lệ pin nhưng sai body/header/identity/tên/policy.
Chỉ kiểm source/TCP, chưa native candidate; profile account tổng quát và session cùng NAT không được suy là đã sửa.
Audit nhóm AUD080; [contract/giới hạn](PROTOCOL.md#rust-r1c-player-slots). **Tiếp R1d**, cuối R1 vẫn giữ native bên dưới.

**R1f — CP269:** account profiles keyed PlayerId, legacy IP index riêng; frontend token chọn expectedmetadata sau
lookup issuedplayer, transport/sourceIP chỉ restriction. Channel lease chọn owner/profile, không có fixedroom IP→identity
fence cũ. Shared-IPconfig chỉ accountmode; endpoint theo currentchannelconnection và LegacyDirectPlan(round/members/slot).
104Rust/fmt/clippy/lockedbuild,306TCPlegacy và3169+24frameCP265 khớp;59sharedIPfullaccountlife/16accountserving đạt.
Chỉ source/TCP; native cuối R1 vẫn bắt buộc. Relay/key256/portreuse/M5generalprofile giữ riêng; [scope/evidence](PROTOCOL.md#rust-r1f-session-binding).

**R1g — CP270:** `max_connections` được validate1..1024 và allowlist1..256entry, độc lập số actor/profile.
Mỗi actor cấu hình vẫn phải nằm trong allowlist; diagnostic không map được IP thì không có identity, account giữ lease.
105Rust/fmt/clippy/lockedbuild,306TCPlegacy/3169+24frameCP269 khớp;10ca admission gồm65held/66thdenied/release recovery,
invalidbounds prebind và extraallowedIP không cấp actor. Không nhận65socket là65KR hoặc tải50người.
R2 giữ đúng actualcheckout/producer/sourcefreeze, R1wrongcwd giữ raw/excluded. [Scope/evidence](PROTOCOL.md#rust-r1g-admission).
Native cuối R1 và relay opt-in/50playerM8 vẫn chưa đạt; không đóng fullR1 từ refactor.

R1a–R1b và R1d không đổi hành vi. R1c, R1e, R1f và R1g chỉ thêm hành vi ở chế độ mới; profile cũ phải giữ nguyên byte.
Acceptance cuối R1: toàn bộ bộ TCP hiện có đạt, cộng **một lượt native hồi quy cho mỗi profile 2, 4, 8 người và đường
account**, không thêm claim mới. R1b trở đi bắt đầu sau khi lát cắt account → phòng → trận merge, vì cùng sửa `room_io.rs`
và `lib.rs`.

**R1a đã làm 11/11**, đã merge [PR329](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/329)/`073700e`; lần PR329 không tạo checkpoint. Bản đồng bộ thật sau accountPR330 được ghi ở CP259, không sửa evidence cũ: `rules.rs` giữ
sáu cờ, ba luật dẫn xuất, bảng bom mặc định, cỡ hàng đợi và kiểm chéo cấu hình; `room.rs`/`room_io.rs` chỉ đổi chỗ gọi.
Kiểm: fmt và clippy đạt; 86 test Rust đạt (84 cũ, 2 mới). Các bộ TCP tổng hợp đạt trên binary sau refactor: hai người
room 43 ca và life 51 ca; bốn người normal 32 ca, abort A và D mỗi bộ 20 ca; tám người Manner và đội cố định mỗi bộ
16 ca; visitor thứ chín 6 ca; guard cấu hình trước bind 6 ca. Đối chứng: binary dựng từ `78daa71` cho cùng kết quả ở
bộ room và guard. Raw và hash ở `local/research/claude/r1a-tcp-regression-r1/`. Đây là TCP tổng hợp trên loopback,
**không phải bằng chứng native**; lượt native hồi quy nằm ở acceptance cuối R1.
Rollback: từng bước revert được độc lập; Python reference không đổi.



### Ghi nhận N1 trước kết luận cuối

**Lịch sử CP261 — lúc ghi chưa đóng gate (CP264 đã chốt theo phạm vi bên trên):** recipe hai account/basic đạt haioutcomes/reset/Start3 vàactualUDP2chiều;
99 tựbáo≠postNAT ởA, Bnative trảvềnguồnquan sát. Ca chungNAT đãghi negative; session/lease binding R1f cònthiếu.
Nhánh(a) **chỉ trên đường đã đo**, không cần relay/serverhọcport ởđườngnày. Chưa xácnhận điều kiện uplinkdiđộng/
ISP-CGNAT ởtrên; không dùngnhãnagent/IPprefix làm bằngchứng, giữN1OPEN và acceptance nguyênvẹn.
[Provenance/giới hạn/phép tiếp](PROTOCOL.md#rust-internet-observation).



### Diễn biến Rust trước khi rút câu hỏi hiện hành

Trạng thái lịch sử tại [CP251](PROTOCOL.md#rust-eight-teams): fixed1v7 code/TCP và native8Manner1v7negative trên8d; Freepositive2outcomes/selector0persist chưa có do copy4bD/8dGpost17preRoom. CùngCP250HGfamily/causalunknown, không retry hoặc SDKgraph khi chưa có observable mới; Tại mốc đó PLAN chuyển native9admissionbridge/consumer độc lập; CP252 phía trên đã đo gap này. Raw/hash/failedprebindchecker-harness được giữ, không nâng M4.

[CP231](PROTOCOL.md#rust-foundation) đã có M0/M1; [CP235](PROTOCOL.md#rust-m2-acceptance) chốt M2 scoped theo checklist:2actualfixedroom/lifecycle/barrier và healthyBsession sau EOF. [CP237](PROTOCOL.md#rust-movement-floor) có nativeUDP/movement hai chiều+floorRoller; [CP238](PROTOCOL.md#rust-bombs) thêmhaiownerbasicbom118/159/158; [CP239](PROTOCOL.md#rust-terrain) thêmterrain195flags/revealhidden27-28/ba165effects; [CP240](PROTOCOL.md#rust-life-results) thêm hai native outcome/reset/Ready2-Start3; [CP241](PROTOCOL.md#rust-return-cleanup) có activehostEOF→nativeRoom/chief/fence và dead-owner/drain kernelcontrols; [CP242](PROTOCOL.md#rust-m3-acceptance) chốt M3 scoped qua nativeitemreset/OScontest/exactoldUDP-currentpositive và queue controls; [CP243](PROTOCOL.md#rust-account-worker) có boundedworker/handoff/compensation; [CP244](PROTOCOL.md#rust-account-idle-rate) có idleTTL/rate/schema2; [CP245](PROTOCOL.md#rust-four-startup) có native4startup/room/loading/12remoteprogress; [CP246](PROTOCOL.md#rust-four-owner-bombs) thêmfour-ownerbom/knownstats/16native159158; [CP247](PROTOCOL.md#rust-four-life) có17/19→3peer170/native56→1/all4states và syntheticclear21completion; [CP248](PROTOCOL.md#rust-four-results) đã có same4opposite2v2/rank4-native175180/reset-Start3; [CP249](PROTOCOL.md#rust-four-abort) thêm scopedguest-hostEOF/unrankedreturn/threeRoom/chief; [CP250](PROTOCOL.md#rust-eight-baseline) có8Manner4v4/loading-mesh56-statbom8/life-rankreset; CP251 đã portfixedteams/nativeMannernegative và CP252native9bridge scope1KR+8synthetic; M4cònFree2outcomes/selector0persist/bridgeM5 riêng; [PLAN](PLAN.md#current-order) giữ thứ tự, không broadRE. Hai-client dùngfirstCRCrequest/endpoint/stagelease; WCPport biếnđổi đã giải theoCP006/103, khônglặpportonly. DiagnosticIP/ancillary shape policy chưa accountauthentication; unknownmode/header giữ giới hạn.



### Bảng RUST-Q trước khi rút diễn biến đã đo

| Câu hỏi | Có thể chọn backend policy design ngay | Bắt buộc đo KR / phép thử nhỏ nhất |
|---|---|---|
| RUST-Q1 authbridge/ownership | Account→player schema, Argon2id/PHC, onegamesession, expiry/revoke/ban, CLI/HTTPS riêng; accountDB không chờ BattleRE | [CP225](PROTOCOL.md#auth-fresh-process): processB mới cùngaccount có native0/ID-name-echo6/67/newlease-session/Aowner8s; badfresh32token→native1/nochannel/noBsession/Aowner45s. Priorselector11fromgetter/storebound, setterrequest1notreadback. SameprocessfatalCP224/before17receiverCP220blocked; chỉ mở khi có input/observable mới hoặc milestonechặn. Freshlogin khôngresume. [CP236](PROTOCOL.md#rust-account-backend) đã có SQLite/Argon2id/account-player/profile/sessionexpiry-revoke-epoch/CLI backend, 11tests+33CLIprocess; CP243đãcóboundedworker/handoff/exacttokencompensation; CP244đãcóidleTTL/rate/schema2; CP253 đã có library oneuse frontend32/redirect/channel owner và worker executionUTC/7ca6DB/3nativebodyreplay. CP254 đã có listener/private launcher/fixed2 native profile; fullauth/generalprofile/accountroom-game chưa đủ. TCP negatives đã có, CP256 có native freshlogin và exactnative1-6 replay senderPython bị từ chối; CP258 fixed2accountroom-basic/lifecycle/manualfreshrejoin đã có; DBqueue-native failclosed/generalprofile còn thiếu; CP255nativewrongcorrelator/revoke/expiry đã có theo [scope](PROTOCOL.md#rust-account-native-lifecycle), chưa modalclose/recovery/generalprofile; CP258 đã nối fixed2room/basic+manualfreshB/newround nhưng affectedBfatal, khôngsameprocessrecovery; [PLAN](PLAN.md#current-order) tiếp M4native và M5bridge độc lập, không đợi fullauth/quétSDK/VM/lặphello |
| RUST-Q2 dynamicroom/profileprojection | Registry/roomquota/sessionowner/maxslots/API kiểm quyền, DBprofile/name độc lập | [CP230](PROTOCOL.md#room-shared-cleanup): private nativeguestBack/EOF/hostBack/107IDclear/110UIchief; CP229join/move đã có. Đủ consumer đầu vào M2; M0→M1 không đợi module Python/reference hoặc roomintegration. Theo owner mới, chỉ RE discrepancy chặn lát cắt; typed seam/partialsend khi port phòng cần. Không broad RE/lặp solvedroom. Productionglobalroom/dynamicoccupancy/quota/thirdnativeotherroom/failure-send/restartfence cònthiếu; raw585zero không wireepoch; hostpostgetter unobservedbudget20 |
| RUST-Q3 inventory/consumables | Definitions/revision/ownership/quantity/expiry/reservation/transaction; worlddrop khác persistentstack; disabled unknownkr_id | Native query/equip/use/request/confirm/count/effect/reset/freshlogin cùng1knownitem. Kim145fixturecredit không inventoryauth; 4/8integration đo khi mở |
| RUST-Q4 progression/rewards | Explicit nonredeemableformula/version, result/grantunique/ledger/outbox/backendtransactions. WikiEXP chỉ tham chiếu product | Actual175profile/resultfield→XP/currency/count/UI/relogin consumption; đo 0 và1delta/control/bounds, không portVN173/layout hoặc coi bảngwiki làfieldmapping |
| RUST-Q5 authority/freshness | Giữ fuse/drop/reportguard/fence policy đã đo; backendreward pending khi outcome chưa đủ tin cậy | CP242 exactoldUDP/currentpositive đã đo trên Rust; còn oldTCP/membership/IDreuse và phạm vi ngoài keywindow đã kiểm; full142distance/auth/low8wrap nếu validator/deployment cần. Không hứa server lọc mọiP2Ptraffic/fullworldauthority |
| RUST-Q6 operation/capacity | SQLiteWAL/backup-restore/adminaudit/config/metrics/queue limits design không cần mọi gameplayRE | [CP231](PROTOCOL.md#rust-foundation) foundation/14TCP/SIGTERM/rebind và [CP235](PROTOCOL.md#rust-m2-acceptance) nghiệm thu M2 scoped/healthyBsessionEOF; guardprofile trướcbind khácnativewrongversionUI. CP241activehostEOF đã có nativeRoom, loading/selectedEndingEOF và fullrecovery giữ giới hạn riêng; shutdownTCPtimeoutchưarõrootcause/CP237táihiệnkernel/PIDgonepostinspection/repeat22casesđạt, targetload/SLO/generalbackpressure/DBfault. CP236SQLite realbusy/FULL/audit/entropy/clock/schemafault có backendtests; CP243boundedworker/rawSQLite-threadfaults đã có; OSdiskfull/worker-nativeintegration/nativeDBfault/restore/SLO chưa đủ. CP240 đã có native life17/19/170 và hai175/180/reset/Start3; M3scopedacceptedCP242; loggerR4maskederror được sửa supervision, kernelR2flush timeout event141 chưa rõrootcause/repeat51TCP+nativeR5pass; không tăngdeadline. CP243worker/handoff đã có; CP244idleTTL/rate đã có; tiếp M4 và M5bridge riêng trong PLAN; opaque123/10 chỉobservepolicy/semanticsunknown, 167/body4-16-32 cũngboundedopaque/noresponseunknown;CP239142currentclearance/fullroster158mới mởđúnghidden27/28;old142saucurrentclear vẫnkhôngwireepoch. REchỉnếucầnresponse/field. REonlyportdiscrepancy. Accountresume/matchrestore khác freshlogin/connection; workload/RPO/RTO riêng  CP245fourstartup đã đo Manner1/2v2/4ACKbarrier/12remoteprogress; không lặp positive. CP246fourbom đã có; CP247giải gap12717/19→3peer170/native56→1/all4states; clear21fanoutđãkiểmTCP/nativeclear21chưađo. Không đo lại startup/bom/life17-19positive để tăngCP. CP2508baseline69tests/16eightTCP/native56mesh-8ownerbom-7peerlife/opposite4v4-reset-Start3 đã có; R1Dcollateraldead giữđúngproof. NativeR3H/R4Gpost17EOF causalunknown/R5priorargvpositive không SDKfix; UDPcapturepartial3Aframes/fourIPthreadguard,56nativeprogress/fullTCP độc lập; sửa capture trongmeaningfulteamvariantcontrol/no repeat2cycles chỉcapture; chỉ mở lại khi chặnvariant vớiobservablemới. 142/16722 chỉ boundednoeffect, không reveal/lifeacceptance; CP247clear21đãđợiđủrecipient/currentvisit; CP248outcome đã đợi allrecipientlife delivery/currentwitness/drain/latestdeath-blast/settle8 và có nativefull175180/2outcomes-Start3; CP249scopedguestD-hostA EOF/unranked180-post585-107IDclear/threeRoom/chief đã có; CP250known8baseline đã có; nextfixedFree1v7/selector/Mannernegative/native9admissionbridge, không twoforfeit/fullM4. Aảnh lúcrank chưa thấypanel, nativeAparser-rank-reset cóproof; đo Aảnh muộn trong plannedlifecycle nếu cần, không repeat2cycles. KernelR1shutdowndeadline/PIDgone vẫnchưarõcause. CP249normalR2/R3logflush292/208 tái hiện/R2threadreserve_bytes.cold; nguyênnhâncausalchưarõ. Explicittmpfs32four/51two/haiEOFnative/exporthashpass không là Btrfsfix/durablelog; không tăngdeadline hoặc rerunvôhạn, đo causalI/O tạiM8 khi cótrigger. Phép port/đo tối thiểu: CP201fourowner/capacity-power-fuse →170đủ3recipient/currentclear21 →CP248rank4/opposite2v2/reset/Start3đãcó →CP249abort/EOFportđãcó→CP250known8baselineđãcó→fixedteamvariants/native9admissionbridge. Publicexporter chỉkernel-tested; nativeprivatepreparer/R3retainedinputs khácversion đúngprovenance. BtrfsD-state lúccleanup twoTCPR1 không chứng minh nguyên nhânEOF; nếu tái hiện giữPID/thread/wchan, không tăngdeadline.|
| RUST-Q7 productpolicy còn mở | Chọn baseline mộtchannel/map/basic, diagnosticreward0 rồi internalnonredeemable, onegamesession/refusesecond, ephemeralrounds; khôngpayment/HA banđầu | Giai đoạn 1 đã chốt lưu kết quả và thưởng 0; XP/tiền tệ/inventory thuộc giai đoạn sau. Trust/abuse thresholds và reward khác chỉ mở khi product đổi phạm vi; client projection mới vẫn cần consumer proof |



### Bảng gate lịch sử tại CP212

### Tiến độ hiện hành theo bằng chứng

| Task/mốc | Trạng thái theo nguồn dẫn, cập nhậtCP212 | Nguồn chuẩn |
|---|---|---|
| KRE-004/issue6 | Đã review/merge PR7 và đóng issue:recipe r1 đến local accept;không phải full K1 | [Wine r1](evidence/wine/README.md) |
| Bootstrap reply | PR8 đã merge;debugger ProcessPatchreturn1,không ép success | [bootstrap evidence](evidence/bootstrap-wire/README.md) |
| KRE-005a/issue9,R2 | PR10 đã merge:3 cold start cửa sổ/second accept;issue9 còn mở | [R2 evidence](evidence/wine-r2/README.md) |
| WCP hai-reply | Run CP001 là lịch sử chưa C→S; CP006 đã đo request đầu trên copy tham số hóa | [PROTOCOL](PROTOCOL.md#wcp),[WCP evidence](evidence/wcp/README.md) |
| KRE-005b/issue14 | CP202–207four2v2/bundle/bounds/EOF/health/freshserver/UDPimpairment đã merge; CP208adapter8/nativeclock đã merge; CP209mesh/stat/8ownerbom đã merge; CP210same8two4v4/reset đã merge; CP211native8relocatedbundle có runtime. K6c WAN giả lập/K7 đạt CP212; fullauth/authority/phần ngoài scope là backlog, task ở current-order | [Q-023](RESEARCH.md#q-023), [service](evidence/probe-service/README.md) |
| K0 | Đã nghiệm thu identity/harness tại CP178: originalWindowsVM fullfixedcopy/PID/token/cwd/window/ETW/isolation/cleanup; không bootstrap/gameplay success | [Acceptance](PLAN.md#k0-k2a-acceptance) |
| K1 | Đạt CP179 cho recipe offline R2: 3 coldstart/API đúng PID/contract/patch controls; không e9cold3 hoặc originalWindows bootstrap | [Acceptance](PLAN.md#k1-acceptance) |
| K2a/K2 | K2a đạt CP178; K2 đạt profileoffline CP181 với native sameprocessretry/reset/cursor/stream và boundednativeoracles. Không toàncodec/accountauthority | [Acceptance](PLAN.md#k2-acceptance),[contract](PROTOCOL.md#login-retry-codec) |
| K3 | Đạt CP186 trong scope A/local/Wine/một kênh normal; cold3cache/version/identity/stage và nguồn CP015/016/034/058/181–185 đã đối chiếu. Không fullauth/Windows/multichannel-caption hoặc hai-client291 | [Nghiệm thu](PLAN.md#k3-acceptance), [contract](PROTOCOL.md#k3-version-identity-controls) |
| K4 | CP187 đạt riêng phòng/load/start local currentAB291/copyABIcorrected; oneoutcome/reset/Start2, không gameplay2outcomes | [Nghiệm thu](PLAN.md#k4-acceptance) |
| K5 | Basic local đạt CP188; current2cycles/peer/Bowner/EOFcontrol riêng, designauthority được rà. Không fullinventory/mode/map | [Nghiệm thu](PLAN.md#k5-acceptance) |
| K6a | Local hai actualAB/basic/cùnghost đạt CP189; nativecontestedindex/reset/life và CP188peer/Bowner/EOF cóscope riêng. Không physicalLAN/Internet | [Nghiệm thu](PLAN.md#k6a-acceptance) |
| K6b | Đạt CP191 cho LANgiảlập hai netns/IP/NIC/actualKR theo owner; physicaltwohosts không required | [Nghiệm thu](PLAN.md#k6b-acceptance) |
| K6c WAN giả lập | Đạt CP212 sau chỉ đạo môi trường mới và đối chiếu CP192–196 native/routing/NAT/UDP/two cycles/EOF. Internet thật chưa kiểm | [Acceptance](PLAN.md#k6c-acceptance), [topology/giới hạn](PROTOCOL.md#emulated-wan-acceptance) |
| K7 | Đạt CP212 sau đối chiếu riêng từng tiêu chí với CP190/198/203–207; K6c dependency đạt scope WAN giả lập. Không full game/fullauth/tám client là điều kiện | [Đối chiếu K7](PLAN.md#k7-evidence-review) |



## CP272 — tài liệu contract theo chủ đề

Contract đã chia thành 13 chủ đề, giữ các anchor lịch sử; R1g đã merge tại thời điểm đó. Đợt này chỉ sửa tài liệu
và công cụ kiểm, không thay runtime hoặc phạm vi nghiệm thu. [Record](evidence/checkpoints/records/CP272.json).


## CP273 — relay opt-in prototype hai người

Relay đã nối account channel và room/round ownership; native xác nhận hai chiều, hai vòng, reset, gói cũ tới cổng đã
nghỉ và EOF. Chưa đủ relay mesh hoặc vận hành lâu dài. [Record](evidence/checkpoints/records/CP273.json).


## CP274 — hồi quy native cuối R1

Các profile direct legacy 2/4/8 người và account đạt hồi quy refactor trong phạm vi hiện hành. Lỗi export/capture và
raw phục hồi được ghi riêng, không biến thành pass toàn harness. [Record](evidence/checkpoints/records/CP274.json).


### Mốc trước CP299

**CP298: Rust đã tách terrain theo lựa chọn map; Forest07 có lát native roster2/4/8.**
Seed riêng, thùng, một Roller ẩn theo policy thử nghiệm và hai kết quả/reset đã được đối chiếu native.
Phép đo cũng sửa ngoại suy footprint của giếng3×2. [Contract và giới hạn](protocol/terrain.md#rust-selected-map-terrain),
[bằng chứng](evidence/rust-service/g1-map-terrain.json). Forest07 và G1 vẫn còn roster/nội dung/drop theo acceptance;
M8 giữ phạm vi riêng tại [Bảng điều hành](PLAN.md#dashboard).
