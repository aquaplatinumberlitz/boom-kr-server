//! Fail-closed, opt-in F3 PvE session state.
//!
//! The wire bodies are static candidates only.  In particular, the stage
//! dispatch at `0xE5DE9C` is an opcode-minus-20 jump-table candidate pending
//! live Wine capture; this module does not claim Wine verification.

use crate::f3_pve_policy::{
    DispatchRejection, F3PvePolicy, MonsterAttackGuard, MonsterAttackRejection, PveModeFamily,
};
use kr_wire::kr00967::f3_pve::{F3JumpTableProvenance, MonsterAttackReqCandidate};

pub const MAX_STAGES: u16 = 64;
pub const MAX_WAVES: u16 = 64;
pub const MAX_MONSTERS_PER_WAVE: usize = 128;
pub const MAX_MONSTER_HP: u32 = 10_000_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    pub stages: u16,
    pub waves: u16,
    pub monsters_per_wave: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            stages: MAX_STAGES,
            waves: MAX_WAVES,
            monsters_per_wave: MAX_MONSTERS_PER_WAVE,
        }
    }
}

impl Limits {
    pub fn new(stages: u16, waves: u16, monsters_per_wave: usize) -> Result<Self, RuntimeError> {
        if stages == 0
            || stages > MAX_STAGES
            || waves == 0
            || waves > MAX_WAVES
            || monsters_per_wave == 0
            || monsters_per_wave > MAX_MONSTERS_PER_WAVE
        {
            return Err(RuntimeError::CapacityExceeded);
        }
        Ok(Self {
            stages,
            waves,
            monsters_per_wave,
        })
    }
}

/// An explicit, caller-owned assertion that the route was measured.  The
/// default is missing; static candidate metadata alone never sets this bit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DispatchEvidence {
    present: bool,
    pub provenance: F3JumpTableProvenance,
}

impl DispatchEvidence {
    pub const fn missing() -> Self {
        Self {
            present: false,
            provenance: F3JumpTableProvenance::C_GAME_STAGE,
        }
    }

    /// Construct from an evidence record owned by the caller. This crate does
    /// not manufacture or verify the record, and static provenance remains a
    /// candidate until the caller supplies its own live-Wine record.
    pub const fn present(provenance: F3JumpTableProvenance) -> Self {
        Self {
            present: true,
            provenance,
        }
    }

