# CP010 — object cũ, exception input hook và import runtime

Codex RE ngày 27/10/2026, [issue14](https://github.com/aquaplatinumberlitz/boom-kr-server/issues/14).
CP009 đã review/merge PR18. CP010 là thí nghiệm copy, review snapshot và SHA của raw/recipe ở
[manifest](../checkpoints/manifest.json). Counter/audit tiếp theo ở cùng manifest; không nâng gate.
Contract mạng giữ ở [PROTOCOL](../../PROTOCOL.md#wcp), câu hỏi kế ở [RESEARCH](../../RESEARCH.md#q-023).

Mọi run dùng Wine 10, namespace user/net/mount/PID, Xvfb, loopback và NSS files-only. Mỗi run có
prefix riêng; timing/prefix và một số API binding khác nhau, không phải đối chứng vanilla hay
phép kiểm nhân quả tuyệt đối. WCP/8 response, identity và các patch cũ kế thừa CP006–009.
Source binary/DB giữ nguyên; các copy và raw ở `local/research/codex/object-state-r1/`.

## Các phép đo và delta

| Nhóm run / nguồn | Quan sát và giới hạn |
|---|---|
| `tree-guard`, `tree-probe`; kr_static + kr_runtime | Image unpack đã có pointer `02677C98` tại `19D8CFC`, cạnh guard `19D8D0C=80000006`. Native `574480` có nhánh gọi ctor `5739A0` cho object `19D8CF8`. Copy reset riêng guard về0 đi qua fault cũ và gặp trap khác. Snapshot sau đó thấy pointer mới `09A57790`, epoch `80000010`; confirmed trong run copy, chưa chứng minh toàn bộ object/heap đúng. |
| Recorder trực tiếp; emu_choice + kr_runtime | Hai phép attach WineDbg trước đó gặp `FFF40200` ở thread khác. Copy recorder lưu registers/flags, stack và globals bằng file API, rồi tiếp tục instruction hoặc giữ trap UD2. Không dùng stack attach đó giải thích fault chính. Recorder thay timing, dùng buffer hữu hạn; file giữ lần ghi cuối, không phải lịch sử mọi invocation. |
| `tree-probe`, `deletecs`, `deletecs-probe`; kr_runtime + kr_static | Trap `16EA2A4` có return `14F1280`, argument `this+0C`, phù hợp nhánh destructor critical section; ctor dùng `InitializeCriticalSection`. Thử `DeleteCriticalSection` đưa run tới `16EA69C`. Snapshot slot này có return `1258D1D`, args `3B,8,pointer,0`; static có thêm action3A, thử `SystemParametersInfoA` đi tiếp. Tên API vẫn candidate của copy; không tự khẳng định import nguyên bản. |
| `dwk-big`, `throw-probe`; kr_runtime + kr_static | EBP chain tại trap `UninstallDWK` đi qua destructor `122EA40`; trace có C++ exception trước cleanup. Pre-throw snapshot tại `1525FEB` có return `1104B86`, throwinfo `181F760`, class RTTI `CAppException`, error globals0/0. Nhánh tĩnh ở `1104B10` kiểm `GetModuleHandleA("CAWH.sys")` và ném với literal “Cannot find hooking dll”. Đây là nhánh đã chạy trong copy này, không phải chứng minh mọi lỗi CP009 có cùng nguyên nhân. |
| `no-inputhook`, `nohook-throw`, `no-inputhooks`; emu_choice + kr_runtime | Bỏ một call `10FC3F4` vẫn ném từ cùng site, stack mới có caller `10FAE11`. Copy thử `ret` tại entry `1104B10` (thiscall không có stack argument, body cài hooks) chưa thấy C++ exception đó trong khoảng đo và đi tới import khác. Bàn phím/chuột, mọi caller và side effect chưa kiểm. |
| `globalhandles`, `clipboard`, `key-state`; kr_static + kr_runtime | Thử `GlobalLock/GlobalUnlock` theo chuỗi `GlobalAlloc`, rồi sửa slot `16EA5FC` từ candidate đã bind `RtlEnterCriticalSection` sang `SetClipboardData`. Xref truyền handle cùng format1/13; copy trước fault đọc `0x15` trong Wine, copy sau tới trap khác. Các đường chưa chạy vẫn chưa kiểm. Slot `16EA6A0` có dạng VK/high-bit; thử `GetAsyncKeyState` nhưng chưa phân biệt hoàn toàn với `GetKeyState`. |

## Trạng thái cuối nhóm

`clipboard` và `key-state` có marker `EndOfInit! exitcode=0` rồi log tạo general packet type15.
Stub vẫn chỉ thu66 byte C→S cũ; log builder chưa chứng minh packet15 được gửi. Không suy marker
thành login hoặc nghiệm thu khởi tạo. Run `key-state` cuối cùng gặp UD2 `22638CC`, slot `16EA4E8`,
ứng viên `nxoverlay.dll` ordinal9 theo import report. Chưa có login, sảnh, trận hay K2/K3.

Các log game có type744/745 chỉ là log tạo packet trong copy, không dùng làm proof wire.
`no-inputhook` và `nohook-throw` có khoảng chạy chồng nhau với chung game-dir; loại CALog của hai
run này khỏi suy luận. Trace process, snapshot recorder và TCP namespace riêng là nguồn được dùng.
Các run tiếp theo chạy tuần tự. Những raw trước vẫn giữ nguyên, không sửa để khớp kết luận.

## Recipe và kiểm

Các script private được hash trong manifest: `patch-tree-guard.py`, `bind-import*.py`,
`build-stack-probe*.py`, `build-throw-probe.py`, `run_probe.py` và harness `wine_probe_stack_record.py`.
Metadata mỗi patch giữ source/target SHA, byte trước/sau hoặc diff, VA/RVA, rollback và giới hạn.
Recorder trap lưu header32 + registers36 + stack512/8192 + globals64; loại throw replay đúng6 byte
prologue rồi nhảy lại, có giới hạn stack và timing. Buffer/import/file API chỉ thuộc copy.
Mỗi run có recipe argv và `run.json`; không phát hành binary/asset hoặc raw packet.

`verify.py` đối chiếu exact byte của patch cố định và import, hash copy, snapshot, isolation và các
quan sát log hữu hạn. `patch-diffs.json` lưu diff và SHA phần append của recorder; ngữ nghĩa stub
cần review thủ công, checker không tự xác nhận. Năm artifact nguồn được kiểm lại. Xem
[verification](verification.json) cho kết quả kiểm; chưa review thì vẫn pending trong manifest.
