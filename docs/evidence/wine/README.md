# KR unpack/patch dưới Wine

Yêu cầu chủ dự án 26/10/2026: ưu tiên chạy bản copy KR bằng Wine. Recipe thực nghiệm
**`kre-wine-copy-r1`** đã dựng PE mới từ memory image `modO.bin`, khởi động CRT/WinMain và
đi tới `ProcessPatch`. Đây là **modified client**, chưa vào màn đăng nhập/kênh/sảnh hoặc gameplay.

## Artifact và giới hạn

| Đầu vào/đầu ra | SHA-256 |
|---|---|
| `modO.bin` — memory layout, base400000 | `a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5` |
| `kr-full.exe` — chỉ lấy PE header, gồm directory metadata TLS/resource | `cf72da112308d9f1ca256df8089594c5f787875310933073bf6bd5ba6f70e63b` |
| Candidate import metadata, private `wine-imports-r4.json` | `e5a618a3d1f6c8b29a5895f5354519a7a904a65a02c44abfb82449c2796aaf74` |
| Bản copy chạy, `CA-wine.exe` | `a0c3b9867dae5fe2ff7c315cfe851117661c2cafead45196ce999329f67432f5` |

Candidate metadata được tạo bằng Unicorn từ `modO` + wrapper/export trong `windumpL`; có
489/558 slot có tên khả dĩ. Nhãn header cũ trong JSON vẫn nói `kr-full/P`; đó là nhãn provenance
lỗi của thí nghiệm, **không dùng làm nguồn runtime**. Các row thực dùng pointer của `modO/L`.
Builder khóa toàn bộ SHA của file metadata, source memory và header; hash output phải khớp recipe
đã đo. File này là đầu vào host-local, cùng binary/raw không được commit; clone mới cần cung cấp
đúng các artifact, không thể build chỉ từ Git.

`kr-full.exe` không phải executable hoàn chỉnh: entry vẫn ở packer, import PE chỉ có hai mục và
section đầu chồng `.priv`. Không chạy dump này bằng cách đổi tên thành `CA.exe`.
Memory `modO` và IAT `kr-full` thuộc run khác nhau; không trộn pointer.

## Thay đổi trên bản copy

- PE hai section không chồng: memory image + vùng sửa import; CRT entry VA`1470E40` → WinMain
  `1253B10`. Giữ TLS template/index directory; bỏ packer entry, relocation/ASLR và các directory
  không dùng. Section thực nghiệm RWX; chưa phải bản phát hành được harden.
- Khởi tạo lại **các cache có consumer xác định**: startup guard, onexit, DLL/API handles, locale,
  facet, lowio descriptor blocks, string SSO và cờ Winsock. Giữ cookie/complement; không zero cả data.
- 449 import được Windows loader resolve; xoá bound pointer cũ trong IAT, giữ ILT riêng theo tên API, 105 slot chưa giải/thuộc DLL bị loại có UD2 riêng để
  báo đúng slot. Phần ngoài đường chạy đã đo vẫn `candidate`, không coi là IAT đã phục hồi toàn bộ.
- **`emu_choice`:** bỏ analytics load/send/shutdown và bốn API CrashReporter. Giữ parser `572600`;
  lookup property analytics trả unavailable. Chưa kiểm identity/session khi có launcher arguments.
  Cụ thể, output assignment `nxgsmsid` được giữ nhưng nhánh `sessionid` cần analytics init thành công
  hiện không thể tới lookup; cần phục hồi riêng trước K3. Giữ byte parser không bảo đảm session còn hoạt động.
  Không bỏ cả parser, không áp stdcall `ret N` cho wrapper cdecl.
- Không patch nhánh trả thành công của `ProcessPatch`, không gửi hello giả. Binary cuối không có
  trampoline đánh dấu constructor/call dùng trong các thí nghiệm r11–r29.

TLS/resource **content** lấy từ memory image `modO`; header `kr-full` chỉ cung cấp directory metadata.

API module nội bộ là bẫy quan trọng: resolver từng thấy `GetProcAddressForCaller`,
`RtlInitializeCriticalSectionEx`, `RtlSwitchedVVI`, `NtUserMapVirtualKeyEx`; call site thật yêu cầu
`GetProcAddress`, `InitializeCriticalSection`, `VerifyVersionInfoW`, `MapVirtualKeyA`. Số tham số
khác làm lệch ESP và có thể tạo lỗi ở constructor hoàn toàn không liên quan. Các override khác và
toàn bộ before/after/VA nằm trong báo cáo local do builder sinh.

