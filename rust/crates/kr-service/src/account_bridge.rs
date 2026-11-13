//! Company-designed backend→KR admission seam, NOT recovered KR authentication.
//! Library only: no listener, launcher API, dynamic profile writer or socket adapter.
//! Source: docs/PROTOCOL.md#rust-account-bridge. Field36 is only a correlator.
pub mod wire;
use crate::{
    account_worker::{self, Client},
    accounts::{Player, Token},
};
use sha2::{Digest, Sha256};
use std::{
    collections::{HashMap, HashSet},
    net::Ipv4Addr,
    sync::{Arc, Mutex, MutexGuard},
    time::{Duration, Instant},
};
use zeroize::Zeroizing;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    InvalidInput,
    UnsupportedWire,
    Denied,
    Capacity,
    Stale,
    Unavailable,
    Entropy,
    Exhausted,
    Backend(account_worker::Error),
}
pub type Result<T> = std::result::Result<T, Error>;

/// Separate from the 64-hex backend session. No Debug, Clone or Serialize.
pub struct FrontendToken(Zeroizing<Vec<u8>>);
impl FrontendToken {
    fn from_bytes(bytes: Zeroizing<Vec<u8>>) -> Result<Self> {
        if bytes.len() != 32 || !bytes.iter().all(|b| (33..=126).contains(b)) {
            return Err(Error::UnsupportedWire);
        }
        Ok(Self(bytes))
    }
    /// Explicit delivery to a private launcher, never a log/config diagnostic.
    pub fn expose_secret(&self) -> &str {
        std::str::from_utf8(&self.0).expect("validated ASCII")
    }
    fn digest(&self) -> [u8; 32] {
        Sha256::digest(&self.0).into()
    }
}
fn copy_token(token: &Token) -> Result<Token> {
    Token::parse(token.expose_secret()).map_err(|_| Error::InvalidInput)
}
fn entropy(out: &mut [u8]) -> Result<()> {
    getrandom::fill(out).map_err(|_| Error::Entropy)
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Issuing,
    Issued,
    CheckingLogin,
    Login,
    Redirect,
    CheckingChannel,
    Bound,
}
struct Entry {
    generation: u64,
    phase: Phase,
    token: Token,
    player: Option<Player>,
    frontend: Option<[u8; 32]>,
    echo: Option<u32>,
    peer: Ipv4Addr,
    reported: Option<Ipv4Addr>,
    connection: Option<u64>,
    deadline: Instant,
}
struct Registry {
    entries: HashMap<i64, Entry>,
    generation: u64,
    seen_echo: HashSet<u32>,
    capacity: usize,
}
impl Registry {
    fn sweep(&mut self) {
        let now = Instant::now();
        self.entries
            .retain(|_, e| e.phase == Phase::Bound || now < e.deadline);
    }
    fn connection_in_use(&self, connection: u64) -> bool {
        self.entries
            .values()
            .any(|e| e.connection == Some(connection))
    }
}
type Shared = Arc<Mutex<Registry>>;
fn lock(shared: &Shared) -> Result<MutexGuard<'_, Registry>> {
    shared.lock().map_err(|_| Error::Unavailable)
}
struct Guard {
    shared: Shared,
    player: i64,
    generation: u64,
    phase: Phase,
    armed: bool,
}
impl Guard {
    fn entry<'a>(&self, registry: &'a mut Registry) -> Result<&'a mut Entry> {
        let entry = registry.entries.get_mut(&self.player).ok_or(Error::Stale)?;
        if entry.generation != self.generation
            || entry.phase != self.phase
            || (entry.phase != Phase::Bound && Instant::now() >= entry.deadline)
        {
            return Err(Error::Stale);
        }
        Ok(entry)
    }
    fn transition(&mut self, next: Phase) -> Result<()> {
        self.entry(&mut *lock(&self.shared)?)?.phase = next;
        self.phase = next;
        Ok(())
    }
}
impl Drop for Guard {
    fn drop(&mut self) {
        if self.armed {
            let mut r = self.shared.lock().unwrap_or_else(|e| e.into_inner());
            if r.entries
                .get(&self.player)
                .is_some_and(|e| e.generation == self.generation && e.phase == self.phase)
            {
                r.entries.remove(&self.player);
            }
        }
    }
}
/// Keep until delivered/claimed or dropped. Drop removes only its own issued generation.
pub struct Launch {
    pub player: Player,
    token: FrontendToken,
    _guard: Guard,
}
impl Launch {
    pub(crate) fn generation(&self) -> u64 {
        self._guard.generation
    }
    pub fn token(&self) -> &FrontendToken {
        &self.token
    }
    /// Transfer an issued launch to its bounded registry only after private delivery.
    /// A send failure must drop the handle instead, cancelling the issued generation.
    pub fn delivered(mut self) {
        self._guard.armed = false;
    }
}
/// Kept by a login socket until the actual redirect write succeeds.
pub struct Login {
    pub player: Player,
    pub echo: u32,
    guard: Guard,
}
impl Login {
    /// Call only after successful redirect delivery. Dropping before this cancels.
    /// An unclaimed redirect is retained at most the original launch TTL.
    pub fn redirected(mut self) -> Result<()> {
        self.guard.transition(Phase::Redirect)?;
        self.guard.armed = false;
        Ok(())
    }
}
/// Owns one channel generation. Drop/failed check does not revoke the backend session.
pub struct Channel {
    pub player: Player,
    pub connection: u64,
    client: Client,
    persistence: Client,
    guard: Guard,
}
/// Token-free handle to an existing bound channel; never issues a session.
/// Relay must inspect all participants, then recheck their generations before I/O.
#[derive(Clone)]
pub(crate) struct RelayAuthority {
    shared: Shared,
    player: Player,
    generation: u64,
    connection: u64,
    client: Client,
}
impl RelayAuthority {
    pub(crate) fn player(&self) -> &Player {
        &self.player
    }
    pub(crate) fn generation(&self) -> u64 {
        self.generation
    }
    pub(crate) fn owns(&self, identity: u32, connection: u64) -> bool {
        self.player.wire_id == identity && self.connection == connection && self.current()
    }
    pub(crate) fn current(&self) -> bool {
        lock(&self.shared).is_ok_and(|r| {
            r.entries.get(&self.player.player_id).is_some_and(|e| {
                e.generation == self.generation
                    && e.phase == Phase::Bound
                    && e.connection == Some(self.connection)
                    && e.player.as_ref() == Some(&self.player)
            })
        })
    }
    pub(crate) async fn check_idle(&self) -> Result<()> {
        let result = self.inspect().await;
        if result.is_err() {
            let mut r = self.shared.lock().unwrap_or_else(|e| e.into_inner());
            if r.entries
                .get(&self.player.player_id)
                .is_some_and(|e| e.generation == self.generation)
            {
                r.entries.remove(&self.player.player_id);
            }
        }
        result
    }
    async fn inspect(&self) -> Result<()> {
        let token = {
            let r = lock(&self.shared)?;
            let entry = r.entries.get(&self.player.player_id).ok_or(Error::Stale)?;
            if entry.generation != self.generation
                || entry.phase != Phase::Bound
                || entry.connection != Some(self.connection)
                || entry.player.as_ref() != Some(&self.player)
            {
                return Err(Error::Stale);
            }
            copy_token(&entry.token)?
        };
        let player = self
            .client
            .try_inspect_current(&token, self.player.player_id)
            .map_err(Error::Backend)?
            .wait()
            .await
            .map_err(Error::Backend)?;
        if player != self.player || !self.current() {
            return Err(Error::Denied);
        }
        Ok(())
    }
}
impl Channel {
    pub(crate) fn relay_authority(&self) -> RelayAuthority {
        RelayAuthority {
            shared: self.guard.shared.clone(),
            player: self.player.clone(),
            generation: self.guard.generation,
            connection: self.connection,
            client: self.client.clone(),
        }
    }
    /// Load a DB preference only for this current frontend/channel generation.
    /// This is an internal seam, not a KR profile field or character entitlement.
    pub async fn character_preference(&self) -> Result<crate::accounts::CharacterPreference> {
        self.preference(None).await
    }
    pub async fn select_character(
        &self,
        character: u16,
        expected_revision: i64,
    ) -> Result<crate::accounts::CharacterPreference> {
        self.preference(Some((character, expected_revision))).await
    }
    async fn preference(
        &self,
        selection: Option<(u16, i64)>,
    ) -> Result<crate::accounts::CharacterPreference> {
        let token = {
            let mut registry = lock(&self.guard.shared)?;
            copy_token(&self.guard.entry(&mut registry)?.token)?
        };
        let player = self.player.player_id;
        let job = match selection {
            Some((character, expected_revision)) => {
                account_worker::PersistenceJob::SelectCharacter {
                    token,
                    player,
                    character,
                    expected_revision,
                }
            }
            None => account_worker::PersistenceJob::Character { token, player },
        };
        let result = self
            .persistence
            .try_persist(job)
            .map_err(Error::Backend)?
            .wait()
            .await
            .map_err(Error::Backend)?;
        let account_worker::PersistenceResult::Character(preference) = result else {
            return Err(Error::Unavailable);
        };
        // A committed preference may outlive a dropped result. Never project its
        // response onto a newer frontend generation or a revoked DB session.
        self.authorize(false).await?;
        if preference.player_id != player {
            return Err(Error::Denied);
        }
        Ok(preference)
    }
    /// Must be called at serving authorization boundaries; alive is not authorization.
    /// Any failure invalidates this generation before returning to the caller.
    pub async fn check(&mut self) -> Result<()> {
        self.verify(true).await
    }
    /// A server-side timer checks expiry/revoke without renewing player idle activity.
    pub async fn check_idle(&mut self) -> Result<()> {
        self.verify(false).await
    }
    async fn verify(&mut self, renew_idle: bool) -> Result<()> {
        let result = self.authorize(renew_idle).await;
        if result.is_err() {
            let mut r = self.guard.shared.lock().unwrap_or_else(|e| e.into_inner());
            if r.entries
                .get(&self.guard.player)
                .is_some_and(|e| e.generation == self.guard.generation)
            {
                r.entries.remove(&self.guard.player);
            }
        }
        result
    }
    async fn authorize(&self, renew_idle: bool) -> Result<()> {
        let token = {
            let mut r = lock(&self.guard.shared)?;
            copy_token(&self.guard.entry(&mut r)?.token)?
        };
        let pending = if renew_idle {
            self.client
                .try_authorize_current(&token, self.player.player_id)
        } else {
            self.client
                .try_inspect_current(&token, self.player.player_id)
        };
        let current = pending
            .map_err(Error::Backend)?
            .wait()
            .await
            .map_err(Error::Backend)?;
        let mut r = lock(&self.guard.shared)?;
        let entry = self.guard.entry(&mut r)?;
        if current != self.player || entry.player.as_ref() != Some(&current) {
            return Err(Error::Denied);
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct Bridge {
    client: Client,
    live: Option<Client>,
    shared: Shared,
    ttl: Duration,
    random: fn(&mut [u8]) -> Result<()>,
}
impl Bridge {
    /// Companion capability possession is checked separately from current
    /// account authority. Recheck after DB await before registering a peer.
    pub(crate) async fn validate_transport(&self, player: u32, generation: u64) -> Result<()> {
        let (token, expected) = {
            let mut r = lock(&self.shared)?;
            r.sweep();
            let entry = r
                .entries
                .values()
                .find(|e| {
                    e.generation == generation
                        && e.player.as_ref().is_some_and(|p| p.wire_id == player)
                })
                .ok_or(Error::Denied)?;
            (
                copy_token(&entry.token)?,
                entry.player.clone().ok_or(Error::Denied)?,
            )
        };
        let actual = self
            .live
            .as_ref()
            .unwrap_or(&self.client)
            .try_inspect_current(&token, expected.player_id)
            .map_err(Error::Backend)?
            .wait()
            .await
            .map_err(Error::Backend)?;
        let mut r = lock(&self.shared)?;
        r.sweep();
        if actual != expected
            || r.entries
                .get(&expected.player_id)
                .is_none_or(|e| e.generation != generation || e.player.as_ref() != Some(&expected))
        {
            return Err(Error::Denied);
        }
        Ok(())
    }
    /// Small process-local launch registry. No persistence or multi-process fence.
    pub fn new(client: Client, capacity: usize, ttl: Duration) -> Result<Self> {
        if !(1..=128).contains(&capacity)
            || !(Duration::from_millis(1)..=Duration::from_secs(120)).contains(&ttl)
        {
            return Err(Error::InvalidInput);
        }
        Ok(Self {
            client,
            live: None,
            shared: Arc::new(Mutex::new(Registry {
                entries: HashMap::new(),
                generation: 0,
                seen_echo: HashSet::new(),
                capacity,
            })),
            ttl,
            random: entropy,
        })
    }
    pub(crate) fn with_live_authorization(mut self, client: Client) -> Self {
        self.live = Some(client);
        self
    }
    /// Called by a trusted backend/launcher seam using a strong session token.
    /// The caller cannot turn an arbitrary player ID/source IP into ownership.
    pub async fn issue(&self, token: &Token, player: i64, peer: Ipv4Addr) -> Result<Launch> {
        self.issue_metadata(token, player, peer, None).await
    }
    /// Private authenticated launcher metadata. It never selects backend identity.
    pub(crate) async fn issue_metadata(
        &self,
        token: &Token,
        player: i64,
        peer: Ipv4Addr,
        reported: Option<Ipv4Addr>,
    ) -> Result<Launch> {
        if player < 1 || peer.is_unspecified() || peer.is_multicast() || peer.is_broadcast() {
            return Err(Error::InvalidInput);
        }
        if reported.is_some_and(|ip| ip.is_unspecified() || ip.is_multicast() || ip.is_broadcast())
        {
            return Err(Error::InvalidInput);
        }
        let token = copy_token(token)?;
        let mut guard = {
            let mut r = lock(&self.shared)?;
            r.sweep();
            if r.entries.contains_key(&player) {
                return Err(Error::Denied);
            }
            if r.entries.len() >= r.capacity {
                return Err(Error::Capacity);
            }
            r.generation = r.generation.checked_add(1).ok_or(Error::Exhausted)?;
            let generation = r.generation;
            r.entries.insert(
                player,
                Entry {
                    generation,
                    phase: Phase::Issuing,
                    token,
                    player: None,
                    frontend: None,
                    echo: None,
                    peer,
                    reported,
                    connection: None,
                    deadline: Instant::now() + self.ttl,
                },
            );
            Guard {
                shared: self.shared.clone(),
                player,
                generation,
                phase: Phase::Issuing,
                armed: true,
            }
        };
        let player = self.authorize(&guard).await?;
        if !(3..=15).contains(&player.display_name.len())
            || player.wire_id == 0
            || !player
                .display_name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_')
        {
            return Err(Error::UnsupportedWire);
        }
        let mut raw = Zeroizing::new([0u8; 16]);
        (self.random)(raw.as_mut())?;
        let mut encoded = Zeroizing::new(Vec::with_capacity(32));
        for b in raw.iter() {
            encoded.push(b"0123456789abcdef"[(b >> 4) as usize]);
            encoded.push(b"0123456789abcdef"[(b & 15) as usize]);
        }
        let token = FrontendToken::from_bytes(encoded)?;
        {
            let mut r = lock(&self.shared)?;
            let digest = token.digest();
            if r.entries.values().any(|e| e.frontend == Some(digest)) {
                return Err(Error::Entropy);
            }
            let entry = guard.entry(&mut r)?;
            entry.frontend = Some(digest);
            entry.player = Some(player.clone());
        }
        guard.transition(Phase::Issued)?;
        Ok(Launch {
            player,
            token,
            _guard: guard,
        })
    }
    async fn authorize(&self, guard: &Guard) -> Result<Player> {
        let (token, expected) = {
            let mut r = lock(&self.shared)?;
            let entry = guard.entry(&mut r)?;
            (copy_token(&entry.token)?, entry.player.clone())
        };
        let player = self
            .client
            .try_authorize_current(&token, guard.player)
            .map_err(Error::Backend)?
            .wait()
            .await
            .map_err(Error::Backend)?;
        guard.entry(&mut *lock(&self.shared)?)?;
        if expected
            .as_ref()
            .is_some_and(|expected| *expected != player)
        {
            return Err(Error::Denied);
        }
        Ok(player)
    }
    pub async fn login(&self, body: &[u8], peer: Ipv4Addr, connection: u64) -> Result<Login> {
        self.login_with_reported_address(body, peer, peer, connection)
            .await
    }
    /// Controlled profile metadata can differ after NAT. The token's owner,
    /// lease and connection remain bound to the actual transport `peer` below.
    pub(crate) async fn login_with_reported_address(
        &self,
        body: &[u8],
        peer: Ipv4Addr,
        reported: Ipv4Addr,
        connection: u64,
    ) -> Result<Login> {
        self.login_for_profile(body, peer, connection, |_| Some(reported))
            .await
    }
    /// The token-selected issued player chooses the expected metadata address.
    /// A shared transport IPv4 must never choose a different player's profile.
    pub(crate) async fn login_for_profile(
        &self,
        body: &[u8],
        peer: Ipv4Addr,
        connection: u64,
        expected_address: impl FnOnce(&Player) -> Option<Ipv4Addr>,
    ) -> Result<Login> {
        let reported = wire::reported_address(body)?;
        let digest = wire::login_token(body, reported)?.digest();
        if connection == 0 {
            return Err(Error::InvalidInput);
        }
        let mut guard = {
            let mut r = lock(&self.shared)?;
            r.sweep();
            if r.connection_in_use(connection) {
                return Err(Error::Denied);
            }
            let (&player, entry) = r
                .entries
                .iter_mut()
                .find(|(_, e)| e.frontend == Some(digest))
                .ok_or(Error::Denied)?;
            if entry.phase != Phase::Issued || entry.peer != peer {
                return Err(Error::Denied);
            }
            let owner = entry.player.as_ref().ok_or(Error::Denied)?;
            let expected = entry.reported.or_else(|| expected_address(owner));
            if expected != Some(reported) {
                return Err(Error::Denied);
            }
            entry.frontend = None; // One use, even if subsequent DB authorization fails.
            entry.phase = Phase::CheckingLogin;
            entry.connection = Some(connection);
            Guard {
                shared: self.shared.clone(),
                player,
                generation: entry.generation,
                phase: Phase::CheckingLogin,
                armed: true,
            }
        };
        let player = self.authorize(&guard).await?;
        let echo = {
            let mut r = lock(&self.shared)?;
            guard.entry(&mut r)?;
            if r.seen_echo.len() >= 1024 {
                return Err(Error::Exhausted);
            }
            let mut echo = None;
            for _ in 0..8 {
                let mut bytes = [0u8; 4];
                (self.random)(&mut bytes)?;
                let candidate = u32::from_le_bytes(bytes);
                if candidate != 0 && r.seen_echo.insert(candidate) {
                    echo = Some(candidate);
                    break;
                }
            }
            let echo = echo.ok_or(Error::Entropy)?;
            let entry = guard.entry(&mut r)?;
            entry.echo = Some(echo);
            // Launcher startup has its own lifetime; only the authorized login
            // starts the shorter legacy channel handoff window.
            entry.deadline = Instant::now() + Duration::from_secs(10);
            echo
        };
        guard.transition(Phase::Login)?;
        Ok(Login {
            player,
            echo,
            guard,
        })
    }
    pub async fn channel(&self, body: &[u8], peer: Ipv4Addr, connection: u64) -> Result<Channel> {
        let identity = wire::channel_identity(body)?;
        if connection == 0 {
            return Err(Error::InvalidInput);
        }
        let mut guard = {
            let mut r = lock(&self.shared)?;
            r.sweep();
            if r.connection_in_use(connection) {
                return Err(Error::Denied);
            }
            let (&player, entry) = r
                .entries
                .iter_mut()
                .find(|(_, e)| e.echo == Some(identity.echo))
                .ok_or(Error::Denied)?;
            let owned = entry.player.as_ref().ok_or(Error::Denied)?;
            if entry.phase != Phase::Redirect
                || entry.peer != peer
                || owned.wire_id != identity.wire_id
                || owned.display_name.as_bytes() != identity.name
            {
                return Err(Error::Denied);
            }
            entry.phase = Phase::CheckingChannel;
            entry.connection = Some(connection);
            entry.echo = None;
            Guard {
                shared: self.shared.clone(),
                player,
                generation: entry.generation,
                phase: Phase::CheckingChannel,
                armed: true,
            }
        };
        let player = self.authorize(&guard).await?;
        guard.transition(Phase::Bound)?;
        Ok(Channel {
            player,
            connection,
            client: self.live.as_ref().unwrap_or(&self.client).clone(),
            persistence: self.client.clone(),
            guard,
        })
    }
}

#[cfg(test)]
mod tests;
