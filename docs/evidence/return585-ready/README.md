# CP079 — 585 không cần106 mới trong ca đã biết; Ready2 chưa vào callback

Baseline CP078/PR117 merge `fff30b1f42abd4855a095c0f7502326801565354`;
KRE-042/[issue118](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/118), **acceptance hai vòng chưa đạt**.
Codex solo/tự kiểm, [review waived](../checkpoints/review-waiver.json), chờ PR.
[Contract/giới hạn](../../PROTOCOL.md#return585-ready), [câu hỏi](../../RESEARCH.md#gap-diagnostic-service),
[thứ tự](../../PLAN.md#current-order). Raw/cache/script/seal ở `local/research/codex/return585-ready-r1/`.

Nguồn đúng câu hỏi: CP018106/ownslot/110pending, CP023/039return, CP069successful2cycles,
CP078partial585/Ready, cachedCP028DDE760/DDE900/Readyprobes, CP016UI/input. `reuse-question.json`/
`source-comparison.json` pin thêm KR/VNlobby/session/boom-rs/l10n/archive. Không nativebody mới,
không quét lại codec/writer/auth/cache/clock. Runtime/static/modO IDs giữ hiện hành; 11 điểmprobe/
16byte khớp baartifact trong`probe-code-match.json`, RVA=VA−400000/rawmodObase400000.

**Counterexample tái dùng trực tiếp, kr_runtime/confirmed hẹp:** CP069 không có106 mới saureturn585,
nhưng actualB95(+13,611s saurefresh)/A104(+23,082s) và vòng2 thànhcông. B585 lúc−1,124s trước
refresh. CP078B585+10,382s/A585+10,919s, inputReady cốđịnh+6,424s trước585; noB95. Vì vậy thiếu106
không là điều kiện cần trong profileca đã đo, chưa cócơsởwrapper106 hoặc persistentservicefix.
Khác biệt inputtiming là candidate, chưa độc lập causalproof; không suy mọi585variant khôngcầnreply.
HashCP069 `ba1d167a169b5fd95d1ad1e8a003b1b5d60c3082cbbebdfd98939ba753556f14`,
CP078 `6684e01e22f5778cabf7357f2a8c0e55548e189f9281efd42ec84c84b6f01105`.

**Native trace mới chỉ trong khoảng thu được:** `ready-anchor` dùng service/profileCP077giữ;
actual585phasescan trongcontroller trướcReady2, đúngstatesA/B_env/focus/chờ12s/capture, khôngsidecar.
Có một R1 sauguestBack/rejoin/Ready/Start, nativeclock/2bom/3pickup165/175/180 mỗi view1lần.
R1đủRoller17/Bomb27/Fire28 theoactual124, không mới stat/pointerjoinproof ởđợt này. Lastsnapshot
round1/key12345/refresh, A/Bpostreturn585, khôngnew106/noB95/A104 hoặcsecondkey.
NativeReadycallback đầu: pending0/arg1/localcheck1/count1/host0/throttle/builddesired1. Sau180native
PREPARE_ENABLE_REUSE arg1/pending0 vàảnhBReadyUI đúng hai profile; **không callbackReady2 trong
log thu được**. Không kết luận guard bêntrongReady từchối khi entrychưađượcquan sát.

**Run bị ngắt, không phải finalizedrun:** toolsession87789 kết thúc mã143, khôngrun.json cuối,
service-statelast SHA `891c53dd700e9a3e37db94a724f24be4d3a6514df62a5f9990d0683ded9da131`; laststate.errors[] và
native khôngstop/errorfallback marker trongwindow không là finalerrorfreeproof. Nguyênnhân143
unknown, không gán tooltimeout/clientcrash/usercancel. Quan sát sauexit khôngprocessmatching còn.
`interrupted-seal.json` khóa toànrawhiệncó/recipe/launchlog, không ghép service/log thànhrun.jsongốc.
Finalactionreturncode/roleenv/time/phase và metadataisolationPID cuối không ghi được; **unknown**.
ẢnhpreReadyB vàcontrollerrecipe không thay bằng chứngOSclickhoànthành. Không nhận phaseanchor+
focusdelay làđãchứngminhtimingcause; no2ndcallback cóthể nằmtrướccallback cần đo tiếp.

Recipeprivateuser/net/mount/PID/chỉlo/no route/NSSfiles-only/lease nằm source/argv/hostfiles;
finalruntimeisolationmanifest thiếu nên giữ giới hạn, không giảfinalcontrollerrecord. Sourcebinary/
pack/DB giữ, khôngnativeforcing/newpatch/service/newfixture/106wrapper hoặcsemanticreplay.
RuntimeSHA `e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef`;
static `660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3`;
modO `a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5` raw/base400000.

Kiểm2518relations: exactinterruptedrawseal/codeidentity/currentseqCRCframeassembly/service/source/
nativecontext/imagehash/cachedcounterexample. **Không chạy nativeframer model từrun.jsontựdựng**;
collector dùngservercodec cho bytejoin, knownnativeprobes/callbacktrace/sourcecomparison lànguồnkhác.
Plannedtwo-cycle chưađủ điều kiện chạyverifierfinal/đónggate; verification-interrupted pass chỉcho
scope cóghi. Catalog7/docs/artifacts5/5/localhistory/diff; production83server/18checkpointtests reuse
unchangedcodeCP077. AUD023/CP076, periodicCP080; raw/historyCP001–078 giữ.

Bước tiếpPLAN: reuseCP016WndProc1259C00/125C250/filter,button1509376/1509525/delegates vàCP028
Ready, ghi đồngthời initial vsreturnedclick trước callback. Persist progress/rolePID/display/focus/action
trước/sau từngUI vàlúcphase mới, không chỉcuốirun; giới hạn controller dưới wrapperbudget, không
lặp550s chỉđểchờ95. Nếu khôngnewedge, chuyểncurrenthaiownerbom theoPLAN. Chưa causefix/fullmatch/
K5/K6a/LANInternet/Windowsvanilla/toànauth/codec/validator/reset/hostlivepeer/activeleave/accountresume.
