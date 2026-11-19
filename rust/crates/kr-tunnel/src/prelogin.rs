//! Server-authenticated TLS13 before native credentials exist. This transport
//! grants no player authority: the server's native DB owner authenticates later.
use quinn::{
    ClientConfig, Connection, Endpoint, RecvStream, SendStream, ServerConfig, TransportConfig,
};
use rustls::{
    RootCertStore,
    pki_types::{CertificateDer, PrivatePkcs8KeyDer},
};
use std::{
    io,
    net::SocketAddr,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::watch,
    task::JoinSet,
};
use zeroize::Zeroizing;

const ALPN: &[u8] = b"kre-native/1";
const NAME: &str = "kr-launch.local";
const OPEN: &[u8; 8] = b"KRNAT001";
pub const STREAM_LIMIT: u32 = 4;
pub const STREAM_TOTAL: usize = 32;
pub const BYTE_LIMIT: usize = 32 * 1024;

/// Login uses a lifetime allowance. Only a verified live owner may switch to
/// the separate one-second allowance; active traffic cannot reset pre-login.
pub struct TrafficBudget {
    initial: usize,
    per_second: usize,
    spent: usize,
    active: Option<(Instant, usize)>,
}
impl TrafficBudget {
    pub const fn new(initial: usize, per_second: usize) -> Self {
        Self {
            initial,
            per_second,
            spent: 0,
            active: None,
        }
    }
    pub fn allow(&mut self, units: usize, active: bool, now: Instant) -> bool {
        let (spent, limit) = if active {
            let (from, count) = self.active.get_or_insert((now, 0));
            if now.saturating_duration_since(*from) >= Duration::from_secs(1) {
                *from = now;
                *count = 0;
            }
            (count, self.per_second)
        } else {
            (&mut self.spent, self.initial)
        };
        match spent.checked_add(units) {
            Some(next) if next <= limit => {
                *spent = next;
                true
            }
            _ => false,
        }
    }
}
struct Close(Connection);
impl Drop for Close {
    fn drop(&mut self) {
        self.0.close(0u8.into(), b"companion owner ended");
    }
}

