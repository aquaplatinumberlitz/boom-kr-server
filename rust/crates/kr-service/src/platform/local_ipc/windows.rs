use super::{CONTROL_IO, ExchangeError, RESPONSE_IO, auth};
use crate::platform;
use ring::hmac;
use std::{
    fs::{self, File},
    io::{self, Read, Write},
    net::{Ipv4Addr, SocketAddrV4},
    path::{Path, PathBuf},
};
pub(crate) use tokio::net::TcpStream as Stream;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    time::timeout,
};
use zeroize::Zeroizing;

const MAGIC: &[u8; 8] = b"KRIPCv1\0";
const ENDPOINT_SIZE: usize = MAGIC.len() + 2 + 32;

fn read_endpoint(path: &Path) -> io::Result<(u16, hmac::Key)> {
    platform::private_dir(
        path.parent()
            .ok_or_else(|| io::Error::other("IPC parent missing"))?,
    )?;
    let meta = platform::private_file(path)?;
    if meta.len() != ENDPOINT_SIZE as u64 {
        return Err(io::Error::other("invalid local IPC endpoint size"));
    }
    let mut bytes = Zeroizing::new([0; ENDPOINT_SIZE]);
    File::open(path)?.read_exact(bytes.as_mut())?;
    let port = u16::from_be_bytes([bytes[8], bytes[9]]);
    if &bytes[..8] != MAGIC || port == 0 {
        return Err(io::Error::other("invalid local IPC endpoint"));
    }
    Ok((port, hmac::Key::new(hmac::HMAC_SHA256, &bytes[10..])))
}

pub(crate) struct Listener {
    listener: TcpListener,
    key: hmac::Key,
    path: PathBuf,
    published: bool,
    _lock: File,
}
impl Listener {
    pub(crate) fn bind(path: &Path) -> io::Result<Self> {
        platform::private_dir(
            path.parent()
                .ok_or_else(|| io::Error::other("IPC parent missing"))?,
        )?;
        let mut lock = path.as_os_str().to_owned();
        lock.push(".lock");
        let guard = platform::lock_private_file(Path::new(&lock))?;
        // Only discard a valid descriptor left by a dead instance. The
        // endpoint lock stays held until Drop has removed our descriptor.
        match read_endpoint(path) {
            Ok(_) => fs::remove_file(path)?,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => return Err(e),
        }
        let socket = std::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0))?;
        socket.set_nonblocking(true)?;
        let listener = TcpListener::from_std(socket)?;
        let mut bytes = Zeroizing::new([0; ENDPOINT_SIZE]);
        bytes[..8].copy_from_slice(MAGIC);
        bytes[8..10].copy_from_slice(&listener.local_addr()?.port().to_be_bytes());
        getrandom::fill(&mut bytes[10..]).map_err(io::Error::other)?;
        let mut owner = Self {
            listener,
            key: hmac::Key::new(hmac::HMAC_SHA256, &bytes[10..]),
            path: path.to_owned(),
            published: false,
            _lock: guard,
        };
        // Build a complete descriptor before exposing it to readers. This
        // also avoids a partial descriptor being left behind by a crash.
        let stage = path.with_extension(format!("{}.ipc-new", owner.listener.local_addr()?.port()));
        let mut file = platform::private_options()
            .create_new(true)
            .write(true)
            .open(&stage)?;
        let publish = (|| {
            file.write_all(bytes.as_ref())?;
            file.sync_all()?;
            drop(file);
            platform::publish_snapshot(&stage, path)
        })();
        if publish.is_err() {
            let _ = fs::remove_file(stage);
        }
        publish?;
        owner.published = true;
        Ok(owner)
    }

    pub(crate) async fn accept(&self) -> io::Result<Stream> {
        self.listener.accept().await.map(|(stream, _)| stream)
    }

    pub(crate) async fn authenticate(&self, stream: &mut Stream) -> io::Result<()> {
        auth::server(stream, &self.key).await
    }
}
impl Drop for Listener {
    fn drop(&mut self) {
        if self.published {
            let _ = fs::remove_file(&self.path);
        }
    }
}

pub(crate) fn exchange(path: &Path, request: &[u8]) -> Result<Vec<u8>, ExchangeError> {
    let (port, key) = read_endpoint(path).map_err(|_| ExchangeError::Unavailable)?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|_| ExchangeError::Unavailable)?;
    runtime.block_on(async {
        let mut stream = timeout(CONTROL_IO, async {
            let mut stream = Stream::connect(SocketAddrV4::new(Ipv4Addr::LOCALHOST, port)).await?;
            auth::client(&mut stream, &key).await?;
            Ok::<_, io::Error>(stream)
        })
        .await
        .map_err(|_| ExchangeError::Unavailable)?
        .map_err(|_| ExchangeError::Unavailable)?;
        timeout(CONTROL_IO, async {
            stream.write_all(request).await?;
            stream.shutdown().await
        })
        .await
        .map_err(|_| ExchangeError::Unknown)?
        .map_err(|_| ExchangeError::Unknown)?;
        let mut bytes = Vec::new();
        timeout(RESPONSE_IO, stream.take(32_769).read_to_end(&mut bytes))
            .await
            .map_err(|_| ExchangeError::Unknown)?
            .map_err(|_| ExchangeError::Unknown)?;
        Ok(bytes)
    })
}
