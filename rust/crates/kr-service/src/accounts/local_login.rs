//! One native client may authenticate two distinct DB principals. Publication
//! is atomic; this module grants no transport or room membership by itself.
use super::*;

struct Verified {
    account: i64,
    phc: String,
    version: i64,
    token: Token,
}

impl Store {
    /// Bounded local roster login. The supplied time API is for trusted tests;
    /// live callers use execution-time UTC after acquiring writer transactions.
    pub fn login_local(
        &mut self,
        credentials: &[(&str, &[u8])],
        ttl: i64,
        idle: Option<i64>,
        now: i64,
    ) -> Result<Vec<Grant>> {
        self.login_local_clock(credentials, ttl, idle, &mut || Ok(now))
    }

    pub(crate) fn login_local_current(
        &mut self,
        credentials: &[(&str, &[u8])],
        ttl: i64,
        idle: Option<i64>,
    ) -> Result<Vec<Grant>> {
        self.login_local_clock(credentials, ttl, idle, &mut || {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .ok()
                .and_then(|time| i64::try_from(time.as_secs()).ok())
                .ok_or(Error::ClockRollback)
        })
    }

    fn login_local_clock(
        &mut self,
        credentials: &[(&str, &[u8])],
        ttl: i64,
        idle: Option<i64>,
        clock: &mut impl FnMut() -> Result<i64>,
    ) -> Result<Vec<Grant>> {
        if !(1..=2).contains(&credentials.len())
            || !(1..=TTL_MAX).contains(&ttl)
            || idle.is_some_and(|idle| !(1..=TTL_MAX).contains(&idle))
        {
            return Err(Error::InvalidInput);
        }
        let mut names = Vec::with_capacity(credentials.len());
        for (login, password) in credentials {
            let name = login_name(login)?;
            password_bounds(password)?;
            if names.contains(&name) {
                return Err(Error::InvalidInput);
            }
            names.push(name);
        }
        self.observe_clock(clock)?;
        let mut verified = Vec::with_capacity(credentials.len());
        let mut invalid = false;
        // Perform bounded KDF work for every syntactically valid principal,
        // including unknown users. No first player's session is issued yet.
        for (name, (_, password)) in names.iter().zip(credentials) {
            let row: Option<(i64, String, i64, bool)> = self.db.query_row(
                "SELECT id,password_phc,credential_version,disabled FROM accounts WHERE login=?1",
                [name], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            ).optional()?;
            let valid = verify(password, row.as_ref().map(|r| r.1.as_str()))?;
            match row {
                Some((account, phc, version, false)) if valid => {
                    let raw = Zeroizing::new(self.random::<32>()?);
                    verified.push(Verified {
                        account,
                        phc,
                        version,
                        token: Token(Zeroizing::new(hex(raw.as_ref()))),
                    });
                }
                _ => invalid = true,
            }
        }
        if invalid {
            return Err(Error::InvalidCredentials);
        }
        self.publish_local(verified, ttl, idle, clock)
    }

    fn publish_local(
        &mut self,
        verified: Vec<Verified>,
        ttl: i64,
        idle: Option<i64>,
        clock: &mut impl FnMut() -> Result<i64>,
    ) -> Result<Vec<Grant>> {
        self.at_clock(clock, |tx, now| {
            let expires = now.checked_add(ttl).ok_or(Error::InvalidInput)?;
            let epoch: i64 = tx.query_row("SELECT epoch FROM runtime_state WHERE id=1", [], |r| r.get(0))?;
            if epoch == 0 {
                return Err(Error::InstanceRequired);
            }
            tx.execute("UPDATE sessions SET revoked_at=?1 WHERE revoked_at IS NULL AND (expires_at<=?1 OR (idle_timeout_seconds IS NOT NULL AND last_seen_at<=?1-idle_timeout_seconds))", [now])?;
            let mut grants = Vec::with_capacity(verified.len());
            for row in verified {
                // Reset/disable on another DB connection must win over KDF.
                let current: (String, i64, bool) = tx.query_row(
                    "SELECT password_phc,credential_version,disabled FROM accounts WHERE id=?1",
                    [row.account], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
                )?;
                if current != (row.phc, row.version, false) {
                    return Err(Error::InvalidCredentials);
                }
                let active: i64 = tx.query_row("SELECT count(*) FROM sessions WHERE account_id=?1 AND revoked_at IS NULL", [row.account], |r| r.get(0))?;
                if active != 0 {
                    return Err(Error::ActiveSession);
                }
                tx.execute("INSERT INTO sessions(account_id,token_hash,boot_epoch,credential_version,issued_at,expires_at,idle_timeout_seconds,last_seen_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8)", params![row.account,row.token.digest(),epoch,row.version,now,expires,idle,idle.map(|_|now)])?;
                grants.push(Grant {
                    player: Self::player(tx, row.account)?, token: row.token, expires_at: expires,
                });
            }
            Ok(grants)
        })
    }

