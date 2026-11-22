//! Company companion starts before the game and accepts no account password.
#![forbid(unsafe_code)]
use kr_tunnel::prelogin::trace::{Startup, StopCause, Trace};
use kr_tunnel::prelogin::{friends, heartbeat::Heartbeat};
use serde::Deserialize;
use std::{io, net::SocketAddrV4, path::PathBuf};
use tokio::sync::watch;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    server: Option<SocketAddrV4>,
    certificate: Option<PathBuf>,
    friends: Option<friends::Guest>,
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
    let online = match (config.server, config.certificate, config.friends.is_some()) {
        (Some(server), Some(certificate), false)
            if server.port() != 0
                && !server.ip().is_unspecified()
                && !server.ip().is_multicast()
                && !server.ip().is_broadcast() =>
        {
            Some((server, certificate))
        }
        (None, None, true) => None,
        _ => return Err(io::Error::other("prelogin configuration bounds")),
    };
    if config.cancel_file.exists() {
        return Err(io::Error::other("prelogin cancelled"));
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
    let mut endpoint = None;
    let mut friends_session = None;
    if let Some((server, certificate)) = online {
        let configured = std::fs::read(certificate)
            .and_then(|cert| kr_tunnel::prelogin::endpoint(cert, config.udp_backend));
        trace.startup(Startup::Endpoint, configured.is_ok());
        endpoint = Some((configured?, server));
    }
    if let Some(guest) = config.friends {
        let peer = kr_p2p::room::endpoint(guest.route.relay.is_some()).await;
        trace.startup(Startup::Endpoint, peer.is_ok());
        let peer = peer?;
        let admission = friends::join(&peer, guest).await;
        trace.startup(Startup::RoomAdmission, admission.is_ok());
        let session = match admission {
            Ok(connection) => connection,
            Err(kr_p2p::room::JoinError::Version) => {
                std::process::exit(friends::DIFFERENT_PACKAGE_EXIT)
            }
            Err(error) => return Err(io::Error::other(error)),
        };
        friends_session = Some((peer, session));
    }
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
        let mut progress = Heartbeat::new(std::time::Instant::now());
        let cause = loop {
            tokio::select! {
                _ = &mut signal => break StopCause::Interrupt,
                _ = ticker.tick() => {
                    if cancel.exists() { break StopCause::Cancel; }
                    if heartbeat.as_ref().is_some_and(|p|
                        progress.expired(p, std::time::Instant::now())) {
                        break StopCause::Heartbeat;
                    }
                }
            }
        };
        lifecycle.stop(cause);
        halt.send_replace(true);
    });
    use kr_tunnel::prelogin::{ACCOUNT_SESSION_ENDED_EXIT, AccountSessionEnded};
    if let Some((peer, connection)) = friends_session {
        let result = friends::companion(connection, listeners, stop).await;
        task.abort();
        peer.close().await;
        if result.is_err() {
            std::process::exit(friends::ROOM_CLOSED_EXIT);
        }
        return result;
    }
    let (endpoint, server) = endpoint.expect("validated online mode");
    let result = kr_tunnel::prelogin::companion_traced(
        endpoint.clone(),
        server.into(),
        listeners,
        stop,
        config.udp_backend,
        trace,
    )
    .await;
    task.abort();
    let _ = task.await;
    endpoint.close(0u8.into(), b"companion stopped");
    if result.as_ref().is_err_and(|error| {
        error
            .get_ref()
            .is_some_and(|e| e.is::<AccountSessionEnded>())
    }) {
        // The remote peer already closed; forward the reason before the GUI can
        // mistake the game exiting on EOF for a successful user quit.
        std::process::exit(ACCOUNT_SESSION_ENDED_EXIT);
    }
    tokio::time::timeout(std::time::Duration::from_secs(2), endpoint.wait_idle())
        .await
        .map_err(|_| io::Error::other("prelogin companion stop deadline"))?;
    result
}
