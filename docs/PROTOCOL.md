# KR 00967 — contract tương thích client gốc

Contract và giới hạn tra theo chủ đề dưới đây; các mục có CP ID giữ nguyên phạm vi tại thời điểm đo.
Trạng thái/nghiệm thu hiện hành ở [PLAN](PLAN.md#dashboard); lịch sử ở [HISTORY](HISTORY.md),
câu hỏi mở ở [RESEARCH](RESEARCH.md). Không suy gate hoặc full game từ một mục contract riêng.

## 1. Định danh và nguồn chuẩn

Đọc/đo mới trên các target sau; đường dẫn cũ trong notebook tra bằng `/home/admin/boom/README.md`.

| Artifact | Đường dẫn hiện tại | SHA-256 đo 26/10 |
|---|---|---|
| KR gốc | `/home/admin/boom/clients/kr-00967/CA.exe` | `a52545c14b1fea423de655c3163d6360b628d61c1cd472b46fdbc8a9ffe15f4e` |
| KR patcher | `/home/admin/boom/clients/kr-00967/NewPatcher.exe` | `b59e4dce9ad23c439a7632ce04ecb2eed3481585bd2454100b05c9d7527f5c02` |
| VN gốc 116 | `/home/admin/boom/clients/vn-116/ca.exe` | `bce8644d4e783d1354ab4fc880e1ab9a4ab3325327028f37d99e5439e25f4bbd` |
| VN đã dùng RE/runtime | `/home/admin/boom/repos/boom-re/wine/Boom/ca_fixed_patched.exe` | `bc875ba5765cc112e924f46e75d0ace112904c7801126e46bc0abb1c9cb576b1` |

VN gốc và VN patched là **hai artifact khác nhau**; phương pháp/claim runtime có patch provenance.
Một file `.i64` có cùng tên chưa chứng minh input, image layout và các address khớp. Dump memory
phải ghi base/region; VA trong dump không tự chuyển thành offset file packed gốc.

Nguồn đọc ở lượt lập contract: `boom-re/docs/{protocol,oc_d-login,lobby,oc_e-game}.md`,
`boom-re/tools/{boomproto,fakeserver,verify_emu}.py`, `boom-re/server/src/{proto,session,packets,hub}.rs`;
`boomkr-re/docs/kr.md` §1, §7–§33 và `vn-vs-kr.md` §4. Đây là nguồn tham chiếu local,
không dùng các phần trăm ước lượng reuse/rewrite của notebook làm bằng chứng tương thích.
Kiến thức được nhập vào docs hiện hành:
[VN server reference](REFERENCES.md),
[VN battle runtime](REFERENCES.md),
[VN protocol](REFERENCES.md),
[KR flame/chain](REFERENCES.md),
[KR trap timer](REFERENCES.md),
[KR ghost/position](REFERENCES.md).

## 2. Ma trận tái sử dụng VN → KR

Mức `confirmed` dưới đây chỉ áp dụng nguồn/build/phạm vi ghi trong hàng, không tự xác nhận KR wire.

| Lớp | Điều đã có | Cách áp dụng / phần KR còn thiếu |
|---|---|---|
| Debug/input/IAT | VN có recipe GDB proxy, input dispatch/gate trace và đối chiếu IAT theo ABI | CP012 có command login; CP016 có callback/dialog/request Tạo phòng theo [contract](protocol/room-membership.md#room-create). Recipe, provenance và giới hạn tại [evidence reuse](evidence/vn-reuse/README.md). Không copy địa chỉ hoặc tự đổi GetAsyncKeyState. |
| Pack/map/sprite/ODT | Reader dùng được cho pack KR; inventory 13 pack/51.467 entry; có khác biệt header map và numeric ID | Dùng lại tool và dữ liệu KR. Điều này không xác nhận version, frame hoặc opcode mạng. |
| Framing/primitive | VN `CClientSocket` `A68850`, recv/process `A662E0`, send `A663E0`; encode/decode/CRC đã có kiểm binary | Dùng làm mẫu tìm class/call graph và mẫu fixture. Patch bootstrap có profile riêng [đã kiểm](evidence/bootstrap-wire/README.md); KR có [profile WCP hẹp](protocol/bootstrap-codec.md#wcp) và native capture/field checks tới104/121 theo [contract start](protocol/load-phase.md#start-reply); full codec/reset/bounds còn mở. |
| Crypto/seq | VN XOR `0x53351D9C`/step `0x63CAACE3`, CRC8/32, base/seq/delta, opcode substitution | KR profile mode2/selector3 đã có oracle native và reset tại CP006/057/181; xem [codec](protocol/auth-session.md#login-retry-codec). VN chỉ là mỏ neo, không chứng nhận mode/header khác. |
| Hello/version | VN server-first hello, rồi đổi mode; version 116 và packet bootstrap có thứ tự | Patch socket KR: mode0/selector0, BE16 version967, response local được chấp nhận theo [evidence](evidence/bootstrap-wire/README.md). Consumer WCP, initial0/0 và transition2/3 có bằng chứng theo [contract WCP](protocol/bootstrap-codec.md#wcp); profile codec/hello/reset được nghiệm thu CP181; widercodec vẫn giới hạn, không gộp socket. |
| Login/identity | VN có pipeline login-result/network-info và variant China khác layout | CP058 có shape auth; CP183/184 đã nối identity291 tới panel và integration A. [Contract](protocol/lobby-profile.md#own-profile-integration); fullaccountauthority/cachematrix/vanilla vẫn giới hạn. |
| Cache/redirect/channel | VN cần tag blob; dữ liệu inner LE khác outer wire, cache path khởi tạo theo stage | Kiểm contract KR từ reader/consumer. Giữ test cold/warm cache; không copy blob VN hay replay flag “dùng cache” khi file không tồn tại. |
| Lobby/room/start | VN có request/reply, slot flags, map tự gửi lúc vào phòng, loading/clock nhiều pha | Dùng để xây checklist ca biên. KR có stage11 và17/241 theo [contract sảnh](protocol/lobby-profile.md#channel-lobby); [request81/response86](protocol/room-membership.md#room-create) đã đưa copy tới CPrepareStage và UI phòng chờ; [player/ownslot/controls](protocol/room-membership.md#room-player) đã đo trên copy, [map metadata/start104](protocol/load-phase.md#room-map) và [start121/scene nhân vật](protocol/load-phase.md#start-reply), [clock/input](protocol/load-phase.md#round-go), [nổ/HELP/về phòng](protocol/bombs.md#bomb-cycle) đã đo; bảng kết quả chọn đã có [CP023](protocol/life-results.md#round-result), hai client movement/bom đã đo ở [CP027](protocol/udp-peer.md#peer-duo), join/Ready và chu kỳ pair ở [CP028](protocol/room-membership.md#duo-room-round); authority/reset đầy đủ còn thiếu. |
| Bomb/item/trap/result | VN `111→152→151`; `553`, pickup `117→158`; `120` subtype; kết quả `168→173` có runtime nguồn cũ | Chỉ mỏ neo VN. KR phải nối send/receive tới object/state tương ứng trước khi implement. Fuse/drop/cap policy emulator không phải luật server gốc. |
| Peer/movement | VN C→S92 trao IP/UDP port; S→C146 có peer table; runtime hai máy di chuyển đồng bộ qua UDP | Không giả định KR dùng cùng packet/MTUDP version hoặc không đổi peer contract. TCP-only demo không hoàn tất multiplayer khi peer chưa đồng bộ. |
| Kiến trúc/harness | VN có hub/session/round generation, golden Python↔Rust, control 1–2 client | Dùng pattern và bài học, không coi golden hai implementation cùng logic là oracle KR; không copy legacy server vào crates của rewrite. |
| Luật trận chung | Nhiều claim VN confirmed dùng được theo ADR 0002/0013 | Dùng để thiết kế/kiểm hành vi. Luật tương đương không chứng minh binary layout, field widths và wire numbering tương đương. |

<a id="trusted-wiki-reuse"></a>
### Wiki KR được chủ dự án tin cậy: áp dụng cho luật, đối chiếu theo build

Nguồn [REFERENCES](REFERENCES.md#owner-trusted-namu-wiki), [hash/locator](evidence/namu-wiki/metadata.json).
Đây là **reference do chủ dự án chấp nhận**, không gán thành `kr_runtime`. Khi dùng trong emulator,
rule có thể là `design/emu_choice` dựa trên nguồn này; claim `kr_static/kr_runtime` vẫn cần nguồn build.

| Nhóm | Kiến thức dùng lại | Phần KR00967 chỉ cần kiểm còn thiếu |
|---|---|---|
| Battle cơ bản | Bomb +1 count; Fluid +1 range; Roller +1 speed; Ultra tới max range; Red Devil tới max speed **và đẩy bom** | ItemAttr/consumer/ID, đơn vị và cap theo char đang dùng; không RE lại ý nghĩa item |
| Trapped/cứu/kim | Đồng đội cứu, địch diệt; Ctrl dùng kim khi trapped, oxy kéo dài thời gian | Reuse các probeKR đã có; chỉ kiểm variant/ordering/consume/peer còn chặn demo; wiki không giải simultaneoustick |
| Đội/mode | Free cho lệch đội; Manner cân bằng mọi đội, ví dụ2–2/2–2–2/3–3/4–4 | ReuseCP162/163; selector/opcode/nativevalidation không copy từ tên hoặc wiki |
| Stat nhân vật | Có basic/maximum; Bazzi tham chiếu count1→6/range1→7/speed5→9 | Ánh xạ BomberAttr/build/charID và đơn vị; khác currentpolicy phải ghi khác biệt, không áp số mù |
| Map/mode phụ | Danh mục theme/mission/Revival và luật thắng/thua được mô tả | Chỉ chọn scope khi PLAN mở; geometry/spawn/assets/nativepresence/ID vẫn đối chiếuKR |

Nguồn PC đang dùng; BnB M/mobile để riêng. Trang sửa2026 có thể khác00967, bản dịch lore/char chỉ
một phần; giữ Hàn gốc và ngày sửa để xử lý mâu thuẫn. Giá trị luật tham chiếu giúp bỏ RE ngữ nghĩa
lặp lại, không tự đóng gate hoặc giải blocker K3 cache/version. Thứ tự tiếp duy nhất ở PLAN.

## 3. Mỏ neo KR có sẵn và giới hạn

| Locator | Nguồn / trạng thái | Có thể dùng để tìm | Không kết luận từ mỏ neo |
|---|---|---|---|
| `NewPatcher!sub_4118E0`, xref literal `files.nfo2` tại `0x411AF0`, `0x411D1D` | `kr_static` / `confirmed` cho consumer đọc tĩnh, [evidence](evidence/bootstrap/README.md) | Caller/config manifest/download và nhánh lỗi bootstrap | Không phải handler game; chưa nối tới quyết định thành công của `CA!ProcessPatch` |
| `0x00B7A760`, sender packet **167** | `kr_static` / `confirmed`, R-200…R-202 | Lần ngược packet builder/primitive writer từ CGOGameBomber | Không tự gán semantics/layout VN120 hoặc S→C tương ứng |
| `0x00E67E70→0x00E84460`, packet **184 C→S** | `kr_static` / `confirmed` trong phạm vi status serialize | Writer, context/slot và status report | Không gọi đây là S→C trap event hoặc coi client report là quyết định server |
| Packet **183 S→C**, `CGOGhost→B334C0` | `kr_static` / `confirmed`, R-138 | Reader/dispatch/context, setter đơn vị ô 40px | Chưa chứng minh đây là movement sync của bomber sống; C→S148/149 chỉ candidate |
| `0x00E9C520` / `0x00E9CDC0`, message **27** | `kr_static` / `confirmed` theo R-139…R-141 | Tính vùng nổ/chain và đường gọi từ network event | Message27 là nội bộ; wire158/kind0 được bổ sung riêng ở [CP022](protocol/bombs.md#bomb-cycle), chưa full chain/authority |
| `0x01196A70` | `kr_static`, xem [ItemAttr](REFERENCES.md) | Loader ItemAttr; pickup/layout đã có [contract KR riêng](protocol/items-stats.md#item-pickup) | Không suy weight thành %, hoặc mặc định drop placement do client chọn. `E9E660` là nhánh kind0 bom, không phải receiver pickup |

Các số opcode này là các mảnh đã có, không phải registry hoàn chỉnh. Tìm được writer từ 167/184
có thể mở được primitive codec trước K1; vẫn cần traffic thật để qua K2.

## 4. Record bắt buộc cho mỗi packet

Registry khóa theo `(build, direction, connection_role, stage, opcode)`; không gộp packet cùng số
khác chiều/state. Mỗi entry phải ghi:

- Artifact SHA, VA/RVA của caller/builder/handler/consumer, evidence ID và phạm vi static/runtime.
- Header/checksum/crypto mode; endian outer và nested blob; seq/key trước/sau packet.
- Field order, offset/cursor, width/count/optional guard/encoding và bounds; unknown giữ nguyên tên trung tính.
- State cần trước khi nhận, side effect, reply/ack/broadcast, redirect socket và thời hạn chờ.
- Mẫu raw→decoded→state có nguồn độc lập; hash capture, run config/patch/cache, trailing bytes và giới hạn.
- Nhãn `kr_static`/`kr_runtime`/`vn_static`/`vn_runtime`/`emu_choice`/`design` và trạng thái
  `confirmed`/`candidate`/`unknown`/`design`. Chưa biết giá trị thì không điền default như fact.

Golden codec cần oracle hàm KR thật hoặc capture, bao phủ size qua ranh giới dword, chuỗi cp949/Unicode
theo consumer, blob raw/nén, checksum sai, seq rollover/reconnect, partial/coalesced frame. Parser server
phải từ chối length/count vượt giới hạn trước allocation. Không fuzz client gốc bằng packet chưa có bounds.

## Tra theo chủ đề

| Chủ đề | Nguồn contract | Packet/opcode chính |
|---|---|---|
| Bootstrap, WCP và codec | [bootstrap-codec](protocol/bootstrap-codec.md) | bootstrap, hello, framing/CRC/sequence |
| Đăng nhập, token, account và lease | [auth-session](protocol/auth-session.md), [hồ sơ DB](protocol/auth-session.md#rust-db-account-profiles) | 1/2/6/17, frontend32, session, zero-profile |
| Sảnh, identity, cache và profile | [lobby-profile](protocol/lobby-profile.md) | 17/241/291/528 |
| Phòng, roster và membership | [room-membership](protocol/room-membership.md) | 81/82/86/87/90/91/106/107/110 |
| Map, Ready, loading và phase | [load-phase](protocol/load-phase.md) | 95/99/104/117/121/196/197/585 |
| UDP, peer, movement và stale fences | [udp-peer](protocol/udp-peer.md), [companion thử nghiệm](protocol/udp-peer.md#rust-authenticated-companion-foundation) | 99/153, UDP movement, 127/170 |
| Bom, serial, nổ và capacity | [bombs](protocol/bombs.md) | 118/159/158, chain/timer |
| Terrain, block và cascade | [terrain](protocol/terrain.md) | 142/195, cell/box flags |
| Item, stat, pickup và kim | [items-stats](protocol/items-stats.md) | 124/145/165/582 |
| Trap, death, cứu, kết quả và reset | [life-results](protocol/life-results.md) | 127/170/175/180/202 |
| Bốn/tám người, đội và admission | [multiplayer-team](protocol/multiplayer-team.md) | 106/110/121/153/175/180 |
| LAN, WAN và Internet thật | [networks](protocol/networks.md) | endpoint/routing/NAT/netem/EOF |
| Authority, phạm vi service và ranh giới Rust | [rust-boundary](protocol/rust-boundary.md) | M1/M2/M3, R1, package/guards |

## Anchor tương thích cho liên kết lịch sử

<!-- protocol-compatibility-stubs -->
<a id="rust-port-boundary"></a><a id="ranh-giới-contract-khi-chuyển-rust--design-cp213"></a> [→ Authority, phạm vi service và ranh giới Rust](protocol/rust-boundary.md#rust-port-boundary)
<a id="eight-team-variants"></a><a id="tám-actualclient-free-17-và-manner-44-qua-hai-kết-quảreset--cp213"></a> [→ Bốn/tám người, đội và admission](protocol/multiplayer-team.md#eight-team-variants)
<a id="emulated-wan-acceptance"></a><a id="k6c-wan-giả-lập--phạm-vi-mạng-được-nghiệm-thu-tại-cp212"></a> [→ LAN, WAN và Internet thật](protocol/networks.md#emulated-wan-acceptance)
<a id="helper-k2a-biệt-lập-lịch-sử-oraclestream-phục-hồi-sau-đó"></a> [→ Bootstrap, WCP và codec](protocol/bootstrap-codec.md#helper-k2a-biệt-lập-lịch-sử-oraclestream-phục-hồi-sau-đó)
<a id="bootstrap-recipe-scope"></a><a id="phạm-vi-recipe-bootstrap-đã-đối-chiếu--cp179"></a> [→ Bootstrap, WCP và codec](protocol/bootstrap-codec.md#bootstrap-recipe-scope)
<a id="wcp"></a><a id="wcp-trên-kết-nối-sau-processpatch"></a> [→ Bootstrap, WCP và codec](protocol/bootstrap-codec.md#wcp)
<a id="login-auth"></a><a id="login-auth-trên-copy--cp014"></a> [→ Đăng nhập, token, account và lease](protocol/auth-session.md#login-auth)
<a id="auth-token-request"></a><a id="token-local-trong-cs1--cp214"></a> [→ Đăng nhập, token, account và lease](protocol/auth-session.md#auth-token-request)
<a id="auth-token-bounds"></a><a id="ranh-giới-chuỗi-token-và-token2-only--cp215"></a> [→ Đăng nhập, token, account và lease](protocol/auth-session.md#auth-token-bounds)
<a id="auth-channel-echo"></a><a id="producer-cs6-và-echo-từ-reply-đăng-nhập--cp216"></a> [→ Đăng nhập, token, account và lease](protocol/auth-session.md#auth-channel-echo)
<a id="auth-admission-prototype"></a><a id="adapter-admission-cục-bộ--cp217"></a> [→ Đăng nhập, token, account và lease](protocol/auth-session.md#auth-admission-prototype)
<a id="auth-lease-eof-blocker"></a><a id="lease-expiredrevoked-từ-chối-được-lifecycle-client-lỗi--cp218"></a> [→ Đăng nhập, token, account và lease](protocol/auth-session.md#auth-lease-eof-blocker)
<a id="auth-eof-dispatch"></a><a id="eof-trước-channel17-control-và-nguồn-target-gọi-lỗi--cp219"></a> [→ Đăng nhập, token, account và lease](protocol/auth-session.md#auth-eof-dispatch)
<a id="auth-tcp-ownership"></a><a id="ownership-qua-kernel-tcp-receiver-eof-còn-blocked--cp220"></a> [→ Đăng nhập, token, account và lease](protocol/auth-session.md#auth-tcp-ownership)
<a id="auth-two-account"></a><a id="hai-identity-qua-native-riêng-wrong-binding-tcp-riêng--cp221"></a> [→ Đăng nhập, token, account và lease](protocol/auth-session.md#auth-two-account)
<a id="auth-two-actual"></a><a id="hai-kr-cùng-service-identitysession-riêng-và-duplicate-account--cp222"></a> [→ Đăng nhập, token, account và lease](protocol/auth-session.md#auth-two-actual)
<a id="auth-session-lifecycle"></a><a id="account-đổi-endpoint-lỗi-sau17-và-ranh-giới-retry--cp223"></a> [→ Đăng nhập, token, account và lease](protocol/auth-session.md#auth-session-lifecycle)
<a id="auth-retry-validation"></a><a id="retry-qua-hellovalidation-nhưng-fatal-trước-token-builder--cp224"></a> [→ Đăng nhập, token, account và lease](protocol/auth-session.md#auth-retry-validation)
<a id="auth-fresh-process"></a><a id="fresh-login-bằng-process-mới-sau-session-cũ--cp225"></a> [→ Đăng nhập, token, account và lease](protocol/auth-session.md#auth-fresh-process)
<a id="channel-lobby"></a><a id="từ-auth-tới-ui-sảnh-trên-copy--cp015"></a> [→ Sảnh, identity, cache và profile](protocol/lobby-profile.md#channel-lobby)
<a id="room-create"></a><a id="tạo-phòng-input-danh-sách-kênh-và-request--cp016"></a> [→ Phòng, roster và membership](protocol/room-membership.md#room-create)
<a id="room-list-lifecycle"></a><a id="nhiều-row-cập-nhật91-và-xóa90--cp226"></a> [→ Phòng, roster và membership](protocol/room-membership.md#room-list-lifecycle)
<a id="room-join-projection"></a><a id="chọn-row--reply87--context-phòng-generic-join-error--cp227"></a> [→ Phòng, roster và membership](protocol/room-membership.md#room-join-projection)
<a id="room-local-roster"></a><a id="roster-riêng-của-hai-room-context--cp228"></a> [→ Phòng, roster và membership](protocol/room-membership.md#room-local-roster)
<a id="room-membership-registry"></a><a id="registry-private-backrejoin-và-chuyển-phòng--cp229"></a> [→ Phòng, roster và membership](protocol/room-membership.md#room-membership-registry)
<a id="room-shared-cleanup"></a><a id="cleanup-membership-trong-registry-private--cp230"></a> [→ Phòng, roster và membership](protocol/room-membership.md#room-shared-cleanup)
<a id="room-player"></a><a id="player-own-slot-và-controls-phòng-cp018"></a> [→ Phòng, roster và membership](protocol/room-membership.md#room-player)
<a id="room-map"></a><a id="map-metadata-filter-và-request-start-cp019"></a> [→ Map, Ready, loading và phase](protocol/load-phase.md#room-map)
<a id="battle-eight-roster"></a><a id="capacity-battle-8-slot-và-điều-kiện-chia-đội--cp147163"></a> [→ Bốn/tám người, đội và admission](protocol/multiplayer-team.md#battle-eight-roster)
<a id="start-reply"></a><a id="phản-hồi-start-và-init-cgamestage--cp020"></a> [→ Map, Ready, loading và phase](protocol/load-phase.md#start-reply)
<a id="round-go"></a><a id="phase-clock-map-và-input--cp021"></a> [→ Map, Ready, loading và phase](protocol/load-phase.md#round-go)
<a id="bomb-cycle"></a><a id="bom-nổ-help-và-trở-về-phòng--cp022"></a> [→ Bom, serial, nổ và capacity](protocol/bombs.md#bomb-cycle)
<a id="round-result"></a><a id="event-trapdeath-và-bảng-kết-quả--cp023"></a> [→ Trap, death, cứu, kết quả và reset](protocol/life-results.md#round-result)
<a id="bomb-position"></a><a id="echo-bom-nguồn-spawn-và-map-cụ-thể--cp024"></a> [→ Bom, serial, nổ và capacity](protocol/bombs.md#bomb-position)
<a id="bomb-serial"></a><a id="nhiều-bom-serial-và-id--cp025"></a> [→ Bom, serial, nổ và capacity](protocol/bombs.md#bomb-serial)
<a id="kr-peer-entry"></a><a id="request99-và-bảng-peer-kr153--cp025027"></a> [→ UDP, peer, movement và stale fences](protocol/udp-peer.md#kr-peer-entry)
<a id="peer-duo"></a><a id="hai-client-thật-movement-và-bom--cp027"></a> [→ UDP, peer, movement và stale fences](protocol/udp-peer.md#peer-duo)
<a id="duo-room-round"></a><a id="join-ready-và-chu-kỳ-hai-client--cp028"></a> [→ Phòng, roster và membership](protocol/room-membership.md#duo-room-round)
<a id="peer-events"></a><a id="đồng-bộ-trapdeath-qua-kr170--cp029"></a> [→ Trap, death, cứu, kết quả và reset](protocol/life-results.md#peer-events)
<a id="item-pickup"></a><a id="item-trên-nền-và-xác-nhận-nhặt--cp030"></a> [→ Item, stat, pickup và kim](protocol/items-stats.md#item-pickup)
<a id="item-effects"></a><a id="bombfire-stat-giới-hạn-và-nguồn-power--cp031"></a> [→ Item, stat, pickup và kim](protocol/items-stats.md#item-effects)
<a id="item-families"></a><a id="nhóm-item-và-kiến-thức-tái-sử-dụng--cp045"></a> [→ Item, stat, pickup và kim](protocol/items-stats.md#item-families)
<a id="hidden-items"></a><a id="gạch-item-ẩn-và-khởi-tạo-lại-vòng--cp032"></a> [→ Item, stat, pickup và kim](protocol/items-stats.md#hidden-items)
<a id="bomb-chain"></a><a id="nổ-dây-chuyền-hai-bom--cp033"></a> [→ Bom, serial, nổ và capacity](protocol/bombs.md#bomb-chain)
<a id="room-leave"></a><a id="rời-phòng-chủ-mới-và-chọn-loại-sảnh--cp034"></a> [→ Phòng, roster và membership](protocol/room-membership.md#room-leave)
<a id="room-rejoin"></a><a id="rejoin-và-start-sau-chuyển-chủ--cp035"></a> [→ Phòng, roster và membership](protocol/room-membership.md#room-rejoin)
<a id="collision-rescue"></a><a id="va-chạm-enemy-rescue-và-report21--cp036"></a> [→ Trap, death, cứu, kết quả và reset](protocol/life-results.md#collision-rescue)
<a id="needle-selfescape"></a><a id="kim-tự-thoát-và-đồng-bộ-peer--cp037"></a> [→ Trap, death, cứu, kết quả và reset](protocol/life-results.md#needle-selfescape)
<a id="needle-repeat"></a><a id="bị-nhốt-lần-hai-khi-hết-kim--cp038"></a> [→ Trap, death, cứu, kết quả và reset](protocol/life-results.md#needle-repeat)
<a id="contact-precedence"></a><a id="tranh-chấp-cứudiệt-ba-client-thật-và-nhánh-chọn--cp037"></a> [→ Trap, death, cứu, kết quả và reset](protocol/life-results.md#contact-precedence)
<a id="contact-inversion"></a><a id="đối-chứng-đổi-team-slot12--cp038"></a> [→ Trap, death, cứu, kết quả và reset](protocol/life-results.md#contact-inversion)
<a id="round-finish"></a><a id="trapdeath-nhất-quán--kết-quả--vòng-kế-và-host-eof--cp039"></a> [→ Trap, death, cứu, kết quả và reset](protocol/life-results.md#round-finish)
<a id="abort-rejoin"></a><a id="sau-host-eof-client-mới-vào-slot-trống-chủ-mới-start--cp040"></a> [→ Phòng, roster và membership](protocol/room-membership.md#abort-rejoin)
<a id="round-generation"></a><a id="payload-nổ-cũ-qua-vòng-mới-và-tái-dùng-id--cp041"></a> [→ Bom, serial, nổ và capacity](protocol/bombs.md#round-generation)
<a id="item-generation"></a><a id="xác-nhận-item-cũ-khi-tái-dùng-index-qua-vòng--cp042"></a> [→ Item, stat, pickup và kim](protocol/items-stats.md#item-generation)
<a id="late-bomb-confirm"></a><a id="159-cũ-tới-khi-serial-local-được-dùng-lại--cp043"></a> [→ Bom, serial, nổ và capacity](protocol/bombs.md#late-bomb-confirm)
<a id="abort-timer"></a><a id="timer-bom-còn-chờ-khi-host-eof--cp044"></a> [→ Bom, serial, nổ và capacity](protocol/bombs.md#abort-timer)
<a id="udp-generation"></a><a id="udp-qua-vòng-mới-queue-drain-và-consumer-game--cp045"></a> [→ UDP, peer, movement và stale fences](protocol/udp-peer.md#udp-generation)
<a id="giới-hạn-và-correction-wcp-lịch-sử"></a> [→ Bootstrap, WCP và codec](protocol/bootstrap-codec.md#giới-hạn-và-correction-wcp-lịch-sử)
<a id="5-authority-và-profile-emulator"></a> [→ Authority, phạm vi service và ranh giới Rust](protocol/rust-boundary.md#5-authority-và-profile-emulator)
<a id="authority"></a><a id="authority-matrix-hiện-hành-và-gate-review"></a> [→ Authority, phạm vi service và ranh giới Rust](protocol/rust-boundary.md#authority)
<a id="probe-service"></a><a id="service-fixture-và-phạm-vi-tích-hợp--cp048"></a> [→ Authority, phạm vi service và ranh giới Rust](protocol/rust-boundary.md#probe-service)
<a id="pickup-round-key"></a><a id="guard124-theo-khóa-đã-cấp--cp049"></a> [→ Item, stat, pickup và kim](protocol/items-stats.md#pickup-round-key)
<a id="service-eof"></a><a id="service-eof-và-loading-chờ-đủ99--cp050"></a> [→ Map, Ready, loading và phase](protocol/load-phase.md#service-eof)
<a id="service-replacement"></a><a id="client-mới-thay-slot-sau-eof-trên-service-độc-lập--cp051"></a> [→ Phòng, roster và membership](protocol/room-membership.md#service-replacement)
<a id="setup-eof"></a><a id="guest-rời-lúc-setup-trước-mapstart--cp052"></a> [→ Phòng, roster và membership](protocol/room-membership.md#setup-eof)
<a id="host-setup"></a><a id="host-eof-khi-survivor-chưa-map--cp053"></a> [→ Phòng, roster và membership](protocol/room-membership.md#host-setup)
<a id="prestart-replacement"></a><a id="c-mới-vào-trước-vòng-đầu-sau-setup-eof--cp054"></a> [→ Phòng, roster và membership](protocol/room-membership.md#prestart-replacement)
<a id="pickup-tick"></a><a id="nguồn-và-lifetime-của-t-trong124--cp055"></a> [→ Item, stat, pickup và kim](protocol/items-stats.md#pickup-tick)
<a id="pickup-report-state"></a><a id="guard124-theo-report-nhốtchết-của-fixture--cp056"></a> [→ Item, stat, pickup và kim](protocol/items-stats.md#pickup-report-state)
<a id="codec-stream-boundaries"></a><a id="codec-incremental-trên-actual-frame--cp057"></a> [→ Bootstrap, WCP và codec](protocol/bootstrap-codec.md#codec-stream-boundaries)
<a id="live-codec-stream"></a><a id="codec-trên-socket-wine-thật-split-coalesced-và-crc--cp180"></a> [→ Bootstrap, WCP và codec](protocol/bootstrap-codec.md#live-codec-stream)
<a id="codec-fatal-close-followup"></a><a id="đường-đóng-thông-báo-lỗi-codec--controls-cp181"></a> [→ Bootstrap, WCP và codec](protocol/bootstrap-codec.md#codec-fatal-close-followup)
<a id="login-retry-codec"></a><a id="login-rejection-hợp-lệ-và-codec-reset-khi-retry--cp181"></a> [→ Đăng nhập, token, account và lease](protocol/auth-session.md#login-retry-codec)
<a id="rust-foundation"></a><a id="rust-foundation-và-native-login-errorretry--cp231"></a> [→ Bootstrap, WCP và codec](protocol/bootstrap-codec.md#rust-foundation)
<a id="rust-local-lobby"></a><a id="rust-diagnostic-identity--channel--sảnh--profile291--cp232"></a> [→ Sảnh, identity, cache và profile](protocol/lobby-profile.md#rust-local-lobby)
<a id="rust-two-lobby"></a><a id="rust-hai-diagnostic-identity-với-binding-theo-requestendpointstage--cp233"></a> [→ Sảnh, identity, cache và profile](protocol/lobby-profile.md#rust-two-lobby)
<a id="rust-fixed-room"></a><a id="rust-fixedroom--host-backrejoinready--loading-barrierspawn--cp234"></a> [→ Authority, phạm vi service và ranh giới Rust](protocol/rust-boundary.md#rust-fixed-room)
<a id="rust-m2-acceptance"></a><a id="rust-m2-nghiệm-thu-đường-hai-identityphòngloading-và-survivor-sau-eof--cp235"></a> [→ Authority, phạm vi service và ranh giới Rust](protocol/rust-boundary.md#rust-m2-acceptance)
<a id="auth-profile"></a><a id="successprofile-count1-đã-đọc-sạch-trên-copy--cp058"></a> [→ Sảnh, identity, cache và profile](protocol/lobby-profile.md#auth-profile)
<a id="local-name-model"></a><a id="local-idname-và-guard-panel-sảnh--cp182"></a> [→ Sảnh, identity, cache và profile](protocol/lobby-profile.md#local-name-model)
<a id="own-profile-291"></a><a id="profile-bản-thân-ở-sảnh-sc291--cusermap--panel--cp183"></a> [→ Sảnh, identity, cache và profile](protocol/lobby-profile.md#own-profile-291)
<a id="own-profile-integration"></a><a id="opt-in-identity291-và-sequence-sảnh--phòngmap--cp184"></a> [→ Sảnh, identity, cache và profile](protocol/lobby-profile.md#own-profile-integration)
<a id="own-profile-result-compare"></a><a id="own291-mở-đường-result-name-compare-và-sửa-import-trên-copy--cp187"></a> [→ Sảnh, identity, cache và profile](protocol/lobby-profile.md#own-profile-result-compare)
<a id="current-basic-contest-reset"></a><a id="basic-hiện-hành-tranh-roller-baseline-từng-vai-và-hai-kết-quảreset--cp189"></a> [→ Authority, phạm vi service và ranh giới Rust](protocol/rust-boundary.md#current-basic-contest-reset)
<a id="current-basic-cycles-291"></a><a id="basic-hiện-hành-sau291importabi-hai-outcomesreset-peer-và-eof--cp188"></a> [→ Trap, death, cứu, kết quả và reset](protocol/life-results.md#current-basic-cycles-291)
<a id="current-291-room-integration"></a><a id="current-hai-client291nameguard-phòngloadstart-và-giới-hạn--cp187"></a> [→ Phòng, roster và membership](protocol/room-membership.md#current-291-room-integration)
<a id="channel-cold-route"></a><a id="cold-cache-route-kênh-và-tên-hàng--cp186"></a> [→ Sảnh, identity, cache và profile](protocol/lobby-profile.md#channel-cold-route)
<a id="k3-version-identity-controls"></a><a id="version-lỗi-và-ràng-buộc-identity-local--cp186"></a> [→ Sảnh, identity, cache và profile](protocol/lobby-profile.md#k3-version-identity-controls)
<a id="lobby-cache-route"></a><a id="cache-trước-sảnh-và-service-request528--cp185"></a> [→ Sảnh, identity, cache và profile](protocol/lobby-profile.md#lobby-cache-route)
<a id="dynamic-cache-representation"></a><a id="cache-dynamic-map-và-checksum-representation--cp059"></a> [→ Sảnh, identity, cache và profile](protocol/lobby-profile.md#dynamic-cache-representation)
<a id="current-profile-c"></a><a id="integration-c-với-profile-authcache-mới--cp060"></a> [→ Sảnh, identity, cache và profile](protocol/lobby-profile.md#current-profile-c)
<a id="current-pair-round"></a><a id="hai-chu-kỳ-ab-trên-currentprofile--cp061"></a> [→ Authority, phạm vi service và ranh giới Rust](protocol/rust-boundary.md#current-pair-round)
<a id="current-pickup-race"></a><a id="tranh-index-thật-trên-serializedservice--cp062"></a> [→ Item, stat, pickup và kim](protocol/items-stats.md#current-pickup-race)
<a id="report127-freshness"></a><a id="admission-report127-theo-kết-nối--cp063"></a> [→ UDP, peer, movement và stale fences](protocol/udp-peer.md#report127-freshness)
<a id="mixed-owner-chain"></a><a id="hai-owner-control-khoảng-cách-và-blocker-liền-kề--cp064"></a> [→ Bom, serial, nổ và capacity](protocol/bombs.md#mixed-owner-chain)
<a id="distinct-spawn"></a><a id="spawn-riêng-trên-profile-service-hiện-hành--cp065"></a> [→ Map, Ready, loading và phase](protocol/load-phase.md#distinct-spawn)
<a id="distinct-spawn-race"></a><a id="tranh-item-thật-giữa-hai-spawn-riêng--cp066"></a> [→ Item, stat, pickup và kim](protocol/items-stats.md#distinct-spawn-race)
<a id="c2s118-admission"></a><a id="admission-request-đặt-bom118--cp067"></a> [→ Bom, serial, nổ và capacity](protocol/bombs.md#c2s118-admission)
<a id="current-hidden-items"></a><a id="item-ẩn-trong-currentservice--cp068"></a> [→ Item, stat, pickup và kim](protocol/items-stats.md#current-hidden-items)
<a id="basic-battle"></a><a id="một-profile-floorroller--hiddenbombfire--cp069"></a> [→ Trap, death, cứu, kết quả và reset](protocol/life-results.md#basic-battle)
<a id="current-udp-causality"></a><a id="udp-và-vị-trí-hai-chiều-can-thiệp-trên-service-hiện-hành--cp070"></a> [→ UDP, peer, movement và stale fences](protocol/udp-peer.md#current-udp-causality)
<a id="mixed-owner-power"></a><a id="mixed-ownerpower2-control-ngoài-tầm-và-blocker-ui--cp071"></a> [→ Bom, serial, nổ và capacity](protocol/bombs.md#mixed-owner-power)
<a id="preclock-eof"></a><a id="eof-saustart-trước-clock--cp072"></a> [→ Map, Ready, loading và phase](protocol/load-phase.md#preclock-eof)
<a id="launcher-error-attribution"></a><a id="attribution-popup-launcher-và-lỗi-main--cp073"></a> [→ Bootstrap, WCP và codec](protocol/bootstrap-codec.md#launcher-error-attribution)
<a id="preclock-cleanup"></a><a id="native107-và-ui-sau-preclockeof--cp074"></a> [→ Map, Ready, loading và phase](protocol/load-phase.md#preclock-cleanup)
<a id="current-tcp-replay"></a><a id="body-cũtrùng-qua-tcp-channel-thực--cp075"></a> [→ UDP, peer, movement và stale fences](protocol/udp-peer.md#current-tcp-replay)
<a id="back86-membership"></a><a id="back86-và-tạo-lại-phòng-sau-beof--cp076"></a> [→ Phòng, roster và membership](protocol/room-membership.md#back86-membership)
<a id="guest-back-rejoin"></a><a id="guest-backrejoin-cùng-channel-với-host-còn-phòng--cp077"></a> [→ Phòng, roster và membership](protocol/room-membership.md#guest-back-rejoin)
<a id="rejoin-battle-cycles"></a><a id="battle-sau-guest-rejoin-một-vòng-confirmed-hai-vòng-partial--cp078"></a> [→ Phòng, roster và membership](protocol/room-membership.md#rejoin-battle-cycles)
<a id="return585-ready"></a><a id="585-sau-return-và-ready2--cp079"></a> [→ Map, Ready, loading và phase](protocol/load-phase.md#return585-ready)
<a id="ready-input"></a><a id="input-readystart-sau-return-và-hai-chu-kỳ-sau-rejoin--cp080"></a> [→ Map, Ready, loading và phase](protocol/load-phase.md#ready-input)
<a id="mixed-input"></a><a id="hai-owner-trong-tầm-và-root-callback-chain--cp081"></a> [→ Bom, serial, nổ và capacity](protocol/bombs.md#mixed-input)
<a id="mixed-cycles"></a><a id="hai-ownerchain-qua-hai-kết-quả-và-reset--cp082"></a> [→ Bom, serial, nổ và capacity](protocol/bombs.md#mixed-cycles)
<a id="all-dead-result"></a><a id="kết-quả-cả-hai-bị-loại-rank22--cp083"></a> [→ Trap, death, cứu, kết quả và reset](protocol/life-results.md#all-dead-result)
<a id="three-bomb-chain"></a><a id="ba-bom-aba-trong-root-callback-qua-hai-vòng--cp084"></a> [→ Bom, serial, nổ và capacity](protocol/bombs.md#three-bomb-chain)
<a id="tcp-guard-cycles"></a><a id="selected-tcp-guards-cùng-hai-chu-kỳ-native--cp085"></a> [→ UDP, peer, movement và stale fences](protocol/udp-peer.md#tcp-guard-cycles)
<a id="capacity-fresh-serial"></a><a id="native-third-space-control-và-serial-only-divergence--cp086"></a> [→ Bom, serial, nổ và capacity](protocol/bombs.md#capacity-fresh-serial)
<a id="capacity-reservations"></a><a id="optional-profile-capacityreservation-guard--cp087"></a> [→ Bom, serial, nổ và capacity](protocol/bombs.md#capacity-reservations)
<a id="early-chain-reservation-window"></a><a id="normal-replacement-sau-early-chain-bị-quota-giữ-chỗ--cp088"></a> [→ Bom, serial, nổ và capacity](protocol/bombs.md#early-chain-reservation-window)
<a id="chain-capacity-release"></a><a id="release-reservation-trên-đường-clear-trực-tiếp-của-fixture--cp089"></a> [→ Bom, serial, nổ và capacity](protocol/bombs.md#chain-capacity-release)
<a id="current-map-chain-state"></a><a id="exvillage01-static-grid-và-native-box-stopclear-state--cp090"></a> [→ Terrain, block và cascade](protocol/terrain.md#current-map-chain-state)
<a id="house-after-box"></a><a id="native-house-stop-và-mutable-box-flag--cp091"></a> [→ Terrain, block và cascade](protocol/terrain.md#house-after-box)
<a id="map-flags-snapshot"></a><a id="native-map1d54-flags-và-hai-vòng-reset--cp092"></a> [→ Terrain, block và cascade](protocol/terrain.md#map-flags-snapshot)
<a id="terrain-shadow"></a><a id="offline-terrain-candidate-so-native-oracles-đã-lưu--cp093"></a> [→ Terrain, block và cascade](protocol/terrain.md#terrain-shadow)
<a id="terrain-policy"></a><a id="optional-pinned-terraincapacity-policy-và-native-integration--cp094"></a> [→ Terrain, block và cascade](protocol/terrain.md#terrain-policy)
<a id="opposite-box-control"></a><a id="hai-bom-đối-diện-thùng-thật--cp095"></a> [→ Terrain, block và cascade](protocol/terrain.md#opposite-box-control)
<a id="adjacent-owner-control"></a><a id="hai-owner-sát-một-ô-trên-ô-thùng-đã-phá--cp096"></a> [→ Terrain, block và cascade](protocol/terrain.md#adjacent-owner-control)
<a id="terrain-tcp-guard-cycles"></a><a id="terrain-và-tcp-replay-guards-cùng-hai-chu-kỳ--cp097"></a> [→ Terrain, block và cascade](protocol/terrain.md#terrain-tcp-guard-cycles)
<a id="current-peer-targets"></a><a id="native-udp-áp-tọa-độ-đích-trên-currentservice--cp098"></a> [→ UDP, peer, movement và stale fences](protocol/udp-peer.md#current-peer-targets)
<a id="trap-report-rank-order"></a><a id="kết-quả-dựa-trên-report21-cũ-và-a17-đến-sau-rank--cp099"></a> [→ Trap, death, cứu, kết quả và reset](protocol/life-results.md#trap-report-rank-order)
<a id="result-freshness-guard"></a><a id="guard-kết-quả-tùy-chọn-sau-blast--cp100"></a> [→ Trap, death, cứu, kết quả và reset](protocol/life-results.md#result-freshness-guard)
<a id="selector-closed-readiness"></a><a id="key-selector-còn-sau-cleanup-đóng-recipient--cp101"></a> [→ Trap, death, cứu, kết quả và reset](protocol/life-results.md#selector-closed-readiness)
<a id="drained-result-local-clear"></a><a id="drain-job-hiện-hành-và-transition-local-trước-report21--cp102"></a> [→ Trap, death, cứu, kết quả và reset](protocol/life-results.md#drained-result-local-clear)
<a id="peer21-ending-order"></a><a id="peer17021-native17-mới-sau-result-và-proxy-identity--cp103"></a> [→ UDP, peer, movement và stale fences](protocol/udp-peer.md#peer21-ending-order)
<a id="ending34-state-boundary"></a><a id="handler175-tự-chuyển-state-và-report34-trước-lần-ghi--cp104"></a> [→ Trap, death, cứu, kết quả và reset](protocol/life-results.md#ending34-state-boundary)
<a id="ending-udp-state-apply"></a><a id="udp-state-đi-qua-native175-trong-cùng-ending--cp105"></a> [→ UDP, peer, movement và stale fences](protocol/udp-peer.md#ending-udp-state-apply)
<a id="current-movement-cycles"></a><a id="movementterrainending-trong-cùng-hai-vòng-hiện-hành--cp106"></a> [→ UDP, peer, movement và stale fences](protocol/udp-peer.md#current-movement-cycles)
<a id="host-normal-back"></a><a id="host-backchuyển-chủrejoin-trên-service-hiện-hành--cp107"></a> [→ Phòng, roster và membership](protocol/room-membership.md#host-normal-back)
<a id="promoted-battle-cycles"></a><a id="battle-sau-bpromotion-partial-tại-input-control--cp108"></a> [→ Trap, death, cứu, kết quả và reset](protocol/life-results.md#promoted-battle-cycles)
<a id="promoted-outcome"></a><a id="hai-nativeoutcomereset-sau-host-đổi-vai--cp109"></a> [→ Trap, death, cứu, kết quả và reset](protocol/life-results.md#promoted-outcome)
<a id="return2-visual"></a><a id="renderready-sau-return2-trên-bhost--cp110"></a> [→ Map, Ready, loading và phase](protocol/load-phase.md#return2-visual)
<a id="receiver-prepost"></a><a id="receiver-prepost-giải-missing-progress-của-cp110--cp111"></a> [→ Map, Ready, loading và phase](protocol/load-phase.md#receiver-prepost)
<a id="death-envelope-admission"></a><a id="envelope-report1719-trước-admission--cp112"></a> [→ Trap, death, cứu, kết quả và reset](protocol/life-results.md#death-envelope-admission)
<a id="needle-service"></a><a id="kim145-trong-current-service-credit-và-report-pair-opt-in--cp113"></a> [→ Item, stat, pickup và kim](protocol/items-stats.md#needle-service)
<a id="needle-round-reset"></a><a id="kim-cạn-và-freshcredit-sau-start2-trên-current-service--cp114"></a> [→ Item, stat, pickup và kim](protocol/items-stats.md#needle-round-reset)
<a id="rescue-service"></a><a id="rescue18-opt-in-trên-current-service--cp115"></a> [→ Trap, death, cứu, kết quả và reset](protocol/life-results.md#rescue-service)
<a id="rescue-reverse"></a><a id="cứu-chiều-b-mắc-bóng--a-chạm--cp116"></a> [→ Trap, death, cứu, kết quả và reset](protocol/life-results.md#rescue-reverse)
<a id="rescue-return-start-guard"></a><a id="sau-cứu-resultreturn-có-start2-bị-room-guard3--cp117"></a> [→ Map, Ready, loading và phase](protocol/load-phase.md#rescue-return-start-guard)
<a id="rescue-start-slot"></a><a id="slot-gây-guard3-sau-return--run-lưu-trước-migration-cp118"></a> [→ Map, Ready, loading và phase](protocol/load-phase.md#rescue-start-slot)
<a id="rescue-slot-refresh"></a><a id="diagnostic-slot2-ready-refresh-mở-lại-start2--cp119"></a> [→ Map, Ready, loading và phase](protocol/load-phase.md#rescue-slot-refresh)
<a id="rescue-round2-feedback"></a><a id="fresh-rescue-sau-resultreturnstart2--cp120"></a> [→ Trap, death, cứu, kết quả và reset](protocol/life-results.md#rescue-round2-feedback)
<a id="rescue-two-results"></a><a id="hai-vòng-cứu--kết-quả--về-phòng-và-readyb-lần-hai--cp121"></a> [→ Trap, death, cứu, kết quả và reset](protocol/life-results.md#rescue-two-results)
<a id="recursive-chain"></a><a id="chain-sâu-ngoài-tia-bom-gốc-và-so-candidate-terrain--cp122"></a> [→ Bom, serial, nổ và capacity](protocol/bombs.md#recursive-chain)
<a id="recursive-chain-cycles"></a><a id="chain-sâu-cùng-survivorresultreset-hai-epoch--cp123"></a> [→ Bom, serial, nổ và capacity](protocol/bombs.md#recursive-chain-cycles)
<a id="death-lifecycle-policy"></a><a id="corpus17escapefresh1719-và-admission19-opt-in--cp124"></a> [→ Trap, death, cứu, kết quả và reset](protocol/life-results.md#death-lifecycle-policy)
<a id="normal-battle-exit"></a><a id="normal-guest-battle-exit20286-lobby-và-survivor-return--cp125"></a> [→ Phòng, roster và membership](protocol/room-membership.md#normal-battle-exit)
<a id="battle-exit-rejoin"></a><a id="same-channel-rejoinreadystart2-sau-normal-b-battle-exit--cp126"></a> [→ Phòng, roster và membership](protocol/room-membership.md#battle-exit-rejoin)
<a id="battle-exit-pending-bomb"></a><a id="một-bom-b-đang-chờ-khi-normal-guest-exit--cp127"></a> [→ Bom, serial, nổ và capacity](protocol/bombs.md#battle-exit-pending-bomb)
<a id="postbomb-exit-rejoin"></a><a id="pending-b-bombnormal-exitrejoinstart2-cùng-run--cp128"></a> [→ Phòng, roster và membership](protocol/room-membership.md#postbomb-exit-rejoin)
<a id="host-battle-exit"></a><a id="host-a-normal-battle-exitb-promotionarejoinstart2--cp129"></a> [→ Phòng, roster và membership](protocol/room-membership.md#host-battle-exit)
<a id="udp-membership-lifetime"></a><a id="exact-old-udp-sau-host-remove107rejoin--cp130"></a> [→ UDP, peer, movement và stale fences](protocol/udp-peer.md#udp-membership-lifetime)
<a id="shared-blocker"></a><a id="shared-blocker-hai-tia-trong-một-root-cascade--cp131"></a> [→ Terrain, block và cascade](protocol/terrain.md#shared-blocker)
<a id="shared-flag-timing"></a><a id="sharedbox-flag-tại-process-entry-và-root-return--cp132"></a> [→ Terrain, block và cascade](protocol/terrain.md#shared-flag-timing)
<a id="shared-cell-state"></a><a id="shared-traversal-cellstate-chặn-query-lặp--cp133"></a> [→ Terrain, block và cascade](protocol/terrain.md#shared-cell-state)
<a id="shared-cascade-cycles"></a><a id="shared-cascadeactual-all-deadresultreturnreset-hai-vòng--cp134"></a> [→ Terrain, block và cascade](protocol/terrain.md#shared-cascade-cycles)
<a id="loading-timeout"></a><a id="loading-barrier-và-deadline-tùy-chọn--cp135"></a> [→ Map, Ready, loading và phase](protocol/load-phase.md#loading-timeout)
<a id="loading-timeout-positive"></a><a id="positive-loading-deadline-qua-hai-vòng--cp136"></a> [→ Map, Ready, loading và phase](protocol/load-phase.md#loading-timeout-positive)
<a id="integrated-survivor"></a><a id="normal-survivormovement-với-current-trap-và-deadline--cp137"></a> [→ Trap, death, cứu, kết quả và reset](protocol/life-results.md#integrated-survivor)
<a id="peer-endpoint-ip"></a><a id="ip-endpoint153-và-udpsource-tuple--cp138"></a> [→ UDP, peer, movement và stale fences](protocol/udp-peer.md#peer-endpoint-ip)
<a id="virtual-peer"></a><a id="direct-native-udp-giữa-hai-virtualhost--cp139"></a> [→ UDP, peer, movement và stale fences](protocol/udp-peer.md#virtual-peer)
<a id="virtual-basic-cycles"></a><a id="hai-basic-bombresultreset-cycles-trên-virtualhosts--cp140"></a> [→ Trap, death, cứu, kết quả và reset](protocol/life-results.md#virtual-basic-cycles)
<a id="four-netem-restart"></a><a id="samebundlestore-freshserver-và-bốn-peer-dưới-udp-delayloss--cp207"></a> [→ Bốn/tám người, đội và admission](protocol/multiplayer-team.md#four-netem-restart)
<a id="four-health-identity"></a><a id="health-local-correlation-và-identity-pin-tối-thiểu--cp206"></a> [→ Bốn/tám người, đội và admission](protocol/multiplayer-team.md#four-health-identity)
<a id="four-member-abort-room"></a><a id="bốn-session-mất-guesthost-khi-bom-còn-chờ-ba-survivor-về-phòng--cp205"></a> [→ Bốn/tám người, đội và admission](protocol/multiplayer-team.md#four-member-abort-room)
<a id="four-transport-bounds"></a><a id="bốn-người-recipient-safe-transport-giới-hạn-iorate-và-nativepositive--cp204"></a> [→ Bốn/tám người, đội và admission](protocol/multiplayer-team.md#four-transport-bounds)
<a id="four-portable-guards"></a><a id="guard-input-bốn-người-và-bundle-riêng-đã-chạy-native--cp203"></a> [→ Bốn/tám người, đội và admission](protocol/multiplayer-team.md#four-portable-guards)
<a id="four-two-team-cycles"></a><a id="hai-vòng-2v2-trên-bốn-client-kết-quảresetstart3--cp202"></a> [→ Bốn/tám người, đội và admission](protocol/multiplayer-team.md#four-two-team-cycles)
<a id="four-owner-bombs"></a><a id="bốn-owner-bom-baseline-118159158--cp201"></a> [→ Bốn/tám người, đội và admission](protocol/multiplayer-team.md#four-owner-bombs)
<a id="four-session-native"></a><a id="bốn-session-kr-thật-phòngloadclock-và-movement--cp200"></a> [→ Bốn/tám người, đội và admission](protocol/multiplayer-team.md#four-session-native)
<a id="eight-portable-battle"></a><a id="gói-tám-người-đã-relocation-hai-kết-quảreset-và-admission-có-giới-hạn--cp211"></a> [→ Bốn/tám người, đội và admission](protocol/multiplayer-team.md#eight-portable-battle)
<a id="eight-two-team-cycles"></a><a id="tám-actual-client-hai-kết-quả-4v4-về-phòng-và-start3--cp210"></a> [→ Bốn/tám người, đội và admission](protocol/multiplayer-team.md#eight-two-team-cycles)
<a id="eight-mesh-owner-bombs"></a><a id="tám-client-movement-mesh-baseline-stat-và-owner-confirmnổ--cp209"></a> [→ Bốn/tám người, đội và admission](protocol/multiplayer-team.md#eight-mesh-owner-bombs)
<a id="eight-session-preview"></a><a id="tám-slot-native-và-adapter-tám-session--cp208"></a> [→ Bốn/tám người, đội và admission](protocol/multiplayer-team.md#eight-session-preview)
<a id="four-slot-native-consumers"></a><a id="bốn-slot-start121peer153-trên-copy-hiện-hành--cp199"></a> [→ Bốn/tám người, đội và admission](protocol/multiplayer-team.md#four-slot-native-consumers)
<a id="native-interleaved-startup"></a><a id="native-startup-xen-kẽ-ab-với-binding-opt-in--cp198"></a> [→ Bốn/tám người, đội và admission](protocol/multiplayer-team.md#native-interleaved-startup)
<a id="operator-incoming-binding"></a><a id="incoming-ab-theo-source-ipv4-riêng-opt-in--cp197"></a> [→ Bốn/tám người, đội và admission](protocol/multiplayer-team.md#operator-incoming-binding)
<a id="nat-netem-host-eof"></a><a id="pending-host-eof-qua-natudp-qdisc-giới-hạn-timestamp-và-probe--cp196"></a> [→ LAN, WAN và Internet thật](protocol/networks.md#nat-netem-host-eof)
<a id="udp-netem-cycles"></a><a id="hai-vòng-qua-nat-với-udp-có-delayjitterloss--cp195"></a> [→ LAN, WAN và Internet thật](protocol/networks.md#udp-netem-cycles)
<a id="nat-two-cycles"></a><a id="hai-vòng-battle-bình-thườngreset-qua-mapping-nat-tĩnh--cp194"></a> [→ LAN, WAN và Internet thật](protocol/networks.md#nat-two-cycles)
<a id="nat-endpoint-control"></a><a id="nat-tuple-nhận-thực-tế-phải-qua-kiểm-endpoint-của-slot--cp193"></a> [→ LAN, WAN và Internet thật](protocol/networks.md#nat-endpoint-control)
<a id="routed-Wine-network"></a><a id="kr-wine-qua-hai-subnetrouter-môi-trường-wine-được-owner-chấp-nhận--cp192"></a> [→ LAN, WAN và Internet thật](protocol/networks.md#routed-Wine-network)
<a id="emulated-lan-acceptance"></a><a id="lan-giả-lập-được-chủ-dự-án-chấp-nhận-reset-b-sau-thắng--cp191"></a> [→ LAN, WAN và Internet thật](protocol/networks.md#emulated-lan-acceptance)
<a id="portable-probe-bundle"></a><a id="service-và-fixture-có-thể-chuyển-thư-mục--cp190"></a> [→ Authority, phạm vi service và ranh giới Rust](protocol/rust-boundary.md#portable-probe-bundle)
<a id="service-address-config"></a><a id="cấu-hình-ipv4-service-và-ready-sau-return2--cp141"></a> [→ LAN, WAN và Internet thật](protocol/networks.md#service-address-config)
<a id="virtual-movement-cycles"></a><a id="movement-hai-chiều-item-và-hai-lifecycle-trên-production-virtualhosts--cp142"></a> [→ UDP, peer, movement và stale fences](protocol/udp-peer.md#virtual-movement-cycles)
<a id="virtual-tcp-guard-cycles"></a><a id="selected-tcp-replay-guard-cùng-virtual-movementtwo-cycles--cp143"></a> [→ UDP, peer, movement và stale fences](protocol/udp-peer.md#virtual-tcp-guard-cycles)
<a id="virtual-pending-guest-eof"></a><a id="actual-pending-guest-eof-sau-hai-normal-cycles--cp144"></a> [→ LAN, WAN và Internet thật](protocol/networks.md#virtual-pending-guest-eof)
<a id="virtual-exact-old-udp"></a><a id="exact-old-udp-từ-cùng-endpoint-trên-direct-virtualhosts--cp145"></a> [→ UDP, peer, movement và stale fences](protocol/udp-peer.md#virtual-exact-old-udp)
<a id="direct-tcp-endpoints"></a><a id="direct-private-tcp-endpoint-chain-trên-copy--cp146"></a> [→ LAN, WAN và Internet thật](protocol/networks.md#direct-tcp-endpoints)
<a id="service-channel-endpoint"></a><a id="endpoint-kênh-cấu-hình-trong-service-và-capture-được-khôi-phục--cp148"></a> [→ LAN, WAN và Internet thật](protocol/networks.md#service-channel-endpoint)
<a id="direct-channel-positive"></a><a id="positive-native-channel-trực-tiếp-service-không-tcpproxy--cp149"></a> [→ LAN, WAN và Internet thật](protocol/networks.md#direct-channel-positive)
<a id="direct-item-feedback"></a><a id="fire-xác-nhận-trước-input-kế-và-bom-hai-owner-trên-directservice--cp150"></a> [→ Item, stat, pickup và kim](protocol/items-stats.md#direct-item-feedback)
<a id="direct-contested-pickup"></a><a id="hai-nativeclaim-cùngfire17-trên-distinctspawndirectroute--cp151-partial"></a> [→ Item, stat, pickup và kim](protocol/items-stats.md#direct-contested-pickup)
<a id="loading-observer-timing"></a><a id="đường-load-trước-writer-99-và-ảnh-hưởng-observer--cp152"></a> [→ Map, Ready, loading và phase](protocol/load-phase.md#loading-observer-timing)
<a id="loading-hot-gate-contest"></a><a id="bỏ-stop-observer-thừa-khi-load-tranh-fire-qua-hai-key-và-hai-kết-quả--cp153"></a> [→ Map, Ready, loading và phase](protocol/load-phase.md#loading-hot-gate-contest)
<a id="direct-contest-pending-eof"></a><a id="eof-thực-của-guest-còn-bom-pending-trên-alltcpdirect--cp154"></a> [→ LAN, WAN và Internet thật](protocol/networks.md#direct-contest-pending-eof)
<a id="direct-oldudp-contest-eof"></a><a id="exactoldudpcurrentpositive-cùng-hai-contest-và-directeof--cp155"></a> [→ LAN, WAN và Internet thật](protocol/networks.md#direct-oldudp-contest-eof)
<a id="direct-host-eof-acceptance"></a><a id="host-eof-hiện-tại-đối-chiếu-acceptance-và-aud047--cp156"></a> [→ LAN, WAN và Internet thật](protocol/networks.md#direct-host-eof-acceptance)
<a id="local-demo-acceptance"></a> [→ LAN, WAN và Internet thật](protocol/networks.md#local-demo-acceptance)
<a id="current-host-ui-exit"></a><a id="thoát-host-bằng-ui-trên-cấu-hình-direct-hiện-tại--cp157"></a> [→ LAN, WAN và Internet thật](protocol/networks.md#current-host-ui-exit)
<a id="current-trap-udp-counter"></a><a id="counter-udp-cùng-vòng-và-protected-trap--cp158"></a> [→ Trap, death, cứu, kết quả và reset](protocol/life-results.md#current-trap-udp-counter)
<a id="production-baseline"></a><a id="hai-vòng-bằng-service-production-không-hold165replay--cp159"></a> [→ Authority, phạm vi service và ranh giới Rust](protocol/rust-boundary.md#production-baseline)
<a id="exported-preset-runtime"></a><a id="pinned-preset-đã-chạy-start3-thật-và-aud048--cp160"></a> [→ Authority, phạm vi service và ranh giới Rust](protocol/rust-boundary.md#exported-preset-runtime)
<a id="current-room-title"></a><a id="tên-phòng-current81--guest91--cp161"></a> [→ Phòng, roster và membership](protocol/room-membership.md#current-room-title)
<a id="current-team-mode"></a><a id="freemanner-selector-và-giữ-qua-return180--cp162"></a> [→ Bốn/tám người, đội và admission](protocol/multiplayer-team.md#current-team-mode)
<a id="mode-preset-input-blocker"></a><a id="committed-mode-preset-first-clock-và-input-blocker--cp164-lịch-sử-cp165-giải-scope-mới"></a> [→ Bốn/tám người, đội và admission](protocol/multiplayer-team.md#mode-preset-input-blocker)
<a id="input-batch-gating"></a><a id="native-batch-observer-control-và-committed-mode-preset-start3--cp165"></a> [→ Bốn/tám người, đội và admission](protocol/multiplayer-team.md#input-batch-gating)
<a id="current-mode-host-rejoin"></a><a id="current-mannertitle-host-back-chuyển-chủ-và-vào-lại--cp166"></a> [→ Phòng, roster và membership](protocol/room-membership.md#current-mode-host-rejoin)
<a id="current-mode-promoted-cycles"></a><a id="hai-normal-cyclesresetready-sau-promotion-trên-currentmode--cp167"></a> [→ Phòng, roster và membership](protocol/room-membership.md#current-mode-promoted-cycles)
<a id="current-mode-promoted-start3"></a><a id="currentmode-bstart3-sau-hai-normalcycles-và-audit--cp168"></a> [→ Phòng, roster và membership](protocol/room-membership.md#current-mode-promoted-start3)
<a id="current-basic-hidden"></a><a id="basicprofile-ghép-trên-currentmodebhost-native-reveal-và-ba-hiệu-ứng--cp169"></a> [→ Item, stat, pickup và kim](protocol/items-stats.md#current-basic-hidden)
<a id="current-basic-cycles"></a><a id="copybasicprofile-currentmodebhost-hai-nativecyclesresetready--cp170"></a> [→ Trap, death, cứu, kết quả và reset](protocol/life-results.md#current-basic-cycles)
<a id="basic-preset-start3"></a><a id="preset-basic-được-export-hai-vòng-và-normal-bstart3--cp171"></a> [→ Map, Ready, loading và phase](protocol/load-phase.md#basic-preset-start3)
<a id="hidden-cell-admission-current"></a><a id="hidden142-ràng-buộc-ô-đã-phá-trong-bounded-terrain--cp172"></a> [→ Item, stat, pickup và kim](protocol/items-stats.md#hidden-cell-admission-current)
<a id="basic-bidirectional"></a><a id="basic-hai-owner-bom-movement-hai-chiều-và-hai-vòng--cp173"></a> [→ UDP, peer, movement và stale fences](protocol/udp-peer.md#basic-bidirectional)
<a id="basic-host-b-eof"></a><a id="basic-pending-b-host-eof-a-cleanuppromotion-và-round-fence--cp174"></a> [→ LAN, WAN và Internet thật](protocol/networks.md#basic-host-b-eof)
<a id="a-survivor-admission"></a><a id="a-ở-lại-nhận-identity-mới-c-vào-slot-b-đã-rời--cp175"></a> [→ LAN, WAN và Internet thật](protocol/networks.md#a-survivor-admission)
<a id="a-c-first-bomb"></a><a id="cslot1-thực-phát118-và-ac-nhận-cùng-bomnổ--cp176-run-đơn"></a> [→ Map, Ready, loading và phase](protocol/load-phase.md#a-c-first-bomb)
<a id="a-c-basic-cycles"></a><a id="ahostcguest-hai-normal-basic-cycles-peer-hai-chiều-và-reset--cp176"></a> [→ Trap, death, cứu, kết quả và reset](protocol/life-results.md#a-c-basic-cycles)
<a id="replacement-profile"></a><a id="preset-thay-người-và-capacity-theo-slot-trống--cp177"></a> [→ Phòng, roster và membership](protocol/room-membership.md#replacement-profile)
<a id="original-windows-control"></a><a id="control-original-kr00967-trên-windows-vm--cp178"></a> [→ Bootstrap, WCP và codec](protocol/bootstrap-codec.md#original-windows-control)
<a id="rust-account-backend"></a><a id="backend-account-rust-độc-lập-client--cp236--rust-m5--kre-074"></a> [→ Đăng nhập, token, account và lease](protocol/auth-session.md#rust-account-backend)
<a id="rust-movement-floor"></a><a id="rust-movement-và-floor-pickup--cp237--rust-m3-partial--kre-074"></a> [→ UDP, peer, movement và stale fences](protocol/udp-peer.md#rust-movement-floor)
<a id="rust-bombs"></a><a id="rust-hai-owner-bom-và-deadline-ticket--cp238--rust-m3-partial"></a> [→ Bom, serial, nổ và capacity](protocol/bombs.md#rust-bombs)
<a id="rust-terrain"></a><a id="rust-boundedterrainrevealba-basicitems--cp239--m3partial"></a> [→ Terrain, block và cascade](protocol/terrain.md#rust-terrain)
<a id="rust-life-results"></a><a id="rust-report-derived-life-kết-quả-và-reset--cp240--m3-partial"></a> [→ Trap, death, cứu, kết quả và reset](protocol/life-results.md#rust-life-results)
<a id="rust-return-cleanup"></a><a id="rust-active-host-eof-với-bom-chờ-forfeit-và-về-phòng--cp241--m3-partial"></a> [→ Trap, death, cứu, kết quả và reset](protocol/life-results.md#rust-return-cleanup)
<a id="rust-m3-acceptance"></a><a id="rust-m3-battle-basic-hai-client-nghiệm-thu-có-giới-hạn--cp242"></a> [→ Authority, phạm vi service và ranh giới Rust](protocol/rust-boundary.md#rust-m3-acceptance)
<a id="rust-account-worker"></a><a id="worker-accountdb-hữu-hạn-và-login-result-handoff--cp243--m5-partial"></a> [→ Đăng nhập, token, account và lease](protocol/auth-session.md#rust-account-worker)
<a id="rust-account-idle-rate"></a><a id="account-idle-ttl-và-worker-rate-limit-schema2--cp244--m5-partial"></a> [→ Đăng nhập, token, account và lease](protocol/auth-session.md#rust-account-idle-rate)
<a id="rust-four-startup"></a><a id="rust-bốn-client-phòng-loading-và-current-movement--cp245--m4-partial"></a> [→ Bốn/tám người, đội và admission](protocol/multiplayer-team.md#rust-four-startup)
<a id="rust-four-owner-bombs"></a><a id="rust-bốn-owner-bom-opt-in-current-native-entity-và-nổ--cp246--m4-partial"></a> [→ Bốn/tám người, đội và admission](protocol/multiplayer-team.md#rust-four-owner-bombs)
<a id="rust-four-life"></a><a id="rust-life-bốn-người-1719-tới-ba-peer-completion-clear21--cp247--m4-partial"></a> [→ Bốn/tám người, đội và admission](protocol/multiplayer-team.md#rust-four-life)
<a id="rust-four-results"></a><a id="rust-bốn-người-hai-kết-quả-2v2-đối-nghịch-resetstart3--cp248--m4-partial"></a> [→ Bốn/tám người, đội và admission](protocol/multiplayer-team.md#rust-four-results)
<a id="rust-four-abort"></a><a id="rust-bốn-người-guesthost-eof-về-phòng-không-xếp-hạng--cp249--m4-partial"></a> [→ Bốn/tám người, đội và admission](protocol/multiplayer-team.md#rust-four-abort)
<a id="rust-eight-baseline"></a><a id="rust-tám-người-baseline-manner4v4-meshbomlifekết-quảreset--cp250--m4-partial"></a> [→ Bốn/tám người, đội và admission](protocol/multiplayer-team.md#rust-eight-baseline)
<a id="rust-eight-teams"></a><a id="rust-fixed1v7-và-native-manner-lệch-đội--cp251-m4-còn-mở"></a> [→ Bốn/tám người, đội và admission](protocol/multiplayer-team.md#rust-eight-teams)
<a id="rust-ninth-admission-wip"></a> [→ Bốn/tám người, đội và admission](protocol/multiplayer-team.md#rust-ninth-admission-wip)
<a id="rust-ninth-admission"></a><a id="visitor-thứ-chín-trên-rust--cp252-phạm-vi-một-kr-thật-và-tám-member-tcp-tổng-hợp"></a> [→ Bốn/tám người, đội và admission](protocol/multiplayer-team.md#rust-ninth-admission)
<a id="argv-provenance-correction"></a><a id="correction-argv-tại-aud072--không-sửa-cp250251-complete-hoặc-raw"></a> [→ Bootstrap, WCP và codec](protocol/bootstrap-codec.md#argv-provenance-correction)
<a id="rust-account-bridge"></a><a id="cầu-nối-account--admission-kr-ở-mức-thư-viện--cp253--m5-partial"></a> [→ Đăng nhập, token, account và lease](protocol/auth-session.md#rust-account-bridge)
<a id="rust-account-serving"></a><a id="account-lobby-opt-in--cp254-native-positive-hai-account-cố-định"></a> [→ Đăng nhập, token, account và lease](protocol/auth-session.md#rust-account-serving)
<a id="rust-account-native-lifecycle"></a><a id="account-native-negativelifecycle--cp255-không-đổi-server-cp254"></a> [→ Đăng nhập, token, account và lease](protocol/auth-session.md#rust-account-native-lifecycle)
<a id="rust-account-fresh-login"></a><a id="account-fresh-process-và-replay-request-native--cp256-m5-vẫn-mở"></a> [→ Đăng nhập, token, account và lease](protocol/auth-session.md#rust-account-fresh-login)
<a id="rust-account-basic"></a><a id="account-owned-phòng-cố-địnhbasic-xuyên-suốt--cp258-m5-còn-mở"></a> [→ Đăng nhập, token, account và lease](protocol/auth-session.md#rust-account-basic)
<a id="vps-internet-preflight"></a><a id="vps-internet-bổ-sung--network-preflight-chưa-nativegame"></a> [→ LAN, WAN và Internet thật](protocol/networks.md#vps-internet-preflight)
<a id="rust-pr329-integration"></a><a id="đồng-bộ-pr329roomrules-sau-accountbasic--cp259"></a> [→ Authority, phạm vi service và ranh giới Rust](protocol/rust-boundary.md#rust-pr329-integration)
<a id="rust-controlled-network"></a><a id="môi-trường-thử-internet-có-allowlist-tường-minh--đang-đo-n1"></a> [→ LAN, WAN và Internet thật](protocol/networks.md#rust-controlled-network)
<a id="metadata-ipv4-của-login-qua-nat--policy-tường-minh-native-auth-đã-đo"></a> [→ LAN, WAN và Internet thật](protocol/networks.md#metadata-ipv4-của-login-qua-nat--policy-tường-minh-native-auth-đã-đo)
<a id="lỗi-chờ-channel-và-http-trong-đường-đóng--bằng-chứng-hẹp"></a> [→ LAN, WAN và Internet thật](protocol/networks.md#lỗi-chờ-channel-và-http-trong-đường-đóng--bằng-chứng-hẹp)
<a id="rust-internet-observation"></a><a id="native-accountbasic-trên-đường-internet-đã-đo--cp261-n1-còn-mở-môi-trường"></a> [→ LAN, WAN và Internet thật](protocol/networks.md#rust-internet-observation)
<a id="field51-của-login-là-nguồn-ngưỡng-chờ-đã-đo"></a> [→ LAN, WAN và Internet thật](protocol/networks.md#field51-của-login-là-nguồn-ngưỡng-chờ-đã-đo)
<a id="đường-account-và-trận-đã-chạy"></a> [→ LAN, WAN và Internet thật](protocol/networks.md#đường-account-và-trận-đã-chạy)
<a id="99153-nguồn-sau-nat-và-native-udp"></a> [→ LAN, WAN và Internet thật](protocol/networks.md#99153-nguồn-sau-nat-và-native-udp)
<a id="ca-chung-nat-và-điều-kiện-gate-còn-thiếu"></a> [→ LAN, WAN và Internet thật](protocol/networks.md#ca-chung-nat-và-điều-kiện-gate-còn-thiếu)
<a id="rust-r1b-policies"></a><a id="r1b--chính-sách-phòng-có-tên-profile-cũ-giữ-wire--cp262"></a> [→ Authority, phạm vi service và ranh giới Rust](protocol/rust-boundary.md#rust-r1b-policies)
<a id="rust-r1c-player-slots"></a><a id="r1c--identity-slot-và-một-template-106--cp263"></a> [→ Phòng, roster và membership](protocol/room-membership.md#rust-r1c-player-slots)
<a id="rust-r1d-wire-builders"></a><a id="r1d--header-chuẩn-và-builder-theo-opcode--cp264"></a> [→ Authority, phạm vi service và ranh giới Rust](protocol/rust-boundary.md#rust-r1d-wire-builders)
<a id="rust-n1-acceptance"></a><a id="n1--quyết-định-phạm-vi-internet-thật--cp264"></a> [→ LAN, WAN và Internet thật](protocol/networks.md#rust-n1-acceptance)
<a id="rust-r1e-registry"></a><a id="r1e--registry-phòng-listing-và-batch-origin--cp265"></a> [→ Phòng, roster và membership](protocol/room-membership.md#rust-r1e-registry)
<a id="rust-n1b-dual-nat"></a><a id="n1b--hai-phía-nat-đổi-cổng-kết-quả-âm--cp266"></a> [→ LAN, WAN và Internet thật](protocol/networks.md#rust-n1b-dual-nat)
<a id="rust-relay-endpoint-scope"></a><a id="ranh-giới-endpointrelay--thiết-kế-cp267"></a> [→ UDP, peer, movement và stale fences](protocol/udp-peer.md#rust-relay-endpoint-scope)
<a id="documentation-checkpoint-identity"></a><a id="danh-tính-checkpoint-thuần-tài-liệu--correction-cp268"></a> [→ Authority, phạm vi service và ranh giới Rust](protocol/rust-boundary.md#documentation-checkpoint-identity)
<a id="rust-r1f-session-binding"></a><a id="r1f--identityprofile-theo-session-và-endpoint-theo-channel--cp269"></a> [→ Phòng, roster và membership](protocol/room-membership.md#rust-r1f-session-binding)
<a id="rust-r1g-admission"></a><a id="r1g--ngân-sách-transport-và-allowlist--cp270"></a> [→ Authority, phạm vi service và ranh giới Rust](protocol/rust-boundary.md#rust-r1g-admission)
