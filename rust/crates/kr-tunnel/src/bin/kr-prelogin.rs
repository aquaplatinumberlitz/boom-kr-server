//! Company companion starts before the game and accepts no account password.
#![forbid(unsafe_code)]
use kr_tunnel::prelogin::trace::{Startup, StopCause, Trace};
use serde::Deserialize;
use std::{io, net::SocketAddrV4, path::PathBuf};
use tokio::sync::watch;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    server: SocketAddrV4,
    certificate: PathBuf,
    ports: Vec<u16>,
    cancel_file: PathBuf,
    #[serde(default)]
    ready_file: Option<PathBuf>,
    #[serde(default)]
    owner_heartbeat: Option<PathBuf>,
    #[serde(default)]
    udp_backend: kr_tunnel::UdpBackend,
    #[serde(default)]
    diagnostics: bool,
}
#[tokio::main]
async fn main() -> io::Result<()> {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 2 {
        return Err(io::Error::other("usage: kr-prelogin CONFIG"));
    }
    let raw = std::fs::read(&args[1])?;
    if raw.len() > 16384 {
        return Err(io::Error::other("prelogin config bound"));
    }
    let config: Config =
        serde_json::from_slice(&raw).map_err(|_| io::Error::other("prelogin config"))?;
    if config.server.port() == 0
        || config.server.ip().is_unspecified()
        || config.server.ip().is_multicast()
        || config.server.ip().is_broadcast()
        || config.cancel_file.exists()
    {
        return Err(io::Error::other("prelogin configuration bounds"));
    }
    let trace = if config.diagnostics {
        let path = std::path::Path::new(&args[1]);
        Trace::create(
            path.parent()
                .ok_or_else(|| io::Error::other("prelogin run directory"))?,
        )?
    } else {
        Trace::default()
    };
    let endpoint = std::fs::read(config.certificate)
        .and_then(|cert| kr_tunnel::prelogin::endpoint(cert, config.udp_backend));
    trace.startup(Startup::Endpoint, endpoint.is_ok());
    let endpoint = endpoint?;
    let listeners = kr_tunnel::prelogin::bind_traced(&config.ports, &trace).await?;
    if let Some(path) = config.ready_file {
        std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(path)?;
    }
    trace.startup(Startup::Ready, true);
    println!("{{\"event\":\"prelogin_ready\"}}");
    let (halt, stop) = watch::channel(false);
    let cancel = config.cancel_file;
    let heartbeat = config.owner_heartbeat;
    let lifecycle = trace.clone();
    let task = tokio::spawn(async move {
        let mut ticker = tokio::time::interval(std::time::Duration::from_millis(100));
        let signal = tokio::signal::ctrl_c();
        tokio::pin!(signal);
        let cause = loop {
            tokio::select! {
                _ = &mut signal => break StopCause::Interrupt,
                _ = ticker.tick() => {
                    if cancel.exists() { break StopCause::Cancel; }
                    if heartbeat.as_ref().is_some_and(|p|
                        std::fs::metadata(p).and_then(|m| m.modified()).ok()
                            .and_then(|t| t.elapsed().ok())
                            .is_none_or(|age| age >= std::time::Duration::from_secs(10))) {
                        break StopCause::Heartbeat;
                    }
                }
            }
        };
        lifecycle.stop(cause);
        halt.send_replace(true);
    });
    let result = kr_tunnel::prelogin::companion_traced(
        endpoint.clone(),
        config.server.into(),
        listeners,
        stop,
        config.udp_backend,
        trace,
    )
    .await;
    task.abort();
    let _ = task.await;
    endpoint.close(0u8.into(), b"companion stopped");
    tokio::time::timeout(std::time::Duration::from_secs(2), endpoint.wait_idle())
        .await
        .map_err(|_| io::Error::other("prelogin companion stop deadline"))?;
    result
}
