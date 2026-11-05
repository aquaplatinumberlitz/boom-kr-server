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
