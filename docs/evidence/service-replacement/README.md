# CP051 — C mới thay A, B Start và vòng B/C trên service độc lập

Baseline CP050/[PR61](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/61), merge
`9c6ac926630431c246a98475039fdde69a4f03f9`. KRE-014/[issue62](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/62),
phụ thuộc KRE-005b/issue14. Codex solo/tự kiểm; independent review [waived](../checkpoints/review-waiver.json).
[Contract/giới hạn](../../PROTOCOL.md#service-replacement), [câu hỏi](../../RESEARCH.md#gap-disconnect-replacement),
[thứ tự duy nhất](../../PLAN.md#current-order). Raw: `local/research/codex/service-replacement-r1/`.
Snapshot khi ghi CP051: nhánh `kre/service-replacement`, chưa merge. Trạng thái tích hợp hiện tại ở [PR63](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/63); manifest giữ nguyên snapshot trước merge.

## Tái sử dụng đúng câu hỏi

`reuse-question.json` pin path/hash/build/locator/prooflevel: contract/evidence CP034/035/040/044/050,
cache getter/framer, boomkr-re kr-data, boom-re lobby/session, boom-rs VN reference, l10n router và
archive. CP040/044 đã chứng minh C identity3/slot0 và B Start/bom/death/result trong **embedded
harness**, không phải service độc lập. Dùng trực tiếp fixture C/106/87/110/175/180, cách đo ID/chief/
securestate; chỉ tích hợp admission/roster/current-host/livepeer/sequence/key vào service. VN chỉ cho
method/lifecycle tham khảo; không chuyển nguyên wire/ID/layout. Không mở thân hàm binary mới,
không tìm lại consumer item, UDP, login hoặc room. Khoảng trống cần đo là **cùng run service riêng**:
A EOF→cleanup→C Join/Ready→actual B104→B/C bom/statepeer/result/return.

`late-input-reuse.json` đối chiếu ca input vòng2 partial CP048/050, recipe CP044 và ảnh run đầu.
Recipe late input và probe tắt toàn UDP chỉ chuẩn bị; **chưa chạy**, vì run lite đã đạt cạnh cần đo.
Không dùng chúng làm bằng chứng hoặc khẳng định nguyên nhân input.

| Artifact | SHA256 |
|---|---|
| Runtime A/B/C Wine copy | `e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef` |
| Static copy | `660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3` |
| modO raw/base400000 | `a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5` |
| Service dùng hai run | `7b1d55dae801b8c79e5479a38c7ee5701b27888aa9e942be8f89c063f79baec2` |

RVA=VA−400000; không PE-remap modO. `runtime-source/` giữ bytes module thực chạy; code bàn giao
chỉ sửa thêm docstring mô tả A/B/C sau runtime, không đổi behavior. `cp051-snapshot/` giữ bản bàn giao.
Binary/DB/pack nguồn nguyên vẹn; không thêm patch/ép actor/clock hoặc phát synthetic118/17/19.
Namespace user/net/mount/PID, chỉlo/no routes/files-only NSS; runtime lease và controller tuần tự.
Ba prefix/display/game-dir/wineserver riêng. Service quảng bá endpoint direct từ actual99 qua153;
proxy A/B đặt0. UDP proxy đo lường không quảng bá, không lấy log proxy làm proof peer movement.

## Hai run và phạm vi riêng

| Run / UTC01/11/2026 | Kết quả | run.json SHA256 |
|---|---|---|
| replacement-first /13:14:08 | C3/Join/Ready, B104, B/C121/clock/key12346; actual C124/165/Roller. Không B118 vòng2/result. | `47c1331712e8c877e51c67398a247d3ff730db0f104c9d7b33c33598ad2c61e5` |
| replacement-lite /13:23:03 | C3/B104, actual B118→159/158→B17/19→C170/C21→hai175/180/phòng. | `da1d694d15aaf2e732906d280f261b81ef9bd589b5bdf473a87ff9afd10ee7e5` |

Cả hai recipe360s,12connection (4/vai), cùng service/profile41/binary/UI recipe. Vòng1 A/B có
actual B118/159/ID1; controller đóng prefixA trước fuse6s. Ticket gốc bị loại `round_closed`,
không158; B forfeit175/180 rồi107 clear ID1→0/110 chief0→1. C chỉ khởi động sau refresh.
Native C `AUTH_LOCAL_ID_SET slot=0 id=3`, `PREPARE_OWN_MATCH value=1`; actual Join82/Ready95
và B104. Roster vòng2 B2/slot1 + C3/slot0,153 key12346 từ allocator giữ nguyên instance;
A đóng không nhận packet vòng mới. Sequence B tiếp tục bước3, C bắt đầu state riêng bước3.

`replacement-lite`: sau một bounded Space retry, **actual B118**, key12346/serial1→globalID2;
159/158 chỉ tới B/C. B event17/19 được170 tới C; native170 full10/21/errors0, C thực phát21.
Trước175, secure actorB+28=1 trên B localround2 và C localround1 (cùng roomgeneration2),
actorC state0.175 outer302/inner292/errors0, raw rank21/152=1/2 (slot1 thua theo policy),
180full3/errors0 rồi110 giữ chiefB1/C0 và guest unready. Ảnh `replacement-final-room` có Start ở B,
Ready ở C; ảnh không thay native proof. Đây là **một forfeit + một vòng B/C**, không hai trận đủ gate.

Lưu ý phép đo: `service-lite.gdb` disable79–101 trước continue, nhưng Start121 tự enable85–101.
Thực tế chỉ loại probe UDP ingress/queue/drain79–84; không gọi đây là control tắt toàn UDP.
Native log phơi rõ key/game probes còn chạy. `measurement-notes.json` ghi sửa cách hiểu này;
`service-no-udp.gdb` bỏ autoenable và harness retry45s/60s chưa chạy. Không kết luận queue probes,
tutorial hoặc focus gây thiếu118 run đầu; xdotool return0 chỉ chứng minh gửi input, không native xử lý.

## Kiểm và tái lập

`verification.json`:2313 quan hệ/hash của run lite, không tự chứng nhận semantics/gate.
Hai run native reports chung:288 packet cases (valid/CRC/partial),24native153,684secure snapshots.
`partial-first-reports/` giữ bộ báo cáo run đầu105packet/12native153/276snapshots, không cộng lại
thành oracle độc lập. Framer14EF300/typed/blob/vector153 E8A340 và securegetter52A350 dùng code
native; receiver/heap là model. Counter emulation19D88D4=0/reset1A7E438 bỏ periodic rekey/storage,
không sửa counter client runtime. `*-hashes.json` niêm phong raw; `visual-index.json` và hai composite
pin ảnh đã đọc. `runtime-source-sha.json` pin code thực chạy; source ledger phân biệt static/runtime/design.
36server tests có3replacement regressions;7catalog/18checkpoint tests cùng kiểm docs/artifact/history
được lưu ở các file `*-check.txt`/`*-tests.txt`. Các kiểm chỉ kiểm quan hệ có thể đo được.

```bash
python3 tools/export_probe_profile.py --source-root local/research/codex --out local/new-profile-replacement.json
python3 local/research/codex/service-replacement-r1/run_probe.py new-run CA-r2-stage-social.exe 360 wine_probe_service_replacement.py local/research/codex/service-replacement-r1/service-lite.gdb
python3 local/research/codex/service-replacement-r1/seal_run.py new-run
```

Đọc exact argv tại `<run>-recipe.json`; harness hiện trỏ profile41local. Dùng label/prefix mới,
không đè sealedrun; chạy controller tuần tự. `verify.py` nhận label nhưng acceptance riêng cho ca
thành công với B/C lifecycle; không chạy rồi sửa raw run partial để giả pass.

Admission theo thứ tự/max12 và fixture identities không là account auth hoặc membership tổng quát.
Chưa kiểm setup/idleEOF/sendfailure/timeout/loadfail/reset variants, toàn124validator/t/distance/
actorstate/forged-current-key, roster/key lifetime/fullUDPgrammar/auth/cache/codec/Windows vanilla.
K0–K7/K5/K6a/LAN/Internet giữ trạng thái cũ. AUD015/CP048 giữ; audit định kỳ kếCP052.
EOF ở setup là phép kiểm tiếp đã xác định trong RESEARCH, không cần tìm lại consumer đã giải.
