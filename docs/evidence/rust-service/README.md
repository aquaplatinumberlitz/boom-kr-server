# Khảo sát để lập kế hoạch dịch vụ Rust KR00967

KRE-074/[#285](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/285), Codex solo, independent review waived.
[Chỉ đạo](../owner-directives/rust-service.json), [PLAN milestone/acceptance](../../PLAN.md#rust-service-milestones),
[kiến trúc/ma trận hiện có→Rust→thiếu](../../RUST_SERVICE_ARCHITECTURE.md),
[unknown/phép thử](../../RESEARCH.md#rust-service-gaps).

Baseline main c0c7b9478dcf5b639a76f67ee51a5f6b927cb0f9/CP212/PR284. CLIlatest xác nhận trước chuyển việc;
K0–K7 scoped đạt, K6c WAN giả lập, **Internet thật chưa kiểm**. Mọi Rust milestone chưa triển khai/nativeRustchưađo.
Code team variants còn WIP, không commit trong planning PR; kết quả sealed trước request giữ ở CP213/eight-player.
Không source/config/input run nào bị sửa; sáu run có rawseal/sourcefreeze/ownedcleanup0 trước khảo sát.

[Survey](survey.json) pin63sourcefiles/5repo commits+archive/wiki và34completedrecords, sáuWIP native runs.
Rawsurvey/sourcecopies/WIPhash ở local/research/codex/rust-service-survey-r1/. Đây là khảo sát read-only:
hash/source snapshot không proof nativeKRsemantic. Wronginitialpath needle_escape.py/rescue_contact.py được sửa
về test_needle_escape.py/test_rescue_contact.py thật; logic actual ởProbeService, không inventmodule.
Không chạy test/serve boom-rs hoặc sửa nguồn/router của repo khác; không lấy các tỷ lệ V1 của nó làm tiến độ KR.

Các quyết định reuse/kiến trúc/ma trận giữ duy nhất trong tài liệu kiến trúc; workorder/acceptance chỉ PLAN.
Không xây tracker/tool mới: existing catalog/docs/checkpoint/issue checkers kiểm link/declaration/hash/history,
không tự kiểm semantic kiến trúc hoặc gate. Nativefutureacceptance cần build/hash/recipe/raw mới trên Rust.
Đợt này không cargo build/test/nativeRust vì không tạo Rust; Python366server/13bundle+5preset kiểm WIP đãđo
trước khi nhận chỉ đạo. Compileplanning không là mục tiêu; full sourceWIP giữ exact hashes và không stage.

Code/delivery scope: planning PR chỉ docs/metadata/manifest; đầy đủ lịch sửCP/audit/raw không rewrite.
CP213 gom milestone RE đã seal và planning request, không CP cho mỗi thao tác; AUD062qua212, audit kếCP216.

Kiểm trước commit phát hiện refreshscript dùng lại tên raw APIcp213 sau khi pin. Đã khôi phục issuepage từ
object trước được giữ và pullpages về metadatarepo.size31340 của response cũ; **toàn bộ byte/fileSHA đúng
expectedhash đã ghi** trước khi thay. Response mới giữ riêng cp213-after-epic-*.json. Không sửa recordcomplete/
evidencemanifest/audit hoặc kết luận; all9233newpins kiểm lại size/hash khớp. Incident/recoverymethod ở
local/research/codex/rust-service-survey-r1/snapshot-recovery.json. Các capture sau dùng tên mới, không ghi đè rawpin.

Rà design trước PR làm rõ idempotency key không gồm policyversion có thể đổi: pin policy vào match/result,
unique(match,player,rewardslot), versionchange không cấp lại; corrections qua ledger cóaudit. Cargo integration
tests đặt trong từng crate, không virtualworkspace root. Đây là hoàn thiện design, chưa implementation.
