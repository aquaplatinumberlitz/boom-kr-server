use super::*;
use crate::account_bridge::local::native::fixtures;
use crate::accounts::{Admin, Store, TEST_SERIAL};
use kr_wire::{Width, read_uint, write_uint};

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
}
fn hex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
fn templates() -> Templates {
    Templates {
        lobby: Some(crate::account_bridge::local::native::LobbyTemplates {
            ready: vec![0x5a],
            channel_list: vec![0x5a],
        }),
        account: Arc::new(
            kr_wire::kr00967::AccountTemplate::parse(
                &hex(fixtures::AUTH),
                &hex(fixtures::OWN),
                &hex(fixtures::PLAYER),
            )
            .unwrap(),
        ),
        header: [0; 37],
        netinfo_header: [0; 37],
        flags_header: [0; 37],
        redirect_header: [0; 37],
        channel_header: [0; 37],
        netinfo: vec![0x5a],
        redirect: vec![0x5a],
        channel: vec![0x5a],
    }
}
const PASSWORD: &[u8] = b"private-test-password";
fn credentials() -> Vec<u8> {
    credentials_for(2)
}
fn credentials_for(count: u8) -> Vec<u8> {
    let mut body = write_uint(0, Width::U8).unwrap();
    body.extend(write_uint(u64::from(count), Width::U8).unwrap());
    for login in ["AlphaOne", "BetaTwo"].into_iter().take(count as usize) {
        for (bytes, key) in [(login.as_bytes(), 0x11223344), (PASSWORD, 0x44332211)] {
            body.extend(write_uint(bytes.len() as u64, Width::U16).unwrap());
            body.extend(kr_wire::crypt(bytes, key, false));
        }
    }
    body.extend([0; 16]);
    body.extend(write_uint(0, Width::U32).unwrap());
    body.extend(write_uint(0x0200007f, Width::U32).unwrap());
    body.extend(write_uint(0, Width::U8).unwrap());
    body
}
async fn send(send: &mut quinn::SendStream, sequence: &mut u32, opcode: u32, body: &[u8]) {
    send.write_all(&kr_wire::encode(&[0; 37], opcode, body, *sequence).unwrap())
        .await
        .unwrap();
    *sequence += 3;
}
async fn receive(recv: &mut quinn::RecvStream, sequence: &mut u32) -> kr_wire::Packet {
    tokio::time::timeout(Duration::from_secs(3), async {
        let mut header = [0; 37];
        recv.read_exact(&mut header).await.unwrap();
        let n = u32::from_le_bytes(header[13..17].try_into().unwrap()) as usize;
        assert!(n <= 512);
        let mut raw = header.to_vec();
        raw.resize(38 + n, 0);
        recv.read_exact(&mut raw[37..]).await.unwrap();
        let packet = kr_wire::decode(&raw, *sequence).unwrap();
        *sequence += 3;
        packet
    })
    .await
    .unwrap()
}
fn channel(auth: &[u8]) -> Vec<u8> {
    channel_for(auth, 2)
}
fn channel_for(auth: &[u8], count: u8) -> Vec<u8> {
    let mut result = write_uint(u64::from(count), Width::U8).unwrap();
    result.extend(write_uint(255, Width::U8).unwrap());
    let mut start = 13;
    for expected in [b"DBUserA".as_slice(), b"DBUserB".as_slice()]
        .into_iter()
        .take(count as usize)
    {
        let id = read_uint(&auth[start + 12..start + 16], Width::U32).unwrap();
        let n = read_uint(&auth[start + 16..start + 18], Width::U16).unwrap() as usize;
        let name = kr_wire::kr00967::decode_bytes(&auth[start + 18..start + 18 + n]).unwrap();
        assert_eq!(name, expected);
        let echo = read_uint(&auth[start + 73 + n..start + 77 + n], Width::U32).unwrap();
        assert_ne!(echo, 0);
        result.extend(write_uint(n as u64, Width::U16).unwrap());
        result.extend(kr_wire::kr00967::encode_bytes(&name).unwrap());
        result.extend(write_uint(id, Width::U32).unwrap());
        result.extend(write_uint(echo, Width::U32).unwrap());
        result.extend(write_uint(1, Width::U64).unwrap());
        result.extend(write_uint(0, Width::U8).unwrap());
        start += 98 + n;
    }
    for (value, width) in [
        (0, Width::U8),
        (1, Width::U8),
        (0, Width::U32),
        (0, Width::U32),
        (0, Width::U8),
        (0, Width::U32),
    ] {
        result.extend(write_uint(value, width).unwrap());
    }
    result
}
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        use std::os::unix::fs::DirBuilderExt;
        let mut bytes = [0; 8];
        getrandom::fill(&mut bytes).unwrap();
        let p = std::env::temp_dir().join(format!(
            "kr-prelogin-{}-{:x}",
            std::process::id(),
            u64::from_le_bytes(bytes)
        ));
        fs::DirBuilder::new().mode(0o700).create(&p).unwrap();
        Self(p)
    }
    fn db(&self) -> PathBuf {
        self.0.join("accounts.sqlite")
    }
    fn active(&self) -> i64 {
        rusqlite::Connection::open(self.db())
            .unwrap()
            .query_row(
                "SELECT count(*) FROM sessions WHERE revoked_at IS NULL",
                [],
                |r| r.get(0),
            )
            .unwrap()
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn resources(client: Client) -> Arc<Resources> {
    let (events, _) = mpsc::channel(8);
    Arc::new(Resources {
        owner: Arc::new(AsyncMutex::new(None)),
        client,
        templates: templates(),
        hello: vec![0; 15],
        wcp: vec![1; 15],
        ports: vec![3838, 3938],
        bytes: AtomicUsize::new(0),
        events,
        hub: None,
        lobby: AsyncMutex::new(None),
        persist_character: false,
        config: Config {
            bind: "127.0.0.1:50000".parse().unwrap(),
            certificate: FixturePin {
                path: PathBuf::new(),
                bytes: 0,
                sha256: String::new(),
            },
            private_key: PathBuf::new(),
            max_connections: 4,
            max_unauthenticated: 2,
            max_per_ip: 2,
            prelogin_seconds: 10,
            session_ttl_seconds: 60,
            session_idle_seconds: 30,
        },
    })
}

#[test]
fn tls_group_handoff_cannot_be_claimed_by_a_second_connection_and_revoke_closes_every_stream() {
    let _serial = TEST_SERIAL.lock().unwrap();
    let temp = Temp::new();
    let admin = Admin {
        actor: "test",
        reason: "prelogin TLS real DB controls",
    };
    let mut store = Store::open(&temp.db()).unwrap();
    store.begin_instance(&admin, 100).unwrap();
    let a = store
        .create_account(&admin, "AlphaOne", "DBUserA", PASSWORD, 100)
        .unwrap();
    let b = store
        .create_account(&admin, "BetaTwo", "DBUserB", PASSWORD, 100)
        .unwrap();
    runtime().block_on(async {
        let worker = crate::account_worker::Worker::open(temp.db(), 8, Duration::from_secs(5))
            .await
            .unwrap();
        let (cert, key) = kr_tunnel::launch_tls::generate_identity().unwrap();
        let server = quinn::Endpoint::server(
            prelogin::server_config(cert.clone(), key.to_vec()).unwrap(),
            "127.0.0.1:0".parse().unwrap(),
        )
        .unwrap();
        let mut client = quinn::Endpoint::client("127.0.0.1:0".parse().unwrap()).unwrap();
        client.set_default_client_config(prelogin::client_config(cert).unwrap());
        let endpoint = server.clone();
        let mut authority = resources(worker.client());
        let hub = crate::room_service::Hub::new(
            &crate::room_service::Config {
                max_rooms: 8,
                round_profile: None,
                player_choices: false,
                capacity: None,
                player_round: None,
            },
            crate::room_io::registry_foundation_tests::empty(0x77),
            None,
            None,
        )
        .unwrap();
        Arc::get_mut(&mut authority).unwrap().hub = Some(hub.clone());
        let observed = authority.clone();
        let foreign = resources(worker.client());
        let (halt, stop) = watch::channel(false);
        let gate = Arc::new(Semaphore::new(2));
        let gate2 = gate.clone();
        let stop2 = stop.clone();
        let task = tokio::spawn(async move {
            let tls = endpoint.accept().await.unwrap().await.unwrap();
            connection(tls, authority, stop, gate.acquire_owned().await.unwrap()).await
        });
        let conn = prelogin::connect(&client, server.local_addr().unwrap())
            .await
            .unwrap();
        let (mut login, mut reply) = prelogin::open(&conn, 3838).await.unwrap();
        let mut hello = [0; 15];
        reply.read_exact(&mut hello).await.unwrap();
        let mut tx = 40;
        let mut rx = 43;
        send(&mut login, &mut tx, 1, &credentials()).await;
        let auth = receive(&mut reply, &mut rx).await;
        assert_eq!(auth.opcode, 4);
        assert_eq!(temp.active(), 2);
        let claims = channel(&auth.payload);
        send(&mut login, &mut tx, 9, &[0; 8]).await;
        assert_eq!(receive(&mut reply, &mut rx).await.opcode, 15);
        assert_eq!(receive(&mut reply, &mut rx).await.opcode, 62);
        assert_eq!(receive(&mut reply, &mut rx).await.opcode, 62);
        send(&mut login, &mut tx, 5, &[0; 9]).await;
        assert_eq!(receive(&mut reply, &mut rx).await.opcode, 7);
        login.finish().unwrap();
        let endpoint = server.clone();
        let intruder = tokio::spawn(async move {
            let tls = endpoint.accept().await.unwrap().await.unwrap();
            connection(tls, foreign, stop2, gate2.acquire_owned().await.unwrap()).await
        });
        let another = prelogin::connect(&client, server.local_addr().unwrap())
            .await
            .unwrap();
        let (mut forged, mut answer) = prelogin::open(&another, 3938).await.unwrap();
        answer.read_exact(&mut hello).await.unwrap();
        send(&mut forged, &mut 40, 6, &claims).await;
        tokio::time::timeout(Duration::from_secs(3), another.closed())
            .await
            .unwrap();
        assert_eq!(temp.active(), 2);
        intruder.await.unwrap().unwrap_err();
        let (mut channel, mut reply) = prelogin::open(&conn, 3938).await.unwrap();
        reply.read_exact(&mut hello).await.unwrap();
        send(&mut channel, &mut 40, 6, &claims).await;
        let mut channel_rx = 43;
        assert_eq!(receive(&mut reply, &mut channel_rx).await.opcode, 17);
        send(
            &mut channel,
            &mut 43,
            67,
            &write_uint(0, Width::U16).unwrap(),
        )
        .await;
        for opcode in [241, 91, 14] {
            assert_eq!(receive(&mut reply, &mut channel_rx).await.opcode, opcode);
        }
        let own = receive(&mut reply, &mut channel_rx).await;
        assert_eq!(own.opcode, 291);
        let expected = kr_wire::kr00967::local_own291(&[b"DBUserA", b"DBUserB"]).unwrap();
        assert_eq!(own.payload, expected);
        assert!(observed.lobby.lock().await.as_ref().unwrap().current());
        assert!(hub.register(a.wire_id, 77).is_err());
        assert!(hub.register(b.wire_id, 77).is_err());
        let handles = observed
            .owner
            .lock()
            .await
            .as_ref()
            .unwrap()
            .authorities()
            .to_vec();
        assert_eq!(handles[0].player(), &a);
        assert_eq!(handles[1].player(), &b);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        store.revoke(&admin, b.account_id, now).unwrap();
        tokio::time::timeout(Duration::from_secs(3), conn.closed())
            .await
            .unwrap();
        let _ = task.await.unwrap();
        assert!(observed.lobby.lock().await.is_none());
        drop(hub.register(a.wire_id, 77).unwrap());
        drop(hub.register(b.wire_id, 77).unwrap());
        assert!(handles.iter().all(|h| !h.current()));
        assert_eq!(temp.active(), 0);
        halt.send_replace(true);
        server.close(0u8.into(), b"done");
        client.close(0u8.into(), b"done");
        worker.shutdown().await.unwrap();
    });
}

#[test]
fn tls_single_account_can_select_return_and_select_fresh_room_then_revoke_cleans_membership() {
    let _serial = TEST_SERIAL.lock().unwrap();
    let temp = Temp::new();
    let admin = Admin {
        actor: "test",
        reason: "native TLS managed room ownership",
    };
    let mut store = Store::open(&temp.db()).unwrap();
    store.begin_instance(&admin, 100).unwrap();
    let player = store
        .create_account(&admin, "AlphaOne", "DBUserA", PASSWORD, 100)
        .unwrap();
    runtime().block_on(async {
        let worker = crate::account_worker::Worker::open(temp.db(), 8, Duration::from_secs(5))
            .await
            .unwrap();
        let hub = crate::room_service::Hub::new(
            &crate::room_service::Config {
                max_rooms: 8,
                round_profile: None,
                player_choices: false,
                capacity: None,
                player_round: None,
            },
            crate::room_io::registry_foundation_tests::selection_prototype(),
            None,
            None,
        )
        .unwrap();
        let mut authority = resources(worker.client());
        Arc::get_mut(&mut authority).unwrap().hub = Some(hub.clone());
        let observed = authority.clone();
        let (cert, key) = kr_tunnel::launch_tls::generate_identity().unwrap();
        let server = quinn::Endpoint::server(
            prelogin::server_config(cert.clone(), key.to_vec()).unwrap(),
            "127.0.0.1:0".parse().unwrap(),
        )
        .unwrap();
        let mut client = quinn::Endpoint::client("127.0.0.1:0".parse().unwrap()).unwrap();
        client.set_default_client_config(prelogin::client_config(cert).unwrap());
        let endpoint = server.clone();
        let (_halt, stop) = watch::channel(false);
        let permit = Arc::new(Semaphore::new(1)).acquire_owned().await.unwrap();
        let task = tokio::spawn(async move {
            let tls = endpoint.accept().await.unwrap().await.unwrap();
            connection(tls, authority, stop, permit).await
        });
        let conn = prelogin::connect(&client, server.local_addr().unwrap())
            .await
            .unwrap();
        let (mut login, mut replies) = prelogin::open(&conn, 3838).await.unwrap();
        let mut hello = [0; 15];
        replies.read_exact(&mut hello).await.unwrap();
        let (mut tx, mut rx) = (40, 43);
        send(&mut login, &mut tx, 1, &credentials_for(1)).await;
        let auth = receive(&mut replies, &mut rx).await;
        let claims = channel_for(&auth.payload, 1);
        send(&mut login, &mut tx, 9, &[0; 8]).await;
        for opcode in [15, 62] {
            assert_eq!(receive(&mut replies, &mut rx).await.opcode, opcode);
        }
        send(&mut login, &mut tx, 5, &[0; 9]).await;
        assert_eq!(receive(&mut replies, &mut rx).await.opcode, 7);
        login.finish().unwrap();
        let (mut channel, mut replies) = prelogin::open(&conn, 3938).await.unwrap();
        replies.read_exact(&mut hello).await.unwrap();
        let (mut tx, mut rx) = (40, 43);
        send(&mut channel, &mut tx, 6, &claims).await;
        assert_eq!(receive(&mut replies, &mut rx).await.opcode, 17);
        send(
            &mut channel,
            &mut tx,
            67,
            &write_uint(0, Width::U16).unwrap(),
        )
        .await;
        for opcode in [241, 91, 14, 291] {
            assert_eq!(receive(&mut replies, &mut rx).await.opcode, opcode);
        }
        let title = b"OwnedRoom";
        let mut create = write_uint(title.len() as u64, Width::U16).unwrap();
        create.extend(kr_wire::kr00967::encode_bytes(title).unwrap());
        create.extend(write_uint(0, Width::U16).unwrap());
        for value in [0, 0, 5, 8191, 0, 0, 0, u32::MAX, 0, 0, 0, 0] {
            create.extend(write_uint(u64::from(value), Width::U32).unwrap());
        }
        for _ in 0..4 {
            create.extend(write_uint(0, Width::U8).unwrap());
        }
        for room in 1..=2 {
            send(&mut channel, &mut tx, 81, &create).await;
            let reply = receive(&mut replies, &mut rx).await;
            assert_eq!(reply.opcode, 86);
            assert_eq!(read_uint(&reply.payload[1..3], Width::U16).unwrap(), room);
            assert!(hub.register(player.wire_id, 99).is_err());
            send(&mut channel, &mut tx, 86, &[]).await;
            assert_eq!(receive(&mut replies, &mut rx).await.opcode, 241);
            let catalog = receive(&mut replies, &mut rx).await;
            assert_eq!(catalog.opcode, 91);
            assert!(
                kr_wire::kr00967::listing91_parse_rows(&catalog.payload)
                    .unwrap()
                    .is_empty()
            );
            assert_eq!(temp.active(), 1);
        }
        // Keep a third room owned when the DB session is revoked: dropping the
        // TLS owner must remove both room membership and account lobby owner.
        send(&mut channel, &mut tx, 81, &create).await;
        assert_eq!(receive(&mut replies, &mut rx).await.opcode, 86);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        store.revoke(&admin, player.account_id, now).unwrap();
        tokio::time::timeout(Duration::from_secs(3), conn.closed())
            .await
            .unwrap();
        let _ = task.await.unwrap();
        assert!(observed.lobby.lock().await.is_none());
        let mut replacement = hub.register(player.wire_id, 99).unwrap();
        let catalog = replacement
            .catalog(&std::collections::BTreeSet::new())
            .unwrap();
        assert!(
            kr_wire::kr00967::listing91_parse_rows(&catalog.packets.last().unwrap().1)
                .unwrap()
                .is_empty()
        );
        drop(replacement);
        assert_eq!(temp.active(), 0);
        client.close(0u8.into(), b"done");
        server.close(0u8.into(), b"done");
        worker.shutdown().await.unwrap();
    });
}
#[test]
fn slow_or_corrupt_native_frames_and_unauthenticated_lifetime_issue_no_db_session() {
    let _serial = TEST_SERIAL.lock().unwrap();
    let temp = Temp::new();
    let mut store = Store::open(&temp.db()).unwrap();
    store
        .begin_instance(
            &Admin {
                actor: "test",
                reason: "prelogin abuse controls",
            },
            100,
        )
        .unwrap();
    runtime().block_on(async {
        let worker = crate::account_worker::Worker::open(temp.db(), 4, Duration::from_secs(3))
            .await
            .unwrap();
        for case in ["slow", "corrupt", "idle"] {
            let (cert, key) = kr_tunnel::launch_tls::generate_identity().unwrap();
            let server = quinn::Endpoint::server(
                prelogin::server_config(cert.clone(), key.to_vec()).unwrap(),
                "127.0.0.1:0".parse().unwrap(),
            )
            .unwrap();
            let mut client = quinn::Endpoint::client("127.0.0.1:0".parse().unwrap()).unwrap();
            client.set_default_client_config(prelogin::client_config(cert).unwrap());
            let resource = resources(worker.client());
            let endpoint = server.clone();
            let (_halt, stop) = watch::channel(false);
            let permit = Arc::new(Semaphore::new(1)).acquire_owned().await.unwrap();
            let task = tokio::spawn(async move {
                let tls = endpoint.accept().await.unwrap().await.unwrap();
                connection(tls, resource, stop, permit).await
            });
            let conn = prelogin::connect(&client, server.local_addr().unwrap())
                .await
                .unwrap();
            let mut held = None;
            if case != "idle" {
                let (mut send, mut recv) = prelogin::open(&conn, 3838).await.unwrap();
                let mut hello = [0; 15];
                recv.read_exact(&mut hello).await.unwrap();
                let mut frame = kr_wire::encode(&[0; 37], 1, &credentials(), 40).unwrap();
                if case == "slow" {
                    send.write_all(&frame[..38]).await.unwrap();
                } else {
                    *frame.last_mut().unwrap() ^= 1;
                    send.write_all(&frame).await.unwrap();
                }
                held = Some((send, recv));
            }
            tokio::time::timeout(Duration::from_secs(12), conn.closed())
                .await
                .unwrap();
            assert!(task.await.unwrap().is_err());
            assert_eq!(temp.active(), 0);
            drop(held);
            server.close(0u8.into(), b"done");
            client.close(0u8.into(), b"done");
        }
        worker.shutdown().await.unwrap();
    });
}

#[test]
fn peer_admission_is_bounded_and_releases_active_slots_without_resetting_rate() {
    let shared = Arc::new(Mutex::new(Admission::default()));
    let now = Instant::now();
    let ip = "127.0.0.1".parse().unwrap();
    let one = Admission::admit(&shared, ip, 1, now).unwrap();
    assert!(Admission::admit(&shared, ip, 1, now).is_err());
    drop(one);
    for _ in 0..4 {
        drop(Admission::admit(&shared, ip, 1, now).unwrap());
    }
    assert!(Admission::admit(&shared, ip, 1, now).is_err());
    drop(Admission::admit(&shared, ip, 1, now + Duration::from_secs(60)).unwrap());
    let mut retained = Vec::new();
    for n in 1..=255 {
        retained.push(
            Admission::admit(
                &shared,
                IpAddr::V4(Ipv4Addr::new(10, 0, 0, n)),
                1,
                now + Duration::from_secs(60),
            )
            .unwrap(),
        );
    }
    assert!(
        Admission::admit(
            &shared,
            "10.1.0.1".parse().unwrap(),
            1,
            now + Duration::from_secs(60)
        )
        .is_err()
    );
    drop(retained);
    assert!(
        Admission::admit(
            &shared,
            "10.1.0.1".parse().unwrap(),
            1,
            now + Duration::from_secs(120)
        )
        .is_ok()
    );
}
