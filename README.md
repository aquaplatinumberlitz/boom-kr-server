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

Recipe Wine R2 đã review/merge, qua ProcessPatch và đạt cửa sổ800×600 + kết nối local thứ hai.
Probe trước CP006 được debugger xác nhận nhận phản hồi WCP; run đó zero C→S và cuối run báo
MW_SEP(10)/Login Failed. **Chưa có login hoạt động hoặc gameplay.**
Probe hai-reply vẫn là thí nghiệm riêng; review tài liệu theo [CP001](docs/evidence/checkpoints/manifest.json), tách khỏi R2 đã merge.
Phạm vi từng recipe/bằng chứng ở [WCP](docs/evidence/wcp/README.md);
CP004–CP005 ghi error state 1/1958 từ dump trên bản R2 và nhánh launcher trong run bản
copy thử; [trạng thái review](docs/evidence/checkpoints/manifest.json). Điểm chặn và phép kiểm kế ở [Q-023](docs/RESEARCH.md#q-023).

CP006 đã review và merge qua PR16: client copy gửi request đầu và nhận chuỗi response thử.
CP007 thay identity bằng giá trị local thử, tiếp tục qua một số import/call khởi tạo và dừng
ở assertion audio của Wine, kể cả phép thử ALSA null. Các giá trị và patch chỉ là chẩn đoán;
chi tiết, đối chứng và trạng thái review ở [evidence CP007](docs/evidence/nmco-identity/README.md).

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
