//! Company-owned account backend. No KR wire adapter or network authentication.
//! Synchronous: call from an isolated bounded worker, never a Tokio room task.
use argon2::{Algorithm, Argon2, Params, PasswordHasher, PasswordVerifier, Version};
use rusqlite::{
    Connection, OpenFlags, OptionalExtension, Transaction, TransactionBehavior, params,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    fs::{self, OpenOptions},
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
    path::Path,
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};
use zeroize::Zeroizing;

mod matches;
mod preferences;
pub use matches::*;
pub use preferences::CharacterPreference;

const MIGRATION: &str = include_str!("../migrations/0001_accounts.sql");
const MIGRATION_IDLE: &str = include_str!("../migrations/0002_session_idle.sql");
const MIGRATION_MATCHES: &str = include_str!("../migrations/0003_match_history.sql");
pub const SCHEMA_VERSION: i32 = 3;
const APPLICATION_ID: i32 = 0x4b524143;
const TTL_MAX: i64 = 86_400;
static HASH_WORK: AtomicUsize = AtomicUsize::new(0);
#[cfg(test)]
pub(crate) static TEST_SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Redacted errors: SQL/PHC/token/password values never become user diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    InvalidInput,
    InvalidCredentials,
    InvalidSession,
    ActiveSession,
    Conflict,
    Busy,
    Storage,
    Schema,
    ClockRollback,
    Entropy,
    InstanceRequired,
    Exhausted,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for Error {}
impl From<rusqlite::Error> for Error {
    fn from(e: rusqlite::Error) -> Self {
        match e.sqlite_error_code() {
            Some(rusqlite::ErrorCode::DatabaseBusy | rusqlite::ErrorCode::DatabaseLocked) => {
                Self::Busy
            }
            Some(rusqlite::ErrorCode::ConstraintViolation) => Self::Conflict,
            _ => Self::Storage,
        }
    }
}
pub type Result<T> = std::result::Result<T, Error>;

struct HashPermit;
impl HashPermit {
    fn acquire() -> Result<Self> {
        HASH_WORK
            .try_update(Ordering::AcqRel, Ordering::Acquire, |n| {
                (n < 2).then_some(n + 1)
            })
            .map(|_| Self)
            .map_err(|_| Error::Busy)
    }
}
impl Drop for HashPermit {
    fn drop(&mut self) {
        HASH_WORK.fetch_sub(1, Ordering::AcqRel);
    }
}
fn argon() -> Argon2<'static> {
    Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(19_456, 2, 1, Some(32)).expect("fixed valid Argon2id parameters"),
    )
}
fn password_bounds(password: &[u8]) -> Result<()> {
    if !(12..=128).contains(&password.len()) {
        return Err(Error::InvalidInput);
    }
    Ok(())
}
fn password_hash(password: &[u8], salt: &[u8]) -> Result<String> {
    password_bounds(password)?;
    let _permit = HashPermit::acquire()?;
    argon()
        .hash_password_with_salt(password, salt)
        .map(|s| s.to_string())
        .map_err(|_| Error::Storage)
}
fn verify(password: &[u8], phc: Option<&str>) -> Result<bool> {
    password_bounds(password)?;
    let _permit = HashPermit::acquire()?;
    let Some(phc) = phc else {
        // Same configured KDF work for unknown principal; this is not a password salt.
        argon()
            .hash_password_with_salt(password, b"KR-dummy-salt-01")
            .map_err(|_| Error::Storage)?;
        return Ok(false);
    };
    // Never let a corrupt/untrusted PHC request unbounded memory/iterations.
    let fields: Vec<_> = phc.split('$').collect();
    if fields.len() != 6
        || fields[..4] != ["", "argon2id", "v=19", "m=19456,t=2,p=1"]
        || fields[4].len() != 22
        || fields[5].len() != 43
    {
        return Err(Error::Schema);
    }
    Ok(argon().verify_password(password, phc).is_ok())
}
fn entropy(out: &mut [u8]) -> Result<()> {
    getrandom::fill(out).map_err(|_| Error::Entropy)
}
fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        write!(&mut out, "{b:02x}").expect("String formatting");
    }
    out
}

