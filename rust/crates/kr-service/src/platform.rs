//! Filesystem and lifecycle primitives for a private service state directory.
use std::{
    fs::{self, File, OpenOptions},
    io,
    path::Path,
};

pub(crate) mod local_ipc;
#[cfg(windows)]
#[allow(unsafe_code)]
mod windows;

pub(crate) fn private_dir(path: &Path) -> io::Result<()> {
    check_private(path, true).map(|_| ())
}

pub(crate) fn private_file(path: &Path) -> io::Result<fs::Metadata> {
    check_private(path, false)
}

fn check_private(path: &Path, directory: bool) -> io::Result<fs::Metadata> {
    let meta = fs::symlink_metadata(path)?;
    if if directory {
        !meta.is_dir()
    } else {
        !meta.is_file()
    } {
        return Err(io::Error::other(
            "expected private directory or regular file",
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if meta.permissions().mode() & 0o077 != 0 {
            return Err(io::Error::other("file or directory is not private"));
        }
    }
    #[cfg(windows)]
    windows::check_private(path, &meta, directory)?;
    Ok(meta)
}

/// The parent is trusted and already private. On Windows we create the
/// directory with its own inheritable DACL even if the parent is broader.
fn create_private_dir(path: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        fs::DirBuilder::new().mode(0o700).create(path)
    }
    #[cfg(windows)]
    windows::create_private_dir(path)
}

/// Create a state directory, or verify the existing directory. Its parent
/// must already exist and be at a trusted path.
pub fn provision_private_dir(path: &Path) -> io::Result<()> {
    match create_private_dir(path) {
        Ok(()) => {}
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
        Err(e) => return Err(e),
    }
    private_dir(path)
}

pub(crate) fn private_options() -> OpenOptions {
    #[allow(unused_mut)]
    let mut options = OpenOptions::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    // Files created on Windows inherit the verified parent DACL. This also
    // protects SQLite's WAL and SHM files, which SQLite creates itself.
    options
}

pub(crate) fn publish_snapshot(source: &Path, destination: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        fs::hard_link(source, destination)
    }
    #[cfg(windows)]
    windows::publish_snapshot(source, destination)
}

pub(crate) fn sync_directory(path: &Path) -> io::Result<()> {
    #[cfg(unix)]
    File::open(path)?.sync_all()?;
    // Windows does not support FlushFileBuffers on a normal directory handle.
    // Snapshot publication uses a write-through move; retention deletes have
    // no separate metadata flush.
    #[cfg(windows)]
    let _ = path;
    Ok(())
}

pub(crate) struct Shutdown {
    #[cfg(unix)]
    term: tokio::signal::unix::Signal,
    #[cfg(windows)]
    ctrl_c: tokio::signal::windows::CtrlC,
    #[cfg(windows)]
    ctrl_break: tokio::signal::windows::CtrlBreak,
}

impl Shutdown {
    pub(crate) fn new() -> io::Result<Self> {
        Ok(Self {
            #[cfg(unix)]
            term: tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?,
            #[cfg(windows)]
            ctrl_c: tokio::signal::windows::ctrl_c()?,
            #[cfg(windows)]
            ctrl_break: tokio::signal::windows::ctrl_break()?,
        })
    }

    pub(crate) async fn recv(&mut self) -> io::Result<()> {
        #[cfg(unix)]
        tokio::select! {
            _ = self.term.recv() => Ok(()),
            value = tokio::signal::ctrl_c() => value,
        }
        #[cfg(windows)]
        tokio::select! {
            _ = self.ctrl_c.recv() => Ok(()),
            _ = self.ctrl_break.recv() => Ok(()),
        }
    }
}

pub(crate) fn lock_private_file(path: &Path) -> io::Result<File> {
    match private_file(path) {
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    let lock = private_options()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(path)?;
    lock.try_lock()?;
    Ok(lock)
}
