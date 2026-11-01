# CP049 — guard124 theo khóa đã cấp, hiệu ứng client và giới hạn

Baseline CP048/[PR57](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/57), merge
`4d77eafb9d30e8f40b7bc9d5a066089ba95aa035`. KRE-012/issue58 phụ thuộc KRE-005b/issue14.
Codex solo/tự kiểm; review độc lập [waived](../checkpoints/review-waiver.json).
[Contract/policy duy nhất](../../PROTOCOL.md#pickup-round-key), [thứ tự](../../PLAN.md#current-order),
[câu hỏi](../../RESEARCH.md#gap-floor-pickup). Raw ở `local/research/codex/stale124-r1/`.

## Reuse trước phép đo

`reuse-question.json` giữ path/hash/build/locator/prooflevel và khoảng trống tối thiểu. Đọc CP030/
042/047/048, cached E2E390/RVAA2E390 (E2E5AB đọc record+20), ItemClaims và native report124;
boomkr-re kr-data; boom-re oc_e-game444–455/session.rs360–364; boom-rs VN reference, l10n router
và archive KNOWLEDGE. VN117→158/broadcast-all là phương pháp/cảnh báo validator, không contract KR.
Dùng lại124/165, codec, secure getter, Roller, recipe isolation/GDB và keyrotation; không RE lại
receiver/table/kim/queue/movement handler, không nhập code server VN hoặc suy layout từ tên item.

Runtime A/B copy SHA `e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef`;
static `660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3`;
modO raw/base400000 `a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5`.
RVA=VA−400000; không PE-remap modO. Không sửa binary/pack/DB hoặc ép actor/stat mới.
Mỗi controller dùng lease, namespace user/net/mount/PID, chỉlo/no route/files-only NSS;
A/B prefix/display/wineserver/game-dir riêng. Wine diagnostic copy, không Windows nguyên bản.

## Bốn run, không gộp setup lỗi thành proof gameplay

| Run / UTC01/11/2026 | Phạm vi/kết quả | run.json SHA256 |
|---|---|---|
| unguarded-control / 11:41:41 | Proxy TCP3858→23858; dừng trước sảnh/item,4connection. | `182cfd7a279064f91d1f5a9dfc8f0451e94463cca95a0dbd3a9a4e547c151767` |
| unguarded-trace / 11:43:42 | Thêm S2Ctrace; cùng stop trước item, không replay124. | `e3e45e97d7969b9b008e9e1721f5b64c2f4f70dc03bf4c8631a75bd0c09cbdce` |
| dispatch-control / 12:11:30 | Service CP048; oldbody được nhận,165/stat vòng2 trước input,hai result/return. | `6acf3a54467ff127d745b320f7ee5b24d2c425bf03ada5f65ddc02a2f5b68519` |
| dispatch-guard / 12:17:03 | Oldbody bị loại,actualcurrent124 vẫn165/stat,hai result/return. | `943296dfafdf226c753d7bafe3384a7cb1c101274bba8e67beef05c16cc4a2da` |

Hai setup proxy có SIGILL2260DF8/caller1259F11 sau request6/nhánh launcher, chưa67/Start/item.
Trace có forward nguyên75byte channel17/SHA5bb5a42c… nhưng chưa thấy native framer xử lý reply đó.
`error=None`/launcherreturn0 trong raw không là pass. Nguyên nhân unknown; không quy toàn lỗi cho
proxy chỉ từ tương quan và không patch thêm import để ép qua. Giữ `<run>-hashes.json`, script/trace.
Proxy v1 chưa được archive lúc load; `tcp_replay-v1-reconstructed.py` chỉ đảo delta thêm S2Cnote,
không được coi là source hash chụp tại runtime. `measurement-change.json` ghi giới hạn này.

Đổi phép đo: `runtime_dispatch.py` dùng production `receive_channel`, nhớ frame/body actual124
A vòng1, sau153 cả hai ở vòng2 khoảng3s chèn lại body đó trước input mới khoảng9s. **Chỉ body cũ
bypass tầng C2S TCP/framer**; actual client TCP và S2C165 đi trực tiếp bình thường. Không đổi C2S
seq thật hoặc gọi injection là request mới do native writer phát. `measurement.injections` và
`item_requests.provenance` phân biệt `captured_body_injected_at_dispatch` với `actual_tcp_client`.
Không tuyên bố đã kiểm TCP admission của gói chèn; hai proxy đầu chưa đạt mục tiêu đó.

## Kết quả native và policy

Control chạy snapshot service CP048 SHA `5bf9470502a3b0539e2ecc1ec7549987d25b9b95011e76947a9a162d545d2e29`;
guard chạy SHA `2fb2cca3949e9da746379bb6bd7ffed3778869bcb876e5a3d50975fc8cd65ac2`.
`baseline/server/`, `guard-source/` và `policy_sources` giữ exact bytes, không sửa module trong run.
Cùng helper/GDB/profile/map/catalog17/kind2/IDs1,2/slots0,1, input/retry hữu hạn; thời gian thực
khác nhau giữa run, không gán mọi khác biệt UI/outcome cho guard.

- Control: chỉ một actual124 ở vòng1/k12345. Oldbody giữ nguyên cả field/frame SHA từ actual đó,
  được dispatch ở vòng2/key12346; positive165 phát A/B trước input vòng2. Native165 đọc12/12/errors0,
  effect3→B48350; secure actorA+5E8 **5→6**, max+5DC7 trên cả hai view; không actual124 vòng2.
- Guard: oldbody/k12345 ghi `round_key_mismatch`, không phát165 và không chiếm claim17.
  Sau input, **actual TCP124 mới/k12346** được nhận và phát165 tới A/B. Callback12/12/errors0,
  secure Roller trước/sau **5→6** cả hai; trước pickup mới vẫn5, slot1 current4 giữ trong snapshot.
  Đây là đo effect của165 hiện tại, không suy từ icon hoặc log service.
- Cả hai run có hai121/clock, actual bom/17/19/170, actorA state1 ở cả hai trước175,
  hai175outer302/inner292 và180full3/phòng trên cùng8connection. Clock vòng2 Roller reset5.
  Rank/fuse6s/placement/drop/auth/roster vẫn là fixture/policy; không nghiệm thu fullmatch/K5/K6a.

## Kiểm và bước tiếp

`verification.json`:4133 quan hệ cơ học/4sealedruns (2setupfail),369native packet cases,
24native153,187actualUDPchecksumrows/126unique,1110secure snapshots/getterchecks,
123codec/native-prefix comparisons. Native14EF300/typed/blob/CRC và52A350 giữ code; allocation/
receiver storage là model. Getter oracle khởi tạo counter19D88D4=0 và reset1A7E438/read để bỏ
periodic rekey/storage trong emulation; không đổi state/counter runtime client.

28server tests (3 mới) kiểm old/current/duplicate, origin/catalog và fullu32 so với key+256 cùng low8;
`captured-policy-check.json` kiểm body actual/native-checked CP048 qua dispatcher có guard.
Đây là policy regression, không oracle gameplay.12ảnh đã đọc qua2composite, nguồn/hash trong
`visual-index.json`; raw cursor/stat/branch là bằng chứng riêng. Native/GDB probes không có lỗi
ở hai dispatch runs. Checker không tự xác nhận semantics hoặc đóng gate từ số kiểm.

Recipe argv/hash ở `<run>-recipe.json`/`<run>-hashes.json`; chạy label/prefix mới, controller tuần tự:

```bash
python3 local/research/codex/stale124-r1/run_probe.py new-control CA-r2-stage-social.exe 300 wine_probe_stale124_dispatch_control.py local/research/codex/stale124-r1/service-v2.gdb
python3 local/research/codex/stale124-r1/run_probe.py new-guard CA-r2-stage-social.exe 300 wine_probe_stale124_dispatch_guard.py local/research/codex/stale124-r1/service-v2.gdb
python3 local/research/codex/stale124-r1/seal_run.py new-guard
```

Guard equality là **design** của emulator, không validator/auth server gốc. Không kiểm t/distance/
actorstate hoặc request giả đã sửa k hiện tại; không chứng minh wire epoch/toàn stale-safe/UDPtypes.
KRE-011/issue56 đã đóng sau PR57; KRE-005b/012 còn scope mở. AUD015/CP048 giữ, audit kếCP052.
