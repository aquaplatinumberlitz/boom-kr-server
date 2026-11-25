//! Local F4-d HTTP API. Player identity comes only from a trusted token issuer.
mod http;
#[cfg(test)]
mod tests;

use crate::{
    account_worker::{AccountService, Error},
    accounts::{self, CatalogAccess, PlayerProfile, ShopCategory, ShopItem, UsableItem},
};
use http::{Request, Response, Status};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    fmt::Write as _,
    hint::black_box,
    io,
    net::{IpAddr, Ipv4Addr, SocketAddrV4},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::{net::TcpListener, sync::watch, task::JoinSet, time::timeout};
use zeroize::Zeroizing;

const TOKEN_CAPACITY: usize = 1024;
const TOKEN_HEX_LEN: usize = 64;
const PAGE: &str = include_str!("web_shop/index.html");

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub bind: SocketAddrV4,
    pub allowed_peers: Vec<Ipv4Addr>,
    pub max_connections: usize,
    pub request_timeout_ms: u64,
    pub max_body_bytes: usize,
    pub session_ttl_seconds: i64,
    pub unlock_all: bool,
}

impl Config {
    pub fn validate(&self) -> io::Result<()> {
        fn concrete(ip: Ipv4Addr) -> bool {
            !ip.is_unspecified() && !ip.is_multicast() && !ip.is_broadcast()
        }
        if self.bind.port() == 0
            || !concrete(*self.bind.ip())
            || self.allowed_peers.is_empty()
            || !self.allowed_peers.iter().copied().all(concrete)
            || !(1..=64).contains(&self.max_connections)
            || !(250..=10_000).contains(&self.request_timeout_ms)
            || !(256..=16_384).contains(&self.max_body_bytes)
        {
            return Err(invalid("invalid web shop config"));
        }
        session_ttl(self.session_ttl_seconds)?;
        Ok(())
    }

    fn allows(&self, ip: IpAddr) -> bool {
        matches!(ip, IpAddr::V4(ip) if self.allowed_peers.contains(&ip))
    }

    fn mode(&self) -> &'static str {
        if self.unlock_all {
            "unlock_all"
        } else {
            "standard"
        }
    }
}

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}

fn session_ttl(seconds: i64) -> io::Result<Duration> {
    if !(1..=86_400).contains(&seconds) {
        return Err(invalid("invalid web shop session lifetime"));
    }
    Ok(Duration::from_secs(seconds as u64))
}

/// Secret held only for handoff to the intended client. Deliberately has no
/// Debug, Display or serialization implementation.
pub struct WebShopToken(Zeroizing<String>);

impl WebShopToken {
    /// Borrow for a private handoff or an Authorization header. Do not log it.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

type TokenDigest = Zeroizing<[u8; 32]>;

struct Session {
    digest: TokenDigest,
    player_id: i64,
    expires_at: Instant,
}

/// The host issues tokens only after binding an authenticated player. Clones
/// share one process-local table; revoke on logout or player removal.
#[derive(Clone)]
pub struct WebShopAuth {
    ttl: Duration,
    sessions: Arc<Mutex<Vec<Session>>>,
}

impl WebShopAuth {
    pub fn new(session_ttl_seconds: i64) -> io::Result<Self> {
        Ok(Self {
            ttl: session_ttl(session_ttl_seconds)?,
            sessions: Arc::new(Mutex::new(Vec::new())),
        })
    }

