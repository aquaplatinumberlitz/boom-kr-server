# CP147 — Roster8, capacity và nhánh cân bằng đội

[Contract duy nhất/giới hạn](../../PROTOCOL.md#battle-eight-roster),
[câu hỏi](../../RESEARCH.md#gap-roster-ready-host), [thứ tự](../../PLAN.md#current-order).
Baseline `c671301` gồm CP146 chưa merge; nhánh này giữ nguyên CP146 và mọi source/raw/history.
Owner/reviewer Codex, [review độc lập waived](../checkpoints/review-waiver.json); chưa đóng issue14/gate.

Nguồn đối chiếu trước nhánh: CP018106/110/Ready field ledger, CP019 local map dump/getterEDC890,
CP020121/slot26 để tránh suy roster manager thành giới hạn người; `boomkr-re/docs/game-spec-web.md`
§2–3 (nguồn web, không runtime KR), `boom-re/docs/oc_a-room.md` và nguồn lobby trong REFERENCES.
`reuse-question.json` pin source/build/SHA/locator, điều đã biết, phần dùng trực tiếp/phải kiểm KR và
phép đo tối thiểu. Không tìm lại codec/loader hoặc chuyển opcode/layout VN. Dump1151×52 đã đủ trả
capacity278; nhánh mới chỉ đo roster/Ready/Start và phần thiếu ở balance guard.

Raw ở `local/research/codex/eight-player-r1/`; recipe JSON từng run ghi command đầy đủ.
`prepare.py` tái dùng builder106 CP018 vào đường mới, `prepare_variants.py` chuẩn bị toàn bộ variant
trước chạy tuần tự; **không đổi source/config giữa run**. `wine_probe_eight_*.py` giữ namespace recipe
CP019: user/net/mount/PID riêng, chỉ lo/no route, NSS files-only, stub loopback, prefix và game copy
riêng. Mỗi run khởi động một CA thật; slot còn lại tổng hợp. Không dùng production probe_service
hoặc thay state manager/count/Start return qua debugger. GDB giữ init suppression1A966FC=0 của
recipe CP019, vì vậy đây vẫn là copy chẩn đoán; không coi là Windows nguyên bản.

| Run | Kết quả và thời gian có cả setup/cleanup | run.json SHA256 |
|---|---|---|
| `four-four` | 76.004s; check3; actual104=0 | `47f60f0c5951c8f44126a73eeb5f09a74702e0e777f2ef9ea5cfb33ea39b53ed` |
| `ready-four-four` | 75.373s; check0; actual104=1 | `f4d5a5cf10d144e6d8fcc173491d4925d6c5936782d362bbdd18e1a51b299a1f` |
| `ready-one-seven` | 75.376s; check0; actual104=1 | `a0ec0dbebd71beef3d9f3fde9a31adfe156f6833717d45eeb93e2b674943dbe1` |
| `ready-nine` | 75.669s; check5; actual104=0 | `35296b220acb3f9f4a2161e36369f219c6616aaa2037ff51da25bfacd5a32d97` |

Control `four-four` dùng guest flags2 (sai vai trò Ready): callback đọc đủ8 profile nhưng click trả3,
notice yêu cầu Ready, không104. Sửa **input** guest flags4 từ ledger cũ và chạy prefix/game mới;
không sửa log/cấu hình control. Ba run mới đều parser106 cursor196/196/errors0, callback xong từng
slot; tên KR0..KR7 (ca9 thêmKR8) là dữ liệu thiết kế. Ảnh `ready-four-four/run/screen-18.png`
cho tám ô/4đỏ4vàng, host và bảy Ready. Cơ chế/giới hạn không suy riêng từ ảnh.

`verification.json`: **527 joins** cho isolation, thứ tự run đã end trước run kế, SHA executable/harness/
commands, slot/ID/cursor/callback, actual117/238, count/capacity và kết quả **click**. Log còn các check
capacity theo tick/trước table đến; không gộp chúng với kết quả click sau117. Captured104 từ hai ca8
được replay bằng native14EF300 và CRC-sai/partial controls; encoder tự sinh chỉ là fixture. `replay_capture.py`
tái dùng CP019 receiver model; allocator/prefix-off/seq40+3n được khai rõ, không là oracle full auth.

`verify_balance.py`/`balance-emulation.json`:14 ca native blockDCB12C–DCB191, getter123FA10 giữ native;
chỉ singleton544520 và count-index657960 mô hình. `start-guard-instructions.json` và `room-getters.json`
giữ static window/hash/VA (base400000; RVA=VA−400000), không gọi bounded window là toàn hàm.
Bytes guard/getter được đối chiếu nguyên với modO/static. REA `analyze_function(DCAD80)` timeout450s,
`rea-timeout.json` giữ lỗi; không báo có dossier thành công hoặc bỏ mất phép đo runtime để chờ tool.

**Hash/build:** sourceCA `a52545c1…` bất biến; runtime `CA-r2-room-keystate.exe`
`94ee93dc7978a1ab22f25ec877e29ddcab254ca761f2eac22534421e2dd628e8` dùng lại CP018/019;
static copy `660447c2…`, modO `a08950c8…`, map dump `49727b4d…` đều có locator/hash đầy đủ ở
[manifest evidence CP147](../checkpoints/evidence/CP147-eight-player.json).
`raw-seal.json` hash **296 output files** sau `run.json`/end/cleanup; game/prefix loại khỏi raw index,
executable pin riêng. Cleanup wineserver đúng prefix rồi process groups và exit namespace của recipe;
không process của run còn lại. Không binary/asset/decompile được đưa vào Git.

Chưa8 client thật,121/load/Battle8, UDP mesh/room-full service/reset8, tên/input selector1,
Manner1–7 negative runtime, hay Mission/Respawn. Service roleA/B/C và slot cố định vẫn là blocker
mở rộng; chưa port luật web hoặc nhận8 synthetic rows thành integration. Kiểm requiredlocal/history,
docs/catalog/issues và diff ghi tại `checks/`; semantic/gate do người đối chiếu, script không tự nâng.
AUD044 qua144, audit định kỳ kế **CP148**. Bước tiếp theo duy nhất ở PLAN; không mở plan theo phiên.


<a id="cp163"></a>
## CP163 — Manner1 actual8 slot/Start4–4/negative1–7

Baseline CP162 [PR233](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/233),
head `a00e2040c123971bd5ee03a142f909bc0409af93`, merge `a92178f4422bd183d24ab8f65555395ddbb7e146`.
[Contract duy nhất/giới hạn](../../PROTOCOL.md#battle-eight-roster), [PLAN](../../PLAN.md#current-order).

Raw `local/research/codex/balanced-eight-r1/`; prep `prepare_balanced_eight.py`, controllers
`wine_probe_balanced_ready_four_four.py` và `wine_probe_balanced_ready_one_seven.py`. Reuse nguyên
CP1478profile/Ready4/map/getter/Start recipe và CP162 actual81→86selector echo. Mỗi variant frozen
harness/fixture/GDB/code trước run; prefix/game-copy riêng, user/net/mount/PID namespace chỉlo/no route,
files-only DNS, private stub. No source/config used during run changes; owned cleanup và namespace extinct
đã kiểm trong raw-seal/preflight; không native manager/count/return/team edits hoặc sourcebinary/DB/pack.

| Variant | Giây/exit/raw | SHA256 run.json |
|---|---|---|
| ready-four-four |76.094563511/0/75|`79b523a14987bdfb11a0eec2405c16456bda300363d74318ad44402d7c07414d`|
| ready-one-seven |75.695825599/0/75|`b321cdf7694a6da96466ddf8f7742bc4085bb2b26c2e5eb0a8ba3c8cb8bc06fe`|

`verify.py`44 joins/nativecapacity/getter/click/request104/actual81→86SHA/only selectorbyte changed;
`verify_native.py`12 actual81/86 nativeframer/string/primitive/CRCpartial cases. CP147 `replay_capture.py`
reused cho toàn captured C2S/nativeCRC/partial, không encoder golden. `visual-review.json` pin hai frame20:
8slot4đỏ4vàng/Startpending và1đỏ7vàng/notice yêu cầu số người các đội bằng nhau. Copy94ee93dc… bất biến,
khác e9bb4766… production; inherited init suppression1A966FC=0 vẫn là modifiedWine. `reuse-question.json`
pin source/hash/mức chứng minh/phần còn thiếu; không dump map/decompile guard/kiểm Free0 hoặc9 người lại.

Chỉ native room Start validation, không121/loading/Battle8/service roster8/8actualpeers/fullmatch/K6a/
Windows/LAN/Internet/originalauthority. `server-source-reuse.json` chứng minh code/preset không đổi,
reuse257server/4export từCP162; docs/catalog/issues/artifact/history/diff kiểm lại trước commit.
CP163 tự kiểm/review waived/chờPR, AUD048 qua160/kếCP164; next committed mode-preset/Start3/AUD049 ởPLAN.


<a id="cp208"></a>
## CP208 — native8consumer và adapter32TCP, tách tám client thật

Baseline CP207 merge[PR279](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/279)/`7ffed54`.
[Contract duy nhất/giới hạn](../../PROTOCOL.md#eight-session-preview), [PLAN](../../PLAN.md#current-order).
Source-first/hash/build/locator: `local/research/codex/eight-session-scope-r1/source-first.json`;
reuse CP147/163/199/200–207 và VN `oc_e-game.md`475–486 cho kiến trúc, không port wire.

`eight-slot-preview-r1`: hai actual A/B/currentcopy4b781fb…/Wine10.0-20260425, sáu synthetic C–H,
private kernel NAT recipe kế thừa CP199; local stub và không external route. Run
`d425e15c2d9d097edf5df77123b277819baad4e871c78c1366f2549f518d616c`,287,091599736s/exit0/1376raw/285frozen.
Raw trước chuyển việc đã seal/namespace extinct; không chỉnh source/config giữa run.
`verification-v2.json`:1709checks/native285-inner256/205/fullcursors/8ID-position/8actors hai view;
`getter-verification.json`:48copiednative52A350, không inferior call. V1 verifier giữ nguyên;
V2 chỉ trỏ source đã freeze trong `runtime-source/` khi repo tiếp tục sửa adapter. Controller không
thoát sớm vì JSON adjacency check sai; bound vẫn hoàn tất/cleanup, không normalmatch proof.

`eight-service-model-r1`: prototype riêng 487checks/32realTCP/syntheticC2S. `eight-service-production-r1`
lặp controls trên **adapterserver mới**,487checks/32transports/eight IDs/names/seven remotes/barrier/negative,
không giả tám client. `eight_production_tcp.py`/logs giữ cả lỗi invocation thiếuPYTHONPATH trước run đã qua.
Initial server testfail do literal\n trong import `eight_player_profile`; đã sửa,357server tests pass;
16bundle/preset tests pass. Test invocation nhầm tên module được giữ trong log ban đầu.
`eight-delivery-r1/bundle`:68pins/47fixturealiases/Python-I/verify/help; không phát hành client/assets.

Tám actual run riêng `eight-native-r1`: setup assert9 kiểm nhầm danh sách5namespace, trước client;
finally sync_service chưa khai báo che lỗi. `partial-raw-seal.json` giữ raw/sourcefreeze/namespace0;
không dựng run.json giả hoặc tuyên bố native đã thử. R2 sửa hai điểm harness, chạy152,435188146s/exit0/870raw/276frozen/cleanup0, runSHA
`cfb7962f754319aa31a2100d75e6d642a4700e7a69daf84e78300a31c5608791`.
`eight-native-r2/verification.json`:1468joins/32nativeTCP/8own291-name-slot/8actual99beforeGO/
8viewfull121285-inner256/153205/ID1–8-position0–7/8distinctactorsclock;192copiedgetters riêng.
`virtual-network.json`:root+8childnetns/distinctIP/prefix/display/CA-wineserverowners/no default route.
Ảnh`eight-native-contact.png` và eight-actual-native-clock.png mỗi view được Codex đọc: Battle vẽ tám
nhân vật, sidebar đủ tám profile; đây là bổ sung, không movement/normalendingproof.
Service chạy trực tiếp source đã freeze; gói8 mới verify/help, chưa native relocationrun.
Pin chi tiết/run manifests và snapshot docs ở evidence CP208 sau khi hoàn tất batch; không chỉnh record cũ.
Review waived; periodic auditCP208/AUD061, audit kếCP212. Internet/K7 và eightfullmatch vẫn mở.


<a id="cp209"></a>
## CP209 — tám mesh/stat/118 và tám owner159/158

Baseline CP208 merge[PR280](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/280)/`8a99a72`;
[contract duy nhất](../../PROTOCOL.md#eight-mesh-owner-bombs), [PLAN](../../PLAN.md#current-order).
Nguồn/hash/build/locators/known-gap-minimum: `eight-mesh-stats-r1/source-first.json` và
`eight-owner-bombs-r1/source-first.json` dưới `local/research/codex/`. Reuse CP208current8copy4b/
Wine10.0-20260425/privatebridge9netns, CP200lowobserver, CP201stat/confirm/blast/fence, CP031powermapping.
Không mới nativefunctionRE/VNwireport; nguyên binary/DB/assets nguồn, copy offline giữhash4b.

| Run riêng, không ghép thành một trận | Kết quả/raw/frozen/cleanup | run.json SHA256 |
|---|---|---|
|eight-mesh-stats-r1|184,016344755s/exit0/891raw/277frozen/namespace0;4610joins/704copiedgetters;8OSwindows/56remoteprogress;2733PCAP14Aedges/8native118rejected|`f209f0aaf0efcb549ac5519804155390eecb0ae96a7982398049afd9fad3cfc0`|
|eight-owner-bombs-r1|241,599624124s/exit0/1107raw/687frozen/namespace0;2256joins/128copiedgetters;8owner/64nativeentity/full15939/full15813|`9476f9f2efd6806edba3741b654d3ecc3543377bde82a5045642ed106e929c6d`|

Controllers `wine_probe_eight_mesh_stats.py`/`wine_probe_eight_owner_bombs.py`, recipes từngrun,
preflight source+copy/fixture hashes và raw-seal trước sửa source. `verify_eight_mesh_stats.py` kế thừa
known8session verifier; `verify_eight_owner_bombs.py` kế thừa CP201 nhưng đọc measured8powers/capacities,
không dán rank4/AB. `getter-verification.json` là copiednative52A350/Unicorn riêng, khônginferior call.
Prepare mesh ban đầu trỏ nhầm report không tồn tại ở runR3 cũ; sửa locator sang CP201 actualpass **trước**
freeze/runtime, giữ `prepare-failure.txt`; không làm lại hoặc chuyển partialR3 thành pass.

672actualTCPsynthetic controls trong `eight-owner-bombs-r1/tcp-control/`: bốn stage mỗiA–H,
source/seq/barrier,8recipient159/158,GHpower2/BEcap2/F1/duplicate/EOFpendingH fence, không nativeproof.
360server/16bundle-preset tests pass. Production8bomb opt-in/profile exact; healthmetadata pinpolicy.
CP20868pinbundle cũ giữ nguyên; chưa export/nativeverify relocationbundle8bomb, không đóng packaginggate.

Native mỗiownerSpace/đợi7s, ID1…8/6sfuse; G/H power2 từ actual118→159. Không probe8raylength/
exactblastentitylookup; bỏ oldwrongE14360entryprobe, không sửa oldraw. 127 vẫn từ chối, không rank/return/reset;
traps/deaths có thể xảy ra ở native nhưng chưa có integration reportlife8. K7/K6cInternet còn mở.
Reviewwaived; AUD061 qua208/audit kếCP212. Mộtbatchmeaningfulmesh+baseline+acceptedbombs, không CP từnginput.
