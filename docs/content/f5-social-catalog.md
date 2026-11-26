# Kiểm kê tài nguyên xã hội F5-0

Nguồn: [kiểm kê pack](../CONTENT.md), [census JSON](census.json) và [bảng xếp loại A/B/C](classification.md). [Catalog JSON](../evidence/content/f5-social-catalog.json) giữ hash đầu vào và IDX nguồn.
Build `KR00967` có **5 phân hệ, 7 nhóm, 127 mục** trong phạm vi F5-0.

Số mục lấy từ index pack. Tên và số mục giúp chọn điểm bắt đầu kiểm tra; chúng chưa xác nhận mã, bố cục gói tin, quy tắc hay quyền quyết định của server KR.

| Phân hệ | Nhóm tài nguyên (số mục) | Lớp | Wire F5 cần xác nhận |
|---|---|---|---|
| `Chatting` | `FxDesktop/Chatting` (5), `FxDesktop/WhisperLog` (3) | B | `unknown`: Thì thầm, danh sách chặn và báo cáo lạm dụng |
| `Messenger` | `FxDesktop/GlobalUI:BtMessenger` (2) | C | `unknown`: Lời mời, danh sách bạn và trạng thái hiện diện |
| `Guild` | `FxDesktop/Etc:Guild` (2), `FxDesktop/Guild` (26) | C | `unknown`: Danh sách, thành viên và quyền quản lý bang |
| `Couple` | `fx/Couple` (28) | C | `unknown`: Lời mời, xác nhận và trạng thái cặp đôi |
| `Ranking` | `FxContents/UXCAJJangRatingMark` (61) | C | `unknown`: Yêu cầu bảng hạng và dữ liệu trả về |

Chat sảnh/phòng đã có [bằng chứng L1](../protocol/lobby-profile.md#rust-patched-room-chat) trong phạm vi client vá 949x. Bằng chứng đó chưa xác nhận thì thầm, chat bang hội hoặc chặn.
Hàng `Ranking` kiểm kê nhóm CAJJang theo bảng xếp loại; tài nguyên Ladder thuộc phạm vi F2.

Schema 9 dùng giới hạn nội bộ: mỗi người có tối đa 100 bạn, 100 tài khoản chặn và 100 lời mời đang chờ cho mỗi chiều gửi hoặc nhận. Chặn ở một chiều ngăn gửi riêng ở cả hai chiều.
Chat cho phép tối đa 512 byte mỗi tin và 5 tin mỗi cửa sổ UTC 10 giây chung cho chat và thì thầm. Audit lưu lý do, số byte và số tin đã được duyệt trong cửa sổ; không lưu nội dung tin nhắn.
Bang hội cho gia nhập trực tiếp, tối đa 100 người và mỗi người chỉ thuộc một bang. Tên bang dùng chữ cái ASCII, chữ số và khoảng trắng; khóa duy nhất không phân biệt hoa thường.
Bảng hạng nội bộ dùng thứ tự cấp, EXP và số trận thắng đã hoàn tất giảm dần, rồi ID người chơi tăng dần. Couple hiện chỉ có kiểm kê tài nguyên.

Để xác nhận wire F5, cần đối chiếu điểm gọi tĩnh của từng thao tác với capture KR 00967 trong mạng cô lập, có thao tác đối chứng và trạng thái trước/sau.

## Tái lập

```bash
python3 tools/kr_f5_social_audit.py --generate
python3 tools/kr_f5_social_audit.py --check
```