    pub fn issue_token(&self, player_id: i64) -> io::Result<WebShopToken> {
        if player_id < 1 {
            return Err(invalid("invalid web shop player"));
        }
        let mut sessions = self
            .sessions
            .lock()
            .map_err(|_| io::Error::other("web shop sessions unavailable"))?;
        let now = Instant::now();
        sessions.retain(|s| s.expires_at > now);
        if sessions.len() >= TOKEN_CAPACITY {
            return Err(io::Error::other("web shop session table full"));
        }
        let mut bytes = Zeroizing::new([0u8; 32]);
        getrandom::fill(bytes.as_mut())
            .map_err(|_| io::Error::other("web shop entropy unavailable"))?;
        let mut token = Zeroizing::new(String::with_capacity(TOKEN_HEX_LEN));
        for byte in bytes.iter() {
            write!(&mut *token, "{byte:02x}").expect("String formatting");
        }
        sessions.push(Session {
            digest: token_digest(&token),
            player_id,
            expires_at: now + self.ttl,
        });
        Ok(WebShopToken(token))
    }

    pub fn revoke_player(&self, player_id: i64) {
        if let Ok(mut sessions) = self.sessions.lock() {
            sessions.retain(|s| s.player_id != player_id);
        }
    }

    pub fn revoke_token(&self, token: &WebShopToken) {
        let digest = token_digest(token.as_str());
        if let Ok(mut sessions) = self.sessions.lock() {
            sessions.retain(|s| !digest_eq(&s.digest, &digest));
        }
    }

