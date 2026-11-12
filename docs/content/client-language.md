# Ngôn ngữ và bảng mã chữ trong client KR 00967

Kết quả so tĩnh hai file exe ngày 13/11/2026, làm cho [đợt L của lộ trình](../PLAN.md#full-game-roadmap). Mức bằng
chứng: `kr_static`. Chưa có lượt chạy nào; mọi câu về hành vi lúc chạy ở mục cuối là việc phải đo.

## Nguồn

| File | SHA-256 | Ghi chú |
|---|---|---|
| Image đã unpack của client KR 00967 (`kr-full.exe`) | `cf72da112308d9f1ca256df8089594c5f787875310933073bf6bd5ba6f70e63b` | Địa chỉ trong tài liệu này theo database IDA đi kèm image đó |
| Image đã unpack của client VN (`unpacked_ca.exe`, repo `boom-re`) | `13a0a33cccc8f5f5124cdd8dac863b68118e6dda83340649b897fc41faed364a` | Build năm 2015; dùng làm mẫu đã chạy được tiếng Việt |
| `CxUni.fft` của client KR | `d2d79cacb15f00b2f544daebff740caf3248c2c393994ace0c310b69f3a01f7a` | Font bitmap trong thư mục game |
| `cxuni.fft` của client VN | `7c419a8f3bf9ee228ee7d11d0481f5ca1ce475a7c2dda1b7aa48f0de5be393b8` | |

Phân tích chạy trên bản copy của exe và database; file gốc không bị sửa.

## Kết luận

1. **Engine tự giữ bảng mã, không dùng bảng mã hệ thống để vẽ chữ.** Cả hai exe có một biến bảng mã toàn cục. Mọi
   chuỗi trước khi vẽ được đổi sang Unicode theo biến đó, rồi tra hình chữ theo mã Unicode trong font bitmap.
2. **Client KR 00967 đã có sẵn đường xử lý tiếng Việt.** Hàm đổi chuỗi của nó có nhánh riêng cho bảng mã 1258: tra
   bảng byte sang Unicode rồi ghép chữ gốc với dấu thanh thành một ký tự dựng sẵn. Cấu trúc này trùng với client VN.
3. **Client KR 00967 là build đa vùng.** Một hàm chọn bảng mã theo vùng: Hàn 949, Nhật 932, Trung giản thể 936, Trung
   phồn thể 950, Việt 1258, Nga 1251, còn lại 1252. Một hàm khác trả mã ngôn ngữ dùng cho bảng chuỗi: 1042 Hàn,
   2052, 1028, 1041, **1033 Anh**, **1066 Việt**, 1049. Ba ngôn ngữ Hàn, Việt, Anh đều là chế độ có sẵn của engine.
4. **Vùng bị khóa cứng là Hàn lúc build.** Các hàm "có phải vùng X không" trả hằng số: vùng Hàn luôn đúng, các vùng
   khác luôn sai. Các hàm này không đọc cấu hình hay tham số nào.
5. **Font của client KR đã có đủ chữ Việt.** Bốn file `CxUni*.fft` chứa 65.535 hình chữ đánh chỉ số theo Unicode;
   146/146 chữ cái tiếng Việt dựng sẵn (thường và hoa) có hình, cùng đủ 11.172 chữ Hàn. Ô chữ của KR là 12×12 điểm,
   của VN là 12×16, nên chữ hoa có hai tầng dấu trên font KR sẽ chật.

Hệ quả: tiếng Việt trên client 00967 **không cần viết thêm bộ vẽ chữ hay font**. Cần một bản vá nhỏ trên bản copy
của exe để đổi bảng mã và mã ngôn ngữ, cộng dữ liệu chữ đã dịch. Tiếng Anh không cần vá exe: chữ ASCII hiển thị đúng
ngay ở bảng mã 949.

## Địa chỉ đã đọc

| Vai trò | Client KR 00967 | Client VN |
|---|---|---|
| Biến bảng mã của engine | `0x1A7E420`, giá trị đầu 949 | `0xEEE66C` |
| Hàm đặt bảng mã | `0x149AA90` | `0xA21D50`, được gọi với hằng 1258 tại `0x98A29B` |
| Chọn bảng mã theo vùng | `0x138D380`, đọc vùng từ `0x455A30` (trả hằng 1, 49 nơi gọi) | Không có; đặt cứng 1258 |
| Đổi chuỗi byte sang Unicode | `0x149A740`; nhánh 1258 gọi `0x149A7B0` | `0xA22AC0`; nhánh 1258 gọi `0xA22360` |
| Đổi Unicode sang chuỗi byte | `0x149A6D0`; nhánh 1258 gọi `0x149AC90` | `0xA22B20` |
| Bảng mã có ký tự hai byte không | `0x149A420`, `0x149A480` (đúng với 932, 936, 949, 950) | — |
| Mã ngôn ngữ cho bảng chuỗi | `0x13A5770`, một nơi gọi tại `0x13A52D0` | — |
| Hàm "vùng Hàn" / "vùng Việt" / "vùng Anh-Mỹ" | `0x48D8D0` (170 nơi gọi) / `0x48D900` (201) / `0x905740` (27) | — |
| Nạp font `.fft` | chưa định vị | `0xA33320`, bảng tên file tại `0xE5CA2C` |
| Đo và vẽ chuỗi | chưa định vị | `0xA34530`, `0xA34650`, `0xA335E0` |

Định dạng `.fft` đọc từ hàm nạp của client VN và khớp kích thước cả tám file: header 32 byte (số hình chữ ở offset 8,
rộng ở 12, cao ở 16, số byte bitmap mỗi hình ở 28); tiếp theo là `số hình × (1 byte bề rộng + bitmap)` cho nét thường
và một khối thứ hai cùng số hình cho nét viền với ô rộng hơn 2 điểm mỗi chiều.

## Chưa biết, phải đo khi mở đợt L

- **Vá ở mức nào.** Chỉ đổi bảng mã và mã ngôn ngữ là hẹp nhất. Đổi cả vùng sang Việt sẽ bật 201 nhánh riêng của vùng
  Việt, có thể đổi luồng đăng nhập, giao diện và packet đã đo; không làm nếu chưa có lý do.
- **Client có lọc bảng chuỗi theo mã ngôn ngữ không.** Mỗi mục trong `ClientString` của pack mang mã 1042; nếu client
  lọc thì bảng tiếng Việt phải mang 1066.
- **134 chuỗi nằm trong exe** vẫn là byte tiếng Hàn và sẽ hiện sai ở bảng mã 1258.
- **Chữ do người chơi gõ.** Tên, tiêu đề phòng và chat đi qua server dưới dạng byte theo bảng mã của người gửi. Hai
  người dùng hai ngôn ngữ sẽ thấy chữ của nhau sai, trừ khi server chuyển mã theo từng người nhận hoặc giới hạn ký tự.
- **Gõ tiếng Việt trong client** (ô chat, tên phòng) chưa xem đường nhập.
- **Dùng font của client VN** (ô 12×16) trong client KR để chữ có dấu dễ đọc hơn: hàm nạp đọc kích thước từ header nên
  có thể được, nhưng bố cục giao diện có thể giả định dòng cao 12 điểm.
- **Kiểm toàn vẹn** của client với exe và file chữ đã thay.
