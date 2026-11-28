//! Opt-in F4 operations backed by the existing economy and audit tables.
use super::*;
use crate::f4_extended_policy::{
    CouponRequest, Decision, GiftCouponGrant, GiftCouponPolicy, ItemOrigin, LucciVaultPolicy,
    OperationKey, OwnedItem, PolicyError, RecordEraseRequest, RecordEraserPolicy, RecordScope,
    RecordValue, TitlePolicy, TitleUnlockDecision, UpgradeChange, UpgradeFamily, UpgradePolicy,
    UpgradeRequest, UpgradeResources, WinLossRecord,
};
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};

// Operator grant keys cannot contain '/'.
const GRANT_KIND: &str = "admin_grant";

#[derive(Clone, Copy, Debug)]
pub struct RecordEraserOperation<'a> {
    pub key: u64,
    pub sku: &'a str,
    pub scope: RecordScope,
}

#[derive(Clone, Copy, Debug)]
pub struct ItemUpgradeOperation<'a> {
    pub key: u64,
    /// The recipe's base SKU, even when upgrading an already upgraded item.
    pub sku: &'a str,
    pub family: UpgradeFamily,
    pub level: u16,
    /// Trusted server roll; a retry returns the committed result.
    pub roll_bps: u16,
}

impl Store {
    /// Redeem a server-resolved local coupon. The reserved offer grant SKU
    /// `lucci` means currency in units of grant quantity. Other grant SKUs
    /// follow their catalog inventory rules.
    pub fn redeem_gift_coupon(
        &self,
        player_id: i64,
        request: CouponRequest<'_>,
        policy: &GiftCouponPolicy,
        vault: &LucciVaultPolicy,
        catalog: &ShopCatalog,
    ) -> Result<Decision<LedgerEntry>> {
        if !policy.enabled
            || request.key.id == 0
            || request.coupon_id == 0
            || !request.locally_issued
            || request.issued_to != request.key.account
            || request.issued_at > request.now
        {
            return Err(Error::InvalidInput);
        }
        let now = i64::try_from(request.now).map_err(|_| Error::InvalidInput)?;
        let key = format!("f4/coupon/{}", request.coupon_id);
        let hash = request_hash(&json!({
            "v": 1, "op": "f4_coupon", "account": request.key.account,
            "coupon": request.coupon_id, "operation": request.key.id,
            "offer": request.offer, "issued_at": request.issued_at,
        }));
        self.economy_tx(now, |tx| {
            let account = Self::active_player(tx, player_id, now)?;
            if u64::try_from(account) != Ok(request.key.account) {
                return Err(Error::InvalidInput);
            }
            if let Some(replay) = replay_grant(tx, player_id, &key, &hash)? {
                return Ok(replay);
            }
            // The ledger's unique key includes player_id. Check globally
            // under the same writer lock so an ID cannot be used by two players.
            if tx.query_row(
                "SELECT EXISTS(SELECT 1 FROM ledger_entries
                 WHERE kind=?1 AND idempotency_key=?2)",
                params![GRANT_KIND, key],
                |r| r.get::<_, bool>(0),
            )? {
                return Err(Error::Conflict);
            }
            let Decision::Apply(GiftCouponGrant::Coupon { item, .. }) = policy
                .redeem(request, &BTreeSet::new(), &BTreeMap::new())
                .map_err(policy_error)?
            else {
                return Err(Error::InvalidInput);
            };
            let mut profile = Self::read_profile(tx, player_id)?;
            let (delta, granted) = if item.sku == "lucci" {
                let delta = i64::from(item.quantity);
                // No persisted vault expansions exist in this schema.
                profile.lucci = if vault.enabled {
                    vault
                        .deposit(0, profile.lucci, delta)
                        .map_err(policy_error)?
                } else {
                    vault.validate().map_err(policy_error)?;
                    profile.lucci.checked_add(delta).ok_or(Error::Exhausted)?
                };
                (delta, None)
            } else {
                catalog.validate()?;
                let entry = catalog.item(&item.sku).ok_or(Error::InvalidInput)?;
                grant_coupon_item(tx, player_id, entry, item.quantity, now)?;
                (0, Some(item.sku.as_str()))
            };
            Self::write_profile(tx, &profile)?;
            record_grant(tx, player_id, &key, &hash, delta, granted, now)
        })
    }

    /// Apply a self-service eraser from persistent inventory. Level resets
    /// change the stored level and retain earned EXP. Win/loss resets record
    /// the before and after values in the audit; this schema has no mutable
    /// win/loss projection. Ladder is unsupported. Match history stays intact.
    pub fn apply_record_eraser(
        &self,
        player_id: i64,
        request: RecordEraserOperation<'_>,
        policy: &RecordEraserPolicy,
        now: i64,
    ) -> Result<Decision<LedgerEntry>> {
        if !policy.enabled || request.key == 0 {
            return Err(Error::InvalidInput);
        }
        sku(request.sku)?;
        let key = format!("f4/eraser/{}", request.key);
        let scope = match request.scope {
            RecordScope::Level => "level",
            RecordScope::WinLoss => "win_loss",
            RecordScope::Ladder => return Err(Error::InvalidInput),
        };
        let hash = request_hash(&json!({
            "v": 1, "op": "f4_eraser", "player": player_id, "key": request.key,
            "sku": request.sku, "scope": scope,
        }));
        self.economy_tx(now, |tx| {
            let account = active_account(tx, player_id, now)?;
            if let Some(replay) = replay_grant(tx, player_id, &key, &hash)? {
                return Ok(replay);
            }
            let mut profile = Self::read_profile(tx, player_id)?;
            let before = match request.scope {
                RecordScope::Level => {
                    RecordValue::Level(u16::try_from(profile.level).map_err(|_| Error::Schema)?)
                }
                _ => {
                    let (wins, losses, _) = completed_stats(tx, player_id, account)?;
                    RecordValue::WinLoss(WinLossRecord {
                        wins: u32::try_from(wins).map_err(|_| Error::Exhausted)?,
                        losses: u32::try_from(losses).map_err(|_| Error::Exhausted)?,
                    })
                }
            };
            let now_unsigned = u64::try_from(now).map_err(|_| Error::InvalidInput)?;
            let eraser = owned_item(tx, player_id, account, request.sku, now)?;
            let Decision::Apply(reset) = policy
                .erase(
                    RecordEraseRequest {
                        key: OperationKey { account, id: request.key },
                        actor: account,
                        authorized: true,
                        audit_available: true,
                        eraser,
                        before,
                        now: now_unsigned,
                    },
                    &BTreeMap::new(),
                )
                .map_err(policy_error)?
            else {
                return Err(Error::InvalidInput);
            };
            consume_item(tx, player_id, eraser, 1, now)?;
            if let RecordValue::Level(level) = reset.after {
                profile.level = i64::from(level);
            }
            Self::write_profile(tx, &profile)?;
            tx.execute(
                "INSERT INTO admin_audit(request_id,actor,operation,account_id,reason,at_utc,details,target_kind,target_id)
                 VALUES(?1,?2,'f4_record_eraser',?3,'f4 record reset',?4,?5,'account',?2)",
                params![
                    hash,
                    account.to_string(),
                    i64::try_from(account).map_err(|_| Error::InvalidInput)?,
                    now,
                    json!({
                        "key": key, "sku": reset.consume_sku, "scope": scope,
                        "before": record_value(&reset.before), "after": record_value(&reset.after),
                    }).to_string(),
                ],
            )?;
            record_grant(tx, player_id, &key, &hash, 0, Some(request.sku), now)
        })
    }

    /// Apply a recipe to a single permanent or timed inventory unit. The
    /// level is encoded in the SKU (`<base>.f4-<family>-<level>` above level
    /// zero), so a new key cannot apply the old recipe to a spent input.
    /// A successful replacement retains its grant kind and expiry.
    pub fn apply_item_upgrade(
        &self,
        player_id: i64,
        request: ItemUpgradeOperation<'_>,
        policy: &UpgradePolicy,
        now: i64,
    ) -> Result<Decision<LedgerEntry>> {
        if !policy.enabled || request.key == 0 {
            return Err(Error::InvalidInput);
        }
        sku(request.sku)?;
        let family = match request.family {
            UpgradeFamily::Item => "item",
            UpgradeFamily::Pet => "pet",
            UpgradeFamily::WaterBalloon => "water_balloon",
            UpgradeFamily::RpgGear => "rpg_gear",
        };
        let key = format!("f4/upgrade/{}", request.key);
        let hash = request_hash(&json!({
            "v": 1, "op": "f4_upgrade", "player": player_id, "key": request.key,
            "sku": request.sku, "family": family, "level": request.level,
        }));
        self.economy_tx(now, |tx| {
            let account = active_account(tx, player_id, now)?;
            if let Some(replay) = replay_grant(tx, player_id, &key, &hash)? {
                return Ok(replay);
            }
            policy.validate().map_err(policy_error)?;
            let recipe = policy
                .recipes
                .iter()
                .find(|row| {
                    row.family == request.family
                        && row.sku == request.sku
                        && row.from_level == request.level
                })
                .ok_or(Error::InvalidInput)?;
            let input = upgrade_sku(request.sku, family, request.level)?;
            let output = upgrade_sku(request.sku, family, recipe.to_level)?;
            let item = owned_item(tx, player_id, account, &input, now)?;
            let single_unit: bool = tx.query_row(
                "SELECT grant_kind IN ('permanent','timed') AND quantity=1
                 FROM player_inventory WHERE player_id=?1 AND sku_id=?2",
                params![player_id, input],
                |r| r.get(0),
            )?;
            if !single_unit {
                return Err(Error::InvalidInput);
            }
            let mut profile = Self::read_profile(tx, player_id)?;
            if (profile.lucci as u64) < recipe.lucci_cost {
                return Err(Error::InsufficientFunds);
            }
            let materials = recipe
                .materials
                .iter()
                .map(|mat| {
                    sku(&mat.sku)?;
                    if mat.sku == input || mat.sku == output {
                        return Err(Error::InvalidInput);
                    }
                    owned_item(tx, player_id, account, &mat.sku, now)
                })
                .collect::<Result<Vec<_>>>()?;
            let Decision::Apply(result) = policy
                .upgrade(
                    UpgradeRequest {
                        key: OperationKey {
                            account,
                            id: request.key,
                        },
                        // This schema identifies the single unit by its inventory SKU.
                        instance: account,
                        item: OwnedItem {
                            sku: request.sku,
                            ..item
                        },
                        family: request.family,
                        level: request.level,
                        now: u64::try_from(now).map_err(|_| Error::InvalidInput)?,
                        normal_persistent_inventory: true,
                    },
                    UpgradeResources {
                        materials: &materials,
                        lucci_balance: profile.lucci,
                        roll_bps: request.roll_bps,
                    },
                    &BTreeMap::new(),
                )
                .map_err(policy_error)?
            else {
                return Err(Error::InvalidInput);
            };
            for (cost, owned) in result.consume_materials.iter().zip(materials) {
                consume_item(tx, player_id, owned, cost.quantity, now)?;
            }
            let price = amount(result.debit_lucci)?;
            profile.lucci -= price;
            let granted = match result.change {
                UpgradeChange::Level(_) => {
                    clear_equipped(tx, player_id, &input)?;
                    tx.execute(
                        "UPDATE player_inventory SET sku_id=?3,updated_at=?4
                         WHERE player_id=?1 AND sku_id=?2",
                        params![player_id, input, output, now],
                    )?;
                    Some(output.as_str())
                }
                UpgradeChange::Keep => Some(input.as_str()),
                UpgradeChange::Destroy => {
                    consume_item(tx, player_id, item, 1, now)?;
                    None
                }
            };
            if price != 0
                || !result.consume_materials.is_empty()
                || result.change != UpgradeChange::Keep
            {
                Self::write_profile(tx, &profile)?;
            }
            record_grant(tx, player_id, &key, &hash, -price, granted, now)
        })
    }

    /// Read-only title eligibility. Recognized conditions are `level>=N`,
    /// `exp>=N`, `wins>=N`, `losses>=N` and `matches>=N`. Match counters
    /// come from completed history. No title or notification is persisted.
    pub fn evaluate_title_unlock(
        &self,
        player_id: i64,
        title: &str,
        policy: &TitlePolicy,
        now: i64,
    ) -> Result<Option<TitleUnlockDecision>> {
        if !policy.enabled {
            return Ok(None);
        }
        policy.validate().map_err(policy_error)?;
        let Some(row) = policy
            .unlock_table
            .iter()
            .flatten()
            .find(|row| row.title == title)
        else {
            return Ok(None);
        };
        let Some((metric, min)) = row.condition.split_once(">=") else {
            return Ok(None);
        };
        let Ok(min) = min.parse::<u64>() else {
            return Ok(None);
        };
        let tx = Transaction::new_unchecked(&self.db, TransactionBehavior::Deferred)?;
        let account = active_account(&tx, player_id, now)?;
        let actual = match metric {
            "level" => Self::read_profile(&tx, player_id)?.level,
            "exp" => Self::read_profile(&tx, player_id)?.exp,
            "wins" | "losses" | "matches" => {
                let (wins, losses, matches) = completed_stats(&tx, player_id, account)?;
                match metric {
                    "wins" => wins,
                    "losses" => losses,
                    _ => matches,
                }
            }
            _ => return Ok(None),
        };
        if u64::try_from(actual).map_err(|_| Error::Schema)? < min {
            return Ok(None);
        }
        policy
            .unlock(
                title,
                &BTreeSet::new(),
                &BTreeSet::from([row.condition.clone()]),
            )
            .map(Some)
            .map_err(policy_error)
    }
}

