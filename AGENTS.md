# Quy tắc project Boom KR Server

Repo này là nguồn chuẩn riêng cho server tương thích **client KR gốc 00967**.
Các quy tắc/scope V1 của `boom-rs` không tự áp dụng lên project này. Lệnh mới của chủ dự án được ưu tiên.

## Nguồn chuẩn và cách bắt đầu

Đọc `README.md` → `docs/PLAN.md`, rồi `docs/PROTOCOL.md`/`docs/RESEARCH.md` đúng task.
Chỉ `docs/PLAN.md` giữ lộ trình; không tạo plan theo phiên. Protocol/authority ở PROTOCOL, câu hỏi mở
ở RESEARCH. `docs/features.json` chỉ mục ID/cây/dependency/mức bằng chứng, không là plan hoặc contract;
`docs/COVERAGE.md` sinh bằng `python3 tools/feature_catalog.py --generate`, trích khoảng trống/phép đo từ RESEARCH.
Khi đổi cơ chế/trạng thái/câu hỏi, đồng bộ catalog rồi sinh lại coverage; ID không xóa/repurpose (retired giữ lại).
Nguồn cũ/migration ở REFERENCES, metadata bằng chứng ở `docs/evidence/`.
Docs viết tiếng Việt; ID/API/type kỹ thuật giữ nguyên.

