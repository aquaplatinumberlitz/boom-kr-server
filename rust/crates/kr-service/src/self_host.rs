//! Installation and lifecycle shared by the S1 desktop and headless host.
use crate::{
    accounts::{self, Admin, Store},
    monitoring, platform,
};
use kr_p2p::{
    Invite, SecretKey,
    self_host::{self as p2p, Network, Services},
};
use serde::{Deserialize, Serialize};
use std::{
    fmt::Write as _,
    fs,
    io::{self, Write},
    net::SocketAddrV4,
    path::{Path, PathBuf},
    process::Stdio,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{
    process::Command,
    sync::{mpsc, oneshot, watch},
    time::{sleep, timeout},
};
use zeroize::Zeroizing;

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub name: String,
    pub game: PathBuf,
}

impl Settings {
    pub fn validate(&self) -> io::Result<()> {
        if !(1..=128).contains(&self.name.len())
            || self.name.chars().any(char::is_control)
            || self.name.trim().is_empty()
            || !self.game.is_absolute()
            || !self.game.is_dir()
        {
            return Err(io::Error::other("invalid server name or game folder"));
        }
        Ok(())
    }
}

#[derive(Clone, Default)]
pub struct Status {
    pub ready: bool,
    pub relay_online: bool,
    pub relay_expected: bool,
    pub invite: String,
    pub players: u64,
    pub rooms: u64,
    pub message: String,
}

pub enum Action {
    Stop,
    Rotate,
    Backup,
}

#[derive(Deserialize)]
struct Prepared {
    runtime: PathBuf,
    out: PathBuf,
}

pub fn state_dir(path: &Path) -> io::Result<()> {
    if !path.is_absolute()
        || path
            .components()
            .any(|p| p == std::path::Component::ParentDir)
    {
        return Err(io::Error::other("state must be an absolute directory"));
    }
    platform::provision_private_dir(path)
}

fn atomic_private(state: &Path, path: &Path, data: &[u8], replace: bool) -> io::Result<()> {
    let temp = state.join(format!(".write-{}-{}", std::process::id(), stamp()?));
    let mut file = platform::private_options()
        .create_new(true)
        .write(true)
        .open(&temp)?;
    let result = (|| {
        file.write_all(data)?;
        file.sync_all()?;
        drop(file);
        if replace {
            // Do not replace a symlink/reparse point or a public file.
            match platform::private_file(path) {
                Ok(_) => {}
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => return Err(error),
            }
            fs::rename(&temp, path)?;
        } else {
            platform::publish_snapshot(&temp, path)?;
        }
        platform::sync_directory(state)
    })();
    if temp.exists() {
        let _ = fs::remove_file(temp);
    }
    result
}

fn stamp() -> io::Result<u128> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(io::Error::other)?
        .as_nanos())
}

fn random() -> io::Result<[u8; 32]> {
    let mut bytes = [0; 32];
    getrandom::fill(&mut bytes).map_err(io::Error::other)?;
    Ok(bytes)
}

fn read_node(path: &Path) -> io::Result<SecretKey> {
    let invalid = || io::Error::other("invalid S1 node key length");
    if platform::private_file(path)?.len() != 32 {
        return Err(invalid());
    }
    let bytes = Zeroizing::new(fs::read(path)?);
    Ok(SecretKey::from_bytes(
        bytes.as_slice().try_into().map_err(|_| invalid())?,
    ))
}

fn node_key(state: &Path) -> io::Result<SecretKey> {
    let path = state.join("s1-node.key");
    match read_node(&path) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            let bytes = Zeroizing::new(random()?);
            atomic_private(state, &path, bytes.as_ref(), false)?;
            Ok(SecretKey::from_bytes(&bytes))
        }
        result => result,
    }
}

fn cert(state: &Path) -> io::Result<Vec<u8>> {
    let path = state.join("prelogin_tls-identity").join("certificate.der");
    if !(1..=8192).contains(&fs::metadata(&path)?.len()) {
        return Err(io::Error::other("invalid S1 certificate length"));
    }
    fs::read(path)
}

pub fn invite(state: &Path) -> io::Result<String> {
    platform::private_dir(state)?;
    // Read-only: a command on a state never initialized must not create an identity.
    Ok(Invite::server(
        read_node(&state.join("s1-node.key"))?.public(),
        &cert(state)?,
    )
    .to_string())
}

