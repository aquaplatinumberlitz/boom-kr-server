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
    features.json           chỉ mục cơ chế/ID/dependency/trạng thái có nguồn
    COVERAGE.md             bảng/cây/thẻ tra cứu sinh từ catalog
    PROTOCOL.md             contract KR, ma trận reuse VN→KR và authority
    RESEARCH.md             câu hỏi mở, owner và điều kiện đóng
    REFERENCES.md           nguồn RE cũ, provenance và bản đồ migration
    evidence/               metadata/hash; không chứa binary hoặc decompile
  tools/                    harness, probe và kiểm tra artifact/docs
  server/                   code server tương thích KR
  local/                    raw/copy client/DB/capture; git-ignored
```

## Tra nhanh và trạng thái hiện tại

[Coverage và cây chức năng](docs/COVERAGE.md) trả lời: cơ chế nào hiện diện, mức RE/implementation/
runtime nào có nguồn, còn thiếu gì và phép kiểm nào giải quyết. Mỗi ID dẫn thẳng contract/giới hạn,
evidence/hash và câu hỏi; [PLAN](docs/PLAN.md#current-order) giữ duy nhất thứ tự công việc.
Đây là scope đường bootstrap → login → sảnh → phòng → Battle → kết quả/reset, không kiểm kê toàn client.

CP050: service đã xử lý host/guest EOF khi bom còn chờ: loại timer cũ, forfeit/về phòng,
xóa đúng slot và chuyển/giữ chủ bằng getter native. Ca giữ99 củaB10s cho thấy service chờ đủ hai
peer rồi mới phát phase; client tiếp tục input/bom/result vòng1. [Evidence/giới hạn](docs/evidence/service-eof/README.md),
[contract](docs/PROTOCOL.md#service-eof). Chưa client mới vào/Start từ B trong service hoặc account reconnect;
fullvalidator/auth/cache/codec/reset và gate K5/K6a/LAN/Internet vẫn mở.

CP049 đã merge [PR59](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/59) tại `a6b62cd`.
CP050 tự kiểm/integration theo [PR61](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/61); review độc lập
[waived](docs/evidence/checkpoints/review-waiver.json). Audit **AUD015/CP048**, audit định kỳ kế **CP052**;
[counter](docs/evidence/checkpoints/manifest.json), [workflow](docs/evidence/checkpoints/README.md).

Đích đầu: hai client thật chơi hai trận liên tiếp; mở bốn client và hardening sau đó.

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
