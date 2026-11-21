//! S1 transport for the installed server. Native service ports remain on loopback.
use crate::{Connection, Endpoint, EndpointAddr, Invite, RelayMode, SecretKey};
pub use iroh::{RelayConfig, RelayUrl, tls::CaTlsConfig};
use iroh::{
    address_lookup::{DnsAddressLookup, PkarrPublisher},
    endpoint::{PortmapperConfig, QuicTransportConfig, RecvStream, SendStream},
};
use iroh_relay::RelayQuicConfig;
use sha2::{Digest, Sha256};
use std::{
    io,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    sync::Arc,
    time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream, UdpSocket},
    task::{JoinHandle, JoinSet},
    time::{Instant, timeout},
};

const ALPN: &[u8] = b"boom-kr/s1/1";
const WAIT: Duration = Duration::from_secs(20);
const LOCAL: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0);
const MAX_SESSIONS: usize = 128;
// Tunnel the existing pinned QUIC transports without relying on a nested MTU.
// Each fragment fits inside one outer datagram, and a missing fragment is loss.
const CHUNK: usize = 960;
const MAX_PACKET: usize = 2 * CHUNK;
const HEADER: usize = 6;
const SLOTS: usize = 32;

#[derive(Default, Clone)]
pub enum Network {
    #[default]
    Public,
    Local,
    Private {
        relay: RelayUrl,
        quic_port: u16,
        ca_der: Vec<u8>,
    },
}

impl Network {
    pub async fn endpoint(
        &self,
        bind: SocketAddrV4,
        server_key: Option<SecretKey>,
    ) -> io::Result<Endpoint> {
        let publish = server_key.is_some();
        let mode = match self {
            Self::Public => RelayMode::Default,
            Self::Local => RelayMode::Disabled,
            Self::Private {
                relay, quic_port, ..
            } => RelayMode::Custom(
                RelayConfig::new(relay.clone(), Some(RelayQuicConfig::new(*quic_port))).into(),
            ),
        };
        let mut builder = crate::endpoint_builder(bind.into(), mode, server_key)?
            .alpns(if publish { vec![ALPN.to_vec()] } else { vec![] })
            .transport_config(
                QuicTransportConfig::builder()
                    .max_concurrent_bidi_streams(if publish { 5u32 } else { 0u32 }.into())
                    .max_concurrent_uni_streams(0u32.into())
                    .stream_receive_window((64u32 * 1024).into())
                    .build(),
            );
        if publish && matches!(self, Self::Public) {
            builder = builder.portmapper_config(PortmapperConfig::default());
        }
        match self {
            Self::Public if publish => {
                // PkarrPublisher defaults to relay-only publication.
                builder = builder.address_lookup(PkarrPublisher::n0_dns());
            }
            Self::Public => {
                builder = builder.address_lookup(DnsAddressLookup::n0_dns());
            }
            Self::Private { ca_der, .. } => {
                if !ca_der.is_empty() {
                    builder =
                        builder.ca_tls_config(CaTlsConfig::custom_roots([ca_der.clone().into()]));
                }
            }
            Self::Local => {}
        }
        builder.bind().await.map_err(io::Error::other)
    }

    fn route(&self, invite: &Invite, hint: Option<SocketAddr>) -> io::Result<EndpointAddr> {
        let Invite::Server { node, .. } = invite else {
            return Err(io::Error::other("expected an S1 invite"));
        };
        let mut addr = EndpointAddr::new(*node);
        if let Some(hint) = hint {
            addr = addr.with_ip_addr(hint);
        }
        if let Self::Private { relay, .. } = self {
            addr = addr.with_relay_url(relay.clone());
        }
        Ok(addr)
    }
}

#[derive(Clone)]
pub struct Services {
    pub prelogin: SocketAddrV4,
    pub relay: SocketAddrV4,
    pub launcher: Option<SocketAddrV4>,
    pub name: String,
    pub certificate: Vec<u8>,
}

impl Services {
    fn validate(&self) -> io::Result<()> {
        if !(1..=8192).contains(&self.certificate.len())
            || !(1..=128).contains(&self.name.len())
            || self.name.chars().any(char::is_control)
            || [Some(self.prelogin), Some(self.relay), self.launcher]
                .into_iter()
                .flatten()
                .any(|addr| !addr.ip().is_loopback() || addr.port() == 0)
        {
            return Err(io::Error::other("invalid S1 loopback services or identity"));
        }
        Ok(())
    }
}

