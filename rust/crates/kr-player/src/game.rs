//! Own the game and its loopback TLS companion for one launcher session.
use crate::Language;
use kr_tunnel::prelogin::{ACCOUNT_SESSION_ENDED_EXIT, heartbeat, trace::LOG_NAME};
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
    pub server: Option<SocketAddrV4>,
    pub certificate: Option<PathBuf>,
    #[serde(default)]
    pub server_name: Option<String>,
    #[serde(default)]
    pub release_url: Option<String>,
    prelogin_server: Option<SocketAddrV4>,
    prelogin_certificate: Option<PathBuf>,
    companion: PathBuf,
    pub ports: Vec<u16>,
    pub bundle: PathBuf,
    pub manifest_sha256: String,
    pub installation: PathBuf,
    pub state: PathBuf,
    #[serde(default)]
    wine: Option<PathBuf>,
    #[serde(default)]
    pub udp_backend: kr_tunnel::UdpBackend,
    pub friends: Option<crate::friends::Config>,
    #[serde(default)]
    pub friends_directory: Vec<String>,
}
impl Config {
    pub fn route(&self) -> Option<crate::servers::Route> {
        Some(crate::servers::Route {
            account: self.server?,
            account_cert: self.certificate.clone()?,
            game: self.prelogin_server?,
            game_cert: self.prelogin_certificate.clone()?,
        })
    }

    pub fn with_route(mut self, route: &crate::servers::Route) -> Self {
        self.server = Some(route.account);
        self.certificate = Some(route.account_cert.clone());
        self.prelogin_server = Some(route.game);
        self.prelogin_certificate = Some(route.game_cert.clone());
        self
    }

