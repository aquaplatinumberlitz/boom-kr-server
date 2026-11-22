//! Adapt native control streams and authenticated game packets at the edges.
//! Only the host gateway opens a separate pinned, loopback Quinn connection.
use super::{STREAM_LIMIT, STREAM_TOTAL, copy_secret};
use crate::{Data, Link, UdpBackend};
use kr_p2p::{Connection, Endpoint, Invite, room};
use std::{
    io,
    net::{Ipv4Addr, SocketAddrV4},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::{Semaphore, watch},
    task::JoinSet,
};

const GUEST: &[u8; 8] = b"KR-O1-01";
const PEER: &[u8; 8] = b"KR-O1-UD";

pub fn valid_name(name: &str) -> bool {
    kr_wire::kr00967::text949x::encode_name(name).is_ok()
}

pub async fn read_name(connection: &quinn::Connection) -> io::Result<String> {
    tokio::time::timeout(Duration::from_secs(3), async {
        let mut stream = connection.accept_uni().await.map_err(io::Error::other)?;
        let bytes = stream.read_to_end(8 + 45).await.map_err(io::Error::other)?;
        let name = bytes
            .strip_prefix(GUEST)
            .and_then(|value| std::str::from_utf8(value).ok())
            .filter(|name| valid_name(name))
            .ok_or_else(|| io::Error::other("O1 guest metadata"))?;
        Ok(name.into())
    })
    .await
    .map_err(|_| io::Error::other("O1 guest metadata deadline"))?
}

#[derive(Clone)]
pub struct Gateway {
    pub server: SocketAddrV4,
    pub certificate: Vec<u8>,
    pub ports: Vec<u16>,
    pub package: [u8; 32],
    pub backend: UdpBackend,
}

pub async fn serve(endpoint: Endpoint, gateway: Gateway) -> io::Result<()> {
    let local = super::endpoint(gateway.certificate, gateway.backend)?;
    let quota = Arc::new(Semaphore::new(8));
    let mut guests = JoinSet::new();
    loop {
        tokio::select! {
            _ = guests.join_next(), if !guests.is_empty() => {},
            incoming = endpoint.accept() => {
                let Some(incoming) = incoming else { return Ok(()); };
                let Ok(permit) = quota.clone().try_acquire_owned() else { incoming.refuse(); continue; };
                let (local, ports, node) = (local.clone(), gateway.ports.clone(), endpoint.id());
                guests.spawn(async move {
                    let _permit = permit;
                    let Ok(Ok(connection)) = tokio::time::timeout(Duration::from_secs(5), incoming).await else { return; };
                    if let Ok(name) = room::admit(&connection, node, gateway.package, valid_name).await {
                        let _ = host(&connection, local, gateway.server, ports, gateway.backend, name).await;
                    }
                    connection.close(0u32.into(), b"room connection ended");
                });
            }
        }
    }
}

async fn host(
    room: &Connection,
    endpoint: quinn::Endpoint,
    address: SocketAddrV4,
    ports: Vec<u16>,
    backend: UdpBackend,
    name: String,
) -> io::Result<()> {
    let native = super::connect(&endpoint, address.into()).await?;
    let _owner = super::Close(native.clone());
    let mut metadata = native.open_uni().await.map_err(io::Error::other)?;
    metadata.write_all(GUEST).await.map_err(io::Error::other)?;
    metadata
        .write_all(name.as_bytes())
        .await
        .map_err(io::Error::other)?;
    metadata.finish().map_err(io::Error::other)?;
    let bound = Arc::new(AtomicBool::new(false));
    let data = host_peer(room, &native, &bound, backend);
    tokio::pin!(data);
    let mut streams = JoinSet::new();
    let mut total = 0;
    loop {
        tokio::select! {
            _ = room.closed() => return Ok(()),
            _ = native.closed() => return Ok(()),
            result = &mut data => return result,
            done = streams.join_next(), if !streams.is_empty() => {
                if let Some(Err(error)) = done { return Err(io::Error::other(error)); }
            },
            stream = room.accept_bi() => {
                let (mut out, mut input) = stream.map_err(io::Error::other)?;
                total += 1;
                if total > STREAM_TOTAL || streams.len() >= STREAM_LIMIT as usize {
                    return Err(io::Error::other("O1 stream bound"));
                }
                let (native, ports, bound) = (native.clone(), ports.clone(), bound.clone());
                streams.spawn(async move {
                    let port = tokio::time::timeout(Duration::from_secs(2), input.read_u16()).await.map_err(io::Error::other)??;
                    if !ports.contains(&port) { return Err(io::Error::other("O1 endpoint denied")); }
                    let (mut send, mut recv) = super::open(&native, port).await?;
                    copy_pair(&mut input, &mut send, &mut recv, &mut out, &bound).await
                });
            }
        }
    }
}