pub async fn serve(endpoint: &Endpoint, services: Services) -> io::Result<()> {
    services.validate()?;
    let services = Arc::new(services);
    let mut sessions = JoinSet::new();
    loop {
        tokio::select! {
            _ = sessions.join_next(), if !sessions.is_empty() => {}
            incoming = endpoint.accept() => {
                let Some(incoming) = incoming else { return Ok(()) };
                if sessions.len() >= MAX_SESSIONS {
                    incoming.refuse();
                    continue;
                }
                let services = services.clone();
                sessions.spawn(async move {
                    if let Ok(Ok(conn)) = timeout(WAIT, incoming).await {
                        let _ = host_session(&conn, &services).await;
                        conn.close(0u32.into(), b"S1 session closed");
                    }
                });
            }
        }
    }
}

async fn connected(addr: SocketAddrV4) -> io::Result<UdpSocket> {
    let socket = UdpSocket::bind(LOCAL).await?;
    socket.connect(addr).await?;
    Ok(socket)
}

async fn host_session(conn: &Connection, services: &Services) -> io::Result<()> {
    let (mut tx, mut rx) = timeout(WAIT, conn.accept_bi())
        .await?
        .map_err(io::Error::other)?;
    if timeout(WAIT, rx.read_to_end(1))
        .await?
        .map_err(io::Error::other)?
        != [0]
    {
        return Err(io::Error::other("invalid S1 session request"));
    }
    let a = connected(services.prelogin).await?;
    let b = connected(services.relay).await?;
    tx.write_u16(services.certificate.len() as u16).await?;
    tx.write_all(&services.certificate).await?;
    tx.write_u8(services.name.len() as u8).await?;
    tx.write_all(services.name.as_bytes()).await?;
    tx.write_u8(u8::from(services.launcher.is_some())).await?;
    tx.finish().map_err(io::Error::other)?;
    tokio::select! {
        result = datagrams(conn, [&a, &b], [Some(services.prelogin.into()), Some(services.relay.into())]) => result,
        result = host_launcher(conn, services.launcher) => result,
    }
}

async fn host_launcher(conn: &Connection, target: Option<SocketAddrV4>) -> io::Result<()> {
    let mut streams = JoinSet::new();
    loop {
        tokio::select! {
            _ = streams.join_next(), if !streams.is_empty() => {}
            stream = conn.accept_bi() => {
                let (tx, rx) = stream.map_err(io::Error::other)?;
                if let Some(target) = target.filter(|_| streams.len() < 4) {
                    streams.spawn(async move {
                        if let Ok(Ok(socket)) = timeout(WAIT, TcpStream::connect(target)).await {
                            let _ = copy_tcp(socket, tx, rx).await;
                        }
                    });
                }
            }
        }
    }
}

async fn copy_tcp(socket: TcpStream, mut tx: SendStream, mut rx: RecvStream) -> io::Result<()> {
    let (mut reader, mut writer) = socket.into_split();
    tokio::try_join!(
        async {
            tokio::io::copy(&mut reader, &mut tx).await?;
            tx.finish().map_err(io::Error::other)?;
            Ok::<_, io::Error>(())
        },
        async {
            tokio::io::copy(&mut rx, &mut writer).await?;
            writer.shutdown().await
        },
    )?;
    Ok(())
}

pub struct ClientSession {
    pub certificate: Vec<u8>,
    pub name: String,
    pub prelogin: SocketAddrV4,
    pub relay: SocketAddrV4,
    pub launcher: Option<SocketAddrV4>,
    conn: Connection,
    task: JoinHandle<()>,
}

impl ClientSession {
    pub async fn open(
        endpoint: &Endpoint,
        network: &Network,
        invite: &Invite,
        hint: Option<SocketAddr>,
    ) -> io::Result<Self> {
        timeout(WAIT, Self::open_inner(endpoint, network, invite, hint)).await?
    }

