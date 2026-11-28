//! Local portal HTTP API. Player identity comes only from a trusted token issuer.
mod http;
#[cfg(test)]
mod tests;

use crate::{
    account_worker::{AccountService, Error, O1SessionCosmetics, SocialReply},
    accounts::{
        self, CatalogAccess, ItemUpgradeOperation, LedgerEntry, PlayerProfile,
        RecordEraserOperation, ShopCatalog, ShopCategory, ShopItem, UsableItem,
    },
    accounts::{FriendRequest, Guild, GuildLeaderAction, GuildMember, GuildRole, RankingEntry},
    choco_chat::F5SocialPolicy,
    custom_map_store::{CustomMapStore, CustomMapStoreError},
    f3_pve_runtime::F3PveRegistry,
    f4_extended_policy::{
        BASIS_POINTS, CouponRequest, Decision, ItemOrigin, KitRoomRules, KitStats,
        MAX_EQUIPPED_KITS, MilestoneKind, OperationKey, OwnedItem, PolicyError, PrimarySession,
        RecordScope, SecondPasswordAction, SecondPasswordOutcome, UpgradeFamily,
    },
    f4_security_kits::{
        F4SecurityKitsRuntime, F4SecurityKitsStatus, PowerupKitRuntime, SecondPasswordRuntime,
    },
    f6_policy::F6EventPolicy,
};
use argon2::{Algorithm, Argon2, Params, Version};
use http::{Request, Response, Status};
use kr_wire::kr00967::cosmetics::{
    CosmeticTarget, INVENTORY_CAPTURE_GAP, INVENTORY_LAYOUT_STATUS, ProjectionStatus,
    ProjectionSurface, UNMEASURED_WIRE_GAP, validate_mutual_exclusion,
};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeSet, HashMap},
    fmt::Write as _,
    hint::black_box,
    io,
    net::{IpAddr, Ipv4Addr, SocketAddrV4},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::{
    net::TcpListener,
    sync::{Mutex as AsyncMutex, watch},
    task::{JoinSet, spawn_blocking},
    time::timeout,
};
use zeroize::Zeroizing;

const TOKEN_CAPACITY: usize = 1024;
const TOKEN_HEX_LEN: usize = 64;
const COUPONS_PER_TOKEN: usize = 32;
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

#[derive(Clone, Copy)]
struct MilestoneContext {
    account: u64,
    character: u16,
}

#[derive(Clone)]
struct LocalCoupon {
    account: u64,
    offer: String,
    issued_at: u64,
}

struct Session {
    digest: TokenDigest,
    player_id: i64,
    expires_at: Instant,
    primary_session: Option<PrimarySession>,
    cosmetics: Option<O1SessionCosmetics>,
    milestone: Option<MilestoneContext>,
    coupons: HashMap<u64, LocalCoupon>,
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

    /// For a trusted host that has authenticated this player and verified the
    /// primary session. Revoke the token when the primary session ends.
    pub fn issue_token_for_primary_session(
        &self,
        player: &accounts::Player,
        primary_session: PrimarySession,
    ) -> io::Result<WebShopToken> {
        if player.account_id <= 0
            || primary_session.account != player.account_id as u64
            || primary_session.session == 0
        {
            return Err(invalid("invalid web shop primary session"));
        }
        self.issue(player.player_id, None, None, Some(primary_session))
    }

    /// For a trusted issuer with an authenticated player and a server-resolved
    /// character. The claim inputs are bound to the token, never read from HTTP.
    pub fn issue_token_for_milestones(
        &self,
        player: &accounts::Player,
        character: u16,
    ) -> io::Result<WebShopToken> {
        let account = u64::try_from(player.account_id)
            .ok()
            .filter(|&id| id != 0)
            .ok_or_else(|| invalid("invalid web shop account"))?;
        self.issue(
            player.player_id,
            None,
            Some(MilestoneContext { account, character }),
            None,
        )
    }

    /// Attach an already issued local coupon to the intended player's live
    /// token. The trusted host supplies the globally unique ID, offer and
    /// original UTC issue time; the portal does not issue coupons over HTTP.
    pub fn bind_local_coupon(
        &self,
        token: &WebShopToken,
        player: &accounts::Player,
        coupon_id: u64,
        offer: &str,
        issued_at: u64,
    ) -> io::Result<()> {
        let account = u64::try_from(player.account_id)
            .ok()
            .filter(|&id| id != 0)
            .ok_or_else(|| invalid("invalid web shop account"))?;
        if coupon_id == 0 || !(1..=128).contains(&offer.len()) {
            return Err(invalid("invalid local coupon"));
        }
        self.with_session(&token_digest(token.as_str()), |session| {
            if session.player_id != player.player_id
                || session.coupons.len() >= COUPONS_PER_TOKEN
                || session.coupons.contains_key(&coupon_id)
            {
                return Err(invalid("cannot bind local coupon"));
            }
            session.coupons.insert(
                coupon_id,
                LocalCoupon {
                    account,
                    offer: offer.to_owned(),
                    issued_at,
                },
            );
            Ok(())
        })
        .ok_or_else(|| invalid("web shop session unavailable"))?
    }

