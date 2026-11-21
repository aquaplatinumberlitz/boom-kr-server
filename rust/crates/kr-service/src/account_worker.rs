//! Typed bounded account queue; blocking SQLite/KDF work stays off Tokio tasks.
//! No KR socket adapter. Queue expiry is checked before execution, not a timeout
//! or rollback guarantee for an operation already running in SQLite/Argon2.
pub(crate) mod rate;
use crate::accounts::{self, Grant, Player, Store, Token};
pub use rate::RatePolicy;
use std::{
    path::PathBuf,
    sync::{Arc, Condvar, Mutex},
    thread,
    time::{Duration, Instant},
};
use tokio::sync::{mpsc, oneshot};
use zeroize::Zeroizing;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    InvalidInput,
    Busy,
    Closed,
    QueueExpired,
    RateLimited,
    WorkerFault,
    Backend(accounts::Error),
}
pub type Result<T> = std::result::Result<T, Error>;

/// Contains no printable/serializable secret; dropping it cancels queued work.
/// An in-flight login whose result is dropped revokes only its issued token.
pub struct Pending<T> {
    reply: Option<oneshot::Receiver<Result<T>>>,
    handoff: Option<Arc<Handoff>>,
}
impl<T> Pending<T> {
    pub async fn wait(self) -> Result<T> {
        self.receive().await?.accept()
    }
    /// Borrow a result for bounded network delivery before accepting its login
    /// handoff. Dropping this value revokes only that undelivered session.
    pub async fn receive(mut self) -> Result<Received<T>> {
        let value = self
            .reply
            .take()
            .expect("reply owned once")
            .await
            .map_err(|_| Error::Closed)??;
        Ok(Received {
            value: Some(value),
            handoff: self.handoff.take(),
        })
    }
}
pub struct Received<T> {
    value: Option<T>,
    handoff: Option<Arc<Handoff>>,
}
impl<T> Received<T> {
    pub fn value(&self) -> &T {
        self.value.as_ref().expect("result owned until accepted")
    }
    /// Call only after the entire response has been delivered within the worker
    /// handoff deadline. A late caller cannot resurrect an abandoned session.
    pub fn accept(mut self) -> Result<T> {
        if let Some(handoff) = &self.handoff {
            handoff.claim()?;
        }
        self.handoff = None;
        Ok(self.value.take().expect("result accepted once"))
    }
}
impl<T> Drop for Received<T> {
    fn drop(&mut self) {
        if let Some(handoff) = &self.handoff {
            handoff.abandon();
        }
    }
}
impl<T> Drop for Pending<T> {
    fn drop(&mut self) {
        if let Some(handoff) = &self.handoff {
            handoff.abandon();
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Delivery {
    Waiting,
    Claimed,
    Abandoned,
}
struct Handoff {
    state: Mutex<Delivery>,
    changed: Condvar,
}
impl Handoff {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            state: Mutex::new(Delivery::Waiting),
            changed: Condvar::new(),
        })
    }
    fn claim(&self) -> Result<()> {
        let mut state = self.state.lock().map_err(|_| Error::WorkerFault)?;
        if *state != Delivery::Waiting {
            return Err(Error::QueueExpired);
        }
        *state = Delivery::Claimed;
        self.changed.notify_all();
        Ok(())
    }
    fn abandon(&self) {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if *state == Delivery::Waiting {
            *state = Delivery::Abandoned;
            self.changed.notify_all();
        }
    }
    fn accepted(&self, ttl: Duration) -> Result<bool> {
        let state = self.state.lock().map_err(|_| Error::WorkerFault)?;
        let (mut state, _) = self
            .changed
            .wait_timeout_while(state, ttl, |s| *s == Delivery::Waiting)
            .map_err(|_| Error::WorkerFault)?;
        if *state == Delivery::Waiting {
            *state = Delivery::Abandoned;
        }
        Ok(*state == Delivery::Claimed)
    }
}
struct Admission {
    sender: Option<mpsc::Sender<Command>>,
    limiter: rate::Limiter,
}
type Gate = Arc<Mutex<Admission>>;
#[derive(Clone)]
pub struct Client {
    gate: Gate,
    queue_ttl: Duration,
}
pub struct Worker {
    client: Client,
    thread: Option<thread::JoinHandle<Result<()>>>,
}

enum AuthorizationTime {
    Supplied(i64),
    ExecutionUtc,
}

