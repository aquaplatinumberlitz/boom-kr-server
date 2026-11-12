//! Bounded, prebound sockets for the candidate relay. Ports are used once per
//! pool lifetime; keep this owner alive until service shutdown. This does not
//! solve port reuse across restart or authenticate a UDP sender.
use super::*;
use std::{
    net::{SocketAddr, UdpSocket},
    sync::Arc,
};

pub struct SocketPool {
    sockets: Vec<Arc<UdpSocket>>,
    endpoints: Vec<SocketAddrV4>,
    consumed: usize,
    service_generation: Option<u64>,
    used_keys: HashSet<(u32, u16, u64, u8)>,
}
pub struct RoundSockets {
    kernel: Kernel,
    sockets: Vec<Arc<UdpSocket>>,
    receive_cursor: usize,
    receives_per_pump: usize,
}
#[derive(Default, Debug, PartialEq, Eq)]
pub struct PumpStats {
    pub received: usize,
    pub accepted: usize,
    pub dropped: usize,
    pub sent: usize,
}
impl SocketPool {
    /// Call before serving and outside room locks. Bind is all-or-nothing; no
    /// endpoint is published here. No ephemeral ports or implicit NAT mapping.
    pub fn bind(bind_ip: Ipv4Addr, advertised_ip: Ipv4Addr, ports: &[u16]) -> io::Result<Self> {
        let mut distinct = HashSet::new();
        if ports.is_empty()
            || ports.len() > 4096
            || !unicast(advertised_ip)
            || bind_ip.is_multicast()
            || bind_ip.is_broadcast()
            || ports.iter().any(|p| *p == 0 || !distinct.insert(*p))
        {
            return Err(invalid());
        }
        let sockets: Vec<_> = ports
            .iter()
            .map(|p| UdpSocket::bind((bind_ip, *p)))
            .collect::<io::Result<_>>()?;
        Self::from_bound(sockets, advertised_ip)
    }
    fn from_bound(sockets: Vec<UdpSocket>, advertised_ip: Ipv4Addr) -> io::Result<Self> {
        if sockets.is_empty() || sockets.len() > 4096 || !unicast(advertised_ip) {
            return Err(invalid());
        }
        let mut endpoints = Vec::with_capacity(sockets.len());
        let mut distinct = HashSet::new();
        for socket in &sockets {
            socket.set_nonblocking(true)?;
            let SocketAddr::V4(local) = socket.local_addr()? else {
                return Err(invalid());
            };
            let endpoint = SocketAddrV4::new(advertised_ip, local.port());
            if !distinct.insert(endpoint) {
                return Err(invalid());
            }
            endpoints.push(endpoint);
        }
        Ok(Self {
            sockets: sockets.into_iter().map(Arc::new).collect(),
            endpoints,
            consumed: 0,
            service_generation: None,
            used_keys: HashSet::new(),
        })
    }
    pub fn remaining(&self) -> usize {
        self.sockets.len() - self.consumed
    }
    /// Consumes a fresh directed-edge reservation. Even cancellation never
    /// returns its ports to this pool; all remain bound in quarantine.
    pub fn reserve(
        &mut self,
        scope: Scope,
        roster: &[Participant],
        limits: Limits,
        receives_per_pump: usize,
        now: Instant,
    ) -> io::Result<RoundSockets> {
        if !(1..=4096).contains(&receives_per_pump) {
            return Err(invalid());
        }
        if self
            .service_generation
            .is_some_and(|generation| generation != scope.service_generation)
        {
            return Err(io::Error::other(
                "relay pool belongs to another service generation",
            ));
        }
        // Opt-in relay policy only: never reuse a room's low-byte wire key in
        // this instance, including cancelled reservations. This is a resource /
        // stale-data fence, not authentication or proof every UDP type has a key.
        let key = (
            scope.room.id.0,
            scope.room.session,
            scope.room.generation,
            crate::room::wire_key(scope.round)? as u8,
        );
        if self.used_keys.contains(&key) {
            return Err(io::Error::other("relay room low-byte key already consumed"));
        }
        let count = roster
            .len()
            .checked_mul(roster.len().saturating_sub(1))
            .ok_or_else(invalid)?;
        let end = self.consumed.checked_add(count).ok_or_else(invalid)?;
        if end > self.sockets.len() {
            return Err(io::Error::other("relay port pool exhausted"));
        }
        let kernel = Kernel::prepare(
            scope,
            roster,
            &self.endpoints[self.consumed..end],
            limits,
            now,
        )?;
        let sockets = self.sockets[self.consumed..end].to_vec();
        self.consumed = end;
        self.service_generation = Some(scope.service_generation);
        self.used_keys.insert(key);
        Ok(RoundSockets {
            kernel,
            sockets,
            receive_cursor: 0,
            receives_per_pump,
        })
    }
}
impl RoundSockets {
    pub fn routes(&self) -> impl Iterator<Item = (usize, RouteKey, SocketAddrV4)> + '_ {
        self.kernel.routes()
    }
    pub fn endpoints_for(
        &self,
        scope: Scope,
        recipient: Participant,
    ) -> io::Result<Vec<(u32, u16)>> {
        self.kernel.endpoints_for(scope, recipient)
    }
    /// Owner must revalidate room/session/member generations before this call.
    /// Drop datagrams already queued while prepared, with a finite per-socket
    /// budget; fail closed if a socket cannot be drained. Source IP is not auth.
    pub fn activate(&mut self, scope: Scope, now: Instant) -> io::Result<()> {
        if self.kernel.state != State::Prepared {
            return Err(invalid());
        }
        let mut buffer = [0; 256];
        for socket in &self.sockets {
            let mut drained = false;
            for _ in 0..32 {
                match socket.recv_from(&mut buffer) {
                    Ok(_) => {}
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                        drained = true;
                        break;
                    }
                    Err(e) => {
                        self.kernel.invalidate();
                        return Err(e);
                    }
                }
            }
            if !drained {
                self.kernel.invalidate();
                return Err(io::Error::other("relay preactivation queue saturated"));
            }
        }
        self.kernel.activate(scope, now)
    }
    pub fn invalidate(&mut self) {
        self.kernel.invalidate();
    }
    /// Caller serializes ownership validation, pump and invalidation under the
    /// same owner mutex. No await or blocking I/O; receive attempts are bounded
    /// by message budget plus socket count, sends by drain budget. A caller
    /// cannot safely authorize using the supplied scope without checking owners.
    pub fn pump(&mut self, scope: Scope, now: Instant) -> io::Result<PumpStats> {
        if scope != self.kernel.scope {
            self.invalidate();
            return Err(io::Error::other("stale relay pump scope"));
        }
        let mut stats = PumpStats::default();
        if self.kernel.advance(now).is_err() {
            return Ok(stats);
        }
        let mut buffer = [0; 256]; // max_datagram <=255; truncation is rejected.
        let mut empty = 0;
        for _ in 0..self.receives_per_pump + self.sockets.len() {
            if empty == self.sockets.len() || stats.received == self.receives_per_pump {
                break;
            }
            let index = self.receive_cursor;
            self.receive_cursor = (index + 1) % self.sockets.len();
            match self.sockets[index].recv_from(&mut buffer) {
                Ok((n, SocketAddr::V4(from))) => {
                    empty = 0;
                    stats.received += 1;
                    let key = self.kernel.edges[index].key;
                    if self
                        .kernel
                        .ingest(index, key, from, &buffer[..n], now)
                        .is_ok()
                    {
                        stats.accepted += 1;
                    } else {
                        stats.dropped += 1;
                    }
                }
                Ok(_) => {
                    empty = 0;
                    stats.received += 1;
                    stats.dropped += 1;
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    empty += 1;
                }
                Err(e) => {
                    self.invalidate();
                    return Err(e);
                }
            }
        }
        let sockets = &self.sockets;
        stats.sent = self.kernel.drain(now, |reverse, target, bytes| {
            sockets[reverse].send_to(bytes, target)
        })?;
        Ok(stats)
    }
}
impl Drop for RoundSockets {
    fn drop(&mut self) {
        self.invalidate();
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::{fixture, limits};
    use super::*;
    fn pool(n: usize) -> SocketPool {
        SocketPool::from_bound(
            (0..n)
                .map(|_| UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap())
                .collect(),
            Ipv4Addr::LOCALHOST,
        )
        .unwrap()
    }
    #[test]
    fn reservations_are_atomic_and_ports_never_reused_even_after_cancel() {
        let now = Instant::now();
        let (scope, roster, _, _) = fixture(2, now);
        let mut pool = pool(4);
        let a = pool.reserve(scope, &roster, limits(), 4, now).unwrap();
        let first: Vec<_> = a.routes().map(|(_, _, e)| e).collect();
        assert_eq!(pool.remaining(), 2);
        drop(a);
        for endpoint in &first {
            assert!(UdpSocket::bind(endpoint).is_err());
        }
        let mut bad = limits();
        bad.pending_messages = 0;
        assert!(pool.reserve(scope, &roster, bad, 4, now).is_err());
        assert_eq!(pool.remaining(), 2);
        assert!(
            pool.reserve(
                Scope {
                    round: scope.round + 256,
                    ..scope
                },
                &roster,
                limits(),
                4,
                now
            )
            .is_err()
        );
        assert!(
            pool.reserve(
                Scope {
                    service_generation: 2,
                    round: 5,
                    ..scope
                },
                &roster,
                limits(),
                4,
                now
            )
            .is_err()
        );
        assert_eq!(pool.remaining(), 2);
        let b = pool
            .reserve(Scope { round: 5, ..scope }, &roster, limits(), 4, now)
            .unwrap();
        assert!(b.routes().all(|(_, _, e)| !first.contains(&e)));
        assert_eq!(pool.remaining(), 0);
        assert!(pool.reserve(scope, &roster, limits(), 4, now).is_err());
    }
    #[test]
    fn pump_drops_prepared_and_oversize_bytes_and_retirement_stops_sending() {
        let now = Instant::now();
        let (scope, roster, _, _) = fixture(2, now);
        let mut pool = pool(2);
        let mut relay = pool.reserve(scope, &roster, limits(), 1, now).unwrap();
        let endpoints: Vec<_> = relay.routes().map(|(_, _, e)| e).collect();
        let clients = [
            UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap(),
            UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap(),
        ];
        for c in &clients {
            c.set_read_timeout(Some(Duration::from_millis(200)))
                .unwrap();
        }
        clients[0].send_to(b"before", endpoints[0]).unwrap();
        relay.activate(scope, now).unwrap();
        assert_eq!(relay.pump(scope, now).unwrap(), PumpStats::default());
        clients[0].send_to(&[9; 512], endpoints[0]).unwrap();
        assert_eq!(
            relay.pump(scope, now).unwrap(),
            PumpStats {
                received: 1,
                dropped: 1,
                ..PumpStats::default()
            }
        );
        for (i, c) in clients.iter().enumerate() {
            c.send_to(&[i as u8], endpoints[i]).unwrap();
        }
        let first = relay.pump(scope, now).unwrap();
        let second = relay.pump(scope, now).unwrap();
        assert_eq!(first.accepted + second.accepted, 2);
        assert_eq!(first.sent + second.sent, 2);
        let mut buf = [0; 32];
        for (i, c) in clients.iter().enumerate() {
            let (n, from) = c.recv_from(&mut buf).unwrap();
            assert_eq!(&buf[..n], &[1 - i as u8]);
            assert_eq!(from, SocketAddr::V4(endpoints[i]));
        }
        relay.invalidate();
        clients[0].send_to(b"retired", endpoints[0]).unwrap();
        assert_eq!(relay.pump(scope, now).unwrap(), PumpStats::default());
        assert!(clients[1].recv_from(&mut buf).is_err());
    }

    #[test]
    fn old_port_datagram_cannot_enter_new_reservation_with_current_positive_control() {
        let now = Instant::now();
        let (scope, roster, _, _) = fixture(2, now);
        let mut pool = pool(4);
        let mut old = pool.reserve(scope, &roster, limits(), 4, now).unwrap();
        let old_endpoint = old.routes().next().unwrap().2;
        old.activate(scope, now).unwrap();
        old.invalidate();
        drop(old);
        let next = Scope {
            round: scope.round + 1,
            ..scope
        };
        let mut current = pool.reserve(next, &roster, limits(), 4, now).unwrap();
        let endpoints: Vec<_> = current.routes().map(|(_, _, e)| e).collect();
        current.activate(next, now).unwrap();
        let clients = [
            UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap(),
            UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap(),
        ];
        clients[0]
            .send_to(b"exact-old-datagram", old_endpoint)
            .unwrap();
        assert_eq!(current.pump(next, now).unwrap(), PumpStats::default());
        for (i, c) in clients.iter().enumerate() {
            c.set_read_timeout(Some(Duration::from_millis(200)))
                .unwrap();
            c.send_to(&[i as u8], endpoints[i]).unwrap();
        }
        assert_eq!(
            current.pump(next, now).unwrap(),
            PumpStats {
                received: 2,
                accepted: 2,
                dropped: 0,
                sent: 2
            }
        );
        let mut buffer = [0; 32];
        for (i, c) in clients.iter().enumerate() {
            let (n, from) = c.recv_from(&mut buffer).unwrap();
            assert_eq!(&buffer[..n], &[1 - i as u8]);
            assert_eq!(from, SocketAddr::V4(endpoints[i]));
        }
        assert!(current.pump(scope, now).is_err());
        assert_eq!(current.pump(next, now).unwrap(), PumpStats::default());
        assert!(current.activate(next, now).is_err());
    }
}
