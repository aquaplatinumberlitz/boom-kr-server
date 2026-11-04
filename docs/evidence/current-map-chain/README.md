# CP090 — Target map data và native cell stop

KRE-053/[issue140](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/140), baseline
CP089/[PR139](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/139)
`727e94255e1b501ffeb7ac74a58b0a176d5a8553`. Codex solo/tự kiểm,
[review waived](../checkpoints/review-waiver.json), đã merge [PR141](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/141) tại `c142c85`. Contract/limits duy nhất ở
[PROTOCOL](../../PROTOCOL.md#current-map-chain-state), question ở
[RESEARCH](../../RESEARCH.md#gap-blast-block-chain), thứ tự ở [PLAN](../../PLAN.md#current-order).
Raw: `local/research/codex/map-chain-r1/`. Production giữ nguyên CP089.

`reuse-question.json` pin19sources: CP089 actualray/normalroute/profile, CP033 EB9360 cache,
KR_RE_FLAME_CHAIN R139–141, KR MAPS/legacy_map/pack/ObjectTable/TileInfo/Village00reference, KRdata,
VNoc_e-game/idf, l10nrouter/archive. Tái dùng decrypt/CRC/header80/grid/parser/classreference/nativeprobes,
không RE toànformat hoặc port VNheader76/ID. Chỉ decode map ExVillage01 và hai tables tham chiếu,
không toàn mapcatalog/client. Legacy `krpack` inflate chưa hoàn thiện, không dùng nó làm oracle.

`static-verification.json` pinSHA cả4sourcepack files; `map-grid.json` pin index/entryoffset/key/CRC/
cipher/payload. Map FxMap/Map/ExVillage01 offset13090325,size1814,key419948504,CRC2823173596;
payload SHA `aeb57ffdb27a1405bd2ae3d29f229174ae5d584fc84eaf4e87884fe8518f86a4`.
Version1000,15×13,8spawn+80, grid148..1708/195cells×2i32,106tailbytes giữ chưa giải.11usedIDs được
nối ObjectTable/TileInfoData72 trong `static-classes.json`; classraw0/1/2, không suy collision theo tên.
`control-cells.json` chọn cells theo actualgrid/knownnormalinputs, không giả đo đã tới tất cả.

`gap-byte-match.json`: EA1230/EA0B10/E9F9B0/E90A80 currentcopy match runtime/staticPE vàmodOrawbase400000.
Capstone đọc4helper nhỏ, range cópadding/đầu hàm kế không là đãRE mọibody. Runtime sourceSHA
`e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef`, staticcopy
`660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3`, modO
`a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5`; RVA=VA−400000.
Sourcebinary/pack/DB không sửa, extractedpayload/report ởlocal. Reuse providerunavailableCP089,
không sửa cấu hình MCP/IDA.

| Run | Kết quả | run.json SHA256 |
|---|---|---|
| tiles | 358,644s;clock/actualDownWndProc nhưng no124/118/158, ảnhblack/faint. Không đo được tile/stop. | `b42528019e4241426fbb5c3d950b6565b4d645e945785a9e701ac0b2ab9feb34` |
| tiles-gated | 200,292s;newprobe group bật158entry→END; actual5requests/chain/clear+boxstop/resultreturn, noerrors. | `8362dc27134732e4468201266d57e6b1657c9dafc7bdace5b9df43c5845177fe` |

`<label>-recipe.json` argv, `<label>-hashes.json` sealed sau originalfinalrecord. Một privatecontroller/
lease/user-net-mount-PID namespace, lo-only/noIPv4routes/files-onlyNSS; prefix/display/roleenv/OSactions/
focus/pointer/before-after/phase durable. Sameprofile/service/normalinputs; firstnewtile breakpoints
vẫntrap ngoài158 dù commands guard, repeatgated tắt ngoài158. Debugger overhead làcandidatecause
của firstnegative, không causalproof. `first-partial.json` giữ boundedfailure, không fabricatefinalrun.

`tile-runtime.json` parse nativequery/set/cell/step theo callbackindex/line/cell, join staticcoordinates
vàactualnativepath, không trộn mapmutable với staticgrid. `verification.json`: **156 relations**.
Mỗi view29EB9710returns:9boxstops/20clear; all9 cùngcoordinates/callbackIDs, EA0B10callerEB975C
là1→state2/return0, clear0→state1/return1. Các ô cóprecedingE9F220step1 vẫn stop tại helper sau;
không dùng accepted1 làmwholepropagationpass.80E9F9B0queries/view allnull/EA1230zero, không explicit
E9F220stopbranch/nonnullcontext; classflag/directionhouse chưa được đo, không fakeinstancematch.

5actual118/159nonzero/full39 mỗi view,5full15813/errors0; rootprocess1,2,3/newroot4,5/laternoduplicate,
121/clock/175full302/180full3/roomrefresh một lần. Actorsecure/getter/framer/typed làemulation riêng;
không mới hai-cycleacceptance, không originalserverlaw/authority. `visual-observation.json` pin2manual
finalroomimages; firstblackfireimage xemriêng, không collision/causeproof.105server/18checkpointtests
reuse do production/checker không đổi; docs/catalog7/artifacts/localhistory/diff chạy nhưcheckpoint.

Chưa templateTileInfo→runtimeclassflag/direction, house/static/otherclasssets, mutabletile sau phá,
fullmap/recursive/moving/pool/validator/auth/codec/authority/allreset/vanilla/LANInternet/fullmatch/K5/K6a.
Bước tạiCP090 là normal secondroot để tới house; [CP091](../house-after-box/README.md) đã đo house/mutableflag, [PLAN](../../PLAN.md#current-order) giữ gap hiện hành. CP089policy giữ hẹp.
Audit AUD026 qua088, periodicCP092; CP090 không tự đónggate từ sốcheck hoặc từmetadata.
