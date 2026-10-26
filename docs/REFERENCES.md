# Nguồn RE và migration

## Project mới

Theo chỉ đạo 26/10/2026, nguồn chuẩn cho server tương thích KR là repo **boom-kr-server**.
Code ở `server/`, tool ở `tools/`, docs ở `docs/`, raw/copy/DB ở `local/`.
`boom-rs` là project viết lại client/server khác; repo này không có dependency scope/progress V1 của nó.

Plan/contract đã được dsh1 và dsh2 review đến commit `1eee46d0ecf5c7f4eeedeadd7e258ef1cd64bf43`
trên nhánh local `docs/kr-original-client-emulator` của boom-rs trước khi tách. Bản import ở đây cần
review đường dẫn/scope sau migration; việc import không xác nhận runtime KR.

## Các nguồn chỉ đọc

| Nguồn | Vị trí / baseline | Dùng cho |
|---|---|---|
| Client KR gốc | `/home/admin/boom/clients/kr-00967/CA.exe`, SHA `a52545c14b1fea423de655c3163d6360b628d61c1cd472b46fdbc8a9ffe15f4e` | Target server tương thích; không sửa nguồn |
| Client VN gốc | `/home/admin/boom/clients/vn-116/ca.exe`, SHA `bce8644d4e783d1354ab4fc880e1ab9a4ab3325327028f37d99e5439e25f4bbd` | Định danh VN; khác artifact patched dùng cho runtime |
| RE/server VN | `/home/admin/boom/repos/boom-re`, commit `9ac2075bbec0c16217741aaedea968cf685e39a9` | `docs/protocol.md`, `oc_d-login.md`, `lobby.md`, `oc_e-game.md`; `tools/boomproto.py`, `verify_emu.py`, `fakeserver.py`; `server/src` làm reference |
| RE KR cũ | `/home/admin/boom/repos/boomkr-re`, commit `88e11ed30764c753a00f4f23a2f33dafadc4c890` | `docs/kr.md` §18–33, dump/native consumer, `out/kr-full.exe` và sourceDB |
| Nghiên cứu dùng chung | `/home/admin/boom/repos/boom-rs`, docs baseline `dae5e1c717ad31b9e8d32a9b20a76d65942807eb` | `docs/research/findings/{VN_LEGACY_SERVER_REFERENCE,VN_BATTLE_RUNTIME_REFERENCE,KR_RE_FLAME_CHAIN,KR_RE_TRAP_TIMER,KR_RE_TRAPPED_SPEED,KR_RE_BOMBERATTR}.md`; ADR0002/0013 cho lịch sử nguồn |
| Archive | `/home/admin/boom/archive/2026-10-26` | RESUME/KNOWLEDGE/TOOLS/PROCESS là hồ sơ lịch sử; kết luận mới giữ trong repo này |

`kr.md` là notebook nối tiếp: §31 sửa nhận định exitcode cũ thành một run alive-stall; §33 tách dialog
CA với NewPatcher. Không biến kết quả một run cũ thành trạng thái hiện tại; quét bảng VM không đồng
nghĩa đã quét dispatcher mạng. VN và KR chung engine không chứng minh wire giống nhau.

Các tham chiếu legacy protocol/ADR/RULES trong bản import dẫn về danh mục này; tra đúng tài liệu
nguồn theo bảng. Giá trị/claim dùng trong code mới phải có record riêng gắn source+locator rõ.

## Bản đồ chuyển

| Trước | Nguồn chuẩn mới |
|---|---|
| `boom-rs` worktree `kr-emulator-plan/docs/planning/REWRITE_PLAN.md`, chỉ mục KRE | [PLAN](PLAN.md) |
| `docs/specification/networking/LEGACY_KR_PROTOCOL.md` trong worktree đó | [PROTOCOL](PROTOCOL.md) |
| `docs/research/QUESTIONS.md`, chỉ Q-022…029 | [RESEARCH](RESEARCH.md) |
| `docs/evidence/measurements/kr-compat-bootstrap-2026-10-26/` | [Bootstrap evidence](evidence/bootstrap/README.md) |
| `work/team/tmp/kr-emulator-plan-20261026` | `local/plan-origin` (alias tương thích cũ giữ cho report/path lịch sử) |
| `work/team/tmp/kre-implementation-20261026` | `local/research` (alias giữ để worker đang chạy không mất output) |
| Worktree code tạm `work/kre-bootstrap-harness` của boomkr-re | Không có code mới; dọn worktree sau khi chốt import; tool mới ở `tools/` repo này |
| Task KRE-001/002/003 tạm mở ở boom-rs #375/#376/#377 | Đã transfer sang boom-kr-server #1/#2/#3; URL cũ redirect, không còn task tại boom-rs |

Patch series gốc và review report được giữ local để truy vết, không còn worktree docs cũ là nơi làm
việc. Client/asset gốc tiếp tục ở `boom/clients/`; repo mới không copy chúng vào Git.