fn policy_error(error: PolicyError) -> Error {
    match error {
        PolicyError::CapacityExceeded
        | PolicyError::AlreadyRedeemed
        | PolicyError::ReceiptConflict => Error::Conflict,
        PolicyError::NotOwnedOrExpired => Error::InsufficientQuantity,
        PolicyError::Overflow => Error::Exhausted,
        _ => Error::InvalidInput,
    }
}

fn active_account(db: &Connection, player: i64, now: i64) -> Result<u64> {
    if now < 0 {
        return Err(Error::InvalidInput);
    }
    u64::try_from(Store::active_player(db, player, now)?).map_err(|_| Error::Schema)
}

fn replay_grant(
    tx: &Transaction<'_>,
    player: i64,
    key: &str,
    hash: &str,
) -> Result<Option<Decision<LedgerEntry>>> {
    match Store::find_ledger(tx, player, GRANT_KIND, key)? {
        Some(entry) if entry.request_hash == hash => Ok(Some(Decision::Replay(entry))),
        Some(_) => Err(Error::Conflict),
        None => Ok(None),
    }
}

fn record_grant(
    tx: &Transaction<'_>,
    player: i64,
    key: &str,
    hash: &str,
    delta: i64,
    sku: Option<&str>,
    now: i64,
) -> Result<Decision<LedgerEntry>> {
    Store::insert_ledger(
        tx,
        player,
        GRANT_KIND,
        key,
        hash,
        (0, delta),
        None,
        sku,
        now,
    )?;
    Ok(Decision::Apply(
        Store::find_ledger(tx, player, GRANT_KIND, key)?.ok_or(Error::Storage)?,
    ))
}

