# CP057 — Native codec stream boundaries

Baseline CP056/[PR73](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/73) merge
`c367bab` (exact context trong manifest); KRE-020/[issue74](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/74).
Codex solo/tự kiểm, [review waived](../checkpoints/review-waiver.json), chờ PR.
[Contract](../../PROTOCOL.md#codec-stream-boundaries), [gap](../../RESEARCH.md#gap-socket-codec),
[thứ tự duy nhất](../../PLAN.md#current-order). Raw `local/research/codex/codec-boundaries-r1/`.

`reuse-question.json` pin docs/cached14EF300/oldnativeoracles/actualcapture và VNprotocolA662E0,
boomkr-re/boom-rsVNreference/l10n/archive. Nguồn cũ đã có complete/CRC/wholepartial/capture fields;
chỉ đo retention/memmove/coalesced/seq/reset model còn thiếu, không decompile lại hoặc chạy Wine mới.
Nguồn được đọc trước phép đầu; persistent ledger ghi sau76ca đầu, không giả timestamp trước.

| Artifact/nguồn | SHA256 |
|---|---|
| modO raw memory/base400000, native14EF300/RVA10EF300 |`a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5`|
| CP056 state-guard/run.json actualA/channel4/frame0,1 (seq40,43) |`3e4313e8481ab80a8cdacdd8e7bad67be8880d97d6f4df3f694c0c5f0be384a7`|

Các frame actual6/67, first74byte, lưu actual-A-frame0/1.bin có hash trong manifest. Body được
native decoder kiểm, so service độc lập; encoder roundtrip không làm oracle. memmove/CRC/seq native
chạy thật trong Unicorn; socket/receiver/preallocatedbuffer là model, prefixguard+AC0 như oracle cũ,
14F5790 allocationhelper chỉ cho<=1000hex. Không sửa binary/pack/DB hoặc chạy malformed liveclient.

| Phép đo | Kết quả/phạm vi |
|---|---|
| native-stream.json /76scenario | Mọi split0..74: partialret2 giữ bytes/seq, appendrest0/+3. Coalesced memmove giữ đúngframe2, seq43→46/payload đúng. |
| service-stream.json /75scenario | Actualbytes qua productionreceive/socketpair,2logicalconnections/seq riêng40/43/46, handler chỉ recorder. Không freshWine/TCP/handlerintegration. |
| native-boundaries.json /14hoàn tất+1modelblocked | CRC/wrongseq7/noadvance/no consume; capexactpass/onebelow5 (cap do modelcấp). Length lớn córet2/7, service64KiBdesign. FFFFFFFF allocation vượt model, phần sau unknown. |
| native-reset.json /5scenario |4transformedseqfixtures nativeu32wrap; fresh/warm/fresh synthetic socket pass/CRC7/pass. Không actualnativewriter/reconnect/cacheproof. |
| verification.json /697relations | Exactactualsource/frameSHA, nativeleft/seq/payload, productionbody/connectionseq, completed-vs-modelblocked và exact service/codecsource. |

NativeCRC giữbuffer còn servicepop trướcCRC; productionloop đóng connection lỗi, không giả native
resync. Không đổi service/codec ở CP057;50server regressions tại CP056 tái dùng baseline, không chạy
lại để làm số mới. Catalog đổi runtime-ref KR-B003 từ primitives (chỉ emulation) tới probe-service
đã có Wine bằng chứng; không ghi CP057 là runtime mới hoặc thay state/gate. Original source5artifact
verify, docs/catalog/checkpoint local/history/diff theo workflow; AUD017/CP056 giữ, audit kếCP060.

Các script native_stream/native_boundaries/native_reset/service_stream/verify tái tạo từ repo root
bằng `python3 -m local.research.codex.codec-boundaries-r1.<module>`. Oracle helpers import từ cache
cũ có hash trong checkpoint. `native_boundaries` lúc đầu split source ở literal rows gâySyntaxError,
sửa rsplit newline cụ thể; native_stream lúc đầu đọc payload theo sizeframe1 choframe2, đã sửa lấy
headerlength/crosschecknativepayload. Không sửa actualraw để làm pass; finalreports/hash là nguồn đo.

Giữ fullcodec/string/blob/mode/prefix/live reset/auth/cache mở. Không cấp allocation lớn hoặc suy crash
client từ guard model. Theo PLAN tiếp success/profile/cache consumer có nguồn, không lặp76split.
