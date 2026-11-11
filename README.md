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

**CP271: sửa kỳ vọng test tranh chấp SQLite khi bàn giao R1g.**
CI của PR348 đã phát hiện test chỉ chấp nhận `ActiveSession`, trong khi backend có đường trả `Busy` hữu hạn.
Test giữ yêu cầu chỉ một grant, chấp nhận đúng hai lỗi đó, kiểm lại `ActiveSession` sau tranh chấp và ép khóa SQLite
để kiểm `Busy`/phục hồi. Không đổi runtime; [correction và bằng chứng](docs/evidence/rust-service/r1g-ci-correction.json).

R1g CP270 đã kiểm trần admission và allowlist độc lập số profile.
[Contract/giới hạn](docs/PROTOCOL.md#rust-r1g-admission), [evidence](docs/evidence/rust-service/r1g-admission.json).
Cấu hình cho phép1–1024transport và1–256entry allowlist; vẫn kiểm source/lease/profile riêng. Giá trị cấu hình cũ giữ nguyên.
105Rust/fmt/clippy/lockedbuild đạt;306TCPlegacy/3169+24frame khớp CP269;10ca admission đạt, gồm65socket giữ đồng thời,
kết nối66 bị chặn và phục hồi permit. Đây chưa phải kiểm50người chơi KR hoặc native candidate.
R1f đã merge PR347; R1g đang bàn giao PR. Tiếp relay opt-in và native cuối R1 theo [dashboard](docs/PLAN.md#dashboard).
Full M5/M7/M8 còn mở; hạ tầng chỉ local. Mốc cũ ở [HISTORY](docs/HISTORY.md).

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
