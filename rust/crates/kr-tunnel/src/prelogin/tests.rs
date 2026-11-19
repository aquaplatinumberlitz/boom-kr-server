use super::*;
fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
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
        let connection = connect(&client, address).await.unwrap();
        let (halt, stop) = watch::channel(false);
        let companion = tokio::spawn(companion(connection, vec![local], stop));
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
