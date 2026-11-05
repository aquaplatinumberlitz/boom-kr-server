# CP112 — Admission cấu trúc report chết trước side effect

Baseline CP111/[PR181](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/181), merge
`248e11847e11358566997816f9212c63d9abe201`. Codex solo/tự kiểm,
[review waived](../checkpoints/review-waiver.json); SnapshotCP112chờPR lúcghi; đãmerge [PR182](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/182)/1b7f210. Contract/giới hạn chỉ ở
[PROTOCOL](../../PROTOCOL.md#death-envelope-admission), câu hỏi ở
[RESEARCH](../../RESEARCH.md#gap-contact-state), thứ tự chỉ [PLAN](../../PLAN.md#current-order).
Audit định kỳ AUD035 qua112, kế116; lịch sử/raw hoàn tất không sửa.

## Tái sử dụng và phạm vi

Raw/model/provenance: `local/research/codex/firstseen-admission-r1/`; runtime mới:
`local/research/codex/death-envelope-runtime-r1/`. `reuse-question.json` pin nguồn CP062/063/067,
current service/claims và capture CP110; corpus bổ sung CP023/075/085/106/109/110. KR-data và VN
items/oc_e-game chỉ hướng report/confirm/inventory, không chuyển opcode/ID/width/authority sang KR.
Nguồn CP063 đã ghi first-seen/forged-forward thiếu authority: không đo lại writer/tick/cache/codec.
Chỉ đo khoảng trống cấu trúc trước cursor/marker/death/170; không dựng life-state oracle.

`source-captures.json`: actual124 A/seq253/SHA
`084381041d882e14111de1d0804bcca76bb58c4510b76a22a5da5125b5dd562f`, actual17 A/seq520/SHA
`9343d95bcaafc6c874e0a25b3624c0550fa748ed672ae1ee4eb1d341e01d4dd6`, actual19 B/seq280/SHA
`06b5556cc8c8bfa2ca5c4f85e9ece85b1fef027329ebe08bba00c000d2cf014b` từ raw CP110.
`before-source`/`before-hashes.json` giữ production trước sửa và baseline matrix.

- Baseline dispatcher nhận report19 prefix-only/truncated/length sai và cảtail dư: cập nhật cursor,
  pickup-ineligible, death_event và phát170. Đây là model service với actual control/synthetic mutation,
  không phải client thật phát gói sai hoặc bằng chứng server Nexon có lỗi.
- Native14EF300/typedprefix/14EB410 với allocator có giới hạn: actual29byte đọc29/29/lỗi0;
  prefix-only/truncated/declared10 có readerror54=1. Extra1 đọc29/30/lỗi0, nên exact-consumption guard
  ở emulator là policy của variant hỗ trợ, không gọi đó là native tự loại byte dư.
- Guard mới kiểm subtype17/19 envelope đã đo trước admission/history/cursor/marker/forward. Hai capture
  death inner0 và innerFFFFFFFF đều giữ exacttail; không giới hạn inner thành winner/killer enum.
  33 capture thật17/19 từ sáu nhóm nguồn đều qua Python guard, không native/runtime mới cho từng capture.
- `socket-receive-verification.json`: bốn ca đi qua socketpair→recv/frame/currentseq→dispatch; ba malformed
  đóng kết nối/abort theo exception path hiện hành, không giả giữ người chơi trong trận. Send là recordingstub,
  không TCP/client. Direct model giữ claim sau exception khác lifecycle service thật, ghi riêng.

## Runtime đối chứng guard

`cycles-recipe.json`: exact argv, controller một lease/private user/net/mount/PID/lo-only/no route/
files-only NSS; A/B prefix copy riêng, bản copy client và profile47 giữ nguyên. Source/probe/profile
freeze ở `preflight.json`/`runtime-source`; không đổi giữa run. Sourcebinary/pack/DB giữ nguyên.
Runtime copy `e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef`, static
`660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3`, modO raw/base400000
`a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5`, RVA=VA−400000.

Run438,200581995s/SHA
`399d12f6ba763af74aa56e5e76ea8309fc8e54b1583b6ce55a94107878be3217`: service/controllererror0;
actual4trap17/2death19 được nhận, duplicate/old report bị loại riêng; hai native121/clock/175/180
cảhai view,12 entity/12 completed158 mỗi view, hai outcome `post_blast_untrapped_drained`.
Sau return2 hai native refresh110/role đủ6/6/lỗi0; A Ready actual95seq823 rồi110 cảhai. Ảnh đầu đen,
unfocused4s đã vẽphòng, afterReady8s cóready cảhai; Codex đã xem `guard-return2-contact.png`.
Không Start3 hoặc coi đây là fullmatch/K6a/LAN/Internet. Guardnegative không được gửi vào client trong run này.

`runtime-verification.json`:2.375 quan hệ, gồm2.296 originalraw size/SHA; `verification.json`:34joins
matrix/native/corpus/socket. 161 server tests gồm4 test mới/cases malformed high-tick/firstseen,
no marker/cursor/history/forward, claim còn dùng được và later validdeath. Những fixture test cũ dùng
arbitrary `fixture-tail` được thay bằng measured serialized envelope; giữ acceptance freshness/result/lifecycle.
Lần đầu imported TestCase làm suite đếm168 do chạy lặp7test; đã sửa importmodule, final161 là số đúng.
Firstmatrix JSON bytes-serialization failure chỉ có trong transcript; analysis-corrections.json ghi lại
nguồn và sửa serializer, không giả có originalstderr local hoặc sửa raw/expectedoutcome. Native negative
reader có thể giữ byte model cũ khi lỗi1; không dùng inner của ca lỗi làm validdecode.

Kiểm emulation/framer/typedpacket/securefields/flags tách native runtime. Exact fields/innerlife semantics,
firstseen hợp cấu trúc, forged-forward/tick/location/inventory/fullauth vẫn unknown hoặc ngoài policy.
Bước kế tận dụng CP036–038 report1/18/0: service hiện chưa forward escape/rescue hoặc quản lý kim;
phải nối inventory/phase bằng actual controls trước chọn policy, không tự coi result21 là rescue.

Bổ sung CP113: [kim145 opt-in/currentruntime](../needle-service/README.md) đãcó; bướcđầu CP112ởtrên làlịch sử, thứtự hiện hành chỉPLAN.
