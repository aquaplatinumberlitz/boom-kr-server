# Ghi và phát lại phiên KR 00967

Trong lab cô lập, mở capture trên interface nhìn thấy TCP cục bộ của companion
và UDP/QUIC. Nếu chúng nằm trên hai interface, ghi mỗi interface trong cùng
khoảng thời gian và gộp vào một capture trước khi pin. Bật diagnostics của
companion và giữ `rust-events.jsonl` của server cùng journal. Ví dụ:

```sh
python3 tools/session_replay.py capture --interface lo --host 127.0.0.1 \
  --seconds 300 --output local/run/session.pcap
python3 tools/session_replay.py pin --events local/run/rust-events.jsonl \
  --gate local/run/state-gate.jsonl --pcap local/run/session.pcap \
  --companion local/run/prelogin.log --journal local/run/journal.json \
  --client local/run/CA.exe --server local/run/kr-service \
  --output local/run/session-private.json
```

`capture` cần tcpdump và quyền bắt gói đã cấp trên interface. Lưu thông tin mất
gói của tcpdump cùng dữ liệu nếu muốn đánh giá độ đầy đủ. Lặp `--companion` và
`--client` khi có nhiều client. Capture và diagnostics chỉ nằm trong `local/`.
Không lưu khóa TLS hoặc dùng gói của phiên cũ để xác thực phiên mới.

CI dùng hai projection từ [CP373](../evidence/checkpoints/records/CP373.json) và
[CP375](../evidence/checkpoints/records/CP375.json): mỗi phiên hai vòng trên cùng
phòng, có Ready/Start, Load, relay, kết quả, journal và quay lại phòng. CP373 có
`elimination`; CP375 có `all_dead`. Fixture chỉ mang mốc và hash nguồn; payload
nạp lại do driver dựng cho tài khoản, khóa và cổng loopback mới. Test dùng profile
Rust tổng hợp, không thay oracle map, clock hay input gốc. Relay gửi datagram thử
và kiểm cổng cũ sau vòng mới, không phát lại QUIC, WAN hay chat của CP375.

Để tái lập fixture dạng này, dùng `project --events … --gate … [--pcap …] --output …`.
Gate phải có hai dòng `journal-complete-1/2`, mỗi dòng mang mảng `rounds` có bản ghi
`[round, room_id, room_session, state, reason, map]` cuối là ván mới hoàn tất.
`python3 tools/session_replay.py replay <fixture>` chạy harness Rust với một
projection cục bộ. Hiện chỉ projector hai ván Factory07; transcript mã hoá cũ
không cho phép phục hồi payload đầy đủ hoặc thứ tự xuyên các luồng.
