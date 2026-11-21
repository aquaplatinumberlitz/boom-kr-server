//! Company companion transport, not recovered KR wire. No serving integration
//! or native compatibility claim. QUIC/TLS carries unchanged native datagrams;
//! room/account owners must still authorize every logical route and generation.
use quinn::{
    ClientConfig, Connection, Endpoint, RecvStream, SendStream, ServerConfig, TransportConfig,
};
use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer};
use std::{io, net::SocketAddr, sync::Arc, time::Duration};
use zeroize::Zeroizing;

const NAME: &str = "kr-peer.local";
const AUTH_TAG: &[u8; 8] = b"KRPEER01";
pub const MAX_NATIVE: usize = 255;
pub const MAX_PACKET: usize = 16 + 8 + MAX_NATIVE;
pub mod companion;
pub mod control;
pub mod launch_tls;
mod portable_udp;
pub mod prelogin;
pub mod proxy;
pub mod router;
pub mod udp_diagnostics;

#[derive(Clone, Copy, Default, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UdpBackend {
    #[default]
    Native,
    Portable,
}

fn invalid(message: &'static str) -> io::Error {
    io::Error::other(message)
}
fn transport() -> Arc<TransportConfig> {
    let mut cfg = TransportConfig::default();
    cfg.max_concurrent_bidi_streams(1_u8.into());
    cfg.max_concurrent_uni_streams(0_u8.into());
    cfg.datagram_receive_buffer_size(Some(64 * MAX_PACKET));
    cfg.datagram_send_buffer_size(64 * MAX_PACKET);
    cfg.max_idle_timeout(Some(
        Duration::from_secs(20)
            .try_into()
            .expect("constant timeout"),
    ));
    cfg.keep_alive_interval(Some(Duration::from_secs(5)));
    Arc::new(cfg)
}

/// Local private launch capability; never format, serialize or log this value.
/// Distribution belongs to an authenticated launcher seam, not a native KR field.
pub struct Secret(Zeroizing<[u8; 32]>);
impl Secret {
    pub fn random() -> io::Result<Self> {
        let mut bytes = Zeroizing::new([0; 32]);
        getrandom::fill(bytes.as_mut()).map_err(io::Error::other)?;
        Ok(Self(bytes))
    }
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(Zeroizing::new(bytes))
    }
    pub fn expose(&self) -> &[u8; 32] {
        &self.0
    }
    fn accepts(&self, candidate: &[u8; 32]) -> bool {
        self.0
            .iter()
            .zip(candidate)
            .fold(0_u8, |diff, (a, b)| diff | (a ^ b))
            == 0
    }
}
pub struct Identity {
    pub certificate: CertificateDer<'static>,
    key: PrivatePkcs8KeyDer<'static>,
}
impl Identity {
    /// Ephemeral identity for isolated prototype/tests. Operational certificate
    /// provisioning and pin persistence are separate from this constructor.
    pub fn ephemeral() -> io::Result<Self> {
        let cert =
            rcgen::generate_simple_self_signed(vec![NAME.into()]).map_err(io::Error::other)?;
        Ok(Self {
            certificate: CertificateDer::from(cert.cert),
            key: PrivatePkcs8KeyDer::from(cert.signing_key.serialize_der()),
        })
    }
    pub fn listen(self, address: SocketAddr) -> io::Result<Endpoint> {
        server_endpoint(self.server_config()?, address)
    }
    fn server_config(self) -> io::Result<ServerConfig> {
        let mut cfg = ServerConfig::with_single_cert(vec![self.certificate], self.key.into())
            .map_err(io::Error::other)?;
        cfg.transport_config(transport());
        Ok(cfg)
    }
}

