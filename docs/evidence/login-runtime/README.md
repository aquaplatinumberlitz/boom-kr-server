# Login request và phản hồi KR — CP014

Owner Codex; tự kiểm, review độc lập được [chủ dự án miễn](../checkpoints/review-waiver.json).
Baseline `42f4c03f3163fd24ec02c1e3c50e24e7d0e146e0` (CP013/PR21 đã merge).
Contract duy nhất ở [PROTOCOL](../../PROTOCOL.md#login-auth); thứ tự tiếp ở
[Q-023](../../RESEARCH.md#q-023). Không đóng K2/K3 hoặc gọi đây là login/sảnh thành công.
Checkpoint complete chỉ nói raw/metadata/docs đã đủ; trạng thái tích hợp lấy từ PR.

## Target, recipe và provenance

Raw tại `local/research/codex/login-runtime-r1/`; hash/size từng file và artifact trong
[manifest CP014](../checkpoints/manifest.json). Mỗi run có recipe, GDB commands, run.json,
client/CALog, ảnh và hash index. Namespace user/net/mount/PID riêng, chỉ loopback, không route IPv4,
NSS files-only; stub, Wine prefix và Xvfb riêng. Không gọi endpoint Nexon thật; cleanup theo namespace.
Game-dir copy dùng tuần tự vì CALog chung. Source binary/DB giữ nguyên.

Runner `run_probe.py` giữ `multiserveridx:0`; `run_probe_token.py` thêm token giả
`token_1:offline-probe`. Reset global1A966FC trong bộ nhớ như CP012, không ép modal result/state.
Các run dùng hello15byte cho ba kết nối, tám fixture317byte cho connection2. Audio tắt trong
prefix riêng; UI click retry chỉ để thăm dò, không dùng để gán nguồn auto-attempt.
Ví dụ tái tạo trên host với raw đã khóa: chạy runner token, exe `CA-r2-login-windowtext.exe`,
46s, harness `wine_probe_auth_netinfo.py`, commands `auth-netinfo.gdb`; recipe run lưu argv đầy đủ.

| Copy | SHA-256 | Thay đổi thêm / rollback |
|---|---|---|
| `CA-r2-no-overlay-tick.exe` | `70708abab77ebd1cfd230a35f4997b8e84330f74442ed77b0b74c7b03180c82c` | Baseline diagnostic CP011/012; giữ nguyên. |
| `CA-r2-login-toolhelp.exe` | `da3dfde6ea357b475d71231c5fe61c80455507521c57b4dcfeeae7cc565b40ce` | Bind384..394 Toolhelp; rollback baseline. |
| `CA-r2-login-enum.exe` | `3e7cc5778a34829f6f073aa6433f0600e268a78874d530ffc386c886c26c7e84` | Bind628 EnumThreadWindows; rollback toolhelp. |
| `CA-r2-login-windowtext.exe` | `a301bafae5c55989f8c7d1d99b9de3c7ebf00d5f31532f0bb0cd48f48aee4511` | Bind62C GetWindowTextA candidate; rollback enum. |
| `CA-r2-login-postmsg.exe` | `de2df3ba68f2e7deff892186d51571e3eae7b613d7cb02227066015a2eb4789d` | Bind598 PostMessageA candidate; rollback windowtext. |

Patch scripts ghi nguồn/hash, slot và bytes cũ, import descriptors/slack mới. Copy cuối kế thừa
NMCO/identity/heap/API/overlay modifications trước đó; không coi là nguyên bản.
`GetWindowTextA` có alternative ABI `GetClassNameA` chưa loại trừ; không tự nâng thành original API.
Tên file local `adapter-*` là tên giả thuyết đầu: routine409CD0 thực tế liệt kê process/thread/window,
không dùng tên file làm bằng chứng về adapter mạng.

## Chuỗi thí nghiệm ngày28/10 UTC

| Run / quan sát | Kết quả và giới hạn |
|---|---|
| `auth-trigger`, `auth-gates` | Sau WCP vào1255C90, bỏ auth vì getter count0. Tìm nguồn0 ở auto routine11010F0/token config. |
| `auth-count1` | Chỉ đổi argument setter count0→1 trong bộ nhớ: qua guard rồi trap2262C60; chưa auth. Không phải recipe cuối. |
| `token1`, `token-trap` | Tham số token đặt count1 tự nhiên; cùng trap. Stack return40A82C→thunk15980EB→slot16EA384. |
| `token-toolhelp`, `auth-observe` | Toolhelp binding qua trap cũ, dừng226426C/slot628 tại40A9BC. |
| `auth-enum` | Vào callback409BE0, trap2264274/slot62C tại409C11. |
| `auth-windowtext` | Count1/status−1/ready1, base ctor opcode1; capture auth80byte, opcode60/58byte và5×15/69byte. |
| `auth-error` | Fixture seq40 giả định sai: opcode4 vẫn dispatch nhưng result đọc2. Không dùng oracle mẫu1byte để xác nhận seq runtime. |
| `auth-error43` | Đo receiverseq40 ở hello,43 ở phản hồi; fixture mới đọc result1 và vào nhánh lỗi tương ứng. Không login thành công. |
| `control-no-token` | Cùng copy cuối/harness cơ sở, bỏ token: count0, chỉ3×C→S15, không opcode1/60. Prefix/timing/port thay đổi giữa cold run; control xác nhận route trong phạm vi đó. |
| `auth-prefix` | Result0/header đã đọc; breakpoint10FD823 nằm giữa instruction10FD822 gây SIGSEGV10FD824. Không quy lỗi này cho client/schema. |
| `auth-prefix-corrected` | Breakpoint sửa10FD822, không thấy signal trong cửa sổ đo; prefix khớp, first-body getter trả0, packet+58=0 tại điểm đó; client còn gửi C→S9/46byte. Không suy reader bounds hoặc full body hợp lệ. |
| `auth-netinfo` | Trả S→C15 sau C→S9: handler1100590 chạy, đo wait0→2→1→3, modal trả3; trap2263E48/slot598 ở13BFD44. |
| `auth-postmsg` | Bind thử PostMessageA, không ép result/state: attempt trả1 rồi tới message handler; trap2265828/slot8FC ở13CA429. Ảnh cuối vẫn UI login kèm crash dialog; chưa xác nhận stage mới hoặc vào sảnh. |

Các phép thử chưa chạy không phải runtime: `login-result0-zerofill.bin`/`replay-zerofill.py`
kiểm giả thuyết header modes0 bỏ XOR field. Oracle bác giả thuyết (getter đọc90 thay0), không inject.
Lần oracle đầu còn chồng buffer input/destination; đã tách trước khi kết luận âm tính này.
`prefix-probe-correction.json` ghi lỗi breakpoint và phạm vi dữ liệu còn dùng được.

## Kiểm độc lập và giới hạn

`replay-auth.py` chạy byte KR14EF300 từ modO SHA
`a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5` trong Unicorn,
synthetic receiver/prefix-check off, allocator thay bằng buffer có sẵn. Capture thật là input;
positive/CRC-sai/partial được kiểm từng frame. Hai u8 đầu auth được native getter đọc4/1.
Đây là oracle code KR, không phải server gốc, semantic token hoặc reset/reconnect đầy đủ.

VN `boomproto.py` chỉ cung cấp candidate encoder cho fixture; snapshot/hash/provenance local.
Các fixture được native/runtime kiểm riêng. CRC mẫu1byte có thể khớp dù plaintext bị đọc sai khi
giả định seq sai; kết quả `auth-error` được giữ nguyên và sửa bằng phép đo receiver thực.
Prefix13byte result0 cố ý thiếu player body/đuôi; việc client gửi9 không xác nhận auth hợp lệ.
S→C15 candidate dùng blob inner string/u16 LE, host loopback/port0/mode0/tail0 theo reader KR và
serializer VN làm mỏ neo; raw `netinfo-recipe.json`. State đã chạy là kết quả runtime trên fixture
chẩn đoán đó, không phải chứng minh dữ liệu user/cache đầy đủ. PostMessageA còn alternative SendMessageA.
Không có full success schema, identity thật, sảnh/phòng/trận hoặc gate mới.

Kiểm CP014: `verify.py` có302 assertions trên16 run, native replay và raw hash;
`check_docs.py`, `check_checkpoints.py --require-local --require-history`, `artifacts.py verify`,
`git diff --check` là kiểm bàn giao. Kết quả cụ thể và hash trong manifest; các checker chỉ kiểm
nhất quán đo được, không tự xác nhận ngữ nghĩa hoặc đóng gate. Audit định kỳ kế tiếp CP016.

<a id="cp214"></a>
## CP214 — token local → request native, count1/count2

Baseline PR286/main `4f9a963`; Codex solo/self-check, review waived.
[Contract và giới hạn](../../PROTOCOL.md#auth-token-request), [câu hỏi tiếp](../../RESEARCH.md#rust-service-gaps),
[metadata ba run/hash](token-request.json). Raw riêng `local/research/codex/auth-token-request-r1/`;
recipe/preflight/runtime-source/GDB/run/clientlog/cleanup có seal trước phân tích hoặc đổi việc.
Wine10.0-20260425, offlinecopy4b SHA `4b781fbcea21e0f6294c729b63be86ab2953d60ad83b0d0c9219359f826107ed`,
namespace cô lập/chỉ localstub; không auth-success reply, không Nexon thật hoặc credentials thật.

Tái sử dụng CP014 count/guards, CP058 reply và CP181 retry/codec; source-first pin16file/docs/cache/VN/KR cũ.
Tránh đo lại hello, dựng authreply137/291, profile, Battle hoặc port VNlogin. Chỉ bổ sung ba input token giả
có kiểm native writer và capture. Count1 A/B: frame80/body42, count2: frame95/body57;
mỗi record15byte, suffix25byte giữ nguyên. `token-request.json` dẫn runSHA/frames/inputSHA/offsets;
không commit dummy token bytes hoặc assembly. Bốn native record khớp đầy đủ; ba actualframe replay thành công,
ba CRC-flip và ba partial từ chối riêng trong copied-native oracle, không phải serverauth runtime.

IDA thiếu plugin ida_nexus: giữ cấu hình/nguyên DB, dùng cache/GDB thụ động và Capstone đọc đúng bốn vùng native.
Bốn vùng code so khớp modO/copy4b; wrapper/writer/capture cùng nguồn cần phân biệt với backendpolicy.
Oracle ban đầu dùng nhầm layout/vtable nhận và thiếu sender mode2; lỗi local đã giữ report/script riêng,
sửa theo ctor14EAD00/vtable17B0B7C/+48=2, không sửa runtime. Return nativeframer0 là thành công,7 CRC lỗi,2 partial;
kiểm ban đầu giả định1 đã sửa, không dùng return đơn lẻ làm proof.

Chưa kiểm độ dài khác13, token2-only, provider/password branch, toàn suffix, tokenverify hoặc multiconnection
accountownership; không nâng gate, không triển khai Rust. Audit kếCP216 giữ nguyên.

<a id="cp215"></a>
## CP215 — SSO15/heap16 và token2-only

Baseline CP214/PR287 `83de46c`, cùng copy4b/Wine10.0; [contract](../../PROTOCOL.md#auth-token-bounds),
[metadata/hash](token-bounds.json), [phép thử kế](../../RESEARCH.md#rust-service-gaps).
Raw `local/research/codex/auth-token-bounds-r1/`: source-first13 nguồn/cached legacyrefs,
recipe/preflight/runtime-source, passiveGDB, capture/nativeoracle/report/rawseal/cleanup.
Không đổi server/source/config giữa hai run; mỗi run đã seal và ownedcleanup0 trước run kế.

SSOpair:48.655581411s/58raw/runSHA98510fe70082496ecc342b9fa603fd878b8058440047ca5e1e8bab4e1dce9cbb;
token2-only:47.218443766s/55raw/runSHA04e6813a3a0832fdfe448631ca6032c737c0d5bb4a0b8cb585ea28ef299cadb4.
Input giả local, namespace loopback-only/stub3hello, không authsuccess/Nexon thật.
Tái sử dụng nguyên CP214 probe/harness/copy/writer/framer, bỏ disassembly dump đã có, không quét hàm mới.
Hai actualrecord17/18byte khớp copied-native sender;13 helpervectors chỉ emulation; ba nativeframecontrols riêng.

CP214 giới hạn13byte và token2-only unknown giữ nguyên trong record complete; CP215 bổ sung actual15/16/count0control.
Chưa fullauthentication/token2-onlymanualpath/runtimeupperlimit. Không Rust hoặc servingbehaviorchange,
team sourceWIP nguyên byte/chưa merge. Reviewwaived/selfcheck; audit kếCP216. Trạng thái tích hợp lấy từ PR.

<a id="cp216"></a>
## CP216 — C2S6 producer và authfield36 echo

Baseline CP215/PR288 `d3b85df`; [contract/limits](../../PROTOCOL.md#auth-channel-echo),
[metadata](channel-echo.json), [nexttest](../../RESEARCH.md#rust-service-gaps).
Raw riêng `local/research/codex/auth-channel-binding-r1/`;17source-firstfiles và supplementcacheCP014,
recipe/preflight/runtime-source/probe/rawseal/cleanup cho từng run.
Producerbaseline:51.633969701s/59raw/SHA d11c253b3dfe220784929296d061575425527aad4af1f08e668017f587c242ca;
auxmarker:49.684913572s/59raw/SHA a6fc5faab8a4f72d4ef0fae88d539f1b00a96fcc1a1f6dea1123a59e0f6dee12.
Cùng Wine10.0/copy4b/namespace loopback-only/knownProfile47; stub gửi auth137 đã có parserproof.
Không force nativeauthstate/functioncall, không Nexon hoặc credential thật. Mỗi run sourcefrozen/ownedcleanup0
đã seal trước đổi input, production source/teamWIP nguyênbyte/chưaRust.

Nativeconstructor/send/writer nối producer và cache cũ. Control chỉ đổi authfield36u32+CRC;
reader→setter→getter→serializer→actualwire có marker/caller/timeorder; haiC2S6frames81/body43 chỉ khác4bodybytes.
Hai actualframe/26logicalfields vàCRC/partialcontrol được copied-native kiểm riêng. OldCP058A/Bframes
được đọc lại đúng layout thay giả định8u32 chỉ để so sánh; không coi nguồn cũ đã có đầy đủ schema.

IDAplugin vẫn thiếu; giữ DB/config, dùng cachednative và boundedCapstone/currentcopybytecompare.
Một producer/serializer/callerslice/localgetter pass1; getter/setteraux pass2 vì field36edge cụ thể.
Initialprologuelookup750..92E bỏ sót entry740; mở hẹp5F0..92E tìm thấy. Callerprologuelookupkhôngcó,
chỉ ghi verifiedcallsite11018DB, không bịa functionstart. Wrappernewread trùng một phần CP014cache,
đã bổ sung source-first để tránh RE lại. Không quét providergraph hoặc suy nonce từ fieldname.

Chưa credentialverify/expiry/replay/revoke/swap/sessionownership. Field36echo không nguyênsemantics auth hoặc
credential đủmạnh; backendpolicy sẽ làdesign riêng. AUD063 periodicquaCP216/audit kếCP220; reviewwaived/selfcheck.

<a id="cp217"></a>
## CP217 — native đúng/sai token trên adapter RE

Baseline CP216/PR289 `af16278`; [contract/limits](../../PROTOCOL.md#auth-admission-prototype),
[metadata/policy](admission-prototype.json), [phép thử kế](../../RESEARCH.md#rust-service-gaps).
Raw/source/recipe/preflight/snapshot/privateadapter ở `local/research/codex/auth-admission-r1/`.
Reusecrypt19nativecomparisons thay viết cryptor; purepolicy24checks ghi rõ emu_choice/design.
Policy prototype không thuộc productionserver hoặc Rust; tám sourceWIP hiện có không đổi.

Nativepositive:51.321367412s/59raw/SHA258c5bb3ede0ca55a81b827c65cb7a0e88b53303aab453222693d83fe6d6979a.
Nativebadtoken:50.727851899s/59raw/SHA3e23b793084bc33698a5b84125717e6ba24718a150d27761e280ccfd0702236f.
Cùng Wine10.0/copy4b/loopbackisolated/Profile47/prototypepolicy, chỉ đổi dummy32chartoken đúng→sai.
Cảhai sourcefrozen/ownedcleanup0, seal trước đổi input hoặc chuyển việc.
Input đúng native0/C2S6echo/840470marker+actual67; sai native1/không6-67 trong45s. Không gộp hai run thành
accountresume hoặc lấy server log chứng minh client. Prototype10s chỉ pendinglease, khôngsessionexpiry;
32bitcorrelator/sourceIP/inmemory chưa productionauth. Nativeexpiry/revoke/replay/twoaccountswap chưa chạy.
CP216 đã mergePR289/af16278; AUD063through216/audit kếCP220, không audit lại hoặc nâng gate ởCP217.

<a id="cp218"></a>
## CP218 — nativelease negative và lỗi EOF sau từ chối

Cùng batch giao CP217–218; [contract/limits/blocker](../../PROTOCOL.md#auth-lease-eof-blocker),
[metadata](lease-controls.json). Raw/source/recipe/preflight/declaredscenario/verification:
`local/research/codex/auth-lease-controls-r1/`; source-firstEOF/cache/caller/RTTI ở `auth-channel-eof-r1/`.
Expired:40.758334335s/47raw/SHA28f035ddd8d7b0ae78d55d4f749842579057a5c690acd68f77d2f9a2c336a663;
revoked:40.552730587s/47raw/SHA29d86a85ea6a3629235ba8d49000a60d27d96a523dcf415c6ff0a4137f2a3b89.
Cảhai nguồn/configfrozen, seal/ownedcleanup0 trước chuyển việc; không đổi sourceprototypeCP217.

Policy từ chối đúng pha sau validtoken/native0/actual6echo; client sauEOF lỗiSIGSEGV17C7F4C qua122EA12/12541E0.
Harnessreturn0 vàcleanup0 không che lỗiclient; fullnegative/lifecycle test ghi fail. Không gán lỗi cho leasevalue,
heap/SDK/Wine từ typenameATL CWin32Heap. Staticchỉ exactcall1257360/protectedjump2052077 vàRTTIrecord.
Initiallineardecode122E9C0 rỗng do boundary; exactE8at122EA0D→1257360 mới dùng. Bytes sauprotectedjump
được giữ nhưng không diễn giải; nguyênbinary/DB/config khôngđổi, không thử patch.

CP216 đã mergePR289/af16278/AUD063; recordCP217complete không sửa, CP218 ghi giới hạn/lỗi mới.
Gộp giao haiCP trong mộtPR; next ởPLAN là focusedEOF/callback trace. Không nânggate/fullauth/Rustclaim.
Audit kếCP220 giữ nguyên.

<a id="cp219"></a>
## CP219 — matched/zero EOF control và native dispatch provenance

[Contract/giới hạn](../../PROTOCOL.md#auth-eof-dispatch), [metadata](eof-controls.json),
[phép thử tiếp](../../RESEARCH.md#rust-service-gaps), [thứ tự](../../PLAN.md#current-order).
CP217–218 đã merge [PR290](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/290)/`f9f48d2`;
record complete/raw/audit cũ giữ nguyên. Codex solo/tự kiểm, review waived; audit kế CP220.

| Run | Thay đổi và quan sát | Wall/raw | run.json SHA256 |
|---|---|---|---|
| matched-close | Lease hợp lệ10s, cùng echo191720555 như expired; claim/release rồi close trước17; nativeauth0/actual6 rồi cùng SIGSEGV17C7F4C | 40.019680433s/47 | `b78fd4a8d948b951b6c41104ccf59be0c56a29bcdb258ae178a1d40270e22744` |
| zero-close | Baseline field36=0, không admission policy; intentional close sau6; nativeauth0/actual6 rồi cùng SIGSEGV17C7F4C | 39.941851375s/47 | `84ec751a9f8b2145b176879f70b4088cabc40f43db763a6f98efd9d0964f9b32` |
| zero-stack | Giữ zero service/input; thêm post-send6 entry probe và fault stack0x300; nativeauth0/actual6 rồi cùng SIGSEGV17C7F4C | 39.374265785s/47 | `336aea061f56510ed59955a54f74677e1662003f3cdcfa75355730ed006cefa6` |
| zero-dispatch | Giữ zero service/input; thêm3site dispatch/operand/vtable probe; nativeauth0/actual6 rồi cùng SIGSEGV17C7F4C | 39.127362807s/47 | `8bcb0a39d35da4c8a8e8b7d8ff5aca36c8b3213fa5580b9d830abf92ff83e4a0` |

Raw root theo `cases[].raw_root` trong metadata; mỗi root có `recipe.json`, `runtime-preflight.json`,
`raw-seal.json`, `cycles/client.log`, `service-state.json`, `run.json`, screenshots và ownedcleanup.
Derived verification/script ở `local/research/codex/auth-channel-eof-analysis-r1/` và
`local/research/codex/verify_auth_eof_controls.py`, `verify_auth_eof_dispatch.py`.
Bốn run/188 raw hash kiểm đủ, mọi frozen input vẫn khớp; harnessreturn0/cleanup0 không là health pass.
Matched frame6 trùng expired byte-exact; no17/native67 chỉ trong cửa sổ run kết thúc sớm.
Zero baseline đổi adapter/authport nên không gọi toàn wire/input trùng nhau.

Raw stack/source/three post6 snapshots nối vtable+50 → target17C7F40 → local → call12578AC →
return12578AF → fault17C7F4C; actualthis RTTICObject. Callback snapshots cũ preauth không là prefault,
protected entry1257360 không hit sau6. Receiver selection/rootcause chưa giải, nativelease-negative
lifecycle FAIL; không patch/binary/source production hoặc Rust. Không đóng #14/#285/gate.

<a id="cp220"></a>
## CP220 — receiver trace dừng có blocker; kernelTCP ownership riêng

[Contract/giới hạn](../../PROTOCOL.md#auth-tcp-ownership), [metadata](transport-controls.json),
[câu hỏi](../../RESEARCH.md#rust-service-gaps), [thứ tự](../../PLAN.md#current-order).
CP219 đã merge [PR291](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/291)/`09435d9`.
Bốn run liên quan, không checkpoint từng thao tác; AUD064 periodic qua220, audit kế224.

| Run/root dưới local/research/codex/ | Quan sát/giới hạn | Wall/raw | run.json SHA256 |
|---|---|---|---|
| `auth-channel-receiver-r1/zero-receiver` | Getter4null ở caller khác, store125783B không hit; cùng native fault, không receiver proof | 39.887805350s/47 | `ba4f33ec03124cfac9ed69d850bdda7356f32031920b9f880dd5b19c66111f3f` |
| `auth-channel-receiver-watch-r1/zero-watch` | Watch armed/no snapshot; Wine unhandled17C7F4C, GDB no registers; diagnostic inconclusive | 39.937802774s/47 | `cd90bf9840b19c171ba322666f3682dd28b2d0944b047fc1c03ebb7712a770e4` |
| `auth-tcp-replay-r1/kernel-replay` | TCP driver typed-field error giữ nguyên | 0.112849524s/4 | `d4f3bb1367ae10b917ea72e309eafd3155a138d199dc1af8b874d25bd1e74ce6` |
| `auth-tcp-replay-r1/kernel-replay-r2` | 22 scoped kernelTCP checks qua; duplicate/replay giữ owner, ownerEOF release; không nativeKR | 0.232905161s/4 | `b20612d401981ec41cb2b0ab30deb0a21efb66490e21704e33379daed4a9e730` |

Mỗi root có recipe/preflight/raw-seal/cycles; native có Wine/client/SEH/GDB/rawscreens, kernelTCP có
`wire.json`, `adapter.log`, `service/service-state.json`, `run.json`. Toàn102 raw/sourcefreeze/cleanup
được kiểm; failures giữ nguyên. Reports/verifiers trong local auth-channel-receiver-r1,
auth-channel-receiver-watch-r1, auth-tcp-replay-r1 và verify_auth_receiver*.py/verify_auth_tcp_replay.py.
Không đổi source/config/input giữa run. Bản adapter/policy CP217 và mọi record/audit complete cũ giữ nguyên.

TCP driver dùng captured native login6/67 và fixed public ticket, duplicate6/request67 sequence sửa rõ;
byte-exact captured6 replay trên socket mới bị EOF trong khi owner cũ vẫn nhận reply67.
Không chạy client KR ở ca TCP; không nativehealthydenial/twoactualswap/restartfence/entropy/fullauth proof.
R1 fixedticket decode sai như little-endian; R2 dùng `codec.read_fields` đã có, không đổi wire contract.
No-watch-snapshot không là no-write hoặc supportedwatch proof. Không lặp getter/watch không có observable.
#14/#285 còn mở; không gate/Rust/production behavior change;8WIP byte-exact. Internet thật chưa kiểm.

<a id="cp221"></a>
## CP221 — hai native positive identity riêng và wrong-binding TCP

[Contract/giới hạn](../../PROTOCOL.md#auth-two-account), [metadata](two-account.json),
[câu hỏi](../../RESEARCH.md#rust-service-gaps), [thứ tự](../../PLAN.md#current-order).
CP220 complete/AUD064 giữ nguyên; CP220–221 cùngPR292 cần kiểm exacthead mới. Audit kếCP224.
Raw native ở `local/research/codex/auth-two-account-r1/`, TCP ở `auth-two-account-controls-r1/`.
Mỗi root có recipe/preflight/raw-seal/cycles; native có writer/aux/serializer/auth17markers/actualC2S,
TCP có wire.json/service-state/adapter log. Code/probe/capturedframe/verification local; metadata được commit.

| Run | Wall/raw | run.json SHA256 |
|---|---|---|
| native-A | 50.964269862s/59 | `f43cb2eefc95dfe5adb98e825cf748765aee24def23336cf499565cc50ffeb41` |
| native-B | 49.842327846s/60 | `9cb395e68c353ff8519076782c313fb499be83de463a8663ca55c698a3c844e2` |
| wrong-identity | 0.214679842s/5 | `bd559076f7f0db2052dda5562214fa99ce9de7a3e0084308b30177be151230fd` |
| wrong-source | 0.214523143s/5 | `ef835f1d0f9062e58ee278fa9e16e6db434d693ed35f6e501791c4798f16643a` |
| wrong-identity-r2 | 0.281753195s/5 | `ceb89eda0924a0a9b64a9e1597d64251c60faeb18ae53368711c41a51342c021` |
| wrong-source-r2 | 0.282928595s/5 | `e4a77be1a67528906c811717439b47c78860b1c763c1adc99b71969d1f4d78ea` |

Hai native run119raw: A1/KRLocal, B2/KRGuest,32token/auth0/echo đúng/actual67, sourcefreeze/cleanup0;
chạy riêng trên cùng copy/Wine, không2actual đồng thời. Bốn TCP run20raw: hai R1 failure do assertion
channel lịch sử/active sai giữ nguyên; hai R2 mỗi34scopedchecks dùng active/session/openA/closedB.
WrongID/name hoặc sai source giữ pendingB và ownerA; actualTCP B sau đó vẫn bind, cảhai nhận67,
BEOF không xóa ownerA. Driver synthetic/fixedpublictickets, không nativehealthydenial/swap/securityproof.
Bản CP217 giữ nguyên; derived2accountadapter chỉ chọn known auth_A/B và role từ verifiedaccount.
Không production/Rust integration hoặc gate mới;8WIP giữ nguyên. Internet thật chưa kiểm.

<a id="cp222"></a>
## CP222 — hai actual KR chung service; suffix môi trường và duplicate account

[Contract/giới hạn](../../PROTOCOL.md#auth-two-actual), [metadata](two-actual.json),
[câu hỏi](../../RESEARCH.md#rust-service-gaps), [thứ tự](../../PLAN.md#current-order).
CP220–221 đã merge [PR292](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/292)/`f5caefb`.
Raw/code/probe/recipe ở `local/research/codex/auth-two-actual-r1/`;4run gom một checkpoint, audit kế224.

| Run | Quan sát | Wall/raw | run.json SHA256 |
|---|---|---|---|
| pair-r1 | Validtoken nhưng constant suffix policydenial; không native0 | 81.636530747s/177 | `8af2a15b9e7a2a40b268ace15ceec66eae2d7d3218801c0aa1a7910db0ec9e2c` |
| pair-suffix-r2 | Giữ policydeny; nativewriter/cursor→suffix25 joined | 58.052439451s/122 | `7323c2dca3be914bb897d77bb50bdbe737d551341648d2fcb28ab171318bd29d` |
| pair-r3 | Parserprofile mới;2actual/native0/ID1-2/echo/67/2session+hold | 40.781146896s/100 | `2fa9d958e2330d9b690bbe8b97be9861d4549ea708b8efd720b878846795bd20` |
| pair-duplicate-r4 | TokenB=A; Aowner giữ, Bnative1/no6-67/8s+hold | 40.251220213s/99 | `c6f5cef1bd5d5ccd4386d2942fa536222b5f530d75f5a10f5d4c755f92c40388` |

Mỗi root có recipe/runtime-preflight/raw-seal/cycles. cycles có A/B nativeclient/token/aux/suffixwriterlogs,
CA/wineserver ownership trong virtual-network.json, raw tcp-auth-pair.pcap/index, service-state/run,
proof-service-state[-final].json trước cleanup khi run đủ control, screenshots/process/window capture.
498 raw kiểm hash đủ/sourcefrozen/ownedcleanup0; failedR1/R2 giữ nguyên. Clonedprefix/registry snapshots
có provenance; copy4b và47wirefixture pins giữ nguyên. Profile thử chỉ bỏ replacement_slot, không sửa nguồn.
Không source/config/input thay đổi giữa run. Prefix là runtime state có thể đổi; preflight pin snapshot input.

R2 layoutjoin dùng exactcached13AE3BD/knownwriters/cursor36→52→56→60,existingtyped/blockcodec; không
newcryptor/fullproviderRE. Opaque16 semanticsunknown;0/IP/0 chỉ nhánh đã đo, profile normalize làdesign.
10policychecks tách native. R3 positive2accounts/2actual và R4 duplicate-loginnegative là scope riêng;
R4 hold8s+ không phải45s hoặc channelEOF health. EOFreceiver/fullauth/sessionexpiry/restartfence/DB/Rust
chưa giải; không đóng #14/#285/gate.8sourceWIP byte-exact. Internet thật chưa kiểm.

<a id="cp223"></a>
## CP223 — account đổi endpoint và post17 retry boundary

[Contract/giới hạn](../../PROTOCOL.md#auth-session-lifecycle), [metadata](session-lifecycle.json),
[câu hỏi](../../RESEARCH.md#rust-service-gaps), [thứ tự](../../PLAN.md#current-order).
CP222 đã merge [PR293](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/293)/`4bcabd8`.
Một batch source-guided/milestone+blocker, không checkpoint từng run; audit periodic kếCP224.

| Root dưới local/research/codex/ | Wall/raw/exit | run.json SHA256 |
|---|---|---|
| `auth-endpoint-account-r1/swapped-positive` | 41.043315091s/100/0 | `7a3a2956323670b63c369cf6ccf70159fb7096167f4ac2c29196b83110205b6a` |
| `auth-admitted-lifecycle-r1/B-channel-eof` | 55.946399947s/135/0 | `e7e1aade669498e900e208896f134d0e5823e8b61b9b6fa2b28ff7ca740c7f53` |
| `auth-admitted-lifecycle-r1/B-channel-eof-r2` | 66.000024111s/152/0 | `3146ab48b749c525e666b445c3694b5db9907eaa0e2a1e20ce04bc264d4f8c8b` |
| `auth-admitted-retry-r1/B-retry` | 96.527916168s/224/1 | `7aa319dd26c1d64a871426a99ed86bbe3967adbc63c91f330bf66d1ffff79604` |
| `auth-admitted-retry-r1/B-retry-ack` | 80.958389884s/189/1 | `defc6356e8c5775807610eae1c73605c5ae5645aff10709b38208c3e3d44f321` |
| `auth-retry-connect-r1/connect-path` | 65.751350452s/154/1 | `c26566f7e2e8996ed90fa6375f16d20d26592545cc1857b089e6bc3c8310a578` |
| `auth-retry-connect-r1/connect-path-r2` | 65.610310215s/154/1 | `b5f38bb8d2ebba748bd9b0a4d9ee4522defee6765cba8a2f1fda755167d24cfb` |

Mỗi root có recipe/preflight/raw-seal/cycles; raw A/Bclient logs/probes, window/process/screens,
virtual-network ownership, TCPPCAP/index, service-state/run/proof snapshots. All1108raw/sourcefrozen/
ownedcleanup verified; returned0 observationrun không là healthpass. Code và derived reports ở local:
auth-endpoint-account-r1,auth-admitted-lifecycle-r1,auth-admitted-retry-r1,auth-retry-connect-r1.

Endpointaccountvalidtoken permutation confirmed; notwrongchanneldenial/securityproof. Post17Bclose sau
haiactual67+5s là termination test policy, không TTL gốc. ErrorUI/newconnect có evidence; Aowner retained
chỉ openTCP+policy/nativebaseline, không giả gameplayprogress. Retry chưa secondauth/6/67/newsession.
Prototype4/source và ordinal8 guards được xử lý riêng/failures giữ; không port-retag hoặc force native.
Corrected observer captures newconnect caller +recv15 exactfixture +framerret0/seq40→43/AsyncSelect;
không nâng bytesparse thành consumer/accountresume success.

`observation-correction.json` giữ lỗi đọc ảnh trướcclose và report đầu đã pin; `eof-r2-verification.json`
khẳng định error10/khôngsend81, không coi input create thành room. `auth-retry-connect-r1/verification.json`
giữ probe ordinal sai; `verification-r2.json` ghi actualendpointpredicate/read/framer. Không sửa raw/cũ.
8WIP byte-exact; noRust/productionauth/gateclosure; #14/#285open. Internet thật chưa kiểm.

<a id="cp224"></a>
## CP224 — exact hello/validation returns và fatal retry correction

[Contract/limits](../../PROTOCOL.md#auth-retry-validation), [metadata](retry-validation.json),
[unknown](../../RESEARCH.md#rust-service-gaps), [thứ tự duy nhất](../../PLAN.md#current-order).
CP223 merge [PR294](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/294)/`51bbf41`.

Root `local/research/codex/auth-retry-consumer-r1/`; tái dùng frozenpair/harness/profile/input/serviceCP223,
chỉ thêm observer. Không đổi source/config giữa run; cả ba run có preflight/recipe/raw-seal/cycles.

| Run | Wall/raw/exit | run.json SHA256 |
|---|---|---|
| `consumer-path` | 66.69060825s/154/1 | `4c8a447e73c8df361bbebb52fc8bc9240a01bfae7c250d08af3c2d5bad5b3508` |
| `fixed-returns` | 68.004627887s/154/1 | `fc2b8c9e1280e217c94ec68112a4ef0a8bdc4e15eb3041756372080447f0882d` |
| `continuation-path` | 66.83623407s/154/1 | `0f2de01f0bfd4bac1167043138850c20d014e19eac6345f0fa7227cac2f6a8e3` |

`r1-observer-review.json` giữ giới hạn FinishBreakpointatentry/prologue: returnmarker nhảy sangcaller,
không dùng result đó xác nhận callee. `fixed-returns-verification.json` join Bclose→13AD79C/1255C73/
1255A69/125BE47đềuEAX1/phase2→SIGABRT→Wineillegalinstruction. R2actualnewendpoint3838 khác R1/CP2233848;
không retag stage hoặc lấy boolean watch3848 làm oracle. Không postclose builder/countset/newauth6-67/session.
ẢnhR2Bscreen25 có WineProgramError/blackbackground/LoginError2000:0behind; không gán popup thành cause.

`cp223-fatal-correction.json` pin đúng logCP223 đã cóSIGABRT/illegalinstruction, không sửa report/raw/complete.
Harness cũ chỉ nhậnSIGSEGV và bỏ qua Bfailurepostclose; fullretryFAIL vẫn đúng, failurecausebịbỏsót được ghi
ở mốc mới. 462raw/sourcefreeze/cleanup verified; không production/Rust/fullauth/healthyretry/gateclaim.
Không lặp codec/hello/permutation/failedretry; next nguồndiscriminator11 theoPLAN; AUD065qua224/kế228.

`continuation-verification.json` join fixedEAX1→success125BEBD→handoff1255F80→125BEC9
localEBP-28=11→125BF8Cepilogue/ret; không first/secondguard/local==1path. Source/meaning11unknown;
không token-count/stage/rootcauseclaim. `continuation*.json/asm` là boundedread-onlyCapstone/copy-modelmatch,
không REA/fullfunction. REAIDA timeout450s/export0records, close+ownedheadlessprocesscleanup có receipt,
copyhash giữ nguyên; không cấu hình/install mới hoặc tác động runtimeWine. Run3 cũng154raw/fatal/cleanup.

<a id="cp225"></a>
## CP225 — fresh process positive/negative và prior selector

[Contract/limits](../../PROTOCOL.md#auth-fresh-process), [metadata](fresh-process.json),
[unknown](../../RESEARCH.md#rust-service-gaps), [thứ tự](../../PLAN.md#current-order).
CP224merge [PR295](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/295)/`928f2fb`.

| Root dưới local/research/codex/ | Wall/raw/exit | run.json SHA256 |
|---|---|---|
| `auth-retry-discriminator-r1/getter-path` | 67.75752156s/154/1 | `1b985c8e0526a64201f3a7e5ef8b66d51acf93e287ada9c8fdd282766e1763a7` |
| `auth-fresh-process-r1/fresh-B` | 71.399918226s/181/0 | `fcaec9abd9ae7d23d496419c04423d69667b72f6a28c9070e46fe38b0b98c727` |
| `auth-fresh-process-negative-r1/bad-fresh-B` | 107.895601349s/260/0 | `b2244afe0cf2f11c18a94235094b778ceb0524470f3ce893dd8e31ee281ff782` |

Ba root giữ source-first/preflight/recipe/seal/cycles, source/configfrozenuntilcleanup. Getter-run vẫnfatal,
đo prior11→store trướcsetterrequest1, khôngpretendfix. `historical-fatal-scan.json` chỉliteralmarkers/sáu
CP223post17logs, khôngevidenceabsence-health. Existinggetterbody2×48bytes/knowncallerprefix256+192read-only
samecopy/model, không fullfunction/SDK/VMgraph; source/provenance riêng trongauth-retry-discriminator-r1.

Freshpositive controllercleanupoldB+3s/chưaautoattempt, launchesBfresh/newPID/sameargs; servicepolicy/profile
không đổi. 12TCP/oldlease-sessionreleased/newlease-session/ID2-name-echo6/67/AownerPIDretained/8s+nativehold,
PCAP/2wineserver/2netns/noexternalroute. `verification.json` và `visual-review.json` tách native/wire/UI.
Freshnegative chỉ đổi tokenfresh32wrong, initialABvalid; native1/nochannel/noBlease-session/Aowner45s+,
11TCP. 595raw/hash/sourcefreeze/cleanup verified; harnessreturn không là oracle độc lập.
Không sameprocessresume/accountDB/expiry/revoke/restartfence/production/Rust/gateclaim. AUD065through224/kế228.
