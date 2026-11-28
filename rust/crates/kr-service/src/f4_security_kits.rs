//! In-memory F4 kit and second-password state for trusted server callers.
//! Account, inventory, room, clock, and verifier inputs are server authority.
//! This module does not handle KR packets or persist credentials or state.

use crate::{
    account_serving::F4ExtendedPolicyBundle,
    f4_extended_policy::{
        EquippedKits, ItemOrigin, KitRoomRules, KitStats, MAX_EQUIPPED_KITS, OwnedItem,
        PolicyError, PowerupKitPolicy, PrimarySession, SecondPasswordAction, SecondPasswordOutcome,
        SecondPasswordPolicy, SecondPasswordState,
    },
};
use serde::Serialize;
use std::{
    collections::HashMap,
    sync::{Mutex, MutexGuard},
};

const DEFAULT_MAX_PLAYERS: usize = 1_024;
pub const MAX_UNLOCKED_SESSIONS_PER_PLAYER: usize = 8;

type Result<T> = std::result::Result<T, PolicyError>;

fn require_enabled(enabled: bool) -> Result<()> {
    if enabled {
        Ok(())
    } else {
        Err(PolicyError::Disabled)
    }
}

fn lock<T>(state: &Mutex<T>) -> Result<MutexGuard<'_, T>> {
    state.lock().map_err(|_| PolicyError::NotAllowed)
}

