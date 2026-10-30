# Join/Ready và hai chu kỳ pair — CP028

Codex solo/tự kiểm; review độc lập [waived](../checkpoints/review-waiver.json).
Baseline CP027/PR35 merge`b55ec6cb4e1202f0f6007fa614f3c3a8b4363c79`.
CP028 là snapshot trước tích hợp, PR nhánh`kre/duo-room-round` giữ trạng thái merge hiện tại.
Contract chỉ ở [PROTOCOL](../../PROTOCOL.md#duo-room-round), việc tiếp ở
[RESEARCH](../../RESEARCH.md#q-023), gate ở [PLAN](../../PLAN.md).
AUD008/CP028 đã đọc lại bốn docs chính và raw/report mới; audit kếCP032. Không gate mới đóng.

## Artifact, cô lập và recipe

Raw`local/research/codex/duo-round-r1/`, ignored; scripts probe cạnh bên dưới`local/research/codex/`.
Bảy run tuần tự, mỗi controller giữ lease trong user/net/mount/PID namespace chỉlo/no route/files-only
NSS/localstub. Pair gồm hai prefix/wineserver/display/game-dir riêng trong cùng namespace.
Không sửa source binary/pack/DB hoặc patch thêm vào copy diagnostic CP020.

| Artifact | SHA256 |
|---|---|
| RuntimeA/B CA-r2-stage-social.exe |`e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef`|
| Static CA-r2-login-logout.exe |`660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3`|
| modO.bin raw memory-layout/base400000 |`a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5`|

RVA=VA−400000; map modO raw, không PE-remap.11 vùng code khớp cả bốn image.
Kế thừa sửa import/API/AC/guard, không vanilla. GDB không ép identity/ready/stage; guard reset cũ
vẫn có trong recipe, không coi đây là client nguyên bản chạy không can thiệp.

| Run / bắt đầu UTC30/10/2026 | Config | Quan sát/giới hạn |
|---|---|---|
|room-row-join /12:07:51|80s/1client|Row37/error1 control vẫn vẽ card và phát82; không87.|
|join82-reply87 /12:10:06|95s/1client|Row38/error0,82→87slot0/full7, stage12 rồi chuỗi single-client cũ.|
|guest-join-ready /12:11:51|80s/1client|ID2/slot1,Ready95 thật; chưa ACK95, ở phòng.|
|duo-join-ready /12:15:15|180s/2client|A81,B82/87,B95/110,A104/121;4bom mỗi phía. Bsession0/A1 control; không175/180.|
|duo-two-rounds /12:19:15|215s/2client|Cùng session1;175/180 parse đủ và về phòng một vòng, returnclick chưa95.|
|duo-round-retry /12:24:31|225s/2client|ReturnReady95 lần hai/110; returnclickA chưa104, chỉ một vòng.|
|duo-round-restart /12:28:59|240s/2client|Hai vòng cùng pair/sockets;2actual104A/95B/99 mỗi phía;2bom/175/180 mỗi phía và về phòng.|

Mọi runexit0/noerror; ba run đầu và hai run thử restart vẫn chỉ đạt phạm vi ghi trong bảng.
Tên`duo-two-rounds` là mục tiêu recipe, không phải kết quả đạt hai vòng.
Run pair đầu vẫn có session lệch; hai cycle cuối không thay raw/control đó. Một số actionlabel
`click-create-room` củaB trong recipe đầu thực tế là doubleclick join; capture82 mới chứng minh thao tác.

Recipe argv/hash/GDB ở`<run>-recipe.json`, full raw index ở`<run>-hashes.json`. Tái lập với runID/prefix
mới, Bgame copy riêng đã chuẩn bị; không ghi đè run/prefix đã seal:

```bash
python3 local/research/codex/duo-round-r1/run_probe.py new-pair CA-r2-stage-social.exe 240 wine_probe_duo_round_restart.py local/research/codex/duo-round-r1/retry.gdb
python3 local/research/codex/duo-round-r1/seal_run.py new-pair
python3 local/research/codex/duo-round-r1/derive_roles.py new-pair
```

## Đọc bằng chứng và kiểm

**10376 kiểm cơ học pass**, bảy sealedrun/hash/isolation/lease/sequential, actualrequest/parser/
seq/callbacks và11 coderanges; **315 ca native framing/selectedgetters/blob** cùngCRCbad/partial.
Tất cả C→S được replaynative14EF300 riêng từng connection/role; full95context chỉ decodeblob,
không gán toàn semantics. Native allocator/free/HeapAlloc/receiver có mô hình, giữ native checksum/
decompression; lỗi thiếuHeapAlloc và streamindices ban đầu được giữ trong notes/raw.
Full153/121/175/180 consumer dùng Wine callbacks, không suy fullparse từ framer.
Checker không xác nhận semantics, policy server gốc hoặc tự đóng gate.

Run cuối Aport9339/B9256 giữ qua hai99; endpoint phát từactual99. DynamicS2C đầu73..136 bước3,
121seq76/109,15385/118,17594/127,18097/130;159/158 chen88/91 và121/124.
Ready95 đầu body60/stored51/decode265; sau trận body322/decode2845, desired1/tail0 ở cả hai.
Hai bom A có serial0/0 ở hai stage, chungID1/2; không thửBbom trong cùng runhai vòng.
CP027/firstpair có Bbom riêng; không ghép các ca thành proof mọi hành vi qua hai vòng.

Đọc thủ công ảnh`duo-round-restart/{A,B}/rank-r1-delayed.png`,rank-r2-delayed.png cho cùng
LOSEKRLocal/PERFECTKRGuest, highlight theo ownID;`final-both.png` cả hai có phòng sau180lần hai.
Bấm sau return cần retry hữu hạn:3clicksA→2actual104,3clicksB→2actual95. Chưa chứng minh nguyên nhân
click đầu không phát request; focus/delay/retry được chỉnh trong recipe, không gọi causal isolation.
ẢnhB`blast-2`/rank không tự chứng minh Bđã chấp nhận cùng trap/death vớiA. Rank/outcome do stub chọn;
item/pickup/collision/authority/fullreset/disconnect vẫn thiếu, chưa fullmatch/K6a/LAN/Internet.

VNroom_list_with tại`boom-re@9ac2075/server/src/packets.rs:228–273` là pattern reference;
KRfeaturetail/join/Ready đọc riêng. SnippetDDEA60/DDE900/DC6A00 lệch đầu là navigation, dùngaligned
DDE760 và runtime probes cho guard. REA coldCP024 timeout/closed không lặp unchanged query;
checkpoint này dùng cache/Capstone/hàm KR native và WineGDB, không nhận decompile chưa có.
Chi tiết lỗi/giới hạn/visual nằm`measurement-notes.json`; không quét/devirt toàn client hoặc gọi worker.

## Hash metadata

Full file/size/hash trong từng sealed index; manifest pin các index và raw/report/recipe quan trọng.

| File dưới duo-round-r1 | SHA256 |
|---|---|
|verification.json|`7c2b8c1f7f306f87c4f2d4e0e8bcbb0a05b4d7bcb24cc0b3c42402b172da154a`|
|runtime-summary.json|`a34f83cec224a1c5cae84cebd5b1e5e826d04d79e1dd7ffbb3f5cd6aaf120888`|
|native-fields.json|`765f3914862de6f25efa14dd7ce03fd9dd826964aa46707ec31becea14b463d2`|
|code-match.json|`878c858aab88a8a7a855245756f2bb1ebb7f9d50d43821604a0be52529bc6950`|
|measurement-notes.json|`eb3fe4d68834551a232b91706b94909a914ab19eb6fe43a3cea712aa416193be`|
|room-row-join-hashes.json|`299340a28b73eed680be7b1ab1556e881a058f6a89c744f9793905956a0ffcae`|
|join82-reply87-hashes.json|`fdfe8dabb631ec072144b774a47b3b267c74b90921b0e928e2aeca1b58660b3f`|
|guest-join-ready-hashes.json|`e3a65e0c1338d3f41978ed0d50585647d4a6bbe62203bfcf47a0c514034984a0`|
|duo-join-ready-hashes.json|`b96cb9ad2a86f10d9b1ab1ca91e9441e5189df52b312be8822edbcb5b09cd703`|
|duo-two-rounds-hashes.json|`333d3e615aa8692895898bd2081acf2b6d47187a276ff7f5b13e086b9f79f32e`|
|duo-round-retry-hashes.json|`0b2dfe0b693f91746896ac0ad544d1824e6698787ba8de8198c32f91f7b2b889`|
|duo-round-restart-hashes.json|`9bf5691a2a8c69cd1602b4a69b2f63442431d406801c982b99733c7d432a614d`|

Counter/delta/AUD008 ở [manifest](../checkpoints/manifest.json). Audit kếCP032.
