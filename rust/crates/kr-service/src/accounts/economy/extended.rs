//! Explicit F4 settlements on the SQLite account backend.
use super::*;
use crate::f4_extended_policy::{
    BoosterRoomContext, BoosterTicket, BoosterTicketPolicy, Decision, LevelGrant, LevelReward,
    LevelRewardPolicy, LucciVaultPolicy, PolicyError, SettlementKey,
};
use serde_json::json;

// Schema 7 has no other credit kind. Keep these in its non-match grant kind
// with "/" in the key, which cannot occur in an operator grant's public key.
const GRANT_KIND: &str = "admin_grant";

/// The room rules result and server-issued ticket snapshot for a journaled match.
/// Keep the same snapshot on retry; these are trusted server inputs.
#[derive(Clone, Copy, Debug)]
pub struct BoosterSettlementRequest<'a> {
    pub key: SettlementKey,
    pub room: BoosterRoomContext<'a>,
    pub ticket: BoosterTicket,
}

impl Store {
    /// Add the eligible booster bonus to an already settled base match reward.
    /// The match and account identify the credit; ticket expiry uses the
    /// journaled finish time so a delayed settlement cannot change eligibility.
    pub fn apply_booster_settlement(
        &self,
        player_id: i64,
        request: BoosterSettlementRequest<'_>,
        booster: &BoosterTicketPolicy,
        vault: &LucciVaultPolicy,
        progression: &EconomyPolicy,
        now: i64,
    ) -> Result<Decision<LedgerEntry>> {
        let BoosterSettlementRequest { key, room, ticket } = request;
        if !booster.enabled
            || key.account == 0
            || key.match_id == 0
            || ticket.account != key.account
        {
            return Err(Error::InvalidInput);
        }
        booster.validate().map_err(policy_error)?;
        progression.validate()?;
        let cap = lucci_cap(vault)?;
        let match_id = i64::try_from(key.match_id).map_err(|_| Error::InvalidInput)?;
        let base_key = format!("match:{match_id}");
        let receipt_key = format!("f4/booster/{match_id}");
        let hash = request_hash(&json!({
            "v": 1, "op": "f4_booster", "match_id": match_id, "account": key.account,
            "room": [room.room_group, room.mode, room.map, room.room_rules_allow, room.normal_unranked],
            "ticket": [ticket.account, ticket.expires_at, ticket.free_server_grant],
        }));
        self.economy_tx(now, |tx| {
            let account = check_account(tx, player_id, key.account, now)?;
            if let Some(receipt) = replay_grant(tx, player_id, &receipt_key, &hash)? {
                return Ok(receipt);
            }
            let (map, ended_at): (String, i64) = tx
                .query_row(
                    "SELECT m.map_name,m.ended_at FROM matches m
                     JOIN match_players p ON p.match_id=m.id
                     WHERE m.id=?1 AND m.state='complete' AND p.player_id=?2 AND p.account_id=?3",
                    params![match_id, player_id, account],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .optional()?
                .ok_or(Error::InvalidInput)?;
            let ended_at = u64::try_from(ended_at).map_err(|_| Error::Schema)?;
            if map != room.map
                || !booster
                    .allows_ticket_map(key.account, ended_at, ticket, room)
                    .map_err(policy_error)?
            {
                return Err(Error::InvalidInput);
            }
            let base = Self::find_ledger(tx, player_id, "match_reward", &base_key)?
                .filter(|entry| entry.match_id == Some(match_id))
                .ok_or(Error::Conflict)?;
            let Decision::Apply(settlement) = booster
                .settle(
                    key,
                    ended_at,
                    Some(ticket),
                    room,
                    RewardAmount {
                        exp: u64::try_from(base.exp_delta).map_err(|_| Error::Schema)?,
                        lucci: u64::try_from(base.lucci_delta).map_err(|_| Error::Schema)?,
                    },
                    &Default::default(),
                )
                .map_err(policy_error)?
            else {
                return Err(Error::Conflict);
            };
            let delta = (
                amount(settlement.bonus.exp)?,
                amount(settlement.bonus.lucci)?,
            );
            let profile = Self::read_profile(tx, player_id)?;
            let lucci = credit_lucci(profile.lucci, delta.1, cap)?;
            if delta != (0, 0) {
                let exp = profile.exp.checked_add(delta.0).ok_or(Error::Exhausted)?;
                let level = profile.level.max(progression.level_for(exp as u64));
                Self::write_profile(
                    tx,
                    &PlayerProfile {
                        exp,
                        level,
                        lucci,
                        ..profile
                    },
                )?;
            }
            record_grant(tx, player_id, &receipt_key, &hash, delta, None, now)
        })
    }

    /// Claim one configured grant per player and reached level. The caller
    /// supplies the server-resolved character. Chest SKUs use the catalog's
    /// inventory rules. A different grant for an already claimed level conflicts.
    pub fn claim_level_milestone_reward(
        &self,
        player_id: i64,
        grant: &LevelGrant,
        levels: &LevelRewardPolicy,
        vault: &LucciVaultPolicy,
        catalog: &ShopCatalog,
        now: i64,
    ) -> Result<Decision<LedgerEntry>> {
        let claim = grant.claim;
        if !levels.enabled || !(2..=LEVEL_MAX).contains(&usize::from(claim.level)) {
            return Err(Error::InvalidInput);
        }
        if !levels
            .grants_for_level_gain(
                claim.account,
                claim.character,
                claim.level - 1,
                claim.level,
                &Default::default(),
            )
            .map_err(policy_error)?
            .contains(grant)
        {
            return Err(Error::InvalidInput);
        }
        let cap = lucci_cap(vault)?;
        let reward = match &grant.reward {
            LevelReward::Lucci(lucci) => json!({"lucci": lucci}),
            LevelReward::Chest(chest) => json!({"sku": chest.sku, "quantity": chest.quantity}),
        };
        let hash = request_hash(&json!({
            "v": 1, "op": "f4_milestone", "account": claim.account,
            "character": claim.character, "level": claim.level, "reward": reward,
        }));
        let receipt_key = format!("f4/milestone/{}", claim.level);
        self.economy_tx(now, |tx| {
            check_account(tx, player_id, claim.account, now)?;
            if let Some(receipt) = replay_grant(tx, player_id, &receipt_key, &hash)? {
                return Ok(receipt);
            }
            let profile = Self::read_profile(tx, player_id)?;
            if profile.level < i64::from(claim.level) {
                return Err(Error::InvalidInput);
            }
            let (delta, sku_id) = match &grant.reward {
                LevelReward::Lucci(lucci) => (amount(*lucci)?, None),
                LevelReward::Chest(chest) => (0, Some(chest.sku.as_str())),
            };
            let lucci = credit_lucci(profile.lucci, delta, cap)?;
            if let LevelReward::Chest(chest) = &grant.reward {
                sku(&chest.sku)?;
                let item = catalog.item(&chest.sku).ok_or(Error::InvalidInput)?;
                catalog.validate()?;
                grant_chest(tx, player_id, item, chest.quantity, now)?;
            }
            Self::write_profile(tx, &PlayerProfile { lucci, ..profile })?;
            record_grant(tx, player_id, &receipt_key, &hash, (0, delta), sku_id, now)
        })
    }
}

fn check_account(tx: &Transaction<'_>, player: i64, account: u64, now: i64) -> Result<i64> {
    let owner = Store::active_player(tx, player, now)?;
    if u64::try_from(owner) != Ok(account) {
        return Err(Error::InvalidInput);
    }
    Ok(owner)
}

fn policy_error(error: PolicyError) -> Error {
    match error {
        PolicyError::CapacityExceeded => Error::Conflict,
        PolicyError::Overflow => Error::Exhausted,
        _ => Error::InvalidInput,
    }
}

fn lucci_cap(vault: &LucciVaultPolicy) -> Result<Option<i64>> {
    if vault.enabled {
        // Schema 7/8 does not persist expansions, so use the base capacity.
        Ok(Some(vault.capacity(0).map_err(policy_error)?))
    } else {
        vault.validate().map_err(policy_error)?;
        Ok(None)
    }
}

fn credit_lucci(balance: i64, delta: i64, cap: Option<i64>) -> Result<i64> {
    let next = balance.checked_add(delta).ok_or(Error::Exhausted)?;
    if cap.is_some_and(|cap| balance > cap || next > cap) {
        return Err(Error::Conflict);
    }
    Ok(next)
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
    delta: (i64, i64),
    sku: Option<&str>,
    now: i64,
) -> Result<Decision<LedgerEntry>> {
    Store::insert_ledger(tx, player, GRANT_KIND, key, hash, delta, None, sku, now)?;
    Ok(Decision::Apply(
        Store::find_ledger(tx, player, GRANT_KIND, key)?.ok_or(Error::Storage)?,
    ))
}

fn grant_chest(
    tx: &Transaction<'_>,
    player: i64,
    item: &ShopItem,
    quantity: u32,
    now: i64,
) -> Result<()> {
    let current: Option<(String, i64, Option<i64>)> = tx
        .query_row(
            "SELECT grant_kind,quantity,expires_at FROM player_inventory
             WHERE player_id=?1 AND sku_id=?2",
            params![player, item.sku_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?;
    let kind = item.grant.kind();
    if current.as_ref().is_some_and(|row| row.0 != kind) {
        return Err(Error::Conflict);
    }
    let (quantity, expires_at) = match item.grant {
        ItemGrant::Permanent if quantity != 1 => return Err(Error::InvalidInput),
        ItemGrant::Permanent if current.is_some() => return Err(Error::Conflict),
        ItemGrant::Permanent => (1, None),
        ItemGrant::Stackable { max_stack } => {
            let total = current
                .as_ref()
                .map_or(0, |row| row.1)
                .checked_add(i64::from(quantity))
                .filter(|&total| total <= i64::from(max_stack))
                .ok_or(Error::Conflict)?;
            (total, None)
        }
        ItemGrant::Timed { duration_seconds } => {
            let duration = duration_seconds
                .checked_mul(u64::from(quantity))
                .ok_or(Error::Exhausted)?;
            let start = now.max(current.and_then(|row| row.2).unwrap_or(0));
            (
                1,
                Some(
                    start
                        .checked_add(amount(duration)?)
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
        params![player, item.sku_id, kind, quantity, expires_at, now],
    )?;
    Ok(())
}
