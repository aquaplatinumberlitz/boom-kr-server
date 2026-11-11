use super::*;
use crate::room_registry::RoomId;
pub(super) fn limits() -> Limits {
    Limits {
        max_routes: 64,
        max_datagram: 255,
        pending_messages: 2,
        pending_bytes: 300,
        rate_messages: 10,
        rate_window: Duration::from_secs(1),
        pending_ttl: Duration::from_millis(100),
        lease_ttl: Duration::from_secs(10),
        sends_per_drain: 64,
    }
}
pub(super) fn fixture(
    n: usize,
    now: Instant,
) -> (Scope, Vec<Participant>, Vec<SocketAddrV4>, Kernel) {
    let scope = Scope {
        service_generation: 1,
        room: RoomKey {
            id: RoomId(7),
            session: 2,
            generation: 3,
        },
        round: 4,
    };
    let roster: Vec<_> = (0..n)
        .map(|i| Participant {
            member: Membership {
                identity: 70000 + i as u32,
                connection: 11 + i as u64,
                visit: 21 + i as u64,
            },
            session_generation: 41 + i as u64,
            source_ip: Ipv4Addr::LOCALHOST,
        })
        .collect();
    let addresses: Vec<_> = (0..n * (n - 1))
        .map(|i| SocketAddrV4::new(Ipv4Addr::LOCALHOST, 30000 + i as u16))
        .collect();
    let kernel = Kernel::prepare(scope, &roster, &addresses, limits(), now).unwrap();
    (scope, roster, addresses, kernel)
}
fn sender(i: usize) -> SocketAddrV4 {
    SocketAddrV4::new(Ipv4Addr::LOCALHOST, 40000 + i as u16)
}
fn keys(k: &Kernel) -> Vec<RouteKey> {
    k.routes().map(|(_, key, _)| key).collect()
}
#[test]
fn prepared_requires_activation_and_old_owner_keys_never_grant_a_route() {
    let now = Instant::now();
    let (scope, _, _, mut k) = fixture(2, now);
    let key = keys(&k);
    assert_eq!(
        k.ingest(0, key[0], sender(0), b"a", now),
        Err(DropReason::Inactive)
    );
    let mut wrong = scope;
    wrong.round += 1;
    assert!(k.activate(wrong, now).is_err());
    k.activate(scope, now).unwrap();
    for replacement in [
        RouteKey {
            scope: wrong,
            ..key[0]
        },
        RouteKey {
            source: Participant {
                session_generation: 99,
                ..key[0].source
            },
            ..key[0]
        },
        RouteKey {
            source: Participant {
                member: Membership {
                    visit: 99,
                    ..key[0].source.member
                },
                ..key[0].source
            },
            ..key[0]
        },
    ] {
        assert_eq!(
            k.ingest(0, replacement, sender(0), b"a", now),
            Err(DropReason::StaleRoute)
        );
    }
    k.invalidate();
    assert!(k.activate(scope, now).is_err());
}
#[test]
fn reverse_socket_source_and_observed_destination_preserve_each_original_payload() {
    let now = Instant::now();
    let (scope, _, _, mut k) = fixture(2, now);
    let key = keys(&k);
    k.activate(scope, now).unwrap();
    k.ingest(0, key[0], sender(0), b"A-to-B-original", now)
        .unwrap();
    assert_eq!(
        k.drain(now, |_, _, _| panic!("B has no observed mapping"))
            .unwrap(),
        0
    );
    k.ingest(1, key[1], sender(1), b"B-to-A-original", now)
        .unwrap();
    let mut sent = Vec::new();
    assert_eq!(
        k.drain(now, |edge, to, bytes| {
            sent.push((edge, to, bytes.to_vec()));
            Ok(bytes.len())
        })
        .unwrap(),
        2
    );
    assert!(sent.contains(&(1, sender(1), b"A-to-B-original".to_vec())));
    assert!(sent.contains(&(0, sender(0), b"B-to-A-original".to_vec())));
    assert_eq!(k.pending(), (0, 0));
    k.invalidate();
    assert_eq!(
        k.drain(now, |_, _, _| panic!("retired route sent"))
            .unwrap(),
        0
    );
    assert_eq!(
        k.ingest(0, key[0], sender(0), b"old", now),
        Err(DropReason::Inactive)
    );
}
#[test]
fn shared_ip_is_not_identity_and_unknown_source_or_rebinding_does_not_replace_tuple() {
    let now = Instant::now();
    let (scope, _, _, mut k) = fixture(2, now);
    let key = keys(&k);
    k.activate(scope, now).unwrap();
    assert_eq!(
        k.ingest(
            0,
            key[0],
            SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 2), 40000),
            b"bad",
            now
        ),
        Err(DropReason::Source)
    );
    assert_eq!(
        k.ingest(0, key[0], sender(0), b"", now),
        Err(DropReason::Size)
    );
    k.ingest(0, key[0], sender(0), b"good", now).unwrap();
    assert_eq!(
        k.ingest(0, key[0], sender(7), b"change", now),
        Err(DropReason::Rebinding)
    );
    k.ingest(1, key[1], sender(1), b"other", now).unwrap();
    k.drain(now, |edge, to, bytes| {
        assert_eq!(to, sender(edge));
        Ok(bytes.len())
    })
    .unwrap();
}
#[test]
fn ttl_budget_rate_and_clock_fences_do_not_renew_or_publish_old_pending_bytes() {
    let now = Instant::now();
    let (scope, roster, endpoints, _) = fixture(2, now);
    let mut bounds = limits();
    bounds.rate_messages = 2;
    let mut k = Kernel::prepare(scope, &roster, &endpoints, bounds, now).unwrap();
    let key = keys(&k);
    k.activate(scope, now).unwrap();
    k.ingest(0, key[0], sender(0), &[1; 200], now).unwrap();
    assert_eq!(
        k.ingest(0, key[0], sender(0), &[2; 200], now),
        Err(DropReason::Queue)
    );
    k.ingest(0, key[0], sender(0), b"b", now).unwrap();
    assert_eq!(
        k.ingest(0, key[0], sender(0), b"c", now),
        Err(DropReason::Rate)
    );
    let later = now + Duration::from_millis(101);
    k.ingest(1, key[1], sender(1), b"current", later).unwrap();
    let mut sent = Vec::new();
    k.drain(later, |_, _, b| {
        sent.push(b.to_vec());
        Ok(b.len())
    })
    .unwrap();
    assert_eq!(sent, vec![b"current".to_vec()]);
    assert_eq!(
        k.ingest(0, key[0], sender(0), b"old-time", now),
        Err(DropReason::Clock)
    );
    assert_eq!(
        k.ingest(
            0,
            key[0],
            sender(0),
            b"expired-lease",
            now + Duration::from_secs(11)
        ),
        Err(DropReason::Inactive)
    );
    assert_eq!(k.pending(), (0, 0));
}
#[test]
fn backpressure_on_one_edge_does_not_block_another_and_failed_send_retires_lease() {
    let now = Instant::now();
    let (scope, _, _, mut k) = fixture(2, now);
    let key = keys(&k);
    k.activate(scope, now).unwrap();
    k.ingest(0, key[0], sender(0), b"blocked", now).unwrap();
    k.ingest(1, key[1], sender(1), b"flows", now).unwrap();
    let sent = k
        .drain(now, |edge, _, b| {
            if edge == 1 {
                Err(io::ErrorKind::WouldBlock.into())
            } else {
                Ok(b.len())
            }
        })
        .unwrap();
    assert_eq!(sent, 1);
    assert_eq!(k.pending(), (1, 7));
    assert!(k.drain(now, |_, _, _| Ok(0)).is_err());
    assert_eq!(k.pending(), (0, 0));
    assert_eq!(
        k.ingest(1, key[1], sender(1), b"late", now),
        Err(DropReason::Inactive)
    );
}
#[test]
fn preparation_is_generic_bounded_and_rejects_ambiguous_participants_or_endpoints() {
    let now = Instant::now();
    for n in [2, 4, 8] {
        let (_, _, _, k) = fixture(n, now);
        assert_eq!(k.routes().count(), n * (n - 1));
    }
    let (scope, mut roster, mut endpoints, _) = fixture(2, now);
    endpoints[1] = endpoints[0];
    assert!(Kernel::prepare(scope, &roster, &endpoints, limits(), now).is_err());
    endpoints[1].set_port(30001);
    roster[1].member.identity = roster[0].member.identity;
    assert!(Kernel::prepare(scope, &roster, &endpoints, limits(), now).is_err());
    roster[1].member.identity += 1;
    let mut bounds = limits();
    bounds.max_routes = 1;
    assert!(Kernel::prepare(scope, &roster, &endpoints, bounds, now).is_err());
    bounds = limits();
    bounds.pending_messages = 0;
    assert!(Kernel::prepare(scope, &roster, &endpoints, bounds, now).is_err());
}

