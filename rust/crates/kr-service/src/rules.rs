//! Policies selected by the measured diagnostic profiles (RUST-R1b).
//! Legacy JSON flags are interpreted only at admission/configuration time.
//! These are emulator choices, not recovered original KR authority.
use crate::room_io::Config;
use std::io;

/// Legacy profiles retain identity-1 slots; general admission allocates a
/// vacant slot independently of the backend/session player identity.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SlotAllocation {
    #[default]
    LegacyIdentity,
    FirstVacant,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum ResultPolicy {
    #[default]
    Disabled,
    Duel,
    Teams,
    Players,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum DeparturePolicy {
    #[default]
    Leave,
    Forfeit,
    UnrankedReturn,
    FenceActive,
    ContinuePlayers,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum BombPolicy {
    #[default]
    Basic,
    Disabled,
    OwnerReports,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum LifePolicy {
    #[default]
    Observe,
    DuelReports,
    TeamReports,
}

impl LifePolicy {
    pub(crate) fn reports_enabled(self) -> bool {
        self != Self::Observe
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum TeamPolicy {
    #[default]
    Any,
    BalancedManner,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum EndpointPolicy {
    #[default]
    LegacyDirect,
    Relay,
    AuthenticatedRelay,
}

/// Active roster policy, independent of the room's admission capacity.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum RoundRoster {
    #[default]
    Full,
    Contiguous(usize),
    /// Current members, including vacant middle slots. Opt-in player round.
    Players,
}
impl RoundRoster {
    pub(crate) fn size(self, capacity: usize) -> usize {
        match self {
            Self::Full | Self::Players => capacity,
            Self::Contiguous(count) => count,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RoomRules {
    pub(crate) item_policy: crate::pickups::Policy,
    pub(crate) life_actions: crate::life_actions::Policy,
    pub(crate) roster: RoundRoster,
    pub(crate) endpoint: EndpointPolicy,
    pub(crate) slot_allocation: SlotAllocation,
    pub(crate) result: ResultPolicy,
    pub(crate) departure: DeparturePolicy,
    pub(crate) bomb: BombPolicy,
    pub(crate) life: LifePolicy,
    pub(crate) teams: TeamPolicy,
    /// Staged room-only choices. Battle with chosen characters needs its own
    /// measured Start/stat projection before this policy can serve a round.
    pub(crate) player_choices: bool,
    /// Opt-in ninth diagnostic lobby actor for the full-room negative consumer.
    pub(crate) ninth_admission_probe: bool,
}

impl RoomRules {
    /// Validates the opt-in combination against the actor count and returns the
    /// rules with the room member count. Check order and messages are part of
    /// the startup contract exercised by the pre-bind guards.
    pub(crate) fn from_config(cfg: &Config, actor_count: usize) -> io::Result<(Self, usize)> {
        let count = if cfg.ninth_admission_probe {
            if actor_count != 9 || !cfg.eight_battle {
                return Err(err(
                    "ninth admission probe requires nine lobby actors and eight battle",
                ));
            }
            8
        } else {
            actor_count
        };

        if cfg.eight_team_layout.is_some() && (!cfg.eight_battle || count != 8) {
            return Err(err("eight team layout requires opt-in eight battle"));
        }
        if cfg.eight_battle && count != 8 {
            return Err(err("eight battle requires eight diagnostic actors"));
        }
        if cfg.four_abort_room && (count != 4 || !cfg.four_results) {
            return Err(err("four abort-room requires opt-in four results"));
        }
        if cfg.four_results && (count != 4 || !cfg.four_life_reports || !cfg.four_owner_bombs) {
            return Err(err("four results require opt-in four life and owner bombs"));
        }
        if cfg.four_life_reports && (count != 4 || !cfg.four_owner_bombs) {
            return Err(err("four life reports require opt-in four-owner bombs"));
        }
        if cfg.four_owner_bombs && count != 4 {
            return Err(err("four-owner bombs require the four diagnostic profile"));
        }
        Ok((
            Self {
                roster: RoundRoster::Full,
                endpoint: cfg.endpoint_policy,
                slot_allocation: cfg.slot_allocation,
                result: if cfg.four_results || cfg.eight_battle {
                    ResultPolicy::Teams
                } else if cfg.battle.is_some() {
                    ResultPolicy::Duel
                } else {
                    ResultPolicy::Disabled
                },
                departure: match count {
                    8 => DeparturePolicy::FenceActive,
                    _ if cfg.four_abort_room => DeparturePolicy::UnrankedReturn,
                    _ if cfg.battle.is_some() => DeparturePolicy::Forfeit,
                    _ => DeparturePolicy::Leave,
                },
                bomb: if cfg.four_owner_bombs || cfg.eight_battle {
                    BombPolicy::OwnerReports
                } else if matches!(count, 4 | 8) {
                    BombPolicy::Disabled
                } else {
                    BombPolicy::Basic
                },
                life: if cfg.four_life_reports || cfg.eight_battle {
                    LifePolicy::TeamReports
                } else if cfg.battle.is_some() {
                    LifePolicy::DuelReports
                } else {
                    LifePolicy::Observe
                },
                teams: if cfg.eight_battle {
                    TeamPolicy::BalancedManner
                } else {
                    TeamPolicy::Any
                },
                ninth_admission_probe: cfg.ninth_admission_probe,
                life_actions: cfg.life_actions,
                item_policy: cfg.item_policy,
                player_choices: false,
            },
            count,
        ))
    }
    pub(crate) fn life_enabled(&self) -> bool {
        self.life == LifePolicy::TeamReports
    }
    pub(crate) fn team_results(&self) -> bool {
        matches!(self.result, ResultPolicy::Teams | ResultPolicy::Players)
    }
    pub(crate) fn owner_bombs(&self) -> bool {
        self.bomb == BombPolicy::OwnerReports
    }
}

/// Additional source rosters are for DB-owned managed player rounds only.
/// Legacy diagnostic serving keeps its measured profiles and ninth probe.
pub(crate) fn template_roster_supported(count: usize, managed_players: bool, ninth: bool) -> bool {
    matches!(count, 2 | 4 | 8)
        || (managed_players && (2..=8).contains(&count))
        || (ninth && count == 9)
}

/// Retained diagnostic defaults, truncated to the prototype's bounded size.
/// Managed player rounds replace these with the selected character's stats.
pub(crate) fn default_bomb_capacity(capacity: usize) -> Vec<usize> {
    [1, 2, 1, 1, 2, 1, 1, 1]
        .into_iter()
        .take(capacity)
        .collect()
}

/// Eight setup can hold own106/110 + seven peer106 + map = ten batches.
/// Two/four queues stay at eight; the local pending bound stays sixteen.
pub(crate) fn recipient_queue(capacity: usize) -> usize {
    if capacity == 8 { 16 } else { 8 }
}

fn err(s: &str) -> io::Error {
    io::Error::other(s.to_owned())
}

/// Measured report shapes and initial power by round profile. Observation-only
/// messages carry no authority or effect merely because their length is known.
pub(crate) struct ReportLimits {
    pub power: Vec<u8>,
    pub opaque167: &'static [usize],
}
impl ReportLimits {
    // Managed player rounds: observed native envelopes only, no ACK/effects.
    // Patrit14 capture adds body52; this does not assign semantics to167.
    pub(crate) const PLAYER_OPAQUE167: &'static [usize] = &[4, 12, 16, 22, 32, 42, 52];

    pub(crate) fn measured(count: usize) -> io::Result<Self> {
        match count {
            2 => Ok(Self {
                power: vec![1, 1],
                opaque167: &[4, 12, 16, 22, 32, 42],
            }),
            4 => Ok(Self {
                power: vec![1, 1, 1, 1],
                opaque167: &[4, 16, 22, 32],
            }),
            8 => Ok(Self {
                power: vec![1, 1, 1, 1, 1, 1, 2, 2],
                opaque167: &[4, 22, 32, 42],
            }),
            _ => Err(err("unmeasured report profile")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_rules_follow_the_measured_profiles() {
        let two = RoomRules::default();
        assert!(!two.life_enabled() && !two.team_results() && !two.owner_bombs());
        let four = RoomRules {
            bomb: BombPolicy::OwnerReports,
            life: LifePolicy::TeamReports,
            result: ResultPolicy::Teams,
            ..RoomRules::default()
        };
        assert!(four.life_enabled() && four.team_results() && four.owner_bombs());
        let eight = RoomRules {
            bomb: BombPolicy::OwnerReports,
            life: LifePolicy::TeamReports,
            result: ResultPolicy::Teams,
            departure: DeparturePolicy::FenceActive,
            teams: TeamPolicy::BalancedManner,
            ..RoomRules::default()
        };
        assert!(eight.life_enabled() && eight.team_results() && eight.owner_bombs());
    }

    #[test]
    fn tables_match_the_measured_capacities() {
        assert_eq!(default_bomb_capacity(2), [1, 2]);
        assert_eq!(default_bomb_capacity(4), [1, 2, 1, 1]);
        assert_eq!(default_bomb_capacity(8), [1, 2, 1, 1, 2, 1, 1, 1]);
        assert_eq!(
            (recipient_queue(2), recipient_queue(4), recipient_queue(8)),
            (8, 8, 16)
        );
    }

    #[test]
    fn additional_template_rosters_require_managed_players() {
        for count in [3, 5, 6, 7] {
            assert!(!template_roster_supported(count, false, false));
            assert!(!template_roster_supported(count, false, true));
            assert!(template_roster_supported(count, true, false));
        }
        for count in [0, 1, 9, 10] {
            assert!(!template_roster_supported(count, true, false));
        }
        assert!(template_roster_supported(9, false, true));
    }
}
