//! F4 company-design economy: configurable progression, Lucci ledger, shop,
//! inventory and equip. No KR wire field, original price or server formula is
//! inferred; every amount comes from operator configuration.
use super::*;
use serde::Deserialize;

mod consumption;
pub use consumption::*;
mod inventory;
mod ledger;
#[cfg(test)]
mod tests;

/// Upper bound for any configured single amount (reward, price, duration).
/// Products of amounts are still computed with checked arithmetic.
pub const ECONOMY_AMOUNT_MAX: u64 = 1_000_000_000_000;
/// Upper bound for the number of configured levels.
pub const LEVEL_MAX: usize = 10_000;
/// Upper bound for the number of catalog entries.
pub const CATALOG_ITEMS_MAX: usize = 100_000;

/// Explicit operator validation for configuration loaded from JSON.
pub trait Validate {
    fn validate(&self) -> Result<()>;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RewardAmount {
    pub exp: u64,
    pub lucci: u64,
}
impl Validate for RewardAmount {
    fn validate(&self) -> Result<()> {
        if self.exp > ECONOMY_AMOUNT_MAX || self.lucci > ECONOMY_AMOUNT_MAX {
            return Err(Error::InvalidInput);
        }
        Ok(())
    }
}

/// Match reward and level policy. The default is disabled, which keeps the M6
/// zero-reward behavior for every existing flow.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EconomyPolicy {
    pub enabled: bool,
    pub win: RewardAmount,
    pub loss: RewardAmount,
    pub draw: RewardAmount,
    /// Players whose exit was `left` or `disconnected` receive zero unless set.
    pub grant_on_disconnect_or_left: bool,
    /// `level_thresholds[i]` is the total EXP needed for level `i + 1`. It must
    /// start at 0 and increase strictly; the maximum level is its length.
    pub level_thresholds: Vec<u64>,
}
impl Default for EconomyPolicy {
    fn default() -> Self {
        Self {
            enabled: false,
            win: RewardAmount::default(),
            loss: RewardAmount::default(),
            draw: RewardAmount::default(),
            grant_on_disconnect_or_left: false,
            level_thresholds: vec![0],
        }
    }
}
impl Validate for EconomyPolicy {
    fn validate(&self) -> Result<()> {
        self.win.validate()?;
        self.loss.validate()?;
        self.draw.validate()?;
        let t = &self.level_thresholds;
        if t.first() != Some(&0)
            || t.len() > LEVEL_MAX
            || t.windows(2).any(|w| w[0] >= w[1])
            || t.iter().any(|&v| v > i64::MAX as u64)
        {
            return Err(Error::InvalidInput);
        }
        Ok(())
    }
}
impl EconomyPolicy {
    /// Level for a total EXP; multi-level jumps and the max-level cap follow
    /// from counting reached thresholds.
    pub fn level_for(&self, exp: u64) -> i64 {
        let reached = self.level_thresholds.partition_point(|&t| t <= exp);
        // Validation keeps thresholds[0] == 0 and len <= LEVEL_MAX.
        reached.max(1) as i64
    }
    pub fn reward(&self, outcome: MatchOutcome, exit: MatchExit) -> RewardAmount {
        let present = match exit {
            MatchExit::Present => true,
            MatchExit::Left | MatchExit::Disconnected => self.grant_on_disconnect_or_left,
            MatchExit::Unknown => false,
        };
        if !self.enabled || !present {
            return RewardAmount::default();
        }
        match outcome {
            MatchOutcome::Win => self.win,
            MatchOutcome::Loss => self.loss,
            MatchOutcome::Draw => self.draw,
            MatchOutcome::Cancelled => RewardAmount::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShopCategory {
    Character,
    Decoration,
    Emotion,
    Consumable,
    Collection,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ItemGrant {
    Permanent,
    Stackable { max_stack: u32 },
    Timed { duration_seconds: u64 },
}
impl ItemGrant {
    pub(crate) fn kind(self) -> &'static str {
        match self {
            Self::Permanent => "permanent",
            Self::Stackable { .. } => "stackable",
            Self::Timed { .. } => "timed",
        }
    }
}
impl Validate for ItemGrant {
    fn validate(&self) -> Result<()> {
        match *self {
            Self::Permanent => Ok(()),
            Self::Stackable { max_stack } if max_stack >= 1 => Ok(()),
            Self::Timed { duration_seconds }
                if (1..=ECONOMY_AMOUNT_MAX).contains(&duration_seconds) =>
            {
                Ok(())
            }
            _ => Err(Error::InvalidInput),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShopItem {
    pub sku_id: String,
    pub category: ShopCategory,
    pub price_lucci: u64,
    pub grant: ItemGrant,
    pub equip_slot: Option<String>,
    pub enabled: bool,
}
impl Validate for ShopItem {
    fn validate(&self) -> Result<()> {
        sku(&self.sku_id)?;
        if let Some(slot) = &self.equip_slot {
            equip_slot(slot)?;
        }
        if self.price_lucci > ECONOMY_AMOUNT_MAX {
            return Err(Error::InvalidInput);
        }
        self.grant.validate()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShopCatalog {
    pub items: Vec<ShopItem>,
}
impl Validate for ShopCatalog {
    fn validate(&self) -> Result<()> {
        if self.items.len() > CATALOG_ITEMS_MAX {
            return Err(Error::InvalidInput);
        }
        let mut seen = std::collections::BTreeSet::new();
        for item in &self.items {
            item.validate()?;
            if !seen.insert(item.sku_id.as_str()) {
                return Err(Error::InvalidInput);
            }
        }
        Ok(())
    }
}
impl ShopCatalog {
    pub fn item(&self, sku_id: &str) -> Option<&ShopItem> {
        self.items.iter().find(|item| item.sku_id == sku_id)
    }
    /// O1 `unlock_all` entitlements: every enabled SKU, computed from the
    /// catalog alone without any database access or Lucci charge.
    pub fn session_unlocked(&self) -> Vec<UsableItem> {
        self.items
            .iter()
            .filter(|item| item.enabled)
            .map(|item| UsableItem {
                sku_id: item.sku_id.clone(),
                unlocked_by_session: true,
                quantity: None,
                expires_at: None,
            })
            .collect()
    }
}

fn token(value: &str, max: usize, extra: &[u8]) -> Result<()> {
    if !(1..=max).contains(&value.len())
        || !value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b) || extra.contains(&b))
    {
        return Err(Error::InvalidInput);
    }
    Ok(())
}
pub(crate) fn sku(value: &str) -> Result<()> {
    token(value, 64, b"")
}
pub(crate) fn equip_slot(value: &str) -> Result<()> {
    token(value, 32, b"")
}
pub(crate) fn idempotency_key(value: &str) -> Result<()> {
    token(value, 128, b":")
}
/// Deterministic SHA-256 over a canonical JSON request description.
fn request_hash(value: &serde_json::Value) -> String {
    hex(&Sha256::digest(value.to_string().as_bytes()))
}
fn amount(value: u64) -> Result<i64> {
    i64::try_from(value).map_err(|_| Error::Exhausted)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PlayerProfile {
    pub player_id: i64,
    pub exp: i64,
    pub level: i64,
    pub lucci: i64,
    pub version: i64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LedgerEntry {
    pub id: i64,
    pub player_id: i64,
    pub kind: String,
    pub idempotency_key: String,
    pub request_hash: String,
    pub exp_delta: i64,
    pub lucci_delta: i64,
    pub match_id: Option<i64>,
    pub sku_id: Option<String>,
    pub created_at: i64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct InventoryItem {
    pub sku_id: String,
    pub grant_kind: String,
    pub quantity: i64,
    pub expires_at: Option<i64>,
    pub acquired_at: i64,
    pub updated_at: i64,
    /// False for a timed item whose `expires_at <= now`.
    pub active: bool,
    pub equipped_slots: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct EquippedItem {
    pub slot: String,
    pub sku_id: String,
    pub equipped_at: i64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PlayerReward {
    pub player_id: i64,
    pub account_id: i64,
    pub slot: u8,
    pub outcome: MatchOutcome,
    pub exit: MatchExit,
    pub exp_delta: i64,
    pub lucci_delta: i64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct MatchRewardReceipt {
    pub match_id: i64,
    /// True when ledger rows exist for this match (new or replayed).
    pub settled: bool,
    pub replayed: bool,
    pub request_hash: Option<String>,
    pub players: Vec<PlayerReward>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PurchaseReceipt {
    pub ledger_id: i64,
    pub player_id: i64,
    pub sku_id: String,
    pub quantity: u32,
    pub lucci_spent: i64,
    pub replayed: bool,
    pub profile: PlayerProfile,
    pub item: Option<InventoryItem>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct AdminGrantReceipt {
    pub ledger_id: i64,
    pub player_id: i64,
    pub lucci_delta: i64,
    pub sku_id: Option<String>,
    pub replayed: bool,
    pub profile: PlayerProfile,
    pub item: Option<InventoryItem>,
}

/// O1 `unlock_all` session access. With `unlock_all`, every enabled catalog
/// SKU is usable without any Lucci charge or database access; otherwise only
/// the player's own non-expired inventory counts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogAccess {
    pub unlock_all: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct UsableItem {
    pub sku_id: String,
    /// True when the entitlement comes from `unlock_all`, not from inventory.
    pub unlocked_by_session: bool,
    pub quantity: Option<i64>,
    pub expires_at: Option<i64>,
}

impl Store {
    /// One `BEGIN IMMEDIATE` writer transaction for every economy mutation.
    /// The durable clock watermark is checked in the same transaction.
    fn economy_tx<T>(&self, now: i64, op: impl FnOnce(&Transaction<'_>) -> Result<T>) -> Result<T> {
        let tx = Transaction::new_unchecked(&self.db, TransactionBehavior::Immediate)?;
        Self::clock(&tx, now)?;
        let out = op(&tx)?;
        tx.commit()?;
        Ok(out)
    }
    fn player_exists(db: &Connection, player: i64) -> Result<i64> {
        db.query_row(
            "SELECT account_id FROM players WHERE id=?1",
            [player],
            |r| r.get(0),
        )
        .optional()?
        .ok_or(Error::InvalidInput)
    }
    /// Account is enabled, not deleted and not an expired trial guest.
    fn active_player(db: &Connection, player: i64, now: i64) -> Result<i64> {
        let account = Self::player_exists(db, player)?;
        let active: bool = db.query_row(
            "SELECT disabled=0 AND login NOT GLOB '~deleted_*'
               AND NOT EXISTS(SELECT 1 FROM guest_accounts WHERE account_id=?1 AND expires_at<=?2)
             FROM accounts WHERE id=?1",
            params![account, now],
            |r| r.get(0),
        )?;
        if !active {
            return Err(Error::InvalidCredentials);
        }
        Ok(account)
    }
    fn read_profile(db: &Connection, player: i64) -> Result<PlayerProfile> {
        Ok(db
            .query_row(
                "SELECT exp,level,lucci,version FROM player_profiles WHERE player_id=?1",
                [player],
                |r| {
                    Ok(PlayerProfile {
                        player_id: player,
                        exp: r.get(0)?,
                        level: r.get(1)?,
                        lucci: r.get(2)?,
                        version: r.get(3)?,
                    })
                },
            )
            .optional()?
            .unwrap_or(PlayerProfile {
                player_id: player,
                exp: 0,
                level: 1,
                lucci: 0,
                version: 1,
            }))
    }
    /// Write a changed profile, creating the default row first when absent.
    fn write_profile(tx: &Transaction<'_>, profile: &PlayerProfile) -> Result<PlayerProfile> {
        tx.execute(
            "INSERT INTO player_profiles(player_id) VALUES(?1) ON CONFLICT(player_id) DO NOTHING",
            [profile.player_id],
        )?;
        let next = PlayerProfile {
            version: profile.version.checked_add(1).ok_or(Error::Exhausted)?,
            ..profile.clone()
        };
        if tx.execute(
            "UPDATE player_profiles SET exp=?1,level=?2,lucci=?3,version=?4
             WHERE player_id=?5 AND version=?6",
            params![
                next.exp,
                next.level,
                next.lucci,
                next.version,
                next.player_id,
                profile.version
            ],
        )? != 1
        {
            return Err(Error::Conflict);
        }
        Ok(next)
    }
    /// Read-only. Players without an F4 row (for example upgraded from schema 6)
    /// have the default profile.
    pub fn get_profile(&self, player_id: i64) -> Result<PlayerProfile> {
        Self::player_exists(&self.db, player_id)?;
        Self::read_profile(&self.db, player_id)
    }
    pub fn list_ledger(&self, player_id: i64, limit: u8) -> Result<Vec<LedgerEntry>> {
        if !(1..=100).contains(&limit) {
            return Err(Error::InvalidInput);
        }
        Self::player_exists(&self.db, player_id)?;
        let mut query = self.db.prepare(
            "SELECT id,player_id,kind,idempotency_key,request_hash,exp_delta,lucci_delta,match_id,sku_id,created_at
             FROM ledger_entries WHERE player_id=?1 ORDER BY id DESC LIMIT ?2",
        )?;
        Ok(query
            .query_map(params![player_id, limit], ledger_row)?
            .collect::<std::result::Result<_, _>>()?)
    }
    /// O1 helper. Never charges Lucci and, with `unlock_all`, never reads or
    /// writes the database.
    pub fn usable_items(
        &self,
        player_id: i64,
        catalog: &ShopCatalog,
        access: CatalogAccess,
        now: i64,
    ) -> Result<Vec<UsableItem>> {
        catalog.validate()?;
        if access.unlock_all {
            return Ok(catalog.session_unlocked());
        }
        Ok(self
            .list_inventory(player_id, now)?
            .into_iter()
            .filter(|item| item.active)
            .map(|item| UsableItem {
                sku_id: item.sku_id,
                unlocked_by_session: false,
                quantity: Some(item.quantity),
                expires_at: item.expires_at,
            })
            .collect())
    }
    pub fn can_use_sku(
        &self,
        player_id: i64,
        catalog: &ShopCatalog,
        access: CatalogAccess,
        sku_id: &str,
        now: i64,
    ) -> Result<bool> {
        sku(sku_id)?;
        Ok(self
            .usable_items(player_id, catalog, access, now)?
            .iter()
            .any(|item| item.sku_id == sku_id))
    }
    /// Ledger invariant for schema 7: every balance change has a ledger row.
    pub(super) fn validate_economy(db: &Connection) -> Result<()> {
        let broken: bool = db.query_row(
            "SELECT EXISTS(
               SELECT 1 FROM (SELECT player_id,sum(exp_delta) AS e,sum(lucci_delta) AS l
                              FROM ledger_entries GROUP BY player_id) s
               LEFT JOIN player_profiles p ON p.player_id=s.player_id
               WHERE coalesce(p.exp,0)<>s.e OR coalesce(p.lucci,0)<>s.l)
             OR EXISTS(SELECT 1 FROM player_profiles p WHERE (p.exp<>0 OR p.lucci<>0)
               AND NOT EXISTS(SELECT 1 FROM ledger_entries l WHERE l.player_id=p.player_id))",
            [],
            |r| r.get(0),
        )?;
        if broken {
            return Err(Error::Schema);
        }
        Ok(())
    }
    #[allow(clippy::too_many_arguments)]
    fn insert_ledger(
        tx: &Transaction<'_>,
        player: i64,
        kind: &str,
        key: &str,
        hash: &str,
        delta: (i64, i64),
        match_id: Option<i64>,
        sku_id: Option<&str>,
        now: i64,
    ) -> Result<i64> {
        tx.execute(
            "INSERT INTO ledger_entries(player_id,kind,idempotency_key,request_hash,exp_delta,lucci_delta,match_id,sku_id,created_at)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",
            params![player, kind, key, hash, delta.0, delta.1, match_id, sku_id, now],
        )?;
        Ok(tx.last_insert_rowid())
    }
    fn find_ledger(
        db: &Connection,
        player: i64,
        kind: &str,
        key: &str,
    ) -> Result<Option<LedgerEntry>> {
        Ok(db
            .query_row(
                "SELECT id,player_id,kind,idempotency_key,request_hash,exp_delta,lucci_delta,match_id,sku_id,created_at
                 FROM ledger_entries WHERE player_id=?1 AND kind=?2 AND idempotency_key=?3",
                params![player, kind, key],
                ledger_row,
            )
            .optional()?)
    }
}
fn ledger_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<LedgerEntry> {
    Ok(LedgerEntry {
        id: r.get(0)?,
        player_id: r.get(1)?,
        kind: r.get(2)?,
        idempotency_key: r.get(3)?,
        request_hash: r.get(4)?,
        exp_delta: r.get(5)?,
        lucci_delta: r.get(6)?,
        match_id: r.get(7)?,
        sku_id: r.get(8)?,
        created_at: r.get(9)?,
    })
}
