//! Server-owned F2 commands and the single supported wire projection. Normal
//! KR room startup provides the roster; no client packet is F2 event authority.
use super::{
    Config, Policy, TENTATIVE_MAP_FAMILIES,
    runtime::{Action, Applied, Error, Life, Outcome, Player, Runtime},
};
use crate::{Reply, room::Member, room_packets::packet};
use kr_wire::kr00967::{RebirthToGhost, RoomTeamSelector};
use serde::{Deserialize, Serialize};
use std::io;

pub use super::runtime::Setup;

impl Config {
    pub(super) fn check_extended_activation(&self) -> io::Result<()> {
        // Reject unmeasured requests before choosing any legacy or F2 room
        // path. No requested policy may silently fall back to a normal room.
        self.extended_rules.check_activation()?;
        if let Some(requested) = &self.candidate_map_family {
            let known = TENTATIVE_MAP_FAMILIES
                .iter()
                .any(|(family, _)| *family == requested);
            return Err(io::Error::other(if known {
                "F2 candidate map family activation is unmeasured"
            } else {
                "unknown F2 candidate map family"
            }));
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "command", rename_all = "snake_case", deny_unknown_fields)]
pub enum Command {
    Start {
        round: u64,
        setup: Setup,
    },
    Apply {
        round: u64,
        sequence: u64,
        event: Event,
    },
}

impl Command {
    pub(crate) fn round(&self) -> u64 {
        match *self {
            Self::Start { round, .. } | Self::Apply { round, .. } => round,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum Event {
    AdvanceTo { milliseconds: u64 },
    Defeat { slots: Vec<u8> },
    Respawn { slot: u8 },
    PassBomb { from: u8, to: u8 },
    Objective { winner: Option<u8> },
}

impl Event {
    fn action(&self) -> Action<'_> {
        match *self {
            Self::AdvanceTo { milliseconds } => Action::AdvanceTo(milliseconds),
            Self::Defeat { ref slots } => Action::Defeat(slots),
            Self::Respawn { slot } => Action::Respawn(slot),
            Self::PassBomb { from, to } => Action::PassBomb { from, to },
            Self::Objective { winner } => Action::Objective(winner),
        }
    }
}

#[derive(Serialize)]
pub(crate) struct RoundView {
    round: u64,
    replay: bool,
    outcome: Outcome,
    players: Vec<PlayerView>,
    bomb: Option<(u8, u64)>,
}

#[derive(Serialize)]
struct PlayerView {
    slot: u8,
    team: u8,
    life: Life,
    team_score: Option<u32>,
}

/// Staged with its packet batch so a failed reservation cannot consume an event.
#[derive(Clone)]
pub(crate) struct RoomMode(pub(super) Runtime);

fn runtime_error(error: Error) -> io::Error {
    io::Error::other(format!("F2 runtime: {error:?}"))
}

impl Policy {
    pub(crate) fn room_mode(&self, team: RoomTeamSelector) -> io::Result<Option<RoomMode>> {
        // Both named and GameType presets have already resolved to ModeKind.
        self.runtime_for(team)
            .map(|runtime| runtime.map(RoomMode))
            .map_err(runtime_error)
    }
}

impl RoomMode {
    pub(crate) fn apply<'a>(
        &mut self,
        command: Command,
        members: impl Iterator<Item = &'a Member>,
    ) -> io::Result<(RoundView, Vec<(String, Reply)>)> {
        let mut roster: Vec<_> = members
            .map(|m| Player {
                slot: m.slot,
                team: m.team,
            })
            .collect();
        roster.sort_unstable_by_key(|p| p.slot);
        let round = command.round();
        let mut replies = Vec::new();
        let replay = match command {
            Command::Start { setup, .. } => {
                self.0
                    .start_round(round, &roster, setup)
                    .map_err(runtime_error)?;
                false
            }
            Command::Apply {
                sequence, event, ..
            } => {
                let applied = self
                    .0
                    .apply(round, sequence, event.action())
                    .map_err(runtime_error)?;
                if applied == Applied::Applied
                    && let Event::Defeat { slots } = event
                {
                    for slot in slots {
                        if self.0.life(slot) == Some(Life::Ghost) {
                            replies.push((
                                "rebirth_to_ghost185".into(),
                                packet(
                                    RebirthToGhost::OPCODE,
                                    RebirthToGhost { selector: slot }.encode()?,
                                ),
                            ));
                        }
                    }
                }
                applied == Applied::Replay
            }
        };
        let players = roster
            .into_iter()
            .map(|Player { slot, team }| {
                Ok(PlayerView {
                    slot,
                    team,
                    life: self
                        .0
                        .life(slot)
                        .ok_or_else(|| io::Error::other("stale F2 roster"))?,
                    team_score: self.0.score(team),
                })
            })
            .collect::<io::Result<_>>()?;
        Ok((
            RoundView {
                round,
                replay,
                outcome: self
                    .0
                    .outcome()
                    .ok_or_else(|| io::Error::other("missing F2 outcome"))?,
                players,
                bomb: self.0.bomb(),
            },
            replies,
        ))
    }
}
