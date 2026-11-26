//! Account lobby/basic adapter with explicit fixed or DB zero-profile mode.
//! The launcher is private; this does not expose a public password API.
#[cfg(test)]
#[path = "f4a_economy_wire_tests.rs"]
mod f4a_economy_wire_tests;

use crate::{
    account_bridge::{self, Bridge},
    account_worker::{AccountService, Client, EconomySettings, Worker},
    accounts::{CatalogAccess, EconomyPolicy, ShopCatalog, Token},
    platform::{self, local_ipc},
};
use serde::Deserialize;
use std::{
    fs, io,
    net::Ipv4Addr,
    path::{Component, Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::{Semaphore, watch},
    task::JoinSet,
    time::timeout,
};
use zeroize::Zeroizing;

pub(crate) fn validate_projection(
    packets: &std::collections::HashMap<String, crate::Reply>,
    name: &[u8],
) -> io::Result<()> {
    use kr_wire::{Width, read_uint};
    let field = |bytes: &[u8], width| read_uint(bytes, width).map_err(io::Error::other);
    let body = &packets["own291"].payload;
    let size = field(&body[1..5], Width::U32)? as usize;
    if field(&body[..1], Width::U8)? != 1 || size != 108 + name.len() || body.len() != 13 + size {
        return Err(io::Error::other("unsupported account zero-profile body"));
    }
    let decoded = crate::lobby::decode_bytes(&body[5..5 + size])?;
    let mut expected = vec![0x18];
    expected.extend_from_slice(&((size - 5) as u32).to_le_bytes());
    expected.push(0);
    expected.extend_from_slice(&(name.len() as u32).to_le_bytes());
    expected.extend_from_slice(name);
    expected.extend([0; 98]);
    if decoded != expected
        || field(&body[5 + size..9 + size], Width::U32)? != 0
        || body[9 + size..]
            .iter()
            .any(|b| *b != kr_wire::write_uint(0, Width::U8).expect("constant")[0])
    {
        return Err(io::Error::other(
            "unsupported account zero-profile contents",
        ));
    }
    Ok(())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    pub backup: Option<crate::operations::BackupSchedule>,
    #[serde(default)]
    pub prelogin_tls: Option<crate::ingame_tls::Config>,
    #[serde(default)]
    pub public_launcher: Option<crate::public_launcher::Config>,
    #[serde(default)]
    pub persist_character: bool,
    #[serde(default)]
    pub persist_matches: bool,
    pub database: PathBuf,
    pub launch_socket: PathBuf,
    pub launch_ttl_seconds: u64,
    #[serde(default = "default_worker_queue_capacity")]
    pub worker_queue_capacity: usize,
    #[serde(default)]
    pub profile_mode: ProfileMode,
    /// All clients in this deployment must carry the owner's 949x patch.
    /// Default false retains ASCII-only native admission for unpatched clients.
    #[serde(default)]
    pub patched_949x_names: bool,
    #[serde(default)]
    pub max_sessions: Option<usize>,
    /// Diagnostic test accounts only, keyed by display name: value written into the
    /// auth reply's empty identity string (L1 room-title editing). Empty by default.
    #[serde(default)]
    pub diagnostic_identity_strings: std::collections::BTreeMap<String, String>,
    /// F4 match rewards. Absent means `enabled: false`: no ledger row is
    /// written and M6 `match_players.reward` stays 0. Enabling requires
    /// `persist_matches`, because only journaled matches settle.
    #[serde(default)]
    pub economy_policy: EconomyPolicy,
    /// F4 shop catalog. Absent means empty: every purchase/equip is rejected.
    #[serde(default)]
    pub shop_catalog: ShopCatalog,
    /// Optional external JSON file containing `{ economy_policy, shop_catalog, summary? }`
    /// (e.g., emitted by `tools/kr_f4_catalog_export.py`). Must not be combined with
    /// non-default inline `economy_policy` or `shop_catalog`.
    #[serde(default)]
    pub economy_profile_path: Option<PathBuf>,
    /// Opt-in F4-a projection: emit static-confirmed `S->C 135` (`remain_lucci`)
    /// after lobby `own291` and settled post-match `175` replies.
    #[serde(default)]
    pub emit_lucci_135: bool,
    /// Opt-in F4-a projection: write DB `profile.level` (`Lv.%3d` at
    /// `UserSimpleInfo` offset `+11..+13`) into outgoing `S->C 291` (`own291`).
    #[serde(default)]
    pub project_own291_level: bool,
    /// Opt-in O1 preset flag: expose `CatalogAccess { unlock_all: true }` from
    /// validated DB account mode without mutating DB inventory or Lucci.
    #[serde(default)]
    pub unlock_all_catalog: bool,
    /// Optional embedded F4-d HTTP web shop listener bound to DB account mode.
    #[serde(default)]
    pub web_shop: Option<crate::web_shop::Config>,
}
#[derive(Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProfileMode {
    #[default]
    Fixed,
    DatabaseZero,
}
fn default_worker_queue_capacity() -> usize {
    4
}
const MAX_ECONOMY_PROFILE_BYTES: u64 = 16 * 1024 * 1024;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EconomyProfileFile {
    economy_policy: EconomyPolicy,
    shop_catalog: ShopCatalog,
    #[serde(default)]
    summary: Option<serde::de::IgnoredAny>,
}

