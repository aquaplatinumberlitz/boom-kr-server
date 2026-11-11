use super::TEST_SERIAL as SERIAL;
use super::*;
use std::sync::atomic::AtomicU64;
static NEXT: AtomicU64 = AtomicU64::new(0);
const PASSWORD: &[u8] = b"test-only-password-one";
const OTHER: &[u8] = b"test-only-password-two";
fn admin() -> Admin<'static> {
    Admin {
        actor: "test-admin",
        reason: "isolated backend acceptance",
    }
}
struct Temp(std::path::PathBuf);
impl Temp {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "kr-store-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        std::fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
        Self(path)
    }
    fn db(&self) -> std::path::PathBuf {
        self.0.join("accounts.sqlite")
    }
    fn store(&self) -> Store {
        Store::open(&self.db()).unwrap()
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn fresh() -> (Temp, Store, Player) {
    let temp = Temp::new();
    let mut s = temp.store();
    s.begin_instance(&admin(), 100).unwrap();
    let p = s
        .create_account(&admin(), "Alice", "Alice", PASSWORD, 100)
        .unwrap();
    (temp, s, p)
}
#[test]
fn reopen_preserves_identity_and_session_until_explicit_instance_restart() {
    let _serial = SERIAL.lock().unwrap();
    let (temp, mut s, p) = fresh();
    let g = s.login("ALICE", PASSWORD, 60, 101).unwrap();
    drop(s);
    let mut s = temp.store();
    assert_eq!(s.lookup("alice").unwrap(), p);
    assert_eq!(s.authorize(&g.token, p.player_id, 102).unwrap(), p);
    assert_eq!(s.begin_instance(&admin(), 103), Ok(2));
    assert_eq!(
        s.authorize(&g.token, p.player_id, 104),
        Err(Error::InvalidSession)
    );
    let next = s.login("alice", PASSWORD, 60, 104).unwrap();
    assert_ne!(g.token.expose_secret(), next.token.expose_secret());
    assert_eq!(next.player, p);
}
#[test]
fn credentials_owner_and_input_bounds() {
    let _serial = SERIAL.lock().unwrap();
    let (_temp, mut s, p) = fresh();
    let b = s
        .create_account(&admin(), "Bob", "Bobby", OTHER, 100)
        .unwrap();
    assert_ne!(p.wire_id, b.wire_id);
    assert!(matches!(
        s.create_account(&admin(), "ALICE", "NewName", OTHER, 100),
        Err(Error::Conflict)
    ));
    assert!(matches!(
        s.login("alice", OTHER, 60, 101),
        Err(Error::InvalidCredentials)
    ));
    assert!(matches!(
        s.login("nobody", PASSWORD, 60, 101),
        Err(Error::InvalidCredentials)
    ));
    assert!(matches!(
        s.login("alice", PASSWORD, 0, 101),
        Err(Error::InvalidInput)
    ));
    assert!(matches!(
        s.login("alice", PASSWORD, 86_401, 101),
        Err(Error::InvalidInput)
    ));
    assert!(matches!(
        s.create_account(&admin(), "a';sql", "BadName", PASSWORD, 101),
        Err(Error::InvalidInput)
    ));
    assert!(matches!(
        s.create_account(&admin(), "valid", "Bad Name", PASSWORD, 101),
        Err(Error::InvalidInput)
    ));
    assert!(matches!(
        s.create_account(&admin(), "valid", "Valid", b"short", 101),
        Err(Error::InvalidInput)
    ));
    let a = s.login("alice", PASSWORD, 60, 101).unwrap();
    assert_eq!(
        s.authorize(&a.token, b.player_id, 102),
        Err(Error::InvalidSession)
    );
    assert_eq!(
        s.authorize(&Token::parse(&"00".repeat(32)).unwrap(), p.player_id, 102),
        Err(Error::InvalidSession)
    );
    assert!(Token::parse("short").is_err());
    assert!(Token::parse(&"G0".repeat(32)).is_err());
}
#[test]
fn expiry_observation_is_durable_even_on_denial_clock_cannot_revive_token() {
    let _serial = SERIAL.lock().unwrap();
    let (temp, mut s, p) = fresh();
    let g = s.login("alice", PASSWORD, 2, 101).unwrap();
    assert_eq!(s.authorize(&g.token, p.player_id, 102), Ok(p.clone()));
    assert_eq!(
        s.authorize(&g.token, p.player_id, 103),
        Err(Error::InvalidSession)
    );
    drop(s);
    let mut s = temp.store();
    assert_eq!(
        s.authorize(&g.token, p.player_id, 102),
        Err(Error::ClockRollback)
    );
    assert!(s.login("alice", PASSWORD, 60, 103).is_ok());
}
#[test]
fn revoke_disable_enable_and_password_reset() {
    let _serial = SERIAL.lock().unwrap();
    let (_temp, mut s, p) = fresh();
    let first = s.login("alice", PASSWORD, 100, 101).unwrap();
    assert!(matches!(
        s.login("alice", PASSWORD, 100, 101),
        Err(Error::ActiveSession)
    ));
    s.revoke(&admin(), p.account_id, 102).unwrap();
    assert_eq!(
        s.authorize(&first.token, p.player_id, 102),
        Err(Error::InvalidSession)
    );
    let second = s.login("alice", PASSWORD, 100, 102).unwrap();
    s.set_disabled(&admin(), p.account_id, true, 103).unwrap();
    assert_eq!(
        s.authorize(&second.token, p.player_id, 103),
        Err(Error::InvalidSession)
    );
    assert!(matches!(
        s.login("alice", PASSWORD, 100, 103),
        Err(Error::InvalidCredentials)
    ));
    s.set_disabled(&admin(), p.account_id, false, 104).unwrap();
    assert_eq!(
        s.authorize(&second.token, p.player_id, 104),
        Err(Error::InvalidSession)
    );
    let third = s.login("alice", PASSWORD, 100, 104).unwrap();
    s.reset_password(&admin(), p.account_id, OTHER, 105)
        .unwrap();
    assert_eq!(
        s.authorize(&third.token, p.player_id, 105),
        Err(Error::InvalidSession)
    );
    assert!(matches!(
        s.login("alice", PASSWORD, 100, 105),
        Err(Error::InvalidCredentials)
    ));
    assert!(s.login("alice", OTHER, 100, 105).is_ok());
}
#[test]
fn two_real_connections_race_login_only_one_active_and_no_plaintext_at_rest() {
    let _serial = SERIAL.lock().unwrap();
    let (temp, s, _p) = fresh();
    drop(s);
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
    let threads: Vec<_> = (0..2)
        .map(|_| {
            let path = temp.db();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                let mut s = Store::open(&path).unwrap();
                barrier.wait();
                s.login("alice", PASSWORD, 60, 101)
            })
        })
        .collect();
    let results: Vec<_> = threads.into_iter().map(|t| t.join().unwrap()).collect();
    assert_eq!(results.iter().filter(|r| r.is_ok()).count(), 1);
    assert_eq!(
        results
            .iter()
            .filter(|r| matches!(r, Err(Error::ActiveSession)))
            .count(),
        1
    );
    let token = results
        .iter()
        .find_map(|r| r.as_ref().ok())
        .unwrap()
        .token
        .expose_secret()
        .as_bytes();
    let s = temp.store();
    let hashes: Vec<Vec<u8>> =
        s.db.prepare("SELECT token_hash FROM sessions")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .collect::<std::result::Result<_, _>>()
            .unwrap();
    assert_eq!(hashes.len(), 1);
    assert_eq!(hashes[0].len(), 32);
    let phc: String =
        s.db.query_row("SELECT password_phc FROM accounts", [], |r| r.get(0))
            .unwrap();
    assert!(phc.starts_with("$argon2id$v=19$m=19456,t=2,p=1$"));
    for entry in fs::read_dir(&temp.0).unwrap() {
        let data = fs::read(entry.unwrap().path()).unwrap();
        assert!(!data.windows(PASSWORD.len()).any(|w| w == PASSWORD));
        assert!(!data.windows(token.len()).any(|w| w == token));
    }
}
#[test]
fn migration_reopen_future_checksum_unknown_database_and_permissions() {
    let _serial = SERIAL.lock().unwrap();
    let temp = Temp::new();
    drop(temp.store());
    drop(temp.store());
    let db = Connection::open(temp.db()).unwrap();
    db.pragma_update(None, "user_version", 3).unwrap();
    assert!(matches!(Store::open(&temp.db()), Err(Error::Schema)));
    db.pragma_update(None, "user_version", 2).unwrap();
    db.execute("UPDATE schema_migrations SET sha256=?1", ["0".repeat(64)])
        .unwrap();
    assert!(matches!(Store::open(&temp.db()), Err(Error::Schema)));
    drop(db);
    fs::set_permissions(temp.db(), fs::Permissions::from_mode(0o644)).unwrap();
    assert!(matches!(Store::open(&temp.db()), Err(Error::InvalidInput)));
    let other = temp.0.join("other.sqlite");
    let db = Connection::open(&other).unwrap();
    db.execute_batch("CREATE TABLE unrelated(id INTEGER);")
        .unwrap();
    drop(db);
    fs::set_permissions(&other, fs::Permissions::from_mode(0o600)).unwrap();
    assert!(matches!(Store::open(&other), Err(Error::Schema)));
    let link = temp.0.join("link.sqlite");
    std::os::unix::fs::symlink(temp.db(), &link).unwrap();
    assert!(matches!(Store::open(&link), Err(Error::InvalidInput)));
}
#[test]
fn audit_failure_is_atomic_busy_is_bounded_and_entropy_fails_closed() {
    let _serial = SERIAL.lock().unwrap();
    let (temp, mut s, p) = fresh();
    s.db.execute_batch("CREATE TRIGGER fail_audit BEFORE INSERT ON admin_audit BEGIN SELECT RAISE(ABORT,'synthetic audit fault'); END;").unwrap();
    assert!(
        s.create_account(&admin(), "Bobby", "Bobby", PASSWORD, 101)
            .is_err()
    );
    assert!(s.lookup("bobby").is_err());
    assert!(s.set_disabled(&admin(), p.account_id, true, 101).is_err());
    let disabled: bool =
        s.db.query_row(
            "SELECT disabled FROM accounts WHERE id=?1",
            [p.account_id],
            |r| r.get(0),
        )
        .unwrap();
    assert!(!disabled);
    s.db.execute_batch("DROP TRIGGER fail_audit").unwrap();
    let mut lock = Connection::open(temp.db()).unwrap();
    let tx = lock
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .unwrap();
    let start = std::time::Instant::now();
    assert_eq!(s.begin_instance(&admin(), 102), Err(Error::Busy));
    assert!(start.elapsed() < Duration::from_secs(2));
    drop(tx);
    s.random = |_| Err(Error::Entropy);
    assert!(matches!(
        s.login("alice", PASSWORD, 60, 103),
        Err(Error::Entropy)
    ));
    assert!(matches!(
        s.create_account(&admin(), "Bobby", "Bobby", PASSWORD, 103),
        Err(Error::Entropy)
    ));
    let count: i64 =
        s.db.query_row("SELECT count(*) FROM sessions", [], |r| r.get(0))
            .unwrap();
    assert_eq!(count, 0);
}
#[test]
fn hash_work_and_phc_are_bounded_instance_and_allocator_fail_closed() {
    let _serial = SERIAL.lock().unwrap();
    let p1 = HashPermit::acquire().unwrap();
    let p2 = HashPermit::acquire().unwrap();
    assert!(matches!(HashPermit::acquire(), Err(Error::Busy)));
    drop(p1);
    drop(p2);
    assert_eq!(
        verify(PASSWORD, Some("$argon2id$v=19$m=999999999,t=2,p=1$a$b")),
        Err(Error::Schema)
    );
    let temp = Temp::new();
    let mut s = temp.store();
    s.create_account(&admin(), "Alice", "Alice", PASSWORD, 100)
        .unwrap();
    assert!(matches!(
        s.login("alice", PASSWORD, 60, 101),
        Err(Error::InstanceRequired)
    ));
    s.db.execute("UPDATE runtime_state SET next_wire_id=4294967296", [])
        .unwrap();
    assert!(matches!(
        s.create_account(&admin(), "Bobby", "Bobby", PASSWORD, 102),
        Err(Error::Exhausted)
    ));
}