#[test]
fn four_participants_keep_all_directed_edges_and_reject_other_room_or_service_keys() {
    let now = Instant::now();
    let (scope, roster, endpoints, mut k) = fixture(4, now);
    let key = keys(&k);
    for (source, recipient) in roster.iter().enumerate() {
        let rows = k.endpoints_for(scope, *recipient).unwrap();
        assert_eq!(rows[source], (0, 0));
        for (index, route) in key.iter().enumerate() {
            if route.source == *recipient {
                let destination = roster.iter().position(|p| *p == route.destination).unwrap();
                assert_eq!(
                    rows[destination],
                    (u32::from(*endpoints[index].ip()), endpoints[index].port())
                );
            }
        }
    }
    assert!(
        k.endpoints_for(
            scope,
            Participant {
                session_generation: 99,
                ..roster[0]
            }
        )
        .is_err()
    );
    assert!(
        k.endpoints_for(Scope { round: 5, ..scope }, roster[0])
            .is_err()
    );
    k.activate(scope, now).unwrap();
    for (index, route) in key.iter().enumerate() {
        for foreign_scope in [
            Scope {
                service_generation: 2,
                ..scope
            },
            Scope {
                room: RoomKey {
                    id: RoomId(8),
                    ..scope.room
                },
                ..scope
            },
            Scope {
                room: RoomKey {
                    generation: 4,
                    ..scope.room
                },
                ..scope
            },
        ] {
            assert_eq!(
                k.ingest(
                    index,
                    RouteKey {
                        scope: foreign_scope,
                        ..*route
                    },
                    sender(index),
                    b"foreign",
                    now
                ),
                Err(DropReason::StaleRoute)
            );
        }
        k.ingest(index, *route, sender(index), &[index as u8], now)
            .unwrap();
    }
    let mut sent = HashSet::new();
    assert_eq!(
        k.drain(now, |reverse, to, bytes| {
            let forward = usize::from(bytes[0]);
            assert_eq!(key[reverse].source, key[forward].destination);
            assert_eq!(key[reverse].destination, key[forward].source);
            assert_eq!(to, sender(reverse));
            assert!(sent.insert(forward));
            Ok(bytes.len())
        })
        .unwrap(),
        12
    );
    assert_eq!(sent.len(), 12);
    assert_eq!(k.pending(), (0, 0));
    k.invalidate();
    assert!(k.endpoints_for(scope, roster[0]).is_err());
}

