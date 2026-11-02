# CP059 — Named cache CRC raw/nén và persistence

Baseline CP058/[PR77](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/77) merge
`07cf48d3f90cd308ded49bc35a56bfd82c20b4a2`; KRE-022/[issue78](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/78).
Codex solo/tự kiểm, [review waived](../checkpoints/review-waiver.json), chờ PR.
[Contract](../../PROTOCOL.md#dynamic-cache-representation), [gap](../../RESEARCH.md#gap-profile-cache),
[thứ tự duy nhất](../../PLAN.md#current-order). Raw `local/research/codex/cache-boundary-r1/`.

`reuse-question.json` pin CP016/019loader/cacheCRC/blob/row/cachedDFE850/DFE8D0/DFEB10/writer,
CP058 sạchauth, VNlobbycache/method, boomkr-re/boom-rsVNreference/l10n/archive. Không mang filename/
CRC/opcode VN hoặc RE lại login/authfields/row120/checksumhelper. Mở3body `14EA7C0→14EAB40→14E9560`
chỉ vì runtime cachedCRC804314B9 khácexpected8F400A8B dù cùng69byte decompressedinner.
PrivatebatchIDA/fallback knownMCP missingida_nexus, không config/sourceDB changes hoặc fakeEvidenceID.
`body-comparisons.json` pin3vùng code khớp runtimecopy; cachedrawasm còn giữ locus/sourcehash.

A/B game-dir **reflink riêng**, `cp -a --reflink=always`, không hardlink. `private-copies.json` và
snapshots pin bytes/sourcehash; chỉ xóa `fxdynmapinfo.ssd` trên bản thử giữa run, không sourcecache.
Các cache khác giữ, nên không gọi coldtoànclient. Freshprefix/display/wineserver riêng mỗi run,
namespace user/net/mount/PID chỉlo/no route/files-only NSS/runtimelease/controller tuần tự.
Runtimecopy SHA `e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef`;
static `660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3`, modO **raw/base400000**
`a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5`; RVA=VA−400000, không PE-remapraw.
Không thêm patch/forceID/state/CRC/getterreturn trong client. Auth137/errors0 giữ ở cả4run/A/B.

| Run / UTC02/11/2026 | Phạm vi chính A/B | run.json SHA256 |
|---|---|---|
| cache-present /11:47:40 /180s |fileload1, CRC804314B9≠8F400A8B/loader0; actual528; raw238 ghi file. |`0d2d464f30987619793c9605a0cdcbcca1a077252c2cdb795e547863e5262d94`|
| cache-missing /11:51:06 /160s |xóa namedfile, fileload0/528; raw238 ghi lại. |`b4d49e9cae3e387ed8a1e3c9d269679f30a0b8dd56011966aa366d2f4c2f6458`|
| compressed-missing /11:54:41 /160s |xóa namedfile,528; compressed238/advertised804314B9 khớp/nativewrite. |`5adde89de49e28a6a8d5ebca4ac6f192266155055fc8b5e8564cb014b08fbb1a`|
| compressed-warm /11:57:54 /150s |giữ file/newprefix; CRCkhớp/nativeparse69/69/load1 **không528** trước238 sau. |`99f0fb6c95a809eddd7cbdd81cf9196cab9d82f790490091732032a25273d3ac`|

CRC được native `14E9560` tính trên **storageSelector+encodedrepresentation trước inflate**,
không mặc định plaintext. Selector0 zero+inner/seed0 trùng checksuminner; selector1 1+u32usize+zlib
cho804314B9. OuterSSI magic/filechecksum97201510 kiểm riêng; blobchecksum là giá trị quảng bá241.
Hai inner69byte byte-identical, không rowdelta/corruptcache; `cache-wire-delta.json` và nativeblob/
inflate/checksum chứng minh. Candidate nhận **exact blob đã được native ghi**, không vá cache/binary.
`native-compressed.json` kiểm238/241cursor/error0 và inflate69/checksum804314B9, heap/free model.

`verification.json`:3349relations/hash,1755native actualframer valid/CRCbad/partial cases,4sealedruns,
file/source snapshots và exactpolicycode; actual528 mỗi ca fallback/frame38/bodyrỗng/seq67, warm0cảhai.
Cache sau nativewrite rồiwarm ổn định76byte/SHA`6a4745c6010378a50ab4bf34827bbe122d7eb270c78051391cd21bd7378b1053`.
Warm read-only parserexitprobe bổ sung tạiDFEEC6 đo file+wire return1/cursor69/inner69; không đổi nhánh.
Raw tagLOAD_END `count` là manager+14pointer **không** semanticrowcount; parseru16local count1 mới có nguồn.
RawWRITE_BEGIN DFE9A0 là handler238/cacheupdate, WRITE_RESULT14EA940=1 cộng snapshotfile là proofghi.

Service vẫn gửi238 sau config vô điều kiện; warmaccepted trước callback238 và không528 chứng minh
fileloadacceptance, không chứng minh offlinecache-only/sessionresumption. Gameplay/lifecycle partial
không dùng đóng fullmatch/K5/K6a/LAN. Không gán khác biệt UI/input/duration cho CRCfix.
Exporter41frames đổi ready/table,39alias khác giữ (kể cả auth mới); byte xuất khớp exactprofile-compressed.
ProfileJSON cũ giữ để tái lập, không sửa để trốn hash. Server/codec không đổi; baseline50server+18checkpoint
CP056 được tái dùng, checks mới nativeblob/actualstreams/export/catalog/docs/artifacts/localhistory/diff.

Run mới: run_probe.py labelmới CA-r2-stage-social.exe 160 wine_probe_cache_compressed.py cache-warm.gdb,
seal_run.py khi completion, snapshot_cache.py trước/sau; parser/verify chạy modulemode từreporoot.
Lỗi decoderđầu dùng zliboffset17 thay18 đã sửa bằng headerlength, không nativefailure/rawmutation;
metadata oldtable sequenceNone đối chiếu actual67 trướcdecode. Thử representation xảy ra chỉ sau
cạnh runtime/static rõ, không tăng buffer hoặc bỏ unknown để pass.

Tiếp runtimeCauth135/newprofile/currentB/Cround theoPLAN; không C auth proof ởCP059 hoặc toàncachefamily/
version/corrupt/delete/reconnect/token_2/allvariants/vanilla/authauthority. AUD018/CP058 giữ, periodicdueCP060.
