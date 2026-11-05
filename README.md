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

**Mốc mới CP132:** trong shared cascade đã đo, flag thùng(2,3) còn1 tại process-entry root2,
bridge4 và secondary3 trên cả hai client; tới root158 return mới thấy0. Ô(2,2) giữ1.
[Contract/giới hạn](docs/PROTOCOL.md#shared-flag-timing), [raw/hash/recipe](docs/evidence/three-bomb-chain/README.md#cp132).
Đây là mốc quan sát, chưa xác định chính xác instruction/pha xóa flag hoặc chứng minh globalimmutable/FIFO.

CP131 đã merge [PR202](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/202) tại `9e9824a`.
CP132 docs/evidence/tự kiểm trong thay đổi hiện tại; review độc lập
[waived](docs/evidence/checkpoints/review-waiver.json). #1/#2/#3/#9/#14 và K0–K7 vẫn mở;
ca mới chỉ một Battle cascade/noresult, không fullmatch/K6a/LANInternet. Raw/complete history giữ nguyên.

Audit thủ công **AUD041 qua CP132**, kế **CP136**, đã đối chiếu canonical docs với raw/source và sửa
các pending-merge/gap stale. Bước kế chỉ [PLAN](docs/PLAN.md#current-order): nối query native của đúng
thùng chung với snapshot vừa đo; không lặp before/after hoặc quét toàn chain/client.
Theo [yêu cầu chủ dự án](docs/evidence/owner-directives/continuous-work.json), tiếp tục việc kế sau PR.

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
