# Ma trận F3 mở rộng cho KR 00967

Tài liệu sinh từ metadata đã lưu. [JSON](../evidence/content/f3-extended-pve-matrix.json) giữ hash, biến thể, cờ script, neo tĩnh và đối chứng cho từng mục.

Có **17 map thuộc 8 họ `F3?`**, gồm **17 map có script**, và **4 hệ thống mở rộng**. Tám họ vẫn là ứng viên theo [bảng xếp loại](classification.md) và [quyết định bao phủ nội dung](../evidence/owner-directives/full-content-coverage.json).

## Tám họ map xếp tạm

Tên từng biến thể, lưới và số vị trí xuất phát lấy từ [census](census.json), đối chiếu với [kiểm kê nội dung](../CONTENT.md). Số vị trí trong file chưa xác định số người được vào phòng. Hai cột script phân biệt cờ hiện diện trong file và số map có tag bổ sung mà bảng xếp loại ghi ở `script_unknown`. Các họ này nằm ngoài [catalog F3-0](f3-pve-catalog.md). Riêng `Antarctic`, hàng mode cùng tên ở `F2?`; hàng họ map vẫn ở `F3?`.

| Họ | Map | Biến thể | Lưới | Spawn/map | Có script / có tag bổ sung | Neo riêng |
|---|---:|---|---|---:|---|---|
| Antarctic | 1 | `Antarctic01` | 15×101 | 4 | 1 / 1 | `CMapScriptStageClear` (`0xF16BC0`) |
| DragonPalace | 1 | `DragonPalace01` | 15×101 | 4 | 1 / 1 | `CMapScriptStageClear` (`0xF16BC0`) |
| EscapeMaze | 3 | `EscapeMaze00`, `EscapeMaze01`, `EscapeMaze02` | 29×25 | 8 | 3 / 3 | `CMapScriptEscapeMaze` (`0x6513B0`) |
| GolemRun | 1 | `GolemRun01` | 15×101 | 4 | 1 / 1 | `CMapScriptStageClear` (`0xF16BC0`) |
| IamWriggle | 3 | `IamWriggle01`, `IamWriggle02`, `IamWriggle03` | 15×13 | 8 | 3 / 3 | `CreateDlgGameTypeIamWriggle` (`0xc6c080`) |
| MineExp | 1 | `MineExp01` | 15×101 | 4 | 1 / 1 | `CMapScriptStageClear` (`0xF16BC0`) |
| Minibot | 4 | `Minibot01`, `Minibot02`, `Minibot03`, `Minibot04` | 15×13 | 4 | 4 / 4 | `CMapScriptRepairMinibot` (`0x6ABA20`) |
| SavetheEarth | 3 | `SavetheEarth01`, `SavetheEarth02`, `SavetheEarth03` | 15×13 | 4 | 3 / 3 | `CMapScriptCatchTail` (`0x62A100`) |

Factory chung ở `0xeba2d0`. Ghép tên tag script với lớp controller để chọn điểm quan sát. Cần thấy controller được tạo trong mode thật mới xác nhận đường chạy.

## Hệ thống mở rộng theo PR #713

