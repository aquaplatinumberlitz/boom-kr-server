//! Launcher ownership for the O1 service, room endpoint, and companion recipe.
use crate::game;
use kr_p2p::{
    Endpoint, Invite,
    room::{self, Discovery, Route},
};
use kr_tunnel::prelogin::{
    friends::{self, Gateway, Guest},
    heartbeat,
};
use serde::Deserialize;
use std::{
    fs::{self, File},
    io,
    net::{Ipv4Addr, SocketAddrV4},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Config {
    package: PathBuf,
    manifest_sha256: String,
    python: PathBuf,
}

impl Config {
    pub fn resolve(&mut self, root: &Path) -> io::Result<()> {
        fingerprint(&self.manifest_sha256)?;
        for path in [&mut self.package, &mut self.python] {
            if !path.is_absolute() {
                *path = root.join(&*path);
            }
            *path = path.canonicalize()?;
        }
        Ok(())
    }
}

fn fingerprint(value: &str) -> io::Result<[u8; 32]> {
    if value.len() != 64 || !value.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(io::Error::other("package fingerprint"));
    }
    let mut hash = [0; 32];
    for (index, byte) in hash.iter_mut().enumerate() {
        *byte =
            u8::from_str_radix(&value[index * 2..index * 2 + 2], 16).map_err(io::Error::other)?;
    }
    Ok(hash)
}

pub(crate) fn guest(config: &game::Config, route: Route, name: String) -> io::Result<Guest> {
    Ok(Guest {
        code: Invite::room(route.node).to_string(),
        route,
        package: fingerprint(&config.manifest_sha256)?,
        name,
    })
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Prepared {
    config: PathBuf,
    out: PathBuf,
    address: SocketAddrV4,
    certificate: PathBuf,
    ports: Vec<u16>,
}

fn hidden(executable: &Path) -> Command {
    let mut command = Command::new(executable);
    command.stdin(Stdio::null()).stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x0800_0000);
    }
    command
}

struct Service {
    child: Option<Child>,
    lock: Option<File>,
    beat: PathBuf,
    tick: u64,
}
impl Service {
    fn check(&mut self) -> io::Result<()> {
        let child = self.child.as_mut().expect("owned child");
        if child.try_wait()?.is_some() {
            return Err(io::Error::other("friends server exited"));
        }
        self.tick += 1;
        heartbeat::publish(&self.beat, self.tick)
    }
}
impl Drop for Service {
    fn drop(&mut self) {
        let _ = fs::write(self.beat.with_extension("cancel"), []);
        if let Some(mut child) = self.child.take() {
            let lock = self.lock.take();
            thread::spawn(move || {
                let deadline = Instant::now() + Duration::from_secs(12);
                while matches!(child.try_wait(), Ok(None)) && Instant::now() < deadline {
                    thread::sleep(Duration::from_millis(50));
                }
                let _ = child.kill();
                let _ = child.wait();
                drop(lock);
            });
        }
    }
}

fn prepare(cfg: &game::Config) -> io::Result<(Service, Gateway)> {
    let spec = cfg
        .friends
        .as_ref()
        .ok_or_else(|| io::Error::other("no O1 server in package"))?;
    let (value, service) = {
        let lock = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(cfg.state.join("friends.lock"))?;
        lock.try_lock().map_err(io::Error::other)?;
        let output = hidden(&spec.python)
            .current_dir(&spec.package)
            .arg("-I")
            .arg("-B")
            .arg(spec.package.join("kr-package.py"))
            .arg("prepare-friends")
            .arg("--manifest-sha256")
            .arg(&spec.manifest_sha256)
            .arg("--state")
            .arg(cfg.state.join("friends-data"))
            .arg("--game-dir")
            .arg(&cfg.bundle)
            .arg("--player-manifest-sha256")
            .arg(&cfg.manifest_sha256)
            .stderr(Stdio::piped())
            .output()?;
        if !output.status.success() || output.stdout.len() > 16384 {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let tail = stderr.trim();
            let tail = tail
                .char_indices()
                .rev()
                .nth(239)
                .map_or(tail, |(idx, _)| &tail[idx..]);
            return Err(io::Error::other(format!(
                "friends server setup (exit={:?}): {tail}",
                output.status.code()
            )));
        }
        let value: Prepared = serde_json::from_slice(&output.stdout)?;
        if value.ports != cfg.ports || !value.address.ip().is_loopback() {
            return Err(io::Error::other("friends server port mismatch"));
        }
        let parent = value
            .out
            .parent()
            .ok_or_else(|| io::Error::other("friends run directory"))?;
        let beat = parent.join("owner.beat");
        heartbeat::publish(&beat, 0)?;
        let service = hidden(&spec.package.join(if cfg!(windows) {
            "bin/kr-service.exe"
        } else {
            "bin/kr-service"
        }))
        .current_dir(&spec.package)
        .arg("--config")
        .arg(&value.config)
        .arg("--out")
        .arg(&value.out)
        .arg("--owner-heartbeat")
        .arg(&beat)
        .stdout(Stdio::null())
        .spawn()?;
        (
            value,
            Service {
                child: Some(service),
                lock: Some(lock),
                beat,
                tick: 0,
            },
        )
    };
    let mut service = service;
    let deadline = Instant::now() + Duration::from_secs(12);
    loop {
        service.check()?;
        if fs::read(value.out.join("service-state.json"))
            .ok()
            .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
            .is_some_and(|state| state["ready"] == true && state["stopped"] == false)
        {
            break;
        }
        if Instant::now() >= deadline {
            return Err(io::Error::other("friends server startup deadline"));
        }
        thread::sleep(Duration::from_millis(100));
    }
    Ok((
        service,
        Gateway {
            server: value.address,
            certificate: fs::read(value.certificate)?,
            ports: value.ports,
            package: fingerprint(&cfg.manifest_sha256)?,
            backend: cfg.udp_backend,
        },
    ))
}

