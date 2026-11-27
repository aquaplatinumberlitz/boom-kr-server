# Ma trận projection mỹ phẩm F4 — KR 00967

Sinh bằng `python3 tools/kr_f4_cosmetics_spec.py --write`; kiểm lại bằng `--check`.
Bản máy đọc: [`docs/evidence/content/f4-cosmetics-projection-spec.json`](../evidence/content/f4-cosmetics-projection-spec.json). SHA-256 nội dung chuẩn: `8a91f0ac7302d957094566da06ce41a463d5104aef72c3994694f1a5acfea839`.

Ma trận dùng nguồn đã commit: exporter, preset, metadata KR, codec Rust và migration SQLite.
Slot của catalog là token nội bộ (1–32 ký tự); preset chỉ gán `character`. Các slot đề xuất và
fixture chỉ ghi lại phạm vi nguồn. Cột bề mặt ghi khoảng trống khi chưa biết token, trường,
guard hoặc audience. Các SKU sở hữu riêng không được suy thành dữ liệu công khai cho peer.

Các hàng `balloon`, `flag`, `hat`, `effect`, `outfit`, `vehicle` và `ring` giữ chỗ để theo dõi
bề mặt cần đo. Chưa có ánh xạ xác nhận từ tên này sang nhóm SKU F4. Cột DB của `hat` chỉ nói
về fixture `head`; cột DB của slot đề xuất chỉ áp dụng nếu catalog gán slot và SKU còn hiệu lực.

## Ma trận theo slot và bề mặt

