//! Bounded two-player fixed-room model. Policy, not original KR authority.
//! Wire adapters reference CP028/034/035/229/230; no gameplay is implemented here.
//! Generation arguments fence internal work; raw585/99 lack a measured visit/round token.
//! Passing a current generation cannot prove an old wire ACK was rejected.
use std::{
    collections::{BTreeMap, BTreeSet},
    io,
};
/// CP213 fixed operator layouts; not a client team-selection protocol.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Deserialize)]
pub enum EightTeamLayout {
    #[default]
    #[serde(rename = "4v4")]
    Balanced,
    #[serde(rename = "1v7")]
    OneSeven,
}
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
    pub team: u8,
    pub ready: bool,
    pub setup: bool,
    late_after_abort: u16,
    opaque123: u16,
    opaque167: u16,
    opaque142: u16,
    pub(crate) life: crate::life::Player,
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
    Ending {
        round: u64,
        refreshed: BTreeSet<Membership>,
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
    capacity: usize,
    fixed_teams: Vec<u8>,
    sealed_abort: bool,
    pub(crate) bomb_capacity: Vec<usize>,
    pub(crate) terrain: Option<crate::terrain::Terrain>,
    pub(crate) last_blast: Option<std::time::Instant>,
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
            capacity: 2,
            fixed_teams: vec![0, 1],
            sealed_abort: false,
            bomb_capacity: vec![1, 2],
            terrain: None,
            last_blast: None,
        }
    }
}
impl Room {
    /// Explicit diagnostic profiles measured at CP235/200. Not general admission.
    pub(crate) fn with_capacity(capacity: usize) -> io::Result<Self> {
        if !matches!(capacity, 2 | 4 | 8) {
            return Err(io::Error::other("unsupported diagnostic room capacity"));
        }
        Ok(Self {
            capacity,
            fixed_teams: (0..capacity).map(|slot| (slot % 2) as u8).collect(),
            bomb_capacity: match capacity {
                8 => vec![1, 2, 1, 1, 2, 1, 1, 1], // CP209 actual baseline
                4 => vec![1, 2, 1, 1],
                _ => vec![1, 2],
            },
            ..Self::default()
        })
    }
    pub(crate) fn capacity(&self) -> usize {
        self.capacity
    }