async fn host_peer(
    room: &Connection,
    native: &quinn::Connection,
    bound: &AtomicBool,
    backend: UdpBackend,
) -> io::Result<()> {
    let mut offers = super::read_offers(native).await?;
    if offers.len() != 1 || !offers[0].0.ip().is_loopback() {
        return Err(io::Error::other("O1 peer offer bounds"));
    }
    let (address, certificate, secret) = offers.pop().expect("single offer");
    let endpoint = crate::client_configured(
        (Ipv4Addr::LOCALHOST, 0).into(),
        certificate.into(),
        backend,
        false,
    )?
    .0;
    let mut link = Link::connect(&endpoint, address.into(), &secret).await?;
    let (mut local_out, mut local_in) = link.take_control()?;
    let (mut room_out, mut room_in) = room.open_bi().await.map_err(io::Error::other)?;
    room_out.write_all(PEER).await.map_err(io::Error::other)?;
    bound.store(true, Ordering::Release);
    let remote = Data::Friends(room.clone());
    let local = link.data();
    let packets = async {
        loop {
            tokio::select! {
                packet = remote.receive() => local.send(&packet?)?,
                packet = local.receive() => remote.send(&packet?)?,
            }
        }
    };
    let control = async {
        tokio::try_join!(
            tokio::io::copy(&mut local_in, &mut room_out),
            tokio::io::copy(&mut room_in, &mut local_out)
        )
        .map(|_| ())
    };
    tokio::select! { result = packets => result, result = control => result }
}

async fn copy_pair(
    a_read: &mut (impl tokio::io::AsyncRead + Unpin),
    b_write: &mut (impl tokio::io::AsyncWrite + Unpin),
    b_read: &mut (impl tokio::io::AsyncRead + Unpin),
    a_write: &mut (impl tokio::io::AsyncWrite + Unpin),
    bound: &AtomicBool,
) -> io::Result<()> {
    let (mut up, mut down, mut u, mut d) = (0, 0, "", "");
    tokio::try_join!(
        copy_secret(a_read, b_write, bound, &mut up, &mut u),
        copy_secret(b_read, a_write, bound, &mut down, &mut d)
    )
    .map(|_| ())
}

pub const ROOM_CLOSED_EXIT: i32 = 21;
pub const DIFFERENT_PACKAGE_EXIT: i32 = 22;

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Guest {
    pub route: room::Route,
    pub code: String,
    pub package: [u8; 32],
    pub name: String,
}

pub async fn join(endpoint: &Endpoint, guest: Guest) -> Result<Connection, room::JoinError> {
    let code: Invite = guest.code.parse().map_err(|error| match error {
        kr_p2p::CodeError::Version(_) => room::JoinError::Version,
        kr_p2p::CodeError::Format => room::JoinError::Denied,
    })?;
    room::join(endpoint, &code, &guest.route, guest.package, &guest.name).await
}

pub async fn companion(
    room: Connection,
    listeners: Vec<TcpListener>,
    mut stop: watch::Receiver<bool>,
) -> io::Result<()> {
    let mut accepts = JoinSet::new();
    let (tx, mut rx) = tokio::sync::mpsc::channel(4);
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
    let mut streams = JoinSet::new();
    let mut total = 0;
    let bound = Arc::new(AtomicBool::new(false));
    let peer = guest_peer(room.clone(), stop.clone(), bound.clone());
    tokio::pin!(peer);
    let result = loop {
        tokio::select! {
            biased;
            _ = stop.changed() => break Ok(()),
            _ = room.closed() => break Err(io::Error::new(io::ErrorKind::ConnectionAborted, "O1 room closed")),
            result = &mut peer => break result,
            done = accepts.join_next(), if !accepts.is_empty() => {
                break done.ok_or_else(|| io::Error::other("O1 listeners ended"))?.map_err(io::Error::other)?;
            },
            done = streams.join_next(), if !streams.is_empty() => {
                if let Some(Err(error)) = done { break Err(io::Error::other(error)); }
            },
            item = rx.recv() => {
                let Some((mut socket, port)) = item else { break Err(io::Error::other("O1 listeners ended")); };
                total += 1;
                if total > STREAM_TOTAL || streams.len() >= STREAM_LIMIT as usize { break Err(io::Error::other("O1 stream bound")); }
                let (room, bound) = (room.clone(), bound.clone());
                streams.spawn(async move {
                    let (mut send, mut recv) = room.open_bi().await.map_err(io::Error::other)?;
                    send.write_u16(port).await?;
                    let (mut read, mut write) = socket.split();
                    copy_pair(&mut read, &mut send, &mut recv, &mut write, &bound).await
                });
            }
        }
    };
    room.close(0u32.into(), b"guest companion stopped");
    result
}

