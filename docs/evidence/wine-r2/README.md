# Wine sau ProcessPatch — recipe post-bootstrap-r2

27/10/2026 JST, issue9; baseline e633736 ([bootstrap reply](../bootstrap-wire/README.md)).
Recipe riêng post-bootstrap-r2 phục hồi thêm đường tạo cửa sổ của bản copy KR.
**Đã thấy cửa sổ Crazy Arcade800×600 và kết nối TCP loopback thứ hai**, sau đó dialog Login Failed
với CALog [XRpt] NTFC - WCP(failed)... trong ca15s; hai ca10s còn chờ. Chưa có UI đăng nhập hoạt động, hello game hoặc demo.
K0/K1/K2 còn các điều kiện chưa nghiệm thu.

## Thay đổi có nguồn

Input modO/header/candidate JSON giữ SHA của [recipe r1](../wine/README.md).
R1 mặc định vẫn tạo a0c3b9867dae5fe2ff7c315cfe851117661c2cafead45196ce999329f67432f5.
R2 SHA **7b3b435b5d8e110185359107376b736615ceabe9452cea8154d02c8628f44c29**,
455 imports,99 missing/excluded UD2 traps,4 CrashReporter stubs; chưa phục hồi toàn IAT.

| Thay đổi R2 | Nguồn/locator | Trạng thái và giới hạn |
|---|---|---|
| NOP5 byte12358E7, trước E8 E4 F3 30 FF, bỏ call544CD0 | kr_static: caller12358E0 vẫn gọi545E40;544CD0 gọi ngclient ordinal1 với this+90, return bị bỏ | emu_choice: bỏ NGClient init offline; không thay ABI ordinal1 hoặc giả lập callback |
| slot16EA160 →kernel32.lstrcpyA | Bound6790000:4 synthetic strings copy NUL, trả dest,stdcall ret8; caller14A8362 cấp strlen+1 rồi copy FxCharacter | kr_static confirmed trong ca kiểm; public-name equivalence candidate ngoài ca kiểm; invalid/overlap chưa kiểm |
| slot16EA664 →user32.SetWindowsHookExA | Bound6DD0ADB truyền4 args +internal flag2 tới77485380; public A7746AE70 flag2,W774809D0 flag0; call13860E0 có4 args | kr_static confirmed ABI/A; không dùng hàm nội bộ SetWindowsHookExAW |
| slot16EA6C4 →user32.LoadIconA | Bound7B208D1,call125BC4F/5E integer68; ECX=hInstance,EDX68,args[3,0,0,8040] | kr_static confirmed integer path; string probe bound/A cùng dừng unmapped TEB, không coi conversion pass |
| slot16EA5EC →user32.LoadCursorA | Bound6D609EC; call125BC6x hInstance0,integer7F00; args[1,0,0,8040],public ret8 | kr_static confirmed integer/ABI; A-name candidate từ string-route, không khẳng định full encoding |
| slot16EA5AC →user32.RegisterClassExA | Bound6B20D25 với WNDCLASSEX48 đi774621FB,ECXstruct,EDX0,args[0,80,1]; A cùng route,W đi7746FAFD | kr_static confirmed1 arg; atom/OS behavior chỉ trong runtime đã đo |
| slot16EA630 →user32.AdjustWindowRect | Bound6D9078C với RECT +synthetic TEB đi7746DE4C,ECXrect,EDXstyle,args[menu,0]; callerCCE370 có3 args | kr_static confirmed route với context tổng hợp; không phải Ex4 args |
| slot16EA5C0 →user32.DefWindowProcA | Bound6B30D2E nhảy77463910; public A qua NTDLL77C57F10/[77CF9020] cũng77463910; W dùng774761C0 | kr_static confirmed target/4 args; phục hồi callback, không ép success |

545E40 là singleton accessor, không phải fresh allocation mỗi lần. Ctor545260 đặt +90=0 chỉ ở nhánh
khởi tạo. Debug cursor-1 tại12358E7 thấy ECX/EAX19D89B8,[19D8A48]=0,guard[19D8A4C]=80000009.
Giữ guard; không reset receiver. Cleanup544F10/consumer544F40 kiểm null+90. Chưa biết mọi consumer khác
hoặc lần gọi sau chịu được NGClient vắng mặt; không coi recipe tương đương client nguyên bản.

## Kiểm thực nghiệm

Mỗi bước giữ reply15 byte đã kiểm, đổi một API trên binary kế tiếp; runtime cô lập outbound,
loopback-only,NSS files-only,private/run/tmp. Raw local/research/wine-r2/{cold-1,icon-1,cursor-1,class-1,rect-1}
cho thấy trap lần lượt16EA6C4→16EA5EC→16EA5AC→16EA630→16EA5C0. Lỗi thứ cấp16EA2A4 khi unwind ở
rect-1 chưa phục hồi; không dùng cleanup fault làm root cause. Sau DefWindowProcA, plain run đi tới
cửa sổ và kết nối thứ hai.

