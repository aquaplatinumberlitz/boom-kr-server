# KR writer helpers — đối chiếu byte gốc bằng emulation

Ngày 26/10/2026. Source `kr_static`, trạng thái `confirmed` **chỉ cho helper biệt lập**.
Codex kiểm direct disassembly và chạy byte từ image KR trong Unicorn; không chạy CA/bootstrap/network.

Target unpack `kr-full.exe`, SHA-256
`cf72da112308d9f1ca256df8089594c5f787875310933073bf6bd5ba6f70e63b`, image base `0x400000`.
Đây là dump image khác file packed CA.exe; VA sau chỉ áp cho image này. Byte code đọc từ section
`.priv`, không commit raw code/dump. Tool tái tạo: [verify_primitives.py](../../../tools/verify_primitives.py).
Metadata: [verification.json](verification.json).

| VA / RVA | Hành vi helper | Ca kiểm | Kết quả |
|---|---|---:|---|
| `0x14A5B90` / `0x10A5B90` | Ghi hai byte high trước, low sau từ 16 bit thấp của argument | 65.540 | PASS |
| `0x14A5C90` / `0x10A5C90` | XOR argument với `0xA569`, ghi hai byte low16 theo big-endian | 65.540 | PASS |
| `0x14A5C80` / `0x10A5C80` | XOR byte thấp với `0x5A`, ghi một byte | 260 | PASS |

Mỗi helper kiểm hết miền 16-bit hoặc 8-bit, cộng bốn argument32 ngoài miền. Mỗi call reset stack,
buffer guard; phải return trong100instruction và không ghi ngoài số byte của helper. Expected transform
lấy từ disassembly của KR, rồi so với byte helper thực thi trong emulator; không lấy key VN làm default.

Ví dụ phạm vi đã kiểm: plain16 `0x1234 → 12 34`; xor16 `0 → A5 69`, `0xFFFF → 5A 96`;
byteXOR `0 → 5A`, `0xFF → A5`. Đây không phải mẫu frame đầy đủ.

## Giới hạn bắt buộc

- Chưa chứng minh mode nào được chọn trong một live connection hoặc mọi packet167; global trong
  snapshot dump không tự chứng minh runtime invariant.
- Chưa xác nhận header/type/length/checksum/payload crypto/seq/hello/socket role. `0xA569` là field
  transform trong helper này, không gọi là key/seq toàn frame.
- Guard/return và exhaustive input chỉ kiểm helper, không chứng minh caller/layout hay client K1/K2.
- Input binary sai hash bị chặn; output cũ không được ghi đè. Raw local và hash ghi trong JSON.

## Receiver ownership ở send path

Review raw của dsh2 cho thấy `0x1164D50` push packet nhưng đặt **ECX bằng object từ `0x545E30`**
trước `call 0x13ADD10`. Trong `13ADD10`, `13ADA90` và field `+0x3C` đọc từ chính ECX đó;
packet là argument `[ebp+8]` được chuyển tiếp tới virtual `+0x40`.

**Confirmed trong call chain đã đọc:** `+0x3C` ở guard/delegate thuộc receiver từ `545E30`, không phải
packet. Writer field/socket trên route WCP về sau có tại [PROTOCOL](../../PROTOCOL.md#wcp);
full send delegate/codec vẫn unknown. Không nối field `+0x3C` có
cùng offset trong base constructor packet với receiver khác này. Raw giữ local trong
`local/research/dsh2/codec-r1.asm` và `codec-r3.asm`. Snapshot lịch sử;bước hiện hành ở [RESEARCH](../../RESEARCH.md#q-023).
