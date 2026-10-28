# Người chơi, own-slot và controls phòng — CP018

Codex RE/code/runtime, KRE-005b/issue14; [review waived](../checkpoints/review-waiver.json), tự kiểm.
Baseline `624cef14a89737c5b6e1324dc9d898a511201f5b`, CP017/PR25 đã merge.
Contract duy nhất ở [PROTOCOL](../../PROTOCOL.md#room-player), phép kiểm tiếp ở
[RESEARCH](../../RESEARCH.md#q-023). Không có gate mới, PR là nguồn trạng thái tích hợp.

Raw `local/research/codex/room-player-r1/`, ngày29/10 JST/28/10 UTC23h. Manifest khóa script,
metadata, oracle, static ranges và hash index bao phủ mỗi run. Source binary/DB bất biến;
5 artifact nguồn đã verify. Không dùng ID0 như bản sửa full login hoặc ID server gốc.

## Dùng docs VN để chọn phép đo

`vn-reuse.json` khóa snapshot VN `auth-request-r1/vn-login-references/docs/lobby.md`
(cập nhật9/10/16/18/19) và `server/src/packets.rs` (`room_player/room_slot_state/user_list`).
Các mỏ neo: base/accessory của CSetPlayerOnPreparePacket, tên đầu profile blob, match ID cục bộ,
phản hồi trạng thái ô mở controls. KR được kiểm riêng bằng RTTI, targeted readers, native và runtime.
VN103/106/337 khác KR106/110/585; packed state VN3 khác KR4byte. Không copy zero padding256byte,
opcode, map ID, room-type19 hoặc policy VN làm fact KR. VN337 còn dùng ngoài phòng, nên gửi106
sau585 trong stub chỉ là scheduling thí nghiệm; chưa chứng minh pairing server gốc.

API key-state lấy thêm mỏ neo snapshot `vn-reuse-r1/key-state-import-calls.json`; không dùng số
xref hoặc signature tương tự để xác nhận import KR. REA/IDA provider vẫn chưa sẵn như CP017;
GDB Wine/PEfile/Capstone/Unicorn tiếp tục dùng, không đổi DB/cấu hình tool.

## Artifact, patch, recipe và rollback

| Copy | SHA256 / thay đổi |
|---|---|
| CA-r2-login-logout.exe | `660447c293ce92270cb9b5815331bca976e8f156366e1fe50b217b6c12e60ce3`; control CP017 giữ nguyên. |
| CA-r2-room-ptrcheck.exe | `5cf3b798b7cb8b97426e89e1aee0a6102b1a8df69d4e8c3cb173d13db475d051`; import16EA2B4 bind IsBadReadPtr candidate. |
| CA-r2-room-keystate.exe | `94ee93dc7978a1ab22f25ec877e29ddcab254ca761f2eac22534421e2dd628e8`; thêm16EA6A8 bind GetKeyState candidate. |

Copy dưới `local/research/codex/wcp-guard-r1/game/`; scripts patch-pointer-check.py/patch-key-state.py
khóa input SHA, exclusive output, kiểm descriptor bằng pefile, lưu byte before/after và rollback.
Chỉ đổi import descriptor/IAT/slack, không ép API return/handler/ownslot/stage. Tên API gốc vẫn
candidate: IsBadWritePtr/GetAsyncKeyState có thể khớp signature. Trap bytes0F0B+slot do rebuild
sinh, không phải bằng chứng original anti-tamper. SIGILL control vẫn giữ nguyên và không gọi là pass client.

Kế thừa patch chẩn đoán trước, memory1A966FC reset0 sau guard, token local; audiooff/NSSfilesonly,
namespace riêng user/net/mount/PID, loopback-only/no route, Xvfb/prefix riêng. Shared game-dir/cache
không chuẩn hóa. Không suy causal duy nhất, cold/warm hoặc vanilla equivalence từ các run này.

Recipe example với label mới (không ghi đè run đã khóa):

```sh
python3 local/research/codex/room-player-r1/run_probe.py repeat-new CA-r2-room-keystate.exe 59 wine_probe_self_slot110.py local/research/codex/room-player-r1/self-slot110.gdb
```

Harness ghi argv, timestamps, CALog-before/run, screenshots, captures và sent fixture hash. Player106
sau actualC2S585,110 khoảng2s sau106, guest khoảng1s sau110 đều là emu_choice. Mọi kết nối local.
Rollback: chọn copy trước theo patch metadata, harness CP017 `wine_probe_create_reply86.py` không gửi106;
kết thúc namespace/prefix của run. Không patch source hoặc ép count/team/ownslot từ debugger.

## Nhóm run

| Run | Quan sát / giới hạn |
|---|---|
| player106 /49s tối đa | Control copy cũ: profile109/109, full106204/204/errors0; sau parse chạm trap22624DC/import16EA2B4 trong tạo object hiển thị; callbacks chưa xong. |
| ptrcheck /50s | Bind API thật candidate; return0 khi kiểm memory96byte, callbacks chạy hết; ảnh screen-12 hiện KRLocal/nhãn chủ phòng nhưng nút vẫn Sẵn sàng. |
| self-id /54s | ID1 khác local count1/ID0, lookup−1, own_slot0. Click Sẵn sàng không có C2S mới ngoài periodic trong cửa sổ đo. |
| self-slot111 /incomplete | Phiên bị ngắt khi bootstrap, không run.json/end metadata; log không106/585/slot markers. Giữ raw/index; không tính vào7 run hoàn tất hoặc chứng minh đã inject111. |
| self-slot110 /57s tối đa | ID0 match0/own_slot1, gửi thêm96/266; sau callbacks chạm trap22645D8/import16EA6A8 trước khi gửi110. Không dùng run này làm proof handler110. |
| keystate /59s | Bind key API thật candidate;106/own match,1106/6/pending0/enable1; screen-12 nút Bắt đầu sáng, screen-16 click tới modal không đủ người chơi. |
| room-guest /59s | Cùng copy, thêm synthetic KRGuest slot1/ID2/state ready; hai ô hiện, own slot giữ0. Start tới thông báo không đủ đội (screen-16). Đây là một client với roster giả, không multiplayer. |
| room-teams /boot failure | wineboot timeout35s trước client,0 TCP event/observations. Giữ raw, không game success; không sửa timeout để nâng kết quả. |
| teams-retry /59s | Prefix mới cùng copy/harness/fixture: self team0,guest team1; ảnh screen-14 đỏ/vàng, screen-16 click báo map tối đa0 người. Không request start hoặc game. |

Thời gian trên không gồm wineboot; hai trap run kết thúc sớm, không tính sống đủ hạn. Count0 array,
profile scalar0/strings ngắn, guest synthetic; chưa kiểm malformed/count bounds/nonempty accessory,
optional branch khác vùng, ready/team/character request+reply roundtrip, map hoặc game.

## Native và corrections

`build_player.py` dựng field ledger theo KR, raw selector của blob giữ bài học CP016.
`verify_constructor.py`: actual frame + AE4000/AE3360/AE3950/D2D310, đầy đủ packet/profile input
readers/control flow; allocator/object setters/storage side effects được mô hình/bỏ qua theo log.
1220030 được mô hình10; region gate thực0 được đo runtime riêng. Oracle này không chứng minh full object.
Full profile109byte/cursor109 và body204byte/cursor204/errors0 được breakpoint KR xác nhận độc lập.
`verify-frame.py`/verify-slot110*.py có CRC-sai/partial negatives; getter state4 được kiểm byte.
`replay-run.py` dùng mọi capture game socket3/4, seq40+3n reset từng kết nối, positives+negatives.
96/266 getter u16/u32 hoặc3u32 tiêu hết payload chỉ là thử width candidate, chưa schema/semantic.

`measurement-corrections.json` giữ lỗi lần đầu oracle thiếu channel object, giả thuyết111 sai,
run interrupted và breakpointDC6ADD lệch instruction (sửaDC6ADC trước110 run), ranges lệch prefix,
nhầm identity NMCO1 với local user-ID0, và giới hạn generated traps/API name. Không chỉnh log cũ
hoặc completedCP017. Native111 primitive pass không chứng minh đúng handler; jump table bác bỏ pairing.

**verify.py:1.762 kiểm tra /7 run client pass về tính nhất quán**, cộng1 boot failure và1 interruptedbootstrap
được báo riêng. 5 modeled constructor fixtures,21 native fixture cases; expected trap controls vẫn là failed client run.
Manifest/verify.py kiểm raw/hash/isolation/native và các markers có thể đo; đọc timeline/code/ảnh
thủ công riêng. Không có review độc lập; CP018 complete chỉ nói record/docs đủ. AUD005 tạiCP016,
audit kếCP020; map/start/fullauth và hai-client gate tiếp tục mở.
