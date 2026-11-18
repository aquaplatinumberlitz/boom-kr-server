//! Operator control using the same installer as the graphical launcher.
use std::{env, path::PathBuf, process::ExitCode};
fn main() -> ExitCode {
    let args: Vec<_> = env::args_os().collect();
    let result = (|| {
        if args.len() != 5 {
            return Err(std::io::Error::other("arguments"));
        }
        let bundle = PathBuf::from(&args[2]);
        let expected = args[3]
            .to_str()
            .ok_or_else(|| std::io::Error::other("hash"))?;
        let target = PathBuf::from(&args[4]);
        if args[1] == "install" {
            kr_player::install(&bundle, expected, &target)
        } else if args[1] == "verify" {
            kr_player::verify_install(&bundle, expected, &target)
        } else {
            Err(std::io::Error::other("mode"))
        }
    })();
    if result.is_ok() {
        println!("{{\"ok\":true}}");
        ExitCode::SUCCESS
    } else {
        eprintln!("Player installation check failed; no launch authorized.");
        ExitCode::FAILURE
    }
}
