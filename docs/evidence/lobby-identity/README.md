# Identity và consumer panel sảnh — CP182

Owner Codex, tự kiểm; [review waived](../checkpoints/review-waiver.json).
Baseline CP181/PR253 merged `5a154e5`. Contract duy nhất [PROTOCOL](../../PROTOCOL.md#local-name-model),
câu hỏi [RESEARCH](../../RESEARCH.md#gap-lobby-identity-followup), thứ tự [PLAN](../../PLAN.md#current-order).
K3 giữ mở; checkpointcomplete không nghiệm thu UI hoặc fullauth.

## Recipe và bằng chứng

Raw `local/research/codex/k3-identity-cache-r1/`; hai runidentity và bốn runpanel có recipeJSON,
preparation pin input, runJSON, client/service logs, ảnh và raw-seal sau cleanup. Metadata cũ
[in-progress](in-progress.json) giữ nguyên snapshot hai runidentity trước khi nối UI, không là trạng thái mới.
Danh sách raw/size/SHA ở manifest bằng chứng CP182, không sao chép raw vào docs.

Copy Wine `e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef`,
analysisinput `cf72da112308d9f1ca256df8089594c5f787875310933073bf6bd5ba6f70e63b`;
base400000, RVA=VA−400000. Không sửa sourcebinary/DB/pack, không gọi nativegamefunction hoặc ép
record/guard. Kế thừa copydiagnostic/bootstrapglobalreset đã có; không nguyên bảnWindows.
Productionservice/47fixtureprofile không đổi; mỗi run dùng user/net/mount/PID namespace riêng,
loopbackonly/noexternalroute/NSS files-only, prefix/Xvfb riêng, source/config frozen trongrun.
Game/prefix copy kế thừa cache; không gọi là coldcache. Rollback dùng recipeidentityr2 khôngpanelprobe.
Cleanup ownednamespace/process/prefix theo harness trước khi chuyển phép thử, raw/hash lưu ngay.

| Run | Kết quả / giới hạn |
|---|---|
| identity-r1 | SetterID/name vàactual6; FinishBreakpointname không tin cậy, giữ candidate |
| identity-r2 | ExactgetterreturnPC/name/ID vàactual6nativeparse; không UIproof |
| panel-r1 | ActualRoomUIctor; SquareLeftselectedcallbacks khônghit trong60s, không suy classabsence |
| panel-r2 | Read-onlyready snapshot62RTTI candidates; actualembeddedCUserListUIWnd được xác định |
| panel-r3 | C++exception/SIGABRT trướcready, guard chưa đo; không gọi harnessexit0 là thành công |
| panel-r4 | Hoãn probe tớiready, hai actualrenderguard0 và record544byte/nameempty; không causalfixclaim |

VN `lobby.md` giúp phân biệt UserMap list/ownpanel với authmanager; ownlevel/exp phíaVN cũng chưa
kiểm nên không có contract hoàn chỉnh để port. CP058/059/060/181 auth/cache/nativecodec vàCP016
UIdelegation được dùng lại, tránh đo lại auth56field/CRCrepresentation/186getterxrefs.
16UI/storebody mới được so byte với runtime; badSP/None decompile và privateDBloadfailure giữ
raw, dùng assembly/privateDBcopy khác, không sửa cấu hình MCP. `D7A260` chỉ levelchange consumer.

`python3 local/research/codex/k3-identity-cache-r1/verify_panels.py` đã chạy **777 kiểm**:6rawseals,
inputpins/isolation/ownedcleanup/nonoverlap,16nativebodies và guard/SSOraw. Script ghi output một lần;
không chạy lại để ghi đè report. Checker chứng minh hash/khai báo, không ngữ nghĩa/gate.
Các kiểm repo bắt buộc và exactheadPR bổ sung trong bàn giaoGit. Audit AUD054 qua180, kếCP184.

Tại CP182 bước tiếp là producerrecord→dispatch→store→render; CP183 bên dưới đã giải scope đó; fullcache/identityauthority/Windows/LAN/
Internet/8client/fullmatch không được khôi phục bởi checkpoint này.

<a id="cp183"></a>
## CP183 — packet291, control và positive trên panel bản thân

Baseline CP182/PR254 merge`59196f83d5a93a2594295e6c2a583d7e56b607b0`;
[contract291](../../PROTOCOL.md#own-profile-291), [gap](../../RESEARCH.md#gap-lobby-identity-followup).
Raw `local/research/codex/k3-ownrecord-producer-r1/`: source-first,8targetedbody+bytecomparison,
fixture291metadata/2binarycandidate, recipe/preparation, run77files/raw-seal và verification.
Source-first đọc nguồncurrent→boomkr-re→VNlogin/lobby; VNUserMap giúp địnhhướng, KRprofile109
CP018/nativeD2D310 được dùng lại sau khi so readerorderAE6330. Tránh RE lại profile/auth/name/cacheCRC.

Cùng sourcehash/runtimecopye9 như CP182, game/prefixcopy mới kế thừa cache. Namespace loopback-only,
NSSfiles-only, nooutbound; freezeproductionservice/47fixtures. Labsubclass gửi inactive291seq55 sau2s,
positive291seq58 sau2s nữa sau bộreadyfixtures. Đây là schedulethínghiệm, không recoveredpolicy.
Framecontrol39byte/SHA1d85c01bfcae363df3ac8f272c04901d1ef737c3180222b8d75b28036739f7e8;
positive166byte/SHA3f72f432a8c528d9cf34acb0e47d91c01a1a0c84e25776f879381627a6e83cb7.
Rollback về recipeCP182 khônglabinjection; không sửa sourcebinary/DB hoặc ép guard/store/callgame.

Nativecontrol1/1 vàpositive128/128/error0; store/name vàrenderguard0→1 có trace; screen19 đã được
Codex xem, hiệnKRLocal/Lv.0/00.0%. Scalar/level/exp0 làemu_choice, không luật servergốc.
Run60s hoàn tất/cleanup/inputpins/rawhash đúng; `verification.json` **158kiểm/8bodybytes**, manualảnh/
assembly bổ sung, không scriptsuyngữnghĩa. Bằng chứng/hash từngfile ở manifestCP183.

Chưa productionintegration/create/map sau291; initialfixtureseq55..70/dynamic73 phải xử lý khi tíchhợp.
Cold/missing/warm toànmatrix, wrongstage/version/identity và K3–K7/LAN/Internet/8client còn mở.
CP183 chờPR khi viết; reviewwaived/Codexselfcheck. AuditAUD054qua180/kếCP184, khôngreset.

<a id="cp184"></a>
## CP184 — opt-in service, initial sequence và periodic audit

Baseline CP183/PR255 merge`5c1f3fbb216588360fd1bf1ff0be80a5d65c8624`.
[Contract](../../PROTOCOL.md#own-profile-integration), [checklist K3](../../PLAN.md#k3-evidence-review).
Raw `local/research/codex/k3-ownprofile-integration-r1/`: source-first, derived/authname fixtures,
2recipes/preparations, r1 source snapshots, 2run103files/seals, verification. Không RE nativebody mới.
C roomprofile KRLocal/auth KRNew được phát hiện nhờ đối chiếu CP058/native-final, đã sửa binding
ở helper; C126body chỉ model/candidate, không runtime C. Tránh copy blob mù hoặc RE lại codec/profile.

Exporter `--preset two-client-basic-replacement --own-profile-291` giữ47pin; source/config frozen
trong từng run. Dùng copye9/game/prefix mới, inheritedcache, namespace loopback-only/NSSfiles-only.
OSclickcreate286,81/confirm343,479 kế thừa CP016, chỉ chạy khi probe ownpanel đọc guard1.
Không gamefunctioncall/storewrite. Rollback bỏ flag/own_profile policy, giữ recipe cũ/default-off.

r1 đo integration trước guard mới, cleanup/hash rồi snapshot source; r2 đo lại code có guard.
Cả hai A80s/103rawfiles, native291128cursor/errors0, ownname/render và framer43…73→next76 tới map;
ảnh panel-before-create và screen25 đã được Codex xem. Guest KRGuest là synthetic, không B thật.
Framer cursor0 trước dispatch không bodyproof; producer291 là probe khác. Globalnote khiến producer
cùng prefixK3_FRAMER, phân biệt bằng kind. Không sửa raw để chuẩn hóa label.

`verification.json`:392quan hệ, explicit r1 version snapshots, allraw/inputpins/isolation/cleanup,
nativeframe/producer/sequence/OSactions. 290server tests và7exporter tests pass; hai lệnh test sai
module/importpath giữ trong raw rồi chạy discovery đúng, không ghi chúng là codefailure.
Original5artifacts kiểm riêng. Các kiểm repo/exacthead và trạng thái merge ở bàn giao Git.
[AUD055](../checkpoints/audits/AUD055.json) đọc core→evidence/raw, sửa stale source/status/thứ tự;
audit kếCP188. Reviewwaived; CP184 chưa gate/fullmatch, chưa runtime B/C/start/rejoin sau291.