fn transport() -> Arc<TransportConfig> {
    let mut cfg = TransportConfig::default();
    cfg.max_concurrent_bidi_streams(STREAM_LIMIT.into());
    cfg.max_concurrent_uni_streams(1u8.into());
    cfg.stream_receive_window((8 * 1024u32).into());
    cfg.receive_window((32 * 1024u32).into());
    cfg.send_window(32 * 1024);
    cfg.datagram_receive_buffer_size(None);
    cfg.datagram_send_buffer_size(0);
    cfg.max_idle_timeout(Some(
        Duration::from_secs(15)
            .try_into()
            .expect("fixed idle timeout"),
    ));
    cfg.keep_alive_interval(Some(Duration::from_secs(5)));
    Arc::new(cfg)
}
pub fn server_config(certificate: Vec<u8>, key: Vec<u8>) -> io::Result<ServerConfig> {
    let mut tls = rustls::ServerConfig::builder_with_protocol_versions(&[&rustls::version::TLS13])
        .with_no_client_auth()
        .with_single_cert(
            vec![CertificateDer::from(certificate)],
            PrivatePkcs8KeyDer::from(key).into(),
        )
        .map_err(|_| io::Error::other("native TLS identity"))?;
    tls.alpn_protocols = vec![ALPN.to_vec()];
    tls.max_early_data_size = 0;
    let crypto =
        quinn::crypto::rustls::QuicServerConfig::try_from(tls).map_err(io::Error::other)?;
    let mut server = ServerConfig::with_crypto(Arc::new(crypto));
    server.transport_config(transport());
    // Bound Initial packets before the service can acquire its task/peer
    // permits; application admission alone does not cap Quinn's pending queue.
    server.max_incoming(16);
    server.incoming_buffer_size(8 * 1024);
    server.incoming_buffer_size_total(128 * 1024);
    Ok(server)
}
pub fn client_config(certificate: Vec<u8>) -> io::Result<ClientConfig> {
    if certificate.is_empty() || certificate.len() > 8192 {
        return Err(io::Error::other("native TLS pin bound"));
    }
    let mut roots = RootCertStore::empty();
    roots
        .add(CertificateDer::from(certificate))
        .map_err(io::Error::other)?;
    let mut tls = rustls::ClientConfig::builder_with_protocol_versions(&[&rustls::version::TLS13])
        .with_root_certificates(roots)
        .with_no_client_auth();
    tls.alpn_protocols = vec![ALPN.to_vec()];
    tls.enable_early_data = false;
    let crypto =
        quinn::crypto::rustls::QuicClientConfig::try_from(tls).map_err(io::Error::other)?;
    let mut client = ClientConfig::new(Arc::new(crypto));
    client.transport_config(transport());
    Ok(client)
}
/// No unpinned fallback, resumption API or 0-RTT path is exposed.
pub async fn connect(endpoint: &Endpoint, address: SocketAddr) -> io::Result<Connection> {
    tokio::time::timeout(Duration::from_secs(3), async {
        endpoint
            .connect(address, NAME)
            .map_err(io::Error::other)?
            .await
            .map_err(io::Error::other)
    })
    .await
    .map_err(|_| io::Error::other("native TLS handshake deadline"))?
}
pub async fn open(connection: &Connection, port: u16) -> io::Result<(SendStream, RecvStream)> {
    tokio::time::timeout(Duration::from_secs(2), async {
        let (mut send, recv) = connection.open_bi().await.map_err(io::Error::other)?;
        send.write_all(OPEN).await.map_err(io::Error::other)?;
        send.write_all(&port.to_be_bytes())
            .await
            .map_err(io::Error::other)?;
        Ok((send, recv))
    })
    .await
    .map_err(|_| io::Error::other("native stream open deadline"))?
}
/// Port is an allowlisted protocol endpoint selector, never a server address,
/// account ID, native echo, source address or transport ownership capability.
pub async fn accept_header(recv: &mut RecvStream, ports: &[u16]) -> io::Result<u16> {
    tokio::time::timeout(Duration::from_secs(2), async {
        let mut header = [0; 10];
        recv.read_exact(&mut header)
            .await
            .map_err(io::Error::other)?;
        let port = u16::from_be_bytes([header[8], header[9]]);
        if &header[..8] != OPEN || !ports.contains(&port) {
            return Err(io::Error::other("native endpoint denied"));
        }
        Ok(port)
    })
    .await
    .map_err(|_| io::Error::other("native stream header deadline"))?
}

async fn copy_secret<R: tokio::io::AsyncRead + Unpin, W: tokio::io::AsyncWrite + Unpin>(
    reader: &mut R,
    writer: &mut W,
    bound: &AtomicBool,
) -> io::Result<()> {
    let mut buffer = Zeroizing::new(vec![0; 1024]);
    let mut budget = TrafficBudget::new(BYTE_LIMIT, 128 * 1024);
    loop {
        let size = reader.read(&mut buffer).await?;
        if size == 0 {
            writer.shutdown().await?;
            return Ok(());
        }
        if !budget.allow(size, bound.load(Ordering::Acquire), Instant::now()) {
            return Err(io::Error::other("native stream byte bound"));
        }
        writer.write_all(&buffer[..size]).await?;
        buffer.fill(0);
    }
}
/// Bind listeners before reporting companion readiness. These sockets are
/// loopback only; no direct native connection to a public password port exists.
pub async fn bind(ports: &[u16]) -> io::Result<Vec<TcpListener>> {
    if ports.is_empty() || ports.len() > 32 || ports.contains(&0) {
        return Err(io::Error::other("native local endpoint bounds"));
    }
    let mut listeners = Vec::with_capacity(ports.len());
    for port in ports {
        listeners.push(TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, *port)).await?);
    }
    Ok(listeners)
}
/// Application close sent only when the server's pre-login timer expires.
pub const PRELOGIN_EXPIRED: u32 = 1;

