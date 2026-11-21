//! Optional first-party launcher API; TLS credentials never enter native KR
//! packets. The transport peer comes from the accepted socket, never JSON.
use crate::{
    FixturePin,
    account_serving::{Active, LaunchRequest, Secret},
    account_worker::rate::{Limiter, RatePolicy},
    accounts::Management,
};
use kr_tunnel::launch_tls::{self, ServerTls};
use serde::Deserialize;
use std::{
    fs, io,
    net::{Ipv4Addr, SocketAddrV4, TcpStream},
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{
    net::TcpListener,
    sync::{Semaphore, watch},
    task::JoinSet,
};
use zeroize::Zeroizing;

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub bind: SocketAddrV4,
    pub certificate: FixturePin,
    pub private_key: PathBuf,
    pub max_connections: usize,
    pub timeout_ms: u64,
    pub session_ttl_seconds: i64,
    pub session_idle_seconds: i64,
}
impl Config {
    pub(crate) fn validate(&self) -> io::Result<()> {
        if self.bind.port() == 0
            || self.bind.ip().is_multicast()
            || self.bind.ip().is_broadcast()
            || !(1..=64).contains(&self.max_connections)
            || !(250..=2000).contains(&self.timeout_ms)
            || !(1..=86400).contains(&self.session_ttl_seconds)
            || !(1..=self.session_ttl_seconds).contains(&self.session_idle_seconds)
            || !self.private_key.is_absolute()
            || self
                .private_key
                .components()
                .any(|part| matches!(part, std::path::Component::ParentDir))
        {
            return Err(io::Error::other("public launcher configuration bounds"));
        }
        let parent = self
            .private_key
            .parent()
            .ok_or_else(|| io::Error::other("launcher key parent"))?;
        crate::platform::private_dir(parent)?;
        let file = crate::platform::private_file(&self.private_key)?;
        if file.len() == 0 || file.len() > 8192 {
            return Err(io::Error::other("launcher key has invalid length"));
        }
        Ok(())
    }
    pub(crate) async fn open(&self) -> io::Result<Opened> {
        self.validate()?;
        let tls =
            launch_tls::server_config(self.certificate.load()?, fs::read(&self.private_key)?)?;
        let listener = TcpListener::bind(self.bind).await?;
        Ok(Opened { listener, tls })
    }
}
pub(crate) struct Opened {
    listener: TcpListener,
    tls: ServerTls,
}

#[derive(Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
enum Request {
    Status,
    Account {
        login: String,
        action: Management,
    },
    Logout {
        session: Secret,
        player_id: i64,
    },
    Login {
        login: String,
        password: Secret,
    },
    Launch {
        session: Secret,
        player_id: i64,
        #[serde(default)]
        reported_login_ipv4: Option<Ipv4Addr>,
    },
}

struct Access {
    peers: Vec<Ipv4Addr>,
    registration: Mutex<Limiter>,
}
impl Access {
    fn register(&self, peer: Ipv4Addr) -> bool {
        let key = Limiter::key(&peer.to_string());
        let now = std::time::Instant::now();
        if let Ok(mut table) = self.registration.lock()
            && table.check(key, now).is_ok()
        {
            table.commit(key, now);
            return true;
        }
        false
    }
}

