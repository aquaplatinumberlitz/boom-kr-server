# Tool của server KR

`check_docs.py`: kiểm file đích của link Markdown nội bộ, không gọi network; **không kiểm fragment/anchor**.
`artifacts.py verify`: kiểm SHA/size đầu vào RE theo manifest **host-local**; chỉ đọc source.
Máy khác dùng `--manifest /path/to/local-artifacts.json`; tool không tải hoặc tự tạo binary nguồn.

`verify_primitives.py`: chạy byte helper KR gốc trong Unicorn, kiểm output, guard byte và bounded
return. Chỉ nhận image unpack có SHA đã khóa; không chạy client/bootstrap hoặc network. Cần dependency
RE tùy chọn trong `requirements-re.txt`; kết quả mới phải có output path riêng, không ghi đè raw cũ.

```bash
python3 tools/verify_primitives.py \
  --binary local/research/dsh2/kr-full.exe \
  --out local/research/codex/primitives-run2.json
```

Phạm vi bằng chứng: [writer primitive](../docs/evidence/primitives/README.md). Tool không xác nhận
header/checksum/crypto/seq hoặc mode selection của packet/socket đang chạy.

`build_wine_copy.py`: dựng đúng recipe KR copy đã đo, khóa SHA3 đầu vào và output, ghi before/after
từng patch vào report local. --recipe bootstrap-r1 là mặc định, SHA output giữ nguyên;
--recipe post-bootstrap-r2 là profile riêng đi tới cửa sổ/kết nối thứ hai theo
[evidence R2](../docs/evidence/wine-r2/README.md). Cần `pefile==2024.8.26`; candidate metadata là artifact host-local.
`wine_probe.py`: chạy bounded trong namespace riêng, fresh CALog/window/screenshot/Winsock và accept
TCP loopback mặc định silent; `--bootstrap-reply` gửi fixture local một lần trên connection đầu tiên. Runtime paths chỉ nhận copy dưới `local/`; xem
[recipe Wine](../docs/evidence/wine/README.md) để biết lệnh chạy và giới hạn.

Harness/probe/bootstrap và codec mới đặt tại đây hoặc `server/`; không ghi vào tool repo VN/KR cũ.
Raw output, bản copy client/DB và capture đặt trong `local/`, không commit.

`verify_bootstrap.py`: native length/copy/receive/reader với modO SHA khóa, allocator tổng hợp,
254 ca; không thực thi full ProcessPatch hoặc xác nhận hello game. Lệnh và giới hạn ở
[bootstrap wire](../docs/evidence/bootstrap-wire/README.md).

check_checkpoints.py: kiểm manifest/counter append-only theo Git,required fields,docs-sync links,
evidence hash,closed-task/active-next-step và cadence audit;dùng lại check_docs.py.
Trên host chạy --require-local --require-history;clone thiếu raw báo UNVERIFIED khi không flag.
Pending review hiện WARNING,không thành semantic approval. [Quy trình/giới hạn](../docs/evidence/checkpoints/README.md).
Tests: python3 tools/test_checkpoints.py.

`feature_catalog.py --generate`: sinh [coverage/cây/thẻ cơ chế](../docs/COVERAGE.md) từ
[catalog ID ổn định](../docs/features.json). Không flag: kiểm bản sinh đã đồng bộ, schema/trạng thái,
file/anchor, dependency/parent DAG và ID/key so HEAD. RESEARCH giữ nguồn câu hỏi/phép đo để trích;
PLAN giữ thứ tự. Check này cũng chạy trong `check_docs.py`; kiểm Markdown thường vẫn không kiểm fragment.
External URL không fetch. Tool không suy nghĩa/kết luận, xác nhận runtime hoặc tự đóng gate.
Negative tests: `python3 tools/test_feature_catalog.py`.

`export_probe_profile.py --source-root local/research/codex --out <local/profile.json>`: chỉ lập index path/size/SHA/sequence của
fixture RE hiện hữu cho [service chẩn đoán](../server/README.md); không nhập binary/asset vào Git.
Profile dùng chính xác bytes/seq đã đo, không là schema auth/cache tổng quát hoặc oracle tự sinh.

CP048: checkpoint checker phân biệt docs hiện hành với snapshot cũ. Khi `.md` repository đã đổi,
chỉ chấp nhận bytes lịch sử khớp size/SHA chính xác từ Git và báo revision; không bỏ qua mismatch.
Code/raw/local/binary vẫn so đúng file hiện tại. Negative test kiểm cả tìm đúng bytes cũ và từ chối
hash không tồn tại; không sửa lịch sử checkpoint để cập nhật canonical docs.

`terrain_shadow.py`: facade offline cho purekernel `server/terrain_model.py`, model candidate cho profile map tường/box/floor rõ ràng và simple
bombpower≤2. Không service/socket/report; dùng API `TerrainShadow`/`Bomb` để so retained native
callbacks/flag snapshots trước khi tích hợp policy. MapSHA là metadata, caller phải kiểm actualbytes.
[Contract/giới hạn](../docs/PROTOCOL.md#terrain-shadow),
[phép so/source](../docs/evidence/terrain-shadow/README.md). Constraints:
`python3 tools/test_terrain_shadow.py`; tests/model output không thay native oracle hoặc đóng gate.


`export_probe_profile.py --host-back-rejoin`: opt-in5fixtureCP035 cho normalAhostBack/Bpromotion/Aguest
rejoin/ReadyA; chỉ thêm index path/size/SHA/sequence vàpolicyname, không phân phối binary hoặc tựchứngnhận
hành vi. [Contract/currentlimits](../docs/PROTOCOL.md#host-normal-back). Kiểm `python3 tools/test_export_host_back.py`.


`check_issue_links.py`: read-only GitHub ↔ currentcatalog/PLAN/taskdeclarations, dùng
[datedsnapshot/review](../docs/evidence/issue-reconciliation/README.md) mặc định; `--live` chỉ đọcGitHub,
`--capture PATH` tạo snapshot mới cóUTC/caplimit. Check open/closed/backlinks/URL/PRexistence/PLANactive
markers vàclosedtask-active-step; không tự đọc hiểu acceptance, đóngtask/gate hoặc sửacompletedhistory.
`check_docs.py` gọi offlinecheck; snapshotcũwarning>24h, output luôn nêuas-of. Kiểmnegative:
`python3 tools/test_issue_links.py`. Thứ tự công việc vẫn chỉ ởPLAN, issue là nguồn workstatus.
