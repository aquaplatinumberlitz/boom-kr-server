# Hướng dẫn chơi BOOM PLAY

<!-- overview -->
## Hai chế độ chơi

**Máy chủ** dành cho cộng đồng có máy chạy riêng. Bạn dùng tài khoản của máy đó và giữ tiến trình theo cấu hình của họ.

**Chơi với bạn bè** (Sắp có) dành cho những buổi chơi riêng. Máy của một người bạn tạo phòng, không cần tài khoản.

<!-- compare -->
## So sánh hai chế độ

| Tiêu chí | Máy chủ | Chơi với bạn bè — Sắp có |
| --- | --- | --- |
| Ai chạy máy chủ? | Chủ cộng đồng chạy trên PC hoặc VPS. | Máy của người tạo phòng. |
| Tài khoản | Tài khoản riêng cho từng máy chủ. | Không cần; chỉ đặt tên. |
| Tiến trình | Lưu trên máy chủ, theo cấu hình. | Không chuyển sang máy chủ cộng đồng. |
| Nội dung | Theo cấu hình và tính năng máy chủ hỗ trợ. | Mở khoá toàn bộ vật phẩm được hỗ trợ. |
| Kết nối | Máy có sẵn trong gói; ở S1 dùng mã mời LAN/Internet, không cần tự mở cổng khi kết nối P2P. | Mã phòng qua LAN hoặc Internet ở O1, không cần tự mở cổng. |
| Thời gian | Có thể chạy lâu dài. | Phòng đóng khi người tạo thoát. |
| Số người | Theo giới hạn máy chủ và phòng. | Nhóm nhỏ; giới hạn O1 sẽ công bố khi mở. |

<!-- choose -->
## Nên chọn chế độ nào?

- Muốn chơi nhanh với vài người: chọn **Chơi với bạn bè** khi O1 mở.
- Nhóm chơi thường xuyên, muốn giữ đồ hoặc tiến trình: chọn **Máy chủ**.
- Muốn phục vụ một cộng đồng lâu dài: cân nhắc VPS hay máy chạy 24/7. Quy trình tự dựng gói người chơi ở S1 vẫn sắp có.

<!-- servers -->
## Bắt đầu với Máy chủ

1. Mở launcher, chọn **Máy chủ**, rồi chọn máy có sẵn trong gói. Chờ trạng thái máy hoạt động.
2. Chưa có tài khoản? Bấm **Tạo tài khoản** và giữ mã khôi phục ở nơi an toàn. Tài khoản chỉ dùng cho máy đã chọn.
3. Bấm **CHƠI**, rồi đăng nhập tại màn hình trong game.
4. Muốn lấy bản mới? Nếu gói có liên kết, bấm **Tải bản mới** để mở thư mục Google Drive và tải gói đầy đủ.

**Sắp có — S1:** Dán mã mời trong mục **Thêm máy chủ** để lưu máy. S1 sẽ có hướng dẫn tự dựng máy chủ trên Windows và Linux.

<!-- friends -->
## Bắt đầu chơi với bạn bè

**Sắp có — O1:** Một người chọn tạo phòng rồi gửi mã. Bạn bè sẽ nhập mã để vào qua LAN hoặc Internet; không cần tài khoản hay tự mở cổng. Phòng đóng khi người tạo thoát.

<!-- connection -->
## Không kết nối được

- Chọn đúng máy trong gói, kiểm tra mạng và bấm **Thử lại**. Nếu vẫn lỗi, hỏi người quản lý máy xem dịch vụ có đang chạy không.
- Mở game qua launcher để đường kết nối được khởi động đúng.

<!-- version -->
## Khác phiên bản hoặc lỗi tệp

- Hỏi chủ máy đang dùng bản nào. Nếu có nút **Tải bản mới**, mở link và tải gói đầy đủ đúng bản. Launcher không tự cập nhật.
- Nếu launcher báo lỗi tệp, mở **Cài đặt → Kiểm tra & sửa tệp** rồi thử lại.

<!-- firewall -->
## Tường lửa Windows

- Nếu Windows chặn kết nối, kiểm tra mục **Cho phép ứng dụng qua tường lửa**. Chỉ cho phép launcher và các chương trình từ gói mà bạn tin cậy, trên loại mạng phù hợp.
- Không tắt toàn bộ tường lửa. Hỏi người quản lý máy nếu vẫn không chơi được.

<!-- Standalone guide appendix; the launcher embeds the sections above. -->

Tự dựng máy chủ (S1)
--------------------------

Máy chủ S1 lưu tài khoản và kết quả trên máy bạn. Máy bạn cần chạy trong lúc mọi người chơi. Hãy chuẩn bị thư mục game KR 00967 có các cặp tệp `FxMap`, `FxContents` và `fx` (mỗi cặp gồm `.idx` và `.idd`).

**Windows**

1. Giải nén cả gói server Windows vào một thư mục. Mở **BoomServer.exe**.
2. Nhập tên máy chủ. Nếu ứng dụng đã tìm đúng thư mục game thì giữ nguyên; nếu chưa, bấm **Chọn…** và trỏ tới thư mục đã chuẩn bị. Bấm **Chạy** và chờ trạng thái **Đang chạy**.
3. Bấm **Sao chép** cạnh **Mã mời S1** rồi gửi mã cho người chơi. Người chơi sẽ dùng **Máy chủ → Thêm máy chủ** khi bản launcher hỗ trợ S1.
4. Đóng cửa sổ để máy chủ chạy nền. Bấm biểu tượng BoomServer ở khay hệ thống hoặc mở lại `BoomServer.exe` để xem trạng thái. Bấm **Dừng** trước khi tắt máy. **Sao lưu ngay** tạo bản sao cơ sở dữ liệu; **Mở thư mục dữ liệu** mở nơi chứa bản sao và thông tin tài khoản chủ máy. **Tạo lại mã mời** khởi động lại máy chủ, làm mã cũ hết hiệu lực và ngắt người đang chơi.

Khi Windows hỏi quyền cho kết nối trực tiếp, bạn có thể cho phép trên mạng **Riêng tư**. Nếu từ chối hoặc router không mở được cổng, BoomServer vẫn thử kết nối qua relay. Nếu người chơi chưa vào được ngay, chờ dòng trạng thái báo kết nối relay rồi thử lại.

![Cửa sổ BoomServer khi hai người đang chơi; mã mời đã được rút gọn](images/boomserver-s1.png)

**Linux**

1. Giải nén gói server Linux. Với Docker Compose, mở terminal trong thư mục vừa giải nén, dùng đường dẫn tuyệt đối tới thư mục game mà container có thể đọc, rồi chạy:

   ```sh
   export BOOM_GAME_DIR="/duong/dan/toi/game"
   export BOOM_NAME="Boom Club"
   docker compose -f deploy/compose.yaml up -d --build
   docker compose -f deploy/compose.yaml exec boom /opt/boom/bin/boom-server invite
   ```

2. Nếu dùng Linux có systemd và không muốn dùng Docker, chạy `sudo ./deploy/install.sh /duong/dan/toi/game "Boom Club"`. Khi dịch vụ đã chạy, lấy mã bằng `sudo -u boom-server boom-server invite`.
3. Gửi mã mời S1 cho người chơi. Giữ volume dữ liệu Docker hoặc thư mục `/var/lib/boom-server` nếu cài bằng systemd; đó là nơi lưu danh tính máy chủ, tài khoản và kết quả.

Xem thêm mã nguồn và tài liệu vận hành ở [repo GitHub](https://github.com/aquaplatinumberlitz/boom-kr-server).
