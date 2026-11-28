//! Opt-in F3 policy for KR00967. Wire candidates remain capture-pending.

use kr_wire::kr00967::f3_pve::{F3MonActionProvenance, F3WireStatus};
use serde::de::{MapAccess, Visitor, value::MapAccessDeserializer};
use serde::{Deserialize, Deserializer};
use std::fmt;

/// Policy identifiers, independent of client GameType, map names and wire IDs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PveModeFamily {
    MonsterS1,
    MonsterS2,
    Mission,
    RPG,
    CAHero,
    HiddenCatch,
    Bonus,
    AIBattle,
    Popcade,
    SandBox,
    IamBoss,
    Tutorial,
    // The eight F3? map families in docs/content/f3-extended-pve-matrix.md.
    Antarctic,
    DragonPalace,
    EscapeMaze,
    GolemRun,
    IamWriggle,
    MineExp,
    Minibot,
    SavetheEarth,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PveAuthority {
    /// The host is a candidate source of proposals; this grants no decision rights.
    HostCandidate,
    ServerAuthoritative,
    Unmeasured,
}

impl PveModeFamily {
    /// The authority classification backed by the reviewed Wine dispatch record.
    pub fn authority(self) -> PveAuthority {
        measured_authority(self, REVIEWED_WINE_DISPATCH)
    }
}

struct WineDispatchEvidence {
    family: PveModeFamily,
    authority: PveAuthority,
    locator: &'static str,
}

// F3-1/F3-2 and the extended matrix leave KR routes and authority unknown.
// A future entry needs committed KR00967 Wine evidence for the actual mode,
// wire route and authority, with a control and a host swap where supported.
const REVIEWED_WINE_DISPATCH: &[WineDispatchEvidence] = &[];

fn measured_authority(family: PveModeFamily, evidence: &[WineDispatchEvidence]) -> PveAuthority {
    evidence
        .iter()
        .find(|record| record.family == family)
        .filter(|record| !record.locator.trim().is_empty())
        .map_or(PveAuthority::Unmeasured, |record| record.authority)
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct F3PvePolicy {
    pub enabled: bool,
}

impl<'de> Deserialize<'de> for F3PvePolicy {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Default, Deserialize)]
        #[serde(default, deny_unknown_fields)]
        struct Fields {
            enabled: bool,
        }

        struct PolicyObject;

        impl<'de> Visitor<'de> for PolicyObject {
            type Value = F3PvePolicy;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("an F3 policy object")
            }

            fn visit_map<M: MapAccess<'de>>(self, map: M) -> Result<Self::Value, M::Error> {
                let Fields { enabled } = Fields::deserialize(MapAccessDeserializer::new(map))?;
                Ok(F3PvePolicy { enabled })
            }
        }

        deserializer.deserialize_map(PolicyObject)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DispatchRejection {
    Disabled,
    WineEvidenceRequired(PveModeFamily),
}

impl F3PvePolicy {
    pub fn authority(&self, family: PveModeFamily) -> PveAuthority {
        family.authority()
    }

    pub fn authority_is_unmeasured(&self, family: PveModeFamily) -> bool {
        self.authority(family) == PveAuthority::Unmeasured
    }

    /// MonAction remains a static candidate until a live two-client capture
    /// measures delegation and report routing.  This helper is deliberately a
    /// policy gate only; room state and candidate bounds are checked by the
    /// runtime under its room lock.
    pub fn ensure_mon_action_enabled(&self) -> Result<(), DispatchRejection> {
        if self.enabled {
            Ok(())
        } else {
            Err(DispatchRejection::Disabled)
        }
    }

    pub const fn mon_action_status(&self) -> F3WireStatus {
        F3MonActionProvenance::CP405_STATIC.status
    }

    /// Checks the family gate. Callers must also verify the specific route and
    /// action; evidence for one action does not validate a family's other scripts.
    pub fn ensure_dispatch_allowed(&self, family: PveModeFamily) -> Result<(), DispatchRejection> {
        self.ensure_dispatch_with_evidence(family, REVIEWED_WINE_DISPATCH)
    }

    fn ensure_dispatch_with_evidence(
        &self,
        family: PveModeFamily,
        evidence: &[WineDispatchEvidence],
    ) -> Result<(), DispatchRejection> {
        if !self.enabled {
            return Err(DispatchRejection::Disabled);
        }
        if measured_authority(family, evidence) == PveAuthority::Unmeasured {
            return Err(DispatchRejection::WineEvidenceRequired(family));
        }
        Ok(())
    }
}

