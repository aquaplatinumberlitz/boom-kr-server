//! Local operational samples. No traffic payloads or session credentials.
use crate::{accounts, authenticated_peer, room_service};
use serde::Serialize;
use serde_json::{Value, json};
use std::{
    io,
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::watch;

const PERIOD: Duration = Duration::from_secs(5);
const MAX_AGE: i64 = 15;
const SCHEMA: &str = "kr-health-v1";

#[derive(Serialize)]
pub(crate) struct Room {
    pub room_id: u32,
    pub room_session: u16,
    pub generation: u64,
    pub phase: &'static str,
    pub round: Option<u64>,
}
#[derive(Serialize)]
pub(crate) struct Lobby {
    pub online_players: usize,
    pub room_count: usize,
    pub rooms: Vec<Room>,
    pub active_rounds: usize,
    pub loading_rounds: usize,
}
#[derive(Serialize)]
pub(crate) struct Relay {
    pub udp_listeners: usize,
    pub companions: usize,
    pub reserved_routes: usize,
    pub received: u64,
    pub admitted: u64,
    pub rejected: u64,
    pub expired_in_pump: u64,
    pub submitted_to_quic: u64,
}

pub(crate) struct Monitor {
    pub out: PathBuf,
    pub database: PathBuf,
    pub epoch: i64,
    pub hub: Arc<room_service::Hub>,
    pub relay: Option<Arc<authenticated_peer::Hub>>,
    pub worker: crate::account_worker::Client,
    pub start: Instant,
    pub boot: u128,
}

impl Monitor {
    pub(crate) async fn run(
        self,
        mut stop: watch::Receiver<bool>,
        log: crate::Log,
    ) -> io::Result<()> {
        let mut timer = tokio::time::interval(PERIOD);
        timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        let mut db_probe_errors = 0u64;
        loop {
            tokio::select! {
                biased;
                _ = stop.changed() => return Ok(()),
                _ = timer.tick() => {}
            }
            let path = self.database.clone();
            let epoch = self.epoch;
            let database = match tokio::task::spawn_blocking(move || {
                accounts::Store::probe_epoch(&path)
            })
            .await
            {
                Ok(Ok(observed)) if observed == epoch => "ok",
                Ok(Ok(_)) => "epoch_mismatch",
                Ok(Err(accounts::Error::Busy)) => "busy",
                Ok(Err(accounts::Error::Schema)) => "schema",
                _ => "unavailable",
            };
            if database != "ok" {
                db_probe_errors = db_probe_errors.saturating_add(1);
            }
            let lobby = self.hub.health();
            let relay = self.relay.as_ref().map(|hub| hub.health()).transpose();
            let worker_ok = !self.worker.is_closed();
            let healthy = database == "ok" && worker_ok && lobby.is_ok() && relay.is_ok();
            let snapshot = json!({
                "schema":SCHEMA, "boot":self.boot.to_string(), "epoch":epoch,
                "at_utc":accounts::utc().map_err(io::Error::other)?, "healthy":healthy,
                "database_probe":database, "database_probe_errors":db_probe_errors, "account_worker_ok":worker_ok,
                "lobby":lobby.ok(), "relay":relay.ok().flatten()
            });
            let mut event = snapshot.clone();
            event["event"] = json!("health_sample");
            crate::emit(&log, crate::stamp(self.start, event)).await?;
            let tmp = self.out.join("health.json.tmp");
            tokio::fs::write(&tmp, serde_json::to_vec(&snapshot)?).await?;
            tokio::fs::rename(tmp, self.out.join("health.json")).await?;
        }
    }
}

/// Local probe for supervisors: both process readiness and a fresh, healthy
/// sample are required. An old ready file cannot mask a dead service forever.
pub fn check(run: &Path) -> io::Result<Value> {
    let read = |name| -> io::Result<Value> {
        Ok(serde_json::from_slice(&std::fs::read(run.join(name))?)?)
    };
    let state = read("service-state.json")?;
    if state["ready"] != true || state["stopped"] != false {
        return Err(io::Error::other("service not ready"));
    }
    let health = read("health.json")?;
    let now = accounts::utc().map_err(io::Error::other)?;
    let fresh = health["at_utc"]
        .as_i64()
        .is_some_and(|t| (now.saturating_sub(MAX_AGE)..=now).contains(&t));
    if health["schema"] != SCHEMA || !fresh || health["healthy"] != true {
        return Err(io::Error::other("service sample stale or unhealthy"));
    }
    // Read readiness again to catch a drain that began while reading the sample.
    if read("service-state.json")?["ready"] != true {
        return Err(io::Error::other("service not ready"));
    }
    Ok(health)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn rejects_stale_future_unhealthy_and_stopped_samples() {
        let mut suffix = [0; 8];
        getrandom::fill(&mut suffix).unwrap();
        let run =
            std::env::temp_dir().join(format!("kr-health-{:016x}", u64::from_ne_bytes(suffix)));
        fs::create_dir(&run).unwrap();
        let state = run.join("service-state.json");
        let sample = run.join("health.json");
        let write = |path, value: Value| fs::write(path, value.to_string()).unwrap();
        write(&state, json!({"ready":true,"stopped":false}));
        assert!(check(&run).is_err());
        let now = accounts::utc().unwrap();
        for (at_utc, healthy, expected) in [
            (now - 60, true, false),
            (now + 60, true, false),
            (now, false, false),
            (now, true, true),
        ] {
            write(
                &sample,
                json!({"schema":SCHEMA,"at_utc":at_utc,"healthy":healthy}),
            );
            assert_eq!(check(&run).is_ok(), expected);
        }
        write(&state, json!({"ready":false,"stopped":true}));
        assert!(check(&run).is_err());
        fs::remove_dir_all(run).unwrap();
    }
}
