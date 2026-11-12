# Checkpoint RE và audit tài liệu

Quy trình ghi chép bắt buộc, không phải plan khác. Lộ trình ở [PLAN](../../PLAN.md).
Counter/record duy nhất: [manifest.json](manifest.json). CP001 là mốc đầu áp dụng27/10;
không backfill thí nghiệm cũ thành checkpoint giả. Phiên/worker/compact tiếp tục từ last_id.

| Thông tin | Nguồn chuẩn |
|---|---|
| Trạng thái,mốc,điểm chặn | [README](../../../README.md) |
| Task/gate/thứ tự | [PLAN](../../PLAN.md) |
| Contract KR/limits | [PROTOCOL](../../PROTOCOL.md) |
| Câu hỏi/phép kiểm/owner kế | [RESEARCH](../../RESEARCH.md) |
| Recipe/run/kết quả/hash | docs/evidence/ |
| Checkpoint delta/counter/audit/sync | Manifest này và audits/ |
| Binary/copy/DB/disasm/log/capture/report đầy đủ | local/ hoặc source bất biến,không commit |

## Hoàn tất và review

Chọn ID kế không gap,previous trỏ mốc trước; lifecycle in_progress trong lúc làm.
Record ghi question/scope,context commit,artifactSHA,VA/RVA/run,findings(source/status/method/limits),
verification(check/result/evidence/hash),delta kể cả correction/supersession,
blocker,next-step/owner/reviewer và docs_sync cho4 docs chính+evidence.
Tóm tắt delta và link contract/next-step chuẩn,không tạo bản kiến thức độc lập theo phiên.

Đồng bộ docs bị ảnh hưởng trước lifecycle complete. Doc không đổi ghi unchanged+kèm lý do.
Thiếu sync phải ghi blocked/lý do; checkpoint còn incomplete.
Complete chỉ nói record và docs đầy đủ,không nói RE đã thành công,đã review/merge hoặc đã qua gate.
Chưa review vẫn ghi kiến thức và review.pending/reference,không đợi merge.

Review độc lập đúng commit qua PR. Review có as_of_utc,approved phải có exact head.
Có thể bổ sung review pending về sau; approved (kể cả head) đã commit thì bất biến.
Đổi/rút approval phải ghi checkpoint correction mới, không hồi quy ngầm.
Không sửa findings/evidence của checkpoint đã complete.
Correction khoa học là checkpoint tiếp,chỉ rõ claim cũ bị sửa;không sửa raw cũ để làm check pass.
Review/integration là snapshot có ngày;PR là nguồn trạng thái tích hợp hiện tại,checkpoint sau ghi
merge khi có. Không tự gán commit của chính file đang viết hoặc tự approve.

Từ chỉ đạo chủ dự án28/10, review độc lập được miễn theo [nguồn](review-waiver.json). Mốc mới hoặc
review pending được đổi thành `waived`, có `authorization.by=project_owner`, reference tới file nguồn
và reason rõ ràng. Codex tự kiểm, không đổi waiver thành `approved`; approval lịch sử vẫn bất biến.
Waiver đã commit cũng bất biến; thay chính sách về sau ghi correction checkpoint. Checker chỉ kiểm
khai báo/nguồn tồn tại, không tự xác thực tác giả hoặc cấp quyền miễn review. PR và kiểm tra vẫn giữ.

Task_states là snapshot khai báo rõ các task liên quan,link PLAN/issue. Next-step cùng ID có thể
đóng/supersede ở checkpoint sau,không viết lại lịch sử. Task closed không được có next-step active
ở trạng thái cuối. Không tự đóng task/gate từ checker.

## Audit thủ công

Theo [chỉ đạo chủ dự án11/11](../owner-directives/feature-group-audits.json), từ sauCP256 **bỏ lịch audit mỗi4checkpoint**.
Audit khi hoàn tất một nhóm tính năng: đọc README→PLAN→PROTOCOL→RESEARCH→evidence rồi đối chiếu raw/report/runtime.
Ghi AUD ID liên tục, `trigger=feature_group`, `feature_groups`, `completed_through`, auditor, read_files theo thứ tự,
findings/sửa ở đâu và remaining. Checkpoint hoàn tất nhóm khai `completed_feature_groups` tương ứng; checker yêu cầu
một audit cùng checkpoint cho nhóm đã khai hoàn tất. PLAN vẫn là nguồn scope/thứ tự, không nhân bản thành plan nhóm riêng.
Không dùng chạy script thay audit thủ công; checker không tự biết nhóm đã hoàn tất về mặt ngữ nghĩa.

