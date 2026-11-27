# Kiểm kê và đặc tả sự kiện theo mùa, chăm sóc pet F6 mở rộng

Build `KR00967`: **19 hệ thống `in_scope`**, **25 phân hệ `roadmap_only`** và **1 mục `excluded_policy`**. Các phân hệ là lát cắt công việc: chúng chi tiết hóa 7 nhóm bổ sung và một số hệ thống sẵn có, đồng thời bao phủ 5 họ `F6?` (5 map). Con số khoảng 25 trong PLAN là ước lượng, không phải số hàng mới của bảng xếp loại.

Nguồn: [xếp loại](classification.json), [catalog F6-0](f6-events-catalog.md), [policy F6-1](f6-events-policy.md), [policy Rust](../../rust/crates/kr-service/src/f6_policy.rs) và [lộ trình F6](../PLAN.md#full-game-roadmap). [JSON](../evidence/content/f6-extended-events-spec.json) giữ tên hàng gốc, locator và SHA-256 đầu vào. Catalog F6-0 giữ snapshot xếp loại trước khi PR #713 mở rộng lộ trình; đối chiếu tập hệ thống theo từng hàng. Năm mục `related_candidate` của catalog vẫn thuộc F2, F3 hoặc F4.

Tên tài nguyên và số map là metadata tĩnh (`kr_static`); các nhóm lộ trình và liên hệ theo tên là `candidate`. Quota, khóa chống lặp và gate bên dưới là `design`. Chưa chạy Wine cho đặc tả này. Opcode, layout, giá, thưởng, tỷ lệ gốc và trạng thái hoạt động trên KR 00967 đều là `unknown`.

## Mặc định và bất biến áp dụng chung

Mặc định `enabled: false`, `start_utc: null`, `end_utc: null` và hai `rate_caps` đều bằng 0. JSON này là đặc tả, không phải profile hợp lệ để nạp vào Rust.

- Mỗi phân hệ cần opt-in riêng, lịch UTC [start_utc, end_utc), giới hạn dương theo tài khoản/giờ (tối đa 60) và toàn server/phút (tối đa 600), và quota nghiệp vụ ở bảng dưới. Thiếu cấu hình, giới hạn bằng 0, chưa đạt gate, ngoài lịch hoặc không kiểm được quota thì từ chối trước khi đổi trạng thái. Bật hệ thống cha không bật phân hệ hay map.
- Server lấy account, mùa, chu kỳ và điều kiện đủ từ trạng thái có thẩm quyền; không dùng đồng hồ hoặc ID tài khoản do client khai. Các ngưỡng dương ngoài ràng buộc nêu rõ chưa được chọn và không lấy từ KR gốc.
- Khóa chống lặp có phần chung (system_id, account_id, season_id) và phần riêng trong bảng. ID thao tác/lượt phải do server cấp hoặc gắn với một sự kiện có thẩm quyền, không cấp lượt mới khi gửi lại cùng hành động. Cùng khóa và dữ liệu trả kết quả đã lưu, kể cả kết quả ngẫu nhiên; cùng khóa nhưng dữ liệu khác thì từ chối.
- Kiểm quyền, quota nghiệp vụ, ghi khóa, trừ đầu vào, cấp quà và đổi trạng thái bền vững phải nguyên tử; lần gửi lại không trừ quota nghiệp vụ hay cấp quà lần nữa. Giới hạn lưu lượng vẫn áp dụng cho lần gửi lại. Chuyển cảnh chỉ khóa theo phiên hiện tại và không ghi thưởng.
- `no-real-money-monetization`: Không nạp tiền, bán lượt, thanh toán bằng tiền thật, rút tiền hay giao dịch có giá trị ngoài game. Trao đổi có giá cố định chỉ dùng tài sản trong game không quy đổi được.
- `no-gambling-for-money`: Bốc thăm chỉ dùng lượt miễn phí hoặc vật phẩm không mua, chuyển nhượng hay quy đổi được. Quà ngẫu nhiên cũng không được quy đổi, đem bán hoặc dùng làm cược.

## Các hệ thống trong policy hiện tại

Các tên dưới đây là đúng tập `SYSTEM_NAMES` của Rust, được đối chiếu trong test. Rust kiểm opt-in và lịch (bao gồm cả hai mốc); tài liệu policy mô tả khoảng loại mốc cuối. Khoảng nửa mở ở đặc tả này là đề xuất cho phần mở rộng. Việc thực thi các quota và khóa riêng bên dưới thuộc handler tương lai.

| Hệ thống `in_scope` | Quota nghiệp vụ đề xuất | Phần riêng của khóa chống lặp |
|---|---|---|
| CAPass | Giới hạn cập nhật tiến độ và số mốc nhận theo mùa. | ID kết quả đã chốt cho tiến độ; ID nhánh và mốc cho quà. |
| Câu cá | Giới hạn lượt câu và lượt đổi cá theo ngày. | ID lượt câu hoặc giao dịch đổi do server cấp. |
| DIYBomb | Giới hạn lượt ghi thay đổi theo ngày. | ID thao tác server và phiên bản vật phẩm đích. |
| DTT | Giới hạn lượt đổi theo ngày và lượng hàng theo mùa. | ID giao dịch đổi do server cấp. |
| Exchange | Giới hạn lượt đổi mỗi ngày và lượng đổi theo công thức. | ID giao dịch đổi do server cấp. |
| GrowSprout | Giới hạn lượt tăng tiến độ và nhận quà theo ngày. | ID nguồn tiến độ hoặc ID mốc nhận quà. |
| HoneyButterChip | Giới hạn lượt tham gia và cấp quà theo ngày. | ID lượt tham gia do server cấp. |
| LineFriends | Giới hạn số lần nhận theo mùa. | ID đợt phát và mục quà. |
| LuckyCard | Giới hạn lần lật theo ngày và tổng quà theo mùa. | ID thẻ hoặc lượt lật; giữ kết quả đã ghi. |
| MagicMarket | Giới hạn lượt chế tạo theo ngày và theo công thức. | ID đơn chế tạo; gắn phiên bản công thức. |
| MineExplore | Giới hạn lượt tham gia và nhận quà theo ngày. | ID lượt do server cấp và ID mốc tương ứng. |
| MonWars | Giới hạn đóng góp và nhận quà theo ngày. | ID nguồn đóng góp hoặc mốc nhận quà. |
| Nhiệm vụ | Giới hạn lần nhận theo chu kỳ nhiệm vụ. | ID nhiệm vụ, chu kỳ và mốc; ghi nguồn tiến độ một lần. |
| Partners | Giới hạn lần nhận theo chiến dịch. | ID chiến dịch và mục quà. |
| Pet | Giới hạn số lần đổi trạng thái mỗi pet theo ngày. | ID pet và phiên bản trạng thái trước thao tác. |
| RetroWriggle | Giới hạn lượt tham gia và nhận quà theo ngày. | ID lượt hoặc mốc do server cấp. |
| SelectionBox | Giới hạn lượt mở theo ngày và một lần mỗi hộp. | ID thực thể hộp; không đổi lựa chọn sau khi chốt. |
| Sự kiện TakeCareOfEvent | Giới hạn đóng góp theo ngày và số mốc nhận theo mùa. | ID nguồn đóng góp hoặc mốc nhận quà. |
| Sự kiện theo tiền tố Event | Giới hạn tham gia theo ngày và quà theo từng sự kiện. | ID sự kiện và ID lượt tham gia hoặc mốc quà. |

## Phân hệ mở rộng

Mỗi dòng có ID đặc tả riêng và chưa có khóa cấu hình trong policy Rust. `Dwarf/libcef` là phụ thuộc chung F4/F6, không bật tính năng theo mùa. `LobbyPet` giữ nhãn `F5?` và không được tính vào F6.

| Phân hệ `roadmap_only` | Hàng xếp loại | Quota nghiệp vụ đề xuất | Phần riêng của khóa chống lặp | Probe Wine riêng |
|---|---|---|---|---|
| Chăm sóc pet cơ bản | `system`: Chăm sóc pet (애정도, 봉인, 듀얼펫) | Giới hạn lượt chăm sóc theo pet và ngày. | ID pet và ID lần chăm sóc; không trừ vật phẩm lần hai. | Dùng một lượt chăm sóc, mở lại UI và xác nhận trạng thái. |
| Độ thân thiết pet | `system`: Chăm sóc pet (애정도, 봉인, 듀얼펫) | Giới hạn điểm tăng theo pet và ngày; chặn tăng quá trần. | ID pet và sự kiện sinh điểm; cùng nguồn chỉ cộng một lần. | So sánh trước và sau một nguồn điểm hợp lệ và một nguồn bị từ chối. |
| Niêm phong và mở niêm phong pet | `system`: Chăm sóc pet (애정도, 봉인, 듀얼펫) | Giới hạn lượt niêm phong và mở theo pet và ngày. | ID pet, phiên bản và hướng chuyển; giữ một chủ sở hữu hợp lệ. | Niêm phong rồi mở; thử mở lại trạng thái đã mở. |
| Nâng bậc pet | `system`: Chăm sóc pet (애정도, 봉인, 듀얼펫) | Giới hạn lượt nâng theo pet và ngày; chỉ cho phép cặp bậc đã khai báo. | ID pet, phiên bản và bậc đích; nguyên liệu và bậc cập nhật trong một giao dịch. | Quan sát một lần nâng và một lần thiếu nguyên liệu. |
| Trang bị hai pet | `system`: Chăm sóc pet (애정도, 봉인, 듀얼펫) | Giới hạn đổi trang bị theo ngày; tối đa hai ô trong thiết kế hai pet. | ID thao tác và phiên bản bộ trang bị; cùng pet không chiếm hai ô. | Trang bị hai pet, thử trùng hoặc vượt ô, rồi đăng nhập lại. |
| Pet tự dùng Khiên hoặc Kim | `system`: Chăm sóc pet (애정도, 봉인, 듀얼펫) | Giới hạn theo pet và trận; tuân thủ RoomRules và tồn kho. | ID trận, sự kiện kích hoạt và pet; tối đa một lần dùng cho cùng sự kiện. | Quan sát một kích hoạt hợp lệ và một tình huống bị RoomRules từ chối. |
| Điểm danh | `system`: Điểm danh và quà quay lại (출석, 귀환 스탬프) | Tối đa một dấu điểm danh theo ngày UTC và lần nhận theo từng mốc mùa. | Ngày UTC hoặc ID mốc; đăng nhập lại không cấp lại. | Kiểm hai lần đăng nhập cùng ngày và qua mốc đổi ngày UTC. |
| Tem và quà quay lại | `system`: Điểm danh và quà quay lại (출석, 귀환 스탬프) | Giới hạn dấu theo đợt đủ điều kiện và số mốc nhận theo chiến dịch. | ID đợt đủ điều kiện và mốc; đăng nhập đồng thời không tạo đợt thứ hai. | Đối chiếu tài khoản đủ và không đủ điều kiện qua một lần đăng nhập lại. |
| Bingo sự kiện | `system`: Mini-game sự kiện (빙고, 주사위, 퀴즈) | Giới hạn lượt đánh dấu theo ngày và quà theo bảng. | ID bảng, ô hoặc hàng thưởng; mỗi ô và mốc thưởng chỉ ghi một lần. | Đánh dấu một ô, hoàn tất một mốc và thử nhận mốc lại. |
| Xúc xắc sự kiện | `system`: Mini-game sự kiện (빙고, 주사위, 퀴즈) | Giới hạn lượt gieo theo ngày và tiến độ theo bảng. | ID lượt gieo; phát lại giữ nguyên mặt và vị trí. | Gieo một lượt rồi thử gửi lại cùng lượt. |
| Câu đố sự kiện | `system`: Mini-game sự kiện (빙고, 주사위, 퀴즈) | Giới hạn số lần nộp mỗi phiên câu hỏi và tổng lượt theo ngày. | ID phiên câu hỏi và lượt nộp; điểm và quà chỉ ghi một lần. | Nộp một câu đúng, một câu sai hoặc quá hạn và quan sát phản hồi. |
| Tiến độ và nhận quà nhiệm vụ ngày | `system`: Nhiệm vụ | Giới hạn đóng góp hợp lệ theo ngày UTC và một quà mỗi nhiệm vụ trong chu kỳ. | ID nguồn tiến độ hoặc ID nhiệm vụ và ngày nhận; không cộng lại khi phát lại kết quả trận. | Hoàn thành một điều kiện, nhận một quà và kiểm biên đổi ngày. |
| Lượt câu và đổi cá | `system`: Câu cá | Giới hạn lượt câu và lượt đổi theo ngày; kiểm tồn kho trước khi đổi. | ID lượt câu hoặc đơn đổi; không bốc lại kết quả hoặc đổi lại cùng cá đã tiêu. | Câu một lượt, kiểm túi đồ và đối chiếu một lượt đổi cá. |
| Map CAPassShopMap | `map`: CAPassShopMap | Giới hạn chuyển cảnh theo phiên và một lần vào đang mở. | ID phiên và lượt chuyển; tải lại không tự nhận quà CAPass. | Thử vào, mở UI, rời và quay lại mà không nhận thưởng. |
| Map DailyMissionShop | `map`: DailyMissionShop | Giới hạn chuyển cảnh theo phiên và một lần vào đang mở. | ID phiên và lượt chuyển; tải lại không đặt lại tiến độ nhiệm vụ. | Thử vào, mở UI nhiệm vụ và rời trong khi giữ nguyên tiến độ. |
| Map LobbySquareFishingTest | `map`: LobbySquareFishingTest | Giới hạn chuyển cảnh theo phiên; vào map không tiêu lượt câu. | ID phiên và lượt chuyển; không sinh cá hay thưởng từ lần vào lặp. | Xác định client có vào được không và có mở màn câu cá hay không. |
| Map MagicMarketmap | `map`: MagicMarketmap | Giới hạn chuyển cảnh theo phiên và một lần vào đang mở. | ID phiên và lượt chuyển; vào lại không tạo thêm đơn chế tạo. | Thử vào, mở UI, hủy trước khi giao dịch và rời map. |
| Map SelectionBox | `map`: SelectionBox | Giới hạn chuyển cảnh theo phiên và một lần vào đang mở. | ID phiên và lượt chuyển; không mở hộp khi tải lại map. | Thử chuyển cảnh và kiểm map có thực sự mở UI chọn hộp hay không. |
| Luồng lật thẻ LuckyCard | `system`: LuckyCard | Giới hạn lật theo ngày; tối đa một lần mỗi thẻ và có trần quà theo mùa. | ID thẻ; lưu lượt lật và kết quả trước khi phản hồi. | Lật thẻ với vật phẩm thử miễn phí rồi thử lật lại. |
| Luồng chế tạo MagicMarket | `system`: MagicMarket | Giới hạn chế tạo theo công thức và ngày; kiểm lượng nguyên liệu tối đa. | ID đơn và phiên bản công thức; tiêu nguyên liệu và cấp đồ trong cùng giao dịch. | Xem trước, chế tạo một món, thử thiếu nguyên liệu và đối chiếu túi đồ. |
| Chợ sách phép | `system`: Chợ sách phép (마법거래소) | Giới hạn tin đang mở và lượt đăng, mua, hủy theo ngày. | ID tin và phiên bản; giao dịch chỉ cho một người mua, không vừa bán vừa trả đồ. | Hai client thử đăng, mua hoặc hủy; đối chiếu quyền sở hữu ở hai phía. |
| Bốc thăm và đổi đậu may mắn | `system`: Bốc thăm và đậu may mắn (뽀끼뽀끼, 럭키빈) | Giới hạn lượt bốc, lần đổi và tổng quà; chỉ dùng chìa khóa và đậu miễn phí không quy đổi. | ID lượt bốc hoặc đơn đổi; lưu kết quả bốc cùng lần tiêu chìa khóa. | Thử một lượt bốc và một lượt đổi bằng vật phẩm thử miễn phí. |
| Đập trứng, ngọc phép và hộp quà | `system`: Các kiểu bốc thăm (알쏭달쏭 쾅쾅, 마법구슬, 선물상자) | Giới hạn theo loại lượt mở và ngày, cùng trần quà cho cả mùa. | ID thực thể hoặc lượt mở và loại; phát lại nhận cùng kết quả đã lưu. | Kiểm riêng UI của từng loại trước khi thử một lượt mở miễn phí của loại đó. |
| Hợp thành vật phẩm | `system`: Hợp thành và dừng thời gian (합성, 타임스톱) | Giới hạn số lần hợp thành theo ngày và theo công thức. | ID giao dịch và phiên bản các vật phẩm nguồn; không tái sử dụng nguyên liệu đã tiêu. | Quan sát xem trước, một lần hợp thành và một lần dùng nguyên liệu không hợp lệ. |
| Dừng thời gian vật phẩm | `system`: Hợp thành và dừng thời gian (합성, 타임스톱) | Giới hạn số lần đổi hạn theo ngày và số lượt mỗi vật phẩm. | ID vật phẩm và phiên bản hạn dùng; phát lại không gia hạn thêm. | Đo hạn trước và sau thao tác, kể cả trường hợp đã đổi hạn. |

| Họ map `F6?` | Map | Script chưa rõ / tổng |
|---|---:|---:|
| `CAPassShopMap` | 1 | 0/0 |
| `DailyMissionShop` | 1 | 0/0 |
| `LobbySquareFishingTest` | 1 | 1/1 |
| `MagicMarketmap` | 1 | 0/0 |
| `SelectionBox` | 1 | 0/0 |

## Loại trừ bắt buộc

`PCRoomBenefit` (ưu đãi phòng máy Nexon) là `excluded_policy`. Không có đường opt-in, opcode, phần thưởng hay ngoại lệ trên LANBoom/self-hosted. Từ chối khóa cấu hình này kể cả khi enabled: false; không chạy probe để mở mục này.

## Gate Wine trước khi mở opcode

1. Cô lập outbound, chỉ cho phép Wine/client kết nối server và capture cục bộ. Dùng bản sao KR 00967; ghi hash client, bản build/hash server, phiên bản Wine và mọi sai khác của bản sao. Không kết nối Nexon; nếu phải sửa bản sao thì ghi phép đối chứng, giới hạn và cách hoàn nguyên.
2. Với từng opcode định bật, lưu capture từ thao tác thật trong client, chiều, độ dài và trường đã đối chiếu với bộ đọc/ghi KR (VA/RVA khi có); gắn ID run và hash capture. Đối chiếu một thao tác thành công với UI/trạng thái client trước và sau; chỉ có cửa sổ, kết nối TCP, log server hay gói tự sinh chưa đủ.
3. Chạy đối chứng tắt và ngoài lịch, một trường hợp sai điều kiện hoặc chạm quota, và kiểm biên lịch/quota. Với thao tác ghi, phát lại, thử đồng thời và thử lại sau đăng nhập lại hoặc khởi động lại server. Xác nhận không trừ hoặc cấp lần hai, không đổi kết quả đã chốt và không ghi gì khi bị từ chối.
4. Chỉ xét đúng opcode và ngữ cảnh đã đo. Nếu client không phát hoặc xử lý được gói tương ứng thì giữ tắt và ghi unknown. Quy tắc gốc và trạng thái còn hoạt động cũng giữ unknown khi chưa có bằng chứng; giá, thưởng và tỷ lệ cục bộ nếu có phải ghi là design. Lưu raw ở local/ và chỉ commit metadata cùng công thức tái lập.

## Tái lập offline

```bash
python3 tools/kr_f6_extended_events_audit.py --write
python3 tools/kr_f6_extended_events_audit.py --check
```

Công cụ chỉ đọc các đầu vào tài liệu và JSON đã lưu trong repo; `--check` so khớp byte của cả hai đầu ra. SHA-256 băm byte gốc, ngoại trừ PLAN chỉ băm dòng F6 và một LF cuối.