fn handle(
    socket: TcpStream,
    tls: ServerTls,
    config: &Config,
    active: &Active,
    access: &Access,
    peer: Ipv4Addr,
    runtime: &tokio::runtime::Handle,
) -> io::Result<()> {
    let mut stream = launch_tls::accept(socket, tls, Duration::from_millis(config.timeout_ms))?;
    let input = launch_tls::read_frame(&mut stream, launch_tls::REQUEST_LIMIT)?;
    let request: Request = serde_json::from_slice(&input)
        .map_err(|_| io::Error::other("invalid public launcher request"))?;
    drop(input);
    match request {
        Request::Status => {
            // TLS endpoint + admission health only; no player/session data and
            // no database or native-match health claim.
            let response = if active.client.is_closed() {
                b"{\"ok\":true,\"ready\":false}\n".as_slice()
            } else {
                b"{\"ok\":true,\"ready\":true}\n".as_slice()
            };
            launch_tls::write_frame(&mut stream, response, launch_tls::RESPONSE_LIMIT)
        }
        Request::Account { login, action } => {
            let result = if matches!(&action, Management::Register { .. }) && !access.register(peer)
            {
                None
            } else {
                let remaining = stream.sock.remaining()?;
                match active.client.try_manage(login, action) {
                    Ok(pending) => runtime
                        .block_on(async { tokio::time::timeout(remaining, pending.wait()).await })
                        .ok()
                        .and_then(Result::ok),
                    Err(_) => None,
                }
            };
            let response = Zeroizing::new(match result {
                Some(Some(code)) => format!("{{\"ok\":true,\"recovery_code\":\"{}\"}}\n", *code),
                Some(None) => "{\"ok\":true}\n".into(),
                None => "{\"ok\":false,\"error\":\"denied_or_busy\"}\n".into(),
            });
            launch_tls::write_frame(&mut stream, response.as_bytes(), launch_tls::RESPONSE_LIMIT)
        }
        Request::Logout { session, player_id } => {
            let token = crate::accounts::Token::parse(&session.0)
                .map_err(|_| io::Error::other("invalid public session"))?;
            let remaining = stream.sock.remaining()?;
            let result = match active.client.try_logout_current(&token, player_id) {
                Ok(pending) => runtime
                    .block_on(async { tokio::time::timeout(remaining, pending.wait()).await })
                    .map_err(|_| io::Error::other("public logout deadline"))?,
                Err(error) => Err(error),
            };
            if result.is_ok() {
                active
                    .bridge
                    .forget_session(&token, player_id)
                    .map_err(|_| io::Error::other("public logout registry unavailable"))?;
            }
            let response = if result.is_ok() {
                b"{\"ok\":true}\n".as_slice()
            } else {
                b"{\"ok\":false,\"error\":\"denied_or_busy\"}\n".as_slice()
            };
            launch_tls::write_frame(&mut stream, response, launch_tls::RESPONSE_LIMIT)
        }
        Request::Login { login, password } => {
            let pending = active.client.try_login_with_idle_current(
                &login,
                password.0.as_bytes(),
                config.session_ttl_seconds,
                config.session_idle_seconds,
            );
            drop(password);
            let received = match pending {
                Ok(pending) => {
                    let remaining = stream.sock.remaining()?;
                    runtime
                        .block_on(async {
                            tokio::time::timeout(remaining, pending.receive()).await
                        })
                        .map_err(|_| io::Error::other("public login deadline"))?
                }
                Err(error) => Err(error),
            };
            let received = match received {
                Ok(received) => received,
                Err(_) => {
                    return launch_tls::write_frame(
                        &mut stream,
                        b"{\"ok\":false,\"error\":\"denied_or_busy\"}\n",
                        launch_tls::RESPONSE_LIMIT,
                    );
                }
            };
            let grant = received.value();
            let player = serde_json::to_string(&grant.player).map_err(io::Error::other)?;
            let response = Zeroizing::new(format!(
                "{{\"ok\":true,\"player\":{player},\"session\":\"{}\",\"expires_at\":{}}}\n",
                grant.token.expose_secret(),
                grant.expires_at
            ));
            launch_tls::write_frame(&mut stream, response.as_bytes(), launch_tls::RESPONSE_LIMIT)?;
            received
                .accept()
                .map_err(|_| io::Error::other("public login handoff expired"))?;
            Ok(())
        }
        Request::Launch {
            session,
            player_id,
            reported_login_ipv4,
        } => {
            let request = LaunchRequest {
                session,
                player_id,
                peer,
                reported_login_ipv4,
            };
            let remaining = stream.sock.remaining()?;
            let prepared = runtime
                .block_on(async {
                    tokio::time::timeout(
                        remaining,
                        crate::account_serving::prepare_launch(request, active, &access.peers),
                    )
                    .await
                })
                .map_err(|_| io::Error::other("public launch deadline"))?;
            let prepared = match prepared {
                Ok(prepared) => prepared,
                Err(_) => {
                    return launch_tls::write_frame(
                        &mut stream,
                        b"{\"ok\":false,\"error\":\"denied_or_busy\"}\n",
                        launch_tls::RESPONSE_LIMIT,
                    );
                }
            };
            launch_tls::write_frame(&mut stream, prepared.bytes(), launch_tls::RESPONSE_LIMIT)?;
            prepared.delivered();
            Ok(())
        }
    }
}