Mâu thuẫn nghiêm trọng vẫn audit ngay bằng `serious_contradiction`, không tự coi đó là audit hoàn tất nhóm chưa kiểm.
AUD001 initial và mọi audit/record/raw lịch sử giữ nguyên. `audit_policy` pin chỉ đạo/chuyển tiếp sauCP256;
`audit_interval`/`next_periodic_audit` hiện là null, `last_periodic_audit=256` chỉ là counter lịch sử.
Checker giữ nghĩa vụ lịch4CP trước chuyển tiếp, không ép CP260 hoặc số checkpoint cố định mới.

Phần mềm không chứng minh người ghi đã đọc/thấu hiểu;reviewer phải kiểm report.

## Kiểm và giới hạn

Trên host có raw: python3 tools/check_checkpoints.py --require-local --require-history,
cùng python3 tools/check_docs.py và git diff --check. Checker dùng lại check_docs để kiểm Markdown.

Kiểm ID/previous/counter,field/source/status,owner khác reviewer,docs_sync links,
file/size/SHA,taskclosed/nextactive,audit theo policy chuyển tiếp và append-only qua mọi revision manifest trên
first-parent Git history rồi working tree. Xóa checkpoint đã commit rồi commit reset vẫn lỗi.
Không bảo vệ được cố ý rewrite/xóa toàn Git history;archive/shallow thiếu lịch sử phải nêu rõ.
Không đọc GitHub để tự đoán task;người ghi cập nhật snapshot có reference.

Clone thiếu local/artifact: không flag báo UNVERIFIED+số thiếu,không tuyên bố hash đã kiểm.
--require-local biến thiếu raw thành lỗi. Thiếu Git, chưa có manifest commit hoặc shallow history báo counter rollback UNVERIFIED;
--require-history biến thành lỗi. Pending review hiện WARNING dù exit0.
Waived review cũng hiện WARNING rõ là chưa review độc lập, không phải kiểm tra bị treo.
Complete có blocked docs,lệchhash hoặc audit tới hạn đều exit1.

Checker không đọc hiểu disassembly,suy diễn kiến thức,xác nhận ngữ nghĩa,đo runtime hoặc đóng gate.
Fragment link chỉ kiểm file,không kiểm anchor;semantic sync do audit/reviewer.
Tests tools/test_checkpoints.py là schema/negative cases,không phải protocol oracle.

`findings.method=test` ghi phép kiểm tự động của source/backend/codec hoặc so sánh tổng hợp
(CP259 dùng nhãn này cho hồi quy TCP và build/test ARM). Checker hỗ trợ thêm nhãn này và cảnh báo
rõ không phải chứng nhận native client; không suy thành `kr_runtime` hoặc `confirmed`, không đổi
`source`/`status`, kiểm hash, continuity, audit hay acceptance. Các method cũ giữ nguyên nghĩa.
Đây là mở rộng schema trước khi giao PR331; record CP259 đã commit giữ nguyên.
Bàn giao ghi ID/counter,audit kế,pending,kết quả kiểm,commit/PR,blocker.


Từ CP048, tài liệu repository `.md` đã được pin ở checkpoint cũ có thể tiếp tục chỉnh hiện trạng.
Nếu bytes hiện tại khác, checker phải tìm **đúng size/SHA lịch sử trong Git** rồi báo rõ
`HISTORICAL DOC VERIFIED` kèm revision; không dùng file hiện tại thay bằng chứng cũ. Không tìm
được bytes cũ vẫn lỗi. Clone thiếu Git blob phải khôi phục lịch sử, không tạo bytes thay thế.

Từ CP172, source Python của chính project ở `server/` hoặc `tools/`, `kind=repository`, còn tồn tại
trong working tree có thêm kiểm phiên bản hẹp: tìm commit trên lịch sử HEAD lần đầu thêm record v2
`records/CPxxx.json`; commit đó phải có record complete khai báo **đúng path/size/SHA** và source blob
cùng size/SHA. External evidence manifest tại commit ấy phải khớp pin/hash/count. Checker báo
`HISTORICAL SOURCE VERIFIED` với revision; không dùng source hiện tại thay source checkpoint và
không lấy blob/khai báo thêm ở commit về sau để cứu một pin thiếu lúc công bố. CP166 là ca đầu:
pin `server/probe_service.py` khớp nguyên bytes commit `09759d0`, record/evidence cũ không sửa.