    pub(crate) async fn launcher_operation(&self) -> tokio::sync::OwnedMutexGuard<()> {
        self.launcher_ops.clone().lock_owned().await
    }

    pub(crate) fn issue_until(
        &self,
        player_id: i64,
        account_deadline: Option<Instant>,
    ) -> io::Result<WebShopToken> {
        self.issue(player_id, account_deadline, None, None)
    }

    fn issue(
        &self,
        player_id: i64,
        account_deadline: Option<Instant>,
        milestone: Option<MilestoneContext>,
        primary_session: Option<PrimarySession>,
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
            primary_session,
            cosmetics: None,
            milestone,
            coupons: HashMap::new(),
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
    shop: WebShop,
    auth: WebShopAuth,
}

impl Opened {
    pub(crate) fn auth(&self) -> WebShopAuth {
        self.auth.clone()
    }

    #[allow(
        dead_code,
        reason = "available for the host to attach its PvE registry"
    )]
    pub(crate) fn with_f3_pve_registry(mut self, registry: Arc<F3PveRegistry>) -> Self {
        self.shop = self.shop.with_f3_pve_registry(registry);
        self
    }

    pub(crate) fn with_f4_security_kits(mut self, kits: Arc<F4SecurityKitsRuntime>) -> Self {
        self.shop = self.shop.with_f4_security_kits(kits);
        self
    }

    pub(crate) fn with_policies(
        mut self,
        f6_policy: Option<Arc<F6EventPolicy>>,
        custom_map_store: Option<CustomMapStore>,
    ) -> Self {
        if let Some(policy) = f6_policy {
            self.shop = self.shop.with_f6_policy(policy);
        }
        if let Some(store) = custom_map_store {
            self.shop = self.shop.with_custom_map_store(store);
        }
        self
    }
}

pub(crate) async fn open(config: Config) -> io::Result<Opened> {
    config.validate()?;
    let auth = WebShopAuth::new(config.session_ttl_seconds)?;
    let listener = TcpListener::bind(config.bind).await?;
    Ok(Opened {
        listener,
        shop: WebShop::new(config),
        auth,
    })
}

/// A local portal with optional storage and status policies.
pub struct WebShop {
    config: Config,
    custom_maps: Option<Arc<AsyncMutex<CustomMapStore>>>,
    f3_pve_registry: Arc<F3PveRegistry>,
    f4_security_kits: Arc<F4SecurityKitsRuntime>,
    second_password_credentials: Arc<AsyncMutex<HashMap<u64, SecondPasswordCredential>>>,
    #[cfg(test)]
    second_password_test_now: Arc<std::sync::atomic::AtomicU64>,
    f6_policy: Arc<F6EventPolicy>,
    f6_configured: bool,
}

impl WebShop {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            custom_maps: None,
            f3_pve_registry: Arc::new(F3PveRegistry::default()),
            f4_security_kits: Arc::new(F4SecurityKitsRuntime::default()),
            second_password_credentials: Arc::default(),
            #[cfg(test)]
            second_password_test_now: Arc::default(),
            f6_policy: Arc::new(F6EventPolicy::disabled_default()),
            f6_configured: false,
        }
    }

    /// Opt in to local map storage. Set `max_body_bytes` high enough for the
    /// base64 and JSON overhead (12 KiB accommodates an 8192-byte map).
    pub fn with_custom_map_store(mut self, store: CustomMapStore) -> Self {
        self.custom_maps = Some(Arc::new(AsyncMutex::new(store)));
        self
    }

    /// Share the host's PvE registry for read-only status snapshots.
    pub fn with_f3_pve_registry(mut self, registry: Arc<F3PveRegistry>) -> Self {
        self.f3_pve_registry = registry;
        self
    }

    /// Share the host's kit and second-password runtime. The portal keeps its
    /// own in-memory credential verifier for passwords set up through HTTP.
    pub fn with_f4_security_kits(mut self, kits: Arc<F4SecurityKitsRuntime>) -> Self {
        self.f4_security_kits = kits;
        self
    }

    /// Opt in to the validated, read-only F6 event status policy.
    pub fn with_f6_policy(mut self, policy: Arc<F6EventPolicy>) -> Self {
        self.f6_policy = policy;
        self.f6_configured = true;
        self
    }

    /// Bind the configured IPv4 interface. Dropping this future closes the
    /// listener and aborts connection tasks. A mutation already running may
    /// still commit.
    pub async fn serve(self, service: AccountService, auth: WebShopAuth) -> io::Result<()> {
        self.config.validate()?;
        if session_ttl(self.config.session_ttl_seconds)? != auth.ttl {
            return Err(invalid("web shop session lifetime mismatch"));
        }
        let listener = TcpListener::bind(self.config.bind).await?;
        serve_loop(listener, self, service, auth, None).await
    }
}

/// Serve the portal without custom map storage.
pub async fn serve(config: Config, service: AccountService, auth: WebShopAuth) -> io::Result<()> {
    WebShop::new(config).serve(service, auth).await
}

