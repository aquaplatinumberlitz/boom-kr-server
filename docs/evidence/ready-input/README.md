# CP080 — Writer của nút Ready/Start và hai chu kỳ sau rejoin

Baseline CP079/[PR119](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/119),
merge `be860a2eab2726f6cbc6e52c317900dc07c00a18`.
KRE-043/[issue120](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/120);
Codex tự kiểm, [review độc lập waived](../checkpoints/review-waiver.json), tích hợp theo [PR121](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/121).
Contract duy nhất ở [PROTOCOL](../../PROTOCOL.md#ready-input), câu hỏi ở
[RESEARCH](../../RESEARCH.md#gap-diagnostic-service), thứ tự ở [PLAN](../../PLAN.md#current-order).
Raw/cache/recipe/model/source/seal: `local/research/codex/ready-input-r1/`.

## Tái sử dụng trước khi đo

`reuse-question.json` pin docs/cache CP016/028/069/079, KR `kr-data`, VN `lobby`/`oc_e-game`,
boom-rs reference, l10n migration và archive. CP016 cung cấp WndProc/filter/button/delegate;
CP028 có lịch sử retry hữu hạn; CP069/079 bác nhu cầu 106 mới trong ca đã đo. VN dùng checklist
input/stage/control, không chuyển opcode/ID/offset/layout. Tránh tìm lại codec/auth/cache/writer,
tránh thêm wrapper106 hoặc sửa production khi chưa có nguyên nhân.

Các probe cũ khớp 16 byte trên static/runtime/modO trong `input-code-match.json`. Chỉ đọc thêm
một helper `1504D0A` sau khi watchpoint chỉ đúng cạnh; `native-new-edge.json` pin range `0x300`
khớp cả ba artifact và listing tới ret. Không sửa DB/binary nguồn hoặc cấu hình provider.

## Các run cùng câu hỏi

| Run | Quan sát và giới hạn | SHA256 run.json |
|---|---|---|
| dispatch | Double-click rejoin trước native room91 mới, không tới Ready. Codex dừng controller bằng SIGINT có record; exit130, original finally ghi run.json. | `1f820a52578edd471c037d4f9ed349fa94722c58e4d561030d10c36540f96661` |
| dispatch-v2 | Chờ native91; một vòng, ghi đủ click đầu/sau return: mouse-up sau return state0, không callback. Stop theo boundary, exit0. | `ed73f64dd4aa8b8d373b55cfdf099856e63646879f97e21b9b85f1e3a7463cbe` |
| watch | Watch +148/+14C xác định writer trên A/B; một retry B phát95, AStart đầu vẫn mất nhấn. Một vòng, không104 vòng2. | Hash trong `watch-verification.json`/manifest |
| hover | Chuột ra500,450 → vào605,526, chờ0,5s mỗi bước rồi nhấn: Ready/Start sau return thành công ngay, không retry. Hai chu kỳ native kết quả/return/reset cùng run. | `ead2b8ec682d83b5b37e2ba13f7e4e4a602aab88bcbdbb136d97cf23b9081a34` |

`<label>-recipe.json` giữ argv, `<label>-hashes.json` niêm phong file gốc. Trước/sau mọi xdotool
ghi `ui-progress.jsonl`: role từ controller states, DISPLAY/prefix, focus/pointer/process snapshot,
return code/time; `progress.json` ghi nguyên tử/fsync isolation/phase/native counts khi đang chạy.
Không sidecar hoặc controller thứ hai. First run không là lỗi client/protocol; preflight regex đếm
thiếu indirect breakpoint được sửa trước runtime, có record riêng. Không tái dựng run.json.

## Bằng chứng native và phạm vi

Chi tiết writer/callback/control chỉ ở [contract](../../PROTOCOL.md#ready-input).
`input-summary.json` nối timestamp native với OS action; `watch-verification.json` kiểm PC/value/
stack và actual95 riêng. Watch PC là **sau instruction**, không nhầm thành địa chỉ instruction ghi.
Hover control có actual B95/A104 sau return, không106 mới; tám TCP/ID1,2/slot0,1, seq tiếp tục,
keys12345→12346. Hai phía native121/129 byte, clock,175/302 byte,180/3 byte đều hai lần/errors0.
Mỗi vòng actual124 Roller17/Bomb27/Fire28, delta165 cảhai view và baseline vòng2; bốn actual118
ID1..4/power1,2,1,2, native159/158 tương ứng, stateA1 trước175. Hidden-pointer join không đo mới.
Policy fuse/drop/rank/host vẫn design; không phải phục hồi toàn luật server gốc.

Giữ riêng failure planned đầu do parser đánh đồng `Backtrace stopped: Cannot access memory at
address 0x1c` với probe đọc bộ nhớ lỗi. Thực tế đúng hai cảnh báo mỗi role trong bt8 tại watch
post-PC152590D **sau** Start2/stage13, trace vẫn chạy tiếp đủ chu kỳ. `unwind-classification.json`
ghi correction; parser tách warning, checker kiểm đúng text/PC/context và giữ toàn assertion
hai vòng/stat. Không phục hồi được frame thiếu. Các ghi về storage cũ sau rời UI không được hiểu
là trạng thái button còn sống hoặc free/destructor đã xác nhận. Phép đo sau cần tháo watch khi rời UI.

Kiểm: first-two consistency4263; watch verification riêng; hover994; integration1515;
native framer981, typedpacket339, native15312, secure snapshots408. Các số có phần giao nhau,
không cộng thành lượng proof độc lập. Callback/capture runtime riêng với framer/getter/heap emulation.
Run trước partial giữ nguyên. Catalog7/docs/artifacts5/5/checkpoint local/history/diff;
production83 server/18 checkpoint tests tái sử dụng CP077 vì code không đổi.

Private user/net/mount/PID, lo-only/no IPv4 route/NSS files-only, lease một controller;
copy/prefix/display A/B riêng. Runtime SHA
`e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef`,
static `660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3`,
modO raw/base400000 `a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5`;
RVA=VA−400000. Production/profile/fixture/binary không đổi; không native forcing/semantic replay.

Đây là hai **chu kỳ thử Battle local trên modified Wine copy**, chưa fullmatch/K5/K6a, vanilla,
LAN/Internet/full auth/codec/validator/authority/reset/general membership. AUD024 định kỳ qua080,
audit kếCP084. Tiếp theo PLAN là input/placement bom khác owner trong profile hiện hành;
tận dụng writer/control này, không lặp nhánh106 hoặc giữ phím vô hướng.

[CP081](../mixed-input/README.md) bổ sung mixed-owner chain; source-first sửa target của CP071 feedback là Aroot, không B tổng quát. CP080 record giữ nguyên; thứ tự hiện hành chỉ PLAN.
