# Ngôn ngữ và bảng mã chữ trong client KR 00967

Khảo sát cho [đợt L của lộ trình](../PLAN.md#full-game-roadmap) bắt đầu bằng so tĩnh hai exe ngày 13/11/2026;
các kết luận đó mang nguồn `kr_static` hoặc nguồn tương ứng được ghi tại chỗ. Những đối chứng native về sau dùng
client diagnostic bản copy và được ghi giới hạn riêng ở mục UTF-8. Bản vá 949x đã hiện chữ ba ngôn ngữ trên client
chẩn đoán; giới hạn ở [mục UTF-8](#utf8).

## Nguồn

| File | SHA-256 | Ghi chú |
|---|---|---|
| Image đã unpack của client KR 00967 (`kr-full.exe`) | `cf72da112308d9f1ca256df8089594c5f787875310933073bf6bd5ba6f70e63b` | Địa chỉ trong tài liệu này theo database IDA đi kèm image đó |
| Image đã unpack của client VN (`unpacked_ca.exe`, repo `boom-re`) | `13a0a33cccc8f5f5124cdd8dac863b68118e6dda83340649b897fc41faed364a` | Build năm 2015; dùng làm mẫu đã chạy được tiếng Việt |
| `CxUni.fft` của client KR | `d2d79cacb15f00b2f544daebff740caf3248c2c393994ace0c310b69f3a01f7a` | Font bitmap trong thư mục game |
| `cxuni.fft` của client VN | `7c419a8f3bf9ee228ee7d11d0481f5ca1ce475a7c2dda1b7aa48f0de5be393b8` | |

Phân tích chạy trên bản copy của exe và database; file gốc không bị sửa.

## Kết luận

1. **Engine tự giữ bảng mã.** Cả hai exe có một biến bảng mã toàn cục; các đường chuyển mã và trình bày được đọc có
   bước đổi sang Unicode. Chưa chứng minh mọi đường chữ đều đi qua biến này hoặc cùng một font.
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

Font có sẵn là ứng viên hiển thị tiếng Việt; độ dễ đọc, cách đo và mọi đường vẽ còn phải kiểm trên client. Chủ dự án
chọn [đưa chữ về UTF-8](#utf8) để người chơi khác ngôn ngữ đọc được chữ của nhau.

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
| Đo, cắt và chuyển tiếp chuỗi để trình bày | `0x14BAC80`, `0x14BADF0`, `0x14BA910`, `0x14BB5E0`, `0x14BB7B0`; đích vtable chưa định danh | `0xA34530`, `0xA34650`, `0xA335E0` |

Định dạng `.fft` đọc từ hàm nạp của client VN và khớp kích thước cả tám file: header 32 byte (số hình chữ ở offset 8,
rộng ở 12, cao ở 16, số byte bitmap mỗi hình ở 28); tiếp theo là `số hình × (1 byte bề rộng + bitmap)` cho nét thường
và một khối thứ hai cùng số hình cho nét viền với ô rộng hơn 2 điểm mỗi chiều.

<a id="poptag"></a>

## Client PopTag 0.7.8 (bản tiếng Anh, nhận ngày 13/11/2026)

Công ty cấp thêm bộ cài `PopTagSetup_v0.7.8.exe` (SHA-256
`7a339e7b76cc51d00140a68a729dc0836119b2cbc6b9f7f0a1123e290ba65c01`). Bộ cài được bung ra trong một Wine prefix riêng;
client chưa được chạy lần nào. Mọi điều dưới đây đọc từ file, mức `kr_static`.

**Đây là gì.** `CA.exe` bên trong (SHA-256 `d1f9f9b080abe2557693e6c906c26e22674be5fca6abe13d8913fe1a7c1db5d7`) là build
ngày 15/10/2010 của bản Bắc Mỹ, không bị đóng gói, bảng import đầy đủ. Một nhóm bên ngoài đã sửa nó: thêm mười hai
section mã và dữ liệu, và bắt exe nạp `PopTagDLL.dll` (build 09/11/2026). Gói còn có trình cập nhật tự tải file từ
một máy chủ bên ngoài. Vì vậy client này khác build với 00967, chưa so protocol, và **không được chạy khi còn đường
ra mạng**.

**Ba điều dùng được cho đợt L:**

1. **Cùng engine đa vùng.** Build 2010 có cùng hàm chọn bảng mã theo vùng (quanh `0x6BEFAF`) và cùng hàm mã ngôn ngữ
   1042/1033/1066 (quanh `0x4D7520`). `ClientString` trong pack của nó mang mã 1033. Đây là một exe không đóng gói của
   cùng engine, đọc dễ hơn image unpack của 00967.
2. **Một mẫu xử lý chữ nhiều ngôn ngữ.** Section `.u8code` bổ sung có đường dùng bảng mã 65001; section `.vkinp`
   có đường gọi `PeekMessageW`/`PostMessageW`. Gói kèm bốn bộ font chọn bằng một file `.bat`: Anh-Hàn, Đài Loan,
   Việt, gốc. Khảo sát tĩnh ban đầu cho thấy còn một lớp mã hai byte và đánh dấu riêng; chưa thể kết luận engine dùng
   UTF-8 thuần hoặc mọi đường chữ đã được bao phủ. Chưa nghiệm thu hành vi chạy của mẫu này.
3. **Font 12×16 nạp được ở build không phải VN.** Bộ font Việt trong gói có ô 12×16; `CxUni.fft` và `CxUniBold.fft`
   của nó trùng từng byte với font client VN. Ba bộ còn lại ô 12×12 như 00967.

**Bản dịch tiếng Anh.** Phiên Codex Terra đã trích chuỗi và ghép theo ID với catalog Hàn–Việt. Kết quả nằm ở checkout
`boom-re-l10n` (repo `boom-re`, nhánh `claude/l10n-kr-vi`), thư mục `l10n/`, chưa commit lúc viết: `catalog_en.csv`,
`english_sources.csv`, `images_en_candidates.csv`, `stock_names_en.csv`. Mọi dòng là ứng viên, chưa duyệt.

| Nguồn chuỗi KR | Số chuỗi | Có tiếng Anh |
|---|---|---|
| `ClientString` trong pack (giao diện) | 7.002 | 5.577 (80%) |
| `csysstr.ssd` (chuỗi server gửi), phần không trùng `ClientString` | 6.803 | 392 (6%) |
| Tên map (`fxdynmapinfo.ssd`) | 773 | 215 (28%) |
| `csysqstr.ssd` (nhiệm vụ) | 7.845 | 0 |
| Danh sách ô của map tự tạo | 2.174 | 0 |
| Chuỗi trong exe và các nguồn nhỏ khác | 724 | 0 |
| **Tổng** | **25.321** | **6.184 (24%)** |

Trong 6.184 dòng có 102 dòng nhiều biến thể phải chọn tay và 34 dòng lệch tham số định dạng. Ngoài bảng trên có thêm
199 dòng ghép được khi so ID không phân biệt hoa thường, mang trạng thái riêng `casefold_candidate`. Phần thiếu còn
18.938 dòng, phải dịch mới.

Các số trên là lúc trích ngày 13/11; số hiện hành sau khi soát nghĩa và dịch bù ở [mục dữ liệu](#data-files).

**Rà độ phủ ngày 13/11** (Claude đọc lại toàn bộ 87 pack và các file lẻ của client, độc lập với công cụ trích; Terra làm
tiếp lượt hai theo kết quả rà):

- Các nguồn đã trích (cache chuỗi, `ClientString`, tên map, 610 tên vật phẩm, 5 tên kênh, 61 tên hiệu ứng) được lấy đủ.
- **Tên vật phẩm không ghép được theo ID.** Bốn file `fxstock0.ini`–`fxstock3.ini` của KR có 8.165 record kèm tên.
  Trong 610 tên của PopTag chỉ 113 ID trùng số với KR, và cả 113 đều khác loại lẫn icon: hai bản dùng hai dải ID
  khác nhau. Chưa thử ghép theo đường dẫn icon.
- Chưa trích: chữ giao diện viết cứng trong `PopTagDLL.dll` (cỡ vài trăm câu, thuộc tính năng riêng của PopTag, không
  có ID).
- Không có trong client: chữ nhiệm vụ (script chỉ gọi ID, chữ do server gửi), chữ tiếng Anh cho chuỗi trong exe (exe
  của PopTag vẫn giữ tiếng Hàn), `Help` và mô tả ô map tự tạo (vẫn là tiếng Hàn).
- **Ảnh có chữ:** 1.187 trong 3.038 ảnh nghi có chữ của KR có entry cùng tên trong pack PopTag; 611 giống nội dung giải
  mã, 576 khác. Danh sách 576 ảnh là ứng viên để mở xem. Kết quả đọc chữ trên từng ảnh ở [mục dữ liệu](#data-files).

**Giới hạn.** Ghép theo ID không chứng minh hai bản cùng nghĩa: cùng ID có thể đổi nội dung sau mười sáu năm. Theo
[thông báo ngày 13/11](../evidence/owner-directives/l10n-text-rights.json), công ty có thỏa thuận dùng toàn quyền chữ của
bản Bắc Mỹ, bản KR và bản VN. Chưa tách được chữ nào của bản Bắc Mỹ gốc, chữ nào do nhóm sửa client PopTag tự viết; phần
tự viết đó chưa được thông báo nói tới.

<a id="utf8"></a>

## Hướng đã chốt: mọi chữ về UTF-8

Theo [quyết định 13/11](../evidence/owner-directives/l10n-utf8.json), dữ liệu chữ của cả ba ngôn ngữ phát hành dưới dạng
UTF-8 và một bản vá chung cho phép Hàn, Việt, Anh hiện cùng lúc. Khác nhau giữa ba ngôn ngữ chỉ còn là dữ liệu: bảng
chuỗi, ảnh có chữ và font. Trình mở game chọn bộ dữ liệu và áp bản vá; đổi ngôn ngữ có hiệu lực ở lần mở game sau.

### Thiết kế đã chọn: mã nội bộ 949x (15/11)

Client giữ bảng mã 949 bên trong, như cách client PopTag làm, và thêm một **bảng mở rộng tĩnh** cho ký tự ngoài 949.
Ký tự có trong 949 giữ nguyên byte 949. Ký tự ngoài 949 thuộc các dải Latin mở rộng, dấu kết hợp, Hy Lạp, Kirin,
Latin Extended Additional (chữ Việt dựng sẵn), dấu câu, ký hiệu tiền tệ (có `₫`) và vài dải ký hiệu nhận một mã hai byte
cố định ở vùng mà 949/UHC không dùng: byte đầu `0xC7..0xFE`, byte sau `0x41..0x5A`, `0x61..0x7A`, `0x81..0xA0`. Bản 1
có 2.275 ký tự; mã thứ `k` ứng với ký tự thứ `k` theo thứ tự dải trong [`tools/l10n_949x.py`](../../tools/l10n_949x.py),
dải mới chỉ nối vào cuối nên mã cũ không đổi. Wine từ chối mọi mã này khi giải 949 gốc (kiểm cả 2.275 mã), nên không
mã nào trùng ký tự 949 có sẵn.

Lý do chọn, so với UTF-8 thuần trong client:

- **Không phải đụng literal và consumer 949.** Exe có ít nhất 134 chuỗi 949 và nhiều lời gọi chuyển mã cứng 949. Mọi
  lời gọi `MultiByteToWideChar`/`WideCharToMultiByte` trong exe đi qua hai slot IAT (38 và 30 chỗ gọi `call [IAT]`),
  nên chỉ cần thay hai slot đó. Literal, phép cắt chuỗi theo `IsDBCSLeadByteEx`/`CharNextExA` và mọi đường chuyển mã
  giữ nguyên, vì mã mở rộng vẫn là cặp byte đầu/byte sau hợp lệ của 949.
- **Độ dài byte không tăng.** Chữ Việt có dấu chiếm hai byte như chữ Hàn; chữ Hàn và ASCII giữ nguyên byte. Giới hạn byte
  của tên, tiêu đề phòng và chat trong client và packet vẫn đúng nghĩa; UTF-8 sẽ làm chữ Hàn dài gấp rưỡi.
- **Bảng tĩnh thay cho bảng động của PopTag.** PopTag cấp mã hai byte động theo phiên nên phải bọc chuỗi `J9:` mang UTF-8
  qua mạng. Với bảng tĩnh, mọi client vá dùng cùng mã, packet không đổi định dạng, server không cần biết bảng để chuyển
  tiếp. Tiếng Hàn và ASCII trên dây giống hệt client chưa vá; client chưa vá chỉ thấy sai các ký tự mở rộng.
- **Dữ liệu vẫn là UTF-8.** Bản dựng phát hành là UTF-8 (`boom-re-l10n`, `tools/l10n_build.py --encoding utf-8`); trình mở
  game đổi sang 949x khi chuẩn bị thư mục game (`tools/l10n_949x.py convert`). Đổi bộ Hàn UTF-8 sang 949x cho ra đúng
  từng byte dữ liệu giải nén của bảy file `ssi1` gốc. Từ bản dựng ghi `csysfxmg.ssd` (dòng chạy chữ và mẹo chat,
  `u16 số dòng` rồi từng `u8 loại` + chuỗi), `convert` đổi thêm file đó nếu bộ nguồn có; bộ nguồn cũ không có thì bỏ qua. Video mở đầu `nm.bik` (logo và màn GRB chữ Hàn dựng sẵn trong video) được bản dựng vi/en thay bằng video của client VN/Bắc Mỹ (không chữ); `convert` chép nguyên, bản ko giữ video gốc.

Server muốn lưu chữ người chơi dưới dạng UTF-8 thì đổi 949x sang UTF-8 bằng cùng bảng ở biên lưu trữ; việc này chưa làm.

### Bản vá

| Điểm | Cách vá | Nguồn |
|---|---|---|
| Section `.l10n` mới (VA `0x2266000`) và entry point mới trên bản sao `CA-r2-stage-social-cmp.exe` | Entry mới thay năm slot IAT rồi nhảy về OEP `0x1470E40`; địa chỉ chính lấy từ PE nguồn có hash cố định | [`tools/l10n_949x_patch.py`](../../tools/l10n_949x_patch.py), [`tools/l10n_949x_hook.c`](../../tools/l10n_949x_hook.c) |
| IAT `MultiByteToWideChar` `0x16EA4B4` | Với codepage 949: giải thêm mã 949x, các đoạn khác chuyển nguyên cho API gốc; codepage khác đi thẳng API gốc | như trên |
| IAT `WideCharToMultiByte` `0x16EA4AC` | Với codepage 949: ghi mã 949x cho ký tự trong bảng | như trên |
| IAT `PeekMessageA` `0x16EA638` | Lấy message bằng `PeekMessageW`; `WM_CHAR` ngoài ASCII thành `WM_IME_COMPOSITION` với `GCS_RESULTSTR` | như trên |
| IAT `ImmGetCompositionStringA` `0x16EA0D0`, `ImmGetCompositionStringW` `0x16EA0E8` | Khi có ký tự tiêm: trả ký tự đó (A: mã 949x, W: UTF-16); còn lại đi API gốc | như trên |
| Font `CxUniGulim.fft`, `CxUniGulimBold.fft` (bản sao) | Ô nhập liệu dùng font Gulim, vốn có hình hộp cho chữ Việt ngoài Latin-1; chép 1.709 hình từ `CxUni.fft`/`CxUniBold.fft` | `tools/l10n_949x.py fonts` |

Đường nhập liệu đã đọc tĩnh: message bàn phím tới một trong hai hệ ô nhập, `0x15222C8` hoặc `0x1514A29`. Hệ đầu ghép cặp
DBCS đến bằng `WM_CHAR` (`0x152015D` qua `0x149A740`); hệ sau cố ý bỏ cặp đó. Cả hai nhận chữ ngoài ASCII qua composition
IME: `0x152248F` đọc `ImmGetCompositionStringW`, `0x1514E11` đọc `ImmGetCompositionStringA`. Vì vậy hook đổi mọi
`WM_CHAR` ngoài ASCII thành một kết quả composition. Composition thật của IME đi nguyên vẹn.
Gõ tiếng Việt kiểu Unikey (`SendInput` với `KEYEVENTF_UNICODE`) được mô phỏng bằng
[`tools/l10n_unicode_input.c`](../../tools/l10n_unicode_input.c).

### Kết quả native

Ba lượt hai client trên exe runtime `CA-r2-stage-social-cmp.exe` đã vá, dịch vụ Rust chẩn đoán và LAN ảo cô lập theo
recipe G1 Factory r3; dữ liệu là bản dựng UTF-8 của `boom-re-l10n@11bc77c` đổi sang 949x lúc chuẩn bị thư mục game.
Ảnh chụp và raw nằm dưới `local/runs/` của worktree; ảnh đã được xem trực tiếp.

| Lượt | Client A / B | Thấy được | Evidence |
|---|---|---|---|
| mix-02 | Việt (chưa có ảnh) / Hàn | Sảnh và hộp tạo phòng của A hiện chữ Việt đúng dấu. A tạo phòng với tên ngẫu nhiên tiếng Việt `Phải chơi Lịch sự`; B giao diện Hàn hiện đúng tên đó | [mix-02](../evidence/l10n-utf8-949x-mix-02.json) |
| vi-04 | Hàn / Việt có 459 ảnh | B: sảnh, hộp tạo phòng, phòng bằng tiếng Việt cùng nút ảnh đã dịch, tên phòng Hàn của A. A (giao diện Hàn) gõ chat `Xin chào các bạn 안녕하…`, server nhận đúng byte 949x | [vi-04](../evidence/l10n-utf8-949x-vi-04.json) |
| en-05 | Hàn / Anh có 314 ảnh | B: sảnh, phòng bằng tiếng Anh cùng nút ảnh, tên phòng Hàn. A gõ `Hello xin chào 반가워요 ₫ ơư`, ô chat hiện đủ, server nhận đủ 32 byte 949x | [en-05](../evidence/l10n-utf8-949x-en-05.json) |

Bộ Hàn UTF-8 qua 949x hiện như dữ liệu gốc ở client A của vi-04 và en-05. Đọc tĩnh và fixture Wine của bản vá ở
[evidence tĩnh 949x](../evidence/l10n-utf8-949x-static.json).

### Bổ sung sau CP310 (16/11)

**Font: bộ VN cho nhãn, Gulim 12×12 gọn cho ô nhập.** Bộ font VN của PopTag (`CxUni`/`CxUniBold` 12×16) giữ đủ dấu chồng
của chữ hoa (`Ệ`, `Ố`, `Ầ`), còn hình mượn từ `CxUni` KR 12×12 thì mất dấu mũ của `Ệ` và dính dấu ở `ế`. Engine 00967 đọc
khổ cao từ header nên nạp được bộ này cho nhãn. Ô nhập và ô chọn người nhận của layout KR chỉ cao 12 điểm nên cắt hàng
dưới của Gulim VN 15/16 hàng (mất dấu nặng, đuôi `g`); ảnh client VN đã lưu không thấy cắt, chưa rõ vì sao (không phải do khổ 1024×768; xem [phóng to cửa sổ](#window-scale)). Vì
vậy `tools/l10n_949x.py vnfonts` dựng `CxUniGulim`/`CxUniGulimBold` 12×12: chữ Hàn/CJK lấy từ Gulim KR, chữ Latin/Việt lấy
từ Gulim VN, giữ 10 hàng thân và chân chữ, gộp vùng dấu phía trên vào 2 hàng, viền sinh lại bằng giãn 8 hướng (đúng cách
file font KR lưu viền). Chữ Hàn ở `CxUni` VN lấy advance của font KR. Cách mượn hình vào Gulim KR (`fonts`) giữ làm dự phòng.

**Chữ thấp trong ô nhỏ (17/11).** Loader font của 00967 (`0x14BAEC0`) chỉ đọc số ký tự, khổ và số byte mỗi hình; hai
trường header ở offset 20/24 không dùng. Hàm vẽ (`0x14BB240`) đặt đỉnh ô chữ ở đúng y bên gọi truyền vào, không căn theo
chiều cao font, và không có hàm lấy chiều cao font: y là hằng của layout KR, tính cho ô 12 hàng mà chân chữ Latin ở hàng 9.
`CxUni`/`CxUniBold` VN chừa 4 hàng trên cho dấu chồng nên chân chữ Latin ở hàng 12: mọi chữ Việt vẽ bằng hai font này thấp
hơn chữ KR cùng chỗ 3 điểm, thấy rõ ở ô thấp (tab chat sảnh/phòng: đuôi `g` của `chung` chạm viền dưới; khung map phòng:
`Factory07`, `Số chỗ: 8` sát đáy). Client VN dùng cùng engine nhưng layout riêng với ô cao hơn. Sửa ở bộ font:
`vnfonts` nâng mọi hình của `CxUni`/`CxUniBold` lên 2 hàng (`RAISE_ROWS`); đỉnh ô giữ nguyên nên không cắt thêm ở mép trên.
Hình có ít hơn 2 hàng trống phía trên: chữ Latin có dấu (tách được theo NFD) giữ thân dời 2 hàng và xếp các hàng dấu vào
chỗ còn lại (bỏ hàng trống, thiếu chỗ thì gộp hai hàng kề nhau); chữ khác (Hàn/CJK có đỉnh ở hàng 1) dời bằng số hàng trống.
Đổi lại: chữ hoa có dấu chồng mất khoảng trống giữa dấu và thân, và vài chữ có ba hàng dấu như `Ẩ`, `Ẫ`, `Ẳ`, `Ổ`, `Ỗ` bị gộp
dấu hỏi/ngã với dấu mũ vào hai hàng. Gulim 12×12 gọn (ô nhập) không đổi; client chỉ dùng font KR gốc không bị ảnh hưởng
([evidence](../evidence/l10n-utf8-949x-text-align.json)).

**Tab chat và dòng chạy chữ (17/11, lượt sau).** Nâng font 2 hàng chưa đủ cho hai chỗ; cả hai sửa bằng vá layout trên bản
sao exe `--lang vi` (`VI_LAYOUT`, `VI_LAYOUT_CALLS` trong [`tools/l10n_949x_patch.py`](../../tools/l10n_949x_patch.py):
17 byte và 2 lời gọi chuyển sang stub trong `.l10n`, kiểm byte cũ trước khi ghi), font không đổi. Bản `ko`/`en` không đổi.

- Tab lọc chat sảnh (`Thông báo chung`, `Tất cả`, `Hội`): `sub_D357E0` đặt y chữ = đỉnh tab + 7 (`0xD35AD5`,
  `0xD35C06`, `0xD35CE8`). Lòng tab chưa chọn cao 17 hàng; với `CxUniBold` đã nâng, đuôi `g` và dấu nặng xuống tới hàng
  cuối và viền đen của chữ đè lên hàng viền tab. Đổi `+7` thành `+5`: mực chữ nằm ở hàng 2–15 của lòng tab, sảnh còn 2 hàng
  trống trên viền dưới như chữ KR, dấu `ấ` của tab chưa chọn không chạm viền trên.
- Tab chat phòng (`Tất cả`, `Hội`) do `sub_D364D0` vẽ, y chữ = đỉnh khung lấy bằng `call sub_149BEF0` (`0xD365C8`,
  `0xD36898`), không có hằng để sửa tại chỗ; dấu nặng của `Hội` chạm viền dưới. Hai lời gọi chuyển sang stub
  `call sub_149BEF0; sub eax, 2; ret` đặt sau chuỗi dịch trong section `.l10n`.
- Dòng chạy chữ ở đầu sảnh/phòng (dòng loại 2 của `csysfxmg.ssd`): `sub_D39490` đặt khung clip từ chữ nhật của từng màn
  (`sub_D39440`, 13 chỗ gọi, mọi chỗ cao `0x0C`) rồi gọi DrawText kiểu 2 (chữ có viền), họ font 1 (`CxUniGulim`), y = 1 trong
  khung. Họ font là tham số thứ tám (`0xD395B6`); kiểu 0..7 tra bảng ở `0x19A927C` (`sub_14BB0F0`). Gulim 12×12 gọn
  chỉ có 2 hàng cho dấu nên dấu chồng (`ể`, `ổ`, `ấ`) bị gộp trông như bị cắt, và khung cao 12 cắt hàng đuôi chữ cuối và viền
  dưới. Vá: họ font 0 (`CxUni` VN 12×16, đủ dấu chồng) và khung cao 16 (14 hàng mực, viền 1 điểm mỗi phía); khung mới vẫn nằm
  trong dải tối của sảnh và phòng. Exe đặt lại vị trí cuộn khi chữ trôi hết `len*6` điểm; bề rộng `CxUni` của mọi dòng loại 2
  nhỏ hơn số đó. Đổi lại: tên GM tiếng Hàn trong dòng chạy chữ vẽ bằng hình Hàn của bộ `CxUni` VN.

Lượt vi-10 (dữ liệu `boom-re-l10n@fa030b8`) chụp trước/sau ([evidence](../evidence/l10n-utf8-949x-tab-marquee.json)).

**Layout của 00967 là hằng trong exe**, không có file layout trong pack; nhãn và giá trị đặt ở toạ độ cố định. Client VN
cũng vẽ giá trị ở toạ độ cố định (ảnh VN cũ của `boom-re` cho thấy khoá `PREMAX` bị số đè, chữ dài chạy ra ngoài nút loại
game). Vì vậy chữ tràn được sửa bằng câu ngắn hơn trong catalog. Đo bằng font: nhãn số người tối đa khoảng 41 điểm
(`Số chỗ` 39 vừa, `Số  người` 55 đè số), nút loại game khoảng 71 điểm (`Bắt thủ lĩnh` 65 và `Leader Hunt` 69 vừa;
`Bắt đội trưởng` 81 và `Catch the Leader` 97 tràn), nhãn chế độ khoảng 95 điểm (`Even Teams` 70 vừa, `Even Teams Only` 99
bị ô kế che). Các khoá `PREMAX`, `CreateDlgGameTypeFlagHunt`, `CreateDlgModeManner` có ở cả `ClientString` và
`csysstr.ssd` và client dùng bản `csysstr`, nên catalog phải sửa cả hai. Khung chat ngắt dòng theo bề rộng ký tự, không theo
từ; chèn xuống dòng `\n` ở ranh giới từ trong chuỗi `csysstr` (ví dụ `SuperChiefCanStartDetail` tiếng Anh) cho dòng gãy
đúng chỗ. Trước/sau ở `local/review-fix/` ([evidence](../evidence/l10n-utf8-949x-layout-fix.json)).

**Ảnh khác shape.** Dựng với `--allow-shape-change` thay được 549/549 ảnh Việt và 339 ảnh Anh (4 ảnh Anh không có bản Bắc
Mỹ). Client không lỗi; các nút trước đây bị bỏ như `TẠO PHÒNG`, `ĐI TẮT`, `TÙY CHỌN`, `SIÊU THỊ`, `CREATE ROOM`,
`QUICK PLAY` đã hiện. Ảnh còn chữ Hàn trong các màn đã chụp, cần vẽ: `레벨` (vi), bong bóng `칭호 UPDATE` (en), nhãn ô
người chơi `방 장`/`준 비`, tab nhân vật `전체`/`일반`/`럭셔리`/`슈퍼`, `팀선택`, `맵`, `크리에이터 맵`/`오리지널 맵` (vi),
`자동준비`, banner `다오` và nút `시 작` của chủ phòng (en).

**Chuỗi cứng trong exe.** Catalog có 134 dòng `exe:kr-full.exe`; 81 dòng là tên cấm giữ tiếng Hàn. Bảng
[`tools/l10n_949x_exe_literals.json`](../../tools/l10n_949x_exe_literals.json) ghi 40 chuỗi và 45 operand trỏ tới chúng
(xref IDA trên bản sao). `tools/l10n_949x_patch.py --lang vi|en` đặt bản dịch 949x vào section `.l10n` và sửa operand sau
khi kiểm đúng địa chỉ cũ. Các tham chiếu đưa vào `0x409AD0` bị loại vì đó là danh sách tiêu đề cửa sổ của phần mềm tiệm
net mà client so với chương trình khác, dịch sẽ làm hỏng phép so. Các consumer còn lại đã đọc tạo `std::string`, định dạng vào
chuỗi động, cấp phát theo `strlen` hoặc vẽ trực tiếp, nên chuỗi dài hơn không chép vào bộ đệm cố định. Các màn đã chụp không chứa chuỗi nào
trong 40 chuỗi này, nên chưa thấy chúng trên màn hình.

**Gõ tên phòng.** Client chặn sửa tên khi chuỗi định danh của người chơi rỗng: handler đăng nhập `0x10FD540` đọc theo
từng người chơi `u32 id`, tên, một `u8` rồi chuỗi định danh vào `CSelfInfo+0x21C`, và `0x13B2320` báo
`CannotChangeTitleByNoNexonId` khi chuỗi này rỗng. Dịch vụ Rust thêm cấu hình chẩn đoán
`account_admission.diagnostic_identity_strings` (mặc định rỗng, theo tên hiển thị) để điền chuỗi đó cho tài khoản thử;
thứ tự và cách mã hóa trường không đổi. Với tài khoản thử, ô tên phòng nhận gõ; server nhận create81
`Phòng Việt 한국` (17 byte) và `ỆỐẦ Việt 한국 ếộ` (22 byte) dạng 949x, client Hàn hiện đúng tên đó.

**Chat bị cắt: đã sửa.** Ký tự gõ kiểu Unikey đến dưới dạng phím `VK_PACKET` mang mã Unicode ở `HIWORD(lParam)`. Bảng phím
tắt trong phòng (`0x1229450`, gọi từ `0xD5AA60` và các hàm cùng kiểu) đọc phần đó như scan code, nên một số ký tự (ví dụ `하`)
trùng lệnh mở hộp vật phẩm; lệnh này tăng bộ đếm khóa `CInputMan+0x24` (`0xB916D0`) và `0x1514A29` bỏ mọi chữ gõ trong
khoảng 1,5 giây. Bisect trước đó cho thấy lỗi đi theo chuỗi gõ, không theo dữ liệu, font hay exe
([bisect](../evidence/l10n-utf8-949x-chat-bisect.json), [lượt trước](../evidence/l10n-utf8-949x-chat-drop.json)).
Hook giờ tự dịch phím `VK_PACKET` thành `WM_CHAR` và giao `WM_NULL` cho game; chuỗi Việt trước bị cắt ở 26 byte và chuỗi Hàn
trước bị cắt ở 10 byte đều tới server đủ ([evidence](../evidence/l10n-utf8-949x-chat-fix.json)). IME tiếng Hàn thật chưa thử.

**Ô chọn người nhận chat giới hạn 14 byte.** `CChatStage` tạo ô người nhận với độ dài tối đa 14 (struct tạo control ở
`0xD34400`, trường độ dài mà `0x151F795` chép vào `+0x13C`) và đặt chữ `ChatToAll`/`ChatToOurTeam` vào ô đó; chuỗi dài hơn
thì không vào ô nên ô trống. `<To All>` 8 byte, `<모두에게>` 10, `<우리팀에게>` 12 vừa; `<Gởi cho tất cả>` 19 và
`<Gởi cho đồng đội>` 23 byte thì không. Client VN không mang chữ cho hai khoá này (bảng chuỗi giao diện do server cũ gửi); server
của dự án `boom-re` dùng `<Mọi người>` và `<Đồng đội>`, đều 14 byte ở 949x, vừa giới hạn.

**Tên phòng mặc định.** Exe VN có khoá `DefaultRoomTitle0`–`5` nhưng không có chữ: giá trị do server VN gửi và không còn
nguồn. Bản Việt cần tối đa 22 byte ở 949x; hai câu đang vượt cần thay.

Evidence: [ảnh và chuỗi exe](../evidence/l10n-utf8-949x-images-r4.json), [tên phòng, font và chat](../evidence/l10n-utf8-949x-title-font.json).

<a id="window-scale"></a>
### Phóng to cửa sổ không thêm chỗ cho chữ

00967 luôn bố cục và vẽ ở 800×600 (render target cố định, rồi kéo giãn sang backbuffer). Client có sẵn tuỳ chọn
`창모드 배율` (`OptionDlgResolution`: 1배/1.5배/2배 = 800×600, 1200×900, 1600×1200) cùng ô `창모드`; chỉ phóng ảnh, không
đổi bố cục. Vì vậy phóng to không sửa được chữ tràn khung; vẫn phải rút ngắn câu hoặc đổi font. Không có chế độ giao diện
1024×768, không có ảnh nền lớn hơn 800×600 ở cả KR 00967 (49.205 ảnh đã đo) lẫn VN 116 (20.648 ảnh), và exe VN không có
cặp hằng 1024×768 hay tuỳ chọn tương tự. Câu "client VN dùng giao diện 1024×768" trước đây là không có bằng chứng. Chưa kiểm:
nơi lưu giá trị Resolution, toạ độ chuột khi phóng, chưa chạy native. Chi tiết VA ở
[evidence tĩnh phóng to cửa sổ](../evidence/l10n-window-scale-static.json).

### Giới hạn và việc còn lại

- **Tiệm chưa tới được.** Nút tiệm ở phòng chỉ hiện tooltip `Đến Siêu thị`; dịch vụ Rust chưa có tiệm.
- **Chat chưa chuyển tiếp.** Dịch vụ Rust từ chối opcode chat 18 (`lobby_request_denied`) và đóng kết nối sảnh, nên chưa
  thấy chat chéo giữa hai client; chữ trộn đã thấy ở tên phòng và ô nhập.
- **Sửa tên phòng** cần chuỗi định danh của tài khoản; mới có cho tài khoản thử qua cấu hình chẩn đoán (mục trên).
- **Tên phòng tối đa 22 byte.** Hai trong sáu tên phòng mặc định tiếng Việt dài 26 và 27 byte trong 949x
  (`DefaultRoomTitle1`, `DefaultRoomTitle2`); khi client chọn ngẫu nhiên trúng hai tên này, nút xác nhận tắt (lượt vi-01).
  Dữ liệu cần câu ngắn hơn. Nhãn `ChatToAll`/`ChatToOurTeam` tiếng Việt vượt giới hạn 14 byte của ô người nhận (mục trên).
- **Ảnh còn chữ Hàn**: danh sách cần vẽ ở mục bổ sung phía trên.
- **Chữ do server gửi và chữ cứng trong exe** (thông báo chạy chữ, nhắc trong ô chat phòng) vẫn tiếng Hàn. Tên map ở phòng
  lấy từ dữ liệu map server gửi (client ghi đè `fxdynmapinfo.ssd`), nên là `Factory07`.
- **Nhãn tràn**: sửa bằng câu ngắn trong catalog (mục layout phía trên); các câu đã đo chờ cập nhật trong boom-re-l10n.
- **IME tiếng Hàn thật** (composition từng âm tiết) chưa thử trên Wine; đường đó đi `ImmGetCompositionStringW/A` sẵn có
  và qua hook khi không có ký tự tiêm.
- **Server lưu UTF-8**: chưa làm; server nhận và chuyển byte 949x. Client chưa vá thấy sai các ký tự mở rộng.
- **Màn đăng nhập** không có trong recipe (đăng nhập bằng token); chưa chụp.
- **Exe gốc `CA.exe`** vẫn chưa chạy trên Wine; bản vá áp trên exe runtime chẩn đoán.

**Để sau giai đoạn 1** theo [quyết định 15/11](../evidence/owner-directives/l10n-utf8-followup.json): chữ server gửi
(thông báo chạy chữ: xác định nguồn, nếu do server gửi thì server giữ theo ba thứ tiếng), chat chuyển tiếp (chế độ thử
nghiệm server chuyển nguyên byte chat 949x trong phòng), tiệm (server trả tiệm mẫu để mở màn hình) và gõ tiếng Hàn bằng
IME thật. Làm ngay: ảnh bị bỏ vì khác shape, chuỗi cứng trong exe, chat bị cắt và gõ tên phòng.

### Các lượt chẩn đoán trước (Codex, 14–15/11)

Trước khi chọn 949x, Codex đi theo hướng UTF-8 thuần và chạy các lượt chẩn đoán trên client copy: đọc tĩnh các hàm
chuyển mã, duyệt chuỗi, đo chữ và nhập liệu ([evidence tĩnh](../evidence/l10n-utf8-static.json),
[PopTag tĩnh](../evidence/l10n-utf8-poptag-static.json)); bộ chuyển mã tham khảo `tools/l10n_utf8_codec.py`, lõi C
`tools/l10n_utf8_core.c` và [probe Wine](../evidence/l10n-utf8-winprobe.json); các đối chứng
[01](../evidence/l10n-utf8-control-01.json), [02](../evidence/l10n-utf8-control-02.json),
[03](../evidence/l10n-utf8-control-03.json) (03 là baseline hai client tới phòng với dữ liệu Hàn gốc); các lượt glyph
[01](../evidence/l10n-utf8-glyph-01.json)–[04](../evidence/l10n-utf8-glyph-04.json), dialog
[05](../evidence/l10n-utf8-dialog-05.json), [06](../evidence/l10n-utf8-dialog-06.json),
[07](../evidence/l10n-utf8-dialog-07.json), [08](../evidence/l10n-utf8-dialog-08.json),
[09](../evidence/l10n-utf8-dialog-09.json), [10](../evidence/l10n-utf8-dialog-10.json) và
[fixture GDB](../evidence/l10n-utf8-gdb-fixture.json). Các lượt đó thay hoặc quan sát từng chuỗi bằng GDB và không
hiện được chữ mới; kết quả có giá trị còn dùng là: chuyển mã đi qua `0x149A740` ở bước đo chữ `0x14BADF0` (lượt 10),
gate ảnh cho hộp tạo phòng, và recipe hai client của đối chứng 03 mà các lượt 949x tái dùng. Bộ chuyển mã UTF-8 và
`tools/l10n_utf8_literal_plan.py` không dùng trong bản vá 949x.

**Đối chiếu IDA tĩnh, đang thực hiện.** Với PopTag SHA-256
`d1f9f9b080abe2557693e6c906c26e22674be5fca6abe13d8913fe1a7c1db5d7`, mã tại `0xB9A400` thử chuyển từng
đơn vị wide sang 949, rồi dùng bảng mã hai byte riêng khi ký tự không biểu diễn được; `0xB9A800` tra bảng đó hoặc
giải mã 949. `0xBC0000` có đường chuyển wide sang UTF-8 và thêm tiền tố `J9:` sau một bước mã hóa; `0xBC0400`
có đường tìm tiền tố, giải mã UTF-8 rồi chuyển về lớp byte riêng. `0xBBF000` xử lý một đường `VK_PACKET`, bỏ qua
surrogate và chuyển qua `0xB9A400`. Đây là kết quả tĩnh của build PopTag, không phải hành vi 00967; tên và độ phủ
các điểm vào còn chưa đầy đủ. [Metadata riêng](../evidence/l10n-utf8-poptag-static.json) ghi giới hạn IDA.
`0xB9AC00` lấy composition qua API W cho index 8 và 0x800 rồi chuyển qua `0xB9A400`, các index khác đi API A.
`0xB9B400`/`0xB9B600` thử giải lớp byte sang wide trước khi gọi vtable, có fallback; chưa định danh phép đo hay vẽ.

Image unpack 00967 SHA-256
`cf72da112308d9f1ca256df8089594c5f787875310933073bf6bd5ba6f70e63b`, database phân tích riêng;
nguồn `kr_static`, chưa kiểm runtime. `0x138D380` chọn 949 ở vùng 1 và gọi setter `0x149AA90`, setter chỉ ghi
`word_1A7E420`. `0x149A6D0` và `0x149A740` chuyển mã qua API Windows với biến đó, flags 0; ở đường không phải 1258,
hai tham số default character của chiều wide sang byte đều NULL. Khi độ dài đầu vào dương, `0x149A6D0` ghi NUL tại
`dst[result]` mà bản thân hàm không kiểm `result < capacity`; chưa biết caller có cấp thêm byte hay không.
`0x1499C20` và `0x1499C70` gọi API duyệt ký tự theo codepage. `0x149A420` chỉ nhận bốn codepage DBCS
932/936/949/950 là ký tự nhiều byte; `0x149A480` cũng phân biệt bốn codepage này. Vì vậy thay biến thành 65001
chưa đủ chứng minh duyệt, cắt, đo hoặc vẽ đúng UTF-8. Chưa có đặc tả vá hoàn chỉnh hay nghiệm thu ba ngôn ngữ;
cần kiểm từng caller, giới hạn buffer, font, nhập liệu và các đường mã cứng trước khi chọn thiết kế cuối.
Trong bốn caller được đọc tiếp, `0x8DEE90` và `0xB6C600` dùng kết quả `0x149A420` để chọn nhánh độ rộng và
ghép dòng; `0x14FE713` cộng độ rộng theo giá trị boolean cộng một và bước theo độ dài byte trả ra. `0x151E0E6`
dùng `0x1499C20` khi sao chuỗi vào một control. Chưa định danh đầy đủ control, đường IME hay độ phủ các caller;
đây là ứng viên cần xử lý, không phải danh sách điểm vá đã nghiệm thu.
`0x14BAC80` tính chiều rộng theo từng UTF-16 unit, chọn một trong hai bước rộng theo ngưỡng 0x7F; nhánh
`0x14BADF0` gọi vtable cho từng unit. `0x14BA910` cắt theo chiều rộng rồi chuyển wide về byte;
`0x14BB5E0`/`0x14BB7B0` chuyển sang wide trước khi chuyển tiếp để xử lý trình bày. Các hàm này dùng buffer
1024 wide unit; độ phủ glyph và hành vi surrogate vẫn chưa đo. Vtable ứng viên tại `0x17AF0CC` có entry `+0x18`
trỏ `0x14BAB10`, đọc advance theo UTF-16 unit từ dữ liệu font; entry `+0x14` và `+0x1C` chuyển tiếp sang các
backend trình bày. Chưa xác nhận object này được mọi control sử dụng.
Ba body khác có codepage cố định: `0x1238150` và `0x12381F0` gọi chuyển byte sang wide với 949;
`0x463A20` chuyển UTF-8 sang wide rồi sang 949. Chưa biết các đường này dùng cho nghiệp vụ nào, nên chưa thể
đổi toàn bộ hằng 949. Metadata và hash của các phép đọc nằm ở [evidence tĩnh](../evidence/l10n-utf8-static.json).
Ở đường nhập liệu, `0x152248F` dùng hai thunk tới slot ứng viên `ImmGetCompositionStringA/W`; hai nhánh W lấy
composition rồi chuyển con trỏ wide và chiều dài qua `0x1522910` tới control bằng vtable. Ánh xạ slot lấy từ báo cáo
khôi phục import và vẫn là candidate; đích vtable và chuyển chữ tới packet chưa được xác định. Kiểm tiếng Việt,
tiếng Hàn, surrogate và giới hạn byte cần lượt runtime riêng.
Các hàm control `0x15206B0` và `0x15206EE` lần lượt chuyển buffer byte sang wide và wide sang buffer byte 1023;
chưa chứng minh control này là ô chat hoặc tên phòng. Độ dài ký tự của `0x149A420` đi qua `0x1499C50`, là hiệu
giữa con trỏ kế và con trỏ đầu; sửa traversal ảnh hưởng trực tiếp cách các caller chia chuỗi.

**Thiết kế ứng viên cho 00967.** Ưu tiên UTF-8 trực tiếp trong dữ liệu và các trường văn bản của client đã vá,
với chuyển sang UTF-16 ở biên Windows và xử lý độ dài theo byte có kiểm tra. Cách này phù hợp mục tiêu dữ liệu chung
và tránh phụ thuộc vào bảng mã tạm theo trạng thái và marker riêng của build PopTag. Đây là lựa chọn thiết kế, chưa
chứng minh đủ điều kiện áp dụng: cần kiểm đường lookup khoá Hàn, 134 chuỗi 949 trong exe và các consumer 949 cố định;
không đổi packet của client chưa vá. Nếu các consumer không thể phân biệt nguồn mã hóa thì phải khảo sát lại thiết kế,
không coi chuyển biến codepage là bản vá hoàn chỉnh.

Các điểm ứng viên gồm setter `0x149AA90` và chuyển mã `0x149A6D0`/`0x149A740` với capacity và NUL đúng; duyệt
`0x1499C20`/`0x1499C70` và các caller cắt chuỗi theo ranh giới UTF-8 hợp lệ; nhánh độ rộng `0x149A420`/`0x149A480`
cùng caller đo và xuống dòng; font, glyph và biên UTF-16 khi vẽ; message input, composition và giới hạn trường trước
khi gửi. Các VA này là điểm điều tra, chưa có bản vá UTF-8 hay hợp đồng thay thế được nghiệm thu. Đặc biệt cần đo
caller có cấp dư chỗ cho NUL, xử lý chuỗi sai mã và ký tự ngoài BMP, kiểm giới hạn chat/tên phòng ở client và server,
cũng như xác định font và đường nhập thực tế. Chưa khẳng định nghiệm thu hiển thị hoặc nhập liệu.

| Điểm ứng viên | Hành vi cần có | Phần còn phải kiểm trước khi vá |
|---|---|---|
| `0x138D380`, `0x149AA90`, `0x1A7E420` | Khởi tạo encoding UTF-8 của bản vá chung, giữ lựa chọn vùng và ngôn ngữ riêng. | Thời điểm ghi, các lần setter khác và chuỗi 949 còn tồn tại. |
| `0x149A740`, `0x149A6D0` | Chuyển UTF-8 và UTF-16 với độ dài rõ, kiểm lỗi, chỗ trống và NUL. | Quy ước `-1`, số byte/unit và capacity của từng caller; hành vi lỗi không được âm thầm thành chuỗi khác. |
| `0x1499C20`, `0x1499C70`, `0x1499C50` | Tiến/lùi đúng ranh giới mã UTF-8 và từ chối chuỗi không hợp lệ theo policy thống nhất. | Phạm vi bộ đệm và cách báo lỗi của caller; không tách chuỗi giữa các byte. |
| `0x149A420`, `0x149A480`, `0x8DEE90`, `0x14FE713`, `0xB6C600` | Phân biệt độ dài byte với độ rộng và điểm cắt, kể cả ký tự ba hoặc bốn byte. | Boolean hiện tại chỉ phân loại DBCS; cần đối chiếu caller, không thay đơn giản thành `length > 1`. |
| `0x14BAC80`, `0x14BADF0`, `0x14BA910`, `0x14BB5E0`, `0x14BB7B0` | Đo, cắt và chuyển tiếp cùng văn bản Unicode, không cắt giữa cặp surrogate. | Font/đích vtable, đơn vị width và giới hạn 1023 unit; quy tắc combining mark chưa đo. |
| `0x152248F`, `0x1522910`, `0x15206B0`, `0x15206EE` | Giữ composition và ký tự nhập đến control; chỉ mã hóa UTF-8 tại biên byte đã xác định. | Ánh xạ slot API, vtable và control thật, hành vi IME, NUL và giới hạn chat/tên phòng. |
| `0x1238150`, `0x12381F0`, `0x463A20` | Xác định từng nguồn trước khi quyết định giữ 949 hay chuyển UTF-8. | Nghiệp vụ, caller và tính đầy đủ của kiểm kê hằng codepage. |

Các hành vi trên là yêu cầu thiết kế ứng viên; các hook quan sát hoặc thay thế thử có giới hạn được ghi riêng ở dưới,
chưa có bản vá UTF-8 được nghiệm thu.

**Ranh giới dữ liệu cần giải trước hook.** Bộ bảng chuỗi UTF-8 là đầu vào đã biết; ít nhất 134 chuỗi 949 ứng viên
trong exe vẫn có thể đi qua cùng các consumer. Danh sách này trích theo heuristic ít nhất ba âm tiết Hàn liên tiếp,
có thể sót literal ngắn. Cả 134 chuỗi đều khớp byte nguồn và cần nhiều byte hơn khi mã hóa UTF-8; IDA tìm được 141
tham chiếu tới đúng đầu chuỗi, trong đó 84 site không thuộc hàm đã nhận diện. Chưa kiểm tham chiếu gián tiếp, con trỏ
vào giữa chuỗi hoặc độ phủ toàn bộ literal. Con trỏ tới chuỗi gốc không bảo đảm nguồn còn nhận diện được sau khi sao chép,
nên phân loại theo địa chỉ hoặc thử giải UTF-8 rồi fallback 949 chưa đủ an toàn: byte 949 có thể tình cờ là UTF-8
hợp lệ. Bản vá dự kiến phải chuyển các literal 949 tại nguồn hoặc giữ nhãn encoding qua mọi lần sao chép, rồi kiểm
đủ consumer trước khi đặt codepage chung. Chưa có bảng tham chiếu, capacity hay provenance đủ để chọn cách nào đã
triển khai; không bật setter 65001 toàn cục làm bản thử ba ngôn ngữ. Bước implementation đầu là primitive chuyển mã
và adapter có contract kiểm length, lỗi, NUL; sau đó kiểm bản copy với input control, malformed và đối chứng chuỗi
949, rồi mới chạy bộ dữ liệu UTF-8. Chuyển hay mở rộng literal không được sửa image nguồn.
`tools/l10n_utf8_literal_plan.py` chỉ tạo danh sách ứng viên cho image diagnostic đã pin hash: nó kiểm byte, NUL và
141 vị trí pointer, nhưng không chỉnh image. Kết quả chưa đủ để áp relocation vì còn thiếu kiểm instruction, tham chiếu
gián tiếp, literal bị bỏ sót, lifetime và consumer.

`tools/l10n_utf8_codec.py` là bộ hàm tham khảo ngoài client: kiểm UTF-8 nghiêm ngặt, tìm ranh giới và cắt theo
byte, chuyển UTF-16 và cộng độ rộng do caller cung cấp. Bộ kiểm riêng dùng ví dụ Hàn, Việt, Anh, surrogate và byte
sai. Lõi C `tools/l10n_utf8_core.c` thực hiện giải mã và cắt với buffer có capacity; kiểm độc lập so với codec Python
và biên dịch chéo thành object i686. [Probe riêng trên Wine](../evidence/l10n-utf8-winprobe.json) khớp API Windows
ở một chuỗi hỗn hợp và các ca byte sai, ranh giới, buffer thiếu. Hai bộ này không áp hook, không cung cấp số đo font
và không chứng minh client hay wire hoạt động. API dùng độ dài tường minh, không tự thêm NUL; caller phải xử lý
terminator và không chồng buffer.

Một [đối chứng diagnostic riêng](../evidence/l10n-utf8-control-01.json) dùng chín file chữ gốc đã quan sát được
đường xử lý gói xác thực, nhưng client dừng trước bằng chứng sảnh với SIGABRT và sau đó illegal instruction; nguyên
nhân chưa biết. Không có bản vá hay dữ liệu UTF-8 trong lượt đó và chưa có ảnh sảnh, nên chưa thể dùng nó để kết luận
về chất lượng chữ hoặc nghiệm thu bản vá. [Đối chứng thứ hai](../evidence/l10n-utf8-control-02.json) thu stack
SIGABRT trỏ vào đường parser JSON `expected value`, phù hợp nhánh HTTP content đã biết ở CP073/CP201; lượt này không
thu được nội dung hay độ dài JSON nên chưa xác định nguyên nhân upstream. Illegal instruction xuất hiện sau khi
debugger detach, không phải PC của SIGABRT. Phép thử tiếp cần một điều kiện phân biệt mới hoặc recipe đã tới được UI.
Một [đối chứng hai client khác](../evidence/l10n-utf8-control-03.json) dùng recipe dịch vụ từng thành công và file
chữ Hàn gốc: ảnh cả hai phía cho thấy phòng Factory07 và hai người chơi. Đây là baseline cho client diagnostic,
không có bản vá UTF-8; cache map ở cả hai bản copy đã thay đổi trong lượt chạy. Client gốc, tiệm và nhập liệu vẫn chưa
được kiểm từ đối chứng này.
Một [phép thử glyph](../evidence/l10n-utf8-glyph-01.json) đã cài hook nhưng không khớp tiêu đề: client chọn một
trong sáu tiêu đề mặc định khác với chuỗi duy nhất hook nhận. Kết quả chưa xác định được khả năng vẽ chữ Việt; không
suy rằng đường chuyển mã không chạy hoặc font không hỗ trợ.
[Phép thử kế tiếp](../evidence/l10n-utf8-glyph-02.json) khớp một tiêu đề nhưng điểm dừng trả về của GDB không đáng
tin và phát lỗi; mọi quan sát thay thế trong lượt đó bị loại khỏi bằng chứng glyph. Chưa có kết luận font từ hai lượt.
[Lượt thứ ba](../evidence/l10n-utf8-glyph-03.json) ghi nhận thay thế có kiểm return và capacity, nhưng ảnh sau
thao tác mở hộp vẫn ở sảnh và không chứa chuỗi thử; log server không ghi create81. Chưa xác định nguyên nhân, cũng
chưa chứng minh glyph được vẽ hay server từ chối tiêu đề.
[Đối chứng pass-through](../evidence/l10n-utf8-glyph-04.json) chỉ quan sát một conversion, không ghi kết quả, cũng
không thấy hộp tạo phòng sau click. Sự khác biệt với control03 chưa phân biệt được overhead instrumentation, thời điểm
UI và các điều kiện khác. Phép đo tiếp phải có bằng chứng hộp đã sẵn sàng trước thao tác kế tiếp; chỉ thử thay chuỗi
sau khi control pass-through tới được hộp và được chụp. Không dùng sleep hay tên stage làm bằng chứng UI.
[Đối chứng không hook có gate ảnh](../evidence/l10n-utf8-dialog-05.json) đã chụp đúng hộp tạo phòng và sau đó
vào phòng; ảnh đã được xem trực tiếp. Gate là heuristic giới hạn cho 1024×768, không thay kiểm ảnh ở lượt tiếp.
[Đối chứng pass-through cùng gate](../evidence/l10n-utf8-dialog-06.json) khớp một conversion mà không ghi bộ nhớ,
nhưng debugger báo lỗi nội bộ GDB và kết thúc ngay sau marker; hết 20 giây không thấy hộp, ảnh cuối vẫn ở sảnh.
Chưa biết cơ chế lỗi GDB hay ảnh hưởng của nó tới UI; không suy đây là SIGSEGV của game hoặc lỗi font. Chưa chạy
thay chuỗi với controller này.

<a id="cp1258"></a>

**Exe 00967 có nhánh bảng mã 1258, không dùng.** Đọc tĩnh 14/11: công tắc vùng `0x138D380` chọn 949 cho vùng 1
(`push 3B5h` tại `0x138D3B1`) và 1258 cho vùng 64; các hàm chữ ở [bảng trên](#địa-chỉ-đã-đọc) đều có nhánh 1258 riêng
(`0x149A7B0`, `0x149AC90`, `0x1499C20`). Một bảng mã cho cả client không hiện được Hàn, Việt và Anh cùng lúc, nên
[hướng UTF-8 đã chốt](#utf8) giữ nguyên; các nhánh này chỉ là chỗ cần biết khi vá.

<a id="data-files"></a>

## Dữ liệu chữ và file dựng thử (trạng thái 14/11/2026)

Dữ liệu nằm ở checkout `boom-re-l10n` (repo `boom-re`, nhánh `claude/l10n-kr-vi`, commit `e5d1bac`, đã push).

| | Tiếng Việt | Tiếng Anh |
|---|---|---|
| Tổng số dòng catalog | 33.486 | 33.486 |
| Đã có bản dịch dùng được | 33.484 | 32.942 |
| Trong đó lấy từ bản chính thức | 3.813 (client VN) | 5.724 (PopTag, theo ID) |
| Trong đó dịch máy | 29.671 | 27.218 |
| Chờ duyệt | 2 (điều khoản pháp lý của Nexon, phải thay bằng điều khoản của dự án) | 0 |
| Không cần dịch | (tính trong dòng đã dịch) | 544: 105 dòng không có câu Hàn, 358 dòng không có chữ Hàn, 81 tên cấm trong exe giữ tiếng Hàn |

Catalog gồm 25.321 dòng chuỗi cũ và 8.165 dòng tên vật phẩm đọc từ `fxstock0.ini`–`fxstock3.ini` (4.967 tên khác
nhau). Bản dịch máy do model `gpt-6.1-terra` dịch theo lô. Tiếng Anh dùng tên nhân vật và thuật ngữ của bản Bắc Mỹ
(Boz, Evie, Plunk, Posh, Town…). Chưa có người đọc soát toàn bộ; các lượt kiểm đã làm đều bằng máy:

- **Kiểm định dạng trên mọi dòng:** tham số `printf` đúng số lượng và thứ tự, thẻ `<font>` giữ nguyên, số lần xuống
  dòng, chữ Hàn sót, từ miền Bắc trong bản Việt. Công cụ kiểm của repo báo 0 lỗi.
- **Soát nghĩa chữ PopTag so với câu Hàn** (5.417 cặp khác nhau, phủ 6.281 dòng): 652 dòng lệch nghĩa đã thay bằng bản
  dịch từ câu Hàn; 922 dòng giữ câu chữ PopTag nhưng lấy lại chỗ xuống dòng của câu Hàn, vì PopTag viết liền một dòng;
  15 dòng không đủ ngữ cảnh nên giữ chữ PopTag; còn lại khớp nghĩa.
- **Soát lần hai nhóm người chơi thấy nhiều nhất** (`csysstr.ssd` và `ClientString`, 8.964 câu): model đọc lại bản
  dịch với vai người soát lỗi. Đã áp dụng 364 sửa nội dung và 259 chỉnh khoảng trắng cho tiếng Việt, 68 sửa cho tiếng
  Anh. 162 đề xuất chạm vào dòng đã có quyết định duyệt trước đó thì không áp dụng. Người soát cùng model với người
  dịch nên không phải kiểm độc lập. Câu nhiệm vụ (`csysqstr.ssd`) và tên vật phẩm chưa qua lượt này.
- **Dòng chờ duyệt đã chốt:** 207 câu tiếng Anh từng treo (nhiều biến thể PopTag, tên tạm ghép theo icon, bản dịch bị
  máy kiểm đánh dấu) và 46 dòng tiếng Việt. Quyết định tiếng Việt do Claude đưa ra theo uỷ quyền của chủ dự án
  ngày 14/11/2026.

**Dấu `%` nghĩa đen trong bản dịch.** Câu Hàn như `경험치 +50% 3일` an toàn vì sau `%` là byte chữ Hàn. Bản dịch
`+50% for 3 Days` hay `giảm 20% nha` thì tạo ra `% f`, `% n`, là tham số `printf` hợp lệ; nếu client đưa chuỗi đó qua
`printf` thì đọc sai tham số. Chưa biết chuỗi nào đi qua `printf`, nên công cụ kiểm coi mọi trường hợp như vậy là lỗi
và các câu đã được viết lại để sau `%` là dấu câu hoặc hết câu.

**Bề rộng chữ (ước lượng tĩnh).** `tools/l10n_width.py` cộng bề rộng bước của từng ký tự trong `CxUni.fft` rồi so
dòng rộng nhất của bản dịch với câu Hàn. Trung vị của bản dịch máy rộng gấp 1,16–1,42 lần câu Hàn, và phân bố này
trùng với bản chính thức (chữ PopTag cho tiếng Anh, chữ client VN cho tiếng Việt: trung vị 1,2–1,36 lần; mức 95% là
1,7–2,4 lần ở cả hai nhóm). Tức bản dịch máy không dài hơn các bản từng phát hành. Phép đo không biết bề rộng khung
thật của từng nhãn và không biết nhãn nào dùng font nào; chữ tràn ở đâu phải xem trên client.

**Định dạng file `ssi1`** (`csysstr.ssd`, `csysqstr.ssd`, `fxdynmapinfo.ssd`, `fxstock*.ini` và các file `.ssd` khác):

| Offset | Nội dung |
|---|---|
| 0 | `"ssi1"` |
| 4 | u32 checksum: Adler-32 với giá trị khởi đầu 0, tính trên mọi byte từ offset 8 tới hết file |
| 8 | u8 `0x18` |
| 9 | u32 độ dài phần còn lại sau trường này (độ dài file trừ 13) |
| 13 | u8 `1` |
| 14 | u32 độ dài dữ liệu sau giải nén |
| 18 | luồng zlib |

Checksum đọc từ exe VN đã unpack: hàm đọc `0xA62D10` tính lại rồi so, lệch thì không nạp file; hàm ghi `0xA62C20`;
hàm băm `0xA0E1E0`. Công thức khớp cả 13 file `ssi1` của client KR. Chưa đọc hàm tương ứng trong exe KR.

**Công cụ dựng** `tools/l10n_build.py` ghi catalog thành bảy file `ssi1` và entry `ClientData/ClientString` của pack
`FxDesktop` (ghi nối vào cuối `.idd`, sửa `.idx`), cho `ko`, `vi`, `en` với `cp949` hoặc `utf-8`. Phép kiểm đã đạt:

- Dựng lại tiếng Hàn: cả bảy file giải nén ra giống gốc từng byte (luồng nén khác nên file nén khác byte); pack
  không đổi mục nào.
- Mọi file `ssi1` của ba bộ dựng có checksum, hai trường độ dài và độ dài giải nén đúng.
- Pack của bộ Việt và bộ Anh: 2.076 trên 2.077 mục index không đổi, phần `.idd` gốc giữ nguyên, bảng chuỗi mới đọc
  lại được trọn vẹn theo một bảng mã và đủ 6.993 mục.
- Bản dựng tiếng Việt UTF-8 thay 32.098 chuỗi; bản dựng tiếng Anh theo bảng mã gốc `cp949` thay
  32.102 chuỗi. Không còn chuỗi nào bị loại vì lệch định dạng hay không mã hoá được.

**L-probe-b: dữ liệu chữ đã được dùng trên bản runtime G1; chưa phải nghiệm thu toàn bộ bản dịch.**
[Metadata, recipe và raw/hash](../evidence/l10n-probe-b.json) giữ ma trận native, ảnh và đối chứng checksum.
Bộ dựng lấy nguyên source/catalog `boom-re-l10n@11bc77c55c769e3b75ae87d27d80098f22c51ff2` qua bản Git archive riêng,
không sửa công cụ hoặc bản dịch nguồn. Bộ Hàn giải nén khớp gốc; các file dựng có checksum/độ dài đúng.
Lượt này dùng đúng exe diagnostic của native G1, SHA-256
`4b781fbcea21e0f6294c729b63be86ab2953d60ad83b0d0c9219359f826107ed`, Wine
`10.0-20260425 (CachyOS)`, hai account, prefix riêng và LAN/relay ảo cô lập trên một máy. Không thêm vá ngôn ngữ,
UTF-8, font hoặc region. [CP300](../evidence/checkpoints/records/CP300.json) giữ nguyên năm lượt `CA.exe` gốc
bị page fault trước login; bằng chứng diagnostic mới không biến các lượt gốc thành đạt.

| Lượt | File thay | Quan sát native và giới hạn |
|---|---|---|
| 0 | Không thêm dữ liệu ngôn ngữ | Hai client tới sảnh. Đối chứng thao tác menu sau đó có phòng Camp08, player106 đọc hết và đúng identity; shop/mission chưa mở được |
| 1 | Bảy `ssi1` tiếng Hàn ghi lại | Hai client tới sảnh, chữ Hàn giữ đúng ở ảnh đã xem. Không có chứng minh riêng cho bốn loader stock hoặc toàn bộ câu nhiệm vụ |
| 2 | Bảy `ssi1` tiếng Anh CP949 | Hai client tới sảnh; menu “Shop”, “My Items”, “Quests”, “Options”, các tab/chat hiện Anh. Nút dạng ảnh và dòng thông báo vẫn có Hàn |
| 3 | Lượt 2 cộng `FxDesktop.idx/.idd` tiếng Anh | Tới sảnh/phòng; tên phòng mặc định, “Players: 8”, câu hướng dẫn và menu hiện Anh. Nút ảnh vẫn Hàn; không suy toàn bộ pack/chuỗi đã render |
| 4 | Chỉ `csysstr.ssd` tiếng Anh đổi một byte nén tại offset 34, giữ checksum cũ | Loader tính checksum khác, trả 0; `csysqstr.ssd` hợp lệ cùng lượt trả 1. Client vẫn tới phòng, câu hướng dẫn hiện nguyên khóa `SuperChiefCanStartDetail`; không phục hồi file hỏng trong lượt đo |
| 5 | Anh CP949 và 314 ảnh Bắc Mỹ trong năm pack | Tới sảnh/phòng; ảnh VIEW ALL/PRACTICE/MENU/POP SHOP/MY PAGE, ROOM SETTINGS, Creator Maps/Original Maps và GO! hiện Anh. Một số nút/ảnh vẫn Hàn; không chứng minh mọi entry đã render |

**Dữ liệu thực sự được dùng.** Chữ Anh nhìn thấy trong menu và phòng là bằng chứng native; sự kiện mở/đóng file
chỉ là đối chứng I/O, không tự chứng minh consumer dùng hết dữ liệu. Tên phòng “Beginners welcome!” và câu hướng dẫn
về Super Room Leader Item khớp key/catalog. Ở các hàng menu đã xem chưa thấy chữ tràn hoặc placeholder định dạng
bị in nguyên; câu hướng dẫn trong phòng xuống dòng. Không có kiểm mọi câu hoặc mọi kích thước cửa sổ.

**Checksum được kiểm trên KR.** Tái dùng loader KR `14EA7C0` đã có ở
[contract cache](../protocol/lobby-profile.md#dynamic-cache-representation), probe chỉ đọc tại `14EA885` trước so
và `14EA8E8` sau lệnh trả EAX. File hỏng cho checksum tính khác checksum lưu, return 0 ở cả hai client; file hợp lệ
đối chứng khớp và return 1. Đây là từ chối riêng file, không phải yêu cầu client phải crash hoặc thoát.

**Cache map bị server cập nhật.** Các lượt thay `fxdynmapinfo.ssd` đầy đủ đều ghi lại cache 71 byte của profile
Camp08. Inner giải nén của cache sau run giống từng byte với S→C238 thực sự đã capture; đây là đường cache đã biết,
không là client khôi phục file dịch hay nhận patch từ Internet. Các file chữ/stock và pack còn lại giữ nguyên hash
trước/sau, kể cả `csysstr.ssd` cố ý hỏng. Không suy toàn bộ map-display translation đã được dùng từ file dựng ban đầu.

**Giới hạn còn lại.** Shop và mission đã thử bằng UI nhưng chưa hiện nội dung trong recipe này; không nghiệm thu
bốn file stock, tên item hoặc câu nhiệm vụ từ việc chúng có mặt. Không thêm protocol shop hay đổi service để ép mở.
Các lần điều chỉnh thao tác và lỗi đường dẫn trước namespace giữ riêng, không gọi chúng là client từ chối bộ chữ.
Chỉ xác nhận Wine/copy diagnostic với các màn đã chụp, chưa Windows thật hoặc exe gốc. L1 UTF-8/IME vẫn do Claude
phụ trách và chưa được nghiệm thu từ L-probe-b. Nhóm chẩn đoán dữ liệu đã kết thúc; Codex tiếp G1 theo PLAN.

**Khoá có chữ Hàn.** 29 khoá trong `csysstr.ssd`, một khoá trong `csysqstr.ssd` và 30 khoá trong XML `ClientString`
chứa chữ Hàn (ví dụ `럭마`, `슈방`: tên tắt của vật phẩm). Quét image exe đã unpack, 13 file `ssi1` và 51.467 mục của
mọi pack KR không thấy nơi nào tham chiếu các khoá này bằng byte `cp949`, ngoài chính chỗ định nghĩa. Vì vậy bản dựng
UTF-8 chuyển cả khoá sang UTF-8 để mỗi file chỉ mang một bảng mã; cờ `--keep-key-bytes` giữ byte gốc. Chưa biết client
tra các khoá này từ đâu lúc chạy (chữ người chơi gõ hay chuỗi server gửi).

**Ảnh có chữ.** Model `gpt-6-luna` đọc 3.038 mục ảnh nghi có chữ trên các tờ ảnh ghép đánh số (mỗi mục tối đa ba ảnh
con: đầu, giữa, cuối), kết quả ở `l10n/images_text.csv`:

| Nhãn | Số mục |
|---|---|
| Có chữ Hàn | 1.104 |
| Chỉ có từ Latin (START, OK…) | 149 |
| Chỉ có chữ số, ký hiệu | 165 |
| Không có chữ | 1.598 (gồm 18 mục không có ảnh con để xem) |
| Không đọc được | 22 |

Trong 1.104 mục có chữ Hàn (746 cụm chữ khác nhau), 654 mục có ảnh cùng tên nhưng khác nội dung ở client VN (564 mục) hoặc ở
PopTag (358 mục), là ứng viên lấy thẳng sau khi xem bằng mắt; 450 mục không có bản nào, phải vẽ lại. Đối chiếu
tay 15 ô của một tờ thì model đọc đúng cả 15; chưa kiểm rộng hơn. 282 mục có nhiều hơn ba ảnh con, có thể sót chữ ở ảnh
không được hiện. Cùng tên và khác byte không chứng minh ảnh kia đã được dịch.

## Chưa biết, phải đo khi mở đợt L

- **Giới hạn độ dài tính theo byte.** Với [949x](#utf8) chữ Việt có dấu chiếm hai byte. Tiêu đề phòng giới hạn 1..22 byte
  ở client (`0xC542C0`); chat và tên nhân vật chưa đo giới hạn.
- **Client có lọc bảng chuỗi theo mã ngôn ngữ không.** Mỗi mục trong `ClientString` của pack mang mã 1042.
- **Dùng font của client VN** (ô 12×16) trong client KR để chữ có dấu dễ đọc hơn: hàm nạp đọc kích thước từ header và
  [client PopTag](#poptag) phát kèm đúng bộ font đó cho build 2010, nhưng bố cục giao diện của 00967 có thể giả định
  dòng cao 12 điểm.
- **Server biết ngôn ngữ của từng người bằng cách nào** để gửi đúng bảng chuỗi: lưu theo account hay trình mở game báo
  lúc đăng nhập.
- **Kiểm toàn vẹn** của client với exe vá và pack/ssi1 dựng lại: các lượt 949x trên exe runtime `CA-r2-*` nạp được
  bộ dựng mà không thấy phục hồi file; exe `CA.exe` nguyên bản vẫn chưa chạy được trên Wine (CP300).
- **Client tra 30 khoá có chữ Hàn từ đâu lúc chạy** và bản dựng UTF-8 đổi khoá sang UTF-8 có làm mất chức năng nào
  không; quét tĩnh không thấy tham chiếu nào.
- **Chuỗi nào đi qua `printf`** với chính nó làm chuỗi định dạng; hiện coi mọi dấu `%` nghĩa đen là có nguy cơ.
- **Không đổi vùng.** Đổi vùng sang Việt sẽ bật 201 nhánh riêng của vùng Việt, có thể đổi luồng đăng nhập, giao diện và
  packet đã đo; không làm nếu chưa có lý do.
