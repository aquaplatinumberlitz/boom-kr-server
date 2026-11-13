//! Bounded room model. Policy, not original KR authority.
//! Wire adapters reference CP028/034/035/229/230; no gameplay is implemented here.
//! Generation arguments fence internal work; raw585/99 lack a measured visit/round token.
//! Passing a current generation cannot prove an old wire ACK was rejected.
use crate::rules::SlotAllocation;
use std::{
    collections::{BTreeMap, BTreeSet},
    io,
};
/// Backend/session player identity. Slot indexes are allocated separately.
pub type PlayerId = u32;

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
    pub identity: PlayerId,
    pub connection: u64,
    pub visit: u64,
}
#[derive(Clone, Debug)]
pub struct Member {
    pub key: Membership,
    pub slot: u8,
    pub team: u8,
    pub character: Option<u16>,
    pub(crate) item_stats: Option<crate::characters::Stats>,
    pub ready: bool,
    pub setup: bool,
    late_after_abort: u16,
    opaque123: u16,
    opaque167: u16,
    opaque142: u16,
    loading79: bool,
    pub(crate) life: crate::life::Player,
}
/// Frozen admission/choice rows for one round, independent of live membership.
/// A departed transport never remains a command owner merely to retain a rank row.
#[derive(Clone, Debug)]
pub(crate) struct Participant {
    pub member: Member,
    pub departed: Option<std::time::Instant>,
    pending: BTreeSet<Membership>,
}

