//! Opt-in server design from the F4-e spec and F4 progression catalog.
//! These evaluators do not implement KR wire, persistence, or original server
//! formulas. Inputs about ownership, authorization, rooms, time and receipts
//! come from server state. Evaluate and commit against the same state: a new
//! decision and its receipt/claim must be written in one transaction.

use crate::accounts::{ECONOMY_AMOUNT_MAX, LEVEL_MAX, RewardAmount};
use serde::de::{MapAccess, Visitor, value::MapAccessDeserializer};
use serde::{Deserialize, Deserializer};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::marker::PhantomData;

pub const MAX_EQUIPPED_KITS: usize = 2;
pub const BASE_REWARD_PERCENT: u16 = 100;
/// Server design bound, not a KR rate.
pub const MAX_REWARD_PERCENT: u16 = 1_000;
pub const BASIS_POINTS: u16 = 10_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PolicyError {
    InvalidConfig(&'static str),
    Disabled,
    InvalidInput,
    NotAllowed,
    NotOwnedOrExpired,
    CapacityExceeded,
    AlreadyEquipped,
    AlreadyRedeemed,
    ReceiptConflict,
    Overflow,
}

impl fmt::Display for PolicyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidConfig(field) => write!(f, "invalid F4 policy configuration: {field}"),
            other => write!(f, "{other:?}"),
        }
    }
}

impl std::error::Error for PolicyError {}

type Result<T> = std::result::Result<T, PolicyError>;

// Derived Serde structs also accept positional arrays. Use a map visitor for
// each policy and nested object, retaining Serde's unknown/duplicate key checks.
struct Object<T>(T);

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Object<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        struct MapOnly<T>(PhantomData<T>);

        impl<'de, T: Deserialize<'de>> Visitor<'de> for MapOnly<T> {
            type Value = Object<T>;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a configuration object")
            }

            fn visit_map<M: MapAccess<'de>>(
                self,
                map: M,
            ) -> std::result::Result<Self::Value, M::Error> {
                T::deserialize(MapAccessDeserializer::new(map)).map(Object)
            }
        }

        deserializer.deserialize_map(MapOnly(PhantomData))
    }
}

macro_rules! config_object {
    ($kind:ident $name:ident { $($(#[$attr:meta])* $field:ident: $ty:ty),* $(,)? }) => {
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub struct $name {
            $($(#[$attr])* pub $field: $ty,)*
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
                config_object!(@fields $kind $name { $($field: $ty),* });
                let Object(fields) = Object::<Fields>::deserialize(deserializer)?;
                let value = Self { $($field: fields.$field),* };
                config_object!(@validate $kind value);
                Ok(value)
            }
        }
    };
    (@fields policy $name:ident { $($field:ident: $ty:ty),* }) => {
        #[derive(Deserialize)]
        #[serde(default, deny_unknown_fields)]
        struct Fields { $($field: $ty,)* }

        impl Default for Fields {
            fn default() -> Self {
                let defaults = $name::default();
                Self { $($field: defaults.$field),* }
            }
        }
    };
    (@fields row $name:ident { $($field:ident: $ty:ty),* }) => {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Fields { $($field: $ty,)* }
    };
    (@validate policy $value:ident) => {
        $value.validate().map_err(serde::de::Error::custom)?;
    };
    (@validate row $value:ident) => {};
}

fn require(condition: bool, error: PolicyError) -> Result<()> {
    if condition { Ok(()) } else { Err(error) }
}

fn configured(condition: bool, field: &'static str) -> Result<()> {
    require(condition, PolicyError::InvalidConfig(field))
}

fn enabled(value: bool) -> Result<()> {
    require(value, PolicyError::Disabled)
}

fn label(value: &str) -> bool {
    !value.is_empty() && value.trim() == value && !value.chars().any(char::is_control)
}

fn unique<T: Ord>(values: impl IntoIterator<Item = T>) -> bool {
    let mut seen = BTreeSet::new();
    values.into_iter().all(|value| seen.insert(value))
}

fn labels(values: &[String]) -> bool {
    values.iter().all(|value| label(value)) && unique(values)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemOrigin {
    PersistentInventory,
    SessionEntitlement,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OwnedItem<'a> {
    pub owner: u64,
    pub sku: &'a str,
    pub quantity: u32,
    pub expires_at: Option<u64>,
    pub origin: ItemOrigin,
}

impl OwnedItem<'_> {
    fn active_for(self, account: u64, now: u64) -> bool {
        account != 0
            && self.owner == account
            && self.origin == ItemOrigin::PersistentInventory
            && self.quantity > 0
            && self.expires_at.is_none_or(|end| now < end)
    }
}

/// A replay carries the previous result for reporting; only Apply authorizes
/// a new write. The caller supplies committed receipts keyed by operation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Decision<T> {
    Apply(T),
    Replay(T),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct OperationKey {
    pub account: u64,
    pub id: u64,
}

impl OperationKey {
    fn validate(self) -> Result<()> {
        require(self.account != 0 && self.id != 0, PolicyError::InvalidInput)
    }
}

config_object!(row KitStats {
    bombs: u8,
    blast: u8,
    speed: u8,
});

impl Copy for KitStats {}

impl KitStats {
    fn min(self, other: Self) -> Self {
        Self {
            bombs: self.bombs.min(other.bombs),
            blast: self.blast.min(other.blast),
            speed: self.speed.min(other.speed),
        }
    }

    fn add_capped(self, bonus: Self, caps: Self) -> Self {
        Self {
            bombs: self.bombs.saturating_add(bonus.bombs),
            blast: self.blast.saturating_add(bonus.blast),
            speed: self.speed.saturating_add(bonus.speed),
        }
        .min(caps)
    }
}

/// Authoritative inputs for the selected character, mode and map. The room
/// owner supplies these from its rules; kit configuration cannot set them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KitRoomRules {
    pub allows_kits: bool,
    pub character_caps: KitStats,
    pub mode_caps: KitStats,
    pub map_caps: KitStats,
}

pub type EquippedKits<'a> = [Option<OwnedItem<'a>>; MAX_EQUIPPED_KITS];

config_object!(row KitDefinition {
    sku: String,
    bonus: KitStats,
});

config_object!(policy PowerupKitPolicy {
    enabled: bool,
    max_equipped_slots: usize,
    kits: Vec<KitDefinition>,
});

impl Default for PowerupKitPolicy {
    fn default() -> Self {
        Self {
            enabled: false,
            max_equipped_slots: MAX_EQUIPPED_KITS,
            kits: Vec::new(),
        }
    }
}

impl PowerupKitPolicy {
    pub fn validate(&self) -> Result<()> {
        configured(
            self.max_equipped_slots <= MAX_EQUIPPED_KITS
                && (!self.enabled || self.max_equipped_slots > 0),
            "kit slot limit",
        )?;
        configured(
            (!self.enabled || !self.kits.is_empty())
                && self.kits.iter().all(|kit| {
                    label(&kit.sku) && (kit.bonus.bombs | kit.bonus.blast | kit.bonus.speed) != 0
                })
                && unique(self.kits.iter().map(|kit| &kit.sku)),
            "kit definitions",
        )
    }

    fn definition(&self, sku: &str) -> Option<&KitDefinition> {
        self.kits.iter().find(|kit| kit.sku == sku)
    }

    /// Return the proposed slots. The caller keeps the previous slots on
    /// rejection; a room ban is checked when applying kits at match start.
    pub fn equip<'a>(
        &self,
        account: u64,
        now: u64,
        slots: &EquippedKits<'a>,
        kit: OwnedItem<'a>,
    ) -> Result<EquippedKits<'a>> {
        self.validate()?;
        enabled(self.enabled)?;
        require(kit.active_for(account, now), PolicyError::NotOwnedOrExpired)?;
        require(self.definition(kit.sku).is_some(), PolicyError::NotAllowed)?;
        let active =
            |slot: &Option<OwnedItem<'_>>| slot.is_some_and(|item| item.active_for(account, now));
        require(
            !slots
                .iter()
                .filter(|slot| active(slot))
                .flatten()
                .any(|item| item.sku == kit.sku),
            PolicyError::AlreadyEquipped,
        )?;
        require(
            slots.iter().filter(|slot| active(slot)).count() < self.max_equipped_slots,
            PolicyError::CapacityExceeded,
        )?;
        let index = slots
            .iter()
            .position(|slot| !active(slot))
            .ok_or(PolicyError::CapacityExceeded)?;
        let mut next = *slots;
        next[index] = Some(kit);
        Ok(next)
    }

    /// Snapshot start stats with currently valid kits. Existing invalid/expired
    /// gear has no effect. A room ban or a disabled policy uses the base stats.
    pub fn starting_stats(
        &self,
        account: u64,
        now: u64,
        slots: &EquippedKits<'_>,
        base: KitStats,
        room: KitRoomRules,
    ) -> Result<KitStats> {
        self.validate()?;
        let caps = room.character_caps.min(room.mode_caps).min(room.map_caps);
        require(base == base.min(caps), PolicyError::InvalidInput)?;
        if !self.enabled || !room.allows_kits {
            return Ok(base);
        }

        let active = || {
            slots
                .iter()
                .flatten()
                .filter(|item| item.active_for(account, now))
        };
        require(
            active().count() <= self.max_equipped_slots,
            PolicyError::CapacityExceeded,
        )?;
        require(
            unique(active().map(|item| item.sku)),
            PolicyError::InvalidInput,
        )?;
        Ok(active()
            .filter_map(|item| self.definition(item.sku))
            .fold(base, |stats, kit| stats.add_capped(kit.bonus, caps)))
    }
}

