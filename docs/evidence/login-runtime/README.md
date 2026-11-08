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