pub fn settings(state: &Path) -> io::Result<Option<Settings>> {
    let path = state.join("self-host.json");
    match platform::private_file(&path) {
        Ok(meta) if meta.len() <= 16_384 => Ok(Some(serde_json::from_slice(&fs::read(path)?)?)),
        Ok(_) => Err(io::Error::other("settings too large")),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

pub fn save_settings(state: &Path, settings: &Settings) -> io::Result<()> {
    settings.validate()?;
    atomic_private(
        state,
        &state.join("self-host.json"),
        &serde_json::to_vec(settings)?,
        true,
    )
}

fn first_account(state: &Path) -> io::Result<()> {
    let db = state.join("accounts.sqlite");
    let _instance = crate::operations::lock_database(&db)?;
    let mut store = if db.exists() {
        Store::open_existing(&db)
    } else {
        Store::open(&db)
    }
    .map_err(io::Error::other)?;
    const LOGIN: &str = "HostAdmin";
    match store.lookup(LOGIN) {
        Ok(_) => return Ok(()),
        Err(accounts::Error::InvalidCredentials) => {}
        Err(error) => return Err(io::Error::other(error)),
    }
    let path = state.join("host-account.json");
    let password = match platform::private_file(&path) {
        Ok(meta) if meta.len() <= 1024 => {
            let data = Zeroizing::new(fs::read(path)?);
            let value = serde_json::from_slice::<HostAccount>(&data)?;
            if value.login != LOGIN || value.password.len() != 64 {
                return Err(io::Error::other("invalid local host account record"));
            }
            value.password
        }
        Ok(_) => return Err(io::Error::other("invalid local host account record")),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            let entropy = Zeroizing::new(random()?);
            let mut password = Zeroizing::new(String::with_capacity(64));
            for byte in entropy.iter() {
                write!(password, "{byte:02x}").expect("writing to a string");
            }
            let record = serde_json::to_vec_pretty(&HostAccount {
                login: LOGIN.into(),
                password,
            })?;
            let record = Zeroizing::new(record);
            atomic_private(state, &path, &record, false)?;
            serde_json::from_slice::<HostAccount>(&record)?.password
        }
        Err(error) => return Err(error),
    };
    let admin = Admin {
        actor: "boom-server",
        reason: "Provision initial local host account",
    };
    store
        .create_account(&admin, LOGIN, LOGIN, password.as_bytes(), None)
        .map_err(io::Error::other)?;
    Ok(())
}

#[derive(Deserialize, Serialize)]
struct HostAccount {
    login: String,
    password: Zeroizing<String>,
}

async fn prepare(root: &Path, state: &Path, game: &Path) -> io::Result<Prepared> {
    #[cfg(windows)]
    let python = root.join("python").join("python.exe");
    #[cfg(not(windows))]
    let python = PathBuf::from("python3");
    let expected = fs::read_to_string(root.join("manifest.sha256"))?;
    let mut command = Command::new(python);
    #[cfg(windows)]
    command.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    let output = command
        .current_dir(root)
        .arg(root.join("kr-package.py"))
        .arg("prepare-s1")
        .args(["--manifest-sha256", expected.trim()])
        .arg("--state")
        .arg(state)
        .arg("--game-dir")
        .arg(game)
        .stdin(Stdio::null())
        .output()
        .await?;
    if !output.status.success() {
        return Err(io::Error::other(
            String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        ));
    }
    Ok(serde_json::from_slice(&output.stdout)?)
}

pub async fn backup(state: &Path) -> io::Result<PathBuf> {
    let parent = state.join("manual-backups");
    platform::provision_private_dir(&parent)?;
    let path = parent.join(format!("accounts-{}.sqlite", stamp()?));
    let database = state.join("accounts.sqlite");
    let output = path.clone();
    tokio::task::spawn_blocking(move || crate::accounts::snapshot(&database, &output))
        .await
        .map_err(io::Error::other)?
        .map_err(io::Error::other)?;
    Ok(path)
}

pub async fn listen(
    root: PathBuf,
    state: PathBuf,
    settings: Settings,
    bind: SocketAddrV4,
    network: Network,
    actions: mpsc::Receiver<Action>,
    status: watch::Sender<Status>,
) -> io::Result<()> {
    state_dir(&state)?;
    settings.validate()?;
    let _lock = platform::lock_private_file(&state.join("service.lock"))?;
    let mut prep = prepare(&root, &state, &settings.game).await?;
    let mut key = node_key(&state)?;
    let certificate = cert(&state)?;
    let state_for_account = state.clone();
    tokio::task::spawn_blocking(move || first_account(&state_for_account))
        .await
        .map_err(io::Error::other)??;
    // Existing fixture paths in the manifest are rooted at the bundle.
    std::env::set_current_dir(&root)?;
    let mut host = Host {
        state,
        settings,
        bind,
        network,
        actions,
        status,
    };
    while host.serve_instance(prep, key, &certificate).await? {
        // Reboot the inner transports too. Dropping the outer UDP bridge
        // cannot deliver the inner QUIC CLOSE, and can otherwise leave an
        // active account owner until its timeout.
        let bytes = Zeroizing::new(random()?);
        atomic_private(
            &host.state,
            &host.state.join("s1-node.key"),
            bytes.as_ref(),
            true,
        )?;
        key = SecretKey::from_bytes(&bytes);
        prep = prepare(&root, &host.state, &host.settings.game).await?;
    }
    Ok(())
}

struct Host {
    state: PathBuf,
    settings: Settings,
    bind: SocketAddrV4,
    network: Network,
    actions: mpsc::Receiver<Action>,
    status: watch::Sender<Status>,
}

impl Host {
    async fn serve_instance(
        &mut self,
        prep: Prepared,
        key: SecretKey,
        certificate: &[u8],
    ) -> io::Result<bool> {
        let endpoint = self.network.endpoint(self.bind, Some(key)).await?;
        let profile = crate::load_config(&prep.runtime)?;
        let admission = profile
            .config
            .diagnostic_two_lobby
            .as_ref()
            .and_then(|c| c.account_admission.as_ref())
            .ok_or_else(|| io::Error::other("S1 needs managed accounts"))?;
        let services = Services {
            prelogin: admission
                .prelogin_tls
                .as_ref()
                .ok_or_else(|| io::Error::other("S1 needs prelogin TLS"))?
                .bind,
            relay: profile
                .config
                .authenticated_relay
                .as_ref()
                .ok_or_else(|| io::Error::other("S1 needs authenticated relay"))?
                .bind,
            launcher: admission.public_launcher.as_ref().map(|c| c.bind),
            name: self.settings.name.clone(),
            certificate: certificate.to_vec(),
        };
        let (stop, stopped) = oneshot::channel();
        let out = prep.out;
        let mut service = std::pin::pin!(crate::serve_until(profile, &out, async {
            let _ = stopped.await;
            Ok(())
        }));
        let wait_ready = async {
            loop {
                if monitoring::check(&out).is_ok() {
                    return Ok::<_, io::Error>(());
                }
                sleep(Duration::from_millis(150)).await;
            }
        };
        tokio::select! {
            result = &mut service => return result.map(|()| false),
            result = timeout(Duration::from_secs(20), wait_ready) => {
                if let Err(error) = result {
                    let _ = stop.send(());
                    service.await?;
                    return Err(error.into());
                }
            }
        }
        let relay_expected = !matches!(self.network, Network::Local);
        let mut backups = tokio::task::JoinSet::new();
        let mut timer = tokio::time::interval(Duration::from_secs(2));
        self.status.send_replace(Status {
            invite: Invite::server(endpoint.id(), certificate).to_string(),
            ready: true,
            relay_expected,
            relay_online: p2p::relay_connected(&endpoint),
            ..Status::default()
        });
        let result = {
            let server = p2p::serve(&endpoint, services);
            tokio::pin!(server);
            loop {
                tokio::select! {
                    ret = &mut server => break ret.map(|()| false),
                    ret = &mut service => return ret.map(|()| false),
                    _ = timer.tick() => {
                        let relay_online = relay_expected && p2p::relay_connected(&endpoint);
                        self.status.send_modify(|s| {
                            s.relay_online = relay_online;
                            if let Ok(health) = monitoring::check(&out) {
                                s.ready = true;
                                s.players = health["lobby"]["online_players"].as_u64().unwrap_or(0);
                                s.rooms = health["lobby"]["room_count"].as_u64().unwrap_or(0);
                            } else {
                                s.ready = false;
                            }
                        });
                    }
                    action = self.actions.recv() => match action {
                        Some(Action::Rotate) => break Ok(true),
                        Some(Action::Backup) if backups.is_empty() => {
                            let state = self.state.clone();
                            backups.spawn(async move { backup(&state).await });
                        }
                        Some(Action::Backup) => {}
                        Some(Action::Stop) | None => break Ok(false),
                    },
                    Some(result) = backups.join_next(), if !backups.is_empty() => {
                        self.status.send_modify(|s| s.message = match result {
                            Ok(Ok(path)) => path.display().to_string(),
                            Ok(Err(error)) => error.to_string(),
                            Err(error) => error.to_string(),
                        });
                    }
                }
            }
        };
        self.status.send_replace(Status::default());
        let _ = stop.send(());
        let (stopped, ()) = tokio::join!(service, endpoint.close());
        while backups.join_next().await.is_some() {}
        stopped?;
        result
    }
}

pub async fn command_line_stop(actions: mpsc::Sender<Action>) -> io::Result<()> {
    let mut signals = platform::Shutdown::new()?;
    signals.recv().await?;
    actions.send(Action::Stop).await.map_err(io::Error::other)
}

pub fn network_from_environment() -> io::Result<Network> {
    if std::env::var_os("BOOM_OFFLINE").is_some_and(|value| value == "1") {
        Ok(Network::Local)
    } else if let Some(url) = std::env::var_os("BOOM_RELAY_URL") {
        let relay = url
            .to_str()
            .ok_or_else(|| io::Error::other("invalid relay URL"))?
            .parse()
            .map_err(io::Error::other)?;
        let quic_port = std::env::var("BOOM_RELAY_QUIC_PORT")
            .ok()
            .map_or(Ok(7842), |value| value.parse().map_err(io::Error::other))?;
        let ca_der = std::env::var_os("BOOM_RELAY_CA").map_or_else(
            || Ok(Vec::new()),
            |path| {
                if fs::metadata(&path)?.len() > 16384 {
                    return Err(io::Error::other("private relay CA exceeds size limit"));
                }
                fs::read(path)
            },
        )?;
        Ok(Network::Private {
            relay,
            quic_port,
            ca_der,
        })
    } else {
        Ok(Network::Public)
    }
}

pub fn bind_from_environment() -> io::Result<SocketAddrV4> {
    std::env::var("BOOM_BIND")
        .unwrap_or_else(|_| "0.0.0.0:41600".into())
        .parse()
        .map_err(io::Error::other)
}
