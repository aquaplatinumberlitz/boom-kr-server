//! Operator provisioning for the first-party launcher. Outputs are exclusive
//! creates; private key material is never printed or committed.
#![forbid(unsafe_code)]
#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::{
    fs::OpenOptions,
    io::{self, Write},
    path::Path,
};
fn create(path: &Path, bytes: &[u8], private: bool) -> io::Result<()> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(if private { 0o600 } else { 0o644 });
    #[cfg(not(unix))]
    let _ = private;
    let mut file = options.open(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}
fn run() -> io::Result<()> {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 2 {
        return Err(io::Error::other(
            "usage: kr-launch-identity <new-private-directory>",
        ));
    }
    let root = Path::new(&args[1]);
    std::fs::create_dir(root)?;
    #[cfg(unix)]
    std::fs::set_permissions(root, std::fs::Permissions::from_mode(0o700))?;
    let (certificate, key) = kr_tunnel::launch_tls::generate_identity()?;
    create(&root.join("certificate.der"), &certificate, false)?;
    create(&root.join("private.pk8"), &key, true)?;
    println!(
        "Launcher identity created. Distribute certificate.der through the trusted player package; keep private.pk8 server-only."
    );
    Ok(())
}
fn main() {
    if run().is_err() {
        eprintln!("Cannot create launcher identity; destination must be new and private.");
        std::process::exit(1);
    }
}