Theo [chỉ đạo công ty08/11](docs/evidence/owner-directives/rust-service.json), chuyển từng contract ổn định
sang Rust, giữ Python RE/reference; [PLAN](docs/PLAN.md#rust-service-milestones) giữ thứ tự. Đợt KRE-074
chỉ khảo sát/thiết kế, không tạo Rust source/repo/worktree hoặc đổi runtime. Khi mở implementation sau này,
vị trí thiết kế là rust/ trong repo này theo [kiến trúc](docs/RUST_SERVICE_ARCHITECTURE.md); không nhập
scope/wire/authority của boom-rs. Quyền triển khai của đợt khác không bỏ giới hạn planning-only lần này.

Mọi code/docs/tool mới thuộc repo này: `server/`, `tools/`, `docs/`. Không tiếp tục ghi bản tương ứng
vào boom-rs, boom-re hoặc boomkr-re. Không nhập nguyên code legacy; dùng làm reference có provenance.

## Quyền và phạm vi

Chủ dự án đã cho triển khai tiếp và toàn quyền thao tác phục vụ project. Không hỏi lại quyết định đã có.
Theo [yêu cầu trực tiếp01/11](docs/evidence/owner-directives/continuous-work.json), hoàn tất một checkpoint/commit/PR
thì **tiếp tục ngay công việc kế tiếp theo PLAN**, không tự dừng hoặc chờ chủ dự án nhắc lại.
Blocker một nhánh phải ghi rõ rồi tiếp việc độc lập có thể làm; giữ nguyên evidence, audit, kiểm tra và PR.
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

Chưa biết thì ghi unknown và bước kiểm; không bịa wire/default. Gate K0–K7 vẫn cần evidence và
verification; từ28/10 chủ dự án miễn review độc lập, Codex tự kiểm. Miễn review không tự đóng gate.
Duyệt plan không phải hoàn thành implementation/runtime.

## Checkpoint và tài liệu RE bắt buộc

Trước mỗi nhánh RE mới, đối chiếu nguồn theo đúng câu hỏi: docs/evidence/cache hiện hành →
boomkr-re → boom-re; thêm boom-rs, boom-re-l10n và archive khi có nguồn liên quan. Theo router
và migration tới nguồn thật; không dùng snapshot cũ làm scope của repo này. Ghi ngắn nguồn/build/
hash/locator, điều đã biết cùng mức static/emulation/runtime/policy, phần dùng trực tiếp/phải đối chiếu
KR, khoảng trống và phép đo tối thiểu. Chỉ RE phần khác biệt/còn thiếu. Bảng đối chiếu gọn ở PROTOCOL,
phép thử tiếp ở RESEARCH, provenance/hash ở evidence; không tạo plan hay bản kiến thức độc lập theo phiên.

Ưu tiên RE động dẫn đường và thử nghiệm tiếp khi có giả thuyết cụ thể; gom các phép đo liên quan
thành một checkpoint tại mốc quan trọng hoặc blocker rõ ràng, không tạo checkpoint/PR cho mỗi phép thử.
Mỗi checkpoint phải đồng bộ docs bị ảnh hưởng trước khi complete;không chỉ giữ kiến thức trong
hội thoại/report worker/checkpoint local. README current state,PLAN task/gate/thứ tự,PROTOCOL
contract/limits,RESEARCH câu hỏi/phép kiểm tiếp,evidence metadata/recipe/hash; raw ở local.
Tóm tắt delta và link nguồn chuẩn,không tạo plan/contract theo phiên.
Theo [workflow](docs/evidence/checkpoints/README.md),lấy ID/counter tăng liên tục từ
[manifest](docs/evidence/checkpoints/manifest.json),kể cả đổi phiên/compact/worker.
Từ kho v2, dùng `python3 tools/checkpoints.py latest`, rồi `show CPxxx`/`evidence CPxxx`
(summary/page/`--verify`). Không cat toàn manifest/records/raw index vào context; đọc đúng câu hỏi.
Record complete và run/batch evidence manifest đã pin giữ bất biến; [hướng dẫn](docs/evidence/checkpoints/README.md).
Phân biệt source/status/loại kiểm với review/merge/gate;chưa review vẫn ghi pending/candidate.
Không nâng gate từ alive,TCP accept,cửa sổ hoặc encoder tự sinh;không báo phần trăm thiếu bằng chứng.
Mỗi 4 checkpoint complete audit thủ công README→PLAN→PROTOCOL→RESEARCH→evidence với raw mới,
ghi audit/counter bền vững. Mâu thuẫn nghiêm trọng audit ngay,không reset lịch4 checkpoint.


## Làm việc nhóm

Task là issue trong chính repo này, có KRE ID, baseline, owner/reviewer, dependency, file và acceptance.
Một owner mỗi file/target/runtime. Theo chỉ đạo mới nhất28/10, Codex solo RE/code/runtime và tự kiểm;
Claude đã hết token, không gọi worker/reviewer nữa. Chủ dự án nói “claude hết token rồi nên mày tự solo đi ko cần review”;
nguồn quyết định ở [review waiver](docs/evidence/checkpoints/review-waiver.json).
Review/report cũ giữ nguyên provenance. Mốc mới ghi review `waived` kèm nguồn chủ dự án, không ghi
`approved` hoặc giả người review. Các vấn đề chất lượng đã phát hiện vẫn cần sửa và ghi chép.
Ca khó checkpoint theo mốc/blocker, không tự quét vô hạn.
Lượt RE mặc định tối đa8 request/4 thân hàm mới, hai lượt; mở tiếp khi có cạnh/thí nghiệm mới rõ.

Code trên nhánh/worktree riêng của repo này; giữ thay đổi người khác,  không reset/clean hoặc stage toàn
repo. Commit task của mình và kiểm diff. Tích hợp main qua PR sau kiểm tra phù hợp; theo miễn review
28/10, Codex được tự merge trong phạm vi đã cấp quyền. Không gọi việc này là review độc lập.
Không push binary/decompile/credential. Không đổi cấu hình cá nhân/IDA/MCP trong repo.

## Kiểm và bàn giao

Docs/checkpoint: chạy python3 tools/check_docs.py,python3 tools/check_checkpoints.py --require-local
--require-history và git diff --check. Thiếu raw/pending/unsynced phải báo rõ;checker không tự xác nhận
ngữ nghĩa hoặc đóng gate.
Tool/codec/server: chạy kiểm phù hợp hành vi thay đổi; không mặc định phải build client Rust cũ.
CI/build server hiện chỉ Linux. Theo yêu cầu26/10, ưu tiên Wine cho bản copy unpack/patch đã đo;
Theo [chỉ đạo chủ dự án](docs/evidence/owner-directives/wine-accepted.json), Wine được chấp nhận làm môi trường
nghiệm thu thay Windows thật cho project; không yêu cầu WindowsVM/máyWindows hoặc chạy lại các gate
chỉ để có môi trường đó. Ghi đúng Wineversion/hash/copy/patch và nativeevidence để tái lập; không sửa
lịch sử hoặc mô tả sai môi trường run chưa chạy. [LAN giả lập](docs/evidence/owner-directives/emulated-lan.json)
cũng được chủ dự án chấp nhận thay hai máy vật lý; giữ rõ topology/interface/IP/capture/giới hạn.
Theo [chỉ đạo WAN giả lập](docs/evidence/owner-directives/emulated-wan.json), K6c giữ ID và nghiệm thu bằng
WAN giả lập thay Internet thật; các tiêu chí hành vi giữ nguyên. **Internet thật chưa kiểm**; kiểm Internet/ISP/CGNAT
thực tế là backlog riêng, không dependency K6c/K7. Không suy chỉ đạo môi trường tự đóng gate.
Bàn giao nêu commit/file, nguồn đã đọc, lệnh kiểm+kết quả, evidence và blocker/bước tiếp.
