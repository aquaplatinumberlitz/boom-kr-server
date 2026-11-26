# Kiểm kê sự kiện và kinh tế theo mùa F6-0

Nguồn: [Hệ thống của client](../CONTENT.md#hệ-thống-của-client), [xếp loại A/B/C](classification.md) và [phạm vi F6](../PLAN.md#full-game-roadmap). [Catalog JSON](../evidence/content/f6-events-catalog.json) giữ locator và hash đầu vào.
Trong 59 hệ thống của KR00967, bảng dưới có **19 hệ thống được xếp vào F6**, 5 hệ thống đối chiếu và 1 mục bị loại.
Con số khoảng 25 trong PLAN là ước lượng phạm vi. Các mục đối chiếu giữ nguyên đợt trong bảng xếp loại. Dwarf/libcef dùng chung F4/F6 nhưng không có hàng hệ thống riêng trong census.

Số mục và locator là metadata tĩnh từ index pack. Chúng cho biết tài nguyên có mặt, chưa cho biết client có mở tính năng hay server xử lý thế nào. Cả ba trường: wire KR, giá trị/giá/tỷ lệ của server gốc và trạng thái còn chạy hay đã ngừng trong build 00967 đều là `unknown` cho từng hệ thống.

| Hệ thống | Phạm vi / đợt xếp loại | Nhóm tài nguyên (số mục) | Ghi chú |
|---|---|---|---|
| CAPass | `in_scope` / F6 | `FxContents/CAPass` (9) | — |
| Câu cá | `in_scope` / F6 | `CharTE/FishingDeco` (1), `FxDesktop/FishTrade` (2), `fx/FishingObjects` (70) | — |
| DIYBomb | `in_scope` / F6 | `FxContents/DIYBomb` (9) | — |
| DTT | `in_scope` / F6 | `fx/DTT` (29), `fx/DTT_seasonShop` (14) | — |
| Exchange | `in_scope` / F6 | `FxDesktop/Exchange` (13) | — |
| GrowSprout | `in_scope` / F6 | `fx/GrowSprout` (43) | — |
| HoneyButterChip | `in_scope` / F6 | `FxContents/HoneyButterChip` (18) | — |
| LineFriends | `in_scope` / F6 | `fx/LineFriends` (7) | — |
| LuckyCard | `in_scope` / F6 | `FxContents/LuckyCard` (5) | — |
| MagicMarket | `in_scope` / F6 | `FxContents/MagicMarket` (57) | — |
| MineExplore | `in_scope` / F6 | `FxContents/MineExplore` (8) | — |
| MonWars | `in_scope` / F6 | `fx/MonWars` (7) | — |
| Nhiệm vụ | `in_scope` / F6 | `FxContents/DailyMission` (22), `FxDesktop/Quest` (59), `FxStage/Game:Mission` (2), `fx/CAMission` (20), `fx/MissionPong` (10) | — |
| Partners | `in_scope` / F6 | `fx/Partners` (13), `fx/TilePartners` (10), `fx/TilePartners_Ground` (4) | — |
| Pet | `in_scope` / F6 | `fx/Pet` (1356) | — |
| RetroWriggle | `in_scope` / F6 | `fx/RetroWriggle` (15) | — |
| SelectionBox | `in_scope` / F6 | `fx/SelectionBox` (7) | — |
| Sự kiện TakeCareOfEvent | `in_scope` / F6 | `FxDesktop/TakeCareOfEvent` (39) | — |
| Sự kiện theo tiền tố Event | `in_scope` / F6 | `FxDesktop/Etc:Event` (2), `FxStage/Game:Event` (3), `FxStage/Lobby:Event` (11) | — |
| PCRoomBenefit | `excluded_policy` / ngoài phạm vi | `fx/PCRoomBenefit` (16) | F6 bỏ ưu đãi phòng máy Nexon (PC-bang). |
| Antarctic | `related_candidate` / F2 | `FxContents/Antarctic` (15) | Tên gợi chủ đề mùa lạnh; bảng Mode xếp tạm vào F2. |
| Bộ sưu tập item | `related_candidate` / F4 | `FxContents/ItemCollection` (27) | Đối chiếu kinh tế vật phẩm ở F4; chưa có bằng chứng dùng cho sự kiện F6. |
| IamWriggle | `related_candidate` / F3 | `FxContents/IamWriggle` (7) | Tên có phần Wriggle chung với RetroWriggle; chưa xác định quan hệ. |
| Monster | `related_candidate` / F3 | `FxMonster/Monster` (1022), `FxMonster/MonsterSeason2` (236) | Có nhóm MonsterSeason2; lộ trình giữ gameplay hai mùa ở F3. |
| SnowyChamp | `related_candidate` / F2 | `FxContents/SnowyChamp` (6) | Tên gợi chủ đề tuyết; bảng Mode xếp tạm vào F2. |

`DailyMission` nằm trong hàng Nhiệm vụ; các nhóm `Quest`, `CAMission` và `MissionPong` vẫn giữ nguyên theo census. Câu cá gồm tài nguyên trang trí, vật thể và giao diện trao đổi; DTT có nhóm `DTT_seasonShop`.
[Bảng Mode](../CONTENT.md#mode) nhận diện MagicMarket là hệ chế tạo từ sảnh. Nguồn tĩnh chưa xác định công thức. Bảng Pet trong census cho biết có dữ liệu XML, chưa xác định tỷ lệ của server gốc.

Khi mở từng hệ thống F6, lần từ tên nhóm tới UI và bộ đọc/ghi của build KR 00967. Kiểm đường mở UI và trao đổi dữ liệu bằng client trong mạng cô lập với server cục bộ. Cần nguồn KR riêng để xác nhận giá, thưởng và tỷ lệ gốc. Nếu không có nguồn, giữ `unknown`; giá trị tự chọn cho server giả lập phải ghi riêng là quyết định thiết kế.

## Tái lập

```bash
python3 tools/kr_f6_events_audit.py
python3 tools/kr_f6_events_audit.py --check
```

Công cụ chỉ đọc `census.json` và `classification.json`; không cần client, pack hay mạng. `input_sha256` băm byte gốc của hai file. `catalog_sha256` băm object JSON bỏ hai trường hash đó: UTF-8, khóa sắp tăng, không khoảng trắng, một LF cuối.
