use kr_p2p::{CodeError, EndpointAddr, Invite, MAX_GAME_PACKET, RelayMode, SecretKey};
use std::{io, time::Duration};
use tokio::{net::UdpSocket, time::timeout};

#[test]
fn code_kinds_version_and_pins() {
    let node = SecretKey::from_bytes(&[7; 32]).public();
    let other = SecretKey::from_bytes(&[8; 32]).public();
    let room = Invite::room(node);
    let server = Invite::server(node, b"abc");
    let pin = [
        0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae, 0x22,
        0x23, 0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61, 0xf2, 0x00,
        0x15, 0xad,
    ];
    assert_eq!(
        server,
        Invite::Server {
            node,
            tls_sha256: pin
        }
    );
    for (invite, prefix) in [(room, "O1-1-"), (server, "S1-1-")] {
        let text = invite.to_string();
        assert!(text.starts_with(prefix));
        assert_eq!(text.parse::<Invite>(), Ok(invite.clone()));
        assert!(invite.matches_node(node));
        assert!(!invite.matches_node(other));
        assert_eq!(
            text.replacen("-1-", "-2-", 1).parse::<Invite>(),
            Err(CodeError::Version(2))
        );
    }
    let room = "O1-1-00ll-Iioo-k7m2-4qx8".parse::<Invite>().unwrap();
    assert_eq!(room.to_string(), "O1-1-0011-1100-K7M2-4QX8");
    for malformed in [
        "O1-1-K7M-2QX",
        "O1-1-0000-0000-0000-000U",
        "S1-1-abc",
        "X1-1-abc",
    ] {
        assert_eq!(malformed.parse::<Invite>(), Err(CodeError::Format));
    }
}

async fn local_endpoint() -> kr_p2p::Endpoint {
    kr_p2p::endpoint_builder("127.0.0.1:0".parse().unwrap(), RelayMode::Disabled, None)
        .unwrap()
        .bind()
        .await
        .unwrap()
}

async fn udp_pair() -> (UdpSocket, UdpSocket) {
    let a = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let b = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    a.connect(b.local_addr().unwrap()).await.unwrap();
    b.connect(a.local_addr().unwrap()).await.unwrap();
    (a, b)
}

#[tokio::test]
async fn pinned_direct_connection_preserves_udp_boundaries_both_ways() {
    timeout(Duration::from_secs(10), async {
        let (host, guest) = tokio::join!(local_endpoint(), local_endpoint());
        let route = EndpointAddr::new(host.id()).with_ip_addr(host.bound_sockets()[0]);
        let code = Invite::room(host.id()).to_string().parse().unwrap();
        let wrong = Invite::room(guest.id());
        let rejected = kr_p2p::connect(&guest, &wrong, route.clone()).await;
        assert!(matches!(rejected, Err(e) if e.kind() == io::ErrorKind::PermissionDenied));

        let (host_result, guest_result) = tokio::join!(
            kr_p2p::accept(&host, guest.id()),
            kr_p2p::connect(&guest, &code, route)
        );
        let host_conn = host_result.unwrap();
        let guest_conn = guest_result.unwrap();
        assert_eq!(guest_conn.remote_id(), host.id());
        assert!(
            guest_conn
                .paths()
                .iter()
                .any(|path| path.is_ip() && path.is_selected())
        );

        let ((native_host, host_udp), (native_guest, guest_udp)) =
            tokio::join!(udp_pair(), udp_pair());
        let host_task = tokio::spawn(async move { kr_p2p::bridge(&host_conn, host_udp).await });
        let guest_task = tokio::spawn(async move { kr_p2p::bridge(&guest_conn, guest_udp).await });

        // If the oversized packet were forwarded truncated, it could arrive
        // before or after the marker. Check for any unexpected extra packet.
        let over = [0x55; MAX_GAME_PACKET + 128];
        native_guest.send(&over).await.unwrap();
        let mut buf = [0; MAX_GAME_PACKET + 1];
        let request = [0x13, 0, 0x88, 0xff];
        native_guest.send(&request).await.unwrap();
        let size = native_host.recv(&mut buf).await.unwrap();
        assert_eq!(&buf[..size], &request);
        assert!(
            timeout(Duration::from_millis(100), native_host.recv(&mut buf))
                .await
                .is_err()
        );
        let response = [0xa5; MAX_GAME_PACKET];
        native_host.send(&response).await.unwrap();
        let size = native_guest.recv(&mut buf).await.unwrap();
        assert_eq!(&buf[..size], &response);

        host_task.abort();
        guest_task.abort();
        tokio::join!(host.close(), guest.close());
    })
    .await
    .expect("local P2P deadline");
}

#[tokio::test]
async fn host_rejects_an_unapproved_iroh_identity() {
    timeout(Duration::from_secs(10), async {
        let (host, guest) = tokio::join!(local_endpoint(), local_endpoint());
        let route = EndpointAddr::new(host.id()).with_ip_addr(host.bound_sockets()[0]);
        let invite = Invite::server(host.id(), b"synthetic DER bytes");
        let (incoming, outgoing) = tokio::join!(
            kr_p2p::accept(&host, host.id()),
            kr_p2p::connect(&guest, &invite, route)
        );
        assert!(matches!(incoming, Err(e) if e.kind() == io::ErrorKind::PermissionDenied));
        if let Ok(conn) = outgoing {
            conn.closed().await;
        }
        tokio::join!(host.close(), guest.close());
    })
    .await
    .expect("local rejection deadline");
}