#[cfg(test)]
mod participant_departure_tests {
    use super::*;
    use std::time::{Duration, Instant};
    #[test]
    fn departures_keep_original_rank_rows_prune_targets_and_fence_next_round() {
        let mut room = Room::with_allocation(8, SlotAllocation::FirstVacant).unwrap();
        room.round_roster = crate::rules::RoundRoster::Players;
        let mut keys = Vec::new();
        for id in [70000, 42, 503, 88] {
            let key = room.enter(id, u64::from(id), keys.is_empty()).unwrap();
            room.setup(key).unwrap();
            room.choose(key, Some(0), None).unwrap();
            room.choose(key, None, Some(if keys.len() < 2 { 0 } else { 3 }))
                .unwrap();
            keys.push(key);
        }
        for key in &keys[1..] {
            room.ready(*key).unwrap();
        }
        let round = room.begin(keys[0]).unwrap();
        for key in &keys {
            room.ack(*key, round).unwrap();
        }
        let now = Instant::now();
        room.depart_player(keys[3], now).unwrap();
        assert!(!room.current(keys[3]));
        assert_eq!(room.phase, Phase::Armed { round });
        assert_eq!(room.participants.len(), 4);
        assert!(room.valid_round_roster());
        assert!(room.departure_deliverable(keys[3], keys[2], round));
        room.depart_player(keys[2], now).unwrap();
        assert!(!room.departure_deliverable(keys[3], keys[2], round));
        assert!(room.depart_player(keys[2], now).is_err());
        let report = crate::life::Report {
            subtype: 21,
            slot: 0,
            tick: 100,
            digest: "survivor-current".into(),
            forward: None,
        };
        room.members.get_mut(&keys[0].identity).unwrap().life.apply(
            0,
            &report,
            now + Duration::from_secs(1),
            None,
        );
        assert_eq!(
            crate::four_results::winner(&room, now + Duration::from_secs(8)),
            None
        );
        for source in &keys[2..] {
            for recipient in &keys[..2] {
                room.departure_delivered(*source, *recipient, round)
                    .unwrap();
            }
        }
        assert!(!room.departure_pending());
        assert!(room.departure_delivered(keys[3], keys[0], round).is_err());
        assert_eq!(
            crate::four_results::winner(&room, now + Duration::from_secs(8)),
            Some(0)
        );
        room.end(round).unwrap();
        for key in &keys[..2] {
            room.refreshed(*key, round).unwrap();
        }
        assert_eq!(room.phase, Phase::Idle);
        assert!(room.begin(keys[0]).is_err()); // One team cannot start a new round.
        room.choose(keys[1], None, Some(3)).unwrap();
        room.ready(keys[1]).unwrap();
        let next = room.begin(keys[0]).unwrap();
        assert!(next > round);
        assert_eq!(room.participants.len(), 2);
        assert!(!room.departure_deliverable(keys[3], keys[0], next));
        assert!(room.departure_delivered(keys[3], keys[0], round).is_err());
        assert!(!room.deliverable(keys[0], Some(round)));
    }
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
    pub members: BTreeMap<PlayerId, Member>,
    pub host: Option<PlayerId>,
    pub phase: Phase,
    visit: u64,
    round: u64,
    epochs: Option<std::sync::Arc<crate::round_epochs::RoundEpochs>>,
    aborted: Option<(u64, Vec<Membership>)>,
    pub(crate) pickups: crate::pickups::Round,
    pub(crate) bombs: crate::bombs::Ledger,
    capacity: usize,
    pub(crate) round_roster: crate::rules::RoundRoster,
    allocation: SlotAllocation,
    fixed_teams: Vec<u8>,
    sealed_abort: bool,
    pub(crate) bomb_capacity: Vec<usize>,
    pub(crate) terrain: Option<crate::terrain::Terrain>,
    pub(crate) last_blast: Option<std::time::Instant>,
    pub(crate) participants: BTreeMap<Membership, Participant>,
}
impl Default for Room {
    fn default() -> Self {
        Self {
            members: BTreeMap::new(),
            host: None,
            phase: Phase::Idle,
            visit: 0,
            round: 0,
            epochs: None,
            aborted: None,
            pickups: crate::pickups::Round::default(),
            bombs: crate::bombs::Ledger::default(),
            capacity: 2,
            round_roster: crate::rules::RoundRoster::Full,
            allocation: SlotAllocation::LegacyIdentity,
            fixed_teams: vec![0, 1],
            sealed_abort: false,
            bomb_capacity: crate::rules::default_bomb_capacity(2),
            terrain: None,
            last_blast: None,
            participants: BTreeMap::new(),
        }
    }
}
impl Room {
    /// Explicit diagnostic profiles measured at CP235/200. Not general admission.
    #[cfg(test)]
    pub(crate) fn with_capacity(capacity: usize) -> io::Result<Self> {
        Self::with_allocation(capacity, SlotAllocation::LegacyIdentity)
    }
    pub(crate) fn with_allocation(capacity: usize, allocation: SlotAllocation) -> io::Result<Self> {
        if !matches!(capacity, 2 | 4 | 8) {
            return Err(io::Error::other("unsupported diagnostic room capacity"));
        }
        Ok(Self {
            capacity,
            allocation,
            fixed_teams: (0..capacity).map(|slot| (slot % 2) as u8).collect(),
            bomb_capacity: crate::rules::default_bomb_capacity(capacity),
            ..Self::default()
        })
    }
    pub(crate) fn set_allocation(&mut self, allocation: SlotAllocation) -> io::Result<()> {
        if !self.members.is_empty() || self.phase != Phase::Idle {
            return Err(io::Error::other("slot allocation requires pristine room"));
        }
        self.allocation = allocation;
        Ok(())
    }
    pub(crate) fn capacity(&self) -> usize {
        self.capacity
    }
    pub(crate) fn set_admission_capacity(&mut self, capacity: usize) -> io::Result<()> {
        if !(2..=self.capacity).contains(&capacity)
            || !self.members.is_empty()
            || self.phase != Phase::Idle
        {
            return Err(io::Error::other(
                "capacity requires pristine room within prototype bound",
            ));
        }
        self.capacity = capacity;
        self.fixed_teams.truncate(capacity);
        self.bomb_capacity.truncate(capacity);
        Ok(())
    }

