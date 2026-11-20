use super::*;
fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
}

#[test]
fn active_traffic_renews_only_its_own_allowance_and_never_renews_preauth() {
    let now = Instant::now();
    let mut budget = TrafficBudget::new(BYTE_LIMIT, 64 * 1024);
    assert!(budget.allow(BYTE_LIMIT, false, now));
    assert!(!budget.allow(1, false, now + Duration::from_secs(30)));
    assert!(budget.allow(64 * 1024, true, now));
    assert!(!budget.allow(1, true, now + Duration::from_millis(999)));
    assert!(budget.allow(64 * 1024, true, now + Duration::from_secs(1)));
    assert!(!budget.allow(1, false, now + Duration::from_secs(60)));
}

#[test]
fn two_peer_offers_stay_inside_original_tls_and_stop_closes_both_udp_owners() {
    runtime().block_on(async {
        let identity = crate::Identity::ephemeral().unwrap();
        let certificate = identity.certificate.as_ref().to_vec();
        let peer = identity.listen("127.0.0.1:0".parse().unwrap()).unwrap();
        let address = peer.local_addr().unwrap();
        let secret = crate::Secret::random().unwrap();
        let bytes = Zeroizing::new(*secret.expose());
        let (accepted, mut incoming) = tokio::sync::mpsc::channel(2);
        let endpoint = peer.clone();
        let peer_server = tokio::spawn(async move {
            let mut owners = JoinSet::new();
            for _ in 0..2 {
                let connection = endpoint.accept().await.unwrap().await.unwrap();
                let secret = crate::Secret::from_bytes(*bytes);
                let accepted = accepted.clone();
                owners.spawn(async move {
                    let (link, ()) = crate::Link::accept_with_async(
                        connection.clone(),
                        |candidate| async move { candidate.accepts(secret.expose()).then_some(()) },
                    )
                    .await
                    .unwrap();
                    accepted.send(()).await.unwrap();
                    connection.closed().await;
                    drop(link);
                });
            }
            while let Some(result) = owners.join_next().await {
                result.unwrap();
            }
        });
        let (cert, key) = crate::launch_tls::generate_identity().unwrap();
        let server = Endpoint::server(
            server_config(cert.clone(), key.to_vec()).unwrap(),
            "127.0.0.1:0".parse().unwrap(),
        )
        .unwrap();
        let mut client = Endpoint::client("127.0.0.1:0".parse().unwrap()).unwrap();
        client.set_default_client_config(client_config(cert).unwrap());
        let connect = connect(&client, server.local_addr().unwrap());
        let accept = async { server.accept().await.unwrap().await.unwrap() };
        let (connected, remote) = tokio::join!(connect, accept);
        let connection = connected.unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let (halt, stop) = watch::channel(false);
        let task = tokio::spawn(companion(connection, vec![listener], stop));
        let mut send = remote.open_uni().await.unwrap();
        send.write_u8(2).await.unwrap();
        for _ in 0..2 {
            send.write_all(&[127, 0, 0, 1]).await.unwrap();
            send.write_u16(address.port()).await.unwrap();
            send.write_u16(certificate.len() as u16).await.unwrap();
            send.write_all(&certificate).await.unwrap();
            send.write_all(secret.expose()).await.unwrap();
        }
        send.finish().unwrap();
        for _ in 0..2 {
            tokio::time::timeout(Duration::from_secs(3), incoming.recv())
                .await
                .unwrap()
                .unwrap();
        }
        halt.send_replace(true);
        task.await.unwrap().unwrap();
        tokio::time::timeout(Duration::from_secs(3), peer_server)
            .await
            .unwrap()
            .unwrap();
        let _rebound = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, port))
            .await
            .unwrap();
        server.close(0u8.into(), b"done");
        client.close(0u8.into(), b"done");
        peer.close(0u8.into(), b"done");
    });
}

