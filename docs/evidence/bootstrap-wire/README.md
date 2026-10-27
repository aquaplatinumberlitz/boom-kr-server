# Phản hồi ProcessPatch KR dưới Wine

Snapshot lịch sử27/10/2026 JST,trước R2. Trạng thái mới ở [README](../../../README.md),
contract hiện hành ở [PROTOCOL](../../PROTOCOL.md#wcp). KRE-002, baseline `fcf5dfb`; tiếp nối [recipe Wine](../wine/README.md).
**Bản copy đã nhận phản hồi bootstrap local và trả thành công từ `ProcessPatch`.** Đây chưa phải
hello game/login: client tiếp tục rồi dừng ở UD2 của import `ngclient.aes` ordinal1 chưa phục hồi.
Binary vẫn SHA `a0c3b9867dae5fe2ff7c315cfe851117661c2cafead45196ce999329f67432f5`;
không ép return của ProcessPatch, không sửa thêm client trong lượt này.

## Contract đã kiểm

Nguồn `kr_static`, target memory `modO.bin`, base `400000`, SHA
`a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5`.
Các kết luận dưới đây `confirmed` trong phạm vi native/emulation; không phải schema server gốc đầy đủ.

- `ProcessPatch1236810` tạo socket `115B840` và response `14ED440`, vtable `17B0D18`.
  Constructor response có selector2, nhưng `14EF300` gọi virtual8=`14EEFA0` với socket `+40/+44`;
  các giá trị được lấy từ globals `1A33920/24` bằng `14EF6C0`. Trong image này là `0/0`, nên
  **mode0/selector0**, không dùng selector2 mặc định để dựng frame bootstrap.
- `14EEB40` selector0: prefix1 + BE16 payload length ở offset1, tổng `length+3`.
  `14EDC20` bỏ header, chép body, đặt cursor0. Selector2 có header4 chỉ là ca kiểm riêng.
- `14EDF50` selector0 tiêu thụ **u8 guard**, ProcessPatch yêu cầu bằng0. Selector2 tiêu thụ
  **u16**, vì `14EF8F0(2)=2`; mô tả cũ trong raw dsh1-r4/r5 về selector2 đọc u8 là sai.
- Tiếp theo: `14ED750` đọc u16; lần hai đọc version u16; `14ED7E0` đọc u32;
  `14EDF90→14EE040` đọc u16 byte-length + chuỗi không NUL; `14ED6C0` đọc u8 cuối.
  Mode0 dùng BE16/BE32 và byte nguyên. Chuỗi kiểm đủ prefix2 và đủ length; lỗi đặt `+30=1`.
- Version guard `1237A30` loại `FFFF/FFFE`; `1237A60` so với `1238450()=967`.
  Version khớp đi nhánh `1236A62`, cleanup rồi return1; chưa đi các nhánh PreparePatch/launcher.
  Firstword và u8 cuối có consumer trong nhánh mismatch; không coi chúng là vô nghĩa ở mọi version.

`+AC` ở socket là **offset hex**, không phải tên anti-cheat. Base ctor khởi tạo0; nếu bật,
virtual48=`14EF200` kiểm prefix, khởi tạo seed rồi `14EDC20` có thể đổi byte guard.
Oracle thử cả bật/tắt: prefix `69` cho seed0 với các hằng socket của constructor. Giá trị prefix
này là **`emu_choice`**, không phải magic/constant server gốc. Không suy absence của module AC.

## Fixture probe và kết quả runtime

Frame S→C local, 15 byte:

```text
69 00 0c | 00 | 00 00 | 03 c7 | 00 00 00 00 | 00 00 | 00
prefix/len  guard firstword version  opaque        strlen  tail
```

Fixture SHA `75be47b3067e2b1e9d2cf99b5a7eb7f949a074482409fb3eafc1a553cec53cd1`.
Guard0/version967 theo consumer; firstword0/u32zero/empty string/tail0 là **`emu_choice`** cho
nhánh equal-version. Chưa khôi phục ý nghĩa/giá trị server gốc, cập nhật patch hoặc nội dung manifest.

Cùng binary, cùng template game/prefix copy, outbound bị cô lập với chỉ loopback, NSS files-only,
private `/run`/`/tmp`, routes IPv4 rỗng. Mỗi ca plain dùng game/prefix copy riêng; hai ca debugger
reuse prefix của ca reply và **không tính cold start**. Không có kết nối Nexon thật.

| Ca / raw root `local/research/bootstrap-runtime/` | Quan sát | Phạm vi |
|---|---|---|
| `silent/run` | Accept3848, zero C→S, CALog mới `RecvPatchInfo: Failed`, dialog Login Failed | Control silent, 15s |
| `reply/run` | Gửi đúng15 byte trên3868, zero C→S; tiếp tục tới trap2263878 | Plain, 15s; không lấy alive làm pass |
| `reply/debug-success` | Breakpoint1236A62, EAX1, EDX3C7 | Xác nhận nhánh version khớp, có debugger |
| `reply/debug-return` | **Breakpoint1234875 sau call ProcessPatch, EAX1**; sau đó trap2263878 | Xác nhận return thật, có debugger |
| `badguard/run` | Chỉ đổi payload guard0→1, accept3878; dialog Login Failed, không thấy trap2263878 trong12s | Negative control; không suy exit/return từ riêng title |
| `reply-final/run` | Final harness, accept3888, gửi fixture đúng; trap2263878 lặp lại | Plain copy riêng, 10s |

Trap `2263878` thuộc slot `16EA4C8`, excluded `ngclient.aes #1` theo patch report cũ; debug stack có
return `544EFD`, caller `12358EC→12314A7`. Đây là cạnh RE kế tại lúc run này;R2 về sau bỏ call init theo [evidence](../wine-r2/README.md).
Trong snapshot này chưa xác định ABI/return/side
effects để stub. D3DX load xuất hiện cả ca guard sai; **không dùng D3DX load chứng minh bootstrap pass**.
Chưa có C→S tiếp theo, UI đăng nhập/kênh/sảnh, Windows control tương đương hoặc demo chơi được.
Không nâng K1/K2 chỉ từ kết quả này; recipe reply chưa có ba cold start thành công tới hello tiếp theo.

## Tái lập và giới hạn tool

```bash
python3 tools/verify_bootstrap.py \
  --memory /home/admin/boom/repos/boomkr-re/out/modO.bin \
  --out local/research/codex/bootstrap-native-next.json

python3 tools/wine_probe.py \
  --game-dir local/research/bootstrap-runtime/reply-final/game \
  --prefix local/research/bootstrap-runtime/reply-final/prefix \
  --out local/research/bootstrap-runtime/reply-final/run-next \
  --exe CA-wine.exe --seconds 10 --bootstrap-local \
  --bootstrap-reply local/research/codex/bootstrap-reply-r1.bin
```

Oracle yêu cầu Unicorn đúng2.1.4 và output dưới repo `local/`; sai version/path bị từ chối.
File fixture tạo từ hex trên trong `local/`; raw path không có trong clone sạch. Output phải mới.
`--bootstrap-reply` chỉ nhận file local 1..65536 byte và bắt buộc `--bootstrap-local`; gửi **một lần
trên kết nối accept đầu tiên toàn run**, giữ socket để capture tối đa65536 byte C→S. Nếu send lỗi,
record lỗi và không tự gửi lại trên connection khác. Đây là harness thử nghiệm, chưa phải service.
Mặc định không có flag vẫn silent. Không bật relay trong các ca này.

Oracle `tools/verify_bootstrap.py`: **254 ca pass** = 224 length/copy +12 receive/field/string +18
fragment/coalesced. Chạy byte gốc trong Unicorn2.1.4; chỉ thay allocator `14F5790` bằng buffer cấp sẵn,
object/socket/SEH memory là input tổng hợp. Có guard byte quanh đích, width/endian, chuỗi SSO0/5/15
byte, available0..15 và hai frame gộp. Không test allocation failure, string heap>15, mọi mode/seq,
full ProcessPatch hoặc toàn bộ wire. Một-byte body selector2 chỉ kiểm copy, không phải packet hợp lệ.
Metadata và digest raw: [verification.json](verification.json). Binary/decompile/capture vẫn local.

Sửa sau review: listener setup nằm trong cleanup/finally; lỗi bind được ghi `run.json`, đóng các
socket đã mở và không gán CALog cũ cho run chưa launch. Fault-injection ở listener thứ hai đã kiểm
đường này trước khi khởi động client. Native tool từ chối version khác/nonlocal output.
`reply-final/final-smoke` dùng harness sau sửa, reuse prefix, gửi đúng fixture và lặp trap2263878;
không tính thêm cold start. Hash ở trường `post_review_smoke`/`review_fix_checks`.