fn completed_stats(db: &Connection, player: i64, account: u64) -> Result<(i64, i64, i64)> {
    Ok(db.query_row(
        "SELECT count(CASE WHEN p.outcome='win' THEN 1 END),
                count(CASE WHEN p.outcome='loss' THEN 1 END), count(*)
         FROM match_players p JOIN matches m ON m.id=p.match_id
         WHERE p.account_id=?1 AND p.player_id=?2 AND m.state='complete'",
        params![
            i64::try_from(account).map_err(|_| Error::InvalidInput)?,
            player
        ],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )?)
}

fn record_value(value: &RecordValue) -> serde_json::Value {
    match value {
        RecordValue::Level(level) => json!(level),
        RecordValue::WinLoss(record) => json!({"wins": record.wins, "losses": record.losses}),
        RecordValue::Ladder(points) => json!(points),
    }
}

fn upgrade_sku(base: &str, family: &str, level: u16) -> Result<String> {
    let value = if level == 0 {
        base.to_owned()
    } else {
        format!("{base}.f4-{family}-{level}")
    };
    sku(&value)?;
    Ok(value)
}

fn owned_item<'a>(
    tx: &Transaction<'_>,
    player: i64,
    account: u64,
    sku: &'a str,
    now: i64,
) -> Result<OwnedItem<'a>> {
    let (quantity, expires_at) = tx
        .query_row(
            "SELECT quantity,expires_at FROM player_inventory
         WHERE player_id=?1 AND sku_id=?2 AND (expires_at IS NULL OR expires_at>?3)",
            params![player, sku, now],
            |r| Ok((r.get::<_, i64>(0)?, r.get::<_, Option<i64>>(1)?)),
        )
        .optional()?
        .ok_or(Error::InsufficientQuantity)?;
    Ok(OwnedItem {
        owner: account,
        sku,
        quantity: u32::try_from(quantity).map_err(|_| Error::Schema)?,
        expires_at: expires_at
            .map(|value| u64::try_from(value).map_err(|_| Error::Schema))
            .transpose()?,
        origin: ItemOrigin::PersistentInventory,
    })
}