    pub(crate) fn round_size(&self) -> usize {
        if self.round_roster == crate::rules::RoundRoster::Players {
            self.members.len()
        } else {
            self.round_roster.size(self.capacity)
        }
    }
    pub(crate) fn valid_round_roster(&self) -> bool {
        self.members.len() == self.round_size()
            && match self.round_roster {
                crate::rules::RoundRoster::Full => true,
                crate::rules::RoundRoster::Players => {
                    if matches!(self.phase, Phase::Armed { .. } | Phase::Ending { .. }) {
                        !self.members.is_empty()
                            && !self.participants.is_empty()
                            && self.members.values().all(|member| {
                                self.participants.get(&member.key).is_some_and(|p| {
                                    p.departed.is_none()
                                        && p.member.slot == member.slot
                                        && p.member.team == member.team
                                        && p.member.character == member.character
                                })
                            })
                    } else {
                        (2..=self.capacity).contains(&self.members.len())
                            && self
                                .members
                                .values()
                                .map(|member| member.team)
                                .collect::<BTreeSet<_>>()
                                .len()
                                >= 2
                    }
                }
                crate::rules::RoundRoster::Contiguous(count) => {
                    matches!(count, 2 | 4 | 8)
                        && count <= self.capacity
                        && (0..count).all(|slot| {
                            self.members.values().any(|member| {
                                usize::from(member.slot) == slot
                                    && self.fixed_teams.get(slot) == Some(&member.team)
                            })
                        })
                }
            }
    }
    pub(crate) fn player_round(&self) -> bool {
        self.round_roster == crate::rules::RoundRoster::Players
    }
    pub(crate) fn round_teams(&self) -> &[u8] {
        &self.fixed_teams[..self.round_size()]
    }

    pub(crate) fn set_round_epochs(
        &mut self,
        epochs: std::sync::Arc<crate::round_epochs::RoundEpochs>,
    ) -> io::Result<()> {
        if !self.members.is_empty() || self.phase != Phase::Idle || self.round != 0 {
            return Err(io::Error::other("round allocator requires pristine room"));
        }
        self.epochs = Some(epochs);
        Ok(())
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
        let one = self
            .members
            .values()
            .filter(|m| m.team == if self.player_round() { 3 } else { 1 })
            .count();
        zero > 0 && zero == one && zero + one == self.round_size()
    }

