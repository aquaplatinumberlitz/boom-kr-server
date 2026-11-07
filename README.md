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

**K0 và bước tĩnh K2a đạt CP178; K1 đạt riêng recipe Wine R2 offline tại CP179**, đã merge
[PR250](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/250)/`2f2b87f` và
[PR251](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/251)/`2e6dae4`.
[Phạm vi nghiệm thu](docs/PLAN.md#k1-acceptance) không bao gồm bootstrap Windows nguyên bản hoặc ba coldstart e9.

**CP181: K2 đạt cho profile codec/hello offline đã đo**, sau đối chiếu thủ công từng tiêu chí
giữ nguyên. Native retry trong cùng process: result1 hợp lệ → command1002 → login socket mới →
auth137/137/errors0 → channel socket mới; hello/mode/seq reset được đo trên từng connection.
[Nghiệm thu/giới hạn](docs/PLAN.md#k2-acceptance), [contract](docs/PROTOCOL.md#login-retry-codec),
[run/hash](docs/evidence/live-codec-stream/README.md#cp181). CP180 split/coalesced/CRCcontrol đã merge
[PR252](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/252) tại`fe242cc`.

**K3–K7, K6a/trận đầy đủ/LAN/Internet vẫn mở.** Fullgrammar/unknownheader/prefix/mode và
accountauthority chưa được khôi phục đầy đủ; K2 không nghiệm thu các phần đó. Thứ tự tiếp ở
[PLAN](docs/PLAN.md#current-order): identity/cache/UI sảnh theo evidence cũ, không lặp knowncodec/gameplay.
CP181 đã merge [PR253](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/253) tại `5a154e5`.
**CP184: identity291 đã tích hợp opt-in vào service.** Hai run A đã hiện tên ở sảnh rồi tạo phòng,
nhận roster/map; native framer nhận sequence43…73 liên tục, next76. [Contract/giới hạn](docs/PROTOCOL.md#own-profile-integration),
[run/hash](docs/evidence/lobby-identity/README.md#cp184). Đây là một client thật với guest tổng hợp,
không thay bằng chứng hai client hoặc nghiệm thu trận. CP183 đã merge [PR255](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/255)
tại`5c1f3fb`; CP184 chờ PR. **K3 vẫn mở**, phần thiếu cụ thể ở [checklist nghiệm thu](docs/PLAN.md#k3-evidence-review).

Ưu tiên khép K3 → rà K4 → K5/K6a; chỉ mở RE mới khi phục vụ tiêu chí chưa đạt của gate hiện hành.
AUD055 periodic qua184, audit kế **CP188**. [Review waived](docs/evidence/checkpoints/review-waiver.json).

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
