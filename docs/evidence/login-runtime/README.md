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
