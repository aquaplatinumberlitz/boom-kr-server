# Evidence khảo sát và triển khai dịch vụ Rust KR00967

KRE-074/[#285](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/285), Codex solo, independent review waived.
[Chỉ đạo](../owner-directives/rust-service.json), [PLAN milestone/acceptance](../../PLAN.md#rust-service-milestones),
[kiến trúc/ma trận hiện có→Rust→thiếu](../../RUST_SERVICE_ARCHITECTURE.md),
[unknown/phép thử](../../RESEARCH.md#rust-service-gaps).

Hiện hành: [contract/limits](../../PROTOCOL.md#rust-two-lobby), [thứ tự](../../PLAN.md#current-order);
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