config_object!(row BoosterRoom {
    room_group: String,
    mode: String,
    maps: Vec<String>,
});

config_object!(row BoosterBonusCap {
    exp: u64,
    lucci: u64,
});

config_object!(policy BoosterTicketPolicy {
    enabled: bool,
    exp_percent: u16,
    lucci_percent: u16,
    bonus_cap: Option<BoosterBonusCap>,
    allowed_rooms: Vec<BoosterRoom>,
});

impl Default for BoosterTicketPolicy {
    fn default() -> Self {
        Self {
            enabled: false,
            exp_percent: BASE_REWARD_PERCENT,
            lucci_percent: BASE_REWARD_PERCENT,
            bonus_cap: None,
            allowed_rooms: Vec::new(),
        }
    }
}

/// RoomRules admission includes the existing map pool, roster, events and
/// other room restrictions. This policy can only narrow that decision.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BoosterRoomContext<'a> {
    pub room_group: &'a str,
    pub mode: &'a str,
    pub map: &'a str,
    pub room_rules_allow: bool,
    pub normal_unranked: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BoosterTicket {
    pub account: u64,
    pub expires_at: u64,
    /// Granted free by the server under its eligibility rules.
    pub free_server_grant: bool,
}

impl BoosterTicket {
    fn active_for(self, account: u64, now: u64) -> bool {
        account != 0 && self.account == account && self.free_server_grant && now < self.expires_at
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SettlementKey {
    pub match_id: u64,
    pub account: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BoosterSettlement {
    pub bonus: RewardAmount,
    pub total: RewardAmount,
}

impl BoosterTicketPolicy {
    pub fn validate(&self) -> Result<()> {
        let percentages = [self.exp_percent, self.lucci_percent];
        configured(
            percentages
                .iter()
                .all(|pct| (BASE_REWARD_PERCENT..=MAX_REWARD_PERCENT).contains(pct)),
            "booster percentages",
        )?;
        let cap = self.bonus_cap.as_ref();
        configured(
            cap.is_none_or(|cap| cap.exp <= ECONOMY_AMOUNT_MAX && cap.lucci <= ECONOMY_AMOUNT_MAX)
                && (self.exp_percent == BASE_REWARD_PERCENT || cap.is_some_and(|cap| cap.exp > 0))
                && (self.lucci_percent == BASE_REWARD_PERCENT
                    || cap.is_some_and(|cap| cap.lucci > 0)),
            "booster bonus cap",
        )?;
        configured(
            (!self.enabled || !self.allowed_rooms.is_empty())
                && unique(
                    self.allowed_rooms
                        .iter()
                        .map(|row| (&row.room_group, &row.mode)),
                )
                && self.allowed_rooms.iter().all(|row| {
                    label(&row.room_group)
                        && label(&row.mode)
                        && !row.maps.is_empty()
                        && labels(&row.maps)
                }),
            "booster room/map allowlist",
        )
    }

    fn compatible(&self, room: BoosterRoomContext<'_>) -> bool {
        self.enabled
            && room.room_rules_allow
            && room.normal_unranked
            && self.allowed_rooms.iter().any(|row| {
                row.room_group == room.room_group
                    && row.mode == room.mode
                    && row.maps.iter().any(|map| map == room.map)
            })
    }

    /// Additional ticket eligibility, not general room/map admission.
    pub fn allows_ticket_map(
        &self,
        account: u64,
        now: u64,
        ticket: BoosterTicket,
        room: BoosterRoomContext<'_>,
    ) -> Result<bool> {
        self.validate()?;
        Ok(ticket.active_for(account, now) && self.compatible(room))
    }

    /// Settle from the authoritative base reward and match ticket snapshot.
    /// Disabled or ineligible tickets leave base rewards unchanged. Commit a
    /// receipt even for a zero bonus, alongside the match settlement.
    pub fn settle(
        &self,
        key: SettlementKey,
        now: u64,
        ticket: Option<BoosterTicket>,
        room: BoosterRoomContext<'_>,
        base: RewardAmount,
        receipts: &BTreeMap<SettlementKey, BoosterSettlement>,
    ) -> Result<Decision<BoosterSettlement>> {
        require(
            key.account != 0 && key.match_id != 0,
            PolicyError::InvalidInput,
        )?;
        if let Some(previous) = receipts.get(&key) {
            return Ok(Decision::Replay(*previous));
        }
        self.validate()?;
        require(
            base.exp <= ECONOMY_AMOUNT_MAX && base.lucci <= ECONOMY_AMOUNT_MAX,
            PolicyError::InvalidInput,
        )?;
        let mut bonus = RewardAmount::default();
        if ticket.is_some_and(|t| t.active_for(key.account, now))
            && self.compatible(room)
            && let Some(cap) = &self.bonus_cap
        {
            bonus.exp = reward_bonus(base.exp, self.exp_percent, cap.exp);
            bonus.lucci = reward_bonus(base.lucci, self.lucci_percent, cap.lucci);
        }
        Ok(Decision::Apply(BoosterSettlement {
            bonus,
            total: RewardAmount {
                exp: base.exp + bonus.exp,
                lucci: base.lucci + bonus.lucci,
            },
        }))
    }
}

fn reward_bonus(base: u64, percent: u16, bonus_cap: u64) -> u64 {
    // Round the bonus down to whole units. The per-channel cap limits the
    // bonus, and the existing economy amount bound limits base + bonus.
    let extra = u64::from(percent - BASE_REWARD_PERCENT);
    (base * extra / u64::from(BASE_REWARD_PERCENT))
        .min(bonus_cap)
        .min(ECONOMY_AMOUNT_MAX - base)
}

config_object!(row TitleUnlock {
    title: String,
    condition: String,
});

config_object!(policy TitlePolicy {
    enabled: bool,
    equip_slot: Option<String>,
    notify_on_unlock: bool,
    unlock_table: Option<Vec<TitleUnlock>>,
});

#[expect(
    clippy::derivable_impls,
    reason = "keep the disabled policy default explicit"
)]
impl Default for TitlePolicy {
    fn default() -> Self {
        Self {
            enabled: false,
            equip_slot: None,
            notify_on_unlock: false,
            unlock_table: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TitleUnlockDecision {
    /// Persist the unlock before emitting the notification.
    New {
        notify_after_commit: bool,
    },
    AlreadyUnlocked,
}

impl TitlePolicy {
    pub fn validate(&self) -> Result<()> {
        configured(
            self.equip_slot.as_deref().is_none_or(label)
                && (!self.enabled || self.equip_slot.is_some()),
            "title equip slot",
        )?;
        configured(
            self.unlock_table.as_ref().is_none_or(|rows| {
                rows.iter()
                    .all(|row| label(&row.title) && label(&row.condition))
                    && unique(rows.iter().map(|row| &row.title))
            }) && (!self.enabled
                || self
                    .unlock_table
                    .as_ref()
                    .is_some_and(|rows| !rows.is_empty())),
            "title unlock table",
        )
    }

    fn title(&self, title: &str) -> Result<&TitleUnlock> {
        self.validate()?;
        enabled(self.enabled)?;
        self.unlock_table
            .iter()
            .flatten()
            .find(|row| row.title == title)
            .ok_or(PolicyError::NotAllowed)
    }

    /// The unlock ledger and satisfied conditions are for the same account.
    pub fn unlock(
        &self,
        title: &str,
        unlocked: &BTreeSet<String>,
        satisfied_conditions: &BTreeSet<String>,
    ) -> Result<TitleUnlockDecision> {
        let row = self.title(title)?;
        if unlocked.contains(title) {
            return Ok(TitleUnlockDecision::AlreadyUnlocked);
        }
        require(
            satisfied_conditions.contains(&row.condition),
            PolicyError::NotAllowed,
        )?;
        Ok(TitleUnlockDecision::New {
            notify_after_commit: self.notify_on_unlock,
        })
    }

    /// Equipping a title never creates an unlock or a notification.
    pub fn equip(&self, title: &str, unlocked: &BTreeSet<String>) -> Result<&str> {
        self.title(title)?;
        require(unlocked.contains(title), PolicyError::NotAllowed)?;
        self.equip_slot
            .as_deref()
            .ok_or(PolicyError::InvalidConfig("title equip slot"))
    }
}

config_object!(row ItemGrant {
    sku: String,
    quantity: u32,
});

impl ItemGrant {
    fn valid(&self) -> bool {
        label(&self.sku) && self.quantity > 0
    }
}

config_object!(row LevelChest {
    character: u16,
    level: u16,
    chest: ItemGrant,
});

config_object!(row LevelLucci {
    character: u16,
    level: u16,
    lucci: u64,
});

config_object!(policy LevelRewardPolicy {
    enabled: bool,
    character_milestones: Option<Vec<LevelChest>>,
    level_up_lucci_table: Option<Vec<LevelLucci>>,
});

#[expect(
    clippy::derivable_impls,
    reason = "keep the disabled policy default explicit"
)]
impl Default for LevelRewardPolicy {
    fn default() -> Self {
        Self {
            enabled: false,
            character_milestones: None,
            level_up_lucci_table: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum MilestoneKind {
    Lucci,
    Chest,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MilestoneClaim {
    pub account: u64,
    pub character: u16,
    pub level: u16,
    pub kind: MilestoneKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LevelReward {
    Lucci(u64),
    Chest(ItemGrant),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LevelGrant {
    pub claim: MilestoneClaim,
    pub reward: LevelReward,
}

fn milestone_level(level: u16) -> bool {
    (2..=LEVEL_MAX).contains(&usize::from(level))
}

impl LevelRewardPolicy {
    pub fn validate(&self) -> Result<()> {
        let chests = self.character_milestones.as_deref().unwrap_or_default();
        let lucci = self.level_up_lucci_table.as_deref().unwrap_or_default();
        configured(
            !self.enabled || !chests.is_empty() || !lucci.is_empty(),
            "level reward tables",
        )?;
        configured(
            chests
                .iter()
                .all(|row| milestone_level(row.level) && row.chest.valid())
                && unique(chests.iter().map(|row| (row.character, row.level))),
            "level chest milestones",
        )?;
        configured(
            lucci.iter().all(|row| {
                milestone_level(row.level) && (1..=ECONOMY_AMOUNT_MAX).contains(&row.lucci)
            }) && unique(lucci.iter().map(|row| (row.character, row.level))),
            "level Lucci milestones",
        )
    }

    /// Enumerate all new grants in (old_level, new_level], in ascending claim
    /// order. Claim each reward kind separately, in the grant transaction.
    pub fn grants_for_level_gain(
        &self,
        account: u64,
        character: u16,
        old_level: u16,
        new_level: u16,
        claimed: &BTreeSet<MilestoneClaim>,
    ) -> Result<Vec<LevelGrant>> {
        self.validate()?;
        enabled(self.enabled)?;
        require(
            account != 0
                && old_level > 0
                && old_level <= new_level
                && usize::from(new_level) <= LEVEL_MAX,
            PolicyError::InvalidInput,
        )?;
        let claim = |level, kind| MilestoneClaim {
            account,
            character,
            level,
            kind,
        };
        let qualifies =
            |candidate, level| candidate == character && old_level < level && level <= new_level;
        let mut grants = Vec::new();
        for row in self.level_up_lucci_table.iter().flatten() {
            let claim = claim(row.level, MilestoneKind::Lucci);
            if qualifies(row.character, row.level) && !claimed.contains(&claim) {
                grants.push(LevelGrant {
                    claim,
                    reward: LevelReward::Lucci(row.lucci),
                });
            }
        }
        for row in self.character_milestones.iter().flatten() {
            let claim = claim(row.level, MilestoneKind::Chest);
            if qualifies(row.character, row.level) && !claimed.contains(&claim) {
                grants.push(LevelGrant {
                    claim,
                    reward: LevelReward::Chest(row.chest.clone()),
                });
            }
        }
        grants.sort_unstable_by_key(|grant| grant.claim);
        Ok(grants)
    }
}

config_object!(policy LucciVaultPolicy {
    enabled: bool,
    base_lucci_cap: Option<i64>,
    expansion_step: Option<i64>,
    max_expansions: Option<u32>,
    max_lucci_cap: Option<i64>,
});

#[expect(
    clippy::derivable_impls,
    reason = "keep the disabled policy default explicit"
)]
impl Default for LucciVaultPolicy {
    fn default() -> Self {
        Self {
            enabled: false,
            base_lucci_cap: None,
            expansion_step: None,
            max_expansions: None,
            max_lucci_cap: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VaultExpansion {
    pub expansions: u32,
    pub capacity: i64,
}

impl LucciVaultPolicy {
    pub fn validate(&self) -> Result<()> {
        match (
            self.base_lucci_cap,
            self.expansion_step,
            self.max_expansions,
            self.max_lucci_cap,
        ) {
            (None, None, None, None) if !self.enabled => Ok(()),
            (Some(base), Some(step), Some(count), Some(max)) => configured(
                base > 0 && step > 0 && vault_cap(base, step, count).is_ok_and(|cap| cap <= max),
                "vault capacity bounds",
            ),
            _ => Err(PolicyError::InvalidConfig("vault capacity fields")),
        }
    }

    pub fn capacity(&self, expansions: u32) -> Result<i64> {
        self.validate()?;
        enabled(self.enabled)?;
        let (Some(base), Some(step), Some(max_expansions)) = (
            self.base_lucci_cap,
            self.expansion_step,
            self.max_expansions,
        ) else {
            return Err(PolicyError::InvalidConfig("vault capacity fields"));
        };
        require(expansions <= max_expansions, PolicyError::CapacityExceeded)?;
        vault_cap(base, step, expansions)
    }

    pub fn deposit(&self, expansions: u32, balance: i64, amount: i64) -> Result<i64> {
        let cap = self.capacity(expansions)?;
        require(balance >= 0 && amount > 0, PolicyError::InvalidInput)?;
        let balance = balance.checked_add(amount).ok_or(PolicyError::Overflow)?;
        require(balance <= cap, PolicyError::CapacityExceeded)?;
        Ok(balance)
    }

    pub fn expand(&self, expansions: u32, balance: i64) -> Result<VaultExpansion> {
        let capacity = self.capacity(expansions)?;
        require(balance >= 0, PolicyError::InvalidInput)?;
        require(balance <= capacity, PolicyError::CapacityExceeded)?;
        let expansions = expansions
            .checked_add(1)
            .ok_or(PolicyError::CapacityExceeded)?;
        Ok(VaultExpansion {
            expansions,
            capacity: self.capacity(expansions)?,
        })
    }
}

fn vault_cap(base: i64, step: i64, expansions: u32) -> Result<i64> {
    step.checked_mul(i64::from(expansions))
        .and_then(|extra| base.checked_add(extra))
        .ok_or(PolicyError::Overflow)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordScope {
    Level,
    WinLoss,
    Ladder,
}

config_object!(row WinLossRecord {
    wins: u32,
    losses: u32,
});

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordValue {
    Level(u16),
    WinLoss(WinLossRecord),
    Ladder(u32),
}

impl RecordValue {
    fn scope(&self) -> RecordScope {
        match self {
            Self::Level(_) => RecordScope::Level,
            Self::WinLoss(_) => RecordScope::WinLoss,
            Self::Ladder(_) => RecordScope::Ladder,
        }
    }

    fn valid(&self) -> bool {
        !matches!(self, Self::Level(level) if !(1..=LEVEL_MAX).contains(&usize::from(*level)))
    }
}

config_object!(row EraserDefinition {
    sku: String,
    reset_to: RecordValue,
});

config_object!(policy RecordEraserPolicy {
    enabled: bool,
    allowed_scopes: Vec<RecordScope>,
    require_audit: bool,
    erasers: Vec<EraserDefinition>,
});

impl Default for RecordEraserPolicy {
    fn default() -> Self {
        Self {
            enabled: false,
            allowed_scopes: Vec::new(),
            require_audit: true,
            erasers: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordEraseRequest<'a> {
    /// The operation is keyed by the target account.
    pub key: OperationKey,
    pub actor: u64,
    pub authorized: bool,
    pub audit_available: bool,
    pub eraser: OwnedItem<'a>,
    pub before: RecordValue,
    pub now: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordReset {
    pub key: OperationKey,
    pub actor: u64,
    /// Consume one of this SKU from the actor's inventory.
    pub consume_sku: String,
    pub before: RecordValue,
    pub after: RecordValue,
}

impl RecordEraserPolicy {
    pub fn validate(&self) -> Result<()> {
        configured(self.require_audit, "record erase audit is mandatory")?;
        configured(
            (!self.enabled || !self.erasers.is_empty())
                && unique(&self.allowed_scopes)
                && unique(
                    self.erasers
                        .iter()
                        .map(|row| (&row.sku, row.reset_to.scope())),
                )
                && self.erasers.iter().all(|row| {
                    label(&row.sku)
                        && row.reset_to.valid()
                        && self.allowed_scopes.contains(&row.reset_to.scope())
                }),
            "record erase scopes and item mapping",
        )
    }

    /// Commit only the scoped reset, consumption and audit receipt, together.
    /// The audit holds actor, target, before/after values and operation ID.
    pub fn erase(
        &self,
        request: RecordEraseRequest<'_>,
        receipts: &BTreeMap<OperationKey, RecordReset>,
    ) -> Result<Decision<RecordReset>> {
        request.key.validate()?;
        require(request.actor != 0, PolicyError::InvalidInput)?;
        require(
            request.authorized && request.audit_available,
            PolicyError::NotAllowed,
        )?;
        let scope = request.before.scope();
        if let Some(previous) = receipts.get(&request.key) {
            require(
                previous.key == request.key
                    && previous.actor == request.actor
                    && previous.consume_sku == request.eraser.sku
                    && previous.after.scope() == scope,
                PolicyError::ReceiptConflict,
            )?;
            return Ok(Decision::Replay(previous.clone()));
        }
        self.validate()?;
        enabled(self.enabled)?;
        require(request.before.valid(), PolicyError::InvalidInput)?;
        require(
            request.eraser.active_for(request.actor, request.now),
            PolicyError::NotOwnedOrExpired,
        )?;
        let row = self
            .erasers
            .iter()
            .find(|row| row.sku == request.eraser.sku && row.reset_to.scope() == scope)
            .ok_or(PolicyError::NotAllowed)?;
        Ok(Decision::Apply(RecordReset {
            key: request.key,
            actor: request.actor,
            consume_sku: row.sku.clone(),
            before: request.before,
            after: row.reset_to.clone(),
        }))
    }
}

config_object!(row GiftLimits {
    max_quantity: u32,
    /// Count committed gift operations by the sender in this window.
    max_per_window: u32,
    window_seconds: u64,
});

config_object!(row CouponOffer {
    offer: String,
    valid_for_seconds: u64,
    grant: ItemGrant,
});

config_object!(policy GiftCouponPolicy {
    enabled: bool,
    giftable_skus: Vec<String>,
    gift_limits: Option<GiftLimits>,
    coupon_offers: Vec<CouponOffer>,
});

#[expect(
    clippy::derivable_impls,
    reason = "keep the disabled policy default explicit"
)]
impl Default for GiftCouponPolicy {
    fn default() -> Self {
        Self {
            enabled: false,
            giftable_skus: Vec::new(),
            gift_limits: None,
            coupon_offers: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GiftRequest<'a> {
    /// Keyed by the sender.
    pub key: OperationKey,
    pub recipient: u64,
    pub item: OwnedItem<'a>,
    pub quantity: u32,
    /// Count from the authoritative gift ledger for the configured window.
    pub gifts_in_window: u32,
    pub now: u64,
}

/// A server-resolved coupon reference. Raw codes, billing and external coupon
/// systems are outside the policy. Each local coupon ID is single use.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CouponRequest<'a> {
    pub key: OperationKey,
    pub coupon_id: u64,
    pub offer: &'a str,
    pub issued_to: u64,
    pub issued_at: u64,
    pub now: u64,
    pub locally_issued: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GiftCouponGrant {
    /// Move an owned stack without extending its expiry.
    Gift {
        recipient: u64,
        item: ItemGrant,
        expires_at: Option<u64>,
    },
    Coupon {
        coupon_id: u64,
        offer: String,
        item: ItemGrant,
    },
}

impl GiftCouponPolicy {
    pub fn validate(&self) -> Result<()> {
        configured(
            (!self.enabled || !self.giftable_skus.is_empty() || !self.coupon_offers.is_empty())
                && labels(&self.giftable_skus)
                && (self.giftable_skus.is_empty() || self.gift_limits.is_some())
                && self.gift_limits.as_ref().is_none_or(|limits| {
                    limits.max_quantity > 0
                        && limits.max_per_window > 0
                        && limits.window_seconds > 0
                }),
            "gift SKU allowlist and limits",
        )?;
        configured(
            unique(self.coupon_offers.iter().map(|row| &row.offer))
                && self
                    .coupon_offers
                    .iter()
                    .all(|row| label(&row.offer) && row.valid_for_seconds > 0 && row.grant.valid()),
            "coupon offers",
        )
    }

    pub fn gift(
        &self,
        request: GiftRequest<'_>,
        receipts: &BTreeMap<OperationKey, GiftCouponGrant>,
    ) -> Result<Decision<GiftCouponGrant>> {
        request.key.validate()?;
        require(
            request.recipient != 0
                && request.recipient != request.key.account
                && request.quantity > 0,
            PolicyError::InvalidInput,
        )?;
        if let Some(previous) = receipts.get(&request.key) {
            require(
                matches!(previous, GiftCouponGrant::Gift { recipient, item, .. }
                    if *recipient == request.recipient && item.sku == request.item.sku && item.quantity == request.quantity),
                PolicyError::ReceiptConflict,
            )?;
            return Ok(Decision::Replay(previous.clone()));
        }
        self.validate()?;
        enabled(self.enabled)?;
        require(
            request.item.active_for(request.key.account, request.now),
            PolicyError::NotOwnedOrExpired,
        )?;
        require(
            self.giftable_skus.iter().any(|sku| sku == request.item.sku),
            PolicyError::NotAllowed,
        )?;
        let limits = self.gift_limits.as_ref().ok_or(PolicyError::NotAllowed)?;
        require(
            request.quantity <= request.item.quantity
                && request.quantity <= limits.max_quantity
                && request.gifts_in_window < limits.max_per_window,
            PolicyError::CapacityExceeded,
        )?;
        Ok(Decision::Apply(GiftCouponGrant::Gift {
            recipient: request.recipient,
            item: ItemGrant {
                sku: request.item.sku.to_owned(),
                quantity: request.quantity,
            },
            expires_at: request.item.expires_at,
        }))
    }

    /// The used-coupon index is global by local coupon ID. Write it along
    /// with the grant and operation receipt so changing operation IDs cannot
    /// redeem the same coupon twice.
    pub fn redeem(
        &self,
        request: CouponRequest<'_>,
        used_coupons: &BTreeSet<u64>,
        receipts: &BTreeMap<OperationKey, GiftCouponGrant>,
    ) -> Result<Decision<GiftCouponGrant>> {
        request.key.validate()?;
        require(
            request.coupon_id != 0 && request.now >= request.issued_at,
            PolicyError::InvalidInput,
        )?;
        require(
            request.locally_issued && request.issued_to == request.key.account,
            PolicyError::NotAllowed,
        )?;
        if let Some(previous) = receipts.get(&request.key) {
            require(
                matches!(previous, GiftCouponGrant::Coupon { coupon_id, offer, .. }
                    if *coupon_id == request.coupon_id && offer == request.offer),
                PolicyError::ReceiptConflict,
            )?;
            return Ok(Decision::Replay(previous.clone()));
        }
        self.validate()?;
        enabled(self.enabled)?;
        require(
            !used_coupons.contains(&request.coupon_id),
            PolicyError::AlreadyRedeemed,
        )?;
        let row = self
            .coupon_offers
            .iter()
            .find(|row| row.offer == request.offer)
            .ok_or(PolicyError::NotAllowed)?;
        require(
            request.now - request.issued_at < row.valid_for_seconds,
            PolicyError::NotOwnedOrExpired,
        )?;
        Ok(Decision::Apply(GiftCouponGrant::Coupon {
            coupon_id: request.coupon_id,
            offer: row.offer.clone(),
            item: row.grant.clone(),
        }))
    }
}

config_object!(policy SecondPasswordPolicy {
    enabled: bool,
    max_failed_attempts: u8,
    failure_window_seconds: u64,
    cooldown_seconds: u64,
});

impl Default for SecondPasswordPolicy {
    fn default() -> Self {
        Self {
            enabled: false,
            max_failed_attempts: 5,
            failure_window_seconds: 900,
            cooldown_seconds: 300,
        }
    }
}

/// An authenticated primary session resolved by the caller. Session IDs
/// must be unique for the lifetime of the authority holding this state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PrimarySession {
    pub account: u64,
    pub session: u64,
}

/// One account's state shared across all primary sessions. Use a consistent
/// clock across requests. No credential or verifier is stored here.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SecondPasswordState {
    account: u64,
    configured: bool,
    failures: u8,
    failure_started: Option<u64>,
    lock_started: Option<u64>,
    last_attempt: Option<u64>,
    unlocked_sessions: BTreeSet<u64>,
}

impl SecondPasswordState {
    pub fn new(account: u64, configured: bool) -> Result<Self> {
        require(account != 0, PolicyError::InvalidInput)?;
        Ok(Self {
            account,
            configured,
            failures: 0,
            failure_started: None,
            lock_started: None,
            last_attempt: None,
            unlocked_sessions: BTreeSet::new(),
        })
    }

    fn primary_session(&self, session: Option<PrimarySession>) -> Result<u64> {
        session
            .filter(|s| s.account == self.account && s.session != 0)
            .map(|s| s.session)
            .ok_or(PolicyError::NotAllowed)
    }

    fn clear_failures(&mut self) {
        self.failures = 0;
        self.failure_started = None;
        self.lock_started = None;
    }

    /// Revoke this session on relock or logout, without clearing account
    /// attempt limits. This remains available if the policy is turned off.
    pub fn revoke_session(&self, session: PrimarySession) -> Result<Self> {
        let id = self.primary_session(Some(session))?;
        let mut next = self.clone();
        next.unlocked_sessions.remove(&id);
        Ok(next)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SecondPasswordAction {
    Unlock,
    Change,
    Disable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SecondPasswordOutcome {
    Unlocked,
    Incorrect,
    Locked,
    Changed,
    Disabled,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SecondPasswordAttempt {
    pub outcome: SecondPasswordOutcome,
    /// Persist failed attempts too. Change/disable must commit with the
    /// corresponding credential update from the external verifier service.
    pub state: SecondPasswordState,
}

impl SecondPasswordPolicy {
    pub fn validate(&self) -> Result<()> {
        configured(
            (1..=5).contains(&self.max_failed_attempts)
                && (60..=900).contains(&self.failure_window_seconds)
                && (60..=900).contains(&self.cooldown_seconds),
            "second password attempt bounds",
        )
    }

    /// Set up with an authenticated primary session. Commit with the new
    /// credential; setup grants no session unlock.
    pub fn setup(
        &self,
        state: &SecondPasswordState,
        session: Option<PrimarySession>,
    ) -> Result<SecondPasswordState> {
        self.validate()?;
        enabled(self.enabled)?;
        state.primary_session(session)?;
        require(!state.configured, PolicyError::NotAllowed)?;
        let mut next = state.clone();
        next.configured = true;
        Ok(next)
    }

    pub fn is_unlocked(
        &self,
        state: &SecondPasswordState,
        session: Option<PrimarySession>,
    ) -> Result<bool> {
        self.validate()?;
        Ok(self.enabled
            && state.configured
            && state
                .primary_session(session)
                .is_ok_and(|id| state.unlocked_sessions.contains(&id)))
    }

    fn check_attempt(
        &self,
        state: &SecondPasswordState,
        session: Option<PrimarySession>,
        now: u64,
    ) -> Result<(u64, bool)> {
        self.validate()?;
        enabled(self.enabled)?;
        let id = state.primary_session(session)?;
        require(state.configured, PolicyError::NotAllowed)?;
        require(
            state.last_attempt.is_none_or(|last| now >= last),
            PolicyError::InvalidInput,
        )?;
        Ok((
            id,
            state
                .lock_started
                .is_some_and(|start| now - start < self.cooldown_seconds),
        ))
    }

    /// Check before verifying a credential; a false result means cooldown.
    pub fn may_attempt(
        &self,
        state: &SecondPasswordState,
        session: Option<PrimarySession>,
        now: u64,
    ) -> Result<bool> {
        self.check_attempt(state, session, now)
            .map(|(_, locked)| !locked)
    }

    /// Takes only the trusted verifier result, never a password. Windows and
    /// locks end at their exact configured duration; rejected attempts during
    /// a lock do not extend it.
    pub fn attempt(
        &self,
        state: &SecondPasswordState,
        session: Option<PrimarySession>,
        now: u64,
        action: SecondPasswordAction,
        verified: bool,
    ) -> Result<SecondPasswordAttempt> {
        let (id, locked) = self.check_attempt(state, session, now)?;
        let mut next = state.clone();
        if locked {
            return Ok(SecondPasswordAttempt {
                outcome: SecondPasswordOutcome::Locked,
                state: next,
            });
        }
        next.last_attempt = Some(now);
        if next.lock_started.is_some()
            || next
                .failure_started
                .is_some_and(|start| now - start >= self.failure_window_seconds)
        {
            next.clear_failures();
        }
        let outcome = if verified {
            next.clear_failures();
            match action {
                SecondPasswordAction::Unlock => {
                    next.unlocked_sessions.insert(id);
                    SecondPasswordOutcome::Unlocked
                }
                SecondPasswordAction::Change => {
                    next.unlocked_sessions.clear();
                    SecondPasswordOutcome::Changed
                }
                SecondPasswordAction::Disable => {
                    next.configured = false;
                    next.unlocked_sessions.clear();
                    SecondPasswordOutcome::Disabled
                }
            }
        } else {
            next.unlocked_sessions.remove(&id);
            next.failure_started.get_or_insert(now);
            next.failures += 1;
            if next.failures >= self.max_failed_attempts {
                next.lock_started = Some(now);
                SecondPasswordOutcome::Locked
            } else {
                SecondPasswordOutcome::Incorrect
            }
        };
        Ok(SecondPasswordAttempt {
            outcome,
            state: next,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpgradeFamily {
    Item,
    Pet,
    WaterBalloon,
    RpgGear,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpgradeFailure {
    Keep,
    Destroy,
}

config_object!(row UpgradeRecipe {
    family: UpgradeFamily,
    sku: String,
    from_level: u16,
    to_level: u16,
    max_level: u16,
    materials: Vec<ItemGrant>,
    lucci_cost: u64,
    success_bps: u16,
    on_failure: UpgradeFailure,
    charge_on_failure: bool,
});

config_object!(policy UpgradePolicy {
    enabled: bool,
    recipes: Vec<UpgradeRecipe>,
});

#[expect(
    clippy::derivable_impls,
    reason = "keep the disabled policy default explicit"
)]
impl Default for UpgradePolicy {
    fn default() -> Self {
        Self {
            enabled: false,
            recipes: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UpgradeRequest<'a> {
    pub key: OperationKey,
    pub instance: u64,
    pub item: OwnedItem<'a>,
    pub family: UpgradeFamily,
    pub level: u16,
    pub now: u64,
    pub normal_persistent_inventory: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UpgradeResources<'a> {
    pub materials: &'a [OwnedItem<'a>],
    pub lucci_balance: i64,
    /// Server-side roll in [0, 10_000); ignored on replay.
    pub roll_bps: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UpgradeChange {
    Level(u16),
    Keep,
    Destroy,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UpgradeResult {
    pub instance: u64,
    pub sku: String,
    pub family: UpgradeFamily,
    pub change: UpgradeChange,
    pub consume_materials: Vec<ItemGrant>,
    pub debit_lucci: u64,
}

impl UpgradePolicy {
    pub fn validate(&self) -> Result<()> {
        configured(
            (!self.enabled || !self.recipes.is_empty())
                && unique(
                    self.recipes
                        .iter()
                        .map(|row| (row.family, &row.sku, row.from_level)),
                ),
            "upgrade recipe keys",
        )?;
        let mut caps = BTreeMap::new();
        configured(
            self.recipes.iter().all(|row| {
                label(&row.sku)
                    && row.from_level < row.to_level
                    && row.to_level <= row.max_level
                    && row.success_bps <= BASIS_POINTS
                    && row.lucci_cost <= ECONOMY_AMOUNT_MAX
                    && row
                        .materials
                        .iter()
                        .all(|mat| mat.valid() && mat.sku != row.sku)
                    && unique(row.materials.iter().map(|mat| &mat.sku))
                    && *caps.entry((row.family, &row.sku)).or_insert(row.max_level) == row.max_level
            }),
            "upgrade recipe bounds",
        )
    }

    /// Return the explicit target change and material/Lucci debits. Commit all
    /// three with the receipt; retries return it without using a new roll.
    pub fn upgrade(
        &self,
        request: UpgradeRequest<'_>,
        resources: UpgradeResources<'_>,
        receipts: &BTreeMap<OperationKey, UpgradeResult>,
    ) -> Result<Decision<UpgradeResult>> {
        request.key.validate()?;
        require(request.instance != 0, PolicyError::InvalidInput)?;
        if let Some(previous) = receipts.get(&request.key) {
            require(
                previous.instance == request.instance
                    && previous.sku == request.item.sku
                    && previous.family == request.family,
                PolicyError::ReceiptConflict,
            )?;
            return Ok(Decision::Replay(previous.clone()));
        }
        self.validate()?;
        enabled(self.enabled)?;
        require(request.normal_persistent_inventory, PolicyError::NotAllowed)?;
        require(
            request.item.active_for(request.key.account, request.now),
            PolicyError::NotOwnedOrExpired,
        )?;
        require(
            resources.roll_bps < BASIS_POINTS && resources.lucci_balance >= 0,
            PolicyError::InvalidInput,
        )?;
        let row = self
            .recipes
            .iter()
            .find(|row| {
                row.family == request.family
                    && row.sku == request.item.sku
                    && row.from_level == request.level
            })
            .ok_or(PolicyError::NotAllowed)?;
        require(
            resources.lucci_balance as u64 >= row.lucci_cost,
            PolicyError::CapacityExceeded,
        )?;
        for material in &row.materials {
            // Subtract matching usable stacks from the requirement; the
            // accumulator never overflows even for large inventories.
            let mut needed = material.quantity;
            for owned in resources.materials.iter().filter(|owned| {
                owned.sku == material.sku && owned.active_for(request.key.account, request.now)
            }) {
                needed = needed.saturating_sub(owned.quantity);
                if needed == 0 {
                    break;
                }
            }
            require(needed == 0, PolicyError::NotOwnedOrExpired)?;
        }
        let success = resources.roll_bps < row.success_bps;
        let change = if success {
            UpgradeChange::Level(row.to_level)
        } else {
            match row.on_failure {
                UpgradeFailure::Keep => UpgradeChange::Keep,
                UpgradeFailure::Destroy => UpgradeChange::Destroy,
            }
        };
        let (consume_materials, debit_lucci) = if success || row.charge_on_failure {
            (row.materials.clone(), row.lucci_cost)
        } else {
            (Vec::new(), 0)
        };
        Ok(Decision::Apply(UpgradeResult {
            instance: request.instance,
            sku: row.sku.clone(),
            family: row.family,
            change,
            consume_materials,
            debit_lucci,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reward_rounds_down_and_respects_each_independent_bound() {
        assert_eq!(reward_bonus(3, 150, 100), 1);
        assert_eq!(reward_bonus(3, 199, 1), 1);
        assert_eq!(reward_bonus(80, 100, 100), 0);
        assert_eq!(
            reward_bonus(ECONOMY_AMOUNT_MAX - 1, MAX_REWARD_PERCENT, 100),
            1
        );
        assert_eq!(
            reward_bonus(ECONOMY_AMOUNT_MAX, MAX_REWARD_PERCENT, u64::MAX),
            0
        );
    }

    #[test]
    fn signed_vault_math_checks_multiplication_and_addition_separately() {
        assert_eq!(vault_cap(1, i64::MAX, 2), Err(PolicyError::Overflow));
        assert_eq!(vault_cap(i64::MAX, 1, 1), Err(PolicyError::Overflow));
        assert_eq!(vault_cap(i64::MAX - 1, 1, 1), Ok(i64::MAX));
    }

    #[test]
    fn maximum_timestamp_never_wraps_a_password_cooldown() {
        let policy = SecondPasswordPolicy {
            enabled: true,
            max_failed_attempts: 1,
            ..Default::default()
        };
        let state = SecondPasswordState::new(1, true).unwrap();
        let session = Some(PrimarySession {
            account: 1,
            session: 1,
        });
        let result = policy
            .attempt(
                &state,
                session,
                u64::MAX,
                SecondPasswordAction::Unlock,
                false,
            )
            .unwrap();
        assert_eq!(result.outcome, SecondPasswordOutcome::Locked);
        assert_eq!(
            policy.may_attempt(&result.state, session, u64::MAX),
            Ok(false)
        );
        assert_eq!(
            policy.may_attempt(&result.state, session, u64::MAX - 1),
            Err(PolicyError::InvalidInput)
        );
    }
}
