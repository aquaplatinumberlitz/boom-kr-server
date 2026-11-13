# CP109 — Liên kết docs ↔ GitHub và rà acceptance

[KRE-072/#178](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/178), baseline CP108/PR177
`985a01db1fa2db339d1f4e51008c719ad9188547`; Codex solo/tự kiểm/[review waived](../checkpoints/review-waiver.json).
CP109 chờ PR. Raw runtime cuối đã complete/cleanup/save/hash trước đổi việc; [kết quả RE](../promoted-outcome/README.md).
Audit AUD034 đối chiếu issue qua109; periodic AUD033 qua108/kế112, không reset counter hoặc sửa completedcheckpoint.

[Review có ngày](review.json) giữ đối chiếu acceptance từng issue (feature/KRE/known/remaining/nexttest/
acceptance/reason/PR/evidence/successor). [Snapshot GitHub](github-snapshot.json) ghi UTC lấy dữ liệu và
trạng thái lúc đó; **không là trạng thái live**. Raw before/after bodies/PR metadata/timeline162/actions
và hash ở `local/research/codex/issue-reconciliation-r1/`; không tạo plan hay tracker thứ hai.

| Quyết định thủ công | Issue | Căn cứ/giới hạn |
|---|---|---|
| Đủ scope hẹp, đóng | #58, #60, #102, #104, #110, #162, #176 | CP049+075/085guard; CP050EOF; CP081power2chain; CP074nativecleanup; CP085TCP2cycles; CP101cachedselector; CP109promoted2nativecycles. Review.json dẫn source/PR cụ thể, không dựa checkpoint tồn tại là đủ. |
| Được thay thế, đóng notplanned | #88, #106 | #88adjacent sang#152/CP096, distance/control sang#102/CP081, remaininggeneralchain về#14. #106auxlocaleattributionđãđoCP073 nhưng startupmaincausechưafix, chuyển#9/#14; khônggọiLocaleđãsửa. |
| Giữ backlog/cập nhật scope | #1, #2, #3, #9 | Windows/control/bootstrap residual; fullcodec/livebounds; intermittentmainstartup. Worker/review cũ là lịch sử, không currentinstruction. |
| Giữ active | #14, #178 | #14epic checklist/childscope/canonicalgap, #178deliveryđợtreconcile quaPR. Thứ tự chỉ [PLAN](../../PLAN.md#issue-order). |

#162 GitHubtimeline có cross-referencePR163 nhưng không closeevent; PR163 đãmerge/code/socketpair5cases/
129tests đủ đúngacceptance, manifestcurrentđãclosed. Đợtnày đóngGitHub sau đọc bằngchứng và cập nhật
currenttask_states tại CP109; **không sửa CP101 hoặc record complete**. Nhiều taskcurrentmanifest cũ
cònopen trongkhiGitHubclosed cũng được sync theo snapshot, không biến lịch sử thànhlive status.

Mỗi openissue có featureID, known/gap/nexttest/acceptance, links PLAN/PROTOCOL/RESEARCH và PR/evidence;
[Catalog](../../COVERAGE.md) sinh feature→issue links, issue→canonicalrefs. Issue#14 giữepic/checklist,
khôngnhétcontract/plan độc lập vào issue. Sửa mô tả trước bằng cập nhật bodycóngày, trướcđó archive
body/raw/SHA; không xoá lịch sửGitHub event/checkpoint/evidence.

`python3 tools/check_issue_links.py` kiểm snapshot↔currentdeclarations/catalog/PLANmarkers/linkexistence/
PRstate+backlinks/closedtask-active-step. `--live` chỉreadGitHub đểso trạng thái thật; `--capture PATH`
chỉlưu datedsnapshotlocally, khôngwritesGitHub. Snapshot cap1000 phảifail nếuchạmcap thaygiảexhaustive.
Checker không đọc ngữ nghĩa raw, xácnhận acceptance, suy blocked, tựcloseissue haygate. Snapshot >24h
warning, rawcompletehistorykhôngso nhưlive taskstatus. `check_docs.py` gọi check này offline.

Đóngtask khôngđónggate K0–K7. #176nativeacceptanceđủ12actualentities/2outcomes, không14optionalbom/
finalsecondreturnresponsive. Đợt docs này xong phảitiếp #14theoPLAN: focus/wait/windowownership/native95
sau return2; Y389 đã giải, khônglàm lại oldwork. PRdelivery/selfcheckkhônglàindependentapproval.


CP110 delivery/currentstate update: PR179 merged `8f3ea2f`; #178 đóng sau merge vàđãtiếpRE,
[CP110UI/Ready](../../PROTOCOL.md#return2-visual) là kếtquả thực. Current snapshot lấy UTC trongJSON,
#1/#2/#3/#9/#14 cònopen; #178completed, khôngcònPLANactive. CP109snapshotpending/raw đượcpin
local/cp109-snapshot, khôngsửa complete record. Checker live đãbáo3lệch sauclosure trước refresh;
loggiữởlocal/return2-visual-r2/github-pending-refresh-check.txt, khôngsửa lời kết đểcheckerpass.

[Đính chính sau CP298](cp298-correction.json) giữ lỗi snapshot cũ và commit công bố đầu tiên.
Snapshot/review hiện hành đã đối soát PR399/401 và issue402/403; complete record, audit và raw cũ giữ nguyên.
