//! Design contracts for opt-in modes. A named condition identifies what needs
//! an authority/handler; it does not establish timing, results or KR wire IDs.
use super::{Mode, ModeKind};
use serde::Deserialize;
use std::{collections::BTreeMap, io};

const DESCRIPTORS: &str = include_str!("rules.json");

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
pub(super) enum Wave {
    F2,
    #[serde(rename = "F2?")]
    F2Candidate,
    F3,
}

impl Wave {
    pub(super) fn in_f2(self) -> bool {
        matches!(self, Self::F2 | Self::F2Candidate)
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ModeRules {
    group: ModeKind,
    wave: Wave,
    win: WinCondition,
    tie: TiePolicy,
    respawn: RespawnPolicy,
    timer: TimerPolicy,
    reset: ResetPolicy,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum WinCondition {
    Unknown,
    ReviveOutcome,
    BombOutcome,
    LeaderObjective,
    TeamOutcome,
    AsymmetricElimination,
    MissionObjective,
    NotApplicable,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum TiePolicy {
    Unknown,
    NotApplicable,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum RespawnPolicy {
    Unknown,
    Required,
    NotApplicable,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum TimerPolicy {
    Unknown,
    BombCountdown,
    NotApplicable,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum ResetPolicy {
    Round,
    RoundAndRespawn,
    RoundAndBomb,
    RoundAndLeader,
    QueueHandoff,
}

impl ModeRules {
    fn validate(&self) -> io::Result<()> {
        use ResetPolicy::*;

        let coherent = if self.group == ModeKind::Matchmaking {
            self.win == WinCondition::NotApplicable
                && self.tie == TiePolicy::NotApplicable
                && self.respawn == RespawnPolicy::NotApplicable
                && self.timer == TimerPolicy::NotApplicable
                && self.reset == QueueHandoff
        } else {
            let expected_reset = match self.win {
                WinCondition::ReviveOutcome => RoundAndRespawn,
                WinCondition::BombOutcome => RoundAndBomb,
                WinCondition::LeaderObjective => RoundAndLeader,
                _ => Round,
            };
            self.win != WinCondition::NotApplicable
                && self.tie == TiePolicy::Unknown
                && self.respawn
                    == if expected_reset == RoundAndRespawn {
                        RespawnPolicy::Required
                    } else {
                        RespawnPolicy::Unknown
                    }
                && self.timer
                    == if expected_reset == RoundAndBomb {
                        TimerPolicy::BombCountdown
                    } else {
                        TimerPolicy::Unknown
                    }
                && self.reset == expected_reset
        };
        if !coherent {
            return Err(io::Error::other(format!(
                "inconsistent F2 rule descriptor: {:?}",
                self.group
            )));
        }
        Ok(())
    }
}

pub(super) struct RuleTable(BTreeMap<ModeKind, ModeRules>);

impl RuleTable {
    pub(super) fn load(modes: &[Mode]) -> io::Result<Self> {
        Self::parse(DESCRIPTORS, modes)
    }

    fn parse(json: &str, modes: &[Mode]) -> io::Result<Self> {
        let rows: Vec<ModeRules> = serde_json::from_str(json).map_err(io::Error::other)?;
        let mut table = BTreeMap::new();
        for row in rows {
            row.validate()?;
            if table.insert(row.group, row).is_some() {
                return Err(io::Error::other("duplicate F2 rule descriptor"));
            }
        }
        let mut seen = std::collections::BTreeSet::new();
        for mode in modes {
            if !seen.insert(mode.ruleset_group)
                || !table
                    .get(&mode.ruleset_group)
                    .is_some_and(|rules| rules.wave == mode.wave)
            {
                return Err(io::Error::other(
                    "F2 catalog mode does not match rule table",
                ));
            }
        }
        if seen.len() != table.len() || seen.is_empty() {
            return Err(io::Error::other("F2 rule table does not match catalog"));
        }
        Ok(Self(table))
    }

    pub(super) fn check_start(&self, group: ModeKind) -> Result<(), &'static str> {
        let rules = self.0.get(&group).ok_or("mode_outside_f2")?;
        if !rules.wave.in_f2() {
            return Err("mode_outside_f2");
        }
        // An opt-in descriptor and even an exact map match supply no measured
        // start dispatch, event authority or rules implementation.
        Err("f2_start_dispatch_unmeasured")
    }
}

#[cfg(test)]
mod tests;
