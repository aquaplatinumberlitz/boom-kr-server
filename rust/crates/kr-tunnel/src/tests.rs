use super::*;

#[tokio::test]
async fn loopback_proxy_preserves_udp_tuple_and_filters_replay_and_foreign_route() {
    use crate::proxy::{Binding, Proxy};
    use tokio::{net::UdpSocket, sync::watch};
    let identity = Identity::ephemeral().unwrap();
    let certificate = identity.certificate.clone();
    let server = identity.listen("127.0.0.1:0".parse().unwrap()).unwrap();
    let address = server.local_addr().unwrap();
    let endpoint = client("127.0.0.1:0".parse().unwrap(), certificate).unwrap();
    let route = RouteId::random().unwrap();
    let denied = RouteId::random().unwrap();
    let expected = Secret::from_bytes([9; 32]);
    let hub = tokio::spawn(async move {
        let connection = server.accept().await.unwrap().await.unwrap();
        let link = Link::accept(connection, &expected).await.unwrap();
        let packet = link.receive().await.unwrap();
        assert_eq!(packet.route, route);
        assert_eq!(packet.native, b"native opaque bytes");
        link.send(&Packet {
            route: denied,
            sequence: 900,
            native: b"foreign".to_vec(),
        })
        .unwrap();
        link.send(&Packet {
            route,
            sequence: 1,
            native: packet.native.clone(),
        })
        .unwrap();
        link.send(&Packet {
            route,
            sequence: 1,
            native: b"duplicate".to_vec(),
        })
        .unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;
        link.send(&Packet {
            route,
            sequence: 2,
            native: b"next".to_vec(),
        })
        .unwrap();
        let _ = link.receive().await;
        server.close(0_u8.into(), b"end");
    });
    let link = Link::connect(&endpoint, address, &Secret::from_bytes([9; 32]))
        .await
        .unwrap();
    let proxy = Proxy::bind(
        &[Binding {
            route,
            address: "127.0.0.1:0".parse().unwrap(),
        }],
        "127.0.0.1".parse().unwrap(),
    )
    .await
    .unwrap();
    let local = proxy.bindings().unwrap()[0].address;
    let (stop, recv) = watch::channel(false);
    let task = tokio::spawn(proxy.run(link, recv));
    let native = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    native.send_to(b"native opaque bytes", local).await.unwrap();
    let mut bytes = [0; 256];
    for expected in [b"native opaque bytes".as_slice(), b"next"] {
        let (n, source) =
            tokio::time::timeout(Duration::from_secs(2), native.recv_from(&mut bytes))
                .await
                .unwrap()
                .unwrap();
        assert_eq!(source, local);
        assert_eq!(&bytes[..n], expected);
    }
    stop.send(true).unwrap();
    let stats = task.await.unwrap().unwrap();
    assert_eq!(stats.native_received, 1);
    assert_eq!(stats.native_sent, 2);
    assert_eq!(stats.replay, 1);
    assert_eq!(stats.unknown_route, 1);
    assert!(UdpSocket::bind(local).await.is_ok());
    hub.await.unwrap();
    endpoint.close(0_u8.into(), b"end");
}

#[test]
fn replay_window_allows_bounded_reordering_without_duplicate_or_wrap() {
    let mut w = ReplayWindow::default();
    assert!(!w.admit(0));
    assert!(w.admit(4));
    assert!(w.admit(2));
    assert!(!w.admit(2));
    assert!(w.admit(67));
    assert!(!w.admit(2));
    assert!(w.admit(5));
    assert!(!w.admit(4));
    assert!(w.admit(u64::MAX));
    assert!(!w.admit(67));
    assert!(!w.admit(u64::MAX));
    assert!(!w.admit(0));
}

#[tokio::test]
async fn pinned_quic_roundtrip_preserves_bytes_and_denies_wrong_capability_same_ip() {
    let identity = Identity::ephemeral().unwrap();
    let certificate = identity.certificate.clone();
    let server = identity.listen("127.0.0.1:0".parse().unwrap()).unwrap();
    let address = server.local_addr().unwrap();
    let endpoint = client("127.0.0.1:0".parse().unwrap(), certificate).unwrap();
    let expected = Secret::from_bytes([7; 32]);
    let task = tokio::spawn(async move {
        let bad = server.accept().await.unwrap().await.unwrap();
        assert!(Link::accept(bad, &expected).await.is_err());
        let incoming = server.accept().await.unwrap().await.unwrap();
        let mut link = Link::accept(incoming, &expected).await.unwrap();
        let packet = link.receive().await.unwrap();
        assert_eq!(packet.native, (0..=254).collect::<Vec<_>>());
        link.send(&packet).unwrap();
        let mut finish = [0; 1];
        let (_, mut recv) = link.take_control().unwrap();
        recv.read_exact(&mut finish).await.unwrap();
        assert_eq!(finish, [1]);
        server.close(0_u8.into(), b"test end");
    });
    assert!(
        Link::connect(&endpoint, address, &Secret::from_bytes([8; 32]))
            .await
            .is_err()
    );
    let mut link = Link::connect(&endpoint, address, &Secret::from_bytes([7; 32]))
        .await
        .unwrap();
    let packet = Packet {
        route: RouteId::random().unwrap(),
        sequence: 1,
        native: (0..=254).collect(),
    };
    link.send(&packet).unwrap();
    assert_eq!(
        tokio::time::timeout(Duration::from_secs(3), link.receive())
            .await
            .unwrap()
            .unwrap(),
        packet
    );
    let (mut send, _) = link.take_control().unwrap();
    send.write_all(&[1]).await.unwrap();
    task.await.unwrap();
    endpoint.close(0_u8.into(), b"test end");
}

#[tokio::test]
async fn different_certificate_cannot_impersonate_pinned_server() {
    let identity = Identity::ephemeral().unwrap();
    let wrong = Identity::ephemeral().unwrap().certificate;
    let server = identity.listen("127.0.0.1:0".parse().unwrap()).unwrap();
    let address = server.local_addr().unwrap();
    let endpoint = client("127.0.0.1:0".parse().unwrap(), wrong).unwrap();
    let attempt = endpoint.connect(address, NAME).unwrap();
    let task = tokio::spawn(async move {
        assert!(server.accept().await.unwrap().await.is_err());
        server.close(0_u8.into(), b"end");
    });
    assert!(
        tokio::time::timeout(Duration::from_secs(3), attempt)
            .await
            .unwrap()
            .is_err()
    );
    task.await.unwrap();
}
