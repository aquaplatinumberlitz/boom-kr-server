//! Own the game and its loopback TLS companion for one launcher session.
use crate::Language;
use kr_tunnel::prelogin::trace::LOG_NAME;
use serde::Deserialize;
use std::{
    fs,
    io::{self, Write},
    net::SocketAddrV4,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Config {
    pub server: SocketAddrV4,
    pub certificate: PathBuf,
    prelogin_server: SocketAddrV4,
    prelogin_certificate: PathBuf,
    companion: PathBuf,
    ports: Vec<u16>,
    pub bundle: PathBuf,
    pub manifest_sha256: String,
    pub installation: PathBuf,
    pub state: PathBuf,
    #[serde(default)]
    wine: Option<PathBuf>,
    #[serde(default)]
    udp_backend: kr_tunnel::UdpBackend,
}
impl Config {
    pub fn load() -> io::Result<Self> {
        let path = std::env::args_os()
            .nth(1)
            .map(PathBuf::from)
            .unwrap_or(std::env::current_exe()?.with_file_name("launcher.json"));
        let path = path.canonicalize()?;
        let root = path
            .parent()
            .ok_or_else(|| io::Error::other("launcher config root"))?;
        let mut cfg: Self = serde_json::from_slice(&fs::read(&path)?).map_err(io::Error::other)?;
        for path in [
            &mut cfg.certificate,
            &mut cfg.prelogin_certificate,
            &mut cfg.companion,
            &mut cfg.bundle,
            &mut cfg.installation,
            &mut cfg.state,
        ] {
            if !path.is_absolute() {
                *path = root.join(&*path);
            }
        }
        if let Some(wine) = &mut cfg.wine {
            if !wine.is_absolute() {
                *wine = root.join(&*wine);
            }
            *wine = wine.canonicalize()?;
        }
        for path in [
            &mut cfg.certificate,
            &mut cfg.prelogin_certificate,
            &mut cfg.companion,
            &mut cfg.bundle,
        ] {
            *path = path.canonicalize()?;
        }
        for address in [cfg.server, cfg.prelogin_server] {
            if address.port() == 0
                || address.ip().is_unspecified()
                || address.ip().is_multicast()
                || address.ip().is_broadcast()
            {
                return Err(io::Error::other("launcher endpoint bounds"));
            }
        }
        if !(1..=32).contains(&cfg.ports.len()) || cfg.ports.contains(&0) {
            return Err(io::Error::other("launcher local port bounds"));
        }
        fs::create_dir_all(&cfg.state)?;
        cfg.state = cfg.state.canonicalize()?;
        let parent = cfg
            .installation
            .parent()
            .ok_or_else(|| io::Error::other("installation parent"))?
            .canonicalize()?;
        cfg.installation = parent.join(
            cfg.installation
                .file_name()
                .ok_or_else(|| io::Error::other("installation name"))?,
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&cfg.state, fs::Permissions::from_mode(0o700))?;
        }
        Ok(cfg)
    }
}

#[derive(Clone)]
pub(crate) struct Session(pub Arc<Mutex<OwnedSession>>);
pub(crate) struct OwnedSession {
    companion: Child,
    game: Option<Child>,
    run: PathBuf,
    stopping: bool,
    exit: Option<(&'static str, Option<i32>)>,
}
impl OwnedSession {
    pub fn request_stop(&mut self) {
        self.stopping = true;
        if let Some(game) = self.game.as_mut() {
            let _ = game.kill();
        }
        let _ = fs::write(self.run.join("cancel"), []);
    }
    pub fn poll(&mut self) -> Option<bool> {
        if fs::write(self.run.join("beat"), []).is_err() {
            return Some(false);
        }
        match self.game.as_mut()?.try_wait() {
            Ok(Some(status)) => {
                self.exit = Some(("game", status.code()));
                Some(status.success() || self.stopping)
            }
            Err(_) => {
                self.exit = Some(("game_wait", None));
                Some(false)
            }
            Ok(None) => match self.companion.try_wait() {
                Ok(None) => None,
                Ok(Some(status)) => {
                    self.exit = Some(("companion", status.code()));
                    Some(self.stopping)
                }
                Err(_) => {
                    self.exit = Some(("companion_wait", None));
                    Some(self.stopping)
                }
            },
        }
    }
}
impl Drop for OwnedSession {
    fn drop(&mut self) {
        self.request_stop();
        if let Some(game) = self.game.as_mut() {
            let _ = game.wait();
        }
        let deadline = Instant::now() + Duration::from_secs(3);
        while matches!(self.companion.try_wait(), Ok(None)) && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(25));
        }
        let _ = self.companion.kill();
        let _ = self.companion.wait();
        let log = self.run.join(LOG_NAME);
        if let Ok(mut file) = fs::OpenOptions::new().append(true).open(&log) {
            let (first, exit_code) = self.exit.unwrap_or(("launcher_stop", None));
            let _ = writeln!(
                file,
                "event=launcher_end first={first} exit_code={exit_code:?}"
            );
            drop(file);
            let _ = retain_log(&log, self.run.parent().expect("run parent"));
        }
        let _ = fs::remove_dir_all(&self.run);
    }
}

