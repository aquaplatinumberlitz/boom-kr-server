# CP048 — chỉ mục chức năng có nguồn

Owner Codex; review độc lập waived theo [nguồn chủ dự án](../checkpoints/review-waiver.json).
Catalog là refactor tổ chức kiến thức, không là phép đo client hay nghiệm thu gate.

[Catalog](../../features.json) giữ ID/key cơ chế, cây nhóm, dependency kịch bản và các trạng thái
khai báo kèm nguồn. [Coverage](../../COVERAGE.md) sinh bảng, cây, dependency và thẻ tra nhanh.
Khoảng trống/phép đo được trích từ [RESEARCH](../../RESEARCH.md#feature-gaps), thứ tự chỉ liên kết
[PLAN](../../PLAN.md#current-order), contract/giới hạn dẫn về [PROTOCOL](../../PROTOCOL.md).
[Manifest](../checkpoints/manifest.json) giữ checkpoint/review/merge/hash; không chuyển lịch sử vào catalog.

Đối chiếu: README/PLAN/PROTOCOL/RESEARCH hiện hành, evidence theo anchor cơ chế, raw/cache CP045–047
được trỏ từ bảng item/UDP, rồi [service CP048](../probe-service/README.md). Không quét lại binary,
không tái tạo bản kiến thức VN. Các nhóm item dùng lại bảng đã phân nhóm ở PROTOCOL, không coi từng
row ItemAttr là một cơ chế. `unknown`/`candidate` giữ rõ khi chỉ có table hoặc thiếu runtime.

Các cột độc lập: presence cơ chế hẹp; VN static/runtime reference; KR static; emulation oracle;
runtime Wine copy/Windows nguyên bản/LAN/Internet; implementation/delivery; integration; gate.
Scope cụ thể nằm ở nguồn dẫn. Code service và policy là design/emu_choice, không chứng minh
client có một server bên trong: hai hàng service để presence KR `unknown`. Gate hiện giữ open.
Trạng thái branch_pending chỉ ra code mới đang chờ PR, không phải approved/merged.

Kiểm: `python3 tools/feature_catalog.py` và `python3 tools/check_docs.py` kiểm file/anchor nội bộ,
ID/key stable so HEAD, dependency/parent DAG, enum/môi trường/required refs và bản sinh không stale.
ID đã dùng phải giữ lại (có thể retired); không repurpose mechanism key. External URL chỉ khai báo,
không fetch. Không kiểm ý nghĩa claim, không xác nhận runtime hoặc tự đóng gate; passed chỉ là khai
báo phải có acceptance reference riêng để người audit kiểm.

Negative tests: ID trùng/xóa/đổi cơ chế, cycle/missing dep, file/anchor mất, enum sai, positive thiếu
nguồn, thiếu môi trường, thiếu câu hỏi và gate thiếu acceptance. Các test là consistency, không oracle KR.
Không áp catalog lên snapshot checkpoint cũ. Audit định kỳ CP048 so bản sinh với nguồn chuẩn và raw.