/// Trusted service persistence jobs. Callers retain the exact snapshot/revision
/// when retrying Busy or a lost response; this is not a client-facing API.
pub enum PersistenceJob {
    BeginMatch(accounts::MatchStart),
    FinishMatch(accounts::MatchKey, accounts::MatchFinish),
    Character {
        token: Token,
        player: i64,
    },
    SelectCharacter {
        token: Token,
        player: i64,
        character: u16,
        expected_revision: i64,
    },
}
pub enum PersistenceResult {
    Match(accounts::MatchRecord),
    Character(accounts::CharacterPreference),
}

enum LocalLoginRequest {
    Accounts(Vec<(Zeroizing<String>, Zeroizing<Vec<u8>>)>),
    Guests(u8, Option<String>),
}
enum Command {
    Manage {
        login: String,
        operation: accounts::Management,
        deadline: Instant,
        reply: oneshot::Sender<Result<Option<Zeroizing<String>>>>,
    },
    EndLocal {
        grants: Vec<Grant>,
        reply: oneshot::Sender<Result<()>>,
    },
    Logout {
        token: Token,
        player: i64,
        deadline: Instant,
        reply: oneshot::Sender<Result<()>>,
    },
    Persist {
        job: PersistenceJob,
        deadline: Instant,
        reply: oneshot::Sender<Result<PersistenceResult>>,
    },
    BeginInstance {
        reply: oneshot::Sender<Result<i64>>,
    },
    Login {
        login: String,
        password: Zeroizing<Vec<u8>>,
        ttl: i64,
        idle: Option<i64>,
        now: AuthorizationTime,
        deadline: Instant,
        reply: oneshot::Sender<Result<Grant>>,
        handoff: Arc<Handoff>,
        handoff_ttl: Duration,
    },
    LocalLogin {
        request: LocalLoginRequest,
        ttl: i64,
        idle: Option<i64>,
        now: AuthorizationTime,
        deadline: Instant,
        reply: oneshot::Sender<Result<Vec<Grant>>>,
        handoff: Arc<Handoff>,
        handoff_ttl: Duration,
    },
    Authorize {
        token: Token,
        player: i64,
        now: AuthorizationTime,
        renew_idle: bool,
        deadline: Instant,
        reply: oneshot::Sender<Result<Player>>,
    },
    #[cfg(test)]
    Pause {
        entered: oneshot::Sender<()>,
        resume: std::sync::mpsc::Receiver<()>,
    },
    #[cfg(test)]
    Panic,
}
fn close(gate: &Gate) {
    gate.lock().unwrap_or_else(|e| e.into_inner()).sender.take();
}
struct CloseOnExit(Gate);
impl Drop for CloseOnExit {
    fn drop(&mut self) {
        close(&self.0);
    }
}

