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
