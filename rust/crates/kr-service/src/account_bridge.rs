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
    pub fn token(&self) -> &FrontendToken {
        &self.token
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
    guard: Guard,
}
impl Channel {
    /// Must be called at serving authorization boundaries; alive is not authorization.
    /// Any failure invalidates this generation before returning to the caller.
    pub async fn check(&mut self) -> Result<()> {
        let result = self.authorize().await;
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
    async fn authorize(&self) -> Result<()> {
        let token = {
            let mut r = lock(&self.guard.shared)?;
            copy_token(&self.guard.entry(&mut r)?.token)?
        };
        let current = self
            .client
            .try_authorize_current(&token, self.player.player_id)
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
    shared: Shared,
    ttl: Duration,
    random: fn(&mut [u8]) -> Result<()>,
}
impl Bridge {
    /// Small process-local launch registry. No persistence or multi-process fence.
    pub fn new(client: Client, capacity: usize, ttl: Duration) -> Result<Self> {
        if !(1..=32).contains(&capacity)
            || !(Duration::from_millis(1)..=Duration::from_secs(10)).contains(&ttl)
        {
            return Err(Error::InvalidInput);
        }
        Ok(Self {
            client,
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
    /// Called by a trusted backend/launcher seam using a strong session token.
    /// The caller cannot turn an arbitrary player ID/source IP into ownership.
    pub async fn issue(&self, token: &Token, player: i64, peer: Ipv4Addr) -> Result<Launch> {
        if player < 1 || peer.is_unspecified() || peer.is_multicast() || peer.is_broadcast() {
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
        if !matches!(player.display_name.len(), 5 | 7)
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
        let digest = wire::login_token(body, peer)?.digest();
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
            guard.entry(&mut r)?.echo = Some(echo);
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
            client: self.client.clone(),
            guard,
        })
    }
}

#[cfg(test)]
mod tests;
