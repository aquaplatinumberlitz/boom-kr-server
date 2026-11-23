//! Real SQLite/bridge ownership checks plus synthetic room queues, not native proof.
use super::*;
use crate::{
    account_bridge,
    account_worker::Worker,
    accounts::{Admin, Store, TEST_SERIAL},
    room_registry::{RoomId, RoomKey},
};
use std::{
    fs,
    net::Ipv4Addr,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
const PASSWORD: &[u8] = b"chat-test-password";
const PEER: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 2);
fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
fn admin() -> Admin<'static> {
    Admin {
        actor: "room-chat-test",
        reason: "queued owner lifetime controls",
    }
}
fn request(text: &str) -> Vec<u8> {
    let bytes = wire::text949x::encode(text).unwrap();
    let mut body = kr_wire::write_uint(bytes.len() as u64, Width::U16).unwrap();
    body.extend(wire::encode_bytes(&bytes).unwrap());
    body
}
fn login_body(secret: &str) -> Vec<u8> {
    let mut out = Vec::new();
    for (value, width) in [
        (4, Width::U8),
        (1, Width::U8),
        (secret.len() as u64, Width::U16),
    ] {
        out.extend(kr_wire::write_uint(value, width).unwrap());
    }
    out.extend(kr_wire::crypt(secret.as_bytes(), 0x1122_3344, false));
    out.extend([0xa5; 16]);
    out.extend(kr_wire::write_uint(0, Width::U32).unwrap());
    out.extend(kr_wire::write_uint(u32::from_le_bytes(PEER.octets()) as u64, Width::U32).unwrap());
    out.extend(kr_wire::write_uint(0, Width::U8).unwrap());
    out
}
fn channel_body(player: &crate::accounts::Player, echo: u32) -> Vec<u8> {
    let name = wire::text949x::encode(&player.display_name).unwrap();
    let mut out = Vec::new();
    out.extend(kr_wire::write_uint(1, Width::U8).unwrap());
    out.extend(kr_wire::write_uint(255, Width::U8).unwrap());
    out.extend(kr_wire::write_uint(name.len() as u64, Width::U16).unwrap());
    out.extend(wire::encode_bytes(&name).unwrap());
    out.extend(kr_wire::write_uint(player.wire_id as u64, Width::U32).unwrap());
    out.extend(kr_wire::write_uint(echo as u64, Width::U32).unwrap());
    out.extend([0x8c; 24]);
    out[31 + name.len()] = kr_wire::write_uint(0, Width::U8).unwrap()[0];
    out
}
struct Case {
    path: PathBuf,
    _worker: Worker,
    owners: Vec<account_bridge::Channel>,
    shared: Shared,
    sessions: Vec<Session>,
    keys: Vec<Membership>,
}
impl Drop for Case {
    fn drop(&mut self) {
        self.shared.lock().unwrap().channels.clear();
        let _ = fs::remove_dir_all(&self.path);
    }
}
impl Case {
    async fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "kr-chat-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
        let db = path.join("accounts.sqlite");
        let mut store = Store::open(&db).unwrap();
        store.begin_instance(&admin(), now()).unwrap();
        let players = vec![
            store
                .create_account(&admin(), "Alice", "Việt01", PASSWORD, now())
                .unwrap(),
            store
                .create_account(&admin(), "Bob", "한국02", PASSWORD, now())
                .unwrap(),
        ];
        let worker = Worker::open(db, 4, Duration::from_secs(2)).await.unwrap();
        let bridge = account_bridge::Bridge::new(worker.client(), 4, Duration::from_secs(60))
            .unwrap()
            .with_patched_949x_names(true);
        let mut owners = Vec::new();
        let mut channels = HashMap::new();
        let mut receivers = Vec::new();
        let mut keys = Vec::new();
        let mut model = Room::with_allocation(2, rules::SlotAllocation::FirstVacant).unwrap();
        for (index, (login, player)) in ["Alice", "Bob"].iter().zip(players).enumerate() {
            let grant = worker
                .client()
                .try_login(login, PASSWORD, 60, now())
                .unwrap()
                .wait()
                .await
                .unwrap();
            let launch = bridge
                .issue(&grant.token, player.player_id, PEER)
                .await
                .unwrap();
            let connection = 100 + index as u64;
            let authenticated = bridge
                .login(
                    &login_body(launch.token().expose_secret()),
                    PEER,
                    connection + 10,
                )
                .await
                .unwrap();
            let body = channel_body(&player, authenticated.echo);
            authenticated.redirected().unwrap();
            let owner = bridge.channel(&body, PEER, connection).await.unwrap();
            let key = model.enter(player.wire_id, connection, index == 0).unwrap();
            model.setup(key).unwrap();
            keys.push(key);
            let (sender, inbox) = mpsc::channel(8);
            receivers.push(inbox);
            channels.insert(
                player.wire_id,
                Channel {
                    connection,
                    sender,
                    endpoint: 0,
                    relay_authority: Some(owner.relay_authority()),
                },
            );
            owners.push(owner);
        }
        let shared = Arc::new(Mutex::new(Runtime {
            maps: None,
            selected_map: 0,
            map_revision: 0,
            journal: None,
            public_key: Some(RoomKey {
                session: 1,
                id: RoomId(3),
                generation: 1,
            }),
            managed_round: None,
            relay: None,
            authenticated: None,
            scope: Arc::new(()),
            model,
            players: HashMap::new(),
            bodies: HashMap::new(),
            channels,
            last_visits: HashMap::new(),
            requested: HashSet::new(),
            rostered: HashSet::new(),
            title: b"Chat".to_vec(),
            selector: 0,
            ready: packet(241, vec![]),
            ips: HashMap::new(),
            ports: HashMap::new(),
            floor_catalog: pickups::Catalog::default(),
            rules: RoomRules::default(),
            abort: None,
        }));
        let sessions = receivers
            .into_iter()
            .zip(&keys)
            .map(|(inbox, key)| Session {
                shared: shared.clone(),
                inbox,
                identity: key.identity,
                connection: key.connection,
            })
            .collect();
        Self {
            path,
            _worker: worker,
            owners,
            shared,
            sessions,
            keys,
        }
    }
    fn queue(&mut self, text: &str) -> Vec<Batch> {
        self.sessions[0].room_chat(&request(text)).unwrap();
        self.sessions
            .iter_mut()
            .map(|s| s.inbox.try_recv().unwrap())
            .collect()
    }
}
fn run(f: impl std::future::Future<Output = ()>) {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(f);
}
#[test]
fn unicode_owner_prefix_reaches_both_current_recipients_without_claimed_identity() {
    let _serial = TEST_SERIAL.lock().unwrap();
    run(async {
        let mut c = Case::new().await;
        let batches = c.queue("Xin chào 한국");
        for (s, b) in c.sessions.iter().zip(&batches) {
            assert!(s.valid(b));
            assert!(s.authorize_batch_source(b).await.unwrap());
            let reply = &b.packets[0].1;
            assert_eq!(reply.opcode, 28);
            let size = read_uint(&reply.payload[..2], Width::U16).unwrap() as usize;
            let text =
                wire::text949x::decode(&wire::decode_bytes(&reply.payload[2..2 + size]).unwrap())
                    .unwrap();
            assert_eq!(text, "Việt01 : Xin chào 한국");
        }
        assert!(c.sessions[0].room_chat(&[0; 4]).is_err());
        assert!(c.sessions.iter_mut().all(|s| s.inbox.try_recv().is_err()));
    });
}
#[test]
fn source_and_recipient_revoke_expiry_and_disconnect_fence_queued_chat() {
    let _serial = TEST_SERIAL.lock().unwrap();
    run(async {
        for mode in [
            "source-revoke",
            "source-expiry",
            "source-disconnect",
            "recipient-revoke",
        ] {
            let mut c = Case::new().await;
            let batches = c.queue("hello");
            match mode {
                "source-revoke" => {
                    Store::open(&c.path.join("accounts.sqlite"))
                        .unwrap()
                        .revoke(&admin(), c.owners[0].player.account_id, now())
                        .unwrap();
                }
                "source-expiry" => {
                    // Both operations can fall in the same wall-clock second.
                    // Keep the expired fixture within the session schema.
                    let changed = rusqlite::Connection::open(c.path.join("accounts.sqlite"))
                        .unwrap()
                        .execute(
                            "UPDATE sessions SET issued_at=min(issued_at, ?1-1), expires_at=?1 WHERE account_id=?2",
                            rusqlite::params![now(), c.owners[0].player.account_id],
                        )
                        .unwrap();
                    assert_eq!(changed, 1);
                }
                "source-disconnect" => {
                    c.owners.remove(0);
                }
                "recipient-revoke" => {
                    Store::open(&c.path.join("accounts.sqlite"))
                        .unwrap()
                        .revoke(&admin(), c.owners[1].player.account_id, now())
                        .unwrap();
                    assert!(c.owners[1].check_idle().await.is_err());
                }
                _ => unreachable!(),
            }
            assert!(
                !c.sessions[1]
                    .authorize_batch_source(&batches[1])
                    .await
                    .unwrap(),
                "{mode}"
            );
            assert!(!c.sessions[1].valid(&batches[1]), "{mode}");
        }
    });
}
#[test]
fn queue_full_publishes_no_partial_chat_and_stale_visit_round_or_scope_never_delivers() {
    let _serial = TEST_SERIAL.lock().unwrap();
    run(async {
        let mut c = Case::new().await;
        let sender = c.shared.lock().unwrap().channels[&c.keys[1].identity]
            .sender
            .clone();
        for _ in 0..8 {
            sender
                .try_send(Runtime::member_batch(c.keys[1], None, 0, vec![]))
                .unwrap();
        }
        assert!(c.sessions[0].room_chat(&request("bounded")).is_err());
        assert!(c.sessions[0].inbox.try_recv().is_err());
        for _ in 0..8 {
            c.sessions[1].inbox.try_recv().unwrap();
        }
        let batches = c.queue("fresh");
        let mut foreign = Case::new().await;
        assert!(!foreign.sessions[1].valid(&batches[1]));
        let other = foreign.queue("another room");
        assert!(!c.sessions[1].valid(&other[1]));
        {
            let mut r = c.shared.lock().unwrap();
            r.model.ready(c.keys[1]).unwrap();
            r.model.begin(c.keys[0]).unwrap();
        }
        assert!(!c.sessions[1].valid(&batches[1]));
        // Same current visits back in Idle must not revive an old waiting-room message.
        c.shared.lock().unwrap().model.phase = Phase::Idle;
        assert!(!c.sessions[1].valid(&batches[1]));
        let next = c.queue("new round");
        assert!(c.sessions[1].valid(&next[1]));
        {
            let mut r = c.shared.lock().unwrap();
            r.model.leave(c.keys[0]).unwrap();
            let key = r
                .model
                .enter(c.keys[0].identity, c.keys[0].connection, false)
                .unwrap();
            assert_ne!(key.visit, c.keys[0].visit);
        }
        assert!(!c.sessions[1].valid(&next[1]));
    });
}
#[test]
fn two_local_players_sharing_connection_receive_one_chat_copy_from_primary_owner() {
    let _serial = TEST_SERIAL.lock().unwrap();
    run(async {
        let path = std::env::temp_dir().join(format!(
            "kr-chat-2p-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
        let db = path.join("accounts.sqlite");
        let mut store = Store::open(&db).unwrap();
        store.begin_instance(&admin(), now()).unwrap();
        let remote_player = store
            .create_account(&admin(), "Bob", "한국02", PASSWORD, now())
            .unwrap();
        let worker = Worker::open(db, 4, Duration::from_secs(2)).await.unwrap();
        let bridge = account_bridge::Bridge::new(worker.client(), 4, Duration::from_secs(60))
            .unwrap()
            .with_patched_949x_names(true);

        let mut local = account_bridge::local::LocalSession::new(worker.client(), 1)
            .unwrap()
            .with_guest_name(Some("Việt01".into()));
        let rows = local
            .trial(&account_bridge::local::tests::trial(2), 60, None)
            .await
            .unwrap();
        local.redirected(1).unwrap();
        let local_owners = local
            .channel(&account_bridge::local::tests::channel(&rows), 100)
            .await
            .unwrap();

        let grant = worker
            .client()
            .try_login("Bob", PASSWORD, 60, now())
            .unwrap()
            .wait()
            .await
            .unwrap();
        let launch = bridge
            .issue(&grant.token, remote_player.player_id, PEER)
            .await
            .unwrap();
        let auth = bridge
            .login(&login_body(launch.token().expose_secret()), PEER, 111)
            .await
            .unwrap();
        let body = channel_body(&remote_player, auth.echo);
        auth.redirected().unwrap();
        let remote_owner = bridge.channel(&body, PEER, 101).await.unwrap();

        let mut model = Room::with_allocation(3, rules::SlotAllocation::FirstVacant).unwrap();
        let mut keys = model
            .enter_local_roster(
                &[
                    local_owners[0].player().wire_id,
                    local_owners[1].player().wire_id,
                ],
                100,
                true,
            )
            .unwrap();
        keys.push(model.enter(remote_player.wire_id, 101, false).unwrap());
        let mut channels = HashMap::new();
        let mut receivers = Vec::new();
        for (key, conn, authority) in [
            (keys[0], 100u64, local_owners[0].relay_authority()),
            (keys[1], 100u64, local_owners[1].relay_authority()),
            (keys[2], 101u64, remote_owner.relay_authority()),
        ] {
            model.setup(key).unwrap();
            let (sender, inbox) = mpsc::channel(8);
            receivers.push(inbox);
            channels.insert(
                key.identity,
                Channel {
                    connection: conn,
                    sender,
                    endpoint: 0,
                    relay_authority: Some(authority),
                },
            );
        }

        let shared = Arc::new(Mutex::new(Runtime {
            maps: None,
            selected_map: 0,
            map_revision: 0,
            journal: None,
            public_key: Some(RoomKey {
                session: 1,
                id: RoomId(4),
                generation: 1,
            }),
            managed_round: None,
            relay: None,
            authenticated: None,
            scope: Arc::new(()),
            model,
            players: HashMap::new(),
            bodies: HashMap::new(),
            channels,
            last_visits: HashMap::new(),
            requested: HashSet::new(),
            rostered: HashSet::new(),
            title: b"Chat2P".to_vec(),
            selector: 0,
            ready: packet(241, vec![]),
            ips: HashMap::new(),
            ports: HashMap::new(),
            floor_catalog: pickups::Catalog::default(),
            rules: RoomRules::default(),
            abort: None,
        }));
        let mut sessions: Vec<_> = receivers
            .into_iter()
            .zip(&keys)
            .map(|(inbox, key)| Session {
                shared: shared.clone(),
                inbox,
                identity: key.identity,
                connection: key.connection,
            })
            .collect();

        let decode = |batch: &Batch| {
            let reply = &batch.packets[0].1;
            assert_eq!(reply.opcode, 28);
            let size = read_uint(&reply.payload[..2], Width::U16).unwrap() as usize;
            wire::text949x::decode(&wire::decode_bytes(&reply.payload[2..2 + size]).unwrap())
                .unwrap()
        };

        sessions[0].room_chat(&request("Xin chào 2P")).unwrap();
        let host_copy = sessions[0].inbox.try_recv().unwrap();
        let peer_copy = sessions[2].inbox.try_recv().unwrap();
        assert!(sessions[0].valid(&host_copy));
        assert!(
            sessions[0]
                .authorize_batch_source(&host_copy)
                .await
                .unwrap()
        );
        assert_eq!(decode(&host_copy), "Việt01 : Xin chào 2P");
        assert!(sessions[2].valid(&peer_copy));
        assert!(
            sessions[2]
                .authorize_batch_source(&peer_copy)
                .await
                .unwrap()
        );
        assert_eq!(decode(&peer_copy), "Việt01 : Xin chào 2P");
        assert!(sessions[0].inbox.try_recv().is_err());
        assert!(sessions[1].inbox.try_recv().is_err());
        assert!(sessions[2].inbox.try_recv().is_err());

        sessions[2].room_chat(&request("Chào 2 bạn")).unwrap();
        let host_reply = sessions[0].inbox.try_recv().unwrap();
        let peer_reply = sessions[2].inbox.try_recv().unwrap();
        assert!(
            sessions[0]
                .authorize_batch_source(&host_reply)
                .await
                .unwrap()
        );
        assert_eq!(decode(&host_reply), "한국02 : Chào 2 bạn");
        assert_eq!(decode(&peer_reply), "한국02 : Chào 2 bạn");
        assert!(sessions[1].inbox.try_recv().is_err());

        shared.lock().unwrap().channels.clear();
        local.finish().await.unwrap();
        drop(remote_owner);
        worker.shutdown().await.unwrap();
        let _ = fs::remove_dir_all(&path);
    });
}
