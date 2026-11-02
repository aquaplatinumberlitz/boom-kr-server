# CP074 — Native cleanup trước clock và UI survivor

Baseline CP073/[PR107](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/107) merge
`5c642930ed6c9ac40993055c9a01ca2a819c0778`; KRE-037/[issue108](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/108).
Codex solo/tự kiểm, [review waived](../checkpoints/review-waiver.json); tích hợp chờ PR.
[Contract/giới hạn](../../PROTOCOL.md#preclock-cleanup), [gap](../../RESEARCH.md#gap-disconnect-replacement),
[thứ tự](../../PLAN.md#current-order). Raw/script/probes/captures/seals: `local/research/codex/preclock-cleanup-r1/`.

**Confirmed hẹp:** sau realBEOF trướcclock, nativeA175/180/107/slot1ID2→0/110/chief1, rồi UIphòng hiện
mộtKRLocal/guesttrống/nútStart. NormalBack tạo actual86 và UI sảnh. Không là trận chơi đầy đủ hoặc
rejoin/accountresume/membership tổngquát. Productionservice không xử lý86 để reset/recreate room.

`reuse-question.json` pin CP050/052–054/072EOF/schema/hold99/native175180107/slot/chief, CP073control
attribution/currentduo67positive và CP0041254CC0/errorfallback; KR/VN lobby/session, boom-rs/l10n/archive.
`ui-followup-reuse.json` thêm CP034Back113581/actual86 và ảnh-firstblack: chỉ đo phần thiếu render/input,
không RE lại consumer/binarybody/clock/auth/cache hoặc nguyênproviderNMCO. Currentprofile41basic/
productionguards/forfeit/cleanup unchanged, privatewrapper chỉ giữactualB99≤30s và discard sauEOF.

RuntimeA/B SHA `e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef`, static
`660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3`, modO raw/base400000
`a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5`, RVA=VA−400000. Sourcebinary/
pack/DB giữ; không nativebody mới/clientpatch/provider/globalconfig hoặc force hàm/state. Hai controller
riêng tuần tự/privateuser/net/mount/PID/chỉlo/no route/NSSfiles-only/mộtlease; privategamesCP059/
prefixwarmCP069/display/wineserver riêng. Read-only knownerrorstate/CreateProcess/HTTP probes để
phân loại nếu failure, **không hit trong hai run**, không là proof lỗi lịch sử đã được sửa.

| Run / UTC02/11 | Kết quả | run.json SHA256 |
|---|---|---|
|cleanup-observed /21:26:30|ActualB99hold2434,399ms, nativeA107/IDclear/chief đã thấy; observer10,485s sau servicecleanup. Ảnh ngaylúcstop đen, chưa usableUIproof.|`6637bd0e2e781e4a794b0887048e7f4b18bbdae988b9f95e0d930e63c4ec0e2f`|
|cleanup-ui /21:32:39|Cùngnativecleanup, focus/chờ4/8/12s, UIphòng hiện; actualBack86/seq205/payload0 và UIsảnh; không errors.|`401946b0b2e8c943a8ea9495b62712f93a74ede1906c229fa8afea7d831b175f`|

Hai actualStart/121 đọc129/errors0 mỗi run, actualA99 đã dispatch/B99 giữ, Bprefix kill sauhold+2s.
No197/196/153/nativeclock ở cảhai, không bom/pickup/17/19/21. Productionabortforfeit sauEOF:
A175outer302/inner292/errors0→180full3/stage12→107base6/6/errors0→rewind0/final1/6 intentional.
Native12400D0/12401A9 cùngslot1/expectedID2/actual2→0;110full6 và getterchief1 sauclear. B không nhận
postEOF/result. Không giả forfeit là death hoặc giữ99 là loss/timeout/servergốc. Cảhai serviceerrors[]
và controller không exception; localmodels/framer khác nativegameplay execution.

Firststop có đủ nativecleanup nhưng ảnh đen; không gọi room usable ngay. Một số ảnhperiodic trướcEOF
cũng đen, nên không quy blackframe là doEOF hoặc enginehang. Followup đổi duy nhất observation/UI:
focussurvivor sau nativecleanup, chụp4/8/12s, rồi normalBack113581, chờ5s; không thêm packetreply/
patch. Ảnh ởmốc4/8/12 thấyphòngchỉKRLocal/nútStart/ôBtrống; cùngnative107/IDclear là evidence độc lập.
Không tách được causal effect focus vs wait, không hiệu chỉnh thời gian native từdebuggerwallclock.

NormalBackreturn0 củaOS không đủ: nativeDDA32C→stage11, actualC2S86/emptybody/wire38/seq205/SHA
`03408fac7b88f33fc0819b61e011b60bf2d319d26eebe32f28492962411cd1e2`, nativeframer0/CRCbad7/partial2,
ảnh UIsảnh hiện. Serverchưa explicit86handler; UI này không proof fullprofile/list/recreate/rejoin hoặc
roster đã được service giải phóng. Giữ debtmembership ởRESEARCH, không mở mọi lifecycle variant.

Kiểm first-verification:770relations/291nativeframer/36packet/24secure; ui-verification:783relations,
324nativeframer của followup, 75nativepacket và 48secure snapshots gộphairun. Không cộngbộkiểm trùng
thành bằng chứng độc lập. Pixelhistogram chỉ đo được ảnhđen/nonblack, Codex đã đọc ảnhphòng/sảnh;
sourceSHA tạiui-verification. Heap/receiver/gettercounter models chỉemulation, nativecallbacks thật.
Code/export không đổi, baseline72server/18checkpoint tests reuse; catalog7/docs/artifacts5/5/
requiredlocalhistory/diff kiểm bàn giao. AUD022/CP072, periodicCP076. Gate/fullmatch không tự nâng.

Recipeexact tại`<label>-recipe.json`, allraw`<label>-hashes.json`; run_probe.py label/prefixmới
CA-r2-stage-social.exe300/wine_probe_preclock_cleanup.py hoặc _ui.py/cleanup.gdb. Seal saucomplete,
parsefields/lifecycle/derive/native_packets/verify first, analyze_ui cho cảhairun. Không sửa rawđãseal,
không chạy chồnglease. `first-verification.json` giữ scope native-first; ui-verification thêm render/input,
không sửa CP072 raw hoặc nhận lacktrace cũ là nativefailure. Sourcebinary/DB verified nguyên.

Bước tiếp theoPLAN: currentTCPcaptured-body replay qua socket/framer/currentsequence cho guard124/
118/127. Tái dùng CP049failedproxy scope và CP063/067dispatchcontrol, đổi sang currentauth/cache/
basicprofileCP058/059/069 đã qua67 (CP073duocontrol), trướcinjection phải forward exactbyte. Một
boundedattempt/newcondition/provenance originSHA/bodySHA/seq; replayproxyemitted không nativewriter
mới, không auth/fullvalidator. Không tìm lại writer/consumer/clock. Nativecleanup/knownUI vừa đo không
K4/K5/K6a/fullmatch/LANInternet/Windowsvanilla/toànreset/accountresume/mọiEOFnhiềupeer.
