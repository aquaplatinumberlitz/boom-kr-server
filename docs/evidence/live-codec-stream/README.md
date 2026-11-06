# CP180 — native codec trên TCP thật và CRC control

Codex solo/tự kiểm; [review waived](../checkpoints/review-waiver.json). Baseline CP179/PR251 merge
`2e6dae49870683d3d103030e0f7e5abfec3ea195`. [Contract/giới hạn](../../PROTOCOL.md#live-codec-stream),
[phép kiểm tiếp](../../RESEARCH.md#gap-socket-codec), [thứ tự](../../PLAN.md#current-order).
Không thay gatecriteria/productionserver hoặc record complete cũ; CP180 snapshot trước PRdelivery.

Raw/copy/prefix/scripts: `local/research/codex/k2-live-stream-r1/`. `source-first-reuse.json` giữ
nguồn/locator/build/proof trước phépđo; tận dụng CP057/VNmethodology/CP058/profile/currente9/harness
namespace. Tránh76modelcases/newnativefunctionscan/2clientgameplay hoặc nhậpVNopcode/layout.

| Run | Recipe / kết quả | run.json SHA256 |
|---|---|---|
| stream-r1 | Nominal60s, actual67,719462273s; auth13/162split, nativepartial/full/cursor/coalesced, login→channelreset và UI sảnh nhìn thấy | `7fa6215573dc7df87fe88feca270d308404682057de3072ffcf1235ecf7ec182` |
| crc-negative-r1 | Nominal42s, actual48,911328033s; mộtbitCRCauth, ret7/seqbuffer giữ/noauthcallback; chỉ3connection | `df5c7b7fca41a06f1b0df6e8813800ad5ac4d9cd1483af27e825397ecc74885f` |

`stream-r1-recipe.json`/`crc-negative-r1-recipe.json` giữ argv đầy đủ. Harness riêng
`local/research/codex/wine_probe_k2_live_stream_r1.py` và `wine_probe_k2_crc_negative_r1.py`
dùng `stream.gdb`/`native_probe.py`; service subclasses `transport_service.py`/`crc_negative_service.py`
chỉ đổi scheduling hoặc mộtCRCbit trướcsend. `profile.json`47fixture pins giữ exactproductionbytes.
Copygame/prefix mới từ CP177C0r3, không sửa tree/prefix cũ; negative dùng tiếp privategame/prefix
saupositivecleanup/seal, nên không onevariable pristineenv hoặc coldcacheclaim.

Isolation user/net/mount/PID riêng, chỉlo/noIPv4route, hostsfiles-only/mapsloopback. Không Nexonlive,
noUIclicks/packetfunctioninjection/forcedreturns. Inheritedglobal1A966FCreset làdiagnosticrecipe,
khôngvanilla. Rollback: chọn harnessservice gốc và copy/prefix riêng mới; không ghi đè files của run.
Source/config được pin trước run, kiểm sau cleanup. `raw-seal.json` phủ log/CALog/recipe/ảnh/native
và service outputs; binary/DB/assets không commit. Toàn bộ hash qua
[CP180](../checkpoints/records/CP180.json)/[evidence manifest](../checkpoints/evidence/CP180-live-codec-stream.json).

`verification.json`:321 measurable checks, includes raw/inputpins/nativepartialstate/authcursor/
coalescedleftbytes/seq, exact1bitnegative, independentcodec rejectsbadCRC, nonoverlap/isolation/cleanup.
50partialpolls cùng một split không50case. Nativecode làoracle; generatedfixtureencoder không tựproof.
Mode0fixedpacketoffsets trong rawlogger không đúnglayout: không dùng chúng làm kết luận, chỉ dùng
socket/returnstate. `screen-20.png` positive được Codex nhìn: sảnh khôngoverlay, tên/profile display
chưađược xácnhận; không tựnângK3. Negative CALog CRC/59 không làproofretry/reconnect hoặc disconnect.

AUD054 periodic quaCP180 đọc currentdocs→source/raw/runtime mới và evidencehẹp; lịch kếCP184.
FullK2/auth/cache/downstream/fullmatch/LAN/Internet/8client giữ mở, xem nguồn chuẩn. Kiểm đầy đủ
local/history/docs/catalog/issues/diff trước PR/exactheadmerge; noCIworkflows, không độc lậpapproval.

<a id="cp181"></a>
## CP181 — same-process login retry và nghiệm thu K2profile

Baseline CP180/PR252 merge`fe242cc87a29463a7f70b8537a932f37effdb177`; Codex solo/tự kiểm,
[review waived](../checkpoints/review-waiver.json). [Contract/giới hạn](../../PROTOCOL.md#login-retry-codec),
[acceptance duy nhất](../../PLAN.md#k2-acceptance), [câu hỏi](../../RESEARCH.md#gap-socket-codec).
Raw hai batch `local/research/codex/k2-retry-reset-r1/` và `k2-auth-retry-r1/`.

| Run | Kết quả / phạm vi |
|---|---|
| retry-r1 | CRCnegative + UIactions; không1002/newTCP; SIGILL2260DF8/caller1259F11, chưa causalmodalcloseproof. |
| close-route-r1 | HWwatchpoint làm WineDbg internalcrash14001D25A; failedinstrumentation, không clientrootcause. |
| close-route-r2 | BỏHW, functionprobes: setter1503DFD/stackreturn1254CED→waitreturn1/global1→cleanup1259EC0→DWKcandidate trap. Không lặp/stubcleanup làmretryproof. |
| auth-retry-r1 | Exact39byteKRresult1: nativeCRCvalid/result1/global0; actualUI1002/newTCP4/seqreset. Stubordinal4channel không đápauth; control giữ nguyên. |
| auth-retry-r2 | Labadapter map4/5theoroute vừa đo; sameCApid/GDBinferior,3hello40→43/0-0→2-3, firstresult1 rồi137cursor/errors0/wait3, TCP5channel. run.jsonSHA`bfa8243780d73802a086bb6de867d4d2a91c384787c142784d86d35376a2ed5c`. |

`source-first-reuse.json` từngbatch pin sources/build/locator/proof trước runtime; CP010/012/014 và
VNerrorbranchmethodology dùng làm reference, không portVNcode/opcode/layout. Tránh newfunctionscan/
76modelcases/fullgameplay. `verification.json`batchauth:655measurablerelationships/raw/inputpins/
nonoverlap/PID/nativeframer/seq/cursor/writerframes; nămrun đều cleanup/seal trước đổi recipe.
`raw-seal.json` từngrun phủ logs/ảnh/service/timeline; failure/control không ghi đè. SHA toànbatch và
selectedoldoracles ở [CP181](../checkpoints/records/CP181.json)/[manifest](../checkpoints/evidence/CP181-codec-retry.json),
CP180evidence manifest được reuse bằng reference nguyên hash, không copyrawindex lớn.

Adapter/harness/GDB chỉ ở local, productionserver/codec và47fixtures không đổi. UIclick chỉ sau
native decodedresult1/30s; chỉ close+confirmmộtlần. Script names/argv trong`*-recipe.json`, prep files
giữ exactsourcehash; isolateduser/net/mount/PID onlylo/noIPv4route/NSSfiles-only. Cleanup prefix-owned
wineserver/childgroups, không kill WineVN/VM. Không có runtimeactive sau seal, original5artifacts verify.

Analyzer đầu đòiWinSocktrace trong harnesschỉbậtseh/timestamp/pid; đã đọcraw và sửa attributionthành
oneCAinit/GDBinferior/actualLinuxPID/nativeframeorder, không bịaAPI hoặc đổiraw. Mode0packetfields
không dùng do layout khác. Positive screenshot22 nhìn thấy sảnh, chưa xácnhận UIidentity/K3.

`manual-k2-acceptance.json` là đốichiếu thủ công nguyên tiêu chí với CP178/179/180/181 và CP057native
boundaryscope; script không tựđónggate. K2profile đạt, widercodec/K3–K7/fullmatch/LAN/Internet vẫnmở.
CP181pendingPRdelivery; AUD054periodic180/auditkế184 giữ, không checkpoint riêng từngphépđo.
