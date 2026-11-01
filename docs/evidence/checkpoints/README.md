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

Mỗi 4 checkpoint complete: đọc README→PLAN→PROTOCOL→RESEARCH→evidence rồi đối chiếu raw/report/runtime.
Ghi AUD ID liên tục, trigger,completed_through,auditor,read_files theo thứ tự,findings/sửa ở đâu,
remaining. Record audits/ được hash trong manifest. Không dùng chạy script thay cho audit.

Mâu thuẫn nghiêm trọng audit ngay. AUD001 là initial_reconciliation tại CP001;ad-hoc audit không
reset lịch CP004,CP008,CP012… . Lấy counter/next_periodic_audit từ manifest,không đếm lại theo phiên.
Complete CP004 thiếu periodic auditCP004 gây lỗi MANUAL AUDIT DUE;phải audit và sửa docs bị ảnh hưởng.
Phần mềm không chứng minh người ghi đã đọc/thấu hiểu;reviewer phải kiểm report.

## Kiểm và giới hạn

Trên host có raw: python3 tools/check_checkpoints.py --require-local --require-history,
cùng python3 tools/check_docs.py và git diff --check. Checker dùng lại check_docs để kiểm Markdown.

Kiểm ID/previous/counter,field/source/status,owner khác reviewer,docs_sync links,
file/size/SHA,taskclosed/nextactive,cadence và append-only qua mọi revision manifest trên
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
Bàn giao ghi ID/counter,audit kế,pending,kết quả kiểm,commit/PR,blocker.


Từ CP048, tài liệu repository `.md` đã được pin ở checkpoint cũ có thể tiếp tục chỉnh hiện trạng.
Nếu bytes hiện tại khác, checker phải tìm **đúng size/SHA lịch sử trong Git** rồi báo rõ
`HISTORICAL DOC VERIFIED` kèm revision; không dùng file hiện tại thay bằng chứng cũ. Không tìm
được bytes cũ vẫn lỗi. Quy tắc này chỉ cho documentation repository, không cho code/raw/binary/
local evidence; record cũ không sửa, không nới hash hoặc tự suy ngữ nghĩa đồng bộ. Clone thiếu
Git blob cần khôi phục lịch sử để kiểm, không tạo bytes thay thế từ docs mới.
