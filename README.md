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

[CP115](docs/PROTOCOL.md#rescue-service) đã tích hợp và đo **cứu đồng đội** trên service opt-in:
report18 thật, peer170/18 đủ7byte/lỗi0, native hai phía56→26 và di chuyển sau cứu trên hai view.
Scope hai copy Wine cùng đội + một actor đối địch tổng hợp; [CP116](docs/PROTOCOL.md#rescue-reverse) thêm B được A cứu/aux0, có control khôngforward và positive native hai phía. [CP117](docs/PROTOCOL.md#rescue-return-start-guard) có result/return/ReadyB sau cứu; [CP119](docs/PROTOCOL.md#rescue-slot-refresh) đã mở Start2 bằng diagnostic slot2 refresh; [CP120](docs/PROTOCOL.md#rescue-round2-feedback) đãđo freshR2rescue/nativeboth56→26; controlCP119no18 giữnguyên. [CP121](docs/PROTOCOL.md#rescue-two-results) đã hoàn tất hai chu kỳ cứu → kết quả được chọn → về phòng, B Ready lần hai; chưa nghiệm thu fullmatch/K6a.
[Nguồn/hash/giới hạn](docs/evidence/rescue-service/README.md); bước kế theo PLAN, gate giữ mở.

CP110 xác nhận **sau hai vòng Battle trên copy Wine, cả A/B hiện lại phòng và A Ready được**:
ảnh đầu đen, tới mốc chờ4giây đã vẽ phòng trước thao tác focus mới; native110 đủ6/6/lỗi0,
A phát95/seq868 và cảhai đọc110 sau đó. [Contract/giới hạn](docs/PROTOCOL.md#return2-visual),
[evidence](docs/evidence/promoted-outcome/README.md). Chưa thử Start trận3, fullmatch/K6a/LAN/Internet.
[CP111](docs/PROTOCOL.md#receiver-prepost) giải gapreceiver từrawCP110: verifier cũ chỉthốngkê
beforeXY, callbackcuối đãđổi159→219 rồifreshsample294. Proof4phaseprepost mới giữscopebranch4snap/0deferred;
failurecũgiữnguyên, không thêmruntime hoặc nânggate. [CP122](docs/PROTOCOL.md#recursive-chain) đã đo chain sâu ngoài tia root, all195flags khớp model trong ca hẹp; [CP123](docs/PROTOCOL.md#recursive-chain-cycles) ghép chain sâu với survivor/result/reset hai epoch; survivor currentterrain đã có từ CP106, không mở lại; [CP124](docs/PROTOCOL.md#death-lifecycle-policy) có corpus64report/nativeframe và guard19 opt-in đã đo hai rescue/result/return epochs; bước kế normal Battle exit theo PLAN; CP115–116 đã đo hai chiều A/aux1 và B/aux0.

[Đối chiếu issue](docs/evidence/issue-reconciliation/README.md): #178 đã giao và đóng sau
[PR179](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/179), merge `8f3ea2f`;
#1/#2/#3/#9/#14 còn mở. Snapshot có UTC; lịch sử checkpoint không sửa.
CP110 [PR180](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/180) đãmerge `7c5d4ea`; CP111 [PR181](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/181) đãmerge `248e118`; CP112 [PR182](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/182) đãmerge `1b7f210`; CP113 [PR183](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/183) đãmerge `2695042`; CP114 [PR184](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/184) đãmerge `989dd3a`; CP115 [PR185](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/185) đãmerge `80e4d48`; CP116 [PR186](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/186) đãmerge `f33f3b8`; CP117 [PR187](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/187) đãmerge `c2ca94d`; CP118 [PR189](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/189) đãmerge `dae23b7`; CP119 [PR190](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/190) đãmerge `bfd526b`; CP120 [PR191](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/191) đã merge `18b73f1`; CP121 [PR192](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/192) đã merge `c24687a`; CP122 [PR193](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/193) đã merge `1cc76af`; CP123 [PR194](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/194) đã merge `06765c7`; CP124 tự kiểm/chờ PR, review [waived](docs/evidence/checkpoints/review-waiver.json).
CP112 thêm [guard envelope17/19](docs/PROTOCOL.md#death-envelope-admission) trước side effect;
161tests và runhai-vòngBhost438,2s/actualA95seq823/native110 đãqua. Validfirstseen semantic vẫnthiếu.
CP113 thêm [kim145 opt-in](docs/PROTOCOL.md#needle-service): actualpickup/17→1→0, một170/1,
A/B native56→26, ôkimtrống vàmovement sauđó. 175tests qua; CP113run chỉmộtescape.
[CP114](docs/PROTOCOL.md#needle-round-reset) thêmcurrentsecondtrapcạn→actualdeath/result/return→Start2/
fresh145/escape, giữcùngsource/profile. R2chưa kếtthúc; CP120 đã có result/Start2freshrescue; fullinventory/gate vẫn mở.
Audit định kỳ **AUD039/CP124**, mốc kế **CP128**; AUD038 giữ correction CP106; K0–K7 vẫn mở.
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
