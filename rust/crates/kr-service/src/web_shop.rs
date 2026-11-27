//! Local portal HTTP API. Player identity comes only from a trusted token issuer.
mod http;
#[cfg(test)]
mod tests;

use crate::{
    account_worker::{AccountService, Error, O1SessionCosmetics},
    accounts::{
        self, CatalogAccess, PlayerProfile, ShopCatalog, ShopCategory, ShopItem, UsableItem,
    },
    accounts::{FriendRequest, Guild, GuildLeaderAction, GuildMember, GuildRole, RankingEntry},
    choco_chat::F5SocialPolicy,
};
use http::{Request, Response, Status};
use kr_wire::kr00967::cosmetics::{
    CosmeticTarget, INVENTORY_CAPTURE_GAP, INVENTORY_LAYOUT_STATUS, ProjectionStatus,
    ProjectionSurface, UNMEASURED_WIRE_GAP, validate_mutual_exclusion,
};
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
    cosmetics: Option<O1SessionCosmetics>,
}

/// The host issues tokens only after binding an authenticated player. Clones
/// share one process-local table; revoke on logout or player removal.
#[derive(Clone)]
pub struct WebShopAuth {
    ttl: Duration,
    sessions: Arc<Mutex<Vec<Session>>>,
    launcher_ops: Arc<tokio::sync::Mutex<()>>,
}

impl WebShopAuth {
    pub fn new(session_ttl_seconds: i64) -> io::Result<Self> {
        Ok(Self {
            ttl: session_ttl(session_ttl_seconds)?,
            sessions: Arc::new(Mutex::new(Vec::new())),
            launcher_ops: Arc::new(tokio::sync::Mutex::new(())),
        })
    }

    pub fn issue_token(&self, player_id: i64) -> io::Result<WebShopToken> {
        self.issue_until(player_id, None)
    }

    pub(crate) async fn launcher_operation(&self) -> tokio::sync::OwnedMutexGuard<()> {
        self.launcher_ops.clone().lock_owned().await
    }

    pub(crate) fn issue_until(
        &self,
        player_id: i64,
        account_deadline: Option<Instant>,
    ) -> io::Result<WebShopToken> {
        if player_id < 1 {
            return Err(invalid("invalid web shop player"));
        }
        let mut sessions = self
            .sessions
            .lock()
            .map_err(|_| io::Error::other("web shop sessions unavailable"))?;
        let now = Instant::now();
        let expires_at = account_deadline.map_or(now + self.ttl, |limit| limit.min(now + self.ttl));
        if expires_at <= now {
            return Err(invalid("web shop session expired"));
        }
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
            expires_at,
            cosmetics: None,
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
        self.with_session(digest, |session| session.player_id)
    }

