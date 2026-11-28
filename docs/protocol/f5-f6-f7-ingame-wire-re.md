# P233 — RE tĩnh wire F5, F6 và F7 của KR 00967

Nguồn là bản sao IDA của client KR 00967, image base `0x400000`, SHA-256 `6be15a1fccea0837ee95c50893345e9f13dc217b0fa570975b5c284f0d547fd1`. Phân tích dùng IDA headless, IDAPython và Hex-Rays; không chạy client và không dùng capture mạng. JSON bằng chứng đi kèm giữ VA, hướng đọc/ghi và trạng thái cho từng phát hiện.

## F5 — CHOCOPacket và xã hội trong sảnh

`CStage::OnPacket` ở `0xD68C90` dispatch các opcode 467, 474, 478, 257, 273, 284, 640 và 674. Opcode 474 tại `0xD769D0` đọc theo thứ tự `str, str, u32, u32, str`; nhánh chặn lời mời gửi C→S 633 qua builder `0x1164C10`. Cặp đôi 670 được tạo tại `0x583530` và `0x583E80`; body của request còn chưa gán được nghĩa trường.

`CChattingStage::OnResConnectToSvr` ở `0xD58760` đọc `u8 ok, u8, u32, u32 ip, u16 port`, rồi gọi `0x13AD4D0`. IDA có RTTI `CHOCOPacket` và sáu lớp: `P_CHOCO_CHATMSG`, `P_CHOCO_GUILDCHATMSG`, `P_CHOCO_SHOUTMSG`, `P_CHOCO_WHISPERMSG`, `P_CHOCO_TEAMCHATMSG`, `P_RES_CHOCO_EVENT`. Reader/writer của chúng đã được ghim trong JSON; ví dụ shout ghi bốn byte, count byte và các phần tử, còn event ghi hai byte và reader chặn cả hai giá trị ở `0..1`. Đây là chuỗi field của lớp client, chưa phải framing wire hoàn chỉnh hay opcode chat.

Các symbol Nexon Messenger cho note/memo có mặt tại `0xAB8707`, `0xAB8717`, `0xAB8B57`, `0xAB8B67`, `0xAB8B77` và `0xAB8B87`. `CLobbySquareStage::OnPacket` ở `0x82A530` có các nhánh user list, room, chat/plaza và event; `OnSquareUserList` là `0x82DA60`. Payload movement, chat quảng trường, note và quyền server vẫn để `candidate` hoặc `unknown`.

## F6 — event packet và guard phía client

`CGameStage::OnPacket` ở `0xE5D3A0` dùng các bảng nhảy `0xE5DF9C` và `0xE5E0DC`. Các nhánh liên quan event/reward/status gồm 321→`0xE6AA90`, 322/323→`0xE6AEB0`, 548→`0xE69440`, 549→`0xE69D30`, 553→`0xE69D80`, 566→`0xE6D830`, 568/569→`0xE75C90`/`0xE76ED0`, và 574/576/577→`0xE75CB0`/`0xE763A0`/`0xE760B0`. 321 có guard event id không vượt `0x14E`, giới hạn reader phụ `<=24`, và chỉ phát C→S 174 khi một stage entity chấp nhận event. 548 đọc một chuỗi byte/dword và nhánh reward; 553 đọc `u8, u16, u8`.

Ở dispatcher sảnh, 408/413 đi `0xD7B2B0` với status branch `0..5`; 409 đi `0xD7B5D0`; 414 đi `0xD7C2C0` và chỉ tiếp tục khi mode là 11, 14 hoặc 21; 418/419 đi `0xD852C0` với prefix `u8, u16, u32` và phần còn lại phụ thuộc nhánh. Đây là guard và consumer của client, không xác nhận giá trị thưởng, lịch event hay authority của server gốc. `CLobbyStage` có vtable và `OnResConnectToSvr`, nhưng chưa có contract event wire đủ để chốt.

## F7 — CustomMap/UserMap

IDA nhận diện dispatcher `CustomMap` tại `0x5A3BF0`: packet type là 666 và subtype switch có dải 19–38. RTTI có các request/response check hash, publish, load, remove, load-new/best/good/search, Festa vote/report/good, load-my-PC và error. Một số cặp reader/writer đã ghim: `P_REQ_CHECK_HASH_VALUE` (`0x4D68F0`/`0x4D6980`), `P_REQ_PUBLISH` (`0x4D6B80`/`0x4D6C10`), `P_REQ_LOAD` (`0x4D6E60`/`0x4D6EC0`), `P_REQ_LOAD_SEARCH` (`0x4D6FC0`/`0x4D7080`) và `P_REQ_GOOD` (`0x4D7350`/`0x4D73E0`).

Editor và local map metadata đi qua `0x508370`, `0x5091F0`, `0x4621C0` và object serializer `0x57FD80`, với các trường tên, mô tả, URL name, cờ map mới, kích thước, slot, multiserver, level và nickname. UI upload có state/constructor `CCustomMapUpload` tại `0x9BFC80` và handler `0x5B11D0`. `CUserMap` có RTTI `0x19C7E80` và vtable `0x175E38C`; một reader metadata phòng tại `0xD72FD0` được gọi từ opcode 273.

Các subtype 666 chưa được nối chắc với numeric wire opcode, outer framing, blob tile, quyền upload/chọn hoặc đồng bộ map phòng. Không dùng schema chuẩn hóa F7 làm layout wire KR.

## Giới hạn và bước đo

Mọi mục `candidate` trong hồ sơ này chỉ là hợp đồng phía client. Cần capture Wine sống trong mạng cô lập với stub/server cục bộ để ghi request và response từ mở UI, gửi gói, lỗi từ chối, chọn map, sync metadata và bắt đầu ván. Chỉ sau khi capture khớp reader/writer và đường dispatch mới được nâng trạng thái; các mục `unknown` vẫn phải giữ unknown nếu phép đo chưa phân biệt được.
