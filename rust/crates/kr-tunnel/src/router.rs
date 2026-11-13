//! Account-owned route/round kernel. Caller supplies current authenticated
//! owners and rechecks DB/channel membership before activation and every pump.
//! No native payload field grants authority. No sockets, tasks or blocking I/O.
use crate::{MAX_NATIVE, Packet, ReplayWindow, RouteId};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    io,
    net::{Ipv4Addr, SocketAddrV4},
    time::{Duration, Instant},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Owner {
    pub player: u32,
    pub session: u64,
}
impl Owner {
    fn valid(self) -> bool {
        self.player != 0 && self.session != 0
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Scope {
    pub service: [u8; 16],
    pub room: u32,
    pub room_session: u16,
    pub room_generation: u64,
    pub round: u64,
}
impl Scope {
    fn valid(self) -> bool {
        self.service != [0; 16]
            && self.room != 0
            && self.room_session != 0
            && self.room_generation != 0
            && self.round != 0
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Member {
    pub owner: Owner,
    pub connection: u64,
    pub visit: u64,
    pub slot: u8,
}
impl Member {
    fn valid(self) -> bool {
        self.owner.valid() && self.connection != 0 && self.visit != 0 && self.slot < 26
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Assignment {
    pub slot: u8,
    pub route: RouteId,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Binding {
    pub route: RouteId,
    pub address: SocketAddrV4,
}
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub routes: usize,
    pub queue: usize,
    pub sends: usize,
    pub pending: Duration,
    pub lease: Duration,
}
impl Limits {
    fn valid(self) -> bool {
        (2..=4096).contains(&self.routes)
            && (1..=64).contains(&self.queue)
            && (1..=4096).contains(&self.sends)
            && self.pending > Duration::ZERO
            && self.pending <= Duration::from_secs(5)
            && self.lease >= self.pending
            && self.lease <= Duration::from_secs(3600)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Denied {
    Stale,
    Owner,
    Phase,
    Size,
    Replay,
    Queue,
    Clock,
}
#[derive(Default, Debug, PartialEq, Eq)]
pub struct Stats {
    pub sent: usize,
    pub expired: usize,
}
struct Queued {
    at: Instant,
    sequence: u64,
    native: Vec<u8>,
}
struct Edge {
    scope: Scope,
    source: Member,
    destination: Member,
    reverse: RouteId,
    window: ReplayWindow,
    pending: VecDeque<Queued>,
}
struct Round {
    members: Vec<Member>,
    active: bool,
    created: Instant,
    last: Instant,
    bindings: HashMap<Owner, Vec<Binding>>,
    order: Vec<RouteId>,
    cursor: usize,
}
pub struct Router {
    limits: Limits,
    rounds: HashMap<Scope, Round>,
    edges: HashMap<RouteId, Edge>,
}
fn denied() -> io::Error {
    io::Error::other("invalid or stale authenticated route reservation")
}
impl Router {
    pub fn new(limits: Limits) -> io::Result<Self> {
        if !limits.valid() {
            return Err(denied());
        }
        Ok(Self {
            limits,
            rounds: HashMap::new(),
            edges: HashMap::new(),
        })
    }
    pub fn remaining(&self) -> usize {
        self.limits.routes - self.edges.len()
    }
    /// Caller reserves all control queues and rolls back this scope on any
    /// publish failure. A prepared scope never forwards data before every peer
    /// acknowledges its exact route set and the room loading barrier opens.
    pub fn prepare(
        &mut self,
        scope: Scope,
        members: &[Member],
        now: Instant,
    ) -> io::Result<Vec<(Owner, Vec<Assignment>)>> {
        if !scope.valid()
            || self.rounds.contains_key(&scope)
            || !(2..=26).contains(&members.len())
            || members.iter().any(|m| !m.valid())
        {
            return Err(denied());
        }
        let count = members.len() * (members.len() - 1);
        if count > self.remaining() {
            return Err(io::Error::other("authenticated route pool exhausted"));
        }
        let mut owners = HashSet::new();
        let mut slots = HashSet::new();
        for m in members {
            if !owners.insert(m.owner)
                || !slots.insert(m.slot)
                || self
                    .rounds
                    .values()
                    .any(|r| r.members.iter().any(|old| old.owner == m.owner))
            {
                return Err(denied());
            }
        }
        let mut rows = HashMap::new();
        let mut allocated = HashSet::new();
        for source in members {
            for destination in members {
                if source.owner == destination.owner {
                    continue;
                }
                let id = RouteId::random()?;
                if self.edges.contains_key(&id) || !allocated.insert(id) {
                    return Err(io::Error::other("route identity entropy collision"));
                }
                rows.insert((source.owner, destination.owner), id);
            }
        }
        let mut assignments = Vec::new();
        let mut order = Vec::with_capacity(count);
        for source in members {
            let mut assignment = Vec::new();
            for destination in members {
                if source.owner == destination.owner {
                    continue;
                }
                let id = rows[&(source.owner, destination.owner)];
                let reverse = rows[&(destination.owner, source.owner)];
                order.push(id);
                self.edges.insert(
                    id,
                    Edge {
                        scope,
                        source: *source,
                        destination: *destination,
                        reverse,
                        window: ReplayWindow::default(),
                        pending: VecDeque::new(),
                    },
                );
                assignment.push(Assignment {
                    slot: destination.slot,
                    route: id,
                });
            }
            assignment.sort_by_key(|row| row.slot);
            assignments.push((source.owner, assignment));
        }
        self.rounds.insert(
            scope,
            Round {
                members: members.to_vec(),
                active: false,
                created: now,
                last: now,
                bindings: HashMap::new(),
                order,
                cursor: 0,
            },
        );
        Ok(assignments)
    }
    pub fn acknowledge(
        &mut self,
        scope: Scope,
        owner: Owner,
        bindings: &[Binding],
        now: Instant,
    ) -> io::Result<()> {
        let round = self.rounds.get(&scope).ok_or_else(denied)?;
        if round.active
            || now < round.last
            || now.duration_since(round.created) >= self.limits.lease
            || round.bindings.contains_key(&owner)
            || !round.members.iter().any(|m| m.owner == owner)
        {
            return Err(denied());
        }
        let expected: HashSet<_> = self
            .edges
            .iter()
            .filter(|(_, edge)| edge.scope == scope && edge.source.owner == owner)
            .map(|(id, _)| *id)
            .collect();
        let mut seen = HashSet::new();
        let mut addresses = HashSet::new();
        for binding in bindings {
            if !binding.address.ip().is_loopback()
                || binding.address.port() == 0
                || !expected.contains(&binding.route)
                || !seen.insert(binding.route)
                || !addresses.insert(binding.address)
            {
                return Err(denied());
            }
        }
        if seen != expected {
            return Err(denied());
        }
        let round = self.rounds.get_mut(&scope).expect("checked round");
        round.bindings.insert(owner, bindings.to_vec());
        round.last = now;
        Ok(())
    }
    pub fn ready(&self, scope: Scope) -> bool {
        self.rounds
            .get(&scope)
            .is_some_and(|r| r.members.iter().all(|m| r.bindings.contains_key(&m.owner)))
    }
    pub fn endpoints(&self, scope: Scope, owner: Owner) -> io::Result<Vec<(u8, u32, u16)>> {
        let round = self.rounds.get(&scope).ok_or_else(denied)?;
        let bindings = round.bindings.get(&owner).ok_or_else(denied)?;
        let mut result = Vec::new();
        if !self.ready(scope) {
            return Err(denied());
        }
        for binding in bindings {
            let edge = self.edges.get(&binding.route).ok_or_else(denied)?;
            if edge.scope != scope || edge.source.owner != owner {
                return Err(denied());
            }
            result.push((
                edge.destination.slot,
                u32::from(*binding.address.ip()),
                binding.address.port(),
            ));
        }
        result.sort_by_key(|row| row.0);
        Ok(result)
    }
    pub fn activate(&mut self, scope: Scope, current: &[Member], now: Instant) -> io::Result<()> {
        if !self.ready(scope) {
            return Err(denied());
        }
        let round = self.rounds.get_mut(&scope).ok_or_else(denied)?;
        if round.active
            || round.members != current
            || now < round.last
            || now.duration_since(round.created) >= self.limits.lease
        {
            return Err(denied());
        }
        round.active = true;
        round.last = now;
        Ok(())
    }
    /// Departure removes only routes touching that exact owner. Survivors keep
    /// their current route identifiers and native source endpoints. No joins or
    /// replacement member may enter a round through this subset transition.
    pub fn retain(&mut self, scope: Scope, current: &[Member]) -> io::Result<()> {
        let round = self.rounds.get_mut(&scope).ok_or_else(denied)?;
        if !round.active
            || current.iter().any(|m| !round.members.contains(m))
            || current
                .iter()
                .map(|m| m.owner)
                .collect::<HashSet<_>>()
                .len()
                != current.len()
        {
            return Err(denied());
        }
        round.members = current.to_vec();
        round
            .bindings
            .retain(|owner, _| current.iter().any(|m| m.owner == *owner));
        self.edges.retain(|_, edge| {
            edge.scope != scope
                || (current.contains(&edge.source) && current.contains(&edge.destination))
        });
        if current.len() < 2 {
            self.retire(scope);
        }
        Ok(())
    }
    pub fn ingest(&mut self, owner: Owner, packet: Packet, now: Instant) -> Result<(), Denied> {
        let edge = self.edges.get_mut(&packet.route).ok_or(Denied::Stale)?;
        if edge.source.owner != owner {
            return Err(Denied::Owner);
        }
        let round = self.rounds.get_mut(&edge.scope).ok_or(Denied::Stale)?;
        if now < round.last {
            return Err(Denied::Clock);
        }
        round.last = now;
        if !round.active || now.duration_since(round.created) >= self.limits.lease {
            return Err(Denied::Phase);
        }
        if packet.native.is_empty() || packet.native.len() > MAX_NATIVE {
            return Err(Denied::Size);
        }
        if edge.pending.len() >= self.limits.queue {
            return Err(Denied::Queue);
        }
        if !edge.window.admit(packet.sequence) {
            return Err(Denied::Replay);
        }
        edge.pending.push_back(Queued {
            at: now,
            sequence: packet.sequence,
            native: packet.native,
        });
        Ok(())
    }
    /// Call under the room owner lock, after external asynchronous account/DB
    /// checks and a fresh exact membership comparison. Sends must not block or
    /// acquire a room lock; false means backpressure with pending TTL retained.
    pub fn drain(
        &mut self,
        scope: Scope,
        current: &[Member],
        now: Instant,
        mut send: impl FnMut(Owner, Packet) -> io::Result<bool>,
    ) -> io::Result<Stats> {
        let round = self.rounds.get_mut(&scope).ok_or_else(denied)?;
        if !round.active
            || round.members != current
            || now < round.last
            || now.duration_since(round.created) >= self.limits.lease
        {
            return Err(denied());
        }
        round.last = now;
        let mut stats = Stats::default();
        // At most one nonexpired message per directed edge in a pass, with a
        // rotating cursor. Hot or blocked edges cannot monopolize every pump.
        for _ in 0..round.order.len() {
            let id = round.order[round.cursor];
            round.cursor = (round.cursor + 1) % round.order.len();
            let Some(edge) = self.edges.get_mut(&id) else {
                continue;
            };
            while let Some(front) = edge.pending.front() {
                if now.duration_since(front.at) >= self.limits.pending {
                    edge.pending.pop_front();
                    stats.expired += 1;
                    continue;
                }
                if !send(
                    edge.destination.owner,
                    Packet {
                        route: edge.reverse,
                        sequence: front.sequence,
                        native: front.native.clone(),
                    },
                )? {
                    break;
                }
                edge.pending.pop_front();
                stats.sent += 1;
                break;
            }
            if stats.sent == self.limits.sends {
                break;
            }
        }
        Ok(stats)
    }
    pub fn retire(&mut self, scope: Scope) {
        self.rounds.remove(&scope);
        self.edges.retain(|_, edge| edge.scope != scope);
    }
    pub fn disconnect(&mut self, owner: Owner) {
        self.edges
            .retain(|_, edge| edge.source.owner != owner && edge.destination.owner != owner);
        for round in self.rounds.values_mut() {
            round.members.retain(|m| m.owner != owner);
            round.bindings.remove(&owner);
        }
        let retired: Vec<_> = self
            .rounds
            .iter()
            .filter(|(_, round)| round.members.len() < 2)
            .map(|(scope, _)| *scope)
            .collect();
        for scope in retired {
            self.retire(scope);
        }
    }
    pub fn round_count(&self) -> usize {
        self.rounds.len()
    }
    pub fn owners(&self, scope: Scope) -> Vec<Owner> {
        self.rounds
            .get(&scope)
            .map(|r| r.members.iter().map(|m| m.owner).collect())
            .unwrap_or_default()
    }
}

/// Helpers preserve the native loopback address as data, not a public endpoint.
pub fn loopback(port: u16) -> SocketAddrV4 {
    SocketAddrV4::new(Ipv4Addr::LOCALHOST, port)
}

#[cfg(test)]
mod tests;
