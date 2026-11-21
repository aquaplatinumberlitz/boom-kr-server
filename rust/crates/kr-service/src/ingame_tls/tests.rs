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
        room_title_identities: Arc::default(),
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
        Self::login_ports(connection, login, name, 3838, 3938).await
    }
    async fn login_ports(
        connection: quinn::Connection,
        login: &str,
        name: &[u8],
        login_port: u16,
        channel_port: u16,
    ) -> Self {
        let (mut outgoing, mut incoming) = prelogin::open(&connection, login_port).await.unwrap();
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
        let (mut outgoing, mut incoming) = prelogin::open(&connection, channel_port).await.unwrap();
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
    async fn requestless(&mut self, opcode: u32) -> kr_wire::Packet {
        for _ in 0..32 {
            let packet = receive(&mut self.recv, &mut self.rx).await;
            if packet.opcode == opcode {
                return packet;
            }
            assert!(matches!(packet.opcode, 90 | 91 | 106 | 110 | 238));
        }
        panic!("Expected unsolicited packet {opcode} absent");
    }
    async fn expect(&mut self, opcode: u32) -> kr_wire::Packet {
        let packet = receive_for(&mut self.recv, &mut self.rx, Duration::from_secs(12)).await;
        assert_eq!(packet.opcode, opcode);
        packet
    }
}

// Compare the two slot snapshots the KR room receives, using the slot index
// instead of assuming that delivery order follows the room's slot order.
async fn waiting_slots(client: &mut NativeClient, order: Option<[usize; 2]>) -> [u64; 2] {
    let mut state = [None; 2];
    for expected in order.map(|slots| slots.map(Some)).unwrap_or([None; 2]) {
        let update = client.expect(110).await;
        assert_eq!(update.payload.len(), 6);
        let slot = read_uint(&update.payload[..1], Width::U8).unwrap() as usize;
        assert_eq!(read_uint(&update.payload[5..], Width::U8).unwrap(), 0);
        if let Some(expected) = expected {
            assert_eq!(slot, expected, "return snapshot slot order");
        }
        let packed = read_uint(&update.payload[1..5], Width::U32).unwrap();
        assert!(
            state[slot].replace(packed).is_none(),
            "duplicate slot {slot}"
        );
    }
    state.map(|slot| slot.expect("occupied slot"))
}

async fn room_marker(log: &mut mpsc::Receiver<serde_json::Value>, name: &str) -> serde_json::Value {
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            let event = log.recv().await.expect("room event channel");
            if event["event"] == name {
                return event;
            }
        }
    })
    .await
    .unwrap_or_else(|_| panic!("missing room marker {name}"))
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
fn cp373_replays_two_rounds_and_return_slot_order() {
    replay_session(include_str!("tests/fixtures/cp373.json"));
}

#[test]
fn cp375_replays_two_all_dead_rounds_and_relay() {
    replay_session(include_str!("tests/fixtures/cp375.json"));
}

#[test]
#[ignore = "pass a local semantic session with tools/session_replay.py replay"]
fn replay_local_session() {
    let path = std::env::var_os("KR_SEMANTIC_SESSION").expect("local replay fixture path");
    replay_session(&std::fs::read_to_string(path).expect("local replay fixture"));
}

#[derive(serde::Deserialize)]
struct ReplaySession {
    schema: String,
    rounds: Vec<ReplayRound>,
}

#[derive(serde::Deserialize)]
struct ReplayRound {
    steps: Vec<String>,
    result: String,
}

