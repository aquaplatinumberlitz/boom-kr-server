//! Desktop actions run outside the UI thread. Paths are passed as arguments.
use crate::game::Config;
use std::{io, path::Path, process::Command};

#[derive(Clone, Copy)]
pub(crate) enum Action {
    Game,
    Logs,
    #[cfg(windows)]
    Shortcut,
}

fn open_directory(path: &Path) -> io::Result<()> {
    if !path.is_dir() {
        return Err(io::Error::other("directory unavailable"));
    }
    #[cfg(windows)]
    let editor = "explorer.exe";
    #[cfg(not(windows))]
    let editor = "xdg-open";
    let status = Command::new(editor).arg(path).status()?;
    #[cfg(not(windows))]
    if !status.success() {
        return Err(io::Error::other("directory opener failed"));
    }
    #[cfg(windows)]
    let _ = status;
    Ok(())
}

pub(crate) fn run(config: Config, action: Action) -> io::Result<()> {
    match action {
        Action::Game => open_directory(&config.installation),
        Action::Logs => {
            let path = config.state.join("prelogin-logs");
            std::fs::create_dir_all(&path)?;
            open_directory(&path)
        }
        #[cfg(windows)]
        Action::Shortcut => {
            use std::os::windows::process::CommandExt;
            // Use fixed code. Do not execute a mutable script from the package.
            let launcher = std::env::current_exe()?;
            let launcher_dir = launcher
                .parent()
                .ok_or_else(|| io::Error::other("launcher path"))?;
            let status = Command::new("powershell.exe")
                .args([
                    "-NoLogo",
                    "-NoProfile",
                    "-NonInteractive",
                    "-Command",
                    r#"
                    $ErrorActionPreference = 'Stop'
                    $shell = New-Object -ComObject WScript.Shell
                    $link = $shell.CreateShortcut((Join-Path $env:KR_SHORTCUT_DIR 'Boom Play.lnk'))
                    $link.TargetPath = $env:KR_SHORTCUT_TARGET
                    $link.WorkingDirectory = $env:KR_SHORTCUT_HOME
                    $link.Save()
                "#,
                ])
                .env("KR_SHORTCUT_DIR", &config.bundle)
                .env("KR_SHORTCUT_TARGET", &launcher)
                .env("KR_SHORTCUT_HOME", launcher_dir)
                .creation_flags(0x0800_0000)
                .status()?;
            status
                .success()
                .then_some(())
                .ok_or_else(|| io::Error::other("shortcut creation failed"))
        }
    }
}
