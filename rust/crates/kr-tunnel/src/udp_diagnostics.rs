//! Opt-in socket counters. Delegates every operation unchanged; never records
//! packet bytes, addresses, tokens or arbitrary error strings.
use quinn::{
    AsyncUdpSocket, UdpPoller,
    udp::{RecvMeta, Transmit},
};
use std::{
    fmt,
    io::{self, IoSliceMut},
    net::SocketAddr,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicI32, AtomicU64, Ordering},
    },
    task::{Context, Poll},
};

#[derive(Default)]
pub struct Counters {
    send_calls: AtomicU64,
    send_ok: AtomicU64,
    send_bytes: AtomicU64,
    send_would_block: AtomicU64,
    recv_batches: AtomicU64,
    io_errors: AtomicU64,
    last_os_error: AtomicI32,
    suppressed_send_errors: AtomicU64,
    suppressed_os_error: AtomicI32,
}
#[derive(serde::Serialize)]
pub struct Snapshot {
    pub send_calls: u64,
    pub send_ok: u64,
    pub send_bytes: u64,
    pub send_would_block: u64,
    pub recv_batches: u64,
    pub io_errors: u64,
    pub last_os_error: Option<i32>,
    pub suppressed_send_errors: u64,
    pub last_suppressed_os_error: Option<i32>,
}
impl Counters {
    pub fn snapshot(&self) -> Snapshot {
        let read = |a: &AtomicU64| a.load(Ordering::Relaxed);
        let code = self.last_os_error.load(Ordering::Relaxed);
        Snapshot {
            send_calls: read(&self.send_calls),
            send_ok: read(&self.send_ok),
            send_bytes: read(&self.send_bytes),
            send_would_block: read(&self.send_would_block),
            recv_batches: read(&self.recv_batches),
            io_errors: read(&self.io_errors),
            last_os_error: (code != 0).then_some(code),
            suppressed_send_errors: read(&self.suppressed_send_errors),
            last_suppressed_os_error: match self.suppressed_os_error.load(Ordering::Relaxed) {
                0 => None,
                code => Some(code),
            },
        }
    }
    fn error(&self, error: &io::Error) {
        self.io_errors.fetch_add(1, Ordering::Relaxed);
        if let Some(code) = error.raw_os_error() {
            self.last_os_error.store(code, Ordering::Relaxed);
        }
    }
}

/// Quinn's UDP adapter logs and suppresses non-WouldBlock send errors. Its
/// upstream message also formats endpoint addresses, so never forward it.
/// Keep only a numeric OS code from the pinned upstream Debug shape.
pub fn install_redacted_log(counters: Arc<Counters>) -> io::Result<()> {
    tracing::subscriber::set_global_default(Redacted(counters))
        .map_err(|_| io::Error::other("diagnostic subscriber unavailable"))
}
struct Redacted(Arc<Counters>);
struct Visitor<'a>(&'a Counters);
impl tracing::field::Visit for Visitor<'_> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn fmt::Debug) {
        if field.name() != "message" {
            return;
        }
        let text = zeroize::Zeroizing::new(format!("{value:?}"));
        if !text.starts_with("sendmsg error: ") {
            return;
        }
        self.0
            .suppressed_send_errors
            .fetch_add(1, Ordering::Relaxed);
        if let Some(tail) = text.strip_prefix("sendmsg error: Os { code: ") {
            let number: String = tail
                .chars()
                .take_while(|c| c.is_ascii_digit() || *c == '-')
                .take(12)
                .collect();
            if let Ok(code) = number.parse::<i32>() {
                self.0.suppressed_os_error.store(code, Ordering::Relaxed);
            }
        }
    }
}
impl tracing::Subscriber for Redacted {
    fn enabled(&self, metadata: &tracing::Metadata<'_>) -> bool {
        metadata.target() == "quinn_udp"
    }
    fn new_span(&self, _: &tracing::span::Attributes<'_>) -> tracing::span::Id {
        tracing::span::Id::from_u64(1)
    }
    fn record(&self, _: &tracing::span::Id, _: &tracing::span::Record<'_>) {}
    fn record_follows_from(&self, _: &tracing::span::Id, _: &tracing::span::Id) {}
    fn event(&self, event: &tracing::Event<'_>) {
        if self.enabled(event.metadata()) {
            event.record(&mut Visitor(&self.0));
        }
    }
    fn enter(&self, _: &tracing::span::Id) {}
    fn exit(&self, _: &tracing::span::Id) {}
}

pub(crate) struct Observed {
    pub inner: Arc<dyn AsyncUdpSocket>,
    pub counters: Arc<Counters>,
}
impl fmt::Debug for Observed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ObservedUdpSocket")
    }
}
impl AsyncUdpSocket for Observed {
    fn create_io_poller(self: Arc<Self>) -> Pin<Box<dyn UdpPoller>> {
        self.inner.clone().create_io_poller()
    }
    fn try_send(&self, transmit: &Transmit) -> io::Result<()> {
        self.counters.send_calls.fetch_add(1, Ordering::Relaxed);
        let result = self.inner.try_send(transmit);
        match &result {
            Ok(()) => {
                self.counters.send_ok.fetch_add(1, Ordering::Relaxed);
                self.counters
                    .send_bytes
                    .fetch_add(transmit.contents.len() as u64, Ordering::Relaxed);
            }
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                self.counters
                    .send_would_block
                    .fetch_add(1, Ordering::Relaxed);
            }
            Err(error) => self.counters.error(error),
        }
        result
    }
    fn poll_recv(
        &self,
        cx: &mut Context<'_>,
        bufs: &mut [IoSliceMut<'_>],
        meta: &mut [RecvMeta],
    ) -> Poll<io::Result<usize>> {
        let result = self.inner.poll_recv(cx, bufs, meta);
        match &result {
            Poll::Ready(Ok(n)) => {
                self.counters
                    .recv_batches
                    .fetch_add(*n as u64, Ordering::Relaxed);
            }
            Poll::Ready(Err(error)) => self.counters.error(error),
            _ => {}
        }
        result
    }
    fn local_addr(&self) -> io::Result<SocketAddr> {
        self.inner.local_addr()
    }
    fn max_transmit_segments(&self) -> usize {
        self.inner.max_transmit_segments()
    }
    fn max_receive_segments(&self) -> usize {
        self.inner.max_receive_segments()
    }
    fn may_fragment(&self) -> bool {
        self.inner.may_fragment()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn upstream_send_error_retains_numeric_code_without_endpoint_or_error_text() {
        let counters = Arc::new(Counters::default());
        tracing::subscriber::with_default(Redacted(counters.clone()), || {
            tracing::warn!(target:"quinn_udp","sendmsg error: Os {{ code: 10022, kind: InvalidInput, message: \"private-sentinel\" }}, Transmit: {{ destination: 192.0.2.18:46000 }}");
            tracing::warn!(target:"other_dependency","sendmsg error: Os {{ code: 10013 }} private-sentinel");
        });
        let snapshot = counters.snapshot();
        assert_eq!(snapshot.suppressed_send_errors, 1);
        assert_eq!(snapshot.last_suppressed_os_error, Some(10022));
        let encoded = serde_json::to_string(&snapshot).unwrap();
        assert!(
            !encoded.contains("private-sentinel")
                && !encoded.contains("192.0.2.18")
                && !encoded.contains("destination")
        );
    }
}
