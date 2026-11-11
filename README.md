# Boom KR Server

Server giả lập cho **the lastest client Crazy Arcade KR build00967 before end of service**, dựa trên kiến thức từ client
VN VBNB_116. 
Chú ý:
1- dự án không cung cấp client game
2- dự án chứa lượng lớn text và kiến thức RE được viết bằng agents, ko phải con người, nếu đọc không hiểu thì hãy hỏi AI giải thích. 
<img width="1024" height="1230" alt="fixed-contact" src="https://github.com/user-attachments/assets/739b9237-b44a-47f0-87a3-97ccd3c9f669" />

**Thư mục làm việc hiện tại trên host:** `/home/admin/boom/repos/boom-kr-server`.
Docs, tools, code và evidence mới đều ở đây. Không cần mở worktree của repo Rust cũ để làm project này.

```text
boom-kr-server/
  AGENTS.md                 quy tắc của project này
  docs/
    PLAN.md                 lộ trình Rust, acceptance và thứ tự hiện hành
    features.json           chỉ mục cơ chế/ID/dependency/trạng thái có nguồn
    COVERAGE.md             bảng/cây/thẻ tra cứu sinh từ catalog
    PROTOCOL.md             mục lục contract; chi tiết theo chủ đề trong protocol/
    RESEARCH.md             câu hỏi mở, owner và điều kiện đóng
    REFERENCES.md           nguồn RE cũ, provenance và bản đồ migration
    evidence/               metadata/hash; không chứa binary hoặc decompile
  tools/                    harness, probe và kiểm tra artifact/docs
  rust/                     kr-wire + kr-service; lát cắt Rust từng bước
  server/                   Python serving/RE/reference vẫn giữ
  local/                    raw/copy client/DB/capture; git-ignored
```

**Đích bàn giao giai đoạn 1 (chốt 11/11):** dịch vụ nội bộ bền vững, người chơi qua Internet thật, khoảng 50 người đồng
thời, chế độ thường với nhiều map và item phổ biến. Dài hạn vẫn hướng tới gameplay đầy đủ.
[Đích, việc còn lại và thứ tự](docs/PLAN.md#delivery-target).

## Tra nhanh và trạng thái hiện tại

**CP272: contract đã chia theo chủ đề; R1g đã merge PR348.**
Tra packet/cơ chế tại [mục lục contract](docs/PROTOCOL.md), câu hỏi còn mở tại [RESEARCH](docs/RESEARCH.md).
PLAN giữ thứ tự công việc; diễn biến checkpoint chuyển sang [HISTORY](docs/HISTORY.md).
Full R1 còn native hồi quy 2/4/8/account; N1R còn integration và native relay qua hai NAT.
M5 còn hồ sơ tổng quát/native DB-queue fault; M7/G1/M6/M8 theo [Bảng điều hành](docs/PLAN.md#dashboard).
Không đổi runtime hoặc phạm vi nghiệm thu trong đợt tài liệu này. Hạ tầng thực chỉ lưu local.

## Bắt đầu

Đọc [AGENTS](AGENTS.md), [PLAN](docs/PLAN.md), rồi [PROTOCOL](docs/PROTOCOL.md) và
[RESEARCH](docs/RESEARCH.md) theo task.

```bash
python3 tools/feature_catalog.py
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
