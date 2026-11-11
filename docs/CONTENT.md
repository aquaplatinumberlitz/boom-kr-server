# Kiểm kê nội dung client KR 00967

Sinh từ [census.json](content/census.json) bằng `python3 tools/content_census.py --generate`; không sửa tay.

Tài liệu này trả lời câu hỏi **"full game gồm những gì"** bằng dữ liệu tĩnh trong pack của client.
Nó không chứng minh client hay server làm gì với từng mục. Cột nhóm nội dung và cột wiki là **suy từ tên**;
cột "server phải làm gì" để trống cho tới khi có ca chạy thử. Thứ tự công việc vẫn chỉ ở
[PLAN](PLAN.md#delivery-target).

## Tóm tắt

| Hạng mục | Số lượng | Ghi chú |
|---|---|---|
| Mục trong 13 pack | 51,467 | chỉ mục đọc trọn |
| Map | 1,151 thuộc 333 họ tên | 1,140 đọc trọn file, 11 đọc tới hết lưới ô, 0 lỗi |
| Map có script | 770 | 370 map đọc trọn và không có script |
| Map có bảng rơi item riêng | 1,025 | trọng số nằm trong file map |
| Map có item đặt sẵn | 542 | vị trí và xác suất trong file map |
| Dòng item trong `Attr/ItemAttr` | 335 | 105 dòng được ít nhất một map dùng |
| Phương tiện trong `Attr/ItemAttr` | 28 | mục `Ship/*` |
| Dòng nhân vật trong `Attr/BomberAttr` | 67 | 63 tên khác nhau |
| Theme trong `FxMap/Theme` | 142 | gồm cả icon và mục ngẫu nhiên |

Ba điều rút ra cho server:

1. **Phần lớn file map mang sẵn một bảng tên item kèm trọng số.** `boom-re/docs/map.md` đọc khối này là bảng rơi
   item của map; đây là nguồn tốt nhất hiện có cho tỉ lệ rơi theo từng map, tốt hơn wiki. Ba điều chưa đo: bên
   nào dùng bảng trong trận thật (client tự sinh hay server gốc gửi xuống), bảng kết hợp thế nào với trọng số
   mặc định trong `Attr/ItemAttr` (Village00 chỉ liệt kê tiền Lucci), và vì sao ExVillage01 đang chạy native
   không có bảng, chỉ có bốn item đặt sẵn.
2. **Chỉ 105 trong 335 dòng item được file map nhắc tới.** Các dòng còn lại chưa rõ được cấp qua đường
   nào (script, phần thưởng, cửa hàng, sự kiện). Thứ tự ưu tiên nên lấy từ bảng "item được map dùng" bên dưới.
3. **370 map không có script.** Đây là nhóm có khả năng chỉ cần server gửi đúng ID map và bảng
   item; điều đó là giả thuyết cho tới khi thử một map mỗi họ.

## Map theo nhóm nội dung

Cột: họ tên (bỏ số cuối), số map, kích thước lưới, số điểm spawn, số map có script, số item khác nhau có
trọng số lớn hơn 0 trong bảng rơi của họ, cặp `d1/d2` trong header (chưa rõ nghĩa), mục wiki nếu tên khớp rõ.

### Map đấu thường theo theme — 169 map, 24 họ

| Họ | Map | Lưới | Spawn | Có script | Item rơi | d1/d2 | Wiki |
|---|---|---|---|---|---|---|---|
| Island | 15 | 15×13 | 8 | 0 | 22 | 8/1 | 보물섬 |
| Xmas | 14 | 15×13 | 4–8 | 0 | 16 | 8/1 | 크리스마스 (đã ngừng) |
| Camp | 13 | 15×13 | 4–8 | 0 | 19 | 2/2, 2/3, 2/4, 4/1, 4/2, 8/1 | 캠프 |
| Forest | 12 | 15×13 | 4–8 | 0 | 25 | 2/4, 8/1 | 포레스트 |
| LovelyRoom | 10 | 15×13 | 8 | 10 | 5 | 8/1 | 포키의러블리룸 |
| PangLand | 10 | 15×13 | 8 | 10 | 14 | 4/2, 8/1 | 팡랜드 |
| Aqua | 9 | 15×13 | 8 | 9 | 19 | 2/4, 8/1 | 아쿠아 |
| Patrit | 8 | 15×13 | 4–8 | 0 | 18 | 2/4, 4/1, 8/1 | 해적 |
| Space | 8 | 15×13 | 8 | 8 | 0 | 12/1, 8/1 | 우주맵 |
| Cemetery | 7 | 15×13 | 4–8 | 0 | 30 | 2/4, 8/1 | 공동묘지 |
| Cookie | 7 | 15×13 | 4–8 | 0 | 14 | 4/1, 8/1 | 쿠키 |
| Desert | 7 | 15×13 | 4–8 | 0 | 10 | 8/1 | 데저트 |
| Factory | 7 | 15×13 | 8 | 0 | 15 | 8/1 | 팩토리 |
| Ice | 7 | 15×13 | 4–8 | 0 | 11 | 2/4, 8/1 | 아이스 |
| Mine | 7 | 15×13 | 6–8 | 0 | 14 | 8/1 | 광산 |
| Ocean | 7 | 15×13 | 8 | 0 | 15 | 8/1 | 바다 |
| Village | 6 | 15×13 | 4–8 | 0 | 11 | 8/1 | 빌리지 |
| Tomb | 5 | 15×13 | 6–8 | 0 | 28 | 8/1 | 무덤 |
| kfg | 3 | 15×13 | 8 | 0 | 11 | 8/1 | 켄터키 |
| Kfg | 3 | 15×13 | 6–8 | 0 | 15 | 8/1 | 켄터키 |
| forest | 1 | 15×13 | 8 | 0 | 5 | 8/1 | 포레스트 |
| mine | 1 | 15×13 | 2 | 0 | 0 | 2/1 | 광산 |
| Pangland | 1 | 15×13 | 8 | 1 | 0 | 8/1 | 팡랜드 |
| patrit | 1 | 15×13 | 8 | 0 | 7 | 8/1 | 해적 |

### Biến thể của map đấu (tiền tố Ex/Retro/Event/Novice/NXCL/ladder…) — 144 map, 88 họ

| Họ | Map | Lưới | Spawn | Có script | Item rơi | d1/d2 | Wiki |
|---|---|---|---|---|---|---|---|
| CaJJ_Wriggle | 7 | 15×13 | 4 | 7 | 7 | 2/2 | biến thể của Wriggle |
| RetroCampN | 6 | 15×13 | 8 | 6 | 8 | 12/1 | biến thể của Camp |
| RetroFactoryN | 6 | 15×13 | 8 | 6 | 7 | 12/1 | biến thể của Factory |
| RetroForestN | 6 | 15×13 | 8 | 6 | 7 | 12/1 | biến thể của Forest |
| RetroPatritN | 6 | 15×13 | 8 | 6 | 8 | 12/1 | biến thể của Patrit |
| RetroVillageN | 6 | 15×13 | 8 | 6 | 8 | 12/1 | biến thể của Village |
| CaJJ_CAPong | 5 | 15×13 | 4 | 0 | 7 | 2/2 | biến thể của CAPong |
| CaJJ_Clover | 5 | 15×13 | 8 | 0 | 0 | 2/4 | biến thể của Clover |
| RetroWriggleN | 5 | 15×13 | 4 | 5 | 7 | 2/2 | biến thể của Wriggle |
| Fm_CAPong | 3 | 15×13 | 4 | 0 | 7 | 2/2 | biến thể của CAPong |
| Fm_RescueET | 3 | 15×13 | 4 | 3 | 5 | 2/2 | biến thể của RescueET |
| CaJJ_Space | 2 | 15×13 | 8 | 2 | 0 | 12/1, 8/1 | biến thể của Space |
| ExBula | 2 | 15×13 | 8 | 0 | 7 | 2/4, 4/2 | biến thể của Bula |
| ExCamp | 2 | 15×13 | 8 | 0 | 9 | 2/4 | biến thể của Camp |
| ExFactory | 2 | 15×13 | 8 | 0 | 11 | 4/2, 8/1 | biến thể của Factory |
| ExPatrit | 2 | 15×13 | 8 | 0 | 13 | 8/1 | biến thể của Patrit |
| ExTomb | 2 | 15×13 | 8 | 0 | 21 | 8/1 | biến thể của Tomb |
| ExVillage | 2 | 15×13 | 8 | 0 | 7 | 2/4, 4/2 | biến thể của Village |
| Fm_Space | 2 | 15×13 | 8 | 2 | 0 | 12/1 | biến thể của Space |
| Fm_Wriggle | 2 | 15×13 | 4 | 2 | 7 | 2/2 | biến thể của Wriggle |
| BSBigCamp | 1 | 19×17 | 8 | 0 | 17 | 2/4 | biến thể của Camp |
| BSBigPatrit | 1 | 19×17 | 8 | 0 | 11 | 12/1 | biến thể của Patrit |
| BSBigVillage | 1 | 19×17 | 8 | 0 | 11 | 8/1 | biến thể của Village |
| BSSmallCamp | 1 | 15×13 | 4 | 0 | 14 | 2/2 | biến thể của Camp |
| BSSmallPatrit | 1 | 15×13 | 4 | 0 | 11 | 12/1 | biến thể của Patrit |
| BSSmallVillage | 1 | 15×13 | 4 | 0 | 11 | 12/1 | biến thể của Village |
| CaJJ_Camp | 1 | 15×13 | 8 | 0 | 14 | 2/4 | biến thể của Camp |
| CaJJ_Factory | 1 | 15×13 | 8 | 0 | 11 | 8/1 | biến thể của Factory |
| CaJJ_Forest | 1 | 15×13 | 8 | 0 | 10 | 8/1 | biến thể của Forest |
| CaJJ_Ocean | 1 | 15×13 | 8 | 0 | 14 | 8/1 | biến thể của Ocean |
| CaJJ_Patrit | 1 | 15×13 | 8 | 0 | 11 | 8/1 | biến thể của Patrit |
| CaJJ_RetroCamp | 1 | 15×13 | 8 | 0 | 14 | 2/4 | biến thể của Camp |
| CaJJ_RetroFactory | 1 | 15×13 | 8 | 0 | 11 | 8/1 | biến thể của Factory |
| CaJJ_RetroForest | 1 | 15×13 | 8 | 0 | 10 | 8/1 | biến thể của Forest |
| CaJJ_RetroPatrit | 1 | 15×13 | 8 | 0 | 11 | 8/1 | biến thể của Patrit |
| CaJJ_RetroVillage | 1 | 15×13 | 6 | 0 | 11 | 8/1 | biến thể của Village |
| CaJJ_Tomb | 1 | 15×13 | 6 | 0 | 21 | 8/1 | biến thể của Tomb |
| CaJJ_Village | 1 | 15×13 | 6 | 0 | 11 | 8/1 | biến thể của Village |
| EventCamp | 1 | 15×13 | 8 | 0 | 0 | 8/1 | biến thể của Camp |
| EventFactory | 1 | 15×13 | 8 | 0 | 0 | 8/1 | biến thể của Factory |
| EventForest | 1 | 15×13 | 8 | 0 | 0 | 8/1 | biến thể của Forest |
| EventOcean | 1 | 15×13 | 8 | 0 | 0 | 8/1 | biến thể của Ocean |
| EventPatrit | 1 | 15×13 | 8 | 0 | 0 | 8/1 | biến thể của Patrit |
| ExForest | 1 | 15×13 | 8 | 0 | 4 | 8/1 | biến thể của Forest |
| Fm_AIBtlDesert | 1 | 15×13 | 4 | 0 | 0 | 8/1 | biến thể của Desert |
| Fm_AIBtlForest | 1 | 15×13 | 8 | 0 | 18 | 2/4 | biến thể của Forest |
| Fm_AIBtlPatrit | 1 | 15×13 | 8 | 0 | 11 | 8/1 | biến thể của Patrit |
| Fm_AIBtlTomb | 1 | 15×13 | 8 | 0 | 5 | 8/1 | biến thể của Tomb |
| Fm_Camp | 1 | 15×13 | 8 | 0 | 14 | 2/4 | biến thể của Camp |
| Fm_Coloring | 1 | 15×13 | 4 | 1 | 12 | 2/2 | biến thể của Coloring |
| Fm_Patrit | 1 | 15×13 | 8 | 0 | 11 | 8/1 | biến thể của Patrit |
| Fm_Tomb | 1 | 15×13 | 8 | 0 | 5 | 8/1 | biến thể của Tomb |
| Fm_Village | 1 | 15×13 | 6 | 0 | 11 | 8/1 | biến thể của Village |
| LJ_Camp | 1 | 15×13 | 8 | 0 | 14 | 2/4 | biến thể của Camp |
| LJ_Factory | 1 | 15×13 | 8 | 0 | 11 | 8/1 | biến thể của Factory |
| LJ_Forest | 1 | 15×13 | 8 | 0 | 10 | 8/1 | biến thể của Forest |
| LJ_Patrit | 1 | 15×13 | 8 | 0 | 11 | 8/1 | biến thể của Patrit |
| LJ_Tomb | 1 | 15×13 | 6 | 0 | 21 | 8/1 | biến thể của Tomb |
| LJ_Village | 1 | 15×13 | 6 | 0 | 11 | 8/1 | biến thể của Village |
| NL_Camp | 1 | 15×13 | 8 | 0 | 14 | 2/4 | biến thể của Camp |
| NL_Factory | 1 | 15×13 | 8 | 0 | 11 | 8/1 | biến thể của Factory |
| NL_Forest | 1 | 15×13 | 8 | 0 | 10 | 8/1 | biến thể của Forest |
| NL_Patrit | 1 | 15×13 | 8 | 0 | 11 | 8/1 | biến thể của Patrit |
| NL_Tomb | 1 | 15×13 | 6 | 0 | 21 | 8/1 | biến thể của Tomb |
| NL_Village | 1 | 15×13 | 6 | 0 | 11 | 8/1 | biến thể của Village |
| NoviceFactory | 1 | 15×13 | 8 | 0 | 11 | 8/1 | biến thể của Factory |
| NoviceForest | 1 | 15×13 | 8 | 0 | 10 | 8/1 | biến thể của Forest |
| NoviceMine | 1 | 15×13 | 8 | 0 | 10 | 8/1 | biến thể của Mine |
| NovicePatrit | 1 | 15×13 | 8 | 0 | 11 | 8/1 | biến thể của Patrit |
| NoviceVillage | 1 | 15×13 | 6 | 0 | 11 | 8/1 | biến thể của Village |
| NXCL_Aqua | 1 | 15×13 | 8 | 1 | 10 | 8/1 | biến thể của Aqua |
| NXCL_Camp | 1 | 15×13 | 8 | 0 | 15 | 2/4 | biến thể của Camp |
| NXCL_Cookie | 1 | 15×13 | 8 | 0 | 11 | 8/1 | biến thể của Cookie |
| NXCL_Desert | 1 | 15×13 | 8 | 0 | 8 | 8/1 | biến thể của Desert |
| NXCL_Factory | 1 | 15×13 | 8 | 0 | 11 | 8/1 | biến thể của Factory |
| NXCL_Forest | 1 | 15×13 | 8 | 0 | 12 | 8/1 | biến thể của Forest |
| NXCL_Ice | 1 | 15×13 | 8 | 0 | 6 | 8/1 | biến thể của Ice |
| NXCL_Mine | 1 | 15×13 | 6 | 0 | 13 | 8/1 | biến thể của Mine |
| NXCL_Ocean | 1 | 15×13 | 8 | 0 | 11 | 8/1 | biến thể của Ocean |
| NXCL_PangLand | 1 | 15×13 | 8 | 1 | 12 | 8/1 | biến thể của PangLand |
| NXCL_Patrit | 1 | 15×13 | 8 | 0 | 11 | 8/1 | biến thể của Patrit |
| NXCL_Tomb | 1 | 15×13 | 6 | 0 | 22 | 8/1 | biến thể của Tomb |
| NXCL_Village | 1 | 15×13 | 6 | 0 | 13 | 8/1 | biến thể của Village |
| WinningStreak_Patrit | 1 | 15×13 | 8 | 0 | 4 | 8/1 | biến thể của Patrit |
| WL_Camp | 1 | 15×13 | 8 | 0 | 14 | 2/4 | biến thể của Camp |
| WL_Patrit | 1 | 15×13 | 8 | 0 | 11 | 8/1 | biến thể của Patrit |
| WL_Space | 1 | 15×13 | 8 | 1 | 0 | 8/1 | biến thể của Space |
| WL_Tomb | 1 | 15×13 | 6 | 0 | 21 | 8/1 | biến thể của Tomb |

### Nhiệm vụ và mode riêng — 122 map, 18 họ

| Họ | Map | Lưới | Spawn | Có script | Item rơi | d1/d2 | Wiki |
|---|---|---|---|---|---|---|---|
| SnowyFields | 18 | 15×13 | 4–8 | 13 | 33 | 1/4, 2/2, 2/4, 8/1 | 눈싸움 (sự kiện, chưa chắc) |
| Gumiho | 11 | 15×13 | 8 | 11 | 10 | 12/1, 8/1 | 구미호 |
| Baseball | 10 | 15×13 | 8 | 10 | 0 | 2/4 | 프로야구 |
| Hamster | 8 | 15×13 | 8 | 8 | 0 | 4/2 | 햄스터팜 |
| Kart | 8 | 15×13 | 4–8 | 0 | 13 | 8/1 | 카트 |
| Bula | 7 | 15×13 | 6–8 | 0 | 2 | 3/2, 4/2 | 불라 |
| Wriggle | 7 | 15×13 | 4 | 7 | 7 | 2/2 | 꼬물이 |
| Clover | 6 | 15×13 | 8 | 6 | 0 | 2/4 | 클로버 |
| Rabbit | 6 | 15×13 | 8 | 6 | 3 | 4/2 | 바니팜 |
| Zombie | 6 | 15×13 | 8 | 6 | 10 | 8/1 | 좀비 |
| CAPong | 5 | 15×13 | 4 | 0 | 7 | 2/2 | 크아퐁 |
| Coloring | 5 | 15×13 | 4 | 5 | 20 | 2/2 | 색칠하기 |
| PopcadeA | 5 | 15×13 | 1 | 5 | 18 | 1/1 | 팝케이드 |
| PopcadeB | 5 | 15×13 | 2 | 5 | 18 | 1/2 | 팝케이드 |
| PopcadeC | 5 | 15×13 | 4 | 5 | 18 | 1/4 | 팝케이드 |
| Zzolman | 5 | 15×13 | 4 | 5 | 18 | 1/4 | 쫄맨 (đã ngừng) |
| RescueET | 3 | 15×13 | 4 | 3 | 5 | 2/2 | 외계인구출 (đã ngừng) |
| SuperFighter | 2 | 15×13 | 8 | 0 | 11 | 2/4 | 슈퍼파이터 (sự kiện) |

### Monster mode — 368 map, 61 họ

| Họ | Map | Lưới | Spawn | Có script | Item rơi | d1/d2 | Wiki |
|---|---|---|---|---|---|---|---|
| Monster | 47 | 15×13 | 4 | 47 | 24 | 1/4 |  |
| MonSeal | 37 | 15×13 | 4 | 37 | 17 | 1/4 | 물개의분노 |
| MonPen | 34 | 15×13 | 4 | 34 | 13 | 1/4 | 황제의귀환 |
| MonSnowBear | 30 | 15×13 | 4 | 30 | 4 | 1/4 |  |
| MonGolem | 27 | 15×13 | 4 | 27 | 17 | 1/4 | 골렘대침공 |
| MonRobot | 27 | 15×13 | 4 | 27 | 15 | 1/4 | 로봇대공습 |
| MonTurtle | 27 | 15×13 | 4 | 27 | 11 | 1/4 | 거북대격돌 |
| MonS2Fire | 15 | 100×10 +3 cỡ | 1–4 | 15 | 3 | 1/1, 1/4 |  |
| MonUnlimited | 10 | 15×13 | 4 | 10 | 11 | 1/4 |  |
| MonCity | 9 | 15×13 | 4 | 9 | 14 | 1/4 |  |
| MonS2Sand | 5 | 100×10 | 4 | 5 | 0 | 1/4 |  |
| MonWarsHP | 5 | 15×13 | 4 | 5 | 11 | 1/4 |  |
| MonWarsHS | 5 | 15×13 | 4 | 5 | 11 | 1/4 |  |
| MonWarsBG | 4 | 15×13 | 4 | 4 | 16 | 1/4 |  |
| MonWarsBM | 4 | 15×13 | 4 | 4 | 14 | 1/4 |  |
| MonWarsBR | 4 | 15×13 | 4 | 4 | 14 | 1/4 |  |
| MonWarsBT | 4 | 15×13 | 4 | 4 | 11 | 1/4 |  |
| Fm_MonGolem | 3 | 15×13 | 4 | 3 | 16 | 1/4 | 골렘대침공 |
| Fm_MonSeal | 3 | 15×13 | 4 | 3 | 11 | 1/4 | 물개의분노 |
| Fm_MonSnowBear | 3 | 15×13 | 4 | 3 | 4 | 1/4 |  |
| Fm_Monster | 3 | 15×13 | 4 | 3 | 14 | 1/4 |  |
| Fm_MonTurtle | 3 | 15×13 | 4 | 3 | 11 | 1/4 | 거북대격돌 |
| MonWarsNG | 3 | 15×13 | 4 | 3 | 11 | 1/4 |  |
| MonWarsNM | 3 | 15×13 | 4 | 3 | 12 | 1/4 |  |
| MonWarsNP | 3 | 15×13 | 4 | 3 | 12 | 1/4 |  |
| MonWarsNR | 3 | 15×13 | 4 | 3 | 13 | 1/4 |  |
| MonWarsNS | 3 | 15×13 | 4 | 3 | 13 | 1/4 |  |
| MonWarsNT | 3 | 15×13 | 4 | 3 | 11 | 1/4 |  |
| MonNoviceGolem | 2 | 15×13 | 4 | 2 | 16 | 1/4 |  |
| MonNoviceSnow | 2 | 15×13 | 4 | 2 | 4 | 1/4 |  |
| MonNoviceTako | 2 | 15×13 | 4 | 2 | 13 | 1/4 |  |
| MonNoviceTurtle | 2 | 15×13 | 4 | 2 | 0 | 1/4 |  |
| MonWarsBS | 2 | 15×13 | 4 | 2 | 11 | 1/4 |  |
| MonWarsHH | 2 | 15×13 | 4 | 2 | 12 | 1/4 |  |
| MonWarsHM | 2 | 15×13 | 4 | 2 | 11 | 1/4 |  |
| MonWarsHR | 2 | 15×13 | 4 | 2 | 13 | 1/4 |  |
| MonS2Sand01Boss | 1 | 40×10 | 4 | 1 | 0 | 1/4 |  |
| MonS2Sand02Boss | 1 | 40×10 | 4 | 1 | 0 | 1/4 |  |
| MonS2Sand03Boss | 1 | 40×10 | 4 | 1 | 0 | 1/4 |  |
| MonS2Sand04Boss | 1 | 40×10 | 4 | 1 | 0 | 1/4 |  |
| MonS2Sand05Boss | 1 | 40×10 | 4 | 1 | 0 | 1/4 |  |
| MonS2Sand06Boss | 1 | 40×10 | 4 | 1 | 0 | 1/4 |  |
| MonWarsBBR | 1 | 15×13 | 8 | 1 | 0 | 8/1 |  |
| MonWarsBGB | 1 | 15×13 | 4 | 1 | 11 | 1/4 |  |
| MonWarsBLB | 1 | 15×13 | 4 | 1 | 11 | 1/4 |  |
| MonWarsBOB | 1 | 15×13 | 4 | 1 | 11 | 1/4 |  |
| MonWarsBPB | 1 | 15×13 | 4 | 1 | 11 | 1/4 |  |
| MonWarsBSB | 1 | 15×13 | 4 | 1 | 10 | 1/4 |  |
| MonWarsBTB | 1 | 15×13 | 4 | 1 | 11 | 1/4 |  |
| MonWarsHGB | 1 | 15×13 | 4 | 1 | 16 | 1/4 |  |
| MonWarsHLB | 1 | 15×13 | 4 | 1 | 13 | 1/4 |  |
| MonWarsHOB | 1 | 15×13 | 4 | 1 | 6 | 1/4 |  |
| MonWarsHPB | 1 | 15×13 | 4 | 1 | 11 | 1/4 |  |
| MonWarsHSB | 1 | 15×13 | 4 | 1 | 11 | 1/4 |  |
| MonWarsHTB | 1 | 15×13 | 4 | 1 | 11 | 1/4 |  |
| MonWarsNGB | 1 | 15×13 | 4 | 1 | 11 | 1/4 |  |
| MonWarsNLB | 1 | 15×13 | 4 | 1 | 12 | 1/4 |  |
| MonWarsNOB | 1 | 15×13 | 4 | 1 | 11 | 1/4 |  |
| MonWarsNPB | 1 | 15×13 | 4 | 1 | 11 | 1/4 |  |
| MonWarsNSB | 1 | 15×13 | 4 | 1 | 11 | 1/4 |  |
| MonWarsNTB | 1 | 15×13 | 4 | 1 | 11 | 1/4 |  |

### RPG và CA Hero — 124 map, 53 họ

| Họ | Map | Lưới | Spawn | Có script | Item rơi | d1/d2 | Wiki |
|---|---|---|---|---|---|---|---|
| CAhero | 53 | 15×13 | 8 | 26 | 38 | 2/4, 8/4 | 크아 히어로 |
| RPG0801_N | 3 | 21×16 +2 cỡ | 4 | 3 | 0 | 1/4 |  |
| RPG08_N | 3 | 20×22 +2 cỡ | 4 | 3 | 0 | 1/4 |  |
| RPG09_N | 3 | 23×17 +2 cỡ | 4 | 3 | 0 | 1/4 |  |
| RPG10_N | 3 | 20×16 +2 cỡ | 4 | 3 | 0 | 1/4 |  |
| RPG11_N | 3 | 25×25 | 4 | 3 | 0 | 1/4 |  |
| RPG04_N | 2 | 20×25 +1 cỡ | 4 | 2 | 0 | 1/4 |  |
| RPG0501_N | 2 | 25×12 +1 cỡ | 4 | 2 | 0 | 1/4 |  |
| RPG05_N | 2 | 25×17 +1 cỡ | 4 | 2 | 0 | 1/4 |  |
| RPG0601_N | 2 | 20×15 +1 cỡ | 4 | 2 | 0 | 1/4 |  |
| RPG06_N | 2 | 20×20 +1 cỡ | 4 | 2 | 0 | 1/4 |  |
| RPG0701_N | 2 | 20×25 +1 cỡ | 4 | 2 | 0 | 1/4 |  |
| RPG07_N | 2 | 25×12 +1 cỡ | 4 | 2 | 0 | 1/4 |  |
| RPG0901_N | 2 | 20×25 +1 cỡ | 4 | 2 | 0 | 1/4 |  |
| RPG1001_N | 2 | 21×21 +1 cỡ | 4 | 2 | 0 | 1/4 |  |
| RPG | 1 | 21×19 | 4 | 1 | 0 | 1/4 |  |
| RPG01_B | 1 | 20×18 | 4 | 1 | 0 | 1/4 |  |
| RPG01_N | 1 | 25×15 | 4 | 1 | 0 | 1/4 |  |
| RPG01_S | 1 | 25×20 | 4 | 1 | 0 | 1/4 |  |
| RPG02_B | 1 | 21×20 | 4 | 1 | 0 | 1/4 |  |
| RPG02_N | 1 | 25×17 | 4 | 1 | 0 | 1/4 |  |
| RPG02_S | 1 | 20×13 | 4 | 1 | 0 | 1/4 |  |
| RPG03_B | 1 | 20×12 | 4 | 1 | 0 | 1/4 |  |
| RPG03_N | 1 | 25×12 | 4 | 1 | 0 | 1/4 |  |
| RPG03_S | 1 | 20×12 | 4 | 1 | 0 | 1/4 |  |
| RPG04_B | 1 | 20×12 | 4 | 1 | 0 | 1/4 |  |
| RPG04_S | 1 | 25×20 | 4 | 1 | 0 | 1/4 |  |
| RPG0501_B | 1 | 20×18 | 4 | 1 | 0 | 1/4 |  |
| RPG0501_S | 1 | 20×12 | 4 | 1 | 0 | 1/4 |  |
| RPG05_B | 1 | 25×20 | 4 | 1 | 0 | 1/4 |  |
| RPG05_S | 1 | 25×22 | 4 | 1 | 0 | 1/4 |  |
| RPG0601_B | 1 | 20×12 | 4 | 1 | 0 | 1/4 |  |
| RPG0601_S | 1 | 20×25 | 4 | 1 | 0 | 1/4 |  |
| RPG06_B | 1 | 20×12 | 4 | 1 | 0 | 1/4 |  |
| RPG06_S | 1 | 20×12 | 4 | 1 | 0 | 1/4 |  |
| RPG0701_B | 1 | 20×12 | 4 | 1 | 0 | 1/4 |  |
| RPG0701_S | 1 | 20×12 | 4 | 1 | 0 | 1/4 |  |
| RPG07_B | 1 | 20×12 | 4 | 1 | 0 | 1/4 |  |
| RPG07_S | 1 | 21×12 | 4 | 1 | 0 | 1/4 |  |
| RPG0801_B | 1 | 21×17 | 4 | 1 | 0 | 1/4 |  |
| RPG0801_S | 1 | 21×12 | 4 | 1 | 0 | 1/4 |  |
| RPG08_B | 1 | 20×12 | 4 | 1 | 0 | 1/4 |  |
| RPG08_S | 1 | 20×20 | 4 | 1 | 0 | 1/4 |  |
| RPG0901_B | 1 | 20×15 | 4 | 1 | 0 | 1/4 |  |
| RPG0901_S | 1 | 25×15 | 4 | 1 | 0 | 1/4 |  |
| RPG09_B | 1 | 20×12 | 4 | 1 | 0 | 1/4 |  |
| RPG09_S | 1 | 25×20 | 4 | 1 | 0 | 1/4 |  |
| RPG1001_B | 1 | 20×19 | 4 | 1 | 0 | 1/4 |  |
| RPG1001_S | 1 | 20×20 | 4 | 1 | 0 | 1/4 |  |
| RPG10_B | 1 | 20×15 | 4 | 1 | 0 | 1/4 |  |
| RPG10_S | 1 | 20×19 | 4 | 1 | 0 | 1/4 |  |
| RPG11_B | 1 | 21×20 | 4 | 1 | 0 | 1/4 |  |
| RPG11_S | 1 | 25×20 | 4 | 1 | 0 | 1/4 |  |

### Hidden Catch — 37 map, 30 họ

| Họ | Map | Lưới | Spawn | Có script | Item rơi | d1/d2 | Wiki |
|---|---|---|---|---|---|---|---|
| Hidden | 6 | 15×13 | 4 | 6 | 12 | 1/4 | 히든캐치 |
| Hdn_Mine | 2 | 15×13 | 6–8 | 0 | 11 | 8/1 | 히든캐치 |
| Hdn_Patrit | 2 | 15×13 | 4–8 | 0 | 10 | 4/1, 8/1 | 히든캐치 |
| Hdn_Alche_Aqua | 1 | 15×13 | 8 | 1 | 17 | 2/4 | 히든캐치 |
| Hdn_Alche_Cam | 1 | 15×13 | 8 | 0 | 14 | 2/4 | 히든캐치 |
| Hdn_Alche_Ceme | 1 | 15×13 | 8 | 0 | 12 | 8/1 | 히든캐치 |
| Hdn_Alche_Fac | 1 | 15×13 | 8 | 0 | 5 | 2/4 | 히든캐치 |
| Hdn_Alche_Kart | 1 | 15×13 | 8 | 0 | 1 | 8/1 | 히든캐치 |
| Hdn_Alche_Rab | 1 | 15×13 | 8 | 1 | 0 | 2/4 | 히든캐치 |
| Hdn_Alice | 1 | 15×13 | 8 | 0 | 6 | 12/1 | 히든캐치 |
| Hdn_BlackLucy | 1 | 15×13 | 8 | 0 | 6 | 12/1 | 히든캐치 |
| Hdn_Caffe | 1 | 15×13 | 8 | 0 | 0 | 8/1 | 히든캐치 |
| Hdn_Camp | 1 | 15×13 | 4 | 0 | 2 | 4/1 | 히든캐치 |
| Hdn_Clover | 1 | 15×13 | 8 | 1 | 1 | 2/4 | 히든캐치 |
| Hdn_DarkCastle | 1 | 15×13 | 8 | 1 | 9 | 12/1 | 히든캐치 |
| Hdn_Factory | 1 | 15×13 | 8 | 0 | 9 | 8/1 | 히든캐치 |
| Hdn_Ice | 1 | 15×13 | 8 | 0 | 3 | 2/4 | 히든캐치 |
| Hdn_Ocean | 1 | 15×13 | 8 | 0 | 11 | 8/1 | 히든캐치 |
| Hdn_R_Camp | 1 | 15×13 | 8 | 0 | 14 | 2/4 | 히든캐치 |
| Hdn_R_Factory | 1 | 15×13 | 8 | 0 | 11 | 8/1 | 히든캐치 |
| Hdn_R_Forest | 1 | 15×13 | 8 | 0 | 10 | 8/1 | 히든캐치 |
| Hdn_R_Patrit | 1 | 15×13 | 8 | 0 | 11 | 8/1 | 히든캐치 |
| Hdn_R_Village | 1 | 15×13 | 6 | 0 | 11 | 8/1 | 히든캐치 |
| Hdn_Smash | 1 | 15×13 | 8 | 0 | 11 | 12/1 | 히든캐치 |
| Hdn_Square | 1 | 15×13 | 8 | 0 | 0 | 8/1 | 히든캐치 |
| Hdn_SuperFight | 1 | 15×13 | 8 | 0 | 11 | 2/4 | 히든캐치 |
| Hdn_Tomb | 1 | 15×13 | 8 | 0 | 8 | 8/1 | 히든캐치 |
| Hdn_Village | 1 | 15×13 | 6 | 0 | 6 | 8/1 | 히든캐치 |
| Hdn_Wriggle | 1 | 15×13 | 4 | 1 | 7 | 2/2 | 히든캐치 |
| HiddenCatchTest | 1 | 15×13 | 8 | 0 | 10 | 8/1 | 히든캐치 |

### Màn thưởng — 43 map, 13 họ

| Họ | Map | Lưới | Spawn | Có script | Item rơi | d1/d2 | Wiki |
|---|---|---|---|---|---|---|---|
| BonusBigReward | 13 | 15×51 +4 cỡ | 4–8 | 13 | 0 | 12/1, 8/1 |  |
| Bonus | 12 | 15×13 | 4–8 | 0 | 0 | 8/1 |  |
| BonusCp | 3 | 15×13 | 8 | 0 | 0 | 8/1 |  |
| BonusExp | 3 | 15×13 | 8 | 0 | 0 | 8/1 |  |
| BonusBigRPG | 2 | 22×22 +1 cỡ | 4 | 2 | 0 | 1/4 |  |
| BonusLargeGourd | 2 | 15×13 | 8 | 2 | 0 | 8/1 |  |
| BonusMazeMap | 2 | 25×25 | 8 | 2 | 0 | 8/1 |  |
| Bonus_16th | 1 | 15×13 | 8 | 0 | 0 | 8/1 |  |
| Bonus_19th | 1 | 15×13 | 8 | 1 | 0 | 8/1 |  |
| BonusBig | 1 | 24×24 | 8 | 0 | 0 | 8/1 |  |
| BonusCP_18th | 1 | 15×13 | 8 | 0 | 0 | 12/1 |  |
| BonusCP_LINE | 1 | 15×13 | 8 | 0 | 0 | 12/1 |  |
| BonusPCRoom | 1 | 24×24 | 8 | 1 | 8 | 8/1 |  |

### Sảnh, cửa hàng, tutorial, map tuỳ biến — 36 map, 17 họ

| Họ | Map | Lưới | Spawn | Có script | Item rơi | d1/d2 | Wiki |
|---|---|---|---|---|---|---|---|
| LobbySquare | 9 | 49×37 | 6–8 | 9 | 0 | 8/1 |  |
| LobbyCaffe | 5 | 49×37 | 6–7 | 5 | 0 | 8/1 |  |
| CustomMap15x | 2 | 15×13 +1 cỡ | 12 | 2 | 0 | 12/1 |  |
| CustomMap31x | 2 | 31×13 +1 cỡ | 12 | 2 | 0 | 12/1 |  |
| LobbyClothing | 2 | 49×37 | 6 | 2 | 0 | 8/1 |  |
| LobbyJewelry | 2 | 49×37 | 6–7 | 2 | 0 | 8/1 |  |
| LobbyPet | 2 | 49×37 | 4–8 | 2 | 0 | 8/1 |  |
| shopmap | 2 | 3×3 +1 cỡ | 1 | 0 | 0 | 8/1 |  |
| Square | 2 | 49×37 | 1 | 2 | 0 | 8/1 |  |
| CAPassShopMap | 1 | 4×2 | 1 | 0 | 0 | 8/1 |  |
| CustomMap21x | 1 | 21×21 | 12 | 1 | 0 | 12/1 |  |
| DailyMissionShop | 1 | 4×4 | 1 | 0 | 0 | 12/1 |  |
| LobbySquareFishingTest | 1 | 49×37 | 1 | 1 | 0 | 8/1 |  |
| MagicMarketmap | 1 | 3×2 | 1 | 0 | 0 | 8/1 |  |
| SelectionBox | 1 | 3×2 | 1 | 0 | 0 | 12/1 |  |
| tlshopmap | 1 | 3×2 | 1 | 0 | 0 | 8/1 |  |
| Tutorial | 1 | 15×13 | 8 | 1 | 9 | 8/1 |  |

### Chưa phân nhóm — 108 map, 29 họ

| Họ | Map | Lưới | Spawn | Có script | Item rơi | d1/d2 | Wiki |
|---|---|---|---|---|---|---|---|
| UserContest | 17 | 15×13 | 2–8 | 0 | 32 | 2/4, 8/1 |  |
| BlackLucy | 9 | 15×13 | 8 | 0 | 6 | 12/1 |  |
| DarkCastle | 9 | 15×13 | 8 | 9 | 9 | 12/1, 8/1 |  |
| DarkVampire | 8 | 15×28 +5 cỡ | 8 | 7 | 9 | 8/1 |  |
| NewXmas | 6 | 15×13 | 8 | 6 | 6 | 8/1 |  |
| Balansa | 5 | 15×13 | 8 | 0 | 1 | 12/1 |  |
| Baram | 4 | 15×13 | 8 | 0 | 16 | 8/1 |  |
| Fm_BigBattle | 4 | 15×13 +3 cỡ | 24 | 4 | 10 | 1/1, 12/1 |  |
| Minibot | 4 | 15×13 | 4 | 4 | 8 | 2/2 |  |
| Smash | 4 | 15×16 +2 cỡ | 12 | 0 | 19 | 12/1 |  |
| Wriggletest | 4 | 15×13 | 4 | 4 | 7 | 2/2 |  |
| Clovertest | 3 | 15×13 | 8 | 3 | 0 | 2/4 |  |
| ConfusedFight | 3 | 15×13 | 12 | 1 | 18 | 12/1 |  |
| EscapeMaze | 3 | 29×25 | 8 | 3 | 0 | 12/1 |  |
| GoldButterfly | 3 | 15×13 | 8 | 0 | 11 | 12/1 |  |
| IamWriggle | 3 | 15×13 | 8 | 3 | 10 | 12/1, 8/1 |  |
| OkongOfLegend | 3 | 15×13 | 2 | 0 | 8 | 2/1 |  |
| SavetheEarth | 3 | 15×13 | 4 | 3 | 5 | 2/2 |  |
| Space_TW | 2 | 15×13 | 8 | 2 | 0 | 8/1 |  |
| WhiteAngel | 2 | 15×13 | 8 | 2 | 10 | 2/4 |  |
| Antarctic | 1 | 15×101 | 4 | 1 | 0 | 1/4 |  |
| DragonPalace | 1 | 15×101 | 4 | 1 | 0 | 1/4 |  |
| Fm_BigBattle_PreparePlay | 1 | 15×13 | 24 | 0 | 0 | 12/1 |  |
| Fm_BigBattleH_PreparePlay | 1 | 15×13 | 24 | 0 | 0 | 12/1 |  |
| GolemRun | 1 | 15×101 | 4 | 1 | 0 | 1/4 |  |
| Hiphopclub | 1 | 15×13 | 8 | 0 | 10 | 12/1 |  |
| MagNest | 1 | 15×13 | 8 | 1 | 9 | 2/4 |  |
| MineExp | 1 | 15×101 | 4 | 1 | 0 | 1/4 |  |
| WinningStreak_Colosseum | 1 | 15×13 | 4 | 0 | 18 | 2/2 |  |

Map chỉ đọc tới hết lưới ô (đuôi file có cấu trúc riêng, bảng rơi và script chưa lấy được): Bula00, Bula01, Bula02, Bula03, Bula04, Bula15, Bula16, EventForest00, EventPatrit00, ExBula01, ExBula02.

### Nhóm map do client khai báo (`Attr/MapGroup`)

| Nhóm | Số map | Map |
|---|---|---|
| PraticeMapGroup | 45 | Village00, Village05, Village06, Village09, Village10, Xmas01, Xmas03, Xmas06, Xmas09, Xmas10, Forest02, Forest03, Forest07, Forest12, Forest14, Ice00, Ice01, Ice06, Ice07, Ice09, Desert00, Desert03, Desert04, Desert06, Desert07, Factory00, Factory02, Factory07, KFG01, KFG02, KFG03, Mine00, Mine01, Mine02, Mine05, Mine06, Ocean00, Ocean01, Ocean02, Ocean04, Ocean08, Patrit01, Patrit04, Patrit05, Patrit07 |
| LadderMapGroup | 6 | Camp08, Factory07, Forest07, Village10, Tomb12, Patrit14 |
| ReBirthMapGroup | 2 | Tomb?, Cemetery? |

## Item

### Item được map dùng

Xếp theo số map có item trong bảng rơi với trọng số lớn hơn 0. "Đặt sẵn" là số map đặt item ở vị trí cố định. Tên trong file map
được ghép với `Attr/ItemAttr` không phân biệt hoa thường và tiền tố `Item`. Cột trạng thái lấy từ
[PROTOCOL](PROTOCOL.md#item-families), không phải kết quả của tool này.

| Kind | Tên | Nhóm | Trọng số mặc định | Map có trong bảng rơi | Map đặt sẵn | Trạng thái |
|---|---|---|---|---|---|---|
| 145 | Item/ItemNiddle | 23 | 0 | 603 | 51 | đã đo native trên Python (kim), chưa port Rust |
| 143 | Item/ItemDart | 21 | 0 | 521 | 1 |  |
| 148 | Spring | 26 | 0 | 407 | 48 |  |
| 151 | Item/ItemShield | 29 | 0 | 407 | 26 |  |
| 150 | RemoteBomb | 28 | 0 | 371 | 24 |  |
| 40 | Item/ItemAirship | 2 | 150 | 363 | 1 |  |
| 44 | Item/ItemAirshipBroken | 2 | 70 | 345 | 0 |  |
| 41 | Item/ItemOwl | 2 | 350 | 344 | 1 |  |
| 152 | Item/ItemSensor | 30 | 0 | 340 | 5 |  |
| 68 | LucciSilver | 99 | 1100 | 337 | 18 |  |
| 69 | LucciGold | 99 | 250 | 336 | 29 |  |
| 70 | LucciPocketBronze | 99 | 30 | 336 | 16 |  |
| 71 | LucciPocketSilver | 99 | 30 | 335 | 14 |  |
| 72 | LucciPocketGold | 99 | 15 | 333 | 23 |  |
| 42 | Item/ItemTurtle0 | 2 | 340 | 325 | 9 |  |
| 43 | Item/ItemTurtle1 | 2 | 60 | 325 | 2 |  |
| 67 | LucciBronze | 99 | 800 | 320 | 20 |  |
| 5 | Item/ItemMaxPower | 1 | 150 | 262 | 97 | đã đo native (CP045) |
| 0 | Item/ItemBomb | 1 | 3650 | 240 | 228 | đã đo native (2/4/8 người) |
| 2 | Item/ItemRoller | 1 | 2000 | 238 | 206 | đã đo native |
| 1 | Item/ItemFire | 1 | 2800 | 162 | 234 | đã đo native |
| 146 | Item/ItemScanner | 24 | 0 | 123 | 0 |  |
| 224 | Item/ItemBond | 11 | 0 | 108 | 2 |  |
| 3 | Item/ItemShoe | 1 | 150 | 100 | 89 |  |
| 10 | Item/ItemSuperMan | 1 | 0 | 95 | 52 |  |
| 4 | Item/ItemPunch | 1 | 300 | 70 | 47 |  |
| 11 | Item/ItemGreenDevil | 1 | 0 | 60 | 10 |  |
| 7 | Item/ItemSkeleton0 | 1 | 200 | 54 | 18 |  |
| 9 | Item/ItemMoonWalk | 1 | 0 | 49 | 2 |  |
| 155 | HollowBomb | 33 | 0 | 41 | 1 |  |
| 6 | Item/ItemHollow | 1 | 200 | 40 | 15 |  |
| 14 | Item/ItemSnow | 1 | 0 | 35 | 18 |  |
| 142 | Camouflage | 1 | 0 | 30 | 1 |  |
| 39 | Item/ItemMonsterStop | 3 | 0 | 29 | 13 |  |
| 225 | Item/ItemTrap | 11 | 0 | 28 | 1 |  |
| 52 | Larva | 2 | 0 | 23 | 3 |  |
| 8 | Item/ItemSkeleton1 | 1 | 200 | 21 | 1 |  |
| 140 | RedDevil | 1 | 80 | 18 | 33 |  |
| 144 | Item/ItemOxygen | 22 | 0 | 16 | 0 |  |
| 172 | Item/ItemRescueCoin | 47 | 0 | 13 | 4 |  |
| 45 | Item/ItemOctopus0 | 2 | 0 | 10 | 0 |  |
| 46 | Item/ItemOctopus1 | 2 | 0 | 10 | 0 |  |
| 57 | Rabbit | 2 | 0 | 10 | 0 |  |
| 47 | Item/ItemRailCar | 2 | 0 | 8 | 1 |  |
| 227 | Banana1 | 11 | 0 | 7 | 0 |  |
| 51 | Tank | 2 | 0 | 6 | 41 |  |
| 147 | Item/ItemPotion | 25 | 0 | 6 | 8 |  |
| 58 | Rudolph | 2 | 0 | 6 | 0 |  |
| 154 | Item/ItemFuel | 32 | 0 | 6 | 0 |  |
| 50 | Item/ItemKart0 | 2 | 0 | 5 | 2 |  |
| 65 | RetroMouse | 2 | 0 | 5 | 2 |  |
| 153 | Nitro | 31 | 0 | 5 | 2 |  |
| 48 | Chicken | 2 | 0 | 5 | 0 |  |
| 64 | BalansaCap | 2 | 0 | 5 | 0 |  |
| 149 | Drill | 27 | 0 | 4 | 1 |  |
| 229 | Banana3 | 11 | 0 | 4 | 0 |  |
| 230 | Banana8 | 11 | 0 | 4 | 0 |  |
| 307 | Item/ItemShower | 116 | 0 | 4 | 0 |  |
| 308 | Item/ItemShine | 117 | 0 | 4 | 0 |  |
| 309 | Item/ItemThorny | 118 | 0 | 4 | 0 |  |
| 55 | Shark | 2 | 0 | 3 | 9 |  |
| 228 | Banana2 | 11 | 0 | 3 | 0 |  |
| 141 | Skate | 1 | 0 | 2 | 0 |  |
| 49 | Item/ItemKart | 2 | 0 | 1 | 7 |  |
| 226 | Pin | 11 | 0 | 1 | 1 |  |
| 66 | MagTurtle | 2 | 0 | 1 | 0 |  |
| 177 | Item/ItemWildGinseng | 48 | 0 | 1 | 0 |  |
| 289 | Item/WinningDart | 21 | 0 | 1 | 0 |  |
| 290 | Item/WinningScanner | 24 | 0 | 1 | 0 |  |
| 334 | Item/ItemJewelHammer | 56 | 0 | 1 | 0 |  |
| 125 | TreasureGold | 97 | 0 | 0 | 30 |  |
| 128 | TreasurePocketGold | 97 | 0 | 0 | 26 |  |
| 53 | MonS2Tank | 2 | 0 | 0 | 24 |  |
| 124 | TreasureSilver | 97 | 0 | 0 | 24 |  |
| 17 | Item/ItemHPNeedle | 1 | 0 | 0 | 22 |  |
| 123 | TreasureBronze | 97 | 0 | 0 | 22 |  |
| 127 | TreasurePocketSilver | 97 | 0 | 0 | 18 |  |
| 84 | BonusMapReward4 | 110 | 0 | 0 | 12 |  |
| 326 | ItemVampireCape | 1 | 0 | 0 | 8 |  |
| 327 | ItemHolyGrail | 1 | 0 | 0 | 8 |  |
| 328 | ItemHolyBible | 125 | 0 | 0 | 8 |  |
| 329 | ItemAngelWing | 126 | 0 | 0 | 8 |  |
| 330 | ItemHolyWater | 127 | 0 | 0 | 8 |  |
| 331 | ItemBat | 2 | 0 | 0 | 8 |  |
| 54 | MonS2UpgradeTank | 2 | 0 | 0 | 7 |  |
| 81 | BonusMapReward | 93 | 0 | 0 | 7 |  |
| 126 | TreasurePocketBronze | 97 | 0 | 0 | 6 |  |
| 272 | SkillThrowNuclearBomb | 60 | 0 | 0 | 6 |  |
| 19 | GoldFlag | 1 | 0 | 0 | 5 |  |
| 246 | ItemExpGold | 100 | 0 | 0 | 5 |  |
| 247 | ItemCpGold | 103 | 0 | 0 | 5 |  |
| 248 | ItemCpPocket | 103 | 0 | 0 | 5 |  |
| 18 | Item/ItemCloneBomber | 1 | 0 | 0 | 4 |  |
| 56 | Dragon | 2 | 0 | 0 | 2 |  |
| 59 | RetroFrog | 2 | 0 | 0 | 2 |  |
| 60 | RetroSnail | 2 | 0 | 0 | 2 |  |
| 82 | BonusMapReward2 | 106 | 0 | 0 | 2 |  |
| 245 | ItemExpSilver | 100 | 0 | 0 | 2 |  |
| 15 | Item/ItemBlackDevil | 1 | 0 | 0 | 1 |  |
| 77 | LucciPocketRandom | 99 | 0 | 0 | 1 |  |
| 83 | BonusMapReward3 | 108 | 0 | 0 | 1 |  |
| 232 | Item/ItemMonS2Spanner | 12 | 0 | 0 | 1 |  |
| 233 | Item/ItemCan | 12 | 0 | 0 | 1 |  |
| 244 | ItemExpBronze | 100 | 0 | 0 | 1 |  |
| 267 | Item/GolemKey | 79 | 0 | 0 | 1 |  |

3 tên item trong file map không khớp tên nào trong `Attr/ItemAttr` sau khi đã ghép không phân biệt hoa thường và tiền tố `Item`; số trong ngoặc là số map có trong bảng rơi/đặt sẵn: !!StartMapScript!! (0/1), 7 (1/0), Item/ItemWinningNiddle (1/0).

### Toàn bộ dòng item theo nhóm

Nhóm là trường `group` trong bảng. Tên nhóm theo PROTOCOL khi đã biết; để trống khi chưa có nguồn.

| Nhóm | Số dòng | Được map dùng | Ý nghĩa đã biết | Ví dụ |
|---|---|---|---|---|
| 1 | 57 | 22 | Chỉ số và năng lực nhặt trong trận (Bomb/Fire/Roller, Shoe, Punch, MaxPower…) | ItemBomb, ItemFire, ItemRoller, ItemShoe… |
| 2 | 28 | 25 | Phương tiện cưỡi và biến hình | ItemAirship, ItemOwl, ItemTurtle0, ItemTurtle1… |
| 80 | 26 | 0 |  | AIBattleEventNpcReward, MonsterEventNpcReward, BigMonsterReward, DarkEventReward… |
| 23 | 12 | 1 | Kim và biến thể | ItemNiddle, ItemNeedleG, ItemNeedleC, ItemNeedleSP… |
| 99 | 12 | 7 | Tiền Lucci rơi trong trận | LucciBronze, LucciSilver, LucciGold, LucciPocketBronze… |
| 105 | 12 | 0 |  | BingGo01, BingGo02, BingGo03, BingGo04… |
| 46 | 11 | 0 |  | ItemSJNeedle1, ItemSJNeedle2, ItemSJNeedle3, ItemSJNeedle4… |
| 98 | 11 | 0 |  | Alphabet01, Alphabet02, Alphabet03, Alphabet04… |
| 29 | 10 | 1 | Khiên (Shield) | ItemShield, ItemShieldC, ItemShieldSP, ItemShieldFD… |
| 48 | 9 | 1 |  | ItemWildGinseng, ItemMyriadGinseng, ItemWildGinsengC, ItemMyriadGinsengC… |
| 11 | 7 | 7 |  | ItemBond, ItemTrap, Pin, Banana1… |
| 12 | 7 | 2 |  | ItemCanFD, ItemSpannerFD, ItemSpanner, ItemMonS2Spanner… |
| 26 | 7 | 1 | Lò xo (Spring) | Spring, SpringG, SpringC, SpringSP… |
| 30 | 7 | 1 | Cảm biến (Sensor) | ItemSensor, ItemSensorG, ItemSensorC, ItemSensorSP… |
| 47 | 7 | 1 |  | ItemRescueCoin, ItemRescueCoinC, ItemRescueCoinSP, ItemRescueCoinPCRoom… |
| 21 | 6 | 2 | Phi tiêu (Dart) | ItemDart, ItemDartC, ItemDartSP, ItemDartFD… |
| 60 | 6 | 1 |  | Skill00, Skill01, Skill02, SkillSnow00… |
| 97 | 6 | 6 |  | TreasureBronze, TreasureSilver, TreasureGold, TreasurePocketBronze… |
| 119 | 6 | 0 |  | MonWarsMaterial01_Octopus01, MonWarsMaterial02_Penguin01, MonWarsMaterial03_Seal01, MonWarsMaterial04_Turtle01… |
| 49 | 5 | 0 |  | ResurrectionItem, ResurrectionItemAIBattle, ResurrectionItemPCRoom, ResurrectionItemAIBattlePCRoom… |
| 50 | 5 | 0 |  | ItemStrengthPotionRPGType1Lv1, ItemStrengthPotionRPGType1Lv2, ItemStrengthPotionRPGType1Lv3, ItemStrengthPotionRPGType1Lv4… |
| 24 | 4 | 2 | Máy quét (Scanner) | ItemScanner, ItemScannerFD, WinningScanner, WinningScannerC |
| 100 | 3 | 3 |  | ItemExpBronze, ItemExpSilver, ItemExpGold |
| 121 | 3 | 0 |  | EscapeMaze_MoonKey, EscapeMaze_StarKey, EscapeMaze_SunKey |
| 4 | 2 | 0 |  | Basket00, MarblePot00 |
| 13 | 2 | 0 |  | ItemMotorFD, ItemMotor |
| 25 | 2 | 1 | Thuốc (Potion) | ItemPotion, ItemPotionFD |
| 27 | 2 | 1 | Khoan (Drill) | Drill, DrillRPG |
| 28 | 2 | 1 | Bom điều khiển từ xa | RemoteBomb, RemoteBombG |
| 31 | 2 | 1 |  | Nitro, ItemNitroFD |
| 32 | 2 | 1 |  | ItemFuel, ItemFuelFD |
| 41 | 2 | 0 |  | ItemFakeHedgehog, ItemFakeHedgehogFD |
| 42 | 2 | 0 |  | ItemSuperHedgehog, ItemSuperHedgehogFD |
| 51 | 2 | 0 |  | ItemTeamShield, ItemTeamShieldC |
| 52 | 2 | 0 |  | ItemSnowSensor, ItemSnowSensorC |
| 53 | 2 | 0 |  | ItemAutoShield, ItemAutoShieldC |
| 91 | 2 | 0 |  | BonusBigReward, BonusBigReward02 |
| 94 | 2 | 0 |  | AIBattleQuestReward, AIBattleQuestNote |
| 101 | 2 | 0 |  | BonusLargeGourd00A, BonusLargeGourd00B |
| 103 | 2 | 2 |  | ItemCpGold, ItemCpPocket |
| 123 | 2 | 0 |  | ItemWaterGun, ItemWaterGun10 |
| 124 | 2 | 0 |  | ItemIceGun, ItemIceGun10 |
| 3 | 1 | 1 |  | ItemMonsterStop |
| 22 | 1 | 1 | Bình oxy | ItemOxygen |
| 33 | 1 | 1 |  | HollowBomb |
| 34 | 1 | 0 |  | ItemCookieRoller |
| 45 | 1 | 0 |  | ItemCpNiddle |
| 54 | 1 | 0 |  | ItemCoupleAutoShield |
| 55 | 1 | 0 |  | ItemJavelin |
| 56 | 1 | 1 |  | ItemJewelHammer |
| 79 | 1 | 1 |  | GolemKey |
| 89 | 1 | 0 |  | RPGCollectItem |
| 93 | 1 | 1 |  | BonusMapReward |
| 96 | 1 | 0 |  | AIBattleReward |
| 102 | 1 | 0 |  | BonusLargeGourd01 |
| 104 | 1 | 0 |  | PresentBoxEvent |
| 106 | 1 | 1 |  | BonusMapReward2 |
| 107 | 1 | 0 |  | NpcCaptainReward |
| 108 | 1 | 1 |  | BonusMapReward3 |
| 109 | 1 | 0 |  | BonusMapSweetCandy |
| 110 | 1 | 1 |  | BonusMapReward4 |
| 111 | 1 | 0 |  | WinningHeart |
| 113 | 1 | 0 |  | SnowFastCast |
| 114 | 1 | 0 |  | ItemCannedHam |
| 115 | 1 | 0 |  | ItemTowerReward |
| 116 | 1 | 1 |  | ItemShower |
| 117 | 1 | 1 |  | ItemShine |
| 118 | 1 | 1 |  | ItemThorny |
| 120 | 1 | 0 |  | NpcCaptain2Reward |
| 122 | 1 | 0 |  | ItemLifejacket |
| 125 | 1 | 1 |  | ItemHolyBible |
| 126 | 1 | 1 |  | ItemAngelWing |
| 127 | 1 | 1 |  | ItemHolyWater |
| 1000 | 1 | 0 |  | HCHint |

### Phương tiện — 28 mục `Ship/*`

Airship, Owl, Turtle0, Turtle1, AirshipBroken, Octopus0, Octopus1, RailCar, Chicken, Kart, Kart, Tank, Larva, MonS2Tank, MonS2UpgradeTank, Shark, Dragon, Rabbit, ItemBat, Rudolph, RetroFrog, RetroSnail, RetroPig, RetroDuck, RetroChicken, BalansaCap, RetroMouse, MagTurtle.

## Nhân vật

Mỗi dòng `Attr/BomberAttr` có 11 số. Theo `boomkr-re/docs/kr-data.md`, đây là chỉ số nhân vật (bom, tầm nổ,
tốc độ và giá trị tối đa); ánh xạ từng cột cho KR vẫn theo nguồn đó, tool này không xác nhận lại.

| Tiền tố | Số dòng | Tên |
|---|---|---|
| C | 34 | C00, C01, C03, C05, C04, C06, C02, C07, C14, C15, C16, C23, C24, C10, C11, C12, C13, C17, C08, C09, C22, C26, C18, C19, C20, C21, C25, C27, C28, C10, C29, C15, C08, C26 |
| N | 28 | N01, N02, N03, N04, N05, N06, N07, N08, N09, N10, N11, N12, N13, N14, N15, N16, N17, N18, N19, N20, N21, N22, N23, N24, N25, N26, N27, N28 |
| Z | 2 | Z01, Z02 |
| B | 2 | B01, B02 |
| W | 1 | W01 |

## Mode

Pack không có một bảng liệt kê mode; dưới đây là hai danh sách đặt cạnh nhau, **chưa ghép từng dòng**.
Mã mode trên wire (trường mode khi tạo phòng) và luật thắng của từng mode phải lấy từ RE client.

- **Theo wiki (trang 게임 모드):** 일반모드 (thường), 몬스터모드 (monster), 협공배틀 (đánh phối hợp), bốn kiểu
  게임타입: 부활대전 (hồi sinh), 고슴도치 (nhím), 대장잡기 (bắt thủ lĩnh), 시한폭탄 (bom hẹn giờ); 팝케이드,
  혼자놀기 (chơi một mình); hai mode sự kiện cũ 눈싸움 và 슈퍼파이터.
- **Nhóm giao diện theo mode trong `FxContents`:** AIBattle (10), Antarctic (15), CAHero (69), CAPass (9), CharacterSelect (34), CharacterSelectAll (33), ConfusedFight (50), DIYBomb (9), DailyMission (22), HoneyButterChip (18), IamBoss (4), IamWriggle (7), ItemCollection (27), Ladder (9), LuckyCard (5), MagicMarket (57), MineExplore (8), RPG (134), RPGMyInfo (29), SandBox (79), SnowyChamp (6), UXCAJJangRatingMark (61), UXMatching (163), WeeklyLadder (12).
- **Pack riêng:** FxHiddenCatch (Hidden Catch), FxMonster (quái mùa 1 và mùa 2, tàu).
- **Theo số map ở trên:** map đấu thường theo theme 169, biến thể của map đấu 144, nhiệm vụ và mode riêng 122, monster mode 368, rpg và ca hero 124, hidden catch 37, màn thưởng 43, sảnh, cửa hàng, tutorial, map tuỳ biến 36, chưa phân nhóm 108.

## Script map

770 map có `<mapscript>`. Số trong ngoặc là số map dùng. Script do client đọc; phần nào cần server
tham gia (đồng bộ trigger, quái, phần thưởng) chưa được đo.

- **Thẻ:** mapscript (770), ActionList (507), TriggerTable (506), Trigger (500), StageClearScript (499), InitInfo (477), ScriptObject (477), Time (476), ScriptObjects (474), Action (441), mapscriptoption (433), action (321), Item (128), time (118), ItemTable (89), ItemTables (89), ItemSkillAttrTable (84), MapItemConfig (84), OverrideInitInfo (82), ScriptObjectTemplate (82), ScriptObjectTemplates (81), MonDieRewardProb (71), Move00 (71), MoveMap (71), RPGScript (71), GameMapItemAttrTable (61), state (58), MapScriptOption (44), bomb (43), wave (43), Move01 (36), Domain (35), popup (35), GlobalScript (34), item (31), Domains (28), Match (28), Matches (28), Sweeping (28), WriggleScript (28).
- **Hàm của Action:** free (342), line (213), playandwait (192), absolute2 (128), absolute (125), waitonly (122), relative (77), rectangle (62), directiondest (40), autokick (29), fallingrock (28), nuclearbomb (27), playonly (27), roll (26), laser (17), directattack (12), fallingrocksnow (12), bombmissile (11), avalanchesnow (9), absolutex (9), createboxmon (5), fallingbomb (5), direction (2).
- **Loại Trigger:** default (133), selflocked (83), bomberlocked (76), selfdamaged (73), bomberdie (62), die (59), selfreleased (53), selfdie (43), bossanger (39), approach (33), lastmonleft (26), init (23), locked (7), appear (6), failaction (4), firstmondie (4), bosslocked (3), classalldie (2), createdefault (1), aibomberdied (1), aibomberlocked (1), aibomberreleased (1), bomberdied (1), bombereatneedle (1), bombereatpunch (1), bombereatshield (1), bombereatshoe (1), bombereatskeleton0 (1), bombereatskeleton1 (1), bombereatspring (1).

## Điền cột "server phải làm gì"

Mỗi họ map và mỗi nhóm item cần một ca chạy thử trên client thật để xếp vào một trong ba loại:

- **A — chỉ cần dữ liệu:** server gửi đúng ID và bảng item, client tự lo phần còn lại.
- **B — dùng packet đã có:** nhặt, tranh item, báo sống/chết như đã đo; kiểm một ca mỗi nhóm.
- **C — cần packet hoặc trạng thái mới:** phải RE trước khi làm.

Chưa họ nào được xếp loại ngoài `ExVillage` (ExVillage01 đã chạy native theo PLAN). Thứ tự thử đề xuất:
map đấu thường không script trước, sau đó các item được nhiều map dùng nhất mà chưa đo.
