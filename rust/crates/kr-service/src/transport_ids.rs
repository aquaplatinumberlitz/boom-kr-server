//! One namespace for actual native streams, shared by TCP and TLS ingress.
use std::{
    io,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(1);

pub(crate) fn next() -> io::Result<u64> {
    NEXT.try_update(Ordering::AcqRel, Ordering::Acquire, |id| id.checked_add(1))
        .map_err(|_| io::Error::other("native connection identity exhausted"))
}
