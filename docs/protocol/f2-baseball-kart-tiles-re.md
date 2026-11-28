# P241 — B2: tile `Baseball` và `Kart` fail-closed

## Phạm vi và nguồn

Đợt này chỉ đọc build KR `00967`: IDA database bản sao có SHA-256
`6be15a1fccea0837ee95c50893345e9f13dc217b0fa570975b5c284f0d547fd1`, `TileInfo/TileInfoList`
trong pack `fx` có SHA-256 `51b738448f5bb0c9dc54c0ef6782bd612561660f256717dc5df1d59c2f5ec795`,
và tám map `Kart` cùng mười map `Baseball` trong census. Reader là `tools/kr_pack_export.py`,
`tools/export_map_profile.py` và `tools/kr_f1e_script_audit.py`; các hash reader, pack và payload
được ghim trong [evidence](../evidence/content/f2-baseball-kart-tiles-re.json). Mọi tuple trong evidence
là đúng 18 số nguyên `TileInfo`, theo thứ tự little-endian của `Data`, kèm chỉ số `ObjectTable`.

Map `Baseball01..10` đều là lưới `15×13`, có `mapscript` với `time/state/wave/bomb`. Map `Kart02`,
`Kart05`, `Kart06`, `Kart07`, `Kart14`, `Kart15`, `Kart16`, `Kart17` đều là `15×13` và không có script.
Các kết quả này là `kr_static/confirmed`; không dùng tên map để suy mã mode hoặc wire.

## Đường đọc IDA

`B9D8E0` khởi tạo `CGOTileGround` và chép các trường `TileInfo` vào object, gồm các trường tương ứng
`+36`, `+40`, `+48`, `+52`, `+56`, `+60`, `+62`. `B9DEA0` đọc word `+62`; caller được thấy là đường
vẽ nền và `CBulaComponent`. `ED2350` chỉ trả hủy bom cho type `0` khi word `+124 >= 10000`, còn type
`1`, `2`, `3`, `4` trả false. `BA1420` đọc `TileInfo +36`, nên marker riêng của lốp không thể coi là
metadata trang trí.

`EB48E0` chọn `CGOTileLap` khi byte thấp của trường `+60` khác không. `E9F760` chỉ trả object khi
RTTI là `CGOTileGround` và `CGOTileLap`; đây là nhánh vật lý đặc biệt, không phải nền tĩnh. `E90A80`
đọc `+44` cho đường tia của type `3`, `ED23E0` giải mask hướng của type `2` và byte hướng của type
`1`, còn `EB4C10` tạo biến thể type `3` khi `+52 == 101`. `F09BC0` là constructor `CKartComponent`;
`F0A310` khởi tạo trạng thái item/đua của component. Các consumer này xác nhận cần giữ riêng luật
đua, gas và lốp; chúng không xác nhận wire hay công thức tốc độ.

## Phân loại

Evidence liệt kê đầy đủ mọi tile xuất hiện trong 18 map. Trường `classification.*.static_verified`
là object ánh xạ tên tile tới `object_table_index` và tuple 18 số; các tile này là ground, box hoặc static
đã khớp rule type 0/1/2 hiện hành và không nằm trong `unmapped_tiles`, nên có thể tiếp tục đi qua
adapter terrain đã kiểm chứng.

Trường `classification.*.fail_closed_dynamic_or_special_physics` chứa toàn bộ tile còn lại và tuple
chính xác của từng tile:

- `Tile70_Ground/Tile01`, `Tile02`, `Tile04`, `Tile05`, `Tile06`, `Tile07`, `Tile08`: có cờ `+60`
  khiến loader đi qua `CGOTileLap`. Dù tên là `Ground`, chưa có bằng chứng rằng chúng chỉ là nền
  tĩnh; giữ fail-closed cho Baseball.
- `Tile12_Ground/LapTile01..04` theo hai hướng: có cờ đường đua và giá trị `+60` riêng. Chưa chốt
  thứ tự vòng, hướng, chạm ô hay tác động lên tốc độ.
- `Tile12_Ground/Race00..13` và `Start_Horz`, `Start_Horz1`, `Start_Vert`, `Start_Vert1`: có cờ
  `+52 == 1` và giá trị đánh dấu đường đua/start. Không chiếu thành ground thường.
- `Tile12_Passable/GasRight00`, `GasRight01`: type `3`, `+52 == 100`; đây là passable/hiệu ứng gas,
  không phải vật cản tĩnh. `EB4C10` cho thấy trường này có nhánh factory riêng.
- `Tile12_Static/Tire00..04`: type `1`, kích thước `1×1`, nhưng `+36` lần lượt là `30`, `50`, `70`,
  `80`, `95` và các trường sau đó mang marker `0xCCCCCCCC`. `BA1420` đọc marker này; chưa biết
  lực cản, va chạm hay tương tác với kart.

Không có tile trong nhóm fail-closed được thêm vào adapter hoặc dùng để sinh `ray_ports`, tốc độ,
lap, boost, base, strike, gas, tire hay điểm số. Việc cùng xuất hiện trong map chỉ chứng minh payload
và tên asset, không chứng minh client đã thực thi luật mode.

## Giới hạn và bước đo tiếp

Đợt này không chạy Wine và không có capture. Vì vậy `wire_mode`, objective, thứ tự lap/start/finish,
quyền server, tốc độ, boost, gas, tire và va chạm đều còn `unknown`. Bước tối thiểu tiếp theo là capture
hai client KR 00967 qua Wine cho từng họ, giữ raw/capture và kiểm riêng các nhánh `CGOTileLap`,
`CKartComponent`, gas và lốp trước khi mở bất kỳ tile nào trong server adapter. Baseball cũng cần capture
script `wave/bomb` trước khi coi map đã terrain-ready.
