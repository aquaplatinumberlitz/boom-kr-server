# REA tĩnh NewPatcher — đầu mối bootstrap KR 00967

Ngày 26/10/2026. Nguồn `kr_static`; mục tiêu trả lời có đường native nào để bắt đầu KRE bootstrap.
Kết luận consumer dưới đây `confirmed` trong phạm vi decompile; **chưa quan sát runtime** và chưa
giải nguyên nhân CA không đi tiếp qua `ProcessPatch`. Lộ trình ở
[REWRITE_PLAN — KRE](../../PLAN.md),
định danh/contract ở [LEGACY_KR_PROTOCOL](../../PROTOCOL.md).

## Target và phương pháp

- Input `/home/admin/boom/clients/kr-00967/NewPatcher.exe`, PE x86,
  SHA-256 `b59e4dce9ad23c439a7632ce04ecb2eed3481585bd2454100b05c9d7527f5c02`.
- REA `rea-agents` 6.7.0; provider `ida`, adapter1, headless private workspace;
  image base `0x00400000`. IDA engine/upstream distribution version không được provider xác nhận.
- `open_binary` input gốc với `provider_id=ida`; provider tạo bản riêng trong cache, không ghi DB gốc.
  `search_strings` lấy inventory IDA; xref literal `files.nfo2`; decompile procedure chứa `0x411AF0`.
  Không chạy CA/NewPatcher, không gọi URL trong strings và không sửa binary/pack/database nguồn.
- Đã export 8 evidence record rồi `close_binary`. JSON metadata bổ sung:
  [observations.json](observations.json). Không commit raw decompile hoặc binary.

## Quan sát và cách dùng

| ID | Phép đọc / kết quả | Giới hạn |
|---|---|---|
| `ev_15dd5285cf5cf8036c440e12b683423e37ac39fe452086a1a3a943d30b63b12f` | `files.nfo2` tại VA `0x4552C4`, chuỗi đường dẫn tại `0x4554DC` | Literal tồn tại không chứng minh endpoint runtime |
| `ev_16adea90b5fd24a2ae6d95fb788a1d3753e8dc90da59a5c06e63019acd6b5587` | Xref trả `0x411AF0`, `0x411D1D`, `0x414D6B` | Adapter không báo đầy đủ typed/body references; không tuyên bố call graph hoàn chỉnh |
| `ev_323b4d2fbc72067a30a2e6a59ca0eb523597742f58fa8d20a42fe21ba4913d0d` | Procedure chứa `0x411AF0` được IDA định danh `sub_4118E0`. Nhánh type3 so `files.nfo`/`files.nfo2`, đối xử khác nhánh thêm `.gz`; nhánh type4/5/6 có đường delta. Hàm có tạo/ghi file, đường kết nối/đọc mạng qua interface, báo lỗi và return thành công/thất bại | Chỉ xác nhận consumer này. Chưa truy caller/config/argument và chưa chứng minh nó nằm trên đường lỗi bootstrap đã quan sát ở run KR cũ |
| `ev_223f7f23db499653c5255cb80c2aab32436161f69f92b832426d95c9c6a52e24` | Inventory có `NexonHttpClient` và tên API HTTP cùng URL support/setup | Không fetch URL; không gán URL support thành patch endpoint |
| `ev_8ec447b8bc2694350c092bb070b612391613d1424d56b324ad693a4baad866ba` | Inventory có tên `CreateProcessA` tại `0x4641C0` | Chưa thấy caller/arguments; không chứng minh process tree hoặc contract IPC |

Tìm literal `ProcessPatch` và `CA.exe` không có kết quả **trong inventory strings của NewPatcher
ở profile này** (`ev_146faa0cbdd33a0d0ddeeaeed08e15dd7984b11c76a5fbd397c57c1b9b995302`,
`ev_80b0c308c70de9368da08b2bf66084a9f3818d52727cae9c31fa7e4e1a9282b6`). Không chứng minh không
có quan hệ với CA, không có chuỗi mã hóa/packed hoặc không có ProcessPatch trong **CA.exe**.
Search `NGM` chỉ khớp hai tên RTTI có substring `StringMgr`; không dùng làm bằng chứng NGM contract.

