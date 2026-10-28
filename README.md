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

CP006–011 đã tích hợp qua PR16–19: bản copy Wine nhận dữ liệu thử và vẽ UI đăng nhập.
CP012 đã theo input tới command xác nhận1002 của CLoginStage và tìm hostname login riêng.
Thêm ánh xạ local cho hostname này giúp luồng tự động trước click mở kết nối thứ ba;
fixture hello đã qua poll/consumer WCP trên kết nối đó. Sau phép reset global trong bộ nhớ,
**đã thu4 frame C→S type15 (276 byte mới)**, kiểm parser/CRC native với seq40,43,46,49.
**Chưa xác nhận request auth hoặc login thành công.**
Đã đo global từ dump làm `CWaitForLoginDlg` đóng với result1; thử reset riêng global trong
bộ nhớ giữ được dialog đếm chờ. State runtime vẫn0. CP013 đối chiếu VN đã giải tĩnh dispatcher KR cho hai phản hồi S→C4/15
nối chuỗi state chờ; request auth và payload đầy đủ vẫn chưa phục hồi.

Recipe/giới hạn tại [evidence handler](docs/evidence/ui-handler/README.md);
nguồn tham chiếu VN ở [VN→KR](docs/evidence/vn-reuse/README.md).
Bước tiếp duy nhất ở [Q-023](docs/RESEARCH.md#q-023). CP012 đã merge PR20; CP013 là kết quả tĩnh đang chờ tích hợp khi lập checkpoint;
PR là nguồn trạng thái merge hiện tại. Chủ dự án miễn review độc lập; Codex tự kiểm và giữ gate bằng chứng.

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
