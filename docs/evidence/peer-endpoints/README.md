# CP138 — IP quảng bá153, UDPsource và peer lookup trên loopbackalias

[Contract/giới hạn](../../PROTOCOL.md#peer-endpoint-ip), [câu hỏi](../../RESEARCH.md#gap-peer-relay),
[thứ tự duy nhất](../../PLAN.md#current-order). Baseline CP137/PR208 merge`15270fb`; Codex tự kiểm,
[review waived](../checkpoints/review-waiver.json). AUD042 qua136/kế140. Gate/issue14 giữ mở.

Nguồn reuse-question của hai thư mục pin path/SHA: CP027153/99/endpoint, CP045cached13DE880/
14E57B0/queue, CP098/106nativepeer-observer và CP111prepost; KRoldkr-data chỉTCP/asset, VNoc_e-game
§5a92/146 chỉ dẫn P2P/LANIP, không port opcode/layout. Không đọc thân hàm mới hoặc RE lại codec.
Currentproduction/profile giữ nguyên; private shim sửa đúng remoteIP/port khi phát153, re-encode bằng
seq hiện tại. Không UDPpayloadrewrite/actorfake/stateforce hoặc gọi nativegamefunction.

| Run local/research/codex/ | Kết quả | run.json SHA256 |
|---|---|---|
|peer-endpoint-alias-r1|178,951s/exit0/956raw; chỉ153remoteIP→127.0.0.2/3, giữactual99port. 346PCAPrecords, nguồn thực127.0.0.1; hai input cóingress nhưng0handlerpacket/remote khôngdịch.|`38712fd08061cdffbdbc814c3e48b5eb129ca0f0c8f740fc74a50c1a85a7b98b`|
|peer-endpoint-match-r1|178,496s/exit0/974raw;153remoteIP+port→ownedproxies127.0.0.2:14022/127.0.0.3:14023. Proxyphát nguyênpayload từorigin tuple đãquảngbá tớiactual99port, 622PCAPrecords; handler/target/remoteprogress cócảhaiinput.|`e85db6f28b67d3f8b7b8d4c27f2739ca14d2f2c2e2e08b0a399f401cb3b20c54`|

Cảhai chỉ hai R1inputwindows/clock, dừng trướcbom/kếtquả; khôngmatchproof. AF_PACKET bindlo trong
privateuser/net/mount/PIDnamespace/chỉlo/no-route/NSSfiles-only/mộtlease; PCAP Ethernet/UDPpayloadhash
đối chiếu index/nativeingress/forwardrecord. Loopback ghioutgoing+incomingcopy, sốrecord khôngunique
packet. capture/proxystop trongcleanup; preflight/raw-seal kiểm inputSHA trướcchuyểnviệc. Runtimecopy
SHAe9bb4766…/static660447…/modOa08950…/base400000/RVA=VA−400000, khôngbinary/DB/packmutation.

`endpoint-verification.json`4317joins: exact153body chỉdeclared4/6bytes đổi, key/restbody/99ports,
PCAP→native payload/source/target vàunchangedforwardSHA. Negative ADown: originY159→204/279,
remoteY159; BRight: originX120→156/192, remoteX120. Positive remoteY159→246/X120→164/200;
298scopedendpointapply/progresschecks,192copiednativegetters/570actors,23positive/54negativeCRC và
6native153cases mỗi run làmodelkiểm riêng. Positive 192selectedpeer samples trùng pointer đãset
endpoint;155branch-site samples đềumissing0. Hai quota/count này khôngđượcghép one-to-one hoặc
suy mọi packet đi cùngnhánh. Negative chưa probe exactfallback, nên nguyênnhân tổngquát giữcandidate.

Giữ `target-check.txt` (before-only) và `prepost-check.txt` (across-call) failures của ADown, cùng các
getterdependenterrors. Câu hỏi endpoint khôngđòi chứngminh luậtinterpolation: lasttrackedapply
Y204 trước/sau vẫn159(distance45<threshold50), freshsample saucall cóY246, rawwrapper/copiedgetter
cũng246. `endpoint-peer-verification.json` ghi rõ phương pháp fresh-after-last-apply, khôngghi hai
strictreaders đãpass. BRight cóprogress inside/between calls. Scope chỉ nativeapply/target+fresh
remoteprogress; không giải lạitick/interpolationlaw. Firstendpointchecker giảđịnh hai quota đều96
đãfail/giữ; final báo192selected và155branchsamples riêng, không đổiraw/counter để làm pass.

Kếtluận confirmed hẹp: client gửi tớiIP153đãchọn; hai ca mismatch/match cóđúng kếtquả quan sát,
positive lookupregisteredpeer vàhandler/target/progress được nối bằngactualpayload. Không khẳngđịnh
universal sourceIPauth/NAT/reliability hoặc exactnegativefallback. Proxy là measurement/design,
không implementationproductionrelay. Production khôngđổi;226servertests reuse, docs/catalog/issues/
artifacts/requiredlocalhistory/diff kiểm bàn giao. CP137finalUIdebt giữ nguyên, khônggọi haiinput làhai
trận. Bước kế PLAN: topology hai networknamespace/veth vớiIP riêng/directUDP và kiểm actualCA/
wineserver namespace; đây vẫn là virtual-host test, chưa realLAN/Internet/K6b/c.


<a id="cp139"></a>
## CP139 — Hai actual CA/wineserver khác netns, direct UDP qua veth

Baseline CP138/PR209 merge`cb98a51`; source-firstreuse `virtual-peer-r1/reuse-question.json` pinCP138/
known99/153/probes vànetwork_smoke. Smoke trướcclient đã tạorootbridge+hai childnetns/no default:
UDP10.77.0.2:19002→10.77.0.3:19003/ack vàTCPtới10.77.0.1 cónguồn.2; hostnetns khôngđổi.
Đây chỉinfrastructure, khôngdùngsmokethaynativeproof.

Run`virtual-peer-r1/cycles`173,270s/exit0/702raw, run.json SHA256
`9d8dd3570c463e52da4c445c52f150625954e502eb762d1d60d5c4958aa6827d`.
Source/profile/copy giữ, helper/shim thực nghiệm bindstub10.77.0.1 và153IPpeer10.77.0.2/.3+actual99port;
TCPforward child127→rootstub, **khôngUDPproxy trongđường dữliệu**. Prefix/display riêng, Winecommands
chạytrongchildnetns. `virtual-network.json` pinactualA CA203/wineserver89 trongnetns3790, BCA554/
wineserver441 trongnetns4063 (inodefulltrongreport), rootnetnsriêng; kiểm trướcinput, khôngsuy từ
nsenterfrontend. C2S99IP0=.2/.3 khớp interface/TCProotpeer; IP1field nguyênopaque, khônggánpublicIP.

`virtual-verification.json`1423joins:140PCAPrecords/vethkv-a source.2:9753↔.3:9832 khôngproxyport,
exactpayload→nativeingress. R1ADown/BRight có4/6nativeapply,2targetchangesmỗiinput/currentprogress
inside/betweencalls.260scopedpeerjoins/168copiedgetters/612actors/20UDPCRC/6native153cases táchmodel.
Hai nativeclock/cảhai99/barrier25,713s, khôngtimeout; khôngbom/17/19/21/result/reset2.12report39
đượcgiữsemanticsunknown, khônggọiACK hoặcmatchresult. Khôngnativebody/opcode/offsetREmới.

Legacyparentcode vẫnkhai udp_proxy.mode=stale và cólo sockets khôngdùng; packets/replaysđều0,
service--proxy-a/b0, actualvethIP/port/nativepayloadprovenance chứngminh route direct. Giữfield/raw
nguyên, khôngsửa chỉđểhợpnhãn. Draftchecker banđầu giảmode=direct rồigiảevent_reportsrỗng đãfail;
`virtual-check.txt`/`virtual-final-check.txt`giữ, final kiểmconfiguredports/actualwire vànêu39unknown.
Đây là sửaassumption/checkscope, khôngcodeclient haysemanticconclusion đểlàmpass.

Cleanup toànchilds/proxyTCP/capture rồi delownednetns/outerPIDexit, preflightSHA/rawseal giữ. Runtime
copy/static/modO hashes nhưCP138; binary/DB/packnguồn bấtbiến.226server testsreusecodeunchanged;
docs/catalog/issues/artifacts/requiredlocalhistory/diff kiểm lúc bàn giao. CP139selfcheck/currentchange,
AUD042 qua136; audit địnhkỳ **CP140**. CP138pushretrycommit_refs_rules đãkhôiphục/sameheadPR209merged,
loglocalpinởnextcheckpoint; khôngsửacompletehistory.

Mộtphysicalhost/hai virtualnetns khôngrealLAN/K6b, Internet/K6c, NAT/relay hoặcfullmatch/K6a. CP137
finalUIdebt vẫnriêng. TiếpPLAN: reuse sharedcascade/nativeactualboth19/tworesults/reset2 profilevớiguards,
trên chínhtopologydirectUDP; chờnative180/110rồiquansátrender4/8/12/focusknownCP110, khôngshortinput
routeCP137 hoặcstateforce. Khôngghép runCP139haiinput vớiCP134haikếtquả thànhmộtgate.


<a id="cp140"></a>
## CP140 — Hai basic cycles trên virtualhosts, giữ sharedpartial; AUD043

Baseline CP139/PR210 merge`1ecd3fe`. Sourcefirst`reuse-question.json` của hai run dùngCP139topology/
actualCA+wineserver/99/153, CP134sharedroute/195flags, CP061oneSpace/stay/actual17→19/survivor/
ReadyStart2 vàCP110native-firstwait/render. Chỉdùng methodCP061, khôngđưaRoller/profile/itemsVNsang
currentfixture. Nativebody/codec/chainkhôngRElại; productioncode/pinnedguards giữnguyên.

| Run local/research/codex/ | Kết quả | run.json SHA256 |
|---|---|---|
|virtual-cycles-r1|319,097s/exit1/1051raw:1survivorresult/return/Start2 rồi alignmiss:r2-A-neutral-opening-col4. R1bridgeactual4,4≠sharedtarget3,4; B19/A21, khôngA19/all-dead/shared2cycles.|`2ce3020edeeb8cd602731aecabb042c73458614018f05325b8822ecafbe3a717`|
|virtual-basic-cycles-r1|289,290s/exit0/1149raw:oneA Space/stayspawn mỗi vòng→actual17/19/currenttrapSHA/peer170/B21→rank2,1/native175180/phòng/reset2;526vethPCAPrecords/directUDP.|`f24fb76318391a951b3d2b987409581dba2bd4d176850f337b2045e70daae043`|

Khôngcoigiữmọioptionmàoutcomephải giốngCP134; actualgeometry/reportquyếtđịnhscope. Bảnsharedpartial
và1146attribution/hashchecks giữnguyên, khôngquycausaldoIP/guard/timing. PLAN đổi mục tiêu sangbaseline
2basiccycles đểtránhsađàcanhtọađộ; khônggọi basicrun là sharedcascade/fullmotion proof.

`basic-verification.json`2917joins gồmraw/source/actualowners/veth/2epochs. Hai actual99 cóIP0.2/.3,
barrier15,300/15,332s<30s; A4/B8samechannel/key12345→12346/ID1→2. Hai actualA118cell3,3/power1,
159full39/entity/processID hiện tại/158full13cảhai view;12nativeflagsnapshots×195cells before/after
khớppredictionvàseedởclock2.24combo capacity/power/logicalspeed A1/1/5/B2/1/4khớpquadviewepochs,
khôngfullinventoryreset/pxs. CurrentA19 bindfreshA17SHA/cùnggeneration; B21 sauA19 vàlastblast, policy
post_blast_untrapped_drained chọnhairank2,1. Native175outer302/180full3/errors0/viewhai lần,110sau
return2, ReadyB/StartAgiữ; noEOFtrướcmốcđo. Khôngmintlife/outcome/forceclock hoặcinputcoordinates.

ActualCA+wineservervẫnđúngchildnetns/prefixtrướcSpace.526PCAPrecords là trafficvethactual99IPport
hai chiềucrossvirtualhosts, khôngUDPproxy; khôngnativefour-phase movementobserver ởrun này.
2364copiedflaggetters/2340array/192actor/252typedpacket/12native153cases làmodelchecksriêng;
partialrun3600actor/261packet/3940flaggetters/3900arraykhôngđược cộngthànhindependentgameproof.
Nativegate180110 xongmớichụprender: contact8ảnh(nativegate/unfocused4,12/focused4)cảhaiđềuhiệnphòng
KRLocal/KRGuest/AhostStart/BReady; actualwindowPID202/536khớpowner. Khôngchứngminhfocuscausalfix,
không95Ready saureturn2/Start3; complex-routeblack/inputdebtCP137vàsharedgeometrycrossvirtual giữ.

Source/preflight/lease/NSSfiles-only/user-net-mount-PID/rootbridge+childveth/no-default-route/cleanup/
rawsealpinđầyđủ; binary/DB/packsource bấtbiến. **AUD043 quaCP140/kếCP144**, đọc core docs/authority/
evidenceđốichiếu raw/source, sửa currentall-dead/sharedassumption, táchrenderkhỏiReadyresponse, cậpnhật
merge vàpeeraddress authority.226server testsreusevìproductionkhôngđổi; docs/catalog/issues/artifacts/
requiredlocalhistory/diffkiểm bàn giao. CP140/AUD043selfcheck/currentchange, reviewwaived; no gateclosure.

TiếpPLAN: thayprivatebind/153address shim bằng cấu hình tườngminh ởservice, default127/legacyprofile
khôngđổi; verifywire/nativebasic2cycles vàReadyB/110saureturn2, khôngStart3. KhôngtựtinIPclient99,
khônggánIP1làpublicIP hoặc triểnkhaiNAT/relay. Đây là virtualhosts/mộtphysicalhost, chưarealLAN/K6b/
Internet/K6c/fullmatch/K5/K6a/originalauthauthority.


<a id="cp141"></a>
## CP141 — Production bind/peer IPv4, basic2cycles và Ready sau return2

Baseline CP140/PR211 merge`f4f3371`; tái sử dụng CP138–140 layout/namespace/directUDP/basic Space-stay
và các native reader hiện có. [Contract/giới hạn](../../PROTOCOL.md#service-address-config),
[câu hỏi](../../RESEARCH.md#gap-peer-relay), [thứ tự](../../PLAN.md#current-order). Nguồn cũ KR/VN
và hash/build/locator trong `local/research/codex/service-peer-address-r1/reuse-question.json`;
không đọc thêm body native, port opcode VN hoặc tìm lại cơ chế bom/codec.

Run `service-peer-address-r1/cycles`:296,620s/exit0/**1167 raw**; run.json SHA256
`aa05fbfaead45ec8e7b85aeb607dfe5afe2741e925ec3c56845792ee5566e152`.
`cycles-recipe.json` chạy module production với explicit bind10.77.0.1/A.2/B.3; không alias_service.py,
không socket/153 shim.20 listener thực privateIP, actual CA+wineserver namespace trước input;521
veth PCAP records/noUDPproxy/actual99ports và bốn153 khớp toàn97byte theo profile/key/role.

`basic-verification.json`2921 checks xác minh hai cycles/current17→19hash/IDs/keys/195seed/statbaseline/
native159158175180110/ReadyStart giữa vòng; `address-ready-verification.json`25 checks xác minh
module/config/wire/actual95seq427/SHA/native110 cả hai trước cleanup/noStart3. 231server tests pass,
trong đó5 address regressions;192 actor/261 typedpacket/12 native153/2364flaggetter+2340array checks
là emulation trên copied runtime bytes, tách khỏi runtime. `visual-review.json` pin bốn ảnh gốc
và contact đã xem: phòng cả hai, ready badge sau BReady. Recipe/preflight/rawseal ghi source frozen,
cleanup hoàn tất trước seal; không thay binary/DB/pack nguồn.

`analysis-corrections.json` giữ lỗi invocation thiếu PYTHONPATH và failed exact+1native110 draft:
B có thêm survivor110 sau observation khi cleanup A; final verifier giới hạn native trước107,
đối chiếu actual95 chunk và hai ready110 sends trước observation. Không đổi raw/conclusion/gate.
Không movement/itempickup/sharedcascade/fullmatch/K6a/LAN/NAT/Internet; CP137 complexUI debt riêng.
IP1/subtype39 opaque. CP141 selfcheck/pendingPR, review waived; AUD043 qua140/kế144.


<a id="cp142"></a>
## CP142 — Cùng-run movement/item/basic2cycles/Ready trên production virtualhosts

Baseline CP141/PR212 merge`b1f2c72`, code production không đổi; reuse231 server tests.
[Contract/giới hạn](../../PROTOCOL.md#virtual-movement-cycles), [câu hỏi](../../RESEARCH.md#gap-peer-movement),
[thứ tự](../../PLAN.md#current-order). Sourcefirst CP139 finite observers/knownreturnsetup + CP141
productionconfig/basic lifecycle/Ready cuối; nguồn cũ/hash/build/locator đã pin ở reuse-question.
Không đọc thêm nativebody hoặc port opcode/layout/profile VN.

| Run local/research/codex/ | Kết quả | run.json SHA256 |
|---|---|---|
|virtual-production-movement-r1|169,071s/exit1/691raw; observer ackB timeout trướcOSinput. Bcommands cònexplicitroleA/pathA; khôngbom/result, khôngnetworkfailureclaim.|`ff0c0697b9057437fabaf758e6ce53d32821243a2b054bbbf998a804860e1143`|
|virtual-production-movement-r2|344,960s/exit0/1320raw;4finitewindows/590vethrecords;actualBombFire mỗi vòng/native effects/bompower2/2survivorresults/reset/Ready cuối.|`8ea46808a0a31324ef0f0b13b8698c6deb24b6d6109fd44372d923b9e51be964`|

R1 rawseal/sourcefrozen/cleanup giữ nguyên; `observer-attribution.json` pin generated Bcommands,
r2 prelaunch assert roleB/pathB và output/prefix mới. Không sửa cấu hình/source giữa run.
R2 `endpoint-peer-verification.json`632joins/4phases/26recipientapplies/8targetchanges/currentprogress,
`movement-wire-verification.json`1356joins ghép actual99IP/port/veth/nativeingress/currentkeyXORID/
actualowners/freshackboth/currentbombordering; không lấy nhãn namespacefrontend làm ownershipproof.
`basic-verification.json`3284checks/two175180110/currenttrap+deathSHA/IDskeys/195seed/clockbaseline;
barrier25,901/25,755s<30. `pickup-verification.json`70checks/eight viewrounditemjoins với165full12,
A1,1→2,1→2,2 và reset clock2; không contestedpickup/fullinventory. `address-ready-verification.json`
25checks/no-shim/module/actual95seq457+native110both/noStart3. `visual-review.json` pin ảnh gốc đã xem:
phòng cảhai/readybadge sauinput. Native/emulation:1314actor/408peergetters/297typedpackets/12native153/
45UDPCRC/2364flaggetter+2340arraychecks, tách khỏi livegameproof.

`analysis-corrections.json` giữ draft pickup selector lẫn feedback và thiếu XORprofileID trong check:
đối chiếu contract CP047 trước sửa verifier; raw không đổi. Các failure lịch sử không được nâng thành
pass hoặc giấu. Known complex-routeCP137UI/sharedgeometry/physicalLAN/NAT/Internet/fullauth/authority/
fullmatch/gates mở. CP142 pendingPR/selfcheck; reviewwaived, AUD043qua140/kế144. Tiếp PLAN ghép
selected current/same-old TCP bodies vào profile này, tránh thêm tracker hoặc RE lại codec.


<a id="cp143"></a>
## CP143 — Selected same/old TCP guard trên production virtual2cycles

Baseline CP142/PR213 merge`062906a`, production không đổi/reuse231tests; nguồn/hash/build/locator
pin trong `local/research/codex/virtual-tcp-guard-r1/reuse-question.json`. [Contract](../../PROTOCOL.md#virtual-tcp-guard-cycles),
[câu hỏi](../../RESEARCH.md#gap-diagnostic-service), [thứ tự](../../PLAN.md#current-order).
CP106/137 proxy đổi rootprivatebind3858→23858 và actualA17/19 source; CP142 input/profile/consumer
reused, không body native mới/portVN opcode/layout. S2C forward/archive, UDPdirect/noaddressshim.

Run`virtual-tcp-guard-r1/cycles`:344,196s/exit0/**1882raw**, run.json SHA256
`4eb9bf57efcecd6eb69a3a313abf73e690e94a5a410b0623a92fb6d7a7900dca`.
`guard-integration-verification.json`87checks/8same4old/actualbodySHA/front-upstreamseq/nativegetter/
guardrejection/noextraIDs165170; A135front→147upstream/B141→141,876frontoraclecases. Replays do
proxy phát, không gán emission cho nativeclient. Source S2Cunchanged/archive không thay fullwirecapture.
`basic-verification.json`3950joins/2native175180110/reset/currenttrap/195/keyID/Ready cuối;
`endpoint-peer-verification.json`632joins/4windows/26recipientapplies/8targetchanges/currentprogress,
`movement-wire-verification.json`1423joins/624vethrecords/sourceactual99/ownership/currentepoch.
70pickupchecks/eight viewrounditemjoins,25addressReadychecks/actualB95seq457/native110both/noStart3.
Actor1386/peer408copiedgetters,333typedpacket/12native153/46UDPCRC và2364flaggetter+2340arraychecks là model riêng; không cộng thành gameplayproof. Metadata nguồn
emulation ở actor-fields/peer-getter/native-reports/native153/CRC/flag-readers; full index pin tất cả.
`visual-review.json` pin4ảnhgốc/contact đã xem:phòng/badgeReady sau input cảhai. Preflight/rawseal
sourcefrozen/cleanup hoàn tất, không binary/DB/packsourcechange.

Không fullmatch/K6a/LAN/NAT/Internet/fullinventory/originalauthauthority/CP137complexUI/sharedgeometry;
selectedcapturedbody guards không là arbitraryforgedinputvalidator. CP143selfcheck/pendingPR,
reviewwaived/AUD043through140/next144. TiếpPLAN actualpendingguestEOF sau hai vòng, trước đó kiểm
TCPmeasurementproxyEOFpropagation để không nhầm proxy giữ upstream với client/server lỗi.


<a id="cp144"></a>
## CP144 — Pending guest EOF sau hai completed cycles, AUD044

Baseline CP143/PR214 merge`c692c83`; productionkhôngđổi/reuse231tests. [Contract](../../PROTOCOL.md#virtual-pending-guest-eof),
[câu hỏi](../../RESEARCH.md#gap-disconnect-replacement), [thứ tự](../../PLAN.md#current-order).
`virtual-pending-eof-r1/reuse-question.json` pin CP050/074/128/135/143/oldsource/hash/build/locator;
no newnativebody/VNlayoutport. Bench `proxy-eof-baseline.json` fail EOF3directions/deliveredbytespass;
`proxy-eof-fixed.json` cùng3cases pass với newcopyhelperclosepair. Không rewrite historicalhelper.

Run`virtual-pending-eof-r1/cycles`:434,087s/exit0/**2253raw**, run.json SHA256
`9b12adfb084aa65ab911cdb9df03ccf1e5931460ba29a395868c27dafb824d27`.
`pending-eof-verification.json`47joins:hai completedcycles+Ready→Start3/actualB118pending→ownedCA-B
PID543/prefix/netnsrevalidate→SIGKILL còn4,877s→frontproxyEOF/serviceEOF46,569ms/fence/no158/không
B17,19/nativeAforfeit175180107/slotID2→0/110chief1/render. NoStart4; killedB khôngnhậnforfeit.
Actualclock3/1593 nhưng158chỉ2 mỗi view; two normalresult2/bothviews, thirdforfeit chỉA.771vethrecords.
`basic-verification.json`4762checks xác minh first2normal/thirdcount tách; `guard-integration-verification.json`
87checks/8same4old/987frontframeremulationcases;70pickupchecks/29addressReadychecks/4peerwindows/
sourcekeytargetcurrentprogress cócùngrun. Modelcounts rawreport kiểmriêng, không cộng thành gameplay.
`visual-review.json` pin4originals/contactđãxem:bothReadyafterreturn2/map3/finalAroomBempty.
Source/config frozen/ownedcleanup/rawseal đầyđủ; sourcebinary/DB/pack bấtbiến.

DraftEOFchecker đoán110fields[0,1,1] đãfail; `analysis-corrections.json` pin actualslot_Afixture[0,2,0]
và nativechief1 độc lập. Failedscript/log giữ nguyên, không sửa raw. Ba barrier29,724/29,037/29,337s
<30 nhưng margin nhỏ; không attributiondohelper/probe và khôngcoldstartguarantee.
AUD044 đọc core docs→authority/currentcontracts→evidence/raw/reports/production/helper; giữ CP137UI/
sharedpartial/oldfailures scope, sửa activeEOFnextstep/110semantics/timeoutmargin/currentmerge/counter.
CP144selfcheck/pendingPR; reviewwaived, periodicnext **CP148**. Không fullmatch/K6a/LAN/NAT/Internet/
fullinventory/authauthority. TiếpPLAN exactoldUDP/currentkey trên directvirtualtopology, reuseCP047/130.


<a id="cp145"></a>
## CP145 — Exact old UDP/directvirtual key-drop, offload control và R2positive

Baseline CP144/PR215 merge`13b9f3a`; productionunchanged/reuse231tests. [Contract](../../PROTOCOL.md#virtual-exact-old-udp),
[câu hỏi](../../RESEARCH.md#gap-peer-movement), [thứ tự](../../PLAN.md#current-order). Reusequestions
ở`virtual-old-udp-r{1,2}` pinCP047/130known12sites/queueframe/keydrop/source/oldKR-VNscope;
không nativebodyRE hoặc port opcode/layout. Ownednamespace control khôngđụnghostinterfaces.

|Run local/research/codex/|Kết quả|run.json SHA256|
|---|---|---|
|virtual-old-udp-r1|352,402s/exit0/1896raw/2normalcycles; exactoldreplay khôngexactnativeingress, storedUDPchecksumresidue2485. Partialmục tiêukeydrop, khônggọiKRdrop.|`dc6dd3b69bed85a36928100140089906bf3a99385882cc2739036d499f129e37`|
|virtual-old-udp-r2|349,084s/exit0/1905raw/637vethrecords; exactoldingress/endpoint/queue+19/drain−19/59≠56keydrop/noapply,current13applypositive/tworesultresetReady.|`a4cb131280e20d85538157af34ce1953a2ae746f78c085d9dc21ea1967698b1b`|

R1`checksum-observation.json` và`offload-control-01/report.json` giữ candidate/control: synthetictest
ởprivateveth defaultcsumresidue42515/receive1; txrxoff csum0/receive2. Rawframes/socketscope/hostnet
beforeafter/cleanup pin; không gọi model là KRruntime. R2 topologytắt4vethoffload trướcclientlaunch,
exact59byteEthernetSHA`a2f3e898ae68a10c82099e02c0736659079a39e9f30de5d0486a561f6bbd8318`,
UDP17SHA`4a1f34f7dd63e7fd80c5255bd1af153af873419e490678033f5390a800f7c15d`, IP/UDPchecksums0.
`exact-old-verification.json`19joins/source+replayPCAP/nativeingress→endpoint→15queue19/drain→13key
wire59expected56→E667C2/noexactapply; armedA12beforeAF_PACKETsend. Cùngphase13currentnormalapply,
BRightpositive sau đó currentprogress. `native-getters.json`162copied52A350wrappers model riêng.
`basic-verification.json`4013joins/2nativeendings/reset/Ready;594peer+1444wirejoins/4windows;
70pickup/87TCPguard/25addressReadychecks;1560actor/384peergetters/333typedpacket/12native153/
43UDPCRC/2364flaggetter+2340array models riêng. `visual-review.json` pin4originals đãxem/bothroom/
Readybadge. Preflight/sourcefrozen/ownedcleanup/rawseal đầyđủ; binary/DB/packsource bấtbiến.

Khôngnânggate hoặcgộpCP144EOFvàoR2này. Low8wrap/currentforgery/wholeUDP/NAT/LAN/Internet/authority/
fullinventory/CP137complexUI/sharedgeometry mở. CP145selfcheck/pendingPR/reviewwaived; AUD044qua144/
kế148. TiếpPLAN endpointTCPchain/directbootstrap-login đểbỏchildloopbackforward, reuseCP008/015;
không tuyênbố IPpeerconfig là hoànthiện TCPLANflow.


<a id="cp146"></a>
## CP146 — Bỏ childTCPforward, directprivatebootstrap/WCP/auth/channel và2cycles

Baseline CP145/PR216 merge`cca0427`, productionkhôngđổi/reuse231tests. [Contract](../../PROTOCOL.md#direct-tcp-endpoints),
[câu hỏi](../../RESEARCH.md#gap-diagnostic-service), [thứ tự](../../PLAN.md#current-order).
`direct-tcp-endpoints-r1/reuse-question.json` pinCP006–008/CP015fields/currentprofiles/locators;
newgamecopies/binarye9bb4766 giữ nguyên; chỉ privatehostmap+typedredirect7IP, khôngpatchsourcebinary/
packDB/cache nguồn. SSI multisvrcachevẫnoriginalIPkhôngchứngminhactiveendpoints; runtimecallerscónguồn.

Run344,644s/exit0/**1888raw**, run.json SHA256
`10b306429a95ea0cbedd5deae58806999c07681cf4133f1ca4a44a36552283de`.
`direct-verification.json`49checks/eightnativeconnects/sockaddr/callerframes/servicepeers:
A3848/3848/3848/3858, B3838/3838/3848/3858, tấtcả đích10.77.0.1; first3actualpeerA.2/B.3,
channelrootproxy.1→service23858. Nochildforward/helperlogs; actualCA+wineserverNSđúng.
`fixture-change.json` pinbefore/afterS2C7typedIPonly/seq52/header/tail/3858; native2clock/result/
return/110/reset/finalReady đủ.2080basic/575peer/70pickup/87TCPguard/25Readychecks cùngrun;
models/nativegetters vàmanualrender ởreport/hash riêng. `visual-review.json` pin4originalsđãxem/phòng/
Readybadge. Frozeninputs/cleanup/rawseal đầyđủ, khôngsame-runLAN/gateclaim.

**analysis-limits.json:** compositionremovedUDPcaptureinitializer cùngforwardblock, nênPCAPcount0,
khôngwirejoins. Giữsourcefrozentrongrun, khôngbùbằngcapturecũ; nativepeerreportsvalidscope riêng.
Initialstackregexonly#0 sửaparserđọcthêmfromlines/8frames, khôngrawrewrite. Hostmap+redirectđổi
cùnglúc, khôngcausalcachefieldclaim; hollowWCPkhôngfullschema. CP146selfcheck/pendingPR/reviewwaived;
AUD044through144/next148. TiếpPLAN explicitservicechannelendpoint/defaultfixtureunchanged +native
followuprestoredcapture; fullauth/LAN/NAT/Internet/gates/CP137UI/sharedgeometry/fullinventory mở.


<a id="cp148"></a>
## CP148 — Production channel endpoint, baseprofile và UDPcapture; AUD045

Baseline CP146–147/PR218 merge`a3d1d94`; [contract/giới hạn](../../PROTOCOL.md#service-channel-endpoint),
[câu hỏi](../../RESEARCH.md#gap-diagnostic-service), [thứ tự duy nhất](../../PLAN.md#current-order).
Codex solo/self-check; review waived, issue14/gates giữ mở. Reuse CP0157/seq52/header/tail,
CP141explicitIPv4 config/tests, CP146actualconnect/topology/sourcefreeze và CP142–1454peerwindows/
basic2cycles/pickup/selectedTCPguard. `service-channel-endpoint-r1/reuse-question.json` pinnguồn;
oldKR/VNchỉđịnhhướngtừCP146, không thêmnativebody/quétcodec/cache hoặc portVNwire.

Code `ProbeService` thêm API/CLI channelIP/port; default rawfixture nguyên byte. Testssocketpair
xác minh emittedframe/header/seq/tail/only6endpointbytes và profile không mutation; invalidpair/variant
fail trước selector/listen/output. 237server tests pass; native model3cases trên renderedfixture14byte
vàCRC/partial negatives là model, **khôngwirecapture7**. Runtime nativeconnect là proof đích riêng.

| Run local/research/codex/ | Kết quả | run.json SHA256 |
|---|---|---|
|service-channel-endpoint-r1/cycles|0.365s/exit1/10raw; service reject trước client vì yêu cầu metadata52, baseprofile rawsequence:null. Capture đã init/stop,0UDP; giữ failure/freeze/cleanup.|`c3f95e9d5555df1acb7b2aaea3267befb81c904cd544e090df48d330c2a902a0`|
|service-channel-endpoint-r2/cycles|345.338s/exit0/1894raw; productionCLI explicit10.77.0.1:3858/baseprofile nguyên byte,2cycles/4peerwindows/Ready;604vethrecords.|`0d883b150487a4a0ce4b8c8c5dfa254da9b493211853573b7db30337332c4986`|

Sửa r1 bằng hỗ trợ metadata null của đường raw, **vẫn verify fixed52CRC/knownvariant**; test regression
thêm legacy:null và chạy thư mục/prefix/game/freeze mới. Không đổi source/config giữa run; r1bytes
server/runtime-source riêng và rawseal giữ. Initialtestsocketpair thiếu `chunks` trong fixturetest,
parser thiếu PYTHONPATH và gọi guardchecker trước reportReady có rawfailedlogs riêng; không sửa
raw hoặc kết luận game để làm checkerpass. Nativeupstream replay sửa selection theo reply_ordinal
để không coi bootstrapB là framedchannel; declaredpartial vẫn fail nếu thực sự thiếu byte.

R2 `runtime-preflight.json` parse harness/assert đúng1Capture init sauTopology, stop trongcleanup;
source/controller/profile/binary hash kiểm lại saucleanup trước chuyển việc. ActualCA+wineserverA/B
trongchildnetns riêng/rootbridge/veth/files-onlyhostmap10.77.0.1/no default, nguyêncopye9bb4766…
(base400000/RVA=VA−400000; native model modOa08950…; sourceCAa52545… bất biến).
First3TCPservicepeerA.2/B.3; channelrootguardproxy.1:3858→service23858 còn lại; khôngchildforward/
UDPproxy. Baselinefixture redirect52byte vẫnSHA`b514742b…`/metadata null, service CLI chọnIP/port.

`direct-verification.json`53joins; `basic-verification.json`3905source/lifecycle/2epochs;
`movement-wire-verification.json`1374joins/604actualvethrecords→nativeAPI exactpayload/actual99ports;
`endpoint-peer-verification.json`537joins/4windows/currentkey/target/currentprogress;70pickup,
87selectedTCPguard(8same4old)/25Ready. Native2clock/full129121/full39159/full13158/current17SHA→19/
survivor21/rank2,1/175302/1803/110/reset2; barriers26.229/26.357s<30s. NativeRoomRender4ảnh đãxem,
Readybadge sau95 ởcảhai view; noStart3. BằngchứngcurrentTCPguards/13movement không làfullvalidator.
Models tách riêng:1338actor/348peergetter/2364flaggetter+2340array/330typedpacket/12native153/
43UDPCRC/3redirectcases. `raw-seal.json` pin1894outputs; cleanup capture/service/proxy/Wine/namespace
xong trước analysis/sourcechanges. Không ghép604PCAPvào CP146 vốn thiếu capture.

**AUD045 quaCP148/kếCP152:** đọc coredocs/authority/capacity/endpoint +evidence/raw/report/source/
4renderframes; sửa stale CP146–147pendingPR thành PR218merged, header contractCP146,
RESEARCHresult/reset/disconnect/peer nextoldUDP/TCPchain đã giải. PLAN chuyển sangpositivechannel
trực tiếp listener service/no rootguardproxy. Không sửa record complete/pinnedrun hoặc xóa failure;
CP1478slot vẫn synthetic, CP137complexUI/sharedgeometry/fullinventory, fullauth/Windowscontrol/
physicalLAN/NAT/Internet/fullmatch/K6a/gates còn mở. CP148 selfcheck/pendingcommit/PR/exacthead.


<a id="cp149"></a>
## CP149 — Positive allTCPdirect, không rootguardproxy; giữ R1Firepartial

Baseline CP148head`d197c14`/PR219merge`f27c667`; runtime chuẩn bị trênheadd197c14 lúc PRđược merge,
serverbytesgiữ nguyên/quakiểmpreflight, khôngsource/configđổigiữarun. [Contract](../../PROTOCOL.md#direct-channel-positive),
[câu hỏi](../../RESEARCH.md#gap-floor-pickup), [thứ tự](../../PLAN.md#current-order).
`direct-channel-positive-r1/reuse-question.json` pinCP148runtime/contracts/99/153/currentcontrollers;
chỉ khoảng trống channelpeer.1 còn thiếu, khôngnativebody/codecRE hoặc chuyểnVNwire.

Run `local/research/codex/direct-channel-positive-r1/cycles` **352.599s/exit0/1334raw**,
run.json SHA256`bb78dc2f3cb2c5093ddea7e04ad142627fab09e9af6c38977de263dfc0042429`.
Controllercopy chỉbỏspawnrootproxy vàserviceport23858rewrite, giữAPIchannelIP.1/port3858/baseprofile/
timeout/privatehosts/observers. `runtime-preflight.json`/`raw-seal.json` pinbinarysource/profile/
controller và cleanup, khôngprocess của run còn lại. SourceCA/modO/static/Wine e9bb4766… giữnhưCP148.
Nativeconnect ở8callsite khớpactualserviceport, mọiTCPsourceA.2/B.3 gồmchannel3858; khôngTCPproxy/
childforward/UDPproxy; positiveinputsonly, khôngsamed-oldTCPreplay. Không gộpCP148negativeproof.

`direct-verification.json`55joins; `basic-verification.json`3289joins;586vethPCAPrecords/index,
1334wire→nativejoins/499peerjoins/4windows/25Ready. Hai nativeclocks/full129121/39byte159/13byte158/
195flags/power/current17SHA→19/survivor21/native175302/1803/110/reset/95Ready/noStart3 cónguồn.
Models/syntheticreceiver/counterreset riêng:1248actor,324peergetter/2364flaggetter+
2340array/12native153/42UDPCRC; không dùng model thay runtime hoặc tự đóng gate. Fouroriginals
return2/finalReady đãxem qua `ready-contact.png`; cảhai room/hostStart/guestReadybadge, không chữa
CP137complex-routeUIcause. Barrier vẫn<30s, không tăng hạn.

**Item correction quan trọng:** R1chỉactual124Bomb16, native165tăngcapacity1→2/power1; actual118/159
power1. R2actual124Bomb16vàFire17/native165/effect+power2, hai viewresetclock1,1. Tổng3pickup/6viewjoins,
`pickup-verification.json`56checks chỉscope này. `basic-power-assumption-failed.txt` giữchecker kếthừa
CP148power2cảhai vòng đãfail; raw actual118 vàabsence12417/R1 bácgiảđịnh, không sửa raw.
`analysis-corrections.json` ghi nativecallbacks/hashes/currentinputs vàmissingR1Fireunknowncause;
modelterrain nhậnactualpower1/2 đãcócontract, khôngđổi consumerhaykếtluậnđểlàmcheckpass.
Không tuyênbố Fiređủcảhai vòng hoặc proxy removal gây miss/loss. CP148same-run2Fire giữlịch sửriêng.

237servertests reuseCP148codeunchanged, docs/catalog/liveissues/sourceartifact/fullrawhistory/diff
kiểm trước bàn giao; Codexselfcheck/waivedreview. CP149khôngauditđịnhkỳ, AUD045qua148/kế152.
CP149pendingPR; gate/fullmatch/K6a/realLAN/NAT/Internet/Windowsoriginal/authauthority/8actualclients/
CP137UI/sharedgeometry/fullinventory mở. TiếpPLAN sourcefirstreuseCP082feedback/CP106native165:
newrunboundedOSfreshfeedback/current12417→165Fire trướcreturnsetup, rồi BSpacevòng2 đểkiểmowner
hai chiều; không fakepacket/actorstate, giữmiss/blocker khi chưa đo đủ.


<a id="cp150"></a>
## CP150 — Native Firegate và A/Bowner qua hai kết quả/reset trên directroute

Baseline CP149head`50c28ce`/PR220merge`262a72f`; [contract](../../PROTOCOL.md#direct-item-feedback),
[câu hỏi](../../RESEARCH.md#gap-floor-pickup), [PLAN](../../PLAN.md#current-order). Productioncode/profile
unchanged/reuse237tests; Codexselfcheck/reviewwaived. Sourcefirst `direct-item-feedback-r1/reuse-question.json`
pinCP082freshfeedback/CP106165/CP149actualpartial/currentprofile/mapdata; nativefield/operator/window
reusekhôngnewbodyRE/codec hoặccopyopcodeVN. R2chuẩnbị trênCP149head, sourcebytesunchanged qua merge.

|Run local/research/codex/|Kết quả|run.json SHA256|
|---|---|---|
|direct-item-feedback-r1/cycles|199.741s/exit1/720raw: actualFire124/native165/stats đãcó trướcextraalignment nhưng20pulse/tolerance4tớiY279fail; noSpace/noresult.|`b86300e276444288a484fa8f8b3e39c1dd7248228ce793c211a054718ad269da`|
|direct-item-feedback-r2/cycles|364.479s/exit0/1348raw: Firegate/nativeconfirmhai vòng; actualA118R1/B118R2,2nativeoppositeending/reset/finalReady;664vethrecords.|`e741fddb8cd0e5ba7378a4bbd27e81d5c98ebe9def98c4fe070a60193c136a82`|

R1`partial-observation.json` pinactual12417SHA/timestamp<trystart, native165cursor12/errors0/
copygetter ởA/Bcapacitypower1,1→2,1→2,2. Yneutral20samples159/209/259/319/269…/319 cho biết vìsao
exactcenterconditionkhôngđạt trongbound; không gọi đây làFirefail hoặcluậttick/speed đãRE. Model
fourphaseparser đòiR2nênStopIteration ởpartial giữlog, không đếm làfourwindowproof.
R2không nới20pulse/.045s/3sneutral/5swait; chỉkiểmactual124index17/kind1+native165hai view trước
phátthêmphím vàstop khi quan sát đủ. Vòng1khôngextraalignment; vòng2cóalignment dừng bởi nativepickup.
`fire-feedback-verification.json`16joins pin124SHA/165/timegate<Space vàcorrectowner/power. Không
fake124/165, gọi gamefunction/ghi actor hoặc đổi sourceconfig giữarun; failedR1/cleanup/freeze giữ nguyên.

Actualfour124/Bomb16+Fire17 mỗi vòng cócurrentkey12345→12346;8native165events/70pickupjoins pin
beforeaftercapacity/power trênhai view. Actual118/159Aowner0/ID1/cell3,3/power2 vàBowner1/ID2/cell9,3/
power1; BkhôngnhặtFire. Native158/process/195beforeafter/seedreset khớppower thực vàmap/policypin.
CurrentA17→19/B21 vòng1, B17→19/A21 vòng2; native175302/rankstorage2,1→1,2/1803/Prepare110/reset2/
actualB95Ready/native110/noStart3. Không quy currentreport policy làworld/serverauthority gốc.

3537lifecycle/1490wire/499peer/70pickup/16gate/25Ready/55allTCPdirectchecks;4peerwindows/664vethPCAP
payload→nativeingress/currentkey/target/progress, native8TCPconnectactualservicepeerA.2/B.3/channel3858/
noTCPproxy/childforward/UDPproxy. Models1668actor/324peergetter/2364flaggetter+2340array/294typedpacket/
297report/12native153/42UDPCRC riêng. Hai barriers27.376/27.838s<30; không tănghạn/coldstartguarantee.
4nativegatedreturn2frames đãxem/hostAStart/guestBReadybadge; CP137complexUIcause chưađược giải.

SourceCAa52545…/Winee9bb4766…/modOa08950…/base400000/RVA=VA−400000 giữ; preflight+rawseal xác nhận
source/configunchanged tới ownedcleanup/capturestop/end trướcchuyểnviệc. CP149noR1Fire vẫnunknowncause;
CP150R2đủFirekhông backfill/chứngminhcausalproxy/input/loss. K6a/fullmatch/LAN/Internet/8realclients/
originalauth/fullinventory giữmở. CP150khôngđếnaudit; AUD045qua148/kếCP152. CP149PR220 đãmerge,
CP150selfcheck/pendingPR. NextPLAN reuseCP030/062actualcontest/delay165/onewinner vàcurrentdirectroute,
chỉ đo integrationgap; khôngRElại schemaitem/owner hoặcđónggate dohai vòngbasic.


<a id="cp151"></a>
## CP151 — R1actualcontest/onewinner; R2loadingdeadline partial, không2cycles

Baseline CP150head`bcf543c`/PR221merge`1dbc369`; [contract](../../PROTOCOL.md#direct-contested-pickup),
[câu hỏi](../../RESEARCH.md#gap-map-spawn-loading), [PLAN](../../PLAN.md#current-order).
Sourcefirst `direct-contested-pickup-r1/reuse-question.json` pinCP030/062/150/profile/claimlogic và
khácbiệt overlapspawn/Roller cũ so distinctspawn/Fire hiện tại; không RE lại native124165/codec.
`runtime_hold.py` **nguyên bytes CP062** delaypositive1652s, không sửaproductionclaims/consume/key;
measurementdesign, khôngNAT/proxy/fairnesslaw. Controllercopy đưa A→Y199/B→Y359 bằngfreshfeedback,
actualOSDownA/UpB0.30s/release, assert chưa cóFire17claim trước race; khôngfake124/forcewinner.

Run`local/research/codex/direct-contested-pickup-r1/cycles` **353.595s/exit0/1204raw/407vethrecords**,
run.json SHA`93664ce4a666d33da181959956ea18ef54c36436ad6cefb0c13182b8d8f06479`.
Exit0là cleanup/report, `early_completion.reason=virtual_two_returns_native_observer_timeout`,
**khônghai chu kỳpass**. Source/profile/client e9bb4766…/base400000/CP150API/noTCPproxy giữ,
wrapper/input/copy thaycóhash/recipe/preflight/rollbackrõ. Không đổi source/configgiữarun.

R1actualA124Fire17SHA`dbe9320b…`, B124SHA`b450e9fd…` cùngcell6,3/kind1/key12345. Aarrivesfirst,
B`claim_not_current_or_already_consumed`; Brequest trước positive165delivery1.692042763s;
Firehold2.032375181s. NativeoneFire165/receiver/cursor12/errors0/slot0, A4BCpower1→2 vàBpower1
khôngđổi, capacitycảhai2giữ. `contest-verification.json`45joins; nativerequestsproducer/authauthority
khôngsuytừstub. R1oneBombA118/ID1/power2/158/current17SHA→19/B21/native175180110/resultreset cónguồn.

**R2blocker:** bothnative121full129 vàactor0/1init, actualTCPstreamkhông99trướchạn30s; loadingpolicy
đóngcảhai channels vớiloading_timeout. R2khôngclock/contest/results/Ready2. Servicelog rank/return/
refreshforfeit cóbroadcastrỗng, khôngnativereceipt; khônggộp CP150positive vào partialrun này.
`partial-verification.json`905joins pinnative2start/1clock/1result/bothclose/deadline/actualC2Safter121;
laststage13→startup1/connect chỉhệquảobserved sauclose, khôngclientrootcause. CP021writerE7F2A0 và
onceguardstage+6410/callerE7F4DD/13ADD10op99 đãbiết, nextreadonlytimestamptrace đểphân biệtentry/
guard/build/send/lateactual99; giữ30s/no99fake/forcedclock. Chưa causal165delay/packetloss.

298peerjoins/2R1windows/192peergetters và407PCAP exactpayload→native22CRC mẫu đãkiểm; noR2window.
Models1176actor/1182flaggetter+1170array/171typedpacket/174report/6native153 táchruntime. Initial
contestchecker thiếuGEN165context nên trùngsnapshot, fixjoin đúngnativebefore/aftercallback giữlog;
initialpartialchecker giảR2khôngrank, thaybằng phânbiệt emptyforfeit vsnative1result cóbằngchứng,
không sửa raw/record cũ. Allrawseal1204outputs/sourcefrozen/cleanupcapture/service/Wine/namespace xong.

237servertests CP148 reusecodeunchanged; docs/catalog/liveissues/artifacts/rawhistory/diff kiểmhandoff.
Codexselfcheck/waivedreview; CP151pendingPR, auditđịnhkỳ kế **CP152**. TiếpPLAN loading99writer/send
trace cónewobservable, khôngrandomrerun/increase30s. Fullmatch/K6a/LAN/Internet/8realclients/fullauth/
CP137complexUI/sharedgeometry/fullinventory cònmở; R1contest khôngđónggatehaytaskhai vòng.


<a id="cp152"></a>
## CP152 — load trước99, cold milestones và đối chứng observer

Baseline CP151 đã merge [PR222](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/222)
head`1a0a494`/merge`79a51e6`. [Contract/kết quả](../../PROTOCOL.md#loading-observer-timing),
[câu hỏi](../../RESEARCH.md#gap-map-spawn-loading), [thứ tự](../../PLAN.md#current-order).
Các dòng pendingPR trong CP148–151 là snapshot lịch sử tại CP đó; không là trạng thái tích hợp hiện tại.

Source-first: CP021 writer/guard/send, CP024 cached `E59F30/E5A290/E0F6C0`, CP030/062 tranh item,
CP150/151 profile/directroute/claims. Hai cửa sổ mới `E59550` và `E57C20` có static SHA660447…;
`native-preimage-comparison.json` so3 vùng với hai copy Wine e9bb4766… nguyên bytes. Không chạm
sourceCA a52545…/pack/DB. REA IDA đang gắn copy đúng hash nhưng request containing-function lỗi
producer data; đã close session và dùng helper pefile/capstone có sẵn, không sửa cấu hình/DB.
Static cửa sổ tuyến tính, không nhận là CFG đầy đủ. Raw/report sau đo giữ local, không commit binary.

| Run local/research/codex/ | Giây / exit / raw | run.json SHA256 |
|---|---:|---|
| contest-loading-trace-r1/cycles | 211.529s / 130 / 899 | `38dc6f7ff9c172d287d69b03aba92faca4a18ede6b43fb86fef10fab58657c38` |
| contest-loading-milestones-r2/cycles | 191.986s / 1 / 726 | `b9632da226b697ac9425aef95590911fcb43f1a93b69e588a022ed2568d297ff` |
| contest-loading-staging-r3/cycles | 204.841s / 0 / 684 | `454d3de1c3e2835e0a5d7c654d9d2827e1fd1b4f1069323eb5c42b4640d301f5` |
| loading-observer-control-r4/cycles | 174.444s / 0 / 726 | `9609d49c0d4a8b29c254d09391d1ea627abec166696eea3bd5fcc83a6fc96dac` |

- Trace r1: guard0/writer→send xảy ra sau close; actualTCP99 không có. Operator SIGINT sau khi đủ
  dữ liệu, `finally` cleanup/exit130; repeated guard1/bt đã giữ. Không gán nguyên nhân R2 CP151.
- Milestones r2: R1 barrier trước30s, `+2D0`≈28,39s; sau đó dừng20-pulse OS alignment199±12.
  Native neutral159↔214/234, chỉ Bomb16 claim, chưa Fire17; không gán item/codec fail.
- Staging r3: đổi riêng hai approach tolerance12→20, thêm4 cold timestamp từ cache CP024;
  R1 vẫn deadline, nên **chưa kiểm được tolerance mới**. Các segment/timestamp tại contract.
- Observer r4: A giữ4 internal state hook, B tắt lúc load; cả hai public `B32390` còn929 hit.
  B actual99 trước hạn, A muộn; chưa clock cả hai, không contest/Ready2. Forfeit cleanup không
  là trận được chơi. `gdb.breakpoints()` ở đây không trả internalPython bp, không báo counter của
  chúng bằng0. Chưa kiểm branch khôi phục tại clock vì run không có clock.

`loading-trace-verification.json`915 assertions; `loading-milestones.json` các run r2/r3/r4 lần lượt
739/695/738 assertions, **bao gồm kiểm hash raw**; observer4 checks/policy/hit-delta. Đầu r4 analyzer
chưa nối marker qua dòng `OBSERVER_HITS`, giữ `loading-milestones-initial-join.json`; sửa regex cho
đúng sequence rồi đối chiếu timestamp native, không sửa log. Internal hit counter không có thì khai
scope public, không tự biến absence thành0. Không chạy lại framer/getter models đã đủ nguồn để chứng
minh contract cũ; CP152 là load observation. 237 server checks CP148 được reuse sau so41 source file
SHA giống hệt, không nhận là vừa chạy lại test.

Bốn run đều verify95 input sau cleanup; raw seals giữ end-run/captureclose/ownedprocess/namespace
không còn PID. Exit0/1/130 không là acceptance. Production/profile/deadline30s/delay1652s giữ trong
mỗi run. No fake99/clock/guard/actor write/nativefunctioncall; runtime vẫn modifiedWine/privatevirtualhost
cùng host vật lý. Hai epoch contest, nguyên nhân CP151R2, originalauthority/fullmatch/K6a/LAN/Internet/
8 client thật/CP137UI/sharedgeometry/generalreset còn mở. Không gộp bốn run thành trận hoàn tất.

AUD046 đọc docs nguồn theo thứ tự và đối chiếu raw/report/cachedasm/currentissue; sửa navigation
pendingPR và next-step endpoint/feedback cũ. Counter qua152, kỳ tiếp156. CP152 tự kiểm/reviewwaived,
chờ commit/PR. Tiếp PLAN gate riêng B32390 trong recipe copy lúc load rồi bật lại tại clock; không
nới timeout hoặc lặp RE124/165. Hồ sơ immutable tại records/CP152 sau khi docs đã sync.


<a id="cp153"></a>
## CP153 — loading observer gate, hai contest và hai nativeoutcome/Ready

Baseline CP152/[PR223](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/223), head`8d11943`/
merge`a92fdc2`. [Contract](../../PROTOCOL.md#loading-hot-gate-contest), [gaps](../../RESEARCH.md#gap-floor-pickup),
[PLAN](../../PLAN.md#current-order). CP152 pendingPR là snapshot lịch sử, không còn hiện trạng.

Sourcefirst `loading-hot-gate-r5/reuse-question.json` pin CP152counter/observer/statehooks/control,
CP021/024native locators, CP030/062/151claims và CP150profile. Không đọc body native mới. Copy giữ
SHAe9bb4766…; sourceCAa52545…/pack/DB bất biến. Chỉ observerB32390 disable khi Start121 với3tracked
bombpointer0, assert loadinghitdelta0 rồi restore tại nativeclock; cả4Chronology statehooks vẫn bật.
Cold milestones/OSstaging20/2s165measurement/productionclaims/profile/deadline30s giữ như r3CP152.
Đợt restart daemon chỉ mất handle tool; process vẫn chạy và được theo dõi tiếp, không launch trùng.

Run `local/research/codex/loading-hot-gate-r5/cycles`: **372.488s/exit0/1132raw**;
SHA`f18905f82b62002ecef3ce5ef1e7177ac5e52a0d10365d902a1e66dc6e76c10c`. `early_completion=rejoin_two_rounds_refreshed_ready95`, không error;
96 frozeninputs verify sau end/cleanup, namespace không PID/capture/service/Wine owned cleanup xong.
`raw-seal.json` pin raw trước analysis; không đổi source/config giữa run, không ép99/clock/actor/guard.

`loading-milestones.json`1155 assertions gồm rawhash/actualTCP99/seq/CRC/timestamps; gate16checks pin
4start/v2D0/clockrestore sequences/loadinghit0. Times/+2D0/actual99/barriers và giới hạn tại contract,
không quy timing nguyên bản hoặc nguyên nhân chính xác CP151R2. B32390 vẫn quan sát gameplay sau clock.

`contest-verification.json`84checks/2keys/Bwinner cảhai vòng; `capture-claim-verification.json`14checks
nối6actual124 SHA/seq/typedbody từ actualC2S, không dùng encoder tự sinh. Mỗi view chỉ4successful165
(Bomb+Fire mỗi vòng), onlyBpower1→2/resetR2baseline;2484captured securefields gọi native52A350 model.
`cycle-boundary-verification.json`43checks pin AownerR1/ID1/power1 và BownerR2/ID2/power2,
currenttrapSHA→19/oppositesurvivor21/chosenranks2,1→1,2/native175180110/fullread/errors0/actualReady95.
`endpoint-peer-verification.json`499assertions/4phases,324nativepeergettermodel; bản R1-only đầu giữ
`endpoint-r1-only.json`, sau mở đúng vòng2 trước khi nhận là4window. Không tạo lại nativeframer model
để chứng minh contract đã biết. CP148237servertests reuse sourceunchanged, không nhận là vừa chạy lại.

Hai frame `A/B/return2-ready-native110.png` đã xem, SHA ở `visual-review.json`: phòng có2user,
AStart/BReady/guestReadybadge. Không dùng8 ôUI làm bằng chứng8 người chơi, không giải CP137complexUI.
Delay1652s/first-arrival/fuse/rank/currentreportguard vẫn là measurement/policy emulator; fullauth/position/
fullworld/originalauthority/Windows/generalreset/K6a/fullmatch/LAN/Internet/8realclients còn mở.

CP153 tự kiểm/reviewwaived/chờPR; AUD046qua152/kế156. Đóng hai next-step hẹp loadrecipe/contested2epoch,
không đóng issue14/gate. Tiếp PLAN currentallTCPdirectpendingguestEOF/nativeAcleanup thay proxyCP144,
reuse raw/handler/recipe, không RE lại đã biết hoặc ghép các run cũ thành gate.


<a id="cp154"></a>
## CP154 — actualpendingBEOF trên allTCPdirect sau2contestedcycles

Baseline CP153/[PR224](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/224) head`68c8003`/
merge`3468200`; [contract](../../PROTOCOL.md#direct-contest-pending-eof),
[câu hỏi](../../RESEARCH.md#gap-disconnect-replacement), [PLAN](../../PLAN.md#current-order).
Reuse-question pin CP144/050/074/135 knownownedkill/native107-slotclear-chief/110 và currentCP153
profile/directroute/contest/loadinggate. Controller copy nguyên block Start3/actualB118/ownership/
kill/timeout/cleanup; GDB chỉ thêm knownnative107/slotclear/chief sites. Không scan body native mới,
không port VN layout, không thay production/profile/30s/2s165/gateB32390onlyloading.

Run`local/research/codex/direct-contest-pending-eof-r1/cycles` **422.640s/exit0/1283raw**,
runSHA`1a7ae6edf73df8066fbe8a231a7878745741049059b628abf0ef49b59f9c1cdc`. `early_completion=two_normal_cycles_then_pending_guest_eof_return`;
autoseal sau run/cleanup,96inputs hash giữ, namespace không PID/capture/service/Wine cleanup xong.
NativeCAPID512/prefix/netns/comm revalidated trướckill, không dùng PID543 cũ củaCP144.

50EOFchecks nối normalReady2→Start3/actual99/clock→B118pending→ownedkill→directserviceEOF;
remainingfuse4.948958128s, EOFsau mốc after-signal41.417847ms
(khoảng record-signal có giới hạn; không so tốc độ với proxyCP144). Job3round_closed/no158/noB17,19;
chờ quá fuse+4s; forfeit chỉA. NativeA175180 lần3/107ID2→0/110chief1 và ảnh roomBempty/Start
đã xem. Typed107[1,2,0]/110fixture[0,2,0] táchnativechief1; noStart4/Breceipt/3normalmatchesclaim.

First2cycle43checks, contest84checks/Awinner cảhai key (BwinnerCP153 giữ riêng), captureclaim14checks/
6actual124;746peerassertions/4windows,2664securefields+480peergetters native52A350model riêng;
loading1316assertions gồmrawhash/actual99,24gatechecks/3clocks/view/restoreB32390. Không chạy lại
codecbodyRE/model để đóng kiến thức đã biết.237serverchecks CP148 reuse sau so41sourcefiles unchanged,
không nhận là vừa rerun.3originalPNGđãxem/hash ởvisual-review; 8ôUI không8realclients.

Docs/issue14 cập nhật gapdirectEOFđãgiải và freshnessintegration kế, CP153PR224merge; snapshot CP153
pendingPR giữlịch sử. CP154 tựkiểm/reviewwaived/chờPR, AUD046qua152/kế156. NoK6a/fullmatch/LAN/
Internet/8players/authority/Windows/accountresume/generalreset; không rewrite raw/completehistory.


<a id="cp155"></a>
## CP155 — same-run oldUDP/currentpositive, contest/lifeagreement và directEOF

Baseline CP154/[PR225](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/225) head`a15862e`/
merge`b766222`. [Contract/limits](../../PROTOCOL.md#direct-oldudp-contest-eof),
[gaps](../../RESEARCH.md#gap-udp-freshness), [PLAN](../../PLAN.md#current-order).
Reuse-question cảhai run pin CP145offloadcontrol/capture/12nativeprobes/armedack/knownconsumer và
CP154currentdirectcontest/EOF. Ba helper nguyênbytesCP145R2, production/profile/30s/2s165/B32390gate giữ.
SourceCAa52545…/Winee9bb4766…/modOa08950…/base400000 bất biến; không nativebody RE mới/VNlayoutport.

| Run local/research/codex/ | Giây/exit/raw | run.json SHA256 |
|---|---:|---|
| direct-contest-oldudp-eof-r1/cycles | 350.859s / 1 / 987 | `478c1c91c828116789db9ea9f73a17869a083d477d75affb510dc5c6e2ae9ec5` |
| direct-contest-oldudp-eof-r2/cycles | 436.491s / 0 / 1274 | `c56d80bcb893002c81182c173acefde4024ec5d3c1766f26c2c7b4c4e123650b` |

r1partial:17exactoldUDP/keydropassertions/17samephasecurrentapply,2contests/BwinnerR1/AwinnerR2;
R2returnsetupneutralfeedback không đủ3mẫugiống nhau trước3s. Lastkeyup có3samples289,214,214;
`partial-observation.json` giữ mẫu/time/error, không kết luận oldUDP tác động localA hoặc speedlaw.
Không ending2/Ready/EOF3 trong r1. r2 chỉ hostbudget3→6s cho `contest-returnsetup`, còn3freshsame/20pulse/
.045s/tolerance/keyrelease/deadlinegame30s giữ; không coi đây là causalclientfix hay rewrite r1.

r2full:19exactoldUDPchecks/11samephasecurrentfull13apply + freshR2BRight; rawEthernet59/UDP17checksum0/
exactsource/SHA/actual99ports→ingress/registeredpeer/queue+19/drain−19→wire59expected56slot1/keydrop/
noexactapply. Replayharnesskhác nativeBsecondemission; payload/headerCRCkey không rewrite.
24loadgate/84contest/14actualcaptureclaim/43first2cycle/50directEOF checks;575peerassertions/4windows,
3018securefields/372peergetters/150UDPwrappers/16lifewrappers native52A350model riêng.
23lifechecks pin cảhai viewloser56→1/beforerankother0;26entity/195flagsresetchecks nối actualID1/2/
native15939/15813/full/errors0/process và seedreset;2758flaggetter+2730nativearrayaddr trên14snapshots.
1307loadingassertions gồmrawhash/actual99. Source/policy/model/runtime được phân biệt ở contract.

Bdebuggercuối log cómemory/KR_STOP sauentity3/ownedfault; whole-loghealth draft fail, sửa phạm vi
entity/flagcheck tới native159end3, giữpost_entity_probe_messages và analysis-corrections; không gọi
wholeloghealthy hoặc sửa raw. Importsdraftterrain_shadow/probe_terrain fail được sửa theo module
sẵn có terrain_model; không có kết luận runtime từ importfail.
97inputs verify sau cleanup mỗi run; rawseal/end/namespacekhôngPID/capture/service/Wine cleanupxong.
237CP148tests reuse sau41sourceSHAunchanged, không rerunclaim.3PNGReadyA/B/roomAguestempty đã xem,
hash ởvisual-review; no8realclients/Start4/CP137UIcauseclaim.

CP155 tự kiểm/reviewwaived/chờPR; CP154mergePR225 cập nhậtcurrentdocs/issue14, lịch sử immutable giữ.
AUD046qua152; **CP156 audit bắt buộc**. Tiếp đối chiếu acceptanceK6a/authority và gapthực tế theoPLAN,
không tự nâng gate hoặc randomrerun. Fullmatch/LAN/Internet/8realclients/Windows/fullauth/generalreset mở.


<a id="cp156"></a>
## CP156 — host EOF/Bpromotion, manualacceptance và AUD047

Baseline CP155/[PR226](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/226) head`52a16da`/
merge`2b9249e`; [contract](../../PROTOCOL.md#direct-host-eof-acceptance),
[acceptance table](../../PROTOCOL.md#local-demo-acceptance), [PLAN](../../PLAN.md#current-order).
Sourcefirst CP155currentcombinedrecipe, CP050/053serviceclose/promotion, CP107/129known107/slotclear/
chief/110 vàhostUI20286 provenance. No newnativebody/codec/consumerRE. Đối chiếuK6hostdeparture chỉra
rằng guestEOFkhôngthayhostEOF, và cảhaiEOFkhôngthaynormalUIBack. TCPnegativeCP148source vẫnđúng,
không tựbiếnthànhđiềukiệnmớiK6a đểlặp run. Artifact/sourcebinary/DB/pack vàfrozenproducer giữ.

| Run local/research/codex/ | Giây/exit/raw | run.json SHA256 |
|---|---:|---|
| direct-contest-host-eof-r1/cycles | 305.992s / 1 / 991 | `eb5cf6cf219737c5afc3bf2abdc11cc00ead574f336d0da92028f95292112090` |
| direct-contest-host-eof-r2/cycles | 417.374s / 0 / 1295 | `77e511b38eb2804373f554ae8d0cbda28ae0b229680eb6cbb775a701b69b4685` |

r1partial: R1normalending, R2clocks/oldUDP nhưngA199±20 chỉ159↔224/229/20pulse, chưaFire124R2/hostfault.
`partial-observation` giữcoordinates/time/error/nofault. r2copy chỉtwoapproach tolerance20→30; assertion
Fireunclaimed/actualrace/twoactual124/onewinner/nativeeffects giữ, không force actor hoặcclaim/winner.
Không suy vìr2pass mà oldUDP/inputtiming causalfixed.97inputs verify từngrun/cleanup/seal đầyđủ.

r2full:50hostEOFchecks sau2normalcycles/Ready/currentoldUDP. ActualA118ID3owner0/row3col3/power1,
ownedPID163/prefix/netns/comm revalidated; killcòn5,113133827s, directserviceEOF42,465609ms sauafter-signal.
Job3round_closed/no158/noA17,19; B nhậnthirdforfeit175180/native107slot0ID1→0/110promote_B/chief1;
3PNGReadyA/B/Broom chỉKRGuest/chieflabel/Start đãxem/hash ởvisual-review. NoStart4/killedAreceipt/
UI20286/fullleave/3normalmatchesclaim.26nativeentity195flagchecks/575peerassertions4windows/
84contestBwinnerr1/Awinnerr2/43first2cycles/14captureclaims/19exactoldUDPchecks+12currentapply/
24loadinggate/23lifeagreement;2460actor/372peer/132UDP/16life/2758flag+2730array native-model riêng.
1328loadingassertions gồmrawhash/actual99, không cộng modelcounts thànhgate hoặcworldauthority.

Saucleanup mới sửa tools/export_probe_profile.py:--host-battle-exit kèmknownmode+dependency5aliases.
2exporttests/24hostpolicytests pass,46realfixturehash/servicepreflightpass, snapshotfinal-tool-source pinSHA;
đây làdesign/toolimplementation, chưa currentnativeUI20286. Server41sourcefiles vẫngiốngCP155/148;
237fullserverchecks reuse,24relatedtests vừa rerun đểkiểm knownmodeprereq. No runtimeproducer/configchange.

AUD047 đọc core docs→authority/currentcontracts→evidence vớiraw/report/config/source/tests; localmanual
criteria/draft ởk6a-acceptance-audit-r1 pin nguồn, canonicaltable ởPROTOCOL (khôngtracker/planmới).
Sửa authoritynavigation quá cũ và nhãnfullschema-before-anydemo, giữunknownscope; updatePR226merge/
EOF-vs-normalBack/gapactive, khôngđónggatebằngscript. Counterqua156/kế160, recordcompletehistory/raw giữ.
CP156Codexselfcheck/waivedreview/chờPR; tiếp knownnormalhostUIBack integration, khôngRE lại107/110.


<a id="cp157"></a>
## CP157 — current host UI exit và transient life trước rank

Baseline CP156/[PR227](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/227) head`b0daf57`/
merge`fddf81f`; [contract/giới hạn](../../PROTOCOL.md#current-host-ui-exit),
[phép đo tiếp](../../RESEARCH.md#gap-contact-state), [PLAN](../../PLAN.md#current-order).
Raw/copy/producer ở `local/research/codex/direct-contest-host-ui-r1/` và `-r2/`; không source binary/DB/pack change.
Tái dùng CP129 fixture/consumer/UI button, CP156 exporter và currentdirect recipe; không newnativebody RE.

| Run | Giây/exit/raw | run.json SHA256 |
|---|---:|---|
| direct-contest-host-ui-r1 | 76,699 /1/115 | `c203b966691ef711fcbf8d8ecdb63583f2d6b89c05f412da99a7b77316713d69` |
| direct-contest-host-ui-r2 | 440,692 /0/1489 | `0bea8370b7d5245c698d26e94e74891f1cbacedbe5125739f02716feec8004b8` |

R1 giữ partial: firstB root screenshot timeout5s trướcRound1; cleanup/disconnect không là nativefailure.
R2 chỉ freshcopy/namespace/relocation; cùngprofile/97frozeninputs/timeout5s, không causalfixclaim.
Profile so CP156 chỉ thêm host_A_202_86/idle_A_to_B +5knownaliases; không đổi fixture cũ/terrain/network/
30s/2s165/loadinggate. Runtime cleanup/netnamespace extinct/raw seal verified trước analysis.

`host-ui-verification.json`:54checks nối actual A20222/86empty/SHA/currentadmission tới nativeAstage11/
lobby91full50/B175180107slot0ID1→0/110chief1. Hai channel sống tới completion, không kill/EOF/fakepacket.
Năm ending202 khác reject membership; verifier đầu chỉ dự kiến4, giữ sourceđầu và analysis-corrections,
assert exact set5 sau raw inspection. Hai PNG manualreview/hash A lobbyWAITING1/BchiefStart cóởvisual-review.
84contestBwinnerr1/Awinnerr2/44cycle/594peer4windows/19exactold10currentapply/24loading/26entity195reset;
2652actor/384peer/132UDP/18life/2758flag+2730array modelchecks và354nativepacketmodelcases tách runtime.

**Không che failed proof:** inherited `verify_life_agreement.py`/`.log` giữ FAIL B/R1, protected56→0→56→1.
`observe_life_boundary.py`/`life-boundary-observation.json` ghi đúng20checks/finalrankagreement, không
đổi strictchecker cho pass. Zero41,121868ms/firsttrap→zero266,754171ms; second56 stackquaB4EAA6/E668B2,
state0 sharedcallerB51CF4 chưa exactapply. Reuse CP104/105 knownUDPoverwrite; đo đúng currentwindow
là bước kế, không suy resurrect/oldround/gamebug hoặc nâng gate từ nativeending/UI success.

CP156/227 đã merge; CP157 self-check/reviewwaived/chờPR. Audit AUD047 qua156/kế160. History/evidence
complete giữ nguyên; không Start4/rejoin/3normalmatches/fullmatch/K6a/LAN/Internet/8clients claim.


<a id="cp158"></a>
## CP158 — grouped trap/counter windows và exact current-frame controls

Baseline CP157/[PR228](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/228) head e3032f1/
mergef44fd66; [contract/causal limits](../../PROTOCOL.md#current-trap-udp-counter),
[PLAN](../../PLAN.md#current-order), [natural question](../../RESEARCH.md#gap-contact-state).
Source-first reuse CP104/105/145/canonical counter1935–1939/cacheE66680-prefix/B4E6F0: không newbodyRE.
Raw/copiedbinary/producer ở `local/research/codex/` theo tênrun; sourcebinary/DB/pack giữ.

| Run | Giây/exit/raw | run.json SHA256 |
|---|---:|---|
| trap-udp-window-r1 | 437.564/0/1492 | `e1ae9010b735f72b1da5b8f0f6b03008da3b7e1d27e6ac28681cf3bf6bec484b` |
| trap-udp-preroll-r2 | 437.381/0/1492 | `63c5d5624d4020e912e924d3fee4790c9ac46c3991cc881c08af5b1266685bd1` |
| trap-udp-current-replay-r3 | 432.647/0/1496 | `dcbb49ce4981ef8bf584715316483860b8c4ab334860acd119cc7c1c614beae5` |
| trap-udp-counter-r4 | 222.477/0/861 | `17ba0cd9d8c398b1d15eba4cea6e2e6a794699ac4a203a4bece0909904b2700f` |
| trap-udp-counter-gap-r5 | 229.872/0/852 | `acc8f3825d64c76da5fdfc02cc90d6a3b53a3cda3a27a3d1518a88d037d67253` |

6193raw/5×97frozeninputs/cleanup namespace extinct/profilebytes same CP157. R1–3 có2normalcycles+
thirdUIforfeit riêng mỗi run; R4/R5 stop sau firstnormalreturn, không ghép kết quả thành cùng một game.
`trap-window-verification`: first56 71checks/27ingress16apply/14APIjoin2missing; preroll159 166checks/
54ingress40apply/39join1missing/18neutralbeforetrap22trapafter. Rawstate0CP157 không bị backfill.

R3 exactcurrentneutral đếnBAPI sau trap/noapply, không gán dropbranch chỉ từ absence. R4
`counter-control-verification`:24checks actualkey56pass/incoming152stored156→E66884/drop/noapply.
R5:27checks key56pass/incoming140stored157/gap17→actual13apply/protected56→0; zero stack đi known
B4EAA6/E668B2, fresh56apply khôi phục, actualdeath1. Đoạn061,280385ms; 24nativegettermodelchecks riêng,
hai view native175entry loser1/other0. `verify_trap_counter_controls.py` dùng actualframe/hash/IPUDPchecksum/
actual99ports/BAPIwireSHA/key/counter/stack, không encoder tự sinh oracle. Source/status phân biệt trongcontract.

PCAPkv-a chỉ originalEthernet; replaykv-b có exactsenderbufferhash+nativeBAPIwire/source. Initialchecker
đòi hai fullEthernetcapture đã FAIL và được giữ source, sửa claim/recipe scope minh bạch, không sửaraw.
R1 firstchecker đòi mọiAPIjoin cũng FAIL: ghi14/2 thay vì che missing; preroll39/1 vẫn tách giới hạn.
R5 `counter-getter-verification` dùng copiednative52A350/modelcounterreset, không Winegamecall/write.

Controlledlargegap chứng minh capability của byte heuristic đã phục hồi, không nguyên nhân naturalCP157,
không resurrection/gamefix/auth/fullmatch/K6a/LAN/8clients. Tài liệu/currentnext đã chuyển productionservice
baseline, không delay165/replay chủ động; naturalcause giữ open theo trigger. AUD047qua156/kế160,
Codexselfcheck/reviewwaived/CP158chờPR; lịch sử complete và artifacthash trước giữ nguyên.


<a id="cp159"></a>
## CP159 — unwrapped production service, two outcomes và pinned preset

Baseline CP158/[PR229](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/229) head8e3eca1/
merge259282b; [contract](../../PROTOCOL.md#production-baseline), [PLAN](../../PLAN.md#current-order).
Run `local/research/codex/production-baseline-r1/`:364,493952518s/exit0/1197raw/96frozeninputs,
runSHA `01deaf0aecf06434c2df7b5e26b6230035ecf0d3efa3f04cbd5f8dd461764dd8`. Cleanup/netnamespaceextinct/rawseal
đầyđủ; copiedclient/profile/terrain/30s/loadinggate giữ. Source-firstCP157profile/native2cycles vàCP158
counterlimits, khôngbodyRE. Controller bỏ holdwrapper/activeoldUDPmeasurement, service argv đúng module
`server.probe_service`; không measurementstate/holds/replays. Khôngđổi production/nativebinary/DB/pack.

75productionpickupchecks: actualFireR1Aonly/R2A+B/Awinboth/secondconsumedreject; one165/item/view/
full12/errors0/onlyApowerdelta/reset hai key. Broadcast delayobserved0,2508–0,42968ms không SLA.
44cycle/594peer4windows/384copiedpeergetters/12actualC2Sclaimjoins/16loadinggate/26entity195reset;
2214actor/2364flag+2340array/306nativepacketmodels/23strictlife+16lifegetters tách source/runtime/model.
Native56→1 loser/other0 trước175 cả2view/2epoch ởrun này, không phủđịnhCP157failedproof/controlCP158.
BomID1/A/power2 rồiID2/B/power1; hai native175302/1803/1106/currentkey/stat/195seedreset/actual95Ready.
Không thirdforfeit/Start3/normalhostUIexit trongrun.4PNGmanualreview: AStart rõfocused4/native110,
Awhiteoneimmediatefinalframe, BReady/guestReadybadge; không suy persistentUIbug hoặcStart3success.

Saucleanup thêm `server/profiles/two-client-battle.json`: syntheticpolicy/47relativefixturepins, khôngbinary.
Tool `--preset two-client-battle` chỉ rebasepath/verifyhash, rejectdrift vàsymlinkescape, khôngrehashinputđổi.
4exporttests pass (2cũ+2pin/path), real47hashes/productionpreflight/exactobjectequality pass; source snapshots
ởfinal-tool-source pinSHA. **Chưa** native run từ exportedfile CP159, không đổi claim/key/consume cho pass.
Server41sourcefiles vẫn giữ,237tests reuseCP148; newexportchanges tested, không rerun toànserver vôcớ.

CP159reviewwaived/selfcheck/chờPR; periodicAUD048CP160 bắtbuộc với smoke exportedpreset/actualStart3.
K6a/fullmatch/auth/originalauthority/Windows/LAN/Internet/8client/generalreset/sharedgeometry/CP137UI giữmở;
không đónggate từ assertioncount/cửa sổ/encoder haygộprun.


<a id="cp160"></a>
## CP160 — committed preset native, Start3 và AUD048

Baseline CP159/[PR230](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/230) headb084233/
merge2a7332c; [contract/authority](../../PROTOCOL.md#exported-preset-runtime), [PLAN](../../PLAN.md#current-order).
Run `local/research/codex/exported-preset-smoke-r1/`:394,801002246s/exit0/1313raw/98frozeninputs,
SHA `e14fb36edb70bdee013b7b506c5f45090b6f2a82cff4bd12925cd22f64565c21`. SourcefirstCP159profile/native/
export47pins, CP110/156knownStartinput vàclock; khôngnewbodyRE/production/nativebinary/DB/packchange.

Actualargv dùngexportedfile/presetexactobject/productionmodule, nohold165/activereplay.26smokechecks:
2normaloutcomes/reset/Ready→oneactualA104/third12199/barrier30s/clocksboth/render+8, noBomb/noaccepted17,19.
Thirdactivekhôngthirdnormalmatch; cleanupEOF/abort3saucompletion358,7ms giữ, khôngforbidmọiabortloghay
coicleanup làtesthostfault. Initialchecker forbadeanyabort FAIL giữscript; correctedtimeboundary explicit.
75pickup/10actualclaimsjoins/44cycle/689peer4windows/24loading/26entity195reset/23strictlife;
2184actor/444peer/2758flag+2730array/16life/321nativepacketmodels táchsource/runtime/model.
ActualFireAonlycả2key/oneeffect/view/onlyApower/reset; inheritedcount1then2checker FAIL, source/loggiữ,
useplanned1or2criterion/immediatedelivery observational, khôngthaysource/guard cho pass.

4fixedframes pinvisual-review; Start3+8cả2view map/actors/items/timer1:50 đãxem. AStart3thật giải usability
run này, không claimwholeUI/resetall/CP137fixed. `room-title-observation` reassembledactual81titleKorean
khácguest91KRRoom; CP016/028/076nativeknownstring/blob/session reused,3native81framer/str/CRCpartial
cases full67. Khônggánmọifield/create81semantics từ số; dynamic91titleecho/runtimefix là nexttask.

AUD048manualcore→authority/currentcontracts→RESEARCH→evidence→raw/reports/profile/producer/PR đãđọc;
sửaauthorityCP156/hold2s/UIpending, currentPR230/presetproof/nexttitlegap, khôngsửahistorycomplete.
Counterqua160/kế164. K5/K6aassessmentgiữmeasuredprofile/source/environment/controls riêng vàknown
peereditablecounterlimit/naturalCP157unknown; khôngđónggatebằngscript/positivecount/merge.


<a id="cp161"></a>
## CP161 — current room title và production hai vòng

Baseline CP160 [PR231](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/231),
head `a70351439adda5bd97b4d66c2c3a27cbd6f04e17`, merge `a588e9530f712ebb2d53a1eef93e3b079c3f50c0`.
[Contract/giới hạn](../../PROTOCOL.md#current-room-title), [PLAN](../../PLAN.md#current-order).

| Run dưới `local/research/codex/` | Giây/exit/raw/frozen inputs | SHA256 run.json |
|---|---|---|
| room-title-echo-r1 | 99.580380489/0/367/100 | `c9d797df4c5afb8e7a7ee3c95fb999bead7fa892d27802a503b8228316408686` |
| room-title-integrated-r2 | 368.298314404/0/1192/102 | `507e2cfc0fb20fef758e6e64490b0eca48685d0b4a37fd628fff9fa0e2255300` |

Recipe: `prepare_room_title.py`/`wine_probe_room_title.py` và
`prepare_room_title_integrated.py`/`wine_probe_room_title_integrated.py`; copied exe/hash/build tại
`runtime-preflight.json`, frozen profile/production source/isolated two-host topology, owned cleanup
và namespace extinct tại `raw-seal.json`. Không sửa source/config giữa run hoặc source binary/DB/pack.
Phân tích `verify_room_title_runtime.py`, `run_title_native_actual.py`, `analyze_title_integration.py`.
Reuse CP016/028/076 actual81/two strings/row91/session/87/context, không new body RE.

Mỗi run24 title checks,3 actual81 native valid/CRC/partial,6 actual91 native initial/dynamic cases;
actual title13 inline15 và16 heap31. R1 proof UI ở `before-guest-back.png` hai view; final B đã Back,
không gọi final đen là parser lỗi. R2 guest Back/rejoin và hai normal outcomes/reset/Ready:75 pickup,
45 cycle,461 peer/4 windows,10 claim joins,16 loading gate,26 world reset,23 strict life;2256 actor,
300 peer getter,2364 flag+2340 array,16 life và303 native packet model cases tách nguồn/model/runtime.
4 ảnh pin tại `visual-review.json`, tên trùng A/B; no Start3 hoặc third match ở r2.

Giữ các lỗi phân tích ban đầu: blob decoder đã bỏ compression byte, verifier pickup cũ hardcode winner A
trong khi actual r2 winner B, discovery command `-t .` không phù hợp namespace package. Corrected parser/
actual winner assertions và247 server+4 export tests đạt; không đổi raw/service/guard để pass. R1 shell
đã launch sau postassert sai, được ghi minh bạch trong `analysis-corrections.json`; native framer/blob/
CRCpartial đã đạt, không có packet fix. `frozen-preset-runtime.json` và `preset-provenance-update.json`
giữ exact preset trước/sau provenance-only update; operational profile không đổi, raw seal không sửa.

CP161 tự kiểm/review waived/chờ PR, AUD048 qua160/kế164. Known A/B/empty-password/single row và32 byte
là policy; general room/full auth/original authority/Windows/LAN/Internet/8 client/K6a chưa nâng.
Phần next chỉ ở PLAN: input Manner/Free; lịch sử CP160 pending title là trạng thái tại run cũ.


<a id="cp162"></a>
## CP162 — actual team input, ACK/return reset và mode1 hai vòng

Baseline CP161 [PR232](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/232),
head `7f2bf84cd7d1c7bb9dbc9e7e5184b42a8aeda63e`, merge `593bdc0e48ffb9c742d4a624357a6d8fc376bb6c`.
[Contract duy nhất/giới hạn](../../PROTOCOL.md#current-team-mode), [PLAN](../../PLAN.md#current-order).

| Run dưới `local/research/codex/` | Giây/exit/raw/frozen | SHA256 run.json |
|---|---|---|
| team-mode-free-r1 |109.603290609/0/353/102|`1ab82c7b82014a5ff29f135832b77fe15e0daf724769b79a27550d4eb5f6c047`|
| team-mode-manner-r2 |110.005519714/0/344/102|`ed0bea080059db937bbc3d5a17c0defbb9ee67f363acf449660d33e3771a1e91`|
| team-mode-integrated-r3 |444.902234357/0/1179/106|`2bb407bf3d29c2463be4c883d5bcfb20e850892698f10a42eef8ea37b62aee3b`|
| team-mode-reset-r4 |388.062478881/0/1214/106|`3fd7959d786b4062fe8aefb532ac003ed8d5c4d40cbda98bf0e8c73bedf4a814`|

Prep/controller pairs `prepare_team_mode_free.py`/`wine_probe_team_mode_free.py`,
`prepare_team_mode_manner.py`/`wine_probe_team_mode_manner.py`,
`prepare_team_mode_integrated.py`/`wine_probe_team_mode_integrated.py`,
`prepare_team_mode_reset.py`/`wine_probe_team_mode_reset.py`. Mỗi recipe/preflight/raw-seal ghi command,
source/copy/profile/fixture hash, owned cleanup/namespace extinct. Không source/config dùng trong run đổi
khi đang chạy; source/DB/pack gốc bất biến. `reuse-question.json` pin nguồn cũ/câu hỏi/gap/measurement.

Reuse CP147 native selector/balance, CP016/076 typed81, CP017DB29A0, CP028817820, CP022E64950;
VN lobby/oc_a-room và legacyKR chỉ reference. `verify_team_mode.py`:7/10/33/40 joins theo run;
r3 **failed mode persistence** ghi riêng với actual0/write/caller, không sửa raw để pass. R1/r2 chỉ setup
và dừng trước guest Back; r3/r4 actual2 outcomes/Ready, nhưng chỉ r4 giữ Manner qua cả hai return.
Actual81native3 cases/run và91native3/3/6/6; r3/r4 actual86/87/18021 native cases/run, source/runtime/model tách.
R4 75pickup/46cycle/670peer4windows/14claimjoins/16loading/26entityreset/23strictlife;2598actor/432peer/
2364flag+2340array/16life/309nativepacket models. Tên phòng retained/nativeUI proof cùng raw.

`analyze_team_integrated.py` reuse verifiers đã sửa actual winner từ CP161, chỉ khai thêm allowed mode
profile delta; không hardcode winner hoặc đổi tiêu chí gameplay. `analysis-corrections.json` giữ orchestration
missingPYTHONPATH/prep-wait và r3 control; REA assembly DB2FE0 timeout450s được giữ, không dossier mới thành công,
provider đã close. `visual-review.json` pin Free/Manner checkbox và finalr4 hai view; A whiteStart một ảnh
không Start3/persistentbug proof. `frozen-preset-runtime.json`/`preset-update.json` giữ provenance và phân biệt
r4 CP161export+policy với chính file preset mới; không đổi47fixturepins/rawseal. 257server/4export checks.

CP162 Codex self-check/review waived/chờ PR; AUD048 qua160/kế164. No8actualclients/fullmatch/K6a/
Windows/LAN/Internet/originalauthority. Next balanced8 Start theo PLAN, không mở lại mapdump/guardRE.


<a id="cp164"></a>
## CP164 — committed mode-preset partial/input blocker và AUD049

Baseline CP163 [PR234](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/234),
head `260a652b41124f0a1f3b14ef9f613334a9aff9ac`, merge `c5c1bba76965042a1cba4b3f2dc88107045c3742`.
[Contract/giới hạn](../../PROTOCOL.md#mode-preset-input-blocker), [PLAN](../../PLAN.md#current-order).

| Run dưới `local/research/codex/` | Giây/exit/raw/frozen | SHA256 run.json |
|---|---|---|
|mode-preset-smoke-r1|194.085240319/1/674/106|`7d57ad3a48bc882be97054045e7e9e00f9aaddcd4d487ba5bd33647ed1628732`|
|mode-preset-smoke-r2|183.089188757/1/692/106|`dce2994e5551805f9b62bc6d7f4b71dcafeb7f03e93e4f1b8ebbdeb766a1f1fb`|

Prep/controller `prepare_mode_preset_smoke.py`/`wine_probe_mode_preset_smoke.py` và
`prepare_mode_preset_smoke_retry.py`/`wine_probe_mode_preset_smoke_retry.py`. Recipe/preflight pin
source/copiedexe/profile47fixtures/controller; source/config không đổi giữa run. Private namespace,
actual serviceargv/exportedfile, copy/prefix/display/host riêng, no Nexon/default route, owned cleanup
và namespace extinct tại raw-seal. Sourcebinary/DB/pack bất biến.

R1 alignmiss20freshneutral samples159↔234 quanh199±30/pulse45ms. R2 chỉ pulse15ms vẫn159↔234
6samples rồineutral3s deadline; không tăng tolerance/timeout, không phát fakeinput/report/nativegamewrite.
Hai run actualManner81/86/B87/Back/rejoin/firstnative121129/clocks/selector1, **chưaFire/bom/outcome/
Ready2/Start3**. `partial-observation.json`, `partial-verification.json`18joins/run và12native81/86/87cases/run
chỉ xác nhận phần này; getter-model120/36cases từ snapshotread-only riêng. Verifier full-room setup cũ
đúng là failed predicate noerror0 của partial; không nới hoặc gọi fullpass, partialverifier riêng khai scope.
`analysis-corrections.json` giữ giới hạn/control. `server-source-reuse.json` xác nhận production/preset
không đổi từCP162/257server/4exporttests, không rerun vôcớ. R1unused analysisproducer giữ như recipe,
không báo có báo cáo smoke đầy đủ. Source-first CP082/094/123/150/156 cho phép đo input tiếp, không codecscan.

AUD049 periodic đọc core→authority→research→evidence/rawCP161–164, sửa stale title/input/PR/currentgate
navigation; K4currentmodepromotion aliasjoin_A chưa hỗ trợ/đo, K6a/fullmatch/Windows/LAN/8actualclients
chưa đóng. Record/audit/evidence cũ giữ nguyên. CP164 checkpoint blocker+audit/tựkiểm/reviewwaived/chờPR,
không plannedsmoke đạt. Auditqua164/kế168; source tiếp việc duy nhất ởPLAN.


<a id="cp165"></a>
## CP165 — native15step batch, observer control và committedpreset Start3

Baseline CP164 [PR235](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/235),
head `bb5457908c66f7795a1f914af5d5f210acbea7ff`, merge `38284668d7e60c10b30b58bc8c36f2aa50b458d8`.
[Contract/giới hạn](../../PROTOCOL.md#input-batch-gating), [PLAN](../../PLAN.md#current-order).

| Run dưới `local/research/codex/` | Giây/exit/raw/frozen | SHA256 run.json |
|---|---|---|
|input-step-probe-r1|168.428151904/0/680/107|`ea0ddd81dc3afa10b921fef4342ebb9c7cbb5ff42f446f7cb665daff723b9972`|
|input-lite-probe-r2|166.695487614/0/686/106|`e8032722fb621183e6ed8be005e95cf8e2e6037f9388e0f5f52ec1bd6605696d`|
|mode-preset-smoke-r3|397.659234656/0/1366/107|`6083d6a5a090bd3d4cfafaf2df8fde93157756f5a8ccf272fb1f824f54d4d32f`|

Sourcefirst raw/hash/build tại `input-pacing-r1/reuse-question.json`, `body-match.json`4targetedbodies
vàIDAroute/tick/move-loop JSON; sourceDB copy mới/read-onlysource/hash, MCPmissingida_nexus record riêng,
existinglocalIDAbatch fallback không chỉnhconfig. VN gdb dispatch/gates chỉ cáchđo, không portoffset.
Prep/controller `prepare_input_step_probe.py`/`wine_probe_input_step.py`,
`prepare_input_lite_probe.py`/`wine_probe_input_lite.py`,
`prepare_mode_preset_light.py`/`wine_probe_mode_preset_light.py`. Recipe/preflight/source/profile/copy
frozen trước run, namespace private2hosts/no route/NSSfiles-only, actualmoduleargv vàcleanup extinct/rawseal.
Không sourcebinary/pack/DB gốc hoặc code/config giữa run bị đổi; measurementBPenabledstates thay đổi
trongwindows đã khai, không gamefunction/data/count/timingwrite.

R1 `batch-verification.json`35relations/4524gettermodels: actualA batch15steps5px=75 từ159→234,
WndProckeyup trướcbatch, state23→15 xảy rasau loop nhưng trướcvirtualE2D130. Bposition234 khôngexactUDPapplyproof.
R2 `lite-verification.json`17relations/24getters: same45ms/nativecountsmostly1/position159→174,
112BPstates/viewrestoređúng; explicitobservereffectcontrol không nguyênnhânmọiinput/naturalCP157/vanilla law.
`timing-comparison.json` giữ19pairs/20actions R1CP164 (1missing),6/6R2; initialcompletepairassumption sửa
khôngbịa coverage. Heavytracecũngperturbtiming, không gọi 15làmọitick/clientcap.

R3 actualexportedcommitted47pinprofile/directproduction/nohold/noUDPintervention,2normaloutcomes/Ready→
actualnormalStart3/12199/clocks/render8s/nobomb/no3rdoutcome.162gatingchecks/12windows/2viewready+restore/
sametolerance20attempt/3freshneutral; all4native165/view vàpeer/Bomb/life/reset checks giữ.26preset/42mode/
24title/3native81/6native91/21modeframer/75pickup/46cycle/651peer4windows/14claim/24loading/26entity/
23strictlife;648actor/420peer/2758flag+2730array/16life/330packetmodels riêng. `visual-review.json`4frames,
Start3+8bothviews timer1:50/map/2actor/items; title16heap31. Someeventsduringalignmentgated/unobserved,
khôngwholeinstructiontrace; sourceproduction/preset unchanged/reuse257server4export exacthash.

`analysis-corrections.json` giữproviderfailure/19pairs/virtualcausalityscope/initialgatingverifierwrongkey
schema (round/created_ns correctedfromraw, initialsourcegiữ). Không đổi raw/guard/acceptancecho pass.
SnapshotCP165 tựkiểm/reviewwaived/chờPR; đãmergePR236/060ff22. Gapjoin_AđượcCP166giải; K4vẫnmở, auditcountercurrentxemmanifest;
K6a/fullmatch/Windows/LAN/Internet/8actualclients/originalauthority/naturalCP157 giữmở. Next chỉPLAN.


<a id="cp166"></a>
## CP166 — currentmode idlehostBack/promotion/rejoin control và fix

Baseline CP165 [PR236](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/236),
head `0d1081aa21b79365b1c8a75ec417c8a30163e737`, merge `060ff22971640493532d14722c261295146c04af`.
[Contract/giới hạn](../../PROTOCOL.md#current-mode-host-rejoin), [K4 rà acceptance](../../PLAN.md#k4-evidence-review).

| Run dưới local/research/codex | Giây/exit/raw/frozen | SHA256 run.json |
|---|---|---|
|mode-host-rejoin-control-r1|116.869338442/0/418/106|`faca380b05cc4d3cf65b0109cefa5f024a68ee8c2a3546e4c7186c2ee9f324cf`|
|mode-host-rejoin-fixed-r2|164.328823171/0/668/108|`1be58003c95ccac4302b93741dfb2af4a1b9550f16dc97ef7658f428d26d8a44`|

Mỗi run giữ reuse-question/build/source locator, cycles-recipe, runtime-preflight/server snapshots,
launch-exit, raw-seal và membership-verification. `prepare_mode_host_rejoin.py` tạo thư mục/copy mới,
`wine_probe_mode_host_rejoin_{control_r1,fixed_r2}.py` làcontrollerfile riêng/frozen; chỉ idlepath,
không lặp2cycles. User/net/mount/PID namespaces, veth/private2hosts/NSSfiles-only/no default route;
ownedclient/wineserver/service cleanup vànamespaceextinct. Source/config không đổi trong mỗi run.
Copy e9bb4766…/static660447…/modOa08950…/base400000 giữbuildCP165, khôngsourcebinary/pack/DBchange.

514control/815fixedchecks gồm toànrawhash và actualC2S86/82/hash/admission, native107clear/110chief,
91variabletitle/fullcursor, continuousS2Cseq/ACK87full7/modegetter1/ReadyA/StartB/12199/clock/render8.
`native_mode_host_rejoin.py` dùngretainedKRframer/getters/CRCpartial:9control/21fixedmodels riêngruntime.
`visual-review.json` pin2finalframes đãđọc: map/timer1:50/hai actor tại ôkhácnhau/cảhai view; không là
gameplay/wholeworldoracle. `analysis-corrections.json` và `verify-initial.py` giữ fail của giảđịnh eventreports
rỗng: fixed có2startupsubtype39, không17/18/19/21/bom/pickup/result; raw/production không đổi.

`mode-host-rejoin-tests.txt`265server và `mode-host-rejoin-export-tests.txt`4export pass. Lệnh discovery
đầu dùng -t . bịimporterror (server namespacepackage), exporterimport saiđườngdẫn; sửa invocation sang
README discover -s server / discover -s tools -p test_export*.py, không sửasource đểnétest.
RunCP166control/fixedseparate khôngfullmatch/chỉclock; CP167sauđócóhai currentmodecycles. SnapshotCP166
tựkiểm/reviewwaived/chờPR; đãmergePR237/7598cc9. Khôngđónggate; auditcountercurrentxemmanifest. Bước tiếp duy nhấtPLAN.

[CP166 validation bổ sung](cp166-validation.json): docs-check banđầu gặp snapshot cũ thiếu PR236;
snapshotGitHubđãđồngbộ và docs-check-final/issuecheck pass. Filefail và recordcomplete giữnguyên;
không coi outputfail cũ là pass hoặc đổi raw đểlàmcheckqua.
Bản CP166 sinh lầnđầu chưa hợp lệschema trướccommit (2kind/path và1methodenum); bảncandidate lỗi giữlocal,
chỉ sửa khai báo bảnmới. Báocáo bổsung trên trỏ correction/hash; CP001–165/raw/findingkhôngđổi.


<a id="cp167"></a>
## CP167 — currentmode Bhost two cycles và rare165 observer retention

Baseline CP166 [PR237](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/237), head
`09759d0bed6545de322c3f3d1f545b52a729aa2c`, merge `7598cc9ec1929f0c8ac59fb2127c5fbaf32ea5ce`.
Runs đượcchuẩnbị trên09759d0; sourcebytes giữquaPRmerge, không đổi source/config giữa run.
[Contract/giới hạn](../../PROTOCOL.md#current-mode-promoted-cycles), [PLAN](../../PLAN.md#current-order).

| Run local/research/codex | Giây/exit/raw/frozen | SHA256 run.json |
|---|---|---|
|mode-promoted-cycles-r1|378.305974972/0/1216/109|`29f008dcc2c405ec49c3c700d3793bbdf297c270a1d59ce235d54e2a0870deac`|
|mode-promoted-cycles-r2|369.832789235/0/1198/109|`287230cdc8b478a66a49a517a21749e23c51e397ff7a0acc7616772c666c98ae`|

Mỗi run có reuse-question/sourcepins, cycles-recipe/controllerfile, runtime-preflight/server snapshots,
launch-exit/raw-seal/namespaceextinct; bảncopy/prefixdisplay/privatehosts/no outbound giữ recipeCP166.
`prepare_mode_promoted_cycles{,_r2}.py`, `wine_probe_mode_promoted_cycles{,_r2}.py` tạo file/copy mới,
không override sealedinputs. R2chỉkeep3rare165BPs trongalignment; `kept-pickup-probes.json`24actualarms,
`alignment-gating-verification.json`162relation+restore/samecriteria, `production-pickup-verification.json`
75relation/all4callbackview/actualclaims/nativepower; R1failedfull4check và `observer-gap.json` giữnguyên.

Retainedanalyses: actor-fields/nativegetters, cycle-boundary(46), endpoint/peer/prepost/CRC/current4windows,
captureclaim/hotloading, world-reset(26)/195flags, lifeagreement(23)/16getters và nativepacketmodels
R1309/R2312; `promoted-mode-verification.json`205/210 +24modeframer/gettercases/run trên actual81/86/
87join_A/180. Model dùngcopiednativebytes/CRCpartial, không encoderoracle hoặc runtimegamecall.
`server-source-reuse.json` khóa toànserver*.py vềCP166265tests/4exporttests; khôngreruntestsunchanged.

`visual-review.json` mỗi run pin2finalframes đãđọc; R2Bbuttonwhite1frame giữunknownfunctionalStart3,
R1coloredkhôngbackfill. Initialanalysisdriver subprocess thiếuPYTHONPATH, script/logfail giữlocal rồi
chỉnh invocation; missingBomb16checkkhôngđượcnới đểpass. R2read-onlyprobe whitelist giải measurement
coverage trong recipe, không retrospectiveproof cho missingcallbackR1/vanillatiming.

SnapshotCP167tựkiểm/reviewwaived/chờPR; đãmergePR238/e48e10b. CP168đoBStart3/audit sauđó;
raw/whiteframe/controls giữscope riêng, currentnext chỉPLAN. Không fullmatch/K6a/LAN/8client hoặc sourcebinary/DB/packchange.


<a id="cp168"></a>
## CP168 — boundedinput controls, actual BStart3 và AUD050

Baseline CP167 [PR238](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/238), head
`5ad76eb288bf1e1f2603bbbeedec6c5b9a71accf`, merge `e48e10baf3abc8a82cc576c7550f68462088b5aa`.
Runs chuẩn bị trên5ad76eb/sourcebyteskhôngđổi qua merge; khôngsource/configchange giữa run.
[Contract/giới hạn](../../PROTOCOL.md#current-mode-promoted-start3), [PLAN](../../PLAN.md#current-order).

| Run local/research/codex | Giây/exit/raw/frozen | SHA256 run.json |
|---|---|---|
|mode-promoted-start3-r1|166.910034079/1/645/109|`cd933049bd38c3d534a65c73a7bcd1daa82c49ab28dd7be66b7c4b5f480d6a45`|
|mode-promoted-start3-r2|166.746944231/1/658/109|`348fe1aaf2ab333380469b552601f72283f0d864342061ae82afbb86a9e8c13f`|
|mode-promoted-start3-r3|405.945854372/0/1374/109|`9264d3e31cc74de60f115deaa7fd1730f9d0a435da1e08a802b6f87ce7b2e45d`|

Mỗi root giữ reuse-question/build/sourcepin, preflight109/runtime-source, cycles-recipe/controller riêng,
launch-exit/raw-seal/ownedcleanup/namespaceextinct. R1/R2partial-observation giữ noFire124/noBomb/noresult/
noStart3. R2contest-input-snapshots/contest-getter-verification:16samples/view/192copiedgetters,
khôngglobaltick/pxs; R3reuseCP150 stop-on-actualpickup principle/newcontrollerdesign đượcghirõ trước run.
Source/game/profile/target/tolerance/deadline giữ, fixed0,30s khôngđượcreclassifypass; observednormalkeyhold
0,617006739s/0,595376959s/1claimmỗivòng, no2nativecontendersproof. VN/reference chỉmethod đãpin,
không nativebody mới hoặcportopcode/offset. Binary/pack/DBnguồn giữ nguyên.

R3feedback-input-verification58checks/2newcontestwindows+exactrestore; 28armskeep3rare165/14groups,
12alignment/162originalcriteriarestorechecks; all4native165/view/75pickupchecks. SourceactualB104/
121129/993rdbarrier/render8:exported-runtime-verification26, promoted-mode213/24nativecases;
cycle46/life23/16getters/map195/world26/4peerwindows/nativepacket321 vàcaptureclaims riêng.
visual-review3frames đãxem: BcoloredStartbeforeclick vàthirdmap/timer1:50/2actor cảhai+8s.
Chỉthirdactive/noBomb/noaccepted17,19, khôngthirdnormalmatch hoặc retroR2whiteframecause.

gate-scope-review xácnhậncurrentpresetnohidden_items/revealed_hidden={}3epochs; CP068069knowncombined
profile là nguồn táidùng chonextintegration, không backfillcurrentrun/dropalgorithm/K5. AUD050manual
core→evidence/raw fixescurrentnavigation/knownpromotion/reset/pendingPR vàscopefloor-vs-hidden;
[auditrecord](../checkpoints/audits/AUD050.json). Auditqua168/next172; recordcomplete/oldfail giữbấtbiến.
CP168tựkiểm/reviewwaived/chờPR, khôngđónggate. Currentnext chỉPLAN, không tạo plan theo phiên.

AUD050 đínhchính nhãnCP167: missingindex16 là **Bomb/kind0/capacity**, khôngRoller/kind2.
Raw/typedfields/hash/count/fail giữ; recordCP167bấtbiến có nhãnsai, contractCP167đãghierratum.


<a id="cp169"></a>
## CP169 — currentbasic copyprofile: hai reveal và ba nativeeffects trong một epoch

Baseline CP168 [PR239](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/239), head
`902994eb90e3f6e00b3a131977183fb27d132b06`, merge `2ee6556bc3d5a429124ec7691c77d68eb5c2ea71`.
[Contract/limits](../../PROTOCOL.md#current-basic-hidden), [PLAN](../../PLAN.md#current-order).

| Run local/research/codex | Giây/exit/raw/frozen | SHA256 run.json |
|---|---|---|
|current-basic-hidden-r1|194.314060126/0/742/108|`f260c8b82eae380ed07478dd52980c5daef6568a446e05157cd6b33cac01ba8f`|
|current-basic-hidden-r2|191.128464361/0/723/109|`472925b8c0b72f86fa69ee9a75a6253b098a4682a7f0582015ce6567e6ee14f4`|
|current-basic-hidden-r3|339.989344987/0/1253/109|`309be133a7f12b22bfb95b5deb4b886af0931e9902907e28267c0e45f7fadcf0`|
|current-basic-hidden-r4|192.983476401/0/721/109|`6e8a5858eda20753c31433f65430d7ea762f5405a71fe0cf8b66086a5fb218a2`|

prepare_current_basic_hidden.py composes exactknownCP069floor/catalog with46unchangedaliases/native5823/
constructor beforelaunch. runtime/feedback/complete/retry preparers tạo thư mục/copyprefix mới; eachroot
giữreuse-question/sourcepins/profile/recipe/controller/preflight/runtime-source/launch-exit/raw-seal.
Private2hosts/nooutbound/ownercleanup/namespaceextinct, source/config frozen; sourcebinary/DB/pack giữ.

R1accepted17/27 vàR2accepted17/28 làpartials; nativehidden27/28store/revealproof ởcảhai, khônggộp
3effects. R3startup7connections/noBchannel4/noclock/tillbound, sourcefrozen/exit0/noerror khôngpass;
startup-observationkhônggánrootcause. R4freshcopycùnglogic, missing28feedbackrow3/col2 hit; missing27
fallbackkhônghit. R2controller-before-preflight-correction.py giữgeneratedindentcandidate, corrected
trướclaunch/refreeze; lầnlaunchR4trướckhiprepdone thiếurecipekhôngspawnclient, đợiprepdone rồi chạy.

hidden-integration-verification R1/2/4:167/170/176checks/nativeFixedBox→reveal/typed124142/hash;
R4hidden-effects-verification122checks actual3callbacks/2view/full12/errors0/onlyAeffectdelta/lookup27/28
samepointerreveal. native_actual60capturedframer/gettercases, native-fixture3cases vàconstructorpreflight
riêng; actor-fieldssecurewrappergetters không runtimegamecall. alignment-verification4viewphaseacks/
2groups giữ3rare165+lookup/exactrestore; disabledchronology excluded. visual-review2finalmapframes
đãxem, khôngreset/originalauthorityoracle. 265server/4exporttests táidùngexactsourcehashCP166;
production/committedpresetkhôngđổi, profileghépchỉởlocal/nativeproof metadata commit.

CP169tựkiểm/reviewwaived/chờPR; AUD050qua168/next172. Nextresult/reset/twoepochs củaprofileghép
chỉPLAN; khôngK5/K6a/fullmatch/LAN/Internet/8client hoặc randomdrop/fullvalidatorclaim.


<a id="cp170"></a>
## CP170 — currentbasic copyprofile hai nativecycles/reset/Ready

Baseline CP169 [PR240](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/240), head
`9266b7829468915de2c0609097bbe30b1c70c070`, merge `387f93600afb220b95b403abf82450a4432bee47`.
Runđượcchuẩnbị/chạy trên9266b78; bytesruntimegiữquaPRmerge, khôngsource/configchangegiữarun.
[Contract/limits](../../PROTOCOL.md#current-basic-cycles), [PLAN](../../PLAN.md#current-order).

| Run local/research/codex | Giây/exit/raw/frozen | SHA256 run.json |
|---|---|---|
|current-basic-cycles-r1|388.409514005/0/1316/109|`e716d951e3644b5355b19aa28d5cb440d95bcc8c6af1e0e62223f3ee0eacd398`|

Rootgiữreuse-question/CP069168169pins, exactprofile/recipe/controller/preflight109/runtime-source,
launch-exit/raw-seal/namespaceextinct. User/net/mount/PID/private2hosts/nooutbound/directproduction,
sameManner/title/Bhost/2realcopies; no sourcebinary/pack/DB/production/defaultpresetchange.
`prepare_current_basic_cycles.py` chỉlặpknownCP169route trênround2 +CP069normalfinalSpace +CP168
ReadyAStartB/return2Ready, khôngfake14212417/19/104 hoặcgamecall/actorwrite/tolerance-timeoutrelaxation.

basic-cycle-verification1614relations incl1316rawhash/sourceclaims/2Box+2hiddeneffects/3statsbaseline/
all195flags/native159pointer→158processedID/reset/normal175180/Ready. Nativeframer/typedreader174cases,
securefields516models, life-agreement23/16getters riêng; mode-gating4actual180[0,1,0]/12nativecases và
boundednativegetterreads1, khôngnewsettertrace. alignment7groups/14viewacks/exactrestore/rare165lookup;
disabledchronologywindowsexcluded. visual-review2finalroomframes đãđọc: AReady/Bchief, BStartwhite1frame;
chưaactualBStart3 vàkhôngpersistenthangclaim.

analysis-driver-initial.py/analysis-driver.txt/actor-fields-initial.json vàanalysis-corrections.json giữ
initialSTAT_SNAPSHOTjointrùngdoCP069parsergáncollisionvàlastGEN165context. FinalreuseCP168parsercó
event/phase rồi lọc đúngSTATevent/phase, khôngbỏraw/getter/cursorcheck hoặcđổilệchstatthànhpass.
`server-source-reuse.json` pinCP166265server/4exporttests exactsources, khôngrerunchangedtests.
CP170tựkiểm/reviewwaived/chờPR; AUD050qua168/next172. Nextoptionalpreset/export/actualBStart3 chỉPLAN;
khôngfullmatch/K5/K6a/LAN/Internet/8client/Windows/originalauthority/generalresetqualification.


<a id="cp171"></a>
## CP171 — actual optional basic export, hai vòng/reset và BStart3

Baseline [preset PR242](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/242), head
`b805ed5c2e6f63718c5bde785a23957c0814ab1d`, merge `87e7175f163c0ceaf39a03bc8a6fac535781f8b9`.
[CP170 PR241](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/241) merge `cbcd15ddd912faf8f337376cf3468455fb53cc49`.
[Contract/giới hạn](../../PROTOCOL.md#basic-preset-start3), [thứ tự tiếp](../../PLAN.md#current-order).

| Run local/research/codex | Giây/exit/raw/frozen | SHA256 run.json |
|---|---|---|
|basic-preset-start3-r1|402,663072569/0/1443/156|`7003baf1592d8dd9e273f527f02ead9d6a0064cef23141848a46046324eaf301`|

`prepare_basic_preset_start3.py`, `wine_probe_basic_preset_start3.py`, `run_basic_preset_start3.py` và
root giữ actual exported profile/recipe/reuse-question/preflight/source copies/launch-exit/raw-seal.
Profile SHA `bf1f99989e45a4b5e47e831165ad1f43f296418253388d8ba8338532d665afee` bằng exact CP170.
Không service/nativebinary/DB/pack/configchange giữa run; no fake104/142/124 hoặc gamecall/statewrite.
Lần gọi runner trước khi chuẩn bị xong thất bại vì thiếu preflight, trước mọi spawn; giữ prelaunch note.

`basic-cycle-verification.json`:1741checks/2cycles/3effects/stat+195+eligibility/key reset;
`exported-start3-verification.json`:417checks/3actualB104/barrier3/clock thêm9,146s/5alignmentgroups/
10viewphaseACK/rare165lookup giữ/exactrestore;180 native packet/552 field cases và life23/16getters riêng.
`visual-review.json`: đã xem hai ảnh cuối Battle, không thay native trace bằng ảnh.
`analysis-corrections.json` và initialscript/log/fields giữ parser attribution lỗi khi Space snapshot dùng
event/phase pickup cũ; thêm CAP_SPACE marker, không bỏ raw/native getter/uniqueness checks. Cleanup
abort sau completion0,401806744s được phân biệt, không gọi fault control. Có retained quoting/log-schema
driver errors trong phân tích, không lỗi native runtime; final reports là kết quả độc lập đã đối chiếu.

`basic-preset-delivery/` giữ6exporttests/exact47pins/servicepreflight/checks/PR242 exacthead/merge;
`server-source-reuse.json` chỉ tái dùng CP166265server tests theo unchanged hash, không lấy4exporttests
cũ làm validation cho exporter mới. `hidden-cell-admission-reuse.json` đối chiếu8actual142 với current
clearedcell/commit, nguồn CP068 policy/test/terrain; guard chưa triển khai, không originalauthority.
CP171 tự kiểm/chờ PR, review waived. AUD050 qua168, audit kếCP172; gate/fullmatch/LAN/Internet/8clients mở.
