//! Exercise the same pinned two-owner service behind both launcher transports.
use super::{friends, *};
use crate::{
    Identity, Link, Packet, RouteId, UdpBackend, control,
    router::{Assignment, Scope},
};
use kr_p2p::{Invite, SecretKey, room, self_host};
use tokio::net::{TcpStream, UdpSocket};

const LOCAL: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0);
const OWNERS: [u8; 2] = [17, 23];
const BACKEND: UdpBackend = UdpBackend::Portable;

fn v4(addr: SocketAddr) -> SocketAddrV4 {
    let SocketAddr::V4(addr) = addr else {
        panic!("expected IPv4");
    };
    addr
}

struct Fixture {
    prelogin: Endpoint,
    relay: Endpoint,
    prelogin_cert: Vec<u8>,
    relay_cert: Vec<u8>,
}

impl Fixture {
    fn new() -> Self {
        let (prelogin_cert, key) = crate::launch_tls::generate_identity().unwrap();
        let prelogin = crate::server_endpoint(
            server_config(prelogin_cert.clone(), key.to_vec()).unwrap(),
            LOCAL.into(),
        )
        .unwrap();
        let identity = Identity::ephemeral().unwrap();
        let relay_cert = identity.certificate.to_vec();
        let relay = identity.listen(LOCAL.into()).unwrap();
        Self {
            prelogin,
            relay,
            prelogin_cert,
            relay_cert,
        }
    }

    async fn offer(&self, port: u16, guest: Option<&str>, advertised: SocketAddrV4) -> Connection {
        let native = self.prelogin.accept().await.unwrap().await.unwrap();
        if let Some(guest) = guest {
            assert_eq!(friends::read_name(&native).await.unwrap(), guest);
        }
        let (mut reply, mut request) = native.accept_bi().await.unwrap();
        assert_eq!(accept_header(&mut request, &[port]).await.unwrap(), port);
        assert_eq!(request.read_u8().await.unwrap(), 42);
        let mut out = native.open_uni().await.unwrap();
        out.write_u8(2).await.unwrap();
        for owner in OWNERS {
            out.write_all(&advertised.ip().octets()).await.unwrap();
            out.write_u16(advertised.port()).await.unwrap();
            out.write_u16(self.relay_cert.len() as u16).await.unwrap();
            out.write_all(&self.relay_cert).await.unwrap();
            out.write_all(&[owner; 32]).await.unwrap();
        }
        out.finish().unwrap();
        reply.write_u8(43).await.unwrap();
        reply.finish().unwrap();
        native
    }