## Chạy lại trên host này

Dependency RE tùy chọn: `pefile==2024.8.26`. Runtime: Wine CachyOS10.0-20260425, Xvfb,
util-linux `unshare`, iproute2, xwininfo, ImageMagick, procps/getent. Không tự cài dependency hoặc
đổi prefix VN.

Tại root repo, dùng thư mục output mới:

```bash
python3 tools/build_wine_copy.py \
  --memory /home/admin/boom/repos/boomkr-re/out/modO.bin \
  --headers /home/admin/boom/repos/boomkr-re/out/kr-full.exe \
  --candidates local/research/wine-imports-r4.json \
  --out local/research/wine-unpacked/game/CA-wine-next.exe \
  --report local/research/wine-unpacked/rebuild-next.json

python3 tools/wine_probe.py \
  --game-dir local/research/wine-verified/cold-1/game \
  --prefix local/research/wine-verified/cold-1/prefix \
  --out local/research/wine-verified/cold-1/run-next \
  --exe CA-wine.exe --seconds 10 --bootstrap-local
```

Các thư mục `game` đã copy đầy đủ client KR và binary mới; prefix riêng. Với máy khác phải chuẩn
bị các copy tương đương trước. Builder/probe từ chối output cũ và đường runtime ngoài `local/`.
Rollback: ngừng đúng run và bỏ bản copy; source client/DB không thay đổi.

Probe dùng user/net/mount/PID namespace, chỉ có loopback, route IPv4 rỗng; `/tmp` và `/run` riêng,
NSS `hosts: files`, hai hostname bootstrap →127.0.0.1 được kiểm bằng `getent`. `/etc` trên host không
bị đổi. Chỉ tách network namespace **chưa đủ** để ngăn NSS gọi systemd-resolved qua UNIX socket;
các run trước khi có files-only NSS không được dùng làm chứng cứ DNS offline hoàn chỉnh.

Preset listener3808..3998 bước10 là **`emu_choice`** để phủ các cổng đo được, chưa phải registry cổng
KR gốc. Listener chỉ accept/ghi byte, không trả lời; không có giả thuyết schema được gửi cho client.
`run.json`, log Winsock, fresh CALog, process tree, cửa sổ và ảnh đều trong output riêng. Capture TCP
không gắn PID bằng ETW; attribution dựa namespace riêng, log connect của client và accept tương ứng.
Cleanup bằng wineserver của prefix riêng và PID namespace, không kill Wine VN.
`--trace-api` chỉ dành cho thăm dò có thời hạn: relay log chưa có giới hạn byte, có thể tăng nhanh.
Recipe ba cold start không bật relay. Probe không phải sandbox filesystem tổng quát; binary chỉ chạy
từ copy và vẫn có quyền file của user trong namespace. File command của winedbg cũng phải ở `local/`.

## Kết luận được phép

`kr_runtime/confirmed` cho **bản copy đúng SHA trên Wine này**: CRT/WinMain → `InitBeforePatch` →
`ProcessPatch`, Winsock1.1 init và connect loopback được listener accept. Khi listener im lặng,
client không gửi byte application và timeout `RecvPatchInfo: Failed`; đây là control server-first
candidate, chưa chứng minh full frame/schema hoặc login. Metadata run/check nằm ở
[verification.json](verification.json).

Không đánh dấu K2/login/gameplay hoàn thành. Gate K1 toàn project vẫn cần review contract/bootstrap
và đối chứng Windows; bằng chứng này đóng đường tự động hóa Wine cho thí nghiệm bootstrap local.

Recipe riêng tiếp theo [post-bootstrap-r2](../wine-r2/README.md) bỏ một call NGClient init và phục hồi
sáu import mới và sửa một A/W import trên copy; đã tạo cửa sổ và attempt local thứ hai, còn WCP/Login Failed.
R1 trong tài liệu này giữ binary SHA/recipe cũ; không tính kết quả R2 cho r1.

Snapshot r1/r2 silent không phải trạng thái mới nhất;xem [WCP](../wcp/README.md) và [contract](../../PROTOCOL.md#wcp).
