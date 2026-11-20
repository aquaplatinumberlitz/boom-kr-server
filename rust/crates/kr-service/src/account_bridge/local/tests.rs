use super::*;
use crate::account_bridge::tests::{PASSWORD, Temp, admin, now, rt, worker};
use crate::accounts::TEST_SERIAL;
use kr_wire::{Width, write_uint};

pub(crate) fn login(names: &[&str]) -> Vec<u8> {
    let mut body = write_uint(0, Width::U8).unwrap();
    body.extend(write_uint(names.len() as u64, Width::U8).unwrap());
    for name in names {
        for (bytes, key) in [(name.as_bytes(), 0x1122_3344), (PASSWORD, 0x4433_2211)] {
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
pub(crate) fn trial(count: u8) -> Vec<u8> {
    let mut body = write_uint(count.into(), Width::U8).unwrap();
    body.extend([0; 16]);
    body.extend(write_uint(0, Width::U32).unwrap());
    body.extend(write_uint(0x0200007f, Width::U32).unwrap());
    body
}
pub(crate) fn channel(rows: &[(Player, u32)]) -> Vec<u8> {
    let mut body = write_uint(rows.len() as u64, Width::U8).unwrap();
    body.extend(write_uint(255, Width::U8).unwrap());
    for (player, echo) in rows {
        let name = kr_wire::kr00967::text949x::encode_name(&player.display_name).unwrap();
        body.extend(write_uint(name.len() as u64, Width::U16).unwrap());
        body.extend(kr_wire::kr00967::encode_bytes(&name).unwrap());
        body.extend(write_uint(player.wire_id.into(), Width::U32).unwrap());
        body.extend(write_uint((*echo).into(), Width::U32).unwrap());
        body.extend(write_uint(1, Width::U64).unwrap());
        body.extend(write_uint(0, Width::U8).unwrap());
    }
    for (value, width) in [
        (0, Width::U8),
        (1, Width::U8),
        (0, Width::U32),
        (0, Width::U32),
        (0, Width::U8),
        (0, Width::U32),
    ] {
        body.extend(write_uint(value, width).unwrap());
    }
    body
}
pub(crate) fn active(temp: &Temp) -> i64 {
    rusqlite::Connection::open(temp.db())
        .unwrap()
        .query_row(
            "SELECT count(*) FROM sessions WHERE revoked_at IS NULL",
            [],
            |r| r.get(0),
        )
        .unwrap()
}

#[test]
fn in_game_udp_capabilities_bind_both_original_accounts_and_close_on_group_revoke() {
    use crate::{account_bridge::Bridge, authenticated_peer};
    use kr_tunnel::{Link, Secret, router::Owner};
    let _serial = TEST_SERIAL.lock().unwrap();
    let (temp, _, b) = Temp::new();
    rt().block_on(async {
        let worker = worker(&temp).await;
        // Intentionally empty launcher registry: an in-game owner must never
        // need a second launcher login/grant or an IP-based registry lookup.
        let bridge = Bridge::new(worker.client(), 4, Duration::from_secs(30)).unwrap();
        let held = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
        let std::net::SocketAddr::V4(address) = held.local_addr().unwrap() else {
            unreachable!()
        };
        drop(held);
        let hub = authenticated_peer::Hub::new(&authenticated_peer::Config {
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
        let endpoint = hub.bind().unwrap();
        let (halt, stop) = watch::channel(false);
        let task = tokio::spawn(hub.clone().run(endpoint, bridge, stop));
        let mut session = LocalSession::new(worker.client(), 1).unwrap();
        let rows = session
            .login(&login(&["Alice", "Bob"]), 60, None)
            .await
            .unwrap();
        session.redirected(1).unwrap();
        let handles = session.channel(&channel(&rows), 2).await.unwrap();
        let mut links = Vec::new();
        let mut endpoints = Vec::new();
        let mut owners = Vec::new();
        for handle in &handles {
            let authority = handle.relay_authority();
            let owner = Owner {
                player: handle.player().wire_id,
                session: authority.generation(),
            };
            let offer = hub
                .offer_local(authority.clone(), Duration::from_secs(30))
                .unwrap();
            assert!(hub.offer_local(authority, Duration::from_secs(30)).is_err());
            let endpoint =
                kr_tunnel::client("127.0.0.1:0".parse().unwrap(), offer.certificate_der.into())
                    .unwrap();
            assert!(
                Link::connect(&endpoint, address.into(), &Secret::from_bytes([9; 32]))
                    .await
                    .is_err()
            );
            let link = Link::connect(&endpoint, address.into(), &offer.capability)
                .await
                .unwrap();
            assert!(
                Link::connect(&endpoint, address.into(), &offer.capability)
                    .await
                    .is_err()
            );
            assert!(hub.connected(owner));
            links.push(link);
            endpoints.push(endpoint);
            owners.push(owner);
        }
        temp.store().revoke(&admin(), b.account_id, now()).unwrap();
        // Whichever lane spots the revocation first, other consumers must
        // still be told it was a session denial rather than a stale handle.
        assert_eq!(
            handles[0].relay_authority().check_idle().await,
            Err(INVALID_SESSION)
        );
        assert_eq!(handles[1].authorize_activity().await, Err(INVALID_SESSION));
        assert_eq!(session.inspect().await, Err(INVALID_SESSION));
        assert!(handles.iter().all(|owner| !owner.current()));
        for handle in &handles {
            assert!(
                hub.offer_local(handle.relay_authority(), Duration::from_secs(30))
                    .is_err()
            );
            hub.retire_local(&handle.relay_authority());
        }
        assert!(owners.iter().all(|owner| !hub.connected(*owner)));
        session.finish().await.unwrap();
        assert_eq!(active(&temp), 0);
        halt.send_replace(true);
        task.await.unwrap().unwrap();
        drop(links);
        for endpoint in endpoints {
            endpoint.close(0u8.into(), b"done");
        }
        worker.shutdown().await.unwrap();
    });
}

#[test]
fn trial_roster_uses_the_same_revoke_expiry_and_replacement_fences() {
    let _serial = TEST_SERIAL.lock().unwrap();
    let (temp, _, _) = Temp::new();
    rt().block_on(async {
        let worker = worker(&temp).await;
        let mut retired = Vec::new();
        for expire in [false, true] {
            let mut session = LocalSession::new(worker.client(), 1).unwrap();
            let rows = session
                .trial(&trial(2), if expire { 1 } else { 60 }, None)
                .await
                .unwrap();
            assert!(retired.iter().all(|old: &LocalAuthority| {
                rows.iter()
                    .all(|row| row.0.player_id != old.player().player_id)
            }));
            session.redirected(1).unwrap();
            let handles = session.channel(&channel(&rows), 2).await.unwrap();
            assert!(retired.iter().all(|old| !old.current()));
            handles[0].inspect().await.unwrap();
            if expire {
                tokio::time::sleep(Duration::from_millis(1200)).await;
            } else {
                temp.store()
                    .revoke(&admin(), rows[1].0.account_id, now())
                    .unwrap();
            }
            assert!(handles[0].inspect().await.is_err());
            assert!(handles.iter().all(|h| !h.current()));
            session.finish().await.unwrap();
            assert_eq!(active(&temp), 0);
            retired.extend(handles);
        }
        worker.shutdown().await.unwrap();
    });
}

#[test]
fn two_identities_own_one_channel_and_old_handles_never_gain_a_reused_socket() {
    let _serial = TEST_SERIAL.lock().unwrap();
    let (temp, a, b) = Temp::new();
    rt().block_on(async {
        let worker = worker(&temp).await;
        let mut session = LocalSession::new(worker.client(), 11).unwrap();
        let rows = session
            .login(&login(&["Alice", "Bob"]), 60, Some(20))
            .await
            .unwrap();
        assert_eq!(rows.iter().map(|r| &r.0).collect::<Vec<_>>(), vec![&a, &b]);
        assert_ne!(rows[0].1, rows[1].1);
        assert!(session.channel(&channel(&rows), 22).await.is_err());
        assert!(session.redirected(99).is_err());
        session.redirected(11).unwrap();
        assert!(!session.disconnected(11).await.unwrap());
        assert!(!session.disconnected(99).await.unwrap());
        assert_eq!(active(&temp), 2);
        let mut swapped = rows.clone();
        swapped.reverse();
        assert!(session.channel(&channel(&swapped), 22).await.is_err());
        let mut forged = rows.clone();
        forged[1].1 = rows[0].1;
        assert!(session.channel(&channel(&forged), 22).await.is_err());
        let handles = session.channel(&channel(&rows), 22).await.unwrap();
        assert_eq!(handles.len(), 2);
        assert!(handles[0].owns(a.wire_id, 22));
        assert!(handles[1].owns(b.wire_id, 22));
        assert!(!handles[0].owns(b.wire_id, 22));
        assert!(!handles[0].owns(a.wire_id, 23));
        handles[0].inspect().await.unwrap();
        handles[1].authorize_activity().await.unwrap();
        let first = handles[0].character_preference(None).await.unwrap();
        let second = handles[1].character_preference(None).await.unwrap();
        assert_eq!(
            (first.player_id, second.player_id),
            (a.player_id, b.player_id)
        );
        let changed = handles[0]
            .character_preference(Some((1, first.revision)))
            .await
            .unwrap();
        assert_eq!(changed.character_key, 1);
        assert_eq!(
            handles[1]
                .character_preference(None)
                .await
                .unwrap()
                .character_key,
            second.character_key
        );
        // The TLS stream may disappear immediately after retirement. The
        // roster loses authority now, while its owner still owns DB cleanup.
        assert!(session.retire_disconnected(22).unwrap());
        assert!(handles.iter().all(|handle| !handle.current()));
        assert_eq!(active(&temp), 2);
        assert!(
            handles[0]
                .character_preference(Some((0, changed.revision)))
                .await
                .is_err()
        );
        session.finish().await.unwrap();
        assert_eq!(active(&temp), 0);
        assert!(
            handles
                .iter()
                .all(|h| !h.current() && *h.stopped().borrow())
        );
        let mut replacement = LocalSession::new(worker.client(), 11).unwrap();
        let fresh = replacement
            .login(&login(&["Alice"]), 60, None)
            .await
            .unwrap();
        replacement.redirected(11).unwrap();
        let current = replacement.channel(&channel(&fresh), 22).await.unwrap();
        assert!(current[0].current());
        assert!(handles[0].inspect().await.is_err());
        assert!(current[0].current());
        replacement.finish().await.unwrap();
        worker.shutdown().await.unwrap();
    });
}

#[test]
fn abandoned_login_and_expired_redirect_cannot_leave_channel_membership() {
    let _serial = TEST_SERIAL.lock().unwrap();
    let (temp, _a, _b) = Temp::new();
    rt().block_on(async {
        let worker = worker(&temp).await;
        let mut abandoned = LocalSession::new(worker.client(), 1).unwrap();
        abandoned
            .login(&login(&["Alice", "Bob"]), 60, None)
            .await
            .unwrap();
        assert!(abandoned.disconnected(1).await.unwrap());
        assert_eq!(active(&temp), 0);
        abandoned.finish().await.unwrap();
        let mut stale = LocalSession::new(worker.client(), 3).unwrap();
        let rows = stale
            .login(&login(&["Alice", "Bob"]), 60, None)
            .await
            .unwrap();
        stale.redirected(3).unwrap();
        stale.shared.lock().unwrap().redirect_until = Some(Instant::now() - Duration::from_secs(1));
        assert!(stale.inspect().await.is_err());
        assert!(stale.channel(&channel(&rows), 4).await.is_err());
        stale.finish().await.unwrap();
        assert_eq!(active(&temp), 0);
        worker.shutdown().await.unwrap();
    });
}

#[test]
fn revoke_or_expiry_of_one_account_retires_the_whole_transport() {
    let _serial = TEST_SERIAL.lock().unwrap();
    let (temp, _a, b) = Temp::new();
    rt().block_on(async {
        let worker = worker(&temp).await;
        let mut session = LocalSession::new(worker.client(), 1).unwrap();
        let rows = session
            .login(&login(&["Alice", "Bob"]), 60, None)
            .await
            .unwrap();
        session.redirected(1).unwrap();
        let handles = session.channel(&channel(&rows), 2).await.unwrap();
        temp.store().revoke(&admin(), b.account_id, now()).unwrap();
        assert!(handles[0].inspect().await.is_err());
        assert!(handles.iter().all(|h| !h.current()));
        session.finish().await.unwrap();
        let mut next = LocalSession::new(worker.client(), 3).unwrap();
        let rows = next
            .login(&login(&["Alice", "Bob"]), 1, None)
            .await
            .unwrap();
        next.redirected(3).unwrap();
        let handles = next.channel(&channel(&rows), 4).await.unwrap();
        tokio::time::sleep(Duration::from_millis(1200)).await;
        assert!(handles[1].inspect().await.is_err());
        assert!(handles.iter().all(|h| !h.current()));
        next.finish().await.unwrap();
        assert_eq!(active(&temp), 0);
        worker.shutdown().await.unwrap();
    });
}

#[test]
fn eof_cancellation_revokes_both_and_foreign_transport_cannot_claim_the_roster() {
    let _serial = TEST_SERIAL.lock().unwrap();
    let (temp, _a, _b) = Temp::new();
    rt().block_on(async {
        let worker = worker(&temp).await;
        let mut session = LocalSession::new(worker.client(), 1).unwrap();
        let rows = session
            .login(&login(&["Alice", "Bob"]), 60, None)
            .await
            .unwrap();
        session.redirected(1).unwrap();
        let mut foreign = LocalSession::new(worker.client(), 10).unwrap();
        assert!(foreign.channel(&channel(&rows), 20).await.is_err());
        foreign.finish().await.unwrap();
        let handles = session.channel(&channel(&rows), 2).await.unwrap();
        drop(session); // Same fallback as an aborted outer connection task.
        assert!(handles.iter().all(|h| !h.current()));
        // Give the spawned exact-token cleanup its first poll, then wait on
        // the worker's command barrier, rather than a fixed cleanup delay.
        tokio::task::yield_now().await;
        let resume = crate::account_worker::tests::pause(&worker.client()).await;
        assert_eq!(active(&temp), 0);
        resume.send(()).unwrap();
        worker.shutdown().await.unwrap();
    });
}

#[test]
fn cancelling_queued_login_never_publishes_accounts_or_membership() {
    let _serial = TEST_SERIAL.lock().unwrap();
    let (temp, _a, _b) = Temp::new();
    rt().block_on(async {
        let worker = worker(&temp).await;
        let resume = crate::account_worker::tests::pause(&worker.client()).await;
        let mut session = LocalSession::new(worker.client(), 1).unwrap();
        let shared = session.shared.clone();
        let job =
            tokio::spawn(async move { session.login(&login(&["Alice", "Bob"]), 60, None).await });
        tokio::task::yield_now().await;
        job.abort();
        let _ = job.await;
        assert!(shared.lock().unwrap().phase == Phase::Ended);
        resume.send(()).unwrap();
        let resume = crate::account_worker::tests::pause(&worker.client()).await;
        assert_eq!(active(&temp), 0);
        resume.send(()).unwrap();
        worker.shutdown().await.unwrap();
    });
}

#[test]
fn first_retirement_keeps_a_redacted_backend_reason_across_cleanup() {
    use crate::{account_worker::Error as Worker, accounts::Error as Store};
    let _serial = TEST_SERIAL.lock().unwrap();
    let (temp, _, _) = Temp::new();
    rt().block_on(async {
        let worker = worker(&temp).await;
        let mut session = LocalSession::new(worker.client(), 1).unwrap();
        let rows = session.trial(&trial(1), 60, None).await.unwrap();
        session.redirected(1).unwrap();
        let handles = session.channel(&channel(&rows), 2).await.unwrap();
        let failure = Error::Backend(Worker::Backend(Store::Storage));
        assert_eq!(
            retire_on(&session.shared, failure, Some("relay_health")),
            failure
        );
        assert_eq!(
            retire_on(&session.shared, INVALID_SESSION, Some("session_inspect")),
            failure
        );
        assert_eq!(handles[0].inspect().await, Err(failure));
        assert_eq!(session.inspect().await, Err(failure));
        assert_eq!(
            authority_end_event(failure, "relay_health"),
            serde_json::json!({
                "event": "native_account_authority_ended",
                "stage": "relay_health",
                "reason": "storage"
            })
        );
        assert!(!handles[0].current());
        assert!(*handles[0].stopped().borrow());
        session.finish().await.unwrap();
        assert_eq!(active(&temp), 0);
        worker.shutdown().await.unwrap();
    });
}