    async fn exercise(
        &self,
        native: &UdpSocket,
        assert_shared_socket: bool,
    ) -> (Vec<SocketAddrV4>, Vec<Connection>) {
        let mut jobs = JoinSet::new();
        for _ in OWNERS {
            let incoming = self.relay.accept().await.unwrap();
            jobs.spawn(async {
                let conn = incoming.await.unwrap();
                let (link, owner) = Link::accept_with(conn.clone(), |candidate| {
                    OWNERS.into_iter().find(|owner| *candidate == [*owner; 32])
                })
                .await
                .unwrap();
                (link, owner, conn)
            });
        }
        let mut configured = Vec::new();
        while let Some(accepted) = jobs.join_next().await {
            let (mut link, owner, conn) = accepted.unwrap();
            let (mut tx, mut rx) = link.take_control().unwrap();
            let scope = Scope {
                service: [1; 16],
                room: 1,
                room_session: 1,
                room_generation: 1,
                round: 1,
            };
            let route = RouteId([owner; 16]);
            control::write(
                &mut tx,
                &control::Server::Configure {
                    scope,
                    routes: vec![Assignment { slot: owner, route }],
                },
            )
            .await
            .unwrap();
            let control::Client::Ready {
                scope: ready,
                bindings,
            } = control::read(&mut rx).await.unwrap();
            assert_eq!(scope, ready);
            assert_eq!(bindings.len(), 1);
            assert_eq!(bindings[0].route, route);
            configured.push((owner, bindings[0].address, link, conn, tx, rx));
        }
        configured.sort_by_key(|entry| entry.0);
        assert_eq!(
            configured.iter().map(|entry| entry.0).collect::<Vec<_>>(),
            OWNERS
        );
        assert_ne!(configured[0].1, configured[1].1);
        if assert_shared_socket {
            assert_eq!(
                configured[0].3.remote_address(),
                configured[1].3.remote_address()
            );
        }
        for (owner, address, ..) in &configured {
            native.send_to(&[*owner], address).await.unwrap();
        }
        // Send both first, then verify each Link received only its own traffic.
        for (owner, _, link, ..) in &configured {
            let packet = link.receive().await.unwrap();
            assert_eq!(packet.route, RouteId([*owner; 16]));
            assert_eq!(packet.native, [*owner]);
            link.send(&Packet {
                native: vec![*owner + 1],
                ..packet
            })
            .unwrap();
        }
        let mut replies = Vec::new();
        for _ in OWNERS {
            let mut bytes = [0; 4];
            let (count, source) = native.recv_from(&mut bytes).await.unwrap();
            assert_eq!(count, 1);
            replies.push((bytes[0], v4(source)));
        }
        replies.sort();
        assert_eq!(
            replies,
            [
                (OWNERS[0] + 1, configured[0].1),
                (OWNERS[1] + 1, configured[1].1)
            ]
        );

        // Keep the links and their control streams live until the companion stops.
        let ports = configured.iter().map(|entry| entry.1).collect();
        let connections = configured
            .iter()
            .map(|entry| entry.3.clone())
            .collect::<Vec<_>>();
        tokio::spawn(async move {
            for conn in configured {
                conn.3.closed().await;
            }
        });
        (ports, connections)
    }
}

async fn local_request(address: SocketAddr) {
    let mut tcp = TcpStream::connect(address).await.unwrap();
    tcp.write_u8(42).await.unwrap();
    assert_eq!(tcp.read_u8().await.unwrap(), 43);
}

async fn stopped(address: SocketAddr, udp_ports: Vec<SocketAddrV4>, owners: Vec<Connection>) {
    for owner in owners {
        owner.closed().await;
    }
    TcpListener::bind(address).await.unwrap();
    for addr in udp_ports {
        loop {
            match UdpSocket::bind(addr).await {
                Ok(_) => break,
                Err(error) if error.kind() == io::ErrorKind::AddrInUse => {
                    tokio::task::yield_now().await;
                }
                Err(error) => panic!("{error}"),
            }
        }
    }
}

