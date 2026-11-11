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
    PLAN.md                 lộ trình K0–K7, task và gate
    features.json           chỉ mục cơ chế/ID/dependency/trạng thái có nguồn
    COVERAGE.md             bảng/cây/thẻ tra cứu sinh từ catalog
    PROTOCOL.md             contract KR, ma trận reuse VN→KR và authority
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

**CP264: R1d header chuẩn và builder theo opcode đã kiểm; N1 được ghi nhận đạt phạm vi đã chốt.**
[Contract R1d](docs/PROTOCOL.md#rust-r1d-wire-builders), [bằng chứng](docs/evidence/rust-service/r1d-wire-builders.json).
96 test Rust/fmt/clippy/locked build;306 ca TCP đạt,3169 frame phòng cũ và24 frame first-vacant khớp baseline CP263.
Alias JSON chỉ còn adapter nạp cấu hình; runtime dùng template có kiểu/opcode và header chuẩn. Chưa native binary mới.
[N1](docs/PROTOCOL.md#rust-n1-acceptance) đạt **ít nhất một phía tới được theo địa chỉ server báo**, dựa CP261 và xác nhận4G
của chủ dự án. N1b hai phía NAT đổi cổng còn mở; shared-NAT gameplay và full M5 vẫn chưa đạt.
R1c đã merge [PR338](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/338). Audit nhóm AUD081; R1d đang bàn giao PR.
Việc kế tiếp theo [Bảng điều hành](docs/PLAN.md#dashboard): R1e → N1b → R1f/R1g → native cuối R1.
Mốc cũ ở [HISTORY](docs/HISTORY.md). Thông tin hạ tầng giữ local.

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
