# CP109 — Hai outcome/reset native sau Bpromotion và nguồn issue

KRE-071/[#176](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/176), baseline CP108/
[PR177](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/177) `985a01db1fa2db339d1f4e51008c719ad9188547`.
Codex solo/tự kiểm/[review waived](../checkpoints/review-waiver.json), Snapshot CP109 khi ghi chờPR; đãmerge PR179/8f3ea2f.
[Contract/limits](../../PROTOCOL.md#promoted-outcome), [gap](../../RESEARCH.md#gap-result-return),
[thứ tự duy nhất](../../PLAN.md#current-order). Cùng checkpoint có [issue reconciliation](../issue-reconciliation/README.md).
AUD034 đối chiếu issue qua109; periodic AUD033 qua108, kế112. Không nâng gate.

Raw `local/research/codex/promoted-{capacity-input-r1,capacity-input-r2,terrain-input-r1,trap-outcome-r1}/`.
Mỗi `<root>/cycles-recipe.json` exactargv; `cycles-hashes.json` hoặc `startup-hashes.json` và
trap-outcome`sealed-verification.json` kiểm size/SHA 885+1178+2016+2203=6.282 files.
Không ghép partial/startup thành một run full. Clientcopy SHAe9bb4766…/static660447…/modOa08950…/
base400000/RVA=VA−400000; sourcebinary/pack/DB không đổi. Private user/net/mount/PID/lo-only/
no routes/filesNSS/lease đơn, không đổi source/config giữa run. Không worker.

| Root | Kết quả | SHA256 run.json |
|---|---|---|
|capacity-input-r1|Bconn3 đóng/noC2Sauth/dialogloginerror trướcBack; ownedcontrollerSIGINT/finallycleanup, return130; không testY389.|`startup-verification.json` pin, không dùng như gameplayfailure|
|capacity-input-r2|213,405s; freshAY294→339→399 tại3,130s/keyup3,179s, Y389 qua.5nativeentities/4completed158perview/all195flagsmatch; Up4sno-freshsample timeout.|`capacity-partial-verification.json`37checks|
|terrain-input-r1|364,395s; Up8sR1freshY119after4,245s; R1ending/return, R2ReadyA/StartB. R2Aactual17/native56→53/freshY302→258 chậm, khôngY199.12entities/A12B11callbacks; khôngR2ending.|`trap-transition-verification.json`15joins/30copiedgetters, strict4phase đãpass riêng|
|trap-outcome-r1|382,456695112s; realfreshA17→releaseUp/skip2optionalextras/wait unchangednativeoutcome. Hai result/return/refresh Bhost/sameTCP;12entities/12completed158/view.|`a57afa75028089c3ff484fae3234abc06e16b7b8fcedead1d7f18b43f3fe2b32`|

Source-first CP106route/API/key/target/terrain/TCPguards vàCP107normalBack/Bpromotion/profile47,
CP108freshness/partial158; khiactual17 xuất hiện, reuse CP036known17trap/report19 và CP083all-dead/
existingoptionalrankalias. VN game chỉ hướng dẫn bỏ input khi trapped, không portopcode/stateenum.
Không đọc nativebody mới. Chỉcontroller đổi capacitybound3→6, terrainbound4→8, rồi nhảUp khi
service nhận acceptedA17 mới/cùnground/sauUpstart; ngừng2Spacephụ, chờnative outcome. Không đổi
service/profile/fuse/report/geometry hoặc ép state; existingall-deadalias có sẵn nhưng **không dùng**
trong finalrun: cảhai vòng là `post_blast_untrapped_drained`/B19/A21; A17 localclear/report21 về sau.

Final `peer-verification.json`:500relationships/all4phases/targetchange/currentprogress,324copied
native52A350/40actualCRC. `outcome-integration-verification.json`:132joins, R1IDs1..7/R2IDs8..12,
12actual118/159 và12complete158/view,52map snapshots×195flags/process/boxdelta/resetseed khớp.
10.244nativegetters/10.140array-addresschecks làemulation. BốnBombFirepickups/delta1→2/reset1 ở
secondStart. `transport-verification.json`:232joins,8same/4old11812412717,19 bịloại/noextraIDs/165/170;
frontwriter khácproxyreplay. `promoted-membership-verification.json`:25joins/sameABIDslot/Bchief/
postreturn110B22/A0/actualA95/B104bothrounds/clock2/reset, không suy authority từslot.

Native175/180/585/refresh cảhai vòng; **tại native180** hai view cùng A3/B1 vàXY
R1A(80,164)/B(200,159), R2A(200,317)/B(200,159). Không épY159/entry175equality hoặcglobaltick.
Release/skip2extras dựaexactacceptedfresh17; khôngclaim14entityroute, không sửa originalfuse/resultpolicy.
Existing bothdead branch khônghit, không giảđã kiểm variant đó. 504typedmodelcases tách native.

Giữ transport firstmissingnative-packets.json và inheritedwording14 ở report cũ; chạy actualnative_packets
và script riênggiữnguyênchecks/countsactual12, không sửa raw. Giữtypedprefixanalysisfail report17rawbyte
ởterrainrun vàsửa bằngknownKRtypedreader/script riêng; no field/byteexpectedchange. Originalstrictfullroute
và startup/partialfailures giữ. `visual-review.json`: R1returnedroomAReady/BStart đã xem; finalsecondreturn
hai ảnhđen, **chưa chứngminh render/input responsive**. Đây là nextactualblocker, không mặcđịnhhang.

#176 đượcđóngtheonarrowtwo-native-cycleacceptance; finalUI/gates vẫnở#14/PLAN. NextREreusesCP074focus/
wait4/8/12/currentwindowPID+CP080nativeReadywriter, đo finalsecondreturn không newpackets/stateforcing.
Fullauth/codec/authority/fullvalidator/allreset/vanilla/fullmatch/K5/K6a/LANInternet vẫn mở.


<a id="cp110"></a>
## CP110 — Secondreturn UI/Ready đã kiểm, không nâng toàn integration

Baseline PR179/merge `8f3ea2f987c9ef7428898502294d75415d048828`; CP109/PR179 là lịch sử đãmerge,
record complete giữ nguyên. Codex solo/tự kiểm/waived; Snapshot CP110 khi ghi chờPR; đãmerge PR180/7c5d4ea.
Raw `local/research/codex/return2-visual-r{1,2,3}/`; recipes exact, originalseals1232+1116+2292=4640files.
`r3/return2-verification.json` SHA/runtime/image/phase/actual95/ownerPID; `return2-contact.png` đãxem
thủcông cảhai phòng ởunfocused4/12, sauReadycảhaihiệnready. Contractchỉở [PROTOCOL](../../PROTOCOL.md#return2-visual),
[gapreceiver](../../RESEARCH.md#gap-peer-movement), [thứtự](../../PLAN.md#current-order).

Hai partial vàfailurescript giữnguyên; run3 controller/serviceerror0/457,296629636s. SourcefirstpinCP074/
080/109/KRdata/VNlobby, táidùng mọi site/profile/route. GiảiUI bằngnativePrepareInit/Enable/110/fullcursor
và actualA95seq868/nativewriter→native110cảhai; khônggọiactual95bodyempty hoặc chỉOSreturn0 làReady.
4.686checks gồm4640hashraw; typedpacket/framer/1734copiedsecurefields/flagreader emulation riêng.
StrictfourphaseR1currentprogress failed giữ `return2-target-check.txt`, khôngthay bằng UIpass haygom
runCP1094phase. Bước kế tra đúngreceiver/window/quota/late snapshots trênraw trước thêmruntime.
Service/profile/sourcebinary/pack/DB giữ nguyên; prefix copies riêng, namespace/lease/lo-only/filesNSS.
611feedbackedges Aexplicitrole, khôngrecordAdoBwrite, khônggọi timingchange causalfix. Không gate/fullmatch.


<a id="cp111"></a>
## CP111 — Phân tích lại receiver pre/post, không chạy runtime mới

CP110/PR180 đãmerge `7c5d4eae6aac367049c73851d3b012e1a336aa4a`; baseline này cho CP111.
Rawreceiver/runtime giữnguyênởreturn2-visual-r3; analysismớiở`local/research/codex/receiver-prepost-r1/`.
`reuse-question.json`pin cache/raw/CP098106consumer; `peer-prepost-verification.json`556checks/
4pha/targetchange/currentprogress+freshpostinputs, `peer-getter-verification.json`360native52A350/
`peer-crc-verification.json`49CRC. Không thêm nativebody, gamecall, forcefield hoặc runtime.

[Contract/giới hạn](../../PROTOCOL.md#receiver-prepost) lànguồnduynhất. OldCP110strictbefore-only
vẫnfail đúng statistic của nó; callbackcuốicóafter219 vàfreshsample294 giải missingprogress.
Firstnewanalysis vẫnđòideferredvariant vàfail; `analyze_prepost_initial.py`/failure giữ, finalquestion
chỉđòi4phaseprepostprogress+existingpercallbranch checks, report4snap/0nonzerodeferred, khônggiả
coverageCP106branch. CP110completehistory/evidence unchanged. Snapshot CP111 khi ghi selfcheck/chờPR; đãmerge PR181/248e118. Thứ tự hiện hành chỉPLAN.
