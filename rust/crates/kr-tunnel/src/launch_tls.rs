//! Company launcher TLS transport. This is independent of the original game
//! protocol and carries one bounded JSON frame per authenticated TLS connection.
use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer, ServerName};
use rustls::{ClientConfig, ClientConnection, RootCertStore, ServerConfig, ServerConnection};
use std::{
    io::{self, Read, Write},
    net::TcpStream,
    sync::Arc,
    time::{Duration, Instant},
};
use zeroize::Zeroizing;

pub const NAME: &str = "kr-launch.local";
pub const REQUEST_LIMIT: usize = 4096;
pub const RESPONSE_LIMIT: usize = 8192;
pub type ServerTls = Arc<ServerConfig>;

/// Generate an identity for operator provisioning. Persist it privately and
/// distribute the certificate pin with the player package; never rotate it as
/// a side effect of accepting a connection.
pub fn generate_identity() -> io::Result<(Vec<u8>, Zeroizing<Vec<u8>>)> {
    let certificate = rcgen::generate_simple_self_signed(vec![NAME.into()])
        .map_err(|_| io::Error::other("launcher identity generation failed"))?;
    Ok((
        certificate.cert.der().to_vec(),
        Zeroizing::new(certificate.signing_key.serialize_der()),
    ))
}

/// Recompute the remaining absolute deadline for every underlying TLS read and
/// write. A peer cannot extend a handshake by sending one byte per timeout.
pub struct DeadlineIo {
    socket: TcpStream,
    deadline: Instant,
}
impl DeadlineIo {
    fn new(socket: TcpStream, duration: Duration) -> io::Result<Self> {
        if duration.is_zero() || duration > Duration::from_secs(30) {
            return Err(io::Error::other("launcher connection deadline bounds"));
        }
        socket.set_nodelay(true)?;
        Ok(Self {
            socket,
            deadline: Instant::now() + duration,
        })
    }
    pub fn remaining(&self) -> io::Result<Duration> {
        self.deadline
            .checked_duration_since(Instant::now())
            .filter(|duration| !duration.is_zero())
            .ok_or_else(|| io::Error::new(io::ErrorKind::TimedOut, "launcher connection deadline"))
    }
}
impl Read for DeadlineIo {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.socket.set_read_timeout(Some(self.remaining()?))?;
        self.socket.read(buf)
    }
}
impl Write for DeadlineIo {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.socket.set_write_timeout(Some(self.remaining()?))?;
        self.socket.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.remaining()?;
        self.socket.flush()
    }
}

pub fn server_config(certificate: Vec<u8>, key: Vec<u8>) -> io::Result<Arc<ServerConfig>> {
    let mut config = ServerConfig::builder_with_protocol_versions(&[&rustls::version::TLS13])
        .with_no_client_auth()
        .with_single_cert(
            vec![CertificateDer::from(certificate)],
            PrivatePkcs8KeyDer::from(key).into(),
        )
        .map_err(|_| io::Error::other("launcher TLS identity invalid"))?;
    config.max_early_data_size = 0;
    config.alpn_protocols = vec![b"kre-launch/1".to_vec()];
    Ok(Arc::new(config))
}

pub fn accept(
    socket: TcpStream,
    config: Arc<ServerConfig>,
    duration: Duration,
) -> io::Result<rustls::StreamOwned<ServerConnection, DeadlineIo>> {
    let mut connection = ServerConnection::new(config)
        .map_err(|_| io::Error::other("launcher TLS startup failed"))?;
    connection.set_buffer_limit(Some(16 * 1024));
    let mut stream = rustls::StreamOwned::new(connection, DeadlineIo::new(socket, duration)?);
    while stream.conn.is_handshaking() {
        stream.conn.complete_io(&mut stream.sock)?;
    }
    if stream.conn.alpn_protocol() != Some(b"kre-launch/1") {
        return Err(io::Error::other("launcher TLS protocol mismatch"));
    }
    Ok(stream)
}

pub fn connect(
    socket: TcpStream,
    certificate: Vec<u8>,
    duration: Duration,
) -> io::Result<rustls::StreamOwned<ClientConnection, DeadlineIo>> {
    let mut roots = RootCertStore::empty();
    roots
        .add(CertificateDer::from(certificate.clone()))
        .map_err(|_| io::Error::other("launcher TLS trust invalid"))?;
    let mut config = ClientConfig::builder_with_protocol_versions(&[&rustls::version::TLS13])
        .with_root_certificates(roots)
        .with_no_client_auth();
    config.enable_early_data = false;
    config.alpn_protocols = vec![b"kre-launch/1".to_vec()];
    let mut connection = ClientConnection::new(
        Arc::new(config),
        ServerName::try_from(NAME).expect("constant name"),
    )
    .map_err(|_| io::Error::other("launcher TLS startup failed"))?;
    connection.set_buffer_limit(Some(16 * 1024));
    let mut stream = rustls::StreamOwned::new(connection, DeadlineIo::new(socket, duration)?);
    while stream.conn.is_handshaking() {
        stream.conn.complete_io(&mut stream.sock)?;
    }
    if stream.conn.alpn_protocol() != Some(b"kre-launch/1")
        || stream
            .conn
            .peer_certificates()
            .and_then(|certificates| certificates.first())
            .is_none_or(|actual| actual.as_ref() != certificate)
    {
        return Err(io::Error::other("launcher TLS pin or protocol mismatch"));
    }
    Ok(stream)
}