fn load_economy_profile_file(path: &Path) -> io::Result<(EconomyPolicy, ShopCatalog)> {
    if path.as_os_str().is_empty()
        || path
            .components()
            .any(|part| matches!(part, Component::ParentDir))
    {
        return Err(io::Error::other("invalid economy_profile_path"));
    }
    let meta = fs::metadata(path)?;
    if !meta.is_file() || meta.len() > MAX_ECONOMY_PROFILE_BYTES {
        return Err(io::Error::other("economy profile file size out of bounds"));
    }
    let bytes = fs::read(path)?;
    let parsed: EconomyProfileFile = serde_json::from_slice(&bytes)
        .map_err(|_| io::Error::other("invalid economy profile json"))?;
    let _ = parsed.summary;
    Ok((parsed.economy_policy, parsed.shop_catalog))
}

impl Config {
    /// Validated economy for the account worker; defaults stay disabled.
    pub(crate) fn economy(&self) -> io::Result<EconomySettings> {
        let (policy, catalog) = if let Some(path) = &self.economy_profile_path {
            if self.economy_policy != EconomyPolicy::default()
                || !self.shop_catalog.items.is_empty()
            {
                return Err(io::Error::other(
                    "economy_profile_path cannot be combined with inline economy config",
                ));
            }
            load_economy_profile_file(path)?
        } else {
            (self.economy_policy.clone(), self.shop_catalog.clone())
        };
        let settings = EconomySettings::new(policy, catalog)
            .map_err(|_| io::Error::other("invalid economy policy or shop catalog"))?;
        if settings.rewards_enabled() && !self.persist_matches {
            return Err(io::Error::other(
                "economy rewards require persisted matches",
            ));
        }
        if (settings.rewards_enabled()
            || !settings.catalog().items.is_empty()
            || self.emit_lucci_135
            || self.project_own291_level
            || self.unlock_all_catalog)
            && self.profile_mode != ProfileMode::DatabaseZero
        {
            return Err(io::Error::other("economy requires database profiles"));
        }
        Ok(settings)
    }
    pub(crate) fn validate(&self) -> io::Result<()> {
        self.economy()?;
        if let Some(backup) = &self.backup {
            backup.validate()?;
        }
        if let Some(tls) = &self.prelogin_tls {
            if self.profile_mode != ProfileMode::DatabaseZero || !self.patched_949x_names {
                return Err(io::Error::other("in-game TLS requires patched DB profiles"));
            }
            tls.validate()?;
        }
        if self.patched_949x_names && self.profile_mode != ProfileMode::DatabaseZero {
            return Err(io::Error::other("949x names require database profiles"));
        }
        if let Some(public) = &self.public_launcher {
            if self.profile_mode != ProfileMode::DatabaseZero {
                return Err(io::Error::other(
                    "public launcher requires database profiles",
                ));
            }
            public.validate()?;
        }
        if let Some(web_shop) = &self.web_shop {
            if self.profile_mode != ProfileMode::DatabaseZero {
                return Err(io::Error::other("web shop requires database profiles"));
            }
            if web_shop.unlock_all != self.unlock_all_catalog {
                return Err(io::Error::other(
                    "web_shop.unlock_all must match unlock_all_catalog",
                ));
            }
            web_shop.validate()?;
        }
        if ((self.persist_character || self.persist_matches)
            && self.profile_mode != ProfileMode::DatabaseZero)
            || !(30..=120).contains(&self.launch_ttl_seconds)
            || self.database == self.launch_socket
            || !(1..=64).contains(&self.worker_queue_capacity)
            || self.max_sessions.is_some_and(|n| !(1..=128).contains(&n))
            || (self.profile_mode == ProfileMode::Fixed && self.max_sessions.is_some())
            || self.diagnostic_identity_strings.len() > 8
        {
            return Err(io::Error::other("invalid account serving bounds"));
        }
        for path in [&self.database, &self.launch_socket] {
            if !path.is_absolute()
                || path
                    .components()
                    .any(|c| matches!(c, std::path::Component::ParentDir))
            {
                return Err(io::Error::other(
                    "account paths must be absolute trusted paths",
                ));
            }
            let parent = path
                .parent()
                .ok_or_else(|| io::Error::other("account parent missing"))?;
            platform::private_dir(parent)?;
        }
        platform::private_file(&self.database)?;
        Ok(())
    }
}
#[derive(Clone)]
pub(crate) struct Active {
    pub(crate) authenticated: Option<Arc<crate::authenticated_peer::Hub>>,
    pub bridge: Bridge,
    pub client: Client,
    pub(crate) service: Option<AccountService>,
    pub(crate) emit_lucci_135: bool,
    pub(crate) project_own291_level: bool,
    pub(crate) unlock_all_catalog: bool,
    pub(crate) web_shop: Option<crate::web_shop::WebShopAuth>,
    projections: Arc<std::collections::HashMap<u32, Vec<u8>>>,
    projection: Option<Arc<kr_wire::kr00967::AccountTemplate>>,
}
impl Active {
    #[cfg(test)]
    pub(crate) fn login_test_client(client: Client) -> Self {
        Self {
            authenticated: None,
            bridge: Bridge::new(client.clone(), 4, Duration::from_secs(30)).unwrap(),
            client,
            service: None,
            emit_lucci_135: false,
            project_own291_level: false,
            unlock_all_catalog: false,
            web_shop: None,
            projections: Arc::new(std::collections::HashMap::new()),
            projection: None,
        }
    }
    pub(crate) fn bridge(&self) -> Bridge {
        self.bridge.clone()
    }
    pub(crate) fn account_service(&self) -> Option<&AccountService> {
        self.service.as_ref()
    }
    pub(crate) fn catalog_access(&self) -> CatalogAccess {
        CatalogAccess {
            unlock_all: self.unlock_all_catalog,
        }
    }
    pub(crate) fn match_items(&self, persist_matches: bool) -> Option<MatchItems> {
        let service = self.account_service()?;
        if !self.unlock_all_catalog && !(persist_matches && service.settings().rewards_enabled()) {
            return None;
        }
        Some(MatchItems {
            service: service.clone(),
            access: self.catalog_access(),
        })
    }
    pub(crate) async fn issue_web_shop_token(
        &self,
        token: &Token,
        player: i64,
    ) -> Option<crate::web_shop::WebShopToken> {
        let auth = self.web_shop.as_ref()?;
        if self.unlock_all_catalog {
            let _operation = auth.launcher_operation().await;
            let (hard, idle) = self.client.session_deadlines(token, player).await.ok()?;
            let observed = std::time::Instant::now();
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .ok()?;
            let deadline = Duration::from_secs(u64::try_from(idle.unwrap_or(hard).min(hard)).ok()?);
            let lifetime = deadline.checked_sub(now)?;
            auth.issue_until(player, Some(observed.checked_add(lifetime)?))
                .ok()
        } else {
            let authorized = self
                .client
                .try_authorize_current(token, player)
                .ok()?
                .wait()
                .await
                .ok()?;
            (authorized.player_id == player)
                .then(|| auth.issue_token(player).ok())
                .flatten()
        }
    }

