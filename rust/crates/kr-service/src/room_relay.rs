//! Opt-in relay ownership; no original-server authority or native proof claim.
//! Socket binding happens before installing this owner, outside the room mutex.
use crate::{
    account_bridge::RelayAuthority,
    room::{Membership, Phase, Room},
    room_registry::RoomKey,
    udp_relay::{self, Limits, Participant, PumpStats, RoundSockets, Scope, SocketPool},
};
use std::{io, net::Ipv4Addr, time::Instant};

pub(crate) struct Owner {
    pool: SocketPool,
    base: Scope,
    limits: Limits,
    receive_budget: usize,
    lease: Option<Lease>,
}
pub(crate) struct Lease {
    scope: Scope,
    participants: Vec<Participant>,
    authorities: Vec<RelayAuthority>,
    sockets: RoundSockets,
    active: bool,
}
impl Lease {
    fn matches(&self, model: &Room) -> bool {
        matches!(model.phase,Phase::Loading {round,..}|Phase::Armed {round} if round==self.scope.round)
            && model.members.len() == self.participants.len()
            && self.participants.iter().enumerate().all(|(slot, p)| {
                model
                    .members
                    .get(&p.member.identity)
                    .is_some_and(|m| m.key == p.member && usize::from(m.slot) == slot)
            })
            && self.authorities.iter().all(RelayAuthority::current)
    }
    pub(crate) fn endpoints(&self, member: Membership) -> io::Result<Vec<(u32, u16)>> {
        let p = self
            .participants
            .iter()
            .find(|p| p.member == member)
            .ok_or_else(|| io::Error::other("foreign relay recipient"))?;
        self.sockets.endpoints_for(self.scope, *p)
    }
}
impl Owner {
    pub(crate) fn new(cfg: &udp_relay::Config, key: RoomKey) -> io::Result<Self> {
        let limits = cfg.limits()?;
        let pool = SocketPool::bind(cfg.bind, cfg.advertise, &cfg.ports)?;
        let mut bytes = [0; 8];
        getrandom::fill(&mut bytes).map_err(io::Error::other)?;
        let generation = u64::from_le_bytes(bytes);
        if generation == 0 {
            return Err(io::Error::other("relay service generation entropy"));
        }
        Ok(Self {
            pool,
            base: Scope {
                service_generation: generation,
                room: key,
                round: 0,
            },
            limits,
            receive_budget: cfg.receive_budget,
            lease: None,
        })
    }
    pub(crate) fn prepare(
        &mut self,
        model: &Room,
        rows: Vec<(u8, Membership, Ipv4Addr, RelayAuthority)>,
    ) -> io::Result<Lease> {
        let Phase::Loading { round, .. } = model.phase else {
            return Err(io::Error::other("relay requires loading reservation"));
        };
        let mut slots = vec![None; model.capacity()];
        for (slot, member, source_ip, authority) in rows {
            if !authority.owns(member.identity, member.connection) || !model.current(member) {
                return Err(io::Error::other(
                    "relay participant has no current account owner",
                ));
            }
            let entry = slots
                .get_mut(usize::from(slot))
                .ok_or_else(|| io::Error::other("relay slot out of range"))?;
            if entry.is_some() {
                return Err(io::Error::other("duplicate relay slot"));
            }
            *entry = Some((
                Participant {
                    member,
                    session_generation: authority.generation(),
                    source_ip,
                },
                authority,
            ));
        }
        let slots: Vec<_> = slots
            .into_iter()
            .collect::<Option<_>>()
            .ok_or_else(|| io::Error::other("incomplete relay roster"))?;
        let (participants, authorities): (Vec<_>, Vec<_>) = slots.into_iter().unzip();
        let scope = Scope { round, ..self.base };
        let sockets = self.pool.reserve(
            scope,
            &participants,
            self.limits,
            self.receive_budget,
            Instant::now(),
        )?;
        Ok(Lease {
            scope,
            participants,
            authorities,
            sockets,
            active: false,
        })
    }
    pub(crate) fn install(&mut self, lease: Lease) {
        self.invalidate();
        self.lease = Some(lease);
    }
    pub(crate) fn endpoints(&self, member: Membership) -> io::Result<Vec<(u32, u16)>> {
        self.lease
            .as_ref()
            .ok_or_else(|| io::Error::other("missing relay reservation"))?
            .endpoints(member)
    }
    /// Called after reserving every TCP recipient queue and before publishing
    /// model/batches. Any ownership/phase change retires the UDP lease first.
    pub(crate) fn transition(&mut self, next: &Room) -> io::Result<()> {
        if self.lease.as_ref().is_some_and(|l| !l.matches(next)) {
            self.invalidate();
        }
        if matches!(next.phase, Phase::Armed { .. }) {
            let lease = self
                .lease
                .as_mut()
                .ok_or_else(|| io::Error::other("missing current relay lease"))?;
            if !lease.active {
                lease.sockets.activate(lease.scope, Instant::now())?;
                lease.active = true;
            }
        }
        Ok(())
    }
    pub(crate) fn authorization(&self, model: &Room) -> Option<(Scope, Vec<RelayAuthority>)> {
        self.lease
            .as_ref()
            .filter(|l| l.active && l.matches(model) && matches!(model.phase, Phase::Armed { .. }))
            .map(|l| (l.scope, l.authorities.clone()))
    }
    pub(crate) fn invalidate(&mut self) {
        if let Some(mut lease) = self.lease.take() {
            lease.sockets.invalidate();
        }
    }
    pub(crate) fn invalidate_scope(&mut self, scope: Scope) {
        if self.lease.as_ref().is_some_and(|l| l.scope == scope) {
            self.invalidate();
        }
    }
    /// Revalidate under the room owner mutex after every asynchronous DB check.
    pub(crate) fn pump(&mut self, scope: Scope, model: &Room) -> io::Result<PumpStats> {
        let Some(lease) = self.lease.as_mut().filter(|l| l.scope == scope) else {
            return Ok(PumpStats::default());
        };
        if !lease.active || !lease.matches(model) {
            self.invalidate();
            return Ok(PumpStats::default());
        }
        lease.sockets.pump(scope, Instant::now())
    }
}
