# Quy tắc project Boom KR Server

Repo này là nguồn chuẩn riêng cho server tương thích **client KR gốc 00967**.
Các quy tắc/scope V1 của `boom-rs` không tự áp dụng lên project này. Lệnh mới của chủ dự án được ưu tiên.

## Nguồn chuẩn và cách bắt đầu

Đọc `README.md` → `docs/PLAN.md`, rồi `docs/PROTOCOL.md`/`docs/RESEARCH.md` đúng task.
Chỉ `docs/PLAN.md` giữ lộ trình; không tạo plan theo phiên. Protocol/authority ở PROTOCOL, câu hỏi mở
ở RESEARCH, nguồn cũ/migration ở REFERENCES, metadata bằng chứng ở `docs/evidence/`.
Docs viết tiếng Việt; ID/API/type kỹ thuật giữ nguyên.

Mọi code/docs/tool mới thuộc repo này: `server/`, `tools/`, `docs/`. Không tiếp tục ghi bản tương ứng
vào boom-rs, boom-re hoặc boomkr-re. Không nhập nguyên code legacy; dùng làm reference có provenance.

## Quyền và phạm vi

Chủ dự án đã cho triển khai tiếp và toàn quyền thao tác phục vụ project. Không hỏi lại quyết định đã có.
Mục tiêu offline-preservation, identity/server local; không login/capture server Nexon thật.
Trước runtime, cô lập outbound của VM/client, chỉ cho host/stub/capture cục bộ theo allowlist.

Không sửa client/pack/IDA DB nguồn. Bản copy/DB riêng/capture/raw/decompile/patch thử đặt trong `local/`
(git-ignored); metadata/hash mới commit. Theo KRE-D01 trong PLAN, được chọn stub, loại AC hoặc patch
file trên bản copy offline khi cần, có evidence, before/after, control run và rollback; không phát hành
binary/asset/AC. Không suy modified-client behavior là nguyên bản. Tránh lặp hook anti-tamper đã thất
bại nếu chưa có điều kiện mới.

## Bằng chứng và chất lượng

Mỗi kết luận có source (`kr_static`, `kr_runtime`, `vn_static`, `vn_runtime`, `emu_choice`, `design`)
và trạng thái (`confirmed`, `candidate`, `unknown`, `design`), hash/build, VA/RVA hoặc run/capture và giới hạn.
VN confirmed dùng cho luật chung; opcode/key/layout/ID theo build phải kiểm KR. Không dùng golden tự
sinh làm oracle, không dùng log emulator chứng minh client thực thi hoặc luật server gốc.

Chưa biết thì ghi unknown và bước kiểm; không bịa wire/default. Gate K0–K7 qua được khi có evidence,
verification và reviewer độc lập. Duyệt plan không phải hoàn thành implementation/runtime.

## Làm việc nhóm

Task là issue trong chính repo này, có KRE ID, baseline, owner/reviewer, dependency, file và acceptance.
Một owner mỗi file/target/runtime. Codex điều phối/code/runtime. Theo chỉ đạo chủ dự án27/10,
**Claude Sonnet5.5** thay dsh1/dsh2 đã hết token cho các việc RE và review tiếp theo; không giao thêm
việc cho hai dsh. Khi gọi worker, chọn đúng model claude-sonnet-5-5 và chỉ rõ task/cwd/owner/output.
Review phải độc lập với tác giả; các report dsh cũ vẫn giữ provenance lịch sử. Ca khó checkpoint sớm
để Codex vào phụ, không tự quét vô hạn.
Lượt RE mặc định tối đa8 request/4 thân hàm mới, hai lượt; mở tiếp khi có cạnh/thí nghiệm mới rõ.

Code trên nhánh/worktree riêng của repo này; giữ thay đổi người khác, không reset/clean hoặc stage toàn
repo. Commit task của mình và kiểm diff. Sau commit khởi tạo repo, tích hợp main qua PR, kiểm tra phù
hợp và reviewer khác tác giả approve đúng head; không tự merge code chưa review.
Không push binary/decompile/credential. Không đổi cấu hình cá nhân/IDA/MCP trong repo.

## Kiểm và bàn giao

Docs: `python3 tools/check_docs.py` và `git diff --check`.
Tool/codec/server: chạy kiểm phù hợp hành vi thay đổi; không mặc định phải build client Rust cũ.
CI/build server hiện chỉ Linux. Theo yêu cầu26/10, ưu tiên Wine cho bản copy unpack/patch đã đo;
Windows VM giữ vai trò đối chứng KR nguyên bản. Không suy modified Wine tương đương vanilla Windows.
Bàn giao nêu commit/file, nguồn đã đọc, lệnh kiểm+kết quả, evidence và blocker/bước tiếp.
