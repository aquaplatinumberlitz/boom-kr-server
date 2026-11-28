//! Design rules for an opt-in emulator, with no implicit defaults or packet authority.
//! The caller supplies an authoritative order and monotonic time in milliseconds.
use super::{
    ModeKind, Policy,
    rules::{Wave, WinCondition},
};
use kr_wire::kr00967::RoomTeamSelector;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) type Slot = u8;
pub(crate) type Team = u8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Player {
    pub slot: Slot,
    pub team: Team,
}

impl ModeKind {
    fn evaluates_unknown_outcome(self) -> bool {
        // Explicit evaluator allowlist. An export or an unknown rule
        // descriptor alone does not enable a runtime.
        matches!(
            self,
            Self::Capong
                | Self::Coloring
                | Self::Gumiho
                | Self::Hamster
                | Self::Hedgehog
                | Self::PopcadeA
                | Self::PopcadeB
                | Self::PopcadeC
                | Self::SuperFighter
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Life {
    Alive,
    Ghost,
    Eliminated,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Outcome {
    Playing,
    Winner(Team),
    /// The round ended, but ModeRules does not establish how to project a draw
    /// or an unknown result. The caller must not award either side a win.
    Unresolved,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
pub enum Setup {
    Revive {
        team_budget: Option<u16>,
        player_budget: Option<u16>,
        duration_ms: u64,
        score_limit: u32,
    },
    Timebomb {
        initial_carrier: Slot,
        fuse_ms: u64,
    },
    LeaderCapture {
        leaders: Vec<Slot>,
    },
    Evaluate,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Action<'a> {
    AdvanceTo(u64),
    /// A single authoritative batch of simultaneous defeats.
    Defeat(&'a [Slot]),
    Respawn(Slot),
    PassBomb {
        from: Slot,
        to: Slot,
    },
    /// A result from the CoOp objective authority; None means no unique winner.
    /// Evaluation of an unknown win condition only accepts None.
    Objective(Option<Team>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Error {
    UnsupportedMode,
    InvalidSetup,
    InvalidRoster,
    WrongRound,
    SequenceGap,
    Finished,
    ClockReversed,
    UnsupportedAction,
    InvalidPlayer,
    InvalidTransition,
    NoRevives,
    UnknownTeam,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Applied {
    Applied,
    Replay,
}

#[derive(Clone)]
struct Roster {
    players: BTreeMap<Slot, (Team, Life)>,
    teams: BTreeSet<Team>,
}

impl Roster {
    fn new(players: &[Player]) -> Result<Self, Error> {
        let mut roster = Self {
            players: BTreeMap::new(),
            teams: BTreeSet::new(),
        };
        for &Player { slot, team } in players {
            if roster.players.insert(slot, (team, Life::Alive)).is_some() {
                return Err(Error::InvalidRoster);
            }
            roster.teams.insert(team);
        }
        if roster.teams.len() < 2 {
            return Err(Error::InvalidRoster);
        }
        Ok(roster)
    }

    fn player(&self, slot: Slot) -> Result<(Team, Life), Error> {
        self.players.get(&slot).copied().ok_or(Error::InvalidPlayer)
    }

    fn set_life(&mut self, slot: Slot, life: Life) {
        self.players.get_mut(&slot).expect("validated slot").1 = life;
    }

    fn live_team_outcome(&self) -> Outcome {
        let mut alive = self
            .players
            .values()
            .filter_map(|&(team, life)| (life == Life::Alive).then_some(team));
        match alive.next() {
            None => Outcome::Unresolved,
            Some(team) if alive.all(|other| other == team) => Outcome::Winner(team),
            Some(_) => Outcome::Playing,
        }
    }

    fn validate_defeat(&self, slots: &[Slot]) -> Result<(), Error> {
        let mut seen = BTreeSet::new();
        if slots.is_empty() {
            return Err(Error::InvalidTransition);
        }
        for &slot in slots {
            if self.player(slot)?.1 != Life::Alive || !seen.insert(slot) {
                return Err(Error::InvalidTransition);
            }
        }
        Ok(())
    }

    fn opponent(&self, team: Team) -> Team {
        *self
            .teams
            .iter()
            .find(|&&other| other != team)
            .expect("two teams")
    }
}

#[derive(Clone)]
struct Revive {
    team_budget: Option<BTreeMap<Team, u16>>,
    player_budget: Option<BTreeMap<Slot, u16>>,
    scores: BTreeMap<Team, u32>,
    duration_ms: u64,
    score_limit: u32,
}

impl Revive {
    fn defeat(&mut self, roster: &mut Roster, slots: &[Slot]) -> Result<Outcome, Error> {
        roster.validate_defeat(slots)?;
        for &slot in slots {
            let opponent = roster.opponent(roster.player(slot)?.0);
            *self.scores.get_mut(&opponent).expect("validated team") += 1;
            roster.set_life(slot, Life::Ghost);
        }
        Ok(
            if self.scores.values().any(|&score| score >= self.score_limit) {
                self.score_outcome()
            } else {
                Outcome::Playing
            },
        )
    }

    fn score_outcome(&self) -> Outcome {
        let mut scores = self.scores.iter();
        let (&a, &a_score) = scores.next().expect("two teams");
        let (&b, &b_score) = scores.next().expect("two teams");
        match a_score.cmp(&b_score) {
            std::cmp::Ordering::Greater => Outcome::Winner(a),
            std::cmp::Ordering::Less => Outcome::Winner(b),
            std::cmp::Ordering::Equal => Outcome::Unresolved,
        }
    }

    fn respawn(&mut self, roster: &mut Roster, slot: Slot) -> Result<Outcome, Error> {
        let (team, life) = roster.player(slot)?;
        if life != Life::Ghost {
            return Err(Error::InvalidTransition);
        }
        let team_uses = self
            .team_budget
            .as_mut()
            .map(|budget| budget.get_mut(&team).expect("validated team"));
        let player_uses = self
            .player_budget
            .as_mut()
            .map(|budget| budget.get_mut(&slot).expect("validated slot"));
        if team_uses.as_deref() == Some(&0) || player_uses.as_deref() == Some(&0) {
            return Err(Error::NoRevives);
        }
        for uses in [team_uses, player_uses].into_iter().flatten() {
            *uses -= 1;
        }
        roster.set_life(slot, Life::Alive);
        Ok(Outcome::Playing)
    }
}

#[derive(Clone)]
struct Timebomb {
    carrier: Option<Slot>,
    remaining_ms: u64,
    fuse_ms: u64,
}

impl Timebomb {
    fn pass(&mut self, roster: &Roster, from: Slot, to: Slot) -> Result<Outcome, Error> {
        if self.carrier != Some(from) || from == to || roster.player(to)?.1 != Life::Alive {
            return Err(Error::InvalidTransition);
        }
        self.carrier = Some(to);
        Ok(Outcome::Playing)
    }

    fn advance(&mut self, roster: &mut Roster, mut elapsed: u64) -> Outcome {
        // Each expired fuse removes a player. Large time jumps are bounded by
        // the number of survivors, and produce the same state as smaller ticks.
        while elapsed >= self.remaining_ms {
            let carrier = self.carrier.take().expect("active bomb has a carrier");
            elapsed -= self.remaining_ms;
            roster.set_life(carrier, Life::Eliminated);
            let outcome = roster.live_team_outcome();
            if outcome != Outcome::Playing {
                self.remaining_ms = 0;
                return outcome;
            }
            // Explicit emulator policy: arm the next fuse on the lowest live slot.
            self.carrier = roster
                .players
                .iter()
                .find_map(|(&slot, &(_, life))| (life == Life::Alive).then_some(slot));
            self.remaining_ms = self.fuse_ms;
        }
        self.remaining_ms -= elapsed;
        Outcome::Playing
    }
}

#[derive(Clone)]
struct LeaderCapture {
    leaders: BTreeSet<Slot>,
}

impl LeaderCapture {
    fn defeat(&self, roster: &mut Roster, slots: &[Slot]) -> Result<Outcome, Error> {
        roster.validate_defeat(slots)?;
        for &slot in slots {
            roster.set_life(slot, Life::Eliminated);
        }
        let mut fallen = slots.iter().filter(|slot| self.leaders.contains(slot));
        Ok(match (fallen.next(), fallen.next()) {
            (None, _) => Outcome::Playing,
            (Some(&slot), None) => Outcome::Winner(roster.opponent(roster.player(slot)?.0)),
            (Some(_), Some(_)) => Outcome::Unresolved,
        })
    }
}

#[derive(Clone)]
enum State {
    Revive(Revive),
    Timebomb(Timebomb),
    LeaderCapture(LeaderCapture),
    // No native objective is measured. Only an explicit unresolved result
    // can end this round; the clock never supplies a result.
    EvaluateUnknown,
    Free1v7,
    CoOp,
}

impl State {
    fn new(mode: ModeKind, setup: Setup, roster: &Roster) -> Result<Self, Error> {
        let two_teams = roster.teams.len() == 2;
        Ok(match (mode, setup) {
            (
                ModeKind::Revive,
                Setup::Revive {
                    team_budget,
                    player_budget,
                    duration_ms,
                    score_limit,
                },
            ) if two_teams
                && duration_ms > 0
                && score_limit > 0
                && (team_budget.is_some() || player_budget.is_some()) =>
            {
                Self::Revive(Revive {
                    team_budget: team_budget
                        .map(|uses| roster.teams.iter().map(|&team| (team, uses)).collect()),
                    player_budget: player_budget
                        .map(|uses| roster.players.keys().map(|&slot| (slot, uses)).collect()),
                    scores: roster.teams.iter().map(|&team| (team, 0)).collect(),
                    duration_ms,
                    score_limit,
                })
            }
            (
                ModeKind::Timebomb,
                Setup::Timebomb {
                    initial_carrier,
                    fuse_ms,
                },
            ) if fuse_ms > 0 && roster.players.contains_key(&initial_carrier) => {
                Self::Timebomb(Timebomb {
                    carrier: Some(initial_carrier),
                    remaining_ms: fuse_ms,
                    fuse_ms,
                })
            }
            (ModeKind::LeaderCapture, Setup::LeaderCapture { leaders })
                if two_teams && leaders.len() == 2 =>
            {
                if roster.player(leaders[0])?.0 == roster.player(leaders[1])?.0 {
                    return Err(Error::InvalidSetup);
                }
                Self::LeaderCapture(LeaderCapture {
                    leaders: leaders.into_iter().collect(),
                })
            }
            (mode, Setup::Evaluate) if mode.evaluates_unknown_outcome() => Self::EvaluateUnknown,
            (ModeKind::Free1v7, Setup::Evaluate) if two_teams && roster.players.len() == 8 => {
                let solo = roster.teams.iter().any(|team| {
                    roster
                        .players
                        .values()
                        .filter(|(member, _)| member == team)
                        .count()
                        == 1
                });
                if !solo {
                    return Err(Error::InvalidRoster);
                }
                Self::Free1v7
            }
            (ModeKind::Free1v7, Setup::Evaluate) => return Err(Error::InvalidRoster),
            (ModeKind::CoOp, Setup::Evaluate) => Self::CoOp,
            _ => return Err(Error::InvalidSetup),
        })
    }

    fn apply(
        &mut self,
        roster: &mut Roster,
        now_ms: &mut u64,
        action: Action<'_>,
    ) -> Result<Outcome, Error> {
        if let Action::AdvanceTo(at) = action {
            let elapsed = at.checked_sub(*now_ms).ok_or(Error::ClockReversed)?;
            *now_ms = at;
            return Ok(match self {
                Self::Revive(state) if at >= state.duration_ms => state.score_outcome(),
                Self::Timebomb(state) => state.advance(roster, elapsed),
                _ => Outcome::Playing,
            });
        }
        match (self, action) {
            (Self::Revive(state), Action::Defeat(slots)) => state.defeat(roster, slots),
            (Self::Revive(state), Action::Respawn(slot)) => state.respawn(roster, slot),
            (Self::Timebomb(state), Action::PassBomb { from, to }) => state.pass(roster, from, to),
            (Self::LeaderCapture(state), Action::Defeat(slots)) => state.defeat(roster, slots),
            (Self::Free1v7, Action::Defeat(slots)) => {
                roster.validate_defeat(slots)?;
                for &slot in slots {
                    roster.set_life(slot, Life::Eliminated);
                }
                Ok(roster.live_team_outcome())
            }
            (Self::CoOp, Action::Objective(Some(team))) if roster.teams.contains(&team) => {
                Ok(Outcome::Winner(team))
            }
            (Self::CoOp, Action::Objective(Some(_))) => Err(Error::UnknownTeam),
            (Self::CoOp | Self::EvaluateUnknown, Action::Objective(None)) => {
                Ok(Outcome::Unresolved)
            }
            _ => Err(Error::UnsupportedAction),
        }
    }
}

#[derive(Clone)]
struct Round {
    id: u64,
    last_seq: u64,
    now_ms: u64,
    roster: Roster,
    state: State,
    outcome: Outcome,
}

#[derive(Clone)]
pub(crate) struct Runtime {
    mode: ModeKind,
    round: Option<Round>,
}

impl Policy {
    pub(crate) fn runtime_for(&self, selector: RoomTeamSelector) -> Result<Option<Runtime>, Error> {
        let Some(mode) = self.runtime.for_team(selector) else {
            return Ok(None);
        };
        // Export coverage alone cannot install an adapter. Family evaluation
        // presets can report an unresolved round without modeling its gameplay.
        let supported = match (mode, self.rules.runtime_win(mode)) {
            (ModeKind::Revive, Some(WinCondition::ReviveOutcome))
            | (ModeKind::Timebomb, Some(WinCondition::BombOutcome))
            | (ModeKind::LeaderCapture, Some(WinCondition::LeaderObjective))
            | (ModeKind::Free1v7, Some(WinCondition::AsymmetricElimination))
            | (ModeKind::CoOp, Some(WinCondition::TeamOutcome)) => true,
            (mode, Some(WinCondition::Unknown)) => mode.evaluates_unknown_outcome(),
            // RuleTable filters tentative CAPong from runtime_win. Allow its
            // named local evaluator only; this grants no native start route
            // and treats the room roster as input, without running map rules.
            (ModeKind::Capong, None) => self
                .catalog
                .mode(mode)
                .is_some_and(|entry| entry.wave == Wave::F2Candidate),
            _ => false,
        };
        if !supported {
            return Err(Error::UnsupportedMode);
        }
        Ok(Some(Runtime { mode, round: None }))
    }
}

impl Runtime {
    /// Install a fresh round atomically. Round IDs must increase, so a retry or
    /// delayed event cannot reset or mutate a newer round.
    pub(crate) fn start_round(
        &mut self,
        id: u64,
        players: &[Player],
        setup: Setup,
    ) -> Result<(), Error> {
        if id == 0 || self.round.as_ref().is_some_and(|round| id <= round.id) {
            return Err(Error::WrongRound);
        }
        let roster = Roster::new(players)?;
        let state = State::new(self.mode, setup, &roster)?;
        self.round = Some(Round {
            id,
            last_seq: 0,
            now_ms: 0,
            roster,
            state,
            outcome: Outcome::Playing,
        });
        Ok(())
    }

    /// Sequence starts at 1 each round. Replays leave all state unchanged;
    /// gaps and invalid actions do not consume the expected sequence.
    pub(crate) fn apply(
        &mut self,
        id: u64,
        sequence: u64,
        action: Action<'_>,
    ) -> Result<Applied, Error> {
        let round = self.round.as_mut().ok_or(Error::WrongRound)?;
        if id == 0 || id > round.id {
            return Err(Error::WrongRound);
        }
        if sequence == 0 {
            return Err(Error::SequenceGap);
        }
        if id < round.id || sequence <= round.last_seq {
            return Ok(Applied::Replay);
        }
        if sequence != round.last_seq + 1 {
            return Err(Error::SequenceGap);
        }
        if round.outcome != Outcome::Playing {
            return Err(Error::Finished);
        }
        round.outcome = round
            .state
            .apply(&mut round.roster, &mut round.now_ms, action)?;
        round.last_seq = sequence;
        Ok(Applied::Applied)
    }

    pub(crate) fn outcome(&self) -> Option<Outcome> {
        self.round.as_ref().map(|round| round.outcome)
    }

    pub(crate) fn life(&self, slot: Slot) -> Option<Life> {
        self.round
            .as_ref()?
            .roster
            .player(slot)
            .ok()
            .map(|(_, life)| life)
    }

    pub(crate) fn bomb(&self) -> Option<(Slot, u64)> {
        match &self.round.as_ref()?.state {
            State::Timebomb(state) => Some((state.carrier?, state.remaining_ms)),
            _ => None,
        }
    }

    pub(crate) fn score(&self, team: Team) -> Option<u32> {
        match &self.round.as_ref()?.state {
            State::Revive(state) => state.scores.get(&team).copied(),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests;