Các mục tính năng lấy từ [lộ trình F3](../PLAN.md#full-game-roadmap). Số lượng và neo nhóm lấy từ [catalog F3-0](f3-pve-catalog.md) và [ma trận F3-1](f3-script-authority-matrix.md). Chúng mô tả cả nhóm, chưa phân map theo từng nhánh mode hoặc tính năng.

| Hệ thống | Tính năng trong lộ trình | Map / họ trong catalog | Tài nguyên pack (số mục) |
|---|---|---|---|
| CA Hero | HP; Kỹ năng; Trang bị; Thuốc; Nhiệm vụ | 53 / 1 | `FxContents/CAHero` (69) |
| AIBattle (협공배틀) | Độ khó theo sao; Nhân vật ẩn mở bằng điểm (협공배틀 포인트, 로두마니) | 4 / 4 | `FxContents/AIBattle` (10) |
| Hidden Catch (히든 캐치) | Chế độ thường; Chế độ thi đấu (vs); Chế độ máy xu, 100 màn theo lộ trình | 37 / 30 | `FxHiddenCatch/HiddenCatch` (291), `FxHiddenCatch/Prepare_HC` (12) |
| Monster mùa 2 và 몬스터대작전 | Monster mùa 2 dạng hành động; 몬스터대작전 | 368 / 61 | `FxMonster/Monster` (1022), `FxMonster/MonsterSeason2` (236) |

Tiền tố `MonS2` cho **26 map ứng viên thuộc 8 họ**. Catalog Monster gồm cả hai mùa; chưa nối các map ứng viên với 몬스터대작전. Số 100 màn của Hidden Catch là mô tả trong lộ trình, chưa được kiểm bằng danh sách map hay luồng chơi máy xu.

## Neo tĩnh và câu hỏi chờ Wine r2/r3

[Neo IDA](../protocol/f2-f5-f3-ida-anchors.md) ghi GameType client `16` (`CreateDlgGameTypeAIBattle`), `28` (`HiddenCatch_NormalMode`), `29` (`HiddenCatch_CompetitionMode`), `43` (`CreateDlgGameTypeIamWriggle`). Các liên hệ với mode và route phải kiểm trong Wine.

Nhãn đã lưu của `CGameStage::OnPacket`: case `9` → `0xe6aa90`, case `80` → `0xe6d730`, case `135` → `0xe67640`, case `136` → `0xe67770`, case `177` → `0xe751f0`. Nhãn case Hex-Rays không phải opcode; đối chiếu frame, jump table và VA handler. Case quest chưa nối riêng với CA Hero.

Mỗi mục giữ `authority_status: unknown` và route KR `unknown`. Guard `RollAttack::MonsterAttackReq` có kiểm cờ host hoặc `attackSeq`. Phân quyền HP, AI, sinh quái, trigger, qua màn và thưởng vẫn chưa đo. [Probe r1](f2-f3-wine-probe-r1.md) chỉ tải mẫu CA Hero/AIBattle trong phòng `roomtype=0`.

| Mục | Câu hỏi còn mở | Đối chứng đề xuất |
|---|---|---|
| Antarctic | Điều gì kích hoạt đường chạy và qua màn? | So lần vượt mốc có trigger với lượt dừng trước mốc. |
| DragonPalace | Điều kiện chết hoặc bị khóa có đổi kết quả màn không? | So cùng đoạn đường với và không có trạng thái bị khóa. |
| EscapeMaze | Ai xác nhận lấy chìa, mở cửa và thoát mê cung? | Thử cùng cửa khi có và khi thiếu chìa; theo dõi cả hồi sinh. |
| GolemRun | Ai kích hoạt đá rơi và điều kiện qua màn? | So lượt đi qua vùng có dấu trigger với lượt không vào vùng. |
| IamWriggle | Cửa vào GameType 43 có tải đúng họ map và áp dụng cấu hình vật phẩm không? | So trạng thái vật phẩm trước và sau thao tác trong mode thật. |
| MineExp | Ai kích hoạt laser và ghi nhận sát thương? | So một lượt tiếp xúc laser với một lượt tránh được. |
| Minibot | Ai cập nhật trạng thái Minibot tại trạm sửa? | So cùng trạng thái trước và sau khi vào trạm với lượt đứng ngoài. |
| SavetheEarth | Ai quyết định bắt mục tiêu hoặc cứu máy bay? | So lần hoàn tất mục tiêu với lượt thiếu điều kiện cuối. |
| CA Hero | Ai xác nhận thay đổi HP, hiệu ứng kỹ năng/trang bị, tiêu hao thuốc và tiến độ nhiệm vụ? | So hai cấu hình trang bị, trúng/hụt và dùng/không dùng thuốc; kiểm nhiệm vụ trước, sau và sau đăng nhập lại. |
| AIBattle (협공배틀) | Mức sao đổi AI thế nào, điểm được tính ở đâu, ngưỡng mở là gì và khi nào 로두마니 xuất hiện? | So hai mức sao trên cùng map; đối chiếu điểm và danh sách nhân vật trước/sau kết quả và sau đăng nhập lại. |
| Hidden Catch (히든 캐치) | Ai xác nhận lựa chọn, điểm và kết quả trong từng mode? Đường vào, thứ tự và số màn máy xu là gì? | So lựa chọn đúng/sai và lần qua màn ở chế độ thường và thi đấu; nếu mở được máy xu, đo luồng vào và chuyển màn riêng. |
| Monster mùa 2 và 몬스터대작전 | Ai quyết định AI, đợt quái, sát thương, qua màn và thưởng? 몬스터대작전 dùng cửa vào và map nào? | Đo mẫu MonS2 với đối chứng đánh hụt và thiếu điều kiện qua màn; tìm cửa vào 몬스터대작전 riêng trước khi nối map. |

Dùng cửa vào thật của mode, lặp với đối chứng và đảo host khi có hai client. Ghi bên phát, bên áp dụng và bên ghi bền; ghi blocker nếu chưa vào được mode. Xem [thiết kế probe F3-2](f3-pve-authority-design.md).

Sinh lại: `python3 tools/kr_f3_extended_pve_audit.py --write`. Kiểm ngoại tuyến: `python3 tools/kr_f3_extended_pve_audit.py --check`.

SHA-256 nội dung: `22fb721354a7f0aedce5d339b6903cf69ed5f1c32e90db560105399a4210d1e0`. Hash dùng JSON UTF-8 với khóa sắp tăng, dấu phân tách gọn và một ký tự xuống dòng cuối, bỏ trường `matrix_sha256`.
