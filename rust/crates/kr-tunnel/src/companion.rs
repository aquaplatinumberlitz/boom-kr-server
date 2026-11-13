//! Dynamic client-side owner of loopback snapshots. All socket/queue resources
//! are scoped to this authenticated Link and retired on stop or disconnect.
use crate::{
    Link, control,
    proxy::{Proxy, Quarantine},
    router::{Binding, Scope},
};
use std::{
    collections::HashSet,
    io,
    net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4},
    time::Duration,
};
use tokio::{
    sync::{mpsc, watch},
    task::JoinHandle,
};
struct Active {
    scope: Scope,
    stop: watch::Sender<bool>,
    task: JoinHandle<io::Result<crate::proxy::Stats>>,
}
impl Active {
    async fn finish(mut self) -> io::Result<()> {
        let _ = self.stop.send(true);
        match tokio::time::timeout(Duration::from_secs(2), &mut self.task).await {
            Ok(result) => result.map_err(io::Error::other)?.map(|_| ()),
            Err(_) => {
                self.task.abort();
                let _ = self.task.await;
                Err(io::Error::other("companion snapshot stop deadline"))
            }
        }
    }
}
/// A finite per-session native endpoint budget is explicit. Retired sockets
/// stay bound, receive no forwarding and are released when this Link finishes.
/// Reaching the budget fails closed; it never reuses a native endpoint silently.
pub async fn run(
    mut link: Link,
    native_ip: IpAddr,
    port_budget: usize,
    mut stop: watch::Receiver<bool>,
) -> io::Result<()> {
    if !(26..=32768).contains(&port_budget) {
        return Err(io::Error::other("companion local port budget"));
    }
    let (mut send, mut recv) = link.take_control()?;
    let (commands, mut incoming) = mpsc::channel(4);
    let mut reader_stop = stop.clone();
    let reader = tokio::spawn(async move {
        loop {
            let command = tokio::select! {_=reader_stop.changed()=>return Ok::<(),io::Error>(()),value=control::read::<control::Server>(&mut recv)=>value?};
            tokio::select! {_=reader_stop.changed()=>return Ok(()),result=commands.send(command)=>if result.is_err(){return Ok(())}}
        }
    });
    let mut active: Option<Active> = None;
    let mut retired: Vec<Quarantine> = Vec::new();
    let mut used = 0;
    let mut scopes = HashSet::new();
    let mut health = tokio::time::interval(Duration::from_millis(100));
    let result=async {
        loop {
            if *stop.borrow(){return Ok(());}
            let command=tokio::select! {
                biased;
                _=stop.changed()=>return Ok(()),
                _=health.tick()=>{
                    if active.as_ref().is_some_and(|a|a.task.is_finished()){return Err(io::Error::other("companion data pump stopped"));}
                    continue;
                },
                value=incoming.recv()=>value.ok_or_else(||io::Error::other("peer control disconnected"))?
            };
            match command {
                control::Server::Retire {scope}=>{
                    if active.as_ref().is_some_and(|old|old.scope==scope){active.take().expect("checked active").finish().await?;}
                },
                control::Server::Configure {scope,routes}=>{
                    if scopes.contains(&scope)||routes.is_empty()||routes.len()>26||used+routes.len()>port_budget{return Err(io::Error::other("stale or exhausted companion assignment"));}
                    let mut slots=HashSet::new();let mut ids=HashSet::new();
                    if routes.iter().any(|row|row.slot>=26||!slots.insert(row.slot)||!ids.insert(row.route)){return Err(io::Error::other("duplicate companion assignment"));}
                    if let Some(old)=active.take(){old.finish().await?;}
                    let rows:Vec<_>=routes.iter().map(|route|crate::proxy::Binding {route:route.route,address:SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST,0))}).collect();
                    let proxy=Proxy::bind(&rows,native_ip).await?;let bindings=proxy.bindings()?;
                    let quarantine=proxy.quarantine();used+=quarantine.len();retired.push(quarantine);scopes.insert(scope);
                    let (halt,recv)=watch::channel(false);let data=link.data();let task=tokio::spawn(proxy.run_data(data,recv));active=Some(Active {scope,stop:halt,task});
                    let bindings=bindings.into_iter().map(|binding|match binding.address {SocketAddr::V4(address)=>Ok(Binding {route:binding.route,address}),_=>Err(io::Error::other("companion requires IPv4"))}).collect::<io::Result<Vec<_>>>()?;
                    tokio::time::timeout(Duration::from_secs(2),control::write(&mut send,&control::Client::Ready {scope,bindings})).await.map_err(|_|io::Error::other("companion readiness write deadline"))??;
                }
            }
        }
    }.await;
    // Abort only the single blocking control reader; every UDP child is stopped
    // and joined by its snapshot owner before retained sockets and Link drop.
    reader.abort();
    let _ = reader.await;
    let stopped = if let Some(old) = active.take() {
        old.finish().await
    } else {
        Ok(())
    };
    link.close();
    drop(retired);
    result.and(stopped)
}