fn check_capacity<T>(players: &HashMap<u64, T>, account: u64, max_players: usize) -> Result<()> {
    if players.len() >= max_players && !players.contains_key(&account) {
        return Err(PolicyError::CapacityExceeded);
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct KitSlot {
    definition: usize,
    quantity: u32,
    expires_at: Option<u64>,
}

type PowerupKitLoadout = [Option<KitSlot>; MAX_EQUIPPED_KITS];

pub struct PowerupKitRuntime {
    policy: PowerupKitPolicy,
    max_players: usize,
    players: Mutex<HashMap<u64, PowerupKitLoadout>>,
}

impl PowerupKitRuntime {
    pub fn new(policy: PowerupKitPolicy, max_players: usize) -> Result<Self> {
        policy.validate()?;
        Ok(Self {
            policy,
            max_players,
            players: Mutex::default(),
        })
    }

    fn check_slot(&self, account: u64, slot: usize) -> Result<()> {
        require_enabled(self.policy.enabled)?;
        if account == 0 || slot >= self.policy.max_equipped_slots {
            return Err(PolicyError::InvalidInput);
        }
        Ok(())
    }

    fn policy_slots(&self, account: u64, slots: PowerupKitLoadout) -> EquippedKits<'_> {
        slots.map(|slot| {
            slot.map(|kit| OwnedItem {
                owner: account,
                sku: &self.policy.kits[kit.definition].sku,
                quantity: kit.quantity,
                expires_at: kit.expires_at,
                origin: ItemOrigin::PersistentInventory,
            })
        })
    }

    /// Equips into a zero-based slot, replacing that slot only on success.
    /// The caller must clear selections when their inventory ownership changes.
    pub fn equip(&self, account: u64, now: u64, slot: usize, kit: OwnedItem<'_>) -> Result<()> {
        self.check_slot(account, slot)?;
        let mut players = lock(&self.players)?;
        check_capacity(&players, account, self.max_players)?;
        let mut next = players.get(&account).copied().unwrap_or_default();
        next[slot] = None;
        // The policy checks an addition against the other slots. The runtime
        // places the validated item in the explicitly requested slot.
        self.policy
            .equip(account, now, &self.policy_slots(account, next), kit)?;
        let definition = self
            .policy
            .kits
            .iter()
            .position(|row| row.sku == kit.sku)
            .ok_or(PolicyError::NotAllowed)?;
        next[slot] = Some(KitSlot {
            definition,
            quantity: kit.quantity,
            expires_at: kit.expires_at,
        });
        players.insert(account, next);
        Ok(())
    }

    /// Clears a slot even if its item has expired. Empty loadouts free capacity.
    pub fn clear(&self, account: u64, slot: usize) -> Result<()> {
        self.check_slot(account, slot)?;
        let mut players = lock(&self.players)?;
        if let Some(slots) = players.get_mut(&account) {
            slots[slot] = None;
            if slots.iter().all(Option::is_none) {
                players.remove(&account);
            }
        }
        Ok(())
    }

    pub fn starting_stats(
        &self,
        account: u64,
        now: u64,
        base: KitStats,
        room: KitRoomRules,
    ) -> Result<KitStats> {
        require_enabled(self.policy.enabled)?;
        if account == 0 {
            return Err(PolicyError::InvalidInput);
        }
        let slots = lock(&self.players)?
            .get(&account)
            .copied()
            .unwrap_or_default();
        self.policy
            .starting_stats(account, now, &self.policy_slots(account, slots), base, room)
    }
}

struct PasswordEntry {
    state: SecondPasswordState,
    unlocked_sessions: usize,
}

pub struct SecondPasswordRuntime {
    policy: SecondPasswordPolicy,
    max_players: usize,
    players: Mutex<HashMap<u64, PasswordEntry>>,
}

impl SecondPasswordRuntime {
    pub fn new(policy: SecondPasswordPolicy, max_players: usize) -> Result<Self> {
        policy.validate()?;
        Ok(Self {
            policy,
            max_players,
            players: Mutex::default(),
        })
    }

    /// Record successful credential setup. Setup does not unlock the session.
    pub fn setup(&self, session: PrimarySession) -> Result<()> {
        require_enabled(self.policy.enabled)?;
        let mut players = lock(&self.players)?;
        check_capacity(&players, session.account, self.max_players)?;
        if players.contains_key(&session.account) {
            return Err(PolicyError::NotAllowed);
        }
        let state = SecondPasswordState::new(session.account, false)?;
        let state = self.policy.setup(&state, Some(session))?;
        players.insert(
            session.account,
            PasswordEntry {
                state,
                unlocked_sessions: 0,
            },
        );
        Ok(())
    }

    /// Preflight before credential verification. `attempt` checks again under
    /// the same mutex before applying any transition.
    pub fn may_attempt(&self, session: PrimarySession, now: u64) -> Result<bool> {
        require_enabled(self.policy.enabled)?;
        let players = lock(&self.players)?;
        let entry = players
            .get(&session.account)
            .ok_or(PolicyError::NotAllowed)?;
        self.policy.may_attempt(&entry.state, Some(session), now)
    }

    /// Apply a trusted verifier result. The credential owner must serialize
    /// verification and credential changes with these calls for each account.
    pub fn attempt(
        &self,
        session: PrimarySession,
        now: u64,
        action: SecondPasswordAction,
        verified: bool,
    ) -> Result<SecondPasswordOutcome> {
        use SecondPasswordOutcome as Outcome;

        require_enabled(self.policy.enabled)?;
        let mut players = lock(&self.players)?;
        let entry = players
            .get_mut(&session.account)
            .ok_or(PolicyError::NotAllowed)?;
        if !self.policy.may_attempt(&entry.state, Some(session), now)? {
            return Ok(Outcome::Locked);
        }
        let was_unlocked = self.policy.is_unlocked(&entry.state, Some(session))?;
        if verified
            && action == SecondPasswordAction::Unlock
            && !was_unlocked
            && entry.unlocked_sessions >= MAX_UNLOCKED_SESSIONS_PER_PLAYER
        {
            return Err(PolicyError::CapacityExceeded);
        }
        let result = self
            .policy
            .attempt(&entry.state, Some(session), now, action, verified)?;
        match result.outcome {
            Outcome::Unlocked => entry.unlocked_sessions += usize::from(!was_unlocked),
            Outcome::Incorrect | Outcome::Locked => {
                entry.unlocked_sessions -= usize::from(was_unlocked);
            }
            Outcome::Changed | Outcome::Disabled => entry.unlocked_sessions = 0,
        }
        if result.outcome == Outcome::Disabled {
            players.remove(&session.account);
        } else {
            entry.state = result.state;
        }
        Ok(result.outcome)
    }

    /// A protected action should proceed only on `Ok(true)` for its session.
    pub fn is_unlocked(&self, session: PrimarySession) -> Result<bool> {
        if !self.policy.enabled {
            return Ok(false);
        }
        let players = lock(&self.players)?;
        match players.get(&session.account) {
            Some(entry) => self.policy.is_unlocked(&entry.state, Some(session)),
            None => Ok(false),
        }
    }

    /// Call on disconnect or logout after stopping requests from this session.
    /// Keeps account-wide lockout state. Future sessions must use unique IDs.
    pub fn revoke_session(&self, session: PrimarySession) -> Result<()> {
        require_enabled(self.policy.enabled)?;
        if session.account == 0 || session.session == 0 {
            return Err(PolicyError::NotAllowed);
        }
        let mut players = lock(&self.players)?;
        if let Some(entry) = players.get_mut(&session.account)
            && self.policy.is_unlocked(&entry.state, Some(session))?
        {
            entry.state = entry.state.revoke_session(session)?;
            entry.unlocked_sessions -= 1;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct F4RuntimeStatus {
    pub enabled: bool,
    pub tracked_players: usize,
    pub max_players: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct F4SecurityKitsStatus {
    pub powerup_kits: F4RuntimeStatus,
    pub second_password: F4RuntimeStatus,
    pub max_unlocked_sessions_per_player: usize,
}

pub struct F4SecurityKitsRuntime {
    pub powerup_kits: PowerupKitRuntime,
    pub second_password: SecondPasswordRuntime,
}

impl Default for F4SecurityKitsRuntime {
    fn default() -> Self {
        Self::from_bundle(&F4ExtendedPolicyBundle::default())
            .expect("default F4 security policies are valid")
    }
}

impl F4SecurityKitsRuntime {
    pub fn from_bundle(bundle: &F4ExtendedPolicyBundle) -> Result<Self> {
        Self::with_capacity(bundle, DEFAULT_MAX_PLAYERS)
    }

    /// Construct with an explicit per-runtime player bound for deployments
    /// that choose a smaller admission budget.
    pub fn with_capacity(bundle: &F4ExtendedPolicyBundle, max_players: usize) -> Result<Self> {
        Ok(Self {
            powerup_kits: PowerupKitRuntime::new(bundle.powerup_kits.clone(), max_players)?,
            second_password: SecondPasswordRuntime::new(
                bundle.second_password.clone(),
                max_players,
            )?,
        })
    }

    /// Returns only aggregate state and configuration bounds.
    pub fn snapshot(&self) -> Result<F4SecurityKitsStatus> {
        let kits = lock(&self.powerup_kits.players)?;
        let passwords = lock(&self.second_password.players)?;
        Ok(F4SecurityKitsStatus {
            powerup_kits: F4RuntimeStatus {
                enabled: self.powerup_kits.policy.enabled,
                tracked_players: kits.len(),
                max_players: self.powerup_kits.max_players,
            },
            second_password: F4RuntimeStatus {
                enabled: self.second_password.policy.enabled,
                tracked_players: passwords.len(),
                max_players: self.second_password.max_players,
            },
            max_unlocked_sessions_per_player: MAX_UNLOCKED_SESSIONS_PER_PLAYER,
        })
    }
}
