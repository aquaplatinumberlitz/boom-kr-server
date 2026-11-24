# Full F1: phân cụm cơ chế KR 00967

Batch `p95-f1-full-ida` bao phủ **331 kind ngoài lớp A**, **33 hàng nhân vật lớp C**, **23 hàng Ship ngoài baseline F1-d** và **133 map**. Đây là độ phủ của danh sách cần làm. [JSON bằng chứng](../evidence/content/f1-full-ida-clusters.json) giữ từng kind, row/selector, map code, hash, VA và trạng thái. `confirmed` chỉ xác nhận phát hiện tĩnh ghi ở dòng đó; `candidate` là nhóm để kiểm tiếp; `unknown` là cạnh chưa nối. Không nâng lớp A hoặc kết luận về quyền xử lý của server gốc từ batch này.

Nguồn chính là pack KR 00967 đã kiểm hash và ảnh unpack `cf72da11…` (image base `0x400000`, RVA = VA trừ image base). Tái dùng [item p93](../protocol/items-stats.md#f1b-next-items), [nhân vật F1-c](../protocol/auth-session.md#f1-c-character-catalog), [xe F1-d](../protocol/vehicles.md), [địa hình round 3](../protocol/terrain.md#f1a-ida-round3) và [khảo sát mapscript](../protocol/load-phase.md#f1e-map-scripts). Hai Attr khớp bản trích ở `boomkr-re/data/kr/`; `boomkr-re/docs/kr-data.md`, `kr-content.md` và `boom-re/docs/{items,map}.md` giúp đối chiếu bố cục. Kind, selector, VA và giới hạn dưới đây đều lấy từ KR.

Lượt IDA tĩnh tiếp theo đã nối wire cho Dart `138/172`, vật đặt group 11 `139/173` và `126/168`, các item Ctrl chủ động,
độ rộng khóa nhân vật trong `Start 121`/`C→S 79` và các nhánh Ship riêng. Xem [opcode phụ trợ F1](../protocol/f1-aux-opcodes-and-classc.md)
và [bằng chứng](../evidence/content/f1-aux-opcodes-and-classc-ida.json).

## Item: đường nhặt và đường dùng

[Catalog](classification.md) có 75 dòng lớp C: **74 dòng nhóm số chứa 328 kind** và **một token map chưa ghép** (`Item/ItemWinningNiddle`). Thêm Shoe, Punch, Needle lớp B thành 331 kind ngoài A. Trong 328 kind lớp C, 138 nằm ở 13 dòng nhóm có nhãn F1; catalog nối trực tiếp **49 tên** với placement/drop dương F1. Con số 98 tên map candidate tính cả map ngoài F1. Các mẫu số khác nhau nên batch giữ toàn bộ 328 kind, kể cả biến thể chưa có dấu vết F1. Tên `Item/ItemWinningNiddle` chưa được quy thành kind 287 hay 293.

**`kr_static/confirmed`:** `1190A40` kiểm guard, đọc `ItemAttr+160` qua `B81B40`. Nếu khác 0, nó gọi `vf+0x18(kind,field160,0)` trước switch pickup; dòng có switch fallback trả 0 vẫn có thể đi đường này. Với `+160=0`, `1192700` chọn nhánh theo group và group 1 gọi `1191840` cho bốn tail đầu. Nhánh dùng `B4D999 → 1190C20` có guard kind, amount và virtual gate trước khi chọn group. JSON giữ VA của cả hai switch cho từng kind.

| Cụm | Kind ngoài A / lớp C / lớp C có tên F1 | Điểm tựa và giới hạn |
|---|---:|---|
| Pickup stat, buff, hiệu ứng | 17 / 15 / 11 | `11927AE → 1191840`. Các effect 7/8/9/10/11/12/15/19 tách Hollow, Skeleton, MoonWalk, SuperMan, GreenDevil, Skate/RedDevil, Camouflage theo pack; slot được xác nhận, thời gian và hiệu lực lên peer chưa chốt. |
| Ctrl cứu, dò, nhảy và biến thể | 59 / 58 / 5 | Needle group 23 `1190FCC`, Scanner 24 `1191003`, Spring 26 `1191063`; có các nhánh khác cho Oxygen/Potion và các biến thể cứu. Chỉ kind 145 và 146 có `kr_runtime/confirmed` cho `127/sub0` ở nguồn cũ. Kind khác dùng chung writer là ứng viên. |
| Vật đặt group 11 | 7 / 7 / 7 | `1190D4F → vf+0x08 → B4C1F0` ở lớp cơ sở, rồi kiểm ô và gọi `B9DEF0`. Selector của Bond/Trap/Banana/Pin là 1/2/3/4; ba cờ còn lại không đồng nhất. Chưa xác nhận kích hoạt và owner khi người khác bước lên. |
| Chủ động bật khiên, sensor, buff | 34 / 34 / 2 | Shield group 29 ở `11910F8`; Sensor group 30 ở `119114A`. Group 53/54 cũng gọi slot `+0x34` qua nhánh riêng. Đây là chuyên biệt ở bước use, kể cả khi hiệu ứng sau đó tồn tại như buff. |
| Bom, vũ khí, đạn ném | 24 / 24 / 5 | Dart group 21 ở `1190F62 → B4A780` tra hướng, gọi `E2C950` nếu qua guard; Drill 27, RemoteBomb 28 và HollowBomb 33 có case khác. Chưa biết đích, số lần tiêu hao và gói phát của từng họ. |
| Nhặt xe group 2 | 28 / 28 / 15 | `11928A2 → 1197900/B978B0`; selector là tail đầu. Cả năm kind 40–44 ở đây để đối chứng, không tính lại vào 23 hàng Ship bổ sung. |
| Biến hình hoặc điều khiển xe | 19 / 19 / 0 | CloneBomber/VampireCape dùng effect group 1; 12/13/31/32 và Hedgehog 41/42 có case Ctrl. Nhóm cơ chế là ứng viên; chưa buộc tất cả vào `CGOShip`. |
| Lucci, kho báu, EXP | 32 / 32 / 6 | `1192B40` và `1192BDA` dẫn group 99/97 qua `B4A230`. Các arm 98/100 trả thành công riêng. Chưa biết giá trị tiền từ server gốc, đối soát hay persistence. |
| Thu thập, phần thưởng, mode riêng | 105 / 105 / 1 | Có cả group 1 effect 99, code riêng, arm pickup trả 1 không gọi effect và các nhánh use phụ. Tên và dấu vết map phân luồng đo; khóa mode còn là ứng viên nếu chưa có guard. |
| Nhánh rỗng hoặc bị từ chối | 6 / 6 / 0 | Kind 15/16/303/304 chỉ có effect 0, `1191882` trả 1 mà không áp effect. Kind 324/325 gặp effect 31/32 tại `1191C1F` trả 0, nên group 1 không báo pickup thành công ở đường này. Chưa chứng minh mọi đường cấp kind đều vô hiệu. |

Writer `AEE809–AEE866` nối `u8/u16/u8/u32` sau prefix của sub0 rồi gửi. Dùng writer làm điểm kiểm hội tụ, vì gọi được một slot Ctrl chưa chứng minh đã tiêu hao hay gửi sub0. `S→C381` là **opcode**, không phải kind: `E68BD1 → 703A87` đặt AL=0 trước `E68C5A`; nhánh 381 chết ở build này, còn `165` vẫn là đường mount đã nối. Không có cơ sở để kết luận một mục trong pack hết hạn chỉ vì không qua switch thông thường.

## Nhân vật và phương tiện

| Cụm nhân vật lớp C | Hàng | Dấu vết |
|---|---:|---|
| `f1=3`, N01–N28 | 28 | `13500D0` nạp `BomberAttr`, đăng ký nhóm 3 riêng. `E0F770` có nhánh khác cho 4000–4030 khi đối số 5 bằng 95 và `AEDE60` bật. Chưa biết khóa nào có thể chọn trong UI trận thường. |
| `f1=4`, Z01/Z02 | 2 | Nhánh đăng ký nhóm 4; không có trong `F1_CHARACTERS`. Chưa nối key tới lớp actor hoặc skill riêng. |
| `f1=6`, B01/B02/W01 | 3 | Nhánh đăng ký nhóm 6; chưa biết selector/mode và skill cụ thể. |

Trong 67 hàng, **31** thuộc whitelist F1-c. Ba hàng trùng tên có key **4019/4025/4026** và `f1=3` nằm ngoài 31 khóa nhưng không thuộc 33 dòng lớp C của catalog; JSON giữ làm đối chứng. `1350840` tra key, `B3CC80` lấy profile và `B3D1D0` reset ở đường spawn đã biết. Việc dùng parser chung xác nhận khả năng nạp dữ liệu, chưa xác nhận lựa chọn, cap/reset hoặc `Loading79` của các key 4000/5000. [CP395](../evidence/checkpoints/records/CP395.json) là mỏ neo native cho C03/C05; byte Loading79 đã đo không xác lập cách mã hóa khóa lớn hơn 255. Không có hàng lớp C nào đủ căn cứ để nhập vào flow trận thường chỉ bằng mở whitelist.

| Cụm Ship bổ sung | Hàng | Selector | Phần cần kiểm |
|---|---:|---|---|
| Xe mặt đất, không effect Ship | 9 | 6, 7, 13, 17, 18, 20, 26, 27, 28 | Dùng chung lookup và nạp; kiểm từng guard cấp kind, hit và reset với một xe mặt đất đối chứng. |
| Cờ bay | 4 | 9, 19, 21, 22 | Cùng đường `raw[0]==2` nhưng khác danh sách effect. So với Airship trước khi dùng chung luật cản. |
| RailCar | 1 | 8 | Kind 47, tốc độ raw 2/10; nhánh phụ dùng cờ ray, đo chung với 5 map Mine. |
| Có effect hoặc nâng cấp | 8 | 11, 12, 14, 15, 16, 23, 24, 25 | `1191D20` đọc effect trong bốn vị trí đầu khi pickup. Tank 12 có nhánh riêng; `1370710` chỉ cho 14→15 qua guard thay xe của đường pickup. |
| Kart thiếu cạnh floor | 1 | 10 | Cả kind 49 và 50 trỏ selector **11**. Selector 10 có hàng đọc được; chưa loại trừ caller khác. |

`1196A70` nạp Ship stride 192, `1197900` trả hàng đầu nếu thiếu khóa. **22/23** hàng bổ sung có ít nhất một kind group 2 trỏ selector. Đây là cạnh tĩnh tới `CGOShip`, không suy ra cả 22 đều xuất hiện trong trận thường. `165 → 11928A2 → B978B0` nạp xe khi qua guard; `B73680` và `E866F0` là hai đầu `127/sub16 → 170/sub16`. Quyết định relay, hai view và reset qua ván kế vẫn cần phép thử riêng.

## Map F1-a: 34 trường hợp adapter còn từ chối

Tập này lấy theo ma trận round 3 **180/214**, kiểm lại từng map code và pack payload. Lý do hiển thị là điểm từ chối đầu tiên, một map có thể chứa nhiều tile đặc biệt.

| Cụm | Map | Kết quả tĩnh |
|---|---:|---|
| `partial_payload` | 4 | EventForest00, EventPatrit00, ExBula01/02 tới hết lưới, drop và placement rồi vấp độ dài blob. `EA1EE0 → 118FC30 → vf+0x64` là đường cursor cần bám. Ý nghĩa các đuôi nhị phân chưa biết; JSON giữ offset và byte đầu sai. |
| Tent `passable_variant_flags` | 5 | Ba cụm byte `+24/+28/+32 = 0x01000100`, flag `+44=1`; `E90A80 → E9F220/E9C520` phân biệt tia vào/rời theo hướng. Adapter chặn toàn ô không biểu diễn được. |
| Island `box_variant_flags` | 3 | `B9C760` chép byte tile +33 vào `FixedBox+0xE0`; `B9CC10 → E9EEE0` tạo vụ nổ ở ô. Quyền hit, owner và dây chuyền chưa chốt. |
| `conveyor` | 8 | `B9D8E0` đọc type nền 2 và các raw +36/+40; `B9DB70` giải mask 1/2/4/8 thành 0–3; tick `B44000` gọi `B9DBF0` tại `B44647` nếu qua guard, có phép chặn bước theo khoảng cách. Còn cần đo ưu tiên input, hướng chuyển ô và bom trên băng. |
| `mine_rail` | 5 | Cờ tile +52 qua `B9E4E0 → B44000/B46D80`; `B46940` dùng nó để chọn tốc độ Ship chính/thay thế. Chưa biết luật bắt ray, rẽ hoặc dừng. |
| `Tile42_gameplay_flags` | 8 | CAPong Red/Blue ghi word +62 là 1/2, có consumer giới hạn mode 38 trong nguồn trước. `Block03+36` đọc qua `BA1420 → B50370` khi va chạm. Chưa biết điểm và quyền kết quả. |
| `service_scope` | 1 | NXCL_Aqua01 bị adapter chặn vì có script. Pack có `StageClearScript` và JellyBombBlock00; chưa biết quan hệ giữa hai cơ chế. |

## Map F1-e và AIBattle: 99 map

Trong **95 map lớp C F1-e** có **92** blob mang marker mapscript, kể cả comment và dữ liệu XML lỗi; **3** map BigBattle không có script. Bốn map `Fm_AIBtl*` thuộc F3, cả ba blob đều rỗng. `EA1EE0` đọc root `mapscript`, tra option và gọi `EBA2D0` tại `EA4945`; factory có các consumer khác nhau. Với toàn bộ nhóm, quyền chọn tick, kết quả, phần thưởng và đồng bộ của server gốc vẫn **unknown**.

| Họ trigger hoặc dữ liệu | Map | Dấu hiệu client và giới hạn |
|---|---:|---|
| Aqua stage clear, chỉ marker | 5 | `EBA334` nhận tên, `CMapScriptStageClear` có loader riêng. Chưa biết stage clear có tác dụng gì cho blob chỉ có marker. |
| Aqua stage clear + SharkMon/Golem | 4 | `ScriptObject`, `TriggerTable`, `BomberLocked/BomberDie/Die/SelfDie`, action `absolute/absolute2`. Parser client và lớp stage clear có thật; cạnh từ đối tượng từng map tới tick là ứng viên. |
| Space hẹn giờ đổi ô | 13 | `changetile` và `position`; `6341A0` đọc node, `634B50` so counter với mốc rồi gọi action. Client có consumer thời gian; quyền trên va chạm/hit và hai view chưa biết. |
| PangLand bắn bom theo đợt | 12 | `time → wave/SprinklerBomb → bomb/state`. Client có factory và `CMapScriptSprinkler`; chưa chốt owner bom, đích RNG hoặc đồng bộ. |
| Wriggle object, domain và điểm | 14 | `EBA3F6 → 1013E00 → F9E7F0`; `Default/SelfLocked/SelfReleased` đi với `Domain/Match/Sweeping`. Client nạp lớp Wriggle; match score, combo và authority chưa biết. |
| Retro popup, Retro Forest cloud/respawn, Retro Village ride | 18 / 6 / 6 | `CMapScriptPopUp` là dấu hiệu client. Hai cụm sau có `retroscript`; ride type 60/61/62 trùng số kind group 2, chưa xác nhận lookup của `rideinfo`. |
| LovelyRoom giới hạn bom | 10 | `mapscriptoption enablebomb=0`. Cả lịch sinh Lucci và bảng item nằm trong comment ở cả 10 blob; không tính là trigger sống. |
| RescueET | 3 | Alien, BadAlien, TailAlien, Spaceship và bảng xuất hiện; không thấy arm `RescueAliensScript` trong dispatcher đã đọc. Consumer và điểm chưa biết. |
| Coloring | 1 | `ColoringPattern` có attr số không đặt trong dấu nháy, parser XML chuẩn từ chối. Chưa biết client đọc bằng parser nào hoặc có xử lý không. |
| BigBattle lưới lớn | 3 | Không có blob script; kiểm kích thước, spawn và mode trước. |
| AIBattle lưới F3 | 4 | Không có blob script. `CMapSelectAIBattleStrategy` và asset AIBattle là dấu vết flow riêng; chưa biết bên nào chọn AI, target hoặc kết quả. |

Không thấy tag portal, teleport, warp, switch, door, lever hoặc button trong 99 map của bảng. Việc không thấy tag chỉ giới hạn kết luận về blob; trigger có thể nằm trong tile hoặc code theo mode. Không dùng `EscapeMazeScript` của tập khác để gán portal cho F1.

## Thứ tự batch đề xuất

1. **Ctrl và vật đặt.** Dùng Needle/Scanner cùng input đối chứng và giữ cả hai phiên qua sub0. Thử một đại diện mỗi slot; với group 11, phép thử chung là ô trống/ô bị cản và một lần peer đi qua cho mỗi selector. Tách writer, tiêu hao, spawn và tác dụng lên peer.
2. **Terrain theo loại va chạm.** Đối chiếu Tent hai trục và Island Block30–32 với ô control trước. Gom RailCar/Mine vào phép đo tốc độ xe, rồi đo băng thẳng/góc và vật thể bị đẩy. Phân tích riêng bốn đuôi payload và giữ CAPong sau phép kiểm mode 38.
3. **Roster và xe.** Lấy một key mỗi cụm 120+/150+/3000+ của F1-c làm đối chứng Start121/Loading79/cap/reset trước khi thử bất kỳ key 4000/5000. Với xe, chọn đại diện mỗi cụm có cạnh group 2, giữ hai view qua `165`, hit, lọc riêng `170/sub16` và sang ván kế.
4. **Script theo consumer.** Dùng một map popup làm đối chứng, rồi Space, PangLand và Wriggle cho tick/tác động thực. Tiếp đến Aqua, Retro và RescueET theo cạnh còn thiếu. Giữ Coloring và AIBattle riêng cho tới khi nối được consumer hoặc flow mode.
