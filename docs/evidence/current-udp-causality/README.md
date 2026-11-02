# CP070 — Cắt/khôi phục UDP hai chiều trên service hiện hành

Baseline CP069/PR99 merge `03f1afbeb6eb6d638f47f4a4ce5ea9845ae95fa3`;
KRE-033/[issue100](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/100).
Codex solo/tự kiểm; [review waived](../checkpoints/review-waiver.json), tích hợp chờ PR.
[Contract/giới hạn](../../PROTOCOL.md#current-udp-causality),
[câu hỏi còn mở](../../RESEARCH.md#gap-peer-movement), [thứ tự](../../PLAN.md#current-order).
Raw/script/log/capture/seal: `local/research/codex/current-udp-causality-r1/`.

`reuse-question.json` pin CP027/045–047/061/065/069, native actor getters, cached mainloop
`1256B3B` trong `pickup-time-r1/ida/observed-counter-write.json`; KR cũ, VN oc_e-game §5a-6,
boom-rs VN reference, l10n và archive. Dùng lại endpoint99→153, probe/readers và phương pháp
local/remote; tránh RE lại handler, clock, item, auth/cache hoặc quét toàn UDP grammar.
CP049 proxy thất bại trước Start giữ nguyên; thử này dùng UDP trực tiếp và netfilter trong namespace.
Không thêm thân hàm native, patch client hoặc sửa production service/profile CP065 distinct/floor.

Runtime A/B SHA `e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef`;
static `660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3`;
modO raw/base400000 `a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5`.
RVA=VA−400000. Nguồn binary/pack/DB giữ nguyên. Một controller/lease; Wine copy CP059, prefix warm
copy CP062, user/net/mount/PID namespace chỉ lo/không route/NSS files-only, local stub và display riêng.

Preflight trong namespace bỏ đi: input hook drop đúng hai cổng UDP synthetic, counter1, flush rồi
receive lại; output hook trước đó gây sendto EPERM, raw được giữ. Không sửa firewall host.
Runtime lấy cổng từ actual99, đối chiếu peer153; rule chỉ input/127.0.0.1/UDP sport+dport thực,
chặn toàn luồng tương ứng, counter trước flush và delete table trong finally. Không packet rewrite.
Native snapshot đọc wrapper state+28/x+4C/y+58 ở mainloop return; GDB Python chỉ đọc phase file/
host monotonic clock. Native52A350 và checksum độc lập kiểm giá trị; không gọi hàm/ép actor runtime.

| Run | Kết quả | run.json SHA256 |
|---|---|---|
|cut-current|Controller assert sai kiểu metadata interfaces/routes; trước mọi nft command, `error` tồn tại nhưng chuỗi rỗng. Không là lỗi client/mạng.|`f6a172328344f924dd18ecf4e63600326efddb05a4b2ebd929119d7a33b457d2`|
|cut-v2|Hai counter24/28 dương, control hoàn tất; mẫu mỗi60hits thiếu A-own-cut và A-view-B-restored-after, chỉ bằng chứng partial.|`7776fa5532bc5f843f85595e215b09162aa23e5a1f717219587ccc6a3cdbb787`|
|cut-v3|Mỗi6hits, baseline6s/cut-after8s/restore-after8s; bắt đủ paired phases cả hai chiều, không lỗi controller/service.|`94a19f73fcef7834eba9d99b49332bb114f5da4d7762bf600f16d1da5532edf5`|

Run cut-v3 bắt đầu 2026-11-02T17:26:34Z, một Round/8 connections, không bom/result/chu kỳ2.
Sampling đổi có thể đổi quãng đi của UI; vị trí native thực là nguồn, không suy tốc độ/thời gian game.
Actual99 A9701/B9618, peer153 quảng bá đúng cổng đối phương:

| Chiều cắt | Drop packets/bytes | Vị trí own trước → trong cut | Remote trong cut | Sau restore + input mới, hai view |
|---|---|---|---|---|
|A→B|34/1518|(120,159)→(120,279)|(120,159)|(120,339)|
|B→A|37/1665|(120,399)→(172,399)|(120,399)|(200,399)|

Ở phase restored-before, trước input mới, remote cũng quan sát được vị trí cuối cut: A(120,279),
B(172,399). Không quy điều này cho queue cũ được flush: chưa phân biệt periodic send/retransmit/
queue hoặc gói cụ thể. Không dùng counter tổng làm grammar của movement. Trong cut, own khác
remote; khi mở lại, cập nhật được nhận trước và sau input mới. Đây là can thiệp transport trên
Wine copy local hiện hành; không chỉ ghép paired positions từ run cũ.

`verification.json`: 2795 quan hệ/hash/seals/source, 378 native framer cases,
6 native153 cases, 1170 secure-field snapshots trên ba run,
146 phase samples ở cut-v3. GDB phase timestamp chỉ hostclock, từng view có thời điểm riêng;
không same-tick/fairness proof. Native heap/receiver/getter counter models chỉ ở emulation.
Code service/export không đổi: dùng lại baseline72 server/18 checkpoint tests; docs/catalog/artifacts/
required local-history/diff kiểm lúc bàn giao. Audit AUD021/CP068, periodic kế CP072.

Tái tạo label/prefix/phasepath mới, không overwrite sealed raw hoặc chạy chồng lease:
`run_probe.py new-label CA-r2-stage-social.exe 300 wine_probe_udp_cut_v3.py cut-v3.gdb`,
đổi phasepath theo label trước chạy; preflight private namespace trước. Sau run completed: seal_run.py,
parse_fields.py với các run, analyze_samples.py, derive_roles.py/native_153.py, verify.py.
Recipe/hash chính xác trong `<label>-recipe.json`, `<label>-hashes.json`, `sampling-retry.json`;
không dùng ví dụ label mới với phasepath cũ. Counter/hash kiểm cơ học không tự xác nhận ngữ nghĩa/gate.

Chưa full UDP grammar/reliability/fallback/NAT/auth/relay/Windows vanilla/LAN/Internet/K5/K6a/fullmatch.
CP045–047 vẫn nguồn riêng cho oldUDP/key/queue; thử này không replay oldUDP hoặc đo reset vòng mới.
Bước tiếp: dùng đường đi tới biên đã đo CP066 và Fire effect CP068 để tạo hai owner/actual cells cách2,
root power2 so control power1; đọc state/process trong callback root158 trước timer riêng bom2.
Đây là điều kiện mới cho blocker CP064, không lặp vô hạn Right hold để cố được ô liền kề.
