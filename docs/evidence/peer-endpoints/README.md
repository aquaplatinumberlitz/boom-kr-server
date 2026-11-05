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