/// Use Quinn's native UDP features where available. Wine and some Winsock
/// stacks do not support the required ancillary socket options. An endpoint
/// bound to a concrete address can use ordinary datagrams with the same TLS
/// and transport configuration. Leave bind/address/permission failures intact.
pub fn server_endpoint(config: ServerConfig, address: SocketAddr) -> io::Result<Endpoint> {
    #[cfg(not(windows))]
    {
        Endpoint::server(config, address)
    }
    #[cfg(windows)]
    match Endpoint::server(config.clone(), address) {
        Err(error)
            if error.kind() == io::ErrorKind::Unsupported
                || matches!(error.raw_os_error(), Some(10042 | 10045)) =>
        {
            portable_server(config, address)
        }
        result => result,
    }
}

#[cfg(any(windows, test))]
fn portable_server(config: ServerConfig, address: SocketAddr) -> io::Result<Endpoint> {
    if address.ip().is_unspecified() {
        return Err(invalid("portable QUIC server requires a concrete bind IP"));
    }
    let runtime = quinn::default_runtime().ok_or_else(|| invalid("QUIC runtime unavailable"))?;
    Endpoint::new_with_abstract_socket(
        quinn::EndpointConfig::default(),
        Some(config),
        portable_udp::Socket::bind(address)?,
        runtime,
    )
}

pub fn client(address: SocketAddr, certificate: CertificateDer<'static>) -> io::Result<Endpoint> {
    let cfg = client_config(certificate)?;
    let mut endpoint = Endpoint::client(address)?;
    endpoint.set_default_client_config(cfg);
    Ok(endpoint)
}
fn client_config(certificate: CertificateDer<'static>) -> io::Result<ClientConfig> {
    let mut roots = rustls::RootCertStore::empty();
    roots.add(certificate).map_err(io::Error::other)?;
    let mut cfg =
        ClientConfig::with_root_certificates(Arc::new(roots)).map_err(io::Error::other)?;
    cfg.transport_config(transport());
    Ok(cfg)
}
pub fn client_observed(
    address: SocketAddr,
    certificate: CertificateDer<'static>,
) -> io::Result<(Endpoint, Arc<udp_diagnostics::Counters>)> {
    let (endpoint, counters) = client_configured(address, certificate, UdpBackend::Native, true)?;
    Ok((endpoint, counters.expect("requested observation")))
}
pub fn client_configured(
    address: SocketAddr,
    certificate: CertificateDer<'static>,
    backend: UdpBackend,
    observe: bool,
) -> io::Result<(Endpoint, Option<Arc<udp_diagnostics::Counters>>)> {
    if backend == UdpBackend::Native && !observe {
        return client(address, certificate).map(|endpoint| (endpoint, None));
    }
    let cfg = client_config(certificate)?;
    let runtime = quinn::default_runtime().ok_or_else(|| invalid("QUIC runtime unavailable"))?;
    let inner = match backend {
        UdpBackend::Native => runtime.wrap_udp_socket(std::net::UdpSocket::bind(address)?)?,
        UdpBackend::Portable => portable_udp::Socket::bind(address)?,
    };
    let (socket, counters): (Arc<dyn quinn::AsyncUdpSocket>, _) = if observe {
        let counters = Arc::new(udp_diagnostics::Counters::default());
        (
            Arc::new(udp_diagnostics::Observed {
                inner,
                counters: counters.clone(),
            }),
            Some(counters),
        )
    } else {
        (inner, None)
    };
    let mut endpoint = Endpoint::new_with_abstract_socket(
        quinn::EndpointConfig::default(),
        None,
        socket,
        runtime,
    )?;
    endpoint.set_default_client_config(cfg);
    Ok((endpoint, counters))
}

