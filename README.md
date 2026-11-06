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
  server/                   code server tương thích KR
  local/                    raw/copy client/DB/capture; git-ignored
```

## Tra nhanh và trạng thái hiện tại

[Coverage và cây chức năng](docs/COVERAGE.md) trả lời: cơ chế nào hiện diện, mức RE/implementation/
runtime nào có nguồn, còn thiếu gì và phép kiểm nào giải quyết. Mỗi ID dẫn thẳng contract/giới hạn,
evidence/hash và câu hỏi; [PLAN](docs/PLAN.md#current-order) giữ duy nhất thứ tự công việc.
Đây là scope đường bootstrap → login → sảnh → phòng → Battle → kết quả/reset, không kiểm kê toàn client.
Tra checkpoint: `python3 tools/checkpoints.py latest`, `show CPxxx`, `evidence CPxxx`
(summary/page hoặc `--verify --require-local`); [kho v2/migration](docs/evidence/checkpoints/README.md#kho-checkpoint-v2-và-cách-đọc-chọn-lọc)
giữ record cũ nguyên object. Không cần đọc manifest lớn.

Hai copy KR Wine đã chạy login/sảnh/phòng/Battle và các chu kỳ kết quả/reset có bằng chứng;
**chưa nghiệm thu trận đầy đủ/K6a, LAN hoặc Internet**. [Movement/terrain hai vòng](docs/PROTOCOL.md#current-movement-cycles),
[kim/reset](docs/PROTOCOL.md#needle-round-reset), [cứu hai chiều và hai kết quả](docs/PROTOCOL.md#rescue-two-results),
[deep chain/reset](docs/PROTOCOL.md#recursive-chain-cycles) giữ phạm vi và giới hạn tại nguồn chuẩn.

**Mốc mới CP177:** preset tùy chọn `two-client-basic-replacement` đã kiểm trên cả hai slot trống:
C identity3 lấy capacity từ vai đã rời; slot0 initial1/max10, slot1 initial2/max7. Hai run riêng có
Join/Ready/host Start, native initial stats, actual C118 và cùng159/158 trên hai client hiện hành.
Maximum là policy; không suy đã kiểm capacity exhaustion. [Contract/giới hạn](docs/PROTOCOL.md#replacement-profile),
[run/hash và ca startup lỗi giữ nguyên](docs/evidence/peer-endpoints/README.md#cp177).
CP176 hai vòng A/C bình thường vẫn có [nguồn riêng](docs/PROTOCOL.md#a-c-basic-cycles), đã merge
[PR248](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/248) tại `bd34ed4`.

Đây là copy KR Wine/cùng physical host/mạng thử riêng; Windows nguyên bản, LAN/Internet/tám client
và K0–K7/K5/K6a/fullmatch chưa nghiệm thu. [PLAN](docs/PLAN.md#current-order) tiếp đối chiếu acceptance
bằng bằng chứng cũ và chỉ đo phần chặn thực tế; không lặp hai trận hoặc mọi biến thể thay người.
CP177 complete/PR pending trước tích hợp; trạng thái merge tại PR/Git. AUD052 qua CP176, audit kế
**CP180**. [Review waived](docs/evidence/checkpoints/review-waiver.json).

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