    /// Compensate a whole unaccepted roster in one writer transaction. Exact
    /// tokens fence this cleanup from a later login of either account.
    pub(crate) fn release_local_undelivered(&mut self, grants: &[Grant], now: i64) -> Result<()> {
        self.retire_local(
            grants,
            now,
            "abandoned_login",
            "local roster login result not accepted",
        )
    }

    pub(crate) fn end_local_session(&mut self, grants: &[Grant], now: i64) -> Result<()> {
        self.retire_local(
            grants,
            now,
            "local_disconnect",
            "owned local transport ended",
        )
    }

    fn retire_local(
        &mut self,
        grants: &[Grant],
        now: i64,
        operation: &str,
        reason: &str,
    ) -> Result<()> {
        if !(1..=2).contains(&grants.len()) || now < 0 {
            return Err(Error::InvalidInput);
        }
        let requests = grants
            .iter()
            .map(|_| self.random::<32>().map(|raw| hex(&raw)))
            .collect::<Result<Vec<_>>>()?;
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let last: i64 = tx.query_row("SELECT last_utc FROM runtime_state WHERE id=1", [], |r| {
            r.get(0)
        })?;
        let now = now.max(last);
        Self::clock(&tx, now)?;
        for (grant, request) in grants.iter().zip(requests) {
            let changed = tx.execute("UPDATE sessions SET revoked_at=?1 WHERE token_hash=?2 AND account_id=?3 AND revoked_at IS NULL", params![now,grant.token.digest(),grant.player.account_id])?;
            if changed != 0 {
                Self::audit(
                    &tx,
                    &request,
                    &Admin {
                        actor: "account-worker",
                        reason,
                    },
                    operation,
                    Some(grant.player.account_id),
                    now,
                )?;
            }
        }
        tx.commit()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounts::tests::fresh;
    const PASSWORD: &[u8] = b"test-only-password-one";
    const OTHER: &[u8] = b"test-only-password-two";

    fn admin() -> Admin<'static> {
        Admin {
            actor: "test",
            reason: "atomic local roster",
        }
    }
    fn active(store: &Store) -> i64 {
        store
            .db
            .query_row(
                "SELECT count(*) FROM sessions WHERE revoked_at IS NULL",
                [],
                |r| r.get(0),
            )
            .unwrap()
    }

    #[test]
    fn local_pair_failure_never_leaves_first_principal_logged_in() {
        let _serial = TEST_SERIAL.lock().unwrap();
        let (_temp, mut store, alice) = fresh();
        store
            .create_account(&admin(), "Bobby", "Bobby", PASSWORD, 100)
            .unwrap();
        let bad = [("Alice", PASSWORD), ("Bobby", OTHER)];
        assert!(matches!(
            store.login_local(&bad, 60, None, 101),
            Err(Error::InvalidCredentials)
        ));
        assert_eq!(active(&store), 0);
        let unknown = [("Alice", PASSWORD), ("Missing", PASSWORD)];
        assert!(matches!(
            store.login_local(&unknown, 60, None, 102),
            Err(Error::InvalidCredentials)
        ));
        assert_eq!(active(&store), 0);
        assert!(matches!(
            store.login_local(&[("Alice", PASSWORD), ("ALICE", PASSWORD)], 60, None, 103),
            Err(Error::InvalidInput)
        ));
        let existing = store.login("Bobby", PASSWORD, 60, 103).unwrap();
        assert!(matches!(
            store.login_local(&[("Alice", PASSWORD), ("Bobby", PASSWORD)], 60, None, 104),
            Err(Error::ActiveSession)
        ));
        assert_eq!(active(&store), 1);
        store
            .authorize(&existing.token, existing.player.player_id, 104)
            .unwrap();
        let first = store.login("Alice", PASSWORD, 60, 105).unwrap();
        assert_eq!(first.player, alice);
    }