    pub(crate) fn set_eight_layout(&mut self, layout: EightTeamLayout) -> io::Result<()> {
        if self.capacity != 8 || !self.members.is_empty() || self.phase != Phase::Idle {
            return Err(io::Error::other(
                "eight layout requires empty idle eight-room",
            ));
        }
        self.fixed_teams = match layout {
            EightTeamLayout::Balanced => (0..8).map(|slot| slot % 2).collect(),
            EightTeamLayout::OneSeven => vec![0, 1, 1, 1, 1, 1, 1, 1],
        };
        Ok(())
    }
    pub(crate) fn fixed_teams(&self) -> &[u8] {
        &self.fixed_teams
    }
    pub(crate) fn fixed_teams_match(&self) -> bool {
        self.members
            .values()
            .all(|m| self.fixed_teams.get(usize::from(m.slot)) == Some(&m.team))
    }
    pub(crate) fn balanced_teams(&self) -> bool {
        let zero = self.members.values().filter(|m| m.team == 0).count();
        let one = self.members.values().filter(|m| m.team == 1).count();
        zero > 0 && zero == one && zero + one == self.capacity
    }

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
        if self.sealed_abort
            || !(1..=self.capacity as u32).contains(&identity)
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
        } else if self.host.is_none() || self.members.len() >= self.capacity {
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
                team: self.fixed_teams[(identity - 1) as usize],
                ready: false,
                setup: false,
                late_after_abort: 0,
                opaque123: 0,
                opaque167: 0,
                opaque142: 0,
                life: crate::life::Player::default(),
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
            Phase::Loading { round, .. } | Phase::Armed { round } | Phase::Ending { round, .. } => {
                Some(round)
            }
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
        if self.sealed_abort
            || !self.current(key)
            || self.host != Some(key.identity)
            || self.phase != Phase::Idle
            || self.members.len() != self.capacity
            || !self
                .members
                .values()
                .all(|m| m.setup && (m.key.identity == key.identity || m.ready))
        {
            return Err(io::Error::other(
                "Start requires current host and all configured loaded-room ready members",
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
            member.opaque142 = 0;
            member.life.begin();
            member.late_after_abort = 0;
        }
        self.aborted = None;
        self.round = round;
        self.last_blast = None;
        self.bombs.begin(round);
        if let Some(terrain) = &mut self.terrain {
            terrain.begin(round);
        }
        self.bomb_capacity = match self.capacity {
            8 => vec![1, 2, 1, 1, 2, 1, 1, 1],
            4 => vec![1, 2, 1, 1],
            _ => vec![1, 2],
        };
        self.phase = Phase::Loading {
            round,
            roster,
            loaded: BTreeSet::new(),
        };
        Ok(round)
    }
    /// CP188/189 chosen forfeit policy: invalidate every pending job immediately,
    /// retain current survivor membership until delayed rank/return/cleanup.
    pub fn abort_play(&mut self, key: Membership) -> io::Result<u64> {
        if !self.current(key) {
            return Err(io::Error::other("stale abort membership"));
        }
        let Phase::Armed { round } = self.phase else {
            return Err(io::Error::other("forfeit requires active Battle"));
        };
        self.leave(key)?;
        if !self.members.is_empty() {
            self.phase = Phase::Ending {
                round,
                refreshed: BTreeSet::new(),
            };
        }
        Ok(round)
    }
    /// CP205 chosen unranked single-loss policy. No three-player restart/replacement.
    pub(crate) fn abort_four(&mut self, key: Membership) -> io::Result<u64> {
        if self.capacity != 4 || self.members.len() != 4 {
            return Err(io::Error::other(
                "unranked abort requires complete fixed-four room",
            ));
        }
        let round = self.abort_play(key)?;
        self.sealed_abort = true;
        Ok(round)
    }
    /// Eight current-member loss fences the room; no native seven-survivor
    /// return contract is claimed or cloned from the four-player abort path.
    pub(crate) fn fence_eight(&mut self, key: Membership) -> io::Result<()> {
        if self.capacity != 8 || !matches!(self.phase, Phase::Armed { .. } | Phase::Loading { .. })
        {
            return Err(io::Error::other(
                "eight fence requires current loading/Battle",
            ));
        }
        self.leave(key)?;
        self.sealed_abort = true;
        Ok(())
    }
    pub fn outcome_deliverable(&self, key: Membership, round: u64) -> bool {
        self.current(key)
            && matches!(self.phase, Phase::Ending { round: current, .. } if current == round)
    }
    pub fn end(&mut self, round: u64) -> io::Result<()> {
        if self.phase != (Phase::Armed { round }) {
            return Err(io::Error::other("outcome requires active round"));
        }
        self.phase = Phase::Ending {
            round,
            refreshed: BTreeSet::new(),
        };
        self.bombs.finish();
        self.pickups.finish();
        if let Some(terrain) = &mut self.terrain {
            terrain.finish();
        }
        for member in self.members.values_mut() {
            member.ready = false;
        }
        Ok(())
    }
    pub fn refreshed(&mut self, key: Membership, round: u64) -> io::Result<()> {
        if !self.current(key) {
            return Err(io::Error::other("stale outcome recipient"));
        }
        let Phase::Ending {
            round: current,
            refreshed,
        } = &mut self.phase
        else {
            return Err(io::Error::other("no pending outcome"));
        };
        if *current != round || !refreshed.insert(key) {
            return Err(io::Error::other("duplicate/stale outcome completion"));
        }
        if refreshed.len() == self.members.len() {
            self.aborted = Some((round, self.members.values().map(|m| m.key).collect()));
            self.phase = Phase::Idle;
        }
        Ok(())
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
    /// Four bomb-only known142: bounded observation, no reveal or effect.
    pub fn observe_opaque142(&mut self, key: Membership) -> io::Result<()> {
        if !self.current(key) || !matches!(self.phase, Phase::Armed { .. }) {
            return Err(io::Error::other("opaque142 current armed member required"));
        }
        let member = self.members.get_mut(&key.identity).expect("current member");
        if member.opaque142 >= 64 {
            return Err(io::Error::other("opaque142 observation budget"));
        }
        member.opaque142 += 1;
        Ok(())
    }
    /// Known167 sizes, semantics unknown; isolated no-response budget.
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
    pub fn observe_closed(&mut self, key: Membership) -> io::Result<()> {
        if !self.current(key) || !matches!(self.phase, Phase::Ending { .. }) {
            return Err(io::Error::other("not current closed member"));
        }
        let member = self.members.get_mut(&key.identity).expect("current member");
        if member.late_after_abort >= 64 {
            return Err(io::Error::other("closed report budget"));
        }
        member.late_after_abort += 1;
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
    fn four_members_require_all_ready_and_current_ack_and_keep_two_path_separate() {
        assert!(Room::with_capacity(3).is_err());
        assert_eq!(
            Room::with_capacity(8).unwrap().bomb_capacity,
            [1, 2, 1, 1, 2, 1, 1, 1]
        );
        let mut r = Room::with_capacity(4).unwrap();
        let a = r.enter(1, 11, true).unwrap();
        let b = r.enter(2, 22, false).unwrap();
        let c = r.enter(3, 33, false).unwrap();
        for key in [a, b, c] {
            r.setup(key).unwrap();
        }
        r.ready(b).unwrap();
        r.ready(c).unwrap();
        assert!(r.begin(a).is_err());
        let d = r.enter(4, 44, false).unwrap();
        r.setup(d).unwrap();
        assert!(r.begin(a).is_err());
        r.ready(d).unwrap();
        assert!(r.enter(5, 55, false).is_err());
        let round = r.begin(a).unwrap();
        assert_eq!(r.bomb_capacity, [1, 2, 1, 1]);
        assert_eq!(
            r.members
                .values()
                .map(|m| (m.slot, m.team))
                .collect::<Vec<_>>(),
            [(0, 0), (1, 1), (2, 0), (3, 1)]
        );
        for key in [a, b, c] {
            assert!(!r.ack(key, round).unwrap());
        }
        assert!(matches!(r.phase, Phase::Loading { .. }));
        assert!(r.ack(c, round).is_err());
        assert!(r.ack(d, round + 1).is_err());
        assert!(r.ack(d, round).unwrap());
        let survivors = r.leave(b).unwrap();
        assert_eq!(survivors.len(), 3);
        assert_eq!(r.phase, Phase::Idle);
        assert!(!r.deliverable(a, Some(round)));
        let replacement = r.enter(2, 66, false).unwrap();
        assert_ne!(replacement, b);
        assert!(!r.current(b));
        assert!(r.ack(b, round).is_err());
        let mut two = Room::default();
        assert!(two.enter(3, 33, true).is_err());
        assert_eq!(two.capacity(), 2);
        assert_eq!(two.bomb_capacity, [1, 2]);
    }
    #[test]
    fn active_abort_fences_jobs_and_keeps_survivor_until_forfeit_refresh() {
        let (mut r, a, b) = prepared();
        let round = r.begin(a).unwrap();
        r.ack(a, round).unwrap();
        r.ack(b, round).unwrap();
        assert_eq!(r.abort_play(a).unwrap(), round);
        assert_eq!(r.host, Some(2));
        assert!(!r.current(a));
        assert!(r.outcome_deliverable(b, round));
        assert!(!r.deliverable(b, Some(round)));
        assert!(r.abort_play(a).is_err());
        assert!(r.begin(b).is_err());
        r.refreshed(b, round).unwrap();
        assert_eq!(r.phase, Phase::Idle);
        assert!(!r.outcome_deliverable(b, round));
        let rejoin = r.enter(1, 33, false).unwrap();
        r.setup(rejoin).unwrap();
        r.ready(rejoin).unwrap();
        let next = r.begin(b).unwrap();
        assert_eq!(next, round + 1);
        assert!(r.refreshed(b, round).is_err());
    }
    #[test]
    fn outcome_barrier_blocks_start_and_old_completion_cannot_adopt_next_round() {
        let (mut r, a, b) = prepared();
        let round = r.begin(a).unwrap();
        r.ack(a, round).unwrap();
        r.ack(b, round).unwrap();
        r.end(round).unwrap();
        assert!(!r.deliverable(a, Some(round)));
        assert!(r.outcome_deliverable(a, round));
        assert!(r.begin(a).is_err());
        r.refreshed(a, round).unwrap();
        assert!(r.refreshed(a, round).is_err());
        assert!(matches!(r.phase, Phase::Ending { .. }));
        r.refreshed(b, round).unwrap();
        assert_eq!(r.phase, Phase::Idle);
        assert!(!r.outcome_deliverable(a, round));
        r.ready(b).unwrap();
        let next = r.begin(a).unwrap();
        assert_eq!(next, round + 1);
        assert!(r.refreshed(a, round).is_err());
        assert!(matches!(r.phase,Phase::Loading{round:current,..} if current==next));
        r.ack(a, next).unwrap();
        r.ack(b, next).unwrap();
        r.end(next).unwrap();
        r.leave(a).unwrap();
        assert!(!r.outcome_deliverable(b, next));
        assert!(r.refreshed(b, next).is_err());
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
        for _ in 0..64 {
            r.observe_opaque142(a).unwrap();
        }
        assert!(r.observe_opaque142(a).is_err());
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
        assert!(r.observe_opaque142(a).is_err());
        assert!(r.observe_opaque142(a2).is_ok());
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
