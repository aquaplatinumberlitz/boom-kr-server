//! Own the game and its loopback TLS companion for one launcher session.
use crate::Language;
use serde::Deserialize;
use std::{
    fs, io,
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
            Ok(Some(status)) => Some(status.success() || self.stopping),
            Err(_) => Some(false),
            Ok(None) => match self.companion.try_wait() {
                Ok(None) => None,
                _ => Some(self.stopping),
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
        let _ = fs::remove_dir_all(&self.run);
    }
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
