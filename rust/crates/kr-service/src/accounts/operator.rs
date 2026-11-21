use super::*;
use serde::Deserialize;

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuditFilter {
    pub actor: Option<String>,
    pub operation: Option<String>,
    pub target_kind: Option<String>,
    pub target_id: Option<String>,
    pub result: Option<String>,
    pub since_utc: Option<i64>,
    pub before_id: Option<i64>,
    pub limit: Option<u8>,
}

#[derive(Debug, Serialize)]
pub struct AuditRow {
    pub id: i64,
    pub request_id: String,
    pub actor: String,
    pub operation: String,
    pub account_id: Option<i64>,
    pub target_kind: Option<String>,
    pub target_id: Option<String>,
    pub reason: String,
    pub at_utc: i64,
    pub result: String,
}

pub fn utc() -> Result<i64> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .and_then(|time| i64::try_from(time.as_secs()).ok())
        .ok_or(Error::ClockRollback)
}

impl Store {
    /// Audit a local operator query, denial or intent. Do not include secrets
    /// in the fields. Account mutations audit successes in their own transaction.
    pub fn operator_event(
        &mut self,
        admin: &Admin<'_>,
        op: &str,
        target: (&str, &str),
        result: &str,
    ) -> Result<String> {
        admin.validate()?;
        let request = hex(&self.random::<32>()?);
        let now = utc()?;
        self.db.execute(
            "INSERT INTO admin_audit(request_id,actor,operation,reason,at_utc,details,target_kind,target_id,result)
             VALUES(?1,?2,?3,?4,?5,'{}',?6,?7,?8)",
            params![request,admin.actor,op,admin.reason,now,target.0,target.1,result],
        )?;
        Ok(request)
    }
    pub(crate) fn finish_operator_event(&mut self, request: &str, result: &str) -> Result<()> {
        if self.db.execute(
            "UPDATE admin_audit SET result=?1 WHERE request_id=?2 AND result='pending'",
            params![result, request],
        )? != 1
        {
            return Err(Error::Conflict);
        }
        Ok(())
    }
    pub fn operator_audit(&self, f: AuditFilter) -> Result<Vec<AuditRow>> {
        let limit = f.limit.unwrap_or(50);
        if !(1..=100).contains(&limit) {
            return Err(Error::InvalidInput);
        }
        let mut query = self.db.prepare(
            "SELECT id,request_id,actor,operation,account_id,target_kind,target_id,reason,at_utc,result
             FROM admin_audit
             WHERE (?1 IS NULL OR actor=?1) AND (?2 IS NULL OR operation=?2)
               AND (?3 IS NULL OR target_kind=?3) AND (?4 IS NULL OR target_id=?4)
               AND (?5 IS NULL OR result=?5) AND (?6 IS NULL OR at_utc>=?6)
               AND (?7 IS NULL OR id<?7)
             ORDER BY id DESC LIMIT ?8",
        )?;
        Ok(query
            .query_map(
                params![
                    f.actor,
                    f.operation,
                    f.target_kind,
                    f.target_id,
                    f.result,
                    f.since_utc,
                    f.before_id,
                    limit
                ],
                |r| {
                    Ok(AuditRow {
                        id: r.get(0)?,
                        request_id: r.get(1)?,
                        actor: r.get(2)?,
                        operation: r.get(3)?,
                        account_id: r.get(4)?,
                        target_kind: r.get(5)?,
                        target_id: r.get(6)?,
                        reason: r.get(7)?,
                        at_utc: r.get(8)?,
                        result: r.get(9)?,
                    })
                },
            )?
            .collect::<std::result::Result<_, _>>()?)
    }

    pub fn reset_recovery(
        &mut self,
        admin: &Admin<'_>,
        account: i64,
        now: impl Into<Option<i64>>,
    ) -> Result<Zeroizing<String>> {
        admin.validate()?;
        let request = hex(&self.random::<32>()?);
        let code = self.new_recovery_code()?;
        let digest = Sha256::digest(code.as_bytes());
        self.at_admin(now, |tx, now| {
            Self::fence_registered_account(tx, account, now)?;
            tx.execute(
                "INSERT INTO account_recovery VALUES(?1,?2)
                 ON CONFLICT(account_id) DO UPDATE SET digest=excluded.digest",
                params![account, &digest[..]],
            )?;
            Self::audit(tx, &request, admin, "reset_recovery", Some(account), now)
        })?;
        Ok(code)
    }

    pub fn delete_account(
        &mut self,
        admin: &Admin<'_>,
        account: i64,
        now: impl Into<Option<i64>>,
    ) -> Result<()> {
        admin.validate()?;
        let request = hex(&self.random::<32>()?);
        let random = Zeroizing::new(self.random::<32>()?);
        let phc = password_hash(random.as_ref(), &self.random::<16>()?)?;
        self.at_admin(now, |tx, now| {
            Self::fence_registered_account(tx, account, now)?;
            tx.execute(
                "UPDATE accounts SET password_phc=?1 WHERE id=?2",
                params![phc, account],
            )?;
            Self::tombstone(tx, account)?;
            Self::audit(tx, &request, admin, "delete_account", Some(account), now)
        })
    }

    pub(super) fn fence_registered_account(
        tx: &Transaction<'_>,
        account: i64,
        now: i64,
    ) -> Result<()> {
        if tx.execute(
            "UPDATE accounts SET credential_version=credential_version+1
             WHERE id=?1 AND login NOT GLOB '~deleted_*'
               AND NOT EXISTS(SELECT 1 FROM guest_accounts WHERE account_id=?1)",
            [account],
        )? != 1
        {
            return Err(Error::InvalidInput);
        }
        tx.execute(
            "UPDATE sessions SET revoked_at=?1 WHERE account_id=?2 AND revoked_at IS NULL",
            params![now, account],
        )?;
        Ok(())
    }

    pub(super) fn tombstone(tx: &Transaction<'_>, account: i64) -> Result<()> {
        // Keep FK identities for match and audit history; the old login can be
        // registered again with entirely new account/player IDs.
        tx.execute(
            "UPDATE accounts SET login=?1,disabled=1 WHERE id=?2",
            params![format!("~deleted_{account}"), account],
        )?;
        tx.execute("UPDATE players SET display_name='Deleted',profile_version=profile_version+1 WHERE account_id=?1", [account])?;
        tx.execute("DELETE FROM player_preferences WHERE player_id=(SELECT id FROM players WHERE account_id=?1)", [account])?;
        tx.execute(
            "DELETE FROM account_recovery WHERE account_id=?1",
            [account],
        )?;
        Ok(())
    }
}