    pub(crate) async fn logout_public(&self, token: Token, player: i64) -> io::Result<bool> {
        if let Some(auth) = self.web_shop.as_ref().filter(|_| self.unlock_all_catalog) {
            let operation = auth.launcher_operation().await;
            let active = self.clone();
            // Finish revocation even if the RPC times out while SQLite is
            // committing the logout. The operation guard bounds this task.
            return tokio::spawn(async move {
                let _operation = operation;
                active.finish_public_logout(&token, player).await
            })
            .await
            .map_err(|_| io::Error::other("public logout task unavailable"))?;
        }
        self.finish_public_logout(&token, player).await
    }

    async fn finish_public_logout(&self, token: &Token, player: i64) -> io::Result<bool> {
        let logged_out = match self.client.try_logout_current(token, player) {
            Ok(pending) => pending.wait().await.is_ok(),
            Err(_) => false,
        };
        if logged_out {
            if let Some(auth) = &self.web_shop {
                auth.revoke_player(player);
            }
            self.bridge
                .forget_session(token, player)
                .map_err(|_| io::Error::other("public logout registry unavailable"))?;
        }
        Ok(logged_out)
    }
}
#[derive(Clone)]
pub(crate) struct MatchItems {
    pub(crate) service: AccountService,
    pub(crate) access: CatalogAccess,
}
pub(crate) struct Owner {
    pub(crate) prelogin_tls: Option<quinn::Endpoint>,
    pub(crate) public_launcher: Option<crate::public_launcher::Opened>,
    pub(crate) web_shop: Option<crate::web_shop::Opened>,
    pub(crate) boot_epoch: Option<i64>,
    worker: Worker,
    live_worker: Option<Worker>,
    pub active: Active,
    pub listener: Option<local_ipc::Listener>,
    _instance: fs::File,
}
impl Owner {
    pub async fn open(
        config: &Config,
        projections: std::collections::HashMap<u32, Vec<u8>>,
        projection: Option<Arc<kr_wire::kr00967::AccountTemplate>>,
    ) -> io::Result<Self> {
        config.validate()?;
        if (config.profile_mode == ProfileMode::DatabaseZero) != projection.is_some() {
            return Err(io::Error::other(
                "account projection mode/template mismatch",
            ));
        }
        let instance = crate::operations::lock_database(&config.database)?;
        let listener = local_ipc::Listener::bind(&config.launch_socket)?;
        let public_launcher = match &config.public_launcher {
            Some(public) => Some(public.open().await?),
            None => None,
        };
        let web_shop = match &config.web_shop {
            Some(ws) => Some(crate::web_shop::open(ws.clone()).await?),
            None => None,
        };
        let prelogin_tls = config
            .prelogin_tls
            .as_ref()
            .map(crate::ingame_tls::Config::open)
            .transpose()?;
        let worker = Worker::open_with_economy(
            config.database.clone(),
            config.worker_queue_capacity,
            Duration::from_secs(3),
            config.economy()?,
        )
        .await
        .map_err(|e| io::Error::other(format!("account worker startup: {e:?}")))?;
        let boot_epoch = if projection.is_some() {
            Some(
                worker
                    .client()
                    .try_begin_instance()
                    .map_err(|e| io::Error::other(format!("account instance startup: {e:?}")))?
                    .wait()
                    .await
                    .map_err(|e| io::Error::other(format!("account instance start: {e:?}")))?,
            )
        } else {
            None
        };
        let mut bridge = match Bridge::new(
            worker.client(),
            if projection.is_some() {
                config.max_sessions.unwrap_or(32)
            } else {
                projections.len()
            },
            Duration::from_secs(config.launch_ttl_seconds),
        ) {
            Ok(bridge) => bridge,
            Err(error) => {
                let _ = worker.shutdown().await;
                return Err(io::Error::other(format!(
                    "account bridge startup: {error:?}"
                )));
            }
        };
        let live_worker = if projection.is_some() {
            let live = Worker::open_live(config.database.clone(), 64, Duration::from_secs(1))
                .await
                .map_err(|error| {
                    io::Error::other(format!("live authorization startup: {error:?}"))
                })?;
            bridge = bridge.with_live_authorization(live.client());
            Some(live)
        } else {
            None
        };
        bridge = bridge
            .with_patched_949x_names(config.patched_949x_names)
            .with_unlock_all(config.unlock_all_catalog);
        Ok(Self {
            prelogin_tls,
            public_launcher,
            boot_epoch,
            active: Active {
                authenticated: None,
                bridge,
                client: worker.client(),
                service: Some(worker.service()),
                emit_lucci_135: config.emit_lucci_135,
                project_own291_level: config.project_own291_level,
                unlock_all_catalog: config.unlock_all_catalog,
                web_shop: web_shop.as_ref().map(crate::web_shop::Opened::auth),
                projections: Arc::new(projections),
                projection,
            },
            web_shop,
            worker,
            live_worker,
            listener: Some(listener),
            _instance: instance,
        })
    }
    pub async fn shutdown(self) -> io::Result<()> {
        if let Some(live) = self.live_worker {
            live.shutdown()
                .await
                .map_err(|e| io::Error::other(format!("live authorization shutdown: {e:?}")))?;
        }
        self.worker
            .shutdown()
            .await
            .map_err(|e| io::Error::other(format!("account worker shutdown: {e:?}")))
    }
}
pub(crate) struct Secret(pub(crate) Zeroizing<String>);
impl<'de> Deserialize<'de> for Secret {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        String::deserialize(d).map(|s| Self(Zeroizing::new(s)))
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct LaunchRequest {
    pub(crate) session: Secret,
    pub(crate) player_id: i64,
    pub(crate) peer: Ipv4Addr,
    #[serde(default)]
    pub(crate) reported_login_ipv4: Option<Ipv4Addr>,
}
pub(crate) struct PreparedLaunch {
    response: Zeroizing<String>,
    launch: Option<crate::account_bridge::Launch>,
}
impl PreparedLaunch {
    pub(crate) fn bytes(&self) -> &[u8] {
        self.response.as_bytes()
    }
    pub(crate) fn delivered(self) {
        if let Some(launch) = self.launch {
            launch.delivered();
        }
    }
}
async fn issue(
    stream: &mut local_ipc::Stream,
    active: &Active,
    peers: &[Ipv4Addr],
) -> io::Result<()> {
    let mut input = Zeroizing::new(Vec::new());
    let mut chunk = Zeroizing::new([0u8; 256]);
    loop {
        let n = stream.read(&mut chunk[..]).await?;
        if n == 0 {
            return Err(io::Error::other("incomplete private launch request"));
        }
        input.extend_from_slice(&chunk[..n]);
        if input.len() > 4096 {
            return Err(io::Error::other("private launch request bound"));
        }
        if input.contains(&b'\n') {
            if input.iter().position(|b| *b == b'\n') != Some(input.len() - 1) {
                return Err(io::Error::other(
                    "one launch request per control connection",
                ));
            }
            break;
        }
    }
    let request: LaunchRequest = serde_json::from_slice(&input)
        .map_err(|_| io::Error::other("invalid private launch request"))?;
    let prepared = prepare_launch(request, active, peers).await?;
    stream.write_all(prepared.bytes()).await?;
    prepared.delivered();
    Ok(())
}
pub(crate) async fn prepare_launch(
    request: LaunchRequest,
    active: &Active,
    peers: &[Ipv4Addr],
) -> io::Result<PreparedLaunch> {
    if !peers.contains(&request.peer) {
        return Err(io::Error::other("launch peer outside allowlist"));
    }
    let session = Token::parse(&request.session.0)
        .map_err(|_| io::Error::other("invalid private launch session"))?;
    if active.projection.is_none() && request.reported_login_ipv4.is_some() {
        return Err(io::Error::other(
            "fixed profile metadata must be configured",
        ));
    }
    let issued = active
        .bridge
        .issue_metadata(
            &session,
            request.player_id,
            request.peer,
            request.reported_login_ipv4,
        )
        .await;
    let launch = match issued {
        Ok(launch) => launch,
        Err(account_bridge::Error::Backend(
            error @ (crate::account_worker::Error::Busy
            | crate::account_worker::Error::QueueExpired
            | crate::account_worker::Error::Backend(crate::accounts::Error::Busy)),
        )) => {
            let reason = match error {
                crate::account_worker::Error::Busy => "account_queue_full",
                crate::account_worker::Error::QueueExpired => "account_queue_expired",
                _ => "database_busy",
            };
            return Ok(PreparedLaunch {
                response: Zeroizing::new(format!(
                    "{{\"ok\":false,\"error\":\"busy_retry\",\"reason\":\"{reason}\"}}\n"
                )),
                launch: None,
            });
        }
        Err(error) => {
            return Err(io::Error::other(format!(
                "private launch denied: {error:?}"
            )));
        }
    };
    // Fixed mode keeps its fixture fence. Database mode uses the authenticated
    // player's bounded zero-profile template, never an IP-selected identity.
    let launch_name = kr_wire::kr00967::text949x::encode_name(&launch.player.display_name)?;
    if active.projection.is_none()
        && (launch.player.profile_version != 1
            || active
                .projections
                .get(&launch.player.wire_id)
                .is_none_or(|name| name.as_slice() != launch_name))
    {
        return Err(io::Error::other("unsupported account profile projection"));
    }
    let player = serde_json::to_string(&launch.player)
        .map_err(|_| io::Error::other("player serialization failed"))?;
    let companion = active
        .authenticated
        .as_ref()
        .map(|hub| {
            hub.offer(
                kr_tunnel::router::Owner {
                    player: launch.player.wire_id,
                    session: launch.generation(),
                },
                Duration::from_secs(60),
            )
        })
        .transpose()?;
    let companion = if let Some(offer) = companion {
        Zeroizing::new(format!(
            ",\"peer_transport\":{}",
            serde_json::to_string(&offer).map_err(io::Error::other)?
        ))
    } else {
        Zeroizing::new(String::new())
    };
    let response = Zeroizing::new(format!(
        "{{\"ok\":true,\"player\":{player},\"frontend\":\"{}\"{}}}\n",
        launch.token().expose_secret(),
        *companion
    ));
    Ok(PreparedLaunch {
        response,
        launch: Some(launch),
    })
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn connection(
    mut socket: tokio::net::TcpStream,
    id: u64,
    profile: &crate::Validated,
    active: &Active,
    log: crate::Log,
    mut stop: watch::Receiver<bool>,
    start: std::time::Instant,
) -> io::Result<()> {
    use kr_wire::{Receiver, Width, encode, write_uint};
    let peer = match socket.peer_addr()?.ip() {
        std::net::IpAddr::V4(peer) => peer,
        _ => return Err(io::Error::other("account IPv4 profile only")),
    };
    let endpoints = profile
        .config
        .diagnostic_two_lobby
        .as_ref()
        .expect("validated account endpoints");
    let bound = profile.bound.as_ref().expect("validated account profiles");
    let port = socket.local_addr()?.port();
    socket.set_nodelay(true)?;
    let write_limit = Duration::from_millis(profile.config.write_timeout_ms);
    timeout(write_limit, socket.write_all(&profile.bootstrap))
        .await
        .map_err(|_| io::Error::other("account hello timeout"))??;
    let mut rx = Receiver::new(40);
    let mut buf = [0u8; 4096];
    loop {
        if let Some(packet) = rx.next_packet().map_err(io::Error::other)? {
            if packet.opcode == 0
                && packet.payload.len() == 28
                && endpoints.wcp_ports.contains(&port)
            {
                if rx.next_packet().map_err(io::Error::other)?.is_some() {
                    return Err(io::Error::other("extra account WCP request"));
                }
                timeout(write_limit, socket.write_all(&profile.wcp_second))
                    .await
                    .map_err(|_| io::Error::other("account WCP timeout"))??;
                let n = tokio::select! {_=stop.changed()=>return Ok(()),v=timeout(Duration::from_secs(profile.config.idle_timeout_seconds),socket.read(&mut buf))=>v.map_err(|_|io::Error::other("account WCP idle"))??};
                return if n == 0 {
                    rx.finish().map_err(io::Error::other)
                } else {
                    Err(io::Error::other("extra account WCP data"))
                };
            }
            let (owned, lease, echo) = if packet.opcode == 1
                && (port == endpoints.login_port || endpoints.login_candidate_ports.contains(&port))
            {
                let result = tokio::select! {
                    _=stop.changed()=>return Ok(()),
                    v=active.bridge.login_for_profile(&packet.payload, peer, id, |player| {
                        if active.projection.is_some() { return Some(peer); }
                        bound.actors.get(&player.wire_id)
                            .filter(|actor| kr_wire::kr00967::text949x::encode_name(&player.display_name)
                                .is_ok_and(|name| actor.name == name))
                            .map(|actor| actor.reported_login_ipv4.unwrap_or(peer))
                    })=>v
                };
                let login = match result {
                    Ok(login) => login,
                    Err(_) => {
                        let reply = encode(&profile.error_header, 4, &[0x5b], 43)
                            .map_err(io::Error::other)?;
                        timeout(write_limit, socket.write_all(&reply))
                            .await
                            .map_err(|_| io::Error::other("account denial timeout"))??;
                        crate::emit(&log,crate::stamp(start,serde_json::json!({"event":"account_login_denied","connection":id,"policy":"failclosed_no_identity_grant"}))).await?;
                        // Keep only measured ancillary60/15 after result1, no retry/resume assumption.
                        let mut budget = 0;
                        loop {
                            while let Some(packet) = rx.next_packet().map_err(io::Error::other)? {
                                if !matches!(
                                    (packet.opcode, packet.payload.len()),
                                    (60, 20) | (15, 31)
                                ) || budget >= 64
                                {
                                    return Err(io::Error::other(
                                        "account rejected ancillary bound",
                                    ));
                                }
                                budget += 1;
                            }
                            let cap = buf.len().min(rx.remaining_capacity());
                            if cap == 0 {
                                return Err(io::Error::other("account denial receive bound"));
                            }
                            let n = tokio::select! {_=stop.changed()=>return Ok(()),v=timeout(Duration::from_secs(profile.config.idle_timeout_seconds),socket.read(&mut buf[..cap]))=>v.map_err(|_|io::Error::other("account denial idle"))??};
                            if n == 0 {
                                return rx.finish().map_err(io::Error::other);
                            }
                            rx.push(&buf[..n]).map_err(io::Error::other)?;
                        }
                    }
                };
                (login.player.clone(), Lease::Login(Some(login)), true)
            } else if packet.opcode == 6 && port == endpoints.channel_port {
                let channel = tokio::select! {_=stop.changed()=>return Ok(()),v=active.bridge.channel(&packet.payload,peer,id)=>v.map_err(|e|io::Error::other(format!("account channel rejected: {e:?}")))?};
                (channel.player.clone(), Lease::Channel(channel), false)
            } else {
                return Err(io::Error::other("account first opcode/endpoint rejected"));
            };
            let owned_name = kr_wire::kr00967::text949x::encode_name(&owned.display_name)?;
            let actor = if active.projection.is_some() {
                bound.actors.values().min_by_key(|actor| actor.identity)
            } else {
                bound
                    .actors
                    .values()
                    .find(|a| a.identity == owned.wire_id && a.name == owned_name)
                    .filter(|_| owned.profile_version == 1)
            }
            .ok_or_else(|| io::Error::other("unsupported owned account profile"))?;
            // The issued frontend and channel lease own the identity. Source IPv4
            // remains a transport restriction, never a selector for room ownership.
            let mut packets = actor.packets.clone();
            if let Some(template) = &active.projection {
                let rendered = template.project(owned.wire_id, &owned_name, 0)?;
                let own_payload = if active.project_own291_level {
                    let svc = active
                        .service
                        .as_ref()
                        .ok_or_else(|| io::Error::other("own291 level service absent"))?;
                    let profile = svc
                        .profile_get(owned.player_id)
                        .await
                        .map_err(|_| io::Error::other("own291 level profile lookup"))?;
                    let level = kr_wire::kr00967::wire_level_u16(profile.level)?;
                    let exp_u64 = u64::try_from(profile.exp)
                        .map_err(|_| io::Error::other("negative profile exp"))?;
                    let exp_permille = svc.settings().policy().exp_permille_for(exp_u64);
                    kr_wire::kr00967::own291_with_progression(&owned_name, level, exp_permille)?
                } else {
                    rendered.own291
                };
                packets
                    .get_mut("auth_A")
                    .expect("validated auth template")
                    .payload = rendered.auth;
                packets
                    .get_mut("own291")
                    .expect("validated own template")
                    .payload = own_payload;
            }
            if let Some(id) = endpoints
                .account_admission
                .as_ref()
                .and_then(|cfg| cfg.diagnostic_identity_strings.get(&owned.display_name))
            {
                let auth = packets.get_mut("auth_A").expect("validated auth reply");
                auth.payload = kr_wire::kr00967::with_identity_string(
                    &auth.payload,
                    owned_name.len(),
                    id.as_bytes(),
                )?;
            }
            if echo {
                let Lease::Login(Some(login)) = &lease else {
                    unreachable!()
                };
                // Echo u32 sits 44 bytes before the end (86 + name for the base layout).
                let offset = packets["auth_A"].payload.len() - 44;
                packets
                    .get_mut("auth_A")
                    .expect("validated auth reply")
                    .payload[offset..offset + 4]
                    .copy_from_slice(
                        &write_uint(login.echo.into(), Width::U32).map_err(io::Error::other)?,
                    );
            }
            crate::emit(&log,crate::stamp(start,serde_json::json!({"event":"account_binding","connection":id,"player_id":owned.player_id,"identity":owned.wire_id,"profile_version":owned.profile_version,"transport":if echo{"login"}else{"channel"},"policy":"backend_owned_fixed_zero_profile"}))).await?;
            let flow = crate::lobby::Flow::for_identity(!echo, owned.wire_id, &owned_name);
            return crate::local_lobby_connection(
                socket,
                id,
                profile,
                &packets,
                flow,
                None,
                Some(lease),
                Some((rx, packet)),
                log,
                stop,
                start,
            )
            .await;
        }
        let cap = buf.len().min(rx.remaining_capacity());
        if cap == 0 {
            return Err(io::Error::other("account receive bound"));
        }
        let n = tokio::select! {_=stop.changed()=>return Ok(()),v=timeout(Duration::from_secs(profile.config.idle_timeout_seconds),socket.read(&mut buf[..cap]))=>v.map_err(|_|io::Error::other("account first request idle"))??};
        if n == 0 {
            return rx.finish().map_err(io::Error::other);
        }
        rx.push(&buf[..n]).map_err(io::Error::other)?;
    }
}
pub(crate) async fn control(
    listener: local_ipc::Listener,
    active: Active,
    peers: Vec<Ipv4Addr>,
    mut stop: watch::Receiver<bool>,
) -> io::Result<()> {
    let listener = Arc::new(listener);
    let permits = Arc::new(Semaphore::new(4));
    let peers = Arc::new(peers);
    let mut jobs = JoinSet::new();
    loop {
        tokio::select! {
            _=stop.changed()=>break,
            result=jobs.join_next(), if !jobs.is_empty()=>{ result.expect("nonempty jobs").map_err(io::Error::other)?; },
            pair=listener.accept()=>{
                let mut stream=pair?;
                let Ok(permit)=permits.clone().try_acquire_owned() else { drop(stream); continue; };
                let active=active.clone();let peers=peers.clone();let mut stop=stop.clone();
                let listener=listener.clone();
                jobs.spawn(async move {
                    let _permit=permit;
                    let result=tokio::select! {
                        _=stop.changed()=>return,
                        result=timeout(Duration::from_secs(3),async {
                            listener.authenticate(&mut stream).await?;
                            issue(&mut stream,&active,&peers).await
                        })=>result,
                    };
                    if !matches!(result,Ok(Ok(()))) {
                        let response:&[u8]=if result.is_err(){b"{\"ok\":false,\"error\":\"busy_retry\"}\n"}else{b"{\"ok\":false,\"error\":\"denied\"}\n"};
                        let _=timeout(Duration::from_millis(200),stream.write_all(response)).await;
                    }
                });
            }
        }
    }
    jobs.abort_all();
    while jobs.join_next().await.is_some() {}
    Ok(())
}
pub(crate) enum Lease {
    Login(Option<account_bridge::Login>),
    Channel(account_bridge::Channel),
}
impl Lease {
    pub async fn authorize(&mut self, activity: bool) -> io::Result<()> {
        if let Self::Channel(channel) = self {
            let result = if activity {
                channel.check().await
            } else {
                channel.check_idle().await
            };
            result.map_err(|e| io::Error::other(format!("account channel denied: {e:?}")))?;
        }
        Ok(())
    }
    pub fn redirected(&mut self) -> io::Result<()> {
        if let Self::Login(login) = self {
            login
                .take()
                .ok_or_else(|| io::Error::other("account redirect already delivered"))?
                .redirected()
                .map_err(|e| io::Error::other(format!("account redirect failed: {e:?}")))?;
        }
        Ok(())
    }
}