    fn player(&self, digest: &TokenDigest) -> Option<i64> {
        let sessions = self.sessions.lock().ok()?;
        let now = Instant::now();
        let mut player = None;
        for session in sessions.iter() {
            if digest_eq(&session.digest, digest) && session.expires_at > now {
                player = Some(session.player_id);
            }
        }
        player
    }
}

fn token_digest(token: &str) -> TokenDigest {
    Zeroizing::new(Sha256::digest(token.as_bytes()).into())
}

fn digest_eq(a: &[u8; 32], b: &[u8; 32]) -> bool {
    // Accumulate every byte; keep the optimizer from rewriting this as memcmp.
    let mut difference = 0;
    for (&a, &b) in a.iter().zip(b) {
        difference |= black_box(a ^ b);
    }
    difference == 0
}

pub(crate) struct Opened {
    listener: TcpListener,
    config: Config,
    auth: WebShopAuth,
}

impl Opened {
    pub(crate) fn auth(&self) -> WebShopAuth {
        self.auth.clone()
    }
}

pub(crate) async fn open(config: Config) -> io::Result<Opened> {
    config.validate()?;
    let auth = WebShopAuth::new(config.session_ttl_seconds)?;
    let listener = TcpListener::bind(config.bind).await?;
    Ok(Opened {
        listener,
        config,
        auth,
    })
}

/// Bind the configured IPv4 interface. Dropping this future closes the listener
/// and aborts connection tasks. A DB mutation already running may still commit.
pub async fn serve(config: Config, service: AccountService, auth: WebShopAuth) -> io::Result<()> {
    config.validate()?;
    if session_ttl(config.session_ttl_seconds)? != auth.ttl {
        return Err(invalid("web shop session lifetime mismatch"));
    }
    let listener = TcpListener::bind(config.bind).await?;
    serve_loop(listener, config, service, auth, None).await
}

pub(crate) async fn serve_until(
    opened: Opened,
    service: AccountService,
    stop: watch::Receiver<bool>,
) -> io::Result<()> {
    serve_loop(
        opened.listener,
        opened.config,
        service,
        opened.auth,
        Some(stop),
    )
    .await
}

#[cfg(test)]
async fn serve_listener(
    listener: TcpListener,
    config: Config,
    service: AccountService,
    auth: WebShopAuth,
) -> io::Result<()> {
    serve_loop(listener, config, service, auth, None).await
}

async fn stop_signal(stop: Option<&mut watch::Receiver<bool>>) {
    match stop {
        Some(stop) => {
            if !*stop.borrow() {
                let _ = stop.changed().await;
            }
        }
        None => std::future::pending::<()>().await,
    }
}

async fn serve_loop(
    listener: TcpListener,
    config: Config,
    service: AccountService,
    auth: WebShopAuth,
    mut stop: Option<watch::Receiver<bool>>,
) -> io::Result<()> {
    let config = Arc::new(config);
    let mut connections = JoinSet::new();
    loop {
        if stop.as_ref().is_some_and(|s| *s.borrow()) {
            break;
        }
        tokio::select! {
            biased;
            _ = stop_signal(stop.as_mut()) => break,
            _ = connections.join_next(), if !connections.is_empty() => {}
            accepted = listener.accept() => {
                let (mut stream, peer) = accepted?;
                if !config.allows(peer.ip()) {
                    continue;
                }
                while connections.try_join_next().is_some() {}
                if connections.len() >= config.max_connections {
                    continue;
                }
                let (config, service, auth) = (config.clone(), service.clone(), auth.clone());
                connections.spawn(async move {
                    let _ = timeout(Duration::from_millis(config.request_timeout_ms), async {
                        let response = match http::read(&mut stream, config.max_body_bytes).await {
                            Ok(request) => dispatch(&config, &service, &auth, request).await,
                            Err(Some(status)) => Response::error(status),
                            Err(None) => return Ok(()),
                        };
                        response.write(&mut stream).await
                    }).await;
                });
            }
        }
    }
    drop(listener);
    connections.abort_all();
    while connections.join_next().await.is_some() {}
    Ok(())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Purchase {
    sku: String,
    quantity: Option<u32>,
    request_id: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Equip {
    sku: String,
    slot: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Unequip {
    slot: String,
}

#[derive(Serialize)]
struct Profile {
    #[serde(flatten)]
    profile: PlayerProfile,
    mode: &'static str,
}

#[derive(Serialize)]
struct CatalogItem<'a> {
    #[serde(flatten)]
    item: &'a ShopItem,
    usable: bool,
    quantity: Option<i64>,
    expires_at: Option<i64>,
    unlocked_by_session: bool,
}

fn category(query: Option<&str>) -> Result<Option<ShopCategory>, Status> {
    Ok(Some(match query {
        None => return Ok(None),
        Some("category=character") => ShopCategory::Character,
        Some("category=decoration") => ShopCategory::Decoration,
        Some("category=emotion") => ShopCategory::Emotion,
        Some("category=consumable") => ShopCategory::Consumable,
        Some("category=collection") => ShopCategory::Collection,
        _ => return Err(Status::BadRequest),
    }))
}

async fn catalog(
    config: &Config,
    service: &AccountService,
    player: i64,
    query: Option<&str>,
) -> Result<Response, Status> {
    let filter = category(query)?;
    let usable = service
        .usable_items(
            player,
            CatalogAccess {
                unlock_all: config.unlock_all,
            },
        )
        .await?;
    let usable: HashMap<&str, &UsableItem> =
        usable.iter().map(|i| (i.sku_id.as_str(), i)).collect();
    let items = service
        .settings()
        .catalog()
        .items
        .iter()
        .filter(|i| i.enabled && filter.is_none_or(|c| c == i.category))
        .map(|item| {
            let access = usable.get(item.sku_id.as_str());
            CatalogItem {
                item,
                usable: access.is_some(),
                quantity: access.and_then(|i| i.quantity),
                expires_at: access.and_then(|i| i.expires_at),
                unlocked_by_session: access.is_some_and(|i| i.unlocked_by_session),
            }
        })
        .collect::<Vec<_>>();
    #[derive(Serialize)]
    struct Catalog<'a> {
        mode: &'static str,
        items: Vec<CatalogItem<'a>>,
    }
    Ok(Response::json(&Catalog {
        mode: config.mode(),
        items,
    }))
}

async fn dispatch(
    config: &Config,
    service: &AccountService,
    auth: &WebShopAuth,
    request: Request,
) -> Response {
    match route(config, service, auth, &request).await {
        Ok(response) => response,
        Err(status) => Response::error(status),
    }
}

async fn route(
    config: &Config,
    service: &AccountService,
    auth: &WebShopAuth,
    request: &Request,
) -> Result<Response, Status> {
    let path = request.path();
    let query = request.query();
    if matches!(path, "/" | "/health") {
        request.require_method("GET")?;
        if query.is_some() {
            return Err(Status::BadRequest);
        }
        return Ok(if path == "/" {
            Response::html(PAGE)
        } else {
            Response::json(&json!({
                "ok": true,
                "mode": config.mode(),
                "catalog_items": service.settings().catalog().items.len(),
                "rewards_enabled": service.settings().rewards_enabled(),
            }))
        });
    }

    if !path.starts_with("/api/v1/") {
        return Err(Status::NotFound);
    }
    let player = request
        .digest()
        .and_then(|digest| auth.player(&digest))
        .ok_or(Status::Unauthorized)?;

    if path != "/api/v1/catalog" && query.is_some() {
        return Err(Status::BadRequest);
    }
    match path {
        "/api/v1/profile" => {
            request.require_method("GET")?;
            let profile = service.profile_get(player).await?;
            Ok(Response::json(&Profile {
                profile,
                mode: config.mode(),
            }))
        }
        "/api/v1/catalog" => {
            request.require_method("GET")?;
            catalog(config, service, player, query).await
        }
        "/api/v1/inventory" => {
            request.require_method("GET")?;
            #[derive(Serialize)]
            struct Inventory {
                inventory: Vec<accounts::InventoryItem>,
                equipped: Vec<accounts::EquippedItem>,
            }
            let inventory = service.inventory_list(player).await?;
            let equipped = service.equipped_list(player).await?;
            Ok(Response::json(&Inventory {
                inventory,
                equipped,
            }))
        }
        "/api/v1/purchase" => {
            request.require_method("POST")?;
            let input: Purchase = request.json()?;
            let quantity = input.quantity.unwrap_or(1);
            if quantity == 0 {
                return Err(Status::BadRequest);
            }
            if config.unlock_all {
                return Err(Status::UnlockAllNoPurchase);
            }
            let receipt = service
                .purchase_shop_item(player, &input.sku, quantity, &input.request_id)
                .await?;
            Ok(Response::json(&json!({ "ok": true, "receipt": receipt })))
        }
        "/api/v1/equip" => {
            request.require_method("POST")?;
            let input: Equip = request.json()?;
            let slot = input
                .slot
                .as_deref()
                .or_else(|| {
                    service
                        .settings()
                        .catalog()
                        .item(&input.sku)
                        .and_then(|item| item.equip_slot.as_deref())
                })
                .ok_or(Status::BadRequest)?;
            let equipped = service.equip_item(player, &input.sku, slot).await?;
            Ok(Response::json(&json!({ "ok": true, "equipped": equipped })))
        }
        "/api/v1/unequip" => {
            request.require_method("POST")?;
            let input: Unequip = request.json()?;
            let removed = service.unequip_slot(player, &input.slot).await?;
            Ok(Response::json(&json!({ "ok": true, "removed": removed })))
        }
        _ => Err(Status::NotFound),
    }
}

impl From<Error> for Status {
    fn from(error: Error) -> Self {
        match error {
            Error::InvalidInput | Error::Backend(accounts::Error::InvalidInput) => Self::BadRequest,
            Error::Backend(accounts::Error::InsufficientFunds | accounts::Error::Exhausted) => {
                Self::PaymentRequired
            }
            Error::Backend(accounts::Error::Conflict | accounts::Error::ActiveSession) => {
                Self::Conflict
            }
            Error::Backend(
                accounts::Error::InvalidCredentials | accounts::Error::InvalidSession,
            ) => Self::Forbidden,
            Error::Busy
            | Error::QueueExpired
            | Error::Closed
            | Error::Backend(accounts::Error::Busy) => Self::Unavailable,
            Error::RateLimited => Self::TooManyRequests,
            Error::WorkerFault | Error::Backend(_) => Self::Internal,
        }
    }
}