fn clear_equipped(tx: &Transaction<'_>, player: i64, sku: &str) -> Result<()> {
    tx.execute(
        "DELETE FROM player_equipped WHERE player_id=?1 AND sku_id=?2",
        params![player, sku],
    )?;
    Ok(())
}

fn consume_item(
    tx: &Transaction<'_>,
    player: i64,
    item: OwnedItem<'_>,
    quantity: u32,
    now: i64,
) -> Result<()> {
    let remaining = item
        .quantity
        .checked_sub(quantity)
        .ok_or(Error::InsufficientQuantity)?;
    if remaining == 0 {
        clear_equipped(tx, player, item.sku)?;
        tx.execute(
            "DELETE FROM player_inventory WHERE player_id=?1 AND sku_id=?2",
            params![player, item.sku],
        )?;
    } else {
        tx.execute(
            "UPDATE player_inventory SET quantity=?3,updated_at=?4 WHERE player_id=?1 AND sku_id=?2",
            params![player, item.sku, remaining, now],
        )?;
    }
    Ok(())
}

fn grant_coupon_item(
    tx: &Transaction<'_>,
    player: i64,
    item: &ShopItem,
    quantity: u32,
    now: i64,
) -> Result<()> {
    let current: Option<(String, i64, Option<i64>)> = tx.query_row(
        "SELECT grant_kind,quantity,expires_at FROM player_inventory WHERE player_id=?1 AND sku_id=?2",
        params![player, item.sku_id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    ).optional()?;
    let kind = item.grant.kind();
    if current.as_ref().is_some_and(|row| row.0 != kind) {
        return Err(Error::Conflict);
    }
    let (quantity, expiry) = match item.grant {
        ItemGrant::Permanent if quantity != 1 => return Err(Error::InvalidInput),
        ItemGrant::Permanent if current.is_some() => return Err(Error::Conflict),
        ItemGrant::Permanent => (1, None),
        ItemGrant::Stackable { max_stack } => (
            current
                .as_ref()
                .map_or(0, |r| r.1)
                .checked_add(i64::from(quantity))
                .filter(|&qty| qty <= i64::from(max_stack))
                .ok_or(Error::Conflict)?,
            None,
        ),
        ItemGrant::Timed { duration_seconds } => {
            let extension = duration_seconds
                .checked_mul(u64::from(quantity))
                .ok_or(Error::Exhausted)?;
            let start = now.max(current.and_then(|r| r.2).unwrap_or(0));
            (
                1,
                Some(
                    start
                        .checked_add(amount(extension)?)
                        .ok_or(Error::Exhausted)?,
                ),
            )
        }
    };
    tx.execute(
        "INSERT INTO player_inventory(player_id,sku_id,grant_kind,quantity,expires_at,acquired_at,updated_at)
         VALUES(?1,?2,?3,?4,?5,?6,?6)
         ON CONFLICT(player_id,sku_id) DO UPDATE SET
            quantity=excluded.quantity,expires_at=excluded.expires_at,updated_at=excluded.updated_at",
        params![player, item.sku_id, kind, quantity, expiry, now],
    )?;
    Ok(())
}
