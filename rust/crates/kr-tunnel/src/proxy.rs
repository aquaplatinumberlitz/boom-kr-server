//! Loopback UDP companion. Static route snapshot foundation only; the serving
//! adapter must install/remove snapshots using account/room/round ownership.
//! This module neither discovers game routes nor authenticates a local process.
use crate::{Data, Link, MAX_NATIVE, Packet, ReplayWindow, RouteId};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    io,
    net::{IpAddr, SocketAddr},
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{
    net::UdpSocket,
    sync::{mpsc, watch},
    task::JoinSet,
};

pub struct Binding {
    pub route: RouteId,
    pub address: SocketAddr,
}
struct Edge {
    route: RouteId,
    socket: Arc<UdpSocket>,
}
pub struct Proxy {
    edges: Vec<Edge>,
    native_ip: IpAddr,
}
#[derive(Default, Debug)]
pub struct Stats {
    pub native_sent: usize,
    pub native_received: usize,
    pub unknown_route: usize,
    pub replay: usize,
    pub queue_full: usize,
    pub denied_native: usize,
    pub expired: usize,
}
impl Stats {
    fn add(&mut self, other: Self) {
        self.native_sent += other.native_sent;
        self.native_received += other.native_received;
        self.denied_native += other.denied_native;
        self.expired += other.expired;
        self.queue_full += other.queue_full;
    }
}
impl Proxy {
    /// Bind a complete pre-authorized snapshot before publishing endpoints.
    /// Only loopback listeners; native_ip is an explicit local source restriction.
    /// Zero ports ask the OS for bounded ephemeral local resources, never public
    /// relay/NAT mappings. Dropping a failed preparation releases every socket.
    pub async fn bind(bindings: &[Binding], native_ip: IpAddr) -> io::Result<Self> {
        if bindings.is_empty()
            || bindings.len() > 26
            || native_ip.is_unspecified()
            || native_ip.is_multicast()
        {
            return Err(io::Error::other("invalid companion snapshot"));
        }
        let mut routes = HashSet::new();
        let mut edges = Vec::new();
        for binding in bindings {
            if !binding.address.ip().is_loopback() || !routes.insert(binding.route) {
                return Err(io::Error::other(
                    "companion requires distinct routes and loopback listeners",
                ));
            }
            edges.push(Edge {
                route: binding.route,
                socket: Arc::new(UdpSocket::bind(binding.address).await?),
            });
        }
        Ok(Self { edges, native_ip })
    }
    pub fn bindings(&self) -> io::Result<Vec<Binding>> {
        self.edges
            .iter()
            .map(|edge| {
                Ok(Binding {
                    route: edge.route,
                    address: edge.socket.local_addr()?,
                })
            })
            .collect()
    }
    /// Own all child tasks, sockets and Link. No detached pumps or unbounded
    /// queues. Control-plane generation changes require replacing this owner.
    pub async fn run(self, link: Link, mut stop: watch::Receiver<bool>) -> io::Result<Stats> {
        let data = link.data();
        let (children_stop, child_stop) = watch::channel(false);
        let mut routes = HashMap::new();
        let mut tasks = JoinSet::new();
        let mut stats = Stats::default();
        for edge in self.edges {
            let (send, recv) = mpsc::channel(16);
            routes.insert(edge.route, (send, ReplayWindow::default()));
            tasks.spawn(pump(
                edge,
                data.clone(),
                recv,
                child_stop.clone(),
                self.native_ip,
            ));
        }
        let result = loop {
            if *stop.borrow() {
                break Ok(());
            }
            tokio::select! {
                value=stop.changed()=>{let _=value;break Ok(());},
                result=tasks.join_next()=>{
                    match result {
                        Some(Ok(Ok(part)))=>{stats.add(part);break Ok(());},
                        Some(Ok(Err(e)))=>break Err(e),
                        Some(Err(e))=>break Err(io::Error::other(e)),
                        None=>break Ok(()),
                    }
                },
                packet=data.receive()=>{
                    let packet=match packet {Ok(packet)=>packet,Err(e)=>break Err(e)};
                    let Some((send,window))=routes.get_mut(&packet.route)else{stats.unknown_route+=1;continue;};
                    if !window.admit(packet.sequence){stats.replay+=1;continue;}
                    if send.try_send(packet.native).is_err(){stats.queue_full+=1;}
                }
            }
        };
        let _ = children_stop.send(true);
        let drained = tokio::time::timeout(Duration::from_secs(1), async {
            while let Some(value) = tasks.join_next().await {
                if let Ok(Ok(part)) = value {
                    stats.add(part);
                }
            }
        })
        .await
        .is_ok();
        tasks.abort_all();
        while tasks.join_next().await.is_some() {}
        link.close();
        result?;
        if !drained {
            return Err(io::Error::other("companion cleanup deadline"));
        }
        Ok(stats)
    }
}
async fn pump(
    edge: Edge,
    data: Data,
    mut incoming: mpsc::Receiver<Vec<u8>>,
    mut stop: watch::Receiver<bool>,
    native_ip: IpAddr,
) -> io::Result<Stats> {
    let mut sequence = 0_u64;
    let mut native = None;
    let mut pending: VecDeque<(Instant, Vec<u8>)> = VecDeque::new();
    let mut stats = Stats::default();
    let mut buffer = [0; MAX_NATIVE + 1];
    loop {
        tokio::select! {
            value=stop.changed()=>{let _=value;return Ok(stats);},
            value=edge.socket.recv_from(&mut buffer)=>{
                let (n,from)=value?;
                if from.ip()!=native_ip||native.is_some_and(|old|old!=from)||n==0||n>MAX_NATIVE {
                    stats.denied_native+=1;continue;
                }
                native=Some(from);
                sequence=sequence.checked_add(1).ok_or_else(||io::Error::other("companion sequence exhausted"))?;
                data.send(&Packet {route:edge.route,sequence,native:buffer[..n].to_vec()})?;
                stats.native_received+=1;
                while let Some((at,bytes))=pending.pop_front() {
                    if at.elapsed()>=Duration::from_millis(500){stats.expired+=1;continue;}
                    edge.socket.send_to(&bytes,from).await?;stats.native_sent+=1;
                }
            },
            bytes=incoming.recv()=>{
                let Some(bytes)=bytes else{return Ok(stats);};
                if let Some(target)=native {edge.socket.send_to(&bytes,target).await?;stats.native_sent+=1;}
                else if pending.len()<16 {pending.push_back((Instant::now(),bytes));}
                else {stats.queue_full+=1;}
            }
        }
    }
}
