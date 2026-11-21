//! Linux retains the Unix socket protocol. Windows uses the same byte stream
//! on loopback after authenticating both peers with a private endpoint file.
use std::time::Duration;

#[cfg(any(windows, test))]
mod auth;
#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

#[cfg(unix)]
pub(crate) use unix::*;
#[cfg(windows)]
pub(crate) use windows::*;

pub(crate) const CONTROL_IO: Duration = Duration::from_secs(2);
pub(crate) const RESPONSE_IO: Duration = Duration::from_secs(15);

pub(crate) enum ExchangeError {
    Unavailable,
    Unknown,
}
