//! Multi-principal ownership for one trusted native transport. This is an
//! internal seam: no public password listener or TLS admission is installed.
pub mod native;
use super::{Error, Result, copy_token, ingame_wire, wire};
use crate::{
    account_worker::Client,
    accounts::{Grant, Player, Token},
};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::watch;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Empty,
    CheckingLogin,
    Login,
    Redirect,
    CheckingChannel,
    Bound,
    Ended,
}

struct State {
    generation: u64,
    phase: Phase,
    grants: Vec<Grant>,
    echoes: Vec<u32>,
    login: u64,
    channel: Option<u64>,
    redirect_until: Option<Instant>,
    stopped: watch::Sender<bool>,
    end_reason: Option<Error>,
}
type Shared = Arc<Mutex<State>>;

const INVALID_SESSION: Error = Error::Backend(crate::account_worker::Error::Backend(
    crate::accounts::Error::InvalidSession,
));

fn retire(shared: &Shared) {
    retire_on(shared, Error::Stale, None);
}

fn retire_on(shared: &Shared, error: Error, stage: Option<&'static str>) -> Error {
    let mut state = shared.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(first) = state.end_reason {
        return first;
    }
    state.end_reason = Some(error);
    state.phase = Phase::Ended;
    state.channel = None;
    state.stopped.send_replace(true);
    drop(state);
    if let Some(stage) = stage {
        eprintln!("{}", authority_end_event(error, stage));
    }
    error
}

fn authority_end_event(error: Error, stage: &'static str) -> serde_json::Value {
    use crate::{account_worker::Error as Worker, accounts::Error as Store};
    let reason = match error {
        INVALID_SESSION => "session_invalid",
        Error::Backend(Worker::Backend(Store::ClockRollback)) => "clock_rollback",
        Error::Backend(Worker::Busy | Worker::Backend(Store::Busy)) => "worker_busy",
        Error::Backend(Worker::QueueExpired) => "queue_expired",
        Error::Backend(Worker::Closed | Worker::WorkerFault) => "worker_unavailable",
        Error::Backend(Worker::Backend(Store::Storage)) => "storage",
        Error::Backend(_) => "backend_denied",
        Error::Denied => "roster_mismatch",
        Error::Stale => "owner_stale",
        _ => "authority_unavailable",
    };
    // The first retiring path reports only two static labels. No tokens,
    // player fields, packet content or underlying error text cross this boundary.
    let mut event = serde_json::json!({"event": "native_account_authority_ended", "stage": stage, "reason": reason});
    if reason == "owner_stale" {
        event["classification"] = serde_json::json!("expected_ttl_or_relogin");
    }
    event
}

struct Cancel(Shared, bool);
impl Drop for Cancel {
    fn drop(&mut self) {
        if self.1 {
            retire(&self.0);
        }
    }
}

/// Owned by the outer trusted transport task, never found by IP, player name,
/// reported address or the small legacy echo. A fresh object has fresh state;
/// old handles cannot become valid again when the numeric socket ID is reused.
pub struct LocalSession {
    client: Client,
    shared: Shared,
    guest_name: Option<String>,
}

#[derive(Clone)]
pub struct LocalAuthority {
    shared: Shared,
    client: Client,
    slot: usize,
    player: Player,
    connection: u64,
}

impl LocalAuthority {
    pub(crate) fn generation(&self) -> u64 {
        self.shared
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .generation
    }
    pub(crate) fn relay_authority(&self) -> super::RelayAuthority {
        super::RelayAuthority::InGame(self.clone())
    }
    pub fn player(&self) -> &Player {
        &self.player
    }
    pub fn connection(&self) -> u64 {
        self.connection
    }
    fn matches(&self, state: &State) -> bool {
        state.phase == Phase::Bound
            && state.channel == Some(self.connection)
            && state
                .grants
                .get(self.slot)
                .is_some_and(|grant| grant.player == self.player)
    }
    pub fn current(&self) -> bool {
        self.ensure_current().is_ok()
    }
    pub fn owns(&self, identity: u32, connection: u64) -> bool {
        self.player.wire_id == identity && self.connection == connection && self.current()
    }
    pub fn stopped(&self) -> watch::Receiver<bool> {
        self.shared
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .stopped
            .subscribe()
    }
    fn ensure_current(&self) -> Result<()> {
        let state = self.shared.lock().map_err(|_| Error::Unavailable)?;
        if let Some(error) = state.end_reason {
            return Err(error);
        }
        if self.matches(&state) {
            Ok(())
        } else {
            Err(Error::Stale)
        }
    }

