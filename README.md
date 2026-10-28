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

CP015: **copy Wine đã hiển thị sảnh KR và đóng hộp “đang kết nối kênh” qua handler nhận của client**.
Sau request auth đã đo ở CP014, client đi qua request5 → redirect local → socket kênh/request6;
S→C17 vào `CLobbyStage`, S→C241 đặt ready và đóng notice. Không ép modal result hoặc cờ ready.

Đây là mốc UI/đường nhận trên **copy chẩn đoán**, chưa phải login/profile/cache đầy đủ hoặc gate K3.
Body result0 còn thiếu; tên/profile trống. Hai lần click Tạo phòng chưa có request77, chưa xác nhận
hit control; chưa tạo phòng hoặc vào trận. Contract/giới hạn tại
[PROTOCOL](docs/PROTOCOL.md#channel-lobby), recipe/ảnh/hash tại
[evidence sảnh](docs/evidence/postwait/README.md); bước tiếp ở [Q-023](docs/RESEARCH.md#q-023).

CP014 đã merge [PR22](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/22).
CP015 là checkpoint mới chờ tích hợp khi ghi record; PR là nguồn trạng thái merge hiện tại.
Chủ dự án miễn review độc lập; Codex tự kiểm, giữ gate bằng chứng. Audit định kỳ kế CP016.

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
