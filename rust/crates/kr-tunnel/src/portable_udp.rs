//! Single-datagram client socket without ECN/GSO ancillary messages.
use quinn::{
    AsyncUdpSocket, UdpPoller,
    udp::{RecvMeta, Transmit},
};
use std::{
    fmt,
    future::Future,
    io::{self, IoSliceMut},
    net::SocketAddr,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
pub(crate) struct Socket {
    io: Arc<tokio::net::UdpSocket>,
}
impl fmt::Debug for Socket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PortableClientUdp")
    }
}
impl Socket {
    pub fn bind(address: SocketAddr) -> io::Result<Arc<dyn AsyncUdpSocket>> {
        let socket = std::net::UdpSocket::bind(address)?;
        socket.set_nonblocking(true)?;
        Ok(Arc::new(Self {
            io: Arc::new(tokio::net::UdpSocket::from_std(socket)?),
        }))
    }
}
type Writable = Pin<Box<dyn Future<Output = io::Result<()>> + Send + Sync>>;
struct Poller {
    socket: Arc<tokio::net::UdpSocket>,
    waiting: Option<Writable>,
}
impl fmt::Debug for Poller {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PortableUdpPoller")
    }
}
impl UdpPoller for Poller {
    fn poll_writable(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        if this.waiting.is_none() {
            let socket = this.socket.clone();
            this.waiting = Some(Box::pin(async move { socket.writable().await }));
        }
        let result = this
            .waiting
            .as_mut()
            .expect("installed writable future")
            .as_mut()
            .poll(cx);
        if result.is_ready() {
            this.waiting = None;
        }
        result
    }
}
impl AsyncUdpSocket for Socket {
    fn create_io_poller(self: Arc<Self>) -> Pin<Box<dyn UdpPoller>> {
        Box::pin(Poller {
            socket: self.io.clone(),
            waiting: None,
        })
    }
    fn try_send(&self, transmit: &Transmit<'_>) -> io::Result<()> {
        if transmit
            .segment_size
            .is_some_and(|size| size < transmit.contents.len())
        {
            return Err(io::Error::other("portable socket requires one datagram"));
        }
        let sent = self
            .io
            .try_send_to(transmit.contents, transmit.destination)?;
        if sent != transmit.contents.len() {
            return Err(io::Error::new(
                io::ErrorKind::WriteZero,
                "partial UDP datagram",
            ));
        }
        Ok(())
    }
    fn poll_recv(
        &self,
        cx: &mut Context<'_>,
        bufs: &mut [IoSliceMut<'_>],
        meta: &mut [RecvMeta],
    ) -> Poll<io::Result<usize>> {
        if bufs.is_empty() || meta.is_empty() {
            return Poll::Ready(Err(io::Error::other("empty UDP receive buffers")));
        }
        let mut buffer = tokio::io::ReadBuf::new(&mut bufs[0]);
        match self.io.poll_recv_from(cx, &mut buffer) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(error)) => Poll::Ready(Err(error)),
            Poll::Ready(Ok(addr)) => {
                let len = buffer.filled().len();
                meta[0] = RecvMeta {
                    addr,
                    len,
                    stride: len.max(1),
                    ecn: None,
                    dst_ip: None,
                };
                Poll::Ready(Ok(1))
            }
        }
    }
    fn local_addr(&self) -> io::Result<SocketAddr> {
        self.io.local_addr()
    }
    fn may_fragment(&self) -> bool {
        true
    }
}
