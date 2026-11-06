# Kế hoạch server giả lập KR 00967

Nguồn chuẩn của project `boom-kr-server`; bản nhập từ plan đã được dsh1/dsh2 review tại `1eee46d`.
CP178: K0 identity/Windowscontrol harness và K2a staticprobe được Codex nghiệm thu theo acceptance/waiver; K1–K7/fullK2/fullmatch giữ mở. CP177/PR249 mergedb7972a8. Tiếp nguồn/coldrecipe K1; AUD053adhocqua178, periodic176/kế180.


### Mục tiêu, baseline và phạm vi

Yêu cầu chủ dự án 26/10/2026: dùng kiến thức RE VN `VBNB_116` để dựng server giả lập cho
**chính `CA.exe` KR `00967`**. Đây là project độc lập `boom-kr-server`; không dùng scope, gate,
registry hoặc tỷ lệ tiến độ V1 của bản viết lại `boom-rs` làm điều kiện của repo này.

Revision kế hoạch **`KR-00967-EMU-1`**, trạng thái **hai client thật đã chạy hai chu kỳ thử trên copy Wine; chưa nghiệm thu trận đầy đủ/K6a**.
Các mốc nguồn được phân biệt rõ:

- **Tài liệu tham chiếu lịch sử:** `boom-rs@dae5e1c717ad31b9e8d32a9b20a76d65942807eb`.
- **Plan KRE trước khi tách, đã review:** `boom-rs@1eee46d0ecf5c7f4eeedeadd7e258ef1cd64bf43`.
- **Snapshot khởi tạo repo mới:** `boom-kr-server@a19c8f68af68b01844c92a7870aa2a22eebf5f5f`;
  các sửa sau migration nằm trong lịch sử của chính repo này.
- **Nguồn RE:** `boom-re@9ac2075bbec0c16217741aaedea968cf685e39a9` và
  `boomkr-re@88e11ed30764c753a00f4f23a2f33dafadc4c890`.

Hash client và ma trận tái sử dụng:
[LEGACY_KR_PROTOCOL](PROTOCOL.md).

Đích đầu: bản copy client KR → bootstrap offline → đăng nhập identity cục bộ → một kênh/sảnh →
tạo/vào phòng → ready/start → Battle thường hai client → kết quả → về phòng → trận thứ hai.
Một map thật của KR, ưu tiên Village00 sau khi xác nhận loader/mode cho phép; nhân vật cơ bản theo
ID/Attr KR. Không áp index Village01, roster, drop override hoặc policy ready của V1 Rust lên client gốc.
Mở rộng bốn client chỉ sau khi hai client ổn. Shop, economy, guild, mode phụ và kết nối Nexon thật
để sau. Không dùng client Rust thay bằng chứng nghiệm thu `CA.exe`.

**Quyết định kỹ thuật:** ưu tiên chứng minh tương thích bằng service nhỏ và state machine theo
client; chưa ghép `boom-core` vào đường trận. VN có P2P và client tự tính một phần gameplay;
KR đã có consumer nổ/trap ở client nhưng chưa có bản đồ authority đầy đủ. Một server mới chạy được
với client Rust không chứng minh có thể điều khiển client KR bằng snapshot/command của ADR 0009.

### Tiến độ hiện hành theo bằng chứng

