//! Design contracts for opt-in modes. A named condition identifies what needs
//! an authority/handler; it does not establish timing, results or KR wire IDs.
use super::{Mode, ModeKind};
use kr_wire::kr00967::GameTypeId;
use serde::Deserialize;
use std::{collections::BTreeMap, io};

const DESCRIPTORS: &str = include_str!("rules.json");

/// Operator choices from the KR00967 extended modes matrix (design/candidate).
/// These fields describe requests, without granting any room or lobby rights.
#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub(super) struct ExtendedRules {
    no_item: bool,    // 노템전
    no_shop: bool,    // 노샵
    basic_shop: bool, // 기샵
    super_host: bool,
    vote_kick: bool,
    priority_room: Option<PriorityRoom>,
    ladder_season: bool, // Seasonal 1:1 (크아짱), distinct from the Ladder preset.
    channel_kind: Option<ChannelKind>,
    big_battle_12: bool, // 대난투 has no confirmed relation to the big_battle group.
}

#[derive(Deserialize)]
#[serde(try_from = "u32")]
struct PriorityRoom(u8);

impl TryFrom<u32> for PriorityRoom {
    type Error = &'static str;

    fn try_from(slot: u32) -> Result<Self, Self::Error> {
        if !(1..=50).contains(&slot) {
            return Err("priority_room must be in 1..=50 (S01–S50)");
        }
        Ok(Self(slot as u8))
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum ChannelKind {
    Beginner,
    Event,
    NoItem,
    Practice,
}

impl ExtendedRules {
    fn validate(&self) -> Result<(), &'static str> {
        // Each label selects a whole item preset. Their composition and the
        // relationship between a channel and room item permissions are unknown.
        if [self.no_item, self.no_shop, self.basic_shop]
            .into_iter()
            .filter(|&selected| selected)
            .count()
            > 1
        {
            return Err("no_item, no_shop and basic_shop are mutually exclusive F2 item presets");
        }
        Ok(())
    }

    pub(super) fn check_activation(&self) -> io::Result<()> {
        self.validate().map_err(io::Error::other)?;
        if let Some(PriorityRoom(slot)) = self.priority_room {
            return Err(io::Error::other(format!(
                "F2 priority_room S{slot:02} reservation is unmeasured"
            )));
        }
        for (requested, policy) in [
            (self.no_item, "no_item"),
            (self.no_shop, "no_shop"),
            (self.basic_shop, "basic_shop"),
            (self.super_host, "super_host"),
            (self.vote_kick, "vote_kick"),
            (self.ladder_season, "ladder_season"),
            (self.channel_kind.is_some(), "channel_kind"),
            (self.big_battle_12, "big_battle_12"),
        ] {
            if requested {
                return Err(io::Error::other(format!(
                    "F2 {policy} policy is unmeasured"
                )));
            }
        }
        Ok(())
    }
}

impl ModeKind {
    /// Operator aliases for recognizable dialog labels at KR00967 0xC6C080.
    /// This correspondence is policy, and supplies no native dispatch or
    /// runtime support. Distinct IDs sharing a label are not interchangeable.
    /// Family evaluation presets have no separate wire IDs. SuperFighter keeps
    /// its existing alias, and Popcade's alias does not select an A/B/C family.
    fn game_type_id(self) -> Option<GameTypeId> {
        Some(match self {
            Self::Revive => GameTypeId::Rehab,
            Self::Hedgehog => GameTypeId::Hedgehog,
            Self::Timebomb => GameTypeId::TimeBomb,
            Self::SuperFighter => GameTypeId::SuperFighter,
            Self::Popcade => GameTypeId::Popcade,
            _ => return None,
        })
    }

    fn from_game_type_id(id: GameTypeId) -> Option<Self> {
        match id {
            GameTypeId::Rehab => Some(Self::Revive),
            GameTypeId::Hedgehog => Some(Self::Hedgehog),
            GameTypeId::TimeBomb => Some(Self::Timebomb),
            GameTypeId::SuperFighter => Some(Self::SuperFighter),
            GameTypeId::Popcade => Some(Self::Popcade),
            _ => None,
        }
    }
}

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

// These operator evaluation presets have no mode row in the shipped JSON table.
const EVALUATION_RULES: &[ModeRules] = &[
    evaluation_rules(ModeKind::Coloring),
    evaluation_rules(ModeKind::Gumiho),
    evaluation_rules(ModeKind::Hamster),
    evaluation_rules(ModeKind::PopcadeA),
    evaluation_rules(ModeKind::PopcadeB),
    evaluation_rules(ModeKind::PopcadeC),
];

const fn evaluation_rules(group: ModeKind) -> ModeRules {
    ModeRules {
        group,
        wave: Wave::F2Candidate,
        win: WinCondition::Unknown,
        tie: TiePolicy::Unknown,
        respawn: RespawnPolicy::Unknown,
        timer: TimerPolicy::Unknown,
        reset: ResetPolicy::Round,
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum WinCondition {
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

    pub(super) fn mode_for_game_type(&self, id: GameTypeId) -> Option<ModeKind> {
        let mode = ModeKind::from_game_type_id(id)?;
        (mode.game_type_id() == Some(id) && self.descriptor(mode).is_some()).then_some(mode)
    }

    fn descriptor(&self, group: ModeKind) -> Option<&ModeRules> {
        self.0
            .get(&group)
            .or_else(|| EVALUATION_RULES.iter().find(|rules| rules.group == group))
    }

    fn parse(json: &str, modes: &[Mode]) -> io::Result<Self> {
        let rows: Vec<ModeRules> = serde_json::from_str(json).map_err(io::Error::other)?;
        let mut table = BTreeMap::new();
        for rules in EVALUATION_RULES {
            rules.validate()?;
        }
        for row in rows {
            row.validate()?;
            if EVALUATION_RULES
                .iter()
                .any(|rules| rules.group == row.group)
                || table.insert(row.group, row).is_some()
            {
                return Err(io::Error::other("duplicate F2 rule descriptor"));
            }
        }
        let table = Self(table);
        let mut seen = std::collections::BTreeSet::new();
        for mode in modes {
            if !seen.insert(mode.ruleset_group)
                || !table
                    .descriptor(mode.ruleset_group)
                    .is_some_and(|rules| rules.wave == mode.wave)
            {
                return Err(io::Error::other(
                    "F2 catalog mode does not match rule table",
                ));
            }
        }
        if seen.len() != table.0.len() + EVALUATION_RULES.len() {
            return Err(io::Error::other("F2 rule table does not match catalog"));
        }
        Ok(table)
    }

    pub(super) fn check_start(&self, group: ModeKind) -> Result<(), &'static str> {
        let rules = self.descriptor(group).ok_or("mode_outside_f2")?;
        if !rules.wave.in_f2() {
            return Err("mode_outside_f2");
        }
        // An opt-in descriptor and even an exact map match supply no measured
        // start dispatch, event authority or rules implementation.
        Err("f2_start_dispatch_unmeasured")
    }

    pub(super) fn runtime_win(&self, group: ModeKind) -> Option<WinCondition> {
        let rules = self.descriptor(group)?;
        (rules.wave == Wave::F2 || (rules.wave == Wave::F2Candidate && group.evaluates_family()))
            .then_some(rules.win)
    }
}

#[cfg(test)]
mod tests;