impl Worker {
    /// Opens/schema-checks an existing or fresh private DB on the dedicated thread.
    /// Does not begin a new boot epoch/revoke sessions; that remains explicit.
    /// Bounds: 1..64 queued jobs plus one running, 1ms..60s queue TTL and, separately, result handoff TTL.
    pub async fn open(path: PathBuf, capacity: usize, queue_ttl: Duration) -> Result<Self> {
        Self::open_with_rate(path, capacity, queue_ttl, RatePolicy::default()).await
    }
    pub async fn open_with_rate(
        path: PathBuf,
        capacity: usize,
        queue_ttl: Duration,
        rate: RatePolicy,
    ) -> Result<Self> {
        Self::open_mode(path, capacity, queue_ttl, rate, false).await
    }
    pub(crate) async fn open_live(
        path: PathBuf,
        capacity: usize,
        queue_ttl: Duration,
    ) -> Result<Self> {
        Self::open_mode(path, capacity, queue_ttl, RatePolicy::default(), true).await
    }
    async fn open_mode(
        path: PathBuf,
        capacity: usize,
        queue_ttl: Duration,
        rate: RatePolicy,
        live: bool,
    ) -> Result<Self> {
        if !(1..=64).contains(&capacity)
            || !(Duration::from_millis(1)..=Duration::from_secs(60)).contains(&queue_ttl)
        {
            return Err(Error::InvalidInput);
        }
        let (tx, rx) = mpsc::channel(capacity);
        let gate = Arc::new(Mutex::new(Admission {
            sender: Some(tx),
            limiter: rate::Limiter::new(rate)?,
        }));
        let client = Client {
            gate: gate.clone(),
            queue_ttl,
        };
        let (startup, ready) = oneshot::channel();
        let task = thread::Builder::new()
            .name("kr-account-db".into())
            .spawn(move || {
                let _close = CloseOnExit(gate);
                let store = Store::open(&path).map_err(Error::Backend);
                let mut store = match store {
                    Ok(store) => store,
                    Err(error) => {
                        let _ = startup.send(Err(error));
                        return Err(error);
                    }
                };
                if startup.send(Ok(())).is_err() {
                    return Ok(());
                }
                run(&mut store, rx, live)
            })
            .map_err(|_| Error::WorkerFault)?;
        let owner = Self {
            client,
            thread: Some(task),
        };
        match ready.await {
            Ok(Ok(())) => Ok(owner),
            failure => {
                // Retain the initialization error and join on every failed open.
                let error = failure.unwrap_or(Err(Error::WorkerFault)).unwrap_err();
                let _ = owner.shutdown().await;
                Err(error)
            }
        }
    }
    pub fn client(&self) -> Client {
        self.client.clone()
    }
    /// Close admission for every cloned handle, drain accepted jobs and join.
    /// OS I/O is not forcibly cancellable; no hard shutdown-latency claim.
    pub async fn shutdown(mut self) -> Result<()> {
        close(&self.client.gate);
        let task = self.thread.take().expect("worker task owned once");
        tokio::task::spawn_blocking(move || task.join().map_err(|_| Error::WorkerFault)?)
            .await
            .map_err(|_| Error::WorkerFault)?
    }
}
impl Drop for Worker {
    // Avoid blocking a Tokio executor in Drop. Explicit shutdown is required to
    // establish joined cleanup; a dropped owner only initiates closing admission.
    fn drop(&mut self) {
        close(&self.client.gate);
    }
}
impl Client {
    /// One passwordless roster per trusted native transport; public network
    /// admission belongs to the bounded prelogin TLS owner.
    pub fn try_trial_local_current(
        &self,
        count: u8,
        ttl: i64,
        idle: Option<i64>,
    ) -> Result<Pending<Vec<Grant>>> {
        self.submit_trial(count, None, ttl, idle, AuthorizationTime::ExecutionUtc)
    }
    pub fn try_trial_local(
        &self,
        count: u8,
        ttl: i64,
        idle: Option<i64>,
        now: i64,
    ) -> Result<Pending<Vec<Grant>>> {
        self.submit_trial(count, None, ttl, idle, AuthorizationTime::Supplied(now))
    }
    pub(crate) fn try_named_guest(
        &self,
        name: String,
        ttl: i64,
        idle: Option<i64>,
    ) -> Result<Pending<Vec<Grant>>> {
        self.submit_trial(1, Some(name), ttl, idle, AuthorizationTime::ExecutionUtc)
    }
    fn submit_trial(
        &self,
        count: u8,
        name: Option<String>,
        ttl: i64,
        idle: Option<i64>,
        now: AuthorizationTime,
    ) -> Result<Pending<Vec<Grant>>> {
        if !(1..=2).contains(&count)
            || !(1..=86400).contains(&ttl)
            || idle.is_some_and(|idle| !(1..=86400).contains(&idle))
            || matches!(now, AuthorizationTime::Supplied(value) if value < 0)
        {
            return Err(Error::InvalidInput);
        }
        let (reply, rx) = oneshot::channel();
        let handoff = Handoff::new();
        let command = Command::LocalLogin {
            request: LocalLoginRequest::Guests(count, name),
            ttl,
            idle,
            now,
            deadline: Instant::now() + self.queue_ttl,
            reply,
            handoff: handoff.clone(),
            handoff_ttl: self.queue_ttl,
        };
        // A fixed separate bucket bounds guest creation even when every request
        // would otherwise supply a fresh principal. The network owner adds its
        // peer/prelogin limits; no attacker-chosen metadata keys this table.
        let mut guard = self.gate.lock().map_err(|_| Error::Closed)?;
        if guard.sender.as_ref().is_none_or(mpsc::Sender::is_closed) {
            return Err(Error::Closed);
        }
        let key = rate::Limiter::key("native-trial-admission");
        let admitted = Instant::now();
        guard.limiter.check(key, admitted)?;
        guard
            .sender
            .as_ref()
            .ok_or(Error::Closed)?
            .try_send(command)
            .map_err(|error| match error {
                mpsc::error::TrySendError::Full(_) => Error::Busy,
                mpsc::error::TrySendError::Closed(_) => Error::Closed,
            })?;
        guard.limiter.commit(key, admitted);
        Ok(Pending {
            reply: Some(rx),
            handoff: Some(handoff),
        })
    }
    /// Trusted transport teardown only. Exact tokens cannot revoke a newer
    /// login, and queue rejection leaves the caller's originals available.
    pub(crate) fn try_end_local(&self, grants: &[Grant]) -> Result<Pending<()>> {
        if !(1..=2).contains(&grants.len()) {
            return Err(Error::InvalidInput);
        }
        let grants = grants
            .iter()
            .map(|grant| {
                Ok(Grant {
                    player: grant.player.clone(),
                    token: Token::parse(grant.token.expose_secret()).map_err(Error::Backend)?,
                    expires_at: grant.expires_at,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let (reply, rx) = oneshot::channel();
        self.submit(Command::EndLocal { grants, reply })?;
        Ok(Pending {
            reply: Some(rx),
            handoff: None,
        })
    }
    /// Revoke only the authenticated caller's token using execution-time UTC.
    pub fn try_logout_current(&self, token: &Token, player: i64) -> Result<Pending<()>> {
        if player < 1 {
            return Err(Error::InvalidInput);
        }
        let token = Token::parse(token.expose_secret()).map_err(Error::Backend)?;
        let (reply, rx) = oneshot::channel();
        self.submit(Command::Logout {
            token,
            player,
            deadline: Instant::now() + self.queue_ttl,
            reply,
        })?;
        Ok(Pending {
            reply: Some(rx),
            handoff: None,
        })
    }
    /// Bounded nonblocking admission. Blocking transactions run only on the
    /// dedicated normal worker, never on the live-authorization lane or room lock.
    /// Dropping queued work cancels it; an already committed event is durable and
    /// must be recovered by the same match key/preference revision, not a new key.
    pub fn try_persist(&self, job: PersistenceJob) -> Result<Pending<PersistenceResult>> {
        let (reply, rx) = oneshot::channel();
        self.submit(Command::Persist {
            job,
            deadline: Instant::now() + self.queue_ttl,
            reply,
        })?;
        Ok(Pending {
            reply: Some(rx),
            handoff: None,
        })
    }
    pub(crate) fn try_begin_instance(&self) -> Result<Pending<i64>> {
        let (reply, rx) = oneshot::channel();
        self.submit(Command::BeginInstance { reply })?;
        Ok(Pending {
            reply: Some(rx),
            handoff: None,
        })
    }
    pub(crate) fn try_manage(
        &self,
        login: String,
        operation: accounts::Management,
    ) -> Result<Pending<Option<Zeroizing<String>>>> {
        if !(3..=32).contains(&login.len())
            || !login
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_')
        {
            return Err(Error::InvalidInput);
        }
        let key = rate::Limiter::key(&login);
        let (reply, rx) = oneshot::channel();
        let now = Instant::now();
        let mut guard = self.gate.lock().map_err(|_| Error::Closed)?;
        guard.limiter.check(key, now)?;
        guard
            .sender
            .as_ref()
            .ok_or(Error::Closed)?
            .try_send(Command::Manage {
                login,
                operation,
                reply,
                deadline: now + self.queue_ttl,
            })
            .map_err(|e| match e {
                mpsc::error::TrySendError::Full(_) => Error::Busy,
                mpsc::error::TrySendError::Closed(_) => Error::Closed,
            })?;
        guard.limiter.commit(key, now);
        Ok(Pending {
            reply: Some(rx),
            handoff: None,
        })
    }
    fn submit(&self, command: Command) -> Result<()> {
        let guard = self.gate.lock().map_err(|_| Error::Closed)?;
        guard
            .sender
            .as_ref()
            .ok_or(Error::Closed)?
            .try_send(command)
            .map_err(|e| match e {
                mpsc::error::TrySendError::Full(_) => Error::Busy,
                mpsc::error::TrySendError::Closed(_) => Error::Closed,
            })
    }
    pub fn is_closed(&self) -> bool {
        self.gate
            .lock()
            .map(|g| g.sender.as_ref().is_none_or(mpsc::Sender::is_closed))
            .unwrap_or(true)
    }
    pub fn try_login(
        &self,
        login: &str,
        password: &[u8],
        ttl: i64,
        now: i64,
    ) -> Result<Pending<Grant>> {
        self.submit_login(login, password, ttl, None, AuthorizationTime::Supplied(now))
    }
    /// One/two-account DB publication only. The transport adapter must bind
    /// both grants to its exact TLS admission before granting membership.
    pub fn try_login_local_current(
        &self,
        credentials: &[(&str, &[u8])],
        ttl: i64,
        idle: Option<i64>,
    ) -> Result<Pending<Vec<Grant>>> {
        self.submit_local_login(credentials, ttl, idle, AuthorizationTime::ExecutionUtc)
    }
    pub fn try_login_local(
        &self,
        credentials: &[(&str, &[u8])],
        ttl: i64,
        idle: Option<i64>,
        now: i64,
    ) -> Result<Pending<Vec<Grant>>> {
        self.submit_local_login(credentials, ttl, idle, AuthorizationTime::Supplied(now))
    }
    fn submit_local_login(
        &self,
        credentials: &[(&str, &[u8])],
        ttl: i64,
        idle: Option<i64>,
        now: AuthorizationTime,
    ) -> Result<Pending<Vec<Grant>>> {
        if !(1..=2).contains(&credentials.len())
            || !(1..=86400).contains(&ttl)
            || idle.is_some_and(|idle| !(1..=86400).contains(&idle))
            || matches!(now, AuthorizationTime::Supplied(value) if value < 0)
        {
            return Err(Error::InvalidInput);
        }
        let mut keys = Vec::with_capacity(credentials.len());
        for (login, password) in credentials {
            if !(3..=32).contains(&login.len())
                || !login
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'_')
                || !(12..=128).contains(&password.len())
            {
                return Err(Error::InvalidInput);
            }
            let key = rate::Limiter::key(login);
            if keys.contains(&key) {
                return Err(Error::InvalidInput);
            }
            keys.push(key);
        }
        let (reply, rx) = oneshot::channel();
        let handoff = Handoff::new();
        let command = Command::LocalLogin {
            request: LocalLoginRequest::Accounts(
                credentials
                    .iter()
                    .map(|(name, password)| {
                        (
                            Zeroizing::new((*name).to_owned()),
                            Zeroizing::new(password.to_vec()),
                        )
                    })
                    .collect(),
            ),
            ttl,
            idle,
            now,
            deadline: Instant::now() + self.queue_ttl,
            reply,
            handoff: handoff.clone(),
            handoff_ttl: self.queue_ttl,
        };
        let mut guard = self.gate.lock().map_err(|_| Error::Closed)?;
        if guard.sender.as_ref().is_none_or(mpsc::Sender::is_closed) {
            return Err(Error::Closed);
        }
        let now = Instant::now();
        guard.limiter.check_group(&keys, now)?;
        guard
            .sender
            .as_ref()
            .ok_or(Error::Closed)?
            .try_send(command)
            .map_err(|error| match error {
                mpsc::error::TrySendError::Full(_) => Error::Busy,
                mpsc::error::TrySendError::Closed(_) => Error::Closed,
            })?;
        for key in keys {
            guard.limiter.commit(key, now);
        }
        Ok(Pending {
            reply: Some(rx),
            handoff: Some(handoff),
        })
    }
    pub fn try_login_with_idle(
        &self,
        login: &str,
        password: &[u8],
        ttl: i64,
        idle: i64,
        now: i64,
    ) -> Result<Pending<Grant>> {
        if !(1..=86400).contains(&idle) {
            return Err(Error::InvalidInput);
        }
        self.submit_login(
            login,
            password,
            ttl,
            Some(idle),
            AuthorizationTime::Supplied(now),
        )
    }
    pub fn try_login_with_idle_current(
        &self,
        login: &str,
        password: &[u8],
        ttl: i64,
        idle: i64,
    ) -> Result<Pending<Grant>> {
        if !(1..=86400).contains(&idle) {
            return Err(Error::InvalidInput);
        }
        self.submit_login(
            login,
            password,
            ttl,
            Some(idle),
            AuthorizationTime::ExecutionUtc,
        )
    }
    fn submit_login(
        &self,
        login: &str,
        password: &[u8],
        ttl: i64,
        idle: Option<i64>,
        now: AuthorizationTime,
    ) -> Result<Pending<Grant>> {
        // Bound all secret/name allocation before admission, including rejected jobs.
        if !(3..=32).contains(&login.len())
            || !login
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_')
            || !(12..=128).contains(&password.len())
            || !(1..=86400).contains(&ttl)
            || matches!(now, AuthorizationTime::Supplied(value) if value < 0)
        {
            return Err(Error::InvalidInput);
        }
        let (reply, rx) = oneshot::channel();
        let handoff = Handoff::new();
        let command = Command::Login {
            login: login.into(),
            password: Zeroizing::new(password.to_vec()),
            ttl,
            idle,
            now,
            deadline: Instant::now() + self.queue_ttl,
            reply,
            handoff: handoff.clone(),
            handoff_ttl: self.queue_ttl,
        };
        let key = rate::Limiter::key(login);
        let now_monotonic = Instant::now();
        let mut guard = self.gate.lock().map_err(|_| Error::Closed)?;
        if guard.sender.as_ref().is_none_or(mpsc::Sender::is_closed) {
            return Err(Error::Closed);
        }
        guard.limiter.check(key, now_monotonic)?;
        guard
            .sender
            .as_ref()
            .ok_or(Error::Closed)?
            .try_send(command)
            .map_err(|e| match e {
                mpsc::error::TrySendError::Full(_) => Error::Busy,
                mpsc::error::TrySendError::Closed(_) => Error::Closed,
            })?;
        guard.limiter.commit(key, now_monotonic);
        Ok(Pending {
            reply: Some(rx),
            handoff: Some(handoff),
        })
    }
    pub fn try_authorize(&self, token: &Token, player: i64, now: i64) -> Result<Pending<Player>> {
        if player < 1 || now < 0 {
            return Err(Error::InvalidInput);
        }
        self.authorize(token, player, AuthorizationTime::Supplied(now), true)
    }
    /// Serving adapters use execution-time UTC, never the timestamp of queue admission.
    /// The existing explicit-time API remains for offline tests/trusted callers.
    pub fn try_authorize_current(&self, token: &Token, player: i64) -> Result<Pending<Player>> {
        if player < 1 {
            return Err(Error::InvalidInput);
        }
        self.authorize(token, player, AuthorizationTime::ExecutionUtc, true)
    }
    /// Background revocation/expiry check; successful inspection does not renew idle TTL.
    pub fn try_inspect_current(&self, token: &Token, player: i64) -> Result<Pending<Player>> {
        if player < 1 {
            return Err(Error::InvalidInput);
        }
        self.authorize(token, player, AuthorizationTime::ExecutionUtc, false)
    }
    fn authorize(
        &self,
        token: &Token,
        player: i64,
        now: AuthorizationTime,
        renew_idle: bool,
    ) -> Result<Pending<Player>> {
        let token = Token::parse(token.expose_secret()).map_err(Error::Backend)?;
        let (reply, rx) = oneshot::channel();
        self.submit(Command::Authorize {
            token,
            player,
            now,
            renew_idle,
            deadline: Instant::now() + self.queue_ttl,
            reply,
        })?;
        Ok(Pending {
            reply: Some(rx),
            handoff: None,
        })
    }
}
fn deliver_login(
    store: &mut Store,
    reply: oneshot::Sender<Result<Grant>>,
    result: Result<Grant>,
    now: i64,
    handoff: &Handoff,
    ttl: Duration,
) -> Result<()> {
    let grant = match result {
        Ok(grant) => grant,
        Err(error) => {
            let _ = reply.send(Err(error));
            return Ok(());
        }
    };
    // Keep one bounded zeroized copy until the caller actually accepts the result;
    // successful oneshot.send alone does not prove the receiver consumed it.
    let retained = Grant {
        player: grant.player.clone(),
        token: Token::parse(grant.token.expose_secret()).map_err(Error::Backend)?,
        expires_at: grant.expires_at,
    };
    let accepted = if reply.send(Ok(grant)).is_ok() {
        handoff.accepted(ttl)
    } else {
        Ok(false)
    };
    if !matches!(accepted, Ok(true)) {
        store
            .release_undelivered(&retained, now)
            .map_err(Error::Backend)?;
    }
    accepted.map(|_| ())
}
fn run(store: &mut Store, mut commands: mpsc::Receiver<Command>, live: bool) -> Result<()> {
    while let Some(command) = commands.blocking_recv() {
        match command {
            Command::Manage {
                login,
                operation,
                deadline,
                reply,
            } => {
                if reply.is_closed() {
                    continue;
                }
                let result = if live {
                    Err(Error::InvalidInput)
                } else if Instant::now() >= deadline {
                    Err(Error::QueueExpired)
                } else {
                    store.manage(&login, operation).map_err(Error::Backend)
                };
                let _ = reply.send(result);
            }
            Command::EndLocal { grants, reply } => {
                // Cleanup must execute even if the waiter disappeared. Its
                // exact-token transaction is also safe after the queue deadline.
                if live {
                    let _ = reply.send(Err(Error::InvalidInput));
                    continue;
                }
                let observed = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .ok()
                    .and_then(|time| i64::try_from(time.as_secs()).ok())
                    .unwrap_or(0);
                let result = store
                    .end_local_session(&grants, observed)
                    .map_err(Error::Backend);
                let _ = reply.send(result);
            }
            Command::LocalLogin {
                request,
                ttl,
                idle,
                now,
                deadline,
                reply,
                handoff,
                handoff_ttl,
            } => {
                if live {
                    let _ = reply.send(Err(Error::InvalidInput));
                    continue;
                }
                if reply.is_closed() {
                    continue;
                }
                if Instant::now() >= deadline {
                    let _ = reply.send(Err(Error::QueueExpired));
                    continue;
                }
                let result = match request {
                    LocalLoginRequest::Accounts(credentials) => {
                        let borrowed: Vec<_> = credentials
                            .iter()
                            .map(|(name, password)| (name.as_str(), password.as_slice()))
                            .collect();
                        // All credential allocations are erased before awaiting
                        // group delivery, for either accepted or abandoned work.
                        match now {
                            AuthorizationTime::Supplied(time) => {
                                store.login_local(&borrowed, ttl, idle, time)
                            }
                            AuthorizationTime::ExecutionUtc => {
                                store.login_local_current(&borrowed, ttl, idle)
                            }
                        }
                    }
                    LocalLoginRequest::Guests(count, name) => match now {
                        AuthorizationTime::Supplied(time) => {
                            store.trial_local(count, ttl, idle, time)
                        }
                        AuthorizationTime::ExecutionUtc => {
                            store.trial_local_current(count, name.as_deref(), ttl, idle)
                        }
                    },
                }
                .map_err(Error::Backend);
                let grants = match result {
                    Ok(grants) => grants,
                    Err(error) => {
                        let _ = reply.send(Err(error));
                        continue;
                    }
                };
                let retained = grants
                    .iter()
                    .map(|grant| {
                        Ok(Grant {
                            player: grant.player.clone(),
                            token: Token::parse(grant.token.expose_secret())
                                .map_err(Error::Backend)?,
                            expires_at: grant.expires_at,
                        })
                    })
                    .collect::<Result<Vec<_>>>()?;
                let accepted = if reply.send(Ok(grants)).is_ok() {
                    handoff.accepted(handoff_ttl)
                } else {
                    Ok(false)
                };
                if !matches!(accepted, Ok(true)) {
                    // The DB compensator clamps to the durable watermark;
                    // a later admin timestamp must not prevent cleanup.
                    store
                        .release_local_undelivered(&retained, 0)
                        .map_err(Error::Backend)?;
                }
                accepted?;
            }
            Command::Logout {
                token,
                player,
                deadline,
                reply,
            } => {
                if live {
                    let _ = reply.send(Err(Error::InvalidInput));
                    continue;
                }
                if reply.is_closed() {
                    continue;
                }
                if Instant::now() >= deadline {
                    let _ = reply.send(Err(Error::QueueExpired));
                    continue;
                }
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .ok()
                    .and_then(|duration| i64::try_from(duration.as_secs()).ok())
                    .ok_or(Error::Backend(accounts::Error::ClockRollback));
                let result =
                    now.and_then(|now| store.logout(&token, player, now).map_err(Error::Backend));
                let _ = reply.send(result);
            }
            Command::Persist {
                job,
                deadline,
                reply,
            } => {
                if live {
                    let _ = reply.send(Err(Error::InvalidInput));
                    continue;
                }
                if reply.is_closed() {
                    continue;
                }
                if Instant::now() >= deadline {
                    let _ = reply.send(Err(Error::QueueExpired));
                    continue;
                }
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .ok()
                    .and_then(|t| i64::try_from(t.as_secs()).ok())
                    .ok_or(Error::Backend(accounts::Error::ClockRollback));
                let result = now.and_then(|now| {
                    match job {
                        PersistenceJob::BeginMatch(start) => {
                            store.begin_match(&start, now).map(PersistenceResult::Match)
                        }
                        PersistenceJob::FinishMatch(key, finish) => store
                            .finish_match(&key, &finish, now)
                            .map(PersistenceResult::Match),
                        PersistenceJob::Character { token, player } => store
                            .character_preference(&token, player, now)
                            .map(PersistenceResult::Character),
                        PersistenceJob::SelectCharacter {
                            token,
                            player,
                            character,
                            expected_revision,
                        } => store
                            .select_character(&token, player, character, expected_revision, now)
                            .map(PersistenceResult::Character),
                    }
                    .map_err(Error::Backend)
                });
                let _ = reply.send(result);
            }
            Command::BeginInstance { reply } => {
                let result = if live {
                    Err(Error::InvalidInput)
                } else {
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .ok()
                        .and_then(|t| i64::try_from(t.as_secs()).ok())
                        .ok_or(Error::Backend(accounts::Error::ClockRollback));
                    now.and_then(|now| {
                        store
                            .begin_instance(
                                &accounts::Admin {
                                    actor: "kr-service",
                                    reason: "fresh database-profile service instance",
                                },
                                now,
                            )
                            .map_err(Error::Backend)
                    })
                };
                let _ = reply.send(result);
            }
            Command::Login {
                login,
                password,
                ttl,
                idle,
                now,
                deadline,
                reply,
                handoff,
                handoff_ttl,
            } => {
                if live {
                    let _ = reply.send(Err(Error::InvalidInput));
                    continue;
                }
                if reply.is_closed() {
                    continue;
                }
                if Instant::now() >= deadline {
                    let _ = reply.send(Err(Error::QueueExpired));
                    continue;
                }
                let result = match (&now, idle) {
                    (AuthorizationTime::Supplied(now), Some(idle)) => {
                        store.login_with_idle(&login, &password, ttl, idle, *now)
                    }
                    (AuthorizationTime::Supplied(now), None) => {
                        store.login(&login, &password, ttl, *now)
                    }
                    (AuthorizationTime::ExecutionUtc, Some(idle)) => {
                        store.login_with_idle_current(&login, &password, ttl, idle)
                    }
                    (AuthorizationTime::ExecutionUtc, None) => {
                        store.login_current(&login, &password, ttl)
                    }
                }
                .map_err(Error::Backend);
                let delivered_at = match now {
                    AuthorizationTime::Supplied(now) => now,
                    AuthorizationTime::ExecutionUtc => std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .ok()
                        .and_then(|time| i64::try_from(time.as_secs()).ok())
                        .ok_or(Error::Backend(accounts::Error::ClockRollback))?,
                };
                deliver_login(store, reply, result, delivered_at, &handoff, handoff_ttl)?;
            }
            Command::Authorize {
                token,
                player,
                now,
                renew_idle,
                deadline,
                reply,
            } => {
                if reply.is_closed() {
                    continue;
                }
                if Instant::now() >= deadline {
                    let _ = reply.send(Err(Error::QueueExpired));
                    continue;
                }
                let result = if live && matches!(now, AuthorizationTime::ExecutionUtc) {
                    store
                        .inspect_live_current(&token, player, renew_idle)
                        .map_err(Error::Backend)
                } else {
                    let now = match now {
                        AuthorizationTime::Supplied(now) => Ok(now),
                        AuthorizationTime::ExecutionUtc => std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .ok()
                            .and_then(|t| i64::try_from(t.as_secs()).ok())
                            .ok_or(Error::Backend(accounts::Error::ClockRollback)),
                    };
                    now.and_then(|now| {
                        if live {
                            store.inspect_live(&token, player, now, renew_idle)
                        } else if renew_idle {
                            store.authorize(&token, player, now)
                        } else {
                            store.inspect_session(&token, player, now)
                        }
                        .map_err(Error::Backend)
                    })
                };
                let _ = reply.send(result);
            }
            #[cfg(test)]
            Command::Pause { entered, resume } => {
                let _ = entered.send(());
                let _ = resume.recv();
            }
            #[cfg(test)]
            Command::Panic => panic!("injected worker fault"),
        }
    }
    Ok(())
}
#[cfg(test)]
pub(crate) mod tests;