**Suy luận / candidate:** bắt đầu ở caller/config của manifest consumer là cạnh khả thi cho KRE-002,
có thể giảm phụ thuộc vào hook runtime vốn kích hoạt anti-tamper trong các run cũ. **Unknown:**
input bootstrap thực, IPC/token/exit status, yêu cầu file/service, logic hash NFO và điểm success CA chờ.

## Chứng cứ lịch sử và điều cần sửa khi đọc lại

`/home/admin/boom/repos/boomkr-re/docs/kr.md` là notebook nối tiếp; §18/§22 về dump và §31/§33 về
process/dialog muộn hơn các kết luận đầu file. §31 ghi CA còn sống 45 thread/~75 MB, snapshot netstat
không có kết nối; log `EndOfInit! exitcode=1` được nhận diện là của run trước. Run từ share và bản
local tối thiểu 48 MB cũng không phải cùng control với client đầy đủ; chưa tái xác nhận trạng thái
hiện tại. §33 ghi **hai cửa sổ**: Nexon thuộc NewPatcher, `??? ??` thuộc CA. Không gộp ownership
hoặc kết luận quan hệ bootstrap từ title. §12 quét bảng VM Themida, không chứng minh không có dispatcher mạng.
Dump `.text`/private region đã cho phép RE nhiều consumer KR; entropy `.themida` không chứng minh
client phải hoàn tất patcher trước khi đọc được mọi code. Hồ sơ cũ chưa chứng minh qua bootstrap.

Tại lúc lập plan: docs `boom-rs@dae5e1c7`, legacy `boom-re@9ac2075b`, `boomkr-re@88e11ed3`.
Đây là đọc lại bằng chứng lịch sử, không tái chạy thí nghiệm ETW/WinBoat/Wine của notebook.

## Raw local và điều phối

Raw export local: `/home/admin/boom/repos/boom-kr-server/local/plan-origin/rea-newpatcher-evidence.json`;
digest/size nằm trong observations. Raw chỉ giữ local, repo công khai giữ metadata.
REA IDA cho biết thiếu complete body ranges/typed references/unresolved indirect-call coverage;
decompile là kết quả công cụ, không phải source nguyên bản hoặc trace execution.

Đã xác minh durable title của session `dsh1`=`session-206fda5f-ee85-4f9c-87b8-598cd94b10e3`,
`dsh2`=`session-1506e63f-05d8-46f8-8919-58e65556cbf9`. Hai brief nằm trong cùng thư mục raw local.
Kênh web gửi localhost ban đầu bị sandbox host chặn `socket()`; nâng quyền cũng bị policy từ chối.
Sau khi chủ dự án mở quyền, `session/prompt` mode `queue` gửi vào đúng hai session đều trả
`accepted: true`. Các phiên headless mới đã thử nhầm không được dùng làm
thay thế; chúng kết thúc vì thiếu credential. Không nhập credential hoặc settings cá nhân vào repo.
Bằng chứng dương trong tài liệu này do phiên điều phối đọc trực tiếp bằng REA.
Lỗi cwd của session web sau đó được khắc phục bằng alias local `/home/admin/boom-rs-dsh` trỏ tới
`/home/admin/boom/work/dsh`; hai session đã thực thi lệnh đọc tài liệu. Không sửa session store hoặc
thay bằng phiên mới. Review bootstrap vòng đầu của dsh1 yêu cầu sửa; điều phối đối chiếu nguồn và
sửa K0/K1, giữ các quan sát lịch sử theo từng run thay vì gán thành runtime mới. Review vòng hai:

| Reviewer | Nội dung được kiểm | Commit / kết luận |
|---|---|---|
| dsh1 | Bootstrap/launcher/ProcessPatch, K0/K1, control và quyền patch offline | `5e9ebb64dae249ee5e3328790e9ab7b5de2a4a07` — **APPROVE** |
| dsh2 | Reuse VN→KR, codec/socket/cache, authority/P2P và gate | `5e9ebb64dae249ee5e3328790e9ab7b5de2a4a07` — **APPROVE** |

Báo cáo local `dsh1-report.md`/`dsh2-report.md` trong thư mục raw cùng run; không copy báo cáo phiên
thành nguồn kế hoạch mới. Bổ sung control NewPatcher đơn lẻ và làm rõ ba cold start mỗi recipe khi
chốt tài liệu. Không có runtime KR mới hoặc wire/feature nào được nghiệm thu bởi review này.