fn replay_session(fixture: &str) {
    let fixture: ReplaySession = serde_json::from_str(fixture).unwrap();
    assert_eq!(fixture.schema, "kre-semantic-session-v1");
    assert_eq!(fixture.rounds.len(), 2);
    let _serial = TEST_SERIAL.lock().unwrap();
    let temp = Temp::new();
    let admin = Admin {
        actor: "test",
        reason: "distinct native streams across TLS owners",
    };
    let mut store = Store::open(&temp.db()).unwrap();
    let epoch = store.begin_instance(&admin, 100).unwrap();
    // The host takes slot 0 even when the guest has the smaller account/player ID.
    let b = store
        .create_account(&admin, "BetaTwo", "DBUserB", PASSWORD, 100)
        .unwrap();
    let a = store
        .create_account(&admin, "AlphaOne", "DBUserA", PASSWORD, 100)
        .unwrap();
    assert!(a.wire_id > b.wire_id);
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
        let mut event_logs = Vec::new();
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
            let (events, log) = mpsc::channel(64);
            settings.events = events;
            event_logs.push(log);
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
        let stale = temp.age(&a, 30);
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
            assert_eq!(ids, [u64::from(b.wire_id), u64::from(a.wire_id)]);
        }
        assert!(temp.last_seen(&a) > stale, "room setup must renew idle");
        let stale = temp.age(&a, 30);
        clients[0].send(745, &[]).await;
        // Table request shares the same channel and is itself read-only.
        clients[0].send(528, &[]).await;
        clients[0].expect(238).await;
        assert_eq!(temp.last_seen(&a), stale, "keepalive must not renew idle");
        clients[1].send(97, &[0x59]).await;
        let initial_slots = waiting_slots(&mut clients[0], None).await;
        assert_eq!(waiting_slots(&mut clients[1], None).await, initial_slots);
        assert_eq!(initial_slots[0] & 0x1e, 2, "host retains chief bit");
        assert_eq!(initial_slots[1] & 0x1e, 0, "guest starts unready");
        let return_setup = fields(&[(0, Width::U32)]);
        let mut result_tail = fields(&[(0, Width::U8), (1, Width::U8)]);
        result_tail.extend([0; 20]); // Opaque one-actor row.
        let mut last_key = None;
        let mut last_routes = None;
        let mut played = Vec::new();
        for (index, trace) in fixture.rounds.iter().enumerate() {
            let round = index as u64 + 1;
            let all_dead = match trace.result.as_str() {
                "elimination" => false,
                "all_dead" => true,
                _ => panic!("unknown replay outcome"),
            };
            let mut expected = trace.steps.iter().map(String::as_str);
            let mut step = |got| assert_eq!(expected.next(), Some(got), "round {round}");
            let mut ready = fields(&[(6, Width::U32)]);
            ready.extend([0; 6]);
            ready.extend(fields(&[(1, Width::U8), (0, Width::U32)]));
            let stale = temp.age(&b, 30);
            clients[1].send(95, &ready).await;
            step("95");
            for client in &mut clients {
                let update = client.expect(110).await;
                step("ready");
                assert_eq!(update.payload.len(), 6);
                assert_eq!(read_uint(&update.payload[..1], Width::U8).unwrap(), 1);
                // DCAD80 requires the guest's Ready group to equal one. No
                // other slot flags, team or character may change on Ready95.
                assert_eq!(
                    read_uint(&update.payload[1..5], Width::U32).unwrap(),
                    initial_slots[1] | 4
                );
                assert_eq!(read_uint(&update.payload[5..], Width::U8).unwrap(), 0);
            }
            assert!(temp.last_seen(&b) > stale, "Ready must renew idle");
            let stale = temp.age(&a, 30);
            clients[0].send(104, &[0; 20]).await;
            step("104");
            for client in &mut clients {
                client.expect(582).await;
                client.expect(121).await;
                step("start");
            }
            assert!(temp.last_seen(&a) > stale, "Start must renew idle");
            let mut markers = Vec::new();
            for log in &mut event_logs {
                let ready_marker = room_marker(log, "native_tls_room_ready_written").await;
                let marker = room_marker(log, "native_tls_room_start_written").await;
                let stream_id = marker["stream_id"].as_u64().expect("numeric stream id");
                assert!(stream_id > 0);
                assert_eq!(ready_marker, serde_json::json!({
                    "event": "native_tls_room_ready_written", "stream_id": stream_id, "opcode": 110
                }));
                assert_eq!(marker, serde_json::json!({
                    "event": "native_tls_room_start_written", "stream_id": stream_id, "opcode": 121
                }));
                markers.push(stream_id);
            }
            assert_ne!(markers[0], markers[1]);
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
                step("99");
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
            let relay_sample = relay.health().unwrap();
            assert_eq!(relay_sample.reserved_routes, 2);
            assert_eq!(relay_sample.admitted, 2 * round);
            assert_eq!(relay_sample.submitted_to_quic, 2 * round);
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
            let stale = temp.age(&a, 30);
            clients[0].send(118, &bomb(wire_keys[0], 3)).await;
            step("118");
            for client in &mut clients {
                let spawn = client.expect(159).await;
                assert_eq!(read_uint(&spawn.payload[17..18], Width::U8).unwrap(), 3);
                assert_eq!(
                    read_uint(&spawn.payload[24..26], Width::U16).unwrap(),
                    round
                );
            }
            assert!(temp.last_seen(&a) > stale, "bomb placement must renew idle");
            for client in &mut clients {
                client.expect(158).await;
            }
            let tick = round as u32 * 100;
            clients[1].send(127, &report(1, 17, tick)).await;
            clients[0].expect(170).await;
            clients[1].send(127, &report(1, 19, tick + 1)).await;
            clients[0].expect(170).await;
            if all_dead {
                clients[0].send(127, &report(0, 17, tick + 2)).await;
                clients[1].expect(170).await;
                clients[0].send(127, &report(0, 19, tick + 3)).await;
                clients[1].expect(170).await;
            } else {
                clients[0].send(127, &report(0, 21, tick + 2)).await;
            }
            // Logs preserve the second bomb request, but not its body.
            // Only the first known bomb shape is reproduced in this test profile.
            step("118");
            for client in &mut clients {
                assert_eq!(client.expect(175).await.payload.len(), 302);
                client.send(202, &result_tail).await;
                step("202");
            }
            let mut return_steps: Vec<_> = expected.collect();
            return_steps.sort_unstable();
            for client in &mut clients {
                let returning = client.expect(180).await;
                assert_eq!(returning.payload, fields(&[(0, Width::U8); 3]));
                client.send(585, &return_setup).await;
                assert_eq!(return_steps.remove(0), "585");
            }
            for ((client, log), stream_id) in clients.iter_mut().zip(&mut event_logs).zip(markers) {
                // The native return tail must preserve the room visit, then
                // refresh the same slots with Ready cleared on both transports.
                assert_eq!(waiting_slots(client, Some([0, 1])).await, initial_slots);
                let marker = room_marker(log, "native_tls_room_return_refreshed").await;
                assert_eq!(return_steps.remove(0), "return");
                assert_eq!(marker, serde_json::json!({
                    "event": "native_tls_room_return_refreshed", "stream_id": stream_id
                }));
            }
            assert!(return_steps.is_empty());
            let db = rusqlite::Connection::open(temp.db()).unwrap();
            let room_keys: i64 = db
                .query_row(
                    "SELECT count(*) FROM (SELECT DISTINCT room_id, room_session FROM matches)",
                    [],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(room_keys, 1);
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
                let player = if slot == 0 { &a } else { &b };
                let past_all_dead = fixture.rounds[n / 2].result == "all_dead";
                let outcome = if past_all_dead { "draw" } else if slot == 0 { "win" } else { "loss" };
                assert_eq!(
                    u64::from(crate::room::wire_key(row.0.try_into().unwrap()).unwrap()),
                    played[n / 2]
                );
                assert_eq!((&*row.1, &*row.2), ("complete", fixture.rounds[n / 2].result.as_str()));
                assert_eq!((row.3, row.4, row.5), (slot as i64, player.account_id, player.player_id));
                assert_eq!((&*row.6, &*row.7), (outcome, "present"));
            }
        }
        proxy_halt.send_replace(true);
        for proxy in proxies {
            proxy.await.unwrap().unwrap();
        }
        temp.age(&a, 90);
        let ended = tokio::time::timeout(Duration::from_secs(3), clients[0].connection.closed())
            .await
            .unwrap();
        assert!(prelogin::account_session_ended(&ended));
        assert!(clients[1].connection.close_reason().is_none());
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
fn chat_bytes(bytes: &[u8]) -> Vec<u8> {
    let mut body = fields(&[(bytes.len() as u64, Width::U16)]);
    body.extend(kr_wire::kr00967::encode_bytes(bytes).unwrap());
    body
}