// Fixed slots also bound storage if more than one launcher instance completes.
const RETAINED_LOGS: usize = 4;

fn retain_log(source: &Path, state: &Path) -> io::Result<()> {
    let directory = state.join("prelogin-logs");
    match fs::create_dir(&directory) {
        Ok(()) => (),
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => (),
        Err(error) => return Err(error),
    }
    if !fs::symlink_metadata(&directory)?.is_dir() {
        return Err(io::Error::other("invalid prelogin log directory"));
    }
    let mut oldest = None;
    for n in 0..RETAINED_LOGS {
        let slot = directory.join(format!("prelogin-{n}.log"));
        let modified = match fs::symlink_metadata(&slot) {
            Ok(meta) if meta.is_file() => meta.modified()?,
            Ok(_) => continue,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return fs::rename(source, slot);
            }
            Err(error) => return Err(error),
        };
        if oldest.as_ref().is_none_or(|(_, time)| modified < *time) {
            oldest = Some((slot, modified));
        }
    }
    let (slot, _) = oldest.ok_or_else(|| io::Error::other("prelogin log slots unavailable"))?;
    fs::rename(source, slot)
}

fn ready(child: &mut Child, signal: &Path) -> io::Result<()> {
    let deadline = Instant::now() + Duration::from_secs(4);
    loop {
        if child.try_wait()?.is_some() || Instant::now() >= deadline {
            return Err(io::Error::other("prelogin companion not ready"));
        }
        if signal.is_file() {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(25));
    }
}

fn start_inner(config: Config, language: Language) -> io::Result<Session> {
    kr_player::verify_language(
        &config.bundle,
        &config.manifest_sha256,
        &config.installation,
        language.id(),
    )?;
    let run = config.state.join(format!(
        "run-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(io::Error::other)?
            .as_nanos()
    ));
    fs::create_dir(&run)?;
    let signal = run.join("ready");
    let path = run.join("prelogin.json");
    let mut owned = (|| {
        let beat = run.join("beat");
        fs::write(&beat, [])?;
        let value = serde_json::json!({
            "server": config.prelogin_server, "certificate": config.prelogin_certificate,
            "ports": config.ports, "udp_backend": config.udp_backend,
            "cancel_file": run.join("cancel"), "ready_file": signal, "owner_heartbeat": beat,
            "diagnostics": true,
        });
        fs::write(&path, serde_json::to_vec(&value).map_err(io::Error::other)?)?;
        let mut command = Command::new(config.companion);
        command
            .arg(&path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x0800_0000);
        }
        Ok::<_, io::Error>(OwnedSession {
            companion: command.spawn()?,
            game: None,
            run: run.clone(),
            stopping: false,
            exit: None,
        })
    })()
    .inspect_err(|_| {
        let _ = fs::remove_dir_all(&run);
    })?;
    ready(&mut owned.companion, &signal)?;
    let directory = config.installation.join(language.id());
    let executable = directory.join("CA.exe");
    let mut game = if let Some(wine) = config.wine {
        let mut game = Command::new(wine);
        game.arg(&executable).env(
            "WINEPREFIX",
            config.state.join(format!("wine-{}", language.id())),
        );
        game
    } else {
        Command::new(executable)
    };
    owned.game = Some(
        game.arg("multiserveridx:0")
            .current_dir(directory)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?,
    );
    Ok(Session(Arc::new(Mutex::new(owned))))
}

pub(crate) fn start(config: Config, language: Language) -> Result<Session, ()> {
    start_inner(config, language).map_err(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn completed_log_rotation_is_bounded_and_preserves_latest_log() {
        let root = std::env::temp_dir().join(format!(
            "kr-player-logs-{}-{:x}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        let log = root.join(LOG_NAME);
        for n in 0..RETAINED_LOGS + 3 {
            fs::write(&log, n.to_string()).unwrap();
            retain_log(&log, &root).unwrap();
            assert!(!log.exists());
        }
        let archive = fs::read_dir(root.join("prelogin-logs"))
            .unwrap()
            .collect::<io::Result<Vec<_>>>()
            .unwrap();
        assert_eq!(archive.len(), RETAINED_LOGS);
        assert!(
            archive
                .iter()
                .any(|e| fs::read_to_string(e.path()).unwrap() == (RETAINED_LOGS + 2).to_string())
        );
        fs::remove_dir_all(root).unwrap();
    }
}
