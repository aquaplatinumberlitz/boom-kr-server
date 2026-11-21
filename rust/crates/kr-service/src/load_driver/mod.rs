//! Operator-driven, bounded acceptance workload for an isolated KR service.
mod client;
mod replay;

use client::Client;
use replay::Replay;
use serde::Deserialize;
use serde_json::json;
use std::{
    collections::HashSet,
    fs, io,
    net::SocketAddrV4,
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
pub(crate) struct Config {
    server: SocketAddrV4,
    relay: SocketAddrV4,
    // Loopback driver can use a local listener when the pinned offer still
    // advertises the public address for the native participants.
    relay_dial: Option<SocketAddrV4>,
    certificate: PathBuf,
    login_port: u16,
    channel_port: u16,
    replay: PathBuf,
    rooms: Vec<Vec<Account>>,
    ramp_ms: u64,
    active_seconds: u64,
    minimum_run_seconds: u64,
    minimum_rounds: u64,
    // Permit operator/controller orchestration with no public control endpoint.
    first_start_file: Option<PathBuf>,
    #[serde(default)]
    delivery_buckets: bool,
    delivery_p99_ms_max: f64,
    delivery_loss_percent_max: f64,
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
            || !(1.0..=1000.).contains(&self.delivery_p99_ms_max)
            || !(0.0..=5.).contains(&self.delivery_loss_percent_max)
            || self.login_port == self.channel_port
            || self.login_port == 0
            || self.channel_port == 0
            || [self.server, self.relay].iter().any(|addr| {
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
            players.push(
                Client::login(
                    &self.endpoint,
                    &self.config,
                    account,
                    companions,
                    cancel.clone(),
                )
                .await?,
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
            for p in &mut players[1..] {
                p.ready().await?;
            }
            for p in &mut *players {
                p.ready_updates(roster)
                    .await
                    .map_err(|e| phase("ready", e))?;
            }
            players[0].start().await?;
            for p in &mut *players {
                p.load().await.map_err(|e| phase("start/load", e))?;
            }
            let mut routes = Vec::with_capacity(roster);
            for (slot, p) in players.iter_mut().enumerate() {
                routes.push(
                    p.routes(slot, roster)
                        .await
                        .map_err(|e| phase("routes", e))?,
                );
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
            let passed = d.received > 0
                && d.p99_ms <= self.config.delivery_p99_ms_max
                && d.loss_percent <= self.config.delivery_loss_percent_max
                && d.pacing_late_over_20ms == 0;
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
                p.returned(roster).await.map_err(|e| phase("return", e))?;
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

async fn drive(config: Config, replay: Replay, certificate: Vec<u8>) -> io::Result<()> {
    config.validate()?;
    replay.validate()?;
    let start = Instant::now();
    if config.delivery_buckets {
        emit_clock_sample(start);
    }
    let first = Arc::new(Barrier::new(config.rooms.len()));
    let delivery_failed = Arc::new(AtomicBool::new(false));
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
    use std::os::unix::fs::PermissionsExt as _;
    let meta = fs::symlink_metadata(path)?;
    if !meta.is_file() || meta.permissions().mode() & 0o077 != 0 || meta.len() > 32768 {
        return Err(failure("configuration must be a private file (0600)"));
    }
    let raw = Zeroizing::new(fs::read(path)?);
    serde_json::from_slice::<Config>(&raw).map_err(|_| failure("invalid load configuration JSON"))
}

pub async fn execute(path: &Path) -> io::Result<()> {
    let config = private_config(path)?;
    if fs::metadata(&config.replay)?.len() > 1_000_000
        || fs::metadata(&config.certificate)?.len() > 8192
    {
        return Err(failure("oversized replay or certificate"));
    }
    let replay = serde_json::from_slice(&fs::read(&config.replay)?)
        .map_err(|_| failure("invalid replay JSON"))?;
    let certificate = fs::read(&config.certificate)?;
    drive(config, replay, certificate).await
}
