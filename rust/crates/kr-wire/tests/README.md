# Oracle codec KR00967

Hai frame C→S giữ cố định từ CP057: `local/research/codex/pickup-state-r1/state-guard/run.json`,
A/connection4/frame0–1/seq40–43. Plaintext expected từ KR native emulation `native-stream.json/coalesced`,
không từ encoder Rust. Build image native `a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5`.
Nguồn chuẩn/giới hạn: `docs/PROTOCOL.md#login-retry-codec`, `docs/evidence/codec-boundaries/README.md`.
Đây là packet channel diagnostic6/544, không credential/loginpayload, không binary/asset game.
CRC/typed/bounds/stream tests không thay nghiệm thu client KR thật trên Rust; mode/header khác chưa chứng minh.