    #[test]
    fn local_pair_publication_rolls_back_on_second_write_and_credential_reset() {
        let _serial = TEST_SERIAL.lock().unwrap();
        let (temp, mut store, _alice) = fresh();
        let bob = store
            .create_account(&admin(), "Bobby", "Bobby", PASSWORD, 100)
            .unwrap();
        store.db.execute_batch(&format!("CREATE TRIGGER deny_second BEFORE INSERT ON sessions WHEN NEW.account_id={} BEGIN SELECT RAISE(ABORT,'injected test failure'); END",bob.account_id)).unwrap();
        let pair = [("Alice", PASSWORD), ("Bobby", PASSWORD)];
        assert!(store.login_local(&pair, 60, None, 101).is_err());
        assert_eq!(active(&store), 0);
        store.db.execute_batch("DROP TRIGGER deny_second").unwrap();
        // Snapshot the actually verified DB rows, then reset a credential on
        // an independent connection before the publication transaction.
        let verified =
            pair.iter()
                .map(|(name, password)| {
                    let (account,phc,version): (i64,String,i64) = store.db.query_row(
                "SELECT id,password_phc,credential_version FROM accounts WHERE login=?1",
                [name.to_ascii_lowercase()], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?))).unwrap();
                    assert!(verify(password, Some(&phc)).unwrap());
                    Verified {
                        account,
                        phc,
                        version,
                        token: Token(Zeroizing::new(hex(&store.random::<32>().unwrap()))),
                    }
                })
                .collect();
        temp.store()
            .reset_password(&admin(), bob.account_id, OTHER, 102)
            .unwrap();
        let result = store.publish_local(verified, 60, None, &mut || Ok(102));
        assert!(matches!(result, Err(Error::InvalidCredentials)));
        assert_eq!(active(&store), 0);
    }

    #[test]
    fn abandoned_roster_cleanup_cannot_revoke_replacement_and_expiry_is_per_identity() {
        let _serial = TEST_SERIAL.lock().unwrap();
        let (_temp, mut store, _alice) = fresh();
        store
            .create_account(&admin(), "Bobby", "Bobby", PASSWORD, 100)
            .unwrap();
        let grants = store
            .login_local(
                &[("Alice", PASSWORD), ("Bobby", PASSWORD)],
                60,
                Some(10),
                101,
            )
            .unwrap();
        assert_eq!(active(&store), 2);
        assert_eq!(grants[0].expires_at, 161);
        store
            .logout(&grants[0].token, grants[0].player.player_id, 102)
            .unwrap();
        let replacement = store.login("Alice", PASSWORD, 60, 103).unwrap();
        store.release_local_undelivered(&grants, 101).unwrap();
        assert_eq!(active(&store), 1);
        store
            .authorize(&replacement.token, replacement.player.player_id, 104)
            .unwrap();
        assert!(matches!(
            store.authorize(&grants[1].token, grants[1].player.player_id, 104),
            Err(Error::InvalidSession)
        ));
        // A different pair can be issued only once the current account's own
        // session is retired; renewing one idle TTL never renews the other.
        store
            .logout(&replacement.token, replacement.player.player_id, 105)
            .unwrap();
        let next = store
            .login_local(
                &[("Alice", PASSWORD), ("Bobby", PASSWORD)],
                60,
                Some(10),
                106,
            )
            .unwrap();
        store
            .authorize(&next[0].token, next[0].player.player_id, 115)
            .unwrap();
        assert!(matches!(
            store.inspect_session(&next[1].token, next[1].player.player_id, 116),
            Err(Error::InvalidSession)
        ));
        store
            .inspect_session(&next[0].token, next[0].player.player_id, 116)
            .unwrap();
    }
}
