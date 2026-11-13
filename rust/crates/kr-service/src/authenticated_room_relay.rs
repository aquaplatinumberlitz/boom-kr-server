//! Bridge between the synchronous room owner and authenticated companion hub.
//! No native payload grants routing authority; every pump follows DB/member checks.
use crate::{
    account_bridge::RelayAuthority,
    authenticated_peer::Hub,
    room::{Membership, Phase, Room},
    room_registry::RoomKey,
};
use kr_tunnel::router::{Member, Owner as AccountOwner, Scope};
use std::{io, net::Ipv4Addr, sync::Arc};
struct Participant {
    member: Member,
    key: Membership,
    authority: RelayAuthority,
}
struct Lease {
    scope: Scope,
    participants: Vec<Participant>,
    active: bool,
}
pub(crate) struct Owner {
    hub: Arc<Hub>,
    room: RoomKey,
    lease: Option<Lease>,
}
fn invalid() -> io::Error {
    io::Error::other("invalid authenticated room relay ownership")
}
impl Owner {
    pub(crate) fn new(hub: Arc<Hub>, room: RoomKey) -> Self {
        Self {
            hub,
            room,
            lease: None,
        }
    }
    pub(crate) fn prepare(
        &mut self,
        model: &Room,
        rows: Vec<(u8, Membership, Ipv4Addr, RelayAuthority)>,
    ) -> io::Result<()> {
        let Phase::Loading { round, .. } = model.phase else {
            return Err(invalid());
        };
        self.invalidate();
        let mut participants = Vec::new();
        for (slot, key, _source_ip, authority) in rows {
            if !model.current(key) || !authority.owns(key.identity, key.connection) {
                return Err(invalid());
            }
            participants.push(Participant {
                member: Member {
                    owner: AccountOwner {
                        player: key.identity,
                        session: authority.generation(),
                    },
                    connection: key.connection,
                    visit: key.visit,
                    slot,
                },
                key,
                authority,
            });
        }
        participants.sort_by_key(|p| p.member.slot);
        let scope = Scope {
            service: self.hub.generation,
            room: self.room.id.0,
            room_session: self.room.session,
            room_generation: self.room.generation,
            round,
        };
        self.hub.prepare(
            scope,
            &participants.iter().map(|p| p.member).collect::<Vec<_>>(),
        )?;
        self.lease = Some(Lease {
            scope,
            participants,
            active: false,
        });
        Ok(())
    }
    fn current(hub: &Hub, lease: &Lease, model: &Room) -> io::Result<Vec<Member>> {
        let mut members = Vec::new();
        for current in model.members.values() {
            let p = lease
                .participants
                .iter()
                .find(|p| p.key == current.key && p.member.slot == current.slot)
                .ok_or_else(invalid)?;
            // Committed account denial invalidates its bridge generation even
            // before TCP teardown removes membership. It cannot keep a UDP route.
            if p.authority.current() && hub.connected(p.member.owner) {
                members.push(p.member);
            }
        }
        members.sort_by_key(|m| m.slot);
        Ok(members)
    }
    pub(crate) fn transition(&mut self, model: &Room) -> io::Result<()> {
        let Some(lease) = self.lease.as_mut() else {
            return if matches!(model.phase, Phase::Loading { .. } | Phase::Armed { .. }) {
                Err(invalid())
            } else {
                Ok(())
            };
        };
        if !matches!(model.phase,Phase::Loading {round,..}|Phase::Armed {round}if round==lease.scope.round)
        {
            self.invalidate();
            return Ok(());
        }
        let current = Self::current(&self.hub, lease, model)?;
        if matches!(model.phase, Phase::Loading { .. }) {
            if current.len() != lease.participants.len() {
                self.invalidate();
                return Err(invalid());
            }
            return Ok(());
        }
        if !lease.active {
            self.hub.activate(lease.scope, &current)?;
            lease.active = true;
        } else if current.len() >= 2 {
            self.hub.retain(lease.scope, &current)?;
        } else {
            self.hub.retire(lease.scope);
        }
        Ok(())
    }
    pub(crate) fn ready(&self) -> bool {
        self.lease
            .as_ref()
            .is_some_and(|lease| self.hub.ready(lease.scope))
    }
    pub(crate) fn decorate(
        &self,
        key: Membership,
        mut direct: Vec<(u8, u32, u16)>,
    ) -> io::Result<Vec<(u8, u32, u16)>> {
        let lease = self.lease.as_ref().ok_or_else(invalid)?;
        let own = lease
            .participants
            .iter()
            .find(|p| p.key == key)
            .ok_or_else(invalid)?;
        let remote = self.hub.endpoints(lease.scope, own.member.owner)?;
        for row in &mut direct {
            if row.0 == own.member.slot {
                continue;
            }
            let endpoint = remote
                .iter()
                .find(|endpoint| endpoint.0 == row.0)
                .ok_or_else(invalid)?;
            *row = *endpoint;
        }
        if direct.len() != remote.len() + 1 {
            return Err(invalid());
        }
        Ok(direct)
    }
    pub(crate) fn authorization(&self, model: &Room) -> Option<(Scope, Vec<RelayAuthority>)> {
        let lease = self.lease.as_ref()?;
        if !lease.active || !matches!(model.phase,Phase::Armed {round}if round==lease.scope.round) {
            return None;
        }
        let current = Self::current(&self.hub, lease, model).ok()?;
        if current.len() < 2 {
            return None;
        }
        Some((
            lease.scope,
            lease
                .participants
                .iter()
                .filter(|p| current.contains(&p.member))
                .map(|p| p.authority.clone())
                .collect(),
        ))
    }
    pub(crate) fn pump(
        &self,
        scope: Scope,
        model: &Room,
    ) -> io::Result<crate::udp_relay::PumpStats> {
        let Some(lease) = self
            .lease
            .as_ref()
            .filter(|lease| lease.active && lease.scope == scope)
        else {
            return Ok(crate::udp_relay::PumpStats::default());
        };
        if !matches!(model.phase,Phase::Armed {round}if round==scope.round) {
            return Ok(crate::udp_relay::PumpStats::default());
        }
        let current = Self::current(&self.hub, lease, model)?;
        if current.len() < 2 {
            self.hub.retire(scope);
            return Ok(crate::udp_relay::PumpStats::default());
        }
        self.hub.retain(scope, &current)?;
        self.hub.pump(scope, &current)
    }
    pub(crate) fn invalidate(&mut self) {
        if let Some(lease) = self.lease.take() {
            self.hub.retire(lease.scope);
        }
    }
}
impl Drop for Owner {
    fn drop(&mut self) {
        self.invalidate();
    }
}