Ba cold start cùng R2 cuối: defproc-1/run,final-cold-2/run,final-cold-3/run. Game/prefix copy riêng
từ cùng template đã khởi tạo; không phải ba cài Wine mới. Debug state reuse cursor-1 prefix, không
tính cold start. Không lấy alive/absence of UD2 làm success login; mốc chỉ cửa sổ800×600 và attempt
local thứ hai đã ghi. Hash/time ở [verification.json](verification.json).

Các run snapshot R2 này gửi fixture một lần lên socket đầu;socket hai silent/zero C→S.
Đây không còn là thí nghiệm mới nhất:xem [probe WCP](../wcp/README.md).
Contract hiện hành ở [PROTOCOL](../../PROTOCOL.md#wcp);không gán kết quả hai-reply cho R2.
D3DX load có cả ở negative bootstrap trước đó,không dùng làm proof.

Hai private run bootstrap-runtime/ngskip/run và ngskip/debug overlap prefix~1.16s: **loại khỏi
đối chứng**. ngskip-recheck/run bản copy/prefix riêng thay control NGskip. Các run R2 trên đây tuần tự.
Pointer/handle process cũ và import chưa dùng có thể chặn bước sau; recipe thực nghiệm, không phát hành binary.

## Tái lập

Dùng tools/build_wine_copy.py --recipe post-bootstrap-r2 với ba input pinned như r1, chọn --out và
--report mới dưới local/. Output phải khớp SHA R2 trên. Đặt exe trong full game copy, chạy
tools/wine_probe.py với game/prefix/out riêng, --exe CA-r2.exe --seconds 10 --bootstrap-local
--bootstrap-reply local/research/codex/bootstrap-reply-r1.bin. Lệnh đầy đủ cùng yêu cầu namespace ở
[bootstrap harness](../bootstrap-wire/README.md). Raw/binary/asset không nằm trong Git.

Rollback chọn --recipe bootstrap-r1 hoặc bỏ flag; không sửa source hoặc ghi đè binary cũ.
Verification cần rebuild độc lập đúng SHA,PE import count/API/patch exact,default-r1 regression,
source hashes,docs/diff check và reviewer khác tác giả đúng commit. CI chưa cấu hình.

Metadata connections giữ ordinal,listen_port,peer và timestamp; final-cold-2 có hai connection
trên cùng listener3908, không phải hai port khác nhau. Report R2 dùng schema kre-wine-copy-r2,
R1 vẫn schema cũ và output SHA cũ.

<a id="cp179"></a>
## CP179 — đối chiếu lại cold3 bằng API/PID, không chạy lại

Baseline CP178/PR250 merge`2f2b87f`; Codex solo/tự kiểm, independent review waived.
Snapshot27/10 bên trên giữ lịch sử; [nghiệm thu hiện hành](../../PLAN.md#k1-acceptance) và
[contract/giới hạn](../../PROTOCOL.md#bootstrap-recipe-scope) là nguồn chuẩn, không sửa raw/record cũ.
Raw mới `local/research/codex/k1-cold-recipe-r1/`: `source-first-reuse.json`, `verify_reuse.py`,
`reuse-verification.json` (68 quan hệ), `bootstrap-region-control-comparison.json` (44 control pins),
`manual-k1-acceptance.json`. Hash đầy đủ ở [record CP179](../checkpoints/records/CP179.json) qua
[manifest evidence](../checkpoints/evidence/CP179-k1-reuse.json); checker không tự quyết định gate.

| Run lịch sử | WinePID từ init_peb và connect | Endpoint hai lần / giới hạn |
|---|---|---|
| defproc-1 | 01a0 | 3888→3898; quan sát15s, về sauWCPtimeout |
| final-cold-2 | 01ac | 3908→3908; quan sát10s, hai connection cùngport |
| final-cold-3 | 0174 | 3898→3878; quan sát10s |

Ba commandline khớp sau bỏ riêng copy/outputpaths và observationduration; milestone có trong10s
đầu cảba. Nativeconnect không chỉ được suy từ TCPaccept; nonblockingreturn không bị gọi success.
LauncherPID và actualCA PID khác nhau; không dùng launcher để quy socket. Reply socket1 trước
attempt socket2 theo listener clock/order; không đồng nhất hai nguồn timestamp. Cleanup theo
harnessfinally/prefix/children và endmetadata, không kiểm lại bằng killWine toànhost.

Read-only PE range/hash đối chiếu R1/R2/e9 và patchreports; không chạy tooldecompile/scans mới,
không dựng lại toàn bộ chuỗi import hoặc lặp3coldstarts/gameplay. Giữ e9 extra patches/state và
R1 native return/control riêng, không nâng thành vanilla/bootstrap/fullK2/auth/LAN/8client.
CP179 gom reuse+manualacceptance+currentdocs/#1closed delivery; periodic audit vẫn kếCP180.
