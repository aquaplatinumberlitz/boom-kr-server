//! Room rules of the measured diagnostic profiles (RUST-R1a).
//! Single home for what a fixed room supports. Flags, tables and startup checks
//! are moved here unchanged from room.rs/room_io.rs; this is emulator policy
//! selection, not original KR authority. New rules belong here, not in new
//! per-capacity branches of the packet handlers.
use crate::room_io::Config;
use std::io;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RoomRules {
    /// Opt-in measured CP201 capacity1/2/1/1, power1, design fuse6 seconds.
    pub(crate) four_owner_bombs: bool,
    pub(crate) four_life_reports: bool,
    pub(crate) four_results: bool,
    pub(crate) four_abort_room: bool,
    /// CP208–210 explicit fixed8 baseline; not four abort/items/terrain.
    pub(crate) eight_battle: bool,
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
                four_owner_bombs: cfg.four_owner_bombs,
                four_life_reports: cfg.four_life_reports,
                four_results: cfg.four_results,
                four_abort_room: cfg.four_abort_room,
                eight_battle: cfg.eight_battle,
                ninth_admission_probe: cfg.ninth_admission_probe,
            },
            count,
        ))
    }
    pub(crate) fn life_enabled(&self) -> bool {
        self.four_life_reports || self.eight_battle
    }
    pub(crate) fn team_results(&self) -> bool {
        self.four_results || self.eight_battle
    }
    pub(crate) fn owner_bombs(&self) -> bool {
        self.four_owner_bombs || self.eight_battle
    }
}

/// Measured per-slot bomb capacity at round start; CP209 actual baseline for eight.
pub(crate) fn default_bomb_capacity(capacity: usize) -> Vec<usize> {
    match capacity {
        8 => vec![1, 2, 1, 1, 2, 1, 1, 1],
        4 => vec![1, 2, 1, 1],
        _ => vec![1, 2],
    }
}

/// Eight setup can hold own106/110 + seven peer106 + map = ten batches.
/// Two/four queues stay at eight; the local pending bound stays sixteen.
pub(crate) fn recipient_queue(capacity: usize) -> usize {
    if capacity == 8 { 16 } else { 8 }
}

fn err(s: &str) -> io::Error {
    io::Error::other(s.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_rules_follow_the_measured_profiles() {
        let two = RoomRules::default();
        assert!(!two.life_enabled() && !two.team_results() && !two.owner_bombs());
        let four = RoomRules {
            four_owner_bombs: true,
            four_life_reports: true,
            four_results: true,
            ..RoomRules::default()
        };
        assert!(four.life_enabled() && four.team_results() && four.owner_bombs());
        let eight = RoomRules {
            eight_battle: true,
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
}