/// No Debug/Serialize: tokens may be revealed once by the explicit admin CLI.
pub struct Token(Zeroizing<String>);
impl Token {
    pub fn parse(secret: &str) -> Result<Self> {
        if secret.len() != 64
            || !secret
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(Error::InvalidSession);
        }
        Ok(Self(Zeroizing::new(secret.to_owned())))
    }
    pub fn expose_secret(&self) -> &str {
        &self.0
    }
    fn digest(&self) -> Vec<u8> {
        Sha256::digest(self.0.as_bytes()).to_vec()
    }
}
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Player {
    pub account_id: i64,
    pub player_id: i64,
    pub wire_id: u32,
    pub display_name: String,
    pub profile_version: i64,
}
pub struct Grant {
    pub player: Player,
    pub token: Token,
    pub expires_at: i64,
}
/// Trusted local administrator only; no claim of remote RBAC or client identity.
pub struct Admin<'a> {
    pub actor: &'a str,
    pub reason: &'a str,
}
impl Admin<'_> {
    fn validate(&self) -> Result<()> {
        if !(1..=64).contains(&self.actor.len())
            || !(1..=256).contains(&self.reason.len())
            || self
                .actor
                .chars()
                .chain(self.reason.chars())
                .any(char::is_control)
        {
            return Err(Error::InvalidInput);
        }
        Ok(())
    }
}
fn login_name(login: &str) -> Result<String> {
    if !(3..=32).contains(&login.len())
        || !login
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'_')
    {
        return Err(Error::InvalidInput);
    }
    Ok(login.to_ascii_lowercase())
}
fn display_name(name: &str) -> Result<()> {
    // Backend policy only; native dynamic profile/name encoding is a separate adapter.
    if !(3..=15).contains(&name.len())
        || !name.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_')
    {
        return Err(Error::InvalidInput);
    }
    Ok(())
}