#[derive(Clone)]
pub(crate) struct Host(Arc<OwnedHost>);
struct OwnedHost {
    pub route: Route,
    service: Mutex<Service>,
    endpoint: Endpoint,
    gateway: tokio::task::JoinHandle<io::Result<()>>,
    _discovery: Option<Discovery>,
    directory: Option<tokio::task::JoinHandle<io::Result<()>>>,
}
impl Drop for OwnedHost {
    fn drop(&mut self) {
        self.gateway.abort();
        if let Some(task) = &self.directory {
            task.abort();
        }
        let endpoint = self.endpoint.clone();
        if let Ok(runtime) = tokio::runtime::Handle::try_current() {
            runtime.spawn(async move {
                endpoint.close().await;
            });
        }
    }
}
impl Host {
    pub fn route(&self) -> Route {
        self.0.route.clone()
    }
    pub fn code(&self) -> String {
        Invite::room(self.0.route.node).to_string()
    }
    pub fn healthy(&self) -> bool {
        !self.0.gateway.is_finished()
            && self
                .0
                .service
                .lock()
                .is_ok_and(|mut service| service.check().is_ok())
    }
}

pub(crate) async fn host(cfg: game::Config, name: String) -> io::Result<Host> {
    let relays = cfg.friends_directory.clone();
    let (service, gateway) = tokio::task::spawn_blocking(move || prepare(&cfg))
        .await
        .map_err(io::Error::other)??;
    let endpoint = room::endpoint(!relays.is_empty()).await?;
    let port = endpoint
        .bound_sockets()
        .into_iter()
        .find(|addr| addr.is_ipv4())
        .ok_or_else(|| io::Error::other("no IPv4 room socket"))?
        .port();
    let node = endpoint.id();
    let discovery = Discovery::new()
        .and_then(|discovery| {
            discovery.advertise_host(node, port, &name)?;
            Ok(discovery)
        })
        .ok();
    let task = tokio::spawn(friends::serve(endpoint.clone(), gateway));
    let directory = (!relays.is_empty())
        .then(|| tokio::spawn(kr_p2p::directory::publish(endpoint.clone(), relays)));
    Ok(Host(Arc::new(OwnedHost {
        route: Route {
            node,
            addresses: vec![(Ipv4Addr::LOCALHOST, port).into()],
            relay: None,
        },
        service: Mutex::new(service),
        endpoint,
        gateway: task,
        _discovery: discovery,
        directory,
    })))
}

pub(crate) async fn discover() -> io::Result<Vec<room::DiscoveredRoom>> {
    Discovery::new()?.named_rooms(Duration::from_secs(2)).await
}

pub(crate) async fn resolve(code: Invite, relays: Vec<String>) -> Option<Route> {
    let lan = discover().await.ok().and_then(|rooms| {
        rooms
            .into_iter()
            .map(|room| room.route)
            .find(|route| code.matches_node(route.node))
    });
    if lan.is_some() || relays.is_empty() {
        return lan;
    }
    kr_p2p::directory::resolve(&code, &relays).await.ok()
}
