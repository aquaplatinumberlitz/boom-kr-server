# Evidence khảo sát và triển khai dịch vụ Rust KR00967

KRE-074/[#285](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/285), Codex solo, independent review waived.
[Chỉ đạo](../owner-directives/rust-service.json), [PLAN milestone/acceptance](../../PLAN.md#rust-service-milestones),
[kiến trúc/ma trận hiện có→Rust→thiếu](../../RUST_SERVICE_ARCHITECTURE.md),
[unknown/phép thử](../../RESEARCH.md#rust-service-gaps).

Hiện hành: [contract/limits](../../PROTOCOL.md#rust-m2-acceptance), [thứ tự](../../PLAN.md#current-order);
[owner đã mở implementation](../owner-directives/rust-implementation.json). Phần dưới giữ lịch sử khảo sát CP213.

## CP213 — khảo sát/lập kế hoạch đã giao

Tại CP213, baseline main c0c7b9478dcf5b639a76f67ee51a5f6b927cb0f9/CP212/PR284. CLIlatest xác nhận trước chuyển việc;
K0–K7 scoped đạt, K6c WAN giả lập, **Internet thật chưa kiểm**. Ở mốc CP213 mọi Rust milestone chưa triển khai/nativeRustchưađo.
Code team variants còn WIP, không commit trong planning PR; kết quả sealed trước request giữ ở CP213/eight-player.
Không source/config/input run nào bị sửa; sáu run có rawseal/sourcefreeze/ownedcleanup0 trước khảo sát.

[Survey](survey.json) pin63sourcefiles/5repo commits+archive/wiki và34completedrecords, sáuWIP native runs.
Rawsurvey/sourcecopies/WIPhash ở local/research/codex/rust-service-survey-r1/. Đây là khảo sát read-only:
hash/source snapshot không proof nativeKRsemantic. Wronginitialpath needle_escape.py/rescue_contact.py được sửa
về test_needle_escape.py/test_rescue_contact.py thật; logic actual ởProbeService, không inventmodule.
Không chạy test/serve boom-rs hoặc sửa nguồn/router của repo khác; không lấy các tỷ lệ V1 của nó làm tiến độ KR.

Các quyết định reuse/kiến trúc/ma trận giữ duy nhất trong tài liệu kiến trúc; workorder/acceptance chỉ PLAN.
Không xây tracker/tool mới: existing catalog/docs/checkpoint/issue checkers kiểm link/declaration/hash/history,
không tự kiểm semantic kiến trúc hoặc gate. Nativefutureacceptance cần build/hash/recipe/raw mới trên Rust.
Đợt này không cargo build/test/nativeRust vì không tạo Rust; Python366server/13bundle+5preset kiểm WIP đãđo
trước khi nhận chỉ đạo. Compileplanning không là mục tiêu; full sourceWIP giữ exact hashes và không stage.

Code/delivery scope: planning PR chỉ docs/metadata/manifest; đầy đủ lịch sửCP/audit/raw không rewrite.
CP213 gom milestone RE đã seal và planning request, không CP cho mỗi thao tác; AUD062qua212, audit kếCP216.

Kiểm trước commit phát hiện refreshscript dùng lại tên raw APIcp213 sau khi pin. Đã khôi phục issuepage từ
object trước được giữ và pullpages về metadatarepo.size31340 của response cũ; **toàn bộ byte/fileSHA đúng
expectedhash đã ghi** trước khi thay. Response mới giữ riêng cp213-after-epic-*.json. Không sửa recordcomplete/
evidencemanifest/audit hoặc kết luận; all9233newpins kiểm lại size/hash khớp. Incident/recoverymethod ở
local/research/codex/rust-service-survey-r1/snapshot-recovery.json. Các capture sau dùng tên mới, không ghi đè rawpin.

Rà design trước PR làm rõ idempotency key không gồm policyversion có thể đổi: pin policy vào match/result,
unique(match,player,rewardslot), versionchange không cấp lại; corrections qua ledger cóaudit. Cargo integration
tests đặt trong từng crate, không virtualworkspace root. Đây là hoàn thiện design, chưa implementation.

<a id="cp231"></a>
## CP231 — M0/M1 Rust foundation/native error và retry

[Code/build](../../../rust/README.md), [contract/limits](../../PROTOCOL.md#rust-foundation),
[metadata](foundation.json), [next/acceptance](../../PLAN.md#current-order).
CP230 merge [PR301](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/301)/`115c6b8`;
ownerimplementation directive giữ mới, oldplanning scope không sửa record. Codex solo/selfcheck/reviewwaived.

Raw dưới `local/research/codex/`; không kèm gamebinary/assets trong PR.

| Run | Wall/raw/exit | run.json SHA256 |
|---|---|---|
| `rust-m1-r1/native-error-retry` | 57.930625153s/53/1 | `cdea2ae112b263fa80620fc64c3e80bdbe8cccdc97a7e268f0ba7a3527b49d85` |
| `rust-m1-r2/native-error-retry` | 52.348077562s/63/0 | `816d50a2e99bee8c0acea790bdaeeec474da987716244ceb0c07c2fcffe07424` |
| `rust-m1-r3/native-error-retry` | 60.091387984s/54/0 | `cbddcf23fd6d9d79f98dec6d7d76ad76dd2c34d1f6e3ab47f861372bb64c9d46` |

`rust-m1-r1/source-first.json` trỏ CP057nativevectors/CP180181/nativecopy; `baseline.json` pin exactPython784a0d8/
tar/server-tools/privateCP181adapter/47fixtures/recipe; no8WIP. `native-vector-provenance.json` pin captured6/544 và
retainednative plaintext emulation; Rustfixedvectors không lấy expected từ Rustencoder.
`kernel-tcp-r1`13cases vàr2 14cases giữ rawJSONlogs/config/process; r2 thêm ancillary60/15keepopen regression.
Finalfmt/clippy/test5/build logs ở `rust-m1-r1/final-*.txt`; native-tested binary SHA
`520d7e37ad4c41cc8de47cb3da62841ba35bc849469eac9d06c65fcbf577588f` giữ sau finalbuild.

Mỗi native root có game/prefix copy riêng, recipe/preflight/allsource-binary-config-GDB-fixture pins, cycles/
raw-seal, no source/config/input change trongrun. Wine10.0/e9copy/globalreset1A966FC/privateuser-net-mount-PID/
onlylo/noIPv4route/files-onlyNSS, ownedcleanup/netnsextinct; không vanilla/hardwareWindowsclaim.
R1 `failure-review.json`: server senterror nhưng closeactual60/body20, **khôngnative result1**, laterfatal; raw giữ.
R2 `capture-limitation.json`: native2errors+retry positive, PCAPcap12000 do unrelatedloopbacktraffic/partial,
không completecaptureclaim. R3capture chỉ KRports,108records/selectedstreams đầy đủ. RustbinaryR2/R3 giống nhau.
`rust-m1-delivery-r1/input-retention.json` và R1retention giữ exactfrozeninputbytes khi source/test thay sauseal;
raw/preflight/seal cũ không sửa. Maxframe/wrap test bổ sung sauseal chỉ thay testsource, binarynative không đổi.

R3 `verification.json`: cùngCAPID144, ordinarycommand1002/close+confirm/newTCPseq40,2hello0/0seq40→43 và
2nativeopcode4/result1/mode2/selector3seq43→46/global0,2Rustreply exact39bytes/PCAP/typedC2S joins,9.517s+hold.
`ui-observer-review.json`: screen11 normalcredential-error notice; cuối raw có framerentryTargetdisconnected/
GDBinternal-error tại teardown, không healthdebuggerclaim. Controllerfinalobservations launcher alive/CA/UI trướccleanup;
mode0packetfields ignore, dùngsocketstate. Firststrictverifier error/correction giữ riêng, không che instrumentationlimit.

M0/M1 được đối chiếu thủ công theo PLANscope, không fullcodec/authsuccess/accountresume/sảnh/phòng/game/UDP/DB/
production. Python vẫn serving/RE/reference; rollback choosebefore-run exactPythonpin/cleanupfirst. NextM2,
REonlyimplementationblocker. AUD066qua228/audit kếCP232. Internet thật chưa kiểm.

<a id="cp232"></a>
## CP232 — M2partial local identity/channel/sảnh/291

[Code/run](../../../rust/README.md), [contract/limits](../../PROTOCOL.md#rust-local-lobby),
[metadata](local-lobby.json), [next](../../PLAN.md#current-order). CP231 merge
[PR302](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/302)/`bc55356`.

| Run local/research/codex/ | Wall/raw/exit | run.json SHA256 |
|---|---|---|
| `rust-m2-r1/native-local-lobby` | 0.22990233s/6/1 | `633a845c1d53a9c096059412134e0f8c8526753176bfc100a6e10829d8030992` |
| `rust-m2-r2/native-local-lobby` | 39.999612273s/38/0 | `d62bc2d832a29fba7f1aca07e81d25d8216c5485de7b803725a5aa0f68210254` |
| `rust-m2-r3/native-local-lobby` | 56.41411837s/47/0 | `63f932b11040d5d584b7c95dfee0232079d0e0992bb848cc381fc461cbaa0990` |

91raw/frozen/seal/ownedcleanup0; eachroot source-first/own291input/config/profilepins/GDB/harness/recipe/preflight/
cycles/seal. R1 `failure-review.json`: validator netinfo18 khác knownfixture15, prelisten/no client;
R2 correctiondeclaration/nativepositive trênlocalbuild, R3 finalrootbuild/nativepositive để pin đúng finalartifact.
`verification.json`R2/R3: nativeauth137/137/result0/global0/newchannel17→actual67→24191/14/own291128/errors0/
nameKRLocal/personalrenderguard1; PCAP160records/9replies/alltypedC2S joins,8s+hold; `ui-review.json` xem normalLobby.
FinalR3screen10/KRLocal/level0/EXP0/sixemptycards/9.362070214s hold. RootRustbinary SHA
`6d758286c51025a05acf25b4b4d3ff2ebc4ca119dc8471e74c8af58f03f7dde6`; stagingbinary khácbuildpath/hash giữ riêng.

`rust-m2-delivery-r1/input-retention.json` giữ exactfrozeninputs từngversion trướcdocs/testschange, rawpreflight/seal
không sửa. `rust-m2-r2/final-*.txt`fmt/clippy/test7/build và `m1-regression-tcp/verification.json`14kernelcases;
2newstatetests chỉpolicystage/identity, không nativeoracle. Exporter tạo configpin từexistingmeasuredprofile/ownbody,
không regeneratewireoracle. Wine10/e9copy/globalreset/privatenet-mount-PID/lo-only/noIPv4route/files-onlyNSS.

M2 còn haiactual/explicitbindings/fixedroom/ownslot/Ready/loadingbarrier/Back-rejoin/lifecycle; projector1KRLocal
không authentication/accountownership/DB/fullservice/Battle/UDP/production. Pythonserving/RE/reference/WIP giữ.
AUD067qua232/kế236, nextM2knowncontract/REonlyportblocker. Internet thật chưa kiểm.

<a id="cp233"></a>
## CP233 — hai identity/sảnh Rust, bỏ acceptordinal ở profile hai-client

[Contract/limits](../../PROTOCOL.md#rust-two-lobby), [metadata](two-lobby.json),
[PLAN/next/acceptance](../../PLAN.md#current-order). CP232 merge [PR303](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/303)/`5b3ef81`.

| local/research/codex/ | Wall/raw/exit | run.json SHA256 |
|---|---|---|
| `rust-two-lobby-r1/native-pair` |82.965255701s/193/1|`9fa8c61581086ef4728472af9284d6ed9228de870bdef3c39ac46e7c9a4f3ec9`|
| `rust-two-lobby-r2/native-pair` |40.740784007s/98/0|`b7433b807fbca992bebb660a72b99d843e6a12d869d8ffee240747e85422a70b`|

291raw/frozen/seal/cleanup0. Mỗi root giữ `native-pair/runtime-preflight.json`, recipe/cycles/run/raw-seal và
`input-retention.json`→`frozen-inputs/` exactbytes trước lần sửa tiếp; không sửa raw/preflight để trỏversionmới.
R1failure-review giữ portonlyassumption; R2source-reuse/verification/UIreview ghi knownCP006/103correction và
native auth137/291128/render/actualC2S6ID1-2/PCAP162/18reply/8.710shold. Screen14numericlatest thấy KRLocal/KRGuest;
screen9Bstartupblack không làm finalUIproof. `rea-provider-limits.json` ghi REA thiếuTShark; parserstdlib có coverage/limits riêng.

R2rootbinary SHA`bdf0c70d5a5366fd40d6e071e2f87f7660ab05881799880a6343b28148362d8e`, finalbuild cùnghash;
10Rusttests (5wire+3Flow+2binding), `kernel-tcp/verification.json`16syntheticpolicycases và `m1-regression/verification.json`
14M1cases tách nativeproof. Native là2KR/e9copy/Wine10/privateuser-net-mount-PID/bridge-veth/2netns/noexternalroute/files-onlyNSS,
2CA/2wineserver/cursorfull+render+UI+wirejoin. Khôngclientpatch mới/không SDKRE.

M2 vẫn thiếu fixedroom/ownslot/host/Ready/loadingbarrier/Back-rejoin. Diagnostic IP/name/ID/lease không là accountauth;
Pythonserving/reference/WIP giữ, K0–K7scope cũ khôngđổi, audit kếCP236. Internet thật chưa kiểm.

<a id="cp234"></a>
## CP234 — Rust fixedroom/lifecycle/withheldACK barrier/native spawn

[Contract/limits](../../PROTOCOL.md#rust-fixed-room), [metadata](fixed-room.json), [PLAN](../../PLAN.md#current-order).
CP233 merge [PR304](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/304)/`b10952f`.

| local/research/codex/ | Wall/raw/exit | run.json SHA256 |
|---|---|---|
|`rust-room-r1/native-lifecycle`|42.337080398s/23/1|`dc49eba51708444d173cf286ec72cdc774890e4f15a6121394d3e145ac97461f`|
|`rust-room-r2/native-lifecycle`|51.708281336s/27/1|`cced3cd13d8f4b8a72ba692a0956099145ac077a1255db719c0ce8c8e26e7d03`|
|`rust-room-r3/native-lifecycle`|51.071492933s/25/1|`6331b885d17652deee9f0fbc7078b32e2cb56f75abe375bb29db94de7bbc2861`|
|`rust-room-r4/native-lifecycle`|83.766200353s/37/0|`a24289047e5dd9ac1d8ea9457ded683b9a16a4403f31567e4942d1a915fcb51c`|
|`rust-room-r5/native-lifecycle`|67.229003817s/40/0|`861db8a041f009d6ee9071a0e80494192a900970fdbf1d1afb014b23cb38e55e`|

152raw/frozen/seal/ownedcleanup0; từngroot giữ source-reuse/recipe/preflight/cycles/raw-seal/input-retention/frozen-inputs.
R1loginport mismatch, R2titleinputASCII, R3known96/266missing, R4unported190/JSONmarker20stimeout+unhealthyhold;
`failure-review.json` giữ rõ, khôngsuccess từharness0. R5`verification.json`/UIreview pinpositivepath vàpostholdlimits;
`strict-verification.txt` allclosuresEOFfail giữ, `scoped-verification-output-r2.txt` có đầyđủnative/Wire/FINboundaryproof.

R5nativeauth137/291→create/join/own106204/1106/slots0-1→A86/B107ID1clear/chief→A82rejoin/95Ready/B104→121129,
B99held3.363547654s/A99/no197/noclock→releaseB99→197196153/clock/spawn/UI8.278646918s. PCAP334/56replyframes/
alltuplestreams/C2Sjoins. `ui-review.json`: afterhostBackAatLobby/Bchief vàfinalbothactorownarrow; fixtureoccupancy/capacity
UI khôngacceptancequota. FinalbinarySHA`554fa5b318ab58bfa93bd27933674d8ab73cd739fe586cb89d6515105766a04b` khớpnative.

Postholdownedcleanup: AFIN+0.215898273s/BFIN+0.27270637s sauearlycompletion; roundIdle nênBfinal12714 bịdeny.
Khônggracefulactivepeerabortclaim. R5kernel-tcp timeoutlầnđầu giữlog+raw, chưa rõrootcause; r2/repeat1/repeat2 cùngbinaryđạt.
`final-*.txt`14Rusttests/fmt/clippy/build;8roomTCPcase/16binding/14M1regression táchsynthetic/nativeproof.
M2cònnegative-version/cleanup đối chiếu, khôngBattle/fullservice; audit kếCP236. Internet thật chưa kiểm.

<a id="cp235"></a>
## CP235 — survivor EOF và nghiệm thu M2 scoped

[Contract/limits](../../PROTOCOL.md#rust-m2-acceptance), [checklist](m2-acceptance.json),
[PLAN](../../PLAN.md#current-order). CP234 merge [PR305](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/305)/`f15a2ba`.
Run `local/research/codex/rust-room-eof-r1/native-lifecycle`:70.535651754s/44raw/exit0,
run.json SHA`e93d0e3b2606da253840bac91917264915976a6289b46ca158fba472efa86d57`.
Recipe/preflight/run/PCAP/raw-seal và `input-retention.json`→`frozen-inputs/` giữ source/config/binary/observer
đúng version; cleanup0. Không chạy lại native chỉ vì phiên tiếp bị gián đoạn.

`verification.json`: native137/291/room/slots/Back/chief/rejoin/Ready/121129 và99hold3.360600112s giữ; dừng riêngA,
BsamePID/channel/nativeIDclear/chínnative127→chínwire127drop/noeffect, clock/UI8.230957675s. PCAP345/56exactreplies/
alltuples/allC2Sjoins, BFIN saucompletion. `ui-review.json`:sidebarAclear/timer1:56→1:48/noerrormodal nhưng spriteA
còn trênmap, chưasceneactordeletion/result/forfeit/return. Binary/finalbuild
`ab06ae3f8d00e7baa3146ff503f3d11086df5e31819208025dcf53ca1c18992e`.

`m2-acceptance.json` local giữ manual10criteria có method/limits; bản navigation được commit tại checklist trên.
`profiles/verification.json`6prelistenconfigcases, `shutdown/verification.json`12kernelraces,
`room/verification.json`8TCP, `binding/verification.json`16TCP, `m1/verification.json`14regression;15Rusttests/fmt/clippy/build
ở `final-*.txt`. CP234timeout chưarootcausefix, thêm12racebaseline ở `m2-shutdown-races-r1/` và12race mới đềuđạt;
không sửa oldfailure. M2scopedaccepted khôngfullgame/accountauth/generalroom/fullservice; PLANM3next/auditCP236.
Internet thật chưa kiểm.


<a id="cp236"></a>
## CP236 — M5 backend account/DB trước gameplay hoàn chỉnh

[Contract/giới hạn](../../PROTOCOL.md#rust-account-backend), [metadata](accounts-backend.json),
[CLI/recipe](../../../rust/README.md#account-admin-cli). Baseline CP235 mergePR306/bb673a4; M2native đã giữ binary cũ,
không chạy lại native để chứng minh backend. Nguồn design công ty, không là wire/accountserverKR confirmed.

`local/research/codex/rust-accounts-r1/` giữ 26Rusttests/fmt/clippy/build và binaryCLI `f0180548e88ea8d257bddbb48e91c64d743bda72605f483098e04dd3b16ff810`.
`cli/` failedunknownfield giữ nguồn/binary/rawseal; `cli-r2/` 33process/2account/9audit, strictnegative,
accountidentity/profileCAS/reopen/ownership/ban-reset-revoke/epoch, integrity/FK/plaintextscan vàcleanup0.
Cả hai run frozen source/binary/tool, chỉ sửa sau seal; raw token là synthetic secret nội bộ/private0700,
không commit raw/DB/binary. Checkpoint external manifest pin đầy đủ file/size/hash của hai run và snapshots.

11accounttests gồm twoDBconnections loginrace, resetafterverify-beforecommit, expiry→denial→rollback→reopen,
wrongowner, dummyKDF/PHCbounds/processquota, auditabort/SQLiteFULLpagecap/allocatorrollback, busy250ms,
entropyfault và future/checksum/missinguniqueindex/privatepath. Không suy testmodel hoặc CLI là nativeKRauth.
M5 giữpartial (bridge/lease/worker/idleTTL/rate/restore); M3knownUDPmovement kế tiếp, không phụ thuộc fullauth.
AUD068 periodic qua236; next240. #14/#285OPEN/Python+8WIP giữ; Internet thật chưa kiểm.


<a id="cp237"></a>
## CP237 — Rust native movement hai chiều và floor124→165

[Contract](../../PROTOCOL.md#rust-movement-floor), [metadata](movement-floor.json), [code/run](../../../rust/README.md).
Baseline CP236/PR307/9748dcb, Codexsolo/tựkiểm/reviewwaived. Giữ Python/8WIP; M3partial, K0–K7khôngnângscope.

Ba root `local/research/codex/rust-m3-movement-r1`, `-r2`, `-r3` giữ lần lượt51/55/58raw (164runtime raw tổng),
preflight/retainedinput/frozenbinary/helper/config, recipe vàcleanup0. R1C2S124/19 chưaadapter closesA (127/185đãaccept);
R2floor165nativepositive nhưngB123/10closesB/onephase; R3boundedopaque123policy vàhai movementwindows/6shold đạt.
`movement-verification.json`, `peer-getter-verification.json`, `peer-crc-verification.json`, `ui-review.json`,
`native-oracle-byte-review.json` tại rootR3. R3binarye1ae6aeb…/run2206fd89…; TCP368/58reply, UDP217/87APIjoins,
696copiedgetters/87CRC; mỗi165native12/error0/found17/5E8:5→6. Chuỗi byte không phát hành, chỉ metadata/hash.

`rust-m3-final-tests.txt`/fmt/clippy/build và `rust-m3-floor-final-r2/room/verification.json` giữ30tests/22TCPcases.
`rust-m3-floor-final/room` giữ kernelshutdownfailure/SIGTERM-SIGKILLdeadline/PIDgonepostinspection/không đủPIDstate;
instrumentedrepeatpass khôngrootcausefix. Hai failedunit-test logs giữ để truytestfixture sửa endian/teardown;
không gán là lỗi runtimeparser. REApacketprovider chưa có, stdlibPCAPjoins không là REAEvidence; copiedKRmodO/native
Unicorn/helperbytes có provenance riêng. Không nativeaccountauth/bom/result-reset/fullM3. Audit kếCP240; Internet thật chưa kiểm.


<a id="cp238"></a>
## CP238 — Rusthaiownerbom/serial-ID/deadlinefences

[Contract](../../PROTOCOL.md#rust-bombs), [metadata](bombs.json), [code/run](../../../rust/README.md).
BaselineCP237/PR308/70f00d0. `local/research/codex/rust-bombs-native-r1`, `-r2`, `-r3` giữ44/50/48raw,
preflight/inputretention/frozenbinary/recipe/rawseal/cleanup0. R1Apositive/16732closes;R2bothpositive/1674closesA,
Akeyhold~39s-controllerblocking giữtimestamplimits;R3nosnapshotwhilekeyheld/haiowner/6sholdpass.
Preflight135argumentrejection giữ riêng/trước actualruntime, recipe120s mới. `bomb-verification.json` rootR3 giữ
TCP419/66reply/allC2S/native118→159/entity→158/lookupID1-2kind0cleanup, source/actualtimings/limits;
UIreview riêng. Binary `71986ec657eeb23b13589aba3f2d1661b3e6390caf5eecd16b14193d16cea661` native-tested.

`rust-bombs-r3-tests.txt`/fmt/clippy/build, `rust-bombs-kernel-r4/room/verification.json` giữ32Rusttests/40TCPcases.
Kernelr2 launchedbeforependingbuildcompletion/old167reject/finalhashambiguous; failedresultgiữ, r3-r4pintrướcPopen/
unchangedcheck/awaitbuildpass. No OSdiskfull/auth/gateclaim từkernelmodel. No newbinaryfunctionRE; provenanceknown
Pythonpolicy/wire andKRreadonlycallbacksites. 167unknownopaque;142noreveal;M3terrain/life/result-reset kế tiếp.
Python/8WIP/accountDBgiữ;AUD068qua236/kế240. Internet thật chưa kiểm.


<a id="cp239"></a>
## CP239 — Rustterrain/revealhidden và ba itemeffects

[Contract](../../PROTOCOL.md#rust-terrain), [metadata](terrain.json), [code/run](../../../rust/README.md).
BaselineCP238/PR309/7ecfba3; `local/research/codex/rust-terrain-native-r1` giữpreflight/inputretention/frozenbinary/
recipe/bounded180s/54rawsealed/cleanup0. Actual86.997765664s/oneepoch, không rerunCP238twoownerbombscope.
`terrain-verification.json` giữ410TCP/66reply/allC2Sjoins/195flagsbothviews/clear2cells/current142→three124165effects;
`peer-getter-verification.json` giữ1314copiedgetters/rawwrappers;`ui-review.json` đãxemhai ảnh cuối.

`rust-terrain-tests-r3.txt`/fmt/clippy/build và`rust-terrain-kernel-r1/room/verification.json` giữ35tests/43TCPcases,
partialfullroster158no-clearance/hiddenbefore142nogrant/geometry-reset/duplicate-currentclaim/EOFfences.
CompileLowerHex/privateinterfacefixes cófailedlogs riêng, khôngnativebeforefix; no source/configchangesmidrun.
Pythonkernel reuse CP172/174 là comparisonđộc lậpRust, nativeflags/packet/stat mới là compatibilityproofhẹp;
không allworldauthority/nativechainvariants/native reset/results. Opaque123/167 vàshutdownunknown giữ.
M3tiếplife170/result175/return180/reset/twooutcomes;CP240 phải auditperiodic. Python/8WIP/oldrecords giữ.
Internet thật chưa kiểm.

<a id="cp240"></a>
## CP240 — Rust life/result/reset, hai kết quả đối nghịch

[Contract](../../PROTOCOL.md#rust-life-results), [metadata](life-results.json), [PLAN](../../PLAN.md#current-order).
Raw: `local/research/codex/rust-life-native-r5/native-life/`; `life-verification.json`, copied-getter report,
`input-retention.json` và `native-life/raw-seal.json`. Run SHA `657a48b88409763d3ac6931ecf5095bf11deb07999a15f1288a8365935827ca5`;
66 raw/201.905s/frozen inputs/cleanup0, binary `a4798db20fb0797d523226a70b0b51ee9b4c264db3f1aae1d6448e5dcd0cf229`.
Recipe: `prepare_rust_life_native_r5.py` → `run_rust_life_r5.py` → `verify_rust_life.py`;
Wine/e9/ba namespaces riêng, no default route, router/topology giống CP239 LAN đã pin. Không Internet thật.

Hai vòng có actual17→19/opposite21/170; native hai view loser1/other0 trước175, outer302/inner292/rank2-1 đảo,
1803 và Ready/Start3/clock6s. Kernel44 cases và40 Rust tests riêng. Native peer21clear/all-dead chưa chạy;
M3 vẫn thiếu control trong PLAN. R1/R2/R3/R4:46+46+48+50 raw đã seal trước sửa16716/202/post180585; giữ failures. R5 dùng input CP188 focus12s/heldmouse1s đã đo; budget240s khai trước run cho bốn thao tác return, service/loading deadlines giữ nguyên.
Audit AUD069 qua240 sửa ma trận kiến trúc còn nói account/DB chưa triển khai dù CP236 đã merge;
SQLite/backend CLI đã có, native bridge chưa có. Audit kếCP244, không reset counter. Internet thật chưa kiểm.

<a id="cp241"></a>
## CP241 — Pending-hostEOF→survivor forfeit, nativeRoom et cleanup

[Contract](../../PROTOCOL.md#rust-return-cleanup), [metadata](return-cleanup.json), [PLAN](../../PLAN.md#current-order).
Raw: `local/research/codex/rust-return-native-r3/native-return/`; verification/UI/getter reports,
`input-retention.json` và `native-return/raw-seal.json`. Run SHA `336b72d66941c0eb56863d1eac78dfeedf8805de706b2530dec6f8fbddcd8249`;
48 raw/89.914s/cleanup0, binary `bff20c8890f5415847b7f3a872e1d9cd292f3150f64599c55889e6e00a0beb40`.
Recipe `prepare_rust_return_native_r3.py`→`run_rust_return_r3.py`→`verify_rust_return.py`;
Wine/e9/two private virtualhosts/no default route nhưCP240; BownedPID fault có hash/ns/startticks/env/mappingguards.
NativeA175302-292/rank1-2/1803/107ID2→0/110chief1/Room/4shold/no158. R1startupfailure25raw vàR2knownABIcrash46raw đãseal/cleanup giữ; R3copy4b từexistingCP187strictpatch/newfilename/e9untouched,
BchannelEOF trước67/room/fault; no rootcausefix.41Rusttests/50kernelTCP tách nguồn; M3 vẫnpartial/AUD069next244.
Internet thật chưa kiểm.


<a id="cp242"></a>
## CP242 — M3 scoped acceptance, itemreset/oldUDP/queue và logger supervision

[Acceptance](m3-acceptance.json), [metadata](m3-controls.json), [contract/limits](../../PROTOCOL.md#rust-m3-acceptance).
Baseline CP241/PR312/08a9a0d; native binary `1fc83cef97ae0140f3a07d2926ee7359a01a7119de90f8db2880c40f44c23dba`.
`local/research/codex/rust-m3-controls-native-r5/native-controls/` giữ recipe/preflight/raw-seal/cycles;
runSHA `2972ed26d4ec716593036dd8c6c710f6ec850f2c1fbf573571a470c6395664ff`/84raw/306,382s/frozen119inputs/cleanup0.
Root R5 giữ controls-verification/peer-getter-verification/UIreview/verifier-review/input-retention.
Recipe prepare_rust_m3_controls_r5.py→run_rust_m3_controls_r5.py→verify_rust_m3_controls.py/getters;
copy4b/Wine/privatehosts/no default route như CP241. Verifier sửa biến output shadowing trên cùng raw, không đổi runtime.

R1/R2 route X80 trước blast sai; R3 controller dùng reply cache sai sau long route; R4 logger receiver đóng/maskederror:
51+51+55+60raw giữ nguyên/seal/cleanup. Correct root X120/Y159→blast→FireX80 đã biết CP239, không RE lại.
KernelR2 lỗi flush timeout event141 được bản sửa giữ đúng, 27case trước lỗi/PIDnone; R3 51case pass/cùngbinary/
unchanged/PIDnone; không tăngdeadline hoặc nhận causalrootcause. 44Rusttests; tests/fmt/clippy/build logs suffix controls-r2/r3.
M3 chốt bằng đọc acceptance và đối chiếu CP237–242, không từ exitcode/script. OS contest chỉ một actual124/index17/epoch;
hai claimant negative thuộc kernel. Hai outcome đối nghịch CP240, activehostEOF CP241, không duplicate claim.
M5backendCP236 đã có; tiếp worker/idleTTL/rate độc lập rồiM4 trong PLAN. AUD069qua240/next244; Python/8WIP giữ nguyên.
Internet thật chưa kiểm.


<a id="cp243"></a>
## CP243 — bounded account worker/handoff/compensation; M5 partial

[Contract](../../PROTOCOL.md#rust-account-worker), [metadata](account-worker.json), [API/run](../../../rust/README.md#account-worker-nội-bộ--cp243).
CP242/PR313/merge da1b925; tái dùng CP236Store/schema1/Argon2/clock/audit. Không RE mới, không đổi diagnostic KR transports.
`local/research/codex/rust-account-worker-acceptance-r1/` giữ preflight/source-testbinary frozen/processlog/5SQLiteDBs,
verification.json/retained-db-review.json. Recipe `python3 tools/test_rust_account_worker.py --out <new-private-dir>`;
5worker tests source-frozen/processgone và independent SQLite integrity/FK/schema/active/audit check.
FaultDB active1 do compensation auditabort rollback, workerclosed/rootConflict; không tuyên bố thu hồi thành công.
Unpolled successful oneshot vẫn phải claim, expiry/droppedcaller revoke exacttoken/audit, stalecleanup không hủy newergrant.
Raw local/private có synthetic test credentials/DB; chỉ metadata/hash commit, không upload raw/token/PHC.

49workspace tests/lockedbuild/fmt/clippy; logs `rust-account-worker-*-final.txt` và clippy-r3.txt.
Clippy-r1 collapsible-if failure giữ, r2/r3pass trước acceptance. Testpanic được chủ động inject vào riêng workerthread,
không là native client crash. BackendCLI/absoluteTTL CP236 giữ; chưa idleTTL/rate/KRauthlease/bridge/sessionresume/restore/load.
Không chạy lại M3positive để tăngCP: baseline nativeCP242 vẫn pin riêng, binarybuildCP243 không tự chứng minh native compatibility.
PLAN tiếp idleTTL/rate M5 rồiM4; audit kếCP244/không reset. Python8WIP giữ nguyên. Internet thật chưa kiểm.


<a id="cp244"></a>
## CP244 — schema2 idleTTL/rate backend và AUD070; M5 vẫn partial

[Contract](../../PROTOCOL.md#rust-account-idle-rate), [metadata](account-idle-rate.json), [PLAN](../../PLAN.md#current-order).
BaselineCP243/PR314/mergeddc71e33. 0001hash/schema1 history giữ; 002transaction validateoldschema-first/checksum/version,
nullableidleoldrows; CLIinit schema2. Backenddesign, không KRwire/newnative RE hoặc nativeauth/servingintegration.
`local/research/codex/rust-idle-rate-worker-acceptance-r1/`:7worker cases/7schema2DBs/54pinnedfiles/sourcefrozen/PIDgone.
`rust-idle-migration-acceptance-r1/`:5sealedCP243DB copies/10CLIopens/109files, independent hashes/counts mọi oldbusiness
rows/schema1→2/repeatopen/FK/integrity/oldidleNULL; originalbaselineunchanged. `rust-idle-rate-cli-acceptance-r2/`
giữ33CLIprocess/2account/9audit, regressionabsoluteTTL. R1cũngpass nhưngpreflightchưapin002riêng; giữR1, thêm002pintrướcR2. Migrationraw/filehash/report và testbinary frozen local/private.
55Rusttests/fmt/clippy/lockedbuild; logs rust-idle-rate-*-final.txt. Initialcompilefail missingidleparameter và oldtestissuearg
giữclippy-r1/r2; sửa trước runs. Unit002SQLiteFULL rollback dùngmax_page_count, khôngOSdiskfull.
AUD070 đối chiếu README→PLAN→PROTOCOL→RESEARCH→evidence/code/raw CP241–244; nextCP248/không reset.
Sửa currentPLAN/RESEARCH staleM3partial wording, không sửa completecheckpoint/audit/raw cũ. M3CP242nativebaseline giữ;
M5backendidle/rate đạt subset, bridge/ownership/worker-to-KR chưa đủ. TiếpM4four/eight, không broadSDK/duplicatepositive.
Internet thật chưa kiểm.


<a id="cp245"></a>
## CP245 — bốn native startup/room/loading/currentprogress; M4 partial

[Contract/giới hạn](../../PROTOCOL.md#rust-four-startup), [metadata](four-startup.json), [PLAN](../../PLAN.md#current-order).
Baseline PR315/e601ca7; source reuse tại `local/research/codex/rust-m4-source-survey-r1/reuse.json`.
`rust-four-startup-native-r3/`: four-verification/peer-getter-verification/ui-review/input-retention.json,
native-four/runtime-preflight/recipe/raw-seal.json và cycles A–D/rawPCAP/native logs/ảnh. R3Manner1, khôngFree0.
64raw/105,413713755s/runSHA4d0aed96…/binarybab87fed…/cleanup0;698TCP/112exactreply/16transports;
12remote currentprogress/4self/7.128copiedgetters, kv-a UDP sáu cạnh A. Đã xem ảnh cả4 tên/actor/arrow/clock01:33.
`rust-four-startup-native-r1/`:39raw/exit1/cleanup0/5448 bị từ chối; failure-review/channel-input-review/
cp200-ancillary-reuse.json giữ đối chiếu CP200. R2:43raw/intentional130/finallycleanup/controller sai tọa độ,
failure-review/controller-stop-request giữ; R3 dùng known605,526. Không patch mới, không freshprefix claim.
Old verification/getter copies trước mode review giữ; final verifier đọc exact retained pre-run inputs,
publicexporter nâng sau run không sửa raw. Hai script verify_rust_four_startup.py/verify_rust_four_getters.py local.
`rust-four-startup-public-kernel-r1/`:7syntheticfour cases/sourcefrozen/currentbinary; export-r1 generatedinputs
không oracle/native acceptance. `rust-four-startup-two-regression-r3/`:51twoTCP/currentbinarypass, R2oldbinary51pass;
R1EOF/cleanupfailure giữPIDobservations/D-stateBtrfs, chưa causalrootcause. Logs fmt/clippy/tests/build-final giữ58tests.
Không fourbom/life/result/reset/EOF/tám/fullauthgateclosure; startup4 chặn unportedBattle. Tám PythonWIP byte-exact,
AUD070 qua244/audit kế248. Internet thật chưa kiểm.


<a id="cp246"></a>
## CP246 — four-owner bom/native159-158; M4 life/result/reset còn mở

[Contract](../../PROTOCOL.md#rust-four-owner-bombs), [metadata](four-owner-bombs.json), [PLAN](../../PLAN.md#current-order).
BaselineCP245/PR316/mergedd0c2ba3; source-first rust-four-bombs-source-r1/reuse.json + CP201142/167CRCseqshapes.
`local/research/codex/rust-four-bombs-native-r2/`:bomb-verification/bomb-getter-verification/ui-review.json,
four-verification.json là scoped startup subcheck riêng; native-four/runtime-preflight/recipe/raw-seal/cycles.
84raw/139,844435953s/sourcefrozen/cleanup0/runSHAeb35624f…;native16entity159/full39/16full158/lookup-kind0-cleanup,
902TCP/144exactreply/16transports/Manner1/knownstats4views/copy4b/Wine10.0-20260425/7.196copiedgetters riêng.
Ảnh own/peer trap-death khác nhau vì127 chưaforward; không sharedlife/resultacceptance. No165/170/175/180.
R1:73raw/150,766527131s/exit1/cleanup0/runSHAf37b0544…;actual16722 làm4channelclose, controller tiếp screenshotCtimeout,
failure-review giữ causalgiới hạn. CP20112old +4new16722 CRC/seqkhớp; fixfour-onlyopaque22/noeffect/budget,
controller kiểm closure giữaownerinputs; không patch mới/tăngdeadline/freshprefixclaim hoặc sửa raw cũ.
`rust-four-bombs-kernel-r2/`:14fourTCP/sourcefrozen/PIDgone; R1 có13cases trước167fix giữ.
`rust-four-bombs-two-regression-r2/`:51life/reset/EOF/sourcefrozen/fullsource-inputpins/PIDgone/samefinalbinary.
R1wrongoldroomharness/rankconfig:41cases rồi chờ immediate107 sai delayedforfeit/timeout, sourcefrozen/cleanup giữ;
failure-review giải thích sửa harness, không serverfix hoặc Btrfscausalclaim.59Rusttests/fmt/clippy/lockedbuild logs giữ. Baactualprocessprebindreject:fourflagontwo/fourrank/fourterrain, raw tại rust-four-bombs-config-negatives-r1/.
M4 còn170ba recipient/clear21/rank4/outcomes-reset/EOF→8; M5backendnativebridge riêng, #14/#285OPEN.
AUD070qua244/audit kế248/historyimmutable. Internet thật chưa kiểm.


<a id="cp247"></a>
## CP247 — life17/19 tới ba peer/native bốn state đồng thuận; M4 còn rank/reset

[Contract](../../PROTOCOL.md#rust-four-life), [metadata](four-life.json), [PLAN](../../PLAN.md#current-order).
BaselineCP246/PR317/merged1b3bddf. Source reuse tại `local/research/codex/rust-four-life-source-r1/reuse.json`.
`rust-four-life-native-r1/`:life-verification/life-getter-verification/ui-review.json, native-four/preflight/recipe/raw-seal/cycles.
76raw/134,675276088s/exit0/frozen/cleanup0/runSHAf6910b4a…/binary9fb52288…;874TCP/140reply/16transports/7.240copiedgetters.
ActualA/C17-19→12recipient170/native56→1/freshall4states[1,0,1,0];Bốn ảnh đã xem/clock01:06. No175180/rank/reset/EOFacceptance.
`four-verification.json` là scopedstartupsubcheck, `life-verification.json` là proof thêm; copiedgetters riêngUnicorn.
VerifierR1 log rust-four-life-native-r1-verification.txt giữ failure vì một D127 sauEOF khôngtypedlife;
oldverifier `verify_rust_four_life_before_cleanup_review.py` giữ. R2verifier phân biệt explicitboundedabortdrop saucurrentEOF,
không sửa raw, không healthyfourEOFclaim; nativeR1 không cần chạy lại.
KernelR1:17behaviorcases rồi cleanupdeadline/PID885548 chưa reap; post-cleanup-review xác nhậnPIDgone/chưarõcause.
TwoTCPR1/51pass có timinglimit vì mở trước xác nhậncleanup4; giữreports, không acceptance cuối.
KernelR2 thêm PID/thread/wchan/17pass/exit0/PIDgone; two-regressionR2 chạy sau đó51pass/fullsourcepins/samebinary/PIDgone.
Ba prebindcases ở rust-four-life-config-negatives-r1.60Rusttests/fmt/clippy/lockedbuild; clear21all3completion chỉunit/TCP.
Không tăngdeadline/fixclaim/shutdownrootcause hoặc nativeclear21/rescue/needle. Tám PythonWIP giữ nguyên;
#14/#285OPEN/M4partial/M5bridge riêng; AUD070qua244/audit kế248. Internet thật chưa kiểm.


<a id="cp248"></a>
## CP248 — same4opposite2v2/native175180/reset/Start3; AUD071

[Contract](../../PROTOCOL.md#rust-four-results), [metadata](four-results.json), [PLAN](../../PLAN.md#current-order).
BaselineCP247/PR318/mergedd6cbf9e. `local/research/codex/rust-four-results-source-r1/reuse.json` + cp202-rank-oracles.json
giữ oldnativeaccepted1758frames/2bodyhashes/layout; constructorhashregression riêng currentnativeconsumerproof.
`rust-four-results-native-r1/`:results-verification/results-getter-verification/ui-review.json,
native-four/runtime-preflight/recipe/raw-seal/cycles.84raw/236,896244281s/exit0/frozen/cleanup0/runSHAdf28e911…,
binary8503e27e…/currentManner1/fixed2v2/same16transports/1691TCP280reply/48copiedgetters riêngUnicorn.
Nativeall4pre175states[1,0,1,0]→[0,1,0,1]/ranks2-1-2-1→1-2-1-2/full562outer552inner16slots/1803,
actualReadyStart2-3/keys12346–48/currentStart3freshstates0/6shold. Budget300s hai fullcycles, I/Odeadlines giữ.
UIreview xem12ảnh: B/C/Dscoretwooppositevectors; A2rankshotsstillBattle nên không AUIpanelproof/timingcause.
All4Start3ảnhclock01:53/actorsnames; nativeA175180-rankmemory/reset proof riêng, không pixelidentity.
VerifierR1 lỗi elapsed_ns ởmetadataevent, oldverifier `verify_rust_four_results_before_elapsed_fix.py`/loggiữ;
R2phânvùngtimestampedevents/cùngrawpass, retainedpre-run verifierhash không sửa. No native rerun.
`rust-four-results-kernel-r1/`:24fourTCP/sourcefrozen/cleanup0/PIDgone; `rust-four-results-two-regression-r1/`:51twoTCP
tuần tự/fullsourcepins/samebinary/PIDgone.4prebindcases config-negatives-r1;62Rusttests/fmt/clippy/lockedbuild.
AUD071 periodicqua248 đối chiếu README→PLAN→PROTOCOL→RESEARCH→evidence/code/raw CP245–248;next252/khôngreset.
Sửa currentrankpending/legacyBattlecfgambiguity, táchnativeall4parserproof vớiAUIcapturegap; giữoldcomplete/audits/raw/failures.
M4 cònunrankedhostguestabortEOF→roomcleanup/promotion rồi8;nativeclear21/rescue/needle/terrain-inventory4 chưa đủ.
M5backendpresent/nativebridge riêng/#14/#285OPEN/8WIPbyteexact;shutdownrootcausechưa rõ; Internet thật chưa kiểm.


<a id="cp249"></a>
## CP249 — scoped bốn người guest/host EOF, abort không xếp hạng và storage control

[Contract](../../PROTOCOL.md#rust-four-abort), [metadata](four-abort.json), [PLAN](../../PLAN.md#current-order).
CP248 merge PR319/b58abf8; `rust-four-abort-source-r1/reuse.json` và integration-baseline.json giữ đối chiếu CP205,
committedPython/known180107110/chief; không import WIP/VNwire hoặc tìm thêm hàm native. Newbinary7b1e2338…,
copy4b/Wine10.0-20260425/currentManner1/private4virtualhosts/sharedkernel/no default route.

`local/research/codex/rust-four-abort-{guest,host}-native-r1/` giữ abort-verification/ui-review/input-retention,
native-four/{runtime-preflight,recipe,raw-seal,storage-export,cycles}. Mỗi run đúng ownedpendingPIDloss,
3native180full3/3setup585/3native107IDclear/9wire110/3Roomscreens/chief/pastfuse/no158175. Metadata pin riêng
runSHA/binary/rawcount/duration/TCP/cleanup; verifiers dùng fulltuple-C2Sseq-CRC-SHA PCAP joins, không logserver-only.
Native evidence sink explicitprivate256MiBtmpfs; sau ownedcleanup/captureclose copy-size-SHAverify/unmount/rename
về cùng localpath, storage-export.json pin toàn raw. Không sửa raw/preflight cũ hoặc tăngdeadline.

65workspace/20guestTCP/20hostTCP/6prebind; `rust-four-abort-tmpfs-control-r1/` giữ32normalfour/82exportedfiles,
`rust-four-abort-two-tmpfs-control-r1/` giữ51two/85exportedfiles. Chỉ temporarytree thuộc run bị xóa sau export/hash,
lexicallocalpaths còn nguyên/real directory, không mất evidence qua symlink. Đây là storage control, không nativeoracle.
Giữ clippyR1 typecomplexity/nestedif lỗi và before-source; R2 sửa cấu trúc trước mọi run/65tests/lockedbuild.

NormalkernelR1 pendingD158 đúngfuse cắt vàoC17quiet; failure-review/source trướcdrain/raw giữ, explicitdrain trước
independentlifecases sửa harness. R2logflush timeout event292 và R3event208/exit1/PIDgone/sourcefrozen giữ nguyên.
R2PID-thread snapshot có reserve_bytes.cold; chưa đủ causaldiagnosis hoặc Btrfsfix. Storage controls/nativetmpfs
không chứng minh durablelog/production. CP247shutdownunknown/history/AUD071next252 giữ. M4 tiếp8/M5bridge riêng;
không AUIrankcapture/newnormalcycles/reconnect/fullgame/auth/gateclosure. Internet thật chưa kiểm.


<a id="cp250"></a>
## CP250 — Rust8baseline Manner4v4/mesh-statbom-life/opposite-rankreset-Start3

[Contract](../../PROTOCOL.md#rust-eight-baseline), [metadata](eight-baseline.json), [PLAN](../../PLAN.md#current-order).
CP249mergePR320/9fdb57b. `local/research/codex/rust-eight-source-r1/` giữreuse/opaque-reuse/rank-oracles/seams,
committedPython dc71e33 snapshots/CP208–210; không currenteightWIPimport/VNwire/newfunctionRE.
`rust-eight-native-r5/` giữ eight-verification/eight-getter-verification/ui-review/collateral-review/verifier-corrections,
native-eight/{runtime-preflight,recipe,raw-seal,storage-export,cycles}, frozen-inputs/input-retention.
RunSHAd3ace521…/binary9105bc65…/4b/Wine10.0-20260425/Manner1/private8netns+root/32TCP/newprocesseswarmcopies,
156raw/384,236496429s/exit0/frozen-cleanup0/5019TCP912reply/fulltuple-C2Sseq-CRC-SHAjoins/53616copiedgetters riêng.
8actual99/heldH3s/8native121285-153205;56remoteprogress/knownstats/all8owner118/64native159-158;
R1actualdead1-3-4-5-7/Dcollateral70peer170, R2dead2-4-6-8/56peer170. All8pre175states/rankmemory/full1082-1072/1803,
64refresh/7Ready2–3/Start3key12348/freshall8state0hold6s. UI24images3contact sheets all8boards/twooppositevectors/Start3;
Aboard proof ở8run, không backfill oldfourimages. No fullM4/teamvariants/admission9/native8EOF/generalrooms/auth.

69Rusttests/fmt-clippy-lockedbuild; `rust-eight-kernel-tmpfs-r4/`16synthetic8cases/export93files/PIDgone,
config-guards-r1/5prebind. TCPR1missingloginredirect,R2inbox8vs10startup,R3ledgerroster2/4missed8 giữ failures/source,
 sửa scopedknown8guards/newdirectories; no deadlineincrease. Hồi quy2/4/pins/raw giữ riêngtheometa.
NativeR1prearg540>480exit2/no namespace/pre-namespace-seal;R2wrongephemeralports/capture4IP/baseline4,
controlledstop67raw125,840s/exit130/cleanup0 giữ. R3Hpost17EOF67raw63,358s vàR4Gpost17EOF67raw81,339s,
Hcopycontrolinconclusive;37byte17 bằngCP210nativeaccepted. R5roles dùng exactprior8argv/newprocesses/original8copies,
không causalSDKfix/authclaim. Verifier initialonlylosersdead/oldtag/slotshadow lỗi giữ cảsources/logs/retainedpre-runhash;
R4 đúngactualdeathset/7recipient/nativeall8state/rankbyteam/mesh56, raw/preflight không sửa/nativekhôngrerun.
`eight-controls-verification.json` kiểm8auth/own291-render/roster/nativeownslots/3round8ACK-beforeGO.
UDPcapturer cònfourIPassert/threadfail khiE, sealedPCAP/index chỉ3frameA→B/C/D; `udp-capture-limit.json` giữ rõ.
Không14/56UDPwirecapture/nativeAPI-keyapplyclaim;56nativecurrentprogress/fullTCPcapture độc lập. Sửa trong plannedvariantcontrol.
Allnativepositive/private256MiBtmpfs evidence sink/export-sizeSHA/unmount/renameaftercleanup. Btrfsflushfaults còn
unknown, TMPFS không durabilityfix/load/production. AUD071qua248/next252/#14-285OPEN/8WIPexact.
Internet thật chưa kiểm.


<a id="cp251"></a>
## CP251 — fixed1v7 port, native8 Mannernegative; Free startup blocker giữ nguyên

[Contract/giới hạn](../../PROTOCOL.md#rust-eight-teams), [metadata/hash](eight-teams.json), [PLAN](../../PLAN.md#current-order).
Baseline `83abe4b`/PR322 giao nguyên8Pythonteamfiles từ CP213; Rust binary SHA`521012fa…` dùng cho mọi run bên dưới.
Nguồn/source-first và bodyhash CP213 ở `local/research/codex/rust-teams-source-r1/`, `rust-team-rank-oracles.json`;
không phân tích hàm native mới hoặc port VN wire.

- `rust-eight-teams-manner-negative-r1/`: 90raw/127,113194352s/runSHA`33acefb4972b7a8773849f87b3da66a030627cd8505386b74ff99f23e8309d29`,
  32TCP/1.513records/266reply, actual8/copy8d/Manner1v7/nativecount8-cap8-2teams/balancebranch/return7/no104,
  3shold/cleanup0. `negative-verification.json` + `negative-controls.json` giữ independentraw/native joins.
- `rust-eight-teams-free-r1/`: copy4bD post17beforeRoom,66raw/52,509084929s/runSHA`d21dba1b11d8f2a3fddba412cc15d376ba6647f6d01f8f24791bc38885ff4aff`,exit1/cleanup0.
- `rust-eight-teams-free-guard-r2/`: alternate8dG post17beforeRoom,67raw/60,841184141s/runSHA`e8d37eb8aefa8765d8472c73a322a9d03a9c84b3dedda6abccbd7539ab9dca42`,exit1/cleanup0.
  Không nhận Freegameplay hoặc causalcopyrepair; không retry khi chưa có observable mới. OldCP250HGunknown giữ riêng.

Recipe mới `prepare_rust_team_native.py`/`prepare_rust_team_negative.py`/`prepare_rust_team_free_guard.py` tạo thư mục
mới, source/config/preflight frozen, known8prefixes/newprocesses/private8netns+bridge/nooutside/Wine10.0-20260425.
Run→ownedcleanup→TMPFSexportallsizeSHA→unmount→`seal_rust_m1_r3.py`; mọi run seal trước đổi config/source.
Nativecopy4b và8d SHA đầy đủ ở metadata/inputpins; không clientpatch mới. Mode/count/startreturn dùng probe CP147162163
đã có. File freezer/failedinput/verifier source đều giữ ở mỗi run, không sửa complete/runtime cũ.

71Rusttests/35eightTCP/8prebind/32fourTCP, fmt/clippy/lockedbuild đạt. Kernel raw ở
`rust-eight-teams-kernel-{free,manner,negative}-r2/`, storage/exporthash giữ riêng; R1test UnboundLocalError/93raw
và guardchecker wrongexpectedreason giữ ở source-first/failure reports. UDP supervision fakefaultcontrol đạt,
negative actualcapturer không lỗi nhưng khôngBattleUDP; chưa có14Aedges/full56wire/nativeAPIkeyapplyproof mới.
AUD071qua248/audit kếCP252; reviewwaived/Codexselfcheck/PRexacthead, không gateclosure. Internet thật chưa kiểm.


## AUD072 — correction argv sau CP251

[Metadata/SHA/source sites](argv-provenance.json), [contract/giới hạn](../../PROTOCOL.md#argv-provenance-correction).
Audit đối chiếu source launch và frozenconfig/run.role_args CP250R3/R4/R5, ba run CP251, cùng actualCP210.
Claim32perrole không đúng: actualtoken13, scalar32 không tới launch; không sửa complete/audit/raw cũ hoặc bỏ nativeproof.
Đây là seriouscontradiction audit riêng, periodic kếCP252 vẫn giữ.


<a id="cp252"></a>
## CP252 — native visitor thứ chín, argv correction và Freecontrol vẫn bị chặn

[Contract/giới hạn](../../PROTOCOL.md#rust-ninth-admission), [metadata/SHA](ninth-admission.json), [PLAN](../../PLAN.md#current-order).
Baseline PR323/`9b37b14`; binary SHA`4d21781a…` dùng cho native9 và Freeactual32control.

- `local/research/codex/rust-ninth-native-r2/`:19raw/35,407594284s/runSHA`290775e86b5b04bb2a67679a134b57d5dc3eebfe616dad81103b41c5a34fd755`,
  20TCP/1.384records/249exactreply, oneactualvisitor9/eightTCPsynthetics/actual82/native87generic2/ordinaryReturn/
  handler_end1of1-errors0/hold3s/no585/Roomgrant/cleanup0. `ninth-verification.json`, actual-CA-argv và hai ảnh được xem giữ nguồn riêng.
- `rust-ninth-native-r1/`:17raw/121,273s/runSHA`a5d4d308fd9acd896ae395a41243df25b822d616bdc596cdda1ea59b6743d474`,exit1:
  nativeerror2branch đã đọc1/1/errors0, nhưng controller không đóng notice nên synthetic120sidle; không gọi toàn runpass.
- `rust-eight-free-argv32-r1/`:66raw/48,888173952s/runSHA`69f87cd77f84234a8a7a8e1e44f9576beffd8bb63ab8853d552233dd2b634c8f`,exit1/cleanup0.
  All8actualPopenlist32, independentCAargv A–G bao gồmfailedD; Hchưa observer ghi. Dpost17 trướcRoom, không Freegameplay/causalfix.

Recipe `prepare_rust_ninth_native_r2.py`→preflight→run→ownedcleanup→TMPFSexportallsizeSHA→seal; một Winehostảo vàroot8loopbacks,
nooutside/sharedkernel/copy4b/Wine10.0-20260425/cachedprefix/newprocess. Native errorprobe từ CP227, không hàm mới/patch mới.
Freecontrol dùng `prepare_rust_eight_free_argv32.py`, cùng copy/profile, currentbinaryninthflagoff; actual argv được assert trước
Popen và đối chiếu procCA riêng. Source/config giữ nguyên tới seal. Không dùng scalar CLI làm actualargv nữa.

6TCP/4prebind/73Rusttests/fmt-clippy-lockedbuild đạt. TCPR1 loaderalias9 sai,94raw giữ/PIDgone; R2 chỉ đòi tám playerfixtures.
NativeR2 verifier đầu reject blanketnonEOF, reportfail giữ; bản mới phân loại7Brokenpipe trong ownedcleanup bằng actual9channel
FIN/RST saucompletion, không xóa lỗi hoặc rerun client. Script/version/raw/inputretention đều được pin.
AUD072 correction giữ riêng; AUD073periodicquaCP252/audit kếCP256, không reset cadence. M4OPEN vìFreepositive thiếu;
M5worker-to-KR/ownershipbridge tiếp độc lập. Internet thật chưa kiểm.

<a id="cp253"></a>
## CP253 — account admission library, chưa listener/native M5

[Contract](../../PROTOCOL.md#rust-account-bridge), [metadata](account-bridge.json), [thứ tự](../../PLAN.md#current-order).
Baseline PR324/e17b78b; reuse nguồn ở `local/research/codex/rust-m5-bridge-source-r1/reuse.json` và
`oracle.json`: CP217 complete pins giữ nguyên, expected token SHA/name-ID-echo từ native writer/chain.
Không gọi native client mới, không RE binary hoặc thay runtime diagnostic.

```bash
python3 tools/test_rust_account_bridge.py \
  --out local/research/codex/rust-account-bridge-acceptance-r2 \
  --oracle-manifest local/research/codex/rust-m5-bridge-source-r1/oracle.json
```

R2:7ca thư viện/6DB/12account-player,3body replay,65file raw/15,069s/sourcefrozen/PIDgone;
DB integrity/FK/schema2 và không plaintext password đạt. Output example chỉ digest/identity/correlator,
không xuất token/password. 80workspace tests/fmt/clippy/lockedbuild tách native proof.
R1 pass nhưng chưa constrain bool optional-string, giữ source/raw; R2 thêm guard/negative trước run mới.
Preliminary compile errors và queuecancel test Busy giữ log; barrier sửa giả định drain trong test.
Không tăng queue/timeout, không đổi source/input trong frozen run. Full checkpoint pin cả hai run/failure.

Next: listener/profilewriter/launcher integration và native ownership/expiry/revoke/lifecycle theo PLAN;
M5 vẫn partial. Backend64 không là native32, echo32 không strongcredential, tokenAPI chưa exposed.
K6c WAN giả lập; **Internet thật chưa kiểm**.

<a id="cp254"></a>
## CP254 — native account lobby hai profile cố định

[Contract/limits](../../PROTOCOL.md#rust-account-serving), [metadata](account-serving.json), [PLAN](../../PLAN.md#current-order).
Rust `account_admission` dùng DB/private Unix socket và hai known zero-profile, session64→frontend32 riêng, không password trênTCP.
82Rust/9library8DB3retainednativebody/16TCP/11startupguards; nativeR5 có haiKR/Wine/copy4b/137-291-render/argv/18exactTCP.
Raw ở `local/research/codex/rust-account-native-r5/native-pair`, 100file/41,852s/cleanup0 và native-verification/UIreview ởparent.
PrivateTMPFS DB/log/raw/exporthash, khôngBtrfsdurabilityfix. NativeR2/R3/R4fail và preparationfail giữ nguồn/raw.
`tools/test_rust_account_serving.py --binary <kr-service> --admin <kr-admin> --config <two-profile> --out <new-private-dir>`
kiểm synthetic TCP, khôngnativeoracle. Accountmode reject room/battle/unknownprofile; native negative/lifecycle/fullM5 còn thiếu.
Python/diagnostic rollback giữ. Internet thật chưa kiểm.

<a id="cp255"></a>
## CP255 — native account wrong-correlator/revoke/absolute expiry

[Contract](../../PROTOCOL.md#rust-account-native-lifecycle), [metadata/limits](account-native-lifecycle.json), [PLAN](../../PLAN.md#current-order).
SameCP254server/4b, no newserver source; GDB/ordinaryCLI controllers/sourcepins ở `local/research/codex/rust-account-swap-source-r1`
và frozen-inputs từngrun. Swap78raw41,686s/13reply; revoke79raw44,268s/18reply; expire123raw70,445s/18reply, tấtcảseal/cleanup0.
`negative-verification.json` ở `rust-account-swap-r1`; `lifecycle-verification.json` và UIreview ở
`rust-account-revoke-native-r1`, `rust-account-expire-native-r1`. FullPCAP/native cursors/control/argv/SQLite/audit/FIN và6UIảnh kiểm.
Sourcehashbinary/knownABI reused, không mở thêmbinarybody/SDK; REAsessionavailabilitymetadata retained, không là nativeEvidence.
A notice10/boundednoterminal, chưa ordinaryReturn/modalclose/retry/freshlogin; Bpending→ownclaim hoặc activeB giữ≥8s đúng từngcase.
TMPFSDB-log-raw/exporthash khôngdurability/SLO/Btrfsfix; M5partial/nextperPLAN, auditCP256. Internet thật chưa kiểm.

<a id="cp256"></a>
### CP256 — fresh login/new backend-frontend và old native replay denied

[Contract/giới hạn](../../PROTOCOL.md#rust-account-fresh-login), [metadata](account-fresh-login.json), [PLAN](../../PLAN.md#current-order).
`local/research/codex/rust-account-fresh-r2/fresh-verification.json` và `ui-review.json`; raw tại `native-pair/cycles`,
preflight/seal/storage-export riêng.122raw70,794s/28exactTCP/cleanup0;A mới/B lobby sau oldnative1-6 replay đã xem.
R1ClockRollback/68raw42,544s/cleanup0 và preparation syntaxfailure giữ lại. Clock fix/84Rust/fmt/clippy/lockedbuild,
source-first/REA session metadata ở `rust-account-fresh-source-r1`; không gọi shell-native proof là REA Evidence ID.
AUD074 qua256/audit kế260; M5 OPEN, tiếp account-owned fixedroom/basic theo chỉ đạo mới. Internet thật chưa kiểm.

<a id="cp258"></a>
### CP258 — account-owned fixed2/basic end-to-end và lifecycle/fresh rejoin

[Contract](../../PROTOCOL.md#rust-account-basic), [metadata/hash/limits](account-basic.json), [PLAN](../../PLAN.md#current-order).
`local/research/codex/rust-account-basic-native-r3/life-verification.json`, `ownership-verification-r2.json`,
`peer-getter-verification.json`, `ui-review.json` đối chiếu normal2oppositeoutcomes/reset/Start3/ID-profile-channel-slot.
Ba `rust-account-basic-{revoke,expire,disconnect}-native-r1/return-verification.json` và UIreview chứng minh survivorRoom/no late158;
affectedB revoke/expire cóWinefatal/SessionError2000, khônggracefulrecovery.
`rust-account-basic-fresh-native-r1/fresh-verification.json`/getter/UI: newBprocess-session-frontend/channel/rejoin/newkey,
exactoldUDPnativekeydrop/noapply/currentBmovementprogress. Native5runs/280raw/375exactTCP, sourcefrozen/seal/cleanup0.
Mỗiroot có `native-account-basic` recipe/preflight/raw-seal/storage-export, raw `cycles`; copiedgetters/nativememory riêng.
R1normalfailure27raw vàR2preparationfailure/source preserved; freshcheckerSyntaxError/newcheckerfix khôngnative rerun.
Sourcefirst ở `rust-account-basic-source-r1/reuse.json`, scripts/inputhash/raw giữlocal.84Rust/59accountbasicTCP/16lobbyregression.
AUD076 audit nhómfixed2 theo policy mới; M5OPEN/generalprofile/nativeDBqueuefault/recoverylimits; M6/M7/M8 riêng.
Wine/sharedhostvirtualLAN/privateTMPFS/exporthash, khôngdurability/strongUDPauth/Internetclaim. Internet thật chưa kiểm.