    async fn open_inner(
        endpoint: &Endpoint,
        network: &Network,
        invite: &Invite,
        hint: Option<SocketAddr>,
    ) -> io::Result<Self> {
        let Invite::Server { tls_sha256, .. } = invite else {
            return Err(io::Error::other("expected an S1 invite"));
        };
        let route = network.route(invite, hint)?;
        // Iroh authenticates route.id via its TLS handshake; the inner service
        // must also present exactly the certificate whose hash is in the invite.
        let conn = endpoint
            .connect(route, ALPN)
            .await
            .map_err(io::Error::other)?;
        let (mut tx, mut rx) = conn.open_bi().await.map_err(io::Error::other)?;
        tx.write_u8(0).await?;
        tx.finish().map_err(io::Error::other)?;
        let size = rx.read_u16().await? as usize;
        if !(1..=8192).contains(&size) {
            return Err(io::Error::other("invalid S1 certificate length"));
        }
        let mut certificate = vec![0; size];
        AsyncReadExt::read_exact(&mut rx, &mut certificate).await?;
        if Sha256::digest(&certificate).as_slice() != tls_sha256 {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "S1 TLS pin mismatch",
            ));
        }
        let size = rx.read_u8().await? as usize;
        if !(1..=128).contains(&size) {
            return Err(io::Error::other("invalid S1 server name length"));
        }
        let mut name = vec![0; size];
        AsyncReadExt::read_exact(&mut rx, &mut name).await?;
        let name = String::from_utf8(name).map_err(io::Error::other)?;
        if name.chars().any(char::is_control) {
            return Err(io::Error::other("invalid S1 server name"));
        }
        let launcher = match rx.read_u8().await? {
            0 => None,
            1 => Some(TcpListener::bind(LOCAL).await?),
            _ => return Err(io::Error::other("invalid S1 service flags")),
        };
        if !rx
            .read_to_end(0)
            .await
            .map_err(io::Error::other)?
            .is_empty()
        {
            return Err(io::Error::other("unexpected S1 response tail"));
        }
        let a = UdpSocket::bind(LOCAL).await?;
        let b = UdpSocket::bind(LOCAL).await?;
        let local = |addr| match addr {
            SocketAddr::V4(addr) => Ok(addr),
            _ => Err(io::Error::other("expected IPv4 loopback")),
        };
        let prelogin = local(a.local_addr()?)?;
        let relay = local(b.local_addr()?)?;
        let launcher_addr = launcher
            .as_ref()
            .map(|s| s.local_addr().and_then(local))
            .transpose()?;
        let link = conn.clone();
        let task = tokio::spawn(async move {
            tokio::select! {
                _ = datagrams(&link, [&a, &b], [None, None]) => {}
                _ = client_launcher(&link, launcher) => {}
            }
            link.close(0u32.into(), b"S1 local bridge closed");
        });
        Ok(Self {
            certificate,
            name,
            prelogin,
            relay,
            launcher: launcher_addr,
            conn,
            task,
        })
    }
}

impl Drop for ClientSession {
    fn drop(&mut self) {
        self.conn.close(0u32.into(), b"S1 client finished");
        self.task.abort();
    }
}

async fn client_launcher(conn: &Connection, listener: Option<TcpListener>) -> io::Result<()> {
    let Some(listener) = listener else {
        return std::future::pending().await;
    };
    let mut streams = JoinSet::new();
    loop {
        tokio::select! {
            _ = streams.join_next(), if !streams.is_empty() => {}
            accepted = listener.accept() => {
                let (socket, _) = accepted?;
                if streams.len() < 4 {
                    let conn = conn.clone();
                    streams.spawn(async move {
                        if let Ok(Ok((tx, rx))) = timeout(WAIT, conn.open_bi()).await {
                            let _ = copy_tcp(socket, tx, rx).await;
                        }
                    });
                }
            }
        }
    }
}

struct PartialPacket {
    seq: u32,
    lane: usize,
    until: Instant,
    parts: [Option<Vec<u8>>; 2],
}

type Assembly = [Option<PartialPacket>; SLOTS];

fn reassemble(stash: &mut Assembly, input: &[u8], now: Instant) -> Option<(usize, Vec<u8>)> {
    if !(HEADER + 1..=HEADER + CHUNK).contains(&input.len()) || input[0] > 1 {
        return None;
    }
    let lane = input[0] as usize;
    let part = input[1] as usize;
    let body = &input[HEADER..];
    if part == 0 {
        return Some((lane, body.to_vec()));
    }
    if !(1..=2).contains(&part) || (part == 1 && body.len() != CHUNK) {
        return None;
    }
    let seq = u32::from_be_bytes(input[2..HEADER].try_into().ok()?);
    let entry = &mut stash[seq as usize % SLOTS];
    let frame = match entry.take() {
        Some(frame) if frame.seq == seq && frame.lane == lane && now <= frame.until => frame,
        _ => PartialPacket {
            seq,
            lane,
            until: now + Duration::from_secs(1),
            parts: [None, None],
        },
    };
    let frame = entry.insert(frame);
    frame.parts[part - 1] = Some(body.to_vec());
    if frame.parts.iter().any(Option::is_none) {
        return None;
    }
    let [Some(mut a), Some(b)] = entry.take()?.parts else {
        return None;
    };
    a.extend(b);
    Some((lane, a))
}

fn emit(conn: &Connection, seq: u32, lane: usize, packet: &[u8]) -> io::Result<()> {
    if packet.is_empty() || packet.len() > MAX_PACKET {
        return Ok(());
    }
    for (index, piece) in packet.chunks(CHUNK).enumerate() {
        let mut frame = Vec::with_capacity(HEADER + piece.len());
        frame.push(lane as u8);
        frame.push(if packet.len() <= CHUNK {
            0
        } else {
            1 + index as u8
        });
        frame.extend(seq.to_be_bytes());
        frame.extend(piece);
        conn.send_datagram(frame.into()).map_err(io::Error::other)?;
    }
    Ok(())
}