    // Keep lookup and in-memory selection changes in the same critical section
    // so they cannot race revocation. Callers must not await under this lock.
    fn with_session<T>(
        &self,
        digest: &TokenDigest,
        op: impl FnOnce(&mut Session) -> T,
    ) -> Option<T> {
        let mut sessions = self.sessions.lock().ok()?;
        let now = Instant::now();
        sessions.retain(|session| session.expires_at > now);
        let mut found = None;
        for (index, session) in sessions.iter().enumerate() {
            if digest_eq(&session.digest, digest) {
                found = Some(index);
            }
        }
        Some(op(&mut sessions[found?]))
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

impl Equip {
    fn resolve_slot<'a>(&'a self, catalog: &'a ShopCatalog) -> Result<&'a str, Status> {
        self.slot
            .as_deref()
            .or_else(|| {
                catalog
                    .item(&self.sku)
                    .and_then(|item| item.equip_slot.as_deref())
            })
            .ok_or(Status::BadRequest)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Unequip {
    slot: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SocialTarget {
    player_id: i64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FriendResponseInput {
    requester_player_id: i64,
    accept: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GuildCreateInput {
    name: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GuildMemberInput {
    player_id: i64,
    guild_id: Option<i64>,
}

/// An empty object is valid for an ordinary member leave. Leaders must send
/// either explicit confirmation or the leader-leave flag; the disband route is
/// itself an explicit intent.
#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct GuildLeaveInput {
    guild_id: Option<i64>,
    confirm: Option<bool>,
    leader_leave: Option<bool>,
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct GuildDisbandInput {
    guild_id: Option<i64>,
    confirm: Option<bool>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RankingQuery {
    offset: Option<u32>,
    limit: Option<u32>,
}

#[derive(Serialize)]
struct FriendsResponse {
    friends: Vec<i64>,
    friend_requests: Vec<FriendRequest>,
    incoming_requests: Vec<FriendRequest>,
    outgoing_requests: Vec<FriendRequest>,
}

#[derive(Serialize)]
struct FriendRequestsResponse {
    incoming: Vec<FriendRequest>,
    outgoing: Vec<FriendRequest>,
}

#[derive(Serialize)]
struct BlocksResponse {
    blocks: Vec<i64>,
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

async fn chat_status(service: &AccountService, player: i64) -> Result<Response, Status> {
    let mute = service.social_mute_status(player).await?;
    let policy = F5SocialPolicy::default();
    Ok(Response::json(&json!({
        "mute_status": "available",
        "muted": mute.is_some(),
        "mute_until": mute.as_ref().map(|grant| grant.mute_until),
        "mute_reason": mute.as_ref().map(|grant| grant.reason.as_str()),
        "choco_chat": {
            "extended_live_routing": false,
            "wire_status": "unmeasured",
            "local_policy_defaults": {
                "whisper": policy.whisper,
                "guild_chat": policy.guild_chat,
                "megaphone": policy.megaphone.is_some(),
                "offline_note": policy.note_inbox_cap.is_some(),
                "couple": policy.couple,
                "plaza": !policy.plaza_zones.is_empty(),
            },
        },
    })))
}

fn projection_status(status: ProjectionStatus) -> &'static str {
    match status {
        ProjectionStatus::Measured => "measured",
        ProjectionStatus::StaticCandidate => "static_candidate",
        ProjectionStatus::Unmeasured => "unmeasured",
    }
}

fn cosmetics_summary(source: &str, mut equipped: Vec<(String, String)>) -> Response {
    equipped.sort_unstable_by(|left, right| left.0.cmp(&right.0));
    let slots: Vec<_> = equipped.iter().map(|(slot, _)| slot.as_str()).collect();
    let exclusion = validate_mutual_exclusion(&slots);
    let equipped: Vec<_> = equipped
        .iter()
        .map(|(slot, sku_id)| {
            let target = CosmeticTarget::ALL
                .iter()
                .copied()
                .find(|target| target.spec().slot.token() == Some(slot));
            // The matrix measures routes for the base character key only.
            // It supplies no mapping for the equipped SKU, even in that slot.
            let target_surfaces = ProjectionSurface::ALL.map(|surface| {
                let status = target.map_or(ProjectionStatus::Unmeasured, |target| {
                    target.projection_status(surface)
                });
                let anchor = target
                    .and_then(|target| target.validate_slot_projection(slot, surface).ok())
                    .map(|projection| projection.anchor());
                json!({
                    "surface": surface.as_str(),
                    "status": projection_status(status),
                    "anchor": anchor,
                    "gap": (status == ProjectionStatus::Unmeasured).then_some(UNMEASURED_WIRE_GAP),
                })
            });
            json!({
                "slot": slot,
                "sku_id": sku_id,
                "target": target.map(CosmeticTarget::as_str),
                "projection_status": projection_status(ProjectionStatus::Unmeasured),
                "wire_gap": UNMEASURED_WIRE_GAP,
                "target_surfaces": target_surfaces,
            })
        })
        .collect();
    Response::json(&json!({
        "source": source,
        "equipped": equipped,
        "mutual_exclusion": {"valid": exclusion.is_ok(), "conflicting_slot": exclusion.err()},
        "inventory_layout": {
            "status": projection_status(INVENTORY_LAYOUT_STATUS),
            "gap": INVENTORY_CAPTURE_GAP,
        },
    }))
}

async fn cosmetics_summary_route(
    config: &Config,
    service: &AccountService,
    auth: &WebShopAuth,
    digest: &TokenDigest,
    request: &Request,
) -> Result<Response, Status> {
    if config.unlock_all {
        let equipped = auth
            .with_session(digest, |session| -> Result<_, Status> {
                request.require_queryless_get()?;
                Ok(session
                    .cosmetics
                    .as_ref()
                    .map(|cosmetics| cosmetics.snapshot().into_iter().collect())
                    .unwrap_or_default())
            })
            .ok_or(Status::Unauthorized)??;
        return Ok(cosmetics_summary("session", equipped));
    }
    let player = auth.player(digest).ok_or(Status::Unauthorized)?;
    request.require_queryless_get()?;
    let equipped = service
        .inventory_list(player)
        .await?
        .into_iter()
        .filter(|item| item.active)
        .flat_map(|item| {
            item.equipped_slots
                .into_iter()
                .map(move |slot| (slot, item.sku_id.clone()))
        })
        .collect();
    Ok(cosmetics_summary("inventory", equipped))
}

fn o1_selection_route(
    service: &AccountService,
    session: &mut Session,
    request: &Request,
) -> Result<Response, Status> {
    #[derive(Serialize)]
    struct Selection {
        slot: String,
        sku_id: String,
    }

    if request.query().is_some() {
        return Err(Status::BadRequest);
    }
    let cosmetics = session.cosmetics.get_or_insert_with(|| {
        service
            .o1_session_cosmetics(CatalogAccess { unlock_all: true })
            .expect("O1 selection route has unlock_all access")
    });
    match request.path() {
        "/api/v1/inventory" => {
            request.require_method("GET")?;
            let equipped: Vec<_> = cosmetics
                .snapshot()
                .into_iter()
                .map(|(slot, sku_id)| Selection { slot, sku_id })
                .collect();
            Ok(Response::json(
                &json!({ "inventory": [], "equipped": equipped }),
            ))
        }
        "/api/v1/equip" => {
            request.require_method("POST")?;
            let input: Equip = request.json()?;
            let slot = input.resolve_slot(service.settings().catalog())?;
            cosmetics.equip(&input.sku, slot).map_err(Error::Backend)?;
            let equipped = Selection {
                slot: slot.into(),
                sku_id: input.sku.clone(),
            };
            Ok(Response::json(&json!({ "ok": true, "equipped": equipped })))
        }
        "/api/v1/unequip" => {
            request.require_method("POST")?;
            let input: Unequip = request.json()?;
            let removed = cosmetics.unequip(&input.slot).map_err(Error::Backend)?;
            Ok(Response::json(&json!({ "ok": true, "removed": removed })))
        }
        _ => Err(Status::NotFound),
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

    if path.starts_with("/api/social/") {
        let digest = request.digest().ok_or(Status::Unauthorized)?;
        let player = auth.player(&digest).ok_or(Status::Unauthorized)?;
        return social_route(service, player, request).await;
    }
    if !path.starts_with("/api/v1/") {
        return Err(Status::NotFound);
    }
    let digest = request.digest().ok_or(Status::Unauthorized)?;
    if path == "/api/v1/cosmetics-summary" {
        return cosmetics_summary_route(config, service, auth, &digest, request).await;
    }
    if config.unlock_all
        && matches!(
            path,
            "/api/v1/inventory" | "/api/v1/equip" | "/api/v1/unequip"
        )
    {
        return auth
            .with_session(&digest, |session| {
                o1_selection_route(service, session, request)
            })
            .ok_or(Status::Unauthorized)?;
    }
    let player = auth.player(&digest).ok_or(Status::Unauthorized)?;

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
            let slot = input.resolve_slot(service.settings().catalog())?;
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

async fn social_route(
    service: &AccountService,
    player: i64,
    request: &Request,
) -> Result<Response, Status> {
    if request.path() == "/api/social/chat-status" {
        request.require_queryless_get()?;
        return chat_status(service, player).await;
    }
    if request.query().is_some() && request.path() != "/api/social/ranking" {
        return Err(Status::BadRequest);
    }
    match request.path() {
        "/api/social/friends" => match request.method() {
            "GET" => {
                let requests = service.social_friend_requests(player).await?;
                let friend_requests = requests.clone();
                let (incoming_requests, outgoing_requests): (Vec<_>, Vec<_>) = requests
                    .into_iter()
                    .partition(|request| request.recipient_id == player);
                Ok(Response::json(&FriendsResponse {
                    friends: service.social_friends(player).await?,
                    friend_requests,
                    incoming_requests,
                    outgoing_requests,
                }))
            }
            "POST" | "DELETE" => {
                request.require_method(request.method())?;
                let input: SocialTarget = request.json()?;
                if input.player_id < 1 || input.player_id == player {
                    return Err(Status::BadRequest);
                }
                let changed = if request.method() == "POST" {
                    service.social_friend_add(player, input.player_id).await?
                } else {
                    service
                        .social_friend_remove(player, input.player_id)
                        .await?
                };
                Ok(Response::json(&json!({"changed": changed})))
            }
            _ => Err(Status::MethodNotAllowed),
        },
        "/api/social/friend-requests" => {
            request.require_method("GET")?;
            let requests = service.social_friend_requests(player).await?;
            let (incoming, outgoing): (Vec<_>, Vec<_>) = requests
                .into_iter()
                .partition(|request| request.recipient_id == player);
            Ok(Response::json(&FriendRequestsResponse {
                incoming,
                outgoing,
            }))
        }
        "/api/social/friends/respond" => {
            request.require_method("POST")?;
            let input: FriendResponseInput = request.json()?;
            if input.requester_player_id < 1 || input.requester_player_id == player {
                return Err(Status::BadRequest);
            }
            let changed = service
                .social_friend_respond(player, input.requester_player_id, input.accept)
                .await?;
            Ok(Response::json(&json!({"changed": changed})))
        }
        "/api/social/blocks" => match request.method() {
            "GET" => Ok(Response::json(&BlocksResponse {
                blocks: service.social_blocks(player).await?,
            })),
            "POST" | "DELETE" => {
                request.require_method(request.method())?;
                let input: SocialTarget = request.json()?;
                if input.player_id < 1 || input.player_id == player {
                    return Err(Status::BadRequest);
                }
                let changed = if request.method() == "POST" {
                    service.social_block_add(player, input.player_id).await?
                } else {
                    service.social_block_remove(player, input.player_id).await?
                };
                Ok(Response::json(&json!({"changed": changed})))
            }
            _ => Err(Status::MethodNotAllowed),
        },
        "/api/social/guild" => {
            request.require_method("GET")?;
            let result = service.social_guild(player).await?;
            let (guild, members) = result.map_or(
                (Option::<Guild>::None, Vec::<GuildMember>::new()),
                |(guild, members)| (Some(guild), members),
            );
            let is_leader = guild.as_ref().is_some_and(|g| g.leader_player_id == player);
            Ok(Response::json(
                &json!({"guild": guild, "members": members, "is_leader": is_leader}),
            ))
        }
        "/api/social/guild/create" => {
            request.require_method("POST")?;
            let input: GuildCreateInput = request.json()?;
            let guild = service.social_guild_create(player, input.name).await?;
            let members = vec![GuildMember {
                player_id: player,
                role: GuildRole::Leader,
                joined_at: guild.created_at,
            }];
            Ok(Response::json(
                &json!({"guild": guild, "members": members, "is_leader": true}),
            ))
        }
        "/api/social/guild/leave" => {
            request.require_method("POST")?;
            let input: GuildLeaveInput = request.json()?;
            let Some((guild, _)) = service.social_guild(player).await? else {
                return Err(Status::Conflict);
            };
            if let Some(requested) = input.guild_id
                && (requested < 1 || requested != guild.id)
            {
                return Err(Status::BadRequest);
            }
            let explicit_leader_leave =
                input.confirm == Some(true) || input.leader_leave == Some(true);
            if guild.leader_player_id == player {
                if !explicit_leader_leave {
                    return Err(Status::Conflict);
                }
                let changed = service.social_guild_disband(player, guild.id).await?;
                return Ok(Response::json(
                    &json!({"changed": changed, "disbanded": true}),
                ));
            }
            let changed = service.social_guild_leave(player, guild.id).await?;
            Ok(Response::json(
                &json!({"changed": changed, "disbanded": false}),
            ))
        }
        "/api/social/guild/disband" => {
            request.require_method("POST")?;
            let input: GuildDisbandInput = request.json()?;
            if input.confirm == Some(false) {
                return Err(Status::BadRequest);
            }
            let Some((guild, _)) = service.social_guild(player).await? else {
                return Err(Status::Conflict);
            };
            if let Some(requested) = input.guild_id
                && (requested < 1 || requested != guild.id)
            {
                return Err(Status::BadRequest);
            }
            let changed = service.social_guild_disband(player, guild.id).await?;
            Ok(Response::json(
                &json!({"changed": changed, "disbanded": true}),
            ))
        }
        "/api/social/guild/add-member" => {
            guild_member_route(service, player, request, GuildLeaderAction::AddMember).await
        }
        "/api/social/guild/remove-member" => {
            guild_member_route(service, player, request, GuildLeaderAction::RemoveMember).await
        }
        "/api/social/guild/transfer-leader" => {
            guild_member_route(
                service,
                player,
                request,
                GuildLeaderAction::TransferLeadership,
            )
            .await
        }
        "/api/social/ranking" => {
            request.require_method("GET")?;
            let query = request.query().unwrap_or("");
            let input = parse_ranking_query(query)?;
            let rows: Vec<RankingEntry> = service
                .social_ranking(
                    player,
                    input.offset.unwrap_or(0),
                    input.limit.unwrap_or(100),
                )
                .await?;
            Ok(Response::json(&json!({"ranking": rows})))
        }
        _ => Err(Status::NotFound),
    }
}

async fn guild_member_route(
    service: &AccountService,
    player: i64,
    request: &Request,
    action: fn(i64) -> GuildLeaderAction,
) -> Result<Response, Status> {
    request.require_method("POST")?;
    let input: GuildMemberInput = request.json()?;
    let changed = service
        .social_guild_manage(player, input.guild_id, action(input.player_id))
        .await?;
    Ok(Response::json(&json!({"changed": changed})))
}

fn parse_ranking_query(query: &str) -> Result<RankingQuery, Status> {
    if query.is_empty() {
        return Ok(RankingQuery {
            offset: None,
            limit: None,
        });
    }
    let mut offset = None;
    let mut limit = None;
    for pair in query.split('&') {
        let (key, value) = pair.split_once('=').ok_or(Status::BadRequest)?;
        match key {
            "offset" if offset.is_none() => {
                offset = Some(value.parse().map_err(|_| Status::BadRequest)?)
            }
            "limit" if limit.is_none() => {
                limit = Some(value.parse().map_err(|_| Status::BadRequest)?)
            }
            _ => return Err(Status::BadRequest),
        }
    }
    if offset.unwrap_or(0) > 10_000 || !matches!(limit.unwrap_or(100), 1..=100) {
        return Err(Status::BadRequest);
    }
    Ok(RankingQuery { offset, limit })
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
