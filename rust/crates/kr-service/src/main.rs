#![forbid(unsafe_code)]
use std::{io, path::PathBuf};
#[tokio::main]
async fn main() -> io::Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 4 || args[0] != "--config" || args[2] != "--out" {
        return Err(io::Error::other(
            "usage: kr-service --config CONFIG --out NEW_RUN_DIRECTORY",
        ));
    }
    let profile = kr_service::load_config(&PathBuf::from(&args[1]))?;
    kr_service::serve(profile, &PathBuf::from(&args[3])).await
}
