//! Typed bounded account queue; blocking SQLite/KDF work stays off Tokio tasks.
//! No KR socket adapter. Queue expiry is checked before execution, not a timeout
//! or rollback guarantee for an operation already running in SQLite/Argon2.
use crate::accounts::{self, Grant, Player, Store, Token};
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
    pub async fn wait(mut self) -> Result<T> {
        let result = self
            .reply
            .take()
            .expect("reply owned once")
            .await
            .map_err(|_| Error::Closed)?;
        if result.is_ok()
            && let Some(handoff) = &self.handoff
        {
            handoff.claim()?;
        }
        self.handoff = None;
        result
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
type Gate = Arc<Mutex<Option<mpsc::Sender<Command>>>>;
#[derive(Clone)]
pub struct Client {
    gate: Gate,
    queue_ttl: Duration,
}
pub struct Worker {
    client: Client,
    thread: Option<thread::JoinHandle<Result<()>>>,
}

enum Command {
    Login {
        login: String,
        password: Zeroizing<Vec<u8>>,
        ttl: i64,
        now: i64,
        deadline: Instant,
        reply: oneshot::Sender<Result<Grant>>,
        handoff: Arc<Handoff>,
        handoff_ttl: Duration,
    },
    Authorize {
        token: Token,
        player: i64,
        now: i64,
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
    gate.lock().unwrap_or_else(|e| e.into_inner()).take();
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
        if !(1..=64).contains(&capacity)
            || !(Duration::from_millis(1)..=Duration::from_secs(60)).contains(&queue_ttl)
        {
            return Err(Error::InvalidInput);
        }
        let (tx, rx) = mpsc::channel(capacity);
        let gate = Arc::new(Mutex::new(Some(tx)));
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
                run(&mut store, rx)
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
    fn submit(&self, command: Command) -> Result<()> {
        let guard = self.gate.lock().map_err(|_| Error::Closed)?;
        guard
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
            .map(|g| g.as_ref().is_none_or(mpsc::Sender::is_closed))
            .unwrap_or(true)
    }
    pub fn try_login(
        &self,
        login: &str,
        password: &[u8],
        ttl: i64,
        now: i64,
    ) -> Result<Pending<Grant>> {
        // Bound all secret/name allocation before admission, including rejected jobs.
        if !(3..=32).contains(&login.len())
            || !login
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_')
            || !(12..=128).contains(&password.len())
            || !(1..=86400).contains(&ttl)
            || now < 0
        {
            return Err(Error::InvalidInput);
        }
        let (reply, rx) = oneshot::channel();
        let handoff = Handoff::new();
        self.submit(Command::Login {
            login: login.into(),
            password: Zeroizing::new(password.to_vec()),
            ttl,
            now,
            deadline: Instant::now() + self.queue_ttl,
            reply,
            handoff: handoff.clone(),
            handoff_ttl: self.queue_ttl,
        })?;
        Ok(Pending {
            reply: Some(rx),
            handoff: Some(handoff),
        })
    }
    pub fn try_authorize(&self, token: &Token, player: i64, now: i64) -> Result<Pending<Player>> {
        if player < 1 || now < 0 {
            return Err(Error::InvalidInput);
        }
        let token = Token::parse(token.expose_secret()).map_err(Error::Backend)?;
        let (reply, rx) = oneshot::channel();
        self.submit(Command::Authorize {
            token,
            player,
            now,
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
fn run(store: &mut Store, mut commands: mpsc::Receiver<Command>) -> Result<()> {
    while let Some(command) = commands.blocking_recv() {
        match command {
            Command::Login {
                login,
                password,
                ttl,
                now,
                deadline,
                reply,
                handoff,
                handoff_ttl,
            } => {
                if reply.is_closed() {
                    continue;
                }
                if Instant::now() >= deadline {
                    let _ = reply.send(Err(Error::QueueExpired));
                    continue;
                }
                let result = store
                    .login(&login, &password, ttl, now)
                    .map_err(Error::Backend);
                deliver_login(store, reply, result, now, &handoff, handoff_ttl)?;
            }
            Command::Authorize {
                token,
                player,
                now,
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
                let _ = reply.send(store.authorize(&token, player, now).map_err(Error::Backend));
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
mod tests;