async fn datagrams(
    conn: &Connection,
    sockets: [&UdpSocket; 2],
    mut peers: [Option<SocketAddr>; 2],
) -> io::Result<()> {
    let connected = peers.map(|peer| peer.is_some());
    let mut stash: Assembly = std::array::from_fn(|_| None);
    let mut seq = 0u32;
    let mut a = [0; MAX_PACKET + 1];
    let mut b = [0; MAX_PACKET + 1];
    loop {
        let (lane, data, recvd) = tokio::select! {
            recvd = sockets[0].recv_from(&mut a) => (0, a.as_slice(), recvd),
            recvd = sockets[1].recv_from(&mut b) => (1, b.as_slice(), recvd),
            frame = conn.read_datagram() => {
                let frame = frame.map_err(io::Error::other)?;
                if let Some((lane, packet)) = reassemble(&mut stash, &frame, Instant::now())
                    && let Some(peer) = peers[lane]
                {
                    if connected[lane] {
                        sockets[lane].send(&packet).await?;
                    } else {
                        sockets[lane].send_to(&packet, peer).await?;
                    }
                }
                continue;
            }
        };
        let (length, peer) = match recvd {
            Ok(received) => received,
            #[cfg(windows)]
            Err(error) if error.raw_os_error() == Some(10040) => continue,
            Err(error) => return Err(error),
        };
        if peer.ip().is_loopback() && *peers[lane].get_or_insert(peer) == peer {
            emit(conn, seq, lane, &data[..length])?;
            seq = seq.wrapping_add(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn s1_checks_the_tls_pin_before_opening_fixed_tcp_and_udp_lanes() {
        let network = Network::Local;
        let key = SecretKey::from_bytes(&[11; 32]);
        let host = network.endpoint(LOCAL, Some(key.clone())).await.unwrap();
        let client = network.endpoint(LOCAL, None).await.unwrap();
        let hint = host.addr().ip_addrs().next().copied().unwrap();
        let prelogin = UdpSocket::bind(LOCAL).await.unwrap();
        let relay = UdpSocket::bind(LOCAL).await.unwrap();
        let launcher = TcpListener::bind(LOCAL).await.unwrap();
        let ipv4 = |addr: SocketAddr| match addr {
            SocketAddr::V4(addr) => addr,
            _ => panic!("expected IPv4"),
        };
        let cert = vec![1, 2, 3];
        let services = Services {
            prelogin: ipv4(prelogin.local_addr().unwrap()),
            relay: ipv4(relay.local_addr().unwrap()),
            launcher: Some(ipv4(launcher.local_addr().unwrap())),
            name: "테스트".into(),
            certificate: cert.clone(),
        };
        let h = host.clone();
        let _serve = tokio::spawn(async move { serve(&h, services).await });
        let test = async {
            let wrong = Invite::server(key.public(), &[4]);
            assert!(matches!(
                ClientSession::open(&client, &network, &wrong, Some(hint)).await,
                Err(err) if err.kind() == io::ErrorKind::PermissionDenied
            ));
            let invite = Invite::server(key.public(), &cert);
            let session = ClientSession::open(&client, &network, &invite, Some(hint))
                .await
                .unwrap();
            assert_eq!(session.name, "테스트");
            let first = connected(session.prelogin).await.unwrap();
            let second = connected(session.relay).await.unwrap();
            let payload = vec![23; 1452];
            let mut buf = [0; MAX_PACKET];
            for (local, server, data) in [
                (&first, &prelogin, payload.as_slice()),
                (&second, &relay, b"relay"),
            ] {
                local.send(data).await.unwrap();
                let (count, remote) = server.recv_from(&mut buf).await.unwrap();
                assert_eq!(&buf[..count], data);
                server.send_to(data, remote).await.unwrap();
                let count = local.recv(&mut buf).await.unwrap();
                assert_eq!(&buf[..count], data);
            }
            let mut tunnel = TcpStream::connect(session.launcher.unwrap()).await.unwrap();
            tunnel.write_all(b"request").await.unwrap();
            tunnel.shutdown().await.unwrap();
            let (mut backend, _) = launcher.accept().await.unwrap();
            let mut request = Vec::new();
            backend.read_to_end(&mut request).await.unwrap();
            assert_eq!(request, b"request");
            backend.write_all(b"response").await.unwrap();
            backend.shutdown().await.unwrap();
            let mut response = Vec::new();
            tunnel.read_to_end(&mut response).await.unwrap();
            assert_eq!(response, b"response");
        };
        timeout(Duration::from_secs(10), test).await.unwrap();
        host.close().await;
        client.close().await;
    }
}
