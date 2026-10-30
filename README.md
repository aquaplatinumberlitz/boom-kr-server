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

CP023: **copy Wine đã chạy hai vòng thử trên cùng kết nối: phòng → trận → bom/HELP → bảng kết quả → phòng**.
Client hiện **LOSE KRLocal / PERFECT KRGuest** theo rank thử; vòng thứ hai bắt đầu bằng click thật,
phát104, đọc121 đủ byte và khởi tạo scene mới. [Contract](docs/PROTOCOL.md#round-result),
[evidence](docs/evidence/round-repeat/README.md).

Đây là **một client với guest tổng hợp**, trên bản copy chẩn đoán. Stub chọn outcome từ báo cáo19;
chưa trận hoàn chỉnh được nghiệm thu, hai client hoặc gate K2–K6. Điểm chặn tiếp là đồng bộ tọa độ/
spawn/serial/timer và peer; full auth/profile/cache còn thiếu. Thứ tự ở [RESEARCH](docs/RESEARCH.md).

CP022 đã merge [PR30](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/30).
CP023 là snapshot trước tích hợp; PR nhánh `kre/round-repeat-runtime` giữ trạng thái merge hiện tại.
Codex tự kiểm, review độc lập waived. AUD006 ở CP020; audit kế **CP024**.

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
