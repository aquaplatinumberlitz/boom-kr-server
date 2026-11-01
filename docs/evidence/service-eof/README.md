# CP050 — EOF trong service riêng và ca giữ99

Baseline CP049/[PR59](https://github.com/aquaplatinumberlitz/boom-kr-server/pull/59) merge
`a6b62cd93bb96a30a5e7fc55b697d90dbbce9703`. KRE-013/issue60, phụ thuộc KRE-005b/issue14.
Codex solo/tự kiểm; independent review [waived](../checkpoints/review-waiver.json).
[Contract/policy](../../PROTOCOL.md#service-eof), [câu hỏi](../../RESEARCH.md#gap-disconnect-replacement),
[thứ tự duy nhất](../../PLAN.md#current-order). Raw ở `local/research/codex/service-eof-r1/`.

## Nguồn dùng lại, không tìm lại receiver

`reuse-question.json`: PROTOCOL/RESEARCH, CP039 hostEOF/CP040 guestEOF/CP044 pendingtimer,
CP049 service, RoundFence và cached harness/native getters; boomkr-re kr-data, boom-re lobby/session.rs,
boom-rs VN reference, l10n router và archive KNOWLEDGE. Ledger pin path/hash/build/locator,
static/runtime/design, phần reuse/KR gap/phép đo tối thiểu. VN membership/hosttransfer là method;
không áp opcode/layout/policy V1 Rust lên KR. Không RE lại107/110/175/180/clock/actor/queue.
CP039/040 native no-policy controls giữ giá trị, không cần chạy lại cùng ghost/stall để tích hợp service.

`loadbarrier-reuse.json` đối chiếu CP021/027/028 và code đã chờ cả hai99. Chỉ thêm phép giữ
actual B99 tại dispatcher10s; không tìm lại99writer/153reader hoặc mở thân hàm binary mới.

| Artifact | SHA256 |
|---|---|
| Runtime A/B copy | `e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef` |
| Static copy | `660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3` |
| modO raw/base400000 | `a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5` |
| Service dùng cả ba run | `ef56704a56a0bf127bd879e1ccaea5f645be37bd0a60f7844e734c8a8b1f32a9` |

RVA=VA−400000; không PE-remap modO. Source binary/DB/pack giữ nguyên; không thêm patch/ép state.
Controller có lease, namespace user/net/mount/PID chỉlo/no route/files-only NSS; A/B prefix/display/
wineserver/game-dir riêng. Profile mới34fixture thêm rankAwin/Blose và promoteB đã đo từ CP029/034,
loader kiểm bytes/SHA/CRC. Profile32 cũ không đủ lifecycle: fail rõ trước listen, không sửa raw/profile cũ.

## Ba run có phạm vi riêng

| Run / UTC01/11/2026 | Kết quả hẹp | run.json SHA256 |
|---|---|---|
| host-pending / 12:31:26 | Actual bomB/159, killA trước fuse; timerdrop, B175/180/107clearA/110chief1. | `67c3d63b43fc91d42d0d9b58d82db35a02ca4f19780718e50d1dd1c251402c2c` |
| guest-pending / 12:35:34 | Actual bomB/159, killB; timerdrop, A175/180/107clearB/110chief vẫn1. | `51eb44e9a10ef200fba63b04bc5f8f704b747128db5d7466e4d0250774610d21` |
| delayed-load / 12:39:02 | GiữactualB99 dispatch10,002s; không phase sớm, release→197/196/153/clock hai phía; một result/return và Start2/clock2. | `569222e29909c17df8029a0c50b0bb35a145d188f3fe1f4fb7aee9a44a9c4e26` |

`host-pending`/`guest-pending`: cùng ExVillage01/278, IDs1/2/slots0/1,8connection, B actual118
owner1/serial1→ID1/159 cả hai. Controller kill đúng prefix A hoặc B khoảng2s sau server record,
trước fuse thử6s; actual channel EOF vào `close_connection`. Service finish ngay trong owner,
ticket ban đầu còn trong scheduler tới deadline rồi bị loại `round_closed`, không phát158.
Không gọi đó là hủy/xóa scheduler entry ngay tại EOF; không có synthetic17/19/death.

Forfeit theo policy:12s sau EOF phát175 cho survivor,5s sau180,3s sau107/110. Native175
outer302/inner292/errors0,180full3. Native blob rankraw21/152 lần lượt2/1 khi A rời,1/2 khi B rời.
Đây là outcome được emulator chọn, không thuật toán rank/authority server gốc. Secure actor trước
rank không state1 (hostEOF: A0/B14; guestEOF giữ giá trị riêng trong actor-fields), không dùng
forfeit thay bằng chứng death. Không gửi dynamic packet sau timestamp close của bất kỳ connection.

Native107 D603A0 đọc6/6/errors0, wrapper rewind rồi cursor cuối1/6 intentional;12400D0/12401A9
xóa đúng ID1→0/slot0 trênB hoặcID2→0/slot1 trênA. Getter chiefB0→1, chiefA giữ1;
ảnh `after-cleanup` có đúng một người và nút Start. Helper UI tiếp tục thử nút sau đó nên ảnh cuối
có modal một người; không gọi modal đó là protocol failure hay proof Start với peer mới. Chưa C join.
Teardown cuối guest run còn đóng A, ghi thêm cleanup **rỗng/không packet**; khác lần cleanup B ban đầu.
Verify đầu giả định mọi cleanup record phải chỉ một; sửa phân biệt wirecleanup với teardown rỗng,
không đổi raw/câu chữ để giấu một lần phát trùng. Không có107/110 thứ hai tới client.

## Giữ99 và giới hạn đo loading

`delay99.py` wrapper nhận frameTCP B99 thật, pin `held-B99.bin` SHA
`31ada4d8e396d2c6fdf68d9b9985195053a2e4c131afbce1bfd7471e2966a924`, sequence124.
Giữ processing99 **10002,462492ms**, còn A đã có init99. Không có197/196/153 send trong khoảng giữ;
release mới xử lý body99 và production service phát phase. Native frame/getters kiểm actual99 và
197/196; decoder153 full97, negativeCRC/partial; cả hai có marker clock thật sau phase.
Actual input→124/165 và118/bom/17/19/175/180 vòng1; Start2/clock2 đã có nhưng hết185s chỉ một
bom/result/return, không suy hai trận hoàn chỉnh. Không gán nguyên nhân thiếu request vòng2 từ
ảnh tutorial. Wrapper không force clock/client state, không là networkloss hoặc server KR gốc
load-policy proof; timeout/latejoin/loadfail/barrier mọi variant còn mở.

## Kiểm và tái lập

`verification.json`:1700 quan hệ EOF, `load-verification.json`:812 quan hệ hold/release/phase,
3sealedcontrollers. Native reports chung261packet cases (gồm99/197/196 bổ sung),12native153cases,
420secure snapshots/getter checks; EOF-only báo cáo đầu78packet/120snapshots lưu `eof-reports/`
như scope lúc đó, không cộng hai bộ thành oracle độc lập.32test đầu, sau thêm profile-missing case
thành **33server tests**,5mới cho close/ticket/rank/duplicate/deadtarget/profile; cùng7catalog/18checkpoint.

Framer14EF300/typed/blob/CRC và52A350 giữ code native; allocation/receiver là model. Getter chỉ
trong emulation khởi tạo19D88D4=0/reset1A7E438/read để bỏ periodic rekey/storage, không đổi counter
runtime client.12ảnh nguồn đọc qua3composite; hash ở visual-index. Checker chỉ kiểm quan hệ,
không xác nhận semantics/gate từ số kiểm. Code/helper/sourceSHA/run.json pin qua manifest CP050.

```bash
python3 tools/export_probe_profile.py --source-root local/research/codex --out local/new-profile-eof.json
python3 local/research/codex/service-eof-r1/run_probe.py new-host CA-r2-stage-social.exe 185 wine_probe_service_eof_host.py local/research/codex/service-eof-r1/service.gdb
python3 local/research/codex/service-eof-r1/run_probe.py new-guest CA-r2-stage-social.exe 185 wine_probe_service_eof_guest.py local/research/codex/service-eof-r1/service.gdb
python3 local/research/codex/service-eof-r1/run_probe.py new-load CA-r2-stage-social.exe 185 wine_probe_service_delayed99.py local/research/codex/service-eof-r1/service.gdb
```

Đọc argv exact tại `<run>-recipe.json`; script hiện trỏ profile đã đo trong local, dùng label/prefix mới,
controller tuần tự, không đè sealedrun. `service-source/` giữ exact module bytes của ba run.
Chưa service accept C/identity mới/roster mới/Start từ B, account reconnect, idle-EOF/setup race,
sendfailure/broadcast race, fullvalidator/auth/cache/codec/vanilla/reset hoặc K5/K6a/LAN/Internet.
AUD015/CP048 giữ nguyên; audit định kỳ kếCP052. Phần cần tiếp ở nguồn chuẩn đã dẫn, không tạo plan riêng.
