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

Harness/probe/bootstrap và codec mới sẽ đặt tại đây hoặc `server/`; không ghi vào tool repo VN/KR cũ.
Raw output, bản copy client/DB và capture đặt trong `local/`, không commit.