pub struct Store {
    db: Connection,
    random: fn(&mut [u8]) -> Result<()>,
    live_utc: i64,
}
impl Store {
    /// Parent must already be a private directory under a trusted administrator path.
    /// Opening the DB never starts a new instance or invalidates sessions.
    pub fn open(path: &Path) -> Result<Self> {
        let parent = path.parent().ok_or(Error::InvalidInput)?;
        let meta = fs::symlink_metadata(parent).map_err(|_| Error::Storage)?;
        if !meta.is_dir() || meta.permissions().mode() & 0o077 != 0 {
            return Err(Error::InvalidInput);
        }
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(path)
        {
            Ok(_) => (),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => (),
            Err(_) => return Err(Error::Storage),
        }
        let meta = fs::symlink_metadata(path).map_err(|_| Error::Storage)?;
        if !meta.is_file() || meta.permissions().mode() & 0o077 != 0 {
            return Err(Error::InvalidInput);
        }
        let mut db = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        db.busy_timeout(Duration::from_millis(250))?;
        db.pragma_update(None, "foreign_keys", true)?;
        db.pragma_update(None, "trusted_schema", false)?;
        Self::migrate(&mut db)?;
        db.pragma_update(None, "journal_mode", "WAL")?;
        db.pragma_update(None, "synchronous", "FULL")?;
        let quick: String = db.query_row("PRAGMA quick_check", [], |r| r.get(0))?;
        let broken_fk = db.prepare("PRAGMA foreign_key_check")?.exists([])?;
        if quick != "ok" || broken_fk {
            return Err(Error::Schema);
        }
        Ok(Self {
            live_utc: 0,
            db,
            random: entropy,
        })
    }
    fn migrate(db: &mut Connection) -> Result<()> {
        let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let app: i32 = tx.pragma_query_value(None, "application_id", |r| r.get(0))?;
        let mut version: i32 = tx.pragma_query_value(None, "user_version", |r| r.get(0))?;
        let scripts = [MIGRATION, MIGRATION_IDLE, MIGRATION_MATCHES];
        let digests: Vec<_> = scripts
            .iter()
            .map(|script| hex(&Sha256::digest(script.as_bytes())))
            .collect();
        if app == 0 && version == 0 {
            let count: i64 = tx.query_row(
                "SELECT count(*) FROM sqlite_master WHERE name NOT LIKE 'sqlite_%'",
                [],
                |r| r.get(0),
            )?;
            if count != 0 {
                return Err(Error::Schema);
            }
            tx.execute_batch(MIGRATION)?;
            tx.execute("INSERT INTO schema_migrations VALUES(1,?1)", [&digests[0]])?;
            tx.pragma_update(None, "application_id", APPLICATION_ID)?;
            tx.pragma_update(None, "user_version", 1)?;
            version = 1;
        } else if app != APPLICATION_ID || !(1..=SCHEMA_VERSION).contains(&version) {
            return Err(Error::Schema);
        }
        let migrations: Vec<(i32, String)> = tx
            .prepare("SELECT version,sha256 FROM schema_migrations ORDER BY version")?
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
            .collect::<std::result::Result<_, _>>()?;
        let wanted: Vec<_> = digests
            .iter()
            .take(version as usize)
            .enumerate()
            .map(|(i, s)| (i as i32 + 1, s.clone()))
            .collect();
        if migrations != wanted {
            return Err(Error::Schema);
        }
        // Validate the old live schema BEFORE upgrading, so a migration cannot
        // conceal a pre-existing missing index/foreign trigger/altered constraint.
        let expected = Connection::open_in_memory()?;
        for script in scripts.iter().take(version as usize) {
            expected.execute_batch(script)?;
        }
        if Self::schema(&tx)? != Self::schema(&expected)? {
            return Err(Error::Schema);
        }
        let state: (i64, i64, i64) = tx.query_row(
            "SELECT epoch,last_utc,next_wire_id FROM runtime_state WHERE id=1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )?;
        let max_wire: i64 =
            tx.query_row("SELECT coalesce(max(wire_id),0) FROM players", [], |r| {
                r.get(0)
            })?;
        if state.0 < 0 || state.1 < 0 || state.2 <= max_wire || state.2 > 4_294_967_296 {
            return Err(Error::Schema);
        }
        while version < SCHEMA_VERSION {
            tx.execute_batch(scripts[version as usize])?;
            tx.execute(
                "INSERT INTO schema_migrations VALUES(?1,?2)",
                params![version + 1, &digests[version as usize]],
            )?;
            expected.execute_batch(scripts[version as usize])?;
            version += 1;
            tx.pragma_update(None, "user_version", version)?;
            if Self::schema(&tx)? != Self::schema(&expected)? {
                return Err(Error::Schema);
            }
        }
        tx.commit()?;
        Ok(())
    }
    fn schema(db: &Connection) -> Result<Vec<(String, String, String)>> {
        Ok(db.prepare("SELECT type,name,coalesce(sql,'') FROM sqlite_master WHERE name NOT LIKE 'sqlite_%' ORDER BY type,name")?
            .query_map([],|r| Ok((r.get(0)?,r.get(1)?,r.get(2)?)))?
            .collect::<std::result::Result<_,_>>()?)
    }
    fn clock(tx: &Transaction<'_>, now: i64) -> Result<()> {
        let last: i64 = tx.query_row("SELECT last_utc FROM runtime_state WHERE id=1", [], |r| {
            r.get(0)
        })?;
        if now < 0 || now < last {
            return Err(Error::ClockRollback);
        }
        tx.execute("UPDATE runtime_state SET last_utc=?1 WHERE id=1", [now])?;
        Ok(())
    }
    #[cfg(test)]
    fn observe(&mut self, now: i64) -> Result<()> {
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        Self::clock(&tx, now)?;
        tx.commit()?;
        Ok(())
    }
    fn observe_clock(&mut self, clock: &mut impl FnMut() -> Result<i64>) -> Result<()> {
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        Self::clock(&tx, clock()?)?;
        tx.commit()?;
        Ok(())
    }
    fn at_clock<T>(
        &mut self,
        clock: &mut impl FnMut() -> Result<i64>,
        op: impl FnOnce(&Transaction<'_>, i64) -> Result<T>,
    ) -> Result<T> {
        // Persist observation even on denial. Sample inside each acquired writer
        // transaction: another connection may advance UTC during KDF or lock wait.
        self.observe_clock(clock)?;
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let now = clock()?;
        Self::clock(&tx, now)?;
        let out = op(&tx, now)?;
        tx.commit()?;
        Ok(out)
    }
    fn at<T>(&mut self, now: i64, op: impl FnOnce(&Transaction<'_>) -> Result<T>) -> Result<T> {
        self.at_clock(&mut || Ok(now), |tx, _| op(tx))
    }
    fn random<const N: usize>(&self) -> Result<[u8; N]> {
        let mut bytes = [0; N];
        (self.random)(&mut bytes)?;
        Ok(bytes)
    }
    fn audit(
        tx: &Transaction<'_>,
        id: &str,
        admin: &Admin<'_>,
        op: &str,
        account: Option<i64>,
        now: i64,
    ) -> Result<()> {
        tx.execute("INSERT INTO admin_audit(request_id,actor,operation,account_id,reason,at_utc,details) VALUES(?1,?2,?3,?4,?5,?6,'{}')",
            params![id,admin.actor,op,account,admin.reason,now])?;
        Ok(())
    }
    pub fn begin_instance(&mut self, admin: &Admin<'_>, now: i64) -> Result<i64> {
        admin.validate()?;
        let request = hex(&self.random::<32>()?);
        self.at(now, |tx| {
            let epoch: i64 =
                tx.query_row("SELECT epoch FROM runtime_state WHERE id=1", [], |r| {
                    r.get(0)
                })?;
            let next = epoch.checked_add(1).ok_or(Error::Exhausted)?;
            tx.execute("UPDATE runtime_state SET epoch=?1 WHERE id=1", [next])?;
            tx.execute(
                "UPDATE sessions SET revoked_at=?1 WHERE revoked_at IS NULL",
                [now],
            )?;
            matches::cancel_running(tx, now)?;
            Self::audit(tx, &request, admin, "begin_instance", None, now)?;
            Ok(next)
        })
    }
    pub fn create_account(
        &mut self,
        admin: &Admin<'_>,
        login: &str,
        name: &str,
        password: &[u8],
        now: i64,
    ) -> Result<Player> {
        admin.validate()?;
        let login = login_name(login)?;
        display_name(name)?;
        let phc = password_hash(password, &self.random::<16>()?)?;
        let request = hex(&self.random::<32>()?);
        self.at(now, |tx| {
            let wire: i64 = tx.query_row(
                "SELECT next_wire_id FROM runtime_state WHERE id=1",
                [],
                |r| r.get(0),
            )?;
            if wire > i64::from(u32::MAX) {
                return Err(Error::Exhausted);
            }
            tx.execute(
                "INSERT INTO accounts(login,password_phc,created_at) VALUES(?1,?2,?3)",
                params![login, phc, now],
            )?;
            let account = tx.last_insert_rowid();
            tx.execute(
                "INSERT INTO players(account_id,wire_id,display_name) VALUES(?1,?2,?3)",
                params![account, wire, name],
            )?;
            tx.execute(
                "UPDATE runtime_state SET next_wire_id=next_wire_id+1 WHERE id=1",
                [],
            )?;
            Self::audit(tx, &request, admin, "create_account", Some(account), now)?;
            Self::player(tx, account)
        })
    }
    fn player(db: &Connection, account: i64) -> Result<Player> {
        db.query_row("SELECT account_id,id,wire_id,display_name,profile_version FROM players WHERE account_id=?1", [account], |r| Ok(Player {
            account_id:r.get(0)?, player_id:r.get(1)?, wire_id:r.get(2)?, display_name:r.get(3)?, profile_version:r.get(4)?
        })).optional()?.ok_or(Error::InvalidCredentials)
    }
    pub fn lookup(&self, login: &str) -> Result<Player> {
        let login = login_name(login)?;
        let account: i64 = self
            .db
            .query_row("SELECT id FROM accounts WHERE login=?1", [login], |r| {
                r.get(0)
            })
            .optional()?
            .ok_or(Error::InvalidCredentials)?;
        Self::player(&self.db, account)
    }
    pub fn login(&mut self, login: &str, password: &[u8], ttl: i64, now: i64) -> Result<Grant> {
        self.issue_session(login, password, ttl, None, now)
    }
    /// Trusted live login: sample UTC after acquiring each writer transaction,
    /// including publication after password verification. Explicit-time APIs keep
    /// their supplied-time semantics for deterministic callers and policy tests.
    pub fn login_current(&mut self, login: &str, password: &[u8], ttl: i64) -> Result<Grant> {
        self.issue_session_clock(login, password, ttl, None, &mut || {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .ok()
                .and_then(|time| i64::try_from(time.as_secs()).ok())
                .ok_or(Error::ClockRollback)
        })
    }
    /// Company backend policy, opt-in for new sessions. Existing schema1 sessions
    /// retain absolute TTL after migration; only successful own-player auth renews.
    pub fn login_with_idle(
        &mut self,
        login: &str,
        password: &[u8],
        ttl: i64,
        idle: i64,
        now: i64,
    ) -> Result<Grant> {
        if !(1..=TTL_MAX).contains(&idle) {
            return Err(Error::InvalidInput);
        }
        self.issue_session(login, password, ttl, Some(idle), now)
    }
    fn issue_session(
        &mut self,
        login: &str,
        password: &[u8],
        ttl: i64,
        idle: Option<i64>,
        now: i64,
    ) -> Result<Grant> {
        self.issue_session_clock(login, password, ttl, idle, &mut || Ok(now))
    }
    fn issue_session_clock(
        &mut self,
        login: &str,
        password: &[u8],
        ttl: i64,
        idle: Option<i64>,
        clock: &mut impl FnMut() -> Result<i64>,
    ) -> Result<Grant> {
        let login = login_name(login)?;
        if !(1..=TTL_MAX).contains(&ttl) {
            return Err(Error::InvalidInput);
        }
        self.observe_clock(clock)?;
        let row: Option<(i64, String, i64, bool)> = self
            .db
            .query_row(
                "SELECT id,password_phc,credential_version,disabled FROM accounts WHERE login=?1",
                [&login],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .optional()?;
        let valid = verify(password, row.as_ref().map(|r| r.1.as_str()))?;
        let Some((account, phc, version, disabled)) = row else {
            return Err(Error::InvalidCredentials);
        };
        if !valid || disabled {
            return Err(Error::InvalidCredentials);
        }
        self.issue(account, phc, version, ttl, idle, clock)
    }
    fn issue(
        &mut self,
        account: i64,
        phc: String,
        version: i64,
        ttl: i64,
        idle: Option<i64>,
        clock: &mut impl FnMut() -> Result<i64>,
    ) -> Result<Grant> {
        let raw = Zeroizing::new(self.random::<32>()?);
        let token = Token(Zeroizing::new(hex(raw.as_ref())));
        let digest = token.digest();
        self.at_clock(clock, |tx, now| {
            let expires = now.checked_add(ttl).ok_or(Error::InvalidInput)?;
            let epoch: i64 = tx.query_row("SELECT epoch FROM runtime_state WHERE id=1", [], |r| r.get(0))?;
            if epoch==0 { return Err(Error::InstanceRequired); }
            // Verification happened outside the write transaction. Reset/ban must
            // win over an in-flight hash; never publish a session for old credentials.
            let current: (String,i64,bool) = tx.query_row("SELECT password_phc,credential_version,disabled FROM accounts WHERE id=?1", [account], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?)))?;
            if current!=(phc,version,false) { return Err(Error::InvalidCredentials); }
            tx.execute("UPDATE sessions SET revoked_at=?1 WHERE revoked_at IS NULL AND (expires_at<=?1 OR (idle_timeout_seconds IS NOT NULL AND last_seen_at<=?1-idle_timeout_seconds))", [now])?;
            if tx.query_row("SELECT count(*) FROM sessions WHERE account_id=?1 AND revoked_at IS NULL", [account], |r| r.get::<_,i64>(0))? != 0 { return Err(Error::ActiveSession); }
            tx.execute("INSERT INTO sessions(account_id,token_hash,boot_epoch,credential_version,issued_at,expires_at,idle_timeout_seconds,last_seen_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8)", params![account,digest,epoch,version,now,expires,idle,idle.map(|_|now)])?;
            Ok(Grant {player:Self::player(tx,account)?,token,expires_at:expires})
        })
    }
    pub fn authorize(&mut self, token: &Token, expected_player: i64, now: i64) -> Result<Player> {
        self.check_session(token, expected_player, now, true)
    }
    /// Check revocation/expiry without converting a server timer into player activity.
    pub fn inspect_session(
        &mut self,
        token: &Token,
        expected_player: i64,
        now: i64,
    ) -> Result<Player> {
        self.check_session(token, expected_player, now, false)
    }
    /// Live service lane: read the current WAL snapshot without taking the
    /// login/admin writer lock. Revocation, epoch, credentials, expiry, profile
    /// and idle validity still come from SQLite on every check; no cached grant.
    /// Optional idle/watermark writes are best effort only while a writer is busy.
    pub(crate) fn inspect_live(
        &mut self,
        token: &Token,
        expected_player: i64,
        now: i64,
        activity: bool,
    ) -> Result<Player> {
        self.db.busy_timeout(Duration::ZERO)?;
        if now < 0 || now < self.live_utc {
            return Err(Error::ClockRollback);
        }
        let digest = token.digest();
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Deferred)?;
        let last: i64 = tx.query_row("SELECT last_utc FROM runtime_state WHERE id=1", [], |r| {
            r.get(0)
        })?;
        if now < last {
            return Err(Error::ClockRollback);
        }
        // Denials also advance the in-process watermark: a later clock rollback
        // cannot make an already observed expired session valid in this lane.
        self.live_utc = now;
        let account:Option<i64>=tx.query_row("SELECT a.id FROM sessions s JOIN accounts a ON a.id=s.account_id JOIN runtime_state r ON r.id=1 JOIN players p ON p.account_id=a.id WHERE s.token_hash=?1 AND p.id=?2 AND s.revoked_at IS NULL AND s.expires_at>?3 AND s.issued_at<=?3 AND s.boot_epoch=r.epoch AND s.credential_version=a.credential_version AND a.disabled=0 AND (s.idle_timeout_seconds IS NULL OR s.last_seen_at>?3-s.idle_timeout_seconds)",params![digest,expected_player,now],|r|r.get(0)).optional()?;
        let player = Self::player(&tx, account.ok_or(Error::InvalidSession)?)?;
        tx.commit()?;
        // Never wait behind an administrative/KDF writer. Session expiry remains
        // effective if an enabled idle deadline elapses during a prolonged lock.
        let update = (|| -> Result<()> {
            let tx = self
                .db
                .transaction_with_behavior(TransactionBehavior::Immediate)?;
            Self::clock(&tx, now)?;
            if activity {
                tx.execute("UPDATE sessions SET last_seen_at=?1 WHERE token_hash=?2 AND account_id=?3 AND idle_timeout_seconds IS NOT NULL AND revoked_at IS NULL AND expires_at>?1 AND last_seen_at<=?1",params![now,digest,player.account_id])?;
            }
            tx.commit()?;
            Ok(())
        })();
        match update {
            Ok(()) | Err(Error::Busy) => Ok(player),
            Err(error) => Err(error),
        }
    }
    fn session_account(
        db: &Connection,
        digest: &[u8],
        expected_player: i64,
        now: i64,
    ) -> Result<i64> {
        db.query_row("SELECT a.id FROM sessions s JOIN accounts a ON a.id=s.account_id JOIN runtime_state r ON r.id=1 JOIN players p ON p.account_id=a.id WHERE s.token_hash=?1 AND p.id=?2 AND s.revoked_at IS NULL AND s.expires_at>?3 AND s.issued_at<=?3 AND s.boot_epoch=r.epoch AND s.credential_version=a.credential_version AND a.disabled=0 AND (s.idle_timeout_seconds IS NULL OR s.last_seen_at>?3-s.idle_timeout_seconds)", params![digest,expected_player,now], |row|row.get(0)).optional()?.ok_or(Error::InvalidSession)
    }
    fn check_session(
        &mut self,
        token: &Token,
        expected_player: i64,
        now: i64,
        renew_idle: bool,
    ) -> Result<Player> {
        let digest = token.digest();
        self.at(now, |tx| {
            let account = Self::session_account(tx, &digest, expected_player, now)?;
            if renew_idle {
                tx.execute("UPDATE sessions SET last_seen_at=?1 WHERE token_hash=?2 AND account_id=?3 AND idle_timeout_seconds IS NOT NULL",params![now,digest,account])?;
            }
            Self::player(tx,account)
        })
    }
    /// Worker-only compensation for a committed login result never accepted by
    /// its caller. Revoke only that token, never a newer session of the account.
    pub(crate) fn release_undelivered(&mut self, grant: &Grant, now: i64) -> Result<()> {
        if now < 0 {
            return Err(Error::ClockRollback);
        }
        let request = hex(&self.random::<32>()?);
        let digest = grant.token.digest();
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let last: i64 = tx.query_row("SELECT last_utc FROM runtime_state WHERE id=1", [], |r| {
            r.get(0)
        })?;
        // Another DB connection may have observed a later UTC during KDF/result
        // handoff. Compensation must not turn the durable watermark backwards.
        let now = now.max(last);
        Self::clock(&tx, now)?;
        let changed = tx.execute("UPDATE sessions SET revoked_at=?1 WHERE token_hash=?2 AND account_id=?3 AND revoked_at IS NULL", params![now,digest,grant.player.account_id])?;
        if changed != 0 {
            Self::audit(
                &tx,
                &request,
                &Admin {
                    actor: "account-worker",
                    reason: "login result not accepted by caller",
                },
                "abandoned_login",
                Some(grant.player.account_id),
                now,
            )?;
        }
        tx.commit()?;
        Ok(())
    }
    /// Administrative revocation; not a client logout wire/API claim.
    pub fn revoke(&mut self, admin: &Admin<'_>, account: i64, now: i64) -> Result<()> {
        admin.validate()?;
        let request = hex(&self.random::<32>()?);
        self.at(now, |tx| {
            Self::player(tx, account)?;
            tx.execute(
                "UPDATE sessions SET revoked_at=?1 WHERE account_id=?2 AND revoked_at IS NULL",
                params![now, account],
            )?;
            Self::audit(tx, &request, admin, "revoke", Some(account), now)
        })
    }
    pub fn set_disabled(
        &mut self,
        admin: &Admin<'_>,
        account: i64,
        disabled: bool,
        now: i64,
    ) -> Result<()> {
        admin.validate()?;
        let request = hex(&self.random::<32>()?);
        self.at(now, |tx| {
            if tx.execute(
                "UPDATE accounts SET disabled=?1 WHERE id=?2",
                params![disabled, account],
            )? != 1
            {
                return Err(Error::InvalidInput);
            }
            if disabled {
                tx.execute(
                    "UPDATE sessions SET revoked_at=?1 WHERE account_id=?2 AND revoked_at IS NULL",
                    params![now, account],
                )?;
            }
            Self::audit(
                tx,
                &request,
                admin,
                if disabled { "disable" } else { "enable" },
                Some(account),
                now,
            )
        })
    }
    pub fn rename_player(
        &mut self,
        admin: &Admin<'_>,
        account: i64,
        name: &str,
        expected_version: i64,
        now: i64,
    ) -> Result<Player> {
        admin.validate()?;
        display_name(name)?;
        let request = hex(&self.random::<32>()?);
        self.at(now, |tx| {
            if tx.execute("UPDATE players SET display_name=?1,profile_version=profile_version+1 WHERE account_id=?2 AND profile_version=?3", params![name,account,expected_version])?!=1 { return Err(Error::Conflict); }
            Self::audit(tx,&request,admin,"rename_player",Some(account),now)?;
            Self::player(tx,account)
        })
    }
    pub fn reset_password(
        &mut self,
        admin: &Admin<'_>,
        account: i64,
        password: &[u8],
        now: i64,
    ) -> Result<()> {
        admin.validate()?;
        let phc = password_hash(password, &self.random::<16>()?)?;
        let request = hex(&self.random::<32>()?);
        self.at(now, |tx| {
            if tx.execute("UPDATE accounts SET password_phc=?1,credential_version=credential_version+1 WHERE id=?2", params![phc,account])?!=1 { return Err(Error::InvalidInput); }
            tx.execute("UPDATE sessions SET revoked_at=?1 WHERE account_id=?2 AND revoked_at IS NULL", params![now,account])?;
            Self::audit(tx,&request,admin,"reset_password",Some(account),now)
        })
    }
}

#[cfg(test)]
mod tests;
