# Xuất cấu hình kinh tế và catalog F4-0

[Exporter](../../tools/kr_f4_catalog_export.py) đọc các pack KR 00967 đã ghim SHA-256 và tạo một file
JSON cho dịch vụ tài khoản và web shop phía server. Công cụ chỉ đọc pack và không khởi động client
hay gọi mạng. [Tóm tắt kiểm chứng](../evidence/content/f4-catalog-export-summary.json) lưu số đếm và
hash, không lưu bản xuất thật.

## Cách dùng

```bash
python3 tools/kr_f4_catalog_export.py --game-dir "$GAME_DIR" --out f4-standard.json
python3 tools/kr_f4_catalog_export.py --game-dir "$GAME_DIR" --unlock-all --out f4-o1.json
python3 tools/kr_f4_catalog_export.py --game-dir "$GAME_DIR" \
  --overrides f4-overrides.json --enable-economy --levels 3 --out f4-custom.json
```

`--out` phải là đường dẫn mới bên ngoài thư mục game. Bỏ `--out` để nhận JSON trên stdout. Kết quả
gồm `economy_policy`, `shop_catalog` và `summary`. Để cấu hình Rust, chép hai trường đầu vào
`diagnostic_two_lobby.account_admission` trong [cấu hình server](../OPERATIONS.md#accounts-rooms-and-audit).
Catalog có mục yêu cầu `profile_mode: "database_zero"`; bật thưởng trận còn yêu cầu
`persist_matches: true`. `summary` là metadata của bản xuất, nằm ngoài cấu hình Rust.

| Preset | Hành vi |
|---|---|
| `standard` | Dùng [file cấu hình mặc định](../../server/profiles/f4-economy-standard.json) và override nếu có. Thưởng trận mặc định tắt; bật bằng `--enable-economy` hoặc trong override. |
| `--unlock-all` (O1) | Đặt giá mọi SKU về 0 sau khi kiểm override; giữ nguyên grant và trạng thái bật/tắt từng SKU. Quyền dùng không cần mua thuộc về `CatalogAccess.unlock_all` của phiên O1. Exporter chỉ tạo dữ liệu. |

Số đếm gồm cả SKU đã tắt. Catalog được sắp theo `sku_id`; `summary.catalog_sha256` băm đúng object
`shop_catalog` dưới dạng JSON UTF-8 gọn, khóa sắp tăng, có một ký tự xuống dòng LF ở cuối.
`summary.level_count` và `summary.unlock_all` ghi số cấp và preset đã chọn.

## Nguồn dữ liệu và phạm vi

| Nguồn pack | Ánh xạ SKU |
|---|---|
| `FxContents/Attr/BomberAttr` | `character.<f0>`; khóa phân biệt các dòng trùng tên. |
| `FxContents/Attr/ItemAttr` | `consumable.<kind>` cho các dòng được chọn theo tên trong `consumables` của preset. Tiền rơi, vật phẩm bản đồ nằm ngoài danh sách và bảng phương tiện không tạo SKU. |
| `CharTE/ObjectTable/CharTEObjectTable` | Các nhóm `AvatarDeco`, `BombDeco`, `CharDeco`, `FishingDeco`, `GradeDeco` và `LadderDeco` vào `decoration`. |
| `EmoTE/ObjectTable/EmoTEObjectTable` | `IDDeco` và `Slot` vào `decoration`; `Character` và `Fanfare` vào `emotion`. |
| `fx/ObjectTable/ObjectTable` | Các mẫu `Pet/Pet_<số>` vào `collection`; bỏ các biến thể và hiệu ứng. |

SKU từ ObjectTable có dạng `<category>.<pack>.<group>.<object>`. Chỉ mục cho biết tên asset; phân nhóm,
grant, slot và khả năng bán là lựa chọn thiết kế. Đối chiếu [kiểm kê pack](../CONTENT.md#bảng-dữ-liệu-ngoài-attr)
để xem ranh giới dữ liệu. Exporter chưa xác nhận cửa hàng gốc bán các mục này hay client KR sử dụng chúng
qua wire nào. Luật rơi đồ trong trận nằm ngoài bản xuất F4-0.

Preset dùng `total_exp` từ bảng wiki trong `boomkr-re/data/kr/exp_table.json`. Hai hàng giảm tại cấp
346 và 576 được sửa bằng `total_exp` trước đó cộng `need_exp` tương ứng; kết quả cũng khớp mốc sau trừ
`need_exp` của mốc đó. Giữ nguyên các mốc tăng còn lại. Các giá tiêu hao có nguồn dùng giá mua lẻ wiki;
riêng Super Hedgehog dùng giá cho gói 10 chia cho 10. Giá còn lại, thưởng trận và phần kéo dài đường
EXP là giá trị thiết kế; cấu hình chúng trong JSON thay vì sửa Rust.

## File ghi đè

Ví dụ `f4-overrides.json` cho lệnh thứ ba ở trên:

```json
{
  "economy_policy": {
    "win": {"exp": 120, "lucci": 150},
    "loss": {"lucci": 20},
    "draw": {"lucci": 40},
    "grant_on_disconnect_or_left": false,
    "level_thresholds": [0, 250, 1200]
  },
  "items": {
    "character.0": {
      "price_lucci": 800,
      "disabled": true,
      "grant": {"kind": "timed", "duration_seconds": 86400}
    },
    "consumable.151": {
      "price_lucci": 1500,
      "grant": {"kind": "stackable", "max_stack": 99},
      "equip_slot": null
    }
  }
}
```

- `economy_policy` là bản ghi đè từng trường. Có thể ghi đè riêng `exp` hoặc `lucci` của từng kết quả.
  Thưởng và giá nhận số nguyên từ 0 đến `10^12`; `enabled` và `grant_on_disconnect_or_left` nhận boolean.
  `--enable-economy` thắng `enabled: false`.
- `level_thresholds` thay cả danh sách: từ 1 đến 10.000 mốc EXP nguyên, tăng nghiêm ngặt từ 0, mỗi mốc
  không quá `i64::MAX`. Nếu dùng cùng `--levels N`, số mốc phải bằng N. Khi chỉ dùng `--levels N`, công
  cụ lấy N mốc đầu của preset hoặc nối thêm theo bước `extra_level_exp`.
- `items` dùng khóa `sku_id` đã có trong bản xuất standard. Có thể đổi `price_lucci`, đặt `disabled`
  (`false` bật lại SKU), đổi `equip_slot` (`null` hoặc 1–32 ký tự `A-Z a-z 0-9 _ . -`) và thay toàn bộ
  `grant`. Các dạng grant là `{"kind":"permanent"}`, `{"kind":"stackable","max_stack":N}`
  (1 đến `u32::MAX`) hoặc `{"kind":"timed","duration_seconds":N}` (1 đến `10^12`).
- Công cụ từ chối SKU không tồn tại, khóa lạ, khóa JSON trùng và giá trị ngoài giới hạn.
