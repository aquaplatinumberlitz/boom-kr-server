//! Loopback-only account acceptance harness; never a public password endpoint.
//! One isolated native client, through the channel handoff. Lobby/gameplay and
//! prelogin TLS remain separate acceptance parts. No credentials in diagnostics.
use kr_service::{
    LobbyPacketPin,
    account_bridge::local::{
        LocalSession,
        native::{self, Driver, Lane, Templates},
    },
    account_worker::Worker,
};
use serde::Deserialize;
use std::{io, net::Ipv4Addr, path::PathBuf, sync::Arc, time::Duration};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::{Mutex, Semaphore, mpsc},
    task::JoinSet,
};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    profile: PathBuf,
    database: PathBuf,
    netinfo: LobbyPacketPin,
    redirect: LobbyPacketPin,
    seconds: u64,
}
async fn connection(
    mut socket: TcpStream,
    id: u64,
    hello: &[u8],
    wcp: &[u8],
    driver: Arc<Mutex<Option<Driver>>>,
    client: kr_service::account_worker::Client,
    templates: Templates,
) -> io::Result<()> {
    socket.write_all(hello).await?;
    let mut sequence = 40;
    let first = native::read(&mut socket, &mut sequence).await?;
    println!(
        "{{\"event\":\"first_request\",\"connection\":{id},\"opcode\":{}}}",
        first.opcode
    );
    if first.opcode == 0 {
        socket.write_all(wcp).await?;
        let mut extra = [0; 1];
        let _ = tokio::time::timeout(Duration::from_secs(30), socket.read(&mut extra)).await;
        return Ok(());
    }
    let mut lane = match first.opcode {
        1 => Lane::login(id)?,
        6 => Lane::channel(id)?,
        _ => return Err(io::Error::other("first opcode")),
    };
    let mut pending = Some(first);
    let result=async {
        loop {
            let input=if let Some(input)=pending.take(){input}else{native::read(&mut socket,&mut sequence).await?};
            let opcode=input.opcode;
            let mut state=driver.lock().await;
            if state.is_none() && opcode==1 {*state=Some(Driver::new(LocalSession::new(client.clone(),id).map_err(|_|io::Error::other("owner creation"))?,templates.clone())?);}
            let owned=state.as_mut().ok_or_else(||io::Error::other("no account owner"))?;
            // The account slice intentionally ends at the first lobby request.
            if opcode==67 {
                println!("{{\"event\":\"native_lobby_boundary\",\"connection\":{id}}}");
                return Ok(());
            }
            owned.packet(&mut lane,input,&mut socket,120,None).await?;
            if opcode==6 {
                let players:Vec<_>=owned.authorities().iter().map(|a|a.player().wire_id).collect();
                println!("{{\"event\":\"native_db_channel\",\"connection\":{id},\"players\":{players:?}}}");
            }
        }
    }.await;
    if let Some(owned) = driver.lock().await.as_mut() {
        owned.disconnected(&lane).await?;
    }
    result
}
#[tokio::main(flavor = "current_thread")]
async fn main() -> io::Result<()> {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 2 {
        return Err(io::Error::other("usage: ingame_local PRIVATE_INPUT_JSON"));
    }
    let input: Input = serde_json::from_slice(&std::fs::read(&args[1])?)
        .map_err(|_| io::Error::other("private input"))?;
    if !(10..=600).contains(&input.seconds) {
        return Err(io::Error::other("runtime bound"));
    }
    let profile = kr_service::load_config(&input.profile)?;
    let (hello, wcp) = Templates::startup_frames(&profile);
    let mut templates = Templates::from_profile(&profile)?;
    templates.local_handoff(&input.netinfo, &input.redirect)?;
    let worker = Worker::open(input.database, 8, Duration::from_secs(5))
        .await
        .map_err(|_| io::Error::other("account worker"))?;
    let driver = Arc::new(Mutex::new(None::<Driver>));
    let capacity = Arc::new(Semaphore::new(8));
    let (accepted, mut incoming) = mpsc::channel(8);
    let mut listeners = JoinSet::new();
    for port in (3808..4000).step_by(10) {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, port)).await?;
        let accepted = accepted.clone();
        listeners.spawn(async move {
            loop {
                let (socket, _) = listener.accept().await?;
                let _ = accepted.try_send(socket);
            }
            #[allow(unreachable_code)]
            Ok::<(), io::Error>(())
        });
    }
    drop(accepted);
    let mut connections = JoinSet::new();
    let mut next = 0u64;
    let until = tokio::time::sleep(Duration::from_secs(input.seconds));
    tokio::pin!(until);
    let mut health = tokio::time::interval(Duration::from_secs(1));
    println!("{{\"event\":\"loopback_ready\",\"scope\":\"account_slice_only\"}}");
    loop {
        tokio::select! {
            _=tokio::signal::ctrl_c()=>break,
            _=&mut until=>break,
            _=health.tick()=>{
                if let Some(owned)=driver.lock().await.as_mut() && owned.inspect().await.is_err(){break;}
            },
            _=connections.join_next(),if !connections.is_empty()=>{},
            socket=incoming.recv()=>{
                let Some(socket)=socket else{break};
                let Ok(permit)=capacity.clone().try_acquire_owned()else{continue};
                next+=1;if next>32{continue;}
                let id=next;let driver=driver.clone();let client=worker.client();
                let templates=templates.clone();let hello=hello.clone();let wcp=wcp.clone();
                connections.spawn(async move {let _permit=permit;if let Err(error)=connection(socket,id,&hello,&wcp,driver,client,templates).await {eprintln!("{{\"event\":\"connection_ended\",\"connection\":{id},\"reason\":{error:?}}}");}});
            }
        }
    }
    listeners.abort_all();
    while listeners.join_next().await.is_some() {}
    connections.abort_all();
    while connections.join_next().await.is_some() {}
    if let Some(owned) = driver.lock().await.take() {
        owned.finish().await?;
    }
    worker
        .shutdown()
        .await
        .map_err(|_| io::Error::other("account worker shutdown"))?;
    println!("{{\"event\":\"loopback_stopped\"}}");
    Ok(())
}
