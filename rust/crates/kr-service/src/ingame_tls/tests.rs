use super::*;
use crate::account_bridge::local::native::fixtures;
use crate::accounts::{Admin, Store, TEST_SERIAL};
use kr_wire::{Width, read_uint, write_uint};
use tokio::io::AsyncReadExt;

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
    credentials_named(&["AlphaOne", "BetaTwo"][..count as usize])
}
fn credentials_named(names: &[&str]) -> Vec<u8> {
    let mut body = write_uint(0, Width::U8).unwrap();
    body.extend(write_uint(names.len() as u64, Width::U8).unwrap());
    for login in names {
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
    receive_for(recv, sequence, Duration::from_secs(3)).await
}
async fn receive_for(
    recv: &mut quinn::RecvStream,
    sequence: &mut u32,
    wait: Duration,
) -> kr_wire::Packet {
    tokio::time::timeout(wait, async {
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
    channel_named(
        auth,
        &[b"DBUserA".as_slice(), b"DBUserB".as_slice()][..count as usize],
    )
}
fn channel_named(auth: &[u8], names: &[&[u8]]) -> Vec<u8> {
    let mut result = write_uint(names.len() as u64, Width::U8).unwrap();
    result.extend(write_uint(255, Width::U8).unwrap());
    let mut start = 13;
    for expected in names {
        let id = read_uint(&auth[start + 12..start + 16], Width::U32).unwrap();
        let n = read_uint(&auth[start + 16..start + 18], Width::U16).unwrap() as usize;
        let name = kr_wire::kr00967::decode_bytes(&auth[start + 18..start + 18 + n]).unwrap();
        assert_eq!(&name, expected);
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
fn create_room() -> Vec<u8> {
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
    create
}

struct Temp(PathBuf);

struct NativeClient {
    connection: quinn::Connection,
    send: quinn::SendStream,
    recv: quinn::RecvStream,
    tx: u32,
    rx: u32,
}
impl NativeClient {
    async fn login(connection: quinn::Connection, login: &str, name: &[u8]) -> Self {
        let (mut outgoing, mut incoming) = prelogin::open(&connection, 3838).await.unwrap();
        let mut hello = [0; 15];
        incoming.read_exact(&mut hello).await.unwrap();
        let (mut tx, mut rx) = (40, 43);
        send(&mut outgoing, &mut tx, 1, &credentials_named(&[login])).await;
        let auth = receive(&mut incoming, &mut rx).await;
        assert_eq!(auth.opcode, 4);
        let claims = channel_named(&auth.payload, &[name]);
        send(&mut outgoing, &mut tx, 9, &[0; 8]).await;
        for opcode in [15, 62] {
            assert_eq!(receive(&mut incoming, &mut rx).await.opcode, opcode);
        }
        send(&mut outgoing, &mut tx, 5, &[0; 9]).await;
        assert_eq!(receive(&mut incoming, &mut rx).await.opcode, 7);
        outgoing.finish().unwrap();
        let (mut outgoing, mut incoming) = prelogin::open(&connection, 3938).await.unwrap();
        incoming.read_exact(&mut hello).await.unwrap();
        let (mut tx, mut rx) = (40, 43);
        send(&mut outgoing, &mut tx, 6, &claims).await;
        assert_eq!(receive(&mut incoming, &mut rx).await.opcode, 17);
        send(
            &mut outgoing,
            &mut tx,
            67,
            &write_uint(0, Width::U16).unwrap(),
        )
        .await;
        for opcode in [241, 91, 14, 291] {
            assert_eq!(receive(&mut incoming, &mut rx).await.opcode, opcode);
        }
        Self {
            connection,
            send: outgoing,
            recv: incoming,
            tx,
            rx,
        }
    }
    async fn request(&mut self, opcode: u32, body: &[u8], expected: u32) -> kr_wire::Packet {
        send(&mut self.send, &mut self.tx, opcode, body).await;
        for _ in 0..8 {
            let packet = receive(&mut self.recv, &mut self.rx).await;
            if packet.opcode == expected {
                return packet;
            }
            assert!(matches!(packet.opcode, 90 | 91));
        }
        panic!("Expected reply absent");
    }
    async fn send(&mut self, opcode: u32, body: &[u8]) {
        send(&mut self.send, &mut self.tx, opcode, body).await;
    }
    async fn expect(&mut self, opcode: u32) -> kr_wire::Packet {
        let packet = receive_for(&mut self.recv, &mut self.rx, Duration::from_secs(12)).await;
        assert_eq!(packet.opcode, opcode);
        packet
    }
}

async fn connect_peer_offer(offers: &mut quinn::RecvStream) -> (quinn::Endpoint, kr_tunnel::Link) {
    let mut ip = [0; 4];
    offers.read_exact(&mut ip).await.unwrap();
    let address = SocketAddrV4::new(ip.into(), offers.read_u16().await.unwrap());
    let size = offers.read_u16().await.unwrap() as usize;
    assert!(size <= 4096);
    let mut certificate = vec![0; size];
    offers.read_exact(&mut certificate).await.unwrap();
    let mut secret = zeroize::Zeroizing::new([0; 32]);
    offers.read_exact(secret.as_mut()).await.unwrap();
    let endpoint = kr_tunnel::client("127.0.0.1:0".parse().unwrap(), certificate.into()).unwrap();
    let link = kr_tunnel::Link::connect(
        &endpoint,
        address.into(),
        &kr_tunnel::Secret::from_bytes(*secret),
    )
    .await
    .unwrap();
    (endpoint, link)
}

fn fields(items: &[(u64, Width)]) -> Vec<u8> {
    items
        .iter()
        .flat_map(|&(value, width)| write_uint(value, width).unwrap())
        .collect()
}

fn report(slot: u8, subtype: u8, tick: u32) -> Vec<u8> {
    use Width::*;
    let mut body = fields(&[
        (subtype.into(), U8),
        (slot.into(), U8),
        (120, U16),
        (159, U16),
        (tick.into(), U32),
        (0, U32),
    ]);
    if matches!(subtype, 17 | 19) {
        let size = if subtype == 17 { 14 } else { 11 };
        let mut blob = vec![0; size];
        blob[0] = 24;
        blob[1..5].copy_from_slice(&((size - 5) as u32).to_le_bytes());
        if subtype == 17 {
            blob[10..14].copy_from_slice(&7u32.to_le_bytes());
        }
        body.extend(fields(&[(size as u64, U32)]));
        body.extend(kr_wire::kr00967::encode_bytes(&blob).unwrap());
    }
    body
}

fn bomb(key: u64, column: u8) -> Vec<u8> {
    use Width::*;
    fields(&[
        (key, U32),
        (0, U8),
        (0, U8),
        (1, U32),
        (3, U8),
        (column.into(), U8),
        (1, U8),
        (0, U8),
        (0, U8),
        (0, U8),
        (1, U8),
        (1, U32),
        (0, U8),
    ])
}

#[test]
fn two_tls_owners_have_distinct_connection_keys_and_finish_two_owned_rounds() {
    let _serial = TEST_SERIAL.lock().unwrap();
    let temp = Temp::new();
    let admin = Admin {
        actor: "test",
        reason: "distinct native streams across TLS owners",
    };
    let mut store = Store::open(&temp.db()).unwrap();
    let epoch = store.begin_instance(&admin, 100).unwrap();
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
        let held = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
        let std::net::SocketAddr::V4(address) = held.local_addr().unwrap() else {
            unreachable!()
        };
        drop(held);
        let relay = crate::authenticated_peer::Hub::new(&crate::authenticated_peer::Config {
            bind: address,
            advertise: address,
            max_peers: 4,
            route_capacity: 4,
            queue_messages: 4,
            send_budget: 4,
            pending_ms: 500,
            lease_seconds: 120,
        })
        .unwrap();
        let peer_endpoint = relay.bind().unwrap();
        let bridge =
            crate::account_bridge::Bridge::new(worker.client(), 4, Duration::from_secs(30))
                .unwrap();
        let (peer_halt, peer_stop) = watch::channel(false);
        let peer_task = tokio::spawn(relay.clone().run(peer_endpoint, bridge, peer_stop));
        let hub = crate::room_service::Hub::new(
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
            Some(relay.clone()),
        )
        .unwrap();
        let journal = crate::match_journal::Manager::open(worker.client(), epoch);
        hub.install_journal(journal.factory()).unwrap();
        let (cert, key) = kr_tunnel::launch_tls::generate_identity().unwrap();
        let server = quinn::Endpoint::server(
            prelogin::server_config(cert.clone(), key.to_vec()).unwrap(),
            "127.0.0.1:0".parse().unwrap(),
        )
        .unwrap();
        let mut client = quinn::Endpoint::client("127.0.0.1:0".parse().unwrap()).unwrap();
        client.set_default_client_config(prelogin::client_config(cert).unwrap());
        let mut clients = Vec::new();
        let mut tasks = Vec::new();
        let mut halts = Vec::new();
        let mut observed = Vec::new();
        let (proxy_halt, proxy_stop) = watch::channel(false);
        let mut proxies = Vec::new();
        let mut native = Vec::new();
        for (login, name) in [("AlphaOne", b"DBUserA"), ("BetaTwo", b"DBUserB")] {
            let mut authority = resources(worker.client());
            let settings = Arc::get_mut(&mut authority).unwrap();
            settings.hub = Some(hub.clone());
            settings.authenticated = Some(relay.clone());
            settings.config.session_ttl_seconds = 120;
            settings.config.session_idle_seconds = 90;
            observed.push(authority.clone());
            let endpoint = server.clone();
            let (halt, stop) = watch::channel(false);
            halts.push(halt);
            let permit = Arc::new(Semaphore::new(1)).acquire_owned().await.unwrap();
            tasks.push(tokio::spawn(async move {
                let tls = endpoint.accept().await.unwrap().await.unwrap();
                connection(tls, authority, stop, permit).await
            }));
            let connection = prelogin::connect(&client, server.local_addr().unwrap())
                .await
                .unwrap();
            let attached = NativeClient::login(connection, login, name).await;
            let mut offers = tokio::time::timeout(Duration::from_secs(3), attached.connection.accept_uni())
                .await
                .unwrap()
                .unwrap();
            assert_eq!(offers.read_u8().await.unwrap(), 1);
            let (endpoint, link) = connect_peer_offer(&mut offers).await;
            assert_eq!(offers.read(&mut [0; 1]).await.unwrap(), None);
            let stop = proxy_stop.clone();
            proxies.push(tokio::spawn(async move {
                let result = kr_tunnel::companion::run(link, Ipv4Addr::LOCALHOST.into(), 26, stop).await;
                endpoint.close(0u8.into(), b"done");
                result
            }));
            clients.push(attached);
            native.push(tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap());
        }
        let mut keys = Vec::new();
        for authority in &observed {
            keys.push(authority.owner.lock().await.as_ref().unwrap().authorities()[0].connection());
        }
        assert_ne!(keys[0], keys[1]);
        // Raw legacy TCP uses this same allocator. It cannot borrow either
        // authenticated native channel key even while both TLS owners live.
        let tcp_key = crate::transport_ids::next().unwrap();
        assert!(!keys.contains(&tcp_key));
        let created = clients[0].request(81, &create_room(), 86).await;
        let room = read_uint(&created.payload[1..3], Width::U16).unwrap();
        let join: Vec<_> = [
            (room, Width::U16),
            (room, Width::U32),
            (0, Width::U16),
            (1, Width::U8),
            (5, Width::U16),
        ]
        .into_iter()
        .flat_map(|(value, width)| write_uint(value, width).unwrap())
        .collect();
        let joined = clients[1].request(82, &join, 87).await;
        assert_eq!(read_uint(&joined.payload[1..2], Width::U8).unwrap(), 1);
        assert_eq!(temp.active(), 2);
        assert!(hub.register(a.wire_id, tcp_key).is_err());
        assert!(hub.register(b.wire_id, tcp_key).is_err());
        for client in &mut clients {
            client.send(585, &fields(&[(0, Width::U32)])).await;
        }
        for client in &mut clients {
            // The room already exists when B enters; skip only catalog frames
            // before setup, and require exactly two projected DB profiles.
            let mut ids = Vec::new();
            loop {
                let packet =
                    receive_for(&mut client.recv, &mut client.rx, Duration::from_secs(8)).await;
                match packet.opcode {
                    106 => ids.push(read_uint(&packet.payload[1..5], Width::U32).unwrap()),
                    117 => break,
                    90 | 91 | 110 | 238 => {}
                    opcode => panic!("unexpected setup opcode {opcode}"),
                }
            }
            ids.sort_unstable();
            assert_eq!(ids, [u64::from(a.wire_id), u64::from(b.wire_id)]);
        }
        clients[1].send(97, &[0x59]).await;
        for client in &mut clients {
            client.expect(110).await;
            client.expect(110).await;
        }
        let mut last_key = None;
        let mut last_routes = None;
        let mut played = Vec::new();
        for round in 1..=2 {
            let mut ready = fields(&[(6, Width::U32)]);
            ready.extend([0; 6]);
            ready.extend(fields(&[(1, Width::U8), (0, Width::U32)]));
            clients[1].send(95, &ready).await;
            for client in &mut clients {
                client.expect(110).await;
            }
            clients[0].send(104, &[0; 20]).await;
            for client in &mut clients {
                client.expect(582).await;
                client.expect(121).await;
            }
            for (slot, client) in clients.iter_mut().enumerate() {
                use Width::*;
                let load = fields(&[
                    (0, U32),
                    (0, U32),
                    (native[slot].local_addr().unwrap().port().into(), U16),
                    (0, U32),
                    (0, U16),
                    (0, U8),
                    (0, U8),
                    (0, U8),
                    (0, U32),
                ]);
                client.send(99, &load).await;
            }
            let mut wire_keys = Vec::new();
            let mut routes = Vec::new();
            for (slot, client) in clients.iter_mut().enumerate() {
                client.expect(197).await;
                client.expect(196).await;
                let packet = client.expect(153).await;
                wire_keys.push(read_uint(&packet.payload[15..19], Width::U32).unwrap());
                // The other occupied row follows the 35-byte header. Each
                // occupied row has a presence byte and three six-byte endpoints.
                let at = 36 + 19 * (1 - slot);
                assert_eq!(
                    read_uint(&packet.payload[at..at + 4], Width::U32).unwrap(),
                    u64::from(u32::from(Ipv4Addr::LOCALHOST))
                );
                let port = read_uint(&packet.payload[at + 4..at + 6], Width::U16).unwrap() as u16;
                assert_ne!(port, native[1 - slot].local_addr().unwrap().port());
                routes.push(SocketAddrV4::new(Ipv4Addr::LOCALHOST, port));
            }
            assert_eq!(wire_keys[0], wire_keys[1]);
            assert_ne!(Some(wire_keys[0]), last_key);
            if let Some(old) = last_key {
                clients[0].send(118, &bomb(old, 4)).await;
            }
            last_key = Some(wire_keys[0]);
            played.push(wire_keys[0]);
            if let Some(old) = last_routes.replace(routes.clone()) {
                assert_ne!(old, routes);
                native[0].send_to(b"previous round", old[0]).await.unwrap();
            }
            for (slot, socket) in native.iter().enumerate() {
                socket.send_to(&[round as u8, slot as u8], routes[slot]).await.unwrap();
            }
            for (slot, socket) in native.iter().enumerate() {
                let mut data = [0; 32];
                let (size, source) = tokio::time::timeout(Duration::from_secs(3), socket.recv_from(&mut data))
                    .await.unwrap().unwrap();
                assert_eq!(source, routes[slot].into());
                assert_eq!(&data[..size], &[round as u8, (1 - slot) as u8]);
            }
            if round == 2 {
                let mut unexpected = [0; 32];
                assert!(
                    tokio::time::timeout(
                        Duration::from_millis(350),
                        native[1].recv_from(&mut unexpected)
                    )
                    .await
                    .is_err(),
                    "the previous round's UDP port must not forward a late datagram"
                );
            }
            clients[0].send(118, &bomb(wire_keys[0], 3)).await;
            for client in &mut clients {
                let spawn = client.expect(159).await;
                assert_eq!(read_uint(&spawn.payload[17..18], Width::U8).unwrap(), 3);
                assert_eq!(
                    read_uint(&spawn.payload[24..26], Width::U16).unwrap(),
                    round as u64
                );
            }
            for client in &mut clients {
                client.expect(158).await;
            }
            let tick = round as u32 * 100;
            clients[1].send(127, &report(1, 17, tick)).await;
            clients[0].expect(170).await;
            clients[1].send(127, &report(1, 19, tick + 1)).await;
            clients[0].expect(170).await;
            clients[0].send(127, &report(0, 21, tick + 2)).await;
            for client in &mut clients {
                client.expect(175).await;
                client.expect(180).await;
                client.expect(110).await;
                client.expect(110).await;
            }
            let db = rusqlite::Connection::open(temp.db()).unwrap();
            let rows = db
                .prepare(
                    "SELECT m.round, m.state, m.reason, p.slot, p.account_id, p.player_id, p.outcome, p.exit_kind
                     FROM matches m JOIN match_players p ON p.match_id = m.id
                     ORDER BY m.round, p.slot",
                )
                .unwrap()
                .query_map([], |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, i64>(3)?,
                        row.get::<_, i64>(4)?,
                        row.get::<_, i64>(5)?,
                        row.get::<_, String>(6)?,
                        row.get::<_, String>(7)?,
                    ))
                })
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            assert_eq!(rows.len(), 2 * round as usize);
            for (n, row) in rows.iter().enumerate() {
                let slot = n % 2;
                let (player, outcome) = if slot == 0 { (&a, "win") } else { (&b, "loss") };
                assert_eq!(
                    u64::from(crate::room::wire_key(row.0.try_into().unwrap()).unwrap()),
                    played[n / 2]
                );
                assert_eq!((&*row.1, &*row.2), ("complete", "elimination"));
                assert_eq!((row.3, row.4, row.5), (slot as i64, player.account_id, player.player_id));
                assert_eq!((&*row.6, &*row.7), (outcome, "present"));
            }
        }
        proxy_halt.send_replace(true);
        for proxy in proxies {
            proxy.await.unwrap().unwrap();
        }
        for client in &clients {
            client.connection.close(0u8.into(), b"test EOF");
        }
        for task in tasks {
            let _ = task.await.unwrap();
        }
        assert_eq!(temp.active(), 0);
        let mut replacement = hub.register(a.wire_id, tcp_key).unwrap();
        let catalog = replacement
            .catalog(&std::collections::BTreeSet::new())
            .unwrap();
        assert!(
            kr_wire::kr00967::listing91_parse_rows(&catalog.packets.last().unwrap().1)
                .unwrap()
                .is_empty()
        );
        drop(replacement);
        peer_halt.send_replace(true);
        peer_task.await.unwrap().unwrap();
        journal.shutdown().await.unwrap();
        server.close(0u8.into(), b"done");
        client.close(0u8.into(), b"done");
        worker.shutdown().await.unwrap();
    });
}
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
        bytes: Mutex::new(prelogin::TrafficBudget::new(
            prelogin::BYTE_LIMIT,
            64 * 1024,
        )),
        events,
        hub: None,
        lobby: AsyncMutex::new(None),
        persist_character: false,
        authenticated: None,
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
fn unvalidated_initial_gets_retry_before_any_admission() {
    runtime().block_on(async {
        let (cert, key) = kr_tunnel::launch_tls::generate_identity().unwrap();
        let server = quinn::Endpoint::server(
            prelogin::server_config(cert.clone(), key.to_vec()).unwrap(),
            "127.0.0.1:0".parse().unwrap(),
        )
        .unwrap();
        let mut client = quinn::Endpoint::client("127.0.0.1:0".parse().unwrap()).unwrap();
        client.set_default_client_config(prelogin::client_config(cert).unwrap());
        let address = server.local_addr().unwrap();
        let dial = tokio::spawn(async move { prelogin::connect(&client, address).await });
        let first = server.accept().await.unwrap();
        assert!(!first.remote_address_validated());
        assert!(validated(first).is_none());
        let second = server.accept().await.unwrap();
        let tls = validated(second).expect("Retry token validates the source");
        let _accepted = tls.await.unwrap();
        dial.await.unwrap().unwrap();
    });
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
    let epoch = store.begin_instance(&admin, 100).unwrap();
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
        let held = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
        let std::net::SocketAddr::V4(address) = held.local_addr().unwrap() else {
            unreachable!()
        };
        drop(held);
        let relay = crate::authenticated_peer::Hub::new(&crate::authenticated_peer::Config {
            bind: address,
            advertise: address,
            max_peers: 4,
            route_capacity: 4,
            queue_messages: 4,
            send_budget: 4,
            pending_ms: 500,
            lease_seconds: 60,
        })
        .unwrap();
        let hub = crate::room_service::Hub::new(
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
            Some(relay.clone()),
        )
        .unwrap();
        let journal = crate::match_journal::Manager::open(worker.client(), epoch);
        hub.install_journal(journal.factory()).unwrap();
        let config = Arc::get_mut(&mut authority).unwrap();
        config.hub = Some(hub.clone());
        config.authenticated = Some(relay.clone());
        config.config.session_ttl_seconds = 150;
        config.config.session_idle_seconds = 100;
        let peer_endpoint = relay.bind().unwrap();
        let bridge =
            crate::account_bridge::Bridge::new(worker.client(), 4, Duration::from_secs(30))
                .unwrap();
        let (peer_halt, peer_stop) = watch::channel(false);
        let peer_task = tokio::spawn(relay.clone().run(peer_endpoint, bridge, peer_stop));
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
        let mut channel_tx = 43;
        send(
            &mut channel,
            &mut channel_tx,
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
        let mut offers = tokio::time::timeout(Duration::from_secs(2), conn.accept_uni())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(offers.read_u8().await.unwrap(), 2);
        let mut peer_links = Vec::new();
        let mut endpoints = Vec::new();
        for handle in &handles {
            let (endpoint, link) = connect_peer_offer(&mut offers).await;
            assert!(relay.connected(kr_tunnel::router::Owner {
                player: handle.player().wire_id,
                session: handle.generation(),
            }));
            peer_links.push(link);
            endpoints.push(endpoint);
        }
        send(&mut channel, &mut channel_tx, 81, &create_room()).await;
        assert_eq!(receive(&mut reply, &mut channel_rx).await.opcode, 86);
        send(
            &mut channel,
            &mut channel_tx,
            585,
            &write_uint(0, Width::U32).unwrap(),
        )
        .await;
        for (slot, player) in [&a, &b].into_iter().enumerate() {
            let profile = receive(&mut reply, &mut channel_rx).await;
            assert_eq!(profile.opcode, 106);
            assert_eq!(
                read_uint(&profile.payload[..1], Width::U8).unwrap(),
                slot as u64
            );
            assert_eq!(
                read_uint(&profile.payload[1..5], Width::U32).unwrap(),
                u64::from(player.wire_id)
            );
            assert_eq!(receive(&mut reply, &mut channel_rx).await.opcode, 110);
        }
        for opcode in [238, 117] {
            assert_eq!(receive(&mut reply, &mut channel_rx).await.opcode, opcode);
        }
        send(&mut channel, &mut channel_tx, 92, &[0x5b, 0xa5, 0x68]).await;
        let mut chars = [None; 2];
        for _ in 0..2 {
            let choice = receive(&mut reply, &mut channel_rx).await;
            assert_eq!(choice.opcode, 110);
            let slot = read_uint(&choice.payload[..1], Width::U8).unwrap() as usize;
            let state = read_uint(&choice.payload[1..5], Width::U32).unwrap();
            chars[slot] = Some((state >> 16) & 0x1fff);
        }
        assert_eq!(chars, [Some(0), Some(1)]);
        send(&mut channel, &mut channel_tx, 97, &[0x49]).await;
        let mut teams = [None; 2];
        for _ in 0..2 {
            let choice = receive(&mut reply, &mut channel_rx).await;
            assert_eq!(choice.opcode, 110);
            let slot = read_uint(&choice.payload[..1], Width::U8).unwrap() as usize;
            let state = read_uint(&choice.payload[1..5], Width::U32).unwrap();
            teams[slot] = Some((state >> 5) & 0x1f);
        }
        assert_eq!(teams, [Some(0), Some(3)]);
        let mut key = None;
        for n in 1..=2 {
            send(&mut channel, &mut channel_tx, 104, &[0; 20]).await;
            for opcode in [582, 121] {
                assert_eq!(receive(&mut reply, &mut channel_rx).await.opcode, opcode);
            }
            let mut loading = [0; 24];
            loading[8..10].copy_from_slice(&write_uint(31337, Width::U16).unwrap());
            send(&mut channel, &mut channel_tx, 99, &loading).await;
            for opcode in [197, 196] {
                assert_eq!(receive(&mut reply, &mut channel_rx).await.opcode, opcode);
            }
            let p = receive(&mut reply, &mut channel_rx).await;
            assert_eq!(p.opcode, 153);
            let k = read_uint(&p.payload[15..19], Width::U32).unwrap();
            for at in [35, 54] {
                assert_eq!(read_uint(&p.payload[at..at + 1], Width::U8).unwrap(), 1);
                assert_eq!(read_uint(&p.payload[at + 1..at + 5], Width::U32).unwrap(), 0);
                assert_eq!(read_uint(&p.payload[at + 5..at + 7], Width::U16).unwrap(), 0);
            }
            if let Some(old) = key.replace(k) {
                assert_ne!(old, k);
                let mut stale = bomb(old, 4);
                stale[5] = 0x5b;
                send(&mut channel, &mut channel_tx, 118, &stale).await;
            }
            let mut placement = bomb(k, 3);
            placement[5] = 0x5b; // second account uses room slot 1
            send(&mut channel, &mut channel_tx, 118, &placement).await;
            let p = receive(&mut reply, &mut channel_rx).await;
            assert_eq!(p.opcode, 159);
            assert_eq!(read_uint(&p.payload[17..18], Width::U8).unwrap(), 3);
            let p = receive_for(&mut reply, &mut channel_rx, Duration::from_secs(10)).await;
            assert_eq!(p.opcode, 158);
            let tick = n * 100;
            send(&mut channel, &mut channel_tx, 127, &report(1, 17, tick)).await;
            assert_eq!(receive(&mut reply, &mut channel_rx).await.opcode, 170);
            send(&mut channel, &mut channel_tx, 127, &report(1, 19, tick + 1)).await;
            assert_eq!(receive(&mut reply, &mut channel_rx).await.opcode, 170);
            send(&mut channel, &mut channel_tx, 127, &report(0, 21, tick + 2)).await;
            let rank = receive_for(&mut reply, &mut channel_rx, Duration::from_secs(12)).await;
            assert_eq!(rank.opcode, 175);
            // Exact trailing 202 body from the shared-client Windows capture.
            let post_rank = hex(concat!(
                "5a5896ca5e1b96ca539596ca539596ca539596ca5394",
                "96ca5e1b96ca539596ca539596ca539596ca5395"
            ));
            send(&mut channel, &mut channel_tx, 202, &post_rank).await;
            for opcode in [180, 110, 110] {
                let p = receive_for(&mut reply, &mut channel_rx, Duration::from_secs(12)).await;
                assert_eq!(p.opcode, opcode);
            }
            let db = rusqlite::Connection::open(temp.db()).unwrap();
            for (account, outcome) in [(&a, "win"), (&b, "loss")] {
                let count: u32 = db.query_row(
                    "SELECT count(*) FROM match_players WHERE account_id=?1 AND player_id=?2 AND outcome=?3",
                    (account.account_id, account.player_id, outcome), |r| r.get(0)
                ).unwrap();
                assert_eq!(count, n);
            }
        }
        send(&mut channel, &mut channel_tx, 86, &[]).await;
        assert_eq!(receive(&mut reply, &mut channel_rx).await.opcode, 241);
        assert_eq!(receive(&mut reply, &mut channel_rx).await.opcode, 91);
        send(&mut channel, &mut channel_tx, 81, &create_room()).await;
        let recreated = receive(&mut reply, &mut channel_rx).await;
        assert_eq!(recreated.opcode, 86);
        assert_eq!(read_uint(&recreated.payload[1..3], Width::U16).unwrap(), 2);

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
        for handle in &handles {
            assert!(!relay.connected(kr_tunnel::router::Owner {
                player: handle.player().wire_id,
                session: handle.generation(),
            }));
        }
        peer_halt.send_replace(true);
        peer_task.await.unwrap().unwrap();
        drop(peer_links);
        for endpoint in endpoints {
            endpoint.close(0u8.into(), b"done");
        }
        drop(hub.register(a.wire_id, 77).unwrap());
        drop(hub.register(b.wire_id, 77).unwrap());
        assert!(handles.iter().all(|h| !h.current()));
        assert_eq!(temp.active(), 0);
        halt.send_replace(true);
        server.close(0u8.into(), b"done");
        client.close(0u8.into(), b"done");
        journal.shutdown().await.unwrap();
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
        let create = create_room();
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