#[test]
fn profile_edit_is_persistent_versioned_and_does_not_change_ownership() {
    let _serial = SERIAL.lock().unwrap();
    let (temp, mut s, p) = fresh();
    let g = s.login("alice", PASSWORD, 60, 101).unwrap();
    let new = s
        .rename_player(&admin(), p.account_id, "NewName", 1, 102)
        .unwrap();
    assert_eq!(new.player_id, p.player_id);
    assert_eq!(new.wire_id, p.wire_id);
    assert_eq!(new.profile_version, 2);
    assert_eq!(
        s.rename_player(&admin(), p.account_id, "LostUpdate", 1, 102),
        Err(Error::Conflict)
    );
    drop(s);
    let mut s = temp.store();
    assert_eq!(s.lookup("alice"), Ok(new.clone()));
    assert_eq!(s.authorize(&g.token, p.player_id, 103), Ok(new));
}

#[test]
fn credential_reset_wins_after_hash_before_session_commit_and_live_schema_is_checked() {
    let _serial = SERIAL.lock().unwrap();
    let (temp, mut s, p) = fresh();
    let phc: String =
        s.db.query_row(
            "SELECT password_phc FROM accounts WHERE id=?1",
            [p.account_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(verify(PASSWORD, Some(&phc)), Ok(true));
    let mut concurrent = temp.store();
    concurrent
        .reset_password(&admin(), p.account_id, OTHER, 101)
        .unwrap();
    assert!(matches!(
        s.issue(p.account_id, phc, 1, 60, None, &mut || Ok(101)),
        Err(Error::InvalidCredentials)
    ));
    let count: i64 =
        s.db.query_row("SELECT count(*) FROM sessions", [], |r| r.get(0))
            .unwrap();
    assert_eq!(count, 0);
    s.db.execute_batch("DROP INDEX one_active_session").unwrap();
    drop(s);
    drop(concurrent);
    assert!(matches!(Store::open(&temp.db()), Err(Error::Schema)));
}

#[test]
fn sqlite_full_during_audit_rolls_back_account_player_and_allocator() {
    let _serial = SERIAL.lock().unwrap();
    let (_temp, mut s, _p) = fresh();
    let before: i64 =
        s.db.query_row("SELECT next_wire_id FROM runtime_state", [], |r| r.get(0))
            .unwrap();
    s.db.execute_batch("CREATE TABLE pressure(data BLOB); CREATE TRIGGER fill_on_audit BEFORE INSERT ON admin_audit BEGIN INSERT INTO pressure VALUES(zeroblob(1048576)); END;").unwrap();
    let pages: i64 =
        s.db.pragma_query_value(None, "page_count", |r| r.get(0))
            .unwrap();
    s.db.pragma_update(None, "max_page_count", pages).unwrap();
    assert!(matches!(
        s.create_account(&admin(), "Bobby", "Bobby", PASSWORD, 101),
        Err(Error::Storage)
    ));
    assert!(s.lookup("bobby").is_err());
    let after: i64 =
        s.db.query_row("SELECT next_wire_id FROM runtime_state", [], |r| r.get(0))
            .unwrap();
    assert_eq!(before, after);
    let accounts: i64 =
        s.db.query_row("SELECT count(*) FROM accounts", [], |r| r.get(0))
            .unwrap();
    let players: i64 =
        s.db.query_row("SELECT count(*) FROM players", [], |r| r.get(0))
            .unwrap();
    assert_eq!((accounts, players), (1, 1));
}

#[test]
fn explicit_idle_authorization_renews_only_owner_never_extends_absolute_or_revives_after_clock_rollback()
 {
    let _serial = SERIAL.lock().unwrap();
    let (temp, mut s, p) = fresh();
    let g = s.login_with_idle("alice", PASSWORD, 100, 10, 101).unwrap();
    assert_eq!(s.authorize(&g.token, p.player_id, 110).unwrap(), p);
    assert_eq!(
        s.authorize(&g.token, p.player_id + 1, 119),
        Err(Error::InvalidSession)
    );
    assert_eq!(
        s.authorize(&g.token, p.player_id, 120),
        Err(Error::InvalidSession)
    );
    assert_eq!(
        s.authorize(&g.token, p.player_id, 119),
        Err(Error::ClockRollback)
    );
    drop(s);
    let mut s = temp.store();
    assert_eq!(
        s.authorize(&g.token, p.player_id, 120),
        Err(Error::InvalidSession)
    );
    let next = s.login_with_idle("alice", PASSWORD, 20, 10, 120).unwrap();
    assert!(s.authorize(&next.token, p.player_id, 129).is_ok());
    assert!(s.authorize(&next.token, p.player_id, 138).is_ok());
    assert_eq!(
        s.authorize(&next.token, p.player_id, 140),
        Err(Error::InvalidSession)
    );
    assert!(matches!(
        s.login_with_idle("alice", PASSWORD, 10, 0, 140),
        Err(Error::InvalidInput)
    ));
}

fn legacy_idle_fixture(db: &mut Connection) -> Token {
    db.execute_batch(MIGRATION).unwrap();
    let digest = hex(&Sha256::digest(MIGRATION.as_bytes()));
    db.execute("INSERT INTO schema_migrations VALUES(1,?1)", [digest])
        .unwrap();
    db.pragma_update(None, "application_id", APPLICATION_ID)
        .unwrap();
    db.pragma_update(None, "user_version", 1).unwrap();
    db.execute(
        "UPDATE runtime_state SET epoch=1,last_utc=100,next_wire_id=43 WHERE id=1",
        [],
    )
    .unwrap();
    let phc = password_hash(PASSWORD, b"fixture-old-salt").unwrap();
    db.execute(
        "INSERT INTO accounts(id,login,password_phc,created_at) VALUES(9,'alice',?1,100)",
        [phc],
    )
    .unwrap();
    db.execute("INSERT INTO players(id,account_id,wire_id,display_name,profile_version) VALUES(7,9,42,'Alice',3)",[]).unwrap();
    let token = Token::parse(&"a".repeat(64)).unwrap();
    db.execute("INSERT INTO sessions(id,account_id,token_hash,boot_epoch,credential_version,issued_at,expires_at) VALUES(6,9,?1,1,1,100,1000)",[token.digest()]).unwrap();
    db.execute("INSERT INTO admin_audit(id,request_id,actor,operation,account_id,reason,at_utc,details) VALUES(2,?1,'test','historical',9,'preserve',100,'{}')",["b".repeat(64)]).unwrap();
    token
}
fn old_idle_data(db: &Connection) -> Vec<Vec<Vec<rusqlite::types::Value>>> {
    ["SELECT * FROM accounts ORDER BY id","SELECT * FROM players ORDER BY id","SELECT * FROM runtime_state","SELECT id,account_id,token_hash,boot_epoch,credential_version,issued_at,expires_at,revoked_at FROM sessions ORDER BY id","SELECT * FROM admin_audit ORDER BY id"]
        .iter().map(|sql|{let mut stmt=db.prepare(sql).unwrap();let n=stmt.column_count();stmt.query_map([],|r|(0..n).map(|i|r.get(i)).collect::<std::result::Result<Vec<_>,_>>()).unwrap().collect::<std::result::Result<Vec<_>,_>>().unwrap()}).collect()
}
#[test]
fn schema1_upgrade_preserves_all_old_account_session_profile_audit_data_and_old_absolute_ttl() {
    let _serial = SERIAL.lock().unwrap();
    let mut db = Connection::open_in_memory().unwrap();
    let token = legacy_idle_fixture(&mut db);
    let before = old_idle_data(&db);
    Store::migrate(&mut db).unwrap();
    assert_eq!(old_idle_data(&db), before);
    Store::migrate(&mut db).unwrap();
    assert_eq!(old_idle_data(&db), before);
    let version: i64 = db
        .pragma_query_value(None, "user_version", |r| r.get(0))
        .unwrap();
    assert_eq!(version, 2);
    let idle: (Option<i64>, Option<i64>) = db
        .query_row(
            "SELECT idle_timeout_seconds,last_seen_at FROM sessions WHERE id=6",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(idle, (None, None));
    let mut store = Store {
        db,
        random: entropy,
    };
    let p = store.authorize(&token, 7, 900).unwrap();
    assert_eq!(
        (p.account_id, p.player_id, p.wire_id, p.profile_version),
        (9, 7, 42, 3)
    );
    assert_eq!(store.authorize(&token, 7, 1000), Err(Error::InvalidSession));
}
#[test]
fn schema1_corruption_or_upgrade_full_rejects_without_partial_columns_or_version_bump() {
    let _serial = SERIAL.lock().unwrap();
    let mut db = Connection::open_in_memory().unwrap();
    legacy_idle_fixture(&mut db);
    db.execute_batch("DROP INDEX session_expiry").unwrap();
    assert_eq!(Store::migrate(&mut db), Err(Error::Schema));
    let version: i64 = db
        .pragma_query_value(None, "user_version", |r| r.get(0))
        .unwrap();
    assert_eq!(version, 1);
    assert!(
        db.prepare("SELECT idle_timeout_seconds FROM sessions")
            .is_err()
    );
    let mut db = Connection::open_in_memory().unwrap();
    legacy_idle_fixture(&mut db);
    let before = old_idle_data(&db);
    let pages: i64 = db
        .pragma_query_value(None, "page_count", |r| r.get(0))
        .unwrap();
    db.pragma_update(None, "max_page_count", pages).unwrap();
    assert_eq!(Store::migrate(&mut db), Err(Error::Storage));
    assert_eq!(old_idle_data(&db), before);
    let version: i64 = db
        .pragma_query_value(None, "user_version", |r| r.get(0))
        .unwrap();
    assert_eq!(version, 1);
    assert!(
        db.prepare("SELECT idle_timeout_seconds FROM sessions")
            .is_err()
    );
    db.pragma_update(None, "max_page_count", pages + 32)
        .unwrap();
    Store::migrate(&mut db).unwrap();
    assert_eq!(old_idle_data(&db), before);
}

#[test]
fn live_login_resamples_publication_after_other_connection_advances_clock() {
    let _serial = SERIAL.lock().unwrap();
    let (temp, mut store, player) = fresh();
    let mut other = temp.store();
    let phc: String = store
        .db
        .query_row(
            "SELECT password_phc FROM accounts WHERE id=?1",
            [player.account_id],
            |r| r.get(0),
        )
        .unwrap();
    // Deterministic interleaving at the post-verification publication seam:
    // caller observed 101 before KDF; another SQLite connection observes 102.
    store.observe(101).unwrap();
    other.observe(102).unwrap();
    assert!(matches!(
        store.issue(player.account_id, phc.clone(), 1, 60, None, &mut || Ok(101)),
        Err(Error::ClockRollback)
    ));
    let mut ticks = [103, 104].into_iter();
    let grant = store
        .issue(player.account_id, phc, 1, 60, None, &mut || {
            Ok(ticks.next().unwrap())
        })
        .unwrap();
    assert!(ticks.next().is_none());
    assert_eq!(grant.expires_at, 164);
    assert_eq!(
        store
            .db
            .query_row("SELECT issued_at FROM sessions", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        104
    );
    other.revoke(&admin(), player.account_id, 105).unwrap();
    let mut ticks = [106, 107, 108].into_iter();
    let fresh = store
        .issue_session_clock("alice", PASSWORD, 60, None, &mut || {
            Ok(ticks.next().unwrap())
        })
        .unwrap();
    assert!(ticks.next().is_none());
    assert_eq!(fresh.expires_at, 168);
    assert_ne!(fresh.token.expose_secret(), grant.token.expose_secret());
    assert_eq!(
        store
            .authorize(&fresh.token, player.player_id, 108)
            .unwrap(),
        player
    );
}

#[test]
fn sampled_live_login_keeps_rollback_and_credential_reset_fences() {
    let _serial = SERIAL.lock().unwrap();
    let (temp, mut store, player) = fresh();
    let mut other = temp.store();
    let mut ticks = [101, 100].into_iter();
    assert!(matches!(
        store.issue_session_clock("alice", PASSWORD, 60, None, &mut || Ok(ticks
            .next()
            .unwrap())),
        Err(Error::ClockRollback)
    ));
    assert_eq!(
        store
            .db
            .query_row("SELECT last_utc FROM runtime_state", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        101
    );
    let phc: String = store
        .db
        .query_row(
            "SELECT password_phc FROM accounts WHERE id=?1",
            [player.account_id],
            |r| r.get(0),
        )
        .unwrap();
    other
        .reset_password(&admin(), player.account_id, OTHER, 102)
        .unwrap();
    assert!(matches!(
        store.issue(player.account_id, phc, 1, 60, None, &mut || Ok(103)),
        Err(Error::InvalidCredentials)
    ));
    assert_eq!(
        store
            .db
            .query_row("SELECT count(*) FROM sessions", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        0
    );
    assert!(matches!(
        store.issue_session_clock("alice", PASSWORD, 60, None, &mut || Ok(104)),
        Err(Error::InvalidCredentials)
    ));
}
