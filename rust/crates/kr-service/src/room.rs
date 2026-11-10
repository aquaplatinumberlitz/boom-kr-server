//! Bounded two-player fixed-room model. Policy, not original KR authority.
//! Wire adapters reference CP028/034/035/229/230; no gameplay is implemented here.
//! Generation arguments fence internal work; raw585/99 lack a measured visit/round token.
//! Passing a current generation cannot prove an old wire ACK was rejected.
use std::{
    collections::{BTreeMap, BTreeSet},
    io,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Membership {
    pub identity: u32,
    pub connection: u64,
    pub visit: u64,
}
#[derive(Clone, Debug)]
pub struct Member {
    pub key: Membership,
    pub slot: u8,
    pub ready: bool,
    pub setup: bool,
    late_after_abort: u16,
    opaque123: u16,
    opaque167: u16,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Phase {
    Idle,
    Loading {
        round: u64,
        roster: Vec<Membership>,
        loaded: BTreeSet<Membership>,
    },
    Armed {
        round: u64,
    },
}
#[derive(Clone, Debug)]
pub struct Room {
    pub members: BTreeMap<u32, Member>,
    pub host: Option<u32>,
    pub phase: Phase,
    visit: u64,
    round: u64,
    aborted: Option<(u64, Vec<Membership>)>,
    pub(crate) pickups: crate::pickups::Round,
    pub(crate) bombs: crate::bombs::Ledger,
    pub(crate) bomb_capacity: [usize; 2],
    pub(crate) terrain: Option<crate::terrain::Terrain>,
}
impl Default for Room {
    fn default() -> Self {
        Self {
            members: BTreeMap::new(),
            host: None,
            phase: Phase::Idle,
            visit: 0,
            round: 0,
            aborted: None,
            pickups: crate::pickups::Round::default(),
            bombs: crate::bombs::Ledger::default(),
            bomb_capacity: [1, 2],
            terrain: None,
        }
    }
}
impl Room {
    pub fn current(&self, key: Membership) -> bool {
        self.members
            .get(&key.identity)
            .is_some_and(|m| m.key == key)
    }
    pub fn enter(
        &mut self,
        identity: u32,
        connection: u64,
        create: bool,
    ) -> io::Result<Membership> {
        if !(1..=2).contains(&identity)
            || self.phase != Phase::Idle
            || self.members.contains_key(&identity)
            || self
                .members
                .values()
                .any(|m| m.key.connection == connection)
        {
            return Err(io::Error::other(
                "identity/connection already in room or room not idle",
            ));
        }
        if create {
            if identity != 1 || self.host.is_some() || !self.members.is_empty() {
                return Err(io::Error::other("fixed room requires first host create"));
            }
        } else if self.host.is_none() || self.members.len() >= 2 {
            return Err(io::Error::other("no fixed room or full"));
        }
        let visit = self
            .visit
            .checked_add(1)
            .ok_or_else(|| io::Error::other("visit counter exhausted"))?;
        let key = Membership {
            identity,
            connection,
            visit,
        };
        self.visit = visit;
        self.members.insert(
            identity,
            Member {
                key,
                slot: (identity - 1) as u8,
                ready: false,
                setup: false,
                late_after_abort: 0,
                opaque123: 0,
                opaque167: 0,
            },
        );
        if create {
            self.host = Some(identity);
        }
        Ok(key)
    }
    pub fn deliverable(&self, key: Membership, round: Option<u64>) -> bool {
        self.current(key)
            && match round {
                None => self.phase == Phase::Idle,
                Some(expected) => {
                    matches!(&self.phase,Phase::Loading{round,..}|Phase::Armed{round} if *round==expected)
                }
            }
    }
    pub fn setup(&mut self, key: Membership) -> io::Result<()> {
        if !self.current(key) || self.phase != Phase::Idle {
            return Err(io::Error::other("stale or loading setup"));
        }
        let m = self.members.get_mut(&key.identity).expect("current member");
        if m.setup {
            return Err(io::Error::other("duplicate setup"));
        }
        m.setup = true;
        Ok(())
    }
    pub fn ready(&mut self, key: Membership) -> io::Result<()> {
        if !self.current(key) || self.phase != Phase::Idle || self.host == Some(key.identity) {
            return Err(io::Error::other("stale/loading/host Ready"));
        }
        let m = self.members.get_mut(&key.identity).expect("current member");
        if !m.setup || m.ready {
            return Err(io::Error::other("missing setup or duplicate Ready"));
        }
        m.ready = true;
        Ok(())
    }
    pub fn leave(&mut self, key: Membership) -> io::Result<Vec<Member>> {
        if !self.current(key) {
            return Err(io::Error::other("stale membership cleanup"));
        }
        let round = match self.phase {
            Phase::Loading { round, .. } | Phase::Armed { round } => Some(round),
            Phase::Idle => None,
        };
        if let Some(round) = round {
            self.aborted = Some((round, self.members.values().map(|m| m.key).collect()));
            for m in self.members.values_mut() {
                m.late_after_abort = 0;
            }
        }
        self.members.remove(&key.identity);
        if self.host == Some(key.identity) {
            self.host = self
                .members
                .values()
                .min_by_key(|m| m.slot)
                .map(|m| m.key.identity);
        }
        self.bombs.finish();
        if let Some(terrain) = &mut self.terrain {
            terrain.finish();
        }
        self.pickups.finish();
        self.phase = Phase::Idle;
        for m in self.members.values_mut() {
            m.ready = false;
        }
        Ok(self.members.values().cloned().collect())
    }
    pub fn begin(&mut self, key: Membership) -> io::Result<u64> {
        if !self.current(key)
            || self.host != Some(key.identity)
            || self.phase != Phase::Idle
            || self.members.len() != 2
            || !self
                .members
                .values()
                .all(|m| m.setup && (m.key.identity == key.identity || m.ready))
        {
            return Err(io::Error::other(
                "Start requires current host and two loaded-room ready members",
            ));
        }
        let round = self
            .round
            .checked_add(1)
            .ok_or_else(|| io::Error::other("round counter exhausted"))?;
        let roster = self.members.values().map(|m| m.key).collect();
        for member in self.members.values_mut() {
            member.opaque123 = 0;
            member.opaque167 = 0;
        }
        self.aborted = None;
        self.round = round;
        self.bombs.begin(round);
        if let Some(terrain) = &mut self.terrain {
            terrain.begin(round);
        }
        self.bomb_capacity = [1, 2];
        self.phase = Phase::Loading {
            round,
            roster,
            loaded: BTreeSet::new(),
        };
        Ok(round)
    }
    /// Opaque actual123/body10 while Armed: bounded reference no-response policy,
    /// not recovered message semantics. No queue, world, inventory or ACK effect.
    pub fn observe_opaque123(&mut self, key: Membership) -> io::Result<()> {
        if !self.current(key) || !matches!(self.phase, Phase::Armed { .. }) {
            return Err(io::Error::other("opaque123 current armed member required"));
        }
        let member = self.members.get_mut(&key.identity).expect("current member");
        if member.opaque123 >= 64 {
            return Err(io::Error::other("opaque123 observation budget"));
        }
        member.opaque123 += 1;
        Ok(())
    }
    /// Actual167/body32 observed afterbomb1, semanticsunknown. Same isolated
    /// reference no-response policy as123; its own budget cannot borrow123's.
    pub fn observe_opaque167(&mut self, key: Membership) -> io::Result<()> {
        if !self.current(key) || !matches!(self.phase, Phase::Armed { .. }) {
            return Err(io::Error::other("opaque167 current armed member required"));
        }
        let member = self.members.get_mut(&key.identity).expect("current member");
        if member.opaque167 >= 64 {
            return Err(io::Error::other("opaque167 observation budget"));
        }
        member.opaque167 += 1;
        Ok(())
    }
    pub fn observe_after_abort(&mut self, key: Membership) -> io::Result<()> {
        if !self.current(key)
            || self.phase != Phase::Idle
            || !self
                .aborted
                .as_ref()
                .is_some_and(|(_, roster)| roster.contains(&key))
        {
            return Err(io::Error::other("no captured aborted-round membership"));
        }
        let member = self
            .members
            .get_mut(&key.identity)
            .expect("current membership");
        if member.late_after_abort >= 64 {
            return Err(io::Error::other("late report after abort budget"));
        }
        member.late_after_abort += 1;
        Ok(())
    }
    pub fn ack(&mut self, key: Membership, round: u64) -> io::Result<bool> {
        if !self.current(key) {
            return Err(io::Error::other("stale loading membership"));
        }
        let Phase::Loading {
            round: current,
            roster,
            loaded,
        } = &mut self.phase
        else {
            return Err(io::Error::other("no active loading barrier"));
        };
        if *current != round || !roster.contains(&key) || loaded.contains(&key) {
            return Err(io::Error::other("wrong round/roster or duplicate ACK"));
        }
        loaded.insert(key);
        let complete = loaded.len() == roster.len();
        if complete {
            self.phase = Phase::Armed { round };
        }
        Ok(complete)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn prepared() -> (Room, Membership, Membership) {
        let mut r = Room::default();
        let a = r.enter(1, 11, true).unwrap();
        let b = r.enter(2, 22, false).unwrap();
        r.setup(a).unwrap();
        r.setup(b).unwrap();
        r.ready(b).unwrap();
        (r, a, b)
    }
    #[test]
    fn opaque123_has_current_armed_membership_and_finite_per_round_budget() {
        let (mut r, a, b) = prepared();
        assert!(r.observe_opaque123(a).is_err());
        let round = r.begin(a).unwrap();
        assert!(r.observe_opaque123(a).is_err());
        r.ack(a, round).unwrap();
        r.ack(b, round).unwrap();
        for _ in 0..64 {
            r.observe_opaque123(a).unwrap();
        }
        assert!(r.observe_opaque123(a).is_err());
        for _ in 0..64 {
            r.observe_opaque167(a).unwrap();
        }
        assert!(r.observe_opaque167(a).is_err());
        r.leave(a).unwrap();
        assert!(r.observe_opaque123(b).is_err());
        assert!(r.observe_opaque123(a).is_err());
        let a2 = r.enter(1, 12, false).unwrap();
        r.setup(a2).unwrap();
        r.ready(a2).unwrap();
        let next = r.begin(b).unwrap();
        r.ack(a2, next).unwrap();
        r.ack(b, next).unwrap();
        assert!(r.observe_opaque123(a).is_err());
        assert!(r.observe_opaque123(a2).is_ok());
        assert!(r.observe_opaque167(a).is_err());
        assert!(r.observe_opaque167(a2).is_ok());
        assert_eq!(r.phase, Phase::Armed { round: next });
    }
    #[test]
    fn host_leave_promotes_survivor_and_old_visit_cannot_clear_rejoin() {
        let (mut r, a, b) = prepared();
        let survivors = r.leave(a).unwrap();
        assert_eq!(survivors[0].key, b);
        assert_eq!(r.host, Some(2));
        let a2 = r.enter(1, 11, false).unwrap();
        assert_ne!(a, a2);
        assert!(r.leave(a).is_err());
        assert!(!r.deliverable(a, None));
        assert!(r.deliverable(a2, None));
        assert!(r.current(a2));
        assert!(r.ready(a).is_err());
        r.setup(a2).unwrap();
        r.ready(a2).unwrap();
        assert!(r.begin(a2).is_err());
        assert!(r.begin(b).is_ok());
    }
    #[test]
    fn missing_or_stale_ack_never_opens_barrier() {
        let (mut r, a, b) = prepared();
        let round = r.begin(a).unwrap();
        assert!(!r.ack(a, round).unwrap());
        assert!(matches!(r.phase, Phase::Loading { .. }));
        assert!(r.ack(a, round).is_err());
        r.leave(b).unwrap();
        assert!(r.ack(b, round).is_err());
        let b2 = r.enter(2, 23, false).unwrap();
        r.setup(b2).unwrap();
        r.ready(b2).unwrap();
        let next = r.begin(a).unwrap();
        assert!(next > round);
        assert!(r.ack(b2, round).is_err());
        assert!(!r.ack(a, next).unwrap());
        assert!(r.ack(b2, next).unwrap());
        assert_eq!(r.phase, Phase::Armed { round: next });
    }
    #[test]
    fn admission_and_not_ready_start_do_not_change_counters() {
        let mut r = Room::default();
        assert!(r.enter(2, 22, false).is_err());
        assert_eq!(r.visit, 0);
        let a = r.enter(1, 11, true).unwrap();
        assert!(r.enter(2, 11, false).is_err());
        assert_eq!(r.visit, 1);
        let b = r.enter(2, 22, false).unwrap();
        r.setup(a).unwrap();
        r.setup(b).unwrap();
        assert!(r.begin(a).is_err());
        assert_eq!(r.round, 0);
        r.ready(b).unwrap();
        assert!(r.begin(a).is_ok());
    }
}

#[cfg(test)]
mod abort_tests {
    use super::*;
    #[test]
    fn late_abort_observation_is_bounded_to_captured_survivor_and_never_opens_round() {
        let mut r = Room::default();
        let a = r.enter(1, 11, true).unwrap();
        let b = r.enter(2, 22, false).unwrap();
        r.setup(a).unwrap();
        r.setup(b).unwrap();
        r.ready(b).unwrap();
        let round = r.begin(a).unwrap();
        r.ack(a, round).unwrap();
        r.ack(b, round).unwrap();
        r.leave(a).unwrap();
        assert_eq!(r.phase, Phase::Idle);
        assert!(r.observe_after_abort(a).is_err());
        for _ in 0..64 {
            r.observe_after_abort(b).unwrap();
        }
        assert!(r.observe_after_abort(b).is_err());
        assert_eq!(r.phase, Phase::Idle);
        let a2 = r.enter(1, 12, false).unwrap();
        assert!(r.observe_after_abort(a2).is_err());
        r.setup(a2).unwrap();
        r.ready(a2).unwrap();
        let next = r.begin(b).unwrap();
        assert!(next > round);
        assert!(r.observe_after_abort(b).is_err());
    }
}

/// Existing fixed diagnostic12345+generation policy; not account authentication.
pub(crate) fn wire_key(round: u64) -> io::Result<u32> {
    12345u32
        .checked_add(u32::try_from(round).map_err(|_| io::Error::other("round key exhausted"))?)
        .ok_or_else(|| io::Error::other("round key exhausted"))
}