#[test]
fn loopback_udp_preserves_bytes_and_uses_the_reverse_edge_socket_as_source() {
    use std::net::{SocketAddr, UdpSocket};
    fn socket() -> UdpSocket {
        let s = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        s.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
        s
    }
    fn address(s: &UdpSocket) -> SocketAddrV4 {
        match s.local_addr().unwrap() {
            SocketAddr::V4(v4) => v4,
            SocketAddr::V6(_) => unreachable!(),
        }
    }
    let now = Instant::now();
    let (scope, roster, _, _) = fixture(2, now);
    let relay = [socket(), socket()];
    let clients = [socket(), socket()];
    let endpoints = relay.each_ref().map(address);
    let mut k = Kernel::prepare(scope, &roster, &endpoints, limits(), now).unwrap();
    let key = keys(&k);
    k.activate(scope, now).unwrap();
    let payloads: [&[u8]; 2] = [b"\0A\xff\x80original-to-B", b"\xffB\0\x81original-to-A"];
    let mut buffer = [0; 512];
    for index in 0..2 {
        clients[index]
            .send_to(payloads[index], endpoints[index])
            .unwrap();
        let (n, from) = relay[index].recv_from(&mut buffer).unwrap();
        assert_eq!(from, SocketAddr::V4(address(&clients[index])));
        k.ingest(
            index,
            key[index],
            address(&clients[index]),
            &buffer[..n],
            Instant::now(),
        )
        .unwrap();
        relay[index].set_nonblocking(true).unwrap();
    }
    assert_eq!(
        k.drain(Instant::now(), |reverse, to, bytes| {
            relay[reverse].send_to(bytes, to)
        })
        .unwrap(),
        2
    );
    for index in 0..2 {
        let (n, from) = clients[index].recv_from(&mut buffer).unwrap();
        assert_eq!(&buffer[..n], payloads[1 - index]);
        assert_eq!(from, SocketAddr::V4(endpoints[index]));
    }
    k.invalidate();
    assert_eq!(k.pending(), (0, 0));
}