    pub fn release_url(&self) -> Option<&str> {
        self.release_url
            .as_deref()
            .and_then(crate::desktop::drive_url)
    }

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
        for path in [&mut cfg.companion, &mut cfg.bundle] {
            *path = path.canonicalize()?;
        }
        for path in [&mut cfg.certificate, &mut cfg.prelogin_certificate]
            .into_iter()
            .flatten()
        {
            if !path.is_absolute() {
                *path = root.join(&*path);
            }
            *path = path.canonicalize()?;
        }
        let configured = [
            cfg.server.is_some(),
            cfg.prelogin_server.is_some(),
            cfg.certificate.is_some(),
            cfg.prelogin_certificate.is_some(),
        ];
        if configured.iter().any(|&present| present) && !configured.iter().all(|&present| present) {
            return Err(io::Error::other("incomplete online server route"));
        }
        if let Some(friends) = &mut cfg.friends {
            friends.resolve(root)?;
        }
        for address in [cfg.server, cfg.prelogin_server].into_iter().flatten() {
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
    _installation_lock: fs::File,
    companion: Child,
    game: Option<Game>,
    run: PathBuf,
    stopping: bool,
    exit: Option<(&'static str, Option<i32>)>,
    generation: u64,
    game_ended_at: Option<Instant>,
}

enum Game {
    Process(Child),
    #[cfg(windows)]
    Shell(crate::windows_game::Game),
}

impl Game {
    fn spawn(command: &mut Command, executable: &Path, directory: &Path) -> io::Result<Self> {
        let result = command.spawn();
        #[cfg(windows)]
        if result
            .as_ref()
            .is_err_and(|error| error.raw_os_error() == Some(740))
        {
            return crate::windows_game::Game::open(executable, directory).map(Self::Shell);
        }
        #[cfg(not(windows))]
        let _ = (executable, directory);
        result.map(Self::Process)
    }

    fn try_wait(&mut self) -> io::Result<Option<std::process::ExitStatus>> {
        match self {
            Self::Process(game) => game.try_wait(),
            #[cfg(windows)]
            Self::Shell(game) => game.try_wait(),
        }
    }

    fn kill(&mut self) -> io::Result<()> {
        match self {
            Self::Process(game) => game.kill(),
            #[cfg(windows)]
            Self::Shell(game) => game.kill(),
        }
    }

    fn wait(&mut self) -> io::Result<std::process::ExitStatus> {
        match self {
            Self::Process(game) => game.wait(),
            #[cfg(windows)]
            Self::Shell(game) => game.wait(),
        }
    }
}

#[derive(PartialEq, Eq)]
pub(crate) enum End {
    Stopped,
    Failed,
    SessionEnded,
    RoomClosed,
}
impl OwnedSession {
    pub fn request_stop(&mut self) {
        self.stopping = true;
        if let Some(game) = self.game.as_mut() {
            let _ = game.kill();
        }
        let _ = fs::write(self.run.join("cancel"), []);
    }
    pub fn poll(&mut self) -> Option<End> {
        self.generation = self.generation.saturating_add(1);
        if heartbeat::publish(&self.run.join("beat"), self.generation).is_err() {
            return Some(End::Failed);
        }
        let game = self.game.as_mut()?.try_wait();
        let companion = self.companion.try_wait();
        if matches!(&companion, Ok(Some(status)) if status.code() == Some(ACCOUNT_SESSION_ENDED_EXIT))
        {
            self.exit = Some(("companion", Some(ACCOUNT_SESSION_ENDED_EXIT)));
            return Some(if self.stopping {
                End::Stopped
            } else {
                End::SessionEnded
            });
        }
        if !self.stopping
            && matches!(&companion, Ok(Some(status)) if status.code() == Some(kr_tunnel::prelogin::friends::ROOM_CLOSED_EXIT))
        {
            return Some(End::RoomClosed);
        }
        match game {
            Ok(Some(status)) => {
                // A native EOF can end the game before the companion exits.
                if !self.stopping
                    && matches!(companion, Ok(None))
                    && self
                        .game_ended_at
                        .get_or_insert_with(Instant::now)
                        .elapsed()
                        < Duration::from_secs(1)
                {
                    return None;
                }
                self.exit = Some(("game", status.code()));
                Some(if status.success() || self.stopping {
                    End::Stopped
                } else {
                    End::Failed
                })
            }
            Err(_) => {
                self.exit = Some(("game_wait", None));
                Some(End::Failed)
            }
            Ok(None) => {
                self.exit = match companion {
                    Ok(None) => return None,
                    Ok(Some(status)) => Some(("companion", status.code())),
                    Err(_) => Some(("companion_wait", None)),
                };
                Some(if self.stopping {
                    End::Stopped
                } else {
                    End::Failed
                })
            }
        }
    }
}
impl Drop for OwnedSession {
    fn drop(&mut self) {
        self.request_stop();
        if let Some(game) = self.game.as_mut()
            && game.kill().is_ok()
        {
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

fn ready(child: &mut Child, signal: &Path) -> Result<(), StartError> {
    let deadline = Instant::now() + Duration::from_secs(18);
    loop {
        if let Some(exit) = child.try_wait().map_err(|_| StartError::Launch)? {
            return Err(
                if exit.code() == Some(kr_tunnel::prelogin::friends::DIFFERENT_PACKAGE_EXIT) {
                    StartError::Version
                } else {
                    StartError::Launch
                },
            );
        }
        if Instant::now() >= deadline {
            return Err(StartError::Launch);
        }
        if signal.is_file() {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(25));
    }
}

fn start_inner(
    config: Config,
    language: Language,
    installation_lock: fs::File,
    friends: Option<kr_tunnel::prelogin::friends::Guest>,
) -> Result<Session, StartError> {
    let run = config.state.join(format!(
        "run-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| StartError::Launch)?
            .as_nanos()
    ));
    fs::create_dir(&run).map_err(|_| StartError::Launch)?;
    let signal = run.join("ready");
    let path = run.join("prelogin.json");
    let mut owned = (|| {
        let beat = run.join("beat");
        heartbeat::publish(&beat, 0)?;
        let value = serde_json::json!({
            "server": config.prelogin_server.filter(|_| friends.is_none()),
            "certificate": config.prelogin_certificate.filter(|_| friends.is_none()),
            "friends": friends,
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
            _installation_lock: installation_lock,
            companion: command.spawn()?,
            game: None,
            run: run.clone(),
            stopping: false,
            exit: None,
            generation: 0,
            game_ended_at: None,
        })
    })()
    .inspect_err(|_| {
        let _ = fs::remove_dir_all(&run);
    })
    .map_err(|_| StartError::Launch)?;
    ready(&mut owned.companion, &signal).inspect_err(|_| {
        owned.exit = Some((
            "companion_start",
            owned
                .companion
                .try_wait()
                .ok()
                .flatten()
                .and_then(|s| s.code()),
        ));
    })?;
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
        Command::new(&executable)
    };
    game.arg("multiserveridx:0")
        .current_dir(&directory)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let (complete, done) = std::sync::mpsc::channel::<()>();
    let beat = owned.run.join("beat");
    let pulse = thread::spawn(move || {
        let mut n = 0u64;
        while done.recv_timeout(Duration::from_millis(250))
            == Err(std::sync::mpsc::RecvTimeoutError::Timeout)
        {
            n += 1;
            let _ = heartbeat::publish(&beat, n);
        }
        n
    });
    let child = Game::spawn(&mut game, &executable, &directory);
    drop(complete);
    owned.generation = pulse.join().unwrap_or_default();
    if let Err(error) = &child {
        owned.exit = Some(("game_start", error.raw_os_error()));
    }
    owned.game = Some(child.map_err(|_| StartError::Launch)?);
    Ok(Session(Arc::new(Mutex::new(owned))))
}

#[derive(Clone, Copy)]
pub(crate) enum StartError {
    Integrity,
    WritableFiles,
    Launch,
    Version,
}

pub(crate) fn start(config: Config, language: Language) -> Result<Session, StartError> {
    start_mode(config, language, None)
}

pub(crate) fn start_mode(
    config: Config,
    language: Language,
    friends: Option<kr_tunnel::prelogin::friends::Guest>,
) -> Result<Session, StartError> {
    let lock = kr_player::lock_installation(&config.installation)
        .map_err(|_| StartError::WritableFiles)?;
    kr_player::prepare_language(
        &config.bundle,
        &config.manifest_sha256,
        &config.installation,
        language.id(),
    )
    .map_err(|error| match error {
        kr_player::PreparationError::Integrity => StartError::Integrity,
        kr_player::PreparationError::WritableFiles => StartError::WritableFiles,
    })?;
    start_inner(config, language, lock, friends)
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
