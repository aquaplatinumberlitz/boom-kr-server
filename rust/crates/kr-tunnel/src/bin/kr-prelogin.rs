//! Company companion starts before the game and accepts no account password.
#![forbid(unsafe_code)]
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
    let mut endpoint = quinn::Endpoint::client("0.0.0.0:0".parse().expect("constant bind"))?;
    endpoint.set_default_client_config(kr_tunnel::prelogin::client_config(std::fs::read(
        config.certificate,
    )?)?);
    let connection = kr_tunnel::prelogin::connect(&endpoint, config.server.into()).await?;
    let listeners = kr_tunnel::prelogin::bind(&config.ports).await?;
    println!("{{\"event\":\"prelogin_ready\"}}");
    let (halt, stop) = watch::channel(false);
    let cancel = config.cancel_file;
    let task = tokio::spawn(async move {
        loop {
            tokio::select! {
                _=tokio::signal::ctrl_c()=>break,
                _=tokio::time::sleep(std::time::Duration::from_millis(100))=>if cancel.exists(){break;}
            }
        }
        halt.send_replace(true);
    });
    let result = kr_tunnel::prelogin::companion(connection, listeners, stop).await;
    task.abort();
    let _ = task.await;
    endpoint.close(0u8.into(), b"companion stopped");
    tokio::time::timeout(std::time::Duration::from_secs(2), endpoint.wait_idle())
        .await
        .map_err(|_| io::Error::other("prelogin companion stop deadline"))?;
    result
}