    /// Inspect both accounts without renewing idle TTL from a background timer.
    /// Any failed member check retires every handle on this transport.
    pub async fn inspect(&self) -> Result<()> {
        self.inspect_at("member_inspect").await
    }

    pub(super) async fn inspect_relay(&self) -> Result<()> {
        self.inspect_at("relay_health").await
    }

    async fn inspect_at(&self, stage: &'static str) -> Result<()> {
        self.ensure_current()?;
        inspect_roster(&self.client, &self.shared)
            .await
            .map_err(|error| retire_on(&self.shared, error, Some(stage)))?;
        self.ensure_current()
    }
    /// Player activity renews only that account after checking the whole roster.
    /// The task must still recheck current() immediately before room I/O.
    pub async fn authorize_activity(&self) -> Result<()> {
        self.inspect().await?;
        let token = {
            let state = self.shared.lock().map_err(|_| Error::Unavailable)?;
            copy_token(&state.grants.get(self.slot).ok_or(Error::Stale)?.token)?
        };
        let result = match self
            .client
            .try_authorize_current(&token, self.player.player_id)
        {
            Ok(pending) => pending.wait().await.map_err(Error::Backend),
            Err(error) => Err(Error::Backend(error)),
        };
        let result = result.and_then(|player| {
            if player == self.player {
                Ok(())
            } else {
                Err(Error::Denied)
            }
        });
        result.map_err(|error| retire_on(&self.shared, error, Some("member_activity")))?;
        self.inspect().await
    }
    pub(crate) async fn character_preference(
        &self,
        selection: Option<(u16, i64)>,
    ) -> Result<crate::accounts::CharacterPreference> {
        use crate::account_worker::{PersistenceJob, PersistenceResult};
        self.inspect().await?;
        let token = {
            let state = self.shared.lock().map_err(|_| Error::Unavailable)?;
            copy_token(&state.grants.get(self.slot).ok_or(Error::Stale)?.token)?
        };
        let player = self.player.player_id;
        let job = match selection {
            Some((character, expected_revision)) => PersistenceJob::SelectCharacter {
                token,
                player,
                character,
                expected_revision,
            },
            None => PersistenceJob::Character { token, player },
        };
        let result = self
            .client
            .try_persist(job)
            .map_err(Error::Backend)?
            .wait()
            .await
            .map_err(Error::Backend)?;
        self.inspect().await?;
        let PersistenceResult::Character(preference) = result else {
            return Err(Error::Unavailable);
        };
        if preference.player_id != player {
            return Err(Error::Denied);
        }
        Ok(preference)
    }
}

async fn inspect_roster(client: &Client, shared: &Shared) -> Result<()> {
    let snapshot: Vec<(Token, Player)> = {
        let state = shared.lock().map_err(|_| Error::Unavailable)?;
        if let Some(error) = state.end_reason {
            return Err(error);
        }
        if state.grants.is_empty() {
            return Err(Error::Stale);
        }
        state
            .grants
            .iter()
            .map(|g| Ok((copy_token(&g.token)?, g.player.clone())))
            .collect::<Result<_>>()?
    };
    for (token, expected) in snapshot {
        let actual = client
            .try_inspect_current(&token, expected.player_id)
            .map_err(Error::Backend)?
            .wait()
            .await
            .map_err(Error::Backend)?;
        if actual != expected {
            return Err(Error::Denied);
        }
        if let Some(error) = shared.lock().map_err(|_| Error::Unavailable)?.end_reason {
            return Err(error);
        }
    }
    Ok(())
}

async fn end_grants(client: Client, grants: Vec<Grant>) -> Result<()> {
    if grants.is_empty() {
        return Ok(());
    }
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let result = match client.try_end_local(&grants) {
            Ok(pending) => pending.wait().await,
            Err(error) => Err(error),
        };
        match result {
            Ok(()) => return Ok(()),
            Err(
                crate::account_worker::Error::Busy
                | crate::account_worker::Error::Backend(crate::accounts::Error::Busy),
            ) if Instant::now() < deadline => tokio::time::sleep(Duration::from_millis(25)).await,
            Err(error) => return Err(Error::Backend(error)),
        }
    }
}

impl LocalSession {
    pub fn new(client: Client, login_connection: u64) -> Result<Self> {
        if login_connection == 0 {
            return Err(Error::InvalidInput);
        }
        let (stopped, _) = watch::channel(false);
        Ok(Self {
            client,
            guest_name: None,
            shared: Arc::new(Mutex::new(State {
                generation: super::fresh_generation()?,
                phase: Phase::Empty,
                grants: Vec::new(),
                echoes: Vec::new(),
                login: login_connection,
                channel: None,
                redirect_until: None,
                stopped,
                end_reason: None,
            })),
        })
    }

