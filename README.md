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

CP043: **159 cũ có thể bind nhầm ID cho bom dự đoán vòng mới khi serial local dùng lại**,
khiến timer ID hiện tại nổ ở B nhưng không ở A. Control và ca fence loại job159 cũ đều đủ hai vòng.
[Contract/phạm vi](docs/PROTOCOL.md#late-bomb-confirm), [recipe/control/hash](docs/evidence/late-bomb-confirm/README.md).
CP041–042 đã có [fence job bom](docs/PROTOCOL.md#round-generation) và [claim item](docs/PROTOCOL.md#item-generation).

Đây là thí nghiệm copy chẩn đoán Wine; **chưa nghiệm thu trận hoàn chỉnh/K6a**.
Timer abort, peerqueue, fullvalidator và full auth/cache/codec còn thiếu; chưa LAN/Internet.
[Bước tiếp](docs/RESEARCH.md#q-023).

CP042 đã merge [PR50](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/50) tại `ed47c97`.
CP043 là snapshot trước tích hợp trên nhánh `kre/late-bomb-confirmation`.
Codex tự kiểm, review độc lập waived. Audit gần nhất **AUD013/CP040**, audit định kỳ kế **CP044**.

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
