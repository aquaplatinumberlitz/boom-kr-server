//! Desktop actions run outside the UI thread. Paths are passed as arguments.
use crate::game::Config;
use std::{io, path::Path, process::Command};

#[derive(Clone, Copy)]
pub(crate) enum Action {
    Game,
    Logs,
    Release,
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

/// The bundle supplies an HTTPS Google Drive link, never a command or protocol handler.
pub(crate) fn drive_url(raw: &str) -> Option<&str> {
    let url = raw.trim();
    let path = url.strip_prefix("https://drive.google.com/")?;
    (!path.is_empty()
        && url.len() <= 2048
        && path
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-._~/%?&=+#:@!$',;()*".contains(&b)))
    .then_some(url)
}

fn open_release(url: &str) -> io::Result<()> {
    #[cfg(windows)]
    let status = {
        use std::os::windows::process::CommandExt;
        Command::new("powershell.exe")
            .args([
                "-NoLogo",
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "$ErrorActionPreference = 'Stop'; Start-Process -FilePath $env:KR_RELEASE_URL",
            ])
            .env("KR_RELEASE_URL", url)
            .creation_flags(0x0800_0000)
            .status()?
    };
    #[cfg(not(windows))]
    let status = Command::new("xdg-open").arg(url).status()?;
    status
        .success()
        .then_some(())
        .ok_or_else(|| io::Error::other("browser could not open the link"))
}

pub(crate) fn run(config: Config, action: Action) -> io::Result<()> {
    match action {
        Action::Release => open_release(
            config
                .release_url()
                .ok_or_else(|| io::Error::other("release link unavailable"))?,
        ),
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

#[cfg(test)]
mod tests {
    use super::drive_url;
    #[test]
    fn release_link_must_be_a_google_drive_https_url() {
        let url = "https://drive.google.com/drive/folders/example?usp=sharing";
        assert_eq!(drive_url(url), Some(url));
        assert_eq!(drive_url(" "), None);
        assert_eq!(drive_url("http://drive.google.com/file"), None);
        assert_eq!(
            drive_url("https://drive.google.com.evil.invalid/file"),
            None
        );
        assert_eq!(drive_url("https://drive.google.com/file\nother"), None);
    }
}