pub(crate) async fn serve_until(
    opened: Opened,
    service: AccountService,
    stop: watch::Receiver<bool>,
) -> io::Result<()> {
    serve_loop(
        opened.listener,
        opened.shop,
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
    serve_loop(listener, WebShop::new(config), service, auth, None).await
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
    shop: WebShop,
    service: AccountService,
    auth: WebShopAuth,
    mut stop: Option<watch::Receiver<bool>>,
) -> io::Result<()> {
    let shop = Arc::new(shop);
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
                if !shop.config.allows(peer.ip()) {
                    continue;
                }
                while connections.try_join_next().is_some() {}
                if connections.len() >= shop.config.max_connections {
                    continue;
                }
                let (shop, service, auth) = (shop.clone(), service.clone(), auth.clone());
                connections.spawn(async move {
                    let config = &shop.config;
                    let _ = timeout(Duration::from_millis(config.request_timeout_ms), async {
                        let response = match http::read(&mut stream, config.max_body_bytes).await {
                            Ok(request) => dispatch(&shop, &service, &auth, request).await,
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
struct MilestoneInput {
    level: u16,
    kind: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RedeemCouponInput {
    coupon_id: u64,
    request_id: u64,
    /// These fields are supplied by a trusted local issuer. A coupon bound to
    /// a token by `bind_local_coupon` takes precedence over them.
    offer: Option<String>,
    issued_to: Option<u64>,
    issued_at: Option<u64>,
    locally_issued: Option<bool>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EraseRecordInput {
    sku: String,
    scope: RecordScope,
    request_id: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UpgradeItemInput {
    sku: String,
    family: UpgradeFamily,
    level: u16,
    request_id: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct KitEquipInput {
    slot: usize,
    sku: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct KitClearInput {
    slot: usize,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SecondPasswordSetupInput {
    pin: Zeroizing<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SecondPasswordAttemptInput {
    action: String,
    pin: Zeroizing<String>,
    new_pin: Option<Zeroizing<String>>,
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
    shop: &WebShop,
    service: &AccountService,
    auth: &WebShopAuth,
    request: Request,
) -> Response {
    match route(shop, service, auth, &request).await {
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

fn f4_security_kits_status(
    kits: &F4SecurityKitsRuntime,
    primary_session: Option<PrimarySession>,
) -> Result<Response, Status> {
    #[derive(Serialize)]
    struct SecurityKitsStatus {
        #[serde(flatten)]
        snapshot: F4SecurityKitsStatus,
        second_password_unlocked: bool,
    }

    let snapshot = kits.snapshot().map_err(|_| Status::Unavailable)?;
    let second_password_unlocked = primary_session
        .is_some_and(|session| kits.second_password.is_unlocked(session) == Ok(true));
    Ok(Response::json(&SecurityKitsStatus {
        snapshot,
        second_password_unlocked,
    }))
}

async fn social_policy_status(service: &AccountService, player: i64) -> Result<Response, Status> {
    let SocialReply::F5PolicyStatus(policy) = service
        .client
        .try_f5_social_policy_status(player)?
        .wait()
        .await?
    else {
        return Err(Status::Internal);
    };
    let unmuted = service.social_mute_status(player).await?.is_none();
    let megaphone_global = policy.megaphone.is_some_and(|p| p.global);
    let megaphone_channel = policy.megaphone.is_some_and(|p| p.channel);

    // This is the sender-side policy and moderation check. Delivery also
    // depends on the live route, recipients, and rate limits.
    Ok(Response::json(&json!({
        "enabled": {
            "whisper": policy.whisper,
            "guild_chat": policy.guild_chat,
            "megaphone": policy.megaphone.is_some(),
            "offline_note": policy.note_inbox_cap.is_some(),
            "couple": policy.couple,
            "plaza": !policy.plaza_zones.is_empty(),
        },
        "channel_eligibility": {
            "whisper": unmuted && policy.whisper,
            "guild_chat": unmuted && policy.guild_chat,
            "megaphone_global": unmuted && megaphone_global,
            "megaphone_channel": unmuted && megaphone_channel,
        },
    })))
}

fn events_status(policy: &F6EventPolicy, configured: bool) -> Response {
    let core: Vec<_> = policy
        .enabled_systems()
        .map(|(system, value)| {
            json!({
                "system": system,
                "start_utc": value.start_utc,
                "end_utc": value.end_utc,
                "rate_caps": value.rate_caps,
            })
        })
        .collect();
    let extended: Vec<_> = policy
        .enabled_extended_systems()
        .map(|(system, value)| {
            json!({
                "system": system,
                "start_utc": value.start_utc,
                "end_utc": value.end_utc,
                "season_key": &value.season_key,
                "rate_caps": value.rate_caps,
                "max_grants_per_day": value.max_grants_per_day,
                "max_grants_per_season": value.max_grants_per_season,
            })
        })
        .collect();
    Response::json(&json!({
        "configured": configured || !core.is_empty() || !extended.is_empty(),
        "core_total": 19,
        "extended_total": 25,
        "excluded_systems": ["PCRoomBenefit"],
        "enabled_systems": core,
        "extended_systems": extended,
    }))
}

fn f4_extended_status(service: &AccountService) -> Response {
    let settings = service.settings();
    let booster = settings.booster_ticket_policy();
    let levels = settings.level_reward_policy();
    let vault = settings.lucci_vault_policy();
    let coupons = settings.gift_coupon_policy();
    let erasers = settings.record_eraser_policy();
    let upgrades = settings.upgrade_policy();
    let titles = settings.title_policy();
    // The service stores effective policies, without the source file or a
    // presence flag. Report whether each differs from its disabled default.
    let booster_configured = *booster != Default::default();
    let levels_configured = *levels != Default::default();
    let vault_configured = *vault != Default::default();
    let coupons_configured = *coupons != Default::default();
    let erasers_configured = *erasers != Default::default();
    let upgrades_configured = *upgrades != Default::default();
    let titles_configured = *titles != Default::default();
    let chests = levels.character_milestones.as_deref().unwrap_or_default();
    let lucci = levels.level_up_lucci_table.as_deref().unwrap_or_default();
    let tiers: BTreeSet<_> = chests
        .iter()
        .map(|row| row.level)
        .chain(lucci.iter().map(|row| row.level))
        .collect();
    Response::json(&json!({
        "configured": booster_configured || levels_configured || vault_configured
            || coupons_configured || erasers_configured || upgrades_configured || titles_configured,
        "booster_tickets": {
            "configured": booster_configured,
            "enabled": booster.enabled,
            "exp_percent": booster.exp_percent,
            "lucci_percent": booster.lucci_percent,
            "bonus_cap": booster.bonus_cap.as_ref().map(|cap| json!({
                "exp": cap.exp, "lucci": cap.lucci,
            })),
            "room_rule_count": booster.allowed_rooms.len(),
        },
        "level_rewards": {
            "configured": levels_configured,
            "enabled": levels.enabled,
            "chest_rule_count": chests.len(),
            "lucci_rule_count": lucci.len(),
            "tiers": tiers,
        },
        "lucci_vault": {
            "configured": vault_configured,
            "enabled": vault.enabled,
            "base_lucci_cap": vault.base_lucci_cap,
            "expansion_step": vault.expansion_step,
            "max_expansions": vault.max_expansions,
            "max_lucci_cap": vault.max_lucci_cap,
        },
        "gift_coupons": {
            "configured": coupons_configured,
            "enabled": coupons.enabled,
            "offer_count": coupons.coupon_offers.len(),
        },
        "record_erasers": {
            "configured": erasers_configured,
            "enabled": erasers.enabled,
            "eraser_count": erasers.erasers.len(),
        },
        "upgrades": {
            "configured": upgrades_configured,
            "enabled": upgrades.enabled,
            "recipe_count": upgrades.recipes.len(),
        },
        "titles": {
            "configured": titles_configured,
            "enabled": titles.enabled,
            "title_count": titles.unlock_table.as_ref().map_or(0, Vec::len),
        },
    }))
}

async fn claim_milestone(
    config: &Config,
    service: &AccountService,
    player: i64,
    context: Option<MilestoneContext>,
    request: &Request,
) -> Result<Response, Status> {
    request.require_method("POST")?;
    let MilestoneInput { level, kind } = request.json_object()?;
    let reward_kind = match kind.as_str() {
        "lucci" => MilestoneKind::Lucci,
        "chest" => MilestoneKind::Chest,
        _ => return Err(Status::BadRequest),
    };
    if !(2..=accounts::LEVEL_MAX).contains(&usize::from(level)) {
        return Err(Status::BadRequest);
    }
    if config.unlock_all {
        return Err(Status::UnlockAllNoPurchase);
    }
    let MilestoneContext { account, character } = context.ok_or(Status::Forbidden)?;
    let grant = service
        .settings()
        .level_reward_policy()
        .grants_for_level_gain(account, character, level - 1, level, &BTreeSet::new())
        .map_err(|error| match error {
            PolicyError::Disabled => Status::Forbidden,
            PolicyError::InvalidInput => Status::BadRequest,
            _ => Status::Internal,
        })?
        .into_iter()
        .find(|grant| grant.claim.kind == reward_kind)
        .ok_or(Status::BadRequest)?;
    let (replayed, entry) = match service.claim_level_milestone_reward(player, &grant).await? {
        Decision::Apply(entry) => (false, entry),
        Decision::Replay(entry) => (true, entry),
    };
    Ok(Response::json(&json!({
        "ok": true,
        "level": level,
        "kind": kind,
        "replayed": replayed,
        "lucci_delta": entry.lucci_delta,
        "sku_id": entry.sku_id,
    })))
}

fn f4_mutation_input<T: DeserializeOwned>(
    config: &Config,
    request: &Request,
    enabled: bool,
) -> Result<T, Status> {
    require_f4_mutation(config, request)?;
    if !enabled {
        return Err(Status::Forbidden);
    }
    request.json_object()
}

fn require_f4_mutation(config: &Config, request: &Request) -> Result<(), Status> {
    request.require_method("POST")?;
    if config.unlock_all {
        return Err(Status::UnlockAllNoPurchase);
    }
    Ok(())
}

fn f4_runtime_error(error: PolicyError) -> Status {
    match error {
        PolicyError::Disabled => Status::Forbidden,
        PolicyError::InvalidInput => Status::BadRequest,
        PolicyError::NotAllowed
        | PolicyError::NotOwnedOrExpired
        | PolicyError::AlreadyEquipped
        | PolicyError::CapacityExceeded => Status::Conflict,
        _ => Status::Internal,
    }
}

fn utc_seconds() -> Result<u64, Status> {
    accounts::utc()
        .ok()
        .and_then(|now| u64::try_from(now).ok())
        .ok_or(Status::Internal)
}

fn kit_projection(
    runtime: &PowerupKitRuntime,
    account: u64,
    now: u64,
    slot: usize,
    outcome: &'static str,
) -> Result<Response, Status> {
    // The shop has no character or room. Project the kit contribution from
    // zero with only the stat type's ceiling; match start supplies real caps.
    let base = KitStats {
        bombs: 0,
        blast: 0,
        speed: 0,
    };
    let caps = KitStats {
        bombs: u8::MAX,
        blast: u8::MAX,
        speed: u8::MAX,
    };
    let stats = runtime
        .starting_stats(
            account,
            now,
            base,
            KitRoomRules {
                allows_kits: true,
                character_caps: caps,
                mode_caps: caps,
                map_caps: caps,
            },
        )
        .map_err(f4_runtime_error)?;
    Ok(Response::json(&json!({
        "outcome": outcome,
        "slot": slot,
        "projection": "kit_bonus_preview",
        "starting_stats": {"bombs": stats.bombs, "blast": stats.blast, "speed": stats.speed},
    })))
}

async fn update_powerup_kit(
    shop: &WebShop,
    service: &AccountService,
    auth: &WebShopAuth,
    digest: &TokenDigest,
    player: i64,
    request: &Request,
) -> Result<Response, Status> {
    require_f4_mutation(&shop.config, request)?;
    let runtime = &shop.f4_security_kits.powerup_kits;
    if !shop
        .f4_security_kits
        .snapshot()
        .map_err(|_| Status::Unavailable)?
        .powerup_kits
        .enabled
    {
        return Err(Status::Forbidden);
    }
    // Each account has one DB player; use that stable key for every portal
    // token, including ones without a bound primary session.
    let account = u64::try_from(player).map_err(|_| Status::Internal)?;
    if request.path() == "/api/v1/f4/kits/clear" {
        let KitClearInput { slot } = request.json_object()?;
        return auth
            .with_session(digest, |_| {
                let now = utc_seconds()?;
                runtime.clear(account, slot).map_err(f4_runtime_error)?;
                kit_projection(runtime, account, now, slot, "cleared")
            })
            .ok_or(Status::Unauthorized)?;
    }

    let KitEquipInput { slot, sku } = request.json_object()?;
    if slot >= MAX_EQUIPPED_KITS
        || sku.is_empty()
        || sku.trim() != sku
        || sku.chars().any(char::is_control)
    {
        return Err(Status::BadRequest);
    }
    let item = service
        .inventory_list(player)
        .await?
        .into_iter()
        .find(|item| item.active && item.sku_id == sku)
        .ok_or(Status::Conflict)?;
    let quantity = u32::try_from(item.quantity).map_err(|_| Status::Conflict)?;
    let expires_at = item
        .expires_at
        .map(u64::try_from)
        .transpose()
        .map_err(|_| Status::Conflict)?;
    auth.with_session(digest, |_| {
        let now = utc_seconds()?;
        runtime
            .equip(
                account,
                now,
                slot,
                OwnedItem {
                    owner: account,
                    sku: &item.sku_id,
                    quantity,
                    expires_at,
                    origin: ItemOrigin::PersistentInventory,
                },
            )
            .map_err(f4_runtime_error)?;
        kit_projection(runtime, account, now, slot, "equipped")
    })
    .ok_or(Status::Unauthorized)?
}

struct SecondPasswordCredential {
    salt: [u8; 16],
    hash: Zeroizing<[u8; 32]>,
}

impl SecondPasswordCredential {
    fn derive(pin: &str, salt: &[u8; 16]) -> Result<Zeroizing<[u8; 32]>, Status> {
        let mut hash = Zeroizing::new([0; 32]);
        Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            Params::new(19_456, 2, 1, Some(32)).expect("fixed valid Argon2id parameters"),
        )
        .hash_password_into(pin.as_bytes(), salt, hash.as_mut())
        .map_err(|_| Status::Internal)?;
        Ok(hash)
    }

    fn new(pin: &str) -> Result<Self, Status> {
        let mut salt = [0; 16];
        getrandom::fill(&mut salt).map_err(|_| Status::Internal)?;
        let hash = Self::derive(pin, &salt)?;
        Ok(Self { salt, hash })
    }

    fn verify(&self, pin: &str) -> Result<bool, Status> {
        Ok(digest_eq(&*Self::derive(pin, &self.salt)?, &self.hash))
    }
}

fn validate_pin(pin: &str) -> Result<(), Status> {
    if !(4..=128).contains(&pin.len()) || pin.trim().is_empty() || pin.chars().any(char::is_control)
    {
        return Err(Status::BadRequest);
    }
    Ok(())
}

fn password_summary(
    runtime: &SecondPasswordRuntime,
    session: PrimarySession,
    outcome: &'static str,
) -> Result<Response, Status> {
    let unlocked = runtime.is_unlocked(session).map_err(f4_runtime_error)?;
    Ok(Response::json(
        &json!({"outcome": outcome, "unlocked": unlocked}),
    ))
}

enum VerifiedPasswordAction {
    Setup(SecondPasswordCredential),
    Attempt {
        action: SecondPasswordAction,
        verified: bool,
        replacement: Option<SecondPasswordCredential>,
    },
}

fn second_password_now(_shop: &WebShop) -> Result<u64, Status> {
    #[cfg(test)]
    {
        let now = _shop
            .second_password_test_now
            .load(std::sync::atomic::Ordering::Relaxed);
        if now != 0 {
            return Ok(now);
        }
    }
    utc_seconds()
}

async fn update_second_password(
    shop: &WebShop,
    auth: &WebShopAuth,
    digest: &TokenDigest,
    request: &Request,
) -> Result<Response, Status> {
    require_f4_mutation(&shop.config, request)?;
    let runtime = &shop.f4_security_kits.second_password;
    if !shop
        .f4_security_kits
        .snapshot()
        .map_err(|_| Status::Unavailable)?
        .second_password
        .enabled
    {
        return Err(Status::Forbidden);
    }
    // Move the guard into the blocking task to serialize verifier work even if
    // the HTTP request times out. Commit only after rechecking the live token.
    let credentials = shop.second_password_credentials.clone().lock_owned().await;
    let session = auth
        .with_session(digest, |session| session.primary_session)
        .ok_or(Status::Unauthorized)?
        .ok_or(Status::Forbidden)?;
    let now = second_password_now(shop)?;
    let (action, pin, new_pin) = if request.path() == "/api/v1/f4/second-password/setup" {
        let SecondPasswordSetupInput { pin } = request.json_object()?;
        if credentials.contains_key(&session.account) {
            return Err(Status::Conflict);
        }
        (None, pin, None)
    } else {
        let SecondPasswordAttemptInput {
            action,
            pin,
            new_pin,
        } = request.json_object()?;
        let action = match (action.as_str(), new_pin.is_some()) {
            ("unlock", false) => SecondPasswordAction::Unlock,
            ("change", true) => SecondPasswordAction::Change,
            ("disable", false) => SecondPasswordAction::Disable,
            _ => return Err(Status::BadRequest),
        };
        if !runtime
            .may_attempt(session, now)
            .map_err(f4_runtime_error)?
        {
            return password_summary(runtime, session, "locked");
        }
        (Some(action), pin, new_pin)
    };
    validate_pin(&pin)?;
    if let Some(pin) = new_pin.as_deref() {
        validate_pin(pin)?;
    }
    let (mut credentials, operation) = spawn_blocking(move || {
        let operation = if let Some(action) = action {
            let current = credentials.get(&session.account).ok_or(Status::Conflict)?;
            let verified = current.verify(&pin)?;
            let replacement = new_pin
                .filter(|_| verified)
                .map(|pin| SecondPasswordCredential::new(&pin))
                .transpose()?;
            VerifiedPasswordAction::Attempt {
                action,
                verified,
                replacement,
            }
        } else {
            VerifiedPasswordAction::Setup(SecondPasswordCredential::new(&pin)?)
        };
        Ok::<_, Status>((credentials, operation))
    })
    .await
    .map_err(|_| Status::Internal)??;

    auth.with_session(digest, |_| {
        let outcome = match operation {
            VerifiedPasswordAction::Setup(credential) => {
                runtime.setup(session).map_err(f4_runtime_error)?;
                credentials.insert(session.account, credential);
                "configured"
            }
            VerifiedPasswordAction::Attempt {
                action,
                verified,
                mut replacement,
            } => {
                match runtime
                    .attempt(session, second_password_now(shop)?, action, verified)
                    .map_err(f4_runtime_error)?
                {
                    SecondPasswordOutcome::Unlocked => "unlocked",
                    SecondPasswordOutcome::Incorrect => "incorrect",
                    SecondPasswordOutcome::Locked => "locked",
                    SecondPasswordOutcome::Changed => {
                        let credential = replacement.take().ok_or(Status::Internal)?;
                        credentials.insert(session.account, credential);
                        "changed"
                    }
                    SecondPasswordOutcome::Disabled => {
                        credentials.remove(&session.account);
                        "disabled"
                    }
                }
            }
        };
        password_summary(runtime, session, outcome)
    })
    .ok_or(Status::Unauthorized)?
}

fn f4_mutation_result(result: Result<Decision<LedgerEntry>, Error>) -> Result<Response, Status> {
    match result {
        Ok(Decision::Apply(entry)) => Ok(Response::json(&json!({
            "ok": true,
            "lucci_delta": entry.lucci_delta,
            "sku_id": entry.sku_id,
        }))),
        Ok(Decision::Replay(_))
        | Err(Error::Backend(
            accounts::Error::InsufficientFunds
            | accounts::Error::InsufficientQuantity
            | accounts::Error::Exhausted,
        )) => Err(Status::Conflict),
        Err(error) => Err(error.into()),
    }
}

async fn redeem_coupon(
    config: &Config,
    service: &AccountService,
    auth: &WebShopAuth,
    digest: &TokenDigest,
    request: &Request,
) -> Result<Response, Status> {
    let input: RedeemCouponInput = f4_mutation_input(
        config,
        request,
        service.settings().gift_coupon_policy().enabled,
    )?;
    let (player, coupon) = auth
        .with_session(digest, |session| {
            (
                session.player_id,
                session.coupons.get(&input.coupon_id).cloned(),
            )
        })
        .ok_or(Status::Unauthorized)?;
    let coupon = coupon
        .or_else(|| {
            if input.locally_issued != Some(true) {
                return None;
            }
            Some(LocalCoupon {
                account: input.issued_to?,
                offer: input.offer?,
                issued_at: input.issued_at?,
            })
        })
        .ok_or(Status::BadRequest)?;
    if input
        .locally_issued
        .is_some_and(|locally_issued| !locally_issued)
    {
        return Err(Status::BadRequest);
    }
    f4_mutation_result(
        service
            .redeem_gift_coupon(
                player,
                CouponRequest {
                    key: OperationKey {
                        account: coupon.account,
                        id: input.request_id,
                    },
                    coupon_id: input.coupon_id,
                    offer: &coupon.offer,
                    issued_to: coupon.account,
                    issued_at: coupon.issued_at,
                    now: 0, // AccountService substitutes the worker's UTC time.
                    locally_issued: true,
                },
            )
            .await,
    )
}

async fn erase_record(
    config: &Config,
    service: &AccountService,
    player: i64,
    request: &Request,
) -> Result<Response, Status> {
    let input: EraseRecordInput = f4_mutation_input(
        config,
        request,
        service.settings().record_eraser_policy().enabled,
    )?;
    f4_mutation_result(
        service
            .apply_record_eraser(
                player,
                RecordEraserOperation {
                    key: input.request_id,
                    sku: &input.sku,
                    scope: input.scope,
                },
            )
            .await,
    )
}

fn upgrade_roll() -> Result<u16, Status> {
    // Reject the tail of the u16 range to give each basis point equal weight.
    const LIMIT: u16 = u16::MAX - u16::MAX % BASIS_POINTS;
    loop {
        let mut bytes = [0; 2];
        getrandom::fill(&mut bytes).map_err(|_| Status::Internal)?;
        let value = u16::from_ne_bytes(bytes);
        if value < LIMIT {
            return Ok(value % BASIS_POINTS);
        }
    }
}

async fn upgrade_item(
    config: &Config,
    service: &AccountService,
    player: i64,
    request: &Request,
) -> Result<Response, Status> {
    let input: UpgradeItemInput =
        f4_mutation_input(config, request, service.settings().upgrade_policy().enabled)?;
    f4_mutation_result(
        service
            .apply_item_upgrade(
                player,
                ItemUpgradeOperation {
                    key: input.request_id,
                    sku: &input.sku,
                    family: input.family,
                    level: input.level,
                    roll_bps: upgrade_roll()?,
                },
            )
            .await,
    )
}

async fn titles_status(
    service: &AccountService,
    player: i64,
    request: &Request,
) -> Result<Response, Status> {
    request.require_queryless_get()?;
    let policy = service.settings().title_policy();
    if !policy.enabled {
        return Err(Status::Forbidden);
    }
    let mut titles = Vec::new();
    for row in policy.unlock_table.iter().flatten() {
        let eligible = service
            .evaluate_title_unlock(player, &row.title)
            .await?
            .is_some();
        titles.push(json!({"title": row.title, "eligible": eligible}));
    }
    Ok(Response::json(&json!({"titles": titles})))
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

const CUSTOM_MAP_WIRE_CONTRACT: &str = "unmeasured";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CustomMapSave {
    map_id: String,
    payload_base64: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CustomMapDelete {
    map_id: String,
}

// RFC 4648 standard base64, with padding and zero trailing bits required.
fn decode_map_payload(encoded: &str) -> Option<Vec<u8>> {
    fn digit(byte: u8) -> Option<u8> {
        Some(match byte {
            b'A'..=b'Z' => byte - b'A',
            b'a'..=b'z' => byte - b'a' + 26,
            b'0'..=b'9' => byte - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => return None,
        })
    }

    let (chunks, remainder) = encoded.as_bytes().as_chunks::<4>();
    if !remainder.is_empty() {
        return None;
    }
    let mut chunks = chunks.iter().peekable();
    let mut decoded = Vec::with_capacity(encoded.len() / 4 * 3);
    while let Some(&[a, b, c, d]) = chunks.next() {
        let (a, b) = (digit(a)?, digit(b)?);
        decoded.push((a << 2) | (b >> 4));
        if c == b'=' {
            return (chunks.peek().is_none() && d == b'=' && b & 15 == 0).then_some(decoded);
        }
        let c = digit(c)?;
        decoded.push((b << 4) | (c >> 2));
        if d == b'=' {
            return (chunks.peek().is_none() && c & 3 == 0).then_some(decoded);
        }
        decoded.push((c << 6) | digit(d)?);
    }
    Some(decoded)
}

fn custom_map_error(error: CustomMapStoreError) -> Response {
    use CustomMapStoreError as MapError;
    let (status, error) = match error {
        MapError::Disabled => (Status::Forbidden, "custom_maps_disabled"),
        MapError::InvalidMapId => (Status::BadRequest, "invalid_map_id"),
        MapError::InvalidMap(_) => (Status::BadRequest, "invalid_custom_map"),
        MapError::PayloadTooLarge => (Status::RequestTooLarge, "custom_map_too_large"),
        MapError::QuotaExceeded => (Status::Conflict, "custom_map_quota_exceeded"),
        MapError::NotFound => (Status::NotFound, "custom_map_not_found"),
        _ => (Status::Internal, "custom_maps_unavailable"),
    };
    Response::json_with_status(
        status,
        &json!({"error": error, "wire_contract": CUSTOM_MAP_WIRE_CONTRACT}),
    )
}

async fn custom_maps_route(
    shop: &WebShop,
    player: i64,
    request: &Request,
) -> Result<Response, Status> {
    let delete = request.path() == "/api/v1/custom-maps/delete";
    match (delete, request.method()) {
        (false, "GET" | "POST") | (true, "POST") => {}
        _ => return Err(Status::MethodNotAllowed),
    }
    let Some(store) = &shop.custom_maps else {
        return Ok(custom_map_error(CustomMapStoreError::Disabled));
    };

    enum Operation {
        List,
        Save { map_id: String, payload: Vec<u8> },
        Delete(String),
    }
    let operation = match (delete, request.method()) {
        (true, _) => Operation::Delete(request.json_object::<CustomMapDelete>()?.map_id),
        (false, "GET") => Operation::List,
        (false, _) => {
            let CustomMapSave {
                map_id,
                payload_base64,
            } = request.json_object()?;
            let payload = decode_map_payload(&payload_base64).ok_or(Status::BadRequest)?;
            Operation::Save { map_id, payload }
        }
    };

    // Keep filesystem work off the async runtime. The owned guard limits the
    // portal to one store operation even if the HTTP request times out.
    let store = store.clone().lock_owned().await;
    let owner = player.to_string();
    let result = spawn_blocking(move || -> Result<_, CustomMapStoreError> {
        Ok(match operation {
            Operation::List => json!({
                "maps": store.list_by_owner(&owner)?,
                "quota_remaining": store.quota_remaining(&owner)?,
                "wire_contract": CUSTOM_MAP_WIRE_CONTRACT,
            }),
            Operation::Save { map_id, payload } => json!({
                "map": store.save(&owner, &map_id, &payload)?,
                "wire_contract": CUSTOM_MAP_WIRE_CONTRACT,
            }),
            Operation::Delete(map_id) => {
                if !store.delete(&owner, &map_id)? {
                    return Err(CustomMapStoreError::NotFound);
                }
                json!({"removed": true, "wire_contract": CUSTOM_MAP_WIRE_CONTRACT})
            }
        })
    })
    .await
    .map_err(|_| Status::Internal)?;
    Ok(match result {
        Ok(body) => Response::json(&body),
        Err(error) => custom_map_error(error),
    })
}

async fn route(
    shop: &WebShop,
    service: &AccountService,
    auth: &WebShopAuth,
    request: &Request,
) -> Result<Response, Status> {
    let config = &shop.config;
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
    if path == "/api/v1/f3/pve-status" {
        auth.player(&digest).ok_or(Status::Unauthorized)?;
        request.require_queryless_get()?;
        return Ok(Response::json(&shop.f3_pve_registry.snapshot()));
    }
    if path == "/api/v1/events-status" {
        auth.player(&digest).ok_or(Status::Unauthorized)?;
        request.require_queryless_get()?;
        return Ok(events_status(shop.f6_policy.as_ref(), shop.f6_configured));
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
    let (player, milestone, primary_session) = auth
        .with_session(&digest, |session| {
            (
                session.player_id,
                session.milestone,
                session.primary_session,
            )
        })
        .ok_or(Status::Unauthorized)?;
    if path != "/api/v1/catalog" && query.is_some() {
        return Err(Status::BadRequest);
    }
    match path {
        "/api/v1/f4-extended-status" => {
            request.require_queryless_get()?;
            Ok(f4_extended_status(service))
        }
        "/api/v1/f4/security-kits-status" => {
            request.require_queryless_get()?;
            f4_security_kits_status(&shop.f4_security_kits, primary_session)
        }
        "/api/v1/f4/kits/equip" | "/api/v1/f4/kits/clear" => {
            update_powerup_kit(shop, service, auth, &digest, player, request).await
        }
        "/api/v1/f4/second-password/setup" | "/api/v1/f4/second-password/attempt" => {
            update_second_password(shop, auth, &digest, request).await
        }
        "/api/v1/f4/claim-milestone" => {
            claim_milestone(config, service, player, milestone, request).await
        }
        "/api/v1/f4/redeem-coupon" => redeem_coupon(config, service, auth, &digest, request).await,
        "/api/v1/f4/erase-record" => erase_record(config, service, player, request).await,
        "/api/v1/f4/upgrade-item" => upgrade_item(config, service, player, request).await,
        "/api/v1/f4/titles" => titles_status(service, player, request).await,
        "/api/v1/custom-maps" | "/api/v1/custom-maps/delete" => {
            custom_maps_route(shop, player, request).await
        }
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
    if request.path() == "/api/social/policy-status" {
        request.require_queryless_get()?;
        return social_policy_status(service, player).await;
    }
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
