# Tool của server KR

`check_docs.py`: kiểm file đích của link Markdown nội bộ, không gọi network; **không kiểm fragment/anchor**.
`artifacts.py verify`: kiểm SHA/size đầu vào RE theo manifest **host-local**; chỉ đọc source.
Máy khác dùng `--manifest /path/to/local-artifacts.json`; tool không tải hoặc tự tạo binary nguồn.

`verify_primitives.py`: chạy byte helper KR gốc trong Unicorn, kiểm output, guard byte và bounded
return. Chỉ nhận image unpack có SHA đã khóa; không chạy client/bootstrap hoặc network. Cần dependency
RE tùy chọn trong `requirements-re.txt`; kết quả mới phải có output path riêng, không ghi đè raw cũ.

```bash
python3 tools/verify_primitives.py \
  --binary local/research/dsh2/kr-full.exe \
  --out local/research/codex/primitives-run2.json
```

Phạm vi bằng chứng: [writer primitive](../docs/evidence/primitives/README.md). Tool không xác nhận
header/checksum/crypto/seq hoặc mode selection của packet/socket đang chạy.

`build_wine_copy.py`: dựng đúng recipe KR copy đã đo, khóa SHA3 đầu vào và output, ghi before/after
từng patch vào report local. Cần `pefile==2024.8.26`; candidate metadata là artifact host-local.
`wine_probe.py`: chạy bounded trong namespace riêng, fresh CALog/window/screenshot/Winsock và accept
TCP loopback mặc định silent; `--bootstrap-reply` gửi fixture local một lần trên connection đầu tiên. Runtime paths chỉ nhận copy dưới `local/`; xem
[recipe Wine](../docs/evidence/wine/README.md) để biết lệnh chạy và giới hạn.

Harness/probe/bootstrap và codec mới đặt tại đây hoặc `server/`; không ghi vào tool repo VN/KR cũ.
Raw output, bản copy client/DB và capture đặt trong `local/`, không commit.

`verify_bootstrap.py`: native length/copy/receive/reader với modO SHA khóa, allocator tổng hợp,
254 ca; không thực thi full ProcessPatch hoặc xác nhận hello game. Lệnh và giới hạn ở
[bootstrap wire](../docs/evidence/bootstrap-wire/README.md).