| Task/mốc | Trạng thái tại CP178 | Nguồn chuẩn |
|---|---|---|
| KRE-004/issue6 | Đã review/merge PR7 và đóng issue:recipe r1 đến local accept;không phải full K1 | [Wine r1](evidence/wine/README.md) |
| Bootstrap reply | PR8 đã merge;debugger ProcessPatchreturn1,không ép success | [bootstrap evidence](evidence/bootstrap-wire/README.md) |
| KRE-005a/issue9,R2 | PR10 đã merge:3 cold start cửa sổ/second accept;issue9 còn mở | [R2 evidence](evidence/wine-r2/README.md) |
| WCP hai-reply | Run CP001 là lịch sử chưa C→S; CP006 đã đo request đầu trên copy tham số hóa | [PROTOCOL](PROTOCOL.md#wcp),[WCP evidence](evidence/wcp/README.md) |
| KRE-005b/issue14 | CP006–176 và PR248 đã merge; CP177 optional replacement preset/C0-C1 capacity/actual118/native159158 có nguồn; task hiện hành ở current-order; fullauth/authority/fullmatch/gates và gaps giữ mở | [Q-023](RESEARCH.md#q-023), [service](evidence/probe-service/README.md) |
| K0 | Đã nghiệm thu identity/harness tại CP178: originalWindowsVM fullfixedcopy/PID/token/cwd/window/ETW/isolation/cleanup; không bootstrap/gameplay success | [Acceptance](#k0-k2a-acceptance) |
| K1 | Windows controls đã định danh ownership; chọn/đối chiếu ba coldstart cùng recipe tới localnetwork attempt vẫn mở | [Gap bootstrap](RESEARCH.md#gap-process-patch) |
| K2a/K2 | K2a static oracle cho probe đã nghiệm thu CP178; full/live K2/authauthority/reconnect/prefix/bounds còn mở | [PROTOCOL](PROTOCOL.md#login-auth),[Q-023](RESEARCH.md#q-023) |
| K3 | Copy đã vẽUI/đóngnotice, A/B/Cprofile đọc sạch và mộtnamedcachewarm được nhận; identityauthority/toàncache/codec/K3 chưa nghiệm thu | [Contract sảnh](PROTOCOL.md#channel-lobby) |
| K4–K7 | [CP166](PROTOCOL.md#current-mode-host-rejoin) currentmodehostBack/rejoin, [CP168](PROTOCOL.md#current-mode-promoted-start3) Bhost2normalcycles/Ready/Start3 có nguồn. [Rà K4](#k4-evidence-review), [authority](PROTOCOL.md#authority) và [rà basic K5/K6a](#basic-acceptance-review) giữ scope; chưa gate/fullmatch/LAN/Internet/8client. | [Coverage/gap](COVERAGE.md), [membership](RESEARCH.md#gap-room-membership) |

<a id="current-order"></a>
### Thứ tự công việc hiện hành

CP048 tổ chức catalog/service fixture, CP049 bổ sung guard124 trong scope hẹp;
[KRE-011/issue56](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/56) giữ acceptance của refactor.
KRE-005b/issue14 vẫn mở. Các bước dưới là **thứ tự ưu tiên**, không tự thay dependency/nghiệm thu K0–K7.

**Yêu cầu chủ dự án01/11:** [làm liên tục qua các checkpoint](evidence/owner-directives/continuous-work.json);
xong checkpoint/commit/PR thì chuyển ngay việc kế tiếp trong thứ tự này, không tự dừng/chờ nhắc.
Một blocker chỉ kết thúc nhánh đo tương ứng; ghi rõ rồi tiếp việc độc lập có thể làm, không tự đóng gate.

KRE-073/[#188](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/188) đã giao
[PR189](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/189)/merge`dae23b7`, CP118 vàđóng;
selectiveCLI/historicalmigration xong, không còn taskactive tổ chức. CP119 đãtiếpactualRE sau merge.
Yêu cầu kiểm8 người có kết quả hẹp [CP147/163](PROTOCOL.md#battle-eight-roster), không thay gate:
Free0/Manner1 input/ACK/reset đã nối CP162; selector1 actual4–4/1–7 đã đo. Không lặp roster/dump/guard. Service8 peer
và trận8 client thật là phần mở rộng sau hai/bốn client ổn ở bước5, chưa được đánh dấu hoàn thành.
1. **KRE-002/KR-B001–B002 — khép acceptance bootstrap K1 theo recipe đang dùng:**
   `NEXT-LOCAL-ACCEPTANCE` khép tại [CP178](#k0-k2a-acceptance): K0 identity/harness và bước K2a
   RE tĩnh đã đủ acceptance; K1/fullK2/K3/dependencyK4–K7 giữ mở. Original Windows CA, NewPatcher,
   Both mỗi recipe một control, actual token/cwd/module/ETW/window/isolation/cleanup đã kiểm.
   CA có modal/CALog marker `ProcessPatch`, exactPC/return chưa đo; không là ba coldstart K1pass.
   CP177 đã merge [PR249](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/249) `b7972a8`;
   source/default/service không đổi ở CP178. Không lặp hai ACgames/native consumers/profile replacement.
   `NEXT-K1-COLD-RECIPE`: source-first WineR2 ba coldstart/CP004–006/WCP/source patch provenance
   và current e9copy/bootstrap inputs/coldrun đã có. Đối chiếu **cùng recipe**, PID/API/localattempt,
   source/copy/config/cache và cleanup; chỉ chạy ba coldstart tối thiểu nếu bằng chứng cũ chưa đáp
   đúng recipe/acceptance. Reuse +E78/multiserveridx và bootstrap/WCPoracle, không tìm lại item/codec.
   Ưu tiên Wine copy đã đo; original Windows control là đối chứng có giới hạn, không suy hai môi
   trường tương đương. KRE-D01 giữ control/patch/sourceSHA/sideeffects/rollback; nếu gặp blocker
   mới thì trace đúng nhánh/consumer/API thay vì thêm genericlifecycle. Không sửa tiêu chí hoặc
   ghép các run khác recipe để đóng K1. Không vào liveauth/Nexon hoặc nới egress.
   [#2](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/2) active bootstrap;
   [#14](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/14) umbrella giữ mở.
   <!-- active-issue: 2 --> <!-- active-issue: 14 --> AUD053 ad-hoc qua178; periodic176/kế **CP180**.

2. **KR-B003/KR-F006 — nợ có giới hạn:** [CP057](PROTOCOL.md#codec-stream-boundaries) đã đo stream/
   seq/bounds trong model, không mở lại76split hoặc cấp buffer lớn chỉ để vượt allocationguardFFFFFFFF.
   Full modes/prefix guard/live reset, distance/auth/transport/kim-cứu của reportguard giữ trong RESEARCH;
   chỉ mở đúng variant gây blocker integration. Không lặp nguồn t/k/forgedbody hoặc mọi EOF.
3. **Battle tối thiểu theo trigger sau integration:** CP084 có ba bom fork, CP095 có actualbox control, CP096 có adjacentmixedowner; CP122–123 deep recursion vàCP131sharedblocker ca hẹp đã đo; genericmoving/order/allmaps còn thiếu. List2 chỉ gắn item vào FixedBox đã có, không tạo tường mới; ca vật cản phải có actualmap/tile/control. Runtime max/speed hoặc ability khi ca Battle dùng
   nhóm đó. Tái dùng [item families](PROTOCOL.md#item-families), không quét mọi row hoặc tìm lại kim/Roller.
4. **Lifecycle còn thiếu:** account reconnect/general membership/idleEOF/sendfailure/timeout/loadfail/
   reset variants chưa đủ; chỉ mở ca có trigger cụ thể khi chặn integration/acceptance, không kiểm
   kê hết mọi race trước validator/auth. C mới không là account resume; một vòng B/C không K6a.
5. **Nghiệm thu local rồi nhiều máy:** gom cùng-run consistency/malformed/stale/reset/disconnect theo
   K5/K6a và [authority](PROTOCOL.md#authority); sau đó K6b LAN, K6c Internet rồi K7 bốn client/hardening.
   Sau baseline hai/bốn client, mở rộng service roster/slot/session/Ready/barrier/peer tới8;
   kiểm8 client thật, reject người thứ9 theo map,4–4/1–7 và kiểu cân bằng sau khi input KR đã phục hồi,
   kết quả/reset hai vòng. Không coi CP147 đã nghiệm thu phần mở rộng này.
   Không ghép các run riêng thành một trận đã nghiệm thu. LAN/Internet/relay hiện unknown.

[Bằng chứng service](evidence/probe-service/README.md) phân biệt run đủ hai chu kỳ với run partial.
Policy fuse/drop/rank/forfeit vẫn là emulator choice. K0 đã nghiệm thu theo đối chiếu thủ công CP178; K2a tĩnh đã đạt. K1/K2/K3/K4/K5/K6/K7 vẫn mở; checker không tự đóng gate.
Chu kỳ audit/counter tại [manifest](evidence/checkpoints/manifest.json), không tạo plan theo phiên.

<a id="issue-order"></a>
### Issue hiện hành và backlog theo nguồn chuẩn

Bảng này chỉ phân công/thứtự; acceptance/PR/bằngchứng ởissue. [Catalog](COVERAGE.md) dẫn featureID→issue,
[reconciliation cóngày](evidence/issue-reconciliation/README.md) giữ lýdođóng/thaythế, khônglàplan thứhai.

| Thứ tự/scope | Issue | Trạng thái/phần còn thiếu |
|---|---|---|
| Hiện hành RE | [#14/KRE-005b](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/14) | CP177/PR249 merged; CP178 K0/K2a accepted theo nguồn; tiếp K1 coldrecipe/source. K6a/fullmatch không tự đóng; gaps giữ; fullauth/authority/fullmatch/gates mở. |
| Control theo trigger/gate | [#1/KRE-001](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/1), [#2/KRE-002](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/2) | CP178 K0 originalWindowscontrol/ownership đã đủ, #1 acceptance ready/pending delivery; #2 tiếpK1 recipe/control/fullbootstrapconfig, reuseWine/import/reply. |
| Codec/identity khi cần | [#3/KRE-003](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/3), [#9/KRE-005a](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/9) | Fullcodec/livebounds vàintermittentmainstartupcause cònthiếu; khônglặpprimitive/auxiliarylocale. |

### Tra theo chức năng

[Cây/coverage/dependency sinh từ catalog](COVERAGE.md) thay bản đồ lặp theo checkpoint tại đây.
Catalog chỉ dẫn về contract/câu hỏi/evidence; không giữ một thứ tự công việc khác.
Lịch sử từng checkpoint bất biến ở [manifest](evidence/checkpoints/manifest.json).

### Bằng chứng lịch sử khi lập plan (26/10)

- Tài liệu KR cũ chưa chứng minh qua `ProcessPatch`. §31 của `boomkr-re/docs/kr.md` ghi một run CA
  còn sống (45 thread/~75 MB), snapshot `netstat` không có kết nối; dòng `EndOfInit! exitcode=1` là
  log run trước. Không gọi đây là trạng thái runtime hiện tại khi chưa chạy lại; snapshot không có
  kết nối cũng chưa loại trừ connect ngắn đã xảy ra. §33 tách dialog **Nexon của NewPatcher** và
  dialog **CA** có title `??? ??`; quan hệ nhân quả với `ProcessPatch` vẫn unknown. Trạng thái unpack
  khác nhau giữa run/dump; không dùng entropy `.themida` để kết luận toàn bộ code game chưa đọc được.
- REA mới đọc được `NewPatcher.exe`: consumer manifest `sub_4118E0` phân biệt `files.nfo`/`files.nfo2`,
  `.gz` và delta; có đường báo lỗi mạng/file. Đây là cạnh RE tĩnh mới cho bootstrap, **chưa** chứng minh
  nguyên nhân `ProcessPatch` lỗi, IPC với CA hoặc schema patch service đầy đủ. Xem
  [evidence bootstrap](evidence/bootstrap/README.md).
- VN đã có codec kiểm bằng hàm binary và server chạy 1–2 client. Dùng lại cách kiểm và kiến trúc
  thăm dò; mọi key/seq/opcode/schema KR cần chứng minh riêng. Không có “port VN sang KR bằng đổi version”.
- `kr.md` §12 tìm **bảng handler VM Themida**, không phải kiểm kê dispatcher packet KR. Không suy
  “không có dispatcher mạng” từ kết quả âm tính về VM. Message nội bộ 27 cũng không phải wire opcode.

<a id="basic-acceptance-review"></a>
### Rà K5/K6a cập nhật audit CP176 — chưa đóng gate

| Acceptance đang dùng | Nguồn/đã biết | Khoảng trống còn lại |
|---|---|---|
| Clock/bom/phá hộp/item/trap/result/reset | [CP176](PROTOCOL.md#a-c-basic-cycles): current A/C2normalcycles/3effects/195/stat/key reset/Ready2; CP171/172 giữ old scope | Không third outcome/toàn inventory/timer/entity variants; chỉ map/profile được đo |
| Authority của hành vi thực dùng | [Matrix](PROTOCOL.md#authority): creator/simulator/validator/broadcaster/peer-editable tách native khỏi policy; guard142 thêm đúng ô | Fullauth/distance/142freshness/original drop/result arbitration unknown; design có kiểm không tự đổi thành luật gốc |
| Đồng bộ A↔B và hai owner bom trong basic run | [CP176](PROTOCOL.md#a-c-basic-cycles) A/C cùng2keys/4nativepeerwindows/C118+Abombs/native195/effects/reset | Chỉ simplecorridor/profile; không fullUDP/auth/movinggeneric/races |
| Contest/disconnect/freshness | CP155/156/158/168 có các control đã biết theo profile/run | [CP174](PROTOCOL.md#basic-host-b-eof) currentbasicBhostEOF/pendingfence/Acleanup đã đo; [CP175](PROTOCOL.md#a-survivor-admission) Cslot1 admission đã mở bằng opt-in/guards; CP176 ACnormalreset đã kiểm. Contest/freshness control cũ giữ riêng, không gọi cùng-run với AC |
| Dependency/đích nhiều máy | K0 và K2a đã đạt CP178; K1 chosenrecipe, K2 livecodec/reset, K3 cache/identity và dependency K4 vẫn mở | K5/K6a/fullmatch chưa chứng nhận; LAN/Internet/4–8client chưa nghiệm thu. Không thêm gate con hoặc phần trăm |

<a id="gates"></a>
### Dependency và gate

```mermaid
flowchart LR
  K0["K0 Identity + harness · đạt CP178"] --> K1["K1 Qua bootstrap"]
  K0 --> K2a["K2a RE codec tĩnh · đạt CP178"]
  K1 --> K2["K2 Hello + codec thật"]
  K2a --> K2
  K2 --> K3["K3 Login + sảnh"]
  K3 --> K4["K4 Phòng + load map"]
  K4 --> K5["K5 Trận 1 client"]
  K5 --> K6["K6 2 client + 2 trận"]
  K6 --> K7["K7 4 client + đóng gói"]
```

| Mốc | Công việc và đầu ra | Gate bắt buộc trước khi mở phần phụ thuộc |
|---|---|---|
| **K0: identity/harness** | Manifest source/copy/dump/DB: SHA, image base, RVA, section hoặc memory-layout, patch và config; DB dùng copy/private workspace, không ghi DB nguồn/shared. Windows VM là control bootstrap chính: copy đầy đủ lên ổ local fixed, cwd cạnh exe/DLL, kiểm manifest `requireAdministrator` và effective token; không dùng share/UNC làm đường chạy control. Mỗi run ghi PID/parent/start-time/image SHA/command line, window owner/class/title, CALog mới, log/capture cùng run ID. | Khớp artifact KR/VN; không ghi vào `clients/`; không nhầm CA/NewPatcher/CAWH hoặc log run cũ. Không suy thiếu DLL chỉ từ dialog (inject từng gây cùng lỗi giả). Có lệnh tái tạo và cleanup chỉ PID thuộc run. |
| **K1: bootstrap** | Ba bước K1a ownership/control → K1b contract/caller → K1c stub/patch có đối chứng, theo định nghĩa dưới bảng. NewPatcher manifest là một giả thuyết để kiểm, không phải route đã chốt. | Ba cold start của cùng recipe đi xa hơn điểm chặn được định danh, tới **network attempt local/state chờ hello**, có event API/capture gắn đúng PID; CALog marker mới nếu có. Không dùng alive/không exit/không còn dòng ProcessPatch làm proof. Màn login thuộc K3; không đòi server trả hello trước K2. |
| **K2a: bước con RE tĩnh** | Tìm primitive reader/writer, framer, checksum/crypto/seq và mode/hello từ **KR**, dùng mỏ neo167/184/183 hoặc socket call graph. Fixture hàm KR thật (emulation nếu đủ context), kiểm cursor/endian/bounds, chiều/socket và reset; lưu phần chưa mở được. | Có oracle độc lập cho phần codec probe K2 cần; không lấy encoder tự sinh golden làm proof. Không đòi registry toàn game trước hello; nếu primitive cần thiết chưa có oracle thì K2 chưa mở, ghi checkpoint Codex. |
| **K2: codec/hello** | RE `send/recv` → buffer/frame → primitive reader/writer → checksum/crypto/seq → callback hello/version; lập profile KR riêng theo từng kết nối và chiều. Fixture dùng hàm KR thật hoặc quan sát KR; probe server chỉ có accept/hello/log. | Client nhận hello, gửi request kế tiếp; parse đủ byte, checksum/seq đúng qua nhiều frame và reconnect. Kiểm boundary length, chia/gộp TCP, đổi mode sau hello và reset state. Golden do encoder tự sinh chỉ là regression. |
| **K3: login/kênh/sảnh** | State machine bootstrap-data → identity local → auth result → redirect/chọn kênh → lobby. RE consumer cache/version và gói tối thiểu theo thứ tự; cấp ID ổn định giữa socket login/channel, ghi encoding chuỗi. Patch/game/redirect socket có codec state riêng theo lifecycle KR đã đo. | Cold cache, cache thiếu và warm cache đều có kết quả rõ; UI identity/kênh đúng, keepalive không timeout. Kiểm gửi đúng byte nhưng sai socket/stage bị từ chối có log; không gửi blob trước stage/cache-path init hoặc tái dùng seq của socket trước. Sai version/identity lỗi rõ. |
| **K4: phòng/load** | RE room list/create/join/slot/host/ready/start; map name/id/spawn/roster/seed/placement; loading barrier và điều kiện chuyển scene. Mỗi field có provenance và ghi vào `kr-emu-dev`; không trộn KR native với `emu_choice`/`design` hoặc áp V1 Rust implicit. Mỗi request có reply/broadcast theo stage. | Hai client thấy cùng phòng/slot/host; map đúng payload KR; spawn khác nhau trên ô hợp lệ; một người chưa load không làm người kia vào trận sai state. Kiểm host rời, join/leave và start lặp; reviewer kiểm source/design của các field cần để load. |
| **K5: trận một client** | Chạy clock thật; di chuyển, đặt/nổ bom, phá block, hiện/nhặt item cơ bản, trapped/dead, kết quả và về phòng. Hoàn thiện authority matrix theo contract §5; fuse/drop/timeout không khôi phục được là `design`, có profile riêng. | Chu kỳ trận hoàn chỉnh bằng client KR; cùng bom có ID ổn định, không xác nhận pickup lặp, không kẹt GO/loading. **Reviewer duyệt authority matrix** cho từng hành vi đang dùng: creator/simulator/validator/broadcast/peer-editable state; unknown lịch sử được ghi giới hạn và policy design tương thích có kiểm, không mặc định server-authoritative. Trace client và capture cùng run. |
| **K6: hai client** | RE peer-discovery, session key/slot, UDP packet/reliability/relay nếu có; chọn P2P local hoặc relay theo consumer thật. Đồng bộ hai chiều, bom/item/trap/result rồi reset round. | A di chuyển B thấy và ngược lại; A đặt bom B thấy; tranh item chỉ một kết quả; hai máy đồng thuận trapped/death/result. Hai trận liên tiếp không rò entity/timer/seq/peer. Disconnect/host rời không treo peer còn lại. Reviewer cập nhật authority matrix bằng ca peer thực; unknown làm thiếu consistency của demo thì chưa qua gate. |
| **K7: hardening/demo** | Mở bốn client, bounded parser, backpressure, rate limit, persistence identity tối thiểu, log/health, loss/jitter và server restart. Đóng gói server Linux cùng config/harness hướng dẫn. | Fresh launch theo README; hai và bốn client tái lập; malformed/duplicate/stale input không nhân kết quả hoặc làm crash server; log xác định được build/run/session/room/round. Báo giới hạn authority và sửa client cần thiết. |

<a id="k0-k2a-acceptance"></a>
### Nghiệm thu K0 và bước tĩnh K2a tại CP178

Codex đối chiếu thủ công từng acceptance ở bảng trên, [review waived](evidence/checkpoints/review-waiver.json);
decision/nguồn/giới hạn tại [evidence CP178](evidence/peer-endpoints/README.md#cp178). Không đổi tiêu chí,
không dùng checker hoặc số checkpoint để đóng gate. **K0 đạt về identity/harness; K2a đạt phần tĩnh
cần cho probe K2. K1/fullK2 và K3–K7 vẫn mở.** PR/Git giữ delivery status; CP178 chưa merge lúc ghi record.

| Acceptance giữ nguyên | Đối chiếu bằng chứng | Quyết định / giới hạn |
|---|---|---|
| K0 source/copy/dump/DB SHA/layout/private provenance | Current5KRartifactverify + VNoriginal/patchedreference hash; source187file tree, fullfixedC copies187/187/hash; old private analysis provenance giữ nguyên | Đạt identity; không suy image memory nguyên bản tương đương copy Wine/model |
| K0 fixedlocal/cwd/admin | CNTFS/DriveType3, CA RT_MANIFEST1 requireAdministrator, actualPEB32cwd/TokenQuery+kernelEvent1high/elevated | Đạt trong Windows10 VM; không bareWindows/gameplay claim |
| K0 PID/parent/start/image/cmd/window/newlog/capture | CA1328, NP9656, Both11136/7876; native+ETW exactstartticks/modulebase; modal đúng PID, freshCALog/PCAP/ETW | Đạt ownership; CA cómarkerProcessPatch/modal, exactPC/return chưa đo; title không là rootcause hoặc K1success |
| K0 isolation/recipe/cleanup | IPv4/6DROP/DNSproxy, owned local sentinelSYN trênbridge/khôngeth0/khôngaccept; inputfreeze/ownedPIDguards/exactfirewallrestore | Đạt harness; không file/registry/networkcontract đầy đủ hoặc máy pristine |
| K2a writer/reader/framer/CRC/crypto/seq/hello | KRnativehelpers/exhaustive, WCP0/0→2/3, CP006/014 actualnativeframe, CP057stream/bounds/resetmodel, CP058strings/cursor/errors0 | Đạt oracle cho probe; model prefixguardoff/cap/FFFFFFFF/fullmodes/livereset vẫn giới hạn K2 |
| Dependency | K0 đã đạt; K2a cho chạy song song K1 theo kế hoạch gốc | K2 vẫn đòi K1 và livecodec acceptance; không đóng toàn KRE-003/#3 |

AUD053 ad-hoc qua178 không reset periodic176/kế180. KRE-001/#1 đủ scope identity/control, chờ PRdelivery;
KRE-002/#2 bootstrap, KRE-003/#3 fullcodec và #14 giữ mở. Thứ tự duy nhất ở current-order.

<a id="k4-evidence-review"></a>
**Rà acceptance K4 tại CP166, cập nhật audit CP168 — ánh xạ bằng chứng, chưa đóng gate:**

| Điều kiện | Nguồn hiện có / giới hạn còn lại |
|---|---|
| Hai client cùng phòng/slot/host; create/join | [CP161 title](PROTOCOL.md#current-room-title), [CP162 mode](PROTOCOL.md#current-team-mode), [CP166 promotion](PROTOCOL.md#current-mode-host-rejoin); synthetic identity/known single room, không general room/password/account authority. |
| Map/roster/spawn hợp lệ | [CP065](evidence/distinct-spawn/README.md) native121/innerledger, point0/1 riêng; current CP165/166 giữ preset/map/probes. Chỉ map/roster đã đo; không suy mọi map/8client. |
| Chờ người chưa load | [CP135/136](evidence/loading-timeout/README.md) connected B99hold/control/deadline và positive, [CP153](PROTOCOL.md#loading-hot-gate-contest) current observer control. Timeout/forfeit là design; chưa currentmodeBhost deadline variant. |
| Host rời, join/leave, Start lặp | [CP167](PROTOCOL.md#current-mode-promoted-cycles) currentmode idlepromotion/Arejoin/Bhost2normaloutcomes/reset/Ready; CP165 AhostStart3 riêng. [CP168](PROTOCOL.md#current-mode-promoted-start3) actualBStart3/nativeclock/render8s đãđo trongrunmới; khôngretroactiveR2whiteframecause hoặc gộp oldprofile/run. |
| Provenance/authority và dependency | [Authority](PROTOCOL.md#authority) tách native consumer khỏi emulator policy; Codex tự kiểm theo waiver. K0/K2a đã đạt CP178; K1 chosenrecipe, K2 fullcodec/reconnect và K3 cache/identity acceptance vẫn mở; chưa chứng nhận dependency K4. |

Rà này không tạo gate con hay scope mới, không buộc RE mọi variant trước demo. Phần đang đo và thứ tự
chỉ ở [current-order](#current-order); contract/unknown tiếp ở PROTOCOL/RESEARCH.

**Các bước nghiệm thu K6, đều chưa đạt:**

- **K6a — hai client cùng host:** identity/slot riêng trong mạng thử cô lập; hai chiều movement/bom,
  cùng trap/death/result và hai trận liên tiếp, không dùng guest tổng hợp thay client B.
- **K6b — hai máy LAN:** lặp cùng ca giữa hai máy, endpoint/port đúng và disconnect/rejoin có log.
- **K6c — hai mạng Internet:** kiểm endpoint/NAT và P2P hoặc relay được chọn; hai bên cùng state/result,
  hai trận liên tiếp, latency/loss/disconnect được ghi. Chỉ endpoint thử do dự án kiểm soát trong allowlist;
  giữ block Nexon/live auth. P2P local đã đo từ CP027; relay/NAT và Internet còn chưa kiểm ở CP028.

K2a chạy song song K1 bằng dump/DB có sẵn. Nếu K1 thất bại, kết quả K2a vẫn là RE tĩnh và K2 chưa
qua gate. Không dựng cả lobby trước khi có hello KR thật. Không hứa ngày xong K6 khi K1/K2 còn
unknown; ước lượng lại sau từng gate từ số câu hỏi đã đóng và lượt đo thực tế.

**Điều kiện network từ K0/K1 trở đi:** cô lập outbound của VM/client trước khi chạy; chỉ cho host
local stub/capture theo allowlist. Không có traffic tới Nexon/NGM/live auth thật là điều kiện pass.
Nếu quan sát attempt tới endpoint thật, giữ egress block, lưu API/log/pcap rồi dừng run để sửa local
redirect/config theo contract; không nới egress hoặc fetch patch CDN để vượt gate. Attempt bị chặn
chưa phải traffic đã tới server thật, nhưng run đó không được dùng làm demo local hoàn chỉnh.

**Ba bước của K1:**

- **K1a — ownership/control:** kiểm ba recipe CA đơn lẻ, NewPatcher đơn lẻ và CA+NewPatcher trên cùng
  bản copy/VM; recipe được chọn để qua gate K1 phải có **ba cold start riêng của recipe đó**. Ghi process tree,
  module CA tại base tương ứng, cửa sổ đúng PID và event file/registry/network bằng quan sát ngoài
  process (ETW/Procmon khi có). Kết quả phải phân biệt exit, loop, alive-stall và modal dialog. Đây là
  characterization; chứng minh stall giúp đóng bước này nhưng **không** cho K1 qua gate network.
- **K1b — contract:** truy `CA!ProcessPatch` riêng và caller/config `NewPatcher!4118E0`, ghép command
  line/IPC/token/file/return mà CA thật sự chờ. Chỉ kết nối hai đường khi có evidence; có thể loại bỏ
  giả thuyết NewPatcher nếu không nằm trên route đã quan sát. Không đoán patch schema từ literal NFO.
- **K1c — thử nghiệm:** điều phối chọn stub local đúng contract trước; nếu không đủ thì patch bản copy
  theo KRE-D01. Giữ control plain, chỉ đổi một yếu tố mỗi ca, rồi kiểm gate ba cold start. Khi cả hai
  lượt không có cạnh/observable mới, dừng nhánh và Codex ghi checkpoint để đổi phép đo.

**Quyết định vận hành DESIGN KRE-D01 (26/10/2026):** theo quyền offline-preservation và chỉ đạo toàn
quyền mới của chủ dự án, Codex được chọn stub launcher/patch service, loại bỏ/đổi tên thành phần AC
và patch **file trên bản copy offline** của CA/NewPatcher khi có bằng chứng cần thiết; không phải xin
lại quyết định đó. Trước mỗi thí nghiệm ghi target/hash, module/process ownership, lý do, offset/RVA,
byte trước/sau, file bị loại/đổi tên, giả thuyết side effect và recipe rollback. Control dùng bản copy
plain đầy đủ, cùng VM/config/cache và log mới; so với ca sửa một yếu tố. Chọn quan sát ngoài process
trước vì inject/hook CA từng làm nhiễu. Không lặp DLL/IAT/prologue hook đã thất bại nếu chưa có điều
kiện mới; patch memory/hook xâm lấn không thuộc recipe mặc định. Runtime cô lập khỏi Nexon, không
phân phối binary/AC; lựa chọn policy này không biến kết quả modified-client thành vanilla behavior.

### Phương pháp khi mở nhánh và điểm dừng

Khi mở một nhánh, áp dụng phương pháp tương ứng; backlog hiện hành chỉ ở mục thứ tự phía trên.

1. **Nhánh bootstrap:** K1a xác định process/cửa sổ và điểm chờ; sau đó kiểm riêng `CA!ProcessPatch`
   và giả thuyết `NewPatcher` consumer/caller/config → tham số/IPC → điều kiện thành công. Đọc request
   thực trước khi dựng HTTP/DNS/launcher stub. Không đoán `version=967`
   là toàn bộ handshake và không lấy URL có trong binary làm endpoint đã quan sát.
2. **Wire từ hai đầu:** xref socket API và serializer từ phía gửi; đi ngược handler nhận từ các
   mỏ neo KR đã biết. Ghép cặp theo field order/width/consumer, không theo số opcode gần nhau.
   Chỉ mở những wrapper/VM chặn hello hoặc packet đang cần, không devirt toàn Themida.
3. **Phục hồi state machine:** ghi state trước/sau, guard, request→reply/broadcast, timer, cache,
   socket nào và byte nào được tiêu thụ; kiểm từng chuyển cảnh rồi mới thêm gói.
4. **Authority trước core:** phân biệt vị trí hiển thị, mô phỏng local, báo cáo client, xác nhận server
   và peer. Không ghép handler KR183 dành cho `CGOGhost` thành movement packet của bomber sống.

Mỗi câu hỏi tối đa **hai lượt**, mỗi lượt mặc định **8 request phân tích / 4 thân hàm mới**;
caller/callee đọc từ cache cùng hash không tính như phát hiện. Hết lượt không có cạnh mới:
ghi entry/blocker, vùng đã tìm, dữ liệu runtime còn thiếu và thí nghiệm tiếp; điều phối tự chuyển
hướng có bằng chứng. Bootstrap/codec đang unknown không được “giải” bằng design packet rồi đánh dấu
tương thích. Design được dùng cho policy server mất nguồn sau khi client đã chấp nhận wire contract.

Runtime dùng bản copy, chỉ network cục bộ; không sửa binary/pack/DB gốc. Linux giữ CI/build server.
Theo chỉ đạo26/10,ưu tiên Wine;recipe/merge ở bảng hiện hành.
Các run socket hai silent giữ trong evidence từng recipe;probe WCP sau đó có phạm vi riêng tại
[PROTOCOL](PROTOCOL.md#wcp). Không ghi kết quả mới thành plan khác.
Windows VM cô lập giữ vai trò
đối chứng nguyên bản; lỗi Wine không tự bác kết quả Windows. Không mở lại CI/đóng gói
Windows/macOS của Rust. Patch offline theo KRE-D01, phạm vi triển khai vẫn theo từng task KRE.

### Kiến trúc server đề xuất và mức tái sử dụng

Triển khai tại `server/` trong **chính repo `boom-kr-server`**; tool ở `tools/`, docs ở `docs/`.
Các repo VN/KR/Rust cũ chỉ là nguồn tham chiếu. Không thêm code/doc mới vào chúng để tránh hai nguồn chuẩn.

```text
CA.exe KR (bản copy) ── TCP ── KrWire/PacketReader ── KrSession/Stage ── lobby/room/round
         │                         │                    │                   │
         └── UDP peer đã đo; relay ? ──────┴────────────────────┴── authority audit ┘
                                   └── raw + decoded trace → fixture/replay
```

| Thành phần | Thiết kế / cách tái sử dụng |
|---|---|
| `kr-wire` (đề xuất) | Buffer TCP có giới hạn, codec state riêng từng socket/chiều, packet registry `(build,direction,stage,opcode)`, reader có cursor và báo trailing bytes. Dùng VN làm reference/fixture control; constants KR vào profile có nguồn. |
| `kr-session` / server (đề xuất) | Auth local, redirect token một lần, ID account/session/slot tách nhau; state machine idempotent, timer gắn round generation. SQLite chỉ cần khi identity contract ổn; không thiết kế economy sớm. |
| Room/round | Một owner cập nhật mỗi phòng, broadcast theo roster, phân biệt request/ack/result. Tái dùng ý tưởng generation guard từ VN; policy ready/host/timeout phải khớp consumer KR hoặc ghi design. |
| UDP | P2P local đã đo ở CP027–028 qua actual99→153 và UDP receive. Relay/NAT còn unknown; kiểm consumer trước lựa chọn cho LAN/Internet. Không dùng 3 port login VN làm endpoint movement. |
| `boom-core` | Ứng viên dùng sau audit authority: shadow validation trước; chỉ phát kết quả authoritative khi chứng minh client có đường nhận/correction phù hợp. Nếu không có, giữ mô hình lai để đạt demo tương thích và ghi rõ giới hạn. |
| Content | Dùng reader pack/map/ODT đã có và inventory KR; map/item/character numeric ID có crosswalk theo build. Client gốc tự render/audio; không port renderer mới để giải wire. |

### Task nhỏ và phân công worker

Các task đang chạy nằm tại chính repo này: **KRE-001 [#1](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/1)**,
**KRE-002 [#2](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/2)**,
**KRE-003 [#3](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/3)**. Issue cũ boom-rs đã được
transfer, không giữ hai bảng việc. Mỗi task có baseline, owner/reviewer, file và acceptance theo AGENTS.

| Task | Câu hỏi / đầu ra | Dependency | Owner đề xuất / reviewer |
|---|---|---|---|
| KRE-001 | Khóa target/copy/dump/DB, chạy control và manifest | — | Codex / tự kiểm (review miễn28/10) |
| KRE-002 | CA ProcessPatch: response/caller/guard; bootstrap local và điểm chặn sau đó; NewPatcher là nhánh tham chiếu | KRE-001 | Codex RE/code/runtime / tự kiểm (review miễn28/10) |
| KRE-003 | KR socket→framer→reader/writer; constants và fixture độc lập | KRE-001; gate runtime cần KRE-002 | Codex RE / tự kiểm (review miễn28/10) |
| KRE-004 | Wine unpack/patch copy + harness, issue6/PR7 đã xong trong scope tới local accept; chưa nghiệm thu toàn K1 | KRE-001/002 | Codex / tự kiểm (review miễn28/10) |
| KRE-005 | Probe server hello, version/mode/seq, qua K2 | KRE-003/004 | Codex / tự kiểm (review miễn28/10) |
| KRE-006 | Login/channel/cache→lobby và K3 | KRE-005 | Codex RE/code / tự kiểm (review miễn28/10) |
| KRE-007 | Room/roster/map/spawn/loading và K4 | KRE-006 | Codex RE/code / tự kiểm (review miễn28/10) |
| KRE-008 | Bomb/item/trap/result, authority map và K5 | KRE-007 | Codex RE/code / tự kiểm (review miễn28/10) |
| KRE-009 | UDP/peer hoặc relay, đồng bộ hai client và K6 | KRE-008; có thể RE static từ KRE-003 | Codex RE/code / tự kiểm (review miễn28/10) |
| KRE-010 | Race/disconnect, bốn client, Linux packaging và K7 | KRE-009 | Codex / tự kiểm (review miễn28/10) |
| KRE-017 / issue68 | Admission C trước round1 sau host setup EOF; B/C played round CP054; baseline75606eb; tự kiểm/integration theo [PR69](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/69) | KRE-005b/CP040/051/053 | Codex / tự kiểm (waived) |
| KRE-016 / issue66 | Host EOF khi B chưa map: deferred cleanup/promotion CP053; baseline3617e69; tự kiểm/integration theo [PR67](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/67) | KRE-005b/CP034/050–052 | Codex / tự kiểm (waived) |
| KRE-015 / issue64 | Guest setup EOF/slot ma: control + native fix CP052, audit AUD016; baselinec764aa5; tự kiểm/integration theo [PR65](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/65) | KRE-005b/CP034/050/051 | Codex / tự kiểm (waived) |
| KRE-014 / issue62 | Service A/B→B/C: C identity3, B Start và một vòng bom/statepeer/result/return CP051; baseline9c6ac92; tự kiểm/integration theo [PR63](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/63) | KRE-005b/CP040/044/050 | Codex / tự kiểm (waived) |
| KRE-013 / issue60 | Service EOF/forfeit/cleanup + hold99 CP050; baselinea6b62cd; integration theo [PR61](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/61) | KRE-005b/CP039–044 | Codex / tự kiểm (waived) |
| KRE-012 / issue58 | Guard124 old/current, native client effect CP049; baseline4d77eaf; integration theo [PR59](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/59) | KRE-005b/CP048 | Codex / tự kiểm (waived) |
| KRE-011 / issue56 | Catalog/cây/coverage và consistency check CP048; baseline5ffd925; PR57 đã merge/issue56 đóng | KRE-005b/current evidence | Codex / tự kiểm (waived) |

Từ chỉ đạo28/10, Codex solo RE/code/runtime và tự kiểm; không gọi dsh/Claude nữa.
Review độc lập được chủ dự án miễn theo [nguồn](evidence/checkpoints/review-waiver.json).
Các tiêu chí evidence/verification/gate và PR vẫn giữ; yêu cầu reviewer trong plan trước đây
được thay bằng tự kiểm theo AGENTS, không tự coi gate đã pass. Report/approval cũ giữ provenance.
Một owner mỗi target/runtime, DB riêng; raw giữ local và checkpoint theo mốc/blocker.



Trong lượt lập kế hoạch26/10 (lịch sử), sau khi mở quyền và sửa alias cwd cũ, hai session đã research/review độc
lập. Findings vòng đầu đã được xử lý; **dsh1 và dsh2 APPROVE nội dung plan ở `5e9ebb64`** theo phạm vi
bootstrap và protocol/authority. Bằng chứng review xem [hồ sơ](evidence/bootstrap/README.md).
Đây là duyệt kế hoạch, không phải đã chạy KR hoặc nghiệm thu gate K0–K7; lỗi điều phối ban đầu không
phải kết quả RE phủ định.
Câu hỏi cần đóng: [Q-022…Q-029](RESEARCH.md).
