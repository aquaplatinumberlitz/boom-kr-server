Các mục CP dưới đây là snapshot theo run/checkpoint lúc ghi, gồm trạng thái PR/review và bước tiếp
lịch sử. Trạng thái hiện tại đọc ở [README](../../../README.md), thứ tự ở [PLAN](../../PLAN.md#current-order);
PR là nguồn trạng thái merge. Không dùng một dòng “chờ PR” hoặc next-step cũ làm trạng thái hiện hành.

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


<a id="cp172"></a>
## CP172 — current terrain-cell142 guard và native basic positive

Baseline [CP171 PR243](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/243), head
`defd8a1a123db0e9db091272e7c7af3d18f4fcbe`, merge `89f228b2af177e9008ea99df35e6caf28737f577`.
Service guard/test là working-tree delta được snapshot/pin trước run; source/config giữ nguyên đến seal.
[Contract/giới hạn](../../PROTOCOL.md#hidden-cell-admission-current), [PLAN](../../PLAN.md#current-order).

| Run local/research/codex | Giây/exit/raw/frozen | SHA256 run.json |
|---|---|---|
|hidden-cell-native-r1|406,400966485/0/1438/158|`419b67bf0a643fbaf7ffa33a320ed53b2217d4168f8268441f41c417667302cb`|

`prepare_hidden_cell_native.py`, `wine_probe_hidden_cell_native.py`, `run_hidden_cell_native.py`, root
profile/reuse-question/recipe/preflight/runtime-source/launch-exit/raw-seal giữ delta và cleanup.
Profile SHA `bf1f99989e45a4b5e47e831165ad1f43f296418253388d8ba8338532d665afee` như CP171, helpers/
controller chỉ rebase outputpaths; không sourcebinary/DB/pack/gamecall/actorwrite/fake142/104/boundschange.

`basic-cycle-verification.json`1736checks/180nativepacket/552fieldcases, life23/16 riêng;
`exported-start3-verification.json`419checks/barrier3/clock9,229s/4alignmentgroups/8viewACK/rare165+lookup
enabled/exactrestore; hai final Battle images đã xem trong visual-review. CleanupEOF sau completion,
không intentionaldisconnect. `terrain-admission-verification.json`/`verify_admission.py`:52joins,
8accepted/8outsidecatalog/4duplicate actual142; knowncells root1/3/currentclearedcell/commit/fullroster.
`admission-analysis-correction.json` giữ initial all16accepted assumption bị bác bỏ; guard/raw không đổi.

`hidden-cell-admission/server-tests.txt`270suite và targeted-tests-final5tests; discovery-error và initial
test helper-import duplicate count giữ lịch sử, không gọi28importedtests là5newtests. Native run không
inject negative reports/oldUDP; unit policy controls tách khỏi native compatibility. Fullfreshness/auth/
distance/old142saulegitclear/no-terrain weakscope được giữ rõ. CP172 tự kiểm/chờ PR; review waived.
Audit AUD051 qua172, kế176; K5/K6a/fullmatch/dependencies/LAN/Internet/8clients không đóng.


CP172 checker gặp pin repository source CP166 khác working source sau guard mới. Metadata/report
`local/research/codex/source-pin-publication/` chứng minh CP166 record bất biến và99.707bytes/SHA
`81b0c8d38cfd79cdcce163adf91c764f1343827b54c7b9b16b318bca5d8149a6` khớp original publication
`09759d0bed6545de322c3f3d1f545b52a729aa2c`. Checker source fallback hẹp đòi đúng declaration/blob lúc
công bố record v2;34schema/history tests (gồm6newcases) đạt. Không sửa record complete/raw/hash/gate;
v1-only hoặc blob chỉ xuất hiện về sau không được dùng để pass. Quy tắc ở [checkpoint docs](../checkpoints/README.md).


<a id="cp173"></a>
## CP173 — basicBowner/peer hai chiều: hai partial và same-run positive

Baseline [CP172 PR244](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/244), head
`9c922fee956a68f00c74235d3b7fd8b4ec36f19d`, merge`3706a6ec288a93588b5e4f8fc8ea2da6d4602561`.
[Contract/giới hạn](../../PROTOCOL.md#basic-bidirectional), [PLAN](../../PLAN.md#current-order).

| Run local/research/codex | Giây/exit/raw/frozen | SHA256 run.json |
|---|---|---|
|basic-bidirectional-r1|380,083940772/1/1127/156|`8e053ca029ccab806be97aa86a162a3bc572e4393c64ef10b2c2fc259bb25a65`|
|basic-bidirectional-r2|502,907843300/0/1418/157|`0df8b5e74b12c2bd73c3d8c2248b0e55b9c4e173d6ac2f5765dcca9ac308371d`|
|basic-bidirectional-r3|462.819517789/0/1470/157|`01a9d53da8c83b345d1e2823099bc102dab9b8663e73926f223def0000268109`|

prepare/wine_probe/run_basic_bidirectional*.py và roots giữ sourcefirst/recipe/exactprofile47pins,
preflight/runtime-source/launch-exit/raw-seal/namespaceextinct. Production/profile cùngCP172, no binary/
DB/pack/statewrite/gamecall/forgedrequest/nativebodychange; freeze đếncleanup. Inputwindowdesign
R1ADown→R2AUp→R3ARight; all20pulse/.045s/tolerance12/30sloading/OS.25/window3s giữ.

R1partial-verification:4nativepeerphases651checks/420getters/Bownerbothkeys/native159158samepointer;
chỉ1outcome/reset, R2A80,399/alignmiss/beforeFire. R2basic1728/Start3560passed, nhưng AUp window
không changedtarget trên nativeendpoint nênkhôngbidirectionalpass; initialassert/log giữ. Khônggộprun.
R3 chỉ chuyển hướng/axis A sang hànhlangfloor, negativepositivecontrolsLeftRight, khônghạcriterion.

R3 1780firsttwo/548Start3/575peer/372getters; 234nativepacket/834securefieldmodels, life23/16; 14alignmentgroups/rare165lookup/exactrestore; barrier3 6.385199635s, clock thêm9.189975736s.
Native all195 kiểm sequential3bomb/epoch/IDs1..6/ownersB,A,A; hidden/stat/eligibility/key reset và
threeeffects hai view. Hai final Battle images đã xem; profile default giữ, noactiveoldUDP/EOFfault/
thirdoutcome. Server270tests reuseexacthashCP172, không rerununchangedsource; copiednative models
táchruntime/Pythonpolicy. CP173 tự kiểm/chờ PR; reviewwaived/AUD051qua172/kế176. Khônggate/fullmatch/
Windows/LANInternet/8client. Nextcurrentbasic BhostEOF/ownedpendingBbomb/Acleanup/promote/fence chỉPLAN.


<a id="cp174"></a>
## CP174 — current basic pending BhostEOF/Acleanup: observer partial và positive

Baseline [CP173 PR245](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/245), head
`3c4fd1375101b345d118886f8e2a91264a76afa7`, merge`0d4da2ede263e67eb11dafb25502fcbce373cea8`.
[Contract/giới hạn](../../PROTOCOL.md#basic-host-b-eof), [PLAN](../../PLAN.md#current-order).

| Run local/research/codex | Giây/exit/raw/frozen | SHA256 run.json |
|---|---|---|
|basic-host-b-eof-r1|323.310706336/1/1089/157|`345d8774468e0839cd90464793554210ca9ce2d73a1dd0d143883a2a46462760`|
|basic-host-b-eof-r2|503.191808517/0/1466/159|`4b7b0c63341eca88877f35b0cf5e89367063c92a9005182690bdfa7da88830bb`|

prepare/wine_probe/run/analyze_basic_host_b_eof*.py, roots giữ exactprofile47pins/sourcefirst/recipe/
preflight/runtime-source/launch-exit/raw-seal/namespaceextinct. Profile nhưCP173; source/client/DB/
pack/production/guard/inputbound/fuse khôngđổi. R1partial-observation ghiBsyncFileNotFound/noACK/
R2-Areturnsetup, chỉ1normaloutcome và3peerwindows, không tớiBfault. check_align_gate_race.py/R2
gate-read-control3cases: oldexists→missingread lỗi/noACK, fixed directread/catchFileNotFound restore,
malformedJSON vẫn lỗi. Oldexceptionfilenamekhôngcó: causalhazardcandidate, khônguniversalrootcause.
R2observer-verification exactsavedBPstates/4rare165lookup enabled,34viewphaseACK/17groups/noerror.

R2 1778firsttwo/55EOF/632peer/408nativepeergetters; 243packet/882fieldmodels; life23/16.
host-eof-verification55checks nốiownedPID/startticks/prefix/netns/comm/exe/command/mappedimageSHA,
actualB118/ID7/159both→signal→actualTCP EOFbeforefuse→A-onlyforfeit/native107slot1id2clear/110chief1/
openAtransport/room→originaljobdeadline round_closed/no158. Remaining5,117498130s; 2normalresults
prefixbeforethirdStart vàthirdforfeit tách. verify_first_two_basic/peer/life analyses reuseCP173,
chỉđổiboundary đểthirdoverride khôngđược tínhnormal; khônghạeffect/nativecriteria hoặcbỏraw.
visual-review đãxemAroom/host/slotBempty/coloredStart; khôngStart4click, deadBkhôngnhậnforfeit.

admission-scope-gap.json giữ source/method/hostmodel: currentAalive/Bclosed/hostA replacement_allowed
False, oldAclosed/BhostTrue; ROLE_SLOTS C0trùngA0. Đây là emulatordesign/source gap, chưaactualC
connection/identity/slot1contract. NextactualCcontrol/boundedvacant-slotpath chỉPLAN. Server270tests
reuseexactunchangedhashCP172; no newnativebody/opcode/legacywireport. CP174selfcheck/chờPR,
reviewwaived/AUD051qua172/kế176; nogates/fullmatch/Windows/LANInternet/8client/accountresume.


<a id="cp175"></a>
## CP175 — actual admission control, Cslot1/AStart4 và preReady consistency

Checkpoint CP175 gom control/implementation/native/visual delta; không tự đóng gate. CP174 đã merge [PR246](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/246),
head `a441215093911ee73e01badbb5e3a27a0962ca30`, merge `281ca375218f26e3240f1172630db265b4000b27`.
[Contract/giới hạn duy nhất](../../PROTOCOL.md#a-survivor-admission), [phép đo](../../RESEARCH.md#gap-disconnect-replacement),
[thứ tự](../../PLAN.md#current-order). Codex solo; review waived; audit kế CP176.

| Run dưới `local/research/codex/` | Kết quả và hash run.json |
|---|---|
| A-survivor-control-r1 | Partial trước fault/C: alignmiss x80/y399;715 raw,222.793s; `b217aecf93af4a2ac3c3da964b8eb2fff82d59a1d0cea80b9d436a7e09df6a9a` |
| A-survivor-control-r2 | Actual C→ordinal9/replacement_not_ready;1459 raw,494.542s; `6c8adff39c6a0982d628e721742a0fe9a0f3d501cf509ec2d8104fb900f2c153` |
| A-survivor-positive-r1 | Prelisten policy preflight từ chối battle_exit; chưa client,8 raw/0.390s; `47b04af84b0bd391d1b971b5927b8054fa818423f81d618fd014abc9f0d79e82` |
| A-survivor-positive-r2 | Native Cslot1/ID3/Join/Ready/AStart4/clock;1691 raw,573.565s; preReady color discrepancy giữ riêng; `974fa4133611c48dd3ee2fa4bfc8c5a03cedbc74c9d32742022a335f5a908c3b` |
| A-survivor-positive-r3 | Tổng thời gian hết, chưa C auth/Join/slot110 mới;1598 raw,609.271s,exit0 nhưng không completion; `f4aea2d72f2ac6465ea504adf8700d59acc4526ec1c0c3a1574bdce24c78fb8f` |
| A-survivor-prestart-r2 | Native CJoin/Ready/AStart1/clock; Asetup110 trước C95/ảnh A đúng đội, CpreReady đen/unknown;586 raw,184.868s; `7717bb65d998b44bcd1b1552336baa25b08fae71d977ea9119601a25e8d1b1bf` |

Mỗi run có `cycles-recipe.json`, `runtime-preflight.json`, `launch-exit.json`, `raw-seal.json` sau cleanup;
raw list/size/SHA giữ tại local. Binary Wine copy SHA `e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef`,
source binary/DB/pack không đổi. R1→R2 control chỉ thêm OS alignment về hành lang đã có, không nới20pulse/
45ms/tolerance12/loading30s/fuse6s. Positive opt-in dùng8bodypins/instance slot map; profile C khai báo1/10
để thử Start, không coi đó là gameplay stats đã kiểm. R3 source chỉ thêm slot110 sang A nhưng chưa chạy tới nó. Control prestart tiếp theo giữ fixture,
đổi entry recipe và Ccapacity2/7; không coi là đối chứng runtime chỉ đổi một biến.

Control `admission-control-verification.json`:1488 checks/1459 raw; native baseline1771,
243 packet cases/876 secure fields, peer632/EOF55. Positive R2:2003 firsttwo,
249 native packet cases/912 secure fields, peer670, admission43. `visual-review.json` ghi bốn ảnh đã
đọc; `C-capacity-gap.json` gắn initial slot1 stats với secure native getters/report SHA.
`analyze_A_survivor_positive.py` tách cleanup abort/Ready marker bị xóa khỏi captured C95/104;
report lỗi đầu giữ nguyên, không sửa raw để làm pass.

`A-survivor-positive-r1/source-recovery-report.json` và `source-recovery/` giữ đúng bytes recipe/harness/source
R1 so SHA preflight sau khi helper chuyển R2. Trong khôi phục, preparation script không dùng bởi runtime
bị tạm ghi bytes R1 rồi khôi phục ngay đúng SHA R2; không đổi harness/production/profile/config đang
chạy. Kiểm hash đầu/cuối không là giám sát filesystem liên tục. R3 dùng helper tên riêng để giữ locator.
Không đổi preflight/raw/seal hoặc complete checkpoint cũ. Snapshot issue có thời điểm, không là plan.


Focused recipe `prepare_A_survivor_prestart_r2.py`/`wine_probe_A_survivor_prestart_r2.py`/
`run_A_survivor_prestart_r2.py`:300s total,120s C phase/loading30s giữ; B guest EOF trước trận đầu,
Acleanup/chief, fresh C/12 transports/ownslot1/ID3/Join87/Ready95/A104/121full/clock/153 và8s quan sát.
Không hai vòng AB hoặc forfeit trong run này. `native-analysis.log`, `admission-positive-verification.json`,
`preReady-slot-verification.json`, `C-initial-capacity-verification.json`, `visual-review.json` giữ phép kiểm/giới hạn.
Ảnh `B/` lúc này là display của C trên host B đã rời; nhãn `Start4` kế thừa tên file, actual epoch1/key12345.
Không dùng tên file để nâng epoch. `C-before-ready-contact.png` giữ sáu frame đen đã đọc, không bỏ ảnh xấu.
`A-survivor-prestart-r1` chỉ preparation lỗi shadow variable, chưa launch/preflight; không tính thành runtime.

Profile C2/7 ở focused run khớp initial capacity2 native getter; maximum7/reservation/C118 còn chưa kiểm.
Hai view map active sau Start không thay AC outcome/reset/peer movement. CpreReady render unknown.
CP175 hoàn tất ghi chép không đồng nghĩa merge, gate hoặc fullmatch; PR/Git giữ trạng thái tích hợp.


<a id="cp176-in-progress"></a>
## CP176 — run đơn C bom đầu (giữ anchor cũ)

`local/research/codex/A-C-bomb-r1/`: entry prestart CP175 giữ nguyên, thêm normal CSpace0.4/Right0.8
sau8s clock. `cycles-recipe.json`/`runtime-preflight.json`163pins, total300s/Cstartup120s/loading30s/
fuse6s cũ; source binary/DB/pack nguyên vẹn. `raw-seal.json`:197.705936s/626raw/exit0/cleanup0PID,
runSHA `b630ca5ff1e3890ecdf1f59e0fe5453f92cb6070bdcf6c07a2f4c161269854cf`.
`C-bomb-verification.json`:23 checks; `native-actual.json`:33 actual/CRCbad/partial native cases;
`chain-summary.json`: A/C cùng ID1/entity159/blast158 một lần. `native-analysis.log` và `-r2.log`
giữ lỗi analyzer cũ, `native-analysis-r3.log` pass; không sửa raw/counter/checkpoint CP175.
`actor-fields.json`/`C-initial-capacity-verification.json` giữ getter theo context gốc, không tự gọi
context MAP_CLOCK_STARTED là snapshot sau blast. Native/codec models là loại kiểm riêng với Wine.

[Contract/giới hạn](../../PROTOCOL.md#a-c-first-bomb), [thứ tự tiếp](../../PLAN.md#current-order).
Run đơn không AC outcome/reset/peer; hai vòng được đo riêng bên dưới, không ghép phạm vi.
Capacitymaximum/fullmatch/gates vẫn mở; CP176 gom cả hai run và AUD052, audit kế CP180.


<a id="cp176"></a>
## CP176 — hai A/C normal basic cycles và periodic audit AUD052

Baseline [CP175/PR247](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/247), head
`1b640d68fa64e84aaef645cc75c26fc6ca09ca8a`, merge `957b37ac0451ac9cf32d65fe6348e063b10add5e`.
[Contract/limits duy nhất](../../PROTOCOL.md#a-c-basic-cycles); [thứ tự tiếp](../../PLAN.md#current-order).
`local/research/codex/A-C-cycles-r1/`:466.655250245s/1079raw/163frozen/exit0/cleanup0PID,
runSHA `ad0f81c3c4ba816f524e08b4315ba9787e8eb8bbf3eec618ddf8a3e035dda407`.
Recipe `prepare_A_C_cycles_r1.py`/`wine_probe_A_C_cycles_r1.py`/`run_A_C_cycles_r1.py` dùng prestart
entry CP175, đúng fixed-profile/C2/7/binary e9bb…; source production không đổi, không ABgame/forfeit.
Ahost/Cguest role mapping/control/log/ACK C rõ, topology host B khác identity C; pointer/PID ownership
C được kiểm trước input. Max600s cho observation2cycles, Cstartup120s trước clock đầu; không đổi
loading30/fuse6/input20/.045/tolerance12/neutral3/restore3 hoặc ép actor/gamecall/wire.

`basic-cycle-verification.json`:1391checks (1079raw+312core),228 native packet cases/810fields;
`endpoint-peer-verification.json`:708checks/4phases; `peer-getter-verification.json`:456 nativegetters;
life agreement23/getters16. `oracle-reuse-comparison.json` so cùng312nonrawcriteria với oldAB2003−1691;
đổi current A/C/host/epoch/cleanup boundary tường minh, không giảm chỉ tiêu physics/effects.
`native-analysis.log`/`-r2.log` giữ lỗi AB-only analyzer; `-r3.log` pass. `visual-review.json` giữ cả ảnh
return2 đen lẫn finalReady hiện phòng A/C; StartA trắng dưới hover không tự là fail/pass Start3.
`audit-before-index.json`/`audit-before/` giữ docs đã đọc trước sync; audit đọc README→PLAN→PROTOCOL
(authority/newcontracts)→RESEARCH→evidence/raw/report/code/images, không chỉ chạy checker.

Cùng hai keys12345/12346: C/A/A IDs1–3,4–6, power1/1/2; actual142 hai view/3A124/native165,
A56→1/C0 trước cả175/180,195/stat/key/eligibility reset và CReady95/110/AStart2/Ready2. Không AC
contest/oldUDP injection/disconnect/thirdStart, maximum/genericreset/fullauth/Windows/LANInternet/8client.
Complete record/audit không tự chứng nhận gate; [AUD052](../checkpoints/audits/AUD052.json), kếCP180.


<a id="cp177"></a>
## CP177 — exported replacement preset, vacancy capacity và C0/C1 native delta

Baseline CP176/[PR248](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/248) merged
`bd34ed426ef0d5f863c2f47a97d98e95c7ea6602`; Codex solo/review waived, CP177 PR pending trước
tích hợp. [Contract/giới hạn](../../PROTOCOL.md#replacement-profile), [gap](../../RESEARCH.md#gap-disconnect-replacement),
[thứ tự duy nhất](../../PLAN.md#current-order). Không đổi source/config trong run; cleanup/seal
hoàn tất trước đổi việc. Source binary/DB/assets giữ nguyên; default/old flag giữ behavior.

Reuse CP051 C0 native stats1/1/5 và identity3, CP175 C1 binding/initial2/slot110, CP176 actualC118/
159158 và hai normal ACcycles. Chỉ đo policy delta/export/currentbasicC0 còn thiếu; tránh chạy lại
hai trận hoặc tìm lại codec/consumer. Known121/118/159/158/getter/frame oracles tái dùng, không VNwire.
`reuse-question.json` từng run pin nguồn/known/mức chứng minh/gap/minimum; `profile.json` được export
từ preset47pin, chỉ A/B capacity và derived-policy declaration.

| Run local/research/codex/ | Kết quả | run.json SHA256 / thời lượng |
|---|---|---|
| replacement-profile-C1-r1 | Ahost/BguestEOF→Cslot1; native2/1/4, actualC118/ID1/owner1/159158both;843checks=632raw+163preflight+48relations;33nativepacket cases | `6068047d24bf8c5caf532db6069f401360424647714fab84fa7e5149d59c8f50` /207.311767302s |
| replacement-profile-C0-r2 | Bstartup SIGSEGV/jypc.dll tạiVA79b5fa99;6transports/noC/no binding, không pass;215raw/clean | `5037639584dd5512e8ed0f3f716e82cdce86324086ddd51a5717cdacb34d00aa` /69.389981917s |
| replacement-profile-C0-r3 | AhostEOF/Bpromotion→Cslot0; native1/1/5, actualC118/ID1/owner0/159158both;831checks=620raw+163preflight+48relations;42nativepacket cases | `e8fc40174f881e075850395815877ebb0a719a6fac66e9d5938ce7f50086d6f7` /205.812514754s |

Hai positives có48quan hệ ngoài raw/preflight, cả hai actual C95/host104/121/99/barrier/clock và
C native authID3/ownslot/currentkey. `profile-delta-verification.json`, `actor-fields.json`,
`native-actual.json`, `chain-summary.json`, transformed oracle source+hash giữ rõ model/runtime.
`raw-seal.json` verify163frozenfile mỗi run, original6sfuse/loading30s/total300s, exit0/0remainingPID.
Model/getter oracle không là actual gamecall; inputreturn0 không làm proof cho118/entity.

`replacement-profile-C0-r1` preparation-only assertion do overlapping text transform; chưa runtime,
giữ profile/preparation report. R2 sửa cách chọn source span/role; R3 fresh sameconfig đối chứng
startup, không code/binary patch hoặc nới deadline. `startup-failure-verification.json` giữ module/
logSHA; đối chứng pass không xác định nguyên nhân SIGSEGV. Một số nhãn harness C0 còn wording cũ;
verifier dùng actual slot/host/namespace/field, không nhãn. Scripts chuẩn bị/controller/wrapper/analyzer
tên riêng; old complete artifacts giữ nguyên.

284server/7export tests; strict missing/ambiguous/unknown/invalid-declaration, capacity-copy isolation,
oldexplicit/default/admission/Ready/host/round guards và preset equality/relocation/hash drift. Các
test dùng fixture tổng hợp là policy regression; actual exported47pin được runtimepreflight riêng.
Maximum10/7 vẫn operatorpolicy; không exhaustion/reservationcount/survival/peer/result/reset mới.
CP176 AC2cycles giữ riêng, không gộp hai probes thành K6a. Gate/dependencies/Windows/LAN/Internet/
8client/fullauth/general membership còn mở; AUD052 qua176, audit kếCP180.


<a id="cp178"></a>
## CP178 — original Windows control và nghiệm thu identity/static prerequisite

Baseline CP177/[PR249](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/249), merge
`b7972a87eda9abd8d76b7bd6cd39f39385a9e3cf`. [Contract/giới hạn](../../PROTOCOL.md#original-windows-control),
[acceptance do Codex đối chiếu](../../PLAN.md#k0-k2a-acceptance), [next](../../PLAN.md#current-order),
[gap](../../RESEARCH.md#gap-process-patch). Review độc lập waived, không independentapproval;
CP178 pending PR lúc ghi. Gate decision thủ công ở PLAN, không do verifier/sốCP hoặc processalive.

Raw/harness tại `local/research/codex/local-acceptance-r1/`. `reuse-question.json` pin nguồn cũ,
`source-tree.json`187originalfile/1861590835bytes; `guest-copy-verification.json` hai187file fullcopy
fixedCNTFS/DriveType3/khôngmismatch. `guest-control-preflight.json` nativeRT_MANIFEST1 CA524bytes/
SHA`e3880876e4c5bc886a88cb396037a7672139fe35575cff147466e129205e4089` requireAdministrator;
originalsourceCA`a52545c14b1fea423de655c3163d6360b628d61c1cd472b46fdbc8a9ffe15f4e`, NewPatcher
`b59e4dce9ad23c439a7632ce04ecb2eed3481585bd2454100b05c9d7527f5c02`. Source/dump/DB current5verify; VN original116/bce và VNpatchedruntime/bc875 đối chiếu riêng theo REFERENCES, không VNruntime mới.

| Run | Actual roots / selectedETW | control.json SHA256 |
|---|---|---|
| ca-control-r1 | CA1328;349/15447ETWevents;3374checks | `1a2ce14a4b58cf5a8de87a297cde67d950cfd747315dbde314fd501abeb5de79` |
| newpatcher-control-r2 | NewPatcher9656;288/12903;4919checks | `b6916b13be3672f65a326dcce85c0cfa1ea39cd15ef3e2fff6bb1ce86733fbb2` |
| both-control-r2 | NP11136/CA7876;575/16602;4013checks | `2e95bbb120e67c0b7000c6921ffefa9703ac2f0a6fa04dc786090246d39f8fbe` |

`original-controls-verification.json`12314checks chủ yếu file/frameintegrity; actualUTC trongcontrol,
10×2s là nominalsleep có preparation/observeroverhead, không báo thời lượng thật20s. NativePEB32cwd/
token, modulebase/size và kernelEvent1PID/parent/startticks/high12288/elevated1 khớp. NewCALog CA46byte
và modal theo đúngPID, Both tách CA??? với PatcherNexon/embeddedIE. No bootstrap/login/gameplayclaim.

Mỗi run có `process.etl`, decoded selected-PID`*-etw.json`, bridge/eth0PCAP+SHA và local sentinelnegative
actualSYN2/khôngeth0/khôngaccept. `isolation-and-cleanup.json` giữ commands/IPv4/6rules/guard/restore;
`raw-seal.json` pin raw saucleanup, 0owned/externalchild/newdriver, sourcefrozen. Full187source không
sửa; `source-tree` chứng minh đúng dữ liệu được copy, checker dùng external source-evidence manifest.
Capture có Windowsbackground/QMP/SMB, không gán mọi traffic choCA hoặc suy absentTCPsample là noAPI.

Helpers mới tên riêng, nguồn v1/v2/v3 giữ nguyên. `newpatcher-control-r1` preparation-only robocopy
returncode≥8, chưa game launch; fullraw/source giữ, sourcev3 quote logoption đúng/normalize separator.
v2 beforeRestoreDifference có falsepositive nested paths, không native mutation. Analyzer r1/r2 giả
extra blankline CALog, r3 khớp hai dòng thực; không sửa raw hay nội dung marker. REA fullPEtreeblocked
range khôngfile-backed; targetedWindowsresourceAPI đọc được manifest, không sửa/execute entrypoint.

[Windows control tool](../../../tools/windows_control.ps1) và [console helper](../../../tools/windows_console.py)
giữ nguyên bytes source đã chạy; `tool-source-promotion.json` đối chiếu SHA. Source/capture/lease
riêng môi trường vẫn raw local có pin, không gộp thành framework mới.

`manual-acceptance-review.json` có từng criterion K0/K2a/nguồn/giới hạn, [AUD053](../checkpoints/audits/AUD053.json)
đọc source/docs/raw theo scope. K2a reuse nativewriter131340calls, WCPconsumer, CP006/014capture,
CP05776stream/14boundary/5reset/697relations và CP058cursor137/137/135/errors0; không RE lại codec.
Modelprefix/cap/FFFFFFFF/fullmodes/live reconnect vẫn K2 debt. K0 identity/harness và K2a staticprobe
được nghiệm thu; K1/fullK2/K3–K7 chưa đạt. Ba recipes một control không là ba coldstart recipeK1pass.
Tiếp K1source/coldrecipe theo PLAN; source/privateDB/defaultserver không đổi, audit periodic kếCP180.


<a id="cp188"></a>
## CP188 — current basic hai vòng, peer, partial controls, EOF và K5 hẹp

Baseline CP187/a3d0ff1; [PR259](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/259) đã merge
`e57e6283dbfeda3fc30943ece4f4e307404c6ccd`. KRE-005b/[#14](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/14),
Codex solo/tự kiểm/[review waived](../checkpoints/review-waiver.json), không review độc lập.
[Contract/giới hạn](../../PROTOCOL.md#current-basic-cycles-291), [nghiệm thu K5](../../PLAN.md#k5-acceptance),
[decision theo tiêu chí](k5-acceptance.json), [câu hỏi tiếp](../../RESEARCH.md#gap-lobby-identity-followup).

Copy Wine SHA4b781f… giữ đúng CP187; không sửa source binary/DB/pack hoặc game code giữa run.
Hai private virtual hosts có actual A/B và direct production service; mạng không route ngoài,
NSS/hosts riêng, source/config/47 fixture freeze đến cleanup/seal. Profile basic export thêm opt-in291/
nameguard; physics/clock/pickup/report không bị ép bằng gamefunction hoặc memory write.
Rollback dùng recipe cũ và giữ file nguồn/copye9. Mỗi root có recipe, preflight, logs/capture/raw-seal;
SHA/size từng file nằm trong external evidence manifest CP188, không chép raw vào docs.

| Root dưới `local/research/codex/` | Thời lượng / raw / frozen / exit | Kết quả và giới hạn |
|---|---|---|
| `k5-neutral-keyup-r1` | preclient recovery / 12 / 167 / interrupted | Preclient wineboot−15, zeroTCP; recovery12raw/frozen167 sau interruption, không gameplaynegative hoặc nguyên nhân reboot. |
| `k5-neutral-keyup-r2` | 413.403716917s / 1450 / 167 / 0 | Hai normaloutcomes/reset/Ready2/Start3; chỉ A move/bom.16R1neutralwindows đủ3freshstable/OSkeyup0; exactfailedR2branch không chạy. |
| `k6-current-peer-cycles-r1` | 453.875873934s / 1486 / 167 / 0 | Hai normaloutcomes/reset/Start3, bốn A↔B windows, B/A/A bombs mỗi vòng/IDs1..6, hidden27/28+floor17/baeffects,195/stat/key reset/life56→1/other0. Không contest/EOF trong run này. |
| `k6-current-host-eof-r1` | 374.766890422s / 1158 / 167 / 1 | Một normaloutcome rồi alignmiss:R2-A-hidden-row;20stable x80/y399/OSkeyup0, chưa tới fault. |
| `k6-current-host-eof-r2` | 465.448559303s / 1492 / 167 / 1 | CP175entrycolumn120 control qua hai normaloutcomes/reset/Start3 và pendingID7. FileNotFound ownershipimage cũ trước kill, exit1 giữ nguyên; firsttwo được kiểm riêng. |
| `k6-current-host-eof-short-r1` | 160.110170286s / 688 / 167 / 1 | Focusedfirstload/pendingBID1/159both; KeyError game_dir ở controller trước fault, không EOFpass. |
| `k6-current-host-eof-short-r2` | 194.716078338s / 714 / 166 / 0 | Focusedfirstepoch pendingBID1/ownedkill/directEOF/Aforfeit175180/107ID2clear/chief110/room và fence qua due+4s. Không normaloutcomes/Start2 trong run này. |

Đối chiếu nguồn trước đo: CP171 cycle/oracle, CP173 nativepeer/Bowner, CP174 pendinghostEOF,
CP175 entrycolumn và CP187 ABI/nameguard. Vì vậy **không mở hàm native mới**, không RE lại codec,
ItemAttr, luật/stat/wiki hoặc chuyển opcode/offset VN. R1entry fallback thiếu bước đã biết được sửa
bằng exactinput CP175; nó không chứng minh mọi physics/keyrelease cause. R2faultpath và short-r1statekey
là lỗi controller, giữ raw/exit1 và sửa sau cleanup; short-r2 đánh giá path từ exactrecipe/hash trước launch,
giữ toàn bộ PID/startticks/namespace/prefix/comm/command/mappedimage guards, không kill theo tên.

Các script local `analyze_neutral_keyup_r2.py`, `analyze_current_peer_cycles.py`,
`analyze_current_host_eof_r2.py`, `analyze_current_host_eof_short.py` dùng lại oracle KR đã có.
Peer run:1796cycle assertions/234nativepacket cases/834securefields/670peer assertions/432getter models;
life23 assertions/16nativegetter models và identity1829checks. Neutralrun1748cycle/403Start3/97neutral;
đây là quan hệ kiểm đo được, không số tính tiến độ/gate. Firsttwo của EOF-r2 loại riêng epoch3/pathfailure,
không thay error thành pass; focusedEOF verifier không chạy verifier đòi hai normalcycles.

FocusedEOF run SHA`edb46b7d607ad617dacac7c7e231be3828f992ad6ed7b1b819525b75bf6e562d`, kill còn 5.283077328s trước fuse; 932 kiểm gồm raw/inputhash và native cleanup.

Ảnh contact của neutral/peer và survivingA room đã được Codex xem; native trace/capture/getter là proof
riêng. Hai normalending ở một run không bị ghép với EOF/oldUDP/contest ở run khác. Chọn fuse6s/drop/
rank/forfeit/freshness/terrain guard vẫn design, không luật server Nexon đã phục hồi.

[AUD056](../checkpoints/audits/AUD056.json) thủ công qua188 sửa nextK4/codec stale, audit184 stale,
B291pending snapshot và deliveryPR259; audit kế **CP192**. K5 basic local nghiệm thu theo tiêu chí cũ/
authority matrix; K6a **còn currentbasiccontest**, LAN/Internet/bốn–tám actualclient/fullauth/fullcodec/
142freshness/distance/generalreset/allinventory vẫn mở. [PLAN](../../PLAN.md#current-order) là thứ tự duy nhất.
CP188 complete ghi chép trước giaoPR; trạng thái merge nằm trong Git/PR, không sửa snapshot sau merge.


<a id="cp189"></a>
## CP189 — current Roller contest, hai normalcycles/reset và K6a local

Baseline CP188/[PR260](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/260), exacthead
`009505cd788d4181dbd6ca448f97d81d1c011c79`, merge`2ea548bb1c8d0929ed87ae464aeb9511cbaf48f2`.
Codex solo/tự kiểm/reviewwaived, KRE-005b/#14. [Contract/giới hạn](../../PROTOCOL.md#current-basic-contest-reset),
[K6a decision](k6a-acceptance.json), [acceptance](../../PLAN.md#k6a-acceptance), [nextgap](../../RESEARCH.md#gap-lobby-identity-followup).

Tái dùng CP155/159160 oneclaim/oneeffect/productionimmediate165, CP188 basic47pins/nativepeer/life/
reset và CP175navigation; không thân hàm native mới hoặc opcode/layout VN. Source/profile/copy4b/SHA
bất biến giữa run; privatevirtualhosts/directproduction/nohold165/noactiveUDP, freeze→ownedcleanup→seal.
Raw/hash/recipe/inputpreflight/analysis/probes/images ở root local dưới đây; external evidence manifest
CP189 pin từngfile, không chép raw/binary/asset vào Git. Client gốc/DB/oldcomplete/evidence không sửa.

| Root `local/research/codex/` | Giây / raw / frozen / exit | Phạm vi |
|---|---|---|
| `k6-current-basic-contest-r1` | 185.548594027 / 707 / 167 / 0 | Haiactual124/Bwinner/Aconsumed, một165/view/nativeB4→5/A5unchanged; không kết quả/reset. |
| `k6-current-contest-reset-r1` | 197.177912461 / 711 / 167 / 1 | Partial: A thắng/Bconsumed nhưng20nhịpBreturn mới375/đích399; chưa bom/kết quả. Giữ exit1. |
| `k6-current-contest-reset-r2` | 391.085047274 / 1272 / 167 / 0 | Hai normalcontestedcycles/kết quả/room/Ready2; R1hai claims/Awins, R2mộtclaim/Bwins. Cùng4peerwindows/two keys/life; không EOF/oldUDP/Start3. |

Focusedrun dùng lại native124/165/52A350/securechecksum. Giả định analyzer cảhai baseline5 FAIL ởB;
source/log được giữ, đối chiếu CP188 xác nhận A5/B4 rồi sửa đúngper-rolebaseline; chỉ B4→5, A giữ5,
capacity/fire không đổi. Không sửa client/raw/profile để pass. Đây là statfixture, không mọi char/wiki.
Partialr1 controllerreturn vẫn dịch chuyển nhưng thiếuquãngđi trong20nhịp; r2 thêm normalDown.5 trước
finefeedback20pulse/.045/tol12, không nới bounds. Existing6sneutralvariant cho contest-returnsetup có
nguồn CP155r2/CP159, global3s giữ; source-first r1 chỉ ghi3s được qualify, không viết lại record/raw.

R2 actualAselfSpace/IDs1,2/power1/native159158 cùngpeer, A17→19/B21/native56→1/other0before175,
full175302/1803/errors0 và actualReady95/BStart2/nativeclock/reset rồi Ready2. MộtRollerconfirm/view/
epoch; statbaselineA5/B4 trướcR2, A6→5 reset sauR1. B thắngR2 nên **không quan sát B5→4 sau thắng**
vì khôngStart3 trongrun, giữ giới hạn resetvariant. CP188 đã có basichidden3effects/Bowner/Start3;
EOFcurrent control và exactoldUDP CP155 là run riêng, không nối thành cùng-run.

`contest-reset-verification.json`:1553 assertions; `endpoint-peer-verification.json`:632 assertions/4windows,
408nativepeergetters; life23 assertions/16nativegetterchecks. Focusedcontest932checks/2actualclaims.
Hashes/typedfield/nativecallback/protectedgetter models và service logs tách source; không tự tínhgate.
Ảnh contact/finalReady cảhai view đã xem, không suy Start3 hoặc allworld/same-tick từ hình.
`continuity-review.json` đối chiếu nguyên source server/currentprofile/copy-image và phạm vi reusecontrol.

Codex đối chiếu thủ công tiêu chí K6 cho K6a và authority matrix; **K6a đạt riêng samehost/basic**,
K6bLAN/K6cInternet/K7/bốn–tám actualclient/fullauth/fullcodec/generalreset/allitems/maps/originalauthority
vẫn mở. CatalogK6 tổng vẫn open ngoài scopeK6a. AUD056through188, audit kếCP192, không tạo audit/CP
cho từng microtest. CP189 complete trước commit/PR, không giả đã merge; Git/PR giữ delivery.


<a id="cp190"></a>
## CP190 — portable service/fixture và native relocation control

Baseline CP189/[PR261](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/261), headfbf2f688/
merge`1559ad57253369e92a4141d102312ee6904b9cc0`. Codex solo/tự kiểm, reviewwaived, KRE-005b/#14.
[Contract duy nhất](../../PROTOCOL.md#portable-probe-bundle), [CLI](../../../tools/README.md#portable-bundle),
[phần thiếu](../../RESEARCH.md#gap-lobby-identity-followup), [thứ tự](../../PLAN.md#current-order).

Raw `local/research/codex/portable-bundle-r1/`: source-first, exporter/test outputs, hai export riêng,
gói final chuyển thành `relocated bundle/`, verify từ cwd/tmp/Python-I.47alias/43fixturefiles/13module/
59pinnedfiles, toàn fixture/policy/sourcebytes bằng currentbasic đo CP188189 (chỉ path rebased).
`bundle.json` chứa contextcommit trước thay đổi tool; đây là provenance, không giảsource đãmerge.
Gói chỉ local/gitignored; không client/DB/asset hoặc binary được phát lênGit.

`portable-bundle-runtime-r1`: recipe/preflight229pins/690raw/seal/187.247701577s/exit0/namespacecleanup0.
Serviceprocess Python-I/launcher tại relocatedbundle, actualservice_module tronggói và profileSHAstartup
khớp. Hai own291 native128cursor/errors0/name/renderguard1 vàrequest6nameguard; S2C43+3n, realroom/
hostBack/rejoin/Ready/first12199clock. BSpaceID1/power1 rồiownedBkill còn5.197152337s trướcfuse;
actualdirectEOF/Aforfeit175180/native107ID2clear/chief110/room vàjobround_closed/no158pastdue+4s.
1046checks nativeEOF/ownership/frozen/relocation và66identity/sequence/framechecks; nativegetter/
source/runtimepolicy vẫn tách nguồn. Ảnh contact đã Codex xem, tutorialoverlay ởfirstclock không
bằng inputblocked hoặc fullmatch; B đãdừng không được claim nhậnforfeit. Không2normalcycles/Start2/LAN.

Tiền kiểm TCP `k6b-admission-preflight-r1` đã tạo sauCP189 nay gomCP190: actualunchangedserviceaccept
serializedAAAA BBBB đúng8/8, xenABABABAB sai6/8 role/stage. Cảhai ca loopbackrealTCP/ownedsocketsclosed;
đây là policy emulator, không nativeaccount/physicalLAN. Initialmetadata candidate giữ nguyên lịch sử;
source/rawseal/derivedLANargvtemplate/hash nay được pin trong evidenceCP190.

Six meaningfulbundle tests: relocation + deleted syntheticfixture source + actualTCP + SIGTERMcleanup,
source/destination drift/missing/symlinkescape/overwrite/profileoverride đều kiểm;7exportregressions,
294server và39checkpoint tests. Không thay service/nativewire, vì vậy test không nghiệmthu LANgate.
Raw/hash từngfile ở externalmanifestCP190; immutableCP188189 không sửa. AUD056qua188/kếCP192.
K6b/K6c/K7 vàgeneralaccount/142freshness/distance/codec/modes/bốn–tám actualclient vẫn mở theoPLAN.


<a id="cp191"></a>
## CP191 — nghiệm thu LANgiảlập theoowner và BpostwinRollerreset

Baseline CP190/[PR262](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/262), head3748df88/
merge`b7e83f46d4ab834209ef4c293dcbd93a77d72583`. Codex solo/tự kiểm/reviewwaived, KRE-005b/#14.
[Owner directive](../owner-directives/emulated-lan.json), [K6b decision](k6b-acceptance.json),
[acceptance](../../PLAN.md#k6b-acceptance), [contract](../../PROTOCOL.md#emulated-lan-acceptance),
[câu hỏi kế](../../RESEARCH.md#gap-lobby-identity-followup). Chỉ đạo mới chấp nhận hai host giảlập thay
hai máy vật lý; acceptance môi trường đổi rõ ràng, không viết lại complete/rawCP189190.

Reviewread-only `local/research/codex/virtual-LAN-review-r1/` tái dùng nguyênCP189run:
2CA+2wineserver/netns/NIC/IPriêng .2/.3, privatebridge .1, khôngdefault-route/proxy. Typednative99localIP/
dynamicUDPports và153remoteendpoint/key nối1282EthernetPCAPrecords/checksum/tuple/wireSHA→nativeAPI/
keyslot/target/currentprogress4windows/2keys.5183mechanicaljoins; manualgatecriteriareview riêng.
CùngCP189two normalcontestedcycles/life/full175180/Ready2 vànormalAhostBack/nativeBclear/chief/Arejoin
cólog. CP190bundledBhostEOF/Acleanup/fence control riêng, không resumeaccountB hoặc cùngrunvới2ending.
**K6b đạt LANgiảlập/sharedkernel/host theoowner**, không suy physicalLAN/Internet/NAT/relay đã đo.

Phép đo mới `portable-B-winner-reset-r1/`: nguyênportablebundle/47fixture/current291/copy4b/Wine,
chỉBnormalUp nhặt Roller17 ởhai vòng, Aidle. Không fake124/forcewinnerstate/actorwrite/gamecall;
production165immediate, nativeB124/key12345/12346→một165/view/epoch/B4→5/A5unchanged. ActualASpace/
ID1,2/power1/159158→real17/19/B21→native175302/1803/errors0/twoBnormalwin/room/Ready2/normalBStart3.
Nativeprotectedclockstats cảhai view đềuB4/A5 ở3epoch; B5→4 sau thắng được kiểm2lần (Start2 vàStart3),
khép đúng limitCP189, không universalreset/itemlaw. Vòng3active≥8s/noBomb/noThirdoutcome.

Run407.828178758s/1376raw/frozen227/exit0/cleanup0; runSHA`ea3d82878c7d5ce3aed5ff35cc8139f21f4c6b9297d0236aaa956302c0c6336a`.
1714cyclechecks/90nativepacket cases/528securefieldcases;632peerchecks/4windows,69Start3/bundlechecks.
Nativegettermodel/capturedmemory/clock/seq/wire separatefromPythonpolicy/render; images đã xem,
không Start4/thirdresult/contenders/EOF trongrun mới. Không thêmnativebody/codec/ItemAttrRE, source
client/DB/assets/oldrecord/cache giữ nguyên; freezeđếncleanup/seal. Hash từngfile ởexternalmanifestCP191.
CP189/190gate/raw scope lịch sử giữ, quyết địnhK6b mới phát sinh sauowner; script không tựnghiệmthu.
PeriodicAUD056qua188, audit **CP192** kế tiếp; K6cInternet/K7/four-eightactualclient/generalaccount/
fullcodec/auth/142freshness/maps/items/resetvariants cònmở. Bước tiếprouted/NATpreflight theoPLAN,
không bắtchủdựcấp2physicalmachines hoặc gọi môphỏng là Internetthật.


<a id="cp192"></a>
## CP192 — hai subnet routed/Wine và periodicAUD057

Baseline CP191/[PR263](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/263), head a62af1e0/
merge`56a08dd67a5dce858611e3b80e5f588be8fa7477`. Codexsolo/tự kiểm/[reviewwaived](../checkpoints/review-waiver.json),
KRE-005b/#14. [OwnerWine accepted](../owner-directives/wine-accepted.json), [contract](../../PROTOCOL.md#routed-Wine-network),
[gap/next](../../RESEARCH.md#gap-lobby-identity-followup), [PLAN](../../PLAN.md#current-order).
Không yêu cầu WindowsVM/máyvậtlý làm gateblocker; vẫn giữ đúng provenanceWine/copy/patch/source,
không sửa nhãn lịch sửWindowsunknown/notphysicalLAN thành kết quả chưađo.

`routed-transport-r1` dùng privateuser/net/mount/pidnamespace vàactualsockets, khôngclient:2L2subnet/
veth/rootrouter, noexternaldefaultroute. Forward0UDPtimeout/Bkhôngnhận→Forward1echo với tuple thực;
TCP từ10.78.1.2/10.78.2.3 đếnservice10.77.0.1;27IPv4PCAPrecords/ownedcleanup/hostnetnsunchanged.
`routed-KR-two-cycles-r1` tái dùng nguyênCP191/BsoleRoller/normal2cycles/Start3, CP190bundle/copy4b/
47fixture/13runtimemodule; chỉtopology/explicitpeerIP/captureallowlist đổi. Không nativebody/opcode/
statlawRE hoặcclient/sourceDB/asset/policy/inputbounds change; source/configfreeze đếncleanup/seal.

Actualrun404.113676936s/1394raw/frozen227/exit0/cleanup0; SHA`9443c28d9479fdfd2d4f02e013efae09b98c6886cb69625a3edf1db054978f3f`.
1732cyclechecks/90nativepacketcases/528securefieldcases;69Start3/bundlechecks.
4683endpointchecks/1155PCAPrecords; typed153actualIP/UDPport/currentkey→PCAPtuple/hash/checksum→nativeAPI→4peerwindows/2keys/target/currentprogress.

HaiBnormalRollerclaim/one165/view/epoch/B4→5/A5unchanged, normalAselfSpace/IDs1,2/native159158/17→19/
B21/full175180/Ready2/normalBStart3. NativeprotectedbaselineB4/A5 ở3clock/keys12345/12346/12347,
195seedreset/life/peerđã kiểm từ cácoracle đãcó. Vòng3active≥8s/noBomb/noThirdoutcome, không intentionalEOF/
oldUDP/contenders trongrunmới; knownhostEOFCP190controlgiữriêng, không ghép thành routedEOF.
Verifier kếthừaCP189 banđầu chỉ nhận2keys FAIL ở153Start3; source/loggiữnguyên. Sửa checker đểđối
chiếu153key với actualepoch theo timestamp/loadingbounds, không chỉ thêmmagicconstant đểpass;
không sửa wire/client/service/raw. `routed-endpoint-review-r1` giữoldassumption/log+finalproof/seal.

Nativecontroller/script/probes/capture/rawseals/ảnhcontact đềuởlocal/gitignored; SHA từngfile ởmanifest
bằngchứngCP192. Audit[AUD057](../checkpoints/audits/AUD057.json) thủcông README→PLAN→PROTOCOL→RESEARCH→
evidence/raw/report/owners/PCAP/chosenpolicy/ownerinstructions, không chỉchecker. Cập nhậtAGENTS/runtime
environment theoWineacceptance, giữoldcomplete/raw/evidence bấtbiến; audit kếCP196.
K6a/K6bgiảlập giữscopeđãđạt; không K6cInternet/NAT/relay/fullauth/allreset/allmaps/4–8actualclients.
Bước tiếpNATsourceIP/portmapping so với99/153/nativeIPregistration trongtopologycôlập, khôngliveNexon.

<a id="cp193"></a>
## CP193 — đối chứng endpoint dưới NAT, không phải Internet gate

KRE-005b/#14, baseline CP192/[PR264](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/264)
merge`1a6daad0c17d718bc86574162bf2946bca1ed800`. Codex solo, review waived;
[contract duy nhất](../../PROTOCOL.md#nat-endpoint-control), [khoảng trống](../../RESEARCH.md#gap-lobby-identity-followup),
[thứ tự/issue](../../PLAN.md#current-order). Hai KR00967 copy Wine4b, portable CP190 giữ byte/profile/fixture;
không thay source binary/DB/assets hoặc cấu hình trong run. Router/user/net/mount/PID namespace cô lập,
không external default route/Nexon, cleanup rồi seal trước đổi recipe.

Nguồn tái dùng: CP045–047 cache13DE880/14E5710/13E0E00/consumer-tail/callsite13E100A, CP145 membership
helper và source-first refs VN/boomkr-re, CP192 routed native99/153/profile/current Wine. Đã tránh RE lại
codec/CRC, queue/drain, key/getter, endpoint schema và không cần body mới. Khoảng trống chỉ là tuple
sau NAT và đường consumer/dispatch chưa được đối chiếu trên tuple đó; source-first từngrun pin locator/hash.

Transport `NAT-transport-r1`: forwarding0 negative, forwarding1/private positive rồi SNAT
A10.78.1.2:19022→10.78.2.1:29022 tại B, reverse echo qua conntrack;29 IPv4 frames.
Native `NAT-KR-peer-r1` chỉ cấu hình SNAT một chiều nhưng mapping chưa quan sát; giữ
mapping-not-observed.json và analyzer assertion/log, không gán NAT pass/fail. R2 SNAT hai chiều có
counter1/rule và native API mapped ở cả A/B; peer khác0/queue/drain nhưng chưa handler. Probe tiếp
`NAT-KR-downstream-r1` nối exact payload tới consumer trả0, dùng cache để đặt phép đo tối thiểu kế.

| Run (local/research/codex/) | Thời gian / raw / frozen | SHA run.json |
|---|---|---|
| `NAT-KR-peer-r1` | 166.549730307s / 695 / 227 | `2e764155d8559961a6b4a5443159402fd89e0a6c24117266f01d04640f6f41ba` |
| `NAT-KR-peer-r2` | 168.621473145s / 680 / 226 | `7760f30b535f4898ba5011435f8a76816f4c3bc380adc25d1bd16152c59a4245` |
| `NAT-KR-downstream-r1` | 163.941602773s / 692 / 227 | `fb9292f60911a4d6426dbbff27652c67013e83e8144d2c9a6471cf1ff375ff59` |
| `NAT-KR-endpoint-private-r1` | 163.348797091s / 692 / 227 | `d23e25a5a8f2014b6dc108b32cb7aaa6e6b14058dee7507b62bf96e7aa8ea245` |
| `NAT-KR-endpoint-mapped-r1` | 164.133693145s / 656 / 227 | `4dccf87678399999870ec8fd98ba561d6326c253e163046d3767d2881ae2a33b` |

Mỗi run exit0/source frozen/owned cleanup0; exit0 không thay semantic acceptance. Hai endpoint controls
cùng probe/normal OS Right: private helper0/consumer0/no handler; mapped helper khác0/output13/exactSHA
→ E66680/key/apply/target/current progress hai chiều. DNAT điền port99 thực theo recipe trước153; service
chỉ dùng peer-ip/proxyport options có sẵn. Verifier trace và comparison source/log/raw/hash trong external
manifest CP193; không có file capture mới được tự tạo bằng encoder làm oracle.
Comparison r1 FAIL vì đòi mọi output13, trong khi có control3byte/selector0 và empty polls trả0.
R2 ghép input/output theo thread/thứ tự, kiểm riêng movement15→13/SHA/native handler/apply;29checks pass.
Giữ cả script/log thất bại và bản sửa, không sửa packet/raw.

Không toàn thân helper13DF710 mới, không tách IP-only/port-only, không NAT discovery/relay gốc; PCAP
chỉ kv-a. Metadata legacy noNAT/udp_proxy trong run cũ giữ nguyên và giải thích ở PROTOCOL, không sửa raw
cho phù hợp kết luận. Không bomb/outcome/reset trong hai controls; EOF cuối là cleanup. Bước kế hai
normal cycles/reset trên mapped topology. CP193 complete chỉ tài liệu/evidence đã đồng bộ, review waived,
merge theo PR; K6c/K7 mở. AUD057 qua192, audit periodic kế CP196.

<a id="cp194"></a>
## CP194 — hai vòng bình thường và reset qua NAT tĩnh

Baseline CP193/[PR265](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/265), exacthead`b528ed9e9347bb6b3bbc2aae12ca2d4f3f2848a0`,
merge`8acf57c81a727892c04e735c28a20a5319d70fd9`. KRE-005b/#14/Codex solo/review waived.
[Contract/giới hạn](../../PROTOCOL.md#nat-two-cycles), [câu hỏi kế](../../RESEARCH.md#gap-lobby-identity-followup),
[PLAN](../../PLAN.md#current-order). Tái dùng CP192 recipe B sole Roller/two wins/native reset/Start3,
CP193 tuple/control/private router/typed153 và CP190 portable bundle; tránh RE lại stat/codec/queue/
endpoint hoặc native body. Delta chỉ topology SNAT/DNAT và các tùy chọn endpoint service có sẵn.

`NAT-KR-two-cycles-r1`: 408.482927410s / 1394 raw / 227 frozen / exit0 / cleanup0;
run SHA `40626c6022d57bb56f8f38f363d0165947f72a42995111e01e8f98fd3f28f85c`. 1173 PCAP records / 3580 endpoint checks.

Nguồn: native124/165/stat getter, actual118/159/158/same entity ID, trap/death/B21/175/180/room,
Ready2/normalBStart3/3clocks/keys và195seed reset; hai B4→5→4 wins, không bom/kết quả thứ ba.
Bốn native peer windows/hai key đều có target/current progress. Typed153 mapped IP/port/key từng epoch,
actual99 private IP/port, PCAP checksum/index/wire SHA được nối API sau NAT và apply; chỉ capture kv-a,
không kv-b riêng. Counter SNAT/DNAT có thể chỉ tăng ở luồng mở conntrack, không ép mỗi rule đều tăng.
DNAT điền actual99port theo recipe trước153 đầu tiên, nguyên source/config/input đến seal/cleanup.

Raw/scripts/source-first/preflight/capture/ảnh/native reports và hash từng file ở external manifest CP194,
không commit binary/asset/decompile. Getter emulation tách runtime; protocol/policy limits giữ nguồn chuẩn.
Host EOF/rejoin control cũ độc lập không đổi thành EOF qua NAT cùng run. Không originalNATdiscovery/
relay/Internet/loss/generalreset/4–8clients hoặc gate mới. Bước tiếp transport delay/loss có đối chứng rồi
native KR trên qdisc cấu hình cố định, theo PLAN. AUD057 qua192, audit periodic kế CP196.

<a id="cp195"></a>
## CP195 — hai chu kỳ qua NAT với UDP delay/jitter/loss

Baseline CP194/[PR266](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/266), exacthead`760f9cdf078ef48f1e17ece29ff2611911b1ca58`,
merge`9f4ebf53d29d22092b0971c7fb34e07dbcb2cce1`. KRE-005b/#14/Codex solo/review waived.
[Contract/giới hạn](../../PROTOCOL.md#udp-netem-cycles), [câu hỏi kế](../../RESEARCH.md#gap-lobby-identity-followup),
[PLAN](../../PLAN.md#current-order). Tái dùng CP194 native two-cycle recipe/typed153/NAT/getter/peer/PCAP;
không native body hoặc codec/queue/item/mode RE mới. Delta chỉ hai qdisc UDP trước launch; TCP baseline.

`netem-transport-r1` source-first → baseline64/64 → drop100%0/3 → delay40±5ms/loss5%seed194:
58/64 echoes/Bnhận61, RTT trung vị80,544438ms so baseline0,035740ms.529checks/raw/hash/ownedcleanup.
Counter kv-b6 sau ca delay/loss là tổng tích lũy gồm3negative và3newdrop; kv-a0→3. Không sửa raw,
verifier tính delta theo hai snapshot; không lấy mỗi counter6 làm số drop mới của phase5%.

`netem-KR-two-cycles-r1`: 403.626352514s / 1388 raw / 227 frozen / exit0 / cleanup0;
SHA `a0fdf455b957870dfcba347789b7673efadce9b379ee2e98687ec5f21483bb43`. 3538 endpoint checks / 1159 PCAP records.
Qdisc `kv-a`: counter ban đầu0; sau run 631 sent / 43 drops / 631 delayed.
Qdisc `kv-b`: counter ban đầu0; sau run 493 sent / 37 drops / 493 delayed.

Native same-run: twoBsoleRollerwins/4→5→4/A5, actualASpace/bomb IDs/native159158/trapdeath/B21/full175180/
room hai lần, Ready2/normalBStart3/3clocks/keys/195seed reset. Fourpeerwindows/twokeys/currentkey/apply/
target/current progress/PCAP exactSHA/mappedAPI/typed153epoch; getter emulation tách actualruntime.
Freshqdisc0counter, fixedconfig trước firstnativeTCP; chỉ sáu add/qdiscfilter, không mutate giữa run.
Counterdrops/delayed có tăng ở cả hai queue, không gán từng packet không thấy cho drop hoặc suy RTT
transport riêng thành nativeping dưới GDB. PCAP kv-a theo pre/postNAT, không kv-b capture độc lập.

Scripts/source-first/preflight/raw/capture/seals/ảnh/report/hash từngfile ở externalmanifest CP195. Không
sourcebinary/DB/asset/history thay đổi. Không originalNATdiscovery/relay/Internet/fullUDP/alllossdistribution/
EOFinthisrun hoặc gate mới. Tiếp pendingBhostEOF/remainingAcleanup/fence trên topology này theo PLAN;
CP196 audit thủ công bắt buộc, AUD057 qua192/counter không reset.

<a id="cp196"></a>
## CP196 — pending B-host EOF qua NAT/qdisc và periodic AUD058

Baseline CP195/[PR267](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/267), exacthead`1d914b3cedc2c4a9a0cc30a3c5dd94530bd949b4`,
merge`fe329b0b4da66aa4138c0e1a242da6ab3033d4a2`. KRE-005b/#14/Codex solo/review waived.
[Contract và giới hạn](../../PROTOCOL.md#nat-netem-host-eof), [câu hỏi kế](../../RESEARCH.md#gap-lobby-identity-followup),
[PLAN](../../PLAN.md#current-order). Tái dùng exact CP190 pendingB/owned PID/image guards/107110/fence,
CP195 mappedNAT/UDP40±5ms/loss5%seed194/TCPbaseline và cached native probes; không native body/codec RE.

R1 `NAT-netem-host-EOF-r1`:185.327672031s/699raw/frozen227/exit0/cleanup0,
SHA`9e459ca7f8625335d0b85e141d3221e1e1bc07015d4c2cf4218140df80127738`. Native A175/180/107ID2clear/
chief110 và service round_closed/no158 đã thấy, nhưng full verifier FAIL: controller dừng ở native_stop
sau Python AssertionError/1256B3B/live_positions khi actor storage qua scene12, không native SIGSEGV.
Giữ raw/failedscript/log/diagnosis, **không** gọi pass hoặc sửa verifier để nhận partial.
R2 chỉ đóng UDP/membership/feedback observer sau2s từ kill trước forfeit; CRC assert/client/service/policy/
input/NAT/qdisc giữ nguyên. Đối chứng hỗ trợ lifetime hypothesis, không tự suy toàn rootcause r1.

R2 `NAT-netem-host-EOF-r2`:185.051682329s/696raw/frozen227/exit0/cleanup0,
SHA`edc9729c6aefc4fdf9d2d0e5c7f83c97b9ad13161a4c25e3c8f6c0292b9725ec`.1050oldEOFchecks+137tail/qdiscchecks/127PCAPrecords.

B actualSpace/118ID1/owner1/9,3/power1/159both; kill còn4,914883s trước6sfuse. Native A175/180/107 clear
slot1/ID2/chief110/room, oneepoch/forfeit chỉ A; pendingjob round_closed/no158/không giả17/19, qua due+4s.
Qdisc fresh0 có actual drops/delayed cảhai, source/config freeze tới cleanup/seal. R2 không debugger stop.

Chronology r1: exact frame B→A PCAP trước kill41,869850ms, GDB ingress observation sau kill287,404317ms;
currentkey/handler apply quan sát trước server107. Report v1 dùng chữ late chưa rõ, v2 pin thêm PCAPindex/
monotonic để phân biệt dữ liệu trướckill với native/GDB observation muộn. Giữ cả hai report/script, raw
không sửa. R2 không thấy API saukill trong cửa sổarm, không proof queueclear/khôngcòngóicũ/post107 bypass.
Không ghép EOF riêng thành normaloutcome hoặc hai vòng CP195; không account resume/Internet gate.

[AUD058](../checkpoints/audits/AUD058.json) periodic qua196 đọc core docs theo thứ tự, targetedcurrent
contracts/authority/gaps/evidence/runtime/source/raw/images CP193–196, không chỉ checker. Sửa nextstep
routed/NATtuple/K6b/counter cũ, authorityNATunknown quá rộng, ghi chú K1/K2 cũ chưa gắn lịch sử, tableCP193
thiếu separator và temporalclaim PCAP/GDB. Source-first/cache/failedoracles/boundedgetterlimits/reviewwaiver
vẫn giữ. Không recertify toàn lịch sử6500line hoặc thay record complete; audit kế **CP200**.
K6c Internet/K7 còn mở; tiếp incoming admission xen kẽ (CP1906/8wrong) độc lập trước bốn nativeclients,
không tự nghiệm thu Internet bằng netem hoặc thêm networkvariant không có blocker. Raw/hash/scripts và
ảnh audit ở local; metadata/externalmanifest CP196 ghi từng pin. PR/exacthead theo AGENTS.

<a id="cp197"></a>
## CP197 — incoming admission A/B riêng và native bundle compatibility

Baseline CP196/[PR268](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/268), head`537d98a6832435fd59042f91cb725d281ec5949b`,
merge`e842347b7cac073c6822b2251096747710abc170`. Codex solo/review waived/KRE-005b/#14.
[Contract/giới hạn](../../PROTOCOL.md#operator-incoming-binding), [phép thử tiếp](../../RESEARCH.md#gap-lobby-identity-followup),
[PLAN](../../PLAN.md#current-order). ReuseCP190actualTCP6/8wrong/sourceaccept/noincomingbinding và CP195
NAT/qdisc native two-cycle recipe. Không nativebody/codec/wire RE hoặc sourceclient/DB/asset thay đổi.

Implementation: opt-in --admission-ip A/BdistinctIP, per-rolefourtransport counters; defaultglobalordinal
vàreplacement giữ nguyên. Unknownsource/exhaustion bị reject trước event/channel/counter; noresume/
accountauth/C/D/replacement_slotcombination. Newbundle59filepins/exact47alias43fixture/rawprofile bytes,
source service thay đổi explicitdesign; oldbundle immutable. Counter/hash/source first/currentservice proofs.

RealTCP:defaultABABABAB6wrong/8; opt-inABABABAB/BBAAAABB0wrong/8, unknownIP/fifthtransportnegative.
8newtests +302server suite;17bundle/export/admission tests và43checkpointtests pass. Lệnh tools tests đầu
thiếu PYTHONPATHtools gây2importerrors; giữ log, rerun đúngđườngdẫn pass, không đổi test/code để né lỗi.

Native `admission-KR-two-cycles-r1`:402.261681333s/1388raw/frozen293/exit0/cleanup0,
SHA`feb963b3863fe89b8a4ef3006792a25b9d067d01f342a9d26a4034086539f24c`; 1754binding/nativechecks/3439endpointchecks/1126PCAPrecords.

ActualTCPsourceA10.78.1.2/B10.78.2.3/count4/stages1..4, outgoingNAT153A10.78.2.1/B10.78.1.1 riêng.
Cùngrun haiBsoleRoller4→5→4/normalwins/A5/Aselfbomb/trapdeath/B21/175180/room/Ready2/normalBStart3/
3clocks/keys/195seed/fourpeerwindows/nativekey/apply/currentprogress, newmodule insideadmissionbundle.
Native UI startup vẫn serialized, không claiminterleavednative; realTCP control là độc lập. Existing47fixture/
profilebytes unchanged, getter emulation táchruntime, UDPqdisc40±5/loss5%/TCPbaseline/sharedkernel.

Raw/hash/seals/source/preflight/bundlepins/verifier/nativecapture/sourcefirst ở externalmanifest CP197.
Không originalauth/sameNATIPaccounts/resume/native4–8/Internet/K7gate. Bước tiếp earlyBnative startup
vàUIJoingate actualAroom, đoactualacceptinterleaving trước mởthêmclient; khônglặpsame2cycles không biến.
AUD058qua196/auditkếCP200; records/evidence complete cũ giữ nguyên, commitPRexacthead theoAGENTS.

<a id="cp198"></a>
## CP198 — actual native TCP startup xen kẽ và hai chu kỳ

KRE-005b/#14/Codexsolo/reviewwaived; baseline CP197/PR269. [Contract/giới hạn](../../PROTOCOL.md#native-interleaved-startup),
[phép thử kế](../../RESEARCH.md#gap-lobby-identity-followup), [PLAN](../../PLAN.md#current-order).
Tái dùng nguyên bundle/source/profile/fixture47/copy4b/NATqdisc/nativeinput/oracles CP197; chỉ launch B sau
transport2 A và UIJoinBchờactualAroom. Không nativebody/codec/schema RE hoặc thay wire để ép đúng.

`native-interleave-r1`:377.703822943s/1316raw/frozen293/exit0/cleanup0;
SHA`d76e7a3e04ff7fab362a1abcf44241b456927143fbbb57b013c45b0feb463da1`; 1684nativebindingchecks/3442endpointchecks/1127PCAPrecords.

ActualorderA1A2B1B2A3A4B3B4 từ accepttimestamps/nativeTCPsource/stage/count, incoming privateIP tách
outgoing153mappedtuple; nativeidentity/room/chief/rejoin/start đúng. CùngrunhaiBnormalwins/Roller4→5→4/
A5/Aselfbomb/trapdeath/B21/full175180/room/Ready2/normalBStart3/3clocks/keys/195seed/fourpeerwindows/
key/apply/target/currentprogress/PCAPSHA. Getteremulation riêngnative, UDP40±5/loss5%/TCPbaseline.
Không từlaunch/tcpacceptalonekhẳngđịnhlogin/gameplay. Run source/config frozen/ownedcleanup trướcseal.

Scope chỉ lịchABnày/uniqueincomingIP/fourtransportslifetime, khônggeneralaccountbinding/sameNATIP/C-D/
4–8actualclients/mọiinterleavings/Internetgate/K7. Raw/sourcefirst/preflight/seals/scripts/PCAP/hash ở
externalmanifestCP198. Tiếp source-first hardcode/service roster/session/153 tối thiểu4người theoPLAN,
không RE lại known8slot/Free-Manner/codec/ABadmission. AUD058qua196/auditkếCP200.

<a id="cp199"></a>
## CP199 — bốn native slot/actor, hai actual client + hai synthetic

KRE-005b/#14/Codexsolo/reviewwaived; baseline CP198/PR270 merge`f6914d17`.
[Contract/reuse/giới hạn](../../PROTOCOL.md#four-slot-native-consumers),
[câu hỏi kế](../../RESEARCH.md#gap-lobby-identity-followup), [PLAN](../../PLAN.md#current-order).

`four-slot-r1`: 141.524046084s/562raw/242frozen/exit0/cleanup0;
runSHA`2930d6b5169ad551faf62fa9d7e2dc9f96e1a026aee02fcde88d350aa630cc7b`. 883mechanicalchecks + 203reusednativepeerchecks.

Recipe: `python3 local/research/codex/prepare_four_slot.py`, task-local adapter/probe rồi preflight,
`python3 local/research/codex/run_four_slot.py`; profile và input frozen trước launch. Chuẩn bị/adapter/control/
controller/GDB/sourcefirst/preflight/rawseal/verifier được pin trong external evidence CP199; không cần cat rawindex.
Run chỉlo/privateuser-net-mount-PID namespaces/tworoutedhosts/staticNAT/UDP40±5ms/loss5%seed194/TCPbaseline,
không default/externalroute. Giữ copy4b/sourcea525/DB/assets/bundle/service code; subclass chỉ106/121/153.
Tái dùng row26/slotCount/Ready/team/peer26 và consumer addresses; không quét/nativebody RE mới.

Cả hai client native đọc121181/181/inner152/152/153133/133/errors0, IDs1–4/position0–3/bốnspawnobject,
clock tọa độ riêng và ảnh bốn nhân vật; hai A/B nativepeer windows ingress/key/apply/target/currentprogress.
Peerregister chỉ3remote/view (localself bỏ), verifier đầu đòi4calls bị bác bỏ/failure giữ nguyên; không sửa run.
Hai synthetic giữ KRGuest/profilezero/Ready4/slot2–3/ID3–4/team0–1/endpoint0: không cóC/Dsession/auth/291/
client hoặc syntheticUDPdelivery. Không outcome/reset2v2/bốnactualclient/gate. Scope bốn session đã chọn,
service/realTCP trước actual4Wine ở PLAN; AUD058qua196/auditkếCP200.
