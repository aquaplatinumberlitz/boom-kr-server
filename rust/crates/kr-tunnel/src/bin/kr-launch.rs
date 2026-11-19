//! First-party player launcher. Password/session/capability input is never
//! accepted on argv or printed; only the measured native frontend token is
//! passed through the existing KR command-line seam.
#![forbid(unsafe_code)]
use kr_tunnel::launch_tls;
use serde::Deserialize;
use std::{
    io::{self, Read, Write},
    net::{Ipv4Addr, SocketAddrV4, TcpStream},
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use zeroize::Zeroizing;

struct Secret(Zeroizing<String>);
impl<'de> Deserialize<'de> for Secret {
    fn deserialize<D: serde::Deserializer<'de>>(value: D) -> Result<Self, D::Error> {
        String::deserialize(value).map(|value| Self(Zeroizing::new(value)))
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    server: SocketAddrV4,
    certificate: PathBuf,
    reported_login_ipv4: Ipv4Addr,
    native_ip: Ipv4Addr,
    companion: PathBuf,
    native_program: PathBuf,
    native_arguments: Vec<String>,
    native_directory: PathBuf,
    port_budget: usize,
    #[serde(default)]
    cancel_file: Option<PathBuf>,
    #[serde(default)]
    owner_heartbeat: Option<PathBuf>,
    #[serde(default)]
    peer_diagnostics: bool,
    #[serde(default)]
    udp_backend: kr_tunnel::UdpBackend,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Credentials {
    login: String,
    password: Secret,
}
#[derive(Deserialize)]
struct Player {
    player_id: i64,
}
#[derive(Deserialize)]
struct Login {
    ok: bool,
    #[serde(default)]
    player: Option<Player>,
    #[serde(default)]
    session: Option<Secret>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Offer {
    endpoint: SocketAddrV4,
    certificate_der: Vec<u8>,
    capability: Secret,
}
#[derive(Deserialize)]
struct Launch {
    ok: bool,
    #[serde(default)]
    frontend: Option<Secret>,
    #[serde(default)]
    peer_transport: Option<Offer>,
}

fn request(config: &Config, certificate: &[u8], data: &[u8]) -> io::Result<Zeroizing<Vec<u8>>> {
    let socket = TcpStream::connect_timeout(&config.server.into(), Duration::from_secs(5))?;
    let mut stream = launch_tls::connect(socket, certificate.to_vec(), Duration::from_secs(5))?;
    launch_tls::write_frame(&mut stream, data, launch_tls::REQUEST_LIMIT)?;
    launch_tls::read_frame(&mut stream, launch_tls::RESPONSE_LIMIT)
}
fn encoded_secret(value: &Secret) -> io::Result<Zeroizing<String>> {
    serde_json::to_string(&*value.0)
        .map(Zeroizing::new)
        .map_err(io::Error::other)
}
struct OwnedChild(Child);
impl Drop for OwnedChild {
    fn drop(&mut self) {
        if self.0.try_wait().ok().flatten().is_none() {
            let _ = self.0.kill();
        }
        let _ = self.0.wait();
    }
}
struct Heartbeat {
    previous: Vec<u8>,
    changed: std::time::Instant,
}
fn cancelled(config: &Config, stop: &AtomicBool, heartbeat: &mut Heartbeat) -> io::Result<bool> {
    if stop.load(Ordering::Acquire) {
        return Ok(true);
    }
    if let Some(path) = &config.owner_heartbeat {
        let metadata = std::fs::symlink_metadata(path)?;
        if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > 32 {
            return Ok(true);
        }
        let bytes = std::fs::read(path)?;
        if bytes != heartbeat.previous {
            heartbeat.previous = bytes;
            heartbeat.changed = std::time::Instant::now();
        }
        if heartbeat.changed.elapsed() > Duration::from_secs(10) {
            return Ok(true);
        }
    }
    if let Some(path) = &config.cancel_file {
        match std::fs::symlink_metadata(path) {
            Ok(_) => return Ok(true),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
    }
    Ok(false)
}
fn launch(
    config: &Config,
    certificate: &[u8],
    player: i64,
    session: &Secret,
    stop: &AtomicBool,
    stage: &mut &'static str,
) -> io::Result<()> {
    let mut heartbeat = Heartbeat {
        previous: Vec::new(),
        changed: std::time::Instant::now(),
    };
    *stage = "launch_grant";
    if cancelled(config, stop, &mut heartbeat)? {
        return Err(io::Error::other("launcher cancelled"));
    }
    let encoded = encoded_secret(session)?;
    let query = Zeroizing::new(format!(
        "{{\"op\":\"launch\",\"session\":{},\"player_id\":{player},\"reported_login_ipv4\":\"{}\"}}\n",
        *encoded, config.reported_login_ipv4
    ));
    let response = request(config, certificate, query.as_bytes())?;
    let launch: Launch = serde_json::from_slice(&response)
        .map_err(|_| io::Error::other("invalid launcher response"))?;
    if !launch.ok {
        *stage = "launch_denied";
        return Err(io::Error::other("launch denied or busy"));
    }
    let frontend = launch
        .frontend
        .ok_or_else(|| io::Error::other("missing native grant"))?;
    if frontend.0.len() != 32
        || !frontend
            .0
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(io::Error::other("invalid native grant"));
    }
    let offer = launch
        .peer_transport
        .ok_or_else(|| io::Error::other("missing authenticated peer offer"))?;
    let capability = encoded_secret(&offer.capability)?;
    if cancelled(config, stop, &mut heartbeat)? {
        return Err(io::Error::other("launcher cancelled"));
    }
    let certificate_der =
        serde_json::to_string(&offer.certificate_der).map_err(io::Error::other)?;
    let diagnostics = if config.peer_diagnostics {
        ",\"diagnostics\":true"
    } else {
        ""
    };
    let backend = if config.udp_backend == kr_tunnel::UdpBackend::Portable {
        ",\"udp_backend\":\"portable\""
    } else {
        ""
    };
    let input = Zeroizing::new(format!(
        "{{\"peer_transport\":{{\"endpoint\":\"{}\",\"certificate_der\":{certificate_der},\"capability\":{}}},\"native_ip\":\"{}\",\"port_budget\":{}{diagnostics}{backend}}}",
        offer.endpoint, *capability, config.native_ip, config.port_budget
    ));
    *stage = "companion_start";
    let mut companion = OwnedChild(
        Command::new(&config.companion)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?,
    );
    {
        let mut stdin = companion.0.stdin.take().expect("piped child stdin");
        stdin.write_all(input.as_bytes())?;
    }
    // A separate thread enforces readiness timeout without polling a blocking
    // pipe indefinitely. Killing the owned child also releases the pipe reader.
    let stdout = companion.0.stdout.take().expect("piped child stdout");
    let (send, recv) = std::sync::mpsc::sync_channel(1);
    let reader = std::thread::spawn(move || {
        let mut stream = stdout;
        let result = launch_tls::read_frame(&mut stream, 256);
        let _ = send.send(result);
    });
    *stage = "companion_authentication";
    let readiness = recv.recv_timeout(Duration::from_secs(8));
    if readiness.is_err() {
        drop(companion);
        let _ = reader.join();
        return Err(io::Error::other("companion readiness deadline"));
    }
    let ready = readiness.expect("checked readiness")?;
    let _ = reader.join();
    let event: serde_json::Value = serde_json::from_slice(&ready).map_err(io::Error::other)?;
    if event["event"] != "transport_ready" {
        return Err(io::Error::other("companion did not authenticate"));
    }
    let native_token = Zeroizing::new(format!("token_1:{}", *frontend.0));
    if cancelled(config, stop, &mut heartbeat)? {
        return Err(io::Error::other("launcher cancelled"));
    }
    *stage = "native_start";
    let mut native = OwnedChild(
        Command::new(&config.native_program)
            .args(&config.native_arguments)
            .arg(&*native_token)
            .current_dir(&config.native_directory)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?,
    );
    println!("{{\"event\":\"client_started\",\"player_id\":{player}}}");
    *stage = "native_running";
    loop {
        if cancelled(config, stop, &mut heartbeat)? {
            return Err(io::Error::other("launcher cancelled"));
        }
        if let Some(status) = native.0.try_wait()? {
            // Numeric process status is safe diagnostic output: never include
            // the native command line, grant, session or companion offer.
            if !status.success() {
                println!(
                    "{{\"event\":\"client_failed\",\"exit_code\":{}}}",
                    status
                        .code()
                        .map_or("null".to_owned(), |code| code.to_string())
                );
            }
            return if status.success() {
                Ok(())
            } else {
                Err(io::Error::other("native client exited with failure"))
            };
        }
        if let Some(status) = companion.0.try_wait()? {
            *stage = "companion_stopped";
            let native_status = native.0.try_wait()?;
            println!(
                "{{\"event\":\"companion_stopped\",\"exit_code\":{},\"native_exit_code\":{}}}",
                status
                    .code()
                    .map_or("null".to_owned(), |code| code.to_string()),
                native_status
                    .and_then(|status| status.code())
                    .map_or("null".to_owned(), |code| code.to_string())
            );
            return Err(io::Error::other("authenticated transport stopped"));
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}
fn run(stop: &AtomicBool, stage: &mut &'static str) -> io::Result<()> {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 2 {
        return Err(io::Error::other(
            "usage: kr-launch <player-config.json>; credentials on stdin",
        ));
    }
    let config_path = std::fs::canonicalize(&args[1])?;
    let config_root = config_path
        .parent()
        .ok_or_else(|| io::Error::other("player config parent"))?;
    let config_bytes = std::fs::read(&config_path)?;
    if config_bytes.len() > 32768 {
        return Err(io::Error::other("player config bound"));
    }
    let mut config: Config = serde_json::from_slice(&config_bytes)
        .map_err(|_| io::Error::other("invalid player config"))?;
    if let Some(path) = &mut config.cancel_file {
        if !path.is_absolute() {
            *path = config_root.join(&*path);
        }
        let name = path
            .file_name()
            .ok_or_else(|| io::Error::other("cancel file name"))?
            .to_owned();
        *path = std::fs::canonicalize(
            path.parent()
                .ok_or_else(|| io::Error::other("cancel parent"))?,
        )?
        .join(name);
        if path.exists() {
            return Err(io::Error::other("player launch already cancelled"));
        }
    }
    if let Some(path) = &mut config.owner_heartbeat {
        if !path.is_absolute() {
            *path = config_root.join(&*path);
        }
        let metadata = std::fs::symlink_metadata(&*path)?;
        if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > 32 {
            return Err(io::Error::other("owner heartbeat bounds"));
        }
        *path = std::fs::canonicalize(&*path)?;
    }
    // Resolve at installation/config location, before changing the native
    // child's working directory. Relative executable paths differ by platform
    // when combined with Command::current_dir.
    for path in [
        &mut config.certificate,
        &mut config.companion,
        &mut config.native_program,
        &mut config.native_directory,
    ] {
        if !path.is_absolute() {
            *path = config_root.join(&*path);
        }
        *path = std::fs::canonicalize(&*path)?;
    }
    if config.server.port() == 0
        || config.server.ip().is_unspecified()
        || config.server.ip().is_multicast()
        || config.server.ip().is_broadcast()
        || config.native_ip.is_unspecified()
        || config.native_ip.is_multicast()
        || config.native_ip.is_broadcast()
        || config.reported_login_ipv4.is_unspecified()
        || config.reported_login_ipv4.is_multicast()
        || config.reported_login_ipv4.is_broadcast()
        || !(26..=32768).contains(&config.port_budget)
        || config.native_arguments.len() > 32
        || config
            .native_arguments
            .iter()
            .any(|arg| arg.len() > 4096 || arg.starts_with("token_1:"))
    {
        return Err(io::Error::other("invalid player bounds"));
    }
    let certificate = std::fs::read(&config.certificate)?;
    if certificate.is_empty() || certificate.len() > 8192 {
        return Err(io::Error::other("launcher certificate bound"));
    }
    let mut bytes = Zeroizing::new(Vec::new());
    io::stdin().lock().take(4097).read_to_end(&mut bytes)?;
    if bytes.len() > 4096 {
        return Err(io::Error::other("credential input bound"));
    }
    let credentials: Credentials =
        serde_json::from_slice(&bytes).map_err(|_| io::Error::other("invalid credential input"))?;
    drop(bytes);
    let login = serde_json::to_string(&credentials.login).map_err(io::Error::other)?;
    let password = encoded_secret(&credentials.password)?;
    let query = Zeroizing::new(format!(
        "{{\"op\":\"login\",\"login\":{login},\"password\":{}}}\n",
        *password
    ));
    *stage = "account_login";
    let response = request(&config, &certificate, query.as_bytes())?;
    drop(query);
    drop(password);
    drop(credentials);
    let login: Login = serde_json::from_slice(&response)
        .map_err(|_| io::Error::other("invalid login response"))?;
    if !login.ok {
        *stage = "account_denied";
        return Err(io::Error::other("login denied or busy"));
    }
    let player = login
        .player
        .ok_or_else(|| io::Error::other("missing player identity"))?
        .player_id;
    let session = login
        .session
        .ok_or_else(|| io::Error::other("missing session"))?;
    let result = launch(&config, &certificate, player, &session, stop, stage);
    let encoded = encoded_secret(&session)?;
    let query = Zeroizing::new(format!(
        "{{\"op\":\"logout\",\"session\":{},\"player_id\":{player}}}\n",
        *encoded
    ));
    let logout = request(&config, &certificate, query.as_bytes());
    if result.is_ok() {
        *stage = "session_logout";
        let frame = logout?;
        let response: serde_json::Value =
            serde_json::from_slice(&frame).map_err(io::Error::other)?;
        if response["ok"] != true {
            return Err(io::Error::other(
                "logout denied or busy; session remains bounded by expiry",
            ));
        }
    }
    result
}
#[cfg(unix)]
fn termination_signal() -> io::Result<impl std::future::Future<Output = ()>> {
    let mut signal = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    Ok(async move {
        signal.recv().await;
    })
}
#[cfg(windows)]
fn termination_signal() -> io::Result<impl std::future::Future<Output = ()>> {
    let mut close = tokio::signal::windows::ctrl_close()?;
    let mut shutdown = tokio::signal::windows::ctrl_shutdown()?;
    let mut interrupt = tokio::signal::windows::ctrl_break()?;
    Ok(async move {
        tokio::select! { _=close.recv()=>{}, _=shutdown.recv()=>{}, _=interrupt.recv()=>{} }
    })
}
#[tokio::main]
async fn main() {
    let terminate = match termination_signal() {
        Ok(signal) => signal,
        Err(_) => {
            eprintln!("Cannot install launcher lifecycle handler.");
            std::process::exit(1);
        }
    };
    let cancelled = Arc::new(AtomicBool::new(false));
    let stop = cancelled.clone();
    let mut task = tokio::task::spawn_blocking(move || {
        let mut stage = "installation";
        let result = run(&stop, &mut stage);
        if result.is_err() {
            // Only fixed stage labels; never print arbitrary I/O errors,
            // endpoint paths, command lines or authentication bodies.
            eprintln!("{{\"event\":\"launcher_failed\",\"stage\":\"{stage}\"}}");
        }
        result
    });
    tokio::pin!(terminate);
    let finished = tokio::select! {
        result=&mut task=>Some(result),
        _=tokio::signal::ctrl_c()=>None,
        _=&mut terminate=>None,
    };
    let result = match finished {
        Some(result) => result.map_err(io::Error::other).and_then(|result| result),
        None => {
            cancelled.store(true, Ordering::Release);
            tokio::time::timeout(Duration::from_secs(20), &mut task)
                .await
                .map_err(|_| io::Error::other("launcher cleanup deadline"))
                .and_then(|result| result.map_err(io::Error::other))
                .and_then(|result| result)
        }
    };
    if result.is_err() {
        eprintln!(
            "Launcher stopped; check installation, trust, credentials and service availability."
        );
    }
    // A cancelled stdin read before child creation owns no account/process;
    // process exit avoids waiting indefinitely in blocking runtime teardown.
    std::process::exit(if result.is_ok() { 0 } else { 1 });
}