#[test]
fn malformed_or_partial_peer_offers_close_the_whole_native_transport() {
    runtime().block_on(async {
        for bytes in [
            vec![0],
            vec![3],
            vec![1],
            vec![1, 127, 0, 0, 1, 0, 1, 0x10, 1],
        ] {
            let (cert, key) = crate::launch_tls::generate_identity().unwrap();
            let server = Endpoint::server(
                server_config(cert.clone(), key.to_vec()).unwrap(),
                "127.0.0.1:0".parse().unwrap(),
            )
            .unwrap();
            let mut client = Endpoint::client("127.0.0.1:0".parse().unwrap()).unwrap();
            client.set_default_client_config(client_config(cert).unwrap());
            let (connected, remote) =
                tokio::join!(connect(&client, server.local_addr().unwrap()), async {
                    server.accept().await.unwrap().await.unwrap()
                });
            let connection = connected.unwrap();
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let port = listener.local_addr().unwrap().port();
            let (_halt, stop) = watch::channel(false);
            let task = tokio::spawn(companion(connection, vec![listener], stop));
            let mut send = remote.open_uni().await.unwrap();
            send.write_all(&bytes).await.unwrap();
            // Keep truncated offers open: the offer deadline, rather than EOF,
            // must end the native owner and release its loopback sockets.
            let result = tokio::time::timeout(Duration::from_secs(3), task)
                .await
                .unwrap()
                .unwrap();
            assert!(result.is_err());
            let _rebound = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, port))
                .await
                .unwrap();
            server.close(0u8.into(), b"done");
            client.close(0u8.into(), b"done");
        }
    });
}
#[test]
fn companion_starts_before_login_and_stop_releases_every_local_socket() {
    runtime().block_on(async {
        let local = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = local.local_addr().unwrap().port();
        let (cert, key) = crate::launch_tls::generate_identity().unwrap();
        let server = Endpoint::server(
            server_config(cert.clone(), key.to_vec()).unwrap(),
            "127.0.0.1:0".parse().unwrap(),
        )
        .unwrap();
        let mut client = Endpoint::client("127.0.0.1:0".parse().unwrap()).unwrap();
        client.set_default_client_config(client_config(cert).unwrap());
        let address = server.local_addr().unwrap();
        let endpoint = server.clone();
        let task = tokio::spawn(async move {
            let tls = endpoint.accept().await.unwrap().await.unwrap();
            let (mut send, mut recv) = tls.accept_bi().await.unwrap();
            accept_header(&mut recv, &[port]).await.unwrap();
            send.write_all(b"ready").await.unwrap();
            let mut request = [0; 5];
            recv.read_exact(&mut request).await.unwrap();
            assert_eq!(&request, b"login");
            send.write_all(b"bound").await.unwrap();
            tls.closed().await;
        });
        let run = std::env::temp_dir().join(format!(
            "kr-tls-diagnostics-{}-{:x}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&run).unwrap();
        let trace = Trace::create(&run).unwrap();
        let (halt, stop) = watch::channel(false);
        let companion = tokio::spawn(companion_traced(
            client.clone(),
            address,
            vec![local],
            stop,
            crate::UdpBackend::Native,
            trace,
        ));
        let mut socket = tokio::net::TcpStream::connect((std::net::Ipv4Addr::LOCALHOST, port))
            .await
            .unwrap();
        let mut hello = [0; 5];
        socket.read_exact(&mut hello).await.unwrap();
        assert_eq!(&hello, b"ready");
        socket.write_all(b"login").await.unwrap();
        socket.read_exact(&mut hello).await.unwrap();
        assert_eq!(&hello, b"bound");
        halt.send_replace(true);
        companion.await.unwrap().unwrap();
        let log = std::fs::read_to_string(run.join(trace::LOG_NAME)).unwrap();
        for label in [
            "event=tcp_accept",
            "event=tls_connect_ok",
            "event=stream_open_ok",
            "up=5 down=10",
            "reason=stop",
        ] {
            assert!(log.contains(label), "missing {label}");
        }
        assert!(!log.contains("login") && !log.contains("ready") && !log.contains("bound"));
        std::fs::remove_dir_all(run).unwrap();
        match socket.read(&mut hello).await {
            Ok(0) => {}
            Err(error) if error.kind() == io::ErrorKind::ConnectionReset => {}
            result => panic!("local socket remained open: {result:?}"),
        }
        let _rebound = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, port))
            .await
            .unwrap();
        task.await.unwrap();
        server.close(0u8.into(), b"done");
        client.close(0u8.into(), b"done");
    });
}

