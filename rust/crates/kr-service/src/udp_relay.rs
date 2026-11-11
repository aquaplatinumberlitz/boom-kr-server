//! Experimental relay queue/route kernel (design candidate, not KR compatibility).
//! No serving listener or endpoint policy is enabled by this module. A trusted
//! room/session owner must activate and invalidate it; source IP is not auth.
use crate::{room::Membership, room_registry::RoomKey};
use std::{
    collections::{HashSet, VecDeque},
    io,
    net::{Ipv4Addr, SocketAddrV4},
    time::{Duration, Instant},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Scope {
    pub service_generation: u64,
    pub room: RoomKey,
    pub round: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Participant {
    pub member: Membership,
    pub session_generation: u64,
    pub source_ip: Ipv4Addr,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RouteKey {
    pub scope: Scope,
    pub source: Participant,
    pub destination: Participant,
}
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub max_routes: usize,
    pub max_datagram: usize,
    pub pending_messages: usize,
    pub pending_bytes: usize,
    pub rate_messages: usize,
    pub rate_window: Duration,
    pub pending_ttl: Duration,
    pub lease_ttl: Duration,
    pub sends_per_drain: usize,
}
impl Limits {
    fn valid(self) -> bool {
        (1..=4096).contains(&self.max_routes)
            && (1..=255).contains(&self.max_datagram)
            && (1..=256).contains(&self.pending_messages)
            && (self.max_datagram..=65_536).contains(&self.pending_bytes)
            && (1..=4096).contains(&self.rate_messages)
            && !self.rate_window.is_zero()
            && self.rate_window <= Duration::from_secs(60)
            && !self.pending_ttl.is_zero()
            && self.pending_ttl <= Duration::from_secs(10)
            && self.lease_ttl >= self.pending_ttl
            && self.lease_ttl <= Duration::from_secs(3600)
            && (1..=4096).contains(&self.sends_per_drain)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DropReason {
    Inactive,
    StaleRoute,
    Clock,
    Source,
    Rebinding,
    Size,
    Rate,
    Queue,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    Prepared,
    Active,
    Retired,
}
struct Pending {
    bytes: Vec<u8>,
    received: Instant,
}
struct Edge {
    key: RouteKey,
    advertised: SocketAddrV4,
    learned: Option<SocketAddrV4>,
    pending: VecDeque<Pending>,
    bytes: usize,
    rate_start: Instant,
    rate_count: usize,
    reverse: usize,
}
pub struct Kernel {
    scope: Scope,
    roster: Vec<Participant>,
    edges: Vec<Edge>,
    limits: Limits,
    state: State,
    created: Instant,
    last_time: Instant,
    cursor: usize,
}
fn invalid() -> io::Error {
    io::Error::other("invalid relay reservation")
}
fn unicast(ip: Ipv4Addr) -> bool {
    !ip.is_unspecified() && !ip.is_multicast() && !ip.is_broadcast()
}
impl Kernel {
    /// Caller supplies a current, already-authorized roster in slot order and
    /// separately reserved endpoints in source-major order, excluding self.
    pub fn prepare(
        scope: Scope,
        roster: &[Participant],
        endpoints: &[SocketAddrV4],
        limits: Limits,
        now: Instant,
    ) -> io::Result<Self> {
        let n = roster.len();
        let count = n.checked_mul(n.saturating_sub(1)).ok_or_else(invalid)?;
        if !limits.valid()
            || !(2..=26).contains(&n)
            || count > limits.max_routes
            || endpoints.len() != count
            || scope.service_generation == 0
            || scope.room.id.0 == 0
            || scope.room.session == 0
            || scope.room.generation == 0
            || scope.round == 0
        {
            return Err(invalid());
        }
        let mut players = HashSet::new();
        let mut connections = HashSet::new();
        for p in roster {
            if p.member.identity == 0
                || p.member.connection == 0
                || p.member.visit == 0
                || p.session_generation == 0
                || !unicast(p.source_ip)
                || !players.insert(p.member.identity)
                || !connections.insert(p.member.connection)
            {
                return Err(invalid());
            }
        }
        let mut distinct = HashSet::new();
        if endpoints
            .iter()
            .any(|p| p.port() == 0 || !unicast(*p.ip()) || !distinct.insert(*p))
        {
            return Err(invalid());
        }
        let mut edges = Vec::with_capacity(count);
        let mut indices = vec![vec![None; n]; n];
        for (source, participant) in roster.iter().enumerate() {
            for (destination, target) in roster.iter().enumerate() {
                if source == destination {
                    continue;
                }
                indices[source][destination] = Some(edges.len());
                edges.push(Edge {
                    key: RouteKey {
                        scope,
                        source: *participant,
                        destination: *target,
                    },
                    advertised: endpoints[edges.len()],
                    learned: None,
                    pending: VecDeque::new(),
                    bytes: 0,
                    rate_start: now,
                    rate_count: 0,
                    reverse: 0,
                });
            }
        }
        for (i, row) in indices.iter().enumerate() {
            for (j, index) in row.iter().enumerate() {
                if let Some(index) = index {
                    edges[*index].reverse = indices[j][i].expect("complete directed graph");
                }
            }
        }
        Ok(Self {
            scope,
            roster: roster.to_vec(),
            edges,
            limits,
            state: State::Prepared,
            created: now,
            last_time: now,
            cursor: 0,
        })
    }
    pub fn routes(&self) -> impl Iterator<Item = (usize, RouteKey, SocketAddrV4)> + '_ {
        self.edges
            .iter()
            .enumerate()
            .map(|(i, e)| (i, e.key, e.advertised))
    }
    /// Slot-order endpoint rows for the existing 153 builder. The caller must
    /// bind roster order to actual room slots and validate current ownership.
    /// This snapshot alone does not authorize publication or sending.
    pub fn endpoints_for(
        &self,
        scope: Scope,
        recipient: Participant,
    ) -> io::Result<Vec<(u32, u16)>> {
        if self.state == State::Retired || self.scope != scope || !self.roster.contains(&recipient)
        {
            return Err(invalid());
        }
        let mut endpoints = vec![(0, 0); self.roster.len()];
        for edge in self
            .edges
            .iter()
            .filter(|edge| edge.key.source == recipient)
        {
            let slot = self
                .roster
                .iter()
                .position(|p| *p == edge.key.destination)
                .ok_or_else(invalid)?;
            endpoints[slot] = (u32::from(*edge.advertised.ip()), edge.advertised.port());
        }
        Ok(endpoints)
    }
    /// Must be called only after owner revalidates its room/session reservation.
    pub fn activate(&mut self, scope: Scope, now: Instant) -> io::Result<()> {
        if self.state != State::Prepared
            || scope != self.scope
            || now < self.last_time
            || now.duration_since(self.created) >= self.limits.lease_ttl
        {
            return Err(invalid());
        }
        self.state = State::Active;
        self.last_time = now;
        Ok(())
    }
    /// Irreversible. Caller must serialize this before retiring any owner lease.
    pub fn invalidate(&mut self) {
        self.state = State::Retired;
        for edge in &mut self.edges {
            edge.learned = None;
            edge.pending.clear();
            edge.bytes = 0;
        }
    }
    fn advance(&mut self, now: Instant) -> Result<(), DropReason> {
        if now < self.last_time {
            return Err(DropReason::Clock);
        }
        self.last_time = now;
        if now.duration_since(self.created) >= self.limits.lease_ttl {
            self.invalidate();
        }
        if self.state != State::Active {
            return Err(DropReason::Inactive);
        }
        for edge in &mut self.edges {
            while edge
                .pending
                .front()
                .is_some_and(|p| now.duration_since(p.received) >= self.limits.pending_ttl)
            {
                edge.bytes -= edge.pending.pop_front().expect("front exists").bytes.len();
            }
        }
        Ok(())
    }
    /// A route's source IP constrains a candidate sender, never authenticates it.
    /// Tuple pinning is intentionally immutable; rebinding requires a new lease.
    pub fn ingest(
        &mut self,
        index: usize,
        key: RouteKey,
        from: SocketAddrV4,
        bytes: &[u8],
        now: Instant,
    ) -> Result<(), DropReason> {
        self.advance(now)?;
        let edge = self
            .edges
            .get_mut(index)
            .filter(|e| e.key == key)
            .ok_or(DropReason::StaleRoute)?;
        if *from.ip() != edge.key.source.source_ip || from.port() == 0 {
            return Err(DropReason::Source);
        }
        if edge.learned.is_some_and(|p| p != from) {
            return Err(DropReason::Rebinding);
        }
        if bytes.is_empty() || bytes.len() > self.limits.max_datagram {
            return Err(DropReason::Size);
        }
        if now.duration_since(edge.rate_start) >= self.limits.rate_window {
            edge.rate_start = now;
            edge.rate_count = 0;
        }
        if edge.rate_count >= self.limits.rate_messages {
            return Err(DropReason::Rate);
        }
        if edge.pending.len() >= self.limits.pending_messages
            || bytes.len() > self.limits.pending_bytes.saturating_sub(edge.bytes)
        {
            return Err(DropReason::Queue);
        }
        edge.rate_count += 1;
        edge.learned = Some(from);
        edge.bytes += bytes.len();
        edge.pending.push_back(Pending {
            bytes: bytes.to_vec(),
            received: now,
        });
        Ok(())
    }
    /// Caller holds the kernel's owner mutex throughout this synchronous call.
    /// `send` must be nonblocking (e.g. UdpSocket::try_send_to), never await/call
    /// back into the room lock. It receives reverse socket index and original bytes.
    /// WouldBlock leaves the packet queued; other errors retire the whole lease.
    pub fn drain(
        &mut self,
        now: Instant,
        mut send: impl FnMut(usize, SocketAddrV4, &[u8]) -> io::Result<usize>,
    ) -> io::Result<usize> {
        if self.advance(now).is_err() {
            return Ok(0);
        }
        let mut sent = 0;
        let mut attempts = 0;
        let mut scanned = 0;
        while attempts < self.limits.sends_per_drain && scanned < self.edges.len() {
            let index = self.cursor;
            self.cursor = (self.cursor + 1) % self.edges.len();
            scanned += 1;
            let reverse = self.edges[index].reverse;
            let Some(target) = self.edges[reverse].learned else {
                continue;
            };
            let Some(packet) = self.edges[index].pending.front() else {
                continue;
            };
            attempts += 1;
            match send(reverse, target, &packet.bytes) {
                Ok(n) if n == packet.bytes.len() => {
                    self.edges[index].bytes -= self.edges[index]
                        .pending
                        .pop_front()
                        .expect("front exists")
                        .bytes
                        .len();
                    sent += 1;
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {}
                other => {
                    self.invalidate();
                    return Err(other
                        .err()
                        .unwrap_or_else(|| io::Error::other("short UDP send")));
                }
            }
        }
        Ok(sent)
    }
    pub fn pending(&self) -> (usize, usize) {
        self.edges
            .iter()
            .fold((0, 0), |(n, b), e| (n + e.pending.len(), b + e.bytes))
    }
}

#[cfg(test)]
mod tests;

mod sockets;
pub use sockets::{PumpStats, RoundSockets, SocketPool};