| Target | Nhóm SKU / nguồn pack | Slot nội bộ | `kr_wire` | `own291` | `start121` | `room_slot` | DB ở Schema 8 |
|---|---|---|---|---|---|---|---|
| `avatar` | `decoration` / `CharTE/AvatarDeco` | `cosmetic.avatar` (đề xuất) | [unknown](#wire-gap) | [unknown](#wire-gap) | [unknown](#wire-gap) | [unknown](#wire-gap) | [`db_inventory`](#anchor-db_inventory), [`db_equipped`](#anchor-db_equipped) |
| `backup` | `consumable` | `backup` (fixture Rust) | [unknown](#wire-gap) | [unknown](#wire-gap) | [unknown](#wire-gap) | [unknown](#wire-gap) | [`db_inventory`](#anchor-db_inventory), [`db_equipped`](#anchor-db_equipped), [`db_consumption`](#anchor-db_consumption) |
| `balloon` | — | — | [unknown](#wire-gap) | [unknown](#wire-gap) | [unknown](#wire-gap) | [unknown](#wire-gap) | `unknown` |
| `bomb` | `decoration` / `CharTE/BombDeco` | `cosmetic.bomb` (đề xuất) | [unknown](#wire-gap) | [unknown](#wire-gap) | [unknown](#wire-gap) | [unknown](#wire-gap) | [`db_inventory`](#anchor-db_inventory), [`db_equipped`](#anchor-db_equipped) |
| `char_decoration` | `decoration` / `CharTE/CharDeco` | `cosmetic.char` (đề xuất) | [unknown](#wire-gap) | [unknown](#wire-gap) | [unknown](#wire-gap) | [unknown](#wire-gap) | [`db_inventory`](#anchor-db_inventory), [`db_equipped`](#anchor-db_equipped) |
| `character` | `character` / `FxContents/Attr/BomberAttr` | `character` (preset) | [`character92`](#anchor-character92) | [unknown](#wire-gap) | [`character_start121`](#anchor-character_start121) | [`character_room_slot`](#anchor-character_room_slot) | [`db_inventory`](#anchor-db_inventory), [`db_equipped`](#anchor-db_equipped), [`db_character_preference`](#anchor-db_character_preference) |
| `collection` | `collection` / `fx/Pet` | — | [unknown](#wire-gap) | [unknown](#wire-gap) | [unknown](#wire-gap) | [unknown](#wire-gap) | [`db_inventory`](#anchor-db_inventory) |
| `collection_panel` | `FxContents/ItemCollection` | — | [unknown](#wire-gap) | [unknown](#wire-gap) | [unknown](#wire-gap) | [unknown](#wire-gap) | `unknown` |
| `consumable` | `consumable` / `FxContents/Attr/ItemAttr` | — | [unknown](#wire-gap) | [unknown](#wire-gap) | [unknown](#wire-gap) | [unknown](#wire-gap) | [`db_inventory`](#anchor-db_inventory), [`db_consumption`](#anchor-db_consumption) |
| `effect` | — | — | [unknown](#wire-gap) | [unknown](#wire-gap) | [unknown](#wire-gap) | [unknown](#wire-gap) | `unknown` |
| `emote` | `emotion` | `emote` (fixture Rust) | [unknown](#wire-gap) | [unknown](#wire-gap) | [unknown](#wire-gap) | [unknown](#wire-gap) | [`db_inventory`](#anchor-db_inventory), [`db_equipped`](#anchor-db_equipped) |
| `emote_slot` | `decoration` / `EmoTE/Slot` | — | [unknown](#wire-gap) | [unknown](#wire-gap) | [unknown](#wire-gap) | [unknown](#wire-gap) | [`db_inventory`](#anchor-db_inventory) |
| `emoticon` | `emotion` / `EmoTE/Character` | — | [unknown](#wire-gap) | [unknown](#wire-gap) | [unknown](#wire-gap) | [unknown](#wire-gap) | [`db_inventory`](#anchor-db_inventory) |
| `emoticon_effect` | `fx/Emoticon` | — | [unknown](#wire-gap) | [unknown](#wire-gap) | [unknown](#wire-gap) | [unknown](#wire-gap) | `unknown` |
| `fanfare` | `emotion` / `EmoTE/Fanfare` | — | [unknown](#wire-gap) | [unknown](#wire-gap) | [unknown](#wire-gap) | [unknown](#wire-gap) | [`db_inventory`](#anchor-db_inventory) |
| `fishing` | `decoration` / `CharTE/FishingDeco` | — | [unknown](#wire-gap) | [unknown](#wire-gap) | [unknown](#wire-gap) | [unknown](#wire-gap) | [`db_inventory`](#anchor-db_inventory) |
| `flag` | — | — | [unknown](#wire-gap) | [unknown](#wire-gap) | [unknown](#wire-gap) | [unknown](#wire-gap) | `unknown` |
| `grade` | `decoration` / `CharTE/GradeDeco` | `cosmetic.grade` (có điều kiện) | [unknown](#wire-gap) | [unknown](#wire-gap) | [unknown](#wire-gap) | [unknown](#wire-gap) | [`db_inventory`](#anchor-db_inventory), [`db_equipped`](#anchor-db_equipped) |
| `hat` | — | `head` (fixture Rust) | [unknown](#wire-gap) | [unknown](#wire-gap) | [unknown](#wire-gap) | [unknown](#wire-gap) | [`db_inventory`](#anchor-db_inventory), [`db_equipped`](#anchor-db_equipped) |
| `id` | `decoration` / `EmoTE/IDDeco` | `cosmetic.id` (đề xuất) | [unknown](#wire-gap) | [unknown](#wire-gap) | [unknown](#wire-gap) | [unknown](#wire-gap) | [`db_inventory`](#anchor-db_inventory), [`db_equipped`](#anchor-db_equipped) |
| `ladder` | `decoration` / `CharTE/LadderDeco` | — | [unknown](#wire-gap) | [unknown](#wire-gap) | [unknown](#wire-gap) | [unknown](#wire-gap) | [`db_inventory`](#anchor-db_inventory) |
| `outfit` | — | — | [unknown](#wire-gap) | [unknown](#wire-gap) | [unknown](#wire-gap) | [unknown](#wire-gap) | `unknown` |
| `quick` | `consumable` | `quick` (fixture Rust) | [unknown](#wire-gap) | [unknown](#wire-gap) | [unknown](#wire-gap) | [unknown](#wire-gap) | [`db_inventory`](#anchor-db_inventory), [`db_equipped`](#anchor-db_equipped), [`db_consumption`](#anchor-db_consumption) |
| `ring` | — | — | [unknown](#wire-gap) | [unknown](#wire-gap) | [unknown](#wire-gap) | [unknown](#wire-gap) | `unknown` |
| `vehicle` | `FxContents/Attr/ItemAttr: Ship/*` | — | [unknown](#wire-gap) | [unknown](#wire-gap) | [unknown](#wire-gap) | [unknown](#wire-gap) | `unknown` |

### Giới hạn theo mục

- `backup`: Kiểm thử thay catalog để hai slot cùng tham chiếu stack; không phải hotbar mặc định.
- `character`: Khóa BomberAttr của nhân vật nền đã chọn. Chưa nối SKU trang bị với lựa chọn phòng.
- `collection`: Chỉ mẫu Pet/Pet_<số> được xuất. Sở hữu Pet chưa xác nhận tiến độ, mục tiêu hoặc thưởng.
- `collection_panel`: Tài nguyên panel và constructor manager chưa xác nhận đồng bộ tiến độ.
- `consumable`: Chỉ SKU ItemAttr được preset chọn; pickup trong trận nằm ngoài quyền túi đồ.
- `emote`: Slot fixture của Web Shop; không xác nhận trang bị để phát emoticon.
- `emote_slot`: Tên nhóm asset Slot chưa xác nhận có thanh gán hoặc enum trang bị.
- `emoticon`: Chưa đo quyền phát, chủ thể và bề mặt render của peer.
- `emoticon_effect`: Asset hiệu ứng ngoài exporter. Chưa ghép với SKU của EmoTE/Character.
- `grade`: Chỉ đặt slot khi xác nhận có lựa chọn riêng. Cấp hồ sơ không cấp quyền sở hữu SKU.
- `hat`: test.hat dùng slot head trong fixture Rust; chưa có token hoặc phân nhóm hat của KR.
- `quick`: Slot stack trong kiểm thử nội bộ; không có index hotbar KR đã đo.
- `vehicle`: Ship/* và route pickup đã đo thuộc F1-d; chưa xác nhận slot trang bị F4. [`f1_vehicle_pickup`](#anchor-f1_vehicle_pickup).

<a id="wire-gap"></a>
## Khoảng trống wire và phép đo

- `unmeasured_wire_projection`: Chưa đo ánh xạ SKU/slot sang token và trường wire cho bề mặt này.
- `in_game_219_224_225_313_317_566_pending_capture`: áp dụng cho route túi đồ F4, chưa gán opcode theo từng slot.
  Đo riêng S→C `219/224/225/313/314/315/316/317` và C→S `566`. Nhánh tĩnh `225` đọc tám cặp
  `u32 + chuỗi` mỗi phần tử ngoài; chưa gán tám vị trí đó cho bất kỳ tên slot nào.
  Site gửi `566` trong phân tích tĩnh chưa xác nhận thao tác thật kích hoạt nó hoặc vai trò ACK.
- Đối chiếu A/B từ baseline, đổi đúng một SKU đã biết rồi tháo, vào phòng muộn và tái đăng nhập.
  So UI mỗi bên, cursor và guard của writer/reader, frame có hướng/opcode và DB trước/sau.
  Kiểm riêng standard hết hạn/hết stack và O1 theo token. Lựa chọn O1 nằm trong bộ nhớ,
  hết cùng token và không ghi SQLite.

## Neo đã kiểm và đối chứng

Ảnh IDA: `cf72da112308d9f1ca256df8089594c5f787875310933073bf6bd5ba6f70e63b` (base `0x400000`). Client: `a52545c14b1fea423de655c3163d6360b628d61c1cd472b46fdbc8a9ffe15f4e`.

<a id="anchor-character92"></a>
- `character92` (kr_runtime/confirmed): Client chọn nhân vật nền bằng ordinal cục bộ và khóa u16. Body: `u8 local player, u16 character`.
  [`rust/crates/kr-wire/src/kr00967/room.rs`](../../rust/crates/kr-wire/src/kr00967/room.rs) (`local_character92`); [`docs/protocol/room-membership.md`](../protocol/room-membership.md) (`Lựa chọn nhân vật, đội và lỗi phòng đầy`).
<a id="anchor-character_room_slot"></a>
- `character_room_slot` (kr_runtime/confirmed): Roster 106 và cập nhật 110 dùng cùng khóa nhân vật trong state.
  player106: `packed u32 tại body[12..16]`; slot110: `u8 slot, packed u32, u8 zero`; character: `bit 16..28 inclusive của packed u32`. [`rust/crates/kr-wire/src/kr00967/room.rs`](../../rust/crates/kr-wire/src/kr00967/room.rs) (`player106_character`); [`rust/crates/kr-service/src/room_wire.rs`](../../rust/crates/kr-service/src/room_wire.rs) (`player; slot`); [`rust/crates/kr-wire/src/kr00967.rs`](../../rust/crates/kr-wire/src/kr00967.rs) (`slot110`); [`docs/protocol/room-membership.md`](../protocol/room-membership.md) (`room-player; Lựa chọn nhân vật, đội và lỗi phòng đầy`).
<a id="anchor-character_start121"></a>
- `character_start121` (kr_runtime/confirmed): Khóa nhân vật nền theo slot, độc lập với spawn và team.
  layout: `hàng 26 byte trong blob Start đã giải mã`; slot: `row[0]`; character: `u16 LE tại row[3..5]`. [`rust/crates/kr-wire/src/kr00967/room.rs`](../../rust/crates/kr-wire/src/kr00967/room.rs) (`start121_characters; start121_players`); [`docs/protocol/room-membership.md`](../protocol/room-membership.md) (`rust-selected-player-rounds`); [`docs/protocol/f1-aux-opcodes-and-classc.md`](../protocol/f1-aux-opcodes-and-classc.md) (`f1-aux-character-keys`).
<a id="anchor-own291_progression"></a>
- `own291_progression` (kr_runtime/confirmed): Đối chứng hồ sơ: bật opt-in để chiếu cấp và EXP; chưa định danh slot mỹ phẩm.
  level: `u16 LE tại tail[11..13] tính sau tên trong blob`; exp_permille: `u32 ngoài, ngay sau blob UserSimpleInfo`. [`rust/crates/kr-wire/src/kr00967/account.rs`](../../rust/crates/kr-wire/src/kr00967/account.rs) (`own291; own291_with_progression; local_own291_with_progression`); [`docs/evidence/content/f4-w6-ida-own291-and-hud.json`](../evidence/content/f4-w6-ida-own291-and-hud.json) (`f4a_own291_and_hud_mapping`); [`docs/evidence/content/f4-w8-ida-exp-175-f2.json`](../evidence/content/f4-w8-ida-exp-175-f2.json) (`f4a_own291_exp_permille`); [`docs/evidence/content/f4a-progression-wine.json`](../evidence/content/f4a-progression-wine.json) (`observed_own291_frames; ui_screenshots`).
<a id="anchor-player106_template"></a>
- `player106_template` (design/design): PlayerTemplate giữ hậu tố opaque và nhánh hồ sơ rỗng; không gán nghĩa mỹ phẩm.
  [`rust/crates/kr-wire/src/kr00967/player.rs`](../../rust/crates/kr-wire/src/kr00967/player.rs) (`PlayerTemplate::parse_name_branch; PlayerTemplate::project_name`).
<a id="anchor-db_inventory"></a>
- `db_inventory` (design/design): Schema 8 giữ bảng sở hữu từ Schema 7; tài sản có hạn phải còn hiệu lực khi chiếu. Bảng `player_inventory`, khóa `player_id, sku_id`.
  [`rust/crates/kr-service/migrations/0007_f4_progression_inventory_shop.sql`](../../rust/crates/kr-service/migrations/0007_f4_progression_inventory_shop.sql) (`player_inventory`); [`rust/crates/kr-service/src/accounts/economy/inventory.rs`](../../rust/crates/kr-service/src/accounts/economy/inventory.rs) (`InventoryItem.active`).
<a id="anchor-db_equipped"></a>
- `db_equipped` (design/design): Slot là token catalog nội bộ. Hàng trang bị tham chiếu SKU sở hữu của đúng người. Bảng `player_equipped`, khóa `player_id, slot`.
  [`rust/crates/kr-service/migrations/0007_f4_progression_inventory_shop.sql`](../../rust/crates/kr-service/migrations/0007_f4_progression_inventory_shop.sql) (`player_equipped`); [`rust/crates/kr-service/src/accounts/economy/inventory.rs`](../../rust/crates/kr-service/src/accounts/economy/inventory.rs) (`equip_item; list_equipped`).
<a id="anchor-db_character_preference"></a>
- `db_character_preference` (design/design): Preference nhân vật nền opt-in, riêng với SKU đã equip và không cấp entitlement. Bảng `player_preferences`, khóa `player_id`.
  [`rust/crates/kr-service/migrations/0003_match_history.sql`](../../rust/crates/kr-service/migrations/0003_match_history.sql) (`player_preferences.character_key`); [`docs/protocol/auth-session.md`](../protocol/auth-session.md) (`rust-persistent-character`).
<a id="anchor-db_consumption"></a>
- `db_consumption` (design/design): Schema 8 lưu biên nhận tiêu hao standard; hết stack thì gỡ slot tham chiếu. O1 không ghi DB. Bảng `match_item_consumptions`, khóa `player_id, idempotency_key`.
  [`rust/crates/kr-service/migrations/0008_f4_match_consumptions.sql`](../../rust/crates/kr-service/migrations/0008_f4_match_consumptions.sql) (`match_item_consumptions`); [`docs/architecture/f4-economy-service.md`](../architecture/f4-economy-service.md) (`Session::consume_item_in_match`); [`rust/crates/kr-service/src/accounts/economy/consumption.rs`](../../rust/crates/kr-service/src/accounts/economy/consumption.rs) (`consume_equipped_item_in_match; consume_item`).
<a id="anchor-o1_session_cosmetics"></a>
- `o1_session_cosmetics` (design/design): O1 giữ lựa chọn theo token trong bộ nhớ, không ghi SQLite.
  [`rust/crates/kr-service/src/account_worker/economy/session_cosmetics.rs`](../../rust/crates/kr-service/src/account_worker/economy/session_cosmetics.rs) (`O1SessionCosmetics`); [`docs/architecture/f4-economy-service.md`](../architecture/f4-economy-service.md) (`WebShopAuth`).
<a id="anchor-f1_vehicle_pickup"></a>
- `f1_vehicle_pickup` (kr_static/confirmed): Đối chứng pickup phương tiện F1-d, tách khỏi quyền trang bị trong túi.
  [`docs/protocol/vehicles.md`](../protocol/vehicles.md) (`Nhặt và thay xe`).

## SHA-256 nguồn đã commit

| Nguồn | SHA-256 (byte gốc) |
|---|---|
| [`docs/architecture/f4-economy-service.md`](../architecture/f4-economy-service.md) | `21eb13a8695782cc33ad4cdcf8898f36e42c1e2c2edc2dd1e9d499703d7d5eab` |
| [`docs/evidence/content/f4-bc-spec-summary.json`](../evidence/content/f4-bc-spec-summary.json) | `30f942e0acc085163cc949367166bcf8277e5a2e36e82cd3093dd9683b516848` |
| [`docs/evidence/content/f4-catalog-export-summary.json`](../evidence/content/f4-catalog-export-summary.json) | `fcc631940a2b318d81fdafeb7ef8d0cb085cc56f4344d7c968c7be9b2cbe1b36` |
| [`docs/evidence/content/f4-w2-ida-deepdive.json`](../evidence/content/f4-w2-ida-deepdive.json) | `47d2aa0607da2cbc7a42b2dfdd7a1662be6e47b8249eb6a5d3f0a74a5a7a671a` |
| [`docs/evidence/content/f4-w3-ida-records.json`](../evidence/content/f4-w3-ida-records.json) | `bfbd9b89bc644ce7376288f9fcef69723ec3e8aa291c3c50ef5e5491f1e0b961` |
| [`docs/evidence/content/f4-w6-ida-own291-and-hud.json`](../evidence/content/f4-w6-ida-own291-and-hud.json) | `e5fef91791139a439a8bd78891194abdae5168159a3051e1e1994bf789af0acb` |
| [`docs/evidence/content/f4-w8-ida-exp-175-f2.json`](../evidence/content/f4-w8-ida-exp-175-f2.json) | `bc2b0cab379766b7338cac96343f8b7fb08e635b16d281f9dbbaff4daa270689` |
| [`docs/evidence/content/f4a-progression-wine.json`](../evidence/content/f4a-progression-wine.json) | `a640a40b8def649b5dad5e3014164f48d4a6c01312b2e166ca48079875af5888` |
| [`docs/protocol/auth-session.md`](../protocol/auth-session.md) | `f7382a2ef1ca52f5e8997cabc5a45d42df9531293b8a78a787e866827910c062` |
| [`docs/protocol/f1-aux-opcodes-and-classc.md`](../protocol/f1-aux-opcodes-and-classc.md) | `ddc93e2da901408f0e06bbe3003a7a64c8df52613ce6b15d6c9ddc7e8e0979f7` |
| [`docs/protocol/f4-inventory-cosmetics.md`](../protocol/f4-inventory-cosmetics.md) | `d3e2d80d9a5e0a62bceccc8c1c5d43608e1da6bbc85bce38b790e52998fa677d` |
| [`docs/protocol/room-membership.md`](../protocol/room-membership.md) | `7ea884599b6895639d3ac662cf46485727fd07048fd5b72dd7798e5a702d89ac` |
| [`docs/protocol/vehicles.md`](../protocol/vehicles.md) | `77d9533abf44ff4fcc09d615b3dd71e2c74692c7884bad27027ca1bf81df3583` |
| [`rust/crates/kr-service/migrations/0003_match_history.sql`](../../rust/crates/kr-service/migrations/0003_match_history.sql) | `cb8305293b1c7577da9f739afb4d5d0fbac63ac93d57c4fc6fec8069863dcbbf` |
| [`rust/crates/kr-service/migrations/0007_f4_progression_inventory_shop.sql`](../../rust/crates/kr-service/migrations/0007_f4_progression_inventory_shop.sql) | `1155d06d5e7798c899bbf67660e8d5745323566c982abd4d3b99d75dd52e7da0` |
| [`rust/crates/kr-service/migrations/0008_f4_match_consumptions.sql`](../../rust/crates/kr-service/migrations/0008_f4_match_consumptions.sql) | `8b5211949a7dd8d14387f1ceff9c568186a4cf40f0fccbcf90831934cfaa0021` |
| [`rust/crates/kr-service/src/account_worker/economy/session_cosmetics.rs`](../../rust/crates/kr-service/src/account_worker/economy/session_cosmetics.rs) | `0a18b9924fddee8db01356306ad3abac287a8ad9039d4431de9cf97a11fcd6f1` |
| [`rust/crates/kr-service/src/accounts/economy/consumption.rs`](../../rust/crates/kr-service/src/accounts/economy/consumption.rs) | `f1e1ba4fc13df730302b3ff4e2b54009c2e241c89d487da76b794f6029d33c8f` |
| [`rust/crates/kr-service/src/accounts/economy/inventory.rs`](../../rust/crates/kr-service/src/accounts/economy/inventory.rs) | `49fc782e4b1d0ccdd8672f6aed540b433e2dbd48ac8be61270e8452f3c015dce` |
| [`rust/crates/kr-service/src/accounts/economy/tests/consumption.rs`](../../rust/crates/kr-service/src/accounts/economy/tests/consumption.rs) | `e7c3281d328fe9020ed9f0400a319f34a4c3ee63661abd4465d6d1fd1c7d9127` |
| [`rust/crates/kr-service/src/f4bc_runtime_tests.rs`](../../rust/crates/kr-service/src/f4bc_runtime_tests.rs) | `51c35749489ce7c841d827226503e198dc295fec25113b006bb26444ebd80899` |
| [`rust/crates/kr-service/src/room_wire.rs`](../../rust/crates/kr-service/src/room_wire.rs) | `4bd8473f94393106cdaf976425f91b1e0183eb0e12535b5795878250af73129c` |
| [`rust/crates/kr-service/src/web_shop/tests.rs`](../../rust/crates/kr-service/src/web_shop/tests.rs) | `9c642a6f1beb8e80855f7f6e315f05ed585f8a4b7403326f7f433aed9d21d33d` |
| [`rust/crates/kr-wire/src/kr00967.rs`](../../rust/crates/kr-wire/src/kr00967.rs) | `ee65445db146c5cf0532ae380d2035da0be75a6a4b209ded9b689b92594ba607` |
| [`rust/crates/kr-wire/src/kr00967/account.rs`](../../rust/crates/kr-wire/src/kr00967/account.rs) | `d55678bf3a9bfafdf1bb480f75c6dae11495a45b9baa5c6077bdbbd4a671d226` |
| [`rust/crates/kr-wire/src/kr00967/player.rs`](../../rust/crates/kr-wire/src/kr00967/player.rs) | `00f2e7d29eeb316355c296564a20babacc3b52862e0233cd9cd83b65fdbf0873` |
| [`rust/crates/kr-wire/src/kr00967/room.rs`](../../rust/crates/kr-wire/src/kr00967/room.rs) | `b9ea54de373a8e4296f6db31e4f79639572474c3053b85e9449ce833401b2e6e` |
| [`server/profiles/f4-economy-standard.json`](../../server/profiles/f4-economy-standard.json) | `846a3988dcb02f89c4674d83444844749d0542c9b53eeba4f848abcba9178ad5` |
| [`tools/kr_f4_catalog_export.py`](../../tools/kr_f4_catalog_export.py) | `de562d4c021e61f717207a4acf660b389b261fc52df6f67d9067c23cdac42ae2` |