/// Authenticated connection plus one ordered control stream. Data may arrive
/// before authentication at QUIC level, but must never be read by a room owner
/// until this type exists. No 0-RTT acceptance or unpinned certificate path.
pub struct Link {
    connection: Connection,
    control: Option<(SendStream, RecvStream)>,
}
impl Link {
    pub fn data(&self) -> Data {
        Data::Quinn(self.connection.clone())
    }
    pub async fn connect(
        endpoint: &Endpoint,
        address: SocketAddr,
        secret: &Secret,
    ) -> io::Result<Self> {
        Self::connect_observed(endpoint, address, secret, |_| {}).await
    }
    /// Fixed local progress labels only; no addresses, capability or peer data.
    /// Observation does not retry or change the authentication deadline.
    pub async fn connect_observed(
        endpoint: &Endpoint,
        address: SocketAddr,
        secret: &Secret,
        mut progress: impl FnMut(&'static str),
    ) -> io::Result<Self> {
        tokio::time::timeout(Duration::from_secs(5), async {
            progress("quic_connect_setup");
            let connection = endpoint.connect(address, NAME).map_err(io::Error::other)?;
            progress("quic_handshake");
            let connection = connection.await.map_err(io::Error::other)?;
            progress("control_open");
            let (mut send, mut recv) = connection.open_bi().await.map_err(io::Error::other)?;
            progress("capability_send");
            send.write_all(AUTH_TAG).await.map_err(io::Error::other)?;
            send.write_all(secret.expose())
                .await
                .map_err(io::Error::other)?;
            let mut accepted = [0; 1];
            progress("authority_reply");
            recv.read_exact(&mut accepted)
                .await
                .map_err(io::Error::other)?;
            if accepted != [1] {
                connection.close(1_u8.into(), b"denied");
                return Err(invalid("peer authorization denied"));
            }
            Ok(Self {
                connection,
                control: Some((send, recv)),
            })
        })
        .await
        .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "peer connect deadline"))?
    }
    /// The service must supply an already scoped one-use/owned capability and
    /// separately enforce connection admission, revoke/expiry and room ownership.
    /// This foundation verifies possession only, not account/route membership.
    pub async fn accept(connection: Connection, expected: &Secret) -> io::Result<Self> {
        Self::accept_with(connection, |candidate| {
            expected.accepts(candidate).then_some(())
        })
        .await
        .map(|(link, ())| link)
    }
    /// Resolve a capability to an owner synchronously under the caller's
    /// registry lock. One-use claims/admission/current-generation checks belong
    /// to that registry. Do not resolve ownership using a public source IP.
    pub async fn accept_with<T>(
        connection: Connection,
        authorize: impl FnOnce(&[u8; 32]) -> Option<T>,
    ) -> io::Result<(Self, T)> {
        Self::accept_with_async(connection, |candidate| {
            std::future::ready(authorize(candidate.expose()))
        })
        .await
    }
    /// Account adapters may await a bounded backend check before the positive
    /// handshake reply. The secret stays zeroized in its owned request scope.
    pub async fn accept_with_async<T, F: std::future::Future<Output = Option<T>>>(
        connection: Connection,
        authorize: impl FnOnce(Secret) -> F,
    ) -> io::Result<(Self, T)> {
        let result = tokio::time::timeout(Duration::from_secs(3), async {
            let (mut send, mut recv) = connection.accept_bi().await.map_err(io::Error::other)?;
            let mut tag = [0; 8];
            let mut candidate = Zeroizing::new([0; 32]);
            recv.read_exact(&mut tag).await.map_err(io::Error::other)?;
            recv.read_exact(candidate.as_mut())
                .await
                .map_err(io::Error::other)?;
            if tag != *AUTH_TAG {
                return Err(invalid("peer authorization denied"));
            }
            let owner = authorize(Secret(candidate))
                .await
                .ok_or_else(|| invalid("peer authorization denied"))?;
            send.write_all(&[1]).await.map_err(io::Error::other)?;
            Ok((send, recv, owner))
        })
        .await
        .map_err(|_| invalid("peer authorization deadline"))
        .and_then(|v| v);
        match result {
            Ok((send, recv, owner)) => Ok((
                Self {
                    connection,
                    control: Some((send, recv)),
                },
                owner,
            )),
            Err(error) => {
                connection.close(1_u8.into(), b"denied");
                Err(error)
            }
        }
    }
    pub fn send(&self, packet: &Packet) -> io::Result<()> {
        self.data().send(packet)
    }
    pub fn take_control(&mut self) -> io::Result<(SendStream, RecvStream)> {
        self.control
            .take()
            .ok_or_else(|| invalid("peer control stream already owned"))
    }
    pub async fn receive(&self) -> io::Result<Packet> {
        self.data().receive().await
    }
    pub fn close(&self) {
        self.connection.close(0_u8.into(), b"closed");
    }
    pub fn remote_address(&self) -> SocketAddr {
        self.connection.remote_address()
    }
}
/// Cloned data handle does not own the authorization lifetime. Dropping Link
/// closes its connection even when data pumps still have a clone.
#[derive(Clone)]
pub enum Data {
    Quinn(Connection),
    Friends(kr_p2p::Connection),
}
impl Data {
    pub fn send(&self, packet: &Packet) -> io::Result<()> {
        let bytes = packet.encode()?.into();
        match self {
            Self::Quinn(c) => c.send_datagram(bytes).map_err(io::Error::other),
            Self::Friends(c) => c.send_datagram(bytes).map_err(io::Error::other),
        }
    }
    pub async fn receive(&self) -> io::Result<Packet> {
        let bytes = match self {
            Self::Quinn(c) => c.read_datagram().await.map_err(io::Error::other)?,
            Self::Friends(c) => c.read_datagram().await.map_err(io::Error::other)?,
        };
        Packet::decode(&bytes)
    }
}
impl Drop for Link {
    fn drop(&mut self) {
        self.close();
    }
}

