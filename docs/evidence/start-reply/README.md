# CP020 — start121, identity local và scene trận

Correction CP058: IDprefix vẫn gán đúng ID, nhưng4byte recipe cũ định dùng cho `string(empty)`
không là encoding chuỗi hợp lệ; native u16length lỗi tại cursor29. [Bằng chứng/sửa hiện hành](../../PROTOCOL.md#auth-profile).
Không sửa raw/run hoặc proof scene/ID CP020.

Contract duy nhất ở [PROTOCOL](../../PROTOCOL.md#start-reply); thứ tự/gate ở
[PLAN](../../PLAN.md), phép đo tiếp ở [RESEARCH](../../RESEARCH.md).
Baseline `759f60e6a0eddfded855aa6471444b8e12234bfd` (CP019/PR27 đã merge).
Codex solo/tự kiểm, review độc lập **waived** theo chủ dự án. CP020 snapshot trước tích hợp;
PR nhánh `kre/start-reply-runtime` là nguồn trạng thái merge. AUD006 định kỳ CP020, kế CP024.
Raw `local/research/codex/start-reply-r1/`; SHA từng artifact/script/run/log/ảnh/native oracle trong
[manifest CP020](../checkpoints/manifest.json). Không commit binary/disasm/capture.

## Target và tái tạo

Control dùng CP018 copy SHA256 `94ee93dc7978a1ab22f25ec877e29ddcab254ca761f2eac22534421e2dd628e8`.
Copy `CA-r2-stage-menu.exe` SHA `a6a9c88dbbe953c596ed8cc4b5b05176dfd7620acccfd0ddfaf7f11b045896a1`
chỉ bind import16EA6CC→GetMenuItemCount candidate.
Copy `CA-r2-stage-social.exe` SHA `e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef`
thêm16EA8BC→NMCOChangeMyInfo candidate, dùng DLL bản copy SHA
`7e038696ec69db579f71e029da66f5b5500c56375ff77b24d93529ba7faf9493`.
Patch chỉ import descriptor/slack/IAT, real exports, không ép return/stage hoặc sửa ownflag/local-ID.
`menu-patch.json`/`social-patch.json` giữ byte before/after và rollback về source copy;
kiểm reconstruct byte-exact. Client/DB/DLL nguồn không sửa. Tên import gốc vẫn candidate.

Static target `CA-r2-login-logout.exe` SHA660447… và native modO base400000 SHAa08950…
đã khóa ở [CP019 evidence](../room-map/README.md); các code edge đang dùng chung với runtimecopy.
VA/RVA theo base400000. REA provider chưa phục hồi; dùng công cụ hiện có GDB/PE/Capstone/Unicorn.
`vn-start-reuse.json` khóa hash snapshot VN lobby/packets; chỉ mỏ neo normalstart/load/clock.
KR opcode121/layout/u16 position được phục hồi riêng, không chuyển VN117/501 thẳng sang KR.

Chín run client tuần tự và một launcher fail trước runtime. Mỗi run có `*-recipe.json` argv đầy đủ,
`run.json` PID/timeline/copy/harness/network/hash, log/CALog/ảnh, và `*-hashes.json` phủ raw.
Namespace user/net/mount/PID chỉ lo, không route outbound, NSS files-only, peer loopback.
Prefix riêng, game-dir/cache kế thừa chung **không chuẩn hóa**; persistence/cold/warm vẫn mở.
Recipe run mới (không dùng ID/output đã có):

```bash
python3 local/research/codex/start-reply-r1/run_probe.py NEW-RUN CA-r2-stage-social.exe 58 wine_probe_start121_id1.py local/research/codex/start-reply-r1/start121-lean.gdb
```

## Các phép đo gom tại mốc

| Run | Quan sát/giới hạn |
|---|---|
| start121 |34s probe hết sau106, trước start; không phải negative121. |
| start121-long |121 nhận map tên/index, inner100/100; slot0ID0 bị loại, vềstage11. |
| start121-slot |Đo ID0/position0/flag0 và slot1position256 do đặt nhầm u16/u8. |
| start121-layout |Sửa đúng spawnu8/positionu16, slot1position1; ID0 vẫn bị loại. |
| start121-guest-only |Bỏ ID0 khỏi list, emptyposition−1; kiểm slot qua, stage13 requested; trap menu22646D4. Không own-player participation. |
| start121-guest-menu |Launcher fail vì patch assertion phát hiện C8 đã bind; chưa có executable/run.json. Giữ recipe/log/attemptscript. |
| start121-guest-menu2 |Bind chỉCC menu thật, vượt trap cũ, gửi99; trap NMCO22657A8. |
| start121-id1 |Auth34byte cấpID1,106ID1 match own; final121129/129, các slot hợp lệ, init CGameStage vt176A470; menu count9; trap NMCO. |
| start121-social |Bind NMCO thật, return0; scene map/HUD/timer02:00; GDB host crash sau đó, không gán thành client crash. |
| start121-lean |Cùng copy/fixture, probes gọn; init/stage13/99 lặp lại, ảnh hai nhân vật trên map/HUD; không log signal. Clock/input chưa thử. |

`start121-social/screen-18.png`: map băng/HUD/roster. `start121-lean/screen-17.png` và
`screen-18.png`: hai nhân vật, local có mũi tên, guest tên KRGuest; timer02:00.
Ảnh là bằng chứng scene/nhân vật đã vẽ; không chứng minh clock, input, gameplay/Battle playable hoặc multiplayer.
Exact asset path/geometry variant chưa trace. End-run teardown/disconnect không là proof steady-state timeout.

## Kiểm và correction

`verification.json`: **2537 kiểm / 9 run pass** về hash/isolation/pinnedcopy/harness,
all capture frames native CRC/partial negatives,18 ca framing fixture, native bounded121/identity,
control ID0/ID1, init/stage/99 markers và hai patch reversible.
`verify.py` chỉ kiểm tính đo được; đọc ảnh/timeline/code là kiểm thủ công riêng, không tự đóng gate.
Native frame/blob/checksum/readers giữ nguyên; allocator/free mô hình. Getter ledger đầu tự gọi đọc
hết133byte **không chứng minh callback thực dùng hết**: runtime guest chỉ103/107; mẫu cuối cắt4byte
u32 dư, native và runtime129/129. Full auth còn thiếu sau bounded34byte, không gọi login hoàn chỉnh.

`measurement-notes.json` giữ lỗi và sửa giả thuyết: maskAL cho u8 oracle, disasm đi vào data/range
lệch không lấy làm instructions, short run, nhầm nhãn/width spawn-position và4byte tail dư.
`patch-menu-attempt1.py` abort trước ghi vì16EA6C8 đã bind win32u!NtUserDeleteMenu; sửa chỉCC.
`social-export.json`/cluster cũ/ABI5cdecl hỗ trợ candidateNMCO; runtime return0 không khôi phục toàn semantics.
`request99-init-anchors.json` bounded negative ở vùng initE57000..E5A000 không tìm immediate63;
probe13ADD60 cũng không hit99, nhưng actualcapture/CALog có99/61byte. Không suy không có request99.

Các checkpoint/raw CP019 trở về trước giữ nguyên. CP020 sửa giới hạn ID0 workaround và bổ sung
scene/nhân vật, không nâng K2/K3/K4/K5. Bước tiếp và pending chỉ giữ ở nguồn chuẩn RESEARCH.