    pub fn current(&self, key: Membership) -> bool {
        self.members
            .get(&key.identity)
            .is_some_and(|m| m.key == key)
    }
    pub fn enter(
        &mut self,
        identity: PlayerId,
        connection: u64,
        create: bool,
    ) -> io::Result<Membership> {
        if self.sealed_abort
            || identity == 0
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
            if (self.allocation == SlotAllocation::LegacyIdentity && identity != 1)
                || self.host.is_some()
                || !self.members.is_empty()
            {
                return Err(io::Error::other("fixed room requires first host create"));
            }
        } else if self.host.is_none() || self.members.len() >= self.capacity {
            return Err(io::Error::other("no fixed room or full"));
        }
        let slot = match self.allocation {
            SlotAllocation::LegacyIdentity => identity
                .checked_sub(1)
                .filter(|slot| (*slot as usize) < self.capacity)
                .map(|slot| slot as u8),
            SlotAllocation::FirstVacant => (0..self.capacity)
                .find(|slot| {
                    !self
                        .members
                        .values()
                        .any(|member| usize::from(member.slot) == *slot)
                })
                .map(|slot| slot as u8),
        }
        .ok_or_else(|| io::Error::other("no eligible room slot"))?;
        if self.members.values().any(|member| member.slot == slot) {
            return Err(io::Error::other("room slot already occupied"));
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
                slot,
                team: if self.player_round() {
                    0
                } else {
                    self.fixed_teams[usize::from(slot)]
                },
                character: self.player_round().then_some(0),
                item_stats: None,
                ready: false,
                setup: false,
                late_after_abort: 0,
                opaque123: 0,
                opaque167: 0,
                opaque142: 0,
                loading79: false,
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
    /// Design: any choice invalidates readiness of all guests. Never let a
    /// stale/loading/non-setup membership mutate another player's selection.
    pub(crate) fn restore_character(&mut self, key: Membership, character: u16) -> io::Result<()> {
        if !self.current(key)
            || self.phase != Phase::Idle
            || self.members[&key.identity].setup
            || self.sealed_abort
            || !matches!(character, 0 | 1)
        {
            return Err(io::Error::other(
                "unsupported initial character restoration",
            ));
        }
        self.members
            .get_mut(&key.identity)
            .expect("current owner")
            .character = Some(character);
        Ok(())
    }
    pub(crate) fn choose_map(&mut self, key: Membership) -> io::Result<()> {
        if !self.current(key)
            || self.host != Some(key.identity)
            || self.phase != Phase::Idle
            || self.sealed_abort
        {
            return Err(io::Error::other("map choice requires current idle host"));
        }
        for member in self.members.values_mut() {
            if Some(member.key.identity) != self.host {
                member.ready = false;
            }
        }
        Ok(())
    }
    pub(crate) fn choose(
        &mut self,
        key: Membership,
        character: Option<u16>,
        team: Option<u8>,
    ) -> io::Result<()> {
        if !self.current(key)
            || self.phase != Phase::Idle
            || !self.members[&key.identity].setup
            || self.sealed_abort
            || character.is_some_and(|value| !matches!(value, 0 | 1))
            || team.is_some_and(|value| !matches!(value, 0 | 3))
            || character.is_some() == team.is_some()
        {
            return Err(io::Error::other("unsupported or stale room choice"));
        }
        let member = self.members.get_mut(&key.identity).expect("current member");
        if let Some(value) = character {
            member.character = Some(value);
        }
        if let Some(value) = team {
            member.team = value;
        }
        for member in self.members.values_mut() {
            member.ready = false;
        }
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
        self.participants.clear();
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
            || !self.valid_round_roster()
            || !self
                .members
                .values()
                .all(|m| m.setup && (m.key.identity == key.identity || m.ready))
        {
            return Err(io::Error::other(
                "Start requires current host and all configured loaded-room ready members",
            ));
        }
        let round = match &self.epochs {
            Some(epochs) => epochs.next()?,
            None => self
                .round
                .checked_add(1)
                .ok_or_else(|| io::Error::other("round counter exhausted"))?,
        };
        let roster = self.members.values().map(|m| m.key).collect();
        for member in self.members.values_mut() {
            member.opaque123 = 0;
            member.opaque167 = 0;
            member.opaque142 = 0;
            member.loading79 = false;
            member.life.begin();
            member.item_stats = None;
            member.late_after_abort = 0;
        }
        self.aborted = None;
        self.round = round;
        self.last_blast = None;
        self.participants = if self.player_round() {
            self.members
                .values()
                .map(|member| {
                    (
                        member.key,
                        Participant {
                            member: member.clone(),
                            departed: None,
                            pending: BTreeSet::new(),
                        },
                    )
                })
                .collect()
        } else {
            BTreeMap::new()
        };
        self.bombs.begin(round);
        if let Some(terrain) = &mut self.terrain {
            terrain.begin(round);
        }
        self.bomb_capacity = crate::rules::default_bomb_capacity(self.capacity);
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
    /// Experimental player-round departure. Removal fences commands immediately;
    /// already accepted bomb work belongs to this round and its remaining targets.
    pub(crate) fn depart_player(
        &mut self,
        key: Membership,
        now: std::time::Instant,
    ) -> io::Result<u64> {
        if !self.player_round() || !self.current(key) {
            return Err(io::Error::other("departure requires current player round"));
        }
        let round = match self.phase {
            Phase::Armed { round } | Phase::Ending { round, .. } => round,
            _ => {
                return Err(io::Error::other(
                    "departure requires playing or ending round",
                ));
            }
        };
        let armed = matches!(self.phase, Phase::Armed { .. });
        let member = self.members.remove(&key.identity).expect("current member");
        if self.host == Some(key.identity) {
            self.host = self
                .members
                .values()
                .min_by_key(|m| m.slot)
                .map(|m| m.key.identity);
        }
        let completed_blasts = self.bombs.forget_recipient(key);
        // Remaining recipients already completed their writes. A departed last
        // recipient cannot strand a terrain bomb or block later occupancy.
        if armed && self.terrain.is_some() {
            for ticket in completed_blasts {
                self.commit_blast(ticket.round, ticket.id, now);
            }
        }
        for member in self.members.values_mut() {
            member.life.forget_recipient(key);
        }
        for participant in self.participants.values_mut() {
            participant.pending.remove(&key);
        }
        let participant = self
            .participants
            .get_mut(&member.key)
            .ok_or_else(|| io::Error::other("missing round participant"))?;
        // Retain life evidence while preserving immutable Start identity/choices.
        participant.member.life = member.life;
        participant.departed = Some(now);
        if armed {
            participant.pending = self.members.values().map(|m| m.key).collect();
        }
        if self.members.is_empty() {
            self.bombs.finish();
            self.pickups.finish();
            if let Some(terrain) = &mut self.terrain {
                terrain.finish();
            }
            self.phase = Phase::Idle;
        } else if let Phase::Ending { refreshed, .. } = &mut self.phase {
            refreshed.remove(&key);
            if refreshed.len() == self.members.len() {
                self.phase = Phase::Idle;
            }
        }
        Ok(round)
    }
    pub(crate) fn departure_pending(&self) -> bool {
        self.participants.values().any(|p| !p.pending.is_empty())
    }
    pub(crate) fn departure_deliverable(
        &self,
        source: Membership,
        recipient: Membership,
        round: u64,
    ) -> bool {
        self.deliverable(recipient, Some(round))
            && self.phase == (Phase::Armed { round })
            && self
                .participants
                .get(&source)
                .is_some_and(|p| p.departed.is_some() && p.pending.contains(&recipient))
    }
    pub(crate) fn departure_delivered(
        &mut self,
        source: Membership,
        recipient: Membership,
        round: u64,
    ) -> io::Result<()> {
        if !self.departure_deliverable(source, recipient, round) {
            return Err(io::Error::other("stale departure completion"));
        }
        self.participants
            .get_mut(&source)
            .expect("validated participant")
            .pending
            .remove(&recipient);
        Ok(())
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
        self.fence_active(key)
    }
    /// Fail closed after loss in a measured team round. No survivor restart,
    /// result or recovery packet is invented for an unsupported roster.
    pub(crate) fn fence_active(&mut self, key: Membership) -> io::Result<()> {
        if !matches!(self.phase, Phase::Armed { .. } | Phase::Loading { .. }) {
            return Err(io::Error::other("fence requires current loading/Battle"));
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
    pub(crate) fn commit_blast(&mut self, round: u64, id: u16, now: std::time::Instant) {
        if self.phase != (Phase::Armed { round }) {
            return;
        }
        self.last_blast = Some(now);
        for member in self.members.values_mut() {
            member.life.blast_committed();
        }
        if let Some(terrain) = &mut self.terrain {
            terrain.blast_committed(round, id);
        }
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
    /// Exact observed packet79 is budget-only. Its semantics remain unknown;
    /// it cannot authorize activity, mutate gameplay, or complete loading.
    pub(crate) fn observe_loading79(&mut self, key: Membership) -> io::Result<()> {
        if !self.current(key) || !matches!(self.phase, Phase::Loading { .. }) {
            return Err(io::Error::other(
                "opaque79 needs current loading membership",
            ));
        }
        let member = self.members.get_mut(&key.identity).expect("current member");
        if !matches!(member.character, Some(0 | 1)) || member.loading79 {
            return Err(io::Error::other(
                "opaque79 outside measured choice or budget",
            ));
        }
        member.loading79 = true;
        Ok(())
    }
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
    #[test]
    fn duel_roster_admits_nonfull_room_but_rejects_sparse_extra_and_wrong_team() {
        let mut room = Room::with_allocation(4, SlotAllocation::FirstVacant).unwrap();
        room.round_roster = crate::rules::RoundRoster::Contiguous(2);
        let host = room.enter(90, 11, true).unwrap();
        room.setup(host).unwrap();
        assert!(room.begin(host).is_err());
        let guest = room.enter(70000, 22, false).unwrap();
        room.setup(guest).unwrap();
        room.ready(guest).unwrap();
        let mut wrong_team = room.clone();
        wrong_team.members.get_mut(&70000).unwrap().team = 0;
        assert!(wrong_team.begin(host).is_err());
        let mut sparse = room.clone();
        sparse.members.get_mut(&70000).unwrap().slot = 3;
        assert!(sparse.begin(host).is_err());
        let mut extra = room.clone();
        let third = extra.enter(8, 33, false).unwrap();
        extra.setup(third).unwrap();
        extra.ready(third).unwrap();
        assert!(extra.begin(host).is_err());
        let round = room.begin(host).unwrap();
        assert_eq!(room.members.len(), 2);
        assert_eq!(room.capacity(), 4);
        assert!(!room.ack(host, round).unwrap());
        assert!(room.ack(guest, round).unwrap());
    }

    #[test]
    fn shared_epochs_keep_two_room_barriers_results_and_abandoned_candidates_apart() {
        fn pair(
            ids: [u32; 2],
            epochs: std::sync::Arc<crate::round_epochs::RoundEpochs>,
        ) -> (Room, [Membership; 2]) {
            let mut room = Room::with_allocation(2, SlotAllocation::FirstVacant).unwrap();
            room.set_round_epochs(epochs).unwrap();
            let host = room.enter(ids[0], u64::from(ids[0]), true).unwrap();
            let guest = room.enter(ids[1], u64::from(ids[1]), false).unwrap();
            room.setup(host).unwrap();
            room.setup(guest).unwrap();
            (room, [host, guest])
        }
        let epochs = std::sync::Arc::new(crate::round_epochs::RoundEpochs::default());
        let (mut a, am) = pair([70000, 42], epochs.clone());
        let (mut b, bm) = pair([3, 4], epochs);
        assert!(a.begin(am[0]).is_err()); // Not ready: no lease consumed.
        a.ready(am[1]).unwrap();
        b.ready(bm[1]).unwrap();
        let ar = a.begin(am[0]).unwrap();
        let burned = b.clone().begin(bm[0]).unwrap(); // Failed publish cannot reuse a key.
        let br = b.begin(bm[0]).unwrap();
        assert_eq!((ar, burned, br), (1, 2, 3));
        assert_ne!(wire_key(ar).unwrap() as u8, wire_key(br).unwrap() as u8);
        assert!(a.ack(am[0], br).is_err());
        assert!(b.ack(bm[0], ar).is_err());
        assert!(!a.ack(am[0], ar).unwrap());
        assert!(a.ack(am[1], ar).unwrap());
        assert!(!b.ack(bm[0], br).unwrap());
        assert!(matches!(b.phase, Phase::Loading { round, .. } if round == br));
        a.end(ar).unwrap();
        a.refreshed(am[0], ar).unwrap();
        a.refreshed(am[1], ar).unwrap();
        a.ready(am[1]).unwrap();
        let next = a.begin(am[0]).unwrap();
        assert_eq!(next, 4);
        assert!(!a.deliverable(am[0], Some(ar)));
        assert!(a.refreshed(am[0], ar).is_err());
        assert!(b.ack(bm[1], br).unwrap());
        assert_eq!(b.phase, Phase::Armed { round: br });
        assert!(a.ack(am[0], br).is_err());
    }

    #[test]
    fn unrelated_player_ids_allocate_slots_and_fence_reused_memberships() {
        let mut room = Room::with_allocation(2, SlotAllocation::FirstVacant).unwrap();
        assert!(room.enter(0, 1, true).is_err());
        let creator = room.enter(70000, 11, true).unwrap();
        let guest = room.enter(42, 22, false).unwrap();
        assert_eq!(room.host, Some(70000));
        assert_eq!((room.members[&70000].slot, room.members[&42].slot), (0, 1));
        assert!(room.enter(90000, 33, false).is_err());
        assert!(room.enter(70000, 44, false).is_err());
        let visits = room.visit;
        room.leave(creator).unwrap();
        assert_eq!(room.host, Some(42));
        assert!(room.enter(90000, 22, false).is_err());
        assert_eq!(room.visit, visits);
        let replacement = room.enter(70000, 44, false).unwrap();
        assert_eq!(room.members[&70000].slot, 0);
        assert_eq!(room.host, Some(42));
        assert!(replacement.visit > creator.visit);
        assert!(!room.current(creator));
        assert!(room.leave(creator).is_err());
        assert!(room.current(replacement) && room.current(guest));
        room.setup(replacement).unwrap();
        room.setup(guest).unwrap();
        room.ready(replacement).unwrap();
        let round = room.begin(guest).unwrap();
        assert!(!room.deliverable(creator, Some(round)));
        assert!(room.ack(creator, round).is_err());
        assert!(!room.ack(replacement, round).unwrap());
        assert!(room.ack(guest, round).unwrap());
    }
    #[test]
    fn legacy_allocation_keeps_identity_slots_and_rejects_out_of_range_atomically() {
        let mut room = Room::default();
        assert!(room.enter(42, 11, true).is_err());
        assert!(room.enter(2, 22, true).is_err());
        let host = room.enter(1, 11, true).unwrap();
        let visit = room.visit;
        assert!(room.enter(42, 22, false).is_err());
        assert_eq!(room.visit, visit);
        assert!(room.current(host));
        let guest = room.enter(2, 22, false).unwrap();
        assert_eq!(room.members[&guest.identity].slot, 1);
    }
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
    fn choices_follow_current_membership_clear_ready_and_reject_unknown_or_active_input() {
        let (mut room, host, guest) = prepared();
        assert!(room.members[&guest.identity].ready);
        room.choose(host, Some(1), None).unwrap();
        assert_eq!(room.members[&host.identity].character, Some(1));
        assert!(!room.members[&guest.identity].ready);
        room.choose(guest, None, Some(0)).unwrap();
        assert_eq!(room.members[&guest.identity].team, 0);
        let before = room.members[&guest.identity].clone();
        for (character, team) in [
            (Some(4), None),
            (None, Some(2)),
            (None, None),
            (Some(0), Some(0)),
        ] {
            assert!(room.choose(guest, character, team).is_err());
            assert_eq!(room.members[&guest.identity].character, before.character);
            assert_eq!(room.members[&guest.identity].team, before.team);
        }
        room.leave(guest).unwrap();
        let fresh = room
            .enter(guest.identity, guest.connection + 1, false)
            .unwrap();
        assert!(room.choose(guest, Some(0), None).is_err());
        assert!(room.choose(fresh, Some(0), None).is_err());
        room.setup(fresh).unwrap();
        room.choose(fresh, Some(0), None).unwrap();
        room.ready(fresh).unwrap();
        let round = room.begin(host).unwrap();
        assert!(room.choose(fresh, None, Some(1)).is_err());
        room.ack(host, round).unwrap();
        room.ack(fresh, round).unwrap();
        assert!(room.choose(host, Some(0), None).is_err());
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

#[cfg(test)]
mod player_round_tests {
    use super::*;

    #[test]
    fn current_roster_allows_three_then_sparse_two_without_reusing_old_membership_or_round() {
        let mut room = Room::with_allocation(8, SlotAllocation::FirstVacant).unwrap();
        room.round_roster = crate::rules::RoundRoster::Players;
        let mut keys = Vec::new();
        for (id, conn) in [(70000, 11), (42, 22), (900, 33)] {
            let key = room.enter(id, conn, keys.is_empty()).unwrap();
            room.setup(key).unwrap();
            keys.push(key);
        }
        assert!(!room.valid_round_roster()); // One team cannot start a match.
        room.choose(keys[2], None, Some(3)).unwrap();
        room.choose(keys[0], Some(1), None).unwrap();
        assert!(room.valid_round_roster());
        room.ready(keys[1]).unwrap();
        room.ready(keys[2]).unwrap();
        let first = room.begin(keys[0]).unwrap();
        for key in &keys[..2] {
            assert!(!room.ack(*key, first).unwrap());
        }
        assert!(room.ack(keys[2], first).unwrap());
        room.end(first).unwrap();
        for key in &keys {
            room.refreshed(*key, first).unwrap();
        }
        room.leave(keys[1]).unwrap();
        assert!(room.ready(keys[1]).is_err());
        assert_eq!(room.members[&900].slot, 2);
        assert_eq!(room.members[&70000].character, Some(1));
        room.ready(keys[2]).unwrap();
        let second = room.begin(keys[0]).unwrap();
        assert_ne!(first, second);
        assert!(room.ack(keys[0], first).is_err());
        assert!(!room.ack(keys[0], second).unwrap());
        assert!(room.ack(keys[2], second).unwrap());
    }
}

#[cfg(test)]
mod loading_observation_tests {
    use super::*;
    #[test]
    fn opaque79_never_completes_barrier_and_rejects_wrong_choice_phase_or_duplicate() {
        let mut room = Room::with_allocation(4, SlotAllocation::FirstVacant).unwrap();
        room.round_roster = crate::rules::RoundRoster::Players;
        let a = room.enter(70000, 11, true).unwrap();
        let b = room.enter(42, 22, false).unwrap();
        room.setup(a).unwrap();
        room.setup(b).unwrap();
        room.choose(a, Some(1), None).unwrap();
        room.choose(b, None, Some(3)).unwrap();
        assert!(room.observe_loading79(a).is_err());
        room.ready(b).unwrap();
        let round = room.begin(a).unwrap();
        room.members.get_mut(&b.identity).unwrap().character = Some(4);
        assert!(room.observe_loading79(b).is_err());
        room.members.get_mut(&b.identity).unwrap().character = Some(0);
        room.observe_loading79(b).unwrap();
        room.observe_loading79(a).unwrap();
        assert!(room.observe_loading79(a).is_err());
        assert!(!room.ack(a, round).unwrap());
        assert!(room.ack(b, round).unwrap());
        assert!(room.observe_loading79(a).is_err());
    }
}

#[cfg(test)]
mod restored_character_tests {
    use super::*;
    #[test]
    fn account_character_restores_before_setup_and_old_visit_cannot_change_new_membership() {
        let mut room = Room::with_allocation(8, SlotAllocation::FirstVacant).unwrap();
        let old = room.enter(70000, 10, true).unwrap();
        room.restore_character(old, 1).unwrap();
        assert_eq!(room.members[&70000].character, Some(1));
        room.setup(old).unwrap();
        assert!(room.restore_character(old, 0).is_err());
        room.leave(old).unwrap();
        let new = room.enter(70000, 11, true).unwrap();
        assert_ne!(new, old);
        assert!(room.restore_character(old, 1).is_err());
        assert!(room.restore_character(new, 4).is_err());
        room.restore_character(new, 0).unwrap();
        assert_eq!(room.members[&70000].character, Some(0));
        assert_eq!(room.members[&70000].slot, 0);
    }
}

#[cfg(test)]
mod map_choice_policy_tests {
    use super::*;
    #[test]
    fn initial_host_map_sync_and_ready_clear_never_authorize_guest_stale_or_active_choice() {
        let mut room = Room::with_allocation(8, SlotAllocation::FirstVacant).unwrap();
        room.round_roster = crate::rules::RoundRoster::Contiguous(2);
        let host = room.enter(70000, 10, true).unwrap();
        room.choose_map(host).unwrap(); // Native96 is sent before final setup117.
        room.setup(host).unwrap();
        let guest = room.enter(42, 11, false).unwrap();
        room.setup(guest).unwrap();
        room.ready(guest).unwrap();
        assert!(room.choose_map(guest).is_err());
        assert!(room.members[&42].ready);
        room.choose_map(host).unwrap();
        assert!(!room.members[&42].ready);
        room.ready(guest).unwrap();
        let round = room.begin(host).unwrap();
        assert!(room.choose_map(host).is_err());
        room.ack(host, round).unwrap();
        room.ack(guest, round).unwrap();
        assert!(room.choose_map(host).is_err());
        room.leave(host).unwrap();
        assert!(room.choose_map(host).is_err());
    }
}
