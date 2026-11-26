# Thiết kế đo quyền PvE F3-2

Nguồn mẫu là [ma trận quyền thực thi script F3-1](f3-script-authority-matrix.md) và
[neo IDA F2/F5/F3](../protocol/f2-f5-f3-ida-anchors.md) của build `KR00967`.
[Ma trận probe JSON](../evidence/content/f3-wine-probe-plan.json) giữ hash nguồn,
thao tác, đối chứng và tiêu chí cho **12 map ưu tiên và 7 map bổ sung**.
Đây là thiết kế thử; hành vi trên wire là `unknown` / `pending_wine_capture`.

Factory `0xEBA2D0` và guard `RollAttack::MonsterAttackReq` là neo tĩnh. Guard đọc cờ host
`*(this + 157)` hoặc so `attackSeq` tại `*(this + 316)`; chưa cho biết bên nào quyết định HP,
đợt quái hay kết quả. Các số 9, 80, 135, 136, 177 tại `CGameStage::OnPacket` (`0xE5D3A0`) là nhãn tĩnh.
Các số 9/80/135/136/177 là nhãn tĩnh từ nguồn IDA. Với mỗi frame bắt được, đối chiếu opcode giải mã từ wire, giá trị vào CGameStage::OnPacket, chỉ số jump table (kiểm phép trừ 20) và VA handler thực chạy. Không lấy nhãn làm opcode wire hoặc tự cộng 20.

1. Kiểm lease Wine, cô lập outbound và bật capture TCP/UDP cùng log Wine của cả hai client và log server.
   Dùng bản sao KR 00967 đã pin hash và server cục bộ. A tạo phòng, B vào, A bắt đầu; xác minh map cả hai tải.
2. Thực hiện thao tác và đối chứng của từng map trong các lượt mới. B tạo và bắt đầu phòng ở lượt đảo host.
   Giữ cùng người thao tác nếu cửa vào cho phép. Thu dấu sinh quái, đòn đánh, HP, trigger, qua màn và thưởng
   nếu có. Chỉ thử mẫu bổ sung khi cần phân biệt cụm script khác.
3. Giữ frame, log và dấu UI theo thời gian và ID đối tượng trong `local/`. Phân biệt bên phát, bên áp dụng
   và bên ghi bền. Nếu cửa vào chưa có hoặc chỉ hỗ trợ chơi đơn, ghi giới hạn thay vì ép chạy 2P.
   Dọn bản sao và môi trường tạm theo [quy định sau mỗi lượt](../evidence/owner-directives/run-cleanup.json).

| Quyền còn `unknown` | Cách phân biệt |
|---|---|
| Sinh quái, chuyển đợt và đòn quái | Ghép ID quái, đợt và guard với thông điệp đầu tiên và lần đổi trạng thái ở phía nhận, rồi so khi đảo host. |
| Sát thương và HP | So đòn trúng/hụt theo cùng ID; tìm nơi quyết định HP hoặc quái chết, tách khỏi hoạt ảnh. |
| Trigger và chuyển màn | So thao tác đủ và thiếu điều kiện; tách thay đổi UI cục bộ khỏi sự kiện được bên khác áp dụng. |
| Qua màn và thưởng | So lượt hoàn tất với lượt còn thiếu mục tiêu; ghi biến động tài khoản trước, sau và sau đăng nhập lại. |

Nếu dấu vết đi theo vai host, kiểm nó là lệnh đề xuất hay trạng thái bên nhận áp dụng. Nếu frame đến từ
server, kiểm server chuyển tiếp hay tự tính/kiểm tra kết quả. Một probe có thể đạt tiêu chí thu dấu vết mà
vẫn chưa phân được quyền. Log server cục bộ mô tả emulator; kết quả không xác nhận luật của server Nexon.

Hướng thiết kế (`design`): sau khi đo, tách đề xuất của client, quyết định của chế độ trong `RoomRules`,
và việc ghi thưởng qua dịch vụ tài khoản. Chỉ đưa một luồng vào server sau khi có đối chứng cho quyền của
luồng đó. Nếu thiết kế cần ghi thưởng, gắn giao dịch với kết quả ván đã kiểm và chống ghi lặp;
công thức thưởng, giá trị gói và hành vi chưa đo tiếp tục giữ `unknown`.

Sinh lại: `python3 tools/kr_f3_probe_plan.py --write`. Kiểm hai đầu ra từ JSON đã lưu:
`python3 tools/kr_f3_probe_plan.py --check`.
