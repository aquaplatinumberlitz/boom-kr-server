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
    bad_name.display_name = "four".into();
    assert!(wire::channel_identity(&channel_body(&bad_name, 123)).is_err());
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
        assert!(bridge.channel(&ca, PEER, 5).await.is_err());
        assert!(bridge.channel(&cb, PEER, 5).await.is_err());
        ch_a.check().await.unwrap();
        ch_b.check().await.unwrap();
        temp.store().revoke(&admin(), a.account_id, now()).unwrap();
        assert!(ch_a.check().await.is_err());
        ch_b.check().await.unwrap();
        drop(ch_a);
        drop(ch_b);
        assert!(lock(&bridge.shared).unwrap().entries.is_empty());
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
