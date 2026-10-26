# Boom KR Server

Server giả lập cho **client Crazy Arcade KR gốc build 00967**, dựa trên RE KR và kiến thức từ client
VN VBNB_116. Đây là project độc lập với bản viết lại client/server Rust `boom-rs`.

**Thư mục làm việc duy nhất:** `/home/admin/boom/repos/boom-kr-server`.
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

K0 đang khóa artifact và chuẩn bị harness Windows VM; dsh1 nghiên cứu bootstrap/ProcessPatch,
dsh2 nghiên cứu codec từ writer/reader KR. Chưa có hello KR hoặc demo chơi được; không dùng tiến độ
36/37 của `boom-rs` để báo tiến độ project này.

Đích đầu: bootstrap offline → login cục bộ → sảnh/phòng → Battle → kết quả → hai client chơi hai
trận liên tiếp. Bốn client và hardening sau đó. Chi tiết ở [PLAN](docs/PLAN.md).

## Bắt đầu

Đọc [AGENTS](AGENTS.md), [PLAN](docs/PLAN.md), rồi [PROTOCOL](docs/PROTOCOL.md) và
[RESEARCH](docs/RESEARCH.md) theo task.

```bash
python3 tools/check_docs.py
python3 tools/artifacts.py verify
```

Client nguồn giữ ở `/home/admin/boom/clients/kr-00967`; không sửa hoặc commit. Artifact và database
đã unpack ở repo RE cũ chỉ là đầu vào tham chiếu; mỗi worker dùng bản copy riêng trong `local/`.
Nguồn/commit và giới hạn bằng chứng: [REFERENCES](docs/REFERENCES.md).
