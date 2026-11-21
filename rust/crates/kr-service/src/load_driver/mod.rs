//! Operator-driven, bounded acceptance workload for an isolated KR service.
mod client;
mod replay;

use client::Client;
use kr_p2p::{
    Endpoint, Invite,
    self_host::{ClientSession, Network, RelayUrl},
};
use replay::Replay;
use serde::Deserialize;
use serde_json::json;
use std::{
    collections::HashSet,
    fs, io,
    net::{SocketAddr, SocketAddrV4},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tokio::{
    sync::{Barrier, watch},
    task::JoinSet,
    time::{Instant, sleep, sleep_until, timeout},
};
use zeroize::Zeroizing;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Account {
    login: String,
    name: String,
    password: Zeroizing<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MapChoice {
    code: u16,
    index: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Config {
    server: Option<SocketAddrV4>,
    relay: Option<SocketAddrV4>,
    s1: Option<S1>,
    // Loopback driver can use a local listener when the pinned offer still
    // advertises the public address for the native participants.
    relay_dial: Option<SocketAddrV4>,
    certificate: Option<PathBuf>,
    login_port: u16,
    channel_port: u16,
    replay: PathBuf,
    rooms: Vec<Vec<Account>>,
    ramp_ms: u64,
    active_seconds: u64,
    minimum_run_seconds: u64,
    minimum_rounds: u64,
    #[serde(default)]
    maps: Vec<MapChoice>,
    // Permit operator/controller orchestration with no public control endpoint.
    first_start_file: Option<PathBuf>,
    // Isolated TLS duel acceptance; one sequence of up to eight rounds.
    character_rounds: Option<Vec<[u16; 2]>>,
    #[serde(default)]
    check_fixed_items: bool,
    #[serde(default)]
    delivery_buckets: bool,
    delivery_p99_ms_max: f64,
    delivery_loss_percent_max: f64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct S1 {
    invite: String,
    hint: Option<SocketAddr>,
    #[serde(default)]
    local_only: bool,
    private_relay: Option<RelayUrl>,
    relay_ca: Option<PathBuf>,
    relay_quic_port: Option<u16>,
}

struct S1Dial {
    endpoint: Endpoint,
    network: Network,
    invite: Invite,
    hint: Option<SocketAddr>,
}

impl S1Dial {
    async fn new(config: &S1) -> io::Result<Self> {
        let invite = config.invite.parse().map_err(io::Error::other)?;
        let ca_der = if let Some(path) = &config.relay_ca {
            if fs::metadata(path)?.len() > 16384 {
                return Err(failure("private relay CA exceeds size limit"));
            }
            fs::read(path)?
        } else {
            Vec::new()
        };
        let network = match &config.private_relay {
            None if config.local_only
                && config.hint.is_some()
                && config.relay_ca.is_none()
                && config.relay_quic_port.is_none() =>
            {
                Network::Local
            }
            _ if config.local_only => return Err(failure("local S1 requires only a direct hint")),
            Some(relay) => Network::Private {
                relay: relay.clone(),
                quic_port: config.relay_quic_port.unwrap_or(7842),
                ca_der,
            },
            None if config.relay_ca.is_none() && config.relay_quic_port.is_none() => {
                Network::Public
            }
            None => return Err(failure("private relay options need a relay URL")),
        };
        let endpoint = network.endpoint("0.0.0.0:0".parse().unwrap(), None).await?;
        Ok(Self {
            endpoint,
            network,
            invite,
            hint: config.hint,
        })
    }

    async fn connect(&self) -> io::Result<ClientSession> {
        ClientSession::open(&self.endpoint, &self.network, &self.invite, self.hint).await
    }
}

fn failure(message: &'static str) -> io::Error {
    io::Error::other(message)
}

impl Config {
    fn validate(&self) -> io::Result<()> {
        if !(1..=8).contains(&self.rooms.len())
            || self.rooms.iter().any(|room| !(2..=8).contains(&room.len()))
            || self.rooms.iter().map(Vec::len).sum::<usize>() > 50
            || !(100..=10_000).contains(&self.ramp_ms)
            || !(2..=180).contains(&self.active_seconds)
            || self.minimum_run_seconds > 86400
            || !(1..=8).contains(&self.minimum_rounds)
            || (!self.maps.is_empty() && self.maps.len() != self.minimum_rounds as usize)
            || self.maps.iter().any(|m| m.index >= 16)
            || !(1.0..=1000.).contains(&self.delivery_p99_ms_max)
            || !(0.0..=5.).contains(&self.delivery_loss_percent_max)
            || self.login_port == self.channel_port
            || self.login_port == 0
            || self.channel_port == 0
            || match &self.s1 {
                Some(_) => {
                    self.server.is_some()
                        || self.relay.is_some()
                        || self.certificate.is_some()
                        || self.relay_dial.is_some()
                }
                None => self.server.is_none() || self.relay.is_none() || self.certificate.is_none(),
            }
            || [self.server, self.relay].into_iter().flatten().any(|addr| {
                addr.port() == 0
                    || addr.ip().is_unspecified()
                    || addr.ip().is_broadcast()
                    || addr.ip().is_multicast()
            })
            || self
                .relay_dial
                .is_some_and(|addr| !addr.ip().is_loopback() || addr.port() == 0)
        {
            return Err(failure("load configuration out of bounds"));
        }
        if let Some(rounds) = &self.character_rounds
            && (self.rooms.len() != 1
                || self.rooms[0].len() != 2
                || rounds.len() as u64 != self.minimum_rounds
                || self.minimum_run_seconds != 0
                || rounds.iter().flatten().any(|&key| key > 0x1fff))
        {
            return Err(failure("character acceptance requires a bounded TLS duel"));
        }
        if self.check_fixed_items
            && (self.rooms.len() != 1
                || self.rooms[0].len() != 2
                || self.maps.is_empty()
                || self.minimum_run_seconds != 0
                || self.character_rounds.is_some())
        {
            return Err(failure(
                "fixed floor acceptance requires a bounded map duel",
            ));
        }
        let mut logins = HashSet::new();
        for account in self.rooms.iter().flatten() {
            if !(4..=20).contains(&account.login.len())
                || !account.login.bytes().all(|b| b.is_ascii_alphanumeric())
                || !logins.insert(account.login.to_ascii_lowercase())
                || !(1..=20).contains(&account.name.len())
                || !account.name.bytes().all(|b| b.is_ascii_alphanumeric())
                || !(8..=64).contains(&account.password.len())
                || !account.password.bytes().all(|b| b.is_ascii_graphic())
            {
                return Err(failure("invalid synthetic account"));
            }
        }
        Ok(())
    }
}

struct RoomContext {
    config: Arc<Config>,
    replay: Arc<Replay>,
    endpoint: quinn::Endpoint,
    s1: Option<Arc<S1Dial>>,
    index: usize,
    offset: usize,
    start: Instant,
    first: Arc<Barrier>,
    delivery_failed: Arc<AtomicBool>,
}

impl RoomContext {
    async fn room(
        &self,
        companions: &mut JoinSet<io::Result<()>>,
        cancel: watch::Receiver<bool>,
        block: u64,
    ) -> io::Result<Vec<Client>> {
        let entries = &self.config.rooms[self.index];
        let mut players = Vec::with_capacity(entries.len());
        for (slot, account) in entries.iter().enumerate() {
            if block == 0 {
                sleep_until(
                    self.start
                        + Duration::from_millis(self.config.ramp_ms * (self.offset + slot) as u64),
                )
                .await;
            } else if slot > 0 {
                sleep(Duration::from_millis(self.config.ramp_ms)).await;
            }
            let bridge = match &self.s1 {
                Some(dial) => Some(dial.connect().await.map_err(|e| phase("S1 invite", e))?),
                None => None,
            };
            players.push(
                Client::login(
                    &self.endpoint,
                    &self.config,
                    account,
                    companions,
                    cancel.clone(),
                    bridge,
                )
                .await
                .map_err(|e| phase("player login", e))?,
            );
        }
        let room = players[0].create(&format!("Load{:02}", self.index)).await?;
        for (slot, player) in players.iter_mut().enumerate().skip(1) {
            player.join(room, slot).await?;
        }
        for player in &mut players {
            player.setup_room().await.map_err(|e| phase("setup", e))?;
        }
        // Each selection clears Ready. Wait for a full snapshot from each
        // change before sending Ready on any separate owner's stream.
        for slot in (1..players.len()).step_by(2) {
            players[slot].team().await?;
            let roster = players.len();
            for player in &mut players {
                player
                    .team_updates(roster, slot)
                    .await
                    .map_err(|e| phase("team", e))?;
            }
        }
        emit(
            json!({"event":"room_ready","room":self.index,"service_room":room,
                    "players":players.len(),"session_block":block}),
        );
        Ok(players)
    }

    async fn cycles(
        &self,
        players: &mut [Client],
        block: u64,
        completed: &mut u64,
        began: &mut Option<Instant>,
    ) -> io::Result<()> {
        let roster = players.len();
        if block == 0 {
            self.first.wait().await;
            if let Some(path) = &self.config.first_start_file {
                timeout(Duration::from_secs(180), async {
                    while !path.is_file() {
                        sleep(Duration::from_millis(100)).await;
                    }
                })
                .await?;
            }
        }
        for _ in 0..8 {
            if block != 0 && self.delivery_failed.load(Ordering::Relaxed) {
                break;
            }
            let map = self
                .config
                .maps
                .get((*completed % self.config.minimum_rounds) as usize);
            if let Some(map) = map {
                players[0].select_map(map.code, map.index).await?;
                for p in &mut *players {
                    p.map_selected(map.code, map.index).await?;
                }
            }
            let characters = self
                .config
                .character_rounds
                .as_ref()
                .map(|r| r[*completed as usize]);
            if let Some(keys) = characters {
                for (slot, key) in keys.into_iter().enumerate() {
                    players[slot].character(key).await?;
                    for p in &mut *players {
                        p.character_updates(slot, key).await?;
                    }
                }
            }
            for p in &mut players[1..] {
                p.ready().await?;
            }
            for p in &mut *players {
                p.ready_updates(roster)
                    .await
                    .map_err(|e| phase("ready", e))?;
            }
            players[0].start().await?;
            let mut floor = None;
            for (slot, p) in players.iter_mut().enumerate() {
                let received = p
                    .load(
                        characters,
                        map.map(|m| m.code),
                        self.config.check_fixed_items,
                    )
                    .await
                    .map_err(|e| phase("start/load", e))?;
                if slot == 0 {
                    floor = received;
                } else if self.config.check_fixed_items && floor != received {
                    return Err(failure("inconsistent round floor catalogs"));
                }
            }
            let mut routes = Vec::with_capacity(roster);
            let mut round_key = None;
            for (slot, p) in players.iter_mut().enumerate() {
                let (key, peers) = p
                    .routes(slot, roster)
                    .await
                    .map_err(|e| phase("routes", e))?;
                if round_key.replace(key).is_some_and(|prior| prior != key) {
                    return Err(failure("inconsistent round key"));
                }
                routes.push(peers);
            }
            if let Some(floor) = floor {
                let key = round_key.expect("duel key");
                if self.config.check_fixed_items {
                    let mut claimed = Vec::new();
                    for (i, item) in floor.visible_items().enumerate() {
                        let slot = i % 2;
                        players[slot].claim_item(slot, item, key).await?;
                        for p in &mut *players {
                            p.pickup_confirmed(slot, item).await?;
                        }
                        claimed.push([u16::from(item.1), u16::from(item.2), item.3]);
                    }
                    emit(
                        json!({"event":"fixed_items_confirmed","map":map.expect("map duel").code,
                                "items":claimed}),
                    );
                } else {
                    let mut count = [0; 3];
                    for item in floor.visible_basic_items() {
                        let n = &mut count[usize::from(item.3)];
                        if *n < 20 {
                            let slot = *n % 2;
                            players[slot].claim_item(slot, item, key).await?;
                            for p in &mut *players {
                                p.pickup_confirmed(slot, item).await?;
                            }
                            *n += 1;
                        }
                    }
                    if count != [20; 3] {
                        return Err(failure(
                            "character acceptance needs 20 visible items of each basic kind",
                        ));
                    }
                }
            }
            let warmup = Instant::now();
            replay::prime(players, &routes)
                .await
                .map_err(|e| phase("route warm-up", e))?;
            emit(
                json!({"event":"routes_warmed","room":self.index,"round":*completed+1,
                        "routes":roster*(roster-1),"duration_ms":warmup.elapsed().as_millis()}),
            );
            began.get_or_insert_with(Instant::now);
            emit(
                json!({"event":"round_active","room":self.index,"round":*completed+1,
                        "routes":roster*(roster-1),"at_unix_ms":unix_ms()}),
            );
            let d = replay::exchange(
                players,
                &routes,
                &self.replay,
                Duration::from_secs(self.config.active_seconds),
                self.offset + *completed as usize,
                ((self.index << 5) | (*completed as usize & 31)) as u8,
                self.config.delivery_buckets.then_some(self.start),
            )
            .await?;
            // Pacing lateness is retained in the telemetry for diagnosis. The
            // M8 relay gate is delivery p99 and loss; a scheduler slip in the
            // synthetic sender must not turn an otherwise delivered round
            // into a transport failure.
            let passed = d.received > 0
                && d.p99_ms <= self.config.delivery_p99_ms_max
                && d.loss_percent <= self.config.delivery_loss_percent_max;
            emit(
                json!({"event":"delivery","room":self.index,"round":*completed+1,
                        "at_unix_ms":unix_ms(),"passed":passed,"delivery":d}),
            );
            if !passed {
                self.delivery_failed.store(true, Ordering::Relaxed);
            }
            for (slot, p) in players.iter_mut().enumerate() {
                p.die(slot, *completed + 1).await?;
            }
            for p in &mut *players {
                p.finish().await.map_err(|e| phase("result", e))?;
            }
            for p in &mut *players {
                p.returned(roster, characters)
                    .await
                    .map_err(|e| phase("return", e))?;
            }
            *completed += 1;
            emit(json!({"event":"round_returned","room":self.index,"round":*completed}));
            if self.delivery_failed.load(Ordering::Relaxed) || self.done(*completed, *began) {
                break;
            }
        }
        // Leave guests first, then the host; assert the explicit return to
        // lobby and let the connection owner drop before checking health.
        for p in players.iter_mut().rev() {
            p.leave().await.map_err(|e| phase("lobby return", e))?;
        }
        Ok(())
    }

    fn done(&self, completed: u64, began: Option<Instant>) -> bool {
        completed >= self.config.minimum_rounds
            && began.is_some_and(|t| {
                t.elapsed() >= Duration::from_secs(self.config.minimum_run_seconds)
            })
    }

    async fn run(&self) -> io::Result<()> {
        let mut completed = 0;
        let mut block = 0;
        let mut began = None;
        loop {
            let (stop, cancel) = watch::channel(false);
            let mut companions = JoinSet::new();
            let mut players = self.room(&mut companions, cancel, block).await?;
            tokio::select! {
                biased;
                result = companions.join_next() => {
                    if let Some(result) = result {
                        result.map_err(io::Error::other)??;
                    }
                    return Err(failure("companion ended before planned close"));
                }
                result = self.cycles(&mut players, block, &mut completed, &mut began) => result?,
            }
            stop.send_replace(true);
            while let Some(result) = timeout(Duration::from_secs(5), companions.join_next()).await?
            {
                result.map_err(io::Error::other)??;
            }
            drop(players);
            emit(
                json!({"event":"room_closed","room":self.index,"rounds":completed,"session_block":block}),
            );
            if self.delivery_failed.load(Ordering::Relaxed) || self.done(completed, began) {
                return Ok(());
            }
            block += 1;
            sleep(Duration::from_secs(5)).await;
        }
    }
}

fn phase(stage: &str, e: io::Error) -> io::Error {
    io::Error::other(format!("{stage}: {e}"))
}

fn unix_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
fn emit(value: serde_json::Value) {
    println!("{value}");
}

fn emit_clock_sample(start: Instant) {
    let lower = start.elapsed();
    let wall = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let upper = start.elapsed();
    emit(json!({
        "event": "delivery_clock_sample",
        "elapsed_before_ns": lower.as_nanos(),
        "unix_ns": wall.as_nanos(),
        "elapsed_after_ns": upper.as_nanos()
    }));
}

async fn drive(config: Config, replay: Replay) -> io::Result<()> {
    config.validate()?;
    replay.validate()?;
    let start = Instant::now();
    if config.delivery_buckets {
        emit_clock_sample(start);
    }
    let first = Arc::new(Barrier::new(config.rooms.len()));
    let delivery_failed = Arc::new(AtomicBool::new(false));
    let (s1, certificate) = if let Some(config) = &config.s1 {
        let dial = S1Dial::new(config).await?;
        let certificate = dial
            .connect()
            .await
            .map_err(|e| phase("S1 TLS pin", e))?
            .certificate
            .clone();
        (Some(Arc::new(dial)), certificate)
    } else {
        let path = config.certificate.as_ref().expect("validated");
        if fs::metadata(path)?.len() > 8192 {
            return Err(failure("oversized certificate"));
        }
        (None, fs::read(path)?)
    };
    let endpoint = prelogin_endpoint(certificate)?;
    let config = Arc::new(config);
    let replay = Arc::new(replay);
    let mut jobs = JoinSet::new();
    let mut offset = 0;
    for (index, room) in config.rooms.iter().enumerate() {
        let context = RoomContext {
            config: config.clone(),
            replay: replay.clone(),
            endpoint: endpoint.clone(),
            s1: s1.clone(),
            index,
            offset,
            start,
            first: first.clone(),
            delivery_failed: delivery_failed.clone(),
        };
        jobs.spawn(async move {
            context
                .run()
                .await
                .map_err(|e| io::Error::other(format!("room {index}: {e}")))
        });
        offset += room.len();
    }
    while let Some(result) = jobs.join_next().await {
        result.map_err(io::Error::other)??;
    }
    endpoint.wait_idle().await;
    if config.delivery_buckets {
        emit_clock_sample(start);
    }
    if delivery_failed.load(Ordering::Relaxed) {
        return Err(failure("delivery gate failed after room cleanup"));
    }
    emit(json!({"event":"load_complete","at_unix_ms":unix_ms(),
                "players":offset,"rooms":config.rooms.len()}));
    Ok(())
}

fn prelogin_endpoint(certificate: Vec<u8>) -> io::Result<quinn::Endpoint> {
    kr_tunnel::prelogin::endpoint(certificate, kr_tunnel::UdpBackend::Native)
}

fn private_config(path: &Path) -> io::Result<Config> {
    let meta = crate::platform::private_file(path)?;
    if meta.len() > 32768 {
        return Err(failure("configuration must be a private file"));
    }
    let raw = Zeroizing::new(fs::read(path)?);
    serde_json::from_slice::<Config>(&raw).map_err(|_| failure("invalid load configuration JSON"))
}

pub async fn execute(path: &Path) -> io::Result<()> {
    let config = private_config(path)?;
    if fs::metadata(&config.replay)?.len() > 1_000_000 {
        return Err(failure("oversized replay"));
    }
    let replay = serde_json::from_slice(&fs::read(&config.replay)?)
        .map_err(|_| failure("invalid replay JSON"))?;
    drive(config, replay).await
}