pub fn read_frame(stream: &mut impl Read, limit: usize) -> io::Result<Zeroizing<Vec<u8>>> {
    if limit == 0 || limit > RESPONSE_LIMIT {
        return Err(io::Error::other("launcher frame limit"));
    }
    let mut frame = Zeroizing::new(Vec::new());
    let mut chunk = Zeroizing::new([0; 512]);
    loop {
        let count = stream.read(&mut *chunk)?;
        if count == 0 {
            return Err(io::Error::other("incomplete launcher frame"));
        }
        if frame.len() + count > limit {
            return Err(io::Error::other("launcher frame bound"));
        }
        frame.extend_from_slice(&chunk[..count]);
        if let Some(position) = frame.iter().position(|byte| *byte == b'\n') {
            if position + 1 != frame.len() {
                return Err(io::Error::other("one launcher frame per connection"));
            }
            return Ok(frame);
        }
    }
}

pub fn write_frame(stream: &mut impl Write, frame: &[u8], limit: usize) -> io::Result<()> {
    if limit == 0
        || limit > RESPONSE_LIMIT
        || frame.is_empty()
        || frame.len() > limit
        || frame.last() != Some(&b'\n')
        || frame[..frame.len() - 1].contains(&b'\n')
    {
        return Err(io::Error::other("launcher output frame bound"));
    }
    stream.write_all(frame)?;
    stream.flush()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{net::TcpListener, thread};
    fn identity() -> (Vec<u8>, Vec<u8>) {
        let cert = rcgen::generate_simple_self_signed(vec![NAME.into()]).unwrap();
        (cert.cert.der().to_vec(), cert.signing_key.serialize_der())
    }
    #[test]
    fn pinned_tls_delivers_bounded_frames_and_rejects_wrong_identity() {
        for trusted in [true, false] {
            let (certificate, key) = identity();
            let config = server_config(certificate.clone(), key).unwrap();
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let address = listener.local_addr().unwrap();
            let server = thread::spawn(move || -> io::Result<()> {
                let mut stream = accept(listener.accept()?.0, config, Duration::from_secs(2))?;
                assert_eq!(
                    &*read_frame(&mut stream, REQUEST_LIMIT)?,
                    b"{\"op\":\"test\"}\n"
                );
                write_frame(&mut stream, b"{\"ok\":true}\n", RESPONSE_LIMIT)
            });
            let trust = if trusted { certificate } else { identity().0 };
            let client = connect(
                TcpStream::connect(address).unwrap(),
                trust,
                Duration::from_secs(2),
            );
            if trusted {
                let mut stream = client.unwrap();
                write_frame(&mut stream, b"{\"op\":\"test\"}\n", REQUEST_LIMIT).unwrap();
                assert_eq!(
                    &*read_frame(&mut stream, RESPONSE_LIMIT).unwrap(),
                    b"{\"ok\":true}\n"
                );
                server.join().unwrap().unwrap();
            } else {
                assert!(client.is_err());
                assert!(server.join().unwrap().is_err());
            }
        }
    }
    #[test]
    fn plaintext_and_stalled_handshakes_never_deliver_a_frame() {
        for plaintext in [true, false] {
            let (certificate, key) = identity();
            let config = server_config(certificate, key).unwrap();
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let address = listener.local_addr().unwrap();
            let server = thread::spawn(move || {
                let started = Instant::now();
                assert!(
                    accept(
                        listener.accept().unwrap().0,
                        config,
                        Duration::from_millis(100)
                    )
                    .is_err()
                );
                assert!(started.elapsed() < Duration::from_secs(2));
            });
            let mut socket = TcpStream::connect(address).unwrap();
            if plaintext {
                socket.write_all(b"{\"op\":\"login\"}\n").unwrap();
            }
            server.join().unwrap();
        }
    }
    #[test]
    fn framing_rejects_oversize_truncation_and_multiple_commands() {
        for mut input in [
            &b"no newline"[..],
            &b"{}\n{}\n"[..],
            &[b'x'; REQUEST_LIMIT + 1][..],
        ] {
            assert!(read_frame(&mut input, REQUEST_LIMIT).is_err());
        }
        assert!(write_frame(&mut Vec::new(), b"{}\n{}\n", REQUEST_LIMIT).is_err());
    }
}
