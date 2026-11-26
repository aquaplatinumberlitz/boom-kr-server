# Chính sách vận hành sự kiện theo mùa F6-1

F6-1 định nghĩa cấu hình và công cụ kiểm định offline cho KR 00967. Cấu hình là quyết định vận hành (`design`). Kết quả kiểm định chỉ xác nhận cấu hình hợp lệ; lát cắt này chưa nối profile vào server hay thực thi lịch và giới hạn tần suất.

## Kiểm định và chuẩn bị cấu hình

[Profile mặc định](../../server/profiles/f6-events-disabled.json) tắt cả 19 hệ thống, không đặt lịch và đặt mọi giới hạn bằng 0. Chạy từ thư mục gốc của repo:

```bash
python3 tools/kr_f6_policy_validate.py --check server/profiles/f6-events-disabled.json
```

Để chuẩn bị một mùa, sao chép profile sang tệp đề xuất, chỉ sửa mục cần bật rồi chạy `--check` với đường dẫn của bản sao. Mã thoát `0` nghĩa là hợp lệ; mã khác `0` nghĩa là từ chối toàn bộ profile. Dùng lại profile mặc định khi cần chuẩn bị cấu hình tắt toàn bộ.

## Cấu trúc `kre.f6-events-policy.v1`

Profile phải có đúng ba khóa cấp cao nhất: `schema`, `default_enabled` và `systems`. `schema` phải là `kre.f6-events-policy.v1`; `default_enabled` phải được khai rõ là `false`. `systems` phải chứa đúng 19 tên `in_scope` trong [catalog KR 00967](../evidence/content/f6-events-catalog.json), giữ nguyên chữ hoa, chữ thường và dấu tiếng Việt. Các mục `related_candidate` nằm ngoài cấu hình F6.

| Khóa bắt buộc cho mỗi hệ thống | Giá trị |
|---|---|
| `enabled` | Boolean. Chọn `false` để tắt riêng hệ thống, kể cả khi đã chuẩn bị lịch. |
| `start_utc`, `end_utc` | Cả hai có thể là `null` nếu hệ thống tắt. Nếu có lịch hoặc `enabled: true`, phải khai cả hai theo dạng UTC `YYYY-MM-DDTHH:MM:SSZ` và thỏa `start_utc < end_utc`. Khoảng hoạt động theo thiết kế gồm mốc đầu và không gồm mốc cuối. |
| `rate_caps.per_account_per_hour` | Số nguyên từ 0 đến 60 thao tác mỗi tài khoản mỗi giờ. |
| `rate_caps.global_per_minute` | Số nguyên từ 0 đến 600 thao tác cho hệ thống trên toàn server mỗi phút. |

Mỗi hệ thống phải có `rate_caps` với đủ hai khóa. Theo thiết kế, thao tác phải nằm trong cả hai giới hạn; giá trị `0` chặn mọi thao tác thuộc giới hạn đó. Công cụ từ chối khóa lạ, khóa trùng, trường thiếu và giá trị sai kiểu.

Ví dụ thay mục `systems.CAPass` trong bản sao profile. Giữ nguyên cả 18 mục còn lại:

```json
{
  "enabled": true,
  "start_utc": "2027-01-01T00:00:00Z",
  "end_utc": "2027-01-08T00:00:00Z",
  "rate_caps": {"per_account_per_hour": 6, "global_per_minute": 60}
}
```

## Mục loại trừ và giới hạn bằng chứng

Theo [phạm vi F6 trong PLAN](../PLAN.md#full-game-roadmap), `PCRoomBenefit` (ưu đãi phòng máy Nexon) mang trạng thái `excluded_policy`. Profile bị từ chối nếu khai mục này, kể cả với `enabled: false`. Công cụ cũng từ chối nếu catalog không còn ghi nhận mục này là `excluded_policy`.

[Kiểm kê F6-0](f6-events-catalog.md) ghi metadata tĩnh (`kr_static`) của build KR 00967; catalog JSON giữ hash và locator. Opcode wire, giá, tỷ lệ của server KR gốc và trạng thái hoạt động chưa đo đều giữ `unknown`. Các giới hạn và lịch ví dụ ở đây là giá trị do operator chọn trong phạm vi chính sách, không phải số liệu KR gốc.
