# Boom KR Server

Server giả lập cho **client Crazy Arcade KR gốc build 00967**, dựa trên RE KR và kiến thức từ client
VN VBNB_116. Đây là project độc lập với bản viết lại client/server Rust `boom-rs`.

**Thư mục làm việc hiện tại trên host:** `/home/admin/boom/repos/boom-kr-server`.
Docs, tools, code và evidence mới đều ở đây. Không cần mở worktree của repo Rust cũ để làm project này.

```text
boom-kr-server/
  AGENTS.md                 quy tắc của project này
  docs/
    PLAN.md                 lộ trình K0–K7, task và gate
    PROTOCOL.md             contract KR, ma trận reuse VN→KR và authority
    RESEARCH.md             câu hỏi mở, owner và điều kiện đóng
    REFERENCES.md           nguồn RE cũ, provenance và bản đồ migration
    evidence/               metadata/hash; không chứa binary hoặc decompile
  tools/                    harness, probe và kiểm tra artifact/docs
  server/                   code server tương thích KR
  local/                    raw/copy client/DB/capture; git-ignored
```

## Đang làm

CP014: bản copy Wine **đã gửi request auth opcode1 (80 byte)** tới stub local,
tiếp theo opcode60; capture đã kiểm bằng parser/CRC native KR. Token local giúp luồng tự động
đặt playerCount1; đối chứng cùng copy thiếu token vẫn count0 và không gửi auth.
Phản hồi lỗi đã vào handler đúng mã1; phép thử result0-prefix + network-info chạy state chờ
**0→2→1→3**, modal trả3 và attempt trả1, rồi dừng ở native helper chưa phục hồi `16EA8FC`.
**Chưa xác nhận login hợp lệ, chưa vào sảnh**: result0 vẫn cố ý thiếu body đầy đủ.

Các phép thử dùng copy có patch và reset global trong bộ nhớ; không tương đương client nguyên bản.
Contract/giới hạn ở [PROTOCOL](docs/PROTOCOL.md#login-auth), recipe/raw/hash tại
[evidence login](docs/evidence/login-runtime/README.md); bước tiếp ở [Q-023](docs/RESEARCH.md#q-023).
CP012/013 đã merge PR20/21; CP014 đang chờ tích hợp khi lập checkpoint. PR là nguồn trạng thái
merge hiện tại. Chủ dự án miễn review độc lập; Codex tự kiểm và giữ gate bằng chứng.

Checkpoint bền vững: [manifest](docs/evidence/checkpoints/manifest.json).
Quy trình ghi chép/review/audit: [checkpoint workflow](docs/evidence/checkpoints/README.md).

Đích đầu: bootstrap offline → login cục bộ → sảnh/phòng → Battle → kết quả → hai client chơi hai
trận liên tiếp. Bốn client và hardening sau đó. Chi tiết ở [PLAN](docs/PLAN.md).

## Bắt đầu

Đọc [AGENTS](AGENTS.md), [PLAN](docs/PLAN.md), rồi [PROTOCOL](docs/PROTOCOL.md) và
[RESEARCH](docs/RESEARCH.md) theo task.

```bash
python3 tools/check_docs.py
python3 tools/check_checkpoints.py --require-local --require-history
python3 tools/artifacts.py verify
```

`artifacts.py verify` dùng manifest **host-local** trong `docs/evidence/artifacts.json`; client/DB
không nằm trong Git nên clone sạch trên máy khác chưa thể chạy lệnh này. Khi chuyển máy, dùng
`--manifest /path/to/local-artifacts.json` với đường dẫn nguồn và hash đã xác nhận tại máy đó.

Client nguồn giữ ở `/home/admin/boom/clients/kr-00967`; không sửa hoặc commit. Artifact và database
đã unpack ở repo RE cũ chỉ là đầu vào tham chiếu; mỗi phép thử dùng bản copy riêng trong `local/`.
Nguồn/commit và giới hạn bằng chứng: [REFERENCES](docs/REFERENCES.md).
