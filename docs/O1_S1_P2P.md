# O1 / S1 peer transport

**Transport.** Embed iroh 1.5 (Rust, QUIC). O1 does not require UPnP or port forwarding; S1 may try a port mapping for its direct path.
Start via Number 0's public relay and switch to direct UDP when hole punching
works. A package may configure a private iroh relay later, including on a VPS.
The game server remains on the host; LAN play needs neither Internet nor relay.

**Codes.** O1 uses `O1-1-K7M2-4QX8-7RDT-25PH`: mode, version and 16 Crockford
base32 symbols. The body commits to a fresh room's public key using the first
80 bits of domain-separated SHA-256. Six symbols would give only 30 bits for a
public key pin. S1 uses `S1-1-<base64url>`: the 32-byte persistent node ID followed
by the SHA-256 of the DER TLS leaf certificate. An S1 server entry may provide a
separate address or relay hint. Reject unsupported versions before dialing and
show “Download the current package” with the configured Drive link. Version the
ALPN too and negotiate game/package compatibility before login.

**Discovery.** Publish the ephemeral O1 node and local route over mDNS. For
Internet play, use a public rendezvous mailbox (candidate: public Nostr
relays with expiring records). Index by a domain-separated hash of the code;
encrypt the node key, relay URL and expiry using AEAD with a key derived from
the code via HKDF. Check expiry and the code commitment before dialing. Iroh
proves possession of the full private key; the host separately approves the
guest node before enabling game traffic. Validate mailbox availability and
retention before release: iroh relays route by node ID and cannot look up
room codes. S1 can use iroh's signed address lookup with relay-only records.
Keep IPs out of codes and records; exchange direct candidates only on LAN
or over the authenticated tunnel. Direct peers see each other's path IP;
relay and directory operators can see their clients' source IP and metadata.

**Integration.** O1 runs `kr-service` on host loopback with separate guest
state; `kr-companion` exposes local endpoints to the KR client. Reliable
iroh streams forward launcher TCP/TLS to fixed services. Obtain the O1
certificate over the authenticated link; for S1 check the invited hash
before handing the certificate to the existing TLS client. At the edges of
`kr-prelogin` and `kr-tunnel`, adapt reliable control to iroh streams and opaque
game UDP to QUIC DATAGRAMs. Preserve capability, slot, route and epoch
checks; avoid nesting whole QUIC packets in game datagrams. Close sessions
when the host exits. The O1 prototype covers code parsing, identity and a fixed
loopback UDP bridge.

**Phần server S1.** `boom-kr/s1/1` là ALPN riêng cho server tự dựng. Server dùng khóa node bền vững,
công bố riêng địa chỉ relay bằng Pkarr và trả tên máy cùng chứng chỉ qua stream đã được iroh xác thực.
Client kiểm vân tay DER trong mã trước khi mở proxy cục bộ. Tạo lại mã đổi khóa node và đóng các kết nối cũ;
chứng chỉ TLS, cơ sở dữ liệu và tài khoản giữ nguyên. Mã mời làm nhiệm vụ định tuyến và ghim danh tính;
đăng nhập và đăng ký vẫn theo cơ chế tài khoản.

Launcher TCP/TLS đi qua stream. Phần prelogin và companion hiện có giữ nguyên xác thực inner QUIC.
Lát cắt server dùng hai luồng UDP cố định tới loopback; datagram lớn chia tối đa hai mảnh có bộ đệm giới hạn,
không chuyển qua stream và không phát lại mảnh mất ở lớp S1. Cách ghép tương thích này thêm overhead so
với tích hợp trực tiếp vào companion. Client đo `kr-load` kết nối bằng mã S1 và giữ TLS inner; hành vi nút
Thêm máy chủ của launcher thuộc đợt tiếp theo. Xem [đóng gói và vận hành](OPERATIONS.md#self-hosting).

Run `cargo test --manifest-path rust/Cargo.toml --locked -p kr-p2p`.
Build the lab with `cargo build --manifest-path rust/Cargo.toml --locked
-p kr-p2p --features lab --example udp_lab`. Run `tools/o1_p2p_namespace.sh`
with `direct` or `relay`, the built binary and an empty output directory.
See [PLAN](PLAN.md#full-game-roadmap) for acceptance.
