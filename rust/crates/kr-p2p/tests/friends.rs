use kr_p2p::{
    CodeError, Invite,
    room::{self, Discovery, JoinError, Route},
};
use std::{net::Ipv4Addr, time::Duration};

#[tokio::test(flavor = "multi_thread")]
async fn two_launcher_endpoints_find_verify_and_close_one_room() {
    let host = room::endpoint(false).await.unwrap();
    let guest = room::endpoint(false).await.unwrap();
    let code = Invite::room(host.id());
    let port = host.bound_sockets()[0].port();
    let advertise = Discovery::new().unwrap();
    advertise.advertise(host.id(), port).unwrap();
    let discover = Discovery::new().unwrap();
    let route = discover
        .rooms(Duration::from_secs(3))
        .await
        .unwrap()
        .into_iter()
        .find(|route| route.node == host.id())
        .expect("LAN DNS-SD discovery");
    assert_eq!(Invite::room(route.node), code);
    assert_eq!(
        "O1-42-0000-0000-0000-0000".parse::<Invite>(),
        Err(CodeError::Version(42))
    );
    assert_eq!("invalid".parse::<Invite>(), Err(CodeError::Format));

    let fake = Invite::room(guest.id());
    assert!(matches!(
        room::join(&guest, &fake, &route, [0; 32], "GuestOne").await,
        Err(JoinError::Unavailable)
    ));
    let local = Route {
        node: host.id(),
        addresses: vec![(Ipv4Addr::LOCALHOST, port).into()],
        relay: None,
    };
    let accept = || async {
        let connection = host.accept().await.unwrap().await.unwrap();
        let result = room::admit(&connection, host.id(), [2; 32], |name| name == "GuestOne").await;
        (connection, result)
    };
    let (join, (_incoming, admitted)) = tokio::join!(
        room::join(&guest, &code, &local, [3; 32], "GuestOne"),
        accept()
    );
    assert!(matches!(join, Err(JoinError::Version)));
    assert!(admitted.is_err());

    let (join, (incoming, admitted)) = tokio::join!(
        room::join(&guest, &code, &route, [2; 32], "GuestOne"),
        accept()
    );
    assert_eq!(admitted.unwrap(), "GuestOne");
    let session = join.unwrap();
    assert_eq!(incoming.remote_id(), guest.id());
    host.close().await;
    tokio::time::timeout(Duration::from_secs(3), session.closed())
        .await
        .unwrap();
    guest.close().await;
}
