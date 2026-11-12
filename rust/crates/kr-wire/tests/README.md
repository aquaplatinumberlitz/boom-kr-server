# Oracle codec KR00967

Hai frame C→S giữ cố định từ CP057: `local/research/codex/pickup-state-r1/state-guard/run.json`,
A/connection4/frame0–1/seq40–43. Plaintext expected từ KR native emulation `native-stream.json/coalesced`,
không từ encoder Rust. Build image native `a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5`.
Nguồn chuẩn/giới hạn: `docs/PROTOCOL.md#login-retry-codec`, `docs/evidence/codec-boundaries/README.md`.
Đây là packet channel diagnostic6/544, không credential/loginpayload, không binary/asset game.
CRC/typed/bounds/stream tests không thay nghiệm thu client KR thật trên Rust; mode/header khác chưa chứng minh.


`native_start_vectors.rs` giữ plaintext Start121 của các input2/4/8 đã được consumer native chấp nhận ở CP274,
`local/research/codex/r1-final-native-r1/{two,four,eight}/native-config.json`, các pin `fixed_room.packets.start`.
SHA-256 body lần lượt `6d0905319f31ac340bc4a1ba1afebd419a67d9d9a0283b9b5dd7b51450158ded`,
`bbee86669c9035e15cb862ae7253c66f5f56e50b67a143f71327a04e76d87012` và
`3ead376d6eb701898361563f18ef8234f59439ef4bb958cc2e7a09a87e9bdc89`.
Đây là vector wire để đối chiếu phép chọn hàng: chúng đã có xác nhận native độc lập ở nguồn cũ, nhưng không phải
bằng chứng client thật đã chạy implementation/roster serving mới. Không có client binary hoặc game asset trong test.