#[test]
fn companion_preserves_account_session_end_from_the_pinned_service() {
    runtime().block_on(async {
        let (cert, key) = crate::launch_tls::generate_identity().unwrap();
        let server = Endpoint::server(
            server_config(cert.clone(), key.to_vec()).unwrap(),
            "127.0.0.1:0".parse().unwrap(),
        )
        .unwrap();
        let client = endpoint(cert, crate::UdpBackend::Native).unwrap();
        for code in [0, ACCOUNT_SESSION_ENDED] {
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let addr = listener.local_addr().unwrap();
            let (halt, stop) = watch::channel(false);
            let worker = tokio::spawn(companion_traced(
                client.clone(),
                server.local_addr().unwrap(),
                vec![listener],
                stop,
                crate::UdpBackend::Native,
                Trace::default(),
            ));
            let _game = tokio::net::TcpStream::connect(addr).await.unwrap();
            let tls = server.accept().await.unwrap().await.unwrap();
            let (_send, mut recv) = tls.accept_bi().await.unwrap();
            accept_header(&mut recv, &[addr.port()]).await.unwrap();
            tls.close(code.into(), b"test");
            let error = tokio::time::timeout(Duration::from_secs(3), worker)
                .await
                .unwrap()
                .unwrap()
                .unwrap_err();
            assert_eq!(
                error
                    .get_ref()
                    .is_some_and(|e| e.is::<AccountSessionEnded>()),
                code == ACCOUNT_SESSION_ENDED
            );
            drop(halt);
        }
    });
}

#[test]
fn pinned_tls_opens_only_declared_endpoints_and_foreign_certificates_fail() {
    runtime().block_on(async {
        let (cert, key) = crate::launch_tls::generate_identity().unwrap();
        let server = Endpoint::server(
            server_config(cert.clone(), key.to_vec()).unwrap(),
            "127.0.0.1:0".parse().unwrap(),
        )
        .unwrap();
        let mut client = Endpoint::client("127.0.0.1:0".parse().unwrap()).unwrap();
        client.set_default_client_config(client_config(cert).unwrap());
        let address = server.local_addr().unwrap();
        let server_task = tokio::spawn(async move {
            let conn = server.accept().await.unwrap().await.unwrap();
            let (mut send, mut recv) = conn.accept_bi().await.unwrap();
            assert_eq!(accept_header(&mut recv, &[3838]).await.unwrap(), 3838);
            let mut message = [0; 5];
            recv.read_exact(&mut message).await.unwrap();
            assert_eq!(&message, b"login");
            send.write_all(b"hello").await.unwrap();
            send.finish().unwrap();
            let (_, mut denied) = conn.accept_bi().await.unwrap();
            assert!(accept_header(&mut denied, &[3838]).await.is_err());
            conn.close(0u8.into(), b"test done");
            server.close(0u8.into(), b"test done");
        });
        let conn = connect(&client, address).await.unwrap();
        let (mut send, mut recv) = open(&conn, 3838).await.unwrap();
        send.write_all(b"login").await.unwrap();
        let mut reply = [0; 5];
        recv.read_exact(&mut reply).await.unwrap();
        assert_eq!(&reply, b"hello");
        let (_send, _recv) = open(&conn, 22).await.unwrap();
        server_task.await.unwrap();
        client.close(0u8.into(), b"done");
        let (cert, key) = crate::launch_tls::generate_identity().unwrap();
        let (foreign, _) = crate::launch_tls::generate_identity().unwrap();
        let server = Endpoint::server(
            server_config(cert, key.to_vec()).unwrap(),
            "127.0.0.1:0".parse().unwrap(),
        )
        .unwrap();
        let mut client = Endpoint::client("127.0.0.1:0".parse().unwrap()).unwrap();
        client.set_default_client_config(client_config(foreign).unwrap());
        let address = server.local_addr().unwrap();
        let observed = tokio::spawn(async move {
            let result = server.accept().await.unwrap().await;
            assert!(result.is_err());
            server.close(0u8.into(), b"done");
        });
        assert!(connect(&client, address).await.is_err());
        client.close(0u8.into(), b"done");
        observed.await.unwrap();
    });
}
