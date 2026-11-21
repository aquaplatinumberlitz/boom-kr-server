//! Experimental O1/S1 transport. Discovery and the native companion are separate.

mod code;

pub use code::{CodeError, Invite, VERSION};
use iroh::endpoint::{Builder, presets};
pub use iroh::{Endpoint, EndpointAddr, EndpointId, RelayMode, SecretKey, endpoint::Connection};
use std::{io, net::SocketAddr};
use tokio::net::UdpSocket;

const ALPN: &[u8] = b"boom-kr/p2p/1";
/// Keep each game packet within one QUIC datagram, including on the relay.
pub const MAX_GAME_PACKET: usize = 1100;

/// No global address publication or UPnP. Discovery supplies routing hints separately.
pub fn endpoint_builder(
    bind: SocketAddr,
    relay: RelayMode,
    key: Option<SecretKey>,
) -> io::Result<Builder> {
    let mut builder = Endpoint::builder(presets::Minimal)
        .clear_ip_transports()
        .bind_addr(bind)
        .map_err(io::Error::other)?
        .relay_mode(relay)
        .alpns(vec![ALPN.to_vec()]);
    if let Some(key) = key {
        builder = builder.secret_key(key);
    }
    Ok(builder)
}

/// Check the routing hint against the invite before dialing. Iroh's handshake
/// then authenticates the full node key. S1 callers still pin the inner TLS.
pub async fn connect(
    endpoint: &Endpoint,
    invite: &Invite,
    route: EndpointAddr,
) -> io::Result<Connection> {
    if !invite.matches_node(route.id) {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "routing hint does not match the invite",
        ));
    }
    endpoint
        .connect(route, ALPN)
        .await
        .map_err(io::Error::other)
}

/// One incoming session with a preapproved peer. The launcher will own admission.
pub async fn accept(endpoint: &Endpoint, allowed: EndpointId) -> io::Result<Connection> {
    let incoming = endpoint.accept().await.ok_or(io::ErrorKind::NotConnected)?;
    let connection = incoming.await.map_err(io::Error::other)?;
    if connection.remote_id() != allowed {
        connection.close(1u32.into(), b"peer denied");
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "peer was not admitted",
        ));
    }
    Ok(connection)
}

/// Carry opaque game UDP packets to and from one fixed loopback service.
/// A packet beyond the protocol bound is dropped whole, not truncated or retried.
pub async fn bridge(connection: &Connection, udp: UdpSocket) -> io::Result<()> {
    if !udp.local_addr()?.ip().is_loopback() || !udp.peer_addr()?.ip().is_loopback() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "game socket must be connected over loopback",
        ));
    }
    let mut buf = [0; MAX_GAME_PACKET + 1];
    loop {
        tokio::select! {
            size = udp.recv(&mut buf) => {
                let size = match size {
                    Ok(size) => size,
                    // Winsock can report WSAEMSGSIZE instead of returning a truncated prefix.
                    #[cfg(windows)]
                    Err(err) if err.raw_os_error() == Some(10040) => continue,
                    Err(err) => return Err(err),
                };
                if size <= MAX_GAME_PACKET {
                    connection.send_datagram(buf[..size].to_vec().into()).map_err(io::Error::other)?;
                }
            }
            packet = connection.read_datagram() => {
                let packet = packet.map_err(io::Error::other)?;
                if packet.len() <= MAX_GAME_PACKET {
                    udp.send(&packet).await?;
                }
            }
        }
    }
}
