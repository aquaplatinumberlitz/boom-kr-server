//! Linux namespace driver. Its only game endpoint is a loopback UDP echo.
use iroh_relay::tls::CaTlsConfig;
use iroh_relay::{
    RelayConfig, RelayQuicConfig,
    server::{CertConfig, QuicConfig, RelayConfig as ServerRelay, Server, ServerConfig, TlsConfig},
};
use kr_p2p::{Connection, EndpointAddr, EndpointId, Invite, MAX_GAME_PACKET, RelayMode};
use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer};
use std::{error::Error, future::pending, io, net::SocketAddr, path::Path, time::Duration};
use tokio::{
    net::UdpSocket,
    time::{self, Instant},
};

const RELAY_IP: &str = "198.18.0.1";
const RELAY_TCP: u16 = 8443;
const RELAY_UDP: u16 = 7842;
const PACKETS: usize = 1200;
const SIZES: [usize; 6] = [24, 64, 128, 256, 512, 1100];
type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [role, dir] if role == "relay" => relay(Path::new(dir)).await,
        [role, dir, path] if matches!(role.as_str(), "host" | "guest") => {
            peer(role, Path::new(dir), path).await
        }
        _ => Err(io::Error::other(
            "usage: udp_lab relay DIR | udp_lab (host|guest) DIR (direct|relay)",
        )
        .into()),
    }
}

async fn relay(dir: &Path) -> Result<()> {
    let pair = rcgen::generate_simple_self_signed(vec![RELAY_IP.to_owned()])?;
    let cert = pair.cert.der().clone();
    let key = PrivatePkcs8KeyDer::from(pair.signing_key.serialize_der());
    let crypto = rustls::ServerConfig::builder_with_provider(
        rustls::crypto::ring::default_provider().into(),
    )
    .with_safe_default_protocol_versions()?
    .with_no_client_auth()
    .with_single_cert(vec![cert.clone()], key.into())?;
    let mut relay = ServerRelay::new(format!("{RELAY_IP}:8080").parse::<SocketAddr>()?);
    relay.tls = Some(TlsConfig::new(
        format!("{RELAY_IP}:{RELAY_TCP}").parse::<SocketAddr>()?,
        CertConfig::Manual {
            server_config: crypto,
        },
    ));
    let mut config = ServerConfig::default();
    config.relay = Some(relay);
    config.quic = Some(QuicConfig::new(
        format!("{RELAY_IP}:{RELAY_UDP}").parse::<SocketAddr>()?,
    ));
    let _server = Server::spawn(config).await?;
    tokio::fs::write(dir.join("relay.der"), cert).await?;
    println!("relay-ready");
    pending().await
}

async fn peer(role: &str, dir: &Path, wanted: &str) -> Result<()> {
    let relay_url = format!("https://{RELAY_IP}:{RELAY_TCP}").parse()?;
    let relay = RelayConfig::new(relay_url, Some(RelayQuicConfig::new(RELAY_UDP)));
    let der = tokio::fs::read(dir.join("relay.der")).await?;
    let endpoint = kr_p2p::endpoint_builder(
        "0.0.0.0:41600".parse()?,
        RelayMode::Custom(relay.clone().into()),
        None,
    )?
    .ca_tls_config(CaTlsConfig::custom_roots([CertificateDer::from(der)]))
    .bind()
    .await?;
    time::timeout(Duration::from_secs(12), endpoint.online()).await?;
    if role == "host" {
        // Simulate handing the host-generated code to the guest out of band.
        tokio::fs::write(
            dir.join("room.code"),
            Invite::room(endpoint.id()).to_string(),
        )
        .await?;
    }
    tokio::fs::write(dir.join(format!("{role}.tmp")), endpoint.id().to_string()).await?;
    tokio::fs::rename(
        dir.join(format!("{role}.tmp")),
        dir.join(format!("{role}.id")),
    )
    .await?;
    let (app_udp, bridge_udp) = udp_pair().await?;

    if role == "host" {
        let guest = peer_id(dir, "guest").await?;
        let connection =
            time::timeout(Duration::from_secs(20), kr_p2p::accept(&endpoint, guest)).await??;
        tokio::select! {
            result = echo(app_udp) => result?,
            result = kr_p2p::bridge(&connection, bridge_udp) => {
                if connection.close_reason().is_none() {
                    result?;
                }
            },
        }
    } else {
        let host = peer_id(dir, "host").await?;
        let code: Invite = tokio::fs::read_to_string(dir.join("room.code"))
            .await?
            .parse()?;
        // Bootstrap exclusively via relay; no preconfigured peer UDP address.
        let route = EndpointAddr::new(host).with_relay_url(relay.url.clone());
        let connection = time::timeout(
            Duration::from_secs(15),
            kr_p2p::connect(&endpoint, &code, route),
        )
        .await??;
        let bridge_conn = connection.clone();
        let bridge = tokio::spawn(async move { kr_p2p::bridge(&bridge_conn, bridge_udp).await });
        let measurement = measure(&connection, app_udp, wanted).await;
        endpoint.close().await;
        bridge.abort();
        measurement?;
    }
    Ok(())
}

async fn peer_id(dir: &Path, peer: &str) -> Result<EndpointId> {
    let path = dir.join(format!("{peer}.id"));
    for _ in 0..100 {
        match tokio::fs::read_to_string(&path).await {
            Ok(data) => return Ok(data.parse()?),
            Err(err) if err.kind() == io::ErrorKind::NotFound => {
                time::sleep(Duration::from_millis(100)).await
            }
            Err(err) => return Err(err.into()),
        }
    }
    Err(io::Error::other("peer registration deadline").into())
}

