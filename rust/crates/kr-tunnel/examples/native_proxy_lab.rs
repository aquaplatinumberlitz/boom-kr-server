//! Isolated transport compatibility experiment only. Static capabilities/routes
//! are declared private runtime inputs; there is no account/room lifecycle here.
//! Never deploy this example as the game server or infer N1R-b acceptance.
use kr_tunnel::{
    Identity, Link, Packet, ReplayWindow, RouteId, Secret, client,
    proxy::{Binding, Proxy},
};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    collections::{HashMap, HashSet},
    io,
    net::{IpAddr, SocketAddr},
    path::PathBuf,
    time::Duration,
};
use tokio::{
    sync::{mpsc, watch},
    task::JoinSet,
};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Route {
    id: String,
    port: u16,
    target: usize,
    target_id: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Peer {
    secret: String,
    routes: Vec<Route>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    server: SocketAddr,
    certificate: PathBuf,
    peers: Vec<Peer>,
    role: Option<usize>,
    native_ip: Option<IpAddr>,
}
fn hex<const N: usize>(s: &str) -> io::Result<[u8; N]> {
    if s.len() != 2 * N {
        return Err(io::Error::other("private lab hex length"));
    }
    let mut bytes = [0; N];
    for (i, part) in s.as_bytes().as_chunks::<2>().0.iter().enumerate() {
        let digit = |b: u8| match b {
            b'0'..=b'9' => Ok(b - b'0'),
            b'a'..=b'f' => Ok(b - b'a' + 10),
            _ => Err(io::Error::other("private lab hex character")),
        };
        bytes[i] = (digit(part[0])? << 4) | digit(part[1])?;
    }
    Ok(bytes)
}
fn emit(event: serde_json::Value) {
    println!("{event}");
}
#[tokio::main]
async fn main() -> io::Result<()> {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 3 {
        return Err(io::Error::other(
            "usage: native_proxy_lab hub|proxy private-config.json",
        ));
    }
    let config: Config =
        serde_json::from_slice(&std::fs::read(&args[2])?).map_err(io::Error::other)?;
    if config.peers.len() < 2 || config.peers.len() > 26 {
        return Err(io::Error::other("lab peer bound"));
    }
    let mut tokens = HashSet::new();
    for peer in &config.peers {
        if !tokens.insert(hex::<32>(&peer.secret)?)
            || peer.routes.is_empty()
            || peer.routes.len() > 26
        {
            return Err(io::Error::other("lab capability/route bound"));
        }
        for route in &peer.routes {
            let id = hex::<16>(&route.id)?;
            let target = hex::<16>(&route.target_id)?;
            if route.target >= config.peers.len()
                || !config.peers[route.target]
                    .routes
                    .iter()
                    .any(|r| hex::<16>(&r.id).ok() == Some(target))
                || id == [0; 16]
            {
                return Err(io::Error::other("lab route target"));
            }
        }
    }
    if args[1] == "hub" {
        hub(config).await
    } else if args[1] == "proxy" {
        proxy(config).await
    } else {
        Err(io::Error::other("invalid lab mode"))
    }
}
async fn hub(config: Config) -> io::Result<()> {
    let identity = Identity::ephemeral()?;
    std::fs::write(&config.certificate, identity.certificate.as_ref())?;
    let endpoint = identity.listen(config.server)?;
    let mut capabilities: HashMap<_, _> = config
        .peers
        .iter()
        .enumerate()
        .map(|(i, p)| Ok((hex::<32>(&p.secret)?, i)))
        .collect::<io::Result<_>>()?;
    let mut links = HashMap::new();
    let (send, mut recv) = mpsc::channel(64);
    let mut tasks: JoinSet<io::Result<()>> = JoinSet::new();
    let mut term = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    let mut windows: HashMap<(usize, RouteId), ReplayWindow> = HashMap::new();
    let mut sequences: HashMap<(usize, RouteId), u64> = HashMap::new();
    let routes: HashMap<_, _> = config
        .peers
        .iter()
        .enumerate()
        .flat_map(|(i, p)| {
            p.routes.iter().map(move |r| {
                Ok((
                    (i, RouteId(hex::<16>(&r.id)?)),
                    (r.target, RouteId(hex::<16>(&r.target_id)?)),
                ))
            })
        })
        .collect::<io::Result<_>>()?;
    emit(
        serde_json::json!({"event":"ready","limits":"Static private transport lab, no account/room proof"}),
    );
    let result = loop {
        tokio::select! {
            _=term.recv()=>break Ok(()),
            incoming=endpoint.accept()=>{
                let Some(incoming)=incoming else{break Ok(());};
                let connection=match incoming.await{Ok(c)=>c,Err(_)=>continue};
                let (link,owner)=match Link::accept_with(connection,|candidate|capabilities.remove(candidate)).await{Ok(value)=>value,Err(_)=>{emit(serde_json::json!({"event":"authorization_denied"}));continue;}};
                let remote=link.remote_address();let data=link.data();let send=send.clone();
                tasks.spawn(async move {loop {let packet=data.receive().await?;send.send((owner,packet)).await.map_err(|_|io::Error::other("lab receiver closed"))?;}});
                links.insert(owner,link);emit(serde_json::json!({"event":"authorized","owner":owner,"remote":remote}));
            },
            message=recv.recv()=>{
                let Some((owner,packet))=message else{break Ok(());};
                let Some(&(target,target_id))=routes.get(&(owner,packet.route))else{emit(serde_json::json!({"event":"route_denied","owner":owner}));continue;};
                if !windows.entry((owner,packet.route)).or_default().admit(packet.sequence){emit(serde_json::json!({"event":"replay_drop","owner":owner}));continue;}
                let Some(link)=links.get(&target)else{continue;};let sequence=sequences.entry((target,target_id)).or_default();let Some(next)=sequence.checked_add(1)else{break Err(io::Error::other("lab sequence exhausted"));};*sequence=next;
                let hash=Sha256::digest(&packet.native).iter().map(|b|format!("{b:02x}")).collect::<String>();let original=packet.native.iter().map(|b|format!("{b:02x}")).collect::<String>();
                if let Err(e)=link.send(&Packet {route:target_id,sequence:next,native:packet.native}){break Err(e);}
                emit(serde_json::json!({"event":"forwarded","owner":owner,"target":target,"route":packet.route.0,"sequence":packet.sequence,"target_sequence":next,"sha256":hash,"native_hex":original}));
            },
            ended=tasks.join_next(),if !tasks.is_empty()=>{
                match ended {Some(Ok(Ok(())))=>{},Some(Ok(Err(e)))=>{emit(serde_json::json!({"event":"peer_closed","error":e.to_string()}));},Some(Err(e))=>break Err(io::Error::other(e)),None=>{}}
            }
        }
    };
    for link in links.values() {
        link.close();
    }
    tasks.abort_all();
    while tasks.join_next().await.is_some() {}
    endpoint.close(0_u8.into(), b"lab closed");
    emit(serde_json::json!({"event":"stopped"}));
    result
}
async fn proxy(config: Config) -> io::Result<()> {
    let role = config
        .role
        .ok_or_else(|| io::Error::other("private lab role required"))?;
    let peer = config
        .peers
        .get(role)
        .ok_or_else(|| io::Error::other("lab role bound"))?;
    let bindings = peer
        .routes
        .iter()
        .map(|route| {
            Ok(Binding {
                route: RouteId(hex::<16>(&route.id)?),
                address: SocketAddr::from(([127, 0, 0, 1], route.port)),
            })
        })
        .collect::<io::Result<Vec<_>>>()?;
    let proxy = Proxy::bind(
        &bindings,
        config
            .native_ip
            .ok_or_else(|| io::Error::other("native source IP required"))?,
    )
    .await?;
    let endpoint = client(
        SocketAddr::from(([0, 0, 0, 0], 0)),
        rustls::pki_types::CertificateDer::from(std::fs::read(&config.certificate)?),
    )?;
    let link = Link::connect(
        &endpoint,
        config.server,
        &Secret::from_bytes(hex::<32>(&peer.secret)?),
    )
    .await?;
    emit(serde_json::json!({"event":"ready","owner":role}));
    let (stop, recv) = watch::channel(false);
    let mut task = tokio::spawn(proxy.run(link, recv));
    let mut term = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    let result = tokio::select! {value=&mut task=>value.map_err(io::Error::other)?,_=term.recv()=>{let _=stop.send(true);tokio::time::timeout(Duration::from_secs(2),&mut task).await.map_err(|_|io::Error::other("lab proxy shutdown deadline"))?.map_err(io::Error::other)?}};
    endpoint.close(0_u8.into(), b"lab closed");
    let stats = result?;
    emit(
        serde_json::json!({"event":"stopped","native_received":stats.native_received,"native_sent":stats.native_sent,"replay":stats.replay,"unknown_route":stats.unknown_route}),
    );
    Ok(())
}
