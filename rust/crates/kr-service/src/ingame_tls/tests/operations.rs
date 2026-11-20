use super::*;
use crate::operations::{LiveCommand, RoomTarget};
use serde_json::{Value, json};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    net::{SocketAddrV4, UdpSocket},
    os::unix::fs::OpenOptionsExt,
    path::Path,
    process::{Child, Command, Stdio},
};
use tokio::time::{sleep, timeout};

const CHILD: &str = "ingame_tls::tests::operations::server_process";

#[derive(serde::Serialize, serde::Deserialize)]
struct ServerConfig {
    db: PathBuf,
    out: PathBuf,
    tls: SocketAddrV4,
    relay: SocketAddrV4,
    schedule: bool,
    tcp: Vec<u16>,
}

fn port() -> SocketAddrV4 {
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    match socket.local_addr().unwrap() {
        std::net::SocketAddr::V4(addr) => addr,
        _ => unreachable!(),
    }
}

fn loopback_profile(input: &ServerConfig) -> crate::Validated {
    // Synthetic provenance, no game process, native fixtures or production
    // config changes. Everything after fixture loading is the actual server.
    let path = input.db.with_file_name("cert.der");
    let tls = Config {
        bind: input.tls,
        certificate: FixturePin {
            sha256: crate::sha256_hex(&fs::read(&path).unwrap()),
            bytes: fs::metadata(&path).unwrap().len() as usize,
            path,
        },
        private_key: input.db.with_file_name("private.pk8"),
        ..resources_config()
    };
    let packet = crate::Reply {
        header: [0; 37],
        opcode: 0,
        payload: vec![0x5a],
    };
    let names = [
        "ready",
        "channel_list",
        "auth_A",
        "netinfo",
        "flags",
        "redirect",
        "channel",
    ];
    let relay = crate::authenticated_peer::Config {
        bind: input.relay,
        advertise: input.relay,
        max_peers: 4,
        route_capacity: 4,
        queue_messages: 4,
        send_budget: 4,
        pending_ms: 500,
        lease_seconds: 120,
    };
    let authenticated = crate::authenticated_peer::Hub::new(&relay).unwrap();
    let managed = crate::room_service::Hub::new(
        &crate::room_service::Config {
            max_rooms: 8,
            round_profile: None,
            player_choices: true,
            capacity: None,
            player_round: Some(crate::room_io::PlayerRoundConfig {
                characters: vec![0, 1],
            }),
        },
        crate::room_io::registry_foundation_tests::player_round_prototype(),
        None,
        Some(authenticated.clone()),
    )
    .unwrap();
    let pin = tls.certificate.clone();
    crate::Validated {
        config: crate::Config {
            udp_relay: None,
            authenticated_relay: Some(relay),
            schema: "kr-rust-diagnostic-v1".into(),
            diagnostic_login_error_only: false,
            bind: Ipv4Addr::LOCALHOST,
            allowed_peers: vec![Ipv4Addr::LOCALHOST],
            network_scope: Default::default(),
            ports: input.tcp.clone(),
            max_connections: 8,
            idle_timeout_seconds: 15,
            write_timeout_ms: 2000,
            bootstrap: pin.clone(),
            login_error: pin.clone(),
            wcp_second: pin,
            diagnostic_local_lobby: None,
            diagnostic_two_lobby: Some(crate::TwoLobbyConfig {
                wcp_ports: vec![],
                login_port: input.tcp[0],
                login_candidate_ports: vec![],
                channel_port: input.tcp[1],
                actors: vec![],
                fixed_room: None,
                managed_rooms: None,
                account_admission: Some(crate::account_serving::Config {
                    backup: input.schedule.then_some(crate::operations::BackupSchedule {
                        interval_seconds: 1,
                        keep: 2,
                    }),
                    prelogin_tls: Some(tls),
                    public_launcher: None,
                    persist_character: false,
                    persist_matches: true,
                    database: input.db.clone(),
                    launch_socket: input.db.with_file_name("launch.sock"),
                    launch_ttl_seconds: 60,
                    worker_queue_capacity: 16,
                    profile_mode: crate::account_serving::ProfileMode::DatabaseZero,
                    patched_949x_names: true,
                    max_sessions: Some(16),
                    diagnostic_identity_strings: Default::default(),
                }),
            }),
        },
        bootstrap: vec![0; 15],
        error_header: [0; 37],
        wcp_second: vec![1; 15],
        lobby: None,
        bound: Some(crate::BoundProfiles {
            authenticated: Some(authenticated),
            account_template: Some(templates().account),
            managed: Some(managed),
            actors: [(
                1,
                crate::ActorProfile {
                    identity: 1,
                    name: b"Template".to_vec(),
                    packets: names
                        .into_iter()
                        .map(|name| (name.into(), packet.clone()))
                        .collect(),
                    reported_login_ipv4: None,
                    configured_peer: Ipv4Addr::LOCALHOST,
                },
            )]
            .into(),
            legacy_peers: Default::default(),
            registry: Default::default(),
            rooms: None,
        }),
        accounts: None,
    }
}
fn resources_config() -> Config {
    Config {
        bind: "127.0.0.1:1".parse().unwrap(),
        certificate: FixturePin {
            path: PathBuf::new(),
            sha256: String::new(),
            bytes: 0,
        },
        private_key: PathBuf::new(),
        max_connections: 4,
        max_unauthenticated: 4,
        max_per_ip: 4,
        prelogin_seconds: 15,
        admissions_per_ip_per_minute: 20,
        session_ttl_seconds: 120,
        session_idle_seconds: 90,
    }
}

