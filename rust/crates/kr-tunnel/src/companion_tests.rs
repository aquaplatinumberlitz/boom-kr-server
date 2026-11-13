use crate::{
    Identity, Link, Packet, RouteId, Secret, client, companion, control,
    router::{Assignment, Scope},
};
use std::{io, time::Duration};
use tokio::{
    net::UdpSocket,
    sync::{mpsc, watch},
};

fn scope(round: u64) -> Scope {
    Scope {
        service: [3; 16],
        room: 1,
        room_session: 1,
        room_generation: 1,
        round,
    }
}
#[tokio::test]
async fn dynamic_rounds_quarantine_old_native_endpoints_and_drop_old_route_data() {
    let identity = Identity::ephemeral().unwrap();
    let cert = identity.certificate.clone();
    let server = identity.listen("127.0.0.1:0".parse().unwrap()).unwrap();
    let address = server.local_addr().unwrap();
    let endpoint = client("127.0.0.1:0".parse().unwrap(), cert).unwrap();
    let secret = Secret::from_bytes([55; 32]);
    let a = RouteId::random().unwrap();
    let b = RouteId::random().unwrap();
    let (bindings, mut ports) = mpsc::channel(2);
    let (advance, mut moves) = mpsc::channel(1);
    let server_task = tokio::spawn(async move {
        let mut link = Link::accept(server.accept().await.unwrap().await.unwrap(), &secret)
            .await
            .unwrap();
        let (mut send, mut recv) = link.take_control().unwrap();
        for (round, route) in [(1, a), (257, b)] {
            control::write(
                &mut send,
                &control::Server::Configure {
                    scope: scope(round),
                    routes: vec![Assignment { slot: 7, route }],
                },
            )
            .await
            .unwrap();
            let control::Client::Ready {
                scope: received,
                bindings: ready,
            } = control::read(&mut recv).await.unwrap();
            assert_eq!(received, scope(round));
            assert_eq!(ready.len(), 1);
            assert_eq!(ready[0].route, route);
            bindings.send(ready[0].address).await.unwrap();
            let packet = tokio::time::timeout(Duration::from_secs(2), link.receive())
                .await
                .unwrap()
                .unwrap();
            assert_eq!(packet.route, route);
            assert_eq!(packet.sequence, 1);
            assert_eq!(
                packet.native,
                if round == 1 {
                    b"first".to_vec()
                } else {
                    b"current".to_vec()
                }
            );
            if round != 1 {
                link.send(&Packet {
                    route: a,
                    sequence: 2,
                    native: b"old encrypted route".to_vec(),
                })
                .unwrap();
            }
            link.send(&packet).unwrap();
            moves.recv().await.unwrap();
            control::write(
                &mut send,
                &control::Server::Retire {
                    scope: scope(round),
                },
            )
            .await
            .unwrap();
        }
        let _ = link.receive().await;
        server.close(0_u8.into(), b"test end");
    });
    let link = Link::connect(&endpoint, address, &Secret::from_bytes([55; 32]))
        .await
        .unwrap();
    let (stop, recv) = watch::channel(false);
    let proxy = tokio::spawn(companion::run(link, "127.0.0.1".parse().unwrap(), 26, recv));
    let native = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let mut bytes = [0; 256];
    let old = ports.recv().await.unwrap();
    native.send_to(b"first", old).await.unwrap();
    let (n, from) = tokio::time::timeout(Duration::from_secs(2), native.recv_from(&mut bytes))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(from, std::net::SocketAddr::V4(old));
    assert_eq!(&bytes[..n], b"first");
    advance.send(()).await.unwrap();
    let new = ports.recv().await.unwrap();
    assert_ne!(old, new);
    assert_eq!(
        UdpSocket::bind(old).await.unwrap_err().kind(),
        io::ErrorKind::AddrInUse
    );
    native.send_to(b"stale native bytes", old).await.unwrap();
    native.send_to(b"current", new).await.unwrap();
    let (n, from) = tokio::time::timeout(Duration::from_secs(2), native.recv_from(&mut bytes))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(from, std::net::SocketAddr::V4(new));
    assert_eq!(&bytes[..n], b"current");
    advance.send(()).await.unwrap();
    stop.send(true).unwrap();
    proxy.await.unwrap().unwrap();
    assert!(UdpSocket::bind(old).await.is_ok());
    assert!(UdpSocket::bind(new).await.is_ok());
    server_task.await.unwrap();
    endpoint.close(0_u8.into(), b"end");
}