/// Keep one guard per room. The authenticated actor, current host and trusted
/// round epoch come from server state or are checked against it under the room lock.
#[derive(Debug)]
pub struct MonsterAttackGuard {
    current_round_epoch: u64,
    last_attack_sequence: Option<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MonsterAttackRejection {
    NotHost,
    WrongRoundEpoch,
    NonIncreasingRoundEpoch,
    NonIncreasingAttackSequence,
}

impl MonsterAttackGuard {
    /// KR00967 static anchor: `RollAttack::MonsterAttackReq` at `0x6C1BC0`
    /// and `0xF3B740` checks a host flag or matches `attackSeq`. This suggests
    /// a host role for the attack alone and does not establish mode authority.
    pub const AUTHORITY: PveAuthority = PveAuthority::HostCandidate;

    pub fn new(current_round_epoch: u64) -> Self {
        Self {
            current_round_epoch,
            last_attack_sequence: None,
        }
    }

    /// Only the server's round lifecycle may advance the epoch. Reentering
    /// the same epoch would reopen accepted sequences, so it is rejected.
    pub fn begin_round(&mut self, new_round_epoch: u64) -> Result<(), MonsterAttackRejection> {
        if new_round_epoch <= self.current_round_epoch {
            return Err(MonsterAttackRejection::NonIncreasingRoundEpoch);
        }
        self.current_round_epoch = new_round_epoch;
        self.last_attack_sequence = None;
        Ok(())
    }

    /// Checks and records a host proposal. As a server policy, this requires
    /// all three checks: the current host, the exact active epoch and a strictly
    /// increasing sequence. This does not apply damage or execute a script.
    pub fn accept(
        &mut self,
        actor_id: u32,
        current_host_id: u32,
        round_epoch: u64,
        attack_sequence: u64,
    ) -> Result<(), MonsterAttackRejection> {
        if actor_id != current_host_id {
            return Err(MonsterAttackRejection::NotHost);
        }
        if round_epoch != self.current_round_epoch {
            return Err(MonsterAttackRejection::WrongRoundEpoch);
        }
        if self
            .last_attack_sequence
            .is_some_and(|previous| attack_sequence <= previous)
        {
            return Err(MonsterAttackRejection::NonIncreasingAttackSequence);
        }
        self.last_attack_sequence = Some(attack_sequence);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dispatch_requires_opt_in_and_matching_wine_evidence() {
        let monster = PveModeFamily::MonsterS1;
        let mission = PveModeFamily::Mission;
        let policy = F3PvePolicy { enabled: true };
        let mut evidence = [WineDispatchEvidence {
            family: monster,
            authority: PveAuthority::HostCandidate,
            locator: "test fixture",
        }];

        assert_eq!(
            F3PvePolicy::default().ensure_dispatch_with_evidence(monster, &evidence),
            Err(DispatchRejection::Disabled)
        );
        assert_eq!(
            policy.ensure_dispatch_with_evidence(monster, &evidence),
            Ok(())
        );
        assert_eq!(
            policy.ensure_dispatch_with_evidence(mission, &evidence),
            Err(DispatchRejection::WineEvidenceRequired(mission))
        );
        evidence[0].authority = PveAuthority::ServerAuthoritative;
        assert_eq!(
            policy.ensure_dispatch_with_evidence(monster, &evidence),
            Ok(())
        );
        evidence[0].authority = PveAuthority::Unmeasured;
        assert_eq!(
            policy.ensure_dispatch_with_evidence(monster, &evidence),
            Err(DispatchRejection::WineEvidenceRequired(monster))
        );
        evidence[0].authority = PveAuthority::ServerAuthoritative;
        evidence[0].locator = " ";
        assert_eq!(
            policy.ensure_dispatch_with_evidence(monster, &evidence),
            Err(DispatchRejection::WineEvidenceRequired(monster))
        );
    }

    #[test]
    fn sequence_wrap_requires_a_new_epoch() {
        let mut guard = MonsterAttackGuard::new(5);
        assert_eq!(guard.accept(1, 1, 5, u64::MAX), Ok(()));
        assert_eq!(
            guard.accept(1, 1, 5, 0),
            Err(MonsterAttackRejection::NonIncreasingAttackSequence)
        );
        assert_eq!(guard.begin_round(6), Ok(()));
        assert_eq!(guard.accept(1, 1, 6, 0), Ok(()));
    }
}
