use super::{CONTROL_IO, ExchangeError, RESPONSE_IO};
use std::{
    fs,
    io::{self, Read, Write},
    os::unix::{
        fs::{FileTypeExt, MetadataExt, PermissionsExt},
        net::UnixStream as BlockingStream,
    },
    path::{Path, PathBuf},
};
use tokio::net::UnixListener;
pub(crate) use tokio::net::UnixStream as Stream;

pub(crate) struct Listener {
    listener: UnixListener,
    _owner: Owner,
}
impl Listener {
    pub(crate) fn bind(path: &Path) -> io::Result<Self> {
        let (listener, owner) = Owner::bind(path)?;
        Ok(Self {
            listener,
            _owner: owner,
        })
    }

    pub(crate) async fn accept(&self) -> io::Result<Stream> {
        self.listener.accept().await.map(|(stream, _)| stream)
    }

    pub(crate) async fn authenticate(&self, _stream: &mut Stream) -> io::Result<()> {
        Ok(())
    }
}

pub(crate) fn exchange(path: &Path, request: &[u8]) -> Result<Vec<u8>, ExchangeError> {
    let mut stream = BlockingStream::connect(path).map_err(|_| ExchangeError::Unavailable)?;
    stream
        .set_write_timeout(Some(CONTROL_IO))
        .map_err(|_| ExchangeError::Unknown)?;
    stream
        .set_read_timeout(Some(RESPONSE_IO))
        .map_err(|_| ExchangeError::Unknown)?;
    stream
        .write_all(request)
        .map_err(|_| ExchangeError::Unknown)?;
    stream
        .shutdown(std::net::Shutdown::Write)
        .map_err(|_| ExchangeError::Unknown)?;
    let mut bytes = Vec::new();
    stream
        .take(32_769)
        .read_to_end(&mut bytes)
        .map_err(|_| ExchangeError::Unknown)?;
    Ok(bytes)
}

struct Owner {
    path: PathBuf,
    device: u64,
    inode: u64,
}
impl Owner {
    fn bind(path: &Path) -> io::Result<(UnixListener, Self)> {
        // The caller holds the instance lock. Never unlink a live listener or
        // any non-socket, even after an unclean process exit.
        match fs::symlink_metadata(path) {
            Ok(meta) if meta.file_type().is_socket() => match BlockingStream::connect(path) {
                Err(e) if e.kind() == io::ErrorKind::ConnectionRefused => fs::remove_file(path)?,
                _ => return Err(io::Error::other("operator socket path already active")),
            },
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            _ => return Err(io::Error::other("operator socket path occupied")),
        }
        let listener = UnixListener::bind(path)?;
        let meta = fs::symlink_metadata(path)?;
        let owner = Self {
            path: path.to_owned(),
            device: meta.dev(),
            inode: meta.ino(),
        };
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
        Ok((listener, owner))
    }
}
impl Drop for Owner {
    fn drop(&mut self) {
        if let Ok(meta) = fs::symlink_metadata(&self.path)
            && meta.file_type().is_socket()
            && meta.dev() == self.device
            && meta.ino() == self.inode
        {
            let _ = fs::remove_file(&self.path);
        }
    }
}
