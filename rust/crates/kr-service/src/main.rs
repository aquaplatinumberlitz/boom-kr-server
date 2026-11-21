#![forbid(unsafe_code)]
use std::{io, path::PathBuf};
#[tokio::main]
async fn main() -> io::Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if !matches!(args.len(), 4 | 6)
        || args[0] != "--config"
        || args[2] != "--out"
        || (args.len() == 6 && args[4] != "--owner-heartbeat")
    {
        return Err(io::Error::other(
            "usage: kr-service --config CONFIG --out NEW_RUN_DIRECTORY [--owner-heartbeat FILE]",
        ));
    }
    let profile = kr_service::load_config(&PathBuf::from(&args[1]))?;
    let heartbeat = args.get(5).map(PathBuf::from);
    kr_service::serve_owned(profile, &PathBuf::from(&args[3]), heartbeat.as_deref()).await
}
