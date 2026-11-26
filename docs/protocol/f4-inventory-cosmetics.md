# KR 00967: túi đồ, trang bị và mỹ phẩm F4

`p118` chốt phần có thể đọc từ dump IDA F4-0 của KR `00967` (image base `0x400000`; địa chỉ dưới đây là VA
của ảnh unpack, RVA = VA trừ image base). [Tóm tắt bằng chứng](../evidence/content/f4-bc-spec-summary.json)
ghim SHA-256 của binary, dump và nguồn đối chiếu: [phân cụm F4-0](../content/f4-mechanic-clusters.md),
[catalog F4-0](../content/f4-economy-catalog.md) và [kiểm kê pack](../CONTENT.md#hệ-thống-của-client).
Đây là bản đọc lại dump đã pin; không có lượt Wine mới và không dùng opcode hoặc layout VN.
Đối chiếu kế thừa từ F4-0: `boomkr-re` bổ sung ngữ cảnh dữ liệu pack, `boom-re` chỉ cho biết chủ đề cần đo;
không nguồn nào trong hai repo được dùng làm căn cứ cho trường wire dưới đây.

`kr_static/confirmed` xác nhận case, lời gọi, đoạn đọc hoặc tài nguyên trong nguồn đã ghim.
`candidate` là liên hệ suy từ consumer, tên hoặc chuỗi. `unknown` là phần chưa đủ bằng chứng để đóng codec
hoặc gán nghĩa nghiệp vụ. `design` mô tả dữ liệu và chính sách của Rust, kể cả phần đề xuất chưa có mã.

<a id="f4b-inventory-contract"></a>
## F4-b: túi đồ, trang bị, tiêu hao và coupon

RTTI `CItemInventoryNew` có vtable `0x178E624`; dump giữ entry 0–3 là `0x118E3C0`, `0x14EFAC0`,
`0x14EFAB0`, `0x117E280`. Đây là mốc tra consumer; chưa gán vai trò cho từng entry. Bảng sau khóa route theo
**chiều + dispatcher + opcode**. Các case và đoạn đọc thấy được là `kr_static/confirmed`; từng gói đầy đủ vẫn
`unknown`. Theo [primitive KR](f1-aux-opcodes-and-classc.md#f1-aux-codec), slot reader `+0x3C/+0x40/+0x44`
đọc `u8/u16/u32`. `r48` là slot `+0x48` trả về 64 bit trong decompile, chưa chốt biểu diễn, thứ tự byte hoặc
dấu theo nghiệp vụ. `r4c` và `r80` chỉ đặt tên theo slot `+0x4C/+0x80`, chưa biết kiểu và độ dài mã hóa.
Thứ tự bên dưới tính từ lần đọc đầu của handler, sau dispatch; số slot vtable không phải offset payload.

### S→C qua `CStage` (`0xD68C90`)

| Opcode | Case → handler | Đoạn đọc và giới hạn |
|---|---|---|
| `219` | `0xD693B7` → `0xD76860` → `0x11CEA10` | Wrapper gọi `0x52A4E0` rồi giao packet cho callee. Không có reader của callee trong dump. |
| `224` | `0xD68FA9` → `0xD6D370` → `0x11CE4C0` | Cùng accessor `0x52A4E0`; sau khi giao packet còn gọi `0x11CDE80` và `0xD80C20`. Không thấy thứ tự trường của callee. |
| `225` | `0xD68FBB` → `0xD6D410` → `0x11CE6A0` | Cùng accessor, sau đó gọi `0x11CE080`. Không thấy thứ tự trường của callee. |
| `228` | `0xD6917C` → `0xD85F80` | Handler có `CantUseLucciBox` và không có lệnh đọc packet trực tiếp trong đoạn dump; chưa chứng minh toàn case có payload rỗng. |
| `245` | `0xD6AC9C` → `0xD7AC20` | Không có thân reader. Chuỗi `ItemLockErrWrongPassword`, `ResellItemClose/Fail/Success` gợi ý bán lại và khóa vật phẩm (`candidate`). |
| `313` | `0xD6AB2D` → `0xD779C0` | `u8 a, u32 b, u32 c`. Nếu `0x11E0E40(b)` trả khác 0: đọc tiếp `u32 d, u16 e, u8 f`. Consumer dùng `24 * (d == 0 ? 100000 : d)`; chưa biết đơn vị và ý nghĩa của `d`. |
| `314` | `0xD6AB3F` → `0xD77BD0` | `u8 a, u32 b, u32 c, u32 d`; tra `0x11CFF80(a,b)`. Các nhánh đã thấy phân biệt `d == 0` hoặc `d == 100000` với giá trị khác. Dump bị cắt; chưa chốt phần đuôi hay nghĩa của `c/d`. |
| `315` | `0xD6AB51` → `0xD77D60` | `u8 a, u8 b, u32 c, u32 d, u32 e, r48 f` trước lookup `0x11E0E40(c)`. Nếu lookup thành công, nhánh thấy được chuyển `c,d,e` cho `0x119A960` cùng các đối số nội bộ, rồi gọi `0x11AECE0(a, 27, ..., &f, 0, b)`. |
| `316` | `0xD6AB63` → `0xD77EF0` | `u8 a, u32 b, u32 c`. Đọc tiếp `r48 d` nếu `0x1220830`, lookup `0x11E0E40(b)` và phép kiểm `0x11E8F70` trên giá trị lấy từ record đều trả khác 0. Nhánh còn lại tra `0x11CFF80(a,b)` rồi có đường tới `0x11CFD50`; chưa đủ để gọi `c` là số trừ. |
| `317` | `0xD6AF10` → `0xD84410` | `u8 a, u8 b, u32 c, u32 d`. Nếu `a != 0`: `r80` rồi `u32`; dump dừng trong nhánh. Chuỗi `PeriodChange_fail` và `PeriodChange_Success` gợi ý đổi thời hạn (`candidate`). |
| `377` | `0xD6ABF3` → `0xD78E10` | Không có thân reader; chuỗi `ItemLockDisableSuccess`, `ItemLockDeleteSuccess`, `ItemLockErrWrongPassword`, `ItemLockDisableAbnormalFail`. |
| `378` | `0xD6AC05` → `0xD791D0` | Không có thân reader; chuỗi `ItemLockChangeSuccess`, `ItemLockChangeWrongPassword`, `ItemLockChangeAbnormalFail`. |
| `418` | `0xD6AF22` → `0xD847C0` | Không có thân reader; chuỗi `PowerUpKitFail`, `PowerUpKitInvenFull`, `PowerUpKitUnlimitedFull`, `PurchaseItemBagByLevelUpTreasure`. |
| `419` | `0xD6AF34` → `0xD852C0` | Không có thân reader; chuỗi `InstanceRewardFail`, `InstanceRewardInvenFail`, `InstanceRewardUnlimitedFull`, `InstanceRewardFull`. |

`315` nối tĩnh tới `0x11AECE0`, nơi có site `0x11AFA5D` gọi builder `0x1164C10(566)` (C→S).
Chưa có điều kiện đi tới site, writer, lời gọi gửi hay frame `566` trong dump F4-0. Không coi nó là ACK hoặc
gửi `315` để buộc client trả `566`. Vai trò snapshot, delta, equip hoặc use của các case còn là `candidate`;
chưa nối được trường nào với `sku_id`, slot, số lượng hay `expires_at` của DB.

### S→C qua `CShopStageNew` (`0x1072B10`)

| Opcode | Case → handler | Đoạn đọc và giới hạn |
|---|---|---|
| `236` | `0x1072DDF` → `0x1074130` | Chỉ có khai báo, cảnh báo `bad sp`; chưa có prefix. Case giữ chuỗi `UseItemCoupon`, `ValidFor`, `Each`. |
| `237` | `0x1072DFD` → `0x1075900` | Chỉ có khai báo; chưa có prefix. Case giữ chuỗi `UseItemCoupon`, `Each`, ` %d`. |
| `345` | `0x1072EFC` → `0x107E660` | `r4c`; kết quả qua `0x4509A0`, nhánh trả khác 0 thoát sớm, nhánh kia có `CrazyPark_ReceiveCouponGift`. Chưa biết encoding hay trường còn lại. |
| `446` | `0x1072E2D` → `0x107C0F0` | `r4c, u32 n`; mỗi vòng `i < n` bắt đầu bằng ít nhất một `u32`, dùng qua `0x117F1E0`. Chưa thấy hết record; chuỗi `UMItemExpired1/2` gợi ý hết hạn. |
| `447` | `0x1072E3F` → `0x107C3D0` | `r4c, u32 n`; mỗi vòng `i < n` bắt đầu bằng hai `u32`, dword đầu vào `0x11CFD50` và `0x11E0E40`. Chưa thấy hết record; chuỗi `UMItemSold1` gợi ý đã bán. |

`n` là cận lặp quan sát được; giới hạn hợp lệ và cách xử lý giá trị bất thường còn `unknown`. Các case này
không xác nhận request nhập coupon, kiểu mã, mã kết quả, điều kiện cấp quà, quyền đổi hoặc khóa chống lặp.

### Nối Schema 7 và ranh giới triển khai

[Schema 7](../../rust/crates/kr-service/migrations/0007_f4_progression_inventory_shop.sql) và
[`AccountService`](../../rust/crates/kr-service/src/account_worker/economy.rs) là contract nội bộ (`design`).

| Ý định sau khi nối được route KR | API và bất biến hiện có |
|---|---|
| Đọc sở hữu thực | `inventory_list(player_id)` trả `InventoryItem` theo SKU, có `grant_kind`, `quantity`, `expires_at`, dấu thời gian, `active` và `equipped_slots`. `player_inventory` khóa `(player_id, sku_id)`, `quantity >= 1`; chỉ grant `timed` có `expires_at`, `permanent` có số lượng 1. Grant chồng theo `max_stack`; grant có hạn gia hạn từ mốc muộn hơn giữa `now` và hạn cũ. |
| Đọc slot đã lưu | `equipped_list(player_id)` trả `slot, sku_id, equipped_at`. `player_equipped` khóa `(player_id, slot)` và tham chiếu hàng sở hữu cùng người. Kết quả đọc thô có thể còn slot của món hết hạn; projection phải kiểm `active` (`expires_at > now` hoặc không có hạn) trước khi cho dùng hay render. |
| Đổi hoặc xóa slot | `equip_item(player_id, sku_id, slot)` kiểm người chơi còn hoạt động, SKU có trong catalog, `equip_slot` trùng slot, có sở hữu và còn hạn. Nó thay slot nguyên tử; gửi lại cùng SKU giữ `equipped_at`. SKU đã tắt bán vẫn có thể trang bị nếu thỏa các điều kiện trên. `unequip_slot(player_id, slot)` trả `false` khi slot đã trống. |
| Đọc quyền sử dụng | `usable_items(player_id, access)` ở standard lấy hàng sở hữu còn hiệu lực, có số lượng thật; O1 `unlock_all` lấy mọi SKU đang bật trong catalog, trả `unlocked_by_session = true`, `quantity = None`, `expires_at = None`, không đọc/ghi túi hay trừ Lucci. |

Worker Rust có thể chuẩn bị projection nội bộ, ánh xạ token KR đã kiểm sang SKU/slot và các fixture đọc prefix.
Bind người chơi từ phiên đã xác thực, kể cả khi frame có trường giống chỉ số người chơi. Chỉ bật builder và
handler cho route đã chốt chiều, stage, nghĩa trường, nhánh và độ dài bằng consumer/capture. Không gửi SKU
chuỗi, tên slot, timestamp DB, `100000` hay số lượng O1 lên wire như giá trị mặc định.

**Tiêu hao trong trận (`design`).** API nội bộ `consume_item_in_match` nhận hành động dùng vật phẩm trong túi
đã được luật trận chấp nhận. Bên gọi ràng buộc phiên, người chơi và lượt hiện hành, phân biệt vật phẩm này với
pickup hay đồ do trận cấp, và giữ nguyên khóa cùng SHA-256 của hành động khi thử lại. Với standard, giao dịch
chỉ trừ stack của người chơi có trong roster đã lưu của trận đang chạy. Khi hết stack, nó xóa các slot tham
chiếu trước khi xóa hàng sở hữu; lỗi trừ hoặc lỗi ghi biên nhận đều hoàn tác giao dịch.

[Schema 8](../../rust/crates/kr-service/migrations/0008_f4_match_consumptions.sql) lưu biên nhận riêng theo
`(player_id, idempotency_key)`. Replay khớp trả lại biên nhận gốc, kể cả số lượng còn lại tại lần dùng đó
và sau khi trận đã kết thúc; đổi digest, trận, SKU hoặc số lượng với cùng khóa thì bị từ chối. Bên gọi chỉ
phát kết quả sau khi nhận thành công và có thể dùng biên nhận để phát lại. Tiêu hao không ghi ledger và không
được lặp khi chốt thưởng. Với O1 `unlock_all`, API chỉ kiểm SKU stackable đang bật trong catalog; quyền phiên
không đọc hay trừ túi và không ghi biên nhận. Luật trận vẫn quản lý giới hạn dùng và replay của O1. Chưa nối
API này vào đường xử lý gói KR; các yêu cầu về quyền phiên và luật trận thuộc phần nối đó.

### Phép đo tối thiểu cho F4-b

1. Mở túi với một tài khoản trống, rồi với một SKU đã biết; thay từng biến một: SKU, số lượng `1/2`, hạn hoặc
   slot. Đặt breakpoint ở `0x11CEA10/0x11CE4C0/0x11CE6A0` và các handler `313–317`. Đo cursor trước/sau mỗi lần
   đọc, lookup đầu vào/đầu ra, phép ghi vào manager và số byte còn lại; tách nhánh lookup trúng/trượt và guard
   của `316/317`. Xác định ranh giới record, token vật phẩm/chỉ số, sentinel, đơn vị hạn và kiểu các reader chưa rõ.
2. Ghi từng thao tác mở lại túi không đổi trạng thái, equip, thay món cùng slot, unequip và relogin. Với các case
   thông báo, ghép thao tác với request builder và kết quả UI trước khi gán mã lỗi. Bám `0x11AECE0` và site `0x11AFA5D`; chỉ
   ghi nhận `566` thực gửi nếu thấy writer và frame ở đường gửi, cùng điều kiện xuất hiện hoặc đối chứng không gửi.
3. Với một stack tiêu hao xác định, đo một lần dùng hợp lệ và một lần bị từ chối; lặp cùng hành động đã nhận,
   thử hết hàng, hết hạn hoặc không sở hữu. Kiểm hiệu ứng trong trận, cursor, số lượng DB trước/sau và UI sau
   relogin. Đối chứng bằng pickup để phân biệt vật phẩm túi với vật phẩm cấp trong trận.
4. Đo coupon trên stub cục bộ bằng mã thử: hợp lệ, từ chối và gửi lặp. Bám UI tới builder để xác định opcode,
   encoding/độ dài mã, guard và giới hạn; đo `236/237/345` và thông báo `446/447` riêng theo thao tác thực tế.
   So UI, trạng thái đổi và grant trong DB; một thông báo lên màn hình chưa chứng minh đã đổi coupon.

<a id="f4c-cosmetics-contract"></a>
## F4-c: trang trí, emoticon và bộ sưu tập

RTTI, entry đầu và sự hiện diện của các nhóm pack là `kr_static/confirmed`; cột mốc manager dùng để lần xref,
liên hệ asset → manager → render vẫn là `candidate`. Tên slot gợi ý là khóa **nội bộ** (`design`), không phải
enum KR. Số mục là entry pack, không phải số SKU có thể dùng.

| Nhóm pack KR | Mốc manager trong dump | Ánh xạ Schema 7 đề xuất |
|---|---|---|
| `CharTE/AvatarDeco` (7797) | `CCharDecoMan` vtable `0x174590C`, entry 0 `0xB6B0D0` là mốc kiểm nhóm avatar/nhân vật/cấp; cạnh avatar chưa có. | `decoration`, slot `cosmetic.avatar` khi đã nối token. |
| `CharTE/CharDeco` (3896) | Cùng mốc `CCharDecoMan`. | `decoration`, slot `cosmetic.char`; tách slot nhân vật nền `character` khỏi trang trí. |
| `CharTE/GradeDeco` (211) | Cùng mốc `CCharDecoMan`; chưa biết đường áp riêng. | `decoration`, chỉ dùng slot `cosmetic.grade` nếu xác nhận người chơi chọn riêng; không chiếu cấp hồ sơ thành đồ sở hữu. |
| `CharTE/BombDeco` (594) | `CBombDecoMan` vtable `0x174425C`, entry 0 `0xB3BB50`. Dump có 40 entry thô, biên bảng và vai trò entry chưa chốt. | `decoration`, slot `cosmetic.bomb`. Cần kiểm asset trên bom của A ở màn hình B. |
| `EmoTE/IDDeco` (3176) | `CIDDecoMan` vtable `0x17958D4`, entry 0 `0x12772E0`. | `decoration`, slot `cosmetic.id`; kiểm token và bề mặt hiện ID. |
| `EmoTE/Character` (1479), `fx/Emoticon` (52) | Không có cạnh tới writer, reader hay render theo người chơi trong dump F4-0. | `emotion` cho SKU `EmoTE/Character`; dùng quyền SKU khi phát emoticon. Chưa gán SKU trực tiếp cho `fx/Emoticon` hay slot equip cho sự kiện; nếu thấy thanh gán, đo layout riêng. |
| `FxContents/ItemCollection` (27) | `CCAItemCollectionMgr` vtable `0x172C618`, entry 0 `0x927E20`. | Exporter dùng mẫu `Pet/Pet_<số>` từ `fx/ObjectTable/ObjectTable` làm SKU `collection`. Quyền sở hữu SKU này không suy ra tiến độ, mục tiêu, phần thưởng hay slot collection KR. |

Với standard, gán `equip_slot` rõ trong catalog thử và dùng `equip_item` cho một SKU còn hạn mỗi slot đã nối.
Với O1, `O1SessionCosmetics` giữ lựa chọn của từng phiên trong bộ nhớ (`design`). Nó chỉ nhận SKU đang bật
và có `equip_slot` khớp, kể cả slot `character` cho nhân vật nền. Phiên mới bắt đầu trống; tháo, xóa hoặc
đổi lựa chọn không đọc hay ghi SQLite. Chưa nối bộ chọn này vào projection hoặc handler KR.
Không gửi cả catalog như snapshot túi. Khi có route đã đo, dựng projection công khai tối thiểu từ selection
hợp lệ; dùng cùng selection cho snapshot và cập nhật peer, kiểm hạn khi chiếu, không phát private inventory.
Opcode, payload hai chiều, token/enum, guard, phạm vi peer và persistence native của F4-c đều còn `unknown`.

### Nghiệm thu Wine hai client cho F4-c

1. Kiểm lease Wine chung, dùng hai prefix sạch và hai account local A/B trên cùng dịch vụ, cùng map và nhân vật
   đối chứng. Cô lập outbound theo allowlist host/stub/capture cục bộ, không kết nối Nexon. Ghi phiên bản Wine,
   hash client/copy/patch và binary Rust build locked, cấu hình, profile standard hoặc O1, và trạng thái DB trước
   lượt. Giữ một lượt baseline với đồ mặc định và chụp cùng góc nhìn ở hai client.
2. Mỗi lần chỉ đổi một asset đã biết trong bảng, ưu tiên avatar, nhân vật, bom, ID rồi grade nếu có chọn riêng.
   Ghi thao tác của A, giao diện túi/MyInfo và sảnh của A; B kiểm bề mặt hiện người chơi đó trong sảnh và roster
   phòng, rồi trong trận. Với bom, A đặt một quả cùng điều kiện baseline; B nhìn quả bom của A trước nổ. Lưu
   ảnh A/B có mốc thời gian, kiểm chủ thể và token render; ghi rõ bề mặt không hỗ trợ hoặc chưa quan sát thay
   vì coi thiếu hiệu ứng là pass. Lặp khi B vào phòng sau thay đổi để kiểm snapshot, tháo/đổi lại để kiểm delta;
   đảo vai ít nhất một lần để kiểm đồng bộ theo người gửi.
3. Phát một emoticon từ A ở từng stage có UI phát. So render A và B, chủ thể, token và kênh chuyển tiếp;
   thử một SKU không được phép. Với collection, mở panel với fixture chưa có và có một thay đổi đã xác định,
   đo cursor và UI tiến độ/phần thưởng nếu có. Chỉ kỳ vọng B đổi UI nếu thấy route công khai tới B; kiểm B không
   nhận dữ liệu sở hữu riêng. Không dùng grant Pet thay cho phép đo collection native.
4. Với standard, thử món hết hạn hoặc không sở hữu, rồi relogin A và cho B vào lại để kiểm lựa chọn hợp lệ
   được phục hồi và món bị từ chối không xuất hiện. Lặp với O1: SKU bật mà túi trống dùng được trong phiên,
   SKU tắt bị từ chối, và không sinh sở hữu/trừ Lucci. Với lựa chọn O1 chỉ giữ trong phiên, kiểm phiên mới không
   kế thừa lựa chọn cũ nếu chưa có contract lưu riêng.
5. Cho cả hai client, giữ capture TCP/UDP và đối chiếu frame đã giải mã với stage, chiều, opcode, VA writer/
   reader, cursor, guard và kết quả render. Đặt breakpoint theo RVA tại base thực tế, ghi giá trị reader và
   consumer, ghép theo thời gian với ảnh A/B và DB trước/sau. Chỉ chốt từng route khi frame, cursor và UI cùng
   khớp đối chứng; local preview không chứng minh peer đã nhận. Giữ raw ở `local/` và dọn bản sao/prefix tạm
   theo [quy định dọn lượt chạy](../evidence/owner-directives/run-cleanup.json).

<a id="f4b-inventory-callees-w2"></a>
## Bổ sung tĩnh Wave 2 cho callee F4-b (`219/224/225` và `566`)

Bằng chứng JSON chi tiết nằm ở [f4-w2-ida-deepdive.json](../evidence/content/f4-w2-ida-deepdive.json). Các điểm `kr_static/confirmed` mới tách được trong decompile:

- **`sub_11CEA10` (`0x11CEA10`, callee ứng với nhánh `219`)**: đọc outer `count` (`vtable+60`, `u8`), lặp mỗi cụm đọc thêm `u8` (`vtable+60`) và `item_count` (`vtable+68`), rồi lặp `sub_11CE930(v4, a2)`, tra `sub_117F1E0` và chèn vào `this + 61320*i + 36` qua `sub_11CB910`; khác với `sub_11CE4C0`, hàm này **không** gọi bộ reset snapshot.
- **`sub_11CE4C0` (`0x11CE4C0`, callee ứng với nhánh `224`)**: đọc outer `count` (`vtable+60`), với mỗi phần tử đọc `v23` (`vtable+60`, `u8`) và `v13` (`vtable+68`, `u32`), **gọi reset snapshot `sub_11DE900(&this[20*v23 + 123852])` và `sub_1180360(&this[61320*v23 + 36])`** trước khi đọc `v13` bản ghi qua `sub_11CE930`, tiếp theo là `sub_11DEB00`/`sub_11DE6C0`/`sub_11DA990`, và kết thúc bằng `sub_11CDE80` + `sub_11D0070`.
- **`sub_11CE6A0` (`0x11CE6A0`, callee ứng với nhánh `225`)**: đọc `v7` (`vtable+60`, `u8`), xóa sạch **`8` slot mỗi phần tử** (`sub_55FD30(0)` x2, `sub_455630`), rồi lặp `v7 * 8` lần đọc cặp `v8` (`vtable+68`, `u32`) + chuỗi (`vtable+80`); nếu `v8 != 0` và tra được qua `sub_11E0E40(v8)` thì ghi bằng `sub_55FD30`/`sub_449C90` và đồng bộ qua `sub_11CE080` (`sub_11CF5E0`/`sub_11CF9C0`).
- **Site `C->S 566` (`0x11AFA5D` trong `sub_11AECE0`, nhánh `sub_1199D00 == 35`)**: đọc bitmask `+56` của mục tra từ `sub_11E0E40` (`& 1` gọi `sub_13E4790(a3, 1)`, `& 0x200` gọi `sub_13E2380(a3, 0)`, `& 0x400` gọi `sub_13E2440(a3, 0)`), sau đó dựng `C->S 566` (`0x236`) với đúng **một byte `u8`** qua `sub_14EBC50` và gửi bằng `sub_1164D50`.
- **Phần còn `unknown`**: layout bên trong của `sub_11CE930`, ý nghĩa chuỗi `vtable+80` trong `S->C 225`, và thao tác UI kích hoạt thực gửi `C->S 566` trên Wine.

<a id="f4b-item-record-11ce930-w3"></a>
## Bổ sung tĩnh Wave 3 cho `sub_11CE930` (`219/224`) và `S->C 313..317`

Chi tiết kiểm chứng ở [f4-w3-ida-records.json](../evidence/content/f4-w3-ida-records.json) (`kr_static/confirmed`):

- **Thứ tự đọc một bản ghi item trong `sub_11CE930` (`0x11CE930`, dùng chung cho `219` và `224`)**: đọc lần lượt `vtable+68` (`u32`), `vtable+64` (`u16`), `vtable+68` (`u32`), `vtable+80` (chuỗi vào `a2+108`), `vtable+88` (8 byte vào `v8`) + `vtable+68` (`u32`) qua `sub_11CB9B0`, và cuối cùng `vtable+60` (`u8`). Nghĩa nghiệp vụ từng slot (hạn/timestamp vs số lượng/serial) còn **unknown**.
- **`S->C 313` (`sub_D779C0`)**: đọc `u8`, `u32`, `u32`, `u32` (fallback `100000`, nhân `24`), `u16`, `u8`; tra catalog `sub_11E0E40`, dựng struct `196B` `sub_11CB1D0` và thêm qua `sub_117E750(v5, 0, 0)`.
- **`S->C 314` (`sub_D77BD0`)**: đọc `u8`, `u32`, `u32`, `u32`; tra `sub_11CFF80`, cập nhật `sub_6BC7B0` và `sub_11CB7F0(24*v9)` / `sub_11CB880`.
- **`S->C 315` (`sub_D77D60`)**: đọc `u8`, `u8`, `u32`, `u32`, `u32`, `u64/i64` (`vtable+72`); tra catalog và chuyển tiếp `sub_11AECE0`.
- **`S->C 316` (`sub_D77EF0`)**: đọc `u8`, `u32`, `u32` (+ `u64` khi nhánh `sub_11E8F70` bật); trừ trường `+96`, khi `<= 0` xóa/gỡ qua `sub_11CFD50` + `sub_1094080`.
- **`S->C 317` (`sub_D84410`)**: đọc `u8 ok`, `u8 slot`, `u32 old`, `u32 item`; khi `ok != 0` đọc thêm `16` byte (`vtable+128`) + `u32`, cập nhật hạn dùng (`sub_11CB960`) và hiện `PeriodChange_Success` (hoặc `PeriodChange_fail`).

<a id="f4c-cosmetic-xrefs-w3"></a>
### Kiểm tra xrefs vtable F4-c Wave 3

`entry 0` của `CCharDecoMan` (`0xB6B0D0`), `CBombDecoMan` (`0xB3BB50`) và `CIDDecoMan` (`0x12772E0`) trong dump tĩnh chỉ là destructor wrapper gọi `sub_B6AD00`/`sub_B36860`/`sub_1276C00`; không dựng route runtime từ ba vftable này khi chưa có đo Wine hoặc tìm được caller thực. Riêng `CCAItemCollectionMgr` có constructor `sub_914130` được `sub_1231BD0` gọi tại `0x1234300`.

<a id="f4b-mypage-wine-probe-w3"></a>
### Kết quả probe Wine `MyPage` (Wave 3)

Trong lần chạy Wine `f4-ui-probe-v1` ([f4-ui-probe-wine.json](../evidence/content/f4-ui-probe-wine.json), `f4a-rust-v1`, bật thưởng `Tomb12`), sau khi `CookieA` nhận `S->C 135` (`lucci=100`) và về sảnh, thao tác click đơn vào nút `마이페이지` tại `(770, 678)` làm bật tooltip `내 아이템 보기` nhưng `CA.exe` giữ nguyên màn hình sảnh và chỉ gửi các gói nền định kỳ (`715`, `15`); chưa phát request mở MyPage hay đồng bộ túi đồ.

<a id="f4b-mypage-entry-guards"></a>
### Các cổng trước khi vào MyPage

[CP401](../evidence/checkpoints/records/CP401.json) xác nhận các cạnh tĩnh trên KR 00967
(`kr_static/confirmed`). Bảng hàm ảo của `CLobbyStage` tại `0x1766A2C` chứa `0xDA1E80` ở
`+0x2B4`; handler gọi `0x812090(this + 0xAFDC)` và chỉ chuyển tới `0xD73C50` nếu kết quả bằng 0.
Hai xref `Dialog/BtMyPage` của Wave 5 nằm trong các dialog thưởng người mới và người quay lại;
chúng chưa chứng minh nút sảnh tại tọa độ của probe gọi handler này.

`0xD73C50` chứa đường xác nhận `ConfirmGoToMyPageNormal`. Trước khi đi đến các nhánh gửi, hàm
kiểm chỉ số có dấu trong `-1..1`, tách `sub_1220080() == 3` sang thông báo `DeferShutdown`, và
chỉ đi tiếp nếu `sub_13E4720(sub_559BB0()) != 0`. Getter `0x13E4720` đọc qua `0x529C60` ở
`this + 0xF8`; chưa chốt nguồn gán hoặc ý nghĩa giá trị. Case S→C `235` tại `0xD69095` cũng có
đường gọi `0xD73C50` qua `0xD6D810`, có điều kiện stage 11/12 và cờ; đây là locator tĩnh để
đối chiếu, chưa đủ layout để tạo gói `235`.

Phép đo tiếp theo: với cùng thao tác và một đối chứng chỉ rê chuột, quan sát vào `0xDA1E80`,
kết quả tại `0xDA1E95`, rồi chỉ số và các giá trị tại `0xD73C8E` và `0xD73D30`. Nếu hàm đi
tới builder, đối chiếu opcode, cursor và frame đã gửi; nếu callback không chạy, lần ngược sự kiện
của nút sảnh trước. Không suy cổng nào đã chặn probe cũ chỉ từ tooltip hoặc việc không thấy gói.

[CP402](../evidence/checkpoints/records/CP402.json) giữ hai lượt chuẩn bị Wine đã về sảnh sau `Tomb12`.
GDB đã kiểm được byte lệnh của bản copy ở lượt sửa cổng kết nối, nhưng dừng do `SIGSEGV` khi tiếp tục,
trước cả phép đối chứng rê chuột. Chưa có quan sát cổng MyPage mới. Trước khi lặp ván, kiểm attach và
resume của GDB với kiến trúc do proxy Wine báo, trên fixture WOW64 cô lập và không đặt breakpoint;
chỉ thử lại MyPage khi đối chứng debugger không dừng tiến trình.

[CP403](../evidence/checkpoints/records/CP403.json) đối chiếu bằng fixture PE32 chạy trên WOW64:
chạy thường và khởi chạy dưới WineDbg đều hoàn tất; attach muộn với kiến trúc tự chọn hoặc ép `i386`
đều dừng với `SIGSEGV` khi không đặt breakpoint. Với bản copy KR, khởi chạy dưới WineDbg và gắn
observer MyPage đã qua kiểm byte lệnh, nhưng tiến trình thoát mã `03` trước khi sẵn sàng đăng nhập.
Chưa biết nguyên nhân thoát. Nếu thử tiếp, giữ cùng bản copy và tham số, khởi chạy dưới WineDbg
không gắn observer rồi so với cách chạy thường. Chỉ đo nút khi đối chứng đã tới sảnh.

<a id="f4b-reader-slots-w4"></a>
### Giải mã bảng vtable reader `CReceivedPacketBase` (Wave 4)

Từ [f4-w4-ida-gates.json](../evidence/content/f4-w4-ida-gates.json) (`kr_static/confirmed`), `CReceivedPacketBase` (`0x17B0BF4`, kế thừa `IReceivedPacket` `0x17B0AD0`) khóa nguyên thủy của mọi slot reader dùng trong `sub_11CE930` và `S->C 313..317`:

- **`+60` (`0x14EAFB0`)**: `u8/i8` (1 byte, qua `+96`, tăng cursor `+1`).
- **`+64` (`0x14EB040`)**: `u16` (2 byte, qua `+100`, tăng cursor `+2`).
- **`+68` (`0x14EB0D0`)**: `u32/i32` (4 byte, qua `+104`, tăng cursor `+4`).
- **`+72` (`0x14EB160`)**: `u64/i64` (8 byte, qua `+108`, tăng cursor `+8`, khớp `r48` trước đây).
- **`+76` (`0x14EB9E0`) / `+80` (`0x14EBA70`)**: đọc chuỗi qua `+112` vào `std::string` (`+76`) hoặc buffer đầu ra (`+80`), tăng cursor theo số byte đọc.
- **`+88` (`0x14EB200`)**: đọc `len` byte thô qua `+120(cursor, buf, len)`.
- **`+128` (`0x14EBBD0`)**: đọc liên tiếp `8` trường `u16` qua `+64` (`16` byte, khớp `r80` / nhánh có hạn dùng trong `S->C 317`).

Điều này xác nhận phần đọc `sub_11CE930` gồm `(u32, u16, u32, string, 8B + u32, u8)`; nhãn nghiệp vụ từng cột vẫn `unknown` tới khi đo Wine.

[CP400](../evidence/checkpoints/records/CP400.json) đối chiếu lại hai slot trên ảnh unpack KR 00967
(`kr_static/confirmed`, phạm vi mode 2). Slot `+80` gọi `0x14EBA90` qua `+112`: đọc độ dài byte
`u16` rồi giải mã đúng số byte đó bằng `0x14A5AD0`. Slot `+88` gọi `0x14EB220` qua `+120`:
độ dài 8 byte do `sub_11CE930` truyền vào, nên không có trường độ dài trên wire. Hàm dùng cùng
`0x14A5AD0`; với 8 byte, nó giải mã hai nhóm `u32` riêng rồi ghi hai giá trị theo thứ tự little-endian
vào buffer. `kr-wire` giữ tám byte đã giải mã mà chưa gán nghĩa; bốn byte đầu và bốn byte sau không
được đọc như một `u64`. Phép kiểm dùng reader biệt lập, chưa có frame túi đồ native hoặc nghiệm thu Wine.