pub(crate) async fn serve(
    opened: Opened,
    config: Config,
    active: Active,
    peers: Vec<Ipv4Addr>,
    mut stop: watch::Receiver<bool>,
) -> io::Result<()> {
    let quota = Arc::new(Semaphore::new(config.max_connections));
    let mut jobs = JoinSet::new();
    let config = Arc::new(config);
    let access = Arc::new(Access {
        peers,
        registration: Mutex::new(
            Limiter::new(RatePolicy {
                attempts: 3,
                principals: 256,
                interval: Duration::from_secs(60),
            })
            .map_err(|_| io::Error::other("registration rate policy"))?,
        ),
    });
    let runtime = tokio::runtime::Handle::current();
    loop {
        if *stop.borrow() {
            break;
        }
        tokio::select! {
            biased;
            _ = stop.changed() => break,
            result = jobs.join_next(), if !jobs.is_empty() => {
                // Request failures are expected and contain no request/secret
                // logs. A panic in a bounded handler is a listener fault.
                if result.is_some_and(|result| result.is_err()) {
                    return Err(io::Error::other("public launcher handler failed"));
                }
            },
            incoming = opened.listener.accept() => {
                let (socket, address) = incoming?;
                let std::net::IpAddr::V4(peer) = address.ip() else { continue; };
                if !access.peers.contains(&peer) { continue; }
                let Ok(permit) = quota.clone().try_acquire_owned() else { continue; };
                let socket = socket.into_std()?;
                socket.set_nonblocking(false)?;
                let (tls, config, active, access, runtime) = (opened.tls.clone(), config.clone(), active.clone(), access.clone(), runtime.clone());
                jobs.spawn_blocking(move || {
                    let _permit = permit;
                    handle(socket, tls, &config, &active, &access, peer, &runtime)
                });
            },
        }
    }
    // Each blocking handler has one absolute <=2s deadline, including its DB
    // receive; drain before the account worker shuts down.
    drop(opened);
    while let Some(result) = jobs.join_next().await {
        if result.is_err() {
            return Err(io::Error::other("public launcher handler failed"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        account_worker::Worker,
        accounts::{Admin, Store, TEST_SERIAL, Token},
    };
    use std::os::unix::fs::PermissionsExt;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn tls_login_uses_real_sqlite_identity_and_keeps_failures_secret_free() {
        let _serial = TEST_SERIAL.lock().unwrap();
        let directory = std::env::temp_dir().join(format!(
            "kr-public-launch-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&directory).unwrap();
        fs::set_permissions(&directory, fs::Permissions::from_mode(0o700)).unwrap();
        let database = directory.join("accounts.sqlite");
        let mut store = Store::open(&database).unwrap();
        let admin = Admin {
            actor: "test-admin",
            reason: "public launcher TLS test",
        };
        store.begin_instance(&admin, 100).unwrap();
        let player = store
            .create_account(&admin, "Alice", "Alice", b"test-login-password", 100)
            .unwrap();
        drop(store);
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(async {
            let worker = Worker::open(database.clone(), 4, Duration::from_secs(3)).await.unwrap();
            let active = Active::login_test_client(worker.client());
            let (certificate, key) = launch_tls::generate_identity().unwrap();
            let cert_path = directory.join("certificate.der");
            let key_path = directory.join("private.pk8");
            fs::write(&cert_path, &certificate).unwrap();
            fs::write(&key_path, &*key).unwrap();
            fs::set_permissions(&key_path, fs::Permissions::from_mode(0o600)).unwrap();
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let std::net::SocketAddr::V4(address) = listener.local_addr().unwrap() else { unreachable!() };
            let config = Config { bind: address, certificate: FixturePin { path: cert_path,
                bytes: certificate.len(), sha256: crate::sha256_hex(&certificate) }, private_key: key_path,
                max_connections: 2, timeout_ms: 2000, session_ttl_seconds: 60, session_idle_seconds: 30 };
            config.validate().unwrap();
            let opened = Opened { listener, tls: launch_tls::server_config(certificate.clone(), key.to_vec()).unwrap() };
            let (halt, stop) = watch::channel(false);
            let server = tokio::spawn(serve(opened, config, active, vec![Ipv4Addr::LOCALHOST], stop));
            let status_certificate = certificate.clone();
            let status = tokio::task::spawn_blocking(move || {
                let mut tls = launch_tls::connect(TcpStream::connect(address).unwrap(), status_certificate, Duration::from_secs(3)).unwrap();
                launch_tls::write_frame(&mut tls, b"{\"op\":\"status\"}\n", launch_tls::REQUEST_LIMIT).unwrap();
                serde_json::from_slice::<serde_json::Value>(&launch_tls::read_frame(&mut tls, launch_tls::RESPONSE_LIMIT).unwrap()).unwrap()
            }).await.unwrap();
            assert_eq!(status, serde_json::json!({"ok":true,"ready":true}));
            for correct in [false, true] {
                let certificate = certificate.clone();
                let response = tokio::task::spawn_blocking(move || {
                    let mut tls = launch_tls::connect(TcpStream::connect(address).unwrap(), certificate, Duration::from_secs(3)).unwrap();
                    let request = if correct { b"{\"op\":\"login\",\"login\":\"ALICE\",\"password\":\"test-login-password\"}\n".as_slice() }
                        else { b"{\"op\":\"login\",\"login\":\"Alice\",\"password\":\"wrong-test-password\"}\n".as_slice() };
                    launch_tls::write_frame(&mut tls, request, launch_tls::REQUEST_LIMIT).unwrap();
                    let frame = launch_tls::read_frame(&mut tls, launch_tls::RESPONSE_LIMIT).unwrap();
                    serde_json::from_slice::<serde_json::Value>(&frame).unwrap()
                }).await.unwrap();
                if !correct {
                    assert_eq!(response, serde_json::json!({"ok":false,"error":"denied_or_busy"}));
                } else {
                    assert_eq!(response["player"]["player_id"].as_i64(), Some(player.player_id));
                    let token = Token::parse(response["session"].as_str().unwrap()).unwrap();
                    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
                    assert_eq!(worker.client().try_authorize(&token, player.player_id, now).unwrap().wait().await,
                        Ok(player.clone()));
                    assert!(worker.client().try_authorize(&token, player.player_id + 1, now).unwrap().wait().await.is_err());
                }
            }
            for index in 0..4 {
                let cert = certificate.clone();
                let response = tokio::task::spawn_blocking(move || {
                    let mut tls = launch_tls::connect(TcpStream::connect(address).unwrap(), cert, Duration::from_secs(3)).unwrap();
                    let request = format!("{{\"op\":\"account\",\"login\":\"New{index}\",\"action\":{{\"kind\":\"register\",\"display_name\":\"New{index}\",\"password\":\"self-service-test-only\"}}}}\n");
                    launch_tls::write_frame(&mut tls, request.as_bytes(), launch_tls::REQUEST_LIMIT).unwrap();
                    serde_json::from_slice::<serde_json::Value>(&launch_tls::read_frame(&mut tls, launch_tls::RESPONSE_LIMIT).unwrap()).unwrap()
                }).await.unwrap();
                if index < 3 {
                    assert_eq!(response["ok"], true);
                    assert_eq!(response["recovery_code"].as_str().unwrap().len(), 64);
                } else {
                    assert_eq!(response, serde_json::json!({"ok":false,"error":"denied_or_busy"}));
                }
            }
            halt.send(true).unwrap();
            server.await.unwrap().unwrap();
            worker.shutdown().await.unwrap();
        });
        fs::remove_dir_all(directory).unwrap();
    }
    #[test]
    fn client_cannot_supply_authoritative_peer_or_extra_identity_fields() {
        for input in [
            r#"{"op":"login","login":"Alice","password":"test-only-password","peer":"127.0.0.2"}"#,
            r#"{"op":"launch","session":"test-only","player_id":1,"peer":"127.0.0.2"}"#,
            r#"{"op":"launch","session":"test-only","player_id":1,"account_id":2}"#,
        ] {
            assert!(serde_json::from_str::<Request>(input).is_err());
        }
        assert!(
            serde_json::from_str::<Request>(
                r#"{"op":"launch","session":"test-only","player_id":1}"#
            )
            .is_ok()
        );
    }
}