async fn expect_chat(client: &mut NativeClient, name: &[u8], bytes: &[u8]) {
    let packet = client.requestless(28).await;
    let mut literal = name.to_vec();
    literal.extend_from_slice(b" : ");
    literal.extend_from_slice(bytes);
    let mut expected = chat_bytes(&literal);
    expected.extend(fields(&[(0, Width::U8)]));
    assert_eq!(packet.payload, expected);
}

#[test]
fn vi_and_ko_chat_keeps_message_bytes_in_lobby_and_room_with_rate_and_scope() {
    let _serial = TEST_SERIAL.lock().unwrap();
    let temp = Temp::new();
    let admin = Admin {
        actor: "test",
        reason: "vi and ko chat scope",
    };
    let mut store = Store::open(&temp.db()).unwrap();
    store.begin_instance(&admin, 100).unwrap();
    store
        .create_account(&admin, "AlphaOne", "Việt01", PASSWORD, 100)
        .unwrap();
    store
        .create_account(&admin, "BetaTwo", "한국02", PASSWORD, 100)
        .unwrap();
    runtime().block_on(async {
        let worker = crate::account_worker::Worker::open(temp.db(), 8, Duration::from_secs(5))
            .await
            .unwrap();
        let hub = crate::room_service::Hub::new(
            &crate::room_service::Config {
                max_rooms: 4,
                round_profile: None,
                player_choices: true,
                capacity: None,
                player_round: Some(crate::room_io::PlayerRoundConfig {
                    characters: vec![0, 1],
                }),
            },
            crate::room_io::registry_foundation_tests::player_round_prototype(),
            None,
            None,
        )
        .unwrap();
        let (cert, key) = kr_tunnel::launch_tls::generate_identity().unwrap();
        let server = quinn::Endpoint::server(
            prelogin::server_config(cert.clone(), key.to_vec()).unwrap(),
            "127.0.0.1:0".parse().unwrap(),
        )
        .unwrap();
        let mut endpoint = quinn::Endpoint::client("127.0.0.1:0".parse().unwrap()).unwrap();
        endpoint.set_default_client_config(prelogin::client_config(cert).unwrap());
        let directory = Arc::new(lobby_chat::Directory::default());
        let (halt, stop) = watch::channel(false);
        let (events, mut log) = mpsc::channel(32);
        let mut clients = Vec::new();
        let mut tasks = Vec::new();
        let vi = b"Vi\xd2\x84t01".as_slice();
        let ko = b"\xc7\xd1\xb1\xb902".as_slice();
        for (login, name) in [("AlphaOne", vi), ("BetaTwo", ko)] {
            let mut authority = resources(worker.client());
            let cfg = Arc::get_mut(&mut authority).unwrap();
            cfg.hub = Some(hub.clone());
            cfg.patched_chat = true;
            cfg.lobby_chat = directory.clone();
            cfg.events = events.clone();
            let listener = server.clone();
            let stop = stop.clone();
            let permit = Arc::new(Semaphore::new(1)).acquire_owned().await.unwrap();
            tasks.push(tokio::spawn(async move {
                let incoming = listener.accept().await.unwrap().await.unwrap();
                connection(incoming, authority, stop, permit).await
            }));
            let tls = prelogin::connect(&endpoint, server.local_addr().unwrap())
                .await
                .unwrap();
            clients.push(NativeClient::login(tls, login, name).await);
            loop {
                let event = tokio::time::timeout(Duration::from_secs(2), log.recv())
                    .await
                    .unwrap()
                    .unwrap();
                if event["event"] == "native_local_lobby_ready" {
                    break;
                }
            }
        }
        let [a, b] = &mut clients[..] else {
            unreachable!()
        };
        // The input messages mix independently pinned CP949 and extension bytes.
        let vtext = b"Vi\xd2\x84t \xc7\xd1\xb1\xb9";
        let ktext = b"\xc7\xd1\xb1\xb9 Vi\xd2\x84t";
        a.send(18, &chat_bytes(vtext)).await;
        expect_chat(a, vi, vtext).await;
        expect_chat(b, vi, vtext).await;
        b.send(18, &chat_bytes(ktext)).await;
        expect_chat(a, ko, ktext).await;
        expect_chat(b, ko, ktext).await;
        let created = a.request(81, &create_room(), 86).await;
        let room = read_uint(&created.payload[1..3], Width::U16).unwrap();
        a.send(585, &fields(&[(0, Width::U32)])).await;
        a.requestless(117).await;
        tokio::time::sleep(Duration::from_millis(500)).await;
        b.send(18, &chat_bytes(ktext)).await;
        expect_chat(b, ko, ktext).await;
        // Request a real room table reply as a stream barrier; receiving a
        // lobby chat before it would fail rather than be silently skipped.
        a.send(528, &[]).await;
        a.expect(238).await;
        let join = fields(&[
            (room, Width::U16),
            (room, Width::U32),
            (0, Width::U16),
            (1, Width::U8),
            (5, Width::U16),
        ]);
        b.request(82, &join, 87).await;
        b.send(585, &fields(&[(0, Width::U32)])).await;
        b.requestless(117).await;
        a.send(18, &chat_bytes(vtext)).await;
        expect_chat(a, vi, vtext).await;
        expect_chat(b, vi, vtext).await;
        tokio::time::sleep(Duration::from_millis(500)).await;
        b.send(18, &chat_bytes(ktext)).await;
        expect_chat(a, ko, ktext).await;
        expect_chat(b, ko, ktext).await;
        b.send(18, &chat_bytes(ktext)).await;
        tokio::time::timeout(Duration::from_secs(3), b.connection.closed())
            .await
            .unwrap();
        let task = tasks.pop().unwrap();
        assert_eq!(
            task.await.unwrap().unwrap_err().to_string(),
            "chat_rate_bound"
        );
        assert!(a.connection.close_reason().is_none());
        halt.send_replace(true);
        for task in tasks {
            task.await.unwrap().unwrap();
        }
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
    fn age(&self, player: &crate::accounts::Player, seconds: i64) -> i64 {
        rusqlite::Connection::open(self.db()).unwrap().query_row(
            "UPDATE sessions SET last_seen_at=unixepoch()-?2, issued_at=min(issued_at,unixepoch()-?2) WHERE account_id=?1 AND revoked_at IS NULL RETURNING last_seen_at",
            (player.account_id, seconds), |r| r.get(0),
        ).unwrap()
    }
    fn last_seen(&self, player: &crate::accounts::Player) -> i64 {
        rusqlite::Connection::open(self.db())
            .unwrap()
            .query_row(
                "SELECT last_seen_at FROM sessions WHERE account_id=?1 AND revoked_at IS NULL",
                [player.account_id],
                |r| r.get(0),
            )
            .unwrap()
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
        lobby_chat: Arc::new(lobby_chat::Directory::default()),
        patched_chat: false,
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
            admissions_per_ip_per_minute: 5,
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

// Windows r2 c2s-60420, sequences 148–232. Only the two 118 round keys
// are rebound to the fixture's fresh 153 key. No login bytes are retained.
const WINDOWS_R2_GAMEPLAY: &[(u32, u32, &str)] = &[
    (
        148,
        127,
        "7d5aa57da57196ca505496ca53915b5b96ca531796ca2e8d940752954cb25395d6a947f6fa82df908f4d73dcf6f8e0dd0cee15979868521c244b3551c6a631d4964baca03edc0f48f30631cf5b4101d70ce005b9119f5250b2ec7f5e2f303fec20c123bd44cbb46551501779b5d4371574aa8da171241b79c9d9e0d58e5a9869e8823f6aa12a71761c0941acd771dc98a8b853954848",
    ),
    (
        151,
        127,
        "7d5ba72da57196ca505496ca53915b5b96ca538296ca418d969752954cb2539536a947f69eca5388a5cb42",
    ),
    (
        154,
        15,
        "916469f696ca539596ca539596ca539596ca539596ca539296ca518da5695a",
    ),
    (157, 190, ""),
    (160, 190, ""),
    (
        163,
        15,
        "91641e6096ca539596ca539596ca539596ca539596ca539296ca518da5695a",
    ),
    (
        166,
        15,
        "9164321496ca539596ca539596ca539596ca539596ca539296ca518da5695a",
    ),
    (
        169,
        15,
        "9164269e96ca539596ca539596ca539596ca539596ca539296ca518da5695a",
    ),
    (
        172,
        15,
        "9164db0d96ca539596ca539596ca539596ca539596ca539296ca518da5695a",
    ),
    (
        175,
        15,
        "9164cfb296ca539596ca539596ca539596ca539596ca539296ca518da5695a",
    ),
    (178, 118, "96ca62ac5a5a96ca53955a5a5b5a5a5a5b96ca53945a"),
    (181, 118, "96ca62ac5a5b96ca53945a545b5a5a5a5b96ca53945a"),
    (
        184,
        15,
        "9164fc2c96ca539596ca539596ca539596ca539596ca539296ca518da5695a",
    ),
    (
        187,
        127,
        "4b5aa569a54e96ca5bf396ca539096ca539b96ca5a8d96c8539596ca5395a569",
    ),
    (
        190,
        15,
        "916490d796ca539596ca539596ca539596ca539596ca539296ca518da5695a",
    ),
    (193, 127, "725ba759a54e96ca5b1596ca539796ca53975a"),
    (
        196,
        127,
        "4b5ba759a54e96ca5b1596ca539096ca539b96ca5a8d96c8539596ca5395a569",
    ),
    (
        199,
        15,
        "9164854396ca539596ca539596ca539596ca539596ca539296ca518da5695a",
    ),
    (202, 167, "a569a56f"),
    (205, 167, "a569a561"),
    (
        208,
        127,
        "495aa569a54e96ca5ad196ca539596ca539e96ca558d693553955a965a",
    ),
    (211, 127, "4f5ba759a54e96ca5ad196ca5393"),
    (214, 127, "4f5ba759a54e96ca5afd96ca5397"),
    (217, 127, "4f5ba759a54e96ca5a1f96ca5397"),
    (220, 127, "4f5ba759a54e96ca5a3996ca5397"),
    (
        223,
        15,
        "9164b9fc96ca539596ca539596ca539596ca539596ca539296ca518da5695a",
    ),
    (226, 127, "4f5ba759a54e96ca5a5b96ca5397"),
    (229, 127, "4f5ba759a54e96ca5a6596ca5397"),
    (232, 127, "4f5ba759a54e96ca598796ca5397"),
];

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
            if n == 1 {
                for &(sequence, opcode, raw_body) in WINDOWS_R2_GAMEPLAY {
                    let mut body = hex(raw_body);
                    if opcode == 118 {
                        body[..4].copy_from_slice(&write_uint(k, Width::U32).unwrap());
                    }
                    send(&mut channel, &mut channel_tx, opcode, &body).await;
                    if sequence == 181 {
                        for opcode in [159, 159, 158, 158] {
                            let packet = receive_for(&mut reply, &mut channel_rx, Duration::from_secs(10)).await;
                            assert_eq!(packet.opcode, opcode);
                        }
                    }
                }
                for subtype in [17, 17, 19, 21] {
                    let packet = receive(&mut reply, &mut channel_rx).await;
                    assert_eq!(packet.opcode, 170);
                    assert_eq!(read_uint(&packet.payload[..1], Width::U8).unwrap(), subtype);
                }
            } else {
                let mut placement = bomb(k, 3);
                placement[5] = 0x5b; // second account uses room slot 1
                send(&mut channel, &mut channel_tx, 118, &placement).await;
                let p = receive(&mut reply, &mut channel_rx).await;
                assert_eq!(p.opcode, 159);
                assert_eq!(read_uint(&p.payload[17..18], Width::U8).unwrap(), 3);
                let p = receive_for(&mut reply, &mut channel_rx, Duration::from_secs(10)).await;
                assert_eq!(p.opcode, 158);
                let tick = n * 3000;
                send(&mut channel, &mut channel_tx, 127, &report(1, 17, tick)).await;
                assert_eq!(receive(&mut reply, &mut channel_rx).await.opcode, 170);
                send(&mut channel, &mut channel_tx, 127, &report(1, 19, tick + 1)).await;
                assert_eq!(receive(&mut reply, &mut channel_rx).await.opcode, 170);
                send(&mut channel, &mut channel_tx, 127, &report(0, 21, tick + 2)).await;
            }
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
            let outcomes = if n == 1 { ["loss", "win"] } else { ["win", "loss"] };
            for (account, outcome) in [&a, &b].into_iter().zip(outcomes) {
                let count: u32 = db.query_row(
                    "SELECT count(*) FROM match_players WHERE account_id=?1 AND player_id=?2 AND outcome=?3",
                    (account.account_id, account.player_id, outcome), |r| r.get(0)
                ).unwrap();
                assert_eq!(count, 1);
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
    let one = Admission::admit(&shared, ip, 1, 7, now).unwrap();
    assert!(Admission::admit(&shared, ip, 1, 7, now).is_err());
    drop(one);
    for _ in 0..6 {
        drop(Admission::admit(&shared, ip, 1, 7, now).unwrap());
    }
    assert!(Admission::admit(&shared, ip, 1, 7, now).is_err());
    drop(Admission::admit(&shared, ip, 1, 7, now + Duration::from_secs(60)).unwrap());
    let mut retained = Vec::new();
    for n in 1..=255 {
        retained.push(
            Admission::admit(
                &shared,
                IpAddr::V4(Ipv4Addr::new(10, 0, 0, n)),
                1,
                5,
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
            5,
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
            5,
            now + Duration::from_secs(120)
        )
        .is_ok()
    );
}

#[test]
fn owner_end_event_keeps_safe_guard_and_opcode_without_underlying_error_text() {
    let slot = native_failure(
        io::Error::other("room slot does not belong to this transport"),
        "room_request",
        Some(127),
        Some(21),
    );
    assert_eq!(
        owner_ended(&Err(slot)),
        serde_json::json!({
            "event": "native_tls_owner_ended", "ok": false, "guard": "foreign_room_slot",
            "opcode": 127, "subtype": 21,
        })
    );
    let untrusted = native_failure(
        io::Error::other("private data must not appear in logs"),
        "room_request",
        Some(118),
        None,
    );
    assert_eq!(
        owner_ended(&Err(untrusted)),
        serde_json::json!({
            "event": "native_tls_owner_ended", "ok": false, "guard": "room_request",
            "opcode": 118, "subtype": null,
        })
    );
}

#[test]
fn room_poll_owner_event_keeps_stage_and_stream_without_renaming_known_guards() {
    for (err, guard) in [
        ("native room batch retired", "retired_room_batch"),
        ("arbitrary private error", "room_poll"),
    ] {
        let error = room_poll_failure(io::Error::other(err), "write", Some(121));
        let error = room_poll_stream(error, 24);
        let error = native_failure(error, "outer", None, None);
        assert_eq!(
            owner_ended(&Err(error)),
            serde_json::json!({
                "event": "native_tls_owner_ended", "ok": false, "guard": guard,
                "opcode": 121, "subtype": null, "stream_id": 24, "poll_stage": "write",
            })
        );
    }
}

#[test]
fn companion_reopens_expired_wcp_owner_and_authenticates_two_accounts_on_next_socket() {
    use tokio::io::AsyncWriteExt;
    let _serial = TEST_SERIAL.lock().unwrap();
    let temp = Temp::new();
    let mut store = Store::open(&temp.db()).unwrap();
    let admin = Admin {
        actor: "test",
        reason: "lazy companion regression",
    };
    store.begin_instance(&admin, 100).unwrap();
    for name in ["AlphaOne", "BetaTwo"] {
        store
            .create_account(&admin, name, name, PASSWORD, 100)
            .unwrap();
    }
    drop(store);
    runtime().block_on(async {
        tokio::time::timeout(Duration::from_secs(15), async {
            let worker = crate::account_worker::Worker::open(temp.db(), 4, Duration::from_secs(3))
                .await
                .unwrap();
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            let local = listener.local_addr().unwrap();
            let (cert, key) = kr_tunnel::launch_tls::generate_identity().unwrap();
            let endpoint = quinn::Endpoint::server(
                prelogin::server_config(cert.clone(), key.to_vec()).unwrap(),
                "127.0.0.1:0".parse().unwrap(),
            )
            .unwrap();
            let server = endpoint.local_addr().unwrap();
            let backend = kr_tunnel::UdpBackend::Portable;
            let client_endpoint = prelogin::endpoint(cert, backend).unwrap();
            let (halt, stop) = watch::channel(false);
            let companion = tokio::spawn(prelogin::companion_on_demand(
                client_endpoint,
                server,
                vec![listener],
                stop.clone(),
                backend,
            ));
            assert!(
                tokio::time::timeout(Duration::from_millis(150), endpoint.accept())
                    .await
                    .is_err()
            );
            let worker_client = worker.client();
            let server = tokio::spawn(async move {
                for time in [1, 10] {
                    let mut resource = resources(worker_client.clone());
                    Arc::get_mut(&mut resource).unwrap().ports = vec![local.port()];
                    Arc::get_mut(&mut resource).unwrap().config.prelogin_seconds = time;
                    let tls = endpoint.accept().await.unwrap().await.unwrap();
                    let permit = Arc::new(Semaphore::new(1)).acquire_owned().await.unwrap();
                    let result = connection(tls, resource, stop.clone(), permit).await;
                    if time == 1 {
                        assert_eq!(owner_ended(&result)["guard"], "prelogin_deadline");
                    }
                }
            });
            let mut wcp = tokio::net::TcpStream::connect(local).await.unwrap();
            let mut hello = [0; 15];
            wcp.read_exact(&mut hello).await.unwrap();
            wcp.write_all(&kr_wire::encode(&[0; 37], 0, &[], 40).unwrap())
                .await
                .unwrap();
            wcp.read_exact(&mut hello).await.unwrap();
            assert_eq!(hello, [1; 15]);
            match wcp.read(&mut hello).await {
                Ok(0) => (),
                Err(e) if e.kind() == io::ErrorKind::ConnectionReset => (),
                other => panic!("expired WCP socket still open: {other:?}"),
            }
            assert_eq!(temp.active(), 0);
            let mut login = tokio::net::TcpStream::connect(local).await.unwrap();
            login.read_exact(&mut hello).await.unwrap();
            login
                .write_all(&kr_wire::encode(&[0; 37], 1, &credentials(), 40).unwrap())
                .await
                .unwrap();
            let mut header = [0; 37];
            login.read_exact(&mut header).await.unwrap();
            let n = u32::from_le_bytes(header[13..17].try_into().unwrap()) as usize;
            assert!((2..=512).contains(&n));
            let mut packet = header.to_vec();
            packet.resize(38 + n, 0);
            login.read_exact(&mut packet[37..]).await.unwrap();
            let auth = kr_wire::decode(&packet, 43).unwrap();
            assert_eq!(auth.opcode, 4);
            assert_eq!(read_uint(&auth.payload[..1], Width::U8).unwrap(), 0);
            assert_eq!(temp.active(), 2);
            halt.send_replace(true);
            companion.await.unwrap().unwrap();
            server.await.unwrap();
            worker.shutdown().await.unwrap();
            assert_eq!(temp.active(), 0);
        })
        .await
        .unwrap();
    });
}

mod operations;
