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

Hệ quả: tiếng Việt trên client 00967 **không cần viết thêm bộ vẽ chữ hay font**. Đường ngắn nhất là đổi bảng mã và mã
ngôn ngữ; chủ dự án đã chọn đường khác, [đưa mọi chữ về UTF-8](#utf8), để người chơi khác ngôn ngữ đọc được chữ của nhau.

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
2. **Một mẫu chạy nhiều ngôn ngữ trong cùng một client.** Section `.u8code` do nhóm sửa thêm vào được gọi từ 63 chỗ
   trong mã gốc và dùng bảng mã 65001 (UTF-8); section `.vkinp` bọc `PeekMessageW`/`PostMessageW` của đường nhập phím.
   Gói kèm bốn bộ font chọn bằng một file `.bat`: Anh-Hàn, Đài Loan, Việt, gốc. Tức là họ đưa chữ về UTF-8 thay vì đổi
   bảng mã theo từng ngôn ngữ. Chưa đọc kỹ từng hàm trong `.u8code`.
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

Theo [quyết định 13/11](../evidence/owner-directives/l10n-utf8.json), đợt L đưa mọi chữ của client về UTF-8 như
[client PopTag](#poptag), không đổi bảng mã theo từng ngôn ngữ. Hệ quả:

- **Một bản vá chung cho cả ba ngôn ngữ.** Khác nhau giữa Hàn, Việt, Anh chỉ còn là dữ liệu: bảng chuỗi, font và ảnh
  có chữ. Người chơi khác ngôn ngữ đọc được tên, tiêu đề phòng và chat của nhau.
- **Chỗ phải vá ở 00967** là các hàm đã định vị ở bảng trên: đổi byte sang Unicode `0x149A740`, đổi ngược `0x149A6D0`,
  và hai hàm "ký tự hai byte" `0x149A420`, `0x149A480` (UTF-8 có ký tự hai và ba byte nên phép cắt chuỗi, đếm ký tự và
  di chuyển con trỏ phải đổi theo). Nhánh 1258 và bước ghép dấu có sẵn không còn cần.
- **Server đổi theo.** Tên, tiêu đề phòng và chat lưu và chuyển dưới dạng UTF-8; chuỗi server gửi cũng là UTF-8. Dữ
  liệu đã lưu theo bảng mã 949 phải chuyển một lần.
- **Tiếng Hàn không còn là "không tốn việc":** bảng chuỗi tiếng Hàn phải chuyển mã sang UTF-8, làm bằng máy.
- **Chọn ngôn ngữ ở bước mở game.** Trình mở game chọn bộ dữ liệu và áp bản vá; không thêm mục chọn vào menu của
  client. Lý do: client nạp bảng chuỗi, font và ảnh một lần lúc khởi động; exe gốc bị đóng gói nên bản vá nhiều khả năng
  phải áp lúc chạy, tức là đằng nào cũng cần trình mở game; thêm màn hình mới vào giao diện client là việc dịch ngược
  riêng. Đổi ngôn ngữ có hiệu lực ở lần mở game sau.

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

**L-probe đã chạy năm lượt, nhưng bị chặn ở khởi động exe gốc trên Wine.**
[CP300 và chỉ mục raw/hash](../evidence/checkpoints/records/CP300.json) ghi nguồn, recipe, ảnh, trace file,
exception và cleanup. Bộ dựng dùng `boom-re-l10n@0e35df10dfa527898ae4a7617d7654b87db403ba`, là hậu duệ của
`e5d1bac`; không sửa công cụ hoặc bản dịch. Mỗi lượt dùng bản sao đầy đủ và `CA.exe` nguyên bản có SHA-256
`a52545c14b1fea423de655c3163d6360b628d61c1cd472b46fdbc8a9ffe15f4e`, prefix riêng cùng mẫu, Wine
`10.0-20260425 (CachyOS)` trên một máy và mạng namespace chỉ có loopback. Không vá exe, font hoặc region.

| Lượt | File thay so với bản gốc | Quan sát native | Login → sảnh → shop → mission → phòng |
|---|---|---|---|
| 0 | Không | Hộp Wine `Program Error`, page fault | Không tới được |
| 1 | Bảy `ssi1` tiếng Hàn ghi lại | Hộp Wine `Program Error`, page fault | Không tới được |
| 2 | Bảy `ssi1` tiếng Anh CP949 | Hộp Wine `Program Error`, page fault | Không tới được |
| 3 | Lượt 2 cộng `FxDesktop.idx/.idd` tiếng Anh | Hộp Wine `Program Error`, page fault | Không tới được |
| 4 | Chỉ `csysstr.ssd` tiếng Anh đổi một byte nén tại offset 34, giữ checksum cũ | Hộp Wine `Program Error`, page fault | Không tới được |

Cả năm lượt có ảnh khởi động/lỗi, window tree và trace `+file`; không thấy tên chín file mục tiêu trong trace.
Đây là quan sát âm có giới hạn, chưa xác nhận consumer đọc dữ liệu hoặc nguyên nhân page fault. Không có kết nối
TCP tới listener bootstrap, nên fixture 15 byte chưa được gửi; không có packet danh sách file/hash hoặc ghi đè từ
server. Hash trước/sau của cả chín file mục tiêu không đổi trong từng lượt, kể cả file cố ý hỏng. Điều này chỉ cho
biết chưa thấy phục hồi file trong cửa sổ đo. Client nguồn không đổi; mọi run đã cleanup và seal.

Bộ Hàn giải nén khớp gốc; 14 file dựng có checksum/độ dài đúng bằng kiểm tĩnh. Đối chứng lượt 4 khác đúng một byte,
checksum sai và zlib không giải nén được bằng kiểm tĩnh. **Chưa đo được KR chấp nhận file dựng hoặc từ chối checksum
sai**; baseline chưa tới login nên không phân đôi file hay retry cùng cấu hình. Chưa đánh giá được chữ Anh/Hàn,
clipping, tràn khung hoặc tham số định dạng trên UI game. Chữ Anh trong hộp lỗi thuộc Wine, không phải bản dịch game.
Không suy kết quả trên Windows thật. Việc xác minh nạp dữ liệu còn mở cho đợt L, cần baseline exe gốc tới được
consumer trước; không thay kết quả đó bằng bằng chứng từ exe chẩn đoán đã vá và không chặn G1.

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

- **Đoạn mã UTF-8 của PopTag làm gì ở từng chỗ.** Đọc kỹ `.u8code`, `.u8data`, `.vkinp` và 63 chỗ gọi trong build 2010,
  rồi tìm chỗ tương ứng ở 00967. Đây là việc đầu tiên của đợt.
- **Áp bản vá vào 00967 bằng cách nào.** Exe gốc bị đóng gói và image đã unpack không có bảng import, nên chưa có file
  exe vá sẵn chạy được; phương án đang nghĩ tới là vá trong bộ nhớ lúc mở game.
- **Giới hạn độ dài tính theo byte.** Chữ Hàn dài ba byte ở UTF-8 thay vì hai, chữ Việt có dấu dài hai hoặc ba byte. Tên,
  tiêu đề phòng và chat có giới hạn trong client và trong packet; phải đo từng trường.
- **Client có lọc bảng chuỗi theo mã ngôn ngữ không.** Mỗi mục trong `ClientString` của pack mang mã 1042.
- **134 chuỗi nằm trong exe** là byte theo bảng mã 949 và sẽ hiện sai nếu đọc như UTF-8; phải vá hoặc cho đi đường riêng.
- **Gõ tiếng Việt và tiếng Hàn trong client** (ô chat, tên phòng) sau khi vá: chưa xem đường nhập.
- **Dùng font của client VN** (ô 12×16) trong client KR để chữ có dấu dễ đọc hơn: hàm nạp đọc kích thước từ header và
  [client PopTag](#poptag) phát kèm đúng bộ font đó cho build 2010, nhưng bố cục giao diện của 00967 có thể giả định
  dòng cao 12 điểm.
- **Server biết ngôn ngữ của từng người bằng cách nào** để gửi đúng bảng chuỗi: lưu theo account hay trình mở game báo
  lúc đăng nhập.
- **Kiểm toàn vẹn** của client với mã đã vá và file chữ đã thay. Nguồn checksum `ssi1` và giới hạn phép thử KR
  nằm ở [mục dữ liệu chữ](#data-files); phần còn lại (pack, exe) chưa đo.
- **Client tra 30 khoá có chữ Hàn từ đâu lúc chạy** và bản dựng UTF-8 đổi khoá sang UTF-8 có làm mất chức năng nào
  không; quét tĩnh không thấy tham chiếu nào.
- **Chuỗi nào đi qua `printf`** với chính nó làm chuỗi định dạng; hiện coi mọi dấu `%` nghĩa đen là có nguy cơ.
- **Không đổi vùng.** Đổi vùng sang Việt sẽ bật 201 nhánh riêng của vùng Việt, có thể đổi luồng đăng nhập, giao diện và
  packet đã đo; không làm nếu chưa có lý do.
