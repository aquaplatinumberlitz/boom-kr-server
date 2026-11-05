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

CP110 xác nhận **sau hai vòng Battle trên copy Wine, cả A/B hiện lại phòng và A Ready được**:
ảnh đầu đen, tới mốc chờ4giây đã vẽ phòng trước thao tác focus mới; native110 đủ6/6/lỗi0,
A phát95/seq868 và cảhai đọc110 sau đó. [Contract/giới hạn](docs/PROTOCOL.md#return2-visual),
[evidence](docs/evidence/promoted-outcome/README.md). Chưa thử Start trận3, fullmatch/K6a/LAN/Internet.
[CP111](docs/PROTOCOL.md#receiver-prepost) giải gapreceiver từrawCP110: verifier cũ chỉthốngkê
beforeXY, callbackcuối đãđổi159→219 rồifreshsample294. Proof4phaseprepost mới giữscopebranch4snap/0deferred;
failurecũgiữnguyên, không thêmruntime hoặc nânggate. Bước kế firstseen/current-generation guards theoPLAN.

[Đối chiếu issue](docs/evidence/issue-reconciliation/README.md): #178 đã giao và đóng sau
[PR179](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/179), merge `8f3ea2f`;
#1/#2/#3/#9/#14 còn mở. Snapshot có UTC; lịch sử checkpoint không sửa.
CP110 [PR180](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/180) đãmerge `7c5d4ea`; CP111 tự kiểm/chờPR, review [waived](docs/evidence/checkpoints/review-waiver.json).
Audit gần nhất **AUD034/CP109**, periodic kế **CP112**; K0–K7 vẫn mở.
[Yêu cầu chủ dự án](docs/evidence/owner-directives/continuous-work.json): tiếp tục ngay việc kế trong
PLAN sau mỗi checkpoint/commit/PR, không tự dừng/chờ nhắc.

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