fn expired(error: quinn::ConnectionError) -> bool {
    matches!(error, quinn::ConnectionError::ApplicationClosed(close)
        if close.error_code == PRELOGIN_EXPIRED.into())
}

pub fn endpoint(certificate: Vec<u8>, backend: crate::UdpBackend) -> io::Result<Endpoint> {
    let mut endpoint = crate::client_configured(
        "0.0.0.0:0".parse().expect("fixed bind"),
        certificate.clone().into(),
        backend,
        false,
    )?
    .0;
    endpoint.set_default_client_config(client_config(certificate)?);
    Ok(endpoint)
}

/// Listeners outlive a pre-login timeout. Dial on demand, without replaying
/// any bytes from an old socket or reattaching an authenticated roster.
pub async fn companion_on_demand(
    endpoint: Endpoint,
    address: SocketAddr,
    listeners: Vec<TcpListener>,
    stop: watch::Receiver<bool>,
    backend: crate::UdpBackend,
) -> io::Result<()> {
    run_companion(None, Some((endpoint, address)), listeners, stop, backend).await
}

#[cfg(test)]
async fn companion(
    connection: Connection,
    listeners: Vec<TcpListener>,
    stop: watch::Receiver<bool>,
) -> io::Result<()> {
    run_companion(
        Some(connection),
        None,
        listeners,
        stop,
        crate::UdpBackend::Native,
    )
    .await
}

async fn run_companion(
    mut connection: Option<Connection>,
    dial: Option<(Endpoint, SocketAddr)>,
    listeners: Vec<TcpListener>,
    mut stop: watch::Receiver<bool>,
    backend: crate::UdpBackend,
) -> io::Result<()> {
    let (tx, mut rx) = tokio::sync::mpsc::channel(4);
    let mut accepts = JoinSet::new();
    for listener in listeners {
        let tx = tx.clone();
        accepts.spawn(async move {
            let port = listener.local_addr()?.port();
            loop {
                let (socket, _) = listener.accept().await?;
                let _ = tx.try_send((socket, port));
            }
            #[allow(unreachable_code)]
            Ok::<(), io::Error>(())
        });
    }
    drop(tx);
    let result = async {
        loop {
            if *stop.borrow() { return Ok(()); }
            let initial = if connection.is_none() {
                let ticket = tokio::select! {
                    _ = stop.changed() => return Ok(()),
                    ticket = rx.recv() => ticket.ok_or_else(|| io::Error::other("native listeners ended"))?,
                };
                let (endpoint, address) = dial.as_ref().ok_or_else(|| io::Error::other("native TLS ended"))?;
                let tls = tokio::select! {
                    _ = stop.changed() => return Ok(()),
                    result = connect(endpoint, *address) => result?,
                };
                connection = Some(tls);
                Some(ticket)
            } else {
                None
            };
            let tls = connection.take().expect("connection was established");
            let _close = Close(tls.clone());
            let bound = Arc::new(AtomicBool::new(false));
            let mut streams = JoinSet::new();
            let mut total = 0;
            let peer = peers(tls.clone(), stop.clone(), bound.clone(), backend);
            tokio::pin!(peer);
            let mut initial = initial;
            loop {
                tokio::select! {
                    biased;
                    _ = stop.changed() => return Ok(()),
                    error = tls.closed() => {
                        if dial.is_some() && expired(error) && !bound.load(Ordering::Acquire) {
                            break;
                        }
                        return Err(io::Error::other("native TLS ended"));
                    }
                    result = &mut peer => {
                        if dial.is_some() && !bound.load(Ordering::Acquire)
                            && tls.close_reason().is_some_and(expired) {
                            break;
                        }
                        return result;
                    }
                    result = streams.join_next(), if !streams.is_empty() => {
                        if let Some(Err(error)) = result { return Err(io::Error::other(error)); }
                    }
                    accepted = async {
                        match initial.take() {
                            Some(ticket) => Some(ticket),
                            None => rx.recv().await,
                        }
                    } => {
                        let Some((mut socket, port)) = accepted else {
                            return Err(io::Error::other("native listeners ended"));
                        };
                        total += 1;
                        if total > STREAM_TOTAL || streams.len() >= STREAM_LIMIT as usize {
                            return Err(io::Error::other("native companion stream bound"));
                        }
                        let tls = tls.clone();
                        let bound = bound.clone();
                        streams.spawn(async move {
                            let (mut send, mut recv) = open(&tls, port).await?;
                            let (mut read, mut write) = socket.split();
                            tokio::try_join!(
                                copy_secret(&mut read, &mut send, &bound),
                                copy_secret(&mut recv, &mut write, &bound)
                            ).map(|_| ())
                        });
                    }
                }
            }
            streams.shutdown().await;
        }
    }.await;
    accepts.shutdown().await;
    result
}

