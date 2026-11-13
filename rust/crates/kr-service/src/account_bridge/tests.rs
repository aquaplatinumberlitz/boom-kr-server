use super::*;
use crate::{
    account_worker::Worker,
    accounts::{Admin, Store, TEST_SERIAL},
};
use kr_wire::{Width, write_uint};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
const PASSWORD: &[u8] = b"bridge-test-password";
const PEER: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 2);
fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
fn admin() -> Admin<'static> {
    Admin {
        actor: "bridge-test",
        reason: "M5 ownership seam tests",
    }
}
struct Temp(PathBuf, bool);
impl Temp {
    fn new() -> (Self, Player, Player) {
        let retained = std::env::var_os("KR_ACCOUNT_BRIDGE_TEST_RAW").map(PathBuf::from);
        let retain = retained.is_some();
        let parent = retained.unwrap_or_else(std::env::temp_dir);
        if retain {
            assert_eq!(
                fs::metadata(&parent).unwrap().permissions().mode() & 0o077,
                0
            );
        }
        let current = std::thread::current();
        let name = current
            .name()
            .unwrap_or("unknown")
            .rsplit("::")
            .next()
            .unwrap();
        let p = parent.join(format!(
            "kr-bridge-{}-{}-{name}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&p).unwrap();
        fs::set_permissions(&p, fs::Permissions::from_mode(0o700)).unwrap();
        let temp = Self(p, retain);
        let mut store = temp.store();
        store.begin_instance(&admin(), now()).unwrap();
        let a = store
            .create_account(&admin(), "Alice", "Alice", PASSWORD, now())
            .unwrap();
        let b = store
            .create_account(&admin(), "Bob", "KRGuest", PASSWORD, now())
            .unwrap();
        (temp, a, b)
    }
    fn db(&self) -> PathBuf {
        self.0.join("accounts.sqlite")
    }
    fn store(&self) -> Store {
        Store::open(&self.db()).unwrap()
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        if !self.1 {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
}
fn rt() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
}
async fn worker(temp: &Temp) -> Worker {
    Worker::open(temp.db(), 4, Duration::from_secs(3))
        .await
        .unwrap()
}
async fn grant(worker: &Worker, login: &str) -> crate::accounts::Grant {
    worker
        .client()
        .try_login(login, PASSWORD, 60, now())
        .unwrap()
        .wait()
        .await
        .unwrap()
}
fn login_body(secret: &str, peer: Ipv4Addr) -> Vec<u8> {
    let mut out = vec![];
    for (value, width) in [
        (4, Width::U8),
        (1, Width::U8),
        (secret.len() as u64, Width::U16),
    ] {
        out.extend(write_uint(value, width).unwrap());
    }
    out.extend(kr_wire::crypt(secret.as_bytes(), 0x1122_3344, false));
    out.extend([0xa5; 16]); // Intentionally opaque, not used as a credential.
    out.extend(write_uint(0, Width::U32).unwrap());
    out.extend(write_uint(u32::from_le_bytes(peer.octets()) as u64, Width::U32).unwrap());
    out.extend(write_uint(0, Width::U8).unwrap());
    out
}
fn channel_body(player: &Player, echo: u32) -> Vec<u8> {
    let mut out = vec![];
    out.extend(write_uint(1, Width::U8).unwrap());
    out.extend(write_uint(255, Width::U8).unwrap());
    out.extend(write_uint(player.display_name.len() as u64, Width::U16).unwrap());
    let mut name = player.display_name.as_bytes();
    while !name.is_empty() {
        let (n, w) = if name.len() >= 4 {
            (4, Width::U32)
        } else if name.len() >= 2 {
            (2, Width::U16)
        } else {
            (1, Width::U8)
        };
        let mut bytes = [0u8; 8];
        bytes[..n].copy_from_slice(&name[..n]);
        out.extend(write_uint(u64::from_le_bytes(bytes), w).unwrap());
        name = &name[n..];
    }
    out.extend(write_uint(player.wire_id as u64, Width::U32).unwrap());
    out.extend(write_uint(echo as u64, Width::U32).unwrap());
    out.extend([0x8c; 24]);
    out[31 + player.display_name.len()] = write_uint(0, Width::U8).unwrap()[0];
    out
}

#[test]
fn wire_guards_reject_malformed_count_size_peer_and_unsupported_projection() {
    let body = login_body("0123456789abcdef0123456789abcdef", PEER);
    assert_eq!(
        wire::login_token(&body, PEER).unwrap().expose_secret(),
        "0123456789abcdef0123456789abcdef"
    );
    for n in 0..body.len() {
        assert!(wire::login_token(&body[..n], PEER).is_err());
    }
    let mut extra = body.clone();
    extra.push(0);
    assert!(wire::login_token(&extra, PEER).is_err());
    assert!(wire::login_token(&body, Ipv4Addr::LOCALHOST).is_err());
    for i in [0, 1, 2, 3, 52, 56, 60] {
        let mut changed = body.clone();
        changed[i] ^= 1;
        assert!(wire::login_token(&changed, PEER).is_err());
    }
    let mut opaque = body.clone();
    opaque[36..52].fill(0);
    assert!(wire::login_token(&opaque, PEER).is_ok());
    assert!(wire::login_token(&login_body("short", PEER), PEER).is_err());
    assert!(wire::login_token(&login_body(&"\0".repeat(32), PEER), PEER).is_err());
    let p = Player {
        account_id: 1,
        player_id: 9,
        wire_id: 17,
        display_name: "Alice".into(),
        profile_version: 1,
    };
    let body = channel_body(&p, 123);
    assert_eq!(wire::channel_identity(&body).unwrap().wire_id, 17);
    for n in 0..body.len() {
        assert!(wire::channel_identity(&body[..n]).is_err());
    }
    for i in [0, 1, 2, 3] {
        let mut bad = body.clone();
        bad[i] ^= 1;
        assert!(wire::channel_identity(&bad).is_err());
    }
    let mut optional = body.clone();
    optional[31 + p.display_name.len()] = write_uint(1, Width::U8).unwrap()[0];
    assert!(wire::channel_identity(&optional).is_err());
    let mut bad_name = p;
    bad_name.display_name = "xx".into();
    assert!(wire::channel_identity(&channel_body(&bad_name, 123)).is_err());
    for name in ["four", "Fifteen_chars99"] {
        bad_name.display_name = name.into();
        assert_eq!(
            wire::channel_identity(&channel_body(&bad_name, 123))
                .unwrap()
                .name,
            name.as_bytes()
        );
    }
}

#[test]
fn two_accounts_same_ip_one_use_and_channel_owner_are_independent() {
    let _serial = TEST_SERIAL.lock().unwrap();
    let (temp, a, b) = Temp::new();
    rt().block_on(async {
        let w = worker(&temp).await;
        let ga = grant(&w, "Alice").await;
        let gb = grant(&w, "Bob").await;
        let bridge = Bridge::new(w.client(), 2, Duration::from_secs(10)).unwrap();
        assert!(bridge.issue(&ga.token, b.player_id, PEER).await.is_err());
        let la = bridge.issue(&ga.token, a.player_id, PEER).await.unwrap();
        let lb = bridge.issue(&gb.token, b.player_id, PEER).await.unwrap();
        assert_eq!(la.token().expose_secret().len(), 32);
        assert_eq!(ga.token.expose_secret().len(), 64);
        assert_ne!(la.token().digest(), lb.token().digest());
        assert!(bridge.issue(&ga.token, a.player_id, PEER).await.is_err());
        let ba = login_body(la.token().expose_secret(), PEER);
        let bb = login_body(lb.token().expose_secret(), PEER);
        assert!(bridge.login(&ba, Ipv4Addr::LOCALHOST, 1).await.is_err());
        let login_a = bridge.login(&ba, PEER, 1).await.unwrap();
        assert!(bridge.login(&ba, PEER, 3).await.is_err());
        assert!(bridge.login(&bb, PEER, 1).await.is_err());
        let login_b = bridge.login(&bb, PEER, 2).await.unwrap();
        let ca = channel_body(&a, login_a.echo);
        let cb = channel_body(&b, login_b.echo);
        assert!(bridge.channel(&ca, PEER, 3).await.is_err()); // Redirect not delivered.
        login_a.redirected().unwrap();
        login_b.redirected().unwrap();
        drop(la);
        drop(lb); // Old launcher handles cannot remove transferred ownership.
        assert!(
            bridge
                .channel(
                    &channel_body(&b, wire::channel_identity(&ca).unwrap().echo),
                    PEER,
                    3
                )
                .await
                .is_err()
        );
        assert!(bridge.channel(&ca, Ipv4Addr::LOCALHOST, 3).await.is_err());
        assert!(bridge.channel(&ca, PEER, 1).await.is_err()); // Login socket cannot become channel.
        let mut ch_b = bridge.channel(&cb, PEER, 4).await.unwrap();
        let mut ch_a = bridge.channel(&ca, PEER, 3).await.unwrap();
        let relay_a = ch_a.relay_authority();
        let relay_b = ch_b.relay_authority();
        assert!(relay_a.owns(a.wire_id, 3));
        assert!(!relay_a.owns(b.wire_id, 3));
        assert!(!relay_a.owns(a.wire_id, 4));
        relay_a.check_idle().await.unwrap();
        assert!(bridge.channel(&ca, PEER, 5).await.is_err());
        assert!(bridge.channel(&cb, PEER, 5).await.is_err());
        ch_a.check().await.unwrap();
        ch_b.check().await.unwrap();
        temp.store().revoke(&admin(), a.account_id, now()).unwrap();
        assert!(ch_a.check().await.is_err());
        assert!(!relay_a.current());
        assert!(relay_a.check_idle().await.is_err());
        relay_b.check_idle().await.unwrap();
        ch_b.check().await.unwrap();
        drop(ch_a);
        drop(ch_b);
        assert!(!relay_b.current());
        assert!(lock(&bridge.shared).unwrap().entries.is_empty());
        w.shutdown().await.unwrap();
    });
}

#[test]
fn account_owned_relay_reserves_before_publish_forwards_original_bytes_and_revoke_fences_both_sides()
 {
    use crate::{
        room::Room,
        room_registry::{RoomId, RoomKey},
        room_relay::Owner,
        udp_relay,
    };
    use std::net::{SocketAddr, UdpSocket};
    let _serial = TEST_SERIAL.lock().unwrap();
    let (temp, a, b) = Temp::new();
    rt().block_on(async {
        let w = worker(&temp).await;
        let bridge = Bridge::new(w.client(), 2, Duration::from_secs(10)).unwrap();
        let mut channels = Vec::new();
        for (player, login, connection) in [(&a, "Alice", 3), (&b, "Bob", 4)] {
            let grant = grant(&w, login).await;
            let launch = bridge
                .issue(&grant.token, player.player_id, PEER)
                .await
                .unwrap();
            let body = login_body(launch.token().expose_secret(), PEER);
            let login = bridge.login(&body, PEER, connection + 10).await.unwrap();
            let body = channel_body(player, login.echo);
            login.redirected().unwrap();
            channels.push(bridge.channel(&body, PEER, connection).await.unwrap());
        }
        let authorities: Vec<_> = channels.iter().map(Channel::relay_authority).collect();
        let mut model = Room::default();
        let ma = model.enter(a.wire_id, 3, true).unwrap();
        let mb = model.enter(b.wire_id, 4, false).unwrap();
        model.setup(ma).unwrap();
        model.setup(mb).unwrap();
        model.ready(mb).unwrap();
        let round = model.begin(ma).unwrap();
        // Obtain an unused local test port list; production config forbids zero.
        let held: Vec<_> = (0..6)
            .map(|_| UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap())
            .collect();
        let ports = held
            .iter()
            .map(|s| s.local_addr().unwrap().port())
            .collect();
        drop(held);
        let cfg = udp_relay::Config {
            bind: Ipv4Addr::LOCALHOST,
            advertise: Ipv4Addr::LOCALHOST,
            ports,
            receive_budget: 32,
            send_budget: 32,
            queue_messages: 8,
            queue_bytes: 2048,
            rate_per_second: 100,
            pending_ms: 500,
            lease_seconds: 60,
        };
        let mut owner = Owner::new(
            &cfg,
            RoomKey {
                id: RoomId(1),
                session: 1,
                generation: 1,
            },
        )
        .unwrap();
        let make_rows = || {
            vec![
                (1, mb, PEER, authorities[1].clone()),
                (0, ma, PEER, authorities[0].clone()),
            ]
        };
        let mut wrong = make_rows();
        wrong[0].3 = authorities[0].clone();
        assert!(owner.prepare(&model, wrong).is_err());
        let reservation = owner.prepare(&model, make_rows()).unwrap();
        owner.install(reservation);
        owner.transition(&model).unwrap();
        assert!(owner.authorization(&model).is_none());
        let rows = [owner.endpoints(ma).unwrap(), owner.endpoints(mb).unwrap()];
        assert_eq!(rows[0][0], (0, 0));
        assert_eq!(rows[1][1], (0, 0));
        let mut endpoints = [
            std::net::SocketAddrV4::new(rows[0][1].0.into(), rows[0][1].1),
            std::net::SocketAddrV4::new(rows[1][0].0.into(), rows[1][0].1),
        ];
        model.ack(ma, round).unwrap();
        model.ack(mb, round).unwrap();
        owner.transition(&model).unwrap();
        let (mut scope, witnesses) = owner.authorization(&model).unwrap();
        for witness in witnesses {
            witness.check_idle().await.unwrap();
        }
        let clients = [
            UdpSocket::bind((PEER, 0)).unwrap(),
            UdpSocket::bind((PEER, 0)).unwrap(),
        ];
        let payloads: [&[u8]; 2] = [b"A\0\xff-original", b"B\xff\0-original"];
        for (i, client) in clients.iter().enumerate() {
            client
                .set_read_timeout(Some(Duration::from_millis(100)))
                .unwrap();
            client.send_to(payloads[i], endpoints[i]).unwrap();
        }
        let stats = owner.pump(scope, &model).unwrap();
        assert_eq!(stats.accepted, 2);
        assert_eq!(stats.sent, 2);
        let mut bytes = [0; 256];
        for (i, client) in clients.iter().enumerate() {
            let (n, from) = client.recv_from(&mut bytes).unwrap();
            assert_eq!(&bytes[..n], payloads[1 - i]);
            assert_eq!(from, SocketAddr::V4(endpoints[i]));
        }
        let old_scope = scope;
        let old_endpoint = endpoints[0];
        model.end(round).unwrap();
        owner.transition(&model).unwrap();
        assert!(owner.endpoints(ma).is_err());
        model.refreshed(ma, round).unwrap();
        model.refreshed(mb, round).unwrap();
        model.ready(mb).unwrap();
        let next_round = model.begin(ma).unwrap();
        let reservation = owner.prepare(&model, make_rows()).unwrap();
        owner.install(reservation);
        model.ack(ma, next_round).unwrap();
        model.ack(mb, next_round).unwrap();
        owner.transition(&model).unwrap();
        scope = owner.authorization(&model).unwrap().0;
        let rows = [owner.endpoints(ma).unwrap(), owner.endpoints(mb).unwrap()];
        endpoints = [
            std::net::SocketAddrV4::new(rows[0][1].0.into(), rows[0][1].1),
            std::net::SocketAddrV4::new(rows[1][0].0.into(), rows[1][0].1),
        ];
        assert_ne!(endpoints[0], old_endpoint);
        // A callback which waited across reset cannot retire or send on the new lease.
        owner.invalidate_scope(old_scope);
        assert_eq!(owner.pump(old_scope, &model).unwrap().sent, 0);
        clients[0].send_to(payloads[0], old_endpoint).unwrap();
        assert_eq!(owner.pump(scope, &model).unwrap().received, 0);
        for (i, client) in clients.iter().enumerate() {
            client.send_to(payloads[i], endpoints[i]).unwrap();
        }
        assert_eq!(owner.pump(scope, &model).unwrap().sent, 2);
        for (i, client) in clients.iter().enumerate() {
            let (n, from) = client.recv_from(&mut bytes).unwrap();
            assert_eq!(&bytes[..n], payloads[1 - i]);
            assert_eq!(from, SocketAddr::V4(endpoints[i]));
        }
        // Queue input, revoke B, and observe denial before any forwarding.
        clients[0]
            .send_to(b"must-not-forward", endpoints[0])
            .unwrap();
        temp.store().revoke(&admin(), b.account_id, now()).unwrap();
        assert!(authorities[1].check_idle().await.is_err());
        assert_eq!(owner.pump(scope, &model).unwrap().sent, 0);
        assert!(clients[1].recv_from(&mut bytes).is_err());
        assert!(owner.endpoints(ma).is_err());
        // Late callbacks for retired scopes and bound handles never transfer ownership.
        owner.invalidate_scope(scope);
        drop(channels);
        assert!(authorities.iter().all(|w| !w.current()));
        w.shutdown().await.unwrap();
    });
}

#[test]
fn controlled_metadata_does_not_replace_transport_or_session_ownership() {
    let _serial = TEST_SERIAL.lock().unwrap();
    let (temp, player, _) = Temp::new();
    rt().block_on(async {
        let w = worker(&temp).await;
        let grant = grant(&w, "Alice").await;
        let bridge = Bridge::new(w.client(), 2, Duration::from_secs(10)).unwrap();
        let transport = Ipv4Addr::new(198, 51, 100, 7);
        let reported = Ipv4Addr::new(10, 1, 2, 3);
        let launch = bridge
            .issue(&grant.token, player.player_id, transport)
            .await
            .unwrap();
        let body = login_body(launch.token().expose_secret(), reported);
        // The legacy public method remains strict and a metadata mismatch must
        // not consume the grant before the deliberate controlled-profile call.
        assert!(bridge.login(&body, transport, 1).await.is_err());
        assert!(
            bridge
                .login_with_reported_address(&body, transport, PEER, 1)
                .await
                .is_err()
        );
        assert!(
            bridge
                .login_with_reported_address(&body, reported, reported, 1)
                .await
                .is_err()
        );
        let forged = login_body(&"0".repeat(32), reported);
        assert!(
            bridge
                .login_with_reported_address(&forged, transport, reported, 1)
                .await
                .is_err()
        );
        let login = bridge
            .login_with_reported_address(&body, transport, reported, 1)
            .await
            .unwrap();
        assert_eq!(login.player, player);
        assert!(
            bridge
                .login_with_reported_address(&body, transport, reported, 2)
                .await
                .is_err()
        );
        let channel = channel_body(&player, login.echo);
        login.redirected().unwrap();
        assert!(bridge.channel(&channel, reported, 2).await.is_err());
        let mut owned = bridge.channel(&channel, transport, 2).await.unwrap();
        owned.check().await.unwrap();
        temp.store()
            .revoke(&admin(), player.account_id, now())
            .unwrap();
        assert!(owned.check().await.is_err());
        drop(owned);
        assert!(lock(&bridge.shared).unwrap().entries.is_empty());
        w.shutdown().await.unwrap();
    });
}

#[test]
fn private_launch_metadata_is_bound_to_the_frontend_owner_not_an_ip_lookup() {
    let _serial = TEST_SERIAL.lock().unwrap();
    let (temp, player, _) = Temp::new();
    rt().block_on(async {
        let worker = Worker::open(temp.db(), 4, Duration::from_secs(2))
            .await
            .unwrap();
        let session = grant(&worker, "Alice").await;
        let bridge = Bridge::new(worker.client(), 128, Duration::from_secs(60)).unwrap();
        assert!(Bridge::new(worker.client(), 129, Duration::from_secs(60)).is_err());
        let transport = Ipv4Addr::new(192, 0, 2, 19);
        let reported = Ipv4Addr::new(10, 71, 0, 8);
        let launch = bridge
            .issue_metadata(&session.token, player.player_id, transport, Some(reported))
            .await
            .unwrap();
        let wrong = login_body(launch.token().expose_secret(), transport);
        assert!(
            bridge
                .login_for_profile(&wrong, transport, 81, |_| Some(transport))
                .await
                .is_err()
        );
        let body = login_body(launch.token().expose_secret(), reported);
        assert!(
            bridge
                .login_for_profile(&body, reported, 81, |_| Some(reported))
                .await
                .is_err()
        );
        let login = bridge
            .login_for_profile(&body, transport, 81, |_| None)
            .await
            .unwrap();
        assert_eq!(login.player, player);
        let channel = channel_body(&player, login.echo);
        login.redirected().unwrap();
        let owned = bridge.channel(&channel, transport, 82).await.unwrap();
        assert!(owned.relay_authority().owns(player.wire_id, 82));
        drop(owned);
        worker.shutdown().await.unwrap();
    });
}

#[test]
fn expiry_revoke_rename_and_unknown_session_fail_closed_at_handoff() {
    let _serial = TEST_SERIAL.lock().unwrap();
    let (temp, a, _) = Temp::new();
    rt().block_on(async {
        let w = worker(&temp).await;
        let ga = grant(&w, "Alice").await;
        let bridge = Bridge::new(w.client(), 2, Duration::from_secs(10)).unwrap();
        let bad = Token::parse(&"0".repeat(64)).unwrap();
        assert!(bridge.issue(&bad, a.player_id, PEER).await.is_err());
        assert!(lock(&bridge.shared).unwrap().entries.is_empty());
        let l = bridge.issue(&ga.token, a.player_id, PEER).await.unwrap();
        lock(&bridge.shared)
            .unwrap()
            .entries
            .get_mut(&a.player_id)
            .unwrap()
            .deadline = Instant::now();
        assert!(
            bridge
                .login(&login_body(l.token().expose_secret(), PEER), PEER, 1)
                .await
                .is_err()
        );
        let newer = bridge.issue(&ga.token, a.player_id, PEER).await.unwrap();
        drop(l); // Old generation cleanup cannot remove the replacement.
        assert_eq!(lock(&bridge.shared).unwrap().entries.len(), 1);
        let body = login_body(newer.token().expose_secret(), PEER);
        temp.store().revoke(&admin(), a.account_id, now()).unwrap();
        assert!(bridge.login(&body, PEER, 1).await.is_err());
        assert!(lock(&bridge.shared).unwrap().entries.is_empty());
        let ga = grant(&w, "Alice").await;
        let l = bridge.issue(&ga.token, a.player_id, PEER).await.unwrap();
        let auth = bridge
            .login(&login_body(l.token().expose_secret(), PEER), PEER, 2)
            .await
            .unwrap();
        let chan = channel_body(&a, auth.echo);
        auth.redirected().unwrap();
        temp.store().revoke(&admin(), a.account_id, now()).unwrap();
        assert!(bridge.channel(&chan, PEER, 3).await.is_err());
        assert!(lock(&bridge.shared).unwrap().entries.is_empty());
        let ga = grant(&w, "Alice").await;
        let l = bridge.issue(&ga.token, a.player_id, PEER).await.unwrap();
        let body = login_body(l.token().expose_secret(), PEER);
        // Same ID with a changed profile must require a newly projected launch.
        temp.store()
            .rename_player(&admin(), a.account_id, "Carol", a.profile_version, now())
            .unwrap();
        assert!(bridge.login(&body, PEER, 4).await.is_err());
        w.shutdown().await.unwrap();
    });
}

#[test]
fn abandoned_delivery_capacity_echo_exhaustion_entropy_and_closed_worker_cleanup() {
    let _serial = TEST_SERIAL.lock().unwrap();
    let (temp, a, b) = Temp::new();
    rt().block_on(async {
        let w = worker(&temp).await;
        let ga = grant(&w, "Alice").await;
        let gb = grant(&w, "Bob").await;
        let mut bridge = Bridge::new(w.client(), 1, Duration::from_secs(10)).unwrap();
        let l = bridge.issue(&ga.token, a.player_id, PEER).await.unwrap();
        assert!(matches!(
            bridge.issue(&gb.token, b.player_id, PEER).await,
            Err(Error::Capacity)
        ));
        let old = login_body(l.token().expose_secret(), PEER);
        drop(l);
        assert!(bridge.login(&old, PEER, 1).await.is_err());
        let l = bridge.issue(&ga.token, a.player_id, PEER).await.unwrap();
        let login = bridge
            .login(&login_body(l.token().expose_secret(), PEER), PEER, 2)
            .await
            .unwrap();
        let old_channel = channel_body(&a, login.echo);
        drop(login); // Failed auth/redirect delivery.
        assert!(bridge.channel(&old_channel, PEER, 3).await.is_err());
        drop(l);
        bridge.random = |_| Err(Error::Entropy);
        assert!(matches!(
            bridge.issue(&ga.token, a.player_id, PEER).await,
            Err(Error::Entropy)
        ));
        assert!(lock(&bridge.shared).unwrap().entries.is_empty());
        bridge.random = entropy;
        let l = bridge.issue(&ga.token, a.player_id, PEER).await.unwrap();
        lock(&bridge.shared).unwrap().seen_echo.extend(1..=1024);
        assert!(matches!(
            bridge
                .login(&login_body(l.token().expose_secret(), PEER), PEER, 4)
                .await,
            Err(Error::Exhausted)
        ));
        assert!(lock(&bridge.shared).unwrap().entries.is_empty());
        w.shutdown().await.unwrap();
        assert!(matches!(
            bridge.issue(&ga.token, a.player_id, PEER).await,
            Err(Error::Backend(account_worker::Error::Closed))
        ));
        assert!(lock(&bridge.shared).unwrap().entries.is_empty());
    });
}

#[test]
fn queued_cancel_busy_and_expired_generation_cannot_publish_late_results() {
    let _serial = TEST_SERIAL.lock().unwrap();
    let (temp, a, b) = Temp::new();
    rt().block_on(async {
        let w = Worker::open(temp.db(), 1, Duration::from_secs(3))
            .await
            .unwrap();
        let client = w.client();
        let ga = grant(&w, "Alice").await;
        let gb = grant(&w, "Bob").await;
        let bridge = Bridge::new(client.clone(), 2, Duration::from_secs(10)).unwrap();
        let resume = account_worker::tests::pause(&client).await;
        // Poll exactly once: reservation exists, authorization queued, future pending.
        let mut pending = Box::pin(bridge.issue(&ga.token, a.player_id, PEER));
        std::future::poll_fn(|cx| {
            assert!(std::future::Future::poll(pending.as_mut(), cx).is_pending());
            std::task::Poll::Ready(())
        })
        .await;
        assert!(matches!(
            bridge.issue(&gb.token, b.player_id, PEER).await,
            Err(Error::Backend(account_worker::Error::Busy))
        ));
        assert_eq!(lock(&bridge.shared).unwrap().entries.len(), 1);
        drop(pending);
        assert!(lock(&bridge.shared).unwrap().entries.is_empty());
        resume.send(()).unwrap();
        // Barrier behind the cancelled job: queue admission is nonblocking, not a drain guarantee.
        account_worker::tests::pause(&client)
            .await
            .send(())
            .unwrap();
        let l = bridge.issue(&ga.token, a.player_id, PEER).await.unwrap();
        let body = login_body(l.token().expose_secret(), PEER);
        let resume = account_worker::tests::pause(&client).await;
        let mut pending = Box::pin(bridge.login(&body, PEER, 1));
        std::future::poll_fn(|cx| {
            assert!(std::future::Future::poll(pending.as_mut(), cx).is_pending());
            std::task::Poll::Ready(())
        })
        .await;
        lock(&bridge.shared)
            .unwrap()
            .entries
            .get_mut(&a.player_id)
            .unwrap()
            .deadline = Instant::now();
        resume.send(()).unwrap();
        assert!(matches!(pending.await, Err(Error::Stale)));
        assert!(lock(&bridge.shared).unwrap().entries.is_empty());
        assert!(bridge.login(&body, PEER, 2).await.is_err());
        drop(l);
        w.shutdown().await.unwrap();
    });
}

#[test]
fn backend_session_expiry_during_queue_wait_uses_execution_utc() {
    let _serial = TEST_SERIAL.lock().unwrap();
    let (temp, a, _) = Temp::new();
    rt().block_on(async {
        let w = worker(&temp).await;
        let granted_at = now();
        let ga = w
            .client()
            .try_login("Alice", PASSWORD, 2, granted_at)
            .unwrap()
            .wait()
            .await
            .unwrap();
        let bridge = Bridge::new(w.client(), 1, Duration::from_secs(10)).unwrap();
        let l = bridge.issue(&ga.token, a.player_id, PEER).await.unwrap();
        let body = login_body(l.token().expose_secret(), PEER);
        let resume = account_worker::tests::pause(&w.client()).await;
        let mut pending = Box::pin(bridge.login(&body, PEER, 1));
        std::future::poll_fn(|cx| {
            assert!(std::future::Future::poll(pending.as_mut(), cx).is_pending());
            std::task::Poll::Ready(())
        })
        .await;
        tokio::time::timeout(Duration::from_secs(3), async {
            while now() < granted_at + 2 {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap();
        resume.send(()).unwrap();
        assert!(matches!(
            pending.await,
            Err(Error::Backend(account_worker::Error::Backend(
                crate::accounts::Error::InvalidSession
            )))
        ));
        assert!(lock(&bridge.shared).unwrap().entries.is_empty());
        drop(l);
        w.shutdown().await.unwrap();
    });
}

#[test]
fn channel_cancel_and_db_fault_invalidate_only_owned_generation() {
    let _serial = TEST_SERIAL.lock().unwrap();
    let (temp, a, _) = Temp::new();
    rt().block_on(async {
        let w = worker(&temp).await; let ga = grant(&w, "Alice").await;
        let bridge = Bridge::new(w.client(), 1, Duration::from_secs(10)).unwrap();
        let l = bridge.issue(&ga.token, a.player_id, PEER).await.unwrap();
        let login = bridge.login(&login_body(l.token().expose_secret(), PEER), PEER, 1).await.unwrap();
        let body = channel_body(&a, login.echo); login.redirected().unwrap();
        let resume = account_worker::tests::pause(&w.client()).await;
        let mut pending = Box::pin(bridge.channel(&body, PEER, 2));
        std::future::poll_fn(|cx| {
            assert!(std::future::Future::poll(pending.as_mut(), cx).is_pending());
            std::task::Poll::Ready(())
        }).await;
        drop(pending); drop(l);
        assert!(lock(&bridge.shared).unwrap().entries.is_empty());
        resume.send(()).unwrap();
        account_worker::tests::pause(&w.client()).await.send(()).unwrap();
        assert!(bridge.channel(&body, PEER, 3).await.is_err());
        let l = bridge.issue(&ga.token, a.player_id, PEER).await.unwrap();
        let login = bridge.login(&login_body(l.token().expose_secret(), PEER), PEER, 4).await.unwrap();
        let body = channel_body(&a, login.echo); login.redirected().unwrap();
        let mut channel = bridge.channel(&body, PEER, 5).await.unwrap();
        // Real SQLite write admission failure; no fake authorize result.
        let sql = rusqlite::Connection::open(temp.db()).unwrap();
        sql.execute_batch("CREATE TRIGGER fail_bridge_clock BEFORE UPDATE ON runtime_state BEGIN SELECT RAISE(ABORT,'injected clock write failure'); END;").unwrap();
        assert!(matches!(channel.check().await, Err(Error::Backend(account_worker::Error::Backend(crate::accounts::Error::Conflict)))));
        assert!(lock(&bridge.shared).unwrap().entries.is_empty());
        sql.execute_batch("DROP TRIGGER fail_bridge_clock;").unwrap();
        let newer = bridge.issue(&ga.token, a.player_id, PEER).await.unwrap();
        drop(channel); drop(l);
        assert_eq!(lock(&bridge.shared).unwrap().entries.len(), 1);
        drop(newer);
        w.shutdown().await.unwrap();
    });
}

#[test]
fn delivered_launch_retained_for_startup_then_new_short_handoff_deadline() {
    let _serial = TEST_SERIAL.lock().unwrap();
    let (temp, a, _) = Temp::new();
    rt().block_on(async {
        let w = worker(&temp).await;
        let grant = grant(&w, "Alice").await;
        assert!(Bridge::new(w.client(), 2, Duration::from_secs(121)).is_err());
        let bridge = Bridge::new(w.client(), 2, Duration::from_secs(60)).unwrap();
        let launch = bridge.issue(&grant.token, a.player_id, PEER).await.unwrap();
        let body = login_body(launch.token().expose_secret(), PEER);
        launch.delivered();
        assert!(
            lock(&bridge.shared).unwrap().entries[&a.player_id].deadline
                > Instant::now() + Duration::from_secs(50)
        );
        // Model elapsed startup independently of the actual native WCP→login lower bound.
        lock(&bridge.shared)
            .unwrap()
            .entries
            .get_mut(&a.player_id)
            .unwrap()
            .deadline = Instant::now() + Duration::from_secs(30);
        let login = bridge.login(&body, PEER, 1).await.unwrap();
        let deadline = lock(&bridge.shared).unwrap().entries[&a.player_id].deadline;
        assert!(deadline < Instant::now() + Duration::from_secs(11));
        assert!(deadline > Instant::now() + Duration::from_secs(9));
        let body = channel_body(&a, login.echo);
        login.redirected().unwrap();
        let channel = bridge.channel(&body, PEER, 2).await.unwrap();
        drop(channel);
        w.shutdown().await.unwrap();
    });
}

#[test]
fn worker_background_inspection_does_not_renew_idle_but_player_activity_does() {
    let _serial = TEST_SERIAL.lock().unwrap();
    let (temp, a, _) = Temp::new();
    rt().block_on(async {
        let w = worker(&temp).await;
        let granted = w
            .client()
            .try_login_with_idle("Alice", PASSWORD, 60, 30, now())
            .unwrap()
            .wait()
            .await
            .unwrap();
        let seen = || {
            rusqlite::Connection::open(temp.db())
                .unwrap()
                .query_row(
                    "SELECT last_seen_at FROM sessions WHERE revoked_at IS NULL",
                    [],
                    |r| r.get::<_, i64>(0),
                )
                .unwrap()
        };
        let before = seen();
        tokio::time::sleep(Duration::from_millis(1100)).await;
        assert_eq!(
            w.client()
                .try_inspect_current(&granted.token, a.player_id)
                .unwrap()
                .wait()
                .await,
            Ok(a.clone())
        );
        assert_eq!(seen(), before);
        assert_eq!(
            w.client()
                .try_authorize_current(&granted.token, a.player_id)
                .unwrap()
                .wait()
                .await,
            Ok(a.clone())
        );
        assert!(seen() > before);
        w.shutdown().await.unwrap();
    });
}

#[test]
fn frontend_selected_player_validates_distinct_metadata_behind_one_transport_ip() {
    let _serial = TEST_SERIAL.lock().unwrap();
    let (temp, a, b) = Temp::new();
    rt().block_on(async {
        let w = worker(&temp).await;
        let ga = grant(&w, "Alice").await;
        let gb = grant(&w, "Bob").await;
        let bridge = Bridge::new(w.client(), 2, Duration::from_secs(10)).unwrap();
        let addresses = [
            (a.wire_id, Ipv4Addr::new(10, 1, 0, 2)),
            (b.wire_id, Ipv4Addr::new(10, 1, 0, 3)),
        ];
        let resolve = |p: &Player| {
            addresses
                .iter()
                .find(|(id, _)| *id == p.wire_id)
                .map(|(_, ip)| *ip)
        };
        let la = bridge.issue(&ga.token, a.player_id, PEER).await.unwrap();
        let lb = bridge.issue(&gb.token, b.player_id, PEER).await.unwrap();
        let wrong = login_body(lb.token().expose_secret(), addresses[0].1);
        assert!(
            bridge
                .login_for_profile(&wrong, PEER, 11, resolve)
                .await
                .is_err()
        );
        let ab = login_body(la.token().expose_secret(), addresses[0].1);
        let bb = login_body(lb.token().expose_secret(), addresses[1].1);
        // Unknown profiles and wrong metadata do not consume another player's token.
        assert!(
            bridge
                .login_for_profile(&ab, PEER, 11, |_| None)
                .await
                .is_err()
        );
        let aa = bridge
            .login_for_profile(&ab, PEER, 11, resolve)
            .await
            .unwrap();
        let ba = bridge
            .login_for_profile(&bb, PEER, 22, resolve)
            .await
            .unwrap();
        assert_eq!(aa.player, a);
        assert_eq!(ba.player, b);
        let ac = channel_body(&a, aa.echo);
        let bc = channel_body(&b, ba.echo);
        aa.redirected().unwrap();
        ba.redirected().unwrap();
        assert!(
            bridge
                .channel(&channel_body(&a, u32::MAX), PEER, 33)
                .await
                .is_err()
        );
        let mut ca = bridge.channel(&ac, PEER, 33).await.unwrap();
        let mut cb = bridge.channel(&bc, PEER, 44).await.unwrap();
        ca.check().await.unwrap();
        cb.check().await.unwrap();
        temp.store().revoke(&admin(), a.account_id, now()).unwrap();
        assert!(ca.check().await.is_err());
        cb.check().await.unwrap();
        drop(ca);
        drop(cb);
        w.shutdown().await.unwrap();
    });
}

#[test]
fn authenticated_companions_follow_account_room_round_and_revoke_without_ip_ownership() {
    use crate::{
        authenticated_peer, authenticated_room_relay,
        room::Room,
        room_registry::{RoomId, RoomKey},
    };
    use kr_tunnel::{Link, Secret, client, companion, router::Owner as PeerOwner};
    use tokio::{net::UdpSocket, sync::watch};
    let _serial = TEST_SERIAL.lock().unwrap();
    let (temp, a, b) = Temp::new();
    rt().block_on(async {
        let w = worker(&temp).await;
        let bridge = Bridge::new(w.client(), 2, Duration::from_secs(60)).unwrap();
        let held = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
        let address = match held.local_addr().unwrap() {
            std::net::SocketAddr::V4(a) => a,
            _ => unreachable!(),
        };
        drop(held);
        let hub = authenticated_peer::Hub::new(&authenticated_peer::Config {
            bind: address,
            advertise: address,
            max_peers: 2,
            route_capacity: 2,
            queue_messages: 4,
            send_budget: 4,
            pending_ms: 500,
            lease_seconds: 60,
        })
        .unwrap();
        let listener = hub.bind().unwrap();
        let (hub_stop, stopped) = watch::channel(false);
        let server = tokio::spawn(hub.clone().run(listener, bridge.clone(), stopped));
        let mut channels = Vec::new();
        let mut endpoints = Vec::new();
        let mut proxies = Vec::new();
        let mut stops = Vec::new();
        let mut owners = Vec::new();
        for (player, name, connection) in [(&a, "Alice", 3), (&b, "Bob", 4)] {
            let grant = grant(&w, name).await;
            let launch = bridge
                .issue(&grant.token, player.player_id, PEER)
                .await
                .unwrap();
            let owner = PeerOwner {
                player: player.wire_id,
                session: launch.generation(),
            };
            bridge
                .validate_transport(owner.player, owner.session)
                .await
                .unwrap();
            assert!(
                bridge
                    .validate_transport(owner.player, owner.session + 100)
                    .await
                    .is_err()
            );
            let offer = hub.offer(owner, Duration::from_secs(60)).unwrap();
            assert!(hub.offer(owner, Duration::from_secs(60)).is_err());
            let endpoint =
                client("127.0.0.1:0".parse().unwrap(), offer.certificate_der.into()).unwrap();
            assert!(
                Link::connect(
                    &endpoint,
                    offer.endpoint.into(),
                    &Secret::from_bytes([9; 32])
                )
                .await
                .is_err()
            );
            let link = Link::connect(&endpoint, offer.endpoint.into(), &offer.capability)
                .await
                .unwrap();
            assert!(
                Link::connect(&endpoint, offer.endpoint.into(), &offer.capability)
                    .await
                    .is_err()
            );
            let body = login_body(launch.token().expose_secret(), PEER);
            launch.delivered();
            let login = bridge.login(&body, PEER, connection + 10).await.unwrap();
            let body = channel_body(player, login.echo);
            login.redirected().unwrap();
            channels.push(bridge.channel(&body, PEER, connection).await.unwrap());
            let (stop, stopped) = watch::channel(false);
            proxies.push(tokio::spawn(companion::run(
                link,
                "127.0.0.1".parse().unwrap(),
                26,
                stopped,
            )));
            stops.push(stop);
            endpoints.push(endpoint);
            owners.push(owner);
        }
        let authorities: Vec<_> = channels.iter().map(Channel::relay_authority).collect();
        let epochs = Arc::new(crate::round_epochs::RoundEpochs::authenticated());
        let mut relay = authenticated_room_relay::Owner::new(
            hub.clone(),
            RoomKey {
                id: RoomId(1),
                session: 1,
                generation: 1,
            },
        );
        let natives = [
            UdpSocket::bind("127.0.0.1:0").await.unwrap(),
            UdpSocket::bind("127.0.0.1:0").await.unwrap(),
        ];
        let mut old: Option<[std::net::SocketAddrV4; 2]> = None;
        let mut held_ports = Vec::new();
        let mut last = None;
        let mut low_key = None;
        // More rounds than the two-entry directed-route pool; native low byte
        // repeats intentionally. This is a transport test, not a KR oracle.
        for iteration in 0..4 {
            let mut model = Room::default();
            model.set_round_epochs(epochs.clone()).unwrap();
            let ma = model.enter(a.wire_id, 3, true).unwrap();
            let mb = model.enter(b.wire_id, 4, false).unwrap();
            model.setup(ma).unwrap();
            model.setup(mb).unwrap();
            model.ready(mb).unwrap();
            let round = model.begin(ma).unwrap();
            let key = crate::room::wire_key(round).unwrap() as u8;
            if let Some(old) = low_key {
                assert_eq!(key, old);
            }
            low_key = Some(key);
            let rows = vec![
                (0, ma, PEER, authorities[0].clone()),
                (1, mb, PEER, authorities[1].clone()),
            ];
            let mut wrong = rows.clone();
            wrong[0].3 = authorities[1].clone();
            assert!(relay.prepare(&model, wrong).is_err());
            relay.prepare(&model, rows).unwrap();
            assert!(relay.authorization(&model).is_none());
            tokio::time::timeout(Duration::from_secs(2), async {
                while !relay.ready() {
                    tokio::time::sleep(Duration::from_millis(5)).await;
                }
            })
            .await
            .unwrap();
            let native_rows = vec![
                (0, u32::from(Ipv4Addr::LOCALHOST), 10001),
                (1, u32::from(Ipv4Addr::LOCALHOST), 10002),
            ];
            let left = relay.decorate(ma, native_rows.clone()).unwrap();
            let right = relay.decorate(mb, native_rows).unwrap();
            assert_eq!(left[0].2, 10001);
            assert_eq!(right[1].2, 10002);
            let ports = [
                std::net::SocketAddrV4::new(left[1].1.into(), left[1].2),
                std::net::SocketAddrV4::new(right[0].1.into(), right[0].2),
            ];
            held_ports.extend(ports);
            model.ack(ma, round).unwrap();
            model.ack(mb, round).unwrap();
            relay.transition(&model).unwrap();
            let (scope, witnesses) = relay.authorization(&model).unwrap();
            for witness in witnesses {
                witness.check_idle().await.unwrap();
            }
            if let Some(old) = old {
                assert_ne!(ports, old);
                natives[0]
                    .send_to(b"stale old native bytes", old[0])
                    .await
                    .unwrap();
            }
            let payloads = [
                format!("A round {iteration}"),
                format!("B round {iteration}"),
            ];
            for i in 0..2 {
                natives[i]
                    .send_to(payloads[i].as_bytes(), ports[i])
                    .await
                    .unwrap();
            }
            let mut seen = [false; 2];
            tokio::time::timeout(Duration::from_secs(2), async {
                while seen != [true; 2] {
                    relay.pump(scope, &model).unwrap();
                    for i in 0..2 {
                        let mut buffer = [0; 256];
                        match natives[i].try_recv_from(&mut buffer) {
                            Ok((n, from)) => {
                                assert!(!seen[i]);
                                assert_eq!(from, std::net::SocketAddr::V4(ports[i]));
                                assert_eq!(&buffer[..n], payloads[1 - i].as_bytes());
                                seen[i] = true;
                            }
                            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                            Err(e) => panic!("native test receive: {e}"),
                        }
                    }
                    tokio::time::sleep(Duration::from_millis(5)).await;
                }
            })
            .await
            .unwrap();
            old = Some(ports);
            if iteration == 3 {
                last = Some((scope, model));
            } else {
                relay.invalidate();
            }
        }
        let (scope, model) = last.unwrap();
        temp.store().revoke(&admin(), a.account_id, now()).unwrap();
        assert!(authorities[0].check_idle().await.is_err());
        authorities[1].check_idle().await.unwrap();
        assert_eq!(relay.pump(scope, &model).unwrap().sent, 0);
        assert!(
            bridge
                .validate_transport(owners[0].player, owners[0].session)
                .await
                .is_err()
        );
        relay.invalidate();
        for stop in stops {
            stop.send(true).unwrap();
        }
        for task in proxies {
            let _ = tokio::time::timeout(Duration::from_secs(3), task)
                .await
                .unwrap()
                .unwrap();
        }
        hub_stop.send(true).unwrap();
        server.await.unwrap().unwrap();
        for owner in owners {
            assert!(!hub.connected(owner));
        }
        for port in held_ports {
            assert!(std::net::UdpSocket::bind(port).is_ok());
        }
        for endpoint in endpoints {
            endpoint.close(0_u8.into(), b"test end");
        }
        drop(channels);
        w.shutdown().await.unwrap();
    });
}

#[test]
fn same_ip_authenticated_first_packet_spoof_and_old_route_are_denied_with_current_wire_controls() {
    use crate::authenticated_peer;
    use kr_tunnel::{
        Link, Packet, client, control,
        router::{Binding, Member, Owner as PeerOwner, Scope},
    };
    use tokio::sync::watch;
    let _serial = TEST_SERIAL.lock().unwrap();
    let (temp, a, b) = Temp::new();
    rt().block_on(async {
        let w = worker(&temp).await;
        let bridge = Bridge::new(w.client(), 2, Duration::from_secs(60)).unwrap();
        let held = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
        let address = match held.local_addr().unwrap() {
            std::net::SocketAddr::V4(a) => a,
            _ => unreachable!(),
        };
        drop(held);
        let cfg = authenticated_peer::Config {
            bind: address,
            advertise: address,
            max_peers: 2,
            route_capacity: 2,
            queue_messages: 4,
            send_budget: 4,
            pending_ms: 500,
            lease_seconds: 60,
        };
        let hub = authenticated_peer::Hub::new(&cfg).unwrap();
        let listener = hub.bind().unwrap();
        let (stop, stopped) = watch::channel(false);
        let server = tokio::spawn(hub.clone().run(listener, bridge.clone(), stopped));
        let mut channels = Vec::new();
        let mut endpoints = Vec::new();
        let mut links = Vec::new();
        let mut controls = Vec::new();
        let mut members = Vec::new();
        for (slot, player, name, connection) in [(0, &a, "Alice", 3), (1, &b, "Bob", 4)] {
            let granted = grant(&w, name).await;
            let launch = bridge
                .issue(&granted.token, player.player_id, PEER)
                .await
                .unwrap();
            let owner = PeerOwner {
                player: player.wire_id,
                session: launch.generation(),
            };
            let offer = hub.offer(owner, Duration::from_secs(60)).unwrap();
            let endpoint =
                client("127.0.0.1:0".parse().unwrap(), offer.certificate_der.into()).unwrap();
            let mut link = Link::connect(&endpoint, offer.endpoint.into(), &offer.capability)
                .await
                .unwrap();
            controls.push(link.take_control().unwrap());
            links.push(link);
            endpoints.push(endpoint);
            let body = login_body(launch.token().expose_secret(), PEER);
            launch.delivered();
            let login = bridge.login(&body, PEER, connection + 10).await.unwrap();
            let body = channel_body(player, login.echo);
            login.redirected().unwrap();
            channels.push(bridge.channel(&body, PEER, connection).await.unwrap());
            members.push(Member {
                owner,
                connection,
                visit: 1,
                slot,
            });
        }
        // Both authenticated clients originate at the identical public tuple IP;
        // ports and route identifiers alone never confer another account's right.
        assert!(
            links
                .iter()
                .all(|link| link.remote_address().ip().is_loopback())
        );
        let mut previous = None;
        for round in [256, 512, 768, 1024] {
            let scope = Scope {
                service: hub.generation,
                room: 1,
                room_session: 1,
                room_generation: 1,
                round,
            };
            hub.prepare(scope, &members).unwrap();
            let mut routes = Vec::new();
            for (i, (send, recv)) in controls.iter_mut().enumerate() {
                loop {
                    let command = tokio::time::timeout(
                        Duration::from_secs(2),
                        control::read::<control::Server>(recv),
                    )
                    .await
                    .unwrap()
                    .unwrap();
                    match command {
                        control::Server::Retire { .. } => continue,
                        control::Server::Configure {
                            scope: received,
                            routes: rows,
                        } => {
                            assert_eq!(received, scope);
                            assert_eq!(rows.len(), 1);
                            assert_eq!(rows[0].slot, members[1 - i].slot);
                            routes.push(rows[0].route);
                            control::write(
                                send,
                                &control::Client::Ready {
                                    scope,
                                    bindings: vec![Binding {
                                        route: rows[0].route,
                                        address: format!("127.0.0.1:{}", 20000 + i)
                                            .parse()
                                            .unwrap(),
                                    }],
                                },
                            )
                            .await
                            .unwrap();
                            break;
                        }
                    }
                }
            }
            tokio::time::timeout(Duration::from_secs(2), async {
                while !hub.ready(scope) {
                    tokio::time::sleep(Duration::from_millis(5)).await;
                }
            })
            .await
            .unwrap();
            hub.activate(scope, &members).unwrap();
            if let Some(old) = previous {
                assert_ne!(old, routes[0]);
                links[0]
                    .send(&Packet {
                        route: old,
                        sequence: 1,
                        native: b"exact old logical packet".to_vec(),
                    })
                    .unwrap();
            }
            // A sends the first packet naming B's current route, with a sequence
            // that must remain usable by B. A vulnerable owner check would also
            // consume B's sequence and either forward the forgery or block B.
            links[0]
                .send(&Packet {
                    route: routes[1],
                    sequence: 1,
                    native: b"forged first B route from A".to_vec(),
                })
                .unwrap();
            tokio::time::sleep(Duration::from_millis(30)).await;
            assert_eq!(hub.pump(scope, &members).unwrap().sent, 0);
            assert!(
                tokio::time::timeout(Duration::from_millis(50), links[0].receive())
                    .await
                    .is_err()
            );
            links[1]
                .send(&Packet {
                    route: routes[1],
                    sequence: 1,
                    native: b"current B sequence one".to_vec(),
                })
                .unwrap();
            links[0]
                .send(&Packet {
                    route: routes[0],
                    sequence: 1,
                    native: b"current A sequence one".to_vec(),
                })
                .unwrap();
            let mut count = 0;
            tokio::time::timeout(Duration::from_secs(2), async {
                while count < 2 {
                    count += hub.pump(scope, &members).unwrap().sent;
                    tokio::time::sleep(Duration::from_millis(5)).await;
                }
            })
            .await
            .unwrap();
            for i in 0..2 {
                let received = tokio::time::timeout(Duration::from_secs(2), links[i].receive())
                    .await
                    .unwrap()
                    .unwrap();
                assert_eq!(received.route, routes[i]);
                assert_eq!(received.sequence, 1);
                assert_eq!(
                    received.native,
                    if i == 0 {
                        b"current B sequence one".to_vec()
                    } else {
                        b"current A sequence one".to_vec()
                    }
                );
            }
            links[1]
                .send(&Packet {
                    route: routes[1],
                    sequence: 1,
                    native: b"replay different bytes".to_vec(),
                })
                .unwrap();
            tokio::time::sleep(Duration::from_millis(30)).await;
            assert_eq!(hub.pump(scope, &members).unwrap().sent, 0);
            assert!(
                tokio::time::timeout(Duration::from_millis(50), links[0].receive())
                    .await
                    .is_err()
            );
            previous = Some(routes[0]);
            hub.retire(scope);
        }
        stop.send(true).unwrap();
        server.await.unwrap().unwrap();
        for link in links {
            link.close();
        }
        for endpoint in endpoints {
            endpoint.close(0_u8.into(), b"test end");
        }
        drop(channels);
        w.shutdown().await.unwrap();
    });
}

#[test]
fn restart_rebinds_public_port_but_old_pin_capability_scope_and_route_cannot_authorize_new_accounts()
 {
    use crate::authenticated_peer;
    use kr_tunnel::{
        Link, Packet, client, control,
        router::{Binding, Member, Owner as PeerOwner, Scope},
    };
    use tokio::sync::watch;
    let _serial = TEST_SERIAL.lock().unwrap();
    let (temp, a, b) = Temp::new();
    rt().block_on(async {
        let w = worker(&temp).await;
        let held = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
        let address = match held.local_addr().unwrap() {
            std::net::SocketAddr::V4(a) => a,
            _ => unreachable!(),
        };
        drop(held);
        let cfg = authenticated_peer::Config {
            bind: address,
            advertise: address,
            max_peers: 2,
            route_capacity: 2,
            queue_messages: 4,
            send_budget: 4,
            pending_ms: 500,
            lease_seconds: 60,
        };
        let mut old_offers: Vec<authenticated_peer::Offer> = Vec::new();
        let mut old_routes = Vec::new();
        let mut old_scope = None;
        let mut old_backend = None;
        for boot in 0..2 {
            if boot != 0 {
                // Quinn closes transport asynchronously; restart readiness is
                // bounded port availability, not an immediate synchronous Drop.
                tokio::time::timeout(Duration::from_secs(5), async {
                    loop {
                        match std::net::UdpSocket::bind(address) {
                            Ok(probe) => {
                                drop(probe);
                                break;
                            }
                            Err(error) if error.kind() == std::io::ErrorKind::AddrInUse => {
                                tokio::time::sleep(Duration::from_millis(5)).await;
                            }
                            Err(error) => panic!("restart port probe: {error}"),
                        }
                    }
                })
                .await
                .unwrap();
                temp.store().begin_instance(&admin(), now()).unwrap();
                let token = old_backend.as_ref().unwrap();
                assert!(
                    w.client()
                        .try_inspect_current(token, a.player_id)
                        .unwrap()
                        .wait()
                        .await
                        .is_err()
                );
            }
            let bridge = Bridge::new(w.client(), 2, Duration::from_secs(60)).unwrap();
            let hub = authenticated_peer::Hub::new(&cfg).unwrap();
            let listener = hub.bind().unwrap();
            assert_eq!(
                listener.local_addr().unwrap(),
                std::net::SocketAddr::V4(address)
            );
            let (stop, stopped) = watch::channel(false);
            let server = tokio::spawn(hub.clone().run(listener, bridge.clone(), stopped));
            let mut channels = Vec::new();
            let mut endpoints = Vec::new();
            let mut links = Vec::new();
            let mut controls = Vec::new();
            let mut members = Vec::new();
            let mut offers = Vec::new();
            for (slot, player, name, connection) in [(0, &a, "Alice", 3), (1, &b, "Bob", 4)] {
                let granted = grant(&w, name).await;
                if slot == 0 {
                    old_backend = Some(Token::parse(granted.token.expose_secret()).unwrap());
                }
                let launch = bridge
                    .issue(&granted.token, player.player_id, PEER)
                    .await
                    .unwrap();
                let owner = PeerOwner {
                    player: player.wire_id,
                    session: launch.generation(),
                };
                let offer = hub.offer(owner, Duration::from_secs(60)).unwrap();
                let endpoint = client(
                    "127.0.0.1:0".parse().unwrap(),
                    offer.certificate_der.clone().into(),
                )
                .unwrap();
                if boot != 0 {
                    let stale = client(
                        "127.0.0.1:0".parse().unwrap(),
                        old_offers[slot as usize].certificate_der.clone().into(),
                    )
                    .unwrap();
                    assert!(
                        Link::connect(
                            &stale,
                            address.into(),
                            &old_offers[slot as usize].capability
                        )
                        .await
                        .is_err()
                    );
                    stale.close(0_u8.into(), b"old pin test end");
                    // Even with the newly pinned certificate, old capability is
                    // not a new launch. It cannot consume the actual new offer.
                    assert!(
                        Link::connect(
                            &endpoint,
                            address.into(),
                            &old_offers[slot as usize].capability
                        )
                        .await
                        .is_err()
                    );
                }
                let mut link = Link::connect(&endpoint, address.into(), &offer.capability)
                    .await
                    .unwrap();
                controls.push(link.take_control().unwrap());
                links.push(link);
                endpoints.push(endpoint);
                offers.push(offer);
                let body = login_body(launch.token().expose_secret(), PEER);
                launch.delivered();
                let login = bridge.login(&body, PEER, connection + 10).await.unwrap();
                let body = channel_body(player, login.echo);
                login.redirected().unwrap();
                channels.push(bridge.channel(&body, PEER, connection).await.unwrap());
                members.push(Member {
                    owner,
                    connection,
                    visit: 1,
                    slot,
                });
            }
            let scope = Scope {
                service: hub.generation,
                room: 1,
                room_session: 1,
                room_generation: 1,
                round: 256,
            };
            if let Some(old) = old_scope {
                assert_ne!(scope, old);
                assert!(hub.prepare(old, &members).is_err());
            }
            hub.prepare(scope, &members).unwrap();
            let mut routes = Vec::new();
            for (i, (send, recv)) in controls.iter_mut().enumerate() {
                let control::Server::Configure {
                    scope: received,
                    routes: rows,
                } = tokio::time::timeout(Duration::from_secs(2), control::read(recv))
                    .await
                    .unwrap()
                    .unwrap()
                else {
                    panic!("unexpected control")
                };
                assert_eq!(received, scope);
                assert_eq!(rows.len(), 1);
                routes.push(rows[0].route);
                control::write(
                    send,
                    &control::Client::Ready {
                        scope,
                        bindings: vec![Binding {
                            route: rows[0].route,
                            address: format!("127.0.0.1:{}", 21000 + i).parse().unwrap(),
                        }],
                    },
                )
                .await
                .unwrap();
            }
            tokio::time::timeout(Duration::from_secs(2), async {
                while !hub.ready(scope) {
                    tokio::time::sleep(Duration::from_millis(5)).await;
                }
            })
            .await
            .unwrap();
            hub.activate(scope, &members).unwrap();
            if boot != 0 {
                assert_ne!(routes, old_routes);
                links[0]
                    .send(&Packet {
                        route: old_routes[0],
                        sequence: 1,
                        native: b"old exact logical packet".to_vec(),
                    })
                    .unwrap();
                tokio::time::sleep(Duration::from_millis(30)).await;
                assert_eq!(hub.pump(scope, &members).unwrap().sent, 0);
                assert!(
                    tokio::time::timeout(Duration::from_millis(50), links[1].receive())
                        .await
                        .is_err()
                );
            }
            for (i, link) in links.iter().enumerate() {
                link.send(&Packet {
                    route: routes[i],
                    sequence: 1,
                    native: format!("current boot{boot} member{i}").into_bytes(),
                })
                .unwrap();
            }
            let mut sent = 0;
            tokio::time::timeout(Duration::from_secs(2), async {
                while sent < 2 {
                    sent += hub.pump(scope, &members).unwrap().sent;
                    tokio::time::sleep(Duration::from_millis(5)).await;
                }
            })
            .await
            .unwrap();
            for (i, link) in links.iter().enumerate() {
                let received = tokio::time::timeout(Duration::from_secs(2), link.receive())
                    .await
                    .unwrap()
                    .unwrap();
                assert_eq!(received.route, routes[i]);
                assert_eq!(received.sequence, 1);
                assert_eq!(
                    received.native,
                    format!("current boot{boot} member{}", 1 - i).as_bytes()
                );
            }
            stop.send(true).unwrap();
            server.await.unwrap().unwrap();
            for link in links {
                link.close();
            }
            for endpoint in endpoints {
                endpoint.close(0_u8.into(), b"boot ended");
            }
            drop(channels);
            old_scope = Some(scope);
            old_routes = routes;
            old_offers = offers;
        }
        w.shutdown().await.unwrap();
    });
}

#[test]
fn character_preference_is_bound_to_current_channel_owner_revision_and_normal_worker_not_live_lane()
{
    let _serial = TEST_SERIAL.lock().unwrap();
    let (temp, a, b) = Temp::new();
    rt().block_on(async {
        let w = worker(&temp).await;
        let live = Worker::open_live(temp.0.join("accounts.sqlite"), 4, Duration::from_secs(1))
            .await
            .unwrap();
        let bridge = Bridge::new(w.client(), 2, Duration::from_secs(60))
            .unwrap()
            .with_live_authorization(live.client());
        let mut channels = Vec::new();
        for (player, name, connection) in [(&a, "Alice", 3), (&b, "Bob", 4)] {
            let grant = grant(&w, name).await;
            let launch = bridge
                .issue(&grant.token, player.player_id, PEER)
                .await
                .unwrap();
            let body = login_body(launch.token().expose_secret(), PEER);
            launch.delivered();
            let login = bridge.login(&body, PEER, connection + 10).await.unwrap();
            let body = channel_body(player, login.echo);
            login.redirected().unwrap();
            channels.push(bridge.channel(&body, PEER, connection).await.unwrap());
        }
        assert_eq!(
            channels[0].character_preference().await.unwrap().revision,
            0
        );
        let selected = channels[0].select_character(1, 0).await.unwrap();
        assert_eq!(
            (
                selected.player_id,
                selected.character_key,
                selected.revision
            ),
            (a.player_id, 1, 1)
        );
        assert_eq!(channels[0].select_character(1, 0).await.unwrap(), selected);
        assert_eq!(
            channels[1]
                .character_preference()
                .await
                .unwrap()
                .character_key,
            0
        );
        channels[0].select_character(0, 1).await.unwrap();
        assert!(matches!(
            channels[0].select_character(1, 0).await,
            Err(Error::Backend(account_worker::Error::Backend(
                crate::accounts::Error::Conflict
            )))
        ));
        temp.store().revoke(&admin(), a.account_id, now()).unwrap();
        assert!(channels[0].select_character(1, 2).await.is_err());
        assert_eq!(
            channels[1]
                .character_preference()
                .await
                .unwrap()
                .character_key,
            0
        );
        drop(channels);
        live.shutdown().await.unwrap();
        w.shutdown().await.unwrap();
    });
}
