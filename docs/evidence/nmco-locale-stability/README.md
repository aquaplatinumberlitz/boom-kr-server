# CP073 — Tách lỗi launcher khỏi đường lỗi main

Baseline CP072/[PR105](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/105) merge
`690d9559fbbccf93e5697107ffe273d32bf7182d`; KRE-036/[issue106](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/106).
Codex solo/tự kiểm, [review waived](../checkpoints/review-waiver.json), tích hợp chờ PR.
[Contract/giới hạn](../../PROTOCOL.md#launcher-error-attribution), [gap](../../RESEARCH.md#gap-auth-identity),
[thứ tự](../../PLAN.md#current-order). Raw/script/probe/copy ở `local/research/codex/nmco-locale-stability-r1/`.

**Phát hiện:** popup2000:0 tái hiện ở auxiliary CALauncher không có gameTCP; chưa có bằng chứng lỗi
locale là nguyên nhân mainSIGABRT. Đã sửa hướng “stack96A15A là locale caller” của CP072. Chưa sửa
client/service/provider hay gọi force success. Startup main historicalcause và native107preclock còn mở.

`reuse-question.json` pin CP006–020 NMCOidentity/imports/guard/copy, CP064failure, CP072raw positive/
negative, KR/VN login/protocol/session, boom-rs/l10n/archive; followup reuse CP0041254CC0/error/fallback
1258231. Tái dùng packet/service/GDB/isolation/nativeframer; không tìm lại auth137/135/cache/clock.
`96A155→96A480` qua JSON/HTTP `contentsUrl`, URL `/placements/%d/services/%s/contents?memberType=%s&memberSN=%s`,
không phải NMCOlocale wrapper đã chứng minh. Stack chỉ định nhánh đang fatal, không tự định danh cause
của popup đồng thời. `A90B90/A90C00` tìm exportNMCO_SetLocaleAndRegion; configured main probes không hit
ở các control hiện tại, chỉ là absence trong window. Không quét toàn client hoặc sửa IDA/provider/config.

Runtime mainA/B SHA `e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef`,
static `660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3`, modO raw/base400000
`a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5`, RVA=VA−400000. Exactranges của
main gồm96A107/96A250/96A480/96E300/A90B90-C84 khớp A/B/static/modO. Fragments96A100/A90B80 ban đầu
bắt giữa instruction, tiền tố giải sai bị loại; 96E300 là continuation, không prologue. Không lấy decode
sai làm claim hoặc sửa raw. Mở targeted thứ hai theo cạnh cross-artifact, không phân tích toàn provider.

Auxiliary giữ đúng bytes clientnguồn và hai privategamecopy, có provenance trong auxiliary-identities:

| File | SHA256 |
|---|---|
|CALauncher.exe|`8cc2668bec7a4a0281f1acab26ab3487c4b4e5047d7ce75ab32b299ef16f527d`|
|nmcogame.dll|`25746a9a0e29deb7abb70ed1889c01072eb7343d1cafb4a9ffc76b69fac3ae9d`|
|nmco3.dll|`7e038696ec69db579f71e029da66f5b5500c56375ff77b24d93529ba7faf9493`|

Chuỗi UTF16 `nmcogame.dll SetLocaleAndRegion Error(%d:%d)` thuộc CALauncher VA4FBF08/ref449439,
importlocaleIAT4FA630. nmcogame exportRVA1D140: điều kiệnID/code invalid ghi1000/0; valid thì gọi
internalinit1C6F0, trả trị và errorpointers. Mainwrapper không resolve export ghi3000. Các số này là
nhánh tĩnh riêng, không suy lỗi2000 là invalidlocale hoặc missingDLL. Toàn provider semantics chưa RE.

| Run / UTC02/11 | Phạm vi | run.json SHA256 |
|---|---|---|
|locale-observe /18:16:15|actual67/currentprofile_positive, targetbranch_notobserved|`7281aea89d843d7f92485d0f74ad231ec6d85efaaba78b324bbb3c76bb15bb94`|
|content-observe /21:11:19|actual67/currentprofile_positive, targetbranch_notobserved|`f713bdcaf40f009e2c0de9327361bfd7044124587d6ab169b10e641e074c8a71`|
|content-duo /21:13:07|actual67/currentprofile_positive, targetbranch_notobserved|`2a7c5a6837489494fcbb1e5fe96fd4d1fa806aaea3a42d8751003be5ac888860`|
|channel-close /21:15:18|controlledEOF→nativefallbacklauncher/emptycmd→mainSIGSEGV, nothistoricalcauseproof|`abc374495450514d67d3a26110aeabdc9c1c3cf55e3d3191ed329a4e0e7b567e`|
|launcher-direct /21:16:20|measurement_setup_failure_ASLR, notnativeexception|`b9d883045d25294dee49b537e6bc673637f2207f0c9395e291855327e70e536c`|
|launcher-relocated /21:17:24|actualauxlocalevalid1/0_theninitfalse2000:0/samepopup,noTCP|`92df2666bbbee91f60cb389770557447f4e9933301e81b0f69e4ea87d873a421`|
|launcher-emptyargs /21:18:57|actualauxlocalevalid1/0_theninitfalse2000:0/samepopup,noTCP|`e10719dd0b59dd17ea3326392abeec22077438f00c48ea531b93c239a5359ba1`|

Hai A-only control90s và duo actual67+5s: 4/4/8 actualconnections/currentprofile41, không lỗi native/
service/probe; không hit locale/HTTP/CreateProcess markers trong window. Không cóStart/gameplay/gate.
Không coi hai client/prefix là cause vì cùng điều kiện đã có cả pass/fail lịch sử.

Negative `channel-close` là **design input**: wrapper giữ bytes actualrequest6/hash rồi gọi production
close thật trước17. Native main hit CreateProcessA return1258231, app`CALauncher.exe`, commandline rỗng,
flags20, sau đó SIGSEGV17C7F4C. Không HTTP/localewrapper marker; đây là control fallback do wireEOF,
**không tái hiện historical SIGABRT96A** hay chứng minh CP072 đã mất TCP vì cùng nguyên nhân. Accept
thứ5 được service gán roleB theo policy, controller không spawn clientB; không gọi đó là B thật/login.

Standalone auxiliary `launcher-relocated` và `launcher-emptyargs`: nmcogame thực loadbase7A540000,
MZ+6exportbytes đã kiểm trước đặt breakpoint RVA1D140. Nativeentry return49959A nhận `(1,0)`, validation1,
internalinit trả0, mainerror2000/sub0; đúng popup đã đọc ảnh, **zero gameTCP connections**. Genericrecipe
ban đầu còn truyền haiCAargs multiserveridx/token dù metadata nói noargs; đã ghi correction và đo thêm
`launcher-emptyargs` với exactclient_args[] tương ứng commandline rỗng của fallback. Cảhai cho cùng
entry/return/error, không dùng argv sai để tự nhận matchedcontrol. Không gọi NMCO export bằng debugger.

`launcher-direct` đầu dùng preferredbase10000000 sai vì ASLR thực7A540000: không insert đượcbp,
continue bị abort. MarkerLAUNCHER_NATIVE_STOP sau commandfailure không nativeexception/proofexport;
raw/seal vẫn giữ. Retry đổi resolverbase theo loadmetadata trong cùng debuggerlog, kiểm MZ/exportbytes
rồi cộngRVA, không sửaDLL/globalconfig hoặc đoán địa chỉ. Main runtime vẫn useknownbase400000.

`verification.json`: 1169 quan hệ/hash/seals/source, 423 nativeactualframer,
3 nativepacket cases. Receiver/preallocatedbuffer/heap models chỉemulation; exportreturn/error là native
runtime read-only. Bảy run tuần tự/private user/net/mount/PID namespace, lo/no route/files-onlyNSS/
mộtlease, privategameCP059/warmCP069prefix/display/wineserver riêng. Không patch/force hàm/state,
không sourcebinary/pack/DB hoặc servicecodechange; negativewrapper chỉ local measurement. Baseline72
server/18checkpoint tests reuseunchangedcode; catalog7/docs/artifacts5/5/localhistory/diff kiểm bàn giao.
AuditAUD022/CP072 giữ, periodic kếCP076. Không auth/vanilla/fullmatch/K5/K6a/LAN/Internet/stabilityfix.

Tái tạo bằng label/prefix mới, exactargv/hash `<label>-recipe.json`/`<label>-hashes.json`; main dùng
run_probe.py CA-r2-stage-social.exe90/_nmco_locale.py hoặc _nmco_content.py, duo180/_nmco_duo.py,
negative90/_nmco_channel_close.py/content.gdb. Auxiliary30s CALauncher.exe/_nmco_launcher.py;
**đổi logpath resolver theo label**, dùng run_launcher_empty.py để bỏCAargs, không genericrecipe rồi
nhận là emptyargv. Seal saucomplete, derive_roles/replay/native_packets/verify từrepo root; không
chạy probe preferredVA đã thất bại. Không overwrite raw hoặc chạy chồng lease. Code/raw chi tiết giữlocal.

Kết luận bị sửa: CP072 popup2000:0 không đủ để ưu tiên “fix locale trong main”; CP073 tách auxiliary
providerinitfail với mainHTTPfatal/causeunknown. Currentduo đã qua67, tiếp exactpreclockEOF native107/
slot1ID2→0/chief observer≤30s, đồng thời trace known1254CC0 errorstate/CreateProcess và HTTP nếu
startupfail để giữ đúng cause. Chưa retry TCPproxy/chain hoặc bypasslocale/abort; không mở provider
sâu hơn khi nó chưa chặn mainpositivecontrols. Khi có failure mới, dùng actualerror/caller/input để mở
đúng nhánh; sendercleanup vẫn không là native107proof.
