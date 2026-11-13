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
pub mod proxy;

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
        let mut cfg = ServerConfig::with_single_cert(vec![self.certificate], self.key.into())
            .map_err(io::Error::other)?;
        cfg.transport_config(transport());
        Endpoint::server(cfg, address)
    }
}
pub fn client(address: SocketAddr, certificate: CertificateDer<'static>) -> io::Result<Endpoint> {
    let mut roots = rustls::RootCertStore::empty();
    roots.add(certificate).map_err(io::Error::other)?;
    let mut cfg =
        ClientConfig::with_root_certificates(Arc::new(roots)).map_err(io::Error::other)?;
    cfg.transport_config(transport());
    let mut endpoint = Endpoint::client(address)?;
    endpoint.set_default_client_config(cfg);
    Ok(endpoint)
}

/// Authenticated connection plus one ordered control stream. Data may arrive
/// before authentication at QUIC level, but must never be read by a room owner
/// until this type exists. No 0-RTT acceptance or unpinned certificate path.
pub struct Link {
    connection: Connection,
    pub control_send: SendStream,
    pub control_recv: RecvStream,
}
impl Link {
    pub fn data(&self) -> Data {
        Data(self.connection.clone())
    }
    pub async fn connect(
        endpoint: &Endpoint,
        address: SocketAddr,
        secret: &Secret,
    ) -> io::Result<Self> {
        tokio::time::timeout(Duration::from_secs(5), async {
            let connection = endpoint
                .connect(address, NAME)
                .map_err(io::Error::other)?
                .await
                .map_err(io::Error::other)?;
            let (mut send, mut recv) = connection.open_bi().await.map_err(io::Error::other)?;
            send.write_all(AUTH_TAG).await.map_err(io::Error::other)?;
            send.write_all(secret.expose())
                .await
                .map_err(io::Error::other)?;
            let mut accepted = [0; 1];
            recv.read_exact(&mut accepted)
                .await
                .map_err(io::Error::other)?;
            if accepted != [1] {
                connection.close(1_u8.into(), b"denied");
                return Err(invalid("peer authorization denied"));
            }
            Ok(Self {
                connection,
                control_send: send,
                control_recv: recv,
            })
        })
        .await
        .map_err(|_| invalid("peer connect deadline"))?
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
            let owner =
                authorize(&candidate).ok_or_else(|| invalid("peer authorization denied"))?;
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
                    control_send: send,
                    control_recv: recv,
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
pub struct Data(Connection);
impl Data {
    pub fn send(&self, packet: &Packet) -> io::Result<()> {
        self.0
            .send_datagram(packet.encode()?.into())
            .map_err(io::Error::other)
    }
    pub async fn receive(&self) -> io::Result<Packet> {
        let bytes = self.0.read_datagram().await.map_err(io::Error::other)?;
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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
mod tests;