/// Server offers are carried only inside the pinned original TLS connection.
/// They delegate the existing DB owner to the existing authenticated UDP hub;
/// no capability enters the native KR stream, argv, disk or logs.
async fn peers(
    connection: Connection,
    stop: watch::Receiver<bool>,
    bound: Arc<AtomicBool>,
    backend: crate::UdpBackend,
) -> io::Result<()> {
    let mut recv = connection.accept_uni().await.map_err(io::Error::other)?;
    let offers = tokio::time::timeout(Duration::from_secs(2), async {
        let count = recv.read_u8().await?;
        if !(1..=2).contains(&count) {
            return Err(io::Error::other("native peer roster bound"));
        }
        let mut offers = Vec::new();
        for _ in 0..count {
            let mut ip = [0; 4];
            recv.read_exact(&mut ip).await.map_err(io::Error::other)?;
            let address = std::net::SocketAddrV4::new(ip.into(), recv.read_u16().await?);
            let size = recv.read_u16().await? as usize;
            if address.port() == 0
                || address.ip().is_unspecified()
                || address.ip().is_multicast()
                || address.ip().is_broadcast()
                || size == 0
                || size > 4096
            {
                return Err(io::Error::other("native peer offer bounds"));
            }
            let mut certificate = vec![0; size];
            recv.read_exact(&mut certificate)
                .await
                .map_err(io::Error::other)?;
            let mut secret = Zeroizing::new([0; 32]);
            recv.read_exact(secret.as_mut())
                .await
                .map_err(io::Error::other)?;
            offers.push((address, certificate, crate::Secret::from_bytes(*secret)));
        }
        Ok::<_, io::Error>(offers)
    })
    .await
    .map_err(|_| io::Error::other("native peer offer deadline"))??;
    // The pinned service sends an offer only after the DB roster has taken
    // ownership of its native channel. This affects copying limits, not auth.
    bound.store(true, Ordering::Release);
    let mut tasks = JoinSet::new();
    for (address, certificate, secret) in offers {
        let stop = stop.clone();
        tasks.spawn(async move {
            let endpoint = crate::client_configured(
                "0.0.0.0:0".parse().expect("fixed bind"),
                certificate.into(),
                backend,
                false,
            )?
            .0;
            let link = crate::Link::connect(&endpoint, address.into(), &secret).await?;
            let result =
                crate::companion::run(link, std::net::Ipv4Addr::LOCALHOST.into(), 8192, stop).await;
            endpoint.close(0u8.into(), b"native peer ended");
            result
        });
    }
    // JoinSet aborts all peer owners on cancellation. A stopped peer makes the
    // whole native transport fail, keeping the 1P/2P roster lifetime atomic.
    let result = tasks
        .join_next()
        .await
        .ok_or_else(|| io::Error::other("native peer roster empty"))?
        .map_err(io::Error::other)?;
    tasks.abort_all();
    while tasks.join_next().await.is_some() {}
    result
}

#[cfg(test)]
mod tests;
