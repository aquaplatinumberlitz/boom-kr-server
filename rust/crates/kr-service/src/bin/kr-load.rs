use std::{io, path::Path};

#[tokio::main]
async fn main() {
    let mut args = std::env::args_os().skip(1);
    let Some(config) = args.next().filter(|_| args.next().is_none()) else {
        eprintln!("usage: kr-load PRIVATE_CONFIG.json");
        std::process::exit(2);
    };
    let result = tokio::select! {
        result = kr_service::load_driver::execute(Path::new(&config)) => result,
        _ = tokio::signal::ctrl_c() => Err(io::Error::other("load interrupted")),
    };
    if let Err(e) = result {
        eprintln!("kr-load: {e}");
        std::process::exit(1);
    }
}