#[tokio::test]
async fn o1_two_offers_keep_both_peers_and_datagram_routes_separate() {
    tokio::time::timeout(Duration::from_secs(15), async {
        let fixture = Fixture::new();
        let listener = TcpListener::bind(LOCAL).await.unwrap();
        let address = listener.local_addr().unwrap();
        let host = room::endpoint(false).await.unwrap();
        let guest = room::endpoint(false).await.unwrap();
        let route = room::Route {
            node: host.id(),
            addresses: vec![
                SocketAddrV4::new(Ipv4Addr::LOCALHOST, host.bound_sockets()[0].port()).into(),
            ],
            relay: None,
        };
        let gateway = tokio::spawn(friends::serve(
            host.clone(),
            friends::Gateway {
                server: v4(fixture.prelogin.local_addr().unwrap()),
                certificate: fixture.prelogin_cert.clone(),
                ports: vec![address.port()],
                package: [1; 32],
                backend: BACKEND,
            },
        ));
        let connection = friends::join(
            &guest,
            friends::Guest {
                code: Invite::room(route.node).to_string(),
                route,
                package: [1; 32],
                name: "Guest01".into(),
            },
        )
        .await
        .unwrap();
        let (halt, stop) = watch::channel(false);
        let companion = tokio::spawn(friends::companion(connection, vec![listener], stop));
        let (tls, ()) = tokio::join!(
            fixture.offer(
                address.port(),
                Some("Guest01"),
                v4(fixture.relay.local_addr().unwrap())
            ),
            local_request(address),
        );
        let native = UdpSocket::bind(LOCAL).await.unwrap();
        let (ports, owners) = fixture.exercise(&native, false).await;
        halt.send_replace(true);
        companion.await.unwrap().unwrap();
        stopped(address, ports, owners).await;
        drop(tls);
        gateway.abort();
        guest.close().await;
        host.close().await;
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn s1_prelogin_uses_only_the_local_relay_lane_for_both_offers() {
    use iroh_relay::{
        RelayConfig,
        server::{RelayConfig as RelayServer, Server, ServerConfig},
    };
    use kr_p2p::RelayMode;

    tokio::time::timeout(Duration::from_secs(20), async {
        let mut config = ServerConfig::default();
        config.relay = Some(RelayServer::new(LOCAL));
        let relay = Server::spawn(config).await.unwrap();
        let relay_url: kr_p2p::RelayUrl = format!("http://{}", relay.http_addr().unwrap())
            .parse()
            .unwrap();
        let mode = RelayMode::Custom(RelayConfig::new(relay_url.clone(), None).into());
        let network = self_host::Network::Private {
            relay: relay_url,
            quic_port: 0,
            ca_der: vec![],
        };
        let host =
            kr_p2p::endpoint_builder(LOCAL.into(), mode.clone(), Some(SecretKey::generate()))
                .unwrap()
                .clear_ip_transports()
                .alpns(vec![b"boom-kr/s1/1".to_vec()])
                .bind()
                .await
                .unwrap();
        let guest = kr_p2p::endpoint_builder(LOCAL.into(), mode, None)
            .unwrap()
            .clear_ip_transports()
            .bind()
            .await
            .unwrap();
        assert!(host.bound_sockets().is_empty() && guest.bound_sockets().is_empty());
        let fixture = Fixture::new();
        let services = self_host::Services {
            prelogin: v4(fixture.prelogin.local_addr().unwrap()),
            relay: v4(fixture.relay.local_addr().unwrap()),
            launcher: None,
            name: "Local S1 relay test".into(),
            certificate: fixture.prelogin_cert.clone(),
        };
        let invite = Invite::server(host.id(), &fixture.prelogin_cert);
        let owner = host.clone();
        let service = tokio::spawn(async move { self_host::serve(&owner, services).await });
        host.online().await;
        let session = self_host::ClientSession::open(&guest, &network, &invite, None)
            .await
            .unwrap();

        // Even in the same process, the advertised endpoint is unrouteable:
        // this socket receives no packets unless the mapping is bypassed.
        let trap = UdpSocket::bind(LOCAL).await.unwrap();
        let listener = TcpListener::bind(LOCAL).await.unwrap();
        let address = listener.local_addr().unwrap();
        let ep = endpoint(session.certificate.clone(), BACKEND).unwrap();
        let mapping = RelayBridge::new(session.prelogin.into(), session.relay).unwrap();
        let (halt, stop) = watch::channel(false);
        let companion = tokio::spawn(companion_with_relay(
            ep.clone(),
            session.prelogin.into(),
            Some(mapping),
            vec![listener],
            stop,
            BACKEND,
            Trace::default(),
        ));
        let (tls, ()) = tokio::join!(
            fixture.offer(address.port(), None, v4(trap.local_addr().unwrap())),
            local_request(address),
        );
        let native = UdpSocket::bind(LOCAL).await.unwrap();
        let (ports, owners) = fixture.exercise(&native, true).await;
        assert_eq!(
            trap.try_recv(&mut [0; 1]).unwrap_err().kind(),
            io::ErrorKind::WouldBlock
        );
        halt.send_replace(true);
        companion.await.unwrap().unwrap();
        stopped(address, ports, owners).await;
        drop(tls);
        drop(session);
        ep.close(0u8.into(), b"test ended");
        guest.close().await;
        host.close().await;
        service.abort();
        relay.shutdown().await.unwrap();
    })
    .await
    .unwrap();
}