    pub(crate) fn with_guest_name(mut self, name: Option<String>) -> Self {
        self.guest_name = name;
        self
    }

    /// The native trial button supplies only a local roster count. The DB
    /// creates fresh passwordless principals; reported metadata grants nothing.
    pub async fn trial(
        &mut self,
        body: &[u8],
        ttl: i64,
        idle: Option<i64>,
    ) -> Result<Vec<(Player, u32)>> {
        let request = ingame_wire::parse(62, body)?;
        let ingame_wire::Players::Trial { count } = request.players else {
            return Err(Error::UnsupportedWire);
        };
        {
            let mut state = self.shared.lock().map_err(|_| Error::Unavailable)?;
            if state.phase != Phase::Empty {
                return Err(Error::Stale);
            }
            state.phase = Phase::CheckingLogin;
        }
        let mut cancel = Cancel(self.shared.clone(), true);
        let pending = match &self.guest_name {
            Some(name) if count == 1 => self.client.try_named_guest(name.clone(), ttl, idle),
            Some(_) => return Err(Error::Denied),
            None => self.client.try_trial_local_current(count, ttl, idle),
        };
        let received = pending
            .map_err(Error::Backend)?
            .receive()
            .await
            .map_err(Error::Backend)?;
        let rows = self.accept_roster(received)?;
        cancel.1 = false;
        Ok(rows)
    }

    /// Native credentials produce DB principals, never fixture identities. The
    /// outer adapter must enable this only on its trusted prelogin transport.
    pub async fn login(
        &mut self,
        body: &[u8],
        ttl: i64,
        idle: Option<i64>,
    ) -> Result<Vec<(Player, u32)>> {
        if self.guest_name.is_some() {
            return Err(Error::Denied);
        }
        let request = ingame_wire::parse(1, body)?;
        let ingame_wire::Players::Accounts(credentials) = request.players else {
            return Err(Error::UnsupportedWire);
        };
        {
            let mut state = self.shared.lock().map_err(|_| Error::Unavailable)?;
            if state.phase != Phase::Empty {
                return Err(Error::Stale);
            }
            state.phase = Phase::CheckingLogin;
        }
        let mut cancel = Cancel(self.shared.clone(), true);
        let borrowed: Vec<_> = credentials
            .iter()
            .map(|c| (c.login(), c.password()))
            .collect();
        let received = self
            .client
            .try_login_local_current(&borrowed, ttl, idle)
            .map_err(Error::Backend)?
            .receive()
            .await
            .map_err(Error::Backend)?;
        drop(borrowed);
        drop(credentials);
        let rows = self.accept_roster(received)?;
        cancel.1 = false;
        Ok(rows)
    }

    fn accept_roster(
        &self,
        received: crate::account_worker::Received<Vec<Grant>>,
    ) -> Result<Vec<(Player, u32)>> {
        let mut echoes = Vec::with_capacity(received.value().len());
        for grant in received.value() {
            kr_wire::kr00967::text949x::encode_name(&grant.player.display_name)
                .map_err(|_| Error::UnsupportedWire)?;
            let mut echo = None;
            for _ in 0..8 {
                let mut bytes = [0; 4];
                getrandom::fill(&mut bytes).map_err(|_| Error::Entropy)?;
                let value = u32::from_le_bytes(bytes);
                if value != 0 && !echoes.contains(&value) {
                    echo = Some(value);
                    break;
                }
            }
            echoes.push(echo.ok_or(Error::Entropy)?);
        }
        let mut state = self.shared.lock().map_err(|_| Error::Unavailable)?;
        if state.phase != Phase::CheckingLogin {
            return Err(Error::Stale);
        }
        state.grants = received.accept().map_err(Error::Backend)?;
        state.echoes = echoes;
        state.phase = Phase::Login;
        Ok(state
            .grants
            .iter()
            .zip(&state.echoes)
            .map(|(g, e)| (g.player.clone(), *e))
            .collect())
    }

    /// Called after the complete redirect write succeeds on this owned login
    /// stream. A packet on a foreign stream cannot advance the handoff.
    pub fn redirected(&mut self, login_connection: u64) -> Result<()> {
        let mut state = self.shared.lock().map_err(|_| Error::Unavailable)?;
        if state.phase != Phase::Login || state.login != login_connection {
            return Err(Error::Stale);
        }
        state.redirect_until = Some(Instant::now() + Duration::from_secs(10));
        state.phase = Phase::Redirect;
        Ok(())
    }