/// Random current route identifier is scoped by the account/room owner. A
/// packet's identifier is never sufficient to authorize its authenticated sender.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct RouteId(pub [u8; 16]);
impl RouteId {
    pub fn random() -> io::Result<Self> {
        let mut bytes = [0; 16];
        getrandom::fill(&mut bytes).map_err(io::Error::other)?;
        Ok(Self(bytes))
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct Packet {
    pub route: RouteId,
    pub sequence: u64,
    pub native: Vec<u8>,
}
impl Packet {
    pub fn encode(&self) -> io::Result<Vec<u8>> {
        if self.native.is_empty() || self.native.len() > MAX_NATIVE || self.sequence == 0 {
            return Err(invalid("peer packet bound"));
        }
        let mut bytes = Vec::with_capacity(24 + self.native.len());
        bytes.extend(self.route.0);
        bytes.extend(self.sequence.to_le_bytes());
        bytes.extend(&self.native);
        Ok(bytes)
    }
    pub fn decode(bytes: &[u8]) -> io::Result<Self> {
        if !(25..=MAX_PACKET).contains(&bytes.len()) {
            return Err(invalid("peer packet bound"));
        }
        let sequence = u64::from_le_bytes(bytes[16..24].try_into().expect("bounded header"));
        if sequence == 0 {
            return Err(invalid("zero peer sequence"));
        }
        Ok(Self {
            route: RouteId(bytes[..16].try_into().expect("bounded route")),
            sequence,
            native: bytes[24..].to_vec(),
        })
    }
}
/// Bounded replay window per current direction/route. Admit only after ownership
/// and identifier checks; an unknown sender must not advance a valid window.
#[derive(Default, Debug)]
pub struct ReplayWindow {
    high: u64,
    seen: u64,
}
impl ReplayWindow {
    pub fn admit(&mut self, sequence: u64) -> bool {
        if sequence == 0 {
            return false;
        }
        if sequence > self.high {
            let delta = sequence - self.high;
            self.seen = if delta >= 64 {
                1
            } else {
                (self.seen << delta) | 1
            };
            self.high = sequence;
            return true;
        }
        let age = self.high - sequence;
        if age >= 64 || self.seen & (1 << age) != 0 {
            return false;
        }
        self.seen |= 1 << age;
        true
    }
}

#[cfg(test)]
mod companion_tests;
#[cfg(test)]
mod tests;
