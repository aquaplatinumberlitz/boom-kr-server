# Rust foundation KR00967

Đợt implementation được mở bởi [owner09/11](../docs/evidence/owner-directives/rust-implementation.json).
[PLAN](../docs/PLAN.md#current-order) giữ thứ tự; [contract/limits](../docs/PROTOCOL.md#rust-foundation)
và [evidence](../docs/evidence/rust-service/README.md#cp231) giữ kết luận có nguồn.

`kr-wire` không có dependency: profile mode2/selector3, header37 giữ opaque, crypt/CRC, typed unsigned fields,
framing tối đa64KiB và seq theo connection. Giới hạn64KiB là design; không phải maximum của client gốc.
`kr-service` là binary Tokio/Linux cho **diagnostic hello → login error → retry**, chưa account authentication,
đăng nhập thành công, sảnh/phòng hoặc gameplay. Hai transport đầu dùng mapping WCP của recipe CP181;
mọi login ở profile này nhận lỗi1. Không dùng arrival order làm thiết kế identity production.

Build/kiểm từ root repo:

```bash
cargo fmt --manifest-path rust/Cargo.toml --all --check
cargo clippy --manifest-path rust/Cargo.toml --workspace --all-targets --locked -- -D warnings
cargo test --manifest-path rust/Cargo.toml --workspace --locked
cargo build --manifest-path rust/Cargo.toml --workspace --locked
```

Rust1.99.0 và lockfile được pin. Tokio cho I/O, semaphore/queue/timeout/signal; serde/JSON cho config và
structured logs; SHA-256 kiểm input fixture trước bind. Không import server/clientengine từ boom-rs/VN.
Binary không kèm client/asset/binary fixture. Từ fixture Python đã giữ cục bộ, export **path/size/hash**, không sinh wire:

```bash
python3 tools/export_rust_profile.py --python-profile local/profile.json \
  --login-error local/login-error1-key43.bin --port 3838 --port 3848 --out local/rust-profile.json
rust/target/debug/kr-service --config local/rust-profile.json --out local/new-rust-run
```

Recipe native cô lập ở evidence dùng port3808..3998/step10 theo CP181; config chỉ cho loopback/allowlist.
Giữ source/binary/config/input frozen từ trước run tới seal; không hot-swap backend. Rollback chọn Python
baseline đã pin trước run mới sau cleanup. Python vẫn serving/RE/reference; không rút khỏi đường game hiện tại.

Giới hạn: admission tối đa64connection, acceptqueue32, logqueue256, buffer64KiB/socket, write/read/log timeout,
shutdown SIGTERM/SIGINT hữu hạn; không unbounded per-packet tasks. Ghi opcode/seq/size, không raw credential/token
hoặc hash auth body. Request60/body20 và15/body31 sau rejection chỉ được ignore có budget64 theo traceCP181,
không khẳng định ý nghĩa. Malformed/CRC/unknownstage/duplicate login không thay seq hoặc tạo success giả.
Chưa load/SLO/DB/UDP/Internet; **Internet thật chưa kiểm**.
