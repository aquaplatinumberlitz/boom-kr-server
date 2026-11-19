//! Server-authenticated TLS13 before native credentials exist. This transport
//! grants no player authority: the server's native DB owner authenticates later.
use quinn::{
    ClientConfig, Connection, Endpoint, RecvStream, SendStream, ServerConfig, TransportConfig,
};
use rustls::{
    RootCertStore,
    pki_types::{CertificateDer, PrivatePkcs8KeyDer},
};
use std::{io, net::SocketAddr, sync::Arc, time::Duration};
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
struct Close(Connection);
impl Drop for Close {
    fn drop(&mut self) {
        self.0.close(0u8.into(), b"companion owner ended");
    }
}

fn transport() -> Arc<TransportConfig> {
    let mut cfg = TransportConfig::default();
    cfg.max_concurrent_bidi_streams(STREAM_LIMIT.into());
    cfg.max_concurrent_uni_streams(0u8.into());
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
) -> io::Result<()> {
    let mut buffer = Zeroizing::new(vec![0; 1024]);
    let mut total = 0;
    loop {
        let size = reader.read(&mut buffer).await?;
        if size == 0 {
            writer.shutdown().await?;
            return Ok(());
        }
        total += size;
        if total > BYTE_LIMIT {
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
/// One pinned TLS connection owns all WCP/login/channel streams. Closing it
/// tears down every local socket; a replacement connection gets a fresh owner.
pub async fn companion(
    connection: Connection,
    listeners: Vec<TcpListener>,
    mut stop: watch::Receiver<bool>,
) -> io::Result<()> {
    let _close = Close(connection.clone());
    let (tx, mut rx) = tokio::sync::mpsc::channel(4);
    let mut accepts = JoinSet::new();
    for listener in listeners {
        let tx = tx.clone();
        accepts.spawn(async move {
            loop {
                let (socket, _) = listener.accept().await?;
                let port = listener.local_addr()?.port();
                let _ = tx.try_send((socket, port));
            }
            #[allow(unreachable_code)]
            Ok::<(), io::Error>(())
        });
    }
    drop(tx);
    let mut streams = JoinSet::new();
    let mut total = 0;
    let result=async{loop{
        if *stop.borrow(){return Ok(());}
        tokio::select!{
            _=stop.changed()=>return Ok(()),
            _=connection.closed()=>return Err(io::Error::other("native TLS ended")),
            result=streams.join_next(),if !streams.is_empty()=>{
                if let Some(Err(error))=result{return Err(io::Error::other(error));}
            },
            accepted=rx.recv()=>{
                let Some((mut socket,port))=accepted else{return Err(io::Error::other("native listeners ended"));};
                total+=1;
                if total>STREAM_TOTAL||streams.len()>=STREAM_LIMIT as usize{return Err(io::Error::other("native companion stream bound"));}
                let tls=connection.clone();streams.spawn(async move{
                    let (mut send,mut recv)=open(&tls,port).await?;
                    let (mut read,mut write)=socket.split();
                    tokio::try_join!(copy_secret(&mut read,&mut send),copy_secret(&mut recv,&mut write)).map(|_|())
                });
            }
        }
    }}.await;
    accepts.abort_all();
    while accepts.join_next().await.is_some() {}
    streams.abort_all();
    while streams.join_next().await.is_some() {}
    connection.close(0u8.into(), b"native owner ended");
    result
}

#[cfg(test)]
mod tests;