    pub const fn is_present(self) -> bool {
        self.present
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MonsterSpawn {
    pub id: u32,
    pub hp: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MonsterAttack {
    pub actor_id: u32,
    pub round_epoch: u64,
    pub attack_sequence: u64,
    pub monster_id: u32,
    pub damage: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Idle,
    Briefing { stage: u16 },
    Active { stage: u16, wave: u16 },
    Cleared { stage: u16 },
    Failed { stage: u16 },
    Aborted { stage: Option<u16> },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StageResult {
    pub stage: u16,
    pub waves: u16,
    pub defeated: u32,
    pub damage: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeError {
    DispatchDisabled,
    WineDispatchEvidenceRequired,
    InvalidTransition,
    CapacityExceeded,
    InvalidStage,
    InvalidWave,
    EmptyWave,
    InvalidMonster,
    DuplicateMonster,
    NotHost,
    WrongRoundEpoch,
    NonIncreasingSequence,
    UnknownMonster,
    InvalidDamage,
    MonsterAlreadyDefeated,
    InvalidAttackState,
    InvalidRoundEpoch,
}

impl From<DispatchRejection> for RuntimeError {
    fn from(value: DispatchRejection) -> Self {
        match value {
            DispatchRejection::Disabled => Self::DispatchDisabled,
            DispatchRejection::WineEvidenceRequired(_) => Self::WineDispatchEvidenceRequired,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Monster {
    id: u32,
    hp: u32,
    max_hp: u32,
}

pub struct F3PveRuntime {
    policy: F3PvePolicy,
    family: PveModeFamily,
    evidence: DispatchEvidence,
    limits: Limits,
    phase: Phase,
    host_id: Option<u32>,
    round_epoch: Option<u64>,
    attack_guard: MonsterAttackGuard,
    monsters: Vec<Monster>,
    waves_started: u16,
    defeated: u32,
    damage: u64,
    last_result: Option<StageResult>,
}

impl F3PveRuntime {
    pub fn new(policy: F3PvePolicy, family: PveModeFamily, evidence: DispatchEvidence) -> Self {
        Self::with_limits(policy, family, evidence, Limits::default())
            .expect("default F3 PvE limits are valid")
    }

    pub fn with_limits(
        policy: F3PvePolicy,
        family: PveModeFamily,
        evidence: DispatchEvidence,
        limits: Limits,
    ) -> Result<Self, RuntimeError> {
        Ok(Self {
            policy,
            family,
            evidence,
            limits,
            phase: Phase::Idle,
            host_id: None,
            round_epoch: None,
            attack_guard: MonsterAttackGuard::new(0),
            monsters: Vec::with_capacity(limits.monsters_per_wave),
            waves_started: 0,
            defeated: 0,
            damage: 0,
            last_result: None,
        })
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn host_id(&self) -> Option<u32> {
        self.host_id
    }

    pub fn monster_count(&self) -> usize {
        self.monsters.len()
    }

    pub fn monster_hp(&self, monster_id: u32) -> Option<u32> {
        self.monsters
            .iter()
            .find(|m| m.id == monster_id)
            .map(|m| m.hp)
    }

    pub fn last_result(&self) -> Option<StageResult> {
        self.last_result
    }

    pub fn limits(&self) -> Limits {
        self.limits
    }

    fn gate(&self) -> Result<(), RuntimeError> {
        if !self.policy.enabled {
            return Err(RuntimeError::DispatchDisabled);
        }
        let _authority = self.policy.authority(self.family);
        // Keep the policy check explicit.  Its reviewed table is currently
        // empty, so the external evidence bit is also required to opt in.
        if !self.evidence.is_present() {
            return Err(RuntimeError::WineDispatchEvidenceRequired);
        }
        Ok(())
    }

    pub fn begin_briefing(
        &mut self,
        stage: u16,
        host_id: u32,
        round_epoch: u64,
    ) -> Result<(), RuntimeError> {
        self.gate()?;
        if self.phase != Phase::Idle {
            return Err(RuntimeError::InvalidTransition);
        }
        if stage == 0 || stage > self.limits.stages {
            return Err(RuntimeError::InvalidStage);
        }
        if round_epoch == 0 {
            return Err(RuntimeError::InvalidRoundEpoch);
        }
        self.attack_guard
            .begin_round(round_epoch)
            .map_err(|error| match error {
                MonsterAttackRejection::NonIncreasingRoundEpoch => RuntimeError::InvalidRoundEpoch,
                _ => RuntimeError::WrongRoundEpoch,
            })?;
        self.phase = Phase::Briefing { stage };
        self.host_id = Some(host_id);
        self.round_epoch = Some(round_epoch);
        self.monsters.clear();
        self.waves_started = 0;
        self.defeated = 0;
        self.damage = 0;
        self.last_result = None;
        Ok(())
    }

    pub fn start_wave(&mut self, wave: u16, spawns: &[MonsterSpawn]) -> Result<(), RuntimeError> {
        self.gate()?;
        let stage = match self.phase {
            Phase::Briefing { stage } | Phase::Active { stage, .. } => stage,
            _ => return Err(RuntimeError::InvalidTransition),
        };
        if wave == 0 || wave > self.limits.waves {
            return Err(RuntimeError::InvalidWave);
        }
        if let Phase::Active { .. } = self.phase
            && (wave <= self.waves_started || self.monsters.iter().any(|monster| monster.hp != 0))
        {
            return Err(RuntimeError::InvalidWave);
        }
        if spawns.is_empty() {
            return Err(RuntimeError::EmptyWave);
        }
        if spawns.len() > self.limits.monsters_per_wave {
            return Err(RuntimeError::CapacityExceeded);
        }
        let mut checked = Vec::with_capacity(spawns.len());
        for spawn in spawns {
            if spawn.id == 0 || spawn.hp == 0 || spawn.hp > MAX_MONSTER_HP {
                return Err(RuntimeError::InvalidMonster);
            }
            if checked
                .iter()
                .any(|monster: &Monster| monster.id == spawn.id)
            {
                return Err(RuntimeError::DuplicateMonster);
            }
            checked.push(Monster {
                id: spawn.id,
                hp: spawn.hp,
                max_hp: spawn.hp,
            });
        }
        self.monsters = checked;
        self.waves_started = wave;
        self.phase = Phase::Active { stage, wave };
        Ok(())
    }

    pub fn monster_attack(&mut self, attack: MonsterAttack) -> Result<(), RuntimeError> {
        self.gate()?;
        if !matches!(self.phase, Phase::Active { .. }) {
            return Err(RuntimeError::InvalidTransition);
        }
        let host_id = self.host_id.ok_or(RuntimeError::NotHost)?;
        if attack.actor_id != host_id {
            return Err(RuntimeError::NotHost);
        }
        if Some(attack.round_epoch) != self.round_epoch {
            return Err(RuntimeError::WrongRoundEpoch);
        }
        let index = self
            .monsters
            .iter()
            .position(|monster| monster.id == attack.monster_id)
            .ok_or(RuntimeError::UnknownMonster)?;
        let monster = self.monsters[index];
        if attack.damage == 0 || attack.damage > monster.hp {
            return Err(RuntimeError::InvalidDamage);
        }
        self.attack_guard
            .accept(
                attack.actor_id,
                host_id,
                attack.round_epoch,
                attack.attack_sequence,
            )
            .map_err(|error| match error {
                MonsterAttackRejection::NotHost => RuntimeError::NotHost,
                MonsterAttackRejection::WrongRoundEpoch => RuntimeError::WrongRoundEpoch,
                MonsterAttackRejection::NonIncreasingAttackSequence => {
                    RuntimeError::NonIncreasingSequence
                }
                MonsterAttackRejection::NonIncreasingRoundEpoch => RuntimeError::InvalidRoundEpoch,
            })?;
        self.monsters[index].hp -= attack.damage;
        self.damage += u64::from(attack.damage);
        if self.monsters[index].hp == 0 {
            self.defeated += 1;
        }
        Ok(())
    }

    pub fn monster_attack_candidate(
        &mut self,
        actor_id: u32,
        round_epoch: u64,
        candidate: MonsterAttackReqCandidate,
        monster_id: u32,
        damage: u32,
    ) -> Result<(), RuntimeError> {
        if !candidate.host || !(34..=37).contains(&candidate.state) {
            return Err(if candidate.host {
                RuntimeError::InvalidAttackState
            } else {
                RuntimeError::NotHost
            });
        }
        self.monster_attack(MonsterAttack {
            actor_id,
            round_epoch,
            attack_sequence: u64::from(candidate.attack_seq),
            monster_id,
            damage,
        })
    }

    pub fn clear_stage(&mut self) -> Result<StageResult, RuntimeError> {
        self.gate()?;
        let stage = match self.phase {
            Phase::Active { stage, .. } if self.monsters.iter().all(|monster| monster.hp == 0) => {
                stage
            }
            _ => return Err(RuntimeError::InvalidTransition),
        };
        let result = StageResult {
            stage,
            waves: self.waves_started,
            defeated: self.defeated,
            damage: self.damage,
        };
        self.phase = Phase::Cleared { stage };
        self.last_result = Some(result);
        Ok(result)
    }

    pub fn fail_stage(&mut self) -> Result<(), RuntimeError> {
        self.gate()?;
        let stage = match self.phase {
            Phase::Briefing { stage } | Phase::Active { stage, .. } => stage,
            _ => return Err(RuntimeError::InvalidTransition),
        };
        self.phase = Phase::Failed { stage };
        self.last_result = Some(StageResult {
            stage,
            waves: self.waves_started,
            defeated: self.defeated,
            damage: self.damage,
        });
        Ok(())
    }

    pub fn abort(&mut self) -> Result<(), RuntimeError> {
        self.gate()?;
        let stage = match self.phase {
            Phase::Idle => None,
            Phase::Briefing { stage }
            | Phase::Active { stage, .. }
            | Phase::Cleared { stage }
            | Phase::Failed { stage } => Some(stage),
            Phase::Aborted { .. } => return Err(RuntimeError::InvalidTransition),
        };
        self.phase = Phase::Aborted { stage };
        self.monsters.clear();
        Ok(())
    }
}