    pub async fn channel(&mut self, body: &[u8], connection: u64) -> Result<Vec<LocalAuthority>> {
        let claims = wire::local_channel_identities(body)?;
        {
            let mut state = self.shared.lock().map_err(|_| Error::Unavailable)?;
            if connection == 0
                || connection == state.login
                || state.phase != Phase::Redirect
                || state
                    .redirect_until
                    .is_none_or(|until| Instant::now() >= until)
                || claims.len() != state.grants.len()
            {
                return Err(Error::Denied);
            }
            for ((claim, grant), echo) in claims.iter().zip(&state.grants).zip(&state.echoes) {
                let name = kr_wire::kr00967::text949x::encode_name(&grant.player.display_name)
                    .map_err(|_| Error::UnsupportedWire)?;
                if claim.wire_id != grant.player.wire_id
                    || claim.name != name
                    || claim.echo != *echo
                {
                    return Err(Error::Denied);
                }
            }
            state.phase = Phase::CheckingChannel;
        }
        let mut cancel = Cancel(self.shared.clone(), true);
        inspect_roster(&self.client, &self.shared).await?;
        let mut state = self.shared.lock().map_err(|_| Error::Unavailable)?;
        if state.phase != Phase::CheckingChannel
            || state
                .redirect_until
                .is_none_or(|until| Instant::now() >= until)
        {
            return Err(Error::Stale);
        }
        state.channel = Some(connection);
        state.phase = Phase::Bound;
        cancel.1 = false;
        Ok(state
            .grants
            .iter()
            .enumerate()
            .map(|(slot, grant)| LocalAuthority {
                shared: self.shared.clone(),
                client: self.client.clone(),
                slot,
                player: grant.player.clone(),
                connection,
            })
            .collect())
    }

    /// Called by the outer transport's bounded timer, including while waiting
    /// for the fresh channel stream. Keepalive traffic is not player activity.
    pub async fn inspect(&mut self) -> Result<()> {
        let expired = {
            let state = self.shared.lock().map_err(|_| Error::Unavailable)?;
            state.phase == Phase::Redirect
                && state
                    .redirect_until
                    .is_none_or(|until| Instant::now() >= until)
        };
        let result = if expired {
            Err(Error::Stale)
        } else {
            inspect_roster(&self.client, &self.shared).await
        };
        result.map_err(|error| retire_on(&self.shared, error, Some("session_inspect")))
    }

    /// Login EOF after a delivered redirect is expected; channel EOF and
    /// pre-redirect login EOF retire both principals. Foreign stream events
    /// cannot cancel this transport's roster.
    pub(crate) fn retire_disconnected(&mut self, connection: u64) -> Result<bool> {
        let owned = {
            let state = self.shared.lock().map_err(|_| Error::Unavailable)?;
            state.channel == Some(connection)
                || (state.login == connection
                    && matches!(state.phase, Phase::CheckingLogin | Phase::Login))
        };
        if owned {
            retire_on(&self.shared, Error::Stale, Some("native_stream_ended"));
        }
        Ok(owned)
    }

    pub async fn disconnected(&mut self, connection: u64) -> Result<bool> {
        if !self.retire_disconnected(connection)? {
            return Ok(false);
        }
        let grants =
            std::mem::take(&mut self.shared.lock().map_err(|_| Error::Unavailable)?.grants);
        end_grants(self.client.clone(), grants).await?;
        Ok(true)
    }

    /// Retire membership synchronously before any DB await. Explicit teardown
    /// joins exact-token cleanup; Drop is a fail-closed cancellation fallback.
    pub async fn finish(self) -> Result<()> {
        retire(&self.shared);
        let grants =
            std::mem::take(&mut self.shared.lock().map_err(|_| Error::Unavailable)?.grants);
        end_grants(self.client.clone(), grants).await
    }
}

impl Drop for LocalSession {
    fn drop(&mut self) {
        retire(&self.shared);
        let grants =
            std::mem::take(&mut self.shared.lock().unwrap_or_else(|e| e.into_inner()).grants);
        if grants.is_empty() {
            return;
        }
        let client = self.client.clone();
        if let Ok(runtime) = tokio::runtime::Handle::try_current() {
            runtime.spawn(async move {
                if end_grants(client, grants).await.is_err() {
                    eprintln!("{{\"event\":\"local_account_cleanup_failed\"}}");
                }
            });
        } else {
            // The worker executes this cleanup even after its waiter drops.
            if client.try_end_local(&grants).is_err() {
                eprintln!("{{\"event\":\"local_account_cleanup_failed\"}}");
            }
        }
    }
}

#[cfg(test)]
pub(crate) mod tests;