async fn udp_pair() -> io::Result<(UdpSocket, UdpSocket)> {
    let a = UdpSocket::bind("127.0.0.1:0").await?;
    let b = UdpSocket::bind("127.0.0.1:0").await?;
    a.connect(b.local_addr()?).await?;
    b.connect(a.local_addr()?).await?;
    Ok((a, b))
}

async fn echo(udp: UdpSocket) -> io::Result<()> {
    let mut bytes = [0; MAX_GAME_PACKET];
    loop {
        let size = udp.recv(&mut bytes).await?;
        udp.send(&bytes[..size]).await?;
    }
}

fn selected(conn: &Connection) -> Option<&'static str> {
    conn.paths()
        .iter()
        .find(|path| path.is_selected())
        .map(|path| {
            if path.is_relay() {
                "relay"
            } else if path.is_ip() {
                "direct"
            } else {
                "other"
            }
        })
}

fn frame_counts(conn: &Connection) -> [[u64; 2]; 2] {
    // Rows: direct, relay. Columns: sent, received.
    let mut counts = [[0; 2]; 2];
    for path in conn.paths().iter() {
        let class = if path.is_relay() {
            1
        } else if path.is_ip() {
            0
        } else {
            continue;
        };
        let stats = path.stats();
        counts[class][0] += stats.frame_tx.datagram;
        counts[class][1] += stats.frame_rx.datagram;
    }
    counts
}

async fn measure(conn: &Connection, udp: UdpSocket, wanted: &str) -> Result<()> {
    println!("initial_path={:?}", selected(conn));
    let mut beat = time::interval(Duration::from_micros(16_667));
    beat.set_missed_tick_behavior(time::MissedTickBehavior::Delay);
    // Give discovery time to try direct paths in both conditions.
    time::sleep(Duration::from_secs(8)).await;
    if selected(conn) != Some(wanted) {
        return Err(io::Error::other(format!(
            "wanted {wanted}; selected {:?}; paths {:?}",
            selected(conn),
            conn.paths()
        ))
        .into());
    }
    let before = frame_counts(conn);
    let mut tx = Vec::with_capacity(PACKETS);
    let mut rtt = vec![None; PACKETS];
    let mut buf = [0; MAX_GAME_PACKET];
    let mut last = None;
    let (mut received, mut out_of_order, mut duplicate, mut wrong_path) = (0, 0, 0, 0);
    let deadline = time::sleep(Duration::from_secs(26));
    tokio::pin!(deadline);
    loop {
        tokio::select! {
            _ = beat.tick(), if tx.len() < PACKETS => {
                let seq = tx.len();
                let mut data = [0x5a; MAX_GAME_PACKET];
                data[..4].copy_from_slice(&(seq as u32).to_be_bytes());
                wrong_path += usize::from(selected(conn) != Some(wanted));
                tx.push(Instant::now());
                udp.send(&data[..SIZES[seq % SIZES.len()]]).await?;
                if tx.len() == PACKETS {
                    deadline.as_mut().reset(Instant::now() + Duration::from_secs(1));
                }
            },
            incoming = udp.recv(&mut buf) => {
                let size = incoming?;
                let now = Instant::now();
                if size < 4 {
                    return Err(io::Error::other("truncated UDP response").into());
                }
                let seq = u32::from_be_bytes(buf[..4].try_into()?) as usize;
                if seq >= tx.len() || size != SIZES[seq % SIZES.len()] || buf[4..size].iter().any(|&b| b != 0x5a) {
                    return Err(io::Error::other("wrong UDP data").into());
                }
                if rtt[seq].is_some() {
                    duplicate += 1;
                } else {
                    rtt[seq] = Some(now - tx[seq]);
                    received += 1;
                    if last.is_some_and(|high| seq < high) {
                        out_of_order += 1;
                    } else {
                        last = Some(seq);
                    }
                }
                if received == PACKETS { break; }
            },
            _ = &mut deadline => break,
        }
    }
    let after = frame_counts(conn);
    let [direct_frames, relay_frames] = std::array::from_fn(|i| {
        std::array::from_fn::<_, 2, _>(|j| after[i][j].saturating_sub(before[i][j]))
    });
    let mut longest_missing = 0;
    let mut missing = 0;
    for sample in &rtt {
        missing = if sample.is_none() { missing + 1 } else { 0 };
        longest_missing = longest_missing.max(missing);
    }
    let mut samples: Vec<_> = rtt.into_iter().flatten().collect();
    samples.sort_unstable();
    println!(
        "path={wanted} sent={} received={received} missing_burst={longest_missing} reordered={out_of_order} duplicate={duplicate} wrong_path={wrong_path} direct_tx_rx={direct_frames:?} relay_tx_rx={relay_frames:?}",
        tx.len()
    );
    if !samples.is_empty() {
        let ms =
            |pct: usize| samples[(samples.len() * pct).div_ceil(100) - 1].as_secs_f64() * 1000.0;
        println!(
            "rtt_ms p50={:.2} p95={:.2} p99={:.2} max={:.2}",
            ms(50),
            ms(95),
            ms(99),
            ms(100)
        );
    }
    let (used, unused) = if wanted == "direct" {
        (direct_frames, relay_frames)
    } else {
        (relay_frames, direct_frames)
    };
    let frames_ok = used[0] == PACKETS as u64 && used[1] >= received as u64 && unused == [0; 2];
    if tx.len() != PACKETS
        || received < PACKETS * 9 / 10
        || duplicate != 0
        || wrong_path != 0
        || !frames_ok
    {
        return Err(io::Error::other("UDP path or delivery threshold failed").into());
    }
    Ok(())
}
