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
