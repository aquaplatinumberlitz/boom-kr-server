# F3 PvE: jump table và `MonAction` của KR 00967

Tài liệu này là kết quả RE tĩnh trên IDB KR 00967 đã pin SHA-256
`6be15a1fccea0837ee95c50893345e9f13dc217b0fa570975b5c284f0d547fd1`. Chỉ dùng
bản sao scratch; IDB nguồn không bị sửa. Nguồn đối chiếu là [neo F2–F5–F3](f2-f5-f3-ida-anchors.md),
[bảng dữ liệu client](../content/client-data-tables.md#monaction-runtime) và [CP405](../evidence/checkpoints/records/CP405.json).

## Dispatcher và bảng nhảy

`CGameStage::OnPacket` tại `0xE5D3A0` gọi virtual reader của packet để lấy opcode. Với opcode `<= 306`,
code trừ `20`, kiểm tra phạm vi `0..254`, đọc `byte_E5DE9C[index]` và nhảy qua bảng địa chỉ tại
`0xE5DE9C`/`jpt_E5D409`. Vì vậy nhãn Hex-Rays là index của switch; wire opcode là `case + 20`.
Các entry không mặc định đã đọc được từ bảng và decompiler như sau:

| case | wire | handler | ghi chú tĩnh |
|---:|---:|---:|---|
| 5 | 25 | `0xE65A10` | nhánh component/script |
| 64 | 84 | `0xE6E1A0` | component fallback |
| 90–91 | 110–111 | `0xE5E2C0`, `0xE6DF60` | stage handlers |
| 103 | 123 | vtable `this + 0x2D4` | component dispatch |
| 112 | 132 | `0xE610C0` | đồng bộ/script |
| 119–122 | 139–142 | `0xE662B0`, `0xE66320`, `0xE66360`, `0xE66380` | stage state |
| 132–140 | 152–160 | `0xE5E640`, `0xE5E6D0`, `0xE5E860`, `0xE67640`, `0xE67770`, `0xE5E600`, `0xE5F450`, `0xE5E8F0`, `0xE5EEC0` | object/effect và stage |
| 142, 144–148 | 162, 164–168 | `0xE5EFC0`, `0xE5F2C0`, `0xE63260`, `0xE875E0`, `0xE63E50`, `0xE63F10` | pickup/object |
| 150–153 | 170–173 | `0xE85B60`, `0xE64310`, `0xE64570`, `0xE646E0` | object/state |
| 155–158 | 175–178 | `0xE5F610`, `0xE77D20`, `0xE6DD60`, `0xE77DD0` | script/timer |
| 160–165 | 180, 181, 183–185 | `0xE72490`/vtable, `0xE652F0`, `0xE65350`, `0xE654F0`, `0xE67470` | stage/script |
| 171 | 191 | `0xE64070` | component fallback |
| 176–181 | 196–201 | `0xE73760`, `0xE751F0`, `0xE74410`, `0xE72EC0`, `0xE683C0`, `0xE678A0` | PvE/script cluster |
| 189–190 | 209–210 | `0xE68280`, `0xE67A60` | stage state |
| 195, 200–202 | 215, 220–222 | `0xE7F500`, `0xE65D40`, `0xE65F20`, `0xE65E40` | stage/component |
| 254 | 274 | `0xD73100` | component |

Các case 9, 80, 135, 136 và 177 mà tài liệu cũ gọi tên là **case** phải đọc đúng theo quy tắc trên:
chúng là handler `0xE6AA90`, `0xE6D730`, `0xE67640`, `0xE67770`, `0xE751F0`, tương ứng wire
29, 100, 155, 156 và 197 trong nhánh `opcode - 20`. Không được biến nhãn case thành opcode wire khác
nếu chưa kiểm tra nhánh opcode và bảng nhảy.

`0xE67640` đọc một `u8` qua vtable `a2 + 60`, một `u8` qua `a2 + 56`, rồi gọi RTTI
`IEatObject`/`IAbilityObject` vtable `+36`. `0xE67770` đọc cùng hai byte và gọi vtable `+52`.
`0xE751F0` đọc `u32` (`a2 + 68`), `u8` (`a2 + 60`), rồi hai `u32` (`a2 + 68`), nhân hai giá trị
thời gian với `1000`, đặt cờ nội bộ về `0`, và xử lý danh sách người chơi/đội. Đây là field read/write
đã xác minh tĩnh; ý nghĩa gameplay và body wire đầy đủ vẫn candidate.

## Quyền chạy `MonAction` và gói 204

Owner gate ở `0xB91F60`:

```text
if (object->vtable[+24]() != 0 && sub_11656A0(this - 184) == 0)
    return sub_1017890(packet);
return -1;
```

Như vậy nhịp hành động chỉ chạy khi object hợp lệ và cờ remote/local tại `0x11656A0` bằng `0`.
`0x73C790` → `0x115D440` → `0x115DD30` tạo `CSyncGameObject` cho StageClear và đăng ký owner `-1`.

Bộ đồng bộ `0x115CB10` nhận opcode `204`, đọc `u16` tại reader vtable `+64` (object ID) và `u8`
tại `+60` (slot), rồi gọi `0x115D2E0(this, object_id, slot)`. Hàm `0x115D2E0` duyệt các slot
`0 .. sub_13B07C0(sub_55C7E0())-1`; khi `sub_1240B00(i) == slot`, nó đặt cờ local qua
`sub_11656C0(0)`, reset state `sub_1165780(0)`, lấy object sequence rồi cộng `300`, lưu bằng
`sub_11652B0`, đăng ký object và ghi log `Delegate ObjId[%d] to Local SlotIdx %d`. Nếu không tìm thấy slot,
nó đặt cờ remote `sub_11656C0(1)` và chuyển sang đường object không thuộc máy này.

Các field và nhánh trên là `kr_static/confirmed`. Slot server chọn cho object, thuật toán owner-slot
nguyên bản và body relay giữa các client vẫn là `candidate`.

## Writer C→S 132 và 180

`0xF53320` là writer `C->S 132` của action `Say`: kiểm tra RTTI `CScriptObject` → `CGONpcObject`,
kiểm tra danh sách/điều kiện local, chọn ngẫu nhiên một entry, gọi `sub_1164C10(132)`, rồi ghi:
`u32` session/owner từ `0xE704C0`, `u16` object ID từ `sub_1165180(v14)`, `u16` action kind `0`,
object/target ID qua `sub_14EC5E0`, sau đó đóng packet bằng `sub_1164D50` và `sub_1164C80`.
`0xF50DD0` là dispatcher nhiều bước của `CScriptObject`: các nhánh đều dùng 132 và ghi các kind
`1, 10, 9, 11, 12, 13, 14, 15, 16, 17, 18, 19, 21, 22`; field phụ là `u8`, `u16` hoặc object
ID theo từng bước. Đây là chứng cứ client báo kết quả bước, không phải chứng cứ server relay.

`0x629940` là writer `C->S 180` cho các kết quả MonAction/CScriptObject. Nó gọi `sub_1164C10(180)`
và lần lượt ghi object ID (`sub_1165180`), kind `0`, `4`, `3`, `5`, `6`, `8`; kind `4/3/5` có hai
`u16`, còn kind `6` có selector `14/15/16` và một `u8`. `0x6292A0` ghi kind `9` và object target;
`0x627DE0` ghi kind `10`, object ID, hai object/field phụ. Các `180` writer này đã xác minh lệnh
và field read/write ở mức decompiler; ngữ nghĩa từng kind và relay body vẫn `candidate`.

## Giới hạn và bước kiểm tiếp

Đây là RE tĩnh (`source=kr_static`, `status=confirmed` cho địa chỉ, reads/writes và dispatch). Chưa có
Wine capture. Cần capture hai client F3 với frame raw để xác nhận độ dài/body của 204, 132 và 180, client
nào nhận quyền, thứ tự relay, và mapping object ID/slot. Không suy server owner algorithm hoặc relay body
từ encoder client. Các kết luận này không đóng gate runtime.