Đây là kiểm nguồn theo version, không chứng nhận runtime, semantics hoặc gate. Bằng chứng raw/local/
artifact/binary và repository file khác vẫn cần bytes hiện tại khớp pin; mất file vẫn lỗi/unavailable.
Source chỉ có ở schema v1 hoặc không khớp lúc thêm record v2 **chưa** được fallback tự động; giữ lỗi
và cần locator/migration minh bạch nếu gặp. Append-only records/history qua schema v1/v2 giữ nguyên.

## Kho checkpoint v2 và cách đọc chọn lọc

Từ CP118, `manifest.json` là **index nhỏ** (`kre-checkpoints-v2`, version2): counter, audit,
current task declarations và ID→`records/CPxxx.json`/size/SHA/lifecycle/ngày. Không giữ bản
current-status hoặc thứ tự việc khác; chỉ [PLAN](../../PLAN.md#current-order) giao việc.
Records CP001–117 chuyển nguyên JSON object; findings/review/waiver/inline evidence không sửa.
[Report migration](migration-v2.json) giữ baseline commit/SHA, mapping từng record, object digest,
đối chiếu toàn bộ evidence/audit và kích thước. Audit lịch sử vẫn trong `audits/`; [đo kích thước/file count](storage-validation-v2.json)
phân biệt mức giảm index với tổng dữ liệu legacy được giữ.

**LLM bắt đầu bằng CLI, không `cat` toàn manifest/history/raw index:**

```bash
python3 tools/checkpoints.py latest
python3 tools/checkpoints.py show CP117
python3 tools/checkpoints.py evidence CP117
python3 tools/checkpoints.py evidence CP117 --offset 0 --limit 20
python3 tools/checkpoints.py evidence CP117 --verify --require-local
```

`latest` chỉ đọc index/counter/đường dẫn PLAN. `show` mặc định bỏ inline raw index, giữ nội dung
checkpoint và số evidence; `--full` chỉ dùng khi thực sự cần **toàn bộ record đã chọn**. `evidence`
mặc định summary; page tối đa200 dòng. `--verify` kiểm **mọi evidence của CP**, kể cả ngoài page;
không in mọi dòng raw. `--root` đặt trước subcommand dùng cho fixture/checkout khác.
Kiểm toàn repo/lịch sử vẫn là `python3 tools/check_checkpoints.py --require-local --require-history`.

Checkpoint mới có thể dùng `evidence_manifests`:

```json
{
  "evidence": [],
  "evidence_manifests": [{
    "path": "docs/evidence/checkpoints/evidence/run-or-batch.json",
    "sha256": "<sha256 bytes của file JSON>",
    "bytes": 1234,
    "count": 42
  }]
}
```

File tham chiếu có `schema: kre-evidence-manifest-v1` và `evidence: [...]` dùng nguyên schema dòng
`id/kind/path/bytes/sha256`. Không nested manifest; ID evidence phải duy nhất khi ghép inline và
các manifest ngoài trong một CP. Findings/verification trỏ ID đó như trước. Một run/batch pin một
list, CP chỉ giữ path/size/SHA/count; có thể tái dùng cùng file đã pin thay vì sao chép list.
**Không deduplicate/viết lại evidence trong record complete cũ.** Binary/log vẫn ở local; chỉ metadata
được commit. Index dùng `checkpoint_store.record_reference` để pin file record sau khi docs đã sync.

### Tái chạy và giới hạn migration

Script `tools/migrate_checkpoints.py` nhận baseline v1 đã pin. Nó kiểm bytes nguồn khớp manifest
trong baseline Git, tách theo object (không chia dòng), so toàn bộ records/evidence, pin36audit,
refuse ghi đè record khác nội dung. Tái chạy giữ nguyên file/report hiện có và không reset các CP
mới đã append. Sau lỗi giữa chừng có thể còn một số file record đã tạo; tái chạy sẽ đối chiếu chúng
trước khi hoàn tất index. Không dùng migration như công cụ sửa checkpoint.

Recipe lấy **đúng baseline SHA trong report**, lưu vào local, không đổ JSON vào hội thoại:

```bash
git show c2ca94d1e2a77bbfcf67c052542e37d0a1577e94:docs/evidence/checkpoints/manifest.json > local/checkpoint-v1-baseline.json
python3 tools/migrate_checkpoints.py --source local/checkpoint-v1-baseline.json --baseline-commit c2ca94d1e2a77bbfcf67c052542e37d0a1577e94
```

Loader/checker hỗ trợ v1 lịch sử và v2 hiện tại. Checker đọc từng record và từng manifest evidence,
không cần LLM tải toàn kho; khi đọc revision v1, Python vẫn phải decode JSON cũ nguyên file.
Git history đọc **cả thư mục checkpoints/** trên first-parent, nên record/audit/evidence-only commit
cũng được kiểm: size/SHA ở revision đó, fingerprint nguyên object qua ranh giới schema, append-only,
review approved/waived bất biến. Chỉ giữ fingerprint nhỏ giữa revision, không giữ toàn bộ records lịch sử
trong bộ nhớ cùng lúc. Dữ liệu inline cũ vẫn lớn và lặp, cố ý giữ để bảo toàn object; tổng dữ liệu không
được hứa giảm theo tỷ lệ của index. Clone thiếu Git/local vẫn báo UNVERIFIED/error theo cờ cũ.


### Pin local tuyệt đối đã công bố ở CP187

CP187 có44evidencepin ghi đường dẫn host tuyệt đối trong checkout/local và khai báo kindrepository.
Đây là lỗi ghi metadata, được giữ nguyên trong record/evidence manifest complete. Checker đọc tương thích
chỉ khi đường dẫn sau resolve nằm trong **local của checkout hiện tại**, báo `LEGACY ABSOLUTE LOCAL PIN`
và kiểm size/SHA đúng như raw, không dùng Git fallback. Không chấp nhận absolute source/repository path,
path ngoài local hoặc symlink thoát. Đường dẫn relative mới vẫn bắt buộc kindlocal đúng; checkpoint mới
chuẩn hóa relative khi lập evidence list. Clone host khác thiếu đường dẫn này sẽ báo unverified/error
với require-local, không tự tìm file hay đổi record. Tương thích path không xác nhận kết luận RE hoặc gate.

Bổ sung tại PR của CP260: ba tài liệu JSON hiện hành `docs/features.json`,
`docs/evidence/issue-reconciliation/github-snapshot.json` và `review.json` có thể đổi sau checkpoint.
Pin cũ chỉ được xác minh bằng **đúng khai báo evidence và blob byte/size/SHA tại commit đầu tiên công bố
record v2 complete tương ứng**. Blob trùng ở commit về sau hoặc checkpoint khác không cứu được pin thiếu.
Checker báo `HISTORICAL METADATA DOC VERIFIED`; không đổi record/evidence manifest cũ hoặc dùng nội dung
hiện tại thay nội dung đã pin. Các JSON runtime/evidence, checkpoint/audit và file local/binary không thuộc
ngoại lệ này, vẫn kiểm nghiêm như trước. Đây là xác minh phiên bản tài liệu, không chứng nhận runtime/gate.

Từ CP268, checkpoint **thuần tài liệu** có thể khai `artifacts=[]` khi mọi finding là design/documentation và
status design/candidate/unknown; canonical docs/*.md phải có kind repository/size/SHA trong evidence, mỗi finding
tham chiếu tài liệu đã pin. Checker cảnh báo `DOCUMENTATION identity`, vẫn kiểm locator/hash/history/continuity/
audit/docs_sync. Claim runtime/static/emulation/test/confirmed không dùng đường này. CP267 record cũ bất biến;
lượt schema failure và correction giữ riêng ở [nguồn](../../PROTOCOL.md#documentation-checkpoint-identity).


<a id="publication-bound-source-history"></a>
### Source Rust và census sau khi checkpoint đã công bố

Từ CP278, quy tắc kiểm source tại lần công bố record v2 cũng áp dụng cho `.rs` trong
`rust/crates/*/src/`. `docs/content/census.json` dùng cùng phép kiểm metadata tại lần công bố như catalog.
Checker chỉ nhận khi record complete tại commit đầu tiên đã khai đúng path/size/SHA, evidence manifest ngoài
khớp hash/count và blob Git tại chính commit đó khớp từng byte. Blob giống ở commit sau không cứu được pin sai
lúc công bố. Đường dẫn có traversal, build tree, fixture Rust ngoài source, binary/raw/local và JSON evidence
khác không được mở rộng theo quy tắc này; raw/local vẫn phải khớp file hiện tại.

[Metadata correction](history-source-correction.json) giữ ba lỗi phát hiện khi kiểm CP277: census của CP275 và
hai source Rust của CP276 đã thay đổi hợp lệ nhưng checker cũ chưa hỗ trợ loại file đó. Các blob đúng hash đều
còn tại lần công bố; record complete, audit và evidence manifest cũ không sửa. Kiểm hồi quy gồm sai checkpoint,
pin chỉ khớp ở commit sau, manifest ngoài sai hash, artifact và đường dẫn ngoài phạm vi. Đây là kiểm provenance,
không xác nhận native hoặc tự đóng M7b.
