# Câu hỏi RE của server KR

Đây là backlog riêng KRE. Các ID Q-022…Q-029 được giữ để truy nguồn; không thuộc gate V1 Rust.

Mở theo yêu cầu trực tiếp 26/10/2026. Đây là câu hỏi của **`KR-00967-EMU-1`**, không đổi câu hỏi/gate
V1 Rust. [Lộ trình KRE](PLAN.md) giữ thứ tự
task và điểm dừng; [contract](PROTOCOL.md) giữ ma trận/nguồn.
KRE-001/002/003 giữ backlog theo PLAN và issue hiện hành; owner khi mở nhánh: từ28/10 Codex solo RE/runtime và tự kiểm theo
[miễn review của chủ dự án](evidence/checkpoints/review-waiver.json). Report dsh/Claude cũ giữ lịch sử.
Review plan nguồn đã xong; các câu hỏi dưới đây vẫn open cho tới khi có evidence trả lời.

## Câu hỏi hiện hành

Thứ tự task chỉ ở [PLAN](PLAN.md#current-order). Tra phạm vi đã biết qua
[coverage/cây chức năng](COVERAGE.md); bảng dưới dẫn đến khoảng trống/phép đo theo cơ chế.
Nguồn cũ và run/checkpoint giữ ở [HISTORY](HISTORY.md) và manifest; kết luận tại CP cũ không
được đọc thành trạng thái hiện tại. Tra mức RE/implementation/runtime/gate theo từng chức năng ở
[coverage](COVERAGE.md), contract/giới hạn tại link của từng cơ chế trong bảng gap. Không duy trì
một danh sách tiến độ theo CP khác tại đây; [README](../README.md) giữ mốc hiện tại,
[PLAN](PLAN.md#current-order) giữ thứ tự. Full M5 chưa nghiệm thu. K6c WAN giả lập/K7 giữ scope riêng; N1 Internet thật đã đạt phạm vi một phía tới được, không chứng minh mọi ISP/CGNAT.

| Câu hỏi / owner | Trạng thái | Khoảng trống và phép đo hiện hành |
|---|---|---|
| <a id="q-022"></a>Q-022 bootstrap / Codex tự kiểm | open | [ProcessPatch/control](#gap-process-patch), [WCP/config](#gap-wcp-config) |
| <a id="q-023"></a>Q-023 codec / Codex tự kiểm | open | [Socket/seq/mode/bounds](#gap-socket-codec), [service actual frame](#gap-diagnostic-service) |
| <a id="q-024"></a>Q-024 login/kênh / Codex tự kiểm | open | [Auth/identity](#gap-auth-identity), [sảnh](#gap-redirect-lobby), [cache](#gap-profile-cache) |
| <a id="q-025"></a>Q-025 phòng/load / Codex tự kiểm | open | [Tạo/join](#gap-create-join), [roster](#gap-roster-ready-host), [barrier](#gap-map-spawn-loading), [membership](#gap-room-membership) |
| <a id="q-026"></a>Q-026 authority/trận / Codex tự kiểm | open | [Bom](#gap-bomb-lifetime), [chain/block](#gap-blast-block-chain), [pickup validator](#gap-floor-pickup), [item](#gap-item-stats), [contact](#gap-contact-state), [result/reset](#gap-round-reset) |
| <a id="q-027"></a>Q-027 peer / Codex tự kiểm | open | [Movement](#gap-peer-movement), [freshness](#gap-udp-freshness), [relay/NAT](#gap-peer-relay), [EOF](#gap-disconnect-replacement) |
| <a id="q-028"></a>Q-028 profile/core / Codex tự kiểm | open | [Authority matrix](PROTOCOL.md#authority): audit creator/simulator/validator/broadcast/peer-editable; lựa chọn lai/shadow core là design, chưa là contract client |
| <a id="q-029"></a>Q-029 vận hành / Codex tự kiểm | open | [Hardening/demo nhiều máy](#gap-service-hardening); theo gate K7, không suy từ TCP accept |

<a id="gap-codec-retry-followup"></a>
### Codec retry đã đo CP181; giới hạn còn lại

[Contract/sequence native](PROTOCOL.md#login-retry-codec) giữ nguồn chuẩn: validresult1→UI1002→
sameprocessTCPloginretry→auth137cursor/errors0→channelreset. [Controls/hash](evidence/live-codec-stream/README.md#cp181)
giữ fatalCRCcleanup/WineDbgHWwatchpointfailure/stubordinalcontrol. Không lặp fatalclose hoặc stubDWK.
K2profile được nghiệm thu theo [PLAN](PLAN.md#k2-acceptance); không fullgrammar/allmodes/headerauthority/
accountresume. Widercodec chỉ mở khi có trigger cụ thể từ K3; không quét mọi variant trước sảnh.
Owner/tự kiểm Codex, review waived; AUD062 là lịch sử. [Policy audit hiện hành](evidence/owner-directives/feature-group-audits.json) theo nhóm tính năng từ sauCP256, không lịch4CP.

<a id="gap-lobby-identity-followup"></a>
### Phòng tổng quát, đội và nội dung 4/8 còn thiếu

M4 đã có đội cố định và Manner; Free 1v7 là giới hạn đã chốt, không retry khi chưa có observable mới.
[Phạm vi/bằng chứng](protocol/multiplayer-team.md#rust-eight-teams), [đích](PLAN.md#delivery-target).
Còn interactive chọn đội/nhân vật, phòng đầy/vacancy/replacement tổng quát trong M7; terrain/item/kim/cứu 4/8 thuộc G1.
Chỉ đo consumer hoặc lifecycle đang chặn implementation; không lặp parser/codec/positive đã có để tăng checkpoint.

<a id="gap-real-internet"></a>
### Internet thật: khoảng trống ngoài N1/N1b đã đo

N1 một phía tới được đã kết luận đạt; N1b hai NAT đổi cổng đã kết luận âm sau hai lượt, không retry baseline.
[Nguồn/phạm vi](PLAN.md#rust-n1-result). Câu hỏi kiến trúc trực tiếp hay relay cho recipe đó đã có kết quả;
phần mở là [mở rộng và vận hành relay](#gap-relay-endpoint), PMTU, port reuse/restart, topology và NAT/ISP khác.
Không suy phân loại CGNAT từ hotspot, IP hoặc nhãn; chỉ mở phép đo môi trường khác khi PLAN yêu cầu.
K6c WAN giả lập/K7 không nhận thêm dependency Internet thật hoặc thiết bị vật lý.

<a id="gap-rust-service"></a>
<a id="rust-service-gaps"></a>
## Unknown chặn dịch vụ Rust — KRE-074

[PLAN](PLAN.md#dashboard) giữ thứ tự; [kiến trúc](RUST_SERVICE_ARCHITECTURE.md) giữ design/readiness.
R1a–R1g và native hồi quy các profile 2/4/8/account đã đạt phạm vi refactor. [Bằng chứng và giới hạn](evidence/rust-service/r1-native-final.json). Không suy thêm tính năng từ unit/differential hoặc native hồi quy.
[M5b profile DB](protocol/auth-session.md#rust-db-account-profiles) đã có account tạo sau readiness, native hai ván,
chung nguồn TCP, gameplay khi DB/queue bận và process KR mới đăng nhập sau phục hồi. Graceful retry/resume còn
giới hạn; profile khác zero/ASCII và progression chưa đo.
Không lặp khoảng trống đã đóng hoặc suy native shared-NAT gameplay từ TCP source mapping.
[M7a](protocol/room-membership.md#rust-managed-room-selection) đã có native tạo/chọn/rời hai phòng bằng bốn profile.
Occupancy đã có [native contract](protocol/room-membership.md#rust-room-population); [lifecycle managed 2/4/8](protocol/room-membership.md#rust-managed-team-rosters) đã đạt trong tập cấu hình đã đo. [M7c room-only](protocol/room-membership.md#rust-managed-player-choices) đã có proof; [ownership tích hợp](protocol/room-membership.md#rust-managed-account-ownership) đã đạt trong tập profile hiện hành; M5b bổ sung profile DB. N1R cần proof riêng bên dưới;
M4 Free 1v7 là giới hạn đã biết; M7/G1/M6/M8 và điều kiện bàn giao giữ tại PLAN, không broad RE toàn client.

<a id="gap-client-utf8"></a>
### UTF-8: phần còn mở sau bản vá 949x

[Bản vá 949x](content/client-language.md#utf8) (CP310) đã hiện chữ ba ngôn ngữ và nhận chữ Việt/Hàn ở chat trên client
chẩn đoán. Câu hỏi còn mở và phép kiểm tối thiểu:
- [Chat L1](protocol/lobby-profile.md#rust-patched-room-chat): chat có dấu hai chiều đã thấy trong phòng trên gói vi/ko. Không suy đường chat 2P, team, whisper hoặc guild.
- Tiệm: cần dịch vụ có tiệm; sau đó chụp tiệm ở cả ba bộ dữ liệu.
- Sửa tên phòng: Wine ko đã gõ tiêu đề trộn và hai bản cùng thấy. Nếu mở rộng nhập liệu, thử gõ mới trực tiếp từ Windows vi; không đồng nhất với việc Windows vi đọc tiêu đề đã gửi.
- IME tiếng Hàn thật (composition từng âm tiết) chưa thử. Hai bản đã nhận câu Unicode trên gói qua đường helper; ca này không dùng IME thật.
- Hai lượt trước mất quyền phiên sớm hơn idle/TTL. [CP370](evidence/checkpoints/records/CP370.json) không lặp mất quyền trước kết quả. Nếu tái diễn, đối chiếu nguyên nhân đầu từ kiểm phiên, relay hoặc luồng KR; chưa biết nguyên nhân hai lượt trước.
- Consumer bỏ qua hai slot chuyển mã (bộ giải riêng, GDI) chưa được loại trừ; chưa thấy chỗ nào sai trong các màn đã chụp.
- Tên account có [biên UTF-8/949x strict](protocol/auth-session.md#rust-patched-account-names). Hai tên có dấu và tiêu đề trộn đã hiện đúng ở hai đầu cặp vi/ko.

<a id="gap-room-occupancy"></a>
### Occupancy trong list và lifecycle phòng động

M7a đã nối các consumer phòng cũ; không cần phục hồi lại 81/82/87/91/106/107/110 chỉ vì đổi implementation.
[Hai field occupancy đã đo](protocol/room-membership.md#rust-room-population): offset 5/7 của tail là current/capacity;
20 byte còn lại giữ opaque. Không lặp reader/list đã giải. [Lát cắt duel](protocol/room-membership.md#rust-managed-duel-lifecycle)
đã có hai room chưa đầy với Ready/loading, kết quả/reset/trận tiếp, key/queue/timer riêng và EOF một phòng khi phòng
kia tiếp tục. Stat của Start hai hàng được đo theo slot; không gắn stat vào identity. Thiếu B→A progress trong lượt
lifecycle đã được giải bằng phép đo Right700ms riêng cùng binary; không cần lặp toàn chu kỳ để giải input bị chặn.

M7b đã có [native managed team rosters](protocol/room-membership.md#rust-managed-team-rosters) bên cạnh duel:
Start/stat/peer/life/rank/reset theo contiguous4/8 và EOF/bom chờ cách ly ở hai phòng4. Không cần đo lại profile fixed
hoặc từng giá trị aux142: aux là quan sát opaque không hiệu ứng theo design, chưa có nghĩa hoặc item authority.
Profile team cũ giữ fence khi EOF. [M7e](protocol/room-membership.md#rust-selected-player-rounds) có opt-in riêng cho
roster hiện hành và active EOF; không áp kết quả đó ngược lên profile cũ.

[M7c room-only](protocol/room-membership.md#rust-managed-player-choices) đã đo C2S92/u8+u16, C2S97/u8,
slot110/native visual/peer và Ready clear/reReady cho nhân vật0/1, nút đội A0/B3. Lỗi phòng đầy generic87/result2 và
vacancy được kiểm bằng ba client thật, không có slot ma. Không cần đo lại producer hoặc reader đã giải. Marid4
mới có producer, chưa được mở stat/entitlement; nhiều local player và đội khác chưa hỗ trợ.

[M7d ownership/failure tích hợp](protocol/room-membership.md#rust-managed-account-ownership) đã có native revoke cách ly
hai phòng và ca âm tính SQLite/TCP; không lặp baseline. M5b đã bổ sung native DB/queue fault riêng; giới hạn ở contract account.
[Roster và active EOF](protocol/room-membership.md#rust-selected-player-rounds) đã có native3/6 người, slot0/2,
Start/stat theo nhân vật0/1, UDP/bom/kết quả/reset và4→3→2 sau hai lần mất endpoint. Không lặp các baseline này.
M7e đạt ca tối thiểu theo PLAN; M5b đã nối hồ sơ DB và native DB/hàng đợi theo acceptance giai đoạn 1.
Subtype11 chỉ ẩn peer nên không dùng cho forfeit; death19 tại0/0 là policy đã đo, không là luật authority gốc.
Loading và Ending loss chưa có survivor recovery; interactive UI exit202 và active host loss chưa có native mới.
Các giới hạn này giữ riêng, không suy từ guest EOF hoặc từ source/TCP. Epoch hiện trần255 lần cấp/instance; mixed key không là auth hoặc
durable restart/reuse proof. Relay nhiều người/reuse/restart giữ N1R-b, không gắn vào M7 LAN.

<a id="gap-rust-life-actions"></a>
### Kim/cứu Rust đã có lát native; chỉ đo khoảng trống của nội dung tiếp theo

[Contract kim/cứu](protocol/life-results.md#rust-needle-rescue) đã có native hai client cho kim/cạn/reset và ba client
cho cứu aux1, peer fanout, kim giữ lại và hai kết quả/reset. Không lặp lại consumer/cặp wire đã giải khi chưa có
sai lệch mới. Credit/catalog hiện một bản ghi145/profile; multi-charge, nhiều kim trên map, aux2 và native Rust
cho roster khác chưa được đo. Khi nội dung G1 cần biến thể đó, dùng một ca current positive và ca không đủ điều kiện
với cùng profile, chỉ mở phần khác biệt. Khoảng cách/contact authority và tick mới nhưng body thuộc vòng cũ vẫn
không xác thực được từ wire hiện có; không gọi guard nội bộ là token client đã phục hồi. Packet167/body8 sau pickup
chỉ là envelope quan sát opaque; không suy semantics hoặc ACK. State0 ngắn trước peer clear không có causal fix mới.

<a id="gap-rust-player-terrain"></a>
### Terrain/item nền theo roster đã nối; chỉ đo dữ liệu và consumer khác biệt

[Contract Rust](protocol/terrain.md#rust-player-terrain) đã có native roster2/4/8 trên ExVillage01, ba item nền,
stat nhân vật0/1, power1→2, hidden reveal và hai kết quả/reset. Không lặp codec/claim/reader hay đường account.
Terrain map khác còn phải phân loại ô và đối chiếu native seed→blast→drop; item ngoài Bomb/Fire/Roller cần consumer
và ca không đủ điều kiện trước bật. Source max9 không là native mọi power. Effect snapshot chỉ có slot0/1 trên mọi
view; geometry cuối vòng2 và native sparse-item/EOF của lát này chưa đo. Chỉ bổ sung nếu nội dung hoặc discrepancy
cần đến; không suy chúng đã đạt từ test model. Raw142 thiếu epoch và authority theo report giữ nguyên giới hạn.

<a id="gap-rust-selected-map-terrain"></a>
### Terrain theo map và Forest07: không dùng phân loại từng ô cho object đa ô

[Contract](protocol/terrain.md#rust-selected-map-terrain) có seed/floor riêng theo map, source/TCP/negative controls
và các route native Forest07 roster2/4/8 account, hai vòng. Native seed hai view chỉ ra giếng3×2 làm năm ô ground candidate
thực tế bị chặn; cạnh giếng đã có actual blast và flags không clear. Không suy mọi objectStatic cùng footprint hoặc
các unknown map khác đã giải. Forest07 còn drop/nội dung và sparse roster theo acceptance; một Roller cố định chỉ là
policy thử nghiệm. Khi mở phần tiếp, kiểm đúng consumer/placement hoặc roster còn thiếu, không lặp clock/map lookup.
Phân phối bảng map kết hợp ItemAttr và authority gốc chưa phục hồi; policy công ty phải khai design và giữ unknown
item tắt. [Patrit14](protocol/terrain.md#rust-patrit-terrain) đã đo ba ô chặn của MastTileA và route phía trên/power1 với
roster2/4/8, hai vòng/reset. Không lặp seed/route đã đủ; còn drop, nhóm item và các biến thể chỉ mở khi cần.
[Village10 seed/Herb](protocol/terrain.md#rust-village10-herb) đã có native hai/sáu client cho flags, ba root power2 và
đối chứng nhà; [terrain tích hợp roster2/4/6](evidence/rust-service/g1-village10-terrain.json) đã có hai vòng/reset,
Fire/Roller và toàn195flags mỗi blast. Aux report142 không là discriminator trong policy item opt-in, nghĩa gốc vẫn
unknown; không RE dàn trải các giá trị khi đã có current-cell guard. [Catalog tám/sáu spawn và sparse6](protocol/load-phase.md#rust-map-spawn-capacity)
đã có Start/tọa độ và hai vòng/reset. Còn coverage roster khác, mọi hướng/item/drop; Camp08/Tomb12 có loại ô khác.
Không lặp run sparse6 chỉ để lấy cặp flags cuối vòng hai bị budget probe bỏ qua: công bố đúng giới hạn19/20 blast.
Nếu lát nội dung kế cần quan sát đủ, tính budget từ số clock/blast trước freeze; không sửa raw cũ. Ba channel ở run lỗi
trước đóng QueueExpired trước mainDisconnected; run tách preboot không gặp lại, nhưng nguyên nhân resource/scheduling
chưa xác định. Chỉ mở chẩn đoán tiếp nếu tái hiện hoặc cần budget M8; không patch JSON downstream.

<a id="gap-camp08-bomb-report"></a>
### Camp08: coverage sau route thùng nhiều hit

[Contract và giới hạn](protocol/terrain.md#rust-camp08-boundary) giữ policy122 và route StrongBox/hidden Bomb/Fire
hai vòng/reset cho roster 2/4/8. Herb/Line, footprint/ray static và Line01 ray đã có các đối chứng riêng; không lặp
parser, initializer, game-box sai nhánh hoặc route đã đạt khi chưa có discrepancy mới. Report122 không mang epoch/
serial nên raw freshness vẫn unknown, không có authority trên ledger. Chưa đo trực tiếp actor/getter đi vào Line01
hoặc mọi hướng/sparse roster; không dùng cờ 0 hoặc type 3 để suy mọi khả năng đi qua. Tiếp nội dung còn thiếu theo PLAN.

Storage/log budget và mất mẫu feedback B ở lượt bốn người đầu vẫn chưa rõ. Lượt đạt không kích hoạt control focus,
nên không chốt nguyên nhân. Giữ QueueExpired, log-flush, preboot và capture failures trong evidence; không suy
production fix hoặc độ bền từ tmpfs. Xem lại khi cản G1 hoặc tại M8; L1 vẫn do Claude phụ trách.

<a id="gap-tomb12-terrain"></a>
### Tomb12: route 2/4/6 đã có, còn coverage nội dung

[Contract](protocol/terrain.md#rust-tomb12-boundary) và [ma trận](evidence/rust-service/g1-tomb12-serving-rosters.json) đã có route 2/4/6, hai vòng/reset; không lặp route hoặc body đã đủ nguồn.
Còn hướng/roster khác và nhóm item theo PLAN. Ground+AC vẫn opaque nhưng không chặn route; chỉ mở thêm consumer khi có discrepancy cụ thể ở nội dung kế.
Lượt 6 giữ overall exit 1 và giới hạn capture/debugger/cleanup tại contract; chỉ quan sát native được xác minh riêng. Giữ lỗi setup/verifier/export cũ, không chứng nhận M8.

<a id="gap-factory07-terrain"></a>
### Factory07: route 2/4/8 và static đã có, còn coverage gameplay

[Terrain và route](protocol/terrain.md#rust-factory07-terrain) có đủ hai lớp 195 cờ, bốn loại thùng, Fire/Bomb ẩn và hai vòng/reset/ván kế cho roster 2/4/8. Direct native item stat đã kiểm mọi recipient; report ghi nhầm field được sửa bằng chứng bổ sung, không đổi raw.
Bốn tên static có initializer/source/footprint; Block00/01/02 có ray qua hai vòng, Glass00 có một hướng movement/ray power 2 và tích hợp Rust. Lượt Glass giữ lỗi debugger sau completion và verifier thiếu cài probe cũ.
Tiếp các hướng/roster và nội dung còn thiếu theo acceptance G1; không lặp initial arrays, route trên hoặc body cache. Không sửa exporter hoặc suy gameplay đẩy thùng từ RTTI/dword 28; các ability còn theo nhóm item.

<a id="gap-village10-map"></a>
### Map mới: chọn map đã đo, còn coverage terrain/item và giới hạn roster

[Adapter](protocol/load-phase.md#rust-map-profile-adapter) và [UI chọn map](protocol/load-phase.md#rust-room-map-selection)
đã có Factory07/Village10: current96/117, metadata/checksum, lookup/name/spawn, basic hai kết quả/reset/ván tiếp và
journal đúng map. Không lặp ordinal-only, bảng2row hoặc chọn map khi không có discrepancy mới. Initial96 trước final
setup được đo; không áp lại guard setup=true gây lỗi. Queued selection dùng revision, không dùng index làm epoch.

G1 giữ [giới hạn roster đã chốt](evidence/owner-directives/g1-map-roster-limits.json): Village10/Tomb12 2–6;
bốn map khác 2–8. Với [F1-a](PLAN.md#f1-normal-battle), đối chiếu spawn, đường đi cạnh vật cản, nổ sát thùng
và pickup/reset trên map có placement hoặc drop; chỉ mở RE ở ô hoặc item khác biệt. [CaJJ_Clover đã qua đối chứng](evidence/rust-service/f1-pack-map-adapters.json);
Cookie06/Xmas15 có pickup quan sát được, còn kiểm stat/reset; các họ khác vẫn cần đối chiếu ô và item riêng.
Patrit22 có descriptor MastTileA và anchor row6/column6 trùng Patrit14 trong pack; [metadata candidate](evidence/rust-service/patrit22-adapter-candidate.json) giữ các kiểm âm.
Hai lượt thử native chạy chồng đã dừng và không dùng bằng chứng; cần một lượt native đơn có lease để xác nhận hoặc loại bỏ chiếu footprint. Ca `account_worker_queue_expired` giữ raw; lượt sau đã qua về phòng, chưa quy nguyên nhân.

Factory07 dùng bảng tám màu đội, khác bốn nút A/B/C/D của ExVillage01; click cũ phát đội2 bị guard0/3 từ chối.
Chỉ chọn màu3 đã hỗ trợ trong lượt đạt; chưa có contract phục vụ đội2/4 hoặc mọi màu, không mở scope từ ảnh UI.
Một phép TCP diagnostic rộng sau bốn cache positives timeout ở idle host Back/return241, rồi shutdown wait;
cause chưa rõ. Native Factory07 và cache TCP hẹp có bằng chứng riêng, không làm phép rộng thành pass. Nếu cản
G1/M6/M8: đối chiếu queue/guard/deadline và cleanup trên exact profile, với current positive; không mở broad client RE.
Một lượt TCP persistence cũng timeout setup trước peer106/map117; raw/frozen/cleanup đã giữ tại
[evidence nhân vật](evidence/rust-service/m6-character-serving.json). Lượt diagnostic có fallback15/31 nhưng không
chạy fallback, lượt tool gốc sau đó đạt; chưa xác lập nguyên nhân hoặc cách sửa, không suy CPU/timer là nguyên nhân.

### M6: acceptance thu hẹp đã có; chỉ mở discrepancy chặn G1/M8

[Journal serving và native relogin](protocol/life-results.md#rust-match-journal-serving) đã kiểm snapshot account/
room/round, commit-before-Start/result, retry/DB bận, duplicate/conflict, active departure, kill sau121/175 và CLI.
[Lựa chọn0/1](protocol/auth-session.md#rust-persistent-character) có persistence/relogin riêng; G1 #457 đã đo từ chối C4 bằng current110, giữ socket rồi chọn C0→C1 và chơi hai vòng/reset. Mở thêm nhân vật vẫn để sau. Không RE lại hoặc lặp
baseline đã đủ chỉ vì schema/codec giống nhau. Chỉ mở phép đo nhỏ nếu G1/M8 làm lệch current ownership, projection,
result hoặc rollback. Queue/journal latency dưới tải và vận hành/recovery theo acceptance M8; match restore và economy
là scope sau. Native kết quả không tự giải timeout setup/Back cũ, nguyên nhân vẫn chưa xác định.

<a id="gap-relay-endpoint"></a>
### Relay endpoint sau N1b — KRE-074/KR-F015

Owner và tự kiểm: Codex. Relay đã tích hợp; reverse-edge forwarding có **kr_runtime/confirmed** trong phạm vi
prototype hai người. [Contract](protocol/udp-peer.md#rust-relay-endpoint-scope),
[evidence và reuse](evidence/rust-service/n1r-fixed2.json), [acceptance](PLAN.md#rust-relay).
[Nền companion có xác thực](protocol/udp-peer.md#rust-authenticated-companion-foundation) đã có native control hai
người/route tĩnh trong LAN. [Serving relay](protocol/udp-peer.md#rust-authenticated-serving-relay) đã nối quyền
account/phòng và control từng vòng; đã có native bốn/tám người, roster thưa và survivor sau EOF. N1R-b đạt các hành vi theo [ma trận acceptance](evidence/rust-service/n1rb-controls-acceptance.json); phần vận hành/triển khai còn ở dưới. Không mở lại N1b baseline hoặc RE toàn peer graph. Các câu hỏi bounded về KR chấp nhận reverse socket,
hai chiều/hai vòng, stale tới cổng đã nghỉ và EOF/pastdue đã có kết quả; chỉ phần sau còn mở.

| Unknown | Backend policy hiện có | Phép đo tối thiểu khi milestone cần |
|---|---|---|
| Spoof first datagram giữa session chung IP | Account cấp route, IP là restriction, pin tuple đầu; không nhận CRC/key là MAC | Chọn companion QUIC có pin/capability ngoài native wire; sai capability/route/replay đã kiểm transport. Production Hub/Bridge/QUIC test cùng IP đã kiểm first foreign route/seq1 và own seq1 positive; native chung nguồn TCP/UDP có positive riêng. Không giả test là native attack. M8 còn threat/budget trên gói triển khai. |
| NAT rebinding hoặc lease hết hạn trong trận dài | Từ chối tuple mới, queue và lease có TTL hữu hạn | Đo migration/timeout với current positive nếu chọn hỗ trợ; hiện tại fail closed, không tự rebind. |
| Port reuse/restart và gói cũ gửi vào cổng đang active | Prototype giữ cổng đã nghỉ; companion tái dùng public listener/route pool, giữ riêng proxy local theo session | Real QUIC control bốn vòng vượt pool hai route, old/current và component restart đã đo; native stopped-DB reload/same listener/old ciphertext/current positive đã đo. M8 còn in-place crash/rollback/session budget và chạy dài; không suy resume từ fresh login. |
| Mọi loại UDP, PMTU và kích thước | Relay forward nguyên byte tối đa 255; không có authority gameplay | Chỉ đo type hoặc kích thước đang chặn G1/M7; không suy movement helper bao phủ toàn game. |
| Relay bốn/tám người và nhiều phòng | Thuật toán n×(n−1), pool route hữu hạn; managed native4/8 và sparse/departure đã có | Không lặp native4/8/sparse/departure hoặc negative controls đã đạt. M8 đo 50 người/8 phòng; số socket lý thuyết không là benchmark. |
| Companion xác thực trên Internet thật và gói đa ngôn ngữ | [CP373](evidence/checkpoints/records/CP373.json) đo hai ván Factory07 liên tiếp trong cùng phòng giữa Windows vi và Wine ko. Bản sửa gửi 110 theo thứ tự slot đã thấy ở đường cũ; lượt đạt cũng dùng di chuột và giữ nút Ready/Start. | Chưa tách được ảnh hưởng của thứ tự 110 với thao tác nút, chưa kiểm lại Windows hai actor chung luồng của CP354. Còn nghiệm thu gói trên Internet thật. |
| [Native59 / issue403](evidence/rust-service/g1-native59-observation.json) | Caller/buffer/schema đã có trace; [policy quan sát có giới hạn](protocol/room-membership.md#rust-bounded-diagnostic59) có native actual59 rồi hai ván/reset. Purpose101/units/server gốc vẫn chưa rõ | Không lặp parser/schema hoặc suy policy là original contract. Chỉ mở variant khi actual input chưa hỗ trợ; M8 đo budget16/visit dưới phiên dài/tải. Phép dừng âm tính và các lượt lỗi trước giữ nguyên. |
| Nhịp pump, DB lease checks và overload | Bounded recv/send, poll hiện hành; không gia hạn idle | Đo latency/queue/DB budgets và fairness dưới workload đích, kể cả nonreader và DB lỗi, trước M8. |

Không thêm Internet thật thành dependency ngầm của K6c/K7 đã nghiệm thu WAN giả lập hoặc yêu cầu Windows/hai máy vật lý.

| Câu hỏi còn mở | Backend policy được chọn | Phép RE/đo tối thiểu khi milestone cần |
|---|---|---|
| RUST-Q1 account/profile/recovery | Giữ account→PlayerId, token một lần, expiry/revoke/worker budgets hiện hành | M5b đã đo zero-profile DB và native DB/queue fault. Nhánh profile khác, graceful retry và same-process resume/match restore vẫn chưa có; persistence nhân vật theo M6 |
| RUST-Q2 phòng/đội/nhân vật | Dùng RoomKey/slot/membership generations và quota; source registry đã có | M7: tạo/vào/rời/phòng đầy, chọn đội/nhân vật trên KR thật; không RE lại consumer đã có |
| RUST-Q3 inventory/consumables | G1 giữ world item/kim policy; inventory bền vững thuộc giai đoạn sau | Consumer request/equip/use/count/reset/relogin khi product mở inventory; kim145 không chứng minh inventory auth |
| RUST-Q4 result/reward persistence | M6 thu hẹp đã kiểm journal/native result/relogin, thưởng 0 và idempotency | Vận hành/tải/restore thuộc M8; XP/currency projection chỉ đo khi mở scope tương ứng |
| RUST-Q5 authority/freshness | Giữ fuse/drop/report guards đã đo; raw ACK không có epoch đo được | Native stale-old/current-positive cho cơ chế đang đổi; low8 wrap/restart/portreuse, không suy queue clear ngăn mọi replay |
| RUST-Q6 vận hành | Một instance, bounded queues, admin/log/health, backup/restore | M8: workload 50 người/8 phòng, nonreader/fault/restart/shutdown và restore đã thử; giữ raw nếu tái hiện I/O lỗi |
| RUST-Q7 product policy | Sáu map ladder/item tương ứng, Free1v7 là giới hạn, lưu kết quả/thưởng0 đã chốt | Không còn chờ quyết định reward cho M6; policy XP/economy/mode ngoài scope chỉ mở khi owner quyết định |

Design nội bộ (hashing/store/transaction/scheduler/limits/adminAPI) không là khẳng định KR gốc. Wire/IDs/offsets,
authhandoff/actionconditions/effect/reset phải có build/locator/nativeproof trước expose. Không cần đợi fullRE
cho backend schema/tests hoặc port codec. N1/N1b đã có kết luận; các topology còn thiếu không là dependency K6c/K7.
Đợt planning-only đã giao PR286/CP213; [owner09/11](evidence/owner-directives/rust-implementation.json) đã mở implementation Rust. M0→M1 không đợi fullauth/generalroom; chỉ mở RE khi implementation/native acceptance bị chặn. Thứ tự hiện hành chỉ ở [PLAN](PLAN.md#current-order).

<a id="feature-gaps"></a>
## Khoảng trống theo cơ chế

Đây là câu hỏi/phép đo tối thiểu, **không là thứ tự công việc**; thứ tự duy nhất ở [PLAN](PLAN.md#current-order).
Phạm vi đã biết/giới hạn nằm ở contract được catalog dẫn; không đọc `unknown` thành vắng mặt.
Bảng được [coverage](COVERAGE.md) trích tự động để tra theo chức năng, không sửa bản sinh.

| ID | Khoảng trống hiện hành | Phép kiểm tối thiểu |
|---|---|---|
| <a id="gap-process-patch"></a>KR-B001 | [CP179](PROTOCOL.md#bootstrap-recipe-scope): K1 đạt riêng offlineR2 từ3coldstart/API đúngPID/contract-controls; K0CP178 giữ. Không originalWindowsbootstrap hoặc3e9coldstart. | Cold3R2 đã giải, không đo lại. WiderWCP/launcher/config còn ở #2; currente9input/state bridge và livecodec theo PLAN/#3, chỉ mở bootstrapcause nếu có trigger mới. [L1 UTF-8](RESEARCH.md#gap-client-utf8): Windows vi và Wine ko đã đọc tên, tiêu đề và chat chéo trong phòng; còn xác minh ván đầu, chat sảnh và bản en, không suy từ gate K0/K1. |
| <a id="gap-wcp-config"></a>KR-B002 | Fixture tổng hợp đã qua guard; oneclickuser/+E9C và dữ liệu/cache thật chưa đủ. | Chỉ mở consumer/config đang chặn runtime; so nhánh với và không tham số trên cùng copy, không đo lại +E78 đã giải. |
| <a id="gap-socket-codec"></a>KR-B003 | [CP181](PROTOCOL.md#login-retry-codec) actualsameprocessretry/reset/authcursor + CP180stream/CRC + CP057boundednativeoracle đủ K2profile theo PLAN. Fullgrammar/prefixguardon/modes/headerauth/capenforcement/FFFFFFFF giữ unknown. | Codec backlog theo trigger K3; không lặp solvedretry/closeonly/HWwatchpoint/76modelsplit. Accountresume thuộc identity/lifecycle, không tự thêm gate. |
| <a id="gap-auth-identity"></a>KR-L001 | [CP073](PROTOCOL.md#launcher-error-attribution): auxiliarylocale1,0 valid/init0→2000:0/noTCP; main currentA/Bactual67 positive. HistoricalmainSIGABRT/HTTPstartupcause vẫn unknown, không gánpopup là root. [CP217](PROTOCOL.md#auth-admission-prototype) valid/invalid32token cóproof; [CP223](PROTOCOL.md#auth-session-lifecycle) thêm endpoint-accountremap; [CP225](PROTOCOL.md#auth-fresh-process) freshprocesspositive/badtokennegative đã đo, sameprocessfatal/healthyEOFbefore17/fullidentityauthority vẫn mở. | Theo[PLAN](PLAN.md#current-order) tiếp RustM2haiidentity/phòng với bindings rõ; không mở lại authRE đã blocked hoặc tìm lại auth137/135/string/cache. Trace mainfail chỉ khi thực sự chặn. Providerinit sâu chỉ mở khi thực sự chặn main, chưa phải trọng tâm chơi. |
| <a id="gap-redirect-lobby"></a>KR-L002 | [CP186 K3local](PLAN.md#k3-acceptance) đạt; A291/nameguard/coldroute/versionnegative có nguồn. CP187actualAB291/currentroomintegration đủ K4scope; [CP200](PROTOCOL.md#four-session-native) bổ sung C/D291/name/ownslot. Caption/multichannel giữunknown. | [PLAN](PLAN.md#current-order): CP202–204 đã2v2outcomes/reset/bundle/bounds; tiếp lifecycle theoPLAN, không lặp ABCD291/room/start/NATtuple. Multichannel chỉ theo trigger. |
| <a id="gap-profile-cache"></a>KR-L003 | [CP186](PROTOCOL.md#channel-cold-route) cold3cache/warm có nguồn, K3local đạt. Năm retainedSSI giống clientnguồn; stageCRC0, mọicache/corrupt/reconnect/profile rộng cònunknown. | Reuse [scope/CRC](PROTOCOL.md#dynamic-cache-representation) và [coldroute](PROTOCOL.md#channel-cold-route); chỉ mở cachevariant khác khi K4/integration có trigger. Không lặp3file/CRC hoặc gọi coldroute là toànasset. |
| <a id="gap-create-join"></a>KR-R001 | [M7a](protocol/room-membership.md#rust-managed-room-selection): bốn account/client KR, hai phòng, tuple/list/title mặc định, slot, chủ, chuyển phòng và delete90 đã đo. | Occupancy đã đo ở [contract](protocol/room-membership.md#rust-room-population); [lifecycle managed 2/4/8](protocol/room-membership.md#rust-managed-team-rosters) đã đo trong tập cấu hình. [M7c room-only](protocol/room-membership.md#rust-managed-player-choices) đã đo lựa chọn/full-error; M7d ownership tích hợp đã đạt trong tập profile hiện hành. Sửa title trong UI bị profile hiện tại chặn; không mở entitlement RE khi chưa cần. Restart/durable tuple, password/mode chưa có; profile DB zero/ASCII đã có M5b, nhánh profile khác giữ riêng. |
| <a id="gap-roster-ready-host"></a>KR-R002 | [CP163](PROTOCOL.md#battle-eight-roster) actual selector1/8synthetic slots:4–4 check0/104,1–7 check7/no104, Ready/parser/map/capacity/getter có nguồn. CP162 actual Free/Manner/ACK/reset đã nối. | Không lặp guard/roster/map/input RE. CP165 actualpreset2cycles/Start3 đãđo; CP166 currentmodepromotion/rejoin đãtới clock; CP167đãhai cycleBhost; CP168BStart3/AUD050đãđo; CP171/172basiccycles/Start3 đã đo; tiếp PLAN theo gap thật; [CP208](PROTOCOL.md#eight-session-preview) có native8consumer trên2actual và adapter32TCP; tám actualclock/mesh56/stat8/ownerbom đã kiểm CP208209; CP210 có report-life/rank/reset/two4v4cycles8; [CP213](PROTOCOL.md#eight-team-variants) actualFree1–7/Manner4–4twooutcomes đã có; native8Manner1–7negative copy4b partialdoemptyJSONstartup; alternatecopy8d đãreturn7/no104/1543checks, emptyguardkhônghit nên khôngcause/fixproof. Còn interactiveC2Steamchange/generalroom-full/admission/Random/generalpromotion/row91 metadata. Theo PLAN; không coi sáu actor tổng hợp là sáu client. |
| <a id="gap-map-spawn-loading"></a>KR-R003 | Loading/barrier của profile cũ đã đo. [Village10](protocol/load-phase.md#village10-map-selection): mã 1104 lookup đúng, tên profile cũ làm Start từ chối trước loading. | Adapter và Factory07 đã có native theo [contract](protocol/load-phase.md#rust-map-profile-adapter). Tiếp UI map selection và từng map/terrain/item theo [phép đo](RESEARCH.md#gap-village10-map); Village10 đã có native UI/basic; terrain/item/higher roster còn riêng. Không lặp ID-only. |
| <a id="gap-room-membership"></a>KR-R004 | [M7a](protocol/room-membership.md#rust-managed-room-selection) đã đo nhiều phòng, cross-room rejoin, EOF/revoke và slot độc lập identity; [CP230](PROTOCOL.md#room-shared-cleanup): guest86/EOF→A107ID2→0/base6-rewind0-UI1 vàhost86→B107ID1→0/110full6/UIchief; private registry/native2actual/565positive+166failure raw. Hostpostgetter unobservedbudget20, otherroomC exclusion9policycases only. | [PLAN](PLAN.md#current-order): reusable policy/reference and typed integration seam, recipient-specific sendfailure/partialsend/owner-visit fences beforeproduction. Native only if newimplementation/discrepancy; không lặp solved107/110/UI hoặc fullmembership/accountresume claim |
| <a id="gap-round-clock"></a>KR-F001 | CP155 baepoch actual99/nativeclock/barrier trước30s, B32390restore mỗi clock; same-run freshness/contest/EOF cónguồn. | Không RE lại99/load/clock; tiếp thứ tự hiện hành trong PLAN. Originalclock/timeoutauthority ngoài profile giữunknown. |
| <a id="gap-peer-movement"></a>KR-F002 | [CP176](PROTOCOL.md#a-c-basic-cycles) current A/C4nativewindows/2keys/target/currentprogress cùng2normalcycles; CP173 giữ AB scope. | FullUDP/auth/reliability/generalphysics/NAT discovery còn unknown; [CP194–195](PROTOCOL.md#udp-netem-cycles) đã đo current AB trên NAT tĩnh/UDP impairment. [CP200](PROTOCOL.md#four-session-native) thêm4actualcurrentmovement12chiều; [CP209](PROTOCOL.md#eight-mesh-owner-bombs) thêm8actual/56remoteprogress, gameplay4 theo PLAN, không lặp consumer/AC peer/solvedcodec/K6b đã đạt. |
| <a id="gap-udp-freshness"></a>KR-F003 | [CP155](PROTOCOL.md#direct-oldudp-contest-eof) exactR1Bframe→R2actualendpoint/queue/key59≠56/drop/noapply+11currentpositive vàfreshBRight trong cùng2contest/Ready/EOFrecipe. | Scope1key/currentIDs đã giải, không RE lại CP145consumer. low8wrap/forgedcurrentkey/fullgrammar/auth cònunknown theo trigger; tiếp thứ tự hiện hành trong PLAN. |
| <a id="gap-bomb-lifetime"></a>KR-F004 | [CP174](PROTOCOL.md#basic-host-b-eof) basicB/A/A IDs1..6/native195/reset rồithirdBID7pending/ownedBEOF/round_closed/no158; CP155scope cũ giữ. | [CP201](PROTOCOL.md#four-owner-bombs) có4owner/currentcapacity/159158broadcast/fencecontrols; [CP202](PROTOCOL.md#four-two-team-cycles) đã có2v2outcome/reset; [CP203](PROTOCOL.md#four-portable-guards) đã guarddeadowner118; transport/hardening tiếp theoPLAN. Genericpool/auth/races/low8wrap cònunknown, khônglặpknownfence. |
| <a id="gap-blast-block-chain"></a>KR-F005 | [CP172](PROTOCOL.md#hidden-cell-admission-current) basic Box/reveal27/28/142guard/3effects/reset/Start3 đã kiểm. | Generalmoving/order/allmaps/fullworldauthority unknown; không RE lại consumer. CP173 Bowner/native195 đã ghép; CP174BhostEOF đã kiểm; admission theoPLAN. |
| <a id="gap-floor-pickup"></a>KR-F006 | [CP172](PROTOCOL.md#hidden-cell-admission-current) terrain currentclearedcell/generation/commit/roster guard và native basic2cycles/reset/Start3 có nguồn; no-terrain giữ weak report scope. | Full142key/auth/distance/freshness/fairness/two-nativecontenders unknown. Same old142 sau legitimatecurrentclear chưa phân biệt; không gọi policy guard là original authority. CP173B118/bidirectional đã kiểm; CP174BhostEOF đã kiểm; admission theoPLAN. |
| <a id="gap-item-stats"></a>KR-F007 | [CP087](PROTOCOL.md#capacity-reservations) profileAinitial1max10/B2max7, acceptedBomb tăngtracked/native1→2 và resetthật hai vòng. Runtimeatmax/character/mode/pxs/TTL/fullreset khôngđủ. | [CP201](PROTOCOL.md#four-owner-bombs) đã đo C/Dstat/maxima trêncurrent4clock, không ápABdefaultquaD. Servertrackingdesign/nativepoollaw/runtimeatmax/othercharacters vẫnunknown; mởtheotrigger. |
| <a id="gap-needle"></a>KR-F008 | [Rust account/roster](protocol/life-results.md#rust-needle-rescue) đã có native kim/cạn/reset; giới hạn tại [gap Rust](RESEARCH.md#gap-rust-life-actions). CP113opt-inonecredit/pair/peerescape; [CP114](PROTOCOL.md#needle-round-reset) currentR1secondtrapcạn/no1,0/actual19→resultreturn/Start2fresh145credit/escape. R2active, không2results. | Multi-charge/inventoryauth/generalresetvariants cònmởtheotrigger; khônglặpcurrentonecredit/depleted/reset. Rescue18độc lậpởcontactgap/PLAN. |
| <a id="gap-contact-state"></a>KR-F009 | [CP158](PROTOCOL.md#current-trap-udp-counter): native smallgap4 drop/largegap17 accept/controlledremote56→0→56→1; two observational runs không tái hiện CP157. | Exact naturalCP157state0packet/apply vẫn candidate/unknown; controlledcapability không retrospectivecause. Không lặp counter/body RE; nếu trigger cần, đo actual non-replayed packet/queue trướctrap. Hiện theo PLAN production baseline, không tự sửa client/claimbugfixed. |
| <a id="gap-ability-debuff"></a>KR-F010 | [MaxPower](protocol/items-stats.md#rust-maxpower), [Punch](evidence/rust-service/g1-punch-matrix.json) và [Shoe](evidence/rust-service/g1-shoe-matrix.json) đã có native/controls theo tập G1 giữ lại; ma trận có bốn hướng, khoảng cách/corridor, peer/retirement/reset/account/journal. [Ma trận acceptance G1](evidence/rust-service/g1-acceptance-matrix.json) giữ kết luận theo phạm vi owner đã cắt. | Không mở lát riêng theo selector hoặc nhóm item đã tắt trong G1. Native duplicate/late/inflight/foreign/queued EOF chưa được suy từ guard tests; physics/authority đầy đủ, debuff/TTL và nhóm khác để giai đoạn sau. M8 theo PLAN; không dùng tail119 làm epoch hoặc đoán ACK160. |
| <a id="gap-mount"></a>KR-F011 | Group2 có nhánh riêng; điều kiện/effect/giới hạn/reset runtime KR chưa kiểm. | Chỉ khi fixture Battle cần mount: trace11928A2 tới consumer rồi một pickup/peer/reset; hiện loại khỏi fixture theo policy. |
| <a id="gap-active-other"></a>KR-F012 | Dart/Oxygen/Potion/Spring/Drill/RemoteBomb mới là phân loại, chưa chứng minh cùng đường kim. | Chọn một group đang cần, tìm consumer Ctrl/report/confirm; đo điều kiện, tiêu hao, hiệu ứng và peer, không quét từng row. |
| <a id="gap-timed-variants"></a>KR-F013 | Shield/Sensor/SJNeedle raw tail chưa chứng minh đơn vị/thời hạn/hành vi KR. | Trace consumer timer/group cần dùng, đo start/expiry/cancel/reset và peer với control; không suy ms từ tên hoặc raw số. |
| <a id="gap-mode-items"></a>KR-F014 | ItemAttr cho thấy row/group; presence cơ chế/mode guard/reward runtime vẫn candidate, ngoài Battle tối thiểu. | Chỉ mở group mode khi PLAN mở mode đó; bắt guard/consumer trước thử runtime, HPNeedle17 không là Needle145. |
| <a id="gap-peer-relay"></a>KR-F015 | Relay opt-in đã tích hợp và có native prototype hai người: hai chiều, reset, gói cũ tới cổng đã nghỉ và EOF. [Contract](protocol/udp-peer.md#rust-relay-endpoint-scope), [evidence](evidence/rust-service/n1r-fixed2.json). | Còn spoof giữa người chung IP, rebinding, port reuse/restart, mọi loại UDP và mesh bốn/tám người; [phép đo](RESEARCH.md#gap-relay-endpoint). K6c/K7 và N1 cũ không đổi; không retry N1b. |
| <a id="gap-result-return"></a>KR-X001 | <a id="gap-managed-all-dead"></a>[Managed player all-dead](protocol/life-results.md#rust-managed-all-dead) đã có typed outcome, rank 2 và journal AllDead/Draw; native contiguous 2/4/8 có hai vòng/reset và roster 4/8 có Start/clock kế. Điều kiện 4/8 dùng LAN cô lập với netem 1,5 giây và phím đồng thời; failed/partial giữ riêng. | Gap Armed vô hạn khi current members đều chết đã giải trong scope managed đã đo ở [#453](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/453). Còn native sparse/departed roster và delayed175/revoke/EOF fault injection; TCP/SQLite đã kiểm departure, DB commit và old-round riêng. Không suy tie enum hoặc mở fixed-team legacy. Tiếp coverage G1 theo PLAN, không lặp toàn bộ roster/account đã nghiệm thu. |
| <a id="gap-round-reset"></a>KR-X002 | [CP176](PROTOCOL.md#a-c-basic-cycles) AC2keys/6entities/3Aeffects/195/stat/hiddeneligibility reset và Ready2 có nguồn. | ABStart3 đã có [CP198](PROTOCOL.md#native-interleaved-startup), ACscopeCP176giữ riêng; [CP202–204](PROTOCOL.md#four-transport-bounds) đãfourreset/Start2/3; wholeinventory/modes/low8wrap/generalreset còn thiếu, không lặp knownAB/ACcycles. |
| <a id="gap-disconnect-replacement"></a>KR-X003 | [CP177](PROTOCOL.md#replacement-profile) optional exported preset/capacity theo vacancy và actual C0/C1 initial/118/159158 đã kiểm; CP176 giữ AC2normalcycles riêng. | Maximum/exhaustion/general membership/account resume chưa biết; C0 startup SIGSEGV giữ nguyên, đối chứng pass không là rootcause. Theo PLAN acceptance/control, không lặp known C0/C1 entry hoặc ACcycles. |
| <a id="gap-diagnostic-service"></a>KR-S001 | [CP190](PROTOCOL.md#portable-probe-bundle) gói service/47fixture relocation đã actualKRWine/nativeidentity/room/load/EOFcontrol;  [CP177](PROTOCOL.md#replacement-profile) optional replacement preset/47pins/derived capacity/284server và7export tests; native C0/C1 actual118159158 đã đo. | Theo PLAN đối chiếu acceptance/dependency/control trước phép đo còn thiếu; nofullauth/physicalLANInternet8client/gate. Startup unknown không được suy thành capacity regression/fix. |
| <a id="gap-service-hardening"></a>KR-S002 | LAN giả lập đạtCP191; [CP193–196](PROTOCOL.md#nat-netem-host-eof) có NAT tĩnh/UDP delay-loss/two cycles/EOF. Interleaved admission sai6/8 đã có CP190. | [CP202203](PROTOCOL.md#four-two-team-cycles) đã4normal2v2/reset/bundle, [CP204](PROTOCOL.md#four-transport-bounds) córecipient-safe/bounds/rate; [CP205](PROTOCOL.md#four-member-abort-room) bổ sung native3survivorEOFcleanup; [CP206](PROTOCOL.md#four-health-identity) thêm health/identity/processrestart; [CP207](PROTOCOL.md#four-netem-restart) đã nativefreshrestart/fourUDPimpairment; [CP210211](PROTOCOL.md#eight-portable-battle) có8actualtwo4v4cycles/relocatedbundle; [K7 đạt scoped CP212](PLAN.md#k7-evidence-review); N1 đã đạt phạm vi riêng và N1b âm, không thêm dependency K6c/K7. Nền gói Rust đã có [manifest/state và smoke x86/ARM tách repo](evidence/rust-service/m8-server-package.json); không chứng nhận native hoặc Internet. Đã kiểm bước xuất pack và cài gói Linux trong namespace sạch bằng hai client TLS tổng hợp, backup và dừng; [bằng chứng](evidence/rust-service/m8-pack-export.json). Còn nghiệm thu gói người chơi Wine/Windows, tải và vận hành theo PLAN. Theo PLAN, không tái mở NEXT-K7-FOUR-BATTLE-OUTCOME đã giải. Binding incoming là design riêng, không auth gốc. |

<a id="diễn-biến-lịch-sử-theo-checkpoint"></a>
[Lịch sử theo checkpoint đã chuyển](HISTORY.md#diễn-biến-lịch-sử-theo-checkpoint).
