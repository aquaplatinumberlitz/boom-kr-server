//! Passwordless trial identities use the same player/session/journal ownership
//! as registered accounts. No shared guest identity or reusable password exists.
use super::*;

const ACTIVE_LIMIT: i64 = 64;
// Satisfies the existing storage size constraint, but is deliberately not a PHC.
// Both credential login paths exclude the explicit guest_accounts relation.
const NO_PASSWORD: &str = "!passwordless-trial-principal-with-no-password-or-recovery-secret!";

struct NewGuest {
    login: String,
    name: String,
    token: Token,
    audit: String,
}
fn second_guest_name(base: &str) -> Result<String> {
    for suffix in ["_2P", "_P2", "_2"] {
        let mut prefix = base;
        loop {
            let candidate = format!("{prefix}{suffix}");
            if candidate != base && display_name(&candidate).is_ok() {
                return Ok(candidate);
            }
            let Some((index, _)) = prefix.char_indices().next_back() else {
                break;
            };
            prefix = &prefix[..index];
        }
    }
    Err(Error::InvalidInput)
}

impl Store {
    pub fn trial_local(
        &mut self,
        count: u8,
        ttl: i64,
        idle: Option<i64>,
        now: i64,
    ) -> Result<Vec<Grant>> {
        self.trial_local_clock(count, None, ttl, idle, &mut || Ok(now))
    }
    pub(crate) fn trial_local_current(
        &mut self,
        count: u8,
        name: Option<&str>,
        ttl: i64,
        idle: Option<i64>,
    ) -> Result<Vec<Grant>> {
        self.trial_local_clock(count, name, ttl, idle, &mut || {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .ok()
                .and_then(|time| i64::try_from(time.as_secs()).ok())
                .ok_or(Error::ClockRollback)
        })
    }
    fn trial_local_clock(
        &mut self,
        count: u8,
        name: Option<&str>,
        ttl: i64,
        idle: Option<i64>,
        clock: &mut impl FnMut() -> Result<i64>,
    ) -> Result<Vec<Grant>> {
        if !(1..=2).contains(&count)
            || !(1..=TTL_MAX).contains(&ttl)
            || idle.is_some_and(|idle| !(1..=TTL_MAX).contains(&idle))
        {
            return Err(Error::InvalidInput);
        }
        let named = match name {
            Some(base) => {
                display_name(base)?;
                let mut names = vec![base.to_owned()];
                if count == 2 {
                    names.push(second_guest_name(base)?);
                }
                Some(names)
            }
            None => None,
        };
        let guests = (0..usize::from(count))
            .map(|index| {
                Ok(NewGuest {
                    login: format!("g_{}", hex(&self.random::<12>()?)),
                    name: match &named {
                        Some(names) => names[index].clone(),
                        None => format!("Guest{}", hex(&self.random::<3>()?)),
                    },
                    token: Token(Zeroizing::new(hex(&self.random::<32>()?))),
                    audit: hex(&self.random::<32>()?),
                })
            })
            .collect::<Result<Vec<_>>>()?;
        self.at_clock(clock, |tx, now| {
            let expires = now.checked_add(ttl).ok_or(Error::InvalidInput)?;
            let epoch: i64 = tx.query_row("SELECT epoch FROM runtime_state WHERE id=1", [], |r| r.get(0))?;
            if epoch == 0 { return Err(Error::InstanceRequired); }
            tx.execute("UPDATE sessions SET revoked_at=?1 WHERE revoked_at IS NULL AND (expires_at<=?1 OR (idle_timeout_seconds IS NOT NULL AND last_seen_at<=?1-idle_timeout_seconds))", [now])?;
            let active: i64 = tx.query_row("SELECT count(*) FROM sessions s JOIN guest_accounts g ON g.account_id=s.account_id WHERE s.revoked_at IS NULL", [], |r| r.get(0))?;
            if active + i64::from(count) > ACTIVE_LIMIT { return Err(Error::Busy); }
            let mut grants = Vec::with_capacity(count as usize);
            for guest in guests {
                let wire: i64 = tx.query_row("SELECT next_wire_id FROM runtime_state WHERE id=1", [], |r| r.get(0))?;
                if wire > i64::from(u32::MAX) { return Err(Error::Exhausted); }
                tx.execute("INSERT INTO accounts(login,password_phc,created_at) VALUES(?1,?2,?3)", params![guest.login,NO_PASSWORD,now])?;
                let account = tx.last_insert_rowid();
                tx.execute("INSERT INTO players(account_id,wire_id,display_name) VALUES(?1,?2,?3)", params![account,wire,guest.name])?;
                tx.execute("UPDATE runtime_state SET next_wire_id=next_wire_id+1 WHERE id=1", [])?;
                tx.execute("INSERT INTO guest_accounts(account_id,boot_epoch,expires_at) VALUES(?1,?2,?3)", params![account,epoch,expires])?;
                tx.execute("INSERT INTO sessions(account_id,token_hash,boot_epoch,credential_version,issued_at,expires_at,idle_timeout_seconds,last_seen_at) VALUES(?1,?2,?3,1,?4,?5,?6,?7)", params![account,guest.token.digest(),epoch,now,expires,idle,idle.map(|_|now)])?;
                Self::audit(tx, &guest.audit, &Admin {actor:"account-worker",reason:"passwordless native trial admission"}, "guest_login", Some(account), now)?;
                grants.push(Grant {player:Self::player(tx,account)?, token:guest.token, expires_at:expires});
            }
            Ok(grants)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounts::tests::fresh;
    fn admin() -> Admin<'static> {
        Admin {
            actor: "test",
            reason: "guest ownership controls",
        }
    }
    #[test]
    fn schema3_upgrade_preserves_registered_identity_and_existing_session() {
        let _serial = TEST_SERIAL.lock().unwrap();
        let (temp, mut store, player) = fresh();
        let grant = store
            .login("Alice", b"test-only-password-one", 60, 101)
            .unwrap();
        store.db.execute_batch("DROP INDEX admin_audit_target; ALTER TABLE admin_audit DROP COLUMN result; ALTER TABLE admin_audit DROP COLUMN target_id; ALTER TABLE admin_audit DROP COLUMN target_kind; DROP TABLE account_recovery; DROP INDEX guest_account_expiry; DROP TABLE guest_accounts; DELETE FROM schema_migrations WHERE version>=4; PRAGMA user_version=3;").unwrap();
        drop(store);
        let mut reopened = temp.store();
        assert_eq!(
            reopened
                .authorize(&grant.token, player.player_id, 102)
                .unwrap(),
            player
        );
        assert_eq!(reopened.lookup("Alice").unwrap(), player);
        assert_eq!(
            reopened
                .db
                .pragma_query_value::<i32, _>(None, "user_version", |r| r.get(0))
                .unwrap(),
            super::SCHEMA_VERSION
        );
        let guests = reopened.trial_local(2, 30, None, 103).unwrap();
        assert!(
            guests
                .iter()
                .all(|g| g.player.player_id != player.player_id)
        );
    }
    #[test]
    fn trial_pair_is_distinct_passwordless_expiring_and_never_resumes_old_identity() {
        let _serial = TEST_SERIAL.lock().unwrap();
        let (_temp, mut store, registered) = fresh();
        let pair = store.trial_local(2, 10, Some(5), 101).unwrap();
        assert_ne!(pair[0].player, pair[1].player);
        assert!(
            pair.iter()
                .all(|g| g.player.account_id != registered.account_id)
        );
        let login: String = store
            .db
            .query_row(
                "SELECT login FROM accounts WHERE id=?1",
                [pair[0].player.account_id],
                |r| r.get(0),
            )
            .unwrap();
        store
            .reset_password(
                &admin(),
                pair[0].player.account_id,
                b"known-test-password",
                102,
            )
            .unwrap();
        assert!(matches!(
            store.login(&login, b"known-test-password", 60, 103),
            Err(Error::InvalidCredentials)
        ));
        assert!(matches!(
            store.login_local(&[(&login, b"known-test-password")], 60, None, 103),
            Err(Error::InvalidCredentials)
        ));
        store
            .authorize(&pair[1].token, pair[1].player.player_id, 103)
            .unwrap();
        assert!(
            store
                .inspect_session(&pair[1].token, pair[1].player.player_id, 108)
                .is_err()
        );
        store.end_local_session(&pair, 108).unwrap();
        let next = store.trial_local(1, 10, None, 109).unwrap();
        assert!(
            pair.iter()
                .all(|g| g.player.player_id != next[0].player.player_id)
        );
        assert!(
            store
                .authorize(&pair[0].token, pair[0].player.player_id, 109)
                .is_err()
        );
        store.begin_instance(&admin(), 110).unwrap();
        assert!(
            store
                .authorize(&next[0].token, next[0].player.player_id, 110)
                .is_err()
        );
    }
    #[test]
    fn trial_pair_insert_failure_and_capacity_never_publish_a_half_roster() {
        let _serial = TEST_SERIAL.lock().unwrap();
        let (_temp, mut store, _) = fresh();
        store.db.execute_batch("CREATE TEMP TRIGGER fail_guest BEFORE INSERT ON guest_accounts WHEN (SELECT count(*) FROM guest_accounts)>0 BEGIN SELECT RAISE(ABORT,'test'); END;").unwrap();
        assert!(store.trial_local(2, 60, None, 101).is_err());
        assert_eq!(
            store
                .db
                .query_row("SELECT count(*) FROM guest_accounts", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            0
        );
        store.db.execute_batch("DROP TRIGGER fail_guest").unwrap();
        let mut grants = Vec::new();
        for _ in 0..ACTIVE_LIMIT / 2 {
            grants.extend(store.trial_local(2, 60, None, 102).unwrap());
        }
        assert!(matches!(
            store.trial_local(1, 60, None, 102),
            Err(Error::Busy)
        ));
        store.end_local_session(&grants[..2], 103).unwrap();
        assert_eq!(store.trial_local(2, 60, None, 104).unwrap().len(), 2);
        assert!(store.trial_local(3, 60, None, 104).is_err());
    }
    #[test]
    fn named_trial_supports_one_and_two_local_players_within_name_bounds() {
        let _serial = TEST_SERIAL.lock().unwrap();
        let (_temp, mut store, _) = fresh();
        let now = || {
            i64::try_from(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            )
            .unwrap()
        };
        let one = store
            .trial_local_current(1, Some("Việt01"), 60, None)
            .unwrap();
        assert_eq!(one.len(), 1);
        assert_eq!(one[0].player.display_name, "Việt01");
        store.end_local_session(&one, now()).unwrap();

        let seven_e = "ệ".repeat(7);
        for (base, expected_second) in [
            ("Việt01", "Việt01_2P"),
            ("abcdefghijklmno", "abcdefghijkl_2P"),
            (seven_e.as_str(), "ệệệệệệ_2P"),
            ("abcdefghijkl_2P", "abcdefghijk_2P"),
        ] {
            let pair = store.trial_local_current(2, Some(base), 60, None).unwrap();
            assert_eq!(pair.len(), 2);
            assert_eq!(pair[0].player.display_name, base);
            assert_eq!(pair[1].player.display_name, expected_second);
            assert_ne!(pair[0].player.account_id, pair[1].player.account_id);
            assert_ne!(pair[0].player.player_id, pair[1].player.player_id);
            assert_ne!(pair[0].player.wire_id, pair[1].player.wire_id);
            for grant in &pair {
                assert!(
                    kr_wire::kr00967::text949x::encode_name(&grant.player.display_name).is_ok()
                );
            }
            store.end_local_session(&pair, now()).unwrap();
        }

        assert!(matches!(
            store.trial_local_current(3, Some("Việt01"), 60, None),
            Err(Error::InvalidInput)
        ));
    }
}
