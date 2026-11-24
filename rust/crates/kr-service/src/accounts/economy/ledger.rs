//! Match rewards and administrative Lucci adjustments.
use super::*;
use serde_json::json;

struct Participant {
    player_id: i64,
    account_id: i64,
    slot: u8,
    outcome: Option<String>,
    exit: Option<String>,
}
fn parse<T: serde::de::DeserializeOwned>(label: Option<&str>) -> Result<Option<T>> {
    label
        .map(|s| {
            serde_json::from_value(serde_json::Value::String(s.into())).map_err(|_| Error::Schema)
        })
        .transpose()
}

impl Store {
    /// Settle a finished match once. The same match and policy replay the
    /// original ledger rows; a different policy or reward is a conflict. M6
    /// `match_players.reward` stays 0: F4 amounts live in the profile ledger.
    pub fn settle_match_rewards(
        &self,
        match_id: i64,
        policy: &EconomyPolicy,
        now: i64,
    ) -> Result<MatchRewardReceipt> {
        policy.validate()?;
        if match_id <= 0 {
            return Err(Error::InvalidInput);
        }
        self.economy_tx(now, |tx| Self::settle_tx(tx, match_id, policy, now))
    }

    /// Settlement body shared by `settle_match_rewards` and the journal's
    /// finish-and-settle transaction. The caller validates `policy` and owns
    /// the writer transaction and clock watermark.
    pub(in crate::accounts) fn settle_tx(
        tx: &Transaction<'_>,
        match_id: i64,
        policy: &EconomyPolicy,
        now: i64,
    ) -> Result<MatchRewardReceipt> {
        let state: String = tx
            .query_row("SELECT state FROM matches WHERE id=?1", [match_id], |r| {
                r.get(0)
            })
            .optional()?
            .ok_or(Error::InvalidInput)?;
        if state == "running" {
            return Err(Error::Conflict);
        }
        let cancelled = state == "cancelled";
        let participants: Vec<Participant> = tx
            .prepare(
                "SELECT player_id,account_id,slot,outcome,exit_kind FROM match_players
                 WHERE match_id=?1 ORDER BY slot",
            )?
            .query_map([match_id], |r| {
                Ok(Participant {
                    player_id: r.get(0)?,
                    account_id: r.get(1)?,
                    slot: r.get(2)?,
                    outcome: r.get(3)?,
                    exit: r.get(4)?,
                })
            })?
            .collect::<std::result::Result<_, _>>()?;
        let mut players = Vec::with_capacity(participants.len());
        for p in &participants {
            let outcome: MatchOutcome = parse(p.outcome.as_deref())?.ok_or(Error::Schema)?;
            let exit: MatchExit = parse(p.exit.as_deref())?.ok_or(Error::Schema)?;
            let reward = if cancelled {
                RewardAmount::default()
            } else {
                policy.reward(outcome, exit)
            };
            players.push(PlayerReward {
                player_id: p.player_id,
                account_id: p.account_id,
                slot: p.slot,
                outcome,
                exit,
                exp_delta: amount(reward.exp)?,
                lucci_delta: amount(reward.lucci)?,
            });
        }
        let hash = request_hash(&json!({
            "v": 1,
            "op": "match_reward",
            "match_id": match_id,
            "policy": policy,
            "rewards": players
                .iter()
                .map(|p| json!([p.player_id, p.exp_delta, p.lucci_delta]))
                .collect::<Vec<_>>(),
        }));
        let key = format!("match:{match_id}");
        let existing: Vec<(i64, String, i64, i64)> = tx
            .prepare(
                "SELECT player_id,request_hash,exp_delta,lucci_delta FROM ledger_entries
                 WHERE kind='match_reward' AND match_id=?1 ORDER BY player_id",
            )?
            .query_map([match_id], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
            })?
            .collect::<std::result::Result<_, _>>()?;
        if !existing.is_empty() {
            let mut wanted: Vec<_> = players
                .iter()
                .map(|p| (p.player_id, hash.clone(), p.exp_delta, p.lucci_delta))
                .collect();
            wanted.sort();
            return if existing == wanted {
                Ok(MatchRewardReceipt {
                    match_id,
                    settled: true,
                    replayed: true,
                    request_hash: Some(hash),
                    players,
                })
            } else {
                Err(Error::Conflict)
            };
        }
        if cancelled || !policy.enabled {
            // Zero receipt only: no ledger row, profile, level or version change.
            return Ok(MatchRewardReceipt {
                match_id,
                settled: false,
                replayed: false,
                request_hash: None,
                players,
            });
        }
        for p in &players {
            Self::insert_ledger(
                tx,
                p.player_id,
                "match_reward",
                &key,
                &hash,
                (p.exp_delta, p.lucci_delta),
                Some(match_id),
                None,
                now,
            )?;
            if p.exp_delta == 0 && p.lucci_delta == 0 {
                continue;
            }
            let profile = Self::read_profile(tx, p.player_id)?;
            let exp = profile
                .exp
                .checked_add(p.exp_delta)
                .ok_or(Error::Exhausted)?;
            let lucci = profile
                .lucci
                .checked_add(p.lucci_delta)
                .ok_or(Error::Exhausted)?;
            Self::write_profile(
                tx,
                &PlayerProfile {
                    exp,
                    lucci,
                    level: policy.level_for(exp as u64),
                    ..profile
                },
            )?;
        }
        Ok(MatchRewardReceipt {
            match_id,
            settled: true,
            replayed: false,
            request_hash: Some(hash),
            players,
        })
    }

    /// Audited operator Lucci grant (positive) or correction (negative). A
    /// correction can never make the balance negative.
    pub fn admin_grant_lucci(
        &self,
        admin: &Admin<'_>,
        player_id: i64,
        lucci_delta: i64,
        key: &str,
        now: impl Into<Option<i64>>,
    ) -> Result<AdminGrantReceipt> {
        admin.validate()?;
        idempotency_key(key)?;
        if lucci_delta == 0 || lucci_delta.unsigned_abs() > ECONOMY_AMOUNT_MAX {
            return Err(Error::InvalidInput);
        }
        let hash = request_hash(&json!({
            "v": 1, "op": "admin_grant_lucci", "player_id": player_id, "lucci_delta": lucci_delta,
        }));
        let request = hex(&self.random::<32>()?);
        let now = match now.into() {
            Some(now) => now,
            None => utc()?,
        };
        self.economy_tx(now, |tx| {
            let account = Self::granted_player(tx, player_id)?;
            if let Some(old) = Self::find_ledger(tx, player_id, "admin_grant", key)? {
                if old.request_hash != hash {
                    return Err(Error::Conflict);
                }
                return Ok(AdminGrantReceipt {
                    ledger_id: old.id,
                    player_id,
                    lucci_delta: old.lucci_delta,
                    sku_id: None,
                    replayed: true,
                    profile: Self::read_profile(tx, player_id)?,
                    item: None,
                });
            }
            let profile = Self::read_profile(tx, player_id)?;
            let lucci = profile
                .lucci
                .checked_add(lucci_delta)
                .ok_or(Error::Exhausted)?;
            if lucci < 0 {
                return Err(Error::InsufficientFunds);
            }
            let ledger_id = Self::insert_ledger(
                tx,
                player_id,
                "admin_grant",
                key,
                &hash,
                (0, lucci_delta),
                None,
                None,
                now,
            )?;
            let profile = Self::write_profile(tx, &PlayerProfile { lucci, ..profile })?;
            Self::economy_audit(
                tx,
                &request,
                admin,
                "admin_grant_lucci",
                account,
                player_id,
                now,
            )?;
            Ok(AdminGrantReceipt {
                ledger_id,
                player_id,
                lucci_delta,
                sku_id: None,
                replayed: false,
                profile,
                item: None,
            })
        })
    }

    /// Operator targets: any player whose account is not deleted.
    pub(super) fn granted_player(db: &Connection, player: i64) -> Result<i64> {
        let account = Self::player_exists(db, player)?;
        let deleted: bool = db.query_row(
            "SELECT login GLOB '~deleted_*' FROM accounts WHERE id=?1",
            [account],
            |r| r.get(0),
        )?;
        if deleted {
            return Err(Error::InvalidInput);
        }
        Ok(account)
    }
    /// Success audit in the mutation transaction, targeting the player.
    pub(super) fn economy_audit(
        tx: &Transaction<'_>,
        request: &str,
        admin: &Admin<'_>,
        op: &str,
        account: i64,
        player: i64,
        now: i64,
    ) -> Result<()> {
        tx.execute(
            "INSERT INTO admin_audit(request_id,actor,operation,account_id,reason,at_utc,details,target_kind,target_id)
             VALUES(?1,?2,?3,?4,?5,?6,'{}','player',?7)",
            params![request, admin.actor, op, account, admin.reason, now, player.to_string()],
        )?;
        Ok(())
    }
}
