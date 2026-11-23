//! S1 desktop host (Windows) and local operator CLI (Linux).
#![cfg_attr(windows, windows_subsystem = "windows")]
#![deny(unsafe_code)]

#[cfg(windows)]
#[allow(unsafe_code)]
#[path = "boom_server/windows.rs"]
mod windows;

#[cfg(windows)]
fn main() {
    windows::main();
}

#[cfg(not(windows))]
fn main() -> std::io::Result<()> {
    use kr_service::self_host::{self, Action, Settings};
    use std::{io, path::PathBuf};
    use tokio::sync::{mpsc, watch};

    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    let state =
        PathBuf::from(std::env::var_os("BOOM_STATE").unwrap_or("/var/lib/boom-server".into()));
    match args.as_slice() {
        [command] if command == "invite" => {
            println!("{}", self_host::invite(&state)?);
            Ok(())
        }
        [command] if command == "backup" => {
            self_host::state_dir(&state)?;
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()?;
            println!("{}", runtime.block_on(self_host::backup(&state))?.display());
            Ok(())
        }
        [] | [_] if args.is_empty() || args[0] == "serve" => {
            // Prebuilt game-rule JSON ships in the bundle; BOOM_GAME only
            // re-exports from a KR 00967 folder for dev/build.
            let game = self_host::game_from_environment();
            let name = std::env::var("BOOM_NAME").unwrap_or_else(|_| "Boom Server".into());
            let path = std::env::current_exe()?;
            let root = path
                .parent()
                .and_then(|p| p.parent())
                .ok_or_else(|| {
                    io::Error::other("boom-server must be inside the bundle's bin/ folder")
                })?
                .to_owned();
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(4)
                .enable_all()
                .build()?;
            runtime.block_on(async move {
                let (action, receive) = mpsc::channel::<Action>(4);
                let (status, mut updates) = watch::channel(self_host::Status::default());
                let signals = tokio::spawn(self_host::command_line_stop(action));
                let print_status = tokio::spawn(async move {
                    let mut was_ready = false;
                    let mut relay = false;
                    let mut phase = self_host::Phase::Idle;
                    while updates.changed().await.is_ok() {
                        let current = updates.borrow_and_update();
                        if current.phase != phase {
                            phase = current.phase;
                            if let Some(step) = phase.step() {
                                eprintln!(
                                    "Boom Server step {step}/{}: {}",
                                    self_host::Phase::STEPS,
                                    phase.label()
                                );
                            }
                        }
                        if current.ready != was_ready {
                            eprintln!("Boom Server ready: {}", current.ready);
                            was_ready = current.ready;
                        }
                        if current.relay_online != relay {
                            eprintln!("Boom Server relay connected: {}", current.relay_online);
                            relay = current.relay_online;
                        }
                    }
                });
                let result = self_host::listen(
                    root,
                    state,
                    Settings { name, game },
                    self_host::bind_from_environment()?,
                    self_host::network_from_environment()?,
                    receive,
                    status,
                )
                .await;
                signals.abort();
                print_status.abort();
                result
            })
        }
        _ => Err(io::Error::other("usage: boom-server [serve|invite|backup]")),
    }
}