async fn guest_peer(
    room: Connection,
    stop: watch::Receiver<bool>,
    bound: Arc<AtomicBool>,
) -> io::Result<()> {
    let (send, mut recv) = room.accept_bi().await.map_err(io::Error::other)?;
    let mut marker = [0; 8];
    tokio::time::timeout(Duration::from_secs(2), recv.read_exact(&mut marker))
        .await
        .map_err(io::Error::other)?
        .map_err(io::Error::other)?;
    if marker != *PEER {
        return Err(io::Error::other("O1 peer marker"));
    }
    bound.store(true, Ordering::Release);
    crate::companion::run_parts(
        send,
        recv,
        Data::Friends(room),
        Ipv4Addr::LOCALHOST.into(),
        8192,
        stop,
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test(flavor = "multi_thread")]
    async fn gateway_passes_named_admission_and_native_stream_to_pinned_server() {
        let identity = rcgen::generate_simple_self_signed(vec!["kr-launch.local".into()]).unwrap();
        let certificate = identity.cert.der().to_vec();
        let tls =
            super::super::server_config(certificate.clone(), identity.signing_key.serialize_der())
                .unwrap();
        let server = crate::server_endpoint(tls, (Ipv4Addr::LOCALHOST, 0).into()).unwrap();
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).await.unwrap();
        let address = listener.local_addr().unwrap();
        let host_endpoint = room::endpoint(false).await.unwrap();
        let route = room::Route {
            node: host_endpoint.id(),
            addresses: vec![(Ipv4Addr::LOCALHOST, host_endpoint.bound_sockets()[0].port()).into()],
            relay: None,
        };
        let std::net::SocketAddr::V4(server_addr) = server.local_addr().unwrap() else {
            unreachable!()
        };
        let host_room = tokio::spawn(serve(
            host_endpoint.clone(),
            Gateway {
                server: server_addr,
                certificate,
                ports: vec![address.port()],
                package: [1; 32],
                backend: UdpBackend::Portable,
            },
        ));
        let guest_endpoint = room::endpoint(false).await.unwrap();
        let guest = join(
            &guest_endpoint,
            Guest {
                code: Invite::room(route.node).to_string(),
                route,
                package: [1; 32],
                name: "Việt01".into(),
            },
        )
        .await
        .unwrap();
        let (keep, stop) = watch::channel(false);
        let companion = tokio::spawn(companion(guest, vec![listener], stop));
        // CI runners can take several seconds to start both QUIC endpoints. Keep the
        // assertion bounded, but do not turn scheduling jitter into an O1 failure.
        tokio::time::timeout(Duration::from_secs(20), async {
            let server = async {
                let incoming = server.accept().await.unwrap().await.unwrap();
                assert_eq!(read_name(&incoming).await.unwrap(), "Việt01");
                let (mut send, mut recv) = incoming.accept_bi().await.unwrap();
                assert_eq!(
                    super::super::accept_header(&mut recv, &[address.port()])
                        .await
                        .unwrap(),
                    address.port()
                );
                let mut bytes = [0; 7];
                recv.read_exact(&mut bytes).await.unwrap();
                assert_eq!(&bytes, b"request");
                send.write_all(b"response").await.unwrap();
                send.flush().await.unwrap();
                // Give the peer a scheduling point to consume the response
                // before ending this half of the stream. CI runners have
                // exposed an early-EOF race here.
                tokio::time::sleep(Duration::from_millis(50)).await;
                send.finish().unwrap();
                let _ = send.stopped().await;
            };
            let client = async {
                let mut local = tokio::net::TcpStream::connect(address).await.unwrap();
                local.write_all(b"request").await.unwrap();
                // Signal request completion so the gateway can finish the
                // upload half without racing the response half.
                local.shutdown().await.unwrap();
                let mut bytes = [0; 8];
                local.read_exact(&mut bytes).await.unwrap();
                assert_eq!(&bytes, b"response");
            };
            tokio::join!(server, client);
        })
        .await
        .unwrap();
        keep.send_replace(true);
        let _ = companion.await.unwrap();
        host_room.abort();
        host_endpoint.close().await;
        guest_endpoint.close().await;
    }
}