#[test]
fn server_process() {
    let Some(config) = std::env::var_os("KR_M8_TEST_CHILD") else {
        return;
    };
    let input: ServerConfig = serde_json::from_slice(&fs::read(config).unwrap()).unwrap();
    runtime()
        .block_on(crate::serve(loopback_profile(&input), &input.out))
        .unwrap();
}

struct Server {
    child: Child,
    config: ServerConfig,
    cert: Vec<u8>,
}
impl Drop for Server {
    fn drop(&mut self) {
        if self.child.try_wait().unwrap().is_none() {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}
impl Server {
    async fn launch(db: &Path, run: &str, schedule: bool) -> Self {
        let (cert, key) = kr_tunnel::launch_tls::generate_identity().unwrap();
        fs::write(db.with_file_name("cert.der"), &cert).unwrap();
        OpenOptions::new()
            .mode(0o600)
            .create(true)
            .truncate(true)
            .write(true)
            .open(db.with_file_name("private.pk8"))
            .unwrap()
            .write_all(&key)
            .unwrap();
        let tcp =
            std::array::from_fn::<_, 2, _>(|_| std::net::TcpListener::bind("127.0.0.1:0").unwrap());
        let config = ServerConfig {
            db: db.to_owned(),
            out: db.with_file_name(run),
            tls: port(),
            relay: port(),
            schedule,
            tcp: tcp
                .iter()
                .map(|socket| socket.local_addr().unwrap().port())
                .collect(),
        };
        let input = db.with_file_name("child.json");
        fs::write(&input, serde_json::to_vec(&config).unwrap()).unwrap();
        drop(tcp);
        let child = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", CHILD, "--nocapture"])
            .env("KR_M8_TEST_CHILD", input)
            .stdout(Stdio::null())
            .spawn()
            .unwrap();
        let mut server = Self {
            child,
            config,
            cert,
        };
        timeout(Duration::from_secs(8), async {
            loop {
                assert!(
                    server.child.try_wait().unwrap().is_none(),
                    "server failed to start"
                );
                if server.config.out.join("service-state.json").exists() {
                    assert_eq!(server.state()["ready"], true);
                    break;
                }
                sleep(Duration::from_millis(20)).await;
            }
        })
        .await
        .unwrap();
        server
    }
    fn state(&self) -> Value {
        serde_json::from_slice(&fs::read(self.config.out.join("service-state.json")).unwrap())
            .unwrap()
    }
    async fn stop(mut self, signal: &str) {
        assert!(
            Command::new("kill")
                .args([signal, &self.child.id().to_string()])
                .status()
                .unwrap()
                .success()
        );
        let status = timeout(Duration::from_secs(15), async {
            loop {
                if let Some(exit) = self.child.try_wait().unwrap() {
                    break exit;
                }
                sleep(Duration::from_millis(20)).await;
            }
        })
        .await
        .unwrap();
        assert_eq!(status.success(), signal == "-TERM");
        if status.success() {
            assert_eq!(self.state()["stopped"], true);
        }
    }
    async fn start_round(&self) -> (quinn::Endpoint, Vec<NativeClient>) {
        let mut endpoint = quinn::Endpoint::client("127.0.0.1:0".parse().unwrap()).unwrap();
        endpoint.set_default_client_config(prelogin::client_config(self.cert.clone()).unwrap());
        let mut clients = Vec::new();
        let mut peers = Vec::new();
        for (login, name) in [("AlphaOne", b"DBUserA"), ("BetaTwo", b"DBUserB")] {
            let connection = prelogin::connect(&endpoint, self.config.tls.into())
                .await
                .unwrap();
            let client = NativeClient::login_ports(
                connection,
                login,
                name,
                self.config.tcp[0],
                self.config.tcp[1],
            )
            .await;
            let mut offers = client.connection.accept_uni().await.unwrap();
            assert_eq!(offers.read_u8().await.unwrap(), 1);
            peers.push(connect_peer_offer(&mut offers).await);
            clients.push(client);
        }
        // Detached QUIC link handles stay live for this test until the clients
        // and the server are torn down, even after receiving the peer offers.
        for (peer, link) in peers {
            tokio::spawn(async move {
                let (_hold, stop) = watch::channel(false);
                let _ = kr_tunnel::companion::run(link, Ipv4Addr::LOCALHOST.into(), 26, stop).await;
                peer.close(0u8.into(), b"done");
            });
        }
        let created = clients[0].request(81, &create_room(), 86).await;
        let room = read_uint(&created.payload[1..3], Width::U16).unwrap();
        use Width::*;
        let join = fields(&[(room, U16), (room, U32), (0, U16), (1, U8), (5, U16)]);
        clients[1].request(82, &join, 87).await;
        for client in &mut clients {
            client.send(585, &fields(&[(0, U32)])).await;
        }
        for client in &mut clients {
            for n in 0..16 {
                let packet = receive(&mut client.recv, &mut client.rx).await;
                if packet.opcode == 117 {
                    break;
                }
                assert!(matches!(packet.opcode, 90 | 91 | 106 | 110 | 238) && n != 15);
            }
        }
        clients[1].send(97, &[0x59]).await;
        for client in &mut clients {
            waiting_slots(client, None).await;
        }
        let mut ready = fields(&[(6, U32)]);
        ready.extend([0; 6]);
        ready.extend(fields(&[(1, U8), (0, U32)]));
        clients[1].send(95, &ready).await;
        for client in &mut clients {
            client.expect(110).await;
        }
        clients[0].send(104, &[0; 20]).await;
        for client in &mut clients {
            client.expect(582).await;
            client.expect(121).await;
        }
        // Keep the peer endpoints alive through 99/153 to prove the room has
        // entered the active battle, rather than only allocating a journal row.
        let sockets = [
            UdpSocket::bind("127.0.0.1:0").unwrap(),
            UdpSocket::bind("127.0.0.1:0").unwrap(),
        ];
        for (addr, client) in sockets.into_iter().zip(&mut clients) {
            client
                .send(
                    99,
                    &fields(&[
                        (0, U32),
                        (0, U32),
                        (addr.local_addr().unwrap().port().into(), U16),
                        (0, U32),
                        (0, U16),
                        (0, U8),
                        (0, U8),
                        (0, U8),
                        (0, U32),
                    ]),
                )
                .await;
        }
        for client in &mut clients {
            for opcode in [197, 196, 153] {
                client.expect(opcode).await;
            }
        }
        (endpoint, clients)
    }
}

fn matches(db: &Path) -> Vec<(String, Option<String>)> {
    rusqlite::Connection::open(db)
        .unwrap()
        .prepare("SELECT state,reason FROM matches ORDER BY id")
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap()
}
async fn admin(db: &Path, command: LiveCommand) -> io::Result<Value> {
    let db = db.to_owned();
    tokio::task::spawn_blocking(move || crate::operations::live_command(&db, &operator(), command))
        .await
        .unwrap()
}
fn operator() -> Admin<'static> {
    Admin {
        actor: "m8-test-operator",
        reason: "local server operations acceptance",
    }
}

#[test]
fn live_writer_backup_restore_two_tls_players_and_shutdown_recovery() {
    let _serial = TEST_SERIAL.lock().unwrap();
    runtime().block_on(async {
        let source = Temp::new();
        let clean = Temp::new();
        let mut store = Store::open(&source.db()).unwrap();
        for (login, name) in [("AlphaOne", "DBUserA"), ("BetaTwo", "DBUserB")] {
            store
                .create_account(&operator(), login, name, PASSWORD, None)
                .unwrap();
        }
        let server = Server::launch(&source.db(), "first", true).await;
        let dir = source.db().with_extension("backups");
        fs::write(dir.join("manual.txt"), b"keep").unwrap();
        let floor = fs::read_dir(&dir)
            .unwrap()
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|e| e == "sqlite"))
            .max();
        let mut writer = rusqlite::Connection::open(source.db()).unwrap();
        let tx = writer
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
            .unwrap();
        tx.execute("UPDATE admin_audit SET reason='uncommitted'", [])
            .unwrap();
        let saved = timeout(Duration::from_secs(8), async {
            loop {
                let mut found = fs::read_dir(&dir)
                    .unwrap()
                    .flatten()
                    .map(|e| e.path())
                    .filter(|p| p.extension().is_some_and(|e| e == "sqlite"))
                    .collect::<Vec<_>>();
                found.sort();
                if found.len() == 2 && floor.as_ref().is_none_or(|old| &found[0] > old) {
                    break found[1].clone();
                }
                sleep(Duration::from_millis(40)).await;
            }
        })
        .await
        .unwrap();
        assert_eq!(
            crate::accounts::verify_snapshot(&saved).unwrap(),
            crate::accounts::SCHEMA_VERSION
        );
        assert_eq!(
            crate::accounts::restore(&saved, &clean.db()).unwrap(),
            crate::accounts::SCHEMA_VERSION
        );
        assert_eq!(
            crate::accounts::restore(&saved, &clean.db()),
            Err(crate::accounts::Error::Conflict)
        );
        assert!(dir.join("manual.txt").is_file());
        assert!(
            !Store::open_existing(&clean.db())
                .unwrap()
                .operator_audit(Default::default())
                .unwrap()
                .iter()
                .any(|r| r.reason == "uncommitted")
        );
        tx.rollback().unwrap();
        let (_client, _players) = timeout(Duration::from_secs(20), server.start_round())
            .await
            .unwrap();
        assert_eq!(matches(&source.db()), [("running".into(), None)]);

        server.stop("-TERM").await;
        assert_eq!(
            matches(&source.db()),
            [("cancelled".into(), Some("server_shutdown".into()))]
        );

        let source_server = Server::launch(&source.db(), "restart", false).await;
        let (_new_client, _new_players) =
            timeout(Duration::from_secs(20), source_server.start_round())
                .await
                .unwrap();
        assert_eq!(matches(&source.db()).len(), 2);
        source_server.stop("-KILL").await;
        assert_eq!(matches(&source.db())[1].0, "running");
        let after_kill = Server::launch(&source.db(), "after-kill", false).await;
        assert_eq!(
            matches(&source.db())[1],
            ("cancelled".into(), Some("server_restart".into()))
        );
        after_kill.stop("-TERM").await;

        let restored = Server::launch(&clean.db(), "restored", false).await;
        let (_restored_client, _restored_players) =
            timeout(Duration::from_secs(20), restored.start_round())
                .await
                .unwrap();
        let db = clean.db();
        let list = admin(&db, LiveCommand::ListRooms).await.unwrap();
        let target: RoomTarget = serde_json::from_value(list["rooms"][0].clone()).unwrap();
        let stale = RoomTarget {
            instance: "0".repeat(32),
            ..target.clone()
        };
        assert!(
            admin(&db, LiveCommand::CloseRoom { target: stale })
                .await
                .is_err()
        );
        assert_eq!(matches(&db)[0].0, "running");
        assert_eq!(
            admin(&db, LiveCommand::CloseRoom { target }).await.unwrap(),
            json!({"ok":true})
        );
        assert_eq!(
            admin(&db, LiveCommand::ListRooms).await.unwrap(),
            json!({"rooms":[]})
        );
        assert_eq!(
            matches(&db),
            [("cancelled".into(), Some("admin_closed".into()))]
        );
        let audit = Store::open_existing(&db)
            .unwrap()
            .operator_audit(crate::accounts::AuditFilter {
                actor: Some(operator().actor.into()),
                operation: Some("close_room".into()),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(
            audit.iter().map(|r| r.result.as_str()).collect::<Vec<_>>(),
            ["ok", "rejected"]
        );
        restored.stop("-TERM").await;
    });
}
