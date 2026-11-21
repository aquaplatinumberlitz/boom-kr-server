# Quy tắc project Boom KR Server

Repo này là nguồn chuẩn riêng cho server tương thích **client KR gốc 00967**.
Các quy tắc/scope V1 của `boom-rs` không tự áp dụng lên project này. Lệnh mới của chủ dự án được ưu tiên.

## Nguồn chuẩn và cách bắt đầu

Đọc [Bảng điều hành](docs/PLAN.md#dashboard) ở đầu `docs/PLAN.md` trước; đó là hiện trạng, thứ tự việc và blocker.
Phần còn lại của PLAN, `docs/PROTOCOL.md` và `docs/RESEARCH.md` chỉ tra theo anchor đúng task, không đọc nguyên file.
Nhánh RE và gate K0–K7 trước chỉ đạo Rust đã chuyển sang `docs/PLAN_ARCHIVE.md` (chỉ là lịch sử, không thêm mới).
Khi trạng thái hoặc blocker đổi, **ghi đè** dòng liên quan trong Bảng điều hành. Không nối dòng theo checkpoint.
Chỉ `docs/PLAN.md` giữ lộ trình; không tạo plan theo phiên. Protocol/authority ở PROTOCOL, câu hỏi mở
ở RESEARCH. `docs/features.json` chỉ mục ID/cây/dependency/mức bằng chứng, không là plan hoặc contract;
`docs/COVERAGE.md` sinh bằng `python3 tools/feature_catalog.py --generate`, trích khoảng trống/phép đo từ RESEARCH.
Khi đến kỳ cập nhật catalog theo quy định bên dưới, sinh lại COVERAGE; không xóa hoặc đổi nghĩa ID đã cấp.
Nguồn cũ/migration ở REFERENCES, metadata bằng chứng ở `docs/evidence/`.
Docs viết tiếng Việt; ID/API/type kỹ thuật giữ nguyên.

Theo [chiến lược công ty08/11](docs/evidence/owner-directives/rust-service.json), chuyển từng contract ổn định
sang Rust, giữ Python RE/reference; [PLAN](docs/PLAN.md#rust-service-milestones) giữ thứ tự. Đợt khảo sát CP213
đã giao. [Chỉ đạo implementation mới09/11](docs/evidence/owner-directives/rust-implementation.json) **cho phép
triển khai server Rust và RE song song khi cần**; giới hạn planning-only của đợt cũ không chặn đợt mới.
Bắt đầu M0→M1 ở `rust/` cùng repo, hai crate kr-wire/kr-service theo [kiến trúc](docs/RUST_SERVICE_ARCHITECTURE.md).
Không chờ fullauth/generalroom hoặc mở thêm broadRE trước codec/connection đã đủ contract; chỉ RE discrepancy/gap
chặn implementation/nghiệm thu. Không nhập scope/wire/authority của boom-rs hoặc triển khai giả thuyết như confirmed.

[Đích bàn giao giai đoạn 1 đã chốt 11/11](docs/evidence/owner-directives/delivery-target.json): dịch vụ nội bộ bền vững,
người chơi qua Internet thật, khoảng 50 người đồng thời, chế độ thường với nhiều map và item phổ biến; định hướng dài hạn
vẫn là gameplay đầy đủ. [PLAN](docs/PLAN.md#delivery-target) giữ đường cắt phạm vi, thứ tự và các mốc RUST-N1/R1/G1.
Lát cắt mới không thêm cờ theo sức chứa, alias theo identity hay nhánh `capacity() == N` vào lõi phòng; luật mới đặt
trong `RoomRules` (`rust/crates/kr-service/src/rules.rs`).

Mọi code/docs/tool mới thuộc repo này: `rust/`, `server/`, `tools/`, `docs/`. Không tiếp tục ghi bản tương ứng
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
provenance và hash ở evidence. Ghi phép thử tiếp vào RESEARCH khi đến mốc cập nhật bên dưới.
Không tạo plan hoặc bản kiến thức độc lập theo phiên.

Ưu tiên RE động dẫn đường và thử nghiệm tiếp khi có giả thuyết cụ thể; gom các phép đo liên quan
thành một checkpoint tại mốc quan trọng hoặc blocker rõ ràng, không tạo checkpoint/PR cho mỗi phép thử.
Theo [chỉ đạo 21/11](docs/evidence/owner-directives/lean-evidence.json), mỗi PR có bằng chứng runtime/RE mới
chỉ cập nhật **một checkpoint** và dòng liên quan trong Bảng điều hành PLAN. PR chỉ sửa code có test thì không cần
checkpoint. Chỉ cập nhật README, HISTORY, COVERAGE, RESEARCH và `features.json` khi đóng mốc lớn (milestone/đợt)
hoặc khi nội dung của chính tài liệu đó sai.
Giữ trần 120 dòng cho record mới, test tự động là bằng chứng chính cho mã và giữ các giới hạn còn hiệu lực của
[ngân sách bằng chứng 19/11](docs/evidence/owner-directives/evidence-budget.json).
Trước khi hoàn tất checkpoint, cập nhật mục PROTOCOL hiện có nếu contract hoặc giới hạn đổi, lưu metadata và công thức
tái lập trong evidence, giữ raw ở `local/`. Trong `docs_sync`, ghi `unchanged` cho tài liệu không thay đổi.
Số lượt chạy, số frame và diễn biến nằm trong record hoặc run manifest; không lặp sang tài liệu chủ đề.
Tài liệu mới hoặc đoạn sửa viết bằng câu đầy đủ, có dấu cách giữa các từ; không dính chữ để tiết kiệm dòng
(ví dụ `actualmatchinput`, `từngconsumer`). Người đọc là chủ dự án và agent phiên mới, chưa có ngữ cảnh của phiên viết.
Theo [workflow](docs/evidence/checkpoints/README.md),lấy ID/counter tăng liên tục từ
[manifest](docs/evidence/checkpoints/manifest.json),kể cả đổi phiên/compact/worker.
Từ kho v2, dùng `python3 tools/checkpoints.py latest`, rồi `show CPxxx`/`evidence CPxxx`
(summary/page/`--verify`). Không cat toàn manifest/records/raw index vào context; đọc đúng câu hỏi.
Record complete và run/batch evidence manifest đã pin giữ bất biến; [hướng dẫn](docs/evidence/checkpoints/README.md).
Phân biệt source/status/loại kiểm với review/merge/gate;chưa review vẫn ghi pending/candidate.
Không nâng gate từ alive,TCP accept,cửa sổ hoặc encoder tự sinh;không báo phần trăm thiếu bằng chứng.
Theo [chỉ đạo audit theo nhóm tính năng11/11](docs/evidence/owner-directives/feature-group-audits.json),
**không còn audit bắt buộc mỗi4checkpoint** từ sau CP256. Khi hoàn tất một nhóm tính năng, audit thủ công
README→PLAN→PROTOCOL→RESEARCH→evidence/raw và ghi audit/counter bền vững. Checkpoint hoàn tất nhóm khai
`completed_feature_groups`; audit `trigger=feature_group` ghi cùng `feature_groups`/`completed_through`.
PLAN giữ phạm vi/thứ tự nhóm, không tạo lịch checkpoint thay thế. Mâu thuẫn nghiêm trọng vẫn audit ngay.
Audit/record cũ bất biến; checker giữ kiểm nghĩa vụ lịch sử đến CP256, không ép CP260 hoặc chu kỳ mới.


## Làm việc nhóm

Task là issue trong chính repo này, có KRE ID, baseline, owner/reviewer, dependency, file và acceptance.
Một owner mỗi file/target/runtime. Theo chỉ đạo mới nhất28/10, Codex solo RE/code/runtime và tự kiểm;
Claude đã hết token, không gọi worker/reviewer nữa. Chủ dự án nói “claude hết token rồi nên mày tự solo đi ko cần review”;
nguồn quyết định ở [review waiver](docs/evidence/checkpoints/review-waiver.json).
Ngoại lệ giao Terra high ngày14/11 giữ tại [chỉ đạo lịch sử](docs/evidence/owner-directives/l10n-delegation.json).
Theo [chỉ đạo mới giao L1 cho Claude](docs/evidence/owner-directives/l10n-claude-owner.json), Codex đã dừng Terra,
commit WIP trên `research/l10n-utf8` và tiếp G1; không gọi worker hoặc tiếp RE/runtime L1. Claude do chủ dự án
trực tiếp giao sở hữu L1. Không chạy runtime Wine G1 và L1 đồng thời; kiểm lease chung trước mỗi lượt.
Các phần Codex tiếp tục solo, counter/PR và miễn review độc lập không đổi.

Review/report cũ giữ nguyên provenance. Mốc mới ghi review `waived` kèm nguồn chủ dự án, không ghi
`approved` hoặc giả người review. Các vấn đề chất lượng đã phát hiện vẫn cần sửa và ghi chép.
Ca khó checkpoint theo mốc/blocker, không tự quét vô hạn.
Lượt RE mặc định tối đa8 request/4 thân hàm mới, hai lượt; mở tiếp khi có cạnh/thí nghiệm mới rõ.

Code trên nhánh/worktree riêng của repo này; giữ thay đổi người khác,  không reset/clean hoặc stage toàn
repo. Commit task của mình và kiểm diff. Tích hợp main qua PR sau kiểm tra phù hợp; theo miễn review
28/10, Codex được tự merge trong phạm vi đã cấp quyền. Không gọi việc này là review độc lập.
Không push binary/decompile/credential. Không đổi cấu hình cá nhân/IDA/MCP trong repo.
Theo [chỉ đạo chủ dự án 11/11](docs/evidence/owner-directives/private-infrastructure.json), thông tin định danh và cấu hình VPS/OCI (IP/endpoint, OCID,
hostname, tài khoản/đường dẫn truy cập, policy, ACL và topology thực) chỉ giữ trong `local/`.
Không đưa các chi tiết này vào file tracked, commit, PR, issue hoặc comment GitHub; kiểm cả
diff và nội dung bàn giao trước khi gửi. Báo kết quả/nghiệm thu không kèm chi tiết hạ tầng riêng.

## Token discipline

- Chờ CI bằng một lệnh `gh pr checks <số> --watch` cho mỗi lần kiểm head; không poll riêng.
- Dùng `tail`, `jq`, `grep` hoặc CLI có giới hạn khi đọc log và bằng chứng. Không in cả file lớn.
- Không chạy kiểm toàn kho nhiều lần trong một lượt. Sau khi sửa, chỉ chạy lại phép kiểm bị ảnh hưởng.
- Claude cấp ngân sách chạy thật trong brief. Tiếp tục theo brief khi còn ngân sách, không dừng xin thêm.

## Kiểm và bàn giao

CI GitHub Actions (`.github/workflows/docs.yml`, `rust.yml`) chạy trên mọi PR. CI phải đạt trước khi merge;
CI đỏ thì sửa. Giữ phạm vi kiểm CI hiện có; CI không thay kiểm cần `local/`.
Kiểm local: chạy `python3 tools/check_docs.py` và `git diff --check` khi sửa docs/checkpoint. Nếu có checkpoint mới
hoặc đổi, chạy `python3 tools/check_checkpoints.py --changed --require-local --require-history`, từ điểm chung với
`origin/main`. Checker mặc định chỉ in lỗi và tổng kết; dùng `--verbose` khi cần chi tiết cảnh báo.
Checker lưu cache SHA-256 của file host-local trong `local/cache/`. Khi cần kiểm sâu, bỏ `--changed` để kiểm toàn bộ
và dùng `--no-hash-cache` nếu cần tính lại hash. Báo rõ raw thiếu, kiểm còn pending và docs chưa đồng bộ.
Checker không tự xác nhận ngữ nghĩa hoặc đóng gate.
Danh sách file local của checkpoint nằm trong run manifest `local/evidence-manifests/` (record chỉ giữ con trỏ run);
checkpoint mới gọi `checkpoint_evidence.pack_evidence`, tra cứu bằng `python3 tools/evidence_lookup.py`
([cách dùng](docs/evidence/checkpoints/README.md#lean-evidence)).
Dọn `local/` theo [quy tắc lưu giữ](docs/evidence/owner-directives/local-retention.json): chỉ qua kế hoạch dry run của
`tools/local_retention.py`; `--apply` cần hash kế hoạch chủ dự án đã duyệt, không tự xóa raw ngoài kế hoạch.
**Tự dọn sau mỗi lượt chạy** theo [quy định 18/11](docs/evidence/owner-directives/run-cleanup.json). Ai chạy Wine,
máy ảo hay bản sao game thì khi lượt kết thúc (kể cả lỗi) tự xóa bản sao game, prefix, overlay/ảnh đĩa máy ảo, ISO
tạm và dump. Giữ log, capture, ảnh chính và file đang được trỏ tới. Máy ảo dùng một base cộng overlay, không chép
nguyên ảnh đĩa. /home còn dưới 150 GB thì dọn trước khi chạy, dưới 80 GB thì không chạy. Bàn giao ghi dung lượng đã dọn.
Tool/codec/server: chạy kiểm phù hợp hành vi thay đổi; không mặc định phải build client Rust cũ.
Rust server: `cargo fmt --manifest-path rust/Cargo.toml --all --check`, `cargo clippy --manifest-path rust/Cargo.toml
--workspace --all-targets --locked -- -D warnings`, `cargo test --manifest-path rust/Cargo.toml --workspace --locked`;
locked build/binary hash và native acceptance đúng lát cắt trước tuyên bố tương thích. Toolchain/lockfile giữ pin.
CI/build server hiện chỉ Linux. Theo yêu cầu26/10, ưu tiên Wine cho bản copy unpack/patch đã đo;
Theo [chỉ đạo chủ dự án](docs/evidence/owner-directives/wine-accepted.json), Wine được chấp nhận làm môi trường
nghiệm thu thay Windows thật cho project; không yêu cầu WindowsVM/máyWindows hoặc chạy lại các gate
chỉ để có môi trường đó. Ngoại lệ duy nhất: gói cho người chơi ở M8 được thử thêm trên Windows trong máy ảo theo
[quyết định 12/11](docs/evidence/owner-directives/player-environments.json). Ghi đúng Wineversion/hash/copy/patch và nativeevidence để tái lập; không sửa
lịch sử hoặc mô tả sai môi trường run chưa chạy. [LAN giả lập](docs/evidence/owner-directives/emulated-lan.json)
cũng được chủ dự án chấp nhận thay hai máy vật lý; giữ rõ topology/interface/IP/capture/giới hạn.
Theo [chỉ đạo WAN giả lập](docs/evidence/owner-directives/emulated-wan.json), K6c giữ ID và nghiệm thu bằng
WAN giả lập thay Internet thật; các tiêu chí hành vi giữ nguyên. **Internet thật chưa kiểm**; kiểm Internet/ISP/CGNAT
thực tế là backlog riêng, không dependency K6c/K7. Riêng gói người chơi M8 phải chạy qua Internet thật (server trên VPS, client
Wine và Windows máy ảo ở hai mạng, một mạng 4G/CGNAT) theo [quyết định 18/11](docs/evidence/owner-directives/release-real-network.json).
Không suy chỉ đạo môi trường tự đóng gate.
Bàn giao nêu commit/file, nguồn đã đọc, lệnh kiểm+kết quả, evidence và blocker/bước tiếp.
