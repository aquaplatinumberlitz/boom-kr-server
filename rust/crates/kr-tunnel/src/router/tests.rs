use super::*;
fn member(player: u32, slot: u8) -> Member {
    Member {
        owner: Owner {
            player,
            session: u64::from(player) + 10,
        },
        connection: u64::from(player) + 100,
        visit: 2,
        slot,
    }
}
fn scope(round: u64) -> Scope {
    Scope {
        service: [1; 16],
        room: 20,
        room_session: 30,
        room_generation: 4,
        round,
    }
}
fn router(routes: usize) -> Router {
    Router::new(Limits {
        routes,
        queue: 2,
        sends: 64,
        pending: Duration::from_millis(100),
        lease: Duration::from_secs(60),
    })
    .unwrap()
}
fn start(r: &mut Router, s: Scope, m: &[Member], now: Instant) -> Vec<(Owner, Vec<Assignment>)> {
    let assignments = r.prepare(s, m, now).unwrap();
    assert!(!r.ready(s));
    for (owner, rows) in &assignments {
        let bindings: Vec<_> = rows
            .iter()
            .enumerate()
            .map(|(i, a)| Binding {
                route: a.route,
                address: loopback(20000 + i as u16),
            })
            .collect();
        r.acknowledge(s, *owner, &bindings, now).unwrap();
    }
    assert!(r.ready(s));
    r.activate(s, m, now).unwrap();
    assignments
}
#[test]
fn same_ip_is_not_authority_and_old_routes_cannot_enter_reused_small_pool() {
    let now = Instant::now();
    let m = [member(70000, 0), member(42, 2), member(19, 5)];
    let mut r = router(6);
    let a = start(&mut r, scope(1), &m, now);
    assert_eq!(r.remaining(), 0);
    let route = a[0].1[0].route;
    assert_eq!(
        r.ingest(
            m[1].owner,
            Packet {
                route,
                sequence: 1,
                native: vec![7]
            },
            now
        ),
        Err(Denied::Owner)
    );
    r.ingest(
        m[0].owner,
        Packet {
            route,
            sequence: 1,
            native: vec![7],
        },
        now,
    )
    .unwrap();
    assert_eq!(
        r.ingest(
            m[0].owner,
            Packet {
                route,
                sequence: 1,
                native: vec![7]
            },
            now
        ),
        Err(Denied::Replay)
    );
    let mut received = Vec::new();
    assert_eq!(
        r.drain(scope(1), &m, now, |owner, packet| {
            received.push((owner, packet));
            Ok(true)
        })
        .unwrap()
        .sent,
        1
    );
    assert_eq!(received[0].0, m[1].owner);
    assert_eq!(received[0].1.native, vec![7]);
    assert_ne!(received[0].1.route, route);
    r.retire(scope(1));
    assert_eq!(r.remaining(), 6);
    let second = start(&mut r, scope(257), &m, now);
    assert_ne!(second[0].1[0].route, route);
    assert_eq!(
        r.ingest(
            m[0].owner,
            Packet {
                route,
                sequence: 2,
                native: vec![8]
            },
            now
        ),
        Err(Denied::Stale)
    );
    r.retire(scope(257));
    assert_eq!(r.remaining(), 6);
    assert_eq!(r.round_count(), 0);
}
#[test]
fn exact_ack_and_current_membership_are_required_and_departure_keeps_survivors() {
    let now = Instant::now();
    let m = [member(10, 0), member(20, 1), member(30, 7)];
    let mut r = router(6);
    let a = r.prepare(scope(1), &m, now).unwrap();
    assert!(r.activate(scope(1), &m, now).is_err());
    let bindings = [Binding {
        route: a[0].1[0].route,
        address: loopback(20001),
    }];
    assert!(r.acknowledge(scope(1), m[0].owner, &bindings, now).is_err());
    let mut bad: Vec<_> = a[0]
        .1
        .iter()
        .enumerate()
        .map(|(i, row)| Binding {
            route: row.route,
            address: loopback(20001 + i as u16),
        })
        .collect();
    bad[0].address = SocketAddrV4::new(Ipv4Addr::new(10, 0, 0, 1), 20001);
    assert!(r.acknowledge(scope(1), m[0].owner, &bad, now).is_err());
    r.retire(scope(1));
    let a = start(&mut r, scope(2), &m, now);
    r.retain(scope(2), &m[..2]).unwrap();
    assert_eq!(r.remaining(), 4);
    let a_to_b = a[0].1.iter().find(|row| row.slot == 1).unwrap().route;
    r.ingest(
        m[0].owner,
        Packet {
            route: a_to_b,
            sequence: 1,
            native: vec![9],
        },
        now,
    )
    .unwrap();
    assert!(r.drain(scope(2), &m, now, |_, _| Ok(true)).is_err());
    assert_eq!(
        r.drain(scope(2), &m[..2], now, |owner, _| {
            assert_eq!(owner, m[1].owner);
            Ok(true)
        })
        .unwrap()
        .sent,
        1
    );
    let mut rogue = m[0];
    rogue.visit += 1;
    assert!(r.retain(scope(2), &[rogue, m[1]]).is_err());
    assert!(r.acknowledge(scope(1), m[0].owner, &bindings, now).is_err());
    r.disconnect(m[1].owner);
    assert_eq!(r.remaining(), 6);
}
#[test]
fn expired_queued_data_and_restart_identity_cannot_influence_new_routes() {
    let now = Instant::now();
    let m = [member(1, 0), member(2, 1)];
    let mut r = router(2);
    let a = start(&mut r, scope(1), &m, now);
    let route = a[0].1[0].route;
    r.ingest(
        m[0].owner,
        Packet {
            route,
            sequence: 1,
            native: vec![1],
        },
        now,
    )
    .unwrap();
    assert_eq!(
        r.drain(scope(1), &m, now, |_, _| Ok(false)).unwrap().sent,
        0
    );
    assert_eq!(
        r.drain(
            scope(1),
            &m,
            now + Duration::from_millis(100),
            |_, _| panic!("expired send")
        )
        .unwrap()
        .expired,
        1
    );
    let mut restarted = router(2);
    let mut new = scope(1);
    new.service = [2; 16];
    start(&mut restarted, new, &m, now);
    assert_eq!(
        restarted.ingest(
            m[0].owner,
            Packet {
                route,
                sequence: 2,
                native: vec![2]
            },
            now
        ),
        Err(Denied::Stale)
    );
    assert!(
        r.ingest(
            m[0].owner,
            Packet {
                route,
                sequence: 2,
                native: vec![2]
            },
            now + Duration::from_secs(60)
        )
        .is_err()
    );
}
